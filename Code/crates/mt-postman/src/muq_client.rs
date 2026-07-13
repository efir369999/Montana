//! MUQ-клиент (Этап 2). Подключается к queue-host / entry-proxy БЕЗ overlay-регистрации
//! Этапа 1 (несвязываемость — узел видит эфемерный ключ очереди, не account_id).
//! Получатель: register_queue + subscribe; отправитель: deposit_via_proxy (двуххоп).

use std::net::SocketAddr;

use quinn::{Connection, Endpoint};

use mt_crypto::SecretKey;
use mt_overlay::challenge::CHANNEL_HASH_SIZE;
use mt_overlay::muq::{sign_subscribe, ProxyForward, Queue, QueueId, QueueResp, QueueSubscribe};

use crate::client::ClientError;
use crate::config::{stand_client_config, STAND_SNI};
use crate::muq::{TAG_PROXY_FORWARD, TAG_QUEUE_REGISTER, TAG_QUEUE_SUBSCRIBE};
use crate::wire::{channel_hash, read_fixed, recv_frame, send_frame, write_fixed};

const OK: u8 = 0x01;

pub struct MuqClient {
    conn: Connection,
    _endpoint: Endpoint,
}

impl MuqClient {
    /// Подключиться к MUQ-узлу (host либо proxy). Без overlay-регистрации.
    pub async fn connect(server: SocketAddr) -> Result<Self, ClientError> {
        let mut endpoint = Endpoint::client("0.0.0.0:0".parse().expect("bind any"))?;
        endpoint.set_default_client_config(stand_client_config()?);
        let conn = endpoint.connect(server, STAND_SNI)?.await?;
        Ok(Self {
            conn,
            _endpoint: endpoint,
        })
    }

    /// Получатель регистрирует очередь на хосте. Возвращает true при ok.
    pub async fn register_queue(&self, q: &Queue) -> Result<bool, ClientError> {
        let (mut s, mut r) = self.conn.open_bi().await?;
        write_fixed(&mut s, &[TAG_QUEUE_REGISTER]).await?;
        write_fixed(&mut s, &q.to_bytes()).await?;
        let _ = s.finish();
        let mut ack = [0u8; 1];
        read_fixed(&mut r, &mut ack).await?;
        Ok(ack[0] == OK)
    }

    /// Отправитель кладёт депозит через entry-proxy (двуххоп; sealed непрозрачен proxy).
    pub async fn deposit_via_proxy(&self, pf: &ProxyForward) -> Result<bool, ClientError> {
        let (mut s, mut r) = self.conn.open_bi().await?;
        write_fixed(&mut s, &[TAG_PROXY_FORWARD]).await?;
        send_frame(&mut s, &pf.to_bytes()).await?;
        let mut ack = [0u8; 1];
        read_fixed(&mut r, &mut ack).await?;
        Ok(ack[0] == OK)
    }

    /// Получатель выбирает осколки: host шлёт nonce, клиент подписывает recv_key + channel_hash.
    pub async fn subscribe(
        &self,
        recv_id: QueueId,
        recv_sk: &SecretKey,
    ) -> Result<QueueResp, ClientError> {
        let (mut s, mut r) = self.conn.open_bi().await?;
        write_fixed(&mut s, &[TAG_QUEUE_SUBSCRIBE]).await?;

        let mut nonce = [0u8; 16];
        read_fixed(&mut r, &mut nonce).await?;

        let mut ch = [0u8; CHANNEL_HASH_SIZE];
        ch.copy_from_slice(&channel_hash(&self.conn)?);
        let sig = sign_subscribe(recv_sk, &recv_id, &nonce, &ch)?;
        let sub = QueueSubscribe {
            recv_id,
            nonce,
            sig: *sig.as_bytes(),
        };
        write_fixed(&mut s, &sub.to_bytes()).await?;
        let _ = s.finish();

        let bytes = recv_frame(&mut r).await?;
        QueueResp::decode(&bytes).map_err(ClientError::Decode)
    }
}
