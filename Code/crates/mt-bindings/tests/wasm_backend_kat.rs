//! Cross-backend KAT: pure-Rust ml-dsa (веб/wasm-бэкенд) выдаёт тот же account_id,
//! что нативный OpenSSL (baked KAT 9f199584…131a21). Ловит расхождение бэкендов.
#![cfg(not(target_arch = "wasm32"))]

use ml_dsa::{Keypair, MlDsa65, SigningKey, B32};
use mt_mnemonic::{entropy_to_mnemonic, mldsa_seed_for_role, mnemonic_to_master_seed};
use sha2::{Digest, Sha256};

#[test]
fn ml_dsa_wasm_backend_matches_openssl_kat() {
    let mnemonic = entropy_to_mnemonic(&[0u8; 32]);
    let master = mnemonic_to_master_seed(&mnemonic).unwrap();
    let acc_seed = mldsa_seed_for_role(&master, mt_codec::domain::ACCOUNT_KEY);
    let seed = B32::try_from(&acc_seed[..]).unwrap();
    let sk = SigningKey::<MlDsa65>::from_seed(&seed);
    let pk = sk.verifying_key().encode();
    let mut h = Sha256::new();
    h.update(mt_codec::domain::ACCOUNT);
    h.update([0u8]);
    h.update([0x01u8, 0x00u8]);
    h.update(pk.as_slice());
    let id = hex::encode(h.finalize());
    assert_eq!(
        id,
        "9f199584ed120b987b617ba5bff829e176f23e5465dd70cfac5c141dfb131a21"
    );
}
