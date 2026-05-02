// spec, раздел "Вход и регистрация → Genesis State"

pub mod manifest;

pub use manifest::{GenesisManifest, GenesisPeer, ManifestError};

use std::sync::OnceLock;

use mt_codec::{domain, write_bytes, write_u128, write_u16, write_u64, write_u8, CanonicalEncode};
use mt_crypto::{hash, Hash32, PUBLIC_KEY_SIZE};

// PARAMS_ENCODED_SIZE: layout sum для protocol_params (см. spec раздел "Указ Генезиса").
// Layout (LE):
// d0(8) + reserved(8) + tau2(8) + emission(16) + target_zero(32) + quorum_num(1)
// + quorum_den(1) + dead_zone(2+2) + d_adj(2+2) + vdf_entry(8) + sel_interval(8)
// + admission_divisor(8) + cand_expiry(8) + adapt_thr(2) + adapt_mult(2) + pruning(8)
// + 2×pubkey(2×1952=3904) + app_id(32) + data_hash(32) = 4094 bytes.
pub const PARAMS_ENCODED_SIZE: usize = 4094;

// === Genesis Ceremony 2026-05-02 — финализированные значения ===
//
// Bootstrap operator: Moscow node `montana-moscow` (<front>)
//   account_id = 4c290c3d5d63e84b99c30c83fb4d172e04102af4492b4d56d0642711b09e2072
//   node_id    = 75bfaf9026405c12ef36437f08cc63c040cfe1924773dedcba0abadf8c6928a1
//
// Genesis cohort: 3 узла (мос/фра/зел). После ceremony эти константы immutable.

/// 1952 байта ML-DSA-65 публичного ключа Moscow operator account.
pub const BOOTSTRAP_ACCOUNT_PUBKEY_BYTES: &[u8; PUBLIC_KEY_SIZE] =
    include_bytes!("../include/bootstrap-account-pk.bin");

/// 1952 байта ML-DSA-65 публичного ключа Moscow consensus node.
pub const BOOTSTRAP_NODE_PUBKEY_BYTES: &[u8; PUBLIC_KEY_SIZE] =
    include_bytes!("../include/bootstrap-node-pk.bin");

/// 32 байта initial VDF target. SHA-256("mt-genesis" || account_pk || node_pk
/// || "montana-genesis-3node-2026-05-02").
pub const TARGET_ZERO_BYTES: [u8; 32] = [
    0x23, 0xAE, 0x24, 0x0D, 0xBF, 0xC6, 0x04, 0x9F, 0xA4, 0xE3, 0x23, 0x0D, 0x4A, 0x05, 0x43, 0x46,
    0xD2, 0x56, 0x29, 0x1E, 0xA0, 0x4C, 0xFE, 0xA8, 0xE4, 0x7B, 0xBC, 0xAF, 0xB9, 0x07, 0x3D, 0x97,
];

/// 32 байта хэш Genesis content. SHA-256("montana-genesis-content-2026-05-02-bootstrap").
pub const GENESIS_CONTENT_DATA_HASH_BYTES: [u8; 32] = [
    0xF7, 0xD8, 0xEF, 0x6F, 0x50, 0x24, 0x40, 0x1B, 0x67, 0x64, 0xA1, 0x96, 0xA3, 0x32, 0xF7, 0xA4,
    0x8D, 0x59, 0x2D, 0xF1, 0x26, 0x59, 0xBB, 0x96, 0x60, 0x0F, 0xB2, 0x26, 0xBF, 0xA0, 0xD5, 0x36,
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolParams {
    pub d0: u64,
    pub reserved_m0: [u8; 8],
    pub tau2_windows: u64,
    // spec, раздел "Эмиссия": const emission `reward_moneta(W) = EMISSION_moneta`.
    pub emission_moneta: u128,
    pub target_zero: [u8; 32],
    pub confirmation_quorum_num: u8,
    pub confirmation_quorum_den: u8,
    pub participation_dead_zone_low: u16,
    pub participation_dead_zone_high: u16,
    pub d_adjustment_rate_num: u16,
    pub d_adjustment_rate_den: u16,
    pub vdf_entry_windows: u64,
    pub selection_interval: u64,
    // spec v33.1.6+: slots = max(1, floor(active_nodes / admission_divisor))
    // per selection event. Pin 130 даёт 1/130 ≈ 0.77% steady-state admission
    // rate < 1% upper bound. [C-1] SSOT: ранее жил как hardcoded const в
    // mt-entry::ADMISSION_DIVISOR (M4-LOW-7 closure).
    pub admission_divisor: u64,
    pub candidate_expiry_windows: u64,
    pub adaptive_vdf_threshold: u16,
    pub adaptive_vdf_multiplier: u16,
    pub pruning_idle_windows: u64,
    pub bootstrap_account_pubkey: [u8; PUBLIC_KEY_SIZE],
    pub bootstrap_node_pubkey: [u8; PUBLIC_KEY_SIZE],
    pub genesis_content_app_id: Hash32,
    pub genesis_content_data_hash: Hash32,
}

impl CanonicalEncode for ProtocolParams {
    fn encode(&self, buf: &mut Vec<u8>) {
        write_u64(buf, self.d0);
        write_bytes(buf, &self.reserved_m0);
        write_u64(buf, self.tau2_windows);
        write_u128(buf, self.emission_moneta);
        write_bytes(buf, &self.target_zero);
        write_u8(buf, self.confirmation_quorum_num);
        write_u8(buf, self.confirmation_quorum_den);
        write_u16(buf, self.participation_dead_zone_low);
        write_u16(buf, self.participation_dead_zone_high);
        write_u16(buf, self.d_adjustment_rate_num);
        write_u16(buf, self.d_adjustment_rate_den);
        write_u64(buf, self.vdf_entry_windows);
        write_u64(buf, self.selection_interval);
        write_u64(buf, self.admission_divisor);
        write_u64(buf, self.candidate_expiry_windows);
        write_u16(buf, self.adaptive_vdf_threshold);
        write_u16(buf, self.adaptive_vdf_multiplier);
        write_u64(buf, self.pruning_idle_windows);
        write_bytes(buf, &self.bootstrap_account_pubkey);
        write_bytes(buf, &self.bootstrap_node_pubkey);
        write_bytes(buf, &self.genesis_content_app_id);
        write_bytes(buf, &self.genesis_content_data_hash);
    }
}

// spec: genesis_content_app_id = SHA-256("mt-app" || "montana")
pub fn genesis_app_id() -> Hash32 {
    hash(domain::APP, &[b"montana"])
}

pub fn genesis_params() -> &'static ProtocolParams {
    static INSTANCE: OnceLock<ProtocolParams> = OnceLock::new();
    INSTANCE.get_or_init(|| ProtocolParams {
        d0: 325_000_000,
        reserved_m0: [0u8; 8],
        tau2_windows: 20_160,
        emission_moneta: 13_000_000_000,
        target_zero: TARGET_ZERO_BYTES,
        confirmation_quorum_num: 67,
        confirmation_quorum_den: 100,
        participation_dead_zone_low: 85,
        participation_dead_zone_high: 95,
        d_adjustment_rate_num: 3,
        d_adjustment_rate_den: 100,
        vdf_entry_windows: 20_160,
        selection_interval: 336,
        admission_divisor: 130,
        candidate_expiry_windows: 60_480,
        adaptive_vdf_threshold: 1,
        adaptive_vdf_multiplier: 100,
        pruning_idle_windows: 80_640,
        bootstrap_account_pubkey: *BOOTSTRAP_ACCOUNT_PUBKEY_BYTES,
        bootstrap_node_pubkey: *BOOTSTRAP_NODE_PUBKEY_BYTES,
        genesis_content_app_id: genesis_app_id(),
        genesis_content_data_hash: GENESIS_CONTENT_DATA_HASH_BYTES,
    })
}

// spec v29.7.1+: Genesis State Hash = SHA-256("mt-genesis" || genesis_state_root || canonical_encode(protocol_params))
pub fn compute_genesis_state_hash(state_root: &Hash32, params: &ProtocolParams) -> Hash32 {
    let mut encoded = Vec::with_capacity(PARAMS_ENCODED_SIZE);
    params.encode(&mut encoded);
    hash(domain::GENESIS, &[state_root, &encoded])
}

// Programmatic check для статуса финализации Genesis ceremony.
// Возвращает true если все 4 ceremony-controlled поля содержат
// non-placeholder values (non-zero):
//   - bootstrap_account_pubkey
//   - bootstrap_node_pubkey
//   - target_zero (initial VDF target)
//   - genesis_content_data_hash
//
// До mainnet ceremony — возвращает false (поля = placeholders [0; N]).
// После ceremony — возвращает true; Genesis Decree становится immutable
// и singleton начинает раздавать финализированные значения.
//
// Использование в operator deployment script: assert is_finalized() == true
// перед start узла, иначе fail-fast с инструкциями к ceremony.
pub fn is_genesis_bootstrap_finalized(params: &ProtocolParams) -> bool {
    params.bootstrap_account_pubkey != [0u8; PUBLIC_KEY_SIZE]
        && params.bootstrap_node_pubkey != [0u8; PUBLIC_KEY_SIZE]
        && params.target_zero != [0u8; 32]
        && params.genesis_content_data_hash != [0u8; 32]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn params_encoded_size_matches_layout() {
        let mut buf = Vec::new();
        genesis_params().encode(&mut buf);
        assert_eq!(buf.len(), PARAMS_ENCODED_SIZE);
        assert_eq!(PARAMS_ENCODED_SIZE, 4094);
    }

    #[test]
    fn spec_constants_match() {
        let p = genesis_params();
        assert_eq!(p.d0, 325_000_000);
        assert_eq!(p.reserved_m0, [0u8; 8]);
        assert_eq!(p.tau2_windows, 20_160);
        assert_eq!(p.emission_moneta, 13_000_000_000);
        assert_eq!(p.confirmation_quorum_num, 67);
        assert_eq!(p.confirmation_quorum_den, 100);
        assert_eq!(p.participation_dead_zone_low, 85);
        assert_eq!(p.participation_dead_zone_high, 95);
        assert_eq!(p.d_adjustment_rate_num, 3);
        assert_eq!(p.d_adjustment_rate_den, 100);
        assert_eq!(p.vdf_entry_windows, 20_160);
        assert_eq!(p.selection_interval, 336);
        assert_eq!(p.admission_divisor, 130);
        assert_eq!(p.candidate_expiry_windows, 60_480);
        assert_eq!(p.adaptive_vdf_threshold, 1);
        assert_eq!(p.adaptive_vdf_multiplier, 100);
        assert_eq!(p.pruning_idle_windows, 80_640);
    }

    #[test]
    fn tau2_equals_vdf_entry() {
        let p = genesis_params();
        assert_eq!(p.tau2_windows, p.vdf_entry_windows);
    }

    #[test]
    fn candidate_expiry_is_3_tau2() {
        let p = genesis_params();
        assert_eq!(p.candidate_expiry_windows, 3 * p.tau2_windows);
    }

    #[test]
    fn pruning_idle_is_4_tau2() {
        let p = genesis_params();
        assert_eq!(p.pruning_idle_windows, 4 * p.tau2_windows);
    }

    #[test]
    fn selection_interval_divides_tau2() {
        let p = genesis_params();
        assert_eq!(p.tau2_windows % p.selection_interval, 0);
        assert_eq!(p.tau2_windows / p.selection_interval, 60);
    }

    #[test]
    fn genesis_app_id_deterministic() {
        assert_eq!(genesis_app_id(), genesis_app_id());
    }

    #[test]
    fn genesis_app_id_matches_formula() {
        let expected = hash(domain::APP, &[b"montana"]);
        assert_eq!(genesis_app_id(), expected);
        assert_eq!(genesis_params().genesis_content_app_id, genesis_app_id());
    }

    #[test]
    fn first_8_bytes_encode_d0_little_endian() {
        let mut buf = Vec::new();
        genesis_params().encode(&mut buf);
        assert_eq!(&buf[..8], &genesis_params().d0.to_le_bytes());
        assert_eq!(&buf[..8], &[0x40, 0x1B, 0x5F, 0x13, 0, 0, 0, 0]);
    }

    #[test]
    fn bytes_8_to_16_are_reserved_zeros() {
        let mut buf = Vec::new();
        genesis_params().encode(&mut buf);
        assert_eq!(&buf[8..16], &[0u8; 8]);
    }

    #[test]
    fn bytes_16_to_24_encode_tau2() {
        let mut buf = Vec::new();
        genesis_params().encode(&mut buf);
        assert_eq!(&buf[16..24], &20_160u64.to_le_bytes());
    }

    #[test]
    fn bytes_24_to_40_encode_emission_moneta() {
        let mut buf = Vec::new();
        genesis_params().encode(&mut buf);
        assert_eq!(&buf[24..40], &13_000_000_000u128.to_le_bytes());
    }

    #[test]
    fn encode_deterministic() {
        let mut a = Vec::new();
        genesis_params().encode(&mut a);
        let mut b = Vec::new();
        genesis_params().encode(&mut b);
        assert_eq!(a, b);
    }

    #[test]
    fn compute_hash_deterministic() {
        let root = [0xABu8; 32];
        let a = compute_genesis_state_hash(&root, genesis_params());
        let b = compute_genesis_state_hash(&root, genesis_params());
        assert_eq!(a, b);
    }

    #[test]
    fn compute_hash_detects_param_mutation() {
        let root = [0xABu8; 32];
        let h1 = compute_genesis_state_hash(&root, genesis_params());

        let mut mutated = genesis_params().clone();
        mutated.d0 = 325_000_001;
        let h2 = compute_genesis_state_hash(&root, &mutated);
        assert_ne!(h1, h2);
    }

    #[test]
    fn compute_hash_detects_state_root_mutation() {
        let r1 = [0xABu8; 32];
        let r2 = [0xCDu8; 32];
        let h1 = compute_genesis_state_hash(&r1, genesis_params());
        let h2 = compute_genesis_state_hash(&r2, genesis_params());
        assert_ne!(h1, h2);
    }

    #[test]
    fn encode_detects_field_mutations() {
        let mut orig = Vec::new();
        genesis_params().encode(&mut orig);

        let mutations: Vec<fn(&mut ProtocolParams)> = vec![
            |p| p.d0 += 1,
            |p| p.tau2_windows += 1,
            |p| p.emission_moneta += 1,
            |p| p.target_zero[0] = 0xFF,
            |p| p.confirmation_quorum_num = 68,
            |p| p.confirmation_quorum_den = 101,
            |p| p.participation_dead_zone_low = 86,
            |p| p.participation_dead_zone_high = 96,
            |p| p.selection_interval = 370,
            |p| p.candidate_expiry_windows += 1,
            |p| p.adaptive_vdf_multiplier = 101,
            |p| p.pruning_idle_windows += 1,
            |p| p.bootstrap_account_pubkey[0] = 0xFF,
            |p| p.bootstrap_node_pubkey[0] = 0xFF,
            |p| p.genesis_content_app_id[0] = 0xFF,
            |p| p.genesis_content_data_hash[0] = 0xFF,
        ];

        for (i, m) in mutations.iter().enumerate() {
            let mut mutated = genesis_params().clone();
            m(&mut mutated);
            let mut buf = Vec::new();
            mutated.encode(&mut buf);
            assert_ne!(orig, buf, "mutation {} did not affect encoding", i);
        }
    }

    #[test]
    fn reserved_m0_enforced_zeros_in_default() {
        assert_eq!(genesis_params().reserved_m0, [0u8; 8]);
    }

    // Pre-mainnet статус: возвращает false (placeholder fields).
    // После Genesis ceremony — bootstrap_keypairs_finalized() становится PASS,
    // и этот тест меняется на assert_eq!(true).
    #[test]
    fn is_genesis_bootstrap_finalized_post_ceremony_returns_true() {
        // Genesis Ceremony 2026-05-02: Moscow operator pubkeys + target_zero + content hash
        // финализированы. is_genesis_bootstrap_finalized() должен возвращать true.
        let p = genesis_params();
        assert!(
            is_genesis_bootstrap_finalized(p),
            "Genesis ceremony завершена 2026-05-02 — все 4 поля должны быть non-zero"
        );
    }

    #[test]
    fn is_genesis_bootstrap_finalized_detects_partial_finalization() {
        // Симулируем pre-ceremony state (все 4 поля placeholder zeros) и
        // постепенно финализируем — проверяем all-or-nothing semantic.
        let mut p = genesis_params().clone();
        p.bootstrap_account_pubkey = [0u8; PUBLIC_KEY_SIZE];
        p.bootstrap_node_pubkey = [0u8; PUBLIC_KEY_SIZE];
        p.target_zero = [0u8; 32];
        p.genesis_content_data_hash = [0u8; 32];
        assert!(!is_genesis_bootstrap_finalized(&p), "all zeros → not finalized");

        p.bootstrap_account_pubkey = [0xAB; PUBLIC_KEY_SIZE];
        assert!(!is_genesis_bootstrap_finalized(&p));

        p.bootstrap_node_pubkey = [0xCD; PUBLIC_KEY_SIZE];
        assert!(!is_genesis_bootstrap_finalized(&p));

        p.target_zero = [0xEF; 32];
        assert!(!is_genesis_bootstrap_finalized(&p));

        p.genesis_content_data_hash = [0x42; 32];
        assert!(is_genesis_bootstrap_finalized(&p), "all 4 non-zero → finalized");
    }

    #[test]
    fn bootstrap_keypairs_finalized() {
        // Post-ceremony: проверяем что bootstrap_account_pubkey/bootstrap_node_pubkey
        // не placeholder (содержат реальные байты Moscow operator).
        let p = genesis_params();
        assert!(is_genesis_bootstrap_finalized(p));
        assert_ne!(p.bootstrap_account_pubkey[..16], [0u8; 16]);
        assert_ne!(p.bootstrap_node_pubkey[..16], [0u8; 16]);
        assert_ne!(p.target_zero, [0u8; 32]);
        assert_ne!(p.genesis_content_data_hash, [0u8; 32]);
    }
}
