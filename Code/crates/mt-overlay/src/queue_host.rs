//! Host-store MUQ (Этап 2): queue-таблица (recv_id→Queue, send_id→recv_id) + буфер
//! осколков, двуххоп-депозит, выборка. Off-chain ([P2P-1]). Механика TTL/drop
//! наследует Этап-2-store, ключ буфера = recv_id.

use std::collections::{HashMap, HashSet};

use mt_crypto::Signature;

use crate::challenge::ChannelHash;
use crate::frame::{MsgId, MAX_PAYLOAD_LEN};
use crate::muq::{
    verify_deposit, verify_subscribe, HostDeposit, Queue, QueueId, QueueItem, RELAY_CHANNEL_MARKER,
};

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
    NoQueue,      // send_id не соответствует ни одной очереди
    Unauthorized, // secured-очередь, подпись send_key не прошла
    QuotaFull,    // буфер очереди достиг quota
    OversizeShard,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SubscribeError {
    NoQueue,
    BadSig, // подпись выборки не прошла (recv_pubkey / channel_hash)
    Replay, // двуххоп-выборка: nonce уже использован (украденный QueueSubscribe непереносим)
}

#[derive(Default)]
pub struct QueueHost {
    queues: HashMap<QueueId, Queue>,            // recv_id → Queue
    send_route: HashMap<QueueId, QueueId>,      // send_id → recv_id
    buffer: HashMap<QueueId, Vec<StoredShard>>, // recv_id → осколки
    seen_sub_nonces: HashMap<QueueId, HashSet<crate::challenge::Nonce>>, // anti-replay relay-выборки
}

impl QueueHost {
    pub fn new() -> Self {
        Self::default()
    }

    /// Хост создаёт очередь (recv_id/send_id независимы; получатель прислал recv_pubkey).
    pub fn register_queue(&mut self, q: Queue) {
        self.send_route.insert(q.send_id, q.recv_id);
        self.queues.insert(q.recv_id, q);
    }

    /// Двуххоп-депозит: proxy принёс распечатанный HostDeposit. Маршрут send_id→recv_id
    /// держит только хост (несвязываемость). secured → verify send_key.
    pub fn deposit(&mut self, hd: &HostDeposit, current_window: u64) -> Result<(), DepositError> {
        let recv_id = *self
            .send_route
            .get(&hd.send_id)
            .ok_or(DepositError::NoQueue)?;
        let q = self.queues.get(&recv_id).ok_or(DepositError::NoQueue)?;
        if let Some(send_pubkey) = &q.send_pubkey {
            let sig = Signature::from_slice(&hd.sig).ok_or(DepositError::Unauthorized)?;
            if !verify_deposit(send_pubkey, &hd.send_id, &hd.msg_id, &hd.nonce, &sig) {
                return Err(DepositError::Unauthorized);
            }
        }
        if hd.ct.len() > MAX_PAYLOAD_LEN {
            return Err(DepositError::OversizeShard);
        }
        let buf = self.buffer.entry(recv_id).or_default();
        if buf.len() as u32 >= q.quota {
            return Err(DepositError::QuotaFull);
        }
        buf.push(StoredShard {
            msg_id: hd.msg_id,
            shard_index: hd.shard_index,
            shard_total: hd.shard_total,
            ct: hd.ct.clone(),
            expire_window: current_window.saturating_add(hd.ttl_windows as u64),
        });
        Ok(())
    }

    /// Выборка: подпись recv_key против ХРАНИМОГО recv_pubkey + channel_hash (E-2/F3).
    /// Возвращает срез осколков (без clone).
    pub fn subscribe(
        &self,
        recv_id: &QueueId,
        nonce: &crate::challenge::Nonce,
        channel_hash: &ChannelHash,
        sig: &Signature,
    ) -> Result<&[StoredShard], SubscribeError> {
        let q = self.queues.get(recv_id).ok_or(SubscribeError::NoQueue)?;
        if !verify_subscribe(&q.recv_pubkey, recv_id, nonce, channel_hash, sig) {
            return Err(SubscribeError::BadSig);
        }
        Ok(self.buffer.get(recv_id).map(Vec::as_slice).unwrap_or(&[]))
    }

    /// Двуххоп-выборка через курьер: channel_hash отсутствует (нет прямого канала B↔host),
    /// поэтому подпись проверяется с RELAY_CHANNEL_MARKER (0×32), а anti-replay несёт
    /// nonce-tracking per recv_id — украденный QueueSubscribe непереносим (nonce одноразов).
    /// Возвращает копию осколков (не borrow — вызывающий дропает доставленное отдельно).
    pub fn subscribe_relay(
        &mut self,
        recv_id: &QueueId,
        nonce: &crate::challenge::Nonce,
        sig: &Signature,
    ) -> Result<Vec<QueueItem>, SubscribeError> {
        let recv_pubkey = self
            .queues
            .get(recv_id)
            .ok_or(SubscribeError::NoQueue)?
            .recv_pubkey;
        if !verify_subscribe(&recv_pubkey, recv_id, nonce, &RELAY_CHANNEL_MARKER, sig) {
            return Err(SubscribeError::BadSig);
        }
        if !self
            .seen_sub_nonces
            .entry(*recv_id)
            .or_default()
            .insert(*nonce)
        {
            return Err(SubscribeError::Replay);
        }
        Ok(self.buffer_of(recv_id))
    }

    pub fn drop_delivered(&mut self, recv_id: &QueueId, msg_id: &MsgId) {
        if let Some(v) = self.buffer.get_mut(recv_id) {
            v.retain(|s| &s.msg_id != msg_id);
        }
    }

    pub fn prune(&mut self, current_window: u64) {
        for v in self.buffer.values_mut() {
            v.retain(|s| s.expire_window >= current_window);
        }
        self.buffer.retain(|_, v| !v.is_empty());
    }

    pub fn buffer_of(&self, recv_id: &QueueId) -> Vec<QueueItem> {
        self.buffer
            .get(recv_id)
            .map(|v| {
                v.iter()
                    .map(|s| QueueItem {
                        msg_id: s.msg_id,
                        shard_index: s.shard_index,
                        shard_total: s.shard_total,
                        ct: s.ct.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::muq::sign_deposit;
    use mt_crypto::{keypair_from_seed, PublicKey, SecretKey, PUBLIC_KEY_SIZE};

    fn kp(seed: u8) -> ([u8; PUBLIC_KEY_SIZE], SecretKey) {
        let (pk, sk): (PublicKey, SecretKey) = keypair_from_seed(&[seed; 32]).unwrap();
        (*pk.as_bytes(), sk)
    }

    #[test]
    fn relay_subscribe_nonce_replay_rejected() {
        use crate::muq::sign_subscribe_relay;
        let (rpk, rsk) = kp(0x11);
        let mut host = QueueHost::new();
        host.register_queue(Queue {
            recv_id: [0x71; 32],
            send_id: [0x51; 32],
            recv_pubkey: rpk,
            send_pubkey: None,
            rotation_epoch: 1000,
            quota: 64,
        });
        let recv_id = [0x71u8; 32];
        let nonce = [0x33u8; 16];
        let sig = sign_subscribe_relay(&rsk, &recv_id, &nonce).unwrap();
        // первая выборка — ok
        assert!(host.subscribe_relay(&recv_id, &nonce, &sig).is_ok());
        // повтор ТОГО ЖЕ nonce — Replay (украденный QueueSubscribe непереносим)
        assert_eq!(
            host.subscribe_relay(&recv_id, &nonce, &sig),
            Err(SubscribeError::Replay)
        );
    }

    #[test]
    fn deposit_via_route_then_subscribe() {
        let (rpk, _rsk) = kp(0x11);
        let (spk, ssk) = kp(0x22);
        let mut host = QueueHost::new();
        let q = Queue {
            recv_id: [0x71; 32],
            send_id: [0x51; 32],
            recv_pubkey: rpk,
            send_pubkey: Some(spk),
            rotation_epoch: 1000,
            quota: 64,
        };
        host.register_queue(q);

        let nonce = [0x07; 16];
        let sig = sign_deposit(&ssk, &[0x51; 32], &[0x5A; 16], &nonce).unwrap();
        let hd = HostDeposit {
            send_id: [0x51; 32],
            msg_id: [0x5A; 16],
            ttl_windows: 240,
            shard_index: 0,
            shard_total: 1,
            nonce,
            ct: vec![0xCC; 64],
            sig: sig.as_bytes().to_vec(),
        };
        host.deposit(&hd, 100).unwrap();
        assert_eq!(host.buffer_of(&[0x71; 32]).len(), 1);
    }

    #[test]
    fn foreign_send_id_no_queue() {
        let mut host = QueueHost::new();
        let hd = HostDeposit {
            send_id: [0x99; 32],
            msg_id: [1; 16],
            ttl_windows: 1,
            shard_index: 0,
            shard_total: 1,
            nonce: [0; 16],
            ct: vec![1; 8],
            sig: Vec::new(),
        };
        assert_eq!(host.deposit(&hd, 100), Err(DepositError::NoQueue));
    }

    #[test]
    fn unauthorized_deposit_rejected() {
        let (rpk, _) = kp(0x11);
        let (spk, _ssk) = kp(0x22);
        let (_epk, esk) = kp(0x33); // чужак
        let mut host = QueueHost::new();
        host.register_queue(Queue {
            recv_id: [0x71; 32],
            send_id: [0x51; 32],
            recv_pubkey: rpk,
            send_pubkey: Some(spk),
            rotation_epoch: 1000,
            quota: 64,
        });
        let nonce = [0x07; 16];
        let bad = sign_deposit(&esk, &[0x51; 32], &[0x5A; 16], &nonce).unwrap();
        let hd = HostDeposit {
            send_id: [0x51; 32],
            msg_id: [0x5A; 16],
            ttl_windows: 1,
            shard_index: 0,
            shard_total: 1,
            nonce,
            ct: vec![1; 8],
            sig: bad.as_bytes().to_vec(),
        };
        assert_eq!(host.deposit(&hd, 100), Err(DepositError::Unauthorized));
    }

    #[test]
    fn quota_and_ttl() {
        let (rpk, _) = kp(0x11);
        let mut host = QueueHost::new();
        host.register_queue(Queue {
            recv_id: [0x71; 32],
            send_id: [0x51; 32],
            recv_pubkey: rpk,
            send_pubkey: None,
            rotation_epoch: 1000,
            quota: 2,
        });
        for i in 0..2u8 {
            let hd = HostDeposit {
                send_id: [0x51; 32],
                msg_id: [i; 16],
                ttl_windows: 10,
                shard_index: 0,
                shard_total: 1,
                nonce: [0; 16],
                ct: vec![i; 8],
                sig: Vec::new(),
            };
            host.deposit(&hd, 100).unwrap();
        }
        let over = HostDeposit {
            send_id: [0x51; 32],
            msg_id: [9; 16],
            ttl_windows: 10,
            shard_index: 0,
            shard_total: 1,
            nonce: [0; 16],
            ct: vec![9; 8],
            sig: Vec::new(),
        };
        assert_eq!(host.deposit(&over, 100), Err(DepositError::QuotaFull));
        host.prune(111);
        assert_eq!(host.buffer_of(&[0x71; 32]).len(), 0);
    }
}
