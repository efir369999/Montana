//! Off-chain инбокс-хранилище почтальона (Этап 2): epoch_tag → осколки с TTL,
//! drop-on-delivery, per-tag rate-limit. Не consensus state ([P2P-1]).
//! E-3: принадлежность тега инкапсулирована — store знает account_id своих юзеров
//! и сам проверяет tag ∈ own (внешним булевым флагом забыть невозможно).

use std::collections::HashMap;

use crate::frame::{MsgId, MAX_PAYLOAD_LEN};
use crate::inbox::{epoch_tag_belongs, EpochTag, N_FETCH};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StoredShard {
    pub msg_id: MsgId,
    pub shard_index: u8,
    pub shard_total: u8,
    pub ct: Vec<u8>,
    pub expire_window: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DepositError {
    NotOwnTag,
    RateLimited,
    OversizeShard,
}

pub const PER_TAG_PER_WINDOW_QUOTA: usize = 64;

#[derive(Default)]
pub struct InboxStore {
    own_account_ids: Vec<[u8; 32]>,
    items: HashMap<EpochTag, Vec<StoredShard>>,
    rate: HashMap<(EpochTag, u64), usize>,
}

impl InboxStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Зарегистрировать юзера этого почтальона (из RegHello Этапа 1 → account_id).
    pub fn register_own(&mut self, account_id: [u8; 32]) {
        if !self.own_account_ids.contains(&account_id) {
            self.own_account_ids.push(account_id);
        }
    }

    /// tag принадлежит какому-то own юзеру за окно-диапазон вокруг current_window
    /// (депозит может нести окно из будущего/прошлого в пределах N_FETCH).
    fn is_own_tag(&self, tag: &EpochTag, current_window: u64) -> bool {
        let lo = current_window.saturating_sub(N_FETCH);
        let hi = current_window.saturating_add(N_FETCH);
        self.own_account_ids
            .iter()
            .any(|acc| epoch_tag_belongs(acc, tag, lo, hi))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn deposit(
        &mut self,
        tag: EpochTag,
        current_window: u64,
        msg_id: MsgId,
        shard_index: u8,
        shard_total: u8,
        ttl_windows: u32,
        ct: Vec<u8>,
    ) -> Result<(), DepositError> {
        if !self.is_own_tag(&tag, current_window) {
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

    pub fn fetch(&self, tag: &EpochTag) -> Vec<StoredShard> {
        self.items.get(tag).cloned().unwrap_or_default()
    }

    pub fn drop_delivered(&mut self, tag: &EpochTag, msg_id: &MsgId) {
        if let Some(v) = self.items.get_mut(tag) {
            v.retain(|s| &s.msg_id != msg_id);
            if v.is_empty() {
                self.items.remove(tag);
            }
        }
    }

    pub fn prune(&mut self, current_window: u64) {
        self.items.retain(|_, v| {
            v.retain(|s| s.expire_window >= current_window);
            !v.is_empty()
        });
        self.rate.retain(|(_, w), _| *w + N_FETCH >= current_window);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inbox::epoch_tag;

    const ACC: [u8; 32] = [0x11; 32];

    fn store_with_own() -> InboxStore {
        let mut s = InboxStore::new();
        s.register_own(ACC);
        s
    }

    #[test]
    fn deposit_fetch_drop_own_tag() {
        let mut s = store_with_own();
        let tag = epoch_tag(&ACC, 100);
        s.deposit(tag, 100, [1; 16], 0, 1, 240, vec![1; 64])
            .unwrap();
        assert_eq!(s.fetch(&tag).len(), 1);
        s.drop_delivered(&tag, &[1; 16]);
        assert_eq!(s.fetch(&tag).len(), 0);
    }

    #[test]
    fn reject_foreign_tag_encapsulated() {
        // E-3: тег постороннего (не own) отвергается store-ом изнутри, без внешнего флага.
        let mut s = store_with_own();
        let foreign = epoch_tag(&[0x99; 32], 100);
        assert_eq!(
            s.deposit(foreign, 100, [1; 16], 0, 1, 240, vec![1; 64]),
            Err(DepositError::NotOwnTag)
        );
    }

    #[test]
    fn rate_limit_per_tag_window() {
        let mut s = store_with_own();
        let tag = epoch_tag(&ACC, 100);
        for i in 0..PER_TAG_PER_WINDOW_QUOTA {
            assert!(s
                .deposit(tag, 100, [i as u8; 16], 0, 1, 240, vec![0; 8])
                .is_ok());
        }
        assert_eq!(
            s.deposit(tag, 100, [99; 16], 0, 1, 240, vec![0; 8]),
            Err(DepositError::RateLimited)
        );
    }

    #[test]
    fn ttl_prune_removes_expired() {
        let mut s = store_with_own();
        let tag = epoch_tag(&ACC, 100);
        s.deposit(tag, 100, [1; 16], 0, 1, 10, vec![1; 8]).unwrap();
        s.prune(105);
        assert_eq!(s.fetch(&tag).len(), 1);
        s.prune(111);
        assert_eq!(s.fetch(&tag).len(), 0);
    }
}
