//! e2e Этапа 5: deep-link montana:// — bootstrap-приглашение целиком и резолв личности
//! (mt-адрес) до оверлей-адреса через боевые крейты (address_to_account_id, overlay_addr).

use mt_bindings::{account_id_to_address, address_to_account_id};
use mt_bootstrap::{parse_deep_link, DeepLink, QRBootstrap, QR_KIND_DIRECT_V6};
use mt_overlay::overlay_addr;

#[test]
fn bootstrap_deep_link_full_journey() {
    // Друг делает ссылку → новый телефон парсит → достаёт живой эндпоинт.
    let mut ep = vec![0u8; 16];
    ep[15] = 1;
    ep.extend_from_slice(&8444u16.to_be_bytes());
    let q = QRBootstrap {
        dk: [0x5A; 32],
        expires: 2_000_000,
        ep_kind: QR_KIND_DIRECT_V6,
        ep,
    };
    let link = q.to_deep_link();
    assert!(link.starts_with("montana://b/"));
    match parse_deep_link(&link).unwrap() {
        DeepLink::Bootstrap(parsed) => {
            assert_eq!(parsed, q);
            assert_eq!(
                parsed.current_endpoint(1_000_000).unwrap().to_string(),
                "[::1]:8444"
            );
        },
        _ => panic!("ожидался Bootstrap"),
    }
}

#[test]
fn mt_address_deep_link_resolves_identity_to_overlay() {
    // montana://<mt-address> → account_id → overlay_addr, всё boevыми крейтами byte-exact.
    let account_id = [0x7Au8; 32];
    let address = account_id_to_address(&account_id); // "mt..." Bitcoin-подобный
    assert!(address.starts_with("mt"));
    let link = format!("montana://{address}");
    match parse_deep_link(&link).unwrap() {
        DeepLink::Address(a) => {
            assert_eq!(a, address);
            // личность восстанавливается из адреса (checksum сходится)
            let resolved = address_to_account_id(&a).expect("валидный mt-адрес");
            assert_eq!(resolved, account_id, "личность = адрес кошелька");
            // account_id → overlay-адрес получателя (детерминирован, Этап 1)
            let ov = overlay_addr(&resolved);
            assert_eq!(
                ov,
                overlay_addr(&account_id),
                "оверлей-адрес byte-exact от личности"
            );
        },
        _ => panic!("ожидался Address"),
    }
}
