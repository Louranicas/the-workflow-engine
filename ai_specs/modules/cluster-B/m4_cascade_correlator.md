---
title: m4 — cascade_correlator — Per-Module Spec
date: 2026-05-17 (S1001982)
status: planning-only · HOLD-v2 active · markdown-only · NO .rs files
cluster: B — Habitat Observation
layer: L2
module_id: m4
binary: wf-crystallise
verb_class: passive (observe · correlate · record · emit)
loc_estimate: ~180
test_budget: 60
mutation_kill_threshold: 0.70
feature_gate: none
cc_owned: CC-1 (contributes via m7 join) · CC-3 (evidence-enabling)
cc_consumed: —
gap_owner: F11 (cascade monoculture) — exclusive
boilerplate_lift_pct: 50
status_row: SPEC
---

# m4 — `cascade_correlator` — Per-Module Spec

> **Back to:** [CLAUDE.md](../../../CLAUDE.md) · [CLAUDE.local.md](../../../CLAUDE.local.md) · [MODULE_MATRIX.md](../../MODULE_MATRIX.md) · vault [[cluster-B-habitat-observers]] · canonical V7 [cluster-B plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-B.md) · binding spec [GENESIS_PROMPT_V1_3.md](../../../ai_docs/GENESIS_PROMPT_V1_3.md) § 1
>
> **Sister modules (Cluster B):** [m4](m4_cascade_correlator.md) · [m5](m5_battern_step_record.md) · [m6](m6_context_cost.md)
>
> **Cross-cluster anchors:** Cluster A upstream — [m1_atuin_consumer](../cluster-A/m1_atuin_consumer.md) · Cluster C hub — [m7_workflow_runs](../cluster-C/m7_workflow_runs.md) · Cluster D aspect-wrap — [m8_povm_build_prereq](../cluster-D/m8_povm_build_prereq.md) · [m9_watcher_namespace_guard](../cluster-D/m9_watcher_namespace_guard.md) · Cluster F downstream — [m20_prefixspan_miner](../cluster-F/m20_prefixspan_miner.md)
>
> **Standards:** [PATTERNS.md](../../../PATTERNS.md) · [GOLD_STANDARDS.md](../../../GOLD_STANDARDS.md) · [ANTIPATTERNS.md](../../../ANTIPATTERNS.md)

---

## 1. Purpose

`m4_cascade_correlator` observes the `AtuinStep` stream emitted by `m1_atuin_consumer` and correlates overlapping multi-pane Fleet dispatch chains into **opaque cascade-cluster identifiers**. A cluster identifier encodes nothing about *what* the cascade did — it is a stable, collision-resistant token identifying one temporally coherent multi-pane dispatch event. Downstream, the identifier surfaces in `m7_workflow_runs` as `cascade_cluster_id`, where Cluster F (`m20_prefixspan_miner`) and Cluster E (`m14_habitat_outcome_lift`) reason over cluster cohorts.

m4 owns the engine's structural-new-authorship piece called out in the Boilerplate Hunt: **N-step compositional sub-graph detection with gap-allowed matching**. `m49_task_graph.rs` from the boilerplate-clone corpus provides the Kahn topological-sort backbone (≈50% lift); `m20_heat_source_hebbian.rs` provides the pairwise O(n²) co-activation pattern (≈30% lift). The N-step generalisation — detecting that panes A, B, and C jointly executed a compositional sub-graph within a sliding window even with temporal gaps between contributing steps — is fresh authorship (~80 of ~180 LOC). m4 is the canonical Cluster B owner of **F11 (cascade monoculture)** per [`ai_docs/optimisation-v7/ANTIPATTERNS_REGISTER.md`](../../../ai_docs/optimisation-v7/ANTIPATTERNS_REGISTER.md) § AP-WT-F11.

Verb budget (Phase A passive, retained under single-phase override per [GENESIS_PROMPT_V1_3.md](../../../ai_docs/GENESIS_PROMPT_V1_3.md) § 3): **observe · correlate · record · emit**. m4 never recommends, optimises, selects, or labels — those active verbs land in Cluster F/G.

## 2. Public Surface

Lifted faithfully from the canonical cluster-B vault spec:

```rust
/// Opaque identifier for a correlated multi-pane cascade event.
/// Contains no human-meaningful label (F11 constraint).
/// Format: "cascade_cluster_{u64_hex}" — 16-hex-char suffix.
pub struct CascadeClusterId(pub String);

/// A single tool-call step as read from atuin history.
#[derive(Debug, Clone)]
pub struct AtuinStep {
    pub id: String,
    pub ts_ns: i64,
    pub command: String,
    pub cwd: String,
    pub session: String,
    pub exit: i32,
}

/// A cc-* dispatch log entry read from atuin history (commands matching cc-*).
#[derive(Debug, Clone)]
pub struct DispatchRecord {
    pub ts_ns: i64,
    pub pane_label: String,
    pub binary: String,
    pub session: String,
}

/// A correlated cascade cluster: a set of AtuinSteps + DispatchRecords
/// that co-occurred within the correlation window.
#[derive(Debug, Clone)]
pub struct CascadeCluster {
    pub cluster_id: CascadeClusterId,
    pub window_start_ns: i64,
    pub window_end_ns: i64,
    pub pane_count: usize,
    pub step_count: usize,
    pub has_temporal_gaps: bool,
    pub dag_depth: usize,
    pub observed_at_ms: i64,
}

#[derive(Debug, Clone)]
pub struct CascadeCorrelatorConfig {
    pub max_gap_ms: i64,                // default 30_000
    pub min_pane_count: usize,          // default 2
    pub window_ms: i64,                 // default 300_000
    pub max_steps_per_cluster: usize,   // default 500
    pub atuin_db_path: std::path::PathBuf,
}

pub struct CascadeCorrelator {
    config: CascadeCorrelatorConfig,
}

impl CascadeCorrelator {
    pub fn new(config: CascadeCorrelatorConfig) -> Self;

    /// Read atuin history rows since `since_ts_ns`, paginated.
    /// # Errors
    /// Returns `CascadeError::AtuinSchemaDrift` if schema-version probe fails;
    /// `CascadeError::AtuinIo` for sqlite errors.
    pub fn read_atuin_since(
        &self,
        since_ts_ns: i64,
        limit: usize,
    ) -> Result<Vec<AtuinStep>, CascadeError>;

    /// Correlate a batch of steps into zero or more CascadeClusters.
    /// Infallible — empty input returns empty Vec.
    pub fn correlate(
        &self,
        steps: &[AtuinStep],
        dispatch_records: &[DispatchRecord],
    ) -> Vec<CascadeCluster>;

    /// Assign a stable opaque CascadeClusterId from a correlated group.
    /// Derivation: fnv1a_64(window_range) XOR fnv1a_64(sorted_pane_labels) XOR (step_count as u64).
    pub fn assign_cluster_id(
        &self,
        window_start_ns: i64,
        window_end_ns: i64,
        pane_labels: &[&str],
        step_count: usize,
    ) -> CascadeClusterId;
}

#[derive(Debug, thiserror::Error)]
pub enum CascadeError {
    #[error("atuin schema drift: expected ns timestamps, got {0}")]
    AtuinSchemaDrift(String),
    #[error("atuin io: {0}")]
    AtuinIo(#[from] rusqlite::Error),
    #[error("empty input for cluster id derivation")]
    EmptyInput,
}
```

`CascadeClusterId` is a newtype (`pub struct CascadeClusterId(pub String);`) per the workflow-trace newtype discipline ([GOLD_STANDARDS.md](../../../GOLD_STANDARDS.md) rule 8). Its `Display` impl prints only `cascade_cluster_<hex>` — never the source pane labels. This is the F11 mitigation core and is property-tested.

## 3. Internal Data Structures

**Step graph (in-memory, ephemeral):** `HashMap<String, Vec<usize>>` adjacency list keyed by `session` with values into the step batch. Edge inference: two steps form a DAG edge A→B iff (i) they share `cwd` OR (ii) B's `ts_ns` falls within `max_gap_ms` of A's `ts_ns` and B is in a different `session`. Kahn's topological sort runs over this graph to compute `dag_depth` and surface unexpected cycles (logged via `tracing::warn!`, never silently accepted).

**Cluster accumulator:** `Vec<Vec<usize>>` (cluster index → step indices). Built by a sliding-window scan: as each step enters the window, it is candidate-merged into the most-recent open cluster whose `window_end_ns + max_gap_ms >= step.ts_ns`. If no open cluster qualifies, a new one opens. Clusters close when a gap exceeds `max_gap_ms` with no new arrivals.

**Cluster ID derivation:** `cascade_cluster_{:016x}` where the hex value is:

```text
hash_A = fnv1a_64(format!("{window_start_ns}:{window_end_ns}"))
hash_B = fnv1a_64(sorted_pane_labels.join(","))
id_u64 = hash_A ^ hash_B ^ (step_count as u64)
```

FNV-1a 64-bit is chosen for zero external dependency and acceptable collision resistance at expected cardinality (≪10^6 clusters per habitat lifetime). The FNV constants live in `src/m4_cascade/cluster_id.rs` with the standard `FNV_OFFSET_BASIS = 0xcbf29ce484222325` and `FNV_PRIME = 0x100000001b3` per the cluster-B V7 plan. Collision-detected? Increment `step_count` by 1 and rehash — the resolved id remains opaque (F11 preserved by construction; the modifier is purely numeric).

## 4. Data Flow

```text
atuin history.db (read-only, PRAGMA query_only = ON, busy-timeout 5000ms)
    | read_atuin_since() — lazy cursor pagination
    v
Vec<AtuinStep>                                Vec<DispatchRecord>
   \                                          / (atuin rows matching cc-*)
    \________________  correlate()  _________/
                          | sliding-window scan
                          | DAG edge inference
                          | Kahn topological sort
                          | gap-allowed sub-graph detection
                          v
                  Vec<CascadeCluster>  (emitted; m4 holds no DB handle)
                          v
            Cluster C — m7_workflow_runs writer (CC-1 owner)
                          v
            Cluster F — m20_prefixspan_miner (cohort reader)
            Cluster E — m14_habitat_outcome_lift (lift cohort)
```

m4 **does NOT write to any database**. The observation contract is pure-functional from input-batch to emitted `Vec<CascadeCluster>`. m7 owns the SQLite write path; m13 owns the stcortex emit path. This decoupling keeps m4 unit-testable without a live DB and is load-bearing for CC-1 (the m4 ↔ m6 join happens only inside m7's JSONB `consumer_inputs` column — never directly between modules).

## 5. Boilerplate Lifts

Per [`the-workflow-engine-vault/boilerplate modules/BOILERPLATE_INDEX.md`](../../../the-workflow-engine-vault/boilerplate%20modules/BOILERPLATE_INDEX.md):

| Source | LOC reused | Pattern lifted | Notes |
|---|---:|---|---|
| `m49_task_graph.rs` (category 04) | ~50 | Kahn's topological sort + node/edge adjacency representation + `TaskNodeState` FSM shape | Tool-call steps replace pane tasks as nodes; cycle-detection logic retained verbatim. |
| `m20_heat_source_hebbian.rs` (category 04) | ~30 | O(n²) `CoActivationPair` pairwise iteration pattern | Informs pairwise gap-detection inner loop; N-step generalisation that groups pairs into sub-graph identity is **fresh** (~80 LOC). |
| FNV-1a 64-bit primitive | ~20 | Standard constant table + multiply-then-XOR loop | No upstream crate dependency (god-tier rule: no unneeded deps). |

**Boilerplate lift density: ≈50%.** The remaining ~80 LOC (sliding-window scan + gap-allowed sub-graph identity + cluster ID derivation) is the structural new-authorship piece for F11.

## 6. ME v2 Patterns

Per [PATTERNS.md](../../../PATTERNS.md) § "Module-level patterns (per ME v2 m1_foundation)":

- **`resources.rs` `//!` docstring style:** module header carries Layer / Deps / Tests / Features / Platform / Impl Notes / Related Docs (per [GOLD_STANDARDS.md](../../../GOLD_STANDARDS.md) "File header convention").
- **`signals.rs` `SignalContext`-style timestamping:** `observed_at_ms` is captured once at the entry of `correlate()`, never per-step inside the loop. Avoids clock-drift across a batch.
- **`error.rs` thiserror taxonomy:** `CascadeError` is a thiserror enum; no `Box<dyn Error>` in the public surface (god-tier rule 9).
- **`shared_types.rs` newtype discipline:** `CascadeClusterId` is a newtype, not a raw `String` (rule 8). `AtuinStep::session`, `AtuinStep::id`, `DispatchRecord::session` remain `String` for now — promotion to `SessionId`/`StepId` newtypes is deferred to the `workflow_core::types` consolidation (Cluster D aspect-wrap).
- **`metrics.rs` rolling-window framing:** the sliding window is conceptually equivalent to ME v2's rolling-window metrics; no shared code, but the framing matches.
- **`logging.rs` tracing-subscriber emit:** cycle detection (unexpected in practice) emits `tracing::warn!` with structured fields `{cluster_window_start, cluster_window_end, cycle_node_count}`; never `println!`/`eprintln!` (god-tier rule 7).

## 7. Constraints Satisfied

- **F11 cascade-monoculture (exclusive owner):** `CascadeClusterId` is opaque by construction. Hash mixing destroys any semantic signal about cascade content. No human-readable name is embedded. The `Display` impl emits only `cascade_cluster_<16-hex>`. Property-tested with a 10k-pair label-substring fuzz target asserting no `ALPHA`/`BETA`/`GAMMA`/`LEFT`/`RIGHT`/`TR`/`BR` leak.
- **Phase A passive verbs (retained under override per § 1.b):** observe (atuin reads), correlate (sub-graph detection), record (cluster_id assigned), emit (`Vec<CascadeCluster>` returned). Never recommend, optimise, select, or label.
- **No direct DB write:** m4 holds no DB connection handle for output. Atuin DB is opened **read-only** (`PRAGMA query_only = ON` per [PATTERNS.md](../../../PATTERNS.md) § "Lazy cursor-based pagination").
- **AP30 namespace discipline (consumed via Cluster D aspect-wrap):** when m4's `cluster_id` propagates downstream to m13 stcortex writes, m9 namespace guard validates the `workflow_trace_*` prefix at runtime. m4 emits the raw id; the prefix is appended by m13/m42, not m4.
- **AP-V7-09 substrate-frame discipline:** opaque IDs are precisely the substrate-frame mechanism that prevents anthropocentric "this cluster is about X" rationalisation downstream.
- **God-tier rules ([GOLD_STANDARDS.md](../../../GOLD_STANDARDS.md)):** zero `unwrap()` outside tests (rule 1); zero `unsafe` (rule 2); thiserror error enum (rule 9); doc comments on all public items (rule 6); structured tracing emit (rule 7); newtype discipline (rule 8).

## 8. Tests (≥60, per TEST_DISCIPLINE matrix row m4)

Allocation per [`ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md`](../../../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md) and cluster-B V7 plan:

| Pattern | Count | Focus |
|---|---:|---|
| Unit | 30 | `assign_cluster_id` per input variant; `correlate` for sorted/unsorted/empty/single-row; `read_atuin_since` pagination; Kahn `dag_depth`; cycle detection; min_pane_count filter; gap boundary (29_999ms clusters; 30_001ms splits); max_steps_per_cluster boundary; `CascadeClusterId::Display` non-leakage. |
| Property | 8 | (a) `for_all (window, labels) -> !id.contains(label)` opaque-ID invariant; (b) sorted-pane-label permutation determinism; (c) XOR-fold associativity; (d) same-input id stability across invocations; (e) different-input id distinctness (10k-pair sweep, collision <1%); (f) cluster_id matches `^cascade_cluster_[0-9a-f]{16}$`; (g) step_count delta produces distinct id; (h) window-range hash injectivity at sample scale. |
| Fuzz | 1 | `m4_cascade_id_fuzz` — random UTF-8 bytes into `assign_cluster_id`; assertion = no `ALPHA`/`BETA`/`GAMMA` substring leak in output Display. |
| Integration | 15 | m4 ↔ m1 wiring (real atuin batch fixture); m4 → m7 join schema (snapshot test); m4 → m31 diversity-check downstream; empty atuin DB; mixed cc-dispatch/cc-cascade/cc-broadcast detection; overlap-boundary batch; concurrent cascade derivation across multiple sessions; schema-drift error path; busy-timeout retry; PRAGMA enforcement. |
| Contract | 3 | `CascadeCluster` schema snapshot (insta); `CascadeClusterId` Display stability snapshot; `CascadeError` variant snapshot. |
| Regression | 3 | F11 leak regression slot (filled on first incident); cluster-A schema-change regression; XOR-commutativity regression. |
| **Mutation budget** | — | **≥70% kill** on `cluster_id.rs` (the F11 core). |

Every test carries an inline `// rationale: <constraint or invariant>` comment per [`ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md`](../../../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md) § rationale-comment rule. Property tests run ≥10k iterations per invariant.

## 9. Cross-Cluster Contracts

- **CC-1 Cascade-Cost Coupling (m4 contributes; m7 owns the join):** m4 emits `CascadeCluster` with `cluster_id`; Cluster C's `m7_workflow_runs` writes `cascade_cluster_id` into the `workflow_runs` row keyed by `session_id`. m6's cost columns join via the same `session_id`. m4 and m6 are NEVER directly coupled — the join lives inside m7's JSONB `consumer_inputs` column. This decoupling is load-bearing for F11.
- **CC-3 Evidence-Driven Iteration (m4 contributes; m14 owns lift; m20-m23 consume):** m4's cluster IDs anchor the cohorts that `m14_habitat_outcome_lift` aggregates and `m20_prefixspan_miner` mines. Without m4, Cluster F has no coherent multi-pane event identity to reason over.
- **CC-2 Trust Layer Woven (aspect-IN):** m4 is aspect-wrapped at write-time by `m9_watcher_namespace_guard` when `cluster_id` propagates into m13 stcortex writes. `m8_povm_build_prereq` compile-time gate applies engine-wide.

## 10. Failure Modes

- **F11 cascade monoculture (exclusive owner) — HIGH:** any code path that exposes a human-readable pane label in the cluster identifier collapses exploration evidence into anthropocentric noise. Mitigation: FNV-1a XOR derivation; Display non-leakage; F-Fuzz target; F-Property invariant; F-Regression slot.
- **F3 substrate-input poisoning — MEDIUM:** corrupted/oversized atuin rows could OOM the in-memory step graph. Mitigation: `read_atuin_since` enforces `limit` parameter; `correlate` enforces `max_steps_per_cluster` ceiling; corrupted inputs surface as `CascadeError::AtuinSchemaDrift`, not panic.
- **AP-V7-09 substrate-frame engine confusion — MEDIUM:** downstream module attempts to back-decode `cluster_id` into pane labels. Mitigation: opaque-ID construction is one-way (FNV-1a is non-invertible at this cardinality); `CascadeClusterId` is a newtype, blocking accidental string-parsing patterns; Watcher Class G pre-positioned (see § 12).
- **Clock-skew (multi-host expansion — DEFERRED):** gap-allowed matching uses wall-clock `ts_ns`. Single-host habitat is safe; flagged for multi-host expansion in § 12.

## 11. LOC Estimate

| Section | LOC |
|---|---:|
| Types (`CascadeClusterId`, `AtuinStep`, `DispatchRecord`, `CascadeCluster`, `CascadeCorrelatorConfig`) | ~50 |
| `read_atuin_since` (rusqlite paginated read + PRAGMA + schema-version probe) | ~25 |
| `correlate` (sliding window + DAG construction + Kahn's sort + gap-allowed matching) | ~70 |
| `assign_cluster_id` (FNV-1a XOR + format!) | ~15 |
| `CascadeError` enum | ~20 |
| **Total** | **~180** |

src/ layout per [`ai_docs/optimisation-v7/GENERATIONS/G2-consolidation.md`](../../../ai_docs/optimisation-v7/GENERATIONS/G2-consolidation.md) § Canonical src/ layout: `src/m4_cascade/{mod.rs, cluster_id.rs, window.rs, derive.rs, error.rs}`. **Unpadded** module ID per [GENESIS_PROMPT_V1_3.md](../../../ai_docs/GENESIS_PROMPT_V1_3.md) § 1 OI-4 lock.

## 12. Open Questions

1. **Atuin schema-version probe:** atuin currently uses integer nanosecond timestamps. If atuin migrates to microseconds, the sliding-window arithmetic breaks silently. Spec mandates a schema-version check on first `read_atuin_since` open — but the exact probe SQL is not yet defined. **Decision needed (Luke / Zen):** pin atuin schema version in `CascadeCorrelatorConfig` and fail-loud on mismatch, or detect-and-adapt?
2. **cc-* prefix configurability:** `DispatchRecord` filter currently hardcodes the `cc-` prefix. New fleet binaries with a different prefix (`hb-`, `fleet-`) would be missed. **Decision needed (Luke):** configurable prefix list in `CascadeCorrelatorConfig` (e.g., `Vec<String>` with default `["cc-", "hb-", "fleet-"]`), or stay hardcoded for M0 and broaden post-soak?
3. **Clock-skew across panes (multi-host expansion):** single-host habitat is safe; multi-host expansion (e.g., openclaw-gateway container running cc-* against host atuin) introduces clock-skew that breaks `max_gap_ms` matching. **Watcher escalation:** flag for Class G surveillance Phase 2B+.
4. **Collision-resolved cluster_id ordering:** if two cascades produce the same `id_u64` and the resolver bumps `step_count` by 1, the resolved id is technically derived from a different `step_count` than the cluster's actual cardinality. Downstream readers should never use cluster_id arithmetic — only string equality. **Recommended:** add a `// rationale: collision-resolved; do not interpret hex value` doc comment on `assign_cluster_id`.

## 13. Bidirectional Anchors

> **Back to:** [CLAUDE.md](../../../CLAUDE.md) · [CLAUDE.local.md](../../../CLAUDE.local.md) · [MODULE_MATRIX.md](../../MODULE_MATRIX.md)
>
> **Sister modules (Cluster B):** [m4](m4_cascade_correlator.md) · [m5](m5_battern_step_record.md) · [m6](m6_context_cost.md)
>
> **Vault canonical:** [[cluster-B-habitat-observers]] (~/claude-code-workspace/the-workflow-engine/the-workflow-engine-vault/module specs/cluster-B-habitat-observers.md)
>
> **V7 planning:** [cluster-B plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-B.md) · [ULTRAMAP.md](../../../ai_docs/optimisation-v7/ULTRAMAP.md) View 2 row m4 · [TEST_DISCIPLINE.md](../../../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md) · [GOD_TIER_RUST.md](../../../ai_docs/optimisation-v7/STANDARDS/GOD_TIER_RUST.md) · [ANTIPATTERNS_REGISTER.md](../../../ai_docs/optimisation-v7/ANTIPATTERNS_REGISTER.md) § AP-WT-F11
>
> **Binding spec:** [GENESIS_PROMPT_V1_3.md](../../../ai_docs/GENESIS_PROMPT_V1_3.md) § 1 (architecture) · § 3 (verb-class)
>
> **Standards mirror:** [PATTERNS.md](../../../PATTERNS.md) · [GOLD_STANDARDS.md](../../../GOLD_STANDARDS.md) · [ANTIPATTERNS.md](../../../ANTIPATTERNS.md)
>
> **Watcher class pre-position:** Class A (first `correlate` post-Genesis) · Class G (substrate-frame confusion if `cluster_id` back-decoded) · Class D (four-surface drift if Display reveals source semantics)

*m4 spec v1 · authored 2026-05-17 (S1001982) · planning-only · HOLD-v2 active · no .rs files emitted*
