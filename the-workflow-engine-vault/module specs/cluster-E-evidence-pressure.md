---
title: Cluster E — Evidence + Pressure (m14 + m15)
date: 2026-05-17 (S1001982)
kind: module-spec
status: planning-only · HOLD-v2 active · single-phase deployment
authority: Luke @ node 0.A
cluster: E
modules: [m14, m15]
loc_est: ~200 total (m14 ~120 · m15 ~80)
binary: wf-crystallise
---

# Cluster E — Evidence + Pressure

> Back to: [[HOME]] · [[MASTER_INDEX]] · [[Modules Synergy Clusters and Feature Verification S1001982]] · [[Genesis Prompt v1.2 S1001982]] · `~/claude-code-workspace/the-workflow-engine/` · [[CLAUDE.md]] · [[CLAUDE.local.md]]

**Cluster E is the meta-layer of the engine.** m14 measures whether the engine is producing real value in the habitat — the empirical spine of the m11 sunset decision and the m31 selection gradient. m15 tracks every attempt to expand the engine's scope beyond its chartered verb-set — the institutional immune system against the Fossil's ancestor-rhyme prediction that ambition-shaped codebases die.

Neither module generates workflows, selects them, or dispatches them. Both observe the engine observing the habitat.

---

## Cluster overview

| Module | Name | Job | LOC est. | Binary |
|---|---|---|---|---|
| **m14** | `evidence_aggregator` | Aggregate `workflow_runs` from m7; compute habitat-outcome-lift; emit to m11 + m31 | ~120 | `wf-crystallise` |
| **m15** | `pressure_register` | Detect and log forbidden-verb-pressure events; emit `PHASE-B-RESERVATION-NOTICE` to `agent-cross-talk/` | ~80 | `wf-crystallise` |

---

## Cross-cluster contracts

### Inputs to Cluster E

| Source | Data | Module |
|---|---|---|
| m7 `workflow_arc_record` | `workflow_runs` rows: `(workflow_id, session_id, outcome, cost_tokens, decision_cost_baseline, cascade_success, ts_ms)` | m14 |
| m31 `selector` | Selection weights read-back (advisory; m14 does not depend on m31 at runtime) | m14 |
| Anywhere — spec patches, agent reports, cross-talk content, CLI verb proposals | Pressure event trigger signals | m15 |

### Outputs from Cluster E

| Destination | Data | Source module |
|---|---|---|
| m11 `engine_sunset_lifecycle` | `habitat_outcome_lift: f64` scalar + rolling window CI | m14 |
| m31 `selector` | Per-workflow lift-contribution scores → selection weight gradient | m14 |
| m42 `hebbian_feedback` | Workflow IDs with positive lift delta → LTP signal; negative → LTD | m14 |
| `agent-cross-talk/` directory | `PHASE-B-RESERVATION-NOTICE` JSONL files | m15 |
| Watcher ☤ + Zen | Observer of m15 notices (read-only via shared filesystem) | m15 |

### CC-3 — Evidence-Driven Iteration (E → F)

m14's `habitat_outcome_lift` metric is the gate signal for Cluster F iterators (m20-m22). Iterators only propose variants in domains where m14 has observed at least n≥20 runs with statistically significant lift or loss. An iterator that operates on a domain with n < 20 runs or a CI spanning zero is violating F2 (the sample-size hard gate from [[Genesis Prompt v1.2 S1001982]]).

Concretely: m20 (`cascade_iterator`) reads m14's per-cascade-cluster lift scores before proposing variants. If the cluster has n < 20 or CI spans zero, m20 emits a `NEEDS-MORE-DATA` report rather than a proposal.

### CC-7 — Pressure-Driven Evolution (E → spec interviews)

m15's `PHASE-B-RESERVATION-NOTICE` files are the durable signal that scope-pressure has occurred. They are not gating — they are informational. Watcher and Zen read `agent-cross-talk/` independently. When a pattern of pressure notices accumulates (e.g., three attempts in one session to add `recommend_*` functions), Watcher may elect to initiate a spec amendment interview. m15 never decides whether the amendment proceeds; it only witnesses and records.

---

## m14 — `evidence_aggregator`

### Responsibility

Aggregate `workflow_runs` rows from m7's central output table, compute the `habitat_outcome_lift` metric over a rolling window, and emit the metric to m11 (for sunset decision) and per-workflow lift-contribution scores to m31 (for selection weight updates).

m14 is the **empirical basis** for the m11 sunset law. Without it, the 120-day sunset default is based solely on time; with it, the sunset clock can be extended or accelerated based on observed value. m14 also provides the gradient that differentiates which workflows in the m30 bank deserve LTP reinforcement versus LTD pressure.

### What is `habitat_outcome_lift`?

`habitat_outcome_lift` is a dimensionless ratio measuring whether workflows executed through the engine produce better habitat outcomes than baseline behaviour (pre-engine tool-call sequences). It is not a single measurement — it is a rolling aggregate with explicit uncertainty bounds.

**Formal definition:**

```
// For a window W of N workflow runs (N >= 20 required):
//
// cascade_success_rate = |{r in W : r.cascade_success == true}| / N
// cost_per_decision    = mean(r.cost_tokens / max(r.decisions_made, 1)) for r in W
// baseline_cost        = mean(r.decision_cost_baseline) for r in W
//
// cost_lift = (baseline_cost - cost_per_decision) / baseline_cost
//   → positive means workflow is cheaper per decision than baseline
//   → negative means workflow is more expensive
//
// habitat_outcome_lift = 0.6 * cascade_success_rate + 0.4 * cost_lift.max(-1.0).min(1.0)
//
// Confidence interval (Wilson interval for cascade_success_rate component):
//   z = 1.96  // 95% CI
//   p = cascade_success_rate
//   n = N
//   ci_half = z * sqrt(p * (1 - p) / n + z^2 / (4 * n^2)) / (1 + z^2 / n)
//
// The composite CI is propagated from the cascade component (dominant term).
// Reported as:  lift ± ci_half  (both values must be positive for "evidence of lift").
```

The two candidate metrics from the Operator's town-hall ask 4 are both present:

- **cascade_success_rate** — "cross-pane cascade success rate": whether the workflow, when dispatched, results in a successful cascade outcome (as recorded in m7's `cascade_success` boolean). This is the field-coherence-aligned outcome signal.
- **cost_lift** — "lift in cost-per-decision over baseline": whether the workflow costs fewer tokens per meaningful decision than the pre-engine baseline. The baseline is estimated once per session by m6 (`context_cost_record`) and stored as `decision_cost_baseline` in m7 rows.

The 0.6/0.4 weighting is **configurable** via environment variable `WF_LIFT_CASCADE_WEIGHT` (default 0.6). It must sum to 1.0; the builder validates this at startup. The Operator may tune these weights as empirical data accumulates.

**Why the composite, not just cascade_success?** Cost-per-decision without outcome is a proxy that rewards cheap failures. Cascade success without cost is a proxy that rewards expensive successes. The composite penalises both failure and waste.

### F2 enforcement — n≥20 + CI bars

The F2 constraint from [[Genesis Prompt v1.2 S1001982]] is a **hard gate** in m14's output. When the rolling window has fewer than 20 runs, `habitat_outcome_lift` is `None` — not 0.0, not estimated, but `None`. Callers (m11, m31) must handle `None` by holding their current state rather than treating missing evidence as zero lift.

```rust
/// Rolling evidence state for the habitat-outcome-lift metric.
///
/// `lift` is `None` when fewer than `MIN_SAMPLE_SIZE` runs are present in the
/// window.  Callers MUST treat `None` as "insufficient evidence" and hold their
/// current state rather than treating absence as zero lift.
///
/// F2 invariant: `ci_half` is always provided alongside `lift` when `Some`;
/// downstream consumers that strip CI bars violate the spec gate.
#[derive(Debug, Clone)]
pub struct LiftSnapshot {
    /// Computed lift value.  `None` if `n < MIN_SAMPLE_SIZE`.
    pub lift: Option<f64>,
    /// Half-width of the 95% Wilson CI on the cascade-success component.
    /// Only present when `lift` is `Some`.
    pub ci_half: Option<f64>,
    /// Number of runs in the window that produced this snapshot.
    pub n: usize,
    /// Timestamp of the most recent run in the window.
    pub latest_ts_ms: i64,
}

/// Minimum runs required before any lift value is emitted (F2 hard gate).
pub const MIN_SAMPLE_SIZE: usize = 20;

/// Default rolling window size in runs (not time-based — run-count-based).
pub const DEFAULT_WINDOW_SIZE: usize = 120;
```

### Per-workflow lift-contribution scores (m14 → m31)

Beyond the aggregate metric, m14 computes a per-workflow contribution score: how much did each workflow's runs move the aggregate lift? This is the gradient signal m31 uses to reinforce selection weights.

```rust
/// Per-workflow contribution to the aggregate habitat-outcome-lift.
///
/// Positive `delta` means this workflow's runs were above the current
/// aggregate lift; negative means below.  m31 uses this to adjust
/// selection weights; m42 converts it to LTP/LTD signals for POVM.
#[derive(Debug, Clone)]
pub struct WorkflowLiftContribution {
    pub workflow_id: WorkflowId,
    /// Signed delta from aggregate lift.
    /// Range: approximately [-1.0, +1.0].
    pub delta: f64,
    /// Number of this workflow's runs in the current window.
    pub run_count: usize,
    /// Whether run_count >= MIN_SAMPLE_SIZE for this workflow individually.
    /// m31 should weight contributions from `individually_significant` workflows
    /// more heavily than contributions from low-n workflows.
    pub individually_significant: bool,
}
```

### Aggregation cycle — m16 Hebbian engine shape

m14's aggregation cycle is structurally analogous to the 4-step Hebbian consolidation cycle in `m16_hebbian_engine.rs` (boilerplate Cat 05):

1. **Decay** — slide the rolling window: evict runs older than `DEFAULT_WINDOW_SIZE` from the in-memory ring buffer. O(1) via `VecDeque` pop_front.
2. **Ingest** — pull new rows from m7's `workflow_runs` table since last `latest_ts_ms`. Batch read via paginated SQL.
3. **Compute** — recalculate `LiftSnapshot` and `WorkflowLiftContribution` vec from current window contents.
4. **Emit** — write snapshot to m11's input channel; write contributions vec to m31's input channel; write LTP/LTD list to m42.

The cycle fires on a timer (default: every 5 minutes). It does not fire on every run ingested — that would create write amplification into m11 and m31 on busy sessions.

### Snapshot store — `Arc<RwLock<Option<T>>>` pattern

m14's in-memory state uses the exact `Arc<RwLock<Option<HabitatState>>>` pattern from `habitat-nerve-center_m3_aggregator_mod.rs` (boilerplate Cat 06), adapted to `LiftSnapshot`:

```rust
/// Thread-safe aggregator for the habitat-outcome-lift metric.
///
/// Clone is O(1) — the inner Arc is reference-counted.  Both the wf-crystallise
/// periodic-timer task and any concurrent read from m11 share the same lock.
///
/// Lock discipline: guards dropped inside brace blocks — never held across
/// an await point.
#[derive(Debug, Clone)]
pub struct EvidenceAggregator {
    inner: Arc<EvidenceAggregatorInner>,
}

#[derive(Debug)]
struct EvidenceAggregatorInner {
    snapshot: parking_lot::RwLock<Option<LiftSnapshot>>,
    contributions: parking_lot::RwLock<Vec<WorkflowLiftContribution>>,
    window: parking_lot::RwLock<VecDeque<WorkflowRunRow>>,
    config: AggregatorConfig,
}
```

`clone-on-read` is the read path: callers get `Option<LiftSnapshot>` as an owned value. The lock is never returned to the caller.

### Error type

```rust
#[derive(Debug, thiserror::Error)]
pub enum AggregatorError {
    #[error("insufficient samples: n={n} < MIN_SAMPLE_SIZE={MIN_SAMPLE_SIZE}")]
    InsufficientSamples { n: usize },
    #[error("database read failed: {0}")]
    DbRead(#[from] rusqlite::Error),
    #[error("config validation failed: cascade_weight + cost_weight must equal 1.0")]
    InvalidWeights,
}
```

### Boilerplate reuse summary

| Boilerplate source | What is lifted | Reuse |
|---|---|---|
| `habitat-nerve-center_m3_aggregator_mod.rs` | `Arc<RwLock<Option<T>>>` snapshot store; clone-on-read; `AggregatorError` enum shape | 70% |
| `habitat-nerve-center_main.rs` | `VecDeque` ring-buffer O(1) eviction; `HISTORY_CAPACITY` constant pattern | 60% |
| `m16_hebbian_engine.rs` | 4-step aggregation cycle (decay → ingest → compute → emit) | 80% |

New authorship: the lift formula itself, F2 n≥20 guard, CI computation, per-workflow delta, m31 contribution gradient.

### Test targets (50+ minimum)

- `lift_none_when_n_below_20` — F2 gate holds
- `lift_some_when_n_exactly_20` — boundary condition
- `ci_half_always_present_with_lift` — CI invariant
- `cost_lift_negative_when_workflow_more_expensive` — formula correctness
- `cascade_weight_and_cost_weight_must_sum_to_one` — config validation
- `window_eviction_drops_oldest_first` — ring buffer correct
- `workflow_delta_positive_when_above_aggregate` — contribution sign
- `individually_significant_false_when_run_count_below_20` — per-workflow F2
- `concurrent_ingest_and_read_safe` — lock discipline
- `emit_none_to_m11_when_insufficient` — downstream contract

---

## m15 — `pressure_register`

### Responsibility

Detect, log, and emit `PHASE-B-RESERVATION-NOTICE` files whenever a forbidden-verb-pressure event occurs. m15 is a witness module, not a gating module. It does not block anything. It creates a durable record that scope-pressure happened, where it came from, and what verb or feature was being pushed.

### What is a forbidden-verb-pressure event?

A forbidden-verb-pressure event is any observable signal that someone (an agent, a spec patch, a cross-talk message, a CLI verb proposal, or a user request) is attempting to add a capability that is currently outside the engine's chartered verb-set.

The chartered verb-set for `wf-crystallise` is: **read, correlate, record, emit reports, aggregate evidence, log pressure.**

The chartered verb-set for `wf-dispatch` is: **bank, select, verify, dispatch.**

**Forbidden verbs** (the pressure register's trigger surface):
- `recommend_*` — any function that recommends something to the user without user request
- `auto_*` / `smart_*` — autonomous trigger without explicit user invocation
- `rewrite_*` — modifying source files outside the `wf-*` binary itself
- `route_*` — routing decisions that bypass Conductor (P0 #3)
- `package_*` / `publish_*` — packaging artifacts without explicit verification gate
- `optimise_*` — automated performance tuning without measurement gate
- `promote_*` (Phase A only, now unified into dispatch) — auto-promoting without human gate
- HTTP server / sidecar daemon — adding a persistent listener process (Hard Refusal from [[Genesis Prompt v1.2 S1001982]])
- POVM writes — writing to deprecated POVM namespace (P0 #14)
- `use synthex_v2::*` — importing SYNTHEX v2 wholesale (Hard Refusal)

**Example pressure events:**

1. A spec patch proposes adding `fn recommend_cascade_for_next_session()` to m23 — this is a `recommend_*` verb; m15 fires.
2. An agent report describes an auto-promote feature where workflows above 0.85 fitness are auto-dispatched without Luke's confirmation — this is `auto_*`; m15 fires.
3. A cross-talk message from another CC instance requests that m14 expose an HTTP endpoint for live metric polling — this is an HTTP server surface; m15 fires.
4. Someone proposes adding `workflow_engine_sidecar` as a Batch 5 service that starts alongside `wf-crystallise` — this is a daemon/sidecar; m15 fires.

The trigger detection is necessarily heuristic at this phase (no runtime detection machinery yet). m15's primary detection surface is:
- **Spec patches** applied to `THE_WORKFLOW_ENGINE_GENESIS_PROMPT_V*.md` or any `cluster-*.md` spec file — m15 scans for newly appearing forbidden-verb prefixes in diffs
- **Agent-cross-talk files** written by other CC instances to `agent-cross-talk/` — m15 monitors for content containing forbidden-verb patterns
- **CLI verb proposals** surfaced in `CLAUDE.local.md` or session notes — scanned at crystalliser sweep time

### Pressure event schema

```rust
/// A single forbidden-verb-pressure event observed by m15.
///
/// Schema version: 1.  Any schema change bumps the version field and the
/// notice file name includes the version: `phase-b-reservation-notice-v{N}.jsonl`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PressureEvent {
    /// Monotonic event ID within this engine instance's lifetime.
    pub id: u64,
    /// RFC 3339 timestamp of detection.
    pub detected_at: String,
    /// Which session produced this pressure (from `$CLAUDE_SESSION_ID` env or "unknown").
    pub session_id: String,
    /// The detected forbidden verb or surface category.
    pub forbidden_category: ForbiddenCategory,
    /// Raw text excerpt that triggered detection (max 512 chars, truncated).
    pub trigger_excerpt: String,
    /// Where the pressure originated.
    pub source: PressureSource,
    /// Human-readable summary of what was being proposed.
    pub proposed_feature: String,
    /// Which chartered verb-set would have been violated.
    pub violated_charter: CharterSection,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForbiddenCategory {
    RecommendVerb,
    AutoOrSmartVerb,
    RewriteVerb,
    RouteBypassConductor,
    PackageOrPublish,
    OptimiseWithoutGate,
    HttpServerSurface,
    SidecarDaemon,
    DeprecatedPovmWrite,
    SynthexV2Import,
    Other { description: String },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PressureSource {
    SpecPatch { file: String },
    AgentCrossTalk { from_session: String },
    CliVerbProposal { proposed_command: String },
    SessionNote { note_path: String },
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CharterSection {
    /// wf-crystallise verb-set (read/correlate/record/emit/aggregate/log)
    Crystalliser,
    /// wf-dispatch verb-set (bank/select/verify/dispatch)
    Dispatcher,
    /// Hard refusals that apply to the entire engine (HTTP server, POVM writes, synthex import)
    HardRefusal,
}
```

### The `PHASE-B-RESERVATION-NOTICE` file format

The term "PHASE-B-RESERVATION-NOTICE" is preserved from the original phased architecture where m15 reserved features for a future Phase B activation. In the single-phase architecture, the notice is semantically equivalent but the payload is "forbidden-verb-pressure noticed". The name is kept for continuity with the spec's feature-verification matrix (row 30) and to maintain recognisability for Watcher and Zen.

**File naming:** `agent-cross-talk/PHASE-B-RESERVATION-NOTICE-{iso8601-date}-{session_id_prefix}-{event_id}.jsonl`

**File location:** `~/projects/shared-context/agent-cross-talk/` (the shared-context filesystem layer, always available regardless of service health).

**File format:** JSONL — one `PressureEvent` JSON object per line. Each notice file contains exactly one event. This makes `grep` and `jq` trivial. Multiple events in a session produce multiple files.

**Example notice file content:**

```jsonl
{"id":1,"detected_at":"2026-05-17T14:33:02Z","session_id":"S1001999","forbidden_category":"recommend_verb","trigger_excerpt":"fn recommend_cascade_for_next_session(cluster_id: ClusterId) -> RecommendedWorkflow {","source":{"spec_patch":{"file":"cluster-F-iteration.md"}},"proposed_feature":"Add function to automatically recommend cascade workflows for the next session based on recent cluster outcomes","violated_charter":"crystalliser"}
```

### Structured tracing output

m15 emits `tracing::warn!` events for each detected pressure event, following the ME v2 gold standard logging pattern from `logging.rs`:

```
level=WARN module=m15_pressure_register event=forbidden_verb_pressure category=recommend_verb source=spec_patch file="cluster-F-iteration.md" session_id=S1001999 proposed_feature="Add function to automatically recommend..."
```

This surfaces in the `wf-crystallise` daemon log and is visible to any log monitor without requiring the shared filesystem.

### Observer contract (Watcher + Zen)

Both Watcher ☤ and Zen observe `agent-cross-talk/` notices. The observation pattern is read-only and happens at their own cadence — m15 does not push to them. The contract is:

- m15 WRITES to `agent-cross-talk/` (shared filesystem — always available)
- Watcher and Zen READ from `agent-cross-talk/` at their own cadence
- Neither Watcher nor Zen confirms receipt to m15
- Silence from Watcher and Zen does NOT indicate the notice was missed

This is the **preserve-blanket-guard pattern** analogue for scope-pressure: the S102 lesson (`feedback_preserve_list_discipline.md`) is that named exclusions upstream do not propagate through blanket operations. m15 applies the same principle to scope-pressure: the engine's charter named which verbs are permitted, but that exclusion must be re-asserted at every pressure site. m15 is that re-assertion mechanism — the per-site enumeration-before-execution check, but applied to spec-level scope creep rather than filesystem operations.

### Error type

```rust
#[derive(Debug, thiserror::Error)]
pub enum PressureRegisterError {
    #[error("failed to write notice file: {path}: {source}")]
    NoticeWrite { path: String, source: std::io::Error },
    #[error("agent-cross-talk directory not found or not writable: {path}")]
    DirectoryUnavailable { path: String },
    #[error("event serialization failed: {0}")]
    Serialize(#[from] serde_json::Error),
}
```

### Boilerplate reuse summary

| Boilerplate source | What is lifted | Reuse |
|---|---|---|
| `logging.rs` (ME v2) | `tracing::warn!` structured emission pattern; correlation ID propagation | 60% |
| `hookify.preserve-blanket-guard.local.md` | Conceptual model: per-site named-exclusion enforcement vs. blanket bypass; 3-step enumeration-diff-rewrite protocol | reference/conceptual |
| `feedback_preserve_list_discipline.md` | Structural root cause of the pressure-register's existence | reference |

New authorship: `PressureEvent` schema, `ForbiddenCategory` enum, JSONL notice format, detection heuristics, `agent-cross-talk/` write path.

### Test targets (50+ minimum)

- `notice_file_created_on_pressure_event` — filesystem write confirmed
- `notice_filename_includes_session_and_event_id` — naming convention
- `jsonl_parses_back_to_pressure_event` — round-trip schema
- `recommend_verb_detected_from_spec_patch_excerpt` — detection trigger
- `auto_verb_detected` — category classification
- `http_server_surface_detected` — hard refusal category
- `trigger_excerpt_truncated_at_512_chars` — safety bound
- `missing_directory_returns_error_not_panic` — error handling
- `multiple_events_produce_multiple_files` — one-event-per-file invariant
- `tracing_warn_emitted_with_structured_fields` — log contract

---

## m14 ↔ m31 feedback — how lift informs selection weighting

m31 (`selector`) enforces diversity-weighted selection across the m30 workflow bank. The selection weight for each workflow starts as a function of its RALPH fitness score and diversity distance from recently selected workflows. m14 introduces a third term: the **lift-contribution gradient**.

The update happens via a write-once-per-cycle channel from m14 to m31 (an in-process `tokio::sync::watch` channel carrying `Vec<WorkflowLiftContribution>`). m31 reads the latest contributions on each selection call; it does not pull m14 directly.

The weight update formula in m31 is:

```rust
// Within m31's selection weight computation (pseudocode):
//
// base_weight = ralph_fitness_score(workflow_id) * diversity_distance_bonus
//
// lift_contribution = m14_contributions
//     .iter()
//     .find(|c| c.workflow_id == workflow_id)
//     .map(|c| c.delta)
//     .unwrap_or(0.0);
//
// // Only apply lift signal if individually significant (n >= 20 for this workflow).
// lift_factor = if individually_significant {
//     1.0 + lift_contribution.clamp(-0.3, +0.3)
// } else {
//     1.0  // lift not significant yet; hold weight
// };
//
// final_weight = (base_weight * lift_factor).max(0.0);
```

The `clamp(-0.3, +0.3)` bounds the lift signal so a single workflow cannot dominate selection even with extreme positive lift. This is the RALPH fitness-weighted decay invariant (P0 #8 from [[Town Hall S1001982]]) applied to lift: fitness signal informs selection, but does not override diversity enforcement.

**The m42 (`hebbian_feedback`) connection:**

m14 emits two lists to m42 on each cycle:
- `ltp_workflows`: workflow IDs where `delta > 0.0` and `individually_significant == true`
- `ltd_workflows`: workflow IDs where `delta < 0.0` and `individually_significant == true`

m42 converts these to `POST /reinforce` calls to POVM under the `workflow_trace_*` namespace (AP30 collision avoidance). Workflows not on either list are not touched — m42 does not touch pathways without evidence of lift direction.

This closes the feedback loop: m7 records runs → m14 aggregates lift → m31 updates weights → m32 selects higher-lift workflows more often → those workflows run → m7 records more runs → m14 updates lift again. The loop self-attenuates because diversity enforcement in m31 prevents any single workflow from being selected exclusively.

---

## Standing constraints carried into both modules

| Constraint | Enforcement point |
|---|---|
| F2 n≥20 + CI bars (m14 output must carry CI or be None) | m14 `LiftSnapshot.ci_half` always `Some` when `lift` is `Some` |
| W1 narrowed stcortex consumer (m14 reads m7, not stcortex directly) | m14 reads from local SQLite m7 table; no direct stcortex reads |
| AP30 namespace prefix for POVM writes | m42 owns POVM write; m14 only emits workflow ID lists |
| Hard refusal: no HTTP server surface | Neither m14 nor m15 exposes a listening socket |
| Hard refusal: no POVM writes from m14/m15 directly | m42 is the exclusive POVM writer |
| `#![forbid(unsafe_code)]` + zero `.unwrap()` | All paths return `Result<T>` |
| 50+ tests per module | Stated explicitly in each module's test target list |
| Tracing for all observability (no `println!`) | Both modules use `tracing::info/warn/error` |

---

## Obsidian cross-references

- Architecture: [[Modules Synergy Clusters and Feature Verification S1001982]] §"Cluster E"
- Feature verification: [[Modules Synergy Clusters and Feature Verification S1001982]] rows 8 (F2) and 30 (pressure register)
- Genesis spec: [[Genesis Prompt v1.2 S1001982]] §"Failure-mode table F2" + §"Hard refusals"
- Town Hall P0 constraints: [[Town Hall S1001982]] §"15 P0 hard constraints"
- Boilerplate sources: [[Boilerplate Hunt S1001982]] Cat 05 (decay), Cat 06 (aggregator), Cat 09 (preserve-blanket-guard)
- Preserve-blanket-guard: `feedback_preserve_list_discipline.md` in MEMORY.md
- ME v2 logging gold standard: `~/claude-code-workspace/the_maintenance_engine_v2/src/m1_foundation/logging.rs`
- ME v2 aggregator gold standard: `~/claude-code-workspace/the_maintenance_engine_v2/src/m1_foundation/metrics.rs`
- Nerve Center aggregator clone: `habitat-nerve-center_m3_aggregator_mod.rs` in boilerplate Cat 06

---

*Cluster E spec authored 2026-05-17 (S1001982) · planning-only · HOLD-v2 active · gates G1-G9 required before build*
