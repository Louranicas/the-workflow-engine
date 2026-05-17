---
title: DATABASE_SPEC — SQLite migrations + atuin/injection.db read contracts + stcortex namespace
date: 2026-05-17
status: SPEC
substrates: [workflow-trace SQLite, atuin SQLite, injection.db SQLite, stcortex SpacetimeDB]
---

# DATABASE_SPEC — workflow-trace

> **Back to:** [`INDEX.md`](INDEX.md) · [`MODULE_MATRIX.md`](MODULE_MATRIX.md) · [`../README.md`](../README.md) · [`../ARCHITECTURE.md`](../ARCHITECTURE.md) · [`cross-cutting/persistence.md`](cross-cutting/persistence.md)

## Substrate inventory

| Substrate | Path | Pragma | Owner | Workflow-trace access |
|---|---|---|---|---|
| **workflow-trace bank** | `~/.local/share/workflow-trace/db.sqlite` | WAL + busy_timeout=5000 + foreign_keys=ON + synchronous=NORMAL | workflow-trace itself | read + write |
| **atuin SQLite** | `~/.local/share/atuin/history.db` | WAL + query_only=ON | atuin (external) | read-only (m1) |
| **injection.db SQLite** | `~/.local/share/habitat/injection.db` | WAL + query_only=ON | habitat-memory (external) | read-only partitioned (m3) |
| **stcortex** | `127.0.0.1:3000` (SpacetimeDB) | consumer subscription + HTTP write | stcortex (external) | read via consumer (m2); write via m13 (m42 indirect) |

---

## 1. workflow-trace bank SQLite — schema

Single DB file holds tables for m7 (workflow_runs), m11 (decay sweep state), m14 (lift caches), m15 (reservation log), m30 (bank), m33 (verify_results). Per-module prefix isolation.

### `migrations/0001_workflow_runs.sql` (m7 hub schema)

```sql
CREATE TABLE workflow_runs (
    id                 TEXT PRIMARY KEY,           -- workflow run id (UUIDv7)
    workflow_id        TEXT NOT NULL,              -- references accepted_workflows.id
    cluster_id         TEXT NOT NULL,              -- opaque FNV-1a-16-hex from m4
    started_at         INTEGER NOT NULL,           -- unix ms
    completed_at       INTEGER,                    -- unix ms or NULL if in-flight
    outcome            TEXT,                       -- 'pass_verified'|'pass'|'fail'|'blocked'|NULL
    consumer_inputs    TEXT NOT NULL DEFAULT '{}', -- JSONB blob (CC-1 hub column)
    fitness_dimension  REAL NOT NULL DEFAULT 0.0,  -- F9 zero-weight column
    created_at         INTEGER NOT NULL,
    CHECK (fitness_dimension BETWEEN -1.0 AND 1.0)
);

CREATE INDEX idx_runs_workflow_id ON workflow_runs(workflow_id);
CREATE INDEX idx_runs_cluster_outcome ON workflow_runs(cluster_id, outcome);
CREATE INDEX idx_runs_created_at ON workflow_runs(created_at);

-- ConsumerInputs JSONB shape (validated by workflow_core::schemas::ConsumerInputs):
-- {
--   "cascade": { "cluster_id": "<hex>", "step_count": <int>, "fnv_xor_signature": "<hex>" },
--   "cost":    { "session_type": "<str>", "ema_mean": <f64>, "ema_variance": <f64>, "n": <int> },
--   "battern": { "battern_id": "<id>", "session_id": "<id>", "step_index": <int>, "step_token": "<str>" },
--   "causal":  { "chain_id": "<id>", "resolved": <bool> }
-- }
```

### `migrations/0002_bank.sql` (m30 curated bank schema)

```sql
CREATE TABLE accepted_workflows (
    id                       TEXT PRIMARY KEY,         -- UUIDv7
    lineage                  TEXT NOT NULL,            -- LineageId opaque
    accepted_at              INTEGER NOT NULL,
    accepted_by              TEXT NOT NULL CHECK (accepted_by NOT IN ('', 'agent', 'auto')),
    sunset_at                INTEGER NOT NULL,
    ralph_decay_weight       REAL NOT NULL DEFAULT 1.0,
    ember_state_json         TEXT,
    escape_surface_profile   TEXT NOT NULL,            -- snake_case enum serde
    steps_json               TEXT NOT NULL,            -- canonicalised
    definition_hash          TEXT NOT NULL,            -- FNV-1a 64-bit hex
    curator_note             TEXT NOT NULL DEFAULT '',
    last_verified_at         INTEGER,
    dispatch_count           INTEGER NOT NULL DEFAULT 0,
    CHECK (sunset_at > accepted_at),
    CHECK (ralph_decay_weight BETWEEN 0.0 AND 1.0),
    CHECK (escape_surface_profile IN
        ('sandboxed','sandbox_escape','process_mutate','file_write','network_egress','data_exfil'))
);

CREATE INDEX idx_bank_weight_sunset ON accepted_workflows(ralph_decay_weight DESC, sunset_at);
CREATE INDEX idx_bank_sunset ON accepted_workflows(sunset_at);

CREATE TABLE dispatch_audit_log (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    workflow_id     TEXT NOT NULL REFERENCES accepted_workflows(id),
    event_kind      TEXT NOT NULL,  -- 'accept'|'dispatch'|'verify'|'decay'
    actor           TEXT NOT NULL,
    occurred_at     INTEGER NOT NULL,
    payload_json    TEXT NOT NULL
);

CREATE INDEX idx_audit_wf_time ON dispatch_audit_log(workflow_id, occurred_at);
```

### `migrations/0003_verify_results.sql` (m33 schema)

```sql
CREATE TABLE verify_results (
    workflow_id          TEXT PRIMARY KEY REFERENCES accepted_workflows(id),
    definition_hash      TEXT NOT NULL,        -- m32 reads to match
    verdict              TEXT NOT NULL CHECK (verdict IN ('pass','fail','degraded')),
    verified_at          INTEGER NOT NULL,
    ttl_expires_at       INTEGER NOT NULL,     -- verified_at + 7d
    agents_passed_json   TEXT NOT NULL,        -- {"zen":"pass","security_auditor":"pass",...}
    notes                TEXT,
    CHECK (ttl_expires_at > verified_at)
);

CREATE INDEX idx_verify_expiry ON verify_results(ttl_expires_at);
```

### `migrations/0004_reservation_log.sql` (m15 schema)

```sql
CREATE TABLE reservation_events (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    event_kind      TEXT NOT NULL,           -- 'forbidden_verb'|'sample_size_relax'|'scope_relax'|'handshake_silence'|'escape_surface_escalation'
    context_hash    INTEGER NOT NULL,        -- for 60s dedup
    emitted_at      INTEGER NOT NULL,
    count           INTEGER NOT NULL DEFAULT 1,
    jsonl_path      TEXT NOT NULL,           -- agent-cross-talk filename
    payload_json    TEXT NOT NULL
);

CREATE INDEX idx_reservation_dedup ON reservation_events(event_kind, context_hash, emitted_at);
```

### `migrations/0005_decay_state.sql` (m11 sweep tracking)

```sql
CREATE TABLE decay_sweep_state (
    id              INTEGER PRIMARY KEY,
    last_sweep_at   INTEGER NOT NULL,
    rows_updated    INTEGER NOT NULL,
    sweep_duration_ms  INTEGER NOT NULL
);
```

### `migrations/0006_lift_cache.sql` (m14 read-cache)

```sql
CREATE TABLE lift_cache (
    cluster_id      TEXT PRIMARY KEY,
    n               INTEGER NOT NULL,
    success         INTEGER NOT NULL,
    lower_bound     REAL,    -- NULL when n<20 (None propagation)
    point           REAL,
    upper_bound     REAL,
    computed_at     INTEGER NOT NULL
);
```

---

## 2. atuin SQLite reader contract (m1 access pattern)

**Read-only, cursor-based pagination, WAL busy-timeout 5000 ms.**

```sql
-- m1 query template
SELECT id, command, session, hostname, timestamp_ms, exit_code, duration_ms, cwd
  FROM history
 WHERE id > ?
 ORDER BY id ASC
 LIMIT ?;
```

- **PRAGMA query_only = ON** on the connection (write-prevention at SQLite layer).
- **PRAGMA busy_timeout = 5000** absorbs atuin's concurrent WAL writes.
- **Subprocess fallback** — if SQLite open fails or BUSY persists past timeout, fall back to `atuin history list --format json --limit N` with 5000 ms subprocess timeout.

## 3. injection.db SQLite reader contract (m3 access pattern)

**Read-only, partitioned by resolution status.**

```sql
-- m3 unresolved partition
SELECT id, label, chain_kind, reinforcement_count, first_seen_at, last_seen_at
  FROM causal_chain
 WHERE resolved_session IS NULL
 ORDER BY id ASC;

-- m3 resolved partition
SELECT id, label, chain_kind, reinforcement_count, resolved_session, resolved_at
  FROM causal_chain
 WHERE resolved_session IS NOT NULL
 ORDER BY resolved_at DESC
 LIMIT 100;
```

- **PRAGMA query_only = ON**.
- **Schema is owned by habitat-memory injector** — workflow-trace never migrates injection.db.
- **Future schema additions** to `causal_chain` MUST be additive (workflow-trace's reader is defensive on missing columns).

## 4. stcortex namespace table

stcortex is a SpacetimeDB module; the "tables" are reducer-callback streams + write APIs, not relational SQL. Workflow-trace's namespace contract:

| Namespace | Used by | Purpose |
|---|---|---|
| `workflow_trace_consumer_{instance_id}` | m2 (register consumer) | runtime consumer registration |
| `workflow_trace_{workflow_id}` | m42 (via m13) | per-workflow pathway prefix |
| `workflow_trace_outcome_{outcome}` | m42 (via m13) | per-outcome pathway prefix |
| `workflow_trace_pathway_{pre_id}__{post_id}` | m42 (via m13) | bidi pathway pair (hyphen-slug encoded with `_`) |

**AP30 enforced at m9 namespace-guard** — every write through m13 calls `m9::assert_namespace(id)` which validates the `workflow_trace_*` prefix against `workflow_core::namespace::WORKFLOW_TRACE_PREFIX` constant. Literal `"workflow_trace_*"` strings forbidden in source outside `namespace.rs`.

**AP-Hab-11 hyphen-slug encoding** — `pre_id`/`post_id` slugs convert `-` → `_` at the slug boundary (S1001757 munge bug). `workflow_trace_wf-abc-123` → `workflow_trace_wf_abc_123`.

## Migration sequence pre/post G9

**Pre-G9 (planning-only):** zero migrations land. The migrations above are the v1.3 spec; they are NOT executed against any database. `.claude/anti_patterns.json` includes a pre-G9 hook to refuse any `migrate up` invocation under the workflow-trace project root.

**Post-G9 sequence:**

1. Wave 1 (Day 1) — Cluster D ships. No DB schema yet.
2. Wave 2 Day 2 — Cluster A reads atuin/injection.db (no workflow-trace bank schema yet); Cluster B observation lands.
3. Wave 2 Day 3 — Cluster C ships m7 → migration 0001 lands. `lift_cache` (migration 0006) lands alongside m14 in Cluster E.
4. Wave 3 Day 4 — Cluster F + G ship → migrations 0002 (bank), 0003 (verify), 0004 (reservation log via m15), 0005 (decay state via m11) land.

All migrations are `IF NOT EXISTS` safe; re-run on restart.

## Verify-sync invariants

- Schema integrity: every CHECK constraint has a unit test asserting violations are rejected.
- Consumer schema agreement: `workflow_core::schemas::ConsumerInputs` deserialises every JSONB blob produced by m4/m5/m6 (contract test).
- AP30 enforcement: `rg '"workflow_trace_' src/ | grep -v 'namespace.rs\|m9_watcher'` returns 0.

---

> **Back to:** [`INDEX.md`](INDEX.md) · [`cross-cutting/persistence.md`](cross-cutting/persistence.md)
