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

## Substrate-side load benchmarks (NA-GAP-04 closure)

The four benches above all measure **engine-side** performance — m20 work in the engine's process, m4/m7 in the engine's SQLite, m32 against a mocked Conductor. Per NA-GAP-04, this misses a structural question: what is each substrate's own load envelope, measured at the substrate, before workflow-trace traffic degrades the substrate's foreground work for *other* habitat services (or for the user's shell, in atuin's case)?

The engine cannot dictate a substrate's load policy — that's the co-tenant rule. But the engine MUST measure its own substrate-side load contribution and gate cadence increases against substrate-back-pressure signals defined in each substrate dossier.

### Substrate-side benches (per substrate, measured at substrate)

| Substrate | Bench | Measurement surface | Threshold (substrate-protective) |
|---|---|---|---|
| **S-A atuin** ([`substrates/atuin.md`](substrates/atuin.md) § back-pressure) | `benches/substrate_load_atuin.rs` | m1 cursor advance rate vs atuin daemon `wal_size_bytes` + `busy_timeout_exceeded` count | wal_size growth attributable to m1 < 5% over baseline; busy_timeout < 1/min |
| **S-B injection.db** ([`substrates/injection_db.md`](substrates/injection_db.md)) | `benches/substrate_load_injection_db.rs` | m3 read cadence vs injection.db `database_locked` rate + habitat-memory daemon TTL-sweep contention | locked rate < 1/min; m3 reads should pause during daemon sweep window |
| **S-C stcortex** ([`substrates/stcortex.md`](substrates/stcortex.md)) | `benches/substrate_load_stcortex.rs` | m42 reinforce rate + m13 read rate vs stcortex `:3000` reducer queue depth + idempotency-cache size | reducer queue p99 < 50ms; idempotency cache size stable (TTL sweep keeping pace) |
| **S-D Conductor** ([`substrates/conductor.md`](substrates/conductor.md)) | `benches/substrate_load_conductor.rs` | m32 dispatch rate vs Conductor wave-pane handoff latency (semantic-endpoint probe) | semantic-endpoint p99 < 100ms; wave-pane backlog < 10 |
| **S-E SYNTHEX v2** ([`substrates/synthex.md`](substrates/synthex.md)) | `benches/substrate_load_synthex.rs` | m40 NexusEvent push rate vs SYNTHEX v2 coordinator queue + R13 quiet-period state | push accepted < 5ms p99; coordinator queue depth growth bounded |
| **S-F LCM** ([`substrates/lcm.md`](substrates/lcm.md)) | `benches/substrate_load_lcm.rs` | m41 RPC rate vs LCM supervisor loop_id state-transition latency | loop_id created → ready p99 < 200ms; deploy backlog bounded |

### Methodology

Each substrate-side bench follows the same pattern:

1. **Pre-measure substrate baseline** — capture the substrate's own foreground metrics (e.g. atuin busy_timeout rate from a fresh write-only shell session; stcortex reducer p99 from a no-workflow-trace consumer-only window) over a 60s baseline window.
2. **Apply workflow-trace load** — drive the engine's per-substrate write path at a controlled rate (`{N}` events/sec) for 60s.
3. **Re-measure substrate metrics** — capture the same foreground metrics during the load window.
4. **Compute delta + attribute** — substrate-side delta is what workflow-trace cost the substrate. The threshold gate is on the **delta**, not absolute (since substrates have their own non-workflow-trace traffic).
5. **Emit `SubstrateLoadProfile { substrate_id, baseline, loaded, delta_pct, threshold_crossed }`** — surfaced via `cargo bench` JSON output for `criterion`.

### Wave-end discipline

```bash
# Stage 1: engine-side benches (as before)
cargo bench --workspace

# Stage 2: substrate-side benches (new — requires live substrates)
cargo bench --bench 'substrate_load_*' --features substrate-load -- --baseline previous_wave_end

# Stage 3: substrate-impact report (cross-bench analysis)
~/.local/bin/wf-bench-substrate-report --window 7d
```

Substrate-side benches are **opt-in** via `--features substrate-load` because they require live substrates and are noisier than engine-side benches; they run at Wave-end + nightly, NOT in PR-CI (PR-CI continues to run engine-side benches only).

### Substrate-drift gating

When [`cross-cutting/substrate-drift.md`](cross-cutting/substrate-drift.md) canary fires `SubstrateDriftDetected` for a given substrate, **substrate-side benches for that substrate are quarantined** — their thresholds may no longer represent reality. The bench harness checks substrate-drift state at start and skips quarantined substrates with a `[QUARANTINED]` annotation, unblocking Wave-end without false regression flags.

### Per-substrate cadence-throttle rules

Output of substrate-side benches feeds a **per-substrate cadence-throttle table** that engine modules consult before increasing read/write cadence:

```rust
// SPEC ONLY
pub struct SubstrateCadenceLimit {
    pub substrate_id: SubstrateId,
    pub current_load_pct: f64,        // measured from last bench
    pub recommended_max_ops_per_sec: f64,
    pub last_measured_at: i64,
}
```

m1 (atuin), m3 (injection.db), m13 (stcortex), m40/m41/m42 (Cluster H) all consult this table before tightening cadence. If `current_load_pct > 80`, cadence is locked at current; if > 95, cadence is auto-throttled by 20%. The table is engine-internal (not substrate-controlled), populated from substrate-side bench output.

### Anti-patterns specific to substrate-side benchmarking

- **Measuring at the engine** — defeats the purpose; the gap NA-GAP-04 surfaced is precisely "engine-side measurement misses substrate-internal contention". Substrate-side benches MUST measure at the substrate.
- **Synthetic substrate fixtures alone** — a mock SQLite doesn't reproduce atuin daemon's WAL contention; substrate-side benches MUST run against live substrates (or, deferred, substrate fixtures per NA-GAP-08 — see [`../ai_docs/decisions/2026-05-17-substrate-as-actor-deferrals.md`](../ai_docs/decisions/2026-05-17-substrate-as-actor-deferrals.md)).
- **Single-window measurement** — sub-second windows hide substrate-internal periodic events (atuin WAL checkpoint cadence, habitat-memory daemon TTL sweep, stcortex idempotency-cache rotation). 60s minimum; ideally 5-min windows captured over a 24-hour daily-cycle.
- **Ignoring co-tenant traffic** — substrates are shared (atuin serves all shells; stcortex serves all habitat services). Benches MUST not assume the substrate is otherwise idle.

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
