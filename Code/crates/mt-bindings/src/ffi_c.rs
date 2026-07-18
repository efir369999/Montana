//! C-ABI surface — shared by the iOS staticlib and the Android cdylib.
//!
//! All functions return an i32 status code. Output buffers are caller-supplied,
//! of fixed length documented in `mt_bindings.h`.

// Safety contracts (buffer sizes, non-null pointers) are documented in mt_bindings.h.
#![allow(clippy::missing_safety_doc)]

use core::slice;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

use mt_crypto::{
    keypair_from_seed, keypair_from_seed_mlkem, mlkem_decapsulate, mlkem_encapsulate,
    sign as mldsa_sign, verify as mldsa_verify, MlkemCiphertext, MlkemPublicKey, MlkemSecretKey,
    PublicKey, SecretKey, Signature,
};
use mt_mnemonic::{
    mldsa_seed_for_role, mlkem_seed_for_role, mnemonic_to_entropy, mnemonic_to_master_seed,
};
use mt_state::derive_account_id;
use zeroize::Zeroizing;

use super::*;

/// Canonical suite_id for the ML-DSA-65 account keypair (spec §Suite registry).
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
pub unsafe extern "C" fn mt_mnemonic_to_entropy(
    mnemonic_utf8: *const c_char,
    out_entropy: *mut u8,
) -> c_int {
    guard(|| {
        if mnemonic_utf8.is_null() || out_entropy.is_null() {
            return MT_ERR_NULL_PTR;
        }
        let cs = match CStr::from_ptr(mnemonic_utf8).to_str() {
            Ok(s) => s,
            Err(_) => return MT_ERR_INVALID_UTF8,
        };
        match mnemonic_to_entropy(cs) {
            Ok(ent) => {
                slice::from_raw_parts_mut(out_entropy, 32).copy_from_slice(&ent[..]);
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

/// 24-word mnemonic → ML-DSA-65 account keypair + canonical account_id (suite 0x0001).
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

/// account_id (32 bytes) → textual address "mt…" (Base58Check), writes into out + NUL.
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

/// Textual address "mt…" → account_id (32 bytes). Verifies the checksum.
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
            },
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

/// 32 bytes of entropy → 24-word UTF-8 mnemonic.
///
/// `out_mnemonic_utf8` — a buffer ≥ out_capacity bytes; the function writes a null-terminated string.
/// `out_len` — the bytes actually written (excluding the terminator). If the buffer is too small it returns MT_ERR_BUFFER_TOO_SMALL.
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

/// HKDF-Expand(master_seed, role, 64) -> ML-KEM-768 seed (d‖z). Stage 1: app_kem_key.
#[no_mangle]
pub unsafe extern "C" fn mt_mlkem_seed_for_role(
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
        let seed = Zeroizing::new(mlkem_seed_for_role(&master_arr, role_bytes));
        slice::from_raw_parts_mut(out_seed, MT_MLKEM_SEED_LEN).copy_from_slice(&seed[..]);
        MT_OK
    })
}

/// ML-KEM-768 KeyGen from a 64-byte seed (FIPS 203, deterministic). pk 1184 / sk 2400.
#[no_mangle]
pub unsafe extern "C" fn mt_mlkem_keypair_from_seed(
    seed: *const u8,
    out_pubkey: *mut u8,
    out_seckey: *mut u8,
) -> c_int {
    guard(|| {
        if seed.is_null() || out_pubkey.is_null() || out_seckey.is_null() {
            return MT_ERR_NULL_PTR;
        }
        let mut arr = Zeroizing::new([0u8; MT_MLKEM_SEED_LEN]);
        arr.copy_from_slice(slice::from_raw_parts(seed, MT_MLKEM_SEED_LEN));
        match keypair_from_seed_mlkem(&arr) {
            Ok((pk, sk)) => {
                slice::from_raw_parts_mut(out_pubkey, MT_MLKEM_PUBKEY_SIZE)
                    .copy_from_slice(pk.as_bytes());
                slice::from_raw_parts_mut(out_seckey, MT_MLKEM_SECKEY_SIZE)
                    .copy_from_slice(sk.as_bytes());
                MT_OK
            },
            Err(_) => MT_ERR_KEYGEN_FAILED,
        }
    })
}

/// ML-KEM-768 Encapsulate (FIPS 203 §6.2). pk 1184 -> ct 1088 / ss 32.
#[no_mangle]
pub unsafe extern "C" fn mt_mlkem_encaps(
    pubkey: *const u8,
    out_ct: *mut u8,
    out_ss: *mut u8,
) -> c_int {
    guard(|| {
        if pubkey.is_null() || out_ct.is_null() || out_ss.is_null() {
            return MT_ERR_NULL_PTR;
        }
        let pk =
            match MlkemPublicKey::from_slice(slice::from_raw_parts(pubkey, MT_MLKEM_PUBKEY_SIZE)) {
                Some(k) => k,
                None => return MT_ERR_KEM_FAILED,
            };
        match mlkem_encapsulate(&pk) {
            Ok((ct, ss)) => {
                slice::from_raw_parts_mut(out_ct, MT_MLKEM_CT_SIZE).copy_from_slice(ct.as_bytes());
                slice::from_raw_parts_mut(out_ss, MT_MLKEM_SS_SIZE).copy_from_slice(ss.as_bytes());
                MT_OK
            },
            Err(_) => MT_ERR_KEM_FAILED,
        }
    })
}

/// ML-KEM-768 Decapsulate (FIPS 203 §6.3, implicit-rejection). sk 2400, ct 1088 -> ss 32.
#[no_mangle]
pub unsafe extern "C" fn mt_mlkem_decaps(
    seckey: *const u8,
    ct: *const u8,
    out_ss: *mut u8,
) -> c_int {
    guard(|| {
        if seckey.is_null() || ct.is_null() || out_ss.is_null() {
            return MT_ERR_NULL_PTR;
        }
        let sk =
            match MlkemSecretKey::from_slice(slice::from_raw_parts(seckey, MT_MLKEM_SECKEY_SIZE)) {
                Some(k) => k,
                None => return MT_ERR_KEM_FAILED,
            };
        let ctv = match MlkemCiphertext::from_slice(slice::from_raw_parts(ct, MT_MLKEM_CT_SIZE)) {
            Some(c) => c,
            None => return MT_ERR_KEM_FAILED,
        };
        match mlkem_decapsulate(&sk, &ctv) {
            Ok(ss) => {
                slice::from_raw_parts_mut(out_ss, MT_MLKEM_SS_SIZE).copy_from_slice(ss.as_bytes());
                MT_OK
            },
            Err(_) => MT_ERR_KEM_FAILED,
        }
    })
}

/// 24-word mnemonic -> app_kem_key (ML-KEM-768) via role "mt-app-encryption-key". pk 1184 / sk 2400.
#[no_mangle]
pub unsafe extern "C" fn mt_app_kem_from_mnemonic(
    mnemonic_utf8: *const c_char,
    out_pubkey: *mut u8,
    out_seckey: *mut u8,
) -> c_int {
    guard(|| {
        let mut master = Zeroizing::new([0u8; MT_MASTER_SEED_LEN]);
        let rc = mt_mnemonic_to_master_seed(mnemonic_utf8, master.as_mut_ptr());
        if rc != MT_OK {
            return rc;
        }
        let mut kem_seed = Zeroizing::new([0u8; MT_MLKEM_SEED_LEN]);
        let rc = mt_mlkem_seed_for_role(
            master.as_ptr(),
            mt_codec::domain::APP_ENCRYPTION_KEY.as_ptr(),
            mt_codec::domain::APP_ENCRYPTION_KEY.len(),
            kem_seed.as_mut_ptr(),
        );
        if rc != MT_OK {
            return rc;
        }
        mt_mlkem_keypair_from_seed(kem_seed.as_ptr(), out_pubkey, out_seckey)
    })
}

/// history_key = HKDF-SHA-256(salt=0×32, ikm=entropy_32, info="mt-history-key", 32) — messenger Stage 10.
/// `entropy` — 32 bytes; `out` — 32 bytes. SSOT for history_key across all clients.
#[no_mangle]
pub unsafe extern "C" fn mt_history_key(entropy: *const u8, out: *mut u8) -> c_int {
    guard(|| {
        if entropy.is_null() || out.is_null() {
            return MT_ERR_NULL_PTR;
        }
        let ent = slice::from_raw_parts(entropy, MT_HISTORY_KEY_LEN);
        let prk = mt_mnemonic::hmac_sha256(&[0u8; 32], ent); // HKDF-Extract(salt=0×32, ikm=entropy)
        let okm =
            mt_mnemonic::hkdf_expand(&prk, mt_codec::domain::MSG_HISTORY_KEY, MT_HISTORY_KEY_LEN);
        slice::from_raw_parts_mut(out, MT_HISTORY_KEY_LEN).copy_from_slice(&okm);
        MT_OK
    })
}

/// media_key = HKDF-SHA-256(salt=0×32, ikm=entropy_32, info="mt-media-key", 32) — s.2 Stage 1.
/// Separate seed branch for media at-rest (≠ history_key). `entropy`/`out` — 32 bytes. SSOT for all clients.
#[no_mangle]
pub unsafe extern "C" fn mt_media_key(entropy: *const u8, out: *mut u8) -> c_int {
    guard(|| {
        if entropy.is_null() || out.is_null() {
            return MT_ERR_NULL_PTR;
        }
        let ent = slice::from_raw_parts(entropy, MT_HISTORY_KEY_LEN);
        let prk = mt_mnemonic::hmac_sha256(&[0u8; 32], ent); // HKDF-Extract(salt=0×32, ikm=entropy)
        let okm =
            mt_mnemonic::hkdf_expand(&prk, mt_codec::domain::MSG_MEDIA_KEY, MT_HISTORY_KEY_LEN);
        slice::from_raw_parts_mut(out, MT_HISTORY_KEY_LEN).copy_from_slice(&okm);
        MT_OK
    })
}

// ═══ Stage 1 of the second front — local archive Montana/Chats/<chat>/ ═══

/// Append a single message to the local archive: <base>/Chats/<chat>/conversation.mtlog,
/// sealed under history_key (Rust does encode+seal+file). base = the app's Montana folder.
///
/// # Safety
/// `hk`/`account_id`/`conv_id` → 32 B; strings are valid UTF-8 C-strings; `content` → `content_len` B.
#[no_mangle]
pub unsafe extern "C" fn mt_archive_append(
    base_path: *const c_char,
    chat_name: *const c_char,
    hk: *const u8,
    account_id: *const u8,
    conv_id: *const u8,
    dir: u8,
    send_time: u64,
    content: *const u8,
    content_len: usize,
) -> c_int {
    guard(|| {
        if base_path.is_null()
            || chat_name.is_null()
            || hk.is_null()
            || account_id.is_null()
            || conv_id.is_null()
            || (content.is_null() && content_len != 0)
        {
            return crate::MT_ERR_NULL_PTR;
        }
        let base = match CStr::from_ptr(base_path).to_str() {
            Ok(s) => s,
            Err(_) => return crate::MT_ERR_INVALID_UTF8,
        };
        let chat = match CStr::from_ptr(chat_name).to_str() {
            Ok(s) => s,
            Err(_) => return crate::MT_ERR_INVALID_UTF8,
        };
        let mut hk32 = [0u8; 32];
        hk32.copy_from_slice(slice::from_raw_parts(hk, 32));
        let mut acct = [0u8; 32];
        acct.copy_from_slice(slice::from_raw_parts(account_id, 32));
        let mut conv = [0u8; 32];
        conv.copy_from_slice(slice::from_raw_parts(conv_id, 32));
        let body = if content_len == 0 {
            Vec::new()
        } else {
            slice::from_raw_parts(content, content_len).to_vec()
        };
        let store = match mt_messenger_e2e::archive::ArchiveStore::open(base) {
            Ok(s) => s,
            Err(_) => return crate::MT_ERR_IO,
        };
        // The core assigns a monotonic per-identity block_seq (seq.bin) — the client does not pass seq (no nonce reuse).
        match store.append_item(chat, &hk32, &acct, &conv, dir, send_time, &body) {
            Ok(_seq) => crate::MT_OK,
            Err(_) => crate::MT_ERR_IO,
        }
    })
}

/// Encrypt media under history_key and place it into <base>/Chats/<chat>/Media/<blob_id_hex>.
/// Other applications see only ciphertext; only the client can decrypt it using the seed.
///
/// # Safety
/// strings are valid UTF-8 C-strings; `hk`/`account_id` → 32 B; `blob` → `blob_len` B.
#[no_mangle]
pub unsafe extern "C" fn mt_archive_put_media(
    base_path: *const c_char,
    chat_name: *const c_char,
    blob_id_hex: *const c_char,
    hk: *const u8,
    account_id: *const u8,
    blob: *const u8,
    blob_len: usize,
) -> c_int {
    guard(|| {
        if base_path.is_null()
            || chat_name.is_null()
            || blob_id_hex.is_null()
            || hk.is_null()
            || account_id.is_null()
            || (blob.is_null() && blob_len != 0)
        {
            return crate::MT_ERR_NULL_PTR;
        }
        let base = match CStr::from_ptr(base_path).to_str() {
            Ok(s) => s,
            Err(_) => return crate::MT_ERR_INVALID_UTF8,
        };
        let chat = match CStr::from_ptr(chat_name).to_str() {
            Ok(s) => s,
            Err(_) => return crate::MT_ERR_INVALID_UTF8,
        };
        let bid = match CStr::from_ptr(blob_id_hex).to_str() {
            Ok(s) => s,
            Err(_) => return crate::MT_ERR_INVALID_UTF8,
        };
        let mut mk32 = [0u8; 32];
        mk32.copy_from_slice(slice::from_raw_parts(hk, 32)); // media_key
        let mut acct = [0u8; 32];
        acct.copy_from_slice(slice::from_raw_parts(account_id, 32));
        let data = if blob_len == 0 {
            Vec::new()
        } else {
            slice::from_raw_parts(blob, blob_len).to_vec()
        };
        let store = match mt_messenger_e2e::archive::ArchiveStore::open(base) {
            Ok(s) => s,
            Err(_) => return crate::MT_ERR_IO,
        };
        match store.put_media(chat, bid, &mk32, &acct, &data) {
            Ok(()) => crate::MT_OK,
            Err(_) => crate::MT_ERR_IO,
        }
    })
}

/// Read and decrypt media. Returns the plaintext length (>=0) or an error code (<0).
/// `out_cap` too small → MT_ERR_BUFFER_TOO_SMALL; file missing / decryption failed → MT_ERR_IO.
///
/// # Safety
/// strings are valid UTF-8; `hk`/`account_id` → 32 B; `out` → `out_cap` B.
#[no_mangle]
pub unsafe extern "C" fn mt_archive_get_media(
    base_path: *const c_char,
    chat_name: *const c_char,
    blob_id_hex: *const c_char,
    hk: *const u8,
    account_id: *const u8,
    out: *mut u8,
    out_cap: usize,
) -> isize {
    let r = guard(|| {
        if base_path.is_null()
            || chat_name.is_null()
            || blob_id_hex.is_null()
            || hk.is_null()
            || account_id.is_null()
            || (out.is_null() && out_cap != 0)
        {
            return crate::MT_ERR_NULL_PTR;
        }
        let base = match CStr::from_ptr(base_path).to_str() {
            Ok(s) => s,
            Err(_) => return crate::MT_ERR_INVALID_UTF8,
        };
        let chat = match CStr::from_ptr(chat_name).to_str() {
            Ok(s) => s,
            Err(_) => return crate::MT_ERR_INVALID_UTF8,
        };
        let bid = match CStr::from_ptr(blob_id_hex).to_str() {
            Ok(s) => s,
            Err(_) => return crate::MT_ERR_INVALID_UTF8,
        };
        let mut mk32 = [0u8; 32];
        mk32.copy_from_slice(slice::from_raw_parts(hk, 32)); // media_key
        let mut acct = [0u8; 32];
        acct.copy_from_slice(slice::from_raw_parts(account_id, 32));
        let store = match mt_messenger_e2e::archive::ArchiveStore::open(base) {
            Ok(s) => s,
            Err(_) => return crate::MT_ERR_IO,
        };
        match store.get_media(chat, bid, &mk32, &acct) {
            Some(pt) => {
                if pt.len() > out_cap {
                    return crate::MT_ERR_BUFFER_TOO_SMALL;
                }
                slice::from_raw_parts_mut(out, pt.len()).copy_from_slice(&pt);
                pt.len() as i32
            },
            None => crate::MT_ERR_IO,
        }
    });
    r as isize
}
