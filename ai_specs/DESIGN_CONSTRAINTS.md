---
title: DESIGN_CONSTRAINTS — compile-time + runtime invariants
date: 2026-05-17
status: SPEC
constraint_categories: [compile-time, runtime]
cardinality_amendment: "S1002127 — PrivilegeEscalation inserted at ordinal 30 (D-S1002127-02 ADR)"
---

# DESIGN_CONSTRAINTS — workflow-trace

> **Back to:** [`INDEX.md`](INDEX.md) · [`MODULE_MATRIX.md`](MODULE_MATRIX.md) · [`../README.md`](../README.md) · [`../ARCHITECTURE.md`](../ARCHITECTURE.md) · [`../ai_docs/optimisation-v7/STANDARDS/GOD_TIER_RUST.md`](../ai_docs/optimisation-v7/STANDARDS/GOD_TIER_RUST.md)

## Constraint inventory

Two categories: **compile-time** (enforced by rustc + clippy + build.rs) and **runtime** (enforced by typed assertions, CI gates, and Watcher monitoring). Both categories are non-negotiable; violating either fails the gate.

---

## 1. Compile-time invariants

| # | Invariant | Enforcement |
|---|---|---|
| **C1** | Newtype discipline | every domain value (`WorkflowId`, `ProposalId`, `LineageId`, `SessionId`, `BatternId`, `OpaqueClusterId`, `ConsumerId`, `StepId`) is a newtype wrapping a primitive; no raw `String`/`i64` in domain APIs |
| **C2** | `workflow_core::namespace` constants as single source for namespace strings | `rg '"workflow_trace_' src/ \| grep -v 'namespace.rs\|m9_watcher'` returns 0; const-only access via the module |
| **C3** | `#![forbid(unsafe_code)]` at crate root | compiler error if violated |
| **C4** | `EscapeSurfaceProfile` derives `Ord` (cardinality **7** per D-S1002127-02) | enum has explicit `derive(PartialOrd, Ord, Eq, PartialEq, Hash)`; ordinal `Sandboxed(0) < SandboxEscape(10) < ProcessMutate(20) < PrivilegeEscalation(30) < FileWrite(40) < NetworkEgress(50) < DataExfil(60)`; numeric values gap-reserved (steps of 10) for future inserts |
| **C5** | `deny(clippy::unwrap_used)` + `deny(clippy::expect_used)` outside `#[cfg(test)]` | clippy gate fails on violation |
| **C6** | `deny(clippy::pedantic)` warning-converted-to-error at gate | clippy --pedantic stage |
| **C7** | `#![warn(missing_docs)]` on public items | cargo doc clean |
| **C8** | `EscapeSurfaceProfile` snake_case serde rename + `rename_all = "snake_case"` | contract test pins serialized strings |
| **C9** | `#[must_use]` on every public fn returning `Result`/`Option`/builder | clippy::must_use_candidate |
| **C10** | `#[non_exhaustive]` on every public error enum | allows variant addition without semver-major |
| **C11** | `wf-crystallise` does NOT link Cluster G modules; `wf-dispatch` does NOT link Cluster F modules | workspace dep audit + verify-sync invariant #1 |
| **C12** | m8 `build.rs` sets `cargo:rustc-cfg=povm_calibrated` IFF prereq env present; otherwise `compile_error!` (F7 mitigation) | build.rs hard-fail |
| **C13** | Doc comments on every public fn returning `Result` include `# Errors` section | clippy::missing_errors_doc + `cargo doc` clean |
| **C14** | `HumanAcceptanceSignature` constructor private to `wf-dispatch` CLI; no agent-callable constructor | module-private visibility |

---

## 2. Runtime invariants

| # | Invariant | Module | Enforcement |
|---|---|---|---|
| **R1** | m9 namespace-guard called on every write through m13 (and therefore m42) | m9 + m13 + m42 | `m13::write` first-line `m9::assert_namespace(id)?` |
| **R2** | m32 5-check pre-dispatch sequence; any check failure → typed `DispatchError`; no soft-fail | m32 | per-check unit tests; all 5 mocked-failure paths covered |
| **R3** | m10 Ember 7-trait CI rubric over m12 sample-pack output; Held verdict fails CI | m10 | `tests/ember/ci_gate.rs` exit-code assertion |
| **R4** | m8 build-cfg `povm_calibrated` MUST be present at compile; otherwise hard build-fail (C12 enforcement) | m8 | `compile_error!` |
| **R5** | `m14::compute_lift(success, total)` returns `Ok(None)` when `total < 20` (F2 mitigation) | m14 | unit + property test (10k iters) |
| **R6** | `m23::ProposalBuilder::build()` returns `Err(LiftEvidenceMissing)` when lift is None (CC-3 gate) | m23 | unit test |
| **R7** | `m30::accept` requires `HumanAcceptanceSignature { interactive_terminal: true }`; accepted_by not in `{"", "agent", "auto"}` (F5 mitigation) | m30 | unit + integration test |
| **R8** | `m30` admission rejects any second `accept()` for same id (silent downgrade attack vector blocked) | m30 | unit + regression test |
| **R9** | `EscapeSurfaceProfile` declared by m23/m30 must be `>=` derived-from-steps; downgrade returns `EscapeSurfaceInconsistent`. Cardinality LOCKED at **7** (Sandboxed/SandboxEscape/ProcessMutate/PrivilegeEscalation/FileWrite/NetworkEgress/DataExfil) per D-S1002127-02; insertion of any future variant requires reserved-gap allocation (steps of 10) and re-audit of m9 + m30 + m32 + m33 composition tables | m30 | unit test |
| **R10** | m32 self-dispatch refused at BOTH m30 schema (StepClassifier) AND m32 runtime (defense in depth, AP-V7-08) | m30 + m32 | defense-in-depth unit tests |
| **R11** | Cluster H circuit breaker OPEN after 2 consecutive failures; HALF_OPEN after 30s; CLOSED requires 1 success | m40 + m41 + m42 | unit test simulating failure sequence |
| **R12** | Outbox-first JSONL durability: outbox file written + fsynced + renamed BEFORE network RPC | m40 + m41 + m42 | integration test kills process mid-emit; asserts outbox file complete |
| **R13** | m11 `compute_decay_factor` returns `[0.0, 1.0]` clamped for any finite input | m11 | property test (10k iters) |
| **R14** | m11 decay-factor formula: `base + (1 - base) * (f * g * r).clamp(0, 1)` is canonical; no `if no_dispatches skip_decay` branch | m11 | unit + property test |
| **R15** | m33 4-agent unanimous PASS required; partial PASS → DEGRADED verdict | m33 | unit test simulates 3-of-4 PASS → DEGRADED |
| **R16** | m33 `VerifyResult.ttl_expires_at = verified_at + 7d`; no in-place TTL extension | m33 | unit test + no `extend_ttl` method |
| **R17** | m31 composite-score `α·fitness + β·recency + γ·frequency + δ·diversity = 0.40/0.25/0.20/0.15`; δ > 0 enforced at config validation | m31 | unit test + config validation |
| **R18** | m15 atomic JSONL emit (tmp + rename); one file per event (no append mode) | m15 | unit + integration test |
| **R19** | m15 60s de-dup window for same `(kind, context_hash)` | m15 | unit test |
| **R20** | m42 zero POVM dependency (per ADR 2026-05-17); `rg -i 'povm' src/m42_stcortex_emit/ \| grep -v '// POVM-DECOUPLED'` returns 0 | m42 | rg audit at gate |

---

## Enforcement layers

Each invariant is enforced at one or more of:

1. **Compiler** — `forbid` / `deny` lints, `compile_error!`, `#[must_use]`, `#[non_exhaustive]`, type-system (newtypes, private constructors).
2. **Clippy** — `unwrap_used`, `expect_used`, `pedantic`, `missing_docs`, `missing_errors_doc`, `must_use_candidate`.
3. **Build script** — m8's `build.rs` hard-fails on missing prereq.
4. **Unit / property tests** — every invariant has at least one test; property tests run ≥10k iters.
5. **Integration tests** — cross-module invariants (e.g., outbox-first kill-mid-write).
6. **Contract tests** — wire schemas, metric names, serde stability (insta snapshots).
7. **CI gate** — m10 Ember rubric, 4-stage quality gate (check → clippy → pedantic → test).
8. **Verify-sync** — 20 invariants per [`../ai_docs/optimisation-v7/STANDARDS/GOD_TIER_RUST.md`](../ai_docs/optimisation-v7/STANDARDS/GOD_TIER_RUST.md); run at per-Wave-end + per-PR CI.
9. **Watcher monitoring** — Class-I rolling-window for CC-5 silent-failure; Class-D for schema drift; Class-C for refusal-mode accumulation.

## Constraint stability

These constraints are LOCKED at v1.3; relaxation requires a v1.4+ amendment via the D-B6 AMEND-loop (CC-7 pressure-driven evolution). Constraints CANNOT be relaxed silently — m15 emits a pressure event on any module attempt to violate, and the Watcher / Zen audit lanes pick it up.

## Cross-references

- **god-tier 18 rules:** [`../ai_docs/optimisation-v7/STANDARDS/GOD_TIER_RUST.md`](../ai_docs/optimisation-v7/STANDARDS/GOD_TIER_RUST.md)
- **20 verify-sync invariants:** same doc § "Verify-sync invariants"
- **EscapeSurfaceProfile contract:** [`../ai_specs/modules/cluster-G/m30_curated_bank.md`](modules/cluster-G/m30_curated_bank.md) § 2
- **Cardinality-7 ADR (D-S1002127-02):** [`../ai_docs/optimisation-v7/decisions/2026-05-17-escape-surface-cardinality-7-privilege-escalation.md`](../ai_docs/optimisation-v7/decisions/2026-05-17-escape-surface-cardinality-7-privilege-escalation.md) — PrivilegeEscalation inserted at ordinal 30; closes Gap 3 G7 stability concern via numeric gap reservation (steps of 10)
- **AP30 namespace discipline:** [`SECURITY_SPEC.md`](SECURITY_SPEC.md) + [`synergies/CC-2.md`](synergies/CC-2.md)
- **F7 graceful-degrade ban:** [`CONSENT_SPEC.md`](CONSENT_SPEC.md) + [`SECURITY_SPEC.md`](SECURITY_SPEC.md)

---

> **Back to:** [`INDEX.md`](INDEX.md) · [`../ai_docs/optimisation-v7/STANDARDS/GOD_TIER_RUST.md`](../ai_docs/optimisation-v7/STANDARDS/GOD_TIER_RUST.md)
