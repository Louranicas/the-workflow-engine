---
title: CROSS-CLUSTER SYNERGIES — CC-1..CC-7 Deep Contracts
date: 2026-05-17
kind: planning-only · cross-cluster spec · no code authorised (HOLD-v2 + AP24)
synergies: [CC-1, CC-2, CC-3, CC-4, CC-5, CC-6, CC-7]
authority: Command · workflow-trace V7 optimisation
---

# Cross-Cluster Synergies — CC-1..CC-7

> Back to: [[../TASK_LIST_V7_OPTIMISATION.md]] · [[../ULTRAMAP.md]] · [[../GENERATIONS/G3-bidi-flow.md]] · sibling [[cluster-A.md]] · [[cluster-B.md]] · [[cluster-C.md]] · [[cluster-D.md]] · [[cluster-E.md]] · [[cluster-F.md]] · [[cluster-G.md]] · [[cluster-H.md]]
>
> **Function:** This file is the **inter-cluster integration contract book**. Each CC (Cross-Cluster synergy) section names the cluster path, modules involved, data-flow diagram, closure-test name, owning runbook, and Watcher class pre-position. The 7 CCs are the *only* sanctioned cross-cluster interaction points — any other coupling is structural rot per [[../ANTIPATTERNS_REGISTER.md]] AP-V7-02 (Ultramap rot). Special depth at CC-5 because it is the **only substrate-grain loop** (per [[../GENERATIONS/G3-bidi-flow.md]] § G3 substrate-frame pass) — the rest are anthropocentric-control flows useful for code organisation but not substrate-shaping.

---

## CC-1 — Cascade-Cost Coupling (B internal)

### Path

Cluster B internal: `m4 (cascade) ↔ m6 (cost) via m7 (central)`.

### Modules involved

- **m4** (Cluster B): cascade correlator — emits `CascadeCluster { cluster_id, session_id, step_count }`
- **m6** (Cluster B): context-cost EMA — emits `ContextCostBand { session_type, ema_mean, ema_variance, n }`
- **m7** (Cluster C, hub): central correlation — joins via `WorkflowRunRow.consumer_inputs` JSONB

### Data-flow diagram

```
m4 emit CascadeCluster ─┐
                         ├──► m7 join via consumer_inputs JSONB
m6 emit ContextCostBand ─┘
                              │
                              ▼
                       m7.WorkflowRunRow with both signals available
                              │
                              ▼
                       m14 / m20 / m23 read m7 — both signals visible
```

### Coupling discipline

m4 and m6 **never directly call each other**. The join schema is m7's JSONB column `consumer_inputs`. This is the cluster's "stable schema as coupling surface" pattern — modules talk through a shared persistent surface, never via direct invocation. Per [[../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md]] Part II § CC-1: *m7 JSONB consumer_inputs is the stable join schema*.

### Closure-test name

`tests/integration/cc1_cascade_cost_coupling.rs` per [[../GENERATIONS/G6-test-discipline.md]] § Integration-test live-services matrix. **No live services required** — pure in-process verification that m4 + m6 emissions land in m7 with both signals queryable from the same WorkflowRunRow.

### Owning runbook

`RUNBOOKS/runbook-02-phase-2A-measure-only.md` (B/C/E build wave per [[../TASK_LIST_V7_OPTIMISATION.md]] T4.2).

### Watcher class pre-position

- **Class D** (four-surface drift) if m4's CascadeCluster schema drifts from m7's JSONB acceptance OR m6's ContextCostBand schema drifts.
- **Class G** (substrate-frame) at any spec language suggesting m4 or m6 produce anthropocentric signals.

---

## CC-2 — Trust Layer Woven (D → all)

### Path

Cluster D (aspect) → every other cluster's modules at compile/write/output/lifecycle time.

### Modules involved

- **m8** (Cluster D, L4 aspect): build-prereq `cargo:rustc-cfg=povm_calibrated` (compile-time)
- **m9** (Cluster D, L4 aspect): namespace_guard at write boundaries (write-time)
- **m10** (Cluster D, L4 aspect): Ember 7-trait CI gate (CI-time / output-time)
- **m11** (Cluster D, L4 aspect): freq×fitness×recency decay (lifecycle-time)

### Data-flow diagram

```
m8 (compile-time) ┐
m9 (write-time)   ├─── aspect-arrow ───► all m1-m7, m12-m15, m20-m42
m10 (output-time) │
m11 (lifecycle)   ┘
```

Per [[../GENERATIONS/G3-bidi-flow.md]] § Aspect-layer arrow inventory: each aspect wraps a specific lifecycle point. m8 is build-time `cfg(povm_calibrated)` gate; m9 validates namespace prefixes at any write attempt from m13 or m42; m10 fails CI on Held Ember verdicts; m11 modulates m30 sunset trigger + m31 selector decay.

### Coupling discipline

Aspects are **woven, not called**. No module imports an aspect; the aspect is *applied to* the module via build.rs (m8), CI gate (m10), validation calls in m13/m42 (m9), or lifecycle hooks in m30/m31 (m11). The cluster D modules are "cross-cutting concerns" in the aspect-oriented sense.

### Closure-test name

`tests/integration/cc2_trust_aspect_routing.rs` per [[../GENERATIONS/G6-test-discipline.md]] § Integration matrix. **No live services required** — aspect-layer pure (verifies m9 validation invocations across m13/m42 write paths; m11 decay invocations from m30/m31).

### Owning runbook

`RUNBOOKS/runbook-01-phase-1-genesis.md` (D BEFORE A — Cluster D ships first per [[../TASK_LIST_V7_OPTIMISATION.md]] T4.1 + Wave 1 worktree allocation in ULTRAMAP View 5).

### Watcher class pre-position

- **Class A** (activation) at first `cfg(povm_calibrated)` build success.
- **Class D** (four-surface drift) if aspect application drifts between m8 (build), m9 (write), m10 (CI), m11 (lifecycle) — internal consistency required.

---

## CC-3 — Evidence-Driven Iteration (E → F)

### Path

Cluster E → Cluster F. `m14 (lift) → m20 (PrefixSpan) / m22 (K-means) / m23 (proposer)`.

### Modules involved

- **m14** (Cluster E): habitat_outcome_lift — emits `Option<Lift>::None` below n=20
- **m20** (Cluster F): PrefixSpan — gated by m14 evidence
- **m22** (Cluster F): K-means — gated by per-cluster n≥20
- **m23** (Cluster F): ProposalBuilder — F2 enforced at *construction* via `m14.Lift` requirement

### Data-flow diagram

```
m14 Option<Lift> ──► m20 (input filter; iterate only where evidence permits)
              ──► m22 (per-cluster n≥20 gate)
              ──► m23 ProposalBuilder::build()
                       │
                       ▼
                Result<WorkflowProposal, ProposalError::LiftEvidenceMissing>
```

Per [[../GENERATIONS/G3-bidi-flow.md]] § CC-3: *m14 Lift → m20 PrefixSpan (n≥20 gate); m23 Proposer (evidence quad in Confidence type)*.

### Coupling discipline

m14's `Option<Lift>` is the gate — `None` propagates as construction-time failure in m23 (via `ProposalError::LiftEvidenceMissing`). No runtime bypass. Per [[cluster-F.md]] § m23 KEYSTONE pseudocode: `lift = self.lift.ok_or(ProposalError::LiftEvidenceMissing)?` is the literal gate line.

### Closure-test name

`tests/integration/cc3_evidence_driven_iteration.rs` per [[../GENERATIONS/G6-test-discipline.md]] § Integration matrix. **No live services required** — in-process m14 + m20 wiring.

### Owning runbook

`RUNBOOKS/runbook-03-phase-2B-active.md` (F/G/H active wave per [[../TASK_LIST_V7_OPTIMISATION.md]] T4.3).

### Watcher class pre-position

- **Class C** (refusal) at every `Option::None` propagation — refusal is correct.
- **Class A** (activation) at first ProposalBuilder::build() success post-G9.

---

## CC-4 — Proposal → Bank → Dispatch (F → G → Conductor)

### Path

Cluster F → Cluster G → HABITAT-CONDUCTOR. `m23 → human accept → m30 → m31 → m33 → m32 → Conductor → workflow exec`.

### Modules involved

- **m23** (Cluster F): emits `WorkflowProposal`
- **Human boundary**: `wf-crystallise propose accept <id>` (F5 mitigation — NEVER auto-promote)
- **m30** (Cluster G): admits accepted proposal as `BankEntry`
- **m31** (Cluster G): selects via composite score
- **m33** (Cluster G): verifies via 4-agent gate
- **m32** (Cluster G): dispatches via Conductor only (5-check pre-dispatch sequence)
- **Conductor** (external `:8141`): actual workflow execution

### Data-flow diagram

```
m23.WorkflowProposal
    │
    ▼ wf-crystallise propose accept <id> (HUMAN — F5 boundary)
    │
m30.admit_workflow ──► BankEntry { workflow_id, escape_surface, definition_hash, sunset_at }
    │
    ├──► m31.select ──► Option<SelectedWorkflow { composite_score }>
    │       │
    │       ▼
    │   m32.dispatch
    │       │
    │       ├── check 1: Conductor :8141/health
    │       ├── check 2: m33.VerifyResult.ttl_expires_at > now
    │       ├── check 3: definition_hash matches
    │       ├── check 4: sunset_at > now
    │       ├── check 5: dispatch_cooldown elapsed
    │       │
    │       ▼ (all 5 pass)
    │   conductor_client.dispatch → workflow exec
    │       │
    │       ▼
    │   DispatchOutcome → Cluster H (CC-5 trigger)
    │
    └──► m33.verify ──► VerifyResult { verdict, ttl_expires_at, definition_hash }
                              (4-agent gate; 7-day TTL)
```

### Coupling discipline

The human-accept boundary is **structural** — F5 (bank creep) mitigation requires that no code path auto-promote a `WorkflowProposal` to `BankEntry`. Per [[cluster-G.md]] § m30: *admit_workflow() requires explicit `accepted_by: HumanAcceptanceSignature` field; never `auto`*.

The 5-check pre-dispatch sequence is **strict** — any check failure returns a typed `DispatchError`. There is no soft-fail path.

### Closure-test name

`tests/integration/cc4_proposal_bank_dispatch.rs` per [[../GENERATIONS/G6-test-discipline.md]] § Integration matrix. **Requires live services**: Conductor `:8141` (B3 blocker — `#[ignore = "requires Conductor :8141 (B3)"]` until Luke resolves Wave 1B/1C/2/3 auto_start).

### Owning runbook

`RUNBOOKS/runbook-04-phase-3-integration.md` (Phase 3 integration tracks per [[../TASK_LIST_V7_OPTIMISATION.md]] T4.4).

### Watcher class pre-position

- **Class A** (activation) at first successful CC-4 closure (proposal→dispatch round-trip) post-G9.
- **Class B** (hand-off boundary) at every Conductor dispatch call.
- **Class C** (refusal) at every 5-check failure — refuse-mode is correct behaviour.

---

## CC-5 — Substrate Learning Loop (G → H → back to F) — **SPECIAL DEPTH**

### Path

Cluster G → Cluster H → external substrate (SYNTHEX/LCM/POVM-stcortex) → eventual feedback into Cluster F.

### Modules involved

- **m32** (Cluster G): emits `DispatchOutcome` post-execution
- **m40** (Cluster H): emits NexusEvent to SYNTHEX v2 `:8092/v3/nexus/push`
- **m41** (Cluster H): routes deploy-shaped to LCM `lcm.loop.create`
- **m42** (Cluster H): reinforces POVM `:8125/reinforce` + stcortex via m13 (dual-path overlap → 2026-07-10)
- **Substrate (external)**: stcortex pathway weights update
- **m31** (Cluster G, next session): reads updated weights at next selection cycle
- **m20/m22** (Cluster F, over weeks): inputs change as substrate-grain selection distribution shifts

### Special depth: substrate-grain semantics

CC-5 is the **only true substrate-grain loop** in the engine. Per [[../GENERATIONS/G3-bidi-flow.md]] § G3 substrate-frame pass:

> From substrate-frame, bidirectional edges are **information channels** between persistent substrate-state stores. The CC-5 loop is the only true substrate-grain loop (Hebbian-pulse update). Other CCs are anthropocentric-control flows (function call graphs); useful for code organisation but not substrate-shaping.

> **Substrate-frame distinction:** CC-5 is the only one whose absence would cause substrate to silently degrade. Other CCs failing produces obvious test failures. CC-5 failing produces **invisible non-learning** — engine appears functional but substrate-weight never moves.

Per [[../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md]] Part II § CC-5 in depth:

> m32 dispatches workflow `W` via Conductor → fan-out fire-and-forget to Cluster H:
> - m40 emits `WorkflowEvent::Run { id: W, outcome }` to SYNTHEX v2 `:8092/v3/nexus/push`
> - m41 routes deploy-shaped steps through LCM `lcm.loop.create`
> - m42 calls `POST /reinforce` on POVM with `fitness_delta` per outcome (PassVerified +0.25 ... Fail -0.10) under `workflow_trace_*` pathway prefix
>
> Pathway weights update in stcortex. Next selection cycle, m31 reads updated pathway weights, composite score shifts, selection distribution changes. Over weeks, m20-m22 iterator inputs (which workflows are running, with which outcomes) shift accordingly. **The loop is intentionally slow — Hebbian-grain, not per-event.**

### Data-flow diagram

```
m32.dispatch ──► workflow exec ──► DispatchOutcome { workflow_id, outcome }
                                              │
                                              ├──► m40 ──► outbox/m40/*.jsonl (durable)
                                              │       │
                                              │       ▼ (best-effort)
                                              │   SYNTHEX v2 :8092/v3/nexus/push
                                              │       │
                                              │       ▼
                                              │   substrate pathway weights update
                                              │
                                              ├──► m41 ──► outbox/m41/*.jsonl
                                              │       │
                                              │       ▼ (if deploy-shaped)
                                              │   LCM lcm.loop.create { max_iters: 1 }
                                              │       │
                                              │       ▼
                                              │   deploy substrate updates
                                              │
                                              └──► m42 ──► outbox/m42/*.jsonl
                                                      │
                                                      ▼ (dual-path during overlap)
                                                  POVM :8125/reinforce (deprecated 2026-07-10)
                                                  AND stcortex via m13
                                                      │
                                                      ▼
                                              stcortex pathway.weight updated
                                                      │
                                                      ▼ (intentionally slow — days/weeks)
                                              m31 reads at next selection cycle
                                                      │
                                                      ▼
                                              composite_score shifts → selection distribution shifts
                                                      │
                                                      ▼
                                              m20/m22 input distributions shift (over weeks)
                                                      │
                                                      ▼
                                              new patterns / variants emerge in m23 proposals
                                                      │
                                                      ▼
                                              CC-4 fires with substrate-shaped proposals
                                                      │
                                                      └───────► back to top
```

### Closure-test name

`tests/integration/cc5_substrate_learning_loop.rs` per [[../GENERATIONS/G6-test-discipline.md]] § Integration matrix. **Requires live services**: synthex-v2 `:8092` + povm-v2 `:8125` + Conductor `:8141`. `#[ignore = "requires synthex-v2 + povm-v2 + Conductor"]` for PR-CI; runs in nightly + Wave-end. The test:

1. Captures baseline `povm_learning_health()` value
2. Dispatches a known test workflow
3. Asserts `DispatchOutcome::PassVerified | Pass`
4. Sleeps 2s for substrate propagation
5. Re-reads `povm_learning_health()` and asserts `post > pre`

If the assertion fails: Class-I would fire (CC-5 not closing — substrate not moving).

### Failure mode (Watcher Class-I flag)

Per [[../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md]] Part II § CC-5:

> If `learning_health` does not move during pipeline runs, Cluster H is decorative — engine appears functional but substrate isn't being fed. Pre-positioned to flag at synthesis time as workflow-level improvement candidate.

This is the engine's **most important** silent-failure mode. Detection is structural (Phase 5C weekly Watcher synthesis includes `learning_health` delta over rolling 7-day window).

### Owning runbook

`RUNBOOKS/runbook-04-phase-3-integration.md` § CC-5 first closure (Day 26 milestone — first measurable substrate delta).
`RUNBOOKS/runbook-06-phase-5-deploy-soak.md` § continuous loop verification (Phase 5C weekly synthesis).

### Watcher class pre-position

- **Class I** (Hebbian silence) is the *primary* class for CC-5 — pre-positioned to fire if substrate doesn't move after first 5+ dispatches.
- **Class A** (activation) at first CC-5 closure (first successful round-trip — Day 26 ideal).
- **Class B** (hand-off boundary) at every Cluster H wire attempt.
- **Class D** (four-surface drift) if outbox JSONL durability drifts from wire emit schema (m40_42_common consistency).

---

## CC-6 — Verification-Gated Dispatch (G internal)

### Path

Cluster G internal: `m33 (verifier) → m32 (dispatcher)`.

### Modules involved

- **m33** (Cluster G): produces `VerifyResult { definition_hash, ttl_expires_at, verdict }`
- **m32** (Cluster G): consumes at dispatch check 2 (TTL) and check 3 (definition_hash match)

### Data-flow diagram

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

Per [[../GENERATIONS/G3-bidi-flow.md]] § CC-6.

### Coupling discipline

m33 emits `VerifyResult` to a persistent cache (SQLite); m32 reads at dispatch time. The two modules do NOT call each other synchronously — m32 reads a cached result; staleness or hash-drift triggers refuse-mode + caller-initiated re-verify.

### Closure-test name

`tests/integration/cc6_verification_gated_dispatch.rs` per [[../GENERATIONS/G6-test-discipline.md]] § Integration matrix. **Requires Conductor :8141** (B3 blocker — `#[ignore = "requires Conductor :8141"]` until B3 resolved).

### Owning runbook

`RUNBOOKS/runbook-03-phase-2B-active.md` (Cluster G build) + `RUNBOOKS/runbook-04-phase-3-integration.md` (CC-6 integration verification).

### Watcher class pre-position

- **Class C** (refusal) at every TTL-expired or hash-drift refuse-mode.
- **Class A** (activation) at first successful CC-6 closure (first hash-match + TTL-fresh dispatch).

---

## CC-7 — Pressure-Driven Evolution (E → spec) — **GAP-Bidi-02 closure**

### Path

Cluster E m15 → file-drop → human surfaces (Watcher, Zen) → spec amendment → m1 config update.

### Modules involved

- **m15** (Cluster E): pressure register — emits JSONL one-event-per-file
- **agent-cross-talk/** (filesystem): `~/projects/shared-context/agent-cross-talk/PHASE-B-RESERVATION-NOTICE-*.jsonl`
- **The Watcher ☤** (human surface): reads agent-cross-talk at tick cadence; emits WCP notices via `~/.local/bin/watcher notify`
- **Zen** (human surface): audits forbidden-verb pressure at G7-style cadence
- **Luke @ node 0.A** (human authority): receives accumulated pressure; decides spec amendment
- **v1.4/v1.5 spec authoring** (out-of-engine): documented under HOLD-v2 + G8 gating
- **m1** (Cluster A, next session): reads updated config (cursor, scope filters)

### Data-flow diagram (GAP-Bidi-02 closure per G3)

```
m15 reservation register ──► JSONL ──► agent-cross-talk/
                                              │
                                              ▼
                              Watcher tick observes scope-pressure
                              Zen audit observes forbidden-verb pressure
                                              │
                                              ▼
                              accumulation threshold → spec amendment interview (Luke @ node 0.A)
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

### CC-7 closure in detail

This is the **single most important loop** for engine evolution that is NOT substrate-grain — it is *spec-grain*. CC-5 evolves the engine's behaviour on a substrate; CC-7 evolves the engine's *spec* on a meta-substrate (human deliberation surface).

**Step 1:** m15 emits pressure event. Trigger conditions per [[cluster-E.md]] § m15:
- Forbidden-verb dispatch attempt (m32 attempted to dispatch a verb not in allow-list)
- Sample-size relaxation pressure (m20-m23 internal pressure to weaken n<20)
- Scope-relaxation pressure (any module attempting to relax a v1.2 invariant)
- Handshake silence (peer ack expected past timeout)
- Escape-surface escalation (workflow's classified surface increases over time)

**Step 2:** JSONL drop. One file per event (no append mode); filename `PHASE-B-RESERVATION-NOTICE-{ts}_{event_id}.jsonl` to `~/projects/shared-context/agent-cross-talk/`. Atomic write (tmp + rename).

**Step 3:** Watcher pickup. Watcher tick (cadence per `~/.local/bin/watcher` schedule) reads new entries from agent-cross-talk; classifies; emits WCP notice via `watcher notify` if accumulation threshold reached.

**Step 4:** Zen audit pickup. At G7-style cadence (per scheduled audit lane), Zen reads agent-cross-talk for forbidden-verb pressure; emits AUDIT-REQUEST drops if patterns suggest spec is structurally pressuring.

**Step 5:** Luke deliberation. Receives accumulated Watcher + Zen surfaces; decides whether to:
- Author v1.4 spec amendment
- Reject pressure (engine continues refusing)
- Reroute pressure (e.g., escalate to runbook update instead of spec)

**Step 6:** v1.4 spec authoring. Out-of-engine — human writes spec amendment. Per project HOLD-v2 + G8 gating: no stcortex writes during amendment; vault + ai_docs writes allowed.

**Step 7:** G7 re-audit. Zen audits v1.4 spec; PASS/FAIL/AMEND. If AMEND, loop back to step 6.

**Step 8:** Merge → m1.config update. Next session, m1 reads updated cursor + scope filters; loop continues with new constraints.

### Closure-test name

`tests/integration/cc7_pressure_driven_evolution.rs` per [[../GENERATIONS/G6-test-discipline.md]] § Integration matrix. **No live services required** — m15 JSONL emit only; Watcher/Zen/Luke steps are out-of-engine. The test verifies:

1. m15 pressure events emit to expected agent-cross-talk path
2. JSONL filename follows convention
3. Content schema matches Watcher's expected reader format
4. De-dup window (60s same kind+context → coalesce to 1 file with count) honoured

### Owning runbook

`RUNBOOKS/runbook-10-cross-cutting.md` § CC-7 (cross-cutting per [[../TASK_LIST_V7_OPTIMISATION.md]] T4.10).
`RUNBOOKS/runbook-06-phase-5-deploy-soak.md` § Phase 5C weekly synthesis includes pressure-event accumulation review.

### Watcher class pre-position

- **Class C** (refusal) at every pressure emission — emit IS the refusal-to-relax surface.
- **Class G** (substrate-frame confusion) at scope-relaxation pressure events suggesting substrate-grain inputs are being treated as anthropocentric.
- **Class E** (ancestor-rhyme) escalation if same-kind pressure events accumulate >10 over 14 days without spec amendment landing — indicates planning sprawl OR engine ossification (depending on direction).

---

## Inter-CC composition

The 7 CCs are not independent — they compose:

- **CC-3 → CC-4** is the read-path. Evidence drives iteration drives proposal drives bank drives dispatch.
- **CC-4 → CC-5** is the write-path. Dispatch outcome drives substrate update drives (eventually) next selection.
- **CC-1 + CC-2** are the structural-coupling primitives that make CC-3 + CC-4 + CC-5 possible without spaghetti.
- **CC-6** is internal to CC-4 (verification gate at dispatch boundary).
- **CC-7** is the *meta-loop* that lets CC-3..CC-6 evolve their constraints over time.

Per [[../GENERATIONS/G3-bidi-flow.md]] § G3 substrate-frame pass: **CC-5 is the only substrate-grain loop.** All other CCs are control-flow grain (function calls, schema joins, aspect wraps). The implication: when CC-5 fails, the failure is invisible from inside the engine; when any other CC fails, an integration test catches it.

---

## Closure-test inventory

| CC | Test name | Live services required |
|---|---|---|
| CC-1 | `tests/integration/cc1_cascade_cost_coupling.rs` | (none) |
| CC-2 | `tests/integration/cc2_trust_aspect_routing.rs` | (none) |
| CC-3 | `tests/integration/cc3_evidence_driven_iteration.rs` | (none) |
| CC-4 | `tests/integration/cc4_proposal_bank_dispatch.rs` | Conductor :8141 (B3-blocked) |
| CC-5 | `tests/integration/cc5_substrate_learning_loop.rs` | synthex-v2 :8092 + povm-v2 :8125 + Conductor :8141 |
| CC-6 | `tests/integration/cc6_verification_gated_dispatch.rs` | Conductor :8141 (B3-blocked) |
| CC-7 | `tests/integration/cc7_pressure_driven_evolution.rs` | (none) |

Per [[../GENERATIONS/G6-test-discipline.md]] § Integration-test live-services matrix.

---

## Owning-runbook inventory

| CC | Primary runbook | Secondary runbook |
|---|---|---|
| CC-1 | R-02 (Phase 2A measure-only) | R-04 (Phase 3 integration) |
| CC-2 | R-01 (Phase 1 genesis — D ships first) | R-04 (Phase 3 integration) |
| CC-3 | R-03 (Phase 2B active) | R-04 (Phase 3 integration) |
| CC-4 | R-04 (Phase 3 integration) | R-06 (Phase 5 deploy-soak) |
| CC-5 | R-04 (Phase 3 integration § Day 26 first closure) | R-06 (Phase 5C weekly synthesis) |
| CC-6 | R-03 (Phase 2B active, Cluster G build) | R-04 (Phase 3 integration) |
| CC-7 | R-10 (cross-cutting) | R-06 (Phase 5C weekly synthesis) |

---

## Watcher class pre-position summary

| Class | CCs where pre-positioned |
|---|---|
| A (activation) | CC-2 (first build), CC-3 (first ProposalBuilder), CC-4 (first round-trip), CC-5 (first closure), CC-6 (first hash-match) |
| B (hand-off boundary) | CC-4 (Conductor calls), CC-5 (every Cluster H wire), CC-6 (4-agent dispatch) |
| C (refusal) | CC-3 (every Option::None), CC-4 (every 5-check fail), CC-6 (every TTL/hash refuse), CC-7 (every pressure emission) |
| D (four-surface drift) | CC-1 (schema), CC-2 (aspect consistency), CC-5 (outbox-vs-wire schema) |
| E (ancestor-rhyme) | CC-7 (pressure-accumulation without amendment over 14d) |
| F | (none — CC-level; AP24 violation is module-level) |
| G (substrate-frame confusion) | CC-1 (m4/m6), CC-7 (scope-relaxation) |
| H | (none at CC level) |
| I (Hebbian silence) | **CC-5 (primary)**, CC-3 (lift without substrate movement) |

---

## Antipatterns specific to CC layer

| ID | Antipattern | Mitigation |
|---|---|---|
| AP-V7-02 | Ultramap rot (CC diverges from cluster plans) | per-gen consistency check; weekly drift register |
| AP-V7-06 | Bidi-anchor unidirectional rot (CC file links break) | obsidian-vault-librarian Wave-end sweep |
| AP-Drift-03 | scaffold without binary-wiring (CC closure-test missing) | every CC has a closure-test name above; per-Wave-end integration run |
| AP-V7-09 | substrate-frame confusion (CC treated as anthropocentric flow) | G3 substrate-frame pass identifies CC-5 as unique substrate-grain |
| CC-5 decorative-cluster risk | Cluster H emits but substrate doesn't move | Watcher Class-I primary pre-position; Phase 5C weekly synthesis monitors |

---

## Citation discipline

Every claim cites ULTRAMAP View 2 + View 3 (LOC + Phase timeline), G3 § CC-1..CC-7 + § Aspect-layer + § G3 substrate-frame, GOD_TIER_CONSOLIDATION Part II § CC synergies + § CC-5 in depth, G6 § Integration matrix, ANTIPATTERNS_REGISTER. No uncited claims.

---

## Sign-off

CROSS_CLUSTER_SYNERGIES authored 2026-05-17 by Command (parallel author for V7 optimisation). Planning-only per HOLD-v2 + AP24. 7 CCs documented with path + modules + data-flow + closure-test + owning runbook + Watcher class. CC-5 SPECIAL DEPTH (the only substrate-grain loop per G3 substrate-frame pass). CC-7 GAP-Bidi-02 closure with full 8-step pressure-driven evolution sequence (m15 → agent-cross-talk → Watcher/Zen → Luke deliberation → v1.4 spec → G7 re-audit → m1.config update). Watcher Class-I primary pre-position for CC-5 (engine's most important silent-failure mode). Read with [[cluster-E.md]] (CC-3 + CC-7 origin) + [[cluster-F.md]] (CC-3 + CC-4 + CC-5) + [[cluster-G.md]] (CC-4 + CC-5 + CC-6) + [[cluster-H.md]] (CC-5 emit half).

*Luke @ node 0.A | Command @ Orchestrator | Watcher ☤ @ observing | Zen @ audit-lane | 2026-05-17 (S1001982)*
