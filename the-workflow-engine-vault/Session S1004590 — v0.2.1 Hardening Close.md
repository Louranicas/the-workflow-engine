> Back to: [[HOME]] · [[MASTER_INDEX]] · `~/claude-code-workspace/the-workflow-engine/CLAUDE.local.md` (§ "v0.2.1-hardening CLOSED")

# Session S1004590 — v0.2.1 Hardening Close

**Date:** 2026-05-24 · **Tag:** `v0.2.1-hardening` (no SemVer bump — see CHANGELOG entry "Why this is a hardening tag")
**Commits on `main`:** `28e4209` (code fixes) + `ff5c23a` (CHANGELOG + Plan v2 §6 NA-4 amendment) — pushed origin + gitlab.
**Tests:** 2163 (v0.2.0) → **2164** · clippy + pedantic clean · 4-stage gate green.

## What this round did

Post-ship verification cluster (6 parallel agents on v0.2.0 ship diff) returned 11 findings
across 3 verdict bands. This session folds in **10 of 11** (1 deferred as honest residual
with strum::EnumCount dep concern):

| Closure | Source | What | File |
|---|---|---|---|
| **M2** | silent-failure-hunter | `BackPressureRegistry::is_substrate_explicitly_set` — NA-5 parity with `SubstrateTrust::is_substrate_imagined_for` | `src/back_pressure/mod.rs:188` |
| **M3** | silent-failure-hunter | `SubstrateTrust::set → Option<TrustEntry>` for overwrite detection | `src/substrate_trust/mod.rs:156` |
| **N1** | silent-failure-hunter re-audit | `#[must_use]` on both `SubstrateTrust::set` AND `BackPressureRegistry::set_mode` so the M3 contract is compiler-enforced not docstring-only | `src/substrate_trust/mod.rs:155` + `src/back_pressure/mod.rs:197` |
| **L1** | silent-failure-hunter | `BankSnapshot::variant_id_set: HashSet<u64>` — O(1) consistency lookup | `src/m30_bank/mod.rs:285,440` |
| **L2** | silent-failure-hunter | `dispatch::diversity_score_from_proposal` let-else with tracing::debug on None branch | `src/orchestration/dispatch.rs:519` |
| **L3** | silent-failure-hunter | m13 outbox filename expect documented (constructor validates invariant) | `src/m13_stcortex_writer/mod.rs` |
| **Zen #1** | zen re-review | m16 hot-path `Vec::with_capacity` pre-reserve; residual `format!` deferred honestly | `src/m16_substrate_drift_canary/mod.rs:263,268` |
| **Zen #2** | zen re-review | `refusal_for_unavailable` reason prefixed with `engine_imagined:` / `substrate_unreachable:` / `substrate_authored:` for log-grep audit; signature `String → &str` (caller-friendly) | `src/substrate_trust/mod.rs:222-251` |
| **Zen #3** | zen re-review | `ConsistencyVerifier` HashSet (paired with L1) | `src/m30_bank/mod.rs` |
| **Zen #5** | zen re-review | `SubstrateId` variant-count test honest-residual'd — strum::EnumCount deferred to v0.2.2+ | `src/back_pressure/tests.rs:143` |

## Why this is `v0.2.1-hardening` not v0.2.1

No wire-contract changes. No new public types or modules. Net +1 test (from M3's
`registry_set_mode_overrides_default_and_returns_prior` + 3 NA-5 prefix assertion updates).
Tagged as `v0.2.1-hardening` to distinguish from a real v0.2.1 feature release — this is
the post-ship "drift caught + corrected" round that every substrate-safety milestone
should expect.

## NA-pass meta-pattern preserved

Per `~/claude-code-workspace/CLAUDE.local.md` § "Working Mode": *"For any major plan:
write it once, then ask what frame is that? and write it again from the frame you didn't
take. Both passes are the plan."* — applied recursively here. The post-ship NA-pass on
the ship record itself surfaced NA-4 (V3 self-canary mitigation requires Watcher-side
integration; v0.2.0 shipped m16 KEYSTONE + AlertBudget but NOT the Watcher liveness
consumer). Plan v2 §6 NA-4 row amended in-place with `[v0.2.0 SHIPPED 2026-05-24 honesty
amendment, S1004590]`: risk is mitigated structurally (heartbeat exists, shaped for
consumption) but NOT loop-closed. OP-6 in CHANGELOG `[v0.2.0]` carries the operator
hand-off.

## Honest residuals → v0.2.2+

- m16 alert-path `format!()` String alloc — typed reason enum
- `SubstrateId` `strum::EnumCount` compile-time variant-count enforcement
- V1 RefusalToken consumer-side call-site cascade (~65 occurrences of `RefusalReason`) — per ADR D-S1004XXX-04 § 1.2 + Plan v2 D44 C-2 lean co-land
- Substrate-side schema/daemon work per ADR D-S1004XXX-05 + Plan v2 §11
- Watcher m16 heartbeat liveness consumer (NA-4 loop closure) — OP-6

## 4-surface persistence

| Surface | Anchor |
|---|---|
| ai_docs canonical | `CHANGELOG.md` `[v0.2.1-hardening]` + Plan v2 §6 NA-4 row amendment |
| Obsidian vault | THIS NOTE (`the-workflow-engine-vault/Session S1004590 — v0.2.1 Hardening Close.md`) |
| stcortex | ns `workflow_trace_completion_s1004115` (extends chain; bidi pathway → v0.2.0 ship memory id) |
| CLAUDE.local.md | project `the-workflow-engine/CLAUDE.local.md` "v0.2.0 SHIPPED" section + workspace row honesty amendment |

## Cold-start sequence for fresh Claude window

```bash
cd /home/louranicas/claude-code-workspace/the-workflow-engine
git log --oneline -3      # expected: ff5c23a docs hardening + 28e4209 fix hardening + (v0.2.0 commit)
git tag | grep v0.2       # expected: v0.2.0
$EDITOR CHANGELOG.md       # read [v0.2.1-hardening] block
$EDITOR CLAUDE.local.md    # read "v0.2.0 SHIPPED" + "v0.2.1-hardening CLOSED" anchor
```
