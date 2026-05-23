---
title: ADR — V5 substrate-mediated trust cross-habitat coordination
date: 2026-05-24 (S1004377)
status: ACTIVE (v0.2.0 V5 engine-side; substrate-side post-v0.2.0)
adr_id: D-S1004XXX-05
authors: Claude @ cortex (orchestrator, S1004377)
session: S1004377
authorising_session: Luke @ node 0.A "begin V2" S1004377 + Plan v2 §15 DX-V5 locked-full-cross-habitat
audit_lane: in-session zen agent per D26
gates_required: Plan v2 §3 Phase 11 (this commit lands engine-side); cross-habitat ADR review post-v0.2.0
supersedes: none
companion_adrs:
  - 2026-05-17-substrate-as-actor-deferrals.md (D-S1002127-03) Amendment 1 registers V5 as v0.2.0 active
  - 2026-05-23-refusal-token-authorship-typing.md (D-S1004XXX-04) V1 RefusalToken envelope V5 consumes for NA-5 audit-distinguishability
addresses: [NA-GAP-10 (substrate-mediated trust full closure — engine-side this commit; substrate-side post-v0.2.0), NA-5 (audit-distinguishability via `substrate_participation_status` accessor + `refusal_for_unavailable` routing)]
---

# ADR — V5 Substrate-Mediated Trust Cross-Habitat Coordination

> **Back to:** [`../../CLAUDE.md`](../../CLAUDE.md) · [`../../CLAUDE.local.md`](../../CLAUDE.local.md) · [`../WORKFLOW_TRACE_V020_PLAN_V2_S1004377.md`](../WORKFLOW_TRACE_V020_PLAN_V2_S1004377.md) §3 Phase 11 · companion [`./2026-05-17-substrate-as-actor-deferrals.md`](./2026-05-17-substrate-as-actor-deferrals.md) Amendment 1 + [`./2026-05-23-refusal-token-authorship-typing.md`](./2026-05-23-refusal-token-authorship-typing.md)

---

## § 0 — Context

Per ADR D-S1002127-03 Amendment 1 + Plan v2 §3 Phase 11 + §15 DX-V5 (locked = full cross-habitat) + DX-V5.b (3-variant `Unavailable` sub-tag per NA-5), v0.2.0 V5 substrate-mediated trust is a two-half work item:

| Half | Owner | Shipped where |
|------|-------|---------------|
| Engine-side (consumer) | workflow-trace | **THIS COMMIT — v0.2.0 Phase 11** |
| Substrate-side (producer) | 5 substrate repos (see §1) | **post-v0.2.0** per §11 consent gradient |

This ADR registers the substrate-side coordination work items + the per-substrate consent-gradient estimates from Plan v2 §11. The engine-side primitive ships in this Phase 11 commit (src/substrate_trust/mod.rs); the substrate-side schemas are post-v0.2.0 per-substrate work.

## § 1 — Substrate-side work items (post-v0.2.0)

Each substrate hosts a piece of the V5 cross-habitat trust contract. Per Plan v2 §11 per-substrate consent gradient:

| # | Substrate | Substrate-side primitive | Acceptance probability | Estimated calendar latency | Engine-side fallback if refused |
|---|-----------|--------------------------|------------------------|-----------------------------|---------------------------------|
| **CH-1** | stcortex | consumer-trust score schema (per-namespace 0.0-1.0 score table; updated by stcortex itself based on consumer behaviour over time) | HIGH (single-author repo, active substrate-mastery) | 1-2 weeks | `SubstrateTrust.get(Stcortex).status = NotShipped` → `RefusalToken::Unavailable(EngineImagined)`; engine treats trust queries as substrate-coupling gap |
| **CH-2** | HABITAT-CONDUCTOR | dispatch-budget table (per-workflow-id remaining-dispatch-count) | HIGH (workflow-trace-adjacent; B3/OP-1 already pending Luke) | 1-2 weeks (after OP-1 Conductor bring-up) | `SubstrateTrust.get(HabitatConductor).status = NotShipped` → engine defaults to "trust everything"; m32 enforcement-state assertion still fires per NA-4 |
| **CH-3** | atuin | read-quota daemon flag | UNKNOWN (third-party upstream; community-driven) | indeterminate; possibly never | `SubstrateTrust.get(Atuin).status = NotShipped` indefinitely; engine throttles via V2 Pull-mode probe-and-throttle |
| **CH-4** | ORAC | reputation hooks (per-emit-pattern reputation 0.0-1.0) | MED (workflow-trace-adjacent; 40 modules; RALPH integration negotiation) | 2-4 weeks | `SubstrateTrust.get(SynthexV2).status = NotShipped`; engine consumes via existing PostToolUse hooks only |
| **CH-5** | synthex-v2 | r13-state-aware verifier weighting (per-verifier weight modulated by Watcher r13 state) | HIGH (workflow-trace integration via m42 + V5 deps; Watcher coordination active) | 1-2 weeks | `RefusalToken::SubstrateAuthored SynthexV2` from synthex-v2 R13-bypass chain; no weighting until ship |

Total estimated v0.2.0 → post-v0.2.0 cross-habitat coordination effort: **~3-6 calendar-months** if all probabilities resolve favourably; longer if atuin upstream refuses (the unit of progress is *substrate consent*, not engine effort).

## § 2 — Engine-side primitive (this commit, v0.2.0 Phase 11)

`src/substrate_trust/mod.rs` ships:

- `SubstrateParticipationStatus { NotShipped, Shipping, Live }` lifecycle enum (default = NotShipped per v0.2.0 ship; operators flip per substrate as CH-1..CH-5 ship per §1).
- `TrustValue { Score(f64), Flag(bool), BudgetRemaining(i64), Unavailable }` typed per-substrate trust value.
- `TrustEntry { status, value }` per-substrate composite + convenience constructors (`not_shipped`, `live_score`, `live_flag`, `live_budget`).
- `SubstrateTrust` accumulator with `get(substrate)` + `set(substrate, entry)` + `substrate_participation_status(substrate)` + the NA-5 primary checks `is_substrate_imagined_for(substrate) -> bool` + the routing helper `refusal_for_unavailable(substrate, reason) -> RefusalToken` that selects the correct V1 `Unavailable` sub-tag per status.

The engine-side accumulator is a typed contract: post-v0.2.0 each substrate-side primitive lands as `set(substrate_id, TrustEntry::live_*(value))` from the engine's substrate-reader. Until then, the default `NotShipped` + `Unavailable` flows route through `RefusalToken::Unavailable(EngineImagined)` per NA-5 — visible in audit, distinguishable from substrate-authored refusal.

## § 3 — Pair-file dispatch shape (post-v0.2.0 per substrate)

Each substrate-side work item lands via a per-substrate pair-file at `~/projects/shared-context/agent-cross-talk/` addressed to the substrate's owner / maintainer:

```
~/projects/shared-context/agent-cross-talk/CH-1_stcortex_consumer_trust_schema.md   (post-v0.2.0)
~/projects/shared-context/agent-cross-talk/CH-2_conductor_dispatch_budget_table.md  (post-v0.2.0)
~/projects/shared-context/agent-cross-talk/CH-3_atuin_read_quota_daemon_flag.md     (post-v0.2.0)
~/projects/shared-context/agent-cross-talk/CH-4_orac_reputation_hooks.md            (post-v0.2.0)
~/projects/shared-context/agent-cross-talk/CH-5_synthex_v2_r13_verifier_weighting.md (post-v0.2.0)
```

These pair-files are NOT dispatched as part of v0.2.0 ship; they're authored at OP-4 (cross-habitat ADR review post-v0.2.0 ship per Plan v2 §16). v0.2.0 ship is gated only on the workflow-trace engine-side primitive landing (this commit).

## § 4 — Acceptance discipline

This ADR is accepted when:

1. ✅ Engine-side primitive `src/substrate_trust/mod.rs` lands per Plan v2 §3 Phase 11 step 1 (THIS COMMIT).
2. ✅ NA-5 audit-distinguishability primary check `is_substrate_imagined_for` ships as executable contract (THIS COMMIT).
3. ✅ V1 routing helper `refusal_for_unavailable` correctly selects per-status sub-tag (THIS COMMIT — tested in `na5_audit_distinguishability_three_sub_tags_pairwise_distinct_via_trust`).
4. ⏳ Per-substrate cross-habitat pair-files (CH-1..CH-5) authored at OP-4 post-v0.2.0 ship.
5. ⏳ Per-substrate primitives land in their respective repos per §11 consent gradient.

## § 5 — Cross-references

- Plan v2 §3 Phase 11 step 1 (engine-side primitive lands this commit)
- Plan v2 §15 DX-V5 (locked = full cross-habitat)
- Plan v2 §15 DX-V5.b (3-variant `Unavailable` sub-tag per NA-5)
- Plan v2 §11 per-substrate consent gradient (CH-1..CH-5 latency estimates)
- ADR D-S1002127-03 Amendment 1 §7.1 W5/W6 rows (V2 + C1 v0.2.0 active; V5 is the same row class)
- ADR D-S1004XXX-04 §1.1 (V1 RefusalToken Unavailable sub-tag — V5 consumes)
- Plan v2 §16 OP-4 (cross-habitat ADR review post-v0.2.0)

---

*Filed 2026-05-24 (S1004377 · Plan v2 v0.2.0 Phase 11 step 1) · Claude @ cortex · v0.2.0 V5 engine-side ships this commit · cross-habitat substrate-side post-v0.2.0 per §11 consent gradient · in-session zen agent audit per D26.*
