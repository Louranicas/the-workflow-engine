//! Subscription-applied state machine + the synchronous
//! `register_narrowed_consumer` entry point.
//!
//! Per m2 spec § 5: connection is built via the SDK's
//! `DbConnection::builder()`, the `on_connect` closure invokes the
//! `register_consumer` reducer + subscribes to the two narrowed queries,
//! and `conn.run_threaded()` spins up the SDK worker thread. The applied
//! handshake is awaited on an `mpsc::channel::<()>` driven by
//! `on_applied`.
//!
//! The SpacetimeDB SDK pattern is callback-based with synchronous
//! `run_threaded()` — not tokio async despite m2 spec § 2 mentioning a
//! tokio runtime. The `tokio` dep in Cargo.toml is required transitively
//! by `tokio-tungstenite`. m2 itself is sync.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::error::StcortexConsumerError;
use super::identity::ConsumerIdentity;
use super::module_bindings::{register_consumer, DbConnection};

/// SpacetimeDB WebSocket endpoint for the stcortex instance.
pub const STCORTEX_URI: &str = "ws://127.0.0.1:3000";

/// SpacetimeDB module name.
pub const STCORTEX_DB: &str = "stcortex";

/// Default `on_applied` timeout (ms).
pub const DEFAULT_SUBSCRIPTION_TIMEOUT_MS: u64 = 5_000;

/// Build the narrowed `tool_call` subscription SQL.
///
/// The `namespace` argument is expected to be the
/// [`WORKFLOW_TRACE_PREFIX`](super::identity::WORKFLOW_TRACE_PREFIX) (or
/// a `workflow_trace_*` namespace already validated via
/// [`crate::m9_watcher_namespace_guard::assert_workflow_trace_namespace`]).
/// As a defense-in-depth measure against accidental call-site drift, any
/// single-quote character in `namespace` is stripped — single-quote is
/// the only SpacetimeDB SQL string delimiter, so removing it neutralises
/// quote-injection while preserving every legal `workflow_trace_*` rune.
#[must_use]
pub fn tool_call_query(namespace: &str) -> String {
    // rationale: Adversarial-input discipline. `namespace` is `&str` so
    // a hypothetical caller could supply `"x' OR 1=1; --"` and we'd
    // emit broken SQL. Strip the only delimiter that matters; the m9
    // validator does the structural work upstream.
    let sanitised: String = namespace.chars().filter(|c| *c != '\'').collect();
    format!("SELECT * FROM tool_call WHERE namespace LIKE '{sanitised}_%'")
}

/// Build the narrowed `consumption_event` subscription SQL.
///
/// Note: `consumption_event` rows do not carry the `namespace` field
/// directly — the narrowing happens by `consumer_name` matching the
/// registered consumer. We subscribe to the full table and filter
/// downstream at the row callback boundary. Day-1 keeps the SQL simple.
#[must_use]
pub fn consumption_event_query() -> String {
    "SELECT * FROM consumption_event".to_owned()
}

/// Live registration handle: holds the running SDK connection + the
/// flag set by the subscription `on_applied` callback. m13 reads
/// `is_fresh()` before every write.
pub struct RegistrationHandle {
    identity: ConsumerIdentity,
    registered_at: Instant,
    applied_flag: Arc<AtomicBool>,
    /// `true` when the SDK's `on_disconnect` callback has fired since
    /// registration. **H1 stale-fresh fix:** `applied_flag` is cleared
    /// when this flips to true, so `is_fresh()` reports `false` after
    /// any WebSocket drop until a fresh handshake re-applies the
    /// subscription.
    disconnected_flag: Arc<AtomicBool>,
    // Holding the connection keeps the SDK worker thread alive.
    _conn: DbConnection,
}

/// Triple-state view of a [`RegistrationHandle`]'s liveness.
///
/// Additive surface (H1) — `is_fresh()` keeps its boolean signature for
/// the m13 write-gate hot path; callers that want finer granularity use
/// [`RegistrationHandle::status`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistrationStatus {
    /// Subscription has applied and the WebSocket is still connected.
    Fresh,
    /// `on_disconnect` fired since the last apply; the handle MUST be
    /// considered stale until a fresh handshake re-applies.
    Disconnected,
    /// `on_applied` never fired (e.g., synchronous-path bypass in tests).
    Stale,
}

impl std::fmt::Debug for RegistrationHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegistrationHandle")
            .field("identity", &self.identity)
            .field("registered_at", &self.registered_at)
            .field("applied", &self.applied_flag.load(Ordering::Relaxed))
            .field(
                "disconnected",
                &self.disconnected_flag.load(Ordering::Relaxed),
            )
            .finish_non_exhaustive()
    }
}

impl RegistrationHandle {
    /// `true` once the subscription has applied AND the WebSocket
    /// connection has not since been dropped. m13 calls this before
    /// every write attempt — refuse-write at the DB layer reads the same
    /// invariant.
    ///
    /// **H1 stale-fresh fix:** prior versions only checked
    /// `applied_flag`; if the SDK fired `on_disconnect`, `is_fresh()`
    /// would still return `true` until the SDK was dropped — m13 would
    /// happily write against a dead consumer. The disconnect handler now
    /// clears `applied_flag` via the second `Arc<AtomicBool>` clone, so
    /// the contract `is_fresh()` reports is the actual live state.
    #[must_use]
    pub fn is_fresh(&self) -> bool {
        // Both flags consulted: defence-in-depth. If a phantom
        // on_applied somehow fires after on_disconnect without a fresh
        // handshake (SDK pathology), the disconnected_flag still gates.
        self.applied_flag.load(Ordering::Acquire)
            && !self.disconnected_flag.load(Ordering::Acquire)
    }

    /// Triple-state view of the handle's liveness.
    ///
    /// Useful for callers that need to distinguish "never applied" from
    /// "applied then dropped". m13's hot path keeps using
    /// [`Self::is_fresh`].
    #[must_use]
    pub fn status(&self) -> RegistrationStatus {
        if self.disconnected_flag.load(Ordering::Acquire) {
            RegistrationStatus::Disconnected
        } else if self.applied_flag.load(Ordering::Acquire) {
            RegistrationStatus::Fresh
        } else {
            RegistrationStatus::Stale
        }
    }

    /// Borrow the registered identity.
    #[must_use]
    pub fn identity(&self) -> &ConsumerIdentity {
        &self.identity
    }

    /// Time since registration.
    #[must_use]
    pub fn age(&self) -> Duration {
        self.registered_at.elapsed()
    }

}

/// Test-only freshness state machine — exposes the same applied /
/// disconnected boolean pair as a [`RegistrationHandle`], without
/// owning a live `DbConnection`. Lets us exercise the H1 stale-fresh
/// contract under thread-pool interleavings that would otherwise need a
/// real WebSocket. Not part of the public API.
#[cfg(test)]
pub(crate) struct FreshnessProbe {
    applied_flag: Arc<AtomicBool>,
    disconnected_flag: Arc<AtomicBool>,
}

#[cfg(test)]
impl FreshnessProbe {
    /// Construct in the same initial state as a freshly-built handle —
    /// neither applied nor disconnected.
    pub(crate) fn new() -> Self {
        Self {
            applied_flag: Arc::new(AtomicBool::new(false)),
            disconnected_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Simulate the SDK's `on_applied` callback firing.
    pub(crate) fn simulate_on_applied(&self) {
        self.applied_flag.store(true, Ordering::Release);
    }

    /// Simulate the SDK's `on_disconnect` callback firing — mirrors the
    /// production-side handler (clears applied, sets disconnected).
    pub(crate) fn simulate_on_disconnect(&self) {
        self.disconnected_flag.store(true, Ordering::Release);
        self.applied_flag.store(false, Ordering::Release);
    }

    /// Mirror of [`RegistrationHandle::is_fresh`].
    pub(crate) fn is_fresh(&self) -> bool {
        self.applied_flag.load(Ordering::Acquire)
            && !self.disconnected_flag.load(Ordering::Acquire)
    }

    /// Mirror of [`RegistrationHandle::status`].
    pub(crate) fn status(&self) -> RegistrationStatus {
        if self.disconnected_flag.load(Ordering::Acquire) {
            RegistrationStatus::Disconnected
        } else if self.applied_flag.load(Ordering::Acquire) {
            RegistrationStatus::Fresh
        } else {
            RegistrationStatus::Stale
        }
    }
}

/// Register as a narrowed stcortex consumer and wait for the
/// subscription to apply.
///
/// Synchronous; spins up the SDK worker thread internally. Returns a
/// [`RegistrationHandle`] whose `is_fresh()` flips `true` once the SDK's
/// `on_applied` callback fires.
///
/// # Errors
///
/// - [`StcortexConsumerError::ConnectionFailed`] if the WebSocket
///   handshake fails.
/// - [`StcortexConsumerError::RegisterFailed`] if the `register_consumer`
///   reducer rejects the request.
/// - [`StcortexConsumerError::SubscriptionTimeout`] if `on_applied`
///   does not fire within `timeout_ms`.
#[allow(
    clippy::too_many_lines,
    reason = "End-to-end SDK orchestration: builder-pattern + on_connect/on_applied/on_disconnect closures with captured state form a single causally-ordered sequence per m2 spec § 5. Splitting the on_connect closure into a helper would scatter the H1 fresh/disconnect Arc<AtomicBool> wiring across two surfaces and obscure the single-glance audit of the callback graph. The 105-line count is mechanical (5 over the 100-line lint default) rather than a complexity smell."
)]
pub fn register_narrowed_consumer(
    identity: ConsumerIdentity,
    timeout_ms: u64,
) -> Result<RegistrationHandle, StcortexConsumerError> {
    use spacetimedb_sdk::DbContext;

    let (tx, rx) = mpsc::channel::<()>();
    let tx = Mutex::new(Some(tx));
    let applied_flag = Arc::new(AtomicBool::new(false));
    let applied_for_callback = Arc::clone(&applied_flag);
    // H1 stale-fresh fix: a SECOND clone of applied_flag is passed into
    // on_disconnect so the SDK can clear the freshness gate when the
    // WebSocket drops; the disconnected_flag is the durable "we saw a
    // drop" sentinel for status() observers.
    let applied_for_disconnect = Arc::clone(&applied_flag);
    let disconnected_flag = Arc::new(AtomicBool::new(false));
    let disconnected_for_callback = Arc::clone(&disconnected_flag);
    // rationale: capture any `register_consumer` reducer error so the
    // outer call surfaces it as `RegisterFailed` rather than letting
    // `on_applied` silently flip `is_fresh = true` for a consumer the
    // server has refused (was: silent failure swallowed by tracing::error
    // alone — see debugger Phase 1 finding m2-F2).
    let register_error: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let register_error_for_callback = Arc::clone(&register_error);

    let identity_for_callback = identity.clone();
    let consumer_name = identity.name.as_str().to_owned();
    let namespace = identity.namespace.as_str().to_owned();
    let transport = identity.transport.as_str().to_owned();
    let namespace_for_query = namespace.clone();

    let conn = DbConnection::builder()
        .with_uri(STCORTEX_URI)
        .with_database_name(STCORTEX_DB)
        .on_connect(move |ctx, _identity, _token| {
            // 1) Register as a consumer (idempotent on stcortex side).
            if let Err(e) = ctx.reducers.register_consumer(
                consumer_name.clone(),
                namespace.clone(),
                transport.clone(),
            ) {
                let reason = e.to_string();
                tracing::error!(
                    target: "m2.register_consumer",
                    error = %reason,
                    "register_consumer reducer call failed"
                );
                if let Ok(mut slot) = register_error_for_callback.lock() {
                    *slot = Some(reason);
                }
            }
            // 2) Subscribe to the two narrowed queries.
            let q_tool_call = tool_call_query(&namespace_for_query);
            let q_consumption = consumption_event_query();
            let applied_inner = Arc::clone(&applied_for_callback);
            let tx_inner = Mutex::new(tx.lock().ok().and_then(|mut g| g.take()));
            ctx.subscription_builder()
                .on_applied(move |_ctx| {
                    applied_inner.store(true, Ordering::Release);
                    if let Ok(mut g) = tx_inner.lock() {
                        if let Some(tx) = g.take() {
                            // rationale: best-effort signal — the receiver
                            // may have already timed out and dropped the
                            // mpsc::Receiver; that's not an error from this
                            // side, the applied_flag carries the truth.
                            let _ = tx.send(());
                        }
                    }
                    tracing::info!(
                        target: "m2.subscription.applied",
                        "narrowed subscription applied"
                    );
                })
                .on_error(|_ctx, err| {
                    tracing::error!(
                        target: "m2.subscription.error",
                        error = ?err,
                        "subscription error"
                    );
                })
                .subscribe([q_tool_call, q_consumption]);
            tracing::info!(
                target: "m2.consumer.identity",
                name = %identity_for_callback.name,
                namespace = %identity_for_callback.namespace,
                "m2 consumer registered"
            );
        })
        .on_connect_error(|_ctx, err| {
            tracing::error!(
                target: "m2.connect.error",
                error = ?err,
                "stcortex connection failed"
            );
        })
        .on_disconnect(move |_ctx, err| {
            // H1 stale-fresh fix: prior to S1002600 Wave-A1 this
            // handler was tracing-only — `applied_flag` was never
            // cleared, so `is_fresh()` returned a STALE `true` after
            // any WebSocket drop and m13 wrote against a dead
            // consumer. Both flags are stored with Release ordering
            // so a subsequent Acquire load by m13 sees the new state.
            disconnected_for_callback.store(true, Ordering::Release);
            applied_for_disconnect.store(false, Ordering::Release);
            tracing::warn!(
                target: "m2.connect.disconnect",
                error = ?err,
                "stcortex disconnected — applied_flag cleared, handle now reports !is_fresh"
            );
        })
        .build()
        .map_err(|e| StcortexConsumerError::ConnectionFailed {
            uri: STCORTEX_URI.to_owned(),
            reason: e.to_string(),
        })?;

    // Spin up the SDK worker thread.
    conn.run_threaded();

    match rx.recv_timeout(Duration::from_millis(timeout_ms)) {
        Ok(()) => {
            // rationale: even if `on_applied` fired, the
            // `register_consumer` reducer may have refused the request.
            // Surface that as a typed RegisterFailed *before* returning a
            // misleading "fresh" handle. Anti-silent-failure discipline.
            if let Ok(slot) = register_error.lock() {
                if let Some(reason) = slot.as_ref() {
                    return Err(StcortexConsumerError::RegisterFailed(reason.clone()));
                }
            }
            Ok(RegistrationHandle {
                identity,
                registered_at: Instant::now(),
                applied_flag,
                disconnected_flag,
                _conn: conn,
            })
        }
        Err(_) => Err(StcortexConsumerError::SubscriptionTimeout { timeout_ms }),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        consumption_event_query, tool_call_query, FreshnessProbe, RegistrationStatus,
        DEFAULT_SUBSCRIPTION_TIMEOUT_MS, STCORTEX_DB, STCORTEX_URI,
    };

    #[test]
    fn stcortex_uri_is_loopback_websocket() {
        assert_eq!(STCORTEX_URI, "ws://127.0.0.1:3000");
    }

    #[test]
    fn stcortex_db_constant_is_stcortex() {
        assert_eq!(STCORTEX_DB, "stcortex");
    }

    #[test]
    fn default_subscription_timeout_is_five_seconds() {
        assert_eq!(DEFAULT_SUBSCRIPTION_TIMEOUT_MS, 5_000);
    }

    #[test]
    fn tool_call_query_contains_namespace_like_clause() {
        use super::super::identity::WORKFLOW_TRACE_PREFIX;
        let q = tool_call_query(WORKFLOW_TRACE_PREFIX);
        assert!(q.contains("SELECT * FROM tool_call"));
        assert!(q.contains("LIKE"));
        assert!(q.contains(WORKFLOW_TRACE_PREFIX));
        assert!(q.contains("_%"));
    }

    #[test]
    fn consumption_event_query_is_simple_select() {
        let q = consumption_event_query();
        assert_eq!(q, "SELECT * FROM consumption_event");
    }

    #[test]
    fn tool_call_query_excludes_foreign_namespace_tables() {
        // W1 narrowing invariant: queries reference exactly `tool_call`
        // / `consumption_event`. No pathway / memory / ghost_memory.
        use super::super::identity::WORKFLOW_TRACE_PREFIX;
        let q = tool_call_query(WORKFLOW_TRACE_PREFIX);
        assert!(!q.contains("pathway"));
        assert!(!q.contains("memory"));
        assert!(!q.contains("ghost_memory"));
    }

    // ====================================================================
    // H1 stale-fresh fix (S1002600 Wave-A1) — RegistrationHandle's
    // freshness state machine, exercised via FreshnessProbe (mirrors the
    // applied/disconnected atomic-flag pair without owning a live SDK
    // DbConnection).
    // ====================================================================

    // rationale: Anti-property (H1) — `is_fresh()` MUST return false
    // after a disconnect event, even if `on_applied` had fired
    // beforehand. Pre-fix the disconnect handler was tracing-only and
    // applied_flag stayed `true`.
    #[test]
    fn is_fresh_returns_false_after_disconnect() {
        // rationale: Anti-property (H1 stale-fresh regression)
        let probe = FreshnessProbe::new();
        probe.simulate_on_applied();
        assert!(probe.is_fresh(), "applied should report fresh");
        probe.simulate_on_disconnect();
        assert!(
            !probe.is_fresh(),
            "post-disconnect: is_fresh MUST return false (H1 fix)"
        );
        assert_eq!(probe.status(), RegistrationStatus::Disconnected);
    }

    // rationale: Atomicity contract (H1) — a phantom on_applied that
    // fires AFTER on_disconnect, with no fresh handshake in between,
    // MUST NOT resurrect `is_fresh = true`. The disconnected_flag is
    // sticky until a new RegistrationHandle is constructed.
    #[test]
    fn disconnect_then_reapplied_does_not_resurrect_fresh_without_fresh_handshake() {
        // rationale: Atomicity contract (H1 stickiness invariant)
        let probe = FreshnessProbe::new();
        probe.simulate_on_applied();
        probe.simulate_on_disconnect();
        // Pathological replay — SDK fires on_applied a second time
        // without a fresh connection. The disconnected_flag must still
        // gate; m13 must not be tricked into writing.
        probe.simulate_on_applied();
        assert!(
            !probe.is_fresh(),
            "phantom on_applied after disconnect MUST NOT resurrect fresh"
        );
        assert_eq!(
            probe.status(),
            RegistrationStatus::Disconnected,
            "disconnected_flag is sticky across phantom on_applied"
        );
    }

    // rationale: Contract — initial state is Stale (neither applied nor
    // disconnected). Drift detection for the state-machine invariant.
    #[test]
    fn initial_state_is_stale_not_fresh() {
        // rationale: Contract regression (state-machine initial state)
        let probe = FreshnessProbe::new();
        assert!(!probe.is_fresh(), "initial state is NOT fresh");
        assert_eq!(probe.status(), RegistrationStatus::Stale);
    }

    // rationale: Triple-state surface — the additive `status()` method
    // distinguishes Stale (never applied) from Disconnected (was applied
    // then dropped). H1 fix surface contract.
    #[test]
    fn status_triple_state_surface_distinguishes_stale_from_disconnected() {
        // rationale: Triple-state contract (additive H1 surface)
        let probe = FreshnessProbe::new();
        assert_eq!(probe.status(), RegistrationStatus::Stale);
        probe.simulate_on_applied();
        assert_eq!(probe.status(), RegistrationStatus::Fresh);
        probe.simulate_on_disconnect();
        assert_eq!(probe.status(), RegistrationStatus::Disconnected);
    }
}
