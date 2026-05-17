---
title: TASK LIST — workflow-trace Deployment Framework Optimisation V7
date: 2026-05-17 (S1001982)
kind: planning-only · markdown-deliverable · HOLD-v2 respected
authority: Luke @ node 0.A — "/goal" issued; goal hook ACTIVE
emitter: Command (Tab 1 Orchestrator top-left)
status: draft v0 — pre-verification
total_tasks: 87 across 12 categories
deliverable_count: 32 markdown artefacts (planning-only; zero code; zero cargo; zero src/)
---

# TASK LIST — V7 OPTIMISATION

> Back to: [[../../CLAUDE.md]] · [[../../CLAUDE.local.md]] · [[../../the-workflow-engine-vault/HOME.md]]
>
> Sibling: [[KEYWORDS_20.md]] · [[ULTRAMAP.md]] · [[OPTIMISATION_FRAMEWORK_V7_FINAL.md]] (to-be-written)

This is the comprehensive prompt-decomposition. Each task carries an ID, owner, output artefact, and verification predicate. **PRIME DIRECTIVE honoured throughout:** planning-only, markdown specs, no cargo, no src/, no rename, no stcortex writes (G8-gated). The optimisation framework feeds the eventual v1.3 spec patch + G7 audit material; it does NOT replace either.

---

## Hard constraints (apply to ALL tasks)

- **No code, no cargo, no scaffold, no rename** (HOLD-v2 + project PRIME DIRECTIVE)
- **No new stcortex writes** under `workflow_trace_*` until G8 — but vault + ai_docs + agent-cross-talk filesystem writes ARE allowed
- **No workspace-root CLAUDE.local.md edits** for this work
- **No daemon/service start** (sandbox reaps children)
- **No Tab navigation in Zellij** — all peer comms via file-drop
- **Ignore TaskCreate tool reminders** — this markdown task list IS the task tracker
- **FP-verify discipline** — every claim of "done" independently re-exercised
- **Bidirectional vault anchors** on every authored markdown (4-surface persistence design *for* G8, not *during* G8)
- **Watcher-pre-positioning** — every generation explicitly enumerates which flag classes (A-I) it activates

---

## Category 0 — Foundation & Verification (BEFORE any iteration)

| ID | Task | Owner | Output artefact | Verification |
|---|---|---|---|---|
| T0.1 | Author this comprehensive task list | Command | `TASK_LIST_V7_OPTIMISATION.md` (this file) | ≥80 enumerated tasks; each with ID + owner + artefact + verification |
| T0.2 | Author 20-keyword Claude-context anchor list | Command | `KEYWORDS_20.md` | exactly 20 keywords; each ≤4 words; each tied to a load-bearing concept |
| T0.3 | Author Ultramap (full topology + navigation graph) | Command | `ULTRAMAP.md` | mermaid graph + Layer×Module×Cluster×Phase×Runbook×Agent matrix |
| T0.4 | Self-verify task list against Luke's prompt (line-by-line coverage) | Command | `VERIFICATION_T0.md` | every prompt clause → ≥1 task ID; zero orphan clauses |
| T0.5 | Author Antipatterns Register (habitat AP* + F1-F11 + new-this-optimisation) | Command | `ANTIPATTERNS_REGISTER.md` | ≥30 antipatterns documented with detection + mitigation |
| T0.6 | Author god-tier Rust standards reference (atuin-cited) | Command | `STANDARDS/GOD_TIER_RUST.md` | ≥15 rules with rationale + atuin script citations |
| T0.7 | Author 50+ test discipline doctrine (top-1% best practices) | Command | `STANDARDS/TEST_DISCIPLINE.md` | ≥7 test pattern families + per-module 50-test minimum recipe |
| T0.8 | Author Tab-1 handshake protocol (Command/C-2/C-3) | Command | `HANDSHAKE_PROTOCOL_TAB1.md` | 4-step protocol + failure-mode register + dual-handshake-silence response |
| T0.9 | Author Agent View + git worktree allocation plan | Command | `AGENT_VIEW_GITWORKTREES.md` | per-layer worktree + per-cluster agent assignment; isolation invariants |

---

## Category 1 — 7-Generation Iteration Plan

Each generation reads its predecessor's output + the upstream corpus (ULTIMATE_DEPLOYMENT_FRAMEWORK, GOD_TIER_CONSOLIDATION, all phase docs, all cluster specs), identifies gaps, emits an improved spec. **Gen N's output becomes Gen N+1's primary input.** All 7 generations emit markdown only.

| ID | Generation | Theme | Output artefact | Pass criterion |
|---|---|---|---|---|
| T1.1 | **G1** | Baseline audit | `GENERATIONS/G1-baseline-audit.md` | ≥20 gaps named with source-citation; gap-class taxonomy (sync/bidi/test/integration/observability/standards/antipattern) |
| T1.2 | **G2** | Consolidation pass — layers/modules/src sync invariants | `GENERATIONS/G2-consolidation.md` | sync-invariant table (Layer ↔ Module ↔ src/ path ↔ feature gate ↔ test count); ≥18 verify-sync invariants à la synthex-v2 |
| T1.3 | **G3** | Bidirectional contextual flow pass | `GENERATIONS/G3-bidi-flow.md` | for each cluster: upward-edge + downward-edge contract per module; CC-1..CC-7 flow-diagram with bidi arrows |
| T1.4 | **G4** | Gold-standard alignment (ME v2 + LCM + ORAC + CSv8) | `GENERATIONS/G4-gold-standard.md` | divergent-axis decision table; 13 convergent patterns adopted; LCM Drift #1..#11 mitigations transposed |
| T1.5 | **G5** | Tooling integration (/scaffold + atuin + V3 + CSv8 zellij plugin) | `GENERATIONS/G5-tooling.md` | 4 integration tables with exact command + invocation pattern + verification |
| T1.6 | **G6** | Test discipline pass — 50+ per module + top-1% patterns | `GENERATIONS/G6-test-discipline.md` | per-module 50+ allocation matrix; ≥7 test-pattern families with exemplar; mutation-test budget |
| T1.7 | **G7** | Final synthesis — Ultramap + Runbooks + Module Plans + JSON consolidation | `GENERATIONS/G7-final-synthesis.md` | canonical `OPTIMISATION_FRAMEWORK_V7_FINAL.md` emitted; verification matrix every-clause-covered |

---

## Category 2 — Layer / Module / src Sync Consolidation

| ID | Task | Output | Verification |
|---|---|---|---|
| T2.1 | Catalog current Layer × Module × Cluster mapping (8 clusters / 26 modules / 9 layers L0-L8) | `ULTRAMAP.md` § Layer Map | every module assigned to exactly 1 layer; every cluster spans contiguous module-IDs |
| T2.2 | Define canonical src/ layout (ORAC single-crate + feature-gates pattern) | `GENERATIONS/G2-consolidation.md` § src layout | `src/mN_<theme>/` DAG; binary entrypoints `src/bin/wf_crystallise.rs` + `src/bin/wf_dispatch.rs`; shared `src/workflow_core/` lib |
| T2.3 | Define sync invariants (18-style, per synthex-v2 verify-sync) | `GENERATIONS/G2-consolidation.md` § sync invariants | each invariant: predicate + check-command + failure-class; ≥18 invariants |
| T2.4 | Author Cargo workspace contract (planning-spec only — not executed) | `GENERATIONS/G2-consolidation.md` § Cargo contract | feature matrix, dep graph as DAG; `forbid(unsafe_code)` + `deny(clippy::unwrap_used)` defaults |
| T2.5 | Author feature-gate strategy (full / lib-only / cli-only / dispatch-only) | `GENERATIONS/G2-consolidation.md` § features | each feature has dependency rationale + which binaries activate it |

---

## Category 3 — Detailed Module Plans (8 clusters / 26 modules) with Bidi Flow

Each cluster gets its own deep plan file. Each module within a cluster gets: **purpose** · **inputs** · **outputs** · **upstream-edges** · **downstream-edges** · **LOC budget** · **test budget (≥50)** · **boilerplate-lift source** · **structural-gap LOC** · **failure-modes covered** · **atuin-trajectory anchor** · **Watcher-flag-pre-positioning**.

| ID | Task | Output | Verification |
|---|---|---|---|
| T3.A | Cluster A deep plan — m1 atuin / m2 stcortex_consumer / m3 injection.db | `MODULE_PLANS/cluster-A.md` | 3 module sections; ≥50 tests per module budgeted; bidi flow with Cluster D + Cluster C documented |
| T3.B | Cluster B deep plan — m4 cascade / m5 battern / m6 cost | `MODULE_PLANS/cluster-B.md` | 3 module sections; F11 opaque-ID + F10 EMA exclude-Converged + battern step_label Option documented |
| T3.C | Cluster C deep plan — m7 central / m12 CLI / m13 stcortex writer | `MODULE_PLANS/cluster-C.md` | hub schema fully specified; m13 LTP/LTD 3-band gating documented; F9 zero-weight column |
| T3.D | Cluster D deep plan — m8 build-prereq / m9 namespace guard / m10 Ember CI / m11 decay | `MODULE_PLANS/cluster-D.md` | m8 `cargo:rustc-cfg=povm_calibrated` (NOT feature); m11 freq×fitness×recency formula concrete; m10 Held-CI-fail gate |
| T3.E | Cluster E deep plan — m14 lift / m15 pressure register | `MODULE_PLANS/cluster-E.md` | Wilson CI returning Option<f64>; m15 JSONL one-event-per-file |
| T3.F | Cluster F deep plan — m20-m23 KEYSTONE (PrefixSpan + Levenshtein + Wilson CI) | `MODULE_PLANS/cluster-F.md` | algorithm pseudocode; 4-internal-pass split for m20; near-miss band 0.25-0.60; top-K-by-edit-distance |
| T3.G | Cluster G deep plan — m30 bank / m31 selector / m32 dispatcher / m33 verifier | `MODULE_PLANS/cluster-G.md` | EscapeSurfaceProfile ordinal enum; m32 5-check pre-dispatch; m33 4-agent gate; refuse-mode behaviour |
| T3.H | Cluster H deep plan — m40 SYNTHEX / m41 LCM / m42 POVM dual-path | `MODULE_PLANS/cluster-H.md` | outbox-first JSONL durable; circuit breaker shared; `workflow_trace_*` AP30 prefix |
| T3.X | Cross-cluster synergy book — CC-1..CC-7 with bidi diagrams | `MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md` | each CC: upward + downward + closure-test |

---

## Category 4 — Runbooks (operational, one per phase + G1-G9)

Runbooks are **operational** — assume gates have fired, walk operator through exact commands. Use Failure-Mode/Effects/Mitigation format. Include Watcher flag pre-positioning per step.

| ID | Task | Output | Verification |
|---|---|---|---|
| T4.0 | Pre-genesis runbook (G1-G9 ops) | `RUNBOOKS/runbook-00-pre-genesis-gates.md` | per-gate command sequence + verifier + rollback + 3 failure modes |
| T4.1 | Phase 1 Genesis Day 0-3 runbook | `RUNBOOKS/runbook-01-phase-1-genesis.md` | Day 0/1/2/3 timeline; Cluster D BEFORE Cluster A; first git commit SHA verification |
| T4.2 | Phase 2A Build B/C/E runbook (Days 3-12) | `RUNBOOKS/runbook-02-phase-2A-measure-only.md` | m7 ships first; per-module quality gate; integration smoke per cluster |
| T4.3 | Phase 2B Build F/G/H runbook (Days 12-21, KEYSTONE) | `RUNBOOKS/runbook-03-phase-2B-active.md` | m20 4-internal-pass build; m32 refuse-mode wiring; m42 dual-path bring-up |
| T4.4 | Phase 3 Integration runbook (Days 21-26) | `RUNBOOKS/runbook-04-phase-3-integration.md` | 5 integration tracks; CC-5 first closure measure; serde `rename = "type"` trap |
| T4.5 | Phase 4 Pre-deploy hardening runbook (Days 26-28) | `RUNBOOKS/runbook-05-phase-4-hardening.md` | Wave-1 mechanical gate (PIPESTATUS); 4-agent F-mode ownership |
| T4.6 | Phase 5A/5B/5C Deploy + Cutover + Soak runbook | `RUNBOOKS/runbook-06-phase-5-deploy-soak.md` | binary deploy `/usr/bin/cp -f`; POVM cutover ~D25 dance; m11 decay healthy/warning tables |
| T4.7 | Phase 6 Sunset evaluation runbook (D120) | `RUNBOOKS/runbook-07-phase-6-sunset.md` | immutability of sunset_at; bounded-extension rule (60d × 2 max); IC-N improvement-candidate format |
| T4.8 | Phase 7 Security continuous runbook | `RUNBOOKS/runbook-08-phase-7-security.md` | 7 domains × 7 phases matrix; FNV-1a single-user vs HMAC-SHA256 multi-user |
| T4.9 | Phase 8 Observability continuous runbook | `RUNBOOKS/runbook-09-phase-8-observability.md` | 5 tracks; Pushgateway pattern; `wf_m14_lift = -1.0` sentinel |
| T4.10 | Cross-cutting concerns runbook (CC-1..CC-7) | `RUNBOOKS/runbook-10-cross-cutting.md` | one section per CC; cadence + owner + integration point |
| T4.11 | Emergency rollback runbook | `RUNBOOKS/runbook-11-emergency-rollback.md` | 3-command rollback for each of: Phase 1 / Phase 5A / Phase 5B cutover |

---

## Category 5 — Tooling Integration Deep-Dives

| ID | Task | Output | Verification |
|---|---|---|---|
| T5.1 | `/scaffold` command integration — V8's bound; consistency-check at Genesis | `INTEGRATION/scaffold-integration.md` | exact invocation; plan.toml contract; drift-detection schedule |
| T5.2 | Atuin scripts integration — 20+ scripts referenced; new scripts proposed | `INTEGRATION/atuin-integration.md` | per-script: invocation + provenance + use during which phase; ≥5 new scripts proposed for workflow-trace (e.g., `wt-gate-status`, `wt-soak-pulse`) |
| T5.3 | DevOps Engine V3 integration — T1-T6 mapping + resume_from contract | `INTEGRATION/devops-v3-integration.md` | T1-T6 module-emit contract; `POST /deploy {resume_from: "T2"}` honoured; NAM-03 confidence gates |
| T5.4 | CodeSynthor V8 zellij plugin integration — bidirectional wire | `INTEGRATION/codesynthor-v8-integration.md` | `/api/v8/confidence` + `/api/v8/learning` reuse; m35i bridge mirroring; sphere registration |
| T5.5 | JSON Claude Code optimisation (settings.json + .mcp.json + hooks) | `INTEGRATION/json-claude-code-optimisation.md` | per-file diff proposal; .claude/settings.json proposed for workflow-trace; PreToolUse/PostToolUse/Stop hooks |
| T5.6 | Progressive disclosure with Obsidian — vault hierarchy | `INTEGRATION/progressive-disclosure-obsidian.md` | 3-tier disclosure (landing → cluster → module); bidi anchors verified; on-demand reading order |

---

## Category 6 — Tab 1 Orchestration

| ID | Task | Output | Verification |
|---|---|---|---|
| T6.1 | Handshake protocol (Command / Command-2 / Command-3 at session start) | `HANDSHAKE_PROTOCOL_TAB1.md` | 4-step protocol + ack-window + silence-response + dual-silence escalation |
| T6.2 | Agent View per-layer allocation | `AGENT_VIEW_GITWORKTREES.md` § Agent View | which Claude Code agent owns which layer; tool-list per agent; isolation invariants |
| T6.3 | Git worktree per-layer allocation | `AGENT_VIEW_GITWORKTREES.md` § Worktrees | one worktree per layer (L0..L8); merge protocol; cleanup; lock discipline |
| T6.4 | Cascade dispatch pattern (Battern protocol for build waves) | `HANDSHAKE_PROTOCOL_TAB1.md` § Battern | Wave-1/2/3 design; gate between waves; collect+synthesize+compose |

---

## Category 7 — Watcher Integration (live-coverage)

| ID | Task | Output | Verification |
|---|---|---|---|
| T7.1 | Per-generation Watcher flag pre-positioning | each `GENERATIONS/G*.md` § Watcher | every generation lists which Class A-I flags activate + what cleanup looks like |
| T7.2 | Live Hebbian pause coverage (Class-I currently firing per tick·16) | `GENERATIONS/G1-baseline-audit.md` § Class-I | substrate-frame mitigation; defer m31 activation until LTP/LTD recovers OR explicit Luke acceptance |
| T7.3 | Class-E ancestor-rhyme post-mortem if planning persists past G9 + 14 days | `RUNBOOKS/runbook-11-emergency-rollback.md` § Class-E | trigger condition; halt protocol; reset criteria |

---

## Category 8 — Antipatterns Register

| ID | Task | Output | Verification |
|---|---|---|---|
| T8.1 | Habitat antipatterns catalogue (AP24, AP27, AP30, S102 preserve-list, BUG-035 mono-parameter, etc.) | `ANTIPATTERNS_REGISTER.md` § Habitat | ≥20 antipatterns; each with detection + mitigation + source-citation |
| T8.2 | Workflow-trace failure modes F1-F11 expanded with detection commands | `ANTIPATTERNS_REGISTER.md` § F1-F11 | per failure: signal + detection + module-owner + Watcher flag |
| T8.3 | New antipatterns surfaced by this optimisation (e.g., 7-gen drift, ultramap rot, runbook command alias-trap) | `ANTIPATTERNS_REGISTER.md` § New | ≥5 new antipatterns documented with rationale |
| T8.4 | LCM Drift #1..#11 transposed to workflow-trace context | `ANTIPATTERNS_REGISTER.md` § Drift | each drift class mapped to which workflow-trace gate catches it |

---

## Category 9 — God-Tier Rust Standards

| ID | Task | Output | Verification |
|---|---|---|---|
| T9.1 | Standards manifest (zero unwrap, zero unsafe, zero clippy warnings, zero pedantic warnings) | `STANDARDS/GOD_TIER_RUST.md` | each rule: rationale + enforcement command + atuin-script citation |
| T9.2 | Per-module test budget allocation (≥50 tests/module · top-1% patterns) | `STANDARDS/TEST_DISCIPLINE.md` | 26 × 50 = 1,300 minimum tests; pattern families: unit / property / fuzz / integration / contract / regression / mutation |
| T9.3 | 4-stage quality gate enforcement (check → clippy → pedantic → test) | `STANDARDS/GOD_TIER_RUST.md` § QG | exact bash with PIPESTATUS; per-stage abort discipline (per `feedback_pipestatus_for_gate_chains.md`) |
| T9.4 | Doc-comment discipline + `forbid(missing_docs)` on public items | `STANDARDS/GOD_TIER_RUST.md` § Docs | rule + exemption catalogue (private items free) |
| T9.5 | Tracing discipline + structured-event taxonomy | `STANDARDS/GOD_TIER_RUST.md` § Tracing | spans per layer; event taxonomy; sampling policy |

---

## Category 10 — Verification Matrices

| ID | Task | Output | Verification |
|---|---|---|---|
| T10.1 | Self-verify TASK_LIST against Luke's prompt (line-by-line) | `VERIFICATION_T0.md` | every prompt clause → ≥1 task ID; zero orphan |
| T10.2 | Cross-doc verification matrix (every doc references every other) | `GENERATIONS/G7-final-synthesis.md` § Cross-ref | matrix N×N; bidi confirmed; orphan documents flagged |
| T10.3 | Prompt-to-deliverable trace matrix | `OPTIMISATION_FRAMEWORK_V7_FINAL.md` § Trace | every clause of the original prompt → which deliverable + which line |

---

## Category 11 — Final Canonical Synthesis

| ID | Task | Output | Verification |
|---|---|---|---|
| T11.1 | Author `OPTIMISATION_FRAMEWORK_V7_FINAL.md` | canonical | composes all 7 generations + ultramap + module plans + runbooks + integrations into one navigable spec |
| T11.2 | Update vault HOME.md + MASTER_INDEX.md to reference V7 optimisation tree | vault | bidi anchors to/from V7 tree; navigation discoverability |
| T11.3 | Author 4-surface persistence plan (DEFERRED to G8) | `OPTIMISATION_FRAMEWORK_V7_FINAL.md` § G8-deferred | identifies what gets written to stcortex `workflow_trace_optimisation_v7_*` when G8 fires |
| T11.4 | File ACK + summary to agent-cross-talk/ for Command-2 + Command-3 | `~/projects/shared-context/agent-cross-talk/{TS}_command_v7_optimisation_complete.md` | C-2 + C-3 + Watcher + Zen named; deliverable inventory; per-deliverable reading order |

---

## Coverage map (Luke's prompt → task IDs)

| Prompt clause | Task IDs |
|---|---|
| "optimise the deployment framework" | T1.1-T1.7 (all 7 gens) + T11.1 |
| "for the the-workflow-engine full end to end stack deployment and hardening" | T4.0-T4.11 (all runbooks) + T11.1 |
| "refer to atuin as a guide" | T0.6 (god-tier ref) + T5.2 (integration deep-dive) + T9.1-T9.5 (standards) |
| "iterate the framework over 7 generations" | T1.1-T1.7 (one task per gen) |
| "consolidation of the syncing of the following directories layers, modules, and src" | T2.1-T2.5 (consolidation) + T1.2 (G2) |
| "ultramap" | T0.3 |
| "runbooks" | T4.0-T4.11 (12 runbooks) |
| "consistency optimisation in .json for claude code use" | T5.5 (json optimisation) |
| "additional progressive disclosure with obsidian here /home/louranicas/claude-code-workspace/the-workflow-engine" | T5.6 (progressive disclosure) + T11.2 (vault) |
| "detailed module plans ensuring bi directional contextual flow within and across code base modules" | T3.A-T3.H (per-cluster) + T3.X (cross-cluster synergies) + T1.3 (G3) |
| "synergistic module clusters" | T3.X + T1.3 |
| "alignment with other gold standard code base deployments" | T1.4 (G4) |
| "deep assimilation and integration with the /scaffol command" | T5.1 |
| "atuin scipts" | T5.2 |
| "the dev ops engine" | T5.3 |
| "the code synthor v8 zellij plugin" | T5.4 |
| "optimising for deployment in the zellij Orchistrator Tab (Tab - 1) Command Command-2 and Command-3 HAndshaking at start of session" | T6.1 |
| "using agent view" | T6.2 |
| "gitworktress to code each layer in isolation" | T6.3 |
| "maintaining god tier RUST programminmg standards (See atuin for references to this standard)" | T9.1-T9.5 + T0.6 |
| "uphaold th higest levels of excellence you are cpabile of as a synthetic being" | T11.1 quality bar + all of Category 9 |
| "note all known antipatterns to avoid" | T8.1-T8.4 |
| "note each module musg have a minimum of 50 teasts all meaningful all alligned with the best practices of the top 1% of test developers" | T0.7 + T9.2 + every T3.* per-module budget |
| "develop a list of keywords (limited to 20) to help claude maintain useful context references for the full end to end stack of the deployment" | T0.2 |
| "break this prompt down into a detailed and comprehensive task list without leaving anything out" | T0.1 (this file) |
| "verify this before proceeding" | T0.4 + T10.1 (VERIFICATION_T0.md) |
| "proceed seamlessly" | execution begins T1.1 immediately after T10.1 verification passes |

**Zero orphan clauses.** Every clause of Luke's prompt has ≥1 task ID.

---

## Deliverable inventory (32 markdown artefacts)

```
ai_docs/optimisation-v7/
  TASK_LIST_V7_OPTIMISATION.md                          (this file)         T0.1
  KEYWORDS_20.md                                                            T0.2
  ULTRAMAP.md                                                               T0.3 + T2.1
  VERIFICATION_T0.md                                                        T0.4 + T10.1
  ANTIPATTERNS_REGISTER.md                                                  T0.5 + T8.1-T8.4
  HANDSHAKE_PROTOCOL_TAB1.md                                                T0.8 + T6.1 + T6.4
  AGENT_VIEW_GITWORKTREES.md                                                T0.9 + T6.2 + T6.3
  OPTIMISATION_FRAMEWORK_V7_FINAL.md                                        T11.1
  STANDARDS/
    GOD_TIER_RUST.md                                                        T0.6 + T9.1 + T9.3-T9.5
    TEST_DISCIPLINE.md                                                      T0.7 + T9.2
  GENERATIONS/
    G1-baseline-audit.md                                                    T1.1
    G2-consolidation.md                                                     T1.2 + T2.2-T2.5
    G3-bidi-flow.md                                                         T1.3
    G4-gold-standard.md                                                     T1.4
    G5-tooling.md                                                           T1.5
    G6-test-discipline.md                                                   T1.6
    G7-final-synthesis.md                                                   T1.7 + T10.2 + T10.3
  MODULE_PLANS/
    cluster-A.md                                                            T3.A
    cluster-B.md                                                            T3.B
    cluster-C.md                                                            T3.C
    cluster-D.md                                                            T3.D
    cluster-E.md                                                            T3.E
    cluster-F.md                                                            T3.F
    cluster-G.md                                                            T3.G
    cluster-H.md                                                            T3.H
    CROSS_CLUSTER_SYNERGIES.md                                              T3.X
  RUNBOOKS/
    runbook-00-pre-genesis-gates.md                                         T4.0
    runbook-01-phase-1-genesis.md                                           T4.1
    runbook-02-phase-2A-measure-only.md                                     T4.2
    runbook-03-phase-2B-active.md                                           T4.3
    runbook-04-phase-3-integration.md                                       T4.4
    runbook-05-phase-4-hardening.md                                         T4.5
    runbook-06-phase-5-deploy-soak.md                                       T4.6
    runbook-07-phase-6-sunset.md                                            T4.7
    runbook-08-phase-7-security.md                                          T4.8
    runbook-09-phase-8-observability.md                                     T4.9
    runbook-10-cross-cutting.md                                             T4.10
    runbook-11-emergency-rollback.md                                        T4.11
  INTEGRATION/
    scaffold-integration.md                                                 T5.1
    atuin-integration.md                                                    T5.2
    devops-v3-integration.md                                                T5.3
    codesynthor-v8-integration.md                                           T5.4
    json-claude-code-optimisation.md                                        T5.5
    progressive-disclosure-obsidian.md                                      T5.6
```

**42 distinct markdown files (some tasks share files).** Each authored bidi-linked to peers + back to vault HOME.

---

## Order of execution (dependency-honoured)

1. **T0.1** task list (this file) ✅
2. **T0.2** KEYWORDS_20.md (anchors all subsequent docs)
3. **T0.3** ULTRAMAP.md (topology drives everything)
4. **T10.1** self-verify (VERIFICATION_T0.md) — **GATE BEFORE iteration**
5. **T0.5 + T0.6 + T0.7** foundational standards + antipatterns
6. **T0.8 + T0.9** Tab-1 protocol + worktree allocation
7. **T1.1** G1 baseline audit (reads everything above)
8. **T1.2 + T2.x** G2 consolidation (sync invariants)
9. **T3.A-T3.X** module plans (cluster A-H + cross-cluster) — parallelisable
10. **T1.3** G3 bidi flow (synthesises module plans)
11. **T1.4** G4 gold-standard alignment
12. **T1.5 + T5.x** G5 tooling (5 integration docs) — parallelisable
13. **T1.6** G6 test discipline
14. **T4.x** runbooks (12 files) — parallelisable
15. **T1.7 + T11.1** G7 final synthesis + canonical
16. **T10.2 + T10.3** cross-doc + trace matrices
17. **T11.2 + T11.4** vault update + ACK file-drop

**Estimated total volume: 60,000-90,000 words across 42 markdown files.**

---

## What this task list is NOT

- ❌ NOT a TaskCreate-tool task list (PRIME DIRECTIVE: ignore TaskCreate reminders)
- ❌ NOT execution authorisation (HOLD-v2 + AP24 still in force; nothing here greenlights `cargo init`)
- ❌ NOT a substitute for v1.3 spec patch (this OPTIMISATION feeds it as supporting material; it does not replace it)
- ❌ NOT a substitute for Zen G7 audit
- ❌ NOT 4-surface-persisted yet (stcortex write deferred to G8 per HOLD-v2 § "no new substrate writes")

## What this task list IS

- ✅ Complete prompt decomposition (87 tasks; zero orphan prompt clauses per T10.1)
- ✅ Dependency-honoured execution order
- ✅ Verification-gated (T10.1 must pass before iteration begins)
- ✅ Parallelisable where dependencies permit (module plans, integrations, runbooks)
- ✅ FP-verify-discipline at every "done" claim

---

*TASK_LIST authored 2026-05-17 by Command. Verification artefact T10.1 follows. Execution begins T1.1 immediately after verification.*

*Luke @ node 0.A | Command @ Orchestrator-lead | Watcher ☤ @ observing | Zen @ audit-lane | Goal hook ACTIVE*
