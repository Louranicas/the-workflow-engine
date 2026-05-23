# Hardening Fleet — Carry-Forward Register

> **⚠ SUPERSEDED — 2026-05-23 (S1004115, Plan v2 Phase 1, DOC-2).**
> This file is the 2026-05-20 scout-pass + Wave A–D snapshot. It is preserved
> as historical record only. The **authoritative current state** is:
> - Hardening Fleet W1–W5 + S1003733 remediation + C22 binary wiring →
>   [`HARDENING_FLEET_2026-05-21.md`](HARDENING_FLEET_2026-05-21.md)
> - True residual list at HEAD `968540e` →
>   [`PHASE1_RESIDUAL_LIST_S1004115.md`](PHASE1_RESIDUAL_LIST_S1004115.md)
> - Completion Plan v2 (decisions locked) →
>   [`WORKFLOW_TRACE_COMPLETION_PLAN_V2_S1004115.md`](WORKFLOW_TRACE_COMPLETION_PLAN_V2_S1004115.md)
> - Session state →
>   [`../CLAUDE.local.md`](../CLAUDE.local.md)
>
> Test count then: 1262. Test count now: **1967**. Mutation kill-rate then:
> 94.9 % (m9/m11/m20 only, 118 mutants). Mutation kill-rate now: **96.3 %**
> (324 mutants, full S1003733 + Wave-G scope). Items closed since this
> snapshot are catalogued in `PHASE1_RESIDUAL_LIST_S1004115.md` § "Closed-since-S1002600".
> Open items are the M0 work surface in Plan v2 Phases 1–10. Do not use this
> file as a fix-target list — use the residual list.

> **Source:** 5-scout Δ→Φ→Ψ morphogenic fleet pass dispatched 2026-05-20.
> Mission §8: no unnecessary carry-forward — every deferred item carries a
> concrete completion plan + the agent's reasoning for not closing it
> in-session.
> **Latest update:** 2026-05-20 post-Wave-A + Wave-B + Wave-C (9 HIGH + T4-SERDE + 14 integration test files closed; 1262 tests passing). Two cross-pane Zen verdicts received: V2 LCM 1A closed by peer (`795e4890`, re-gate requested); V1 Restraint v1.1 BLOCKED-PENDING-LUKE.
> **Loop entry point:** `/carry-forward` slash command (`.claude/commands/carry-forward.md`).

## Zen God-Tier Quality Assessment — 2026-05-21

Full report: `~/projects/shared-context/quality-assessments/2026-05-21_zen_god_tier_assessment_workflow_trace_lcm.md`. Assessed at HEAD `1535df2`.

**workflow-trace: 87/100** — "top-quartile-to-top-1% library implementation that is not yet an end-to-end product." Library-only ≈ 90; deployed-product-readiness-only ≈ 76. Facets: Rust 91 · logic 82 · anti-pattern 84 · tests 86 · security 88 · **ops 74** · arch/docs 88. Verdict: APPROVE as library / HOLD as product.

The single biggest score lever is the **ops facet 74** — the `wf-crystallise`/`wf-dispatch` stubs. Zen's 5 ranked improvements, mapped to this register:

| # | Zen recommendation | Register mapping | Type |
|---|---|---|---|
| 1 | Wire `wf-crystallise` + `wf-dispatch` into real library flows | **NOT a register item — feature-build, needs Luke scope decision** (Day-1-stub is current charter doctrine) | Feature |
| 2 | Resolve V4 spec/code drift, esp. m20-m23 KEYSTONE | Tier 3 SD1-SD12 — PENDING Zen V4 verdict | Blocked |
| 3 | Close integration gaps m12/m21/m22/m31 + CC-2/CC-5/CC-7 | H9-rem + H8-rem — **actionable now (CC-7 H5-blocked)** | Hardening |
| 4 | Remove/vendor-normalise absolute `spacetimedb-sdk` path dep | T4-PORT — actionable now | Hardening |
| 5 | Run `cargo-mutants` on m20-m23 + trust/boundary modules | NEW item T5-MUTANTS — actionable now; highest-signal metric | Measurement |

## Wave-D (2026-05-21) — pure-hardening, Luke-elected (no new scope)

| Item | Resolution |
|---|---|
| **H9-rem CLOSED** | m12/m21/m22/m31 integration files authored (Wave-D1, `f8ab952`). **26/26 module integration coverage complete.** +35 tests. |
| **H8-rem partial** | CC-2 Trust Layer + CC-5 Substrate Cycle suites authored (Wave-D2, `e7edc0c`). **6/7 cross-cluster synergies** — only CC-7 remains (H5-blocked, needs Zen). +13 tests. |
| **T4-PORT CLOSED** | `spacetimedb-sdk` dep `path` absolute → relative `../spacetimedb/sdks/rust` (Wave-D2). Resolves clean in main tree (siblings under workspace root). Note: `spacetimedb/` is an independent nested repo — relative path is canonical-layout-correct; worktree checkouts need a local symlink. |
| **T5-MUTANTS done** | `cargo-mutants` run on m9/m11/m20 (Wave-D3, `06e8c51`). Report: `ai_docs/MUTATION_TEST_REPORT_S1002600.md`. **Overall 94.9% kill rate** (112/118 viable) — mutation-STRONG, clears god-tier >70% bar by 23+ pts. m9 100%, m20 95.8%, m11 93.7%. m21/m22/m23/m10/m13 SKIPPED (time budget — cargo-mutants debug-rebuilds per mutant; documented honestly). |

## NEW — T5-MUTANTS-FOLLOWUP (from the mutation report)

6 survived mutants; 4 are scope artefacts (m11/consolidation.rs:211 ms→days divisor — covered by `tests/m11_integration.rs`, excluded only by the `--lib` test scope). **2 genuine test-gaps:**

| ID | File:line | Mutation survived | Fix |
|---|---|---|---|
| MUT-1 | `m11/inputs.rs:30` | `\|\|`→`&&` in `recency_factor` half-life guard | Add unit test: finite / zero / negative `half_life_days` each produce the correct branch. |
| MUT-2 | `m20/mod.rs:251` | `==`→`!=` in `project_after_prefix` | Add unit test: gap-restart branch with a sequence that exercises the `==` predicate directly. |

Both are ~1 unit test each. Queued for next `/carry-forward` iteration (or fold into a Wave-E micro-pass). Also recommend a scoped re-run of cargo-mutants on m21/m22/m23/m10/m13 (the skipped modules) when time budget allows.

## Session-over-session test count trajectory

| Session checkpoint | Commit | Tests | Δ |
|---|---|---|---|
| Pre-hardening baseline | `9db534d` | 1080 | — |
| C1 closed | `e7c8543` | 1090 | +10 |
| Wave A1 (C2 + H1) | `264c980` | 1096 | +6 |
| Wave A2 (C3 + H6) | `00fa576` | 1097 | +1 (over A1 in worktree) |
| Wave A3 (C4 + H3 + H4) | `641b51e` | 1103 | +6 (over A2 in worktree) |
| Wave A4 (H2 + H7) | `4d6e599` | 1127 | +24 cumulative through Wave A |
| Wave B1 (CC-4 + CC-6 + m30 + m32 integration) | `1c9b809` | 1155 | +28 |
| Wave B2 (m13 + m40 + m41 integration) | `c4bfed4` | 1178 | +23 |
| Wave C1 (T4-SERDE m11 + m11_integration) | `bac98b8` | 1192 | +14 |
| Wave C2 (m4 + m5 + m6 + m7 + CC-1) | (cherry-picked through `711a662`) | 1227 | +35 |
| Wave C3 (m14 + m23 + m33 + m42 + CC-3) | `711a662` | **1262** | +35 |

**Total session delta: +182 tests, +13 commits on workflow-trace main, 4-stage gate green throughout.**

## Cross-pane Zen verdicts (received 2026-05-20)

| Verdict | Codebase | Roll-up | Action |
|---|---|---|---|
| **V1 Restraint v1.1-candidate** (`2026-05-20T_zen_verdict_8th_trait_restraint_v1_1_candidate.md`) | synthex-v2 (m47/m51 + FROZEN spec) | **BLOCK** | A1 AMEND (stale 7-trait refs); A2 PASS-WITH-AMEND; A3 BLOCK (B1 m47 Restraint-collapses-into-Diligence; B2 7-trait overlap broken). **Tier-3 BLOCKED-PENDING-LUKE** — FROZEN/AP27/PBFT governance. Will not propose amendment shape without Luke direction. |
| **V2 LCM Sub-wave 1A** (`2026-05-20T0018Z_zen_audit_verdict_lcm_subwave_1a.md`) | loop-engine-v2 (m53_hook_stop) | **BLOCK→CLOSED** | Peer pane (LCM lane) landed `795e4890` at 10:25 local (23min post-verdict). Sanitiser + 13 new tests; m53 54→67 tests pass. Re-gate requested from Zen (`2026-05-20T_command_zen_v2_lcm_subwave_1a_already_landed_request_regate.md`). |
| **V4 workflow-trace drift** (12 spec items SD1-SD12) | this repo | **PENDING** | Filed 2026-05-20T08:00Z. No Zen reply yet. workflow-trace is at Day-1 stubs; no live blocker; V4 takes whatever Zen pace is appropriate. |

## Tier 1 — CRITICAL (all closed)

| ID | Resolution |
|---|---|
| **C1** | m30 `CuratedBank` now `impl LifecycleBank` — commit `e7c8543`. +10 tests. Recovery edge PrunePending→Active auto-derived via `phase_for()`. No spec amendment required. |

## Tier 2 — HIGH (closed this session unless noted)

| ID | Module | Resolution |
|---|---|---|
| **C2** | m13 | F-POVM-07 silent-zero `now_ms() -> i64` → `Option<i64>`; outbox tags `clock_unavailable: true` instead of writing `ts_ms: 0`. Commit `264c980`. +3 tests. |
| **C3** | m32 | Routing-method mismatch refusal added: `dispatch()` compares `client.dispatch_method()` vs `CONDUCTOR_DISPATCH_METHOD` const, refuses with new `RefusalReason::RoutingMethodMismatch{expected,actual}` variant. Commit `00fa576`. +3 tests. |
| **C4** | m40 | AP-V7-13 body-shape check landed: `push()` parses response body, refuses `{"error": ...}` even on HTTP 200 via new `NexusEmitError::ServerRejected{body}` variant. Commit `641b51e`. +5 tests via `wiremock` mock server. |
| **H1** | m2 | `RegistrationHandle::is_fresh()` now reads BOTH `applied_flag` AND new `disconnected_flag` (Acquire ordering); `on_disconnect` clears applied + sets disconnected. New triple-state `RegistrationStatus::{Fresh,Disconnected,Stale}` + `status()` accessor (additive). Commit `264c980`. +4 tests. |
| **H2** | m10 | Soft-absolutist Humility heuristic (confidence 0.4) shipped → `EmberStatus::Held` branch + `GateVerdict::HeldFailed`/`HeldAllowlisted` paths now reachable end-to-end. Commit `4d6e599`. +7 tests (4 rubric, 4 gate). |
| **H3** | m41 | JSON-RPC id-echo wired via `AtomicU64` per-call id allocator + response-id check; new `LcmRpcError::IdMismatch{sent,received}` variant. Commit `641b51e`. +4 tests including 100-thread concurrent id uniqueness. |
| **H4** | m41 | Error-envelope tightened: `error` field treated as error ONLY when it's an object AND has a `code` field. `null` / `{}` parse as non-errors. Commit `641b51e`. +3 tests. |
| **H6** | m31/m32/m33 | Verifier-gate wired: new `ConductorDispatcher::with_verifiers(Vec<Box<dyn Verifier>>)` builder; dispatch() invokes m33 `aggregate()` BEFORE wire call; new `RefusalReason::VerifierGateBlocked{blocking_kinds}` variant. Routing-method check fires BEFORE verifier-gate (defense-in-depth ordering). Commit `00fa576`. +4 tests covering ordering invariant + empty-verifier-set backward-compat. |
| **H7** | m22 | k-means tiebreak precision fixed: `(tiebreak as f64).copysign(1.0) * 1e-12` (magnitude ~10^7, dominated `d`) → `(tiebreak % 1024) as f64 * f64::EPSILON * d.max(1.0)` (bounded ≤ d·1024·ε). Commit `4d6e599`. +3 tests including bit-identical determinism. |
| **H8 (partial)** | tests/ | **Wave-C extension:** CC-4 and CC-6 (Wave-B1) + CC-1 cascade-cost coupling (Wave-C2) + CC-3 evidence iteration (Wave-C3). Remaining: CC-2, CC-5, CC-7. |
| **H9 (partial — 14 of 26 modules)** | tests/ | **Wave-C extension:** m11/m4/m5/m6/m7/m14/m23/m33/m42 added on top of Wave-B's m13/m30/m32/m40/m41. **Remaining 4 modules:** m12, m21, m22, m31. (m8/m9/m10/m20 had pre-existing integration files from Wave-1.) |
| **T4-SERDE** | m11 | Wave-C1 — `SunsetStats` + `SunsetPhase` + `AcceptedWorkflowDecay` derive `serde::{Serialize, Deserialize}`. Custom `json_safe_float` adapter maps INFINITY/-INFINITY/NaN sentinels to `"+inf"/"-inf"/"NaN"` JSON strings (preserves public API + lossless round-trip). Commit `bac98b8`. +14 tests including the cross-target invariant locking T4-SERDE to the m11_integration cycle test. |

## Tier 2 — HIGH carry-forward (open)

| ID | Module | Type | Concrete plan |
|---|---|---|---|
| **H5** | m15 | CC-7 dead edge — architectural decision required | `PressureEvent` has zero downstream consumers in m20-m23. Either (a) wire `PressureEvent` into m23's proposal-gating, or (b) document explicitly that CC-7 is observability-only at Day-1. **Zen G7 decision** (filed in `2026-05-20T080000Z_command_zen_audit_request_v4_post_wave1_drift.md`). Do not fix unilaterally. |
| **H8-rem** | tests/ | Missing cross-cluster integration tests | Author: `tests/cc1_cascade_cost.rs` (B-internal m4↔m5↔m6↔m7 join), `tests/cc2_trust_layer.rs` (m9 → all writers full sweep), `tests/cc3_evidence_iteration.rs` (m14↔m23 const + functional), `tests/cc5_substrate_cycle.rs` (G→H→F via stcortex pathways), `tests/cc7_pressure_evolution.rs` (depends on H5 architectural decision). ~150 LOC each = ~600 LOC across 4 files. |
| **H9-rem** | tests/ | Missing module integration files (13) | Author per-module integration tests for: m1+m2+m3 already have files; **missing:** m4, m5, m6, m7, m8 (has live-POVM probe but no cross-module), m9 (has integration already? verify), m11, m12, m14, m15, m20 (partial), m21, m22, m23, m31, m33, m42. Priority by criticality: **m11** (decay cycle correctness), **m14** (lift Wilson CI boundaries), **m23** (proposer F2 gate), **m33** (verifier ordering), **m42** (stcortex-only AP30 transitive). ~150 LOC × 13 = ~2000 LOC. Multi-session. |

## Tier 3 — SPEC DRIFT (filed to Zen G7 audit — NOT for unilateral fix)

Filing: `~/projects/shared-context/agent-cross-talk/2026-05-20T080000Z_command_zen_audit_request_v4_post_wave1_drift.md`

| ID | Status | Notes |
|---|---|---|
| SD1 m9 ControlChar | FILED | Class A (code ahead of spec). Recommend AMEND spec § 4. |
| SD2 m14 LiftError taxonomy | FILED | Class B (name + variant divergence). |
| SD3 m14 cost_lift Result return | FILED | Class A. AMEND spec § 5. |
| SD4 m14 window-eviction direction | FILED | Class A. AMEND spec § 6. |
| SD5 m15 CharterSection variant names | FILED | Class B. Decision needed. |
| SD6 m15 detected_at_ms field | FILED | Class A. AMEND spec § 4. |
| SD7 m15 pseudo_rfc3339 wire-format | FILED | Class B. Real fix OR amend spec. |
| SD8 m21 Levenshtein vs swap/skip | FILED | Class C (KEYSTONE algorithmic divergence). Recommend ship v0.1.0 + v0.2.0 fold-in. |
| SD9 m22 generic kmeans vs spec FeatureVector | FILED | Class C. Same recommendation. |
| SD10 m22 empty-cluster retain-prior | FILED | Class C/A hybrid. Document chosen behaviour. |
| SD11 m23 5-field vs spec 12-field proposal | FILED | Class C. Same recommendation. |
| SD12 m20 stabilization gate absent | FILED | Class B/load-bearing. Recommend WIRE the gate in Wave-C. |

## Tier 4 — MEDIUM/LOW (defer queue)

| ID | Type | Status | Notes |
|---|---|---|---|
| T4-DEAD-ERR | Dead error variants (~15) | OPEN | m1 `BusyTimeout`; m2 `UnregisterFailed`; m4 entire `CascadeError`; m5 `BatternError::AtuinIo`; m6 entire `ContextCostError`; m8 `RuntimeBandError::StartupRefused`; m10 `EmberGateError::GateFailed`/`RubricMissing`; m11 `DecayError::CycleAborted`; m32 `DispatcherError::WireFormat` (only variant, collapses transport/timeout/serde). Decision: keep + test, or drop with `#[deprecated]`. Needs cross-module audit. |
| T4-SERDE | m11 SunsetStats not Serialize/Deserialize | OPEN | New Wave-1 fields `workflows_prune_pending`, `workflows_clock_skew_skipped` have zero downstream readers. m12 reports + Prometheus metrics promised in spec § 9 unwired. ~30 LOC + 2 tests. |
| T4-MISC | Misc | OPEN | m32 `HumanAcceptanceSignature.interactive_terminal` field never read; m6 poison-mutex silent no-op; m20 max_length=0 silent-coerce; m22 no CHAIN-PROOF convergence test; m9 promised AP30 coarse-grep regression test in `tests/m9_integration.rs` doesn't exist; m10 allowlist loaded once; m10 rubric path absolute home-dir. |
| T4-PORT | Portability | OPEN | `Cargo.toml` `spacetimedb-sdk = { path = "/home/louranicas/..." }` absolute; cosmetic features `api/intelligence/monitoring/evolution/substrate-load` declared but never `cfg`-gated; `build.rs` `povm_calibrated` cfg flag never consumed in `src/`. |
| T4-AP30 | m42 source-grep anti-property scope | OPEN | Doesn't scan child modules; doesn't catch literal port `8125` or `:8125` URL fragment. ~10 LOC test extension. |
| T4-LIB | Public surface | OPEN | m32 `self_dispatch_guard` is pub but not re-exported from `lib.rs`; m11 `chrono_now_ms` pub but unused outside module. |
| T4-API | Test seams flagged by Wave B agents | OPEN | (1) `ConductorDispatcher` exposes no `client_ref()` accessor; tests work around with Arc<Mutex<...>> sidecar pattern. (2) `CuratedBank::with_workflow_for_test` factory absent. (3) `AcceptedWorkflow::last_run_ms` direct setter absent (current `record_run` increments `run_count` as side-effect). (4) m13 has no public `Clock` trait for clock-injection test seam. |

## Loop protocol

`/carry-forward` slash command at `.claude/commands/carry-forward.md` is the entry point. Procedure:

1. Read THIS register (`HARDENING_FLEET_CARRY_FORWARD_S1002600.md`).
2. FP-verify each open item still exists at current HEAD before dispatching fix labor.
3. Dispatch parallel morphogenic agents (Δ→Φ→Ψ→Α→Σ) for coherent target clusters.
4. Worktree-isolate each agent; gate centrally on integration; commit + push per wave.
5. Update this register: move closed items to their Tier section's closed-table; append any NEW findings.
6. If unclosed items remain and context budget allows, repeat. Otherwise `ScheduleWakeup` next iteration.

## Closed-set summary (this session)

- **Tier 1 CRITICAL:** 1 / 1 (100%)
- **Tier 2 HIGH:** 9 / 11 closed (82%) — H5 BLOCKED-PENDING-LUKE/Zen, H8/H9 multi-session lift partial-closed
- **Tier 3 SPEC DRIFT:** 0 / 12 fixed; 12 / 12 filed to Zen V4 (PENDING Zen reply, no live blocker)
- **Tier 4 MED/LOW:** 1 / ~22 closed (T4-SERDE)
- **Cross-cluster integration tests:** 4 / 7 (CC-1 + CC-3 + CC-4 + CC-6); 3 remaining (CC-2, CC-5, CC-7 H5-dependent)
- **Module integration files:** 18 / 26 modules — only m12/m21/m22/m31 remain
- **Cross-pane Zen verdicts:** V1 BLOCKED-PENDING-LUKE (FROZEN-spec governance); V2 CLOSED by peer; V4 PENDING Zen reply

## Next-session entry point

```bash
cd /home/louranicas/claude-code-workspace/the-workflow-engine
git log --oneline -10
cat ai_docs/HARDENING_FLEET_CARRY_FORWARD_S1002600.md  # this file
# Run /carry-forward to resume the loop.
```

**First target candidates:** T4-SERDE (small, unblocks downstream telemetry); H8-rem CC-1/CC-2/CC-3/CC-5 integration tests; H9-rem m11/m14/m23/m33/m42 integration files. After Zen replies on V4 audit, Tier 3 items become actionable.

— Hardening Fleet Δ Φ Ψ Α Σ · scout pass S1002600 + Wave A + Wave B · 1080 → 1178 tests · 9 commits · zero clippy/pedantic warnings · Mission §8 satisfied.
