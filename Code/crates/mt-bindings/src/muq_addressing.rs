//! FFI слой адресации MUQ: деривация ключей очереди, сериализация Queue, генерация id.
//! Дополняет транспортные глаголы (network.rs) слоем адресации клиента-приложения:
//! приложение строит очередь (recv/send id + ключи), сериализует для register, генерит id.
//! SSOT логики — mt_overlay::muq (derive_queue_keypairs, Queue::to_bytes); здесь FFI-обёртка.

use mt_crypto::{PUBLIC_KEY_SIZE, SECRET_KEY_SIZE};
use mt_overlay::muq::{derive_queue_keypairs, Queue, QUEUE_ID_SIZE, QUEUE_WIRE_SIZE};

/// Деривация ключей очереди из routing_secret(32)+queue_index — recv/send ML-DSA keypairs.
/// out_recv_pk[1952] out_recv_sk[4032] out_send_pk[1952] out_send_sk[4032]. 0=успех, -1=ошибка.
///
/// # Safety
/// routing_secret → 32 B; out_* — валидные буферы на указанные размеры.
#[no_mangle]
pub unsafe extern "C" fn mt_muq_derive_queue_keys(
    routing_secret: *const u8,
    queue_index: u64,
    out_recv_pk: *mut u8,
    out_recv_sk: *mut u8,
    out_send_pk: *mut u8,
    out_send_sk: *mut u8,
) -> i32 {
    if routing_secret.is_null()
        || out_recv_pk.is_null()
        || out_recv_sk.is_null()
        || out_send_pk.is_null()
        || out_send_sk.is_null()
    {
        return -1;
    }
    let mut rs = [0u8; 32];
    std::ptr::copy_nonoverlapping(routing_secret, rs.as_mut_ptr(), 32);
    let Ok(((rpk, rsk), (spk, ssk))) = derive_queue_keypairs(&rs, queue_index) else {
        return -1;
    };
    std::ptr::copy_nonoverlapping(rpk.as_bytes().as_ptr(), out_recv_pk, PUBLIC_KEY_SIZE);
    std::ptr::copy_nonoverlapping(rsk.as_bytes().as_ptr(), out_recv_sk, SECRET_KEY_SIZE);
    std::ptr::copy_nonoverlapping(spk.as_bytes().as_ptr(), out_send_pk, PUBLIC_KEY_SIZE);
    std::ptr::copy_nonoverlapping(ssk.as_bytes().as_ptr(), out_send_sk, SECRET_KEY_SIZE);
    0
}

/// Сериализация Queue (wire §413) для регистрации. send_pk null = unsecured-очередь.
/// Возврат: записанные байты (QUEUE_WIRE_SIZE) или 0 при ошибке/малом буфере.
///
/// # Safety
/// recv_id/send_id/recv_pk → 32/32/1952; send_pk → 1952 или null; out → out_cap байт.
#[no_mangle]
pub unsafe extern "C" fn mt_muq_queue_serialize(
    recv_id: *const u8,
    send_id: *const u8,
    recv_pk: *const u8,
    send_pk: *const u8,
    rotation_epoch: u64,
    quota: u32,
    out: *mut u8,
    out_cap: usize,
) -> usize {
    if recv_id.is_null() || send_id.is_null() || recv_pk.is_null() || out.is_null() {
        return 0;
    }
    if out_cap < QUEUE_WIRE_SIZE {
        return 0;
    }
    let mut rid = [0u8; QUEUE_ID_SIZE];
    let mut sid = [0u8; QUEUE_ID_SIZE];
    std::ptr::copy_nonoverlapping(recv_id, rid.as_mut_ptr(), QUEUE_ID_SIZE);
    std::ptr::copy_nonoverlapping(send_id, sid.as_mut_ptr(), QUEUE_ID_SIZE);
    let mut rpk = [0u8; PUBLIC_KEY_SIZE];
    std::ptr::copy_nonoverlapping(recv_pk, rpk.as_mut_ptr(), PUBLIC_KEY_SIZE);
    let spk = if send_pk.is_null() {
        None
    } else {
        let mut s = [0u8; PUBLIC_KEY_SIZE];
        std::ptr::copy_nonoverlapping(send_pk, s.as_mut_ptr(), PUBLIC_KEY_SIZE);
        Some(s)
    };
    let q = Queue {
        recv_id: rid,
        send_id: sid,
        recv_pubkey: rpk,
        send_pubkey: spk,
        rotation_epoch,
        quota,
    };
    let bytes = q.to_bytes();
    if bytes.len() > out_cap {
        return 0;
    }
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), out, bytes.len());
    bytes.len()
}

/// Случайный QueueId (32 B, OS CSPRNG) — recv_id либо send_id. 0=успех, -1=ошибка.
///
/// # Safety
/// out — валиден на 32 байта.
#[no_mangle]
pub unsafe extern "C" fn mt_muq_gen_queue_id(out: *mut u8) -> i32 {
    if out.is_null() {
        return -1;
    }
    let mut id = [0u8; QUEUE_ID_SIZE];
    if getrandom::getrandom(&mut id).is_err() {
        return -1;
    }
    std::ptr::copy_nonoverlapping(id.as_ptr(), out, QUEUE_ID_SIZE);
    0
}
