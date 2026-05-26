# F-CT1 correction — ml_dsa_sign.c:296 is NOT a real CT leak

**Severity correction.** F-CT1 in §13.2 of the audit was classified **HIGH**. After deeper analysis of the
OpenSSL 3.5.5 LTS verify code path, this classification is **incorrect**. The actual severity is
**Observational / Negligible** for production Montana usage.

**This file is a self-correction**, not a patch.

## Original finding (now reconsidered)

§10.2 of the audit flagged `crypto/ml_dsa/ml_dsa_sign.c:296`:

```c
ret = (z_max < (uint32_t)(params->gamma1 - params->beta))
    && memcmp(c_tilde, sig.c_tilde, c_tilde_len) == 0;
```

…as a "plain `memcmp` early-exit timing leak on forgery attempts" worthy of a `CRYPTO_memcmp`
patch in OpenSSL upstream.

## Why it is not a real leak

Both arguments of this `memcmp` are **public values** for the duration of the verify operation:

- `c_tilde` (left operand) — recomputed by the verifier from `(pk, msg, sig.z, sig.hint)`.
  Verifier-side deterministic recomputation:
  `c_tilde = SHAKE-256(mu ‖ w1_encoded)`
  where `mu = SHAKE-256(tr ‖ M)`, `tr = SHAKE-256(pk)`, and `w1_encoded` is derived from the
  attacker-supplied signature components via `A·z − c·t1`. **Any party with `(pk, msg, sig)`
  can compute this value themselves.**

- `sig.c_tilde` (right operand) — the `c_tilde` field of the attacker-supplied signature,
  already on the wire and visible to the attacker by construction.

An adversary submitting forgery attempts and observing early-exit timing learns:
- "Byte 0 of my submitted `c_tilde` matches/does-not-match the value the verifier recomputes."

But the verifier-recomputed value is **already computable by the adversary** — `c_tilde` does not
depend on the verifier's secret key (verifier holds only the public key `pk`). Sign-side `c_tilde`
depends on the signer's secret material, but on the verify path, both inputs are public.

**Result: 0 bits of new information leak per timing observation.**

## Comparison with real CT leaks in the same file

Real CT leaks in the OpenSSL 3.5.5 ML-DSA implementation, by contrast, are at:

| Location | Operand semantics | Severity |
|----------|------------------|----------|
| `ml_dsa_key.c:277` | `memcmp(key1->priv_encoding, key2->priv_encoding)` — both SK material | **MEDIUM** (cold path: `EVP_PKEY_eq`, not on signing hot path) |
| `ml_dsa_key.c:485` | `memcmp(out->priv_encoding, sk, sk_len)` — operator-supplied SK vs reconstructed SK | **MEDIUM** (cold path: `EVP_PKEY_fromdata` import; Montana hits this on every sign call until F-CT-MONTANA-1 patch lands) |

These are the **actual** memcmp leaks in OpenSSL 3.5.5 ML-DSA. Both compare SK material against
either another SK or operator input. Both are on cold paths (key import, key equality), not on the
signing hot path itself.

## Action

§10.2 and §13.2 of the audit are **corrected by addendum**:
- F-CT1 (ml_dsa_sign.c:296) — **downgraded from HIGH to Observational/None**. No upstream patch needed.
  The behavioural-style ctgrind errors in this region are false positives caused by valgrind seeing
  branches that descended from secret-marked SK bytes — but the branches themselves operate on
  values that the verifier could compute independently. The information flow is `secret → public
  recomputation → comparison`, not `secret → branch outcome`.
- F-CT2 (ml_dsa_key.c:277, 485) — **retained as MEDIUM**. Closure via F-CT-MONTANA-1 patch
  (cache EVP_PKEY, avoid `from_secret` re-import path), not via OpenSSL upstream patch. Montana
  controls whether these code paths are hit.

The corrected score for §3 `mt-crypto-native` reverses one of the −0.5 deductions: the in-repo
code score recovers from 6.0 → **6.5** pending closure of F-CT-MONTANA-1.

## Lessons learned

The ctgrind methodology — marking SK as UNDEFINED then running valgrind — produces error reports
on every branch whose path descends from SK bytes. **Not every such branch is a side-channel leak.**
Three filters distinguish real leaks from false positives:

1. **Is the branch outcome dependent on SK after intermediate computations?**
   `c_tilde` verifier recomputation: depends on SK only through public `pk`, not on the SK directly.
2. **Could an external observer reproduce the comparison oracle?**
   Verifier-side `c_tilde` is fully reproducible from `(pk, msg, sig)`. Attacker submits sig,
   computes `c_tilde` themselves, learns nothing new from timing.
3. **Does the comparison gate access to SK-derived secret state?**
   `c_tilde` comparison gates only the verify return value (accept/reject) — which the attacker
   already learns from the response regardless of timing.

`ml_dsa_sign.c:296` fails all three filters for "real leak". `ml_dsa_key.c:277, 485` pass filter 1
(branch depends on SK directly) and fail filters 2, 3 only because `EVP_PKEY_eq` and `fromdata`
import are not typically attacker-triggerable in production.

## Updated F-CT scorecard

| Finding | Original severity | Corrected severity | Mitigation owner |
|---------|-------------------|---------------------|------------------|
| F-CT1 (ml_dsa_sign.c:296) | HIGH | **Observational** | None — public-input comparison, not a leak |
| F-CT2 (ml_dsa_key.c:277,485) | MEDIUM | MEDIUM | Montana via F-CT-MONTANA-1 patch (cache EVP_PKEY) |
| F-CT3 (rejection-loop iteration count) | LOW | LOW | FIPS 204 community accepts; deterministic mode amplifies determinism, not leak |
| F-CT-MONTANA-1 (FFI re-import per call) | HIGH | HIGH (retained) | Montana patch in this directory |
