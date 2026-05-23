---
title: ADR — substrate-as-actor v0.2.0 deferrals (NA-GAP-07 / NA-GAP-08 / NA-GAP-10) — AMENDED 2026-05-23 to add NA-GAP-01 / NA-GAP-04 / NA-GAP-06-drain
date: 2026-05-17 (original) · 2026-05-23 (Amendment 1, S1004377)
status: NOW-ACTIVE (v0.2.0 PLAN v2 RATIFIED S1004377; execution awaiting Luke "start Phase 1")
adr_id: D-S1002127-03
amendment_history:
  - 1: 2026-05-23 (S1004377, Plan v2 v0.2.0 §3 Phase 1 step 2) — adds NA-GAP-01 (V1 RefusalToken) + NA-GAP-04 (V2 substrate back-pressure) + NA-GAP-06-drain (C1 m13 outbox drain half) to the v0.2.0 active work-item list per Phase 2 audit (S1004115) §2 recommendations + v0.2.0 Plan v2 §2.5 carry-overs
authors: Command (Tab 1 Orchestrator top-left), na-gap-analyst (Wave 3 dispatch), na-gap-analyst follow-up (Wave 4.B), Claude @ cortex (Amendment 1 S1004377)
session: S1002127 (original) · S1004377 (Amendment 1)
authorising_session: S1002127 Luke "as per proposal" override (NA-GAP-01..11) · S1004377 Luke "begin V2" / Phase 1 go
audit_lane: Zen G7 (folded into v3 AUDIT-REQUEST per D-B6 AMEND-loop) · Amendment 1 in-session zen agent per D26
gates_required: none for this ADR (v0.2.0 work item registration)
supersedes: none
companion_adrs:
  - 2026-05-17-m42-stcortex-only-pivot.md (D-S1001982-01)
  - 2026-05-17-g8-stcortex-persistence-plan.md (D-S1002127-01)
  - 2026-05-17-escape-surface-cardinality-7-privilege-escalation.md (D-S1002127-02)
  - 2026-05-23-refusal-token-authorship-typing.md (D-S1004XXX-04 NEW — V1 design spec; companion to this Amendment 1)
addresses_original: [NA-GAP-07 (partially — substrate-drift.md covers the cross-cutting half; m16 module deferred), NA-GAP-08 (full defer), NA-GAP-10 (full defer)]
addresses_amendment_1: [NA-GAP-01 (V1 RefusalToken authorship-typing — full defer to v0.2.0 per Phase 2 audit S1004115 §2 NA-GAP-01 row recommendation), NA-GAP-04 (V2 substrate back-pressure budget — full defer per Phase 2 audit §2 NA-GAP-04 row), NA-GAP-06-drain (C1 m13 outbox drain consumer — full defer per Phase 2 audit §2 NA-GAP-06 partial row + Phase 9 §4 #9; the write half shipped at M0)]
location_rationale: |
  Filed at ai_docs/decisions/ (NOT ai_docs/optimisation-v7/decisions/) because this ADR
  registers v0.2.0 follow-on work driven by NA gap analysis (substrate-as-actor frame),
  distinct from the v0.1.0 optimisation-v7 author wave. The optimisation-v7/decisions/
  directory holds three V7-authored ADRs (m42 pivot, G8 persistence, escape-surface 7);
  the top-level decisions/ directory holds NA-remediation ADRs going forward.
---

# ADR — Substrate-as-Actor v0.2.0 Deferrals (NA-GAP-07 / 08 / 10)

> **Back to:** [`../../CLAUDE.md`](../../CLAUDE.md) · [`../../CLAUDE.local.md`](../../CLAUDE.local.md) · [`../../GATE_STATE.md`](../../GATE_STATE.md) · [`../NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md) · [`../../ai_specs/INDEX.md`](../../ai_specs/INDEX.md) · companion [`../optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md`](../optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md) · [`../optimisation-v7/decisions/2026-05-17-g8-stcortex-persistence-plan.md`](../optimisation-v7/decisions/2026-05-17-g8-stcortex-persistence-plan.md) · [`../optimisation-v7/decisions/2026-05-17-escape-surface-cardinality-7-privilege-escalation.md`](../optimisation-v7/decisions/2026-05-17-escape-surface-cardinality-7-privilege-escalation.md)
>
> **Companion specs:** [`../../ai_specs/cross-cutting/substrate-drift.md`](../../ai_specs/cross-cutting/substrate-drift.md) · [`../../ai_specs/substrates/`](../../ai_specs/substrates/) (8 dossiers) · [`../../ai_specs/substrate-couplings/`](../../ai_specs/substrate-couplings/) (NA-GAP-03/09 closure)

---

## § 0 — Context

The Wave 3 na-gap-analyst dispatch surfaced 11 NA gaps under the substrates-as-primary frame ([`../NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md)). § 5 of that analysis recommended five remediation actions; **four are addressed inside this scaffold** (Wave 4.A + Wave 4.B), **three are intentionally deferred to v0.2.0** to keep the v0.1.0 scope deliverable.

Wave 4.B closeout addressed:

1. ✅ NA-GAP-01 / 04 / 05 / 10 (partial) — `ai_specs/substrates/` (8 substrate dossiers, including operator-as-substrate per NA-GAP-05 and watcher as separate persona-actor)
2. ✅ NA-GAP-02 / 05 (cross-ref) / 11 — `cross-cutting/refusal-taxonomy.md` + `ERROR_TAXONOMY.md` amendment introducing `RefusalToken`
3. ✅ NA-GAP-03 / 09 — `ai_specs/substrate-couplings/` (3 files: CC-5, CC-4, CC-7 decomposed)
4. ✅ NA-GAP-04 / 06 — `m42_stcortex_emit.md § 5.1 outbox-policy` amendment + `BENCHMARK_SPEC.md § substrate-side load benchmarks` amendment
5. ⏳ NA-GAP-07 / 08 / 10 (full) — **this ADR** defers to v0.2.0

NA-GAP-07's cross-cutting half is closed in [`../../ai_specs/cross-cutting/substrate-drift.md`](../../ai_specs/cross-cutting/substrate-drift.md) (canary contract + `SubstrateDriftDetected` event); the m16 module proposal is the v0.2.0 work.

---

## § 1 — What is deferred

### § 1.a — NA-GAP-07 (partial): `m16_substrate_drift_canary` as a new module

**Status:** cross-cutting axis closed (substrate-drift.md cross-cutting spec live); module proposal **deferred to v0.2.0**.

The Wave 3 NA recommendation proposed adding `m16_substrate_drift_canary` as a new module in Cluster E (or a new Cluster I) emitting `PHASE-B-RESERVATION-NOTICE` events on detected semantic drift across the 7 substrates. The cross-cutting spec at [`substrate-drift.md`](../../ai_specs/cross-cutting/substrate-drift.md) captures the canary contract design but does NOT add a module to the 26-module locked architecture.

**Why deferred:**

- v0.1.0 module count is **locked at 26** (per [v1.3-amendment](../GENESIS_PROMPT_V1_3.md) OI-3 resolution); adding m16 would unlock the count and trigger a Genesis Prompt v1.4 + re-audit cascade.
- The substrate-drift detection function can be **distributed across existing modules** for v0.1.0: m9 (namespace-guard observes refusal patterns), m13 (stcortex reads observe semantic-drift), m32 (5-check observes Conductor enforcement-state drift), m42 (Hebbian-coordinator observes pathway-weight drift). The cross-cutting spec defines the per-module canary contract participation; the centralised collector / event-bus / register table is what gets deferred.
- The **operator can spot-check** the per-substrate canaries during incident response, which is the current CR-2 baseline; m16 would automate this for v0.2.0.

**v0.2.0 work item:** add m16_substrate_drift_canary as a new module (Cluster E or new Cluster I). Estimated ~250-350 LOC + 40 tests. Triggers Genesis Prompt v1.4 module-count amendment + Zen G7 re-audit.

**v0.1.0 compensating control:** every substrate dossier in [`../../ai_specs/substrates/`](../../ai_specs/substrates/) carries a `drift_indicators` YAML field + `canary_kinds` table. Operator-facing m12 reports surface canary-mismatch hints. Until m16 ships, drift detection is operator-triggered (incident response) + substrate-dossier-documented (so it's at minimum knowable).

### § 1.b — NA-GAP-08 (full): substrate fixture suite

**Status:** **deferred to v0.2.0** — no engine-side test infrastructure changes.

The Wave 3 NA recommendation proposed a `tests/substrate_fixtures/` directory with at minimum:

- `cr2_inflation_fixture` (stcortex returns pre-CR-2 magnitude)
- `refuse_write_no_consumer_fixture`
- `hyphen_slug_reducer_fixture`
- `conductor_enforcement_flag_off_fixture`
- `atuin_wal_contention_fixture`

These would let the engine assert against **known-drifted substrate state** without requiring live substrates in PR-CI.

**Why deferred:**

- v0.1.0 test budget is **locked at 1,594** (per G6 latest matrix) across 26 modules; adding substrate-drift fixtures would expand the budget meaningfully (~80-150 fixture tests).
- Fixtures require *implementing* substrate-stub variants (a mock stcortex reducer returning pre-CR-2 magnitude is non-trivial code) — **HOLD-v2 forbids code authoring pre-G9**. Even post-G9, the v0.1.0 implementation plan is fully scoped without substrate fixtures.
- Engine-side tests for the same scenarios exist in the per-module specs (mock m13 returning `RefuseWriteNoConsumer` is a unit test in m42; CR-2 inflation handling is documented in [`substrate-drift.md`](../../ai_specs/cross-cutting/substrate-drift.md) but not yet test-coded). The fixture suite is **richer** than the engine-side tests because it stresses the cross-module response chain; v0.1.0's per-module tests are sufficient acceptance.

**v0.2.0 work item:** scaffold `tests/substrate_fixtures/` with the five named fixtures. Estimated ~600-800 LOC + 80-150 tests. Coordinates with m16 (above) since both target substrate-drift surface.

**v0.1.0 compensating control:** per-module integration tests gated `#[ignore = "requires <substrate>"]` for the same scenarios when live substrates available; CI nightly runs these against the live habitat. Cross-cutting failure-mode catalogue is in [`../../ANTIPATTERNS.md`](../../ANTIPATTERNS.md) AP-V7-13 + the substrate dossiers.

### § 1.c — NA-GAP-10 (full): substrate-mediated trust

**Status:** **deferred to v0.2.0** — no architectural change.

The Wave 3 NA recommendation observed that Cluster D's "trust" is engine-internal aspect (POVM build-prereq, namespace guard, Ember CI gate, fitness-weighted decay) — none of the 4 modules is a substrate-level trust mechanism (e.g. stcortex-side consumer-trust score, conductor-side dispatch-budget per workflow, atuin-side read-quota).

**Why deferred:**

- The pattern (substrate-mediated reputation rather than engine-internal aspect) is a **structural shift** that touches multiple substrate dossiers + ADRs in three habitat services (stcortex, conductor, atuin). v0.1.0 scope is workflow-trace engine-only; cross-habitat substrate changes are not in scope.
- The current Cluster D scope (m8 / m9 / m10 / m11) is **sufficient for v0.1.0** — m9 namespace-guard alone provides the AP30 enforcement that mostly preempts the trust gap (workflow-trace cannot write anywhere except its own namespace; substrate-side reputation matters less when blast radius is structurally bounded).
- The **operator-as-substrate angle of trust** is captured in [`../../ai_specs/substrate-couplings/CC-7-decomposed.md`](../../ai_specs/substrate-couplings/CC-7-decomposed.md) (operator consent fatigue is a trust signal); this gives v0.1.0 the operator-trust component without requiring substrate-side changes.

**v0.2.0 work item:** propose substrate-mediated trust as a cross-habitat ADR. Touches stcortex consumer-trust schema (substrate-side change), conductor dispatch-budget table (substrate-side change), atuin read-quota daemon flag (substrate-side change). Coordinates with habitat-wide substrate-as-actor refactor.

**v0.1.0 compensating control:** m9 namespace-guard + m30 acceptance signature + m32 5-check + m10 Ember CI gate cover the engine-internal trust aspects; operator-as-substrate trust signals surface in m12 reports.

---

## § 2 — Why these deferrals are not Frame collapse

A reasonable critique of this ADR: "you're deferring exactly the items that would push the engine into substrate-actor maturity — you're collapsing back to the anthropocentric frame." The counter:

1. The Wave 4.B closeout that ships in this scaffold (substrate dossiers + RefusalToken + substrate-couplings + outbox-policy + substrate-side benches) IS Frame-A authored. The frame is held; what's deferred is the **automation** of the Frame-A insights, not the insights themselves.
2. The `drift_indicators` / `canary_kinds` fields in every substrate dossier give operators the data to spot-check; v0.2.0 m16 makes this automatic, but the cognitive surface is already substrate-as-actor.
3. NA-GAP-10 (substrate-mediated trust) genuinely IS a cross-habitat refactor — deferring it does not weaken workflow-trace's own Frame-A authoring. Workflow-trace will participate when the habitat-wide refactor happens, not lead it solo.
4. The four items absorbed into Wave 4.B (NA-GAP-01-06, 09, 11) are the **structural** Frame-A changes; the three deferred (NA-GAP-07, 08, 10) are **operational** automations of Frame-A insights. This is a defensible split.

The Wave 3 NA recommendation explicitly authorised this split:

> 5. DEFER NA-GAP-07 (m16_substrate_drift_canary as new module) + NA-GAP-08 (substrate fixture suite) + NA-GAP-10 (substrate-mediated trust) to v0.2.0 ADRs. File as `ai_docs/decisions/2026-05-17-substrate-as-actor-deferrals.md`. ~30 min.

This ADR is the requested file.

---

## § 3 — v0.2.0 work-item registry

| # | Item | Estimated effort | Triggers | Coordinates with |
|---|---|---|---|---|
| **W1** | m16_substrate_drift_canary module | ~250-350 LOC, ~40 tests | Genesis Prompt v1.4 module-count amendment; Zen G7 re-audit | substrate-drift.md spec; per-substrate canary participation in 8 dossiers |
| **W2** | tests/substrate_fixtures/ suite | ~600-800 LOC, ~80-150 fixture tests | TEST_STRATEGY budget bump; CI config update | W1 (shares substrate-stub infrastructure) |
| **W3** | Substrate-mediated trust ADR | ~5-10 hour authoring; cross-habitat coordination | stcortex consumer-trust schema (substrate); Conductor dispatch-budget (substrate); atuin read-quota (substrate) | Cluster D module re-scope; operator-as-substrate trust signals (already in CC-7-decomposed) |

v0.2.0 entry sequencing: **W1 first** (unlocks m16-emitted events for W2 fixtures to assert against), **W2 next** (uses W1's event surface as test target), **W3 last** (the cross-habitat coordination is the highest-coordination-cost item; sequenced after own-engine maturity).

Total v0.2.0 estimated effort: ~25-40 hours of authoring + audit, plus cross-habitat coordination time for W3.

---

## § 4 — Decision register fields

- **Decision ID:** D-S1002127-03
- **Status:** DEFERRED (target v0.2.0)
- **Decision-makers:** Luke @ node 0.A (S1002127 "as per proposal" authorisation); Command authoring; Zen audit-lane verdict pending (folded into v3 AUDIT-REQUEST per D-B6 AMEND-loop)
- **Affected surfaces:**
  - `ai_specs/cross-cutting/substrate-drift.md` (cross-cutting half of NA-GAP-07 closed Wave 4.B)
  - `ai_specs/substrates/*.md` (8 dossiers — all carry `drift_indicators` field anticipating m16)
  - `ai_specs/substrate-couplings/*.md` (3 decomposition files reference deferred receipts)
  - `ai_specs/BENCHMARK_SPEC.md` (substrate-side benches section references W2 fixture deferral)
  - `ai_specs/ERROR_TAXONOMY.md` (RefusalToken section references W3 substrate-mediated trust as the v0.2.0 maturity step)
  - `ai_specs/INDEX.md` (registers this ADR + v0.2.0 work items)
  - workspace `~/claude-code-workspace/CLAUDE.md` (no edit — workspace charter forbids Command from amending workspace-root CLAUDE.md per project-local rules)
- **Reversal cost:** low — deferral is reversible by Luke promotion to v0.1.0 in any future session; no irrecoverable structural change incurred by deferral.

---

## § 5 — Acceptance discipline

This ADR is accepted when:

1. ✅ All three NA-GAP items (07, 08, 10) have explicit v0.2.0 work-item registration (above).
2. ✅ The v0.1.0 compensating controls are documented (§ 1.a-c).
3. ✅ Zen G7 audit folds this ADR into the v3 AUDIT-REQUEST cycle (per D-B6 AMEND-loop) — verdict separable from approval (REFUSE-with-amend allowed; no blocker on workspace-wide gates).
4. ✅ This ADR is registered in [`../../ai_specs/INDEX.md`](../../ai_specs/INDEX.md) v0.1.0 spec register.
5. ⏳ Wave 4.B closeout commit lands the four absorbed items (substrate dossiers complete; substrate-couplings live; refusal-taxonomy live; substrate-drift live; outbox-policy amended; substrate-side benches amended; this ADR filed; CHANGELOG entry + register update).

---

## § 6 — Cross-reference to companion ADRs

- [`../optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md`](../optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md) — m42 POVM decoupling (D-S1001982-01); substrate-drift learning derived from CR-2 POVM incident
- [`../optimisation-v7/decisions/2026-05-17-g8-stcortex-persistence-plan.md`](../optimisation-v7/decisions/2026-05-17-g8-stcortex-persistence-plan.md) — G8 persistence (D-S1002127-01); reserves stcortex namespace shape for v0.2.0 m16 emit-target
- [`../optimisation-v7/decisions/2026-05-17-escape-surface-cardinality-7-privilege-escalation.md`](../optimisation-v7/decisions/2026-05-17-escape-surface-cardinality-7-privilege-escalation.md) — EscapeSurfaceProfile 7-variant (D-S1002127-02); 7-variant ordinal stability across v0.1.0 → v0.2.0

---

> **Back to:** [`../../CLAUDE.md`](../../CLAUDE.md) · [`../../CLAUDE.local.md`](../../CLAUDE.local.md) · [`../../GATE_STATE.md`](../../GATE_STATE.md) · [`../NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md) · [`../../ai_specs/INDEX.md`](../../ai_specs/INDEX.md) · Amendment 1 cross-refs [`../WORKFLOW_TRACE_V020_PLAN_V2_S1004377.md`](../WORKFLOW_TRACE_V020_PLAN_V2_S1004377.md) · [`./2026-05-23-refusal-token-authorship-typing.md`](./2026-05-23-refusal-token-authorship-typing.md)

*Filed 2026-05-17 (S1002127 · Wave 4.B closeout) · Command · planning-only · HOLD-v2 compliant · v0.2.0 work-item registration.*

---

## § 7 — AMENDMENT 1 (2026-05-23 · S1004377 · Plan v2 Phase 1 step 2)

Per Plan v2 §3 Phase 1 step 2 + the Phase 2 audit (S1004115) §2 recommendation that the project record "8/11 NA gaps closed" be reframed as "3/8 fully + 2/8 partial naturally completed by Phase 6e/6f + 3/8 explicitly deferred or amended": this amendment registers **NA-GAP-01 (V1 RefusalToken)**, **NA-GAP-04 (V2 substrate back-pressure)**, and **NA-GAP-06-drain (C1 m13 outbox drain consumer)** as **now-active v0.2.0 work-items** alongside the original NA-GAP-07/08/10 deferrals.

### § 7.1 — Work-items added to v0.2.0 active list

| # | Item | Source frame | Plan v2 v0.2.0 anchor | Estimated effort |
|---|------|--------------|----------------------|------------------|
| **W4 (Amendment 1)** | NA-GAP-01 V1 `RefusalToken` authorship-typed enum (`SubstrateAuthored / EngineAuthored / OperatorAuthored / Unavailable(EngineImagined \| SubstrateUnreachable \| SubstrateAuthored)`) wired through m9/m32/m13/m40/m41/m42/m33 — replaces flat `RefusalReason` enum at `src/m32_dispatcher/mod.rs:228` | NA frame Phase 2 audit §2 NA-GAP-01 row | v2 §2.1 Tier 1 V1 + Phase 5 co-land with W1 (per C-2) + Phase 7 call-site threading + companion ADR D-S1004XXX-04 | ~150-300 LOC + ~80-150 tests |
| **W5 (Amendment 1)** | NA-GAP-04 V2 substrate back-pressure budget — per-substrate `SubstrateBackPressureMode` enum (`Push / Pull / Unavailable`) per NA-8 reshape | Substrate frame Phase 2 audit §2 NA-GAP-04 row + v0.2.0 NA-8 | v2 §2.1 Tier 1 V2 + Phase 8 | ~200-400 LOC |
| **W6 (Amendment 1)** | NA-GAP-06-drain C1 m13 outbox drain consumer (write half shipped at M0 via `m13_stcortex_writer/mod.rs:307–369` `outbox_path` + `outbox_lock`; drain absent) | Substrate frame Phase 2 audit §2 NA-GAP-06 row | v2 §2.5 C1 + Phase 3 staging + Phase 5 wires consumer via V1 RefusalToken-typed events | ~80-120 LOC (Phase 3 staging) + ~50-100 LOC (Phase 5 consumer wire) |

### § 7.2 — Why the amendment now

The Phase 2 audit (S1004115) §2 surfaced that the v0.1.0 closeout's "8 of 11 NA gaps closed" claim was not faithful to the shipped tree:

- **3/8 genuinely code-backed:** NA-GAP-02, NA-GAP-03, NA-GAP-05.
- **2/8 partial:** NA-GAP-06 (write done, drain absent → C1 in this amendment), NA-GAP-11 (type shipped, trait seam closed in Phase 6e).
- **3/8 spec-only — claimed absorbed but had zero implementation:** NA-GAP-01 (RefusalToken — now V1 / W4 in this amendment), NA-GAP-04 (substrate back-pressure — now V2 / W5), NA-GAP-09 (CC-5 substrate-confirmable receipt — folded into M0 Phase 6f shipped `WorkflowRefused` + `RefusalReceipt` per CHANGELOG [v0.1.0] § Added).

NA-GAP-09 was closed in M0 via Phase 6f. NA-GAP-01 + NA-GAP-04 + NA-GAP-06-drain remained spec-only at M0; this amendment registers them as **now-active v0.2.0 work-items** rather than letting "8/11 closed" persist as a silent over-claim. NA-GAP-07 / 08 / 10 deferrals from §§ 1.a-c are unchanged.

### § 7.3 — Cascade across deferral-language doc surfaces (per C-8 step 2.5)

Per the v0.2.0 Plan v2 conventional gap analysis C-8 finding, this amendment is a *language change* with downstream consequences. Every doc that references this ADR's deferral language must either update to "now-active v0.2.0 work-item" or add a cross-reference to this Amendment 1:

| Doc | Language update needed | Cascade disposition |
|-----|------------------------|---------------------|
| `the-workflow-engine/CHANGELOG.md` `[v0.1.0]` § "Honest residuals — v0.2.0 candidates" | NA-GAP-01 / NA-GAP-04 / NA-GAP-06 still appropriately listed as v0.2.0 candidates (honest residual = v0.2.0 active work-item from M0 vantage) | NO EDIT — language was already "honest residuals" not "deferred indefinitely"; this Amendment 1 simply records that the candidates have been ratified into v0.2.0 active scope |
| `the-workflow-engine/ai_specs/substrate-couplings/INDEX.md` and per-CC decomposition files (CC-5 / CC-4 / CC-7) | Reference to "this ADR defers" → "this ADR Amendment 1 (2026-05-23) registers as v0.2.0 active" | EDIT in Phase 2 substrate-enumeration audit (covers same surface) |
| `the-workflow-engine/ai_specs/cross-cutting/refusal-taxonomy.md` | Reference to v0.2.0 RefusalToken as "future work" → "v0.2.0 V1 active per ADR D-S1002127-03 Amendment 1 + companion D-S1004XXX-04" | EDIT in Phase 5 V1 co-land (covers same surface) |
| `the-workflow-engine/ai_specs/cross-cutting/substrate-drift.md` | NA-GAP-07 m16 module remains deferred per § 1.a (unchanged) | NO EDIT — original deferral still active per DX-V3 = own module Phase 9 |
| `the-workflow-engine/ai_specs/BENCHMARK_SPEC.md` § "Substrate-side load benchmarks" | NA-GAP-04 reference: "v0.2.0 W2 candidate" → "v0.2.0 V2 active per ADR D-S1002127-03 Amendment 1" | EDIT in Phase 8 V2 implementation (covers same surface) |
| `the-workflow-engine/ai_specs/modules/cluster-H/m42_stcortex_emit.md` § 5.1 | NA-GAP-06-drain "fire-and-forget contract — outbox consumer can choose..." → cross-reference Phase 3 C1 drain skeleton + Phase 5 V1-typed consumer wire | EDIT in Phase 3 C1 staging (covers same surface) |

**Cascade discipline:** rather than landing a single Phase-1 cascade-sweep commit touching 4+ spec docs, the language updates are co-located with the implementation phases that actually deliver the new substance. This is honest-coupling per CLAUDE.md "Don't add features beyond what the task requires" — Phase 1's job is *registering* the language change; per-phase updates are where the language change materialises.

### § 7.4 — Reversal cost

Low. If a future v0.2.0 cycle decision (e.g., DX-DAW-1 swap to Tier-1-first) re-shapes V1/V2/C1, this amendment is amended-in-place again; the work-item registry is the source of truth, not the language placement.

### § 7.5 — Acceptance discipline (Amendment 1)

This Amendment 1 is accepted when:
1. ✅ NA-GAP-01 / NA-GAP-04 / NA-GAP-06-drain have explicit v0.2.0 work-item rows in § 7.1 above.
2. ✅ Companion ADR D-S1004XXX-04 (RefusalToken authorship-typing) is filed at `./2026-05-23-refusal-token-authorship-typing.md`.
3. ✅ Cascade discipline named in § 7.3 — per-phase responsibility, not single Phase-1 sweep.
4. ✅ Cross-references in v0.2.0 Plan v2 §2.5 + §2.6 + Phase 1 step 2 already point at this ADR amendment.
5. ⏳ Phase 1 commit lands carrying this amendment + companion ADR D-S1004XXX-04 + Phase 1 done-evidence per D43.

*Amendment 1 authored 2026-05-23 (S1004377) · Claude @ cortex · Plan v2 Phase 1 step 2 · workflow-trace v0.2.0 execution begin · in-session zen agent audit per D26.*
