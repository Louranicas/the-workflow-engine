//! `workflow_core` — shared library for the `wf-crystallise` and `wf-dispatch`
//! binaries.
//!
//! Per Genesis Prompt v1.3 binding spec § 1 (Cluster-D Trust cross-cutting),
//! Cluster D ships first on G9 fire. m8 is the **floor of the trust regime**:
//! a compile-time gate that ensures no code reads POVM `learning_health` until
//! the CR-2 magnitude-weighted formula is verified live.
//!
//! # Module map (Cluster D Day-1 — m8 → m9 → m10 → m11)
//!
//! - [`m8_povm_build_prereq`] — compile-time + runtime CR-2 verification gate
//!   (ships first; all other Cluster D modules transitively depend on it).
//!
//! Subsequent Cluster D modules (m9 namespace guard, m10 Ember CI gate, m11
//! compound decay) land in this lib in their own commits per the
//! non-negotiable phase-1-framework order.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
// `BuildPrereqError` lives in `error` module; `HealthClient` in `health`. The
// repeated stem is intentional habitat convention (matches LCM / ORAC /
// synthex-v2 module-typing pattern). See CLAUDE.md god-tier standards § 8
// (newtype discipline).
#![allow(clippy::module_name_repetitions)]
// Acronyms (POVM, CR-2, URL, JSON, HTTP) in prose doc comments are habitat
// convention — backticking every occurrence would harm readability for
// minimal correctness gain (the spec docs in `ai_specs/` are the canonical
// reference, not the rustdoc).
#![allow(clippy::doc_markdown)]

pub mod m1_atuin_consumer;
pub mod m2_stcortex_consumer;
pub mod m3_injection_db_consumer;
pub mod m4_cascade;
pub mod m5_battern;
pub mod m6_cost;
pub mod m7_workflow_runs;
pub mod m12_cli_reports;
pub mod m13_stcortex_writer;
pub mod m14_lift;
pub mod m15_pressure;
pub mod m20_prefixspan;
pub mod m21_variant_builder;
pub mod m22_kmeans;
pub mod m23_proposer;
pub mod m30_bank;
pub mod m31_selector;
pub mod m32_dispatcher;
pub mod m33_verifier;
pub mod m40_nexus_emit;
pub mod m41_lcm_rpc;
pub mod m42_stcortex_emit;
pub mod m8_povm_build_prereq;

pub use m4_cascade::{
    assign_cluster_id, fnv1a_64, AtuinStep, CascadeCluster, CascadeClusterId, CascadeCorrelator,
    CascadeCorrelatorConfig, CascadeError, DispatchRecord,
};
pub use m5_battern::{
    derive_battern_id, BatternError, BatternId, BatternRecord, BatternStepLabel,
    BatternStepObservation, BatternStepRecord, BatternStepRecordConfig,
};
pub use m6_cost::{
    ContextCostError, ContextCostRecord, ContextCostRecordConfig, CostBand, ExplorationBaseline,
    SessionCostRecord, WorkflowOutcome,
};
pub use m7_workflow_runs::{
    close_run, find_by_id, find_by_outcome, find_open, insert_run, merge_observation,
    open_database as open_workflow_runs_database, open_memory as open_workflow_runs_memory,
    update_cost_tokens, ClusterBObservation, Outcome, RunState, StepOutcome, WorkflowError,
    WorkflowRunRow,
};
pub use m12_cli_reports::{
    render_cluster_cost_table, render_cost_histogram, render_machine, render_outcome_timeline,
    render_summary_line, OutputFormat,
};
pub use m13_stcortex_writer::{
    CorrelationMemory, DeferReason, FreshnessGate, LtpDensityReader, OracHttpReader,
    PromoteOutcome, StcortexWriter, StcortexWriterError, SubstrateWriter,
    LTP_PHASE_1_FLOOR, LTP_PHASE_3_TARGET, MAX_RESPONSE_BYTES,
};
pub use m14_lift::{
    cost_lift, wilson_ci_half, LiftAggregator, LiftAggregatorConfig, LiftError, LiftSnapshot,
    WorkflowId, WorkflowLiftContribution, DEFAULT_CASCADE_WEIGHT, DEFAULT_COST_WEIGHT,
    DEFAULT_WINDOW_SIZE, MIN_SAMPLE_SIZE, WILSON_Z,
};
pub use m15_pressure::{
    classify_excerpt, pressure_scalar_from_count, truncate_excerpt, CharterSection,
    ForbiddenCategory, PressureEvent, PressureRegister, PressureRegisterConfig,
    PressureRegisterError, PressureSource, PRESSURE_SATURATION_N,
};
pub use m20_prefixspan::{
    mine_sequences, MaxGap, MinSupport, MinerError, Pattern, StepToken, DEFAULT_MAX_GAP,
    DEFAULT_MAX_LENGTH, MIN_SUPPORT_FLOOR,
};
pub use m21_variant_builder::{
    build_variants, MutationKind, VariantBuilderError, WorkflowVariant,
    MAX_VARIANTS_PER_PATTERN,
};
pub use m22_kmeans::{
    extract_variant_features, kmeans, recommended_k_for_variant_count, ClusteredPoint,
    KMeansConfig, KMeansError, DEFAULT_CONVERGENCE_EPSILON, DEFAULT_MAX_ITERATIONS,
    FEATURE_LEVENSHTEIN_NORM, FEATURE_STEP_COUNT_NORM, RECOMMENDED_K_MAX,
};
pub use m23_proposer::{
    build_proposal, compose_proposals, compose_proposals_with_pressure, ProposerError,
    WorkflowProposal, MAX_PRESSURE_CONTRIBUTION, PROPOSAL_F2_THRESHOLD,
};
pub use m30_bank::{
    workflow_pathway_id, AcceptedWorkflow, BankError, CuratedBank, DEFAULT_SUNSET_DAYS,
};
pub use m31_selector::{
    select_top_k, ScoreComponents, ScoredCandidate, SelectorConfig, SelectorError,
    DEFAULT_ALPHA, DEFAULT_BETA, DEFAULT_DELTA, DEFAULT_GAMMA, RECENCY_HALF_LIFE_DAYS,
};
pub use m32_dispatcher::{
    self_dispatch_guard, ConductorClient, ConductorDispatcher, DispatchOutcome,
    DispatcherError, EscapeSurfaceProfile, HumanAcceptanceSignature, RefusalReason,
};
pub use m33_verifier::{
    aggregate as aggregate_verifiers, AggregateVerdict, Verifier, VerifierError,
    VerifierKind, VerifierVerdict,
};
pub use m40_nexus_emit::{
    build_event as build_nexus_event, HttpNexusClient, NexusClient, NexusEmitError,
    NexusEvent, NexusEventKind, DEFAULT_NEXUS_URL, DEFAULT_PUSH_TIMEOUT,
};
pub use m41_lcm_rpc::{
    HttpLcmClient, LcmClient, LcmLoopCreateParams, LcmLoopCreateResult, LcmRpcError,
    DEFAULT_LCM_URL, DEFAULT_RPC_TIMEOUT, RPC_METHOD,
};
pub use m42_stcortex_emit::{
    emit_feedback, outcome_summary, signal_for_outcome, HebbianSignal, SubstrateEmitError,
};
pub mod m9_watcher_namespace_guard;
pub mod m10_ember_ci_gate;
pub mod back_pressure;
pub mod m11_fitness_weighted_decay;
pub mod m16_substrate_drift_canary;
pub mod orchestration;
pub mod refusal_token;
pub mod user_facing_strings;

pub use m1_atuin_consumer::{
    canonical_default_path, expand_tilde, fallback_subprocess_ingest,
    open_readonly as open_atuin_readonly, AtuinConsumer, AtuinConsumerConfig,
    AtuinConsumerError, AtuinHistoryRow, PageIter, PageResult, SessionId,
};
pub use m2_stcortex_consumer::{
    consumption_event_query, register_narrowed_consumer, tool_call_query,
    ConsumerIdentity, ConsumerName, ConsumptionEventRow, Namespace, RegistrationHandle,
    RegistrationStatus, StcortexConsumerError, StcortexRow, ToolCallRow, Transport,
    WORKFLOW_TRACE_PREFIX as STCORTEX_WORKFLOW_TRACE_PREFIX,
};
pub use m3_injection_db_consumer::{
    open_readonly as open_injection_db_readonly, parse_causal_chain_row, parse_chain_type,
    parse_consent, CausalChainRow, ChainId, ChainLabel, ChainType, ConsentLevel,
    InjectionDbConfig, InjectionDbConsumer, InjectionDbError,
};

pub use m9_watcher_namespace_guard::{
    assert_workflow_trace_namespace, munge_hyphen_slug, NamespaceViolation,
    ValidatedNamespace, WORKFLOW_TRACE_NS_PREFIX,
};
pub use m10_ember_ci_gate::{
    evaluate_string, evaluate_string_at, is_approved, is_approved_at, load_approvals,
    score_against_rubric, EmberGateError, EmberStatus, GateVerdict, HeldApproval, TraitName,
};
pub use m11_fitness_weighted_decay::{
    chrono_now_ms, compute_decay_factor, fitness_factor, frequency_factor, recency_factor,
    run_consolidation_cycle, AcceptedWorkflowDecay, DecayConfig, DecayError, DecayFactor,
    FrequencyReader, LifecycleBank, PathwayWeightReader, SunsetPhase, SunsetStats,
};

// The orchestration layer — the pipeline drivers behind the two binaries.
// Re-exported under `crystallise_*` / `dispatch_*` prefixes so the two
// same-named `Config` / `Report` / `run` / `ArgError` / `OrchestrationError`
// types from each sub-module do not collide at the crate root.
pub use orchestration::crystallise::{
    parse_args as parse_crystallise_args, run as run_crystallise, ArgError as CrystalliseArgError,
    Config as CrystalliseConfig, OrchestrationError as CrystalliseError,
    Report as CrystalliseReport, ReportFormat as CrystalliseReportFormat,
};
pub use orchestration::dispatch::{
    parse_args as parse_dispatch_args, run as run_dispatch, ArgError as DispatchArgError,
    CandidateOutcome as DispatchCandidateOutcome, Config as DispatchConfig,
    OrchestrationError as DispatchError, Report as DispatchReport,
};
