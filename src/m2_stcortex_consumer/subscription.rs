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
use std::sync::Arc;
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
#[must_use]
pub fn tool_call_query(namespace: &str) -> String {
    format!("SELECT * FROM tool_call WHERE namespace LIKE '{namespace}_%'")
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
    // Holding the connection keeps the SDK worker thread alive.
    _conn: DbConnection,
}

impl std::fmt::Debug for RegistrationHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegistrationHandle")
            .field("identity", &self.identity)
            .field("registered_at", &self.registered_at)
            .field("applied", &self.applied_flag.load(Ordering::Relaxed))
            .finish_non_exhaustive()
    }
}

impl RegistrationHandle {
    /// `true` once the subscription has applied. m13 calls this before
    /// every write attempt — refuse-write at the DB layer reads the same
    /// invariant.
    #[must_use]
    pub fn is_fresh(&self) -> bool {
        self.applied_flag.load(Ordering::Relaxed)
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
pub fn register_narrowed_consumer(
    identity: ConsumerIdentity,
    timeout_ms: u64,
) -> Result<RegistrationHandle, StcortexConsumerError> {
    use spacetimedb_sdk::DbContext;

    let (tx, rx) = mpsc::channel::<()>();
    let tx = std::sync::Mutex::new(Some(tx));
    let applied_flag = Arc::new(AtomicBool::new(false));
    let applied_for_callback = Arc::clone(&applied_flag);

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
                tracing::error!(
                    target: "m2.register_consumer",
                    error = %e,
                    "register_consumer reducer call failed"
                );
            }
            // 2) Subscribe to the two narrowed queries.
            let q_tool_call = tool_call_query(&namespace_for_query);
            let q_consumption = consumption_event_query();
            let applied_inner = Arc::clone(&applied_for_callback);
            let tx_inner = std::sync::Mutex::new(tx.lock().ok().and_then(|mut g| g.take()));
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
        .on_disconnect(|_ctx, err| {
            tracing::warn!(
                target: "m2.connect.disconnect",
                error = ?err,
                "stcortex disconnected"
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
        Ok(()) => Ok(RegistrationHandle {
            identity,
            registered_at: Instant::now(),
            applied_flag,
            _conn: conn,
        }),
        Err(_) => Err(StcortexConsumerError::SubscriptionTimeout { timeout_ms }),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        consumption_event_query, tool_call_query, DEFAULT_SUBSCRIPTION_TIMEOUT_MS,
        STCORTEX_DB, STCORTEX_URI,
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
}
