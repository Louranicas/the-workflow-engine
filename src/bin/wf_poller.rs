//! `wf-poller` — workflow-trace continuous-emit poller binary.
//!
//! **The actual wire** between workflow-trace and synthex-v2. Per Luke's
//! 2026-05-25 directive ("ensure the workflow-engine is fully and
//! comprehensively wired into synthex"), this binary constructs the
//! library pieces (m16 `DriftDetector` + W1 `HeartbeatTransport` + V5
//! `SubstrateTrust`) and runs the per-cycle tick loop that emits real
//! traffic to synthex-v2's `:8092/v3/heartbeat` endpoint.
//!
//! Until this binary lands, the m16 / W1 / V5 library code shipped in
//! v0.2.0 and v0.2.2+ is dead code: no production caller exists. After
//! this binary lands, m16 heartbeats actually flow.
//!
//! # Read-only contract (mirrors LCM chunk-3a discipline)
//!
//! - No writes back to ORAC / scheduler / m11 decay
//! - Tracing-only at each tick (`kind_preview="wf_poller_tick"` per the
//!   Wave-9 Gate A operator-loud convention)
//! - V5 substrate-trust gate enforced: if `SubstrateId::SynthexV2 ==
//!   NotShipped`, transport short-circuits to `EngineImagined` refusal
//!   without HTTP call (no fabricated `SubstrateAuthored` noise)
//!
//! # Termination semantics
//!
//! Run until SIGTERM / SIGKILL. The NA-8' "Goodbye" envelope on graceful
//! shutdown is honestly deferred — synthex-v2 detects WFE silence via
//! the missed-heartbeat path (planned but unwired today; honest
//! residual). Operator may run `kill -SIGTERM <pid>` or just `kill <pid>`
//! to stop the poller; substrate inferences after that point are
//! engine-imagined per the V5 contract.
//!
//! # Source/deploy drift awareness (Zen ZA-2 + NA-4 cond 0)
//!
//! The receiver endpoint on synthex-v2 may not be deployed even if its
//! source exists (the running daemon predates the source HEAD by 19+
//! days as of 2026-05-25). This binary is forward-compatible: every
//! emit routes through `RefusalToken::Unavailable` honestly until the
//! substrate endpoint ships AND the operator redeploys. Operator runs
//! the NA-4 acceptance cond 0 drift check (per Plan v2 §S2) before
//! relying on this binary's output for substrate-state inferences.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic, clippy::doc_markdown)]

use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use workflow_core::m16_substrate_drift_canary::{
    AlertBudget, ClockSample, ClockSampler, ClockSource, DriftDetector, SkewEnvelope,
};
use workflow_core::m16_substrate_drift_canary::transport::{
    tick_and_emit, HeartbeatTransport,
};
use workflow_core::refusal_token::{RefusalToken, SubstrateId, UnavailableReason};
use workflow_core::substrate_trust::{
    SubstrateParticipationStatus, SubstrateTrust, TrustEntry, TrustValue,
};

/// Binary version (auto-bumped via Cargo.toml).
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default tick cadence — 1 Hz per m16 spec (DD-3 §4.1 cadence parity
/// with LCM chunk-3a thermal poller).
const DEFAULT_TICK_INTERVAL: Duration = Duration::from_secs(1);

/// Default synthex-v2 heartbeat endpoint (matches LCM W1 default).
const DEFAULT_ENDPOINT: &str = "http://127.0.0.1:8092/v3/heartbeat";

/// Default V5 substrate-trust mode for synthex-v2 at boot. Operator runs
/// `wf-poller` AFTER a successful source/deploy drift check (Plan v2
/// NA-4 cond 0); the binary will downgrade to honest refusals via
/// `RefusalToken` sub-tags if the endpoint refuses.
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

/// Process-boot identifier. Generated once at binary start; carried in
/// every heartbeat envelope so substrate can detect WFE-restart events.
/// Pure timestamp-based — no `uuid` dep added; resolution = ms.
fn boot_id_for_this_process() -> String {
    let unix_ms = unix_ms_now();
    format!("wf-poller-{unix_ms}")
}

/// Unix wall-clock milliseconds since epoch. Returns `0` on
/// `SystemTime` errors (impossible on any sane host).
fn unix_ms_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
}

/// Minimal `ClockSampler` for the wf-poller's drift-detection set. Just
/// reads wall-clock as the "atuin checkpoint" surrogate clock — real
/// production samplers would query atuin / stcortex / injection.db
/// directly, but this binary's job is to PROVE the wire, not to source
/// authoritative clock signal.
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

fn read_env_config() -> (String, Duration, Option<u64>, String) {
    let endpoint = std::env::var("WF_POLLER_ENDPOINT")
        .unwrap_or_else(|_| DEFAULT_ENDPOINT.to_owned());
    let interval = std::env::var("WF_POLLER_INTERVAL_MS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map_or(DEFAULT_TICK_INTERVAL, Duration::from_millis);
    let max_cycles = std::env::var("WF_POLLER_MAX_CYCLES")
        .ok()
        .and_then(|s| s.parse::<u64>().ok());
    let instance_id = std::env::var("WF_POLLER_INSTANCE")
        .unwrap_or_else(|_| "wf-poller-default".to_owned());
    (endpoint, interval, max_cycles, instance_id)
}

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
                kind_preview = "wf_poller_tick",
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
        Err(RefusalToken::Unavailable(UnavailableReason::EngineImagined { reason, .. })) => {
            counters.refusals += 1;
            tracing::warn!(
                kind_preview = "wf_poller_tick",
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
                kind_preview = "wf_poller_tick",
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
                kind_preview = "wf_poller_tick",
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
                kind_preview = "wf_poller_tick",
                cycle,
                outcome = "unexpected_refusal",
                detail = ?other,
                refusal_count = counters.refusals,
                "unexpected RefusalToken variant"
            );
        }
    }
}

#[derive(Default)]
struct TickCounters {
    ok: u64,
    refusals: u64,
    unreachable: u64,
}

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(tracing::Level::INFO)
        .init();

    let (endpoint, interval_ms, max_cycles, instance_id) = read_env_config();
    let boot_id = boot_id_for_this_process();

    tracing::info!(
        kind_preview = "wf_poller_boot",
        version = VERSION,
        endpoint = endpoint.as_str(),
        interval_ms = u64::try_from(interval_ms.as_millis()).unwrap_or(u64::MAX),
        boot_id = boot_id.as_str(),
        instance_id = instance_id.as_str(),
        max_cycles = max_cycles.unwrap_or(0),
        "wf-poller starting; the actual wire between workflow-trace and synthex-v2"
    );

    let substrate_trust = initial_trust();
    let transport = HeartbeatTransport::new(endpoint.clone(), Arc::clone(&substrate_trust));

    // Detector with 2 wall-clock samplers (proves the pair-skew computation
    // path; real production deployment would feed atuin + stcortex +
    // injection.db samplers, but those are operator-config-driven and out
    // of scope for this MVP binary).
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

    loop {
        if let Some(max) = max_cycles {
            if cycle >= max {
                break;
            }
        }
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

    tracing::info!(
        kind_preview = "wf_poller_shutdown",
        total_cycles = cycle,
        ok_count = counters.ok,
        refusal_count = counters.refusals,
        unreachable_count = counters.unreachable,
        "wf-poller shutting down (max_cycles reached)"
    );

    ExitCode::SUCCESS
}
