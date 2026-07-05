//! C-ABI движка E2E: полный поток шифрование/расшифровка через FFI (iOS-поверхность).
//! Сессии настраиваются Rust-API (handshake), затем encrypt/decrypt идут ЧЕРЕЗ C ABI.

use mt_bindings::ffi_e2e::{mt_e2e_decrypt, mt_e2e_encrypt, mt_e2e_free};
use mt_crypto::{keypair_from_seed, keypair_from_seed_mlkem};
use mt_messenger_e2e::handshake::{
    account_id, build_handshake, process_handshake, RecipientBundle, RecipientKeys, MLDSA_PUBKEY,
    MLKEM_PUBKEY,
};
use mt_messenger_e2e::session::SessionState;

fn setup_blobs() -> (Vec<u8>, Vec<u8>) {
    let (app_pk, app_sk) = keypair_from_seed_mlkem(&[0x11; 64]).unwrap();
    let (spk_pk, spk_sk) = keypair_from_seed_mlkem(&[0x22; 64]).unwrap();
    let app_arr: [u8; MLKEM_PUBKEY] = app_pk.as_bytes().to_owned();
    let spk_arr: [u8; MLKEM_PUBKEY] = spk_pk.as_bytes().to_owned();
    let (bob_acc_pub, _) = keypair_from_seed(&[0x44; 32]).unwrap();
    let bob_pub: [u8; MLDSA_PUBKEY] = bob_acc_pub.as_bytes().to_owned();
    let bob_id = account_id(&bob_pub);
    let (alice_acc_pub, alice_sk) = keypair_from_seed(&[0x55; 32]).unwrap();
    let alice_pub: [u8; MLDSA_PUBKEY] = alice_acc_pub.as_bytes().to_owned();

    let bundle = RecipientBundle {
        account_key_pub: &bob_pub,
        app_kem_pub: &app_pk,
        signed_prekey_pub: &spk_pk,
        spk_id: 7,
        one_time: None,
    };
    let hs = build_handshake(&alice_pub, &alice_sk, &bundle, &[0x66; 64], 1000).unwrap();
    let keys = RecipientKeys {
        account_id: &bob_id,
        app_kem_pub: &app_arr,
        app_kem_sk: &app_sk,
        signed_prekey_pub: &spk_arr,
        signed_prekey_sk: &spk_sk,
        one_time: None,
    };
    let proc = process_handshake(&hs.bytes, &keys, 1001, 604800).unwrap();

    let alice = SessionState::init_initiator(
        hs.transcript_hash,
        hs.session.root_key,
        hs.session.sending_chain_key,
        hs.eph_kem_pub_a,
        hs.eph_kem_sk_a,
        hs.signed_prekey_pub_b,
    );
    let (_p2, spk_sk2) = keypair_from_seed_mlkem(&[0x22; 64]).unwrap();
    let bob = SessionState::init_responder(
        proc.transcript_hash,
        proc.session.root_key,
        proc.session.sending_chain_key,
        proc.eph_kem_pub_a,
        spk_arr,
        spk_sk2,
    );
    (alice.to_bytes(), bob.to_bytes())
}

unsafe fn ffi_encrypt(session: &[u8], pt: &[u8], seed: &[u8; 64]) -> (Vec<u8>, Vec<u8>) {
    let (mut os, mut osl, mut om, mut oml) =
        (std::ptr::null_mut(), 0usize, std::ptr::null_mut(), 0usize);
    let rc = mt_e2e_encrypt(
        session.as_ptr(),
        session.len(),
        pt.as_ptr(),
        pt.len(),
        seed.as_ptr(),
        &mut os,
        &mut osl,
        &mut om,
        &mut oml,
    );
    assert_eq!(rc, 0);
    let new_session = std::slice::from_raw_parts(os, osl).to_vec();
    let msg = std::slice::from_raw_parts(om, oml).to_vec();
    mt_e2e_free(os, osl);
    mt_e2e_free(om, oml);
    (new_session, msg)
}

unsafe fn ffi_decrypt(session: &[u8], msg: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let (mut os, mut osl, mut op, mut opl) =
        (std::ptr::null_mut(), 0usize, std::ptr::null_mut(), 0usize);
    let rc = mt_e2e_decrypt(
        session.as_ptr(),
        session.len(),
        msg.as_ptr(),
        msg.len(),
        &mut os,
        &mut osl,
        &mut op,
        &mut opl,
    );
    assert_eq!(rc, 0);
    let new_session = std::slice::from_raw_parts(os, osl).to_vec();
    let pt = std::slice::from_raw_parts(op, opl).to_vec();
    mt_e2e_free(os, osl);
    mt_e2e_free(op, opl);
    (new_session, pt)
}

#[test]
fn ffi_encrypt_decrypt_flow() {
    let (alice0, bob0) = setup_blobs();
    unsafe {
        // Алиса -> Боб через C ABI
        let (alice1, m1) = ffi_encrypt(&alice0, b"privet cherez FFI", &[0xA1; 64]);
        let (bob1, pt1) = ffi_decrypt(&bob0, &m1);
        assert_eq!(pt1, b"privet cherez FFI");

        // Боб -> Алиса через C ABI (KEM-шаг)
        let (_bob2, r1) = ffi_encrypt(&bob1, b"otvet cherez FFI", &[0xB1; 64]);
        let (_alice2, pt2) = ffi_decrypt(&alice1, &r1);
        assert_eq!(pt2, b"otvet cherez FFI");
    }
}

use mt_bindings::ffi_e2e::{mt_e2e_build_handshake, mt_e2e_process_handshake};

#[allow(clippy::too_many_arguments)]
unsafe fn ffi_build(
    a_pub: &[u8],
    a_sk: &[u8],
    b_pub: &[u8],
    b_app: &[u8],
    b_spk: &[u8],
    spk_id: u32,
    eph: &[u8; 64],
    t: u64,
) -> (Vec<u8>, Vec<u8>) {
    let (mut oh, mut ohl, mut os, mut osl) =
        (std::ptr::null_mut(), 0usize, std::ptr::null_mut(), 0usize);
    let rc = mt_e2e_build_handshake(
        a_pub.as_ptr(),
        a_sk.as_ptr(),
        b_pub.as_ptr(),
        b_app.as_ptr(),
        b_spk.as_ptr(),
        spk_id,
        0,
        0,
        std::ptr::null(),
        eph.as_ptr(),
        t,
        &mut oh,
        &mut ohl,
        &mut os,
        &mut osl,
    );
    assert_eq!(rc, 0);
    let hs = std::slice::from_raw_parts(oh, ohl).to_vec();
    let session = std::slice::from_raw_parts(os, osl).to_vec();
    mt_e2e_free(oh, ohl);
    mt_e2e_free(os, osl);
    (hs, session)
}

#[allow(clippy::too_many_arguments)]
unsafe fn ffi_process(
    hs: &[u8],
    b_id: &[u8],
    b_app_pub: &[u8],
    b_app_sk: &[u8],
    b_spk_pub: &[u8],
    b_spk_sk: &[u8],
    now: u64,
    skew: u64,
) -> Vec<u8> {
    let (mut os, mut osl) = (std::ptr::null_mut(), 0usize);
    let rc = mt_e2e_process_handshake(
        hs.as_ptr(),
        hs.len(),
        b_id.as_ptr(),
        b_app_pub.as_ptr(),
        b_app_sk.as_ptr(),
        b_spk_pub.as_ptr(),
        b_spk_sk.as_ptr(),
        0,
        std::ptr::null(),
        std::ptr::null(),
        now,
        skew,
        &mut os,
        &mut osl,
    );
    assert_eq!(rc, 0);
    let session = std::slice::from_raw_parts(os, osl).to_vec();
    mt_e2e_free(os, osl);
    session
}

#[test]
fn ffi_full_handshake_and_ratchet() {
    // Ключи полностью через публичный крипто-слой (как у клиента).
    let (app_pk, app_sk) = keypair_from_seed_mlkem(&[0x11; 64]).unwrap();
    let (spk_pk, spk_sk) = keypair_from_seed_mlkem(&[0x22; 64]).unwrap();
    let (bob_acc_pub, _) = keypair_from_seed(&[0x44; 32]).unwrap();
    let (alice_acc_pub, alice_acc_sk) = keypair_from_seed(&[0x55; 32]).unwrap();
    let bob_id = account_id(&bob_acc_pub.as_bytes().to_owned());

    unsafe {
        let (hs, alice0) = ffi_build(
            alice_acc_pub.as_bytes(),
            alice_acc_sk.as_bytes(),
            bob_acc_pub.as_bytes(),
            app_pk.as_bytes(),
            spk_pk.as_bytes(),
            7,
            &[0x66; 64],
            1000,
        );
        let bob0 = ffi_process(
            &hs,
            &bob_id,
            app_pk.as_bytes(),
            app_sk.as_bytes(),
            spk_pk.as_bytes(),
            spk_sk.as_bytes(),
            1001,
            604800,
        );
        // переписка целиком через FFI
        let (alice1, m1) = ffi_encrypt(&alice0, b"ves put cherez FFI", &[0xA1; 64]);
        let (bob1, pt1) = ffi_decrypt(&bob0, &m1);
        assert_eq!(pt1, b"ves put cherez FFI");
        let (_bob2, r1) = ffi_encrypt(&bob1, b"otvet", &[0xB1; 64]);
        let (_alice2, pt2) = ffi_decrypt(&alice1, &r1);
        assert_eq!(pt2, b"otvet");
    }
}
