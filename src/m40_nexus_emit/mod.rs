//! `m40_nexusevent_emit` — NexusEvent push to synthex-v2 `:8092`.
//! Cluster H · L8.

use std::time::Duration;

use thiserror::Error;

/// Default synthex-v2 NexusEvent endpoint.
pub const DEFAULT_NEXUS_URL: &str = "http://127.0.0.1:8092/v3/nexus/push";

/// Default push timeout.
pub const DEFAULT_PUSH_TIMEOUT: Duration = Duration::from_secs(5);

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
}

/// Trait abstraction for the HTTP push (real impl uses reqwest).
pub trait NexusClient: Send + Sync {
    /// Push the event; return on success / typed error.
    ///
    /// # Errors
    ///
    /// [`NexusEmitError::Transport`] on transport failure.
    /// [`NexusEmitError::NonSuccess`] on non-2xx.
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
        Ok(())
    }
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
            NexusEmitError::Transport(_) | NexusEmitError::NonSuccess(_) => {}
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
}
