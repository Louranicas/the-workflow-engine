---
title: cross-cutting/substrate-drift — first-class substrate-drift detection
date: 2026-05-17
status: SPEC
axes: [substrate-drift, ap-v7-13, observability, canary-detection]
authority: Luke @ node 0.A — S1002127 "as per proposal"
hold_v2_compliant: true
addresses: [NA-GAP-07]
---

# Substrate Drift — First-Class Detection

> **Back to:** [`../INDEX.md`](../INDEX.md) · [`../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md) · [`./refusal-taxonomy.md`](refusal-taxonomy.md) · [`../substrates/`](../substrates/) (8 dossiers) · [`../../ANTIPATTERNS.md`](../../ANTIPATTERNS.md) AP-V7-13 · [`../../ultramap/INVARIANT_MAP.md`](../../ultramap/INVARIANT_MAP.md) § AP-V7-13 mitigation rows

## Purpose

The scaffold catalogues AP-V7-13 (Health-200 ≠ behaviour-verified) as a single antipattern with two known instantiations: m32 (Conductor semantic-endpoint check) and m42 (CC-5 substrate-delta external verification). Per NA-GAP-07, this is **scattered defensive sub-clauses**, not a first-class detector. The CR-2 `learning_health` 13.6× inflation factor was caught by Luke spot-checking the math, not by the engine — and the scaffold as written does not change that.

This spec elevates substrate-drift detection to a first-class cross-cutting axis with:

1. A **canary contract** every substrate dossier participates in.
2. A **`SubstrateDriftDetected` event** emitted on detection.
3. Wiring into **m15 pressure register** (planning-only — m15 is the canonical pressure event surface).
4. Optional **`m16_substrate_drift_canary`** module reserved for v0.2.0 (NA-GAP-07 recommendation; defer per § 5 of the NA gap analysis).

## The canonical case: CR-2 POVM `learning_health` inflation

The POVM `learning_health` formula changed between binary versions:
- **Pre-CR-2 (binary-aggregation):** `health = (pathways_with_any_write / total_pathways)` → returned `0.9146` because most pathways had at least one write event.
- **Post-CR-2 (magnitude-weighted):** `health = (pathways_with_weight_gt_0.5 / total_pathways)` → returned `0.067` on the same raw data — a **13.6× inflation factor** in the pre-fix reading.

Throughout the formula change, `:8125/health` returned **HTTP 200**. The substrate was Available. It was not Refusing. It was not Unavailable. It was **drifted** — semantically returning a different number than the engine expected, with no observable signal in the standard health-probe surface. This is the seed of AP-V7-13 and the canonical case for substrate-drift.

Per the workspace `CLAUDE.local.md` reconciliation note (Watcher-authored 2026-05-17, Zen-audited PASS-WITH-MINOR-AMEND), the substrate-stock metric `substrate_LTP_density > 0.015` replaces `learning_health > 0.3` as the Phase 1 threshold. The reconciliation is a *substrate-drift response* — the engine adopted a different metric to side-step the drifted one.

## Drift canary contract

Every substrate dossier (`../substrates/<id>.md` § 4 Drift indicators) MUST enumerate its drift indicators. The canary contract is the engine-side mechanism that watches these indicators:

```rust
// SPEC ONLY
pub trait SubstrateDriftCanary {
    /// Substrate identity.
    fn substrate_id(&self) -> SubstrateId;

    /// Frozen baseline snapshot taken at engine startup or after explicit re-baseline.
    fn baseline(&self) -> &SubstrateBaseline;

    /// Measure current substrate state.
    fn measure(&self) -> Result<SubstrateMeasurement, MeasureError>;

    /// Compute drift between baseline and measurement.
    fn drift(&self, baseline: &SubstrateBaseline, measurement: &SubstrateMeasurement) -> Drift;

    /// Threshold beyond which a `SubstrateDriftDetected` event fires.
    fn threshold(&self) -> DriftThreshold;
}

pub struct SubstrateBaseline {
    pub substrate_id: SubstrateId,
    pub baseline_at: i64,
    pub schema_hash: u64,             // structural hash of substrate's exposed surface
    pub semantic_probe_result: f64,   // e.g. learning_health on a frozen test pathway set
    pub envelope_hash: u64,           // wire-envelope structure hash
}

pub struct Drift {
    pub schema_hash_changed: bool,
    pub semantic_probe_delta: f64,   // |measurement - baseline|
    pub envelope_hash_changed: bool,
    pub severity: DriftSeverity,     // None | Minor | Major | CR2Class
}

pub enum DriftSeverity {
    None,
    Minor,        // schema delta within tolerated range
    Major,        // schema/envelope changed; engine should re-handshake
    CR2Class,     // semantic probe delta exceeds 5× factor (the inflation-incident class)
}
```

## Drift indicators by substrate (cross-reference to dossiers)

Each substrate dossier § 4 carries the substrate-specific drift indicators:

| Substrate | Drift indicators (summary) |
|---|---|
| [`S-A atuin`](../substrates/atuin.md#4-drift-indicators-closes-na-gap-07) | schema migration silent, cursor semantic change, WAL checkpoint cadence change, KV namespace relocation |
| [`S-B injection.db`](../substrates/injection_db.md#4-drift-indicators-closes-na-gap-07) | schema migration silent, `reinforcement_count` semantic shift, TTL window change, test timestamp sweep |
| [`S-C stcortex`](../substrates/stcortex.md#4-drift-indicators-closes-na-gap-07-canonical-case) | **CANONICAL** — `learning_health` formula change, pathway weight semantic shift, reducer signature change, namespace relocation, snapshot export format change |
| [`S-D conductor`](../substrates/conductor.md#4-drift-indicators-closes-na-gap-07) | dispatch envelope change, enforcement-flag semantics shift, wave promotion silent, banner format change |
| [`S-E synthex`](../substrates/synthex.md#4-drift-indicators-closes-na-gap-07) | NexusEvent envelope change, Watcher class redefinition, wire-protocol version bump, Hebbian coordinator formula change |
| [`S-F lcm`](../substrates/lcm.md#4-drift-indicators-closes-na-gap-07) | RPC method rename, supervisor state-machine change, deploy-contract drift patch, TierExecutor signature change |
| [`S-watcher`](../substrates/watcher.md#4-drift-indicators-closes-na-gap-07) | persona redefinition, class taxonomy change, observation DB schema change, AP27 boundary drift |
| [`S-G operator`](../substrates/operator.md#4-drift-indicators-closes-na-gap-07) | EscapeSurfaceProfile banner familiarity-blindness, dispatch-density normalisation, frame-switching cost increase, prompt text drift |

The operator's drift is the most subtle and the most operationally important — the substrate is "200 OK" (responding to banners) while semantically drifting (no longer reading them).

## SubstrateDriftDetected event

When a canary's `drift().severity >= Minor`, the engine emits:

```rust
// SPEC ONLY
pub struct SubstrateDriftDetected {
    pub substrate_id: SubstrateId,
    pub indicator: DriftIndicator,         // e.g. "learning_health_formula_change"
    pub baseline_value: serde_json::Value,
    pub measurement_value: serde_json::Value,
    pub severity: DriftSeverity,
    pub detected_at: i64,
    pub recommended_action: Cow<'static, str>,
}
```

Routing:
- **CR2Class** (semantic probe delta > 5×): immediate `WireEvent::Refusal { ... }` + m15 pressure event + watcher-notice file drop + halt of dependent dispatches.
- **Major**: m15 pressure event + watcher-notice; engine continues with breaker HALF_OPEN.
- **Minor**: m15 pressure event only; informational.
- **None**: no emission.

## Wiring into m15 pressure register

m15 is the canonical pressure-event surface (per [`../modules/cluster-E/`](../modules/cluster-E/) m15 spec). Substrate-drift events ARE pressure events — substrate semantics changing under the engine's feet IS pressure. The wiring:

- Each substrate dossier's canary writes to m15's JSONL pressure register on drift detection.
- m15's existing dedup window (60s, same kind+context) applies — repeated drift detections on the same indicator coalesce.
- Operator surfaces (m12 report) include drift events in the pressure summary.

## Frozen baseline lifecycle

The baseline MUST be:
- **Captured at engine startup** by probing each substrate's exposed surface (schema, semantic probe, envelope).
- **Persisted** to `~/.local/share/workflow-trace/baselines/<substrate_id>.json` so cold restarts inherit the baseline.
- **Explicitly re-baselined** only on operator directive (`workflow-trace baseline reset <substrate_id>`) — never silently.
- **Versioned** with the substrate's reported version (if any) so the baseline can be invalidated when the engine knows a substrate upgrade happened.

The re-baseline operator directive is intentionally explicit because automatic re-baselining would hide CR-2-class drift. The operator must consciously accept that the substrate has changed.

## Test fixtures (closes part of NA-GAP-08 via NA-GAP-07)

At `tests/substrate_fixtures/<substrate>/`:
- **`cr2_inflation_fixture`** (stcortex) — emulator returns pre-CR-2 magnitude on `learning_health`; asserts canary fires CR2Class.
- **`schema_hash_drift_fixture`** (each substrate) — emulator returns altered schema; asserts canary fires Major.
- **`semantic_probe_minor_drift_fixture`** (each substrate) — emulator returns slightly altered probe value; asserts canary fires Minor.
- **`no_drift_steady_state_fixture`** (each substrate) — emulator stable; asserts canary stays silent.

## v0.2.0 deferral: `m16_substrate_drift_canary` module

Per NA gap analysis § 5 item 5, the dedicated `m16_substrate_drift_canary` module is **DEFERRED to v0.2.0**. This cross-cutting spec captures the contract; the v0.1.0 implementation distributes drift detection across the existing substrate-touching modules (m1, m2, m3, m32, m40, m41, m42) with the canary trait. v0.2.0 may extract the implementation into a dedicated module. See [`../../ai_docs/decisions/`](../../ai_docs/decisions/) for the deferral ADR (planned).

## Invariants

| # | Invariant | Enforcement |
|---|---|---|
| 1 | Every substrate dossier § 4 enumerates drift indicators | linter on substrate dossier frontmatter |
| 2 | Baseline persisted before first canary measurement | engine-startup integration test |
| 3 | Re-baseline only via explicit operator directive | no `auto_rebaseline()` function exists in spec |
| 4 | CR2Class severity halts dependent dispatches | integration test: simulate CR-2 fixture, assert halt |
| 5 | Minor severity does NOT halt | integration test |

---

> **Back to:** [`../INDEX.md`](../INDEX.md) · [`../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md) · [`../substrates/`](../substrates/) · [`./refusal-taxonomy.md`](refusal-taxonomy.md)

*Filed 2026-05-17 (S1002127 · Wave 4 NA-remediation) · Luke "as per proposal" · planning-only · HOLD-v2 compliant.*
