> Back to: [[HOME]]

---

**Source:** `/home/louranicas/claude-code-workspace/the_maintenance_engine_v2`  
**Commit:** `552b888`  
**Verified:** Session 085 (2026-05-17)  
**Status:** COMPILED — All 8 layers fully implemented (96,683 LOC, 4,083 tests, 0 clippy warnings)

---

# Maintenance Engine V2 — Gold Standard Reference

Comprehensive scaffolding exemplar for next-generation multi-layered Rust services with Hebbian learning, PBFT consensus, Kuramoto field coherence, and morphogenic adaptation.

---

## 1. One-Line Essence

**Maintenance Engine V2** is an autonomous service orchestrator for the ULTRAPLATE Developer Environment (port 8180): monitors 13+ services via health polling, remediates failures through PBFT-consensus escalation (40 agents, n=3f+1, q=27), learns pathways via Hebbian STDP co-activation (+0.05/call), tracks field coherence via Kuramoto r-monitoring, and adapts morphogenic triggers when |r_delta| > 0.05.

---

## 2. Top-Level Structure

```
maintenance-engine-v2/
├── src/                           # 83 Rust files, 8 layers
│   ├── lib.rs                     # 150L crate root + Tensor12D + prelude
│   ├── main.rs                    # 800L Axum HTTP server, 30+ routes
│   ├── engine.rs                  # 400L MaintenanceEngineV2 orchestrator
│   ├── database.rs                # 200L DatabaseManager (12 SQLite)
│   ├── m1_foundation/             # L1 (20,115 LOC, 12 files)
│   │   ├── shared_types.rs        # Core types: Timestamp, ServiceDef, etc.
│   │   ├── error.rs               # NAM-aware error taxonomy (56 codes)
│   │   ├── config.rs              # Figment TOML+env loader
│   │   ├── logging.rs             # Tracing instrumentation
│   │   ├── metrics.rs             # Prometheus-style counters
│   │   ├── state.rs               # RwLock-based state persistence
│   │   ├── resources.rs           # Resource pooling + cleanup
│   │   ├── signals.rs             # Arc<dyn SignalBus> pub/sub
│   │   ├── tensor_registry.rs     # TensorContributor aggregation
│   │   ├── nam.rs                 # NAM framework (R1-R5)
│   │   └── mod.rs                 # Layer coordinator (2,702L)
│   ├── m2_services/               # L2 (9,161 LOC, 6 files)
│   │   ├── service_registry.rs    # 1,285L registry + discovery
│   │   ├── health_monitor.rs      # 1,130L probe FSM + batch parallel
│   │   ├── lifecycle.rs           # 1,898L start/stop/restart FSM
│   │   ├── resilience.rs          # 2,189L circuit breaker + load balancer
│   │   └── mod.rs                 # Layer coordinator (694L, 20 tests)
│   ├── m3_core_logic/             # L3 (12,065 LOC, 8 files)
│   │   ├── pipeline.rs            # Pluggable remediation pipeline
│   │   ├── remediation.rs         # Action selection + execution
│   │   ├── confidence.rs          # Bayesian confidence scoring
│   │   ├── action.rs              # Action executor FSM
│   │   ├── outcome.rs             # Outcome recorder + feedback
│   │   ├── feedback.rs            # Cross-layer signal routing
│   │   └── mod.rs                 # Coordinator
│   ├── m4_integration/            # L4 (15,413 LOC, 13 files)
│   │   ├── rest.rs                # HTTP client (reqwest)
│   │   ├── grpc.rs                # gRPC client stub
│   │   ├── websocket.rs           # WebSocket client (tokio-tungstenite)
│   │   ├── ipc.rs                 # Unix socket to PV2 bus
│   │   ├── event_bus.rs           # EventBus pub/sub (SSE inbound)
│   │   ├── bridge.rs              # Bridge orchestration
│   │   ├── cascade_bridge.rs      # Context transfer to SYNTHEX v2
│   │   ├── peer_bridge.rs         # Peer discovery + replication
│   │   ├── tool_registrar.rs      # Tool Library registration
│   │   ├── auth.rs                # PBFT agent auth (JWT stubs)
│   │   ├── orac_bridge.rs         # RALPH proposals to ORAC
│   │   └── mod.rs                 # Coordinator
│   ├── m5_learning/               # L5 (12,473 LOC, 10 files)
│   │   ├── hebbian.rs             # 1,100L LTP/LTD co-activation
│   │   ├── stdp.rs                # STDP processor + window timing
│   │   ├── pattern.rs             # Pattern recognition via pathways
│   │   ├── pruner.rs              # Pathway pruning (weight < 0.01)
│   │   ├── consolidator.rs        # Memory consolidation (sleep/replay)
│   │   ├── antipattern.rs         # Anti-pattern detection
│   │   ├── decay_scheduler.rs     # HRS-001 decay audit
│   │   ├── sequence.rs            # Sequence learning
│   │   ├── prediction.rs          # Predictive pathway selection
│   │   └── mod.rs                 # Coordinator
│   ├── m6_consensus/              # L6 (9,931 LOC, 9 files)
│   │   ├── pbft.rs                # Three-phase commit manager
│   │   ├── agent.rs               # Agent role (VALIDATOR/EXPLORER/CRITIC)
│   │   ├── voting.rs              # Vote collector + aggregation
│   │   ├── view_change.rs         # View-change FSM (leader failover)
│   │   ├── dissent.rs             # Dissent tracking (NAM-R3)
│   │   ├── active_dissent.rs      # Active dissent semantics
│   │   ├── quorum.rs              # Quorum calculator (2f+1)
│   │   ├── checkpoint.rs          # Checkpoint persistence
│   │   └── mod.rs                 # Coordinator
│   ├── m7_observer/               # L7 (10,185 LOC, 7 files)
│   │   ├── observer_bus.rs        # Event aggregation bus
│   │   ├── fitness.rs             # Fitness evaluator (NAM aggregator)
│   │   ├── log_correlator.rs      # Cross-layer event correlation
│   │   ├── emergence_detector.rs  # RALPH emergence types
│   │   ├── evolution_chamber.rs   # Mutation testing pre-deploy
│   │   ├── thermal_monitor.rs     # Thermal homeostasis (HRS-001)
│   │   └── mod.rs                 # Coordinator
│   ├── nexus/                     # L8 NEW (6,849 LOC, 7 files)
│   │   ├── field_bridge.rs        # Kuramoto r-tracking
│   │   ├── intent_router.rs       # 12D IntentTensor routing
│   │   ├── regime_manager.rs      # Swarm/Fleet/Armada K-detection
│   │   ├── stdp_bridge.rs         # Tool chain STDP from VMS
│   │   ├── evolution_gate.rs      # Mutation → chamber → accept/reject
│   │   ├── morphogenic_adapter.rs # r_delta > 0.05 adaptation
│   │   └── mod.rs                 # Coordinator
│   └── tools/                     # L1-tier helpers (1,167 LOC, 7 files)
│       ├── tensor_contrib.rs      # TensorContributor trait impl
│       ├── health.rs              # Health check tool
│       └── ...                    # 4 more tool modules
├── migrations/                    # 11 SQL files (schema + indices)
│   ├── 001_service_tracking.sql   # Services, health checks, restarts
│   ├── 002_system_synergy.sql     # Connections, bridges, synergy scores
│   ├── 003_hebbian_pulse.sql      # Pathways, LTP/LTD events
│   ├── 004_consensus_tracking.sql # Rounds, votes, dissent log
│   ├── 005_episodic_memory.sql    # Episodes, contexts, outcomes
│   ├── 006_tensor_memory.sql      # 12D tensor snapshots + deltas
│   ├── 007_performance_metrics.sql# Metrics, aggregations, alerts
│   ├── 008_flow_state.sql         # Flow states, transitions, checkpoints
│   ├── 009_security_events.sql    # Security threats, mitigations
│   ├── 010_workflow_tracking.sql  # Workflows, steps, outcomes
│   └── 011_evolution_tracking.sql # Fitness, mutations, emergence
├── tests/                         # 17 integration test files
│   ├── integration_health.rs      # Health check roundtrip
│   ├── integration_pbft.rs        # Consensus multi-round
│   ├── integration_learning.rs    # Hebbian co-activation
│   └── ...                        # 14 more
├── benches/                       # 8 criterion benchmarks
│   ├── tensor_encoding.rs         # 12D normalization perf
│   ├── health_monitoring.rs       # Probe FSM throughput
│   ├── pbft_consensus.rs          # Three-phase commit latency
│   └── ...                        # 5 more
├── config/                        # 10 TOML config files
│   ├── database.toml              # 12 DB connection strings
│   ├── services.toml              # Service definitions (13+)
│   ├── learning.toml              # STDP parameters
│   ├── consensus.toml             # PBFT n/f/q, agent roles
│   └── ...                        # 6 more
├── Cargo.toml                     # Single-crate workspace
│   ├── lints: forbid unsafe, deny unwrap, warn pedantic
│   ├── features: default=full, api, database, observability
│   ├── deps: tokio, axum, sqlx, serde, parking_lot, tracing
│   └── benches: 8 criterion configs
├── CLAUDE.md                      # 460L bootstrap context
├── CLAUDE.local.md                # 600L session state
├── MASTER_INDEX.md                # 220L module + DB inventory
├── SCAFFOLDING_MASTER_PLAN.md     # 310L V2 architecture blueprint
├── README.md                      # Quick start + architecture
├── maintenance-engine-v2-vault/   # Obsidian vault (bidirectionally linked)
│   ├── HOME.md                    # Vault home + quick nav
│   ├── MASTER_INDEX.md            # 200+ note catalog
│   ├── architecture/              # 6 notes (architecture, constraints, etc.)
│   ├── bridges/                   # 5 notes (ORAC, POVM, PV2, SYNTHEX)
│   ├── databases/                 # 12 notes (1 per DB + shared indices)
│   ├── layers/                    # 7+ notes (L1-L7 detail)
│   └── ...                        # 80+ more structured notes
└── data/                          # 12 SQLite databases (5.9MB)
    └── databases/
        ├── service_tracking.db    # 260KB
        ├── system_synergy.db      # 212KB
        ├── hebbian_pulse.db       # 240KB
        ├── consensus_tracking.db  # 248KB
        ├── episodic_memory.db     # 192KB
        ├── tensor_memory.db       # 164KB
        ├── performance_metrics.db # 204KB
        ├── flow_state.db          # 224KB
        ├── security_events.db     # 256KB
        ├── workflow_tracking.db   # 280KB
        ├── evolution_tracking.db  # 3.6MB (fitness history)
        └── remediation_log.db     # 0B (empty schema)
```

**Key stats:**  
- 83 Rust source files  
- 209 total files (source + docs + configs + tests + migrations)  
- 55 directories  
- 106,074 LOC (source) + 12,000L docs

---

## 3. Layer Architecture

| Layer | Directory | Files | LOC | Modules | Dependencies | Theme |
|-------|-----------|-------|-----|---------|--------------|-------|
| **L1** | `m1_foundation/` | 12 | 20,115 | M00-M08, M43, NAM | — | Error taxonomy, config, logging, metrics, signals, tensor registry |
| **L2** | `m2_services/` | 6 | 9,161 | M09-M12 | L1 | Service registry, health monitoring, lifecycle FSM, resilience (circuit breaker) |
| **L3** | `m3_core_logic/` | 8 | 12,065 | M13-M18 | L1, L2 | Pipeline orchestration, remediation selection, confidence scoring, action executor |
| **L4** | `m4_integration/` | 13 | 15,413 | M19-M24, M42, M46-M47 | L1-L3 | REST/gRPC/WS clients, IPC, event bus, 6 bridges, tool registry, auth |
| **L5** | `m5_learning/` | 10 | 12,473 | M25-M30, M41 | L1-L4 | Hebbian LTP/LTD, STDP timing windows, pattern recognition, pruning, decay scheduler |
| **L6** | `m6_consensus/` | 9 | 9,931 | M31-M36 | L1-L5 | PBFT three-phase (n=40, f=13, q=27), agents, voting, view change, dissent capture, quorum |
| **L7** | `m7_observer/` | 7 | 10,185 | M37-M40, M44-M45 | L1-L6 | Observer bus, fitness eval, log correlation, emergence detection, evolution chamber, thermal |
| **L8** | `nexus/` | 7 | 6,849 | N01-N06 | L1-L7 | Kuramoto r-tracking, intent routing, K-regime detection, STDP bridge, evolution gate, morphogenic adaptation (NEW V2) |
| **Tools** | `tools/` | 7 | 1,167 | 7 TensorContributors | L1 | Helpers for health, remediation, learning, consensus, observer, tensor |

**Dependency Direction:** L1 ← L2 ← L3 ← L4 ← L5 ← L6 ← L7 ← L8 (strict DAG, compile-time enforced by module structure).

**Key Invariants:**
- C1: No upward imports (DAG compile-time verified)
- C2: All trait methods `&self` with `parking_lot::RwLock` interior mutability
- C3: Every module implements `TensorContributor`
- C4: Zero `unsafe`, `unwrap`, `expect`, or clippy warnings (`#![forbid(unsafe_code)]`, `#![deny(...)]`)
- C5: Zero `chrono` or `SystemTime` — use `Timestamp`/`Duration` everywhere
- C6: Signal emissions via `Arc<dyn SignalBus>` on state transitions
- C7: Owned returns through RwLock (never return references)
- C8: Timeouts use `std::time::Duration`
- C9: Existing downstream tests must not break
- C10: 50+ tests per layer minimum (verified: 4,083 total, ~500 per layer)
- C11: Every L4+ module has Nexus field capture (pre/post r)
- C12: All service interactions record STDP co-activation (+0.05/call)

---

## 4. Workspace / Cargo Features

**Cargo.toml Structure:**

```toml
[package]
name = "maintenance_engine_v2"
version = "2.0.0"
edition = "2021"
authors = ["Maintenance Engine Team"]

[lints.rust]
unsafe_code = "forbid"                    # C4

[lints.clippy]
pedantic = { level = "warn" }             # C4
unwrap_used = "deny"                      # C4
multiple_crate_versions = "allow"

[features]
default = ["full"]
full = ["api", "database", "observability"]
api = []
database = []
observability = []

[dependencies]
# Async runtime
tokio = { version = "1.35", features = ["full"] }

# Web framework
axum = "0.7"

# Database
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# Logging and tracing
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json", "fmt"] }

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Concurrency
parking_lot = "0.12"

# Date/time (NOTE: used for external APIs, internal code uses Timestamp)
chrono = { version = "0.4", features = ["serde"] }

# UUID generation
uuid = { version = "1.6", features = ["v4", "serde"] }

# HTTP client
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }

# Tower middleware
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }

[dev-dependencies]
tokio-test = "0.4"
criterion = { version = "0.5", features = ["async_tokio", "html_reports"] }
rand = "0.8"
tempfile = "3.10"
proptest = "1.4"

[[bench]]
name = "tensor_encoding"
harness = false

[[bench]]
# ... 7 more benches
```

**Key Dependencies:**
- **Async:** tokio (full features)
- **Web:** axum 0.7
- **Database:** sqlx + sqlite at runtime
- **Concurrency:** parking_lot RwLock (zero std::sync::Mutex)
- **Logging:** tracing + tracing-subscriber
- **Serialization:** serde + serde_json
- **Config:** toml + figment (external)

**Build Profile (`[profile.release]`):**
- `lto = true` — link-time optimization
- `codegen-units = 1` — single codegen (slower compile, faster runtime)
- `panic = "abort"` — no unwinding
- `strip = true` — remove symbols
- `opt-level = 3` — aggressive optimization

---

## 5. Capabilities Catalogue

**HTTP Endpoints (30+):**
1. `GET /api/health` — JSON health check
2. `GET /api/status` — Full engine status
3. `GET /api/engine` — Engine health report
4. `GET /api/services` — Service mesh overview
5. `GET /api/layers` — Per-layer health breakdown
6. `GET /api/consensus` — PBFT consensus state
7. `GET /api/learning` — Hebbian learning state
8. `GET /metrics` — Prometheus metrics
9. `POST /api/remediate` — Manual remediation trigger
10. `POST /api/pipeline` — Pipeline submission
11. `GET /api/pathways` — Hebbian pathway dump
12. `POST /api/vote` — PBFT vote submission
13. `GET /api/dissent` — Dissent log
14. `GET /api/evolution` — Evolution chamber results
15. `POST /api/evolution/test` — Mutation test
16. `GET /api/thermal` — Thermal state
17. `POST /api/thermal/reset` — Reset thermal model
18. `GET /api/events` — SSE event stream
19. `POST /api/events/subscribe` — Event subscription
20. `GET /api/tensor` — 12D tensor snapshot
21. `GET /api/nexus/field` — Kuramoto r-tracking
22. `POST /api/nexus/regime` — K-regime query
23. `GET /api/bridges` — Bridge health/status
24. `POST /api/bridge/orac/sync` — Force ORAC sync
25. `POST /api/tool/register` — Tool registration
26. `GET /api/tool/status` — Tool availability
27. `POST /api/cascade/handoff` — Context handoff
28. `GET /api/databases/health` — Database integrity
29. `POST /api/databases/compact` — Compact DBs
30. `GET /api/version` — Service version

**Learning Mechanisms:**
- Hebbian pathway management (LTP +0.1, LTD -0.05)
- STDP co-activation (+0.05 per service interaction, C12)
- Pathway pruning (weight < 0.01)
- Memory consolidation (sleep/replay)
- Anti-pattern detection (trap matching)
- Decay auditing (HRS-001, decay_rate=0.1)
- Cross-session persistence (POVM bridge)

**Consensus & Quorum:**
- PBFT manager (n=40, f=13, q=27)
- Three-phase commit (pre-prepare → prepare → commit)
- View change FSM (leader election on timeout)
- Dissent tracking (minority opinion capture, NAM-R3)
- Checkpoint persistence (rollback to checkpoint on view change)
- Agent role assignment (VALIDATOR/EXPLORER/CRITIC/INTEGRATOR/HISTORIAN)

**Observer Bus & Fitness:**
- Cross-layer event correlation
- Emergence detection (RALPH types: Coherence, Complexity, Cascade)
- Evolution chamber (3-tier: Hypothetical, Candidate, Deployed)
- Thermal homeostasis (SYNTHEX bridge input, target r~0.7)
- Fitness aggregator (NAM compliance scoring)

**Bridges & Integration:**
1. **ORAC Bridge (M53):** 30s interval, RALPH proposals, evolution signals
2. **Cascade Bridge:** 15s to `:8092/v3/diagnostics`, context transfer
3. **PV2 Field Bridge (N01):** On L4+ ops, Kuramoto r-tracking
4. **POVM Bridge (M25):** On co-activation, Hebbian pathway persistence
5. **Peer Bridge (M24b):** On demand, peer discovery + replication
6. **EventBus SSE:** Inbound `:8180/api/events`, Server-Sent Events
7. **Circuit breakers:** All bridges (Closed → Open → HalfOpen)

**Tool Library Integration:**
- Tool registration (MCP-compatible)
- Tool invocation (async execution)
- Tool result collection + embedding
- Tiered availability (T0-T9)

**Persistence & Databases:**
- 12 SQLite databases (5.9MB)
- 11 migrations (schema + indices)
- Atomic transactions per operation
- Checkpoint persistence (PBFT)
- State snapshots (evolution chamber)

---

## 6. Persistence + Databases

**12 SQLite Databases** (under `data/databases/`):

| # | Database | Size | Purpose | Key Tables | Index Count |
|---|----------|------|---------|-----------|------------|
| 1 | service_tracking.db | 260KB | Service lifecycle, health history | `services`, `health_checks`, `restarts` | 5 |
| 2 | system_synergy.db | 212KB | Cross-system integration scoring | `connections`, `bridges`, `synergy_scores` | 4 |
| 3 | hebbian_pulse.db | 240KB | Neural pathway learning | `pathways`, `ltp_events`, `ltd_events` | 6 |
| 4 | consensus_tracking.db | 248KB | PBFT consensus rounds | `rounds`, `votes`, `dissent_log`, `checkpoints` | 5 |
| 5 | episodic_memory.db | 192KB | Episode recording | `episodes`, `contexts`, `outcomes` | 4 |
| 6 | tensor_memory.db | 164KB | 12D tensor storage | `tensors`, `snapshots`, `deltas` | 3 |
| 7 | performance_metrics.db | 204KB | Performance tracking | `metrics`, `aggregations`, `alerts` | 4 |
| 8 | flow_state.db | 224KB | Flow state transitions | `states`, `transitions`, `checkpoints` | 3 |
| 9 | security_events.db | 256KB | Security monitoring | `events`, `threats`, `mitigations` | 3 |
| 10 | workflow_tracking.db | 280KB | Workflow orchestration | `workflows`, `steps`, `outcomes` | 4 |
| 11 | evolution_tracking.db | 3.6MB | Evolution tracking | `fitness`, `mutations`, `emergence` | 8 |
| 12 | remediation_log.db | 0B | Remediation actions | `actions`, `outcomes`, `confidence` | — |

**Migrations** (`migrations/` directory, 11 files, 240KB):
- `001_service_tracking.sql` — Service registry + health checks
- `002_system_synergy.sql` — Bridge connection tracking
- `003_hebbian_pulse.sql` — Pathway LTP/LTD events
- `004_consensus_tracking.sql` — PBFT rounds, votes, dissent
- `005_episodic_memory.sql` — Episode recording
- `006_tensor_memory.sql` — 12D tensor snapshots
- `007_performance_metrics.sql` — Metrics aggregation
- `008_flow_state.sql` — Flow state FSM
- `009_security_events.sql` — Security threat log
- `010_workflow_tracking.sql` — Workflow DAG
- `011_evolution_tracking.sql` — Fitness + mutations

**Schema Highlights:**
- All tables have `id` (primary key), `created_at`, `updated_at` (timestamps)
- Foreign key constraints between related tables (e.g., votes → rounds)
- Unique indices on service_id, pathway pairs, vote tuples
- Composite indices for common queries (service + health, round + phase)
- No raw SQL string concatenation — sqlx macros (compile-time verification)

**Transactional Pattern:**
```rust
let tx = db.begin().await?;
// ... multiple inserts/updates ...
tx.commit().await?;
```

---

## 7. Bridges + Integrations

**5 Outbound Bridges:**

| Bridge | Module | Target | Protocol | Interval | Use Case | Circuit Breaker |
|--------|--------|--------|----------|----------|----------|-----------------|
| **ORAC** | M53 `orac_bridge.rs` | `:8133` | HTTP POST | 30s | RALPH proposals, evolution signals | Open after 10 failures |
| **Cascade** | `cascade_bridge.rs` | `:8092/v3/diagnostics` | HTTP | 15s | Context transfer to SYNTHEX v2 | Open after 5 failures |
| **PV2 Field** | N01 `field_bridge.rs` | `:8132/health` | HTTP/IPC | On L4+ ops | Kuramoto r-tracking, coherence field | Synthetic floor r=0.0 when Open |
| **POVM** | M25 `hebbian.rs` | `:8125` | HTTP POST | On co-activation | Hebbian pathway persistence | Circuit breaker + retry |
| **Peer Bridge** | M24b `peer_bridge.rs` | Discovered peers | HTTP gRPC | On demand | Peer discovery, replication | Per-peer tracking |

**1 Inbound Bridge:**
- **EventBus SSE:** `event_bus.rs` → `:8180/api/events` — Server-Sent Events stream for external subscribers

**Protocol Details:**
- ORAC: `POST /api/ralph/proposals` (JSON), expects `{ ok, proposal_id }`
- Cascade: `POST /v3/diagnostics` (JSON thermal + tensor), expects `{ ok, action }`
- PV2: `GET /health` (JSON) + Unix socket IPC to `/run/user/1000/pane-vortex-bus.sock`
- POVM: `POST /pathways` (NDJSON), expects `{ id }`
- Peer: Service discovery via mDNS + gRPC reflection

**Circuit Breaker States:**
```
Closed (healthy)
  → Open (consecutive_failures >= threshold)
    → HalfOpen (timeout elapsed, test probe)
      → Closed (probe succeeds) or Open (probe fails)
```

**Integration with Other Services:**
- **ORAC (8133):** Bidirectional evolution feedback (proposals ↔ results)
- **PV2 (8132):** Kuramoto field state (r, K, sphere coupling)
- **SYNTHEX v2 (8092):** Thermal homeostasis input (cascade bridge)
- **POVM (8125):** Hebbian pathway persistence (learning bridge)
- **Nerve Center (8083):** Health aggregation (inbound from Nerve)

---

## 8. Test Discipline

**Test Count & Distribution:**
- **Total:** 4,083 passing unit + integration tests
- **Per layer:** ~500 tests (L1: 625, L2: 279, L3-L7: 300-350 each)
- **Layout:** Module-level `#[cfg(test)] mod tests { ... }` in each .rs file + `/tests/` integration suite

**Test Pattern:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_some_behavior() {
        let obj = construct_test_object();
        assert_eq!(obj.method(), expected);
    }

    #[tokio::test]
    async fn test_async_operation() {
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

**Integration Tests** (`tests/` directory, 17 files):
- `integration_health.rs` — Health check roundtrip
- `integration_pbft.rs` — Consensus multi-round
- `integration_learning.rs` — Hebbian co-activation
- `integration_cascade.rs` — Context handoff
- `integration_bridges.rs` — Bridge health + circuit breaker
- `integration_evolution.rs` — Mutation chamber
- `integration_resilience.rs` — Failure recovery
- ... 10 more

**Criterion Benchmarks** (`benches/`, 8 files):
- `tensor_encoding.rs` — 12D normalization + byte serialization
- `health_monitoring.rs` — Probe FSM throughput
- `pbft_consensus.rs` — Three-phase commit latency
- `hebbian_learning.rs` — Pathway co-activation
- `pipeline_execution.rs` — Remediation pipeline latency
- `service_registry.rs` — Service lookup throughput
- `database_operations.rs` — SQLite transaction latency
- `lock_contention.rs` — RwLock contention under concurrent load

**Test Assertions:**
- All tests use `assert!`, `assert_eq!`, `assert_ne!` (zero `.unwrap()` in tests)
- Async tests via `#[tokio::test]`
- Property testing via `proptest` (random inputs, invariant checking)
- Snapshot testing (golden snapshots for state serialization)

---

## 9. Vault / Obsidian Discipline

**Vault Structure:** `maintenance-engine-v2-vault/` (80+ notes, bidirectionally linked)

```
maintenance-engine-v2-vault/
├── HOME.md                    # Quick nav + structure overview
├── MASTER_INDEX.md            # Complete note catalog
├── architecture/              # 6 notes
│   ├── Architecture Overview.md
│   ├── 8-Layer DAG.md
│   ├── 12D Tensor Encoding.md
│   ├── 12 Database Topology.md
│   ├── NAM Compliance Framework.md
│   └── Engine Orchestrator.md
├── bridges/                   # 5 notes
│   ├── Bridge Topology.md
│   ├── Bridge — ME to ORAC.md
│   ├── Bridge — ME to POVM.md
│   ├── Bridge — ME to Pane-Vortex.md
│   └── Bridge — ME to SYNTHEX.md
├── databases/                 # 12+ notes
│   ├── DB — service_tracking.md
│   ├── DB — hebbian_pulse.md
│   ├── ... (one per database)
├── layers/                    # 7+ notes
│   ├── L1 — Foundation.md
│   ├── L2 — Services.md
│   ├── ... (one per layer + subs)
└── ... (80+ total)
```

**Bidirectional Linking Pattern:**
- `HOME.md` → [[Layer Notes]] → [[Module Index]]
- Each module note → [[Layer Note]] + [[MASTER_INDEX]]
- Bridge notes → [[Service Notes]] (e.g., [[Bridge — ME to ORAC]] → [[ORAC Sidecar — Vault Home]])
- POVM pathway mirror: namespace `me_v2_claude_md_*` (6 pathways at `:8125/pathways`)

**MOC (Map of Contents) Files:**
- `HOME.md` — L1 index (quick nav)
- `MASTER_INDEX.md` — L2 comprehensive catalog with statistics
- `architecture/Architecture Overview.md` — 8-layer DAG with module tree

**Schematic Notes:**
- 12 Mermaid diagrams (system overview, FSMs, ER diagrams, layer DAG)
- Each with ````mermaid` code block + textual description
- Regenerated via `mcp__mermaid__generate` when architecture changes

**Cross-Vault References:**
- Main vault: `[[The Maintenance Engine V2]]`, `[[POVM Engine]]`, `[[Nexus Controller V2]]`
- ORAC vault: `[[ORAC Sidecar — Vault Home]]`
- SYNTHEX vault: `[[synthex-v2 MASTER_INDEX]]`

---

## 10. CLAUDE.md / Governance Pattern

**CLAUDE.md Structure** (460 lines):

```markdown
# The Maintenance Engine V2 — Bootstrap Context

> Back to: (links to related docs)
> Status: COMPILED | Modules: 48 | Layers: 8 | LOC: 96,683 | Tests: 4,083

## Overview
[Executive summary + V2 enhancements]

## Quick Commands
[Build, test, run, health check]

## Architecture: 8 Layers, 48+ Modules
[ASCII tree with LOC/module count per layer]

## Design Constraints (C1-C12)
[Table: ID, Constraint, Enforcement]

## 12D Tensor Encoding
[Table: Dimension, Name, Range, Contributors]

## 12 Databases
[Table: Database, Purpose, Key Tables]

## PBFT Configuration
[n/f/q constants + agent roles]

## STDP Learning Parameters
[LTP/LTD rates, decay, healthy_ratio]

## Nexus Integration (L8 — NEW in V2)
[Kuramoto parameters, field capture pattern, evolution gate]

## Escalation Tiers
[L0-L3 conditions, timeouts, actions]

## NAM Compliance (Target: 95%)
[R1-R5 requirements, target %, descriptions]

## Quality Gates
[Table: Gate, Requirement, Status]

## Key Architectural Patterns (from Gold Standard)
[Code blocks: builder, Result everywhere, interior mutability, TensorContributor, signal emission, scoped locks]

## Anti-Patterns (NEVER)
[List: unsafe, unwrap, panic, println, chrono, unbounded channels]

## Bootstrap (Fresh Context Window)
[atuin scripts run habitat-bootstrap, then read this file]

## Cross-Reference Map
[Table: Need, Path]

## Obsidian Vaults
[Project vault, internal layer vault, main vault notes]

## POVM V2 Pathway Mirror
[Namespace me_v2_claude_md_*, pathway list with anchors]
```

**Governance Enforced:**
1. **Compile-time:** `#![forbid(unsafe_code)]`, `#![deny(clippy::unwrap_used)]`, layer DAG
2. **Pre-commit hooks:** 4-stage gate (check → clippy → pedantic → test)
3. **Documentation:** Every module has `///` docs + `#[doc]` attributes
4. **Testing:** 50+ tests per layer minimum (CI gate)
5. **Quality:** Zero clippy warnings (pedantic + nursery) before merge

**Session State File** (`CLAUDE.local.md`):
```markdown
# Local Development Context

## Status (Updated Session NNN)
[JSON status + per-metric table]

## Implementation Status
[Checklist of phases + items]

## Module Map (48+ Modules)
[Per-layer tables with LOC + test counts]

## Database Inventory (12 Databases)
[Table: Database, Size, Rows, Status]

## Gold Standard Patterns (from M1+M2)
[Traits, architectural patterns, tensor contribution map]

## Architecture Constants
[Rust const definitions for PBFT, STDP, Nexus, escalation]

## Anti-Patterns (Never Do)
[Checklist with reasons]

## Cross-Reference Map
[Paths to other docs + services]

## ULTRAPLATE Health Endpoints
[curl commands for all 11+ services]

## Habitat Bootstrap Protocol
[Commands to run at session start in order]
```

---

## 11. Patterns to Emulate for Scaffolding

**Gold-Standard Patterns (Copy-Adaptable):**

### 1. **Module Structure (every layer)**
```rust
// src/m{N}_{name}/mod.rs
pub mod module_a;
pub mod module_b;
pub mod module_c;

pub use module_a::ModuleA;
pub use module_b::ModuleB;
pub use module_c::ModuleC;

/// Layer coordinator
pub struct Layer {
    a: Arc<RwLock<ModuleA>>,
    b: Arc<RwLock<ModuleB>>,
    c: Arc<RwLock<ModuleC>>,
}

impl Layer {
    pub async fn new() -> Result<Self> { ... }
    pub async fn health(&self) -> Result<Health> { ... }
}
```

### 2. **Error Type (NAM-aware)**
```rust
// m1_foundation/error.rs
#[derive(Debug, thiserror::Error)]
pub enum MaintenanceError {
    #[error("Service not found: {service_id}")]
    ServiceNotFound { service_id: String },
    
    #[error("Database error: {reason}")]
    DatabaseError { reason: String },
    
    #[error("PBFT consensus failed: {reason}")]
    ConsensusError { reason: String },
    
    // ... 50+ codes
}

pub type Result<T> = std::result::Result<T, MaintenanceError>;
```

### 3. **Config Loader (Figment)**
```rust
// m1_foundation/config.rs
#[derive(Deserialize, Clone)]
pub struct Config {
    pub services: Vec<ServiceDef>,
    pub consensus: ConsensusConfig,
    pub learning: LearningConfig,
}

impl Config {
    pub fn load() -> Result<Self> {
        Figment::from(Serialized::defaults(Config::default()))
            .merge(Toml::file("config/config.toml"))
            .merge(Env::prefixed("ME_V2_"))
            .extract()
            .map_err(|e| MaintenanceError::ConfigError { reason: e.to_string() })
    }
}
```

### 4. **Trait Pattern (all modules)**
```rust
/// Every module implements TensorContributor
pub trait TensorContributor: Send + Sync {
    fn contribute_tensor(&self) -> TensorContribution;
}

/// Every layer implements these
pub trait HealthOps: Send + Sync {
    async fn health(&self) -> Result<Health>;
}

// Trait methods always &self (interior mutability via RwLock)
impl HealthOps for SomeModule {
    async fn health(&self) -> Result<Health> {
        let inner = self.inner.read();
        // ... read-only ops ...
        Ok(health)
    }
}
```

### 5. **Signal Bus (pub/sub across layers)**
```rust
// m1_foundation/signals.rs
pub trait SignalBus: Send + Sync {
    fn emit(&self, signal: Signal);
    fn subscribe(&self, handler: Arc<dyn SignalHandler>) -> Result<()>;
}

pub enum Signal {
    HealthChanged { service_id: String, old: Health, new: Health },
    ConsensusRound { round_id: u64, phase: &'static str },
    PathwayUpdated { pathway_id: u64, weight: f64 },
    // ... 20+ signal types
}

// Usage
self.signal_bus.emit(Signal::HealthChanged { service_id: "svc1".into(), old, new });
```

### 6. **Scoped Lock Guard (early drop)**
```rust
// Correct pattern
{
    let guard = self.inner.read();
    let result = expensive_computation(&*guard);
    drop(guard);  // Explicitly release lock
}
// Lock is released before returning

// Avoid long-lived guards
let guard = self.inner.read();
spawn_task(async move {
    use_guard(&*guard);  // WRONG: guard moves into async, held across await
}).await;
```

### 7. **Builder Pattern (all constructors)**
```rust
pub struct ServiceDefinition {
    id: String,
    name: String,
    tier: u8,
    port: u16,
}

impl ServiceDefinition {
    pub fn builder(id: impl Into<String>, name: impl Into<String>) -> ServiceDefinitionBuilder {
        ServiceDefinitionBuilder::new(id, name)
    }
}

pub struct ServiceDefinitionBuilder { ... }

impl ServiceDefinitionBuilder {
    pub fn tier(mut self, tier: u8) -> Self { self.tier = tier; self }
    pub fn port(mut self, port: u16) -> Self { self.port = port; self }
    pub fn build(self) -> Result<ServiceDefinition> { ... }
}

// Usage
let svc = ServiceDefinition::builder("svc1", "My Service")
    .tier(2)
    .port(8080)
    .build()?;
```

### 8. **FMA for Float Precision**
```rust
// Avoid
let val = 0.3 * a + 0.25 * b + 0.2 * c;

// Correct (fused multiply-add)
let val = 0.3f64.mul_add(a, 0.25f64.mul_add(b, 0.2 * c));
```

### 9. **Database Transaction**
```rust
let mut tx = db.begin().await?;
sqlx::query!("INSERT INTO ...")
    .execute(&mut *tx)
    .await?;
sqlx::query!("INSERT INTO ...")
    .execute(&mut *tx)
    .await?;
tx.commit().await?;
```

### 10. **Axum Handler Pattern**
```rust
use axum::extract::State;

struct AppState {
    engine: Engine,
    db: Arc<DatabaseManager>,
}

async fn health_handler(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let health = state.engine.health().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(health))
}

// Router setup
let app = Router::new()
    .route("/health", get(health_handler))
    .with_state(Arc::new(state));
```

### 11. **Timestamp (not chrono::DateTime)**
```rust
// m1_foundation/shared_types.rs
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Timestamp(u64);  // Unix seconds

impl Timestamp {
    pub fn now() -> Self { Timestamp(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()) }
    pub fn elapsed(&self) -> Duration { Duration::from_secs(Self::now().0 - self.0) }
}

#[derive(Clone, Copy, Debug)]
pub struct Duration(u64);  // Milliseconds

// Never use chrono::DateTime internally (only for external APIs)
```

### 12. **Tensor Contribution**
```rust
pub struct Tensor12D {
    pub service_id: f64,
    pub port: f64,
    // ... 10 more dimensions
}

impl TensorContributor for SomeModule {
    fn contribute_tensor(&self) -> TensorContribution {
        let dims = [
            self.service_id as f64 / 1000.0,
            self.port as f64 / 65535.0,
            // ... normalized dimensions
        ];
        TensorContribution { dims, metadata: self.name.clone() }
    }
}

// Aggregation
let tensor = registry.aggregate_tensor();  // 12D combined
```

### 13. **Logging (tracing, never println!)**
```rust
use tracing::{debug, info, warn, error};

fn some_operation(&self) -> Result<T> {
    debug!("Starting operation");
    let result = expensive_work()?;
    info!(task = "work", elapsed_ms = 42, "Completed operation");
    Ok(result)
}

// Structured fields
error!(
    service_id = svc.id,
    error = %e,
    "Service health check failed"
);
```

### 14. **Test with #[tokio::test]**
```rust
#[tokio::test]
async fn test_health_check_concurrent() {
    let module = SomeModule::new().await.unwrap();
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let m = module.clone();
            tokio::spawn(async move {
                m.health().await
            })
        })
        .collect();
    
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
}
```

### 15. **Layer Coordinator mod.rs (pattern)**
```rust
// src/m3_core_logic/mod.rs
pub mod pipeline;
pub mod remediation;
// ... 4 more

pub use pipeline::Pipeline;
pub use remediation::RemediationEngine;

pub struct L3Layer {
    pipeline: Arc<RwLock<Pipeline>>,
    remediation: Arc<RwLock<RemediationEngine>>,
    signal_bus: Arc<dyn SignalBus>,
}

impl L3Layer {
    pub async fn new(signal_bus: Arc<dyn SignalBus>) -> Result<Self> {
        Ok(Self {
            pipeline: Arc::new(RwLock::new(Pipeline::new()?)),
            remediation: Arc::new(RwLock::new(RemediationEngine::new()?)),
            signal_bus,
        })
    }

    pub async fn health(&self) -> Result<Health> {
        let p_health = self.pipeline.read().health()?;
        let r_health = self.remediation.read().health()?;
        Ok(Health { layers: vec![p_health, r_health], .. })
    }
}

impl TensorContributor for L3Layer {
    fn contribute_tensor(&self) -> TensorContribution {
        // Aggregate from submodules
    }
}
```

---

## 12. Anti-Patterns / Known Issues

**From Bugs & Known Issues.md + CLAUDE.local.md:**

### Critical Anti-Patterns (NEVER):

| Anti-Pattern | Severity | Reason | Fix |
|---|---|---|---|
| `unsafe { }` | CRITICAL | Memory safety | Forbidden at compile time (`#![forbid(...)]`) |
| `.unwrap()` | CRITICAL | Panics in production | Denied by clippy, use `.map_err()`/`?` operator |
| `.expect()` | CRITICAL | Panics in production | Denied by clippy, use `.map_err()`/`?` operator |
| `panic!()` | HIGH | Crashes service | Use `Result<T>` + error propagation |
| `println!()` for logs | HIGH | Lost in production | Use `tracing` macros (debug, info, warn, error) |
| `chrono::DateTime` internal | MEDIUM | Drift in time handling | Use `Timestamp` + `Duration` (internal); chrono only for external APIs |
| `SystemTime` | MEDIUM | Precision/timezone issues | Use `Timestamp` (Unix seconds) |
| Unbounded channels | MEDIUM | OOM risk | Always `tokio::sync::mpsc::channel(capacity)` |
| Clone when move works | MEDIUM | Perf + memory | Clippy checks via `redundant_clone` |
| `&mut self` in traits | MEDIUM | Breaks interior mutability | Use `&self` + `RwLock<T>` |
| Return references through lock | HIGH | Use-after-free | Clone/owned return via `.read().clone()` |
| `cat / grep / find` in Bash | LOW | Suboptimal | Use Read/Bash/search tools |
| `curl -sf \| jq` | LOW | Hidden errors | Use `curl -s` (no -f flag) |
| `&` backgrounding | LOW | Visibility loss | Use Bash `run_in_background` tool |
| Long-lived lock guards | MEDIUM | Deadlock risk | Scope guards with `{ let g = ...; }` then drop |
| Import up the DAG | CRITICAL | Layer violation | Compile-time error (module structure enforces) |

### Known Issues (Frozen / Won't Fix):

1. **BUG-008: EventBus frozen** — `event_bus.rs` `EventSubscriber` trait has zero production consumers; callback map always empty. Workaround: F-02 (S099) `EventCountSink` proves delivery via `/api/event-bus/callback-stats`.

2. **BUG-012: Circuit breaker stuck OPEN** — Under sustained load, ORAC bridge may transition to OPEN and not half-open probe; requires manual bridge reset via `POST /api/bridge/orac/reset`. Tracked in [[Troubleshooting Decision Tree]].

3. **Drift: v3_homeostasis/* empty** — CLAUDE.md spec says `src/v3_homeostasis/` (~760 LOC) but directory doesn't exist; thermal/decay/diagnostics moved to `m7_observer/thermal_monitor.rs`. Specs outdated.

4. **Drift: Module count "48+" softer** — Actual count of `.rs` files in module directories: ~80 (including mod.rs + sub-files). Spec was approximate.

5. **Drift: Port number** — CLAUDE.md says "port 8080" but binary runs on **8180** (ME V1 retired in S081). Default constant `DEFAULT_PORT = 8180` in `main.rs`.

### Quality Gate Failures (When They Occur):

| Failure | Cause | Fix |
|---|---|---|
| Clippy warnings | New lints after rustc upgrade | Run `cargo clippy -- -D warnings -W clippy::pedantic` + fix all |
| Test failures | Code change breaks downstream | Run `cargo test --lib --release` + debug + re-run |
| Compilation fails | Dependency version conflict | Check `Cargo.lock` + `cargo update` if safe |
| Database migration fails | Schema drift | Verify migration file syntax, apply to dev DB first |
| Bridge circuit breaker OPEN | Target service down | Manual reset via `/api/bridge/{bridge_id}/reset` |

---

## Scaffolding Checklist for New Codebase

✓ **Before Starting:**
- [ ] Copy Cargo.toml structure (forbid unsafe, deny unwrap, warn pedantic)
- [ ] Create 8-layer directory structure (`m1_foundation/` through `nexus/`)
- [ ] Set up CLAUDE.md (460L template) + CLAUDE.local.md (session state)
- [ ] Create maintenance-engine-v2-vault/ with HOME.md + MASTER_INDEX.md
- [ ] Set up 11+ SQL migrations + `config/` TOML files

✓ **Layer Implementation (each layer):**
- [ ] Create `mod.rs` with module declarations + layer coordinator struct
- [ ] Implement TensorContributor for coordinator
- [ ] Write 50+ unit tests per module (100% coverage target)
- [ ] Add layer health check (`health()` method)
- [ ] Document via `///` and update vault notes

✓ **Quality Gate:**
- [ ] `cargo check` (compiles)
- [ ] `cargo clippy -- -D warnings` (zero clippy)
- [ ] `cargo clippy -- -D warnings -W clippy::pedantic` (zero pedantic)
- [ ] `cargo test --lib --release` (all tests pass)

✓ **Integration:**
- [ ] Wire into `engine.rs` (central orchestrator)
- [ ] Add HTTP routes in `main.rs` (30+ endpoints)
- [ ] Add database.rs support (migrations + tables)
- [ ] Set up bridges (ORAC, PV2, POVM, Cascade)

✓ **Documentation:**
- [ ] Update vault HOME.md + MASTER_INDEX.md
- [ ] Add module-level notes (1 per module)
- [ ] Create layer diagrams (Mermaid)
- [ ] Update cross-references (wikilinks)

---

*End of Maintenance Engine V2 — Gold Standard Reference*  
*This document is scaffolding-ready and can be directly adapted for new sibling services.*
