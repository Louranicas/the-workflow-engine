//! Integration tests for m41 lcm_rpc (Wave-B2).
//!
//! Exercises `HttpLcmClient::loop_create` against a wiremock-driven mock
//! JSON-RPC server at the LCM `/rpc` endpoint:
//!
//! - Matching id → happy path, result decoded into `LcmLoopCreateResult`.
//! - Mismatched id → `LcmRpcError::IdMismatch` (Wave-A3 H3 hardening).
//! - `error: null`               → not an error (Wave-A3 H4).
//! - `error: {}`                 → not an error (Wave-A3 H4).
//! - `error: {code, message}`    → `LcmRpcError::Rpc`.
//! - 32 concurrent calls         → unique ids per request on the wire
//!   (Wave-A3 H3 AtomicU64 generator correctness end-to-end).
//! - `method` field literal      → `lcm.loop.create` (S1001882 drift retraction).
//!
//! `loop_create` is blocking-reqwest; wiremock requires tokio. We route the
//! blocking call through `tokio::task::spawn_blocking`.

#![allow(clippy::doc_markdown)]

use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

use workflow_core::m41_lcm_rpc::{
    HttpLcmClient, LcmClient, LcmLoopCreateParams, LcmRpcError, RPC_METHOD,
};

fn fixture_params(workflow_id: u64) -> LcmLoopCreateParams {
    LcmLoopCreateParams {
        workflow_id,
        conductor_dispatch_id: format!("conductor-{workflow_id:04}"),
        loop_spec: serde_json::json!({"steps": [1, 2, 3]}),
    }
}

/// Mock JSON-RPC server that echoes the request `id` back (the wire-correct
/// behaviour) and returns a canned result. Captures inbound `id` values for
/// concurrency assertions.
struct EchoIdResponder {
    captured_ids: Arc<Mutex<Vec<u64>>>,
    captured_methods: Arc<Mutex<Vec<String>>>,
}

impl Respond for EchoIdResponder {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let body: serde_json::Value =
            serde_json::from_slice(&request.body).unwrap_or(serde_json::Value::Null);
        let id = body
            .get("id")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        self.captured_ids.lock().expect("ids lock").push(id);
        if let Some(m) = body.get("method").and_then(serde_json::Value::as_str) {
            self.captured_methods
                .lock()
                .expect("methods lock")
                .push(m.to_owned());
        }
        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "result": {
                "loop_id": format!("loop-{id}"),
                "created_at_ms": 1_700_000_000_000_i64,
            },
            "id": id,
        });
        ResponseTemplate::new(200)
            .set_body_string(resp.to_string())
            .insert_header("content-type", "application/json")
    }
}

// rationale: Happy path (matching id echo + result decode) — JSON-RPC 2.0
//            § 5 happy path preservation post Wave-A3 H3 id-echo hardening.
#[tokio::test(flavor = "current_thread")]
async fn m41_loop_create_happy_path_with_matching_id() {
    // rationale: Happy path (matching id echo)
    let captured_ids: Arc<Mutex<Vec<u64>>> = Arc::new(Mutex::new(Vec::new()));
    let captured_methods: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/rpc"))
        .respond_with(EchoIdResponder {
            captured_ids: Arc::clone(&captured_ids),
            captured_methods: Arc::clone(&captured_methods),
        })
        .expect(1)
        .mount(&server)
        .await;

    let url = format!("{}/rpc", server.uri());
    let r = tokio::task::spawn_blocking(move || {
        let c = HttpLcmClient::new(url, Duration::from_secs(2));
        c.loop_create(&fixture_params(7))
    })
    .await
    .expect("join");
    let ok = r.expect("happy path must succeed");
    assert!(
        ok.loop_id.starts_with("loop-"),
        "loop_id must be substrate-assigned, got {}",
        ok.loop_id
    );
    assert_eq!(ok.created_at_ms, 1_700_000_000_000_i64);
}

// rationale: Anti-property (Wave-A3 H3 — id mismatch refused) — a mock
//            server that returns id=999 regardless of request must
//            surface as IdMismatch carrying both sent and received.
#[tokio::test(flavor = "current_thread")]
async fn m41_loop_create_refuses_response_with_wrong_id() {
    // rationale: Anti-property (Wave-A3 H3 id-echo verification)
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/rpc"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(
                    r#"{"jsonrpc":"2.0","result":{"loop_id":"x","created_at_ms":0},"id":999}"#,
                )
                .insert_header("content-type", "application/json"),
        )
        .mount(&server)
        .await;
    let url = format!("{}/rpc", server.uri());
    let r = tokio::task::spawn_blocking(move || {
        let c = HttpLcmClient::new(url, Duration::from_secs(2));
        c.loop_create(&fixture_params(1))
    })
    .await
    .expect("join");
    match r {
        Err(LcmRpcError::IdMismatch { sent, received }) => {
            // Per m41 mod doc: per-client AtomicU64 starts at 1; this is
            // the very first call from this client → sent == 1.
            assert_eq!(sent, 1, "first call's sent id must be 1");
            assert_eq!(received, serde_json::json!(999));
        }
        other => panic!("expected IdMismatch, got {other:?}"),
    }
}

// rationale: Adversarial input (Wave-A3 H4 null-error semantics) — null
//            error sibling with a real result must decode normally.
#[tokio::test(flavor = "current_thread")]
async fn m41_loop_create_treats_null_error_field_as_non_error() {
    // rationale: Adversarial input (Wave-A3 H4 null-error)
    let server = MockServer::start().await;
    // The body must echo the request id (1 — first call from a fresh
    // client) to survive id-echo verification, so we hand-craft it.
    Mock::given(method("POST"))
        .and(path("/rpc"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(
                    r#"{"jsonrpc":"2.0","error":null,"result":{"loop_id":"ok","created_at_ms":1700000000000},"id":1}"#,
                )
                .insert_header("content-type", "application/json"),
        )
        .mount(&server)
        .await;
    let url = format!("{}/rpc", server.uri());
    let r = tokio::task::spawn_blocking(move || {
        let c = HttpLcmClient::new(url, Duration::from_secs(2));
        c.loop_create(&fixture_params(2))
    })
    .await
    .expect("join");
    let ok = r.expect("null error must fall through to result");
    assert_eq!(ok.loop_id, "ok");
}

// rationale: Adversarial input (Wave-A3 H4 empty-object error) — `{}` is
//            not a valid JSON-RPC 2.0 § 5.1 error object; result decode.
#[tokio::test(flavor = "current_thread")]
async fn m41_loop_create_treats_empty_object_error_as_non_error() {
    // rationale: Adversarial input (Wave-A3 H4 empty-object error)
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/rpc"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(
                    r#"{"jsonrpc":"2.0","error":{},"result":{"loop_id":"ok","created_at_ms":1700000000000},"id":1}"#,
                )
                .insert_header("content-type", "application/json"),
        )
        .mount(&server)
        .await;
    let url = format!("{}/rpc", server.uri());
    let r = tokio::task::spawn_blocking(move || {
        let c = HttpLcmClient::new(url, Duration::from_secs(2));
        c.loop_create(&fixture_params(3))
    })
    .await
    .expect("join");
    let ok = r.expect("empty-object error must fall through to result");
    assert_eq!(ok.loop_id, "ok");
}

// rationale: Contract regression (Wave-A3 H4 proper error path) — an
//            object with `code` field surfaces as Rpc error with both
//            code and message preserved.
#[tokio::test(flavor = "current_thread")]
async fn m41_loop_create_treats_proper_error_object_with_code_as_error() {
    // rationale: Contract regression (proper JSON-RPC 2.0 error envelope)
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/rpc"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(
                    r#"{"jsonrpc":"2.0","error":{"code":-32601,"message":"method not found"},"id":1}"#,
                )
                .insert_header("content-type", "application/json"),
        )
        .mount(&server)
        .await;
    let url = format!("{}/rpc", server.uri());
    let r = tokio::task::spawn_blocking(move || {
        let c = HttpLcmClient::new(url, Duration::from_secs(2));
        c.loop_create(&fixture_params(4))
    })
    .await
    .expect("join");
    match r {
        Err(LcmRpcError::Rpc { code, message }) => {
            assert_eq!(code, -32601);
            assert_eq!(message, "method not found");
        }
        other => panic!("expected Rpc, got {other:?}"),
    }
}

// rationale: Concurrency (Wave-A3 H3 AtomicU64 generator correctness end-
//            to-end) — 32 concurrent loop_create calls on a shared client
//            put 32 distinct ids on the wire (no collision, no skip).
#[tokio::test(flavor = "current_thread")]
async fn m41_loop_create_assigns_unique_ids_across_concurrent_calls() {
    // rationale: Concurrency (AtomicU64 end-to-end correctness)
    let captured_ids: Arc<Mutex<Vec<u64>>> = Arc::new(Mutex::new(Vec::new()));
    let captured_methods: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/rpc"))
        .respond_with(EchoIdResponder {
            captured_ids: Arc::clone(&captured_ids),
            captured_methods: Arc::clone(&captured_methods),
        })
        .mount(&server)
        .await;
    let url = format!("{}/rpc", server.uri());
    let c = Arc::new(HttpLcmClient::new(url, Duration::from_secs(5)));
    let n = 32_usize;
    let mut handles = Vec::with_capacity(n);
    for i in 0..n {
        let c2 = Arc::clone(&c);
        handles.push(tokio::task::spawn_blocking(move || {
            c2.loop_create(&fixture_params(u64::try_from(i).expect("i fits")))
        }));
    }
    for h in handles {
        let r = h.await.expect("join");
        assert!(r.is_ok(), "concurrent loop_create failed: {r:?}");
    }
    let ids = captured_ids.lock().expect("ids lock").clone();
    assert_eq!(ids.len(), n, "server must observe all 32 requests");
    let unique: HashSet<u64> = ids.iter().copied().collect();
    assert_eq!(
        unique.len(),
        n,
        "32 concurrent calls must yield 32 distinct ids on the wire, got {ids:?}"
    );
}

// rationale: Defensive cross-module (m32 dispatch_method ↔ m41 RPC_METHOD) —
//            wire request body MUST carry method `lcm.loop.create` (not
//            `lcm.deploy` / `lcm.loop.deploy`). Regression lock against
//            S1001882 method-name drift.
#[tokio::test(flavor = "current_thread")]
async fn m41_loop_create_rpc_method_is_lcm_loop_create_literal() {
    // rationale: Defensive cross-module (m32 ↔ m41 method-name lock)
    let captured_ids: Arc<Mutex<Vec<u64>>> = Arc::new(Mutex::new(Vec::new()));
    let captured_methods: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/rpc"))
        .respond_with(EchoIdResponder {
            captured_ids: Arc::clone(&captured_ids),
            captured_methods: Arc::clone(&captured_methods),
        })
        .expect(1)
        .mount(&server)
        .await;
    let url = format!("{}/rpc", server.uri());
    let r = tokio::task::spawn_blocking(move || {
        let c = HttpLcmClient::new(url, Duration::from_secs(2));
        c.loop_create(&fixture_params(99))
    })
    .await
    .expect("join");
    assert!(r.is_ok(), "happy path expected, got {r:?}");
    let methods = captured_methods.lock().expect("methods lock").clone();
    assert_eq!(methods.len(), 1);
    assert_eq!(
        methods[0], "lcm.loop.create",
        "wire method MUST be lcm.loop.create (S1001882 drift lock); got {}",
        methods[0]
    );
    // Belt-and-braces: the module-level const matches the wire literal.
    assert_eq!(RPC_METHOD, "lcm.loop.create");
}
