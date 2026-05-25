//! `wf-daemon` — workflow-trace habitat-service shape (Wave-16, S1005032).
//!
//! The minimal HTTP daemon that makes workflow-trace a first-class managed
//! habitat service. Exposes `GET /health` on port 8142 (the
//! `habitat-nerve-center` + `habitat-plugin.wasm` probe shape) and spawns
//! the existing `wf-poller` tick loop as an internal `tokio::task` so the
//! WFE→SX2 heartbeat wire (m16 `DriftDetector`, W1 `HeartbeatTransport`,
//! V5 `SubstrateTrust`) runs under `devenv start`'s auto-start and
//! auto-restart envelope.
//!
//! # Operational shape
//!
//! - **Port:** 8142 (verified free 2026-05-25; sits adjacent to 8140 Inj)
//!
//! - **Health:** `GET /health` → `{"status":"ok"}` 200 (matches the
//!   `bridge_health` probe at `:8142/health`, NOT `:8142/api/health` — only
//!   ME at 8180 uses the `/api/` prefix in this habitat).
//! - **Poller cadence:** 1 Hz (DD-3 §4.1, same as wf-poller standalone)
//! - **Substrate endpoint:** `http://127.0.0.1:8092/v3/heartbeat` (SX2)
//! - **Env overrides:** `WF_DAEMON_PORT` + `WF_POLLER_ENDPOINT` +
//!   `WF_POLLER_INTERVAL_MS`
//!
//! # Read-only contract
//!
//! Same as `wf-poller` standalone (Wave-15): tracing-only emits, V5
//! gate enforced, no writes back to ORAC/scheduler/m11. The daemon
//! adds only the HTTP `/health` surface (no operational HTTP — see
//! `ai_docs/decisions/` for the future `/crystallise` + `/dispatch` REST
//! discussion).
//!
//! # Source/deploy drift awareness (Zen ZA-2 / AP-V7-13)
//!
//! `/health` returns 200 regardless of SX2 reachability. Substrate
//! reachability is observable via the tracing log
//! (`outcome=ok|substrate_unreachable|engine_imagined`). This intentional
//! split keeps the habitat-plugin grid `WFE` indicator green when the
//! daemon itself is alive, even if the wire downstream is partially
//! drifted — operators read the poller tick log for wire-level health.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic, clippy::doc_markdown)]

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use axum::routing::get;
use axum::Router;
use tokio::net::TcpListener;
use tokio::task;

use workflow_core::m16_substrate_drift_canary::transport::{
    tick_and_emit, HeartbeatTransport,
};
use workflow_core::m16_substrate_drift_canary::{
    AlertBudget, ClockSample, ClockSampler, ClockSource, DriftDetector, SkewEnvelope,
};
use workflow_core::refusal_token::{RefusalToken, SubstrateId, UnavailableReason};
use workflow_core::substrate_trust::{
    SubstrateParticipationStatus, SubstrateTrust, TrustEntry, TrustValue,
};

/// Binary version (auto-bumped via `Cargo.toml`).
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default port for the `/health` endpoint. Port 8142 in the habitat
/// allocation (port 8141 was first attempted but is RESERVED for
/// HABITAT-CONDUCTOR — only currently free because Conductor has
/// `auto_start=false`; the survey miss is documented in vault note
/// "Wave-16 — Port 8142 Re-port S1005032" for future scaffold-mastery
/// false-positive avoidance). 8142 verified free against
/// devenv.toml + all ai_docs/ai_specs + bridge_health.rs.
const DEFAULT_PORT: u16 = 8142;

/// Default tick cadence — 1 Hz per m16 spec DD-3 §4.1.
const DEFAULT_TICK_INTERVAL: Duration = Duration::from_secs(1);

/// Default synthex-v2 heartbeat endpoint.
const DEFAULT_ENDPOINT: &str = "http://127.0.0.1:8092/v3/heartbeat";

/// Wall-clock-only `ClockSampler`. Same shape as `wf-poller` standalone;
/// proves the pair-skew computation path. Real production samplers would
/// query atuin / stcortex / `injection.db`.
struct WallClockSampler {
    source: ClockSource,
}

impl ClockSampler for WallClockSampler {
    fn source(&self) -> ClockSource {
        self.source
    }
    fn sample(&self) -> ClockSample {
        let now_ms = unix_ms_now();
        ClockSample {
            source: self.source,
            clock_value_ms: now_ms,
            observed_at_ms: now_ms,
        }
    }
}

/// Unix wall-clock milliseconds since epoch.
fn unix_ms_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
}

/// Process-boot identifier. Carried in every heartbeat envelope so
/// substrate can detect WFE-restart events.
fn boot_id_for_this_process() -> String {
    let unix_ms = unix_ms_now();
    format!("wf-daemon-{unix_ms}")
}

/// `GET /health` handler. Returns a static JSON body so the
/// `habitat-nerve-center` + Zellij `habitat-plugin.wasm` probe sees the
/// same canonical shape every other habitat service emits.
async fn health() -> &'static str {
    r#"{"status":"ok","service":"workflow-trace","port":8142}"#
}

/// V5 substrate-trust seed for synthex-v2 at boot. Same as `wf-poller`
/// standalone: assume `Live` so the transport actually attempts the wire;
/// the V5 gate will downgrade honestly via `RefusalToken` sub-tags on
/// substrate refusal.
fn initial_trust() -> Arc<SubstrateTrust> {
    let mut trust = SubstrateTrust::new();
    let _prev = trust.set(
        SubstrateId::SynthexV2,
        TrustEntry {
            status: SubstrateParticipationStatus::Live,
            value: TrustValue::Score(0.9),
        },
    );
    Arc::new(trust)
}

/// Per-tick counters logged on each emit for observability.
#[derive(Default)]
struct TickCounters {
    ok: u64,
    refusals: u64,
    unreachable: u64,
}

/// Read env overrides for the poller subsystem. Daemon-side env vars
/// (`WF_DAEMON_PORT`) are parsed in `main` directly; poller env vars
/// (`WF_POLLER_ENDPOINT`, `WF_POLLER_INTERVAL_MS`) are parsed here.
fn read_poller_config() -> (String, Duration) {
    let endpoint = std::env::var("WF_POLLER_ENDPOINT")
        .unwrap_or_else(|_| DEFAULT_ENDPOINT.to_owned());
    let interval = std::env::var("WF_POLLER_INTERVAL_MS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map_or(DEFAULT_TICK_INTERVAL, Duration::from_millis);
    (endpoint, interval)
}

/// Run one poller tick. Same emit-and-log shape as `wf-poller`
/// standalone; routes through tracing only — no writes back.
fn run_one_tick(
    cycle: u64,
    detector: &mut DriftDetector,
    transport: &HeartbeatTransport,
    boot_id: &str,
    instance_id: &str,
    counters: &mut TickCounters,
) {
    let now_ms = unix_ms_now();
    let result = tick_and_emit(
        detector,
        now_ms,
        transport,
        boot_id.to_owned(),
        instance_id.to_owned(),
        1,
    );

    match result {
        Ok(ack) => {
            counters.ok += 1;
            tracing::info!(
                kind_preview = "wf_daemon_tick",
                cycle,
                outcome = "ok",
                cycle_acked = ack.cycle_acked,
                synthex_v2_observed_at_ms = ack.synthex_v2_observed_at_ms,
                ok_count = counters.ok,
                refusal_count = counters.refusals,
                unreachable_count = counters.unreachable,
                "heartbeat accepted by substrate"
            );
        }
        Err(RefusalToken::Unavailable(UnavailableReason::EngineImagined {
            reason, ..
        })) => {
            counters.refusals += 1;
            tracing::warn!(
                kind_preview = "wf_daemon_tick",
                cycle,
                outcome = "engine_imagined",
                reason = reason.as_str(),
                refusal_count = counters.refusals,
                "V5 gate short-circuited (substrate not shipped); not reaching wire"
            );
        }
        Err(RefusalToken::Unavailable(UnavailableReason::SubstrateUnreachable {
            transport_reason,
            ..
        })) => {
            counters.unreachable += 1;
            tracing::warn!(
                kind_preview = "wf_daemon_tick",
                cycle,
                outcome = "substrate_unreachable",
                transport_reason = transport_reason.as_str(),
                unreachable_count = counters.unreachable,
                "substrate unreachable (transport / phase3 pending / r13)"
            );
        }
        Err(RefusalToken::Unavailable(UnavailableReason::SubstrateAuthored {
            substrate_reason,
            ..
        })) => {
            counters.refusals += 1;
            tracing::warn!(
                kind_preview = "wf_daemon_tick",
                cycle,
                outcome = "substrate_authored_refusal",
                substrate_reason = substrate_reason.as_str(),
                refusal_count = counters.refusals,
                "substrate explicitly refused heartbeat"
            );
        }
        Err(other) => {
            counters.refusals += 1;
            tracing::warn!(
                kind_preview = "wf_daemon_tick",
                cycle,
                outcome = "unexpected_refusal",
                detail = ?other,
                refusal_count = counters.refusals,
                "unexpected RefusalToken variant"
            );
        }
    }
}

/// Poller subsystem run as a `tokio::task::spawn_blocking` because
/// `HeartbeatTransport` uses `reqwest::blocking` (per WFE Cargo.toml).
/// Spawning blocking keeps the async tokio runtime free for axum.
///
/// `endpoint` + `instance_id` are taken by value because the function
/// runs inside a `spawn_blocking` move-closure: caller-side ownership
/// transfer is the natural shape (clippy `needless_pass_by_value`
/// suppressed; both values are conceptually consumed even though only
/// `endpoint` is moved into `HeartbeatTransport`).
#[allow(clippy::needless_pass_by_value)]
fn poller_subsystem(
    endpoint: String,
    interval_ms: Duration,
    instance_id: String,
) {
    let boot_id = boot_id_for_this_process();
    let substrate_trust = initial_trust();
    let endpoint_for_log = endpoint.clone();
    let transport = HeartbeatTransport::new(endpoint, Arc::clone(&substrate_trust));

    let samplers: Vec<Box<dyn ClockSampler>> = vec![
        Box::new(WallClockSampler {
            source: ClockSource::AtuinCheckpoint,
        }),
        Box::new(WallClockSampler {
            source: ClockSource::M11Recency,
        }),
    ];
    let mut detector = DriftDetector::new(
        samplers,
        SkewEnvelope { max_skew_ms: 1000 },
        AlertBudget::new(60_000),
    );

    let mut cycle: u64 = 0;
    let mut counters = TickCounters::default();

    tracing::info!(
        kind_preview = "wf_daemon_poller_boot",
        version = VERSION,
        endpoint = endpoint_for_log.as_str(),
        interval_ms = u64::try_from(interval_ms.as_millis()).unwrap_or(u64::MAX),
        boot_id = boot_id.as_str(),
        instance_id = instance_id.as_str(),
        "poller subsystem starting"
    );

    loop {
        run_one_tick(
            cycle,
            &mut detector,
            &transport,
            boot_id.as_str(),
            instance_id.as_str(),
            &mut counters,
        );
        cycle += 1;
        std::thread::sleep(interval_ms);
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(tracing::Level::INFO)
        .init();

    let port: u16 = std::env::var("WF_DAEMON_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_PORT);

    let (endpoint, interval_ms) = read_poller_config();
    let instance_id = std::env::var("WF_POLLER_INSTANCE")
        .unwrap_or_else(|_| "wf-daemon-default".to_owned());

    tracing::info!(
        kind_preview = "wf_daemon_boot",
        version = VERSION,
        port,
        endpoint = endpoint.as_str(),
        interval_ms = u64::try_from(interval_ms.as_millis()).unwrap_or(u64::MAX),
        instance_id = instance_id.as_str(),
        "wf-daemon starting; habitat-service shape for workflow-trace"
    );

    // Spawn poller in a blocking task (sync HeartbeatTransport).
    let _poller_handle = task::spawn_blocking(move || {
        poller_subsystem(endpoint, interval_ms, instance_id);
    });

    // Build axum router with /health only (minimal habitat-service surface).
    let app = Router::new().route("/health", get(health));

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
    let listener = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!(
                kind_preview = "wf_daemon_bind_fail",
                port,
                error = %e,
                "failed to bind /health endpoint; aborting"
            );
            return ExitCode::FAILURE;
        }
    };

    tracing::info!(
        kind_preview = "wf_daemon_ready",
        port,
        endpoint = "/health",
        "wf-daemon /health endpoint ready"
    );

    if let Err(e) = axum::serve(listener, app).await {
        tracing::error!(
            kind_preview = "wf_daemon_serve_fail",
            error = %e,
            "axum::serve returned error"
        );
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
