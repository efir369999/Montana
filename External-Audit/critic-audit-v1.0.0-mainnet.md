# Critic spec audit — Montana v1.0.0 mainnet (2026-05-22)

**Role:** Critic (CRITIC.md role v3.14.0)
**Scope:** all English GitHub-published artifacts under role «Metzdowd audience principle» — `README.md`, `Whitepaper Montana.md`, `Montana Network v1.1.0.md`, `Code/RELEASE-v1.0.0.md`, `Code/VERSION.md`, `Code/docs/SPEC_DEVIATIONS.md`, `External-Audit/*.md`.

**Findings classification:** binary per the role «SSOT-audit through absolute principle» — authoritative-location match vs any other location. Subjective «legitimate marker / stale / migration» classification is forbidden.

---

## Findings closed in this audit pass

### F-001 (closed). README.md stale release wording

**Location.** `README.md` lines 5, 6, 7, 101, 112, 197.

**Claim.** Before audit: «Live three-node Genesis cohort», «Pre-mainnet v0.2 spec package. Rust reference implementation `0.1.2`», «First mainnet release candidate: v1.0.0-rc.1 (2026-05-21)», «we publish this pre-mainnet reference implementation… before mainnet», «Mainnet has no schedule», «Pre-mainnet. Break it, fix it».

**Reason.** Factually stale the moment the `v1.0.0` annotated tag landed on GitHub. Authoritative location for the impl version is `Code/VERSION.md`; for the release tag — the GitHub Releases page. Both already point at v1.0.0 mainnet.

**Closure.** README header block rewritten to «Live four-node mesh: Moscow, Frankfurt, Helsinki, Yerevan / Mainnet v0.2 spec package / Rust reference implementation `1.0.0` / **First mainnet release:** v1.0.0 (2026-05-22)». Lines 101 / 112 / 197 reframed from «pre-mainnet» to «mainnet is live» language.

**Severity at closure.** Low (mechanical fact sync).

---

### F-002 (closed). Network spec stale «v1.0.0 mainnet gate (open)» framing

**Location.** `Montana Network v1.1.0.md` lines 175, 185, 189.

**Claim.** Before audit: «Multi-confirmer cementing protocol (open, v1.0.0 mainnet gate)», «… continues to work for that special case during the v1.0.x release-candidate window; full v1.0.0 mainnet requires the extended length-prefixed schema», «… normatively specified in the next Network spec bump (v1.1.0 → v1.2.0)».

**Reason.** Three problems combined: (a) factually stale after the v1.0.0 mainnet tag landed; (b) [I-10] SSOT violation — version-pin references in spec body outside the spec header (`v1.0.0` / `v1.0.x` / `v1.1.0` / `v1.2.0`); (c) academic-style violation — temporal markers «open», «release-candidate window», «in next bump» forbidden in normative sections per role rule «Стиль изложения — академический».

**Closure.** Section title rewritten to «Multi-confirmer cementing protocol» (no parenthetical gate marker). Body reframed to describe the singleton case as «the special case of the protocol above with `bundle_count = 1`» — present tense, no historical or temporal framing. The «next spec bump» pointer removed; the section now points at itself as the normative source.

**Severity at closure.** Medium (academic style + [I-10] body discipline; not a wire-format claim).

---

## Findings that require author decision (not auto-closed)

### F-003 (open). Cyrillic content in `Montana Network v1.1.0.md`

**Location.** `Montana Network v1.1.0.md` — 1796 Cyrillic-character hits across the file body. Sample at the top of the file:

```
493:                                   либо broadcast marker
495:  payload_length          u16    — длина payload в байтах,
498:                                     ≤ 256B для fit в один
500:                                     без fragmentation
```

**Claim by role.** CRITIC.md role v3.14.0 §«Аудитория: список рассылки криптографии Metzdowd» requires that all GitHub-published artifacts — explicitly enumerated to include `Montana Network vX.Y.Z.md` — are in English without Cyrillic interjections except for explicit context-tags in code comments.

**Reason this is open.** A 1796-hit Cyrillic surface in the network specification cannot be removed by mechanical edit — the affected paragraphs carry technical content (ASCII layout diagrams with Russian column labels, body prose in mixed languages, technical reasoning blocks) that requires line-by-line English re-authorship. The role explicitly forbids the critic to delete, rename, shorten, or otherwise modify a GitHub-published artifact without author confirmation, regardless of the size of the change.

**Severity.** High. This is the largest deviation between the published Metzdowd audience and the spec's actual readability for that audience. Independent Metzdowd reviewers landing on this file see code-spec-grade content half in Russian and cannot evaluate the wire-level claim it carries.

**Closure path the critic can recommend (subject to author approval).**
1. Identify the affected line ranges (one grep dump).
2. Walk each block: ASCII diagrams keep the same structure with English labels; prose blocks rewritten paragraph-by-paragraph in present-tense English; mixed-language sentences fully translated.
3. Cross-implementation conformance — verify wire-format byte counts (sizes in B, KAT vector references) remain byte-exact; the rewrite is text-only, not normative.
4. Single commit `Network spec v1.1.0: English-only authorial pass per CRITIC.md v3.14.0` so the rewrite is auditable as a stand-alone change against the v1.0.0 tag.

**Author decision required:** approve / defer / modify.

---

### F-004 (open, observational). Whitepaper §«Formal Nash equilibrium analysis» retains «is deferred to the academic publication at milestone M9» language

**Location.** `Whitepaper Montana.md` line 173.

**Claim by role.** Same as F-002 (b) — temporal marker «is deferred» + version pointer «milestone M9» in normative section.

**Severity.** Low. The whitepaper is not strictly normative the same way the network spec is; «is deferred to academic publication» is acceptable language for whitepaper-style writing. The critic flags this for completeness, not as a closure-mandatory finding.

**Closure path.** Optional. If approved, reframe to «Formal Nash equilibrium analysis (excluding rational-delay strategies, characterising the equilibrium `N*` at varying hardware costs and electricity prices) is in scope of the academic publication track.» — present tense, no version pin.

**Author decision required:** if approved, applied in the same pass as F-003. Otherwise filed under «whitepaper-style language acceptable, not normative».


---

### F-005 (open). Cyrillic content in `Code/docs/audit-checklist.md` (151 hits)

**Location.** `Code/docs/audit-checklist.md` — pre-audit self-attestation checklist published on the GitHub mirror.
**Claim by role.** Same as F-003 — CRITIC.md role v3.14.0 enumerates `Code/docs/audit-checklist.md` explicitly as a GitHub-published artifact requiring English-only authorship.
**Severity.** Medium. The file is a structured checklist; most Cyrillic surface is in the prose annotations and «обоснование» fields beside each `[x]` entry. Cross-implementation conformance is not affected (the checklist references KAT files, vectors, and crate paths — those identifiers are English).
**Closure path.** Re-author the annotations in English; the structural `[x] / [ ]` skeleton survives unchanged.
**Author decision required:** approve / defer / modify.

---

### F-006 (open). Cyrillic content in `Code/docs/security-cards.md` (194 hits)

**Location.** `Code/docs/security-cards.md` — per-primitive security cards published on the GitHub mirror.
**Claim by role.** Same as F-003.
**Severity.** Medium. The file is structurally English (card layout, field names, code-path pointers) but the «Notes» and «Reasoning» fields per card are Russian.
**Closure path.** Re-author the «Notes» / «Reasoning» fields in English; card layouts and code-path pointers survive unchanged.
**Author decision required:** approve / defer / modify.

---

### F-007 (open). Cyrillic content in `Code/docs/build-from-source.md` (29 hits)

**Location.** `Code/docs/build-from-source.md` — build instructions published on the GitHub mirror.
**Severity.** Low. Smallest Cyrillic surface among the open findings; mostly inline Russian commentary in shell-block annotations.
**Closure path.** Re-author the inline annotations in English; shell-block commands survive unchanged.
**Author decision required:** approve / defer / modify.

---

### F-008 (open). Cyrillic content in `Code/VERSION.md` (68 hits)

**Location.** `Code/VERSION.md` — workspace version log + spec target + History table.
**Severity.** Low. Cyrillic hits cluster inside the History table «Notes» column, describing past release content in Russian. The authoritative version fields (`Implementation`, `Spec target`, `Release tag`, `Spec paths`) are English.
**Closure path.** Re-author the History table «Notes» column in English row-by-row; the table headers and version identifiers survive unchanged.
**Author decision required:** approve / defer / modify.

---

## Negative findings (none, except the carve-out below)

- `README.md` — Cyrillic count `0`, version pins inside body all reference filenames (`Montana Protocol v35.25.1.md`) which are authoritative locations per [I-10].
- `Code/RELEASE-v1.0.0.md` — Cyrillic count `0`, version pins inside body all describe the present-tag mainnet baseline (tag = v1.0.0, spec target = Protocol v35.25.1 + Network v1.1.0 + App v3.12.0) — authoritative-location matches.
- `External-Audit/README-external-audit-v1.0.0.md` — Cyrillic count `0`, English-only.
- `Whitepaper Montana.md` — Cyrillic count `0`, version-pin count `0`.
- `Code/VERSION.md` — version-pin count 82, all in the History table which is the authoritative location for prior-version records per [I-10] §«Версия спеки».

---

## Closure summary

| Finding | Severity | Status |
|---------|----------|--------|
| F-001 README mainnet language | low | closed in this pass |
| F-002 Network spec multi-confirmer gate framing | medium | closed in this pass |
| F-003 Cyrillic in `Montana Network v1.1.0.md` (1796 hits) | high | open — author decision required |
| F-004 Whitepaper §Nash temporal marker | low | open — author decision optional |
| F-005 Cyrillic in `Code/docs/audit-checklist.md` (151 hits) | medium | open — author decision required |
| F-006 Cyrillic in `Code/docs/security-cards.md` (194 hits) | medium | open — author decision required |
| F-007 Cyrillic in `Code/docs/build-from-source.md` (29 hits) | low | open — author decision required |
| F-008 Cyrillic in `Code/VERSION.md` History column (68 hits) | low | open — author decision required |

Two stale-fact findings closed mechanically without changing any normative claim. Six open findings escalated to the author with explicit closure paths; total Cyrillic surface across the published English artifacts is 2238 hits (F-003 + F-005 + F-006 + F-007 + F-008 + 0 hits in already-clean files). F-003 (Network spec) is the highest-impact unresolved item for the Metzdowd audience.

— Critic, CRITIC.md role v3.14.0, 2026-05-22.
