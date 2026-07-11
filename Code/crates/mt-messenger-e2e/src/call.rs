//! Этап 13 — постквантовый медиа-слой звонка (канонический профиль).
//! call_seed доставляется внутри храповика (Этап 6) → PQ-защищён. Из него выводятся
//! call_key и sframe_key (ключ AEAD медиа-кадров SFrame поверх SRTP). Домены "mt-call"/"mt-call-sframe".

use crate::kdf::hkdf_sha256;

/// call_key = HKDF-SHA-256(salt=0×32, IKM=call_seed, info="mt-call", 32).
pub fn call_key(call_seed: &[u8; 32]) -> [u8; 32] {
    let okm = hkdf_sha256(&[0u8; 32], call_seed, b"mt-call", 32);
    let mut out = [0u8; 32];
    out.copy_from_slice(&okm);
    out
}

/// sframe_key = HKDF-SHA-256(salt=0×32, IKM=call_key, info="mt-call-sframe", 32).
pub fn sframe_key(call_key: &[u8; 32]) -> [u8; 32] {
    let okm = hkdf_sha256(&[0u8; 32], call_key, b"mt-call-sframe", 32);
    let mut out = [0u8; 32];
    out.copy_from_slice(&okm);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_key_spec_kat() {
        let ck = call_key(&[0x44; 32]);
        assert_eq!(
            hex::encode(ck),
            "c0a443e76155b699d691a3902eedc5c0f43ec860a28b57f6ca70633fc1d99bde"
        );
        let sf = sframe_key(&ck);
        assert_eq!(
            hex::encode(sf),
            "5a23f3dfbad643d36fddf3c2d415371c0532a5a5640c514c061d94c1a47e3d84"
        );
    }
}
