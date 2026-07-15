//! Node — единая сущность узла Montana (P2P Network Этап 3; TCP+TLS-транспорт, спека §152).
//! Узел ОДНОВРЕМЕННО слушает (host держит очереди + courier релеит чужое) И инициирует
//! соединения (client — шлёт/получает своё) — через одну неразличимую дверь. Нет разделения
//! на «сервер-почтальон» и «клиент»: доступность (always-on десктоп / foreground карман) —
//! режим развёртывания, не разные роли в коде. Каждая клиентская операция — свежее TCP+TLS
//! соединение (MuqClient).

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

use mt_crypto::{MlkemPublicKey, SecretKey};
use mt_overlay::muq::{ProxyForward, Queue, QueueId, QueueResp};
use mt_overlay::OverlayAddr;

use crate::client::ClientError;
use crate::config::tls_acceptor;
use crate::muq::MuqState;
use crate::muq_client::MuqClient;
use crate::server::{handle_connection, Registry, ServerError};

#[derive(Clone)]
pub struct Node {
    listener: Arc<TcpListener>,
    acceptor: TlsAcceptor,
    reg: Arc<Mutex<Registry>>,
    muq: Arc<MuqState>,
}

impl Node {
    /// Поднять узел: один TCP-listener — и слушает (server), и звонит (client-методы).
    pub async fn bind(addr: SocketAddr) -> Result<Self, ServerError> {
        let listener = TcpListener::bind(addr).await?;
        Ok(Self {
            listener: Arc::new(listener),
            acceptor: tls_acceptor()?,
            reg: Arc::new(Mutex::new(Registry::default())),
            muq: Arc::new(MuqState::new()),
        })
    }

    pub fn local_addr(&self) -> Result<SocketAddr, ServerError> {
        Ok(self.listener.local_addr()?)
    }

    /// MUQ-состояние узла (queue-host + courier/proxy-маршруты).
    pub fn muq(&self) -> &Arc<MuqState> {
        &self.muq
    }

    /// Роль courier: сопоставить overlay queue-host его физическому адресу (релей-маршрут).
    pub fn add_courier_route(&self, host_overlay: OverlayAddr, addr: SocketAddr) {
        self.muq.add_proxy_route(host_overlay, addr);
    }

    /// Неразличимая дверь: принимать входящие TCP+TLS-соединения (host + courier + relay).
    pub async fn run(&self) {
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

    // --- своя активность (client-роль единой сущности): свежее TCP+TLS на операцию ---

    /// Зарегистрировать свою очередь на узле-хосте.
    pub async fn register_queue_on(
        &self,
        host: SocketAddr,
        q: &Queue,
    ) -> Result<bool, ClientError> {
        // DEV-051 / §534: прямая регистрация только к своему узлу (self-host = loopback).
        // На чужом хосте раскрывается сетевая личность → register_via_courier.
        if !host.ip().is_loopback() {
            return Err(ClientError::ForeignHostRegistration);
        }
        MuqClient::connect(host).await?.register_queue(q).await
    }

    /// Relay-регистрация очереди на чужом хосте через курьер (host видит курьера, не нас).
    pub async fn register_via_courier(
        &self,
        courier: SocketAddr,
        host_overlay: OverlayAddr,
        host_kem_pk: &MlkemPublicKey,
        q: &Queue,
    ) -> Result<bool, ClientError> {
        MuqClient::connect(courier)
            .await?
            .register_via_courier(host_overlay, host_kem_pk, q)
            .await
    }

    /// Положить депозит через узел-courier (двуххоп; courier не видит recv_id).
    pub async fn deposit_via(
        &self,
        courier: SocketAddr,
        pf: &ProxyForward,
    ) -> Result<bool, ClientError> {
        MuqClient::connect(courier)
            .await?
            .deposit_via_proxy(pf)
            .await
    }

    /// Двуххоп-ВЫБОРКА: забрать свои осколки ЧЕРЕЗ узел-courier (host видит курьера, не нас).
    pub async fn subscribe_via_courier(
        &self,
        courier: SocketAddr,
        host_overlay: OverlayAddr,
        host_kem_pk: &MlkemPublicKey,
        recv_id: QueueId,
        recv_sk: &SecretKey,
    ) -> Result<QueueResp, ClientError> {
        MuqClient::connect(courier)
            .await?
            .subscribe_via_courier(host_overlay, host_kem_pk, recv_id, recv_sk)
            .await
    }

    /// SELF-HOST (абсолют против сговора): забрать из СВОЕЙ очереди локально, без курьера
    /// и без сети. Курьеров нет → сговаривать нечего → получателя не видит НИКТО.
    pub fn subscribe_local(&self, recv_id: &QueueId) -> QueueResp {
        QueueResp {
            items: self.muq.local_drain(recv_id),
        }
    }

    /// Публичный ML-KEM ключ хоста — для запечатывания sealed к этому узлу.
    pub fn host_kem_pubkey(&self) -> MlkemPublicKey {
        self.muq.host_kem_pubkey()
    }
}
