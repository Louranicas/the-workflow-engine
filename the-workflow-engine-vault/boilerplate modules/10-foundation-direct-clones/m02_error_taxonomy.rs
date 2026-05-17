//! `m02_error_taxonomy` — The `SynthexError` enum and layered sub-errors.
//!
//! Every fallible function in the crate returns [`Result<T>`]. The top-level
//! [`SynthexError`] is a thin enum that forwards to sub-errors owned by each
//! layer; this keeps layer concerns decoupled while giving callers a single
//! type to match on.
//!
//! # Sub-errors
//!
//! - [`CoreTypeError`](super::m01_core_types::CoreTypeError) — L1 newtype validation
//! - [`BridgeError`] — outbound HTTP / IPC / CLI
//! - [`DatabaseError`] — `SQLite` + `SpaceTimeDB`
//! - [`RegulationError`] — L4 PID + heat source issues
//! - [`ClassificationError`] — L5 NAM classifier
//! - [`WatcherError`] — L8 Watcher pipeline
//! - [`EmberError`] — 7-trait gate rejection
//! - [`PersistenceError`] — L3 snapshot + migration
//! - [`SchemaError`] — schema drift / version mismatch
//! - [`ConfigError`] — m03 configuration parsing/validation
//!
//! # Design
//!
//! - Every sub-error is `thiserror`-derived and `#[non_exhaustive]` so we can
//!   add variants without a major version bump.
//! - `#[from]` bubbles sub-errors through `?` without ceremony.
//! - Error codes live in the banded `E1xxx..E9xxx` space declared in
//!   [`ErrorCode`]. Tests lock the mapping so downstream dashboards stay stable.
//! - All variants keep the underlying cause chain via `std::error::Error::source()`.

#![allow(clippy::module_name_repetitions)]

use std::io;

use thiserror::Error;

pub use super::m01_core_types::CoreTypeError;

/// Crate-wide result alias.
pub type Result<T> = std::result::Result<T, SynthexError>;

// ---------------------------------------------------------------------------
// Error code bands
// ---------------------------------------------------------------------------

/// Stable numeric code for dashboard/alerting pipelines.
///
/// Codes are banded by owning layer so the band alone identifies the subsystem:
///
/// | Range | Owner |
/// |-------|-------|
/// | 1000–1999 | L1 Foundation (core types, config) |
/// | 2000–2999 | L3 Persistence (SQLite, STDB writer, snapshot) |
/// | 3000–3999 | L2 Ingest (subscribers, pollers, drift) |
/// | 4000–4999 | L4 Regulation (PID, heat, thermal) |
/// | 5000–5999 | L5 Classification (NAM) |
/// | 6000–6999 | L6 Action (bridges, circuit, retry) |
/// | 7000–7999 | L7 Memory + HMX |
/// | 8000–8999 | L8 Watcher (incl. Ember gate) |
/// | 9000–9999 | Cross-cutting (schema drift, top-level I/O) |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ErrorCode(u16);

impl ErrorCode {
    /// Raw numeric value.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }

    /// Owning layer / band label (`"L1"`, `"L3"`, …, or `"X"` for cross-cutting).
    #[must_use]
    pub const fn band(self) -> &'static str {
        match self.0 {
            1000..=1999 => "L1",
            2000..=2999 => "L3",
            3000..=3999 => "L2",
            4000..=4999 => "L4",
            5000..=5999 => "L5",
            6000..=6999 => "L6",
            7000..=7999 => "L7",
            8000..=8999 => "L8",
            _ => "X",
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "E{:04}", self.0)
    }
}

// ---------------------------------------------------------------------------
// Top-level error
// ---------------------------------------------------------------------------

/// Crate-wide error union.
///
/// Fallible APIs should return `Result<T>`; callers that want to react to a
/// particular failure mode should match on the variant, not on the error
/// string.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SynthexError {
    /// L1 newtype validation rejection.
    #[error(transparent)]
    CoreType(#[from] CoreTypeError),

    /// m03 configuration parsing or validation failure.
    #[error(transparent)]
    Config(#[from] ConfigError),

    /// Outbound HTTP / IPC / CLI failure (L6 bridges).
    #[error(transparent)]
    Bridge(#[from] BridgeError),

    /// `SQLite` / `SpaceTimeDB` error (L3).
    #[error(transparent)]
    Database(#[from] DatabaseError),

    /// L2 ingest error.
    #[error(transparent)]
    Ingest(#[from] IngestError),

    /// L4 regulation / PID / heat source error.
    #[error(transparent)]
    Regulation(#[from] RegulationError),

    /// L5 NAM classifier error.
    #[error(transparent)]
    Classification(#[from] ClassificationError),

    /// L7 memory / HMX error.
    #[error(transparent)]
    Memory(#[from] MemoryError),

    /// L8 Watcher pipeline error (Observer/Critic/Verifier/Innovator/Proposer).
    #[error(transparent)]
    Watcher(#[from] WatcherError),

    /// Ember 7-trait gate rejection (L8).
    #[error(transparent)]
    Ember(#[from] EmberError),

    /// L3 snapshot / migration / persistence error.
    #[error(transparent)]
    Persistence(#[from] PersistenceError),

    /// Schema drift / version mismatch (cross-cutting).
    #[error(transparent)]
    Schema(#[from] SchemaError),

    /// Uncategorised `std::io` failure (cross-cutting).
    #[error("io error: {0}")]
    Io(#[from] io::Error),
}

impl SynthexError {
    /// Stable numeric code for dashboards.
    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        match self {
            Self::CoreType(_) => ErrorCode(1001),
            Self::Config(_) => ErrorCode(1101),
            Self::Bridge(_) => ErrorCode(6001),
            Self::Database(_) => ErrorCode(2001),
            Self::Ingest(_) => ErrorCode(3001),
            Self::Regulation(_) => ErrorCode(4001),
            Self::Classification(_) => ErrorCode(5001),
            Self::Memory(_) => ErrorCode(7001),
            Self::Watcher(_) => ErrorCode(8001),
            Self::Ember(_) => ErrorCode(8101),
            Self::Persistence(_) => ErrorCode(2101),
            Self::Schema(_) => ErrorCode(9001),
            Self::Io(_) => ErrorCode(9901),
        }
    }

    /// `true` if the originating class is cross-cutting (band `"X"`).
    #[must_use]
    pub const fn is_cross_cutting(&self) -> bool {
        matches!(self, Self::Schema(_) | Self::Io(_))
    }
}

// ---------------------------------------------------------------------------
// Sub-errors
// ---------------------------------------------------------------------------

/// m03 configuration errors.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ConfigError {
    /// TOML parse failure.
    #[error("config: malformed TOML at {location}: {source}")]
    Toml {
        /// Source file path or `"<env>"`.
        location: String,
        /// Inner parser error.
        #[source]
        source: toml::de::Error,
    },
    /// Semantic validation failed (e.g. PID `Kp < 0`).
    #[error("config: invalid value for {field}: {reason}")]
    InvalidValue {
        /// Dotted TOML path (e.g. `regulation.pid.kp`).
        field: &'static str,
        /// Human-readable reason.
        reason: String,
    },
    /// Required field missing with no default.
    #[error("config: missing required field {0:?}")]
    Missing(&'static str),
    /// File I/O failure loading a config path.
    #[error("config: io error loading {path:?}: {source}")]
    Io {
        /// Attempted path.
        path: String,
        /// Inner io error.
        #[source]
        source: io::Error,
    },
}

/// L6 bridge (HTTP / IPC / CLI) errors.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum BridgeError {
    /// HTTP request failed before we saw a response.
    #[error("bridge {bridge}: http request failed: {message}")]
    HttpRequest {
        /// Bridge identifier (e.g. `"m35g_orac_bridge"`).
        bridge: &'static str,
        /// Summary of the failure.
        message: String,
    },
    /// HTTP response status was not in the expected set.
    #[error("bridge {bridge}: unexpected status {status}")]
    UnexpectedStatus {
        /// Bridge identifier.
        bridge: &'static str,
        /// The status code we received.
        status: u16,
    },
    /// Response body did not match the expected schema.
    #[error("bridge {bridge}: invalid json in response: {message}")]
    InvalidJson {
        /// Bridge identifier.
        bridge: &'static str,
        /// Serde diagnostic.
        message: String,
    },
    /// Circuit breaker is currently open — call was not attempted.
    #[error("bridge {bridge}: circuit breaker open")]
    CircuitOpen {
        /// Bridge identifier.
        bridge: &'static str,
    },
    /// Bridge timed out.
    #[error("bridge {bridge}: timed out after {ms} ms")]
    Timeout {
        /// Bridge identifier.
        bridge: &'static str,
        /// Elapsed milliseconds before timeout fired.
        ms: u64,
    },
    /// CLI sub-process binary not found on `PATH` (AP33).
    #[error("bridge {bridge}: CLI binary not found on PATH")]
    CliNotFound {
        /// Bridge identifier.
        bridge: &'static str,
    },
    /// CLI sub-process exited non-zero.
    #[error("bridge {bridge}: CLI exited {code}: {stderr}")]
    CliFailure {
        /// Bridge identifier.
        bridge: &'static str,
        /// Exit code.
        code: i32,
        /// Captured stderr tail.
        stderr: String,
    },
    /// Retry scheduler exhausted its attempt budget (AP01 regression
    /// fix — surfaces loudly instead of returning a default).
    #[error("bridge {bridge}: max retries ({attempts}) exceeded")]
    MaxRetriesExceeded {
        /// Bridge identifier.
        bridge: &'static str,
        /// Number of attempts already made.
        attempts: u32,
    },
    /// Bridge manager has no registered bridge with that id.
    #[error("bridge {bridge}: not registered with bridge manager")]
    NotRegistered {
        /// Bridge identifier the caller asked for.
        bridge: &'static str,
    },
}

/// L3 persistence — database-layer errors.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DatabaseError {
    /// Query failed to execute.
    #[error("db {db}: query failed: {message}")]
    Query {
        /// Logical database name (e.g. `"gradient_snapshot"`).
        db: &'static str,
        /// Inner rusqlite / STDB error message.
        message: String,
    },
    /// Connection / pool exhausted or unavailable.
    #[error("db {db}: connection failure: {message}")]
    Connection {
        /// Logical database name.
        db: &'static str,
        /// Cause.
        message: String,
    },
    /// Migration application failed.
    #[error("db {db}: migration {version} failed: {message}")]
    Migration {
        /// Logical database name.
        db: &'static str,
        /// Migration version string (e.g. `"013"`).
        version: String,
        /// Cause.
        message: String,
    },
}

/// L2 ingest errors.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum IngestError {
    /// STDB subscription dropped or returned a `tombstone`.
    #[error("ingest: STDB subscription dropped table {table:?}")]
    StdbSubscriptionDropped {
        /// Table name.
        table: String,
    },
    /// HTTP poller could not decode payload.
    #[error("ingest: HTTP poll of {source_label} yielded unparseable body: {message}")]
    PollDecode {
        /// Source label.
        source_label: &'static str,
        /// Parse diagnostic.
        message: String,
    },
    /// Schema drift detected at runtime.
    #[error("ingest: schema drift in {source_label}: {field} type changed")]
    SchemaDrift {
        /// Source label.
        source_label: &'static str,
        /// Field that changed.
        field: String,
    },
    /// Outbound channel at capacity — caller should apply back-pressure.
    #[error("ingest: channel full at {source_label}")]
    ChannelFull {
        /// Source label.
        source_label: &'static str,
    },
}

/// L4 regulation (PID + heat sources).
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum RegulationError {
    /// Heat source update produced a non-finite number.
    #[error("regulation: heat source {id} produced non-finite value")]
    NonFiniteHeat {
        /// `HeatSourceId` code (e.g. `"HS-001"`).
        id: &'static str,
    },
    /// PID output was clamped out of a physically plausible range.
    #[error("regulation: PID output {value} clamped out of [{min}, {max}]")]
    PidClamped {
        /// Raw output before clamping.
        value: f64,
        /// Lower bound used.
        min: f64,
        /// Upper bound used.
        max: f64,
    },
    /// Ordering anomaly — same tick read twice with contradictory state.
    #[error("regulation: tick ordering anomaly at gen {generation}")]
    TickOrdering {
        /// RALPH generation on which the anomaly was seen.
        generation: u64,
    },
    /// PID input (measurement or `dt`) was not finite.
    #[error("regulation: PID input {field} value {value} is not finite")]
    NonFiniteInput {
        /// Field name (`"measurement"` / `"dt"` / `"target"`).
        field: &'static str,
        /// Offending value as observed.
        value: f64,
    },
    /// PID gain adjustment landed outside the `±tune_band` envelope.
    ///
    /// Watcher auto-deploy (Tier 5+) may only tune `Kp`/`Ki`/`Kd` within the
    /// bounded envelope recorded at construction time. A violation is a
    /// hard reject at the controller boundary — the proposal is kicked back
    /// to the Critic before it can reach PBFT.
    #[error(
        "regulation: gain {axis} adjustment {value} outside tune band [{min}, {max}]"
    )]
    TuneBandViolation {
        /// Which gain was being adjusted (`"kp"` / `"ki"` / `"kd"`).
        axis: &'static str,
        /// Requested value.
        value: f64,
        /// Lower bound of the envelope.
        min: f64,
        /// Upper bound of the envelope.
        max: f64,
    },
    /// L5 classification boundary error surfaced at the L4 regulation layer.
    #[error("regulation: classification boundary error: {detail}")]
    ClassificationBoundary {
        /// Static label describing the classification failure mode.
        detail: &'static str,
    },
    /// A PID config parameter failed validation at construction.
    #[error(
        "regulation: invalid PID config field {field} value {value} ({reason})"
    )]
    InvalidPidConfig {
        /// Field name (`"kp"` / `"min_dt"` / `"output_min"` …).
        field: &'static str,
        /// Offending value.
        value: f64,
        /// Human-readable reason (e.g. `"negative"`, `"min >= max"`).
        reason: &'static str,
    },
}

/// L5 classification.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ClassificationError {
    /// NAM class string unknown.
    #[error("classification: unknown NAM class {0:?}")]
    UnknownClass(String),
    /// Confidence score outside `[0.0, 1.0]`.
    #[error("classification: confidence {0} outside [0, 1]")]
    ConfidenceOutOfRange(f64),
    /// Feature vector dimension mismatch.
    #[error("classification: feature vector dim {got} != expected {expected}")]
    FeatureDimMismatch {
        /// Observed length.
        got: usize,
        /// Required length.
        expected: usize,
    },
}

/// L7 memory / HMX.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum MemoryError {
    /// Tier promotion target not reachable.
    #[error("memory: tier promotion from {from} to {to} rejected")]
    TierPromotion {
        /// Source tier label.
        from: &'static str,
        /// Destination tier label.
        to: &'static str,
    },
    /// HMX cluster missing during hydrate.
    #[error("memory: HMX cluster {cluster:?} missing")]
    HmxMissingCluster {
        /// Cluster name.
        cluster: String,
    },
    /// POVM namespace collision (AP09).
    #[error("memory: POVM namespace collision on pathway {0:?}")]
    PovmCollision(String),
    /// Caller supplied an invalid value for a tier operation (bad weight,
    /// empty domain, malformed pathway id, etc.).
    #[error("memory: invalid input for {field}: {reason}")]
    InvalidInput {
        /// Offending field name.
        field: &'static str,
        /// Short machine-grep-able reason (e.g. `"non_finite"`, `"empty"`).
        reason: &'static str,
    },
    /// Downstream sink (POVM bridge, Obsidian writer, RM bridge) reported a
    /// failure while the tier was flushing.
    #[error("memory: sink failure: {reason}")]
    SinkFailure {
        /// Human-readable failure cause.
        reason: String,
    },
}

/// L8 Watcher pipeline.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum WatcherError {
    /// Observer failed to collect required signals.
    #[error("watcher: observer incomplete — missing {signal}")]
    ObserverIncomplete {
        /// Missing signal name.
        signal: &'static str,
    },
    /// Critic / Innovator / Verifier exceeded Opus daily budget.
    #[error("watcher: opus budget exceeded (used ${used}, cap ${cap})")]
    BudgetExceeded {
        /// Dollars consumed today.
        used: u32,
        /// Daily cap.
        cap: u32,
    },
    /// Proposer self-rejected via Ember gate (propagated, not raised here).
    #[error("watcher: proposer self-rejected via ember")]
    EmberSelfRejected,
    /// PBFT quorum could not be reached (`q=27/n=41`).
    #[error("watcher: pbft quorum not reached ({votes}/{needed})")]
    PbftQuorum {
        /// Yes-votes counted.
        votes: u16,
        /// Threshold required.
        needed: u16,
    },
    /// Proposer tried to modify `src/m8_watcher/*` (AP27).
    #[error("watcher: self-modification boundary violated on path {0:?}")]
    SelfModificationBoundary(String),
}

/// Ember 7-trait gate rejection.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum EmberError {
    /// Trait check failed with a human-readable reason.
    #[error("ember: trait {trait_name} rejected proposal: {reason}")]
    TraitRejected {
        /// One of `Equanimity`, `Curiosity`, `Diligence`, `Honesty`,
        /// `Investment`, `Humility`, `Warmth`.
        trait_name: &'static str,
        /// Reason the trait check failed.
        reason: String,
    },
    /// Daily cadence gate not satisfied (Q1 — 30 days OR 100 proposals).
    #[error("ember: cadence gate closed ({reason})")]
    CadenceClosed {
        /// Which half of the OR gate tripped.
        reason: &'static str,
    },
}

/// L3 snapshot / migration.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum PersistenceError {
    /// Snapshot capture failed before write.
    #[error("persistence: snapshot capture failed: {message}")]
    SnapshotCapture {
        /// Cause.
        message: String,
    },
    /// Rollback target not found.
    #[error("persistence: rollback target {id:?} not found")]
    RollbackNotFound {
        /// `SnapshotId` string.
        id: String,
    },
    /// Rollback exceeded 30-second target (ADR-008 / P0-4).
    #[error("persistence: rollback exceeded 30s budget (took {secs} s)")]
    RollbackBudgetExceeded {
        /// Elapsed seconds.
        secs: u64,
    },
}

/// Schema drift / version mismatch (cross-cutting).
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SchemaError {
    /// Schema version on disk does not match the version compiled into the binary.
    #[error("schema: {db} version mismatch (binary={binary}, disk={disk})")]
    VersionMismatch {
        /// Logical database name.
        db: &'static str,
        /// Version the binary expects.
        binary: String,
        /// Version actually present.
        disk: String,
    },
    /// Required view or table absent.
    #[error("schema: {db} missing object {object:?}")]
    MissingObject {
        /// Logical database name.
        db: &'static str,
        /// Object name.
        object: String,
    },
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use std::error::Error as StdError;

    // ---------- ErrorCode (6 tests) ----------

    #[test]
    fn error_code_display_pads_four_digits() {
        assert_eq!(ErrorCode(1001).to_string(), "E1001");
        assert_eq!(ErrorCode(42).to_string(), "E0042");
    }

    #[test]
    fn error_code_band_l1() {
        assert_eq!(ErrorCode(1001).band(), "L1");
        assert_eq!(ErrorCode(1999).band(), "L1");
    }

    #[test]
    fn error_code_band_l3_persistence() {
        assert_eq!(ErrorCode(2001).band(), "L3");
    }

    #[test]
    fn error_code_band_l8_watcher() {
        assert_eq!(ErrorCode(8001).band(), "L8");
    }

    #[test]
    fn error_code_band_cross_cutting_fallback() {
        assert_eq!(ErrorCode(9001).band(), "X");
        assert_eq!(ErrorCode(9901).band(), "X");
    }

    #[test]
    fn error_code_get_roundtrip() {
        assert_eq!(ErrorCode(4242).get(), 4242);
    }

    // ---------- SynthexError::code mapping (14 tests) ----------

    #[test]
    fn code_core_type() {
        let e: SynthexError = CoreTypeError::NonFinite(f64::NAN).into();
        assert_eq!(e.code().get(), 1001);
        assert_eq!(e.code().band(), "L1");
    }

    #[test]
    fn code_config() {
        let e: SynthexError = ConfigError::Missing("foo").into();
        assert_eq!(e.code().get(), 1101);
        assert_eq!(e.code().band(), "L1");
    }

    #[test]
    fn code_bridge() {
        let e: SynthexError = BridgeError::CircuitOpen { bridge: "m35g" }.into();
        assert_eq!(e.code().get(), 6001);
        assert_eq!(e.code().band(), "L6");
    }

    #[test]
    fn code_database() {
        let e: SynthexError = DatabaseError::Connection {
            db: "gradient_snapshot",
            message: "pool exhausted".into(),
        }
        .into();
        assert_eq!(e.code().get(), 2001);
        assert_eq!(e.code().band(), "L3");
    }

    #[test]
    fn code_ingest() {
        let e: SynthexError = IngestError::StdbSubscriptionDropped {
            table: "bridge_status".into(),
        }
        .into();
        assert_eq!(e.code().band(), "L2");
    }

    #[test]
    fn display_ingest_poll_decode() {
        let e = IngestError::PollDecode {
            source_label: "orac",
            message: "missing field".into(),
        };
        let s = e.to_string();
        assert!(s.contains("orac"));
        assert!(s.contains("missing field"));
    }

    #[test]
    fn code_regulation() {
        let e: SynthexError = RegulationError::NonFiniteHeat { id: "HS-001" }.into();
        assert_eq!(e.code().band(), "L4");
    }

    #[test]
    fn code_classification() {
        let e: SynthexError = ClassificationError::UnknownClass("foo".into()).into();
        assert_eq!(e.code().band(), "L5");
    }

    #[test]
    fn code_memory() {
        let e: SynthexError = MemoryError::HmxMissingCluster {
            cluster: "c1".into(),
        }
        .into();
        assert_eq!(e.code().band(), "L7");
    }

    #[test]
    fn code_watcher() {
        let e: SynthexError = WatcherError::EmberSelfRejected.into();
        assert_eq!(e.code().band(), "L8");
    }

    #[test]
    fn code_ember() {
        let e: SynthexError = EmberError::CadenceClosed {
            reason: "too soon",
        }
        .into();
        assert_eq!(e.code().band(), "L8");
        assert_eq!(e.code().get(), 8101);
    }

    #[test]
    fn code_persistence() {
        let e: SynthexError = PersistenceError::RollbackBudgetExceeded { secs: 42 }.into();
        assert_eq!(e.code().band(), "L3");
    }

    #[test]
    fn code_schema() {
        let e: SynthexError = SchemaError::MissingObject {
            db: "x",
            object: "v_foo".into(),
        }
        .into();
        assert_eq!(e.code().band(), "X");
        assert!(e.is_cross_cutting());
    }

    #[test]
    fn code_io() {
        let e: SynthexError = io::Error::new(io::ErrorKind::NotFound, "nope").into();
        assert_eq!(e.code().band(), "X");
        assert!(e.is_cross_cutting());
    }

    #[test]
    fn is_cross_cutting_only_for_schema_and_io() {
        let bridge: SynthexError = BridgeError::CircuitOpen { bridge: "m35g" }.into();
        assert!(!bridge.is_cross_cutting());
    }

    // ---------- From impls / ? operator ergonomics (6 tests) ----------

    #[test]
    fn question_mark_lifts_core_type_error() {
        fn inner() -> Result<()> {
            let _ = super::super::m01_core_types::ServiceId::new("")?;
            Ok(())
        }
        let err = inner().unwrap_err();
        assert!(matches!(err, SynthexError::CoreType(_)));
    }

    #[test]
    fn from_bridge_error() {
        let e: SynthexError = BridgeError::Timeout {
            bridge: "x",
            ms: 1000,
        }
        .into();
        assert!(matches!(e, SynthexError::Bridge(_)));
    }

    #[test]
    fn from_config_error() {
        let e: SynthexError = ConfigError::InvalidValue {
            field: "regulation.pid.kp",
            reason: "must be >= 0".into(),
        }
        .into();
        assert!(matches!(e, SynthexError::Config(_)));
    }

    #[test]
    fn from_ember_error() {
        let e: SynthexError = EmberError::TraitRejected {
            trait_name: "Honesty",
            reason: "overclaims".into(),
        }
        .into();
        assert!(matches!(e, SynthexError::Ember(_)));
    }

    #[test]
    fn from_persistence_error() {
        let e: SynthexError = PersistenceError::SnapshotCapture {
            message: "db locked".into(),
        }
        .into();
        assert!(matches!(e, SynthexError::Persistence(_)));
    }

    #[test]
    fn from_io_error() {
        let ioe = io::Error::new(io::ErrorKind::PermissionDenied, "nope");
        let e: SynthexError = ioe.into();
        assert!(matches!(e, SynthexError::Io(_)));
    }

    // ---------- Display messages (10 tests) ----------

    #[test]
    fn display_bridge_http_request() {
        let e = BridgeError::HttpRequest {
            bridge: "m35g_orac_bridge",
            message: "connection refused".into(),
        };
        let s = e.to_string();
        assert!(s.contains("m35g_orac_bridge"));
        assert!(s.contains("connection refused"));
    }

    #[test]
    fn display_bridge_circuit_open() {
        let e = BridgeError::CircuitOpen { bridge: "m35d" };
        assert_eq!(e.to_string(), "bridge m35d: circuit breaker open");
    }

    #[test]
    fn display_bridge_timeout() {
        let e = BridgeError::Timeout {
            bridge: "m35i",
            ms: 10_000,
        };
        assert!(e.to_string().contains("10000 ms"));
    }

    #[test]
    fn display_bridge_cli_not_found() {
        let e = BridgeError::CliNotFound { bridge: "m35i" };
        assert!(e.to_string().contains("CLI binary not found"));
    }

    #[test]
    fn display_regulation_non_finite_heat() {
        let e = RegulationError::NonFiniteHeat { id: "HS-001" };
        assert!(e.to_string().contains("HS-001"));
    }

    #[test]
    fn display_pid_clamped_shows_bounds() {
        let e = RegulationError::PidClamped {
            value: 2.5,
            min: -1.0,
            max: 1.0,
        };
        let s = e.to_string();
        assert!(s.contains("2.5"));
        assert!(s.contains("-1"));
        assert!(s.contains("1"));
    }

    #[test]
    fn display_classification_feature_dim_mismatch() {
        let e = ClassificationError::FeatureDimMismatch {
            got: 10,
            expected: 11,
        };
        let s = e.to_string();
        assert!(s.contains("10"));
        assert!(s.contains("11"));
    }

    #[test]
    fn display_memory_povm_collision() {
        let e = MemoryError::PovmCollision("synthex_v2_foo::bar".into());
        assert!(e.to_string().contains("synthex_v2_foo::bar"));
    }

    #[test]
    fn display_watcher_pbft_quorum() {
        let e = WatcherError::PbftQuorum {
            votes: 24,
            needed: 27,
        };
        let s = e.to_string();
        assert!(s.contains("24/27"));
    }

    #[test]
    fn display_schema_version_mismatch() {
        let e = SchemaError::VersionMismatch {
            db: "gradient_snapshot",
            binary: "1.2".into(),
            disk: "1.1".into(),
        };
        let s = e.to_string();
        assert!(s.contains("binary=1.2"));
        assert!(s.contains("disk=1.1"));
    }

    // ---------- source() chain (4 tests) ----------

    #[test]
    fn config_toml_preserves_source() {
        let toml_err = toml::from_str::<toml::Value>("= not toml").unwrap_err();
        let e = ConfigError::Toml {
            location: "<test>".into(),
            source: toml_err,
        };
        assert!(e.source().is_some());
    }

    #[test]
    fn config_io_preserves_source() {
        let ioe = io::Error::new(io::ErrorKind::NotFound, "nope");
        let e = ConfigError::Io {
            path: "/etc/x".into(),
            source: ioe,
        };
        assert!(e.source().is_some());
    }

    #[test]
    fn synthex_error_io_variant_has_source() {
        let ioe = io::Error::new(io::ErrorKind::Other, "bang");
        let e: SynthexError = ioe.into();
        // `#[error("io error: {0}")]` uses Display, so the io::Error becomes
        // the source through #[from] automatically.
        assert!(matches!(e, SynthexError::Io(_)));
    }

    #[test]
    fn synthex_error_chains_through_from() {
        let e: SynthexError = CoreTypeError::NonFinite(f64::INFINITY).into();
        // The wrapping SynthexError via #[error(transparent)] delegates to the
        // inner Display, so the outer to_string contains the inner message.
        assert!(e.to_string().contains("finite"));
    }

    // ---------- Std trait conformance (4 tests) ----------

    #[test]
    fn synthex_error_implements_std_error() {
        fn takes_err<E: StdError>(_: &E) {}
        let e: SynthexError = CoreTypeError::NonFinite(f64::NAN).into();
        takes_err(&e);
    }

    #[test]
    fn synthex_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SynthexError>();
    }

    #[test]
    fn every_sub_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ConfigError>();
        assert_send_sync::<BridgeError>();
        assert_send_sync::<DatabaseError>();
        assert_send_sync::<IngestError>();
        assert_send_sync::<RegulationError>();
        assert_send_sync::<ClassificationError>();
        assert_send_sync::<MemoryError>();
        assert_send_sync::<WatcherError>();
        assert_send_sync::<EmberError>();
        assert_send_sync::<PersistenceError>();
        assert_send_sync::<SchemaError>();
    }

    #[test]
    fn result_alias_matches_std_result() {
        fn returns() -> Result<u32> {
            Ok(42)
        }
        assert_eq!(returns().unwrap(), 42);
    }

    // ---------- Non-exhaustive discipline (2 tests — catch-all arms) ----------

    #[test]
    fn match_on_synthex_error_accepts_default_arm() {
        let e: SynthexError = BridgeError::CircuitOpen { bridge: "x" }.into();
        let label = match e {
            SynthexError::Bridge(_) => "bridge",
            _ => "other",
        };
        assert_eq!(label, "bridge");
    }

    #[test]
    fn match_on_bridge_error_accepts_default_arm() {
        let e = BridgeError::Timeout { bridge: "x", ms: 1 };
        let label = match e {
            BridgeError::Timeout { .. } => "timeout",
            _ => "other",
        };
        assert_eq!(label, "timeout");
    }

    // ---------- Code band stability lock-in (4 tests) ----------
    //
    // These are load-bearing — downstream alerting keys off these bands. If
    // you change a code, you MUST update the dashboard Grafana panel at the
    // same time (see decision_s105_error_codes.md once written).

    #[test]
    fn error_code_band_l1_covers_config_and_coretype() {
        let ct: SynthexError = CoreTypeError::NonFinite(0.0).into();
        let cfg: SynthexError = ConfigError::Missing("x").into();
        assert_eq!(ct.code().band(), "L1");
        assert_eq!(cfg.code().band(), "L1");
    }

    #[test]
    fn error_code_band_l3_covers_database_and_persistence() {
        let db: SynthexError = DatabaseError::Query {
            db: "x",
            message: "y".into(),
        }
        .into();
        let p: SynthexError = PersistenceError::RollbackNotFound { id: "z".into() }.into();
        assert_eq!(db.code().band(), "L3");
        assert_eq!(p.code().band(), "L3");
    }

    #[test]
    fn error_code_band_l8_covers_watcher_and_ember() {
        let w: SynthexError = WatcherError::EmberSelfRejected.into();
        let em: SynthexError = EmberError::CadenceClosed { reason: "x" }.into();
        assert_eq!(w.code().band(), "L8");
        assert_eq!(em.code().band(), "L8");
    }

    #[test]
    fn error_code_band_x_covers_schema_and_io() {
        let sc: SynthexError = SchemaError::MissingObject {
            db: "x",
            object: "y".into(),
        }
        .into();
        let io_err: SynthexError = io::Error::other("x").into();
        assert_eq!(sc.code().band(), "X");
        assert_eq!(io_err.code().band(), "X");
    }
}
