// spec, раздел "Ключи → Мнемоника и seed" + "Криптографическая реализация → Primitive layer"

mod bit_packing;
mod hkdf;
mod hmac;
mod mnemonic;
mod pbkdf2;
mod wordlist;

pub use hkdf::hkdf_expand;
pub use hmac::hmac_sha256;
pub use mnemonic::{
    ed25519_seed_for_role, entropy_to_mnemonic, mldsa_seed_for_role, mlkem_seed_for_role,
    mnemonic_to_master_seed, MnemonicError, ED25519_SEED_LEN, KDF_ITER, MASTER_SEED_LEN,
    MLDSA_SEED_LEN, MLKEM_SEED_LEN, MNEMONIC_WORD_COUNT,
};
pub use pbkdf2::pbkdf2_hmac_sha256;
pub use wordlist::{word_index, wordlist, WORDLIST_FINGERPRINT, WORDLIST_SIZE};
