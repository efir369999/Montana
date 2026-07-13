//! Running-тест Этапа 2 (MUQ store-and-forward по реальному QUIC): офлайн-доставка через
//! двуххоп A→proxy→host→буфер (B забирает при открытии) + durability RS(2,3) (B собирает
//! из 2 хостов = третий выключен). Несвязываемость: host видит эфемерный ключ очереди и
//! recv_id, НЕ account_id; proxy видит send_id, НЕ recv_id.

use std::time::Duration;

use mt_overlay::erasure::{rs_reconstruct, rs_split};
use mt_overlay::muq::{derive_queue_keypairs, sign_deposit, HostDeposit, ProxyForward, Queue};
use mt_overlay::OverlayAddr;
use mt_postman::{MuqClient, PostmanServer};

async fn with_timeout<F: std::future::Future>(f: F) -> F::Output {
    tokio::time::timeout(Duration::from_secs(10), f)
        .await
        .expect("не должно виснуть >10s")
}

fn make_queue(
    recv_id: [u8; 32],
    send_id: [u8; 32],
) -> (Queue, mt_crypto::SecretKey, mt_crypto::SecretKey) {
    // routing_secret — общий корень сессии (OOB, из PQXDH в реале). Ключи очереди эфемерны (M-1).
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
    shard_index: u8,
    shard_total: u8,
    ct: Vec<u8>,
    host_overlay: OverlayAddr,
) -> ProxyForward {
    let nonce = [0x07u8; 16];
    let sig = sign_deposit(send_sk, &send_id, &msg_id, &nonce).unwrap();
    let hd = HostDeposit {
        send_id,
        msg_id,
        ttl_windows: 240,
        shard_index,
        shard_total,
        nonce,
        ct,
        sig: sig.as_bytes().to_vec(),
    };
    ProxyForward {
        host_addr: host_overlay,
        sealed: hd.to_bytes(),
    }
}

#[tokio::test]
async fn offline_delivery_two_hop_deposit() {
    // host (queue-host) + proxy (entry-proxy) на стенде.
    let host = PostmanServer::bind("127.0.0.1:0".parse().unwrap()).unwrap();
    let host_addr = host.local_addr().unwrap();
    let proxy = PostmanServer::bind("127.0.0.1:0".parse().unwrap()).unwrap();
    let proxy_addr = proxy.local_addr().unwrap();
    let host_overlay: OverlayAddr = [0xA0u8; 32];
    proxy.muq().add_proxy_route(host_overlay, host_addr);
    tokio::spawn(host.run());
    tokio::spawn(proxy.run());

    let recv_id = [0x71u8; 32];
    let send_id = [0x51u8; 32];
    let (q, recv_sk, send_sk) = make_queue(recv_id, send_id);

    // B регистрирует очередь и уходит офлайн (не подписан).
    let b_reg = with_timeout(MuqClient::connect(host_addr)).await.unwrap();
    assert!(with_timeout(b_reg.register_queue(&q)).await.unwrap());
    drop(b_reg);

    // A кладёт депозит через proxy при закрытом B (двуххоп).
    let msg_id = [0x5Au8; 16];
    let ct = b"sealed-e2e-envelope-for-sleeping-B".to_vec();
    let pf = build_deposit(send_id, &send_sk, msg_id, 0, 1, ct.clone(), host_overlay);
    let a = with_timeout(MuqClient::connect(proxy_addr)).await.unwrap();
    assert!(with_timeout(a.deposit_via_proxy(&pf)).await.unwrap());

    // B открывается и выбирает — конверт дошёл, хотя B был офлайн при отправке.
    let b = with_timeout(MuqClient::connect(host_addr)).await.unwrap();
    let resp = with_timeout(b.subscribe(recv_id, &recv_sk)).await.unwrap();
    assert_eq!(resp.items.len(), 1, "осколок должен лежать в буфере");
    assert_eq!(resp.items[0].ct, ct, "конверт byte-exact");
    assert_eq!(resp.items[0].msg_id, msg_id);

    // drop-on-delivery: повторная выборка пуста.
    let b2 = with_timeout(MuqClient::connect(host_addr)).await.unwrap();
    let resp2 = with_timeout(b2.subscribe(recv_id, &recv_sk)).await.unwrap();
    assert!(resp2.items.is_empty(), "после выдачи — drop-on-delivery");
}

#[tokio::test]
async fn durability_rs_2of3_survives_host_down() {
    // 3 queue-host + 1 proxy. B собирает из 2 хостов (третий «выключен»).
    let mut hosts_addr = Vec::new();
    let mut host_overlays: Vec<OverlayAddr> = Vec::new();
    let proxy = PostmanServer::bind("127.0.0.1:0".parse().unwrap()).unwrap();
    let proxy_addr = proxy.local_addr().unwrap();

    for i in 0..3u8 {
        let h = PostmanServer::bind("127.0.0.1:0".parse().unwrap()).unwrap();
        let a = h.local_addr().unwrap();
        let ov: OverlayAddr = [0xC0 + i; 32];
        proxy.muq().add_proxy_route(ov, a);
        hosts_addr.push(a);
        host_overlays.push(ov);
        tokio::spawn(h.run());
    }
    tokio::spawn(proxy.run());

    let recv_id = [0x71u8; 32];
    let send_id = [0x51u8; 32];
    let (q, recv_sk, send_sk) = make_queue(recv_id, send_id);

    // B регистрирует ту же логическую очередь на всех трёх хостах.
    for a in &hosts_addr {
        let reg = with_timeout(MuqClient::connect(*a)).await.unwrap();
        assert!(with_timeout(reg.register_queue(&q)).await.unwrap());
    }

    // A размазывает конверт RS(2,3): 3 осколка, по одному на хост.
    let msg_id = [0x5Au8; 16];
    let ct = b"a-long-enough-sealed-e2e-envelope-spread-across-three-hosts".to_vec();
    let shards = rs_split(&ct, 2, 3).unwrap();
    assert_eq!(shards.len(), 3);
    let a_client = with_timeout(MuqClient::connect(proxy_addr)).await.unwrap();
    for i in 0..3usize {
        let pf = build_deposit(
            send_id,
            &send_sk,
            msg_id,
            i as u8,
            3,
            shards[i].clone(),
            host_overlays[i],
        );
        assert!(with_timeout(a_client.deposit_via_proxy(&pf)).await.unwrap());
    }

    // B выбирает ТОЛЬКО с host0 и host1 (host2 «выключен» — B к нему не идёт).
    let mut recovered: Vec<Option<Vec<u8>>> = vec![None; 3];
    for a in hosts_addr.iter().take(2) {
        let b = with_timeout(MuqClient::connect(*a)).await.unwrap();
        let resp = with_timeout(b.subscribe(recv_id, &recv_sk)).await.unwrap();
        assert_eq!(resp.items.len(), 1);
        let it = &resp.items[0];
        recovered[it.shard_index as usize] = Some(it.ct.clone());
    }

    // Reed-Solomon восстанавливает из 2 из 3.
    let out = rs_reconstruct(recovered, 2, 3).unwrap();
    assert_eq!(
        &out[..ct.len()],
        &ct[..],
        "конверт восстановлен из 2 хостов"
    );
}
