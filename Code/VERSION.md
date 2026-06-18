# Version

**Implementation:** `1.0.4` (mainnet — Build 27 / v1.0.4 all-in-one (build sha tracked in git log, not pinned here per [C-1]); DEV-017..023c hot-fix track closed: peer t_r_history, fast-sync chunk anchor_window, peer-quorum 30s grace, Reveal pipeline + winner determination, DEV-022/023 rotation disabled pending DEV-021b drain refactor)
**Spec target:** Montana Protocol v35.26.2 + Montana Network v1.4.0 + Montana App v3.12.0 (2026-06-15)
**Release tag:** v1.0.4 (2026-05-30) — first hot-fix track; bootstrap-only proposer baseline; Helsinki node decommissioned; genesis manifest lists 5 discovery peers (moscow + frankfurt + vilnius + armenia + nicosia) — discovery metadata only, not consensus-active operators
**Cohort:** singleton bootstrap on mainnet — protocol_params.n_seed = 0, genesis_active_operators empty; consensus starts from one bootstrap proposer (hash-bound in Genesis State Hash). The 5 manifest nodes are discovery peers only.
**Spec paths:**
- Protocol: `../Montana Protocol v35.26.2.md`
- Network:  `../Montana Network v1.4.0.md`
- App:        `../Montana App v3.12.0.md`
- Whitepaper: `../Montana Whitepaper v0.1.0.md`
