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
}
