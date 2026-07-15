//! Running-тест Этапа 3: узел — единая сущность (Node), non-collusion relay по умолчанию.
//! Регистрация И выборка через курьеров (host не видит B); прямых B↔host операций нет,
//! кроме self-host (свой узел). Абсолют против сговора — self-host (локальная выборка).

use std::time::Duration;

use mt_overlay::muq::{derive_queue_keypairs, sign_deposit, HostDeposit, ProxyForward, Queue};
use mt_overlay::OverlayAddr;
use mt_postman::Node;

async fn with_timeout<F: std::future::Future>(f: F) -> F::Output {
    tokio::time::timeout(Duration::from_secs(10), f)
        .await
        .expect("не должно виснуть >10s")
}

fn make_queue(
    recv_id: [u8; 32],
    send_id: [u8; 32],
) -> (Queue, mt_crypto::SecretKey, mt_crypto::SecretKey) {
    let rs = [0x42u8; 32];
    let ((recv_pk, recv_sk), (send_pk, send_sk)) = derive_queue_keypairs(&rs, 0).unwrap();
    let q = Queue {
        recv_id,
        send_id,
        recv_pubkey: *recv_pk.as_bytes(),
        send_pubkey: Some(*send_pk.as_bytes()),
        rotation_epoch: 1000,
        quota: 64,
    };
    (q, recv_sk, send_sk)
}

fn build_deposit(
    send_id: [u8; 32],
    send_sk: &mt_crypto::SecretKey,
    msg_id: [u8; 16],
    ct: Vec<u8>,
    host_overlay: OverlayAddr,
    host_kem_pk: &mt_crypto::MlkemPublicKey,
) -> ProxyForward {
    let nonce = [0x07u8; 16];
    let sig = sign_deposit(send_sk, &send_id, &msg_id, &nonce).unwrap();
    let hd = HostDeposit {
        send_id,
        msg_id,
        ttl_windows: 240,
        shard_index: 0,
        shard_total: 1,
        nonce,
        ct,
        sig: *sig.as_bytes(),
    };
    let sealed = mt_crypto::seal_to(host_kem_pk, &hd.to_bytes()).unwrap();
    ProxyForward {
        host_addr: host_overlay,
        sealed,
    }
}

fn spawn(n: &Node) {
    let n = n.clone();
    tokio::spawn(async move { n.run().await });
}

#[tokio::test]
async fn node_is_one_entity_host_courier_client() {
    // Три узла — все Node (одна сущность). Регистрация+депозит+выборка через courier-узел.
    let host = Node::bind("127.0.0.1:0".parse().unwrap()).await.unwrap();
    let host_addr = host.local_addr().unwrap();
    let host_kem = host.host_kem_pubkey();
    let courier = Node::bind("127.0.0.1:0".parse().unwrap()).await.unwrap();
    let courier_addr = courier.local_addr().unwrap();
    let sender = Node::bind("127.0.0.1:0".parse().unwrap()).await.unwrap();
    spawn(&host);
    spawn(&courier);
    spawn(&sender);

    let host_overlay: OverlayAddr = [0xA0u8; 32];
    courier.add_courier_route(host_overlay, host_addr);

    let recv_id = [0x71u8; 32];
    let send_id = [0x51u8; 32];
    let (q, recv_sk, send_sk) = make_queue(recv_id, send_id);

    assert!(
        with_timeout(sender.register_via_courier(courier_addr, host_overlay, &host_kem, &q))
            .await
            .unwrap()
    );
    let msg_id = [0x5Au8; 16];
    let ct = b"one-entity-node-delivery".to_vec();
    let pf = build_deposit(
        send_id,
        &send_sk,
        msg_id,
        ct.clone(),
        host_overlay,
        &host_kem,
    );
    assert!(with_timeout(sender.deposit_via(courier_addr, &pf))
        .await
        .unwrap());

    let resp = with_timeout(sender.subscribe_via_courier(
        courier_addr,
        host_overlay,
        &host_kem,
        recv_id,
        &recv_sk,
    ))
    .await
    .unwrap();
    assert_eq!(
        resp.items.len(),
        1,
        "депозит через courier-узел дошёл до host-узла"
    );
    assert_eq!(resp.items[0].ct, ct);
}

#[tokio::test]
async fn two_hop_retrieval_hides_receiver_from_host() {
    // host + курьер депозита + курьер выборки (РАЗНЫЕ хозяева, non-collusion). host не
    // видит B ни при регистрации, ни при выборке.
    let host = Node::bind("127.0.0.1:0".parse().unwrap()).await.unwrap();
    let host_addr = host.local_addr().unwrap();
    let host_kem = host.host_kem_pubkey();
    let dep_courier = Node::bind("127.0.0.1:0".parse().unwrap()).await.unwrap();
    let dep_addr = dep_courier.local_addr().unwrap();
    let recv_courier = Node::bind("127.0.0.1:0".parse().unwrap()).await.unwrap();
    let recv_addr = recv_courier.local_addr().unwrap();
    let peer = Node::bind("127.0.0.1:0".parse().unwrap()).await.unwrap();
    for n in [&host, &dep_courier, &recv_courier, &peer] {
        spawn(n);
    }

    let host_overlay: OverlayAddr = [0xA0u8; 32];
    dep_courier.add_courier_route(host_overlay, host_addr);
    recv_courier.add_courier_route(host_overlay, host_addr);

    let recv_id = [0x71u8; 32];
    let send_id = [0x51u8; 32];
    let (q, recv_sk, send_sk) = make_queue(recv_id, send_id);

    // регистрация через курьер выборки, депозит через курьер депозита (разные хозяева)
    assert!(
        with_timeout(peer.register_via_courier(recv_addr, host_overlay, &host_kem, &q))
            .await
            .unwrap()
    );
    let msg_id = [0x5Au8; 16];
    let ct = b"receiver-hidden-from-host".to_vec();
    let pf = build_deposit(
        send_id,
        &send_sk,
        msg_id,
        ct.clone(),
        host_overlay,
        &host_kem,
    );
    assert!(with_timeout(peer.deposit_via(dep_addr, &pf)).await.unwrap());

    let resp = with_timeout(peer.subscribe_via_courier(
        recv_addr,
        host_overlay,
        &host_kem,
        recv_id,
        &recv_sk,
    ))
    .await
    .unwrap();
    assert_eq!(resp.items.len(), 1);
    assert_eq!(resp.items[0].ct, ct);
}

#[tokio::test]
async fn self_host_absolute_no_courier() {
    // Абсолют против сговора: получатель держит очередь на СВОЁМ узле, забирает ЛОКАЛЬНО.
    // Регистрация прямая — но к СВОЕМУ узлу (host = я), утечки нет.
    let me = Node::bind("127.0.0.1:0".parse().unwrap()).await.unwrap();
    let me_addr = me.local_addr().unwrap();
    spawn(&me);
    let me_overlay: OverlayAddr = [0xA0u8; 32];
    me.add_courier_route(me_overlay, me_addr);

    let recv_id = [0x71u8; 32];
    let send_id = [0x51u8; 32];
    let (q, _recv_sk, send_sk) = make_queue(recv_id, send_id);
    // регистрация на СВОЁМ узле — прямая допустима (свой узел)
    assert!(with_timeout(me.register_queue_on(me_addr, &q))
        .await
        .unwrap());

    let msg_id = [0x5Cu8; 16];
    let ct = b"self-hosted-nobody-sees".to_vec();
    let pf = build_deposit(
        send_id,
        &send_sk,
        msg_id,
        ct.clone(),
        me_overlay,
        &me.host_kem_pubkey(),
    );
    assert!(with_timeout(me.deposit_via(me_addr, &pf)).await.unwrap());

    // ЛОКАЛЬНАЯ выборка — без курьера, без сети; получателя не видит никто
    let resp = me.subscribe_local(&recv_id);
    assert_eq!(resp.items.len(), 1, "self-host: забрал локально");
    assert_eq!(resp.items[0].ct, ct);
    assert!(
        me.subscribe_local(&recv_id).items.is_empty(),
        "drop-on-delivery"
    );
}
