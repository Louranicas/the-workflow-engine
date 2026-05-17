# src/ — PLACEHOLDER (no source files pre-G9)

> **Status:** EMPTY by design. No `.rs` files exist here until Luke types `start coding workflow-trace` (G9 unlock).
> **Why empty:** HOLD-v2 envelope + scaffold-only S1002127 waiver (see [`../PRIME_DIRECTIVE_WAIVER.md`](../PRIME_DIRECTIVE_WAIVER.md)) explicitly forbid `.rs` authoring pre-G9.
> **G9 unlock target layout:** see [`../ARCHITECTURE.md`](../ARCHITECTURE.md) § Canonical src/ layout · [`../ai_docs/optimisation-v7/GENERATIONS/G2-consolidation.md`](../ai_docs/optimisation-v7/GENERATIONS/G2-consolidation.md) § Canonical src/ layout

---

## What lands here at G9-fire

Per [`../plan.toml`](../plan.toml) and ORAC single-crate pattern:

```
src/
├── lib.rs                       # workflow_core public exports
├── types.rs                     # shared types (newtypes: SessionId, ConsumerId, …)
├── schemas.rs                   # JSON schemas (NexusEvent, EscapeSurfaceProfile, …)
├── namespace.rs                 # AP30 constants (workflow_trace_*)
├── errors.rs                    # thiserror taxonomy
├── bin/
│   ├── wf-crystallise/main.rs   # binary 1
│   └── wf-dispatch/main.rs      # binary 2
├── m1_atuin_consumer/           # Cluster A
├── m2_stcortex_consumer/
├── m3_injection_db_consumer/
├── m4_cascade_correlator/       # Cluster B
├── m5_battern_step_record/
├── m6_context_cost/
├── m7_workflow_runs/            # Cluster C
├── m12_cli_reports/
├── m13_stcortex_writer/
├── m8_povm_build_prereq/        # Cluster D (cross-cutting; ships Day 1)
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
└── m42_stcortex_emit/           # POVM DECOUPLED per 2026-05-17 ADR
```

---

## Per-module spec lives at `ai_specs/modules/cluster-<X>/m<N>_<name>.md`

When G9 fires:
1. Read the spec for the module you're about to author
2. Implement the public surface declared in the spec
3. Write tests per the spec's test-kind allocation (50+ min, KEYSTONE F: 250+)
4. Run 4-stage gate (check → clippy → pedantic → test) before commit
5. Update [`../ai_specs/MODULE_MATRIX.md`](../ai_specs/MODULE_MATRIX.md) status: SPEC → WIP → BUILT

---

## Anti-pattern: do NOT create `.rs` files pre-G9

Even an empty `mod.rs` violates HOLD-v2. The presence of a single `.rs` file gives any subagent/hook permission to start `cargo build`, which closes the planning-only envelope before Luke is ready. See [`../ANTIPATTERNS.md`](../ANTIPATTERNS.md) AP-V7-12.

---

> **Back to:** [`../README.md`](../README.md) · [`../ARCHITECTURE.md`](../ARCHITECTURE.md) · [`../plan.toml`](../plan.toml) · [`../PRIME_DIRECTIVE_WAIVER.md`](../PRIME_DIRECTIVE_WAIVER.md)
