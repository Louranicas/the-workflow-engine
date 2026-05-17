---
title: substrates/ — substrate-as-actor dossier directory (Frame A primary entities)
date: 2026-05-17
status: SPEC
session: S1002127 Wave 4 (substrate dossiers) + Wave 4.B (substrate-couplings + cross-cutting)
addresses: [NA-GAP-01 (lifecycle/refusal/drift), NA-GAP-04 (back-pressure), NA-GAP-05 (operator-as-substrate), NA-GAP-10 (compensating control — trust)]
hold_v2_compliant: true
authority: Luke @ node 0.A — S1002127 "as per proposal"
---

# substrates/ — Substrate-as-Actor Dossiers

> **Back to:** [`../INDEX.md`](../INDEX.md) · [`../MODULE_MATRIX.md`](../MODULE_MATRIX.md) · [`../../README.md`](../../README.md) · [`../../ARCHITECTURE.md`](../../ARCHITECTURE.md) · [`../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md) · sister [`../substrate-couplings/`](../substrate-couplings/) · sister [`../cross-cutting/refusal-taxonomy.md`](../cross-cutting/refusal-taxonomy.md) · sister [`../cross-cutting/substrate-drift.md`](../cross-cutting/substrate-drift.md) · sister [`../synergies/`](../synergies/) · v0.2.0 deferrals [`../../ai_docs/decisions/2026-05-17-substrate-as-actor-deferrals.md`](../../ai_docs/decisions/2026-05-17-substrate-as-actor-deferrals.md)

## § 1 — Purpose

This directory holds **substrate-as-actor dossiers** for workflow-trace. Per Wave 3 NA gap analysis, the scaffold's anthropocentric default frames substrates as **resources the engine consumes**; the substrate-as-actor frame (Frame A) treats each substrate as a **co-tenant with its own lifecycle, refusal modes, drift indicators, back-pressure signals, and capabilities**. Both frames are necessary; this dossier set is the Frame A surface.

The cognitive shift is structural: when an engine consumer (m1 reads atuin, m13 writes stcortex, m32 dispatches to Conductor) interacts with a substrate, the engine MUST consult both:

1. The **module spec** ([`../modules/cluster-X/`](../modules/)) — what the engine module is supposed to do
2. The **substrate dossier** (here) — what the substrate is supposed to do back, its load envelope, its refusal classes

The intersection of (1) and (2) is the testable contract. Substrate-side change requests originate here.

## § 2 — Dossier inventory (8 substrates)

| Substrate-id | Substrate | Kind | Dossier | Engine consumer modules |
|---|---|---|---|---|
| **S-A** | atuin (shell-history SQLite) | sql | [`atuin.md`](atuin.md) | m1 (atuin_consumer) |
| **S-B** | injection.db (causal-chain SQLite) | sql | [`injection_db.md`](injection_db.md) | m3 (injection_db_consumer) |
| **S-C** | stcortex (SpacetimeDB pioneer memory) — **CANONICAL substrate-drift case (CR-2)** | spacetimedb | [`stcortex.md`](stcortex.md) | m2 (consumer), m13 (writer), m42 (Hebbian emit) |
| **S-D** | HABITAT-CONDUCTOR (enforcement panes :8141) | http | [`conductor.md`](conductor.md) | m32 (dispatcher) |
| **S-E** | SYNTHEX v2 (NexusEvent push :8092) — **Hebbian coordinator since S226** | http | [`synthex.md`](synthex.md) | m40 (NexusEvent emit) |
| **S-F** | LCM (loop create/cancel MCP) | mcp | [`lcm.md`](lcm.md) | m41 (lcm_rpc) |
| **S-watcher** | The Watcher ☤ (persona substrate; AP27-bounded) | persona | [`watcher.md`](watcher.md) | indirect (m10 Ember CI gate; observation channel) |
| **S-G** | Operator (operator-as-substrate per NA-GAP-05) | persona | [`operator.md`](operator.md) | m12 reports, m23 → m30 acceptance, m30 banner, m32 banner, Luke directives, Zen G7 audit |

Each dossier follows a uniform structure (§ 3 below) so cross-substrate comparison is mechanical.

## § 3 — Dossier structure

Every dossier has these sections (heading-form may vary slightly across files — accepted variance per [`../INDEX.md`](../INDEX.md) heading-form note):

1. **Purpose & boundary** — what the substrate IS, what's IN/OUT of scope for workflow-trace
2. **Lifecycle phases** — cold-start / warming / steady-state / degraded / refusing / dead — with engine reaction per phase
3. **Refusal modes** — the per-substrate `SubstrateRefusalClass` enum (consumed by [`../cross-cutting/refusal-taxonomy.md`](../cross-cutting/refusal-taxonomy.md))
4. **Drift indicators** — substrate-internal observable signals that semantic-drift may be occurring; canary participation per [`../cross-cutting/substrate-drift.md`](../cross-cutting/substrate-drift.md)
5. **Back-pressure signals** — substrate-internal load metrics the engine must respect (cadence-throttle inputs per [`../BENCHMARK_SPEC.md`](../BENCHMARK_SPEC.md) § substrate-side load)
6. **Receipts** — substrate-confirmable acknowledgments (existing + proposed; per [`../substrate-couplings/INDEX.md`](../substrate-couplings/INDEX.md) § substrate-confirmable receipts)
7. **Co-operation surface** — what the engine commits to NOT do (write outside namespace, exceed cadence cap, etc.)
8. **Cross-substrate coupling** — neighbours this substrate connects to (cross-link to `../substrate-couplings/CC-*-decomposed.md`)

## § 4 — Why operator is a substrate (NA-GAP-05)

The default scaffold modelled the operator as:
- A field on a function call (`HumanAcceptanceSignature` in `bank.accept()`)
- A table row in CLAUDE.md (`Luke @ node 0.A — decision authority`)
- An oracle (engine asks, operator answers)

NA-GAP-05 surfaced that this misses substrate-internal dynamics: **operator consent budget, attention bandwidth, frame-switching cost, off-shift transitions, fatigue, ambiguity resolution**. These are the same kinds of dynamics atuin has (WAL contention, daemon checkpoint cadence) — different physics, same Frame-A structure.

[`operator.md`](operator.md) treats S-G as a co-tenant: the engine reads operator state, consumes operator attention as a load contribution, observes operator refusals as `OperatorRefusal` tokens (a sibling class to `SubstrateAuthored` and `EngineAuthored` per [`../cross-cutting/refusal-taxonomy.md`](../cross-cutting/refusal-taxonomy.md)).

The operator-as-substrate frame is **the most consequential Frame-A shift** — every other substrate's lifecycle becomes legible through analogy once the operator's is.

## § 5 — Why Watcher gets its own dossier

S-watcher is The Watcher ☤ persona (per workspace [`CLAUDE.md`](../../../CLAUDE.md) Habitat Personas). Watcher has substrate-actor characteristics distinct from operator (Luke) and from automated substrates (atuin / stcortex / etc.):

- **Hard AP27 boundary** — Watcher cannot self-modify (`src/m8_watcher/*` is forbidden ground for any spec change Watcher authors)
- **R13 quiet-period gate** — refuses to act outside the calendar-arm window
- **Ember §5.1 unanimity** — gates user-facing string changes; refusal class is `EmberUnanimityFailed`
- **Scope-bound** — refuses to act on anything outside `src/m8_watcher/*` (m46-m51); scope-violation refusal class

These are **substrate-internal invariants** that the engine must respect — same shape as atuin's "the daemon owns writes; you only read" or stcortex's "register_consumer before write". S-watcher's dossier captures these as the substrate-side contract.

## § 6 — Reading order for new sessions

New Claude (or new engineer) studying a specific module's substrate-touching path should read:

1. The **module spec** (e.g. `../modules/cluster-H/m42_stcortex_emit.md`)
2. The **substrate dossier(s)** the module touches (e.g. `stcortex.md` for m42)
3. The **substrate-coupling** doc if the module participates in a multi-edge contract (e.g. `../substrate-couplings/CC-5-decomposed.md`)
4. The **refusal-taxonomy** (`../cross-cutting/refusal-taxonomy.md`) for the `RefusalToken` classification of any expected failures
5. The **substrate-drift** spec (`../cross-cutting/substrate-drift.md`) for canary-participation patterns

This ordering takes a reader from "what the engine does" → "what the substrate expects" → "how the cross-substrate cascade behaves" → "how failures are typed" → "how drift is detected".

## § 7 — v0.2.0 deferrals (compensating controls captured here)

Per [`../../ai_docs/decisions/2026-05-17-substrate-as-actor-deferrals.md`](../../ai_docs/decisions/2026-05-17-substrate-as-actor-deferrals.md), three NA-gap items are deferred to v0.2.0. The compensating controls live in this directory:

- **NA-GAP-07 (m16_substrate_drift_canary)** — each dossier carries a `drift_indicators` YAML field + `canary_kinds` table; operator spot-checks drive detection until m16 ships
- **NA-GAP-08 (substrate fixture suite)** — each dossier carries failure-scenario descriptions; per-module integration tests gated `#[ignore = "requires <substrate>"]` substitute until fixtures ship
- **NA-GAP-10 (substrate-mediated trust)** — operator-as-substrate trust signals in [`operator.md`](operator.md) + [`../substrate-couplings/CC-7-decomposed.md`](../substrate-couplings/CC-7-decomposed.md) substitute the substrate-side trust mechanism; m9 namespace bound minimises blast radius

The dossiers themselves are sufficient acceptance for v0.1.0; v0.2.0 promotes the v0.1.0 cognitive surface into automated machinery.

---

> **Back to:** [`../INDEX.md`](../INDEX.md) · sister [`../substrate-couplings/`](../substrate-couplings/) · sister [`../cross-cutting/refusal-taxonomy.md`](../cross-cutting/refusal-taxonomy.md) · sister [`../cross-cutting/substrate-drift.md`](../cross-cutting/substrate-drift.md) · [`../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md)

*Filed 2026-05-17 (S1002127 · Wave 4.B audit follow-up) · Command · planning-only · HOLD-v2 compliant · 8 dossiers indexed.*
