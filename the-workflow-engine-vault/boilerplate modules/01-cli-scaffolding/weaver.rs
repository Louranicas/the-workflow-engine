//! `weaver` — Habitat continuity daemon.
//!
//! Maintains `HABITAT_STATE.md` and serves the Weaver HTTP API on port 8141.
//!
//! # Cadences
//!
//! - Every **5 s**: refresh Atuin KV snapshot (placeholder until Atuin KV
//!   integration is wired in Wave 4).
//! - Every **30 s**: probe ORAC/PV2/SX-v2/ME/POVM → merge → write snapshot row
//!   → render `HABITAT_STATE.md`.
//! - Every **60 s**: cold differentiation pass if any sphere has empty buoys.
//!
//! # Usage
//!
//! ```text
//! weaver [OPTIONS]
//!
//! Options:
//!   --port <PORT>        HTTP port [default: 8141]
//!   --bind <ADDR>        Bind address [default: 127.0.0.1]
//!   --db <PATH>          state.db path [default: ~/.local/share/habitat-conductor/state.db]
//!   --state-md <PATH>    HABITAT_STATE.md output path
//!   -v, --version
//!   -h, --help
//! ```

use std::{net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};

use anyhow::Context as _;
use clap::Parser;
use habitat_conductor::{
    api::{WeaverState, build_router, render_state_md},
    snapshot::{ProbeConfig, probe_all},
    state::{StateDb, SnapshotRow, default_db_path},
};
use parking_lot::Mutex;
use tokio::time::interval;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

/// Weaver — Habitat continuity daemon.
#[derive(Debug, Parser)]
#[command(name = "weaver", version, about)]
struct Args {
    /// HTTP bind port.
    #[arg(long, default_value = "8141")]
    port: u16,

    /// HTTP bind address.
    #[arg(long, default_value = "127.0.0.1")]
    bind: String,

    /// Path to `state.db` (`SQLite` WAL).
    #[arg(long)]
    db: Option<PathBuf>,

    /// Path to write `HABITAT_STATE.md`.
    #[arg(long)]
    state_md: Option<PathBuf>,
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .json()
        .init();

    info!(port = args.port, "weaver starting");

    // Resolve state.db path.
    let db_path = match args.db {
        Some(p) => p,
        None => default_db_path().context("cannot determine state.db path")?,
    };

    // Ensure parent directory exists.
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating db directory: {}", parent.display()))?;
    }

    // Open + migrate database.
    let mut db = StateDb::open(&db_path)
        .with_context(|| format!("opening state.db at {}", db_path.display()))?;
    db.migrate().context("applying migrations")?;
    info!(?db_path, "state.db ready");

    let db = Arc::new(Mutex::new(db));
    let state_md_content = Arc::new(Mutex::new(String::new()));

    let state_md_path = args.state_md.clone().unwrap_or_else(|| {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("habitat-conductor")
            .join("HABITAT_STATE.md")
    });

    // Build shared HTTP state.
    let shared = Arc::new(WeaverState {
        db: Arc::clone(&db),
        state_md: Arc::clone(&state_md_content),
    });

    // Spawn HTTP server.
    let router = build_router(Arc::clone(&shared));
    let addr: SocketAddr = format!("{}:{}", args.bind, args.port)
        .parse()
        .context("invalid bind address")?;
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("binding {addr}"))?;
    info!(%addr, "weaver HTTP server listening");

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, router).await {
            error!("weaver HTTP server error: {e}");
        }
    });

    // Probe loop.
    let probe_cfg = ProbeConfig::default();
    let mut probe_interval = interval(Duration::from_secs(30));
    let mut kv_interval = interval(Duration::from_secs(5));
    let mut diff_interval = interval(Duration::from_secs(60));
    // Skip the first immediate tick so the daemon has time to settle.
    probe_interval.tick().await;
    kv_interval.tick().await;
    diff_interval.tick().await;

    info!("weaver daemon loop starting");

    loop {
        tokio::select! {
            _ = kv_interval.tick() => {
                // Atuin KV snapshot refresh — placeholder for Wave 4 wiring.
                // (no-op until atuin kv client is available as a library)
            }
            _ = probe_interval.tick() => {
                match probe_all(&probe_cfg).await {
                    Ok(snap) => {
                        let row = snapshot_to_row(&snap);
                        let md = render_state_md(&row);

                        // Write to state.db (rusqlite is sync — AP29 fix: wrap in spawn_blocking).
                        let db_clone = Arc::clone(&db);
                        let row_clone = row.clone();
                        match tokio::task::spawn_blocking(move || db_clone.lock().insert_snapshot(&row_clone)).await {
                            Ok(Ok(())) => {}
                            Ok(Err(e)) => warn!("failed to insert snapshot: {e}"),
                            Err(e) => warn!("insert_snapshot spawn_blocking join error: {e}"),
                        }

                        // Update in-memory state_md.
                        md.clone_into(&mut *state_md_content.lock());

                        // Write HABITAT_STATE.md to disk.
                        if let Some(parent) = state_md_path.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        if let Err(e) = std::fs::write(&state_md_path, &md) {
                            warn!(?state_md_path, "failed to write HABITAT_STATE.md: {e}");
                        }

                        info!(
                            fitness = snap.fitness,
                            field_r = snap.field_r,
                            breakers = snap.breakers_open,
                            "snapshot written"
                        );
                    }
                    Err(e) => {
                        warn!("probe_all failed: {e}");
                    }
                }
            }
            _ = diff_interval.tick() => {
                // Cold differentiation pass (placeholder — no PV2 sphere list yet).
                // Full wiring happens when PV2 /spheres endpoint is called.
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Conversion helper
// ---------------------------------------------------------------------------

fn snapshot_to_row(snap: &habitat_conductor::snapshot::HabitatSnapshot) -> SnapshotRow {
    SnapshotRow {
        ts: snap.ts,
        fitness: snap.fitness,
        field_r: snap.field_r,
        thermal_t: snap.thermal_t,
        sphere_count: snap.sphere_count,
        ralph_phase: snap.ralph_phase.clone(),
        ralph_gen: snap.ralph_gen,
        ltp: snap.ltp,
        ltd: snap.ltd,
        mutations_proposed: snap.mutations_proposed,
        mutations_accepted: snap.mutations_accepted,
        breakers_open: snap.breakers_open,
        probe_failures: snap.probe_failures,
        me_fitness: snap.me_fitness,
        raw_json: snap.raw_json.clone(),
    }
}
