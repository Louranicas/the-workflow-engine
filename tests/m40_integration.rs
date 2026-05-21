//! Integration tests for m40 nexus_emit (Wave-B2).
//!
//! Exercises `HttpNexusClient::push` through a wiremock-driven mock
//! synthex-v2 `/v3/nexus/push` endpoint:
//!
//! - 200 OK + `{"ok": true}` body  → success.
//! - 204 No Content + empty body   → success.
//! - 200 OK + `{"error": ...}`     → `NexusEmitError::ServerRejected`
//!   (AP-V7-13 body-shape rejection; Wave A3 commit hardened the m40 push
//!   path with body-inspection — health-200 is not behaviour-verified).
//! - 200 OK + `{"error": null}`    → success (null-error semantics).
//! - Unreachable URL               → `NexusEmitError::Transport`.
//! - Deliberate server delay > client timeout → `Transport` (timeout) fast.
//! - 8 concurrent pushes           → server observed all 8 distinct payloads.
//!
//! `push` is blocking-reqwest under the hood; wiremock requires a tokio
//! runtime; we keep the blocking call off the current-thread runtime by
//! routing through `tokio::task::spawn_blocking`.

#![allow(clippy::doc_markdown)]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

use workflow_core::m40_nexus_emit::{
    build_event, HttpNexusClient, NexusClient, NexusEmitError, NexusEvent,
};

fn fixture_event(i: i64) -> NexusEvent {
    build_event(
        "workflow-trace",
        "workflow.dispatched",
        serde_json::json!({"workflow_id": i, "tag": "wave-b2"}),
        1_700_000_000_000 + i,
    )
}

/// Wiremock responder that captures every inbound payload's
/// `payload.workflow_id` field for cross-thread assertions.
struct CaptureResponder {
    counter: Arc<AtomicUsize>,
    captured: Arc<std::sync::Mutex<Vec<i64>>>,
}

impl Respond for CaptureResponder {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        self.counter.fetch_add(1, Ordering::Relaxed);
        if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&request.body) {
            if let Some(wf_id) = v
                .get("payload")
                .and_then(|p| p.get("workflow_id"))
                .and_then(serde_json::Value::as_i64)
            {
                self.captured.lock().expect("lock").push(wf_id);
            }
        }
        ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#)
    }
}

// rationale: Happy path (200 OK + ok body) — happy-path preservation after
//            the Wave-A3 C4 body-shape check landed.
#[tokio::test(flavor = "current_thread")]
async fn m40_push_accepts_200_with_ok_body() {
    // rationale: Happy path (200 OK + ok body)
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v3/nexus/push"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"{"ok":true,"accepted":1}"#)
                .insert_header("content-type", "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;
    let url = format!("{}/v3/nexus/push", server.uri());
    let r = tokio::task::spawn_blocking(move || {
        let c = HttpNexusClient::new(url, Duration::from_secs(2));
        c.push(&fixture_event(1))
    })
    .await
    .expect("join");
    assert!(r.is_ok(), "200 OK + ok body must succeed, got {r:?}");
}

// rationale: Boundary (204 No Content + empty body) — empty body is
//            legitimate fire-and-forget acceptance.
#[tokio::test(flavor = "current_thread")]
async fn m40_push_accepts_204_no_content() {
    // rationale: Boundary (empty body = success)
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v3/nexus/push"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    let url = format!("{}/v3/nexus/push", server.uri());
    let r = tokio::task::spawn_blocking(move || {
        let c = HttpNexusClient::new(url, Duration::from_secs(2));
        c.push(&fixture_event(2))
    })
    .await
    .expect("join");
    assert!(r.is_ok(), "204 No Content must succeed, got {r:?}");
}

// rationale: Anti-property (AP-V7-13 body-shape rejection) — a 200 OK
//            response with `{"error": "queue_full"}` is the exact shape
//            that motivated the m42 stcortex-only ADR. push MUST surface
//            this as `ServerRejected` carrying the body.
#[tokio::test(flavor = "current_thread")]
async fn m40_push_rejects_200_with_error_body_via_server_rejected() {
    // rationale: Anti-property (AP-V7-13 health-200 ≠ behaviour-verified)
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v3/nexus/push"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"{"error":"queue_full"}"#)
                .insert_header("content-type", "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;
    let url = format!("{}/v3/nexus/push", server.uri());
    let r = tokio::task::spawn_blocking(move || {
        let c = HttpNexusClient::new(url, Duration::from_secs(2));
        c.push(&fixture_event(3))
    })
    .await
    .expect("join");
    match r {
        Err(NexusEmitError::ServerRejected { body }) => {
            assert!(
                body.contains("queue_full"),
                "ServerRejected must carry the raw body, got {body}"
            );
        }
        other => panic!("expected ServerRejected, got {other:?}"),
    }
}

// rationale: Adversarial input (null-error semantics) — `{"error": null}`
//            with a sibling `result` is NOT a rejection; defends against
//            an over-eager body-shape check that would fail-open on null.
#[tokio::test(flavor = "current_thread")]
async fn m40_push_treats_null_error_field_as_success() {
    // rationale: Adversarial input (null-error semantics)
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v3/nexus/push"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"{"error":null,"result":"ok"}"#)
                .insert_header("content-type", "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;
    let url = format!("{}/v3/nexus/push", server.uri());
    let r = tokio::task::spawn_blocking(move || {
        let c = HttpNexusClient::new(url, Duration::from_secs(2));
        c.push(&fixture_event(4))
    })
    .await
    .expect("join");
    assert!(r.is_ok(), "null error field must be success, got {r:?}");
}

// rationale: Network failure typing (unreachable URL → Transport variant) —
//            push must NEVER panic and MUST surface a typed Transport error.
#[test]
fn m40_push_returns_transport_error_on_unreachable_url() {
    // rationale: Network failure typing (RFC-1149-style unreachable)
    let c = HttpNexusClient::new(
        "http://127.0.0.1:1/v3/nexus/push",
        Duration::from_millis(100),
    );
    let r = c.push(&fixture_event(5));
    match r {
        Err(NexusEmitError::Transport(_)) => {}
        Err(NexusEmitError::NonSuccess(_)) => {
            // On some kernels a closed loopback port returns RST → reqwest
            // surfaces this as a transport error; on others the connection
            // attempt times out. Either is acceptable, NonSuccess is not.
            panic!("unreachable URL must NOT surface as NonSuccess");
        }
        Err(NexusEmitError::ServerRejected { .. }) => {
            panic!("unreachable URL must NOT surface as ServerRejected");
        }
        // `NexusEmitError` is `#[non_exhaustive]` — wildcard required for
        // the cross-crate match. Only `Transport` is an acceptable error
        // for an unreachable URL; any other variant is a contract break.
        Err(other) => {
            panic!("unreachable URL must surface as Transport, got {other:?}");
        }
        Ok(()) => panic!("unreachable URL must NOT succeed"),
    }
}

// rationale: Boundary (client timeout < server delay) — a deliberate
//            3-second mock delay against a 100 ms client timeout must
//            surface as Transport in well under 500 ms (no hanging).
#[tokio::test(flavor = "current_thread")]
async fn m40_push_respects_timeout() {
    // rationale: Boundary (client timeout enforcement)
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v3/nexus/push"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(3)))
        .mount(&server)
        .await;
    let url = format!("{}/v3/nexus/push", server.uri());
    let started = std::time::Instant::now();
    let r = tokio::task::spawn_blocking(move || {
        let c = HttpNexusClient::new(url, Duration::from_millis(100));
        c.push(&fixture_event(6))
    })
    .await
    .expect("join");
    let elapsed = started.elapsed();
    assert!(
        matches!(r, Err(NexusEmitError::Transport(_))),
        "timeout must surface as Transport, got {r:?}"
    );
    assert!(
        elapsed < Duration::from_millis(500),
        "timeout must fail fast (<500ms), took {elapsed:?}"
    );
}

// rationale: Concurrency (per-event payload isolation) — 8 concurrent
//            pushes of distinct payloads must all land at the server;
//            wiremock-side capture confirms 8 distinct workflow_ids.
#[tokio::test(flavor = "current_thread")]
async fn m40_concurrent_pushes_isolated_per_event() {
    // rationale: Concurrency (per-call payload isolation)
    let counter = Arc::new(AtomicUsize::new(0));
    let captured: Arc<std::sync::Mutex<Vec<i64>>> =
        Arc::new(std::sync::Mutex::new(Vec::new()));

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v3/nexus/push"))
        .respond_with(CaptureResponder {
            counter: Arc::clone(&counter),
            captured: Arc::clone(&captured),
        })
        .mount(&server)
        .await;

    let url = format!("{}/v3/nexus/push", server.uri());
    let n = 8_usize;
    let mut handles = Vec::with_capacity(n);
    for i in 0..n {
        let url_clone = url.clone();
        handles.push(tokio::task::spawn_blocking(move || {
            let c = HttpNexusClient::new(url_clone, Duration::from_secs(2));
            c.push(&fixture_event(i64::try_from(i + 100).expect("i fits")))
        }));
    }
    for h in handles {
        let r = h.await.expect("join");
        assert!(r.is_ok(), "concurrent push failed: {r:?}");
    }
    assert_eq!(counter.load(Ordering::Relaxed), n, "server must observe all 8");
    let mut ids = captured.lock().expect("captured lock").clone();
    ids.sort_unstable();
    let unique: std::collections::HashSet<_> = ids.iter().copied().collect();
    assert_eq!(
        unique.len(),
        n,
        "8 concurrent pushes must produce 8 distinct workflow_ids on the wire, got {ids:?}"
    );
}
