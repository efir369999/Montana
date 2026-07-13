//! MUQ-клиент (Этап 2). Подключается к queue-host / entry-proxy БЕЗ overlay-регистрации
//! Этапа 1 (несвязываемость — узел видит эфемерный ключ очереди, не account_id).
//! Получатель: register_queue + subscribe; отправитель: deposit_via_proxy (двуххоп).

use std::net::SocketAddr;

use quinn::{Connection, Endpoint};

use mt_crypto::{seal_to, MlkemPublicKey, SecretKey};
use mt_overlay::muq::{
    sign_subscribe_relay, ProxyForward, Queue, QueueId, QueueResp, QueueSubscribe, ReceiveProxy,
};
use mt_overlay::OverlayAddr;

use crate::client::ClientError;
use crate::config::{stand_client_config, STAND_SNI};
use crate::muq::{TAG_PROXY_FORWARD, TAG_PROXY_REGISTER, TAG_QUEUE_REGISTER, TAG_RECEIVE_PROXY};
use crate::wire::{read_fixed, recv_frame, send_frame, write_fixed};

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
        muq_register(&self.conn, q).await
    }

    /// Отправитель кладёт депозит через entry-proxy (двуххоп; sealed непрозрачен proxy).
    pub async fn deposit_via_proxy(&self, pf: &ProxyForward) -> Result<bool, ClientError> {
        muq_deposit(&self.conn, pf).await
    }

    /// Relay-регистрация очереди на чужом хосте через курьер (host видит курьера, не нас).
    pub async fn register_via_courier(
        &self,
        host_overlay: OverlayAddr,
        host_kem_pk: &MlkemPublicKey,
        q: &Queue,
    ) -> Result<bool, ClientError> {
        muq_register_via_courier(&self.conn, host_overlay, host_kem_pk, q).await
    }

    /// Relay-выборка через курьер (host видит курьера, не нас).
    pub async fn subscribe_via_courier(
        &self,
        host_overlay: OverlayAddr,
        host_kem_pk: &MlkemPublicKey,
        recv_id: QueueId,
        recv_sk: &SecretKey,
    ) -> Result<QueueResp, ClientError> {
        muq_subscribe_via_courier(&self.conn, host_overlay, host_kem_pk, recv_id, recv_sk).await
    }
}

// --- MUQ-операции над произвольным соединением (переиспользуют Node и MuqClient) ---

pub(crate) async fn muq_register(conn: &Connection, q: &Queue) -> Result<bool, ClientError> {
    let (mut s, mut r) = conn.open_bi().await?;
    write_fixed(&mut s, &[TAG_QUEUE_REGISTER]).await?;
    write_fixed(&mut s, &q.to_bytes()).await?;
    let _ = s.finish();
    let mut ack = [0u8; 1];
    read_fixed(&mut r, &mut ack).await?;
    Ok(ack[0] == OK)
}

pub(crate) async fn muq_deposit(conn: &Connection, pf: &ProxyForward) -> Result<bool, ClientError> {
    let (mut s, mut r) = conn.open_bi().await?;
    write_fixed(&mut s, &[TAG_PROXY_FORWARD]).await?;
    send_frame(&mut s, &pf.to_bytes()).await?;
    let mut ack = [0u8; 1];
    read_fixed(&mut r, &mut ack).await?;
    Ok(ack[0] == OK)
}

/// Relay-регистрация: очередь регистрируется через курьер, host видит курьера, не B.
/// Queue запечатан ML-KEM к хосту — курьер крипто-слеп к recv_id/recv_pubkey.
pub(crate) async fn muq_register_via_courier(
    conn: &Connection,
    host_overlay: OverlayAddr,
    host_kem_pk: &MlkemPublicKey,
    q: &Queue,
) -> Result<bool, ClientError> {
    let sealed = seal_to(host_kem_pk, &q.to_bytes()).map_err(|_| ClientError::Rejected)?;
    let pf = ProxyForward {
        host_addr: host_overlay,
        sealed,
    };
    let (mut s, mut r) = conn.open_bi().await?;
    write_fixed(&mut s, &[TAG_PROXY_REGISTER]).await?;
    send_frame(&mut s, &pf.to_bytes()).await?;
    let mut ack = [0u8; 1];
    read_fixed(&mut r, &mut ack).await?;
    Ok(ack[0] == OK)
}

/// Двуххоп-выборка: получатель забирает через курьер (host видит курьера, не B).
/// B генерит nonce (host трекает — anti-replay без channel_hash), подписывает recv_key,
/// запечатывает QueueSubscribe в ReceiveProxy для host (курьер не видит recv_id).
pub(crate) async fn muq_subscribe_via_courier(
    conn: &Connection,
    host_overlay: OverlayAddr,
    host_kem_pk: &MlkemPublicKey,
    recv_id: QueueId,
    recv_sk: &SecretKey,
) -> Result<QueueResp, ClientError> {
    let mut nonce = [0u8; 16];
    getrandom::getrandom(&mut nonce).map_err(|_| ClientError::Rejected)?;
    let sig = sign_subscribe_relay(recv_sk, &recv_id, &nonce)?;
    let sub = QueueSubscribe {
        recv_id,
        nonce,
        sig: *sig.as_bytes(),
    };
    // Запечатать QueueSubscribe к ML-KEM ключу хоста — курьер крипто-слеп к recv_id.
    let sealed = seal_to(host_kem_pk, &sub.to_bytes()).map_err(|_| ClientError::Rejected)?;
    let rp = ReceiveProxy {
        host_addr: host_overlay,
        sealed,
    };
    let (mut s, mut r) = conn.open_bi().await?;
    write_fixed(&mut s, &[TAG_RECEIVE_PROXY]).await?;
    send_frame(&mut s, &rp.to_bytes()).await?;
    let bytes = recv_frame(&mut r).await?;
    QueueResp::decode(&bytes).map_err(ClientError::Decode)
}
