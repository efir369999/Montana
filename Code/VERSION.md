# Version

**Implementation:** `1.2.0` (mainnet — reference singleton genesis: bootstrap is the sole genesis Active operator (n_seed=0, genesis_active_operators empty), sole canonical proposer with quorum=1 singleton-cementing per Network spec; PARAMS_ENCODED_SIZE 4108. Non-bootstrap operators (nicosia, mac) join post-genesis via the standard CandidateVdf → Registered → Active admission path; candidates disabled by default (observers sync+heartbeat, --enable-candidate to re-arm). Closes the n_seed=2 code-vs-doc drift left after REAUDIT-04 (code lagged docs). build sha tracked in git log per [C-1])
**Spec target:** Montana Protocol v35.26.2 + Montana Network v1.4.0 + Montana App v3.12.0 (2026-06-15)
**Release tag:** v1.2.0 (2026-06-19) — singleton mainnet genesis (n_seed=0, PARAMS_ENCODED_SIZE 4108); Genesis State Hash re-baked; one bootstrap proposer finalizes every window alone (quorum=1); conformance contract reconciled to singleton
**Cohort:** singleton bootstrap on mainnet — protocol_params.n_seed = 0, genesis_active_operators empty; consensus starts from one bootstrap proposer (hash-bound in Genesis State Hash). The manifest nodes are discovery peers only.
**Spec paths:**
- Protocol: `../Montana Protocol v35.26.2.md`
- Network:  `../Montana Network v1.4.0.md`
- App:        `../Montana App v3.12.0.md`
- Whitepaper: `../Montana Whitepaper v0.1.0.md`
