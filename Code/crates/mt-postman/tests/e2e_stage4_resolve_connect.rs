//! Мост Этап 4 (rendezvous) ↔ Этап 3 (postman): узел находит адрес почтальона
//! ЧЕРЕЗ Mainline DHT (не из конфига), резолвит его и реально к нему коннектится.
//! Доказывает end-to-end замыкание — захардкоженного адреса нет.

use std::net::{IpAddr, SocketAddr};

use mainline::{Dht, Testnet};
use mt_overlay::muq::{derive_queue_keypairs, Queue};
use mt_postman::{MuqClient, PostmanServer};
use mt_rendezvous::dht::RvDht;
use mt_rendezvous::{
    derive_dht_seed, derive_salt, dht_pubkey, dht_signing_key, resolve_endpoint, Endpoint,
    RendezvousRecord, EP_DIRECT_V4,
};

#[test]
fn dht_resolved_address_reaches_live_postman() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter(); // quinn Endpoint::bind требует контекст реактора

    // Живой host + курьер на реальных портах (адрес известен после bind, до run).
    let host = rt
        .block_on(PostmanServer::bind("127.0.0.1:0".parse().unwrap()))
        .unwrap();
    let host_addr = host.local_addr().unwrap();
    let host_kem = host.muq().host_kem_pubkey();
    let courier = rt
        .block_on(PostmanServer::bind("127.0.0.1:0".parse().unwrap()))
        .unwrap();
    let courier_addr = courier.local_addr().unwrap();
    let host_overlay = [0xA0u8; 32];
    courier.muq().add_proxy_route(host_overlay, host_addr);

    // DHT: лист публикует запись, endpoint = физический адрес курьера.
    let testnet = Testnet::new(10).unwrap();
    let rv_pub = RvDht::from_dht(
        Dht::builder()
            .bootstrap(&testnet.bootstrap)
            .build()
            .unwrap(),
    );
    let rv_get = RvDht::from_dht(
        Dht::builder()
            .bootstrap(&testnet.bootstrap)
            .build()
            .unwrap(),
    );
    let dht_seed = derive_dht_seed(&[0x33u8; 32]);
    let dk = dht_pubkey(&dht_signing_key(&dht_seed));
    let salt = derive_salt(&[0x44u8; 32], 7);
    let ip4 = match courier_addr.ip() {
        IpAddr::V4(v) => v.octets(),
        _ => unreachable!("bind 127.0.0.1 = v4"),
    };
    let mut addr = ip4.to_vec();
    addr.extend_from_slice(&courier_addr.port().to_be_bytes());
    let rec = RendezvousRecord {
        overlay_addr: host_overlay,
        endpoints: vec![Endpoint {
            kind: EP_DIRECT_V4,
            addr,
        }],
        pq_hint: [0u8; 32],
        seq: 1,
        valid_until: 9_999,
    };
    rv_pub.put(&dht_seed, &salt, 1, &rec).unwrap();

    rt.block_on(async move {
        tokio::spawn(host.run());
        tokio::spawn(courier.run());

        // Адрес приходит ИЗ DHT (blocking get вне реактора), затем резолв.
        let got = tokio::task::spawn_blocking(move || rv_get.get(&dk, &salt, 0))
            .await
            .unwrap()
            .expect("DHT-запись найдена");
        let resolved: SocketAddr = resolve_endpoint(&got.endpoints[0]).expect("v4 резолв");
        assert_eq!(resolved, courier_addr, "адрес из DHT == живой курьер");

        // Реальный QUIC-коннект к DHT-резолвнутому адресу + регистрация через курьер.
        let ((recv_pk, _), (send_pk, _)) = derive_queue_keypairs(&[0x42u8; 32], 0).unwrap();
        let q = Queue {
            recv_id: [0x71u8; 32],
            send_id: [0x51u8; 32],
            recv_pubkey: *recv_pk.as_bytes(),
            send_pubkey: Some(*send_pk.as_bytes()),
            rotation_epoch: 1000,
            quota: 64,
        };
        let cli = MuqClient::connect(resolved).await.unwrap();
        assert!(
            cli.register_via_courier(host_overlay, &host_kem, &q)
                .await
                .unwrap(),
            "регистрация прошла через DHT-резолвнутый курьер"
        );
    });
}
