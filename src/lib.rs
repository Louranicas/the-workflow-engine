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
    update_cost_tokens, ClusterBObservation, Outcome, StepOutcome, WorkflowError,
    WorkflowRunRow,
};
pub use m12_cli_reports::{
    render_cluster_cost_table, render_cost_histogram, render_machine, render_outcome_timeline,
    render_summary_line, OutputFormat,
};
pub mod m9_watcher_namespace_guard;
pub mod m10_ember_ci_gate;
pub mod m11_fitness_weighted_decay;
pub mod user_facing_strings;

pub use m1_atuin_consumer::{
    canonical_default_path, expand_tilde, fallback_subprocess_ingest,
    open_readonly as open_atuin_readonly, AtuinConsumer, AtuinConsumerConfig,
    AtuinConsumerError, AtuinHistoryRow, PageIter, PageResult, SessionId,
};
pub use m2_stcortex_consumer::{
    consumption_event_query, register_narrowed_consumer, tool_call_query,
    ConsumerIdentity, ConsumerName, ConsumptionEventRow, Namespace, RegistrationHandle,
    StcortexConsumerError, StcortexRow, ToolCallRow, Transport,
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
