//! C-ABI сетевого моста (Montana P2P Network, Этап 6): синхронные обёртки над сетевым
//! ядром (mt-postman/rendezvous/bootstrap) для Swift/Kotlin. Native-only —
//! tokio/quinn не компилируются в wasm. Модель: глобальный tokio-рантайм, блокирующие
//! FFI-вызовы (block_on). Паника через C-границу = UB, поэтому все ошибки —
//! через NULL/код возврата, без unwrap/panic на пути FFI.

use std::ffi::{CStr, CString};
use std::net::SocketAddr;
use std::os::raw::c_char;
use std::sync::{Arc, OnceLock};

use mt_postman::{MuqState, PostmanServer};

fn rt() -> Option<&'static tokio::runtime::Runtime> {
    static RT: OnceLock<Option<tokio::runtime::Runtime>> = OnceLock::new();
    RT.get_or_init(|| tokio::runtime::Runtime::new().ok())
        .as_ref()
}

/// Opaque-хэндл живого почтальона для FFI. Держит адрес, задачу цикла (для остановки)
/// и MuqState (маршруты курьера).
pub struct MtPostman {
    addr: SocketAddr,
    task: tokio::task::JoinHandle<()>,
    muq: Arc<MuqState>,
}

/// # Safety
/// `bind` — валидный C-string (null-terminated) или null. Вызывающий владеет им.
unsafe fn cstr_to_socketaddr(bind: *const c_char) -> Option<SocketAddr> {
    if bind.is_null() {
        return None;
    }
    CStr::from_ptr(bind).to_str().ok()?.parse().ok()
}

/// Запустить почтальон на `bind` (например "0.0.0.0:0"). При успехе записывает реальный
/// адрес (host:port) в `out_addr` (буфер ёмкости `out_cap`, null-terminated) и возвращает
/// opaque-хэндл; при ошибке — null. Хэндл освобождается `mt_postman_stop`.
///
/// # Safety
/// `bind` — валидный C-string; `out_addr` — буфер ≥ `out_cap` байт или null.
#[no_mangle]
pub unsafe extern "C" fn mt_postman_start(
    bind: *const c_char,
    out_addr: *mut c_char,
    out_cap: usize,
) -> *mut MtPostman {
    let Some(addr) = cstr_to_socketaddr(bind) else {
        return std::ptr::null_mut();
    };
    let Some(rt) = rt() else {
        return std::ptr::null_mut();
    };
    let _guard = rt.enter(); // quinn Endpoint::bind требует контекст реактора
    let Ok(server) = PostmanServer::bind(addr) else {
        return std::ptr::null_mut();
    };
    let Ok(real_addr) = server.local_addr() else {
        return std::ptr::null_mut();
    };
    let muq = server.muq().clone();
    let task = rt.spawn(server.run());

    // записать реальный адрес в out_addr, если запрошен
    if !out_addr.is_null() && out_cap > 0 {
        if let Ok(cs) = CString::new(real_addr.to_string()) {
            let bytes = cs.as_bytes_with_nul();
            if bytes.len() <= out_cap {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), out_addr as *mut u8, bytes.len());
            }
        }
    }

    Box::into_raw(Box::new(MtPostman {
        addr: real_addr,
        task,
        muq,
    }))
}

/// Остановить почтальон и освободить хэндл. После вызова `h` невалиден.
///
/// # Safety
/// `h` — хэндл из `mt_postman_start` либо null; не использовать повторно.
#[no_mangle]
pub unsafe extern "C" fn mt_postman_stop(h: *mut MtPostman) {
    if h.is_null() {
        return;
    }
    let postman = Box::from_raw(h);
    postman.task.abort();
    // Box drop освобождает MtPostman; muq (Arc) убывает.
}

/// Порт живого почтальона (0 при null-хэндле) — для диагностики/теста.
///
/// # Safety
/// `h` — валидный хэндл из `mt_postman_start` либо null.
#[no_mangle]
pub unsafe extern "C" fn mt_postman_port(h: *const MtPostman) -> u16 {
    if h.is_null() {
        return 0;
    }
    (*h).addr.port()
}

/// Добавить маршрут курьера: `overlay` (32 B оверлей-адрес хоста) → физический `target`
/// (host:port). Возвращает 0 при успехе, -1 при ошибке аргументов. Модель relay Этапа 3.
///
/// # Safety
/// `h` — валидный хэндл; `overlay` — указатель на 32 байта; `target` — C-string.
#[no_mangle]
pub unsafe extern "C" fn mt_postman_add_route(
    h: *const MtPostman,
    overlay: *const u8,
    target: *const c_char,
) -> i32 {
    if h.is_null() || overlay.is_null() {
        return -1;
    }
    let Some(addr) = cstr_to_socketaddr(target) else {
        return -1;
    };
    let mut ov = [0u8; 32];
    std::ptr::copy_nonoverlapping(overlay, ov.as_mut_ptr(), 32);
    (*h).muq().add_proxy_route(ov, addr);
    0
}

impl MtPostman {
    pub(crate) fn muq(&self) -> &Arc<MuqState> {
        &self.muq
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn postman_start_reports_port_and_stops() {
        let bind = CString::new("127.0.0.1:0").unwrap();
        let mut buf = [0u8; 64];
        let h =
            unsafe { mt_postman_start(bind.as_ptr(), buf.as_mut_ptr() as *mut c_char, buf.len()) };
        assert!(!h.is_null(), "почтальон запущен");
        let port = unsafe { mt_postman_port(h) };
        assert!(port != 0, "реальный порт присвоен");
        // out_addr содержит 127.0.0.1:<port>
        let reported = unsafe { CStr::from_ptr(buf.as_ptr() as *const c_char) }
            .to_str()
            .unwrap();
        assert!(
            reported.starts_with("127.0.0.1:"),
            "адрес записан: {reported}"
        );
        unsafe { mt_postman_stop(h) };
    }

    #[test]
    fn add_route_ok_and_rejects_bad_args() {
        let bind = CString::new("127.0.0.1:0").unwrap();
        let h = unsafe { mt_postman_start(bind.as_ptr(), std::ptr::null_mut(), 0) };
        assert!(!h.is_null());
        let overlay = [0xA0u8; 32];
        let target = CString::new("127.0.0.1:9999").unwrap();
        assert_eq!(
            unsafe { mt_postman_add_route(h, overlay.as_ptr(), target.as_ptr()) },
            0
        );
        // мусорный target → -1
        let bad = CString::new("xxx").unwrap();
        assert_eq!(
            unsafe { mt_postman_add_route(h, overlay.as_ptr(), bad.as_ptr()) },
            -1
        );
        // null-хэндл → -1
        assert_eq!(
            unsafe { mt_postman_add_route(std::ptr::null(), overlay.as_ptr(), target.as_ptr()) },
            -1
        );
        unsafe { mt_postman_stop(h) };
    }

    #[test]
    fn bad_bind_returns_null() {
        let bad = CString::new("not-an-addr").unwrap();
        let h = unsafe { mt_postman_start(bad.as_ptr(), std::ptr::null_mut(), 0) };
        assert!(h.is_null(), "мусорный адрес → null");
        // null-хэндлы безопасны
        assert_eq!(unsafe { mt_postman_port(std::ptr::null()) }, 0);
        unsafe { mt_postman_stop(std::ptr::null_mut()) };
    }
}
