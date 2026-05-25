# MODULE_MATRIX — 26 modules × 30 features

> **Canonical:** [`../ai_docs/optimisation-v7/ULTRAMAP.md`](../ai_docs/optimisation-v7/ULTRAMAP.md) View 2
> **Vault source:** [`../the-workflow-engine-vault/Modules Synergy Clusters and Feature Verification S1001982.md`](../the-workflow-engine-vault/Modules%20Synergy%20Clusters%20and%20Feature%20Verification%20S1001982.md)
> **This file:** compact lookup matrix. Each row = one module; each column = one capability/feature axis.

---

## Matrix (26 × axes)

Columns:
- **Cluster / Layer** — A/B/.../H, L1-L8
- **Binary** — `wf-crystallise` (C) or `wf-dispatch` (D)
- **LOC** — estimated source LOC
- **Tests** — min test count per [TEST_DISCIPLINE](../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md)
- **Test kind** — unit / async / property / integration / bench
- **Feature gate** — `default`/`full`/`api`/`intelligence`/`monitoring`/`evolution` or `none` (Cluster D = aspect-layer)
- **Reads from** — primary input substrate
- **Writes to** — primary output substrate
- **Verb class** — passive (record/ingest/correlate/emit/refuse) vs active (recommend/dispatch/route/select)
- **CC contracts** — synergy contracts owned
- **Gap-owner** — Gap 1 / Gap 2 / Gap 3 / none
- **Boilerplate lift %** — approximate
- **Status** — SPEC / WIP / BUILT / SOAK / LIVE

| # | Module | Clstr | Bin | LOC | Tests | Test kind | Feature | Reads | Writes | Verb | CC | Gap | Lift | Status |
|---|---|---|---|---:|---:|---|---|---|---|---|---|---|---:|---|
| 1 | `m1_atuin_consumer` | A · L1 | C | 80 | 50 | unit | none | atuin.db | iter | passive | CC-1, CC-3 | — | 30% | SPEC |
| 2 | `m2_stcortex_consumer` | A · L1 | C | 80 | 50 | async | none | stcortex :3000 | iter+trust signal | passive | CC-2 | — | 25% | SPEC |
| 3 | `m3_injection_db_consumer` | A · L1 | C | 70 | 50 | unit | none | injection.db | iter | passive | CC-1 | — | 40% | SPEC |
| 4 | `m4_cascade_correlator` | B · L2 | C | 160 | 60 | property | none | m1 iter | m7 row | correlate | CC-1, CC-3 | — | 50% | SPEC |
| 5 | `m5_battern_step_record` | B · L2 | C | 130 | 60 | unit | none | m1 iter | m20 iter | record | CC-3 | — | 35% | SPEC |
| 6 | `m6_context_cost` | B · L2 | C | 170 | 60 | unit | none | m1 iter | m7 JSONB | record | CC-1 | — | 30% | SPEC |
| 7 | `m7_workflow_runs` | C · L3 | C | 140 | 60 | integration | api | m3, m4, m6 | SQLite hub | record | CC-1 (hub) | — | 40% | SPEC |
| 12 | `m12_cli_reports` | C · L3 | C | 110 | 60 | unit | api | m7 | stdout/JSON | emit | — | — | 65% | SPEC |
| 13 | `m13_stcortex_writer` | C · L3 | C | 120 | 60 | async | api | m7, m2 trust signal | stcortex :3000 | emit | CC-2, CC-5 | — | 45% | SPEC |
| 8 | `m8_povm_build_prereq` | D · L4 | C | 60 | 50 | unit | none | env | build.rs cfg | refuse | CC-2 | — | 0% (NEW) | SPEC |
| 9 | `m9_watcher_namespace_guard` | D · L4 | C | 60 | 50 | unit | none | namespace.rs | runtime assert | refuse | CC-2 | Gap 3 | 20% | SPEC |
| 10 | `m10_ember_ci_gate` | D · L4 | C | 90 | 60 | unit | none | rubric | CI gate | refuse | CC-2 | — | 15% | SPEC |
| 11 | `m11_fitness_weighted_decay` | D · L4 | C | 90 | 70 | property | none | m7 stats | m30 input | record | CC-2 | **Gap 2** | 0% (NEW) | SPEC |
| 14 | `m14_habitat_outcome_lift` | E · L5 | C | 110 | 70 | property | intelligence | m7 | m22, m23 input | record | CC-3 | — | 25% | SPEC |
| 15 | `m15_pressure_register` | E · L5 | C | 90 | 60 | unit | intelligence | runtime signals | JSONL files | emit | CC-7 | — | 30% | SPEC |
| 20 | `m20_prefixspan_miner` | **F · L6** | C | 300 | **75** | property+bench | intelligence | m5 iter | m21 input | recommend* | CC-4 | **Gap 1** | 0% (NEW) | SPEC |
| 21 | `m21_variant_builder` | **F · L6** | C | 200 | 65 | property | intelligence | m20 | m22 input | recommend* | CC-4 | Gap 1 | 5% | SPEC |
| 22 | `m22_kmeans_feature` | **F · L6** | C | 170 | 60 | property | intelligence | m21, m14 | m23 input | recommend* | CC-3, CC-4 | Gap 1 | 20% | SPEC |
| 23 | `m23_workflow_proposer` | **F · L6** | C | 180 | 60 | property | intelligence | m22, m14 | operator review queue | recommend* | CC-4 | Gap 1 | 10% | SPEC |
| 30 | `m30_curated_bank` | G · L7 | D | 220 | 70 | integration | api | operator review, m11 | SQLite bank | record+select | CC-4, CC-5 | **Gap 3** | 30% | SPEC |
| 31 | `m31_selector` | G · L7 | D | 240 | 70 | property | api | m30 | m32 input | select | CC-4, CC-5 | — | 25% | SPEC |
| 32 | `m32_conductor_dispatcher` | G · L7 | D | 290 | 75 | integration | api | m31, m9 | HABITAT-CONDUCTOR | dispatch | CC-4, CC-6 | Gap 3 | 35% | SPEC |
| 33 | `m33_verifier` | G · L7 | D | 200 | 70 | integration | api | m30, m32 | bank PASS/FAIL/DEGRADED | refuse | CC-6 | — | 25% | SPEC |
| 40 | `m40_nexusevent_emit` | H · L8 | C | 160 | 65 | async | monitoring | m32 result | SYNTHEX :8092 | emit | CC-5 | — | 40% | SPEC |
| 41 | `m41_lcm_rpc` | H · L8 | C | 140 | 65 | async | monitoring | m32 result | LCM MCP | emit | CC-5 | — | 35% | SPEC |
| 42 | `m42_stcortex_emit` | H · L8 | C | 150 | 70 | async | monitoring | m32 result, m13 | stcortex pathways | emit | CC-5 | — | 30% | SPEC (POVM **DECOUPLED**) |

\* recommend = Phase B active verb permitted under single-phase override per Genesis v1.3 § 1.

**Totals:** 26 modules · ~3,810 LOC · 1,599 tests (matches top-1% per [TEST_DISCIPLINE](../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md) Table A).

---

## Status legend

| Status | Meaning |
|---|---|
| **SPEC** | Markdown spec authored (planning-only, HOLD-v2 respected) |
| **WIP** | Implementation in progress (post-G9 only) |
| **BUILT** | All tests pass, 4-stage gate green |
| **SOAK** | Deployed; observation window active |
| **LIVE** | Promoted past soak; serving production traffic |

All 26 modules currently at SPEC. G9 fires when Luke types `start coding workflow-trace`.

---

## Cross-cluster synergy ownership (verification)

Each module declares which CC contracts it OWNS (writes the contract surface) vs CONSUMES (reads it). Verification at [`../ai_docs/optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md`](../ai_docs/optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md).

| CC | Owners | Consumers |
|---|---|---|
| CC-1 | m7 (JSONB hub) | m4, m6, m3 |
| CC-2 | m8, m9, m10, m11 | all clusters |
| CC-3 | m14 | m20, m21, m22, m23 |
| CC-4 | m23 → m30 → m32 | downstream |
| CC-5 | m32 → m40/m41/m42 → m31 read-back | F cluster |
| CC-6 | m33 → m32 | G-internal |
| CC-7 | m15 | operator (CLI, spec interviews) |

---

## Binaries (Wave-16 update — 4 binaries)

| Binary | Owns | Lifecycle | Wave |
|---|---|---|---|
| `wf-crystallise` | m1-m23 + m40-m42 (substrate observation, mining, proposal) | invoke-and-exit CLI | S1003733 |
| `wf-dispatch` | m30-m33 (bank, select, dispatch via Conductor `:8141`, verify) | invoke-and-exit CLI | S1003733 |
| `wf-poller` | m16 + W1 transport + V5 trust (continuous-tick WFE→SX2 wire driver) | operator-launched CLI | S1005032 Wave-15 |
| `wf-daemon` | m16 + W1 + V5 + axum `/health` on `:8142` | habitat-managed (`devenv start`) | S1005032 Wave-16 |

Shared `workflow_core` lib across all four binaries (types, schemas, namespace constants, m1-m42 modules). The habitat-plugin grid renders `wf-daemon` as `WFE` between `Inj` and `ME` in the 14-service row.

Spec: [`WF_DAEMON_HTTP_SHAPE.md`](WF_DAEMON_HTTP_SHAPE.md) · design: [`../ai_docs/WAVE_16_WF_DAEMON_DESIGN_S1005032.md`](../ai_docs/WAVE_16_WF_DAEMON_DESIGN_S1005032.md) · lifecycle: [`../ultramap/WF_DAEMON_LIFECYCLE.md`](../ultramap/WF_DAEMON_LIFECYCLE.md)

---

> **Back to:** [`INDEX.md`](INDEX.md) · [`../README.md`](../README.md) · [`../ARCHITECTURE.md`](../ARCHITECTURE.md) · [`../ai_docs/optimisation-v7/ULTRAMAP.md`](../ai_docs/optimisation-v7/ULTRAMAP.md)
