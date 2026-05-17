//! 7-step Hebbian buoy consolidation engine.
//!
//! Cycle: INTAKE → DECAY → BUOY → REINFORCE → EMBED → PRUNE → LEASE
//!
//! All steps execute within a single `SQLite` transaction.
//! The engine is a pure function: `consolidate(conn, config, session, fired) -> Result`.

use rusqlite::{params, Connection, Transaction};

use crate::disk;
use crate::types::{BuoyConfig, ConsolidationResult};

/// Error type for engine operations.
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Invalid state: {0}")]
    InvalidState(String),
}

/// Run the full 7-step consolidation cycle within a transaction.
///
/// # Arguments
///
/// * `conn` — The database connection (buoy tables must exist).
/// * `config` — Buoy engine parameters.
/// * `current_session` — Current session number for lease accounting.
/// * `fired_patterns` — Pattern IDs that were naturally fired this session.
/// * `tool_call_count` — Number of tool calls in this session (for weighted lease units, NA-10).
/// * `orac_available` — Whether ORAC was reachable for STDP intake.
///
/// # Errors
///
/// Returns `EngineError` if any step fails. The transaction is NOT committed
/// by this function — the caller must commit (allows embedding in a larger transaction).
pub fn consolidate(
    tx: &Transaction<'_>,
    config: &BuoyConfig,
    current_session: u64,
    fired_patterns: &[String],
    tool_call_count: u64,
    orac_available: bool,
) -> Result<ConsolidationResult, EngineError> {
    let start = std::time::Instant::now();
    let mut result = ConsolidationResult {
        orac_available,
        ..ConsolidationResult::default()
    };

    // Step 1: INTAKE — consume unconsumed STDP events (G-08: skip if ORAC unavailable)
    if orac_available {
        result.intake_consumed = step_intake(tx, config)?;
    }

    // Step 2: DECAY — weight *= decay_rate for all pathways
    result.decayed = step_decay(tx, config)?;

    // Step 3: BUOY — weight += buoy_rate for graduated pathways with active lease
    result.buoyed = step_buoy(tx, config)?;

    // Step 4: REINFORCE — weight += reinforce_rate for fired patterns
    result.reinforced = step_reinforce(tx, config, current_session, fired_patterns)?;

    // Step 5: EMBED — recompute disk_r, disk_theta from hit_count
    result.embedded = step_embed(tx)?;

    // Step 6: PRUNE — delete pathways below prune_threshold (with NA-02 guard)
    result.pruned = step_prune(tx, config)?;

    // Step 7: LEASE — decrement lease, expire if zero (NA-10: weighted units)
    let lease_units = compute_lease_units(tool_call_count, config);
    result.leases_expired = step_lease(tx, config, current_session, lease_units, orac_available)?;

    // Update tier classification based on new weights
    update_tiers(tx)?;

    result.cycle_ms = start.elapsed().as_secs_f64() * 1000.0;

    // Log the cycle result
    tx.execute(
        "INSERT INTO buoy_cycle_log (session, intake_consumed, decayed, buoyed, reinforced, embedded, pruned, leases_expired, cycle_ms, orac_available)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            current_session,
            result.intake_consumed,
            result.decayed,
            result.buoyed,
            result.reinforced,
            result.embedded,
            result.pruned,
            result.leases_expired,
            result.cycle_ms,
            i32::from(orac_available),
        ],
    )?;

    if result.cycle_ms > 50.0 {
        tracing::warn!(cycle_ms = result.cycle_ms, "buoy consolidation exceeded 50ms SLO");
    }

    Ok(result)
}

/// Step 1: Consume STDP events and apply weight deltas.
fn step_intake(tx: &Transaction<'_>, _config: &BuoyConfig) -> Result<u32, EngineError> {
    let events: Vec<(String, f64, String)> = {
        let mut stmt = tx.prepare(
            "SELECT pathway_id, delta, event_type FROM buoy_stdp_intake WHERE consumed = 0",
        )?;
        let rows: Vec<_> = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?, row.get::<_, String>(2)?))
        })?
        .filter_map(std::result::Result::ok)
        .collect();
        rows
    };

    let count = events.len();
    for (pathway_id, delta, event_type) in &events {
        if event_type == "ltp" {
            tx.execute(
                "UPDATE buoy_pathway SET weight = MIN(0.95, weight + ?1), hit_count = hit_count + 1 WHERE id = ?2",
                params![delta, pathway_id],
            )?;
        } else {
            tx.execute(
                "UPDATE buoy_pathway SET weight = MAX(0.05, weight - ABS(?1)) WHERE id = ?2",
                params![delta, pathway_id],
            )?;
        }
    }

    tx.execute("UPDATE buoy_stdp_intake SET consumed = 1 WHERE consumed = 0", [])?;

    #[allow(clippy::cast_possible_truncation)]
    Ok(count as u32)
}

/// Step 2: Decay all pathway weights by `decay_rate`.
fn step_decay(tx: &Transaction<'_>, config: &BuoyConfig) -> Result<u32, EngineError> {
    let changed = tx.execute(
        "UPDATE buoy_pathway SET weight = weight * ?1, updated_at = strftime('%s', 'now')",
        params![config.decay_rate],
    )?;
    #[allow(clippy::cast_possible_truncation)]
    Ok(changed as u32)
}

/// Step 3: Apply buoy maintenance pulse to graduated pathways with active lease.
fn step_buoy(tx: &Transaction<'_>, config: &BuoyConfig) -> Result<u32, EngineError> {
    let changed = tx.execute(
        "UPDATE buoy_pathway SET weight = MIN(0.95, weight + ?1), updated_at = strftime('%s', 'now')
         WHERE lease_remaining > 0 AND hit_count >= ?2",
        params![config.buoy_rate, config.graduation_threshold],
    )?;
    #[allow(clippy::cast_possible_truncation)]
    Ok(changed as u32)
}

/// Step 4: Reinforce pathways that were naturally fired this session.
fn step_reinforce(
    tx: &Transaction<'_>,
    config: &BuoyConfig,
    current_session: u64,
    fired_patterns: &[String],
) -> Result<u32, EngineError> {
    if fired_patterns.is_empty() {
        return Ok(0);
    }
    let mut count = 0u32;
    for pattern_id in fired_patterns {
        let changed = tx.execute(
            "UPDATE buoy_pathway SET
                weight = MIN(0.95, weight + ?1),
                hit_count = hit_count + 1,
                last_fired = ?2,
                lease_remaining = ?3,
                updated_at = strftime('%s', 'now')
             WHERE id = ?4",
            params![config.reinforce_rate, current_session, config.lease_capacity, pattern_id],
        )?;
        #[allow(clippy::cast_possible_truncation)]
        { count += changed as u32; }
    }
    Ok(count)
}

/// Step 5: Recompute disk embedding from `hit_count`.
fn step_embed(tx: &Transaction<'_>) -> Result<u32, EngineError> {
    let pathways: Vec<(String, u32)> = {
        let mut stmt = tx.prepare("SELECT id, hit_count FROM buoy_pathway")?;
        let rows: Vec<_> = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?)))?
            .filter_map(std::result::Result::ok)
            .collect();
        rows
    };

    let count = pathways.len();
    for (id, hit_count) in &pathways {
        let point = disk::embed(*hit_count, id);
        tx.execute(
            "UPDATE buoy_pathway SET disk_r = ?1, disk_theta = ?2 WHERE id = ?3",
            params![point.r, point.theta, id],
        )?;
    }

    #[allow(clippy::cast_possible_truncation)]
    Ok(count as u32)
}

/// Step 6: Prune pathways below threshold (NA-02: guard against pruning in-use pathways).
fn step_prune(tx: &Transaction<'_>, config: &BuoyConfig) -> Result<u32, EngineError> {
    let pruned = tx.execute(
        "DELETE FROM buoy_pathway WHERE weight < ?1 AND hit_count < ?2",
        params![config.prune_threshold, config.graduation_threshold],
    )?;
    #[allow(clippy::cast_possible_truncation)]
    Ok(pruned as u32)
}

/// Step 7: Decrement lease and expire.
fn step_lease(
    tx: &Transaction<'_>,
    _config: &BuoyConfig,
    _current_session: u64,
    lease_units: f64,
    orac_available: bool,
) -> Result<u32, EngineError> {
    // G-08: Don't consume lease during ORAC outage
    if !orac_available {
        return Ok(0);
    }

    tx.execute(
        "UPDATE buoy_pathway SET lease_remaining = MAX(0.0, lease_remaining - ?1)
         WHERE lease_remaining > 0",
        params![lease_units],
    )?;

    // Pathways whose lease just hit zero: demote to floor tier, keep position (plan decision)
    let expired = tx.execute(
        "UPDATE buoy_pathway SET tier = 'floor'
         WHERE lease_remaining <= 0 AND tier = 'buoyed'",
        [],
    )?;

    #[allow(clippy::cast_possible_truncation)]
    Ok(expired as u32)
}

/// Compute weighted lease units for this session (NA-10).
fn compute_lease_units(tool_call_count: u64, config: &BuoyConfig) -> f64 {
    #[allow(clippy::cast_precision_loss)]
    let raw = tool_call_count as f64 / config.lease_tool_call_normalizer;
    raw.min(config.lease_unit_base)
}

/// Update tier classification based on current weights.
fn update_tiers(tx: &Transaction<'_>) -> Result<(), EngineError> {
    tx.execute(
        "UPDATE buoy_pathway SET tier = 'active' WHERE weight >= 0.7",
        [],
    )?;
    tx.execute(
        "UPDATE buoy_pathway SET tier = 'buoyed' WHERE weight >= 0.4 AND weight < 0.7 AND lease_remaining > 0",
        [],
    )?;
    tx.execute(
        "UPDATE buoy_pathway SET tier = 'floor' WHERE weight < 0.4 OR (weight < 0.7 AND lease_remaining <= 0)",
        [],
    )?;
    Ok(())
}

/// Seed initial attractor positions into the database.
///
/// # Errors
///
/// Returns `EngineError` if insert fails.
pub fn seed_attractors(conn: &Connection) -> Result<(), EngineError> {
    let attractors = disk::habitat_attractors();
    for (service, point) in attractors {
        conn.execute(
            "INSERT OR REPLACE INTO buoy_attractor (service, disk_r, disk_theta, pathway_count)
             VALUES (?1, ?2, ?3, 0)",
            params![service, point.r, point.theta],
        )?;
    }
    Ok(())
}

/// Query the current buoy status summary.
///
/// # Errors
///
/// Returns `EngineError` on query failure.
pub fn status(conn: &Connection) -> Result<BuoyStatus, EngineError> {
    let total: u32 = conn.query_row(
        "SELECT COUNT(*) FROM buoy_pathway",
        [],
        |row| row.get(0),
    )?;
    let active: u32 = conn.query_row(
        "SELECT COUNT(*) FROM buoy_pathway WHERE tier = 'active'",
        [],
        |row| row.get(0),
    )?;
    let buoyed: u32 = conn.query_row(
        "SELECT COUNT(*) FROM buoy_pathway WHERE tier = 'buoyed'",
        [],
        |row| row.get(0),
    )?;
    let floor: u32 = conn.query_row(
        "SELECT COUNT(*) FROM buoy_pathway WHERE tier = 'floor'",
        [],
        |row| row.get(0),
    )?;
    let mean_weight: f64 = conn.query_row(
        "SELECT COALESCE(AVG(weight), 0.0) FROM buoy_pathway",
        [],
        |row| row.get(0),
    )?;
    let last_cycle_ms: Option<f64> = conn.query_row(
        "SELECT cycle_ms FROM buoy_cycle_log ORDER BY id DESC LIMIT 1",
        [],
        |row| row.get(0),
    ).ok();

    Ok(BuoyStatus {
        total,
        active,
        buoyed,
        floor,
        mean_weight,
        last_cycle_ms,
    })
}

/// Summary of buoy system state.
#[derive(Debug, Clone, serde::Serialize)]
pub struct BuoyStatus {
    pub total: u32,
    pub active: u32,
    pub buoyed: u32,
    pub floor: u32,
    pub mean_weight: f64,
    pub last_cycle_ms: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema;

    fn test_db() -> Connection {
        schema::open_memory().expect("test db")
    }

    fn seed_pathways(conn: &Connection, n: u32) {
        for i in 0..n {
            conn.execute(
                "INSERT INTO buoy_pathway (id, weight, hit_count, disk_r, disk_theta, tier, service)
                 VALUES (?1, 0.5, ?2, 0.5, 0.0, 'buoyed', 'orac-sidecar')",
                params![format!("path-{i}"), i],
            )
            .expect("seed");
        }
    }

    #[test]
    fn full_cycle_no_fired() {
        let conn = test_db();
        seed_pathways(&conn, 10);
        let tx = conn.unchecked_transaction().expect("tx");
        let config = BuoyConfig::default();
        let result = consolidate(&tx, &config, 100, &[], 50, true).expect("consolidate");
        tx.commit().expect("commit");
        assert_eq!(result.decayed, 10);
        assert!(result.cycle_ms < 100.0);
    }

    #[test]
    fn reinforcement_increases_weight() {
        let conn = test_db();
        conn.execute(
            "INSERT INTO buoy_pathway (id, weight, hit_count, disk_r, disk_theta, tier, service)
             VALUES ('target', 0.4, 2, 0.5, 0.0, 'floor', 'orac-sidecar')",
            [],
        ).expect("seed");

        let tx = conn.unchecked_transaction().expect("tx");
        let config = BuoyConfig::default();
        let fired = vec!["target".to_owned()];
        let result = consolidate(&tx, &config, 100, &fired, 50, true).expect("consolidate");
        tx.commit().expect("commit");

        assert_eq!(result.reinforced, 1);
        let weight: f64 = conn.query_row(
            "SELECT weight FROM buoy_pathway WHERE id = 'target'",
            [],
            |row| row.get(0),
        ).expect("query");
        assert!(weight > 0.4, "weight should have increased from reinforcement, got {weight}");
    }

    #[test]
    fn decay_reduces_weight() {
        let conn = test_db();
        conn.execute(
            "INSERT INTO buoy_pathway (id, weight, hit_count, disk_r, disk_theta, tier, service)
             VALUES ('decay-test', 0.8, 5, 0.3, 0.0, 'active', 'pv2')",
            [],
        ).expect("seed");

        let tx = conn.unchecked_transaction().expect("tx");
        let config = BuoyConfig::default();
        consolidate(&tx, &config, 100, &[], 50, true).expect("consolidate");
        tx.commit().expect("commit");

        let weight: f64 = conn.query_row(
            "SELECT weight FROM buoy_pathway WHERE id = 'decay-test'",
            [],
            |row| row.get(0),
        ).expect("query");
        assert!(weight < 0.8, "weight should have decayed, got {weight}");
    }

    #[test]
    fn buoy_pulse_maintains_graduated() {
        let conn = test_db();
        conn.execute(
            "INSERT INTO buoy_pathway (id, weight, hit_count, disk_r, disk_theta, tier, service, lease_remaining)
             VALUES ('graduated', 0.5, 5, 0.3, 0.0, 'buoyed', 'orac-sidecar', 20.0)",
            [],
        ).expect("seed");

        let tx = conn.unchecked_transaction().expect("tx");
        let config = BuoyConfig::default();
        consolidate(&tx, &config, 100, &[], 50, true).expect("consolidate");
        tx.commit().expect("commit");

        let weight: f64 = conn.query_row(
            "SELECT weight FROM buoy_pathway WHERE id = 'graduated'",
            [],
            |row| row.get(0),
        ).expect("query");
        // decay(0.98) * 0.5 = 0.49, then buoy + 0.02 = 0.51
        assert!((weight - 0.51).abs() < 0.01, "buoy should maintain at ~0.5, got {weight}");
    }

    #[test]
    fn prune_removes_low_weight() {
        let conn = test_db();
        conn.execute(
            "INSERT INTO buoy_pathway (id, weight, hit_count, disk_r, disk_theta, tier, service)
             VALUES ('weak', 0.04, 1, 0.9, 0.0, 'floor', 'me')",
            [],
        ).expect("seed");

        let tx = conn.unchecked_transaction().expect("tx");
        let config = BuoyConfig::default();
        let result = consolidate(&tx, &config, 100, &[], 50, true).expect("consolidate");
        tx.commit().expect("commit");

        assert!(result.pruned > 0);
        let count: u32 = conn.query_row(
            "SELECT COUNT(*) FROM buoy_pathway WHERE id = 'weak'",
            [],
            |row| row.get(0),
        ).expect("query");
        assert_eq!(count, 0, "weak pathway should be pruned");
    }

    #[test]
    fn prune_spares_graduated() {
        let conn = test_db();
        conn.execute(
            "INSERT INTO buoy_pathway (id, weight, hit_count, disk_r, disk_theta, tier, service)
             VALUES ('graduated-low', 0.04, 5, 0.3, 0.0, 'floor', 'orac-sidecar')",
            [],
        ).expect("seed");

        let tx = conn.unchecked_transaction().expect("tx");
        let config = BuoyConfig::default();
        consolidate(&tx, &config, 100, &[], 50, true).expect("consolidate");
        tx.commit().expect("commit");

        let count: u32 = conn.query_row(
            "SELECT COUNT(*) FROM buoy_pathway WHERE id = 'graduated-low'",
            [],
            |row| row.get(0),
        ).expect("query");
        assert_eq!(count, 1, "graduated pathway should NOT be pruned even at low weight");
    }

    #[test]
    fn lease_expiry_demotes_to_floor() {
        let conn = test_db();
        conn.execute(
            "INSERT INTO buoy_pathway (id, weight, hit_count, disk_r, disk_theta, tier, service, lease_remaining)
             VALUES ('expiring', 0.5, 5, 0.3, 0.0, 'buoyed', 'orac-sidecar', 0.5)",
            [],
        ).expect("seed");

        let tx = conn.unchecked_transaction().expect("tx");
        let config = BuoyConfig::default();
        consolidate(&tx, &config, 100, &[], 50, true).expect("consolidate");
        tx.commit().expect("commit");

        let tier: String = conn.query_row(
            "SELECT tier FROM buoy_pathway WHERE id = 'expiring'",
            [],
            |row| row.get(0),
        ).expect("query");
        assert_eq!(tier, "floor", "expired lease should demote to floor");
    }

    #[test]
    fn lease_not_consumed_during_orac_outage() {
        let conn = test_db();
        conn.execute(
            "INSERT INTO buoy_pathway (id, weight, hit_count, disk_r, disk_theta, tier, service, lease_remaining)
             VALUES ('protected', 0.5, 5, 0.3, 0.0, 'buoyed', 'orac-sidecar', 10.0)",
            [],
        ).expect("seed");

        let tx = conn.unchecked_transaction().expect("tx");
        let config = BuoyConfig::default();
        consolidate(&tx, &config, 100, &[], 50, false).expect("consolidate");
        tx.commit().expect("commit");

        let lease: f64 = conn.query_row(
            "SELECT lease_remaining FROM buoy_pathway WHERE id = 'protected'",
            [],
            |row| row.get(0),
        ).expect("query");
        assert!((lease - 10.0).abs() < 0.01, "lease should not decrement during ORAC outage, got {lease}");
    }

    #[test]
    fn stdp_intake_boosts_weight() {
        let conn = test_db();
        conn.execute(
            "INSERT INTO buoy_pathway (id, weight, hit_count, disk_r, disk_theta, tier, service)
             VALUES ('stdp-target', 0.5, 3, 0.3, 0.0, 'buoyed', 'orac-sidecar')",
            [],
        ).expect("seed pathway");
        conn.execute(
            "INSERT INTO buoy_stdp_intake (event_type, pathway_id, delta, source, tick)
             VALUES ('ltp', 'stdp-target', 0.05, 'orac', 1000)",
            [],
        ).expect("seed event");

        let tx = conn.unchecked_transaction().expect("tx");
        let config = BuoyConfig::default();
        let result = consolidate(&tx, &config, 100, &[], 50, true).expect("consolidate");
        tx.commit().expect("commit");

        assert_eq!(result.intake_consumed, 1);
        let hit: u32 = conn.query_row(
            "SELECT hit_count FROM buoy_pathway WHERE id = 'stdp-target'",
            [],
            |row| row.get(0),
        ).expect("query");
        assert_eq!(hit, 4, "LTP should increment hit_count");
    }

    #[test]
    fn weighted_lease_units_proportional() {
        let config = BuoyConfig::default();
        let low = compute_lease_units(5, &config);
        let mid = compute_lease_units(25, &config);
        let high = compute_lease_units(100, &config);
        assert!(low < mid);
        assert!(mid < high);
        assert!((high - 1.0).abs() < 0.01, "100 tool calls should cap at 1.0 unit");
    }

    #[test]
    fn embed_updates_geometry() {
        let conn = test_db();
        conn.execute(
            "INSERT INTO buoy_pathway (id, weight, hit_count, disk_r, disk_theta, tier, service)
             VALUES ('embed-test', 0.5, 10, 0.9, 0.0, 'buoyed', 'orac-sidecar')",
            [],
        ).expect("seed");

        let tx = conn.unchecked_transaction().expect("tx");
        let config = BuoyConfig::default();
        consolidate(&tx, &config, 100, &[], 50, true).expect("consolidate");
        tx.commit().expect("commit");

        let r: f64 = conn.query_row(
            "SELECT disk_r FROM buoy_pathway WHERE id = 'embed-test'",
            [],
            |row| row.get(0),
        ).expect("query");
        assert!(r < 0.9, "hit_count=10 should embed closer to center than 0.9, got {r}");
    }

    #[test]
    fn status_returns_counts() {
        let conn = test_db();
        seed_pathways(&conn, 5);
        let s = status(&conn).expect("status");
        assert_eq!(s.total, 5);
    }

    #[test]
    fn seed_attractors_creates_five() {
        let conn = test_db();
        seed_attractors(&conn).expect("seed");
        let count: u32 = conn.query_row(
            "SELECT COUNT(*) FROM buoy_attractor",
            [],
            |row| row.get(0),
        ).expect("query");
        assert_eq!(count, 5);
    }

    #[test]
    fn cycle_log_persists() {
        let conn = test_db();
        seed_pathways(&conn, 3);
        let tx = conn.unchecked_transaction().expect("tx");
        let config = BuoyConfig::default();
        consolidate(&tx, &config, 205, &[], 50, true).expect("consolidate");
        tx.commit().expect("commit");

        let session: u64 = conn.query_row(
            "SELECT session FROM buoy_cycle_log ORDER BY id DESC LIMIT 1",
            [],
            |row| row.get(0),
        ).expect("query");
        assert_eq!(session, 205);
    }

    #[test]
    fn tier_classification_active() {
        let conn = test_db();
        conn.execute(
            "INSERT INTO buoy_pathway (id, weight, hit_count, disk_r, disk_theta, tier, service, lease_remaining)
             VALUES ('high', 0.85, 10, 0.2, 0.0, 'floor', 'orac-sidecar', 20.0)",
            [],
        ).expect("seed");

        let tx = conn.unchecked_transaction().expect("tx");
        let config = BuoyConfig::default();
        consolidate(&tx, &config, 100, &[], 50, true).expect("consolidate");
        tx.commit().expect("commit");

        let tier: String = conn.query_row(
            "SELECT tier FROM buoy_pathway WHERE id = 'high'",
            [],
            |row| row.get(0),
        ).expect("query");
        assert_eq!(tier, "active");
    }
}
