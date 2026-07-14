#ifndef MONTANA_FFI_H
#define MONTANA_FFI_H

/* Сгенерировано cbindgen из mt-bindings (network.rs/mdns.rs). Не редактировать вручную — SSOT в Rust-коде. Регенерация: cbindgen --config cbindgen.toml crates/mt-bindings -o crates/mt-bindings/include/montana_ffi.h */

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define ABI_VERSION 1

#define MT_MASTER_SEED_LEN 64

#define MT_MLDSA_SEED_LEN 32

#define MT_MLDSA_PUBKEY_SIZE 1952

#define MT_MLDSA_SECKEY_SIZE 4032

#define MT_MLDSA_SIG_SIZE 3309

#define MT_ACCOUNT_ID_LEN 32

#define MT_HISTORY_KEY_LEN 32

#define MT_MAX_MNEMONIC_BYTES 512

#define MT_MLKEM_SEED_LEN 64

#define MT_MLKEM_PUBKEY_SIZE 1184

#define MT_MLKEM_SECKEY_SIZE 2400

#define MT_MLKEM_CT_SIZE 1088

#define MT_MLKEM_SS_SIZE 32

#define MT_OK 0

#define MT_ERR_NULL_PTR -1

#define MT_ERR_INVALID_UTF8 -2

#define MT_ERR_MNEMONIC_WORD_COUNT -3

#define MT_ERR_MNEMONIC_UNKNOWN_WORD -4

#define MT_ERR_MNEMONIC_CHECKSUM -5

#define MT_ERR_KEYGEN_FAILED -6

#define MT_ERR_SIGN_FAILED -7

#define MT_ERR_VERIFY_FAILED -8

#define MT_ERR_BUFFER_TOO_SMALL -9

#define MT_ERR_KDF_FAILED -10

#define MT_ERR_ADDRESS_INVALID -10

#define MT_ERR_KEM_FAILED -11

#define MT_ERR_REPLAY -12

#define MT_ERR_PANIC -100

/**
 * Канонический suite_id для ML-DSA-65 account keypair (spec §Suite registry).
 */
#define MT_SUITE_MLDSA65 1

/**
 * Opaque-хэндл клиента (живое QUIC-соединение к почтальону/курьеру).
 */
typedef struct MtClient MtClient;

/**
 * Opaque-хэндл mDNS-демона (держит регистрацию сервиса живой).
 */
typedef struct MtMdns MtMdns;

/**
 * Opaque-хэндл живого почтальона для FFI. Держит адрес, задачу цикла (для остановки)
 * и MuqState (маршруты курьера).
 */
typedef struct MtPostman MtPostman;

/**
 * Анонсировать свой почтальон в локальной сети на порту `port`. Возвращает хэндл
 * (демон держит анонс живым) или null. `instance` — C-string имя экземпляра.
 *
 * # Safety
 * `instance` — валидный C-string или null.
 */
MtMdns *mt_mdns_advertise(uint16_t port,
                          const char *instance);

/**
 * Найти узлы Montana в локальной сети за `timeout_ms`. Пишет найденные адреса в `out`
 * как "ip:port\n"-разделённый ASCII (ёмкость `out_cap`, null-terminated), возвращает
 * число найденных узлов (0 если никого / ошибка).
 *
 * # Safety
 * `out` — буфер ≥ `out_cap` байт или null.
 */
uintptr_t mt_mdns_browse(uint32_t timeout_ms,
                         uint8_t *out,
                         uintptr_t out_cap);

/**
 * Остановить анонс и освободить хэндл.
 *
 * # Safety
 * `h` — хэндл из `mt_mdns_advertise` либо null; не использовать повторно.
 */
void mt_mdns_stop(MtMdns *h);

/**
 * Запустить почтальон на `bind` (например "0.0.0.0:0"). При успехе записывает реальный
 * адрес (host:port) в `out_addr` (буфер ёмкости `out_cap`, null-terminated) и возвращает
 * opaque-хэндл; при ошибке — null. Хэндл освобождается `mt_postman_stop`.
 *
 * # Safety
 * `bind` — валидный C-string; `out_addr` — буфер ≥ `out_cap` байт или null.
 */
MtPostman *mt_postman_start(const char *bind,
                            char *out_addr,
                            uintptr_t out_cap);

/**
 * Остановить почтальон и освободить хэндл. После вызова `h` невалиден.
 *
 * # Safety
 * `h` — хэндл из `mt_postman_start` либо null; не использовать повторно.
 */
void mt_postman_stop(MtPostman *h);

/**
 * Порт живого почтальона (0 при null-хэндле) — для диагностики/теста.
 *
 * # Safety
 * `h` — валидный хэндл из `mt_postman_start` либо null.
 */
uint16_t mt_postman_port(const MtPostman *h);

/**
 * Добавить маршрут курьера: `overlay` (32 B оверлей-адрес хоста) → физический `target`
 * (host:port). Возвращает 0 при успехе, -1 при ошибке аргументов. Модель relay Этапа 3.
 *
 * # Safety
 * `h` — валидный хэндл; `overlay` — указатель на 32 байта; `target` — C-string.
 */
int32_t mt_postman_add_route(const MtPostman *h,
                             const uint8_t *overlay,
                             const char *target);

/**
 * Записать ML-KEM pubkey почтальона (1184 B) в `out` (ёмкость `out_cap`). Клиент
 * использует его для sealed-депозита. Возвращает записанные байты (0 при ошибке).
 *
 * # Safety
 * `h` — валидный хэндл; `out` — буфер ≥ `out_cap` байт.
 */
uintptr_t mt_postman_kem_pubkey(const MtPostman *h,
                                uint8_t *out,
                                uintptr_t out_cap);

/**
 * Подключиться к почтальону по адресу `addr` (host:port). Возвращает хэндл или null.
 *
 * # Safety
 * `addr` — валидный C-string.
 */
MtClient *mt_client_connect(const char *addr);

/**
 * Зарегистрировать очередь на хосте `host_overlay` (32 B) через курьер, к которому
 * подключён клиент. `host_kem` — 1184 B pubkey хоста; `queue` — сериализованный Queue
 * (`queue_len` байт). Возвращает 0 при успехе, -1 при ошибке.
 *
 * # Safety
 * `client` — валидный хэндл; `host_overlay` → 32 B; `host_kem` → 1184 B; `queue` → `queue_len` B.
 */
int32_t mt_client_register(const MtClient *client,
                           const uint8_t *host_overlay,
                           const uint8_t *host_kem,
                           const uint8_t *queue,
                           uintptr_t queue_len);

/**
 * Освободить хэндл клиента (закрывает соединение).
 *
 * # Safety
 * `c` — хэндл из `mt_client_connect` либо null; не использовать повторно.
 */
void mt_client_free(MtClient *c);

/**
 * Отправить сообщение `msg` в очередь `send_id` на хосте `host_overlay` через курьер
 * (двуххоп-депозит, sealed к ML-KEM хоста). Собирает HostDeposit+подпись+seal внутри.
 * Одиночный shard. Возвращает 0 при успехе, -1 при ошибке.
 *
 * # Safety
 * `client` валиден; `host_overlay`→32; `host_kem`→1184; `send_id`→32; `send_sk`→4032;
 * `msg_id`→16; `msg`→`msg_len`.
 */
int32_t mt_client_send(const MtClient *client,
                       const uint8_t *host_overlay,
                       const uint8_t *host_kem,
                       const uint8_t *send_id,
                       const uint8_t *send_sk,
                       const uint8_t *msg_id,
                       const uint8_t *msg,
                       uintptr_t msg_len);

/**
 * Забрать одно сообщение из очереди `recv_id` на хосте `host_overlay` через курьер.
 * Пишет ct первого конверта в `out` (ёмкость `out_cap`), возвращает его длину;
 * 0 если очередь пуста или ошибка.
 *
 * # Safety
 * `client` валиден; `host_overlay`→32; `host_kem`→1184; `recv_id`→32; `recv_sk`→4032;
 * `out`→`out_cap`.
 */
uintptr_t mt_client_recv(const MtClient *client,
                         const uint8_t *host_overlay,
                         const uint8_t *host_kem,
                         const uint8_t *recv_id,
                         const uint8_t *recv_sk,
                         uint8_t *out,
                         uintptr_t out_cap);

uint32_t mt_abi_version(void);

int mt_mnemonic_to_master_seed(const char *mnemonic_utf8, uint8_t *out_master_seed);

int mt_mnemonic_to_entropy(const char *mnemonic_utf8, uint8_t *out_entropy);

int mt_mldsa_seed_for_role(const uint8_t *master_seed,
                           const uint8_t *role,
                           uintptr_t role_len,
                           uint8_t *out_seed);

int mt_mldsa_keypair_from_seed(const uint8_t *seed, uint8_t *out_pubkey, uint8_t *out_seckey);

int mt_derive_account_id(uint16_t suite_id, const uint8_t *pubkey, uint8_t *out_account_id);

/**
 * 24-словная мнемоника → ML-DSA-65 account keypair + canonical account_id (suite 0x0001).
 */
int mt_account_from_mnemonic(const char *mnemonic_utf8,
                             uint8_t *out_pubkey,
                             uint8_t *out_seckey,
                             uint8_t *out_account_id);

/**
 * account_id (32 байта) → текстовый адрес "mt…" (Base58Check), записывает в out + NUL.
 */
int mt_account_id_to_address(const uint8_t *account_id,
                             uint8_t *out,
                             uintptr_t out_capacity,
                             uintptr_t *out_len);

/**
 * Текстовый адрес "mt…" → account_id (32 байта). Проверяет контрольную сумму.
 */
int mt_address_to_account_id(const char *address_utf8,
                             uint8_t *out_account_id);

int mt_sign(const uint8_t *seckey, const uint8_t *msg, uintptr_t msg_len, uint8_t *out_sig);

int mt_verify(const uint8_t *pubkey, const uint8_t *msg, uintptr_t msg_len, const uint8_t *sig);

/**
 * 32 байта энтропии → 24-словная мнемоника UTF-8.
 *
 * `out_mnemonic_utf8` — буфер ≥ out_capacity байт; функция запишет нуль-терминированную строку.
 * `out_len` — фактически записанные байты (без терминатора). При недостатке буфера вернёт MT_ERR_BUFFER_TOO_SMALL.
 */
int mt_entropy_to_mnemonic(const uint8_t *entropy,
                           uint8_t *out_mnemonic_utf8,
                           uintptr_t out_capacity,
                           uintptr_t *out_len);

/**
 * HKDF-Expand(master_seed, role, 64) -> ML-KEM-768 seed (d‖z). Этап 1: app_kem_key.
 */
int mt_mlkem_seed_for_role(const uint8_t *master_seed,
                           const uint8_t *role,
                           uintptr_t role_len,
                           uint8_t *out_seed);

/**
 * ML-KEM-768 KeyGen из 64-байтного сида (FIPS 203, deterministic). pk 1184 / sk 2400.
 */
int mt_mlkem_keypair_from_seed(const uint8_t *seed, uint8_t *out_pubkey, uint8_t *out_seckey);

/**
 * ML-KEM-768 Encapsulate (FIPS 203 §6.2). pk 1184 -> ct 1088 / ss 32.
 */
int mt_mlkem_encaps(const uint8_t *pubkey, uint8_t *out_ct, uint8_t *out_ss);

/**
 * ML-KEM-768 Decapsulate (FIPS 203 §6.3, implicit-rejection). sk 2400, ct 1088 -> ss 32.
 */
int mt_mlkem_decaps(const uint8_t *seckey, const uint8_t *ct, uint8_t *out_ss);

/**
 * 24-словная мнемоника -> app_kem_key (ML-KEM-768) через роль "mt-app-encryption-key". pk 1184 / sk 2400.
 */
int mt_app_kem_from_mnemonic(const char *mnemonic_utf8,
                             uint8_t *out_pubkey,
                             uint8_t *out_seckey);

/**
 * history_key = HKDF-SHA-256(salt=0×32, ikm=entropy_32, info="mt-history-key", 32) — Этап 10 мессенджера.
 * `entropy` — 32 байта; `out` — 32 байта. SSOT для history_key всех клиентов.
 */
int mt_history_key(const uint8_t *entropy,
                   uint8_t *out);

/**
 * Освободить буфер, выданный функциями mt_e2e_*.
 *
 * # Safety
 * `ptr`/`len` — ровно то, что вернула mt_e2e_* через out-параметры; вызывать однократно.
 */
void mt_e2e_free(uint8_t *ptr,
                 uintptr_t len);

/**
 * RatchetEncrypt через непрозрачный блоб сессии. Возвращает новый блоб сессии + сообщение.
 *
 * # Safety
 * Все указатели валидны на свою длину; `rng_seed` — 64 байта; out-указатели ненулевые.
 */
int mt_e2e_encrypt(const uint8_t *session,
                   uintptr_t session_len,
                   const uint8_t *pt,
                   uintptr_t pt_len,
                   const uint8_t *rng_seed,
                   uint8_t **out_session,
                   uintptr_t *out_session_len,
                   uint8_t **out_msg,
                   uintptr_t *out_msg_len);

/**
 * RatchetDecrypt через непрозрачный блоб сессии. Возвращает новый блоб + открытый текст.
 *
 * # Safety
 * Все указатели валидны на свою длину; out-указатели ненулевые.
 */
int mt_e2e_decrypt(const uint8_t *session,
                   uintptr_t session_len,
                   const uint8_t *msg,
                   uintptr_t msg_len,
                   uint8_t **out_session,
                   uintptr_t *out_session_len,
                   uint8_t **out_pt,
                   uintptr_t *out_pt_len);

/**
 * Сторона Алисы: рукопожатие + инициализация сессии. Возвращает InitialHandshake
 * + блоб сессии инициатора. `account_seed` — 32 байта (сид ML-DSA личности).
 *
 * # Safety
 * Все ключевые указатели валидны на размеры спеки; opk_* читаются лишь при opk_flag=1.
 */
int mt_e2e_build_handshake(const uint8_t *alice_account_pub,
                           const uint8_t *account_seed,
                           const uint8_t *bob_account_pub,
                           const uint8_t *bob_app_kem_pub,
                           const uint8_t *bob_spk_pub,
                           uint32_t spk_id,
                           uint8_t opk_flag,
                           uint32_t opk_id,
                           const uint8_t *bob_opk_pub,
                           const uint8_t *eph_seed,
                           uint64_t send_time,
                           uint8_t **out_hs,
                           uintptr_t *out_hs_len,
                           uint8_t **out_session,
                           uintptr_t *out_session_len);

/**
 * Сторона Боба: обработка рукопожатия + инициализация сессии получателя.
 *
 * # Safety
 * Все ключевые указатели валидны на размеры спеки; opk_* читаются лишь при opk_flag=1.
 */
int mt_e2e_process_handshake(const uint8_t *hs,
                             uintptr_t hs_len,
                             const uint8_t *bob_account_id,
                             const uint8_t *bob_app_kem_pub,
                             const uint8_t *bob_app_kem_sk,
                             const uint8_t *bob_spk_pub,
                             const uint8_t *bob_spk_sk,
                             uint8_t opk_flag,
                             const uint8_t *bob_opk_pub,
                             const uint8_t *bob_opk_sk,
                             uint64_t now,
                             uint64_t accept_skew,
                             uint8_t **out_session,
                             uintptr_t *out_session_len);

/**
 * Запечатать медиа-блоб: sealed_blob = nonce || Seal(blob_key, nonce, input, AD=mt-media).
 * out — owned-буфер (освободить mt_e2e_free). `input` — уже финальный (паддинг pad_len до вызова).
 *
 * # Safety
 * blob_key — 32 байта, nonce — 12 байт, input — input_len байт; out_ptr/out_len ненулевые.
 */
int mt_e2e_seal_blob(const uint8_t *blob_key,
                     const uint8_t *nonce,
                     const uint8_t *input,
                     uintptr_t input_len,
                     uint8_t **out_ptr,
                     uintptr_t *out_len);

/**
 * blob_id = SHA-256(sealed_blob) -> out32 (32 байта).
 *
 * # Safety
 * sealed_blob — len байт; out32 — 32 байта.
 */
int mt_e2e_blob_id(const uint8_t *sealed_blob, uintptr_t len, uint8_t *out32);

/**
 * Расшифровать блоб -> padded plaintext (owned; усечь до size вызывающему). Ошибка -> MT_ERR_KEM_FAILED.
 *
 * # Safety
 * blob_key — 32 байта; sealed_blob — len байт; out_ptr/out_len ненулевые.
 */
int mt_e2e_open_blob(const uint8_t *blob_key,
                     const uint8_t *sealed_blob,
                     uintptr_t len,
                     uint8_t **out_ptr,
                     uintptr_t *out_len);

/**
 * pad_len(n) — целевой размер после паддинга (скрытие размера).
 */
uintptr_t mt_e2e_pad_len(uintptr_t n);

/**
 * safety_number(id_A, id_B) → 60 ASCII-цифр (Этап 8). Оба указателя — 32 байта account_id;
 * выход — owned-буфер (60 байт), освобождать mt_e2e_free.
 *
 * # Safety
 * `id_a`/`id_b` валидны на 32 байта; out-указатели ненулевые.
 */
int mt_e2e_safety_number(const uint8_t *id_a,
                         const uint8_t *id_b,
                         uint8_t **out_ptr,
                         uintptr_t *out_len);

/**
 * party_code(account_id) → 30 ASCII-цифр (Этап 8). Указатель — 32 байта; выход owned.
 *
 * # Safety
 * `id` валиден на 32 байта; out-указатели ненулевые.
 */
int mt_e2e_party_code(const uint8_t *id,
                      uint8_t **out_ptr,
                      uintptr_t *out_len);

/**
 * call_key/sframe_key (Этап 13, PQ-медиа-слой звонка). `call_seed` — 32 байта (из E2E-сигнала);
 * out — 64 байта: call_key(32) ‖ sframe_key(32).
 *
 * # Safety
 * `call_seed` валиден на 32 байта; `out` — на 64 байта.
 */
int mt_e2e_call_key(const uint8_t *call_seed,
                    uint8_t *out);

#endif  /* MONTANA_FFI_H */
