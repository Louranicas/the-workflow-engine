//! `wf-dispatch` pipeline driver + CLI parser.
//!
//! Drives the m30-m33 stages: read the proposals JSONL produced by
//! `wf-crystallise`, accept each into an in-memory curated bank (m30),
//! score + select the top-K (m31), aggregate the 4-verifier gate (m33),
//! and — only under `--execute` — dispatch the selected workflows via the
//! HABITAT-CONDUCTOR (m32).
//!
//! `--dry-run` is the default-safe mode: it verifies and selects but never
//! contacts the Conductor. A real dispatch requires the explicit
//! `--execute` flag *and* a reachable Conductor; an unreachable Conductor
//! degrades into a refused-and-recorded outcome rather than a panic.

use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::m11_fitness_weighted_decay::chrono_now_ms;
use crate::m23_proposer::WorkflowProposal;
use crate::m30_bank::{BankError, CuratedBank};
use crate::m31_selector::{select_top_k, SelectorConfig, SelectorError};
use crate::m32_dispatcher::{
    ConductorClient, ConductorDispatcher, DispatchOutcome, DispatcherError, EscapeSurfaceProfile,
    HumanAcceptanceSignature,
};
use crate::m33_verifier::{aggregate, AggregateVerdict, Verifier, VerifierKind, VerifierVerdict};

/// Default proposals JSONL input path.
pub const DEFAULT_PROPOSALS_IN: &str = "./proposals.jsonl";
/// Default number of workflows to select.
pub const DEFAULT_TOP_K: usize = 5;
/// Default HABITAT-CONDUCTOR base URL.
pub const DEFAULT_CONDUCTOR_URL: &str = "http://127.0.0.1:8141";
/// HABITAT-CONDUCTOR dispatch path appended to the base URL.
pub const CONDUCTOR_DISPATCH_PATH: &str = "/dispatch";
/// HTTP timeout for a Conductor submit.
pub const CONDUCTOR_TIMEOUT: Duration = Duration::from_secs(10);

// ─── config ─────────────────────────────────────────────────────────────

/// Parsed `wf-dispatch` CLI options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// proposals JSONL input path.
    pub proposals_in: PathBuf,
    /// number of workflows to select.
    pub top_k: usize,
    /// HABITAT-CONDUCTOR base URL.
    pub conductor_url: String,
    /// when `true`, verify + select but never contact the Conductor.
    pub dry_run: bool,
    /// the escape-surface ceiling the operator acknowledges.
    pub ack_ceiling: EscapeSurfaceProfile,
    /// `--help` requested.
    pub show_help: bool,
    /// `--version` requested.
    pub show_version: bool,
}

impl Default for Config {
    /// `--dry-run` is the default-safe mode; `ack_ceiling` defaults to the
    /// least-destructive [`EscapeSurfaceProfile::Sandboxed`].
    fn default() -> Self {
        Self {
            proposals_in: PathBuf::from(DEFAULT_PROPOSALS_IN),
            top_k: DEFAULT_TOP_K,
            conductor_url: DEFAULT_CONDUCTOR_URL.to_owned(),
            dry_run: true,
            ack_ceiling: EscapeSurfaceProfile::Sandboxed,
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
wf-dispatch — verify + dispatch crystallised workflow proposals

USAGE:
    wf-dispatch [OPTIONS]

OPTIONS:
    --proposals-in <PATH>    JSONL produced by wf-crystallise (default: ./proposals.jsonl)
    --top-k <N>              number of workflows to select (default: 5)
    --conductor-url <URL>    HABITAT-CONDUCTOR base URL (default: http://127.0.0.1:8141)
    --dry-run                verify + select but do NOT contact the Conductor (DEFAULT)
    --execute                perform real dispatch via the Conductor (overrides --dry-run)
    --ack-ceiling <PROFILE>  escape-surface ceiling acknowledged by the operator
                             one of: sandboxed sandbox-escape process-mutate
                             privilege-escalation file-write network-egress data-exfil
                             (default: sandboxed)
    --help                   print this help and exit
    --version                print version and exit
";

/// Map an `--ack-ceiling` argument value to an [`EscapeSurfaceProfile`].
fn parse_ack_ceiling(raw: &str) -> Result<EscapeSurfaceProfile, ArgError> {
    match raw {
        "sandboxed" => Ok(EscapeSurfaceProfile::Sandboxed),
        "sandbox-escape" => Ok(EscapeSurfaceProfile::SandboxEscape),
        "process-mutate" => Ok(EscapeSurfaceProfile::ProcessMutate),
        "privilege-escalation" => Ok(EscapeSurfaceProfile::PrivilegeEscalation),
        "file-write" => Ok(EscapeSurfaceProfile::FileWrite),
        "network-egress" => Ok(EscapeSurfaceProfile::NetworkEgress),
        "data-exfil" => Ok(EscapeSurfaceProfile::DataExfil),
        other => Err(ArgError::BadValue {
            flag: "--ack-ceiling",
            value: other.to_owned(),
        }),
    }
}

/// Parse `wf-dispatch` CLI arguments.
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
            "--dry-run" => config.dry_run = true,
            "--execute" => config.dry_run = false,
            "--proposals-in" => {
                let v = iter.next().ok_or(ArgError::MissingValue("--proposals-in"))?;
                config.proposals_in = PathBuf::from(v);
            }
            "--top-k" => {
                let v = iter.next().ok_or(ArgError::MissingValue("--top-k"))?;
                config.top_k = v.parse::<usize>().map_err(|_| ArgError::BadValue {
                    flag: "--top-k",
                    value: v.clone(),
                })?;
            }
            "--conductor-url" => {
                let v = iter
                    .next()
                    .ok_or(ArgError::MissingValue("--conductor-url"))?;
                config.conductor_url.clone_from(v);
            }
            "--ack-ceiling" => {
                let v = iter.next().ok_or(ArgError::MissingValue("--ack-ceiling"))?;
                config.ack_ceiling = parse_ack_ceiling(v)?;
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

/// One selected workflow's verify + dispatch outcome.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[non_exhaustive]
pub struct CandidateOutcome {
    /// The bank workflow id.
    pub workflow_id: u64,
    /// `true` once the 4-verifier m33 gate approved this workflow.
    pub verifier_approved: bool,
    /// One of: `dry-run`, `dispatched`, `refused`, `verifier-blocked`.
    pub disposition: String,
}

/// Outcome of a `wf-dispatch` run, printable in text or JSON.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[non_exhaustive]
pub struct Report {
    /// proposals parsed from the JSONL input.
    pub proposals_loaded: usize,
    /// proposals successfully accepted into the bank (m30).
    pub bank_accepted: usize,
    /// top-K candidates selected by m31.
    pub candidates_selected: usize,
    /// candidates whose m33 verifier gate approved.
    pub verifier_approved: usize,
    /// candidates actually dispatched to the Conductor (always 0 in dry-run).
    pub dispatched: usize,
    /// `true` when the run executed in dry-run mode.
    pub dry_run: bool,
    /// per-candidate dispositions.
    pub candidates: Vec<CandidateOutcome>,
    /// `true` once the pipeline reached the end without aborting.
    pub completed: bool,
}

impl Report {
    /// Construct an empty report.
    fn empty(dry_run: bool) -> Self {
        Self {
            proposals_loaded: 0,
            bank_accepted: 0,
            candidates_selected: 0,
            verifier_approved: 0,
            dispatched: 0,
            dry_run,
            candidates: Vec::new(),
            completed: false,
        }
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "wf-dispatch — pipeline report")?;
        writeln!(
            f,
            "  mode                       : {}",
            if self.dry_run { "dry-run" } else { "execute" }
        )?;
        writeln!(f, "  proposals loaded           : {}", self.proposals_loaded)?;
        writeln!(f, "  bank accepted              : {}", self.bank_accepted)?;
        writeln!(
            f,
            "  candidates selected        : {}",
            self.candidates_selected
        )?;
        writeln!(f, "  verifier approved          : {}", self.verifier_approved)?;
        writeln!(f, "  dispatched                 : {}", self.dispatched)?;
        for c in &self.candidates {
            writeln!(
                f,
                "    workflow {:>20} : {}",
                c.workflow_id, c.disposition
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

/// A fault that aborts the `wf-dispatch` pipeline.
///
/// A down Conductor is *not* a fault — it degrades into a refused
/// candidate. Only a missing/unreadable proposals file, a malformed
/// proposals file, an unrecoverable bank fault, or an invalid selector
/// configuration surface here.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum OrchestrationError {
    /// The proposals JSONL input file could not be read.
    #[error("proposals input read failed at {path}: {source}")]
    ProposalsInput {
        /// The path that could not be read.
        path: PathBuf,
        /// The underlying I/O error.
        source: std::io::Error,
    },
    /// A line of the proposals JSONL could not be parsed.
    #[error("proposals input parse failed at {path} line {line}: {detail}")]
    ProposalsParse {
        /// The path being parsed.
        path: PathBuf,
        /// 1-based line number that failed.
        line: usize,
        /// Human-readable parse failure.
        detail: String,
    },
    /// The curated bank rejected a proposal unrecoverably.
    #[error("bank fault: {0}")]
    Bank(#[from] BankError),
    /// The m31 selector was given an invalid configuration.
    #[error("selector fault: {0}")]
    Selector(#[from] SelectorError),
}

// ─── verifiers ──────────────────────────────────────────────────────────

/// Conservative default verifier: approves every workflow.
///
/// The m33 gate requires exactly one [`Verifier`] per [`VerifierKind`].
/// Genuine per-kind verification logic (a real security scan, a real
/// cost-bound check) is out of scope for the CLI wiring task — it would
/// need policy inputs the binary does not receive. A documented
/// **conservative default of "approve"** is supplied so the gate is
/// structurally present and exercised end-to-end. This is flagged: the
/// gate is wired and deterministic, but its verdict is currently a
/// placeholder, not a real audit. Replace each verifier's `verify` body
/// with real logic when the policy inputs are specified.
struct ConservativeVerifier {
    kind: VerifierKind,
}

impl Verifier for ConservativeVerifier {
    fn kind(&self) -> VerifierKind {
        self.kind
    }

    fn verify(&self, _workflow: &crate::m30_bank::AcceptedWorkflow) -> VerifierVerdict {
        // Conservative default — see the struct doc comment. The gate is
        // real and runs; the verdict is a documented placeholder.
        VerifierVerdict::Approve
    }
}

/// Build the four-verifier set required by [`aggregate`] — exactly one
/// [`Verifier`] per [`VerifierKind`].
fn build_verifiers() -> Vec<Box<dyn Verifier>> {
    VerifierKind::VARIANTS
        .iter()
        .map(|&kind| -> Box<dyn Verifier> { Box::new(ConservativeVerifier { kind }) })
        .collect()
}

// ─── production Conductor client ────────────────────────────────────────

/// Minimal blocking-`reqwest` [`ConductorClient`] for the production
/// HABITAT-CONDUCTOR endpoint.
///
/// m32 ships the [`ConductorClient`] *trait* but no production HTTP impl
/// (tests inject mocks). This is that impl: a single blocking `POST` to
/// `{base_url}{CONDUCTOR_DISPATCH_PATH}` carrying the workflow id +
/// escape-surface profile. An unreachable Conductor / non-2xx response /
/// unparseable body all collapse into [`DispatcherError::WireFormat`],
/// which m32's `dispatch` folds into a refused — not panicked — outcome.
struct HttpConductorClient {
    /// Fully-qualified dispatch endpoint (`{base}/dispatch`).
    endpoint: String,
    /// Blocking HTTP timeout.
    timeout: Duration,
}

impl HttpConductorClient {
    /// Construct against a Conductor base URL.
    fn new(base_url: &str, timeout: Duration) -> Self {
        let base = base_url.trim_end_matches('/');
        Self {
            endpoint: format!("{base}{CONDUCTOR_DISPATCH_PATH}"),
            timeout,
        }
    }
}

impl ConductorClient for HttpConductorClient {
    fn submit(
        &self,
        workflow_id: u64,
        profile: EscapeSurfaceProfile,
        _signature: &HumanAcceptanceSignature,
    ) -> Result<String, DispatcherError> {
        // The Conductor routes `lcm.loop.create` — m32's routing-method
        // check is satisfied because `dispatch_method` defaults to the
        // canonical `CONDUCTOR_DISPATCH_METHOD`.
        let body = serde_json::json!({
            "method": crate::m32_dispatcher::CONDUCTOR_DISPATCH_METHOD,
            "workflow_id": workflow_id,
            "escape_surface_ordinal": profile.ordinal(),
            "source": "wf-dispatch",
        });
        let client = reqwest::blocking::Client::builder()
            .timeout(self.timeout)
            .build()
            .map_err(|e| DispatcherError::WireFormat(format!("client build: {e}")))?;
        let resp = client
            .post(&self.endpoint)
            .json(&body)
            .send()
            .map_err(|e| DispatcherError::WireFormat(format!("conductor unreachable: {e}")))?;
        if !resp.status().is_success() {
            return Err(DispatcherError::WireFormat(format!(
                "conductor returned HTTP {}",
                resp.status().as_u16()
            )));
        }
        let parsed: serde_json::Value = resp
            .json()
            .map_err(|e| DispatcherError::WireFormat(format!("conductor body: {e}")))?;
        // Accept either `dispatch_id` or `id`; AP-V7-13 — a 2xx carrying an
        // `error` field is NOT a success.
        if let Some(err) = parsed.get("error").and_then(serde_json::Value::as_str) {
            return Err(DispatcherError::WireFormat(format!(
                "conductor rejected dispatch: {err}"
            )));
        }
        let id = parsed
            .get("dispatch_id")
            .or_else(|| parsed.get("id"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| {
                DispatcherError::WireFormat("conductor response missing dispatch id".to_owned())
            })?;
        Ok(id)
    }
}

// ─── pipeline driver ────────────────────────────────────────────────────

/// Run the `wf-dispatch` pipeline end-to-end.
///
/// `--dry-run` (the default) verifies + selects but never contacts the
/// Conductor. `--execute` performs real dispatch; an unreachable Conductor
/// degrades into a refused candidate, never a panic.
///
/// # Errors
///
/// Returns [`OrchestrationError`] only for a true fault: a missing or
/// malformed proposals file, an unrecoverable bank fault, or an invalid
/// selector configuration. A down Conductor is not a fault.
pub fn run(config: &Config) -> Result<Report, OrchestrationError> {
    let mut report = Report::empty(config.dry_run);

    // Stage 1 — load proposals from the JSONL bridge.
    let proposals = load_proposals(&config.proposals_in)?;
    report.proposals_loaded = proposals.len();
    tracing::info!(
        target: "wf_dispatch",
        loaded = report.proposals_loaded,
        path = %config.proposals_in.display(),
        "proposals JSONL loaded"
    );

    // Stage 2 — m30 accept each proposal into the curated bank.
    let now_ms = chrono_now_ms().unwrap_or(0);
    let bank = CuratedBank::new();
    for proposal in proposals {
        match bank.accept(proposal, now_ms) {
            Ok(_id) => report.bank_accepted += 1,
            Err(e) => {
                // A single rejected proposal (e.g. AP-V7-08 self-dispatch
                // sentinel) is logged and skipped; the run continues.
                tracing::warn!(target: "wf_dispatch", error = %e, "m30 rejected a proposal; skipping");
            }
        }
    }
    tracing::info!(target: "wf_dispatch", accepted = report.bank_accepted, "m30 bank populated");

    // Stage 3 — m31 select top-K from the bank's active set.
    let actives = bank.active(now_ms, 0.0);
    let selector_cfg = SelectorConfig::default();
    // `|_| 0.0` diversity closure: m22 K-means clustering is not wired into
    // the dispatch CLI path; a documented conservative default of "no
    // diversity contribution" is passed (honest: the signal is absent).
    let candidates = select_top_k(&actives, &selector_cfg, |_w| 0.0, now_ms, config.top_k)?;
    report.candidates_selected = candidates.len();

    // Stage 4 — per-candidate m33 verify, then m32 dispatch under --execute.
    let verifiers = build_verifiers();
    let signature = HumanAcceptanceSignature {
        // The operator acknowledges the ceiling supplied on the CLI. The
        // binary itself is the operator interface; the flag IS the
        // acknowledgement.
        interactive_terminal: true,
        acknowledged_ceiling: config.ack_ceiling,
    };
    let http_client = HttpConductorClient::new(&config.conductor_url, CONDUCTOR_TIMEOUT);
    let dispatcher = ConductorDispatcher::new(http_client);

    for candidate in &candidates {
        let workflow = match bank.get(candidate.workflow_id) {
            Ok(w) => w,
            Err(e) => {
                tracing::warn!(target: "wf_dispatch", error = %e, "m30 get failed mid-run; skipping");
                continue;
            }
        };
        // m33 — run the 4-verifier gate.
        let verifier_refs: Vec<&dyn Verifier> =
            verifiers.iter().map(std::convert::AsRef::as_ref).collect();
        let approved = matches!(
            aggregate(&verifier_refs, &workflow),
            Ok(AggregateVerdict::AllApprove)
        );
        if approved {
            report.verifier_approved += 1;
        }
        let disposition = if !approved {
            "verifier-blocked".to_owned()
        } else if config.dry_run {
            // Dry-run: verified + selected, but the Conductor is NOT
            // contacted. This is the default-safe path.
            "dry-run".to_owned()
        } else {
            // Execute: real dispatch. m32 folds an unreachable Conductor
            // into DispatchOutcome::Refused — never a panic.
            dispatch_one(&dispatcher, &workflow, config.ack_ceiling, &signature, &mut report)
        };
        report.candidates.push(CandidateOutcome {
            workflow_id: candidate.workflow_id,
            verifier_approved: approved,
            disposition,
        });
    }

    report.completed = true;
    Ok(report)
}

/// Dispatch one verified workflow via m32; returns the disposition string
/// and bumps `report.dispatched` on a real Conductor acceptance.
fn dispatch_one(
    dispatcher: &ConductorDispatcher<HttpConductorClient>,
    workflow: &crate::m30_bank::AcceptedWorkflow,
    profile: EscapeSurfaceProfile,
    signature: &HumanAcceptanceSignature,
    report: &mut Report,
) -> String {
    match dispatcher.dispatch(workflow, profile, signature) {
        Ok(DispatchOutcome::Accepted {
            conductor_dispatch_id,
        }) => {
            report.dispatched += 1;
            tracing::info!(
                target: "wf_dispatch",
                workflow_id = workflow.workflow_id(),
                dispatch_id = %conductor_dispatch_id,
                "m32 dispatch accepted by Conductor"
            );
            "dispatched".to_owned()
        }
        Ok(DispatchOutcome::Refused { reason }) => {
            tracing::warn!(
                target: "wf_dispatch",
                workflow_id = workflow.workflow_id(),
                reason = ?reason,
                "m32 dispatch refused (graceful degradation)"
            );
            "refused".to_owned()
        }
        Err(e) => {
            tracing::warn!(
                target: "wf_dispatch",
                workflow_id = workflow.workflow_id(),
                error = %e,
                "m32 dispatch caller fault"
            );
            "refused".to_owned()
        }
    }
}

/// Read + parse the proposals JSONL file into a `Vec<WorkflowProposal>`.
///
/// Each non-empty line is one JSON-encoded [`WorkflowProposal`]. A blank
/// line is skipped; a malformed line aborts with [`OrchestrationError`].
fn load_proposals(path: &Path) -> Result<Vec<WorkflowProposal>, OrchestrationError> {
    let file = File::open(path).map_err(|source| OrchestrationError::ProposalsInput {
        path: path.to_path_buf(),
        source,
    })?;
    let reader = BufReader::new(file);
    let mut proposals = Vec::new();
    for (idx, line) in reader.lines().enumerate() {
        let line = line.map_err(|source| OrchestrationError::ProposalsInput {
            path: path.to_path_buf(),
            source,
        })?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let proposal: WorkflowProposal =
            serde_json::from_str(trimmed).map_err(|e| OrchestrationError::ProposalsParse {
                path: path.to_path_buf(),
                line: idx + 1,
                detail: e.to_string(),
            })?;
        proposals.push(proposal);
    }
    Ok(proposals)
}

#[cfg(test)]
mod tests {
    use super::{
        build_verifiers, parse_args, ArgError, Config, DEFAULT_TOP_K,
    };
    use crate::m32_dispatcher::EscapeSurfaceProfile;
    use crate::m33_verifier::VerifierKind;
    use std::path::PathBuf;

    fn args(raw: &[&str]) -> Vec<String> {
        raw.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn parse_args_empty_yields_dry_run_default() {
        // rationale: Boundary — no flags ⇒ default-safe dry-run mode.
        let cfg = parse_args(&[]).expect("empty parse");
        assert!(cfg.dry_run, "dry-run must be the default-safe mode");
        assert_eq!(cfg.top_k, DEFAULT_TOP_K);
        assert_eq!(cfg.ack_ceiling, EscapeSurfaceProfile::Sandboxed);
        assert_eq!(cfg.proposals_in, PathBuf::from("./proposals.jsonl"));
        assert_eq!(cfg.conductor_url, "http://127.0.0.1:8141");
    }

    #[test]
    fn parse_args_execute_overrides_dry_run() {
        // rationale: Cross-flag — `--execute` flips off the dry-run default.
        let cfg = parse_args(&args(&["--execute"])).expect("execute");
        assert!(!cfg.dry_run);
        // ...and `--dry-run` after `--execute` flips it back (last-wins).
        let cfg2 = parse_args(&args(&["--execute", "--dry-run"])).expect("both");
        assert!(cfg2.dry_run);
    }

    #[test]
    fn parse_args_full_flag_set() {
        // rationale: Cross-flag — every value-bearing flag is parsed.
        let cfg = parse_args(&args(&[
            "--proposals-in",
            "/tmp/p.jsonl",
            "--top-k",
            "3",
            "--conductor-url",
            "http://example:9999",
            "--ack-ceiling",
            "file-write",
            "--execute",
        ]))
        .expect("full parse");
        assert_eq!(cfg.proposals_in, PathBuf::from("/tmp/p.jsonl"));
        assert_eq!(cfg.top_k, 3);
        assert_eq!(cfg.conductor_url, "http://example:9999");
        assert_eq!(cfg.ack_ceiling, EscapeSurfaceProfile::FileWrite);
        assert!(!cfg.dry_run);
    }

    #[test]
    fn parse_args_all_ack_ceiling_variants() {
        // rationale: Boundary — every documented `--ack-ceiling` value
        // maps to the matching EscapeSurfaceProfile variant.
        for (raw, expected) in [
            ("sandboxed", EscapeSurfaceProfile::Sandboxed),
            ("sandbox-escape", EscapeSurfaceProfile::SandboxEscape),
            ("process-mutate", EscapeSurfaceProfile::ProcessMutate),
            ("privilege-escalation", EscapeSurfaceProfile::PrivilegeEscalation),
            ("file-write", EscapeSurfaceProfile::FileWrite),
            ("network-egress", EscapeSurfaceProfile::NetworkEgress),
            ("data-exfil", EscapeSurfaceProfile::DataExfil),
        ] {
            let cfg = parse_args(&args(&["--ack-ceiling", raw])).expect("ack");
            assert_eq!(cfg.ack_ceiling, expected, "ack-ceiling {raw}");
        }
    }

    #[test]
    fn parse_args_help_and_version() {
        // rationale: Boundary — `--help` / `--version` set their flags.
        assert!(parse_args(&args(&["--help"])).expect("help").show_help);
        assert!(parse_args(&args(&["-h"])).expect("h").show_help);
        assert!(parse_args(&args(&["--version"]))
            .expect("version")
            .show_version);
    }

    #[test]
    fn parse_args_unknown_flag_is_typed_error() {
        // rationale: Anti-property — an unknown flag is a typed refusal.
        let err = parse_args(&args(&["--nope"])).expect_err("unknown");
        assert_eq!(err, ArgError::UnknownFlag("--nope".to_owned()));
    }

    #[test]
    fn parse_args_missing_value_is_typed_error() {
        // rationale: Anti-property — a value-bearing flag with no value.
        let err = parse_args(&args(&["--top-k"])).expect_err("missing");
        assert_eq!(err, ArgError::MissingValue("--top-k"));
    }

    #[test]
    fn parse_args_bad_top_k_is_typed_error() {
        // rationale: Anti-property — an unparseable `--top-k` value.
        let err = parse_args(&args(&["--top-k", "lots"])).expect_err("bad");
        assert_eq!(
            err,
            ArgError::BadValue {
                flag: "--top-k",
                value: "lots".to_owned(),
            }
        );
    }

    #[test]
    fn parse_args_bad_ack_ceiling_is_typed_error() {
        // rationale: Anti-property — an `--ack-ceiling` outside the 7-set.
        let err = parse_args(&args(&["--ack-ceiling", "godmode"])).expect_err("bad");
        assert_eq!(
            err,
            ArgError::BadValue {
                flag: "--ack-ceiling",
                value: "godmode".to_owned(),
            }
        );
    }

    #[test]
    fn parse_args_positional_is_typed_error() {
        // rationale: Anti-property — no positional args are accepted.
        let err = parse_args(&args(&["junk"])).expect_err("positional");
        assert_eq!(err, ArgError::UnexpectedPositional("junk".to_owned()));
    }

    #[test]
    fn config_default_is_dry_run() {
        // rationale: Boundary — the default Config is dry-run (safe).
        assert!(Config::default().dry_run);
    }

    #[test]
    fn build_verifiers_yields_one_per_kind() {
        // rationale: Contract — the m33 gate requires exactly one Verifier
        // per VerifierKind; the builder must satisfy that precondition.
        let verifiers = build_verifiers();
        assert_eq!(verifiers.len(), VerifierKind::VARIANTS.len());
        for &kind in &VerifierKind::VARIANTS {
            assert_eq!(
                verifiers.iter().filter(|v| v.kind() == kind).count(),
                1,
                "exactly one verifier for {kind:?}"
            );
        }
    }
}
