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
#define MT_ERR_ADDRESS_INVALID           -13
#define MT_ERR_KEM_FAILED                -11
#define MT_ERR_REPLAY                    -12
#define MT_ERR_PANIC                    -100

#define MT_MASTER_SEED_LEN          64
#define MT_MLDSA_SEED_LEN           32
#define MT_MLDSA_PUBKEY_SIZE      1952
#define MT_MLDSA_SECKEY_SIZE      4032
#define MT_MLDSA_SIG_SIZE         3309
#define MT_ACCOUNT_ID_LEN           32
#define MT_SUITE_MLDSA65        0x0001
#define MT_MLKEM_SEED_LEN           64
#define MT_MLKEM_PUBKEY_SIZE      1184
#define MT_MLKEM_SECKEY_SIZE      2400
#define MT_MLKEM_CT_SIZE          1088
#define MT_MLKEM_SS_SIZE            32

uint32_t mt_abi_version(void);
int mt_mnemonic_to_master_seed(const char *mnemonic_utf8, uint8_t *out_master_seed);
int mt_mnemonic_to_entropy(const char *mnemonic_utf8, uint8_t *out_entropy);
int mt_mldsa_seed_for_role(const uint8_t *master_seed, const uint8_t *role, size_t role_len, uint8_t *out_seed);
int mt_mldsa_keypair_from_seed(const uint8_t *seed, uint8_t *out_pubkey, uint8_t *out_seckey);
int mt_derive_account_id(uint16_t suite_id, const uint8_t *pubkey, uint8_t *out_account_id);
int mt_account_from_mnemonic(const char *mnemonic_utf8, uint8_t *out_pubkey, uint8_t *out_seckey, uint8_t *out_account_id);
int mt_sign(const uint8_t *seckey, const uint8_t *msg, size_t msg_len, uint8_t *out_sig);
int mt_verify(const uint8_t *pubkey, const uint8_t *msg, size_t msg_len, const uint8_t *sig);

/* 32 байта энтропии → 24-словная UTF-8 строка в out_mnemonic_utf8 (нуль-терминированная). */
int mt_entropy_to_mnemonic(const uint8_t *entropy, uint8_t *out_mnemonic_utf8, size_t out_capacity, size_t *out_len);

/* account_id (32 bytes) -> textual address "mt..." (Base58Check), NUL-terminated; out_capacity >= 64. */
int mt_account_id_to_address(const uint8_t *account_id, uint8_t *out, size_t out_capacity, size_t *out_len);

/* textual address "mt..." -> account_id (32 bytes). Verifies the checksum. */
int mt_address_to_account_id(const char *address_utf8, uint8_t *out_account_id);

/* ML-KEM-768 (FIPS 203) — Этап 1 app_kem_key + Этапы 4-7 обмен ключами. */
int mt_mlkem_seed_for_role(const uint8_t *master_seed, const uint8_t *role, size_t role_len, uint8_t *out_seed);
int mt_mlkem_keypair_from_seed(const uint8_t *seed, uint8_t *out_pubkey, uint8_t *out_seckey);
int mt_mlkem_encaps(const uint8_t *pubkey, uint8_t *out_ct, uint8_t *out_ss);
int mt_mlkem_decaps(const uint8_t *seckey, const uint8_t *ct, uint8_t *out_ss);
/* 24 слова -> app_kem_key (ML-KEM-768, роль "mt-app-encryption-key"). */
int mt_app_kem_from_mnemonic(const char *mnemonic_utf8, uint8_t *out_pubkey, uint8_t *out_seckey);
int mt_history_key(const uint8_t *entropy, uint8_t *out);

/* Движок E2E (mt-messenger-e2e), Этап 6 хот-путь. Выходы — owned-буферы,
 * освобождать mt_e2e_free(ptr,len). session — непрозрачный блоб SessionState. */
void mt_e2e_free(uint8_t *ptr, size_t len);
int mt_e2e_encrypt(const uint8_t *session, size_t session_len,
                   const uint8_t *pt, size_t pt_len, const uint8_t *rng_seed,
                   uint8_t **out_session, size_t *out_session_len,
                   uint8_t **out_msg, size_t *out_msg_len);
int mt_e2e_decrypt(const uint8_t *session, size_t session_len,
                   const uint8_t *msg, size_t msg_len,
                   uint8_t **out_session, size_t *out_session_len,
                   uint8_t **out_pt, size_t *out_pt_len);
int mt_e2e_build_handshake(const uint8_t *alice_account_pub, const uint8_t *account_seed,
                           const uint8_t *bob_account_pub, const uint8_t *bob_app_kem_pub,
                           const uint8_t *bob_spk_pub, uint32_t spk_id, uint8_t opk_flag,
                           uint32_t opk_id, const uint8_t *bob_opk_pub, const uint8_t *eph_seed,
                           uint64_t send_time, uint8_t **out_hs, size_t *out_hs_len,
                           uint8_t **out_session, size_t *out_session_len);
int mt_e2e_process_handshake(const uint8_t *hs, size_t hs_len, const uint8_t *bob_account_id,
                             const uint8_t *bob_app_kem_pub, const uint8_t *bob_app_kem_sk,
                             const uint8_t *bob_spk_pub, const uint8_t *bob_spk_sk, uint8_t opk_flag,
                             const uint8_t *bob_opk_pub, const uint8_t *bob_opk_sk, uint64_t now,
                             uint64_t accept_skew, uint8_t **out_session, size_t *out_session_len);
int mt_e2e_seal_blob(const uint8_t *blob_key, const uint8_t *nonce, const uint8_t *input, size_t input_len, uint8_t **out_ptr, size_t *out_len);
int mt_e2e_blob_id(const uint8_t *sealed_blob, size_t len, uint8_t *out32);
int mt_e2e_open_blob(const uint8_t *blob_key, const uint8_t *sealed_blob, size_t len, uint8_t **out_ptr, size_t *out_len);
size_t mt_e2e_pad_len(size_t n);
int mt_e2e_safety_number(const uint8_t *id_a, const uint8_t *id_b, uint8_t **out_ptr, size_t *out_len);
int mt_e2e_party_code(const uint8_t *id, uint8_t **out_ptr, size_t *out_len);
int mt_e2e_call_key(const uint8_t *call_seed, uint8_t *out);   /* out = call_key(32) || sframe_key(32) */

/* Noise_PQ XX handshake (spec s.3 section 5.0). Opaque state handles keep secret keys in Rust.
   Wire sizes: msg1=2272, msg2=6349, msg3=5261. Session output: sk_i_to_r[32], sk_r_to_i[32],
   channel_hash[32]. node_id_seed = 32-byte ephemeral ML-DSA node identity seed. */
int mt_noise_initiator_msg1(const uint8_t *responder_kem_pk, const uint8_t *node_id_seed, uint8_t *out_msg1, void **out_state);
int mt_noise_initiator_msg2(void *state, const uint8_t *msg2, void **out_state2);
int mt_noise_initiator_msg3(void *state2, uint8_t *out_msg3, uint8_t *out_sk_i_to_r, uint8_t *out_sk_r_to_i, uint8_t *out_channel_hash);
int mt_noise_responder_msg1(const uint8_t *responder_kem_sk, const uint8_t *node_id_seed, const uint8_t *msg1, uint8_t *out_msg2, void **out_state);
int mt_noise_responder_msg3(void *state, const uint8_t *msg3, uint8_t *out_sk_i_to_r, uint8_t *out_sk_r_to_i, uint8_t *out_channel_hash);
void mt_noise_state_free_initiator1(void *state);
void mt_noise_state_free_initiator2(void *state);
void mt_noise_state_free_responder(void *state);

#ifdef __cplusplus
}
#endif

#endif
