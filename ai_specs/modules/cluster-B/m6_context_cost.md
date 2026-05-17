---
title: m6 — context_cost — Per-Module Spec
date: 2026-05-17 (S1001982)
status: planning-only · HOLD-v2 active · markdown-only · NO .rs files
cluster: B — Habitat Observation
layer: L2
module_id: m6
binary: wf-crystallise
verb_class: passive (record · emit · preserve)
loc_estimate: ~130
test_budget: 55
mutation_kill_threshold: 0.70
feature_gate: none
cc_owned: CC-1 (contributes via m7 join) · CC-1b (via session_id)
cc_consumed: m7 WorkflowOutcome (feedback for EMA classification)
gap_owner: F10 (exploration-cost preservation collapse) — exclusive
boilerplate_lift_pct: 30
status_row: SPEC
---

# m6 — `context_cost` — Per-Module Spec

> **Back to:** [CLAUDE.md](../../../CLAUDE.md) · [CLAUDE.local.md](../../../CLAUDE.local.md) · [MODULE_MATRIX.md](../../MODULE_MATRIX.md) · vault [[cluster-B-habitat-observers]] · canonical V7 [cluster-B plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-B.md) · binding spec [GENESIS_PROMPT_V1_3.md](../../../ai_docs/GENESIS_PROMPT_V1_3.md) § 1
>
> **Sister modules (Cluster B):** [m4](m4_cascade_correlator.md) · [m5](m5_battern_step_record.md) · [m6](m6_context_cost.md)
>
> **Cross-cluster anchors:** Cluster A upstream — [m1_atuin_consumer](../cluster-A/m1_atuin_consumer.md) · [m2_stcortex_consumer](../cluster-A/m2_stcortex_consumer.md) · Cluster C hub — [m7_workflow_runs](../cluster-C/m7_workflow_runs.md) (m6 ← m7 feedback for WorkflowOutcome) · Cluster D aspect-wrap — [m11_fitness_weighted_decay](../cluster-D/m11_fitness_weighted_decay.md) · Cluster F downstream — [m20_prefixspan_miner](../cluster-F/m20_prefixspan_miner.md)
>
> **Standards:** [PATTERNS.md](../../../PATTERNS.md) · [GOLD_STANDARDS.md](../../../GOLD_STANDARDS.md) · [ANTIPATTERNS.md](../../../ANTIPATTERNS.md)

---

## 1. Purpose

`m6_context_cost` records per-session token costs (proxied from stcortex tool-call rows) and correlates them with workflow arc outcomes stored in m7. Its load-bearing function — and the one that makes it the exclusive owner of **F10 (exploration-cost preservation collapse)** per [`ai_docs/optimisation-v7/ANTIPATTERNS_REGISTER.md`](../../../ai_docs/optimisation-v7/ANTIPATTERNS_REGISTER.md) § AP-WT-F10 — is carrying a **rolling exploration-rate baseline** that prevents downstream iteration (Cluster F) from collapsing all token budgets toward the minimum-cost workflow.

Without this baseline, downstream optimisation pressure toward lower-cost workflows would systematically eliminate the exploratory, high-token-cost sessions where novel approaches are discovered. The baseline is a **20-session EMA that explicitly EXCLUDES Converged and Repeated outcomes** — only Explored / Diverged / Unknown contribute. Including Converged would pull the baseline toward exploitation, defeating the F10 mitigation entirely. This exclusion is the F10 core.

Verb budget (Phase A passive, retained under override per [GENESIS_PROMPT_V1_3.md](../../../ai_docs/GENESIS_PROMPT_V1_3.md) § 3): **record · emit · preserve** (the baseline). m6 never recommends a cost target, never optimises toward cheaper sessions, never labels exploration as inefficient.

## 2. Public Surface

Lifted faithfully from the canonical cluster-B vault spec:

```rust
/// Per-session cost record.
#[derive(Debug, Clone)]
pub struct SessionCostRecord {
    pub session_id: String,
    /// Proxy cost: count of read/bash/grep/glob tool calls.
    /// Doc: "proxy: tool-call count, not token count" (until stcortex extension).
    pub token_cost_input_proxy: i64,
    /// Proxy cost: count of write/edit tool calls.
    pub token_cost_output_proxy: i64,
    pub total_cost_proxy: i64,
    /// WorkflowOutcome read back from m7 join; None if session not yet in m7.
    pub outcome: Option<WorkflowOutcome>,
    /// Exploration-rate baseline EMA at the time this record was created.
    /// None during bootstrap (N < baseline_bootstrap_n).
    pub exploration_baseline: Option<f64>,
    pub cost_band: Option<CostBand>,
    pub recorded_at_ms: i64,
}

/// Workflow outcome classification for baseline EMA filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowOutcome {
    Converged,
    Repeated,
    Explored,
    Diverged,
    Unknown,
}

impl WorkflowOutcome {
    /// F10 invariant: only Explored / Diverged / Unknown contribute to baseline EMA.
    /// Converged and Repeated represent exploitation and are excluded.
    pub fn is_exploration(&self) -> bool {
        matches!(self, Self::Explored | Self::Diverged | Self::Unknown)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CostBand {
    BelowBaseline,
    NearBaseline,
    AboveBaseline,
}

/// Running state for the exploration-rate baseline EMA.
#[derive(Debug, Clone)]
pub struct ExplorationBaseline {
    pub ema: Option<f64>,
    pub n: usize,
    pub alpha: f64,    // alpha = 2 / (window + 1)
}

impl ExplorationBaseline {
    pub fn new(window: usize) -> Self;

    /// Update the EMA with a new exploration-session cost.
    /// No-op if `outcome.is_exploration() == false` (F10 invariant).
    pub fn update(&mut self, cost: i64, outcome: WorkflowOutcome);

    /// Classify a cost relative to the current baseline.
    /// Returns None during bootstrap (n < baseline_bootstrap_n).
    pub fn classify(
        &self,
        cost: i64,
        below_threshold: f64,
        above_threshold: f64,
    ) -> Option<CostBand>;
}

#[derive(Debug, Clone)]
pub struct ContextCostRecordConfig {
    pub baseline_ema_window: usize,          // default 20
    pub baseline_bootstrap_n: usize,         // default 5
    pub below_threshold: f64,                // default 0.8
    pub above_threshold: f64,                // default 1.2
    /// stcortex JSON snapshot path; used as fallback if live :3000 unreachable.
    /// Per CLAUDE.md memory row 8: "if :3000 unreachable, read snapshot and SKIP WRITES;
    /// do NOT silently fall back to POVM".
    pub stcortex_snapshot_path: Option<std::path::PathBuf>,
}

pub struct ContextCostRecord {
    config: ContextCostRecordConfig,
    baseline: std::sync::Mutex<ExplorationBaseline>,
}

impl ContextCostRecord {
    pub fn new(config: ContextCostRecordConfig) -> Self;

    /// Read tool_call rows from stcortex for a session_id and compute proxy costs.
    /// # Errors
    /// `ContextCostError::SubstrateUnreachable` if neither live DB nor snapshot accessible.
    pub fn read_session_costs(
        &self,
        session_id: &str,
    ) -> Result<SessionCostRecord, ContextCostError>;

    /// Record a session cost and update the exploration-rate baseline if the session
    /// outcome qualifies as exploration. Returns the updated record with baseline + cost_band.
    pub fn record_and_update_baseline(
        &self,
        record: SessionCostRecord,
    ) -> SessionCostRecord;

    /// Point-in-time snapshot of the current exploration-rate baseline.
    #[must_use]
    pub fn baseline_snapshot(&self) -> ExplorationBaseline;
}

#[derive(Debug, thiserror::Error)]
pub enum ContextCostError {
    #[error("stcortex substrate unreachable: live :3000 + snapshot both failed")]
    SubstrateUnreachable,
    #[error("stcortex snapshot parse: {0}")]
    SnapshotParse(#[from] serde_json::Error),
    #[error("stcortex io: {0}")]
    StcortexIo(#[from] std::io::Error),
}
```

`#[must_use]` on `baseline_snapshot()` because the snapshot is a point-in-time clone, not a live handle. The `Mutex` interior mutability around `ExplorationBaseline` allows concurrent reads from multiple sweep iterations without full struct cloning.

## 3. Internal Data Structures

**`ExplorationBaseline`:** holds the running EMA state — `ema: Option<f64>`, `n: usize`, `alpha: f64`. Protected by `Mutex` in `ContextCostRecord` so concurrent sweep iterations may safely call `read_session_costs` + `record_and_update_baseline` from independent tasks. `RwLock` was considered but rejected: `update()` mutates, and the lock is held briefly enough that contention is not a concern at habitat scale (≤100 sessions/day).

**EMA update formula:**

```text
alpha = 2 / (window + 1)            (default window=20 → alpha ≈ 0.0952)
ema(t) = alpha * cost(t) + (1 - alpha) * ema(t-1)
n(t) = n(t-1) + 1                   (only if outcome.is_exploration())
ema returns None until n >= baseline_bootstrap_n (default 5)
```

**Cost-band classification:**

| Band | Condition |
|---|---|
| `BelowBaseline` | `total_cost < ema * below_threshold` (default `< ema * 0.8`) |
| `NearBaseline` | `ema * 0.8 <= total_cost < ema * 1.2` |
| `AboveBaseline` | `total_cost >= ema * 1.2` |
| `None` | Bootstrap period (`n < baseline_bootstrap_n`) |

The `cost_band` carries NO judgment about which band is good or bad. It is a descriptive observation. Downstream readers (Cluster F m31 selector) may treat `AboveBaseline` exploration cost as evidence-of-exploration to preserve under selection pressure — but that judgment lives in m31, not m6. F10 mitigation is the absence-of-judgment here.

**stcortex access discipline (per [CLAUDE.md](../../../CLAUDE.md) § Memory Systems row 8):**
- Primary: live `:3000` SpacetimeDB query (read-only).
- Fallback: `~/claude-code-workspace/stcortex/data/snapshots/latest.json` if `:3000` unreachable, AND skip baseline writes for this iteration.
- **Forbidden:** silent fallback to POVM. workflow-trace is POVM-decoupled per 2026-05-17 m42 ADR ([`ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md`](../../../ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md)).

**Token cost proxy:** stcortex `consumption_event` rows don't carry token counts directly. m6 proxies via tool-call type counts:
- `token_cost_input_proxy = count(read | bash | grep | glob)` for this session
- `token_cost_output_proxy = count(write | edit)` for this session

Doc comments on both fields explicitly read `proxy: tool-call count, not token count`. When stcortex extends `claude_session` with real `usage_input_tokens` / `usage_output_tokens` fields, m6 upgrades behind a `feature = "precise_token_costs"` gate. See § 12 Q1.

## 4. Data Flow

```text
stcortex :3000 (live, primary)
    OR
stcortex/data/snapshots/latest.json (fallback per CLAUDE.md row 8 discipline)
    | read_session_costs()
    |   - filter tool_call rows by session_id
    |   - compute proxy costs (read|bash|grep|glob vs write|edit)
    v
SessionCostRecord (raw — outcome may be None if session not yet classified)
    | record_and_update_baseline()
    |   - if outcome.is_exploration() -> baseline.update(cost, outcome)
    |   - baseline.classify(cost) -> cost_band
    v
SessionCostRecord (with exploration_baseline + cost_band populated)
    v
Cluster C — m7_workflow_runs writer (CC-1 owner; writes token_cost_*, cost_band,
            exploration_baseline columns into workflow_runs)
    v
Cluster F — m20/m21/m22/m23 iterators (read cost_band as evidence axis)
    v
Cluster E — m14_habitat_outcome_lift (lift computation per cost_band stratum)
```

m6 **does NOT write to any database directly.** It computes and returns `SessionCostRecord`; m7 owns the SQLite write path; m13/m42 own the stcortex emit path. The m6 ↔ m7 relationship is bidirectional in data flow (m6 reads outcome from m7's prior classification, then m7 writes m6's cost columns), but only via shared join keys — no struct-pointer coupling.

## 5. Boilerplate Lifts

Per [`the-workflow-engine-vault/boilerplate modules/BOILERPLATE_INDEX.md`](../../../the-workflow-engine-vault/boilerplate%20modules/BOILERPLATE_INDEX.md):

| Source | LOC reused | Pattern lifted | Notes |
|---|---:|---|---|
| ME v2 telemetry EMA helper (`ema_update(prev, sample, alpha) -> f64`) | ~15 | EMA primitive | ~70% reuse; m6 extends with variance accumulator and bootstrap gate. |
| `m39_fitness_tensor.rs` (category 05) | ~10 | Rolling-mean smoothing pattern + volatile-dimension mask concept | Informs bootstrap gate (`n >= baseline_bootstrap_n` before EMA considered valid). |
| `memory-injection/m07_causal_chain.rs::parse_chain_type` | ~8 | Enum-parse pattern (informs `WorkflowOutcome` parse) | ~50% reuse. |
| `ws_inbound_writer.rs` hourly TTL sweep pattern (category 05) | ~5 | Session-level retention concept (keep N sessions of cost data) | Light influence; not directly code-lifted. |

**Boilerplate lift density: ≈30%.** Fresh authorship dominates the F10 gate + session-type bucketing + the stcortex snapshot fallback discipline.

## 6. ME v2 Patterns

Per [PATTERNS.md](../../../PATTERNS.md) § "Module-level patterns (per ME v2 m1_foundation)":

- **`resources.rs` `//!` docstring style:** module header carries Layer / Deps / Tests / Features / Platform / Impl Notes / Related Docs.
- **`error.rs` thiserror taxonomy:** `ContextCostError` enum; no `Box<dyn Error>` exposed.
- **`tensor_registry.rs` 12D-tensor framing (light):** `CostBand` is a single cost-axis dimension; downstream m22 K-means feature clusterer composes it with m14 lift dimensions.
- **`self_model.rs` engine-knows-about-its-own-state:** `baseline_snapshot()` exposes the EMA's internal state read-only; downstream m31 selector reads it to inform diversity-weighted scoring.
- **Mutex-protected interior mutability:** `ExplorationBaseline` wrapped in `Mutex` analogous to `RwLock` in ME v2 modules (mutation cadence too high for `RwLock` to pay off).
- **`#[must_use]` discipline:** `baseline_snapshot()` is marked because a discarded snapshot has no meaning.
- **Predicate methods on domain enums:** `WorkflowOutcome::is_exploration()` mirrors ME v2's `TaskNodeState::is_terminal()` pattern.
- **`logging.rs` tracing-subscriber emit:** snapshot fallback emits `tracing::warn!` with structured `{snapshot_path, live_endpoint}` — not `println!`.

## 7. Constraints Satisfied

- **F10 exploration-cost preservation (exclusive owner) — HIGH:** the EMA exclusively tracks exploration-classified sessions. `WorkflowOutcome::is_exploration()` is the type-level gate; the `if outcome.is_exploration()` branch in `ExplorationBaseline::update()` is the runtime gate. Converged and Repeated are structurally excluded. Property-tested: `gate(gate(x)) == gate(x)` (idempotence); 100 Converged sessions in a row do not move `ema` or `n`; baseline drifts only on exploration cohorts.
- **F11 opaque-ID discipline (consumed):** m6's `session_id` is opaque (UUID from stcortex `claude_session`). No human-meaningful label leaks into the record.
- **stcortex fallback discipline ([CLAUDE.md](../../../CLAUDE.md) row 8):** if `:3000` unreachable, read snapshot and SKIP writes; NEVER silently fall back to POVM. Encoded in `ContextCostRecordConfig::stcortex_snapshot_path` and the `read_session_costs` error path. POVM-decoupled per 2026-05-17 m42 ADR.
- **Phase A passive verbs (retained under override per § 1.b):** record, emit, preserve (baseline). Never recommend, never optimise toward lower cost.
- **No direct DB write:** m6 computes and returns `SessionCostRecord`; m7 owns the SQLite write; m13/m42 own the stcortex emit.
- **AP30 namespace discipline:** propagated `session_id` is aspect-wrapped by m9 at the m13/m42 boundary.
- **F9 fitness-dimension zero-vs-null distinction:** `exploration_baseline: Option<f64>` distinguishes "bootstrap — no signal yet" from "signal = 0.0". `cost_band: Option<CostBand>` mirrors. The m7 hub's `fitness_dimension REAL NOT NULL DEFAULT 0.0` schema complements this with the column-default-vs-NULL distinction.
- **God-tier rules ([GOLD_STANDARDS.md](../../../GOLD_STANDARDS.md)):** zero `unwrap()` outside tests; zero `unsafe`; thiserror enum; doc comments on all public items; structured tracing emit; newtype discipline (where applicable — `f64` cost is a scalar, not newtype-promoted yet).

## 8. Tests (≥55, per TEST_DISCIPLINE matrix row m6)

Allocation per [`ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md`](../../../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md) and cluster-B V7 plan:

| Pattern | Count | Focus |
|---|---:|---|
| Unit | 28 | EMA initial-value handling; EMA update per `n`; bootstrap gate (n<5 → None; n=5 → Some); F10 gate per outcome arm (5 outcomes × in/out of baseline); `WorkflowOutcome::is_exploration()` truth table; cost-band per condition (Below/Near/Above); cost-band boundary inclusivity (0.8 → Near; 1.2 → Above); proxy cost computation (read|bash|grep|glob vs write|edit); `SessionType` parse per known arm; empty input. |
| Property | 6 | (a) `ema_mean ∈ [min_input, max_input]` for all finite inputs; (b) variance ≥ 0; (c) EMA converges to true mean as n→∞ on stationary input; (d) F10 gate idempotent: `gate(gate(x)) == gate(x)`; (e) 100 Converged sessions in a row leave ema/n unchanged; (f) `cost_band == None` iff `exploration_baseline == None` (jointness invariant). All property tests run ≥10k iterations per [`ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md`](../../../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md) § Property-test pattern. |
| Fuzz | 0 | (no fuzz target this module; F10 protection lives in F-Property + F-Regression). |
| Integration | 15 | m6 ↔ m2 stcortex live read; m6 stcortex snapshot fallback (live :3000 down); m6 both substrates absent (returns SubstrateUnreachable, not panic); full `record_and_update_baseline` cycle; m6 → m7 join (cost_band + exploration_baseline columns populated); concurrent `baseline_snapshot` reads return consistent values; multi-session-type interleave; Converged/Explored/Diverged mix end-to-end; bootstrap → post-bootstrap transition; stcortex consumer-freshness gate (refuse-write at DB layer); POVM fallback FORBIDDEN test (assertion that POVM endpoint is never contacted); namespace prefix validation downstream; m6 ← m7 outcome feedback loop. |
| Contract | 3 | `SessionCostRecord` schema snapshot (insta); `ExplorationBaseline` schema snapshot; `ContextCostError` variant snapshot. |
| Regression | 3 | F10 regression slot (any commit including Converged in baseline EMA); session-type bucketing regression; bootstrap-gate threshold regression. |
| **Mutation budget** | — | **≥70% kill** on F10 gate path (`gate.rs`) and EMA update (`ema.rs`). |

Every test carries `// rationale: F10 — …` or `// rationale: F9 — …` per [`ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md`](../../../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md).

## 9. Cross-Cluster Contracts

- **CC-1 Cascade-Cost Coupling (m6 contributes; m7 owns the join):** m6 emits `SessionCostRecord` with `token_cost_input_proxy`, `token_cost_output_proxy`, `exploration_baseline`, `cost_band`; m7 writes these columns into `workflow_runs` keyed by `session_id`. m4's `cascade_cluster_id` joins via the same key. m4 and m6 NEVER directly coupled.
- **CC-1b Battern-Cost Coupling (m5 ↔ m6 via session_id within battern_id range):** m5's `battern_observations` rows carry `(battern_id, session_id)`; m6's per-session cost joins via `session_id`. Cluster F's m21 battern_iterator does the join through m7. m5 and m6 NEVER directly coupled.
- **CC-2 Trust Layer Woven (aspect-IN):** `m8_povm_build_prereq` compile-time gate (workflow-trace is POVM-decoupled, but the build-prereq cfg `povm_calibrated` still fires at habitat scope); `m9_watcher_namespace_guard` validates `workflow_trace_*` prefix on payloads at m13/m42 boundary.
- **m6 ← m7 feedback loop:** `WorkflowOutcome` is read back from m7's arc-outcome classification (computed by m7 after a workflow run completes). m6's `record_and_update_baseline` uses this to decide whether to update the EMA. First-sweep behaviour: outcomes are `Unknown` (counts toward exploration per `is_exploration()`); intentional bootstrap behaviour flagged in module doc.

## 10. Failure Modes

- **F10 exploration-cost preservation collapse (exclusive owner) — HIGH:** any commit that includes Converged/Repeated in the baseline EMA pulls the baseline toward exploitation, causing downstream m31 selection to optimise away exploration. Mitigation: `is_exploration()` type-level gate; `update()` runtime gate; F-Property idempotence test; F-Regression slot; mutation kill ≥70% on `gate.rs`.
- **F9 workflow-grain fitness distortion — MEDIUM:** confusing "no signal yet" with "signal = 0.0" inflates Cluster F lift estimates. Mitigation: `Option<f64>` for `exploration_baseline`; `Option<CostBand>` for `cost_band`; F-Property invariant ensures jointness.
- **F7 stcortex silent POVM fallback — RESOLVED (per 2026-05-17 ADR):** workflow-trace is POVM-decoupled; m6 has no POVM code path. Integration test asserts POVM endpoint never contacted.
- **AP-Test-02 property-test stub — LOW:** property tests that don't actually exercise the invariant. Mitigation: per [`ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md`](../../../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md) § Property-test pattern, each property test runs ≥10k iterations and asserts a non-trivial relation.
- **AP-V7-13 diagnostics theatre — LOW:** `:3000` returning HTTP 200 but serving stale data. Mitigation: schema-version probe on first read (paired with the m6 doc comment "probes re-verified live <DATE>" discipline per [feedback_runbook_probe_freshness](../../../.claude/projects/-home-louranicas-claude-code-workspace/memory/feedback_runbook_probe_freshness.md)).
- **EMA window misalignment (multi-week real time per § 12 Q2):** if exploration sessions are rare (5/week), the 20-session window spans ~4 calendar weeks. Watcher Class H pre-positioned for atuin-proprioception drift.

## 11. LOC Estimate

| Section | LOC |
|---|---:|
| Types (`SessionCostRecord`, `WorkflowOutcome`, `CostBand`, `ExplorationBaseline`, `ContextCostRecordConfig`) | ~55 |
| `ExplorationBaseline::{new, update, classify}` | ~25 |
| `ContextCostRecord::{new, read_session_costs, record_and_update_baseline, baseline_snapshot}` | ~35 |
| `ContextCostError` enum + config helpers | ~15 |
| **Total** | **~130** |

src/ layout per [`ai_docs/optimisation-v7/GENERATIONS/G2-consolidation.md`](../../../ai_docs/optimisation-v7/GENERATIONS/G2-consolidation.md): `src/m6_cost/{mod.rs, ema.rs, gate.rs, band.rs, error.rs}`. Unpadded module ID.

## 12. Open Questions

1. **Token cost proxy upgrade path:** tool-call count is a rough proxy. When stcortex extends `claude_session` with real `usage_input_tokens` / `usage_output_tokens`, m6 should read them directly behind a `feature = "precise_token_costs"` gate. **Decision needed (Luke / Watcher):** is there a target session for the stcortex extension, or is m6 expected to ship M0 with the proxy and upgrade post-soak?
2. **EMA window vs calendar time:** 20-session EMA is design default. If habitat runs 100 sessions/day but only 5 are exploratory, the 20-session window spans ~4 calendar weeks. **Decision needed (Watcher):** keep session-anchored window (simpler, deterministic), or move to time-anchored (e.g., "EMA over last 30 calendar days of exploration sessions") which better tracks substrate drift? Recommended: session-anchored for v1; revisit at D60 soak.
3. **`Unknown` outcome counting toward exploration:** first-sweep behaviour treats every session as `Unknown` (no m7 classification yet); `is_exploration()` returns `true` for `Unknown`. This intentionally bootstraps the baseline from raw activity. **Watcher flag:** Class A pre-position for first `ContextCostBand` emit; verify the baseline stabilises within 5-10 sessions once m7 starts classifying.
4. **Concurrent baseline reads vs Mutex contention:** `Mutex` chosen over `RwLock` because update cadence matches read cadence at habitat scale. **Decision needed (Zen / Luke):** at >1000 sessions/day, contention may justify `RwLock` or atomic-snapshot pattern; flag for benchmarking post-G9.

## 13. Bidirectional Anchors

> **Back to:** [CLAUDE.md](../../../CLAUDE.md) · [CLAUDE.local.md](../../../CLAUDE.local.md) · [MODULE_MATRIX.md](../../MODULE_MATRIX.md)
>
> **Sister modules (Cluster B):** [m4](m4_cascade_correlator.md) · [m5](m5_battern_step_record.md) · [m6](m6_context_cost.md)
>
> **Vault canonical:** [[cluster-B-habitat-observers]] (~/claude-code-workspace/the-workflow-engine/the-workflow-engine-vault/module specs/cluster-B-habitat-observers.md)
>
> **V7 planning:** [cluster-B plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-B.md) · [ULTRAMAP.md](../../../ai_docs/optimisation-v7/ULTRAMAP.md) View 2 row m6 · [TEST_DISCIPLINE.md](../../../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md) · [GOD_TIER_RUST.md](../../../ai_docs/optimisation-v7/STANDARDS/GOD_TIER_RUST.md) · [ANTIPATTERNS_REGISTER.md](../../../ai_docs/optimisation-v7/ANTIPATTERNS_REGISTER.md) § AP-WT-F10
>
> **Binding spec:** [GENESIS_PROMPT_V1_3.md](../../../ai_docs/GENESIS_PROMPT_V1_3.md) § 1 (architecture) · § 3 (verb-class)
>
> **m42 ADR (POVM decoupling):** [`ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md`](../../../ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md)
>
> **Substrate anchor:** [[stcortex — Pioneer Capability Dossier 2026-05-10]] (workflow-trace's exclusive substrate-feedback target)
>
> **Standards mirror:** [PATTERNS.md](../../../PATTERNS.md) · [GOLD_STANDARDS.md](../../../GOLD_STANDARDS.md) · [ANTIPATTERNS.md](../../../ANTIPATTERNS.md)
>
> **Watcher class pre-position:** Class A (first `ContextCostBand` emit post-Genesis) · Class G (substrate-frame confusion if `ema_mean` back-interpreted as user-effort rather than context-window cost) · Class H (atuin proprioception anomaly if m6 per-session cost diverges >2σ from atuin per-command token totals)

*m6 spec v1 · authored 2026-05-17 (S1001982) · planning-only · HOLD-v2 active · no .rs files emitted*
