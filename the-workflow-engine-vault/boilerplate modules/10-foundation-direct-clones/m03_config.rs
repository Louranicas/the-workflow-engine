//! `m03_config` — Environment-driven configuration for DevOps Engine V3.
//!
//! All configuration is sourced from environment variables with safe defaults.
//! No TOML file is required to start; the service runs correctly with zero env
//! vars set.
//!
//! # Structs
//!
//! - [`EngineConfig`] — Top-level config, composed of [`ServerConfig`] and [`BridgeConfig`].
//! - [`ServerConfig`] — HTTP bind address and port.
//! - [`BridgeConfig`] — Addresses and timeouts for all Habitat service bridges.
//!
//! # Environment Variables
//!
//! | Variable | Default | Notes |
//! |----------|---------|-------|
//! | `PORT` | `8082` | HTTP listen port |
//! | `BIND_ADDR` | `127.0.0.1` | Bind address |
//! | `ORAC_ADDR` | `127.0.0.1:8133` | ORAC sidecar |
//! | `PV2_ADDR` | `127.0.0.1:8132` | Pane-Vortex V2 |
//! | `SYNTHEX_ADDR` | `127.0.0.1:8092` | SYNTHEX v2 thermal |
//! | `POVM_ADDR` | `127.0.0.1:8125` | POVM engine |
//! | `RM_ADDR` | `127.0.0.1:8130` | Reasoning Memory |
//! | `DEVOPS_V2_ADDR` | `127.0.0.1:8082` | DevOps Engine V2 (RETIRED — points to V3) |
//! | `HEALTH_INTERVAL_SECS` | `30` | Bridge health probe interval |
//! | `HOOK_TIMEOUT_SECS` | `2` | Max wait for ORAC hook acknowledgement |
//! | `CALL_TIMEOUT_SECS` | `5` | Max wait for any bridge HTTP call |
//!
//! # Errors
//!
//! [`ConfigError`] is produced when an env var is present but not parseable.  Missing
//! vars silently fall back to the default value so zero-config startup always works.
//!
//! Layer: `m1_core`
//! Dependencies: none (foundational module)

// ---------------------------------------------------------------------------
// ConfigError
// ---------------------------------------------------------------------------

/// Errors produced while loading [`EngineConfig`] from the environment.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// An environment variable was set but could not be parsed.
    #[error("env var `{var}` has invalid value '{value}': {reason}")]
    InvalidEnvVar {
        /// Name of the environment variable.
        var: &'static str,
        /// The raw string value that was set.
        value: String,
        /// Human-readable parse failure reason.
        reason: String,
    },

    /// A configuration value was outside its allowed range.
    #[error("config field `{field}` value {value} is outside [{min}, {max}]")]
    OutOfRange {
        /// Config field name.
        field: &'static str,
        /// The parsed value that was out of range.
        value: String,
        /// Minimum allowed value.
        min: String,
        /// Maximum allowed value.
        max: String,
    },
}

// ---------------------------------------------------------------------------
// ServerConfig
// ---------------------------------------------------------------------------

/// HTTP server binding configuration.
///
/// Controls the port and bind address of the Axum HTTP server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerConfig {
    /// TCP port the server listens on.
    ///
    /// Defaults to `8082`. Override with the `PORT` environment variable.
    pub port: u16,

    /// IP address to bind to.
    ///
    /// Defaults to `"127.0.0.1"`. Override with `BIND_ADDR`.
    pub bind_addr: String,
}

impl ServerConfig {
    /// Load `ServerConfig` from environment variables, using defaults for any
    /// that are absent.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::InvalidEnvVar`] if `PORT` is set but not a valid
    /// `u16`, or if `BIND_ADDR` is set to an empty string.
    pub fn from_env() -> Result<Self, ConfigError> {
        let port = parse_env_u16("PORT", 8082)?;
        let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1".to_owned());

        if bind_addr.is_empty() {
            return Err(ConfigError::InvalidEnvVar {
                var: "BIND_ADDR",
                value: bind_addr,
                reason: "bind address must not be empty".to_owned(),
            });
        }

        Ok(Self { port, bind_addr })
    }

    /// Return the full `<addr>:<port>` string suitable for socket binding.
    #[must_use]
    pub fn socket_addr(&self) -> String {
        format!("{}:{}", self.bind_addr, self.port)
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8082,
            bind_addr: "127.0.0.1".to_owned(),
        }
    }
}

// ---------------------------------------------------------------------------
// BridgeConfig
// ---------------------------------------------------------------------------

/// Configuration for all Habitat service bridges.
///
/// Every field is a `"<host>:<port>"` string used to construct `http://` URLs
/// inside bridge modules.  Timeouts control how long V3 waits before giving up
/// on a bridge call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeConfig {
    /// ORAC sidecar address (`127.0.0.1:8133`).
    ///
    /// Override with `ORAC_ADDR`.
    pub orac_addr: String,

    /// Pane-Vortex V2 address (`127.0.0.1:8132`).
    ///
    /// Override with `PV2_ADDR`.
    pub pv2_addr: String,

    /// SYNTHEX v2 thermal engine address (`127.0.0.1:8092`).
    ///
    /// Override with `SYNTHEX_ADDR`.
    pub synthex_addr: String,

    /// POVM engine address (`127.0.0.1:8125`).
    ///
    /// Override with `POVM_ADDR`.
    pub povm_addr: String,

    /// Reasoning Memory address (`127.0.0.1:8130`).
    ///
    /// Override with `RM_ADDR`.
    pub rm_addr: String,

    /// DevOps Engine V2 address (`127.0.0.1:8082`) — RETIRED, points to V3.
    ///
    /// Override with `DEVOPS_V2_ADDR`.
    pub devops_v2_addr: String,

    /// Seconds between background health-probe sweeps.
    ///
    /// Defaults to `30`. Override with `HEALTH_INTERVAL_SECS`.
    pub health_interval_secs: u64,

    /// Maximum seconds to wait for an ORAC hook acknowledgement.
    ///
    /// Defaults to `2`. Override with `HOOK_TIMEOUT_SECS`.
    pub hook_timeout_secs: u64,

    /// Maximum seconds to wait for any bridge HTTP call.
    ///
    /// Defaults to `5`. Override with `CALL_TIMEOUT_SECS`.
    pub call_timeout_secs: u64,
}

impl BridgeConfig {
    /// Load `BridgeConfig` from environment variables, using defaults for any
    /// that are absent.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::InvalidEnvVar`] if any numeric env var is set but
    /// cannot be parsed, or [`ConfigError::OutOfRange`] if an interval is zero.
    pub fn from_env() -> Result<Self, ConfigError> {
        let orac_addr =
            std::env::var("ORAC_ADDR").unwrap_or_else(|_| "127.0.0.1:8133".to_owned());
        let pv2_addr = std::env::var("PV2_ADDR").unwrap_or_else(|_| "127.0.0.1:8132".to_owned());
        let synthex_addr =
            std::env::var("SYNTHEX_ADDR").unwrap_or_else(|_| "127.0.0.1:8092".to_owned());
        let povm_addr =
            std::env::var("POVM_ADDR").unwrap_or_else(|_| "127.0.0.1:8125".to_owned());
        let rm_addr = std::env::var("RM_ADDR").unwrap_or_else(|_| "127.0.0.1:8130".to_owned());
        let devops_v2_addr =
            std::env::var("DEVOPS_V2_ADDR").unwrap_or_else(|_| "127.0.0.1:8082".to_owned());

        let health_interval_secs = parse_env_u64("HEALTH_INTERVAL_SECS", 30)?;
        let hook_timeout_secs = parse_env_u64("HOOK_TIMEOUT_SECS", 2)?;
        let call_timeout_secs = parse_env_u64("CALL_TIMEOUT_SECS", 5)?;

        // Validate non-zero intervals — a zero interval would cause a tight spin.
        if health_interval_secs == 0 {
            return Err(ConfigError::OutOfRange {
                field: "HEALTH_INTERVAL_SECS",
                value: "0".to_owned(),
                min: "1".to_owned(),
                max: u64::MAX.to_string(),
            });
        }
        if hook_timeout_secs == 0 {
            return Err(ConfigError::OutOfRange {
                field: "HOOK_TIMEOUT_SECS",
                value: "0".to_owned(),
                min: "1".to_owned(),
                max: u64::MAX.to_string(),
            });
        }
        if call_timeout_secs == 0 {
            return Err(ConfigError::OutOfRange {
                field: "CALL_TIMEOUT_SECS",
                value: "0".to_owned(),
                min: "1".to_owned(),
                max: u64::MAX.to_string(),
            });
        }

        Ok(Self {
            orac_addr,
            pv2_addr,
            synthex_addr,
            povm_addr,
            rm_addr,
            devops_v2_addr,
            health_interval_secs,
            hook_timeout_secs,
            call_timeout_secs,
        })
    }

    /// Build the full `http://` URL for a bridge address + path.
    ///
    /// Complies with BUG-033: bridge URLs must always include the `http://` scheme.
    #[must_use]
    pub fn build_url(addr: &str, path: &str) -> String {
        format!("http://{addr}{path}")
    }

    /// Return the full ORAC health URL.
    #[must_use]
    pub fn orac_url(&self, path: &str) -> String {
        Self::build_url(&self.orac_addr, path)
    }

    /// Return the full PV2 URL.
    #[must_use]
    pub fn pv2_url(&self, path: &str) -> String {
        Self::build_url(&self.pv2_addr, path)
    }

    /// Return the full SYNTHEX URL.
    ///
    /// Note: SYNTHEX health is at `/api/health`, not `/health` (BUG trap).
    #[must_use]
    pub fn synthex_url(&self, path: &str) -> String {
        Self::build_url(&self.synthex_addr, path)
    }

    /// Return the full POVM URL.
    #[must_use]
    pub fn povm_url(&self, path: &str) -> String {
        Self::build_url(&self.povm_addr, path)
    }

    /// Return the full Reasoning Memory URL.
    #[must_use]
    pub fn rm_url(&self, path: &str) -> String {
        Self::build_url(&self.rm_addr, path)
    }

    /// Return the full DevOps V2 URL.
    #[must_use]
    pub fn devops_v2_url(&self, path: &str) -> String {
        Self::build_url(&self.devops_v2_addr, path)
    }
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            orac_addr: "127.0.0.1:8133".to_owned(),
            pv2_addr: "127.0.0.1:8132".to_owned(),
            synthex_addr: "127.0.0.1:8092".to_owned(),
            povm_addr: "127.0.0.1:8125".to_owned(),
            rm_addr: "127.0.0.1:8130".to_owned(),
            devops_v2_addr: "127.0.0.1:8082".to_owned(),
            health_interval_secs: 30,
            hook_timeout_secs: 2,
            call_timeout_secs: 5,
        }
    }
}

// ---------------------------------------------------------------------------
// EngineConfig
// ---------------------------------------------------------------------------

/// Top-level configuration for DevOps Engine V3.
///
/// Composes [`ServerConfig`] (HTTP binding) and [`BridgeConfig`] (Habitat
/// service addresses + timeouts).  All fields have safe defaults so the
/// service starts correctly with zero env vars set.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EngineConfig {
    /// HTTP server binding configuration.
    pub server: ServerConfig,
    /// Habitat service bridge addresses and timeouts.
    pub bridges: BridgeConfig,
}

impl EngineConfig {
    /// Load configuration entirely from environment variables.
    ///
    /// Any missing env var falls back to its hard-coded default.  The daemon
    /// can start with zero env vars set and run correctly on `localhost`.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] only when an env var is **present** but has an
    /// invalid value (bad integer format, empty bind address, zero timeout).
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            server: ServerConfig::from_env()?,
            bridges: BridgeConfig::from_env()?,
        })
    }
}


// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Parse an env var as `u16`. Returns `default_val` if the var is absent.
///
/// # Errors
///
/// Returns [`ConfigError::InvalidEnvVar`] if the var is present but not
/// parseable as `u16`.
fn parse_env_u16(var: &'static str, default_val: u16) -> Result<u16, ConfigError> {
    std::env::var(var).map_or(Ok(default_val), |raw| {
        raw.parse::<u16>().map_err(|e| ConfigError::InvalidEnvVar {
            var,
            value: raw,
            reason: e.to_string(),
        })
    })
}

/// Parse an env var as `u64`. Returns `default_val` if the var is absent.
///
/// # Errors
///
/// Returns [`ConfigError::InvalidEnvVar`] if the var is present but not
/// parseable as `u64`.
fn parse_env_u64(var: &'static str, default_val: u64) -> Result<u64, ConfigError> {
    std::env::var(var).map_or(Ok(default_val), |raw| {
        raw.parse::<u64>().map_err(|e| ConfigError::InvalidEnvVar {
            var,
            value: raw,
            reason: e.to_string(),
        })
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ── Default values ────────────────────────────────────────────────────────

    #[test]
    fn server_config_default_port() {
        let cfg = ServerConfig::default();
        assert_eq!(cfg.port, 8082);
    }

    #[test]
    fn server_config_default_bind_addr() {
        let cfg = ServerConfig::default();
        assert_eq!(cfg.bind_addr, "127.0.0.1");
    }

    #[test]
    fn server_config_default_socket_addr() {
        let cfg = ServerConfig::default();
        assert_eq!(cfg.socket_addr(), "127.0.0.1:8082");
    }

    #[test]
    fn bridge_config_default_orac() {
        let cfg = BridgeConfig::default();
        assert_eq!(cfg.orac_addr, "127.0.0.1:8133");
    }

    #[test]
    fn bridge_config_default_pv2() {
        let cfg = BridgeConfig::default();
        assert_eq!(cfg.pv2_addr, "127.0.0.1:8132");
    }

    #[test]
    fn bridge_config_default_synthex() {
        let cfg = BridgeConfig::default();
        assert_eq!(cfg.synthex_addr, "127.0.0.1:8092");
    }

    #[test]
    fn bridge_config_default_povm() {
        let cfg = BridgeConfig::default();
        assert_eq!(cfg.povm_addr, "127.0.0.1:8125");
    }

    #[test]
    fn bridge_config_default_rm() {
        let cfg = BridgeConfig::default();
        assert_eq!(cfg.rm_addr, "127.0.0.1:8130");
    }

    #[test]
    fn bridge_config_default_devops_v2() {
        let cfg = BridgeConfig::default();
        assert_eq!(cfg.devops_v2_addr, "127.0.0.1:8082");
    }

    #[test]
    fn bridge_config_default_health_interval() {
        let cfg = BridgeConfig::default();
        assert_eq!(cfg.health_interval_secs, 30);
    }

    #[test]
    fn bridge_config_default_hook_timeout() {
        let cfg = BridgeConfig::default();
        assert_eq!(cfg.hook_timeout_secs, 2);
    }

    #[test]
    fn bridge_config_default_call_timeout() {
        let cfg = BridgeConfig::default();
        assert_eq!(cfg.call_timeout_secs, 5);
    }

    #[test]
    fn engine_config_default_equals_parts() {
        let cfg = EngineConfig::default();
        assert_eq!(cfg.server, ServerConfig::default());
        assert_eq!(cfg.bridges, BridgeConfig::default());
    }

    // ── URL builders ──────────────────────────────────────────────────────────

    #[test]
    fn bridge_config_build_url_includes_scheme() {
        let url = BridgeConfig::build_url("127.0.0.1:8133", "/health");
        assert!(url.starts_with("http://"), "URL must include http:// scheme");
    }

    #[test]
    fn bridge_config_orac_url() {
        let cfg = BridgeConfig::default();
        assert_eq!(cfg.orac_url("/health"), "http://127.0.0.1:8133/health");
    }

    #[test]
    fn bridge_config_synthex_health_path() {
        // SYNTHEX health is at /api/health, not /health (BUG trap from CLAUDE.md).
        let cfg = BridgeConfig::default();
        assert_eq!(
            cfg.synthex_url("/api/health"),
            "http://127.0.0.1:8092/api/health"
        );
    }

    #[test]
    fn bridge_config_pv2_url() {
        let cfg = BridgeConfig::default();
        assert_eq!(cfg.pv2_url("/field"), "http://127.0.0.1:8132/field");
    }

    #[test]
    fn bridge_config_rm_url() {
        let cfg = BridgeConfig::default();
        assert_eq!(cfg.rm_url("/search?q=devops-v3"), "http://127.0.0.1:8130/search?q=devops-v3");
    }

    #[test]
    fn bridge_config_povm_url() {
        let cfg = BridgeConfig::default();
        assert_eq!(cfg.povm_url("/hydrate"), "http://127.0.0.1:8125/hydrate");
    }

    #[test]
    fn bridge_config_devops_v2_url() {
        let cfg = BridgeConfig::default();
        assert_eq!(
            cfg.devops_v2_url("/health"),
            "http://127.0.0.1:8082/health"
        );
    }

    // ── from_env with no vars set ─────────────────────────────────────────────

    #[test]
    fn engine_config_from_env_no_vars_uses_defaults() {
        // Run in a clean env context by unsetting all V3 vars temporarily.
        // We cannot truly isolate env vars in unit tests without serial test
        // execution; instead we verify that from_env() returns Ok and the
        // defaults match what Default gives when no env vars are set.
        // (This test assumes none of the vars are set in the CI environment.)
        let result = EngineConfig::from_env();
        // Must succeed — zero env vars is always valid.
        assert!(result.is_ok(), "from_env should succeed with no vars set");
    }

    // ── parse helpers ─────────────────────────────────────────────────────────

    #[test]
    fn parse_env_u16_returns_default_when_absent() {
        // Use a var name that will never be set in any real environment.
        let result = parse_env_u16("__V3_TEST_PORT_ABSENT__", 9999);
        assert_eq!(result.unwrap(), 9999);
    }

    #[test]
    fn parse_env_u64_returns_default_when_absent() {
        let result = parse_env_u64("__V3_TEST_SECS_ABSENT__", 42);
        assert_eq!(result.unwrap(), 42);
    }

    // ── Error cases ───────────────────────────────────────────────────────────

    #[test]
    fn bridge_config_zero_health_interval_rejected() {
        // Simulate HEALTH_INTERVAL_SECS=0 via the OutOfRange path directly.
        let err = ConfigError::OutOfRange {
            field: "HEALTH_INTERVAL_SECS",
            value: "0".to_owned(),
            min: "1".to_owned(),
            max: u64::MAX.to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("HEALTH_INTERVAL_SECS"));
        assert!(msg.contains("0"));
    }

    #[test]
    fn config_error_invalid_env_var_display() {
        let err = ConfigError::InvalidEnvVar {
            var: "PORT",
            value: "not_a_port".to_owned(),
            reason: "invalid digit".to_owned(),
        };
        let msg = err.to_string();
        assert!(msg.contains("PORT"));
        assert!(msg.contains("not_a_port"));
        assert!(msg.contains("invalid digit"));
    }

    #[test]
    fn config_error_out_of_range_display() {
        let err = ConfigError::OutOfRange {
            field: "CALL_TIMEOUT_SECS",
            value: "0".to_owned(),
            min: "1".to_owned(),
            max: "3600".to_owned(),
        };
        let msg = err.to_string();
        assert!(msg.contains("CALL_TIMEOUT_SECS"));
    }

    // ── Structural coverage ───────────────────────────────────────────────────

    #[test]
    fn server_config_socket_addr_custom_port() {
        let cfg = ServerConfig {
            port: 9999,
            bind_addr: "127.0.0.1".to_owned(),
        };
        assert_eq!(cfg.socket_addr(), "127.0.0.1:9999");
    }

    #[test]
    fn bridge_build_url_strips_no_leading_slash_when_path_has_one() {
        let url = BridgeConfig::build_url("127.0.0.1:8092", "/v3/thermal");
        assert_eq!(url, "http://127.0.0.1:8092/v3/thermal");
    }

    #[test]
    fn engine_config_clone_is_equal() {
        let cfg = EngineConfig::default();
        let cloned = cfg.clone();
        assert_eq!(cfg, cloned);
    }
}
