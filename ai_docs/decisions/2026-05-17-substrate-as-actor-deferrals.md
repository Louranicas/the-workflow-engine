---
title: ADR — substrate-as-actor v0.2.0 deferrals (NA-GAP-07 / NA-GAP-08 / NA-GAP-10)
date: 2026-05-17
status: DEFERRED (target v0.2.0)
adr_id: D-S1002127-03
authors: Command (Tab 1 Orchestrator top-left), na-gap-analyst (Wave 3 dispatch), na-gap-analyst follow-up (Wave 4.B)
session: S1002127
authorising_session: S1002127 Luke "as per proposal" override (NA-GAP-01..11)
audit_lane: Zen G7 (folded into v3 AUDIT-REQUEST per D-B6 AMEND-loop)
gates_required: none for this ADR (v0.2.0 work item registration)
supersedes: none
companion_adrs:
  - 2026-05-17-m42-stcortex-only-pivot.md (D-S1001982-01)
  - 2026-05-17-g8-stcortex-persistence-plan.md (D-S1002127-01)
  - 2026-05-17-escape-surface-cardinality-7-privilege-escalation.md (D-S1002127-02)
addresses: [NA-GAP-07 (partially — substrate-drift.md covers the cross-cutting half; m16 module deferred), NA-GAP-08 (full defer), NA-GAP-10 (full defer)]
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

> **Back to:** [`../../CLAUDE.md`](../../CLAUDE.md) · [`../../CLAUDE.local.md`](../../CLAUDE.local.md) · [`../../GATE_STATE.md`](../../GATE_STATE.md) · [`../NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md) · [`../../ai_specs/INDEX.md`](../../ai_specs/INDEX.md)

*Filed 2026-05-17 (S1002127 · Wave 4.B closeout) · Command · planning-only · HOLD-v2 compliant · v0.2.0 work-item registration.*
