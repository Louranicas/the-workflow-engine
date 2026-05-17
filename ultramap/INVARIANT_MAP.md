---
title: INVARIANT_MAP — compile-time and runtime invariants per cluster
kind: planning-only · operational view · complements canonical V7 ULTRAMAP
status: Wave-2 author deliverable; no source authored
date: 2026-05-17
cardinality_amendment: "S1002127 — PrivilegeEscalation inserted at ordinal 30 (D-S1002127-02 ADR)"
---

# INVARIANT_MAP — what must always hold

> **Back to:** [`README.md`](README.md) · [`../ARCHITECTURE.md`](../ARCHITECTURE.md) · canonical [`../ai_docs/optimisation-v7/ULTRAMAP.md`](../ai_docs/optimisation-v7/ULTRAMAP.md) · [`../ai_docs/optimisation-v7/ANTIPATTERNS_REGISTER.md`](../ai_docs/optimisation-v7/ANTIPATTERNS_REGISTER.md) · siblings [`MODULE_DEPENDENCY_GRAPH.md`](MODULE_DEPENDENCY_GRAPH.md) · [`DATA_FLOW.md`](DATA_FLOW.md) · [`CONTROL_FLOW.md`](CONTROL_FLOW.md) · [`CONTEXTUAL_FLOW.md`](CONTEXTUAL_FLOW.md)
>
> **Purpose:** the engine's invariant ledger, split between **compile-time** (caught by `rustc`, clippy, or `build.rs`) and **runtime** (caught by typed errors, structural refusal, or external monitoring). Per [`../ARCHITECTURE.md`](../ARCHITECTURE.md) the engine has zero tolerance for compile or clippy warnings (`deny_clippy_warnings = true`, `deny_pedantic_warnings = true`); per [`CONTROL_FLOW.md`](CONTROL_FLOW.md) every Cluster G module returns a typed `Err` on every refusal path. This file catalogues which invariants are which, organised so a reader can answer "if I touch m20, what could I break at compile time vs at runtime?".

---

## Compile-time invariants (caught by the toolchain)

These hold by virtue of the code refusing to compile if violated. They are the engine's *first* line of defense — earlier than tests, earlier than CI gates.

| Invariant | Enforced by | Owner | Cluster scope |
|---|---|---|---|
| **Newtype discipline** — `SessionId`, `WorkflowId`, `LineageId`, `StepToken`, `ConsumerId` are wrappers, not `String`/`u32` aliases | `pub struct SessionId(String);` etc. in `workflow_core::types` | `workflow_core` | all clusters |
| **AP30 namespace constants** — every stcortex pathway ID prefix sourced from `workflow_core::namespace::WORKFLOW_TRACE_PREFIX` | clippy `disallowed_methods` lint banning string literals matching `^"workflow_trace_"`; grep gate in CI as defense-in-depth | `workflow_core::namespace` | D (m9), C (m13), H (m42) |
| **`#![forbid(unsafe_code)]`** in `lib.rs` | `rustc` | `workflow_core` crate root | every module (entire crate) |
| **`cargo:rustc-cfg=povm_calibrated` build prereq** | `build.rs` emits the cfg only when env var probe succeeds; `#[cfg(not(povm_calibrated))] compile_error!("POVM calibration prereq unmet")` at crate root | D (m8) | gates the entire crate |
| **`EscapeSurfaceProfile: Ord`** — cardinality **7** with stable variant order `Sandboxed(0) < SandboxEscape(10) < ProcessMutate(20) < PrivilegeEscalation(30) < FileWrite(40) < NetworkEgress(50) < DataExfil(60)` (per D-S1002127-02; gap-reserved at steps of 10) | `#[derive(Ord, PartialOrd, Eq, PartialEq)]` on enum + `#[repr(u8)]` with explicit discriminants; serde rename attributes stable; clippy non-exhaustive match catches drift across m9 / m30 / m32 / m33 | G (m30) | G (m30), G (m32), G (m33), D (m9) |
| **`thiserror` typed-error taxonomies, no `Box<dyn Error>`** | every module declares `#[derive(thiserror::Error)] pub enum ModuleNameError` | all writer modules | every cluster |
| **`#[must_use]` on m12 report types** | attribute on report struct; clippy `must_use_unit` lint | C (m12) | C |
| **`Result<T, E>` on every fallible API; no `panic!` outside test cfg** | clippy `panic` lint denied; grep gate in CI | all | every cluster |
| **No `Command::*` exec primitives in `wf-dispatch` binary** | clippy `disallowed_methods` listing `std::process::Command::new`, `tokio::process::Command::new`, `std::os::unix::process::CommandExt`; grep gate in CI as defense-in-depth | G (m32) | G (m32) — defends AP-V7-08 |
| **No HTTP server in `wf-dispatch` binary** | grep gate: `cargo metadata` cross-checked against `axum`/`hyper::Server`/`tokio::net::TcpListener::bind` — none allowed in `wf-dispatch`'s dependency closure | G (all four modules) | G — defends "wf-dispatch is sender, not server" |
| **`no_std` not used; tokio runtime mandatory** | `Cargo.toml` declares tokio with `features = ["full"]`; no `#[no_std]` crate-root attribute | `workflow_core` | every async module |
| **Doc comments on every `pub` item** | `#![warn(missing_docs)]` at crate root + CI `cargo doc --no-deps --all-features 2>&1 | grep "warning:" → fail` | all | every cluster |

### m8 build.rs gate — the highest invariant

Per [`../ai_specs/modules/cluster-D` (referenced from MODULE_MATRIX m8 row)] and [`../ai_docs/optimisation-v7/ULTRAMAP.md`](../ai_docs/optimisation-v7/ULTRAMAP.md) View 2 m8:

```rust
// build.rs — planning-spec only, NOT compileable here
fn main() {
    if env_probe_povm_calibration_ok() {
        println!("cargo:rustc-cfg=povm_calibrated");
    }
    // else: cfg is not emitted; the lib.rs guard below fires
}
```

```rust
// src/lib.rs — planning-spec only
#[cfg(not(povm_calibrated))]
compile_error!("POVM calibration prereq unmet — m8 build.rs did not emit cfg(povm_calibrated). \
                See ai_specs/modules/cluster-D for the prereq env probe contract.");
```

The compile_error is the engine's first refusal — without this gate cleared, no other module compiles, no tests run, no binaries are produced. **m8's `povm_calibrated` cfg name is historical** per the 2026-05-17 m42 ADR; the actual prereq remains the env probe success, and the cfg name may be revised in a future spec rev. The invariant itself stands.

---

## Runtime invariants (caught by typed errors or external monitoring)

These are the engine's *second* line of defense — they fire when the toolchain has cleared the code path but the runtime conditions have not. Most return a typed `Err`; a few are observable only externally (CC-5 substrate movement, m15 pressure event accumulation).

| Invariant | Enforced by | Owner | Failure mode |
|---|---|---|---|
| **m9 namespace guard at write boundaries** — every m13/m42 write call passes through `namespace_guard::validate(prefix)` | runtime call in m13/m42 write path | D (m9) | `WriteError::NamespaceViolation` (typed); blocks the write |
| **m10 Ember 7-trait CI gate** — Held verdict on user-facing strings fails CI | CI workflow runs `m10::audit_strings`; any Held → exit 1 | D (m10) | CI fails; PR blocked until amend lands |
| **m13 3-band LTP/LTD gate** — only writes whose Hebbian magnitude lands in one of three configured bands actually fire wire-write | runtime band check in m13; sub-threshold writes deferred to JSONL queue | C (m13) | (silent — band-skipped writes accumulate in deferred queue; not an error) |
| **m23 NO auto-promote (AP-V7-07)** — m30 admits ONLY via explicit `HumanAcceptanceSignature` | `BankDb::accept(.., HumanAcceptanceSignature)` argument required; no `auto_promote()` exists | G (m30) | `BankError::AcceptanceRequiresHumanSignature` |
| **m32 5-check pre-dispatch + no-self-dispatch (AP-V7-08)** — order contractual, self-dispatch refused | runtime check sequence; `Dispatcher::Refuse` variant for B3 blocker | G (m32) | typed `DispatchError::{ConductorNotLive,VerificationStale,DefinitionDrifted,Sunset,CooldownActive,SelfDispatchRefused,ConductorDispatchDisabled}` |
| **m33 4-agent unanimous-PASS for DataExfil** — DataExfil-classed workflows require unanimous PASS from all 4 agents (security-auditor + performance-engineer + silent-failure-hunter + zen); any Degraded/Fail → `VerifyResult::Fail` | runtime tally in m33 | G (m33) | `VerifyResult { verdict: Fail }` (typed); m32 then refuses dispatch on check 2 |
| **m40 outbox-first** — JSONL append + fsync *before* wire attempt | runtime sequence in m40 emit path | H (m40) | (no error; just durability — wire failure is recoverable) |
| **m41 outbox-first** — same | runtime sequence in m41 emit path | H (m41) | (same) |
| **m42 outbox-first** — same; substrate-unavailable returns `ReinforceOutcome::SubstrateUnavailable`; **never** falls back to POVM | runtime sequence in m42 emit path | H (m42) | typed `ReinforceOutcome::SubstrateUnavailable` |
| **Circuit breaker on 2 consecutive failures** — `m40_42_common::breaker` opens after 2 failures; half-open probe after exponential back-off (30s → 5 min cap) | shared `Breaker` state machine | H (m40/41/42) | (no error returned; just skip wire entirely; metric `*_circuit_open_total` increments) |
| **AP-V7-13 Health-200 ≠ behaviour-verified** — every `/health` probe paired with a semantic-endpoint check | m32 Conductor probe checks BOTH `status == 200` AND `body.status == "ok"` (not status-code alone); m42 monitors CC-5 substrate-delta externally rather than trusting stcortex 200 on write | G (m32), H (m42) | typed `DispatchError::ConductorNotLive { reason }` when semantic check fails despite 200 |
| **Per-cluster panic→Result discipline** — every fallible path returns a typed `Err`; `panic!` is reserved for in-test-only paths | clippy + CI grep gate | all | (impossible to violate at runtime if compile gate held) |
| **CC-5 substrate movement** — `stcortex.pathway.weight` delta on `workflow_trace_*` IDs must be observable over rolling 7-day window post first 5+ dispatches | external monitoring — Watcher Class-I pre-position | H (m42), G (m31) | Watcher Class-I fires if no movement for 4+ weeks → workflow-level improvement candidate (not in-engine error) |
| **AP-Hab-11 hyphen-slug discipline** — stcortex `pre_id`/`post_id` slugs replace hyphens with underscores at the slug boundary | runtime call in m42 + m13 stcortex writer | H (m42) | (silent — slug encoded; no error) |
| **m32 audit-first** — `dispatch_log` row inserted **before** Conductor request egress | runtime sequence in m32 (audit insert is check-0, before check 1) | G (m32) | `DispatchError::AuditWrite` (typed) blocks the dispatch — if audit fails, no egress |
| **m32 Gap 3 banner display-before-step** — `EscapeSurfaceProfile::banner_line()` printed to stdout BEFORE every `conductor_client.dispatch_step()` | runtime sequence in m32; suppression via env var **forbidden** | G (m32) + D (m9) | (no error — visible regression in operator terminal if missing) |
| **m33 record_verification only on PASS/DEGRADED** — no "refreshed" timestamp without an underlying probe (AP-V7-13 cousin: diagnostics theatre) | runtime: `m33::record_verification` callable only inside the PASS/DEGRADED branch | G (m33) | (no error — but Watcher would flag refresh-without-probe via Class-D drift) |
| **m15 pressure event de-dup window** — same kind+context within 60s coalesce to one file with `count++` | runtime in m15 emit path | E (m15) | (no error — operator surface stays clean) |
| **stcortex offline JSONL fallback** — if `:3000` is unreachable, read `data/snapshots/latest.json`; **never silently fall back to POVM** | runtime conditional in m13/m42 read path per workspace charter | C (m13), H (m42) | `ReinforceOutcome::SubstrateUnavailable` typed return |
| **Refusal observability** — every refusal (substrate-authored, engine-authored, operator-authored) emits a `WireEvent::Refusal { token: RefusalToken, ... }` Class-C envelope on m40's NexusEvent push **before** the typed `Err` returns to the caller. Closes NA-GAP-11; substrate-readable, not Watcher-inferred-from-absence | runtime emission in m32 / m42 / m30 / m41 refusal paths | G (m32), H (m42), G (m30), H (m41) | typed `Err` plus durable wire emission; verification per [`../ai_specs/cross-cutting/refusal-taxonomy.md`](../ai_specs/cross-cutting/refusal-taxonomy.md) closure test |
| **Substrate-drift first-class** — every substrate-touching path participates in the substrate-drift canary contract; CR-2-class semantic drift (delta > 5× baseline) halts dependent dispatches and emits `SubstrateDriftDetected` to m15 pressure register | runtime canary check at session-start + on-demand per substrate | A (m1, m2, m3), G (m32), H (m40, m41, m42) | `SubstrateDriftDetected` event + m15 pressure event + watcher-notice; spec at [`../ai_specs/cross-cutting/substrate-drift.md`](../ai_specs/cross-cutting/substrate-drift.md) |

---

## Per-cluster invariant summary

### Cluster A — Substrate Ingest (L1)

- **Read-only at every level.** URI `?mode=ro` + `PRAGMA query_only = ON` (m1); reducer-callback subscription is push-only (m2); no write API exists for injection.db consumer (m3).
- **Byte-preserving.** No string normalisation, trimming, or folding in any A module — downstream FNV-1a XOR derivations depend on byte-identity.
- **Cursor monotonicity (m1).** `next_page(cursor)` is deterministic; same cursor in → same rows out.
- **Narrowed-scope subscription (m2).** Only `tool_call` + `consumption` tables; other tables rejected at boundary.

### Cluster B — Habitat Observation (L2)

- **Opaque cluster IDs (m4 — F11).** `cluster_id: u64` is FNV-1a XOR of step hashes — non-invertible without the alphabet; identical cascades across sessions get the same ID; human-readable labels structurally forbidden at this layer.
- **Unlabelled batterns preserved (m5).** `label: Option<String>`; unlabelled batterns are not dropped.
- **EMA excludes Converged (m6 — F10).** 20-session rolling baseline excludes Converged outcomes so the metric represents uncertain-cost, not confident-cost.

### Cluster C — Correlation + Output (L3)

- **F9 zero-weight JSONB schema (m7).** `consumer_inputs` is additive — new consumers join by adding a key; existing keys never removed (deprecated keys tombstoned with null, kept in schema).
- **3-band LTP/LTD gate (m13).** Only writes whose Hebbian magnitude lands in one of three configured bands fire wire-write; sub-threshold deferred to JSONL queue.
- **Stable schema as coupling surface.** m4 and m6 talk only through m7's JSONB; never directly call each other (per [`CROSS_CLUSTER_SYNERGIES.md`](../ai_docs/optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md) § CC-1).

### Cluster D — Trust (L4 aspect, ship-first)

- **Aspect-woven, not called (m8/m9/m10/m11).** No module imports an aspect; the aspect is *applied to* the module via build.rs / CI gate / runtime validation hooks.
- **`povm_calibrated` cfg gates entire crate (m8).** No other module compiles if absent.
- **AP30 namespace constants single-source-of-truth (m9).** All `workflow_trace_*` strings sourced from `workflow_core::namespace`.
- **NOT feature-gated.** Cluster D ships in `default` features — invariants cannot be bypassed by `--no-default-features`.
- **Day 1 ordering.** Ships BEFORE Cluster A (per `plan.toml [[layers]] L4 ship_first = true`).

### Cluster E — Evidence + Pressure (L5)

- **`None` propagates (m14).** Wilson CI below n=20 returns `Option::None` — explicitly NOT a numeric stand-in.
- **JSONL one-event-per-file (m15).** No append mode; atomic tmp + rename; filename `PHASE-B-RESERVATION-NOTICE-{ts}_{event_id}.jsonl`.
- **De-dup window 60s (m15).** Same kind+context within 60s coalesce.

### Cluster F — Iteration KEYSTONE (L6)

- **CC-3 evidence gate at construction (m23).** `lift.ok_or(ProposalError::LiftEvidenceMissing)?` is the literal gate line; m23 cannot construct a proposal without Wilson-CI-stable lift evidence.
- **Opaque `StepToken` alphabet (m20 — F11).** `pub struct StepToken(pub u32);` — numeric, not human-readable; display resolved only at m12 report-emit time.
- **Deterministic pattern ordering (m20).** Sort by support DESC, then length DESC; same input → same output.
- **Bounded right-gap (m20).** `MAX_GAP_STEPS=5`; bounded depth (max pattern length 8) keeps PrefixSpan in practical complexity envelope.
- **Top-K-by-distance N=3 (m23 — "gradient preservation").** Proposals are a small diverse slate, never single greedy best.

### Cluster G — Bank / Select / Dispatch / Verify (L7)

- **F5 admission discipline / AP-V7-07 hard-refusal (m30).** No `auto_promote()` function exists; admission requires `HumanAcceptanceSignature`.
- **Sunset is immutable (m30).** `sunset_at` set at accept time (default +120 days); cannot be extended in-place; SQL `WHERE sunset_at > now_ms` is the eligibility gate.
- **Definition hash is monotonic (m30).** `steps_json` immutable post-accept; m32 hashes resolved steps at dispatch time and compares against m33's stored `definition_hash`.
- **Gap 3 ordinal stability (m30; cardinality 7 per D-S1002127-02).** `EscapeSurfaceProfile` is `Ord`-bearing with stable serde names; cardinality is 7 (Sandboxed/SandboxEscape/ProcessMutate/PrivilegeEscalation/FileWrite/NetworkEgress/DataExfil at ordinals 0/10/20/30/40/50/60 with gap reservation); reordering or renaming variants is a contract break; new variants must occupy reserved numeric gaps.
- **5-check pre-dispatch sequence enforced in order (m32).** Order is contractual; failure at check N short-circuits with the correct typed error; checks N+1..5 do not run.
- **Conductor-only routing (m32).** Hard-refuse when `CONDUCTOR_DISPATCH_ENABLED != "1"` OR `:8141/health` fails. NOT a silent no-op, NOT a fall-through-to-LCM, NOT a delayed retry.
- **Display-before-step Gap 3 banner (m32).** `EscapeSurfaceProfile::banner_line()` printed to stdout BEFORE every `conductor_client.dispatch_step()`; suppression forbidden.
- **Audit-first writes (m32).** `dispatch_log.db` row inserted before Conductor egress.
- **AP-V7-08 self-dispatch refusal (m30 + m32).** Defense in depth: m30 schema rejects steps targeting m32; m32 inspects resolved steps at runtime and returns `Err(SelfDispatchRefused)`.
- **4-agent unanimous-PASS for DataExfil (m33).** Any Degraded/Fail from any of 4 agents → `VerifyResult::Fail`.
- **7-day TTL on VerificationReceipt (m33).** Manual re-verify or scheduled 6h sweep refreshes; m32 refuses dispatch on stale TTL.

### Cluster H — Substrate Feedback (L8)

- **Outbox-first JSONL durability (m40, m41, m42).** Append + fsync before wire attempt; wire failure recoverable from outbox.
- **Circuit breaker on 2 consecutive failures (m40_42_common).** Open after 2 failures; half-open probe with exponential back-off.
- **stcortex-only routing for m42 (per 2026-05-17 ADR).** NO POVM branch, NO `povm_overlap_active` config flag, NO `route_reinforcement` switch. Single code path to stcortex via m13.
- **`fitness_delta` constants module-level (m42).** `FITNESS_PASS_VERIFIED = +0.25`, `FITNESS_PASS = +0.15`, `FITNESS_BLOCKED = -0.05`, `FITNESS_FAIL = -0.10`; clamped `[-1.0, 1.0]` (Hebbian v3 defense-in-depth pattern).
- **AP-Hab-11 hyphen-slug encoding (m42).** Hyphens → underscores at the slug boundary (S1001757 munge bug).
- **Idempotency via `request_id` UUIDv4 (m42).** Stcortex de-dup window 1h.
- **Never silently fall back to POVM (m42).** If stcortex unreachable post-cutover, log ERROR + return `ReinforceOutcome::SubstrateUnavailable` + outbox carries durable record.
- **AP-V7-13 awareness (m42).** stcortex 200 on `/pathway` write does NOT confirm pathway.weight moved; observed externally via Class-I monitoring.

---

## Invariants the engine deliberately does NOT enforce

Some invariants are useful elsewhere but rejected here, on principle. Each rejection is a design decision:

| Not enforced | Why |
|---|---|
| **Strict monotonic dispatch ordering across workflows** | Each workflow has its own cooldown; cross-workflow ordering is for Conductor to decide. Enforcing it here would couple workflow ids unnecessarily. |
| **Auto-recovery of stuck verifications** | m33 verifications expire on TTL; re-verify is operator-driven or scheduled-sweep-driven. Auto-resurrection would mask m33 agent failures. |
| **Workflow merging or rewriting at admit time** | m30 admits `steps_json` immutable. Workflow-rewriting is m21/m23 territory upstream of acceptance. |
| **Centralised metrics aggregation** | Each module owns its counters; aggregation happens at the observability layer (per [Phase 8 of `../ai_docs/optimisation-v7/ULTRAMAP.md`](../ai_docs/optimisation-v7/ULTRAMAP.md) View 3 timeline). Engine-internal aggregation would create a single point of failure. |
| **Hot config reload** | Config baked into `Cargo.toml` features + env vars at boot; runtime reload is out of scope for v0.1.0 (per `plan.toml` versioning). |
| **Cross-binary IPC** | `wf-crystallise` and `wf-dispatch` communicate ONLY through m30 SQLite bank; no shared memory, no Unix sockets, no in-process channel. The SQLite read-side serialises the coupling — simpler than IPC, harder to misuse. |

---

## Reading the invariant ledger as a refusal map

Every invariant has a refusal shape — the structural form of the failure. The engine's Cluster G is built around the principle that **refusal is a first-class result**, not an exception:

| Refusal shape | Examples |
|---|---|
| **Type-level (compile-time)** | newtype discipline; AP30 constants; `compile_error!`; `#![forbid(unsafe_code)]` |
| **Typed error variant (runtime)** | `DispatchError::*` (8 variants for m32 alone); `BankError::AcceptanceRequiresHumanSignature`; `ProposalError::LiftEvidenceMissing` |
| **Refuse-mode (runtime, structural)** | `Dispatcher::Refuse` enum variant — borrow checker prevents bypass; m13 deferred-buffer for sub-threshold writes |
| **Externally observable (substrate)** | CC-5 substrate-delta zero → Watcher Class-I; AP-V7-13 diagnostics-theatre → Watcher Class-D drift |
| **Silent durability (outbox)** | m40/m41/m42 outbox-first: substrate down is recoverable from JSONL, not surfaced as an error to the caller |

Per [`CROSS_CLUSTER_SYNERGIES.md`](../ai_docs/optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md) Watcher Class C summary: every refusal in Cluster G is observable as Class C (refusal is correct behaviour, not a bug). The Class C count is the engine's **healthiest** metric — if Class C is zero over a long window, the engine is failing-open rather than failing-closed.

---

## Cross-references

| Question | Answer | File |
|---|---|---|
| What does the build graph look like? | Mermaid graph TD by cluster | [`MODULE_DEPENDENCY_GRAPH.md`](MODULE_DEPENDENCY_GRAPH.md) |
| What rows travel each edge? | per-edge type table | [`DATA_FLOW.md`](DATA_FLOW.md) |
| When does each module fire? | trigger taxonomy + sequenceDiagrams | [`CONTROL_FLOW.md`](CONTROL_FLOW.md) |
| What metadata attends each emit? | context table | [`CONTEXTUAL_FLOW.md`](CONTEXTUAL_FLOW.md) |
| What is the antipattern register? | AP-V7-* full catalogue | [`../ai_docs/optimisation-v7/ANTIPATTERNS_REGISTER.md`](../ai_docs/optimisation-v7/ANTIPATTERNS_REGISTER.md) |
| What is the per-CC contract surface? | inventory + closure-test paths | [`../ai_docs/optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md`](../ai_docs/optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md) |
| What is the canonical layer view? | View 1 Mermaid | [`../ai_docs/optimisation-v7/ULTRAMAP.md`](../ai_docs/optimisation-v7/ULTRAMAP.md) |

---

> **Back to:** [`README.md`](README.md) · [`../ARCHITECTURE.md`](../ARCHITECTURE.md) · canonical [`../ai_docs/optimisation-v7/ULTRAMAP.md`](../ai_docs/optimisation-v7/ULTRAMAP.md) · [`ULTRAMAP.md`](ULTRAMAP.md) (this folder's master synthesis)
