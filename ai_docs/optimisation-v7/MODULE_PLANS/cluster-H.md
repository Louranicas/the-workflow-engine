---
title: MODULE PLAN — Cluster H (Substrate Feedback) · m40 SYNTHEX emit / m41 LCM router / m42 stcortex emit (POVM dual-path retired per 2026-05-17 ADR)
date: 2026-05-17
kind: planning-only · per-module spec · no code authorised (HOLD-v2 + AP24)
cluster: H
layer: L8
modules: [m40, m41, m42]
loc_budget: ~450
test_budget: 180
mutation_kill_targets: { m40: 75%, m41: 75%, m42: 80% }
authority: Command · workflow-trace V7 optimisation
---

# Cluster H — Substrate Feedback (L8)

> Back to: [[../TASK_LIST_V7_OPTIMISATION.md]] · [[../ULTRAMAP.md]] · [[../GENERATIONS/G3-bidi-flow.md]] · sibling [[cluster-G.md]] · [[CROSS_CLUSTER_SYNERGIES.md]]
>
> **Function:** Cluster H is the engine's **substrate feedback layer** — three bridge modules that propagate dispatch outcomes back to the habitat substrate (SYNTHEX v2 NexusEvent + LCM RPC + POVM/stcortex Hebbian reinforce). Per [[../KEYWORDS_20.md]] § Cluster: H is L8, terminal layer. Per [[../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md]] Part I § Cluster H: *Substrates being down NEVER blocks dispatch (outbox always written).* The cluster is the *only* CC-5 (substrate learning loop) emit path — without H, the engine is decorative.

---

## Overview

Cluster H sits at L8, the terminal layer, downstream of Cluster G m32 dispatcher and feeding back (eventually, via stcortex pathway weight updates) to Cluster F m20-m22 iteration inputs. Three modules, each bridging to one external substrate:
- **m40** → SYNTHEX v2 `:8092/v3/nexus/push` (NexusEvent emit; outbox-first JSONL)
- **m41** → LCM RPC `lcm.loop.create { max_iters: 1 }` (deploy-shaped step routing)
- **m42** → POVM `:8125/reinforce` + stcortex (dual-path during POVM→stcortex overlap → 2026-07-10)

The cluster's **non-blocking discipline**: outbox-first JSONL durability means substrate-down NEVER blocks dispatch. m32 hands DispatchOutcome to Cluster H and continues; Cluster H writes to disk first, then attempts wire emit fire-and-forget. Circuit breakers (one per peer, 3 total) shared via `m40_42_common::Breaker` close the loop without blocking. Per [[../GENERATIONS/G3-bidi-flow.md]] § Bidi-03: *1 breaker per peer (3 total); shared `m40_42_common::Breaker`*.

Per ULTRAMAP View 2: ~450 LOC across 3 modules (~150 LOC/module avg). ~85-95% reuse density (per [[../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md]] Part I § Cluster H) — highest reuse in the engine because Cat 08 (Nexus-LCM-RPC) is gold standard per Command-3 E3. 180 tests budgeted (60/module). Mutation kill 75/75/80; m42 highest because dual-path cutover safety is load-bearing.

---

## m40 — SYNTHEX v2 NexusEvent emitter (L8 · `src/m40_synthex_emit/`)

### Purpose

Emit `WorkflowEvent { id, outcome, fitness_delta, fields }` to SYNTHEX v2 `:8092/v3/nexus/push` for every dispatch outcome. The emit is **outbox-first**: write JSONL to `outbox/m40/*.jsonl` before any wire attempt; HTTP POST is fire-and-forget (best-effort). Circuit breaker (shared with m41/m42 via `m40_42_common::Breaker`) prevents thundering-herd against unhealthy SYNTHEX.

### Edge contract (from [[../GENERATIONS/G3-bidi-flow.md]] § m40)

- **Upstream-IN:** `m32.DispatchOutcome`
- **Downstream-OUT:** outbox-first JSONL `outbox/m40/*.jsonl` (durable) → HTTP fire-and-forget `:8092/v3/nexus/push` (best-effort)
- **Aspect-IN:** m8 build_prereq, m9 namespace_guard (workflow_trace_* prefix), m10 Ember CI
- **Failure-mode mitigated:** substrate down NEVER blocks dispatch (outbox always written); circuit-breaker (1 per peer)

### src/ path (planning-spec only)

```
src/m40_synthex_emit/
├── mod.rs                # public: pub fn emit_workflow_event, pub struct WorkflowEvent
├── outbox.rs             # outbox-first JSONL writer (atomic tmp + rename)
├── wire.rs               # HTTP POST to :8092/v3/nexus/push (fire-and-forget)
├── flush.rs              # background flush worker (drains outbox post-recovery)
└── tests/
```

### LOC budget

~150 LOC (per ULTRAMAP View 2). Outbox ~50 LOC; wire ~40 LOC; flush worker ~40 LOC; mod.rs + types ~20 LOC.

### Test budget (60 tests; per [[../STANDARDS/TEST_DISCIPLINE.md]] § m40)

| Family | Count | Notes |
|---|---:|---|
| F-Unit | 30 | per WorkflowEvent variant; atomic-write semantics; circuit-breaker state transitions |
| F-Property | 5 | invariants: outbox-write-first ordering (write completes before wire attempt); breaker idempotent across concurrent emits |
| F-Fuzz | 1 target (24h budget) | `m40_outbox_jsonl` per [[../GENERATIONS/G6-test-discipline.md]] § Fuzz enumeration — JSONL line-parse robustness |
| F-Integration | 15 | m32→m40 wiring; SYNTHEX v2 live :8092 contract; outbox drain on recovery |
| F-Contract | 5 | F-Contract: WorkflowEvent schema parity vs SYNTHEX v3 nexus/push accepted format (bridge-contract skill) |
| F-Regression | 4 | reserved |
| F-Mutation | 1 budget (≥**75%** per G6) | outbox-first durability critical |

### Mutation kill threshold

**75%** (G6 m40). Outbox-write-first ordering mutations must die — a surviving mutation that swaps write order to wire-first reintroduces blocking risk.

### Boilerplate-lift source

Per [[../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md]] Part IV Category 08 (Nexus-LCM-RPC, **85% reuse** — gold standard per Command-3 E3): m24_povm_bridge pattern lifted directly. Outbox-first + circuit-breaker patterns established.

Fresh authorship: ~30 LOC (WorkflowEvent schema specific to workflow-trace; m32 integration). Lifted: ~120 LOC.

### Structural-gap LOC

None (cluster H owns no structural-gap LOC — Gap 1/2/3 all in upstream clusters).

### Failure-modes covered

| Failure | Mechanism |
|---|---|
| Substrate down blocking dispatch | outbox-first JSONL ensures every event is durable before wire attempt |
| Circuit-breaker thundering herd | shared `m40_42_common::Breaker` 5-fail-Open + 60s-HalfOpen |
| AP-Hab-03 (AP30) | m9 namespace_guard validates `workflow_trace_*` prefix in every WorkflowEvent.id |
| AP-Drift-06 (bridge contract drift) | F-Contract tests + bridge-contract skill pre-merge |

### Atuin trajectory anchor

`wt-m40-emit` (proposed): every emit captures `(workflow_id, outcome, fitness_delta, wire_attempt_outcome, breaker_state)` to atuin.

### Watcher class pre-position

- **Class A** (activation) at first emit post-G9.
- **Class B** (hand-off boundary) at every wire attempt — cross-substrate boundary.
- **Class I** (Hebbian silence) if emits occur but SYNTHEX `learning_health` doesn't trend up — CC-5 broken at substrate.

---

## m41 — LCM router (L8 · `src/m41_lcm_router/`)

### Purpose

Route **deploy-shaped** dispatch outcomes through LCM RPC `lcm.loop.create { max_iters: 1 }` (NOT hypothetical `lcm.deploy` — preserves compat with LCM's existing 9-RPC surface per [[../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md]] Part I § Cluster H). Non-deploy outcomes are ignored (m41 is conditional on outcome shape). Outbox-first JSONL discipline shared with m40.

### Edge contract

- **Upstream-IN:** `m32.DispatchOutcome` with deploy-shaped steps (detected via shape predicate)
- **Downstream-OUT:** LCM RPC `lcm.loop.create { max_iters: 1, …}` (per LCM existing 9-RPC surface)
- **Aspect-IN:** m8, m9
- **Failure-mode mitigated:** routes only deploy-shaped; non-deploy ignored (no over-routing); outbox-first

### src/ path (planning-spec only)

```
src/m41_lcm_router/
├── mod.rs                # public: pub fn route_if_deploy_shaped
├── shape_predicate.rs    # detects deploy-shaped DispatchOutcome (cargo build/release/etc patterns)
├── rpc_client.rs         # JSON-RPC 2.0 client (newline-framed; stdio or TCP)
├── frame.rs              # newline-framed JSON-RPC 2.0 parser (fuzz target)
├── outbox.rs             # outbox-first JSONL (mirrors m40 pattern)
└── tests/
```

### LOC budget

~150 LOC. RPC client ~50 LOC; frame parser ~30 LOC; shape predicate ~30 LOC; outbox ~30 LOC; mod.rs ~10 LOC.

### Test budget (60 tests)

| Family | Count |
|---|---:|
| F-Unit | 30 |
| F-Property | 5 (shape predicate determinism; frame parser invariants) |
| F-Fuzz | 1 target | `m41_jsonrpc_frame` per [[../GENERATIONS/G6-test-discipline.md]] — newline-framed JSON-RPC 2.0 parser must not panic on arbitrary bytes |
| F-Integration | 15 |
| F-Contract | 5 (LCM 9-RPC surface contract) |
| F-Regression | 4 |
| F-Mutation | 1 budget (≥**75%** per G6) |

### Mutation kill threshold

**75%** (G6 m41). RPC framing mutations must die; shape predicate mutations must die (false-positive routes non-deploy through LCM; false-negative drops deploy events).

### Boilerplate-lift source

Per [[../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md]] Part IV Category 08 (Nexus-LCM-RPC, 85% reuse): LCM RPC client lifted ~85% from existing habitat patterns. Outbox-first ~95% lifted from m40.

Fresh authorship: ~25 LOC (shape_predicate workflow-trace-specific). Lifted: ~125 LOC.

### Structural-gap LOC

None.

### Failure-modes covered

| Failure | Mechanism |
|---|---|
| Over-routing | shape_predicate gates routing; non-deploy DispatchOutcomes never reach LCM |
| LCM down blocking | outbox-first JSONL; circuit-breaker shared |
| AP-Drift-06 | F-Contract tests against LCM 9-RPC surface |

### Atuin trajectory anchor

`wt-m41-route` (proposed): captures `(workflow_id, shape_predicate_result, lcm_rpc_outcome)` per attempt.

### Watcher class pre-position

- **Class B** (hand-off boundary) at every LCM RPC call.
- **Class C** (refusal) at every non-deploy-shaped skip — refusal is correct.

---

## m42 — POVM dual-path reinforce (L8 · `src/m42_povm_dual/`)

### Purpose

Reinforce stcortex/POVM pathway weights with `fitness_delta` per DispatchOutcome (PassVerified +0.25 / Pass +0.15 / Blocked -0.05 / Fail -0.10 per [[../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md]] Part I § Cluster H). **Dual-path during POVM→stcortex overlap → 2026-07-10**: BOTH POVM `:8125/reinforce` AND stcortex via m13 are written when `povm_overlap_active=true`; post-cutover routes stcortex-only. AP30 namespace prefix `workflow_trace_*` mandatory.

### Edge contract

- **Upstream-IN:** `m32.DispatchOutcome`
- **Downstream-OUT:** BOTH POVM `:8125/reinforce` (overlap → 2026-07-10) AND stcortex via m13 (post-cutover routing per `povm_overlap_active` flag)
- **Aspect-IN:** m8, m9 (AP30 prefix mandatory), m10
- **Failure-mode mitigated:** F-fitness_delta constants enforce correct sign + magnitude; cutover ~D25 dual-path active; POVM deprecation 2026-07-10

### src/ path (planning-spec only)

```
src/m42_povm_dual/
├── mod.rs                # public: pub fn reinforce_dual, pub const FITNESS_DELTA_*
├── povm_client.rs        # POST :8125/reinforce (DEPRECATED 2026-07-10)
├── stcortex_routing.rs   # routes via m13 stcortex_writer
├── dual_path.rs          # povm_overlap_active flag handling; cutover D25 dance
├── fitness_delta.rs      # PassVerified +0.25 / Pass +0.15 / Blocked -0.05 / Fail -0.10
├── outbox.rs             # outbox-first JSONL (mirrors m40 pattern)
└── tests/
```

### LOC budget

~150 LOC. povm_client ~30 LOC; stcortex_routing ~30 LOC; dual_path ~40 LOC; fitness_delta ~10 LOC; outbox ~30 LOC; mod.rs ~10 LOC.

### Test budget (60 tests)

| Family | Count |
|---|---:|
| F-Unit | 30 |
| F-Property | 5 (fitness_delta bounded [-0.10, +0.25]; dual-path idempotent during overlap) |
| F-Fuzz | 0 |
| F-Integration | 15 (POVM + stcortex live services per [[../GENERATIONS/G6-test-discipline.md]] § matrix) |
| F-Contract | 5 (POVM reinforce schema + stcortex pathway-write schema) |
| F-Regression | 4 |
| F-Mutation | 1 budget (≥**80%** per G6) |

### Mutation kill threshold

**80%** (G6 m42 — highest in Cluster H). Rationale: dual-path cutover safety. A surviving mutation that drops the stcortex write during overlap silently breaks the cutover (POVM deprecates 2026-07-10; stcortex must be carrying load by then). A surviving mutation that drops the POVM write during overlap creates substrate desync.

### Boilerplate-lift source

Per [[../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md]] Part IV Category 08 (Nexus-LCM-RPC, 85% reuse): m24_povm_bridge gold-standard pattern lifted. Per Category 02 (stcortex consumer, 80% reuse): subscriber_main.rs pattern adapted for write-side.

Fresh authorship: ~30 LOC (dual_path cutover logic; fitness_delta constants specific to workflow-trace). Lifted: ~120 LOC.

### Structural-gap LOC

None.

### Failure-modes covered

| Failure | Mechanism |
|---|---|
| AP-Hab-03 (AP30) | m9 namespace_guard validates `workflow_trace_*` prefix at every reinforce call |
| Substrate cutover desync | dual-path during overlap; flag-gated routing; integration test exercises both paths |
| AP-Hab-11 (hyphen-slug munge) | pre_id/post_id slug encoding tested; hyphens → underscores convention enforced |
| AP-Drift-06 | F-Contract tests against both POVM + stcortex schemas |

### Atuin trajectory anchor

`wt-m42-reinforce` (proposed): captures `(workflow_id, outcome, fitness_delta, povm_outcome, stcortex_outcome, dual_path_active)` per call.

### Watcher class pre-position

- **Class B** (hand-off boundary) at every POVM + stcortex call (2 boundary crossings during overlap).
- **Class D** (four-surface drift) at any cutover desync detection.
- **Class I** (Hebbian silence) if reinforce calls land but pathway weights don't change — substrate broken downstream.

---

## Cluster-level synergies

### CC-5 — Substrate Learning Loop (G → H → back to F)

Cluster H IS CC-5's emit half. Per [[../GENERATIONS/G3-bidi-flow.md]] § CC-5:
```
m32 DispatchOutcome
    ──► m40 ──► SYNTHEX v2 :8092/v3/nexus/push
    ──► m41 ──► LCM lcm.loop.create
    ──► m42 ──► POVM/stcortex pathway.weight delta
                                              │
                                              ▼
                              stcortex pathway.weight updated
                                              │
                                              ▼
                              m31 reads at next selection cycle
                                              │
                                              ▼
                  selection distribution shifts (days/weeks)
```

Failure-mode (Watcher Class-I flag): if `learning_health` does not move during pipeline runs, Cluster H is *decorative* — engine appears functional but substrate isn't being fed. Pre-positioned to flag at synthesis as workflow-level improvement candidate.

### Intra-cluster

m40/m41/m42 each consume independently from m32.DispatchOutcome (fan-out). They share `m40_42_common::Breaker` library (one breaker instance per peer, shared lib). They do NOT couple to each other beyond the shared breaker — independent failure domains.

### Shared `m40_42_common::Breaker` spec (planning-spec only)

```rust
// planning-spec only — m40_42_common::breaker
// rationale: one circuit breaker per peer (SYNTHEX, LCM, POVM/stcortex); shared across m40/m41/m42
// Per [[../GENERATIONS/G3-bidi-flow.md]] § Bidi-03 closure

#[derive(Clone, Debug)]
pub enum BreakerState {
    Closed { consecutive_failures: u8 },
    Open { opened_at: SystemTime },
    HalfOpen { probe_in_flight: bool },
}

pub struct Breaker {
    state: Arc<Mutex<BreakerState>>,
    fail_threshold: u8,           // default: 5
    open_duration: Duration,      // default: 60s
}

impl Breaker {
    pub async fn allow(&self) -> bool {
        let mut state = self.state.lock().await;
        match *state {
            BreakerState::Closed { .. } => true,
            BreakerState::Open { opened_at } if opened_at.elapsed().unwrap() >= self.open_duration => {
                *state = BreakerState::HalfOpen { probe_in_flight: true };
                true  // allow one probe through
            }
            BreakerState::Open { .. } => false,
            BreakerState::HalfOpen { probe_in_flight: false } => true,
            BreakerState::HalfOpen { probe_in_flight: true } => false,  // only one probe at a time
        }
    }

    pub async fn record_success(&self) {
        let mut state = self.state.lock().await;
        *state = BreakerState::Closed { consecutive_failures: 0 };
    }

    pub async fn record_failure(&self) {
        let mut state = self.state.lock().await;
        match *state {
            BreakerState::Closed { consecutive_failures } if consecutive_failures + 1 >= self.fail_threshold => {
                *state = BreakerState::Open { opened_at: SystemTime::now() };
            }
            BreakerState::Closed { consecutive_failures } => {
                *state = BreakerState::Closed { consecutive_failures: consecutive_failures + 1 };
            }
            BreakerState::HalfOpen { .. } => {
                *state = BreakerState::Open { opened_at: SystemTime::now() };
            }
            BreakerState::Open { .. } => {} // already open
        }
    }
}
```

### Outbox-first JSONL pattern spec (planning-spec only)

```rust
// planning-spec only — m40/m41/m42 shared outbox discipline
// rationale: substrate-down NEVER blocks dispatch; durability before wire attempt
// Per [[../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md]] Part I § Cluster H

pub async fn emit_outbox_first<T: Serialize>(
    event: &T,
    outbox_dir: &Path,
    wire_fn: impl Future<Output = Result<(), WireError>>,
    breaker: &Breaker,
) -> Result<(), OutboxError> {
    // STEP 1: write JSONL line to outbox atomically
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let tmp = outbox_dir.join(format!(".{ts}.jsonl.tmp"));
    let final_path = outbox_dir.join(format!("{ts}.jsonl"));
    {
        let mut f = tokio::fs::File::create(&tmp).await?;
        f.write_all(&serde_json::to_vec(event)?).await?;
        f.write_all(b"\n").await?;
        f.sync_all().await?;
    }
    tokio::fs::rename(tmp, final_path).await?;
    // OUTBOX DURABLE — dispatch can return success now if caller waits

    // STEP 2: attempt wire emit (fire-and-forget)
    if breaker.allow().await {
        match wire_fn.await {
            Ok(()) => breaker.record_success().await,
            Err(_) => breaker.record_failure().await,
        }
    }
    // Note: outbox file remains until background flush worker confirms wire acceptance
    Ok(())
}
```

---

## Cluster-level antipatterns (Cluster H specific)

| ID | Antipattern | Mitigation |
|---|---|---|
| AP-Hab-03 (AP30) | namespace prefix violation | m9 validates `workflow_trace_*` at every reinforce call (m42) + every NexusEvent emit (m40) |
| AP-Drift-06 | bridge contract drift across 3 peers | F-Contract tests per module; bridge-contract skill pre-merge |
| AP-Hab-11 (hyphen-slug munge) | stcortex slug encoding silently fails | pre_id/post_id hyphens → underscores tested |
| CC-5 decorative-cluster risk | Cluster H emits but substrate doesn't move | Phase 5C weekly Watcher synthesis monitors `learning_health` delta; flags if no movement after 5+ dispatches |
| Substrate-down blocking | wire failure stalls dispatch | outbox-first JSONL absolute discipline |

---

## Citation discipline

Every claim cites ULTRAMAP View 2, G3 § Cluster H + § Bidi-03, GOD_TIER_CONSOLIDATION Part I § Cluster H + Part II § CC-5, G6 § Per-module mutation + § Fuzz enumeration + § Integration matrix, KEYWORDS_20 § AP30 / Cluster, ANTIPATTERNS_REGISTER. No uncited claims.

---

## Sign-off

Cluster H plan authored 2026-05-17 by Command (parallel author for V7 optimisation). Planning-only per HOLD-v2 + AP24. ~180 tests across 3 modules; mutation kill targets 75/75/80 with m42 at 80% (dual-path cutover safety paramount). Owns no structural gap (Gap 1/2/3 all upstream); owns CC-5 emit half (the only substrate-grain loop per G3 substrate-frame pass). Shared `m40_42_common::Breaker` library across all 3 modules; outbox-first JSONL discipline absolute. Watcher Class A pre-positioned for first emit + Class B for every wire boundary + Class I for substrate-silence detection. Read with [[cluster-G.md]] (m32 upstream) + [[CROSS_CLUSTER_SYNERGIES.md]] (CC-5 deep contract).

*Luke @ node 0.A | Command @ Orchestrator | Watcher ☤ @ observing | Zen @ audit-lane | 2026-05-17 (S1001982)*
