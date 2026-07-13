//! Node — единая сущность узла Montana (P2P Network Этап 3). Один QUIC-Endpoint: узел
//! ОДНОВРЕМЕННО слушает (host держит очереди + courier релеит чужое) И инициирует
//! соединения (client — шлёт/получает своё) — через одну неразличимую дверь. Нет
//! разделения на «сервер-почтальон» и «клиент»: доступность (always-on десктоп /
//! foreground карман) — режим развёртывания, не разные роли в коде. Сущность узла одна.

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use quinn::Endpoint;

use mt_crypto::{MlkemPublicKey, SecretKey};
use mt_overlay::muq::{ProxyForward, Queue, QueueId, QueueResp};
use mt_overlay::OverlayAddr;

use crate::client::ClientError;
use crate::config::{stand_client_config, stand_server_config, STAND_SNI};
use crate::muq::MuqState;
use crate::muq_client::{
    muq_deposit, muq_register, muq_register_via_courier, muq_subscribe_via_courier,
};
use crate::server::{handle_connection, Registry, ServerError};

#[derive(Clone)]
pub struct Node {
    endpoint: Endpoint,
    reg: Arc<Mutex<Registry>>,
    muq: Arc<MuqState>,
}

impl Node {
    /// Поднять узел: один Endpoint — и слушает (server), и звонит (client).
    pub fn bind(addr: SocketAddr) -> Result<Self, ServerError> {
        let mut endpoint = Endpoint::server(stand_server_config()?, addr)?;
        endpoint.set_default_client_config(stand_client_config()?);
        Ok(Self {
            endpoint,
            reg: Arc::new(Mutex::new(Registry::default())),
            muq: Arc::new(MuqState::new()),
        })
    }

    pub fn local_addr(&self) -> Result<SocketAddr, ServerError> {
        Ok(self.endpoint.local_addr()?)
    }

    /// MUQ-состояние узла (queue-host + courier/proxy-маршруты).
    pub fn muq(&self) -> &Arc<MuqState> {
        &self.muq
    }

    /// Роль courier: сопоставить overlay queue-host его физическому адресу (релей-маршрут).
    pub fn add_courier_route(&self, host_overlay: OverlayAddr, addr: SocketAddr) {
        self.muq.add_proxy_route(host_overlay, addr);
    }

    /// Неразличимая дверь: принимать входящие соединения (host + courier + relay).
    /// Тот же Endpoint используется client-методами — узел и слушает, и звонит.
    pub async fn run(&self) {
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

    // --- своя активность через тот же Endpoint (client-роль единой сущности) ---

    /// Зарегистрировать свою очередь на узле-хосте.
    pub async fn register_queue_on(
        &self,
        host: SocketAddr,
        q: &Queue,
    ) -> Result<bool, ClientError> {
        let conn = self.endpoint.connect(host, STAND_SNI)?.await?;
        let ok = muq_register(&conn, q).await?;
        conn.close(0u32.into(), b"done");
        Ok(ok)
    }

    /// Relay-регистрация очереди на чужом хосте через курьер (host видит курьера, не нас).
    /// Для своего узла (self-host) — register_queue_on(self_addr) без утечки (свой узел).
    pub async fn register_via_courier(
        &self,
        courier: SocketAddr,
        host_overlay: OverlayAddr,
        host_kem_pk: &MlkemPublicKey,
        q: &Queue,
    ) -> Result<bool, ClientError> {
        let conn = self.endpoint.connect(courier, STAND_SNI)?.await?;
        let ok = muq_register_via_courier(&conn, host_overlay, host_kem_pk, q).await?;
        conn.close(0u32.into(), b"done");
        Ok(ok)
    }

    /// Положить депозит через узел-courier (двуххоп; courier не видит recv_id).
    pub async fn deposit_via(
        &self,
        courier: SocketAddr,
        pf: &ProxyForward,
    ) -> Result<bool, ClientError> {
        let conn = self.endpoint.connect(courier, STAND_SNI)?.await?;
        let ok = muq_deposit(&conn, pf).await?;
        conn.close(0u32.into(), b"done");
        Ok(ok)
    }

    /// Двуххоп-ВЫБОРКА: забрать свои осколки ЧЕРЕЗ узел-courier (host видит курьера, не нас).
    /// Закрывает получателя от хоста симметрично отправителю (Этап 3).
    pub async fn subscribe_via_courier(
        &self,
        courier: SocketAddr,
        host_overlay: OverlayAddr,
        host_kem_pk: &MlkemPublicKey,
        recv_id: QueueId,
        recv_sk: &SecretKey,
    ) -> Result<QueueResp, ClientError> {
        let conn = self.endpoint.connect(courier, STAND_SNI)?.await?;
        let resp =
            muq_subscribe_via_courier(&conn, host_overlay, host_kem_pk, recv_id, recv_sk).await?;
        conn.close(0u32.into(), b"done");
        Ok(resp)
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
