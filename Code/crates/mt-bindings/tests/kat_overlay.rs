//! KAT (Known-Answer Test) оверлей-адресации — байт-точный референс P2P-сети.
//! iOS/Android/Web реализации должны выдавать идентичные значения на том же account_id.
//! Спека: Montana P2P Network, Этап 1 «Оверлей-адрес (byte-exact)»:
//!   overlay_addr = SHA-256("mt-overlay" || 0x00 || account_id)   // 32 B

use mt_overlay::overlay_addr;

fn acc(hex_str: &str) -> [u8; 32] {
    hex::decode(hex_str).unwrap().try_into().unwrap()
}

#[test]
fn kat_overlay_addr_typical() {
    // Тот же account_id, что в KAT слепых меток мессенджера (labels) — симметрия референсов.
    let a = acc("9f199584ed120b987b617ba5bff829e176f23e5465dd70cfac5c141dfb131a21");
    assert_eq!(
        hex::encode(overlay_addr(&a)),
        "f828b971b76ebfd581601a45e5e835cddaf65555301886ec268a25c867efde7b"
    );
}

#[test]
fn kat_overlay_addr_edges() {
    assert_eq!(
        hex::encode(overlay_addr(&[0u8; 32])),
        "916c930e0299e7c20796b0a316be1b5f8c86f687ec23a8da320d387d30cdd020"
    );
    assert_eq!(
        hex::encode(overlay_addr(&[0xFFu8; 32])),
        "b0a2fa23b175e8abd67a4defc904b86219dfdff287e9be18b5ea9b0880954b87"
    );
}

#[test]
fn kat_overlay_frame_encode() {
    // Спека Этап 1 «Формат OverlayFrame»: version 01, type 01 (RELAY),
    // dst 0xBB×32, src 0xAA×32, msg_id 0x11×16, payload "sealed-e2e-envelope".
    use mt_codec::CanonicalEncode;
    let f = mt_overlay::frame::OverlayFrame {
        frame_type: mt_overlay::frame::FrameType::Relay,
        dst_overlay: [0xBB; 32],
        src_overlay: [0xAA; 32],
        msg_id: [0x11; 16],
        payload: b"sealed-e2e-envelope".to_vec(),
    };
    let mut buf = Vec::new();
    f.encode(&mut buf);
    assert_eq!(buf.len(), 105);
    assert_eq!(
        hex::encode(mt_crypto::sha256_raw(&buf)),
        "28decb645927d952e7a044739e3f01f2f969b7187c449aba2ec1eb7e2153a49f"
    );
    assert_eq!(mt_overlay::frame::OverlayFrame::decode(&buf).unwrap(), f);
}

#[test]
fn kat_challenge_message_composition() {
    // Спека «Общий примитив»: msg = "mt-reg" || 0x00 || resource || nonce || channel_hash.
    let msg = mt_overlay::challenge::challenge_message(
        mt_codec::domain::OVERLAY_REG,
        &[0xAA; 32],
        &[0x01; 16],
        &[0x02; 32],
    );
    assert_eq!(
        hex::encode(&msg),
        concat!(
            "6d742d72656700",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "01010101010101010101010101010101",
            "0202020202020202020202020202020202020202020202020202020202020202"
        )
    );
}

#[test]
fn kat_hostdeposit_encode() {
    // Спека Этап 2 (MUQ) HostDeposit: send_id32‖msg_id16‖ttl(u32)‖idx‖total‖nonce16‖ct_len(u32)‖ct‖sig_len(u32)‖sig.
    let hd = mt_overlay::muq::HostDeposit {
        send_id: [0xAA; 32],
        msg_id: [0xBB; 16],
        ttl_windows: 240,
        shard_index: 1,
        shard_total: 4,
        nonce: [0x07; 16],
        ct: vec![0xCC; 32],
        sig: Vec::new(),
    };
    let b = hd.to_bytes();
    assert_eq!(b.len(), 110);
    assert_eq!(
        hex::encode(mt_crypto::sha256_raw(&b)),
        "a90a82744c5840bab7edcaa64b2ba9615ca88036f1175d94d90fec6de4c08f4b"
    );
    assert_eq!(mt_overlay::muq::HostDeposit::decode(&b).unwrap(), hd);
}

#[test]
fn kat_queue_subscribe_composition() {
    // Спека Этап 2 (MUQ): подпись выборки над "mt-queue-sub"‖0x00‖recv_id‖nonce‖channel_hash.
    let msg = mt_overlay::challenge::challenge_message(
        mt_codec::domain::QUEUE_SUB,
        &[0xAA; 32],
        &[0x01; 16],
        &[0x02; 32],
    );
    assert_eq!(
        hex::encode(&msg),
        concat!(
            "6d742d71756575652d73756200",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "01010101010101010101010101010101",
            "0202020202020202020202020202020202020202020202020202020202020202"
        )
    );
}
