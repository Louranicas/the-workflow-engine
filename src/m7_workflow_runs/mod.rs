//! `m7_workflow_runs` — central SQLite hub for workflow trace records.
//!
//! F9 (workflow-grain fitness distortion) prime mitigation: the
//! `fitness_dimension REAL NOT NULL DEFAULT 0.0` column is structural —
//! the default is hardcoded at the DB layer; `insert_run` never accepts a
//! fitness argument; the column stays at zero until m11 / Hebbian v3
//! telemetry confirms the LTP/LTD threshold.

pub mod error;

use std::path::Path;
use std::time::Duration;

use rusqlite::{params, Connection};

pub use error::WorkflowError;

/// Outcome of a completed workflow run (closed enum mirrors DB CHECK).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Outcome {
    /// Workflow completed successfully.
    Ok,
    /// Workflow failed.
    Fail,
    /// Workflow was aborted by the operator or supervisor.
    Abort,
    /// Outcome could not be determined.
    Unknown,
}

impl Outcome {
    /// DB wire-form (stable; matches CHECK constraint).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Fail => "fail",
            Self::Abort => "abort",
            Self::Unknown => "unknown",
        }
    }

    /// Parse a DB string back into an [`Outcome`].
    ///
    /// # Errors
    ///
    /// [`WorkflowError::InvalidOutcome`] for any other value.
    pub fn parse(value: &str) -> Result<Self, WorkflowError> {
        match value {
            "ok" => Ok(Self::Ok),
            "fail" => Ok(Self::Fail),
            "abort" => Ok(Self::Abort),
            "unknown" => Ok(Self::Unknown),
            other => Err(WorkflowError::InvalidOutcome(other.to_owned())),
        }
    }
}

/// Step outcome for a single battern step observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StepOutcome {
    /// Step completed without error.
    Ok,
    /// Step failed.
    Fail,
    /// Step was skipped.
    Skipped,
}

impl StepOutcome {
    /// Stable wire-form.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Fail => "fail",
            Self::Skipped => "skipped",
        }
    }
}

/// One Cluster B observation discriminant + payload.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ClusterBObservation {
    /// m4 cascade correlator emission.
    Cascade {
        /// Opaque cluster id (F11 invariant; no human-meaningful labels).
        cluster_id: String,
        /// Min/max session_ns within the cascade window.
        session_range: (i64, i64),
    },
    /// m5 battern step observation.
    BatternStep {
        /// Opaque battern id.
        battern_id: String,
        /// Step position within the battern (0..=5 canonical, ≥0 actual).
        step_index: u8,
        /// Step duration (ms).
        duration_ms: u64,
        /// Step outcome.
        outcome: String,
    },
    /// m6 context cost observation.
    ContextCost {
        /// Session id (i64 alias for stcortex `claude_session.id`).
        session_id: i64,
        /// Token-count proxy.
        cost_tokens: u64,
    },
    /// Direct injection.db chain id reference.
    InjectionChain {
        /// Injection.db causal_chain.id.
        chain_id: i64,
    },
}

impl ClusterBObservation {
    /// Stable discriminant key used inside `consumer_inputs` JSONB.
    #[must_use]
    pub const fn discriminant(&self) -> &'static str {
        match self {
            Self::Cascade { .. } => "cascade",
            Self::BatternStep { .. } => "battern_step",
            Self::ContextCost { .. } => "context_cost",
            Self::InjectionChain { .. } => "injection_chain",
        }
    }
}

/// One row from the `workflow_runs` table.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkflowRunRow {
    /// Primary key.
    pub id: i64,
    /// Started at (RFC3339 string).
    pub started_at: String,
    /// Ended at; `None` while the run is open.
    pub ended_at: Option<String>,
    /// Outcome wire-form; `None` while the run is open.
    pub outcome: Option<String>,
    /// JSONB blob: stable CC-1 join surface.
    pub consumer_inputs: String,
    /// Token cost; `None` until m6 records.
    pub cost_tokens: Option<i64>,
    /// **F9 zero-weight column** — defaults to 0.0; only m11 writes.
    pub fitness_dimension: f64,
}

const SCHEMA_DDL: &str = "
CREATE TABLE IF NOT EXISTS workflow_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    started_at TEXT NOT NULL,
    ended_at TEXT,
    outcome TEXT CHECK(outcome IS NULL OR outcome IN ('ok','fail','abort','unknown')),
    consumer_inputs TEXT NOT NULL DEFAULT '{}',
    cost_tokens INTEGER,
    -- F9 zero-weight invariant: `fitness_dimension REAL NOT NULL DEFAULT 0.0`
    -- exists so future migrations adding a real fitness signal are additive.
    -- m11 (Hebbian v3 telemetry) is the ONLY writer once LTP/LTD > 0.15.
    fitness_dimension REAL NOT NULL DEFAULT 0.0
);
CREATE INDEX IF NOT EXISTS idx_workflow_runs_open
    ON workflow_runs(started_at DESC) WHERE ended_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_workflow_runs_outcome
    ON workflow_runs(outcome, ended_at DESC) WHERE outcome IS NOT NULL;
";

fn configure_connection(conn: &Connection) -> Result<(), WorkflowError> {
    conn.busy_timeout(Duration::from_secs(5))?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "wal_autocheckpoint", 100_i64)?;
    Ok(())
}

/// Open (or create) the `workflow_runs` database; run idempotent migrations.
///
/// # Errors
///
/// [`WorkflowError::Connection`] / [`WorkflowError::Migration`] on open or
/// DDL apply failure.
pub fn open_database(path: &Path) -> Result<Connection, WorkflowError> {
    let conn = Connection::open(path)
        .map_err(|e| WorkflowError::Connection(format!("open {}: {e}", path.display())))?;
    configure_connection(&conn)?;
    conn.execute_batch(SCHEMA_DDL)
        .map_err(|e| WorkflowError::Migration { step: 1, source: e })?;
    Ok(conn)
}

/// Open an in-memory database with full schema. Used in tests.
///
/// # Errors
///
/// See [`open_database`].
pub fn open_memory() -> Result<Connection, WorkflowError> {
    let conn = Connection::open_in_memory()
        .map_err(|e| WorkflowError::Connection(format!("memory open: {e}")))?;
    configure_connection(&conn)?;
    conn.execute_batch(SCHEMA_DDL)
        .map_err(|e| WorkflowError::Migration { step: 1, source: e })?;
    Ok(conn)
}

/// Insert a new open run.
///
/// # Errors
///
/// [`WorkflowError::Sqlite`] on insert failure.
pub fn insert_run(conn: &Connection, started_at: &str) -> Result<i64, WorkflowError> {
    conn.execute(
        "INSERT INTO workflow_runs (started_at) VALUES (?1)",
        params![started_at],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Merge a Cluster B observation into an existing run's `consumer_inputs`
/// blob (JSON patch under the observation's discriminant key).
///
/// # Errors
///
/// [`WorkflowError::RowNotFound`] if the run id does not exist.
/// [`WorkflowError::JsonPatch`] if the blob can't be parsed/serialised.
/// [`WorkflowError::Sqlite`] on the underlying UPDATE.
pub fn merge_observation(
    conn: &Connection,
    run_id: i64,
    obs: &ClusterBObservation,
) -> Result<(), WorkflowError> {
    let existing: String = conn
        .query_row(
            "SELECT consumer_inputs FROM workflow_runs WHERE id = ?1",
            params![run_id],
            |r| r.get(0),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => WorkflowError::RowNotFound { id: run_id },
            other => WorkflowError::Sqlite(other.to_string()),
        })?;
    let mut value: serde_json::Value = serde_json::from_str(&existing)?;
    let map = value.as_object_mut().ok_or_else(|| {
        WorkflowError::JsonPatch("consumer_inputs not a JSON object".into())
    })?;
    map.insert(obs.discriminant().to_owned(), serde_json::to_value(obs)?);
    let patched = serde_json::to_string(&value)?;
    conn.execute(
        "UPDATE workflow_runs SET consumer_inputs = ?1 WHERE id = ?2",
        params![patched, run_id],
    )?;
    Ok(())
}

/// Record `cost_tokens` directly on the run row.
///
/// # Errors
///
/// [`WorkflowError::Sqlite`] on update failure.
pub fn update_cost_tokens(
    conn: &Connection,
    run_id: i64,
    cost_tokens: i64,
) -> Result<(), WorkflowError> {
    conn.execute(
        "UPDATE workflow_runs SET cost_tokens = ?1 WHERE id = ?2",
        params![cost_tokens, run_id],
    )?;
    Ok(())
}

/// Close a run by recording `ended_at` + `outcome`.
///
/// # Errors
///
/// [`WorkflowError::InvalidOutcome`] if `outcome` is not in the CHECK set.
/// [`WorkflowError::Sqlite`] on update failure.
pub fn close_run(
    conn: &Connection,
    run_id: i64,
    ended_at: &str,
    outcome: &str,
) -> Result<(), WorkflowError> {
    // Validate before hitting the DB so callers see a clean variant.
    Outcome::parse(outcome)?;
    conn.execute(
        "UPDATE workflow_runs SET ended_at = ?1, outcome = ?2 WHERE id = ?3",
        params![ended_at, outcome, run_id],
    )?;
    Ok(())
}

fn parse_row(row: &rusqlite::Row<'_>) -> Result<WorkflowRunRow, WorkflowError> {
    Ok(WorkflowRunRow {
        id: row.get(0)?,
        started_at: row.get(1)?,
        ended_at: row.get(2)?,
        outcome: row.get(3)?,
        consumer_inputs: row.get(4)?,
        cost_tokens: row.get(5)?,
        fitness_dimension: row.get(6)?,
    })
}

const SELECT_COLS: &str =
    "SELECT id, started_at, ended_at, outcome, consumer_inputs, cost_tokens, fitness_dimension \
     FROM workflow_runs";

/// Fetch all open runs (ended_at IS NULL).
///
/// # Errors
///
/// [`WorkflowError::Sqlite`] on SELECT.
pub fn find_open(conn: &Connection, limit: usize) -> Result<Vec<WorkflowRunRow>, WorkflowError> {
    let limit_i = i64::try_from(limit).unwrap_or(i64::MAX);
    let sql = format!(
        "{SELECT_COLS} WHERE ended_at IS NULL ORDER BY started_at DESC LIMIT ?1"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_and_then(params![limit_i], parse_row)?
        .collect::<Result<Vec<_>, WorkflowError>>()?;
    Ok(rows)
}

/// Fetch completed runs with the given outcome.
///
/// # Errors
///
/// [`WorkflowError::InvalidOutcome`] if `outcome` not in the CHECK set.
/// [`WorkflowError::Sqlite`] on SELECT.
pub fn find_by_outcome(
    conn: &Connection,
    outcome: &str,
    limit: usize,
) -> Result<Vec<WorkflowRunRow>, WorkflowError> {
    Outcome::parse(outcome)?;
    let limit_i = i64::try_from(limit).unwrap_or(i64::MAX);
    let sql = format!(
        "{SELECT_COLS} WHERE outcome = ?1 ORDER BY ended_at DESC LIMIT ?2"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_and_then(params![outcome, limit_i], parse_row)?
        .collect::<Result<Vec<_>, WorkflowError>>()?;
    Ok(rows)
}

/// Find one run by primary key.
///
/// # Errors
///
/// [`WorkflowError::RowNotFound`] if no row with that id exists.
pub fn find_by_id(conn: &Connection, id: i64) -> Result<WorkflowRunRow, WorkflowError> {
    let sql = format!("{SELECT_COLS} WHERE id = ?1");
    let mut stmt = conn.prepare(&sql)?;
    let mut iter = stmt.query_and_then(params![id], parse_row)?;
    iter.next()
        .ok_or(WorkflowError::RowNotFound { id })?
}

#[cfg(test)]
mod tests {
    use super::{
        close_run, find_by_id, find_by_outcome, find_open, insert_run, merge_observation,
        open_memory, update_cost_tokens, ClusterBObservation, Outcome, StepOutcome,
        WorkflowError,
    };

    fn mem() -> rusqlite::Connection {
        open_memory().expect("memory open")
    }

    // ---- Outcome enum (5) -----------------------------------------------

    #[test]
    fn outcome_as_str_stable() {
        assert_eq!(Outcome::Ok.as_str(), "ok");
        assert_eq!(Outcome::Fail.as_str(), "fail");
        assert_eq!(Outcome::Abort.as_str(), "abort");
        assert_eq!(Outcome::Unknown.as_str(), "unknown");
    }

    #[test]
    fn outcome_parse_round_trip() {
        for o in [Outcome::Ok, Outcome::Fail, Outcome::Abort, Outcome::Unknown] {
            assert_eq!(Outcome::parse(o.as_str()).unwrap(), o);
        }
    }

    #[test]
    fn outcome_parse_rejects_unknown_string() {
        assert!(matches!(
            Outcome::parse("BAD"),
            Err(WorkflowError::InvalidOutcome(_))
        ));
    }

    #[test]
    fn step_outcome_as_str_stable() {
        assert_eq!(StepOutcome::Ok.as_str(), "ok");
        assert_eq!(StepOutcome::Skipped.as_str(), "skipped");
    }

    #[test]
    fn cluster_b_observation_discriminants_distinct() {
        let a = ClusterBObservation::Cascade {
            cluster_id: "cascade_cluster_x".into(),
            session_range: (0, 1),
        };
        let b = ClusterBObservation::BatternStep {
            battern_id: "battern_x".into(),
            step_index: 0,
            duration_ms: 1,
            outcome: "ok".into(),
        };
        let c = ClusterBObservation::ContextCost {
            session_id: 7,
            cost_tokens: 100,
        };
        let d = ClusterBObservation::InjectionChain { chain_id: 1 };
        assert_eq!(a.discriminant(), "cascade");
        assert_eq!(b.discriminant(), "battern_step");
        assert_eq!(c.discriminant(), "context_cost");
        assert_eq!(d.discriminant(), "injection_chain");
    }

    // ---- Schema + DDL (4) -----------------------------------------------

    #[test]
    fn open_memory_creates_workflow_runs_table() {
        let conn = mem();
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM workflow_runs", [], |r| r.get(0))
            .expect("count");
        assert_eq!(n, 0);
    }

    #[test]
    fn insert_run_returns_monotonic_ids() {
        let conn = mem();
        let id1 = insert_run(&conn, "2026-05-17T00:00:00Z").expect("insert");
        let id2 = insert_run(&conn, "2026-05-17T00:00:01Z").expect("insert");
        assert!(id2 > id1);
    }

    #[test]
    fn fitness_dimension_default_is_zero_f9_invariant() {
        let conn = mem();
        let id = insert_run(&conn, "2026-05-17T00:00:00Z").expect("insert");
        let row = find_by_id(&conn, id).expect("find");
        assert!(row.fitness_dimension.abs() < f64::EPSILON);
    }

    #[test]
    fn consumer_inputs_default_is_empty_object() {
        let conn = mem();
        let id = insert_run(&conn, "2026-05-17T00:00:00Z").expect("insert");
        let row = find_by_id(&conn, id).expect("find");
        assert_eq!(row.consumer_inputs, "{}");
    }

    // ---- CRUD lifecycle (6) ---------------------------------------------

    #[test]
    fn close_run_records_ended_at_and_outcome() {
        let conn = mem();
        let id = insert_run(&conn, "2026-05-17T00:00:00Z").expect("insert");
        close_run(&conn, id, "2026-05-17T01:00:00Z", "ok").expect("close");
        let row = find_by_id(&conn, id).expect("find");
        assert_eq!(row.outcome.as_deref(), Some("ok"));
        assert!(row.ended_at.is_some());
    }

    #[test]
    fn close_run_rejects_invalid_outcome() {
        let conn = mem();
        let id = insert_run(&conn, "2026-05-17T00:00:00Z").expect("insert");
        assert!(matches!(
            close_run(&conn, id, "2026-05-17T01:00:00Z", "weird"),
            Err(WorkflowError::InvalidOutcome(_))
        ));
    }

    #[test]
    fn merge_observation_appends_under_discriminant_key() {
        let conn = mem();
        let id = insert_run(&conn, "2026-05-17T00:00:00Z").expect("insert");
        merge_observation(
            &conn,
            id,
            &ClusterBObservation::Cascade {
                cluster_id: "cascade_cluster_abc".into(),
                session_range: (0, 100),
            },
        )
        .expect("merge");
        let row = find_by_id(&conn, id).expect("find");
        assert!(row.consumer_inputs.contains("cascade"));
        assert!(row.consumer_inputs.contains("cascade_cluster_abc"));
    }

    #[test]
    fn merge_observation_unknown_row_yields_row_not_found() {
        let conn = mem();
        let err = merge_observation(
            &conn,
            9999,
            &ClusterBObservation::ContextCost {
                session_id: 1,
                cost_tokens: 1,
            },
        )
        .unwrap_err();
        assert!(matches!(err, WorkflowError::RowNotFound { id: 9999 }));
    }

    #[test]
    fn update_cost_tokens_persists_value() {
        let conn = mem();
        let id = insert_run(&conn, "2026-05-17T00:00:00Z").expect("insert");
        update_cost_tokens(&conn, id, 42).expect("update");
        let row = find_by_id(&conn, id).expect("find");
        assert_eq!(row.cost_tokens, Some(42));
    }

    #[test]
    fn find_open_returns_only_unclosed_rows() {
        let conn = mem();
        let id_open = insert_run(&conn, "2026-05-17T00:00:00Z").expect("open");
        let id_closed = insert_run(&conn, "2026-05-17T00:00:01Z").expect("ins");
        close_run(&conn, id_closed, "2026-05-17T01:00:00Z", "ok").expect("close");
        let rows = find_open(&conn, 10).expect("find_open");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, id_open);
    }

    #[test]
    fn find_by_outcome_filters_correctly() {
        let conn = mem();
        let a = insert_run(&conn, "2026-05-17T00:00:00Z").expect("ins");
        let b = insert_run(&conn, "2026-05-17T00:00:01Z").expect("ins");
        close_run(&conn, a, "2026-05-17T01:00:00Z", "ok").expect("close");
        close_run(&conn, b, "2026-05-17T02:00:00Z", "fail").expect("close");
        let ok_rows = find_by_outcome(&conn, "ok", 10).expect("ok rows");
        assert_eq!(ok_rows.len(), 1);
        assert_eq!(ok_rows[0].id, a);
        let fail_rows = find_by_outcome(&conn, "fail", 10).expect("fail rows");
        assert_eq!(fail_rows.len(), 1);
        assert_eq!(fail_rows[0].id, b);
    }

    #[test]
    fn find_by_outcome_rejects_invalid_outcome() {
        let conn = mem();
        assert!(matches!(
            find_by_outcome(&conn, "BAD", 10),
            Err(WorkflowError::InvalidOutcome(_))
        ));
    }

    #[test]
    fn find_by_id_yields_row_not_found_for_missing() {
        let conn = mem();
        assert!(matches!(
            find_by_id(&conn, 9999),
            Err(WorkflowError::RowNotFound { id: 9999 })
        ));
    }

    // ---- F9 regression (1) ----------------------------------------------

    #[test]
    fn f9_fitness_dimension_never_set_by_public_api() {
        // Public API has no fitness-writing function; the DDL default is
        // the only path. Re-confirming the invariant for any future
        // refactor that might attempt to add a parameter.
        let conn = mem();
        let id = insert_run(&conn, "2026-05-17T00:00:00Z").expect("insert");
        merge_observation(
            &conn,
            id,
            &ClusterBObservation::ContextCost {
                session_id: 1,
                cost_tokens: 100,
            },
        )
        .expect("merge");
        update_cost_tokens(&conn, id, 200).expect("cost");
        close_run(&conn, id, "2026-05-17T01:00:00Z", "ok").expect("close");
        let row = find_by_id(&conn, id).expect("find");
        assert!(row.fitness_dimension.abs() < f64::EPSILON, "F9 leak");
    }

    // ====================================================================
    // Hardening pass (S1002388) — +10 tests for m7 workflow_runs.
    // F9 zero-weight + concurrency + adversarial input + contract regression.
    // ====================================================================

    use rusqlite::params as rsq_params;

    use super::{open_database, WorkflowRunRow};

    // rationale: Anti-property (F9 zero-weight on m7 nullable columns) —
    // open run must round-trip `ended_at = None` and `outcome = None`
    // after merge + cost update; NEITHER gets fabricated.
    #[test]
    fn f9_open_run_keeps_ended_at_and_outcome_as_none_across_merges() {
        // rationale: Anti-property (F9 zero-weight)
        let conn = mem();
        let id = insert_run(&conn, "2026-05-17T00:00:00Z").expect("insert");
        merge_observation(
            &conn,
            id,
            &ClusterBObservation::BatternStep {
                battern_id: "battern_x".into(),
                step_index: 1,
                duration_ms: 10,
                outcome: "ok".into(),
            },
        )
        .expect("merge");
        update_cost_tokens(&conn, id, 500).expect("cost");
        let row = find_by_id(&conn, id).expect("find");
        assert!(row.ended_at.is_none(), "F9: ended_at must stay None");
        assert!(row.outcome.is_none(), "F9: outcome must stay None");
        assert_eq!(row.cost_tokens, Some(500));
    }

    // rationale: Anti-property (F9 zero-weight) — cost_tokens never
    // collapses None→0 through the public API; explicit 0 stays Some(0).
    #[test]
    fn f9_cost_tokens_none_distinguishes_from_signal_zero() {
        // rationale: Anti-property (F9 zero-weight)
        let conn = mem();
        let id_none = insert_run(&conn, "2026-05-17T00:00:00Z").expect("ins");
        let id_zero = insert_run(&conn, "2026-05-17T00:00:01Z").expect("ins");
        update_cost_tokens(&conn, id_zero, 0).expect("explicit 0");
        let row_none = find_by_id(&conn, id_none).expect("find");
        let row_zero = find_by_id(&conn, id_zero).expect("find");
        assert_eq!(row_none.cost_tokens, None, "no signal");
        assert_eq!(row_zero.cost_tokens, Some(0), "explicit zero signal");
        assert_ne!(row_none.cost_tokens, row_zero.cost_tokens, "F9");
    }

    // rationale: Adversarial input — non-object JSON in consumer_inputs
    // (e.g., a top-level array) MUST surface JsonPatch error, never
    // silently corrupt.
    #[test]
    fn merge_observation_rejects_non_object_consumer_inputs() {
        // rationale: Adversarial input
        let conn = mem();
        let id = insert_run(&conn, "2026-05-17T00:00:00Z").expect("ins");
        conn.execute(
            "UPDATE workflow_runs SET consumer_inputs = ?1 WHERE id = ?2",
            rsq_params!["[1,2,3]", id],
        )
        .expect("manual corrupt");
        let err = merge_observation(
            &conn,
            id,
            &ClusterBObservation::InjectionChain { chain_id: 1 },
        )
        .unwrap_err();
        assert!(matches!(err, WorkflowError::JsonPatch(_)));
    }

    // rationale: Boundary — find_open with limit 0 returns empty Vec
    // (no error).
    #[test]
    fn find_open_with_zero_limit_returns_empty() {
        // rationale: Boundary
        let conn = mem();
        let _id = insert_run(&conn, "2026-05-17T00:00:00Z").expect("ins");
        let rows = find_open(&conn, 0).expect("find_open");
        assert!(rows.is_empty());
    }

    // rationale: Boundary — find_open with usize::MAX limit saturates
    // i64 cast (no overflow).
    #[test]
    fn find_open_with_usize_max_limit_saturates_to_i64_max() {
        // rationale: Boundary
        let conn = mem();
        let id = insert_run(&conn, "2026-05-17T00:00:00Z").expect("ins");
        let rows = find_open(&conn, usize::MAX).expect("find_open");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, id);
    }

    // rationale: Concurrency / resource accounting — open_database is
    // idempotent on the same file path (DDL is CREATE IF NOT EXISTS).
    #[test]
    fn open_database_is_idempotent_on_same_path() {
        // rationale: Concurrency / resource accounting
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("test.db");
        let conn1 = open_database(&path).expect("first open");
        let _id = insert_run(&conn1, "2026-05-17T00:00:00Z").expect("ins");
        drop(conn1);
        let conn2 = open_database(&path).expect("second open");
        let n: i64 = conn2
            .query_row("SELECT COUNT(*) FROM workflow_runs", [], |r| r.get(0))
            .expect("count");
        assert_eq!(n, 1, "DB must persist across open_database calls");
    }

    // rationale: Determinism + Contract regression — serde JSON round-trip
    // identity for WorkflowRunRow preserves all fields including F9
    // zero-weight fitness_dimension.
    #[test]
    fn round_trip_workflow_run_row_serde_via_find_by_id() {
        // rationale: Determinism + Contract regression
        let conn = mem();
        let id = insert_run(&conn, "2026-05-17T00:00:00Z").expect("ins");
        update_cost_tokens(&conn, id, 1234).expect("cost");
        close_run(&conn, id, "2026-05-17T01:00:00Z", "fail").expect("close");
        let a = find_by_id(&conn, id).expect("find a");
        let j = serde_json::to_string(&a).expect("ser");
        let c: WorkflowRunRow = serde_json::from_str(&j).expect("de");
        assert_eq!(a.id, c.id);
        assert_eq!(a.cost_tokens, c.cost_tokens);
        assert_eq!(a.outcome, c.outcome);
        assert!((a.fitness_dimension - c.fitness_dimension).abs() < f64::EPSILON);
    }

    // rationale: Cross-module surface invariant — Outcome wire set is
    // exactly {ok, fail, abort, unknown}, matching SQL CHECK. Cardinality
    // 4 is locked (drift detection for future variant additions).
    #[test]
    fn outcome_wire_set_matches_sql_check_constraint() {
        // rationale: Cross-module surface invariant
        for o in [Outcome::Ok, Outcome::Fail, Outcome::Abort, Outcome::Unknown] {
            assert_eq!(Outcome::parse(o.as_str()).expect("parse"), o);
        }
        let set: std::collections::HashSet<&str> = [
            Outcome::Ok.as_str(),
            Outcome::Fail.as_str(),
            Outcome::Abort.as_str(),
            Outcome::Unknown.as_str(),
        ]
        .into_iter()
        .collect();
        assert_eq!(set.len(), 4);
    }

    // rationale: Resource accounting / contract regression — close_run
    // on an already-closed run succeeds; last write wins (UPDATE).
    #[test]
    fn close_run_is_idempotent_on_already_closed_run() {
        // rationale: Resource accounting / contract regression
        let conn = mem();
        let id = insert_run(&conn, "2026-05-17T00:00:00Z").expect("ins");
        close_run(&conn, id, "2026-05-17T01:00:00Z", "ok").expect("close 1");
        close_run(&conn, id, "2026-05-17T02:00:00Z", "fail").expect("close 2");
        let row = find_by_id(&conn, id).expect("find");
        assert_eq!(row.outcome.as_deref(), Some("fail"));
        assert_eq!(row.ended_at.as_deref(), Some("2026-05-17T02:00:00Z"));
    }

    // rationale: Anti-property / contract regression — close_run with
    // empty `ended_at` persists the empty string (DB has no NOT-NULL-
    // empty check). Regression anchor.
    #[test]
    fn close_run_with_empty_ended_at_persists_empty_string() {
        // rationale: Anti-property / contract regression
        let conn = mem();
        let id = insert_run(&conn, "2026-05-17T00:00:00Z").expect("ins");
        close_run(&conn, id, "", "ok").expect("close");
        let row = find_by_id(&conn, id).expect("find");
        assert_eq!(row.ended_at.as_deref(), Some(""));
        assert_eq!(row.outcome.as_deref(), Some("ok"));
    }
}
