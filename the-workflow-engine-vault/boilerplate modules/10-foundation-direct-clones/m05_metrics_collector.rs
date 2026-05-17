//! `m05_metrics_collector` — Prometheus-compatible metrics façade.
//!
//! Modules across every layer emit metrics through the `metrics` crate macros
//! (`metrics::gauge!`, `counter!`, `histogram!`). This module wires the
//! Prometheus exporter at process start and supplies typed helper functions
//! so the hot paths don't reach into string-interpolation land.
//!
//! # Typed surfaces
//!
//! - [`TensorDim`] — the 11 canonical state-tensor dimensions (`t`, `r`,
//!   `fitness`, `ltp_rate`, `ltd_rate`, `povm_growth`, `thermal_delta`, three
//!   phase one-hots, `flow_state`). m07 owns *ownership* of these values; m05
//!   provides the emission façade.
//! - [`BridgeCounter`] — circuit state transitions (`open` / `half_open` /
//!   `closed`) and call outcomes.
//! - [`HmxLatencyBucket`] — fixed latency bucket identifiers used by the HMX
//!   P50/P95/P99 histograms in L7.
//!
//! # Labels
//!
//! Every metric gets a stable label set.  When a label value comes from a
//! newtype in m01 (`HeatSourceId`, `ServiceId`, `ModuleId`) the helpers use
//! the type's `Display` so the encoding is locked in one place.

#![allow(clippy::module_name_repetitions)]

use std::net::SocketAddr;
use std::sync::Once;

use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::m01_core_types::{HeatSourceId, ModuleId, ServiceId};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors from metrics bootstrapping.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum MetricsError {
    /// Another call to [`MetricsCollector::init`] has already succeeded.
    #[error("metrics: already initialised")]
    AlreadyInitialised,
    /// Prometheus listener bind / install failed.
    #[error("metrics: failed to install Prometheus recorder: {0}")]
    InstallFailed(String),
    /// Listen address could not be parsed.
    #[error("metrics: invalid listen address {input:?}: {message}")]
    InvalidAddress {
        /// Raw input that failed to parse.
        input: String,
        /// Parser message.
        message: String,
    },
}

// ---------------------------------------------------------------------------
// TensorDim — 11 canonical dims
// ---------------------------------------------------------------------------

/// Canonical 11D state tensor dimensions.
///
/// Keep this in lockstep with `.claude/context.json tensor.names`. `m07_tensor_registry`
/// owns the authoritative state; m05 owns the metric emission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TensorDim {
    /// Dim 0: temperature `T`.
    TemperatureT,
    /// Dim 1: Kuramoto order parameter `r`.
    Pv2R,
    /// Dim 2: RALPH fitness.
    RalphFitness,
    /// Dim 3: LTP (long-term potentiation) rate.
    LtpRate,
    /// Dim 4: LTD (long-term depression) rate.
    LtdRate,
    /// Dim 5: POVM pathway growth.
    PovmGrowth,
    /// Dim 6: thermal delta (derivative of `T`).
    ThermalDelta,
    /// Dim 7: phase one-hot — `Recognize`.
    PhaseOnehotRecognize,
    /// Dim 8: phase one-hot — `Explore`.
    PhaseOnehotExplore,
    /// Dim 9: phase one-hot — `Consolidate`.
    PhaseOnehotConsolidate,
    /// Dim 10: flow state fraction.
    FlowState,
}

impl TensorDim {
    /// All dims in canonical index order (0 .. 10).
    pub const ALL: [Self; 11] = [
        Self::TemperatureT,
        Self::Pv2R,
        Self::RalphFitness,
        Self::LtpRate,
        Self::LtdRate,
        Self::PovmGrowth,
        Self::ThermalDelta,
        Self::PhaseOnehotRecognize,
        Self::PhaseOnehotExplore,
        Self::PhaseOnehotConsolidate,
        Self::FlowState,
    ];

    /// `snake_case` label used for the Prometheus metric suffix.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::TemperatureT => "temperature_t",
            Self::Pv2R => "pv2_r",
            Self::RalphFitness => "ralph_fitness",
            Self::LtpRate => "ltp_rate",
            Self::LtdRate => "ltd_rate",
            Self::PovmGrowth => "povm_growth",
            Self::ThermalDelta => "thermal_delta",
            Self::PhaseOnehotRecognize => "phase_onehot_recognize",
            Self::PhaseOnehotExplore => "phase_onehot_explore",
            Self::PhaseOnehotConsolidate => "phase_onehot_consolidate",
            Self::FlowState => "flow_state",
        }
    }

    /// Canonical 0-based index (matches `.claude/context.json tensor.names`).
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::TemperatureT => 0,
            Self::Pv2R => 1,
            Self::RalphFitness => 2,
            Self::LtpRate => 3,
            Self::LtdRate => 4,
            Self::PovmGrowth => 5,
            Self::ThermalDelta => 6,
            Self::PhaseOnehotRecognize => 7,
            Self::PhaseOnehotExplore => 8,
            Self::PhaseOnehotConsolidate => 9,
            Self::FlowState => 10,
        }
    }
}

impl std::fmt::Display for TensorDim {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

// ---------------------------------------------------------------------------
// BridgeCounter — circuit state transitions
// ---------------------------------------------------------------------------

/// Enumerates the counter variants emitted for bridge circuit-breaker state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeCounter {
    /// Breaker transitioned `closed → open` after threshold failures.
    CircuitOpened,
    /// Breaker transitioned `open → half_open` after cooldown.
    CircuitHalfOpened,
    /// Breaker transitioned `half_open → closed` after a successful probe.
    CircuitClosed,
    /// Call succeeded (200 or expected status).
    CallOk,
    /// Call failed (timeout, 5xx, parse error, CLI non-zero exit).
    CallErr,
    /// Call skipped because breaker was open.
    CallSkipped,
}

impl BridgeCounter {
    /// All variants.
    pub const ALL: [Self; 6] = [
        Self::CircuitOpened,
        Self::CircuitHalfOpened,
        Self::CircuitClosed,
        Self::CallOk,
        Self::CallErr,
        Self::CallSkipped,
    ];

    /// `snake_case` label used for the Prometheus counter suffix.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::CircuitOpened => "circuit_opened",
            Self::CircuitHalfOpened => "circuit_half_opened",
            Self::CircuitClosed => "circuit_closed",
            Self::CallOk => "call_ok",
            Self::CallErr => "call_err",
            Self::CallSkipped => "call_skipped",
        }
    }
}

// ---------------------------------------------------------------------------
// HmxLatencyBucket — P50/P95/P99 suffix labels
// ---------------------------------------------------------------------------

/// Fixed percentile bucket labels for HMX latency histograms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HmxLatencyBucket {
    /// Cold-cluster hydrate.
    Cold,
    /// Warm-cluster hydrate.
    Warm,
}

impl HmxLatencyBucket {
    /// `snake_case` label used as the `kind=` tag value.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Cold => "cold",
            Self::Warm => "warm",
        }
    }
}

// ---------------------------------------------------------------------------
// MetricsConfig
// ---------------------------------------------------------------------------

/// Runtime configuration for the metrics subsystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricsConfig {
    /// Listen address for the Prometheus scrape endpoint.
    /// Parsed on init; `None` disables the HTTP listener (in-memory only).
    pub listen_addr: Option<String>,
    /// Global label pair applied to every emitted metric (e.g. `("service",
    /// "synthex-v2-shadow")`).
    pub global_labels: Vec<(String, String)>,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            // In sidecar phase we want /metrics accessible at :9090 of the
            // synthex-v2 pod. Final port is configurable via overlay_env in m03.
            listen_addr: Some("127.0.0.1:9464".to_owned()),
            global_labels: vec![("service".to_owned(), "synthex-v2".to_owned())],
        }
    }
}

impl MetricsConfig {
    /// Parse the listen address into a [`SocketAddr`], if set.
    ///
    /// # Errors
    ///
    /// Returns [`MetricsError::InvalidAddress`] on parse failure.
    pub fn parse_listen(&self) -> std::result::Result<Option<SocketAddr>, MetricsError> {
        self.listen_addr
            .as_deref()
            .map(|s| {
                s.parse::<SocketAddr>().map_err(|e| MetricsError::InvalidAddress {
                    input: s.to_owned(),
                    message: e.to_string(),
                })
            })
            .transpose()
    }
}

// ---------------------------------------------------------------------------
// MetricsCollector (init only)
// ---------------------------------------------------------------------------

/// Once-per-process bootstrapper for the Prometheus recorder.
pub struct MetricsCollector;

impl MetricsCollector {
    /// Install the Prometheus recorder. Idempotent via a module-private `Once`.
    /// Returns a [`PrometheusHandle`] the caller can use to render the metrics
    /// string on-demand (useful in tests and in `/metrics` endpoints that
    /// proxy rather than scrape).
    ///
    /// # Errors
    ///
    /// Returns [`MetricsError::AlreadyInitialised`] on second call,
    /// [`MetricsError::InvalidAddress`] on bad listen addr,
    /// [`MetricsError::InstallFailed`] on recorder install error.
    pub fn init(config: &MetricsConfig) -> std::result::Result<PrometheusHandle, MetricsError> {
        static INIT: Once = Once::new();
        let mut outcome: Result<Option<PrometheusHandle>, MetricsError> = Ok(None);
        let mut ran = false;
        INIT.call_once(|| {
            ran = true;
            outcome = Self::install(config);
        });
        if !ran {
            return Err(MetricsError::AlreadyInitialised);
        }
        match outcome {
            Ok(Some(h)) => Ok(h),
            Ok(None) => Err(MetricsError::InstallFailed(
                "recorder returned no handle".to_owned(),
            )),
            Err(e) => Err(e),
        }
    }

    fn install(
        config: &MetricsConfig,
    ) -> std::result::Result<Option<PrometheusHandle>, MetricsError> {
        let addr = config.parse_listen()?;
        let mut builder = PrometheusBuilder::new();
        for (k, v) in &config.global_labels {
            builder = builder.add_global_label(k, v);
        }
        if let Some(addr) = addr {
            builder = builder.with_http_listener(addr);
        }
        let handle = builder
            .install_recorder()
            .map_err(|e| MetricsError::InstallFailed(e.to_string()))?;
        Ok(Some(handle))
    }
}

// ---------------------------------------------------------------------------
// Typed emission façade
// ---------------------------------------------------------------------------

/// Record the current value of a tensor dimension.
///
/// Creates / updates a Prometheus gauge named
/// `synthex_v2_tensor_<dim_label>`. Calling with a non-finite value is a
/// no-op (metrics backends choke on `NaN`).
pub fn record_tensor_dim(dim: TensorDim, value: f64) {
    if !value.is_finite() {
        return;
    }
    gauge!(format!("synthex_v2_tensor_{}", dim.label())).set(value);
}

/// Record the current magnitude of a heat source.
///
/// Gauge name: `synthex_v2_heat_source`, label `id=<HS-00N>`.
pub fn record_heat_source(id: HeatSourceId, value: f64) {
    if !value.is_finite() {
        return;
    }
    gauge!("synthex_v2_heat_source", "id" => id.code()).set(value);
}

/// Record an observation of HMX hydrate latency (nanoseconds).
///
/// Histogram name: `synthex_v2_hmx_latency_ns`, label `kind=<cold|warm>`.
pub fn observe_hmx_latency(bucket: HmxLatencyBucket, ns: u64) {
    // Histogram takes f64 seconds-ish, but we keep ns in native units; the
    // prometheus exporter renders it faithfully. Precision loss above 2^53 ns
    // (~104 days) is irrelevant for latency observations.
    #[allow(clippy::cast_precision_loss)]
    let value = ns as f64;
    histogram!("synthex_v2_hmx_latency_ns", "kind" => bucket.label()).record(value);
}

/// Increment a bridge counter by 1.
///
/// Counter name: `synthex_v2_bridge_<variant>`, label `bridge=<ModuleId>`.
pub fn incr_bridge_counter(variant: BridgeCounter, bridge: &ModuleId) {
    counter!(
        format!("synthex_v2_bridge_{}", variant.label()),
        "bridge" => bridge.as_str().to_owned()
    )
    .increment(1);
}

/// Increment a general-purpose event counter for a named downstream service.
///
/// Counter name: `synthex_v2_ingest_event_total`, label `service=<ServiceId>`.
pub fn incr_ingest_event(service: &ServiceId) {
    counter!(
        "synthex_v2_ingest_event_total",
        "service" => service.as_str().to_owned()
    )
    .increment(1);
}

/// Increment the WebSocket frames-sent counter.
///
/// Counter: `synthex_v2_ws_frames_sent_total`, label `kind=<message_kind>`.
pub fn incr_ws_frames_sent(kind: &str) {
    counter!("synthex_v2_ws_frames_sent_total", "kind" => kind.to_owned()).increment(1);
}

/// Increment the WebSocket frames-received counter.
///
/// Counter: `synthex_v2_ws_frames_received_total`, label `kind=<message_kind>`.
pub fn incr_ws_frames_received(kind: &str) {
    counter!("synthex_v2_ws_frames_received_total", "kind" => kind.to_owned()).increment(1);
}

/// Set the active WebSocket connections gauge.
///
/// Gauge: `synthex_v2_ws_connections_active`.
pub fn set_ws_connections_active(count: u64) {
    #[allow(clippy::cast_precision_loss)]
    gauge!("synthex_v2_ws_connections_active").set(count as f64);
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    // ---------- TensorDim (7 tests) ----------

    #[test]
    fn tensor_dim_all_has_eleven() {
        assert_eq!(TensorDim::ALL.len(), 11);
    }

    #[test]
    fn tensor_dim_indices_are_zero_to_ten() {
        let indices: Vec<usize> = TensorDim::ALL.iter().map(|d| d.index()).collect();
        assert_eq!(indices, (0..=10).collect::<Vec<_>>());
    }

    #[test]
    fn tensor_dim_labels_are_unique() {
        let mut labels: Vec<&'static str> = TensorDim::ALL.iter().map(|d| d.label()).collect();
        labels.sort_unstable();
        let original_len = labels.len();
        labels.dedup();
        assert_eq!(labels.len(), original_len);
    }

    #[test]
    fn tensor_dim_display_matches_label() {
        assert_eq!(TensorDim::TemperatureT.to_string(), "temperature_t");
        assert_eq!(TensorDim::FlowState.to_string(), "flow_state");
    }

    #[test]
    fn tensor_dim_serde_snake_case_roundtrip() {
        let s = serde_json::to_string(&TensorDim::PhaseOnehotRecognize).unwrap();
        assert_eq!(s, "\"phase_onehot_recognize\"");
        let parsed: TensorDim = serde_json::from_str(&s).unwrap();
        assert_eq!(parsed, TensorDim::PhaseOnehotRecognize);
    }

    #[test]
    fn tensor_dim_label_matches_context_json_order() {
        // Locks the order to .claude/context.json tensor.names. Bump this test
        // AND context.json together if you ever reorder.
        let labels: Vec<&'static str> = TensorDim::ALL.iter().map(|d| d.label()).collect();
        assert_eq!(
            labels,
            vec![
                "temperature_t",
                "pv2_r",
                "ralph_fitness",
                "ltp_rate",
                "ltd_rate",
                "povm_growth",
                "thermal_delta",
                "phase_onehot_recognize",
                "phase_onehot_explore",
                "phase_onehot_consolidate",
                "flow_state",
            ]
        );
    }

    #[test]
    fn tensor_dim_index_is_const() {
        const IDX: usize = TensorDim::RalphFitness.index();
        assert_eq!(IDX, 2);
    }

    // ---------- BridgeCounter (5 tests) ----------

    #[test]
    fn bridge_counter_all_has_six() {
        assert_eq!(BridgeCounter::ALL.len(), 6);
    }

    #[test]
    fn bridge_counter_labels_are_unique() {
        let mut labels: Vec<&'static str> =
            BridgeCounter::ALL.iter().map(|c| c.label()).collect();
        labels.sort_unstable();
        let original = labels.len();
        labels.dedup();
        assert_eq!(labels.len(), original);
    }

    #[test]
    fn bridge_counter_label_circuit_opened() {
        assert_eq!(BridgeCounter::CircuitOpened.label(), "circuit_opened");
    }

    #[test]
    fn bridge_counter_label_call_skipped() {
        assert_eq!(BridgeCounter::CallSkipped.label(), "call_skipped");
    }

    #[test]
    fn bridge_counter_serde_roundtrip() {
        for c in BridgeCounter::ALL {
            let s = serde_json::to_string(&c).unwrap();
            let parsed: BridgeCounter = serde_json::from_str(&s).unwrap();
            assert_eq!(parsed, c);
        }
    }

    // ---------- HmxLatencyBucket (3 tests) ----------

    #[test]
    fn hmx_bucket_labels() {
        assert_eq!(HmxLatencyBucket::Cold.label(), "cold");
        assert_eq!(HmxLatencyBucket::Warm.label(), "warm");
    }

    #[test]
    fn hmx_bucket_serde_roundtrip() {
        for b in [HmxLatencyBucket::Cold, HmxLatencyBucket::Warm] {
            let s = serde_json::to_string(&b).unwrap();
            let parsed: HmxLatencyBucket = serde_json::from_str(&s).unwrap();
            assert_eq!(parsed, b);
        }
    }

    #[test]
    fn hmx_bucket_label_is_const_fn() {
        const LBL: &str = HmxLatencyBucket::Cold.label();
        assert_eq!(LBL, "cold");
    }

    // ---------- MetricsConfig (5 tests) ----------

    #[test]
    fn metrics_config_default_has_listener() {
        let c = MetricsConfig::default();
        assert!(c.listen_addr.is_some());
        assert_eq!(c.global_labels.len(), 1);
    }

    #[test]
    fn metrics_config_parse_listen_happy() {
        let c = MetricsConfig::default();
        let parsed = c.parse_listen().unwrap();
        assert!(parsed.is_some());
    }

    #[test]
    fn metrics_config_parse_listen_none_when_unset() {
        let c = MetricsConfig {
            listen_addr: None,
            global_labels: vec![],
        };
        assert!(c.parse_listen().unwrap().is_none());
    }

    #[test]
    fn metrics_config_parse_listen_rejects_garbage() {
        let c = MetricsConfig {
            listen_addr: Some("not an address".to_owned()),
            global_labels: vec![],
        };
        let err = c.parse_listen().unwrap_err();
        assert!(matches!(err, MetricsError::InvalidAddress { .. }));
    }

    #[test]
    fn metrics_config_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<MetricsConfig>();
    }

    // ---------- MetricsError (4 tests) ----------

    #[test]
    fn metrics_error_already_initialised_display() {
        assert!(MetricsError::AlreadyInitialised
            .to_string()
            .contains("already initialised"));
    }

    #[test]
    fn metrics_error_install_failed_display() {
        let e = MetricsError::InstallFailed("x".to_owned());
        assert!(e.to_string().contains("Prometheus recorder"));
    }

    #[test]
    fn metrics_error_invalid_address_display() {
        let e = MetricsError::InvalidAddress {
            input: "foo".to_owned(),
            message: "bad".to_owned(),
        };
        assert!(e.to_string().contains("foo"));
    }

    #[test]
    fn metrics_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<MetricsError>();
    }

    // ---------- Emission façade (6 tests) ----------
    //
    // Emission is a void side-effect that hits the global recorder. The
    // recorder may or may not be installed in the test process (serial init
    // from m04/m05 tests is fragile), so we test that these calls return
    // without panicking and accept the full range of typed inputs.

    #[test]
    fn record_tensor_dim_ignores_nan() {
        record_tensor_dim(TensorDim::RalphFitness, f64::NAN);
    }

    #[test]
    fn record_tensor_dim_accepts_all_dims() {
        for d in TensorDim::ALL {
            record_tensor_dim(d, 0.5);
        }
    }

    #[test]
    fn record_heat_source_ignores_inf() {
        record_heat_source(HeatSourceId::Hebbian, f64::INFINITY);
    }

    #[test]
    fn record_heat_source_accepts_all_ids() {
        for id in HeatSourceId::ALL {
            record_heat_source(id, 0.25);
        }
    }

    #[test]
    fn observe_hmx_latency_accepts_both_buckets() {
        observe_hmx_latency(HmxLatencyBucket::Cold, 60_000_000);
        observe_hmx_latency(HmxLatencyBucket::Warm, 1_500_000);
    }

    #[test]
    fn incr_bridge_and_ingest_counters() {
        let bridge = ModuleId::new("m35g_orac_bridge").unwrap();
        for v in BridgeCounter::ALL {
            incr_bridge_counter(v, &bridge);
        }
        let svc = ServiceId::new("orac-sidecar").unwrap();
        incr_ingest_event(&svc);
    }
}
