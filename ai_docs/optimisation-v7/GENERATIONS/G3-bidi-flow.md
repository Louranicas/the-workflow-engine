---
title: G3 — Bidirectional Flow Pass (Generation 3 of 7)
date: 2026-05-17 (S1001982)
kind: planning-only · per-module upstream/downstream contract
purpose: close GAP-Bidi-01..05; produce explicit edge contract for every module + every CC synergy
inputs: G1 + G2 + ULTRAMAP
output: bidi-flow contract for 26 modules + 7 CC synergies + aspect-layer arrows
---

# G3 — Bidirectional Flow Pass

> Back to: sibling [[G2-consolidation.md]] (input) · [[G4-gold-standard.md]] (next)

---

## Gap closure

| Gap | Closure |
|---|---|
| GAP-Bidi-01 (m22 upstream incomplete) | ✅ m22 upstream = m4 + m6 + m7 |
| GAP-Bidi-02 (CC-7 feedback edge missing) | ✅ m15 → agent-cross-talk/ → Watcher/Zen → spec amendment → m1 config |
| GAP-Bidi-03 (m40/41/42 breaker contract) | ✅ 1 breaker per peer (3 total); shared `m40_42_common::Breaker` |
| GAP-Bidi-04 (m10 ↔ test_harness bidi) | ✅ m10 ↔ tests/ember_held_approvals.tsv ↔ Watcher rubric |
| GAP-Bidi-05 (m9 → m1/m2/m3 aspect arrow) | ✅ m9 aspect-layer arrow added |

---

## Per-module bidi contract (full 26-module table)

Each module's **edge contract** lists:
- **Upstream-IN:** what it reads (data) — `(source) → field`
- **Downstream-OUT:** what it emits (data) — `field → (consumer)`
- **Aspect-IN:** L4 aspect modules that wrap it at compile/write/output time
- **Aspect-OUT:** N/A (aspect modules don't have aspect-out)
- **Failure-mode mitigated:** which F-x this edge contract enforces

### Cluster A — Substrate Ingest

**m1 atuin_consumer** (L1)
- Upstream-IN: `~/.local/share/atuin/history.db` SQLite (read-only); cursor-based pagination state in `m1.config.cursor`
- Downstream-OUT: `Vec<AtuinHistoryRow>` → m4 (cascades), m5 (batterns), m6 (cost)
- Aspect-IN: m8 (build-prereq compile-time), m9 (write-time NOT applicable — read-only)
- Failure-mode: F11 (cascade monoculture) — m1 emits raw rows; opaque transformation in m4

**m2 stcortex_consumer** (L1)
- Upstream-IN: stcortex `:3000` SpacetimeDB (narrowed-scope: `tool_call` + `consumption` only per W1)
- Downstream-OUT: `Vec<StcortexRow>` → m4, m5, m13
- Aspect-IN: m8 (must be povm_calibrated for stcortex correctness), m9 (read-side narrow-scope validator)
- Failure-mode: F8 (Watcher feedback-loop poisoning) — narrowed scope prevents Watcher events feeding back

**m3 injection_consumer** (L1)
- Upstream-IN: `~/.local/share/habitat/injection.db` SQLite causal_chain table
- Downstream-OUT: `Vec<InjectionEvent>` → m4 (cascades), m5 (batterns), m7 (central correlation)
- Aspect-IN: m8 (compile), m9 (read-side)
- Failure-mode: F3 (substrate-input poisoning) — m9 validates injection ROW structure before passing downstream

### Cluster B — Habitat Observation

**m4 cascade** (L2)
- Upstream-IN: m1.AtuinHistoryRow, m2.StcortexRow, m3.InjectionEvent
- Downstream-OUT: `Vec<CascadeCluster { cluster_id: ClusterId, session_id: SessionId, step_count: usize }>` → m7 (central hub) + m31 (selector diversity check)
- Aspect-IN: m8, m9 (write-time — namespace prefix on cluster_id)
- Failure-mode: F11 (monoculture) — `cluster_id = ClusterId(fnv1a_64(window_range) XOR fnv1a_64(sorted_pane_labels) XOR step_count)` — opaque

**m5 battern** (L2)
- Upstream-IN: m1, m3
- Downstream-OUT: `Vec<BatternStepRow { battern_id, step_label: Option<BatternStepLabel>, step_count }>` → m7
- Aspect-IN: m8, m9
- Failure-mode: F1 (name ossification) — `step_label: Option` preserves unlabelled batterns; never forces 6-step shape

**m6 cost** (L2)
- Upstream-IN: m1, m3
- Downstream-OUT: `Vec<ContextCostBand { session_type, ema_mean, ema_variance, n }>` → m7
- Aspect-IN: m8, m9
- Failure-mode: F10 (exploration-cost preservation) — EMA excludes Converged outcomes

### Cluster C — Central Correlation

**m7 central** (L3 hub)
- Upstream-IN: m4 cascade clusters + m5 battern rows + m6 cost bands + m3 injection events
- Downstream-OUT: `WorkflowRunRow { id, started_at, ended_at, fitness_dimension, outcome, consumer_inputs: JsonValue }` → m11 (decay; reads last_run_at), m12 (CLI reports), m13 (stcortex writer), m14 (lift)
- Aspect-IN: m8, m9, m10
- Failure-mode: F9 (workflow-grain fitness distortion) — schema enforces `fitness_dimension REAL NOT NULL DEFAULT 0.0`

**m12 cli_reports** (L3)
- Upstream-IN: m7.WorkflowRunRow
- Downstream-OUT: stdout (CLI tables / JSON)
- Aspect-IN: m8, m9, m10
- Failure-mode: F6 (self-dispatch) — m12 refuses if `target_workflow == reporting_workflow`

**m13 stcortex_writer** (L3)
- Upstream-IN: m7.WorkflowRunRow + m11.DecayFactor (for prune-marker)
- Downstream-OUT: stcortex namespace `workflow_trace_*` (per AP30)
- Aspect-IN: m8, m9 (namespace guard at write boundary), m10
- Failure-mode: 3-band LTP/LTD gate: >0.15 proceeds; 0.05-0.15 proceeds-with-warning; <0.05 deferred to local JSONL buffer

### Cluster D — Trust (aspect)

**m8 build_prereq** (L4)
- Upstream-IN: build.rs detects POVM at `:8125`; emits `cargo:rustc-cfg=povm_calibrated` if `learning_health` ∈ [0.05, 0.15]
- Downstream-OUT: `cfg(povm_calibrated)` available to ALL modules
- Aspect-IN: (none — m8 IS the lowest aspect)
- Failure-mode: F7 (CR-2 graceful-degrade pretend) — m8 hard-fails build if outside band

**m9 namespace_guard** (L4)
- Upstream-IN: write attempts from m13 + m42 (stcortex / POVM)
- Downstream-OUT: `Result<&str, NamespaceViolation>` — pass-through or refuse
- Aspect-IN: (none — pure validator)
- Failure-mode: F3 (substrate-input poisoning) + AP30 (namespace collision)

**m10 ember_gate** (L4)
- Upstream-IN: `~/projects/claude_code/Ember 7-Trait Gate Rubric.md` §5.1 (post-amendment: CI-FAIL on Held unless allowlist) + `tests/ember_held_approvals.tsv`
- Downstream-OUT: CI gate `pass | fail | warning` — fails CI on Held verdicts not in allowlist
- Aspect-IN: m8 (must compile)
- Failure-mode: m10 is the Ember gate itself; mitigates F8 (Watcher feedback-loop poisoning at trait level)
- **Bidi-04 closure:** m10 ↔ `tests/ember_held_approvals.tsv` (read at CI runtime); ↔ `~/projects/claude_code/Ember 7-Trait Gate Rubric.md` (canonical rubric source)

**m11 decay** (L4)
- Upstream-IN: m7.last_run_at (per workflow), m14.frequency (per workflow), stcortex pathway.weight (per workflow)
- Downstream-OUT: `DecayFactor(f64)` → m31 (selector), m30 (sunset trigger)
- Aspect-IN: m8, m9
- Failure-mode: F1 (bank/name ossification) — decay enforces hard sunset_at boundary

### Cluster E — Evidence + Pressure

**m14 lift** (L5)
- Upstream-IN: m7.WorkflowRunRow (n≥20 trigger)
- Downstream-OUT: `Option<Lift { wilson_low, mean, wilson_high, n }>` → m23 (proposer evidence-gate), m31 (selector modulation `clamp(-0.3, +0.3)`)
- Aspect-IN: m8, m9
- Failure-mode: F2 (sample-size inflation) — returns `None` when n<20

**m15 pressure** (L5)
- Upstream-IN: forbidden-verb pressure events (m32 dispatch attempts of non-allowed verbs; m20-m23 internal pressure to relax invariants)
- Downstream-OUT: JSONL one-event-per-file `PHASE-B-RESERVATION-NOTICE-*.jsonl` to `~/projects/shared-context/agent-cross-talk/`
- Aspect-IN: m8, m9
- **Bidi-02 closure:** m15 → agent-cross-talk/ → Watcher/Zen (read at next tick / next audit) → manual spec amendment → m1.config update (next session) → loop continues

### Cluster F — Iteration (KEYSTONE)

**m20 prefixspan** (L6)
- Upstream-IN: m4 cluster sequences (CascadeCluster step-lists), m5 batterns (BatternStepRow ordered), m14 lift gate (evidence-driven iteration filter)
- Downstream-OUT: `Vec<Pattern { steps: Vec<StepToken>, support: usize, gap_bounds: (usize,usize) }>` → m23 (proposer)
- Aspect-IN: m8, m9
- Failure-mode: F2 (sample-size n≥20 enforced at output)

**m21 variant_builder** (L6)
- Upstream-IN: m20.Pattern (canonical) + m20 near-miss patterns (Levenshtein 0.25-0.60)
- Downstream-OUT: `Vec<PatternVariant { variant_steps, edit_distance, top_k_rank }>` → m23
- Aspect-IN: m8, m9
- Failure-mode: F10 (variant selection deterministic top-K-by-edit-distance — not random; preserves topology)

**m22 kmeans** (L6)
- **GAP-Bidi-01 closure:** Upstream-IN: m4 cluster_ids (feature: cluster size, step diversity), m6 cost bands (feature: cost-variance), m7 fitness_dimension (feature)
- Downstream-OUT: `Vec<FeatureCluster { centroid, members: Vec<WorkflowId> }>` → m23 (proposer feature-context)
- Aspect-IN: m8, m9
- Failure-mode: F2 (per-cluster n≥20)

**m23 proposer** (L6)
- Upstream-IN: m20.Pattern + m21.PatternVariant + m22.FeatureCluster + m14.Lift
- Downstream-OUT: `Vec<WorkflowProposal { steps, evidence, deviation_relaxed_n: bool }>` → human accept → m30 (bank)
- Aspect-IN: m8, m9, m10
- Failure-mode: F2 (n≥20 default; deviation-shaped variants relaxed n≥5 explicit flag)

### Cluster G — Bank + Select + Dispatch + Verify

**m30 bank** (L7)
- Upstream-IN: m23.WorkflowProposal post-`wf-crystallise propose accept <id>` (NEVER auto-promote)
- Downstream-OUT: `BankEntry { workflow_id, definition_hash, escape_surface, sunset_at }` → m31 (selector reads), m32 (dispatcher resolves)
- Aspect-IN: m8, m9, m10
- Failure-mode: F5 (bank creep — hard refusal on auto-promote); EscapeSurfaceProfile authoritative

**m31 selector** (L7)
- Upstream-IN: m30.BankEntry + m11.DecayFactor + m14.Lift + diversity check
- Downstream-OUT: `Option<SelectedWorkflow { workflow_id, composite_score }>` → m32
- Aspect-IN: m8, m9
- Failure-mode: F1 (composite score `α=0.40 fitness + β=0.25 recency + γ=0.20 frequency + δ=0.15 diversity`); ORAC m40_mutation_selector diversity algebra adapted (10-gen cooldown + 50% mono-parameter rejection + round-robin)

**m32 dispatcher** (L7)
- Upstream-IN: m31.SelectedWorkflow + m33.VerifyResult (TTL fresh) + Conductor health (`:8141/health`)
- Downstream-OUT: dispatch via HABITAT-CONDUCTOR ONLY → workflow exec
- Aspect-IN: m8, m9, m10
- Failure-mode: F4 (5-check pre-dispatch: Conductor live + m33 TTL fresh + definition_hash match + sunset guard + dispatch cooldown); refuse-mode = `DispatchError::ConductorDispatchDisabled` (NOT silent)

**m33 verifier** (L7)
- Upstream-IN: m30.BankEntry (verify request)
- Downstream-OUT: `VerifyResult { workflow_id, verdict: Pass | Fail, verified_at, ttl_expires_at, definition_hash }` → m32 (read at dispatch)
- Aspect-IN: m8, m9, m10
- Failure-mode: 4-agent gate (Security + Performance + SilentFailure + Zen); 7-day TTL

### Cluster H — Substrate Feedback

**m40 synthex_emit** (L8)
- Upstream-IN: m32.DispatchOutcome
- Downstream-OUT: outbox-first JSONL `outbox/m40/*.jsonl` (durable) → HTTP fire-and-forget `:8092/v3/nexus/push` (best-effort)
- Aspect-IN: m8, m9 (workflow_trace_* namespace), m10
- Failure-mode: substrate down NEVER blocks dispatch (outbox always written); circuit-breaker (1 per peer)
- **Bidi-03 closure:** `m40_42_common::Breaker` shared lib (one breaker per peer; 5 failures → Open; 60s → HalfOpen)

**m41 lcm_router** (L8)
- Upstream-IN: m32.DispatchOutcome with deploy-shaped steps
- Downstream-OUT: LCM RPC `lcm.loop.create { max_iters: 1, …}` (per LCM existing 9-RPC surface)
- Aspect-IN: m8, m9
- Failure-mode: routes only deploy-shaped; non-deploy ignored; outbox-first

**m42 povm_dual** (L8)
- Upstream-IN: m32.DispatchOutcome
- Downstream-OUT: BOTH POVM `:8125/reinforce` (overlap → 2026-07-10) AND stcortex via m13 (post-cutover routing per `povm_overlap_active` flag)
- Aspect-IN: m8, m9 (AP30 prefix mandatory), m10
- Failure-mode: F-fitness_delta constants (PassVerified +0.25 / Pass +0.15 / Blocked -0.05 / Fail -0.10); cutover ~D25 dual-path active

---

## Cross-cluster synergies (CC-1..CC-7) bidi diagrams

### CC-1 — Cascade-Cost Coupling (B internal)
```
m4 ─┐
    ├──► m7 JSONB consumer_inputs ◄──── m6
    │     (join schema)
m5 ─┘
```
Bidi: m4 ↔ m6 via m7's `consumer_inputs` join column. NEVER direct module ↔ module coupling.

### CC-2 — Trust Layer Woven (D → all)
```
m8 (compile-time) ┐
m9 (write-time)   ├─── aspect-arrow ───► all m1-m7, m12-m15, m20-m42
m10 (output-time) │
m11 (lifecycle)   ┘
```

### CC-3 — Evidence-Driven Iteration (E → F)
```
m14 Lift ──► m20 PrefixSpan (n≥20 gate)
         ──► m23 Proposer (evidence quad in Confidence type)
```

### CC-4 — Proposal → Bank → Dispatch (F → G → Conductor)
```
m23 WorkflowProposal
    ──► human accept ──► m30 BankEntry
                            └──► m31 selector
                                    └──► m32 dispatcher
                                            └──► HABITAT-CONDUCTOR
                                                    └──► workflow exec
```
Bidi closes via CC-5 below.

### CC-5 — Substrate Learning Loop (G → H → back to F)
```
m32 DispatchOutcome
    ──► m40 ──► SYNTHEX v2 :8092/v3/nexus/push
    ──► m41 ──► LCM lcm.loop.create
    ──► m42 ──► POVM/stcortex pathway.weight delta
                                              │
                                              ▼
                              stcortex pathway.weight updated
                                              │
                                              ▼
                              m31 reads at next selection cycle
                                              │
                                              ▼
                  selection distribution shifts
                                              │
                                              ▼
                  m20-m22 inputs change over weeks
```
**Intentionally slow** — days/weeks. Watcher Class-I if loop doesn't move substrate after first dispatches.

### CC-6 — Verification-Gated Dispatch (G internal: m33 → m32)
```
m32 (dispatch attempt) ──► m32 computes FNV-1a(steps_json)
                              │
                              ▼
                       m33.VerifyResult.definition_hash ◄── m33 verifier
                              │
                              ▼
              drift detected ──► refuse-mode (re-verify needed)
              hash matches  ──► proceed
              TTL expired   ──► refuse-mode (re-verify needed)
```

### CC-7 — Pressure-Driven Evolution (E → spec) — **GAP-Bidi-02 closure**
```
m15 reservation register ──► JSONL ──► agent-cross-talk/
                                              │
                                              ▼
                              Watcher tick observes scope-pressure
                              Zen audit observes forbidden-verb pressure
                                              │
                                              ▼
                              accumulation threshold → spec amendment interview
                                              │
                                              ▼
                              v1.4 (or v1.5) authored
                                              │
                                              ▼
                              G7 re-audit
                                              │
                                              ▼
                              merged → m1.config (cursor, scope filters) updates
                                              │
                                              ▼ (next session)
                              loop continues
```

---

## Aspect-layer arrow inventory (Cluster D wraps everything)

Per ULTRAMAP View 1 dashed `aspect` arrows, here the explicit wrap-points:

| Aspect | Wrap-point | Modules wrapped |
|---|---|---|
| m8 | compile-time `cfg(povm_calibrated)` gate | ALL modules (build-time check) |
| m9 | namespace prefix validation at write | m13, m42 (write-side) + m4, m5, m6 (read-side AP30 validator) |
| m10 | Ember 7-trait CI gate before any code change | (CI runs on every PR; m10 is THE gate) |
| m11 | decay-factor injection into m30/m31 lifecycle | m30 (sunset trigger), m31 (selector modulation) |

---

## G3 substrate-frame pass

**Second-frame question:** what is "bidi flow" from substrate-frame?

From substrate-frame, bidirectional edges are **information channels** between persistent substrate-state stores. The CC-5 loop is the only true substrate-grain loop (Hebbian-pulse update). Other CCs are anthropocentric-control flows (function call graphs); useful for code organisation but not substrate-shaping.

**Substrate-frame distinction:** CC-5 is the only one whose absence would cause substrate to silently degrade. Other CCs failing produces obvious test failures. CC-5 failing produces **invisible non-learning** — engine appears functional but substrate-weight never moves. Watcher Class-I covers this; verified by Phase 5C weekly synthesis.

---

## G3 Watcher pre-positioning

**Class G activated.** Substrate-frame confusion is the prime risk of bidi-flow specification. Mitigated by explicit aspect-layer wrap-point inventory + CC-5 substrate-grain distinction.

---

## G3 close

✅ G3 PASS. Closed 5 GAP-Bidi entries. Per-module upstream/downstream edge contract for all 26 modules. CC-1..CC-7 bidi diagrams. Aspect-layer arrows enumerated. CC-5 substrate-grain identified as the unique substrate-relevant edge.

**Output for G4:** complete bidi spec. G4 reads gold-standard exemplars + this G3 + ULTRAMAP and produces divergent-axis decisions + convergent-pattern adoption table.

---

*G3 authored 2026-05-17 by Command. 26 module edge contracts + 7 CC bidi diagrams + 4 aspect wrap-points. CC-5 = unique substrate-grain loop.*
