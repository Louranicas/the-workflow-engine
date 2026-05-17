---
title: m8 — povm_build_prereq (compile-time CR-2 gate)
module_id: m8
cluster: D — Trust (cross-cutting)
layer: L4
binary: wf-crystallise
feature_gate: [none]
verb_class: refuse
ship_first: true
gap_owner: []
status: SPEC · planning-only · HOLD-v2 · NO CODE · NO CARGO
loc_budget: 60
test_budget: 50
mutation_kill: 70
boilerplate_lift: 0
date: 2026-05-17 (S1001982)
authority: Luke @ node 0.A — AP24 gate (G9 "start coding workflow-trace")
binding_spec: Genesis Prompt v1.3 § 1, § 2
primary_contract: CC-2 (Trust Layer Woven — D → all)
---

# m8 — `povm_build_prereq`

> **Back to:** [`cluster-D/INDEX`](./) · [`ai_specs/INDEX`](../../INDEX.md) · [`MODULE_MATRIX`](../../MODULE_MATRIX.md) · vault [[cluster-D-trust-cross-cutting]] · [cluster-D plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-D.md) · [phase-1](../../../the-workflow-engine-vault/deployment%20framework/phase-1-genesis-day-0-3.md) · [Genesis v1.3](../../../ai_docs/GENESIS_PROMPT_V1_3.md)
>
> **Sister modules (Cluster D):** [m8](m8_povm_build_prereq.md) · [m9](m9_watcher_namespace_guard.md) · [m10](m10_ember_ci_gate.md) · [m11](m11_fitness_weighted_decay.md)

---

## 1. Purpose

`build.rs` script + `src/m8_povm_build_prereq/` runtime mirror that emits `cargo:rustc-cfg=povm_calibrated` when the build-time POVM probe finds `learning_health` inside the post-CR-2 magnitude-weighted band `[0.05, 0.15]`. Without that marker, any `#[cfg(povm_calibrated)]` POVM-reading path in the codebase is dead code; any `#[cfg(not(povm_calibrated))]` tombstone path fires a `compile_error!` naming the upstream commit (`e2a8ed3`), the environment variable (`POVM_CR2_DEPLOYED`), and the canonical reference doc (`~/projects/claude_code/Hebbian Deployment Plan v3 — Post-CR-2 Threshold Reconciliation.md`).

m8 is the **floor of the trust regime**. It is the only Cluster D module with no upstream aspect (per [cluster-D plan § m8](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-D.md)). The compile-time gate is structurally non-bypassable: it lives in `build.rs` and emits a `rustc-cfg`, not a Cargo `[features]` flag — `cargo --features full`, `--all-features`, or any future feature-set cannot activate it. The only activation paths are (a) `POVM_CR2_DEPLOYED=1` in the build environment, set after operator verification that povm-v2 commit `e2a8ed3` is live and serving magnitude-weighted `learning_health`, or (b) the live runtime probe in `src/m8_povm_build_prereq/health.rs` returning a band-valid response.

This module exists because the CR-2 inflation fix changed `learning_health` from 0.9114 (binary-counted, 13.6× inflated) to 0.0668 (magnitude-weighted). Any code that compares `learning_health > 0.3` as a "healthy" threshold — common pre-CR-2 idiom — silently misreads substrate state post-CR-2. A runtime guard is too late: misreads cascade through threshold tuning, decay-rate selection, and m11 sunset gating before any check fires. The compiler gate makes the entire class of bug impossible whenever the marker is absent.

---

## 2. Contracts (CC-2 primary)

| Surface | Direction | Detail |
|---|---|---|
| **CC-2 Trust Layer Woven** (PRIMARY) | OUT → all clusters | m8 emits `cfg(povm_calibrated)` consumed by every module that reads POVM data — currently m42 (substrate emit), m11 (fitness signal indirect), m12 (report rendering of `substrate_LTP_density`), m5 (metrics dim D10). |
| Cargo build invocation | IN | `build.rs` runs at every `cargo check` / `cargo build` invocation; environment via `std::env::var("POVM_CR2_DEPLOYED")`. |
| HTTP probe to POVM `:8125/learning_health` | IN (runtime mirror) | `src/m8_povm_build_prereq/health.rs` performs the same band-check at startup; refuses to proceed if outside band. |
| Cargo-emitted `cfg` flag | OUT | `println!("cargo:rustc-cfg=povm_calibrated")` exactly once per build invocation when in-band. |
| Cargo `rerun-if-env-changed` | OUT | `println!("cargo:rerun-if-env-changed=POVM_CR2_DEPLOYED")` + `POVM_HEALTH_URL` so toolchain re-runs `build.rs` on env mutation. |
| `cargo:warning=` lines | OUT | Emitted on band-edge (within 0.01 of either threshold) so operators see precursor signal without build failure. |

m8 has **no upstream aspect** — it IS the aspect-floor. Per [cluster-D plan § m8](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-D.md): m9, m10, m11 all transitively depend on m8 being in place.

---

## 3. Public surface

```rust
// build.rs (workspace root) — the gate emitter
fn main() { /* probe + cfg-emit; see § 5 */ }

// src/m8_povm_build_prereq/mod.rs
pub mod cfg;     // Band thresholds (single source of truth; shared with build.rs via path-include)
pub mod health;  // Runtime mirror probe (HealthClient + BandClassification)
pub mod error;   // BuildPrereqError + RuntimeBandError

pub use cfg::{POVM_LH_BAND_LOW, POVM_LH_BAND_HIGH, POVM_LH_EDGE_TOLERANCE};
pub use health::{HealthClient, BandClassification, probe_band};
pub use error::{BuildPrereqError, RuntimeBandError};
```

Newtype discipline (per [GOLD_STANDARDS § 8](../../../GOLD_STANDARDS.md)):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BandClassification {
    BelowFloor,      // < 0.05  → CR-2 regression OR uncalibrated
    InBand,          // [0.05, 0.15] → healthy magnitude-weighted
    AboveCeiling,    // > 0.15  → pre-CR-2 binary inflation OR substrate anomaly
    Nan,             // probe returned non-finite
}
```

---

## 4. Errors

```rust
#[derive(Debug, thiserror::Error)]
pub enum BuildPrereqError {
    #[error("POVM CR-2 not verified. Set POVM_CR2_DEPLOYED=1 after confirming povm-v2 commit e2a8ed3 is live and the magnitude-weighted learning_health formula is active. See: ~/projects/claude_code/Hebbian Deployment Plan v3 — Post-CR-2 Threshold Reconciliation.md")]
    Cr2MarkerAbsent,

    #[error("POVM probe at {url} returned learning_health={value} (band [{low}, {high}]); classification={classification:?}")]
    OutOfBand { url: String, value: f64, low: f64, high: f64, classification: BandClassification },

    #[error("POVM probe at {url} unreachable: {source}")]
    ProbeFailed { url: String, #[source] source: reqwest::Error },
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeBandError {
    #[error("startup refused: POVM learning_health={value} outside band [{low}, {high}] — refusing to run against uncalibrated substrate")]
    StartupRefused { value: f64, low: f64, high: f64 },
}
```

Error-band assignment per [`ERROR_TAXONOMY.md` § E3xxx — Trust layer violations](../../INDEX.md): `BuildPrereqError::Cr2MarkerAbsent = E3001`, `OutOfBand = E3002`, `ProbeFailed = E3003`. Compile-time `compile_error!` strings carry the same text as `Cr2MarkerAbsent` for cross-surface message stability.

---

## 5. Implementation sketch

`build.rs` (~30 LOC):

```rust
// build.rs — emits cargo:rustc-cfg=povm_calibrated when POVM CR-2 marker present.
// Pattern source: synthex-v2/build.rs + loop-engine-v2/build.rs (~70% reuse for emission idiom).
// Hard-fail discipline source: ME v2 m1_foundation/config.rs (env-only gate, never a Cargo feature).
fn main() {
    println!("cargo:rerun-if-env-changed=POVM_CR2_DEPLOYED");
    println!("cargo:rerun-if-env-changed=POVM_HEALTH_URL");

    let cr2_deployed = std::env::var("POVM_CR2_DEPLOYED")
        .map(|v| v == "1")
        .unwrap_or(false);

    if cr2_deployed {
        println!("cargo:rustc-cfg=povm_calibrated");
    } else {
        // Warnings appear in cargo build output but do NOT fail the build at this point.
        // The compile_error! tombstones at #[cfg(not(povm_calibrated))] read sites do.
        println!("cargo:warning=POVM CR-2 (magnitude-weighted learning_health) not verified.");
        println!("cargo:warning=Set POVM_CR2_DEPLOYED=1 after verifying povm-v2 commit e2a8ed3 is live.");
        println!("cargo:warning=See: ~/projects/claude_code/Hebbian Deployment Plan v3 — Post-CR-2 Threshold Reconciliation.md");
    }
}
```

POVM-reading site annotation pattern (annotation lives in the gated module, NOT in m8 itself):

```rust
// e.g. src/m42_stcortex_emit/fitness_input.rs
#[cfg(povm_calibrated)]
pub fn read_learning_health_calibrated() -> Result<f64, WorkflowError> { /* … */ }

#[cfg(not(povm_calibrated))]
pub fn read_learning_health_calibrated() -> Result<f64, WorkflowError> {
    compile_error!(
        "POVM CR-2 not verified. Set POVM_CR2_DEPLOYED=1 after confirming \
         povm-v2 commit e2a8ed3 is live and the magnitude-weighted learning_health \
         formula is active. See: ~/projects/claude_code/\
         'Hebbian Deployment Plan v3 — Post-CR-2 Threshold Reconciliation.md'"
    )
}
```

Runtime mirror (`src/m8_povm_build_prereq/health.rs`, ~30 LOC): blocking `reqwest` probe to `${POVM_HEALTH_URL:-http://127.0.0.1:8125/learning_health}` with 2s timeout, JSON deserialisation to `{ learning_health: f64 }`, band classification via `cfg::classify(value)`, return `Result<BandClassification, BuildPrereqError>`. `cfg.rs` carries the only declaration of `POVM_LH_BAND_LOW = 0.05`, `POVM_LH_BAND_HIGH = 0.15`, `POVM_LH_EDGE_TOLERANCE = 0.01`. The build script `include!`s this file via `include!("src/m8_povm_build_prereq/cfg.rs")` so build-time and runtime cannot drift.

Error message design follows [ME v2 `error.rs`](../../../the-workflow-engine-vault/boilerplate%20modules/Maintenance%20Engine%20V2%20—%20Gold%20Standard%20Reference.md): every variant names commit SHA, env var, and reference doc. Operators can recover without log-hunting.

---

## 6. Test plan (50 tests, mutation ≥70%)

Per [TEST_DISCIPLINE matrix row m8](../../../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md) and [cluster-D plan § m8 test-pattern allocation](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-D.md):

| Pattern | Count | Examples |
|---|---:|---|
| **F-Unit** | 25 | band-check per arm (below 0.05 / in-band / above 0.15 / NaN / negative); `HealthClient::probe` per HTTP status arm (200/404/500/timeout); env-var override application; `cargo:warning` emission on band-edge; `classify(0.0499)` → `BelowFloor`; `classify(0.0501)` → `InBand`; `classify(0.1499)` → `InBand`; `classify(0.1501)` → `AboveCeiling`; `classify(f64::NAN)` → `Nan`. |
| **F-Property** | 5 | `classify` is monotonic on `[0.0, 0.2]` step 0.001 (ascending `learning_health` → non-decreasing band-ordinal); idempotent on `(POVM_HEALTH_URL, learning_health)` input pair; deterministic across 1k repeats. |
| **F-Integration** | 15 | live POVM `:8125/learning_health` probe (skipped if `:8125` not reachable, NOT silently passing); CI env-var override path; build-time + runtime mirror agreement on identical input; `cfg(povm_calibrated)` compile-emission verification via `cargo expand` snapshot; `--features full` does NOT enable `cfg(povm_calibrated)` (defense vs AP-V7-09). |
| **F-Contract** | 3 | `learning_health` JSON shape matches POVM `:8125` actual response (snapshot test); band-threshold constants match Hebbian v3 reconciliation note thresholds; `Cr2MarkerAbsent` message text contains literal commit SHA `e2a8ed3`. |
| **F-Regression** | 2 | F7 (CR-2 graceful-degrade pretend-fix) regression slot — band-edge `0.0500001` value must NOT silently classify `InBand` if precision drift sneaks in; build-time/runtime mismatch regression. |
| **F-Mutation** | budget | ≥70% kill rate concentrated on `health.rs` band-check core. |

Compile-time gate verification is structural: build twice, once with `POVM_CR2_DEPLOYED=1`, once without; verify exit code and `compile_error!` text via stdout capture. CI runs both modes per [phase-1 § Day 1 m8](../../../the-workflow-engine-vault/deployment%20framework/phase-1-genesis-day-0-3.md).

---

## 7. Boilerplate lift map

| Source | Lift % | Use |
|---|---:|---|
| `synthex-v2/build.rs` + `loop-engine-v2/build.rs` | 70% (pattern) | `println!("cargo:rustc-cfg=…")` + `rerun-if-env-changed` emission idiom |
| ME v2 `m1_foundation/config.rs` | 40% (pattern) | env-only-gate discipline; `ENV_PREFIX = "WF_"` for `WF_POVM_HEALTH_URL` |
| `boilerplate modules/07-conductor-dispatch/conductor_enforcement.rs` | 40% (pattern) | enabled-gate semantics: when marker absent, ALL gated paths produce `NoOp`-equivalent (here: `compile_error!`) |
| `reqwest::blocking::Client` | std crate idiom | Build-time + runtime HTTP probe |
| F7 hard-fail logic | 0% (fresh) | ~15 LOC of band classification + refusal |

**Structural-gap LOC:** none. m8 owns no NEW PRIMITIVE; it is pure plumbing on top of standard Cargo + reqwest idioms. The intellectual contribution is the *placement* of the gate at compile-time rather than runtime.

---

## 8. Failure modes addressed

| ID | Mode | How m8 addresses |
|---|---|---|
| **W2** | Code reads POVM calibration data before CR-2 lands | `#[cfg(not(povm_calibrated))]` tombstones produce `compile_error!`; the read path cannot reach a binary. |
| **F7** | Feature gate misconfigured | `cargo:rustc-cfg` is NOT a `[features]` flag; `--features full`/`--all-features` cannot activate it. Defense-in-depth alongside runtime mirror. |
| **F3** | POVM reads silently return stale data | Runtime mirror `probe_band` at startup; refuses to run when out-of-band. No silent degrade. |
| **AP-V7-01** | Health-200 ≠ behaviour-verified | The probe reads `learning_health` value, not `/health` status. Behaviour-verified by definition. |
| **AP-V7-13** | Diagnostics theatre | Probes execute on every startup; refresh-stamps are derived from probe wall-time, never edited manually. |
| **AP-Drift-11** | Supervisor stub mistaken for live | Runtime probe verifies POVM actually responds with a magnitude-weighted value; never inferred from process-list. |
| **AP-Hab-14** | God-tier dilution | Hard-fail discipline; any commit softening band-check to a warning is a god-tier violation per [GOLD_STANDARDS rule 15](../../../GOLD_STANDARDS.md). |

---

## 9. Observability

`tracing` structured fields per [GOLD_STANDARDS rule 7](../../../GOLD_STANDARDS.md):

```rust
tracing::info!(
    target = "m8.health.probe",
    url = %probe_url,
    learning_health = value,
    band = ?classification,
    "POVM band probe complete"
);

tracing::error!(
    target = "m8.health.refuse",
    learning_health = value,
    band_low = POVM_LH_BAND_LOW,
    band_high = POVM_LH_BAND_HIGH,
    "startup refused: POVM substrate uncalibrated"
);
```

Metric: `m8_povm_band_classification` gauge (0=BelowFloor, 1=InBand, 2=AboveCeiling, 3=Nan) per [m05_metrics_collector](../../../the-workflow-engine-vault/deployment%20framework/phase-1-genesis-day-0-3.md) integration. Read by [`/sweep`](../../../CLAUDE.md) and the Watcher's Class-I monitor.

---

## 10. Pre-conditions / post-conditions

**Pre:** `POVM_CR2_DEPLOYED=1` set in build env (CI or operator-confirmed locally after `sqlite3 ~/.local/share/povm-v2.db "SELECT COUNT(*) FROM sessions WHERE fitness_delta IS NOT NULL"` returns > 0). Alternatively the runtime mirror probes a live POVM at `:8125` and may set the marker dynamically (rejected here — env-var-only per [cluster-D spec § m8](../../../the-workflow-engine-vault/module%20specs/cluster-D-trust-cross-cutting.md) for reproducibility).

**Post:** Either (a) build succeeds with `cfg(povm_calibrated)` emitted and gated POVM-reading paths compile, or (b) build fails with `compile_error!` naming SHA + env var + ref doc, or (c) startup refuses with `RuntimeBandError::StartupRefused` and process exits with code 78 (`EX_CONFIG`).

---

## 11. Watcher class pre-positions

| Class | Triggers when |
|---|---|
| **Class A** (activation) | First successful `cargo build` with `cfg(povm_calibrated)` emitted; first runtime probe returning `InBand`. |
| **Class F** (false-binding) | Cargo emits `cfg(povm_calibrated)` but the runtime probe returns `OutOfBand` — env-var-vs-substrate drift. |
| **Class I** (Hebbian silence) | `learning_health` drops below 0.05 mid-deploy; m8 runtime mirror refuses to run; cascades to m11 input-zero. |

WCP notify on Class F or Class I → file drop to `~/projects/shared-context/watcher-notices/` per [WCP convention](../../../CLAUDE.md).

---

## 12. Atuin trajectory anchor

Proposed atuin scripts (post-G9):
- `wt-build-status` — runs `cargo check` and greps for `cfg(povm_calibrated)` emission to surface band state.
- `povm-probe` (existing atuin script; m8 is the build-side counterpart).

History rows produced during normal authoring: `cargo check`, `POVM_CR2_DEPLOYED=1 cargo check`, `cargo expand --lib | grep povm_calibrated`. Queryable via `atuin search --workspace workflow-trace 'povm_calibrated'`.

---

## 13. Open questions

1. **Live runtime mirror vs env-var-only.** Current spec runs both. If operator policy hardens to env-var-only (CI-set), the runtime mirror becomes a defensive duplicate. Question for G7: keep both or drop runtime mirror once `POVM_CR2_DEPLOYED=1` is asserted by all deploy paths?
2. **Band thresholds [0.05, 0.15] are Phase 1 provisional** per Hebbian v3 reconciliation note. Phase 2 (>0.05 substrate-LTP-density) and Phase 3 (>0.10) thresholds are pending 30-day baseline (window opens 2026-05-17). Question for G7: should m8 band update automatically post-baseline, or require a v1.4 spec patch?
3. **`compile_error!` cannot reference doc paths with spaces** — the reference path contains spaces and em-dashes. The string in § 4 escapes correctly inside `compile_error!` but operators should verify rendering. Question for G7: ASCII-only fallback path acceptable?

---

> **Back to:** [`cluster-D/INDEX`](./) · [`ai_specs/INDEX`](../../INDEX.md) · [`MODULE_MATRIX`](../../MODULE_MATRIX.md) · vault [[cluster-D-trust-cross-cutting]] · [cluster-D plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-D.md) · [phase-1](../../../the-workflow-engine-vault/deployment%20framework/phase-1-genesis-day-0-3.md)
>
> **Sister modules (Cluster D):** [m8](m8_povm_build_prereq.md) · [m9](m9_watcher_namespace_guard.md) · [m10](m10_ember_ci_gate.md) · [m11](m11_fitness_weighted_decay.md)

*Spec authored 2026-05-17 (S1001982). HOLD-v2 active. No code, no Cargo, no scaffold until G1-G9 clear and Luke emits explicit start-coding signal.*
