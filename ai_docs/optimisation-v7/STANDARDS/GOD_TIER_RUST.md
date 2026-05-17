---
title: GOD-TIER RUST STANDARDS — workflow-trace V7
date: 2026-05-17 (S1001982)
kind: planning-only · standards manifest · atuin-cited
purpose: the bar workflow-trace ships to; below this bar = REJECT regardless of correctness
inheritance: ULTRAPLATE habitat-wide standards + ORAC / ME v2 / LCM precedent
cardinality_amendment: "S1002127 — PrivilegeEscalation inserted at ordinal 30 (D-S1002127-02 ADR)"
---

# GOD-TIER RUST STANDARDS — workflow-trace V7

> Back to: [[../TASK_LIST_V7_OPTIMISATION.md]] · [[../KEYWORDS_20.md]] · [[TEST_DISCIPLINE.md]]
>
> *"the greatest synthetic coder in history" is the standard. Pre-existing project warnings do NOT excuse new code.* — MEMORY.md `feedback_god_tier_no_warnings_at_any_level.md`

---

## The 18 god-tier rules

| # | Rule | Enforcement | Atuin script citation |
|---|---|---|---|
| 1 | **`forbid(unsafe_code)`** at crate root | compiler error if violated | `atuin scripts run hab-quality-gate` |
| 2 | **`deny(clippy::unwrap_used)`** + `expect_used` outside tests | compiler error if violated | same |
| 3 | **`deny(clippy::pedantic)`** as warning-converted-to-error at gate | gate fails if any pedantic | `atuin scripts run habitat-density` (lints checked) |
| 4 | **`#![warn(missing_docs)]`** on public items | gate warns; ratchets to deny over phases | per-module `cargo doc --no-deps` clean |
| 5 | **No `tokio::spawn` returning JoinHandle silently dropped** | grep ban: `tokio::spawn(.*); *$` | review-time grep |
| 6 | **No `let _ =`** outside FFI/intentional-drop with `// rationale:` comment | clippy::let_underscore_must_use | gate |
| 7 | **No `.ok()` discarding `Result`** without `// rationale:` | rg ban: `\.ok\(\)$` without comment | review-time + silent-swallow-detect skill |
| 8 | **No `unwrap_or(true)` / `unwrap_or(0)` / `Ok(0)` success sentinels** on health/consent/success paths | silent-swallow-detect skill | `/silent-swallow-detect` |
| 9 | **Every public fn has a doc comment with `# Errors` section** if returning `Result` | cargo doc + custom lint | per-module check |
| 10 | **No `panic!` outside startup or build-time invariant checks** | grep audit | `rg 'panic!\(' src/ \| grep -v 'tests/' \| grep -v 'startup'` |
| 11 | **Error types must implement `std::error::Error` via `thiserror`** | derive macro on every error enum | review-time |
| 12 | **`#[must_use]` on every fn returning `Result`/`Option`/builder** | clippy::must_use_candidate | gate |
| 13 | **`tracing::instrument(skip(..))` on every public fn** with structured fields | clippy::missing_tracing_instrument (custom) | review-time |
| 14 | **No `String::from` / `format!` in hot paths** (gate detected via flamegraph) | criterion bench + flamegraph | `atuin scripts run habitat-evolution-delta` |
| 15 | **No `Vec::push` in inner loops where capacity is bounded** | review-time + clippy::push_in_loop | gate |
| 16 | **`#[cfg(test)] use proptest` for every value-type with invariants** | per-module test count check | gate |
| 17 | **`#[cfg(test)] mod fuzz` for every parser/decoder** | per-module fuzz target | `cargo +nightly fuzz` |
| 18 | **No `chrono`/`time` in hot paths — use `std::time::Instant`** for monotonic intervals | review-time | per-module bench |

---

## 4-stage quality gate (mandatory before every commit)

```bash
# PIPESTATUS discipline mandatory — see AP-Hab-05
set -o pipefail

# Stage 1: cargo check (catches type errors fast)
CARGO_TARGET_DIR=./target cargo check --workspace --all-targets --all-features 2>&1 | tail -20
[[ ${PIPESTATUS[0]} -ne 0 ]] && { echo "STAGE 1 FAIL: cargo check"; exit 1; }

# Stage 2: clippy (style + correctness)
cargo clippy --workspace --all-targets --all-features -- -D warnings 2>&1 | tail -20
[[ ${PIPESTATUS[0]} -ne 0 ]] && { echo "STAGE 2 FAIL: clippy"; exit 2; }

# Stage 3: clippy pedantic (god-tier discipline)
cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic 2>&1 | tail -20
[[ ${PIPESTATUS[0]} -ne 0 ]] && { echo "STAGE 3 FAIL: clippy pedantic"; exit 3; }

# Stage 4: tests
CARGO_TARGET_DIR=./target cargo test --workspace --all-targets --all-features --release 2>&1 | tail -30
[[ ${PIPESTATUS[0]} -ne 0 ]] && { echo "STAGE 4 FAIL: tests"; exit 4; }

echo "ALL 4 STAGES PASS"
```

**Source:** ORAC canonical gate (Session S1000400 — first-ever live `cargo test --lib --release --features full` across all 14 services). 2,893 tests at that time.

**Atuin reference:**
```bash
atuin scripts run hab-quality-gate
```

---

## Boilerplate header for every Rust source file

```rust
// SPDX-License-Identifier: <pending — Luke decision>
//
// Module: src/m{N}_{theme}/{filename}.rs
// Layer: L{n}
// Cluster: {A-H}
// Purpose: <one-line>
// Upstream: <list of mN modules this depends on>
// Downstream: <list of mN modules that depend on this>
// Test budget: {n} (≥50 minimum)
// Antipatterns mitigated: AP-WT-F{x}, AP-Hab-{y}
//
// Authored: <date> per workflow-trace V7 module plan cluster-{X}.md
// 4-stage QG: PASS as of <commit SHA>

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
```

---

## Trait usage discipline

- **`Debug`** required on every public type. Auto-derive unless reasons documented in doc comment.
- **`Clone`** explicit decision — derive only if cheap; `Arc<T>` wrapper if not.
- **`Send + Sync`** required on every type crossing thread boundary; document why if explicit `!Send`.
- **`Default`** derive only if zero-value is semantically meaningful (NOT for builders).
- **`PartialEq + Eq`** for value types; explicit `PartialEq` impl for types where some fields are excluded.
- **`Hash`** for any type used as `HashMap` key — pair with `Eq`.

---

## Async discipline

- **Tokio runtime exclusively** (no async-std, no smol).
- **`tokio::main(flavor = "multi_thread", worker_threads = N)`** explicit; N matches CPU expectation.
- **Sync code in async context** must `spawn_blocking` if > 100μs. AP29 (sync HTTP in tokio::spawn starves) — catalogued S225.
- **Channel choice:** `tokio::sync::mpsc` for backpressure-needed; `flume` if both sync+async producers.
- **Cancellation:** `CancellationToken` propagated; never abort tasks abruptly.
- **Timeout discipline:** every `await` on external IO has `tokio::time::timeout(Duration, future)`.

---

## Error handling discipline

- **Use `thiserror`** for error enums in libs; `anyhow` only at bin entry-points.
- **Error variants enumerate failure modes**, not loosely-typed `Generic(String)`.
- **`#[error("…")]`** message includes operational guidance, not just description.
- **Wrap external errors with context** — `.map_err(|e| MyError::Bridge { source: e, peer: "synthex" })`.
- **No `Box<dyn Error>`** in lib types; only in bin top-level.

---

## Logging / tracing discipline

- **`tracing`** crate exclusively (no `log` direct, no `println!` in production paths).
- **Spans per layer:** `#[tracing::instrument(skip(self), fields(module = "m20"))]`.
- **Levels:**
  - `error!` — operational intervention required
  - `warn!` — degraded but functional
  - `info!` — state transitions
  - `debug!` — per-request detail
  - `trace!` — per-step granularity (compile-time disabled in release)
- **Structured fields:** prefer `event_kind = "dispatch_refused", reason = "conductor_unreachable"` over string concatenation.
- **No PII / secrets:** never log API keys, PATs, user content.

---

## Dependency hygiene

- **Workspace deps only** — never per-crate `[dependencies]` for shared types.
- **Pin minor versions** — `tokio = "1.40"` not `tokio = "*"`.
- **`cargo audit` clean** at every Wave-end.
- **`cargo deny` configured** to forbid GPL/AGPL transitive deps unless explicitly approved.
- **No `chrono`** in favour of `time` (chrono CVE history); exception only with documented rationale.

---

## Per-binary discipline (`wf-crystallise` + `wf-dispatch`)

Each binary:
- **`main.rs`** ≤ 50 LOC — pure orchestration; logic in lib.
- **Clap derive** for CLI parsing; no manual argv handling.
- **`anyhow::Result<()>`** return; exit code via `process::exit`.
- **Signal handlers** for SIGINT/SIGTERM → graceful shutdown via `CancellationToken`.
- **`--version`** prints SemVer + commit SHA + build profile.
- **`--help`** generated from clap; per-subcommand help.
- **Config layering:** env > CLI flag > config file > built-in default (per ORAC layered-TOML pattern).

---

## Per-module discipline

Each `src/mN_<theme>/` module:
- **`mod.rs`** — module facade; public re-exports; no logic.
- **`{theme}_impl.rs`** — core implementation.
- **`tests/`** — ≥50 tests (per TEST_DISCIPLINE.md).
- **`benches/`** — criterion bench if hot path.
- **`README.md`** — module-level doc with bidi anchor back to MODULE_PLANS/cluster-N.md.

---

## Verify-sync invariants (≥18, synthex-v2 pattern)

Workflow-trace target set (per ULTRAMAP § View 2):

| # | Invariant | Check command |
|---|---|---|
| 1 | every L1-L8 module has src/ entry | `for n in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 20 21 22 23 30 31 32 33 40 41 42; do test -d src/m${n}_* \|\| echo "MISS m$n"; done` |
| 2 | every src/ module has cluster-N.md entry | scripted grep |
| 3 | every cluster-N.md module has src/ entry | scripted grep |
| 4 | every module has ≥50 tests | `cargo test --no-run --message-format=json \| jq …` per module |
| 5 | no module exceeds 2× LOC budget | `wc -l src/m*/*.rs` vs ULTRAMAP table |
| 6 | every feature gate has ≥1 module with `#[cfg(feature = X)]` | rg audit |
| 7 | no `unsafe` anywhere | `rg '\bunsafe\b' src/` returns 0 |
| 8 | no `unwrap()` outside `tests/` | scripted |
| 9 | no `panic!()` outside startup | scripted |
| 10 | every public fn has doc comment | `cargo doc --no-deps` clean |
| 11 | every public fn returning Result has `# Errors` doc section | rg `## Errors` per fn |
| 12 | every error enum derives `thiserror::Error` | rg `derive.*Error` |
| 13 | every async fn uses `#[instrument]` | rg audit |
| 14 | every external IO has timeout | rg `tokio::time::timeout` density |
| 15 | every bidi edge in ULTRAMAP appears in src/ as use-statement | static analysis |
| 16 | every AP-WT-F* failure mode has ≥1 test asserting mitigation | grep test names |
| 17 | every CC-1..CC-7 synergy has integration test | `tests/integration/cc_N_*.rs` count |
| 18 | every binary has `--version` SemVer + commit | smoke test |
| 19 | EscapeSurfaceProfile enum cardinality = **7** (Sandboxed/SandboxEscape/ProcessMutate/PrivilegeEscalation/FileWrite/NetworkEgress/DataExfil; ordinals 0/10/20/30/40/50/60 — gap-reserved per D-S1002127-02; was 5 in original V7, 6 in v1.3, now 7) | rg count |
| 20 | m32 dispatch path NEVER invokes workflow exec directly (Conductor-only) | rg ban `dispatch_direct\|exec_local` in m32 |

**Invariant file:** `scripts/verify-sync.sh` — per-Wave-end + per-PR CI.

---

## Anti-patterns specifically banned in workflow-trace

(See full register at [ANTIPATTERNS_REGISTER.md](../ANTIPATTERNS_REGISTER.md). Top 10 most-relevant:)

1. AP-Hab-01 (pre-G9 code)
2. AP-Hab-03 (AP30 namespace violation)
3. AP-Hab-05 (PIPESTATUS swallow)
4. AP-Hab-14 (god-tier dilution)
5. AP-WT-F2 (sample-size inflation — `ProposalBuilder::build()` MUST reject n<20 at construction)
6. AP-WT-F4 (premature dispatch — m32 5-check sequence mandatory)
7. AP-WT-F7 (CR-2 graceful-degrade — m8 must hard-fail outside band)
8. AP-Drift-01 (over-claim gate-clean — Wave-end re-run mandatory)
9. AP-Drift-06 (bridge contract drift — `bridge-contract` skill mandatory)
10. AP-V7-09 (substrate-frame confusion — every module spec includes operationalised inputs)

---

## Atuin-cited references (the standard's provenance)

| Standard rule | Atuin reference | Why cited |
|---|---|---|
| 4-stage QG | `atuin scripts run hab-quality-gate` | habitat-wide canonical |
| Quality density | `atuin scripts run habitat-density` | service-by-service quality baseline |
| Evolution tracking | `atuin scripts run habitat-evolution-delta` | RALPH gen + fitness delta gate |
| Cross-service tensor | `atuin scripts run habitat-cross-tensor` | post-deploy verification |
| Bridge health | `atuin scripts run habitat-bridge-check` | Phase 5B cutover |
| Fingerprint | `atuin scripts run habitat-fingerprint` | per-Wave-end binary check |
| K7 flex | `atuin scripts run habitat-k7-flex` | full-fleet probe (~10-probe parallel) |
| Living loop | `atuin scripts run habitat-living-loop` | continuous health |

**To find new standards as they crystallise:** `atuin scripts list \| grep -E 'gate\|quality\|standard'`.

---

## Verification

```bash
# Standards compliance check (post-implementation, not now):
# Stage 1: 4-stage gate per Quality Gate Protocol
# Stage 2: verify-sync invariants 1-20
# Stage 3: antipattern audit (all AP-Hab + AP-WT + AP-Drift detection commands)
# Stage 4: Watcher class-A-I coverage check (every gate has flag pre-position)
```

---

*GOD_TIER_RUST authored 2026-05-17 by Command. 18 rules + 4-stage gate + 20 verify-sync invariants + 10 banned antipatterns + atuin provenance. The bar.*
