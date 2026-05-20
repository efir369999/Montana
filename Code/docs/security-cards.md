# Security Cards — Montana M1 cryptographic primitives

Mandatory documentation per `Протокол/Code/CRITIC.md` v1.6.0 Pass 17 — каждый primitive имеющий secret material обязан иметь заполненную Security Card перед статусом «closed».

**Last verified:** 2026-04-26 (M1-F audit + Pass 17 Security Card formalization)
**Scope:** M1 foundational layer cryptographic primitives
**Automated regression:** [crates/mt-crypto/tests/security_invariants.rs](../crates/mt-crypto/tests/security_invariants.rs) — 13 invariants verified в CI

---

## Card 1: SecretKey (ML-DSA-65)

```
Security Card для SecretKey (mt_crypto::SecretKey, ML-DSA-65 4032B):

Secret material:
  Type:                 [u8; 4032] heap-allocated через Box<[u8; SECRET_KEY_SIZE]>
  Site of construction: crates/mt-crypto/src/lib.rs — impl SecretKey { from_array, from_slice }
                        + alloc_locked_secret_box helper (search by function name)
  Site of destruction:  crates/mt-crypto/src/lib.rs — impl Drop for SecretKey
                        (line numbers намеренно не фиксированы — синхронизация
                         с кодом через grep -n "fn from_array\|impl Drop for SecretKey")

Lifecycle:
  Construction copies:  1 — bytes копируются один раз с stack source (либо FFI-fill direct в heap Box)
                            в heap-allocated Box. Stack source zeroized после copy.
  Owning type:          mt_crypto::SecretKey (private inner Box<[u8; N]>)
  Transfer pattern:     by-value move; Box pointer copy, не bytes memcpy
  Destruction:          Drop+zeroize: yes; explicit zeroize sites: 1 (Drop impl)
                        + munlock heap страниц перед dealloc

Side-channel surface:
  Branching on secret bytes: no (наш Rust shim не имеет операций над bytes;
                                все ML-DSA arithmetic внутри OpenSSL EVP API)
  Memory access pattern:     N/A в Layer 1; OpenSSL внутри использует
                              constant-time access patterns (FIPS 140-3 validated)
  PartialEq impl на secret type: disabled (verified compile-time через
                                   security_invariants.rs)
  Comparison via ==:         no (PartialEq не derived; verified)
  Constant-time гарантии:    inherited from OpenSSL 3.5.5 LTS — FIPS 140-3
                              constant-time crypto operations

OS-level hygiene:
  mlock applied:        yes; через alloc_locked_secret_box() в mt-crypto/src/lib.rs
                        best-effort (errno ignored на failure — fallback на encrypted swap)
  Stack cleansing FFI buffers: explicit — keypair_from_seed аллоцирует Box ДО FFI call,
                                FFI пишет напрямую в heap; никаких stack temporary buffers
                                с secret bytes (verified в fn keypair_from_seed)
  Swap protection:      mlock primary; encrypted swap (FileVault macOS / LUKS Linux)
                        fallback assumption documented
  Core dump protection: рекомендация для operator: setrlimit(RLIMIT_CORE=0) в
                        production deployment (документируется в Operator Guide,
                        не enforced на code level)

Logging surface:
  println!/log macros на secret: 0 instances (verified file-content scan в
                                   security_invariants.rs::no_println_or_log_on_secret_bytes_in_lib_code)
  Debug impl на secret type:     not derived; struct fields private (no field access)
  Error messages с secret:       нет (CryptoError variants не содержат secret bytes)
  print_sk-like helper gates:    yes — mt-examples/examples/m1_crypto.rs::print_sk gated через
                                  env var M1_DUMP_SK=1 (default redacted; см. fn dump_sk_enabled)

Library properties:
  Underlying impl:      OpenSSL 3.5.5 LTS EVP_PKEY API (vendored через openssl-src)
  Constant-time documented:  yes — OpenSSL FIPS 140-3 validation requires constant-time
  Audit history:        OpenSSL Foundation governance + decades production deployment в
                        TLS world + FIPS 140-3 certified
  Stack cleansing on cleanup: OpenSSL responsibility — EVP_PKEY_free clears internal state

Verified:
  Pass 17 checks 1-8:   8/8 closed
    1. Constant-time:     ✅ inherited OpenSSL
    2. Memory access:     ✅ no secret-indexed access в Layer 1
    3. Branch pattern:    ✅ no secret-dependent branches в Layer 1
    4. Zeroization on drop: ✅ Drop+zeroize verified
    5. Library check:     ✅ OpenSSL FIPS 140-3
    6. Stack hygiene:     ✅ heap-only via Box; FFI пишет в heap
    7. OS-level mlock:    ✅ best-effort applied
    8. Memory barrier:    ✅ zeroize crate имеет compiler_fence(SeqCst)

Status: closed
```

---

## Card 2: MlkemSecretKey (ML-KEM-768)

```
Security Card для MlkemSecretKey (mt_crypto::MlkemSecretKey, ML-KEM-768 2400B):

Secret material:
  Type:                 [u8; 2400] heap-allocated через Box<[u8; MLKEM_SECRET_KEY_SIZE]>
  Site of construction: crates/mt-crypto/src/lib.rs — impl MlkemSecretKey { from_array, from_slice }
                        (search by function name; line numbers намеренно не фиксированы)
  Site of destruction:  crates/mt-crypto/src/lib.rs — impl Drop for MlkemSecretKey

Lifecycle:
  Construction copies:  1 — bytes на heap, stack source zeroized
  Owning type:          mt_crypto::MlkemSecretKey (private Box)
  Transfer pattern:     by-value move (pointer copy)
  Destruction:          Drop+zeroize + munlock

Side-channel surface:
  Branching on secret bytes: no (Layer 1 only передаёт pointer в OpenSSL EVP)
  Memory access pattern:     N/A в Layer 1; OpenSSL constant-time
  PartialEq impl на secret type: disabled (verified)
  Comparison via ==:         no (verified)
  Constant-time гарантии:    inherited OpenSSL FIPS 140-3

OS-level hygiene:
  mlock applied:        yes; alloc_locked_secret_box best-effort
  Stack cleansing FFI buffers: explicit — keypair_from_seed_mlkem uses heap
                                Box directly (mt-crypto/src/lib.rs fn keypair_from_seed_mlkem)
  Swap protection:      mlock primary; encrypted swap fallback
  Core dump protection: operator-level (RLIMIT_CORE=0)

Logging surface:
  println!/log macros на secret: 0 (file-content scan verified)
  Debug impl:           not derived
  Error messages:       sanitized
  Helper gates:         нет direct dump helpers для MlkemSK

Library properties:
  Underlying impl:      OpenSSL 3.5.5 LTS EVP_PKEY ML-KEM-768
  Constant-time documented:  yes — FIPS 140-3
  Audit history:        OpenSSL Foundation
  Stack cleansing on cleanup: OpenSSL EVP_PKEY_free

Threat model (per-primitive — отличается от ML-DSA Card 1):
  - Decapsulation timing: KEM decapsulation алгоритм по design содержит
    secret-dependent control flow при failure mode. OpenSSL EVP реализация
    делает implicit rejection в constant time (FIPS 203 §6.3 Algorithm 18).
  - Plaintext checking attacks: Kyber известна за hijacking decapsulation
    через crafted ciphertext (Hofheinz-Hövelmanns-Kiltz [HHK17]).
    Защита — implicit rejection с pseudorandom output, реализована в
    OpenSSL и проверяется через FIPS 140-3 validation.
  - Encapsulation: PK material не секретный, но ciphertext путь содержит
    секретный сеансовый ключ. Ciphertext output — public material.
  - vs SecretKey (Card 1): ML-DSA SK используется только в Sign (один
    secret-touch operation в lifecycle); ML-KEM SK используется в каждом
    Decap (множественные exposures, выше критичность constant-time).

Verified:
  Pass 17 checks 1-8 (per-primitive analysis для KEM):
    1. Constant-time:           ✅ inherited OpenSSL FIPS 140-3 (включая
                                   implicit rejection путь для decap failure)
    2. Memory access:           ✅ no SK-indexed access в Layer 1
    3. Branch pattern:          ✅ no SK-dependent branches в Layer 1
                                   (decap branching внутри OpenSSL constant-time)
    4. Zeroization on drop:     ✅ Drop+zeroize verified
    5. Library check:           ✅ OpenSSL FIPS 140-3
    6. Stack hygiene:           ✅ heap-only via Box; FFI пишет в heap
    7. OS-level mlock:          ✅ best-effort applied
    8. Memory barrier:          ✅ zeroize crate compiler_fence(SeqCst)

Status: closed
```

---

## Card 3: keypair_from_seed — ML-DSA-65 KeyGen

```
Security Card для keypair_from_seed (mt_crypto::keypair_from_seed):

Secret material handled:
  Input:                seed: &[u8; 32] (caller-owned, function берёт по reference)
  Output secret:        SecretKey (4032B, owned)
  Output public:        PublicKey (1952B, public material)

Lifecycle:
  seed lifecycle:       owned by caller; функция читает через &[u8; N], не копирует
                        в local stack (FFI получает caller's pointer напрямую)
  SK construction:      heap Box allocated с mlock ДО FFI call (через
                        alloc_locked_secret_box). FFI пишет SK bytes напрямую
                        в locked heap memory; no intermediate stack copy.
  Error path:           sk_box.zeroize() явно вызван перед return Err
                        ensures partial-fill bytes не leak при FFI failure

Side-channel surface:
  Branching on secret:  no — Layer 1 проверяет только return code (i32)
  Memory access:        SK bytes в heap, accessed только OpenSSL внутри
  Logging:              0 println/log calls на seed/sk

OS-level hygiene:
  mlock на SK:          yes (через alloc_locked_secret_box)
  mlock на seed:        no — seed caller-owned, function не контролирует
                        (caller responsibility — например mt-mnemonic для
                        derived seeds через PBKDF2/HKDF локирует master_seed
                        на своём слое; documented в audit-checklist)
  Stack cleanup:        no stack temp buffers с secret bytes

Library properties:
  Underlying:           OpenSSL EVP_PKEY ML-DSA-65 KeyGen (FIPS 204 Algorithm 1)
  Determinism:          guaranteed via OSSL_PKEY_PARAM_ML_DSA_SEED parameter
  NIST conformance:     verified byte-exact против NIST ACVP 25/25 KeyGen tests

Threat model (per-primitive — KeyGen specific):
  - Seed quality: KeyGen output определяется seed; weak seed → predictable
    SK (полная компрометация). Caller responsibility (mt-mnemonic
    использует HKDF-Expand от mlocked master_seed; OS CSPRNG в keypair()
    test helper через getrandom).
  - Seed exposure: seed после KeyGen теоретически восстановим из SK
    (FIPS 204 §5.1 ξ encoded внутри SK). Это by-design — recovery flow
    через mnemonic regenerates seed → SK байт-идентично.
  - Determinism как security feature: same seed → same (pk, sk).
    Используется для consensus identity (mt-mnemonic), не уязвимость.
  - Stack hygiene critical: KeyGen — единственный momento когда SK байты
    появляются «из ниоткуда»; любой stack temp buffer = leak surface.
    Защита — heap Box + mlock allocated ДО FFI call, FFI пишет
    напрямую в heap memory.
  - Error path leak: при FFI failure partial-fill bytes могут leak —
    защита через явный sk_box.zeroize() перед return Err.

Verified:
  Pass 17 checks (per-primitive analysis для KeyGen):
    1. Constant-time:           ✅ FIPS 204 Algorithm 1 KeyGen внутри
                                   OpenSSL (validation pending external review,
                                   hardware side-channel separately)
    2. Memory access:           ✅ no secret-indexed access в Layer 1
    3. Branch pattern:          ✅ Layer 1 проверяет только return code (i32);
                                   no seed-dependent / sk-dependent branches
    4. Zeroization on drop:     ✅ через returned SecretKey type (Card 1)
                                   + явный sk_box.zeroize() на error path
    5. Library check:           ✅ OpenSSL FIPS 140-3 (KeyGen validated)
    6. Stack hygiene:           ✅ heap Box + mlock allocated ДО FFI;
                                   no stack temporary buffers с secret bytes
    7. OS-level mlock:          ✅ via alloc_locked_secret_box (best-effort)
    8. Memory barrier:          ✅ inherited from SecretKey Drop

Status: closed
```

---

## Card 4: keypair_from_seed_mlkem — ML-KEM-768 KeyGen

```
Security Card для keypair_from_seed_mlkem:

Secret material handled:
  Input:                seed: &[u8; 64] (d || z per FIPS 203 §6.1)
  Output secret:        MlkemSecretKey (2400B)
  Output public:        MlkemPublicKey (1184B)

Lifecycle:
  seed lifecycle:       caller-owned, &-borrow
  SK construction:      heap Box + mlock ДО FFI call (через alloc_locked_secret_box)
  Error path:           sk_box.zeroize() перед return Err

Side-channel surface:
  Same as ML-DSA KeyGen — Layer 1 thin FFI shim, no secret-dependent operations

OS-level hygiene:
  mlock на SK:          yes
  mlock на seed:        caller responsibility
  Stack cleanup:        no stack temp с secret bytes

Library properties:
  Underlying:           OpenSSL EVP_PKEY ML-KEM-768 KeyGen (FIPS 203 Algorithm 16)
  Determinism:          guaranteed via OSSL_PKEY_PARAM_ML_KEM_SEED
  NIST conformance:     verified byte-exact против NIST ACVP 25/25 KeyGen tests

Threat model (per-primitive — KEM KeyGen specific, vs ML-DSA Card 3):
  - Seed format: 64-byte d ‖ z per FIPS 203 §6.1 (vs 32-byte ξ для ML-DSA).
    Двойная domain separation внутри seed (d для key generation polynomial,
    z для implicit rejection PRF). Oba компонента secret-critical.
  - Implicit rejection key: z часть seed становится PRF-ключом для
    decapsulation failure mode. Compromised z = enable plaintext-checking
    attack [HHK17]. Защита — z никогда не покидает SK heap.
  - Stack hygiene: same as ML-DSA KeyGen (heap Box + mlock ДО FFI).
  - Key reuse: ML-KEM SK можно использовать многократно в Decap (vs
    ML-DSA SK в Sign — multiple operations OK). Lifetime exposure выше
    чем для signature SK → mlock/zeroize критичнее.

Verified:
  Pass 17 checks (per-primitive analysis для KEM KeyGen):
    1. Constant-time:           ✅ FIPS 203 Algorithm 16 KeyGen внутри
                                   OpenSSL (FIPS 140-3 validated)
    2. Memory access:           ✅ no seed/sk-indexed access в Layer 1
    3. Branch pattern:          ✅ Layer 1 проверяет только return code
    4. Zeroization on drop:     ✅ через MlkemSecretKey type (Card 2)
                                   + явный sk_box.zeroize() на error path
    5. Library check:           ✅ OpenSSL FIPS 140-3
    6. Stack hygiene:           ✅ heap Box + mlock allocated ДО FFI
    7. OS-level mlock:          ✅ via alloc_locked_secret_box
    8. Memory barrier:          ✅ inherited from MlkemSecretKey Drop

Status: closed
```

---

## Card 5: sign — ML-DSA-65 deterministic Sign

```
Security Card для sign (mt_crypto::sign):

Secret material handled:
  Input:                sk: &SecretKey (borrowed)
  Output secret:        none — signature is public material

Lifecycle:
  sk access:            read-only borrow; bytes остаются в caller's heap
                        Box на всё время вызова
  signature construct:  Stack-allocated [u8; 3309] — public material, не secret
  Drop:                 sig is public, no zeroize нужен

Side-channel surface:
  Branching on secret bytes: no (Layer 1 проверяет только return code)
  Memory access:        sk.0.as_ptr() передан в FFI; OpenSSL внутри делает
                        constant-time deterministic Sign (FIPS 204 Algorithm 2)
  PartialEq на signature: derived (Signature: PartialEq) — OK, signature public
  Logging:              0 на sk; no println на signature внутри sign()

OS-level hygiene:
  mlock на sk:          inherited from SecretKey (already locked)
  Stack cleanup:        none нужен (no stack secret bytes; signature public)

Library properties:
  Underlying:           OpenSSL EVP_DigestSign + OSSL_SIGNATURE_PARAM_DETERMINISTIC=1
  Determinism:          FIPS 204 Algorithm 2 deterministic variant — required для
                        Montana [I-3] consensus determinism
  Constant-time:        OpenSSL FIPS 140-3
  NIST conformance:     verified byte-exact против NIST ACVP 1/1 deterministic
                        SigGen test (empty context)

Threat model (per-primitive — Sign specific, vs SecretKey Card 1):
  - Deterministic Sign critical для consensus: identical (sk, msg) → identical
    signature. Required для Montana [I-3] determinism — две имплементации
    подписывают тот же message и получают bit-identical signature. Random
    signing variant запрещён в consensus path.
  - Sign timing: внутренний rejection sampling в FIPS 204 Algorithm 2
    может иметь secret-dependent number of iterations. OpenSSL FIPS 140-3
    реализация делает constant-time за счёт fixed-iteration upper bound.
  - Signature output: NOT secret material (signature + msg + pk → public).
    Signature::PartialEq derived OK, no zeroize нужен, stack-allocated OK.
  - Side-channel surface: Sign — main attack target в lattice schemes
    (BLISS, Falcon, Dilithium все имели side-channel papers). OpenSSL
    constant-time implementation проходит FIPS 140-3 attestation, но
    hardware side-channel testing вне scope (см. AUDIT.md Out of Scope §5).
  - SK exposure during Sign: sk.0.as_ptr() передан в FFI; OpenSSL читает
    из heap-locked memory; никаких stack copies SK bytes в Layer 1.

Verified:
  Pass 17 checks (per-primitive analysis для Sign):
    1. Constant-time:           ✅ FIPS 204 Algorithm 2 deterministic Sign
                                   constant-time через OpenSSL FIPS 140-3
    2. Memory access:           ✅ no SK-indexed access в Layer 1
                                   (FFI передаёт only pointer, не indexes)
    3. Branch pattern:          ✅ no SK-dependent branches в Layer 1
    4. Zeroization on drop:     ✅ Signature не содержит secret material
                                   (no zeroize нужен); SK через Card 1
    5. Library check:           ✅ OpenSSL FIPS 140-3
    6. Stack hygiene:           ✅ no SK bytes на stack; signature output
                                   на stack acceptable (public material)
    7. OS-level mlock:          ✅ inherited from SecretKey (already locked)
    8. Memory barrier:          ✅ inherited from SecretKey Drop

Status: closed
```

---

## Card 6: verify — ML-DSA-65 SigVer

```
Security Card для verify (mt_crypto::verify):

Secret material handled: none
  Input:                pk: &PublicKey (public material)
                        msg: &[u8] (public material)
                        sig: &Signature (public material)
  Output:               bool (verify result)

Lifecycle:               no secret material involved

Side-channel surface:
  Branching on PK bytes:   no (PK public, не secret — branching на PK acceptable)
  Memory access:           pk/sig bytes accessed только в OpenSSL (constant-time
                            не critical для public material, но OpenSSL делает
                            constant-time всё равно по design)
  Logging:                 0

Library properties:
  Underlying:           OpenSSL EVP_DigestVerify
  Constant-time:        FIPS 140-3 (по design, хотя для public material не critical)

Threat model (per-primitive — Verify specific, NO secret material):
  - PK / msg / sig — все public. Branching на их bytes acceptable
    (нечего leak-ать). Это фундаментальное отличие от Sign Card 5.
  - Verify result — boolean, public-derivable от inputs. No timing leak
    concern (timing зависит от inputs которые public).
  - DoS surface: malformed signature → constant-time rejection? Не critical
    (caller может rate-limit verify calls на чужих signatures).
  - Cross-implementation conformance: Verify должен accept signature от
    любой FIPS 204 imlementation. Это reverse направление от Sign
    determinism — Sign даёт identical bytes, Verify accepts canonical encoding.

Pass 17 checks (per-primitive analysis для Verify):
  Не applicable полностью — нет secret material:
    1-3. Constant-time / memory access / branching: N/A для public material
    4. Zeroization: N/A для public material
    5. Library check: ✅ OpenSSL FIPS 140-3
    6-8. Stack / mlock / barrier: N/A для public material

Status: closed (no secret material — Security Card minimal по design)
```

---

## Re-audit schedule

Все Security Cards re-verified автоматически через:
- `crates/mt-crypto/tests/security_invariants.rs` — каждый CI run
- Manual re-audit обязателен:
  - При смене upstream library (OpenSSL upgrade)
  - При изменении FFI signature (mt-crypto-native API)
  - При добавлении нового entry point с secret bytes
  - Каждые 6 месяцев wallclock на existing primitives (next: 2026-10-26)

## Cross-references

- Critic role enforcement: [CRITIC.md](../CRITIC.md) v1.6.0 §«Mandatory Security Card per crypto primitive»
- Code under audit: [crates/mt-crypto/src/lib.rs](../crates/mt-crypto/src/lib.rs)
- Automated invariants: [crates/mt-crypto/tests/security_invariants.rs](../crates/mt-crypto/tests/security_invariants.rs)
- High-level audit package: [AUDIT.md](../AUDIT.md)
- Pre-audit checklist: [audit-checklist.md](audit-checklist.md)
