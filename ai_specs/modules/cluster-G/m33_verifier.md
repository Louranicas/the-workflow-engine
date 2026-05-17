---
title: m33 — `m33_verifier` Rust spec
cluster: G — Bank/Select/Dispatch/Verify
layer: L7
binary: wf-dispatch
loc_estimate: ~200
test_count_min: 70
test_kinds: [unit, property, integration, contract, regression, mutation]
feature_gate: [api]
verb_class: refuse
cc_owns: [CC-6 producer]
cc_consumes: [CC-2 (m8/m9/m10), CC-4 via m30]
gap_owner: [none]
boilerplate_lift_pct: 25
status: SPEC
date: 2026-05-17
authority: Luke @ node 0.A
hold_v2_compliant: true
cardinality_amendment: "S1002127 — PrivilegeEscalation inserted at ordinal 30 (D-S1002127-02 ADR)"
---

# m33 — `m33_verifier` Rust spec

> Back to: [vault cluster-G spec](../../../the-workflow-engine-vault/module%20specs/cluster-G-bank-select-dispatch-verify.md) · [MODULE_MATRIX](../../MODULE_MATRIX.md) · [ARCHITECTURE](../../../ARCHITECTURE.md) · [V7 cluster-G plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-G.md) · [GENESIS v1.3 § 1.a](../../../ai_docs/GENESIS_PROMPT_V1_3.md) · vault [[cluster-G-bank-select-dispatch-verify]]
>
> Sister modules: [m30](m30_curated_bank.md) · [m31](m31_selector.md) · [m32](m32_conductor_dispatcher.md) · [m33](m33_verifier.md)

## 1. Purpose & invariants

`m33_verifier` IS the **4-agent dry-run gate** that produces the `VerificationResult` m32 consults at dispatch time. It is the OI-3 / Genesis v1.3 § 1.a additive module (was missing from v1.2; required by Town Hall P0 #9 — Command-3's FP-Verifier-Lead requirement). m33 runs the same shape of gate as `/pre-deploy-hardening` (security-auditor + performance-engineer + silent-failure-hunter + Zen, parallel; APPROVE / REJECT / APPROVE-WITH-NITS verdict composition) but applied to workflow definitions rather than staged diffs. Its output is consumed only by m32 (CC-6); m31 does NOT consult m33 directly (selection and verification are orthogonal — coupling would create non-determinism per vault § Intra-cluster notes).

The module MUST guarantee five invariants. **First — 7-day TTL is HARD**: `VerificationResult::ttl_expires_at = verified_at + 7d` is non-negotiable and stored explicitly (not computed at read time) so the gate value can survive clock-skew across services. `is_fresh(now_ms)` returns true iff `now_ms < ttl_expires_at`. m32's check 2 enforces freshness; stale verifies block dispatch. **Second — definition-hash binding**: every `VerificationResult` carries `definition_hash: String` (FNV-1a 64-bit hex of canonical `steps_json`). m32 recomputes the hash at dispatch time and compares; mismatch yields `DispatchError::DefinitionDrifted` and the verify must be re-run with the current definition. This is the AP-Drift-11 mitigation generalised: a supervisor stub mistaken for live cannot pass verification because the agents see the actual `steps_json` and the hash binds the verdict to that exact byte stream. **Third — verdict composition modulated by escape surface (7-variant; D-S1002127-02)**: per V7 cluster-G plan § m33 4-agent verify gate spec, the threshold for PASS rises with destructiveness. `Sandboxed | FileWrite` requires majority APPROVE (≥3 of 4); `SandboxEscape | ProcessMutate | PrivilegeEscalation | NetworkEgress` requires 3-of-4 + zero REJECT; `DataExfil` requires **unanimous APPROVE** (any REJECT or APPROVE-WITH-NITS → FAIL). `PrivilegeEscalation` is gated at the same composition as `NetworkEgress` (3-of-4 zero-REJECT) — capability gain is high-stakes but bounded; not destructive-by-default. This makes Gap 3 schema load-bearing at the verification gate, not just at display. **Fourth — audit-first guarantee**: on PASS or DEGRADED, m33 calls `BankDb::record_verification(id, now_ms)` *before* returning the result to the caller; on FAIL, no timestamp is written. The write is durable before the caller can act on the verdict — this is the `conductor_enforcement.rs` pattern adapted. **Fifth — 4-agent live-verify mandatory (AP-Drift-11)**: m33 may not return a synthesised verdict from cache or a heuristic; every verify run must engage the 4 agents (in tests, via fixture sub-agent stubs; in production, via the configured agent-dispatch mechanism). Skipping agents on "cache hit" is forbidden; the TTL handles freshness; cache-bypass is a verify-sync break.

Frame violations m33 must structurally refuse: (a) any path that emits a `Pass` verdict without 4 agent verdicts having been collected — guarded by `AgentVerdict::Vec` length check at composition; (b) any path that records a TTL timestamp without an actual PASS/DEGRADED outcome (AP-V7-13 diagnostics-theatre cousin); (c) any cache-based verdict short-circuit that bypasses live agent dispatch (AP-Drift-11); (d) any human-meaningful label in agent evidence — file:line references are fine; "this looks bad" without a structural finding is rejected at evidence parse time; (e) re-using a `VerificationResult` from a different workflow's `definition_hash` (caller bug; guarded by `workflow_id == result.workflow_id` assertion in m32's check 3 prelude).

## 2. Public surface (Rust types — spec only, NOT compileable)

```rust
//! # m33_verifier
//!
//! - **Layer**: L7 (Bank/Select/Dispatch/Verify, Cluster G)
//! - **Deps**: workflow_core::{types::{WorkflowId, StepDef}, errors::VerifierError}, m30::{BankDb, AcceptedWorkflow, EscapeSurfaceProfile}, async agent-dispatch transport (Task-tool-equivalent in tests; production transport TBD at Zen G7)
//! - **Tests**: 70 (35 unit + 5 property + 0 fuzz + 18 integration + 5 contract + 7 regression + mutation ≥80%)
//! - **Features**: api
//! - **Platform**: Linux; SQLite cache for VerificationResult (shared `db.sqlite` per Axis 2); async agent fan-out with `tokio::join!` and per-agent timeout
//! - **Impl Notes**: Lifts ~80% of /pre-deploy-hardening 4-agent gate + ~80% of conductor_enforcement.rs audit-first; TTL + Cluster G integration authored fresh
//! - **Related Docs**: [vault cluster-G spec](../../../the-workflow-engine-vault/module%20specs/cluster-G-bank-select-dispatch-verify.md) · [V7 cluster-G](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-G.md) · [GENESIS v1.3 § 1.a](../../../ai_docs/GENESIS_PROMPT_V1_3.md)

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VerificationVerdict { Pass, Fail, Degraded }

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationAgent { Security, Performance, SilentFailure, Zen }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentVerdict {
    pub agent: VerificationAgent,
    pub approved: bool,
    pub nits_only: bool,           // true only if approved && warnings present
    pub evidence: Vec<String>,     // file:line refs or structured findings
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VerificationResult {
    pub workflow_id: WorkflowId,
    pub verified_at: i64,
    pub ttl_expires_at: i64,       // verified_at + 7d (stored, not computed)
    pub verdict: VerificationVerdict,
    pub agent_verdicts: Vec<AgentVerdict>, // exactly 4
    pub definition_hash: String,   // FNV-1a 64-bit hex of canonical steps_json
}

#[derive(Debug, Clone)]
pub struct VerifierConfig {
    pub ttl_ms: i64,                       // default 7 * 24 * 3600 * 1000
    pub agent_timeout_ms: u64,             // default 300_000 (5 min)
    pub cache_db_path: std::path::PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum VerifierError {
    #[error("verifier: workflow not in bank: {0}")]
    UnknownWorkflow(WorkflowId),
    #[error("verifier: agent dispatch failed: {agent:?}: {reason}")]
    AgentDispatch { agent: VerificationAgent, reason: String },
    #[error("verifier: agent timeout: {agent:?} ({timeout_ms}ms)")]
    AgentTimeout { agent: VerificationAgent, timeout_ms: u64 },
    #[error("verifier: bank read: {0}")]
    Bank(#[from] crate::m30::BankError),
    #[error("verifier: sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("verifier: serde: {0}")]
    Serde(#[from] serde_json::Error),
}

#[async_trait::async_trait]
pub trait AgentDispatch {
    async fn invoke(
        &self,
        agent: VerificationAgent,
        wf: &AcceptedWorkflow,
    ) -> Result<AgentVerdict, VerifierError>;
}

pub struct Verifier<D: AgentDispatch> { /* private */ }

impl<D: AgentDispatch> Verifier<D> {
    pub fn new(cfg: VerifierConfig, dispatcher: D) -> Result<Self, VerifierError>;

    /// Run the 4-agent dry-run gate. AP-Drift-11: NEVER short-circuits from cache.
    pub async fn verify(
        &self,
        bank: &BankDb,
        workflow_id: &WorkflowId,
        now_ms: i64,
    ) -> Result<VerificationResult, VerifierError>;

    pub fn is_fresh(&self, result: &VerificationResult, now_ms: i64) -> bool;
    pub fn cached(&self, workflow_id: &WorkflowId) -> Result<Option<VerificationResult>, VerifierError>;
}
```

## 3. Internal data structures

`Verifier<D>` holds the `VerifierConfig`, the agent dispatcher `D` (parameterised so tests inject a `FixtureDispatch` and production injects the real `Task`-style sub-agent dispatcher), and a `VerifierCacheDb` (rusqlite wrapper on the shared `db.sqlite` with table `verification_results { workflow_id, verified_at, ttl_expires_at, verdict, agent_verdicts_json, definition_hash }`). A private `MechanicalGate` runs the Wave-1 pre-checks (StepDef registry resolution, declared-vs-derived `EscapeSurfaceProfile` consistency, `sunset_at > now`); only on mechanical PASS does the 4-agent wave fire. The `VerdictComposer` is a pure function `(escape_surface, [AgentVerdict; 4]) -> VerificationVerdict` so the escape-surface-modulation table is unit-testable in isolation.

## 4. Data flow

- **INPUT FROM:** `m30::BankDb::get(id)` (workflow definition + escape surface + steps); 4 sub-agent invocations via `AgentDispatch::invoke()`; system clock via caller-provided `now_ms`.
- **OUTPUT TO:** caller (m32's pre-dispatch sequence indirectly; explicit `wf-dispatch verify <id>` CLI subcommand directly); `m30::BankDb::record_verification(id, now_ms)` on PASS/DEGRADED; `verification_results` cache table for staleness reads.
- **SUBSTRATE TOUCHED:** shared `db.sqlite` (cache table); 4 sub-agent transports (in-process channel in tests; configured transport in production).
- **WRITES:** `verification_results` row (audit-first; written before return on PASS/DEGRADED); `record_verification` on m30 (timestamp only on PASS/DEGRADED).

## 5. Algorithm sketch

```text
verify(bank, workflow_id, now_ms):
    let wf = bank.get(workflow_id)?.ok_or(UnknownWorkflow)?

    // WAVE 1 — MECHANICAL GATE (pre-checks)
    MechanicalGate::run(&wf)?
        - every StepDef.kind resolves in workflow_core::step_registry
        - declared escape_surface_profile >= derived (StepClassifier::derive)
        - sunset_at > now_ms

    // WAVE 2 — 4-AGENT PARALLEL DISPATCH (AP-Drift-11: ALWAYS live, never cached)
    let (s, p, sf, z) = tokio::join!(
        with_timeout(cfg.agent_timeout_ms, dispatcher.invoke(Security,      &wf)),
        with_timeout(cfg.agent_timeout_ms, dispatcher.invoke(Performance,   &wf)),
        with_timeout(cfg.agent_timeout_ms, dispatcher.invoke(SilentFailure, &wf)),
        with_timeout(cfg.agent_timeout_ms, dispatcher.invoke(Zen,           &wf)),
    )
    let agent_verdicts = [s?, p?, sf?, z?]    // any agent timeout/error short-circuits

    // WAVE 3 — VERDICT COMPOSITION (escape-surface modulated; 7-variant per D-S1002127-02)
    let verdict = VerdictComposer::compose(wf.escape_surface_profile, &agent_verdicts)
        match wf.escape_surface_profile {
            Sandboxed | FileWrite ->
                if approved_count >= 3 then if any_rejected then Fail else Pass
                else Fail
            SandboxEscape | ProcessMutate | PrivilegeEscalation | NetworkEgress ->
                // 3-of-4 zero-REJECT — capability-gain (PrivilegeEscalation)
                // gated at same threshold as NetworkEgress per D-S1002127-02
                if fail_count == 0 && approved_count >= 3 then
                    if any_nits then Degraded else Pass
                else Fail
            DataExfil ->
                if all_pass_no_nits then Pass else Fail  // UNANIMOUS
        }

    let definition_hash = fnv1a_64_hex(serde_json::to_vec(&wf.steps)?)
    let result = VerificationResult {
        workflow_id: wf.id.clone(),
        verified_at: now_ms,
        ttl_expires_at: now_ms + cfg.ttl_ms,
        verdict,
        agent_verdicts: agent_verdicts.to_vec(),
        definition_hash,
    }

    // AUDIT-FIRST: write cache row, then record on m30 (PASS/DEGRADED only)
    self.cache.upsert(&result)?
    if verdict != Fail:
        bank.record_verification(&wf.id, now_ms)?
    Ok(result)
```

## 6. Boilerplate lifts

Per vault cluster-G spec § m33 Boilerplate lift and V7 cluster-G Category 09/03:

| Source | Lift | % |
|---|---|---:|
| `SKILL-pre-deploy-hardening.md` 4-agent gate | parallel dispatch + `Verdict { agent, APPROVE/REJECT, evidence }` shape + mechanical-gate + agent-wave structure | 80% (95% semantic) |
| `conductor_enforcement.rs::EnforcerAction` enum | adapted as `VerificationVerdict { Pass, Fail, Degraded }`; audit-first writes; cooldown logic adapted as TTL gating | 80% |
| `memory-injection/m06_schema.rs` SQLite | cache-table scaffold | 90% (subset) |
| `m24_povm_bridge` | wire-shape reference for any HTTP-based agent transport | 30% |
| **TTL enforcement + escape-surface-modulated composition + Cluster G integration** | — | **0% (novel ~50 LOC)** |

Net: ~150 LOC lifted / ~50 LOC novel.

## 7. ME v2 m1_foundation patterns referenced

- `error.rs` — `VerifierError` thiserror enum with structured named-field variants (`{ agent, reason }`, `{ agent, timeout_ms }`); pattern-matchable.
- `resources.rs` — `//!` docstring block.
- `state.rs` — `Verifier<D>` central-state pattern; verdict composition is a pure function (testable in isolation).
- `shared_types.rs` — `WorkflowId` newtype; `VerificationAgent` enum tagged for serde stability.
- `config.rs` — `VerifierConfig::Default` + env-override; `agent_timeout_ms` overridable for CI fast-fail.
- `logging.rs` — tracing structured emit per agent invocation (agent name, elapsed_ms, approved flag) and per `verify()` completion (verdict, definition_hash, ttl_expires_at).
- `metrics.rs` — counter `verify_total{verdict}`, histogram `agent_verify_ms{agent}`, gauge `verify_cache_size`.

## 8. Test strategy

- **Test kind**: unit (35) + property (5) + integration (18) + contract (5) + regression (7)
- **Test count**: 70 minimum (per [TEST_DISCIPLINE matrix](../../../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md) row m33; cluster G total 290)
- **Mutation budget**: ≥80% kill on `ttl.rs` + `agents/mod.rs` (composition) + `definition_hash.rs` (G6 m33 threshold)
- **Properties tested** (F-Property 5):
  - TTL monotonic: `ttl_expires_at == verified_at + cfg.ttl_ms` exactly; `is_fresh(now)` is true iff `now < ttl_expires_at`.
  - 4-agent verdict composition is total: every `(escape_surface, [4 verdicts])` input has exactly one verdict output (no panic, no `Option::None`).
  - Composition modulation: `DataExfil` with 3-APPROVE + 1-REJECT is always Fail; `Sandboxed` with the same input is also Fail (REJECT short-circuits both bands at majority).
  - Audit-first: on PASS/DEGRADED, `cache.upsert` happens before `bank.record_verification` in trace order.
  - FNV determinism: same `steps_json` produces same `definition_hash` across 1000 invocations.

Key invariants (sample of 70; full enumeration in vault cluster-G spec § Tests m33):

1. `VerificationVerdict` round-trips through serde with SCREAMING_SNAKE_CASE names (`PASS`, `FAIL`, `DEGRADED`).
2. `VerificationAgent` round-trips through serde with snake_case names.
3. `is_fresh()` returns `false` for `now_ms == ttl_expires_at` (strict `<`).
4. `is_fresh()` returns `false` for `now_ms > ttl_expires_at`.
5. `is_fresh()` returns `true` for `now_ms == ttl_expires_at - 1`.
6. `Sandboxed` + 3 APPROVE + 1 APPROVE = PASS.
7. `Sandboxed` + 3 APPROVE + 1 REJECT = FAIL (any REJECT fails majority band per V7 plan).
8. `Sandboxed` + 4 APPROVE-WITH-NITS = DEGRADED.
9. `NetworkEgress` + 4 APPROVE = PASS.
10. `NetworkEgress` + 3 APPROVE + 1 APPROVE-WITH-NITS = DEGRADED.
11. `NetworkEgress` + 3 APPROVE + 1 REJECT = FAIL (zero-REJECT requirement).
11a. `PrivilegeEscalation` + 4 APPROVE = PASS (composed as NetworkEgress band — 3-of-4 zero-REJECT per D-S1002127-02).
11b. `PrivilegeEscalation` + 3 APPROVE + 1 REJECT = FAIL.
11c. `PrivilegeEscalation` + 3 APPROVE + 1 APPROVE-WITH-NITS = DEGRADED.
12. `DataExfil` + 4 APPROVE = PASS.
13. `DataExfil` + 3 APPROVE + 1 APPROVE-WITH-NITS = FAIL (unanimous-no-nits required).
14. `DataExfil` + any REJECT = FAIL.
15. `definition_hash` is deterministic across 1000 runs on same `steps_json`.
16. `definition_hash` differs when one byte of `steps_json` changes (collision-resistance basics).
17. `verify()` of unknown workflow_id returns `UnknownWorkflow`.
18. Mechanical gate failure (unknown step type) returns from `verify()` before 4-agent wave fires.
19. Mechanical gate failure: escape_surface inconsistency rejected.
20. Mechanical gate failure: past sunset rejected.
21. Agent timeout returns `AgentTimeout { agent, timeout_ms }`.
22. Agent dispatch error returns `AgentDispatch { agent, reason }`.
23. On PASS, `bank.record_verification(id, now_ms)` is called.
24. On FAIL, `bank.record_verification` is NOT called (no diagnostics-theatre stamp).
25. On DEGRADED, `bank.record_verification` IS called (DEGRADED is permitted-with-ack).
26. Cache `upsert` happens before `bank.record_verification` (audit-first).
27. AP-Drift-11: `verify()` always invokes 4 agents (test asserts dispatcher call count == 4 even when cache hot).

(remaining 43 — including F-Contract agent-verdict-evidence-format snapshot, F-Integration full CC-6 closure with m32, F-Regression slots for cache-bypass-attempt, escape-surface-modulation-drift, ttl-zero-edge-case — enumerated in vault spec.)

## 9. Antipatterns to avoid

- **AP-Drift-11** (supervisor stub mistaken for live) — 4-agent live-verify mandatory; cache cannot short-circuit a verify; FP-verify discipline; tests assert all 4 agents invoked per `verify()` call.
- **AP-V7-13** (diagnostics theatre) — `record_verification` only on PASS/DEGRADED; FAIL leaves no stamp; refresh dates pair with re-executed agent probes.
- **AP-WT-F4** (premature dispatch) — TTL hard-gates dispatch via m32 check 2; stale verifies REFUSE the workflow.
- **AP-V7-08** (self-dispatch) — m33 refuses to verify a workflow whose steps target m32 itself (mechanical-gate rejection; defense-in-depth alongside m30 + m32 refusals).
- **AP-Hab-04** (preserve-list discipline) — Destructive escape-surface requires UNANIMOUS PASS; gating is not "more lenient at high stakes" but stricter.
- **AP30** (namespace drift) — agent invocation prompts reference `workflow_core::namespace` constants for any substrate identifier; no literal `workflow_trace_*` strings.
- **AP29** (sync HTTP in tokio::spawn) — agent dispatch via async transport; no `block_on` in spawned tasks.
- **Gap 3 modulation tampering** — composition table is `const`-driven and unit-tested per escape surface; mutation testing catches threshold inversions.
- **Newly surfaced**: `agent_verdicts.len() != 4` slipping through composition — composer asserts exactly 4 verdicts before computing (regression slot reserved).

## 10. Useful patterns applied

- /pre-deploy-hardening 4-agent gate (PATTERNS.md § Module-level patterns).
- thiserror error enums (GOLD_STANDARDS rule 9).
- Newtype + enum discipline (GOLD_STANDARDS rule 8).
- `//!` docstring block (GOLD_STANDARDS rule 13).
- Audit-first writes (`conductor_enforcement.rs`) — cache row before `record_verification`.
- Pure-function verdict composer (functional core / imperative shell).
- Generic `AgentDispatch` trait — test fixture injection; production transport at Zen G7.
- `tokio::join!` for parallel agent fan-out with per-agent timeout (no agent can block another).

## 11. Cross-cluster contracts

- **CC-6 producer (G internal: m33 → m32)**: m33 is the SOLE producer of `VerificationResult`; m32 is the SOLE consumer. m33 does NOT push to m32; m32 pulls (or the CLI `wf-dispatch verify` subcommand triggers `verify()` directly). The contract: `definition_hash` binds the verdict to the exact `steps_json`; `ttl_expires_at` binds it to a freshness window; `workflow_id` binds it to a specific bank entry. m32 must check all three.
- **CC-4 indirect (via m30)**: m33 reads from m30 (`bank.get`) and writes back (`bank.record_verification`); the contract surface is `BankDb`'s `get` + `record_verification` methods.
- **CC-2 trust layer**: m8 (POVM-calibrated build gate), m9 (namespace assertion at any namespace-bearing write), m10 (Ember CI gate verifies that test fixtures cannot accidentally produce synthetic PASS verdicts).
- **Gap 3 consumer contract**: m33 reads `EscapeSurfaceProfile` from m30 and uses ordinal comparison + variant matching for verdict-composition modulation. The composition table is the *verification gate's load-bearing use* of the Gap 3 schema; changes to the enum require synchronous re-audit of the composition table.

## 12. Open questions for G5 interview / Zen G7 audit

1. **Agent-dispatch transport in production**: spec leaves `AgentDispatch` trait as the integration seam; vault spec says "in single-phase, these are sub-agent dispatches via the Task tool or equivalent; the spec leaves agent-dispatch mechanism open to the build-time implementation choice, subject to Zen audit at G7." Should v1.3 lock the transport (in-process channel? HTTP to a `verify-agent` daemon? CLI subprocess invocation per agent?) or defer to G7?
2. **TTL = 7 days fixed vs escape-surface scaled**: should `DataExfil` workflows have a shorter TTL (24h) so re-verification is more frequent for high-stakes flows? Tradeoff: TTL-scaling adds a second escape-surface-aware table to the spec.
3. **Composition table for DEGRADED on `DataExfil`**: current spec says any non-PASS → FAIL for `DataExfil`. Should DEGRADED be permissible at `DataExfil` with a *secondary* operator gate (interactive Y/N prompt before dispatch) rather than outright FAIL? Risks: operator-fatigue auto-Y.
4. **EscapeSurfaceProfile ordinal stability** (shared with m30 + m32 Open Qs) — **CLOSED by D-S1002127-02 (2026-05-17).** Cardinality bumped 6→7; PrivilegeEscalation inserted at ordinal 30. m33's composition table now matches on 7 variants: `Sandboxed|FileWrite` (majority APPROVE), `SandboxEscape|ProcessMutate|PrivilegeEscalation|NetworkEgress` (3-of-4 zero-REJECT), `DataExfil` (UNANIMOUS-no-nits). Verify-sync invariant: any new `EscapeSurfaceProfile` variant must extend the composition table; clippy non-exhaustive match catches drift.
5. **Agent timeout 5 min default**: too long for `Sandboxed` (where Zen review is quick), too short for `DataExfil` (where security-auditor needs to walk the whole trap surface)? Consider escape-surface-scaled timeouts.
6. **`AgentVerdict::evidence` schema**: free-form `Vec<String>` (current) vs structured `Vec<Finding { file, line, severity, message }>` — the structured form helps machine-readable audit but couples m33 to a finding-schema that the 4 agents might disagree on. Recommend Zen G7 ruling.

## 13. Implementation order (post-G9)

1. `error.rs` — `VerifierError` enum (`thiserror`); compile-only.
2. `definition_hash.rs` — FNV-1a 64-bit hex of canonical `steps_json`; 5 unit tests (determinism, byte-flip detection).
3. `ttl.rs` — `is_fresh(now_ms)` + `compute_expires_at(verified_at, ttl_ms)`; 8 unit tests (strict-`<` boundary, overflow guard).
4. `agents/mod.rs` — `AgentDispatch` trait + verdict composition function `compose(escape_surface, [AgentVerdict; 4]) -> VerificationVerdict`; 12 unit tests covering every escape-surface × verdict-shape combination.
5. `agents/security.rs`, `agents/performance.rs`, `agents/silent_failure.rs`, `agents/zen.rs` — agent-invocation wrappers (in-process channel in tests; production transport decided at G7).
6. `persistence.rs` — `VerifierCacheDb` SQLite scaffold; `upsert` + `get` + `list`; 6 unit tests.
7. `mechanical_gate.rs` — Wave-1 pre-check: step registry resolution + escape-surface consistency + sunset; 6 unit tests.
8. `mod.rs` — `Verifier::new` + `verify` orchestration (tokio::join! + timeout per agent + audit-first); 8 unit tests.
9. Property tests (5) — TTL monotonic, composition totality, modulation correctness, audit-first ordering, FNV determinism.
10. Integration tests (18) — `tests/m33_integration.rs` exercising full CC-6 closure with m32, m30→m33 round-trip, all-4-agents-invoked invariant per AP-Drift-11.
11. Contract tests (5) — insta snapshot for `VerificationResult` JSON shape; agent-evidence parse contract.
12. Regression slots (7) — cache-bypass-attempt, escape-surface-modulation-drift, ttl-zero-edge, 4-verdict-len-check, fail-stamps-record-verification, etc.
13. Mutation pass — `cargo mutants --regex 'm33_verifier::(ttl|agents|definition_hash)::.*'`; ≥80% kill required.

---

> Back to: [vault cluster-G spec](../../../the-workflow-engine-vault/module%20specs/cluster-G-bank-select-dispatch-verify.md) · [MODULE_MATRIX](../../MODULE_MATRIX.md) · [ARCHITECTURE](../../../ARCHITECTURE.md) · sister modules: [m30](m30_curated_bank.md) · [m31](m31_selector.md) · [m32](m32_conductor_dispatcher.md) · [m33](m33_verifier.md)
