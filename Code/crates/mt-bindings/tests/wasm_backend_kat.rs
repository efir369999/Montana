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
