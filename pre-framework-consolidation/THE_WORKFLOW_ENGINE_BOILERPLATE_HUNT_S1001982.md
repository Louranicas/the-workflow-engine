---
title: The Workflow Engine — Boilerplate Hunt Report (Wave 1 Fleet)
date: 2026-05-17 (S1001982)
authority: Luke @ node 0.A
emitter: Command (Tab 1 Orchestrator)
kind: TOWN-HALL-REPORT (battern step 6: Compose)
status: planning-only · no code · no scaffolding · no implementation
priors:
  - THE_WORKFLOW_ENGINE_CIRCLE_OF_EXPERTS_S1001982.md (circle disputation)
  - THE_WORKFLOW_ENGINE_TOWN_HALL_S1001982.md (12-persona finals, 15 P0 constraints)
fleet:
  - 9 Explore agents, dispatched in parallel (single message, single tool block)
  - Each scoped to one boilerplate category × ~5 crates + Obsidian triangulation
  - All 9 returned. Total candidates surfaced: 63.
back_to: CLAUDE.md · CLAUDE.local.md · the-workflow-engine/
---

# Boilerplate Hunt Report — Wave 1 Fleet Findings

> Back to: [[HOME]] · [[MASTER_INDEX]] · [[workflow-engine-code-base]] · [[CLAUDE.md]] · [[CLAUDE.local.md]]
>
> Mirror: [[Boilerplate Hunt S1001982]]

The town hall closed with 15 P0 constraints. The fleet was dispatched to hunt existing habitat modules that could be lifted as boilerplate for each constraint family. Nine Explore agents, parallel, scoped, with Obsidian master-index triangulation. All returned.

This report consolidates 63 raw candidates into 9 top-picks, 4 cross-cutting patterns, 3 structural gaps that *cannot* be boilerplated, 1 maturity-ceiling blocker, and a risk register.

**Planning-only.** No code is proposed. No scaffolding is recommended. This document feeds back into the town-hall record awaiting Luke's GO / HOLD / REDIRECT call.

---

## Per-Category Top-Picks (1 line each)

| # | Category | Top-Pick | Strength | One-line risk |
|---|---|---|---|---|
| 1 | **CLI binary scaffolding** | `habitat-conductor/src/bin/{weaver,enforcer,zen}.rs:45-100` + `loop-engine-v2/src/m08_cli/m35_cli_root` | STRONG | No shared CLI library across binaries — would need to author `workflow_core::cli` early. |
| 2 | **stcortex consumer (narrowed scope)** | `stcortex/clients/rust-subscriber/src/capacity.rs:213-297` + `docs/CONSUMER-ONBOARDING.md` refuse-write enforcement | STRONG | No service today does table-scoped (vs namespace-scoped) registration — engine may be the first. |
| 3 | **SQLite read-heavy multi-DB** | `memory-injection/src/m2_schema/m06_schema.rs:22-150` + `m3_injection/m11_parallel_query.rs:35-283` | STRONG | All candidates synchronous; no pagination helper for 263k+ atuin rows. |
| 4 | **Compositional pattern detection** | `memory-injection/src/m2_schema/m10_pattern.rs:185-240` (two-counter buoy + decay) + `orac-sidecar/src/m6_coordination/m49_task_graph.rs:200-300` (Kahn's cycle detection) | MEDIUM | **KEYSTONE GAP** — see §Structural Gaps. |
| 5 | **Decay / TTL / LTD** | `povm-v2/src/l3_consolidation/lifecycle.rs:84-150` + `orac-sidecar/src/m8_evolution/m39_fitness_tensor.rs:40-65` (HYBRID) | STRONG (combined) | **No habitat module does `frequency × fitness × recency` today** — this is a new primitive. |
| 6 | **Daemon + background-task scaffolding** | `synthex-v2/src/daemon/runtime.rs` (557 LOC) + `synthex-v2/src/daemon/shutdown.rs` (222 LOC) + `habitat-nerve-center/src/main.rs` (shared-state + ring buffer) | STRONG | ~2,100 LOC of proven scaffold; needs heartbeat-file (not HTTP /ready) for CLI-not-service shape. |
| 7 | **HABITAT-CONDUCTOR dispatch integration** | `habitat-conductor/src/state.rs:1-250` + `src/enforcement.rs:1-200` | STRONG (pattern) / **BLOCKED (live ceiling)** | **MATURITY CEILING** — see §Conductor Blocker. |
| 8 | **NexusEvent + LCM RPC client** | `loop-engine-v2/src/bin/lcm_supervisor.rs:73-292` (RPC envelope + dispatcher) + `orac-sidecar/src/m5_bridges/m22_synthex_async.rs` (async + circuit-breaker) + dual-transport emitter from habitat-bench-spine | STRONG | NexusEvent.data is untyped JSON — Option-A (additive) for MVP, Option-B (typed enum) for hardening. |
| 9 | **Trap annotation + verify + escape-surface** | `.claude/skills/forge/SKILL.md:225-237` (8 traps) + `.claude/skills/pre-deploy-hardening/SKILL.md:50-125` (4-agent gate) + `.claude/hookify.preserve-blanket-guard.local.md` (S102 scar) | STRONG (per-piece) / **GAP (unified schema)** | No unified destructiveness classification exists anywhere — Cipher's constraint #11 IS the missing primitive. |

---

## Cross-Cutting Patterns

Four boilerplate modules show up across MULTIPLE categories. These are the highest-leverage lifts because one source crate satisfies several P0 constraints simultaneously.

### Cross-cutter 1: `synthex-v2/src/daemon/` (categories 6 + 8 + indirect 3, 4)
- Appears as top-pick for daemon scaffolding (Cat 6) and provides the WebSocket inbound dispatch (Cat 8 — `m11_eventbus_subscriber`, S117 Phase 2.5).
- Indirectly: TTL sweep pattern (Cat 5 — `daemon/tasks/ws_inbound_writer.rs:96-140`) and read-heavy SQLite patterns (Cat 3).
- **Implication:** synthex-v2 is the densest single source of boilerplate. ~3,000 LOC potentially liftable across 4 categories.
- **Watch:** synthex-v2 is itself shadow-mode (Phase G external gate). Lifting from it is safe; *coupling* to it requires its production status.

### Cross-cutter 2: `loop-engine-v2/src/bin/lcm_supervisor.rs` + `m08_cli/m35_cli_root` (categories 1 + 6 + 8)
- 18-subcommand CLI dispatch (Cat 1), supervisor singleton pattern (Cat 6), 9-RPC JSON-RPC envelope (Cat 8).
- **Implication:** LCM's CLI surface is the most-evolved in the habitat. The 11-layer architecture is deep but the L08 CLI module is self-contained.
- **Watch:** LCM is post-M0-verified, pre-W4. Lifting CLI patterns is safe; *delegating* dispatch to LCM requires Drift #9 cleared (deploy_cancel_handler `a60aae6` shipped — see CLAUDE.local.md).

### Cross-cutter 3: `habitat-conductor/src/bin/{weaver,zen,enforcer}` (categories 1 + 7)
- Three production-grade clap/tracing/EnvFilter binaries (Cat 1) AND the canonical Conductor wire-protocol authority (Cat 7).
- **Implication:** If the engine ships as a Conductor client, lifting the binary scaffolding from Conductor itself keeps wire-protocol contracts aligned at the source.
- **Watch:** Wave 1B (Weaver daemon) + Wave 1C (Zen daemon) + Wave 2 (WASM) + Wave 3 (Enforcer) are **built + installed + registered but NOT yet live** (Batch 5 `auto_start=false` pending Luke's terminal bring-up per CLAUDE.local.md). See §Conductor Blocker.

### Cross-cutter 4: `memory-injection/src/m2_schema/m10_pattern.rs` (categories 4 + 5)
- Two-counter discrimination (`hit_count` vs `natural_hit_count`) + buoy lifecycle (Cat 4) AND exponential decay (`weight *= 0.98`) with selective immunity (Cat 5).
- **Implication:** The closest existing analog to a fitness-aware decay law. 100-session property tests already validate three-tier equilibrium (Active ~0.69 / Buoyed ~0.50 / Floor ~0.30).
- **Watch:** Designed for *patterns* (single-step keyword matches), not *workflows* (N-step compositional sub-graphs). Schema adapts but semantics do not transfer wholesale.

---

## Three Structural Gaps (cannot be boilerplated — must be authored)

The fleet was thorough. Three constraints have **no existing habitat analog** strong enough to lift. These define the engine's *new* surface area.

### Gap 1 — N-step compositional sub-graph detection (the engine's keystone)

**Constraint family:** Pillar 1 of the crystalliser. The Architect, the Fossil ("archaeological"), and the Substrate all named this in the town hall.

**What the fleet found:**
- POVM does **pairwise** co-activation (`l5_feedback/reinforcement.rs:31-37` — `CoActivationPair { a, b, ts_ms }`).
- memory-injection m10 tracks **linear** patterns with keyword matching.
- ORAC m49 does **cycle detection** (Kahn's), not pattern mining.
- POVM L4 `spread.rs` does **undirected activation spreading**, not directed sub-graph isomorphism.

**What's missing:**
- N-gram extraction across sessions (find repeated K-step sequences ≥k recurrences).
- Temporal clustering **with gaps** ("tool A, then ≥2 unrelated steps, then tool B").
- Sub-graph isomorphism / DAG-to-DAG comparison with edge-weight tolerance.
- Bottom-up confidence aggregation across sub-steps.

**Verdict:** This is the engine's *creative work*. No boilerplate. The pairwise → N-step generalisation is the gating engineering problem.

### Gap 2 — `frequency × fitness × recency` decay law (RALPH constraint)

**Constraint family:** P0 #8 (RALPH's fitness-weighted decay).

**What the fleet found:**
- POVM: frequency + recency. Fitness decouples at session boundary.
- memory-injection: frequency + buoy immunity. No fitness signal at all.
- SYNTHEX v2 `ws_inbound_writer.rs`: TTL sweep only — wall cutoff, no weighting.
- ORAC m39 fitness tensor: 12D fitness exists, but applied to RALPH selection, not to TTL/decay.

**What's missing:**
- A compound decay formula that blends all three signals.
- Per-workflow-edge confidence aggregation feeding decay.
- A volatile-dimension smoothing pattern (ORAC m39 has the *infrastructure*, not the *application*).

**Verdict:** New primitive. ORAC m39's rolling smoothing + POVM's lifecycle structure are the *raw materials*; the composition is unbuilt.

### Gap 3 — Unified destructiveness / escape-surface classification (Cipher constraint)

**Constraint family:** P0 #11 (Cipher's escape-surface declaration + display-before-run).

**What the fleet found:**
- Trap annotations are scattered: `/forge` (8 traps), `/genesis` (15+), CLAUDE.md, skill bodies, feedback memories.
- Escape-surface categories were *proposed* in the town hall, not implemented anywhere.
- Preserve-list discipline lives as scar tissue (S102 openclaw-prune) — feedback memory + hookify guard — not as a schema.
- Silent-failure P1-P5 taxonomy exists in `bridge-silent-failure-hunt` but is code-auditor-shaped, not workflow-shaped.

**What's missing:**
- A `WorkflowSecurityProfile` schema unifying escape-surface + trap-annotation + preserve-list.
- A promotion-gate that checks declared profile against substrate evidence ("does this `destructive` action recur enough to justify a workflow?").
- A dispatcher display-before-run protocol.

**Verdict:** New primitive. The constituent classifiers exist in five places; the unification is unbuilt.

---

## The Conductor Blocker (maturity ceiling)

P0 #3 (Topologist) mandates: **engine never executes directly; dispatch flows through HABITAT-CONDUCTOR**. The fleet's Cat-7 agent surfaced the real ceiling:

- HABITAT-CONDUCTOR Waves 0 / -1 / 0.5 / 0.75 / 1A are LIVE.
- Waves 1B (Weaver daemon HTTP `/state`) / 1C (Zen daemon `/divergence`) / 2 (WASM) / 3 (Enforcer) are **built + installed + registered but NOT LIVE** — `auto_start=false` in Batch 5 pending Luke's terminal bring-up (per CLAUDE.local.md Active Workstreams row).
- The engine's dispatch path needs Wave 1B at minimum (Weaver `/state`) and ideally Wave 3 (Enforcement gates).

**Implication:** Even with all boilerplate identified, the engine cannot satisfy P0 #3 until Conductor's pending waves are brought up. Two options:

**Option α: Wait for Conductor bring-up.** Aligns with town hall constraint #15 ("No execution until Conductor Wave 1B/1C/2/3 stabilise"). Engine planning continues; build sequence waits.

**Option β: Engine ships against the 80% substrate (atuin KV + habitat-plugin + ORAC hooks).** Cat-7 agent flagged this is *possible* but it sidesteps Conductor entirely — which violates P0 #3 in spirit. Not recommended.

**Recommendation:** Option α. Hold engine build until Conductor terminal-bring-up clears. Planning can proceed without it; the genesis interview and design-doc-with-dual-frame-gap-analysis are productive work that doesn't need live Conductor.

---

## Estimated Boilerplate Volume (if Luke says GO)

If all 9 top-picks were lifted and adapted:

| Category | Lift LOC | Adapt LOC | Source |
|---|---|---|---|
| Daemon scaffold | ~779 | ~200 | synthex-v2 runtime+shutdown |
| Shared-state + ring buffer | ~400 | ~100 | habitat-nerve-center |
| CLI binary + dispatch | ~300 | ~150 | habitat-conductor + LCM m35 |
| stcortex consumer (narrowed) | ~80 | ~50 | stcortex capacity.rs |
| SQLite multi-DB read | ~300 | ~100 | memory-injection m06+m11 |
| Decay/lifecycle scaffold | ~250 | ~150 | povm-v2 lifecycle + ORAC m39 |
| LCM RPC + async client | ~400 | ~200 | LCM supervisor + ORAC async |
| NexusEvent emitter (dual-transport) | ~150 | ~50 | habitat-bench-spine |
| Trap-annotation schema | ~50 | ~100 | /forge + /pre-deploy-hardening |
| **TOTAL BOILERPLATE** | **~2,700** | **~1,100** | |
| **GAP 1 — sub-graph detection** (new) | — | **~600-1,000** | from scratch |
| **GAP 2 — fitness-weighted decay** (new) | — | **~200-300** | from scratch |
| **GAP 3 — escape-surface schema** (new) | — | **~150-250** | from scratch |
| **TOTAL ENGINE** | ~2,700 lift | ~2,000-2,600 author | ~4,700-5,300 LOC |

This sits inside the Architect's town-hall estimate (1,500-2,500 Rust). The discrepancy is the three structural gaps — the Architect assumed boilerplate would carry more of the load. The fleet revealed the three gaps that must be authored from scratch.

**Updated LOC estimate: ~4,700-5,300 Rust total** (still well below "another Airflow" — Synthesis A is still the right shape).

---

## Risk Register (top 5)

| # | Risk | Severity | Surfaced by | Mitigation |
|---|---|---|---|---|
| 1 | Conductor Waves 1B/1C/2/3 not live — engine cannot dispatch correctly until Luke's terminal bring-up | **HIGH** | Cat-7 agent | Hold build; planning continues; align engine MVP gate with Conductor `auto_start=true` flip |
| 2 | NexusEvent extensibility: untyped `data: serde_json::Value` field can silently drift between emitter and subscriber | MEDIUM | Cat-8 agent | Option A (additive untyped) for MVP; S119-pattern round-trip tests for hardening |
| 3 | POVM `learning_health` figures (0.892 / 0.896) **inflated until CR-2 lands** (per Bug Hunter Armada S1001883) — any decay law calibrated against current POVM numbers will mis-tune | MEDIUM | Cross-ref CLAUDE.local.md Open Escalations + Cat-5 agent | Wait for Command-3's CR-2 fix at `povm-v2/.../reinforcement.rs:151` before calibrating decay law |
| 4 | AP30 namespace collision: V3 already owns `P01..P16` Hebbian pathways — engine must use `workflow_engine_*` prefix or it corrupts V3 fitness | MEDIUM | Cat-8 agent | Validate namespace in POVM seed script before any reinforce call; declare prefix in design doc |
| 5 | synthex-v2 itself is shadow-mode (Phase G external gate) — lifting scaffold is safe, but if engine couples tightly to synthex-v2 internals, it inherits the Phase G dependency | MEDIUM | Cross-ref CLAUDE.local.md Phase G row + Cat-6 agent | Lift, don't couple. Use synthex-v2 patterns as templates; do not import synthex-v2 modules directly |

---

## Five Absences Worth Naming

The fleet was asked to flag patterns it expected but didn't find. Consolidated:

1. **Pagination helpers** — no module paginates 263k+ rows. Engine must author or risk OOM on first atuin scan.
2. **Cross-database transactions** — no module does 2PC across multiple SQLite DBs. Engine writes are stcortex-only so this is fine.
3. **Generational pattern frequency tracking** — RALPH does selection but nothing tracks "did this pattern's frequency grow/shrink after gen 500?". The engine's decay law needs this; it must author the counter.
4. **Online vs offline mining** — all existing patterns are online (real-time reinforcement). No batch offline mining of *all* historical sessions. The engine's first crystalliser pass *is* an offline mine; this is unprecedented at habitat scale.
5. **Multi-binary coordination via stcortex** — two engine binaries (`wf-crystallise` + `wf-dispatch`) sharing state via stcortex has no precedent. Either file-based checkpoint (simpler) or stcortex-pathway (POVM-cure compliant). Decision deferred to genesis interview.

---

## What This Report Does NOT Change

- The 15 P0 constraints from the town hall remain in force.
- Synthesis A remains the recommended shape.
- The Skeptic's pain-source verification gate is still active (unchanged by boilerplate-availability).
- The Ember-gate / AP27 boundary remain hard constraints.
- The pre-build sequence (interview → design doc → dual-frame gap analysis → 4-surface persistence → MVP cuts) is unchanged.

The fleet hunt was a *feasibility study*, not a re-design. Boilerplate availability is informational — it does not move the decision from Luke's hands to the circle's.

---

## Decision Updated for Luke

The town-hall verdict stands. The boilerplate hunt adds three modifiers:

1. **Reduce ambiguity on Conductor dependency:** the engine MUST wait for Conductor Waves 1B/1C/2/3 bring-up. If Luke wants engine progress *before* that bring-up, only planning work is available (genesis interview + design doc + gap analyses).
2. **Acknowledge three structural gaps:** the engine is ~50% boilerplate, ~50% new authorship. The new code is the keystone (sub-graph detection), the decay law, and the security schema — none trivial.
3. **POVM-cure crystalliser-write must wait for CR-2** lest the engine calibrate against inflated metrics.

**Standing options remain:**
- **GO** — open genesis interview now (planning-only is safe; build still waits for Conductor + CR-2).
- **HOLD** — file boilerplate report as a position paper; revisit after Armada hygiene + LCM W4-pre + Conductor Wave 2 bring-up.
- **REDIRECT** — tell me which constraint to drop, which gap to defer, or which shape to substitute.

Awaiting your call.

— Command, Tab 1 Orchestrator, 2026-05-17 · The fleet has reported. The town hall is in session. Luke @ node 0.A presides.

---

## Related

- [[THE_WORKFLOW_ENGINE_TOWN_HALL_S1001982]] — 15 P0 constraints that scoped this hunt
- [[THE_WORKFLOW_ENGINE_CIRCLE_OF_EXPERTS_S1001982]] — circle disputation upstream
- [[CONVERGENCE_COMMAND_X_COMMAND3_S1001982]] — extended by Command-3 6-fleet recon (~62% reuse)
- [[THE_WORKFLOW_ENGINE_MODULE_STRUCTURE_S1001982]] — consumed findings into architecture
- [[GENESIS_PROMPT_V0]] — 5-voice prompt that absorbed boilerplate evidence
- [[THE_WORKFLOW_ENGINE_GENESIS_PROMPT_V1_S1001982]] — Zen-locked v1.2 successor
- [[Modules Synergy Clusters and Feature Verification S1001982]] — single-phase architecture (current)
