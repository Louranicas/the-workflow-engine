//! `wf-crystallise` pipeline driver + CLI parser.
//!
//! Drives the m1-m23 + m40-m42 stages: read atuin shell history (m1) and
//! the injection.db causal-chain ledger (m3), correlate cascades (m4),
//! record a workflow run (m7), compute a lift snapshot (m14), mine
//! sequential patterns (m20), enumerate variants (m21), compose proposals
//! (m23), and write each proposal as one JSON line to the proposals
//! output file. Live-service stages (m2 stcortex, m13/m40/m42) are
//! attempted only when `--offline` is not set, and degrade gracefully.
//!
//! The pipeline always runs end-to-end to completion: a down live service
//! is logged and skipped, never a panic.

use std::fmt;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::m1_atuin_consumer::{
    open_readonly as open_atuin_readonly, AtuinConsumerConfig, AtuinConsumerError, AtuinHistoryRow,
};
use crate::m3_injection_db_consumer::{
    open_readonly as open_injection_readonly, InjectionDbConfig, InjectionDbError,
};
use crate::m4_cascade::cluster_id::fnv1a_64;
use crate::m4_cascade::{AtuinStep, CascadeCorrelator, CascadeCorrelatorConfig};
use crate::m7_workflow_runs::{
    close_run, find_open, insert_run, merge_observation, open_database as open_runs_database,
    ClusterBObservation, Outcome, WorkflowError,
};
use crate::m14_lift::{LiftAggregator, LiftAggregatorConfig, LiftError};
use crate::m20_prefixspan::{mine_sequences, MaxGap, MinSupport, MinerError, StepToken};
use crate::m21_variant_builder::{build_variants, WorkflowVariant};
use crate::m22_kmeans::{
    extract_variant_features, kmeans, recommended_k_for_variant_count, KMeansConfig,
};
use crate::m15_pressure::{PressureRegister, PressureRegisterConfig};
use crate::m23_proposer::compose_proposals_with_pressure;

/// Default proposals JSONL output path.
pub const DEFAULT_PROPOSALS_OUT: &str = "./proposals.jsonl";
/// Default workflow-runs SQLite path (created if absent).
pub const DEFAULT_RUNS_DB: &str = "./workflow_runs.db";
/// Default `min_support` for the m20 miner.
pub const DEFAULT_MIN_SUPPORT: usize = 3;
/// Default `max_gap` for the m20 miner.
pub const DEFAULT_MAX_GAP: usize = 5;
/// Maximum mined pattern length handed to m20.
pub const DEFAULT_MAX_PATTERN_LENGTH: usize = 8;

// ─── output format ──────────────────────────────────────────────────────

/// Report rendering format selected via `--format`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    /// Human-readable multi-line text.
    Text,
    /// Single-line JSON object.
    Json,
}

impl ReportFormat {
    /// Parse a `--format` argument value.
    fn parse(raw: &str) -> Result<Self, ArgError> {
        match raw {
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            other => Err(ArgError::BadValue {
                flag: "--format",
                value: other.to_owned(),
            }),
        }
    }
}

// ─── config ─────────────────────────────────────────────────────────────

/// Parsed `wf-crystallise` CLI options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// atuin history SQLite path.
    pub atuin_db: PathBuf,
    /// habitat injection.db SQLite path.
    pub injection_db: PathBuf,
    /// workflow-runs SQLite path (created if absent).
    pub runs_db: PathBuf,
    /// proposals JSONL output path.
    pub proposals_out: PathBuf,
    /// m20 minimum support.
    pub min_support: usize,
    /// m20 maximum gap.
    pub max_gap: usize,
    /// Skip all live-service stages when `true`.
    pub offline: bool,
    /// Report rendering format.
    pub format: ReportFormat,
    /// `--help` requested.
    pub show_help: bool,
    /// `--version` requested.
    pub show_version: bool,
}

impl Config {
    /// Resolve the default atuin DB path (`$HOME/.local/share/atuin/history.db`).
    fn default_atuin_db() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_owned());
        PathBuf::from(format!("{home}/.local/share/atuin/history.db"))
    }

    /// Resolve the default injection.db path
    /// (`$HOME/.local/share/habitat/injection.db`).
    fn default_injection_db() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_owned());
        PathBuf::from(format!("{home}/.local/share/habitat/injection.db"))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            atuin_db: Self::default_atuin_db(),
            injection_db: Self::default_injection_db(),
            runs_db: PathBuf::from(DEFAULT_RUNS_DB),
            proposals_out: PathBuf::from(DEFAULT_PROPOSALS_OUT),
            min_support: DEFAULT_MIN_SUPPORT,
            max_gap: DEFAULT_MAX_GAP,
            offline: false,
            format: ReportFormat::Text,
            show_help: false,
            show_version: false,
        }
    }
}

// ─── arg parsing ────────────────────────────────────────────────────────

/// CLI argument parsing failure.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum ArgError {
    /// An unrecognised flag was supplied.
    #[error("unknown flag: {0}")]
    UnknownFlag(String),
    /// A flag that requires a value was given none.
    #[error("flag {0} requires a value")]
    MissingValue(&'static str),
    /// A flag value could not be parsed.
    #[error("flag {flag} got invalid value {value:?}")]
    BadValue {
        /// The offending flag.
        flag: &'static str,
        /// The raw value supplied.
        value: String,
    },
    /// A positional argument was supplied (none are accepted).
    #[error("unexpected positional argument: {0}")]
    UnexpectedPositional(String),
}

/// Help text for `--help`.
pub const HELP_TEXT: &str = "\
wf-crystallise — observe habitat workflows and crystallise proposals

USAGE:
    wf-crystallise [OPTIONS]

OPTIONS:
    --atuin-db <PATH>        atuin history.db (default: $HOME/.local/share/atuin/history.db)
    --injection-db <PATH>    habitat injection.db (default: $HOME/.local/share/habitat/injection.db)
    --runs-db <PATH>         workflow_runs.db, created if absent (default: ./workflow_runs.db)
    --proposals-out <PATH>   JSONL proposal output (default: ./proposals.jsonl)
    --min-support <N>        m20 minimum support (default: 3)
    --max-gap <N>            m20 maximum gap (default: 5)
    --offline                skip all live-service stages (stcortex/ORAC/synthex)
    --format <text|json>     report format (default: text)
    --help                   print this help and exit
    --version                print version and exit
";

/// Parse a `usize` flag value with a typed error.
fn parse_usize(flag: &'static str, raw: &str) -> Result<usize, ArgError> {
    raw.parse::<usize>().map_err(|_| ArgError::BadValue {
        flag,
        value: raw.to_owned(),
    })
}

/// Parse `wf-crystallise` CLI arguments.
///
/// `args` is the argument slice *excluding* `argv[0]`.
///
/// # Errors
///
/// Returns [`ArgError`] for an unknown flag, a missing flag value, an
/// unparseable value, or an unexpected positional argument.
pub fn parse_args(args: &[String]) -> Result<Config, ArgError> {
    let mut config = Config::default();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--help" | "-h" => config.show_help = true,
            "--version" | "-V" => config.show_version = true,
            "--offline" => config.offline = true,
            "--atuin-db" => {
                let v = iter.next().ok_or(ArgError::MissingValue("--atuin-db"))?;
                config.atuin_db = PathBuf::from(v);
            }
            "--injection-db" => {
                let v = iter.next().ok_or(ArgError::MissingValue("--injection-db"))?;
                config.injection_db = PathBuf::from(v);
            }
            "--runs-db" => {
                let v = iter.next().ok_or(ArgError::MissingValue("--runs-db"))?;
                config.runs_db = PathBuf::from(v);
            }
            "--proposals-out" => {
                let v = iter
                    .next()
                    .ok_or(ArgError::MissingValue("--proposals-out"))?;
                config.proposals_out = PathBuf::from(v);
            }
            "--min-support" => {
                let v = iter.next().ok_or(ArgError::MissingValue("--min-support"))?;
                config.min_support = parse_usize("--min-support", v)?;
            }
            "--max-gap" => {
                let v = iter.next().ok_or(ArgError::MissingValue("--max-gap"))?;
                config.max_gap = parse_usize("--max-gap", v)?;
            }
            "--format" => {
                let v = iter.next().ok_or(ArgError::MissingValue("--format"))?;
                config.format = ReportFormat::parse(v)?;
            }
            other if other.starts_with('-') => {
                return Err(ArgError::UnknownFlag(other.to_owned()));
            }
            other => return Err(ArgError::UnexpectedPositional(other.to_owned())),
        }
    }
    Ok(config)
}

// ─── report ─────────────────────────────────────────────────────────────

/// Outcome of a `wf-crystallise` run, printable in text or JSON.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[non_exhaustive]
pub struct Report {
    /// atuin history rows ingested by m1.
    pub atuin_rows: usize,
    /// injection.db unresolved chains read by m3.
    pub injection_chains: usize,
    /// cascade clusters correlated by m4.
    pub cascade_clusters: usize,
    /// id of the workflow run recorded by m7.
    pub run_id: i64,
    /// observations merged onto the run row.
    pub observations_merged: usize,
    /// open runs the m14 lift snapshot was computed over.
    pub lift_window_runs: usize,
    /// sequential patterns mined by m20.
    pub patterns_mined: usize,
    /// proposals composed by m23 and written to the JSONL output.
    pub proposals_written: usize,
    /// stages skipped because `--offline` was set or a live service was down.
    pub stages_skipped: Vec<String>,
    /// `true` once the pipeline reached the end without aborting.
    pub completed: bool,
}

impl Report {
    /// Construct an empty report (all counts zero, not completed).
    fn empty() -> Self {
        Self {
            atuin_rows: 0,
            injection_chains: 0,
            cascade_clusters: 0,
            run_id: 0,
            observations_merged: 0,
            lift_window_runs: 0,
            patterns_mined: 0,
            proposals_written: 0,
            stages_skipped: Vec::new(),
            completed: false,
        }
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "wf-crystallise — pipeline report")?;
        writeln!(f, "  atuin rows ingested        : {}", self.atuin_rows)?;
        writeln!(f, "  injection chains read      : {}", self.injection_chains)?;
        writeln!(f, "  cascade clusters           : {}", self.cascade_clusters)?;
        writeln!(f, "  workflow run id            : {}", self.run_id)?;
        writeln!(
            f,
            "  observations merged        : {}",
            self.observations_merged
        )?;
        writeln!(f, "  lift window runs           : {}", self.lift_window_runs)?;
        writeln!(f, "  patterns mined             : {}", self.patterns_mined)?;
        writeln!(
            f,
            "  proposals written          : {}",
            self.proposals_written
        )?;
        if self.stages_skipped.is_empty() {
            writeln!(f, "  stages skipped             : none")?;
        } else {
            writeln!(
                f,
                "  stages skipped             : {}",
                self.stages_skipped.join(", ")
            )?;
        }
        write!(
            f,
            "  completed                  : {}",
            if self.completed { "yes" } else { "no" }
        )
    }
}

// ─── orchestration error ────────────────────────────────────────────────

/// A fault that aborts the `wf-crystallise` pipeline.
///
/// Only true faults (a missing required file, a workflow-runs DB that
/// cannot be opened, a proposals output that cannot be written) surface
/// here. A down *live* service never produces an `OrchestrationError` —
/// it degrades into a skipped-stage entry on the [`Report`].
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum OrchestrationError {
    /// The atuin history DB could not be opened.
    #[error("atuin ingest failed: {0}")]
    Atuin(#[from] AtuinConsumerError),
    /// The injection.db could not be opened.
    #[error("injection.db ingest failed: {0}")]
    Injection(#[from] InjectionDbError),
    /// The workflow-runs SQLite store could not be opened or written.
    #[error("workflow-runs store failed: {0}")]
    WorkflowRuns(#[from] WorkflowError),
    /// The m20 miner refused the supplied parameters.
    #[error("pattern miner failed: {0}")]
    Miner(#[from] MinerError),
    /// The m14 lift aggregator was constructed with invalid weights.
    #[error("lift aggregator config invalid: {0}")]
    Lift(#[from] LiftError),
    /// The proposals JSONL output file could not be written.
    #[error("proposals output write failed at {path}: {source}")]
    ProposalsOutput {
        /// The path that could not be written.
        path: PathBuf,
        /// The underlying I/O error.
        source: std::io::Error,
    },
}

// ─── pipeline driver ────────────────────────────────────────────────────

/// Map an m1 [`AtuinHistoryRow`] into an m4 [`AtuinStep`].
fn row_to_step(row: &AtuinHistoryRow) -> AtuinStep {
    AtuinStep {
        id: row.id.clone(),
        // m1 timestamps are ms; m4 steps are ns.
        ts_ns: row.timestamp_ms.saturating_mul(1_000_000),
        command: row.command.clone(),
        cwd: row.cwd.clone(),
        session: row.session.as_str().to_owned(),
        exit: row.exit,
    }
}

/// Map an m1 row's command into an opaque [`StepToken`] (FNV-1a, F11:
/// no human-meaningful label survives into the token value).
///
/// The 64-bit FNV-1a hash is folded into the 32-bit token space; a
/// collision merely merges two commands into one opaque equivalence
/// class, which the miner tolerates — the token is not an identifier.
#[allow(
    clippy::cast_possible_truncation,
    reason = "intentional 64→32-bit fold; the token is an opaque class, not an id"
)]
fn command_to_token(command: &str) -> StepToken {
    StepToken((fnv1a_64(command.as_bytes()) & 0xFFFF_FFFF) as u32)
}

/// Current wall-clock time as an RFC-3339-ish UTC stamp via `SystemTime`.
fn now_stamp() -> String {
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    // A monotone, sortable, parse-stable stamp. Not strictly RFC-3339 but
    // sufficient for the m7 `started_at` / `ended_at` TEXT columns.
    format!("{secs}")
}

/// Run the `wf-crystallise` pipeline end-to-end.
///
/// The pipeline always runs to completion. Live-service stages (stcortex
/// registration, ORAC LTP probe, m40 nexus emit) are attempted only when
/// `config.offline` is `false`; an unreachable service is logged via
/// `tracing` and recorded in [`Report::stages_skipped`], never an abort.
///
/// # Errors
///
/// Returns [`OrchestrationError`] only for a true fault: a missing atuin
/// or injection DB, a workflow-runs store that cannot be opened, an
/// invalid miner/lift parameter set, or a proposals output that cannot be
/// written. A down live service is not a fault.
pub fn run(config: &Config) -> Result<Report, OrchestrationError> {
    let mut report = Report::empty();

    // Stage 0 — trust floor (m8) is intentionally KEEP-DORMANT per the
    // S1003733 F2 assessment: no in-tree POVM-read site remains, so the
    // m8 compile-time gate has nothing to guard and the live POVM probe
    // is not invoked. The trust regime is enforced statically (build.rs
    // `povm_calibrated` cfg) rather than as a runtime pipeline stage.

    // Stage 1 — m1 atuin ingest (file stage; always runs).
    let atuin_cfg = AtuinConsumerConfig {
        db_path_override: Some(config.atuin_db.clone()),
        ..AtuinConsumerConfig::default()
    };
    let atuin_rows = open_atuin_readonly(&atuin_cfg)?.collect_all()?;
    report.atuin_rows = atuin_rows.len();
    tracing::info!(target: "wf_crystallise", rows = report.atuin_rows, "m1 atuin ingest complete");

    // Stage 1b — m3 injection.db ingest (file stage; always runs).
    let injection_cfg = InjectionDbConfig {
        db_path: config.injection_db.clone(),
        ..InjectionDbConfig::default()
    };
    let causal_chains = open_injection_readonly(&injection_cfg)?.read_unresolved()?;
    report.injection_chains = causal_chains.len();
    tracing::info!(
        target: "wf_crystallise",
        chains = report.injection_chains,
        "m3 injection.db ingest complete"
    );

    // Stage 2 — m2 stcortex registration (live stage).
    if config.offline {
        report.stages_skipped.push("m2-stcortex".to_owned());
    } else {
        register_stcortex_consumer(&mut report);
    }

    // Stage 3 — m4 cascade correlation (pure stage; always runs).
    let steps: Vec<AtuinStep> = atuin_rows.iter().map(row_to_step).collect();
    let correlator = CascadeCorrelator::new(CascadeCorrelatorConfig {
        atuin_db_path: config.atuin_db.clone(),
        ..CascadeCorrelatorConfig::default()
    });
    let clusters = correlator.correlate(&steps, &[]);
    report.cascade_clusters = clusters.len();

    // Stage 4 — m7 record run (file stage; always runs).
    let conn = open_runs_database(&config.runs_db)?;
    let started_at = now_stamp();
    let run_id = insert_run(&conn, &started_at)?;
    report.run_id = run_id;
    for cluster in &clusters {
        let obs = ClusterBObservation::Cascade {
            cluster_id: cluster.cluster_id.as_str().to_owned(),
            session_range: (cluster.window_start_ns, cluster.window_end_ns),
        };
        merge_observation(&conn, run_id, &obs)?;
        report.observations_merged += 1;
    }
    for chain in &causal_chains {
        let obs = ClusterBObservation::InjectionChain {
            chain_id: chain.id.get(),
        };
        merge_observation(&conn, run_id, &obs)?;
        report.observations_merged += 1;
    }
    // Stage 5 — m14 lift snapshot (file stage; always runs).
    //
    // The window is computed over the OPEN runs *before* the current run
    // is closed, so the current run plus any prior open runs from earlier
    // invocations of a persistent `--runs-db` are all in scope. Evidence
    // accumulates across invocations; a single fresh-DB run sees `n == 1`,
    // which is honestly below the F2 floor (m23 then skips every proposal).
    let aggregator = LiftAggregator::new(LiftAggregatorConfig::default())?;
    let open_runs = find_open(&conn, LiftAggregatorConfig::default().window)?;
    report.lift_window_runs = open_runs.len();
    let snapshot = aggregator.compute_snapshot(&open_runs);

    // Close the current run now that the lift window has been captured.
    close_run(&conn, run_id, &now_stamp(), Outcome::Ok.as_str())?;
    tracing::info!(
        target: "wf_crystallise",
        n = snapshot.n,
        has_lift = snapshot.lift.is_some(),
        "m14 lift snapshot computed"
    );

    // Stage 6 — m20 mine → m21 variants → m23 compose (pure KEYSTONE stage).
    let min_support = MinSupport::new(config.min_support)?;
    let max_gap = MaxGap::new(config.max_gap);
    // One sequence per atuin session, ordered by row arrival.
    let sequences = build_sequences(&atuin_rows);
    let patterns = mine_sequences(&sequences, min_support, max_gap, DEFAULT_MAX_PATTERN_LENGTH)?;
    report.patterns_mined = patterns.len();
    // m22 K-means diversity clustering — Plan v2 § 15 D17–D20 (R2 wiring).
    // See `build_variant_cluster_map` for the per-variant feature extraction,
    // adaptive-`k`, and degraded-fallback contract.
    let variant_to_cluster = build_variant_cluster_map(&patterns);
    // CC-7 wire — Plan v2 § 15 D22 "Pressure modulates m23 compose-priority
    // (additive, bounded)". Read the current substrate pressure scalar from
    // m15 (count of outstanding PHASE-B-RESERVATION-NOTICE files, saturated
    // at `PRESSURE_SATURATION_N`); a missing notices directory reads as
    // zero. The scalar feeds compose_proposals_with_pressure, which stable-
    // sorts proposals so SAFER variants (Identity > Swap > Skip) surface
    // first under elevated pressure. Zero pressure ⇒ output IDENTICAL to
    // the pre-Phase-7 compose_proposals path. See
    // `read_pressure_scalar_for_compose` for the read-and-degrade contract.
    let pressure = read_pressure_scalar_for_compose(&mut report);
    let proposals = compose_proposals_with_pressure(
        &patterns,
        &snapshot,
        |v| variant_to_cluster.get(&v.variant_id).copied(),
        pressure,
    );

    // Stage 7 — write proposals JSONL.
    report.proposals_written = write_proposals_jsonl(&config.proposals_out, &proposals)?;
    tracing::info!(
        target: "wf_crystallise",
        written = report.proposals_written,
        path = %config.proposals_out.display(),
        "m23 proposals written to JSONL"
    );

    // Stage 8 — m40 nexus emit (live stage).
    if config.offline {
        report.stages_skipped.push("m40-nexus-emit".to_owned());
    } else {
        emit_nexus_completion(&mut report, run_id);
    }

    report.completed = true;
    Ok(report)
}

/// Read the current m15 substrate pressure scalar for the m23 compose
/// path (Phase 7 / CC-7 wire — Plan v2 § 15 D22 + D24).
///
/// The scalar is in `[0.0, 1.0]` and reflects the count of outstanding
/// `PHASE-B-RESERVATION-NOTICE` JSONL files in the m15 notices directory,
/// saturated at [`crate::m15_pressure::PRESSURE_SATURATION_N`]. A missing
/// notices directory reads as `0.0` (fresh-deploy / cold-boot is honest
/// zero pressure, never a fault).
///
/// **Degraded-read contract.** This function NEVER raises an
/// [`OrchestrationError`]: any I/O fault enumerating the notices directory
/// is logged via `tracing::warn` and the stage is added to
/// `Report::stages_skipped` as `"m15-pressure-read"`; the returned scalar
/// is `0.0`, which makes [`compose_proposals_with_pressure`] behave
/// identically to the pre-Phase-7 compose path. m15 is a witness; its
/// failure must not block the engine.
fn read_pressure_scalar_for_compose(report: &mut Report) -> f64 {
    let register = PressureRegister::new(PressureRegisterConfig::default());
    match register.read_pressure_level() {
        Ok(scalar) => {
            tracing::debug!(
                target: "wf_crystallise",
                pressure = scalar,
                "m15 pressure scalar read for m23 compose"
            );
            scalar
        }
        Err(err) => {
            tracing::warn!(
                target: "wf_crystallise",
                error = %err,
                "m15 pressure read failed — degrading to zero pressure (graceful)"
            );
            report.stages_skipped.push("m15-pressure-read".to_owned());
            0.0
        }
    }
}

/// Assemble the `variant_id → cluster_index` map for the m23 diversity
/// closure (Plan v2 § 15 D17–D20, R2 wiring).
///
/// Pre-builds every variant the m23 proposer would compose, extracts a
/// 5-dimensional feature vector per variant (step-count-norm, mutation
/// one-hot ×3, Levenshtein-from-identity-norm — per D17), runs K-means
/// with `k` adaptive to variant count (D19), and returns the
/// `variant_id → cluster_index` map. The `diversity_cluster:
/// Option<usize>` field on each `WorkflowProposal` is then a real m22
/// signal (D20: emitted via the proposals JSONL bridge —
/// `WorkflowProposal::diversity_cluster` is `#[derive(serde::Serialize)]`-
/// included — and forwarded through to substrate-side surfaces;
/// `m31_selector` already accepts a `diversity_score` closure that
/// downstream `wf-dispatch` callers can implement against
/// `proposal.diversity_cluster()` per D18).
///
/// If `kmeans` returns an error (fewer viable points than `k`, dimension
/// mismatch, non-finite input), the map stays empty and the m23 closure
/// falls back to `None` for every variant — the path keeps running, the
/// substrate signal is honestly absent rather than faked.
/// `build_variants` is deterministic (variant ids are FNV-1a hashes of
/// `(steps, mutation)`) so the variants re-derived inside
/// `compose_proposals` carry the same ids as the ones this map was
/// indexed by.
fn build_variant_cluster_map(
    patterns: &[crate::m20_prefixspan::Pattern],
) -> std::collections::HashMap<u64, usize> {
    let all_variants: Vec<WorkflowVariant> = patterns
        .iter()
        .flat_map(|p| build_variants(p).ok().into_iter().flatten())
        .collect();
    let mut variant_to_cluster: std::collections::HashMap<u64, usize> =
        std::collections::HashMap::new();
    if all_variants.is_empty() {
        return variant_to_cluster;
    }
    let features: Vec<Vec<f64>> = all_variants.iter().map(extract_variant_features).collect();
    let k = recommended_k_for_variant_count(all_variants.len());
    let cfg = KMeansConfig {
        k,
        ..KMeansConfig::default()
    };
    match kmeans(&features, &cfg) {
        Ok((clustered, _centroids)) => {
            for (variant, point) in all_variants.iter().zip(clustered.iter()) {
                variant_to_cluster.insert(variant.variant_id, point.cluster);
            }
            tracing::info!(
                target: "wf_crystallise",
                n_variants = all_variants.len(),
                k,
                "m22 K-means clustering assembled variant→cluster map"
            );
        }
        Err(err) => {
            tracing::debug!(
                target: "wf_crystallise",
                error = %err,
                n_variants = all_variants.len(),
                k,
                "m22 K-means clustering skipped; diversity_cluster will be None for this run"
            );
        }
    }
    variant_to_cluster
}

/// Group atuin rows into per-session [`StepToken`] sequences.
fn build_sequences(rows: &[AtuinHistoryRow]) -> Vec<Vec<StepToken>> {
    use std::collections::BTreeMap;
    let mut by_session: BTreeMap<String, Vec<StepToken>> = BTreeMap::new();
    for row in rows {
        by_session
            .entry(row.session.as_str().to_owned())
            .or_default()
            .push(command_to_token(&row.command));
    }
    by_session.into_values().collect()
}

/// Write proposals to `path`, one JSON object per line. Returns the number
/// of lines written.
fn write_proposals_jsonl(
    path: &Path,
    proposals: &[crate::m23_proposer::WorkflowProposal],
) -> Result<usize, OrchestrationError> {
    let file = File::create(path).map_err(|source| OrchestrationError::ProposalsOutput {
        path: path.to_path_buf(),
        source,
    })?;
    let mut writer = BufWriter::new(file);
    for proposal in proposals {
        // `WorkflowProposal` derives `Serialize`; one compact line each.
        let line = serde_json::to_string(proposal).map_err(|e| {
            OrchestrationError::ProposalsOutput {
                path: path.to_path_buf(),
                source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
            }
        })?;
        writer
            .write_all(line.as_bytes())
            .and_then(|()| writer.write_all(b"\n"))
            .map_err(|source| OrchestrationError::ProposalsOutput {
                path: path.to_path_buf(),
                source,
            })?;
    }
    writer
        .flush()
        .map_err(|source| OrchestrationError::ProposalsOutput {
            path: path.to_path_buf(),
            source,
        })?;
    Ok(proposals.len())
}

/// Attempt the m2 stcortex consumer registration; degrade gracefully on
/// any failure (an unreachable substrate is the expected offline mode).
fn register_stcortex_consumer(report: &mut Report) {
    use crate::m2_stcortex_consumer::{
        register_narrowed_consumer, ConsumerIdentity, ConsumerName, Namespace, Transport,
    };
    // Build a workflow_trace_* identity; if either newtype rejects the
    // input the stage is skipped (it cannot in practice — the literals are
    // valid — but we never `unwrap` on it).
    let identity = ConsumerName::new("wf-crystallise")
        .ok()
        .zip(Namespace::new("workflow_trace_crystallise").ok())
        .map(|(name, namespace)| ConsumerIdentity {
            name,
            namespace,
            transport: Transport::Subscription,
        });
    let Some(identity) = identity else {
        tracing::warn!(target: "wf_crystallise", "m2 identity construction failed; skipping");
        report.stages_skipped.push("m2-stcortex".to_owned());
        return;
    };
    match register_narrowed_consumer(identity, 5_000) {
        Ok(handle) => {
            tracing::info!(
                target: "wf_crystallise",
                fresh = handle.is_fresh(),
                "m2 stcortex consumer registered"
            );
        }
        Err(e) => {
            tracing::warn!(
                target: "wf_crystallise",
                error = %e,
                "m2 stcortex unreachable — skipping (graceful degradation)"
            );
            report.stages_skipped.push("m2-stcortex".to_owned());
        }
    }
}

/// Attempt the m40 nexus completion emit; degrade gracefully if synthex-v2
/// is unreachable.
fn emit_nexus_completion(report: &mut Report, run_id: i64) {
    use crate::m40_nexus_emit::{
        build_event, HttpNexusClient, NexusClient, NexusEventKind, DEFAULT_NEXUS_URL,
        DEFAULT_PUSH_TIMEOUT,
    };
    let client = HttpNexusClient::new(DEFAULT_NEXUS_URL, DEFAULT_PUSH_TIMEOUT);
    let ts_ms = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .ok()
        .and_then(|d| i64::try_from(d.as_millis()).ok())
        .unwrap_or(0);
    let event = build_event(
        "wf-crystallise",
        NexusEventKind::WorkflowCompleted,
        serde_json::json!({ "run_id": run_id }),
        ts_ms,
    );
    match client.push(&event) {
        Ok(()) => tracing::info!(target: "wf_crystallise", run_id, "m40 nexus event pushed"),
        Err(e) => {
            tracing::warn!(
                target: "wf_crystallise",
                error = %e,
                "m40 synthex-v2 unreachable — skipping (graceful degradation)"
            );
            report.stages_skipped.push("m40-nexus-emit".to_owned());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_args, ArgError, Config, ReportFormat, DEFAULT_MAX_GAP, DEFAULT_MIN_SUPPORT};
    use std::path::PathBuf;

    fn args(raw: &[&str]) -> Vec<String> {
        raw.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn parse_args_empty_yields_defaults() {
        // rationale: Boundary — no flags ⇒ the documented default Config.
        let cfg = parse_args(&[]).expect("empty parse");
        assert_eq!(cfg.min_support, DEFAULT_MIN_SUPPORT);
        assert_eq!(cfg.max_gap, DEFAULT_MAX_GAP);
        assert!(!cfg.offline);
        assert_eq!(cfg.format, ReportFormat::Text);
        assert!(!cfg.show_help);
        assert!(!cfg.show_version);
        assert_eq!(cfg.proposals_out, PathBuf::from("./proposals.jsonl"));
    }

    #[test]
    fn parse_args_full_flag_set() {
        // rationale: Cross-flag — every value-bearing flag is parsed.
        let cfg = parse_args(&args(&[
            "--atuin-db",
            "/tmp/a.db",
            "--injection-db",
            "/tmp/i.db",
            "--runs-db",
            "/tmp/r.db",
            "--proposals-out",
            "/tmp/p.jsonl",
            "--min-support",
            "7",
            "--max-gap",
            "9",
            "--offline",
            "--format",
            "json",
        ]))
        .expect("full parse");
        assert_eq!(cfg.atuin_db, PathBuf::from("/tmp/a.db"));
        assert_eq!(cfg.injection_db, PathBuf::from("/tmp/i.db"));
        assert_eq!(cfg.runs_db, PathBuf::from("/tmp/r.db"));
        assert_eq!(cfg.proposals_out, PathBuf::from("/tmp/p.jsonl"));
        assert_eq!(cfg.min_support, 7);
        assert_eq!(cfg.max_gap, 9);
        assert!(cfg.offline);
        assert_eq!(cfg.format, ReportFormat::Json);
    }

    #[test]
    fn parse_args_help_and_version() {
        // rationale: Boundary — `--help` / `--version` set their flags.
        assert!(parse_args(&args(&["--help"])).expect("help").show_help);
        assert!(parse_args(&args(&["-h"])).expect("h").show_help);
        assert!(parse_args(&args(&["--version"]))
            .expect("version")
            .show_version);
        assert!(parse_args(&args(&["-V"])).expect("V").show_version);
    }

    #[test]
    fn parse_args_unknown_flag_is_typed_error() {
        // rationale: Anti-property — an unknown flag is a typed refusal.
        let err = parse_args(&args(&["--bogus"])).expect_err("unknown");
        assert_eq!(err, ArgError::UnknownFlag("--bogus".to_owned()));
    }

    #[test]
    fn parse_args_missing_value_is_typed_error() {
        // rationale: Anti-property — a value-bearing flag with no value.
        let err = parse_args(&args(&["--min-support"])).expect_err("missing");
        assert_eq!(err, ArgError::MissingValue("--min-support"));
    }

    #[test]
    fn parse_args_bad_numeric_value_is_typed_error() {
        // rationale: Anti-property — an unparseable numeric value.
        let err = parse_args(&args(&["--max-gap", "xyz"])).expect_err("bad");
        assert_eq!(
            err,
            ArgError::BadValue {
                flag: "--max-gap",
                value: "xyz".to_owned(),
            }
        );
    }

    #[test]
    fn parse_args_bad_format_value_is_typed_error() {
        // rationale: Anti-property — `--format` outside {text,json}.
        let err = parse_args(&args(&["--format", "yaml"])).expect_err("bad fmt");
        assert_eq!(
            err,
            ArgError::BadValue {
                flag: "--format",
                value: "yaml".to_owned(),
            }
        );
    }

    #[test]
    fn parse_args_positional_is_typed_error() {
        // rationale: Anti-property — no positional args are accepted.
        let err = parse_args(&args(&["extra"])).expect_err("positional");
        assert_eq!(err, ArgError::UnexpectedPositional("extra".to_owned()));
    }

    #[test]
    fn config_default_is_offline_false() {
        // rationale: Boundary — default Config is online (offline = false).
        assert!(!Config::default().offline);
    }
}
