---
substrate_id: S-B-injection-db
kind: sql
read_endpoints: ["~/.local/share/habitat/injection.db (SQLite, ro)"]
write_endpoints: ["habitat-memory service :8140 (engine NEVER writes directly)"]
lifecycle_phases: [cold-start, warming, steady-state, degraded, refusing, dead]
refusal_modes: [database_locked, schema_missing, resolved_session_excludes_row, ttl_sweep_active]
drift_indicators: [schema_migration, causal_chain_column_rename, reinforcement_count_semantic_shift, ttl_window_change]
back_pressure_signals: [db_size_bytes, ttl_sweep_in_progress, hourly_sweep_active]
consent_dimensions: [n/a]
status: SPEC
date: 2026-05-17
authority: Luke @ node 0.A — S1002127 "as per proposal"
hold_v2_compliant: true
addresses: [NA-GAP-01, NA-GAP-03, NA-GAP-07, NA-GAP-08]
---

# S-B — injection.db (causal-chain substrate)

> **Back to:** [`INDEX.md`](../INDEX.md) · [`../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md) · [`../cross-cutting/persistence.md`](../cross-cutting/persistence.md) · [`../cross-cutting/refusal-taxonomy.md`](../cross-cutting/refusal-taxonomy.md) · [`../cross-cutting/substrate-drift.md`](../cross-cutting/substrate-drift.md) · sister substrates [`atuin.md`](atuin.md) · [`stcortex.md`](stcortex.md) · [`synthex.md`](synthex.md) · [`lcm.md`](lcm.md) · [`conductor.md`](conductor.md) · [`watcher.md`](watcher.md) · [`operator.md`](operator.md)
>
> Engine consumer: **m3** ([`../modules/cluster-A/m3_injection_db_consumer.md`](../modules/cluster-A/m3_injection_db_consumer.md))

## 1. Purpose & boundary

injection.db is the habitat's **learned-pattern memory** — a SQLite file at `~/.local/share/habitat/injection.db` with 5 tables (currently 74 causal-chain rows + 1,952 trajectory rows). The `habitat-inject` SessionStart hook pre-warms this DB at every Claude session start, injecting < 2KB of high-reinforcement causal-chains into the context window. Owned externally by service `habitat-memory` at `:8140`.

**IN scope for `workflow-trace`:** read-only partitioned access to `causal_chain`, `trajectory`, `pattern`, `behavioral_rule`, and `learned_chain_state` tables via `mode=ro` SQLite open + `PRAGMA query_only=ON`. Filter `WHERE resolved_session IS NULL` for unresolved rows (substrate-side gate semantically equivalent to "include in injection"). Re-derive injection candidates for engine-side analysis (m3).

**OUT of scope:** writing to injection.db (habitat-memory owns this); triggering reinforcement_count updates; running the TTL sweep; resolving causal chains. The engine is a **passive reader** of substrate-internal dynamics.

## 2. Lifecycle phases

| Phase | Indicator | Engine action |
|---|---|---|
| cold-start | habitat-memory daemon recently up; DB possibly mid-migration | m3 defers reads 1s; re-probes schema |
| warming | DB stable; reinforcement_count flowing from session-end hooks | normal reads |
| steady-state | row count linear with sessions; sweeps quiet | normal cadence |
| degraded | DB size > 100MB; sweep frequency increasing | m3 reduces query frequency |
| refusing | `database is locked` from concurrent sweep | retry with exponential backoff |
| dead | file missing or schema absent | m3 returns SubstrateUnavailable; engine proceeds w/o priors |

## 3. Refusal modes (substrate-authored)

- **`SqliteError::Busy`** — habitat-memory's hourly TTL sweep holds an exclusive lock. Substrate-authored. Recovery: backoff 1s→10s.
- **`SchemaMissing`** — `causal_chain` table absent (habitat-memory not yet provisioned). Substrate-authored. Recovery: surface as SubstrateUnavailable, do not auto-create.
- **`resolved_session NOT NULL`** — row exists but is excluded from injection by substrate-side semantics. NOT a refusal — this is correct filtering. Engine respects it.
- **TTL sweep active** — substrate signals via `PRAGMA wal_size` spike; engine should defer non-essential reads.

These map to `RefusalToken::SubstrateAuthored { substrate_id: "injection_db", class, repair_hint }` per [`../cross-cutting/refusal-taxonomy.md`](../cross-cutting/refusal-taxonomy.md).

## 4. Drift indicators (closes NA-GAP-07)

- **Schema migration silent** — habitat-memory bumps `causal_chain` schema (e.g. renames `reinforcement_count` to `weight`). Detector: schema-hash check at session-start.
- **`reinforcement_count` semantic shift** — substrate changes the meaning (e.g. from "raw count" to "EMA-decayed"). This would be a CR-2-class incident; see [`../cross-cutting/substrate-drift.md`](../cross-cutting/substrate-drift.md). Detector: re-derive a sample from raw trajectory rows and compare against reported count.
- **TTL window change** — habitat-memory tunes its sweep window from 30 days to 7 days; rows the engine just observed disappear. Detector: row-stability canary (a known row's persistence across reads).
- **Test timestamp sweep** — per feedback memory `feedback_ttl_sweep_test_timestamps.md`, test events with `ts_ms=0..5` get swept by hourly TTL cleanup. Detector: realistic timestamps in tests.

## 5. Back-pressure signals (closes NA-GAP-04)

- **DB size** — `injection.db` > 100MB indicates habitat-memory's sweep is lagging.
- **Hourly sweep in progress** — observable via WAL spike; engine should defer.
- **Row count drop** — sudden drop in `SELECT count(*) FROM causal_chain WHERE resolved_session IS NULL` suggests a sweep just ran; cached priors may be stale.

## 6. Receipts (closes NA-GAP-09)

injection.db does NOT emit substrate-side receipts. The substrate-confirmable receipt is **structural**: when m42 writes pathway X to stcortex (S-C) and the next session's `habitat-inject` hook pulls a row referencing X out of injection.db, the closed loop is observable only via cross-substrate timing (S-C write at T → S-B row pre-warmed at T + Δ where Δ is "next session start"). This is the **hidden coupling NA-GAP-03 surfaces**: the engine cannot independently observe S-C → S-B without coordinating reads.

Mitigation: Watcher Class-I rolling-window query observes both S-C pathway delta AND S-B causal_chain.reinforcement_count delta on the same session-correlated workflow IDs.

## 7. Capabilities & namespaces (closes NA-GAP-10)

injection.db has NO substrate-side per-consumer ACL — any process with file-system read access reads all tables. Engine reads under the same trust regime as any other co-tenant.

Namespace boundary is enforced at the data-row level: rows carry a `service` column (`workflow_trace`, `habitat_conductor`, `me_v2`, etc.) and the engine MUST filter to its own service on every read (`WHERE service = 'workflow_trace'`). This is an **engine-authored** namespace guard (m9), NOT substrate-authored.

## 8. Substrate-internal couplings (closes NA-GAP-03)

injection.db's substrate-internal edges:
- **habitat-memory daemon ← shell SessionEnd hook** — writes new causal chains based on session outcome.
- **habitat-memory daemon → habitat-inject hook (read)** — pre-warms next session's context window.
- **stcortex (S-C) → injection.db (S-B) via habitat-memory's reinforcement path** — this is the **hidden CC-5 edge** that NA-GAP-03 makes explicit. m42 writes to S-C → habitat-memory's downstream reads → S-B reinforcement_count updated → next session's injection prefers that pattern.

This edge is NOT engine-controlled; it is substrate-coordinated. The engine should NOT assume the edge fires within any specific window; it should observe it externally via Watcher.

## 9. Test-fixture sketch (closes NA-GAP-08)

Fixtures at `tests/substrate_fixtures/injection_db/`:

- **`db_locked_during_sweep_fixture`** — emulated DB with sustained exclusive lock; asserts m3 backs off.
- **`schema_drift_fixture`** — emulated schema with renamed column; asserts m3 fails closed.
- **`reinforcement_count_inflation_fixture`** — reported count diverges from re-derived count; asserts substrate-drift canary fires (parallels CR-2 POVM case).
- **`ttl_sweep_deletes_test_rows_fixture`** — asserts tests use realistic timestamps (per `feedback_ttl_sweep_test_timestamps.md`).
- **`causal_chain_empty_fixture`** — asserts m3 treats empty result as "no priors" not error.

## 10. Watcher class pre-positions

- **Class D (drift)** — schema-hash mismatch; reinforcement_count semantic drift
- **Class I (Hebbian silence)** — `reinforcement_count` delta zero over 4+ weeks on `workflow_trace_*` chains
- **Class B (boundary)** — SubstrateUnavailable from m3
- **Class E (ancestor-rhyme)** — when re-derived pattern matches an older deprecated chain still in DB

---

> **Back to:** [`INDEX.md`](../INDEX.md) · [`../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md)

*Filed 2026-05-17 (S1002127 · Wave 4 NA-remediation) · Luke "as per proposal" · planning-only · HOLD-v2 compliant.*
