# ai_specs/INDEX — workflow-trace prescriptive specs

> **Back to:** [`../README.md`](../README.md) · [`../CLAUDE.md`](../CLAUDE.md) · [`../ARCHITECTURE.md`](../ARCHITECTURE.md)
> **Sister index:** [`../ai_docs/INDEX.md`](../ai_docs/INDEX.md) (descriptive runtime docs) · [`../ultramap/README.md`](../ultramap/README.md) (flow maps)

---

## What lives here

`ai_specs/` carries **prescriptive** specs — what each module MUST be (god-tier Rust contracts that constrain implementation). Compare with `ai_docs/` which carries **descriptive** runtime documentation.

Every `.rs` source file authored post-G9 has its spec here. Specs are authoritative; if code drifts from spec, the spec wins (or the spec is amended via D-B6 AMEND-loop). G9 fired 2026-05-17 and the 26-module codebase is implemented — these specs are now read against live source, not as a pre-build target.

## Descriptive runtime docs (sister surface — `ai_docs/`)

The following operator/developer docs are **descriptive** (how the running system behaves), so they live in `ai_docs/` and `docs/`, not here. Cross-referenced for navigation:

| Doc | Purpose |
|---|---|
| [`../QUICKSTART.md`](../QUICKSTART.md) | 5-minute developer quickstart |
| [`../docs/DIAGNOSTICS.md`](../docs/DIAGNOSTICS.md) | Diagnostics + troubleshooting (gate, both binaries, external-service matrix, m8 build warnings) |
| [`../docs/COMMAND_MAPPING.md`](../docs/COMMAND_MAPPING.md) | `wf-crystallise` / `wf-dispatch` CLI flag reference |
| [`../ai_docs/API_MAP.md`](../ai_docs/API_MAP.md) | Public-API surface (`pub use` inventory) — the descriptive companion to `API_SPEC.md` below |
| [`../ai_docs/HARDENING_FLEET_2026-05-21.md`](../ai_docs/HARDENING_FLEET_2026-05-21.md) | Hardening Fleet remediation record (W0–W5 + S1003733 resolution) |

---

## Module specs (per cluster, 26 total)

[`MODULE_MATRIX.md`](MODULE_MATRIX.md) — 26 modules × 30 features verification matrix

### By cluster

| Cluster | Layer | Modules | Spec dir |
|---|---|---|---|
| **A** Substrate Ingest | L1 | m1, m2, m3 | [`modules/cluster-A/`](modules/cluster-A/) |
| **B** Habitat Observation | L2 | m4, m5, m6 | [`modules/cluster-B/`](modules/cluster-B/) |
| **C** Correlation + Output | L3 | m7, m12, m13 | [`modules/cluster-C/`](modules/cluster-C/) |
| **D** Trust (cross-cutting) | L4 | m8, m9, m10, m11 | [`modules/cluster-D/`](modules/cluster-D/) |
| **E** Evidence + Pressure | L5 | m14, m15 | [`modules/cluster-E/`](modules/cluster-E/) |
| **F** Iteration (KEYSTONE) | L6 | m20, m21, m22, m23 | [`modules/cluster-F/`](modules/cluster-F/) |
| **G** Bank/Select/Dispatch/Verify | L7 | m30, m31, m32, m33 | [`modules/cluster-G/`](modules/cluster-G/) |
| **H** Substrate Feedback | L8 | m40, m41, m42 | [`modules/cluster-H/`](modules/cluster-H/) |

### Cluster-level specs (synthesis from vault + V7)

[`layers/cluster-A.md`](layers/cluster-A.md) · [`layers/cluster-B.md`](layers/cluster-B.md) · [`layers/cluster-C.md`](layers/cluster-C.md) · [`layers/cluster-D.md`](layers/cluster-D.md) · [`layers/cluster-E.md`](layers/cluster-E.md) · [`layers/cluster-F.md`](layers/cluster-F.md) · [`layers/cluster-G.md`](layers/cluster-G.md) · [`layers/cluster-H.md`](layers/cluster-H.md) — TBD Wave 1

---

## Cross-cutting specs (root-level)

| Spec | Purpose | Status |
|---|---|---|
| `API_SPEC.md` | Public APIs across `wf-crystallise` + `wf-dispatch` + `workflow_core` lib | TBD Wave 2 |
| `DATABASE_SPEC.md` | SQLite migrations (workflow_runs hub, atuin reader contract, injection.db reader contract) + stcortex namespace tables | TBD Wave 2 |
| `EVENT_SYSTEM_SPEC.md` | NexusEvent envelope (m40) + pressure register events (m15) + Hebbian emit (m42) | TBD Wave 2 |
| `WIRE_PROTOCOL_SPEC.md` | m42 stcortex pathway envelope + m41 LCM RPC `lcm.loop.create` payload + m40 NexusEvent push schema | TBD Wave 2 |
| `IPC_BUS_SPEC.md` | Internal bus between wf-crystallise and wf-dispatch (if any) — likely none; CLI handoff via JSONL | TBD Wave 2 |
| `DESIGN_CONSTRAINTS.md` | Invariants enforced at compile-time + runtime (newtype discipline, namespace.rs constants, EscapeSurfaceProfile ordinal) | TBD Wave 2 |
| `CONSENT_SPEC.md` | F11 Held semantics, modulation-not-command for non-safety rules, Ember §5.1 amendment integration | TBD Wave 2 |
| `SECURITY_SPEC.md` | auth model, namespace guard, destructiveness scoring, EscapeSurfaceProfile threat model | TBD Wave 2 |
| `ERROR_TAXONOMY.md` | thiserror taxonomy by cluster | TBD Wave 2 |
| `OBSERVABILITY_SPEC.md` | tracing structured fields, metric names, log levels, span hierarchy | TBD Wave 2 |
| `TEST_STRATEGY.md` | Per-cluster test-kind allocation; KEYSTONE Cluster F bench/property/fuzz | TBD Wave 2 (synthesis of [`../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md`](../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md)) |
| `BENCHMARK_SPEC.md` | criterion benchmarks (m20 PrefixSpan 10k rows, m4 correlator at scale) | TBD Wave 2 |

## Synergy contracts

| Spec | Contract | Source |
|---|---|---|
| `synergies/CC-1.md` | Cascade-Cost Coupling (B internal via m7 JSONB) | [Cluster A/B/C specs](../the-workflow-engine-vault/module%20specs/) |
| `synergies/CC-2.md` | Trust Layer Woven (D → all) | [Cluster D spec](../the-workflow-engine-vault/module%20specs/cluster-D-trust-cross-cutting.md) |
| `synergies/CC-3.md` | Evidence-Driven Iteration (E → F) | [Cluster E/F specs](../the-workflow-engine-vault/module%20specs/) |
| `synergies/CC-4.md` | Proposal→Bank→Dispatch Pipeline | [Cluster F/G specs](../the-workflow-engine-vault/module%20specs/) |
| `synergies/CC-5.md` | Substrate Learning Loop (G→H→F) | [Cluster G/H/F specs](../the-workflow-engine-vault/module%20specs/) |
| `synergies/CC-6.md` | Verification-Gated Dispatch (G-internal m33→m32) | [Cluster G spec](../the-workflow-engine-vault/module%20specs/cluster-G-bank-select-dispatch-verify.md) |
| `synergies/CC-7.md` | Pressure-Driven Evolution (E → spec interviews) | [Cluster E spec](../the-workflow-engine-vault/module%20specs/cluster-E-evidence-pressure.md) |
| Canonical | All 7 contracts cross-verified | [`CROSS_CLUSTER_SYNERGIES.md`](../ai_docs/optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md) |

## Patterns

| Spec | Source |
|---|---|
| `patterns/orac-single-crate.md` | [G2-consolidation](../ai_docs/optimisation-v7/GENERATIONS/G2-consolidation.md) |
| `patterns/cluster-d-early-ship.md` | [phase-1](../the-workflow-engine-vault/deployment%20framework/phase-1-genesis-day-0-3.md) |
| `patterns/feature-gate-matrix.md` | [phase-1](../the-workflow-engine-vault/deployment%20framework/phase-1-genesis-day-0-3.md) |
| `patterns/me-v2-m1-foundation-lift.md` | [MODULE_SPECS_INDEX](../the-workflow-engine-vault/module%20specs/MODULE_SPECS_INDEX.md) § ME v2 patterns |

## Cross-cutting (per-axis)

| Spec | Axis |
|---|---|
| `cross-cutting/observability.md` | tracing + metrics + logs |
| `cross-cutting/error-handling.md` | thiserror taxonomy + ? propagation |
| `cross-cutting/concurrency.md` | tokio runtime, spawn discipline, AP29 |
| `cross-cutting/persistence.md` | SQLite + JSONL outbox + stcortex |
| `cross-cutting/feature-gating.md` | default/full/api/intelligence/monitoring/evolution |
| [`cross-cutting/refusal-taxonomy.md`](cross-cutting/refusal-taxonomy.md) | `RefusalToken` — substrate-authored / engine-authored / operator-authored typing + `WireEvent::Refusal` Class-C envelope (NA-GAP-02, NA-GAP-05, NA-GAP-11) |
| [`cross-cutting/substrate-drift.md`](cross-cutting/substrate-drift.md) | first-class substrate-drift detection, canary contract, `SubstrateDriftDetected` event, CR-2 POVM canonical case (NA-GAP-07) |

## Substrate dossiers (NA-GAP remediation Wave 4 — Frame A primary entities)

Per [`../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md), the scaffold's anthropocentric module-primary view is complemented by a Frame A substrate-primary view. Each substrate carries its own lifecycle, refusal modes, drift indicators, back-pressure signals, receipts, capabilities, internal couplings, and test fixtures.

| Substrate | Dossier | Kind | Engine consumers |
|---|---|---|---|
| **S-A** atuin | [`substrates/atuin.md`](substrates/atuin.md) | sql | m1 |
| **S-B** injection.db | [`substrates/injection_db.md`](substrates/injection_db.md) | sql | m3 |
| **S-C** stcortex (CANONICAL substrate-drift case) | [`substrates/stcortex.md`](substrates/stcortex.md) | spacetimedb | m2, m13, m42 |
| **S-D** HABITAT-CONDUCTOR | [`substrates/conductor.md`](substrates/conductor.md) | http | m32 |
| **S-E** SYNTHEX v2 | [`substrates/synthex.md`](substrates/synthex.md) | http | m40 |
| **S-F** LCM | [`substrates/lcm.md`](substrates/lcm.md) | mcp | m41 |
| **S-watcher** The Watcher ☤ persona | [`substrates/watcher.md`](substrates/watcher.md) | persona | indirect |
| **S-G** Operator (operator-as-substrate per NA-GAP-05) | [`substrates/operator.md`](substrates/operator.md) | persona | m12, m23→m30, m30, m32 banner, Luke directives, Zen G7 |

## Substrate-substrate couplings (NA-GAP-03/09 closure — Wave 4.B)

`synergies/CC-*.md` enumerates engine-side cross-cluster contracts. `substrate-couplings/CC-*-decomposed.md` enumerates the **substrate-substrate edges** hidden inside those contracts — information channels between persistent substrate-state stores, owned by substrates (not the engine).

| File | Parent synergy | Substrate edges | NA-GAP closure |
|---|---|---|---|
| [`substrate-couplings/INDEX.md`](substrate-couplings/INDEX.md) | — | landing + verification-discipline pattern + substrate-confirmable-receipt convention | NA-GAP-09 framework |
| [`substrate-couplings/CC-5-decomposed.md`](substrate-couplings/CC-5-decomposed.md) | [`synergies/CC-5.md`](synergies/CC-5.md) (SPECIAL DEPTH) | 5 edges: m32→S-C, m32→S-E→S-C, S-C→habitat-memory→S-B, m32→S-F→V3-partner, S-C→digest→S-G | NA-GAP-03, NA-GAP-09 PRIMARY |
| [`substrate-couplings/CC-4-decomposed.md`](substrate-couplings/CC-4-decomposed.md) | [`synergies/CC-4.md`](synergies/CC-4.md) | 3 edges: m32→S-D, S-D refusal-path, m30→S-G operator (AP-V7-07 acceptance signature) | NA-GAP-03 (secondary) + AP-V7-13 enrichment |
| [`substrate-couplings/CC-7-decomposed.md`](substrate-couplings/CC-7-decomposed.md) | [`synergies/CC-7.md`](synergies/CC-7.md) | 4 edges: m15→S-G pressure-row, S-G→spec-amendment fanout, S-G→S-watcher Ember §5.1 gate, S-G consent-budget→m12 | NA-GAP-03 (operator) + NA-GAP-05 (operator-as-substrate) |

Engine-internal CCs (CC-1, CC-1.subA, CC-2, CC-3, CC-6) are NOT decomposed here — they have no substrate-substrate edges beyond the trivial atuin-read / SQLite-internal cases already documented in `substrates/`.

---

## Binary HTTP shapes

- [`WF_DAEMON_HTTP_SHAPE.md`](WF_DAEMON_HTTP_SHAPE.md) — formal HTTP contract for the `wf-daemon` habitat service (port 8142, single endpoint `GET /health`, liveness-only contract scope, env overrides). Wave-16 / S1005032.

## Schematics

`schematics/` — per-module Mermaid diagrams (module-internal flow) + per-cluster Mermaid diagrams (cluster-internal contracts).

---

> **Back to:** [`../README.md`](../README.md) · [`../ARCHITECTURE.md`](../ARCHITECTURE.md) · sister [`../ai_docs/INDEX.md`](../ai_docs/INDEX.md) · [`MODULE_MATRIX.md`](MODULE_MATRIX.md) · NA remediation [`../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md) · [`substrates/`](substrates/) · [`substrate-couplings/`](substrate-couplings/) · [`cross-cutting/refusal-taxonomy.md`](cross-cutting/refusal-taxonomy.md) · [`cross-cutting/substrate-drift.md`](cross-cutting/substrate-drift.md) · v0.2.0 deferrals [`../ai_docs/decisions/2026-05-17-substrate-as-actor-deferrals.md`](../ai_docs/decisions/2026-05-17-substrate-as-actor-deferrals.md)
