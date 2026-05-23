> **Back to:** [[CLAUDE.md]] · [[CLAUDE.local.md]] · [`../CLAUDE.local.md` § v0.2.0 RATIFIED](../CLAUDE.local.md) · canonical [`../ai_docs/WORKFLOW_TRACE_V020_PLAN_V2_S1004377.md`](../ai_docs/WORKFLOW_TRACE_V020_PLAN_V2_S1004377.md) · v1 DRAFT [`../ai_docs/WORKFLOW_TRACE_V020_PLAN_S1004377.md`](../ai_docs/WORKFLOW_TRACE_V020_PLAN_S1004377.md) · conventional gap analysis [`../ai_docs/WORKFLOW_TRACE_V020_PLAN_S1004377_CONVENTIONAL_GAP_ANALYSIS.md`](../ai_docs/WORKFLOW_TRACE_V020_PLAN_S1004377_CONVENTIONAL_GAP_ANALYSIS.md) · NA gap analysis [`../ai_docs/WORKFLOW_TRACE_V020_PLAN_S1004377_NA_GAP_ANALYSIS.md`](../ai_docs/WORKFLOW_TRACE_V020_PLAN_S1004377_NA_GAP_ANALYSIS.md)
> **Type:** v0.2.0 plan v2 vault mirror · 4-surface persist · cold-start anchor for v0.2.0 execution
> **Date:** 2026-05-23 (S1004377) · **Status:** PLAN v2 RATIFIED — awaiting Luke @ node 0.A "start Phase 1" go per D48

# Workflow-Trace v0.2.0 Plan v2 — Vault Mirror

The full canonical plan is at [`../ai_docs/WORKFLOW_TRACE_V020_PLAN_V2_S1004377.md`](../ai_docs/WORKFLOW_TRACE_V020_PLAN_V2_S1004377.md) (601 lines / ~10,100 words). This vault mirror surfaces the navigable summary + the bidirectional anchor.

## What `v0.2.0` certifies (re-labelled per NA-10 + T-3)

**Engine-side substrate-participation readiness.** Every primitive the engine needs to *participate in* (not *consume*) the substrate is shipped, tested, audited, documented across **7 substrates** (atuin + stcortex + Conductor + CC-5 clocks + Luke + Watcher ☤ + RALPH + Cargo build graph).

It does **not** certify substrate-side primitives — those are post-v0.2.0 cross-habitat coordination, sized per per-substrate consent gradient in §11.

## The 5-tier scope (locked at v2 ratification)

| Tier | Items | LOC | Decision-locked? |
|------|-------|-----|------------------|
| **Tier 1 — substrate-as-actor primitives** | V1 RefusalToken (3-variant Unavailable sub-tag per NA-5) · V2 per-substrate `SubstrateBackPressureMode` enum (NA-8 reshape) · V3 m16 own-module canary (DX-V3) · V4 full deterministic replicas (DX-5) · V5 full cross-habitat trust + `substrate_participation_status` accessor (DX-V5 + NA-5) | ~1,450-1,700 | ✅ |
| **Tier 2 — wire-contracts** | W1 `escape_surface` SemVer-break (DX-W.a retire (iii) + DX-W.b W1 + DX-W.c break) · W3 `cost: i64` using `variant.mutation` count (DX-W3.src) · W4 `CuratedBank::client_ref()` | ~340-510 | ✅ |
| **Tier 3 — real verifiers** | R1 Security real (consumes W1) · R2 Cost real (consumes W3) · R3 Consistency real variant_id-only (DX-R3) | ~200-330 | ✅ |
| **Tier 4 — algorithmic upgrades** | A1 Levenshtein steps-on-proposal (DX-4) · A2 SD9 FeatureVector newtype (Phase 3) · A3 SD10 retain-prior (DX-3) · A4 SD11 12-field shape (DX-A4-coupling — co-lands Phase 5) | ~230-360 | ✅ |
| **Carry-overs** | C1 m13 outbox drain (paired with V1 Phase 5) · C3 CI submodule (DX-CI Option A) · C5 `--execute` Phase 12 acceptance criteria | n/a | ✅ |

## 21 Phase 4 interview decisions (locked S1004377)

### Round A — load-bearing, no defaults

| ID | Locked decision | Rationale anchor |
|----|-----------------|------------------|
| **DX-DAW-1** | Tier 2 first (engineering-frame) | wire-contracts un-block Tier 3 cleanly; §10 T-2 acknowledged as Frame-A geometry |
| **DX-W.a** | Retire (iii) audit-overlay | R1 Security is new enforcement seam, not redundancy |
| **DX-W.b** | W1 wire-bump | substrate can name its surface; ~150-200 LOC |
| **DX-W.c** | SemVer-break | v0.2.0 breaking; CHANGELOG migration note |
| **DX-W3.src** | `variant.mutation` count | existing primitive; simplest |
| **DX-V3** | Own module (m16) | Cluster E expansion; triggers Genesis v1.4 + Zen G7 re-audit |
| **DX-V3.b** | Ship at N=7d with honest residual | Zen-silent cap converts unbounded variance to CHANGELOG honest-residual |
| **DX-V5** | Full cross-habitat | ADR D-S1004XXX-05 pair-filed; engine half ships in v0.2.0; substrate-side post-v0.2.0 per §11 |
| **DX-V5.b** | 3-variant sub-tag | `Unavailable(EngineImagined / SubstrateUnreachable / SubstrateAuthored)` prevents NA-5 indistinguishability |
| **DX-2** | Per-substrate `SubstrateBackPressureMode` enum | NA-8 reshape; heterogeneous substrate landscape; default `Pull` per substrate |
| **DX-1** | 4-variant RefusalToken (no WatcherAuthored) | Watcher emits via observation channel, not refusal channel |
| **DX-5** | Full deterministic replicas | TEST_STRATEGY bump to ~1,750-1,800 |
| **DX-A4-coupling** | Phase 5 co-land with W1+W3 | one wire-contract regen pass; R3 unambiguous |
| **DX-CI** | Option A submodule | Frame-B observation point per NA-2; substrate-coupling explicit |
| **DX-MGB** | Cap 4h per phase | encourages `// mutant-equivalent:` proof discipline |

### Round B — mechanical / policy

| ID | Locked decision |
|----|-----------------|
| **DX-3** | retain-prior (default; M0 ships this) |
| **DX-4** | steps-on-proposal (couples to A4 Phase 5) |
| **DX-R3** | variant_id-only (default; lineage-chain is v0.3.0) |

### Stated defaults (NOT interview slots per C-10)

- **DX-Mut** = hold ≥96.3 %
- **DX-Soak** = 48h baseline
- **DX-1-mechanism** = 4-variant structural floor

## 12-phase structure (Tier-2-first ordering per DX-DAW-1)

1. Re-baseline + ADR D-S1002127-03 amendment cascade + file:line re-verify + `mutation-weight` pin (~1-1.5 d)
2. Deep FP-verification + Tier 2 sizing + 7-substrate audit (~0.5-1 d)
3. A2 FeatureVector + C1 drain skeleton (decision-free) (~0.5-1 d)
4. **Decision interview LOCKED S1004377** (~21 decisions)
5. **Tier 2 W1 + V1 co-land + W3 + W4 + A4 SD11** (one wire-contract regen) (~5-7 d)
6. Tier 3 R1 + R2 + R3 (~3-4 d)
7. V1 call-site threading + drain wire (shrunk per C-2) (~0.5-1 d)
8. V2 per-substrate back-pressure (~2-3 d)
9. **V3 m16 own-module canary (KEYSTONE)** — Genesis v1.4 + Zen 7d ship-residual cap (~5-12 d)
10. V4 full deterministic replicas (~3-4 d)
11. V5 cross-habitat ADR + `substrate_participation_status` accessor (~3-4 d)
12. Tier 4 A1/A3 + integration + Zen audit + v0.2.0 ship (~2-3 d)

**Total v0.2.0 full Plan-v2-arc:** ~31-42 Claude-days; mid-point ~36.

## What Part B (substrate-frame) re-authored

§7 expanded from 4 substrates (v1) to **7 substrates** (v2 per NA-2): atuin / stcortex / Conductor / CC-5 clocks / Luke + Watcher ☤ + RALPH + Cargo build graph. Each gets a paragraph in §7.2-7.8 of the canonical plan + a row in §11 post-v0.2.0 partition.

§9 gained **recursion sub-section §9.2** per NA-9 + convergent C-2. §9.2 catches that §9.1 missed first-order checks on V1 (authorship classification table is Frame-A) and V4 (instrumentation-only; live-substrate exercise is post-v0.2.0). §9.2 confirms §1+§8 certification language re-labelling (NA-10 + T-3): "engine + substrate co-completeness" → "engine-side substrate-participation readiness."

§11 gained **per-substrate consent gradient** (NA-6): stcortex HIGH 1-2wk · Conductor HIGH 1-2wk · atuin UNKNOWN indeterminate · ORAC MED 2-4wk · synthex-v2 HIGH 1-2wk.

## 25 gap-analysis findings folded in

- **Conventional (13):** 4 HIGH (C-2 V1↔W1 coupling → co-land Phase 5; C-3 DX-W split a/b/c; C-4 Phase 9 fork by DX-V3 + DX-V3.b cap; C-5 mutation-weight source pinned) + 7 MED + 2 LOW
- **NA (12):** 5 HIGH (NA-1 DAW-1 reopened as DX-DAW-1; NA-2 §7 expansion to 7 substrates; NA-3 DX-1/DX-2/DX-5 promoted to Round A; NA-4 Watcher liveness assertion; NA-5 RefusalToken sub-tagging) + 7 MED/LOW
- **Tensions (3 load-bearing + 1 added per NA-5):** T-1 DAW-1 reopen; T-2 §9 recursion; T-3 certification re-label; T-4 Unavailable sub-tagging — all reconciled
- **Convergent (4):** C-1 NA-1+T-1 / C-2 NA-9+v0.1.0 precedent / C-3 NA-2+workspace CLAUDE.md / C-4 NA-11+slug discipline — all adopted

No finding rejected; 3 accepted-with-honest-labelling (NA-7 framing label, NA-12 retroactively closed, C-11 spine label-not-resolve).

## What's standing for Luke @ node 0.A

The single remaining gate per D48: **explicit "start Phase 1" go.** All design decisions are locked; Phases 1-3 are decision-free; Phases 5-12 are unblocked by §15. Plus the v0.1.0-era standing items:

- **OP-1 / B3** — Conductor bring-up + 24h NoOp soak + flip `CONDUCTOR_ENFORCEMENT_ENABLED=1` (sandbox-forbidden devenv start)
- **OP-2** — directory rename `the-workflow-engine/` → `workflow-trace/` (still post-M0 cosmetic per D32)
- **OP-3 (NEW)** — Post-v0.2.0 live substrate soak (Watcher carries; 48h default per DX-Soak)
- **OP-4 (NEW)** — Cross-habitat ADR D-S1004XXX-05 review post-v0.2.0 ship (substrate-side changes for 5 substrates per §11 consent gradient)
- **OP-5 (NEW)** — Master Plan v2 / Ember opportunity-cost reopen per Plan v2 D46 after v0.2.0 ships
- **OP-6 (NEW per NA-4)** — Watcher m16 heartbeat liveness integration (closes V3 self-canary loop honestly)

## 4-surface persistence at this save

| Surface | Anchor |
|---------|--------|
| **ai_docs canonical** | [`../ai_docs/WORKFLOW_TRACE_V020_PLAN_V2_S1004377.md`](../ai_docs/WORKFLOW_TRACE_V020_PLAN_V2_S1004377.md) + v1 + 2 gap-analysis docs |
| **Obsidian vault** | **THIS note** + supersedes nothing (first v0.2.0 vault entry; complements [[Session S1004115 — Completion Plan v2 Locked]] from v0.1.0 era) |
| **stcortex** | namespace `workflow_trace_v020_s1004377` — **meta memory id 18511 (READ-BACK VERIFIED per NA-6)** + bidi pathways `workflow_trace_completion_s1004115 ↔ workflow_trace_v020_s1004377` (weight 0.95 each direction). Consumer registered. parent_ids = [18473 (v0.1.1+v0.2.0 prep), 18442 (M0 ship), 18383 (Plan v2 interview-locked)]. |
| **CLAUDE.local.md** | project file § "v0.2.0 RATIFIED" flip from earlier § "v0.2.0 IN FLIGHT (PLANNING)" |
| **CHANGELOG** | `[v0.2.0-WIP]` entry lands in Phase 1 (Plan v2 D44 pattern) |
| **injection.db** | causal_chain **id 119** `workflow_trace_v020_plan_v2_ratified_s1004377` (origin+resolved session 1004377) |
| **git tag** | `v0.2.0` annotated at Phase 12 ship (not now) |

— Workflow-Trace v0.2.0 Plan v2 vault mirror · S1004377 · 2026-05-23.
