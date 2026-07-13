//! MUQ-слой почтальона (Montana P2P Network, Этап 2): несвязываемая store-and-forward
//! доставка поверх QUIC. Роли узла — queue-host (держит очереди + буфер осколков) и
//! entry-proxy (двуххоп: принимает ProxyForward от отправителя, распечатывает транспорт,
//! пересылает sealed HostDeposit хосту). MUQ-клиент подключается БЕЗ overlay-регистрации
//! Этапа 1 (host видит эфемерный ключ очереди, НЕ account_id — несвязываемость).
//! Byte-exact ядро — mt_overlay::{muq, queue_host}; здесь только QUIC-транспорт.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use quinn::{Connection, Endpoint};

use mt_crypto::{Signature, SIGNATURE_SIZE};
use mt_overlay::challenge::{Nonce, CHANNEL_HASH_SIZE, NONCE_SIZE};
use mt_overlay::muq::{
    HostDeposit, ProxyForward, Queue, QueueId, QueueItem, QueueResp, QueueSubscribe, ReceiveProxy,
    QUEUE_ID_SIZE, QUEUE_WIRE_SIZE,
};
use mt_overlay::queue_host::QueueHost;
use mt_overlay::OverlayAddr;

use crate::config::stand_client_config;
use crate::wire::{channel_hash, read_fixed, recv_frame, send_frame, write_fixed, WireError};

/// Теги MUQ-операций (первый байт bi-потока). REG_VERSION=0x01 — путь Этапа 1.
pub const TAG_QUEUE_REGISTER: u8 = 0x10;
pub const TAG_HOST_DEPOSIT: u8 = 0x11;
pub const TAG_PROXY_FORWARD: u8 = 0x12;
pub const TAG_QUEUE_SUBSCRIBE: u8 = 0x13;
pub const TAG_RECEIVE_PROXY: u8 = 0x14; // B → courier (двуххоп-выборка)
pub const TAG_RELAY_SUBSCRIBE: u8 = 0x15; // courier → host

const OK: u8 = 0x01;
const ERR: u8 = 0x00;
const SUB_WIRE: usize = QUEUE_ID_SIZE + NONCE_SIZE + SIGNATURE_SIZE;

/// Состояние MUQ-узла: host-таблица очередей + proxy-маршруты (overlay host → физ.адрес стенда).
pub struct MuqState {
    host: Mutex<QueueHost>,
    proxy_routes: Mutex<HashMap<OverlayAddr, SocketAddr>>,
    /// Текущее окно для TTL депозита. Стенд: фиксировано; деплой подставит floor(unix/60).
    window: u64,
}

impl Default for MuqState {
    fn default() -> Self {
        Self {
            host: Mutex::new(QueueHost::new()),
            proxy_routes: Mutex::new(HashMap::new()),
            window: 0,
        }
    }
}

impl MuqState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Proxy-роль: сопоставить overlay-адрес queue-host его физическому адресу (конфиг стенда).
    pub fn add_proxy_route(&self, host_overlay: OverlayAddr, addr: SocketAddr) {
        self.proxy_routes.lock().unwrap().insert(host_overlay, addr);
    }

    /// Число осколков в буфере очереди (наблюдаемость/тесты).
    pub fn buffer_len(&self, recv_id: &QueueId) -> usize {
        self.host.lock().unwrap().buffer_of(recv_id).len()
    }
}

/// Диспетчер MUQ-соединения: первая операция (тег уже прочитан) + последующие bi-потоки.
pub async fn handle_muq_connection(
    conn: Connection,
    first_tag: u8,
    first_send: quinn::SendStream,
    first_recv: quinn::RecvStream,
    state: Arc<MuqState>,
) -> Result<(), WireError> {
    dispatch(&conn, first_tag, first_send, first_recv, &state).await?;
    loop {
        let (send, mut recv) = match conn.accept_bi().await {
            Ok(s) => s,
            Err(_) => return Ok(()), // соединение закрыто
        };
        let mut tag = [0u8; 1];
        if read_fixed(&mut recv, &mut tag).await.is_err() {
            return Ok(());
        }
        let _ = dispatch(&conn, tag[0], send, recv, &state).await;
    }
}

async fn dispatch(
    conn: &Connection,
    tag: u8,
    send: quinn::SendStream,
    recv: quinn::RecvStream,
    state: &Arc<MuqState>,
) -> Result<(), WireError> {
    match tag {
        TAG_QUEUE_REGISTER => handle_register(send, recv, state).await,
        TAG_HOST_DEPOSIT => handle_deposit(send, recv, state).await,
        TAG_PROXY_FORWARD => handle_proxy_forward(send, recv, state).await,
        TAG_QUEUE_SUBSCRIBE => handle_subscribe(conn, send, recv, state).await,
        TAG_RECEIVE_PROXY => handle_receive_proxy(send, recv, state).await,
        TAG_RELAY_SUBSCRIBE => handle_relay_subscribe(send, recv, state).await,
        _ => Ok(()),
    }
}

/// Получатель регистрирует очередь на хосте (recv_id/send_id независимы, recv_pubkey эфемерный).
async fn handle_register(
    mut send: quinn::SendStream,
    mut recv: quinn::RecvStream,
    state: &Arc<MuqState>,
) -> Result<(), WireError> {
    let mut buf = [0u8; QUEUE_WIRE_SIZE];
    read_fixed(&mut recv, &mut buf).await?;
    let ack = match Queue::decode(&buf) {
        Ok(q) => {
            state.host.lock().unwrap().register_queue(q);
            OK
        },
        Err(_) => ERR,
    };
    write_fixed(&mut send, &[ack]).await?;
    let _ = send.finish();
    Ok(())
}

/// Двуххоп-депозит: proxy принёс распечатанный HostDeposit. host verify send_key (secured) + буфер.
async fn handle_deposit(
    mut send: quinn::SendStream,
    mut recv: quinn::RecvStream,
    state: &Arc<MuqState>,
) -> Result<(), WireError> {
    let bytes = recv_frame(&mut recv).await?;
    let ack = match HostDeposit::decode(&bytes) {
        Ok(hd) => {
            let w = state.window;
            match state.host.lock().unwrap().deposit(&hd, w) {
                Ok(()) => OK,
                Err(_) => ERR,
            }
        },
        Err(_) => ERR,
    };
    write_fixed(&mut send, &[ack]).await?;
    let _ = send.finish();
    Ok(())
}

/// Entry-proxy: распечатывает транспорт, пересылает sealed HostDeposit хосту (proxy не видит recv_id).
async fn handle_proxy_forward(
    mut send: quinn::SendStream,
    mut recv: quinn::RecvStream,
    state: &Arc<MuqState>,
) -> Result<(), WireError> {
    let bytes = recv_frame(&mut recv).await?;
    let ack = match ProxyForward::decode(&bytes) {
        Ok(pf) => {
            let dst = state
                .proxy_routes
                .lock()
                .unwrap()
                .get(&pf.host_addr)
                .copied();
            match dst {
                Some(addr) => match forward_to_host(addr, &pf.sealed).await {
                    Ok(()) => OK,
                    Err(_) => ERR,
                },
                None => ERR, // неизвестный host — proxy не знает маршрут
            }
        },
        Err(_) => ERR,
    };
    write_fixed(&mut send, &[ack]).await?;
    let _ = send.finish();
    Ok(())
}

/// Proxy открывает клиентское соединение к host и шлёт sealed как HostDeposit.
async fn forward_to_host(host_addr: SocketAddr, sealed: &[u8]) -> Result<(), WireError> {
    let mut endpoint =
        Endpoint::client("0.0.0.0:0".parse().expect("bind any")).map_err(|_| WireError::Closed)?;
    endpoint.set_default_client_config(stand_client_config().map_err(|_| WireError::Closed)?);
    let conn = endpoint
        .connect(host_addr, crate::config::STAND_SNI)
        .map_err(|_| WireError::Closed)?
        .await
        .map_err(|_| WireError::Closed)?;
    let (mut s, mut r) = conn.open_bi().await.map_err(|_| WireError::Closed)?;
    write_fixed(&mut s, &[TAG_HOST_DEPOSIT]).await?;
    send_frame(&mut s, sealed).await?;
    // дождаться ack хоста (иначе соединение закроется до доставки депозита)
    let mut ack = [0u8; 1];
    let _ = read_fixed(&mut r, &mut ack).await;
    conn.close(0u32.into(), b"done");
    endpoint.wait_idle().await;
    Ok(())
}

/// Выборка получателем: host шлёт nonce (свежесть), verify подпись против ХРАНИМОГО
/// recv_pubkey + channel_hash (E-2/F3), отдаёт осколки, drop-on-delivery.
async fn handle_subscribe(
    conn: &Connection,
    mut send: quinn::SendStream,
    mut recv: quinn::RecvStream,
    state: &Arc<MuqState>,
) -> Result<(), WireError> {
    let nonce: Nonce = random_nonce();
    write_fixed(&mut send, &nonce).await?;

    let mut buf = [0u8; SUB_WIRE];
    read_fixed(&mut recv, &mut buf).await?;
    let sub = match QueueSubscribe::decode(&buf) {
        Ok(s) => s,
        Err(_) => {
            let _ = send_frame(&mut send, &QueueResp { items: vec![] }.to_bytes()).await;
            return Ok(());
        },
    };
    // nonce challenge должен совпасть с выданным (внутрисоединительная свежесть)
    if sub.nonce != nonce {
        let _ = send_frame(&mut send, &QueueResp { items: vec![] }.to_bytes()).await;
        return Ok(());
    }
    let mut ch = [0u8; CHANNEL_HASH_SIZE];
    ch.copy_from_slice(&channel_hash(conn)?);
    let sig = Signature::from_array(sub.sig);

    let items: Vec<QueueItem> = {
        let host = state.host.lock().unwrap();
        match host.subscribe(&sub.recv_id, &sub.nonce, &ch, &sig) {
            Ok(shards) => shards
                .iter()
                .map(|s| QueueItem {
                    msg_id: s.msg_id,
                    shard_index: s.shard_index,
                    shard_total: s.shard_total,
                    ct: s.ct.clone(),
                })
                .collect(),
            Err(_) => Vec::new(), // NoQueue/BadSig → пустой ответ (не раскрывать причину)
        }
    };
    let resp = QueueResp {
        items: items.clone(),
    };
    send_frame(&mut send, &resp.to_bytes()).await?;

    // drop-on-delivery (§483): после отдачи осколков очистить доставленные msg_id
    if !items.is_empty() {
        let mut host = state.host.lock().unwrap();
        for it in &items {
            host.drop_delivered(&sub.recv_id, &it.msg_id);
        }
    }
    Ok(())
}

/// Courier: принимает ReceiveProxy от получателя, несёт запечатанный QueueSubscribe хосту
/// (двуххоп-выборка), возвращает QueueResp обратно. Курьер НЕ видит recv_id (sealed непрозрачен).
async fn handle_receive_proxy(
    mut send: quinn::SendStream,
    mut recv: quinn::RecvStream,
    state: &Arc<MuqState>,
) -> Result<(), WireError> {
    let bytes = recv_frame(&mut recv).await?;
    let resp = match ReceiveProxy::decode(&bytes) {
        Ok(rp) => {
            let dst = state
                .proxy_routes
                .lock()
                .unwrap()
                .get(&rp.host_addr)
                .copied();
            match dst {
                Some(addr) => forward_subscribe_to_host(addr, &rp.sealed)
                    .await
                    .unwrap_or_else(|_| QueueResp { items: vec![] }.to_bytes()),
                None => QueueResp { items: vec![] }.to_bytes(),
            }
        },
        Err(_) => QueueResp { items: vec![] }.to_bytes(),
    };
    send_frame(&mut send, &resp).await?;
    Ok(())
}

/// Courier открывает соединение к host, шлёт sealed как relay-subscribe, читает QueueResp.
async fn forward_subscribe_to_host(
    host_addr: SocketAddr,
    sealed: &[u8],
) -> Result<Vec<u8>, WireError> {
    let mut endpoint =
        Endpoint::client("0.0.0.0:0".parse().expect("bind any")).map_err(|_| WireError::Closed)?;
    endpoint.set_default_client_config(stand_client_config().map_err(|_| WireError::Closed)?);
    let conn = endpoint
        .connect(host_addr, crate::config::STAND_SNI)
        .map_err(|_| WireError::Closed)?
        .await
        .map_err(|_| WireError::Closed)?;
    let (mut s, mut r) = conn.open_bi().await.map_err(|_| WireError::Closed)?;
    write_fixed(&mut s, &[TAG_RELAY_SUBSCRIBE]).await?;
    send_frame(&mut s, sealed).await?;
    let resp = recv_frame(&mut r).await?;
    conn.close(0u32.into(), b"done");
    endpoint.wait_idle().await;
    Ok(resp)
}

/// Host: relay-выборка от курьера. verify_subscribe(RELAY_CHANNEL_MARKER) + nonce-tracking
/// (anti-replay без channel_hash), отдаёт QueueResp курьеру, drop-on-delivery. Host видит
/// курьера, НЕ сетевую личность получателя B.
async fn handle_relay_subscribe(
    mut send: quinn::SendStream,
    mut recv: quinn::RecvStream,
    state: &Arc<MuqState>,
) -> Result<(), WireError> {
    let bytes = recv_frame(&mut recv).await?;
    let sub = match QueueSubscribe::decode(&bytes) {
        Ok(s) => s,
        Err(_) => {
            let _ = send_frame(&mut send, &QueueResp { items: vec![] }.to_bytes()).await;
            return Ok(());
        },
    };
    let sig = Signature::from_array(sub.sig);
    let items: Vec<QueueItem> = {
        let mut host = state.host.lock().unwrap();
        host.subscribe_relay(&sub.recv_id, &sub.nonce, &sig)
            .unwrap_or_default()
    };
    let resp = QueueResp {
        items: items.clone(),
    };
    send_frame(&mut send, &resp.to_bytes()).await?;
    if !items.is_empty() {
        let mut host = state.host.lock().unwrap();
        for it in &items {
            host.drop_delivered(&sub.recv_id, &it.msg_id);
        }
    }
    Ok(())
}

fn random_nonce() -> Nonce {
    let mut n = [0u8; NONCE_SIZE];
    getrandom::getrandom(&mut n).expect("OS CSPRNG");
    n
}
