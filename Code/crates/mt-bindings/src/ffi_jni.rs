//! JNI surface для Android Kotlin.
//!
//! Реэкспортирует те же функции что ffi_c.rs, но через JNI ABI с правильными
//! Java_<pkg>_<class>_<method> именами. Класс на стороне Kotlin: `quest.montana.app.MtBindings`.
//!
//! Сборка: `cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -p mt-bindings build --release`.
//! Артефакт: `libmt_bindings.so` (3 ABI) → `Android/MontanaApp/app/src/main/jniLibs/{abi}/`.

#![cfg(target_os = "android")]

use jni::objects::{JByteArray, JClass, JString};
use jni::sys::{jbyteArray, jint};
use jni::JNIEnv;

use mt_codec::domain;
use mt_crypto::{
    keypair_from_seed, sign as mldsa_sign, verify as mldsa_verify, PublicKey, SecretKey, Signature,
};
use mt_mnemonic::{entropy_to_mnemonic, mldsa_seed_for_role, mnemonic_to_master_seed};
use mt_state::derive_account_id;

const MT_SUITE_MLDSA65: u16 = 0x0001;

/// Соответствует error codes из ffi_c.rs (отрицательные).
/// На Kotlin стороне трактуются как throw IllegalArgumentException/IllegalStateException.

#[no_mangle]
pub extern "system" fn Java_quest_montana_app_MtBindings_nativeAbiVersion(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    super::ABI_VERSION as jint
}

/// 24 слова → 64-байт master_seed. Возвращает byte[64] или null если ошибка.
#[no_mangle]
pub extern "system" fn Java_quest_montana_app_MtBindings_nativeMnemonicToMasterSeed<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    mnemonic_jstr: JString<'local>,
) -> jbyteArray {
    let null = std::ptr::null_mut();
    let mnemonic: String = match env.get_string(&mnemonic_jstr) {
        Ok(s) => s.into(),
        Err(_) => return null,
    };
    let seed = match mnemonic_to_master_seed(&mnemonic) {
        Ok(s) => s,
        Err(_) => return null,
    };
    match env.byte_array_from_slice(&seed) {
        Ok(arr) => arr.into_raw(),
        Err(_) => null,
    }
}

/// 32 байта entropy → 24-словная UTF-8 мнемоника.
#[no_mangle]
pub extern "system" fn Java_quest_montana_app_MtBindings_nativeEntropyToMnemonic<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    entropy: JByteArray<'local>,
) -> jni::sys::jstring {
    let null = std::ptr::null_mut();
    let bytes = match env.convert_byte_array(entropy) {
        Ok(b) if b.len() == 32 => b,
        _ => return null,
    };
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    let mnemonic = entropy_to_mnemonic(&arr);
    match env.new_string(mnemonic) {
        Ok(s) => s.into_raw(),
        Err(_) => null,
    }
}

/// 24 слова → ML-DSA-65 account: byte[1952 + 4032 + 32] = pk||sk||account_id.
#[no_mangle]
pub extern "system" fn Java_quest_montana_app_MtBindings_nativeAccountFromMnemonic<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    mnemonic_jstr: JString<'local>,
) -> jbyteArray {
    let null = std::ptr::null_mut();
    let mnemonic: String = match env.get_string(&mnemonic_jstr) {
        Ok(s) => s.into(),
        Err(_) => return null,
    };
    let master = match mnemonic_to_master_seed(&mnemonic) {
        Ok(s) => s,
        Err(_) => return null,
    };
    let acc_seed = mldsa_seed_for_role(&master, domain::ACCOUNT_KEY);
    let (pk, sk) = match keypair_from_seed(&acc_seed) {
        Ok(kp) => kp,
        Err(_) => return null,
    };
    let account_id = derive_account_id(MT_SUITE_MLDSA65, pk.as_bytes());

    let mut buf = Vec::with_capacity(
        super::MT_MLDSA_PUBKEY_SIZE + super::MT_MLDSA_SECKEY_SIZE + super::MT_ACCOUNT_ID_LEN,
    );
    buf.extend_from_slice(pk.as_bytes());
    buf.extend_from_slice(sk.as_bytes());
    buf.extend_from_slice(&account_id);

    match env.byte_array_from_slice(&buf) {
        Ok(arr) => arr.into_raw(),
        Err(_) => null,
    }
}

/// ML-DSA-65 sign(seckey[4032], msg) → signature[3309] (или null при ошибке).
#[no_mangle]
pub extern "system" fn Java_quest_montana_app_MtBindings_nativeSign<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    seckey: JByteArray<'local>,
    msg: JByteArray<'local>,
) -> jbyteArray {
    let null = std::ptr::null_mut();
    let sk_bytes = match env.convert_byte_array(seckey) {
        Ok(b) => b,
        Err(_) => return null,
    };
    let m = match env.convert_byte_array(msg) {
        Ok(b) => b,
        Err(_) => return null,
    };
    let sk = match SecretKey::from_slice(&sk_bytes) {
        Some(k) => k,
        None => return null,
    };
    let sig = match mldsa_sign(&sk, &m) {
        Ok(s) => s,
        Err(_) => return null,
    };
    match env.byte_array_from_slice(sig.as_bytes()) {
        Ok(arr) => arr.into_raw(),
        Err(_) => null,
    }
}

/// ML-DSA-65 verify. Возвращает 1 (valid) / 0 (invalid) / -1 (input error).
#[no_mangle]
pub extern "system" fn Java_quest_montana_app_MtBindings_nativeVerify<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    pubkey: JByteArray<'local>,
    msg: JByteArray<'local>,
    sig: JByteArray<'local>,
) -> jint {
    let pk_bytes = match env.convert_byte_array(pubkey) {
        Ok(b) => b,
        Err(_) => return -1,
    };
    let m = match env.convert_byte_array(msg) {
        Ok(b) => b,
        Err(_) => return -1,
    };
    let sig_bytes = match env.convert_byte_array(sig) {
        Ok(b) => b,
        Err(_) => return -1,
    };
    let pk = match PublicKey::from_slice(&pk_bytes) {
        Some(k) => k,
        None => return -1,
    };
    let signature = match Signature::from_slice(&sig_bytes) {
        Some(s) => s,
        None => return -1,
    };
    if mldsa_verify(&pk, &m, &signature) {
        1
    } else {
        0
    }
}
