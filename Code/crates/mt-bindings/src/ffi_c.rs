//! C-ABI surface — общая для iOS staticlib и Android cdylib.
//!
//! Все функции возвращают i32 status code. Output буферы — caller-supplied,
//! фиксированной длины, документированной в `mt_bindings.h`.

use core::slice;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

use mt_crypto::{
    keypair_from_seed, sign as mldsa_sign, verify as mldsa_verify, PublicKey, SecretKey, Signature,
};
use mt_mnemonic::{mldsa_seed_for_role, mnemonic_to_master_seed};
use mt_state::derive_account_id;
use zeroize::Zeroizing;

use super::*;

/// Канонический suite_id для ML-DSA-65 account keypair (spec §Suite registry).
pub const MT_SUITE_MLDSA65: u16 = 0x0001;

#[no_mangle]
pub extern "C" fn mt_abi_version() -> u32 {
    ABI_VERSION
}

#[no_mangle]
pub unsafe extern "C" fn mt_mnemonic_to_master_seed(
    mnemonic_utf8: *const c_char,
    out_master_seed: *mut u8,
) -> c_int {
    guard(|| {
        if mnemonic_utf8.is_null() || out_master_seed.is_null() {
            return MT_ERR_NULL_PTR;
        }
        let cs = match CStr::from_ptr(mnemonic_utf8).to_str() {
            Ok(s) => s,
            Err(_) => return MT_ERR_INVALID_UTF8,
        };
        match mnemonic_to_master_seed(cs) {
            Ok(seed) => {
                slice::from_raw_parts_mut(out_master_seed, MT_MASTER_SEED_LEN)
                    .copy_from_slice(&seed[..]);
                MT_OK
            },
            Err(e) => match e {
                mt_mnemonic::MnemonicError::WordCount(_) => MT_ERR_MNEMONIC_WORD_COUNT,
                mt_mnemonic::MnemonicError::UnknownWord(_) => MT_ERR_MNEMONIC_UNKNOWN_WORD,
                mt_mnemonic::MnemonicError::ChecksumMismatch => MT_ERR_MNEMONIC_CHECKSUM,
            },
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn mt_mldsa_seed_for_role(
    master_seed: *const u8,
    role: *const u8,
    role_len: usize,
    out_seed: *mut u8,
) -> c_int {
    guard(|| {
        if master_seed.is_null() || role.is_null() || out_seed.is_null() {
            return MT_ERR_NULL_PTR;
        }
        let mut master_arr = Zeroizing::new([0u8; MT_MASTER_SEED_LEN]);
        master_arr.copy_from_slice(slice::from_raw_parts(master_seed, MT_MASTER_SEED_LEN));
        let role_bytes = slice::from_raw_parts(role, role_len);
        let seed = Zeroizing::new(mldsa_seed_for_role(&master_arr, role_bytes));
        slice::from_raw_parts_mut(out_seed, MT_MLDSA_SEED_LEN).copy_from_slice(&seed[..]);
        MT_OK
    })
}

#[no_mangle]
pub unsafe extern "C" fn mt_mldsa_keypair_from_seed(
    seed: *const u8,
    out_pubkey: *mut u8,
    out_seckey: *mut u8,
) -> c_int {
    guard(|| {
        if seed.is_null() || out_pubkey.is_null() || out_seckey.is_null() {
            return MT_ERR_NULL_PTR;
        }
        let mut arr = Zeroizing::new([0u8; MT_MLDSA_SEED_LEN]);
        arr.copy_from_slice(slice::from_raw_parts(seed, MT_MLDSA_SEED_LEN));
        match keypair_from_seed(&arr) {
            Ok((pk, sk)) => {
                slice::from_raw_parts_mut(out_pubkey, MT_MLDSA_PUBKEY_SIZE)
                    .copy_from_slice(pk.as_bytes());
                slice::from_raw_parts_mut(out_seckey, MT_MLDSA_SECKEY_SIZE)
                    .copy_from_slice(sk.as_bytes());
                MT_OK
            },
            Err(_) => MT_ERR_KEYGEN_FAILED,
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn mt_derive_account_id(
    suite_id: u16,
    pubkey: *const u8,
    out_account_id: *mut u8,
) -> c_int {
    guard(|| {
        if pubkey.is_null() || out_account_id.is_null() {
            return MT_ERR_NULL_PTR;
        }
        let mut arr = [0u8; MT_MLDSA_PUBKEY_SIZE];
        arr.copy_from_slice(slice::from_raw_parts(pubkey, MT_MLDSA_PUBKEY_SIZE));
        let id = derive_account_id(suite_id, &arr);
        slice::from_raw_parts_mut(out_account_id, MT_ACCOUNT_ID_LEN).copy_from_slice(&id);
        MT_OK
    })
}

/// 24-словная мнемоника → ML-DSA-65 account keypair + canonical account_id (suite 0x0001).
#[no_mangle]
pub unsafe extern "C" fn mt_account_from_mnemonic(
    mnemonic_utf8: *const c_char,
    out_pubkey: *mut u8,
    out_seckey: *mut u8,
    out_account_id: *mut u8,
) -> c_int {
    guard(|| {
        let mut master = Zeroizing::new([0u8; MT_MASTER_SEED_LEN]);
        let rc = mt_mnemonic_to_master_seed(mnemonic_utf8, master.as_mut_ptr());
        if rc != MT_OK {
            return rc;
        }
        let mut acc_seed = Zeroizing::new([0u8; MT_MLDSA_SEED_LEN]);
        let rc = mt_mldsa_seed_for_role(
            master.as_ptr(),
            mt_codec::domain::ACCOUNT_KEY.as_ptr(),
            mt_codec::domain::ACCOUNT_KEY.len(),
            acc_seed.as_mut_ptr(),
        );
        if rc != MT_OK {
            return rc;
        }
        let rc = mt_mldsa_keypair_from_seed(acc_seed.as_ptr(), out_pubkey, out_seckey);
        if rc != MT_OK {
            return rc;
        }
        mt_derive_account_id(MT_SUITE_MLDSA65, out_pubkey, out_account_id)
    })
}

/// account_id (32 байта) → текстовый адрес "mt…" (Base58Check), записывает в out + NUL.
#[no_mangle]
pub unsafe extern "C" fn mt_account_id_to_address(
    account_id: *const u8,
    out: *mut u8,
    out_capacity: usize,
    out_len: *mut usize,
) -> c_int {
    guard(|| {
        if account_id.is_null() || out.is_null() || out_len.is_null() {
            return MT_ERR_NULL_PTR;
        }
        let mut id = [0u8; MT_ACCOUNT_ID_LEN];
        id.copy_from_slice(slice::from_raw_parts(account_id, MT_ACCOUNT_ID_LEN));
        let addr = account_id_to_address(&id);
        let bytes = addr.as_bytes();
        if bytes.len() + 1 > out_capacity {
            return MT_ERR_BUFFER_TOO_SMALL;
        }
        let dst = slice::from_raw_parts_mut(out, out_capacity);
        dst[..bytes.len()].copy_from_slice(bytes);
        dst[bytes.len()] = 0;
        *out_len = bytes.len();
        MT_OK
    })
}

/// Текстовый адрес "mt…" → account_id (32 байта). Проверяет контрольную сумму.
#[no_mangle]
pub unsafe extern "C" fn mt_address_to_account_id(
    address_utf8: *const c_char,
    out_account_id: *mut u8,
) -> c_int {
    guard(|| {
        if address_utf8.is_null() || out_account_id.is_null() {
            return MT_ERR_NULL_PTR;
        }
        let s = match CStr::from_ptr(address_utf8).to_str() {
            Ok(s) => s,
            Err(_) => return MT_ERR_INVALID_UTF8,
        };
        match address_to_account_id(s) {
            Some(id) => {
                slice::from_raw_parts_mut(out_account_id, MT_ACCOUNT_ID_LEN).copy_from_slice(&id);
                MT_OK
            }
            None => MT_ERR_ADDRESS_INVALID,
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn mt_sign(
    seckey: *const u8,
    msg: *const u8,
    msg_len: usize,
    out_sig: *mut u8,
) -> c_int {
    guard(|| {
        if seckey.is_null() || out_sig.is_null() || (msg.is_null() && msg_len > 0) {
            return MT_ERR_NULL_PTR;
        }
        let sk_bytes = slice::from_raw_parts(seckey, MT_MLDSA_SECKEY_SIZE);
        let sk = match SecretKey::from_slice(sk_bytes) {
            Some(k) => k,
            None => return MT_ERR_SIGN_FAILED,
        };
        let m: &[u8] = if msg_len == 0 {
            &[]
        } else {
            slice::from_raw_parts(msg, msg_len)
        };
        match mldsa_sign(&sk, m) {
            Ok(sig) => {
                slice::from_raw_parts_mut(out_sig, MT_MLDSA_SIG_SIZE)
                    .copy_from_slice(sig.as_bytes());
                MT_OK
            },
            Err(_) => MT_ERR_SIGN_FAILED,
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn mt_verify(
    pubkey: *const u8,
    msg: *const u8,
    msg_len: usize,
    sig: *const u8,
) -> c_int {
    guard(|| {
        if pubkey.is_null() || sig.is_null() || (msg.is_null() && msg_len > 0) {
            return MT_ERR_NULL_PTR;
        }
        let pk_bytes = slice::from_raw_parts(pubkey, MT_MLDSA_PUBKEY_SIZE);
        let pk = match PublicKey::from_slice(pk_bytes) {
            Some(k) => k,
            None => return MT_ERR_VERIFY_FAILED,
        };
        let sig_bytes = slice::from_raw_parts(sig, MT_MLDSA_SIG_SIZE);
        let signature = match Signature::from_slice(sig_bytes) {
            Some(s) => s,
            None => return MT_ERR_VERIFY_FAILED,
        };
        let m: &[u8] = if msg_len == 0 {
            &[]
        } else {
            slice::from_raw_parts(msg, msg_len)
        };
        if mldsa_verify(&pk, m, &signature) {
            MT_OK
        } else {
            MT_ERR_VERIFY_FAILED
        }
    })
}

/// 32 байта энтропии → 24-словная мнемоника UTF-8.
///
/// `out_mnemonic_utf8` — буфер ≥ out_capacity байт; функция запишет нуль-терминированную строку.
/// `out_len` — фактически записанные байты (без терминатора). При недостатке буфера вернёт MT_ERR_BUFFER_TOO_SMALL.
#[no_mangle]
pub unsafe extern "C" fn mt_entropy_to_mnemonic(
    entropy: *const u8,
    out_mnemonic_utf8: *mut u8,
    out_capacity: usize,
    out_len: *mut usize,
) -> c_int {
    guard(|| {
        if entropy.is_null() || out_mnemonic_utf8.is_null() || out_len.is_null() {
            return MT_ERR_NULL_PTR;
        }
        let ent_slice = slice::from_raw_parts(entropy, 32);
        let mut ent = [0u8; 32];
        ent.copy_from_slice(ent_slice);
        let mnemonic = mt_mnemonic::entropy_to_mnemonic(&ent);
        let bytes = mnemonic.as_bytes();
        if bytes.len() + 1 > out_capacity {
            return MT_ERR_BUFFER_TOO_SMALL;
        }
        let dst = slice::from_raw_parts_mut(out_mnemonic_utf8, out_capacity);
        dst[..bytes.len()].copy_from_slice(bytes);
        dst[bytes.len()] = 0; // null terminator
        *out_len = bytes.len();
        MT_OK
    })
}
