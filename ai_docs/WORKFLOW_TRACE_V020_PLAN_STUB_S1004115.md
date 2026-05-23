# Workflow-Trace v0.2.0 — Plan Stub (S1004115 continuation)

> **Authored:** 2026-05-23 (post-v0.1.0 / M0 ship, post-v0.1.1 hygiene round)
> **Status:** PLAN STUB — scope catalogued, decisions NOT YET LOCKED, execution awaiting node-0.A explicit milestone authorization
> **Back to:** [`WORKFLOW_TRACE_COMPLETION_PLAN_V2_S1004115.md`](WORKFLOW_TRACE_COMPLETION_PLAN_V2_S1004115.md) § 11 · [`PHASE9_SD_RECONCILIATION_S1004115.md`](PHASE9_SD_RECONCILIATION_S1004115.md) § 4 · [`CHANGELOG.md`](../CHANGELOG.md) `[v0.1.0]` § Honest residuals · [CLAUDE.local.md](../CLAUDE.local.md) § v0.1.0 / M0 SHIPPED

---

## What v0.2.0 is

Per Plan v2 § 8 + § 11 + § 15 D1 / D4, v0.2.0 is the **substrate-safety milestone**.
v0.1.0 / M0 certified engine-internal completeness only — every residual the engine
owned was closed, tested, audited. v0.2.0 closes the engine's safety **as a
substrate-facing organ**: substrate-drift detection, substrate-side test fixtures,
substrate-mediated trust, and the substrate-frame contract gaps named in the
Phase 2 audit + Phase 9 reconciliation.

**Per § 15 D4:** v0.2.0 is a real committed follow-on milestone, not a shelf.

**Per § 15 D40:** the cadence is **invocation-only** — v0.2.0 needs its own plan
+ explicit node-0.A "start Phase 1" go, analogous to Plan v2's gate.

This document is a SCOPE CATALOGUE, not a ratified plan. The full Plan v2-style
treatment (10-phase structure, dual-frame gap analysis, 48-decision interview,
substrate-frame self-check) lands when node 0.A authorizes the v0.2.0 cycle.

---

## Scope catalogue — items in v0.2.0 per Plan v2 § 11 + Phase 9 § 4 + Phase 5 A2

### Tier 1 — substrate-as-actor primitives (the v0.2.0 keystone)

| # | Item | Source | Estimated LOC |
|---|------|--------|---------------|
| **V1** | NA-GAP-01 `RefusalToken` authorship-typed enum (`SubstrateAuthored / EngineAuthored / OperatorAuthored / Unavailable`) wired through m9/m32/m13/m40/m41/m42/m33 + ADR `D-S1002127-03` amendment | Plan v2 §11 + Phase 2 audit | ~150–300 + ADR |
| **V2** | NA-GAP-04 substrate back-pressure budget — substrate-emitted contention signal (not engine-timed) replacing the Phase 8 step 3 Frame-A proxy | Plan v2 §11 + Phase 2 audit + § 15 D37 | ~200–400 |
| **V3** | NA-GAP-07 substrate-drift canary `m16` — periodic prober that samples each CC-5 clock (m11/m13/injection-TTL/stcortex-decay/atuin) + asserts agree-to-skew envelope | Plan v2 §11 (ADR `D-S1002127-03`) + § 15 D38 | ~300 |
| **V4** | NA-GAP-08 substrate test fixtures — deterministic local replicas of stcortex, ORAC, synthex-v2, LCM, HABITAT-CONDUCTOR for integration-test isolation | Plan v2 §11 (ADR `D-S1002127-03`) | ~500 |
| **V5** | NA-GAP-10 substrate-mediated trust — substrate participates in verification (not just substrate-confirmable-receipt); pairs naturally with V1 RefusalToken | Plan v2 §11 (ADR `D-S1002127-03`) | ~400 + ADR |

### Tier 2 — engine-side wire-contract changes enabling Tier 1

| # | Item | Source | Estimated LOC |
|---|------|--------|---------------|
| **W1** | `WorkflowProposal::escape_surface: EscapeSurfaceProfile` wire-contract field — un-blocks m33 Security from its M0 Sandboxed default per the Phase 6a invariant lock; cross-binary serde change with the JSONL bridge | Phase 2 audit option (i) + Phase 6a invariant lock | ~150–200 |
| **W2** | OR a `StepToken → EscapeSurfaceProfile` classification table + variant aggregation — Phase 2 audit option (ii), the no-wire-contract-change alternative | Phase 2 audit option (ii) | ~80–150 |
| **W3** | `WorkflowProposal::cost: i64` (or similar) wire-contract field — un-blocks D9 Cost real verifier per the D10 metric `step-count × mutation-weight` | § 15 D9 + Phase 2 wire-contract sizing | ~150–250 |
| **W4** | `CuratedBank::client_ref()` accessor seam — un-blocks D11 Consistency real verifier per § 15 D12 on-demand discipline | § 15 D11 + D12 + T4-API #1 | ~30–60 |

### Tier 3 — real verifier impls (consume Tier 2 wire-contracts)

| # | Item | Source | Estimated LOC |
|---|------|--------|---------------|
| **R1** | m33 Cost real verifier per D10 metric — consumes W3 | § 15 D9 + D10 | ~50–80 |
| **R2** | m33 Consistency real verifier — bank-conflict detection consuming W4 | § 15 D11 | ~100–150 |
| **R3** | m33 Security real verifier — consumes W1 OR W2 (per node-0.A choice at v0.2.0 plan ratification) | Phase 6a + Phase 2 audit | ~50–100 |

### Tier 4 — algorithmic / shape upgrades (Class-C SD drifts)

| # | Item | Source | Estimated LOC |
|---|------|--------|---------------|
| **A1** | SD8 — m21 true Levenshtein over source pattern's steps (replaces M0 closed-form proxy) | § 15 D27 + Phase 9 § 4 | ~100–150 |
| **A2** | SD9 — m22 typed `FeatureVector` newtype wrap (cosmetic) | § 15 D27 + Phase 9 § 4 | ~30–50 |
| **A3** | SD10 — m22 empty-cluster typed-error OR re-seed (spec amendment + impl) | § 15 D27 + Phase 9 § 4 | ~50–80 |
| **A4** | SD11 — m23 12-field proposal shape (ties to W1 + W3) | § 15 D27 + Phase 9 § 4 | ~50–80 |

### Tier 5 — consumer-side behavioural-loop closures (deferred from Phase 5 zen verdict A2)

| # | Item | Source | Estimated LOC |
|---|------|--------|---------------|
| **C1** | ✅ **CLOSED 2026-05-23 v0.1.1 round** — m31 production caller in `wf-dispatch::run` now consumes `proposal.diversity_cluster()` via `diversity_score_from_proposal`. Phase 5 zen A2 is closed. | Phase 5 zen A2 (now closed) | (closed) |

---

## v0.2.0 effort estimate (top-down, NOT a commitment)

| Tier | Items | Combined LOC | Calibre |
|------|-------|--------------|---------|
| 1 (substrate-as-actor) | V1–V5 | ~1450–1700 | KEYSTONE; needs ADR D-S1002127-03 amendment + dual-frame gap analysis |
| 2 (wire-contract) | W1/W2 + W3 + W4 | ~340–510 | cross-binary serde discipline; JSONL fixtures regenerate |
| 3 (real verifiers) | R1+R2+R3 | ~200–330 | consumes Tier 2 |
| 4 (Class-C upgrades) | A1–A4 | ~230–360 | algorithmic + spec amendments |
| **v0.2.0 total** | | **~2220–2900 LOC** + 2 ADRs + dual-frame gap analysis | **~6–10 Claude-days** at god-tier (estimate; not locked) |

---

## What v0.2.0 needs from node 0.A before execution begins

Per § 15 D40 (invocation-only) + the Plan v2 § 14 discipline ("the single remaining
gate: Luke @ node 0.A gives the explicit go for Phase 1"):

1. **Explicit v0.2.0 plan-authoring authorization** ("start v0.2.0 planning") —
   triggers a full Plan v2-style cycle (dual-frame gap analysis + 12-round
   decision interview + 4-surface plan persistence).
2. **Tier-1 vs Tier-2 ordering decision** — whether to land wire-contract
   changes (Tier 2) before substrate primitives (Tier 1), OR after, OR in
   parallel. v0.1.0's Phase 6a Security stub + Phase 6f WireEvent receipts
   already shipped the substrate-confirmable pattern; Tier 2 W1/W2 is the
   missing piece for Security to fire live.
3. **Phase 6a Security option choice** — node 0.A picks W1 (wire field) vs W2
   (classification table) per the Phase 2 audit option ladder.
4. **D9 Cost wire choice** — W3 (real wire) vs ship stub-forever (M0 + v0.2.0).
5. **D11 Consistency activation** — V0.2.0 wires it or defers further.

---

## Persistence (when this stub is ratified into a plan)

Standard 4-surface per Plan v2 § 13:
- `ai_docs/WORKFLOW_TRACE_V020_PLAN_S1004XXX.md` (canonical, post-authoring)
- Obsidian vault mirror `the-workflow-engine-vault/Workflow-Trace v0.2.0 Plan S1004XXX.md`
- stcortex ns `workflow_trace_v020_s1004XXX` — meta memory + bidi pathway to
  `workflow_trace_completion_s1004115`
- CLAUDE.local.md anchor — a "v0.2.0 IN FLIGHT" block at top, supersedes the
  current "v0.1.0 / M0 SHIPPED" block once that work fires

---

## Bidirectional anchor

> Back to: [`WORKFLOW_TRACE_COMPLETION_PLAN_V2_S1004115.md`](WORKFLOW_TRACE_COMPLETION_PLAN_V2_S1004115.md) §11
> Back to: [`PHASE9_SD_RECONCILIATION_S1004115.md`](PHASE9_SD_RECONCILIATION_S1004115.md) §4
> Back to: [`CHANGELOG.md`](../CHANGELOG.md) `[v0.1.0]` § Honest residuals
> Back to: [`CLAUDE.local.md`](../CLAUDE.local.md) § v0.1.0 / M0 SHIPPED

*v0.2.0 plan stub — scope catalogue only; ratification requires node-0.A go per § 15 D40.*
