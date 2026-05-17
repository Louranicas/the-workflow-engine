# ARCHITECTURE — workflow-trace

> **Canonical:** [`ai_docs/optimisation-v7/ULTRAMAP.md`](ai_docs/optimisation-v7/ULTRAMAP.md) (View 1 = layer; View 2 = module table) · [`ai_docs/GENESIS_PROMPT_V1_3.md`](ai_docs/GENESIS_PROMPT_V1_3.md) § 1 · [`the-workflow-engine-vault/Modules Synergy Clusters and Feature Verification S1001982.md`](the-workflow-engine-vault/Modules%20Synergy%20Clusters%20and%20Feature%20Verification%20S1001982.md)
> **This file:** stable structural summary; does NOT duplicate canonical; bridges them.
> **Status:** locked under v1.3 binding spec (awaiting Zen G7 verdict on amendment-only delta).

---

## 26 modules · 8 clusters · 9 layers (L0-L8) · 2 binaries + shared lib

### Cluster table

| Cluster | Layer | Modules | LOC | Role | Canonical |
|---|---|---|---:|---|---|
| **A** Substrate Ingest | L1 | m1, m2, m3 | ~230 | atuin / stcortex narrowed-scope / injection.db | [cluster-A](ai_specs/modules/cluster-A/) |
| **B** Habitat Observation | L2 | m4, m5, m6 | ~460 | cascade correlator (opaque FNV-1a XOR IDs) / battern step record / context cost (F10 EMA) | [cluster-B](ai_specs/modules/cluster-B/) |
| **C** Correlation + Output | L3 | m7, m12, m13 | ~370 | hub `workflow_runs` (F9 zero-weight) / CLI reports / stcortex writer (3-band gate) | [cluster-C](ai_specs/modules/cluster-C/) |
| **D** Trust (cross-cutting) | L4 | m8, m9, m10, m11 | ~300 | povm_calibrated cfg / namespace guard / Ember CI gate / fitness-weighted decay | [cluster-D](ai_specs/modules/cluster-D/) |
| **E** Evidence + Pressure | L5 | m14, m15 | ~200 | Wilson-CI lift metric / pressure register (JSONL) | [cluster-E](ai_specs/modules/cluster-E/) |
| **F** Iteration (KEYSTONE) | L6 | m20, m21, m22, m23 | ~850 | PrefixSpan miner / variant builder / Wilson CI / gradient-preservation proposer | [cluster-F](ai_specs/modules/cluster-F/) |
| **G** Bank/Select/Dispatch/Verify | L7 | m30, m31, m32, m33 | ~950 | curated bank (EscapeSurfaceProfile) / selector / dispatcher / 4-agent verifier | [cluster-G](ai_specs/modules/cluster-G/) |
| **H** Substrate Feedback | L8 | m40, m41, m42 | ~450 | NexusEvent emit / LCM RPC / stcortex emit (POVM decoupled per 2026-05-17 ADR) | [cluster-H](ai_specs/modules/cluster-H/) |
| **Total** | L0-L8 | **26** | **~3,810** | + ~1,562 tests; ≈ ~5,200 LOC incl. manifests / build.rs / integration / binaries | |

### L0 (substrate frame) / L9 (reserved)

- **L0** = the substrate frame itself (atuin / stcortex / injection.db / SYNTHEX / LCM / Conductor / Watcher) — observed, not authored.
- **L9** = substrate-frame engine — **intentionally absent**; single-phase override partially waived R6 frame-separation; L9 placeholder reserved for post-D120 evaluation.

---

## Binary split (locked, GENESIS v1.3 § 1)

| Binary | Modules owned | Notes |
|---|---|---|
| **`wf-crystallise`** | m1-m23, m40-m42 | Read-heavy: ingest + observation + correlation + iteration + substrate feedback emit |
| **`wf-dispatch`** | m30-m33 | Curated bank + selection + Conductor dispatcher + 4-agent verifier |
| **`workflow_core`** (lib) | — | Shared types, schemas, namespace constants (AP30), errors; **inside same Cargo crate**, NOT a separate workspace member (ORAC pattern, not LCM 10-crate workspace) |

### Feature gate matrix (Cargo)

```toml
[features]
default = ["full"]
full         = ["api", "intelligence", "monitoring", "evolution"]
api          = []   # m12 reports, m32 dispatch CLI surface
intelligence = []   # m14/m15 evidence + m20-m23 iteration
monitoring   = []   # m40/m41/m42 substrate feedback
evolution    = []   # m11 lifecycle/sunset decay
```

**Cluster D is NOT feature-gated** — aspect-layer invariants that every other module routes through. m8's `cargo:rustc-cfg=povm_calibrated` is env-only (not a Cargo feature) so it cannot be bypassed by `--features full`.

---

## Cross-cluster synergy contracts (CC-1..CC-7)

| ID | Contract | Path | Canonical |
|---|---|---|---|
| **CC-1** | Cascade-Cost Coupling (B-internal via m7 JSONB) | m4 ↔ m6 via m7 `consumer_inputs` | [ai_specs/synergies/CC-1.md](ai_specs/synergies/CC-1.md) |
| **CC-2** | Trust Layer Woven (D → all) | m8/m9/m10/m11 → every other cluster | [ai_specs/synergies/CC-2.md](ai_specs/synergies/CC-2.md) |
| **CC-3** | Evidence-Driven Iteration (E → F) | m14/m15 → m20-m23 iterator inputs | [ai_specs/synergies/CC-3.md](ai_specs/synergies/CC-3.md) |
| **CC-4** | Proposal→Bank→Dispatch Pipeline | m23 → m30 → m32 → HABITAT-CONDUCTOR | [ai_specs/synergies/CC-4.md](ai_specs/synergies/CC-4.md) |
| **CC-5** | Substrate Learning Loop (G→H→F) | m32 → m40/m41/m42 → back to m31 via stcortex pathway weights | [ai_specs/synergies/CC-5.md](ai_specs/synergies/CC-5.md) |
| **CC-6** | Verification-Gated Dispatch (G-internal) | m33 → m32 (5-check pre-dispatch sequence) | [ai_specs/synergies/CC-6.md](ai_specs/synergies/CC-6.md) |
| **CC-7** | Pressure-Driven Evolution (E → spec interviews) | m15 PHASE-B-RESERVATION-NOTICE → operator | [ai_specs/synergies/CC-7.md](ai_specs/synergies/CC-7.md) |

Full contract verification table: [`ai_docs/optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md`](ai_docs/optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md).

---

## Structural-gap authorship (cannot be lifted; net-new)

| Gap | Owner | Spec |
|---|---|---|
| **Gap 1** N-step compositional sub-graph detection | F (m20 + m23) | PrefixSpan + Levenshtein similarity + Wilson CI; ~600-1,000 LOC (KEYSTONE) |
| **Gap 2** `frequency × fitness × recency` compound decay | D (m11) | NEW PRIMITIVE formula: `base_rate + (1 − base_rate) × clamp(f×fit×r, 0, 1)`; ~200-300 LOC |
| **Gap 3** Unified destructiveness / EscapeSurfaceProfile schema (cardinality **7** per D-S1002127-02) | G (m30 + m32) + D (m9) | Ordinal enum + display-before-step + namespace guard; ~150-250 LOC; variants Sandboxed/SandboxEscape/ProcessMutate/PrivilegeEscalation/FileWrite/NetworkEgress/DataExfil at ordinals 0/10/20/30/40/50/60 (gap-reserved) |

---

## Canonical src/ layout (G9 unlock target)

Per [`ai_docs/optimisation-v7/GENERATIONS/G2-consolidation.md`](ai_docs/optimisation-v7/GENERATIONS/G2-consolidation.md) § Canonical src/ layout:

```
src/
├── lib.rs                       # workflow_core public exports
├── types.rs                     # shared types (newtypes: SessionId, ConsumerId, …)
├── schemas.rs                   # JSON schemas (NexusEvent, EscapeSurfaceProfile, …)
├── namespace.rs                 # AP30 constants (workflow_trace_*)
├── errors.rs                    # thiserror taxonomy
├── m1_atuin_consumer/           # Cluster A
├── m2_stcortex_consumer/
├── m3_injection_db_consumer/
├── m4_cascade_correlator/       # Cluster B
├── m5_battern_step_record/
├── m6_context_cost/
├── m7_workflow_runs/            # Cluster C
├── m12_cli_reports/
├── m13_stcortex_writer/
├── m8_povm_build_prereq/        # Cluster D (cross-cutting)
├── m9_watcher_namespace_guard/
├── m10_ember_ci_gate/
├── m11_fitness_weighted_decay/
├── m14_habitat_outcome_lift/    # Cluster E
├── m15_pressure_register/
├── m20_prefixspan_miner/        # Cluster F (KEYSTONE)
├── m21_variant_builder/
├── m22_kmeans_feature/
├── m23_workflow_proposer/
├── m30_curated_bank/            # Cluster G
├── m31_selector/
├── m32_conductor_dispatcher/
├── m33_verifier/
├── m40_nexusevent_emit/         # Cluster H
├── m41_lcm_rpc/
└── m42_stcortex_emit/
```

**Module naming:** unpadded `m1`-`m42` throughout. NOT `m01`-`m42`. (OI-4 resolved.)

---

> **Back to:** [`README.md`](README.md) · [`CLAUDE.md`](CLAUDE.md) · [`CLAUDE.local.md`](CLAUDE.local.md) · [`GATE_STATE.md`](GATE_STATE.md) · [`ai_specs/INDEX.md`](ai_specs/INDEX.md) · [`ai_docs/INDEX.md`](ai_docs/INDEX.md) · [`ultramap/README.md`](ultramap/README.md)
