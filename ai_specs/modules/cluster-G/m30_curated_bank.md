---
title: m30 — `m30_curated_bank` Rust spec
cluster: G — Bank/Select/Dispatch/Verify
layer: L7
binary: wf-dispatch
loc_estimate: ~220
test_count_min: 70
test_kinds: [unit, property, integration, contract, regression, mutation]
feature_gate: [api]
verb_class: record+select
cc_owns: [CC-4 entry-point, CC-5 read-side]
cc_consumes: [CC-2 (m8/m9/m10/m11)]
gap_owner: [Gap 3]
boilerplate_lift_pct: 30
status: SPEC
date: 2026-05-17
authority: Luke @ node 0.A
hold_v2_compliant: true
cardinality_amendment: "S1002127 — PrivilegeEscalation inserted at ordinal 30 (D-S1002127-02 ADR)"
---

# m30 — `m30_curated_bank` Rust spec

> Back to: [vault cluster-G spec](../../../the-workflow-engine-vault/module%20specs/cluster-G-bank-select-dispatch-verify.md) · [MODULE_MATRIX](../../MODULE_MATRIX.md) · [ARCHITECTURE](../../../ARCHITECTURE.md) · [V7 cluster-G plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-G.md) · [GENESIS v1.3](../../../ai_docs/GENESIS_PROMPT_V1_3.md) · vault [[cluster-G-bank-select-dispatch-verify]]
>
> Sister modules: [m30](m30_curated_bank.md) · [m31](m31_selector.md) · [m32](m32_conductor_dispatcher.md) · [m33](m33_verifier.md)

## 1. Purpose & invariants

`m30_curated_bank` IS the **curated persistent registry** of workflow definitions that have passed human curator review. It is the single source of truth for the question *"what workflows may be dispatched at all?"* — m31 (selector) reads from it, m32 (dispatcher) resolves through it, and m33 (verifier) records freshness back into it. m30 owns the **Gap 3 EscapeSurfaceProfile schema** — the unified destructiveness classification that replaces five scattered classifiers across `SKILL-forge.md`, `SKILL-genesis.md`, `SKILL-pre-deploy-hardening.md`, `SKILL-silent-swallow-detect.md`, and `hookify.preserve-blanket-guard.local.md` (S102 openclaw scar tissue). The schema is shared at the wire-contract layer with m9 (namespace-guard) and consumed at display time by m32 (banner half).

The module MUST guarantee four invariants. **First — F5 admission discipline (AP-V7-07 hard-refusal):** workflows enter the bank ONLY through the explicit, human-mediated `wf-dispatch bank accept` subcommand. There is no `auto_promote()` function; there is no agent-callable insert path; there is no "if confidence > threshold, insert" branch. `BankDb::accept()` requires an `accepted_by: HumanAcceptanceSignature` field on every call. m23 (Cluster F proposer) produces `WorkflowProposal` artefacts; humans review them; m30 admits the survivors. Bypassing this is an `AP-Hab-class` refusal-mode violation. **Second — sunset is immutable.** Every `AcceptedWorkflow` carries `sunset_at` set at accept time (default `accepted_at + 120 days`, overridable by curator). Sunset cannot be extended in-place; a workflow past sunset is structurally excluded from `eligible()` at the SQL layer (`WHERE sunset_at > ?`). To re-bank, a new entry must be admitted with a new id. **Third — definition hash is monotonic.** Once admitted, `steps_json` is immutable; any edit creates a new entry with a new id. m32 hashes the resolved steps at dispatch time and compares against the m33-stored `definition_hash`; mismatch yields `DispatchError::DefinitionDrifted`. **Fourth — Gap 3 ordinal stability (cardinality 7 per D-S1002127-02).** `EscapeSurfaceProfile` is an `Ord`-bearing enum with stable serde names; the ordering `Sandboxed(0) < SandboxEscape(10) < ProcessMutate(20) < PrivilegeEscalation(30) < FileWrite(40) < NetworkEgress(50) < DataExfil(60)` is contract-binding and consumed by m32's banner display, m9's namespace gate, and m33's verdict-composition modulation. Numeric ordinals use steps of 10 to reserve gaps for future inserts. Reordering or renaming variants is a contract break.

Frame violations m30 must structurally refuse: (a) any path that admits a workflow without an explicit human signature (F5); (b) any mutation of `steps_json` after admission (definition-hash invariance); (c) any path that downgrades an `EscapeSurfaceProfile` after acceptance (a workflow classified `Destructive` cannot be silently reclassified `HostWrite` to slip past a stricter m33 verdict modulation); (d) any namespace string literal in m30's writes — namespace constants live in `workflow_core::namespace` per AP30. m30 is `cfg(povm_calibrated)`-gated at the crate root by m8 (CC-2 trust); m9 wraps any namespace-bearing operation; m10 Ember CI gate verifies acceptance flows in test.

## 2. Public surface (Rust types — spec only, NOT compileable)

```rust
//! # m30_curated_bank
//!
//! - **Layer**: L7 (Bank/Select/Dispatch/Verify, Cluster G)
//! - **Deps**: rusqlite (WAL), workflow_core::{types::{WorkflowId, LineageId, StepDef, StepId}, errors::BankError, namespace}
//! - **Tests**: 70 (35 unit + 5 property + 0 fuzz + 15 integration + 5 contract + 9 regression + mutation ≥75%)
//! - **Features**: api
//! - **Platform**: Linux; SQLite WAL at `~/.local/share/workflow-trace/db.sqlite` (single DB per Axis 2)
//! - **Impl Notes**: Lifts ~90% of m06_schema's `configure_connection` for WAL pragmas; ~70% of conductor_state.rs StateDb constructor; EscapeSurfaceProfile + classifier authored fresh (~80 LOC — Gap 3)
//! - **Related Docs**: [vault cluster-G spec](../../../the-workflow-engine-vault/module%20specs/cluster-G-bank-select-dispatch-verify.md) · [V7 cluster-G](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-G.md) · [GENESIS v1.3 § 2](../../../ai_docs/GENESIS_PROMPT_V1_3.md)

/// The Gap 3 unified destructiveness schema — the ordinal escape-surface profile.
///
/// Replaces 5 scattered classifiers (SKILL-forge / SKILL-genesis /
/// SKILL-pre-deploy-hardening / SKILL-silent-swallow-detect /
/// hookify.preserve-blanket-guard). S102 openclaw scar tissue.
///
/// Cardinality 7 per Luke S1002127 directive (D-S1002127-02 ADR);
/// PrivilegeEscalation inserted at ordinal 30 between ProcessMutate and FileWrite.
///
/// Ordering is **contract-binding**: Sandboxed(0) < SandboxEscape(10) <
/// ProcessMutate(20) < PrivilegeEscalation(30) < FileWrite(40) <
/// NetworkEgress(50) < DataExfil(60). Numeric ordinals use steps of 10 to
/// reserve gaps for future inserts (closes G7 ordinal-stability concern).
/// m32 uses `>=` comparisons to gate banner severity. m9 uses the same
/// ordering for namespace-write capability checks. m33 uses variant match
/// for verdict-composition modulation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum EscapeSurfaceProfile {
    /// Sandboxed read-only: cat, ls, grep, curl HEAD against allowlist
    Sandboxed = 0,
    /// Sandbox-escape risk: setsid, nohup, docker exec into non-this-container
    SandboxEscape = 10,
    /// Process mutation WITHIN current privilege envelope: kill, pkill,
    /// signal-send, devenv start/stop (no capability gain)
    ProcessMutate = 20,
    /// Capability gain or role elevation that grants the calling process new
    /// abilities beyond its pre-call state. Examples: invoking `sudo`;
    /// setuid/setgid; capability acquisition (`cap_set_proc`, `setcap`);
    /// ACL add; container privilege escalation (Docker `--privileged`,
    /// `cap-add`); SELinux/AppArmor profile escape. Distinguished from
    /// `ProcessMutate` (modifies another process WITHIN current privilege
    /// envelope) and `FileWrite` (requires existing write permission but
    /// does NOT acquire new capabilities). Habitat-relevant: openclaw
    /// container UID-1337 escape, sudo gates, role elevations in
    /// nerve-center / Conductor. Inserted at ordinal 30 per D-S1002127-02.
    PrivilegeEscalation = 30,
    /// Host filesystem write under EXISTING permission: edit, write, mkdir,
    /// atomic-rename in user-owned paths (no capability gain)
    FileWrite = 40,
    /// Network egress: HTTP POST/PUT, ssh, git push to non-allowlisted remote
    NetworkEgress = 50,
    /// Data exfil / destructive: rm -rf, drop database, force-push, prune
    DataExfil = 60,
}

impl EscapeSurfaceProfile {
    /// Stable banner line consumed by m32 display-before-step (Gap 3 display half).
    /// Format is contract-binding; m32 contract tests assert this exact text.
    #[must_use]
    pub fn banner_line(&self) -> &'static str { /* … */ }
}

/// A workflow that has passed human curator review and lives in the bank.
/// Ids are opaque (UUID v7) — they carry no human-readable meaning per F11.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AcceptedWorkflow {
    pub id: WorkflowId,
    pub lineage: LineageId,
    pub accepted_at: i64,            // unix ms
    pub sunset_at: i64,              // immutable; default accepted_at + 120d
    pub ralph_decay_weight: f64,     // [0.0, 1.0]; m11 owns mutation
    pub ember_state: Option<EmberGateResult>,
    pub escape_surface_profile: EscapeSurfaceProfile,
    pub steps: Vec<StepDef>,         // immutable after acceptance
    pub curator_note: String,
    pub last_verified_at: Option<i64>,
    pub dispatch_count: u64,
}

/// Human-signed acceptance signature. Constructed only by the wf-dispatch
/// CLI's interactive `bank accept` subcommand. No agent-callable constructor.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HumanAcceptanceSignature {
    pub accepted_by: String,           // operator username
    pub accepted_at: i64,
    pub interactive_terminal: bool,    // must be true; CLI asserts isatty()
}

#[derive(Debug, thiserror::Error)]
pub enum BankError {
    #[error("bank: workflow {0} already exists")]
    AlreadyExists(WorkflowId),
    #[error("bank: acceptance requires interactive human signature (AP-V7-07)")]
    AutoPromoteRefused,
    #[error("bank: sunset_at ({sunset_at}) must be > accepted_at ({accepted_at})")]
    SunsetInvalid { accepted_at: i64, sunset_at: i64 },
    #[error("bank: escape_surface inconsistent with steps (decl={declared:?}, derived={derived:?})")]
    EscapeSurfaceInconsistent { declared: EscapeSurfaceProfile, derived: EscapeSurfaceProfile },
    #[error("bank: namespace assertion failed: {0}")]
    Namespace(String),
    #[error("bank: sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("bank: serde: {0}")]
    Serde(#[from] serde_json::Error),
}

pub struct BankDb { /* private */ }

impl BankDb {
    pub fn open(path: &std::path::Path) -> Result<Self, BankError>;

    /// Admit a curator-approved workflow. CALLED ONLY from the CLI's
    /// `bank accept` subcommand (asserts isatty + human signature).
    /// AP-V7-07: no auto-promote path exists.
    pub fn accept(
        &self,
        wf: &AcceptedWorkflow,
        sig: &HumanAcceptanceSignature,
    ) -> Result<(), BankError>;

    pub fn get(&self, id: &WorkflowId) -> Result<Option<AcceptedWorkflow>, BankError>;
    pub fn eligible(&self, now_ms: i64, limit: usize) -> Result<Vec<AcceptedWorkflow>, BankError>;
    pub fn record_dispatch(&self, id: &WorkflowId, now_ms: i64) -> Result<(), BankError>;
    pub fn record_verification(&self, id: &WorkflowId, now_ms: i64) -> Result<(), BankError>;

    /// Called by m11 on engine sweep; applies decay to eligible rows and
    /// marks sunset entries. m30 NEVER initiates decay itself.
    pub fn apply_decay_tick(&self, now_ms: i64) -> Result<usize, BankError>;
}
```

## 3. Internal data structures

`BankDb` wraps a `rusqlite::Connection` configured with WAL + `busy_timeout = 5000` + `foreign_keys = ON` + `synchronous = NORMAL`, lifted from `m06_schema::configure_connection`. The `accepted_workflows` table uses `id TEXT PRIMARY KEY` (UUID v7), with `escape_surface_profile TEXT NOT NULL` carrying the serde-snake-case form (`sandboxed`, `sandbox_escape`, `process_mutate`, `privilege_escalation`, `file_write`, `network_egress`, `data_exfil` — cardinality 7 per D-S1002127-02). Two indexes serve hot reads: `idx_bank_weight_sunset(ralph_decay_weight DESC, sunset_at)` for m31 eligibility queries, and `idx_bank_sunset(sunset_at)` for m11's decay sweep. `CHECK (sunset_at > accepted_at)` enforces the sunset invariant at the SQL layer; `CHECK (ralph_decay_weight BETWEEN 0.0 AND 1.0)` clamps decay.

### Banner line table (Gap 3 display contract; cardinality 7)

| Variant | Ordinal | `banner_line()` |
|---|---:|---|
| `Sandboxed` | 0 | `SANDBOXED` |
| `SandboxEscape` | 10 | `SANDBOX-ESCAPE` |
| `ProcessMutate` | 20 | `PROCESS-MUTATE` |
| **`PrivilegeEscalation`** | **30** | **`PRIVILEGE-ESCALATION!`** |
| `FileWrite` | 40 | `FILE-WRITE` |
| `NetworkEgress` | 50 | `NETWORK-EGRESS!` |
| `DataExfil` | 60 | `DATA-EXFIL!` |

Banner strings are contract-binding; m32 contract tests assert this exact text. The bang-suffix marks the three highest-stakes variants (PrivilegeEscalation, NetworkEgress, DataExfil) that require additional operator acknowledgement.

A private `StepClassifier` resolves declared `EscapeSurfaceProfile` against the steps' computed escape surface (the maximum across `StepDef::declared_surface()` calls) and refuses admission on mismatch — this is the F5 mitigation's *consistency half*. The classifier reads from a static lookup table mapping step-kind tokens to escape surfaces, lifted ~95% from the unified skill-file classifier rationale.

## 4. Data flow

- **INPUT FROM:** `m23::WorkflowProposal` (via human review → CLI `bank accept`); `m11::DecayFactor` per workflow (via `apply_decay_tick()`); `m33::VerificationResult::verified_at` (via `record_verification()`); m32 dispatch outcomes (via `record_dispatch()`).
- **OUTPUT TO:** m31 (`eligible()` candidate pool); m32 (`get()` for resolution + `record_dispatch()` increment); m33 (`get()` for verification target).
- **SUBSTRATE TOUCHED:** `~/.local/share/workflow-trace/db.sqlite` (single DB per Axis 2 ORAC pattern; the bank shares the file with m7/m11/m14/m15/m33 tables but owns its own table set under the `bank_*` prefix in migrations).
- **WRITES:** acceptance rows (human-initiated only), dispatch-count increments (m32), verification timestamps (m33), decay-weight updates (m11). No agent-initiated insert path.

## 5. Algorithm sketch

```text
accept(wf, sig):
    require sig.interactive_terminal == true            else BankError::AutoPromoteRefused
    require sig.accepted_by != "" && sig.accepted_by != "agent" && sig.accepted_by != "auto"
    require wf.sunset_at > wf.accepted_at               else SunsetInvalid
    let derived = StepClassifier::derive(&wf.steps)
    require derived <= wf.escape_surface_profile        else EscapeSurfaceInconsistent
        (declared must be >= derived; never silently downgrade)
    m9::assert_namespace(wf.id)
    INSERT INTO accepted_workflows (...) VALUES (...) ON CONFLICT FAIL
    record audit row to dispatch_log via outbox (m32 reads on dispatch)

eligible(now_ms, limit):
    SELECT * FROM accepted_workflows
     WHERE sunset_at > now_ms
     ORDER BY ralph_decay_weight DESC, accepted_at DESC
     LIMIT limit

apply_decay_tick(now_ms):                                // called by m11 only
    UPDATE accepted_workflows SET ralph_decay_weight = MAX(0.0, ralph_decay_weight * 0.98)
                              WHERE sunset_at > now_ms
    return rows_affected
```

## 6. Boilerplate lifts

Per vault cluster-G spec § m30 Boilerplate lift and V7 cluster-G Category 03/09:

| Source | Lift | % |
|---|---|---:|
| `conductor_state.rs::StateDb` | WAL constructor, migration framework, `Severity` enum shape | 70% |
| `memory-injection/m06_schema.rs::configure_connection` | WAL/busy_timeout/synchronous/foreign_keys batch | 90% |
| `conductor_divergence.rs::Rule` trait | adapted as `StepClassifier` step-validator | 50% |
| `feedback_preserve_list_discipline.md` | EscapeSurfaceProfile rationale + ordinal ordering | 100% (semantic lift) |
| `SKILL-forge.md` / `SKILL-genesis.md` / `SKILL-pre-deploy-hardening.md` / `SKILL-silent-swallow-detect.md` | classifier intelligence unified into `EscapeSurfaceProfile` (5 scattered → 1 ordinal enum) | 95% (semantic) |
| **EscapeSurfaceProfile ordinal enum + classifier** | — | **0% (novel ~80 LOC — Gap 3 schema)** |

Net: ~140 LOC lifted / ~80 LOC novel (Gap 3 schema + classifier unification).

## 7. ME v2 m1_foundation patterns referenced

- `error.rs` — `BankError` is `thiserror::Error` with structured named-field variants (`{ accepted_at, sunset_at }`, `{ declared, derived }`); pattern-matchable, never `Box<dyn Error>`.
- `resources.rs` — `//!` docstring block (Layer / Deps / Tests / Features / Platform / Impl Notes / Related Docs).
- `state.rs` — `BankDb` central-state pattern; methods are state transitions; no shared mutable static.
- `shared_types.rs` — `WorkflowId(String)`, `LineageId(String)`, `HumanAcceptanceSignature` newtypes; never bare `String` for identity-bearing fields.
- `logging.rs` — tracing structured emit on every `accept()` (with `escape_surface_profile`, `accepted_by`), every `apply_decay_tick()` (row count), every `record_dispatch()`.
- `config.rs` — `BankConfig { db_path_override, decay_factor: f64 = 0.98, default_sunset_days: u64 = 120 }` follows `Default + env-override + override-field`.

## 8. Test strategy

- **Test kind**: unit (35) + property (5) + integration (15) + contract (5) + regression (9)
- **Test count**: 70 minimum (per [TEST_DISCIPLINE matrix](../../../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md) row m30; cluster G total 290)
- **Mutation budget**: ≥75% kill on `escape_surface/` + `admission.rs` (G6 m30 threshold)
- **Properties tested** (F-Property 5):
  - `EscapeSurfaceProfile` total-order respected: for any pair `(a, b)`, exactly one of `a < b`, `a == b`, `a > b` holds.
  - Sunset monotonic: for any admitted workflow, `sunset_at > accepted_at` invariant survives roundtrip through serde + SQLite.
  - Admission idempotent on same `WorkflowId`: second `accept()` with identical row returns `AlreadyExists`, never silently overwrites.
  - Decay clamped: after N applications of `apply_decay_tick`, `0.0 <= weight <= 1.0`.
  - `eligible()` returns no row with `sunset_at <= now_ms`.

Key invariants (sample of 70; full enumeration in vault cluster-G spec § Tests m30):

1. `accept()` with `interactive_terminal: false` returns `AutoPromoteRefused`.
2. `accept()` with `accepted_by = "agent"` returns `AutoPromoteRefused` (string blocklist).
3. `accept()` with `sunset_at <= accepted_at` returns `SunsetInvalid`.
4. `accept()` of declared `Sandboxed` with steps deriving `NetworkEgress` returns `EscapeSurfaceInconsistent`.
5. `accept()` of declared `DataExfil` with steps deriving `Sandboxed` succeeds (declared may over-state).
6. All 7 `EscapeSurfaceProfile` variants roundtrip through serde with stable snake_case names (`sandboxed`, `sandbox_escape`, `process_mutate`, `privilege_escalation`, `file_write`, `network_egress`, `data_exfil`).
7. `banner_line()` returns non-empty `&'static str` for every variant; `PrivilegeEscalation` returns `"PRIVILEGE-ESCALATION!"`.
8. `Ord` agrees with stated ordinal: `Sandboxed(0) < SandboxEscape(10) < ProcessMutate(20) < PrivilegeEscalation(30) < FileWrite(40) < NetworkEgress(50) < DataExfil(60)`.
9. `eligible(now_ms, limit)` excludes rows past sunset.
10. `eligible(now_ms, limit)` ordered by `ralph_decay_weight DESC`.
11. `eligible(now_ms, 0)` returns empty Vec.
12. `record_dispatch()` increments `dispatch_count` by 1.
13. `record_dispatch()` is idempotent on same ms (deterministic for replay).
14. `record_verification()` sets `last_verified_at` to provided ms.
15. `apply_decay_tick()` multiplies `ralph_decay_weight` by 0.98.
16. `apply_decay_tick()` never produces negative weight.
17. `apply_decay_tick()` ignores rows past sunset.
18. `get()` returns `None` for unknown id.
19. `get()` returns `None` for sunset row.
20. SQLite `CHECK (sunset_at > accepted_at)` rejects manual INSERT bypass.

(remaining 50 — including F-Contract banner-line snapshot tests, F-Regression slots for F5/F1/Gap-3 failure scenarios, and the m23→m30→m31→m32→m33 integration cycle — enumerated in vault spec.)

## 9. Antipatterns to avoid

- **AP-V7-07** (auto-promote m23 → m30) — hard refusal via `HumanAcceptanceSignature.interactive_terminal` + accepted_by blocklist; no code path bypasses this.
- **AP-V7-08** (self-dispatch) — m30 schema rejects any `steps` containing a step-kind that dispatches m32 itself; classifier returns `EscapeSurfaceProfile::SandboxEscape` minimum and admission refuses (defense in depth — m32 also refuses at dispatch time).
- **AP-WT-F1** (bank ossification) — `sunset_at` immutable at accept; m11 decay drives entries toward sunset; expired rows excluded at SQL layer.
- **AP-WT-F5** (bank creep) — admission requires interactive human signature; no agent-callable path.
- **AP-WT-F11** (cascade monoculture / label collapse) — `id` is opaque UUID v7, never a human-readable name; `lineage` is opaque m4 cluster id.
- **AP30** (namespace string drift) — all namespace writes via `workflow_core::namespace::WORKFLOW_TRACE_*` constants; literal strings forbidden in m30's SQL or serde paths.
- **AP-V7-13** (diagnostics theatre) — `record_verification()` writes a timestamp only when m33 actually returned PASS/DEGRADED; no "refreshed" stamps without underlying probe.
- **Newly surfaced**: silent escape-surface *downgrade* on re-acceptance — admission rejects any second `accept()` for same id, preventing reclassify-to-softer attacks (regression slot reserved).

## 10. Useful patterns applied

- ORAC single-DB-per-binary (PATTERNS.md § Architectural patterns row 1 + Axis 2 ORAC pattern).
- thiserror error enums (GOLD_STANDARDS rule 9).
- Newtype discipline (GOLD_STANDARDS rule 8) — `WorkflowId`, `LineageId`, `HumanAcceptanceSignature`.
- `//!` docstring block (GOLD_STANDARDS rule 13).
- Audit-first write (cribbed from `conductor_enforcement.rs`) — every `accept()` writes an audit log row before returning Ok.
- Ordinal-enum-as-capability-gate (Gap 3 schema) — `EscapeSurfaceProfile` is both a classifier and an `Ord` comparator that downstream gates use directly via `>=`.

## 11. Cross-cluster contracts

- **CC-4 entry-point (G owns the bank end of F → G → Conductor)**: m23's `WorkflowProposal` is reviewed by a human and then `BankDb::accept()` admits a derived `AcceptedWorkflow`. m30 is the *only* admission surface; m23 cannot insert directly.
- **CC-5 read-side (G → H → back to F via stcortex pathways)**: m30's `dispatch_count` and `last_verified_at` are read by m31 selector; the substrate-feedback loop updates `ralph_decay_weight` via m11's call to `apply_decay_tick()`, which closes the slow learning loop (days/weeks).
- **CC-6 read side (G internal: m33 → m32)**: m33 reads `AcceptedWorkflow` via `get()` for verification target; m32 reads via `get()` for resolution; both must observe the same row (one-DB consistency).
- **Cross-cluster contract with Cluster D m9 (Gap 3 schema sharing)**: m9's namespace_guard imports `EscapeSurfaceProfile` from m30 for write-capability checks. The schema lives in m30; m9 is a consumer. Any change in m30's enum shape is a verify-sync break in m9.

## 12. Open questions for G5 interview / Zen G7 audit

1. **EscapeSurfaceProfile ordinal stability across versions** — **CLOSED by D-S1002127-02 (2026-05-17).** Cardinality bumped 6→7 with PrivilegeEscalation inserted at ordinal 30; numeric gaps reserved (steps of 10) so future variants can be inserted without perturbing existing ordinals. Existing 6 variants retain their ordinal positions. Insertion of any further variant requires reserved-gap allocation and synchronised re-audit of m9 + m30 + m32 + m33 composition tables; clippy non-exhaustive match catches drift at the variant-match layer.
2. **`accepted_by` allowlist vs blocklist**: current spec uses a blocklist (`"agent"`, `"auto"`); should it be an allowlist of registered curator usernames seeded from a config file? Tradeoff = ergonomic onboarding vs harder spoofing.
3. **Sunset extension**: explicitly forbidden — but should there be a "re-bank with carry-forward" CLI path that creates a new id while preserving lineage + verification freshness for ergonomic re-acceptance? Risks: F1 ossification by stealth.
4. **Step-classifier source of truth**: lookup table inline (current spec) vs config-driven TOML loaded at startup? Inline is auditable in source; TOML is operator-tweakable. The audit-cost weighting argues inline.
5. **Decay-factor 0.98 per cycle**: matches m11 default; reconcilable with the m11 spec? Verify-sync invariant — m11 + m30 must agree on factor or the bank drifts.

## 13. Implementation order (post-G9)

1. `error.rs` — `BankError` enum (`thiserror`); compile-only.
2. `escape_surface/mod.rs` — `EscapeSurfaceProfile` ordinal enum + `Display` + `Ord` derivation + `banner_line()` + 12 unit tests (variant roundtrip + ordering + banner stability).
3. `escape_surface/lookup.rs` — step-kind → escape-surface lookup table (lifted from unified skill-file classifier).
4. `escape_surface/classifier.rs` — `StepClassifier::derive(steps) -> EscapeSurfaceProfile`; 8 unit tests.
5. `definition_hash.rs` — FNV-1a 64-bit hex of canonical `steps_json`; 5 unit tests (determinism + collision resistance basics).
6. `sunset.rs` — `default_sunset_at(accepted_at, days)` + sunset SQL helpers; 4 unit tests.
7. `persistence.rs` — `BankDb::open` + `accept` + `get` + `eligible` + `record_dispatch` + `record_verification` + `apply_decay_tick`; 15 unit + 5 contract tests.
8. `admission.rs` — `HumanAcceptanceSignature` + `accept()` orchestration (signature check → SQL); 6 unit + 5 regression tests (F5 paths).
9. `mod.rs` — public re-exports; resources.rs docstring.
10. Property tests (5) — proptest on ordinal totality, sunset monotonicity, idempotent accept, decay clamping, eligible exclusion.
11. Integration tests (15) — `tests/m30_integration.rs` exercising m23→m30 admission, m30→m31 read, m30→m32 resolution, m30→m33 verification handshake; CC-4 + CC-5 + CC-6 cycle.
12. Regression slots (9) — F5/F1/Gap-3 failure scenarios pre-seeded.
13. Mutation pass — `cargo mutants --regex 'm30_curated_bank::escape_surface::.*|m30_curated_bank::admission::.*'`; ≥75% kill required.

---

> Back to: [vault cluster-G spec](../../../the-workflow-engine-vault/module%20specs/cluster-G-bank-select-dispatch-verify.md) · [MODULE_MATRIX](../../MODULE_MATRIX.md) · [ARCHITECTURE](../../../ARCHITECTURE.md) · sister modules: [m30](m30_curated_bank.md) · [m31](m31_selector.md) · [m32](m32_conductor_dispatcher.md) · [m33](m33_verifier.md)
