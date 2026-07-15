//! C-ABI рандеву + bootstrap (Montana P2P Network, Этап 6 §661): DHT-резолв (RvDht) и
//! deep-link/QR-разбор (QRBootstrap/DeepLink) — суверенный вход для Swift/Kotlin.
//! RvDht синхронен (mainline DHT), tokio не нужен. Endpoint-выдача SSRF-фильтрована
//! (resolve_endpoint_public / current_endpoint_public). Пути FFI не паникуют.

use std::ffi::CStr;
use std::net::SocketAddr;
use std::os::raw::c_char;

use mt_bootstrap::{parse_deep_link, DeepLink};
use mt_rendezvous::dht::RvDht;
use mt_rendezvous::{record_binds_account, resolve_endpoint_public, DK_LEN, SALT_LEN};

/// # Safety
/// `link` — валидный C-string (null-terminated) или null.
unsafe fn cstr(link: *const c_char) -> Option<&'static str> {
    if link.is_null() {
        return None;
    }
    CStr::from_ptr(link).to_str().ok()
}

fn write_str(bytes: &[u8], out: *mut u8, out_cap: usize) -> usize {
    if out.is_null() || bytes.len() > out_cap {
        return 0;
    }
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), out, bytes.len()) };
    bytes.len()
}

fn write_addr(addr: SocketAddr, out: *mut u8, out_cap: usize) -> usize {
    write_str(addr.to_string().as_bytes(), out, out_cap)
}

/// Тип deep-link montana://: 0 = bootstrap-payload (montana://b/...), 1 = wallet-адрес
/// (montana://mt...), -1 = ошибка разбора.
///
/// # Safety
/// `link` — валидный C-string.
#[no_mangle]
pub unsafe extern "C" fn mt_deeplink_kind(link: *const c_char) -> i32 {
    let Some(s) = cstr(link) else {
        return -1;
    };
    match parse_deep_link(s) {
        Ok(DeepLink::Bootstrap(_)) => 0,
        Ok(DeepLink::Address(_)) => 1,
        Err(_) => -1,
    }
}

/// Для montana://<mt-address>: пишет адрес кошелька (ASCII) в `out`, возвращает длину
/// (0 если не address / буфер мал / ошибка).
///
/// # Safety
/// `link` — C-string; `out` — ≥ `out_cap` байт.
#[no_mangle]
pub unsafe extern "C" fn mt_deeplink_address(
    link: *const c_char,
    out: *mut u8,
    out_cap: usize,
) -> usize {
    let Some(s) = cstr(link) else {
        return 0;
    };
    let Ok(DeepLink::Address(addr)) = parse_deep_link(s) else {
        return 0;
    };
    write_str(addr.as_bytes(), out, out_cap)
}

/// Для montana://b/<payload>: декодирует QRBootstrap, пишет current_endpoint
/// (SSRF-фильтрован, "host:port" ASCII) в `out`; возвращает длину (0 если протух /
/// внутренний адрес / не bootstrap / ошибка).
///
/// # Safety
/// `link` — C-string; `out` — ≥ `out_cap` байт.
#[no_mangle]
pub unsafe extern "C" fn mt_deeplink_bootstrap_endpoint(
    link: *const c_char,
    now_unix: u64,
    out: *mut u8,
    out_cap: usize,
) -> usize {
    let Some(s) = cstr(link) else {
        return 0;
    };
    let Ok(DeepLink::Bootstrap(qr)) = parse_deep_link(s) else {
        return 0;
    };
    let Some(addr) = qr.current_endpoint_public(now_unix) else {
        return 0;
    };
    write_addr(addr, out, out_cap)
}

/// Подключение к Mainline DHT (публичные bootstrap-ноды BitTorrent). Освобождается
/// `mt_rvdht_free`. null при ошибке.
#[no_mangle]
pub extern "C" fn mt_rvdht_client() -> *mut RvDht {
    match RvDht::client() {
        Ok(d) => Box::into_raw(Box::new(d)),
        Err(_) => std::ptr::null_mut(),
    }
}

/// # Safety
/// `dht` — указатель от `mt_rvdht_client` (не использованный после free) или null.
#[no_mangle]
pub unsafe extern "C" fn mt_rvdht_free(dht: *mut RvDht) {
    if !dht.is_null() {
        drop(Box::from_raw(dht));
    }
}

/// Резолвит рандеву-запись друга по `dk`(32)+`salt`(20) из DHT, пишет первый
/// глобально-маршрутизируемый endpoint (SSRF-фильтрован, "host:port") в `out`;
/// возвращает длину (0 если записи нет / протухла / только внутренние адреса / ошибка).
///
/// # Safety
/// `dht` валиден; `dk` — ≥32 B; `salt` — ≥20 B; `friend_account_id` — ≥32 B или null
/// (null пропускает сверку §595 — не рекомендуется); `out` — ≥ `out_cap` байт.
#[no_mangle]
pub unsafe extern "C" fn mt_rvdht_resolve(
    dht: *const RvDht,
    dk: *const u8,
    salt: *const u8,
    friend_account_id: *const u8,
    now_unix: u64,
    out: *mut u8,
    out_cap: usize,
) -> usize {
    if dht.is_null() || dk.is_null() || salt.is_null() {
        return 0;
    }
    let mut dk_a = [0u8; DK_LEN];
    std::ptr::copy_nonoverlapping(dk, dk_a.as_mut_ptr(), DK_LEN);
    let mut salt_a = [0u8; SALT_LEN];
    std::ptr::copy_nonoverlapping(salt, salt_a.as_mut_ptr(), SALT_LEN);
    let Some(record) = (*dht).get(&dk_a, &salt_a, now_unix) else {
        return 0;
    };
    // DEV-051 / §595 [P2P-5] первая линия: сверка привязки записи к личности друга
    // (account_id известен из E2E-сессии). Подделка overlay_addr → 0 (не доверяем).
    if !friend_account_id.is_null() {
        let mut acc = [0u8; DK_LEN];
        std::ptr::copy_nonoverlapping(friend_account_id, acc.as_mut_ptr(), 32);
        if !record_binds_account(&record, &acc) {
            return 0;
        }
    }
    for ep in &record.endpoints {
        if let Some(addr) = resolve_endpoint_public(ep) {
            return write_addr(addr, out, out_cap);
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use mt_bootstrap::QRBootstrap;
    use std::ffi::CString;

    #[test]
    fn ffi_deeplink_kind_bootstrap_and_address() {
        // Bootstrap: QRBootstrap -> deep-link -> kind 0
        let qr = QRBootstrap {
            dk: [0xAB; 32],
            expires: 2_000_000,
            ep_kind: 0x02,
            // global v6 2606:4700::1 ‖ port 8444
            ep: vec![
                0x26, 0x06, 0x47, 0x00, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x01, 0x20, 0xFC,
            ],
        };
        let blink = CString::new(qr.to_deep_link()).unwrap();
        assert_eq!(unsafe { mt_deeplink_kind(blink.as_ptr()) }, 0);
        // endpoint резолвится (global, не протух при now < expires)
        let mut out = [0u8; 64];
        let n = unsafe {
            mt_deeplink_bootstrap_endpoint(blink.as_ptr(), 1_000_000, out.as_mut_ptr(), out.len())
        };
        assert!(n > 0, "global endpoint резолвится");
        assert!(std::str::from_utf8(&out[..n])
            .unwrap()
            .contains("2606:4700"));

        // Address
        let alink = CString::new("montana://mt1qqqqqq").unwrap();
        assert_eq!(unsafe { mt_deeplink_kind(alink.as_ptr()) }, 1);
        let mut aout = [0u8; 64];
        let an = unsafe { mt_deeplink_address(alink.as_ptr(), aout.as_mut_ptr(), aout.len()) };
        assert_eq!(&aout[..an], b"mt1qqqqqq");
    }

    #[test]
    fn ffi_deeplink_null_and_garbage_safe() {
        assert_eq!(unsafe { mt_deeplink_kind(std::ptr::null()) }, -1);
        let bad = CString::new("not-a-deeplink").unwrap();
        assert_eq!(unsafe { mt_deeplink_kind(bad.as_ptr()) }, -1);
    }

    #[test]
    fn ffi_rvdht_client_free_smoke() {
        // Клиент Mainline DHT создаётся и освобождается (без реального resolve — нужна сеть).
        let d = mt_rvdht_client();
        if !d.is_null() {
            unsafe { mt_rvdht_free(d) };
        }
        // null-безопасность
        unsafe { mt_rvdht_free(std::ptr::null_mut()) };
    }
}
