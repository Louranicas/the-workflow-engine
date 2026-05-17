---
title: Cluster G — Bank / Select / Dispatch / Verify (Layer L7) — Layer Spec
cluster: G
layer: L7
module_count: 4
modules: [m30_curated_bank, m31_selector, m32_conductor_dispatcher, m33_verifier]
binary: wf-dispatch
feature_gates: [api]
cc_owns: [CC-4 (G ownership of bank-select-dispatch), CC-5 (m32 emit-side trigger), CC-6 (m33↔m32)]
cc_consumes: [CC-2, CC-4]
ship_priority: Day 4 Wave 3 (after Cluster F proposes)
status: SPEC
date: 2026-05-17
hold_v2_compliant: true
---

# Cluster G — Bank / Select / Dispatch / Verify (L7)

> **Back to:** [`../INDEX.md`](../INDEX.md) · [`../MODULE_MATRIX.md`](../MODULE_MATRIX.md) · [`../../README.md`](../../README.md) · [`../../ARCHITECTURE.md`](../../ARCHITECTURE.md) · vault [[cluster-G-bank-select-dispatch-verify]] · [`../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-G.md`](../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-G.md)

## Role

Cluster G IS **the dispatch binary** — the four modules that turn human-accepted `WorkflowProposal` artefacts into actual workflow executions dispatched via HABITAT-CONDUCTOR. This is the entirety of `wf-dispatch`; no other binary links m30/m31/m32/m33. The cluster owns the **Gap 3 EscapeSurfaceProfile schema** (m30) — the unified destructiveness classifier that replaces five scattered classifiers across `SKILL-forge.md` / `SKILL-genesis.md` / `SKILL-pre-deploy-hardening.md` / `SKILL-silent-swallow-detect.md` / `hookify.preserve-blanket-guard.local.md` (S102 openclaw scar tissue), the **m32 5-check pre-dispatch sequence** (verifier TTL → namespace → escape-surface display → Conductor health → self-dispatch check), and the **m33 4-agent verifier** (Zen + security-auditor + silent-failure-hunter + performance-engineer; unanimous PASS within 7-day TTL).

The cluster's structural facts are: m30 is the only admission surface (CC-4 F5 mitigation); m31 selection composite-score uses `α·fitness + β·recency + γ·frequency + δ·diversity = 0.40/0.25/0.20/0.15` per Genesis v1.3 § 3 m31 row with refusal-or-flag against degraded substrate; m32 dispatches via Conductor ONLY (Town Hall P0 #3); m32 is never a workflow target (AP-V7-08 — self-dispatch refused at m30 schema layer); m33 verdicts are PASS/FAIL/DEGRADED with 7-day `last_verified_at` TTL gating re-runs.

## Modules

| Module | Spec | LOC | Tests | Verb |
|---|---|---:|---:|---|
| `m30_curated_bank` | [`../modules/cluster-G/m30_curated_bank.md`](../modules/cluster-G/m30_curated_bank.md) | 220 | 70 | record+select admission |
| `m31_selector` | [`../modules/cluster-G/m31_selector.md`](../modules/cluster-G/m31_selector.md) | 240 | 70 | select (diversity-enforced) |
| `m32_conductor_dispatcher` | [`../modules/cluster-G/m32_conductor_dispatcher.md`](../modules/cluster-G/m32_conductor_dispatcher.md) | 290 | 75 | dispatch (Conductor-only) |
| `m33_verifier` | [`../modules/cluster-G/m33_verifier.md`](../modules/cluster-G/m33_verifier.md) | 200 | 70 | verify (4-agent dry-run) |

## Cross-cluster contracts

- **OWNS:**
  - **CC-4 (G ownership)** — m30 admits, m31 selects, m32 dispatches; the entire pipeline downstream of human-review boundary. See [`../synergies/CC-4.md`](../synergies/CC-4.md).
  - **CC-5 (emit-side trigger)** — m32's `DispatchOutcome` is the fire-and-forget input to Cluster H emit modules. See [`../synergies/CC-5.md`](../synergies/CC-5.md).
  - **CC-6 (G-internal: m33↔m32)** — m33's `VerifyResult { definition_hash, ttl_expires_at, verdict }` is read by m32 at dispatch check 2 (TTL) and check 3 (hash match). See [`../synergies/CC-6.md`](../synergies/CC-6.md).
- **CONSUMES:**
  - **CC-2** — m9 namespace-guard wraps every m30/m32 write; m8 build-prereq applies; m10 Ember rubric over m32's banner text.
  - **CC-4** — m23's `WorkflowProposal` is the upstream input; m30 admits via CLI human-signed acceptance.

## Binary placement

All four modules live in **`wf-dispatch`** exclusively. This is the canonical two-binary split — `wf-crystallise` does NOT link Cluster G; this is verified by the workspace-level dependency graph audit and verify-sync invariant #1. The split prevents the worst F8 violation (ingest binary calling dispatch primitives and creating a self-reinforcing loop).

## Feature-gate posture

**`feature = "api"`** gated. m30/m31/m32/m33 expose CLI surfaces and require the api feature; with api off, `wf-dispatch` produces no usable binary (intentional — there is no headless dispatch mode).

## Ship priority

**Day 4 Wave 3 — after Cluster F proposes.** Implementation order (dependency-strict):

1. **m30** first — `EscapeSurfaceProfile` ordinal enum lands first; m9 imports it; m32 displays it.
2. **m33** second — verifier produces `VerifyResult` that m32 reads.
3. **m32** third — depends on m30 (resolution), m33 (TTL + hash), m9 (namespace assert).
4. **m31** last — depends on m30 (eligible read) + m11 decay (Cluster D); selection composite-score is the simplest part of the cluster but ships after the rest because its tests assert end-to-end m30→m31→m32 round-trip.

## Operational invariants

1. **F5 admission discipline (AP-V7-07).** `m30::accept` requires `HumanAcceptanceSignature { interactive_terminal: true, accepted_by != "agent"|"auto" }`. No agent-callable insert path.
2. **EscapeSurfaceProfile ordinal stability.** `Sandboxed < SandboxEscape < ProcessMutate < FileWrite < NetworkEgress < DataExfil` is contract-binding. m32 banner display uses `>=` comparisons; m9 namespace-write gate uses the same ordinal. Reordering is a semver-major break.
3. **m32 5-check pre-dispatch sequence is strict.** (1) Conductor `:8141/health`; (2) `m33.VerifyResult.ttl_expires_at > now`; (3) `definition_hash` matches; (4) `sunset_at > now`; (5) `dispatch_cooldown` elapsed. All five must pass; any failure returns typed `DispatchError`. No soft-fail path.
4. **m32 Conductor-only routing.** Direct workflow exec from m32 is rejected at compile time (no `exec_local` symbol); Conductor breaker OPEN returns `DispatchError::ConductorDispatchDisabled` typed ERROR — never silent no-op.
5. **m33 4-agent unanimous PASS.** Zen + security-auditor + silent-failure-hunter + performance-engineer all must return PASS. Any DEGRADED yields verdict DEGRADED; any FAIL yields FAIL. 7-day TTL on `last_verified_at`; expiry forces re-verify before next dispatch.
6. **m31 diversity-enforced selection.** Composite-score `α·fitness + β·recency + γ·frequency + δ·diversity = 0.40/0.25/0.20/0.15`. The diversity term is the F11 mitigation at selection layer; pure-fitness selection collapses to monoculture.
7. **Self-dispatch refusal (AP-V7-08).** A workflow whose steps include `m32::dispatch` is rejected at m30 schema layer (StepClassifier returns `EscapeSurfaceProfile::SandboxEscape` minimum and the admission asserts m32 is not a `measurement_target`). Defense in depth — m32 also refuses at dispatch time.

## Failure modes the cluster structurally refuses

- **m30 auto-promote from m23.** Hard refusal via `HumanAcceptanceSignature.interactive_terminal` + accepted_by blocklist; no code path bypasses.
- **m32 direct exec.** No `exec_local` / `dispatch_direct` symbol; Conductor-bypass impossible by construction.
- **m32 self-dispatch.** m30 schema refuses; m32 also refuses; double mitigation.
- **m33 partial-quorum PASS.** 4 of 4 must PASS; 3 of 4 → DEGRADED at best.
- **EscapeSurfaceProfile silent downgrade.** m30 admission rejects any second `accept()` for same id; reclassify-to-softer attack vector is blocked.
- **m31 fitness-only selection.** δ·diversity term is contract-binding; setting δ=0 is rejected at config validation.

## Performance envelope

| Operation | Target | Notes |
|---|---|---|
| m30 `accept(wf, sig)` | < 50 ms | SQL insert + audit row write |
| m30 `eligible(now, limit=100)` | < 20 ms | covered by `idx_bank_weight_sunset` |
| m31 `select` (100 candidates) | < 30 ms | composite-score + diversity filter |
| m32 5-check sequence | < 200 ms p99 | Conductor health is the dominant latency (HTTP) |
| m32 dispatch + outcome wait | bounded by Conductor; typed timeout 30 s | |
| m33 4-agent verify | < 5 min | parallel agent dispatch via Conductor; each agent ~60 s budget |

## Verify-sync invariants

- **#1** — `src/m30_curated_bank/`, `src/m31_selector/`, `src/m32_conductor_dispatcher/`, `src/m33_verifier/` all exist post-G9.
- **#15** — `EscapeSurfaceProfile` symbol exported from `m30::escape_surface::EscapeSurfaceProfile`; m9 imports it; m32 imports it. Audit: `rg 'EscapeSurfaceProfile' src/ | grep -v 'tests/'` returns hits only in m30 + m32 + m9.
- **#17** — `tests/integration/cc4_proposal_bank_dispatch.rs`, `tests/integration/cc5_substrate_learning_loop.rs`, `tests/integration/cc6_verification_gated_dispatch.rs` all exist.
- **#19** — `EscapeSurfaceProfile` enum cardinality is 5 (Sandboxed/SandboxEscape/ProcessMutate/FileWrite/NetworkEgress/DataExfil — 6 variants; the original v1.2 spec listed 5; v1.3 adds DataExfil for openclaw scar tissue per Genesis v1.3 § 1.a — verify count is 6 in v1.3).
- **#20** — `m32` dispatch path NEVER invokes workflow exec directly; `rg 'dispatch_direct\|exec_local' src/m32_conductor_dispatcher/` returns 0.

## Per-module cross-links

- [`../modules/cluster-G/m30_curated_bank.md`](../modules/cluster-G/m30_curated_bank.md) — curated bank + EscapeSurfaceProfile (Gap 3)
- [`../modules/cluster-G/m31_selector.md`](../modules/cluster-G/m31_selector.md) — composite-score selector (diversity-enforced)
- [`../modules/cluster-G/m32_conductor_dispatcher.md`](../modules/cluster-G/m32_conductor_dispatcher.md) — Conductor-only dispatcher + 5-check sequence
- [`../modules/cluster-G/m33_verifier.md`](../modules/cluster-G/m33_verifier.md) — 4-agent verifier + 7-day TTL

## Antipatterns specific to Cluster G

- **AP-V7-07** (auto-promote m23 → m30) — structural refusal at m30 admission.
- **AP-V7-08** (self-dispatch) — refused at both m30 (schema) and m32 (runtime).
- **AP-WT-F4** (premature dispatch) — m32 5-check sequence mandatory.
- **AP-WT-F1** (bank ossification) — m30 sunset immutable; m11 decay drives sunset.
- **AP-WT-F5** (bank creep) — admission interactive-only.
- **AP-WT-F11** (monoculture) — m31 δ·diversity term.
- **AP-Drift-06** (bridge contract drift) — m32↔Conductor wire-contract pinned at `tests/contract/m32_conductor_wire.rs`.

---

> **Back to:** [`../INDEX.md`](../INDEX.md) · [`../MODULE_MATRIX.md`](../MODULE_MATRIX.md) · [`../../README.md`](../../README.md) · vault [[cluster-G-bank-select-dispatch-verify]]
