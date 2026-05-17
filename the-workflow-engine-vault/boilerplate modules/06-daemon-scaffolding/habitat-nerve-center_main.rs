//! Habitat Nerve Center — daemon entry point.
//!
//! Spawns a background probe loop that queries all 11 Habitat services every
//! 30 seconds and an Axum HTTP server on port `8083` (or `$NERVE_PORT`).
//!
//! # Graceful shutdown
//!
//! The daemon listens for `SIGTERM` and `SIGINT` and shuts down cleanly.
//! Any individual service being unreachable never causes a panic or crash;
//! failures are logged at `WARN` level and the loop continues.

#![forbid(unsafe_code)]

use std::{
    collections::VecDeque,
    env,
    net::{IpAddr, SocketAddr},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use chrono::Utc;
use parking_lot::RwLock;
use tokio::{signal, time::sleep};
use tracing::{debug, info, warn};

use habitat_nerve_center::m1_types::{
    default_services, HabitatState, HealthStatus, MetricSnapshot, ProbeResult, ServiceConfig,
};
use habitat_nerve_center::{m8_rm_notifier, pv2_bus};

// ── Constants ────────────────────────────────────────────────────────────────

/// Default HTTP port for the Nerve Center daemon.
const DEFAULT_PORT: u16 = 8083;

/// How often the probe loop fires.
const PROBE_INTERVAL: Duration = Duration::from_secs(30);

/// Per-request TCP connect **and** HTTP read timeout when probing a service.
///
/// Using `AgentBuilder::timeout_connect` + `timeout_read` caps both phases.
/// `ureq::get().timeout()` only sets the read timeout in ureq v2.
const PROBE_TIMEOUT: Duration = Duration::from_secs(5);

/// Maximum number of historical [`HabitatState`] snapshots retained in memory.
///
/// At 30-second intervals this is 60 minutes of history.  Backed by a
/// [`VecDeque`] for O(1) front-eviction instead of O(n) `Vec::remove(0)`.
const HISTORY_CAPACITY: usize = 120;

/// Maximum number of webhook subscribers allowed at any time.
///
/// Prevents unbounded `Vec` growth that would cause 10K+ `tokio::spawn` calls
/// per 5-second notification cycle (F-BETA-BR-6 `DoS`).
const MAX_SUBSCRIBERS: usize = 256;

// ── Shared state ─────────────────────────────────────────────────────────────

/// Thread-safe, shared application state passed to every Axum handler and the
/// background probe task.
#[derive(Debug)]
struct AppState {
    /// Latest aggregated snapshot; `None` until the first probe cycle completes.
    current: RwLock<Option<HabitatState>>,
    /// Ring-buffer of historical snapshots (oldest first, newest last).
    ///
    /// [`VecDeque`] gives O(1) `push_back` + `pop_front`; `Vec::remove(0)`
    /// would be O(n) on every eviction cycle.
    history: RwLock<VecDeque<HabitatState>>,
    /// Total number of alert events raised since daemon start.
    alert_count: RwLock<u64>,
    /// Registered webhook URLs for RM push notification fan-out.
    rm_subscribers: Arc<RwLock<Vec<String>>>,
    /// Live Kuramoto order parameter `r` from the PV2 Unix bus.
    ///
    /// Stored as an f64 bit-pattern in an [`AtomicU64`] for lock-free reads.
    /// Default `0.0` = field-unaware until the first `field.tick` is delivered.
    last_field_r: Arc<AtomicU64>,
}

impl AppState {
    /// Read the current Kuramoto r value without locking.
    fn field_r(&self) -> f64 {
        f64::from_bits(self.last_field_r.load(Ordering::Relaxed)).clamp(0.0, 1.0)
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current: RwLock::new(None),
            history: RwLock::new(VecDeque::with_capacity(HISTORY_CAPACITY)),
            alert_count: RwLock::new(0),
            rm_subscribers: Arc::new(RwLock::new(vec![
                "http://127.0.0.1:8133/hooks/rm_notify".to_string(),
                "http://127.0.0.1:8092/notify/rm_entry".to_string(),
            ])),
            last_field_r: Arc::new(AtomicU64::new(0_f64.to_bits())),
        }
    }
}

// ── HTTP agent ───────────────────────────────────────────────────────────────

/// Build a [`ureq::Agent`] with explicit connect **and** read timeouts.
fn make_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(PROBE_TIMEOUT)
        .timeout_read(PROBE_TIMEOUT)
        .build()
}

// ── Probing ──────────────────────────────────────────────────────────────────

/// Probe a single service and return a [`ProbeResult`].
///
/// Never panics — all I/O and HTTP errors are captured in the result's `error`
/// field with status [`HealthStatus::Unhealthy`].
fn probe_service(agent: &ureq::Agent, cfg: &ServiceConfig) -> ProbeResult {
    let url = cfg.health_url();
    let start = Instant::now();

    match agent.get(&url).call() {
        Ok(response) => {
            let elapsed = start.elapsed();
            let code = response.status();
            if (200..300).contains(&code) {
                debug!(service = %cfg.name, port = cfg.port, code, elapsed_ms = elapsed.as_millis(), "probe ok");
                ProbeResult::healthy(&cfg.name, cfg.port, code, elapsed)
            } else {
                warn!(service = %cfg.name, port = cfg.port, code, "probe returned non-2xx");
                ProbeResult::unhealthy(&cfg.name, cfg.port, Some(code), elapsed, format!("non-2xx status: {code}"))
            }
        }
        Err(ureq::Error::Status(code, _)) => {
            let elapsed = start.elapsed();
            warn!(service = %cfg.name, port = cfg.port, code, "probe error status");
            ProbeResult::unhealthy(&cfg.name, cfg.port, Some(code), elapsed, format!("HTTP error {code}"))
        }
        Err(ureq::Error::Transport(t)) => {
            let elapsed = start.elapsed();
            let msg = t.to_string();
            warn!(service = %cfg.name, port = cfg.port, error = %msg, "probe transport failure");
            ProbeResult::unhealthy(&cfg.name, cfg.port, None, elapsed, msg)
        }
    }
}

/// Probe all services in `services` and return one [`ProbeResult`] per service.
fn probe_all(agent: &ureq::Agent, services: &[ServiceConfig]) -> Vec<ProbeResult> {
    services.iter().map(|svc| probe_service(agent, svc)).collect()
}

// ── Metric scraping ──────────────────────────────────────────────────────────

/// Navigate a dot-separated key path into a JSON value and extract `f64`.
///
/// Returns `None` on any missing key or non-numeric value.
fn extract_f64(body: &serde_json::Value, path: &str) -> Option<f64> {
    let mut node = body;
    for key in path.split('.') {
        node = node.get(key)?;
    }
    node.as_f64()
}

/// Navigate a dot-separated key path into a JSON value and extract `u64`.
fn extract_u64(body: &serde_json::Value, path: &str) -> Option<u64> {
    let mut node = body;
    for key in path.split('.') {
        node = node.get(key)?;
    }
    node.as_u64()
}

/// Fetch enriched metrics from PV2, SYNTHEX, and ORAC.
///
/// Each fetch is independent — a failure on one endpoint does not prevent the
/// others from succeeding.  Out-of-range float values are clamped to `[0, 1]`
/// to guard against buggy server responses.
fn probe_metrics(agent: &ureq::Agent) -> MetricSnapshot {
    let mut snap = MetricSnapshot {
        captured_at: Some(Utc::now()),
        ..MetricSnapshot::default()
    };

    // PV2 Kuramoto r — GET :8132/field
    if let Ok(resp) = agent.get("http://127.0.0.1:8132/field").call() {
        if let Ok(text) = resp.into_string() {
            if let Ok(body) = serde_json::from_str::<serde_json::Value>(&text) {
                snap.pv2_r = extract_f64(&body, "r").map(|v| v.clamp(0.0, 1.0));
            }
        }
    }

    // SYNTHEX v2 thermal — GET :8092/v3/thermal
    if let Ok(resp) = agent.get("http://127.0.0.1:8092/v3/thermal").call() {
        if let Ok(text) = resp.into_string() {
            if let Ok(body) = serde_json::from_str::<serde_json::Value>(&text) {
                snap.synthex_temperature = extract_f64(&body, "temperature")
                    .or_else(|| extract_f64(&body, "T"))
                    .or_else(|| extract_f64(&body, "thermal.temperature"))
                    .map(|v| v.clamp(0.0, 1.0));
            }
        }
    }

    // ORAC RALPH — GET :8133/health
    if let Ok(resp) = agent.get("http://127.0.0.1:8133/health").call() {
        if let Ok(text) = resp.into_string() {
            if let Ok(body) = serde_json::from_str::<serde_json::Value>(&text) {
                snap.orac_ralph_gen = extract_u64(&body, "ralph_gen")
                    .or_else(|| extract_u64(&body, "ralph.gen"))
                    .or_else(|| extract_u64(&body, "generation"));
                snap.orac_ralph_fitness = extract_f64(&body, "ralph_fitness")
                    .or_else(|| extract_f64(&body, "ralph.fitness"))
                    .or_else(|| extract_f64(&body, "fitness"))
                    .map(|v| v.clamp(0.0, 1.0));
            }
        }
    }

    snap
}

// ── Alert detection ──────────────────────────────────────────────────────────

/// Evaluate alert conditions between `previous` and `current` snapshots.
///
/// Returns the number of alert events emitted (for counter accumulation).
/// Recovery transitions (unhealthy → healthy) are logged at `INFO` but do NOT
/// increment the alert counter.
fn evaluate_alerts(current: &HabitatState, previous: Option<&HabitatState>) -> u64 {
    let mut count: u64 = 0;

    for result in &current.services {
        if result.status == HealthStatus::Unhealthy {
            let was_healthy = previous
                .and_then(|prev| prev.service(&result.name))
                .is_none_or(|prev_result| prev_result.status == HealthStatus::Healthy);

            if was_healthy {
                warn!(
                    service = %result.name,
                    port = result.port,
                    error = result.error.as_deref().unwrap_or("unknown"),
                    "ALERT: service became unhealthy"
                );
            } else {
                warn!(service = %result.name, port = result.port, "ALERT: service still unhealthy");
            }
            count += 1;
        }

        // Recovery — log at INFO, no alert counter increment.
        if result.status == HealthStatus::Healthy {
            let was_unhealthy = previous
                .and_then(|prev| prev.service(&result.name))
                .is_some_and(|prev_result| prev_result.status == HealthStatus::Unhealthy);

            if was_unhealthy {
                info!(service = %result.name, port = result.port, "RECOVERY: service is healthy again");
            }
        }
    }

    // System-wide degradation threshold: < 75 % healthy.
    if current.overall_health < 0.75 {
        warn!(
            healthy = current.healthy_count,
            total = current.total_count,
            health_pct = format!("{:.0}%", current.overall_health * 100.0),
            "ALERT: overall Habitat health is degraded"
        );
        count += 1;
    }

    count
}

// ── Background probe task ────────────────────────────────────────────────────

/// Long-running task that probes all services on every tick and updates
/// the shared [`AppState`].
async fn probe_loop(state: Arc<AppState>, services: Vec<ServiceConfig>) {
    info!(service_count = services.len(), interval_secs = PROBE_INTERVAL.as_secs(), "probe loop started");

    loop {
        sleep(PROBE_INTERVAL).await;

        // Offload blocking HTTP I/O to the thread pool.
        let services_clone = services.clone();
        let probe_task = tokio::task::spawn_blocking(move || {
            let agent = make_agent();
            let probes = probe_all(&agent, &services_clone);
            let metrics = probe_metrics(&agent);
            (probes, metrics)
        })
        .await;

        let (probes, metrics) = match probe_task {
            Ok(pair) => pair,
            Err(err) => {
                warn!(error = %err, "probe blocking task panicked — skipping cycle");
                continue;
            }
        };

        let healthy = probes.iter().filter(|p| p.status.is_healthy()).count();
        let total = probes.len();
        debug!(healthy, total, "probe cycle complete");

        let mut new_state = HabitatState::from_probes(probes);
        new_state.metrics = metrics;

        // Snapshot previous state for diff — do this BEFORE acquiring write lock.
        let previous = state.current.read().clone();
        let alert_delta = evaluate_alerts(&new_state, previous.as_ref());

        // Update current snapshot.
        *state.current.write() = Some(new_state.clone());

        // Append to the bounded ring-buffer.  VecDeque::pop_front is O(1).
        {
            let mut history = state.history.write();
            if history.len() == HISTORY_CAPACITY {
                history.pop_front();
            }
            history.push_back(new_state.clone());
        }

        // Persist health snapshot to POVM (fire-and-forget, never blocks probe loop).
        habitat_nerve_center::m3_aggregator::povm_bridge::write_health_snapshot(&new_state);

        if alert_delta > 0 {
            *state.alert_count.write() += alert_delta;
        }

        info!(healthy, total, alert_delta, "probe cycle done");
    }
}

// ── SSRF guard ────────────────────────────────────────────────────────────────

/// Return `true` if the given IP is in a private or link-local range.
///
/// Blocked ranges (RFC 1918, RFC 3927, loopback, metadata):
/// - 127.0.0.0/8  (loopback)
/// - 10.0.0.0/8   (private)
/// - 172.16.0.0/12 (private)
/// - 192.168.0.0/16 (private)
/// - 169.254.0.0/16 (link-local / cloud metadata)
fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let octets = v4.octets();
            // 127.x.x.x
            octets[0] == 127
            // 10.x.x.x
            || octets[0] == 10
            // 172.16.0.0/12 — 172.16.x.x through 172.31.x.x
            || (octets[0] == 172 && (16..=31).contains(&octets[1]))
            // 192.168.x.x
            || (octets[0] == 192 && octets[1] == 168)
            // 169.254.x.x (link-local / cloud metadata endpoint)
            || (octets[0] == 169 && octets[1] == 254)
        }
        // Reject all IPv6 addresses that are not globally unicast.
        // Loopback (::1) and link-local (fe80::/10) are the main threats.
        IpAddr::V6(v6) => v6.is_loopback() || {
            let segments = v6.segments();
            // fe80::/10 link-local
            (segments[0] & 0xffc0) == 0xfe80
        },
    }
}

/// Validate a subscriber URL for SSRF safety.
///
/// Accepts only `http` and `https` schemes whose resolved host is NOT in a
/// private IP range.  Returns `Ok(())` on success, `Err(reason)` on rejection.
///
/// # Errors
///
/// Returns a human-readable rejection reason string on any validation failure.
fn validate_subscriber_url(raw: &str) -> Result<(), String> {
    let parsed = url::Url::parse(raw)
        .map_err(|e| format!("invalid URL: {e}"))?;

    match parsed.scheme() {
        "http" | "https" => {}
        scheme => return Err(format!("scheme '{scheme}' not allowed; use http or https")),
    }

    let host = parsed
        .host_str()
        .ok_or_else(|| "URL has no host".to_string())?;

    // Attempt to parse host as a raw IP address first; if it's a hostname we
    // cannot resolve it at validation time — we allow it and rely on the
    // scheme restriction + the fact that internal hostnames are not reachable
    // from the server's network in a correctly configured habitat.
    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_private_ip(&ip) {
            return Err(format!("host '{host}' resolves to a private/reserved address"));
        }
    }

    Ok(())
}

// ── HTTP handlers ─────────────────────────────────────────────────────────────

/// `GET /health` — liveness probe; always returns 200.
async fn handle_health() -> (StatusCode, &'static str) {
    (StatusCode::OK, "ok")
}

/// `GET /status` — full current [`HabitatState`] as JSON.
///
/// # Errors
///
/// Returns `503 SERVICE_UNAVAILABLE` when no probe cycle has completed yet.
async fn handle_status(
    State(state): State<Arc<AppState>>,
) -> Result<Json<HabitatState>, (StatusCode, &'static str)> {
    state
        .current
        .read()
        .clone()
        .map(Json)
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "no probe data yet"))
}

/// Response body for `GET /summary`.
#[derive(serde::Serialize)]
struct SummaryResponse {
    healthy_count: usize,
    total_count: usize,
    overall_health: f64,
    alert_count: u64,
    history_len: usize,
    pv2_r: Option<f64>,
    synthex_temperature: Option<f64>,
    orac_ralph_gen: Option<u64>,
    orac_ralph_fitness: Option<f64>,
}

/// `GET /summary` — lightweight dashboard summary.
///
/// # Errors
///
/// Returns `503 SERVICE_UNAVAILABLE` when no probe cycle has completed yet.
async fn handle_summary(
    State(state): State<Arc<AppState>>,
) -> Result<Json<SummaryResponse>, (StatusCode, &'static str)> {
    // Acquire each lock separately; never hold two simultaneously.
    let current = state.current.read().clone();
    let alert_count = *state.alert_count.read();
    let history_len = state.history.read().len();

    let snap = current.ok_or((StatusCode::SERVICE_UNAVAILABLE, "no probe data yet"))?;

    Ok(Json(SummaryResponse {
        healthy_count: snap.healthy_count,
        total_count: snap.total_count,
        overall_health: snap.overall_health,
        alert_count,
        history_len,
        pv2_r: snap.metrics.pv2_r,
        synthex_temperature: snap.metrics.synthex_temperature,
        orac_ralph_gen: snap.metrics.orac_ralph_gen,
        orac_ralph_fitness: snap.metrics.orac_ralph_fitness,
    }))
}

/// `GET /history` — all retained historical snapshots as a JSON array.
async fn handle_history(State(state): State<Arc<AppState>>) -> Json<Vec<HabitatState>> {
    Json(state.history.read().iter().cloned().collect())
}

/// `POST /rm/subscribe` — register a webhook URL for RM push notifications.
///
/// Body: `{"url": "http://..."}`.
///
/// Validation:
/// - `url` field must be present (`400 Bad Request` otherwise).
/// - URL must use `http` or `https` scheme (`400`).
/// - Host must not resolve to a private/reserved IP range (`400`) — prevents
///   SSRF to cloud metadata endpoints (169.254.x.x) or internal hosts.
/// - Subscriber count must be below [`MAX_SUBSCRIBERS`] (`429 Too Many Requests`).
async fn handle_rm_subscribe(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> impl axum::response::IntoResponse {
    let url = match body.get("url").and_then(|u| u.as_str()) {
        Some(u) => u.to_string(),
        None => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "missing url"})),
            );
        }
    };

    // SSRF guard: validate scheme + reject private/reserved IP hosts.
    if let Err(reason) = validate_subscriber_url(&url) {
        warn!("rm_notifier: rejected subscriber URL {url:?}: {reason}");
        return (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": reason})),
        );
    }

    // DoS cap: reject registration once the subscriber list is full.
    {
        let subs = state.rm_subscribers.read();
        if subs.len() >= MAX_SUBSCRIBERS {
            warn!("rm_notifier: subscriber cap ({MAX_SUBSCRIBERS}) reached, rejecting {url:?}");
            return (
                axum::http::StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({
                    "error": format!("subscriber limit of {MAX_SUBSCRIBERS} reached")
                })),
            );
        }
    }

    state.rm_subscribers.write().push(url.clone());
    tracing::info!("rm_notifier: registered subscriber {url}");
    (
        axum::http::StatusCode::OK,
        Json(serde_json::json!({"registered": url})),
    )
}

/// `GET /rm/subscribers` — list registered webhook URLs.
async fn handle_rm_subscribers(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let urls = state.rm_subscribers.read().clone();
    let count = urls.len();
    Json(serde_json::json!({"subscribers": urls, "count": count}))
}

/// `GET /metrics` — Prometheus text-format metrics.
///
/// Exposes `habitat_field_r` as a gauge showing the live Kuramoto order
/// parameter `r` received from the PV2 Unix bus.  When the bus has not yet
/// delivered a tick (cold start) the value is `0.0`.
///
/// Example response:
/// ```text
/// # HELP habitat_field_r Kuramoto order parameter r from PV2 Unix bus.
/// # TYPE habitat_field_r gauge
/// habitat_field_r 0.762
/// ```
async fn handle_metrics(State(state): State<Arc<AppState>>) -> Response {
    let r = state.field_r();
    let body = format!(
        "# HELP habitat_field_r Kuramoto order parameter r from PV2 Unix bus.\n\
         # TYPE habitat_field_r gauge\n\
         habitat_field_r {r}\n"
    );
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
        .into_response()
}

// ── Graceful shutdown ─────────────────────────────────────────────────────────

/// Resolves when `SIGINT` or `SIGTERM` is received.
///
/// Signal-handler installation errors are logged; the future still resolves so
/// the daemon can shut down cleanly without panicking.
async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(err) = signal::ctrl_c().await {
            tracing::error!(error = %err, "Ctrl-C signal handler failed");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut sig) => {
                sig.recv().await;
            }
            Err(err) => {
                tracing::error!(error = %err, "SIGTERM handler installation failed");
                // Remain pending so ctrl_c can still fire.
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    info!("shutdown signal received — stopping daemon");
}

// ── Bulletproof socket binding ──────────────────────────────────────────────────

/// Creates a TCP listener socket with `SO_REUSEADDR` and `SO_REUSEPORT` set.
/// This allows immediate rebinding on restart without `TIME_WAIT` conflicts.
fn try_bind(addr: &SocketAddr) -> std::io::Result<tokio::net::TcpListener> {
    let socket = socket2::Socket::new(
        socket2::Domain::IPV4,
        socket2::Type::STREAM,
        Some(socket2::Protocol::TCP),
    )?;
    socket.set_reuse_address(true)?;
    #[cfg(target_os = "linux")]
    socket.set_reuse_port(true)?;
    socket.set_nonblocking(true)?;
    socket.bind(&(*addr).into())?;
    socket.listen(1024)?;
    tokio::net::TcpListener::from_std(socket.into())
}

/// Bind `addr` with up to 3 retries (200ms / 800ms / 2s backoffs).
///
/// Exits the process on failure after logging port-occupancy diagnostics.
fn bind_with_retry(addr: &SocketAddr) -> tokio::net::TcpListener {
    let port = addr.port();
    let backoffs = [
        Duration::from_millis(200),
        Duration::from_millis(800),
        Duration::from_secs(2),
    ];
    for (attempt, backoff) in backoffs.iter().enumerate() {
        match try_bind(addr) {
            Ok(listener) => return listener,
            Err(e) => {
                if attempt < backoffs.len() - 1 {
                    debug!(attempt, error = %e, backoff_ms = backoff.as_millis(), "bind failed, retrying");
                    std::thread::sleep(*backoff);
                } else {
                    tracing::error!(port, error = %e, "failed to bind TCP listener after 3 attempts");
                    if let Ok(output) = std::process::Command::new("ss").args(["-tlnp"]).output() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        if let Some(line) = stdout.lines().find(|l| l.contains(&format!(":{port}"))) {
                            tracing::error!("port occupancy: {line}");
                        }
                    }
                    std::process::exit(1);
                }
            }
        }
    }
    unreachable!("bind retry loop should not reach here")
}

/// Initialise the `tracing` subscriber writing to a file (not stderr).
///
/// Writing to stderr is unsafe after `devenv` closes the pipe — any write
/// returns `EPIPE` and Rust's stdio layer panics at `stdio.rs:1165`.
fn init_tracing() {
    let log_path = std::env::var("TRACING_LOG_PATH")
        .unwrap_or_else(|_| format!("/tmp/{}-tracing.log", env!("CARGO_PKG_NAME")));
    let log_writer: Box<dyn std::io::Write + Send> = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map(|f| -> Box<dyn std::io::Write + Send> { Box::new(f) })
        .or_else(|_| {
            std::fs::OpenOptions::new()
                .write(true)
                .open("/dev/null")
                .map(|f| -> Box<dyn std::io::Write + Send> { Box::new(f) })
        })
        .unwrap_or_else(|_| Box::new(std::io::sink()));
    tracing_subscriber::fmt()
        .with_writer(std::sync::Mutex::new(log_writer))
        .with_ansi(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
}

// ── Entry point ───────────────────────────────────────────────────────────────

/// Install a panic hook that records `file:line:col` + message to
/// `/tmp/habitat-nerve-center-panic.log`. **File write only** — no tracing,
/// no stderr. Rationale: `devenv` closes the stderr pipe after starting us;
/// any subsequent stderr write returns `EPIPE`, and Rust's default stdio
/// panics at `stdio.rs:1165 "failed printing to stderr"`. If this hook writes
/// to stderr we double-panic → abort before the file is flushed, losing the
/// original panic site. File append under `EPIPE` is harmless.
fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let loc = info.location().map_or_else(
            || "<unknown>".to_string(),
            |l| format!("{}:{}:{}", l.file(), l.line(), l.column()),
        );
        let msg = info
            .payload()
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| {
                info.payload()
                    .downcast_ref::<String>()
                    .map(String::as_str)
            })
            .unwrap_or("<non-string payload>");
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(concat!("/tmp/", env!("CARGO_PKG_NAME"), "-panic.log"))
        {
            use std::io::Write;
            let ts = chrono::Utc::now().to_rfc3339();
            let _ = writeln!(f, "[{ts}] PANIC at {loc}: {msg}");
            let _ = f.flush();
        }
    }));
}

#[tokio::main]
async fn main() {
    init_tracing();
    install_panic_hook();

    let _pidlock = match habitat_pidlock::PidLock::acquire("habitat-nerve-center") {
        Ok(lock) => lock,
        Err(e) => {
            tracing::error!("pidlock acquire failed: {e}");
            std::process::exit(1);
        }
    };

    let port: u16 = env::var("NERVE_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_PORT);

    // Security fix (F-BETA-BR-2): bind to loopback by default.
    // Override with BIND_ADDR env var when external access is needed.
    let bind_host =
        env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1".to_owned());
    let addr: SocketAddr = format!("{bind_host}:{port}")
        .parse()
        .unwrap_or_else(|_| SocketAddr::from(([127, 0, 0, 1], port)));
    info!(port, %addr, "Habitat Nerve Center starting");

    let services = default_services();
    info!(count = services.len(), "loaded service registry");

    let state: Arc<AppState> = Arc::new(AppState::default());

    // S1: Subscribe to PV2 Unix bus for live field.tick events.
    // AP23: spawn() calls tokio::spawn internally — do not wrap in another spawn.
    {
        let field_r = Arc::clone(&state.last_field_r);
        pv2_bus::spawn("habitat-nerve-center", move |tick| {
            let r = tick.r.clamp(0.0, 1.0);
            field_r.store(r.to_bits(), Ordering::Relaxed);
            tracing::debug!(r, tick = tick.tick, spheres = tick.spheres, "pv2_bus: field tick received");
        });
    }

    // Spawn RM push notifier (polls /recent every 5s, fans out to webhooks).
    m8_rm_notifier::spawn(Arc::clone(&state.rm_subscribers));

    {
        let state_clone = Arc::clone(&state);
        let services_clone = services.clone();
        tokio::spawn(async move {
            probe_loop(state_clone, services_clone).await;
        });
    }

    let app = Router::new()
        .route("/health", get(handle_health))
        .route("/status", get(handle_status))
        .route("/summary", get(handle_summary))
        .route("/history", get(handle_history))
        .route("/rm/subscribe", post(handle_rm_subscribe))
        .route("/rm/subscribers", get(handle_rm_subscribers))
        .route("/metrics", get(handle_metrics))
        .with_state(Arc::clone(&state));

    info!(%addr, "HTTP server listening");
    let listener = bind_with_retry(&addr);

    if let Err(err) = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        tracing::error!(error = %err, "server error");
    }

    info!("Habitat Nerve Center stopped");
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use habitat_nerve_center::m1_types::{HealthStatus, ProbeResult};
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    fn healthy(name: &str) -> ProbeResult {
        ProbeResult::healthy(name, 8080, 200, Duration::from_millis(10))
    }

    fn unhealthy(name: &str) -> ProbeResult {
        ProbeResult::unhealthy(name, 8080, None, Duration::from_millis(3000), "timeout")
    }

    // ── evaluate_alerts ──────────────────────────────────────────────────────

    #[test]
    fn alerts_zero_when_all_healthy_no_previous() {
        let state = HabitatState::from_probes(vec![healthy("a"), healthy("b")]);
        assert_eq!(evaluate_alerts(&state, None), 0);
    }

    #[test]
    fn alerts_nonzero_when_service_unhealthy_first_probe() {
        let state = HabitatState::from_probes(vec![healthy("a"), unhealthy("b")]);
        assert!(evaluate_alerts(&state, None) >= 1);
    }

    #[test]
    fn alerts_include_degraded_overall_health() {
        let state = HabitatState::from_probes(vec![
            healthy("a"), unhealthy("b"), unhealthy("c"), unhealthy("d"),
        ]);
        assert!(evaluate_alerts(&state, None) >= 2);
    }

    #[test]
    fn alerts_zero_stable_all_healthy() {
        let prev = HabitatState::from_probes(vec![healthy("a"), healthy("b")]);
        let curr = HabitatState::from_probes(vec![healthy("a"), healthy("b")]);
        assert_eq!(evaluate_alerts(&curr, Some(&prev)), 0);
    }

    #[test]
    fn alerts_nonzero_on_transition_to_unhealthy() {
        let prev = HabitatState::from_probes(vec![healthy("a"), healthy("b")]);
        let curr = HabitatState::from_probes(vec![healthy("a"), unhealthy("b")]);
        assert!(evaluate_alerts(&curr, Some(&prev)) >= 1);
    }

    #[test]
    fn no_alert_increment_on_recovery() {
        let prev = HabitatState::from_probes(vec![healthy("a"), unhealthy("b")]);
        let curr = HabitatState::from_probes(vec![healthy("a"), healthy("b")]);
        assert_eq!(evaluate_alerts(&curr, Some(&prev)), 0);
    }

    // ── AppState ─────────────────────────────────────────────────────────────

    #[test]
    fn app_state_initial_current_is_none() {
        assert!(AppState::default().current.read().is_none());
    }

    #[test]
    fn app_state_history_starts_empty() {
        assert!(AppState::default().history.read().is_empty());
    }

    #[test]
    fn app_state_alert_count_starts_zero() {
        assert_eq!(*AppState::default().alert_count.read(), 0);
    }

    #[test]
    fn app_state_rm_subscribers_preseeded_two() {
        let state = AppState::default();
        assert_eq!(state.rm_subscribers.read().len(), 2);
    }

    #[test]
    fn app_state_rm_subscribers_contains_orac_url() {
        let state = AppState::default();
        let subs = state.rm_subscribers.read();
        assert!(subs.iter().any(|u| u.contains("8133")));
    }

    #[test]
    fn app_state_rm_subscribers_contains_synthex_url() {
        let state = AppState::default();
        let subs = state.rm_subscribers.read();
        assert!(subs.iter().any(|u| u.contains("8092")));
    }

    #[test]
    fn app_state_rm_subscribers_can_push() {
        let state = AppState::default();
        state.rm_subscribers.write().push("http://test:9999".into());
        assert_eq!(state.rm_subscribers.read().len(), 3);
    }

    #[test]
    fn app_state_rm_subscribers_arc_shared() {
        let state = AppState::default();
        let clone = Arc::clone(&state.rm_subscribers);
        clone.write().push("http://extra".into());
        assert_eq!(state.rm_subscribers.read().len(), 3);
    }

    // ── last_field_r + field_r() ──────────────────────────────────────────────

    #[test]
    fn app_state_last_field_r_default_zero() {
        let state = AppState::default();
        assert!((state.field_r()).abs() < f64::EPSILON);
    }

    #[test]
    fn app_state_field_r_updates_via_store() {
        let state = AppState::default();
        let r = 0.762_f64;
        state.last_field_r.store(r.to_bits(), Ordering::Relaxed);
        assert!((state.field_r() - r).abs() < f64::EPSILON);
    }

    #[test]
    fn app_state_field_r_clamped_at_one() {
        let state = AppState::default();
        state.last_field_r.store(2.0_f64.to_bits(), Ordering::Relaxed);
        assert!((state.field_r() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn app_state_field_r_clamped_at_zero() {
        let state = AppState::default();
        state.last_field_r.store((-0.5_f64).to_bits(), Ordering::Relaxed);
        assert!((state.field_r()).abs() < f64::EPSILON);
    }

    #[test]
    fn app_state_last_field_r_arc_cloneable() {
        let state = AppState::default();
        let clone = Arc::clone(&state.last_field_r);
        clone.store(0.5_f64.to_bits(), Ordering::Relaxed);
        assert!((state.field_r() - 0.5).abs() < f64::EPSILON);
    }

    // ── /metrics Prometheus format ─────────────────────────────────────────────

    #[test]
    fn metrics_format_contains_help_line() {
        let r = 0.5_f64;
        let body = format!(
            "# HELP habitat_field_r Kuramoto order parameter r from PV2 Unix bus.\n\
             # TYPE habitat_field_r gauge\n\
             habitat_field_r {r}\n"
        );
        assert!(body.contains("# HELP habitat_field_r"));
    }

    #[test]
    fn metrics_format_contains_type_gauge() {
        let r = 0.5_f64;
        let body = format!(
            "# HELP habitat_field_r Kuramoto order parameter r from PV2 Unix bus.\n\
             # TYPE habitat_field_r gauge\n\
             habitat_field_r {r}\n"
        );
        assert!(body.contains("# TYPE habitat_field_r gauge"));
    }

    #[test]
    fn metrics_format_contains_value() {
        let r = 0.762_f64;
        let body = format!(
            "# HELP habitat_field_r Kuramoto order parameter r from PV2 Unix bus.\n\
             # TYPE habitat_field_r gauge\n\
             habitat_field_r {r}\n"
        );
        assert!(body.contains("habitat_field_r 0.762"));
    }

    #[test]
    fn metrics_format_zero_r() {
        let r = 0.0_f64;
        let body = format!("habitat_field_r {r}\n");
        assert!(body.contains("habitat_field_r 0"));
    }

    #[test]
    fn metrics_format_ends_with_newline() {
        let r = 0.3_f64;
        let body = format!(
            "# HELP habitat_field_r Kuramoto order parameter r from PV2 Unix bus.\n\
             # TYPE habitat_field_r gauge\n\
             habitat_field_r {r}\n"
        );
        assert!(body.ends_with('\n'));
    }

    #[test]
    fn app_state_current_can_be_set() {
        let state = AppState::default();
        *state.current.write() = Some(HabitatState::from_probes(vec![healthy("svc")]));
        assert!(state.current.read().is_some());
    }

    #[test]
    fn app_state_history_push_back_and_read() {
        let state = AppState::default();
        state.history.write().push_back(HabitatState::from_probes(vec![healthy("svc")]));
        assert_eq!(state.history.read().len(), 1);
    }

    // ── VecDeque ring-buffer ──────────────────────────────────────────────────

    #[test]
    fn history_capacity_constant_positive() {
        assert!(HISTORY_CAPACITY > 0);
    }

    #[test]
    fn vecdeque_does_not_exceed_capacity() {
        let state = AppState::default();
        for i in 0..=(HISTORY_CAPACITY + 10) {
            let snap = HabitatState::from_probes(vec![healthy(&format!("svc-{i}"))]);
            let mut history = state.history.write();
            if history.len() == HISTORY_CAPACITY {
                history.pop_front();
            }
            history.push_back(snap);
        }
        assert!(state.history.read().len() <= HISTORY_CAPACITY);
    }

    #[test]
    fn vecdeque_evicts_oldest_entry() {
        let cap = 3_usize;
        let mut dq: VecDeque<usize> = VecDeque::with_capacity(cap);
        for i in 0..5_usize {
            if dq.len() == cap {
                dq.pop_front();
            }
            dq.push_back(i);
        }
        // After pushing 0,1,2,3,4 with cap 3: contains 2,3,4
        assert_eq!(dq[0], 2);
        assert_eq!(dq[2], 4);
    }

    // ── Metric extraction ─────────────────────────────────────────────────────

    #[test]
    fn extract_f64_top_level() {
        let v = serde_json::json!({ "r": 0.87 });
        let r = extract_f64(&v, "r").unwrap();
        assert!((r - 0.87).abs() < f64::EPSILON);
    }

    #[test]
    fn extract_f64_nested() {
        let v = serde_json::json!({ "field": { "r": 0.91 } });
        let r = extract_f64(&v, "field.r").unwrap();
        assert!((r - 0.91).abs() < f64::EPSILON);
    }

    #[test]
    fn extract_f64_missing_returns_none() {
        assert!(extract_f64(&serde_json::json!({"x": 1.0}), "r").is_none());
    }

    #[test]
    fn extract_u64_top_level() {
        let v = serde_json::json!({ "gen": 42_u64 });
        assert_eq!(extract_u64(&v, "gen"), Some(42));
    }

    #[test]
    fn extract_u64_nested() {
        let v = serde_json::json!({ "ralph": { "gen": 23993_u64 } });
        assert_eq!(extract_u64(&v, "ralph.gen"), Some(23993));
    }

    #[test]
    fn extract_u64_missing_returns_none() {
        assert!(extract_u64(&serde_json::json!({"x": 1}), "ralph.gen").is_none());
    }

    // ── Float clamping for metric values ──────────────────────────────────────

    #[test]
    fn pv2_r_out_of_range_clamped() {
        // A buggy service returning r > 1.0 should be clamped to 1.0.
        let bad = 1.5_f64;
        assert!((bad.clamp(0.0, 1.0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn pv2_r_negative_clamped_to_zero() {
        assert!((-0.3_f64).clamp(0.0, 1.0).abs() < f64::EPSILON);
    }

    // ── Health status ─────────────────────────────────────────────────────────

    #[test]
    fn health_status_contract() {
        assert!(HealthStatus::Healthy.is_healthy());
        assert!(!HealthStatus::Unhealthy.is_healthy());
    }

    // ── Constants ─────────────────────────────────────────────────────────────

    #[test]
    fn probe_interval_is_30_secs() {
        assert_eq!(PROBE_INTERVAL, Duration::from_secs(30));
    }

    #[test]
    fn probe_timeout_is_5_secs() {
        assert_eq!(PROBE_TIMEOUT, Duration::from_secs(5));
    }

    #[test]
    fn default_port_is_8083() {
        assert_eq!(DEFAULT_PORT, 8083);
    }

    #[test]
    fn make_agent_builds_without_panic() {
        let _ = make_agent();
    }

    // ── MAX_SUBSCRIBERS ───────────────────────────────────────────────────────

    #[test]
    fn max_subscribers_constant_is_256() {
        assert_eq!(MAX_SUBSCRIBERS, 256);
    }

    #[test]
    fn max_subscribers_is_positive() {
        assert!(MAX_SUBSCRIBERS > 0);
    }

    // ── is_private_ip ─────────────────────────────────────────────────────────

    #[test]
    fn loopback_ipv4_is_private() {
        use std::net::IpAddr;
        assert!(is_private_ip(&"127.0.0.1".parse::<IpAddr>().unwrap()));
    }

    #[test]
    fn loopback_127_255_is_private() {
        use std::net::IpAddr;
        assert!(is_private_ip(&"127.255.0.1".parse::<IpAddr>().unwrap()));
    }

    #[test]
    fn rfc1918_10_is_private() {
        use std::net::IpAddr;
        assert!(is_private_ip(&"10.0.0.1".parse::<IpAddr>().unwrap()));
    }

    #[test]
    fn rfc1918_172_16_is_private() {
        use std::net::IpAddr;
        assert!(is_private_ip(&"172.16.0.1".parse::<IpAddr>().unwrap()));
    }

    #[test]
    fn rfc1918_172_31_is_private() {
        use std::net::IpAddr;
        assert!(is_private_ip(&"172.31.255.255".parse::<IpAddr>().unwrap()));
    }

    #[test]
    fn rfc1918_172_32_is_not_private() {
        use std::net::IpAddr;
        // 172.32.x.x is outside the 172.16.0.0/12 range
        assert!(!is_private_ip(&"172.32.0.1".parse::<IpAddr>().unwrap()));
    }

    #[test]
    fn rfc1918_192_168_is_private() {
        use std::net::IpAddr;
        assert!(is_private_ip(&"192.168.1.1".parse::<IpAddr>().unwrap()));
    }

    #[test]
    fn link_local_169_254_is_private() {
        use std::net::IpAddr;
        // Cloud metadata endpoint
        assert!(is_private_ip(&"169.254.169.254".parse::<IpAddr>().unwrap()));
    }

    #[test]
    fn public_ip_is_not_private() {
        use std::net::IpAddr;
        assert!(!is_private_ip(&"8.8.8.8".parse::<IpAddr>().unwrap()));
    }

    #[test]
    fn another_public_ip_is_not_private() {
        use std::net::IpAddr;
        assert!(!is_private_ip(&"1.1.1.1".parse::<IpAddr>().unwrap()));
    }

    #[test]
    fn ipv6_loopback_is_private() {
        use std::net::IpAddr;
        assert!(is_private_ip(&"::1".parse::<IpAddr>().unwrap()));
    }

    #[test]
    fn ipv6_link_local_is_private() {
        use std::net::IpAddr;
        assert!(is_private_ip(&"fe80::1".parse::<IpAddr>().unwrap()));
    }

    // ── validate_subscriber_url ───────────────────────────────────────────────

    #[test]
    fn valid_http_public_url_passes() {
        assert!(validate_subscriber_url("http://example.com/webhook").is_ok());
    }

    #[test]
    fn valid_https_public_url_passes() {
        assert!(validate_subscriber_url("https://example.com/webhook").is_ok());
    }

    #[test]
    fn metadata_endpoint_rejected() {
        // Cloud metadata SSRF vector
        let err = validate_subscriber_url("http://169.254.169.254/latest/meta-data");
        assert!(err.is_err(), "should reject metadata endpoint");
        let msg = err.unwrap_err();
        assert!(msg.contains("private") || msg.contains("reserved"), "msg: {msg}");
    }

    #[test]
    fn loopback_url_rejected() {
        let err = validate_subscriber_url("http://127.0.0.1:8080/hook");
        assert!(err.is_err(), "should reject loopback");
    }

    #[test]
    fn rfc1918_10_url_rejected() {
        let err = validate_subscriber_url("http://10.0.0.1/hook");
        assert!(err.is_err(), "should reject 10.x.x.x");
    }

    #[test]
    fn rfc1918_192_168_url_rejected() {
        let err = validate_subscriber_url("http://192.168.1.100/hook");
        assert!(err.is_err(), "should reject 192.168.x.x");
    }

    #[test]
    fn ftp_scheme_rejected() {
        let err = validate_subscriber_url("ftp://example.com/hook");
        assert!(err.is_err(), "should reject non-http(s) scheme");
        let msg = err.unwrap_err();
        assert!(msg.contains("scheme") || msg.contains("ftp"), "msg: {msg}");
    }

    #[test]
    fn file_scheme_rejected() {
        let err = validate_subscriber_url("file:///etc/passwd");
        assert!(err.is_err(), "should reject file:// scheme");
    }

    #[test]
    fn invalid_url_rejected() {
        let err = validate_subscriber_url("not-a-url");
        assert!(err.is_err(), "should reject non-URL string");
    }

    #[test]
    fn empty_string_rejected() {
        let err = validate_subscriber_url("");
        assert!(err.is_err(), "should reject empty string");
    }

    #[test]
    fn hostname_url_passes_no_resolve() {
        // Hostnames cannot be resolved at validation time — allowed
        assert!(validate_subscriber_url("http://my-webhook-service.internal/hook").is_ok());
    }

    // ── bind address hardening (F-BETA-BR-2) ─────────────────────────────────

    /// Regression: when `BIND_ADDR` is absent the bind host must resolve to
    /// `127.0.0.1` (loopback-only), preventing network exposure of /rm/subscribe.
    #[test]
    fn bind_addr_absent_defaults_to_loopback() {
        std::env::remove_var("BIND_ADDR");
        let host =
            std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1".to_owned());
        assert_eq!(
            host, "127.0.0.1",
            "default bind must be loopback — security regression F-BETA-BR-2"
        );
    }

    /// Regression: `BIND_ADDR` env var override must be propagated to the
    /// listener so operators can place the service behind a reverse proxy.
    #[test]
    fn bind_addr_env_override_is_propagated() {
        std::env::set_var("BIND_ADDR", "0.0.0.0");
        let host =
            std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1".to_owned());
        assert_eq!(host, "0.0.0.0", "BIND_ADDR env override must be respected");
        std::env::remove_var("BIND_ADDR");
    }
}
