# Version

**Implementation:** `1.1.0` (mainnet — unified 3-node N_SEED genesis cohort (lauterbourg bootstrap + nicosia + mac operators, n_seed=2); non-empty-N_SEED genesis-fork fixed via single-source build_genesis_state; candidates disabled by default (observers sync+heartbeat, --enable-candidate to re-arm); GPT-5 Codex 02 audit closed (REAUDIT-01..06). build sha tracked in git log per [C-1])
**Spec target:** Montana Protocol v35.26.2 + Montana Network v1.4.0 + Montana App v3.12.0 (2026-06-15)
**Release tag:** v1.1.0 (2026-06-18) — unified mainnet genesis: 3 Active operators baked hash-bound into protocol_params (PARAMS_ENCODED_SIZE 4108 -> 11916); one-command Docker install; conformance-gate GREEN 39/39
**Cohort:** singleton bootstrap on mainnet — protocol_params.n_seed = 0, genesis_active_operators empty; consensus starts from one bootstrap proposer (hash-bound in Genesis State Hash). The 5 manifest nodes are discovery peers only.
**Spec paths:**
- Protocol: `../Montana Protocol v35.26.2.md`
- Network:  `../Montana Network v1.4.0.md`
- App:        `../Montana App v3.12.0.md`
- Whitepaper: `../Montana Whitepaper v0.1.0.md`
