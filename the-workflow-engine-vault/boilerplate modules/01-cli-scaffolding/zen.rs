//! `zen` — Habitat divergence detection daemon.
//!
//! Polls service endpoints every 60 s and evaluates divergence detection rules.
//! Probes: PV2 `/spheres` (sphere ghost detection), PV2 `/field` (tunnel
//! overload), injection.db (injection cache staleness), ORAC `/bridges`
//! (persistent open breakers). Detected divergences are written to
//! `state.db.divergence_reports` directly and broadcast to Atuin KV for
//! quick lookup.
//!
//! # Usage
//!
//! ```text
//! zen [OPTIONS]
//!
//! Options:
//!   --weaver-url <URL>   Weaver base URL [default: http://localhost:8141]
//!   --db <PATH>          Direct state.db path (fallback if Weaver unreachable)
//!   --interval <SECS>    Poll interval in seconds [default: 60]
//!   -v, --version
//!   -h, --help
//! ```

use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use anyhow::Context as _;
use clap::Parser;
use habitat_conductor::{
    divergence::{
        BreakerPersistentOpenRule, DivergenceEvent, InjectionCacheStaleRule,
        Rule, RuleRegistry, SphereGhostRule, TunnelOverloadRule,
    },
    state::{DivergenceReport, Severity, StateDb, default_db_path},
};
use parking_lot::Mutex;
use serde_json::json;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

// ---------------------------------------------------------------------------
// Breaker probe error type (F-CONDUCTOR-01 + F-CONDUCTOR-02)
// ---------------------------------------------------------------------------

/// Errors from the ORAC breaker probe.
///
/// Distinguishes transport failure (ORAC unreachable) from JSON parse failure
/// (schema drift, unexpected response shape) so operators and counters can
/// identify the root cause rather than treating both as "no breakers open".
#[derive(Debug)]
enum BreakerProbeError {
    /// HTTP / network failure (connection refused, timeout, TLS error).
    Transport(String),
    /// JSON deserialisation failure (truncated body, schema drift).
    Parse(String),
    /// The response body was not a JSON array.
    SchemaDrift(String),
}

impl std::fmt::Display for BreakerProbeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transport(e) => write!(f, "transport: {e}"),
            Self::Parse(e) => write!(f, "parse: {e}"),
            Self::SchemaDrift(e) => write!(f, "schema_drift: {e}"),
        }
    }
}

/// Cumulative count of breaker probe failures since daemon start.
///
/// Exposed so the next snapshot cycle can surface the count for observability.
/// Reset on daemon restart.
static BREAKER_PROBE_FAILURES: AtomicU64 = AtomicU64::new(0);

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

/// zen — Habitat divergence detection daemon.
#[derive(Debug, Parser)]
#[command(name = "zen", version, about)]
struct Args {
    /// Weaver base URL (used for probe data).
    #[arg(long, default_value = "http://localhost:8141")]
    weaver_url: String,

    /// Direct state.db path (fallback if Weaver is unreachable).
    #[arg(long)]
    db: Option<PathBuf>,

    /// Polling interval in seconds.
    #[arg(long, default_value = "60")]
    interval: u64,
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
#[allow(clippy::too_many_lines)] // S1001791: AP29 fix added spawn_blocking wraps; the loop body is one coherent unit
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .json()
        .init();

    info!(interval_secs = args.interval, "zen starting");

    // Open state.db for direct divergence writes (Weaver API write endpoint
    // is deferred to Wave 3; Zen writes directly to the shared db).
    let db_path = match args.db {
        Some(p) => p,
        None => default_db_path().context("cannot determine state.db path")?,
    };
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating db directory: {}", parent.display()))?;
    }
    let mut db = StateDb::open(&db_path)
        .with_context(|| format!("opening state.db at {}", db_path.display()))?;
    db.migrate().context("applying migrations")?;
    let db = Arc::new(Mutex::new(db));

    info!(?db_path, "state.db ready");

    // Instantiate detection rules.
    let sphere_ghost_rule = SphereGhostRule::default();
    let tunnel_rule = TunnelOverloadRule::default();
    let injection_rule = InjectionCacheStaleRule::default();
    let breaker_rule = BreakerPersistentOpenRule::default();
    let mut registry = RuleRegistry::new();

    let mut interval = tokio::time::interval(Duration::from_secs(args.interval));
    interval.tick().await; // skip first immediate tick

    info!("zen daemon loop starting");

    loop {
        interval.tick().await;

        // AP29 fix (S1001791): all sync I/O (ureq + rusqlite) routed through
        // spawn_blocking to avoid starving the Tokio runtime. Mirrors the
        // hardened pattern at habitat-conductor::snapshot::probe_all and the
        // ORAC fix in commit aad5af9.
        //
        // F-CONDUCTOR-07: the prior `probe_weaver` call was dead code — its
        // result was bound to `_snap` and immediately discarded. Removed to
        // eliminate the wasted HTTP round-trip per cycle and the misleading
        // "polls weaver state" claim in the doc comment.

        let now_utc = chrono::Utc::now();
        let ts = now_utc.timestamp_millis();
        let mut all_events: Vec<DivergenceEvent> = Vec::new();

        // Rule 1: sphere_ghost — requires sphere list from PV2.
        // Probe PV2 directly for sphere data (Weaver aggregates only scalars).
        if registry.may_fire("sphere_ghost", sphere_ghost_rule.cooldown_secs(), now_utc) {
            let spheres = tokio::task::spawn_blocking(probe_pv2_spheres)
                .await
                .unwrap_or_else(|e| {
                    warn!("pv2 spheres probe join error: {e}");
                    vec![]
                });
            let events = sphere_ghost_rule.detect(&spheres);
            if !events.is_empty() {
                registry.record_fired("sphere_ghost", now_utc);
                all_events.extend(events);
            }
        }

        // Rule 2: tunnel_overload.
        if registry.may_fire("tunnel_overload", tunnel_rule.cooldown_secs(), now_utc) {
            let weaver_url = args.weaver_url.clone();
            let stats = tokio::task::spawn_blocking(move || probe_tunnel_stats(&weaver_url))
                .await
                .unwrap_or_else(|e| {
                    warn!("tunnel probe join error: {e}");
                    None
                });
            if let Some((total, saturated)) = stats {
                let events = tunnel_rule.detect(total, saturated);
                if !events.is_empty() {
                    registry.record_fired("tunnel_overload", now_utc);
                    all_events.extend(events);
                }
            }
        }

        // Rule 3: injection_cache_stale.
        if registry.may_fire("injection_cache_stale", injection_rule.cooldown_secs(), now_utc) {
            let stale = tokio::task::spawn_blocking(probe_injection_staleness)
                .await
                .unwrap_or_else(|e| {
                    warn!("injection staleness probe join error: {e}");
                    None
                });
            if let Some((current, injected)) = stale {
                let events = injection_rule.detect(current, injected);
                if !events.is_empty() {
                    registry.record_fired("injection_cache_stale", now_utc);
                    all_events.extend(events);
                }
            }
        }

        // Rule 4: breaker_persistent_open.
        if registry.may_fire("breaker_persistent_open", breaker_rule.cooldown_secs(), now_utc) {
            let weaver_url = args.weaver_url.clone();
            let probe_result = tokio::task::spawn_blocking(move || probe_breaker_durations(&weaver_url))
                .await;
            // F-CONDUCTOR-01 + F-CONDUCTOR-02: match on error variant to log
            // the specific failure and increment the probe failure counter.
            let breakers = match probe_result {
                Ok(Ok(list)) => list,
                Ok(Err(BreakerProbeError::Transport(ref e))) => {
                    warn!(error = %e, "breaker probe transport failure — divergence detection blind this cycle");
                    BREAKER_PROBE_FAILURES.fetch_add(1, Ordering::Relaxed);
                    vec![]
                }
                Ok(Err(BreakerProbeError::Parse(ref e))) => {
                    warn!(error = %e, "breaker probe JSON parse failure — possible ORAC schema drift");
                    BREAKER_PROBE_FAILURES.fetch_add(1, Ordering::Relaxed);
                    vec![]
                }
                Ok(Err(BreakerProbeError::SchemaDrift(ref e))) => {
                    warn!(error = %e, "breaker probe schema drift — ORAC response is not a JSON array");
                    BREAKER_PROBE_FAILURES.fetch_add(1, Ordering::Relaxed);
                    vec![]
                }
                Err(e) => {
                    warn!("breaker probe spawn_blocking join error: {e}");
                    BREAKER_PROBE_FAILURES.fetch_add(1, Ordering::Relaxed);
                    vec![]
                }
            };
            if !breakers.is_empty() {
                let breaker_refs: Vec<(&str, u64)> = breakers
                    .iter()
                    .map(|(s, d)| (s.as_str(), *d))
                    .collect();
                let events = breaker_rule.detect(&breaker_refs);
                if !events.is_empty() {
                    registry.record_fired("breaker_persistent_open", now_utc);
                    all_events.extend(events);
                }
            }
        }

        // Dedup by kind (one report per kind per cycle).
        let all_events = RuleRegistry::dedup_by_kind(all_events);

        // Persist to state.db (rusqlite is sync — wrap in spawn_blocking).
        for event in &all_events {
            let report = DivergenceReport {
                id: None,
                ts,
                source: event.source.clone(),
                plan_ref: event.plan_ref.clone(),
                observed_ref: event.observed_ref.clone(),
                severity: event.severity,
                kind: event.kind.clone(),
                body: event.body.clone(),
            };
            let db_clone = Arc::clone(&db);
            let report_clone = report.clone();
            let kind = event.kind.clone();
            let severity = event.severity;
            match tokio::task::spawn_blocking(move || db_clone.lock().insert_divergence(&report_clone))
                .await
            {
                Ok(Ok(id)) => info!(id, kind = %kind, severity = %severity, "divergence persisted"),
                Ok(Err(e)) => warn!(kind = %kind, "failed to persist divergence: {e}"),
                Err(e) => warn!(kind = %kind, "divergence persist join error: {e}"),
            }
        }

        // Broadcast HIGH/CRITICAL via Atuin KV.
        for event in RuleRegistry::filter_by_severity(all_events, Severity::High) {
            broadcast_atuin_kv(&event);
        }

        info!(
            "zen cycle complete",
        );
    }
}

// ---------------------------------------------------------------------------
// Probe helpers (all gracefully degrade on failure)
// ---------------------------------------------------------------------------

fn probe_pv2_spheres() -> Vec<habitat_conductor::divergence::SphereInfo> {
    // Probe PV2 /spheres; degrade to empty list on failure.
    match ureq::get("http://localhost:8132/spheres")
        .timeout(Duration::from_secs(2))
        .call()
    {
        Ok(resp) => {
            let json: serde_json::Value = match serde_json::from_reader(resp.into_reader()) {
                Ok(j) => j,
                Err(e) => {
                    warn!("pv2 /spheres parse failed: {e}");
                    return vec![];
                }
            };
            parse_sphere_list(&json)
        }
        Err(e) => {
            warn!("pv2 /spheres probe failed: {e}");
            vec![]
        }
    }
}

fn parse_sphere_list(json: &serde_json::Value) -> Vec<habitat_conductor::divergence::SphereInfo> {
    let Some(arr) = json.as_array() else {
        return vec![];
    };
    arr.iter()
        .filter_map(|item| {
            Some(habitat_conductor::divergence::SphereInfo {
                sphere_id: item["sphere_id"].as_str()?.to_owned(),
                persona: item["persona"].as_str().unwrap_or("unknown").to_owned(),
                freq: item["freq"].as_f64().unwrap_or(0.0),
                phase: item["phase"].as_f64().unwrap_or(0.0),
                buoy_labels: item["buoy_labels"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(str::to_owned))
                            .collect()
                    })
                    .unwrap_or_default(),
                has_worked: item["has_worked"].as_bool().unwrap_or(false),
                memories: item["memories"].as_i64().unwrap_or(0),
            })
        })
        .collect()
}

fn probe_tunnel_stats(_weaver_url: &str) -> Option<(usize, usize)> {
    // Probe PV2 /field for tunnel statistics.
    const URL: &str = "http://localhost:8132/field";
    match ureq::get(URL)
        .timeout(Duration::from_secs(2))
        .call()
    {
        Ok(resp) => {
            // F-CONDUCTOR-08: distinguish transport failure from parse/schema failure.
            // Previously `.ok()?` silently swallowed parse errors — operators could
            // not tell "PV2 down" from "PV2 returned malformed JSON".
            let json: serde_json::Value = match serde_json::from_reader(resp.into_reader()) {
                Ok(v) => v,
                Err(e) => {
                    warn!(url = URL, "tunnel stats /field parse failed: {e}");
                    return None;
                }
            };
            let Some(total) = json["total_tunnels"].as_u64().and_then(|n| usize::try_from(n).ok()) else {
                warn!(url = URL, "tunnel stats missing total_tunnels field");
                return None;
            };
            let saturated = usize::try_from(json["saturated_tunnels"].as_u64().unwrap_or(0))
                .unwrap_or(0);
            Some((total, saturated))
        }
        Err(e) => {
            warn!(url = URL, "tunnel stats probe failed: {e}");
            None
        }
    }
}

fn probe_injection_staleness() -> Option<(u64, u64)> {
    // Read injection.db to compare current vs injected session numbers.
    //
    // F-CONDUCTOR-09: previously used `.unwrap_or(0u64)` for MAX/MIN column
    // reads, which collapses NULL (empty table) and 0 (valid row with
    // session_id=0) into the same value. InjectionCacheStaleRule then sees
    // drift=0 and never fires — a wiped injection.db looks identical to a
    // perfectly-synced one.
    //
    // Fix: use `Option<u64>` column types. If either column is NULL (table
    // empty or all rows resolved), return None — the rule does not fire for
    // "no unresolved sessions" (legitimate empty). If the query itself fails
    // (schema drift, missing table, FK violation), log a warning so operators
    // can distinguish "DB broken" from "empty table".
    let db_path = dirs::home_dir()?
        .join(".local")
        .join("share")
        .join("habitat")
        .join("injection.db");
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => {
            warn!(path = %db_path.display(), "injection.db open failed: {e}");
            return None;
        }
    };
    let result = conn.query_row(
        "SELECT MAX(session_id), MIN(session_id) FROM causal_chain \
         WHERE resolved_session IS NULL",
        [],
        |row| {
            let cur: Option<u64> = row.get(0)?;
            let inj: Option<u64> = row.get(1)?;
            Ok((cur, inj))
        },
    );
    match result {
        Ok((Some(current), Some(injected))) => Some((current, injected)),
        Ok((None, _) | (_, None)) => {
            // Table is empty or all rows resolved — legitimate "no unresolved
            // sessions". Do not fire the staleness rule.
            None
        }
        Err(e) => {
            warn!("injection_staleness probe failed — DB may be broken: {e}");
            None
        }
    }
}

/// Probes ORAC `/bridges` and returns a list of `(service, open_duration_secs)` pairs.
///
/// # F-CONDUCTOR-01 + F-CONDUCTOR-02 fix
///
/// Previously this returned `Vec<(String, u64)>` and swallowed ALL errors
/// (transport, parse, schema drift) as empty vec — indistinguishable from
/// "no breakers open". This caused [`BreakerPersistentOpenRule`] to see zero
/// candidates and never fire, reproducing the S112 incident (SYNTHEX breaker
/// open for days undetected).
///
/// Now returns `Result<_, BreakerProbeError>`. The caller matches on `Err` to
/// log the specific failure variant and increment [`BREAKER_PROBE_FAILURES`].
///
/// Additionally, a breaker whose JSON row is missing `open_duration_secs`
/// (schema drift) is included with `u64::MAX` open seconds and a warn log,
/// so `BreakerPersistentOpenRule` sees it as persistently open rather than
/// silently dropping it.
fn probe_breaker_durations(_weaver_url: &str) -> Result<Vec<(String, u64)>, BreakerProbeError> {
    const URL: &str = "http://localhost:8133/bridges";
    let resp = ureq::get(URL)
        .timeout(Duration::from_secs(2))
        .call()
        .map_err(|e| BreakerProbeError::Transport(e.to_string()))?;

    let json: serde_json::Value = serde_json::from_reader(resp.into_reader())
        .map_err(|e| BreakerProbeError::Parse(e.to_string()))?;

    let arr = json.as_array().ok_or_else(|| {
        BreakerProbeError::SchemaDrift(format!(
            "expected JSON array from {URL}, got: {}",
            json.to_string().chars().take(80).collect::<String>()
        ))
    })?;

    let mut out = Vec::with_capacity(arr.len());
    for b in arr {
        let Some(svc) = b["service"].as_str() else {
            warn!(url = URL, "breaker entry missing 'service' field — skipping");
            continue;
        };
        let open_secs = b["open_duration_secs"].as_u64().unwrap_or_else(|| {
            // F-CONDUCTOR-01: schema drift — field missing or renamed.
            // Include with sentinel MAX so the persistent-open rule fires;
            // the warn lets operators know a field is missing.
            warn!(
                url = URL,
                service = %svc,
                "open_duration_secs missing — passing breaker through with sentinel u64::MAX"
            );
            u64::MAX
        });
        if open_secs > 0 {
            out.push((svc.to_owned(), open_secs));
        }
    }
    Ok(out)
}

fn broadcast_atuin_kv(event: &DivergenceEvent) {
    let key = format!("zen.divergence.{}.last", event.kind);
    let value = serde_json::to_string(&json!({
        "kind": event.kind,
        "severity": event.severity.as_str(),
        "ts": chrono::Utc::now().timestamp_millis(),
    }))
    .unwrap_or_default();

    let status = std::process::Command::new("atuin")
        .args(["kv", "set", "--key", &key, "--value", &value])
        .status();

    match status {
        Ok(s) if s.success() => {}
        Ok(s) => warn!(key, "atuin kv set non-zero: {:?}", s.code()),
        Err(e) => warn!(key, "atuin kv set failed: {e}"),
    }
}
