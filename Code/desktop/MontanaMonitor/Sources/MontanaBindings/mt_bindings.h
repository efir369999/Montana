/* mt_bindings.h — C ABI Montana Protocol для iOS / macOS / Android.
 *
 * SSOT: единственный источник истины крипты/протокола Montana.
 * Реализация — Rust crates (mt-mnemonic, mt-crypto, mt-state, mt-account).
 */

#ifndef MT_BINDINGS_H
#define MT_BINDINGS_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define MT_OK                              0
#define MT_ERR_NULL_PTR                   -1
#define MT_ERR_INVALID_UTF8               -2
#define MT_ERR_MNEMONIC_WORD_COUNT        -3
#define MT_ERR_MNEMONIC_UNKNOWN_WORD      -4
#define MT_ERR_MNEMONIC_CHECKSUM          -5
#define MT_ERR_KEYGEN_FAILED              -6
#define MT_ERR_SIGN_FAILED                -7
#define MT_ERR_VERIFY_FAILED              -8
#define MT_ERR_BUFFER_TOO_SMALL           -9
#define MT_ERR_PANIC                    -100

#define MT_MASTER_SEED_LEN          64
#define MT_MLDSA_SEED_LEN           32
#define MT_MLDSA_PUBKEY_SIZE      1952
#define MT_MLDSA_SECKEY_SIZE      4032
#define MT_MLDSA_SIG_SIZE         3309
#define MT_ACCOUNT_ID_LEN           32
#define MT_SUITE_MLDSA65        0x0001

uint32_t mt_abi_version(void);
int mt_mnemonic_to_master_seed(const char *mnemonic_utf8, uint8_t *out_master_seed);
int mt_mldsa_seed_for_role(const uint8_t *master_seed, const uint8_t *role, size_t role_len, uint8_t *out_seed);
int mt_mldsa_keypair_from_seed(const uint8_t *seed, uint8_t *out_pubkey, uint8_t *out_seckey);
int mt_derive_account_id(uint16_t suite_id, const uint8_t *pubkey, uint8_t *out_account_id);
int mt_account_from_mnemonic(const char *mnemonic_utf8, uint8_t *out_pubkey, uint8_t *out_seckey, uint8_t *out_account_id);
int mt_sign(const uint8_t *seckey, const uint8_t *msg, size_t msg_len, uint8_t *out_sig);
int mt_verify(const uint8_t *pubkey, const uint8_t *msg, size_t msg_len, const uint8_t *sig);

/* 32 байта энтропии → 24-словная UTF-8 строка в out_mnemonic_utf8 (нуль-терминированная). */
int mt_entropy_to_mnemonic(const uint8_t *entropy, uint8_t *out_mnemonic_utf8, size_t out_capacity, size_t *out_len);

#ifdef __cplusplus
}
#endif

#endif
