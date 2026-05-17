# workflow-trace — Performance

> **Back to:** [`README.md`](../README.md) · [`CLAUDE.md`](../CLAUDE.md) · [`ARCHITECTURE_DEEP_DIVE.md`](ARCHITECTURE_DEEP_DIVE.md) · [`CODE_MODULE_MAP.md`](CODE_MODULE_MAP.md) · [`STATE_MACHINES.md`](STATE_MACHINES.md) · per-module specs at `../ai_specs/modules/cluster-{A-H}/m<N>_<name>.md`
>
> **Function:** Hot-path budgets, benchmark targets, profiling guide. Status: planning-only · 0 LOC · all benchmarks listed are planned (`benches/` directory exists with placeholders); numeric targets derived from per-module spec frontmatter.

---

## 1. Top-line performance envelope

| Hot path | Target | Critical | Module | Bench file |
|---|---|---|---|---|
| **m20 PrefixSpan over 10k rows** (KEYSTONE) | <2s | <10s | m20 | `benches/m20_prefixspan.rs` |
| **m4 cascade correlator throughput** | ≥1,000 rows/s | ≥250 rows/s | m4 | `benches/m4_cascade.rs` |
| **m7 hub insert rate** | ≥500 inserts/s | ≥100/s | m7 | `benches/m7_hub.rs` |
| **m32 5-check pre-dispatch latency** | <50ms | <200ms | m32 | `benches/m32_dispatcher.rs` |
| **m11 decay cycle** (full bank ≤500 entries) | <500ms | <2s | m11 | `benches/m11_decay.rs` |
| **m14 Wilson CI compute** (per workflow) | <100µs | <1ms | m14 | `benches/m14_lift.rs` |
| **m22 K-means convergence** (k=8, n=500) | <50ms | <500ms | m22 | `benches/m22_kmeans.rs` |
| **m31 selector composite** (bank ≤500) | <10ms | <100ms | m31 | `benches/m31_selector.rs` |
| **m40/m41/m42 outbox JSONL write** | <5ms | <50ms | m40/m41/m42 | `benches/cluster_h_emit.rs` |
| **m33 verifier 4-agent quorum** (cached) | <20ms cache hit, <2s fan-out | <500ms cache, <30s fan-out | m33 | `benches/m33_verifier.rs` |

---

## 2. Per-binary end-to-end budgets

| Binary subcommand | Target | Critical | Notes |
|---|---|---|---|
| `wf-crystallise observe --since=24h` | <30s | <120s | bounded by atuin row count + m4 correlation |
| `wf-crystallise propose` | <60s | <300s | bounded by m20 PrefixSpan (KEYSTONE) |
| `wf-crystallise propose accept <id>` | <100ms | <500ms | bounded by m30 SQLite insert + m9 namespace check |
| `wf-crystallise report --json` | <500ms | <2s | bounded by m7 SELECT + m10 Ember (post-§5.1 amendment) |
| `wf-dispatch verify <name>` | <30s (fan-out), <50ms (cache hit) | <2min, <500ms | bounded by 4-agent fan-out |
| `wf-dispatch select` | <50ms | <500ms | bounded by m31 composite + m11 decay read |
| `wf-dispatch dispatch <name>` | <500ms | <5s | bounded by m32 5-check + Conductor POST + m40/m41/m42 outbox writes |

---

## 3. m20 PrefixSpan — KEYSTONE budget breakdown

m20 is the engine's KEYSTONE hot path (Gap 1 NEW PRIMITIVE). Per [m20 spec](../ai_specs/modules/cluster-F/m20_prefixspan_miner.md):

| Input size | Min support | Target | Critical | Memory cap |
|---|---|---|---|---|
| 1,000 rows | 5% | <100ms | <500ms | 16 MB |
| 10,000 rows | 5% | <2s | <10s | 256 MB |
| 100,000 rows | 5% | <30s | <120s | 1 GB (spill to disk above) |

**Memory budget enforcement.** `m20::MinerError::MemoryBudgetExceeded` fires above cap; spills to on-disk projection table.

**Tuning knobs:**
- `min_support` ∈ [0.01, 0.20] — higher value = faster + fewer patterns
- `max_pattern_length` cap — typically 8-step max
- `prune_by_evidence` — m14 lift gate skips patterns m14 returns None for (CC-3)

---

## 4. CC-5 substrate-grain timing (intentionally slow)

CC-5 is the only intentionally-slow path. It is **NOT** measured in milliseconds; it is measured in **days/weeks**:

| Stage | Latency | Notes |
|---|---|---|
| m32 dispatch → m40/m41/m42 outbox write | <5ms | hot path; must not block dispatch |
| m40/m41/m42 outbox → wire emit | <500ms (best-effort) | non-blocking; circuit-breaker on 5 failures |
| wire emit → stcortex pathway.weight update | <100ms | substrate update latency |
| pathway.weight update → m31 read next cycle | **1 selector cycle** (typically 1-24 hours) | not a tight loop |
| substrate-grain accumulation (visible in m14 lift) | **days to weeks** | this is the Hebbian-grain semantics |

**Watcher Class-I primary detector.** If `learning_health` delta over rolling 7-day window is flat AND dispatch volume > 5, CC-5 has silently broken. This is the engine's most important silent-failure mode.

---

## 5. Benchmark execution

```bash
cd /home/louranicas/claude-code-workspace/the-workflow-engine   # post-G2 rename to workflow-trace
export CARGO_TARGET_DIR=./target

# All benchmarks
cargo bench 2>&1 | tail -60

# Specific
cargo bench --bench m20_prefixspan
cargo bench --bench m4_cascade
cargo bench --bench m32_dispatcher
```

Criterion 0.5 is the planned bench harness (per [`plan.toml`](../plan.toml) `[dev-dependencies] criterion = "0.5"`).

---

## 6. Profiling guide

### 6.1 CPU profiling (flamegraph)

```bash
# Hot path identification
cargo install flamegraph
cargo flamegraph --bench m20_prefixspan
# Output: flamegraph.svg
```

Look for:
- m20 inner loop iteration count > 10x sequences (algorithmic issue)
- m4 FNV-1a XOR taking >10% (consider caching opaque IDs)
- m11 decay compute > 30% of decay cycle (formula not vectorised)

### 6.2 Allocation profiling

```bash
# heaptrack or dhat-rs
cargo install dhat-rs
RUST_LOG=trace cargo bench --bench m20_prefixspan --features dhat-heap
```

Look for:
- m20 pattern collection allocations > O(n) (use `SmallVec` or arena)
- m7 JSONB serde alloc churn (consider `serde_json::value::RawValue`)

### 6.3 Async profiling (m40/m41/m42 emit)

```bash
# tokio-console
cargo install tokio-console
RUSTFLAGS="--cfg tokio_unstable" cargo run --features console
# In another shell:
tokio-console
```

Look for:
- m40 wire-emit blocking longer than outbox write (circuit-break tuning)
- m41 reconnect-per-call exceeding 100ms (UDS connection re-use trade-off)

---

## 7. Database query budgets

| Query | Target | Critical | Index |
|---|---|---|---|
| `SELECT FROM workflow_runs WHERE session_id = ?` | <1ms | <10ms | `idx_runs_session` |
| `SELECT FROM workflow_runs WHERE last_run_at > ?` (cursor) | <5ms | <50ms | `idx_runs_last_run_at` |
| `SELECT FROM bank WHERE sunset_at > ? AND status = 'admitted'` | <2ms | <20ms | `idx_bank_sunset_status` (partial) |
| `INSERT INTO workflow_runs (…)` | <1ms | <10ms | n/a |
| `UPDATE bank SET decay_factor = ?` (batch ≤500) | <50ms | <500ms | n/a |
| `SELECT FROM verify_cache WHERE workflow_id = ? AND ttl_expires_at > ?` | <500µs | <5ms | `idx_verify_ttl` |

**Discipline.** Always `.schema` before writing SQL ([`CLAUDE.md`](../CLAUDE.md) B1 pattern). Migrations under `../migrations/`.

---

## 8. Memory budgets

| Component | Steady-state | Peak (during m20) | Critical |
|---|---|---|---|
| `wf-crystallise` daemon-mode | 50 MB | 512 MB (m20 KEYSTONE) | 2 GB |
| `wf-crystallise` one-shot CLI | 20 MB | 256 MB | 1 GB |
| `wf-dispatch` daemon-mode | 30 MB | 100 MB | 512 MB |
| `wf-dispatch` one-shot CLI | 15 MB | 50 MB | 256 MB |

CLI-not-service architecture (Phase 5A) means most invocations are one-shot — peak memory bounded by single subcommand.

---

## 9. Observability for performance

Per [`Phase 8 observability`](../the-workflow-engine-vault/deployment%20framework/phase-8-observability-operations.md):

**Prometheus metrics (Pushgateway since CLI exits after work):**

```
wf_m20_prefixspan_duration_ms_histogram     # KEYSTONE p50/p95/p99
wf_m32_dispatch_duration_ms_histogram       # 5-check + Conductor call
wf_m7_hub_insert_duration_ms_histogram      # SQLite insert
wf_m11_decay_cycle_duration_ms_histogram    # full-bank decay
wf_m40_outbox_write_duration_ms_histogram   # JSONL durability
wf_m40_wire_emit_duration_ms_histogram      # best-effort wire
wf_m31_selection_weight_gauge               # FNV-1a u32 hashed labels + overflow bucket (cardinality trap)
wf_m14_lift_gauge                            # -1.0 sentinel for Class-I distinction (NOT zero)
wf_cc5_learning_health_delta_gauge          # rolling 7-day window
```

**Cardinality trap.** `wf_m31_selection_weight` is single highest Prometheus risk if `workflow_id` is raw label. Mitigation: FNV-1a u32 hashing + overflow bucket per Phase 8 doc.

---

## 10. Performance acceptance criteria (per phase)

| Phase | Acceptance |
|---|---|
| Phase 2A | m7 insert <1ms; m4 throughput ≥1k rows/s |
| Phase 2B | m20 over 10k rows <2s; m32 5-check <50ms; m33 cache hit <20ms |
| Phase 3 integration | end-to-end `wf-dispatch dispatch <known>` round-trip <500ms; CC-5 first closure measurable (substrate delta > 0 after dispatch) |
| Phase 4 hardening | performance-engineer agent owns F2 + F10; verifies all hot paths meet TARGET (not CRITICAL) |
| Phase 5C soak | weekly p95 trend within ±20% of TARGET; degradation → Watcher Class-I OR Class-D (drift) |

---

## 11. Anti-patterns specific to performance

| AP | Anti-pattern | Mitigation |
|---|---|---|
| AP-WT-F10 | m6 EMA includes Converged outcomes — exploration baseline inflated | m6 excludes Converged outcomes (spec invariant) |
| AP-V7-07 | Worktree `target/` symlink causes phantom rebuilds | per-worktree `target/`; never symlink |
| AP-Hab-05 | `cargo bench \| tail` masks fail status | `${PIPESTATUS[0]}` per stage |
| (perf-specific) | m20 PrefixSpan memory explosion when min_support too low | clamp min_support ≥0.02 in CLI; bench harness reports `MemoryBudgetExceeded` cases |
| (perf-specific) | m11 decay cycle reads stcortex per workflow synchronously | batch read pathway.weight in single call per cycle |
| (perf-specific) | m32 Conductor health check blocks dispatch on slow Conductor | 2s timeout; refuse-mode on timeout |

---

## 12. Cross-references

- **Architecture deep dive (where hot paths sit):** [`ARCHITECTURE_DEEP_DIVE.md`](ARCHITECTURE_DEEP_DIVE.md)
- **State machines (timing of m11 sunset, m32 5-check, m33 TTL):** [`STATE_MACHINES.md`](STATE_MACHINES.md)
- **Error taxonomy (perf budget violation errors):** [`ERROR_TAXONOMY.md`](ERROR_TAXONOMY.md)
- **Per-module specs (LOC + test + budget per module):** `../ai_specs/modules/cluster-{A-H}/m<N>_<name>.md`
- **Phase 8 observability operations:** [`phase-8-observability-operations`](../the-workflow-engine-vault/deployment%20framework/phase-8-observability-operations.md)
- **plan.toml dev-dependencies:** [`../plan.toml`](../plan.toml)

> **Back to:** [`README.md`](../README.md) · [`CLAUDE.md`](../CLAUDE.md) · [`ARCHITECTURE_DEEP_DIVE.md`](ARCHITECTURE_DEEP_DIVE.md)

*PERFORMANCE authored 2026-05-17 (S1001982) by Command. Hot-path budgets per module + CC-5 intentional slowness explained + cardinality trap + perf-specific anti-patterns.*
