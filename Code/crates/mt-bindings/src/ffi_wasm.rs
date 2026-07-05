//! WASM-bindings — веб-клиент. Крипта на pure-Rust ml-dsa/ml-kem (OpenSSL в браузер
//! не собирается). Байт-идентичность нативу гарантируется FIPS 203/204 KAT:
//! account_id от нулевой мнемоники == 9f199584…131a21 (см. mt-bindings KAT).
#![cfg(all(target_arch = "wasm32", feature = "wasm"))]

use wasm_bindgen::prelude::*;

use ml_dsa::{
    EncodedVerifyingKey, Keypair, MlDsa65, Signature, Signer, SigningKey, Verifier, VerifyingKey,
    B32,
};
use mt_mnemonic::{mldsa_seed_for_role, mnemonic_to_master_seed};
use sha2::{Digest, Sha256};

const SUITE_MLDSA65_LE: [u8; 2] = [0x01, 0x00];

fn account_id_from_pk(pk: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(mt_codec::domain::ACCOUNT);
    h.update([0u8]);
    h.update(SUITE_MLDSA65_LE);
    h.update(pk);
    h.finalize().into()
}

#[wasm_bindgen]
pub fn abi_version() -> u32 {
    super::ABI_VERSION
}

#[wasm_bindgen]
pub fn mnemonic_to_master_seed_js(mnemonic: &str) -> Result<Vec<u8>, JsValue> {
    mnemonic_to_master_seed(mnemonic)
        .map(|s| s.to_vec())
        .map_err(|e| JsValue::from_str(&format!("{e:?}")))
}

/// 24 слова → pk(1952) ‖ acc_seed(32) ‖ account_id(32) = 2016 байт.
/// acc_seed (ξ) — локальный секрет; хранить на устройстве, подписывать через `sign`.
#[wasm_bindgen]
pub fn account_from_mnemonic(mnemonic: &str) -> Result<Vec<u8>, JsValue> {
    let master = mnemonic_to_master_seed(mnemonic)
        .map_err(|e| JsValue::from_str(&format!("mnemonic: {e:?}")))?;
    let acc_seed = mldsa_seed_for_role(&master, mt_codec::domain::ACCOUNT_KEY);
    let seed = B32::try_from(&acc_seed[..]).map_err(|_| JsValue::from_str("seed"))?;
    let sk = SigningKey::<MlDsa65>::from_seed(&seed);
    let pk = sk.verifying_key().encode();
    let account_id = account_id_from_pk(pk.as_slice());
    let mut out = Vec::with_capacity(1952 + 32 + 32);
    out.extend_from_slice(pk.as_slice());
    out.extend_from_slice(&acc_seed);
    out.extend_from_slice(&account_id);
    Ok(out)
}

/// 24 слова -> app_kem_key (ML-KEM-768) через роль "mt-app-encryption-key".
/// Возврат: pk(1184) ‖ sk(2400). Байт-идентично нативу (OpenSSL) по кросс-бэкенд KAT.
#[wasm_bindgen]
pub fn app_kem_from_mnemonic(mnemonic: &str) -> Result<Vec<u8>, JsValue> {
    use ml_kem::{EncodedSizeUser, KemCore, MlKem768, B32 as KemB32};
    let master = mnemonic_to_master_seed(mnemonic)
        .map_err(|e| JsValue::from_str(&format!("mnemonic: {e:?}")))?;
    let seed = mt_mnemonic::mlkem_seed_for_role(&master, mt_codec::domain::APP_ENCRYPTION_KEY);
    let d = KemB32::try_from(&seed[..32]).map_err(|_| JsValue::from_str("d"))?;
    let z = KemB32::try_from(&seed[32..]).map_err(|_| JsValue::from_str("z"))?;
    let (dk, ek) = MlKem768::generate_deterministic(&d, &z);
    let mut out = Vec::with_capacity(1184 + 2400);
    out.extend_from_slice(ek.as_bytes().as_slice());
    out.extend_from_slice(dk.as_bytes().as_slice());
    Ok(out)
}

/// ML-KEM-768 KeyGen из 64-байтного сида (FIPS 203, deterministic). Возврат pk(1184) ‖ sk(2400).
#[wasm_bindgen]
pub fn mlkem_keypair_from_seed(seed: &[u8]) -> Result<Vec<u8>, JsValue> {
    use ml_kem::{EncodedSizeUser, KemCore, MlKem768, B32 as KemB32};
    if seed.len() != 64 {
        return Err(JsValue::from_str("seed must be 64 bytes"));
    }
    let d = KemB32::try_from(&seed[..32]).map_err(|_| JsValue::from_str("d"))?;
    let z = KemB32::try_from(&seed[32..]).map_err(|_| JsValue::from_str("z"))?;
    let (dk, ek) = MlKem768::generate_deterministic(&d, &z);
    let mut out = Vec::with_capacity(1184 + 2400);
    out.extend_from_slice(ek.as_bytes().as_slice());
    out.extend_from_slice(dk.as_bytes().as_slice());
    Ok(out)
}

#[wasm_bindgen]
pub fn sign(acc_seed: &[u8], msg: &[u8]) -> Result<Vec<u8>, JsValue> {
    let seed =
        B32::try_from(acc_seed).map_err(|_| JsValue::from_str("acc_seed must be 32 bytes"))?;
    let sk = SigningKey::<MlDsa65>::from_seed(&seed);
    let sig: Signature<MlDsa65> = sk.sign(msg);
    Ok(sig.encode().as_slice().to_vec())
}

#[wasm_bindgen]
pub fn verify(pubkey: &[u8], msg: &[u8], sig: &[u8]) -> Result<bool, JsValue> {
    let enc = EncodedVerifyingKey::<MlDsa65>::try_from(pubkey)
        .map_err(|_| JsValue::from_str("bad pubkey"))?;
    let vk = VerifyingKey::<MlDsa65>::decode(&enc);
    let s = Signature::<MlDsa65>::try_from(sig).map_err(|_| JsValue::from_str("bad sig"))?;
    Ok(vk.verify(msg, &s).is_ok())
}

#[wasm_bindgen]
pub fn account_id_to_address(account_id: &[u8]) -> Result<String, JsValue> {
    let id: [u8; 32] = account_id
        .try_into()
        .map_err(|_| JsValue::from_str("account_id must be 32 bytes"))?;
    Ok(crate::account_id_to_address(&id))
}
