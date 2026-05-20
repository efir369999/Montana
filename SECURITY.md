# Security Policy

## Reporting a Vulnerability

Montana is a post-quantum blockchain protocol with TimeChain consensus. Security is foundational.

If you discover a security vulnerability, **do NOT open a public issue**. Instead:

- Email: `efir369999@gmail.com`
- Include: clear description, reproduction steps, affected component (e.g. `mt-crypto`, `mt-consensus`, `mt-net`), severity assessment.
- Allow up to **14 days** for initial response.

We follow **responsible disclosure**: vulnerability is fixed before public disclosure. Reporters are credited in release notes (unless they prefer anonymity).

## Scope

In scope:
- Cryptographic primitives in `Code/crates/mt-crypto/` and `Code/crates/mt-crypto-native/`
- Consensus and sequential-delay TimeChain logic in `Code/crates/mt-consensus/`, `Code/crates/mt-timechain/`, `Code/crates/mt-lottery/`
- Network layer in `Code/crates/mt-net/`, `Code/crates/mt-net-transport/`
- Wallet, anchor, transfer logic in respective `mt-*` crates
- Specification ambiguities or contradictions in `Montana Protocol v*.md`, `Montana Network v*.md`, or `Montana App v*.md`

Out of scope:
- Issues only reproducible with non-default `protocol_params`
- Performance issues without security impact
- Anything outside `Code/` and the Montana spec files

## Security Architecture

- **Post-quantum primitives**: ML-DSA-65 (FIPS 204), ML-KEM-768 (FIPS 203), SHA-256, HKDF-SHA256, PBKDF2.
- **Single Source of Truth (SSOT)**: every constant lives in exactly one place; no duplication.
- **Audit trail**: see `Code/docs/audit-checklist.md` (53/53 findings closed for M6 + M9).
- **Reproducible builds**: `Code/docs/build-from-source.md` provides verification steps.

## Audit Status

- **Internal**: Pass 1-17 critic reviews complete. Roles in `CLAUDE.md`, `CRITIC.md`.
- **External**: Pending engagement (target firms — NCC Group, Trail of Bits, Cure53, Quarkslab, Cryspen).
