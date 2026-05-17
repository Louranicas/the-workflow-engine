---
title: cross-cutting/persistence — SQLite + JSONL outbox + stcortex (multi-substrate discipline)
date: 2026-05-17
status: SPEC
axes: [sqlite, jsonl-outbox, stcortex, multi-substrate]
---

# Persistence — Module-Side Guidance

> **Back to:** [`../INDEX.md`](../INDEX.md) · [`../DATABASE_SPEC.md`](../DATABASE_SPEC.md) · [`../../README.md`](../../README.md) · [`../../ARCHITECTURE.md`](../../ARCHITECTURE.md)

## Substrate map (which goes where)

| Substrate | Path | Owner | Access | Used by |
|---|---|---|---|---|
| atuin SQLite | `~/.local/share/atuin/history.db` | atuin (external) | read-only (PRAGMA query_only=ON) | m1 |
| stcortex | `127.0.0.1:3000` (SpacetimeDB) | stcortex (external) | read-via-consumer + write-via-m13 | m2 (read), m13 (write), m42 (write via m13) |
| injection.db | `~/.local/share/habitat/injection.db` | habitat-memory (external) | read-only (partitioned) | m3 |
| workflow-trace bank | `~/.local/share/workflow-trace/db.sqlite` | workflow-trace itself | read + write (WAL) | m7 (workflow_runs), m11 (decay sweep), m14 (lift query), m15 (reservation log), m30 (bank), m33 (verify_results) |
| JSONL outbox | `~/.local/state/workflow-trace/outbox/m{40,41,42}/*.jsonl` | workflow-trace itself | append-only-via-atomic-rename | m40 / m41 / m42 |
| agent-cross-talk | `~/projects/shared-context/agent-cross-talk/*.jsonl` | shared filesystem | atomic write (tmp+rename) | m15 |

## SQLite — pragma discipline (workflow-trace bank)

```rust
// from m06_schema.rs lift (per multiple per-module specs)
fn configure_connection(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("
        PRAGMA journal_mode = WAL;
        PRAGMA busy_timeout = 5000;
        PRAGMA foreign_keys = ON;
        PRAGMA synchronous = NORMAL;
        PRAGMA wal_autocheckpoint = 100;
    ")
}
```

- **WAL mode** — concurrent readers + single writer.
- **busy_timeout = 5000 ms** — absorbs concurrent writers without immediate ERR_BUSY.
- **foreign_keys = ON** — relational integrity at engine level.
- **synchronous = NORMAL** — fsync per checkpoint, not per write (durability tradeoff acceptable for workflow-trace bank).
- **PRAGMA query_only = ON** added on connections to external substrates (atuin, injection.db).

## Single DB per binary (ORAC pattern, Axis 2)

`~/.local/share/workflow-trace/db.sqlite` is **a single SQLite file** holding tables for m7, m11, m14, m15, m30, m33. No separate DB per cluster. Migrations live under `migrations/` with sequential numbering. Each module's tables use a prefix (`bank_*`, `runs_*`, `verify_*`) to avoid name collision.

This is the canonical "single DB per binary" pattern lifted from ORAC. Pros: one connection pool; one migration framework; cross-table joins inside workflow-trace. Cons: schema drift in one module risks affecting unrelated modules — mitigated by per-prefix isolation and contract tests.

## JSONL outbox — durability discipline

Cluster H emit modules write outbox-first:

```rust
// m42 outbox-first pattern
async fn reinforce(&self, payload: ReinforcePayload) -> Result<ReinforceOutcome, ReinforceError> {
    // 1. write to outbox FIRST (durable record)
    let outbox_path = self.outbox_dir.join(format!("{}.jsonl.tmp", payload.request_id));
    let mut file = tokio::fs::File::create(&outbox_path).await?;
    file.write_all(serde_json::to_vec(&payload)?.as_slice()).await?;
    file.sync_data().await?;  // fsync the file before rename
    tokio::fs::rename(&outbox_path, outbox_path.with_extension("jsonl")).await?;

    // 2. THEN attempt the network RPC (best-effort)
    match self.stcortex_writer.write(payload).await {
        Ok(_) => Ok(ReinforceOutcome::Accepted),
        Err(StcortexError::Unreachable) => Ok(ReinforceOutcome::SubstrateUnavailable),
        Err(e) => Err(ReinforceError::Bridge(e)),
    }
}
```

- **Outbox file = canonical record.** Network success or failure does not affect the outbox row.
- **Atomic rename** — `*.jsonl.tmp` → `*.jsonl` only after `sync_data()`. Partial writes never observable.
- **Offline-snapshot replay** — when substrate returns, a background task scans `outbox/m{40,41,42}/` for unflushed records and replays.

## stcortex — multi-substrate write discipline

Per workspace CLAUDE.md row 8 (stcortex pioneer substrate policy):

- **READ both stcortex and POVM v2** (overlap → 2026-07-10) for *other services*.
- **For workflow-trace specifically: WRITE only stcortex** — POVM is decoupled per ADR 2026-05-17.
- **If `:3000` is unreachable**, m42 reads `data/snapshots/latest.json` (offline JSON fallback) and SKIPS WRITES (no silent POVM fall-through).
- **Namespace convention** — `<project>_<domain>_<key>` → for workflow-trace, `workflow_trace_<domain>_<key>` per AP30.

## Multi-substrate write order

For a single workflow outcome being persisted across surfaces:

1. **m32 writes m30 dispatch_count + audit row** (bank SQLite) — transactional.
2. **m32 fires fire-and-forget to Cluster H** — m40 + m41 + m42 each outbox-first.
3. **Each Cluster H module independently** writes outbox → attempts network → updates breaker state.

The order matters: **bank update is transactional + durable BEFORE Cluster H fan-out**. If bank write fails, no emit. If bank write succeeds and Cluster H emit fails, outbox carries the record for replay.

## Verify-sync invariants

- **#14** — outbox fsync before rename verified by integration test (kill process mid-write, assert no partial file).
- m9 namespace-guard called on every stcortex write path.

## Substrate dossier cross-reference (NA-GAP remediation Wave 4)

Per `ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`, each substrate in the table above now has a first-class dossier under [`../substrates/`](../substrates/) documenting its lifecycle, refusal modes, drift indicators, back-pressure signals, receipts, capabilities/namespaces, internal couplings, and test fixtures. Module-side persistence logic MUST cross-reference the dossier's § 3 (Refusal modes) and § 5 (Back-pressure signals) before authoring new persistence sites.

| Substrate (this table row) | Dossier |
|---|---|
| atuin SQLite | [`../substrates/atuin.md`](../substrates/atuin.md) |
| stcortex | [`../substrates/stcortex.md`](../substrates/stcortex.md) |
| injection.db | [`../substrates/injection_db.md`](../substrates/injection_db.md) |
| workflow-trace bank | (engine-internal; no dossier — owned by workflow-trace itself) |
| JSONL outbox | (engine-internal; no dossier — owned by workflow-trace itself) |
| agent-cross-talk | (operator-as-substrate; see [`../substrates/operator.md`](../substrates/operator.md)) |

Refusal-mode handling in this file aligns with the `RefusalToken` taxonomy at [`./refusal-taxonomy.md`](refusal-taxonomy.md); substrate-drift handling aligns with [`./substrate-drift.md`](substrate-drift.md).

---

> **Back to:** [`../INDEX.md`](../INDEX.md) · [`../DATABASE_SPEC.md`](../DATABASE_SPEC.md) · [`../substrates/`](../substrates/) · [`./refusal-taxonomy.md`](refusal-taxonomy.md) · [`./substrate-drift.md`](substrate-drift.md)
