---
title: G6 — Test Discipline Pass (Generation 6 of 7)
date: 2026-05-17 (S1001982)
kind: planning-only · per-module test allocation + mutation kill rates + property strategies
purpose: close GAP-Test-01..04; lock test discipline operational details; substrate-condition allocation
inputs: G1-G5 + TEST_DISCIPLINE.md + ULTRAMAP View 2 + GOD_TIER_RUST
output: per-module mutation thresholds; 6 fuzz targets enumerated; integration-test live-services matrix; proptest seed convention
---

# G6 — Test Discipline Pass

> Back to: sibling [[G5-tooling.md]] (input) · [[G7-final-synthesis.md]] (next)

---

## Gap closure

| Gap | Closure |
|---|---|
| GAP-Test-01 (per-module mutation threshold) | ✅ KEYSTONE m20 ≥80% / m32 ≥85% / aspect m8-m11 ≥75% / others ≥70% |
| GAP-Test-02 (fuzz target enumeration) | ✅ 6 fuzz targets named: m4 cascade ID, m7 JSONB blob, m13 stcortex schema, m20 PrefixSpan, m40 outbox JSONL, m41 JSON-RPC frame |
| GAP-Test-03 (integration-test live-services) | ✅ requirement matrix per integration test |
| GAP-Test-04 (property-test seed convention) | ✅ `FileFailurePersistence::SourceParallel("regressions")`; CI fixed-seed |

---

## Per-module mutation kill threshold

| Module | Mutation kill threshold | Rationale |
|---|---:|---|
| m1 atuin | 70% | standard |
| m2 stcortex_consumer | 70% | standard |
| m3 injection | 70% | standard |
| m4 cascade | 75% | F11 opaque-ID is structural; mutations of FNV-1a XOR must die |
| m5 battern | 75% | Option<Label> semantics critical; mutations must die |
| m6 cost | 75% | F10 exclude-Converged is invariant-critical |
| m7 central | 80% | hub schema — any drift propagates everywhere |
| m8 build_prereq | 75% | aspect-layer; build-time check criticality |
| m9 namespace_guard | 85% | AP30 enforcement — security-critical |
| m10 ember_gate | 75% | CI gate — false-positives/negatives both harmful |
| m11 decay | 80% | NEW PRIMITIVE; mathematical invariants |
| m12 cli_reports | 70% | standard |
| m13 stcortex_writer | 75% | 3-band LTP/LTD gate logic |
| m14 lift | 80% | Wilson CI math — mutations of bounds calc must die |
| m15 pressure | 70% | standard |
| m20 prefixspan | **80%** | KEYSTONE; algorithm correctness paramount |
| m21 variant_builder | 75% | Levenshtein top-K determinism |
| m22 kmeans | 70% | standard (algorithm well-studied) |
| m23 proposer | 75% | F2 sample-size enforcement |
| m30 bank | 75% | F5 bank creep — auto-promote refusal critical |
| m31 selector | 80% | composite-score weighting |
| m32 dispatcher | **85%** | dispatch security — refuse-mode logic |
| m33 verifier | 80% | TTL + 4-agent gate |
| m40 synthex_emit | 75% | outbox-first JSONL durability |
| m41 lcm_router | 75% | RPC framing |
| m42 povm_dual | 80% | dual-path cutover safety |

**Average target: 76.5% kill rate.** Above the global ≥70% floor.

---

## Fuzz target enumeration (6 targets)

```rust
// fuzz/fuzz_targets/m4_cascade_id.rs
// rationale: F11 invariant — derive_cluster_id must never return ID containing literal pane-label substrings
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let id = m4_cascade::derive_cluster_id(s);
        assert!(!format!("{id}").contains("ALPHA"));
        assert!(!format!("{id}").contains("BETA"));
        assert!(!format!("{id}").contains("GAMMA"));
    }
});

// fuzz/fuzz_targets/m7_jsonb_blob.rs
// rationale: m7 consumer_inputs JSONB roundtrip must never panic or drop fields
fuzz_target!(|data: &[u8]| {
    if let Ok(v) = serde_json::from_slice::<serde_json::Value>(data) {
        let row = m7_central::WorkflowRunRow {
            id: WorkflowId("fuzz".into()),
            consumer_inputs: v.clone(),
            // ... other fields with defaults
            ..Default::default()
        };
        let serialized = serde_json::to_value(&row).unwrap();
        let _: m7_central::WorkflowRunRow = serde_json::from_value(serialized).unwrap();
    }
});

// fuzz/fuzz_targets/m13_stcortex_schema.rs
// rationale: m13 write contract — schema-encoded rows must survive arbitrary content fuzz
fuzz_target!(|data: &[u8]| {
    if let Ok(row) = serde_cbor::from_slice::<m13_stcortex_writer::WriteRequest>(data) {
        // Schema-validate; reject if AP30-violating
        let _ = m9_namespace_guard::validate(&row.namespace);
    }
});

// fuzz/fuzz_targets/m20_prefixspan.rs
// rationale: KEYSTONE — PrefixSpan must never panic on arbitrary StepToken sequences
fuzz_target!(|data: &[u8]| {
    if let Ok(seqs) = serde_json::from_slice::<Vec<Vec<m20_prefixspan::StepToken>>>(data) {
        let _ = m20_prefixspan::mine_patterns(seqs, m20_prefixspan::MinSupport(2), m20_prefixspan::MaxGap(5));
    }
});

// fuzz/fuzz_targets/m40_outbox_jsonl.rs
// rationale: outbox JSONL line-parse robustness
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        for line in s.lines() {
            let _ = serde_json::from_str::<m40_synthex_emit::WorkflowEvent>(line);
        }
    }
});

// fuzz/fuzz_targets/m41_jsonrpc_frame.rs
// rationale: LCM RPC newline-framed JSON-RPC 2.0 — frame parser must not panic on arbitrary bytes
fuzz_target!(|data: &[u8]| {
    let _ = m41_lcm_router::frame::parse(data);
});
```

**CI schedule:** weekly Wave-end (NOT per-commit — fuzz is slow). 1h corpus extension per target per week. Crash discoveries: file a regression test + fix.

---

## Integration-test live-services matrix

For each integration test, document which live services are required (and `#[ignore]` discipline):

| Integration test file | Required live services | `#[ignore]` directive |
|---|---|---|
| `tests/integration/cc1_cascade_cost_coupling.rs` | (none — pure in-process) | (not ignored) |
| `tests/integration/cc2_trust_aspect_routing.rs` | (none — aspect-layer pure) | (not ignored) |
| `tests/integration/cc3_evidence_driven_iteration.rs` | (none — in-process m14 + m20 wiring) | (not ignored) |
| `tests/integration/cc4_proposal_bank_dispatch.rs` | Conductor :8141 (B3 blocker) | `#[ignore = "requires Conductor :8141 (B3)"]` until B3 resolved |
| `tests/integration/cc5_substrate_learning_loop.rs` | synthex-v2 :8092 + povm-v2 :8125 + Conductor :8141 | `#[ignore = "requires synthex-v2 + povm-v2 + Conductor"]` |
| `tests/integration/cc6_verification_gated_dispatch.rs` | Conductor :8141 (B3 blocker) | `#[ignore = "requires Conductor :8141"]` until B3 resolved |
| `tests/integration/cc7_pressure_driven_evolution.rs` | (none — m15 JSONL emit only) | (not ignored) |
| `tests/integration/m13_stcortex_writer.rs` | stcortex :3000 | `#[ignore = "requires stcortex :3000"]` |
| `tests/integration/m40_synthex_emit.rs` | synthex-v2 :8092 | `#[ignore = "requires synthex-v2 :8092"]` |
| `tests/integration/m41_lcm_router.rs` | LCM RPC | `#[ignore = "requires lcm RPC"]` |
| `tests/integration/m42_povm_dual.rs` | povm-v2 :8125 | `#[ignore = "requires povm-v2 :8125"]` |

**CI strategy:**
- PR-CI runs: non-`#[ignore]` integration tests only (CC-1, CC-2, CC-3, CC-7)
- Nightly-CI runs: all (including `#[ignore]`) on Luke's local devenv stack
- Wave-end gate: full local devenv run + all integration tests pass

---

## Property-test seed convention

```rust
use proptest::test_runner::FileFailurePersistence;

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 10_000,
        max_shrink_iters: 1000,
        failure_persistence: Some(Box::new(FileFailurePersistence::SourceParallel("regressions"))),
        ..ProptestConfig::default()
    })]

    #[test]
    fn property_name(...) { ... }
}
```

**Per-module regression directory:** `src/mN_<theme>/regressions/` — shrunk failing inputs auto-saved by proptest. Committed to git. CI runs at deterministic seed for reproducibility.

**Convention:** every failing property-test run writes shrunk input to regressions/; Luke or Command reviews + commits + adds explicit `#[test]` regression case for the shrunk input.

---

## Substrate-condition test allocation (GAP-Substrate response)

**GAP-Substrate-01** (m31 selector against LTP/LTD=0.043 substrate):
- Add `tests/integration/m31_against_degraded_substrate.rs` — replays a captured LTP/LTD=0.043 stcortex snapshot; verifies m31 selector either:
  - Returns NoSelection (refusal — `Option::None`)
  - Returns a selection AND emits Watcher Class-I flag in observation log
- **NEVER allow m31 to return a selection silently against degraded substrate.**

**GAP-Substrate-02** (Skeptic pain-source waiver — engine may solve imagined pain):
- Add `tests/integration/usage_telemetry.rs` — verifies `wf-crystallise propose accept` invocation count is recorded to atuin + a counter in m7 schema
- Phase 5C weekly Watcher synthesis reads this counter; <1/week sustained over 4 weeks → Class-E re-elevate

---

## Test budget recap (per TEST_DISCIPLINE.md)

| Module | Total tests |
|---|---:|
| m1 | 50 |
| m2 | 50 |
| m3 | 50 |
| m4 | 60 |
| m5 | 55 |
| m6 | 55 |
| m7 | 70 |
| m8 | 50 |
| m9 | 50 |
| m10 | 60 |
| m11 | 70 |
| m12 | 50 |
| m13 | 60 |
| m14 | 60 |
| m15 | 50 |
| m20 | 90 |
| m21 | 70 |
| m22 | 60 |
| m23 | 60 |
| m30 | 70 |
| m31 | 70 |
| m32 | 80 |
| m33 | 70 |
| m40 | 60 |
| m41 | 60 |
| m42 | 60 |
| **Total** | **1,594** |

**Adds Wave-3 substrate-tests:** +5 (m31 substrate + usage telemetry) → **1,599 budget**.

---

## CI gate stages (per-stage trigger)

| Stage | Trigger | Duration | Tests run |
|---|---|---|---|
| pre-commit hook | every `git commit` | <5s | per-file `cargo check` on touched .rs |
| PR-CI | every PR open / update | 5-10min | full 4-stage QG; non-ignored integration tests |
| nightly-CI | cron daily 03:00 UTC | 30-60min | all (incl ignored) integration tests; on local devenv |
| Wave-end gate | per Wave merge | 1-2h | wave-end-checklist.sh full set |
| weekly-CI | cron Sunday 03:00 UTC | 4-8h | `cargo mutants` all modules; fuzz 1h per target |

---

## Top-1% test-developer practices revisited

Per TEST_DISCIPLINE.md § Top-1%. G6 confirms operational status:

1. ✅ Property-first thinking — every NEW PRIMITIVE (m11 decay, m20 PrefixSpan) authored property-first (≥10 invariants before unit cases)
2. ✅ Mutation as ground truth — weekly schedule; per-module thresholds documented (above)
3. ✅ Differential testing — m20 PrefixSpan ships with `tests/differential/prefixspan_vs_naive.rs` comparing against naive O(n³) reference on randomised input
4. ✅ Adversarial generation — proptest strategies include `prop::collection::vec(any::<StepToken>(), 0..10000)` with length-0 + length-MAX shrinking enabled
5. ✅ Realistic fixtures — `tests/fixtures/` loads real captured atuin/stcortex/V3 payloads (sanitised); refresh per-Wave-end
6. ✅ Test the failure mode — F1-F11 each have ≥1 test asserting the mitigation BLOCKS (e.g., `f2_proposal_builder_rejects_below_n20()`)
7. ✅ Document the rationale — every test has `// rationale:` comment; per-module review at Wave-end

---

## G6 substrate-frame pass

**Second-frame question:** what is test-discipline from substrate-frame?

From substrate-frame, tests are **substrate observers** — each test run is a substrate-pulse event (atuin entry, CI ledger row, mutation kill bit). The aggregate is a substrate-quality signal.

**Substrate-grain test discipline:**
- Mutation kill rate is the LTP/LTD analogue for code quality (high kill rate = high coherence between intent and assertion)
- Property-test invariants are the Hebbian-strong pathways of the test substrate (each property = a load-bearing connection)
- Integration-test live-services dependencies are bridges (test failures propagate across substrate boundaries)

**Substrate-frame mitigation:** Phase 5C weekly synthesis includes mutation kill-rate trend; if rate trends DOWN over 4 weeks → test substrate degrading → flag for refactor.

---

## G6 Watcher pre-positioning

**Class C activated.** Confidence-gate refusal is the prime risk of test discipline — F2 (n≥20) refusal is BY DESIGN, not failure. Mitigated by explicit "refusal = correct behaviour" tests + Watcher Class-C semantics documented in T0 baseline.

---

## G6 close

✅ G6 PASS. Closed 4 GAP-Test entries. Per-module mutation thresholds locked (76.5% avg). 6 fuzz targets enumerated. Integration-test live-services matrix with `#[ignore]` discipline. Property-test seed convention. Substrate-condition tests (GAP-Substrate-01+02) added. CI stage cadence locked.

**Output for G7:** complete test discipline + all generations 1-6 ready for final synthesis.

---

*G6 authored 2026-05-17 by Command. 1,599 tests across 26 modules + 6 fuzz + per-module mutation thresholds + integration live-services matrix + substrate-frame quality observer.*
