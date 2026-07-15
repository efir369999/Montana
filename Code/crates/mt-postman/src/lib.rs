//! mt-postman — плоский TCP+TLS-транспорт почтальона (Montana P2P Network, Этап 1;
//! спека §152 — TCP/TLS-443 обязателен, операторы режут non-443 UDP).
//!
//! Сервер-почтальон принимает TCP+TLS-соединения, гоняет прологовое рукопожатие
//! (RegHello/RegChallenge/RegProof, ML-DSA-65 — mt-overlay::prologue/challenge) и
//! маршрутизирует OverlayFrame по overlay_addr (RELAY→DELIVER/Buffer, ACK — Postman).
//! Транспортный хоп — TCP+TLS 1.3 (admission-обёртка A-3 по [I-16]); реальная security —
//! opaque E2E-payload + ML-DSA-подпись регистрации. Не consensus state.

pub mod client;
pub mod config;
pub mod muq;
pub mod muq_client;
pub mod node;
pub mod path;
pub mod server;
pub mod wire;

pub use client::{ClientError, PostmanClient};
pub use config::{stand_client_config, stand_server_config, ConfigError, STAND_SNI};
pub use muq::{MuqState, TAG_HOST_DEPOSIT};
pub use muq_client::{node_hello, MuqClient};
pub use node::Node;
pub use server::{PostmanServer, ServerError};
pub use wire::WireError;
