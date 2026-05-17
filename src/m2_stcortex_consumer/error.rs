//! Error types for the m2 stcortex narrowed consumer.
//!
//! Per m2 spec § 4. Structured fields preserve diagnostic context so
//! callers can match programmatically rather than parsing display strings.

use thiserror::Error;

/// Failure modes for the m2 stcortex consumer surface.
#[derive(Debug, Error)]
pub enum StcortexConsumerError {
    /// Could not reach stcortex over WebSocket at `uri`.
    #[error("connection to stcortex at {uri} failed: {reason}")]
    ConnectionFailed {
        /// Endpoint attempted (`ws://127.0.0.1:3000` default).
        uri: String,
        /// Underlying transport reason.
        reason: String,
    },

    /// `register_consumer` reducer returned an error.
    #[error("register_consumer reducer failed: {0}")]
    RegisterFailed(String),

    /// `on_applied` did not fire within `timeout_ms`.
    #[error("subscription apply timed out after {timeout_ms}ms")]
    SubscriptionTimeout {
        /// Configured timeout.
        timeout_ms: u64,
    },

    /// Explicit unregister call against stcortex failed.
    #[error("unregister_consumer reducer failed: {0}")]
    UnregisterFailed(String),

    /// Namespace did not satisfy the AP30 prefix invariant.
    #[error("namespace validation failed: {0}")]
    InvalidNamespace(String),

    /// Consumer name failed the alphanumeric + `_-` / length-64 validator.
    #[error("consumer-name validation failed: {0}")]
    InvalidConsumerName(String),
}

#[cfg(test)]
mod tests {
    use super::StcortexConsumerError;

    #[test]
    fn connection_failed_display_names_uri_and_reason() {
        let err = StcortexConsumerError::ConnectionFailed {
            uri: "ws://127.0.0.1:3000".into(),
            reason: "refused".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("ws://127.0.0.1:3000"));
        assert!(msg.contains("refused"));
    }

    #[test]
    fn register_failed_display_carries_reason() {
        assert!(StcortexConsumerError::RegisterFailed("nope".into())
            .to_string()
            .contains("nope"));
    }

    #[test]
    fn subscription_timeout_display_names_timeout() {
        assert!(StcortexConsumerError::SubscriptionTimeout { timeout_ms: 5000 }
            .to_string()
            .contains("5000"));
    }

    #[test]
    fn unregister_failed_display_carries_reason() {
        assert!(
            StcortexConsumerError::UnregisterFailed("conflict".into())
                .to_string()
                .contains("conflict")
        );
    }

    #[test]
    fn invalid_namespace_preserves_bad_value() {
        assert!(StcortexConsumerError::InvalidNamespace("orac_foo".into())
            .to_string()
            .contains("orac_foo"));
    }

    #[test]
    fn invalid_consumer_name_preserves_bad_value() {
        assert!(StcortexConsumerError::InvalidConsumerName("bad name!".into())
            .to_string()
            .contains("bad name!"));
    }

    #[test]
    fn implements_std_error_and_send_sync_static() {
        fn assert_error<T: std::error::Error>() {}
        fn assert_send_sync_static<T: Send + Sync + 'static>() {}
        assert_error::<StcortexConsumerError>();
        assert_send_sync_static::<StcortexConsumerError>();
    }
}
