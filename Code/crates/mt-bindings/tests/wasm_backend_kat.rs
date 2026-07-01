//! Cross-backend KAT: pure-Rust ml-dsa (веб/wasm-бэкенд) и нативный OpenSSL
//! (mt-crypto) обязаны выдавать байт-идентичные ключи, account_id и подписи.
//! Ловит любое расхождение бэкендов на сборке.
#![cfg(not(target_arch = "wasm32"))]

use ml_dsa::{
    EncodedVerifyingKey, Keypair, MlDsa65, Signature as RcSig, Signer, SigningKey, Verifier,
    VerifyingKey, B32,
};
use mt_mnemonic::{entropy_to_mnemonic, mldsa_seed_for_role, mnemonic_to_master_seed};
use sha2::{Digest, Sha256};

fn zero_acc_seed() -> [u8; 32] {
    let mnemonic = entropy_to_mnemonic(&[0u8; 32]);
    let master = mnemonic_to_master_seed(&mnemonic).unwrap();
    mldsa_seed_for_role(&master, mt_codec::domain::ACCOUNT_KEY)
}

#[test]
fn ml_dsa_wasm_backend_matches_openssl_kat() {
    let acc_seed = zero_acc_seed();
    let seed = B32::try_from(&acc_seed[..]).unwrap();
    let sk = SigningKey::<MlDsa65>::from_seed(&seed);
    let pk = sk.verifying_key().encode();
    let mut h = Sha256::new();
    h.update(mt_codec::domain::ACCOUNT);
    h.update([0u8]);
    h.update([0x01u8, 0x00u8]);
    h.update(pk.as_slice());
    let id = hex::encode(h.finalize());
    assert_eq!(
        id,
        "9f199584ed120b987b617ba5bff829e176f23e5465dd70cfac5c141dfb131a21"
    );
}

/// Кросс-бэкенд: OpenSSL (mt-crypto) ↔ pure-Rust (ml-dsa) на одном ξ:
/// (1) публичные ключи байт-идентичны; (2) подписи байт-идентичны
/// (ML-DSA-65 детерминирован, пустой контекст); (3) подпись каждого бэкенда
/// верифицируется другим.
#[test]
fn cross_backend_sign_verify() {
    let acc_seed = zero_acc_seed();
    let msg = b"montana cross-backend kat";

    // OpenSSL backend
    let (pk_o, sk_o) = mt_crypto::keypair_from_seed(&acc_seed).unwrap();
    let sig_o = mt_crypto::sign(&sk_o, msg).unwrap();

    // pure-Rust backend
    let seed = B32::try_from(&acc_seed[..]).unwrap();
    let sk_r = SigningKey::<MlDsa65>::from_seed(&seed);
    let pk_r = sk_r.verifying_key();
    let sig_r: RcSig<MlDsa65> = sk_r.sign(msg);
    let sig_r_bytes = sig_r.encode();

    // (1) pubkeys identical
    assert_eq!(&pk_o.as_bytes()[..], pk_r.encode().as_slice());
    // (2) deterministic signatures identical
    assert_eq!(&sig_o.as_bytes()[..], sig_r_bytes.as_slice());

    // (3a) ml-dsa signature verifies under OpenSSL
    let sig_r_as_o = mt_crypto::Signature::from_slice(sig_r_bytes.as_slice()).unwrap();
    assert!(mt_crypto::verify(&pk_o, msg, &sig_r_as_o));

    // (3b) OpenSSL signature verifies under ml-dsa
    let enc = EncodedVerifyingKey::<MlDsa65>::try_from(&pk_o.as_bytes()[..]).unwrap();
    let vk_r = VerifyingKey::<MlDsa65>::decode(&enc);
    let sig_o_as_r = RcSig::<MlDsa65>::try_from(sig_o.as_bytes().as_slice()).unwrap();
    assert!(vk_r.verify(msg, &sig_o_as_r).is_ok());
}

/// Кросс-бэкенд ML-KEM + подпись identity-KEM: app_kem_pub из сида байт-идентичен
/// (OpenSSL vs ml-kem), app_kem_sig (ML-DSA над "mt-idkem"||0x00||app_kem_pub)
/// валидна. Печатает SHA-256-отпечатки для binding-векторов Этапа 3.
#[test]
fn cross_backend_app_kem_and_idkem_sig() {
    use ml_kem::{EncodedSizeUser, KemCore, MlKem768, B32 as KemB32};

    let mnemonic = entropy_to_mnemonic(&[0u8; 32]);
    let master = mnemonic_to_master_seed(&mnemonic).unwrap();
    let app_kem_seed =
        mt_mnemonic::mlkem_seed_for_role(&master, mt_codec::domain::APP_ENCRYPTION_KEY);

    // OpenSSL ML-KEM
    let (pk_o, _sk_o) = mt_crypto::keypair_from_seed_mlkem(&app_kem_seed).unwrap();
    // pure-Rust ML-KEM (d = first 32, z = last 32)
    let d = KemB32::try_from(&app_kem_seed[..32]).unwrap();
    let z = KemB32::try_from(&app_kem_seed[32..]).unwrap();
    let (_dk, ek) = MlKem768::generate_deterministic(&d, &z);
    // (1) cross-backend app_kem_pub identical
    assert_eq!(&pk_o.as_bytes()[..], ek.as_bytes().as_slice());

    let app_kem_pub = pk_o.as_bytes();

    // app_kem_sig = ML-DSA sign(account_key, "mt-idkem" || 0x00 || app_kem_pub)
    let acc_seed = zero_acc_seed();
    let (pk_acc, sk_acc) = mt_crypto::keypair_from_seed(&acc_seed).unwrap();
    let mut msg = Vec::with_capacity(8 + 1 + 1184);
    msg.extend_from_slice(b"mt-idkem");
    msg.push(0u8);
    msg.extend_from_slice(app_kem_pub);
    let sig = mt_crypto::sign(&sk_acc, &msg).unwrap();
    // (2) app_kem_sig verifies under account_key
    assert!(mt_crypto::verify(&pk_acc, &msg, &sig));

    // baked binding-векторы Этапа 3 (нулевая мнемоника)
    assert_eq!(
        hex::encode(Sha256::digest(app_kem_pub)),
        "b827d37b2b225907c835f25a8652c215af69f8f52bd6a7ef0ae31955d63fd1c4"
    );
    assert_eq!(
        hex::encode(Sha256::digest(sig.as_bytes())),
        "316e908176df3d7e17b5a4cec8d0292ab2f0bdeefa3f51da3eb2bf57df80d595"
    );
}

// ---- Этап 4: PQXDH (чистый ML-KEM-768) ----

fn hmac_sha256(key: &[u8], msg: &[u8]) -> [u8; 32] {
    let mut k = [0u8; 64];
    if key.len() > 64 {
        k[..32].copy_from_slice(&Sha256::digest(key));
    } else {
        k[..key.len()].copy_from_slice(key);
    }
    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5cu8; 64];
    for i in 0..64 {
        ipad[i] ^= k[i];
        opad[i] ^= k[i];
    }
    let mut hi = Sha256::new();
    hi.update(ipad);
    hi.update(msg);
    let inner = hi.finalize();
    let mut ho = Sha256::new();
    ho.update(opad);
    ho.update(inner);
    ho.finalize().into()
}

fn hkdf_sha256(salt: &[u8], ikm: &[u8], info: &[u8], l: usize) -> Vec<u8> {
    let prk = hmac_sha256(salt, ikm);
    let mut okm = Vec::new();
    let mut t: Vec<u8> = Vec::new();
    let mut i = 1u8;
    while okm.len() < l {
        let mut m = t.clone();
        m.extend_from_slice(info);
        m.push(i);
        t = hmac_sha256(&prk, &m).to_vec();
        okm.extend_from_slice(&t);
        i += 1;
    }
    okm.truncate(l);
    okm
}

/// Детерминированное ключевое расписание PQXDH (Этап 4, Шаг 3). Фиксированные
/// секреты + transcript_hash → запечённые initial_root_key / chain (с одноразовым
/// и без). Чистый HKDF-SHA-256 → кросс-платформенно идентично.
#[test]
fn pqxdh_key_schedule_kat() {
    let ss_id = [0x11u8; 32];
    let ss_spk = [0x22u8; 32];
    let ss_opk = [0x33u8; 32];
    let transcript_hash = [0xAAu8; 32];
    let salt = [0u8; 32];
    let mut info = b"mt-pqxdh-root".to_vec();
    info.push(0u8);
    info.extend_from_slice(&transcript_hash);

    // с одноразовым: IKM = ss_id || ss_spk || ss_opk
    let mut ikm = Vec::new();
    ikm.extend_from_slice(&ss_id);
    ikm.extend_from_slice(&ss_spk);
    ikm.extend_from_slice(&ss_opk);
    let okm = hkdf_sha256(&salt, &ikm, &info, 64);
    assert_eq!(
        hex::encode(&okm[..32]),
        "d1d0a8699658a49099eddf5eafa58cf9da1d8ff02ce00f7218245b3bee0efcd1"
    );
    assert_eq!(
        hex::encode(&okm[32..]),
        "082046319cc79abbfa129a7699607dd55fe989ca9f1822ab5af53692788a27b2"
    );

    // без одноразового: IKM = ss_id || ss_spk
    let mut ikm2 = Vec::new();
    ikm2.extend_from_slice(&ss_id);
    ikm2.extend_from_slice(&ss_spk);
    let okm2 = hkdf_sha256(&salt, &ikm2, &info, 64);
    assert_eq!(
        hex::encode(&okm2[..32]),
        "38fa29cc640c4a87e554ece7cb1168bf3d18bd0e4b6ee5683336091c433ca4ca"
    );
    assert_eq!(
        hex::encode(&okm2[32..]),
        "6697d2bb86b5306ff82a86e9213655328bde8b3056226f5d3b1c89b769a76098"
    );
}

/// Согласие сторон PQXDH (Этап 4). Алиса инкапсулирует к трём реальным ML-KEM
/// ключам Боба (OpenSSL), Боб декапсулирует — общие секреты и выведенный корень
/// совпадают байт-в-байт. Проверяет весь поток установления сессии.
#[test]
fn pqxdh_agreement() {
    // ключи Боба
    let mnemonic = entropy_to_mnemonic(&[0u8; 32]);
    let master = mnemonic_to_master_seed(&mnemonic).unwrap();
    let app_seed = mt_mnemonic::mlkem_seed_for_role(&master, mt_codec::domain::APP_ENCRYPTION_KEY);
    let (app_pk, app_sk) = mt_crypto::keypair_from_seed_mlkem(&app_seed).unwrap();
    let (spk_pk, spk_sk) = mt_crypto::keypair_from_seed_mlkem(&[0x55u8; 64]).unwrap();
    let (opk_pk, opk_sk) = mt_crypto::keypair_from_seed_mlkem(&[0x66u8; 64]).unwrap();

    // Алиса инкапсулирует
    let (ct_id, ss_id_a) = mt_crypto::mlkem_encapsulate(&app_pk).unwrap();
    let (ct_spk, ss_spk_a) = mt_crypto::mlkem_encapsulate(&spk_pk).unwrap();
    let (ct_opk, ss_opk_a) = mt_crypto::mlkem_encapsulate(&opk_pk).unwrap();

    // Боб декапсулирует
    let ss_id_b = mt_crypto::mlkem_decapsulate(&app_sk, &ct_id).unwrap();
    let ss_spk_b = mt_crypto::mlkem_decapsulate(&spk_sk, &ct_spk).unwrap();
    let ss_opk_b = mt_crypto::mlkem_decapsulate(&opk_sk, &ct_opk).unwrap();

    assert_eq!(ss_id_a.as_bytes(), ss_id_b.as_bytes());
    assert_eq!(ss_spk_a.as_bytes(), ss_spk_b.as_bytes());
    assert_eq!(ss_opk_a.as_bytes(), ss_opk_b.as_bytes());

    // обе стороны: одно и то же ключевое расписание → равный корень
    let salt = [0u8; 32];
    let mut info = b"mt-pqxdh-root".to_vec();
    info.push(0u8);
    info.extend_from_slice(&[0xCCu8; 32]); // фиксированный transcript_hash (одинаков у обеих сторон)

    let ikm = |a: &[u8; 32], b: &[u8; 32], c: &[u8; 32]| {
        let mut v = Vec::new();
        v.extend_from_slice(a);
        v.extend_from_slice(b);
        v.extend_from_slice(c);
        v
    };
    let root_a = hkdf_sha256(
        &salt,
        &ikm(ss_id_a.as_bytes(), ss_spk_a.as_bytes(), ss_opk_a.as_bytes()),
        &info,
        64,
    );
    let root_b = hkdf_sha256(
        &salt,
        &ikm(ss_id_b.as_bytes(), ss_spk_b.as_bytes(), ss_opk_b.as_bytes()),
        &info,
        64,
    );
    assert_eq!(root_a, root_b);
    assert_eq!(root_a.len(), 64);
}
