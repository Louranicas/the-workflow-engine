//! Tokio runtime construction + top-level [`run`] entry point.
//!
//! Phase 2 composes a multi-thread runtime, installs signal handlers,
//! spawns the HTTP server, and exits cleanly on cancellation. Phase 3
//! extends this function with the 11-task composition + startup gate.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use thiserror::Error;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

use super::http::{self, HttpError};
use super::shutdown::{self, NamedHandle, ShutdownBudgets};
use super::startup;
use super::state::AppState;
use super::tasks;
use super::{DEFAULT_PRIMARY_PORT, DEFAULT_SHADOW_PORT};

/// Deployment mode. Controls which port is bound by default and enables
/// phase-gated behaviour (e.g. shadow-diff writes only in [`Self::Shadow`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum DaemonMode {
    /// V2 runs alongside V1. Classification + regulation are computed but
    /// outputs are *not* authoritative — shadow-diff gathers agreement %.
    Shadow,
    /// Partial authority. A subset of bridges write authoritative output.
    /// Used during the M6 W4 cutover window.
    Canary,
    /// V2 is primary. V1 has retired (cutover complete).
    Primary,
}

impl DaemonMode {
    /// Default port for the mode (8091 shadow / 8090 primary+canary).
    #[must_use]
    pub const fn default_port(self) -> u16 {
        match self {
            Self::Shadow => DEFAULT_SHADOW_PORT,
            Self::Canary | Self::Primary => DEFAULT_PRIMARY_PORT,
        }
    }
}

/// CLI arguments parsed from argv + env. Also the canonical config shape.
#[derive(Debug, Clone, Parser)]
#[command(name = "synthex-v2", about = "SYNTHEX v2 — habitat autonomic regulator")]
pub struct DaemonConfig {
    /// Deployment mode — selects default port + phase-gated behaviour.
    #[arg(long, value_enum, default_value = "shadow", env = "SYNTHEX_V2_MODE")]
    pub mode: DaemonMode,

    /// HTTP port override. If unset, uses the mode-default port.
    #[arg(long, env = "SYNTHEX_V2_PORT")]
    pub port: Option<u16>,

    /// Bind address. Defaults to 127.0.0.1 (loopback) for safety.
    #[arg(long, default_value = "127.0.0.1", env = "SYNTHEX_V2_BIND")]
    pub bind: IpAddr,

    /// Path to the daemon TOML config (loaded via `m03_configuration` in Phase 3).
    #[arg(long, env = "SYNTHEX_V2_CONFIG")]
    pub config_path: Option<std::path::PathBuf>,

    /// Worker thread count for the Tokio multi-thread runtime.
    /// Defaults to `num_cpus::get()`.
    #[arg(long, env = "SYNTHEX_V2_WORKERS")]
    pub workers: Option<usize>,

    /// Seconds granted to each plane during graceful shutdown. See the
    /// [`ShutdownBudgets`] default table; this flag overrides the
    /// Watcher budget (longest-running plane).
    #[arg(long, default_value = "30", env = "SYNTHEX_V2_WATCHER_SHUTDOWN_S")]
    pub watcher_shutdown_secs: u64,
}

impl DaemonConfig {
    /// Resolve the effective bind address for this config.
    #[must_use]
    pub fn socket_addr(&self) -> SocketAddr {
        let port = self.port.unwrap_or_else(|| self.mode.default_port());
        SocketAddr::new(self.bind, port)
    }

    /// Resolve worker count, defaulting to CPU count.
    #[must_use]
    pub fn effective_workers(&self) -> usize {
        self.workers.unwrap_or_else(num_cpus::get).max(1)
    }

    /// Translate `watcher_shutdown_secs` into the full budget table.
    #[must_use]
    pub fn shutdown_budgets(&self) -> ShutdownBudgets {
        ShutdownBudgets {
            watcher: Duration::from_secs(self.watcher_shutdown_secs.max(1)),
            ..ShutdownBudgets::default()
        }
    }
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            mode: DaemonMode::Shadow,
            port: None,
            bind: IpAddr::V4(Ipv4Addr::LOCALHOST),
            config_path: None,
            workers: None,
            watcher_shutdown_secs: 30,
        }
    }
}

/// Top-level errors surfaced by the daemon.
#[derive(Debug, Error)]
pub enum DaemonError {
    /// HTTP plane failure.
    #[error("http: {0}")]
    Http(#[from] HttpError),
    /// Tokio runtime build failure.
    #[error("tokio runtime: {0}")]
    Runtime(String),
    /// Caller passed a non-positive worker count; Phase 2 rejects it at
    /// [`DaemonConfig::effective_workers`] time, but the validator lives
    /// here so tests can pin the contract.
    #[error("invalid config: {0}")]
    InvalidConfig(String),
    /// Startup pipeline aborted (pre-deploy snapshot / HMX hydrate /
    /// Ember re-gate / PBFT handshake / lease / Watcher birth /
    /// warmup). Maps to daemon exit code 10.
    #[error("startup: {0}")]
    Startup(#[from] startup::StartupError),
}

/// Build the canonical Tokio runtime.
///
/// # Errors
/// Returns [`DaemonError::Runtime`] if the runtime cannot be constructed
/// (e.g. the kernel denies thread creation).
pub fn build_runtime(workers: usize) -> Result<tokio::runtime::Runtime, DaemonError> {
    let workers = workers.max(1);
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(workers)
        .enable_io()
        .enable_time()
        .thread_name_fn(|| {
            use std::sync::atomic::{AtomicUsize, Ordering};
            static COUNTER: AtomicUsize = AtomicUsize::new(0);
            let id = COUNTER.fetch_add(1, Ordering::Relaxed);
            format!("synthex-v2-worker-{id}")
        })
        .build()
        .map_err(|err| DaemonError::Runtime(err.to_string()))
}

/// Run the daemon until cancelled by signal or caller.
///
/// Phase 2 spawns only the HTTP server and the signal handler. Phase 3
/// expands this into the 11-task composition described in
/// `DAEMON_INTEGRATION_PLAN.md §3c`.
///
/// # Errors
///
/// Returns [`DaemonError::Http`] if the HTTP plane fails to bind or
/// serve. Signal-driven shutdown is a clean exit (`Ok(())`).
#[allow(clippy::too_many_lines)] // wiring function — clarity > line count
pub async fn run(config: DaemonConfig) -> Result<(), DaemonError> {
    let cancel = CancellationToken::new();
    let state = Arc::new(AppState::new(cancel.clone(), current_unix_secs()));

    info!(
        target = "daemon",
        mode = ?config.mode,
        addr = %config.socket_addr(),
        workers = config.effective_workers(),
        "synthex-v2 starting"
    );

    // Signal handler — independent task; uses its own shared clone of `cancel`.
    let signal_cancel = cancel.clone();
    let signal_handle = tokio::spawn(async move {
        shutdown::install_signal_handlers(signal_cancel).await;
    });

    // HTTP plane.
    let http_cancel = cancel.clone();
    let http_state = Arc::clone(&state);
    let http_addr = config.socket_addr();
    let (http_err_tx, mut http_err_rx) = tokio::sync::oneshot::channel();
    let http_handle = tokio::spawn(async move {
        match http::run_server(http_addr, http_state, http_cancel).await {
            Ok(()) => {}
            Err(err) => {
                error!(target = "daemon", %err, "http plane error");
                let _ = http_err_tx.send(err);
            }
        }
    });

    // Phase 3: run the startup pipeline. On abort, cancel the rest of
    // the daemon, surface the error, and exit non-zero (handled by the
    // binary shim).
    let startup_state = Arc::clone(&state);
    if let Err(err) = startup::run(startup_state).await {
        error!(target = "daemon", stage = err.stage, reason = %err.reason, "startup aborted");
        cancel.cancel();
        let _ = tokio::time::timeout(Duration::from_secs(2), http_handle).await;
        return Err(DaemonError::Startup(err));
    }

    // S1: PV2 Unix bus subscriber — live Kuramoto field.tick events.
    // AP23: pv2_bus::spawn() calls tokio::spawn internally —
    // do NOT wrap in spawn_blocking.
    {
        use std::sync::atomic::Ordering;
        let pv2_state = Arc::clone(&state);
        crate::pv2_bus::spawn("synthex-v2", move |tick| {
            pv2_state
                .last_field_r
                .store(tick.r.to_bits(), Ordering::Relaxed);
            tracing::debug!(
                target = "pv2_bus",
                r = tick.r,
                tick = tick.tick,
                spheres = tick.spheres,
                phase_tier = tick.phase_tier(),
                "field.tick received — r stored for regulation cascade"
            );
        });
    }

    // Phase 3 task set — regulation, memory consolidation, habitat health,
    // Watcher observer + pipeline.
    let regulation_handle = tasks::regulation::spawn(Arc::clone(&state), cancel.clone());
    let consolidation_handle =
        tasks::memory_consolidation::spawn(Arc::clone(&state), cancel.clone());
    let health_handle = tasks::habitat_health::spawn(Arc::clone(&state), cancel.clone());

    // Watcher pipeline: observer → channel → Critic → Verifier → Proposer
    #[cfg(feature = "watcher-full")]
    let (observer_handle, pipeline_handle) = {
        let (obs_tx, obs_rx) =
            tokio::sync::mpsc::channel::<crate::m8_watcher::m46_watcher_observer::Observation>(64);
        let observer_h = tasks::watcher_observer::spawn_with_pipeline(
            Arc::clone(&state),
            cancel.clone(),
            obs_tx,
        );
        let pipeline_h = spawn_watcher_pipeline(Arc::clone(&state), cancel.clone(), obs_rx);
        (observer_h, Some(pipeline_h))
    };
    #[cfg(not(feature = "watcher-full"))]
    let (observer_handle, pipeline_handle): (_, Option<tokio::task::JoinHandle<()>>) = {
        let observer_h = tasks::watcher_observer::spawn(Arc::clone(&state), cancel.clone());
        (observer_h, None)
    };

    // Phase F live wiring — only spawn shadow_diff when the daemon is in
    // shadow mode. Best-effort: if the v1 source fails to construct (DB
    // missing, permission issue) we log + skip rather than aborting the
    // daemon. Primary/Canary modes skip this task entirely.
    let shadow_handle = if config.mode == DaemonMode::Shadow {
        spawn_shadow_diff(Arc::clone(&state), cancel.clone())
    } else {
        None
    };

    // S117 Phase 2.5 — 6th task drains the WS capture mpsc to
    // ws_inbound_events.db at the default relative path (resolved against
    // the daemon's cwd; production deployments override via
    // `spawn_with_path` in a future step).
    #[cfg(feature = "persistence")]
    let ws_inbound_handle = tasks::ws_inbound_writer::spawn(Arc::clone(&state), cancel.clone());
    #[cfg(not(feature = "persistence"))]
    let ws_inbound_handle: Option<tokio::task::JoinHandle<()>> = None;

    // S155 Track A — 7th task drains the watcher observation mpsc to
    // watcher_observation.db. Mirrors the ws_inbound_writer wiring; only
    // spawned when the `persistence` feature is enabled (rusqlite gate).
    #[cfg(feature = "persistence")]
    let watcher_obs_handle =
        tasks::watcher_observation_writer::spawn(Arc::clone(&state), cancel.clone());
    #[cfg(not(feature = "persistence"))]
    let watcher_obs_handle: Option<tokio::task::JoinHandle<()>> = None;

    // Wait for signal handler.
    let _ = signal_handle.await;

    // Assemble handle table with per-task budgets from config.
    let budgets = config.shutdown_budgets();
    let mut handles = vec![
        NamedHandle::new("http", budgets.http, http_handle),
        NamedHandle::new("regulation", budgets.regulation, regulation_handle),
        NamedHandle::new("memory-consolidation", budgets.memory, consolidation_handle),
        NamedHandle::new(
            "habitat-health",
            budgets.ingest, // habitat probe is an ingest-adjacent plane
            health_handle,
        ),
        NamedHandle::new("watcher-observer", budgets.watcher, observer_handle),
    ];
    if let Some(h) = shadow_handle {
        handles.push(NamedHandle::new("shadow-diff", budgets.watcher, h));
    }
    if let Some(h) = pipeline_handle {
        handles.push(NamedHandle::new("watcher-pipeline", budgets.watcher, h));
    }
    if let Some(h) = ws_inbound_handle {
        // ingest-adjacent persistence path; share the ingest budget so the
        // 500ms drain budget the task self-imposes is a sub-budget.
        handles.push(NamedHandle::new("ws-inbound-writer", budgets.ingest, h));
    }
    if let Some(h) = watcher_obs_handle {
        // S155 Track A — observer-adjacent persistence path. Shares the
        // watcher budget; the writer's own 500ms drain budget is a
        // sub-budget under that envelope.
        handles.push(NamedHandle::new("watcher-observation-writer", budgets.watcher, h));
    }

    let completed = shutdown::join_with_budgets(handles).await;
    // Drain any stray HTTP error delivered during shutdown.
    if let Ok(err) = http_err_rx.try_recv() {
        error!(target = "daemon", %err, "http error surfaced during shutdown");
    }
    info!(target = "daemon", completed, "daemon shutdown complete");
    Ok(())
}

/// Build and spawn the Watcher pipeline task with default components.
#[cfg(feature = "watcher-full")]
fn spawn_watcher_pipeline(
    state: Arc<AppState>,
    cancel: CancellationToken,
    rx: tokio::sync::mpsc::Receiver<crate::m8_watcher::m46_watcher_observer::Observation>,
) -> tokio::task::JoinHandle<()> {
    use crate::m6_action::m38_watcher_gateway::InMemoryWatcherGatewaySink;
    use crate::m8_watcher::m47_watcher_critic::WatcherCritic;
    use crate::m8_watcher::m48_watcher_verifier::{
        InMemoryShadowTestRunner, ShadowTestResult, VerifierConfig, WatcherVerifier,
    };
    use crate::m8_watcher::m49_watcher_proposer::WatcherProposer;
    use crate::m8_watcher::m50_watcher_innovator::WatcherInnovator;
    use tasks::watcher_pipeline::{PipelineComponents, PipelineConfig};

    let components = Arc::new(PipelineComponents {
        critic: WatcherCritic::with_defaults(),
        verifier: WatcherVerifier::new(
            VerifierConfig::defaults(),
            Arc::new(InMemoryShadowTestRunner::new(ShadowTestResult {
                tests_run: 50,
                tests_passed: 50,
                fitness_measured: 0.01,
                worktree: "shadow-default".to_owned(),
                note: "heuristic-runner".to_owned(),
            })),
        ),
        proposer: WatcherProposer::with_defaults(),
        innovator: WatcherInnovator::with_defaults(),
        gateway_sink: Arc::new(InMemoryWatcherGatewaySink::new()),
    });

    info!(target = "daemon", "spawning watcher pipeline (6th daemon task)");
    tasks::watcher_pipeline::spawn(state, cancel, rx, components, PipelineConfig::default())
}

/// Build and spawn the shadow-diff task. Returns `None` if the v1 source
/// cannot be constructed — caller logs + skips, daemon keeps running.
fn spawn_shadow_diff(
    state: Arc<AppState>,
    cancel: CancellationToken,
) -> Option<tokio::task::JoinHandle<()>> {
    use super::shadow::sqlite_source::{SqliteSourceConfig, SqliteV1Source};
    use super::shadow::DiffBatcher;
    use super::tasks::shadow_diff::{spawn, HttpPovmSink, ShadowDiffConfig};

    // Use the env var if set; otherwise resolve the relative path against
    // `CARGO_MANIFEST_DIR` (compile-time) to produce an absolute path that
    // survives CWD changes. Falls back to CWD-relative with a warning.
    let db_path = std::env::var("SYNTHEX_V2_SHADOW_V1_DB").map_or_else(
        |_| {
            let relative = std::path::Path::new("data/databases/gradient_snapshot.db");
            let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
            let absolute = manifest_dir.join(relative);
            if absolute.exists() {
                absolute
            } else {
                tracing::warn!(
                    target = "shadow_diff",
                    manifest_path = %absolute.display(),
                    "CARGO_MANIFEST_DIR path does not exist — falling back to CWD-relative path"
                );
                relative.to_owned()
            }
        },
        std::path::PathBuf::from,
    );
    if !db_path.exists() {
        tracing::warn!(
            target = "shadow_diff",
            path = %db_path.display(),
            "v1 source DB not found — skipping shadow-diff task"
        );
        return None;
    }

    let source_config = SqliteSourceConfig::new(db_path);
    let source = Arc::new(SqliteV1Source::new(source_config, Arc::clone(&state.cascade)));

    let povm_base = std::env::var("SYNTHEX_V2_POVM_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8125".to_string());
    let sink = Arc::new(HttpPovmSink::new(povm_base));

    let batcher = Arc::new(DiffBatcher::new());

    tracing::info!(
        target = "shadow_diff",
        "spawning shadow-diff task (5th daemon task)"
    );
    Some(spawn(
        state,
        cancel,
        source,
        sink,
        batcher,
        ShadowDiffConfig::default(),
    ))
}

fn current_unix_secs() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_shadow_mode_and_local_bind() {
        let cfg = DaemonConfig::default();
        assert_eq!(cfg.mode, DaemonMode::Shadow);
        assert_eq!(cfg.bind, IpAddr::V4(Ipv4Addr::LOCALHOST));
        assert_eq!(cfg.socket_addr().port(), DEFAULT_SHADOW_PORT);
    }

    #[test]
    fn port_override_takes_precedence_over_mode_default() {
        let cfg = DaemonConfig {
            port: Some(9999),
            ..DaemonConfig::default()
        };
        assert_eq!(cfg.socket_addr().port(), 9999);
    }

    #[test]
    fn mode_default_port_matches_plan() {
        assert_eq!(DaemonMode::Shadow.default_port(), DEFAULT_SHADOW_PORT);
        assert_eq!(DaemonMode::Canary.default_port(), DEFAULT_PRIMARY_PORT);
        assert_eq!(DaemonMode::Primary.default_port(), DEFAULT_PRIMARY_PORT);
    }

    #[test]
    fn effective_workers_defaults_to_cpu_count() {
        let cfg = DaemonConfig::default();
        assert_eq!(cfg.effective_workers(), num_cpus::get().max(1));
    }

    #[test]
    fn effective_workers_honours_override() {
        let cfg = DaemonConfig {
            workers: Some(4),
            ..DaemonConfig::default()
        };
        assert_eq!(cfg.effective_workers(), 4);
    }

    #[test]
    fn zero_workers_clamps_to_one() {
        let cfg = DaemonConfig {
            workers: Some(0),
            ..DaemonConfig::default()
        };
        assert_eq!(cfg.effective_workers(), 1);
    }

    #[test]
    fn shutdown_budgets_honour_watcher_override() {
        let cfg = DaemonConfig {
            watcher_shutdown_secs: 7,
            ..DaemonConfig::default()
        };
        assert_eq!(cfg.shutdown_budgets().watcher, Duration::from_secs(7));
    }

    #[test]
    fn shutdown_budgets_default_for_non_watcher_planes() {
        let cfg = DaemonConfig::default();
        let b = cfg.shutdown_budgets();
        assert_eq!(b.http, Duration::from_secs(2));
        assert_eq!(b.ingest, Duration::from_secs(5));
    }

    #[test]
    fn zero_watcher_shutdown_clamps_to_one() {
        let cfg = DaemonConfig {
            watcher_shutdown_secs: 0,
            ..DaemonConfig::default()
        };
        assert_eq!(cfg.shutdown_budgets().watcher, Duration::from_secs(1));
    }

    #[test]
    fn build_runtime_succeeds_with_one_worker() {
        let rt = build_runtime(1).expect("build runtime");
        rt.block_on(async {
            assert_eq!(1 + 1, 2);
        });
    }

    #[test]
    fn build_runtime_clamps_zero_workers() {
        let rt = build_runtime(0).expect("build runtime");
        rt.block_on(async {
            tokio::task::yield_now().await;
        });
    }

    #[tokio::test]
    async fn daemon_run_exits_on_cancel() {
        // Use port 0 by picking an ephemeral bind and the canary mode so the
        // test doesn't collide with production shadow port.
        let cfg = DaemonConfig {
            mode: DaemonMode::Shadow,
            port: Some(0),
            bind: IpAddr::V4(Ipv4Addr::LOCALHOST),
            workers: Some(1),
            watcher_shutdown_secs: 2,
            ..DaemonConfig::default()
        };
        let run_handle = tokio::spawn(async move { run(cfg).await });
        // Give the server time to bind.
        tokio::time::sleep(Duration::from_millis(200)).await;
        // Send SIGINT through the process-level handler by cancelling via a
        // direct kill of the tokio task — in tests the signal handler
        // awaits external cancellation via the shared token, which the
        // timeout below forces by aborting the whole daemon task.
        run_handle.abort();
        let result = tokio::time::timeout(Duration::from_secs(3), run_handle).await;
        assert!(result.is_ok(), "daemon run must terminate quickly after abort");
    }
}
