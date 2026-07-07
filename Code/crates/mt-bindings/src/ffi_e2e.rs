//! C-ABI движка E2E (mt-messenger-e2e) для iOS. Переменная длина выходов —
//! owned-буфер: функция аллоцирует, отдаёт (ptr,len); клиент освобождает mt_e2e_free.

use core::slice;
use std::os::raw::c_int;

use mt_messenger_e2e::session::SessionState;

use super::{guard, MT_ERR_KEM_FAILED, MT_ERR_NULL_PTR, MT_OK};

unsafe fn write_out(data: Vec<u8>, out_ptr: *mut *mut u8, out_len: *mut usize) {
    let mut boxed = data.into_boxed_slice();
    *out_len = boxed.len();
    *out_ptr = boxed.as_mut_ptr();
    std::mem::forget(boxed);
}

/// Освободить буфер, выданный функциями mt_e2e_*.
///
/// # Safety
/// `ptr`/`len` — ровно то, что вернула mt_e2e_* через out-параметры; вызывать однократно.
#[no_mangle]
pub unsafe extern "C" fn mt_e2e_free(ptr: *mut u8, len: usize) {
    if !ptr.is_null() && len > 0 {
        drop(Vec::from_raw_parts(ptr, len, len));
    }
}

/// RatchetEncrypt через непрозрачный блоб сессии. Возвращает новый блоб сессии + сообщение.
///
/// # Safety
/// Все указатели валидны на свою длину; `rng_seed` — 64 байта; out-указатели ненулевые.
#[no_mangle]
pub unsafe extern "C" fn mt_e2e_encrypt(
    session: *const u8,
    session_len: usize,
    pt: *const u8,
    pt_len: usize,
    rng_seed: *const u8,
    out_session: *mut *mut u8,
    out_session_len: *mut usize,
    out_msg: *mut *mut u8,
    out_msg_len: *mut usize,
) -> c_int {
    guard(|| {
        if session.is_null()
            || rng_seed.is_null()
            || out_session.is_null()
            || out_msg.is_null()
            || (pt_len > 0 && pt.is_null())
        {
            return MT_ERR_NULL_PTR;
        }
        let mut st = match SessionState::from_bytes(slice::from_raw_parts(session, session_len)) {
            Ok(s) => s,
            Err(_) => return MT_ERR_KEM_FAILED,
        };
        let plaintext = if pt_len == 0 {
            &[][..]
        } else {
            slice::from_raw_parts(pt, pt_len)
        };
        let seed: [u8; 64] = slice::from_raw_parts(rng_seed, 64).try_into().unwrap();
        let msg = match st.encrypt(plaintext, &seed) {
            Ok(m) => m,
            Err(_) => return MT_ERR_KEM_FAILED,
        };
        write_out(st.to_bytes(), out_session, out_session_len);
        write_out(msg, out_msg, out_msg_len);
        MT_OK
    })
}

/// RatchetDecrypt через непрозрачный блоб сессии. Возвращает новый блоб + открытый текст.
///
/// # Safety
/// Все указатели валидны на свою длину; out-указатели ненулевые.
#[no_mangle]
pub unsafe extern "C" fn mt_e2e_decrypt(
    session: *const u8,
    session_len: usize,
    msg: *const u8,
    msg_len: usize,
    out_session: *mut *mut u8,
    out_session_len: *mut usize,
    out_pt: *mut *mut u8,
    out_pt_len: *mut usize,
) -> c_int {
    guard(|| {
        if session.is_null() || msg.is_null() || out_session.is_null() || out_pt.is_null() {
            return MT_ERR_NULL_PTR;
        }
        let mut st = match SessionState::from_bytes(slice::from_raw_parts(session, session_len)) {
            Ok(s) => s,
            Err(_) => return MT_ERR_KEM_FAILED,
        };
        let message = slice::from_raw_parts(msg, msg_len);
        let pt = match st.decrypt(message) {
            Ok(p) => p,
            Err(_) => return MT_ERR_KEM_FAILED,
        };
        write_out(st.to_bytes(), out_session, out_session_len);
        write_out(pt, out_pt, out_pt_len);
        MT_OK
    })
}

use mt_messenger_e2e::handshake::{
    build_handshake, process_handshake, RecipientBundle, RecipientKeys,
};
// DSSOT-3: размеры — из авторитетных констант крейта (SSOT), не magic numbers.
const MLDSA_PUB: usize = crate::MT_MLDSA_PUBKEY_SIZE;
const MLKEM_PUB: usize = crate::MT_MLKEM_PUBKEY_SIZE;
const MLKEM_SK: usize = crate::MT_MLKEM_SECKEY_SIZE;

/// Сторона Алисы: рукопожатие + инициализация сессии. Возвращает InitialHandshake
/// + блоб сессии инициатора. `account_seed` — 32 байта (сид ML-DSA личности).
///
/// # Safety
/// Все ключевые указатели валидны на размеры спеки; opk_* читаются лишь при opk_flag=1.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn mt_e2e_build_handshake(
    alice_account_pub: *const u8,
    account_seed: *const u8,
    bob_account_pub: *const u8,
    bob_app_kem_pub: *const u8,
    bob_spk_pub: *const u8,
    spk_id: u32,
    opk_flag: u8,
    opk_id: u32,
    bob_opk_pub: *const u8,
    eph_seed: *const u8,
    send_time: u64,
    out_hs: *mut *mut u8,
    out_hs_len: *mut usize,
    out_session: *mut *mut u8,
    out_session_len: *mut usize,
) -> c_int {
    guard(|| {
        if alice_account_pub.is_null()
            || account_seed.is_null()
            || eph_seed.is_null()
            || bob_account_pub.is_null()
            || bob_app_kem_pub.is_null()
            || bob_spk_pub.is_null()
            || out_hs.is_null()
            || out_hs_len.is_null()
            || out_session.is_null()
            || out_session_len.is_null()
            || (opk_flag == 1 && bob_opk_pub.is_null())
        {
            return MT_ERR_NULL_PTR;
        }
        let a_pub: [u8; MLDSA_PUB] = slice::from_raw_parts(alice_account_pub, MLDSA_PUB)
            .try_into()
            .unwrap();
        let seed: [u8; 32] = slice::from_raw_parts(account_seed, 32).try_into().unwrap();
        let b_pub: [u8; MLDSA_PUB] = slice::from_raw_parts(bob_account_pub, MLDSA_PUB)
            .try_into()
            .unwrap();
        let app_pub: [u8; MLKEM_PUB] = slice::from_raw_parts(bob_app_kem_pub, MLKEM_PUB)
            .try_into()
            .unwrap();
        let spk_pub: [u8; MLKEM_PUB] = slice::from_raw_parts(bob_spk_pub, MLKEM_PUB)
            .try_into()
            .unwrap();
        let opk_pub: Option<[u8; MLKEM_PUB]> = if opk_flag == 1 {
            Some(
                slice::from_raw_parts(bob_opk_pub, MLKEM_PUB)
                    .try_into()
                    .unwrap(),
            )
        } else {
            None
        };
        let eph: [u8; 64] = slice::from_raw_parts(eph_seed, 64).try_into().unwrap();

        let bundle = RecipientBundle {
            account_key_pub: &b_pub,
            app_kem_pub: &app_pub,
            signed_prekey_pub: &spk_pub,
            spk_id,
            one_time: opk_pub.as_ref().map(|p| (opk_id, p)),
        };
        let hs = match build_handshake(&a_pub, &seed, &bundle, &eph, send_time) {
            Ok(h) => h,
            Err(_) => return MT_ERR_KEM_FAILED,
        };
        let session = SessionState::init_initiator(
            hs.transcript_hash,
            hs.session.root_key,
            hs.session.sending_chain_key,
            hs.eph_kem_pub_a,
            hs.eph_kem_sk_a,
            hs.signed_prekey_pub_b,
        );
        write_out(hs.bytes, out_hs, out_hs_len);
        write_out(session.to_bytes(), out_session, out_session_len);
        MT_OK
    })
}

/// Сторона Боба: обработка рукопожатия + инициализация сессии получателя.
///
/// # Safety
/// Все ключевые указатели валидны на размеры спеки; opk_* читаются лишь при opk_flag=1.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn mt_e2e_process_handshake(
    hs: *const u8,
    hs_len: usize,
    bob_account_id: *const u8,
    bob_app_kem_pub: *const u8,
    bob_app_kem_sk: *const u8,
    bob_spk_pub: *const u8,
    bob_spk_sk: *const u8,
    opk_flag: u8,
    bob_opk_pub: *const u8,
    bob_opk_sk: *const u8,
    now: u64,
    accept_skew: u64,
    out_session: *mut *mut u8,
    out_session_len: *mut usize,
) -> c_int {
    guard(|| {
        if hs.is_null()
            || bob_account_id.is_null()
            || bob_app_kem_pub.is_null()
            || bob_app_kem_sk.is_null()
            || bob_spk_pub.is_null()
            || bob_spk_sk.is_null()
            || out_session.is_null()
            || out_session_len.is_null()
            || (opk_flag == 1 && (bob_opk_pub.is_null() || bob_opk_sk.is_null()))
        {
            return crate::MT_ERR_NULL_PTR;
        }
        let hs_bytes = slice::from_raw_parts(hs, hs_len);
        let acc_id: [u8; 32] = slice::from_raw_parts(bob_account_id, 32)
            .try_into()
            .unwrap();
        let app_pub: [u8; MLKEM_PUB] = slice::from_raw_parts(bob_app_kem_pub, MLKEM_PUB)
            .try_into()
            .unwrap();
        let spk_pub: [u8; MLKEM_PUB] = slice::from_raw_parts(bob_spk_pub, MLKEM_PUB)
            .try_into()
            .unwrap();
        let app_sk = slice::from_raw_parts(bob_app_kem_sk, MLKEM_SK);
        let spk_sk = slice::from_raw_parts(bob_spk_sk, MLKEM_SK);
        let opk: Option<([u8; MLKEM_PUB], &[u8])> = if opk_flag == 1 {
            let pk: [u8; MLKEM_PUB] = slice::from_raw_parts(bob_opk_pub, MLKEM_PUB)
                .try_into()
                .unwrap();
            Some((pk, slice::from_raw_parts(bob_opk_sk, MLKEM_SK)))
        } else {
            None
        };

        let keys = RecipientKeys {
            account_id: &acc_id,
            app_kem_pub: &app_pub,
            app_kem_sk: app_sk,
            signed_prekey_pub: &spk_pub,
            signed_prekey_sk: spk_sk,
            one_time: opk.as_ref().map(|(p, s)| (p, *s)),
        };
        let proc = match process_handshake(hs_bytes, &keys, now, accept_skew) {
            Ok(p) => p,
            Err(_) => return MT_ERR_KEM_FAILED,
        };
        let session = SessionState::init_responder(
            proc.transcript_hash,
            proc.session.root_key,
            proc.session.sending_chain_key,
            proc.eph_kem_pub_a,
            spk_pub,
            spk_sk.to_vec(),
        );
        write_out(session.to_bytes(), out_session, out_session_len);
        MT_OK
    })
}
