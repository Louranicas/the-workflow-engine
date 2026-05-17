---
substrate_id: S-A-atuin
kind: sql
read_endpoints: ["~/.local/share/atuin/history.db (file-path, ro)", "atuin kv get --namespace <ns>"]
write_endpoints: ["atuin internal daemon writes only (NEVER engine-written)"]
lifecycle_phases: [cold-start, warming, steady-state, degraded, refusing, dead]
refusal_modes: [database_locked, wal_contention, sqlite_busy, page_cache_pressure, kv_namespace_missing]
drift_indicators: [schema_migration_silent, history_table_column_rename, wal_checkpoint_cadence_change, kv_namespace_relocation]
back_pressure_signals: [busy_timeout_exceeded, read_lock_held_ms, page_cache_eviction_rate, wal_size_bytes]
consent_dimensions: [n/a — substrate is not an operator]
status: SPEC
date: 2026-05-17
authority: Luke @ node 0.A — S1002127 "as per proposal"
hold_v2_compliant: true
addresses: [NA-GAP-01, NA-GAP-04, NA-GAP-07, NA-GAP-08, NA-GAP-09, NA-GAP-10]
---

# S-A — atuin (shell-history SQLite substrate)

> **Back to:** [`INDEX.md`](../INDEX.md) · [`../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md) · [`../cross-cutting/persistence.md`](../cross-cutting/persistence.md) · [`../cross-cutting/refusal-taxonomy.md`](../cross-cutting/refusal-taxonomy.md) · [`../cross-cutting/substrate-drift.md`](../cross-cutting/substrate-drift.md) · sister substrates [`stcortex.md`](stcortex.md) · [`injection_db.md`](injection_db.md) · [`synthex.md`](synthex.md) · [`lcm.md`](lcm.md) · [`conductor.md`](conductor.md) · [`watcher.md`](watcher.md) · [`operator.md`](operator.md)
>
> Read-side consumer in `workflow-trace`: **m1** ([`../modules/cluster-A/m1_atuin_consumer.md`](../modules/cluster-A/m1_atuin_consumer.md))

## 1. Purpose & boundary

atuin is the habitat's **executed-action memory** — every shell tool-call across every Claude / Luke / Cipher / Zen session lands here as a byte-preserved row. ~263k rows across a single SQLite file in WAL mode. atuin's daemon owns the write path; the engine is a *read-only consumer* via file-path SQLite open with `mode=ro` URI and `PRAGMA query_only=ON`.

**IN scope for `workflow-trace`:** read `history` table (and optionally a small set of derived views) via paginated cursor; read `atuin kv get --namespace habitat` for fleet state; observe atuin's own backpressure signals to throttle m1's read cadence.

**OUT of scope:** writing to atuin (atuin's daemon owns this); causing schema migrations; modifying atuin config; restarting atuin; pinning atuin to a specific version. The engine treats atuin as a **co-tenant**, not a managed resource.

## 2. Lifecycle phases

| Phase | Indicator | Engine action |
|---|---|---|
| cold-start | atuin daemon recently started, WAL has unmerged segments | m1 backs off 500ms; reads tolerated but `wal_size_bytes > 64MB` is a yellow flag |
| warming | WAL stable; recent shell activity flushing into history | normal read cadence |
| steady-state | history table size growing linearly with shell activity; busy_timeout rare | normal read cadence; m1 cursor advance every N seconds |
| degraded | `database is locked` rate > 1/min over rolling 5 min; WAL > 256MB | m1 backs off to half-rate; emits substrate-drift candidate event |
| refusing | every read returns `SqliteError::Busy` despite 5s timeout | m1 returns `IngestError::SubstrateBusy`; pressure-register event fired |
| dead | file missing, file unreadable, schema-detection fails | m1 returns `IngestError::SubstrateUnavailable`; engine continues with stale cache |

## 3. Refusal modes (substrate-authored, NOT engine-authored)

- **`SqliteError::Busy`** — atuin's daemon holds an exclusive WAL writer lock that exceeded the 5000ms busy_timeout. Substrate-authored. Recovery hint: exponential backoff 500ms→5s, max 3 retries.
- **`SqliteError::DatabaseLocked`** — atuin is mid-checkpoint. Substrate-authored. Recovery hint: wait for next checkpoint window (typically 100 WAL pages).
- **`KvNamespaceMissing`** — `atuin kv get --namespace <ns> <key>` returns empty because namespace was never created. Substrate-authored (atuin's KV layer). Recovery hint: treat as "no data"; do not error.
- **Schema-detect refusal** — m1 opens the DB and the `history` table is missing or has unexpected column types. Substrate-authored (atuin upgraded silently). Recovery hint: pin atuin version in `flake.nix`; fail closed.

These map onto `RefusalToken::SubstrateAuthored { substrate_id: "atuin", class, repair_hint }` per [`../cross-cutting/refusal-taxonomy.md`](../cross-cutting/refusal-taxonomy.md). Distinct from `EngineAuthored` refusals like m9 namespace-guard rejection.

## 4. Drift indicators (closes NA-GAP-07)

atuin can drift semantically without health-200 failing, because there is no `/health` endpoint — only file presence. The CR-2 incident at POVM (`learning_health` 0.9146 → 0.067 post-fix; 13.6× inflation factor; see [`../cross-cutting/substrate-drift.md`](../cross-cutting/substrate-drift.md)) is the canonical analog: a substrate that returns "200 OK" while serving a different formula. atuin's analogues:

- **Schema migration silent** — atuin daemon upgrade adds/renames a column; m1 reads the old column name and gets `NULL`. Detector: `PRAGMA table_info(history)` hash compared against a frozen baseline at engine start.
- **Cursor semantic change** — atuin's `id` column changes from autoincrement to ULID; m1's cursor-monotonicity assumption breaks silently. Detector: cursor invariant test on canary rows.
- **WAL checkpoint cadence change** — atuin tunes wal_autocheckpoint downward; m1's reads see stale data. Detector: read-staleness probe (write timestamp on shell side, latency measured at m1 read side).
- **KV namespace relocation** — atuin moves `habitat` namespace to `habitat.v2`; engine reads return empty. Detector: namespace-presence canary check at session start.

## 5. Back-pressure signals (closes NA-GAP-04)

atuin signals "slow down" implicitly. Engine must observe these substrate-side metrics and self-throttle m1's read cadence:

- **`busy_timeout` exceeded rate** — > 1/min indicates atuin's daemon is fighting for the WAL.
- **`PRAGMA wal_checkpoint(PASSIVE)` returns `busy=1`** — atuin holds the checkpoint lock; m1 should defer.
- **WAL size > 256MB** — atuin's checkpointer is behind; m1's reads pay extra random-IO cost on every page miss.
- **m1's own read-latency p99 > 100ms** — emergent signal that something in the substrate is fighting.

The engine has NO ability to tell atuin "slow your writes"; it can only slow its own reads. This asymmetry is named explicitly here so future module authors don't assume bidirectional flow-control.

## 6. Receipts (closes NA-GAP-09)

atuin does NOT emit substrate-side receipts when m1 reads it. Engine-side receipt is m4 cascade correlator's record that "rows N..M read at timestamp T". This is the gap NA-GAP-09 surfaces: CC-5 cannot be substrate-confirmed at the atuin end. Mitigation: the engine should NOT depend on atuin acknowledging reads — instead it should verify reads succeeded via the cursor-monotonicity invariant (the cursor advanced ⇒ rows were read).

## 7. Capabilities & namespaces (closes NA-GAP-10)

atuin authorises:
- **READ** on `history` table for any consumer (no per-consumer ACL)
- **READ** on `kv` table scoped by `--namespace`
- **WRITE** authorised ONLY for atuin's own daemon (engine MUST NOT write)

There is no substrate-side "trust score" for the engine; atuin treats every reader as anonymous. The engine's namespace boundary is enforced engine-side via m9 (`workflow_trace_*` AP30 prefix), not substrate-side. This is a structural asymmetry: writes to stcortex are gated at the substrate (refuse-write at DB layer); reads from atuin are gated only at the engine.

## 8. Substrate-internal couplings (closes NA-GAP-03)

atuin reads from / writes to:
- **atuin daemon's own KV store** (same SQLite file) — `habitat` namespace is a write/read cycle internal to atuin
- **shell wrapper** (bash hook `__atuin_history_add`) — writes to atuin on every command
- **`fleet-heartbeat` script** — writes to `atuin kv` `habitat.fleet.heartbeat_*` keys

NO direct substrate-substrate edges to stcortex / injection.db / synthex. atuin is a **terminal sink** of shell activity, not a router.

## 9. Test-fixture sketch (closes NA-GAP-08, post-G9)

Fixtures needed at `tests/substrate_fixtures/atuin/` (post-G9 implementation; spec only here):

- **`atuin_wal_contention_fixture`** — emulated SQLite file with a sustained writer lock; asserts m1 backs off correctly.
- **`atuin_schema_drift_fixture`** — emulated DB with renamed column; asserts m1 fails closed with structured error.
- **`atuin_empty_kv_namespace_fixture`** — asserts m1 treats missing namespace as no-data, not error.
- **`atuin_cursor_id_change_fixture`** — emulated `id` column type change; asserts cursor-monotonicity invariant test catches it.

## 10. Watcher class pre-positions

Watcher classes that pre-position on atuin anomalies:
- **Class D (drift)** — schema-detect hash change between engine starts
- **Class I (Hebbian silence)** — m1's row-read rate goes to zero (atuin has stopped accepting new shell activity)
- **Class B (boundary)** — m1 read failure surfaces as IngestError
- **Class C (refusal)** — atuin returns `SqliteError::Busy`; this is correct substrate behaviour, not a bug — Class C count should be non-zero in steady-state

---

> **Back to:** [`INDEX.md`](../INDEX.md) · [`../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md) · sister substrates index above

*Filed 2026-05-17 (S1002127 · Wave 4 NA-remediation) · Luke "as per proposal" · planning-only · HOLD-v2 compliant.*
