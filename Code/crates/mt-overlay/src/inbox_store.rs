//! Off-chain инбокс-хранилище почтальона (Этап 2): epoch_tag → осколки с TTL,
//! drop-on-delivery, per-tag rate-limit. Не consensus state ([P2P-1]).

use std::collections::HashMap;

use crate::frame::{MsgId, MAX_PAYLOAD_LEN};
use crate::inbox::EpochTag;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StoredShard {
    pub msg_id: MsgId,
    pub shard_index: u8,
    pub shard_total: u8,
    pub ct: Vec<u8>,
    pub expire_window: u64, // окно депозита + ttl_windows
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DepositError {
    NotOwnTag,     // тег не принадлежит юзерам этого почтальона
    RateLimited,   // превышена квота на тег в окне
    OversizeShard, // осколок больше защитного cap
}

pub const PER_TAG_PER_WINDOW_QUOTA: usize = 64;

#[derive(Default)]
pub struct InboxStore {
    items: HashMap<EpochTag, Vec<StoredShard>>,
    // (epoch_tag, window) -> число депозитов в этом окне (rate-limit)
    rate: HashMap<(EpochTag, u64), usize>,
}

impl InboxStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Депозит осколка. `is_own_tag` — почтальон уже проверил принадлежность тега
    /// своим юзерам (inbox::epoch_tag_belongs) до вызова.
    #[allow(clippy::too_many_arguments)]
    pub fn deposit(
        &mut self,
        is_own_tag: bool,
        tag: EpochTag,
        current_window: u64,
        msg_id: MsgId,
        shard_index: u8,
        shard_total: u8,
        ttl_windows: u32,
        ct: Vec<u8>,
    ) -> Result<(), DepositError> {
        if !is_own_tag {
            return Err(DepositError::NotOwnTag);
        }
        if ct.len() > MAX_PAYLOAD_LEN {
            return Err(DepositError::OversizeShard);
        }
        let counter = self.rate.entry((tag, current_window)).or_insert(0);
        if *counter >= PER_TAG_PER_WINDOW_QUOTA {
            return Err(DepositError::RateLimited);
        }
        *counter += 1;
        self.items.entry(tag).or_default().push(StoredShard {
            msg_id,
            shard_index,
            shard_total,
            ct,
            expire_window: current_window.saturating_add(ttl_windows as u64),
        });
        Ok(())
    }

    /// Фетч всех осколков по тегу (после verify_fetch + ownership). НЕ удаляет —
    /// удаление по явному drop_delivered (drop-on-delivery после reassembly у клиента).
    pub fn fetch(&self, tag: &EpochTag) -> Vec<StoredShard> {
        self.items.get(tag).cloned().unwrap_or_default()
    }

    /// drop-on-delivery: клиент собрал сообщение → удалить все осколки этого msg_id.
    pub fn drop_delivered(&mut self, tag: &EpochTag, msg_id: &MsgId) {
        if let Some(v) = self.items.get_mut(tag) {
            v.retain(|s| &s.msg_id != msg_id);
            if v.is_empty() {
                self.items.remove(tag);
            }
        }
    }

    /// TTL-прунинг: удалить осколки с expire_window < current_window.
    pub fn prune(&mut self, current_window: u64) {
        self.items.retain(|_, v| {
            v.retain(|s| s.expire_window >= current_window);
            !v.is_empty()
        });
        self.rate
            .retain(|(_, w), _| *w + crate::inbox::N_FETCH >= current_window);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dep(store: &mut InboxStore, tag: EpochTag, w: u64, id: u8) -> Result<(), DepositError> {
        store.deposit(true, tag, w, [id; 16], 0, 1, 240, vec![id; 64])
    }

    #[test]
    fn deposit_fetch_drop() {
        let mut s = InboxStore::new();
        let tag = [0xAA; 16];
        dep(&mut s, tag, 100, 1).unwrap();
        assert_eq!(s.fetch(&tag).len(), 1);
        s.drop_delivered(&tag, &[1; 16]);
        assert_eq!(s.fetch(&tag).len(), 0);
    }

    #[test]
    fn reject_foreign_tag() {
        let mut s = InboxStore::new();
        assert_eq!(
            s.deposit(false, [0; 16], 100, [1; 16], 0, 1, 240, vec![1; 64]),
            Err(DepositError::NotOwnTag)
        );
    }

    #[test]
    fn rate_limit_per_tag_window() {
        let mut s = InboxStore::new();
        let tag = [0xBB; 16];
        for i in 0..PER_TAG_PER_WINDOW_QUOTA {
            assert!(s
                .deposit(true, tag, 100, [i as u8; 16], 0, 1, 240, vec![0; 8])
                .is_ok());
        }
        assert_eq!(
            s.deposit(true, tag, 100, [99; 16], 0, 1, 240, vec![0; 8]),
            Err(DepositError::RateLimited)
        );
        // другое окно — квота сбрасывается
        assert!(s
            .deposit(true, tag, 101, [0; 16], 0, 1, 240, vec![0; 8])
            .is_ok());
    }

    #[test]
    fn ttl_prune_removes_expired() {
        let mut s = InboxStore::new();
        let tag = [0xCC; 16];
        s.deposit(true, tag, 100, [1; 16], 0, 1, 10, vec![1; 8])
            .unwrap(); // expire 110
        s.prune(105);
        assert_eq!(s.fetch(&tag).len(), 1);
        s.prune(111);
        assert_eq!(s.fetch(&tag).len(), 0);
    }
}
