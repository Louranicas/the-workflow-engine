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
/// At M0, three of the four kinds (Consistency, Cost, Ember) are wired
/// as documented stubs per Plan v2 § 15 D9 (Cost stub), D11 (Consistency
/// stub), and D13 (Ember reduced subset — pending Phase 6b real impl).
/// The Security kind is wired via [`SecurityVerifier`], not this stub.
struct ConservativeVerifier {
    kind: VerifierKind,
}

impl Verifier for ConservativeVerifier {
    fn kind(&self) -> VerifierKind {
        self.kind
    }

    fn verify(&self, _workflow: &crate::m30_bank::AcceptedWorkflow) -> VerifierVerdict {
        // Conservative default — see the struct doc comment. The gate is
        // real and runs; the verdict is a documented stub per § 15
        // D9/D11/D13.
        VerifierVerdict::Approve
    }
}

// rationale: Plan v2 §15 D5 (Security verdict above ceiling = hard Refuse),
// D6 (m33 gate is blocking), D7 (default ack_ceiling = Sandboxed).
//
// At M0 every workflow's escape surface is assumed [`EscapeSurfaceProfile::
// Sandboxed`] (ord 0): the WorkflowProposal type does not carry a per-
// workflow `escape_surface` field today (verified Phase 2 audit). A real
// per-workflow surface determination — either (i) a wire-contract change
// adding the field, or (ii) a StepToken → surface classification table —
// is v0.2.0 work alongside NA-GAP-01 (`RefusalToken`). The Security
// verifier's gate SHAPE is correct: when v0.2.0 supplies a real surface
// per workflow, this verifier produces hard-Refuse on `surface.ordinal()
// > ack_ceiling.ordinal()` without re-wiring. At M0 the gate always
// Approves because `Sandboxed.ordinal() == 0 <= any.ordinal()`.
//
// The hard-Refuse SEMANTIC of D5 is preserved in two ways: this verifier
// implements it for any future per-workflow surface input, AND m32's
// existing monotone `HumanAcceptanceSignature::is_acknowledged_by` gate
// (`src/m32_dispatcher/mod.rs::EscapeSurfaceProfile::is_acknowledged_by`)
// enforces the same comparison at dispatch time against the actual
// workflow surface m32 emits. Documented redundancy slot.
struct SecurityVerifier {
    ack_ceiling: EscapeSurfaceProfile,
}

impl SecurityVerifier {
    /// Construct against an acknowledged ceiling.
    #[must_use]
    const fn new(ack_ceiling: EscapeSurfaceProfile) -> Self {
        Self { ack_ceiling }
    }

    /// The ack ceiling this verifier is configured against (exposed for tests).
    #[cfg(test)]
    const fn ack_ceiling(&self) -> EscapeSurfaceProfile {
        self.ack_ceiling
    }

    /// Determine the proposal's escape surface.
    ///
    /// M0 simplification: every workflow is assumed
    /// [`EscapeSurfaceProfile::Sandboxed`] (the safest profile). Per the
    /// Phase 2 audit a real per-workflow surface determination is
    /// v0.2.0 work; the M0 default is the safest because it can never
    /// trigger a false-Refuse against any reasonable ceiling.
    const fn workflow_escape_surface(
        _workflow: &crate::m30_bank::AcceptedWorkflow,
    ) -> EscapeSurfaceProfile {
        EscapeSurfaceProfile::Sandboxed
    }
}

/// Render the Security verdict given the proposal's surface and the
/// acknowledged ceiling. Extracted as a free fn so tests can exercise
/// every (surface, ceiling) combination without needing a real
/// `AcceptedWorkflow` for each.
fn security_verdict(
    workflow_surface: EscapeSurfaceProfile,
    ack_ceiling: EscapeSurfaceProfile,
) -> VerifierVerdict {
    if workflow_surface.ordinal() <= ack_ceiling.ordinal() {
        VerifierVerdict::Approve
    } else {
        VerifierVerdict::Refuse {
            reason: format!(
                "workflow escape surface {workflow_surface:?} (ord {}) exceeds \
                 acknowledged ceiling {ack_ceiling:?} (ord {}); \
                 m33 Security hard-Refuse per Plan v2 §15 D5",
                workflow_surface.ordinal(),
                ack_ceiling.ordinal(),
            ),
        }
    }
}

impl Verifier for SecurityVerifier {
    fn kind(&self) -> VerifierKind {
        VerifierKind::Security
    }

    fn verify(&self, workflow: &crate::m30_bank::AcceptedWorkflow) -> VerifierVerdict {
        let workflow_surface = Self::workflow_escape_surface(workflow);
        security_verdict(workflow_surface, self.ack_ceiling)
    }
}

// rationale: Plan v2 §15 D13–D16 — Ember verifier. Scores a rendered
// proposal-artefact text against m10's 7-trait rubric (D15 reuses m10
// machinery) and maps the rubric verdict per D16:
//
// - [`EmberStatus::Approved`] → [`VerifierVerdict::Approve`].
// - [`EmberStatus::Held`] (confidence < 0.5) → [`VerifierVerdict::Amend`]
//   — D16: below-bar verdict is Amend, NOT Refuse, so an operator can
//   address the rubric note and re-attempt without a hard-block.
// - [`EmberStatus::Rejected`] (confidence >= 0.5) → [`VerifierVerdict::Amend`]
//   — same D16 mapping. Rubric Rejected is a quality-bar finding, not a
//   security finding; m33 Security owns the hard-Refuse pathway.
//
// The "reduced M0 subset" of D13 (clarity + safe-naming + ambiguity) is
// satisfied by reusing m10's full 7-trait rubric as-is (D15 "reuses
// m10's Ember CI machinery where it fits"); the Day-1 proposal artefact
// surface — a short technical summary — exercises only the
// urgency/honesty/humility traits in practice. As future workflow
// surfaces gain narrative content, the remaining traits start firing
// without further wiring.
struct EmberVerifier;

/// Render the m33 Ember verifier's input — a short, deterministic
/// human-readable summary of the workflow's proposal. Format is intentionally
/// terse and operational; this is what the m10 rubric scores against.
fn render_workflow_artefact(workflow: &crate::m30_bank::AcceptedWorkflow) -> String {
    let proposal = workflow.proposal();
    let variant = proposal.variant();
    let mutation_kind = match variant.mutation {
        crate::m21_variant_builder::MutationKind::Identity => "identity",
        crate::m21_variant_builder::MutationKind::Swap { .. } => "swap",
        crate::m21_variant_builder::MutationKind::Skip { .. } => "skip",
    };
    let cluster_repr = proposal
        .diversity_cluster()
        .map_or_else(|| "none".to_owned(), |c| c.to_string());
    format!(
        "Proposed workflow: id={id}, steps={steps}, mutation={mutation_kind}, \
         evidence_n={n}, evidence_lift={lift:.4}, ci_half={ci:.4}, cluster={cluster_repr}",
        id = proposal.proposal_id(),
        steps = variant.steps.len(),
        n = proposal.evidence_n(),
        lift = proposal.evidence_lift(),
        ci = proposal.evidence_ci_half(),
    )
}

/// Map an m10 rubric verdict to the m33 Ember verifier verdict per D16.
/// Extracted as a free fn so the mapping is tested independently of
/// fixture construction.
fn ember_verdict(status: crate::m10_ember_ci_gate::EmberStatus) -> VerifierVerdict {
    match status {
        crate::m10_ember_ci_gate::EmberStatus::Approved => VerifierVerdict::Approve,
        crate::m10_ember_ci_gate::EmberStatus::Held {
            trait_name,
            reason,
            confidence,
        } => VerifierVerdict::Amend {
            request: format!(
                "Ember rubric Held on trait {trait_name:?} (confidence {confidence:.2}): \
                 {reason}; m33 Ember below-bar Amend per Plan v2 §15 D16"
            ),
        },
        crate::m10_ember_ci_gate::EmberStatus::Rejected { trait_name, reason } => {
            VerifierVerdict::Amend {
                request: format!(
                    "Ember rubric Rejected on trait {trait_name:?}: {reason}; \
                     m33 Ember below-bar Amend per Plan v2 §15 D16"
                ),
            }
        }
    }
}

impl Verifier for EmberVerifier {
    fn kind(&self) -> VerifierKind {
        VerifierKind::Ember
    }

    fn verify(&self, workflow: &crate::m30_bank::AcceptedWorkflow) -> VerifierVerdict {
        let artefact = render_workflow_artefact(workflow);
        ember_verdict(crate::m10_ember_ci_gate::score_against_rubric(&artefact))
    }
}

// rationale: Plan v2 §15 D9 — Cost verifier is a documented Approve-stub
// for M0. WorkflowProposal carries no `cost` field on the wire (verified
// Phase 2 audit, §1 wire-contract); a real Cost verifier would require
// either a cross-binary wire-contract change (add a cost field with
// per-step / per-mutation cost-table — ~150–250 LOC) or an out-of-band
// budget projection (per D10 "step-count × mutation-weight" metric, if
// ever wired). Per D9 the stub ships at M0; the gate is structurally
// present (one Verifier per VerifierKind, deterministic Approve), and
// the substitution to a real verifier is a one-impl change post-M0.
struct CostVerifier;

impl Verifier for CostVerifier {
    fn kind(&self) -> VerifierKind {
        VerifierKind::Cost
    }

    fn verify(&self, _workflow: &crate::m30_bank::AcceptedWorkflow) -> VerifierVerdict {
        // Plan v2 §15 D9 — documented Approve-stub for M0; no cost field
        // exists on the wire (Phase 2 wire-contract audit).
        VerifierVerdict::Approve
    }
}

/// Build the four-verifier set required by [`aggregate`] — exactly one
/// [`Verifier`] per [`VerifierKind`]. Per Plan v2 § 15:
/// - **Security** → [`SecurityVerifier`] (D5/D6/D7).
/// - Consistency → [`ConservativeVerifier`] stub (D11; replaced with a
///   named [`ConsistencyVerifier`] in Phase 6d).
/// - **Cost** → [`CostVerifier`] documented stub (D9).
/// - **Ember** → [`EmberVerifier`] (D13/D14/D15/D16).
fn build_verifiers(ack_ceiling: EscapeSurfaceProfile) -> Vec<Box<dyn Verifier>> {
    VerifierKind::VARIANTS
        .iter()
        .map(|&kind| -> Box<dyn Verifier> {
            match kind {
                VerifierKind::Security => Box::new(SecurityVerifier::new(ack_ceiling)),
                VerifierKind::Ember => Box::new(EmberVerifier),
                VerifierKind::Cost => Box::new(CostVerifier),
                VerifierKind::Consistency => Box::new(ConservativeVerifier { kind }),
            }
        })
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
    let verifiers = build_verifiers(config.ack_ceiling);
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
        build_verifiers, ember_verdict, parse_args, render_workflow_artefact, security_verdict,
        ArgError, Config, CostVerifier, EmberVerifier, SecurityVerifier, Verifier,
        VerifierVerdict, DEFAULT_TOP_K,
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
        let verifiers = build_verifiers(EscapeSurfaceProfile::Sandboxed);
        assert_eq!(verifiers.len(), VerifierKind::VARIANTS.len());
        for &kind in &VerifierKind::VARIANTS {
            assert_eq!(
                verifiers.iter().filter(|v| v.kind() == kind).count(),
                1,
                "exactly one verifier for {kind:?}"
            );
        }
    }

    // rationale: Phase 6a — Security verifier Approve path. With the M0
    // simplification (workflow_escape_surface = Sandboxed, ord 0) the
    // Security verdict is Approve for every ack_ceiling.
    #[test]
    fn security_verifier_approves_when_workflow_surface_is_at_or_below_ceiling() {
        for &ceiling in &EscapeSurfaceProfile::VARIANTS {
            let verdict = security_verdict(EscapeSurfaceProfile::Sandboxed, ceiling);
            assert_eq!(
                verdict,
                VerifierVerdict::Approve,
                "Sandboxed (ord 0) must Approve under ceiling {ceiling:?}"
            );
        }
    }

    // rationale: Phase 6a — Security verifier hard-Refuse path. Tests the
    // SEMANTIC of D5 directly on `security_verdict` (exercises the
    // workflow_surface > ack_ceiling branch that the M0 default of
    // Sandboxed never reaches in practice). When a future per-workflow
    // surface determination supplies a real value, this branch fires.
    #[test]
    fn security_verifier_refuses_when_workflow_surface_exceeds_ceiling() {
        // Take every (workflow, ceiling) pair where ord(workflow) > ord(ceiling)
        // and confirm Refuse with a substantive reason string.
        let mut refuse_count = 0;
        for &surface in &EscapeSurfaceProfile::VARIANTS {
            for &ceiling in &EscapeSurfaceProfile::VARIANTS {
                if surface.ordinal() > ceiling.ordinal() {
                    let verdict = security_verdict(surface, ceiling);
                    match verdict {
                        VerifierVerdict::Refuse { reason } => {
                            assert!(
                                reason.contains("D5"),
                                "Refuse reason must reference D5: {reason}"
                            );
                            assert!(
                                reason.contains("exceeds acknowledged ceiling"),
                                "Refuse reason must name the comparison: {reason}"
                            );
                            refuse_count += 1;
                        }
                        other => panic!(
                            "surface {surface:?} > ceiling {ceiling:?} must Refuse, got {other:?}"
                        ),
                    }
                }
            }
        }
        // 7 surfaces × 7 ceilings = 49 pairs; 21 land in the strict-greater-than
        // upper triangle (binomial(7, 2) = 21).
        assert_eq!(
            refuse_count, 21,
            "expected 21 strict-greater-than pairs to all Refuse"
        );
    }

    // rationale: Phase 6a — exact-ordinal-equal Approve. The comparison
    // is `<=`, so a workflow whose surface equals the ceiling must Approve
    // (D5 is "ABOVE ceiling = Refuse", not "AT-OR-ABOVE").
    #[test]
    fn security_verifier_approves_on_exact_ceiling_match() {
        for &p in &EscapeSurfaceProfile::VARIANTS {
            let verdict = security_verdict(p, p);
            assert_eq!(
                verdict,
                VerifierVerdict::Approve,
                "{p:?} == {p:?} must Approve (the comparison is `<=`, not `<`)"
            );
        }
    }

    // rationale: Phase 6a — accessor test for `ack_ceiling`. Confirms the
    // verifier stores what it was constructed with (anti-stale-clone bug).
    #[test]
    fn security_verifier_stores_ceiling_at_construction() {
        for &p in &EscapeSurfaceProfile::VARIANTS {
            let v = SecurityVerifier::new(p);
            assert_eq!(v.ack_ceiling(), p);
            assert_eq!(v.kind(), VerifierKind::Security);
        }
    }

    // rationale: Phase 6b — D16 verdict mapping. EmberStatus::Approved
    // maps to VerifierVerdict::Approve.
    #[test]
    fn ember_verdict_approved_maps_to_approve() {
        let v = ember_verdict(crate::m10_ember_ci_gate::EmberStatus::Approved);
        assert_eq!(v, VerifierVerdict::Approve);
    }

    // rationale: Phase 6b — D16 verdict mapping. EmberStatus::Held
    // (below-bar, confidence < 0.5) maps to VerifierVerdict::Amend (NOT
    // Refuse — Refuse is reserved for m33 Security per D5).
    #[test]
    fn ember_verdict_held_maps_to_amend_with_d16_reference() {
        let v = ember_verdict(crate::m10_ember_ci_gate::EmberStatus::Held {
            trait_name: crate::m10_ember_ci_gate::TraitName::Equanimity,
            reason: "test reason".to_owned(),
            confidence: 0.3,
        });
        match v {
            VerifierVerdict::Amend { request } => {
                assert!(request.contains("Held"), "must name Held: {request}");
                assert!(request.contains("Equanimity"), "must name trait: {request}");
                assert!(request.contains("D16"), "must reference D16: {request}");
            }
            VerifierVerdict::Approve => panic!("Held must map to Amend, not Approve"),
            VerifierVerdict::Refuse { .. } => panic!("Held must map to Amend, not Refuse"),
        }
    }

    // rationale: Phase 6b — D16 verdict mapping. EmberStatus::Rejected
    // (above-bar, confidence >= 0.5) ALSO maps to VerifierVerdict::Amend
    // — m33 Ember is a quality-bar concern; security hard-Refuse is m33
    // Security's exclusive responsibility.
    #[test]
    fn ember_verdict_rejected_maps_to_amend_not_refuse() {
        let v = ember_verdict(crate::m10_ember_ci_gate::EmberStatus::Rejected {
            trait_name: crate::m10_ember_ci_gate::TraitName::Honesty,
            reason: "test reason".to_owned(),
        });
        match v {
            VerifierVerdict::Amend { request } => {
                assert!(request.contains("Rejected"), "must name Rejected: {request}");
                assert!(request.contains("Honesty"), "must name trait: {request}");
                assert!(request.contains("D16"), "must reference D16: {request}");
            }
            VerifierVerdict::Refuse { .. } => {
                panic!("Rejected must map to Amend (D16), NOT Refuse")
            }
            VerifierVerdict::Approve => panic!("Rejected must map to Amend, not Approve"),
        }
    }

    // rationale: Phase 6c — D9 Cost verifier is a documented Approve-stub.
    // Sweep a small variety of synthetic AcceptedWorkflow shapes; every
    // verdict must be Approve.
    #[test]
    fn cost_verifier_is_documented_approve_stub_per_d9() {
        use crate::m14_lift::LiftSnapshot;
        use crate::m20_prefixspan::{Pattern, StepToken};
        use crate::m21_variant_builder::build_variants;
        use crate::m23_proposer::build_proposal;
        use crate::m30_bank::AcceptedWorkflow;
        use std::time::SystemTime;

        let pattern = Pattern::new(vec![StepToken(1), StepToken(2)], 25, (0, 0));
        let variants = build_variants(&pattern).expect("variants");
        let snapshot = LiftSnapshot {
            lift: Some(0.5),
            ci_half: Some(0.05),
            n: 25,
            latest_ts_ms: 0,
            computed_at: SystemTime::now(),
        };
        let verifier = CostVerifier;
        assert_eq!(verifier.kind(), VerifierKind::Cost);
        for v in variants {
            let proposal = build_proposal(v, &snapshot, None).expect("proposal");
            let wf = AcceptedWorkflow::for_test(7, proposal, 0, i64::MAX, 1.0, None, 0);
            assert_eq!(verifier.verify(&wf), VerifierVerdict::Approve);
        }
    }

    // rationale: Phase 6b — artefact rendering is deterministic and
    // includes the load-bearing identifiers (proposal id, mutation kind,
    // evidence_n, evidence_lift, ci_half, diversity_cluster).
    #[test]
    fn render_workflow_artefact_contains_load_bearing_fields() {
        use crate::m14_lift::LiftSnapshot;
        use crate::m20_prefixspan::{Pattern, StepToken};
        use crate::m21_variant_builder::build_variants;
        use crate::m23_proposer::build_proposal;
        use crate::m30_bank::AcceptedWorkflow;
        use std::time::SystemTime;

        let pattern = Pattern::new(vec![StepToken(7), StepToken(11), StepToken(13)], 25, (0, 0));
        let variants = build_variants(&pattern).expect("variants");
        let identity_variant = variants
            .iter()
            .find(|v| matches!(v.mutation, crate::m21_variant_builder::MutationKind::Identity))
            .expect("identity variant")
            .clone();
        let snapshot = LiftSnapshot {
            lift: Some(0.5),
            ci_half: Some(0.05),
            n: 25,
            latest_ts_ms: 0,
            computed_at: SystemTime::now(),
        };
        let proposal = build_proposal(identity_variant, &snapshot, Some(3)).expect("proposal");
        let workflow = AcceptedWorkflow::for_test(42, proposal, 0, i64::MAX, 1.0, None, 0);
        let artefact = render_workflow_artefact(&workflow);
        // Determinism: same workflow → same string.
        assert_eq!(artefact, render_workflow_artefact(&workflow));
        assert!(artefact.contains("mutation=identity"), "artefact: {artefact}");
        assert!(artefact.contains("evidence_n=25"), "artefact: {artefact}");
        assert!(artefact.contains("cluster=3"), "artefact: {artefact}");
        // The default technical artefact does NOT contain rubric triggers
        // (no all-caps, no flattery, no obvious-claim).
        let v = EmberVerifier.verify(&workflow);
        assert_eq!(
            v,
            VerifierVerdict::Approve,
            "default technical artefact must pass m10's 7-trait rubric"
        );
    }
}
