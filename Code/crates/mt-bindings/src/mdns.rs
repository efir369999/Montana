//! mDNS-обнаружение узлов Montana в локальной сети (Montana P2P Network, Этап 6, п.3):
//! `_montana._udp.local`. Под-режим A′ Этапа 5 — два телефона в одной Wi-Fi находят
//! друг друга без конфига и без DHT. Discovery-слой: адрес из mDNS проверяется дальше
//! (E2E / overlay), компрометация mDNS = неверный адрес → detected, не breach.

use std::ffi::CString;
use std::os::raw::c_char;
use std::time::Duration;

use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};

const SERVICE_TYPE: &str = "_montana._udp.local.";

/// Opaque-хэндл mDNS-демона (держит регистрацию сервиса живой).
pub struct MtMdns {
    daemon: ServiceDaemon,
}

/// Анонсировать свой почтальон в локальной сети на порту `port`. Возвращает хэндл
/// (демон держит анонс живым) или null. `instance` — C-string имя экземпляра.
///
/// # Safety
/// `instance` — валидный C-string или null.
#[no_mangle]
pub unsafe extern "C" fn mt_mdns_advertise(port: u16, instance: *const c_char) -> *mut MtMdns {
    let name = if instance.is_null() {
        "montana-node".to_string()
    } else {
        match std::ffi::CStr::from_ptr(instance).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return std::ptr::null_mut(),
        }
    };
    let Ok(daemon) = ServiceDaemon::new() else {
        return std::ptr::null_mut();
    };
    let host = format!("{name}.local.");
    let Ok(info) = ServiceInfo::new(SERVICE_TYPE, &name, &host, "", port, None) else {
        return std::ptr::null_mut();
    };
    let info = info.enable_addr_auto(); // авто-адреса интерфейсов
    if daemon.register(info).is_err() {
        return std::ptr::null_mut();
    }
    Box::into_raw(Box::new(MtMdns { daemon }))
}

/// Найти узлы Montana в локальной сети за `timeout_ms`. Пишет найденные адреса в `out`
/// как "ip:port\n"-разделённый ASCII (ёмкость `out_cap`, null-terminated), возвращает
/// число найденных узлов (0 если никого / ошибка).
///
/// # Safety
/// `out` — буфер ≥ `out_cap` байт или null.
#[no_mangle]
pub unsafe extern "C" fn mt_mdns_browse(timeout_ms: u32, out: *mut u8, out_cap: usize) -> usize {
    let Ok(daemon) = ServiceDaemon::new() else {
        return 0;
    };
    let Ok(receiver) = daemon.browse(SERVICE_TYPE) else {
        return 0;
    };
    let deadline = Duration::from_millis(timeout_ms as u64);
    let mut found: Vec<String> = Vec::new();
    // собираем ServiceResolved до дедлайна
    let start = std::time::Instant::now();
    while start.elapsed() < deadline {
        let remaining = deadline.saturating_sub(start.elapsed());
        match receiver.recv_timeout(remaining) {
            Ok(ServiceEvent::ServiceResolved(info)) => {
                let port = info.get_port();
                for addr in info.get_addresses() {
                    let ep = format!("{addr}:{port}");
                    if !found.contains(&ep) {
                        found.push(ep);
                    }
                }
            },
            Ok(_) => continue,
            Err(_) => break,
        }
    }
    let _ = daemon.shutdown();

    if !out.is_null() && out_cap > 0 {
        let joined = found.join("\n");
        if let Ok(cs) = CString::new(joined) {
            let bytes = cs.as_bytes_with_nul();
            if bytes.len() <= out_cap {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), out, bytes.len());
            }
        }
    }
    found.len()
}

/// Остановить анонс и освободить хэндл.
///
/// # Safety
/// `h` — хэндл из `mt_mdns_advertise` либо null; не использовать повторно.
#[no_mangle]
pub unsafe extern "C" fn mt_mdns_stop(h: *mut MtMdns) {
    if h.is_null() {
        return;
    }
    let m = Box::from_raw(h);
    let _ = m.daemon.shutdown();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advertise_handle_and_stop() {
        let instance = CString::new("test-node").unwrap();
        let h = unsafe { mt_mdns_advertise(8444, instance.as_ptr()) };
        assert!(!h.is_null(), "mDNS-анонс запущен");
        unsafe { mt_mdns_stop(h) };
        // null безопасен
        unsafe { mt_mdns_stop(std::ptr::null_mut()) };
    }

    #[test]
    fn browse_short_timeout_no_crash() {
        // короткий browse не должен паниковать; count ≥ 0 (в sandbox обычно 0)
        let mut out = [0u8; 512];
        let n = unsafe { mt_mdns_browse(300, out.as_mut_ptr(), out.len()) };
        let _ = n; // в CI/sandbox multicast может быть недоступен — важно, что нет паники
    }
}
