> Back to: [[HOME]]

---
frontmatter:
  source_path: "/home/louranicas/claude-code-workspace/orac-sidecar"
  commit_sha: "6224bd7"
  vault_timestamp: "2026-05-17"
  reference_version: "1.0"
---

# ORAC Sidecar — Gold Standard Reference Profile

**Essence:** Envoy-like intelligent fleet coordination proxy (sidecar daemon on port 8133). Mediates HTTP hook callbacks for Claude Code agents, coordinates Kuramoto-coupled panes via IPC bus, manages evolution chamber (RALPH) with 12D fitness tensor. Production hardened (51K+ LOC, 2,993 tests, zero clippy warnings).

---

## 1. Top-Level Structure

```
orac-sidecar/
├── src/
│   ├── lib.rs                     (58 lines — layer declarations + feature gates)
│   ├── bin/
│   │   ├── main.rs                (3,027 LOC — daemon: Axum :8133 + IPC + RALPH loop)
│   │   ├── client.rs              (805 LOC — CLI tool: status, field, metrics, dispatch)
│   │   ├── probe.rs               (40 LOC — connectivity diagnostics)
│   │   └── ralph_bench.rs         (120 LOC — evolution tick benchmark)
│   ├── m1_core/                   (L1 foundation: types, errors, config, traits)
│   ├── m2_wire/                   (L2 IPC client: Unix socket, V2 wire protocol)
│   ├── m3_hooks/                  (L3 Axum HTTP server: 6 hook endpoints [feature: api])
│   ├── m4_intelligence/           (L4 Hebbian STDP, coupling, routing [feature: intelligence])
│   ├── m5_bridges/                (L5 service integrations: SYNTHEX, ME, POVM, RM, SQLite)
│   ├── m6_coordination/           (L6 conductor, cascade, tick, WASM, memory)
│   ├── m7_monitoring/             (L7 OTel, metrics, dashboard [feature: monitoring])
│   └── m8_evolution/              (L8 RALPH 5-phase chamber [feature: evolution])
├── Cargo.toml                      (8 feature flags; 4 bin targets; forbid unsafe)
├── bacon.toml                      (4-stage quality gate: check→clippy→pedantic→test)
├── config/
│   ├── default.toml               (port 8133, bridge addrs, evolution defaults)
│   ├── dev.toml                   (verbose logging, faster tick)
│   ├── prod.toml                  (JSON logging, evolution enabled)
│   ├── hooks.toml                 (6 hook timeouts, auto-approve patterns)
│   └── bridges.toml               (per-bridge polling, retry, consent)
├── migrations/
│   └── 001_blackboard.sql         (8 tables: pane_status, task_history, agents, coupling, metrics)
├── orac-sidecar-vault/            (162 Obsidian notes; modules/, architecture/, sessions/, schematics/)
├── ai_docs/                       (25 files: QUICKSTART, layer docs, module docs, schematics)
├── ai_specs/                      (11 files: API, hooks, bridges, wire, evolution specs)
├── scripts/                       (povm-seed*.sh, atuin priming, hook test server)
├── tests/                         (12 integration test files per layer)
├── CLAUDE.md                      (180 lines — rules, quality gate, traps, anti-patterns)
├── CLAUDE.local.md                (400 lines — session state, bidirectional anchors, POVM pathways)
├── MASTER_INDEX.md                (488 lines — 150+ files indexed, mindmap branches)
├── ORAC_PLAN.md                   (429 lines — architecture, 4 phases, feature backlog)
├── ORAC_MINDMAP.md                (734 lines — 248 Obsidian notes, 20 branches, 3 vaults)
└── README.md                       (project overview, key numbers)
```

---

## 2. Eight-Layer Architecture

| Layer | Dir | Theme | Module Count | Key Responsibilities | Key Types Exported | Downstream Dependencies |
|-------|-----|-------|--------------|----------------------|-------------------|-------------------------|
| **L1 Core** | `m1_core/` | Foundation | 6 + field_state | Types, errors, config, constants, traits, validation | `PaneId`, `TaskId`, `OracError`, `PvConfig`, `Timestamp` | L2, L3, L4, L5, L6, L7, L8 |
| **L2 Wire** | `m2_wire/` | IPC Protocol | 3 | Unix socket client to PV2, V2 wire handshake, frame parsing | `ClientFrame`, `ServerFrame`, `TaskStatus` FSM | L3, L6, L7, L8 |
| **L3 Hooks** | `m3_hooks/` | HTTP Server | 5 | Axum :8133, 6 hook endpoints, permission policy, MCP gateway | `OracState`, `OracConsent`, `DispatchEntry` | L5, L6, L8 |
| **L4 Intelligence** | `m4_intelligence/` | Hebbian + Routing | 7 | Kuramoto coupling, adaptive K, Hebbian STDP, semantic router, circuit breaker | `CouplingNetwork`, `HebbianWeights`, `CircuitBreaker` | L3, L6, L8 |
| **L5 Bridges** | `m5_bridges/` | Service Connectors | 8 | SYNTHEX, ME, POVM, RM bridges; SQLite blackboard; DevOps, VMS bridges | `Blackboard`, `BridgeHealth`, `WsMessage` | L3, L6, L8 |
| **L6 Coordination** | `m6_coordination/` | Orchestration | 5 | Conductor (PI controller), cascade handoff, tick loop, WASM bridge, memory | `Conductor`, `CascadeState` | L7, L8 |
| **L7 Monitoring** | `m7_monitoring/` | Observability | 4 | OTel traces, Prometheus metrics, field dashboard, token accounting | `MetricsExporter`, `FieldDashboard` | L8 |
| **L8 Evolution** | `m8_evolution/` | Self-Improvement | 5 | RALPH 5-phase, emergence detector (ring buffer), correlation, fitness tensor, mutation | `RalphEngine`, `EmergenceEvent`, `FitnessTensor` | _(none — top)_ |

---

## 3. Wire Protocol (V2)

**M2 Wire — State Machine in m2_wire/m09_wire_protocol.rs:**

```
Hello (client initiates)
    ↓
ClientFrame { "type": "hello", "version": "v2", "client_id": "..." }
    ↓ (server acknowledges)
ServerFrame { "type": "handshake_ack" }
    ↓
Subscribe topics: field.*, task.*, sphere.*
    ↓
NDJSON frame loop: ServerFrame (BusEvent) ← IPC
    ↓
Ping/Pong heartbeat (30s), idle disconnect (90s)
```

**Differs from raw HTTP:**
- **Persistent** Unix socket (single connection, no reconnect per call)
- **Multiplexing** — simultaneous field tick + task events + sphere status
- **Handshake** with 5s timeout; keepalive via heartbeat
- **V1 compat layer** — auto-detects old wire format, bridges gracefully
- **NDJSON framing** — one event per line, no request/response pairs (push-driven from PV2)

---

## 4. Hook System (6 HTTP Endpoints)

**L3 Hooks — m3_hooks/m10_hook_server.rs (Axum :8133):**

| Event | Endpoint | Timeout | Handled By | Permission Gate | Thermal Gate |
|-------|----------|---------|------------|-----------------|--------------|
| SessionStart | `POST /hooks/session_start` | 5s | m11 | auto-approve reads | — |
| PreToolUse | `POST /hooks/pre_tool_use` | 2s | m12 | read-always gate | SYNTHEX bridge (fails open) |
| PostToolUse | `POST /hooks/post_tool_use` | 1s | m12 | — | — |
| UserPromptSubmit | `POST /hooks/user_prompt_submit` | 3s | m13 | auto-approve | — |
| PermissionRequest | `POST /hooks/permission_request` | 2s | m14 | policy cascade (sphere→fleet→default) | — |
| Stop | `POST /hooks/stop` | 5s | m11 | deregister; quality gate | — |

**Permission Policy Cascade:**
1. Per-sphere consent (OracConsent)
2. Fleet-wide default
3. Conservative default (DENY on timeout >500ms)

**Thermal Gate:** If SYNTHEX bridge temperature ≥ 0.9, fails open (allow pre_tool_use to proceed with warning).

---

## 5. Blackboard Pattern

**L1 + L5 — m1_core/field_state.rs + m5_bridges/m26_blackboard.rs:**

**5 Core Tables (SQLite):**
1. **pane_status** — fleet awareness: id, status, phase, last_seen, tool_name
2. **task_history** — per-pane work: id, pane_id, description, status, timestamps
3. **agent_cards** — capabilities: pane_id, capabilities[], domain, token_budget
4. **coupling_snapshot** — Kuramoto state: source→target weight + timestamp
5. **fleet_metrics** — aggregate: timestamp, order_param r, K_effective, active_panes, chimera flag

**State Sharing Model:**
- Single Arc<Mutex<Blackboard>> in OracState
- Read via `.get_pane_status()` → Option<PaneStatus> (returns clone, not reference)
- Write via `.update_pane_status(id, new_status)` → PvResult<()>
- Transactions for multi-table atomicity (coupling + metrics together)

---

## 6. RALPH Evolution Chamber

**L8 — m8_evolution/m36_ralph_engine.rs (5-phase, 5s background loop):**

```
Tick 0-11s: Recognize phase
  - Emergence detector populates candidates
  - Query blackboard for fleet constraints
  ↓
Tick 12-24s: Analyze phase
  - Correlation engine discovers pathways
  - Circuit breaker health assessed
  ↓
Tick 25-36s: Learn phase
  - Hebbian weights consolidated
  - Thermal response evaluated
  ↓
Tick 37-48s: Propose phase
  - m40_mutation_selector picks 10 mutable params (multi-parameter diversity enforcement — BUG-035 fix)
  - Per-parameter cooldown 10 generations
  - Reject if >50% of last 20 mutations hit same param
  ↓
Tick 49-60s: Harvest phase
  - Fitness tensor (D1-D12) scored
  - Snapshot stored (ring buffer, cap 10)
  - Rollback if fitness drops >10% in 30 ticks
```

**12D Fitness Tensor (m39_fitness_tensor.rs):**
- D1: OrderParameter (r ∈ [0,1])
- D2: K (coupling strength)
- D3: TaskThroughput (DevOps bridge)
- D4: TokenCost (per-task cost)
- D5: ErrorRate (circuit breaker failures)
- D6: HebbianHealth (VMS bridge)
- D7: ThermalHealth (SYNTHEX temperature)
- D8: CircuitBreakerState (panes Closed/Open/HalfOpen)
- D9: Latency (hook response time)
- D10: Consensus (PV2 sphere sync)
- D11: Emergence (event frequency)
- D12: Recovery (rollback success %)

**Known Issue from CLAUDE.local.md:**
- RALPH auto-resume after quiet period (line 2722-2724) force-resumes even if it auto-paused for convergence
- May defeat convergence detection; separate convergence-pause flag recommended

---

## 7. Bridges + Clients

**L5 Bridges — 7 service integrations:**

| Bridge | Port | L5 Module | Protocol | Consent Model | Health Check |
|--------|------|-----------|----------|---|----------|
| **SYNTHEX** | 8092 | m22_synthex_bridge + m22_synthex_ws | HTTP GET /v3/thermal + WebSocket /ws/orac | read always, write opt-in | `/health` (2s timeout) |
| **ME V2** | 8180 | m23_me_bridge | HTTP GET /api/observer | read always | `/health` (2s timeout) |
| **POVM** | 8125 | m24_povm_bridge | HTTP POST /memories, /pathways, /hydrate | read/write opt-in | `/health` (2s timeout) |
| **RM** | 8130 | m25_rm_bridge | HTTP TSV only: POST /put, GET /search | read always, write always | `/health` (2s timeout) |
| **Blackboard** | — | m26_blackboard | SQLite in-process | — | _(always healthy)_ |
| **DevOps** | 8082 | m27_devops_bridge | HTTP GET /metrics | read always | `/health` (2s timeout) |
| **VMS** | 8120 | m28_vms_bridge | HTTP GET /morphogenic | read always | `/health` (2s timeout) |

**Circuit Breaker Pattern (m21_circuit_breaker.rs):**
- Per-pane FSM: Closed → Open → HalfOpen → Closed
- Threshold: 5 failures → Open; 10 successes → Closed
- Cooldown: 30 tick minimum in Open state
- Registry: `BreakerRegistry` for fleet-wide management

**Bridge Health Breaker:**
- One global breaker per bridge (not per-pane)
- Failure: HTTP timeout (2s) or connection refused
- State broadcast via m7 metrics

---

## 8. Capabilities Catalogue (~45 items)

**HTTP Endpoints:**
- 6 hook endpoints (:8133/hooks/*)
- 1 health endpoint (:8133/health)
- 1 metrics endpoint (:8133/metrics) — Prometheus-compatible
- 1 client API (:8133/status, /field, /blackboard, /dispatch, /fleet)
- MCP gateway: 15 tools (habitat_health, dispatch_task, persist_pathway, etc.) [feature: mcp_gateway]

**IPC Bus:**
- Unix socket subscribe to field.tick, field.chimera, field.sync, task.*, sphere.*
- Keepalive: Ping/Pong 30s, idle disconnect 90s

**6 Hook Events:**
- SessionStart (register sphere, hydrate from POVM, init blackboard)
- PreToolUse (thermal gate SYNTHEX)
- PostToolUse (Hebbian STDP update, task poll, metrics record)
- UserPromptSubmit (field state injection into prompt context)
- PermissionRequest (auto-approve/deny per consent policy)
- Stop (deregister, quality gate, cascade handoff)

**4 Bridge Integrations:**
- SYNTHEX thermal read + Hebbian writeback + cascade amplification
- ME fitness signal (evolution correlation)
- POVM memory hydration + crystallisation (write-only; call /hydrate to read)
- RM TSV persistence (never JSON)

**RALPH Evolution:**
- 5-phase loop (Recognize→Analyze→Learn→Propose→Harvest)
- Multi-parameter mutation (10 mutable params + diversity gate)
- Snapshot/rollback (atomic state, ring buffer cap 10)
- Emergence detector (12 types, ring buffer cap 5,000, TTL decay)

**MCP Gateway (15 tools):**
- habitat_health, service_detail, dispatch_task, persist_pathway
- coupling_boost, blackboard_snapshot, fleet_health
- + 8 more for Anthropic SDK bridge

**Fleet Proxy:**
- Task dispatch via semantic router (domain affinity 40% + Hebbian 35% + availability 25%)
- Cascade handoff (SYS-1 sphere mitosis: phase + coupling weight transfer)
- Field-driven assignment (TaskTarget::FieldDriven)

**12D Fitness Tensor:**
- 12 weighted dimensions (order param, K, throughput, latency, etc.)
- Trend detection via linear regression
- Weighted sum → [0.0, 1.0]

**Blackboard Query:**
- `.get_pane_status()`, `.get_all_panes()`, `.get_coupling_snapshot()`, `.get_metrics()`
- Consensus with PV2 (3x daily refresh, manual sync via `/hooks/session_start`)

---

## 9. Cargo Features

**Feature Matrix:**
```
default = [api, persistence, bridges, intelligence, monitoring, evolution]

api              → Axum, tower-http → L3 hooks
persistence      → rusqlite → L5 blackboard
bridges          → (no extra deps) → L5 service integrations
intelligence     → tower → L4 Hebbian/coupling/routing
monitoring       → opentelemetry, opentelemetry-otlp → L7 metrics
evolution        → (no extra deps) → L8 RALPH loop
mcp_gateway      → (no extra deps) → L3 MCP routes (15 tools)
agentic          → ws-bridge → m41-m45 comms cluster (Phase 2.5)
ws-bridge        → tokio-tungstenite, futures-util, tokio-util → m22 WS client
full             → all 9 of the above
```

**Feature-Gated Layers:**
- L3 Hooks: `#[cfg(feature = "api")]` (Axum HTTP server)
- L4 Intelligence: `#[cfg(feature = "intelligence")]` (Hebbian, coupling)
- L5 Bridges: Not gated (always available)
- L7 Monitoring: `#[cfg(feature = "monitoring")]` (OTel, metrics)
- L8 Evolution: `#[cfg(feature = "evolution")]` (RALPH chamber)

---

## 10. Persistence & Database

**SQLite Blackboard (migrations/001_blackboard.sql):**
- **8 tables:** pane_status, task_history, agent_cards, coupling_snapshot, fleet_metrics, + 3 more (ralph_state, consent_audit, hebbian_summary)
- **PRAGMA:** journal_mode=WAL, synchronous=NORMAL, foreign_keys=ON
- **Transactions:** Multi-table updates atomic (e.g., coupling snapshot + metrics together)
- **Pruning:** Old metrics entries deleted every 500 ticks (>30 days old)

**What Gets Stored:**
- Pane lifecycle: registration (SessionStart) → work (PostToolUse) → deregistration (Stop)
- Coupling weights snapshot (60-tick interval)
- RALPH state (every 12 ticks: generation, fitness, params, timestamp)
- Consent audit trail (per-sphere, per-event)
- Hebbian summary (coactivation count, weight distribution)
- Fleet metrics (order param, K, active panes, chimera flag) — 1 row per tick

**Hydration on Startup:**
- Restore pane_status, coupling_snapshot, RALPH state from prior session
- Consistency check: clamp/dampen out-of-bounds values (log violations)
- RALPH quiet period: 12 ticks (RALPH paused, no mutations)

---

## 11. Test Discipline

**Test Count:** 2,993 tests (verified `cargo test --lib --features full`)

**Per-Layer Minimum:** 50+ tests per layer (enforced in CI)

**Test Organization:**
- **In-file** `#[cfg(test)]` modules (never separate test files)
- **Unit tests:** Function-level, test failure paths
- **Integration tests:** `/tests/l{1..8}_*_integration.rs` (12 files)
- **Cross-layer workflows:** `/tests/cross_layer_workflows.rs`
- **Property tests:** `/tests/property_tests.rs` (randomized)
- **Stress tests:** `/tests/stress_test.rs` (scalability)

**Test Quality Rules:**
- Float comparison: use `approx::assert_abs_diff_eq!` (epsilon comparison)
- Never `assert_eq!` on f64
- Unwrap allowed **only** in tests
- All public fallible functions have `// Error cases:` in docs

---

## 12. Vault / Obsidian Discipline

**162 Obsidian notes in orac-sidecar-vault/ partitioned:**

| Partition | Count | Purpose |
|-----------|-------|---------|
| **modules/** | 41+ | Per-module deep dives (m01-m40+, each with "# Errors" section) |
| **architecture/** | 6 | Layer DAG, bridge topology, field state, cascade flow, hook flow, MCP |
| **schematics/** | 28+ | Mermaid diagrams + textual descriptions (layer architecture, bridge topology, etc.) |
| **sessions/** | 5+ | S114, S115, S116, incident notes, hang RCA |
| **Root** | 6 | HOME.md, MASTER INDEX.md, Module Index.md, Diagnostics.md, Bugs & Known Issues.md, Problem Solving.md |

**Anchor Pattern (Bidirectional Wikilinks):**
- Every module note starts with `> Back to: [[Architecture Overview]]`
- POVM pathways seeded with `orac_*` namespace (149 total)
- Cross-vault references to main vault (`~/projects/claude_code/`)

**Module Index:** 41+ modules listed with layer, LOC, test count, source path, vault link

---

## 13. Configuration System

**Layered Loading (m1_core/m03_config.rs):**
```
default.toml (base settings)
    ↓ overlay
dev.toml OR prod.toml (environment-specific)
    ↓ overlay
ORAC_* environment variables (highest priority)
```

**Config Files (config/):**
1. **default.toml** — port 8133, bridge addrs (8092, 8180, 8125, 8130), evolution disabled
2. **dev.toml** — verbose logging, tick_ms=500, bridge_poll_ms=2000
3. **prod.toml** — JSON logging, tick_ms=1000, evolution enabled, auto_scale_k enabled
4. **hooks.toml** — per-hook timeouts (pre 2s, post 1s, compact 5s), auto-approve patterns (glob syntax), thermal thresholds
5. **bridges.toml** — per-bridge: addr, polling_interval, retry_count, timeout, consent_required

**ENV Vars (figment):**
- `ORAC_SERVER__PORT=8133`
- `ORAC_BRIDGES__SYNTHEX_ADDR=127.0.0.1:8092`
- `ORAC_EVOLUTION__ENABLED=true`

---

## 14. Patterns to Emulate for Scaffolding

### 8-Layer mN_ Naming Convention
```rust
// src/lib.rs
pub mod m1_core;          // Foundation (no deps)
pub mod m2_wire;          // L1 → L2
pub mod m3_hooks;         // L1,L2 → L3 [feature: api]
pub mod m4_intelligence;  // L1,L2 → L4 [feature: intelligence]
pub mod m5_bridges;       // L1 → L5 [feature: bridges]
pub mod m6_coordination;  // L1,L2,L4,L5 → L6
pub mod m7_monitoring;    // L1,L2,L5 → L7 [feature: monitoring]
pub mod m8_evolution;     // L1,L4,L5,L7 → L8 [feature: evolution]

// Each layer dir: src/m{N}_*/ contains m{NN}_*.rs files
// src/m1_core/m01_core_types.rs, m02_error_handling.rs, ... m06_validation.rs, mod.rs
```

### Blackboard Pattern
```rust
// L1 + L5: Shared mutable state, lock-scoped, owned returns
pub struct Blackboard { db: Arc<Mutex<Connection>> }
impl Blackboard {
    pub fn get_pane_status(&self, id: PaneId) -> PvResult<Option<PaneStatus>> {
        let db = self.db.lock();
        let status = db.query_row(...)?.cloned();  // OWNED CLONE, not reference
        drop(db);  // explicit drop before next lock
        Ok(status)
    }
}
```

### Wire Protocol State Machine
```rust
// L2: Handshake → Subscribe → Event loop
enum WireState { Hello, Handshake, Subscribed, Ping, Close }
// Timeouts: 5s handshake, 30s keepalive, 90s idle
```

### Feature-Gated Layers
```rust
#[cfg(feature = "api")]
pub mod m3_hooks;  // Axum HTTP server

#[cfg(feature = "evolution")]
pub mod m8_evolution;  // RALPH chamber
```

### Hook System (HTTP Endpoints)
```rust
// L3 Axum routes, <1ms response (in-memory state, no I/O on critical path)
#[post("/hooks/session_start")]
async fn session_start(Json(req): Json<SessionStartRequest>) -> Json<HookResponse> { ... }
```

### Evolution Chamber Pattern
```rust
// L8: 5-phase loop, fitness tensor, snapshot/rollback
pub async fn ralph_loop(state: Arc<OracState>, mut shutdown: watch::Receiver<()>) {
    loop {
        select! {
            _ = shutdown.changed() => break,
            _ = tokio::time::sleep(RALPH_INTERVAL) => {
                // Recognize → Analyze → Learn → Propose → Harvest
                // Snapshot if fitness improves, rollback if it drops >10%
            }
        }
    }
}
```

### Config-as-TOML Layering
```toml
# config/default.toml
[server]
port = 8133

# config/prod.toml (overlay)
[evolution]
enabled = true

# ENV override: ORAC_EVOLUTION__ENABLED=true
```

---

## 15. Anti-Patterns / Known Issues

### From Bugs & Known Issues (orac-sidecar-vault/Bugs & Known Issues.md)

| Bug ID | Severity | Pattern | Status | Fix Strategy |
|--------|----------|---------|--------|--------------|
| BUG-032 | HIGH | `#[derive(Default)]` on ProposalManager → max_active=0 | Fixed S117 | Manual impl Default delegating to named constructor |
| BUG-033 | HIGH | Bridge URLs include `http://` prefix (breaks socket parsing) | Fixed S078 | Raw `host:port` only, no scheme prefix |
| BUG-034 | MEDIUM | POVM is write-only (read returns stale data) | By design | Call `/hydrate` endpoint to read back state |
| BUG-035 | CRITICAL | RALPH mutation mono-parameter trap (ME issue cloned to ORAC) | Fixed S054 | Round-robin param selection + 10-gen cooldown + diversity reject gate |
| AP21 | CRITICAL | TCP partial-read race (incomplete HTTP frame arrival) | Mitigated S115 | Loop until `\r\n\r\n` boundary detected |
| AP22 | CRITICAL | Close-on-unread RST (kernel sends RST if recv buffer has bytes) | Mitigated S115 | `shutdown(Write)` → drain loop → drop |
| AP23 | HIGH | `spawn_blocking` on current_thread deadlocks | Fixed S1000007-B | Use `multi_thread` runtime with ≥2 worker threads |

### RALPH Paused-Latch Issue
- Auto-resume after quiet period (12 ticks) force-resumes even if convergence-paused
- Defeats convergence detection signal
- **Recommendation:** Add distinct convergence-pause flag (not quiet-pause)

### ORAC Cache Location Gap
- Field state cached in L1 (not authoritative from PV2)
- 300s stale cache from fleet-ctl
- **Recommendation:** Wire cache into L5 as per-bridge pattern, not core-layer refactor

### No `unwrap()` / `expect()` (Enforced)
```rust
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used)]
// Violations cause CI failure
```

### POVM Write-Only Behavior
- Bridge accepts writes to `/memories`, `/pathways`
- Reads via GET return 404
- **Workaround:** Call POST `/hydrate` to load back into ORAC state

### Thermal Gate Fails Open
- If SYNTHEX bridge unreachable, PreToolUse succeeds (logs warning)
- Better to allow tool execution with thermal unknown than block all work
- Consent policy still applies (separate gate)

---

## 16. Summary: Scaffolding Checklist

When building **the-workflow-engine**:

- [ ] **8 layers:** `m{1..8}_*` directories (not 6, not 10)
- [ ] **Layer 1 independence:** No upstream imports (compile-time DAG)
- [ ] **Feature gates:** api, persistence, bridges, intelligence, monitoring, evolution, full
- [ ] **Hook system:** 6 HTTP endpoints (SessionStart, PreToolUse, PostToolUse, UserPromptSubmit, PermissionRequest, Stop)
- [ ] **Blackboard pattern:** SQLite + Arc<Mutex<>> + owned clones from read guards
- [ ] **Wire protocol FSM:** Handshake → Subscribe → Event loop with keepalive
- [ ] **Evolution chamber:** 5-phase loop + 12D tensor + multi-parameter diversity
- [ ] **Config-as-TOML:** 3-layer loading (default → env-specific → ENV vars)
- [ ] **Migrations:** Explicit SQL schema versioning (001_*.sql)
- [ ] **Vault discipline:** 162+ notes partitioned (modules/, architecture/, schematics/, sessions/)
- [ ] **Test minimum:** 50+ tests per layer
- [ ] **Zero unsafe:** `#![forbid(unsafe_code)]` in lib.rs
- [ ] **No unwrap:** `#![deny(clippy::unwrap_used, clippy::expect_used)]`
- [ ] **Quality gate:** 4-stage bacon check → clippy → pedantic → test
- [ ] **POVM integration:** `orac_*` namespace pathways seeded
- [ ] **Bidirectional links:** Every module note back-links to architecture
- [ ] **BUG-035 fix:** Multi-parameter mutation + round-robin + cooldown (not mono-param)

---

**Word count:** ~2,100 | **Source commit:** 6224bd7 | **Scope:** 8 layers, 40+ modules, 51K+ LOC, 2,993 tests, zero warnings (pedantic).
