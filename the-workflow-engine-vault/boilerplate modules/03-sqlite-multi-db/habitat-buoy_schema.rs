//! `SQLite` schema for the buoy system.
//!
//! Designed to merge into injection.db as migration v7 (G-06).
//! All tables prefixed with `buoy_` to avoid collisions.

use rusqlite::Connection;

/// Schema version for the buoy tables.
pub const BUOY_SCHEMA_VERSION: u32 = 1;

/// Error type for schema operations.
#[derive(Debug, thiserror::Error)]
pub enum SchemaError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

/// Apply buoy schema to an existing database connection.
///
/// Safe to call multiple times — uses `CREATE TABLE IF NOT EXISTS`.
/// Sets WAL mode and busy timeout for concurrent access (G-03).
///
/// # Errors
///
/// Returns `SchemaError::Sqlite` if table creation fails.
pub fn migrate(conn: &Connection) -> Result<(), SchemaError> {
    conn.execute_batch("PRAGMA journal_mode = WAL;")?;
    conn.execute_batch("PRAGMA busy_timeout = 5000;")?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS buoy_pathway (
            id              TEXT PRIMARY KEY,
            weight          REAL NOT NULL DEFAULT 0.3,
            hit_count       INTEGER NOT NULL DEFAULT 0,
            disk_r          REAL NOT NULL DEFAULT 0.01,
            disk_theta      REAL NOT NULL DEFAULT 0.0,
            tier            TEXT NOT NULL DEFAULT 'floor',
            service         TEXT NOT NULL DEFAULT 'unknown',
            lease_remaining REAL NOT NULL DEFAULT 0.0,
            last_fired      INTEGER,
            kind            TEXT NOT NULL DEFAULT 'associative',
            precedence      INTEGER NOT NULL DEFAULT 0,
            valence         REAL NOT NULL DEFAULT 0.0,
            irreplaceability REAL NOT NULL DEFAULT 0.0,
            created_at      INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            updated_at      INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        );

        CREATE INDEX IF NOT EXISTS idx_buoy_geometry
            ON buoy_pathway(disk_r, disk_theta);

        CREATE INDEX IF NOT EXISTS idx_buoy_tier
            ON buoy_pathway(tier, weight DESC);

        CREATE INDEX IF NOT EXISTS idx_buoy_service
            ON buoy_pathway(service);

        CREATE TABLE IF NOT EXISTS buoy_attractor (
            service         TEXT PRIMARY KEY,
            disk_r          REAL NOT NULL DEFAULT 0.2,
            disk_theta      REAL NOT NULL DEFAULT 0.0,
            pathway_count   INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS buoy_cycle_log (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            session         INTEGER NOT NULL,
            intake_consumed INTEGER NOT NULL DEFAULT 0,
            decayed         INTEGER NOT NULL DEFAULT 0,
            buoyed          INTEGER NOT NULL DEFAULT 0,
            reinforced      INTEGER NOT NULL DEFAULT 0,
            embedded        INTEGER NOT NULL DEFAULT 0,
            pruned          INTEGER NOT NULL DEFAULT 0,
            leases_expired  INTEGER NOT NULL DEFAULT 0,
            cycle_ms        REAL NOT NULL DEFAULT 0.0,
            orac_available  INTEGER NOT NULL DEFAULT 0,
            created_at      INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        );

        CREATE TABLE IF NOT EXISTS buoy_stdp_intake (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            event_type      TEXT NOT NULL,
            pathway_id      TEXT NOT NULL,
            delta           REAL NOT NULL,
            source          TEXT NOT NULL DEFAULT 'orac',
            tick            INTEGER NOT NULL DEFAULT 0,
            consumed        INTEGER NOT NULL DEFAULT 0,
            created_at      INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        );

        CREATE INDEX IF NOT EXISTS idx_stdp_unconsumed
            ON buoy_stdp_intake(consumed)
            WHERE consumed = 0;",
    )?;

    Ok(())
}

/// Open or create a standalone buoy database (for testing or standalone use).
///
/// # Errors
///
/// Returns `SchemaError::Sqlite` if the database cannot be opened.
pub fn open(path: &str) -> Result<Connection, SchemaError> {
    let conn = Connection::open(path)?;
    migrate(&conn)?;
    Ok(conn)
}

/// Create an in-memory buoy database (for testing).
///
/// # Errors
///
/// Returns `SchemaError::Sqlite` if migration fails.
pub fn open_memory() -> Result<Connection, SchemaError> {
    let conn = Connection::open_in_memory()?;
    migrate(&conn)?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrate_creates_tables() {
        let conn = open_memory().expect("should create in-memory db");
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .expect("prepare")
            .query_map([], |row| row.get(0))
            .expect("query")
            .filter_map(|r| r.ok())
            .collect();
        assert!(tables.contains(&"buoy_pathway".to_owned()));
        assert!(tables.contains(&"buoy_attractor".to_owned()));
        assert!(tables.contains(&"buoy_cycle_log".to_owned()));
        assert!(tables.contains(&"buoy_stdp_intake".to_owned()));
    }

    #[test]
    fn migrate_idempotent() {
        let conn = open_memory().expect("should create in-memory db");
        migrate(&conn).expect("second migrate should succeed");
        migrate(&conn).expect("third migrate should succeed");
    }

    #[test]
    fn insert_pathway() {
        let conn = open_memory().expect("db");
        conn.execute(
            "INSERT INTO buoy_pathway (id, weight, hit_count, disk_r, disk_theta, tier, service)
             VALUES ('trap-cp-alias', 0.9, 20, 0.15, 0.0, 'active', 'orac-sidecar')",
            [],
        )
        .expect("insert should succeed");

        let weight: f64 = conn
            .query_row(
                "SELECT weight FROM buoy_pathway WHERE id = 'trap-cp-alias'",
                [],
                |row| row.get(0),
            )
            .expect("query");
        assert!((weight - 0.9).abs() < 1e-10);
    }

    #[test]
    fn insert_attractor() {
        let conn = open_memory().expect("db");
        conn.execute(
            "INSERT INTO buoy_attractor (service, disk_r, disk_theta, pathway_count)
             VALUES ('orac-sidecar', 0.2, 0.0, 42)",
            [],
        )
        .expect("insert");

        let count: u32 = conn
            .query_row(
                "SELECT pathway_count FROM buoy_attractor WHERE service = 'orac-sidecar'",
                [],
                |row| row.get(0),
            )
            .expect("query");
        assert_eq!(count, 42);
    }

    #[test]
    fn insert_stdp_intake() {
        let conn = open_memory().expect("db");
        conn.execute(
            "INSERT INTO buoy_stdp_intake (event_type, pathway_id, delta, source, tick)
             VALUES ('ltp', 'trap-cp-alias', 0.05, 'orac', 1000)",
            [],
        )
        .expect("insert");

        let unconsumed: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM buoy_stdp_intake WHERE consumed = 0",
                [],
                |row| row.get(0),
            )
            .expect("query");
        assert_eq!(unconsumed, 1);
    }

    #[test]
    fn cycle_log_records() {
        let conn = open_memory().expect("db");
        conn.execute(
            "INSERT INTO buoy_cycle_log (session, decayed, buoyed, reinforced, pruned, cycle_ms, orac_available)
             VALUES (205, 100, 30, 5, 2, 8.5, 1)",
            [],
        )
        .expect("insert");

        let ms: f64 = conn
            .query_row(
                "SELECT cycle_ms FROM buoy_cycle_log WHERE session = 205",
                [],
                |row| row.get(0),
            )
            .expect("query");
        assert!((ms - 8.5).abs() < 1e-10);
    }

    #[test]
    fn geometry_index_exists() {
        let conn = open_memory().expect("db");
        let idx_count: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name='idx_buoy_geometry'",
                [],
                |row| row.get(0),
            )
            .expect("query");
        assert_eq!(idx_count, 1);
    }
}
