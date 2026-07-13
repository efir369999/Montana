//! Сервер-почтальон (Этап 1, механика шаги 0/3). Accept QUIC-соединений →
//! прологовое рукопожатие (RegHello/RegChallenge/RegProof, ML-DSA-65) →
//! routing-цикл (RELAY→DELIVER/Buffer, ACK). Транспорт-агностичную маршрутизацию
//! несёт mt_overlay::postman::Postman; здесь — QUIC-слой поверх неё.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use quinn::{Connection, Endpoint};
use thiserror::Error;

use mt_crypto::{Signature, SIGNATURE_SIZE};
use mt_overlay::challenge::{Nonce, NONCE_SIZE};
use mt_overlay::frame::OverlayFrame;
use mt_overlay::postman::{ConnId, Postman, Route};
use mt_overlay::prologue::{
    decode_reg_hello as decode_reg_hello_bytes, decode_reg_proof, encode_reg_challenge,
    REG_HELLO_SIZE, REG_VERSION,
};

use crate::config::{stand_server_config, ConfigError};
use crate::muq::{handle_muq_connection, MuqState};
use crate::wire::{channel_hash, read_fixed, recv_frame, send_frame, write_fixed, WireError};

const REG_OK: u8 = 0x01;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("config: {0}")]
    Config(#[from] ConfigError),
    #[error("endpoint io: {0}")]
    Io(#[from] std::io::Error),
    #[error("connection: {0}")]
    Connection(#[from] quinn::ConnectionError),
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
    conns: HashMap<ConnId, Connection>,
    next: ConnId,
}

#[derive(Clone)]
pub struct PostmanServer {
    endpoint: Endpoint,
    reg: Arc<Mutex<Registry>>,
    muq: Arc<MuqState>,
}

impl PostmanServer {
    /// Поднять почтальон на addr (стенд: 127.0.0.1:0 — ОС выберет свободный порт).
    pub fn bind(addr: SocketAddr) -> Result<Self, ServerError> {
        let endpoint = Endpoint::server(stand_server_config()?, addr)?;
        Ok(Self {
            endpoint,
            reg: Arc::new(Mutex::new(Registry::default())),
            muq: Arc::new(MuqState::new()),
        })
    }

    pub fn local_addr(&self) -> Result<SocketAddr, ServerError> {
        Ok(self.endpoint.local_addr()?)
    }

    pub fn endpoint(&self) -> &Endpoint {
        &self.endpoint
    }

    /// MUQ-состояние узла (queue-host + proxy-маршруты, Этап 2).
    pub fn muq(&self) -> &Arc<MuqState> {
        &self.muq
    }

    /// Число зарегистрированных живых соединений (для тестов/наблюдаемости).
    pub fn registered_count(&self) -> usize {
        self.reg.lock().unwrap().conns.len()
    }

    /// Accept-цикл: на каждое соединение — независимый таск (handshake → routing).
    pub async fn run(self) {
        while let Some(incoming) = self.endpoint.accept().await {
            let reg = self.reg.clone();
            let muq = self.muq.clone();
            tokio::spawn(async move {
                if let Ok(conn) = incoming.await {
                    let _ = handle_connection(conn, reg, muq).await;
                }
            });
        }
    }
}

pub(crate) async fn handle_connection(
    conn: Connection,
    reg: Arc<Mutex<Registry>>,
    muq: Arc<MuqState>,
) -> Result<(), ServerError> {
    // Первый bi-поток: первый байт различает протокол — 0x01 (RegHello version) = Этап 1
    // (overlay-регистрация), 0x10..0x13 = MUQ-операция Этапа 2 (без overlay-регистрации).
    let (mut send, mut recv) = conn.accept_bi().await?;
    let mut tag = [0u8; 1];
    read_fixed(&mut recv, &mut tag).await?;

    if tag[0] != REG_VERSION {
        // Этап 2: MUQ-соединение (host/proxy). Не consensus, не overlay-идентичность.
        handle_muq_connection(conn, tag[0], send, recv, muq).await?;
        return Ok(());
    }

    // Этап 1: дочитать pubkey RegHello (version уже прочитан как tag).
    let mut hello = [0u8; REG_HELLO_SIZE];
    hello[0] = tag[0];
    read_fixed(&mut recv, &mut hello[1..]).await?;
    let account_pubkey =
        decode_reg_hello_bytes(&hello).map_err(|_| ServerError::MalformedPrologue)?;

    let nonce: Nonce = random_nonce();
    write_fixed(&mut send, &encode_reg_challenge(&nonce)).await?;

    let mut proof = [0u8; SIGNATURE_SIZE];
    read_fixed(&mut recv, &mut proof).await?;
    let sig_bytes = decode_reg_proof(&proof).map_err(|_| ServerError::MalformedPrologue)?;
    let sig = Signature::from_array(sig_bytes);

    let ch = channel_hash(&conn)?;

    // register сам делает verify_registration (подпись + привязка account_pubkey↔addr).
    let conn_id = {
        let mut g = reg.lock().unwrap();
        let id = g.next;
        match g.postman.register(id, &account_pubkey, &nonce, &ch, &sig) {
            Some(_addr) => {
                g.conns.insert(id, conn.clone());
                g.next += 1;
                id
            },
            None => return Err(ServerError::RegistrationRejected),
        }
    };

    write_fixed(&mut send, &[REG_OK]).await?;
    let _ = send.finish();

    let result = routing_loop(&conn, conn_id, &reg).await;

    {
        let mut g = reg.lock().unwrap();
        g.postman.deregister(conn_id);
        g.conns.remove(&conn_id);
    }
    result
}

async fn routing_loop(
    conn: &Connection,
    conn_id: ConnId,
    reg: &Arc<Mutex<Registry>>,
) -> Result<(), ServerError> {
    loop {
        // Шаг 3: каждый входящий uni-поток = один OverlayFrame.
        let mut recv = match conn.accept_uni().await {
            Ok(r) => r,
            Err(quinn::ConnectionError::ApplicationClosed(_))
            | Err(quinn::ConnectionError::ConnectionClosed(_))
            | Err(quinn::ConnectionError::LocallyClosed)
            | Err(quinn::ConnectionError::TimedOut) => return Ok(()),
            Err(e) => return Err(e.into()),
        };
        let bytes = match recv_frame(&mut recv).await {
            Ok(b) => b,
            Err(_) => continue,
        };
        let frame = match OverlayFrame::decode(&bytes) {
            Ok(f) => f,
            Err(_) => continue, // мусорный фрейм — соединение живо, игнор
        };

        let route = {
            let g = reg.lock().unwrap();
            g.postman.route(conn_id, frame)
        };
        match route {
            Route::Deliver { conn: dst, frame } | Route::AckToSender { conn: dst, frame } => {
                let target = reg.lock().unwrap().conns.get(&dst).cloned();
                if let Some(c) = target {
                    if let Ok(mut s) = c.open_uni().await {
                        let _ = send_frame(&mut s, &frame.to_bytes()).await;
                    }
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
