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

#define MT_ERR_ADDRESS_INVALID -13

#define MT_ERR_KEM_FAILED -11

#define MT_ERR_REPLAY -12

#define MT_ERR_PANIC -100

/**
 * Канонический suite_id для ML-DSA-65 account keypair (spec §Suite registry).
 */
#define MT_SUITE_MLDSA65 1

/**
 * Opaque-хэндл клиента (живое QUIC-соединение к почтальону/курьеру).
 * `pending` — буфер хвоста пакетной выборки: `subscribe_via_courier` — уничтожающий
 * batch-drain (host отдаёт и дропает ВСЕ элементы очереди разом), поэтому FFI обязан
 * сохранить весь batch и выдавать по одному, иначе items[1..] теряются (§206 «буфер
 * никогда не теряет сообщение»).
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
 * Opaque-хэндл клиента Mainline DHT (рандеву).
 */
typedef struct RvDht RvDht;

/**
 * Opaque-реестр account_id<->wake_handle (телефон-почтальон).
 */
typedef struct WakeRegistry WakeRegistry;

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
size_t mt_mdns_browse(uint32_t timeout_ms,
                         uint8_t *out,
                         size_t out_cap);

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
                            size_t out_cap);

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
size_t mt_postman_kem_pubkey(const MtPostman *h,
                                uint8_t *out,
                                size_t out_cap);

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
                           size_t queue_len);

/**
 * Register a queue DIRECTLY on the connected node (TAG_QUEUE_REGISTER, no courier). Self-host uses
 * this against its own node (loopback): the queue registers locally without any self-connection.
 * `queue` — serialized Queue (`queue_len` bytes). Returns 0 on success, -1 on error.
 *
 * # Safety
 * `client` — handle from `mt_client_connect`; `queue` → `queue_len` bytes.
 */
int32_t mt_client_register_direct(const MtClient *client,
                                  const uint8_t *queue,
                                  size_t queue_len);

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
                       size_t msg_len);

/**
 * Забрать одно сообщение из очереди `recv_id` на хосте `host_overlay` через курьер.
 * Пишет ct первого конверта в `out` (ёмкость `out_cap`), возвращает его длину;
 * 0 если очередь пуста или ошибка.
 *
 * # Safety
 * `client` валиден; `host_overlay`→32; `host_kem`→1184; `recv_id`→32; `recv_sk`→4032;
 * `out`→`out_cap`.
 */
size_t mt_client_recv(const MtClient *client,
                         const uint8_t *host_overlay,
                         const uint8_t *host_kem,
                         const uint8_t *recv_id,
                         const uint8_t *recv_sk,
                         uint8_t *out,
                         size_t out_cap);

/**
 * Подтвердить приём (DEV-049(a) §593): хост дропает буфер очереди recv_id. 0 = успех.
 *
 * # Safety
 * `client` валиден; `host_overlay`→32; `host_kem`→1184; `recv_id`→32; `recv_sk`→4032.
 */
int32_t mt_client_ack(const MtClient *client,
                      const uint8_t *host_overlay,
                      const uint8_t *host_kem,
                      const uint8_t *recv_id,
                      const uint8_t *recv_sk);

/**
 * DEV-049(b): RS(k,n) multi-host отправка — дробит `msg` на `n` осколков и депонирует
 * по одному на каждый из `n` хостов. Хосты — конкатенированные массивы: `host_overlays`
 * (n*32), `host_kems` (n*1184). Возврат: число успешных депозитов (durability при >= k),
 * -1 при ошибке.
 *
 * # Safety
 * `client` валиден; `host_overlays`→`n*32`; `host_kems`→`n*1184`; `send_id`→32;
 * `send_sk`→4032; `msg_id`→16; `msg`→`msg_len`.
 */
int32_t mt_client_send_erasure(const MtClient *client,
                               const uint8_t *host_overlays,
                               const uint8_t *host_kems,
                               size_t n,
                               size_t k,
                               const uint8_t *send_id,
                               const uint8_t *send_sk,
                               const uint8_t *msg_id,
                               const uint8_t *msg,
                               size_t msg_len);

/**
 * DEV-049(b): RS(k,n) multi-host выборка — собирает осколки с `n` хостов и реконструирует
 * из любых `k`. Пишет реконструированный ct в `out`; возврат — длина (0 если собрано < k /
 * ошибка; need > out_cap = буфер мал).
 *
 * # Safety
 * `client` валиден; `host_overlays`→`n*32`; `host_kems`→`n*1184`; `recv_id`→32;
 * `recv_sk`→4032; `out`→`out_cap`.
 */
size_t mt_client_recv_erasure(const MtClient *client,
                                 const uint8_t *host_overlays,
                                 const uint8_t *host_kems,
                                 size_t n,
                                 size_t k,
                                 const uint8_t *recv_id,
                                 const uint8_t *recv_sk,
                                 uint8_t *out,
                                 size_t out_cap);

/**
 * Тип deep-link montana://: 0 = bootstrap-payload (montana://b/...), 1 = wallet-адрес
 * (montana://mt...), -1 = ошибка разбора.
 *
 * # Safety
 * `link` — валидный C-string.
 */
int32_t mt_deeplink_kind(const char *link);

/**
 * Для montana://<mt-address>: пишет адрес кошелька (ASCII) в `out`, возвращает длину
 * (0 если не address / буфер мал / ошибка).
 *
 * # Safety
 * `link` — C-string; `out` — ≥ `out_cap` байт.
 */
size_t mt_deeplink_address(const char *link,
                              uint8_t *out,
                              size_t out_cap);

/**
 * Для montana://b/<payload>: декодирует QRBootstrap, пишет current_endpoint
 * (SSRF-фильтрован, "host:port" ASCII) в `out`; возвращает длину (0 если протух /
 * внутренний адрес / не bootstrap / ошибка).
 *
 * # Safety
 * `link` — C-string; `out` — ≥ `out_cap` байт.
 */
size_t mt_deeplink_bootstrap_endpoint(const char *link,
                                         uint64_t now_unix,
                                         uint8_t *out,
                                         size_t out_cap);

/**
 * Подключение к Mainline DHT (публичные bootstrap-ноды BitTorrent). Освобождается
 * `mt_rvdht_free`. null при ошибке.
 */
RvDht *mt_rvdht_client(void);

/**
 * # Safety
 * `dht` — указатель от `mt_rvdht_client` (не использованный после free) или null.
 */
void mt_rvdht_free(RvDht *dht);

/**
 * Резолвит рандеву-запись друга по `dk`(32)+`salt`(20) из DHT, пишет первый
 * глобально-маршрутизируемый endpoint (SSRF-фильтрован, "host:port") в `out`;
 * возвращает длину (0 если записи нет / протухла / только внутренние адреса / ошибка).
 *
 * # Safety
 * `dht` валиден; `dk` — ≥32 B; `salt` — ≥20 B; `friend_account_id` — ≥32 B или null
 * (null пропускает сверку §595 — не рекомендуется); `out` — ≥ `out_cap` байт.
 */
size_t mt_rvdht_resolve(const RvDht *dht,
                           const uint8_t *dk,
                           const uint8_t *salt,
                           const uint8_t *friend_account_id,
                           uint64_t now_unix,
                           uint8_t *out,
                           size_t out_cap);

uint32_t mt_abi_version(void);

int mt_mnemonic_to_master_seed(const char *mnemonic_utf8, uint8_t *out_master_seed);

int mt_mnemonic_to_entropy(const char *mnemonic_utf8, uint8_t *out_entropy);

int mt_mldsa_seed_for_role(const uint8_t *master_seed,
                           const uint8_t *role,
                           size_t role_len,
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
                             size_t out_capacity,
                             size_t *out_len);

/**
 * Текстовый адрес "mt…" → account_id (32 байта). Проверяет контрольную сумму.
 */
int mt_address_to_account_id(const char *address_utf8,
                             uint8_t *out_account_id);

int mt_sign(const uint8_t *seckey, const uint8_t *msg, size_t msg_len, uint8_t *out_sig);

int mt_verify(const uint8_t *pubkey, const uint8_t *msg, size_t msg_len, const uint8_t *sig);

/**
 * 32 байта энтропии → 24-словная мнемоника UTF-8.
 *
 * `out_mnemonic_utf8` — буфер ≥ out_capacity байт; функция запишет нуль-терминированную строку.
 * `out_len` — фактически записанные байты (без терминатора). При недостатке буфера вернёт MT_ERR_BUFFER_TOO_SMALL.
 */
int mt_entropy_to_mnemonic(const uint8_t *entropy,
                           uint8_t *out_mnemonic_utf8,
                           size_t out_capacity,
                           size_t *out_len);

/**
 * HKDF-Expand(master_seed, role, 64) -> ML-KEM-768 seed (d‖z). Этап 1: app_kem_key.
 */
int mt_mlkem_seed_for_role(const uint8_t *master_seed,
                           const uint8_t *role,
                           size_t role_len,
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
 * media_key = HKDF-SHA-256(0×32, entropy_32, "mt-media-key", 32) — s.2 Этап 1.
 * Отдельная ветвь сида для медиа at-rest (≠ history_key). `entropy`/`out` — 32 байта.
 */
int mt_media_key(const uint8_t *entropy,
                 uint8_t *out);

/**
 * Освободить буфер, выданный функциями mt_e2e_*.
 *
 * # Safety
 * `ptr`/`len` — ровно то, что вернула mt_e2e_* через out-параметры; вызывать однократно.
 */
void mt_e2e_free(uint8_t *ptr,
                 size_t len);

/**
 * RatchetEncrypt через непрозрачный блоб сессии. Возвращает новый блоб сессии + сообщение.
 *
 * # Safety
 * Все указатели валидны на свою длину; `rng_seed` — 64 байта; out-указатели ненулевые.
 */
int mt_e2e_encrypt(const uint8_t *session,
                   size_t session_len,
                   const uint8_t *pt,
                   size_t pt_len,
                   const uint8_t *rng_seed,
                   uint8_t **out_session,
                   size_t *out_session_len,
                   uint8_t **out_msg,
                   size_t *out_msg_len);

/**
 * RatchetDecrypt через непрозрачный блоб сессии. Возвращает новый блоб + открытый текст.
 *
 * # Safety
 * Все указатели валидны на свою длину; out-указатели ненулевые.
 */
int mt_e2e_decrypt(const uint8_t *session,
                   size_t session_len,
                   const uint8_t *msg,
                   size_t msg_len,
                   uint8_t **out_session,
                   size_t *out_session_len,
                   uint8_t **out_pt,
                   size_t *out_pt_len);

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
                           size_t *out_hs_len,
                           uint8_t **out_session,
                           size_t *out_session_len);

/**
 * Сторона Боба: обработка рукопожатия + инициализация сессии получателя.
 *
 * # Safety
 * Все ключевые указатели валидны на размеры спеки; opk_* читаются лишь при opk_flag=1.
 */
int mt_e2e_process_handshake(const uint8_t *hs,
                             size_t hs_len,
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
                             size_t *out_session_len);

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
                     size_t input_len,
                     uint8_t **out_ptr,
                     size_t *out_len);

/**
 * blob_id = SHA-256(sealed_blob) -> out32 (32 байта).
 *
 * # Safety
 * sealed_blob — len байт; out32 — 32 байта.
 */
int mt_e2e_blob_id(const uint8_t *sealed_blob, size_t len, uint8_t *out32);

/**
 * Расшифровать блоб -> padded plaintext (owned; усечь до size вызывающему). Ошибка -> MT_ERR_KEM_FAILED.
 *
 * # Safety
 * blob_key — 32 байта; sealed_blob — len байт; out_ptr/out_len ненулевые.
 */
int mt_e2e_open_blob(const uint8_t *blob_key,
                     const uint8_t *sealed_blob,
                     size_t len,
                     uint8_t **out_ptr,
                     size_t *out_len);

/**
 * pad_len(n) — целевой размер после паддинга (скрытие размера).
 */
size_t mt_e2e_pad_len(size_t n);

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
                         size_t *out_len);

/**
 * party_code(account_id) → 30 ASCII-цифр (Этап 8). Указатель — 32 байта; выход owned.
 *
 * # Safety
 * `id` валиден на 32 байта; out-указатели ненулевые.
 */
int mt_e2e_party_code(const uint8_t *id,
                      uint8_t **out_ptr,
                      size_t *out_len);

/**
 * call_key/sframe_key (Этап 13, PQ-медиа-слой звонка). `call_seed` — 32 байта (из E2E-сигнала);
 * out — 64 байта: call_key(32) ‖ sframe_key(32).
 *
 * # Safety
 * `call_seed` валиден на 32 байта; `out` — на 64 байта.
 */
int mt_e2e_call_key(const uint8_t *call_seed,
                    uint8_t *out);

/**
 * Кодирует WakeInline (recv_id 32 + window 8 LE) в `out` (40 B). true при успехе.
 *
 * # Safety
 * `recv_id` — валиден и ≥32 B; `out` — валиден и ≥40 B.
 */
bool mt_wake_inline_encode(const uint8_t *recv_id, uint64_t window, uint8_t *out);

/**
 * Декодирует WakeInline из `input` (len B). При успехе пишет recv_id (32) + window.
 *
 * # Safety
 * `input` — валиден и ≥`len` B; `out_recv_id` — ≥32 B; `out_window` — валиден.
 */
bool mt_wake_inline_decode(const uint8_t *input,
                           size_t len,
                           uint8_t *out_recv_id,
                           uint64_t *out_window);

/**
 * Кодирует WakeHandle (wake_handle 16 + window 8 LE) в `out` (24 B). true при успехе.
 *
 * # Safety
 * `handle` — валиден и ≥16 B; `out` — валиден и ≥24 B.
 */
bool mt_wake_handle_encode(const uint8_t *handle,
                           uint64_t window,
                           uint8_t *out);

/**
 * Декодирует WakeHandle из `input` (len B). При успехе пишет wake_handle (16) + window.
 *
 * # Safety
 * `input` — валиден и ≥`len` B; `out_handle` — ≥16 B; `out_window` — валиден.
 */
bool mt_wake_handle_decode(const uint8_t *input,
                           size_t len,
                           uint8_t *out_handle,
                           uint64_t *out_window);

/**
 * Арбитр ступеней: возврат — номер ступени 1–4 (высшая суверенность первой).
 */
uint8_t mt_wake_select_rung(bool live_tunnel,
                            bool ibeacon_home,
                            bool unlock_sync);

/**
 * Создаёт реестр account_id↔wake_handle (для телефона-почтальона). Освобождается
 * `mt_wake_registry_free`.
 */
WakeRegistry *mt_wake_registry_new(void);

/**
 * # Safety
 * `reg` — указатель от `mt_wake_registry_new` (не использованный после free) или null.
 */
void mt_wake_registry_free(WakeRegistry *reg);

/**
 * Регистрирует account_id (32 B), пишет 16 B wake_handle. Идемпотентна. true при успехе.
 *
 * # Safety
 * `reg` валиден; `account_id` — ≥32 B; `out_handle` — ≥16 B.
 */
bool mt_wake_register(WakeRegistry *reg,
                      const uint8_t *account_id,
                      uint8_t *out_handle);

/**
 * Ищет wake_handle по account_id. true если найден (пишет out_handle), иначе false.
 *
 * # Safety
 * `reg` валиден; `account_id` — ≥32 B; `out_handle` — ≥16 B.
 */
bool mt_wake_handle_of(const WakeRegistry *reg,
                       const uint8_t *account_id,
                       uint8_t *out_handle);

/**
 * Резолвит account_id по wake_handle (почтальон, ступень 4). true если найден.
 *
 * # Safety
 * `reg` валиден; `handle` — ≥16 B; `out_account` — ≥32 B.
 */
bool mt_wake_account_of(const WakeRegistry *reg,
                        const uint8_t *handle,
                        uint8_t *out_account);

/**
 * Деривация ключей очереди из routing_secret(32)+queue_index — recv/send ML-DSA keypairs.
 * out_recv_pk[1952] out_recv_sk[4032] out_send_pk[1952] out_send_sk[4032]. 0=успех, -1=ошибка.
 *
 * # Safety
 * routing_secret -> 32 B; out_* — валидные буферы на указанные размеры.
 */
int32_t mt_muq_derive_queue_keys(const uint8_t *routing_secret,
                                 uint64_t queue_index,
                                 uint8_t *out_recv_pk,
                                 uint8_t *out_recv_sk,
                                 uint8_t *out_send_pk,
                                 uint8_t *out_send_sk);

/**
 * Сериализация Queue (wire §413) для регистрации. send_pk null = unsecured-очередь.
 * Возврат: записанные байты (QUEUE_WIRE_SIZE) или 0 при ошибке/малом буфере.
 *
 * # Safety
 * recv_id/send_id/recv_pk -> 32/32/1952; send_pk -> 1952 или null; out -> out_cap байт.
 */
size_t mt_muq_queue_serialize(const uint8_t *recv_id,
                                 const uint8_t *send_id,
                                 const uint8_t *recv_pk,
                                 const uint8_t *send_pk,
                                 uint64_t rotation_epoch,
                                 uint32_t quota,
                                 uint8_t *out,
                                 size_t out_cap);

/**
 * Случайный QueueId (32 B, OS CSPRNG) — recv_id либо send_id. 0=успех, -1=ошибка.
 *
 * # Safety
 * out — валиден на 32 байта.
 */
int32_t mt_muq_gen_queue_id(uint8_t *out);

/**
 * Node hello (serverless-автомат): подключиться к узлу addr, получить capability —
 * host_kem (1184 B в out_kem) + send_id (32 B в out_send_id). Отправитель по mDNS находит
 * узел собеседника и узнаёт куда депонировать, без карты. 0=успех, -1=ошибка.
 */
int32_t mt_node_hello(const char *addr, uint8_t *out_kem, uint8_t *out_send_id);


// Этап 1 второго фронта — локальный архив «Монтана/Чаты/<чат>/»
int32_t mt_archive_append(const char *base_path, const char *chat_name,
                          const uint8_t *hk, const uint8_t *account_id,
                          const uint8_t *device_id,
                          const uint8_t *conv_id, uint8_t dir,
                          uint64_t send_time, const uint8_t *content, size_t content_len);
// Этап 2 — ArchiveRoot над всем локальным архивом (отпечаток для якоря/сходимости), 32 байта в out
int32_t mt_archive_root(const char *base_path, const uint8_t *hk,
                        const uint8_t *account_id, uint8_t *out);
// Этапы 3-4 — block-репликация архива (сходимость ArchiveRoot между устройствами)
int32_t mt_writer_tag(const uint8_t *device_id, uint8_t *out);
int32_t mt_archive_block_id(const uint8_t *sealed, size_t sealed_len,
                            uint8_t *out_writer_tag, uint64_t *out_block_seq);
intptr_t mt_archive_export(const char *base_path, const char *chat_name,
                           const uint8_t *writer_tag, uint64_t from_seq,
                           uint8_t *out, size_t out_cap);
int32_t mt_archive_ingest(const char *base_path, const char *chat_name,
                          const uint8_t *hk, const uint8_t *account_id,
                          const uint8_t *sealed, size_t sealed_len);
int32_t mt_archive_peek_conv(const uint8_t *hk, const uint8_t *account_id,
                             const uint8_t *sealed, size_t sealed_len, uint8_t *out);
int32_t mt_archive_put_media(const char *base_path, const char *chat_name,
                             const char *blob_id_hex, const uint8_t *hk, const uint8_t *account_id,
                             const uint8_t *blob, size_t blob_len);
intptr_t mt_archive_get_media(const char *base_path, const char *chat_name,
                              const char *blob_id_hex, const uint8_t *hk, const uint8_t *account_id,
                              uint8_t *out, size_t out_cap);

#endif  /* MONTANA_FFI_H */
