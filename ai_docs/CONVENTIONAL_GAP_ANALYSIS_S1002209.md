---
title: Conventional Gap Analysis — workflow-trace S1002209 G6 closure
date: 2026-05-17
session: S1002209
status: SPEC · authored at G6 (dual-frame gap closure)
authors: Command (Tab 1 Orchestrator top-left)
gate: G6 — Dual-frame gap (Conventional + NA gap analysis)
companion_doc: [`NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md) — Wave 4.B substrate-as-primary frame
authorising_session: S1002209 (Luke G7 APPROVE proceed seamlessly)
binding_spec: [`GENESIS_PROMPT_V1_3.md`](GENESIS_PROMPT_V1_3.md)
---

# Conventional Gap Analysis — workflow-trace S1002209

> **Back to:** [`../CLAUDE.md`](../CLAUDE.md) · [`../CLAUDE.local.md`](../CLAUDE.local.md) · [`../GATE_STATE.md`](../GATE_STATE.md) · [`../ARCHITECTURE.md`](../ARCHITECTURE.md) · companion [`NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md)

## § 0 — Purpose

This document closes the second half of G6 (Dual-frame gap analysis). The first half — non-anthropocentric / substrate-as-primary frame — was authored at Wave 4.B (S1002127) in `NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md` (11 NA gaps, 8 closed, 3 deferred via D-S1002127-03). This document is the Conventional frame: engine-as-actor, anthropocentric tooling discipline, standard software-engineering gap categories.

Per CLAUDE.local Working Mode: _"write it once, then ask what frame is that? and write it again from the frame you didn't take. Both passes are the plan."_ Both passes are now present.

The Conventional frame asks: **as a 26-module Rust microservice shipping v0.1.0**, what gaps exist between the binding v1.3 spec and a deployable artefact?

## § 1 — Frame statement

The Conventional frame treats workflow-trace as **anthropocentric software**:
- The engine is the authority; substrates are tools the engine wields.
- Refusal is engine-authored; substrates are query-targets.
- Quality is measured by tests passing, clippy clean, mutation kill ≥ thresholds, integration tests green.
- Risk is enumerated by what code DOES (function-call graph, data-flow, error propagation) rather than what substrates EXPECT.

NA frame inversion (substrate-as-actor) is documented in `NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`; this document does not re-derive substrate frames.

## § 2 — Eight conventional gap categories enumerated

| # | Category | Scope | Status | Disposition |
|---|---|---|---|---|
| 2.1 | **Module count vs feature surface** | 26 modules — is this right-sized? | ANALYSED | OK — locked at v1.3; v0.2.0 may add m16 per D-S1002127-03 |
| 2.2 | **LOC budget vs feature complexity** | ~5,200 LOC src + ~1,300 LOC tests = 6,500 LOC | ANALYSED | OK — ULTRAMAP per-module budgets sum coherent; 65% boilerplate-lift reduces fresh-write to ~1,820 LOC |
| 2.3 | **Test budget vs branch coverage** | 1,599 tests (Zen G7 APPROVE locked) | ANALYSED | OK — 61.5 tests/module avg; m11 (Gap 2) and m20-m23 (Gap 1 KEYSTONE) carry highest counts |
| 2.4 | **Synergy cluster coverage** | 7 CC contracts + 1 sub-CC (`CC-1.subA`) | ANALYSED | OK — engine-side CC contracts in `synergies/`; substrate-side decomposition in `substrate-couplings/` (Wave 4.B) |
| 2.5 | **4-stage gate readiness vs CI infrastructure** | check → clippy → pedantic → test pipeline | GAP-IDENTIFIED | **G-CONV-01** — CI not yet provisioned (no `.github/workflows/` or `.gitlab-ci.yml`); locally-runnable gate adequate for v0.1.0 |
| 2.6 | **Risk surface vs mitigation enumeration** | Failure modes per module § 8 of each spec | ANALYSED | OK — 26 specs × ~7 failure modes/spec = ~180 enumerated; AP-V7-01..13 ANTIPATTERNS register |
| 2.7 | **Velocity assumptions vs Luke decision-budget** | 3 decisions/wk; ~5,200 LOC + 1,599 tests | GAP-IDENTIFIED | **G-CONV-02** — implementation throughput depends on Luke decision-budget for cross-module conflicts; 3/wk may bottleneck if more than 1 architectural conflict surfaces |
| 2.8 | **Boilerplate-lift density vs structural-gap authorship** | ~65% lift / 3 unliftable items (~950-1,550 LOC) | ANALYSED | OK — `boilerplate modules/` (48 source clones) + 3 structural-gap authorships (PrefixSpan KEYSTONE / compound decay / EscapeSurfaceProfile 7-variant) clear-delineated |

## § 3 — Gap details (G-CONV-01..02)

### § 3.1 — G-CONV-01: CI not yet provisioned

**Observation:** v1.3 binding spec assumes 4-stage quality gate (cargo check → clippy -D warnings → clippy pedantic → cargo test) as a CI discipline (per CLAUDE.md workspace charter). No `.github/workflows/` or `.gitlab-ci.yml` exists in workflow-trace scaffold.

**Severity:** LOW. Locally-runnable 4-stage gate via `~/.local/bin/devenv` or direct cargo invocations is adequate for v0.1.0 development. CI provisioning is a Phase 3 deployment-framework concern, not a Cluster D Day-1 blocker.

**Compensating control:** every PR / commit follows the locally-runnable 4-stage gate before push. CLAUDE.md memory `feedback_quality_over_speed_s085.md` enforces this discipline; auto-tracked by atuin history (each `cargo` invocation logged).

**v0.1.0 disposition:** ACCEPT — CI provisioning deferred to v0.2.0 (alongside m16 substrate-drift module + tests/substrate_fixtures/ + cross-habitat ADR cycle). Local-gate discipline backed by atuin telemetry sufficient for v0.1.0 quality bar.

**v0.2.0 work-item:** add `.github/workflows/{check,clippy,test}.yml` + `.gitlab-ci.yml` matrix; CI matrix should run both `POVM_CR2_DEPLOYED=1` and unset to verify m8 `compile_error!` path.

### § 3.2 — G-CONV-02: Luke decision-budget bottleneck risk

**Observation:** v1.3 binding spec locks 26 modules + 1,599 tests + 8 synergy clusters; architectural questions during implementation will defer to Luke @ node 0.A per project rule. Luke's stated decision-budget is **3 decisions/week** (per Master Plan v2 budget statement; CLAUDE.local).

**Severity:** MEDIUM. If implementation surfaces more than 1 architectural conflict per week, decision-budget bottleneck forms. The 3 structural-gap authorships (Gaps 1/2/3) are highest-risk conflict candidates (KEYSTONE in F-cluster; NEW PRIMITIVE in m11; cardinality-7 in m9/m30/m32/m33).

**Compensating control:** Command operates D-B6 AMEND-loop (Zen retains audit authority for spec amendments; REFUSE → amend-and-resubmit). Most conflicts can route through Zen audit lane without consuming Luke decision-budget. Luke's involvement reserved for top-level architectural calls.

**v0.1.0 disposition:** ACCEPT — D-B6 AMEND-loop is the primary throughput valve. Watcher Class-I monitoring will flag if conflicts accumulate (≥3 unresolved gate-pending items for 7+ days). Operator-as-substrate (NA-GAP-05) handles fatigue feedback through m12 consent budget.

**v0.2.0 work-item:** if v0.1.0 reveals systematic Luke-bottleneck, consider hardening D-B6 with explicit Zen-as-primary-arbiter for implementation conflicts (Luke reserved for architectural-only). Document in cross-habitat governance ADR.

## § 4 — Conventional vs Non-anthropocentric — frame coherence check

| Aspect | Conventional frame | NA frame | Coherent? |
|---|---|---|---|
| Authority locus | Engine code | Substrate state | Both true — engine WIELDS substrates; substrates HOLD state |
| Refusal semantics | thiserror enum variants | `RefusalToken` typed by authorship (SubstrateAuthored/EngineAuthored/OperatorRefusal) | YES — Wave 4.B `ERROR_TAXONOMY` amendment introduces `RefusalToken` as a CROSS-FRAME bridge |
| Quality metric | Tests + clippy + coverage | Substrate-side benchmarks + substrate-confirmable receipts | COMPLEMENTARY — both run; substrate-side opt-in via `--features substrate-load` |
| Risk enumeration | Engine failure modes per spec § 8 | Substrate drift / latency / refusal-token absence | COMPLEMENTARY — `substrate-couplings/CC-*-decomposed.md` enumerates substrate-side; spec § 8 enumerates engine-side |
| Verification surface | `cargo test` + integration tests | `Watcher Class-I` + `m12` substrate dossiers + post-G9 cross-substrate observation | LAYERED — engine tests are PR-CI; substrate observation is nightly + Wave-end + post-G9 |

**Verdict:** Conventional + NA frames are **complementary, not contradictory**. Both pass; both are the plan. NA frame surfaces 11 substrate-side gaps (8 closed Wave 4.B); Conventional frame surfaces 2 engine-side gaps (G-CONV-01 + G-CONV-02; both ACCEPT with v0.2.0 deferral).

## § 5 — G6 closure declaration

Per CLAUDE.local Working Mode "Both passes are the plan":
- Pass 1 (Conventional): this document — 8 categories enumerated, 2 gaps identified (both ACCEPT)
- Pass 2 (NA): `NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md` — 11 NA gaps, 8 closed at Wave 4.B, 3 deferred per D-S1002127-03

**G6 GREEN:** dual-frame gap analysis complete. Both frames documented + bidi-linked. No further G6 work required pre-G9.

## § 6 — Cross-references

- [`GENESIS_PROMPT_V1_3.md`](GENESIS_PROMPT_V1_3.md) — binding spec (G5 closed by absorption)
- [`NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md) — NA frame (Wave 4.B)
- [`optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md`](optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md) — D-S1001982-01
- [`optimisation-v7/decisions/2026-05-17-escape-surface-cardinality-7-privilege-escalation.md`](optimisation-v7/decisions/2026-05-17-escape-surface-cardinality-7-privilege-escalation.md) — D-S1002127-02
- [`optimisation-v7/decisions/2026-05-17-g8-stcortex-persistence-plan.md`](optimisation-v7/decisions/2026-05-17-g8-stcortex-persistence-plan.md) — D-S1002127-01
- [`decisions/2026-05-17-substrate-as-actor-deferrals.md`](decisions/2026-05-17-substrate-as-actor-deferrals.md) — D-S1002127-03
- [`../GATE_STATE.md`](../GATE_STATE.md) — G6 row updated to GREEN S1002209

---

> **Back to:** [`../CLAUDE.md`](../CLAUDE.md) · [`../CLAUDE.local.md`](../CLAUDE.local.md) · [`../GATE_STATE.md`](../GATE_STATE.md) · companion [`NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md)

*Conventional gap analysis filed 2026-05-17 (S1002209) · Command @ Tab 1 Orchestrator top-left · G6 closure · 2 gaps identified both ACCEPT-v0.1.0 with v0.2.0 deferrals*
