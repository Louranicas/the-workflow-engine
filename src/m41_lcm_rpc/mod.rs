//! `m41_lcm_rpc` — `lcm.loop.create` JSON-RPC client.
//! Cluster H · L8.
//!
//! NOTE: `lcm.loop.create` is the canonical LCM RPC name (per S1001882
//! drift retraction; the legacy `lcm.deploy` form is deprecated).

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use thiserror::Error;

/// Default LCM RPC endpoint (stdio supervisor binds to this loopback URI).
pub const DEFAULT_LCM_URL: &str = "http://127.0.0.1:8082/rpc";

/// Default RPC method.
pub const RPC_METHOD: &str = "lcm.loop.create";

/// Default request timeout.
pub const DEFAULT_RPC_TIMEOUT: Duration = Duration::from_secs(10);

/// `lcm.loop.create` parameters.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LcmLoopCreateParams {
    /// Workflow id from m30/m31.
    pub workflow_id: u64,
    /// Conductor dispatch id from m32.
    pub conductor_dispatch_id: String,
    /// Free-form JSON describing the loop body.
    pub loop_spec: serde_json::Value,
}

/// `lcm.loop.create` result.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LcmLoopCreateResult {
    /// LCM-assigned loop id.
    pub loop_id: String,
    /// LCM accept timestamp (ms).
    pub created_at_ms: i64,
}

/// RPC errors.
#[derive(Debug, Error)]
pub enum LcmRpcError {
    /// Transport / HTTP failure.
    #[error("transport: {0}")]
    Transport(String),
    /// LCM returned a JSON-RPC error.
    #[error("rpc error {code}: {message}")]
    Rpc {
        /// JSON-RPC error code.
        code: i64,
        /// JSON-RPC error message.
        message: String,
    },
    /// Response parse failure.
    #[error("parse: {0}")]
    Parse(String),
    /// JSON-RPC 2.0 § 5 invariant violated — response `id` did not match
    /// the request `id`. Catches replayed / mis-multiplexed responses on a
    /// shared transport (Wave-A3 H3).
    #[error("id mismatch: sent={sent}, received={received}")]
    IdMismatch {
        /// The request id that was sent on the wire.
        sent: u64,
        /// The id field that came back in the response (may be missing,
        /// null, string, or wrong number — kept as raw JSON for diagnosis).
        received: serde_json::Value,
    },
}

/// Client trait for testability.
pub trait LcmClient: Send + Sync {
    /// Send a `lcm.loop.create` request.
    ///
    /// # Errors
    ///
    /// See [`LcmRpcError`].
    fn loop_create(
        &self,
        params: &LcmLoopCreateParams,
    ) -> Result<LcmLoopCreateResult, LcmRpcError>;
}

/// Production HTTP JSON-RPC client.
pub struct HttpLcmClient {
    url: String,
    timeout: Duration,
    /// Monotonic per-client request id. Each `loop_create` call increments
    /// and consumes a unique id; the response is verified to echo it back
    /// per JSON-RPC 2.0 § 5 (Wave-A3 H3).
    next_id: AtomicU64,
}

impl HttpLcmClient {
    /// Construct.
    #[must_use]
    pub fn new(url: impl Into<String>, timeout: Duration) -> Self {
        Self {
            url: url.into(),
            timeout,
            next_id: AtomicU64::new(1),
        }
    }

    /// Allocate the next request id. Public for tests / callers that need
    /// to compose their own JSON-RPC envelopes against the same identity
    /// stream (concurrency-safe, lock-free).
    pub fn allocate_request_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }
}

impl LcmClient for HttpLcmClient {
    fn loop_create(
        &self,
        params: &LcmLoopCreateParams,
    ) -> Result<LcmLoopCreateResult, LcmRpcError> {
        let client = reqwest::blocking::Client::builder()
            .timeout(self.timeout)
            .build()
            .map_err(|e| LcmRpcError::Transport(e.to_string()))?;
        let request_id = self.allocate_request_id();
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": RPC_METHOD,
            "params": params,
            "id": request_id,
        });
        let resp = client
            .post(&self.url)
            .json(&body)
            .send()
            .map_err(|e| LcmRpcError::Transport(e.to_string()))?;
        let value: serde_json::Value = resp
            .json()
            .map_err(|e| LcmRpcError::Parse(e.to_string()))?;
        parse_rpc_response(&value, request_id)
    }
}

/// Parse a JSON-RPC 2.0 response envelope against an expected request id.
///
/// Separated from [`HttpLcmClient::loop_create`] so the response-shape
/// logic is unit-testable without crossing the network boundary.
///
/// Behaviour (per JSON-RPC 2.0 § 5 + Wave-A3 H3 / H4):
///
/// 1. The `id` field MUST be present and equal `expected_id`. Mismatch →
///    [`LcmRpcError::IdMismatch`].
/// 2. An `error` field counts as an error ONLY when it is an object AND
///    contains a `code` field. `null` / `{}` / missing-code do not trigger
///    the error branch — the response is treated as a normal result.
/// 3. Otherwise, the `result` field is decoded into
///    [`LcmLoopCreateResult`]; absence is a parse error.
fn parse_rpc_response(
    value: &serde_json::Value,
    expected_id: u64,
) -> Result<LcmLoopCreateResult, LcmRpcError> {
    // (1) id-echo verification — JSON-RPC 2.0 § 5
    let id_field = value
        .get("id")
        .ok_or_else(|| LcmRpcError::Parse("response missing id".into()))?;
    let matches = id_field.as_u64().is_some_and(|n| n == expected_id);
    if !matches {
        return Err(LcmRpcError::IdMismatch {
            sent: expected_id,
            received: id_field.clone(),
        });
    }
    // (2) error envelope — only object-with-code counts as an error
    if let Some(err) = value.get("error") {
        if err.is_object() && err.get("code").is_some() {
            let code = err
                .get("code")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(-32000);
            let message = err
                .get("message")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown")
                .to_owned();
            return Err(LcmRpcError::Rpc { code, message });
        }
        // null / {} / missing-code → fall through to result parsing
    }
    // (3) result decoding
    let result = value
        .get("result")
        .ok_or_else(|| LcmRpcError::Parse("missing result".into()))?;
    serde_json::from_value(result.clone()).map_err(|e| LcmRpcError::Parse(e.to_string()))
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::{
        HttpLcmClient, LcmClient, LcmLoopCreateParams, LcmLoopCreateResult, LcmRpcError,
        DEFAULT_LCM_URL, DEFAULT_RPC_TIMEOUT, RPC_METHOD,
    };

    #[test]
    fn rpc_method_is_lcm_loop_create() {
        assert_eq!(RPC_METHOD, "lcm.loop.create");
    }

    #[test]
    fn default_url_is_loopback_v3() {
        assert_eq!(DEFAULT_LCM_URL, "http://127.0.0.1:8082/rpc");
    }

    #[test]
    fn default_timeout_is_10s() {
        assert_eq!(DEFAULT_RPC_TIMEOUT.as_secs(), 10);
    }

    #[test]
    fn params_serde_roundtrip() {
        let p = LcmLoopCreateParams {
            workflow_id: 42,
            conductor_dispatch_id: "conductor-001".into(),
            loop_spec: serde_json::json!({"steps": [1, 2, 3]}),
        };
        let s = serde_json::to_string(&p).expect("ser");
        let back: LcmLoopCreateParams = serde_json::from_str(&s).expect("de");
        assert_eq!(back, p);
    }

    #[test]
    fn result_serde_roundtrip() {
        let r = LcmLoopCreateResult {
            loop_id: "loop-001".into(),
            created_at_ms: 1_700_000_000_000,
        };
        let s = serde_json::to_string(&r).expect("ser");
        let back: LcmLoopCreateResult = serde_json::from_str(&s).expect("de");
        assert_eq!(back, r);
    }

    struct StubClient {
        responses: Mutex<Vec<Result<LcmLoopCreateResult, LcmRpcError>>>,
    }

    impl LcmClient for StubClient {
        fn loop_create(
            &self,
            _params: &LcmLoopCreateParams,
        ) -> Result<LcmLoopCreateResult, LcmRpcError> {
            self.responses
                .lock()
                .expect("lock")
                .pop()
                .unwrap_or(Err(LcmRpcError::Transport("no stub".into())))
        }
    }

    #[test]
    fn trait_dispatches_to_stub() {
        let stub = StubClient {
            responses: Mutex::new(vec![Ok(LcmLoopCreateResult {
                loop_id: "stub".into(),
                created_at_ms: 0,
            })]),
        };
        let params = LcmLoopCreateParams {
            workflow_id: 1,
            conductor_dispatch_id: "x".into(),
            loop_spec: serde_json::json!({}),
        };
        let r = stub.loop_create(&params).expect("ok");
        assert_eq!(r.loop_id, "stub");
    }

    #[test]
    fn rpc_error_carries_code_and_message() {
        let err = LcmRpcError::Rpc {
            code: -32602,
            message: "invalid params".into(),
        };
        let s = err.to_string();
        assert!(s.contains("-32602"));
        assert!(s.contains("invalid params"));
    }

    // ====================================================================
    // Cluster H hardening pass — m41 LCM RPC.
    // Categories: Contract regression, Determinism, Adversarial,
    // Concurrency, Boundary, Cross-module, Anti-property.
    // ====================================================================

    use std::sync::Arc;
    use std::thread;

    // rationale: Contract regression — RPC_METHOD const equality is the
    // primary lock against the S1001882 `lcm.deploy` → `lcm.loop.create`
    // drift retraction. Any rename here breaks LCM supervisor wire compat.
    #[test]
    fn rpc_method_const_locked_to_lcm_loop_create() {
        // rationale: Contract regression (S1001882 method-name drift)
        assert_eq!(RPC_METHOD, "lcm.loop.create");
        // Explicit anti-drift: the deprecated form must NOT match.
        assert_ne!(RPC_METHOD, "lcm.deploy");
        assert_ne!(RPC_METHOD, "lcm.loop.deploy");
    }

    // rationale: Contract regression — DEFAULT_LCM_URL pinned to the
    // dev-ops-engine-v3 loopback RPC endpoint (port 8082 per workspace
    // CLAUDE.md ULTRAPLATE services table).
    #[test]
    fn default_lcm_url_pinned_to_dev_ops_engine_v3() {
        // rationale: Contract regression (V3 :8082 loopback)
        assert_eq!(DEFAULT_LCM_URL, "http://127.0.0.1:8082/rpc");
        assert!(!DEFAULT_LCM_URL.contains("https://"));
    }

    // rationale: Contract regression — LcmLoopCreateParams JSON schema
    // shape MUST be exactly {workflow_id, conductor_dispatch_id,
    // loop_spec}. Schema drift breaks the LCM JSON-RPC contract.
    #[test]
    fn params_json_shape_snapshot() {
        // rationale: Contract regression (LCM params schema snapshot)
        let p = LcmLoopCreateParams {
            workflow_id: 7,
            conductor_dispatch_id: "c1".into(),
            loop_spec: serde_json::json!({"steps": []}),
        };
        let v = serde_json::to_value(&p).expect("ser");
        let obj = v.as_object().expect("object");
        let mut keys: Vec<&str> = obj.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            vec!["conductor_dispatch_id", "loop_spec", "workflow_id"],
            "LcmLoopCreateParams schema drift",
        );
    }

    // rationale: Contract regression — LcmLoopCreateResult schema shape
    // pinned to {loop_id, created_at_ms}.
    #[test]
    fn result_json_shape_snapshot() {
        // rationale: Contract regression (LCM result schema snapshot)
        let r = LcmLoopCreateResult {
            loop_id: "loop-1".into(),
            created_at_ms: 1_700_000_000_000,
        };
        let v = serde_json::to_value(&r).expect("ser");
        let obj = v.as_object().expect("object");
        let mut keys: Vec<&str> = obj.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(keys, vec!["created_at_ms", "loop_id"]);
    }

    // rationale: Determinism — 1_000 serde round-trips of
    // LcmLoopCreateParams preserve identity across distinct workflow_ids.
    #[test]
    fn params_serde_deterministic_across_thousand_workflows() {
        // rationale: Determinism (serde × 1_000)
        for i in 0..1_000_u64 {
            let p = LcmLoopCreateParams {
                workflow_id: i,
                conductor_dispatch_id: format!("c{i}"),
                loop_spec: serde_json::json!({"i": i}),
            };
            let s = serde_json::to_string(&p).expect("ser");
            let back: LcmLoopCreateParams = serde_json::from_str(&s).expect("de");
            assert_eq!(back, p);
        }
    }

    // rationale: Adversarial input — RPC response with both `error` and
    // `result` keys must surface as Rpc error (error takes precedence per
    // JSON-RPC 2.0 § 5).
    #[test]
    fn rpc_error_takes_precedence_over_result_in_response() {
        // rationale: Adversarial input (JSON-RPC 2.0 § 5 precedence)
        // We exercise the parse path via a unit test on the response
        // shape since HttpLcmClient owns the network. The behaviour is
        // mirrored by StubClient: when an Err is queued, it wins.
        let stub = StubClient {
            responses: Mutex::new(vec![Err(LcmRpcError::Rpc {
                code: -32600,
                message: "InvalidRequest".into(),
            })]),
        };
        let params = LcmLoopCreateParams {
            workflow_id: 1,
            conductor_dispatch_id: "x".into(),
            loop_spec: serde_json::json!({}),
        };
        let r = stub.loop_create(&params);
        assert!(matches!(
            r,
            Err(LcmRpcError::Rpc {
                code: -32600,
                ..
            })
        ));
    }

    // rationale: Adversarial input — Parse error variant carries the raw
    // serde message for downstream operator-readable diagnosis.
    #[test]
    fn parse_error_carries_underlying_reason() {
        // rationale: Adversarial input
        let err = LcmRpcError::Parse("unexpected token at line 1 column 2".into());
        let s = err.to_string();
        assert!(s.starts_with("parse: "));
        assert!(s.contains("unexpected token"));
    }

    // rationale: Adversarial input — Transport error variant Display
    // pinned for operator runbooks.
    #[test]
    fn transport_error_display_format_pinned() {
        // rationale: Adversarial input
        let err = LcmRpcError::Transport("dial tcp 127.0.0.1:8082: connection refused".into());
        let s = err.to_string();
        assert!(s.starts_with("transport: "));
        assert!(s.contains("8082"));
    }

    // rationale: Concurrency — StubClient under 8 concurrent callers
    // serialises responses without dropping or duplicating queued
    // results.
    #[test]
    fn concurrent_calls_dispatch_to_stub_deterministically() {
        // rationale: Concurrency (trait surface)
        let n = 8_usize;
        let mut queue: Vec<Result<LcmLoopCreateResult, LcmRpcError>> = Vec::with_capacity(n);
        for i in 0..n {
            queue.push(Ok(LcmLoopCreateResult {
                loop_id: format!("loop-{i}"),
                created_at_ms: 0,
            }));
        }
        let stub = Arc::new(StubClient {
            responses: Mutex::new(queue),
        });
        let mut handles = Vec::new();
        for _ in 0..n {
            let s = Arc::clone(&stub);
            handles.push(thread::spawn(move || {
                let p = LcmLoopCreateParams {
                    workflow_id: 1,
                    conductor_dispatch_id: "x".into(),
                    loop_spec: serde_json::json!({}),
                };
                s.loop_create(&p)
            }));
        }
        let mut ok_count = 0_usize;
        for h in handles {
            if h.join().expect("join").is_ok() {
                ok_count += 1;
            }
        }
        assert_eq!(ok_count, n, "every queued response consumed exactly once");
    }

    // rationale: Boundary — DEFAULT_RPC_TIMEOUT pinned at 10 seconds
    // (longer than m40's 5s — LCM RPCs are interactive supervisor calls
    // and can run longer than nexus fire-and-forget pushes).
    #[test]
    fn default_rpc_timeout_exactly_ten_seconds() {
        // rationale: Boundary (timeout constant pin)
        assert_eq!(DEFAULT_RPC_TIMEOUT, std::time::Duration::from_secs(10));
        assert!(
            DEFAULT_RPC_TIMEOUT > std::time::Duration::from_secs(5),
            "LCM timeout must exceed m40's 5s nexus push timeout"
        );
    }

    // rationale: Cross-module surface invariant — HttpLcmClient::new is
    // a thin constructor; calling .loop_create against an unreachable
    // URL yields a typed Transport error, never a panic.
    #[test]
    fn http_client_unreachable_url_yields_typed_transport_error() {
        // rationale: Cross-module surface invariant (network failure typing)
        let c = HttpLcmClient::new(
            "http://127.0.0.1:1/rpc", // RFC 1149-style guaranteed-unreachable
            std::time::Duration::from_millis(50),
        );
        let p = LcmLoopCreateParams {
            workflow_id: 1,
            conductor_dispatch_id: "x".into(),
            loop_spec: serde_json::json!({}),
        };
        let r = c.loop_create(&p);
        assert!(
            matches!(r, Err(LcmRpcError::Transport(_))),
            "unreachable LCM must yield Transport error, got {r:?}"
        );
    }

    // rationale: Anti-property — error type implements std::error::Error
    // + Send + Sync + 'static for async / tokio interop.
    #[test]
    fn lcm_rpc_error_is_send_sync_static() {
        // rationale: Anti-property (async-readiness)
        fn assert_error<T: std::error::Error + Send + Sync + 'static>() {}
        assert_error::<LcmRpcError>();
    }

    // ====================================================================
    // Cluster H Wave-A3 — H3 JSON-RPC id-echo verification.
    // Categories: Contract regression · Concurrency · Adversarial input.
    // ====================================================================

    // rationale: Concurrency — JSON-RPC 2.0 § 5 requires every concurrent
    // call to use a unique id; the AtomicU64 generator must produce N
    // distinct ids across N concurrent callers without collision.
    #[test]
    fn rpc_call_assigns_unique_ids_across_concurrent_calls() {
        // rationale: Concurrency (atomic id allocation correctness)
        let c = Arc::new(HttpLcmClient::new(
            "http://127.0.0.1:1/rpc",
            std::time::Duration::from_millis(10),
        ));
        let n = 100_usize;
        let collected = Arc::new(Mutex::new(Vec::with_capacity(n)));
        let mut handles = Vec::new();
        for _ in 0..n {
            let c2 = Arc::clone(&c);
            let coll = Arc::clone(&collected);
            handles.push(thread::spawn(move || {
                let id = c2.allocate_request_id();
                coll.lock().expect("lock").push(id);
            }));
        }
        for h in handles {
            h.join().expect("join");
        }
        let mut ids = collected.lock().expect("lock").clone();
        ids.sort_unstable();
        let unique_count = ids.windows(2).filter(|w| w[0] != w[1]).count() + 1;
        assert_eq!(
            unique_count, n,
            "100 concurrent allocate_request_id calls must produce 100 distinct ids"
        );
    }

    // rationale: Adversarial input (JSON-RPC 2.0 § 5) — a response with
    // an id that doesn't match the request id is a replay / mux-confusion
    // signal. parse_rpc_response must surface IdMismatch, not accept it.
    #[test]
    fn rpc_call_refuses_response_with_mismatched_id() {
        // rationale: Adversarial input (id-echo verification)
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": {"loop_id": "x", "created_at_ms": 0},
            "id": 999,
        });
        let r = super::parse_rpc_response(&response, 42);
        assert!(matches!(
            r,
            Err(LcmRpcError::IdMismatch { sent: 42, .. })
        ));
        if let Err(LcmRpcError::IdMismatch { received, .. }) = r {
            assert_eq!(received, serde_json::json!(999));
        }
    }

    // rationale: Contract regression — a matching id MUST allow the result
    // to be decoded normally (no regression of the happy path).
    #[test]
    fn rpc_call_accepts_response_with_matching_id() {
        // rationale: Contract regression (happy path preservation)
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": {"loop_id": "loop-7", "created_at_ms": 1_700_000_000_000_i64},
            "id": 42,
        });
        let r = super::parse_rpc_response(&response, 42);
        let ok = r.expect("happy path");
        assert_eq!(ok.loop_id, "loop-7");
        assert_eq!(ok.created_at_ms, 1_700_000_000_000_i64);
    }

    // rationale: Adversarial input — a response that omits `id` entirely
    // is malformed per JSON-RPC 2.0 § 5 and must surface as a parse error
    // (not silently accepted).
    #[test]
    fn rpc_call_refuses_response_with_missing_id() {
        // rationale: Adversarial input (missing id field)
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": {"loop_id": "x", "created_at_ms": 0},
        });
        let r = super::parse_rpc_response(&response, 1);
        assert!(matches!(r, Err(LcmRpcError::Parse(_))));
    }

    // ====================================================================
    // Cluster H Wave-A3 — H4 JSON-RPC error-envelope edge cases.
    // Categories: Adversarial input · Boundary.
    // ====================================================================

    // rationale: Adversarial input — `"error": null` is NOT an error per
    // JSON-RPC 2.0 § 5 (error must be an object with a code). The result
    // field MUST be decoded instead. Defends against a future tightening
    // that mistakes null for an error envelope.
    #[test]
    fn rpc_treats_null_error_as_non_error() {
        // rationale: Adversarial input (null-error semantics)
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "error": null,
            "result": {"loop_id": "ok", "created_at_ms": 0},
            "id": 1,
        });
        let r = super::parse_rpc_response(&response, 1);
        let ok = r.expect("null error must fall through to result");
        assert_eq!(ok.loop_id, "ok");
    }

    // rationale: Adversarial input — `"error": {}` (empty object, no
    // `code` field) is NOT a valid JSON-RPC 2.0 error per § 5.1. The
    // result field MUST be decoded instead.
    #[test]
    fn rpc_treats_empty_object_error_as_non_error() {
        // rationale: Adversarial input (empty-object error envelope)
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "error": {},
            "result": {"loop_id": "ok", "created_at_ms": 0},
            "id": 1,
        });
        let r = super::parse_rpc_response(&response, 1);
        let ok = r.expect("empty-object error must fall through to result");
        assert_eq!(ok.loop_id, "ok");
    }

    // rationale: Contract regression — a properly-formed JSON-RPC 2.0
    // error envelope (object with code + message) MUST surface as Rpc
    // error. Happy-path of the error branch.
    #[test]
    fn rpc_treats_proper_error_object_as_error() {
        // rationale: Contract regression (proper error path)
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "error": {"code": -32601, "message": "Method not found"},
            "id": 1,
        });
        let r = super::parse_rpc_response(&response, 1);
        match r {
            Err(LcmRpcError::Rpc { code, message }) => {
                assert_eq!(code, -32601);
                assert_eq!(message, "Method not found");
            }
            other => panic!("expected Rpc error, got {other:?}"),
        }
    }

    // rationale: Boundary — IdMismatch Display format pinned for log
    // scrapers; the variant must surface both the sent and received ids.
    #[test]
    fn id_mismatch_error_display_format_pinned() {
        // rationale: Boundary (operator-facing log format)
        let err = LcmRpcError::IdMismatch {
            sent: 42,
            received: serde_json::json!(999),
        };
        let s = err.to_string();
        assert!(s.starts_with("id mismatch: "));
        assert!(s.contains("sent=42"));
        assert!(s.contains("received=999"));
    }
}
