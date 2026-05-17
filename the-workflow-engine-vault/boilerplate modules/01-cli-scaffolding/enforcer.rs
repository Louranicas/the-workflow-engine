//! Wave 3 enforcer daemon — polls `state.db.divergence_reports` and
//! dispatches rollbacks or proposals for HIGH/CRITICAL divergences.
//!
//! # Activation
//!
//! The enforcer is inert until `CONDUCTOR_ENFORCEMENT_ENABLED=1` is set.
//! Without the env var it runs the poll loop but all evaluations return `NoOp`.
//!
//! # Safety
//!
//! - CRITICAL divergences → auto-rollback via `forge --rollback <service>`.
//! - HIGH divergences → propose-only (writes Atuin KV, awaits human ack).
//! - Exception: `soak_health_degrade` at HIGH → auto-rollback (live metric).
//! - Per-service 300 s cooldown prevents rollback-loop storms.
//! - Every action is written to `enforcement_actions` BEFORE any rollback.
//!
//! # Deployment note
//!
//! Binary: `~/.local/bin/enforcer`
//! devenv: Batch 5, `depends_on = ["weaver", "zen"]`
//! Watch 24 h after first enable — false-positive rollbacks are the dominant risk.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use clap::Parser;
use parking_lot::Mutex;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

use habitat_conductor::{
    enforcement::{EnforcerDb, COOLDOWN_SECS},
    state::{DivergenceReport, StateDb, default_db_path},
};

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

/// Habitat Conductor Wave 3 — enforcer daemon.
///
/// Polls `state.db.divergence_reports` every 10 s. For CRITICAL divergences,
/// invokes `forge --rollback <service>` (requires `CONDUCTOR_ENFORCEMENT_ENABLED=1`).
/// For HIGH divergences, writes an Atuin KV proposal and awaits human approval.
#[derive(Parser, Debug)]
#[command(
    name    = "enforcer",
    version = env!("CARGO_PKG_VERSION"),
    about   = "Habitat Conductor Wave 3 — active enforcement daemon"
)]
struct Cli {
    /// Path to `state.db`. Defaults to `~/.local/share/habitat-conductor/state.db`.
    #[arg(long, env = "CONDUCTOR_DB_PATH")]
    db_path: Option<std::path::PathBuf>,

    /// Poll interval in seconds (default: 10).
    #[arg(long, default_value_t = 10, env = "CONDUCTOR_POLL_SECS")]
    poll_secs: u64,

    /// Per-service rollback cooldown in seconds (default: 300).
    #[arg(long, default_value_t = COOLDOWN_SECS, env = "CONDUCTOR_COOLDOWN_SECS")]
    cooldown_secs: u64,

    /// Number of recent divergence reports to read per poll cycle (default: 50).
    #[arg(long, default_value_t = 50)]
    batch_size: usize,
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

/// F-CONDUCTOR-04: per-report retry counter ceiling.
/// After this many consecutive enforcement failures for the same report id,
/// the report is poison-pilled (audit row written if possible, HWM advances).
const MAX_RETRIES: u32 = 5;

#[tokio::main]
#[allow(clippy::too_many_lines)] // daemon main: boot + signal handler + poll loop are naturally long
async fn main() -> anyhow::Result<()> {
    // Structured tracing — JSON in production, pretty in terminals.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    let enabled = std::env::var("CONDUCTOR_ENFORCEMENT_ENABLED").as_deref() == Ok("1");
    if enabled {
        warn!("enforcement ENABLED — auto-rollbacks may fire for CRITICAL divergences");
    } else {
        info!("enforcement DISABLED (NoOp mode) — set CONDUCTOR_ENFORCEMENT_ENABLED=1 to activate");
    }

    // Resolve DB path.
    let db_path = cli.db_path.map_or_else(default_db_path, Ok)
        .context("cannot determine state.db path")?;

    info!(?db_path, poll_secs = cli.poll_secs, cooldown_secs = cli.cooldown_secs, "enforcer starting");

    // Open and migrate state.db.
    let mut db = StateDb::open(&db_path)
        .with_context(|| format!("cannot open state.db at {}", db_path.display()))?;
    db.migrate().context("state.db migration failed")?;

    let db = Arc::new(Mutex::new(db));
    let mut enforcer_db = EnforcerDb::new(enabled, cli.cooldown_secs, Arc::clone(&db));

    // F-CONDUCTOR-04: load last_seen_id from the kv store so daemon restarts
    // do not replay the entire report history. Falls back to the current
    // maximum report id if no persisted watermark is found.
    //
    // AP29 fix (S1001791): rusqlite is sync — wrap in spawn_blocking.
    let mut last_seen_id: i64 = {
        let db_clone = Arc::clone(&db);
        tokio::task::spawn_blocking(move || -> anyhow::Result<i64> {
            let guard = db_clone.lock();
            // First try: read persisted watermark from kv store.
            if let Some(val) = guard.kv_get("enforcer.last_seen_id").context("kv_get")? {
                if let Ok(id) = val.parse::<i64>() {
                    return Ok(id);
                }
                warn!(val, "enforcer.last_seen_id in kv is not a valid i64 — resetting");
            }
            // Fallback: initialise from the current maximum report id so we
            // don't replay history from the beginning on first run.
            Ok(guard
                .divergence_reports(None, 1)
                .context("initial id probe")?
                .first()
                .and_then(|r| r.id)
                .unwrap_or(0))
        })
        .await
        .context("initial id probe spawn_blocking join")??
    };

    info!(last_seen_id, "poll loop starting");

    // F-CONDUCTOR-04: per-report retry counter (ceiling at module-scope `MAX_RETRIES`).
    // After MAX_RETRIES consecutive enforcement failures for the same report id,
    // we write an `audit_write_failed` action row (if possible) and advance the
    // watermark — the report is considered poison-pilled and won't block the loop.
    let mut retry_counts: HashMap<i64, u32> = HashMap::new();

    loop {
        tokio::time::sleep(Duration::from_secs(cli.poll_secs)).await;

        // Fetch recent reports — process only those newer than last_seen_id.
        // AP29 fix (S1001791): rusqlite is sync — wrap in spawn_blocking.
        let reports = {
            let db_clone = Arc::clone(&db);
            let batch_size = cli.batch_size;
            tokio::task::spawn_blocking(move || -> anyhow::Result<Vec<DivergenceReport>> {
                let guard = db_clone.lock();
                guard
                    .divergence_reports(None, batch_size)
                    .context("divergence_reports query")
            })
            .await
            .context("divergence_reports spawn_blocking join")??
        };

        let new_reports: Vec<_> = reports
            .into_iter()
            .filter(|r| r.id.is_some_and(|id| id > last_seen_id))
            .collect();

        if new_reports.is_empty() {
            continue;
        }

        info!(count = new_reports.len(), "processing new divergence reports");

        for report in &new_reports {
            let report_id = report.id.unwrap_or(0);

            match enforcer_db.process(report) {
                Ok(action) => {
                    info!(
                        kind = %report.kind,
                        severity = %report.severity,
                        action = %action.as_str(),
                        "enforcement decision"
                    );
                    // F-CONDUCTOR-04: advance watermark ONLY on success.
                    // On error we leave last_seen_id untouched so the report
                    // is retried on the next poll cycle.
                    retry_counts.remove(&report_id);
                    if report_id > last_seen_id {
                        last_seen_id = report_id;
                        // Persist watermark — ignore write failure (non-fatal).
                        let db_clone = Arc::clone(&db);
                        let id_str = last_seen_id.to_string();
                        if let Err(e) = tokio::task::spawn_blocking(move || {
                            db_clone.lock().kv_set("enforcer.last_seen_id", &id_str)
                        })
                        .await
                        {
                            warn!("failed to persist last_seen_id to kv: {e}");
                        }
                    }
                }
                Err(e) => {
                    // Log and continue — one failed enforcement must not halt the loop.
                    error!(
                        kind = %report.kind,
                        report_id,
                        error = %e,
                        "enforcement error — report will be retried next cycle"
                    );

                    // F-CONDUCTOR-04: track retry count. After MAX_RETRIES, write
                    // a poison-pill audit row and advance the watermark so the loop
                    // is not permanently blocked by one bad report.
                    let attempts = retry_counts.entry(report_id).or_insert(0);
                    *attempts += 1;
                    if *attempts >= MAX_RETRIES {
                        error!(
                            report_id,
                            kind = %report.kind,
                            attempts = MAX_RETRIES,
                            "max retries exceeded — writing audit_write_failed and advancing watermark"
                        );
                        // Best-effort audit row (poison-pill marker).
                        let db_clone = Arc::clone(&db);
                        let kind = report.kind.clone();
                        let sev = report.severity.as_str().to_owned();
                        let dr_id = report.id;
                        if let Err(e2) = tokio::task::spawn_blocking(move || {
                            db_clone.lock().insert_enforcement_action(
                                chrono::Utc::now().timestamp_millis(),
                                "enforcer_internal",
                                &kind,
                                "audit_write_failed",
                                &sev,
                                dr_id,
                                "unknown",
                                "auto",
                                r#"{"reason":"max_retries_exceeded"}"#,
                            )
                        })
                        .await
                        {
                            warn!("failed to write audit_write_failed row: {e2}");
                        }
                        retry_counts.remove(&report_id);
                        if report_id > last_seen_id {
                            last_seen_id = report_id;
                            let db_clone = Arc::clone(&db);
                            let id_str = last_seen_id.to_string();
                            if let Err(e2) = tokio::task::spawn_blocking(move || {
                                db_clone.lock().kv_set("enforcer.last_seen_id", &id_str)
                            })
                            .await
                            {
                                warn!("failed to persist last_seen_id (poison-pill path): {e2}");
                            }
                        }
                    }
                }
            }
        }
    }
}
