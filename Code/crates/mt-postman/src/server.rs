//! Сервер-почтальон (Этап 1, механика шаги 0/3). Accept TCP+TLS-соединений (спека §152 —
//! TCP/TLS-443 обязателен) → первый байт различает протокол: 0x01 (RegHello version) = Этап 1
//! (overlay-регистрация ML-DSA + routing-цикл), 0x10..0x1B = MUQ-операция Этапа 2 (одно
//! соединение = одна операция). Транспорт-агностичную маршрутизацию несёт
//! mt_overlay::postman::Postman; здесь — TCP+TLS-слой поверх неё.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use thiserror::Error;
use tokio::io::{split, ReadHalf, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex as AsyncMutex;
use tokio_rustls::server::TlsStream;
use tokio_rustls::TlsAcceptor;

use mt_crypto::{Signature, SIGNATURE_SIZE};
use mt_overlay::challenge::{Nonce, NONCE_SIZE};
use mt_overlay::frame::OverlayFrame;
use mt_overlay::postman::{ConnId, Postman, Route};
use mt_overlay::prologue::{
    decode_reg_hello as decode_reg_hello_bytes, decode_reg_proof, encode_reg_challenge,
    REG_HELLO_SIZE, REG_VERSION,
};

use crate::config::{tls_acceptor, ConfigError};
use crate::muq::{handle_muq_op, MuqState};
use crate::wire::{channel_hash, read_fixed, recv_frame, send_frame, write_fixed, WireError};

const REG_OK: u8 = 0x01;

type ServerStream = TlsStream<TcpStream>;
type ConnWriter = Arc<AsyncMutex<WriteHalf<ServerStream>>>;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("config: {0}")]
    Config(#[from] ConfigError),
    #[error("endpoint io: {0}")]
    Io(#[from] std::io::Error),
    #[error("wire: {0}")]
    Wire(#[from] WireError),
    #[error("registration rejected (bad ML-DSA proof or binding)")]
    RegistrationRejected,
    #[error("malformed prologue")]
    MalformedPrologue,
}

#[derive(Default)]
pub(crate) struct Registry {
    postman: Postman,
    writers: HashMap<ConnId, ConnWriter>,
    next: ConnId,
}

#[derive(Clone)]
pub struct PostmanServer {
    listener: Arc<TcpListener>,
    acceptor: TlsAcceptor,
    reg: Arc<Mutex<Registry>>,
    muq: Arc<MuqState>,
}

impl PostmanServer {
    /// Поднять почтальон на addr (стенд: 127.0.0.1:0 — ОС выберет свободный порт).
    pub async fn bind(addr: SocketAddr) -> Result<Self, ServerError> {
        let listener = TcpListener::bind(addr).await?;
        Ok(Self {
            listener: Arc::new(listener),
            acceptor: tls_acceptor()?,
            reg: Arc::new(Mutex::new(Registry::default())),
            muq: Arc::new(MuqState::new()),
        })
    }

    /// Backend-почтальон с persistent ML-KEM-личностью (стабильный host_kem_pk).
    pub async fn bind_with_seed(
        addr: SocketAddr,
        seed: &[u8; mt_crypto::MLKEM_SEED_SIZE],
    ) -> Result<Self, ServerError> {
        let listener = TcpListener::bind(addr).await?;
        Ok(Self {
            listener: Arc::new(listener),
            acceptor: tls_acceptor()?,
            reg: Arc::new(Mutex::new(Registry::default())),
            muq: Arc::new(MuqState::from_seed(seed)),
        })
    }

    pub fn local_addr(&self) -> Result<SocketAddr, ServerError> {
        Ok(self.listener.local_addr()?)
    }

    /// MUQ-состояние узла (queue-host + proxy-маршруты, Этап 2).
    pub fn muq(&self) -> &Arc<MuqState> {
        &self.muq
    }

    /// Число зарегистрированных живых overlay-соединений (для тестов/наблюдаемости).
    pub fn registered_count(&self) -> usize {
        self.reg
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .writers
            .len()
    }

    /// Accept-цикл: на каждое соединение — независимый таск (TLS → tag → handshake/routing либо MUQ).
    pub async fn run(self) {
        // DEV-049(d): периодическая эвикция истёкших шардов по TTL (окно из системных часов).
        let prune_muq = self.muq.clone();
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                tick.tick().await;
                prune_muq.prune_expired(crate::muq::current_window());
            }
        });
        loop {
            let (tcp, _peer) = match self.listener.accept().await {
                Ok(x) => x,
                Err(_) => continue,
            };
            tcp.set_nodelay(true).ok();
            let acceptor = self.acceptor.clone();
            let reg = self.reg.clone();
            let muq = self.muq.clone();
            tokio::spawn(async move {
                let Ok(tls) = acceptor.accept(tcp).await else {
                    return;
                };
                let _ = handle_connection(tls, reg, muq).await;
            });
        }
    }
}

pub(crate) async fn handle_connection(
    mut tls: ServerStream,
    reg: Arc<Mutex<Registry>>,
    muq: Arc<MuqState>,
) -> Result<(), ServerError> {
    // Первый байт различает протокол: 0x01 (RegHello version) = Этап 1 overlay-регистрация,
    // 0x10..0x1B = MUQ-операция Этапа 2.
    let mut tag = [0u8; 1];
    read_fixed(&mut tls, &mut tag).await?;

    if tag[0] != REG_VERSION {
        // Этап 2: одно соединение = одна MUQ-операция.
        handle_muq_op(tag[0], &mut tls, &muq).await?;
        return Ok(());
    }

    // Этап 1: дочитать pubkey RegHello (version уже прочитан как tag).
    let mut hello = [0u8; REG_HELLO_SIZE];
    hello[0] = tag[0];
    read_fixed(&mut tls, &mut hello[1..]).await?;
    let account_pubkey =
        decode_reg_hello_bytes(&hello).map_err(|_| ServerError::MalformedPrologue)?;

    let nonce: Nonce = random_nonce();
    write_fixed(&mut tls, &encode_reg_challenge(&nonce)).await?;

    let mut proof = [0u8; SIGNATURE_SIZE];
    read_fixed(&mut tls, &mut proof).await?;
    let sig_bytes = decode_reg_proof(&proof).map_err(|_| ServerError::MalformedPrologue)?;
    let sig = Signature::from_array(sig_bytes);

    // channel_hash из TLS-соединения ДО split (get_ref теряется после split).
    let ch = channel_hash(tls.get_ref().1)?;

    // register сам делает verify_registration (подпись + привязка account_pubkey↔addr).
    let (reader, writer) = split(tls);
    let writer: ConnWriter = Arc::new(AsyncMutex::new(writer));
    let conn_id = {
        let mut g = reg.lock().unwrap_or_else(|p| p.into_inner());
        let id = g.next;
        match g.postman.register(id, &account_pubkey, &nonce, &ch, &sig) {
            Some(_addr) => {
                g.writers.insert(id, writer.clone());
                g.next += 1;
                id
            },
            None => return Err(ServerError::RegistrationRejected),
        }
    };

    {
        let mut w = writer.lock().await;
        write_fixed(&mut *w, &[REG_OK]).await?;
    }

    let result = routing_loop(reader, conn_id, &reg).await;

    {
        let mut g = reg.lock().unwrap_or_else(|p| p.into_inner());
        g.postman.deregister(conn_id);
        g.writers.remove(&conn_id);
    }
    result
}

async fn routing_loop(
    mut reader: ReadHalf<ServerStream>,
    conn_id: ConnId,
    reg: &Arc<Mutex<Registry>>,
) -> Result<(), ServerError> {
    loop {
        // Шаг 3: каждое входящее сообщение = один OverlayFrame (длина-префикс).
        let bytes = match recv_frame(&mut reader).await {
            Ok(b) => b,
            Err(_) => return Ok(()), // соединение закрыто/сброшено
        };
        let frame = match OverlayFrame::decode(&bytes) {
            Ok(f) => f,
            Err(_) => continue, // мусорный фрейм — соединение живо, игнор
        };

        let route = {
            let g = reg.lock().unwrap_or_else(|p| p.into_inner());
            g.postman.route(conn_id, frame)
        };
        match route {
            Route::Deliver { conn: dst, frame } | Route::AckToSender { conn: dst, frame } => {
                let target = reg
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .writers
                    .get(&dst)
                    .cloned();
                if let Some(w) = target {
                    let mut g = w.lock().await;
                    let _ = send_frame(&mut *g, &frame.to_bytes()).await;
                }
            },
            // Этап 2 (MUQ store-and-forward): получатель офлайн. На Этапе 1 — no-op.
            Route::Buffer { .. } => {},
            Route::Drop => {},
        }
    }
}

fn random_nonce() -> Nonce {
    let mut n = [0u8; NONCE_SIZE];
    getrandom::getrandom(&mut n).expect("OS CSPRNG");
    n
}
