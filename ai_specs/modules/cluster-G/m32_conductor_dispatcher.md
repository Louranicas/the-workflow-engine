---
title: m32 — `m32_conductor_dispatcher` Rust spec
cluster: G — Bank/Select/Dispatch/Verify
layer: L7
binary: wf-dispatch
loc_estimate: ~290
test_count_min: 75
test_kinds: [unit, property, integration, contract, regression, mutation]
feature_gate: [api]
verb_class: dispatch
cc_owns: [CC-4 dispatch-end, CC-5 fan-out trigger, CC-6 internal consumer]
cc_consumes: [CC-2 (m8/m9/m10), CC-3 indirectly via m31]
gap_owner: [Gap 3]
boilerplate_lift_pct: 35
status: SPEC
date: 2026-05-17
authority: Luke @ node 0.A
hold_v2_compliant: true
cardinality_amendment: "S1002127 — PrivilegeEscalation inserted at ordinal 30 (D-S1002127-02 ADR)"
---

# m32 — `m32_conductor_dispatcher` Rust spec

> Back to: [vault cluster-G spec](../../../the-workflow-engine-vault/module%20specs/cluster-G-bank-select-dispatch-verify.md) · [MODULE_MATRIX](../../MODULE_MATRIX.md) · [ARCHITECTURE](../../../ARCHITECTURE.md) · [V7 cluster-G plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-G.md) · [GENESIS v1.3](../../../ai_docs/GENESIS_PROMPT_V1_3.md) · vault [[cluster-G-bank-select-dispatch-verify]]
>
> Sister modules: [m30](m30_curated_bank.md) · [m31](m31_selector.md) · [m32](m32_conductor_dispatcher.md) · [m33](m33_verifier.md)

## 1. Purpose & invariants

`m32_conductor_dispatcher` IS the **only path** by which a verified workflow leaves m30's bank and reaches actual execution — and crucially, **m32 NEVER executes directly**. Every step routes through HABITAT-CONDUCTOR over its wire protocol; `wf-dispatch` is a sender, not an executor. m32 is the highest-stakes module in the workflow-trace engine (G6 mutation kill threshold ≥85%, highest in the engine) because a single surviving mutation in its 5-check pre-dispatch sequence could let a `DataExfil` workflow through with a softer banner or skip verification freshness. m32 owns the **Gap 3 display half** — the mandatory stdout banner derived from `m30::EscapeSurfaceProfile`, printed before each step, surfacing destructiveness to a human reading the terminal before Conductor receives the request.

The module MUST guarantee five invariants. **First — P0 #3 (no direct execution, AP-V7-08 self-dispatch refusal)**: m32's source must not contain `std::process::Command::new`, `tokio::process::Command`, `std::os::unix::process::CommandExt`, or any FFI exec call. The dispatch path is `conductor_client.dispatch_step(step).await`; that's the only egress. Additionally, m32 refuses to dispatch any workflow whose `steps` contain a step-kind that targets `m32` itself (self-dispatch loop), even when admission was permissive. **Second — Conductor-only routing with hard-refuse when unreachable**: when `CONDUCTOR_DISPATCH_ENABLED != "1"` or the `:8141/health` probe fails, every `dispatch()` call returns `DispatchError::ConductorDispatchDisabled` — NOT a silent no-op, NOT a fall-through-to-LCM, NOT a delayed retry. The error prints with the exact remediation banner so the operator can act. This is the Wave-1B/1C/2/3 maturity gate per [HABITAT-CONDUCTOR row in CLAUDE.local.md](../../../../CLAUDE.local.md). **Third — 5-check pre-dispatch sequence enforced in order**: every dispatch runs (1) Conductor live probe → (2) m33 verification freshness (TTL) → (3) `definition_hash` match against the version m33 verified → (4) `m30.sunset_at > now_ms` guard → (5) per-workflow `dispatch_cooldown` (escape-surface scaled). Any failure short-circuits with the *correct* typed `DispatchError` variant; later checks do not run. The order is not a suggestion — failures in early checks should not be masked by failures in later checks. **Fourth — display-before-step (Gap 3 half)**: for every `ResolvedStep` in the workflow, m32 writes the `EscapeSurfaceProfile::banner_line()` plus step kind plus trap annotations to stdout *before* `conductor_client.dispatch_step(step)` fires. This is human-visible output, not an optional log. Suppression via env var is forbidden; the banner is part of the dispatch contract. **Fifth — audit-first writes**: every dispatch attempt writes a row to `dispatch_log.db` *before* the Conductor request is sent (cribbed from `conductor_enforcement.rs` audit-first guarantee). If the audit write fails, the dispatch aborts; the row carries `outcome: Pending` until the Conductor response patches it to `Accepted | Rejected | Error(msg)`.

Frame violations m32 must structurally refuse: (a) any code path that calls `Command::*` or any process-spawn primitive — caught by clippy `disallowed_methods` lint + grep gate in CI; (b) self-dispatch (workflow whose steps target m32) — refused at dispatch entry with `EscapeSurfaceProfile::SandboxEscape` minimum classification and `DispatchError::SelfDispatchRefused`; (c) silent skip of any of the 5 checks (caught by F-Property test asserting "each disabled-check scenario returns the matching error variant"); (d) any HTTP server in the `wf-dispatch` binary — hard refusal per v1.3 § 2 (m32 communicates *out* to Conductor; never accepts inbound); (e) any namespace literal in serialised messages — `workflow_core::namespace` constants only.

## 2. Public surface (Rust types — spec only, NOT compileable)

```rust
//! # m32_conductor_dispatcher
//!
//! - **Layer**: L7 (Bank/Select/Dispatch/Verify, Cluster G)
//! - **Deps**: workflow_core::{types::*, errors::DispatchError, http::raw_http_get, namespace}, m30::{BankDb, AcceptedWorkflow, EscapeSurfaceProfile}, m33::VerificationResult
//! - **Tests**: 75-80 (40 unit + 5 property + 0 fuzz + 20 integration + 5 contract + 9 regression + mutation ≥85% — highest in engine)
//! - **Features**: api
//! - **Platform**: Linux; outbound-only HTTP (m24_povm_bridge gold-standard shape — no http:// prefix); SQLite audit log
//! - **Impl Notes**: 5-check orchestrator + banner + refuse_mode authored fresh (~150 LOC); conductor_enforcement.rs ~80% lift for audit-first; m24_povm_bridge wire shape lift; ~30 LOC of Gap 3 display half
//! - **Related Docs**: [vault cluster-G spec](../../../the-workflow-engine-vault/module%20specs/cluster-G-bank-select-dispatch-verify.md) · [V7 cluster-G](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-G.md) · [GENESIS v1.3 § 2](../../../ai_docs/GENESIS_PROMPT_V1_3.md)

#[derive(Debug, Clone)]
pub struct DispatcherConfig {
    /// HABITAT-CONDUCTOR address (NO http:// prefix; per m24_povm_bridge gold standard).
    pub conductor_addr: String,                   // default "127.0.0.1:8141"
    /// Set false by default; true only when env CONDUCTOR_DISPATCH_ENABLED=1.
    pub dispatch_enabled: bool,
    pub default_cooldown_ms: i64,                 // default 300_000 (5 min)
    pub cooldown_by_surface: std::collections::HashMap<EscapeSurfaceProfile, i64>,
    pub dispatch_log_path: std::path::PathBuf,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ConductorDispatchRequest {
    pub dispatch_id: String,                      // UUID v7; idempotency key
    pub workflow_id: WorkflowId,
    pub steps: Vec<ResolvedStep>,
    pub escape_surface: EscapeSurfaceProfile,
    pub dispatched_at: i64,
    pub operator: String,                         // "wf-dispatch/human" always
    pub verification_receipt: VerificationReceipt,
    pub dry_run: bool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ResolvedStep {
    pub id: StepId,
    pub kind: String,
    pub step_surface: EscapeSurfaceProfile,
    pub traps: Vec<TrapAnnotation>,
    pub conductor_params: serde_json::Value,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TrapAnnotation {
    pub id: String,                               // e.g., "T-BINARY-DEPLOY"
    pub category: String,                         // security | silent-failure | habitat-specific
    pub mitigation: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct VerificationReceipt {
    pub last_verified_at: i64,
    pub definition_hash: String,
    pub verdict: VerificationVerdict,
}

#[derive(Debug, serde::Serialize)]
pub struct WorkflowDispatchEvent {
    pub dispatch_id: String,
    pub workflow_id: WorkflowId,
    pub lineage: LineageId,
    pub step_count: usize,
    pub escape_surface: EscapeSurfaceProfile,
    pub dispatched_at: i64,
    pub conductor_accepted: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum DispatchError {
    #[error("dispatcher: Conductor not live at {addr}: {reason}")]
    ConductorNotLive { addr: String, reason: String },
    #[error("dispatcher: dispatch disabled — set CONDUCTOR_DISPATCH_ENABLED=1 + bring up Conductor Waves 1B/1C/2/3")]
    ConductorDispatchDisabled,
    #[error("dispatcher: verification stale for {workflow_id} (last: {last_verified_at:?})")]
    VerificationStale { workflow_id: WorkflowId, last_verified_at: Option<i64> },
    #[error("dispatcher: workflow definition drifted after verification (expected {expected}, got {actual})")]
    DefinitionDrifted { expected: String, actual: String },
    #[error("dispatcher: workflow {workflow_id} is past sunset")]
    Sunset { workflow_id: WorkflowId },
    #[error("dispatcher: dispatch cooldown active for {workflow_id} (retry at {retry_at})")]
    CooldownActive { workflow_id: WorkflowId, retry_at: i64 },
    #[error("dispatcher: self-dispatch refused (AP-V7-08) for {workflow_id}")]
    SelfDispatchRefused { workflow_id: WorkflowId },
    #[error("dispatcher: audit write failed: {0}")]
    AuditWrite(#[from] rusqlite::Error),
    #[error("dispatcher: Conductor rejected request: {reason}")]
    ConductorRejected { reason: String },
    #[error("dispatcher: serde: {0}")]
    Serde(#[from] serde_json::Error),
}

pub struct Dispatcher { /* private — refuse_mode vs live_mode internally */ }

impl Dispatcher {
    pub fn init(cfg: DispatcherConfig) -> Result<Self, DispatchError>;

    /// Resolve, verify-freshness, audit-write, banner-emit, send.
    pub async fn dispatch(
        &self,
        bank: &BankDb,
        verification: &VerificationResult,
        winner: &SelectedWorkflow,
        now_ms: i64,
    ) -> Result<WorkflowDispatchEvent, DispatchError>;
}
```

## 3. Internal data structures

`Dispatcher` is internally an enum `{ Refuse(RefuseInner), Live(LiveInner) }` so refuse-mode is a *type-level* discipline: `RefuseInner::dispatch()` is a literal `Err(ConductorDispatchDisabled)` and the borrow checker prevents any path that bypasses it. `LiveInner` holds the `ConductorClient` (outbound HTTP wrapper around `raw_http_get`/`raw_http_post` from `workflow_core::http`, modelled after `m24_povm_bridge`), the `AuditDb` handle, and a circuit breaker (lifted from `m40_42_common` per V7 plan) so a flapping Conductor opens the breaker without storming. A private `PreDispatchSequence` struct holds the 5 check functions as a `Vec<Box<dyn Fn(...) -> Result<(), DispatchError>>>` so test fixtures can swap individual checks for assertion (used by F-Property test 5-check order independence at *any* failing check). The `BannerEmitter` is a separate type so it can be redirected to a buffer in tests for stdout assertion.

### Cooldown ladder by escape surface (7-variant; D-S1002127-02)

`DispatcherConfig::cooldown_by_surface` defaults — escape-surface-scaled rate-limiting at check 5:

| Variant | Ordinal | Default cooldown | Banner |
|---|---:|---:|---|
| `Sandboxed` | 0 | 5 min (300_000 ms) | `SANDBOXED` |
| `SandboxEscape` | 10 | 10 min (600_000 ms) | `SANDBOX-ESCAPE` |
| `ProcessMutate` | 20 | 20 min (1_200_000 ms) | `PROCESS-MUTATE` |
| **`PrivilegeEscalation`** | **30** | **25 min (1_500_000 ms)** | **`PRIVILEGE-ESCALATION!`** |
| `FileWrite` | 40 | 30 min (1_800_000 ms) | `FILE-WRITE` |
| `NetworkEgress` | 50 | 45 min (2_700_000 ms) | `NETWORK-EGRESS!` |
| `DataExfil` | 60 | 60 min (3_600_000 ms) | `DATA-EXFIL!` |

`PrivilegeEscalation`'s 25-minute cooldown sits monotonically between `ProcessMutate` (20 min) and `FileWrite` (30 min), preserving the surface-scaled rate-limiting invariant. The default ladder is `HashMap<EscapeSurfaceProfile, i64>` keyed by variant; absence of an entry falls back to `default_cooldown_ms` (5 min). Operator override via `--bypass-cooldown` flag audits the bypass as a separate `DispatchAuditRow` event. The 5-check pre-dispatch sequence's check 5 (cooldown) consults this table; PrivilegeEscalation cooldowns are NOT bypassable without the `HumanAcceptanceSignature.privilege_escalation_acknowledged = true` field per m9 namespace-guard policy.

## 4. Data flow

- **INPUT FROM:** `m31::SelectedWorkflow` (winner); `m33::VerificationResult` (TTL fresh, `definition_hash`); `m30::BankDb::get(id)` (current row + steps for resolution); env `CONDUCTOR_DISPATCH_ENABLED`.
- **OUTPUT TO:** HABITAT-CONDUCTOR `:8141/dispatch` (NDJSON over HTTP per Wave-3 wire shape); `dispatch_log.db` audit table (`dispatch_log` shared file with main `db.sqlite` per Axis 2 single-DB pattern); fan-out `WorkflowDispatchEvent` channel consumed by m40 (SYNTHEX NexusEvent), m41 (LCM RPC for deploy-shaped workflows), m42 (POVM-decoupled per Appendix A — now stcortex-only Hebbian reinforce).
- **SUBSTRATE TOUCHED:** Conductor (mandatory); `dispatch_log.db` (own writes); m30 bank (`record_dispatch` on success); fan-out channel (in-process tokio::mpsc; non-blocking).
- **WRITES:** audit-first row in `dispatch_log`; `dispatch_count++` in m30 after Conductor accept; `WorkflowDispatchEvent` to fan-out channel (fire-and-forget; failure non-fatal but logged).

## 5. Algorithm sketch

```text
init(cfg):
    if !cfg.dispatch_enabled OR env CONDUCTOR_DISPATCH_ENABLED != "1":
        tracing::error!("Dispatcher initialised in REFUSE mode")
        return Ok(Dispatcher::Refuse(...))
    probe_conductor(&cfg.conductor_addr)?
        // raw_http_get(addr, "/health") via m24_povm_bridge shape (NO http:// prefix)
    Ok(Dispatcher::Live(LiveInner { client, audit, breaker, … }))

dispatch(bank, verification, winner, now_ms):
    if Refuse: return Err(ConductorDispatchDisabled)

    wf = bank.get(&winner.workflow_id)?.ok_or(BankRead(...))?

    // 5-CHECK PRE-DISPATCH SEQUENCE — order is contractual
    // (1) Conductor live
    probe_conductor(&self.cfg.conductor_addr).map_err(|e| ConductorNotLive { … })?

    // (2) Verification freshness (CC-6 closure)
    if !is_verification_fresh(&wf, verification, now_ms):
        return Err(VerificationStale { workflow_id: wf.id, last_verified_at: wf.last_verified_at })

    // (3) Definition hash match (drift detection)
    let resolved = resolve_steps(&wf.steps)?
    let current_hash = fnv1a_64_hex(&serde_json::to_vec(&resolved)?)
    if current_hash != verification.definition_hash:
        return Err(DefinitionDrifted { expected: verification.definition_hash.clone(), actual: current_hash })

    // (4) Sunset guard
    if wf.sunset_at <= now_ms:
        return Err(Sunset { workflow_id: wf.id })

    // (5) Per-workflow dispatch cooldown (scaled by escape surface)
    let cooldown = self.cfg.cooldown_by_surface.get(&wf.escape_surface_profile)
        .copied().unwrap_or(self.cfg.default_cooldown_ms)
    let last = self.audit.last_dispatched_at(&wf.id)?
    if last.map_or(false, |t| now_ms - t < cooldown):
        return Err(CooldownActive { workflow_id: wf.id, retry_at: last.unwrap() + cooldown })

    // Self-dispatch refusal (AP-V7-08) — defense-in-depth after m30 admission gate
    if resolved.iter().any(|s| s.kind == "m32_dispatch" || s.conductor_params.points_at_self()):
        return Err(SelfDispatchRefused { workflow_id: wf.id })

    // AUDIT-FIRST WRITE — before Conductor egress
    let dispatch_id = uuid_v7()
    self.audit.insert(DispatchAuditRow {
        dispatch_id, workflow_id: wf.id, dispatched_at: now_ms,
        conductor_addr: self.cfg.conductor_addr.clone(), step_count: resolved.len(),
        escape_surface: wf.escape_surface_profile.banner_line().into(),
        definition_hash: current_hash.clone(), operator: "wf-dispatch/human".into(),
        outcome: DispatchOutcome::Pending,
    })?  // failure aborts dispatch

    // DISPLAY-BEFORE-STEP (Gap 3 display half) — mandatory stdout
    BannerEmitter::for_workflow(&wf, &resolved, &verification).write_to_stdout()

    // CONDUCTOR SEND — the ONLY exec path
    let req = ConductorDispatchRequest { dispatch_id: dispatch_id.clone(), workflow_id: wf.id.clone(),
        steps: resolved.clone(), escape_surface: wf.escape_surface_profile,
        dispatched_at: now_ms, operator: "wf-dispatch/human".into(),
        verification_receipt: VerificationReceipt {
            last_verified_at: verification.verified_at,
            definition_hash: verification.definition_hash.clone(),
            verdict: verification.verdict,
        },
        dry_run: false,
    }
    let resp = self.client.send(req).await?
    self.audit.patch_outcome(&dispatch_id, DispatchOutcome::from(resp))?

    // SIDE-EFFECTS on success
    bank.record_dispatch(&wf.id, now_ms)?
    let event = WorkflowDispatchEvent { dispatch_id, workflow_id: wf.id.clone(), lineage: wf.lineage.clone(),
        step_count: resolved.len(), escape_surface: wf.escape_surface_profile,
        dispatched_at: now_ms, conductor_accepted: resp.accepted }
    fanout_tx.try_send(event.clone()).ok()                  // CC-5 fire-and-forget; non-blocking
    Ok(event)
```

## 6. Boilerplate lifts

Per vault cluster-G spec § m32 Boilerplate lift and V7 cluster-G Category 07/08:

| Source | Lift | % |
|---|---|---:|
| `conductor_enforcement.rs::EnforcerDb::process` | audit-first write pattern, `COOLDOWN_SECS` → `DISPATCH_COOLDOWN_MS` adaptation | 80% |
| `conductor_state.rs::StateDb` | WAL constructor, migration scaffolding, `kv_get/kv_set` for cooldown tracking | 70% |
| `m32_tier_executor.rs::Tier` | `GATE_Tn` confidence-gating shape → adapted as step-outcome gating + checkpoint/resume | 60% |
| `m24_povm_bridge.rs::raw_http_get` | outbound HTTP wire shape (NO http:// prefix; gold-standard `addr = "127.0.0.1:8141"`) | 100% (interface lift) |
| `conductor_api.rs::DeployRequest` | message shape adapted into `ConductorDispatchRequest` | 40% (reference only — HARD REFUSAL on HTTP server) |
| `m40_42_common::CircuitBreaker` | flapping-Conductor protection | 85% |
| **5-check orchestrator + banner emitter + refuse_mode + self-dispatch refusal** | — | **0% (novel ~150 LOC; Gap 3 display half ~30 LOC)** |

Net: ~140 LOC lifted / ~150 LOC novel (highest novel-LOC ratio in the cluster — dispatch security is paramount).

## 7. ME v2 m1_foundation patterns referenced

- `error.rs` — `DispatchError` thiserror enum, 9 variants with structured named fields (`{ addr, reason }`, `{ workflow_id, retry_at }`, `{ expected, actual }`); pattern-matchable; never `Box<dyn Error>`.
- `resources.rs` — `//!` docstring block.
- `state.rs` — `Dispatcher` as `{ Refuse | Live }` enum centralises the refuse-mode-as-type-discipline pattern.
- `shared_types.rs` — `WorkflowId`, `LineageId`, `StepId`, `DispatchId` newtypes.
- `config.rs` — `DispatcherConfig::Default` + env-override; `cooldown_by_surface` keyed by `EscapeSurfaceProfile`.
- `logging.rs` — tracing structured emit on every gate transition (which of the 5 checks failed, with what), every Conductor request (idempotency key), every banner emit (workflow_id, step count).
- `metrics.rs` — counter `dispatch_total`, gauge `conductor_breaker_open`, histogram `dispatch_latency_ms`; consumed by Prometheus per workspace charter.

## 8. Test strategy

- **Test kind**: unit (40) + property (5) + integration (20) + contract (5) + regression (9)
- **Test count**: 75-80 minimum (per [TEST_DISCIPLINE matrix](../../../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md) row m32; highest in cluster G; cluster total 290)
- **Mutation budget**: ≥**85% kill** on `pre_dispatch/*` + `banner.rs` + `refuse_mode.rs` (G6 m32 — highest threshold in engine). `cargo mutants --regex 'm32_dispatcher::.*'`; non-killed mutations require explicit `// IGNORE: cosmetic` rationale with Zen review.
- **Properties tested** (F-Property 5):
  - For each of the 5 checks individually: when that check fails and all others would pass, the returned error variant matches the check.
  - Refuse-mode is total: every `dispatch()` call returns `ConductorDispatchDisabled` (never `Ok`, never any other variant).
  - Audit write precedes Conductor send: a Conductor mock that records receive-time always sees an audit row dated `<= now`.
  - 5-check order: when checks A and B both would fail and A precedes B, the returned error is A's (later checks short-circuited).
  - Banner is emitted exactly `steps.len()` times per successful dispatch (never zero, never duplicated).

Key invariants (sample of 75; full enumeration in vault cluster-G spec § Tests m32):

1. `init()` with `dispatch_enabled=false` returns `Dispatcher::Refuse`.
2. `init()` with env `CONDUCTOR_DISPATCH_ENABLED != "1"` returns `Dispatcher::Refuse` regardless of cfg.
3. Refuse-mode `dispatch()` returns `ConductorDispatchDisabled` 1000/1000 calls.
4. Live-mode with `:8141` unreachable returns `ConductorNotLive { reason: "connection refused" }`.
5. Live-mode with `:8141` 200-OK but body `{"status":"degraded"}` returns `ConductorNotLive`.
6. Check 2: `last_verified_at = None` returns `VerificationStale`.
7. Check 2: `last_verified_at = now - 8d` (TTL=7d) returns `VerificationStale`.
8. Check 2: `last_verified_at = now - 6d` passes.
9. Check 3: tampering with one byte of `steps_json` between verify and dispatch returns `DefinitionDrifted`.
10. Check 4: `sunset_at = now` returns `Sunset` (strict `>`).
11. Check 5: cooldown 300s + last dispatch 200s ago returns `CooldownActive` with `retry_at = last + 300s`.
12. Cooldown is escape-surface scaled: `DataExfil` workflow gets longer cooldown than `Sandboxed`; `PrivilegeEscalation` (25 min) sits monotonically between `ProcessMutate` (20 min) and `FileWrite` (30 min) per D-S1002127-02.
13. Self-dispatch (step kind == "m32_dispatch") returns `SelfDispatchRefused`.
14. Banner emitted to stdout contains `EscapeSurfaceProfile::banner_line()` exact text.
15. Banner emitted for each step (3-step workflow → 3 banner blocks).
16. Audit row written before Conductor `send` (mocked Conductor records `audit.insert` `<=` `send`).
17. Audit write failure aborts dispatch (rusqlite::Error propagates; no Conductor send).
18. Audit row outcome patched to `Accepted` on Conductor 200.
19. Audit row outcome patched to `Rejected` on Conductor 4xx.
20. `WorkflowDispatchEvent` emitted to fan-out channel on success.
21. Fan-out channel full → `try_send` failure non-fatal (dispatch still Ok).
22. Idempotency: same `dispatch_id` retried → Conductor responds duplicate; m32 reports `ConductorRejected { reason: "duplicate" }`.
23. `record_dispatch` on m30 only called on Conductor accept (not on reject).
24. No `Command::*` symbol present anywhere in m32 source (grep gate in CI).
25. No `http://` literal in serialised Conductor request (m24_povm_bridge gold-standard wire shape).

(remaining 50 — including F-Contract bridge-contract skill run against Conductor schema, F-Integration full CC-4 + CC-6 closure cycle, F-Regression slots for AP-Drift-06 bridge-contract drift + cooldown-bypass + self-dispatch-bypass — enumerated in vault spec.)

## 9. Antipatterns to avoid

- **P0 #3 / AP-V7-08** (direct execution + self-dispatch) — `Command::*` lint-banned + grep-gated; self-dispatch refusal as defense-in-depth after m30 admission gate.
- **AP-V7-13** (Health-200 ≠ behaviour-verified) — Conductor probe checks both 200 status AND `body.status == "ok"`; not status-code alone.
- **AP-Drift-06** (bridge contract drift) — F-Contract tests run `bridge-contract` skill against live Conductor schema pre-merge.
- **AP-Drift-11** (supervisor stub mistaken for live) — Conductor probe requires real `/health` round-trip; init-time probe + per-dispatch probe (the latter is check 1).
- **AP-WT-F4** (premature dispatch) — 5-check sequence covers all known premature-dispatch surfaces.
- **AP-V7-09** (substrate-frame confusion) — m32 reads opaque ids; no human-meaningful pattern names cross the Conductor wire.
- **AP30** (namespace drift) — request fields use `workflow_core::namespace` constants; no literal `"workflow_trace_"` string.
- **AP29** (sync HTTP in `tokio::spawn`) — outbound is `async` via `workflow_core::http::raw_http_*`; no `block_on` in spawned context.
- **AP-Hab-04** (preserve-list discipline) — destructive steps still go to Conductor (m32 doesn't block them) but the banner SURFACES them; refusal is m33's job, surfacing is m32's.
- **AP-V7-13 (cousin)** — audit row written *before* Conductor egress; if Conductor lies about success, audit + log still show the attempt.
- **Newly surfaced**: refuse-mode bypass via re-init — `init()` is the ONLY path that decides `Refuse` vs `Live`; `Dispatcher` does not expose a re-init method; refuse-mode is permanent for the binary's lifetime (regression slot reserved).

## 10. Useful patterns applied

- ORAC `EnforcerDb` audit-first (PATTERNS.md § Module-level patterns; conductor_enforcement.rs).
- thiserror error enums (GOLD_STANDARDS rule 9).
- Newtype discipline (GOLD_STANDARDS rule 8).
- `//!` docstring block (GOLD_STANDARDS rule 13).
- Refuse-mode-as-type-discipline (`Dispatcher::Refuse | Live` enum); borrow-checker prevents accidental bypass.
- Circuit-breaker pattern (m40_42_common) — Conductor flapping does not storm.
- Idempotency-key (UUID v7 dispatch_id) — Conductor can dedupe replays.
- Outbound-only contract (no HTTP server) per v1.3 § 2 hard refusal; Axis 3 CLI-first.

## 11. Cross-cluster contracts

- **CC-4 dispatch-end (G owns F → G → Conductor)**: m32 is the final hop before Conductor; it consumes m31's `SelectedWorkflow` + m33's `VerificationResult` and produces a `ConductorDispatchRequest` over the wire. The Conductor schema is contract-binding; any change requires a `bridge-contract` skill run with m32 amended in lockstep.
- **CC-5 fan-out trigger (G → H)**: m32 emits `WorkflowDispatchEvent` to a tokio::mpsc channel consumed by Cluster H modules (m40 NexusEvent, m41 LCM RPC, m42 stcortex-only post-2026-05-17 ADR). The contract: fire-and-forget; m32 returns Ok regardless of fan-out success; channel-full causes a single tracing::warn but never aborts dispatch.
- **CC-6 internal consumer (G internal: m33 → m32)**: m32 reads `VerificationResult::definition_hash` + `verified_at` + `verdict`. The `definition_hash` must match `fnv1a_64_hex(resolved_steps_json)`; mismatch yields `DefinitionDrifted`. The TTL gate (check 2) is the only path that can re-trigger m33.
- **Gap 3 cross-cluster contract** (shared with Cluster D m9 + m30): m32 consumes `EscapeSurfaceProfile` from m30, uses its `banner_line()` for stdout display, and uses ordinal comparison for cooldown scaling. Reordering the enum is a verify-sync break across m9 + m30 + m32 + m33; reserved-numeric-gaps approach to be decided at Zen G7 (see Open Questions).

## 12. Open questions for G5 interview / Zen G7 audit

1. **EscapeSurfaceProfile ordinal stability across versions** (shared with m30 Open Q1) — **CLOSED by D-S1002127-02 (2026-05-17).** Cardinality bumped 6→7; PrivilegeEscalation inserted at ordinal 30; gaps reserved at steps of 10 (Sandboxed=0, SandboxEscape=10, ProcessMutate=20, PrivilegeEscalation=30, FileWrite=40, NetworkEgress=50, DataExfil=60). m32 cooldown ladder + banner table + self-dispatch refusal logic updated in lockstep.
2. **Conductor transport: HTTP NDJSON vs Unix socket vs gRPC**: vault spec says "Unix socket or HTTP endpoint (transport determined by Conductor Wave maturity)". Wave 1B HTTP is operationally simpler; Wave 2 WASM might need Unix socket. Lock transport at v1.3 or leave as Conductor-driven choice?
3. **`dispatch_log.db` vs `dispatch_log` table in main `db.sqlite`**: Axis 2 says single DB; vault spec mentions `dispatch_log.db` separately. Recommend single DB with `dispatch_log` table; verify-sync with the V7 plan.
4. **Cooldown scaling by escape surface**: should `DataExfil` cooldown be 1 hour while `Sandboxed` is 0? Default `cooldown_by_surface` table needs concrete numbers — current spec leaves it as `HashMap` with defaults at config time.
5. **Banner suppression in CI / batch mode**: forbidden in spec — but does CI need an exception (machine-readable stdout)? If yes, the exception must be a *separate output format flag*, not a banner suppression; banner stays mandatory.
6. **Self-dispatch detection mechanism**: current spec uses `step.kind == "m32_dispatch"` string match; vulnerable to renaming. Better: a typed `StepDef::is_self_dispatch_target() -> bool` method enforced at the workflow_core level so all callers agree.

## 13. Implementation order (post-G9)

1. `error.rs` — `DispatchError` enum (9 variants, `thiserror`); compile-only.
2. `refuse_mode.rs` — `RefuseInner` + total `dispatch()` always-Err; 6 unit tests.
3. `conductor_client.rs` — wrapper around `workflow_core::http::raw_http_*` (m24_povm_bridge shape); 8 unit + 5 contract tests (Conductor schema).
4. `pre_dispatch/check_1_conductor_live.rs` — probe + verdict; 6 unit tests.
5. `pre_dispatch/check_2_verify_fresh.rs` — TTL gate; 6 unit tests.
6. `pre_dispatch/check_3_definition_hash.rs` — FNV-1a compute + compare; 6 unit tests.
7. `pre_dispatch/check_4_sunset_guard.rs` — `now_ms > sunset_at`; 4 unit tests.
8. `pre_dispatch/check_5_dispatch_cooldown.rs` — last-dispatched + scaled cooldown; 8 unit tests.
9. `pre_dispatch/mod.rs` — 5-check orchestrator with explicit order; 4 unit tests (order-respected).
10. `banner.rs` — `BannerEmitter::for_workflow` (Gap 3 display half); 8 unit tests + 1 F-Property (per-step emission count).
11. `self_dispatch_guard.rs` — refuse if step kind targets m32; 4 unit tests.
12. `audit_db.rs` — `DispatchAuditRow` + `insert` + `patch_outcome` + `last_dispatched_at`; 8 unit tests.
13. `mod.rs` — `Dispatcher::init` + `dispatch` orchestration; 6 unit tests.
14. Property tests (5) — 5-check coverage, refuse-mode totality, audit-precedes-send, order-respected, banner-count.
15. Integration tests (20) — full CC-4 + CC-6 cycle, mocked Conductor with all response shapes, CC-5 fan-out closure, end-to-end with m30/m31/m33 wired.
16. Contract tests (5) — `bridge-contract` skill against Conductor `:8141` schema; insta snapshots for `ConductorDispatchRequest` JSON shape.
17. Regression slots (9) — AP-Drift-06, cooldown-bypass, self-dispatch-bypass, banner-suppression-attempt, audit-after-send, etc.
18. Mutation pass — `cargo mutants --regex 'm32_dispatcher::.*'`; ≥85% kill required (highest in engine; non-killed mutations require Zen sign-off).

## NA-GAP-11 refusal-observability amendment (Wave 4)

Per [`../../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md) NA-GAP-11, every m32 refusal MUST be engine-emitted as a first-class wire-protocol event, not Watcher-inferred from absence-of-dispatch. When any of the 5-check failures fires (`ConductorNotLive`, `VerificationStale`, `DefinitionDrifted`, `Sunset`, `CooldownActive`, `SelfDispatchRefused`, `ConductorDispatchDisabled`, `AuditWriteFailed`), m32 MUST:

1. Construct a `RefusalToken::EngineAuthored { invariant_id, refusal_class, recovery_hint, observed_at }` per [`../../cross-cutting/refusal-taxonomy.md`](../../cross-cutting/refusal-taxonomy.md).
2. Emit `WireEvent::Refusal { token, workflow_id: Some(wf.id), emitted_by: ModuleId::M32, emitted_at }` via m40's fire-and-forget channel to `/v3/nexus/push`.
3. Return the typed error to the caller.

Wire emission discipline closes the Class-C observability gap: refusal becomes a first-class event observable by SYNTHEX v2 / Watcher / m12 reports, not an absence-of-dispatch heuristic. Per the Conductor substrate dossier [`../../substrates/conductor.md`](../../substrates/conductor.md) § 3, substrate-authored refusals (`EnforcementDisabled`, `SemanticEndpointFailed`) get `RefusalToken::SubstrateAuthored { substrate_id: S-D, ... }` instead — m32 classifies the refusal authorship before emission.

**Invariant addition (Sixth):** every m32 refusal path emits `WireEvent::Refusal` to m40 fan-out before returning the typed error. Verification: integration test simulates each of the 7+ refusal scenarios; asserts wire emission fires for each.

---

> Back to: [vault cluster-G spec](../../../the-workflow-engine-vault/module%20specs/cluster-G-bank-select-dispatch-verify.md) · [MODULE_MATRIX](../../MODULE_MATRIX.md) · [ARCHITECTURE](../../../ARCHITECTURE.md) · sister modules: [m30](m30_curated_bank.md) · [m31](m31_selector.md) · [m32](m32_conductor_dispatcher.md) · [m33](m33_verifier.md) · NA remediation: [`../../substrates/conductor.md`](../../substrates/conductor.md) · [`../../cross-cutting/refusal-taxonomy.md`](../../cross-cutting/refusal-taxonomy.md)
