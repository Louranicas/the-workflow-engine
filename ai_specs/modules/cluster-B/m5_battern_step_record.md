---
title: m5 — battern_step_record — Per-Module Spec
date: 2026-05-17 (S1001982)
status: planning-only · HOLD-v2 active · markdown-only · NO .rs files
cluster: B — Habitat Observation
layer: L2
module_id: m5
binary: wf-crystallise
verb_class: passive (observe · record · emit)
loc_estimate: ~150
test_budget: 55
mutation_kill_threshold: 0.70
feature_gate: none
cc_owned: CC-1b (m5 ↔ m6 via battern_id↔session_id) · CC-3 (evidence-enabling)
cc_consumed: —
gap_owner: F1 (bank/name ossification) — exclusive at observation layer
boilerplate_lift_pct: 35
status_row: SPEC
---

# m5 — `battern_step_record` — Per-Module Spec

> **Back to:** [CLAUDE.md](../../../CLAUDE.md) · [CLAUDE.local.md](../../../CLAUDE.local.md) · [MODULE_MATRIX.md](../../MODULE_MATRIX.md) · vault [[cluster-B-habitat-observers]] · canonical V7 [cluster-B plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-B.md) · binding spec [GENESIS_PROMPT_V1_3.md](../../../ai_docs/GENESIS_PROMPT_V1_3.md) § 1
>
> **Sister modules (Cluster B):** [m4](m4_cascade_correlator.md) · [m5](m5_battern_step_record.md) · [m6](m6_context_cost.md)
>
> **Cross-cluster anchors:** Cluster A upstream — [m1_atuin_consumer](../cluster-A/m1_atuin_consumer.md) · [m3_injection_db_consumer](../cluster-A/m3_injection_db_consumer.md) · Cluster C hub — [m7_workflow_runs](../cluster-C/m7_workflow_runs.md) · Cluster D aspect-wrap — [m9_watcher_namespace_guard](../cluster-D/m9_watcher_namespace_guard.md) · Cluster F downstream — [m20_prefixspan_miner](../cluster-F/m20_prefixspan_miner.md) (m21 battern_iterator reads m5 step records)
>
> **Standards:** [PATTERNS.md](../../../PATTERNS.md) · [GOLD_STANDARDS.md](../../../GOLD_STANDARDS.md) · [ANTIPATTERNS.md](../../../ANTIPATTERNS.md) · skill anchor [`.claude/skills/battern-protocol/SKILL.md`](../../../.claude/skills/battern-protocol/SKILL.md)

---

## 1. Purpose

`m5_battern_step_record` observes executions of the **Battern protocol** — the 6-step fleet coordination pattern (Design / Dispatch / Gate / Collect / Synthesize / Compose) used habitat-wide for multi-pane work — recording per-step durations and outcomes keyed on a `battern_id`. It is the only module in the engine that directly instruments the Battern protocol. Without m5, the protocol's execution cost and success rate are invisible to workflow-trace.

m5's load-bearing invariant: **it does NOT impose the 6-step schema onto observed substrate.** The 6 canonical step names are held as a *soft schema* — heuristics used to *label* observed steps where the substrate's actual pattern matches. If a battern execution diverges (gate step skipped, synthesize step repeated, an unrecognised step interposed), the raw observation is preserved with `step_label = None`. **No step is discarded because it doesn't fit.** This is the exclusive owner of **F1 (bank/name ossification)** at the observation layer per [`ai_docs/optimisation-v7/ANTIPATTERNS_REGISTER.md`](../../../ai_docs/optimisation-v7/ANTIPATTERNS_REGISTER.md) § AP-WT-F1.

Verb budget (Phase A passive, retained under override per [GENESIS_PROMPT_V1_3.md](../../../ai_docs/GENESIS_PROMPT_V1_3.md) § 3): **observe · record · emit**. m5 never recommends a step shape, optimises a battern, or labels an unmatched step as `Other` — `None` is the structural guarantee.

## 2. Public Surface

Lifted faithfully from the canonical cluster-B vault spec:

```rust
/// Opaque identifier for one Battern protocol execution.
/// Format: "battern_{u64_hex}". No semantic content.
pub struct BatternId(pub String);

/// A single observed step within a battern execution.
#[derive(Debug, Clone)]
pub struct BatternStepObservation {
    pub battern_id: BatternId,
    pub step_index: usize,
    /// None when the step did not match any canonical heuristic.
    /// NEVER substituted with a placeholder enum variant (F1 invariant).
    pub step_label: Option<BatternStepLabel>,
    pub duration_ms: i64,
    pub session: String,
    pub exit_code: i32,
    pub is_partial: bool,
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
    pub total_steps: usize,
    pub labelled_steps: usize,
    pub failed_steps: usize,
    pub total_duration_ms: i64,
    pub is_complete: bool,
    pub is_partial: bool,
}

#[derive(Debug, Clone)]
pub struct BatternStepRecordConfig {
    pub inter_step_timeout_ms: i64,    // default 1_800_000 (30 min)
    pub min_steps: usize,              // default 2
    pub atuin_db_path: std::path::PathBuf,
}

pub struct BatternStepRecord {
    config: BatternStepRecordConfig,
}

impl BatternStepRecord {
    pub fn new(config: BatternStepRecordConfig) -> Self;

    /// Infer battern boundaries from a batch of AtuinSteps; emit step observations.
    /// Infallible — empty input returns empty Vec.
    pub fn observe(
        &self,
        steps: &[crate::m4::AtuinStep],
    ) -> Vec<BatternStepObservation>;

    /// Build a summary BatternRecord from observations sharing one battern_id.
    pub fn summarise(observations: &[BatternStepObservation]) -> BatternRecord;
}

#[derive(Debug, thiserror::Error)]
pub enum BatternError {
    #[error("atuin io: {0}")]
    AtuinIo(#[from] rusqlite::Error),
    #[error("regex compile: {0}")]
    RegexCompile(#[from] regex::Error),
}
```

`BatternId` is a newtype (god-tier rule 8). `step_label: Option<BatternStepLabel>` is the F1 structural guarantee — there is no `BatternStepLabel::Other` variant by design. Adding one would constitute a v1.3 violation.

## 3. Internal Data Structures

**Open battern accumulator:** `HashMap<BatternId, Vec<BatternStepObservation>>` accumulates observations for batterns not yet closed. A battern closes when:
- A new Design→Dispatch transition is detected (closes the previous; opens a new id).
- `inter_step_timeout_ms` elapses since the last step in the open battern (closed as timeout-complete).
- The input batch ends (remaining open batterns emit with `is_partial = true`).

**Step label heuristic table:** `Vec<(Regex, BatternStepLabel)>` compiled once at `BatternStepRecord::new()` using the `regex` crate (already used in habitat hooks). The first matching heuristic wins; if no heuristic matches, `step_label = None`. The table:

| Step | Detection heuristic |
|---|---|
| Design | `cc-dispatch` preceded within 5s by a planning-scope command (`rg`, `atuin search`, `Read`) |
| Dispatch | `cc-dispatch` to ≥2 distinct panes within the Design window |
| Gate | `cc-health` / `curl .*:[0-9]+/health` / explicit `gate-check`-pattern commands |
| Collect | `cc-harvest` / `cc-audit` / `atuin search` within expected collection window |
| Synthesize | Long-running (`duration > 10s`) commands not matching dispatch/collect |
| Compose | `cc-cascade` / final aggregation commands / explicit compose-marker |

**Battern-id derivation:** `battern_id = format!("battern_{:016x}", fnv1a_64(&first_dispatch_ts_ns.to_string()))`. Deterministic given first dispatch timestamp; collision-resistant at expected habitat cardinality (<10^4 batterns/lifetime); free of semantic content. Same FNV-1a primitive as m4 (lifted from `src/m4_cascade/cluster_id.rs`).

**Hyphen-slug discipline:** per [CLAUDE.md](../../../CLAUDE.md) § Operational rules ("Hyphen-slug discipline for stcortex — labels with hyphens converted to underscores"), when `battern_id` propagates downstream to m13/m42 stcortex writes, hyphens in any associated slug are converted to underscores at the m13/m42 boundary — m5 emits raw ids. Mitigates AP-Hab-11.

## 4. Data Flow

```text
Vec<AtuinStep>                Vec<m3::InjectionEvent>  (optional label hints)
   (from m1 / m4 read)              (from m3_injection_db_consumer)
        \                            /
         \________ observe() _______/
                  | Design->Dispatch boundary detection
                  | step labelling (soft schema, Option<>)
                  | open-battern accumulator
                  | inter-step timeout sweep
                  v
        Vec<BatternStepObservation>  (emitted)
                  |
                  +---- summarise() -> BatternRecord (per-battern roll-up)
                  v
        Cluster C — m7 battern_observations table writer
        Cluster F — m21 battern_iterator (cohort reader)
```

m5 **does NOT write to any database.** All emission is via returned `Vec<_>`. m7 owns the SQLite write path (`battern_observations` table); m13 owns the stcortex emit. m5's join with m6 (CC-1b) lives in m7's JSONB `consumer_inputs` column via `(battern_id, session_id)` tuples — m5 and m6 are NEVER directly coupled.

## 5. Boilerplate Lifts

Per [`the-workflow-engine-vault/boilerplate modules/BOILERPLATE_INDEX.md`](../../../the-workflow-engine-vault/boilerplate%20modules/BOILERPLATE_INDEX.md):

| Source | LOC reused | Pattern lifted | Notes |
|---|---:|---|---|
| `m4_cascade/window.rs` (sibling module) | ~30 | Sliding-window boundary detection + FNV-1a primitive | Direct intra-cluster reuse; same window semantics, different boundary trigger. |
| `m49_task_graph.rs` (category 04) | ~5 | `TaskNodeState` FSM shape informs `BatternStepLabel` enum (structural inspiration only) | No direct code lifted; soft schema design influence. |
| `.claude/skills/battern-protocol/SKILL.md` (category 09) | 0 | Heuristic table for step detection (Design/Dispatch/Gate/Collect/Synthesize/Compose names + recognition patterns) | Skill prose informs the regex table; no code lifted. |
| `memory-injection/m11_parallel_query.rs::elapsed_ms` helper | ~10 | Timestamp normalisation | ~40% reuse. |

**Boilerplate lift density: ≈35%.** Fresh authorship dominates — step-label option-discipline, Design→Dispatch boundary inference, partial-battern flagging.

## 6. ME v2 Patterns

Per [PATTERNS.md](../../../PATTERNS.md) § "Module-level patterns (per ME v2 m1_foundation)":

- **`resources.rs` `//!` docstring style:** module header carries Layer / Deps / Tests / Features / Platform / Impl Notes / Related Docs.
- **`signals.rs` `SignalContext`-style timestamping:** `recorded_at_ms` captured once at entry of `observe()`, not per-step.
- **`error.rs` thiserror taxonomy:** `BatternError` enum; no `Box<dyn Error>` exposed.
- **`shared_types.rs` newtype discipline:** `BatternId` is a newtype.
- **Builder pattern:** `BatternStepRecord::new(config)` — no bare struct construction; config carries all knobs.
- **`logging.rs` tracing-subscriber emit:** partial-battern emission logs `tracing::info!` with structured `{battern_id, step_count, is_partial}` — not `println!`.
- **`metrics.rs` framing:** total_duration_ms / failed_steps are roll-up metrics consumed downstream by m12 CLI reports.

## 7. Constraints Satisfied

- **F1 bank/name ossification (exclusive owner at observation layer):** `step_label: Option<BatternStepLabel>`. There is NO `Other` variant; there is NO substitution from `None` to `Other`; there is NO discard of unlabelled steps. Tested by F-Property round-trip + F-Regression slot.
- **No imposed 6-step mould:** divergent batterns (5, 7, 12 steps) emit all observations. `total_steps` and `labelled_steps` are separately recorded so downstream readers can see the schema-divergence rate.
- **Opaque `battern_id`:** F11 discipline (same as m4). `battern_` prefix is a namespace marker, not a semantic label.
- **Phase A passive verbs (retained under override per § 1.b):** observe, record, emit. Never optimise or propose action.
- **No direct DB write:** m5 emits observations; m7 owns the write path; m13/m42 own the stcortex emit.
- **AP30 namespace discipline:** propagated `battern_id` is aspect-wrapped by m9 namespace guard at m13/m42 boundary; m5 emits raw id.
- **AP-Hab-11 hyphen-slug munge:** mitigated downstream at m13/m42 (m5 emits raw); flagged in spec doc comment.
- **God-tier rules ([GOLD_STANDARDS.md](../../../GOLD_STANDARDS.md)):** zero `unwrap()` outside tests; zero `unsafe`; thiserror enum; doc comments on all public items; structured tracing emit; newtype discipline.

## 8. Tests (≥55, per TEST_DISCIPLINE matrix row m5)

Allocation per [`ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md`](../../../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md) and cluster-B V7 plan:

| Pattern | Count | Focus |
|---|---:|---|
| Unit | 28 | `BatternStepObservation` construction per step_label arm (None + 6 named); Design→Dispatch detection (positive + negative); boundary closes previous battern; inter-step timeout closure; partial flag on mid-execution start; unlabelled-steps preserved (3-step batch, 1 labelled → 3 observations); min_steps filter discards 1-step battern; `summarise` total_duration_ms / failed_steps / is_complete / labelled_steps. |
| Property | 6 | (a) step_label round-trip: `parse(emit(label)) == label`; (b) `None` preserved through serialisation; (c) `step_count` non-decreasing within a battern_id; (d) `BatternId` stability — same first_dispatch_ts_ns → same id (10k-pair sweep, collision <0.1%); (e) different ts_ns → different id; (f) `summarise(observations).total_steps == observations.len()`. |
| Fuzz | 0 | (no fuzz target this module; F1 protection lives in F-Property + F-Regression). |
| Integration | 15 | m5 ↔ m1 wiring (real atuin batch fixture); m5 ↔ m3 label-hint plumbing; m5 → m7 join with `consumer_inputs` JSONB; concurrent batterns across sessions; divergent battern (7 steps) emits all 7; partial battern at batch end; inter-step timeout straddling batch boundary; empty step batch; single-pane batch with cc-dispatch; multi-pane Design→Dispatch within 5s; regex heuristic table compilation on `new()`; AP-Hab-11 hyphen propagation (downstream contract test). |
| Contract | 3 | `BatternStepRow` schema snapshot (insta); `BatternRecord` snapshot; `BatternStepLabel` enum snapshot. |
| Regression | 3 | F1 regression slot (any commit forcing `label = Other` when `None` was correct); 6-step-shape regression (divergent batterns must not be coerced); TTL-sweep timestamp regression (per [feedback_ttl_sweep_test_timestamps](../../../.claude/projects/-home-louranicas-claude-code-workspace/memory/feedback_ttl_sweep_test_timestamps.md): tests use realistic `now_ms() + i`, not `0..n`). |
| **Mutation budget** | — | **≥70% kill** on step-label option-discipline path (`step_label.rs`). |

Every test carries `// rationale: F1 — …` per [`ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md`](../../../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md) § rationale-comment rule.

## 9. Cross-Cluster Contracts

- **CC-1b Battern-Cost Coupling (m5 ↔ m6 via session_id within battern_id range):** m5 records steps keyed on `(battern_id, session_id)`; m6 records cost keyed on `session_id`. Cluster F's `m21_battern_iterator` joins them via m7's JSONB `consumer_inputs` for cost-per-step approximations. m5 and m6 are NEVER directly coupled.
- **CC-3 Evidence-Driven Iteration:** m5's `BatternStepObservation` rows are the trajectory input for Cluster F's `m21_battern_iterator`, which proposes variant batterns under the gradient-preservation discipline.
- **CC-2 Trust Layer Woven (aspect-IN):** `m8_povm_build_prereq` compile-time gate; `m9_watcher_namespace_guard` runtime prefix validation on `battern_id` payloads at m13/m42 boundary.

## 10. Failure Modes

- **F1 bank/name ossification (exclusive owner) — HIGH:** any code path that substitutes `Some(BatternStepLabel::Other)` for `None`, or discards unlabelled steps, collapses the substrate's actual cadence into a 6-step prescription. Mitigation: `Option<>` discipline at the type level; no `Other` variant exists; F-Regression slot; mutation-test kill ≥70% on `step_label.rs`.
- **AP-Hab-11 hyphen-slug munge — MEDIUM:** if `battern_id` enters a hyphenated stcortex namespace path verbatim, the S1001757 munge bug reappears. Mitigation: hyphen→underscore at m13/m42 boundary; m5 emits raw and flags in doc.
- **AP-Test-05 assertion drift — LOW:** test assertions drifting from the constraint they prove. Mitigation: per-test `// rationale: F1 …` comments link every assertion to its constraint.
- **Design→Dispatch false positive — LOW:** a user manually invoking `cc-dispatch` without a planning prefix could trigger a spurious battern open. Mitigation: `min_steps` filter discards 1-step batterns; spec flags `require_planning_prefix` configurability for future hardening (see § 12).

## 11. LOC Estimate

| Section | LOC |
|---|---:|
| Types (`BatternId`, `BatternStepObservation`, `BatternStepLabel`, `BatternRecord`, `BatternStepRecordConfig`) | ~55 |
| Boundary detection + step labelling in `observe()` | ~60 |
| `summarise()` | ~15 |
| `BatternStepRecord::new()` + heuristic regex table init | ~20 |
| **Total** | **~150** |

src/ layout per [`ai_docs/optimisation-v7/GENERATIONS/G2-consolidation.md`](../../../ai_docs/optimisation-v7/GENERATIONS/G2-consolidation.md): `src/m5_battern/{mod.rs, step_label.rs, derive.rs, error.rs}`. Unpadded module ID.

## 12. Open Questions

1. **Design→Dispatch heuristic window:** 5-second window is tunable but may produce false positives when a user manually runs `cc-dispatch` without a planning prefix. **Decision needed (Luke / Zen):** add `require_planning_prefix: bool` to `BatternStepRecordConfig` (default `true`)? Or rely on `min_steps` filter alone?
2. **Multi-session batterns (cross-session context handoff):** current m5 closes a battern when the atuin window ends; it does not attempt cross-session battern reconstruction. **Watcher escalation:** flag as known limitation in module doc; consider Phase 2+ extension via m3 injection-event chaining.
3. **6-step taxonomy evolution:** if the Battern protocol adds a 7th step (e.g., Verify between Compose and battern-close), the `BatternStepLabel` enum and heuristics table need updating. **Decision needed (Luke):** keep enum closed (forces a versioned spec update on protocol evolution — explicit), or move to a config-driven heuristics TOML for forward-compat (implicit, more flexible)? Recommended: keep closed for v1 to preserve F1 discipline; protocol change is a deliberate event.
4. **stcortex consumer-freshness gate:** when m13/m42 propagates `battern_id` to stcortex, the refuse-write-at-DB-layer enforces consumer freshness. **Watcher:** confirm m5's emit cadence aligns with consumer hooks registered each session per [CLAUDE.md](../../../CLAUDE.md) § Memory Systems row 8.

## 13. Bidirectional Anchors

> **Back to:** [CLAUDE.md](../../../CLAUDE.md) · [CLAUDE.local.md](../../../CLAUDE.local.md) · [MODULE_MATRIX.md](../../MODULE_MATRIX.md)
>
> **Sister modules (Cluster B):** [m4](m4_cascade_correlator.md) · [m5](m5_battern_step_record.md) · [m6](m6_context_cost.md)
>
> **Vault canonical:** [[cluster-B-habitat-observers]] (~/claude-code-workspace/the-workflow-engine/the-workflow-engine-vault/module specs/cluster-B-habitat-observers.md)
>
> **V7 planning:** [cluster-B plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-B.md) · [ULTRAMAP.md](../../../ai_docs/optimisation-v7/ULTRAMAP.md) View 2 row m5 · [TEST_DISCIPLINE.md](../../../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md) · [GOD_TIER_RUST.md](../../../ai_docs/optimisation-v7/STANDARDS/GOD_TIER_RUST.md) · [ANTIPATTERNS_REGISTER.md](../../../ai_docs/optimisation-v7/ANTIPATTERNS_REGISTER.md) § AP-WT-F1
>
> **Binding spec:** [GENESIS_PROMPT_V1_3.md](../../../ai_docs/GENESIS_PROMPT_V1_3.md) § 1 (architecture) · § 3 (verb-class)
>
> **Skill anchor:** [`.claude/skills/battern-protocol/SKILL.md`](../../../.claude/skills/battern-protocol/SKILL.md) (informs heuristic table; no code lifted)
>
> **Standards mirror:** [PATTERNS.md](../../../PATTERNS.md) · [GOLD_STANDARDS.md](../../../GOLD_STANDARDS.md) · [ANTIPATTERNS.md](../../../ANTIPATTERNS.md)
>
> **Watcher class pre-position:** Class A (first `BatternStepRow` post-Genesis with `step_label = Some(Design)`) · Class B (hand-off boundary — every Gate-step is a cross-substrate hand-off) · Class G (substrate-frame confusion if `step_label` back-decoded to ascribe agentic intent)

*m5 spec v1 · authored 2026-05-17 (S1001982) · planning-only · HOLD-v2 active · no .rs files emitted*
