# API_MAP — workflow_core public API reference

> **Back to:** [`../ARCHITECTURE.md`](../ARCHITECTURE.md) · [`../README.md`](../README.md) · [`INDEX.md`](INDEX.md) · [`../GATE_STATE.md`](../GATE_STATE.md) · [`../ultramap/README.md`](../ultramap/README.md)
> **Source of truth:** `src/lib.rs` (`pub use` re-export surface) + `src/orchestration/`. Verified at git `ae7d460` (S1003733 closeout — binaries wired).
> **Purpose:** the single reference a developer consults to answer *"what function do I call, and what is its signature?"* — every `lib.rs`-re-exported identifier, the core-data-type catalogue, and the `orchestration` module surface.

---

## How to read this

`workflow_core` re-exports each module's public surface at the crate root, so a caller writes
`use workflow_core::{mine_sequences, LiftSnapshot, CuratedBank};` rather than reaching into
module paths. Where two modules export the same name, `lib.rs` aliases one (e.g.
`open_readonly` becomes `open_atuin_readonly` / `open_injection_db_readonly`). The tables
below list the **re-exported** identifier — that is the name you actually `use`.

Two modules are **not** root-re-exported and must be reached by full path:
`workflow_core::m8_povm_build_prereq::*` and `workflow_core::user_facing_strings::*`.

---

## 1. Public API by module — `lib.rs` re-exports

Legend: **fn** = free function · **T** = type (struct/enum) · **trait** = trait · **const** = constant.

### Cluster A — Substrate Ingest

#### m1 `m1_atuin_consumer`
| Kind | Re-exported identifier | Notes |
|---|---|---|
| fn | `open_atuin_readonly` | aliased from `open_readonly`; opens a WAL read-only `AtuinConsumer` |
| fn | `expand_tilde` | `&str` → `PathBuf`, expands `~` |
| fn | `canonical_default_path` | the default atuin DB `PathBuf` |
| fn | `fallback_subprocess_ingest` | Day-1 subprocess-ingest stub |
| T | `AtuinConsumer` | cursor-paginated reader; `next_page`, `collect_all`, `into_page_iter` |
| T | `AtuinConsumerConfig` | `page_size`, `row_cap`, `db_path_override`, `subprocess_timeout_ms` |
| T | `AtuinConsumerError` | typed error |
| T | `AtuinHistoryRow` | one shell-history row |
| T | `PageIter` | iterator over `PageResult` |
| T | `PageResult` | `rows`, `last_id`, `exhausted`, `elapsed_ms` |
| T | `SessionId` | newtype |

#### m2 `m2_stcortex_consumer`
| Kind | Re-exported identifier | Notes |
|---|---|---|
| fn | `register_narrowed_consumer` | `(ConsumerIdentity, timeout_ms) -> Result<RegistrationHandle, _>` |
| fn | `tool_call_query` | builds the LIKE-injection-safe `tool_invocation` SELECT |
| fn | `consumption_event_query` | `&'static str` — the `consumption_event` SELECT |
| T | `ConsumerIdentity` | `{ name, namespace, transport }` |
| T | `ConsumerName` | newtype, max 64 chars |
| T | `Namespace` | newtype, must start `workflow_trace_` |
| T | `RegistrationHandle` | `is_fresh()`; implements `FreshnessGate` (CC-2 signal for m13) |
| T | `RegistrationStatus` | registration state enum |
| T | `ConsumptionEventRow`, `StcortexRow`, `ToolCallRow` | subscribed-row shapes |
| T | `Transport` | `Subscription` |
| T | `StcortexConsumerError` | typed error |
| const | `STCORTEX_WORKFLOW_TRACE_PREFIX` | aliased from `WORKFLOW_TRACE_PREFIX` |

#### m3 `m3_injection_db_consumer`
| Kind | Re-exported identifier | Notes |
|---|---|---|
| fn | `open_injection_db_readonly` | aliased from `open_readonly` |
| fn | `parse_causal_chain_row`, `parse_chain_type`, `parse_consent` | row parsers |
| T | `InjectionDbConsumer` | `read_unresolved`, `read_recently_resolved`, `count_unresolved` |
| T | `InjectionDbConfig` | `db_path`, `max_unresolved`, `max_recently_resolved`, `resolved_recency_sessions` |
| T | `CausalChainRow` | one `causal_chain` ledger row |
| T | `ChainId`, `ChainLabel` | newtypes |
| T | `ChainType` | `Bug` \| `Trap` \| `Plan` \| `Pattern` |
| T | `ConsentLevel` | `Emit` \| `Store` \| `Forget` |
| T | `InjectionDbError` | typed error |

### Cluster B — Habitat Observers

#### m4 `m4_cascade`
| Kind | Re-exported identifier | Notes |
|---|---|---|
| fn | `assign_cluster_id` | `&[AtuinStep] -> CascadeClusterId` |
| fn | `fnv1a_64` | `&[u8] -> u64` — shared hash primitive (m20/m21/m23/m30) |
| T | `CascadeCorrelator` | `new(config)`; `correlate(&[AtuinStep], &[DispatchRecord]) -> Vec<CascadeCluster>` |
| T | `CascadeCorrelatorConfig` | `max_gap_ms`, `min_pane_count`, `window_ms`, `max_steps_per_cluster`, `atuin_db_path` |
| T | `AtuinStep` | `{ id, ts_ns, command, cwd, session, exit }` |
| T | `CascadeCluster` | a correlated multi-pane cluster |
| T | `CascadeClusterId` | opaque newtype |
| T | `DispatchRecord` | pane-dispatch record |
| T | `CascadeError` | typed error |

#### m5 `m5_battern`
| Kind | Re-exported identifier | Notes |
|---|---|---|
| fn | `derive_battern_id` | `(first_dispatch_ts_ns, session) -> BatternId` |
| T | `BatternStepRecord` | `new(config)`; `observe(&AtuinStep)`; `summarise() -> BatternRecord` |
| T | `BatternStepRecordConfig` | `min_steps` |
| T | `BatternRecord` | `{ battern_id, steps, is_complete }` |
| T | `BatternStepObservation` | one observed step |
| T | `BatternStepLabel` | closed enum of Battern phases |
| T | `BatternId` | opaque newtype |
| T | `BatternError` | typed error |

#### m6 `m6_cost`
| Kind | Re-exported identifier | Notes |
|---|---|---|
| T | `ContextCostRecord` | `new(config)`; `observe(session_id, cost_tokens, outcome)` |
| T | `ContextCostRecordConfig` | per-session cost config |
| T | `SessionCostRecord` | `new(session_id, cost_tokens)` |
| T | `ExplorationBaseline` | `update(cost)` (EMA, exploration outcomes only); `classify(cost) -> CostBand` |
| T | `WorkflowOutcome` | `Converged`\|`Repeated`\|`Explored`\|`Diverged`\|`Unknown`; `.is_exploration()` |
| T | `CostBand` | `BelowBaseline`\|`NearBaseline`\|`AboveBaseline` |
| T | `ContextCostError` | typed error |

### Cluster C — Correlation + Output

#### m7 `m7_workflow_runs`
| Kind | Re-exported identifier | Notes |
|---|---|---|
| fn | `open_workflow_runs_database` | aliased from `open_database`; WAL SQLite + DDL |
| fn | `open_workflow_runs_memory` | aliased from `open_memory`; in-memory (tests) |
| fn | `insert_run` | `(&Connection, started_at) -> Result<i64, _>` |
| fn | `merge_observation` | `(&Connection, run_id, &ClusterBObservation)` — JSON-patches `consumer_inputs` |
| fn | `update_cost_tokens` | `(&Connection, run_id, cost_tokens)` |
| fn | `close_run` | `(&Connection, run_id, ended_at, outcome)` |
| fn | `find_open` | `(&Connection, limit) -> Result<Vec<WorkflowRunRow>, _>` |
| fn | `find_by_id` | `(&Connection, id) -> Result<WorkflowRunRow, _>` |
| fn | `find_by_outcome` | `(&Connection, Outcome) -> Result<Vec<WorkflowRunRow>, _>` |
| T | `WorkflowRunRow` | `{ id, started_at, run_state, consumer_inputs, cost_tokens, fitness_dimension }` |
| T | `RunState` | `Open` \| `Closed { ended_at, outcome }` |
| T | `ClusterBObservation` | JSONB-keyed: `Cascade`\|`BatternStep`\|`ContextCost`\|`InjectionChain` |
| T | `Outcome` | `Ok`\|`Fail`\|`Abort`\|`Unknown`; `.as_str()` |
| T | `StepOutcome` | `Ok`\|`Fail`\|`Skipped` |
| T | `WorkflowError` | typed error |

#### m12 `m12_cli_reports`
| Kind | Re-exported identifier | Notes |
|---|---|---|
| fn | `render_summary_line` | `&[WorkflowRunRow] -> String` |
| fn | `render_outcome_timeline` | `&[WorkflowRunRow] -> String` |
| fn | `render_cluster_cost_table` | `&[WorkflowRunRow] -> String` |
| fn | `render_cost_histogram` | `&[WorkflowRunRow] -> String` |
| fn | `render_machine` | `(&[WorkflowRunRow], OutputFormat) -> String` |
| T | `OutputFormat` | `Table`\|`Json`\|`NdJson` |

#### m13 `m13_stcortex_writer`
| Kind | Re-exported identifier | Notes |
|---|---|---|
| T | `StcortexWriter` | `new_unchecked`, `with_freshness_gate`; `promote_run(&WorkflowRunRow, namespace_key)` |
| T | `OracHttpReader` | production `LtpDensityReader` impl (ORAC `:8133` GET) |
| trait | `LtpDensityReader` | `read_density() -> Option<f64>` |
| trait | `SubstrateWriter` | `write_memory(&CorrelationMemory) -> Result<i64, _>` |
| trait | `FreshnessGate` | `is_fresh() -> bool` (implemented by `RegistrationHandle`) |
| T | `CorrelationMemory` | the stcortex write payload |
| T | `PromoteOutcome` | `Written`\|`WrittenUnderPressure`\|`Deferred` |
| T | `DeferReason` | `LtpBelowFloor`\|`OracUnreachable`\|`StcortexUnreachable` |
| T | `StcortexWriterError` | typed error |
| const | `LTP_PHASE_1_FLOOR` | `0.015` — defer below this |
| const | `LTP_PHASE_3_TARGET` | `0.10` — write normally at/above |
| const | `MAX_RESPONSE_BYTES` | `1_048_576` (SEC4 body cap) |

### Cluster D — Trust (cross-cutting)

#### m8 `m8_povm_build_prereq` — *not root-re-exported; use `workflow_core::m8_povm_build_prereq::*`*
| Kind | Identifier | Notes |
|---|---|---|
| fn | `resolve_health_url` | honours `POVM_HEALTH_URL` env var |
| T | `HealthClient` | `new(url)`; `probe() -> Result<BandClassification, _>` (KEEP-DORMANT — not called by the pipeline) |
| T | `BandClassification` | `AboveThreshold`\|`BelowThreshold`\|`Indeterminate` |
| T | `BuildPrereqError` | typed error |

#### m9 `m9_watcher_namespace_guard`
| Kind | Re-exported identifier | Notes |
|---|---|---|
| fn | `assert_workflow_trace_namespace` | `&str -> Result<ValidatedNamespace, NamespaceViolation>` |
| fn | `munge_hyphen_slug` | idempotent hyphen→underscore |
| T | `ValidatedNamespace` | opaque write-side evidence newtype |
| T | `NamespaceViolation` | 5-variant refusal error |
| const | `WORKFLOW_TRACE_NS_PREFIX` | `"workflow_trace"` |

#### m10 `m10_ember_ci_gate`
| Kind | Re-exported identifier | Notes |
|---|---|---|
| fn | `score_against_rubric` | `&str -> EmberStatus` |
| fn | `evaluate_string` | `(key, text, &[HeldApproval]) -> GateVerdict` |
| fn | `evaluate_string_at` | as above, with explicit `OffsetDateTime` |
| fn | `load_approvals` | `&Path -> Result<Vec<HeldApproval>, _>` (TSV allowlist) |
| fn | `is_approved`, `is_approved_at` | allowlist lookup |
| T | `EmberStatus` | `Approved`\|`Held { confidence }`\|`Rejected { confidence }` |
| T | `GateVerdict` | `Pass`\|`HeldPass { key }`\|`Fail { key, status }` |
| T | `HeldApproval` | `{ key, approved_by, expires_at }` |
| T | `TraitName` | the 7 Ember traits |
| T | `EmberGateError` | typed error |

#### m11 `m11_fitness_weighted_decay`
| Kind | Re-exported identifier | Notes |
|---|---|---|
| fn | `compute_decay_factor` | Gap 2 formula → `Result<DecayFactor, DecayError>` |
| fn | `recency_factor`, `frequency_factor`, `fitness_factor` | the three formula inputs |
| fn | `run_consolidation_cycle` | decay → reinforce-read → prune → auto-sunset → `SunsetStats` |
| fn | `chrono_now_ms` | `Result<i64, DecayError>` |
| trait | `LifecycleBank` | bank-side decay/sunset interface |
| trait | `PathwayWeightReader` | reads stcortex pathway weights |
| trait | `FrequencyReader` | `run_count(id) -> usize` |
| T | `DecayFactor` | opaque newtype; `.as_f64()` |
| T | `DecayConfig` | `plain_decay_rate`, `recency_half_life_days`, `sunset_threshold`, `prune_threshold` |
| T | `SunsetPhase` | `Active`\|`PrunePending`\|`SunsetExpired` |
| T | `AcceptedWorkflowDecay` | per-workflow decay record |
| T | `SunsetStats` | cycle summary |
| T | `DecayError` | typed error |

### Cluster E — Evidence + Pressure

#### m14 `m14_lift`
| Kind | Re-exported identifier | Notes |
|---|---|---|
| fn | `wilson_ci_half` | `(n_success, n_total) -> Option<f64>` — `None` below `MIN_SAMPLE_SIZE` |
| fn | `cost_lift` | `(baseline, actual) -> Result<f64, LiftError>` |
| T | `LiftAggregator` | `new(config)`; `compute_snapshot(&[WorkflowRunRow]) -> LiftSnapshot` |
| T | `LiftAggregatorConfig` | `window`, `cascade_weight`, `cost_weight` |
| T | `LiftSnapshot` | `{ lift: Option<f64>, ci_half, n, latest_ts_ms, computed_at }` |
| T | `WorkflowId` | newtype; `new(raw)` runs the m9 validator; `from_validated(ValidatedNamespace)` |
| T | `WorkflowLiftContribution` | per-workflow lift delta |
| T | `LiftError` | typed error |
| const | `MIN_SAMPLE_SIZE` | `20` — the F2 floor |
| const | `WILSON_Z` | `1.96` |
| const | `DEFAULT_WINDOW_SIZE`, `DEFAULT_CASCADE_WEIGHT`, `DEFAULT_COST_WEIGHT` | aggregator defaults |

#### m15 `m15_pressure`
| Kind | Re-exported identifier | Notes |
|---|---|---|
| fn | `classify_excerpt` | `&str -> Option<ForbiddenCategory>` |
| fn | `truncate_excerpt` | `(&str, max_chars) -> &str` |
| T | `PressureRegister` | `new(config)`; `record(source, excerpt, section) -> Result<PressureEvent, _>` |
| T | `PressureRegisterConfig` | `output_dir` |
| T | `PressureEvent` | one durable JSONL pressure record |
| T | `ForbiddenCategory` | closed forbidden-verb enum |
| T | `CharterSection` | closed charter-section enum |
| T | `PressureSource` | `AgentReport`\|`SpecPatch`\|`CrossTalk`\|`OperatorPrompt` |
| T | `PressureRegisterError` | typed error |

### Cluster F — Iteration (KEYSTONE)

#### m20 `m20_prefixspan`
| Kind | Re-exported identifier | Notes |
|---|---|---|
| fn | `mine_sequences` | `(&[Vec<StepToken>], MinSupport, MaxGap, max_length) -> Result<Vec<Pattern>, MinerError>` |
| T | `Pattern` | encapsulated; `.steps()`, `.support()`, `.gap_bounds()`, `.canonical_hash()` |
| T | `StepToken` | opaque `u32` token (F11 — no human label) |
| T | `MinSupport` | `new(value) -> Result<Self, _>` — enforces F2 floor at the type level |
| T | `MaxGap` | `new(value)` |
| T | `MinerError` | typed error |
| const | `MIN_SUPPORT_FLOOR` | `2` |
| const | `DEFAULT_MAX_LENGTH`, `DEFAULT_MAX_GAP` | miner defaults |

#### m21 `m21_variant_builder`
| Kind | Re-exported identifier | Notes |
|---|---|---|
| fn | `build_variants` | `&Pattern -> Result<Vec<WorkflowVariant>, VariantBuilderError>` (≤ `MAX_VARIANTS_PER_PATTERN`) |
| T | `WorkflowVariant` | `{ variant_id, steps, mutation, source_pattern_hash }` |
| T | `MutationKind` | `Identity`\|`Swap { at }`\|`Skip { at }` |
| T | `VariantBuilderError` | typed error |
| const | `MAX_VARIANTS_PER_PATTERN` | `8` |

#### m22 `m22_kmeans`
| Kind | Re-exported identifier | Notes |
|---|---|---|
| fn | `kmeans` | `(&[Vec<f64>], &KMeansConfig) -> Result<Vec<ClusteredPoint>, KMeansError>` |
| T | `ClusteredPoint` | `{ coords, cluster }` |
| T | `KMeansConfig` | `k`, `max_iterations`, `convergence_epsilon`, `seed` |
| T | `KMeansError` | typed error |
| const | `DEFAULT_MAX_ITERATIONS`, `DEFAULT_CONVERGENCE_EPSILON` | k-means defaults |

#### m23 `m23_proposer`
| Kind | Re-exported identifier | Notes |
|---|---|---|
| fn | `build_proposal` | `(WorkflowVariant, &LiftSnapshot, Option<usize>) -> Result<WorkflowProposal, ProposerError>` — F2 hard refusal |
| fn | `compose_proposals` | `(&[Pattern], &LiftSnapshot, diversity_of: impl Fn(&WorkflowVariant) -> Option<usize>) -> Vec<WorkflowProposal>` — the CC-4 batch path |
| T | `WorkflowProposal` | encapsulated; `.proposal_id()`, `.variant()`, `.evidence_n()`, `.evidence_lift()`, `.evidence_ci_half()`, `.diversity_cluster()` |
| T | `ProposerError` | typed error |
| const | `PROPOSAL_F2_THRESHOLD` | `20` (= `MIN_SAMPLE_SIZE`) |

### Cluster G — Bank / Select / Dispatch / Verify

#### m30 `m30_bank`
| Kind | Re-exported identifier | Notes |
|---|---|---|
| fn | `workflow_pathway_id` | `u64 -> String` — the CC-5 stcortex pathway identifier |
| T | `CuratedBank` | `new`; `accept`, `get`, `active`, `all_workflows`, `try_apply_decay`, `phase_for`, `record_run` |
| T | `AcceptedWorkflow` | encapsulated; `.workflow_id()`, `.proposal()`, `.weight()`, `.run_count()`, `.last_run_ms()`, `.phase_for()`, `.apply_decay_factor()`, `.is_sunset_expired()` |
| T | `BankError` | typed error |
| const | `DEFAULT_SUNSET_DAYS` | `120` |

#### m31 `m31_selector`
| Kind | Re-exported identifier | Notes |
|---|---|---|
| fn | `select_top_k` | `(&[AcceptedWorkflow], &SelectorConfig, diversity_score: impl Fn(&AcceptedWorkflow) -> f64, now_ms, k) -> Result<Vec<ScoredCandidate>, SelectorError>` |
| T | `SelectorConfig` | `{ alpha, beta, gamma, delta }` — sum = 1.0 (compile-time-asserted) |
| T | `ScoredCandidate` | `{ workflow_id, score, components }` |
| T | `ScoreComponents` | `{ fitness, recency, frequency, diversity }` |
| T | `SelectorError` | typed error |
| const | `DEFAULT_ALPHA`/`BETA`/`GAMMA`/`DELTA` | `0.4`/`0.25`/`0.2`/`0.15` |
| const | `RECENCY_HALF_LIFE_DAYS` | `30.0` |

#### m32 `m32_dispatcher`
| Kind | Re-exported identifier | Notes |
|---|---|---|
| T | `ConductorDispatcher` | `new(client)`; `with_forbidden_proposals`, `with_verifiers`; `dispatch(&AcceptedWorkflow, EscapeSurfaceProfile, &HumanAcceptanceSignature) -> Result<DispatchOutcome, DispatcherError>` |
| trait | `ConductorClient` | `submit(...)`; `dispatch_method() -> &'static str` |
| T | `EscapeSurfaceProfile` | 7-variant closed enum; ordinals 0/10/20/30/40/50/60; `.ordinal()`, `.is_acknowledged_by()` |
| T | `HumanAcceptanceSignature` | `{ interactive_terminal, acknowledged_ceiling }` |
| T | `DispatchOutcome` | `Accepted { conductor_dispatch_id }`\|`Refused { reason }` |
| T | `RefusalReason` | 8-variant refusal enum |
| T | `DispatcherError` | typed error |

#### m33 `m33_verifier`
| Kind | Re-exported identifier | Notes |
|---|---|---|
| fn | `aggregate_verifiers` | aliased from `aggregate`; `(&[&dyn Verifier], &AcceptedWorkflow) -> Result<AggregateVerdict, VerifierError>` |
| trait | `Verifier` | `kind() -> VerifierKind`; `verify(&AcceptedWorkflow) -> VerifierVerdict` |
| T | `VerifierKind` | `Security(0)`\|`Consistency(1)`\|`Cost(2)`\|`Ember(3)`; `VARIANTS: [Self; 4]` |
| T | `VerifierVerdict` | `Approve`\|`Refuse { reason }`\|`Amend { suggestion }`; `.is_blocking()` |
| T | `AggregateVerdict` | `AllApprove`\|`Blocked { per_verifier }` |
| T | `VerifierError` | `MissingVerifier`\|`DuplicateVerifier` |

### Cluster H — Substrate Feedback

#### m40 `m40_nexus_emit`
| Kind | Re-exported identifier | Notes |
|---|---|---|
| fn | `build_nexus_event` | aliased from `build_event`; `(NexusEventKind, Value) -> NexusEvent` |
| T | `HttpNexusClient` | `new(url, timeout)`; production `NexusClient` impl |
| trait | `NexusClient` | `push(&NexusEvent) -> Result<(), NexusEmitError>` |
| T | `NexusEvent` | `{ source, kind, payload, ts_ms }` |
| T | `NexusEventKind` | `WorkflowDispatched`\|`WorkflowCompleted` |
| T | `NexusEmitError` | typed error |
| const | `DEFAULT_NEXUS_URL` | `http://127.0.0.1:8092/v3/nexus/push` |
| const | `DEFAULT_PUSH_TIMEOUT` | 5s |

#### m41 `m41_lcm_rpc`
| Kind | Re-exported identifier | Notes |
|---|---|---|
| T | `HttpLcmClient` | `new(url, timeout)`; `allocate_request_id()`; production `LcmClient` impl |
| trait | `LcmClient` | `loop_create(&LcmLoopCreateParams) -> Result<LcmLoopCreateResult, LcmRpcError>` |
| T | `LcmLoopCreateParams` | `{ workflow_id, conductor_dispatch_id, loop_spec }` |
| T | `LcmLoopCreateResult` | `{ loop_id, created_at_ms }` |
| T | `LcmRpcError` | typed error |
| const | `DEFAULT_LCM_URL` | `http://127.0.0.1:8082/rpc` |
| const | `RPC_METHOD` | `"lcm.loop.create"` |
| const | `DEFAULT_RPC_TIMEOUT` | 10s |

#### m42 `m42_stcortex_emit`
| Kind | Re-exported identifier | Notes |
|---|---|---|
| fn | `emit_feedback` | `(&StcortexWriter, &AcceptedWorkflow, &WorkflowRunRow, namespace_key, HebbianSignal) -> Result<PromoteOutcome, SubstrateEmitError>` — delegates to `m13::promote_run` |
| fn | `signal_for_outcome` | `Option<&str> -> HebbianSignal` |
| fn | `outcome_summary` | `&PromoteOutcome -> String` |
| T | `HebbianSignal` | `Reinforce`\|`Depress` |
| T | `SubstrateEmitError` | typed error |

---

## 2. The `orchestration` module — pipeline driver API

The pipeline logic behind both binaries. Re-exported at the crate root under disambiguating
prefixes (the two sub-modules name their types identically).

### 2.1 `orchestration::crystallise` — re-exported as `Crystallise*` / `*_crystallise`

| Kind | Re-exported identifier | Path identifier | Signature / shape |
|---|---|---|---|
| fn | `parse_crystallise_args` | `parse_args` | `&[String] -> Result<Config, ArgError>` |
| fn | `run_crystallise` | `run` | `&Config -> Result<Report, OrchestrationError>` |
| T | `CrystalliseConfig` | `Config` | `{ atuin_db, injection_db, runs_db, proposals_out, min_support, max_gap, offline, format, show_help, show_version }`; `Default` |
| T | `CrystalliseReport` | `Report` | `{ atuin_rows, injection_chains, cascade_clusters, run_id, observations_merged, lift_window_runs, patterns_mined, proposals_written, stages_skipped, completed }`; `Serialize` + `Display` |
| T | `CrystalliseReportFormat` | `ReportFormat` | `Text`\|`Json` |
| T | `CrystalliseArgError` | `ArgError` | `UnknownFlag`\|`MissingValue`\|`BadValue`\|`UnexpectedPositional` |
| T | `CrystalliseError` | `OrchestrationError` | `Atuin`\|`Injection`\|`WorkflowRuns`\|`Miner`\|`Lift`\|`ProposalsOutput` |

Module constants (full path): `DEFAULT_PROPOSALS_OUT`, `DEFAULT_RUNS_DB`, `DEFAULT_MIN_SUPPORT`
(3), `DEFAULT_MAX_GAP` (5), `DEFAULT_MAX_PATTERN_LENGTH` (8), `HELP_TEXT`.

### 2.2 `orchestration::dispatch` — re-exported as `Dispatch*` / `*_dispatch`

| Kind | Re-exported identifier | Path identifier | Signature / shape |
|---|---|---|---|
| fn | `parse_dispatch_args` | `parse_args` | `&[String] -> Result<Config, ArgError>` |
| fn | `run_dispatch` | `run` | `&Config -> Result<Report, OrchestrationError>` |
| T | `DispatchConfig` | `Config` | `{ proposals_in, top_k, conductor_url, dry_run, ack_ceiling, show_help, show_version }`; `Default` (`dry_run = true`, `ack_ceiling = Sandboxed`) |
| T | `DispatchReport` | `Report` | `{ proposals_loaded, bank_accepted, candidates_selected, verifier_approved, dispatched, dry_run, candidates, completed }`; `Serialize` + `Display` |
| T | `DispatchCandidateOutcome` | `CandidateOutcome` | `{ workflow_id, verifier_approved, disposition }` — disposition is one of `dry-run`/`dispatched`/`refused`/`verifier-blocked` |
| T | `DispatchArgError` | `ArgError` | `UnknownFlag`\|`MissingValue`\|`BadValue`\|`UnexpectedPositional` |
| T | `DispatchError` | `OrchestrationError` | `ProposalsInput`\|`ProposalsParse`\|`Bank`\|`Selector` |

Module constants (full path): `DEFAULT_PROPOSALS_IN`, `DEFAULT_TOP_K` (5),
`DEFAULT_CONDUCTOR_URL`, `CONDUCTOR_DISPATCH_PATH` (`/dispatch`), `CONDUCTOR_TIMEOUT` (10s),
`HELP_TEXT`.

---

## 3. Core data-type catalogue — shape · producer · consumer

| Type | Shape | Produced by | Consumed by |
|---|---|---|---|
| `AtuinHistoryRow` | `{ id, command, session, hostname, timestamp_ms, exit, duration_ms, cwd, deleted_at }` | m1 | m4, m5, orchestration |
| `AtuinStep` | `{ id, ts_ns, command, cwd, session, exit }` | orchestration (`row_to_step`) | m4 |
| `CascadeCluster` | correlated multi-pane cluster (opaque id, window, counts) | m4 | m7 via `ClusterBObservation::Cascade` |
| `BatternRecord` | `{ battern_id, steps, is_complete }` | m5 | m7 via `ClusterBObservation::BatternStep` |
| `CausalChainRow` | `{ id, origin_session, resolved_session, chain_type, label, description, reinforcement_count, consent }` | m3 | m7 via `ClusterBObservation::InjectionChain` |
| `WorkflowRunRow` | `{ id, started_at, run_state, consumer_inputs (JSONB), cost_tokens, fitness_dimension }` | m7 | m12, m13, m14, m42 |
| `RunState` | `Open` \| `Closed { ended_at, outcome }` | m7 | m12, m13, m42 |
| `ClusterBObservation` | discriminant-keyed: `Cascade`\|`BatternStep`\|`ContextCost`\|`InjectionChain` | m4/m5/m6/m3 callers | m7 (`merge_observation`) |
| `LiftSnapshot` | `{ lift: Option<f64>, ci_half, n, latest_ts_ms, computed_at }` | m14 | m23 (F2 gate) |
| `StepToken` | opaque `u32` (FNV-1a; F11 — no human label) | orchestration / m4 | m20, m21 |
| `Pattern` | encapsulated `{ steps, support, gap_bounds, canonical_hash }` | m20 | m21, m23 |
| `WorkflowVariant` | `{ variant_id, steps, mutation, source_pattern_hash }` | m21 | m23 |
| `WorkflowProposal` | encapsulated `{ proposal_id, variant, evidence_n, evidence_lift, evidence_ci_half, diversity_cluster }` | m23 | m30 (`accept`); JSONL bridge payload |
| `AcceptedWorkflow` | encapsulated `{ workflow_id, proposal, accepted_at_ms, sunset_at_ms, weight, last_run_ms, run_count }` | m30 | m31, m32, m33, m42 |
| `ScoredCandidate` | `{ workflow_id, score, components }` | m31 | orchestration (dispatch) |
| `EscapeSurfaceProfile` | 7-variant closed enum, ordinals 0/10/20/30/40/50/60 | caller / CLI `--ack-ceiling` | m32 |
| `HumanAcceptanceSignature` | `{ interactive_terminal, acknowledged_ceiling }` | operator (CLI flag) | m32 |
| `DispatchOutcome` | `Accepted { conductor_dispatch_id }` \| `Refused { reason }` | m32 | orchestration (dispatch) |
| `AggregateVerdict` | `AllApprove` \| `Blocked { per_verifier }` | m33 (`aggregate`) | orchestration (dispatch) |
| `NexusEvent` | `{ source, kind, payload, ts_ms }` | m40 | m40 (`push`) |
| `LcmLoopCreateParams` | `{ workflow_id, conductor_dispatch_id, loop_spec }` | caller | m41 |
| `CorrelationMemory` | the stcortex write payload | m13 | m13 (`SubstrateWriter`) |
| `PromoteOutcome` | `Written`\|`WrittenUnderPressure`\|`Deferred` | m13 | m42, caller |
| `HebbianSignal` | `Reinforce` \| `Depress` | m42 | m42 |
| `WorkflowId` | newtype; guaranteed `workflow_trace_*` prefix | m14 | m13 (write-side) |
| `ValidatedNamespace` | newtype; guaranteed `workflow_trace_*` prefix | m9 | m13, m42 |

---

> **Back to:** [`../ARCHITECTURE.md`](../ARCHITECTURE.md) · [`../README.md`](../README.md) · [`INDEX.md`](INDEX.md) · [`../ultramap/README.md`](../ultramap/README.md)
