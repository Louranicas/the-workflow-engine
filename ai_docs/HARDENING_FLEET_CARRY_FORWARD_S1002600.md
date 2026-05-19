# Hardening Fleet — Carry-Forward Register

> **Source:** 5-scout Δ→Φ→Ψ morphogenic fleet pass dispatched 2026-05-20.
> Mission §8: no unnecessary carry-forward — every deferred item carries a
> concrete completion plan + the agent's reasoning for not closing it
> in-session.
> **Latest update:** 2026-05-20 post-Wave-A + Wave-B (8 HIGH + 7 integration test files closed; 1178 tests passing).
> **Loop entry point:** `/carry-forward` slash command (`.claude/commands/carry-forward.md`).

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
| Wave B2 (m13 + m40 + m41 integration) | `c4bfed4` | **1178** | +23 |

**Total session delta: +98 tests, +9 commits, 4-stage gate green throughout.**

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
| **H8 (partial)** | tests/ | CC-4 and CC-6 cross-cluster integration suites authored (`tests/cc4_proposal_to_dispatch_pipeline.rs` 6 tests, `tests/cc6_verifier_gated_dispatch.rs` 6 tests). CC-1, CC-2 (partial via existing m13↔m9), CC-3 (const-locked, no functional test), CC-5, CC-7 still missing — see Tier 2 carry-forward below. |
| **H9 (partial — 5 of 18 modules)** | tests/ | Integration test files added: `m30_integration.rs` (10 tests), `m32_integration.rs` (6 tests), `m13_integration.rs` (9 tests), `m40_integration.rs` (7 tests), `m41_integration.rs` (7 tests). 13 module integration files still missing — see Tier 2 carry-forward. |

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
- **Tier 2 HIGH:** 9 / 11 closed (82%) — H5 needs Zen, H8/H9 multi-session lift partial-closed
- **Tier 3 SPEC DRIFT:** 0 / 12 fixed; 12 / 12 filed to Zen (100% of charter-allowed action)
- **Tier 4 MED/LOW:** 0 / ~22 (next-session candidates, especially T4-SERDE which unblocks m11 telemetry downstream)
- **Cross-cluster integration tests:** 2 / 7 (CC-4 + CC-6); 4 remaining + H5-dependent CC-7
- **Module integration files:** 8 / 26 modules (m1/m2/m3/m8/m9/m10/m11/m13/m20/m30/m32/m40/m41 — partial overlap)

## Next-session entry point

```bash
cd /home/louranicas/claude-code-workspace/the-workflow-engine
git log --oneline -10
cat ai_docs/HARDENING_FLEET_CARRY_FORWARD_S1002600.md  # this file
# Run /carry-forward to resume the loop.
```

**First target candidates:** T4-SERDE (small, unblocks downstream telemetry); H8-rem CC-1/CC-2/CC-3/CC-5 integration tests; H9-rem m11/m14/m23/m33/m42 integration files. After Zen replies on V4 audit, Tier 3 items become actionable.

— Hardening Fleet Δ Φ Ψ Α Σ · scout pass S1002600 + Wave A + Wave B · 1080 → 1178 tests · 9 commits · zero clippy/pedantic warnings · Mission §8 satisfied.
