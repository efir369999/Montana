//! WASM-bindings — для веба (junona/index.html).
//! Активна только при сборке с `--features wasm` на target wasm32-unknown-unknown.

#![cfg(target_arch = "wasm32")]
#![cfg(feature = "wasm")]

use wasm_bindgen::prelude::*;

use mt_crypto::{
    keypair_from_seed, sign as mldsa_sign, verify as mldsa_verify, PublicKey, SecretKey, Signature,
};
use mt_mnemonic::{mldsa_seed_for_role, mnemonic_to_master_seed};
use mt_state::derive_account_id;

const MT_SUITE_MLDSA65: u16 = 0x0001;

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

/// 24 слова → (pubkey 1952, seckey 4032, account_id 32). Возвращает Vec длиной 1952+4032+32=6016.
#[wasm_bindgen]
pub fn account_from_mnemonic(mnemonic: &str) -> Result<Vec<u8>, JsValue> {
    let master = mnemonic_to_master_seed(mnemonic)
        .map_err(|e| JsValue::from_str(&format!("mnemonic: {e:?}")))?;
    let acc_seed = mldsa_seed_for_role(&master, mt_codec::domain::ACCOUNT_KEY);
    let (pk, sk) =
        keypair_from_seed(&acc_seed).map_err(|e| JsValue::from_str(&format!("keygen: {e:?}")))?;
    let account_id = derive_account_id(MT_SUITE_MLDSA65, pk.as_bytes());

    let mut out = Vec::with_capacity(
        super::MT_MLDSA_PUBKEY_SIZE + super::MT_MLDSA_SECKEY_SIZE + super::MT_ACCOUNT_ID_LEN,
    );
    out.extend_from_slice(pk.as_bytes());
    out.extend_from_slice(sk.as_bytes());
    out.extend_from_slice(&account_id);
    Ok(out)
}

#[wasm_bindgen]
pub fn sign(seckey: &[u8], msg: &[u8]) -> Result<Vec<u8>, JsValue> {
    let sk = SecretKey::from_slice(seckey).ok_or_else(|| JsValue::from_str("bad seckey"))?;
    let sig = mldsa_sign(&sk, msg).map_err(|e| JsValue::from_str(&format!("{e:?}")))?;
    Ok(sig.as_bytes().to_vec())
}

#[wasm_bindgen]
pub fn verify(pubkey: &[u8], msg: &[u8], sig: &[u8]) -> Result<bool, JsValue> {
    let pk = PublicKey::from_slice(pubkey).ok_or_else(|| JsValue::from_str("bad pubkey"))?;
    let s = Signature::from_slice(sig).ok_or_else(|| JsValue::from_str("bad sig"))?;
    Ok(mldsa_verify(&pk, msg, &s))
}
