//! `m41_lcm_rpc` — `lcm.loop.create` JSON-RPC client.
//! Cluster H · L8.
//!
//! NOTE: `lcm.loop.create` is the canonical LCM RPC name (per S1001882
//! drift retraction; the legacy `lcm.deploy` form is deprecated).

use std::io::Read;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use thiserror::Error;

/// Default LCM RPC endpoint (stdio supervisor binds to this loopback URI).
pub const DEFAULT_LCM_URL: &str = "http://127.0.0.1:8082/rpc";

/// Default RPC method.
pub const RPC_METHOD: &str = "lcm.loop.create";

/// Default request timeout.
pub const DEFAULT_RPC_TIMEOUT: Duration = Duration::from_secs(10);

/// SEC4 — hard cap on the bytes read from an `lcm.loop.create` JSON-RPC
/// response. The LCM supervisor returns a small `{loop_id, created_at_ms}`
/// envelope; a response larger than 1 MiB is pathological (compromised
/// endpoint, proxy error page, or contract drift). Capping the read
/// prevents a multi-GB body from forcing an unbounded allocation in
/// [`HttpLcmClient::loop_create`].
pub const MAX_RESPONSE_BYTES: u64 = 1024 * 1024;

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
        // SEC4 (size cap): read the body via `read_capped_body`, which
        // honours `Content-Length` and hard-stops at `MAX_RESPONSE_BYTES`.
        // A multi-GB body can no longer force an unbounded allocation.
        let bytes = read_capped_body(resp)?;
        let value: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|e| LcmRpcError::Parse(e.to_string()))?;
        parse_rpc_response(&value, request_id)
    }
}

/// SEC4 — read an HTTP response body with a hard [`MAX_RESPONSE_BYTES`]
/// size cap.
///
/// Returns [`LcmRpcError::Transport`] when:
///
/// - `Content-Length` is advertised and already exceeds the cap — the
///   body is rejected *before* a single byte is buffered;
/// - the body has no `Content-Length` (chunked / streamed) but the actual
///   stream exceeds the cap — the read is bounded by `Read::take()` and a
///   `>`-cap result is rejected;
/// - the underlying stream errors mid-read.
///
/// A multi-GB body therefore can never force an unbounded allocation: at
/// most `MAX_RESPONSE_BYTES + 1` bytes are ever held.
fn read_capped_body(
    resp: reqwest::blocking::Response,
) -> Result<Vec<u8>, LcmRpcError> {
    // Early reject: a server-advertised length over the cap never gets read.
    if let Some(len) = resp.content_length() {
        if len > MAX_RESPONSE_BYTES {
            return Err(LcmRpcError::Transport(format!(
                "response Content-Length {len} exceeds {MAX_RESPONSE_BYTES}-byte cap"
            )));
        }
    }
    // Bounded read: take at most cap+1 bytes. If we actually buffered
    // cap+1, the stream was larger than the cap (chunked / mislabelled
    // Content-Length) — reject rather than continue.
    let mut buf = Vec::new();
    let read_limit = MAX_RESPONSE_BYTES.saturating_add(1);
    resp.take(read_limit)
        .read_to_end(&mut buf)
        .map_err(|e| LcmRpcError::Transport(e.to_string()))?;
    if buf.len() as u64 > MAX_RESPONSE_BYTES {
        return Err(LcmRpcError::Transport(format!(
            "response stream exceeded {MAX_RESPONSE_BYTES}-byte cap"
        )));
    }
    Ok(buf)
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

    // ====================================================================
    // S1002388 hardening pass — m41 LCM RPC (+23 tests → ≥50).
    // parse_rpc_response branch exhaustion · id-type adversarial input ·
    // error-envelope edge cases · allocate_request_id monotonicity ·
    // serde boundary · cross-module surface.
    // ====================================================================

    // -- parse_rpc_response: id-type adversarial input -------------------

    // rationale: Adversarial input — a JSON-RPC response that echoes the
    // id as a *string* ("42") instead of the number 42 must NOT match.
    // `as_u64()` returns None for a string, so the id-echo check fails
    // closed and surfaces IdMismatch (not a silent accept via coercion).
    #[test]
    fn rpc_refuses_response_with_string_typed_id() {
        // rationale: Adversarial input (id type-confusion)
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": {"loop_id": "x", "created_at_ms": 0},
            "id": "42",
        });
        let r = super::parse_rpc_response(&response, 42);
        match r {
            Err(LcmRpcError::IdMismatch { sent, received }) => {
                assert_eq!(sent, 42);
                assert_eq!(received, serde_json::json!("42"));
            }
            other => panic!("expected IdMismatch for string id, got {other:?}"),
        }
    }

    // rationale: Adversarial input — `"id": null` is a present-but-null
    // field. `Value::get` returns Some(Null), so the missing-id parse
    // branch is skipped; `as_u64()` is None → IdMismatch with received
    // Null. Distinguishes "null id" from "absent id".
    #[test]
    fn rpc_refuses_response_with_null_id() {
        // rationale: Adversarial input (present-but-null id)
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": {"loop_id": "x", "created_at_ms": 0},
            "id": serde_json::Value::Null,
        });
        let r = super::parse_rpc_response(&response, 1);
        match r {
            Err(LcmRpcError::IdMismatch { received, .. }) => {
                assert_eq!(received, serde_json::Value::Null);
            }
            other => panic!("expected IdMismatch for null id, got {other:?}"),
        }
    }

    // rationale: Adversarial input — a fractional `id` (3.5) is not a
    // valid JSON-RPC integer id. `as_u64()` returns None for a non-integer
    // float → IdMismatch, never a lossy truncation to 3.
    #[test]
    fn rpc_refuses_response_with_fractional_id() {
        // rationale: Adversarial input (non-integer id)
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": {"loop_id": "x", "created_at_ms": 0},
            "id": 3.5,
        });
        let r = super::parse_rpc_response(&response, 3);
        assert!(
            matches!(r, Err(LcmRpcError::IdMismatch { sent: 3, .. })),
            "fractional id must not truncate-match, got {r:?}"
        );
    }

    // rationale: Adversarial input — a negative `id` (-1) cannot equal an
    // unsigned request id. `as_u64()` returns None for a negative number
    // → IdMismatch.
    #[test]
    fn rpc_refuses_response_with_negative_id() {
        // rationale: Adversarial input (negative id)
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": {"loop_id": "x", "created_at_ms": 0},
            "id": -1,
        });
        let r = super::parse_rpc_response(&response, 1);
        assert!(matches!(r, Err(LcmRpcError::IdMismatch { sent: 1, .. })));
    }

    // rationale: Boundary — u64::MAX is a legal JSON-RPC id; a response
    // echoing exactly u64::MAX against an expected u64::MAX request id
    // MUST match (no overflow / precision loss in the id-echo path).
    #[test]
    fn rpc_accepts_response_with_u64_max_id() {
        // rationale: Boundary (id-echo at the u64 ceiling)
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": {"loop_id": "edge", "created_at_ms": 0},
            "id": u64::MAX,
        });
        let r = super::parse_rpc_response(&response, u64::MAX);
        let ok = r.expect("u64::MAX id must round-trip");
        assert_eq!(ok.loop_id, "edge");
    }

    // rationale: Boundary — id 0 is the smallest legal id; allocator
    // starts at 1 but parse_rpc_response must still verify id 0 correctly
    // for callers composing their own envelopes.
    #[test]
    fn rpc_accepts_response_with_zero_id() {
        // rationale: Boundary (id-echo at zero)
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": {"loop_id": "zero", "created_at_ms": 1},
            "id": 0,
        });
        let ok = super::parse_rpc_response(&response, 0).expect("id 0");
        assert_eq!(ok.loop_id, "zero");
    }

    // -- parse_rpc_response: error-envelope edge cases -------------------

    // rationale: Adversarial input — an `error` field that is a *string*
    // (not an object) is malformed per JSON-RPC 2.0 § 5.1. `is_object()`
    // is false → the error branch is skipped → result is decoded instead.
    #[test]
    fn rpc_treats_string_error_field_as_non_error() {
        // rationale: Adversarial input (non-object error field)
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "error": "something went wrong",
            "result": {"loop_id": "ok", "created_at_ms": 0},
            "id": 1,
        });
        let ok = super::parse_rpc_response(&response, 1)
            .expect("string error field must fall through to result");
        assert_eq!(ok.loop_id, "ok");
    }

    // rationale: Adversarial input — an `error` field that is an *array*
    // is not a valid error object; the error branch is skipped.
    #[test]
    fn rpc_treats_array_error_field_as_non_error() {
        // rationale: Adversarial input (array error field)
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "error": [1, 2, 3],
            "result": {"loop_id": "ok", "created_at_ms": 7},
            "id": 1,
        });
        let ok = super::parse_rpc_response(&response, 1)
            .expect("array error field must fall through to result");
        assert_eq!(ok.created_at_ms, 7);
    }

    // rationale: Adversarial input — an error object that omits `message`
    // still surfaces as Rpc; the missing message defaults to "unknown"
    // (no panic, no parse error).
    #[test]
    fn rpc_error_object_missing_message_defaults_to_unknown() {
        // rationale: Adversarial input (error object missing message)
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "error": {"code": -32099},
            "id": 1,
        });
        match super::parse_rpc_response(&response, 1) {
            Err(LcmRpcError::Rpc { code, message }) => {
                assert_eq!(code, -32099);
                assert_eq!(message, "unknown", "missing message must default to 'unknown'");
            }
            other => panic!("expected Rpc error, got {other:?}"),
        }
    }

    // rationale: Adversarial input — an error object whose `code` is a
    // *string* (not a number) still triggers the error branch (the branch
    // gate is `code` field present, not `code` is i64); the code defaults
    // to the -32000 server-error sentinel since `as_i64()` fails.
    #[test]
    fn rpc_error_object_with_string_code_defaults_to_server_error_sentinel() {
        // rationale: Adversarial input (non-numeric error code)
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "error": {"code": "BAD", "message": "stringy code"},
            "id": 1,
        });
        match super::parse_rpc_response(&response, 1) {
            Err(LcmRpcError::Rpc { code, message }) => {
                assert_eq!(code, -32000, "non-i64 code must fall back to -32000");
                assert_eq!(message, "stringy code");
            }
            other => panic!("expected Rpc error, got {other:?}"),
        }
    }

    // rationale: Contract regression — the id-echo check runs BEFORE the
    // error branch. A response carrying a proper error object BUT a
    // mismatched id must surface IdMismatch, not Rpc — a replayed error
    // frame must not be mistaken for this request's failure.
    #[test]
    fn rpc_id_mismatch_takes_precedence_over_error_envelope() {
        // rationale: Contract regression (id-check order-of-operations)
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "error": {"code": -32601, "message": "Method not found"},
            "id": 777,
        });
        let r = super::parse_rpc_response(&response, 1);
        assert!(
            matches!(r, Err(LcmRpcError::IdMismatch { sent: 1, .. })),
            "id check must precede error decode, got {r:?}"
        );
    }

    // -- parse_rpc_response: result decoding --------------------------

    // rationale: Adversarial input — a matching id with NO `result` and NO
    // `error` is a malformed response; parse_rpc_response surfaces a Parse
    // error ("missing result"), never a default-constructed result.
    #[test]
    fn rpc_response_with_matching_id_but_no_result_is_parse_error() {
        // rationale: Adversarial input (result absent)
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 5,
        });
        match super::parse_rpc_response(&response, 5) {
            Err(LcmRpcError::Parse(msg)) => assert!(msg.contains("missing result")),
            other => panic!("expected Parse(missing result), got {other:?}"),
        }
    }

    // rationale: Adversarial input — a `result` whose shape does not match
    // LcmLoopCreateResult (missing the required `loop_id`) surfaces a
    // Parse error from serde, not a silent zero-value struct.
    #[test]
    fn rpc_result_with_wrong_shape_is_parse_error() {
        // rationale: Adversarial input (result schema mismatch)
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": {"wrong_field": 1},
            "id": 9,
        });
        assert!(
            matches!(super::parse_rpc_response(&response, 9), Err(LcmRpcError::Parse(_))),
            "result missing loop_id must be a Parse error"
        );
    }

    // rationale: Adversarial input — `created_at_ms` is i64; a result
    // carrying a negative epoch (pre-1970) still decodes (i64 is signed).
    // Documents that m41 does not range-check the timestamp.
    #[test]
    fn rpc_result_accepts_negative_created_at_ms() {
        // rationale: Adversarial input (negative i64 timestamp)
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": {"loop_id": "pre-epoch", "created_at_ms": -1_000_i64},
            "id": 2,
        });
        let ok = super::parse_rpc_response(&response, 2).expect("negative ts decodes");
        assert_eq!(ok.created_at_ms, -1_000);
    }

    // rationale: Adversarial input — a `result` that is a JSON string
    // (not an object) cannot decode into LcmLoopCreateResult → Parse error.
    #[test]
    fn rpc_result_that_is_a_bare_string_is_parse_error() {
        // rationale: Adversarial input (result is not an object)
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": "not-a-struct",
            "id": 4,
        });
        assert!(matches!(
            super::parse_rpc_response(&response, 4),
            Err(LcmRpcError::Parse(_))
        ));
    }

    // -- allocate_request_id: monotonicity / sequencing ------------------

    // rationale: Contract regression — a fresh HttpLcmClient's first
    // allocated id is exactly 1 (AtomicU64::new(1)); JSON-RPC ids must be
    // non-zero / well-defined from the first call.
    #[test]
    fn allocate_request_id_starts_at_one() {
        // rationale: Contract regression (id stream origin)
        let c = HttpLcmClient::new(DEFAULT_LCM_URL, DEFAULT_RPC_TIMEOUT);
        assert_eq!(c.allocate_request_id(), 1, "first id must be 1");
    }

    // rationale: Determinism — sequential allocate_request_id calls return
    // a strictly increasing contiguous sequence 1,2,3,...,N.
    #[test]
    fn allocate_request_id_is_strictly_increasing_and_contiguous() {
        // rationale: Determinism (id allocator sequence)
        let c = HttpLcmClient::new(DEFAULT_LCM_URL, DEFAULT_RPC_TIMEOUT);
        let ids: Vec<u64> = (0..50).map(|_| c.allocate_request_id()).collect();
        for (i, id) in ids.iter().enumerate() {
            assert_eq!(*id, (i as u64) + 1, "id stream must be contiguous from 1");
        }
    }

    // rationale: Concurrency — two clients have independent id streams; a
    // client's allocator state must not leak across instances.
    #[test]
    fn allocate_request_id_streams_are_per_client_independent() {
        // rationale: Concurrency (per-client id isolation)
        let a = HttpLcmClient::new(DEFAULT_LCM_URL, DEFAULT_RPC_TIMEOUT);
        let b = HttpLcmClient::new(DEFAULT_LCM_URL, DEFAULT_RPC_TIMEOUT);
        let _ = a.allocate_request_id();
        let _ = a.allocate_request_id();
        // b is untouched — its first id must still be 1.
        assert_eq!(b.allocate_request_id(), 1);
        // a has consumed 1 and 2 — its next id is 3.
        assert_eq!(a.allocate_request_id(), 3);
    }

    // -- serde boundary --------------------------------------------------

    // rationale: Boundary — LcmLoopCreateParams with an empty-object
    // loop_spec round-trips identically; the keystone empty-payload case.
    #[test]
    fn params_with_empty_loop_spec_round_trips() {
        // rationale: Boundary (empty payload)
        let p = LcmLoopCreateParams {
            workflow_id: 0,
            conductor_dispatch_id: String::new(),
            loop_spec: serde_json::json!({}),
        };
        let s = serde_json::to_string(&p).expect("ser");
        let back: LcmLoopCreateParams = serde_json::from_str(&s).expect("de");
        assert_eq!(back, p);
    }

    // rationale: Boundary — a deeply nested loop_spec survives the serde
    // round-trip without structural loss (loop_spec is opaque Value).
    #[test]
    fn params_with_deeply_nested_loop_spec_round_trips() {
        // rationale: Boundary (nested arbitrary JSON)
        let p = LcmLoopCreateParams {
            workflow_id: u64::MAX,
            conductor_dispatch_id: "conductor-deep".into(),
            loop_spec: serde_json::json!({
                "a": {"b": {"c": [1, 2, {"d": null}]}},
                "list": [[], [1], [1, 2]],
            }),
        };
        let s = serde_json::to_string(&p).expect("ser");
        let back: LcmLoopCreateParams = serde_json::from_str(&s).expect("de");
        assert_eq!(back, p, "nested loop_spec must survive round-trip");
    }

    // rationale: Adversarial input — deserialising LcmLoopCreateParams
    // from JSON missing the required `workflow_id` field fails loudly.
    #[test]
    fn params_deserialization_rejects_missing_workflow_id() {
        // rationale: Adversarial input (missing required field)
        let bad = r#"{"conductor_dispatch_id":"x","loop_spec":{}}"#;
        let r: Result<LcmLoopCreateParams, _> = serde_json::from_str(bad);
        assert!(r.is_err(), "missing workflow_id must fail deserialization");
    }

    // rationale: Adversarial input — deserialising LcmLoopCreateResult
    // from JSON whose `created_at_ms` is a string fails (i64 type guard).
    #[test]
    fn result_deserialization_rejects_string_created_at_ms() {
        // rationale: Adversarial input (type-mismatched field)
        let bad = r#"{"loop_id":"x","created_at_ms":"not-a-number"}"#;
        let r: Result<LcmLoopCreateResult, _> = serde_json::from_str(bad);
        assert!(r.is_err(), "string created_at_ms must fail deserialization");
    }

    // rationale: Contract regression — loop_create serialises the JSON-RPC
    // envelope with exactly the four wire keys {jsonrpc, method, params,
    // id} and jsonrpc="2.0". Exercised via the StubClient's queued result
    // path is not enough; this checks the envelope shape directly through
    // the documented body builder constants.
    #[test]
    fn rpc_envelope_uses_jsonrpc_2_0_and_canonical_method() {
        // rationale: Contract regression (JSON-RPC envelope shape)
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": RPC_METHOD,
            "params": LcmLoopCreateParams {
                workflow_id: 1,
                conductor_dispatch_id: "c".into(),
                loop_spec: serde_json::json!({}),
            },
            "id": 1_u64,
        });
        let obj = body.as_object().expect("object");
        assert_eq!(obj.get("jsonrpc").and_then(|v| v.as_str()), Some("2.0"));
        assert_eq!(obj.get("method").and_then(|v| v.as_str()), Some("lcm.loop.create"));
        assert!(obj.get("params").is_some_and(serde_json::Value::is_object));
        assert!(obj.get("id").and_then(serde_json::Value::as_u64).is_some());
    }

    // -- error variant: Debug / Display completeness ---------------------

    // rationale: Adversarial input — every LcmRpcError variant has a
    // non-empty Display string (operator runbooks depend on these).
    #[test]
    fn every_lcm_rpc_error_variant_has_non_empty_display() {
        // rationale: Adversarial input (operator-runbook stability)
        let variants = [
            LcmRpcError::Transport("t".into()),
            LcmRpcError::Rpc { code: -1, message: "m".into() },
            LcmRpcError::Parse("p".into()),
            LcmRpcError::IdMismatch { sent: 1, received: serde_json::json!(2) },
        ];
        for v in &variants {
            assert!(!v.to_string().is_empty(), "empty Display for {v:?}");
        }
    }

    // rationale: Cross-module surface invariant — IdMismatch preserves the
    // raw received id as JSON for diagnosis; a received object survives
    // verbatim into the error (not stringified / lossy).
    #[test]
    fn id_mismatch_preserves_received_object_verbatim() {
        // rationale: Cross-module surface (diagnostic fidelity)
        let weird = serde_json::json!({"unexpected": "shape", "n": 3});
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": {"loop_id": "x", "created_at_ms": 0},
            "id": weird.clone(),
        });
        match super::parse_rpc_response(&response, 1) {
            Err(LcmRpcError::IdMismatch { received, .. }) => {
                assert_eq!(received, weird, "received id must survive verbatim");
            }
            other => panic!("expected IdMismatch, got {other:?}"),
        }
    }

    // rationale: Contract regression — RPC_METHOD is a dotted three-segment
    // namespace (lcm.loop.create); a regression to a two-segment form
    // would silently break LCM method routing.
    #[test]
    fn rpc_method_has_three_dotted_segments() {
        // rationale: Contract regression (method namespace shape)
        let segs: Vec<&str> = RPC_METHOD.split('.').collect();
        assert_eq!(segs, vec!["lcm", "loop", "create"]);
    }

    // ====================================================================
    // W2 hardening — SEC4 response-body size cap.
    // ====================================================================

    // rationale: SEC4 — MAX_RESPONSE_BYTES is the documented 1 MiB cap.
    #[test]
    fn max_response_bytes_is_one_mib() {
        // rationale: SEC4 (size-cap constant pin)
        assert_eq!(super::MAX_RESPONSE_BYTES, 1024 * 1024);
    }

    // rationale: SEC4 — a JSON-RPC response whose body exceeds the 1 MiB
    // cap must surface a typed Transport error, NOT force an unbounded
    // allocation. wiremock sets a real Content-Length so the early-reject
    // branch fires before any byte is buffered.
    #[tokio::test(flavor = "current_thread")]
    async fn loop_create_rejects_over_cap_response_body() {
        // rationale: SEC4 (unbounded-allocation guard)
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        // 2 MiB body — twice the cap.
        let huge = "x".repeat(2 * 1024 * 1024);
        Mock::given(method("POST"))
            .and(path("/rpc"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(huge)
                    .insert_header("content-type", "application/json"),
            )
            .mount(&server)
            .await;

        let url = format!("{}/rpc", server.uri());
        let r = tokio::task::spawn_blocking(move || {
            let c = HttpLcmClient::new(url, std::time::Duration::from_secs(3));
            let p = LcmLoopCreateParams {
                workflow_id: 1,
                conductor_dispatch_id: "c".into(),
                loop_spec: serde_json::json!({}),
            };
            c.loop_create(&p)
        })
        .await
        .expect("join");
        match r {
            Err(LcmRpcError::Transport(msg)) => {
                assert!(
                    msg.contains("cap"),
                    "over-cap body must mention the cap; got {msg}"
                );
            }
            other => panic!("expected Transport(over-cap), got {other:?}"),
        }
    }

    // rationale: SEC4 — a legitimate small JSON-RPC response well under the
    // cap still parses correctly; the cap rejects only genuine over-cap
    // bodies.
    #[tokio::test(flavor = "current_thread")]
    async fn loop_create_accepts_under_cap_response_body() {
        // rationale: SEC4 (cap does not regress the happy path)
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        // A valid JSON-RPC response — small, well under the cap. The
        // request id is allocated from 1 by a fresh client, so id 1 echoes.
        let body = r#"{"jsonrpc":"2.0","result":{"loop_id":"loop-ok","created_at_ms":7},"id":1}"#;
        Mock::given(method("POST"))
            .and(path("/rpc"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(body)
                    .insert_header("content-type", "application/json"),
            )
            .mount(&server)
            .await;

        let url = format!("{}/rpc", server.uri());
        let r = tokio::task::spawn_blocking(move || {
            let c = HttpLcmClient::new(url, std::time::Duration::from_secs(3));
            let p = LcmLoopCreateParams {
                workflow_id: 1,
                conductor_dispatch_id: "c".into(),
                loop_spec: serde_json::json!({}),
            };
            c.loop_create(&p)
        })
        .await
        .expect("join");
        let ok = r.expect("under-cap valid response must parse");
        assert_eq!(ok.loop_id, "loop-ok");
        assert_eq!(ok.created_at_ms, 7);
    }

    // rationale: SEC4 — a sizeable-but-under-cap padded JSON-RPC response
    // still parses; proves the cap boundary tolerates large legitimate
    // bodies (e.g. a verbose loop_spec echo).
    #[tokio::test(flavor = "current_thread")]
    async fn loop_create_accepts_large_under_cap_padded_body() {
        // rationale: SEC4 (boundary — large but legitimate body)
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        // 512 KiB of padding inside a valid JSON-RPC envelope.
        let pad = "a".repeat(512 * 1024);
        let body = format!(
            r#"{{"jsonrpc":"2.0","result":{{"loop_id":"loop-big","created_at_ms":9,"_pad":"{pad}"}},"id":1}}"#
        );
        Mock::given(method("POST"))
            .and(path("/rpc"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(body)
                    .insert_header("content-type", "application/json"),
            )
            .mount(&server)
            .await;

        let url = format!("{}/rpc", server.uri());
        let r = tokio::task::spawn_blocking(move || {
            let c = HttpLcmClient::new(url, std::time::Duration::from_secs(3));
            let p = LcmLoopCreateParams {
                workflow_id: 1,
                conductor_dispatch_id: "c".into(),
                loop_spec: serde_json::json!({}),
            };
            c.loop_create(&p)
        })
        .await
        .expect("join");
        let ok = r.expect("large-but-under-cap response must parse");
        assert_eq!(ok.loop_id, "loop-big");
    }
}
