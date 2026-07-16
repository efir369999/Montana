//! Montana Messenger E2E — общий Rust-движок сквозного шифрования.
//! Спека Montana Messenger v0.40.0. Клиенты (iOS/Web/Python) линкуют этот крейт,
//! а не реимплементируют крипту — байт-идентичность заперта KAT-векторами спеки.
//!
//! - `pqxdh`   — Этап 5: установление постквантовой сессии (ML-KEM-768 PQXDH).
//! - `ratchet` — Этап 6: двойной храповик (KEM-храповик), AEAD сообщений.
//! - `labels`  — Этап 7: слепая доставка, вращающиеся метки очередей.
//! - `safety`  — Этап 8: сверка отпечатка личности (safety number).
//! - `call`    — Этап 13: постквантовый медиа-ключ звонка (SFrame поверх SRTP).
//! - `content` — Этап 9: кодек Content личного чата 1-на-1.
//! - `contacts` — Этап 11: @имя-заявка, ключ контактов, ContactRecord/List.
//! - `device_registry` — Этап 10: подписанный реестр устройств (мульти-девайс).

pub mod archive;
pub mod call;
pub mod contacts;
pub mod content;
pub mod crypto;
pub mod device_registry;
pub mod handshake;
pub mod kdf;
pub mod labels;
pub mod media;
pub mod pqxdh;
pub mod ratchet;
pub mod safety;
pub mod sealed;
pub mod session;
