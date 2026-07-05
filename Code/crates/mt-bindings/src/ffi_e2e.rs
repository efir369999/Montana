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
        if session.is_null() || rng_seed.is_null() || out_session.is_null() || out_msg.is_null() {
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
