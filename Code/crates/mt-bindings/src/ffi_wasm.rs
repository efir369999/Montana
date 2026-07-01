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
