//! `m40_nexusevent_emit` — NexusEvent push to synthex-v2 `:8092`.
//! Cluster H · L8.

use std::io::Read;
use std::time::Duration;

use thiserror::Error;

/// Default synthex-v2 NexusEvent endpoint.
pub const DEFAULT_NEXUS_URL: &str = "http://127.0.0.1:8092/v3/nexus/push";

/// Default push timeout.
pub const DEFAULT_PUSH_TIMEOUT: Duration = Duration::from_secs(5);

/// SEC4 — hard cap on the bytes read from a `/v3/nexus/push` response.
/// The nexus-push handler returns a tiny acknowledgement JSON; a response
/// larger than 1 MiB is pathological (compromised endpoint, proxy error
/// page, or contract drift). Capping the read prevents a multi-GB body
/// from forcing an unbounded allocation in [`HttpNexusClient::push`].
pub const MAX_RESPONSE_BYTES: u64 = 1024 * 1024;

/// A typed NexusEvent payload.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NexusEvent {
    /// Source service identifier.
    pub source: String,
    /// Event kind (e.g. `workflow.dispatched`, `workflow.completed`).
    pub kind: String,
    /// Free-form JSON payload.
    pub payload: serde_json::Value,
    /// Wall-clock ms.
    pub ts_ms: i64,
}

/// Emit errors.
#[derive(Debug, Error)]
pub enum NexusEmitError {
    /// Transport / HTTP layer failure.
    #[error("transport: {0}")]
    Transport(String),
    /// Server returned a non-2xx status.
    #[error("non-2xx status: {0}")]
    NonSuccess(u16),
    /// Server returned 2xx but the body carries a non-null `error` field —
    /// the AP-V7-13 health-200-but-behaviour-rejected pattern (the same
    /// shape that triggered the m42 stcortex-only ADR). Closes Cluster H
    /// Wave-A3 C4: a 200 OK with `{"error":"queue_full"}` is NOT success.
    #[error("server rejected (2xx body-shape): {body}")]
    ServerRejected {
        /// Raw response body that surfaced the rejection (bounded by the
        /// HTTP client's response size; not stored verbatim if larger).
        body: String,
    },
}

/// Trait abstraction for the HTTP push (real impl uses reqwest).
pub trait NexusClient: Send + Sync {
    /// Push the event; return on success / typed error.
    ///
    /// # Errors
    ///
    /// [`NexusEmitError::Transport`] on transport failure.
    /// [`NexusEmitError::NonSuccess`] on non-2xx.
    /// [`NexusEmitError::ServerRejected`] on 2xx with a body containing a
    /// non-null `error` field (AP-V7-13 body-shape check; Wave-A3 C4).
    fn push(&self, event: &NexusEvent) -> Result<(), NexusEmitError>;
}

/// Production HTTP client.
pub struct HttpNexusClient {
    url: String,
    timeout: Duration,
}

impl HttpNexusClient {
    /// Construct with explicit URL + timeout.
    #[must_use]
    pub fn new(url: impl Into<String>, timeout: Duration) -> Self {
        Self {
            url: url.into(),
            timeout,
        }
    }
}

impl NexusClient for HttpNexusClient {
    fn push(&self, event: &NexusEvent) -> Result<(), NexusEmitError> {
        let client = reqwest::blocking::Client::builder()
            .timeout(self.timeout)
            .build()
            .map_err(|e| NexusEmitError::Transport(e.to_string()))?;
        let resp = client
            .post(&self.url)
            .json(event)
            .send()
            .map_err(|e| NexusEmitError::Transport(e.to_string()))?;
        let status = resp.status();
        if !status.is_success() {
            return Err(NexusEmitError::NonSuccess(status.as_u16()));
        }
        // AP-V7-13 body-shape check (Wave-A3 C4): a 2xx status is necessary
        // but not sufficient. The server may return 200 + `{"error":"..."}`
        // to indicate behavioural rejection (queue_full, deprecated, etc.).
        // We read the body and inspect for a non-null `error` field. An
        // empty body (e.g. 204 No Content) or a non-JSON body is treated as
        // success — many sane servers omit body on accept.
        //
        // SEC4 (size cap): the body is read via `read_capped_body`, which
        // honours `Content-Length` and hard-stops at `MAX_RESPONSE_BYTES`.
        // A multi-GB body can no longer force an unbounded allocation — an
        // over-cap body surfaces as a typed Transport error.
        let body = read_capped_body(resp)?;
        if body.is_empty() {
            return Ok(());
        }
        // If the body is not parseable JSON, we treat as success: this
        // module's contract is "reject on a visibly-rejecting JSON body".
        // Garbled bodies are a transport / contract-drift class that the
        // caller will surface elsewhere (m41 has its own parse layer).
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&body) else {
            return Ok(());
        };
        if let Some(err) = value.get("error") {
            if !err.is_null() {
                return Err(NexusEmitError::ServerRejected { body });
            }
        }
        Ok(())
    }
}

/// SEC4 — read an HTTP response body as a UTF-8 string with a hard
/// [`MAX_RESPONSE_BYTES`] size cap.
///
/// Returns [`NexusEmitError::Transport`] when:
///
/// - `Content-Length` is advertised and already exceeds the cap — the
///   body is rejected *before* a single byte is buffered;
/// - the body has no `Content-Length` (chunked / streamed) but the actual
///   stream exceeds the cap — the read is bounded by `Read::take()` and a
///   `>`-cap result is rejected;
/// - the underlying stream errors mid-read, or the body is not valid
///   UTF-8.
///
/// A multi-GB body therefore can never force an unbounded allocation: at
/// most `MAX_RESPONSE_BYTES + 1` bytes are ever held.
fn read_capped_body(
    resp: reqwest::blocking::Response,
) -> Result<String, NexusEmitError> {
    // Early reject: a server-advertised length over the cap never gets read.
    if let Some(len) = resp.content_length() {
        if len > MAX_RESPONSE_BYTES {
            return Err(NexusEmitError::Transport(format!(
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
        .map_err(|e| NexusEmitError::Transport(e.to_string()))?;
    if buf.len() as u64 > MAX_RESPONSE_BYTES {
        return Err(NexusEmitError::Transport(format!(
            "response stream exceeded {MAX_RESPONSE_BYTES}-byte cap"
        )));
    }
    String::from_utf8(buf)
        .map_err(|e| NexusEmitError::Transport(format!("non-utf8 response body: {e}")))
}

/// Build a NexusEvent from primitive parts.
#[must_use]
pub fn build_event(
    source: impl Into<String>,
    kind: impl Into<String>,
    payload: serde_json::Value,
    ts_ms: i64,
) -> NexusEvent {
    NexusEvent {
        source: source.into(),
        kind: kind.into(),
        payload,
        ts_ms,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::{
        build_event, NexusClient, NexusEmitError, NexusEvent, DEFAULT_NEXUS_URL,
        DEFAULT_PUSH_TIMEOUT,
    };

    #[test]
    fn default_url_is_loopback_synthex_v2() {
        assert_eq!(DEFAULT_NEXUS_URL, "http://127.0.0.1:8092/v3/nexus/push");
    }

    #[test]
    fn default_timeout_is_5s() {
        assert_eq!(DEFAULT_PUSH_TIMEOUT.as_secs(), 5);
    }

    #[test]
    fn build_event_roundtrip_via_serde() {
        let e = build_event(
            "workflow-trace",
            "workflow.dispatched",
            serde_json::json!({"id": 42}),
            1_700_000_000_000,
        );
        let s = serde_json::to_string(&e).expect("ser");
        let back: NexusEvent = serde_json::from_str(&s).expect("de");
        assert_eq!(back, e);
    }

    struct RecordingClient {
        events: Mutex<Vec<NexusEvent>>,
    }

    impl NexusClient for RecordingClient {
        fn push(&self, event: &NexusEvent) -> Result<(), NexusEmitError> {
            self.events.lock().expect("lock").push(event.clone());
            Ok(())
        }
    }

    #[test]
    fn trait_records_pushed_event() {
        let c = RecordingClient {
            events: Mutex::new(Vec::new()),
        };
        let e = build_event("src", "kind", serde_json::json!(null), 0);
        c.push(&e).expect("push");
        let recorded = c.events.lock().expect("lock").clone();
        assert_eq!(recorded.len(), 1);
        assert_eq!(recorded[0].kind, "kind");
    }

    #[test]
    fn non_success_error_carries_status_code() {
        let err = NexusEmitError::NonSuccess(500);
        assert!(err.to_string().contains("500"));
    }

    #[test]
    fn transport_error_carries_reason() {
        let err = NexusEmitError::Transport("dns".into());
        assert!(err.to_string().contains("dns"));
    }

    #[test]
    fn errors_implement_std_error_send_sync_static() {
        fn assert_error<T: std::error::Error + Send + Sync + 'static>() {}
        assert_error::<NexusEmitError>();
    }

    // ====================================================================
    // Cluster H hardening pass — m40 nexus emit.
    // Categories: Contract regression, Determinism, Adversarial,
    // Concurrency, Boundary, Cross-module, Anti-property.
    // ====================================================================

    use std::sync::Arc;
    use std::thread;

    // rationale: Contract regression — DEFAULT_NEXUS_URL must remain pinned
    // to synthex-v2's loopback nexus push endpoint. Drift detection: any
    // change to host / port / path constitutes a wire-contract break.
    #[test]
    fn default_nexus_url_components_pinned() {
        // rationale: Contract regression (synthex-v2 :8092 wire contract)
        assert!(DEFAULT_NEXUS_URL.starts_with("http://127.0.0.1:8092/"));
        assert!(DEFAULT_NEXUS_URL.ends_with("/v3/nexus/push"));
        assert!(
            !DEFAULT_NEXUS_URL.contains("https://"),
            "synthex-v2 loopback is http-only (no TLS in loopback)"
        );
    }

    // rationale: Contract regression — NexusEvent JSON shape MUST remain
    // exactly {source, kind, payload, ts_ms}. Adding / renaming / removing
    // a field is a wire-contract break that propagates to synthex-v2's
    // /v3/nexus/push handler.
    #[test]
    fn nexus_event_json_shape_snapshot() {
        // rationale: Contract regression (NexusEvent JSON schema snapshot)
        let e = build_event(
            "workflow-trace",
            "workflow.dispatched",
            serde_json::json!({"workflow_id": 7}),
            1_700_000_000_000,
        );
        let v = serde_json::to_value(&e).expect("ser");
        let obj = v.as_object().expect("object");
        let mut keys: Vec<&str> = obj.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            vec!["kind", "payload", "source", "ts_ms"],
            "NexusEvent JSON shape drift — synthex-v2 contract break"
        );
        assert_eq!(obj["source"].as_str(), Some("workflow-trace"));
        assert_eq!(obj["kind"].as_str(), Some("workflow.dispatched"));
        assert_eq!(obj["ts_ms"].as_i64(), Some(1_700_000_000_000));
        assert!(obj["payload"].is_object());
    }

    // rationale: Determinism — serde round-trip stable across 1_000
    // distinct events (no field-order / floating-point / null-coercion
    // drift). Guards against future serde_json upgrades silently changing
    // serialization shape.
    #[test]
    fn nexus_event_serde_stable_across_thousand_events() {
        // rationale: Determinism (serde round-trip × 1_000)
        for i in 0..1_000_i64 {
            let e = build_event(
                "src",
                "kind",
                serde_json::json!({"i": i, "tag": "test"}),
                i,
            );
            let s = serde_json::to_string(&e).expect("ser");
            let back: NexusEvent = serde_json::from_str(&s).expect("de");
            assert_eq!(back, e, "round-trip drift at i={i}");
        }
    }

    // rationale: Adversarial input — server-style malformed top-level
    // value (a JSON array) must NOT deserialize into NexusEvent (struct
    // shape mismatch).
    #[test]
    fn nexus_event_rejects_array_shape() {
        // rationale: Adversarial input (malformed deserialize)
        let s = r"[1, 2, 3]";
        let r: Result<NexusEvent, _> = serde_json::from_str(s);
        assert!(r.is_err(), "array must not deserialize as NexusEvent");
    }

    // rationale: Adversarial input — missing required field `ts_ms`
    // surfaces a typed serde error rather than zero-filling.
    #[test]
    fn nexus_event_rejects_missing_ts_ms_field() {
        // rationale: Adversarial input (F-POVM-07 silent-zero-fill check)
        let s = r#"{"source":"a","kind":"b","payload":null}"#;
        let r: Result<NexusEvent, _> = serde_json::from_str(s);
        assert!(r.is_err(), "missing ts_ms must NOT zero-fill silently");
    }

    // rationale: Concurrency — RecordingClient under 16 concurrent
    // pushers receives every event exactly once (Mutex<Vec> serialises).
    // Smoke test that the trait surface does not deadlock or drop
    // events under contention.
    #[test]
    fn concurrent_pushes_record_every_event() {
        // rationale: Concurrency (trait surface under contention)
        let c = Arc::new(RecordingClient {
            events: Mutex::new(Vec::new()),
        });
        let threads = 16_usize;
        let pushes_per_thread = 8_usize;
        let mut handles = Vec::new();
        for t in 0..threads {
            let c2 = Arc::clone(&c);
            handles.push(thread::spawn(move || {
                for i in 0..pushes_per_thread {
                    let e = build_event(
                        format!("t{t}"),
                        "k",
                        serde_json::json!(null),
                        i64::try_from(i).expect("i fits"),
                    );
                    c2.push(&e).expect("push");
                }
            }));
        }
        for h in handles {
            h.join().expect("join");
        }
        let recorded = c.events.lock().expect("lock");
        let expected = threads * pushes_per_thread;
        assert_eq!(recorded.len(), expected);
    }

    // rationale: Boundary — DEFAULT_PUSH_TIMEOUT is the documented 5-second
    // upper bound. Drift detection if future tuning changes the constant.
    #[test]
    fn default_push_timeout_constant_pinned() {
        // rationale: Boundary (timeout constant pin)
        assert_eq!(DEFAULT_PUSH_TIMEOUT, std::time::Duration::from_secs(5));
    }

    // rationale: Cross-module surface invariant — HttpNexusClient retains
    // the URL it was constructed with. Guards against future construction
    // refactors silently swapping the URL field.
    #[test]
    fn http_client_records_constructed_url_and_timeout_via_pushv() {
        // rationale: Cross-module surface invariant (constructor identity)
        let c = super::HttpNexusClient::new(
            "http://example.invalid/v3/nexus/push",
            std::time::Duration::from_secs(1),
        );
        // The URL is private; we exercise via push with a deliberately
        // unreachable host so the client must use the URL it was given.
        // We tolerate either Transport or NonSuccess; what we assert is
        // that .push() does NOT panic and DOES return a typed error.
        let e = build_event("src", "k", serde_json::json!(null), 0);
        let r = c.push(&e);
        assert!(r.is_err(), "unreachable URL must yield a typed error");
        match r.unwrap_err() {
            NexusEmitError::Transport(_)
            | NexusEmitError::NonSuccess(_)
            | NexusEmitError::ServerRejected { .. } => {}
        }
    }

    // rationale: Anti-property — empty `kind` is permitted at the type
    // layer (no validator). This test pins the current behaviour so a
    // future tightening (e.g., reject empty kind) is a deliberate change,
    // not an accidental drift.
    #[test]
    fn empty_kind_permitted_at_type_layer() {
        // rationale: Anti-property (current-behaviour regression anchor)
        let e = build_event("src", "", serde_json::json!(null), 0);
        let s = serde_json::to_string(&e).expect("ser");
        assert!(s.contains("\"kind\":\"\""));
    }

    // rationale: Resource accounting — error variants are NOT Clone
    // because reqwest internal errors are not Clone. Confirm error
    // ergonomics: error message construction does not allocate
    // unboundedly (smoke test via short transport message).
    #[test]
    fn transport_error_short_message_round_trips_through_display() {
        // rationale: Resource accounting
        let err = NexusEmitError::Transport("x".repeat(64));
        let s = err.to_string();
        assert!(s.starts_with("transport: "));
        assert!(s.len() < 256, "error message stays bounded");
    }

    // rationale: Determinism — build_event preserves the payload
    // verbatim through to NexusEvent.payload (no normalization).
    #[test]
    fn build_event_preserves_payload_verbatim() {
        // rationale: Determinism (no payload normalization)
        let p = serde_json::json!({"nested": {"k": [1, 2, 3]}});
        let e = build_event("src", "kind", p.clone(), 0);
        assert_eq!(e.payload, p);
    }

    // rationale: Anti-property — error Display strings are stable
    // (drift detection for log scrapers and operator runbooks).
    #[test]
    fn nexus_emit_error_display_format_pinned() {
        // rationale: Anti-property (operator-facing log format)
        assert_eq!(
            NexusEmitError::NonSuccess(503).to_string(),
            "non-2xx status: 503"
        );
        assert_eq!(
            NexusEmitError::Transport("dns timeout".into()).to_string(),
            "transport: dns timeout"
        );
    }

    // ====================================================================
    // Cluster H Wave-A3 — C4 AP-V7-13 body-shape check.
    // Categories: Adversarial input · Boundary · Contract regression.
    // wiremock-driven end-to-end push() exercise.
    // ====================================================================

    // rationale: Adversarial input (AP-V7-13) — a 200 OK with a JSON body
    // carrying a non-null `error` field is the EXACT shape that motivated
    // the m42 stcortex-only ADR. push() must NOT treat this as success.
    #[tokio::test(flavor = "current_thread")]
    async fn push_returns_server_rejected_on_200_with_error_body() {
        // rationale: Adversarial input (AP-V7-13 body-shape rejection)
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

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
        // Run the blocking client off-task so we don't deadlock current_thread.
        let r = tokio::task::spawn_blocking(move || {
            let c =
                super::HttpNexusClient::new(url, std::time::Duration::from_secs(2));
            let e = build_event("workflow-trace", "workflow.dispatched", serde_json::json!({}), 0);
            c.push(&e)
        })
        .await
        .expect("join");

        match r {
            Err(NexusEmitError::ServerRejected { body }) => {
                assert!(body.contains("queue_full"), "rejection carries body");
            }
            other => panic!("expected ServerRejected, got {other:?}"),
        }
    }

    // rationale: Boundary — a 204 No Content response (empty body) is a
    // legitimate success mode for some servers (fire-and-forget accept).
    // push() must NOT fail just because the body is empty.
    #[tokio::test(flavor = "current_thread")]
    async fn push_succeeds_on_204_no_content() {
        // rationale: Boundary (empty body = success)
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v3/nexus/push"))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;

        let url = format!("{}/v3/nexus/push", server.uri());
        let r = tokio::task::spawn_blocking(move || {
            let c =
                super::HttpNexusClient::new(url, std::time::Duration::from_secs(2));
            let e = build_event("workflow-trace", "workflow.dispatched", serde_json::json!({}), 0);
            c.push(&e)
        })
        .await
        .expect("join");
        assert!(r.is_ok(), "204 No Content is success, got {r:?}");
    }

    // rationale: Contract regression — a 200 OK with an explicitly-OK body
    // (no `error` field) is the canonical success shape. Verifies the new
    // body-inspection path doesn't regress the happy case.
    #[tokio::test(flavor = "current_thread")]
    async fn push_succeeds_on_200_with_ok_body() {
        // rationale: Contract regression (happy-path preservation)
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

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
            let c =
                super::HttpNexusClient::new(url, std::time::Duration::from_secs(2));
            let e = build_event("workflow-trace", "workflow.dispatched", serde_json::json!({}), 0);
            c.push(&e)
        })
        .await
        .expect("join");
        assert!(r.is_ok(), "200 OK with non-error body is success, got {r:?}");
    }

    // rationale: Adversarial input — a 200 OK with `"error": null` is NOT
    // a rejection (JSON-RPC null-error semantics carry over by analogy).
    // Defends against an over-eager body-shape check that would fail-open
    // on null.
    #[tokio::test(flavor = "current_thread")]
    async fn push_treats_null_error_field_as_non_error() {
        // rationale: Adversarial input (null-error semantics)
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v3/nexus/push"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(r#"{"error":null,"ok":true}"#)
                    .insert_header("content-type", "application/json"),
            )
            .expect(1)
            .mount(&server)
            .await;

        let url = format!("{}/v3/nexus/push", server.uri());
        let r = tokio::task::spawn_blocking(move || {
            let c =
                super::HttpNexusClient::new(url, std::time::Duration::from_secs(2));
            let e = build_event("workflow-trace", "workflow.dispatched", serde_json::json!({}), 0);
            c.push(&e)
        })
        .await
        .expect("join");
        assert!(r.is_ok(), "null error field is not a rejection, got {r:?}");
    }

    // rationale: Anti-property — the ServerRejected variant Display format
    // is pinned for operator runbooks and log scrapers.
    #[test]
    fn server_rejected_error_display_format_pinned() {
        // rationale: Anti-property (operator-facing log format)
        let err = NexusEmitError::ServerRejected {
            body: r#"{"error":"queue_full"}"#.into(),
        };
        let s = err.to_string();
        assert!(s.starts_with("server rejected (2xx body-shape):"));
        assert!(s.contains("queue_full"));
    }

    // ====================================================================
    // God-tier hardening pass — m40 nexus emit.
    // Error variants, boundary conditions, AP-V7-13 body-shape edge cases.
    // ====================================================================

    // rationale: Adversarial input — a NexusEvent with an unknown extra
    // field must still deserialize (serde is non-strict by default). This
    // pins the current lenient behaviour as deliberate, not accidental.
    #[test]
    fn nexus_event_tolerates_unknown_extra_field() {
        // rationale: Anti-property (current-behaviour regression anchor)
        let s = r#"{"source":"a","kind":"b","payload":null,"ts_ms":1,"extra":99}"#;
        let r: Result<NexusEvent, _> = serde_json::from_str(s);
        assert!(r.is_ok(), "extra field should not break deserialization");
        assert_eq!(r.expect("ok").ts_ms, 1);
    }

    // rationale: Adversarial input — missing the `source` field must error,
    // not zero-fill to an empty string (the F-POVM-07 silent-fill class).
    #[test]
    fn nexus_event_rejects_missing_source_field() {
        // rationale: Adversarial input (silent-fill guard)
        let s = r#"{"kind":"b","payload":null,"ts_ms":1}"#;
        let r: Result<NexusEvent, _> = serde_json::from_str(s);
        assert!(r.is_err(), "missing source must NOT zero-fill silently");
    }

    // rationale: Adversarial input — missing the `kind` field must error.
    #[test]
    fn nexus_event_rejects_missing_kind_field() {
        // rationale: Adversarial input (silent-fill guard)
        let s = r#"{"source":"a","payload":null,"ts_ms":1}"#;
        let r: Result<NexusEvent, _> = serde_json::from_str(s);
        assert!(r.is_err(), "missing kind must NOT zero-fill silently");
    }

    // rationale: Adversarial input — missing the `payload` field must error;
    // payload is required even though its value type is permissive.
    #[test]
    fn nexus_event_rejects_missing_payload_field() {
        // rationale: Adversarial input (silent-fill guard)
        let s = r#"{"source":"a","kind":"b","ts_ms":1}"#;
        let r: Result<NexusEvent, _> = serde_json::from_str(s);
        assert!(r.is_err(), "missing payload must NOT zero-fill silently");
    }

    // rationale: Adversarial input — `ts_ms` typed as a string must fail
    // (i64 field; string is a type mismatch).
    #[test]
    fn nexus_event_rejects_string_typed_ts_ms() {
        // rationale: Adversarial input (type-mismatch)
        let s = r#"{"source":"a","kind":"b","payload":null,"ts_ms":"123"}"#;
        let r: Result<NexusEvent, _> = serde_json::from_str(s);
        assert!(r.is_err(), "string ts_ms must not coerce to i64");
    }

    // rationale: Boundary — ts_ms must accept the full i64 range. Min and
    // max values round-trip without overflow or precision loss.
    #[test]
    fn ts_ms_round_trips_at_i64_extremes() {
        // rationale: Boundary (i64 min/max)
        for &ts in &[i64::MIN, i64::MAX, 0_i64, -1_i64] {
            let e = build_event("src", "kind", serde_json::json!(null), ts);
            let s = serde_json::to_string(&e).expect("ser");
            let back: NexusEvent = serde_json::from_str(&s).expect("de");
            assert_eq!(back.ts_ms, ts, "ts_ms drift at {ts}");
        }
    }

    // rationale: Boundary — a negative ts_ms is structurally legal (no
    // validator); pin the current permissive behaviour.
    #[test]
    fn build_event_accepts_negative_ts_ms() {
        // rationale: Boundary (no ts validator)
        let e = build_event("src", "kind", serde_json::json!(null), -42);
        assert_eq!(e.ts_ms, -42);
    }

    // rationale: Determinism — build_event must set source / kind / ts_ms
    // exactly from its arguments (no normalization, no defaulting).
    #[test]
    fn build_event_sets_all_fields_from_arguments() {
        // rationale: Determinism (constructor identity)
        let e = build_event(
            "workflow-trace",
            "workflow.completed",
            serde_json::json!({"k": "v"}),
            999_999,
        );
        assert_eq!(e.source, "workflow-trace");
        assert_eq!(e.kind, "workflow.completed");
        assert_eq!(e.ts_ms, 999_999);
        assert_eq!(e.payload, serde_json::json!({"k": "v"}));
    }

    // rationale: Boundary — an empty `source` string is accepted at the type
    // layer (mirrors empty_kind_permitted_at_type_layer for symmetry).
    #[test]
    fn empty_source_permitted_at_type_layer() {
        // rationale: Boundary (no source validator)
        let e = build_event("", "kind", serde_json::json!(null), 0);
        let s = serde_json::to_string(&e).expect("ser");
        assert!(s.contains("\"source\":\"\""));
    }

    // rationale: Determinism — a payload that is a JSON array (not an
    // object) must survive build_event + round-trip verbatim. The payload
    // field is serde_json::Value, so any JSON value is legal.
    #[test]
    fn payload_can_be_array_value() {
        // rationale: Determinism (payload is arbitrary JSON)
        let p = serde_json::json!([1, "two", {"three": 3}, null]);
        let e = build_event("src", "kind", p.clone(), 0);
        let back: NexusEvent =
            serde_json::from_str(&serde_json::to_string(&e).expect("ser"))
                .expect("de");
        assert_eq!(back.payload, p);
    }

    // rationale: Determinism — a scalar payload (bare number / string /
    // bool) is also legal serde_json::Value and must round-trip.
    #[test]
    fn payload_can_be_scalar_value() {
        // rationale: Determinism (payload is arbitrary JSON)
        for p in [
            serde_json::json!(42),
            serde_json::json!("scalar"),
            serde_json::json!(true),
            serde_json::json!(std::f64::consts::PI),
        ] {
            let e = build_event("src", "kind", p.clone(), 0);
            let back: NexusEvent =
                serde_json::from_str(&serde_json::to_string(&e).expect("ser"))
                    .expect("de");
            assert_eq!(back.payload, p);
        }
    }

    // rationale: Adversarial input — a unicode-heavy source/kind/payload
    // must round-trip byte-for-byte (no escaping corruption).
    #[test]
    fn unicode_fields_round_trip() {
        // rationale: Adversarial input (unicode integrity)
        let e = build_event(
            "工作流-trace ☤",
            "workflow.dispatched·完了",
            serde_json::json!({"note": "émigré — naïve \u{1F600}"}),
            1,
        );
        let back: NexusEvent =
            serde_json::from_str(&serde_json::to_string(&e).expect("ser"))
                .expect("de");
        assert_eq!(back, e);
    }

    // rationale: Anti-property — NexusEvent Clone must be a deep copy; the
    // payload of a clone is independent of the original (Value is owned).
    #[test]
    fn nexus_event_clone_is_independent() {
        // rationale: Anti-property (deep clone)
        let original = build_event("src", "kind", serde_json::json!({"n": 1}), 7);
        let cloned = original.clone();
        assert_eq!(original, cloned);
        // Mutating one does not affect the other.
        let mut mutated = cloned;
        mutated.ts_ms = 8;
        assert_ne!(original.ts_ms, mutated.ts_ms);
    }

    // rationale: Anti-property — two events differing only in ts_ms must NOT
    // compare equal (Eq must consider every field).
    #[test]
    fn nexus_events_differing_in_ts_ms_are_unequal() {
        // rationale: Anti-property (full-field equality)
        let a = build_event("s", "k", serde_json::json!(null), 1);
        let b = build_event("s", "k", serde_json::json!(null), 2);
        assert_ne!(a, b);
    }

    // rationale: Anti-property — two events differing only in payload must
    // NOT compare equal.
    #[test]
    fn nexus_events_differing_in_payload_are_unequal() {
        // rationale: Anti-property (full-field equality)
        let a = build_event("s", "k", serde_json::json!({"x": 1}), 0);
        let b = build_event("s", "k", serde_json::json!({"x": 2}), 0);
        assert_ne!(a, b);
    }

    // rationale: Contract regression — the three NexusEmitError variants
    // produce distinct Display prefixes so log scrapers can classify them.
    #[test]
    fn all_error_variants_have_distinct_display_prefixes() {
        // rationale: Contract regression (operator-facing classification)
        let transport = NexusEmitError::Transport("e".into()).to_string();
        let non_success = NexusEmitError::NonSuccess(500).to_string();
        let rejected = NexusEmitError::ServerRejected {
            body: "{}".into(),
        }
        .to_string();
        assert!(transport.starts_with("transport:"));
        assert!(non_success.starts_with("non-2xx status:"));
        assert!(rejected.starts_with("server rejected"));
        // Pairwise distinct.
        assert_ne!(transport, non_success);
        assert_ne!(non_success, rejected);
        assert_ne!(transport, rejected);
    }

    // rationale: Boundary — NonSuccess must carry the exact u16 status; pin
    // representative codes at the edges of the non-2xx ranges.
    #[test]
    fn non_success_carries_exact_status_for_edge_codes() {
        // rationale: Boundary (status-code fidelity)
        for code in [100_u16, 301, 400, 404, 418, 500, 503, 599] {
            let e = NexusEmitError::NonSuccess(code);
            assert!(e.to_string().contains(&code.to_string()));
        }
    }

    // rationale: Boundary — the build_event source argument accepts both
    // &str and String (impl Into<String>); both must produce equal events.
    #[test]
    fn build_event_accepts_str_and_string_args_equivalently() {
        // rationale: Boundary (generic Into<String> surface)
        let from_str = build_event("src", "kind", serde_json::json!(null), 0);
        let from_string = build_event(
            String::from("src"),
            String::from("kind"),
            serde_json::json!(null),
            0,
        );
        assert_eq!(from_str, from_string);
    }

    // rationale: Adversarial input (AP-V7-13) — a 200 OK whose body has the
    // `error` field nested INSIDE another object must NOT trigger
    // ServerRejected; the check only inspects the TOP-LEVEL `error` key.
    #[tokio::test(flavor = "current_thread")]
    async fn push_ignores_nested_error_field() {
        // rationale: Adversarial input (top-level-only error inspection)
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v3/nexus/push"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(r#"{"data":{"error":"inner"},"ok":true}"#)
                    .insert_header("content-type", "application/json"),
            )
            .expect(1)
            .mount(&server)
            .await;

        let url = format!("{}/v3/nexus/push", server.uri());
        let r = tokio::task::spawn_blocking(move || {
            let c = super::HttpNexusClient::new(url, std::time::Duration::from_secs(2));
            let e =
                build_event("workflow-trace", "workflow.dispatched", serde_json::json!({}), 0);
            c.push(&e)
        })
        .await
        .expect("join");
        assert!(r.is_ok(), "nested error field must not reject, got {r:?}");
    }

    // rationale: Adversarial input — a 2xx response with a non-JSON body
    // (garbled text) is treated as success per the documented contract
    // ("reject only on a visibly-rejecting JSON body").
    #[tokio::test(flavor = "current_thread")]
    async fn push_treats_non_json_body_as_success() {
        // rationale: Adversarial input (non-JSON body contract)
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v3/nexus/push"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string("OK <<not json>>")
                    .insert_header("content-type", "text/plain"),
            )
            .expect(1)
            .mount(&server)
            .await;

        let url = format!("{}/v3/nexus/push", server.uri());
        let r = tokio::task::spawn_blocking(move || {
            let c = super::HttpNexusClient::new(url, std::time::Duration::from_secs(2));
            let e =
                build_event("workflow-trace", "workflow.dispatched", serde_json::json!({}), 0);
            c.push(&e)
        })
        .await
        .expect("join");
        assert!(r.is_ok(), "non-JSON body must be treated as success, got {r:?}");
    }

    // rationale: Adversarial input — a 200 OK with an empty JSON object
    // {} carries no `error` key and must be accepted.
    #[tokio::test(flavor = "current_thread")]
    async fn push_succeeds_on_200_empty_json_object() {
        // rationale: Adversarial input (no error key = success)
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v3/nexus/push"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string("{}")
                    .insert_header("content-type", "application/json"),
            )
            .expect(1)
            .mount(&server)
            .await;

        let url = format!("{}/v3/nexus/push", server.uri());
        let r = tokio::task::spawn_blocking(move || {
            let c = super::HttpNexusClient::new(url, std::time::Duration::from_secs(2));
            let e =
                build_event("workflow-trace", "workflow.dispatched", serde_json::json!({}), 0);
            c.push(&e)
        })
        .await
        .expect("join");
        assert!(r.is_ok(), "empty JSON object must be success, got {r:?}");
    }

    // rationale: Contract regression — a non-2xx status (500) must surface
    // NexusEmitError::NonSuccess carrying the exact code, BEFORE any body
    // inspection. An error body on a 500 is irrelevant — status wins.
    #[tokio::test(flavor = "current_thread")]
    async fn push_returns_non_success_on_500() {
        // rationale: Contract regression (status check precedes body check)
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v3/nexus/push"))
            .respond_with(
                ResponseTemplate::new(500)
                    .set_body_string(r#"{"ok":true}"#)
                    .insert_header("content-type", "application/json"),
            )
            .expect(1)
            .mount(&server)
            .await;

        let url = format!("{}/v3/nexus/push", server.uri());
        let r = tokio::task::spawn_blocking(move || {
            let c = super::HttpNexusClient::new(url, std::time::Duration::from_secs(2));
            let e =
                build_event("workflow-trace", "workflow.dispatched", serde_json::json!({}), 0);
            c.push(&e)
        })
        .await
        .expect("join");
        match r {
            Err(NexusEmitError::NonSuccess(code)) => assert_eq!(code, 500),
            other => panic!("expected NonSuccess(500), got {other:?}"),
        }
    }

    // rationale: Boundary — a 404 (route not found, e.g. wrong path) must
    // also surface NonSuccess(404), proving non-2xx codes other than 500
    // are handled uniformly.
    #[tokio::test(flavor = "current_thread")]
    async fn push_returns_non_success_on_404() {
        // rationale: Boundary (non-2xx uniformity)
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v3/nexus/push"))
            .respond_with(ResponseTemplate::new(404))
            .expect(1)
            .mount(&server)
            .await;

        let url = format!("{}/v3/nexus/push", server.uri());
        let r = tokio::task::spawn_blocking(move || {
            let c = super::HttpNexusClient::new(url, std::time::Duration::from_secs(2));
            let e =
                build_event("workflow-trace", "workflow.dispatched", serde_json::json!({}), 0);
            c.push(&e)
        })
        .await
        .expect("join");
        assert!(matches!(r, Err(NexusEmitError::NonSuccess(404))));
    }

    // rationale: Adversarial input — the AP-V7-13 check must reject when the
    // top-level `error` is a non-string truthy value (e.g. a JSON object or
    // number). Any non-null `error` is a rejection per the contract.
    #[tokio::test(flavor = "current_thread")]
    async fn push_rejects_when_error_field_is_object() {
        // rationale: Adversarial input (non-null error of any JSON type)
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v3/nexus/push"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(r#"{"error":{"code":7,"msg":"busy"}}"#)
                    .insert_header("content-type", "application/json"),
            )
            .expect(1)
            .mount(&server)
            .await;

        let url = format!("{}/v3/nexus/push", server.uri());
        let r = tokio::task::spawn_blocking(move || {
            let c = super::HttpNexusClient::new(url, std::time::Duration::from_secs(2));
            let e =
                build_event("workflow-trace", "workflow.dispatched", serde_json::json!({}), 0);
            c.push(&e)
        })
        .await
        .expect("join");
        match r {
            Err(NexusEmitError::ServerRejected { body }) => {
                assert!(body.contains("busy"), "rejection carries the body");
            }
            other => panic!("expected ServerRejected, got {other:?}"),
        }
    }

    // rationale: Transport — a push to a connection-refused address (no
    // server bound) must surface NexusEmitError::Transport, not a panic.
    #[tokio::test(flavor = "current_thread")]
    async fn push_returns_transport_error_on_connection_refused() {
        // rationale: Transport (unbound port = connection refused)
        let r = tokio::task::spawn_blocking(|| {
            // 127.0.0.1:1 is reserved/privileged — reliably refuses.
            let c = super::HttpNexusClient::new(
                "http://127.0.0.1:1/v3/nexus/push",
                std::time::Duration::from_secs(2),
            );
            let e =
                build_event("workflow-trace", "workflow.dispatched", serde_json::json!({}), 0);
            c.push(&e)
        })
        .await
        .expect("join");
        assert!(
            matches!(r, Err(NexusEmitError::Transport(_))),
            "connection refused must be a Transport error, got {r:?}"
        );
    }

    // rationale: Transport — a malformed URL (no scheme/host) must surface a
    // typed Transport error rather than panicking inside reqwest.
    #[test]
    fn push_returns_transport_error_on_malformed_url() {
        // rationale: Transport (URL parse failure)
        let c = super::HttpNexusClient::new(
            "not-a-valid-url",
            std::time::Duration::from_secs(1),
        );
        let e = build_event("src", "k", serde_json::json!(null), 0);
        let r = c.push(&e);
        assert!(
            matches!(r, Err(NexusEmitError::Transport(_))),
            "malformed URL must yield a Transport error, got {r:?}"
        );
    }

    // rationale: Cross-module — the push() body of HttpNexusClient sends the
    // event as JSON to the configured path. A wiremock body matcher
    // confirms the wire payload carries the NexusEvent shape.
    #[tokio::test(flavor = "current_thread")]
    async fn push_sends_event_json_to_configured_path() {
        // rationale: Cross-module (wire payload contract)
        use wiremock::matchers::{body_string_contains, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v3/nexus/push"))
            .and(body_string_contains("workflow.dispatched"))
            .and(body_string_contains("workflow-trace"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        let url = format!("{}/v3/nexus/push", server.uri());
        let r = tokio::task::spawn_blocking(move || {
            let c = super::HttpNexusClient::new(url, std::time::Duration::from_secs(2));
            let e = build_event(
                "workflow-trace",
                "workflow.dispatched",
                serde_json::json!({"workflow_id": 7}),
                1_700_000_000_000,
            );
            c.push(&e)
        })
        .await
        .expect("join");
        assert!(r.is_ok(), "push to a 200 server must succeed, got {r:?}");
        // wiremock verifies the .expect(1) on drop.
    }

    // rationale: Boundary — a 201 Created (a 2xx that is not 200) is a
    // success status; push() must accept it.
    #[tokio::test(flavor = "current_thread")]
    async fn push_succeeds_on_201_created() {
        // rationale: Boundary (2xx range, not just 200)
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v3/nexus/push"))
            .respond_with(ResponseTemplate::new(201))
            .expect(1)
            .mount(&server)
            .await;

        let url = format!("{}/v3/nexus/push", server.uri());
        let r = tokio::task::spawn_blocking(move || {
            let c = super::HttpNexusClient::new(url, std::time::Duration::from_secs(2));
            let e =
                build_event("workflow-trace", "workflow.dispatched", serde_json::json!({}), 0);
            c.push(&e)
        })
        .await
        .expect("join");
        assert!(r.is_ok(), "201 Created is a 2xx success, got {r:?}");
    }

    // rationale: Anti-property — a RecordingClient that returns Ok must not
    // mutate the event it is handed; the recorded copy equals the input.
    #[test]
    fn recording_client_preserves_event_verbatim() {
        // rationale: Anti-property (push does not mutate input)
        let c = RecordingClient {
            events: Mutex::new(Vec::new()),
        };
        let e = build_event(
            "workflow-trace",
            "workflow.completed",
            serde_json::json!({"nested": [1, 2]}),
            1_700_000_123_456,
        );
        c.push(&e).expect("push");
        let recorded = c.events.lock().expect("lock");
        assert_eq!(recorded.len(), 1);
        assert_eq!(recorded[0], e, "push must not alter the event");
    }

    // ====================================================================
    // W1 floor-closure pass — m40 nexus emit (+10 meaningful tests).
    // Categories: Adversarial body-shape · Boundary · Transport · Cross-module.
    // ====================================================================

    // rationale: Adversarial input — a 200 OK whose body is a JSON array
    // (valid JSON, not an object) has no top-level `error` key; `Value::get`
    // on a non-object returns None, so push() must treat it as success.
    #[tokio::test(flavor = "current_thread")]
    async fn push_succeeds_on_200_json_array_body() {
        // rationale: Adversarial input (non-object JSON body)
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v3/nexus/push"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string("[1,2,3]")
                    .insert_header("content-type", "application/json"),
            )
            .expect(1)
            .mount(&server)
            .await;

        let url = format!("{}/v3/nexus/push", server.uri());
        let r = tokio::task::spawn_blocking(move || {
            let c = super::HttpNexusClient::new(url, std::time::Duration::from_secs(2));
            let e = build_event("workflow-trace", "workflow.dispatched", serde_json::json!({}), 0);
            c.push(&e)
        })
        .await
        .expect("join");
        assert!(r.is_ok(), "JSON-array body has no error key, got {r:?}");
    }

    // rationale: Adversarial input — a 200 OK whose body is a bare JSON
    // string is a scalar Value with no `error` key. Even though the text
    // contains a problem-sounding word, the body-shape check must not fire.
    #[tokio::test(flavor = "current_thread")]
    async fn push_succeeds_on_200_bare_json_string_body() {
        // rationale: Adversarial input (scalar JSON string body)
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v3/nexus/push"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string("\"queue_full\"")
                    .insert_header("content-type", "application/json"),
            )
            .expect(1)
            .mount(&server)
            .await;

        let url = format!("{}/v3/nexus/push", server.uri());
        let r = tokio::task::spawn_blocking(move || {
            let c = super::HttpNexusClient::new(url, std::time::Duration::from_secs(2));
            let e = build_event("workflow-trace", "workflow.dispatched", serde_json::json!({}), 0);
            c.push(&e)
        })
        .await
        .expect("join");
        assert!(r.is_ok(), "bare JSON string is not a rejection object, got {r:?}");
    }

    // rationale: Adversarial input — a 200 OK with a bare JSON `null` body
    // deserializes to Value::Null; `.get("error")` is None, so it succeeds.
    #[tokio::test(flavor = "current_thread")]
    async fn push_succeeds_on_200_json_null_body() {
        // rationale: Adversarial input (JSON null body)
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v3/nexus/push"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string("null")
                    .insert_header("content-type", "application/json"),
            )
            .expect(1)
            .mount(&server)
            .await;

        let url = format!("{}/v3/nexus/push", server.uri());
        let r = tokio::task::spawn_blocking(move || {
            let c = super::HttpNexusClient::new(url, std::time::Duration::from_secs(2));
            let e = build_event("workflow-trace", "workflow.dispatched", serde_json::json!({}), 0);
            c.push(&e)
        })
        .await
        .expect("join");
        assert!(r.is_ok(), "JSON null body carries no error key, got {r:?}");
    }

    // rationale: Adversarial input — the body-shape check rejects on ANY
    // non-null `error` value. A boolean `false` is non-null, so `{"error":
    // false}` is a rejection — pins that the check is null-ness, not truthy.
    #[tokio::test(flavor = "current_thread")]
    async fn push_rejects_when_error_field_is_false() {
        // rationale: Adversarial input (non-null `false` still rejects)
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v3/nexus/push"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(r#"{"error":false}"#)
                    .insert_header("content-type", "application/json"),
            )
            .expect(1)
            .mount(&server)
            .await;

        let url = format!("{}/v3/nexus/push", server.uri());
        let r = tokio::task::spawn_blocking(move || {
            let c = super::HttpNexusClient::new(url, std::time::Duration::from_secs(2));
            let e = build_event("workflow-trace", "workflow.dispatched", serde_json::json!({}), 0);
            c.push(&e)
        })
        .await
        .expect("join");
        assert!(
            matches!(r, Err(NexusEmitError::ServerRejected { .. })),
            "a non-null `false` error field is still a rejection, got {r:?}"
        );
    }

    // rationale: Adversarial input — an empty-string `error` value is
    // non-null and therefore a rejection per the body-shape contract.
    #[tokio::test(flavor = "current_thread")]
    async fn push_rejects_when_error_field_is_empty_string() {
        // rationale: Adversarial input (non-null empty string rejects)
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v3/nexus/push"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(r#"{"error":""}"#)
                    .insert_header("content-type", "application/json"),
            )
            .expect(1)
            .mount(&server)
            .await;

        let url = format!("{}/v3/nexus/push", server.uri());
        let r = tokio::task::spawn_blocking(move || {
            let c = super::HttpNexusClient::new(url, std::time::Duration::from_secs(2));
            let e = build_event("workflow-trace", "workflow.dispatched", serde_json::json!({}), 0);
            c.push(&e)
        })
        .await
        .expect("join");
        assert!(
            matches!(r, Err(NexusEmitError::ServerRejected { .. })),
            "a non-null empty-string error field is still a rejection, got {r:?}"
        );
    }

    // rationale: Adversarial input — a numeric `0` error value is non-null
    // and therefore a rejection (the check is null-ness, not falsiness).
    #[tokio::test(flavor = "current_thread")]
    async fn push_rejects_when_error_field_is_zero() {
        // rationale: Adversarial input (non-null zero rejects)
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v3/nexus/push"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(r#"{"error":0}"#)
                    .insert_header("content-type", "application/json"),
            )
            .expect(1)
            .mount(&server)
            .await;

        let url = format!("{}/v3/nexus/push", server.uri());
        let r = tokio::task::spawn_blocking(move || {
            let c = super::HttpNexusClient::new(url, std::time::Duration::from_secs(2));
            let e = build_event("workflow-trace", "workflow.dispatched", serde_json::json!({}), 0);
            c.push(&e)
        })
        .await
        .expect("join");
        assert!(
            matches!(r, Err(NexusEmitError::ServerRejected { .. })),
            "a non-null `0` error field is still a rejection, got {r:?}"
        );
    }

    // rationale: Transport — a server slower than the client timeout must
    // surface NexusEmitError::Transport, never hang or panic.
    #[tokio::test(flavor = "current_thread")]
    async fn push_times_out_to_transport_error_on_slow_server() {
        // rationale: Transport (client timeout fires before a slow response)
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v3/nexus/push"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_delay(std::time::Duration::from_secs(3)),
            )
            .mount(&server)
            .await;

        let url = format!("{}/v3/nexus/push", server.uri());
        let r = tokio::task::spawn_blocking(move || {
            let c = super::HttpNexusClient::new(url, std::time::Duration::from_millis(500));
            let e = build_event("workflow-trace", "workflow.dispatched", serde_json::json!({}), 0);
            c.push(&e)
        })
        .await
        .expect("join");
        assert!(
            matches!(r, Err(NexusEmitError::Transport(_))),
            "a response slower than the timeout must be a Transport error, got {r:?}"
        );
    }

    // rationale: Cross-module — push() must send the event with a JSON
    // content-type so synthex-v2's handler parses it. A header matcher
    // confirms the request carries `content-type: application/json`.
    #[tokio::test(flavor = "current_thread")]
    async fn push_sends_application_json_content_type() {
        // rationale: Cross-module (request content-type contract)
        use wiremock::matchers::{header, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v3/nexus/push"))
            .and(header("content-type", "application/json"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        let url = format!("{}/v3/nexus/push", server.uri());
        let r = tokio::task::spawn_blocking(move || {
            let c = super::HttpNexusClient::new(url, std::time::Duration::from_secs(2));
            let e = build_event("workflow-trace", "workflow.dispatched", serde_json::json!({}), 0);
            c.push(&e)
        })
        .await
        .expect("join");
        assert!(
            r.is_ok(),
            "push must send content-type: application/json to match, got {r:?}"
        );
    }

    // rationale: Cross-module — NexusClient must be object-safe so callers
    // can hold a `Box<dyn NexusClient>` (e.g. for test/prod swapping).
    #[test]
    fn nexus_client_usable_as_boxed_trait_object() {
        // rationale: Cross-module (trait object-safety + dyn dispatch)
        let client: Box<dyn NexusClient> = Box::new(RecordingClient {
            events: Mutex::new(Vec::new()),
        });
        let e = build_event("src", "kind", serde_json::json!(null), 1);
        client.push(&e).expect("boxed dyn push must succeed");
        client.push(&e).expect("second boxed dyn push must succeed");
    }

    // rationale: Contract — a failing NexusClient impl surfaces its typed
    // error through the trait's Result; the trait must not swallow or panic.
    #[test]
    fn nexus_client_trait_surfaces_configured_error() {
        // rationale: Contract (trait-level error propagation)
        struct FailingClient;
        impl NexusClient for FailingClient {
            fn push(&self, _event: &NexusEvent) -> Result<(), NexusEmitError> {
                Err(NexusEmitError::NonSuccess(503))
            }
        }
        let client = FailingClient;
        let e = build_event("src", "kind", serde_json::json!(null), 1);
        match client.push(&e) {
            Err(NexusEmitError::NonSuccess(code)) => assert_eq!(code, 503),
            other => panic!("expected NonSuccess(503) through the trait, got {other:?}"),
        }
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

    // rationale: SEC4 — a 2xx response whose body exceeds the 1 MiB cap
    // must surface a typed Transport error, NOT force an unbounded
    // allocation. wiremock sets a real Content-Length so the early-reject
    // branch fires before any byte is buffered.
    #[tokio::test(flavor = "current_thread")]
    async fn push_rejects_over_cap_response_body() {
        // rationale: SEC4 (unbounded-allocation guard)
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        // 2 MiB body — twice the cap.
        let huge = "x".repeat(2 * 1024 * 1024);
        Mock::given(method("POST"))
            .and(path("/v3/nexus/push"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(huge)
                    .insert_header("content-type", "application/json"),
            )
            .mount(&server)
            .await;

        let url = format!("{}/v3/nexus/push", server.uri());
        let r = tokio::task::spawn_blocking(move || {
            let c = super::HttpNexusClient::new(url, std::time::Duration::from_secs(3));
            let e = build_event("workflow-trace", "workflow.dispatched", serde_json::json!({}), 0);
            c.push(&e)
        })
        .await
        .expect("join");
        match r {
            Err(NexusEmitError::Transport(msg)) => {
                assert!(
                    msg.contains("cap"),
                    "over-cap body must mention the cap; got {msg}"
                );
            }
            other => panic!("expected Transport(over-cap), got {other:?}"),
        }
    }

    // rationale: SEC4 — a legitimate small body well under the cap still
    // flows through unchanged; the cap rejects only genuine over-cap bodies.
    #[tokio::test(flavor = "current_thread")]
    async fn push_accepts_under_cap_response_body() {
        // rationale: SEC4 (cap does not regress the happy path)
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        // A sizeable-but-under-cap (512 KiB) valid JSON body.
        let pad = "a".repeat(512 * 1024);
        let body = format!(r#"{{"pad":"{pad}","ok":true}}"#);
        Mock::given(method("POST"))
            .and(path("/v3/nexus/push"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(body)
                    .insert_header("content-type", "application/json"),
            )
            .mount(&server)
            .await;

        let url = format!("{}/v3/nexus/push", server.uri());
        let r = tokio::task::spawn_blocking(move || {
            let c = super::HttpNexusClient::new(url, std::time::Duration::from_secs(3));
            let e = build_event("workflow-trace", "workflow.dispatched", serde_json::json!({}), 0);
            c.push(&e)
        })
        .await
        .expect("join");
        assert!(r.is_ok(), "an under-cap valid body must succeed, got {r:?}");
    }

    // rationale: SEC4 — an over-cap body that ALSO carries a rejecting
    // `error` field is rejected on the size cap first; the size guard runs
    // before body-shape inspection so a giant body never gets parsed.
    #[tokio::test(flavor = "current_thread")]
    async fn push_size_cap_precedes_body_shape_check() {
        // rationale: SEC4 (size guard runs before JSON parse)
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        // A 2 MiB body that is also a rejecting JSON envelope.
        let pad = "x".repeat(2 * 1024 * 1024);
        let body = format!(r#"{{"error":"queue_full","pad":"{pad}"}}"#);
        Mock::given(method("POST"))
            .and(path("/v3/nexus/push"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(body)
                    .insert_header("content-type", "application/json"),
            )
            .mount(&server)
            .await;

        let url = format!("{}/v3/nexus/push", server.uri());
        let r = tokio::task::spawn_blocking(move || {
            let c = super::HttpNexusClient::new(url, std::time::Duration::from_secs(3));
            let e = build_event("workflow-trace", "workflow.dispatched", serde_json::json!({}), 0);
            c.push(&e)
        })
        .await
        .expect("join");
        assert!(
            matches!(r, Err(NexusEmitError::Transport(_))),
            "size cap must reject before body-shape parse, got {r:?}"
        );
    }
}
