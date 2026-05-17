//! `m20_heat_source_hebbian` — HS-001 Hebbian heat from the PV2 Kuramoto field.
//!
//! ## What this module produces
//!
//! A single `[0.0, 0.85)` scalar — the *weighted* HS-001 contribution to the
//! composite thermal signal `T` consumed by [`m19_pid_controller`].
//!
//! The scalar is derived from three PV2 Kuramoto signals:
//!
//! | Signal | Source | Meaning |
//! |--------|--------|---------|
//! | `r` | PV2 `/field.r` | Order parameter in `[0, 1]`; 1.0 = phase-locked |
//! | `k_mod` | PV2 `/field.k_mod` | Coupling modulation (unbounded, saturating) |
//! | `spheres` | PV2 `/field.spheres` | Active sphere count (`u32`) |
//!
//! ## Soft ceiling at 0.85 (fixes V1 "pegged-at-1.0" bug)
//!
//! V1 normalised its Hebbian contribution to `[0, 1]` and routinely hit the
//! rail, destroying gradient information for the PID. V2 runs every
//! composite through a smooth asymptote that never quite reaches `0.85` —
//! giving the controller a few bits of headroom to differentiate a
//! "saturated" field from an "extremely saturated" field.
//!
//! ## Weight `w = 0.30`
//!
//! The contribution of HS-001 to the 5-source composite is `0.30` by
//! default (configurable via [`HebbianConfig::weight`]). The full composite
//! math lives in `m26_saturation_cap`; this module only produces its own
//! weighted contribution.
//!
//! ## Layer
//!
//! `m4_regulation`
//!
//! ## Dependencies
//!
//! - `m01_core_types` — `HeatSourceId`
//! - `m13_ingest_router` — event type (via [`HebbianInput::from_pv2_payload`])
//!
//! ## Invariants
//!
//! - Output `capped` is always in `[0.0, ceiling)`.
//! - Output `weighted` is always in `[0.0, weight * ceiling)`.
//! - `compute` is pure (no interior mutability, no side effects besides
//!   metric emission via `m05_metrics_collector`).

#![allow(clippy::module_name_repetitions)]

use serde::{Deserialize, Serialize};

use crate::daemon::ws_handler::WsFieldCapabilityTrace;
use crate::m1_foundation::m01_core_types::HeatSourceId;
use crate::m1_foundation::m02_error_taxonomy::RegulationError;
use crate::m1_foundation::m05_metrics_collector::record_heat_source;
use crate::m2_ingest::m13_ingest_router::{IngestEvent, IngestSource};

/// Weight of the capability channel in the composite output. Spec § 7.
pub const M20_CAPABILITY_WEIGHT: f64 = 0.10;

/// EMA half-life in seconds. After 30 s of no `CapabilityTrace` arrivals,
/// the capability term decays to 50% of its last value. Spec § 7.
pub const M20_CAPABILITY_EMA_HALF_LIFE_S: f64 = 30.0;

// ---------------------------------------------------------------------------
// Input
// ---------------------------------------------------------------------------

/// A snapshot of the PV2 Kuramoto field consumed by HS-001.
///
/// All fields are validated at construction; downstream math trusts the
/// bounds (`r ∈ [0, 1]`, `k_mod` finite).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HebbianInput {
    /// Kuramoto order parameter in `[0, 1]`.
    pub r: f64,
    /// Coupling modulation. Any finite value; saturates via `tanh` in the
    /// composite.
    pub k_mod: f64,
    /// Number of active spheres.
    pub spheres: u32,
}

impl HebbianInput {
    /// Build after validating each field.
    ///
    /// # Errors
    ///
    /// Returns [`RegulationError::NonFiniteHeat`] if `r` or `k_mod` is NaN
    /// / ∞, or if `r` is outside `[0, 1]`.
    pub fn new(r: f64, k_mod: f64, spheres: u32) -> Result<Self, RegulationError> {
        if !r.is_finite() || !k_mod.is_finite() {
            return Err(RegulationError::NonFiniteHeat {
                id: HeatSourceId::Hebbian.code(),
            });
        }
        if !(0.0..=1.0).contains(&r) {
            return Err(RegulationError::NonFiniteHeat {
                id: HeatSourceId::Hebbian.code(),
            });
        }
        Ok(Self { r, k_mod, spheres })
    }

    /// Best-effort extraction from a PV2 [`IngestEvent`] payload.
    ///
    /// Returns `None` if the event is not from an HTTP PV2 probe or if the
    /// payload is missing any of the three required fields. Returning
    /// `None` for malformed inputs is intentional — the ingest path is
    /// fire-and-forget; hard-failing on stale PV2 replies would stall the
    /// controller loop.
    #[must_use]
    pub fn from_pv2_payload(event: &IngestEvent) -> Option<Self> {
        if event.source != IngestSource::Http {
            return None;
        }
        let r = event.payload.get("r").and_then(serde_json::Value::as_f64)?;
        let k_mod = event
            .payload
            .get("k_mod")
            .and_then(serde_json::Value::as_f64)?;
        let spheres = event
            .payload
            .get("spheres")
            .and_then(serde_json::Value::as_u64)?;
        let spheres = u32::try_from(spheres).ok()?;
        Self::new(r, k_mod, spheres).ok()
    }
}

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/// HS-001 tunables. Immutable after [`HebbianConfig::validated`].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HebbianConfig {
    /// Contribution weight in the composite. V2 default `0.30`.
    pub weight: f64,
    /// Soft-cap knee — below this, output is linear in raw; above, a
    /// smooth asymptote to `ceiling`. V2 default `0.80`.
    pub knee: f64,
    /// Asymptotic ceiling. Output never reaches `ceiling` exactly. V2
    /// default `0.85`.
    pub ceiling: f64,
    /// Spheres-to-fraction denominator. `spheres / sphere_full = 1.0` at
    /// full saturation; V2 default `8.0`.
    pub sphere_full: f64,
    /// Mix of `r`. V2 default `0.5`.
    pub mix_r: f64,
    /// Mix of `tanh(k_mod)`. V2 default `0.3`.
    pub mix_k_mod: f64,
    /// Mix of `spheres_frac`. V2 default `0.2`.
    /// (`mix_r + mix_k_mod + mix_spheres == 1.0`.)
    pub mix_spheres: f64,
}

impl HebbianConfig {
    /// V2 canonical defaults.
    #[must_use]
    pub const fn defaults() -> Self {
        Self {
            weight: 0.30,
            knee: 0.80,
            ceiling: 0.85,
            sphere_full: 8.0,
            mix_r: 0.5,
            mix_k_mod: 0.3,
            mix_spheres: 0.2,
        }
    }

    /// Validate each bound.
    ///
    /// # Errors
    ///
    /// [`RegulationError::InvalidPidConfig`] — reused field-tagged variant
    /// — if any bound is outside its contract.
    pub fn validated(self) -> Result<Self, RegulationError> {
        for (name, v) in [
            ("weight", self.weight),
            ("knee", self.knee),
            ("ceiling", self.ceiling),
            ("sphere_full", self.sphere_full),
            ("mix_r", self.mix_r),
            ("mix_k_mod", self.mix_k_mod),
            ("mix_spheres", self.mix_spheres),
        ] {
            if !v.is_finite() {
                return Err(RegulationError::InvalidPidConfig {
                    field: name,
                    value: v,
                    reason: "non_finite",
                });
            }
            if v < 0.0 {
                return Err(RegulationError::InvalidPidConfig {
                    field: name,
                    value: v,
                    reason: "negative",
                });
            }
        }
        if self.knee >= self.ceiling {
            return Err(RegulationError::InvalidPidConfig {
                field: "knee",
                value: self.knee,
                reason: "min_ge_max",
            });
        }
        if self.sphere_full == 0.0 {
            return Err(RegulationError::InvalidPidConfig {
                field: "sphere_full",
                value: 0.0,
                reason: "zero",
            });
        }
        let mix_sum = self.mix_r + self.mix_k_mod + self.mix_spheres;
        if (mix_sum - 1.0).abs() > 1e-6 {
            return Err(RegulationError::InvalidPidConfig {
                field: "mix_spheres",
                value: mix_sum,
                reason: "mix_sum_not_one",
            });
        }
        Ok(self)
    }
}

impl Default for HebbianConfig {
    fn default() -> Self {
        Self::defaults()
    }
}

// ---------------------------------------------------------------------------
// Output
// ---------------------------------------------------------------------------

/// Per-tick output of [`HebbianHeatSource::compute`].
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct HebbianHeat {
    /// Always `HeatSourceId::Hebbian` ��� kept for symmetry with m26 composite.
    pub id: HeatSourceId,
    /// Pre-cap composite in `[0, 1]` (the mix sums to 1; each term is
    /// already in `[0, 1]`).
    pub raw: f64,
    /// Soft-capped composite — in `[0, ceiling)`. Equals `raw` when
    /// `raw <= knee`.
    pub capped: f64,
    /// `weight * capped` — the HS-001 contribution to the composite.
    pub weighted: f64,
    /// `true` if `raw > knee` (i.e. the soft-cap regime was active).
    pub ceiling_active: bool,
    /// PV2 channel contribution (weight 0.30), for per-channel telemetry.
    pub pv2_term: f64,
    /// Capability channel contribution (weight 0.10), for telemetry.
    pub capability_channel_term: f64,
}

// ---------------------------------------------------------------------------
// Heat source
// ---------------------------------------------------------------------------

/// HS-001 computer from PV2 Kuramoto field + `CapabilityTrace` channel.
///
/// The PV2 channel (weight 0.30) is stateless — `compute` is pure for
/// identical `input`. The capability channel (weight 0.10) carries EMA
/// state that decays between ticks. Call [`apply_capability_decay`]
/// every regulation tick and [`ingest_capability_trace`] on each
/// inbound `CapabilityTrace` frame.
#[derive(Debug, Clone, Copy)]
pub struct HebbianHeatSource {
    config: HebbianConfig,
    /// Current EMA value of the capability channel, in `[0, 1]`.
    capability_term: f64,
    /// Last update timestamp for decay calculation (Unix epoch ms).
    capability_last_update_ms: i64,
}

impl HebbianHeatSource {
    /// Build from explicit config.
    ///
    /// # Errors
    ///
    /// Propagates any validation failure from [`HebbianConfig::validated`].
    pub fn new(config: HebbianConfig) -> Result<Self, RegulationError> {
        Ok(Self {
            config: config.validated()?,
            capability_term: 0.0,
            capability_last_update_ms: -1,
        })
    }

    /// Build with V2 canonical defaults (`w=0.30`, `knee=0.80`, `ceiling=0.85`).
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(HebbianConfig::defaults()).unwrap_or_else(|_| Self {
            config: HebbianConfig::defaults(),
            capability_term: 0.0,
            capability_last_update_ms: -1,
        })
    }

    /// Active configuration.
    #[must_use]
    pub const fn config(&self) -> &HebbianConfig {
        &self.config
    }

    /// Current capability term value, for observability.
    #[must_use]
    pub const fn capability_term(&self) -> f64 {
        self.capability_term
    }

    /// Ingest a `CapabilityTrace` frame. Replaces the EMA value with a
    /// score derived from the trace's Hebbian summary.
    pub fn ingest_capability_trace(&mut self, trace: &WsFieldCapabilityTrace, now_ms: i64) {
        let score = trace.hebbian_summary.mean_weight.clamp(0.0, 1.0)
            * trace.load.clamp(0.0, 1.0);
        self.capability_term = score;
        self.capability_last_update_ms = now_ms;
    }

    /// Apply EMA decay to the capability channel. Call once per
    /// regulation tick with the current timestamp.
    pub fn apply_capability_decay(&mut self, now_ms: i64) {
        if self.capability_last_update_ms < 0 {
            return;
        }
        let dt_s = f64::from(
            i32::try_from((now_ms - self.capability_last_update_ms).max(0).min(i64::from(i32::MAX)))
                .unwrap_or(i32::MAX),
        ) / 1000.0;
        let decay = 0.5_f64.powf(dt_s / M20_CAPABILITY_EMA_HALF_LIFE_S);
        self.capability_term *= decay;
        self.capability_last_update_ms = now_ms;
    }

    /// Compute the HS-001 contribution for this PV2 snapshot.
    ///
    /// Also emits the canonical `synthex_v2_heat_source{id="HS-001"}` gauge
    /// via [`record_heat_source`] so that Prometheus and the 11D tensor
    /// registry see a consistent value.
    ///
    /// # Errors
    ///
    /// Infallible today, but returns [`Result`] so a future tightening of
    /// the contract (e.g. rejecting impossible `spheres` counts when V3
    /// makes them bounded) is non-breaking.
    #[allow(clippy::unnecessary_wraps)]
    pub fn compute(&self, input: HebbianInput) -> Result<HebbianHeat, RegulationError> {
        let r = input.r.clamp(0.0, 1.0);
        let k = input.k_mod.tanh().clamp(0.0, 1.0);
        let spheres_f = f64::from(input.spheres) / self.config.sphere_full;
        let spheres_frac = spheres_f.clamp(0.0, 1.0);

        let rk = self.config.mix_r.mul_add(r, self.config.mix_k_mod * k);
        let raw = self.config.mix_spheres.mul_add(spheres_frac, rk);

        let capped = soft_cap(raw, self.config.knee, self.config.ceiling);
        let pv2_term = capped * self.config.weight;
        let cap_term = self.capability_term.clamp(0.0, 1.0) * M20_CAPABILITY_WEIGHT;
        let weighted = (pv2_term + cap_term).min(self.config.ceiling);

        record_heat_source(HeatSourceId::Hebbian, capped);

        Ok(HebbianHeat {
            id: HeatSourceId::Hebbian,
            raw,
            capped,
            weighted,
            ceiling_active: raw > self.config.knee,
            pv2_term,
            capability_channel_term: cap_term,
        })
    }
}

impl Default for HebbianHeatSource {
    fn default() -> Self {
        Self::with_defaults()
    }
}

// ---------------------------------------------------------------------------
// Soft cap (pub(crate) — other heat sources reuse it)
// ---------------------------------------------------------------------------

/// Smooth asymptote: linear below `knee`, approaches `ceiling` above.
///
/// For `x <= knee`, returns `max(x, 0.0)`. For `x > knee`, returns
/// `knee + headroom * (x - knee) / ((x - knee) + headroom)` which
/// asymptotes to `ceiling` but never equals it for finite `x`.
///
/// Non-finite `x` returns `0.0`.
#[must_use]
pub(crate) fn soft_cap(x: f64, knee: f64, ceiling: f64) -> f64 {
    if !x.is_finite() {
        return 0.0;
    }
    if x <= knee {
        return x.max(0.0);
    }
    let headroom = ceiling - knee;
    let over = x - knee;
    headroom.mul_add(over / (over + headroom), knee)
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m1_foundation::m01_core_types::Timestamp;
    use assert_matches::assert_matches;
    use proptest::prelude::*;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn input(r: f64, k_mod: f64, spheres: u32) -> HebbianInput {
        HebbianInput::new(r, k_mod, spheres).expect("valid fixture")
    }

    fn source() -> HebbianHeatSource {
        HebbianHeatSource::with_defaults()
    }

    fn pv2_event(r: f64, k_mod: f64, spheres: u64) -> IngestEvent {
        IngestEvent {
            source: IngestSource::Http,
            commit_id: 1,
            seen_at: Timestamp::from_nanos(1),
            payload: serde_json::json!({
                "r": r,
                "k_mod": k_mod,
                "spheres": spheres,
            }),
        }
    }

    // -----------------------------------------------------------------------
    // Input validation (1-8)
    // -----------------------------------------------------------------------

    #[test]
    fn input_new_accepts_valid() {
        let i = HebbianInput::new(0.5, 1.0, 3).expect("valid");
        assert!((i.r - 0.5).abs() < 1e-12);
        assert_eq!(i.spheres, 3);
    }

    #[test]
    fn input_rejects_nan_r() {
        assert_matches!(
            HebbianInput::new(f64::NAN, 0.0, 0),
            Err(RegulationError::NonFiniteHeat { id: "HS-001" })
        );
    }

    #[test]
    fn input_rejects_inf_r() {
        assert_matches!(
            HebbianInput::new(f64::INFINITY, 0.0, 0),
            Err(RegulationError::NonFiniteHeat { .. })
        );
    }

    #[test]
    fn input_rejects_nan_k_mod() {
        assert_matches!(
            HebbianInput::new(0.5, f64::NAN, 0),
            Err(RegulationError::NonFiniteHeat { .. })
        );
    }

    #[test]
    fn input_rejects_r_above_one() {
        assert_matches!(
            HebbianInput::new(1.5, 0.0, 0),
            Err(RegulationError::NonFiniteHeat { .. })
        );
    }

    #[test]
    fn input_rejects_r_below_zero() {
        assert_matches!(
            HebbianInput::new(-0.01, 0.0, 0),
            Err(RegulationError::NonFiniteHeat { .. })
        );
    }

    #[test]
    fn input_accepts_r_at_boundaries() {
        HebbianInput::new(0.0, 0.0, 0).expect("r=0");
        HebbianInput::new(1.0, 0.0, 0).expect("r=1");
    }

    #[test]
    fn input_accepts_zero_spheres() {
        let i = HebbianInput::new(0.5, 0.0, 0).expect("ok");
        assert_eq!(i.spheres, 0);
    }

    // -----------------------------------------------------------------------
    // From PV2 payload (9-14)
    // -----------------------------------------------------------------------

    #[test]
    fn from_pv2_payload_happy_path() {
        let e = pv2_event(0.7, 2.0, 4);
        let i = HebbianInput::from_pv2_payload(&e).expect("parses");
        assert!((i.r - 0.7).abs() < 1e-12);
        assert_eq!(i.spheres, 4);
    }

    #[test]
    fn from_pv2_payload_returns_none_on_wrong_source() {
        let mut e = pv2_event(0.5, 1.0, 2);
        e.source = IngestSource::Sqlite;
        assert!(HebbianInput::from_pv2_payload(&e).is_none());
    }

    #[test]
    fn from_pv2_payload_returns_none_on_missing_field() {
        let e = IngestEvent {
            source: IngestSource::Http,
            commit_id: 0,
            seen_at: Timestamp::from_nanos(0),
            payload: serde_json::json!({ "r": 0.5, "k_mod": 0.0 }),
        };
        assert!(HebbianInput::from_pv2_payload(&e).is_none());
    }

    #[test]
    fn from_pv2_payload_returns_none_on_wrong_type() {
        let e = IngestEvent {
            source: IngestSource::Http,
            commit_id: 0,
            seen_at: Timestamp::from_nanos(0),
            payload: serde_json::json!({ "r": "nan_string", "k_mod": 0.0, "spheres": 2 }),
        };
        assert!(HebbianInput::from_pv2_payload(&e).is_none());
    }

    #[test]
    fn from_pv2_payload_returns_none_on_invalid_range() {
        let e = pv2_event(1.5, 0.0, 0);
        assert!(HebbianInput::from_pv2_payload(&e).is_none());
    }

    #[test]
    fn from_pv2_payload_returns_none_on_spheres_overflow() {
        let e = IngestEvent {
            source: IngestSource::Http,
            commit_id: 0,
            seen_at: Timestamp::from_nanos(0),
            payload: serde_json::json!({
                "r": 0.5,
                "k_mod": 0.0,
                "spheres": u64::from(u32::MAX) + 1,
            }),
        };
        assert!(HebbianInput::from_pv2_payload(&e).is_none());
    }

    // -----------------------------------------------------------------------
    // Config validation (15-22)
    // -----------------------------------------------------------------------

    #[test]
    fn config_defaults_match_spec() {
        let c = HebbianConfig::defaults();
        assert!((c.weight - 0.30).abs() < 1e-12);
        assert!((c.knee - 0.80).abs() < 1e-12);
        assert!((c.ceiling - 0.85).abs() < 1e-12);
    }

    #[test]
    fn config_defaults_are_valid() {
        HebbianConfig::defaults().validated().expect("defaults valid");
    }

    #[test]
    fn config_rejects_non_finite_weight() {
        let bad = HebbianConfig {
            weight: f64::NAN,
            ..HebbianConfig::defaults()
        };
        assert_matches!(
            bad.validated(),
            Err(RegulationError::InvalidPidConfig { field: "weight", .. })
        );
    }

    #[test]
    fn config_rejects_negative_knee() {
        let bad = HebbianConfig {
            knee: -0.01,
            ..HebbianConfig::defaults()
        };
        assert_matches!(
            bad.validated(),
            Err(RegulationError::InvalidPidConfig { field: "knee", reason: "negative", .. })
        );
    }

    #[test]
    fn config_rejects_knee_ge_ceiling() {
        let bad = HebbianConfig {
            knee: 0.85,
            ceiling: 0.85,
            ..HebbianConfig::defaults()
        };
        assert_matches!(
            bad.validated(),
            Err(RegulationError::InvalidPidConfig { field: "knee", reason: "min_ge_max", .. })
        );
    }

    #[test]
    fn config_rejects_zero_sphere_full() {
        let bad = HebbianConfig {
            sphere_full: 0.0,
            ..HebbianConfig::defaults()
        };
        assert_matches!(
            bad.validated(),
            Err(RegulationError::InvalidPidConfig { field: "sphere_full", .. })
        );
    }

    #[test]
    fn config_rejects_mix_sum_not_one() {
        let bad = HebbianConfig {
            mix_r: 0.4,
            mix_k_mod: 0.3,
            mix_spheres: 0.2, // sum = 0.9
            ..HebbianConfig::defaults()
        };
        assert_matches!(
            bad.validated(),
            Err(RegulationError::InvalidPidConfig { field: "mix_spheres", .. })
        );
    }

    #[test]
    fn config_accepts_mix_sum_exactly_one() {
        let ok = HebbianConfig {
            mix_r: 0.5,
            mix_k_mod: 0.3,
            mix_spheres: 0.2,
            ..HebbianConfig::defaults()
        };
        ok.validated().expect("sum=1.0 exactly");
    }

    // -----------------------------------------------------------------------
    // Soft cap (23-29)
    // -----------------------------------------------------------------------

    #[test]
    fn soft_cap_identity_below_knee() {
        assert!((soft_cap(0.5, 0.8, 0.85) - 0.5).abs() < 1e-12);
        assert!((soft_cap(0.0, 0.8, 0.85) - 0.0).abs() < 1e-12);
        assert!((soft_cap(0.799, 0.8, 0.85) - 0.799).abs() < 1e-12);
    }

    #[test]
    fn soft_cap_equals_knee_at_knee() {
        assert!((soft_cap(0.8, 0.8, 0.85) - 0.8).abs() < 1e-12);
    }

    #[test]
    fn soft_cap_strictly_below_ceiling() {
        for x in [0.81_f64, 0.9, 1.0, 5.0, 1e6] {
            let y = soft_cap(x, 0.8, 0.85);
            assert!(y < 0.85, "y={y} must be < ceiling at x={x}");
        }
    }

    #[test]
    fn soft_cap_monotonic_above_knee() {
        let mut prev = soft_cap(0.8, 0.8, 0.85);
        for step in 1..=20 {
            let x = 0.8 + f64::from(step) * 0.05;
            let y = soft_cap(x, 0.8, 0.85);
            assert!(y >= prev);
            prev = y;
        }
    }

    #[test]
    fn soft_cap_non_finite_returns_zero() {
        assert!((soft_cap(f64::NAN, 0.8, 0.85) - 0.0).abs() < 1e-12);
        assert!((soft_cap(f64::INFINITY, 0.8, 0.85) - 0.0).abs() < 1e-12);
    }

    #[test]
    fn soft_cap_approaches_ceiling_asymptotically() {
        let y_large = soft_cap(1e6, 0.8, 0.85);
        assert!((y_large - 0.85).abs() < 1e-3);
        assert!(y_large < 0.85);
    }

    #[test]
    fn soft_cap_clamps_negative_to_zero() {
        assert!((soft_cap(-0.5, 0.8, 0.85) - 0.0).abs() < 1e-12);
    }

    // -----------------------------------------------------------------------
    // Compute happy path (30-38)
    // -----------------------------------------------------------------------

    #[test]
    fn compute_zero_input_is_zero() {
        let out = source().compute(input(0.0, 0.0, 0)).expect("compute");
        assert!((out.raw - 0.0).abs() < 1e-12);
        assert!((out.capped - 0.0).abs() < 1e-12);
        assert!((out.weighted - 0.0).abs() < 1e-12);
        assert!(!out.ceiling_active);
    }

    #[test]
    fn compute_id_is_hebbian() {
        let out = source().compute(input(0.5, 0.5, 4)).expect("compute");
        assert_eq!(out.id, HeatSourceId::Hebbian);
    }

    #[test]
    fn compute_weighted_equals_weight_times_capped() {
        let s = source();
        let out = s.compute(input(0.6, 0.4, 3)).expect("compute");
        assert!((out.weighted - s.config().weight * out.capped).abs() < 1e-12);
    }

    #[test]
    fn compute_saturated_inputs_approach_ceiling() {
        let out = source().compute(input(1.0, 10.0, 8)).expect("compute");
        assert!(out.raw > 0.99);
    }

    #[test]
    fn compute_r_contribution_is_50_percent() {
        let out = source().compute(input(1.0, 0.0, 0)).expect("compute");
        assert!((out.raw - 0.5).abs() < 1e-9);
    }

    #[test]
    fn compute_k_mod_contribution_saturates_at_30_percent() {
        let out = source().compute(input(0.0, 10.0, 0)).expect("compute");
        assert!((out.raw - 0.3).abs() < 0.01);
    }

    #[test]
    fn compute_sphere_contribution_is_20_percent_at_full() {
        let out = source().compute(input(0.0, 0.0, 8)).expect("compute");
        assert!((out.raw - 0.2).abs() < 1e-9);
    }

    #[test]
    fn compute_sphere_saturates_at_full() {
        let a = source().compute(input(0.0, 0.0, 8)).expect("compute");
        let b = source().compute(input(0.0, 0.0, 100)).expect("compute");
        assert!((a.raw - b.raw).abs() < 1e-9);
    }

    #[test]
    fn compute_negative_k_mod_contributes_zero() {
        let with_neg = source().compute(input(0.5, -5.0, 4)).expect("compute");
        let with_zero = source().compute(input(0.5, 0.0, 4)).expect("compute");
        assert!((with_neg.raw - with_zero.raw).abs() < 1e-12);
    }

    // -----------------------------------------------------------------------
    // Compute: soft ceiling behavior (39-44)
    // -----------------------------------------------------------------------

    #[test]
    fn compute_below_knee_has_no_ceiling_action() {
        let out = source().compute(input(0.2, 0.2, 1)).expect("compute");
        assert!(!out.ceiling_active);
        assert!((out.raw - out.capped).abs() < 1e-12);
    }

    #[test]
    fn compute_above_knee_activates_ceiling() {
        // r=1, tanh(1)=0.76, spheres_frac=1 → raw ~ 0.928 > 0.80
        let out = source().compute(input(1.0, 1.0, 8)).expect("compute");
        assert!(out.ceiling_active);
        assert!(out.capped < out.raw);
    }

    #[test]
    fn compute_capped_always_below_ceiling() {
        let s = source();
        let ceil = s.config().ceiling;
        for r in [0.0_f64, 0.3, 0.5, 0.8, 1.0] {
            for k in [0.0_f64, 1.0, 10.0] {
                for sp in [0_u32, 2, 8, 100] {
                    let out = s.compute(input(r, k, sp)).expect("compute");
                    assert!(out.capped < ceil);
                }
            }
        }
    }

    #[test]
    fn compute_weighted_below_weight_times_ceiling() {
        let s = source();
        let bound = s.config().weight * s.config().ceiling;
        let out = s.compute(input(1.0, 100.0, 1000)).expect("compute");
        assert!(out.weighted < bound);
    }

    #[test]
    fn compute_is_deterministic() {
        let s = source();
        let a = s.compute(input(0.7, 0.5, 3)).expect("a");
        let b = s.compute(input(0.7, 0.5, 3)).expect("b");
        assert_eq!(a, b);
    }

    #[test]
    fn compute_monotonic_in_r() {
        let s = source();
        let a = s.compute(input(0.2, 0.0, 0)).expect("a");
        let b = s.compute(input(0.5, 0.0, 0)).expect("b");
        let c = s.compute(input(0.9, 0.0, 0)).expect("c");
        assert!(a.capped <= b.capped);
        assert!(b.capped <= c.capped);
    }

    // -----------------------------------------------------------------------
    // Config permutation (45-48)
    // -----------------------------------------------------------------------

    #[test]
    fn zero_weight_produces_zero_weighted() {
        let s = HebbianHeatSource::new(HebbianConfig {
            weight: 0.0,
            ..HebbianConfig::defaults()
        })
        .expect("build");
        let out = s.compute(input(1.0, 1.0, 8)).expect("compute");
        assert!(out.raw > 0.0);
        assert!((out.weighted - 0.0).abs() < 1e-12);
    }

    #[test]
    fn lower_sphere_full_reaches_saturation_faster() {
        let loose = HebbianHeatSource::new(HebbianConfig {
            sphere_full: 16.0,
            ..HebbianConfig::defaults()
        })
        .expect("build");
        let tight = HebbianHeatSource::new(HebbianConfig {
            sphere_full: 4.0,
            ..HebbianConfig::defaults()
        })
        .expect("build");
        let loose_out = loose.compute(input(0.0, 0.0, 4)).expect("loose");
        let tight_out = tight.compute(input(0.0, 0.0, 4)).expect("tight");
        assert!(tight_out.raw > loose_out.raw);
    }

    #[test]
    fn all_weight_in_r_channel_mirrors_r() {
        let s = HebbianHeatSource::new(HebbianConfig {
            mix_r: 1.0,
            mix_k_mod: 0.0,
            mix_spheres: 0.0,
            ..HebbianConfig::defaults()
        })
        .expect("build");
        let out = s.compute(input(0.3, 100.0, 100)).expect("compute");
        assert!((out.raw - 0.3).abs() < 1e-9);
    }

    #[test]
    fn with_defaults_equals_new_with_defaults() {
        let a = HebbianHeatSource::with_defaults();
        let b = HebbianHeatSource::new(HebbianConfig::defaults()).expect("build");
        assert_eq!(a.config(), b.config());
    }

    // -----------------------------------------------------------------------
    // Property tests (49-53)
    // -----------------------------------------------------------------------

    proptest! {
        #[test]
        fn prop_capped_within_bounds(
            r in 0.0_f64..=1.0,
            k in -10.0_f64..10.0,
            sp in 0_u32..1000,
        ) {
            let out = source().compute(input(r, k, sp)).expect("compute");
            prop_assert!(out.capped >= 0.0);
            prop_assert!(out.capped < source().config().ceiling);
        }

        #[test]
        fn prop_weighted_within_bounds(
            r in 0.0_f64..=1.0,
            k in -10.0_f64..10.0,
            sp in 0_u32..1000,
        ) {
            let s = source();
            let out = s.compute(input(r, k, sp)).expect("compute");
            let max = s.config().weight * s.config().ceiling;
            prop_assert!(out.weighted >= 0.0);
            prop_assert!(out.weighted < max);
        }

        #[test]
        fn prop_monotonic_in_r(
            r1 in 0.0_f64..=0.5,
            dr in 0.0_f64..=0.5,
            k in 0.0_f64..5.0,
            sp in 0_u32..10,
        ) {
            let s = source();
            let r2 = (r1 + dr).min(1.0);
            let a = s.compute(input(r1, k, sp)).expect("a");
            let b = s.compute(input(r2, k, sp)).expect("b");
            prop_assert!(b.capped >= a.capped - 1e-12);
        }

        #[test]
        fn prop_soft_cap_preserves_order(
            a in 0.0_f64..2.0,
            b in 0.0_f64..2.0,
        ) {
            let ya = soft_cap(a, 0.8, 0.85);
            let yb = soft_cap(b, 0.8, 0.85);
            if a <= b { prop_assert!(ya <= yb + 1e-12); } else { prop_assert!(yb <= ya + 1e-12); }
        }

        #[test]
        fn prop_from_payload_roundtrip(
            r in 0.0_f64..=1.0,
            k in -5.0_f64..5.0,
            sp in 0_u64..1_000_000,
        ) {
            let e = pv2_event(r, k, sp);
            let i = HebbianInput::from_pv2_payload(&e).expect("parses");
            prop_assert!((i.r - r).abs() < 1e-12);
            prop_assert!((i.k_mod - k).abs() < 1e-12);
            prop_assert_eq!(u64::from(i.spheres), sp);
        }
    }

    // ── Phase 2.5 (S117) — capability channel tests ──────────────────

    fn sample_trace(mean_weight: f64, load: f64) -> crate::daemon::ws_handler::WsFieldCapabilityTrace {
        crate::daemon::ws_handler::WsFieldCapabilityTrace {
            field_signature: "test".into(),
            hebbian_summary: crate::daemon::ws_handler::WsHebbianSummary {
                top_pathways: vec![],
                top_weights: vec![],
                mean_weight,
                pathway_count: 10,
            },
            active_tools: vec![],
            phase_region: 0.5,
            load,
        }
    }

    #[test]
    fn capability_term_starts_zero() {
        let hs = HebbianHeatSource::with_defaults();
        assert!((hs.capability_term() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn capability_channel_ingest_updates_term() {
        let mut hs = HebbianHeatSource::with_defaults();
        let trace = sample_trace(0.5, 0.6);
        hs.ingest_capability_trace(&trace, 1000);
        let expected = 0.5 * 0.6;
        assert!((hs.capability_term() - expected).abs() < 1e-9);
    }

    #[test]
    fn capability_channel_decays_to_near_zero_over_90s() {
        let mut hs = HebbianHeatSource::with_defaults();
        hs.ingest_capability_trace(&sample_trace(1.0, 1.0), 0);
        assert!((hs.capability_term() - 1.0).abs() < 1e-9);
        // 18 ticks × 5s = 90s = 3 half-lives → ~0.125
        let mut now_ms = 0_i64;
        for _ in 0..18 {
            now_ms += 5000;
            hs.apply_capability_decay(now_ms);
        }
        assert!(hs.capability_term() < 0.13, "after 90s (3 half-lives) expected < 0.13, got {}", hs.capability_term());
        assert!(hs.capability_term() > 0.10, "should not be 0 — got {}", hs.capability_term());
    }

    #[test]
    fn composite_respects_soft_cap_with_max_inputs() {
        let mut hs = HebbianHeatSource::with_defaults();
        hs.ingest_capability_trace(&sample_trace(1.0, 1.0), 1000);
        let input = HebbianInput::new(1.0, 10.0, 8).unwrap();
        let out = hs.compute(input).unwrap();
        assert!(out.weighted < hs.config().ceiling, "weighted {} must be < ceiling {}", out.weighted, hs.config().ceiling);
    }

    #[test]
    fn composite_uses_correct_weights() {
        let hs = HebbianHeatSource::with_defaults();
        let input = HebbianInput::new(1.0, 10.0, 8).unwrap();
        let out = hs.compute(input).unwrap();
        // capability_term = 0, so cap_term = 0, weighted = pv2_term only
        assert!((out.capability_channel_term - 0.0).abs() < f64::EPSILON);
        assert!(out.pv2_term > 0.0);
    }

    #[test]
    fn capability_telemetry_separate_from_pv2() {
        let mut hs = HebbianHeatSource::with_defaults();
        hs.ingest_capability_trace(&sample_trace(0.8, 0.9), 1000);
        let input = HebbianInput::new(0.5, 1.0, 4).unwrap();
        let out = hs.compute(input).unwrap();
        assert!(out.pv2_term > 0.0);
        assert!(out.capability_channel_term > 0.0);
        assert!((out.capability_channel_term - 0.8 * 0.9 * M20_CAPABILITY_WEIGHT).abs() < 1e-9);
    }
}
