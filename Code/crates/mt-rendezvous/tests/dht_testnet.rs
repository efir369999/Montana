//! Running-тест Этапа 4: узел A кладёт рандеву-запись в DHT, узел B находит её тем же
//! ключом — БЕЗ захардкоженного адреса. Локальная Testnet-DHT (mainline) вместо реальной
//! BitTorrent-сети. Доказывает: снятие захардкоженного адреса почтальона работает.

use mainline::{Dht, Testnet};
use mt_rendezvous::dht::RvDht;
use mt_rendezvous::{
    derive_dht_seed, derive_salt, dht_pubkey, dht_signing_key, Endpoint, RendezvousRecord,
    EP_RELAY_CIRCUIT,
};

fn sample_record() -> RendezvousRecord {
    RendezvousRecord {
        overlay_addr: [0xAB; 32],
        endpoints: vec![Endpoint {
            kind: EP_RELAY_CIRCUIT,
            addr: vec![0x7F, 0x00, 0x00, 0x01, 0x21, 0x00], // 127.0.0.1:8448 пример
        }],
        pq_hint: [0xCD; 32],
        seq: 1,
        valid_until: 9_999_999,
    }
}

#[test]
fn two_nodes_find_each_other_via_dht_no_hardcoded_addr() {
    // Локальная DHT из 10 узлов (bootstrap для A и B).
    let testnet = Testnet::new(10).expect("testnet");

    let dht_a = Dht::builder()
        .bootstrap(&testnet.bootstrap)
        .build()
        .expect("dht A");
    let dht_b = Dht::builder()
        .bootstrap(&testnet.bootstrap)
        .build()
        .expect("dht B");
    let rv_a = RvDht::from_dht(dht_a);
    let rv_b = RvDht::from_dht(dht_b);

    // A выводит dht_key из своего master_seed, кладёт запись «я тут».
    let master = [0x42u8; 64];
    let dht_seed = derive_dht_seed(&master);
    let dk = dht_pubkey(&dht_signing_key(&dht_seed));
    let salt = derive_salt(&[0x33; 32], 7); // общий session_id пары
    let record = sample_record();

    rv_a.put(&dht_seed, &salt, record.seq, &record)
        .expect("A put в DHT");

    // B знает dk (из E2E-сессии) + тот же salt → находит запись через DHT.
    let got = rv_b.get(&dk, &salt, 0).expect("B нашёл запись через DHT");
    assert_eq!(
        got, record,
        "запись найдена byte-exact, без захардкоженного адреса"
    );
    assert_eq!(got.endpoints[0].kind, EP_RELAY_CIRCUIT);
}

#[test]
fn wrong_salt_finds_nothing() {
    let testnet = Testnet::new(10).expect("testnet");
    let dht_a = Dht::builder()
        .bootstrap(&testnet.bootstrap)
        .build()
        .unwrap();
    let dht_b = Dht::builder()
        .bootstrap(&testnet.bootstrap)
        .build()
        .unwrap();
    let rv_a = RvDht::from_dht(dht_a);
    let rv_b = RvDht::from_dht(dht_b);

    let dht_seed = derive_dht_seed(&[0x42u8; 64]);
    let dk = dht_pubkey(&dht_signing_key(&dht_seed));
    let salt = derive_salt(&[0x33; 32], 7);
    rv_a.put(&dht_seed, &salt, 1, &sample_record()).unwrap();

    // другой salt (другая эпоха) → чужой target → ничего
    let other_salt = derive_salt(&[0x33; 32], 8);
    assert!(rv_b.get(&dk, &other_salt, 0).is_none());
}

#[test]
fn presigned_batch_republished_by_relay_without_secret() {
    // Лист предподписывает пачку своим dht_key (offline), почтальон (БЕЗ секрета листа)
    // пере-put'ит запись в DHT; читатель находит её байт-в-байт. Секрет не покидал лист.
    use mt_rendezvous::dht::{prepare_batch, RvDht};
    use mt_rendezvous::{
        derive_dht_seed, derive_salt, dht_pubkey, dht_signing_key, Endpoint, RendezvousRecord,
        EP_DIRECT_V4,
    };

    let testnet = Testnet::new(10).unwrap();
    let dht_relay = Dht::builder()
        .bootstrap(&testnet.bootstrap)
        .build()
        .unwrap();
    let dht_reader = Dht::builder()
        .bootstrap(&testnet.bootstrap)
        .build()
        .unwrap();
    let rv_relay = RvDht::from_dht(dht_relay); // почтальон: секрета листа НЕ имеет
    let rv_reader = RvDht::from_dht(dht_reader);

    // Лист: секрет dht_key + предподпись пачки.
    let master = [0x33u8; 32];
    let dht_seed = derive_dht_seed(&master);
    let dk = dht_pubkey(&dht_signing_key(&dht_seed));
    let session_id = [0x44u8; 32];
    let salt = derive_salt(&session_id, 7);
    let rec = RendezvousRecord {
        overlay_addr: [0xAB; 32],
        endpoints: vec![Endpoint {
            kind: EP_DIRECT_V4,
            addr: vec![203, 0, 113, 5, 0x20, 0xFC], // 203.0.113.5:8444
        }],
        pq_hint: [0xCD; 32],
        seq: 0,
        valid_until: 9_999,
    };
    let batch = prepare_batch(&dht_seed, &salt, 1, vec![rec]).unwrap();
    assert_eq!(batch.len(), 1);
    assert_eq!(batch[0].seq, 1);
    assert_eq!(batch[0].dk, dk);

    // Почтальон пере-put'ит предподписанную запись.
    rv_relay.put_presigned(&batch[0]).unwrap();

    // Читатель находит и резолвит физический адрес ИЗ DHT-записи (не из конфига).
    let got = rv_reader.get(&dk, &salt, 0).expect("запись найдена");
    assert_eq!(got.seq, 1);
    assert_eq!(got.overlay_addr, [0xAB; 32]);
    let sock = mt_rendezvous::resolve_endpoint(&got.endpoints[0]).expect("v4 резолв");
    assert_eq!(sock.to_string(), "203.0.113.5:8444");
}

#[test]
fn expired_record_filtered_by_valid_until() {
    // F-5: запись с valid_until в прошлом относительно now → get возвращает None
    // (стена свежести §593 R2 — мёртвый лист не резолвится).
    use mt_rendezvous::dht::RvDht;
    use mt_rendezvous::{
        derive_dht_seed, derive_salt, dht_pubkey, dht_signing_key, Endpoint, RendezvousRecord,
        EP_DIRECT_V4,
    };

    let testnet = Testnet::new(10).unwrap();
    let rv_a = RvDht::from_dht(
        Dht::builder()
            .bootstrap(&testnet.bootstrap)
            .build()
            .unwrap(),
    );
    let rv_b = RvDht::from_dht(
        Dht::builder()
            .bootstrap(&testnet.bootstrap)
            .build()
            .unwrap(),
    );

    let dht_seed = derive_dht_seed(&[0x55u8; 32]);
    let dk = dht_pubkey(&dht_signing_key(&dht_seed));
    let salt = derive_salt(&[0x66u8; 32], 3);
    let rec = RendezvousRecord {
        overlay_addr: [0x77; 32],
        endpoints: vec![Endpoint {
            kind: EP_DIRECT_V4,
            addr: vec![203, 0, 113, 9, 0x20, 0xFC],
        }],
        pq_hint: [0x88; 32],
        seq: 1,
        valid_until: 100, // окно валидности до unix=100
    };
    rv_a.put(&dht_seed, &salt, 1, &rec).unwrap();

    // now=50 (внутри окна) → находит
    assert!(
        rv_b.get(&dk, &salt, 50).is_some(),
        "внутри valid_until — найдена"
    );
    // now=200 (после valid_until) → отброшена
    assert!(
        rv_b.get(&dk, &salt, 200).is_none(),
        "истёкшая — не резолвится (R2)"
    );
}
