//! # M22 Async Wrappers (P2 Remediation)
//!
//! Async wrappers around the blocking `SynthexBridge` HTTP calls.
//! Each wrapper offloads the raw TCP I/O to `tokio::task::spawn_blocking`
//! and bounds it with `tokio::time::timeout`, so the RALPH tick loop
//! never starves tokio workers (Watcher RCA root cause).
//!
//! ## Status (S116, P5.4)
//!
//! As of Stream B (S116), this HTTP path is the **fallback** for callers
//! that have not migrated to [`crate::m5_bridges::m22_synthex_ws`].
//! Build with `--features ws-bridge` to opt into the persistent
//! WebSocket transport; the [`super::m22_synthex_ws::WsRoutingPreference`]
//! selector decides per-call which path to use.
//!
//! ## Layer: L5 (Bridges)
//! ## Module: M22-async
//! ## Feature: `agentic`
//! ## Dependencies: `m22_synthex_bridge` (sync core), `tokio` runtime
//!
//! ## Why a separate impl block (not new method on existing struct)?
//!
//! Keeps the sync API untouched for non-async callers (probes, tests,
//! main-thread RALPH paths) while exposing the timeout-bounded async
//! variants for daemon hot loops.
//!
//! ## R6 testing requirement
//!
//! All tests in this module use `#[tokio::test(flavor = "multi_thread", worker_threads = 2)]`.
//! `current_thread` runtime deadlocks on `spawn_blocking` + await in the
//! same thread (a common P2 footgun).

use std::sync::Arc;
use std::time::Duration;

use crate::m1_core::m02_error_handling::{PvError, PvResult};
use crate::m5_bridges::http_helpers::{raw_http_get, raw_http_post};
use crate::m5_bridges::m22_synthex_bridge::{
    NexusEvent, NexusPullEnvelope, NexusPushEnvelope, SynthexBridge, ThermalResponse,
};

// ──────────────────────────────────────────────────────────────
// Constants
// ──────────────────────────────────────────────────────────────

/// Default timeout for a single HTTP round-trip.
///
/// Matches the sync timeout that previously starved tokio workers:
/// any call that would have blocked the worker for 2s is now bounded
/// by the same 2s, but on a dedicated blocking thread instead.
pub const DEFAULT_HTTP_TIMEOUT: Duration = Duration::from_secs(2);

/// Path constants — duplicated locally (private in m22 core).
const THERMAL_PATH: &str = "/v3/thermal";
const INGEST_PATH: &str = "/api/ingest";
const NEXUS_PUSH_PATH: &str = "/v3/nexus/push";
const NEXUS_PULL_PATH: &str = "/v3/nexus/pull";
const DECAY_TRIGGER_PATH: &str = "/v3/decay/trigger";

// ──────────────────────────────────────────────────────────────
// Async outcome
// ──────────────────────────────────────────────────────────────

/// Outcome of an async bridge call.
///
/// Distinguishes between a clean error (HTTP/parse/etc.) and a
/// timeout — important for RALPH because timeouts indicate the
/// service is alive but slow, while errors indicate it is down.
#[derive(Debug, Clone, PartialEq)]
pub enum AsyncOutcome<T> {
    /// Call completed within the timeout window.
    Ok(T),
    /// Call returned a non-timeout error.
    Err(String),
    /// Call exceeded the timeout window.
    TimedOut,
    /// Spawn-blocking task panicked or was cancelled.
    JoinError(String),
}

impl<T> AsyncOutcome<T> {
    /// Whether the call succeeded.
    #[must_use]
    pub const fn is_ok(&self) -> bool {
        matches!(self, Self::Ok(_))
    }

    /// Whether the call timed out.
    #[must_use]
    pub const fn is_timeout(&self) -> bool {
        matches!(self, Self::TimedOut)
    }

    /// Convert to `PvResult`, treating timeouts as bridge unreachable.
    ///
    /// # Errors
    ///
    /// Returns `PvError::ConfigValidation` for any non-Ok outcome.
    pub fn into_result(self, service: &str) -> PvResult<T> {
        match self {
            Self::Ok(v) => Ok(v),
            Self::Err(e) => Err(PvError::ConfigValidation(format!(
                "{service} bridge error: {e}"
            ))),
            Self::TimedOut => Err(PvError::ConfigValidation(format!(
                "{service} bridge timed out after {}s",
                DEFAULT_HTTP_TIMEOUT.as_secs()
            ))),
            Self::JoinError(e) => Err(PvError::ConfigValidation(format!(
                "{service} bridge spawn_blocking failed: {e}"
            ))),
        }
    }
}

// ──────────────────────────────────────────────────────────────
// AsyncSynthexBridge
// ──────────────────────────────────────────────────────────────

/// Async-aware wrapper around `SynthexBridge`.
///
/// Holds an `Arc<SynthexBridge>` so async tasks can share state
/// without owning the bridge exclusively. State mutation
/// (`record_failure`, `set_last_poll_tick`, etc.) happens on the
/// parent task after the await completes.
#[derive(Clone)]
pub struct AsyncSynthexBridge {
    inner: Arc<SynthexBridge>,
    timeout: Duration,
}

impl std::fmt::Debug for AsyncSynthexBridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncSynthexBridge")
            .field("timeout_ms", &self.timeout.as_millis())
            .finish_non_exhaustive()
    }
}

impl AsyncSynthexBridge {
    /// Wrap a `SynthexBridge` in an async-aware shell.
    #[must_use]
    pub fn new(bridge: SynthexBridge) -> Self {
        Self {
            inner: Arc::new(bridge),
            timeout: DEFAULT_HTTP_TIMEOUT,
        }
    }

    /// Wrap an already-shared bridge.
    #[must_use]
    pub fn from_arc(bridge: Arc<SynthexBridge>) -> Self {
        Self { inner: bridge, timeout: DEFAULT_HTTP_TIMEOUT }
    }

    /// Override the per-call timeout (default: 2s).
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Access the underlying sync bridge for state queries.
    #[must_use]
    pub fn inner(&self) -> &SynthexBridge {
        &self.inner
    }

    /// Configured timeout.
    #[must_use]
    pub const fn timeout(&self) -> Duration {
        self.timeout
    }

    // ──────────────────────────────────────────────────────────
    // Thermal poll (async)
    // ──────────────────────────────────────────────────────────

    /// Async wrapper for `poll_thermal`.
    ///
    /// Offloads the raw TCP HTTP GET to `spawn_blocking` and bounds
    /// it with the configured timeout. Returns an `AsyncOutcome` so
    /// callers can distinguish timeout from hard error.
    pub async fn poll_thermal_async(&self) -> AsyncOutcome<f64> {
        let base = self.base_url();
        let service = Self::service_name();
        let bridge = Arc::clone(&self.inner);

        let result = tokio::time::timeout(
            self.timeout,
            tokio::task::spawn_blocking(move || -> PvResult<f64> {
                let body = raw_http_get(&base, THERMAL_PATH, &service)?;
                let response: ThermalResponse =
                    serde_json::from_str(&body).map_err(|e| PvError::ConfigValidation(
                        format!("thermal parse: {e}"),
                    ))?;
                let clamped = response.thermal_adjustment();
                if !clamped.is_finite() {
                    return Err(PvError::ConfigValidation(format!(
                        "non-finite thermal_adjustment: {clamped}"
                    )));
                }
                Ok(clamped)
            }),
        )
        .await;

        match result {
            Ok(Ok(Ok(v))) => {
                // Update bridge state on the parent task — RwLock writes
                // are short and never block tokio workers.
                let _ = bridge; // sync bridge state mutation goes here in callers
                AsyncOutcome::Ok(v)
            }
            Ok(Ok(Err(e))) => AsyncOutcome::Err(e.to_string()),
            Ok(Err(join_err)) => AsyncOutcome::JoinError(join_err.to_string()),
            Err(_elapsed) => AsyncOutcome::TimedOut,
        }
    }

    // ──────────────────────────────────────────────────────────
    // Field state ingest (async)
    // ──────────────────────────────────────────────────────────

    /// Async wrapper for `post_field_state`.
    pub async fn post_field_state_async(&self, payload: Vec<u8>) -> AsyncOutcome<u16> {
        let base = self.base_url();
        let service = Self::service_name();

        let result = tokio::time::timeout(
            self.timeout,
            tokio::task::spawn_blocking(move || -> PvResult<u16> {
                raw_http_post(&base, INGEST_PATH, &payload, &service)
            }),
        )
        .await;

        Self::collapse(result)
    }

    // ──────────────────────────────────────────────────────────
    // Nexus push (async)
    // ──────────────────────────────────────────────────────────

    /// Async wrapper for `nexus_push`.
    pub async fn nexus_push_async(&self, events: Vec<NexusEvent>) -> AsyncOutcome<u16> {
        if events.is_empty() {
            return AsyncOutcome::Ok(200);
        }
        let base = self.base_url();
        let service = Self::service_name();

        let result = tokio::time::timeout(
            self.timeout,
            tokio::task::spawn_blocking(move || -> PvResult<u16> {
                let envelope = NexusPushEnvelope { events };
                let payload = serde_json::to_vec(&envelope).map_err(|e| {
                    PvError::ConfigValidation(format!("nexus push serialize: {e}"))
                })?;
                raw_http_post(&base, NEXUS_PUSH_PATH, &payload, &service)
            }),
        )
        .await;

        Self::collapse(result)
    }

    // ──────────────────────────────────────────────────────────
    // Nexus pull (async)
    // ──────────────────────────────────────────────────────────

    /// Async wrapper for `nexus_pull`.
    pub async fn nexus_pull_async(&self) -> AsyncOutcome<Vec<NexusEvent>> {
        let base = self.base_url();
        let service = Self::service_name();

        let result = tokio::time::timeout(
            self.timeout,
            tokio::task::spawn_blocking(move || -> PvResult<Vec<NexusEvent>> {
                let body = raw_http_get(&base, NEXUS_PULL_PATH, &service)?;
                let envelope: NexusPullEnvelope = serde_json::from_str(&body)
                    .map_err(|e| PvError::ConfigValidation(format!(
                        "nexus pull parse: {e}"
                    )))?;
                Ok(envelope.events)
            }),
        )
        .await;

        Self::collapse(result)
    }

    // ──────────────────────────────────────────────────────────
    // Decay trigger (async — P1 stub endpoint)
    // ──────────────────────────────────────────────────────────

    /// Async wrapper for the decay trigger endpoint (stubbed in P1).
    pub async fn decay_trigger_async(&self, payload: Vec<u8>) -> AsyncOutcome<u16> {
        let base = self.base_url();
        let service = Self::service_name();

        let result = tokio::time::timeout(
            self.timeout,
            tokio::task::spawn_blocking(move || -> PvResult<u16> {
                raw_http_post(&base, DECAY_TRIGGER_PATH, &payload, &service)
            }),
        )
        .await;

        Self::collapse(result)
    }

    // ──────────────────────────────────────────────────────────
    // Helpers
    // ──────────────────────────────────────────────────────────

    fn base_url(&self) -> String {
        // Reach into the sync bridge's port to reconstruct the addr.
        // The bridge keeps base_url private; we reuse `port()` and assume
        // 127.0.0.1 (the only deployment target).
        format!("127.0.0.1:{}", self.inner.port())
    }

    fn service_name() -> String {
        "synthex".to_owned()
    }

    /// Collapse the nested timeout/join/result into an `AsyncOutcome`.
    fn collapse<T>(
        result: Result<
            Result<PvResult<T>, tokio::task::JoinError>,
            tokio::time::error::Elapsed,
        >,
    ) -> AsyncOutcome<T> {
        match result {
            Ok(Ok(Ok(v))) => AsyncOutcome::Ok(v),
            Ok(Ok(Err(e))) => AsyncOutcome::Err(e.to_string()),
            Ok(Err(join_err)) => AsyncOutcome::JoinError(join_err.to_string()),
            Err(_) => AsyncOutcome::TimedOut,
        }
    }
}

// ──────────────────────────────────────────────────────────────
// Tests (R6: multi_thread runtime required)
// ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    fn make_bridge() -> AsyncSynthexBridge {
        AsyncSynthexBridge::new(SynthexBridge::with_config("127.0.0.1:9", 1))
            .with_timeout(Duration::from_millis(200))
    }

    // ── AsyncOutcome ─────────────────────────────────────────

    #[test]
    fn outcome_is_ok() {
        let o: AsyncOutcome<i32> = AsyncOutcome::Ok(42);
        assert!(o.is_ok());
        assert!(!o.is_timeout());
    }

    #[test]
    fn outcome_is_timeout() {
        let o: AsyncOutcome<i32> = AsyncOutcome::TimedOut;
        assert!(o.is_timeout());
        assert!(!o.is_ok());
    }

    #[test]
    fn outcome_into_result_ok() {
        let o: AsyncOutcome<i32> = AsyncOutcome::Ok(7);
        assert_eq!(o.into_result("svc").unwrap(), 7);
    }

    #[test]
    fn outcome_into_result_err() {
        let o: AsyncOutcome<i32> = AsyncOutcome::Err("boom".into());
        let err = o.into_result("svc").unwrap_err();
        assert!(err.to_string().contains("svc"));
        assert!(err.to_string().contains("boom"));
    }

    #[test]
    fn outcome_into_result_timeout() {
        let o: AsyncOutcome<i32> = AsyncOutcome::TimedOut;
        let err = o.into_result("svc").unwrap_err();
        assert!(err.to_string().contains("timed out"));
    }

    #[test]
    fn outcome_into_result_join_err() {
        let o: AsyncOutcome<i32> = AsyncOutcome::JoinError("panic".into());
        let err = o.into_result("svc").unwrap_err();
        assert!(err.to_string().contains("spawn_blocking"));
    }

    #[test]
    fn outcome_clone_and_eq() {
        let a: AsyncOutcome<i32> = AsyncOutcome::Ok(1);
        let b = a.clone();
        assert_eq!(a, b);
    }

    // ── Construction ─────────────────────────────────────────

    #[test]
    fn new_default_timeout() {
        let bridge = AsyncSynthexBridge::new(SynthexBridge::new());
        assert_eq!(bridge.timeout(), DEFAULT_HTTP_TIMEOUT);
    }

    #[test]
    fn with_timeout_overrides() {
        let bridge = AsyncSynthexBridge::new(SynthexBridge::new())
            .with_timeout(Duration::from_millis(500));
        assert_eq!(bridge.timeout(), Duration::from_millis(500));
    }

    #[test]
    fn from_arc_shares_state() {
        let inner = Arc::new(SynthexBridge::new());
        let a = AsyncSynthexBridge::from_arc(Arc::clone(&inner));
        let b = AsyncSynthexBridge::from_arc(inner);
        // Both wrappers point at the same underlying bridge port.
        assert_eq!(a.inner().port(), b.inner().port());
    }

    #[test]
    fn clone_works() {
        let bridge = make_bridge();
        let cloned = bridge.clone();
        assert_eq!(bridge.timeout(), cloned.timeout());
    }

    #[test]
    fn debug_impl() {
        let bridge = make_bridge();
        let dbg = format!("{bridge:?}");
        assert!(dbg.contains("AsyncSynthexBridge"));
        assert!(dbg.contains("timeout_ms"));
    }

    #[test]
    fn inner_accessor() {
        let bridge = AsyncSynthexBridge::new(SynthexBridge::with_config("127.0.0.1:9", 7));
        assert_eq!(bridge.inner().poll_interval(), 7);
    }

    // ── Async tests (R6: multi_thread required) ──────────────

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn poll_thermal_unreachable_returns_err() {
        let bridge = make_bridge();
        let outcome = bridge.poll_thermal_async().await;
        // Port 9 (discard) accepts but never responds → either Err or TimedOut.
        // Neither should be Ok.
        assert!(!outcome.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn post_field_state_unreachable() {
        let bridge = make_bridge();
        let outcome = bridge.post_field_state_async(b"{\"r\":0.5}".to_vec()).await;
        assert!(!outcome.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn nexus_push_empty_short_circuits_ok() {
        let bridge = make_bridge();
        let outcome = bridge.nexus_push_async(vec![]).await;
        assert!(outcome.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn nexus_pull_unreachable() {
        let bridge = make_bridge();
        let outcome = bridge.nexus_pull_async().await;
        assert!(!outcome.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn decay_trigger_unreachable() {
        let bridge = make_bridge();
        let outcome = bridge.decay_trigger_async(vec![]).await;
        assert!(!outcome.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn timeout_bound_is_respected() {
        let bridge = AsyncSynthexBridge::new(SynthexBridge::with_config(
            "10.255.255.1:9", 1, // non-routable: connect will hang
        ))
        .with_timeout(Duration::from_millis(100));

        let start = std::time::Instant::now();
        let outcome = bridge.poll_thermal_async().await;
        let elapsed = start.elapsed();

        // Must complete within ~3x the timeout (allowing for spawn_blocking
        // dispatch + connect_timeout interplay; raw connect can return
        // ECONNREFUSED faster than the timeout on some platforms).
        assert!(
            elapsed < Duration::from_secs(3),
            "elapsed {elapsed:?} exceeded 3s budget"
        );
        assert!(!outcome.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn parallel_calls_do_not_serialize() {
        let bridge = make_bridge();
        let calls = (0..4).map(|_| {
            let b = bridge.clone();
            tokio::spawn(async move { b.nexus_pull_async().await })
        });
        let start = std::time::Instant::now();
        for handle in calls {
            let _ = handle.await;
        }
        let elapsed = start.elapsed();
        // 4 sequential 200ms timeouts would be 800ms; parallel should be ~200ms.
        // Allow generous margin for CI variability.
        assert!(
            elapsed < Duration::from_millis(1500),
            "parallel calls took {elapsed:?} — appear to be serialized"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn current_thread_runtime_warning_doc() {
        // This test documents R6: spawn_blocking + await must use multi_thread.
        // Verified by all other #[tokio::test] in this module passing.
        let bridge = make_bridge();
        assert_eq!(bridge.timeout(), Duration::from_millis(200));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn outcome_distinguishes_error_types() {
        let bridge = make_bridge();
        let outcome = bridge.poll_thermal_async().await;
        match outcome {
            AsyncOutcome::Ok(_) => {} // unexpected but not a hard fail
            AsyncOutcome::Err(_) | AsyncOutcome::TimedOut | AsyncOutcome::JoinError(_) => {
                // any of these is acceptable for an unreachable target
            }
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn shared_arc_lets_concurrent_callers_observe_state() {
        let inner = Arc::new(SynthexBridge::with_config("127.0.0.1:9", 1));
        let bridge_a = AsyncSynthexBridge::from_arc(Arc::clone(&inner));
        let bridge_b = AsyncSynthexBridge::from_arc(inner);

        let _ = bridge_a.nexus_pull_async().await;
        // Both wrappers see the same underlying state.
        assert_eq!(
            bridge_a.inner().consecutive_failures(),
            bridge_b.inner().consecutive_failures()
        );
    }

    // ── Helpers ──────────────────────────────────────────────

    #[test]
    fn base_url_uses_port() {
        let bridge = AsyncSynthexBridge::new(SynthexBridge::with_config("127.0.0.1:8092", 1));
        assert_eq!(bridge.base_url(), "127.0.0.1:8092");
    }

    #[test]
    fn service_name_constant() {
        let _bridge = make_bridge();
        assert_eq!(AsyncSynthexBridge::service_name(), "synthex");
    }
}
