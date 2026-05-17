---
title: workflow-trace — Genesis Prompt v0
date: 2026-05-17 (S1001971 / S1001982)
authority: Luke @ node 0.A presides
authored_by:
  - Command (Tab 1 top-left — convener)
  - Command-2 (Tab 1 middle-left — Path A chair)
  - Command-3 (Tab 1 middle-right — Path B chair + CR-2 owner)
  - The Watcher ☤ (substrate observer, R13 elapsed, full standing)
  - Zen (Tab "Zen" — Pi code-review agent, audit lane)
kind: GENESIS-PROMPT-V0
status: DRAFT-V1 · pre-build · awaiting Luke `start coding workflow-trace` signal
priors_chronological:
  - 2026-05-16T203752Z_command3_handshake_to_command_and_command2.md
  - 2026-05-16T212306Z_command3_townhall_final_args_workflow_librarian.md
  - 2026-05-16T215114Z_command3_townhall_boilerplate_recon_wave2_synthesis.md
  - the-workflow-engine/THE_WORKFLOW_ENGINE_CIRCLE_OF_EXPERTS_S1001982.md
  - the-workflow-engine/THE_WORKFLOW_ENGINE_TOWN_HALL_S1001982.md
  - the-workflow-engine/THE_WORKFLOW_ENGINE_BOILERPLATE_HUNT_S1001982.md
  - the-workflow-engine/CONVERGENCE_COMMAND_X_COMMAND3_S1001982.md
  - the-workflow-engine/INTERVIEW_QUESTION_BANK_DRAFT.md
  - 2026-05-16T215959Z_watcher_cr2_candidate_a_concur_plus_cross_acks.md
  - 2026-05-16T215920Z_command2_townhall_close_three_proposals_watcher_acks_na_pass.md
  - 2026-05-16T221117_command3_cr2_cr2b_shipped_close.md (CR-2 + CR-2b SHIPPED)
  - 2026-05-16T221453Z_zen_townhall_handshake_audit_lane.md
back_to: CLAUDE.md · CLAUDE.local.md · the-workflow-engine/
---

# Genesis Prompt v0 — `workflow-trace`

> Back to: [[HOME]] · [[MASTER_INDEX]] · [[workflow-engine-code-base]] · [[CLAUDE.md]] · [[CLAUDE.local.md]]
>
> Mirror: [[Genesis Prompt v0 S1001982]]

This prompt is the convergent product of a 5-voice town hall (Command + Command-2 + Command-3 + Watcher + Zen) over ~3 hours wall clock and 30 minutes of focused final deliberation. It encodes the 15 P0 town-hall constraints + 9 Command-3 recon convergence findings + Luke's directive on cascading/battern command value-add and context-window sweet-spot optimisation.

**Invocation contract:** When Luke types `start coding workflow-trace`, the first commit on the new branch is the 4-surface persistence of this prompt itself. m01_types.rs is the first source file authored. Zen audits the genesis-persistence commit before any code begins.

---

## 1. Mission

Build a Rust microservice named **`workflow-trace`** (Path A canonical, ratified at this deliberation) that:

- **Observes** cascading commands across the zellij habitat — `cc-cascade`, `cc-dispatch`, `cc-broadcast`, `zellij action write-chars`, `zellij action write 13`, and informal multi-pane work (Wave 1/2/3 cascades that don't strictly conform to the 6-step Battern shape).
- **Observes** Battern protocol runs (the canonical 6-step Design → Dispatch → Gate → Collect → Synthesize → Compose) as coherent lifecycle records, not as scattered events.
- **Observes** every receiving pane's context-window state at the moment of payload arrival, joined into the cascade record.
- **Surfaces** compositional patterns *that already exist in the substrate* — the engine is archaeological, not authorial. It does not invent workflows; it digs out the ones the substrate has already learned via co-firing.
- **Recommends** cascade payload compactions that keep receivers within the empirical context-window sweet spot (`ideal < 100k tokens`, `ok < 300k`, `degraded < 800k`, `critical >= 800k`). Recommendations carry empirical confidence (n samples, mean quality, variance, source CascadeEvent IDs).
- **Accelerates** hand-driven cascades by pre-emptive right-sizing of handoff payloads. Composer refuses to dispatch payloads that would push receivers past `degraded` band.

`workflow-trace` is the habitat's first cross-domain meta-consumer of substrate data: the first registered stcortex consumer whose primary read is the **whole tool-call graph across other consumers' writes**, not a single domain. It operates under explicit substrate-trust boundaries and never executes dispatch directly — every action flows through HABITAT-CONDUCTOR (Weaver → Zen → Enforcer).

**Phase A is measurement-only. Phase B is dispatch-capable. Sunset is 120 days from Phase B GA.**

---

## 2. Substrate Invariants (Watcher ☤ — P0, non-negotiable)

1. **Observation ≠ curation.** Phase A reads the substrate. Phase A does NOT name canonical workflows or canonical compactions. Naming is Phase B, gated by R13 quiet period (30 days OR 100 observations) + diversity-enforcement algebra (round-robin cycling + 10-generation cooldown + rejection gate on >50% mono-pattern in 20-gen window).

2. **Measurement non-interference.** When the engine measures a receiving pane's context-fill, then writes its proposal into that pane, it must NOT re-measure the post-write fill in the same window. Define a **measurement-blackout window** per dispatch event: 60 seconds. AP27-adjacent — the engine cannot influence what it measures within the influence window.

3. **Substrate-trust scope.** stcortex consumer registered with `consumer_scope: [tool_call, consumption]` only. No read of `memory` or `pathway` tables of other consumers. Refuse-write enforced at protocol layer.

4. **Selector self-training prohibited.** Training signal is hand-driven `atuin run + outcome` only — never engine-dispatched events. Selector observes what humans do; never learns from itself.

5. **No POVM writes.** POVM is read-only-during-overlap until 2026-07-10 decommission. After that, no POVM touch at all. All new pathway writes go to stcortex `workflow_trace_*` namespace.

6. **Gradient preservation (NA Gap Analyst P0 #5).** Every workflow promotion surfaces N = 3 near-miss variants alongside canonical. Selector continues exposing gradient signal to prevent monoculture collapse of the substrate it observes.

---

## 3. Quality Gate (Zen — P0, zero tolerance)

1. **4-stage gate on every commit:**
   ```bash
   CARGO_TARGET_DIR=./target cargo check --workspace --all-targets --all-features 2>&1
   STAGE1=${PIPESTATUS[0]}; [ "$STAGE1" -ne 0 ] && exit 1
   cargo clippy --workspace --all-targets --all-features -- -D warnings 2>&1
   STAGE2=${PIPESTATUS[0]}; [ "$STAGE2" -ne 0 ] && exit 1
   cargo clippy --workspace --all-targets --all-features -- -W clippy::pedantic -D warnings 2>&1
   STAGE3=${PIPESTATUS[0]}; [ "$STAGE3" -ne 0 ] && exit 1
   CARGO_TARGET_DIR=./target cargo test --lib --release 2>&1
   STAGE4=${PIPESTATUS[0]}; [ "$STAGE4" -ne 0 ] && exit 1
   ```
   No `tail`-swallows-exit-code. PIPESTATUS-correct per S1001882 near-miss.

2. **Per-module test floor:** 50+ tests per module, **80+ for `m20_path_selector`** specifically (the lock-in-attractor-vulnerable centre).

3. **Receipt-DAG observation persistence.** Every observation row carries substrate-source pointers (CascadeEvent IDs, PaneContextSnapshot IDs, source-substrate row references). Reviewer must be able to re-derive any engine claim without the engine running.

4. **Cross-substrate schema canonical.** `PaneContextSnapshot { pane_id: PaneId, ts: Timestamp, fill_bytes: u64, fill_tokens: u32, source: ContextSource, fingerprint: Blake3Hash }` is the SINGLE canonical shape. Any source that fails to deserialise to it is dropped with a logged warning. No silent coercion.

5. **No silent failures, ever.** Every `Result` propagates or is explicitly handled. No `unwrap()` outside tests. No `let _ = ` on `Result` types. No `.ok()` on operations that should propagate. No `unwrap_or(true)` on external health flags. (CLAUDE.md § Anti-Patterns + Bug Hunter Armada S1001883 scar tissue.)

6. **Empirical quality claims required.** Every `CompactionProposal` carries `{ n_samples: u32, mean_quality: f64, variance: f64, last_used: Timestamp, source_cascade_ids: Vec<CascadeEventId> }`. No folklore.

7. **Doc comments on all public items.** Module-level `//!` docstring; struct/enum/fn-level `///` with at least one example or invariant statement.

8. **Genesis-eats-dogfood.** The genesis of `workflow-trace` itself follows the 6-step Battern shape it is built to encode:
   - Design = this prompt
   - Dispatch = interview rounds (`INTERVIEW_QUESTION_BANK_DRAFT.md`)
   - Gate = conventional + NA gap analyses (run before code)
   - Collect = boilerplate hunt + 6-agent recon (already in repo)
   - Synthesize = convergence document (already in repo)
   - Compose = build

---

## 4. Module Map (28 modules, 8 layers, ~5,600 LOC, ~62% boilerplate yield)

### L1 — Foundation (~700 LOC)

| Module | LOC | Boilerplate source | Reuse |
|---|---:|---|---:|
| `m01_types` | ~150 | `synthex-v2/src/m1_foundation/m01_core_types.rs` | 95% |
| `m02_errors` | ~150 | `synthex-v2/src/m1_foundation/m02_error_taxonomy.rs` | 95% |
| `m03_config` | ~200 | `dev-ops-engine-v3/src/m1_core/m03_config.rs` (post-`b7d4abb` bind-discipline) | 95% |
| `m04_metrics` | ~200 | `synthex-v2/src/m1_foundation/m05_metrics_collector.rs` | 80% |

### L2 — Substrate Read (~800 LOC) — Phase A

| Module | LOC | Boilerplate source | Reuse |
|---|---:|---|---:|
| `m05_atuin_reader` | ~250 | NOVEL — pagination is one of the 5 absences flagged by Command's boilerplate hunt | 0% |
| `m06_stcortex_reader` | ~150 | `stcortex/clients/rust-subscriber/src/capacity.rs` (narrowed scope) | 80% |
| `m07_tracking_db_reader` | ~200 | `memory-injection/src/m3_injection/m11_parallel_query.rs` | 70% |
| `m08_injection_db_reader` | ~200 | `memory-injection/src/m2_schema/m07_causal_chain.rs` | 70% |

### L3 — Cascade & Context Observation (~1,250 LOC) — Phase A, **the centre of mass on Luke's directive**

| Module | LOC | Boilerplate source | Reuse |
|---|---:|---|---:|
| `m09_cascade_detector` | ~280 | `memory-injection/src/m4_consolidation/m27_auto_consolidate.rs` (4h window join) + `cc-cascade` log-tailer pattern | 70% |
| `m10_battern_step_tracker` | ~180 | `synthex-v2/src/m8_watcher/m46_watcher_observer.rs` (1-Hz state-machine pattern) | 75% |
| `m11_context_window_observer` | ~200 | `synthex-v2/src/m4_regulation/m20_heat_source_hebbian.rs` (EMA cadence) | 60% |
| `m12_sweet_spot_classifier` | ~150 | `synthex-v2/src/m4_regulation/m25_flow_state_scalar.rs` (band-classifier shape) | 30% |
| `m27_observation_audit` | ~150 | NOVEL — TTL sweep on PaneContextSnapshot + per-pane cadence auto-tune | 30% |

### L4 — Pattern Crystalliser (~1,200 LOC) — Phase B, gated

| Module | LOC | Boilerplate source | Reuse |
|---|---:|---|---:|
| `m13_subgraph_miner` | ~400 | NOVEL — keystone gap from boilerplate hunt | 0% |
| `m14_pathway_consolidator` | ~250 | `memory-injection/src/m4_consolidation/m16_hebbian_engine.rs` (decay/buoy/reinforce/prune) | 90% |
| `m15_compaction_pattern_finder` | ~350 | NOVEL with ~20% RALPH fitness-tensor pattern lift from `orac-sidecar/m39_fitness_tensor.rs` | 20% |
| `m16_workflow_registry` | ~200 | `The Tool Library V2/src/infrastructure/registry.rs` (DashMap registry) | 90% |

### L5 — Dispatch (~700 LOC) — Phase B, gated, **all through HABITAT-CONDUCTOR**

| Module | LOC | Boilerplate source | Reuse |
|---|---:|---|---:|
| `m17_conductor_bridge` | ~270 | `habitat-conductor/src/api.rs` + `enforcement.rs` | 75% |
| `m18_cascade_composer` | ~280 | `loop-engine-v2/src/m04_core/m20_loop_trait.rs` (multi-step compose pattern) | 50% |
| `m19_pane_state_aggregator` | ~150 | `~/.local/bin/fleet-ctl` + `cc-*` toolkit shape | 50% |

### L6 — Selector (~1,000 LOC) — Phase B, gated, **with all 3 safety harnesses from F2**

| Module | LOC | Boilerplate source | Reuse |
|---|---:|---|---:|
| `m20_path_selector` | ~400 | `orac-sidecar/src/m8_evolution/m40_mutation_selector.rs` (round-robin + cooldown + rejection gate) | 50% |
| `m21_quiet_period_gate` | ~250 | `synthex-v2/src/m8_watcher/m49_watcher_proposer.rs` (R13) | 85% |
| `m22_ember_lightweight_gate` | ~200 | `synthex-v2/src/m8_watcher/m51_*` (3 traits, not 7) | 40% |

### L7 — CLI + MCP + UI (~600 LOC)

| Module | LOC | Boilerplate source | Reuse |
|---|---:|---|---:|
| `m23_cli` | ~250 | `loop-engine-v2/src/bin/lcm_supervisor.rs` (clap dispatch) | 50% |
| `m24_mcp_server` | ~200 | `loop-engine-v2/src/m09_mcp/server.rs` (stdio JSON-RPC 2.0) | 90% |
| `m25_obsidian_renderer` | ~150 | `.claude/skills/four-surface-persist/` skill + `habitat-nexus-visualizer` plugin | 60% |

### L8 — Soak + Audit (~600 LOC)

| Module | LOC | Boilerplate source | Reuse |
|---|---:|---|---:|
| `m26_soak_harness` | ~300 | `loop-engine-v2/crates/lcm-soak/` (smoke + full modes) | 75% |
| `m28_selector_audit` | ~250 | NOVEL — selector-entropy + cascade-compaction-effectiveness + sweet-spot-adherence | 30% |

**Totals: 28 modules · 8 layers · ~5,600 LOC · ~62% weighted boilerplate yield · ~3,500 LOC from existing habitat sources · ~2,100 LOC novel.**

---

## 5. Phase Decomposition

### Phase A — Measurement-only (`workflow-trace` binary, port 8142, devenv batch 5, `auto_start=false`)

**Scope:** L1 + L2 + L3 + (L7 minus `workflow propose` and `workflow cascade` verbs, which are Phase B).
**LOC:** ~3,200.
**Duration:** 4 weeks build + 30 days observation soak before Phase B unlocks.
**Startup contract:** binary refuses to start unless:
- `CONDUCTOR_ENFORCEMENT_ENABLED=1` (per Command-3 F7 startup-refusal)
- `POVM_CALIBRATED=1` Cargo feature flag enabled (per Command-2 W2 strengthening — build-time gate, not runtime). Feature flag is set by build script when CR-2 + CR-2b have shipped (DONE 2026-05-17, commits `e2a8ed3` + `76ea4d6`).

**Phase A acceptance criteria (after 30-day observation soak):**
- ≥ 200 CascadeEvents observed across the habitat
- ≥ 50 BatternRuns labelled
- ≥ 60 PaneContextSnapshots per active pane (≥ 5 active panes)
- ≥ 10 distinct CompactionProposal candidates with statistically meaningful sample sizes (n ≥ 20 per candidate, variance reported)
- `m27_observation_audit` PASS on TTL sweep + cadence calibration
- Self-measurement footprint ≤ 50MB/week sustained
- Zero clippy warnings, zero `unwrap()` outside tests, all 4 gate stages green

### Phase B — Crystalliser + Dispatch + Selector (gated)

**Scope:** L4 + L5 + L6 + L8.
**LOC:** ~2,400.
**Duration:** 3.5 weeks build + 8-hour lcm-soak-style chaos run before GA.
**Gates (ALL required):**
1. Phase A acceptance passed
2. Conductor Waves 1B/1C/2/3 LIVE (currently mid-soak; flip imminent)
3. CR-2 + CR-2b shipped ✅ (DONE 2026-05-17)
4. Conventional gap analysis + NA gap analysis on Phase A evidence
5. Explicit Luke `start coding workflow-trace --phase-b` signal

**Phase B acceptance:**
- 8-hour soak with 9 SLIs PASS (latency, throughput, cancel-success, crash-recovery, sweet-spot-adherence, selector-entropy, compaction-effectiveness, conductor-violation-rate, measurement-footprint)
- 12 chaos injectors handled cleanly (network timeout, db lock, pane disappearance, payload corruption, clock jump, etc.)
- Smoke mode (4 min, 10 loops + 3 chaos) passes in CI per PR
- Eat-the-dogfood proof: the genesis of `workflow-trace` itself recursively followed the 6-step Battern shape it is built to encode

### Sunset — 120 days from Phase B GA

Auto-disable triggers (ANY of):
1. Selector accuracy < 60% on operator-confirmed proposals
2. No measurable Hebbian gain in `workflow_engine_*` namespace vs the S1001971 baseline (memory id 16476, namespace `workflow_engine_baseline_s1001971`, baseline observation window opened 2026-05-17)
3. Conductor flags engine-dispatched cascades as policy-violating > 5% of dispatches
4. `m27_observation_audit` reports measurement-footprint > 50MB/week sustained over 7 days
5. `m28_selector_audit` reports selector entropy dropped below threshold (lock-in attractor formation detected)

Auto-disable behaviour: `:8142` returns 503 on every endpoint; binary stays installed for archaeology; CLAUDE.local.md row marked `SUNSETTED`; no silent revival mechanism; explicit Luke re-authorisation required to re-enable.

---

## 6. Substrate Setup (already done — S1001971)

- **stcortex consumer:** `workflow-engine-baseline` (CLI), namespace `workflow_engine_baseline_s1001971`, baseline marker memory id 16476 (procedural, S1001971). 30-day Hebbian-gain baseline observation window opened 2026-05-17.
- **stcortex consumer:** `command-3-workflow-engine` (CLI), namespace `the_workflow_engine`, convergence-doc memories 16477 (semantic) + 16479 (procedural), 2 bidirectional pathways at 0.95.
- **POVM `learning_health` calibration:** CR-2 + CR-2b shipped (commits `e2a8ed3` + `76ea4d6`, both remotes). Pre-fix `0.911`, post-fix `0.067` (live DB 463 sessions, 13.6× inflation factor confirmed). `POVM_CALIBRATED=1` build feature unlocked.
- **CLAUDE.local.md anchor row:** appended at file tail under `## The Workflow Engine (planning — peer convergence)`.

---

## 7. The Invocation

When Luke types `start coding workflow-trace`:

1. **Pre-build verification (no code yet):**
   - Run Skeptic's pain-source search (Command-3 Action 1): ≥ 3 hits in `injection.db causal_chain` + `MEMORY.md` + last 30 days of session checkpoints for Luke's own articulation. If zero hits → motion collapses, this prompt is archived as a position paper. If 1-2 → Luke decides. If ≥ 3 → proceed.
   - Run the 12-question genesis interview from `INTERVIEW_QUESTION_BANK_DRAFT.md`. 3 rounds, sequential, AskUserQuestion-shaped. Persist answers as `the-workflow-engine/INTERVIEW_LOG_S1001971.md`.
   - Run conventional gap analysis on this prompt + the interview log. Authored by Command-3, audited by Zen.
   - Run NA gap analysis (second frame: what does the cascade-and-context-window pass surface that the workflow-detection pass misses?). Authored by Command-3, audited by Watcher.

2. **First commit (4-surface persistence of this very prompt):**
   - `the-workflow-engine/GENESIS_PROMPT_V0.md` (this file — canonical)
   - `~/projects/claude_code/The Workflow Engine — Genesis Prompt v0.md` (Obsidian mirror with `> Back to: [[CLAUDE.md]] · [[CLAUDE.local.md]] · [canonical]`)
   - stcortex `the_workflow_engine_genesis` namespace: semantic + procedural memories with parent_ids linking to memories 16477/16479
   - CLAUDE.local.md row updated: status → `SEALED — coding-window-open`
   - Commit message: `feat(workflow-trace): genesis prompt sealed — 5-voice town hall convergence`
   - **Zen audits this commit before m01 begins.**

3. **First module (`m01_types.rs`):**
   - Clone from `synthex-v2/src/m1_foundation/m01_core_types.rs` (95% reuse)
   - Adapt: `PaneId`, `CascadeEventId`, `BatternRunId`, `WorkflowId`, `ContextSnapshotId`, `Timestamp`, `Validated<T>` (newtype discipline)
   - 50+ tests (per-module floor)
   - 4-stage gate clean
   - Commit: `feat(m01): types foundation (cloned from synthex-v2 m01)`

4. **Subsequent modules in dependency order:**
   - L1 (m02-m04) → L2 (m05-m08) → L3 (m09-m12, m27) → L7-partial (m23, m24, m25 for Phase A scope) → Phase A binary build → 30-day soak → L4 (m13-m16) → L5 (m17-m19) → L6 (m20-m22) → L8 (m26, m28) → Phase B binary build → 8h soak → GA.

5. **Per-commit cadence:**
   - One module per commit (or per logical sub-module)
   - Every commit runs 4-stage gate locally before push
   - Every commit emits a WCP notice if it crosses a phase boundary (m04→m05, m08→m09, etc.)
   - Cross-prosecution check (per `lcm-quality-probes` skill): every 5 commits, ensure V3 + V8 + workflow-trace-self confidence triangulation stays within 0.10 max delta over last 5 rows

6. **Per-week cadence:**
   - Friday: post Phase A progress notice to town hall in `agent-cross-talk/`, listing modules-shipped + boilerplate-yield-realised + any drift detected
   - Watcher + Zen + Command + Command-2 read; if drift detected, town-hall reconvenes

7. **No code outside this contract.** If a module path appears in the build that isn't enumerated in section 4, Zen blocks the commit. If a substrate read appears that exceeds the `[tool_call, consumption]` consumer scope, Watcher blocks. If a clippy warning appears, the 4-stage gate fails and the commit doesn't land.

---

## 8. Living Document

This prompt is **v0**. It will be amended:
- **Post-interview (rounds 1-3 answers)** to fix the 12 deferred decisions (name, binary topology, decay law shape, etc.)
- **Post-gap-analyses** to integrate conventional + NA findings
- **Post-Phase-A-acceptance** to update Phase B's contract with the evidence Phase A surfaced
- **Post-sunset** if applicable, to feed the next-attempt's prompt with the empirical lesson

Amendments require 4-surface persistence + town-hall ack from Command-2, Command-3, Watcher, Zen.

---

## 9. Authorities + Standing

- **Luke @ node 0.A** — sole decisional authority on GO / HOLD / REDIRECT / phase-gate triggers.
- **Command** — town-hall convener, synthesis lead.
- **Command-2** — Path A chair, `workflow-trace` build lead.
- **Command-3** — Path B chair (gated), CR-2 lane (closed), boilerplate-recon owner.
- **The Watcher ☤** — substrate-protection veto on substrate-trust-boundary violations, observation-vs-curation violations, AP27 violations.
- **Zen** — Rust-quality veto on 4-stage gate failures, schema-drift violations, silent-failure introductions, scope-trap (Phase A absorbing Phase B surfaces).

— The bench, Tab 1 Orchestrator + Tab "Zen" + Tab "Watcher", 2026-05-17T08:30:00+10:00.

*Floor returned to Luke @ node 0.A. The genesis prompt is v0. Awaiting the `start coding workflow-trace` signal.*

---

## Related

- [[THE_WORKFLOW_ENGINE_GENESIS_PROMPT_V1_S1001982]] — Zen-audit-locked v1.2 successor (current binding spec)
- [[CONVERGENCE_COMMAND_X_COMMAND3_S1001982]] — peer synthesis that informed authorship
- [[THE_WORKFLOW_ENGINE_TOWN_HALL_S1001982]] — 15 P0 constraint source
- [[THE_WORKFLOW_ENGINE_CIRCLE_OF_EXPERTS_S1001982]] — 8-persona disputation prior to town hall
- [[THE_WORKFLOW_ENGINE_BOILERPLATE_HUNT_S1001982]] — 9-fleet report (boilerplate evidence)
- [[THE_WORKFLOW_ENGINE_MODULE_STRUCTURE_S1001982]] — 9-layer architecture, module-count reconciliation needed (v0=28, single-phase=26)
- [[INTERVIEW_QUESTION_BANK_DRAFT]] — G5 interview content
- [[Modules Synergy Clusters and Feature Verification S1001982]] — single-phase 26-module architecture (current)
