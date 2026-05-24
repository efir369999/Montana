//! Montana-bindings — единственный источник истины Montana для не-Rust клиентов.
//!
//! Re-exports канонических функций из mt-mnemonic / mt-crypto / mt-state / mt-account
//! через стабильный C ABI (для iOS xcframework + Android cdylib JNI) и WASM-bindings
//! (для web). Все клиенты обязаны вызывать ЭТИ функции, а не реимплементировать
//! PBKDF2/HKDF/SHA/ML-DSA/address-derivation/transaction-encoding в нативном коде.
//!
//! Spec invariant [SSOT-Rust] — см. `Formal-Docs/02-Spec/SSOT-Rust.md`.

use core::panic::AssertUnwindSafe;

#[cfg(not(target_arch = "wasm32"))]
mod ffi_c;

#[cfg(target_arch = "wasm32")]
mod ffi_wasm;

pub const ABI_VERSION: u32 = 1;

pub const MT_MASTER_SEED_LEN: usize = 64;
pub const MT_MLDSA_SEED_LEN: usize = 32;
pub const MT_MLDSA_PUBKEY_SIZE: usize = 1952;
pub const MT_MLDSA_SECKEY_SIZE: usize = 4032;
pub const MT_MLDSA_SIG_SIZE: usize = 3309;
pub const MT_ACCOUNT_ID_LEN: usize = 32;
pub const MT_MAX_MNEMONIC_BYTES: usize = 512;

pub const MT_OK: i32 = 0;
pub const MT_ERR_NULL_PTR: i32 = -1;
pub const MT_ERR_INVALID_UTF8: i32 = -2;
pub const MT_ERR_MNEMONIC_WORD_COUNT: i32 = -3;
pub const MT_ERR_MNEMONIC_UNKNOWN_WORD: i32 = -4;
pub const MT_ERR_MNEMONIC_CHECKSUM: i32 = -5;
pub const MT_ERR_KEYGEN_FAILED: i32 = -6;
pub const MT_ERR_SIGN_FAILED: i32 = -7;
pub const MT_ERR_VERIFY_FAILED: i32 = -8;
pub const MT_ERR_BUFFER_TOO_SMALL: i32 = -9;
pub const MT_ERR_PANIC: i32 = -100;

#[inline]
fn guard<F: FnOnce() -> i32>(f: F) -> i32 {
    match std::panic::catch_unwind(AssertUnwindSafe(f)) {
        Ok(code) => code,
        Err(_) => MT_ERR_PANIC,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abi_version_set() {
        assert_eq!(ABI_VERSION, 1);
    }

    #[test]
    fn constants_match_workspace() {
        assert_eq!(MT_MASTER_SEED_LEN, mt_mnemonic::MASTER_SEED_LEN);
        assert_eq!(MT_MLDSA_SEED_LEN, mt_mnemonic::MLDSA_SEED_LEN);
        assert_eq!(MT_MLDSA_PUBKEY_SIZE, mt_crypto::PUBLIC_KEY_SIZE);
        assert_eq!(MT_MLDSA_SECKEY_SIZE, mt_crypto::SECRET_KEY_SIZE);
        assert_eq!(MT_MLDSA_SIG_SIZE, mt_crypto::SIGNATURE_SIZE);
    }
}
