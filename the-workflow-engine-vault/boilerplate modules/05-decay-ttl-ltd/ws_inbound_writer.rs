//! `ws_inbound_writer` — 7th daemon task.
//!
//! Drains the bounded mpsc fed by `ws_handler::dispatch` (capture-first
//! ordering, S117 spec § 3.3) into the `ws_inbound_events.db` `SQLite`
//! WAL table created by migration 016.
//!
//! Cadence:
//!  - Per-event: synchronous `INSERT` on receipt (rusqlite is sync, but
//!    expected throughput ≤ 5 frames/s × few connections — the per-row
//!    latency at WAL+NORMAL is well under the daemon tick budget).
//!  - Hourly: TTL sweep deleting rows older than 7 days
//!    ([`WS_INBOUND_RETENTION_MS`] / [`WS_INBOUND_TTL_SWEEP_INTERVAL_S`]).
//!
//! Shutdown semantics (Round 6.2 of the QA decision record):
//!  - `cancel.cancelled()` short-circuits the main loop.
//!  - A *drain budget* of 500 ms is then granted: any remaining frames in
//!    the channel are flushed within that window.
//!
//! Errors are surfaced via the central [`SynthexError`](crate::SynthexError)
//! taxonomy (`Database::Query` for rusqlite failures). The task body logs
//! per-error and continues — a single bad row must not crash the writer.
//!
//! ## Send safety
//!
//! `rusqlite::Connection` is `Send` but `!Sync`, so `&Connection` is
//! `!Send`. Holding an owned `Connection` across `tokio::select!` blocks
//! the generated future's `Send` bound because the macro may capture
//! interior references. We solve this by wrapping in
//! `parking_lot::Mutex<Connection>` (which is `Send + Sync` when `T:
//! Send`). The lock guard lives only inside synchronous blocks — never
//! across an `.await` point.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex as PlMutex;
use rusqlite::{params, Connection};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::{interval, Instant, MissedTickBehavior};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use crate::daemon::state::AppState;
use crate::daemon::ws_handler::{
    WsInboundEvent, WS_INBOUND_DB_RELATIVE_PATH, WS_INBOUND_RETENTION_MS,
    WS_INBOUND_TTL_SWEEP_INTERVAL_S,
};
use crate::m1_foundation::m02_error_taxonomy::DatabaseError;
use crate::Result;

/// Drain budget granted post-cancel before the task gives up on the
/// remaining buffered frames. Spec § 6.2.
const SHUTDOWN_DRAIN_BUDGET_MS: u64 = 500;

/// DB tag used in error messages — keeps the [`DatabaseError`] taxonomy
/// consistent with the migration filename.
const DB_TAG: &str = "ws_inbound_events";

/// Spawn the writer at the default `ws_inbound_events.db` location.
///
/// Takes `Arc<AppState>` by value to match the other daemon-task spawn
/// signatures. Internally this writer only needs the receiver, so the
/// Arc is dropped immediately — but the uniform API keeps `runtime.rs`
/// call sites consistent.
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn spawn(state: Arc<AppState>, cancel: CancellationToken) -> Option<JoinHandle<()>> {
    spawn_with_path(&state, cancel, PathBuf::from(WS_INBOUND_DB_RELATIVE_PATH))
}

/// Spawn the writer at an explicit DB path (used by tests with a tempdir).
#[must_use]
pub fn spawn_with_path(
    state: &AppState,
    cancel: CancellationToken,
    db_path: PathBuf,
) -> Option<JoinHandle<()>> {
    let rx = state.take_inbound_writer_rx()?;
    Some(tokio::spawn(async move {
        if let Err(e) = run(db_path, rx, cancel).await {
            error!(target = "ws_inbound_writer", error = %e, "writer task exited with error");
        }
    }))
}

/// Async loop body — separated from `spawn` so tests can drive it
/// directly without going through [`AppState`].
///
/// # Errors
///
/// Returns the first non-recoverable error from opening the DB. Per-row
/// insert and TTL sweep failures are *logged and continued* — see the
/// task body — so a single bad row never crashes the writer.
pub async fn run(
    db_path: PathBuf,
    mut rx: mpsc::Receiver<WsInboundEvent>,
    cancel: CancellationToken,
) -> Result<()> {
    let raw = open_db_blocking(db_path.clone()).await?;
    let conn = PlMutex::new(raw);

    let mut ttl_tick = interval(Duration::from_secs(WS_INBOUND_TTL_SWEEP_INTERVAL_S));
    ttl_tick.set_missed_tick_behavior(MissedTickBehavior::Delay);

    info!(
        target = "ws_inbound_writer",
        path = %db_path.display(),
        retention_ms = WS_INBOUND_RETENTION_MS,
        sweep_s = WS_INBOUND_TTL_SWEEP_INTERVAL_S,
        "writer task starting"
    );

    let mut written: u64 = 0;
    let mut swept: u64 = 0;

    loop {
        tokio::select! {
            biased;
            () = cancel.cancelled() => {
                info!(target = "ws_inbound_writer", written, swept, "cancel observed");
                break;
            }
            ev = rx.recv() => {
                let Some(ev) = ev else {
                    info!(target = "ws_inbound_writer", written, "sender dropped");
                    return Ok(());
                };
                if let Err(e) = write_event(&conn, &ev) {
                    warn!(target = "ws_inbound_writer", error = %e, "row insert failed");
                } else {
                    written += 1;
                }
            },
            _ = ttl_tick.tick() => {
                match sweep_ttl(&conn, now_ms()) {
                    Ok(n) => {
                        swept += n;
                        if n > 0 {
                            info!(target = "ws_inbound_writer", deleted = n, swept, "ttl sweep");
                        }
                    }
                    Err(e) => warn!(target = "ws_inbound_writer", error = %e, "ttl sweep failed"),
                }
            }
        }
    }

    drain_remaining(&conn, &mut rx).await;
    Ok(())
}

async fn drain_remaining(
    conn: &PlMutex<Connection>,
    rx: &mut mpsc::Receiver<WsInboundEvent>,
) {
    let deadline = Instant::now() + Duration::from_millis(SHUTDOWN_DRAIN_BUDGET_MS);
    let mut drained: u64 = 0;
    loop {
        tokio::select! {
            () = tokio::time::sleep_until(deadline) => break,
            ev = rx.recv() => {
                let Some(ev) = ev else { break };
                if let Err(e) = write_event(conn, &ev) {
                    warn!(target = "ws_inbound_writer", error = %e, "drain insert failed");
                } else {
                    drained += 1;
                }
            }
        }
    }
    info!(target = "ws_inbound_writer", drained, "drain budget elapsed");
}

async fn open_db_blocking(path: PathBuf) -> Result<Connection> {
    tokio::task::spawn_blocking(move || open_db(&path))
        .await
        .map_err(|e| DatabaseError::Connection {
            db: DB_TAG,
            message: format!("open join: {e}"),
        })?
}

fn open_db(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path).map_err(|e| DatabaseError::Connection {
        db: DB_TAG,
        message: format!("open {}: {e}", path.display()),
    })?;
    // Match m15_sqlite_writer's open pattern exactly — individual
    // pragma_update calls. execute_batch wrapped these in a manner that
    // left subsequent INSERTs in an uncommitted implicit transaction
    // (writer's own SELECT returned 0 rows after Ok inserts).
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|e| DatabaseError::Query {
            db: DB_TAG,
            message: format!("pragma journal_mode=WAL: {e}"),
        })?;
    conn.pragma_update(None, "synchronous", "NORMAL")
        .map_err(|e| DatabaseError::Query {
            db: DB_TAG,
            message: format!("pragma synchronous=NORMAL: {e}"),
        })?;
    conn.busy_timeout(Duration::from_millis(500))
        .map_err(|e| DatabaseError::Query {
            db: DB_TAG,
            message: format!("busy_timeout: {e}"),
        })?;
    Ok(conn)
}

fn write_event(conn: &PlMutex<Connection>, ev: &WsInboundEvent) -> Result<()> {
    let payload_text =
        serde_json::to_string(&ev.payload_json).map_err(|e| DatabaseError::Query {
            db: DB_TAG,
            message: format!("serialise payload: {e}"),
        })?;
    conn.lock()
        .execute(
            "INSERT INTO ws_inbound_events
                (ts_ms, sender_sphere_id, message_type, payload_json,
                 processed_class, confidence_delta)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                ev.ts_ms,
                ev.sender_sphere_id,
                ev.message_type,
                payload_text,
                ev.processed_class,
                ev.confidence_delta,
            ],
        )
        .map_err(|e| DatabaseError::Query {
            db: DB_TAG,
            message: format!("insert: {e}"),
        })?;
    Ok(())
}

fn sweep_ttl(conn: &PlMutex<Connection>, now_ms: i64) -> Result<u64> {
    let cutoff =
        now_ms.saturating_sub(i64::try_from(WS_INBOUND_RETENTION_MS).unwrap_or(i64::MAX));
    let deleted = conn
        .lock()
        .execute(
            "DELETE FROM ws_inbound_events WHERE ts_ms < ?1",
            params![cutoff],
        )
        .map_err(|e| DatabaseError::Query {
            db: DB_TAG,
            message: format!("ttl sweep: {e}"),
        })?;
    Ok(deleted as u64)
}

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::{drain_remaining, now_ms, run, sweep_ttl, write_event, SHUTDOWN_DRAIN_BUDGET_MS};
    use crate::daemon::ws_handler::{WsInboundEvent, WS_INBOUND_RETENTION_MS};
    use parking_lot::Mutex as PlMutex;
    use rusqlite::Connection;
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::sync::mpsc;
    use tokio_util::sync::CancellationToken;

    fn apply_test_schema(conn: &Connection) {
        conn.execute_batch(
            r"PRAGMA journal_mode = WAL;
              PRAGMA synchronous = NORMAL;
              CREATE TABLE IF NOT EXISTS ws_inbound_events (
                  id                   INTEGER PRIMARY KEY AUTOINCREMENT,
                  ts_ms                INTEGER NOT NULL,
                  sender_sphere_id     TEXT    NOT NULL,
                  message_type         TEXT    NOT NULL,
                  payload_json         TEXT    NOT NULL,
                  processed_class      TEXT,
                  confidence_delta     REAL,
                  created_at           DATETIME DEFAULT CURRENT_TIMESTAMP
              );
              CREATE INDEX IF NOT EXISTS idx_ws_sender_ts
                  ON ws_inbound_events(sender_sphere_id, ts_ms);
              CREATE INDEX IF NOT EXISTS idx_ws_type_ts
                  ON ws_inbound_events(message_type, ts_ms);",
        )
        .unwrap();
    }

    fn test_conn() -> PlMutex<Connection> {
        let conn = Connection::open_in_memory().unwrap();
        apply_test_schema(&conn);
        PlMutex::new(conn)
    }

    fn sample_event(ts_ms: i64, sender: &str, ty: &str) -> WsInboundEvent {
        WsInboundEvent {
            ts_ms,
            sender_sphere_id: sender.to_string(),
            message_type: ty.to_string(),
            payload_json: serde_json::json!({"k": "v"}),
            processed_class: Some("Nominal".to_string()),
            confidence_delta: Some(0.42),
        }
    }

    fn count_rows(conn: &PlMutex<Connection>) -> i64 {
        conn.lock()
            .query_row("SELECT COUNT(*) FROM ws_inbound_events", [], |r| r.get(0))
            .unwrap()
    }

    #[test]
    fn write_event_inserts_one_row() {
        let conn = test_conn();
        write_event(&conn, &sample_event(1, "orac", "CapabilityTrace")).unwrap();
        assert_eq!(count_rows(&conn), 1);
    }

    #[test]
    fn write_event_persists_all_columns() {
        let conn = test_conn();
        write_event(&conn, &sample_event(123, "orac-pi-1", "Hello")).unwrap();
        let g = conn.lock();
        let (ts, sender, ty, payload, class, conf): (
            i64, String, String, String, Option<String>, Option<f64>,
        ) = g
            .query_row(
                "SELECT ts_ms, sender_sphere_id, message_type, payload_json, \
                 processed_class, confidence_delta FROM ws_inbound_events LIMIT 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)),
            )
            .unwrap();
        assert_eq!(ts, 123);
        assert_eq!(sender, "orac-pi-1");
        assert_eq!(ty, "Hello");
        let parsed: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(parsed, serde_json::json!({"k": "v"}));
        assert_eq!(class.as_deref(), Some("Nominal"));
        assert!((conf.unwrap() - 0.42).abs() < f64::EPSILON);
    }

    #[test]
    fn sweep_ttl_deletes_old_rows() {
        let conn = test_conn();
        let now = 1_000_000_000_000_i64;
        let stale = now - i64::try_from(WS_INBOUND_RETENTION_MS).unwrap() - 1;
        write_event(&conn, &sample_event(stale, "orac", "Hello")).unwrap();
        let deleted = sweep_ttl(&conn, now).unwrap();
        assert_eq!(deleted, 1);
        assert_eq!(count_rows(&conn), 0);
    }

    #[test]
    fn sweep_ttl_keeps_recent_rows() {
        let conn = test_conn();
        let now = 1_000_000_000_000_i64;
        write_event(&conn, &sample_event(now - 60_000, "orac", "Hello")).unwrap();
        let deleted = sweep_ttl(&conn, now).unwrap();
        assert_eq!(deleted, 0);
        assert_eq!(count_rows(&conn), 1);
    }

    #[test]
    fn sweep_ttl_at_boundary_keeps_row() {
        let conn = test_conn();
        let now = 1_000_000_000_000_i64;
        let boundary = now - i64::try_from(WS_INBOUND_RETENTION_MS).unwrap();
        write_event(&conn, &sample_event(boundary, "orac", "Hello")).unwrap();
        assert_eq!(sweep_ttl(&conn, now).unwrap(), 0);
    }

    #[test]
    fn now_ms_is_positive_and_recent() {
        let t = now_ms();
        assert!(t > 1_700_000_000_000);
    }

    #[tokio::test]
    async fn drain_budget_is_honored() {
        let conn = test_conn();
        let (tx, mut rx) = mpsc::channel::<WsInboundEvent>(64);
        for i in 0_i64..32 {
            tx.send(sample_event(i, "orac", "Hello")).await.unwrap();
        }
        drop(tx);
        let start = std::time::Instant::now();
        drain_remaining(&conn, &mut rx).await;
        assert!(start.elapsed() < Duration::from_millis(SHUTDOWN_DRAIN_BUDGET_MS + 200));
        assert_eq!(count_rows(&conn), 32);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn run_exits_on_cancel() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("ws.db");
        {
            let c = Connection::open(&path).unwrap();
            apply_test_schema(&c);
        }
        let cancel = CancellationToken::new();
        let (_tx, rx) = mpsc::channel::<WsInboundEvent>(8);
        let h = tokio::spawn({
            let c = cancel.clone();
            let p = path.clone();
            async move { run(p, rx, c).await }
        });
        tokio::time::sleep(Duration::from_millis(20)).await;
        cancel.cancel();
        tokio::time::timeout(Duration::from_millis(800), h)
            .await
            .expect("exit")
            .expect("join")
            .expect("ok");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn run_persists_pre_cancel_events() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("ws.db");
        // Keep a reader connection open for the entire test — this
        // prevents the tempdir cleanup from racing with the spawned
        // task's Connection::close and ensures the WAL is readable via
        // the same file handle that saw the schema DDL.
        let reader = Connection::open(&path).unwrap();
        apply_test_schema(&reader);
        let cancel = CancellationToken::new();
        let (tx, rx) = mpsc::channel::<WsInboundEvent>(16);
        let h = tokio::spawn({
            let p = path.clone();
            let c = cancel.clone();
            async move { run(p, rx, c).await }
        });
        // Use real-ish timestamps (now + i) so the writer's hourly TTL sweep
        // (cutoff = now - 7 days) does NOT immediately delete these rows.
        // The bug we caught with `ts_ms = i`: events at ts=0..5 were under
        // every plausible cutoff and got swept seconds after insert.
        let base_ts = now_ms();
        for i in 0_i64..5 {
            tx.send(sample_event(base_ts + i, "orac", "IntentToken"))
                .await
                .unwrap();
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
        cancel.cancel();
        drop(tx);
        let join_res = tokio::time::timeout(Duration::from_millis(800), h)
            .await
            .expect("exit within 800ms")
            .expect("join ok");
        join_res.expect("run should succeed");
        let n: i64 = reader
            .query_row("SELECT COUNT(*) FROM ws_inbound_events", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 5);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn run_exits_when_senders_drop() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("ws.db");
        {
            let c = Connection::open(&path).unwrap();
            apply_test_schema(&c);
        }
        let cancel = CancellationToken::new();
        let (tx, rx) = mpsc::channel::<WsInboundEvent>(4);
        let h = tokio::spawn(async move { run(path, rx, cancel).await });
        drop(tx);
        tokio::time::timeout(Duration::from_millis(400), h)
            .await
            .expect("exit")
            .expect("join")
            .expect("ok");
    }
}
