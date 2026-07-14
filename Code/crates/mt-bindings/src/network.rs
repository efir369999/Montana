//! C-ABI сетевого моста (Montana P2P Network, Этап 6): синхронные обёртки над сетевым
//! ядром (mt-postman/rendezvous/bootstrap) для Swift/Kotlin. Native-only —
//! tokio/quinn не компилируются в wasm. Модель: глобальный tokio-рантайм, блокирующие
//! FFI-вызовы (block_on). Паника через C-границу = UB, поэтому все ошибки —
//! через NULL/код возврата, без unwrap/panic на пути FFI.
//!
//! Thread-safety: хэндлы (MtPostman/MtClient/MtMdns) — Send+Sync, безопасны для
//! передачи и использования из любого потока. Одновременные вызовы на ОДНОМ хэндле
//! допустимы (Sync), но порядок операций между потоками не гарантирован.

use std::ffi::{CStr, CString};
use std::net::SocketAddr;
use std::os::raw::c_char;
use std::sync::{Arc, OnceLock};

use zeroize::Zeroize;

use mt_crypto::{MlkemPublicKey, SecretKey, SECRET_KEY_SIZE};
use mt_overlay::muq::{sign_deposit, HostDeposit, ProxyForward, Queue};
use mt_postman::{MuqClient, MuqState, PostmanServer};

fn rt() -> Option<&'static tokio::runtime::Runtime> {
    static RT: OnceLock<Option<tokio::runtime::Runtime>> = OnceLock::new();
    RT.get_or_init(|| tokio::runtime::Runtime::new().ok())
        .as_ref()
}

/// N-1: паника future не должна пробивать C-границу (UB). Оборачиваем block_on в
/// catch_unwind; паника → None → код ошибки FFI.
fn ffi_catch<R>(f: impl FnOnce() -> R) -> Option<R> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).ok()
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

/// Записать ML-KEM pubkey почтальона (1184 B) в `out` (ёмкость `out_cap`). Клиент
/// использует его для sealed-депозита. Возвращает записанные байты (0 при ошибке).
///
/// # Safety
/// `h` — валидный хэндл; `out` — буфер ≥ `out_cap` байт.
#[no_mangle]
pub unsafe extern "C" fn mt_postman_kem_pubkey(
    h: *const MtPostman,
    out: *mut u8,
    out_cap: usize,
) -> usize {
    if h.is_null() || out.is_null() {
        return 0;
    }
    let pk = (*h).muq().host_kem_pubkey();
    let bytes = pk.as_bytes();
    if bytes.len() > out_cap {
        return 0;
    }
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), out, bytes.len());
    bytes.len()
}

/// Opaque-хэндл клиента (живое QUIC-соединение к почтальону/курьеру).
pub struct MtClient {
    inner: MuqClient,
}

/// Подключиться к почтальону по адресу `addr` (host:port). Возвращает хэндл или null.
///
/// # Safety
/// `addr` — валидный C-string.
#[no_mangle]
pub unsafe extern "C" fn mt_client_connect(addr: *const c_char) -> *mut MtClient {
    let Some(a) = cstr_to_socketaddr(addr) else {
        return std::ptr::null_mut();
    };
    let Some(rt) = rt() else {
        return std::ptr::null_mut();
    };
    match ffi_catch(|| rt.block_on(MuqClient::connect(a))) {
        Some(Ok(inner)) => Box::into_raw(Box::new(MtClient { inner })),
        _ => std::ptr::null_mut(),
    }
}

/// Зарегистрировать очередь на хосте `host_overlay` (32 B) через курьер, к которому
/// подключён клиент. `host_kem` — 1184 B pubkey хоста; `queue` — сериализованный Queue
/// (`queue_len` байт). Возвращает 0 при успехе, -1 при ошибке.
///
/// # Safety
/// `client` — валидный хэндл; `host_overlay` → 32 B; `host_kem` → 1184 B; `queue` → `queue_len` B.
#[no_mangle]
pub unsafe extern "C" fn mt_client_register(
    client: *const MtClient,
    host_overlay: *const u8,
    host_kem: *const u8,
    queue: *const u8,
    queue_len: usize,
) -> i32 {
    if client.is_null() || host_overlay.is_null() || host_kem.is_null() || queue.is_null() {
        return -1;
    }
    let Some(rt) = rt() else {
        return -1;
    };
    let mut overlay = [0u8; 32];
    std::ptr::copy_nonoverlapping(host_overlay, overlay.as_mut_ptr(), 32);
    let kem_slice = std::slice::from_raw_parts(host_kem, mt_crypto::MLKEM_PUBLIC_KEY_SIZE);
    let Some(kem) = MlkemPublicKey::from_slice(kem_slice) else {
        return -1;
    };
    let q_slice = std::slice::from_raw_parts(queue, queue_len);
    let Ok(q) = Queue::decode(q_slice) else {
        return -1;
    };
    match ffi_catch(|| rt.block_on((*client).inner.register_via_courier(overlay, &kem, &q))) {
        Some(Ok(true)) => 0,
        _ => -1,
    }
}

/// Освободить хэндл клиента (закрывает соединение).
///
/// # Safety
/// `c` — хэндл из `mt_client_connect` либо null; не использовать повторно.
#[no_mangle]
pub unsafe extern "C" fn mt_client_free(c: *mut MtClient) {
    if !c.is_null() {
        drop(Box::from_raw(c));
    }
}

/// SAFETY-хелпер: собрать SecretKey из FFI-указателя (4032 B). None при null.
///
/// # Safety
/// `ptr` — указатель на `SECRET_KEY_SIZE` байт или null.
unsafe fn secret_from_ptr(ptr: *const u8) -> Option<SecretKey> {
    if ptr.is_null() {
        return None;
    }
    let mut arr = [0u8; SECRET_KEY_SIZE];
    std::ptr::copy_nonoverlapping(ptr, arr.as_mut_ptr(), SECRET_KEY_SIZE);
    let sk = SecretKey::from_array(arr);
    arr.zeroize(); // [u8; N] — Copy: from_array зачистил свою копию, не наш стек-остаток
    Some(sk)
}

/// Отправить сообщение `msg` в очередь `send_id` на хосте `host_overlay` через курьер
/// (двуххоп-депозит, sealed к ML-KEM хоста). Собирает HostDeposit+подпись+seal внутри.
/// Одиночный shard. Возвращает 0 при успехе, -1 при ошибке.
///
/// # Safety
/// `client` валиден; `host_overlay`→32; `host_kem`→1184; `send_id`→32; `send_sk`→4032;
/// `msg_id`→16; `msg`→`msg_len`.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn mt_client_send(
    client: *const MtClient,
    host_overlay: *const u8,
    host_kem: *const u8,
    send_id: *const u8,
    send_sk: *const u8,
    msg_id: *const u8,
    msg: *const u8,
    msg_len: usize,
) -> i32 {
    if client.is_null()
        || host_overlay.is_null()
        || host_kem.is_null()
        || send_id.is_null()
        || msg.is_null()
    {
        return -1;
    }
    let Some(rt) = rt() else {
        return -1;
    };
    let mut overlay = [0u8; 32];
    std::ptr::copy_nonoverlapping(host_overlay, overlay.as_mut_ptr(), 32);
    let kem_slice = std::slice::from_raw_parts(host_kem, mt_crypto::MLKEM_PUBLIC_KEY_SIZE);
    let Some(kem) = MlkemPublicKey::from_slice(kem_slice) else {
        return -1;
    };
    let mut sid = [0u8; 32];
    std::ptr::copy_nonoverlapping(send_id, sid.as_mut_ptr(), 32);
    let Some(sk) = secret_from_ptr(send_sk) else {
        return -1;
    };
    let mut mid = [0u8; 16];
    std::ptr::copy_nonoverlapping(msg_id, mid.as_mut_ptr(), 16);
    let ct = std::slice::from_raw_parts(msg, msg_len).to_vec();

    let mut nonce = [0u8; 16];
    if getrandom::getrandom(&mut nonce).is_err() {
        return -1;
    }
    let Ok(sig) = sign_deposit(&sk, &sid, &mid, &nonce) else {
        return -1;
    };
    let hd = HostDeposit {
        send_id: sid,
        msg_id: mid,
        ttl_windows: 240,
        shard_index: 0,
        shard_total: 1,
        nonce,
        ct,
        sig: sig.as_bytes().to_vec(),
    };
    let Ok(sealed) = mt_crypto::seal_to(&kem, &hd.to_bytes()) else {
        return -1;
    };
    let pf = ProxyForward {
        host_addr: overlay,
        sealed,
    };
    match ffi_catch(|| rt.block_on((*client).inner.deposit_via_proxy(&pf))) {
        Some(Ok(true)) => 0,
        _ => -1,
    }
}

/// Забрать одно сообщение из очереди `recv_id` на хосте `host_overlay` через курьер.
/// Пишет ct первого конверта в `out` (ёмкость `out_cap`), возвращает его длину;
/// 0 если очередь пуста или ошибка.
///
/// # Safety
/// `client` валиден; `host_overlay`→32; `host_kem`→1184; `recv_id`→32; `recv_sk`→4032;
/// `out`→`out_cap`.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn mt_client_recv(
    client: *const MtClient,
    host_overlay: *const u8,
    host_kem: *const u8,
    recv_id: *const u8,
    recv_sk: *const u8,
    out: *mut u8,
    out_cap: usize,
) -> usize {
    if client.is_null() || host_overlay.is_null() || host_kem.is_null() || recv_id.is_null() {
        return 0;
    }
    let Some(rt) = rt() else {
        return 0;
    };
    let mut overlay = [0u8; 32];
    std::ptr::copy_nonoverlapping(host_overlay, overlay.as_mut_ptr(), 32);
    let kem_slice = std::slice::from_raw_parts(host_kem, mt_crypto::MLKEM_PUBLIC_KEY_SIZE);
    let Some(kem) = MlkemPublicKey::from_slice(kem_slice) else {
        return 0;
    };
    let mut rid = [0u8; 32];
    std::ptr::copy_nonoverlapping(recv_id, rid.as_mut_ptr(), 32);
    let Some(sk) = secret_from_ptr(recv_sk) else {
        return 0;
    };
    let Some(Ok(resp)) = ffi_catch(|| {
        rt.block_on(
            (*client)
                .inner
                .subscribe_via_courier(overlay, &kem, rid, &sk),
        )
    }) else {
        return 0;
    };
    let Some(item) = resp.items.first() else {
        return 0;
    };
    if out.is_null() || item.ct.len() > out_cap {
        return 0;
    }
    std::ptr::copy_nonoverlapping(item.ct.as_ptr(), out, item.ct.len());
    item.ct.len()
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
    fn ffi_client_registers_queue_via_courier() {
        use mt_overlay::muq::{derive_queue_keypairs, Queue};
        let loop0 = CString::new("127.0.0.1:0").unwrap();
        // host + courier (FFI)
        let host = unsafe { mt_postman_start(loop0.as_ptr(), std::ptr::null_mut(), 0) };
        let courier = unsafe { mt_postman_start(loop0.as_ptr(), std::ptr::null_mut(), 0) };
        assert!(!host.is_null() && !courier.is_null());
        let host_port = unsafe { mt_postman_port(host) };
        let courier_port = unsafe { mt_postman_port(courier) };
        // маршрут курьер → host
        let host_overlay = [0xA0u8; 32];
        let host_addr = CString::new(format!("127.0.0.1:{host_port}")).unwrap();
        assert_eq!(
            unsafe { mt_postman_add_route(courier, host_overlay.as_ptr(), host_addr.as_ptr()) },
            0
        );
        // ML-KEM pubkey хоста через FFI
        let mut kem = [0u8; 1184];
        let n = unsafe { mt_postman_kem_pubkey(host, kem.as_mut_ptr(), kem.len()) };
        assert_eq!(n, 1184);
        // клиент подключается к курьеру
        let courier_addr = CString::new(format!("127.0.0.1:{courier_port}")).unwrap();
        let client = unsafe { mt_client_connect(courier_addr.as_ptr()) };
        assert!(!client.is_null(), "клиент подключён к курьеру");
        // собрать Queue (эфемерные ключи очереди)
        let ((recv_pk, _), (send_pk, _)) = derive_queue_keypairs(&[0x42u8; 32], 0).unwrap();
        let q = Queue {
            recv_id: [0x71u8; 32],
            send_id: [0x51u8; 32],
            recv_pubkey: *recv_pk.as_bytes(),
            send_pubkey: Some(*send_pk.as_bytes()),
            rotation_epoch: 1000,
            quota: 64,
        };
        let qb = q.to_bytes();
        // регистрация через курьер (host видит курьера, не клиента)
        let rc = unsafe {
            mt_client_register(
                client,
                host_overlay.as_ptr(),
                kem.as_ptr(),
                qb.as_ptr(),
                qb.len(),
            )
        };
        assert_eq!(rc, 0, "FFI-клиент зарегистрировал очередь через FFI-курьер");
        unsafe {
            mt_client_free(client);
            mt_postman_stop(host);
            mt_postman_stop(courier);
        }
    }

    #[test]
    fn ffi_full_message_exchange_a_to_b() {
        use mt_overlay::muq::{derive_queue_keypairs, Queue};
        let loop0 = CString::new("127.0.0.1:0").unwrap();
        let host = unsafe { mt_postman_start(loop0.as_ptr(), std::ptr::null_mut(), 0) };
        let courier = unsafe { mt_postman_start(loop0.as_ptr(), std::ptr::null_mut(), 0) };
        let host_port = unsafe { mt_postman_port(host) };
        let courier_port = unsafe { mt_postman_port(courier) };
        let host_overlay = [0xA0u8; 32];
        let host_addr = CString::new(format!("127.0.0.1:{host_port}")).unwrap();
        unsafe { mt_postman_add_route(courier, host_overlay.as_ptr(), host_addr.as_ptr()) };
        let mut kem = [0u8; 1184];
        unsafe { mt_postman_kem_pubkey(host, kem.as_mut_ptr(), kem.len()) };
        let courier_addr = CString::new(format!("127.0.0.1:{courier_port}")).unwrap();

        // ключи очереди
        let ((recv_pk, recv_sk), (send_pk, send_sk)) =
            derive_queue_keypairs(&[0x42u8; 32], 0).unwrap();
        let recv_id = [0x71u8; 32];
        let send_id = [0x51u8; 32];
        let q = Queue {
            recv_id,
            send_id,
            recv_pubkey: *recv_pk.as_bytes(),
            send_pubkey: Some(*send_pk.as_bytes()),
            rotation_epoch: 1000,
            quota: 64,
        };
        let qb = q.to_bytes();

        // B регистрирует очередь через курьер
        let b = unsafe { mt_client_connect(courier_addr.as_ptr()) };
        assert_eq!(
            unsafe {
                mt_client_register(
                    b,
                    host_overlay.as_ptr(),
                    kem.as_ptr(),
                    qb.as_ptr(),
                    qb.len(),
                )
            },
            0
        );

        // A отправляет сообщение через курьер
        let a = unsafe { mt_client_connect(courier_addr.as_ptr()) };
        let msg = b"privet cherez FFI most";
        let msg_id = [0x5Au8; 16];
        let rc = unsafe {
            mt_client_send(
                a,
                host_overlay.as_ptr(),
                kem.as_ptr(),
                send_id.as_ptr(),
                send_sk.as_bytes().as_ptr(),
                msg_id.as_ptr(),
                msg.as_ptr(),
                msg.len(),
            )
        };
        assert_eq!(rc, 0, "A отправил через FFI");

        // B забирает через курьер → байт-в-байт то же сообщение
        let b2 = unsafe { mt_client_connect(courier_addr.as_ptr()) };
        let mut out = [0u8; 256];
        let n = unsafe {
            mt_client_recv(
                b2,
                host_overlay.as_ptr(),
                kem.as_ptr(),
                recv_id.as_ptr(),
                recv_sk.as_bytes().as_ptr(),
                out.as_mut_ptr(),
                out.len(),
            )
        };
        assert_eq!(&out[..n], msg, "B получил сообщение A через FFI-мост");

        unsafe {
            mt_client_free(a);
            mt_client_free(b);
            mt_client_free(b2);
            mt_postman_stop(host);
            mt_postman_stop(courier);
        }
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
