//! Оверлей-адресация P2P-сети (Этап 1): адрес узла — хеш его account_id,
//! не физический IP. Спека: Montana P2P Network, «Оверлей-адрес (byte-exact)».

pub mod challenge;
pub mod dedup;
pub mod erasure;
pub mod frame;
pub mod inbox;
pub mod inbox_store;
pub mod muq;
pub mod postman;
pub mod prologue;

pub use mt_state::AccountId;

// SSOT: 32-байтный хеш — mt_crypto::Hash32; оверлей-адрес такой же ширины.
pub const OVERLAY_ADDR_SIZE: usize = mt_crypto::HASH_SIZE;

pub type OverlayAddr = mt_crypto::Hash32;

// spec: overlay_addr = SHA-256("mt-overlay" || 0x00 || account_id)
pub fn overlay_addr(account_id: &AccountId) -> OverlayAddr {
    mt_crypto::hash(mt_codec::domain::OVERLAY, &[account_id])
}

#[cfg(test)]
mod tests {
    use super::*;

    // Детерминированный LCG для property-тестов без новых зависимостей.
    fn lcg_fill(state: &mut u64, out: &mut [u8; 32]) {
        for b in out.iter_mut() {
            *state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            *b = (*state >> 33) as u8;
        }
    }

    #[test]
    fn deterministic_on_1000_random_inputs() {
        let mut st = 0x9E3779B97F4A7C15u64;
        for _ in 0..1000 {
            let mut acc = [0u8; 32];
            lcg_fill(&mut st, &mut acc);
            assert_eq!(overlay_addr(&acc), overlay_addr(&acc));
        }
    }

    #[test]
    fn distinct_inputs_distinct_addrs() {
        let mut st = 0xD1B54A32D192ED03u64;
        let mut prev = overlay_addr(&[0u8; 32]);
        for _ in 0..1000 {
            let mut acc = [0u8; 32];
            lcg_fill(&mut st, &mut acc);
            let a = overlay_addr(&acc);
            assert_ne!(a, prev);
            prev = a;
        }
    }

    #[test]
    fn differs_from_raw_sha256_and_from_other_domains() {
        // Домен-сепаратор реально участвует: не совпадает ни с raw SHA-256(account_id),
        // ни с хешем под другим доменом реестра.
        let acc = [0x22u8; 32];
        assert_ne!(overlay_addr(&acc), mt_crypto::sha256_raw(&acc));
        assert_ne!(
            overlay_addr(&acc),
            mt_crypto::hash(mt_codec::domain::ACCOUNT, &[&acc])
        );
    }
}
