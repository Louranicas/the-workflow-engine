//! Runtime mirror probe for the POVM `learning_health` band.
//!
//! Per m8 spec § 5 (Implementation sketch — runtime mirror), this module
//! performs a blocking `reqwest` probe with a 2-second timeout, parses
//! `{ "learning_health": f64 }`, and classifies the value against the
//! Phase-1 band via [`super::cfg::classify`]. The build-time gate in
//! `build.rs` and this runtime mirror share the band constants in
//! [`super::cfg`], so they cannot drift.

use std::time::Duration;

use serde::Deserialize;

use super::cfg::{classify, BandClassification};
use super::error::BuildPrereqError;

/// Default POVM `/learning_health` endpoint. Override at runtime by setting
/// the `POVM_HEALTH_URL` environment variable; the default targets the
/// workspace-local POVM instance per CLAUDE.md ULTRAPLATE services row.
pub const DEFAULT_HEALTH_URL: &str = "http://127.0.0.1:8125/learning_health";

/// Default probe timeout — chosen per m8 spec § 5 ("2s timeout"). Shorter
/// than the default reqwest timeout so a stalled POVM cannot wedge startup.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(2);

/// JSON shape returned by `GET /learning_health` on POVM `:8125`.
///
/// Captured from the live POVM endpoint as of 2026-05-17 post-CR-2; if the
/// shape drifts, the F-Contract test in `tests/m8_integration.rs` will fail
/// against a recorded snapshot before runtime ever sees it.
#[derive(Debug, Clone, Copy, Deserialize)]
struct LearningHealthResponse {
    learning_health: f64,
}

/// Resolve the configured POVM health-probe URL, honouring the
/// `POVM_HEALTH_URL` environment variable.
#[must_use]
pub fn resolve_health_url() -> String {
    std::env::var("POVM_HEALTH_URL").unwrap_or_else(|_| DEFAULT_HEALTH_URL.to_string())
}

/// Blocking HTTP client wrapping a `reqwest::blocking::Client` plus the
/// configured POVM URL.
///
/// Reuse a single instance for repeated probes — `reqwest::blocking::Client`
/// is `Send + Sync` and amortises connection-pool setup.
pub struct HealthClient {
    client: reqwest::blocking::Client,
    url: String,
}

impl std::fmt::Debug for HealthClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HealthClient")
            .field("url", &self.url)
            .field("client", &"reqwest::blocking::Client { .. }")
            .finish()
    }
}

impl HealthClient {
    /// Construct a client against [`resolve_health_url`] with
    /// [`DEFAULT_TIMEOUT`].
    ///
    /// # Errors
    ///
    /// Returns [`BuildPrereqError::ProbeFailed`] if the underlying
    /// `reqwest::Client::builder` fails — typically only on platforms where
    /// rustls cannot initialise.
    pub fn new() -> Result<Self, BuildPrereqError> {
        Self::with_url_and_timeout(resolve_health_url(), DEFAULT_TIMEOUT)
    }

    /// Construct a client against a caller-supplied URL with
    /// [`DEFAULT_TIMEOUT`].
    ///
    /// # Errors
    ///
    /// See [`Self::new`].
    pub fn with_url(url: impl Into<String>) -> Result<Self, BuildPrereqError> {
        Self::with_url_and_timeout(url, DEFAULT_TIMEOUT)
    }

    /// Construct a client against a caller-supplied URL with a caller-supplied
    /// timeout. Useful in tests that want a sub-second timeout to fail fast.
    ///
    /// # Errors
    ///
    /// See [`Self::new`].
    pub fn with_url_and_timeout(
        url: impl Into<String>,
        timeout: Duration,
    ) -> Result<Self, BuildPrereqError> {
        let url = url.into();
        let client = reqwest::blocking::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|source| BuildPrereqError::ProbeFailed {
                url: url.clone(),
                source,
            })?;
        Ok(Self { client, url })
    }

    /// Configured URL (read-only).
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Probe the POVM endpoint and return the raw `learning_health` value.
    ///
    /// # Errors
    ///
    /// - [`BuildPrereqError::ProbeFailed`] on transport failure (DNS,
    ///   connection refused, TLS, timeout).
    /// - [`BuildPrereqError::ProbeFailed`] on non-2xx HTTP status (wrapped
    ///   via `reqwest::Error`).
    /// - [`BuildPrereqError::ProbeFailed`] on JSON parse failure.
    pub fn probe_value(&self) -> Result<f64, BuildPrereqError> {
        let response = self
            .client
            .get(&self.url)
            .send()
            .and_then(reqwest::blocking::Response::error_for_status)
            .map_err(|source| BuildPrereqError::ProbeFailed {
                url: self.url.clone(),
                source,
            })?;
        let body: LearningHealthResponse =
            response.json().map_err(|source| BuildPrereqError::ProbeFailed {
                url: self.url.clone(),
                source,
            })?;
        Ok(body.learning_health)
    }

    /// Probe the POVM endpoint and return both the raw value and the band
    /// classification.
    ///
    /// Emits a `tracing::info!` event at target `m8.health.probe` carrying
    /// `url`, `learning_health`, and `band` per m8 spec § 9 Observability.
    ///
    /// # Errors
    ///
    /// See [`Self::probe_value`].
    pub fn probe_band(&self) -> Result<(f64, BandClassification), BuildPrereqError> {
        let value = self.probe_value()?;
        let band = classify(value);
        tracing::info!(
            target: "m8.health.probe",
            url = %self.url,
            learning_health = value,
            band = band.banner(),
            "POVM band probe complete"
        );
        Ok((value, band))
    }
}

/// Free-function convenience: probe `url` with [`DEFAULT_TIMEOUT`] and return
/// the value + classification.
///
/// Equivalent to constructing a [`HealthClient`] for a one-shot probe.
///
/// # Errors
///
/// See [`HealthClient::probe_value`].
pub fn probe_band(url: &str) -> Result<(f64, BandClassification), BuildPrereqError> {
    HealthClient::with_url(url.to_string())?.probe_band()
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread::{self, JoinHandle};
    use std::time::Duration;

    use super::{
        probe_band, resolve_health_url, HealthClient, DEFAULT_HEALTH_URL, DEFAULT_TIMEOUT,
    };
    use crate::m8_povm_build_prereq::cfg::BandClassification;

    /// Spawn a one-shot TCP mock server that returns a fixed HTTP response
    /// to a single connection, then exits. Returns the URL plus the join
    /// handle (caller awaits the handle to ensure clean shutdown).
    fn spawn_one_shot_server(status_line: &str, body: &str) -> (String, JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
        let port = listener.local_addr().expect("local_addr").port();
        let body = body.to_string();
        let status_line = status_line.to_string();
        let handle = thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                // Read the request (we ignore it; the mock is one-shot).
                let mut buf = [0u8; 1024];
                let _ = stream.read(&mut buf);
                let response = format!(
                    "HTTP/1.1 {status_line}\r\nContent-Length: {len}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{body}",
                    len = body.len()
                );
                let _ = stream.write_all(response.as_bytes());
            }
        });
        let url = format!("http://127.0.0.1:{port}/learning_health");
        (url, handle)
    }

    // ---- F-Unit (construction + URL handling) --------------------------

    #[test]
    fn default_url_is_workspace_povm() {
        assert_eq!(DEFAULT_HEALTH_URL, "http://127.0.0.1:8125/learning_health");
    }

    #[test]
    fn default_timeout_is_two_seconds() {
        assert_eq!(DEFAULT_TIMEOUT, Duration::from_secs(2));
    }

    #[test]
    fn resolve_health_url_uses_default_when_env_unset() {
        // Snapshot + clear + restore the env var to avoid cross-test
        // interference. `set_var` is safe within a process boundary; the
        // serial-test discipline lives in tests/m8_integration.rs where
        // multi-test env mutation could collide.
        let prior = std::env::var("POVM_HEALTH_URL").ok();
        std::env::remove_var("POVM_HEALTH_URL");
        let resolved = resolve_health_url();
        if let Some(p) = prior {
            std::env::set_var("POVM_HEALTH_URL", p);
        }
        assert_eq!(resolved, DEFAULT_HEALTH_URL);
    }

    #[test]
    fn resolve_health_url_honours_env_override() {
        let prior = std::env::var("POVM_HEALTH_URL").ok();
        std::env::set_var("POVM_HEALTH_URL", "http://example.invalid:9999/lh");
        let resolved = resolve_health_url();
        match prior {
            Some(p) => std::env::set_var("POVM_HEALTH_URL", p),
            None => std::env::remove_var("POVM_HEALTH_URL"),
        }
        assert_eq!(resolved, "http://example.invalid:9999/lh");
    }

    #[test]
    fn client_new_succeeds() {
        let client = HealthClient::new().expect("default client build");
        assert!(!client.url().is_empty());
    }

    #[test]
    fn client_with_url_stores_url() {
        let client = HealthClient::with_url("http://x.invalid/lh").expect("client build");
        assert_eq!(client.url(), "http://x.invalid/lh");
    }

    #[test]
    fn client_with_url_and_timeout_stores_url() {
        let client =
            HealthClient::with_url_and_timeout("http://x.invalid/lh", Duration::from_millis(100))
                .expect("client build");
        assert_eq!(client.url(), "http://x.invalid/lh");
    }

    #[test]
    fn client_debug_format_contains_url() {
        let client = HealthClient::with_url("http://debug.invalid/lh").expect("client build");
        let debug = format!("{client:?}");
        assert!(debug.contains("http://debug.invalid/lh"));
    }

    // ---- F-Integration (mock POVM server) ------------------------------

    #[test]
    fn probe_value_parses_in_band_response() {
        let (url, handle) = spawn_one_shot_server("200 OK", "{\"learning_health\":0.067}");
        let client = HealthClient::with_url_and_timeout(&url, Duration::from_secs(2))
            .expect("client build");
        let value = client.probe_value().expect("probe");
        let _ = handle.join();
        assert!((value - 0.067).abs() < f64::EPSILON);
    }

    #[test]
    fn probe_band_classifies_in_band_value() {
        let (url, handle) = spawn_one_shot_server("200 OK", "{\"learning_health\":0.10}");
        let client = HealthClient::with_url_and_timeout(&url, Duration::from_secs(2))
            .expect("client build");
        let (value, band) = client.probe_band().expect("probe_band");
        let _ = handle.join();
        assert!((value - 0.10).abs() < f64::EPSILON);
        assert_eq!(band, BandClassification::InBand);
    }

    #[test]
    fn probe_band_classifies_above_ceiling() {
        // Canonical pre-CR-2 inflated value.
        let (url, handle) = spawn_one_shot_server("200 OK", "{\"learning_health\":0.9114}");
        let client = HealthClient::with_url_and_timeout(&url, Duration::from_secs(2))
            .expect("client build");
        let (_, band) = client.probe_band().expect("probe_band");
        let _ = handle.join();
        assert_eq!(band, BandClassification::AboveCeiling);
    }

    #[test]
    fn probe_band_classifies_below_floor() {
        let (url, handle) = spawn_one_shot_server("200 OK", "{\"learning_health\":0.02}");
        let client = HealthClient::with_url_and_timeout(&url, Duration::from_secs(2))
            .expect("client build");
        let (_, band) = client.probe_band().expect("probe_band");
        let _ = handle.join();
        assert_eq!(band, BandClassification::BelowFloor);
    }

    #[test]
    fn probe_band_handles_nan_value() {
        // POVM returning a literal `null` would be a deserialise error, not
        // a Nan band — for a Nan band the value would have to literally be
        // NaN. Serde-json doesn't deserialise "NaN" by default, so this is
        // synthetic: we drive through cfg::classify directly. Here we just
        // verify the deserialiser refuses an invalid shape.
        let (url, handle) = spawn_one_shot_server("200 OK", "{\"learning_health\":\"oops\"}");
        let client = HealthClient::with_url_and_timeout(&url, Duration::from_secs(2))
            .expect("client build");
        let err = client.probe_band().expect_err("expected JSON parse failure");
        let _ = handle.join();
        let msg = err.to_string();
        assert!(msg.contains("unreachable") || msg.contains("decode") || msg.contains("error"));
    }

    #[test]
    fn probe_value_fails_on_404() {
        let (url, handle) = spawn_one_shot_server("404 Not Found", "not found");
        let client = HealthClient::with_url_and_timeout(&url, Duration::from_secs(2))
            .expect("client build");
        let err = client.probe_value().expect_err("expected 404 to fail");
        let _ = handle.join();
        let msg = err.to_string();
        assert!(msg.contains("unreachable"));
    }

    #[test]
    fn probe_value_fails_on_500() {
        let (url, handle) = spawn_one_shot_server("500 Internal Server Error", "boom");
        let client = HealthClient::with_url_and_timeout(&url, Duration::from_secs(2))
            .expect("client build");
        let err = client.probe_value().expect_err("expected 500 to fail");
        let _ = handle.join();
        let msg = err.to_string();
        assert!(msg.contains("unreachable"));
    }

    #[test]
    fn probe_band_free_function_works() {
        let (url, handle) = spawn_one_shot_server("200 OK", "{\"learning_health\":0.10}");
        let (value, band) = probe_band(&url).expect("free function probe");
        let _ = handle.join();
        assert!((value - 0.10).abs() < f64::EPSILON);
        assert_eq!(band, BandClassification::InBand);
    }

    #[test]
    fn probe_band_fails_on_unreachable_endpoint() {
        // Port 1 is reserved + nobody is listening (except possibly tcpmux on
        // ancient systems). Use a very short timeout so this test runs fast.
        let client = HealthClient::with_url_and_timeout(
            "http://127.0.0.1:1/learning_health",
            Duration::from_millis(200),
        )
        .expect("client build");
        let err = client.probe_value().expect_err("expected unreachable");
        let msg = err.to_string();
        assert!(msg.contains("unreachable"));
    }

    #[test]
    fn probe_band_classifies_band_edges_in_band() {
        // F-Regression: band edges 0.05 and 0.15 must classify InBand.
        for (body, expected) in [
            ("{\"learning_health\":0.05}", BandClassification::InBand),
            ("{\"learning_health\":0.15}", BandClassification::InBand),
        ] {
            let (url, handle) = spawn_one_shot_server("200 OK", body);
            let (_, band) =
                probe_band(&url).expect("probe should succeed at band edges");
            let _ = handle.join();
            assert_eq!(band, expected, "body={body}");
        }
    }
}
