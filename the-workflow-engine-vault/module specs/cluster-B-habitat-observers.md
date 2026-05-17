---
title: Cluster B — HABITAT OBSERVERS — Module Specs
date: 2026-05-17 (S1001982)
status: planning-only · HOLD-v2 active · single-phase deployment (Luke override)
cluster: B
modules: m4 cascade_correlator · m5 battern_step_record · m6 context_cost_record
loc_estimate: ~460 (m4 ~180 + m5 ~150 + m6 ~130)
---

# Cluster B — HABITAT OBSERVERS — Module Specs

> Back to: [[HOME]] · [[MASTER_INDEX]] · [[workflow-engine-code-base]] · [[Modules Synergy Clusters and Feature Verification S1001982]] · [[Genesis Prompt v1.2 S1001982]]
>
> Sibling specs (when written): `cluster-A-substrate-ingest.md` · `cluster-C-central-correlation.md`
>
> Canonical code path (pre-build): `~/claude-code-workspace/the-workflow-engine/`

This cluster is the **highest-novelty** cluster in the single-phase architecture. Where Cluster A lifts ~80-90% boilerplate from existing habitat sources, Cluster B is authorship-dominant. The three modules here instrument phenomena that the habitat has never recorded in structured form: multi-agent cascade correlations, Battern protocol step-level timing, and token-cost-to-outcome correlation with exploration-rate baseline preservation.

---

## Cluster Overview

| Module | Role | Verb budget (Phase A passive) | LOC |
|--------|------|-------------------------------|-----|
| m4 `cascade_correlator` | Correlate Fleet pane traces + cc-* logs into opaque cluster IDs | observe · correlate · record · emit | ~180 |
| m5 `battern_step_record` | Observe Battern protocol step durations and outcomes per battern | observe · record · emit | ~150 |
| m6 `context_cost_record` | Record per-session token costs and carry exploration-rate baseline | record · emit · preserve | ~130 |

**Phase A verb discipline**: these modules observe, correlate, record, and emit. They NEVER recommend, optimise, propose action, or label patterns with human-meaningful names. The single-phase override permits active verbs in m20-m23 (iterators), but Cluster B modules sit in the observation tier — their verb budget remains passive. F11 (cascade monoculture) and F10 (exploration-cost preservation) are the two highest-priority feature constraints in this cluster and are treated as P0.

---

## Cross-Cluster Contracts

### CC-1: Cascade-Cost Coupling (B internal → Cluster C hub)

m4 and m6 are joined through m7 (`workflow_arc_record`, Cluster C) via `session_id` ranges. The join schema is the stable contract Cluster C (m7, m12, m13) consumes — neither m4 nor m6 knows about the other directly. The contract shape:

```
m7.workflow_runs rows carry:
  session_id         TEXT NOT NULL   -- FK origin: atuin history.session
  cascade_cluster_id TEXT            -- populated by m4; NULL if no cascade detected
  token_cost_input   INTEGER         -- populated by m6; NULL if not recorded
  token_cost_output  INTEGER         -- populated by m6; NULL if not recorded
  cost_band          TEXT            -- populated by m6; NULL if no baseline yet
  exploration_baseline REAL          -- populated by m6 rolling mean; NULL until N>=5
```

m4 writes `cascade_cluster_id`; m6 writes the cost columns. The join for downstream analysis is always `SELECT ... FROM workflow_runs WHERE session_id BETWEEN :start AND :end`, not a direct m4-m6 join. This decoupling is intentional and load-bearing for F11.

### CC-1b: Battern-Cost Coupling (m5 ↔ m6 via battern_id range)

m5 records step outcomes keyed on `battern_id`. m6 records token costs keyed on `session_id`. A `battern_id` spans one or more `session_id` values (a battern often runs across a single CC session, but multi-session batterns are possible when a phase causes a context handoff). The join point is the `battern_observations` table carrying both `battern_id` and the `session_id` in which each step was observed. m6's cost rows are joinable via `session_id`, giving Cluster F iterators (m21 `battern_iterator`) a cost-per-step approximation.

---

## m4 — `cascade_correlator`

### Purpose

m4 observes atuin tool-call history across Fleet ALPHA, BETA, and GAMMA panes and correlates overlapping dispatch chains from `cc-*` binaries into opaque cluster identifiers. The cluster identifier encodes nothing about what the cascade did — it is a stable identity token for a temporally coherent multi-pane dispatch event, surfacing later as evidence for Cluster F (iteration) and Cluster E (evidence aggregation).

The keystone gap called out in the Boilerplate Hunt: **N-step compositional sub-graph detection with gap-allowed matching**. The `m49_task_graph.rs` provides the Kahn's-sort DAG backbone (50% reuse) and `m20_heat_source_hebbian.rs` provides the pairwise O(n²) co-activation pattern (30% reuse). The N-step generalisation — detecting that panes A, B, and C ran steps that form a compositional sub-graph within a sliding time window, even with temporal gaps allowed between steps — is fresh authorship and constitutes the structural new-authorship piece of the entire engine.

### Public surface

```rust
/// Opaque identifier for a correlated multi-pane cascade event.
/// Contains no human-meaningful label (F11 constraint).
/// Format: "cascade_cluster_N" where N is a collision-resistant u64
/// derived from (session_range_hash XOR pane_set_hash XOR step_count).
pub struct CascadeClusterId(pub String);

/// A single tool-call step as read from atuin history.
#[derive(Debug, Clone)]
pub struct AtuinStep {
    /// Atuin row ID.
    pub id: String,
    /// Unix timestamp (nanoseconds, matching atuin history.timestamp).
    pub ts_ns: i64,
    /// The command string.
    pub command: String,
    /// Working directory.
    pub cwd: String,
    /// Atuin session identifier — maps to one CC session / fleet pane.
    pub session: String,
    /// Exit code (0 = success).
    pub exit: i32,
}

/// A cc-* dispatch log entry read from atuin history or log file.
#[derive(Debug, Clone)]
pub struct DispatchRecord {
    pub ts_ns: i64,
    pub pane_label: String,   // ALPHA-LEFT, BETA-TR, etc. — opaque label only
    pub binary: String,       // cc-dispatch, cc-cascade, cc-broadcast, etc.
    pub session: String,
}

/// A correlated cascade cluster: a set of AtuinSteps + DispatchRecords
/// that co-occurred within the correlation window.
#[derive(Debug, Clone)]
pub struct CascadeCluster {
    pub cluster_id: CascadeClusterId,
    /// Earliest step ts_ns in the cluster.
    pub window_start_ns: i64,
    /// Latest step ts_ns in the cluster.
    pub window_end_ns: i64,
    /// Number of distinct pane sessions observed.
    pub pane_count: usize,
    /// Number of steps in the sub-graph.
    pub step_count: usize,
    /// Whether the sub-graph exhibited temporal gaps (gap-allowed matching).
    pub has_temporal_gaps: bool,
    /// DAG depth from Kahn's sort (longest path in the step sub-graph).
    pub dag_depth: usize,
    /// Observed at (wall clock, for TTL sweep).
    pub observed_at_ms: i64,
}

/// Configuration for the cascade correlator.
#[derive(Debug, Clone)]
pub struct CascadeCorrelatorConfig {
    /// Maximum temporal gap allowed between steps within one cluster (ms).
    /// Default: 30_000 (30 seconds). Steps further apart split into separate clusters.
    pub max_gap_ms: i64,
    /// Minimum number of panes required to constitute a cascade (not a solo run).
    /// Default: 2.
    pub min_pane_count: usize,
    /// Sliding window size for sub-graph detection (ms).
    /// Default: 300_000 (5 minutes).
    pub window_ms: i64,
    /// Maximum number of steps per cluster before splitting.
    /// Default: 500.
    pub max_steps_per_cluster: usize,
    /// Atuin history DB path.
    pub atuin_db_path: std::path::PathBuf,
}

/// The cascade correlator — reads atuin history and cc-* dispatch records,
/// emits CascadeCluster rows.
pub struct CascadeCorrelator {
    config: CascadeCorrelatorConfig,
}

impl CascadeCorrelator {
    /// Create a new correlator with the given config.
    pub fn new(config: CascadeCorrelatorConfig) -> Self;

    /// Read atuin history rows since `since_ts_ns`, paginated.
    ///
    /// # Errors
    /// Returns error if atuin DB is not accessible or schema has changed.
    pub fn read_atuin_since(
        &self,
        since_ts_ns: i64,
        limit: usize,
    ) -> Result<Vec<AtuinStep>, CascadeError>;

    /// Correlate a batch of steps into zero or more CascadeClusters.
    ///
    /// This is the N-step sub-graph detection core. Steps are grouped by:
    /// 1. Sliding window (window_ms)
    /// 2. Pane-session membership
    /// 3. DAG edge inference from temporal ordering + cwd overlap
    /// 4. Gap-allowed matching (steps may be non-contiguous)
    /// Clusters below min_pane_count are discarded (solo runs, not cascades).
    pub fn correlate(
        &self,
        steps: &[AtuinStep],
        dispatch_records: &[DispatchRecord],
    ) -> Vec<CascadeCluster>;

    /// Assign a stable CascadeClusterId to a correlated group.
    ///
    /// The ID is derived from:
    ///   XOR(murmurhash3(session_range_str), murmurhash3(sorted_pane_labels), step_count as u64)
    /// The result is formatted as "cascade_cluster_{u64_hex}".
    /// No semantic information is encoded (F11).
    pub fn assign_cluster_id(
        &self,
        window_start_ns: i64,
        window_end_ns: i64,
        pane_labels: &[&str],
        step_count: usize,
    ) -> CascadeClusterId;
}
```

### Internal data structures

**Step graph** (in-memory, not persisted): a `HashMap<String, Vec<usize>>` adjacency list where keys are `session` strings and values are indices into the step batch. Edge inference: if two steps share the same `cwd` or if step B's `ts_ns` falls within `max_gap_ms` of step A's `ts_ns` and step B is in a different session, a DAG edge A→B is inferred. Kahn's sort runs on this graph to detect cycles (unexpected in practice) and compute dag_depth.

**Cluster accumulator**: a `Vec<Vec<usize>>` (cluster index → step indices). Built by a sliding-window scan: as each step enters the window, it is candidate-merged into the most recent open cluster whose `window_end_ns + max_gap_ms >= step.ts_ns`. If no open cluster qualifies, a new one is opened. Clusters are closed when a gap exceeds `max_gap_ms` with no new arrivals.

**Cluster ID derivation**: `cascade_cluster_{:016x}` where the hex value is:

```
hash_A = fnv1a_64(format!("{window_start_ns}:{window_end_ns}"))
hash_B = fnv1a_64(sorted_pane_labels.join(","))
id_u64 = hash_A ^ hash_B ^ (step_count as u64)
```

FNV-1a is chosen for its speed (no external dependency) and acceptable collision resistance at the expected cardinality (<<10^6 clusters per habitat lifetime). If a collision is detected (same id, different content), step_count is incremented by 1 and the hash is recomputed — this is safe because the collision-resolved id is still opaque (F11 preserved).

### Data flow

```
atuin history.db (read-only, paginated)
    ↓  read_atuin_since()
Vec<AtuinStep>
    │
    ├── cc-* dispatch log entries (also sourced from atuin: command LIKE 'cc-%')
    │
    ↓  correlate()
    sliding window + DAG edge inference + Kahn's sort
    ↓
Vec<CascadeCluster>  (emitted)
    ↓
m7 workflow_arc_record  (Cluster C writes cascade_cluster_id to workflow_runs)
```

m4 does NOT write to any database directly. It emits `CascadeCluster` values to the calling sweep loop. m7 (Cluster C) owns the write path. This keeps m4's observation contract clean and testable without a live DB.

### Boilerplate lifts

This module is predominantly new authorship. The two partial lifts:

- `m49_task_graph.rs` (Boilerplate category 04) — Kahn's topological sort logic (~50 LOC adapted) and the `TaskNodeState` FSM shape inform the step-state lifecycle. The node/edge adjacency representation is structurally reused, with tool-call steps replacing pane tasks as nodes.
- `m20_heat_source_hebbian.rs` (Boilerplate category 04) — the O(n²) `CoActivationPair` iteration pattern informs the pairwise gap-detection inner loop, but the N-step generalisation that groups pairwise co-activations into a single sub-graph identity is new (~80 LOC of the 180 total).

### ME v2 patterns

- `SignalContext` pattern from `signals.rs`: `CascadeCluster` carries `observed_at_ms` analogous to `Timestamp` — a single monotonic read at observation time, not multiple System::now() calls.
- Docstring style from `resources.rs`: module-level doc comment with Layer / Dependencies / Features table.
- Result-everywhere: `read_atuin_since` returns `Result<Vec<AtuinStep>, CascadeError>` — no panic paths. `correlate` returns `Vec` (never fails, correlation of zero steps returns empty vec).

### Constraints satisfied

- **F11 (cascade monoculture)**: `CascadeClusterId` is intentionally opaque. The derivation mixes hash values in a way that destroys any semantic signal about the cascade's content. No human-readable name is embedded. Downstream modules receive only the id token; they cannot reconstruct which panes were involved without the raw step data.
- **Phase A passive verbs**: observe (atuin reads), correlate (sub-graph detection), record (cluster_id assigned), emit (CascadeCluster returned). Never recommend, optimise, or select.
- **No direct DB write**: m4 does not write to any SQLite or stcortex. Observation is pure.

### Tests (minimum 50)

Test categories:

1. **Unit — cluster ID stability**: same inputs produce same id across invocations; different inputs produce different ids (collision rate < 1% in a 10k-pair sweep).
2. **Unit — gap-allowed matching**: two steps 29,999ms apart cluster together; two steps 30,001ms apart split into separate clusters.
3. **Unit — min pane count filter**: single-session batches produce zero clusters.
4. **Unit — Kahn sort dag_depth**: a linear 3-step chain yields dag_depth=2; a fork-join yields correct depth.
5. **Unit — cycle detection**: artificially constructed cyclic step sequences are detected and logged, not silently accepted.
6. **Unit — F11 compliance**: cluster ids contain no alphanumeric labels matching pane names (ALPHA, BETA, GAMMA, LEFT, RIGHT, TR, BR).
7. **Integration — empty atuin DB**: `read_atuin_since` against an empty DB returns empty vec, no error.
8. **Integration — all cc-* binary detection**: a mock atuin batch containing cc-dispatch, cc-cascade, cc-broadcast, cc-consensus entries are all recognised as DispatchRecord candidates.
9. **Integration — overlap boundary**: a batch spanning two windows produces two separate clusters with no steps shared.
10. **Property — cluster_id is hex-formatted**: all returned ids match `cascade_cluster_[0-9a-f]{16}`.
11. **Boundary — max_steps_per_cluster**: a batch of 501 steps from 3 panes splits into two clusters at the boundary.
12. **Boundary — window_start_ns == window_end_ns**: single-step, multi-pane micro-burst is assigned a valid cluster.

### Open questions

1. The atuin history schema uses integer nanosecond timestamps. If atuin ever migrates to microseconds, the sliding window arithmetic breaks silently — add a schema-version check on first open.
2. cc-* dispatch records are sourced from atuin (commands matching `cc-*`). If a cc-* binary is renamed or a new fleet binary is added with a different prefix, the DispatchRecord filter needs updating. Consider a configurable prefix list rather than hardcoding `cc-`.
3. Gap-allowed matching currently uses wall-clock time gaps. If atuin timestamps drift (clock-skew across panes in a multi-host fleet), cluster boundaries will be wrong. For a single-host habitat this is not a concern; flag for multi-host expansion.

### LOC estimate: ~180

| Section | LOC |
|---|---|
| Types (CascadeClusterId, AtuinStep, DispatchRecord, CascadeCluster, config) | ~50 |
| `read_atuin_since` (rusqlite paginated read) | ~25 |
| `correlate` (sliding window + DAG construction + Kahn's sort) | ~70 |
| `assign_cluster_id` (FNV-1a + format) | ~15 |
| Error type | ~20 |
| **Total** | **~180** |

---

## m5 — `battern_step_record`

### Purpose

m5 observes executions of the Battern protocol — the 6-step fleet coordination pattern (Design / Dispatch / Gate / Collect / Synthesize / Compose) used across habitat fleet operations — recording per-step durations and outcomes keyed on a `battern_id`. The module does not impose the 6-step schema on observed data. It records coherent step sequences from the substrate and maps them to the named steps where the substrate's actual pattern matches; if a battern execution diverges (e.g., a gate step is skipped, a synthesize step is repeated), the raw observation is preserved without forcing it into the 6-step mould.

This is the only module in the engine that directly instruments the Battern protocol. Without it, the protocol's execution cost and success rate are invisible to the engine.

### Battern identification: what is a battern_id?

A `battern_id` is a stable opaque token identifying one run of the Battern protocol. It is not embedded in the atuin history rows — the Battern protocol does not emit a structured record. m5 infers battern boundaries from the substrate.

**Inference strategy (boundary detection):**

A battern boundary is inferred when two consecutive fleet-scope cc-* invocations are separated by a `design_signal` — a command matching `cc-dispatch` followed within 5 seconds by multiple pane-targeted commands. This represents the Design→Dispatch transition that opens a new battern. The previous battern is closed at that boundary.

A `battern_id` is derived as:

```
battern_id = "battern_{fnv1a_64(first_dispatch_ts_ns.to_string())}"
```

This makes the id deterministic given the first dispatch timestamp, collision-resistant at the expected habitat cardinality (<10^4 batterns per habitat lifetime), and free of semantic content.

**Fallback for ambiguous boundaries**: if a design→dispatch transition cannot be detected (the atuin window starts mid-battern), m5 records the partial battern with `is_partial = true`. Partial batterns are included in the observation record but flagged for downstream modules to treat with lower confidence.

### Protocol step taxonomy

The 6 canonical Battern steps are held in m5 as a soft schema — an ordered list of step patterns used to label observed steps, not to filter or gate them:

| Step index | Canonical name | Detection heuristic |
|------------|---------------|---------------------|
| 0 | Design | cc-dispatch invocations preceded by a planning-scope command (rg, atuin search, read) |
| 1 | Dispatch | cc-dispatch to ≥2 distinct panes within the Design window |
| 2 | Gate | cc-health, curl health-endpoints, or explicit gate-check commands |
| 3 | Collect | cc-harvest, cc-audit, or atuin search within the expected collection window |
| 4 | Synthesize | Longer-running commands (duration > 10s) not matching dispatch/collect patterns |
| 5 | Compose | cc-cascade, final aggregation commands, or explicit compose-marker commands |

If observed steps match none of these heuristics, they are recorded as `step_label = NULL` — the raw step is preserved. No step is discarded because it doesn't fit the 6-step shape.

### Public surface

```rust
/// Opaque identifier for one Battern protocol execution.
/// Format: "battern_{u64_hex}". No semantic content.
pub struct BatternId(pub String);

/// A single observed step within a battern execution.
#[derive(Debug, Clone)]
pub struct BatternStepObservation {
    pub battern_id: BatternId,
    /// Step index within this battern (0-based, may exceed 5 for divergent batterns).
    pub step_index: usize,
    /// Canonical step label if detected; None if step did not match any heuristic.
    pub step_label: Option<BatternStepLabel>,
    /// Duration of this step in milliseconds (end_ts_ns - start_ts_ns) / 1_000_000.
    pub duration_ms: i64,
    /// Atuin session in which this step was observed.
    pub session: String,
    /// Exit code of the primary command for this step.
    pub exit_code: i32,
    /// Whether this observation is from a partial (boundary-ambiguous) battern.
    pub is_partial: bool,
    /// Wall-clock milliseconds when this step was recorded by m5.
    pub recorded_at_ms: i64,
}

/// Named step labels (soft schema — used for labelling, not filtering).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatternStepLabel {
    Design,
    Dispatch,
    Gate,
    Collect,
    Synthesize,
    Compose,
}

/// Summary record for one complete battern execution.
#[derive(Debug, Clone)]
pub struct BatternRecord {
    pub battern_id: BatternId,
    /// Total steps observed (may differ from 6 for divergent executions).
    pub total_steps: usize,
    /// Steps that matched a canonical label.
    pub labelled_steps: usize,
    /// Steps with exit_code != 0.
    pub failed_steps: usize,
    /// Total wall-clock duration (last step end − first step start), ms.
    pub total_duration_ms: i64,
    /// Whether this battern completed (Compose step reached or explicit close).
    pub is_complete: bool,
    /// Whether the boundary was ambiguous at observation time.
    pub is_partial: bool,
}

/// Configuration for the battern step recorder.
#[derive(Debug, Clone)]
pub struct BatternStepRecordConfig {
    /// Maximum duration between consecutive steps before a battern is auto-closed (ms).
    /// Default: 1_800_000 (30 minutes).
    pub inter_step_timeout_ms: i64,
    /// Minimum number of steps to record a battern (filter noise).
    /// Default: 2.
    pub min_steps: usize,
    /// Atuin DB path (same as m4; shared read, no write).
    pub atuin_db_path: std::path::PathBuf,
}

/// The battern step recorder.
pub struct BatternStepRecord {
    config: BatternStepRecordConfig,
}

impl BatternStepRecord {
    pub fn new(config: BatternStepRecordConfig) -> Self;

    /// Infer battern boundaries from a batch of AtuinSteps and emit step observations.
    ///
    /// Steps are grouped into batterns by Design→Dispatch transition detection.
    /// Each group is assigned a BatternId, steps are labelled (soft), and
    /// BatternStepObservation rows are returned.
    pub fn observe(
        &self,
        steps: &[crate::m4::AtuinStep],
    ) -> Vec<BatternStepObservation>;

    /// Build a summary BatternRecord from a slice of step observations sharing one battern_id.
    pub fn summarise(observations: &[BatternStepObservation]) -> BatternRecord;
}
```

### Internal data structures

**Open battern accumulator**: `HashMap<BatternId, Vec<BatternStepObservation>>` — accumulates observations for batterns that have not yet been closed. A battern is closed when:
- A new Design→Dispatch transition is detected (closes the previous)
- `inter_step_timeout_ms` elapses since the last step (closed as timeout-complete)
- The input batch ends (remaining open batterns are emitted as partial)

**Step label heuristics**: a `Vec<(Regex, BatternStepLabel)>` table evaluated in order. The first matching heuristic wins. Compiled once at `BatternStepRecord::new()` using the `regex` crate (already used in habitat hooks).

### Data flow

```
Vec<AtuinStep>  (from m4's atuin read, or directly from m1 atuin_ingest)
    ↓  observe()
    Design→Dispatch boundary detection
    Step labelling (soft schema)
    ↓
Vec<BatternStepObservation>  (emitted)
    │
    ├── BatternRecord summaries via summarise()
    │
    ↓
m7 battern_observations table (Cluster C writes; m5 does not write directly)
```

### Boilerplate lifts

This module is largely new authorship. The one structural lift:
- `SKILL-quality-gate.md` (Boilerplate category 09) — the 4-stage gate pattern (Design/Dispatch/Gate/Collect form the first four steps) is the real-world protocol this module instruments. Reading the skill provided the heuristic patterns for step detection. No code is lifted — the skill informed the detection schema.

The `m49_task_graph.rs` TaskNodeState FSM shape informs BatternStepLabel (a soft enum of step states), but there is no direct code reuse.

### ME v2 patterns

- Builder pattern: `BatternStepRecord::new(config)` (no bare struct construction).
- Result-everywhere: `observe` returns `Vec` (infallible observation of a batch); methods that touch I/O return `Result`.
- `SignalContext`-style timestamping: `recorded_at_ms` is captured once at entry, not per-step, to avoid clock drift within a batch.

### Constraints satisfied

- **No imposed 6-step mould**: `step_label` is `Option<BatternStepLabel>`. Unlabelled steps are fully preserved. The module observes what is there, not what the protocol prescribes.
- **Opaque battern_id**: same F11 discipline as m4. The `battern_` prefix is a namespace marker, not a semantic label.
- **Phase A passive verbs**: observe, record, emit. Never optimise or propose action.
- **No direct DB write**: m5 emits observations; m7 owns the write path.

### Tests (minimum 50)

1. **Unit — Design→Dispatch detection**: a batch with cc-dispatch preceded by rg opens a new battern; a batch without it does not.
2. **Unit — boundary closes previous battern**: two consecutive Design→Dispatch transitions produce two distinct BatternIds.
3. **Unit — inter-step timeout**: steps 30 minutes apart close the open battern and start a new one.
4. **Unit — partial battern flag**: a batch starting mid-execution (no Design step) produces is_partial = true.
5. **Unit — step label assignment**: a step containing `cc-dispatch` is labelled Dispatch; a step containing `cc-harvest` is labelled Collect; an unrecognised step gets step_label = None.
6. **Unit — unlabelled steps preserved**: a batch of 3 steps with 1 labelled and 2 unlabelled produces 3 observations (not 1).
7. **Unit — min_steps filter**: a battern with 1 step is discarded.
8. **Unit — summarise total_duration_ms**: last step end − first step start.
9. **Unit — summarise failed_steps**: count of exit_code != 0.
10. **Unit — summarise is_complete**: Compose step present → true; otherwise false.
11. **Unit — summarise labelled_steps**: correct count of non-None step_labels.
12. **Unit — BatternId stability**: same first_dispatch_ts_ns produces same BatternId.
13. **Unit — BatternId uniqueness**: two different ts_ns values produce different ids (10k-pair sweep, collision rate < 0.1%).
14. **Integration — empty step batch**: observe([]) returns empty vec.
15. **Integration — single-pane batch with cc-dispatch**: produces a battern if step count >= min_steps.
16. **Integration — divergent battern (7 steps)**: 7 observations emitted, total_steps = 7, is_complete flag correct.
17. **Property — step_index is contiguous**: within one BatternId, step_indices are 0,1,2,... with no gaps.
18. **Boundary — exactly min_steps**: battern is included; min_steps - 1 is excluded.
19. **Boundary — Compose as last step**: is_complete = true.

### Open questions

1. The Design→Dispatch heuristic requires cc-dispatch to follow a planning command within 5 seconds. This window is tunable but may produce false positives if a user manually runs cc-dispatch without a planning prefix. Consider adding a configurable `require_planning_prefix` flag to the config.
2. Multi-session batterns (context handoff mid-battern) are a known scenario in the habitat. Current m5 design closes a battern when the atuin window ends — it does not attempt cross-session battern reconstruction. This should be flagged as a known limitation in the module's doc comment.
3. The 6-step canonical taxonomy is sourced from the Battern protocol as documented. If the protocol evolves (e.g., a seventh step is added), the `BatternStepLabel` enum and heuristics table need updating. Consider a config-driven heuristics table rather than a hardcoded enum for forward-compatibility.

### LOC estimate: ~150

| Section | LOC |
|---|---|
| Types (BatternId, BatternStepObservation, BatternStepLabel, BatternRecord, config) | ~55 |
| Boundary detection + step labelling in `observe()` | ~60 |
| `summarise()` | ~15 |
| `BatternStepRecord::new()` + heuristic table init | ~20 |
| **Total** | **~150** |

---

## m6 — `context_cost_record`

### Purpose

m6 records per-session token costs (input tokens and output tokens as proxied from stcortex `consumption_event` rows) and correlates them with workflow arc outcomes stored in m7. Its most critical function — and the one carrying the F10 mitigation — is carrying a **rolling exploration-rate baseline** that prevents downstream iteration (Cluster F) from collapsing all token budgets toward the minimum-cost workflow.

Without this baseline, an optimization pressure toward lower-cost workflows would systematically eliminate the exploratory, high-token-cost sessions where novel approaches are discovered. F10 names this failure mode "exploration-cost preservation collapse". m6's baseline record is the structural defence against it.

### What is the exploration-rate baseline, mathematically?

The baseline is a **habitat-wide EMA (exponential moving average) of token costs for sessions where the workflow outcome was not `Converged` or `Repeated`** — specifically, sessions where the arc outcome was `Explored`, `Diverged`, or `Unknown`. These are the sessions that carry novel information, and their cost distribution defines what "normal exploration" costs.

```
baseline_ema(t) = alpha * cost(t) + (1 - alpha) * baseline_ema(t-1)
```

where:
- `alpha = 2 / (N + 1)`, N = 20 (default, making this a 20-session EMA)
- `cost(t)` = `token_cost_input + token_cost_output` for session t, if session outcome is Explored/Diverged/Unknown
- Sessions with outcome Converged or Repeated are **excluded from the EMA** — they represent exploitation, not exploration

The baseline is computed per habitat (habitat-wide, not per-session), stored as a running state in the `workflow_trace.db` managed by m7, and updated each time m6 records a new exploration-classified session.

**Per-session vs habitat-wide**: m6 records per-session costs. The exploration-rate baseline is habitat-wide (a single EMA value, updated incrementally). The two are stored separately: per-session rows carry the raw costs; the `exploration_baseline` column in `workflow_runs` carries the baseline value at the time that session was recorded. This is the "carry" referred to in the F10 mitigation: the baseline travels alongside the per-session cost so Cluster F can always ask "was this session's cost above or below the exploration baseline at the time it ran?"

**Bootstrap period**: the EMA is undefined until at least 5 exploration-classified sessions have been recorded. During the bootstrap period, `exploration_baseline` is NULL in `workflow_runs` rows and `cost_band` is NULL. Downstream modules must handle NULL baselines gracefully.

### Cost-band classification

Once the baseline is established (N >= 5), each session's total cost is classified into a `cost_band`:

| Band | Condition |
|------|-----------|
| `below_baseline` | `total_cost < baseline_ema * 0.8` |
| `near_baseline` | `0.8 <= total_cost / baseline_ema < 1.2` |
| `above_baseline` | `total_cost >= baseline_ema * 1.2` |
| NULL | Bootstrap period (N < 5) |

The thresholds (0.8, 1.2) are configurable. The band classification is opaque — it carries no judgment about whether above-baseline is good or bad (F10 preserved: the engine records, it does not recommend).

### stcortex data source

m6 reads from stcortex `consumption_event` rows, which are written by the habitat's ORAC hook on every `access_memory` call. The `consumption_event` table (from stcortex API.md) carries:
- `memory_id` — the accessed memory
- consumer name + `read_count_total` increments

Token cost is **not directly recorded** in `consumption_event`. m6 uses a proxy:

```
token_cost_input_proxy  = count of tool_call rows for this session WHERE tool_name IN ('read', 'bash', 'grep', 'glob')
token_cost_output_proxy = count of tool_call rows for this session WHERE tool_name IN ('write', 'edit')
```

These are proxies, not exact token counts. The exact token count per session is not currently exposed by the habitat substrate. m6 records these proxies as `token_cost_input` and `token_cost_output`. The doc comment on each column explicitly states `proxy: tool-call count, not token count`. Future sessions may upgrade the proxy to real token counts when stcortex's `claude_session` table is extended.

### Public surface

```rust
/// Per-session cost record.
#[derive(Debug, Clone)]
pub struct SessionCostRecord {
    /// stcortex session_id (from claude_session.session_id).
    pub session_id: String,
    /// Proxy token cost: count of read/bash/grep/glob tool calls.
    pub token_cost_input_proxy: i64,
    /// Proxy token cost: count of write/edit tool calls.
    pub token_cost_output_proxy: i64,
    /// Total proxy cost (input + output).
    pub total_cost_proxy: i64,
    /// Workflow outcome for this session (from m7 join; None if not yet in m7).
    pub outcome: Option<WorkflowOutcome>,
    /// Exploration-rate baseline EMA at the time this record was created.
    /// None during bootstrap period (N < 5 exploration sessions recorded).
    pub exploration_baseline: Option<f64>,
    /// Cost band classification; None during bootstrap.
    pub cost_band: Option<CostBand>,
    /// Wall-clock ms when this record was created.
    pub recorded_at_ms: i64,
}

/// Workflow outcome classification for baseline EMA filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowOutcome {
    /// Session resulted in a repeated (already-known) workflow arc.
    Converged,
    /// Session repeated a previous workflow arc.
    Repeated,
    /// Session explored a novel workflow arc.
    Explored,
    /// Session arc diverged from expected patterns.
    Diverged,
    /// Outcome not yet classifiable.
    Unknown,
}

impl WorkflowOutcome {
    /// Whether this outcome qualifies for the exploration-rate baseline EMA.
    /// Converged and Repeated outcomes are excluded — they represent exploitation.
    pub fn is_exploration(&self) -> bool {
        matches!(self, Self::Explored | Self::Diverged | Self::Unknown)
    }
}

/// Cost band relative to the exploration-rate baseline EMA.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CostBand {
    BelowBaseline,
    NearBaseline,
    AboveBaseline,
}

/// Running state for the exploration-rate baseline EMA.
#[derive(Debug, Clone)]
pub struct ExplorationBaseline {
    /// Current EMA value (None during bootstrap).
    pub ema: Option<f64>,
    /// Number of exploration-classified sessions contributing to the EMA.
    pub n: usize,
    /// EMA smoothing factor alpha = 2 / (window + 1).
    pub alpha: f64,
}

impl ExplorationBaseline {
    /// Create a new baseline with the given EMA window size.
    pub fn new(window: usize) -> Self;

    /// Update the EMA with a new exploration-session cost.
    /// No-op if cost is not from an exploration-classified session.
    pub fn update(&mut self, cost: i64, outcome: WorkflowOutcome);

    /// Classify a cost relative to the current baseline.
    /// Returns None during bootstrap (n < 5).
    pub fn classify(&self, cost: i64, below_threshold: f64, above_threshold: f64) -> Option<CostBand>;
}

/// Configuration for the context cost recorder.
#[derive(Debug, Clone)]
pub struct ContextCostRecordConfig {
    /// EMA window size for exploration-rate baseline (sessions).
    /// Default: 20.
    pub baseline_ema_window: usize,
    /// Minimum exploration-classified sessions before baseline is considered valid.
    /// Default: 5.
    pub baseline_bootstrap_n: usize,
    /// Below-baseline threshold (cost < ema * below_threshold → BelowBaseline).
    /// Default: 0.8.
    pub below_threshold: f64,
    /// Above-baseline threshold (cost >= ema * above_threshold → AboveBaseline).
    /// Default: 1.2.
    pub above_threshold: f64,
    /// stcortex DB path or JSON snapshot path (fallback if :3000 unreachable).
    pub stcortex_snapshot_path: Option<std::path::PathBuf>,
}

/// The context cost recorder.
pub struct ContextCostRecord {
    config: ContextCostRecordConfig,
    baseline: std::sync::Mutex<ExplorationBaseline>,
}

impl ContextCostRecord {
    pub fn new(config: ContextCostRecordConfig) -> Self;

    /// Read tool_call rows from stcortex for a given session_id and compute proxy costs.
    ///
    /// Reads stcortex JSON snapshot if live DB unreachable (per CLAUDE.md: if :3000 unreachable,
    /// read snapshot and skip writes).
    ///
    /// # Errors
    /// Returns error if neither live DB nor snapshot is accessible.
    pub fn read_session_costs(
        &self,
        session_id: &str,
    ) -> Result<SessionCostRecord, ContextCostError>;

    /// Record a session cost and update the exploration-rate baseline if the session
    /// outcome qualifies as exploration.
    ///
    /// Returns the updated SessionCostRecord with baseline and cost_band populated.
    pub fn record_and_update_baseline(
        &self,
        mut record: SessionCostRecord,
    ) -> SessionCostRecord;

    /// Return a snapshot of the current exploration-rate baseline state.
    pub fn baseline_snapshot(&self) -> ExplorationBaseline;
}
```

### Internal data structures

**`ExplorationBaseline`** holds the running EMA state: `ema: Option<f64>`, `n: usize`, `alpha: f64`. Protected by `Mutex` in `ContextCostRecord` to allow concurrent reads from multiple sweep iterations without full struct cloning.

**stcortex access**: m6 reads from the stcortex JSON snapshot at `~/claude-code-workspace/stcortex/data/snapshots/latest.json` when the live `:3000` endpoint is unreachable. The snapshot structure contains `tool_call` rows under the `workflow_trace` namespace. If the namespace is absent from the snapshot, the session's costs are recorded as NULL (no error — absence of data is valid during bootstrap).

### Data flow

```
stcortex :3000 (live) or stcortex/data/snapshots/latest.json (fallback)
    ↓  read_session_costs()
    tool_call rows filtered by session_id
    ↓  proxy cost computation
SessionCostRecord (raw)
    ↓  record_and_update_baseline()
    ExplorationBaseline.update() if outcome.is_exploration()
    ExplorationBaseline.classify() → cost_band
SessionCostRecord (with baseline + cost_band populated)
    ↓
m7 workflow_arc_record  (Cluster C writes to workflow_runs; m6 does not write)
```

### Boilerplate lifts

Partially lifted structures:

- `m39_fitness_tensor.rs` (Boilerplate category 05, 70% reuse for infrastructure) — the rolling-mean smoothing pattern (6-sample window in the original, extended to N-sample EMA here) and the `volatile-dimension mask` concept inform the baseline bootstrap gate (`n >= baseline_bootstrap_n` before the EMA is considered valid).
- `ws_inbound_writer.rs` hourly TTL sweep pattern (Boilerplate category 05) — the session-level retention concept (keep N sessions of cost data, discard older) is adapted from the 7d retention mechanism.

### ME v2 patterns

- `Mutex`-protected interior mutability for `ExplorationBaseline` within `ContextCostRecord` (analogous to `RwLock` in ME v2 modules).
- `#[must_use]` on `baseline_snapshot()` — snapshot is a point-in-time clone, not live.
- `WorkflowOutcome::is_exploration()` mirrors the ME v2 pattern of predicate methods on domain enums (e.g., `TaskNodeState::is_terminal()`).

### Constraints satisfied

- **F10 (exploration-cost preservation)**: the EMA baseline exclusively tracks exploration-classified sessions (Explored/Diverged/Unknown). Converged and Repeated sessions are excluded, preventing the baseline from being pulled toward low-cost exploitation patterns. The `cost_band` column allows downstream modules to flag when exploration cost is being squeezed — without the module itself recommending a remedy.
- **F11 (opaque IDs in this module)**: m6 does not assign cluster IDs, but it does carry `session_id` as its join key. Session IDs from stcortex are opaque UUIDs, satisfying the no-human-meaningful-label constraint.
- **stcortex fallback discipline** (CLAUDE.md / CLAUDE.local.md): if `:3000` unreachable, reads snapshot and skips writes. Does NOT silently fall back to POVM.
- **Phase A passive verbs**: record, emit, preserve (baseline). Never recommend, never optimise.
- **No direct DB write**: m6 computes and returns `SessionCostRecord`; m7 owns the write.

### Tests (minimum 50)

1. **Unit — EMA bootstrap gate**: with n < 5, `classify()` returns None; with n >= 5, returns Some.
2. **Unit — EMA update**: after 5 exploration sessions with cost=100, ema is approximately 100.
3. **Unit — EMA excludes Converged**: Converged sessions do not update n or ema.
4. **Unit — EMA excludes Repeated**: Repeated sessions do not update n or ema.
5. **Unit — EMA includes Explored**: Explored sessions update n and ema.
6. **Unit — EMA includes Diverged**: Diverged sessions update n and ema.
7. **Unit — EMA includes Unknown**: Unknown sessions update n and ema.
8. **Unit — cost band BelowBaseline**: cost = ema * 0.79 → BelowBaseline.
9. **Unit — cost band NearBaseline**: cost = ema * 1.0 → NearBaseline.
10. **Unit — cost band AboveBaseline**: cost = ema * 1.21 → AboveBaseline.
11. **Unit — cost band boundary (0.8 threshold)**: cost = ema * 0.8 → NearBaseline (inclusive).
12. **Unit — cost band boundary (1.2 threshold)**: cost = ema * 1.2 → AboveBaseline (inclusive).
13. **Unit — is_exploration true for Explored**: WorkflowOutcome::Explored.is_exploration() == true.
14. **Unit — is_exploration false for Converged**: WorkflowOutcome::Converged.is_exploration() == false.
15. **Unit — proxy cost computation**: session with 3 read calls + 1 write call → input=3, output=1, total=4.
16. **Unit — NULL baseline in output record**: bootstrap period produces exploration_baseline = None.
17. **Unit — NULL cost_band in output record**: bootstrap period produces cost_band = None.
18. **Unit — stcortex snapshot fallback**: absent live DB, reads snapshot path and returns record.
19. **Unit — stcortex both absent**: returns ContextCostError (not panic).
20. **Unit — session with zero tool calls**: total_cost_proxy = 0, valid record returned.
21. **Unit — EMA convergence**: 100 exploration sessions at cost=200 → ema converges near 200.
22. **Unit — EMA drift detection**: after stable baseline, a burst of cost=1000 sessions shifts ema upward over time.
23. **Integration — full record_and_update_baseline cycle**: session read → EMA updated → cost_band classified → record returned with all fields populated.
24. **Integration — concurrent baseline reads**: two simultaneous `baseline_snapshot()` calls return consistent values.
25. **Property — cost_band None iff exploration_baseline None**: both are always None or both are Some.
26. **Boundary — exactly baseline_bootstrap_n = 5**: 4 sessions → None; 5th session → Some.
27. **Boundary — EMA window = 1**: degenerate window, ema equals last cost exactly.

### Open questions

1. Token cost proxy (tool-call count) is a rough approximation. The stcortex schema does not currently expose actual token counts from the Claude API. When stcortex is extended to carry real token counts (via the `claude_session` table), m6's `read_session_costs` should be updated to read from `usage_input_tokens` and `usage_output_tokens` directly. Add a feature flag `precise_token_costs` gated on the schema version.
2. The EMA window of 20 sessions is a design default. The appropriate window depends on how often exploratory sessions occur in practice. If the habitat runs 100 sessions per day but only 5 are exploratory, the 20-session window spans multiple weeks of real time. Consider a time-anchored baseline (e.g., EMA over the last 30 calendar days of exploration-classified sessions) as a future option.
3. The `WorkflowOutcome` enum requires that m7 has already classified each session's arc outcome before m6 can update the EMA. In the first sweep (before any arc outcomes are classified), all sessions will be `Unknown` — which counts toward the exploration baseline. This is intentional (Unknown is exploration-like) but should be flagged in the module's doc comment so future authors don't treat Unknown as a neutral/exclude case.

### LOC estimate: ~130

| Section | LOC |
|---|---|
| Types (SessionCostRecord, WorkflowOutcome, CostBand, ExplorationBaseline) | ~55 |
| `ExplorationBaseline::new/update/classify` | ~25 |
| `ContextCostRecord::new/read_session_costs/record_and_update_baseline/baseline_snapshot` | ~35 |
| Error type + config | ~15 |
| **Total** | **~130** |

---

## Cluster B Keystone Gap

The Boilerplate Hunt identified **N-step compositional sub-graph detection with gap-allowed matching + DAG isomorphism** as the engine's structural new-authorship piece, located squarely in m4. The gap is not just a missing algorithm — it is the observation that pairwise co-activation (what `m20_heat_source_hebbian.rs` provides) cannot detect that three panes jointly executing a cascade constitute a *single* compositional event. The step from pairwise to N-step requires:

1. A DAG representation of tool-call steps (adapted from `m49_task_graph.rs`)
2. A gap-allowed matching predicate (new authorship — ~80 LOC)
3. A cluster-identity derivation that is stable across restarts (FNV-1a XOR, new authorship)
4. F11 compliance baked into the identity scheme from genesis (no human labels ever enter the id)

Without m4, the engine has no way to identify that a multi-pane fleet operation produced a particular outcome. m5 and m6 would record step durations and costs in isolation, but those observations would be unanchored to a coherent event identity. The cascade cluster id from m4 is the anchor that makes Cluster C (m7) and Cluster F (m20 cascade iterator) coherent.

---

## Synergy Summary

| Coupling | Direction | Mechanism |
|---|---|---|
| m4 → m7 | Observation → Central hub | `cascade_cluster_id` written to `workflow_runs` |
| m5 → m7 | Observation → Central hub | `battern_observations` table rows |
| m6 → m7 | Observation → Central hub | Cost + baseline columns in `workflow_runs` |
| m6 ← m7 | Feedback for EMA classification | `WorkflowOutcome` read from m7 arc outcomes |
| m4 + m6 (CC-1) | Indirect join via m7 | `session_id` range is the join key, not a direct m4-m6 link |
| m5 + m6 (CC-1b) | Indirect join via `battern_id` | m6 cost rows joinable to m5 via session_id within battern window |
| B → F (CC-3) | Evidence enables iteration | m20 cascade iterator reads m4 cluster ids; m21 battern iterator reads m5 step records |

---

*Cluster B spec v1 · authored 2026-05-17 S1001982 · planning-only · HOLD-v2 active*
*Back to: [[HOME]] · [[MASTER_INDEX]] · [[Modules Synergy Clusters and Feature Verification S1001982]]*
