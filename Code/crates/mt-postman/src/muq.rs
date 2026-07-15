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

use mt_crypto::{
    keypair_from_seed_mlkem, open_from, MlkemPublicKey, MlkemSecretKey, Signature, MLKEM_SEED_SIZE,
};
use mt_overlay::muq::{
    HostDeposit, ProxyForward, Queue, QueueId, QueueItem, QueueResp, QueueSubscribe, ReceiveProxy,
    QUEUE_WIRE_SIZE,
};
use mt_overlay::queue_host::QueueHost;
use mt_overlay::OverlayAddr;

use crate::config::stand_client_config;
use crate::wire::{read_fixed, recv_frame, send_frame, write_fixed, WireError};

/// Теги MUQ-операций (первый байт bi-потока). REG_VERSION=0x01 — путь Этапа 1.
pub const TAG_QUEUE_REGISTER: u8 = 0x10;
pub const TAG_HOST_DEPOSIT: u8 = 0x11;
pub const TAG_PROXY_FORWARD: u8 = 0x12;
pub const TAG_RECEIVE_PROXY: u8 = 0x14; // B → courier (двуххоп-выборка)
pub const TAG_RELAY_SUBSCRIBE: u8 = 0x15; // courier → host
pub const TAG_PROXY_REGISTER: u8 = 0x16; // B → courier (relay-регистрация)
pub const TAG_RELAY_REGISTER: u8 = 0x17; // courier → host

const OK: u8 = 0x01;
const ERR: u8 = 0x00;

/// Состояние MUQ-узла: host-таблица очередей + proxy-маршруты (overlay host → физ.адрес стенда).
// V-1: замки восстанавливаются из poison (unwrap_or_else into_inner) — паника одного
// обработчика не отравляет весь узел-почтальон, обслуживающий многих. Мутации под
// замками простые (insert/get/buffer), частичного неконсистентного состояния не создают.
pub struct MuqState {
    host: Mutex<QueueHost>,
    proxy_routes: Mutex<HashMap<OverlayAddr, SocketAddr>>,
    /// Текущее окно для TTL депозита. Стенд: фиксировано; деплой подставит floor(unix/60).
    window: u64,
    /// ML-KEM keypair хоста: клиент запечатывает sealed к host_kem_pk, только host откроет.
    /// Курьер крипто-слеп к содержимому sealed (recv_id/депозит) — вопрос анонимности закрыт.
    host_kem_pk: MlkemPublicKey,
    host_kem_sk: MlkemSecretKey,
}

impl Default for MuqState {
    fn default() -> Self {
        let mut seed = [0u8; MLKEM_SEED_SIZE];
        getrandom::getrandom(&mut seed).expect("OS CSPRNG");
        let (host_kem_pk, host_kem_sk) = keypair_from_seed_mlkem(&seed).expect("ML-KEM keygen");
        Self {
            host: Mutex::new(QueueHost::new()),
            proxy_routes: Mutex::new(HashMap::new()),
            window: 0,
            host_kem_pk,
            host_kem_sk,
        }
    }
}

impl MuqState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Proxy-роль: сопоставить overlay-адрес queue-host его физическому адресу (конфиг стенда).
    pub fn add_proxy_route(&self, host_overlay: OverlayAddr, addr: SocketAddr) {
        self.proxy_routes
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(host_overlay, addr);
    }

    /// Публичный ML-KEM ключ хоста — клиент запечатывает sealed к нему (курьер крипто-слеп).
    pub fn host_kem_pubkey(&self) -> MlkemPublicKey {
        MlkemPublicKey::from_array(*self.host_kem_pk.as_bytes())
    }

    /// Локальный дренаж своей очереди (self-host, БЕЗ курьера) — абсолют против сговора.
    pub fn local_drain(&self, recv_id: &QueueId) -> Vec<QueueItem> {
        let mut host = self.host.lock().unwrap_or_else(|p| p.into_inner());
        let items = host.buffer_of(recv_id);
        for it in &items {
            host.drop_delivered(recv_id, &it.msg_id);
        }
        items
    }

    /// Число осколков в буфере очереди (наблюдаемость/тесты).
    pub fn buffer_len(&self, recv_id: &QueueId) -> usize {
        self.host
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .buffer_of(recv_id)
            .len()
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
    _conn: &Connection,
    tag: u8,
    send: quinn::SendStream,
    recv: quinn::RecvStream,
    state: &Arc<MuqState>,
) -> Result<(), WireError> {
    match tag {
        TAG_QUEUE_REGISTER => handle_register(send, recv, state).await,
        TAG_HOST_DEPOSIT => handle_deposit(send, recv, state).await,
        TAG_PROXY_FORWARD => handle_proxy_forward(send, recv, state).await,
        TAG_RECEIVE_PROXY => handle_receive_proxy(send, recv, state).await,
        TAG_RELAY_SUBSCRIBE => handle_relay_subscribe(send, recv, state).await,
        TAG_PROXY_REGISTER => handle_proxy_register(send, recv, state).await,
        TAG_RELAY_REGISTER => handle_relay_register(send, recv, state).await,
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
            state
                .host
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .register_queue(q);
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
    let sealed = recv_frame(&mut recv).await?;
    let ack = match open_from(&state.host_kem_sk, &sealed)
        .ok()
        .and_then(|b| HostDeposit::decode(&b).ok())
    {
        Some(hd) => {
            let w = state.window;
            match state
                .host
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .deposit(&hd, w)
            {
                Ok(()) => OK,
                Err(_) => ERR,
            }
        },
        None => ERR,
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
                .unwrap_or_else(|p| p.into_inner())
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

/// Courier: принимает ReceiveProxy от получателя, несёт запечатанный QueueSubscribe хосту
/// (двуххоп-выборка), возвращает QueueResp обратно. Курьер НЕ видит recv_id (sealed непрозрачен).
async fn handle_receive_proxy(
    mut send: quinn::SendStream,
    mut recv: quinn::RecvStream,
    state: &Arc<MuqState>,
) -> Result<(), WireError> {
    let bytes = recv_frame(&mut recv).await?;
    // DEV-052: НЕ маппим ошибки в пустой QueueResp — иначе B не отличит «нет почты»
    // (легитимный empty от host) от «доставка сломалась» (oversize frame / host down /
    // нет маршрута). Ошибка форварда → propagate → stream reset → B видит error и делает
    // refetch, вместо тихого «очередь пуста». Пустой QueueResp остаётся только ответом host.
    let rp = ReceiveProxy::decode(&bytes).map_err(|_| WireError::Closed)?;
    let dst = state
        .proxy_routes
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .get(&rp.host_addr)
        .copied();
    let Some(addr) = dst else {
        return Err(WireError::Closed); // нет маршрута — явная ошибка, не тихий empty
    };
    let resp = forward_subscribe_to_host(addr, &rp.sealed).await?; // forward-fail → propagate
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
    let sealed = recv_frame(&mut recv).await?;
    let sub = match open_from(&state.host_kem_sk, &sealed)
        .ok()
        .and_then(|b| QueueSubscribe::decode(&b).ok())
    {
        Some(s) => s,
        None => {
            let _ = send_frame(&mut send, &QueueResp { items: vec![] }.to_bytes()).await;
            return Ok(());
        },
    };
    let sig = Signature::from_array(sub.sig);
    let items: Vec<QueueItem> = {
        let mut host = state.host.lock().unwrap_or_else(|p| p.into_inner());
        host.subscribe_relay(&sub.recv_id, &sub.nonce, &sig)
            .unwrap_or_default()
    };
    let resp = QueueResp {
        items: items.clone(),
    };
    send_frame(&mut send, &resp.to_bytes()).await?;
    if !items.is_empty() {
        let mut host = state.host.lock().unwrap_or_else(|p| p.into_inner());
        for it in &items {
            host.drop_delivered(&sub.recv_id, &it.msg_id);
        }
    }
    Ok(())
}

/// Courier: relay-регистрация — несёт запечатанный Queue хосту (курьер не видит recv_id).
async fn handle_proxy_register(
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
                .unwrap_or_else(|p| p.into_inner())
                .get(&pf.host_addr)
                .copied();
            match dst {
                Some(addr) => match forward_register_to_host(addr, &pf.sealed).await {
                    Ok(()) => OK,
                    Err(_) => ERR,
                },
                None => ERR,
            }
        },
        Err(_) => ERR,
    };
    write_fixed(&mut send, &[ack]).await?;
    let _ = send.finish();
    Ok(())
}

async fn forward_register_to_host(host_addr: SocketAddr, sealed: &[u8]) -> Result<(), WireError> {
    let mut endpoint =
        Endpoint::client("0.0.0.0:0".parse().expect("bind any")).map_err(|_| WireError::Closed)?;
    endpoint.set_default_client_config(stand_client_config().map_err(|_| WireError::Closed)?);
    let conn = endpoint
        .connect(host_addr, crate::config::STAND_SNI)
        .map_err(|_| WireError::Closed)?
        .await
        .map_err(|_| WireError::Closed)?;
    let (mut s, mut r) = conn.open_bi().await.map_err(|_| WireError::Closed)?;
    write_fixed(&mut s, &[TAG_RELAY_REGISTER]).await?;
    send_frame(&mut s, sealed).await?;
    let mut ack = [0u8; 1];
    let _ = read_fixed(&mut r, &mut ack).await;
    conn.close(0u32.into(), b"done");
    endpoint.wait_idle().await;
    Ok(())
}

/// Host: relay-регистрация от курьера — распечатывает Queue (ML-KEM) и регистрирует.
/// Host видит курьера, НЕ получателя B.
async fn handle_relay_register(
    mut send: quinn::SendStream,
    mut recv: quinn::RecvStream,
    state: &Arc<MuqState>,
) -> Result<(), WireError> {
    let sealed = recv_frame(&mut recv).await?;
    let ack = match open_from(&state.host_kem_sk, &sealed)
        .ok()
        .and_then(|b| Queue::decode(&b).ok())
    {
        Some(q) => {
            state
                .host
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .register_queue(q);
            OK
        },
        None => ERR,
    };
    write_fixed(&mut send, &[ack]).await?;
    let _ = send.finish();
    Ok(())
}
