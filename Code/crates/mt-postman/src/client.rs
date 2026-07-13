//! Клиент (телефон, Этап 1). Connect к почтальону → регистрация overlay_addr
//! (RegHello/RegChallenge/RegProof, ML-DSA-65) → отправка RELAY, приём DELIVER/ACK.
//! Входящие фреймы отдаются через канал `recv()`.

use std::net::SocketAddr;

use quinn::{Connection, Endpoint};
use thiserror::Error;
use tokio::sync::mpsc;

use mt_crypto::{SecretKey, PUBLIC_KEY_SIZE};
use mt_overlay::challenge::{sign_registration, Nonce, NONCE_SIZE};
use mt_overlay::dedup::DedupWindow;
use mt_overlay::frame::{FrameType, MsgId, OverlayFrame};
use mt_overlay::prologue::{decode_reg_challenge, encode_reg_hello, encode_reg_proof};
use mt_overlay::{overlay_addr, OverlayAddr};
use mt_state::{derive_account_id, SUITE_MLDSA65};

use crate::config::{stand_client_config, ConfigError, STAND_SNI};
use crate::wire::{channel_hash, read_fixed, recv_frame, send_frame, write_fixed, WireError};

const REG_OK: u8 = 0x01;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("config: {0}")]
    Config(#[from] ConfigError),
    #[error("endpoint io: {0}")]
    Io(#[from] std::io::Error),
    #[error("connect: {0}")]
    Connect(#[from] quinn::ConnectError),
    #[error("connection: {0}")]
    Connection(#[from] quinn::ConnectionError),
    #[error("wire: {0}")]
    Wire(#[from] WireError),
    #[error("crypto: {0}")]
    Crypto(#[from] mt_crypto::CryptoError),
    #[error("malformed challenge from postman")]
    MalformedChallenge,
    #[error("postman rejected registration")]
    Rejected,
}

pub struct PostmanClient {
    conn: Connection,
    _endpoint: Endpoint, // держит транспорт живым, пока жив клиент
    overlay: OverlayAddr,
    incoming: mpsc::Receiver<OverlayFrame>,
}

impl PostmanClient {
    /// Подключиться к почтальону и зарегистрировать overlay_addr.
    pub async fn connect(
        server: SocketAddr,
        account_pubkey: [u8; PUBLIC_KEY_SIZE],
        account_sk: &SecretKey,
    ) -> Result<Self, ClientError> {
        let mut endpoint = Endpoint::client("0.0.0.0:0".parse().expect("bind any"))?;
        endpoint.set_default_client_config(stand_client_config()?);
        let conn = endpoint.connect(server, STAND_SNI)?.await?;

        // Рукопожатие по bi-потоку.
        let (mut send, mut recv) = conn.open_bi().await?;
        write_fixed(&mut send, &encode_reg_hello(&account_pubkey)).await?;

        let mut chal = [0u8; NONCE_SIZE];
        read_fixed(&mut recv, &mut chal).await?;
        let nonce: Nonce =
            decode_reg_challenge(&chal).map_err(|_| ClientError::MalformedChallenge)?;

        let ch = channel_hash(&conn)?;
        let account_id = derive_account_id(SUITE_MLDSA65, &account_pubkey);
        let overlay = overlay_addr(&account_id);
        let sig = sign_registration(account_sk, &overlay, &nonce, &ch)?;
        write_fixed(&mut send, &encode_reg_proof(sig.as_bytes())).await?;

        let mut ok = [0u8; 1];
        read_fixed(&mut recv, &mut ok).await?;
        if ok[0] != REG_OK {
            return Err(ClientError::Rejected);
        }

        // Приёмный таск: входящие uni-потоки от почтальона (DELIVER/ACK) → канал.
        let (tx, rx) = mpsc::channel(64);
        let rconn = conn.clone();
        tokio::spawn(async move {
            // Этап 1 шаг 4 (§396): дедуп входящих по msg_id скользящим окном.
            let mut dedup = DedupWindow::default();
            while let Ok(mut r) = rconn.accept_uni().await {
                match recv_frame(&mut r).await {
                    Ok(bytes) => {
                        if let Ok(f) = OverlayFrame::decode(&bytes) {
                            if !dedup.check_and_insert(&f.msg_id) {
                                continue; // повтор — отбросить
                            }
                            if tx.send(f).await.is_err() {
                                break;
                            }
                        }
                    },
                    Err(_) => continue,
                }
            }
        });

        Ok(Self {
            conn,
            _endpoint: endpoint,
            overlay,
            incoming: rx,
        })
    }

    pub fn overlay(&self) -> OverlayAddr {
        self.overlay
    }

    /// Отправить RELAY получателю dst с непрозрачным E2E-payload.
    pub async fn send_relay(
        &self,
        dst: OverlayAddr,
        msg_id: MsgId,
        payload: Vec<u8>,
    ) -> Result<(), ClientError> {
        self.send_frame_typed(FrameType::Relay, dst, msg_id, payload)
            .await
    }

    /// Ответить ACK отправителю dst по тому же msg_id (payload пуст).
    pub async fn send_ack(&self, dst: OverlayAddr, msg_id: MsgId) -> Result<(), ClientError> {
        self.send_frame_typed(FrameType::Ack, dst, msg_id, Vec::new())
            .await
    }

    async fn send_frame_typed(
        &self,
        frame_type: FrameType,
        dst: OverlayAddr,
        msg_id: MsgId,
        payload: Vec<u8>,
    ) -> Result<(), ClientError> {
        let f = OverlayFrame {
            frame_type,
            dst_overlay: dst,
            src_overlay: self.overlay,
            msg_id,
            payload,
        };
        let mut s = self.conn.open_uni().await?;
        send_frame(&mut s, &f.to_bytes()).await?;
        Ok(())
    }

    /// Дождаться следующего входящего фрейма (DELIVER или ACK) от почтальона.
    pub async fn recv(&mut self) -> Option<OverlayFrame> {
        self.incoming.recv().await
    }
}
