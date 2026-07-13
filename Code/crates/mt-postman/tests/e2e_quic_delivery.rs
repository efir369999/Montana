//! Running-тест Этапа 1: A→B→ACK через реальный QUIC-сокет и почтальон,
//! ноль наших серверов. Два независимых quinn-клиента + сервер-почтальон
//! общаются через ОС loopback (не in-process канал) — доказывает реальный
//! сетевой путь: A --RELAY--> почтальон --DELIVER--> B --ACK--> почтальон --> A.

use std::time::Duration;

use mt_crypto::{keypair_from_seed, PublicKey, SecretKey, PUBLIC_KEY_SIZE};
use mt_overlay::frame::{FrameType, MsgId};
use mt_overlay::{overlay_addr, OverlayAddr};
use mt_postman::{PostmanClient, PostmanServer};
use mt_state::{derive_account_id, SUITE_MLDSA65};

fn ident(seed: u8) -> ([u8; PUBLIC_KEY_SIZE], SecretKey, OverlayAddr) {
    let (pk, sk): (PublicKey, SecretKey) = keypair_from_seed(&[seed; 32]).unwrap();
    let pkb = *pk.as_bytes();
    let addr = overlay_addr(&derive_account_id(SUITE_MLDSA65, &pkb));
    (pkb, sk, addr)
}

async fn with_timeout<F: std::future::Future>(f: F) -> F::Output {
    tokio::time::timeout(Duration::from_secs(10), f)
        .await
        .expect("операция не должна виснуть >10s")
}

#[tokio::test]
async fn a_to_b_to_ack_over_real_quic() {
    // Почтальон на 127.0.0.1:0 (ОС выберет порт) — «коробка, которую держишь ты».
    let server = PostmanServer::bind("127.0.0.1:0".parse().unwrap()).unwrap();
    let postman_addr = server.local_addr().unwrap();
    tokio::spawn(server.run());

    let (pk_a, sk_a, addr_a) = ident(0xA1);
    let (pk_b, sk_b, addr_b) = ident(0xB2);

    // A и B подключаются к почтальону и регистрируют overlay_addr по ML-DSA.
    let client_a = with_timeout(PostmanClient::connect(postman_addr, pk_a, &sk_a))
        .await
        .expect("A регистрируется");
    let mut client_b = with_timeout(PostmanClient::connect(postman_addr, pk_b, &sk_b))
        .await
        .expect("B регистрируется");

    assert_eq!(client_a.overlay(), addr_a);
    assert_eq!(client_b.overlay(), addr_b);

    // A шлёт RELAY→B с непрозрачным «E2E-конвертом».
    let msg_id: MsgId = [0x77; 16];
    let envelope = b"sealed-e2e-envelope-A-to-B".to_vec();
    with_timeout(client_a.send_relay(addr_b, msg_id, envelope.clone()))
        .await
        .expect("A шлёт RELAY");

    // B получает DELIVER (тот же msg_id, тот же payload; почтальон payload не трогал).
    let delivered = with_timeout(client_b.recv())
        .await
        .expect("B получает DELIVER");
    assert_eq!(delivered.frame_type, FrameType::Deliver);
    assert_eq!(delivered.msg_id, msg_id);
    assert_eq!(delivered.dst_overlay, addr_b);
    assert_eq!(delivered.payload, envelope);

    // B отвечает ACK по тому же msg_id → почтальон маршрутизирует назад A.
    with_timeout(client_b.send_ack(delivered.src_overlay, msg_id))
        .await
        .expect("B шлёт ACK");

    let mut client_a = client_a;
    let ack = with_timeout(client_a.recv()).await.expect("A получает ACK");
    assert_eq!(ack.frame_type, FrameType::Ack);
    assert_eq!(ack.msg_id, msg_id);
    assert_eq!(ack.dst_overlay, addr_a);
    assert!(ack.payload.is_empty());
}

#[tokio::test]
async fn relay_to_offline_b_does_not_reach_a_as_deliver() {
    // B офлайн (не подключён) → почтальон буферизует (Этап 2), A не получает ложный DELIVER.
    let server = PostmanServer::bind("127.0.0.1:0".parse().unwrap()).unwrap();
    let postman_addr = server.local_addr().unwrap();
    tokio::spawn(server.run());

    let (pk_a, sk_a, _addr_a) = ident(0xA1);
    let (_pk_b, _sk_b, addr_b) = ident(0xB2);

    let mut client_a = with_timeout(PostmanClient::connect(postman_addr, pk_a, &sk_a))
        .await
        .expect("A регистрируется");

    with_timeout(client_a.send_relay(addr_b, [0x01; 16], b"x".to_vec()))
        .await
        .expect("A шлёт RELAY в офлайн-B");

    // A ничего не должен получить (нет эха, нет ложного DELIVER) в разумное окно.
    let got = tokio::time::timeout(Duration::from_millis(700), client_a.recv()).await;
    assert!(
        got.is_err(),
        "A не должен получать ничего при офлайн-B на Этапе 1"
    );
}

#[tokio::test]
async fn duplicate_msg_id_delivered_once() {
    // §396: A шлёт один msg_id дважды → B получает DELIVER ровно один раз.
    let server = PostmanServer::bind("127.0.0.1:0".parse().unwrap()).unwrap();
    let postman_addr = server.local_addr().unwrap();
    tokio::spawn(server.run());

    let (pk_a, sk_a, _addr_a) = ident(0xA1);
    let (pk_b, sk_b, addr_b) = ident(0xB2);

    let client_a = with_timeout(PostmanClient::connect(postman_addr, pk_a, &sk_a))
        .await
        .expect("A регистрируется");
    let mut client_b = with_timeout(PostmanClient::connect(postman_addr, pk_b, &sk_b))
        .await
        .expect("B регистрируется");

    let msg_id: MsgId = [0x55; 16];
    for _ in 0..2 {
        with_timeout(client_a.send_relay(addr_b, msg_id, b"dup".to_vec()))
            .await
            .expect("A шлёт RELAY (дубль)");
    }

    let first = with_timeout(client_b.recv()).await.expect("первый DELIVER");
    assert_eq!(first.msg_id, msg_id);
    // Второй DELIVER не приходит — отброшен дедупом.
    let second = tokio::time::timeout(Duration::from_millis(700), client_b.recv()).await;
    assert!(
        second.is_err(),
        "дубликат msg_id должен быть отброшен на приёмнике"
    );
}
