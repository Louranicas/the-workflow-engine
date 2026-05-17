//! `m01_core_types` — Core bounded newtypes for SYNTHEX v2.
//!
//! Every typed concept crossing a module boundary has a bounded newtype here.
//! Raw `String`/`u64`/`f64` for domain values is banned at module boundaries
//! (P06). Validation happens at construction; downstream code trusts the
//! invariant without defensive re-checks.
//!
//! # Contents
//!
//! - [`Timestamp`] — nanosecond-precision monotonic-ish wall-clock
//! - [`DurationNs`] — unsigned nanosecond duration
//! - [`ServiceId`] — bounded kebab-case service identifier
//! - [`ModuleId`] — bounded `mNN_snake_case` module identifier
//! - [`HeatSourceId`] — closed enum of the five canonical heat sources
//! - [`FlowState`] — bounded `[0.0, 1.0]` flow-state fraction
//! - [`FitnessDelta`] — bounded `[-1.0, 1.0]` fitness delta
//! - [`Severity`] — ordered severity level (Trace…Critical)
//! - [`SnapshotId`] — `UUIDv7` snapshot identifier
//!
//! # Error handling
//!
//! Every fallible constructor returns [`Result<T, CoreTypeError>`](CoreTypeError).
//! [`m02_error_taxonomy`](super::m02_error_taxonomy) wraps [`CoreTypeError`] in the
//! crate-wide `SynthexError` via `#[from]`.
//!
//! # Layer
//!
//! L1 Foundation. No upward imports. Every other module may depend on this.

use std::cmp::Ordering;
use std::fmt;
use std::time::{Duration as StdDuration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Validation failures produced by core-type constructors.
///
/// Wrapped by `SynthexError::CoreType` in `m02_error_taxonomy` via `#[from]`.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum CoreTypeError {
    /// Identifier length outside allowed range.
    #[error("identifier length {actual} outside [{min}, {max}]")]
    IdentifierLength {
        /// Minimum allowed length (inclusive).
        min: usize,
        /// Maximum allowed length (inclusive).
        max: usize,
        /// Actual length observed.
        actual: usize,
    },
    /// Identifier contains disallowed characters.
    #[error("identifier {0:?} contains disallowed characters (allowed: {1})")]
    IdentifierCharset(String, &'static str),
    /// Numeric value outside bounded range.
    #[error("value {actual} outside [{min}, {max}]")]
    OutOfBounds {
        /// Lower bound (inclusive).
        min: f64,
        /// Upper bound (inclusive).
        max: f64,
        /// Actual value observed.
        actual: f64,
    },
    /// Floating value is `NaN` or infinite.
    #[error("value must be finite, got {0}")]
    NonFinite(f64),
    /// Unknown heat-source discriminator parsed from string.
    #[error("unknown heat source {0:?}")]
    UnknownHeatSource(String),
    /// Unknown severity parsed from string.
    #[error("unknown severity {0:?}")]
    UnknownSeverity(String),
}

// ---------------------------------------------------------------------------
// Timestamp
// ---------------------------------------------------------------------------

/// Nanosecond-precision wall-clock timestamp since UNIX epoch.
///
/// Represented as `u64` nanoseconds so it matches `gradient_snapshot.ts_ns`
/// exactly. Constructed via [`Timestamp::now`] for current time or
/// [`Timestamp::from_nanos`] for replay/tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(u64);

impl Timestamp {
    /// Current system time as nanoseconds since UNIX epoch.
    ///
    /// Falls back to `0` if the system clock is before the epoch (impossible
    /// on any sane host; the branch exists so we never panic).
    #[must_use]
    pub fn now() -> Self {
        let ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| {
                let nanos = d.as_nanos();
                if nanos > u128::from(u64::MAX) {
                    u64::MAX
                } else {
                    // Bound verified immediately above — safe cast.
                    #[allow(clippy::cast_possible_truncation)]
                    {
                        nanos as u64
                    }
                }
            });
        Self(ns)
    }

    /// Construct from raw nanoseconds.
    #[must_use]
    pub const fn from_nanos(ns: u64) -> Self {
        Self(ns)
    }

    /// Nanoseconds since UNIX epoch.
    #[must_use]
    pub const fn as_nanos(self) -> u64 {
        self.0
    }

    /// Seconds since UNIX epoch (truncating). Integer division is intentional —
    /// sub-second precision lives in the full nanosecond view.
    #[must_use]
    #[allow(clippy::integer_division)]
    pub const fn as_secs(self) -> u64 {
        self.0 / 1_000_000_000
    }

    /// Add a duration, saturating at [`u64::MAX`].
    #[must_use]
    pub const fn saturating_add(self, d: DurationNs) -> Self {
        Self(self.0.saturating_add(d.0))
    }

    /// Subtract `earlier`, returning the elapsed duration (saturating at zero).
    #[must_use]
    pub const fn saturating_sub(self, earlier: Self) -> DurationNs {
        DurationNs(self.0.saturating_sub(earlier.0))
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}ns", self.0)
    }
}

impl From<u64> for Timestamp {
    fn from(ns: u64) -> Self {
        Self(ns)
    }
}

impl From<Timestamp> for u64 {
    fn from(t: Timestamp) -> Self {
        t.0
    }
}

// ---------------------------------------------------------------------------
// DurationNs
// ---------------------------------------------------------------------------

/// Unsigned nanosecond duration.
///
/// Wraps `u64` rather than `std::time::Duration` so it round-trips losslessly
/// through `serde_json` and `SQLite` `INTEGER`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DurationNs(u64);

impl DurationNs {
    /// Zero duration.
    pub const ZERO: Self = Self(0);

    /// Construct from raw nanoseconds.
    #[must_use]
    pub const fn from_nanos(ns: u64) -> Self {
        Self(ns)
    }

    /// Construct from milliseconds (saturating to [`u64::MAX`] on overflow).
    #[must_use]
    pub const fn from_millis(ms: u64) -> Self {
        Self(ms.saturating_mul(1_000_000))
    }

    /// Construct from seconds (saturating to [`u64::MAX`] on overflow).
    #[must_use]
    pub const fn from_secs(s: u64) -> Self {
        Self(s.saturating_mul(1_000_000_000))
    }

    /// Total nanoseconds.
    #[must_use]
    pub const fn as_nanos(self) -> u64 {
        self.0
    }

    /// Total milliseconds (truncating).
    #[must_use]
    #[allow(clippy::integer_division)]
    pub const fn as_millis(self) -> u64 {
        self.0 / 1_000_000
    }

    /// Total seconds (truncating).
    #[must_use]
    #[allow(clippy::integer_division)]
    pub const fn as_secs(self) -> u64 {
        self.0 / 1_000_000_000
    }

    /// Convert to a [`std::time::Duration`] for interop with tokio/std APIs.
    #[must_use]
    pub const fn to_std(self) -> StdDuration {
        StdDuration::from_nanos(self.0)
    }
}

impl fmt::Display for DurationNs {
    // Precision loss in the `f64` branches is acceptable — Display is for humans,
    // not for round-trip. Values >= 2^53 ns (~104 days) lose ns-level precision
    // but the displayed ms/s rendering is still correct to 3 decimal places.
    #[allow(clippy::cast_precision_loss)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 >= 1_000_000_000 {
            write!(f, "{:.3}s", (self.0 as f64) / 1e9)
        } else if self.0 >= 1_000_000 {
            write!(f, "{:.3}ms", (self.0 as f64) / 1e6)
        } else {
            write!(f, "{}ns", self.0)
        }
    }
}

impl From<StdDuration> for DurationNs {
    fn from(d: StdDuration) -> Self {
        let nanos = d.as_nanos();
        if nanos > u128::from(u64::MAX) {
            Self(u64::MAX)
        } else {
            // Bound verified immediately above — safe cast.
            #[allow(clippy::cast_possible_truncation)]
            Self(nanos as u64)
        }
    }
}

impl From<DurationNs> for StdDuration {
    fn from(d: DurationNs) -> Self {
        Self::from_nanos(d.0)
    }
}

// ---------------------------------------------------------------------------
// ServiceId
// ---------------------------------------------------------------------------

/// Bounded kebab-case service identifier.
///
/// # Invariants
///
/// - Length 1..=64
/// - Charset: `[a-z0-9-]` only
/// - Must not start or end with `-`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ServiceId(String);

impl ServiceId {
    /// Minimum length (inclusive).
    pub const MIN_LEN: usize = 1;
    /// Maximum length (inclusive).
    pub const MAX_LEN: usize = 64;
    /// Human-readable charset description.
    pub const CHARSET: &'static str = "a-z 0-9 '-' (not leading/trailing)";

    /// Validate and wrap a string.
    ///
    /// # Errors
    ///
    /// Returns [`CoreTypeError::IdentifierLength`] if length is outside `[1, 64]`
    /// or [`CoreTypeError::IdentifierCharset`] if the string contains disallowed
    /// characters or leading/trailing hyphens.
    pub fn new(raw: impl Into<String>) -> Result<Self, CoreTypeError> {
        let s = raw.into();
        let len = s.len();
        if !(Self::MIN_LEN..=Self::MAX_LEN).contains(&len) {
            return Err(CoreTypeError::IdentifierLength {
                min: Self::MIN_LEN,
                max: Self::MAX_LEN,
                actual: len,
            });
        }
        if s.starts_with('-') || s.ends_with('-') {
            return Err(CoreTypeError::IdentifierCharset(s, Self::CHARSET));
        }
        if !s.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-') {
            return Err(CoreTypeError::IdentifierCharset(s, Self::CHARSET));
        }
        Ok(Self(s))
    }

    /// Borrowed string view.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Move the inner `String` out.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Display for ServiceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for ServiceId {
    type Error = CoreTypeError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

impl TryFrom<&str> for ServiceId {
    type Error = CoreTypeError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::new(s.to_owned())
    }
}

impl From<ServiceId> for String {
    fn from(id: ServiceId) -> Self {
        id.0
    }
}

// ---------------------------------------------------------------------------
// ModuleId
// ---------------------------------------------------------------------------

/// Module identifier of the form `mNN[letter]_snake_case`
/// (e.g., `m01_core_types`, `m35i_codesynthor_v8_bridge`).
///
/// # Invariants
///
/// - Length 4..=64
/// - Starts with `m` followed by 2..=3 digits
/// - Optional single lowercase ASCII letter (sub-module suffix: `a`..=`z`)
/// - Then `_`
/// - Tail is `[a-z0-9_]`, no trailing `_`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ModuleId(String);

impl ModuleId {
    /// Minimum length (inclusive).
    pub const MIN_LEN: usize = 4;
    /// Maximum length (inclusive).
    pub const MAX_LEN: usize = 64;
    /// Human-readable charset description.
    pub const CHARSET: &'static str =
        "m + 2-3 digits + optional [a-z] + _ + [a-z0-9_] (no trailing _)";

    /// Validate and wrap a string.
    ///
    /// # Errors
    ///
    /// Returns [`CoreTypeError::IdentifierLength`] or
    /// [`CoreTypeError::IdentifierCharset`] on violation.
    pub fn new(raw: impl Into<String>) -> Result<Self, CoreTypeError> {
        let s = raw.into();
        let len = s.len();
        if !(Self::MIN_LEN..=Self::MAX_LEN).contains(&len) {
            return Err(CoreTypeError::IdentifierLength {
                min: Self::MIN_LEN,
                max: Self::MAX_LEN,
                actual: len,
            });
        }
        if s.ends_with('_') {
            return Err(CoreTypeError::IdentifierCharset(s, Self::CHARSET));
        }
        let charset_err = || CoreTypeError::IdentifierCharset(s.clone(), Self::CHARSET);
        let mut bytes = s.bytes();
        if bytes.next() != Some(b'm') {
            return Err(charset_err());
        }
        let mut digits: u8 = 0;
        let mut saw_suffix = false;
        let tail = loop {
            match bytes.next() {
                Some(b) if b.is_ascii_digit() => {
                    if saw_suffix || digits >= 3 {
                        return Err(charset_err());
                    }
                    digits += 1;
                }
                // Optional single-letter sub-module suffix (m35a, m35i, ...).
                Some(b) if b.is_ascii_lowercase() && !saw_suffix && digits >= 2 => {
                    saw_suffix = true;
                }
                Some(b'_') => break bytes,
                _ => return Err(charset_err()),
            }
        };
        if digits < 2 {
            return Err(charset_err());
        }
        for b in tail {
            if !(b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_') {
                return Err(charset_err());
            }
        }
        Ok(Self(s))
    }

    /// Borrowed string view.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Move the inner `String` out.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Display for ModuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for ModuleId {
    type Error = CoreTypeError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

impl TryFrom<&str> for ModuleId {
    type Error = CoreTypeError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::new(s.to_owned())
    }
}

impl From<ModuleId> for String {
    fn from(id: ModuleId) -> Self {
        id.0
    }
}

// ---------------------------------------------------------------------------
// HeatSourceId
// ---------------------------------------------------------------------------

/// Closed enum of canonical heat sources (matches `gradient_snapshot.hs_NNN_*`).
///
/// New heat sources require a schema migration AND an enum variant. Exhaustive
/// `match` is what keeps L4 regulation and L3 persistence in lockstep.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeatSourceId {
    /// HS-001: Hebbian saturation (LTP/LTD ratio → [0,1]).
    Hebbian,
    /// HS-002: Cascade propagation through service graph.
    Cascade,
    /// HS-003: Resonance (ME/RALPH fitness proxy).
    Resonance,
    /// HS-004: Cross-service synchrony (Kuramoto r over bridges).
    CrossSync,
    /// HS-005: Cross-RALPH phase coherence.
    CrossRalph,
}

impl HeatSourceId {
    /// All variants in canonical HS-001…HS-005 order.
    pub const ALL: [Self; 5] = [
        Self::Hebbian,
        Self::Cascade,
        Self::Resonance,
        Self::CrossSync,
        Self::CrossRalph,
    ];

    /// Canonical short code (`HS-001` … `HS-005`).
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Hebbian => "HS-001",
            Self::Cascade => "HS-002",
            Self::Resonance => "HS-003",
            Self::CrossSync => "HS-004",
            Self::CrossRalph => "HS-005",
        }
    }

    /// `snake_case` label matching the `gradient_snapshot` column suffix.
    #[must_use]
    pub const fn column_suffix(self) -> &'static str {
        match self {
            Self::Hebbian => "hebbian",
            Self::Cascade => "cascade",
            Self::Resonance => "resonance",
            Self::CrossSync => "crosssync",
            Self::CrossRalph => "cross_ralph",
        }
    }

    /// Parse from the [`Self::code`] form (`HS-NNN`).
    ///
    /// # Errors
    ///
    /// Returns [`CoreTypeError::UnknownHeatSource`] if the code does not match
    /// a known variant.
    pub fn from_code(code: &str) -> Result<Self, CoreTypeError> {
        match code {
            "HS-001" => Ok(Self::Hebbian),
            "HS-002" => Ok(Self::Cascade),
            "HS-003" => Ok(Self::Resonance),
            "HS-004" => Ok(Self::CrossSync),
            "HS-005" => Ok(Self::CrossRalph),
            other => Err(CoreTypeError::UnknownHeatSource(other.to_owned())),
        }
    }
}

impl fmt::Display for HeatSourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

// ---------------------------------------------------------------------------
// FlowState — bounded [0.0, 1.0]
// ---------------------------------------------------------------------------

/// Flow-state fraction, bounded `[0.0, 1.0]`.
///
/// Used for tensor dim 10 (`flow_state`). Construction rejects `NaN`/`Inf`
/// and out-of-range values; downstream code trusts the bound.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "f64", into = "f64")]
pub struct FlowState(f64);

impl FlowState {
    /// Minimum value (inclusive).
    pub const MIN: f64 = 0.0;
    /// Maximum value (inclusive).
    pub const MAX: f64 = 1.0;

    /// Validate and wrap.
    ///
    /// # Errors
    ///
    /// Returns [`CoreTypeError::NonFinite`] for `NaN`/`Inf`, or
    /// [`CoreTypeError::OutOfBounds`] if outside `[0.0, 1.0]`.
    pub fn new(v: f64) -> Result<Self, CoreTypeError> {
        if !v.is_finite() {
            return Err(CoreTypeError::NonFinite(v));
        }
        if !(Self::MIN..=Self::MAX).contains(&v) {
            return Err(CoreTypeError::OutOfBounds {
                min: Self::MIN,
                max: Self::MAX,
                actual: v,
            });
        }
        Ok(Self(v))
    }

    /// Clamp any finite `f64` into `[0.0, 1.0]`; returns `0.0` for non-finite
    /// input. Never fails; use when the caller is OK with saturation.
    #[must_use]
    pub const fn clamp_from(v: f64) -> Self {
        if !v.is_finite() {
            return Self(0.0);
        }
        let c = if v < Self::MIN {
            Self::MIN
        } else if v > Self::MAX {
            Self::MAX
        } else {
            v
        };
        Self(c)
    }

    /// Inner value.
    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }
}

impl Eq for FlowState {}

impl PartialOrd for FlowState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FlowState {
    fn cmp(&self, other: &Self) -> Ordering {
        // Values are always finite by construction, so `total_cmp` gives a
        // strict total order with no NaN edge cases to reason about.
        self.0.total_cmp(&other.0)
    }
}

impl fmt::Display for FlowState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4}", self.0)
    }
}

impl TryFrom<f64> for FlowState {
    type Error = CoreTypeError;
    fn try_from(v: f64) -> Result<Self, Self::Error> {
        Self::new(v)
    }
}

impl From<FlowState> for f64 {
    fn from(f: FlowState) -> Self {
        f.0
    }
}

// ---------------------------------------------------------------------------
// FitnessDelta — bounded [-1.0, 1.0]
// ---------------------------------------------------------------------------

/// Fitness delta, bounded `[-1.0, 1.0]`.
///
/// Positive = fitness improved; negative = regressed. Used by the rollback
/// threshold (P28 + P0-4: auto-rollback if Δ < -0.03).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "f64", into = "f64")]
pub struct FitnessDelta(f64);

impl FitnessDelta {
    /// Minimum value (inclusive).
    pub const MIN: f64 = -1.0;
    /// Maximum value (inclusive).
    pub const MAX: f64 = 1.0;
    /// Canonical auto-rollback threshold (proposals with `delta < ROLLBACK_THRESHOLD`
    /// trigger automatic rollback per P28).
    pub const ROLLBACK_THRESHOLD: f64 = -0.03;

    /// Validate and wrap.
    ///
    /// # Errors
    ///
    /// Returns [`CoreTypeError::NonFinite`] for `NaN`/`Inf`, or
    /// [`CoreTypeError::OutOfBounds`] if outside `[-1.0, 1.0]`.
    pub fn new(v: f64) -> Result<Self, CoreTypeError> {
        if !v.is_finite() {
            return Err(CoreTypeError::NonFinite(v));
        }
        if !(Self::MIN..=Self::MAX).contains(&v) {
            return Err(CoreTypeError::OutOfBounds {
                min: Self::MIN,
                max: Self::MAX,
                actual: v,
            });
        }
        Ok(Self(v))
    }

    /// Clamp any finite `f64` into `[-1.0, 1.0]`; returns `0.0` for non-finite.
    #[must_use]
    pub const fn clamp_from(v: f64) -> Self {
        if !v.is_finite() {
            return Self(0.0);
        }
        let c = if v < Self::MIN {
            Self::MIN
        } else if v > Self::MAX {
            Self::MAX
        } else {
            v
        };
        Self(c)
    }

    /// Inner value.
    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }

    /// `true` if `self < ROLLBACK_THRESHOLD`.
    #[must_use]
    pub fn triggers_rollback(self) -> bool {
        self.0 < Self::ROLLBACK_THRESHOLD
    }
}

impl Eq for FitnessDelta {}

impl PartialOrd for FitnessDelta {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FitnessDelta {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl fmt::Display for FitnessDelta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:+.4}", self.0)
    }
}

impl TryFrom<f64> for FitnessDelta {
    type Error = CoreTypeError;
    fn try_from(v: f64) -> Result<Self, Self::Error> {
        Self::new(v)
    }
}

impl From<FitnessDelta> for f64 {
    fn from(d: FitnessDelta) -> Self {
        d.0
    }
}

// ---------------------------------------------------------------------------
// Severity
// ---------------------------------------------------------------------------

/// Ordered severity level.
///
/// Derived ordering: `Trace < Debug < Info < Warn < Error < Critical`.
/// Maps 1:1 to `tracing::Level` for `Trace..=Error`; `Critical` is SYNTHEX-specific
/// for PBFT-blocking conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Fine-grained per-tick diagnostics.
    Trace,
    /// Developer-mode diagnostic.
    Debug,
    /// Normal operational event.
    Info,
    /// Recoverable anomaly; monitoring should notice.
    Warn,
    /// Service-level failure; circuit breaker likely trips.
    Error,
    /// PBFT-blocking / Ember-tripping condition.
    Critical,
}

impl Severity {
    /// All variants in ascending order.
    pub const ALL: [Self; 6] = [
        Self::Trace,
        Self::Debug,
        Self::Info,
        Self::Warn,
        Self::Error,
        Self::Critical,
    ];

    /// Canonical lowercase label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Trace => "trace",
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
            Self::Critical => "critical",
        }
    }

    /// Parse the lowercase label produced by [`Self::as_str`].
    ///
    /// # Errors
    ///
    /// Returns [`CoreTypeError::UnknownSeverity`] if the label does not match.
    pub fn parse(s: &str) -> Result<Self, CoreTypeError> {
        match s {
            "trace" => Ok(Self::Trace),
            "debug" => Ok(Self::Debug),
            "info" => Ok(Self::Info),
            "warn" | "warning" => Ok(Self::Warn),
            "error" => Ok(Self::Error),
            "critical" | "crit" | "fatal" => Ok(Self::Critical),
            other => Err(CoreTypeError::UnknownSeverity(other.to_owned())),
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// SnapshotId — UUIDv7
// ---------------------------------------------------------------------------

/// `UUIDv7` snapshot identifier.
///
/// v7 is time-ordered, so sorting by `SnapshotId` sorts by creation instant —
/// useful for `deployment_snapshot.snapshot_id` PK.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SnapshotId(Uuid);

impl SnapshotId {
    /// Generate a fresh v7 identifier (time-ordered).
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Wrap an existing `Uuid`.
    #[must_use]
    pub const fn from_uuid(u: Uuid) -> Self {
        Self(u)
    }

    /// Extract the inner `Uuid`.
    #[must_use]
    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for SnapshotId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SnapshotId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<Uuid> for SnapshotId {
    fn from(u: Uuid) -> Self {
        Self(u)
    }
}

impl From<SnapshotId> for Uuid {
    fn from(s: SnapshotId) -> Self {
        s.0
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::float_cmp)]
mod tests {
    use super::*;

    // ---------- Timestamp (7 tests) ----------

    #[test]
    fn timestamp_now_is_nonzero() {
        assert!(Timestamp::now().as_nanos() > 0);
    }

    #[test]
    fn timestamp_from_and_as_nanos_roundtrip() {
        let t = Timestamp::from_nanos(1_776_508_215_613_314_209);
        assert_eq!(t.as_nanos(), 1_776_508_215_613_314_209);
    }

    #[test]
    fn timestamp_as_secs_truncates() {
        let t = Timestamp::from_nanos(1_500_000_000);
        assert_eq!(t.as_secs(), 1);
    }

    #[test]
    fn timestamp_saturating_add() {
        let t = Timestamp::from_nanos(10);
        let sum = t.saturating_add(DurationNs::from_nanos(5));
        assert_eq!(sum.as_nanos(), 15);
    }

    #[test]
    fn timestamp_saturating_add_overflow() {
        let t = Timestamp::from_nanos(u64::MAX - 1);
        let sum = t.saturating_add(DurationNs::from_nanos(1000));
        assert_eq!(sum.as_nanos(), u64::MAX);
    }

    #[test]
    fn timestamp_saturating_sub_clamps_to_zero() {
        let a = Timestamp::from_nanos(5);
        let b = Timestamp::from_nanos(10);
        assert_eq!(a.saturating_sub(b), DurationNs::ZERO);
    }

    #[test]
    fn timestamp_ordering_is_monotonic() {
        let a = Timestamp::from_nanos(1);
        let b = Timestamp::from_nanos(2);
        assert!(a < b);
        assert!(b > a);
    }

    #[test]
    fn timestamp_serde_roundtrip() {
        let t = Timestamp::from_nanos(42);
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "42");
        let parsed: Timestamp = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, t);
    }

    // ---------- DurationNs (6 tests) ----------

    #[test]
    fn duration_zero_const() {
        assert_eq!(DurationNs::ZERO.as_nanos(), 0);
    }

    #[test]
    fn duration_from_secs_millis_nanos() {
        assert_eq!(DurationNs::from_secs(1).as_nanos(), 1_000_000_000);
        assert_eq!(DurationNs::from_millis(1).as_nanos(), 1_000_000);
        assert_eq!(DurationNs::from_nanos(1).as_nanos(), 1);
    }

    #[test]
    fn duration_from_secs_saturates_on_overflow() {
        let d = DurationNs::from_secs(u64::MAX);
        assert_eq!(d.as_nanos(), u64::MAX);
    }

    #[test]
    fn duration_display_scales() {
        assert_eq!(DurationNs::from_nanos(500).to_string(), "500ns");
        assert_eq!(DurationNs::from_millis(5).to_string(), "5.000ms");
        assert_eq!(DurationNs::from_secs(2).to_string(), "2.000s");
    }

    #[test]
    fn duration_to_std_roundtrip() {
        let d = DurationNs::from_millis(250);
        let std_d: StdDuration = d.into();
        assert_eq!(std_d.as_millis(), 250);
        let back: DurationNs = std_d.into();
        assert_eq!(back, d);
    }

    #[test]
    fn duration_from_std_saturates_on_overflow() {
        // u64::MAX nanos ≈ 584 years; use a huge StdDuration to trigger the cap.
        let huge = StdDuration::new(u64::MAX, 0);
        let d: DurationNs = huge.into();
        assert_eq!(d.as_nanos(), u64::MAX);
    }

    #[test]
    fn duration_serde_roundtrip() {
        let d = DurationNs::from_millis(100);
        let json = serde_json::to_string(&d).unwrap();
        assert_eq!(json, "100000000");
        let parsed: DurationNs = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, d);
    }

    // ---------- ServiceId (8 tests) ----------

    #[test]
    fn service_id_happy_path() {
        let id = ServiceId::new("synthex-v2").unwrap();
        assert_eq!(id.as_str(), "synthex-v2");
    }

    #[test]
    fn service_id_rejects_empty() {
        assert!(matches!(
            ServiceId::new(""),
            Err(CoreTypeError::IdentifierLength { .. })
        ));
    }

    #[test]
    fn service_id_rejects_too_long() {
        let long = "a".repeat(65);
        assert!(matches!(
            ServiceId::new(long),
            Err(CoreTypeError::IdentifierLength { actual: 65, .. })
        ));
    }

    #[test]
    fn service_id_rejects_uppercase() {
        assert!(matches!(
            ServiceId::new("Synthex-V2"),
            Err(CoreTypeError::IdentifierCharset(_, _))
        ));
    }

    #[test]
    fn service_id_rejects_leading_hyphen() {
        assert!(matches!(
            ServiceId::new("-bad"),
            Err(CoreTypeError::IdentifierCharset(_, _))
        ));
    }

    #[test]
    fn service_id_rejects_trailing_hyphen() {
        assert!(matches!(
            ServiceId::new("bad-"),
            Err(CoreTypeError::IdentifierCharset(_, _))
        ));
    }

    #[test]
    fn service_id_rejects_underscore() {
        assert!(matches!(
            ServiceId::new("bad_name"),
            Err(CoreTypeError::IdentifierCharset(_, _))
        ));
    }

    #[test]
    fn service_id_serde_roundtrip() {
        let id = ServiceId::new("orac-sidecar").unwrap();
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"orac-sidecar\"");
        let parsed: ServiceId = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, id);
    }

    #[test]
    fn service_id_tryfrom_str_and_string() {
        let a: ServiceId = "pv2".try_into().unwrap();
        let b: ServiceId = String::from("pv2").try_into().unwrap();
        assert_eq!(a, b);
    }

    // ---------- ModuleId (7 tests) ----------

    #[test]
    fn module_id_happy_path() {
        let id = ModuleId::new("m01_core_types").unwrap();
        assert_eq!(id.as_str(), "m01_core_types");
    }

    #[test]
    fn module_id_accepts_three_digit() {
        let id = ModuleId::new("m100_future").unwrap();
        assert_eq!(id.as_str(), "m100_future");
    }

    #[test]
    fn module_id_accepts_letter_suffix() {
        let id = ModuleId::new("m35i_codesynthor_v8_bridge").unwrap();
        assert_eq!(id.as_str(), "m35i_codesynthor_v8_bridge");
    }

    #[test]
    fn module_id_accepts_two_digit_letter() {
        let id = ModuleId::new("m35a_architect_bridge").unwrap();
        assert_eq!(id.as_str(), "m35a_architect_bridge");
    }

    #[test]
    fn module_id_rejects_double_letter_suffix() {
        assert!(matches!(
            ModuleId::new("m35ab_bad"),
            Err(CoreTypeError::IdentifierCharset(_, _))
        ));
    }

    #[test]
    fn module_id_rejects_digit_after_letter_suffix() {
        assert!(matches!(
            ModuleId::new("m35i9_bad"),
            Err(CoreTypeError::IdentifierCharset(_, _))
        ));
    }

    #[test]
    fn module_id_rejects_missing_m_prefix() {
        assert!(matches!(
            ModuleId::new("01_core_types"),
            Err(CoreTypeError::IdentifierCharset(_, _))
        ));
    }

    #[test]
    fn module_id_rejects_one_digit() {
        assert!(matches!(
            ModuleId::new("m1_short"),
            Err(CoreTypeError::IdentifierCharset(_, _))
        ));
    }

    #[test]
    fn module_id_rejects_uppercase_tail() {
        assert!(matches!(
            ModuleId::new("m01_Core"),
            Err(CoreTypeError::IdentifierCharset(_, _))
        ));
    }

    #[test]
    fn module_id_rejects_trailing_underscore() {
        assert!(matches!(
            ModuleId::new("m01_core_"),
            Err(CoreTypeError::IdentifierCharset(_, _))
        ));
    }

    #[test]
    fn module_id_serde_roundtrip() {
        let id = ModuleId::new("m35i_codesynthor_v8_bridge").unwrap();
        let json = serde_json::to_string(&id).unwrap();
        let parsed: ModuleId = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, id);
    }

    // ---------- HeatSourceId (6 tests) ----------

    #[test]
    fn heat_source_all_is_ordered() {
        let codes: Vec<_> = HeatSourceId::ALL.iter().map(|h| h.code()).collect();
        assert_eq!(codes, ["HS-001", "HS-002", "HS-003", "HS-004", "HS-005"]);
    }

    #[test]
    fn heat_source_column_suffixes() {
        assert_eq!(HeatSourceId::Hebbian.column_suffix(), "hebbian");
        assert_eq!(HeatSourceId::CrossRalph.column_suffix(), "cross_ralph");
    }

    #[test]
    fn heat_source_from_code_roundtrip() {
        for hs in HeatSourceId::ALL {
            let parsed = HeatSourceId::from_code(hs.code()).unwrap();
            assert_eq!(parsed, hs);
        }
    }

    #[test]
    fn heat_source_from_code_rejects_unknown() {
        assert!(matches!(
            HeatSourceId::from_code("HS-999"),
            Err(CoreTypeError::UnknownHeatSource(_))
        ));
    }

    #[test]
    fn heat_source_display_matches_code() {
        assert_eq!(HeatSourceId::Cascade.to_string(), "HS-002");
    }

    #[test]
    fn heat_source_serde_roundtrip() {
        let hs = HeatSourceId::Resonance;
        let json = serde_json::to_string(&hs).unwrap();
        assert_eq!(json, "\"resonance\"");
        let parsed: HeatSourceId = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, hs);
    }

    // ---------- FlowState (7 tests) ----------

    #[test]
    fn flow_state_happy_path() {
        let f = FlowState::new(0.5).unwrap();
        assert_eq!(f.get(), 0.5);
    }

    #[test]
    fn flow_state_accepts_bounds() {
        assert!(FlowState::new(FlowState::MIN).is_ok());
        assert!(FlowState::new(FlowState::MAX).is_ok());
    }

    #[test]
    fn flow_state_rejects_out_of_range() {
        assert!(matches!(
            FlowState::new(1.1),
            Err(CoreTypeError::OutOfBounds { .. })
        ));
        assert!(matches!(
            FlowState::new(-0.1),
            Err(CoreTypeError::OutOfBounds { .. })
        ));
    }

    #[test]
    fn flow_state_rejects_nan_inf() {
        assert!(matches!(
            FlowState::new(f64::NAN),
            Err(CoreTypeError::NonFinite(_))
        ));
        assert!(matches!(
            FlowState::new(f64::INFINITY),
            Err(CoreTypeError::NonFinite(_))
        ));
    }

    #[test]
    fn flow_state_clamp_from_saturates() {
        assert_eq!(FlowState::clamp_from(2.0).get(), 1.0);
        assert_eq!(FlowState::clamp_from(-5.0).get(), 0.0);
        assert_eq!(FlowState::clamp_from(f64::NAN).get(), 0.0);
    }

    #[test]
    fn flow_state_ord_is_total() {
        let a = FlowState::new(0.1).unwrap();
        let b = FlowState::new(0.9).unwrap();
        assert!(a < b);
        assert_eq!(a.cmp(&b), Ordering::Less);
    }

    #[test]
    fn flow_state_serde_roundtrip() {
        let f = FlowState::new(0.42).unwrap();
        let json = serde_json::to_string(&f).unwrap();
        assert_eq!(json, "0.42");
        let parsed: FlowState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, f);
    }

    // ---------- FitnessDelta (7 tests) ----------

    #[test]
    fn fitness_delta_happy_path() {
        let d = FitnessDelta::new(-0.05).unwrap();
        assert_eq!(d.get(), -0.05);
    }

    #[test]
    fn fitness_delta_accepts_bounds() {
        assert!(FitnessDelta::new(-1.0).is_ok());
        assert!(FitnessDelta::new(1.0).is_ok());
    }

    #[test]
    fn fitness_delta_rejects_out_of_range() {
        assert!(matches!(
            FitnessDelta::new(1.5),
            Err(CoreTypeError::OutOfBounds { .. })
        ));
        assert!(matches!(
            FitnessDelta::new(-1.5),
            Err(CoreTypeError::OutOfBounds { .. })
        ));
    }

    #[test]
    fn fitness_delta_rejects_nan_inf() {
        assert!(matches!(
            FitnessDelta::new(f64::NAN),
            Err(CoreTypeError::NonFinite(_))
        ));
        assert!(matches!(
            FitnessDelta::new(f64::NEG_INFINITY),
            Err(CoreTypeError::NonFinite(_))
        ));
    }

    #[test]
    fn fitness_delta_triggers_rollback_below_threshold() {
        let bad = FitnessDelta::new(-0.05).unwrap();
        let ok = FitnessDelta::new(-0.02).unwrap();
        assert!(bad.triggers_rollback());
        assert!(!ok.triggers_rollback());
    }

    #[test]
    fn fitness_delta_clamp_from_saturates() {
        assert_eq!(FitnessDelta::clamp_from(2.0).get(), 1.0);
        assert_eq!(FitnessDelta::clamp_from(-2.0).get(), -1.0);
        assert_eq!(FitnessDelta::clamp_from(f64::INFINITY).get(), 0.0);
    }

    #[test]
    fn fitness_delta_display_has_sign() {
        assert_eq!(FitnessDelta::new(0.1).unwrap().to_string(), "+0.1000");
        assert_eq!(FitnessDelta::new(-0.1).unwrap().to_string(), "-0.1000");
    }

    // ---------- Severity (6 tests) ----------

    #[test]
    fn severity_ordering_is_canonical() {
        assert!(Severity::Trace < Severity::Debug);
        assert!(Severity::Debug < Severity::Info);
        assert!(Severity::Info < Severity::Warn);
        assert!(Severity::Warn < Severity::Error);
        assert!(Severity::Error < Severity::Critical);
    }

    #[test]
    fn severity_all_len_matches_variants() {
        assert_eq!(Severity::ALL.len(), 6);
    }

    #[test]
    fn severity_as_str_roundtrip() {
        for sev in Severity::ALL {
            let parsed = Severity::parse(sev.as_str()).unwrap();
            assert_eq!(parsed, sev);
        }
    }

    #[test]
    fn severity_parse_accepts_aliases() {
        assert_eq!(Severity::parse("warning").unwrap(), Severity::Warn);
        assert_eq!(Severity::parse("crit").unwrap(), Severity::Critical);
        assert_eq!(Severity::parse("fatal").unwrap(), Severity::Critical);
    }

    #[test]
    fn severity_parse_rejects_unknown() {
        assert!(matches!(
            Severity::parse("panic"),
            Err(CoreTypeError::UnknownSeverity(_))
        ));
    }

    #[test]
    fn severity_serde_roundtrip() {
        let s = Severity::Error;
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(json, "\"error\"");
        let parsed: Severity = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, s);
    }

    // ---------- SnapshotId (6 tests) ----------

    #[test]
    fn snapshot_id_new_is_v7() {
        let id = SnapshotId::new();
        assert_eq!(id.as_uuid().get_version_num(), 7);
    }

    #[test]
    fn snapshot_id_v7_is_time_ordered() {
        let a = SnapshotId::new();
        std::thread::sleep(StdDuration::from_millis(2));
        let b = SnapshotId::new();
        assert!(a < b, "v7 must be time-ordered: a={a} b={b}");
    }

    #[test]
    fn snapshot_id_default_is_fresh() {
        let a = SnapshotId::default();
        std::thread::sleep(StdDuration::from_millis(2));
        let b = SnapshotId::default();
        assert_ne!(a, b);
    }

    #[test]
    fn snapshot_id_from_uuid_roundtrip() {
        let u = Uuid::now_v7();
        let s = SnapshotId::from_uuid(u);
        assert_eq!(s.as_uuid(), u);
    }

    #[test]
    fn snapshot_id_display_matches_uuid() {
        let u = Uuid::now_v7();
        let s = SnapshotId::from_uuid(u);
        assert_eq!(s.to_string(), u.to_string());
    }

    #[test]
    fn snapshot_id_serde_roundtrip() {
        let s = SnapshotId::new();
        let json = serde_json::to_string(&s).unwrap();
        let parsed: SnapshotId = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, s);
    }

    // ---------- CoreTypeError display (3 tests) ----------

    #[test]
    fn core_type_error_identifier_length_display() {
        let e = CoreTypeError::IdentifierLength {
            min: 1,
            max: 64,
            actual: 0,
        };
        assert_eq!(e.to_string(), "identifier length 0 outside [1, 64]");
    }

    #[test]
    fn core_type_error_out_of_bounds_display() {
        let e = CoreTypeError::OutOfBounds {
            min: 0.0,
            max: 1.0,
            actual: 1.5,
        };
        assert_eq!(e.to_string(), "value 1.5 outside [0, 1]");
    }

    #[test]
    fn core_type_error_implements_std_error() {
        fn assert_err<E: std::error::Error>(_: &E) {}
        let e = CoreTypeError::NonFinite(f64::NAN);
        assert_err(&e);
    }
}
