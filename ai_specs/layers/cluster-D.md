---
title: Cluster D — Trust Aspect Layer (Layer L4, cross-cutting) — Layer Spec
cluster: D
layer: L4
module_count: 4
modules: [m8_povm_build_prereq, m9_watcher_namespace_guard, m10_ember_ci_gate, m11_fitness_weighted_decay]
binary: wf-crystallise (shared lib + build.rs)
feature_gates: [none]
cc_owns: [CC-2 (Trust Layer Woven; aspect surface)]
cc_consumes: []
ship_priority: Day 1 Wave 1 (FIRST — BEFORE Cluster A)
status: SPEC
date: 2026-05-17
hold_v2_compliant: true
---

# Cluster D — Trust Aspect Layer (L4 cross-cutting)

> **Back to:** [`../INDEX.md`](../INDEX.md) · [`../MODULE_MATRIX.md`](../MODULE_MATRIX.md) · [`../../README.md`](../../README.md) · [`../../ARCHITECTURE.md`](../../ARCHITECTURE.md) · vault [[cluster-D-trust-cross-cutting]] · [`../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-D.md`](../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-D.md)

## Role

Cluster D IS **the aspect layer** that wraps every other cluster at a specific lifecycle point. It is **not feature-gated** and **not called** — its modules are *woven into* the engine via build.rs (m8), runtime assertions on write paths (m9), CI gates (m10), and lifecycle hooks (m11). No module imports a Cluster D module as a dependency in the conventional sense; the aspect is *applied to* the rest of the engine. This is the canonical Aspect-Oriented Programming pattern adapted for Rust: aspects are compile-time / write-time / output-time / lifecycle hooks, not method invocations.

The four aspects wrap four distinct lifecycle points. **m8** is the build-time aspect: `build.rs` sets `cargo:rustc-cfg=povm_calibrated` if the calibration env-var is present; downstream modules `#[cfg(povm_calibrated)]`-gate. If the env is absent, compile fails with a `compile_error!` rather than producing a degraded binary (F7 mitigation). **m9** is the write-time aspect: every namespace-bearing write path (m13 stcortex, m42 substrate-feedback) calls `m9::assert_namespace(id)` which runtime-validates the `workflow_trace_*` prefix against `workflow_core::namespace` constants; AP30 enforced. **m10** is the output-time aspect: Ember 7-trait CI gate runs over every user-facing string emitted by m12 (CLI reports) — a Held verdict fails CI; this is Genesis v1.3 § 6 Axis 3 inbound discipline. **m11** is the lifecycle aspect: the freq×fitness×recency compound-decay primitive (Gap 2 new arithmetic, ~200 LOC novel) is called by m30 on engine sweep to mutate `ralph_decay_weight` toward sunset.

## Modules

| Module | Spec | LOC | Tests | Lifecycle point |
|---|---|---:|---:|---|
| `m8_povm_build_prereq` | [`../modules/cluster-D/m8_povm_build_prereq.md`](../modules/cluster-D/m8_povm_build_prereq.md) | 60 | 50 | build-time (build.rs) |
| `m9_watcher_namespace_guard` | [`../modules/cluster-D/m9_watcher_namespace_guard.md`](../modules/cluster-D/m9_watcher_namespace_guard.md) | 60 | 50 | write-time (m13/m42 assert) |
| `m10_ember_ci_gate` | [`../modules/cluster-D/m10_ember_ci_gate.md`](../modules/cluster-D/m10_ember_ci_gate.md) | 90 | 60 | output-time (CI rubric) |
| `m11_fitness_weighted_decay` | [`../modules/cluster-D/m11_fitness_weighted_decay.md`](../modules/cluster-D/m11_fitness_weighted_decay.md) | 90 | 70 | lifecycle (m30 decay tick) |

## Cross-cluster contracts

- **OWNS:**
  - **CC-2 (Trust Layer Woven)** — the entirety of CC-2 lives in Cluster D. Aspect routing is documented at [`../synergies/CC-2.md`](../synergies/CC-2.md). m8 → all clusters at build time; m9 → m13/m42 at write time; m10 → m12 at output time; m11 → m30 at lifecycle time.
- **CONSUMES:** none. Cluster D is a producer of constraints, not a consumer.

## Binary placement

- **m8** runs in `build.rs` for both binaries (`wf-crystallise` + `wf-dispatch`). The cfg is set crate-wide.
- **m9** lives in the shared `workflow_core` library so both binaries link the same namespace-guard implementation.
- **m10** is a CI artefact — a Rust test that runs against pre-commit / pre-merge surfaces; not a runtime binary component.
- **m11** lives in `workflow_core` too; m30 (in `wf-dispatch`) calls it on its decay tick.

## Feature-gate posture

**Un-gated** (`feature = "none"`). Genesis v1.3 § 1 + § 4 lock Cluster D as non-gateable — turning off the trust layer would break F5/F11/AP30 mitigations habitat-wide. This is the structural fact that lets Cluster D ship Day 1 with no `#[cfg(feature = ...)]` complexity.

## Ship priority

**Day 1 Wave 1 — FIRST.** Cluster D ships BEFORE Cluster A. Implementation order: m8 (build-prereq compile-error path) → m9 (namespace-guard constants + assert) → m10 (Ember rubric in `tests/ember/`) → m11 (decay arithmetic). Once these four are merged, Cluster A reads land with the trust layer already asserting on writes. This is the canonical phase-1 framework discipline.

## Operational invariants

1. **m8 hard-fails on missing prereq.** If `WORKFLOW_TRACE_POVM_CALIBRATION` env-var (or equivalent post-ADR renaming) is absent, `build.rs` emits `cargo:warning` AND `compile_error!`. There is no "graceful degrade" pretend-fix branch (F7 / AP-WT-F7 + AP-V7-13).
2. **m9 is the SINGLE namespace-guard implementation.** `workflow_core::namespace::WORKFLOW_TRACE_PREFIX` is the only acceptable string source. `m9::assert_namespace(id)` is called by m13 + m42 + any future writer; the assert is a `Result<(), NamespaceError>` (not panic).
3. **m10 Ember verdicts are output-time, not runtime.** The rubric runs in CI over `target/output-samples/` (generated by `m12 report --sample-pack`); a Held verdict fails CI. m10 never runs in the deployed binary.
4. **m11's decay formula is a single canonical function.** `compute_decay_factor(base, frequency, fitness, recency) -> DecayFactor` is the only mutator; it accepts four `[0.0, 1.0]` clamped inputs and returns a `[0.0, 1.0]` clamped output. The formula `base + (1 - base) * (f * g * r).clamp(0, 1)` is the canonical form locked in Genesis v1.3 § 5.
5. **The aspect surface is never bypassed via runtime fallback.** If m9 returns `NamespaceError::PrefixMismatch`, m13 returns `Err` to caller; the caller does NOT retry under a different prefix.

## Failure modes the cluster structurally refuses

- **m8 "if env absent, emit warning and continue".** This is the F7 pattern that landed the POVM `:8125` health-200-but-pre-CR-2-binary trap (ADR 2026-05-17). m8 hard-fails build instead.
- **m9 prefix relaxation under load.** Adding a "compatibility prefix" for legacy namespaces is rejected; namespaces are versioned forward only.
- **m10 sycophancy.** The Ember rubric specifically blocks "Held but suggested PASS with minor amend" verdicts — Held is Held; the spec changes, not the rubric. See `feedback_sycophancy_mitigation.md`.
- **m11 decay-factor short-circuit.** A "skip decay if no dispatches in 30 days" branch is rejected — the decay formula must be called every sweep tick; idle workflows decay toward sunset regardless.

## Performance envelope

| Operation | Target | Notes |
|---|---|---|
| m8 build-time check | < 100 ms | env-var read + cfg emit; runs once per `cargo check` |
| m9 `assert_namespace` | < 1 µs | const-string prefix check; no allocation |
| m10 Ember rubric pass | < 5 s | runs over ~100 output samples |
| m11 `compute_decay_factor` | < 200 ns | pure arithmetic, `mul_add` for FMA |
| m11 `apply_decay_tick` (m30 batch) | < 50 ms (1000 rows) | single UPDATE statement; m11 does the math, m30 does the SQL |

## Verify-sync invariants

- **#1** — `src/m8_povm_build_prereq/` (or `build.rs` if m8 is build-only), `src/m9_watcher_namespace_guard/`, `src/m10_ember_ci_gate/`, `src/m11_fitness_weighted_decay/` all exist post-G9.
- **#7** — `rg '\bunsafe\b' src/m8*/ src/m9*/ src/m10*/ src/m11*/` returns 0.
- **#15** — m9 is the only writer of `workflow_trace_*` strings; verified by `rg '"workflow_trace_' src/ | grep -v 'm9_watcher_namespace_guard/'` returning 0.
- **#16** — every AP-WT-F* failure mode has a Cluster D test asserting refusal: F7 (m8 build-fail), F11 (m9 string-literal ban), F5 (m11 decay-monotonic).

## Per-module cross-links

- [`../modules/cluster-D/m8_povm_build_prereq.md`](../modules/cluster-D/m8_povm_build_prereq.md) — `cargo:rustc-cfg=povm_calibrated` build aspect
- [`../modules/cluster-D/m9_watcher_namespace_guard.md`](../modules/cluster-D/m9_watcher_namespace_guard.md) — AP30 write-time prefix guard
- [`../modules/cluster-D/m10_ember_ci_gate.md`](../modules/cluster-D/m10_ember_ci_gate.md) — 7-trait Ember output gate
- [`../modules/cluster-D/m11_fitness_weighted_decay.md`](../modules/cluster-D/m11_fitness_weighted_decay.md) — Gap 2 freq×fitness×recency decay primitive

## Antipatterns specific to Cluster D

- **AP-WT-F7** (CR-2 graceful-degrade) — m8 hard-fails on missing prereq.
- **AP30** (namespace string drift) — m9 is the structural mitigation.
- **AP-V7-13** (diagnostics theatre — health-200 ≠ behaviour) — m8 is the architectural lesson from this antipattern; m9/m10 the prevention surface.
- **AP-V7-09** (substrate-frame confusion) — m11's decay primitive is substrate-grain (per-pathway), not anthropocentric (per-deploy).

---

> **Back to:** [`../INDEX.md`](../INDEX.md) · [`../MODULE_MATRIX.md`](../MODULE_MATRIX.md) · [`../../README.md`](../../README.md) · vault [[cluster-D-trust-cross-cutting]]
