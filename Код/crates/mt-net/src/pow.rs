// spec, раздел "Сетевой уровень → IBT → Bootstrap exception" + Genesis Decree
//   protocol_params.bootstrap_pow_difficulty
//
// PoW formula:
//   target = 2^256 / bootstrap_pow_difficulty
//   valid_nonce: SHA-256("mt-bootstrap-pow" || proof || u64_LE(nonce)) < target
//
// Genesis-калибровка: bootstrap_pow_difficulty = 65 536 (2^16) → ≈100 ms CPU
// per попытка на genesis-железе.

use mt_codec::domain::BOOTSTRAP_POW;
use sha2::{Digest, Sha256};

// Re-export public-API constant под старым именем для минимизации cascade в Phase B.0 callsites.
pub use mt_codec::domain::BOOTSTRAP_POW as DOMAIN_BOOTSTRAP_POW;

pub const POW_HASH_SIZE: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowError {
    DifficultyZero,
    NonceSpaceExhausted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Target([u8; POW_HASH_SIZE]);

impl Target {
    pub fn from_difficulty(difficulty: u32) -> Result<Target, PowError> {
        if difficulty == 0 {
            return Err(PowError::DifficultyZero);
        }
        // target = floor(2^256 / difficulty)
        // Big-endian unsigned 257-bit -> 256-bit division.
        // Use 64-bit chunks.
        let mut quotient = [0u8; POW_HASH_SIZE];
        let d: u128 = difficulty as u128;
        // remainder accumulator (≤ 2^64 because dividend chunk is u64 and d ≤ 2^32)
        let mut rem: u128 = 0;
        // Numerator = 2^256: bytes [0]=1, rest = 0; treated as 33-byte
        // big-endian. We process MSB first.
        let num_bytes: [u8; POW_HASH_SIZE + 1] = {
            let mut b = [0u8; POW_HASH_SIZE + 1];
            b[0] = 1;
            b
        };
        for (i, &byte) in num_bytes.iter().enumerate() {
            // Shift remainder left 8 bits then OR new byte
            rem = (rem << 8) | (byte as u128);
            let q = rem / d;
            rem %= d;
            if i == 0 {
                // High overflow byte — must be zero for difficulty ≥ 1 since
                // 2^256 / 1 = 2^256 doesn't fit, but for difficulty ≥ 2 it
                // does. For difficulty = 1 we use full saturation.
                if q != 0 {
                    // difficulty == 1 case: target saturates to 0xFF*32
                    quotient = [0xFF; POW_HASH_SIZE];
                    return Ok(Target(quotient));
                }
            } else {
                quotient[i - 1] = q as u8;
            }
        }
        Ok(Target(quotient))
    }

    pub fn as_bytes(&self) -> &[u8; POW_HASH_SIZE] {
        &self.0
    }
}

#[inline]
fn pow_hash(proof_bytes: &[u8], nonce: u64) -> [u8; POW_HASH_SIZE] {
    let mut h = Sha256::new();
    h.update(BOOTSTRAP_POW);
    h.update(proof_bytes);
    h.update(nonce.to_le_bytes());
    let out = h.finalize();
    let mut r = [0u8; POW_HASH_SIZE];
    r.copy_from_slice(&out);
    r
}

#[inline]
fn hash_lt_target(hash: &[u8; POW_HASH_SIZE], target: &Target) -> bool {
    // Big-endian byte-wise comparison: hash < target.
    for (a, b) in hash.iter().zip(target.0.iter()) {
        match a.cmp(b) {
            core::cmp::Ordering::Less => return true,
            core::cmp::Ordering::Greater => return false,
            core::cmp::Ordering::Equal => continue,
        }
    }
    false
}

pub fn pow_verify(proof_bytes: &[u8], nonce: u64, difficulty: u32) -> bool {
    let target = match Target::from_difficulty(difficulty) {
        Ok(t) => t,
        Err(_) => return false,
    };
    let h = pow_hash(proof_bytes, nonce);
    hash_lt_target(&h, &target)
}

pub fn pow_solve(
    proof_bytes: &[u8],
    difficulty: u32,
    max_iterations: u64,
) -> Result<(u64, [u8; POW_HASH_SIZE]), PowError> {
    let target = Target::from_difficulty(difficulty)?;
    for nonce in 0..max_iterations {
        let h = pow_hash(proof_bytes, nonce);
        if hash_lt_target(&h, &target) {
            return Ok((nonce, h));
        }
    }
    Err(PowError::NonceSpaceExhausted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    #[test]
    fn target_difficulty_zero_rejected() {
        assert_eq!(Target::from_difficulty(0), Err(PowError::DifficultyZero));
    }

    #[test]
    fn target_difficulty_one_saturates() {
        let t = Target::from_difficulty(1).unwrap();
        assert_eq!(t.as_bytes(), &[0xFF; POW_HASH_SIZE]);
    }

    #[test]
    fn target_difficulty_2_pow_16_top_byte_zero() {
        // target = 2^256 / 2^16 = 2^240
        // Big-endian: byte[0] should be 0x01 at position 1 (256-240 = 16
        // = 2 bytes of zero MSB), so byte[0] = 0x00, byte[1] = 0x01.
        let t = Target::from_difficulty(65_536).unwrap();
        assert_eq!(t.as_bytes()[0], 0x00);
        assert_eq!(t.as_bytes()[1], 0x01);
        for &b in &t.as_bytes()[2..] {
            assert_eq!(b, 0x00);
        }
    }

    #[test]
    fn target_difficulty_2_pow_10_top_byte() {
        // target = 2^256 / 2^10 = 2^246; 256-246 = 10 bits zero MSB
        // First byte: 0b00000001 ... = bytes[1] highest 6 bits = 0x40
        let t = Target::from_difficulty(1024).unwrap();
        // floor(2^256/1024) = 2^246. In big-endian bytes: byte[0]=0,
        // byte[1] = 2^246 / 2^240 = 2^6 = 0x40, rest = 0.
        assert_eq!(t.as_bytes()[0], 0x00);
        assert_eq!(t.as_bytes()[1], 0x40);
        for &b in &t.as_bytes()[2..] {
            assert_eq!(b, 0x00);
        }
    }

    #[test]
    fn pow_solve_and_verify_difficulty_256() {
        // Difficulty = 2^8 = 256 — easy to find nonce in tests
        let proof: Vec<u8> = (0..16).collect();
        let (nonce, hash) = pow_solve(&proof, 256, 1_000_000).unwrap();
        assert!(pow_verify(&proof, nonce, 256));
        // Verify hash matches deterministic recompute
        let h = pow_hash(&proof, nonce);
        assert_eq!(h, hash);
    }

    #[test]
    fn pow_verify_wrong_nonce_rejected() {
        let proof: Vec<u8> = (0..16).collect();
        let (nonce, _) = pow_solve(&proof, 256, 1_000_000).unwrap();
        assert!(!pow_verify(&proof, nonce.wrapping_add(1), 256));
    }

    #[test]
    fn pow_verify_zero_difficulty_returns_false() {
        let proof: Vec<u8> = (0..16).collect();
        assert!(!pow_verify(&proof, 0, 0));
    }

    #[test]
    fn pow_solve_exhausts_nonce_space() {
        // Difficulty = 2^32 (large) with tiny iter budget — likely exhaust
        let proof: Vec<u8> = (0..16).collect();
        let r = pow_solve(&proof, u32::MAX, 10);
        match r {
            Err(PowError::NonceSpaceExhausted) => {},
            Ok((n, _)) => {
                // accidentally found — verify:
                assert!(pow_verify(&proof, n, u32::MAX));
            },
            Err(_) => panic!("unexpected err"),
        }
    }
}
