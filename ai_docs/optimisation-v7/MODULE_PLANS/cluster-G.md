---
title: MODULE PLAN — Cluster G (Bank + Select + Dispatch + Verify) · m30 / m31 / m32 / m33
date: 2026-05-17
kind: planning-only · per-module spec · no code authorised (HOLD-v2 + AP24)
cluster: G
layer: L7
modules: [m30, m31, m32, m33]
loc_budget: ~950
test_budget: 290
mutation_kill_targets: { m30: 75%, m31: 80%, m32: 85%, m33: 80% }
structural_gap_partial: "Gap 3 — EscapeSurfaceProfile schema (m30 owns) + display-before-step banner (m32 owns) — ~150-250 LOC"
authority: Command · workflow-trace V7 optimisation · Command-3 librarian lane
---

# Cluster G — Bank + Select + Dispatch + Verify (L7)

> Back to: [[../TASK_LIST_V7_OPTIMISATION.md]] · [[../ULTRAMAP.md]] · [[../GENERATIONS/G3-bidi-flow.md]] · sibling [[cluster-F.md]] · [[cluster-H.md]] · [[CROSS_CLUSTER_SYNERGIES.md]]
>
> **Function:** Cluster G is the engine's **operational core** — the four-module pipeline (curated bank + diversity-enforced selector + Conductor-only dispatcher + 4-agent verifier) that turns accepted `WorkflowProposal` artefacts into actual workflow executions via HABITAT-CONDUCTOR. Per [[../KEYWORDS_20.md]] § Conductor: *m32 NEVER executes workflows directly — only routes via Conductor (P0 #3)*. Per [[../KEYWORDS_20.md]] § EscapeSurfaceProfile: Cluster G owns the **Gap 3** unified destructiveness schema (m30 schema + m32 banner). Per [[../KEYWORDS_20.md]] § two-binary: Cluster G is the entire `wf-dispatch` binary. Command-3 owns the librarian lane for the cluster.

---

## Overview

Cluster G sits at L7 in the layer DAG, downstream of Cluster F (m23 proposer) + Cluster D (m11 decay) + Cluster E (m14 lift) and upstream of Cluster H (m40-m42 substrate feedback via dispatch outcome). The four-module pipeline is strictly linear at the *control-flow* level (m30 admits → m31 selects → m33 verifies → m32 dispatches via Conductor) but cross-cuts at the *data-flow* level (m11 modulates m31 selection; m14 modulates m31 weighting; m33 freshness gates m32 dispatch).

The cluster has three load-bearing discipline rules:
1. **m30 NEVER auto-promotes** (F5 mitigation) — admission requires explicit `wf-crystallise propose accept <id>` from a human
2. **m32 NEVER dispatches directly** (P0 #3) — Conductor-only routing; refuse-mode (`DispatchError::ConductorDispatchDisabled`) is the safe-path behaviour when Conductor unreachable
3. **m33 verification TTL is 7 days HARD** — stale verifies refuse dispatch via m32's 5-check sequence

Per ULTRAMAP View 2: ~950 LOC across 4 modules (~240 LOC/module avg) with ~60-70% boilerplate-lift density (per [[../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md]] Part I § Cluster G). 290 tests budgeted (avg 72/module; m32 = 80 highest in cluster). Mutation kill 75/80/85/80; m32 at 85% is the highest threshold in the entire engine — dispatch security is paramount.

---

## m30 — curated bank (L7 · `src/m30_bank/`)

### Purpose

Persistent registry of accepted workflows + their `EscapeSurfaceProfile` classification + `definition_hash` + `sunset_at` immutable deadline. m30 is the **single source of truth** for "what workflows can be dispatched at all". Admission is human-gated (F5 mitigation); sunset is enforced (F1 mitigation via m11 decay + immutable `sunset_at`). m30 owns the **Gap 3 EscapeSurfaceProfile schema** (per [[../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md]] Part III Gap 3).

### Edge contract (from [[../GENERATIONS/G3-bidi-flow.md]] § m30)

- **Upstream-IN:** `m23.WorkflowProposal` post-`wf-crystallise propose accept <id>` (NEVER auto-promote — F5)
- **Downstream-OUT:** `BankEntry { workflow_id, definition_hash, escape_surface: EscapeSurfaceProfile, sunset_at }` → m31 (selector reads), m32 (dispatcher resolves), m33 (verifier reads)
- **Aspect-IN:** m8 build_prereq, m9 namespace_guard, m10 Ember CI
- **Failure-mode mitigated:** F5 (bank creep — hard refusal on auto-promote); F1 (immutable sunset_at boundary); AP-Hab-03 (namespace prefix discipline at admission)

### src/ path (planning-spec only)

```
src/m30_bank/
├── mod.rs                # public: pub struct BankEntry, pub enum EscapeSurfaceProfile, pub struct Bank
├── escape_surface/
│   ├── mod.rs            # EscapeSurfaceProfile ordinal enum + Display/Ord/Hash
│   ├── classifier.rs     # workflow steps → EscapeSurfaceProfile classifier (Gap 3 schema)
│   └── lookup.rs         # step-token → escape-level lookup table (from skill files unification)
├── admission.rs          # admit_workflow() — F5 explicit-human-accept enforcement
├── persistence.rs        # SQLite-backed bank (Cluster D Cat 03 lift pattern)
├── definition_hash.rs    # FNV-1a hash of steps_json (consumed by m32 5-check)
├── sunset.rs             # immutable sunset_at + m11 decay coordination
└── tests/
```

### LOC budget

~250 LOC (per ULTRAMAP View 2). Composition:
- escape_surface/ ~80 LOC (Gap 3 schema + classifier — fresh authorship per Part III)
- admission.rs ~50 LOC (F5 enforcement)
- persistence.rs ~60 LOC (SQLite scaffold; ~90% lifted from Cat 03 m06_schema)
- definition_hash.rs ~20 LOC (FNV-1a)
- sunset.rs ~30 LOC
- mod.rs + glue ~10 LOC

### Test budget (70 tests; per [[../STANDARDS/TEST_DISCIPLINE.md]] § m30)

| Family | Count | Notes |
|---|---:|---|
| F-Unit | 35 | per EscapeSurfaceProfile variant; admission F5 paths; definition_hash determinism |
| F-Property | 5 | invariants: EscapeSurfaceProfile total-order respected · sunset_at monotonic · admission idempotent on same workflow_id |
| F-Fuzz | 0 | (m20 already covers PrefixSpan input fuzz; m30 doesn't parse external bytes) |
| F-Integration | 15 | m23→m30 admission flow; m30→m31 read; m30→m32 resolution; m30→m33 verification handshake |
| F-Contract | 5 | BankEntry schema parity (serde stable); EscapeSurfaceProfile Display format stable (consumed by m32 banner) |
| F-Regression | 9 | reserved — high test count rationale: 9 F5/F1/Gap 3 failure scenarios pre-seeded |
| F-Mutation | 1 budget (≥**75%** per G6) | escape_surface classifier criticality |

### Mutation kill threshold

**75%** (G6 m30 = cluster G m30 threshold). EscapeSurfaceProfile classifier mutations must die (a mutation flipping `SandboxEscape → Network` could let a destructive workflow through with a softer banner). admission.rs auto-promote-refusal mutations must die.

### Boilerplate-lift source

Per [[../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md]] Part IV Category 03 (SQLite multi-DB, 90% reuse) + Category 09 (Trap/verify/escape, 95% reuse for skill-file unification):
- **m06_schema** SQLite scaffold ~90% lifted for persistence.rs
- **SKILL-forge.md / SKILL-genesis.md / SKILL-pre-deploy-hardening.md / SKILL-silent-swallow-detect.md / hookify.preserve-blanket-guard.local.md** classifier intelligence ~95% lifted for the EscapeSurfaceProfile classifier (5 scattered classifiers unified into ordinal enum)

Fresh authorship: ~80 LOC (EscapeSurfaceProfile ordinal enum + classifier unification). Lifted: ~170 LOC.

### Structural-gap LOC

**Gap 3 schema owner** (per [[../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md]] Part III Gap 3). m30 owns ~80 LOC of the ~150-250 LOC Gap 3 budget (schema + classifier); the rest distributes to m32 (banner display ~30 LOC) + m9 (namespace guard at write boundary ~50 LOC; counted in Cluster D).

### EscapeSurfaceProfile ordinal enum spec (planning-spec only)

```rust
// planning-spec only — m30_bank::escape_surface
// rationale: Gap 3 unified destructiveness schema; replaces scattered classifiers in
// SKILL-forge, SKILL-genesis, SKILL-pre-deploy-hardening, SKILL-silent-swallow-detect,
// hookify.preserve-blanket-guard.local.md (S102 scar tissue)
// Per [[../KEYWORDS_20.md]] § EscapeSurfaceProfile

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscapeSurfaceProfile {
    /// Read-only: cat, ls, grep, curl HEAD/GET against allowlisted hosts
    ReadOnly,
    /// Host filesystem write: edit, write, mkdir, atomic-rename to user-owned paths
    HostWrite,
    /// Network egress: HTTP POST/PUT, ssh, git push to non-allowlisted remote
    Network,
    /// Sandbox-escape risk: setsid, nohup, sudo, docker exec into non-this-container
    SandboxEscape,
    /// Destructive: rm -rf, drop database, docker prune, force-push to main
    Destructive,
}

impl EscapeSurfaceProfile {
    /// Format for m32 dispatcher banner (mandatory stdout per Part III Gap 3)
    pub fn banner_line(&self) -> &'static str {
        match self {
            Self::ReadOnly      => "[ESCAPE-SURFACE: read-only]",
            Self::HostWrite     => "[ESCAPE-SURFACE: host-write]",
            Self::Network       => "[ESCAPE-SURFACE: network-egress]",
            Self::SandboxEscape => "[ESCAPE-SURFACE: SANDBOX-ESCAPE — display before each step]",
            Self::Destructive   => "[ESCAPE-SURFACE: DESTRUCTIVE — display before each step + cancel-on-uncertain]",
        }
    }
}
```

### Failure-modes covered

| Failure | Mechanism |
|---|---|
| F5 (bank creep) | admit_workflow() requires explicit `accepted_by: HumanAcceptanceSignature` field; never `auto` |
| F1 (ossification) | sunset_at immutable at admission; m11 decay drives entries toward sunset_at; auto-sunset post-deadline |
| Gap 3 (destructiveness scatter) | unified EscapeSurfaceProfile ordinal enum replaces 5 scattered classifiers |
| AP-Hab-03 (AP30) | m9 namespace_guard validates all workflow_id values at admission |

### Atuin trajectory anchor

`wt-bank-admit` (proposed; T5.2): every admission writes `(workflow_id, escape_surface, definition_hash, sunset_at, accepted_by)` to atuin. `wt-bank-sunset` daily cron captures entries reaching sunset_at.

### Watcher class pre-position

- **Class A** (activation) at first BankEntry admission post-G9 — the moment the bank has ≥1 entry is the moment dispatch becomes possible.
- **Class C** (refusal) at every F5 rejection (attempted auto-promote refused).
- **Class D** (four-surface drift) on Gap 3 schema drift between m30 (definition) and m32 (banner usage).

---

## m31 — selector (L7 · `src/m31_selector/`)

### Purpose

Select **one workflow** from m30's bank for next dispatch, using a composite score `α·fitness + β·recency + γ·frequency + δ·diversity` (0.40/0.25/0.20/0.15 per [[../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md]] Part I § Cluster G). Diversity algebra adapted from ORAC's `m40_mutation_selector`: 10-gen cooldown + 50% mono-parameter rejection + round-robin cycling. m31 is `Option<SelectedWorkflow>::None` if no workflow clears the score floor — NoSelection is the safe path against degraded substrate (LTP/LTD = 0.043 currently; per GAP-Substrate-01 in [[../GENERATIONS/G6-test-discipline.md]]).

### Edge contract

- **Upstream-IN:** `m30.BankEntry` + `m11.DecayFactor` (per-workflow) + `m14.Lift` (modulation `clamp(-0.3, +0.3)`) + diversity-cooldown state
- **Downstream-OUT:** `Option<SelectedWorkflow { workflow_id, composite_score, diversity_features }>` → m32
- **Aspect-IN:** m8, m9
- **Failure-mode mitigated:** F1 (composite-score deterministic ordering); BUG-035 (mono-parameter mutation trap — 50% rejection rule adopted from ORAC); GAP-Substrate-01 (degraded-substrate safety — NoSelection)

### src/ path (planning-spec only)

```
src/m31_selector/
├── mod.rs                # public: pub struct SelectedWorkflow, pub fn select
├── composite_score.rs    # α·fitness + β·recency + γ·frequency + δ·diversity
├── diversity/
│   ├── mod.rs            # adapted from ORAC m40_mutation_selector
│   ├── cooldown.rs       # 10-gen cooldown table
│   ├── mono_parameter.rs # 50% mono-parameter rejection
│   └── round_robin.rs    # cycling state
├── substrate_check.rs    # NoSelection on degraded LTP/LTD (GAP-Substrate-01)
└── tests/
```

### LOC budget

~250 LOC. Composite-score math ~30 LOC; diversity algebra ~120 LOC (ORAC-pattern lift); substrate_check ~30 LOC; persistence/cooldown state ~50 LOC.

### Test budget (70 tests; per [[../STANDARDS/TEST_DISCIPLINE.md]] § m31)

| Family | Count |
|---|---:|
| F-Unit | 35 |
| F-Property | 8 (composite-score monotonic in each component; 10-gen cooldown FIFO; round-robin fairness) |
| F-Fuzz | 0 |
| F-Integration | 15 |
| F-Contract | 5 |
| F-Regression | 6 |
| F-Mutation | 1 budget (≥**80%** per G6) |

**Plus GAP-Substrate-01 integration test** per [[../GENERATIONS/G6-test-discipline.md]]: `tests/integration/m31_against_degraded_substrate.rs` replays captured LTP/LTD=0.043 stcortex snapshot; m31 either returns NoSelection OR emits Watcher Class-I flag.

### Mutation kill threshold

**80%** (G6 m31). Composite-score weighting mutations must die (a mutation flipping α=0.40 → 0.04 silently shifts selection distribution). Cooldown table mutations must die (a mutation collapsing the 10-gen window lets BUG-035 mono-parameter trap recur).

### Boilerplate-lift source

Per [[../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md]] Part IV: ORAC `m40_mutation_selector` diversity algebra ~70% lifted; LCM TierExecutor selection patterns ~50% lifted. Fresh: composite-score numerics + substrate_check.

Fresh authorship: ~80 LOC (composite-score + substrate_check + Cluster F integration). Lifted: ~170 LOC.

### Structural-gap LOC

None (Gap 3 lives in m30/m32/m9; Gap 1 in Cluster F; Gap 2 in m11). m31 is *composition* of existing primitives.

### Failure-modes covered

| Failure | Mechanism |
|---|---|
| F1 (ossification) | composite-score includes recency (β=0.25) + diversity (δ=0.15) — ossified workflows lose to fresher alternatives |
| BUG-035 (ORAC mono-parameter) | 50% mono-parameter rejection rule adopted; cooldown table FIFO |
| GAP-Substrate-01 (degraded substrate) | NoSelection when LTP/LTD < 0.05; Watcher Class-I flag emit |
| AP-V7-09 (substrate-frame) | all inputs operationalised (no "user preferred X" surrogate) |

### Atuin trajectory anchor

`wt-selector-snapshot` (proposed): every selection (or NoSelection) writes `(timestamp, candidates_count, selected_workflow_id_or_none, composite_score, diversity_features)` to atuin.

### Watcher class pre-position

- **Class C** (refusal) at every NoSelection — refusal is correct against degraded substrate.
- **Class I** (Hebbian silence) if m31 selects against substrate where pathway weights never moved — CC-5 broken.

---

## m32 — dispatcher (L7 · `src/m32_dispatcher/`)

### Purpose

Dispatch the m31-selected workflow via **HABITAT-CONDUCTOR only** (P0 #3 per [[../KEYWORDS_20.md]] § Conductor). Executes the 5-check pre-dispatch sequence; emits EscapeSurfaceProfile banner before each step (Gap 3 display half); refuse-mode when Conductor unreachable. m32 is the **highest-stakes module in the engine** — dispatch security is paramount; mutation kill ≥85% (highest in engine per G6).

### Edge contract

- **Upstream-IN:** `m31.SelectedWorkflow` + `m33.VerifyResult` (TTL fresh) + Conductor health (`:8141/health`)
- **Downstream-OUT:** dispatch via HABITAT-CONDUCTOR ONLY → workflow exec; outcome → m40/m41/m42 (Cluster H)
- **Aspect-IN:** m8, m9, m10
- **Failure-mode mitigated:** F4 (premature dispatch — 5-check pre-dispatch sequence); refuse-mode = `DispatchError::ConductorDispatchDisabled` (NOT silent)

### src/ path (planning-spec only)

```
src/m32_dispatcher/
├── mod.rs                # public: pub fn dispatch, pub enum DispatchError
├── pre_dispatch/
│   ├── mod.rs            # 5-check orchestrator
│   ├── check_1_conductor_live.rs    # :8141/health probe
│   ├── check_2_verify_fresh.rs      # m33.VerifyResult.ttl_expires_at > now
│   ├── check_3_definition_hash.rs   # FNV-1a(steps_json) == m33.definition_hash
│   ├── check_4_sunset_guard.rs      # m30.BankEntry.sunset_at > now
│   └── check_5_dispatch_cooldown.rs # m32.last_dispatched_at(workflow_id) + cooldown < now
├── conductor_client.rs   # Conductor :8141 HTTP client (circuit-breaker shared from m40_42_common)
├── banner.rs             # EscapeSurfaceProfile banner stdout (Gap 3 display half)
├── refuse_mode.rs        # DispatchError::ConductorDispatchDisabled emission
└── tests/
```

### LOC budget

~250 LOC. 5-check pre-dispatch ~80 LOC (one file per check); conductor_client ~80 LOC (HTTP + breaker); banner ~30 LOC (Gap 3 display); refuse_mode ~20 LOC; orchestration ~40 LOC.

### Test budget (80 tests; per [[../STANDARDS/TEST_DISCIPLINE.md]] § m32 — highest in cluster)

| Family | Count |
|---|---:|
| F-Unit | 40 |
| F-Property | 5 (5-check order independence at *any* failing check; refuse-mode total) |
| F-Fuzz | 0 |
| F-Integration | 20 (one per check × multiple scenarios; CC-4 + CC-6 closure exercises) |
| F-Contract | 5 (Conductor wire-protocol contract; bridge-contract skill run) |
| F-Regression | 9 |
| F-Mutation | 1 budget (≥**85%** per G6 — highest in engine) |

### Mutation kill threshold

**85%** (G6 m32 = highest threshold in engine). Rationale: any surviving mutation in the 5-check sequence could let a destructive workflow through silently. `cargo mutants --regex 'm32_dispatcher::.*'` enforces; non-killed mutations require explicit `// IGNORE: cosmetic` rationale with Zen review.

### Boilerplate-lift source

Per [[../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md]] Part IV Category 07 (Conductor dispatch, 40% reuse — *BLOCKED on Wave maturity* per B3 blocker): `state.rs` adapted ~40%. Per Category 08 (Nexus-LCM-RPC, 85% reuse): circuit-breaker pattern from m24_povm_bridge.

Fresh authorship: ~150 LOC (5-check orchestrator + banner + refuse_mode + Conductor-specific HTTP client). Lifted: ~100 LOC.

### Structural-gap LOC

**Gap 3 banner display half** (~30 LOC). m32's banner.rs is the *display-before-step* mandate (per Part III Gap 3: *m32 displays profile banner before each step (mandatory stdout)*). Pairs with m30's schema definition.

### Failure-modes covered

| Failure | Mechanism |
|---|---|
| F4 (premature dispatch) | 5-check sequence enforces Conductor live + m33 TTL fresh + definition_hash match + sunset guard + cooldown |
| P0 #3 (direct execution forbidden) | conductor_client.rs is the ONLY dispatch path; refuse-mode total when client unavailable |
| Gap 3 display | banner.rs stdout mandatory before each step |
| AP-Drift-06 (bridge contract drift) | F-Contract tests run `bridge-contract` skill against Conductor schema pre-merge |

### m32 5-check pre-dispatch sequence (spec)

```
INPUT:  SelectedWorkflow { workflow_id, composite_score, .. } from m31
        VerifyResult     { definition_hash, ttl_expires_at, verdict } from m33

CHECK 1 — Conductor live:
    GET http://localhost:8141/health (timeout 2s)
    FAIL → return Err(DispatchError::ConductorDispatchDisabled)
    PASS → continue

CHECK 2 — Verify fresh:
    require: m33.ttl_expires_at > now
    FAIL → return Err(DispatchError::VerificationStale(workflow_id))
    PASS → continue

CHECK 3 — Definition hash match:
    let current_hash = fnv1a_64(serde_json::to_vec(&m30.lookup(workflow_id).steps_json)?);
    require: current_hash == m33.definition_hash
    FAIL → return Err(DispatchError::DefinitionDrift { expected: m33.definition_hash, actual: current_hash })
    PASS → continue

CHECK 4 — Sunset guard:
    require: m30.lookup(workflow_id).sunset_at > now
    FAIL → return Err(DispatchError::WorkflowSunset(workflow_id))
    PASS → continue

CHECK 5 — Dispatch cooldown:
    require: m32.last_dispatched_at(workflow_id) + dispatch_cooldown_for(escape_surface) < now
    FAIL → return Err(DispatchError::CooldownActive { workflow_id, retry_at })
    PASS → continue

EXECUTE:
    for step in workflow.steps {
        stdout.write(escape_surface.banner_line());   // Gap 3 mandatory display
        conductor_client.dispatch_step(step).await?;
    }
    emit_outcome_to_cluster_h(DispatchOutcome { workflow_id, outcome });
```

### Atuin trajectory anchor

`wt-dispatch-attempt` (proposed): every dispatch attempt writes `(workflow_id, check_outcomes, dispatch_result_or_error)` to atuin. `wt-dispatch-refused` daily aggregation captures refuse-mode counts (Watcher Class-C signal).

### Watcher class pre-position

- **Class A** (activation) at first successful dispatch post-G9 — the moment a real workflow runs via the engine is the highest-leverage observation.
- **Class C** (refusal) at every refuse-mode emission — Conductor-unavailable refusals are correct behaviour.
- **Class B** (hand-off boundary) at every Conductor dispatch call — cross-substrate boundary crossing.

---

## m33 — 4-agent verifier (L7 · `src/m33_verifier/`)

### Purpose

Verify a workflow against 4-agent gate (Security + Performance + SilentFailure + Zen) and emit a typed `VerifyResult` with 7-day TTL and `definition_hash`. m32 consults m33 in the 5-check pre-dispatch sequence; stale verifies block dispatch. The 4-agent gate IS the engine's quality-gate at dispatch boundary.

### Edge contract

- **Upstream-IN:** `m30.BankEntry` (verify request triggered by m32 staleness OR explicit `wf-dispatch verify <workflow_id>`)
- **Downstream-OUT:** `VerifyResult { workflow_id, verdict: Pass | Fail | Degraded, verified_at, ttl_expires_at, definition_hash }` → m32 (read at dispatch)
- **Aspect-IN:** m8, m9, m10
- **Failure-mode mitigated:** F4 (premature dispatch — TTL enforcement); AP-Drift-11 (supervisor stub mistaken for live — 4-agent live-verify mandatory)

### src/ path (planning-spec only)

```
src/m33_verifier/
├── mod.rs                # public: pub struct VerifyResult, pub enum Verdict, pub fn verify
├── agents/
│   ├── mod.rs            # 4-agent dispatch orchestrator
│   ├── security.rs       # security-auditor invocation
│   ├── performance.rs    # performance-engineer invocation
│   ├── silent_failure.rs # silent-failure-hunter invocation
│   └── zen.rs            # Zen audit invocation
├── ttl.rs                # 7-day TTL enforcement
├── definition_hash.rs    # FNV-1a hash (consumed by m32 check 3)
├── persistence.rs        # VerifyResult cache (SQLite)
└── tests/
```

### LOC budget

~200 LOC. 4-agent dispatch ~80 LOC; TTL + definition_hash ~30 LOC; persistence ~50 LOC; orchestration ~40 LOC.

### Test budget (70 tests; per [[../STANDARDS/TEST_DISCIPLINE.md]] § m33)

| Family | Count |
|---|---:|
| F-Unit | 35 |
| F-Property | 5 (TTL monotonic; 4-agent verdict composition correct) |
| F-Fuzz | 0 |
| F-Integration | 18 |
| F-Contract | 5 |
| F-Regression | 6 |
| F-Mutation | 1 budget (≥**80%** per G6) |

### Mutation kill threshold

**80%** (G6 m33). TTL enforcement mutations must die; 4-agent verdict composition mutations must die.

### Boilerplate-lift source

Per [[../the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md]] Part IV Category 09 (SKILL-pre-deploy-hardening 4-agent gate ~95% lifted). Per Category 03 (SQLite persistence ~90% lifted for VerifyResult cache).

Fresh authorship: ~50 LOC (TTL + Cluster G integration). Lifted: ~150 LOC.

### Structural-gap LOC

None (cluster G's Gap 3 lives in m30+m32+m9).

### Failure-modes covered

| Failure | Mechanism |
|---|---|
| F4 | 7-day TTL hard; m32 check 2 enforces; stale verifies block dispatch |
| AP-Drift-11 (supervisor stub) | 4-agent live-verify mandatory; FP-verify discipline applied per [[../ANTIPATTERNS_REGISTER.md]] |
| Gap 3 (verification gate) | EscapeSurfaceProfile level modulates 4-agent threshold (Destructive requires UNANIMOUS Pass; ReadOnly requires majority) |

### m33 4-agent verify gate spec

```
INPUT:  BankEntry { workflow_id, definition_hash, escape_surface }

PARALLEL DISPATCH:
    a1 = security-auditor.verify(workflow_id)
    a2 = performance-engineer.verify(workflow_id)
    a3 = silent-failure-hunter.verify(workflow_id)
    a4 = zen.verify(workflow_id)
    awaitAll(a1, a2, a3, a4) with timeout=300s

COMPOSE VERDICT (modulated by escape_surface):
    match escape_surface {
        ReadOnly | HostWrite =>
            // Majority Pass required
            verdict = if pass_count(verdicts) >= 3 { Pass } else { Fail }
        Network | SandboxEscape =>
            // 3 of 4 Pass required; any Fail blocks
            verdict = if fail_count(verdicts) == 0 && pass_count(verdicts) >= 3 { Pass } else { Fail }
        Destructive =>
            // UNANIMOUS Pass required; any Degraded or Fail blocks
            verdict = if all_pass(verdicts) { Pass } else { Fail }
    }

EMIT:
    VerifyResult {
        workflow_id,
        verdict,
        verified_at: now,
        ttl_expires_at: now + Duration::days(7),
        definition_hash,
    }
```

### Atuin trajectory anchor

`wt-verify-result` (proposed): every VerifyResult emit captured to atuin with per-agent verdicts.

### Watcher class pre-position

- **Class C** (refusal) at every Fail verdict — refusal is correct behaviour.
- **Class B** (hand-off boundary) at each 4-agent dispatch (4 cross-pane boundary crossings per verify).

---

## Cluster-level synergies

### CC-4 — Proposal → Bank → Dispatch (F → G → Conductor)

Cluster G is the downstream half of CC-4. m23.WorkflowProposal → human accept → m30.admit → m31.select → m33.verify → m32.dispatch → Conductor. Linear flow; no shortcuts.

### CC-5 — Substrate Learning Loop (G → H → back to F)

Cluster G's m32 dispatch outcome IS the trigger for CC-5. DispatchOutcome → Cluster H (m40/m41/m42) → substrate weights → next session m31 selection shifts. Loop is intentionally slow (days/weeks).

### CC-6 — Verification-Gated Dispatch (G internal: m33 → m32)

Cluster G internal closure. m32's 5-check sequence (specifically checks 2 and 3) consults m33.VerifyResult. Stale or hash-drift cases trigger re-verify before dispatch.

### Intra-cluster

m30 ↔ all (m31 reads, m32 resolves, m33 reads). m30 is the cluster's read-side hub.
m31 → m32 (selection feeds dispatch).
m33 → m32 (verify feeds dispatch).
m31 ↮ m33: independent (selection and verification are orthogonal concerns; coupling would create non-determinism).

---

## Cluster-level antipatterns (Cluster G specific)

| ID | Antipattern | Mitigation |
|---|---|---|
| AP-WT-F4 | premature dispatch | m32 5-check pre-dispatch sequence (above) |
| AP-WT-F5 | bank creep | m30 admit_workflow F5 enforcement |
| AP-WT-F1 | bank ossification | m30 immutable sunset_at + m11 decay |
| AP-Drift-06 | Conductor wire-contract drift | F-Contract tests + `bridge-contract` skill |
| AP-Drift-11 | supervisor stub mistaken for live | m33 4-agent live-verify mandatory |
| BUG-035 (ORAC) | mono-parameter mutation trap | m31 50% mono-parameter rejection rule |
| Gap 3 destructiveness scatter | EscapeSurfaceProfile unified schema (m30) + banner display (m32) |

---

## Citation discipline

Every claim cites ULTRAMAP View 2, G3 § Cluster G, GOD_TIER_CONSOLIDATION Part I + Part III Gap 3, G6 § Per-module mutation, KEYWORDS_20 § Conductor + EscapeSurfaceProfile, ANTIPATTERNS_REGISTER. No uncited claims.

---

## Sign-off

Cluster G plan authored 2026-05-17 by Command (parallel author for V7 optimisation; Command-3 owns librarian lane at execution time). Planning-only per HOLD-v2 + AP24. ~290 tests across 4 modules; mutation kill targets 75/80/85/80 with m32 at 85% (highest in engine — dispatch security paramount). Owns Gap 3 (~150-250 LOC EscapeSurfaceProfile schema + display banner). CC-4 consumer + CC-5 trigger + CC-6 internal. Watcher Class A pre-positioned for first dispatch + Class C for every refuse-mode + Class B for Conductor hand-offs. Read with [[cluster-F.md]] (m23 upstream) + [[cluster-H.md]] (m40-m42 downstream) + [[CROSS_CLUSTER_SYNERGIES.md]] (CC-4/5/6 deep contracts).

*Luke @ node 0.A | Command @ Orchestrator | Command-3 @ librarian | Watcher ☤ @ observing | Zen @ audit-lane | 2026-05-17 (S1001982)*
