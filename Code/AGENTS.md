# AGENTS.md — entry point for AI agents and security researchers

This document is the canonical entry point for any AI agent or human researcher who wants to deploy a Montana node, stress-test it, audit the code, and report findings. Read top-to-bottom; commands are copy-paste ready.

## What Montana is, in 60 seconds

Montana is a from-scratch post-quantum blockchain. Three architectural primitives:

1. **Sequential-delay TimeChain** — globally-ordered windows of ~60 seconds each, sealed by a sequential SHA-256 chain (D = 325 000 000 iterations per window). Cannot be parallelized, cannot be skipped. Verification costs the same order of work as computation, so this is not a VDF in the efficient-verification literature sense.
2. **Time-as-scarcity** — anti-spam through window-rate-limits, chain_length thresholds, seniority gating. No transaction fees. No gas. Cannot accelerate operations by paying.
3. **Post-quantum where the protocol currently claims it** — ML-DSA-65 (FIPS 204) for consensus signatures, ML-KEM-768 (FIPS 203) at the application layer, SHA-256 for hashing and TimeChain, PBKDF2 for key derivation. Transport confidentiality is tracked separately through the Noise_PQ migration.

Spec is the single source of truth: [`../Montana Protocol v35.25.1.md`](../Montana%20Protocol%20v35.25.1.md). ~600 KB markdown, full protocol specification.

## Status

**Pre-mainnet v0.2.** No mainnet date. No token launch. No premine. The implementation is M1-M6 + M9 ready for external audit; M8 (node binary) is still in progress, with known deviations and closures tracked in [`docs/SPEC_DEVIATIONS.md`](docs/SPEC_DEVIATIONS.md). DEV-012 (multi-node proposal apply in the node binary) remains the current mainnet blocker.

Singleton means: each deployed node is its own genesis bootstrap, ticking TimeChain locally and writing its own state. Until multi-node proposal apply is wired into the binary, nodes do not provide production BFT consensus.

This is intentional for v0.2 — it lets you deploy and break a Montana node end-to-end **without** needing a production network of peers.

---

## Deploy

### One command on a clean Linux VPS (Ubuntu 24.04 / Debian 12 / Fedora / Alpine)

```bash
git clone https://github.com/efir369999/Montana.git /opt/montana && \
sudo bash /opt/montana/Code/scripts/install-vps-full.sh
```

What this does (≈10 minutes on a 1 vCPU VPS):

1. Installs system deps (build-essential, clang, git, perl, ca-certificates)
2. Installs Rust toolchain (rustup, stable channel)
3. Builds `montana-node` from source (`cargo build --release -p montana-node`)
4. Creates system user `montana` and `/var/lib/montana`
5. **Generates 24-word recovery mnemonic and prints it once** — save it immediately, no second chance
6. Installs systemd unit with hardening (`User=montana`, `NoNewPrivileges`, `ProtectSystem=strict`)
7. Starts `montana-node.service`
8. Installs xray Reality VPN endpoint as a separate systemd service (optional, runs alongside the node — see [`montana-vpn/README.md`](montana-vpn/README.md))

After install:

```bash
systemctl status montana-node            # is it running
journalctl -u montana-node -f            # live logs (one line per ~60s window)
montana-node status --data-dir /var/lib/montana    # phase, balance, current_window
```

### Just the node, no VPN

```bash
sudo bash /opt/montana/Code/scripts/install-vps.sh
```

### Just the VPN, no node

```bash
sudo bash /opt/montana/Code/montana-vpn/install.sh
```

### macOS (Apple Silicon)

```bash
git clone https://github.com/efir369999/Montana.git ~/Montana && \
bash ~/Montana/Code/scripts/install-local-mac.sh
```

This installs the node under `~/Library/Application Support/Montana/node/` with a launchd agent.

### Build from source manually

See [`docs/build-from-source.md`](docs/build-from-source.md) for the reproducible-build path. Short version:

```bash
cd Code
cargo build --release -p montana-node
./target/release/montana-node --help
```

---

## Verify the node is healthy

After 5 minutes of running:

```bash
montana-node status --data-dir /var/lib/montana
```

Expected output (numbers will differ):

```
phase                : Active
current_window       : 5
D (current)          : 325000000
NodeTable            : 1 records
balance              : 65000000000 nɈ      (5 windows × 13 Ɉ baseline emission)
supply (closed-form) : 78000000000 nɈ      (must equal Σ balance + future emission curve)
```

Healthy invariants:
- `phase == Active` (genesis bootstrap activates immediately, no Candidate VDF wait)
- `current_window` increases by exactly 1 per ~60 seconds
- `Σ balance == supply (closed-form) × correction-for-emission-schedule`
- `state_root` after each apply_proposal byte-equals the expected recompute (logged at INFO level)

Pathological signs (file an Issue):
- Phase regresses (Active → Bootstrap)
- `current_window` stops advancing for >2 minutes
- `state_root` mismatch in logs
- Process panics or OOMs
- Disk usage grows >10 MiB per hour (it should be ~50 KiB per hour)

---

## Stress test — what to throw at it

We have not stress-tested at scale. Here is what would help:

### 1. VDF correctness under chaos

Kill the node mid-window and restart. State must resume from the last cemented window without divergence. Repeat 100×, automate with `kill -9` + immediate `systemctl start`.

```bash
for i in {1..100}; do
  systemctl restart montana-node
  sleep $((RANDOM % 30 + 5))
  systemctl kill -s KILL montana-node
done
journalctl -u montana-node --since "10 minutes ago" | grep -iE 'panic|error|state_root mismatch'
```

Expected: zero panic, zero state_root mismatch. Any divergence = blocker-level finding.

### 2. Disk-full handling

Fill `/var/lib/montana` to 100% mid-operation. Node must fail gracefully, not corrupt state.

```bash
fallocate -l $(df --output=avail /var/lib/montana | tail -1)k /var/lib/montana/_filler
journalctl -u montana-node -f
# clean up:
rm /var/lib/montana/_filler
```

### 3. Clock skew

Set system clock backwards 1 hour mid-operation. Node uses VDF iterations as the clock, not wall-time, so it should ignore the jump.

```bash
date -s "$(date -d '1 hour ago')"
sleep 120
date -s "$(date)"   # let NTP correct it back
journalctl -u montana-node --since "5 minutes ago"
```

Expected: no behavior change. Wall-time is not consensus-critical.

### 4. Determinism — two nodes, same mnemonic, same state_root

Critical. If two independent installs of the same `git rev` with the same seed mnemonic produce different `state_root` after N windows — that is a consensus-fork bug.

```bash
# on host A:
montana-node init --data-dir /tmp/a --mnemonic "<same 24 words>"
montana-node start --data-dir /tmp/a --max-windows 100 --d-test-override 1000

# on host B:
montana-node init --data-dir /tmp/b --mnemonic "<same 24 words>"
montana-node start --data-dir /tmp/b --max-windows 100 --d-test-override 1000

# compare:
diff <(sha256sum /tmp/a/proposals/*.bin) <(sha256sum /tmp/b/proposals/*.bin)
```

Expected: identical SHA-256 sums for every proposal. Any diff = consensus-fork-level finding.

### 5. Memory / CPU profiling

Run the node for 24 hours. Monitor RSS over time.

```bash
while true; do
  ps -p $(pgrep montana-node) -o rss=,vsz= >> /tmp/montana-mem.log
  sleep 60
done
```

Expected: RSS ≈ stable around 30-50 MiB (peak ~100 MiB during VDF burst). Continuous growth >1 MiB/hour = leak.

### 6. Fuzz inputs to apply_proposal

The state machine `apply_proposal(state, proposal) → state'` must reject malformed input deterministically. Use `cargo-fuzz`:

```bash
cd Code
cargo install cargo-fuzz
cargo fuzz run apply_proposal_arbitrary -- -max_total_time=3600
```

(Fuzz harness scaffolding may not yet exist for every entry point — adding more = welcome PR.)

---

## Audit the code against the spec

The spec is at [`../Montana Protocol v35.25.1.md`](../Montana%20Protocol%20v35.25.1.md), authoritative.

### Known deviations

[`docs/SPEC_DEVIATIONS.md`](docs/SPEC_DEVIATIONS.md) lists known deviations, acknowledgments, and closures across the implementation. Each entry:

- Spec quote
- Code location
- What the code actually does
- Severity (mainnet blocker / medium / cosmetic)
- Closure path

Any **un-documented** deviation you find = high-value finding. File as an Issue with:

- Spec section + dossier line number
- Code path (`crates/<crate>/src/<file>:LLL`)
- Test that demonstrates the deviation
- Suggested fix or "needs architect input"

### Internal audit infrastructure

- [`AUDIT.md`](AUDIT.md) — pre-audit self-attestation, single-page summary for an external audit firm
- [`docs/audit-checklist.md`](docs/audit-checklist.md) — what we covered internally
- [`docs/security-cards.md`](docs/security-cards.md) — per-primitive security analysis (FN-DSA, ML-KEM, SHA-256 VDF, PBKDF2)

### Spec-vs-code comments

Every consensus-critical decision in the code references the spec section it implements:

```rust
// spec, раздел "Consensus encoding layer"
fn encode(...) { ... }
```

Grep for `// spec, раздел` to find all anchored references:

```bash
rg "// spec, раздел" --type rust
```

---

## Report findings

GitHub Issues + Pull Requests at https://github.com/efir369999/Montana

**Issue template (free-form, no enforcement):**

```
Title: [SEVERITY] short description

Severity: blocker / high / medium / low / informational

What you did:
  - command sequence to reproduce

What you expected:
  - per spec section X.Y, expected behavior is Z

What happened instead:
  - actual output / log excerpt / state diff

Spec reference:
  - Protokol/Montana Protocol v35.25.1.md, section "X.Y"

Suggested fix (if any):
  - <or "needs architect input">
```

**PR template:**

- Reference the Issue this PR closes
- Include test that fails before the fix and passes after
- Update `docs/SPEC_DEVIATIONS.md` if closing a known deviation
- Run `cargo fmt + clippy + test --all + build --release` (all must be green)

No NDA. No engagement contract. Public review by default.

---

## Things that would be especially welcome

- [ ] Fuzz harness for every public entry point (currently partial)
- [ ] Property-based test coverage for `apply_*` functions (currently has unit + invariants, not property-based)
- [ ] Differential testing harness comparing this Rust impl vs an independent re-implementation (Go / Python / Zig)
- [ ] CI matrix across Linux x86_64 + ARM64 + macOS ARM64 + Windows (currently single-platform local)
- [ ] Benchmark suite measuring VDF iter/sec across CPU classes (we have one micro-bench, not a suite)
- [ ] Deterministic-replay framework (record all inputs to apply_proposal, replay byte-for-byte)
- [ ] Side-channel analysis on PBKDF2 / signature verify (timing, cache, branching)
- [ ] Audit of the crowdsec / fail2ban / ufw default rules for the VPS installer
- [ ] Independent translation of the spec (currently RU primary, EN fragments)

---

## What we will NOT do

- We will **not** sell tokens. Not now, not at mainnet. Montana has no premine, no presale, no airdrop schedule. Block reward (13 Ɉ per window to operator) is the only emission, paid to whoever ran the VDF for that window.
- We will **not** add fees. Anti-spam is time-based by architectural invariant `[I-15]` of the spec.
- We will **not** add ECDSA / RSA / curve25519 fallback. Post-quantum from day one is invariant `[I-1]`.
- We will **not** add KYC, allowlist, or compliance backdoors. Privacy-by-default is invariant `[privacy-default]`.

---

## Contact

- GitHub Issues: https://github.com/efir369999/Montana/issues
- Mastodon (announcements only, no support): see `montana-vpn/MASTODON_ANNOUNCEMENT.md`
- No email, no Discord, no Telegram — public on-record review only

---

*Pre-mainnet. Break it, fix it, send PRs.*
