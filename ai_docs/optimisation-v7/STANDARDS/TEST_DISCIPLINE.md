---
title: TEST DISCIPLINE — workflow-trace V7 (≥50 tests/module, top-1% patterns)
date: 2026-05-17 (S1001982)
kind: planning-only · test doctrine
purpose: define the "≥50 meaningful tests/module" recipe + 7 test-pattern families + mutation budget
target: 1,562 tests across 26 modules (avg 60/module; min 50; KEYSTONE m20 = 90)
---

# TEST DISCIPLINE — workflow-trace V7

> Back to: [[../TASK_LIST_V7_OPTIMISATION.md]] · [[GOD_TIER_RUST.md]] · [[../KEYWORDS_20.md]]
>
> Every module ≥50 meaningful tests. Meaningful = each test asserts a behaviour that would matter if it broke. Not coverage theatre.

---

## The 7 test-pattern families (every module must cover at least 5)

| Family | Purpose | Tooling | Min per module |
|---|---|---|---|
| **F-Unit** | Pure function correctness; per-arm enum coverage | `#[test]` | 20 |
| **F-Property** | Invariants over generated inputs (input space coverage) | `proptest` | 5 (≥10k iters each) |
| **F-Fuzz** | Parsers / decoders / wire-format robustness | `cargo fuzz` (nightly) | 1 per parser (24h budget) |
| **F-Integration** | Module ↔ peer module composition; real local services | `#[tokio::test]` with testcontainers OR live local | 10 |
| **F-Contract** | Bridge / wire / schema parity | `assert_yaml_snapshot!` + `bridge-contract` skill | 5 |
| **F-Regression** | Each fixed bug → reproducer test | `#[test]` + bug-id comment | 1+ per fixed bug; min 3 reserved |
| **F-Mutation** | Mutation-test kill-rate ≥70% | `cargo mutants` | 1 mutation budget per module (≥70% kill) |

**Why these 7:** unit catches per-line correctness; property catches input-space gaps; fuzz catches adversarial; integration catches composition; contract catches drift; regression prevents fix-rot; mutation validates the OTHER six aren't theatre.

---

## ≥50 tests per module — exact allocation (typical module)

```
m20 PrefixSpan (KEYSTONE, 90 tests)
├── F-Unit       45 tests (per-arm coverage; 3 per public fn × 15 fns)
├── F-Property   10 tests (algorithm-invariants — gap-bounds, monotonic-frequency, idempotent-repeat)
├── F-Fuzz        1 target (input: arbitrary StepToken seq; 24h budget; crash gates merge)
├── F-Integration 20 tests (m20 ↔ m4/m5/m14 upstream wiring; m20 ↔ m23 downstream)
├── F-Contract    5 tests (output schema for ProposalBuilder)
├── F-Regression  5 tests (reserved; 5 bug-classes pre-seeded from spec)
├── F-Mutation    1 budget (≥70% kill required for merge)
└── Total        90 tests (KEYSTONE complexity allowance)

m7 central (60 tests)
├── F-Unit       30
├── F-Property    5 (schema invariants — fitness_dimension never null)
├── F-Fuzz        1 (JSONB blob roundtrip)
├── F-Integration 15 (every consumer reads/writes correctly)
├── F-Contract    3 (schema version stability)
├── F-Regression  3
├── F-Mutation    1 budget
└── Total        60 tests
```

(Per-module allocation matrices live in MODULE_PLANS/cluster-*.md.)

---

## Meaningful-test discipline (top-1% pattern doctrine)

A test is **meaningful** iff it satisfies ALL:

1. **Single assertion semantic** — one behaviour per test; multiple assertions OK only if they describe the same behaviour from different angles.
2. **Name describes behaviour, not implementation** — `dispatcher_refuses_when_conductor_unreachable` ✅; `test_m32_dispatch_5` ❌.
3. **Rationale comment** — `// rationale: F4 mitigation per AP-WT-F4`. Or links to spec section / bug ID / failure mode.
4. **Fast** — unit test < 100ms; integration < 2s; if slower, justified comment + `#[ignore]` for opt-in.
5. **Hermetic** — no order dependence; no shared mutable state; parallel-safe.
6. **Self-validating** — exact expected output; never "no panic = pass" for non-panic-targeted tests.
7. **Falsifiable** — must be possible to break the test by breaking the behaviour; otherwise it's tautological.

**Anti-pattern catalogue:** see ANTIPATTERNS_REGISTER.md § Class AP-Test.

---

## Test naming convention (BDD-flavoured Rust)

```rust
#[test]
fn {subject}_{behaviour}_{condition_or_scenario}() { ... }

// Examples (good):
#[test]
fn prefixspan_emits_no_pattern_when_input_below_n20() { ... }    // F-WT-F2
#[test]
fn dispatcher_refuses_when_conductor_health_returns_503() { ... }  // F-WT-F4
#[test]
fn decay_factor_never_exceeds_one_for_any_finite_input() { ... }   // F-Property invariant
```

```rust
// Examples (bad):
#[test]
fn test_1() { ... }                       // no semantic
#[test]
fn it_works() { ... }                     // smoke only
#[test]
fn dispatcher_handles_edge_cases() { ... } // vague — refactor into 3 named cases
```

---

## Property-test patterns (≥10k iters per property)

```rust
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 10_000,
        max_shrink_iters: 1000,
        ..ProptestConfig::default()
    })]

    /// rationale: m11 decay-factor mathematical invariant
    /// per Gap 2 formula: base + (1-base)*clamp(f*g*r, 0, 1)
    #[test]
    fn decay_factor_bounded_zero_to_one(
        base in 0.0f64..=1.0,
        f in 0.0f64..=1.0,
        g in 0.0f64..=1.0,
        r in 0.0f64..=1.0,
    ) {
        let result = compute_decay_factor(base, f, g, r);
        prop_assert!(result >= 0.0 && result <= 1.0);
    }

    #[test]
    fn decay_factor_monotonic_in_frequency(
        base in 0.0f64..=1.0,
        g in 0.1f64..=1.0,
        r in 0.1f64..=1.0,
        f1 in 0.0f64..=0.5,
        f2 in 0.5f64..=1.0,
    ) {
        prop_assume!(f1 < f2);
        let d1 = compute_decay_factor(base, f1, g, r);
        let d2 = compute_decay_factor(base, f2, g, r);
        prop_assert!(d2 >= d1, "decay must be non-decreasing in frequency");
    }
}
```

---

## Fuzz-test pattern (cargo-fuzz nightly)

```rust
// fuzz_targets/m4_cascade_id_fuzz.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use workflow_core::m4_cascade::derive_cluster_id;

fuzz_target!(|data: &[u8]| {
    // rationale: F11 opaque-ID invariant — derive_cluster_id must never panic
    // and must never return an ID containing literal pane-label substrings
    if let Ok(s) = std::str::from_utf8(data) {
        let id = derive_cluster_id(s.to_string());
        assert!(!id.to_string().contains("ALPHA"));
        assert!(!id.to_string().contains("BETA"));
        assert!(!id.to_string().contains("GAMMA"));
    }
});
```

Run command (CI nightly job, NOT every commit):
```bash
cargo +nightly fuzz run m4_cascade_id_fuzz -- -max_total_time=86400
```

---

## Integration-test pattern (real services, no mocks-of-internals)

```rust
// tests/integration/cc5_substrate_learning_loop.rs
//
// rationale: CC-5 first-closure verification per Phase 3 Day 26 milestone
// Asserts: m32 dispatch → m40 emit → SYNTHEX v2 :8092/v3/nexus/push → learning_health delta observable
//
// Pre-req: synthex-v2 running locally (devenv start synthex-v2); povm-v2 running (:8125)
// Time budget: 10s
// Failure-mode: skipped with #[ignore] if synthex-v2 unreachable

#[tokio::test]
#[ignore = "requires live synthex-v2 + povm-v2"]
async fn cc5_dispatch_emits_observable_substrate_delta() -> anyhow::Result<()> {
    // pre-flight: probe synthex-v2
    let ok = reqwest::get("http://localhost:8092/health").await?.status().is_success();
    anyhow::ensure!(ok, "synthex-v2 not reachable; integration test cannot run");

    // capture baseline
    let pre = povm_learning_health().await?;

    // dispatch test workflow
    let result = m32::dispatch_via_conductor(test_workflow()).await?;
    assert!(matches!(result, DispatchOutcome::PassVerified | DispatchOutcome::Pass));

    // wait for substrate propagation (max 2s)
    tokio::time::sleep(Duration::from_secs(2)).await;

    // observe delta
    let post = povm_learning_health().await?;
    assert!(post > pre, "CC-5 loop did not move substrate (Class-I would fire)");

    Ok(())
}
```

---

## Contract-test pattern (schema parity)

```rust
// tests/contract/m40_synthex_emit_schema.rs
//
// rationale: AP-Drift-06 mitigation — outbox JSONL schema must match SYNTHEX v2 accepted format

use insta::assert_yaml_snapshot;

#[test]
fn m40_outbox_event_matches_synthex_v3_nexus_push_schema() {
    let event = m40::WorkflowEvent::Run {
        id: WorkflowId("test-001".to_string()),
        outcome: Outcome::PassVerified,
        fitness_delta: 0.25,
        fields: serde_json::json!({"timestamp": "2026-05-17T00:00:00Z"}),
    };
    let serialized = serde_json::to_value(&event).unwrap();
    assert_yaml_snapshot!(serialized);
    // Snapshot stored in tests/contract/snapshots/
    // Reviewed and approved at v1.3-G7 spec audit
}
```

---

## Regression-test pattern (every bug → test)

```rust
// tests/regression/bug_wt_001_dispatcher_panicked_on_empty_bank.rs
//
// rationale: regression for bug WT-001 (hypothetical) — m32 panicked when m30 returned empty bank
// Fixed at commit XXXX (post-G9)
// Failure-mode this test catches: empty-bank → panic instead of refuse-mode

#[tokio::test]
async fn wt_001_dispatcher_returns_refuse_mode_on_empty_bank() {
    let empty_bank = m30::Bank::empty();
    let dispatcher = m32::Dispatcher::new(empty_bank);
    let result = dispatcher.attempt_dispatch().await;
    assert!(matches!(result, Err(DispatchError::EmptyBank)));
}
```

---

## Mutation-test discipline

```bash
# Per module:
cargo mutants --package workflow_trace --regex 'm20::.*'

# Pass criterion: ≥70% mutations caught (killed)
# Surviving mutations require either:
#   (a) added test catching the mutation, OR
#   (b) `// IGNORE: cosmetic only` annotation with rationale
```

**Schedule:** weekly Wave-end. Slow operation — not per-commit.

---

## Test-count allocation matrix (all 26 modules)

| Module | F-Unit | F-Property | F-Fuzz | F-Integration | F-Contract | F-Regression | F-Mutation | **Total** |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| m1 atuin | 25 | 5 | 0 | 15 | 3 | 2 | 1 | 50 |
| m2 stcortex_consumer | 25 | 5 | 0 | 15 | 3 | 2 | 1 | 50 |
| m3 injection.db | 25 | 5 | 0 | 15 | 3 | 2 | 1 | 50 |
| m4 cascade | 30 | 8 | 1 | 15 | 3 | 3 | 1 | 60 |
| m5 battern | 28 | 6 | 0 | 15 | 3 | 3 | 1 | 55 |
| m6 cost | 28 | 6 | 0 | 15 | 3 | 3 | 1 | 55 |
| m7 central | 30 | 5 | 1 | 18 | 5 | 3 | 1 | 70 |
| m8 build-prereq | 25 | 5 | 0 | 15 | 3 | 2 | 1 | 50 |
| m9 namespace guard | 25 | 5 | 0 | 15 | 3 | 2 | 1 | 50 |
| m10 Ember CI | 30 | 5 | 0 | 18 | 5 | 2 | 1 | 60 |
| m11 decay | 35 | 10 | 0 | 15 | 5 | 4 | 1 | 70 |
| m12 CLI reports | 25 | 5 | 0 | 15 | 3 | 2 | 1 | 50 |
| m13 stcortex writer | 30 | 5 | 0 | 15 | 5 | 4 | 1 | 60 |
| m14 lift | 30 | 8 | 0 | 15 | 4 | 3 | 1 | 60 |
| m15 pressure | 25 | 5 | 0 | 15 | 3 | 2 | 1 | 50 |
| m20 PrefixSpan | 45 | 10 | 1 | 20 | 5 | 8 | 1 | 90 |
| m21 variant builder | 35 | 10 | 0 | 15 | 5 | 4 | 1 | 70 |
| m22 K-means | 30 | 8 | 0 | 15 | 3 | 3 | 1 | 60 |
| m23 proposer | 30 | 5 | 0 | 15 | 5 | 4 | 1 | 60 |
| m30 bank | 35 | 5 | 0 | 15 | 5 | 9 | 1 | 70 |
| m31 selector | 35 | 8 | 0 | 15 | 5 | 6 | 1 | 70 |
| m32 dispatcher | 40 | 5 | 0 | 20 | 5 | 9 | 1 | 80 |
| m33 verifier | 35 | 5 | 0 | 18 | 5 | 6 | 1 | 70 |
| m40 SYNTHEX emit | 30 | 5 | 0 | 15 | 5 | 4 | 1 | 60 |
| m41 LCM router | 30 | 5 | 0 | 15 | 5 | 4 | 1 | 60 |
| m42 POVM dual-path | 30 | 5 | 0 | 15 | 5 | 4 | 1 | 60 |
| **TOTAL** | **786** | **162** | **3** | **412** | **108** | **97** | **26** | **1,594** |

**Hits target ≥1,300; aim ~1,562; actual budget 1,594.** Surplus absorbs additions during implementation.

---

## CI gate (tests every commit, mutation every Wave-end)

```bash
# Every commit (fast):
cargo test --workspace --all-targets --all-features --release

# Wave-end (slow):
cargo mutants --package workflow_trace -- --release
cargo +nightly fuzz run all_targets -- -max_total_time=3600  # 1h corpus extension
```

---

## Top-1% test-developer practices (the "top quartile of distributions outside central tendency" reference)

The phrase from Luke's prompt — "top 1% of test developers drawn from the top 1% in the top quartile of distributions outside central tendency" — picks the **tail behaviours that distinguish elite testing from competent testing**. Captured as 7 disciplines:

1. **Property-first thinking** — when adding a test, first ask "what is the invariant?", THEN write the unit case as a corollary. Most devs write the unit case first and never get to invariants.
2. **Mutation as ground truth** — `cargo mutants` is the only reliable check that tests aren't theatre. Coverage metrics lie; mutation doesn't.
3. **Differential testing** — for any algorithm (m20 PrefixSpan), implement a slower reference and assert equality on randomised inputs. Catches subtle off-by-one + edge cases that proptest alone misses.
4. **Adversarial generation** — proptest strategies include adversarial inputs (length-0, length-MAX, NaN-equivalent, duplicate-heavy). Default strategies are too uniform.
5. **Realistic fixtures** — integration tests load real captured atuin/stcortex/V3 payloads, not synthetic. Catches schema drift the synthetic version hides.
6. **Test the failure mode, not the success** — for F1-F11 mitigations, the test that asserts the failure is BLOCKED is more valuable than the test that asserts success works.
7. **Document the rationale** — every test has a `// rationale:` comment linking to spec section / bug / failure mode. Tests without rationale rot when the spec changes.

---

*TEST_DISCIPLINE authored 2026-05-17 by Command. 1,594 tests budgeted across 26 modules; 7 pattern families; 7 elite-tier disciplines.*
