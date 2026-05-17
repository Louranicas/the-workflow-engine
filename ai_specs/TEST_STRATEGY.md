---
title: TEST_STRATEGY — per-cluster test allocation + KEYSTONE Cluster F bench/property/fuzz
date: 2026-05-17
status: SPEC
total_test_budget: 1594 (TEST_DISCIPLINE matrix) / 1562-1599 range tolerated
test_families: [F-Unit, F-Property, F-Fuzz, F-Integration, F-Contract, F-Regression, F-Mutation]
---

# TEST_STRATEGY — workflow-trace

> **Back to:** [`INDEX.md`](INDEX.md) · [`MODULE_MATRIX.md`](MODULE_MATRIX.md) · [`../README.md`](../README.md) · [`../ARCHITECTURE.md`](../ARCHITECTURE.md) · canonical [`../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md`](../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md)

## Total budget

**1,594 tests across 26 modules** per [`../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md`](../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md) § Test-count allocation matrix. The figure varies across V7 docs (1,562 / 1,594 / 1,599); the canonical row is 1,594 (G6 latest with mutation allocation). Surplus absorbs additions during implementation.

## 7 test-pattern families

| Family | Purpose | Tooling | Min/module |
|---|---|---|---|
| **F-Unit** | Pure function correctness; per-arm enum coverage | `#[test]` | 20 |
| **F-Property** | Invariants over generated inputs | `proptest` (≥10k iters) | 5 |
| **F-Fuzz** | Parsers / decoders / wire-format robustness | `cargo fuzz` nightly (24h budget) | 1 per parser |
| **F-Integration** | Module ↔ peer composition; real local services | `#[tokio::test]` | 10 |
| **F-Contract** | Bridge / wire / schema parity | `insta::assert_yaml_snapshot!` | 5 |
| **F-Regression** | Each fixed bug → reproducer test | `#[test]` + bug-id | 1+ per bug; 3 reserved |
| **F-Mutation** | Mutation kill-rate ≥70% | `cargo mutants` | 1 mutation budget |

Per [`../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md`](../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md) § Meaningful-test discipline: a test is meaningful iff it satisfies all of (single-assertion semantic, behaviour-describing name, rationale comment, fast, hermetic, self-validating, falsifiable).

## Per-cluster allocation (totals)

| Cluster | Modules | Total tests | Notes |
|---|---|---:|---|
| A | m1, m2, m3 | 150 | 50 per module; straight-forward read-paths |
| B | m4, m5, m6 | 170 | 60/55/55; F11 opaque-id property tests in m4 |
| C | m7, m12, m13 | 190 | 70/60/60; m7 contract tests heaviest |
| D | m8, m9, m10, m11 | 230 | 50/50/60/70; m11 property tests on decay arithmetic |
| E | m14, m15 | 130 | 60/50 + 10 surplus; m14 Wilson-CI property tests |
| F | m20, m21, m22, m23 | **290** (KEYSTONE) | **m20=90** (bench + fuzz + property heavy); m21/22/23 = 70/60/60 |
| G | m30, m31, m32, m33 | 290 | 70/70/80/70; m32 has 5 dispatch-refusal integration tests |
| H | m40, m41, m42 | 195 | 65/65/70; m42 zero-POVM audit |

**Total:** 1,594 (matches matrix).

## KEYSTONE Cluster F — special depth

m20 PrefixSpan is the largest test surface (90 tests + criterion bench + 24h cargo-fuzz nightly):

- 45 F-Unit (per-arm coverage; 3 per public fn × 15 fns)
- 10 F-Property (algorithm invariants — gap-bounds, monotonic-frequency, idempotent-repeat)
- 1 F-Fuzz target (input: arbitrary StepToken sequences; 24h budget; crash gates merge)
- 20 F-Integration (m20 ↔ m4/m5/m14 upstream wiring; m20 ↔ m23 downstream)
- 5 F-Contract (output schema for ProposalBuilder)
- 5 F-Regression (5 bug-classes pre-seeded from spec)
- 1 F-Mutation budget (≥70% kill required)

m23 has the **construction-time gate** (F-CC-3 mitigation) — 5 contract tests pin the `LiftEvidenceMissing` path; integration tests verify m14 → m20 → m21 → m22 → m23 → m30 round-trip.

## Property test patterns (≥10k iters)

```rust
proptest! {
    #![proptest_config(ProptestConfig { cases: 10_000, ..ProptestConfig::default() })]

    #[test]
    fn m11_decay_factor_bounded(base in 0.0f64..=1.0, f in 0.0f64..=1.0, g in 0.0f64..=1.0, r in 0.0f64..=1.0) {
        let result = compute_decay_factor(base, f, g, r);
        prop_assert!(result >= 0.0 && result <= 1.0);
    }

    #[test]
    fn m14_lift_none_below_n_20(success in 0u32..20, n_below in 0u32..20) {
        let total = success + n_below;
        prop_assume!(total < 20);
        let lift = m14::compute_lift(success, total).unwrap();
        prop_assert!(lift.is_none());
    }
}
```

## Integration test discipline

Real services where required; mocks of internals never. Per [`synergies/`](synergies/) § Closure-test inventory:

| CC | Test | Live services |
|---|---|---|
| CC-1 | `tests/integration/cc1_cascade_cost_coupling.rs` | none |
| CC-2 | `tests/integration/cc2_trust_aspect_routing.rs` | none |
| CC-3 | `tests/integration/cc3_evidence_driven_iteration.rs` | none |
| CC-4 | `tests/integration/cc4_proposal_bank_dispatch.rs` | Conductor :8141 (`#[ignore]` until B3) |
| CC-5 | `tests/integration/cc5_substrate_learning_loop.rs` | synthex-v2 + povm-v2 + Conductor (`#[ignore]` until live) |
| CC-6 | `tests/integration/cc6_verification_gated_dispatch.rs` | Conductor :8141 |
| CC-7 | `tests/integration/cc7_pressure_driven_evolution.rs` | none |

## Contract test discipline

`insta::assert_yaml_snapshot!` for every wire envelope, every metric name set, every event family. Snapshots reviewed + approved at v1.3-G7 spec audit. Drift fails CI.

```rust
#[test]
fn m42_pathway_envelope_matches_snapshot() {
    let env = m42::build_envelope(test_outcome());
    insta::assert_yaml_snapshot!(env);
}
```

## Mutation test discipline

Per module: `cargo mutants --package workflow_trace --regex 'm{N}::.*'`. Pass criterion ≥70% kill (≥75% for KEYSTONE m20, m30, m32, m33). Surviving mutations require either added test or `// IGNORE: cosmetic only` rationale annotation.

**Schedule:** weekly Wave-end. NOT per-commit.

## CI gate (tests every commit)

```bash
# Fast every-commit gate (target: < 5 min):
CARGO_TARGET_DIR=./target cargo test --workspace --all-targets --all-features --release 2>&1 | tail -30

# Wave-end (slow):
cargo mutants --package workflow_trace -- --release
cargo +nightly fuzz run all_targets -- -max_total_time=3600
```

## Test naming convention

```rust
#[test]
fn {subject}_{behaviour}_{condition_or_scenario}() { ... }

// GOOD:
fn prefixspan_emits_no_pattern_when_input_below_n20() { ... }   // F-WT-F2
fn dispatcher_refuses_when_conductor_health_returns_503() { ... } // F-WT-F4

// BAD:
fn test_1() { ... }
fn it_works() { ... }
fn dispatcher_handles_edge_cases() { ... }  // vague — refactor
```

## Rationale comment per test

Every test has a `// rationale:` comment linking to spec section / bug ID / failure mode:

```rust
#[test]
fn m30_auto_promote_refused_when_not_interactive() {
    // rationale: F5 mitigation per AP-V7-07
    // ...
}
```

## Verify-sync invariants

- **#4** — every module has ≥ minimum test count (50 baseline; 60+/70+/90 per matrix).
- **#16** — every AP-WT-F* failure mode has ≥1 test asserting mitigation.
- **#17** — every CC-1..CC-7 synergy has integration test.

---

> **Back to:** [`INDEX.md`](INDEX.md) · canonical [`../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md`](../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md)
