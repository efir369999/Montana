//! MUQ-слой почтальона (Montana P2P Network, Этап 2): несвязываемая store-and-forward
//! доставка поверх TCP+TLS (спека §152 — TCP/TLS-443 обязателен). Роли узла — queue-host
//! (держит очереди + буфер осколков) и entry-proxy (двуххоп: принимает ProxyForward от
//! отправителя, распечатывает транспорт, пересылает sealed HostDeposit хосту). MUQ-клиент
//! подключается БЕЗ overlay-регистрации Этапа 1 (host видит эфемерный ключ очереди, НЕ
//! account_id — несвязываемость). Byte-exact ядро — mt_overlay::{muq, queue_host}; здесь
//! только TCP+TLS-транспорт. Каждая операция = одно короткое соединение (один дуплексный
//! стрим: сторона читает запрос, затем пишет ответ на том же `&mut S`).

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use rustls::pki_types::ServerName;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;

use mt_crypto::{
    keypair_from_seed_mlkem, open_from, MlkemPublicKey, MlkemSecretKey, Signature, MLKEM_SEED_SIZE,
};
use mt_overlay::muq::{
    HostDeposit, ProxyForward, Queue, QueueId, QueueItem, QueueResp, QueueSubscribe, ReceiveProxy,
    QUEUE_WIRE_SIZE,
};
use mt_overlay::queue_host::QueueHost;
use mt_overlay::OverlayAddr;

use crate::config::{tls_connector, STAND_SNI};
use crate::wire::{read_fixed, recv_frame, send_frame, write_fixed, WireError};

/// Теги MUQ-операций (первый байт соединения). REG_VERSION=0x01 — путь Этапа 1.
pub(crate) fn ack_ok() -> u8 {
    OK
}

pub const TAG_QUEUE_REGISTER: u8 = 0x10;
pub const TAG_HOST_DEPOSIT: u8 = 0x11;
pub const TAG_PROXY_FORWARD: u8 = 0x12;
pub const TAG_RECEIVE_PROXY: u8 = 0x14; // B → courier (двуххоп-выборка)
pub const TAG_RELAY_SUBSCRIBE: u8 = 0x15; // courier → host
pub const TAG_PROXY_REGISTER: u8 = 0x16; // B → courier (relay-регистрация)
pub const TAG_RELAY_REGISTER: u8 = 0x17; // courier → host
pub const TAG_RECEIVE_NONCE: u8 = 0x18; // B → courier: запрос host-issued nonce (§478)
pub const TAG_RELAY_NONCE: u8 = 0x19; // courier → host: forward sealed recv_id, вернуть nonce
pub const TAG_RECEIVE_ACK: u8 = 0x1A; // B → courier: подтверждение приёма (§593)
pub const TAG_RELAY_ACK: u8 = 0x1B; // courier → host: forward sealed recv_id, drop buffer
pub const TAG_NODE_HELLO: u8 = 0x1C; // sender → node: get capability (host_kem + send_id)

const OK: u8 = 0x01;
const ERR: u8 = 0x00;

/// Состояние MUQ-узла: host-таблица очередей + proxy-маршруты (overlay host → физ.адрес стенда).
// V-1: замки восстанавливаются из poison (unwrap_or_else into_inner) — паника одного
// обработчика не отравляет весь узел-почтальон, обслуживающий многих.
pub struct MuqState {
    host: Mutex<QueueHost>,
    proxy_routes: Mutex<HashMap<OverlayAddr, SocketAddr>>,
    /// ML-KEM keypair хоста: клиент запечатывает sealed к host_kem_pk, только host откроет.
    /// Курьер крипто-слеп к содержимому sealed (recv_id/депозит) — анонимность закрыта.
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
            host_kem_pk,
            host_kem_sk,
        }
    }
}

impl MuqState {
    /// DEV-049(d): эвикция истёкших шардов по TTL (вызывается prune-таймером узла).
    pub(crate) fn prune_expired(&self, window: u64) {
        self.host
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .prune(window);
    }

    pub fn new() -> Self {
        Self::default()
    }

    /// Backend-почтальон: детерминированная ML-KEM-личность из persist-seed — клиенты знают
    /// стабильный host_kem_pk между рестартами. Seed хранится оператором (postman-identity.bin).
    pub fn from_seed(seed: &[u8; MLKEM_SEED_SIZE]) -> Self {
        let (host_kem_pk, host_kem_sk) =
            keypair_from_seed_mlkem(seed).expect("ML-KEM keygen from seed");
        Self {
            host: Mutex::new(QueueHost::new()),
            proxy_routes: Mutex::new(HashMap::new()),
            host_kem_pk,
            host_kem_sk,
        }
    }

    /// Proxy-роль: сопоставить overlay-адрес queue-host его физическому адресу.
    pub fn add_proxy_route(&self, host_overlay: OverlayAddr, addr: SocketAddr) {
        self.proxy_routes
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(host_overlay, addr);
    }

    /// send_id зарегистрированной очереди (self-host: одна) — для hello-обмена.
    pub fn primary_send_id(&self) -> Option<QueueId> {
        self.host.lock().unwrap_or_else(|p| p.into_inner()).any_send_id()
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

/// Обработка одной MUQ-операции на свежем TCP+TLS-соединении (тег уже прочитан сервером).
/// Каждая операция клиента = отдельное соединение (модель SimpleX), поэтому цикла нет.
pub async fn handle_muq_op<S: AsyncRead + AsyncWrite + Unpin>(
    tag: u8,
    st: &mut S,
    state: &Arc<MuqState>,
) -> Result<(), WireError> {
    dispatch(tag, st, state).await
}

async fn dispatch<S: AsyncRead + AsyncWrite + Unpin>(
    tag: u8,
    st: &mut S,
    state: &Arc<MuqState>,
) -> Result<(), WireError> {
    match tag {
        TAG_QUEUE_REGISTER => handle_register(st, state).await,
        TAG_HOST_DEPOSIT => handle_deposit(st, state).await,
        TAG_PROXY_FORWARD => handle_proxy_forward(st, state).await,
        TAG_RECEIVE_PROXY => handle_receive_proxy(st, state).await,
        TAG_RELAY_SUBSCRIBE => handle_relay_subscribe(st, state).await,
        TAG_PROXY_REGISTER => handle_proxy_register(st, state).await,
        TAG_RELAY_REGISTER => handle_relay_register(st, state).await,
        TAG_RECEIVE_NONCE => handle_receive_nonce(st, state).await,
        TAG_RELAY_NONCE => handle_relay_nonce(st, state).await,
        TAG_RECEIVE_ACK => handle_receive_ack(st, state).await,
        TAG_RELAY_ACK => handle_relay_ack(st, state).await,
        TAG_NODE_HELLO => handle_node_hello(st, state).await,
        _ => Ok(()),
    }
}

/// Получатель регистрирует очередь на хосте (recv_id/send_id независимы, recv_pubkey эфемерный).
async fn handle_register<S: AsyncRead + AsyncWrite + Unpin>(
    st: &mut S,
    state: &Arc<MuqState>,
) -> Result<(), WireError> {
    let mut buf = [0u8; QUEUE_WIRE_SIZE];
    read_fixed(st, &mut buf).await?;
    let ack = match Queue::decode(&buf) {
        Ok(q) => {
            let accepted = state
                .host
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .register_queue(q, current_window());
            if accepted {
                OK
            } else {
                ERR // DEV-050(d): first-write-wins — hijack (перезапись recv_pubkey) отвергнут
            }
        },
        Err(_) => ERR,
    };
    write_fixed(st, &[ack]).await?;
    Ok(())
}

/// DEV-049(d): текущее окно из системных часов (floor(unix/60)) для TTL хранения шардов.
/// Off-chain ([P2P-1]) — MUQ-транспорт НЕ входит в consensus root, системные часы допустимы
/// для эвикции по TTL (не для consensus-значений; там только TimeChain).
pub(crate) fn current_window() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() / 60)
        .unwrap_or(0)
}

async fn handle_deposit<S: AsyncRead + AsyncWrite + Unpin>(
    st: &mut S,
    state: &Arc<MuqState>,
) -> Result<(), WireError> {
    let sealed = recv_frame(st).await?;
    let ack = match open_from(&state.host_kem_sk, &sealed)
        .ok()
        .and_then(|b| HostDeposit::decode(&b).ok())
    {
        Some(hd) => {
            let w = current_window();
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
    write_fixed(st, &[ack]).await?;
    Ok(())
}

/// Entry-proxy: распечатывает транспорт, пересылает sealed HostDeposit хосту (proxy не видит recv_id).
async fn handle_proxy_forward<S: AsyncRead + AsyncWrite + Unpin>(
    st: &mut S,
    state: &Arc<MuqState>,
) -> Result<(), WireError> {
    let bytes = recv_frame(st).await?;
    let ack = match ProxyForward::decode(&bytes) {
        Ok(pf) => match route_of(state, &pf.host_addr) {
            Some(addr) => match forward_deposit_to_host(addr, &pf.sealed).await {
                Ok(()) => OK,
                Err(_) => ERR,
            },
            None => ERR,
        },
        Err(_) => ERR,
    };
    write_fixed(st, &[ack]).await?;
    Ok(())
}

/// Courier: принимает ReceiveProxy от получателя, несёт запечатанный QueueSubscribe хосту
/// (двуххоп-выборка), возвращает QueueResp обратно. Курьер НЕ видит recv_id (sealed непрозрачен).
async fn handle_receive_proxy<S: AsyncRead + AsyncWrite + Unpin>(
    st: &mut S,
    state: &Arc<MuqState>,
) -> Result<(), WireError> {
    let bytes = recv_frame(st).await?;
    // DEV-052: НЕ маппим ошибки в пустой QueueResp — иначе B не отличит «нет почты» от
    // «доставка сломалась». Ошибка форварда → propagate → B видит error и делает refetch.
    let rp = ReceiveProxy::decode(&bytes).map_err(|_| WireError::Closed)?;
    let Some(addr) = route_of(state, &rp.host_addr) else {
        return Err(WireError::Closed); // нет маршрута — явная ошибка, не тихий empty
    };
    let resp = forward_subscribe_to_host(addr, &rp.sealed).await?; // forward-fail → propagate
    send_frame(st, &resp).await?;
    Ok(())
}

/// DEV-050(c) §478: courier форвардит запрос nonce хосту и возвращает 16-байтный
/// host-issued nonce получателю. Курьер крипто-слеп (sealed recv_id).
async fn handle_receive_nonce<S: AsyncRead + AsyncWrite + Unpin>(
    st: &mut S,
    state: &Arc<MuqState>,
) -> Result<(), WireError> {
    let bytes = recv_frame(st).await?;
    let pf = ProxyForward::decode(&bytes).map_err(|_| WireError::Closed)?;
    let Some(addr) = route_of(state, &pf.host_addr) else {
        return Err(WireError::Closed);
    };
    let nonce = forward_nonce_to_host(addr, &pf.sealed).await?;
    write_fixed(st, &nonce).await?;
    Ok(())
}

/// DEV-050(c): host открывает sealed recv_id и выдаёт свежий одноразовый nonce (issue_nonce).
async fn handle_relay_nonce<S: AsyncRead + AsyncWrite + Unpin>(
    st: &mut S,
    state: &Arc<MuqState>,
) -> Result<(), WireError> {
    let sealed = recv_frame(st).await?;
    let nonce = match open_from(&state.host_kem_sk, &sealed) {
        Ok(b) if b.len() == 32 => {
            let mut rid = [0u8; 32];
            rid.copy_from_slice(&b);
            state
                .host
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .issue_nonce(&rid)
        },
        _ => [0u8; 16], // ошибка → нулевой nonce (subscribe отвергнет: не выдан)
    };
    write_fixed(st, &nonce).await?;
    Ok(())
}

/// DEV-049(a) §593: courier форвардит подтверждение приёма хосту (drop-on-ack).
async fn handle_receive_ack<S: AsyncRead + AsyncWrite + Unpin>(
    st: &mut S,
    state: &Arc<MuqState>,
) -> Result<(), WireError> {
    let bytes = recv_frame(st).await?;
    let pf = ProxyForward::decode(&bytes).map_err(|_| WireError::Closed)?;
    let Some(addr) = route_of(state, &pf.host_addr) else {
        return Err(WireError::Closed);
    };
    let ok = forward_ack_to_host(addr, &pf.sealed).await?;
    write_fixed(st, &[ok]).await?;
    Ok(())
}

/// Host: подтверждение приёма — открывает sealed recv_id и дропает буфер очереди.
async fn handle_relay_ack<S: AsyncRead + AsyncWrite + Unpin>(
    st: &mut S,
    state: &Arc<MuqState>,
) -> Result<(), WireError> {
    let sealed = recv_frame(st).await?;
    let ack = match open_from(&state.host_kem_sk, &sealed)
        .ok()
        .and_then(|b| QueueSubscribe::decode(&b).ok())
    {
        Some(sub) => {
            let sig = Signature::from_array(sub.sig);
            let ok = state
                .host
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .ack_drain(&sub.recv_id, &sub.nonce, &sig);
            if ok {
                OK
            } else {
                ERR
            }
        },
        None => ERR,
    };
    write_fixed(st, &[ack]).await?;
    Ok(())
}

/// Host: relay-выборка от курьера. verify_subscribe + nonce-tracking (anti-replay), отдаёт
/// QueueResp курьеру. Host видит курьера, НЕ сетевую личность получателя B.
async fn handle_relay_subscribe<S: AsyncRead + AsyncWrite + Unpin>(
    st: &mut S,
    state: &Arc<MuqState>,
) -> Result<(), WireError> {
    let sealed = recv_frame(st).await?;
    let sub = match open_from(&state.host_kem_sk, &sealed)
        .ok()
        .and_then(|b| QueueSubscribe::decode(&b).ok())
    {
        Some(s) => s,
        None => {
            let _ = send_frame(st, &QueueResp { items: vec![] }.to_bytes()).await;
            return Ok(());
        },
    };
    let sig = Signature::from_array(sub.sig);
    let items: Vec<QueueItem> = {
        let mut host = state.host.lock().unwrap_or_else(|p| p.into_inner());
        host.subscribe_relay(&sub.recv_id, &sub.nonce, &sig)
            .unwrap_or_default()
    };
    let resp = QueueResp { items };
    send_frame(st, &resp.to_bytes()).await?;
    // DEV-049(a) §593: НЕ дропаем здесь — буфер держится до E2E-подтверждения B
    // (mt_client_ack → ack_drain). Транзит переживает падение плеча курьер→B.
    Ok(())
}

/// Courier: relay-регистрация — несёт запечатанный Queue хосту (курьер не видит recv_id).
async fn handle_proxy_register<S: AsyncRead + AsyncWrite + Unpin>(
    st: &mut S,
    state: &Arc<MuqState>,
) -> Result<(), WireError> {
    let bytes = recv_frame(st).await?;
    let ack = match ProxyForward::decode(&bytes) {
        Ok(pf) => match route_of(state, &pf.host_addr) {
            Some(addr) => match forward_register_to_host(addr, &pf.sealed).await {
                Ok(()) => OK,
                Err(_) => ERR,
            },
            None => ERR,
        },
        Err(_) => ERR,
    };
    write_fixed(st, &[ack]).await?;
    Ok(())
}

/// Host: relay-регистрация от курьера — распечатывает Queue (ML-KEM) и регистрирует.
async fn handle_relay_register<S: AsyncRead + AsyncWrite + Unpin>(
    st: &mut S,
    state: &Arc<MuqState>,
) -> Result<(), WireError> {
    let sealed = recv_frame(st).await?;
    let ack = match open_from(&state.host_kem_sk, &sealed)
        .ok()
        .and_then(|b| Queue::decode(&b).ok())
    {
        Some(q) => {
            let accepted = state
                .host
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .register_queue(q, current_window());
            if accepted {
                OK
            } else {
                ERR
            }
        },
        None => ERR,
    };
    write_fixed(st, &[ack]).await?;
    Ok(())
}

/// Node hello: отдать capability узла — host_kem (1184) + send_id (32) своей очереди.
/// Отправитель по mDNS находит узел, коннектится, получает hello → депонирует без карты.
async fn handle_node_hello<S: AsyncRead + AsyncWrite + Unpin>(
    st: &mut S,
    state: &Arc<MuqState>,
) -> Result<(), WireError> {
    let kem = state.host_kem_pubkey();
    let sid = state.primary_send_id().unwrap_or([0u8; 32]);
    write_fixed(st, kem.as_bytes()).await?;
    write_fixed(st, &sid).await?;
    Ok(())
}

// --- Курьер→host форвардинг: свежее TCP+TLS-соединение на каждую пересылку ---

fn route_of(state: &Arc<MuqState>, host: &OverlayAddr) -> Option<SocketAddr> {
    state
        .proxy_routes
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .get(host)
        .copied()
}

/// Открыть свежий TCP+TLS-стрим курьера к физическому адресу хоста.
async fn open_to_host(host_addr: SocketAddr) -> Result<TlsStream<TcpStream>, WireError> {
    let tcp = TcpStream::connect(host_addr)
        .await
        .map_err(|_| WireError::Closed)?;
    tcp.set_nodelay(true).ok();
    let connector = tls_connector().map_err(|_| WireError::Closed)?;
    let sni = ServerName::try_from(STAND_SNI)
        .map_err(|_| WireError::Closed)?
        .to_owned();
    connector
        .connect(sni, tcp)
        .await
        .map_err(|_| WireError::Closed)
}

/// Proxy открывает соединение к host и шлёт sealed как HostDeposit.
async fn forward_deposit_to_host(host_addr: SocketAddr, sealed: &[u8]) -> Result<(), WireError> {
    let mut st = open_to_host(host_addr).await?;
    write_fixed(&mut st, &[TAG_HOST_DEPOSIT]).await?;
    send_frame(&mut st, sealed).await?;
    let mut ack = [0u8; 1];
    let _ = read_fixed(&mut st, &mut ack).await; // дождаться ack (гарантия доставки депозита)
    Ok(())
}

async fn forward_subscribe_to_host(
    host_addr: SocketAddr,
    sealed: &[u8],
) -> Result<Vec<u8>, WireError> {
    let mut st = open_to_host(host_addr).await?;
    write_fixed(&mut st, &[TAG_RELAY_SUBSCRIBE]).await?;
    send_frame(&mut st, sealed).await?;
    recv_frame(&mut st).await
}

async fn forward_nonce_to_host(
    host_addr: SocketAddr,
    sealed: &[u8],
) -> Result<[u8; 16], WireError> {
    let mut st = open_to_host(host_addr).await?;
    write_fixed(&mut st, &[TAG_RELAY_NONCE]).await?;
    send_frame(&mut st, sealed).await?;
    let mut nonce = [0u8; 16];
    read_fixed(&mut st, &mut nonce).await?;
    Ok(nonce)
}

async fn forward_ack_to_host(host_addr: SocketAddr, sealed: &[u8]) -> Result<u8, WireError> {
    let mut st = open_to_host(host_addr).await?;
    write_fixed(&mut st, &[TAG_RELAY_ACK]).await?;
    send_frame(&mut st, sealed).await?;
    let mut ack = [0u8; 1];
    read_fixed(&mut st, &mut ack).await?;
    Ok(ack[0])
}

async fn forward_register_to_host(host_addr: SocketAddr, sealed: &[u8]) -> Result<(), WireError> {
    let mut st = open_to_host(host_addr).await?;
    write_fixed(&mut st, &[TAG_RELAY_REGISTER]).await?;
    send_frame(&mut st, sealed).await?;
    let mut ack = [0u8; 1];
    let _ = read_fixed(&mut st, &mut ack).await;
    Ok(())
}
