//! Этап 7 — Лестница пробуждения (P2P Network spec). Byte-exact форматы пробуждения
//! WakeInline (ступени 1–3, Apple вне контура) / WakeHandle (ступень 4, APNs/FCM),
//! реестр account_id↔wake_handle у почтальона (R5), арбитр ступеней. Форматы
//! никогда не несут контент или отправителя.

use std::collections::BTreeMap;

use mt_crypto::HASH_SIZE;
use thiserror::Error;

/// recv_id очереди (Этап 2, mt-postman QueueId) — 32 B.
pub const RECV_ID_LEN: usize = HASH_SIZE;
/// account_id — публичный идентификатор личности (SSOT mt-state::derive_account_id) — 32 B.
pub const ACCOUNT_ID_LEN: usize = HASH_SIZE;
/// Непрозрачный дескриптор пробуждения ступени 4 — несвязуем с recv_id (R5).
pub const WAKE_HANDLE_LEN: usize = 16;
/// WakeInline wire-размер: recv_id 32 + window 8.
pub const WAKE_INLINE_LEN: usize = RECV_ID_LEN + 8;
/// WakeHandle wire-размер: wake_handle 16 + window 8.
pub const WAKE_HANDLE_MSG_LEN: usize = WAKE_HANDLE_LEN + 8;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum WakeError {
    #[error("truncated: expected {expected}, got {got}")]
    Truncated { expected: usize, got: usize },
    #[error("trailing bytes: {0}")]
    TrailingBytes(usize),
    #[error("csprng failure")]
    Csprng,
}

fn check_len(got: usize, expected: usize) -> Result<(), WakeError> {
    match got.cmp(&expected) {
        std::cmp::Ordering::Less => Err(WakeError::Truncated { expected, got }),
        std::cmp::Ordering::Greater => Err(WakeError::TrailingBytes(got - expected)),
        std::cmp::Ordering::Equal => Ok(()),
    }
}

/// Пробуждение ступеней 1–3: почтальон → получатель напрямую (Apple вне контура),
/// адрес очереди открыт получателю (не Apple).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct WakeInline {
    pub recv_id: [u8; RECV_ID_LEN],
    pub window: u64,
}

impl WakeInline {
    pub fn encode(&self) -> [u8; WAKE_INLINE_LEN] {
        let mut out = [0u8; WAKE_INLINE_LEN];
        out[..RECV_ID_LEN].copy_from_slice(&self.recv_id);
        out[RECV_ID_LEN..].copy_from_slice(&self.window.to_le_bytes());
        out
    }

    pub fn decode(input: &[u8]) -> Result<Self, WakeError> {
        check_len(input.len(), WAKE_INLINE_LEN)?;
        let mut recv_id = [0u8; RECV_ID_LEN];
        recv_id.copy_from_slice(&input[..RECV_ID_LEN]);
        let window = u64::from_le_bytes(input[RECV_ID_LEN..WAKE_INLINE_LEN].try_into().unwrap());
        Ok(Self { recv_id, window })
    }
}

/// Пробуждение ступени 4: push-шлюз → APNs/FCM. Шлюз и Apple видят ТОЛЬКО wake_handle
/// (не recv_id) — развязка адреса очереди (§11).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct WakeHandle {
    pub wake_handle: [u8; WAKE_HANDLE_LEN],
    pub window: u64,
}

impl WakeHandle {
    pub fn encode(&self) -> [u8; WAKE_HANDLE_MSG_LEN] {
        let mut out = [0u8; WAKE_HANDLE_MSG_LEN];
        out[..WAKE_HANDLE_LEN].copy_from_slice(&self.wake_handle);
        out[WAKE_HANDLE_LEN..].copy_from_slice(&self.window.to_le_bytes());
        out
    }

    pub fn decode(input: &[u8]) -> Result<Self, WakeError> {
        check_len(input.len(), WAKE_HANDLE_MSG_LEN)?;
        let mut wake_handle = [0u8; WAKE_HANDLE_LEN];
        wake_handle.copy_from_slice(&input[..WAKE_HANDLE_LEN]);
        let window = u64::from_le_bytes(
            input[WAKE_HANDLE_LEN..WAKE_HANDLE_MSG_LEN]
                .try_into()
                .unwrap(),
        );
        Ok(Self {
            wake_handle,
            window,
        })
    }
}

/// Реестр account_id↔wake_handle у почтальона (R5). Почтальон знает account_id своего
/// юзера; шлюз держит лишь wake_handle↔device_token и не видит account_id/recv_id.
/// wake_handle генерируется из OS CSPRNG — несвязуем с recv_id по построению.
#[derive(Default)]
pub struct WakeRegistry {
    forward: BTreeMap<[u8; ACCOUNT_ID_LEN], [u8; WAKE_HANDLE_LEN]>,
    reverse: BTreeMap<[u8; WAKE_HANDLE_LEN], [u8; ACCOUNT_ID_LEN]>,
}

impl WakeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Идемпотентная регистрация: повтор для того же account_id возвращает существующий
    /// handle (не плодит записи). Новый handle — 16 B OS CSPRNG, коллизия исключена.
    pub fn register(
        &mut self,
        account_id: [u8; ACCOUNT_ID_LEN],
    ) -> Result<[u8; WAKE_HANDLE_LEN], WakeError> {
        if let Some(h) = self.forward.get(&account_id) {
            return Ok(*h);
        }
        let mut handle = [0u8; WAKE_HANDLE_LEN];
        loop {
            getrandom::getrandom(&mut handle).map_err(|_| WakeError::Csprng)?;
            if !self.reverse.contains_key(&handle) {
                break;
            }
        }
        self.forward.insert(account_id, handle);
        self.reverse.insert(handle, account_id);
        Ok(handle)
    }

    pub fn handle_of(&self, account_id: &[u8; ACCOUNT_ID_LEN]) -> Option<[u8; WAKE_HANDLE_LEN]> {
        self.forward.get(account_id).copied()
    }

    /// Почтальон резолвит handle→account_id при пробуждении ступени 4 (спрашивает свой
    /// реестр, какие recv_id ждут для этого account_id).
    pub fn account_of(&self, handle: &[u8; WAKE_HANDLE_LEN]) -> Option<[u8; ACCOUNT_ID_LEN]> {
        self.reverse.get(handle).copied()
    }

    pub fn len(&self) -> usize {
        self.forward.len()
    }

    pub fn is_empty(&self) -> bool {
        self.forward.is_empty()
    }
}

/// Четыре ступени пробуждения (§10), по убыванию суверенности. Номер = приоритет
/// (меньше — выше суверенность; Apple лишь ступень 4).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WakeRung {
    LiveTunnel = 1,
    IBeaconHome = 2,
    UnlockSync = 3,
    PushGateway = 4,
}

impl WakeRung {
    /// Ступени 1–3 несут WakeInline (Apple вне контура); ступень 4 — WakeHandle.
    pub fn carries_inline(&self) -> bool {
        !matches!(self, WakeRung::PushGateway)
    }
}

/// Арбитр ступеней: высшая доступная первой (1–3 бесплатные и суверенные; APNs/FCM —
/// лишь когда ни одна из 1–3 не сработала). Ступень 4 — гарантийный слой, всегда доступна.
pub fn select_rung(live_tunnel: bool, ibeacon_home: bool, unlock_sync: bool) -> WakeRung {
    if live_tunnel {
        WakeRung::LiveTunnel
    } else if ibeacon_home {
        WakeRung::IBeaconHome
    } else if unlock_sync {
        WakeRung::UnlockSync
    } else {
        WakeRung::PushGateway
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wake_inline_kat() {
        let mut recv_id = [0u8; RECV_ID_LEN];
        for (i, b) in recv_id.iter_mut().enumerate() {
            *b = i as u8;
        }
        let w = WakeInline { recv_id, window: 1 };
        let enc = w.encode();
        assert_eq!(enc.len(), WAKE_INLINE_LEN);
        let expect =
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f0100000000000000";
        assert_eq!(hex::encode(enc), expect);
        assert_eq!(WakeInline::decode(&enc).unwrap(), w);
    }

    #[test]
    fn wake_handle_kat() {
        let w = WakeHandle {
            wake_handle: [0xaa; WAKE_HANDLE_LEN],
            window: 0x0102,
        };
        let enc = w.encode();
        assert_eq!(enc.len(), WAKE_HANDLE_MSG_LEN);
        let expect = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa0201000000000000";
        assert_eq!(hex::encode(enc), expect);
        assert_eq!(WakeHandle::decode(&enc).unwrap(), w);
    }

    #[test]
    fn decode_truncated() {
        assert_eq!(
            WakeInline::decode(&[0u8; 39]),
            Err(WakeError::Truncated {
                expected: 40,
                got: 39
            })
        );
        assert_eq!(
            WakeHandle::decode(&[0u8; 23]),
            Err(WakeError::Truncated {
                expected: 24,
                got: 23
            })
        );
    }

    #[test]
    fn decode_trailing() {
        assert_eq!(
            WakeInline::decode(&[0u8; 41]),
            Err(WakeError::TrailingBytes(1))
        );
        assert_eq!(
            WakeHandle::decode(&[0u8; 25]),
            Err(WakeError::TrailingBytes(1))
        );
    }

    #[test]
    fn registry_idempotent_and_bijective() {
        let mut reg = WakeRegistry::new();
        let acc = [7u8; ACCOUNT_ID_LEN];
        let h1 = reg.register(acc).unwrap();
        let h2 = reg.register(acc).unwrap();
        assert_eq!(h1, h2);
        assert_eq!(reg.len(), 1);
        assert_eq!(reg.handle_of(&acc), Some(h1));
        assert_eq!(reg.account_of(&h1), Some(acc));
        let acc2 = [9u8; ACCOUNT_ID_LEN];
        let h3 = reg.register(acc2).unwrap();
        assert_ne!(h1, h3);
        assert_eq!(reg.len(), 2);
    }

    #[test]
    fn handle_not_derived_from_account() {
        let mut reg = WakeRegistry::new();
        let acc = [0xABu8; ACCOUNT_ID_LEN];
        let h = reg.register(acc).unwrap();
        assert_ne!(&h[..], &acc[..WAKE_HANDLE_LEN]);
    }

    #[test]
    fn arbiter_priority() {
        assert_eq!(select_rung(true, true, true), WakeRung::LiveTunnel);
        assert_eq!(select_rung(false, true, true), WakeRung::IBeaconHome);
        assert_eq!(select_rung(false, false, true), WakeRung::UnlockSync);
        assert_eq!(select_rung(false, false, false), WakeRung::PushGateway);
    }

    #[test]
    fn rung_format_binding() {
        assert!(WakeRung::LiveTunnel.carries_inline());
        assert!(WakeRung::IBeaconHome.carries_inline());
        assert!(WakeRung::UnlockSync.carries_inline());
        assert!(!WakeRung::PushGateway.carries_inline());
    }
}
