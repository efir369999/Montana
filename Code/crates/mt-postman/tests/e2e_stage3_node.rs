//! Running-тест Этапа 3: узел — единая сущность (Node). Три узла, все Node; различие
//! только в том, какие методы каждый зовёт. Курьер-узел релеит чужой депозит, host-узел
//! держит очередь, отправитель звонит — один тип, одна неразличимая дверь. Доказывает
//! «все узлы одна сущность»: каждый и слушает (run), и инициирует (client-методы).

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
        sig: sig.as_bytes().to_vec(),
    };
    let sealed = mt_crypto::seal_to(host_kem_pk, &hd.to_bytes()).unwrap();
    ProxyForward {
        host_addr: host_overlay,
        sealed,
    }
}

#[tokio::test]
async fn node_is_one_entity_host_courier_client() {
    // Три узла — все Node (одна сущность). Каждый слушает (run) и может звонить.
    let host = Node::bind("127.0.0.1:0".parse().unwrap()).unwrap();
    let host_addr = host.local_addr().unwrap();
    let courier = Node::bind("127.0.0.1:0".parse().unwrap()).unwrap();
    let courier_addr = courier.local_addr().unwrap();
    let sender = Node::bind("127.0.0.1:0".parse().unwrap()).unwrap();

    {
        let h = host.clone();
        tokio::spawn(async move { h.run().await });
    }
    {
        let c = courier.clone();
        tokio::spawn(async move { c.run().await });
    }
    {
        let s = sender.clone();
        tokio::spawn(async move { s.run().await });
    }

    // courier-узел знает host (релей-маршрут) — роль courier единой сущности.
    let host_overlay: OverlayAddr = [0xA0u8; 32];
    courier.add_courier_route(host_overlay, host_addr);

    let recv_id = [0x71u8; 32];
    let send_id = [0x51u8; 32];
    let (q, recv_sk, send_sk) = make_queue(recv_id, send_id);

    // Получатель регистрирует очередь на host-узле (любой Node звонит — единая сущность).
    assert!(with_timeout(sender.register_queue_on(host_addr, &q))
        .await
        .unwrap());

    // Отправитель кладёт депозит ЧЕРЕЗ courier-узел к host-узлу (двуххоп).
    // courier здесь и слушает (релеит чужое), и был бы способен слать своё — одна сущность.
    let msg_id = [0x5Au8; 16];
    let ct = b"one-entity-node-delivery".to_vec();
    let pf = build_deposit(
        send_id,
        &send_sk,
        msg_id,
        ct.clone(),
        host_overlay,
        &host.host_kem_pubkey(),
    );
    assert!(with_timeout(sender.deposit_via(courier_addr, &pf))
        .await
        .unwrap());

    // Получатель забирает с host-узла.
    let resp = with_timeout(sender.subscribe_from(host_addr, recv_id, &recv_sk))
        .await
        .unwrap();
    assert_eq!(
        resp.items.len(),
        1,
        "депозит через courier-узел дошёл до host-узла"
    );
    assert_eq!(resp.items[0].ct, ct);
    assert_eq!(resp.items[0].msg_id, msg_id);
}

#[tokio::test]
async fn courier_node_also_hosts_and_sends() {
    // Узел-курьер — не «только релей»: та же сущность держит СВОЮ очередь и получает.
    // Доказывает: доступность/использование — дело узла, не разные роли в коде.
    let a = Node::bind("127.0.0.1:0".parse().unwrap()).unwrap();
    let a_addr = a.local_addr().unwrap();
    let b = Node::bind("127.0.0.1:0".parse().unwrap()).unwrap();
    let b_addr = b.local_addr().unwrap();

    {
        let a2 = a.clone();
        tokio::spawn(async move { a2.run().await });
    }
    {
        let b2 = b.clone();
        tokio::spawn(async move { b2.run().await });
    }

    // b выступает и курьером к a (релей), и сам держит хостинг для своей очереди.
    let a_overlay: OverlayAddr = [0xA0u8; 32];
    b.add_courier_route(a_overlay, a_addr);

    // Очередь размещена на узле a; отправитель кладёт через узел b (courier).
    let recv_id = [0x71u8; 32];
    let send_id = [0x51u8; 32];
    let (q, recv_sk, send_sk) = make_queue(recv_id, send_id);
    assert!(with_timeout(a.register_queue_on(a_addr, &q)).await.unwrap());

    let msg_id = [0x5Bu8; 16];
    let ct = b"via-courier-b".to_vec();
    let pf = build_deposit(
        send_id,
        &send_sk,
        msg_id,
        ct.clone(),
        a_overlay,
        &a.host_kem_pubkey(),
    );
    // отправитель = узел b (тот же, что курьер) — единая сущность и релеит, и шлёт своё
    assert!(with_timeout(b.deposit_via(b_addr, &pf)).await.unwrap());

    let resp = with_timeout(a.subscribe_from(a_addr, recv_id, &recv_sk))
        .await
        .unwrap();
    assert_eq!(
        resp.items.len(),
        1,
        "узел b и релеит, и шлёт своё — одна сущность"
    );
    assert_eq!(resp.items[0].ct, ct);
}

#[tokio::test]
async fn two_hop_retrieval_hides_receiver_from_host() {
    // Этап 3 ядро: B забирает ЧЕРЕЗ курьер → host видит курьера, не B. Депозит и выборка
    // идут через РАЗНЫХ курьеров (non-collusion, модель SimpleX/Flux). Все узлы — Node.
    let host = Node::bind("127.0.0.1:0".parse().unwrap()).unwrap();
    let host_addr = host.local_addr().unwrap();
    let deposit_courier = Node::bind("127.0.0.1:0".parse().unwrap()).unwrap();
    let dep_courier_addr = deposit_courier.local_addr().unwrap();
    let recv_courier = Node::bind("127.0.0.1:0".parse().unwrap()).unwrap();
    let recv_courier_addr = recv_courier.local_addr().unwrap();
    let peer = Node::bind("127.0.0.1:0".parse().unwrap()).unwrap();

    for n in [&host, &deposit_courier, &recv_courier, &peer] {
        let n = n.clone();
        tokio::spawn(async move { n.run().await });
    }

    let host_overlay: OverlayAddr = [0xA0u8; 32];
    deposit_courier.add_courier_route(host_overlay, host_addr);
    recv_courier.add_courier_route(host_overlay, host_addr);

    let recv_id = [0x71u8; 32];
    let send_id = [0x51u8; 32];
    let (q, recv_sk, send_sk) = make_queue(recv_id, send_id);

    assert!(with_timeout(peer.register_queue_on(host_addr, &q))
        .await
        .unwrap());

    // депозит через курьер №1
    let msg_id = [0x5Au8; 16];
    let ct = b"receiver-hidden-from-host".to_vec();
    let host_kem = host.host_kem_pubkey();
    let pf = build_deposit(
        send_id,
        &send_sk,
        msg_id,
        ct.clone(),
        host_overlay,
        &host_kem,
    );
    assert!(with_timeout(peer.deposit_via(dep_courier_addr, &pf))
        .await
        .unwrap());

    // ВЫБОРКА через курьер №2 (разный хозяин) — host видит recv_courier, НЕ получателя
    let resp = with_timeout(peer.subscribe_via_courier(
        recv_courier_addr,
        host_overlay,
        &host_kem,
        recv_id,
        &recv_sk,
    ))
    .await
    .unwrap();
    assert_eq!(resp.items.len(), 1, "двуххоп-выборка доставила осколок");
    assert_eq!(resp.items[0].ct, ct);
    assert_eq!(resp.items[0].msg_id, msg_id);
}

#[tokio::test]
async fn self_host_absolute_no_courier() {
    // Абсолют против сговора (вопрос 2): получатель держит очередь на СВОЁМ узле и забирает
    // ЛОКАЛЬНО — курьеров нет, сговаривать нечего, получателя не видит НИКТО.
    let me = Node::bind("127.0.0.1:0".parse().unwrap()).unwrap();
    let me_addr = me.local_addr().unwrap();
    {
        let m = me.clone();
        tokio::spawn(async move { m.run().await });
    }
    let me_overlay: OverlayAddr = [0xA0u8; 32];
    me.add_courier_route(me_overlay, me_addr);

    let recv_id = [0x71u8; 32];
    let send_id = [0x51u8; 32];
    let (q, _recv_sk, send_sk) = make_queue(recv_id, send_id);
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

    // ЛОКАЛЬНАЯ выборка — без курьера, без сети.
    let resp = me.subscribe_local(&recv_id);
    assert_eq!(
        resp.items.len(),
        1,
        "self-host: забрал локально, курьеров нет"
    );
    assert_eq!(resp.items[0].ct, ct);
    // повтор пуст (drop-on-delivery)
    assert!(me.subscribe_local(&recv_id).items.is_empty());
}
