---
title: BOILERPLATE INDEX — per-file lift map
date: 2026-05-17 (S1001982)
kind: index
status: planning-only · reference-only
---

# BOILERPLATE INDEX — Per-File Lift Map

> Back to: [[../HOME]] · [[../MASTER_INDEX]] · [[../workflow-engine-code-base]] · [[README]]

Per-file table of what to lift, what to adapt, target workflow-trace module, reuse %, risk. Source paths are relative to `~/claude-code-workspace/`.

---

## 01 — CLI Scaffolding

| File | Source | Lift-as-is | Adapt for workflow-trace | Target | Reuse |
|---|---|---|---|---|---|
| `weaver.rs` | `habitat-conductor/src/bin/` | clap `Args` struct, `EnvFilter` setup, tracing JSON output, port/bind/db-path precedence | unify three binaries' CLI into shared DaemonConfig builder; add subcommand dispatch | wf-crystallise / wf-dispatch shell | 70% |
| `enforcer.rs` | `habitat-conductor/src/bin/` | structured error reporting boundary; daemon config defaults | same — pattern is consistent across the 3 conductor binaries | wf-dispatch | 70% |
| `zen.rs` | `habitat-conductor/src/bin/` | tracing + EnvFilter pattern as parallel example (detection-daemon shape) | reference only — wf-crystallise has different sweep cadence | wf-crystallise | 50% |
| `lcm_supervisor.rs` | `loop-engine-v2/src/bin/` | JSON-RPC 2.0 newline-framed dispatch (9 RPC methods, 30s read timeout, SIGTERM); 18-subcommand routing via `m08_cli::m35_cli_root` | replace LCM-domain methods with workflow.* methods (promote/run/verify/decay) | wf-dispatch CLI verbs + m41 lcm_rpc_client | 80% |

## 02 — stcortex Consumer

| File | Source | Lift-as-is | Adapt | Target | Reuse |
|---|---|---|---|---|---|
| `stcortex_subscriber_main.rs` | `stcortex/clients/rust-subscriber/src/main.rs` | `DbConnection::builder()` + `with_uri()` + `with_database_name()` + `on_connect()` callback registration; subscription builder | narrow subscription to `SELECT * FROM tool_call WHERE namespace = <ns>` + `consumption_event` only; remove memory/pathway handlers (W1) | m2 stcortex_consumer | 80% |
| `capacity.rs` | `stcortex/clients/rust-subscriber/src/` | reducer-callback pattern (`Ok(Ok(())) vs Err(...)`); atomic counter aggregation; cross-thread telemetry | strip write paths; keep register_consumer + read callback pattern | m2 stcortex_consumer | 90% |
| `CONSUMER-ONBOARDING.md` | `stcortex/docs/` | refuse-write reducer template (`if consumer_count == 0 && namespace != "scratch"`) | reference — confirms F8/W1 are enforced at DB layer, not Command's responsibility | m9 watcher_namespace_guard | reference |
| `stcortex_API.md` | `stcortex/docs/` | `register_consumer / write_memory / access_memory / context_pack / recall` API signatures + transport enum | reference — workflow-trace uses CLI transport | m2 + m9 | reference |

## 03 — SQLite Multi-DB

| File | Source | Lift-as-is | Adapt | Target | Reuse |
|---|---|---|---|---|---|
| `m06_schema.rs` | `memory-injection/src/m2_schema/` | `open_database()`, `configure_connection()`, idempotent migrate loop, WAL+pragmas, `column_exists()` schema discovery | multi-DB schema init loop (9 tracking DBs + atuin + injection + stcortex JSON snapshot) | m1 atuin_ingest + m3 injection_db_ingest + m13 stcortex_writer | 90% |
| `m07_causal_chain.rs` | `memory-injection/src/m2_schema/` | `CausalChainRow` struct; `auto_resolve_stale_typed()`; per-type TTL thresholds | rename to workflow lifecycle; generalize linear chains to DAG | m7 workflow_arc_record + m11 sunset_lifecycle | 70% |
| `m10_pattern.rs` | `memory-injection/src/m2_schema/` | `PatternRow` (two-counter: `hit_count` vs `natural_hit_count`); decay (`weight *= 0.98`); buoy lifecycle; three-tier equilibrium (Active 0.69 / Buoyed 0.50 / Floor 0.30) | replace pattern keywords with sub-graph fingerprint (tool sequence hash); generalise to N-tuples; pair with fitness signal | m11 + m31 (selection-decay) | 70% |
| `m11_parallel_query.rs` | `memory-injection/src/m3_injection/` | `QueryResult<T>` + `execute_all()` + staleness annotation + cache TTL validation | rename to WorkflowQueryBatch; parametrize via trait | m1 + m3 (multi-substrate read) | 80% |
| `m18_atuin_cache.rs` | `memory-injection/src/m4_consolidation/` | graceful-degrade subprocess wrapper; `read_injection_cache() → Option<T>` with timeout | abstract as FallbackCache<T> trait; generalise beyond atuin | m1 atuin_ingest fallback | 70% |
| `habitat-buoy_schema.rs` | `habitat-buoy/src/schema.rs` | WAL pragmas (`PRAGMA journal_mode = WAL; PRAGMA busy_timeout = 5000;`); bounded LIMIT helpers; query_row(...).optional() | extract PragmaConfig struct; add query timeout wrapper | m13 stcortex_writer + m1/m3 read sides | 50% (small) |

## 04 — Pattern Detection (KEYSTONE GAP — partial reuse only)

| File | Source | Lift-as-is | Adapt | Target | Reuse |
|---|---|---|---|---|---|
| `m49_task_graph.rs` | `orac-sidecar/src/m6_coordination/` | `TaskGraph` + `add_edge()` + `detect_cycles()` (Kahn's topological sort); `TaskNodeState` FSM; `is_terminal()` predicate | nodes = tool-call steps; edges = temporal ordering; add edge-weight (confidence) | m4 cascade_correlator structural backbone | 50% |
| `m20_heat_source_hebbian.rs` | `synthex-v2/src/m4_regulation/` | `HebbianHeatSource::compute()` soft-cap asymptote; CoActivationPair (O(n²) pairwise iteration); fitness_delta weighting; clamp(-1.0, 1.0) | generalize from PAIRWISE to N-step sub-graphs (the KEYSTONE gap); replace fitness_delta scalar with per-edge confidence | m20-m23 (Phase B iteration) | 30% (most is new authorship) |
| `povm-v2_reinforcement.rs` | `povm-v2/src/l5_feedback/` | `ReinforcementSignal { request_id, retrieval_ids, fitness_delta, co_activation_pairs }`; 1h idempotency dedup cache | dedup pattern useful for m13 writes; co_activation_pairs API is the seed of N-step extension | m13 + m20 | 60% |

## 05 — Decay / TTL / LTD (NEW PRIMITIVE GAP — partial reuse)

| File | Source | Lift-as-is | Adapt | Target | Reuse |
|---|---|---|---|---|---|
| `povm-v2_lifecycle.rs` | `povm-v2/src/l3_consolidation/` | `decay_pathways(rate)` + `decay_activations(rate)`; `decay_survived` counter; promotion/demotion gate at ≥5 cycles + ≥20 accesses | the decay primitive is here — needs **fitness signal** wired in for RALPH's `freq×fitness×recency` (NEW PRIMITIVE) | m11 + m31 | 40% (formula is new) |
| `m39_fitness_tensor.rs` | `orac-sidecar/src/m8_evolution/` | 12D fitness tensor with per-dim weights (sum=1.0); rolling-mean smoothing (6-sample = 30s); trend detection (linear regression 10-sample); FMA arithmetic; volatile-dimension mask | extend dimensions: frequency, fitness_delta, recency; smooth only volatile; tensor as gradient-preservation signal | m11 + m6 (cost band fitness) + m31 (selection weighting) | 70% (infrastructure) |
| `ws_inbound_writer.rs` | `synthex-v2/src/daemon/tasks/` | hourly TTL sweep (`DELETE WHERE inserted_at < NOW - RETENTION_MS`); 7d retention; `tokio::select!` shutdown drain; `parking_lot::Mutex<Connection>` Send+Sync; log-and-continue per-row | replace wall-cutoff with gradient-aware cleanup; cut at fitness threshold, not wall | m11 sweep loop | 80% (mechanism) |
| `m16_hebbian_engine.rs` | `memory-injection/src/m4_consolidation/` | 4-step consolidation cycle (decay → buoy → reinforce → prune → auto-resolve) | rename `workflow_consolidation()`; same shape, new domain | m11 + m14 evidence aggregation | 80% |

## 06 — Daemon Scaffolding

| File | Source | Lift-as-is | Adapt | Target | Reuse |
|---|---|---|---|---|---|
| `synthex-v2_daemon_runtime.rs` | `synthex-v2/src/daemon/runtime.rs` (557 LOC) | Tokio multi-thread; 8-task spawn table; per-task `ShutdownBudgets` (5-30s); SIGINT/SIGTERM handlers; `CancellationToken` propagation; `NamedHandle` wrapper | swap 8 tasks for crystalliser sweep + periodic gate + backpressure-check + stcortex-ingest; heartbeat file instead of HTTP /ready | wf-crystallise daemon shell | 80% |
| `synthex-v2_daemon_shutdown.rs` | `synthex-v2/src/daemon/shutdown.rs` (222 LOC) | `install_signal_handlers()` async entry; budget enforcement; named-handle teardown ordering | reuse as-is; daemon shutdown shape is domain-agnostic | wf-crystallise | 95% |
| `synthex_v2.rs` | `synthex-v2/src/bin/` | custom panic hook (file-append, no stderr); tracing_appender non-blocking; jemalloc feature flag | borrow panic-hook + jemalloc for memory-intensive crystallisation | wf-crystallise main | 60% |
| `habitat-nerve-center_main.rs` | `habitat-nerve-center/src/main.rs` (1,337 LOC) | parallel probe loop (30s); shared `AppState` `Arc<RwLock>` + `VecDeque` ring-buffer O(1) eviction; SIGTERM graceful drain; AtomicU64 lock-free reads | replace 11-service probe with single detection sweep; simplify cadence to hourly/weekly | wf-crystallise periodic loop | 60% (structure) |
| `habitat-nerve-center_m3_aggregator_mod.rs` | `habitat-nerve-center/src/m3_aggregator/` | `Arc<RwLock<Option<T>>>` snapshot store; clone-on-read (no lifetime coupling); `AggregatorError` enum | generalize to `Snapshot<T>` for multi-DB federation; add versioning field | m14 evidence_aggregator | 70% |
| `habitat-buoy_engine.rs` | `habitat-buoy/src/engine.rs` | small daemon engine with bounded LIMIT reads | small reference for read-loop sizing | m1/m3 read loops | 40% |

## 07 — Conductor Dispatch (BLOCKED on Wave maturity)

| File | Source | Lift-as-is | Adapt | Target | Reuse |
|---|---|---|---|---|---|
| `conductor_state.rs` | `habitat-conductor/src/state.rs` | `StateDb` constructor; migration framework; `Severity` enum; row-type structs (SnapshotRow, DivergenceReport); WAL setup | replace `weaver_snapshots` + `divergence_reports` tables with workflow_declarations + step_results | m32 dispatcher state-tracking | 70% |
| `conductor_enforcement.rs` | `habitat-conductor/src/enforcement.rs` | `EnforcerAction` enum; `COOLDOWN_SECS`; audit-first writes; per-service rollback tracking | adapt severity→gating from HIGH/CRITICAL to step-outcome levels (pass/fail/blocked) | m32 + m33 (verify gate) | 80% |
| `conductor_api.rs` | `habitat-conductor/src/api.rs` | Axum router + response types; query extractors; shared state `Arc<Mutex>`; build_router() pattern | replace `/state /proposals /divergence` with workflow-trace endpoints (if HTTP added later) | m32 HTTP surface (deferred per HARD REFUSAL) | reference only |
| `conductor_divergence.rs` | `habitat-conductor/src/divergence.rs` | `Rule` trait (kind, severity, cooldown_secs); `DivergenceEvent`; `RuleRegistry` pattern | map `Rule` → step validators (Phase B); `DivergenceEvent` → gate violations | m20-m23 + m32 (Phase B) | 50% (lateral transfer) |
| `m32_tier_executor.rs` | `dev-ops-engine-v3/src/m7_orchestrator/` | `Tier` enum; `GATE_Tn` constants (0.0/0.80/0.85/0.90); checkpoint/resume; adaptive thresholds; confidence formula | replace fitness-based gates with outcome-based gates (PASS/FAIL/DEGRADED per m33 verifier) | m32 + m33 | 60% |

## 08 — Nexus + LCM RPC

| File | Source | Lift-as-is | Adapt | Target | Reuse |
|---|---|---|---|---|---|
| `m22_synthex_bridge.rs` | `orac-sidecar/src/m5_bridges/` | `NexusEvent { event_type, ts, data }` struct + envelope; `NexusPushEnvelope` wrapper; outbox-first dual-transport (JSONL durable + HTTP fire-and-forget); `EmitOutcome { appended, posted, failed }` | extend with WorkflowPromote/Run/Decay event_types (Option A additive untyped per Zen v1.2); Option B = typed enum (S119 hardening) | m40 nexus_event_emitter | 90% |
| `m22_synthex_async.rs` | `orac-sidecar/src/m5_bridges/` | `AsyncOutcome<T>` enum (Success/Failure/CircuitOpen/Timeout); `spawn_blocking` 10s cap; circuit breaker (Closed/Open/HalfOpen); exponential backoff jitter ±25% | reuse for LCM client + SYNTHEX emitter; multi-target circuit-breaker pattern | m40 + m41 | 95% |
| `m24_povm_bridge.rs` | `orac-sidecar/src/m5_bridges/` | `raw_http_post(addr, path, json!)` + `raw_http_get(...)` with proper socket-addr (no `http://` prefix, BUG-033 fix); response parsing for typed structs (Pathway etc.); F-001 silent-swallow fix; BUG-034 explicit `/hydrate` | **GOLD STANDARD per Command-3 E3**: all 3 outbound bridges (stcortex, Conductor, atuin) should crib m24's shape — NOT V3's | m13 + m32 + m41 + m42 | 85% (gold standard) |
| `m38_deployment_api.rs` | `dev-ops-engine-v3/src/m8_api/` | `DeployRequest` / `DeployResponse` struct; `DeployOptions { dry_run, confidence_override }`; `WorkflowId` newtype; MAX_ACTIVE_WORKFLOWS constraint | replace goal-string with WorkflowDecl; adapt options | m32 dispatch API shape (if HTTP added later) | 50% |

## 09 — Trap / Verify / Escape-Surface (per-piece STRONG, unified schema GAP)

| File | Source | Lift-as-is | Adapt | Target | Reuse |
|---|---|---|---|---|---|
| `SKILL-forge.md` | `~/.claude/skills/forge/SKILL.md:225-237` | 8-trap encoding table — `traps: [{ name, how_forge_handles_it }]` | workflow-scoped `traps: [{ id, category, surfaces_at_step, mitigation }]`; m32 surfaces per-step | m32 trap_surfacer | 70% (schema) |
| `SKILL-genesis.md` | `~/.claude/skills/genesis/SKILL.md` | 15+-trap encoding; multi-stage genesis pattern (plan→scaffold→deploy→verify) | reference for workflow-trace's own genesis-eats-dogfood requirement | meta — informs G5/G6 spec process | reference |
| `SKILL-pre-deploy-hardening.md` | `~/.claude/skills/pre-deploy-hardening/SKILL.md:50-125` | 4-agent parallel gate (security + perf + silent-failure + zen); `Verdict { agent, APPROVE/REJECT, evidence }`; pre-flight snapshot + mechanical gate; post-flight receipt | `workflow verify <name>` dispatches 4-agent gate on workflow definition + target codebase; record `VerificationResult { agents_passed[], last_verified_at }` | m33 workflow_verifier | 80% |
| `SKILL-silent-swallow-detect.md` | `~/.claude/skills/silent-swallow-detect/SKILL.md` | 5 anti-patterns P1-P5 with confidence tiers; `SilentFailurePattern { id, name, evidence_template, downstream_harm, confidence_tier }` | classify workflow proposals for silent-failure risk at promotion | m23 workflow_proposer + m30 bank | 60% |
| `SKILL-quality-gate.md` | `~/.claude/skills/quality-gate/SKILL.md` | 4-stage zero-tolerance pipeline (check→clippy→pedantic→test); `set -o pipefail` + PIPESTATUS discipline | universal Phase A invariant; m10 ember_gate runs alongside | applies to all modules during build | 90% |
| `hookify.preserve-blanket-guard.local.md` | `~/claude-code-workspace/.claude/` | PreToolUse hook; blanket-command pattern-matching; 3-check enforcement; bypass env `GUARD_OVERRIDE` | workflows with `destructive` escape-surface trigger PreToolUse hook on every filesystem/network step | Cipher escape-surface + m32 display-before-step | 80% |
| `feedback_preserve_list_discipline.md` | `~/.claude/projects/-home-louranicas-claude-code-workspace/memory/` | Architectural scar tissue from S102 openclaw incident; preserve-list pattern generalises | reference for m30's `escape_surface_profile` schema (Cipher P0 #11) | m30 + Cipher constraint | reference (informs schema) |

## 10 — Foundation Direct Clones (v0 95% reuse)

| File | Source | Lift-as-is | Adapt | Target | Reuse |
|---|---|---|---|---|---|
| `m01_core_types.rs` | `synthex-v2/src/m1_foundation/` | Foundation types; type-aliases; newtypes; identifier wrappers | rename project-specific types (SessionId, ConsumerId, etc.); preserve newtype discipline | v0's m01 / single-phase m1 internal types | **95%** |
| `m02_error_taxonomy.rs` | `synthex-v2/src/m1_foundation/` | Error enum taxonomy; thiserror patterns; context wrapping; trace-id propagation | rename error variants for workflow-trace domain | v0's m02 / single-phase shared lib | **95%** |
| `m05_metrics_collector.rs` | `synthex-v2/src/m1_foundation/` | Metrics collection scaffolding; counter/gauge/histogram; per-module metric registry | swap metric names for workflow-trace domain | v0's m04 / single-phase shared lib | **80%** |
| `m03_config.rs` | `dev-ops-engine-v3/src/m1_core/` | EngineConfig pattern with `Default` + env override + TOML overlay; post-`b7d4abb` bind-discipline (default 127.0.0.1) | rename fields for workflow-trace; preserve bind-discipline | v0's m03 / single-phase shared lib | **95%** |

---

## Cross-cutter notes

### High-leverage source crates (appear across categories)

| Source crate | Categories | Reuse target |
|---|---|---|
| `synthex-v2` | 04, 05, 06, 08, 10 | ~30% of total clone volume; biggest lift source |
| `habitat-conductor` | 01, 07 | dispatch + CLI shape; STRONG-pattern but BLOCKED-on-Wave-maturity |
| `orac-sidecar` | 04, 05, 08 | bridges + fitness tensor + task graph |
| `memory-injection` | 03, 05 | schema + decay + pattern engine |
| `loop-engine-v2` | 01, 08 | LCM CLI dispatch + JSON-RPC supervisor |
| `povm-v2` | 04, 05 | reinforcement + lifecycle decay primitives |
| `dev-ops-engine-v3` | 07, 08, 10 | tier executor + config + deployment API |
| `habitat-nerve-center` | 06 | aggregator + ring buffer |
| `habitat-buoy` | 03, 06 | small WAL daemon reference |
| `stcortex` | 02 | consumer registration + refuse-write |

### Structural gaps NOT covered by any clone (must be authored)

These were identified by the Boilerplate Hunt as cannot-lift gaps:

1. **N-step compositional sub-graph detection with gap-allowed matching + DAG isomorphism** — `m20_heat_source_hebbian.rs` provides the pairwise scaffold (30% reuse) but the N-step generalisation is fresh authorship (~600-1,000 LOC)
2. **`frequency × fitness × recency` compound decay formula** — `povm-v2_lifecycle.rs` + `m39_fitness_tensor.rs` provide the parts; the composition is unbuilt (~200-300 LOC)
3. **Unified `WorkflowSecurityProfile` schema** — Cat 9 has the per-piece pieces (trap-annotations, preserve-list-discipline, silent-failure taxonomy, pre-deploy 4-agent gate) but no unified schema (~150-250 LOC)

### Target module map

Each clone maps to one or more target modules in the single-phase 26-module architecture (see [[../Modules Synergy Clusters and Feature Verification S1001982]]). Cross-reference:

- m1, m2, m3 ← Cat 01, 02, 03
- m4 ← Cat 04 (partial; key new authorship)
- m5, m6 ← Cat 03 + own authorship (cascade/battern/cost specific)
- m7 ← Cat 03 (m07_causal_chain + own)
- m8 ← Cat 07 (conductor_enforcement startup-refusal pattern)
- m9 ← Cat 02 (CONSUMER-ONBOARDING refuse-write)
- m10 ← Cat 09 (Ember gate + skill rubrics)
- m11 ← Cat 03 (m07 + m10) + Cat 05 (lifecycle/decay)
- m12 ← Cat 01 (CLI patterns) + own
- m13 ← Cat 02 (stcortex_writer)
- m14 ← Cat 06 (aggregator pattern)
- m15 ← own + agent-cross-talk emit
- m20-m23 ← Cat 04 (Phase B iteration; partial reuse + structural-gap authorship)
- m30-m33 ← Cat 07 (Phase B dispatch) + Cat 09 (verify gate)
- m40 ← Cat 08 (NexusEvent emitter)
- m41 ← Cat 01 (lcm_supervisor) + Cat 08 (async client)
- m42 ← Cat 08 (POVM reinforce route)
- shared lib ← Cat 10 (95% reuse foundation)

---

## Cloned markdown notes (direct wikilinks)

Most lifted source files are Rust/non-markdown and live in their numbered subdirectories. The markdown clones that document patterns / skills / onboarding are also wikilinked here for graph navigation:

**02 — stcortex Consumer:**
- [[CONSUMER-ONBOARDING]] — refuse-write reducer template; F8/W1 DB-layer enforcement
- [[stcortex_API]] — `register_consumer` / `write_memory` / `access_memory` / `context_pack` / `recall` API + transport enum

**09 — Trap / Verify / Escape-Surface (skill rubrics + feedback):**
- [[SKILL-forge]] — generic build+deploy+verify pipeline (8 habitat traps encoded)
- [[SKILL-genesis]] — end-to-end new-service creation (15+ trap points)
- [[SKILL-pre-deploy-hardening]] — 4-agent parallel pre-deploy gate
- [[SKILL-quality-gate]] — 4-stage zero-tolerance pipeline
- [[SKILL-silent-swallow-detect]] — silent-swallow anti-pattern hunt
- [[feedback_preserve_list_discipline]] — blanket-command preserve-list rebuilds (S102 lesson)
- [[hookify.preserve-blanket-guard.local]] — hookify preserve-list guard config

**Gold Standard Reference profiles** (synthesis: [[Gold Standard Exemplars — Synthesis]]):
- [[Habitat Loop Engine — Gold Standard Reference]]
- [[Maintenance Engine V2 — Gold Standard Reference]]
- [[ORAC Sidecar — Gold Standard Reference]]

---

*BOILERPLATE_INDEX last updated: 2026-05-17 ~11:05 (v1 clone landed)*
