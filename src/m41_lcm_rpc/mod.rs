//! `m41_lcm_rpc` — `lcm.loop.create` JSON-RPC client.
//! Cluster H · L8.
//!
//! NOTE: `lcm.loop.create` is the canonical LCM RPC name (per S1001882
//! drift retraction; the legacy `lcm.deploy` form is deprecated).

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
}

impl HttpLcmClient {
    /// Construct.
    #[must_use]
    pub fn new(url: impl Into<String>, timeout: Duration) -> Self {
        Self {
            url: url.into(),
            timeout,
        }
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
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": RPC_METHOD,
            "params": params,
            "id": 1,
        });
        let resp = client
            .post(&self.url)
            .json(&body)
            .send()
            .map_err(|e| LcmRpcError::Transport(e.to_string()))?;
        let value: serde_json::Value = resp
            .json()
            .map_err(|e| LcmRpcError::Parse(e.to_string()))?;
        if let Some(err) = value.get("error") {
            let code = err.get("code").and_then(serde_json::Value::as_i64).unwrap_or(-32000);
            let message = err
                .get("message")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown")
                .to_owned();
            return Err(LcmRpcError::Rpc { code, message });
        }
        let result = value
            .get("result")
            .ok_or_else(|| LcmRpcError::Parse("missing result".into()))?;
        serde_json::from_value(result.clone()).map_err(|e| LcmRpcError::Parse(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::{
        LcmClient, LcmLoopCreateParams, LcmLoopCreateResult, LcmRpcError, DEFAULT_LCM_URL,
        DEFAULT_RPC_TIMEOUT, RPC_METHOD,
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
}
