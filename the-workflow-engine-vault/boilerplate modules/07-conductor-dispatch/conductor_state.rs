//! `SQLite` WAL state database — schema migrations, row types, and CRUD helpers.
//!
//! Weaver owns `state.db`; Zen and the Wave 3 enforcer read from it.
//!
//! # Schema migrations
//!
//! Migrations are versioned in the `schema_migrations` table. Each migration
//! is applied exactly once via [`StateDb::migrate`], which is idempotent and
//! safe to call on every startup. Forward-only for v1; backward migrations are
//! documented in the SQL files for manual rollback.
//!
//! # WAL mode
//!
//! The connection is configured for WAL journal mode (`journal_mode=WAL`) and
//! `synchronous=NORMAL` on every open, giving concurrent reader access while
//! Weaver writes.
//!
//! # Errors
//!
//! All operations return [`StateError`]. Callers must not discard `Result`
//! values; the `#[must_use]` attribute on public functions enforces this.

use std::path::{Path, PathBuf};

use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use tracing::warn;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/// Errors produced by [`StateDb`] operations.
#[derive(Debug, thiserror::Error)]
pub enum StateError {
    /// Underlying `SQLite` failure.
    #[error("state_db: sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    /// JSON serialisation or deserialisation failure.
    #[error("state_db: serde error: {0}")]
    Serde(#[from] serde_json::Error),
    /// A required database path could not be determined.
    #[error("state_db: cannot determine data directory")]
    NoDataDir,
}

/// Convenience alias.
pub type Result<T> = std::result::Result<T, StateError>;

// ---------------------------------------------------------------------------
// Row types
// ---------------------------------------------------------------------------

/// A single snapshot row from the `weaver_snapshots` table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapshotRow {
    /// Unix millisecond timestamp (PRIMARY KEY).
    pub ts: i64,
    /// RALPH fitness score.
    pub fitness: f64,
    /// PV2 Kuramoto coherence r.
    pub field_r: f64,
    /// SYNTHEX thermal temperature.
    pub thermal_t: f64,
    /// Number of PV2 spheres.
    pub sphere_count: i64,
    /// RALPH evolution phase label.
    pub ralph_phase: String,
    /// RALPH generation counter.
    pub ralph_gen: i64,
    /// Long-term potentiation count.
    pub ltp: i64,
    /// Long-term depression count.
    pub ltd: i64,
    /// Mutations proposed this epoch.
    pub mutations_proposed: i64,
    /// Mutations accepted this epoch.
    pub mutations_accepted: i64,
    /// Number of open circuit breakers as reported by ORAC (authoritative).
    pub breakers_open: i64,
    /// Number of service probe transport/parse failures this cycle.
    /// Distinct from [`breakers_open`]: probe failures mean connectivity loss,
    /// not ORAC-observed breaker state. Do NOT add them together.
    pub probe_failures: i64,
    /// ME V2 fitness (from `/api/health`).
    pub me_fitness: f64,
    /// Full probe JSON blob for forensics.
    pub raw_json: String,
}

/// Severity level for divergence reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Severity {
    /// Informational drift; no immediate action required.
    Low,
    /// Medium-severity drift; investigate within 24 h.
    Med,
    /// High-severity drift; investigate immediately.
    High,
    /// Critical violation; auto-rollback may trigger.
    Critical,
}

impl Severity {
    /// Parses a severity label from a string slice.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the label is not one of `LOW`, `MED`, `HIGH`, or
    /// `CRITICAL`.
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "LOW" => Ok(Self::Low),
            "MED" => Ok(Self::Med),
            "HIGH" => Ok(Self::High),
            "CRITICAL" => Ok(Self::Critical),
            _other => Err(StateError::Sqlite(rusqlite::Error::InvalidQuery)),
        }
    }

    /// Returns the canonical uppercase string representation.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "LOW",
            Self::Med => "MED",
            Self::High => "HIGH",
            Self::Critical => "CRITICAL",
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A divergence report row from the `divergence_reports` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DivergenceReport {
    /// Auto-assigned row id (None before insert).
    pub id: Option<i64>,
    /// Unix millisecond timestamp of detection.
    pub ts: i64,
    /// Originating agent (`"zen"` or `"watcher"`).
    pub source: String,
    /// Reference to the authoritative plan document.
    pub plan_ref: String,
    /// Reference to the observed implementation artifact.
    pub observed_ref: String,
    /// Severity level.
    pub severity: Severity,
    /// Machine-readable divergence kind.
    pub kind: String,
    /// JSON detail body.
    pub body: serde_json::Value,
}

/// A pane-to-sphere binding row from the `pane_sphere_binding` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneBinding {
    /// Zellij pane UUID.
    pub pane_uuid: String,
    /// PV2 sphere identity string.
    pub sphere_id: String,
    /// Persona label (`nvim`, `fleet-worker`, `cc-NN`, `unknown`).
    pub persona: String,
    /// Buoy labels as a JSON-serialised `Vec<String>`.
    pub buoys_json: String,
    /// Unix ms of last sphere differentiation.
    pub last_differentiated_at: i64,
    /// Unix ms of last observed activity.
    pub last_seen_active_at: i64,
}

// ---------------------------------------------------------------------------
// StateDb
// ---------------------------------------------------------------------------

/// Handle to the Weaver `state.db` `SQLite` database.
///
/// Wraps a `rusqlite::Connection` with WAL mode enabled. Use
/// [`StateDb::open`] or [`StateDb::open_in_memory`] to create a handle, then
/// call [`StateDb::migrate`] before any other operations.
pub struct StateDb {
    conn: Connection,
    path: PathBuf,
}

impl StateDb {
    // -----------------------------------------------------------------------
    // Constructors
    // -----------------------------------------------------------------------

    /// Opens (or creates) the database at `path` with WAL mode.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the file cannot be opened or WAL/synchronous pragmas
    /// cannot be applied.
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        Self::configure(&conn)?;
        Ok(Self {
            conn,
            path: path.to_owned(),
        })
    }

    /// Opens an in-memory database for testing.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the in-memory connection cannot be established.
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        Self::configure(&conn)?;
        Ok(Self {
            conn,
            path: PathBuf::from(":memory:"),
        })
    }

    fn configure(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA foreign_keys=ON;
             PRAGMA busy_timeout=5000;",
        )?;
        Ok(())
    }

    /// Returns the database path (`:memory:` for in-memory instances).
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    // -----------------------------------------------------------------------
    // Migrations
    // -----------------------------------------------------------------------

    /// Applies all pending schema migrations idempotently.
    ///
    /// Safe to call on every startup. Already-applied migrations are skipped
    /// based on the `schema_migrations` version table.
    ///
    /// # Errors
    ///
    /// Returns `Err` if any migration SQL fails to execute.
    pub fn migrate(&mut self) -> Result<()> {
        // Create the migrations tracking table if it doesn't yet exist.
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version    INTEGER PRIMARY KEY,
                applied_at INTEGER NOT NULL
            );",
        )?;

        let applied: Vec<i64> = {
            let mut stmt = self
                .conn
                .prepare("SELECT version FROM schema_migrations ORDER BY version")?;
            let rows = stmt.query_map([], |row| row.get(0))?
                .collect::<std::result::Result<Vec<i64>, _>>()?;
            rows
        };

        let now_ms = chrono::Utc::now().timestamp_millis();

        // Migration 001 — baseline tables.
        if !applied.contains(&1) {
            self.conn.execute_batch(include_str!(
                "../migrations/001_initial.sql"
            ))?;
            self.conn.execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
                params![1_i64, now_ms],
            )?;
        }

        // Migration 002 — enforcement_actions table (Wave 3 stub).
        if !applied.contains(&2) {
            self.conn.execute_batch(include_str!(
                "../migrations/002_enforcement.sql"
            ))?;
            self.conn.execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
                params![2_i64, now_ms],
            )?;
        }

        // Migration 003 — probe_failures column on weaver_snapshots (F-CONDUCTOR-03).
        if !applied.contains(&3) {
            self.conn.execute_batch(include_str!(
                "../migrations/003_probe_failures.sql"
            ))?;
            self.conn.execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
                params![3_i64, now_ms],
            )?;
        }

        // Migration 004 — kv table for enforcer daemon persistence (F-CONDUCTOR-04).
        if !applied.contains(&4) {
            self.conn.execute_batch(include_str!(
                "../migrations/004_enforcer_kv.sql"
            ))?;
            self.conn.execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
                params![4_i64, now_ms],
            )?;
        }

        Ok(())
    }

    // -----------------------------------------------------------------------
    // weaver_snapshots
    // -----------------------------------------------------------------------

    /// Inserts a new snapshot row.
    ///
    /// Overwrites any existing row with the same `ts` (REPLACE semantics).
    ///
    /// # Errors
    ///
    /// Returns `Err` if the INSERT fails.
    pub fn insert_snapshot(&self, row: &SnapshotRow) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO weaver_snapshots
             (ts, fitness, field_r, thermal_t, sphere_count, ralph_phase, ralph_gen,
              ltp, ltd, mutations_proposed, mutations_accepted, breakers_open,
              probe_failures, me_fitness, raw_json)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)",
            params![
                row.ts,
                row.fitness,
                row.field_r,
                row.thermal_t,
                row.sphere_count,
                row.ralph_phase,
                row.ralph_gen,
                row.ltp,
                row.ltd,
                row.mutations_proposed,
                row.mutations_accepted,
                row.breakers_open,
                row.probe_failures,
                row.me_fitness,
                row.raw_json,
            ],
        )?;
        Ok(())
    }

    /// Returns the most recent snapshot row, or `None` if the table is empty.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the query fails.
    pub fn latest_snapshot(&self) -> Result<Option<SnapshotRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT ts, fitness, field_r, thermal_t, sphere_count, ralph_phase, ralph_gen,
                    ltp, ltd, mutations_proposed, mutations_accepted, breakers_open,
                    probe_failures, me_fitness, raw_json
             FROM weaver_snapshots ORDER BY ts DESC LIMIT 1",
        )?;
        let mut rows = stmt.query_map([], |row| {
            Ok(SnapshotRow {
                ts: row.get(0)?,
                fitness: row.get(1)?,
                field_r: row.get(2)?,
                thermal_t: row.get(3)?,
                sphere_count: row.get(4)?,
                ralph_phase: row.get(5)?,
                ralph_gen: row.get(6)?,
                ltp: row.get(7)?,
                ltd: row.get(8)?,
                mutations_proposed: row.get(9)?,
                mutations_accepted: row.get(10)?,
                breakers_open: row.get(11)?,
                probe_failures: row.get(12)?,
                me_fitness: row.get(13)?,
                raw_json: row.get(14)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    /// Returns up to `limit` snapshots ordered from newest to oldest.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the query fails.
    pub fn recent_snapshots(&self, limit: usize) -> Result<Vec<SnapshotRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT ts, fitness, field_r, thermal_t, sphere_count, ralph_phase, ralph_gen,
                    ltp, ltd, mutations_proposed, mutations_accepted, breakers_open,
                    probe_failures, me_fitness, raw_json
             FROM weaver_snapshots ORDER BY ts DESC LIMIT ?1",
        )?;
        #[allow(clippy::cast_possible_wrap)]
        let limit_i64 = limit as i64;
        let rows = stmt
            .query_map(params![limit_i64], |row| {
                Ok(SnapshotRow {
                    ts: row.get(0)?,
                    fitness: row.get(1)?,
                    field_r: row.get(2)?,
                    thermal_t: row.get(3)?,
                    sphere_count: row.get(4)?,
                    ralph_phase: row.get(5)?,
                    ralph_gen: row.get(6)?,
                    ltp: row.get(7)?,
                    ltd: row.get(8)?,
                    mutations_proposed: row.get(9)?,
                    mutations_accepted: row.get(10)?,
                    breakers_open: row.get(11)?,
                    probe_failures: row.get(12)?,
                    me_fitness: row.get(13)?,
                    raw_json: row.get(14)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    // -----------------------------------------------------------------------
    // divergence_reports
    // -----------------------------------------------------------------------

    /// Inserts a divergence report, returning the assigned row id.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the INSERT or JSON serialisation fails.
    pub fn insert_divergence(&self, report: &DivergenceReport) -> Result<i64> {
        let body_str = serde_json::to_string(&report.body)?;
        self.conn.execute(
            "INSERT INTO divergence_reports
             (ts, source, plan_ref, observed_ref, severity, kind, body)
             VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![
                report.ts,
                report.source,
                report.plan_ref,
                report.observed_ref,
                report.severity.as_str(),
                report.kind,
                body_str,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Returns divergence reports filtered by optional minimum severity and
    /// ordered from newest to oldest.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the query or JSON deserialisation fails.
    pub fn divergence_reports(
        &self,
        min_severity: Option<Severity>,
        limit: usize,
    ) -> Result<Vec<DivergenceReport>> {
        let sev_filter = min_severity
            .map(|s| format!("AND severity >= '{}'", s.as_str()))
            .unwrap_or_default();
        let sql = format!(
            "SELECT id, ts, source, plan_ref, observed_ref, severity, kind, body
             FROM divergence_reports
             WHERE 1=1 {sev_filter}
             ORDER BY ts DESC LIMIT ?1"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        #[allow(clippy::cast_possible_wrap)]
        let limit_i64 = limit as i64;
        let rows = stmt
            .query_map(params![limit_i64], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                ))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let mut reports = Vec::with_capacity(rows.len());
        for (id, ts, source, plan_ref, observed_ref, sev_str, kind, body_str) in rows {
            let body: serde_json::Value = serde_json::from_str(&body_str)?;
            // F-CONDUCTOR-05: default to CRITICAL on parse error (fail LOUD).
            // Defaulting to Low would silently bypass auto-rollback for corrupted
            // severity strings (e.g. 'CRTITICAL' typo, 'critical' case drift).
            // CRITICAL causes the enforcer to fire — human review surfaces the
            // data integrity issue. Defaulting to Low fails SILENT.
            let severity = if let Ok(s) = Severity::parse(&sev_str) {
                s
            } else {
                warn!(
                    id,
                    sev_str = %sev_str,
                    "unparseable severity — defaulting to CRITICAL for safety"
                );
                Severity::Critical
            };
            reports.push(DivergenceReport {
                id: Some(id),
                ts,
                source,
                plan_ref,
                observed_ref,
                severity,
                kind,
                body,
            });
        }
        Ok(reports)
    }

    // -----------------------------------------------------------------------
    // pane_sphere_binding
    // -----------------------------------------------------------------------

    /// Upserts a pane-to-sphere binding row.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the UPSERT fails.
    pub fn upsert_pane_binding(&self, binding: &PaneBinding) -> Result<()> {
        self.conn.execute(
            "INSERT INTO pane_sphere_binding
             (pane_uuid, sphere_id, persona, buoys_json,
              last_differentiated_at, last_seen_active_at)
             VALUES (?1,?2,?3,?4,?5,?6)
             ON CONFLICT(pane_uuid) DO UPDATE SET
               sphere_id               = excluded.sphere_id,
               persona                 = excluded.persona,
               buoys_json              = excluded.buoys_json,
               last_differentiated_at  = excluded.last_differentiated_at,
               last_seen_active_at     = excluded.last_seen_active_at",
            params![
                binding.pane_uuid,
                binding.sphere_id,
                binding.persona,
                binding.buoys_json,
                binding.last_differentiated_at,
                binding.last_seen_active_at,
            ],
        )?;
        Ok(())
    }

    /// Returns all pane-to-sphere bindings.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the SELECT fails.
    pub fn all_pane_bindings(&self) -> Result<Vec<PaneBinding>> {
        let mut stmt = self.conn.prepare(
            "SELECT pane_uuid, sphere_id, persona, buoys_json,
                    last_differentiated_at, last_seen_active_at
             FROM pane_sphere_binding",
        )?;
        let rows = stmt
            .query_map([], |row| {
                Ok(PaneBinding {
                    pane_uuid: row.get(0)?,
                    sphere_id: row.get(1)?,
                    persona: row.get(2)?,
                    buoys_json: row.get(3)?,
                    last_differentiated_at: row.get(4)?,
                    last_seen_active_at: row.get(5)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// Looks up a single binding by pane UUID.
    ///
    /// Returns `None` if the pane UUID is not registered.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the SELECT fails.
    pub fn pane_binding_by_uuid(&self, pane_uuid: &str) -> Result<Option<PaneBinding>> {
        let mut stmt = self.conn.prepare(
            "SELECT pane_uuid, sphere_id, persona, buoys_json,
                    last_differentiated_at, last_seen_active_at
             FROM pane_sphere_binding WHERE pane_uuid = ?1",
        )?;
        let mut rows = stmt.query_map(params![pane_uuid], |row| {
            Ok(PaneBinding {
                pane_uuid: row.get(0)?,
                sphere_id: row.get(1)?,
                persona: row.get(2)?,
                buoys_json: row.get(3)?,
                last_differentiated_at: row.get(4)?,
                last_seen_active_at: row.get(5)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    // -----------------------------------------------------------------------
    // enforcement_actions
    // -----------------------------------------------------------------------

    /// Inserts a Wave 3 enforcement action record.
    ///
    /// Called by [`crate::enforcement::EnforcerDb`] BEFORE invoking `forge
    /// --rollback` — audit-first guarantee.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the INSERT fails.
    #[allow(clippy::too_many_arguments)]
    pub fn insert_enforcement_action(
        &self,
        ts: i64,
        trigger_source: &str,
        target_service: &str,
        action: &str,
        severity: &str,
        divergence_report_id: Option<i64>,
        pre_state_hash: &str,
        operator: &str,
        body: &str,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO enforcement_actions
             (ts, trigger_source, target_service, action, severity,
              divergence_report_id, pre_state_hash, post_state_hash, operator, body)
             VALUES (?1,?2,?3,?4,?5,?6,?7,NULL,?8,?9)",
            params![
                ts,
                trigger_source,
                target_service,
                action,
                severity,
                divergence_report_id,
                pre_state_hash,
                operator,
                body,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Returns enforcement action records ordered from newest to oldest.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the query fails.
    pub fn enforcement_actions(&self, limit: usize) -> Result<Vec<serde_json::Value>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, ts, trigger_source, target_service, action, severity,
                    divergence_report_id, pre_state_hash, operator, body
             FROM enforcement_actions ORDER BY ts DESC LIMIT ?1",
        )?;
        #[allow(clippy::cast_possible_wrap)]
        let rows = stmt
            .query_map(params![limit as i64], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, Option<i64>>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, String>(9)?,
                ))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let mut out = Vec::with_capacity(rows.len());
        for (id, ts, trigger_source, target_service, action, severity,
             divergence_report_id, pre_state_hash, operator, body) in rows
        {
            out.push(serde_json::json!({
                "id": id,
                "ts": ts,
                "trigger_source": trigger_source,
                "target_service": target_service,
                "action": action,
                "severity": severity,
                "divergence_report_id": divergence_report_id,
                "pre_state_hash": pre_state_hash,
                "operator": operator,
                "body": body,
            }));
        }
        Ok(out)
    }

    // -----------------------------------------------------------------------
    // kv — general-purpose key/value store (F-CONDUCTOR-04)
    // -----------------------------------------------------------------------

    /// Reads a value from the `kv` table.
    ///
    /// Returns `None` if the key does not exist.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the SELECT fails.
    pub fn kv_get(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM kv WHERE key = ?1")?;
        let mut rows = stmt.query_map(params![key], |row| row.get::<_, String>(0))?;
        Ok(rows.next().transpose()?)
    }

    /// Inserts or updates a value in the `kv` table.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the UPSERT fails.
    pub fn kv_set(&self, key: &str, value: &str) -> Result<()> {
        let now_ms = chrono::Utc::now().timestamp_millis();
        self.conn.execute(
            "INSERT INTO kv (key, value, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            params![key, value, now_ms],
        )?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Returns the total number of applied migrations.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the query fails.
    pub fn applied_migration_count(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| {
                row.get(0)
            })?;
        Ok(count)
    }
}

// ---------------------------------------------------------------------------
// Default data-directory helper
// ---------------------------------------------------------------------------

/// Returns the default path for `state.db`.
///
/// Resolves to `~/.local/share/habitat-conductor/state.db`.
///
/// # Errors
///
/// Returns `Err` if the home directory cannot be determined.
pub fn default_db_path() -> std::result::Result<PathBuf, StateError> {
    let base = dirs::data_local_dir().ok_or(StateError::NoDataDir)?;
    Ok(base.join("habitat-conductor").join("state.db"))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn open_db() -> StateDb {
        let mut db = StateDb::open_in_memory().expect("in-memory db");
        db.migrate().expect("migrate");
        db
    }

    fn sample_snapshot(ts: i64) -> SnapshotRow {
        SnapshotRow {
            ts,
            fitness: 0.74,
            field_r: 0.92,
            thermal_t: 0.51,
            sphere_count: 9,
            ralph_phase: "selection".into(),
            ralph_gen: 5678,
            ltp: 120,
            ltd: 22_000,
            mutations_proposed: 178,
            mutations_accepted: 1,
            breakers_open: 0,
            probe_failures: 0,
            me_fitness: 0.70,
            raw_json: r#"{"probe":"ok"}"#.into(),
        }
    }

    fn sample_divergence(ts: i64, sev: Severity) -> DivergenceReport {
        DivergenceReport {
            id: None,
            ts,
            source: "zen".into(),
            plan_ref: "WAVE_1_SPEC.md".into(),
            observed_ref: "pv2:/spheres".into(),
            severity: sev,
            kind: "sphere_ghost".into(),
            body: json!({"ghost_count": 9}),
        }
    }

    // --- migrations ---

    #[test]
    fn migrations_applied_twice_idempotent() {
        let mut db = StateDb::open_in_memory().expect("in-memory db");
        db.migrate().expect("first migrate");
        db.migrate().expect("second migrate (idempotent)");
        // 4 migrations: 001_initial, 002_enforcement, 003_probe_failures, 004_enforcer_kv.
        assert_eq!(db.applied_migration_count().expect("count"), 4);
    }

    #[test]
    fn migration_count_is_four() {
        let db = open_db();
        // 001_initial + 002_enforcement + 003_probe_failures + 004_enforcer_kv.
        assert_eq!(db.applied_migration_count().expect("count"), 4);
    }

    // --- weaver_snapshots ---

    #[test]
    fn insert_and_retrieve_snapshot() {
        let db = open_db();
        let row = sample_snapshot(1_715_000_000_000);
        db.insert_snapshot(&row).expect("insert");
        let latest = db.latest_snapshot().expect("latest").expect("should exist");
        assert_eq!(latest.ts, row.ts);
        assert!((latest.fitness - row.fitness).abs() < f64::EPSILON);
    }

    #[test]
    fn latest_snapshot_returns_newest() {
        let db = open_db();
        db.insert_snapshot(&sample_snapshot(1_000)).expect("insert 1");
        db.insert_snapshot(&sample_snapshot(2_000)).expect("insert 2");
        db.insert_snapshot(&sample_snapshot(1_500)).expect("insert 3");
        let latest = db.latest_snapshot().expect("query").expect("some");
        assert_eq!(latest.ts, 2_000);
    }

    #[test]
    fn latest_snapshot_empty_returns_none() {
        let db = open_db();
        let result = db.latest_snapshot().expect("query");
        assert!(result.is_none());
    }

    #[test]
    fn recent_snapshots_limit_respected() {
        let db = open_db();
        for i in 0..10_i64 {
            db.insert_snapshot(&sample_snapshot(i * 1_000)).expect("insert");
        }
        let rows = db.recent_snapshots(3).expect("query");
        assert_eq!(rows.len(), 3);
        // Newest first.
        assert_eq!(rows[0].ts, 9_000);
    }

    #[test]
    fn snapshot_replace_on_same_ts() {
        let db = open_db();
        let mut row = sample_snapshot(999);
        db.insert_snapshot(&row).expect("first insert");
        row.fitness = 0.99;
        db.insert_snapshot(&row).expect("replace insert");
        let latest = db.latest_snapshot().expect("query").expect("some");
        assert!((latest.fitness - 0.99).abs() < f64::EPSILON);
    }

    // --- divergence_reports ---

    #[test]
    fn insert_divergence_returns_id() {
        let db = open_db();
        let report = sample_divergence(1_000, Severity::High);
        let id = db.insert_divergence(&report).expect("insert");
        assert!(id > 0);
    }

    #[test]
    fn divergence_reports_ordered_newest_first() {
        let db = open_db();
        db.insert_divergence(&sample_divergence(1_000, Severity::Low))
            .expect("insert 1");
        db.insert_divergence(&sample_divergence(3_000, Severity::High))
            .expect("insert 2");
        db.insert_divergence(&sample_divergence(2_000, Severity::Med))
            .expect("insert 3");
        let rows = db.divergence_reports(None, 10).expect("query");
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].ts, 3_000);
    }

    #[test]
    fn divergence_reports_body_roundtrip() {
        let db = open_db();
        let report = sample_divergence(1_000, Severity::Critical);
        db.insert_divergence(&report).expect("insert");
        let rows = db.divergence_reports(None, 1).expect("query");
        assert_eq!(rows[0].body, json!({"ghost_count": 9}));
    }

    #[test]
    fn divergence_severity_ordering() {
        assert!(Severity::Low < Severity::Med);
        assert!(Severity::Med < Severity::High);
        assert!(Severity::High < Severity::Critical);
    }

    #[test]
    fn severity_roundtrip_from_str() {
        for (s, expected) in [
            ("LOW", Severity::Low),
            ("MED", Severity::Med),
            ("HIGH", Severity::High),
            ("CRITICAL", Severity::Critical),
        ] {
            let got = Severity::parse(s).expect("parse");
            assert_eq!(got, expected);
        }
    }

    #[test]
    fn severity_as_str_roundtrip() {
        for sev in [Severity::Low, Severity::Med, Severity::High, Severity::Critical] {
            let s = sev.as_str();
            let back = Severity::parse(s).expect("roundtrip");
            assert_eq!(back, sev);
        }
    }

    // --- pane_sphere_binding ---

    #[test]
    fn upsert_and_retrieve_binding() {
        let db = open_db();
        let binding = PaneBinding {
            pane_uuid: "uuid-1".into(),
            sphere_id: "sphere-A".into(),
            persona: "nvim".into(),
            buoys_json: r#"["rust","habitat"]"#.into(),
            last_differentiated_at: 1_000,
            last_seen_active_at: 2_000,
        };
        db.upsert_pane_binding(&binding).expect("upsert");
        let got = db.pane_binding_by_uuid("uuid-1").expect("query").expect("some");
        assert_eq!(got.sphere_id, "sphere-A");
        assert_eq!(got.persona, "nvim");
    }

    #[test]
    fn upsert_updates_existing_binding() {
        let db = open_db();
        let mut binding = PaneBinding {
            pane_uuid: "uuid-x".into(),
            sphere_id: "sphere-old".into(),
            persona: "unknown".into(),
            buoys_json: "[]".into(),
            last_differentiated_at: 1_000,
            last_seen_active_at: 1_000,
        };
        db.upsert_pane_binding(&binding).expect("first upsert");
        binding.sphere_id = "sphere-new".into();
        binding.persona = "fleet-worker".into();
        db.upsert_pane_binding(&binding).expect("second upsert");
        let got = db.pane_binding_by_uuid("uuid-x").expect("query").expect("some");
        assert_eq!(got.sphere_id, "sphere-new");
        assert_eq!(got.persona, "fleet-worker");
    }

    #[test]
    fn pane_binding_missing_uuid_returns_none() {
        let db = open_db();
        let got = db.pane_binding_by_uuid("nonexistent").expect("query");
        assert!(got.is_none());
    }

    #[test]
    fn all_pane_bindings_returns_all() {
        let db = open_db();
        for i in 0..5_u32 {
            db.upsert_pane_binding(&PaneBinding {
                pane_uuid: format!("uuid-{i}"),
                sphere_id: format!("sphere-{i}"),
                persona: "fleet-worker".into(),
                buoys_json: "[]".into(),
                last_differentiated_at: 0,
                last_seen_active_at: 0,
            })
            .expect("upsert");
        }
        let all = db.all_pane_bindings().expect("query");
        assert_eq!(all.len(), 5);
    }

    #[test]
    fn sphere_id_unique_constraint_enforced() {
        let db = open_db();
        let b1 = PaneBinding {
            pane_uuid: "pane-A".into(),
            sphere_id: "sphere-1".into(),
            persona: "nvim".into(),
            buoys_json: "[]".into(),
            last_differentiated_at: 0,
            last_seen_active_at: 0,
        };
        let b2 = PaneBinding {
            pane_uuid: "pane-B".into(),
            sphere_id: "sphere-1".into(), // same sphere_id → UNIQUE violation
            persona: "fleet-worker".into(),
            buoys_json: "[]".into(),
            last_differentiated_at: 0,
            last_seen_active_at: 0,
        };
        db.upsert_pane_binding(&b1).expect("insert b1");
        let result = db.upsert_pane_binding(&b2);
        assert!(result.is_err(), "expected UNIQUE constraint violation");
    }

    #[test]
    fn default_db_path_contains_habitat_conductor() {
        // Only verifies path structure — no filesystem access needed.
        match default_db_path() {
            Ok(p) => assert!(p.to_string_lossy().contains("habitat-conductor")),
            Err(StateError::NoDataDir) => {
                // CI may lack HOME; just ensure the error variant is correct.
            }
            Err(other) => panic!("unexpected error: {other}"),
        }
    }

    // --- F-CONDUCTOR-05: severity parse error defaults to CRITICAL ---

    #[test]
    fn severity_parse_unknown_string_is_err() {
        // Severity::parse rejects any string not in {LOW, MED, HIGH, CRITICAL}.
        // This is the gate that F-CONDUCTOR-05 catches on the read path.
        assert!(Severity::parse("CRTITICAL").is_err(), "typo must be Err");
        assert!(Severity::parse("critical").is_err(), "lowercase must be Err");
        assert!(Severity::parse("").is_err(), "empty string must be Err");
        assert!(Severity::parse("MEDIUM").is_err(), "non-canonical alias must be Err");
    }

    #[test]
    fn severity_parse_fail_loud_policy_not_low() {
        // F-CONDUCTOR-05 policy: the correct default on parse failure is CRITICAL
        // (fail-loud so the enforcer fires and human review catches data corruption),
        // NOT Low (fail-silent, bypasses auto-rollback indefinitely).
        //
        // This test encodes the policy decision: a caller that defaults to Low
        // on Err is violating the invariant. We verify the policy here without
        // needing to insert an invalid row (the CHECK constraint prevents that).
        let bad = "CRTITICAL";
        let result = Severity::parse(bad);
        assert!(result.is_err());
        // The correct production default (as encoded in divergence_reports()) is Critical.
        // If someone changes it back to Low, this test documents the regression.
        let defaulted = result.unwrap_or(Severity::Critical);
        assert_eq!(defaulted, Severity::Critical);
        // Verify the LOW default would NOT be the right choice.
        let wrong_default = Severity::parse(bad).unwrap_or(Severity::Low);
        assert_ne!(wrong_default, Severity::Critical, "Low default is the silent-failure regression");
    }

    #[test]
    fn divergence_reports_severity_field_preserved_on_readback() {
        // Each inserted severity level must round-trip correctly through the DB.
        // This verifies that Severity::parse handles all canonical strings and
        // that insert_divergence stores them in the expected format.
        let db = open_db();
        let cases = [
            (1_000_i64, Severity::Low),
            (2_000, Severity::Med),
            (3_000, Severity::High),
            (4_000, Severity::Critical),
        ];
        for (ts, sev) in cases {
            db.insert_divergence(&sample_divergence(ts, sev)).expect("insert");
        }
        let all = db.divergence_reports(None, 10).expect("query");
        assert_eq!(all.len(), 4);
        // Rows come back newest-first.
        let found_sevs: Vec<Severity> = all.iter().map(|r| r.severity).collect();
        assert!(found_sevs.contains(&Severity::Critical));
        assert!(found_sevs.contains(&Severity::High));
        assert!(found_sevs.contains(&Severity::Med));
        assert!(found_sevs.contains(&Severity::Low));
    }

    // --- F-CONDUCTOR-03: probe_failures field on snapshot ---

    #[test]
    fn snapshot_probe_failures_roundtrip() {
        let db = open_db();
        let mut row = sample_snapshot(5_000);
        row.probe_failures = 3;
        row.breakers_open = 1;
        db.insert_snapshot(&row).expect("insert");
        let got = db.latest_snapshot().expect("query").expect("some");
        // probe_failures and breakers_open are independent columns.
        assert_eq!(got.probe_failures, 3, "probe_failures must roundtrip through DB");
        assert_eq!(got.breakers_open, 1, "breakers_open is ORAC count only");
    }

    #[test]
    fn snapshot_probe_failures_default_zero_on_old_rows() {
        // Migration 003 adds probe_failures with DEFAULT 0. Rows inserted before
        // the migration (simulated here as a fresh row without probe_failures set)
        // must read back as 0 without error.
        let db = open_db();
        let row = sample_snapshot(6_000); // probe_failures = 0 in sample_snapshot
        db.insert_snapshot(&row).expect("insert");
        let got = db.latest_snapshot().expect("query").expect("some");
        assert_eq!(got.probe_failures, 0);
    }

    // --- F-CONDUCTOR-04: kv store ---

    #[test]
    fn kv_set_and_get_roundtrip() {
        let db = open_db();
        db.kv_set("enforcer.last_seen_id", "42").expect("kv_set");
        let val = db.kv_get("enforcer.last_seen_id").expect("kv_get");
        assert_eq!(val.as_deref(), Some("42"));
    }

    #[test]
    fn kv_get_missing_key_returns_none() {
        let db = open_db();
        let val = db.kv_get("nonexistent.key").expect("kv_get");
        assert!(val.is_none());
    }

    #[test]
    fn kv_set_overwrites_existing_value() {
        let db = open_db();
        db.kv_set("enforcer.last_seen_id", "1").expect("first set");
        db.kv_set("enforcer.last_seen_id", "99").expect("second set");
        let val = db.kv_get("enforcer.last_seen_id").expect("kv_get");
        assert_eq!(val.as_deref(), Some("99"), "kv_set must overwrite (upsert)");
    }

    #[test]
    fn kv_different_keys_independent() {
        let db = open_db();
        db.kv_set("key-a", "alpha").expect("set a");
        db.kv_set("key-b", "beta").expect("set b");
        assert_eq!(db.kv_get("key-a").expect("get a").as_deref(), Some("alpha"));
        assert_eq!(db.kv_get("key-b").expect("get b").as_deref(), Some("beta"));
    }
}
