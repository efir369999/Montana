//! Почтальон — маршрутизация оверлей-фреймов между зарегистрированными соединениями
//! (Этап 1, механика шаги 0/3). Транспорт-агностичная логика: соединения абстрактны
//! (ConnId), интеграция с QUIC — отдельным слоем. Не consensus state.

use std::collections::HashMap;

use crate::challenge::{verify_registration, ChannelHash, Nonce};
use crate::frame::{FrameType, OverlayFrame};
use crate::OverlayAddr;
use mt_crypto::{Signature, PUBLIC_KEY_SIZE};

pub type ConnId = u64;

/// Куда почтальон направляет фрейм после разбора.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Route {
    /// Переслать живому соединению получателя как DELIVER.
    Deliver { conn: ConnId, frame: OverlayFrame },
    /// Получатель офлайн — во входящий буфер (Этап 2 store-and-forward).
    Buffer { frame: OverlayFrame },
    /// ACK назад отправителю (если его соединение живо).
    AckToSender { conn: ConnId, frame: OverlayFrame },
    /// Отбросить (получатель ACK не онлайн / нерелевантно).
    Drop,
}

#[derive(Default)]
pub struct Postman {
    // overlay_addr -> живое соединение (заполняется только после verify_registration)
    by_addr: HashMap<OverlayAddr, ConnId>,
    by_conn: HashMap<ConnId, OverlayAddr>,
}

impl Postman {
    pub fn new() -> Self {
        Self::default()
    }

    /// Шаг 0: зарегистрировать соединение после валидного RegProof.
    /// Возвращает подтверждённый overlay_addr или None (подпись/привязка неверны).
    ///
    /// Контракт транспортного слоя (D2): `nonce` — свежий CSPRNG на каждую регистрацию;
    /// `channel_hash` — привязка к текущему соединению (TLS-Exporter/Noise handshake-hash).
    /// Межсоединительный реплей закрыт `channel_hash` (R4); внутрисоединительный — свежим `nonce`.
    pub fn register(
        &mut self,
        conn: ConnId,
        account_pubkey: &[u8; PUBLIC_KEY_SIZE],
        nonce: &Nonce,
        channel_hash: &ChannelHash,
        sig: &Signature,
    ) -> Option<OverlayAddr> {
        let addr = verify_registration(account_pubkey, nonce, channel_hash, sig)?;
        if let Some(prev) = self.by_conn.insert(conn, addr) {
            if prev != addr {
                self.by_addr.remove(&prev);
            }
        }
        self.by_addr.insert(addr, conn);
        Some(addr)
    }

    pub fn deregister(&mut self, conn: ConnId) {
        if let Some(addr) = self.by_conn.remove(&conn) {
            if self.by_addr.get(&addr) == Some(&conn) {
                self.by_addr.remove(&addr);
            }
        }
    }

    pub fn is_registered(&self, conn: ConnId) -> bool {
        self.by_conn.contains_key(&conn)
    }

    /// Шаг 3: маршрут входящего фрейма. `from` — соединение-источник (для ACK/дедупа).
    /// src_overlay не аутентифицирован (спека): маршрутизируем строго по dst.
    pub fn route(&self, from: ConnId, frame: OverlayFrame) -> Route {
        match frame.frame_type {
            FrameType::Relay => match self.by_addr.get(&frame.dst_overlay) {
                Some(&conn) => Route::Deliver {
                    conn,
                    frame: OverlayFrame {
                        frame_type: FrameType::Deliver,
                        ..frame
                    },
                },
                None => Route::Buffer { frame },
            },
            // ACK от получателя назад отправителю по dst_overlay.
            FrameType::Ack => match self.by_addr.get(&frame.dst_overlay) {
                Some(&conn) => Route::AckToSender { conn, frame },
                None => Route::Buffer { frame },
            },
            // DELIVER — исходящий тип почтальона, входящим быть не должен.
            FrameType::Deliver => {
                let _ = from;
                Route::Drop
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::challenge::{sign_registration, NONCE_SIZE};
    use crate::{overlay_addr, OverlayAddr};
    use mt_crypto::{keypair_from_seed, PublicKey, SecretKey};
    use mt_state::{derive_account_id, SUITE_MLDSA65};

    fn ident(seed: u8) -> ([u8; PUBLIC_KEY_SIZE], SecretKey, OverlayAddr) {
        let (pk, sk): (PublicKey, SecretKey) = keypair_from_seed(&[seed; 32]).unwrap();
        let pkb = *pk.as_bytes();
        let addr = overlay_addr(&derive_account_id(SUITE_MLDSA65, &pkb));
        (pkb, sk, addr)
    }

    fn reg(p: &mut Postman, conn: ConnId, seed: u8) -> OverlayAddr {
        let (pkb, sk, _addr) = ident(seed);
        let nonce = [seed; NONCE_SIZE];
        let ch = [0xC0; 32];
        let sig = sign_registration(
            &sk,
            &overlay_addr(&derive_account_id(SUITE_MLDSA65, &pkb)),
            &nonce,
            &ch,
        )
        .unwrap();
        p.register(conn, &pkb, &nonce, &ch, &sig).expect("register")
    }

    #[test]
    fn relay_to_registered_becomes_deliver() {
        let mut p = Postman::new();
        let a = reg(&mut p, 1, 0xA1);
        let b = reg(&mut p, 2, 0xB2);
        let frame = OverlayFrame {
            frame_type: FrameType::Relay,
            dst_overlay: b,
            src_overlay: a,
            msg_id: [0x01; 16],
            payload: b"x".to_vec(),
        };
        match p.route(1, frame) {
            Route::Deliver { conn, frame } => {
                assert_eq!(conn, 2);
                assert_eq!(frame.frame_type, FrameType::Deliver);
            },
            other => panic!("expected Deliver, got {other:?}"),
        }
    }

    #[test]
    fn relay_to_unknown_buffers() {
        let mut p = Postman::new();
        let a = reg(&mut p, 1, 0xA1);
        let frame = OverlayFrame {
            frame_type: FrameType::Relay,
            dst_overlay: [0xEE; 32],
            src_overlay: a,
            msg_id: [0x01; 16],
            payload: b"x".to_vec(),
        };
        assert!(matches!(p.route(1, frame), Route::Buffer { .. }));
    }

    #[test]
    fn forged_registration_rejected_and_no_hijack() {
        let mut p = Postman::new();
        let (pkb, _sk, _addr) = ident(0xA1);
        let (_pk2, sk2, _a2) = ident(0xB2);
        // Подпись ключом B под адресом A — привязка не сойдётся.
        let nonce = [0x01; NONCE_SIZE];
        let ch = [0xC0; 32];
        let addr_a = overlay_addr(&derive_account_id(SUITE_MLDSA65, &pkb));
        let sig = sign_registration(&sk2, &addr_a, &nonce, &ch).unwrap();
        assert_eq!(p.register(9, &pkb, &nonce, &ch, &sig), None);
        assert!(!p.is_registered(9));
    }

    #[test]
    fn deregister_clears_routing() {
        let mut p = Postman::new();
        let _a = reg(&mut p, 1, 0xA1);
        let b = reg(&mut p, 2, 0xB2);
        p.deregister(2);
        let frame = OverlayFrame {
            frame_type: FrameType::Relay,
            dst_overlay: b,
            src_overlay: [0; 32],
            msg_id: [0x01; 16],
            payload: b"x".to_vec(),
        };
        assert!(matches!(p.route(1, frame), Route::Buffer { .. }));
    }
}
