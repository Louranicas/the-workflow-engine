---
title: BENCHMARK_SPEC — criterion benches (m20 KEYSTONE, m4, m7, m32)
date: 2026-05-17
status: SPEC
tooling: [criterion, flamegraph]
---

# BENCHMARK_SPEC — workflow-trace

> **Back to:** [`INDEX.md`](INDEX.md) · [`MODULE_MATRIX.md`](MODULE_MATRIX.md) · [`../README.md`](../README.md) · [`../ARCHITECTURE.md`](../ARCHITECTURE.md) · [`TEST_STRATEGY.md`](TEST_STRATEGY.md)

## Bench inventory

Four modules carry criterion benches; one runs at every Wave-end (KEYSTONE), three at PR-CI:

| Module | Bench | Schedule | Target |
|---|---|---|---|
| m20 PrefixSpan | `benches/m20_prefixspan.rs` | KEYSTONE — every Wave-end; PR-CI on `intelligence` changes | 10k rows: < 500ms; 100k stress: < 8s |
| m4 cascade_correlator | `benches/m4_correlate.rs` | PR-CI on `m4_*` changes | batch 2000: < 30ms |
| m7 workflow_runs | `benches/m7_insert.rs` | PR-CI on `m7_*` changes | single insert: < 5ms; bulk 1000: < 500ms |
| m32 5-check | `benches/m32_5check.rs` | PR-CI on `m32_*` changes | 5-check sequence: < 200ms p99 |

## m20 PrefixSpan bench (KEYSTONE)

The most expensive surface in the engine. Two sizes:

```rust
// benches/m20_prefixspan.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_prefixspan_10k(c: &mut Criterion) {
    let corpus = generate_corpus(10_000);
    let mut group = c.benchmark_group("m20_prefixspan_10k");
    group.bench_function("default_gap_5_n_min_20", |b| {
        b.iter(|| {
            black_box(prefixspan_mine(black_box(&corpus), 5, 20))
        });
    });
    group.finish();
}

fn bench_prefixspan_100k_stress(c: &mut Criterion) {
    let corpus = generate_corpus(100_000);
    let mut group = c.benchmark_group("m20_prefixspan_100k");
    group.sample_size(10);  // expensive — reduce sample count
    group.bench_function("stress_gap_5_n_min_20", |b| {
        b.iter(|| {
            black_box(prefixspan_mine(black_box(&corpus), 5, 20))
        });
    });
    group.finish();
}

criterion_group!(benches, bench_prefixspan_10k, bench_prefixspan_100k_stress);
criterion_main!(benches);
```

**Targets:**

- 10k rows / gap=5 / n_min=20: **< 500 ms** (commit-blocking).
- 100k rows stress: **< 8 s** (Wave-end-blocking).

Regression > 10% from previous run fails CI; flamegraph diff attached to PR.

## m4 cascade_correlator bench

```rust
fn bench_m4_correlate(c: &mut Criterion) {
    let batch = generate_tool_call_batch(2000);
    c.bench_function("m4_correlate_2000_rows", |b| {
        b.iter(|| black_box(cascade_correlate(black_box(&batch))));
    });
}
```

**Target:** batch of 2000 < 30 ms. FNV-1a XOR is O(n); regression suggests an algorithmic change.

## m7 workflow_runs insert bench

```rust
fn bench_m7_single_insert(c: &mut Criterion) {
    let conn = setup_test_db();
    let row = test_workflow_run_row();
    c.bench_function("m7_insert_single", |b| {
        b.iter(|| black_box(m7::insert_workflow_run(&conn, black_box(&row))));
    });
}

fn bench_m7_bulk_insert(c: &mut Criterion) {
    let conn = setup_test_db();
    let rows: Vec<_> = (0..1000).map(|_| test_workflow_run_row()).collect();
    c.bench_function("m7_insert_bulk_1000", |b| {
        b.iter(|| {
            black_box(m7::insert_workflow_runs_bulk(&conn, black_box(&rows)))
        });
    });
}
```

**Targets:** single < 5 ms; bulk 1000 < 500 ms (transactional).

## m32 5-check bench

```rust
fn bench_m32_5check(c: &mut Criterion) {
    let dispatcher = setup_dispatcher_with_mock_conductor();
    let workflow_id = test_workflow_id();
    c.bench_function("m32_5check_all_pass", |b| {
        b.iter(|| {
            black_box(dispatcher.run_5_check_sequence(black_box(&workflow_id)))
        });
    });
}
```

**Target:** 5-check sequence < 200 ms p99 (Conductor `/health` HTTP is the dominant cost).

## Regression detection

```bash
# baseline (commit before change):
cargo bench --bench m20_prefixspan -- --save-baseline before

# after change:
cargo bench --bench m20_prefixspan -- --baseline before

# criterion auto-detects regression > 5%; CI gate at 10%.
```

## Flamegraph profiling

For KEYSTONE m20:

```bash
cargo flamegraph --bench m20_prefixspan -- --bench
# inspect target/flamegraph.svg
```

Hot-path identified via flamegraph guides:
- removal of `String::from` / `format!` (god-tier rule #14)
- removal of `Vec::push` in inner loops where capacity bounded (rule #15)
- removal of `chrono`/`time` from hot paths (rule #18)

## Wave-end bench discipline

At every Wave-end (per V7 STANDARDS):

```bash
# Stage 1: full bench suite
cargo bench --workspace 2>&1 | tee bench-wave-end.log

# Stage 2: regression analysis vs last Wave-end baseline
cargo criterion --baseline previous_wave_end

# Stage 3: flamegraph capture for KEYSTONE
cargo flamegraph --bench m20_prefixspan
```

Results archived under `target/criterion/` + committed to a `benchmarks/` directory in the repo (NOT generated files; just the summary JSON).

## Anti-patterns

- **Bench in inner loop without `black_box`** — compiler may dead-code-eliminate.
- **Allocating in `b.iter(|| ...)`** — measures allocation, not the algorithm.
- **Unrealistic input sizes** — 10 rows is not representative; use the documented size targets.
- **No baseline comparison** — bench numbers without baseline = noise.
- **`chrono::Utc::now()` in hot path** — use `std::time::Instant` per god-tier rule #18.

---

> **Back to:** [`INDEX.md`](INDEX.md) · [`TEST_STRATEGY.md`](TEST_STRATEGY.md)
