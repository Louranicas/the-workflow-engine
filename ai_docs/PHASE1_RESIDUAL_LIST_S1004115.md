# Phase 1 — True Residual List (S1004115, HEAD `968540e`)

> **Authored:** 2026-05-23 (S1004115, Phase 1)
> **Status:** authoritative successor to [`HARDENING_FLEET_CARRY_FORWARD_S1002600.md`](HARDENING_FLEET_CARRY_FORWARD_S1002600.md)
> (which was a 2026-05-20 scout pass — pre-Hardening-Fleet W1–W5, pre-S1003733 remediation, pre-C22 binary wiring)
> **Method:** Plan v2 § 2.2 ⚠ items each FP-verified against live tree at `968540e`.
> **Back to:** [`WORKFLOW_TRACE_COMPLETION_PLAN_V2_S1004115.md`](WORKFLOW_TRACE_COMPLETION_PLAN_V2_S1004115.md) · [CLAUDE.local.md](../CLAUDE.local.md)

---

## Closed-since-S1002600 (FP-verified against live tree)

| ID | S1002600 state | Verified state at `968540e` | Evidence |
|----|---------------|----------------------------|----------|
| **T4-PORT** | OPEN — `spacetimedb-sdk` absolute path | **CLOSED** — `Cargo.toml:45 spacetimedb-sdk = { path = "../spacetimedb/sdks/rust" }` (relative) | `Cargo.toml:45` |
| **T4-PORT (povm_calibrated cfg)** | OPEN — flag never consumed | **CLOSED** — `build.rs:19` sets cfg; `src/m8_povm_build_prereq/mod.rs:12–13` consumes via `#[cfg(povm_calibrated)]` + tombstone | `build.rs` + m8 |
| **T4-AP30** | OPEN — m42 anti-property grep misses child modules | **CLOSED** — m42 has only `mod.rs` (no child modules). `tests/m42_integration.rs:270` `m42_no_8125_port_literal_in_m42_source` covers the full surface | `ls src/m42_stcortex_emit/` + test |
| **T4-LIB (chrono_now_ms)** | OPEN — unused outside module | **CLOSED** — `src/lib.rs:166` re-exports & consumes | `lib.rs:166` |
| **m20 max_length=0 silent-coerce** | OPEN | **CLOSED** — `src/m20_prefixspan/mod.rs:199` returns `MinerError::MaxLengthZero` | `m20_prefixspan/mod.rs:171–200` |
| **MUT-1** | OPEN — m11 `\|\|`→`&&` survivor in half-life guard | **CLOSED** — explicit NaN-half-life test path documented at `inputs.rs:215–234` (the discriminating input for `\|\|` vs `&&` mutation) | `inputs.rs:215–234` |
| **T4-SERDE** | CLOSED Wave-C1 | **CLOSED** (re-confirmed) | `CHANGELOG.md` |
| **H9-rem** | OPEN (m12/m21/m22/m31 integration files missing) | **CLOSED** — 26/26 module integration coverage complete (Wave-D1 → Hardening-Fleet) | `tests/m*_integration.rs` |
| **H8-rem CC-2 / CC-5** | OPEN | **CLOSED** Wave-D2 | `tests/cc*` |

## Genuinely-open at `968540e` (the M0 work surface)

### Decision-gated (R1 / R2)

| ID | Scope | Plan v2 phase | Decision (§ 15) |
|----|-------|---------------|-----------------|
| **R1** | m33 dispatch verifiers — 4 `ConservativeVerifier` kinds unconditionally `Approve` (`src/orchestration/dispatch.rs:311–335`) | Phase 6 (6a–6f split) | D5–D16 locked: Security hard-Refuse · Ember reduced-M0 · Cost stub · Consistency stub · 6f WireEvent receipts |
| **R2** | m22 K-means diversity never assembled on `wf-crystallise` CLI path (`src/orchestration/crystallise.rs:504–509` passes `\|_v\|None`) | Phase 5 | D17–D20 locked: 5-dim features · influences m31 · k adaptive · diversity_cluster emitted |

### Low-risk, no decision (Phase 3 work)

| ID | Scope | Evidence |
|----|-------|----------|
| **MUT-2** | `m20_prefixspan/mod.rs:322` `project_after_prefix` `==` mutation — no current test exercises the gap-restart branch directly (re: search returned no `gap_restart\|MUT-2` hits) | rg over m20_prefixspan + tests |
| **T4-LIB (self_dispatch_guard)** | `m32` `self_dispatch_guard` is `pub` but not re-exported from `src/lib.rs` (re-search returned no lib.rs hit) | `rg self_dispatch_guard src/lib.rs` |
| **T4-API** | (1) `ConductorDispatcher` exposes no `client_ref()` accessor (tests Arc<Mutex<…>>-workaround); (2) `CuratedBank::with_workflow_for_test` factory absent; (3) `AcceptedWorkflow::last_run_ms` direct setter absent; (4) m13 has no public `Clock` trait for clock-injection seam | rg over orchestration + m32 + m13 |
| **T4-MISC** (open subset, per S1002600 line 128) | m32 `HumanAcceptanceSignature.interactive_terminal` never read · m6 poison-mutex silent no-op · m22 no CHAIN-PROOF convergence test · m9 promised AP30 coarse-grep regression test in `tests/m9_integration.rs` absent · m10 allowlist loaded once · m10 rubric path absolute home-dir | inherited; verify per item when touched |

### Needs interview-locked answer in the body (already locked in § 15)

| ID | Scope | Plan v2 phase | Decision (§ 15) |
|----|-------|---------------|-----------------|
| **m9-TODO** | EscapeSurfaceProfile 7-variant capability table in m9 validator (`src/m9_watcher_namespace_guard/validator.rs:169–177`) — couples to R1 via m9↔m32 `HumanAcceptanceSignature` trait | Phase 6e | C-8 absorbed; trait defined once |
| **T4-DEAD-ERR** | ~15 dead error variants across ~9 modules | Phase 10 | **D31** keep + test (construct each) |
| **CC-7 / H5** | `PressureEvent` (m15) dead edge | Phase 7 | **D21** wire to m23 composition; D22 additive-bounded |

### Audit residuals (Zen-paced)

| ID | Status | Plan v2 phase | Decision (§ 15) |
|----|--------|---------------|-----------------|
| **ZEN-W** | Review **requests** for W1–W4 filed; **zero verdict files** for any workflow-trace wave; S1003733/C22 packets are Command-authored notices, not verdicts | Phase 9 | **D25/D26** if absent by P8 close → substitute = in-session `zen` agent (not /ultrareview) |
| **SD1–SD12** | 12 spec-drift items filed 2026-05-20T08:00Z, no Zen reply 3+ days; 8/12 Class A/B (code-ahead → spec amend); 4/12 Class C (algorithmic → v0.2.0) | Phase 9 | **D27** reconcile 8 A/B now; 4 C → v0.2.0 |

### Doc / persistence + hygiene (Phase 1 scope)

| ID | Scope | Plan v2 phase | This-phase action |
|----|-------|---------------|-------------------|
| **DOC-1** | S1003733 has no stcortex surface in ns `workflow_trace_hardening_2026_05_21` | Phase 1 | write + read-back-verify (this commit) |
| **DOC-2** | `HARDENING_FLEET_CARRY_FORWARD_S1002600.md` is stale pre-Fleet scout pass | Phase 1 | supersession banner (this commit) |
| **DOC-3** | `CHANGELOG.md:31` says "254 caught, 94.4 %"; verified figure is **"259 caught, 96.3 %"** | Phase 1 | edit (this commit) |
| **HYG-1** | Dirty tree: `M .obsidian/workspace.json` · `M` 2× Watcher journals · `D Pasted image …png` (CLAUDE.local.md's flag on `src/m30_bank/mod.rs` is itself stale — `git status -- src/m30_bank/` returns clean) | Phase 1 | discard workspace.json; commit Watcher journals; stage delete (this commit) |
| **HYG-2** | `mutants.out/` + `mutants.out.old/` (~2.8 MB stale local) — `.gitignore`d (`gitignore:53–55`) | Phase 1 | `rm -rf` local dirs (this commit) |

### Operator hand-off (cannot be agent-done)

| ID | Scope | Plan v2 phase | Decision (§ 15) |
|----|-------|---------------|-----------------|
| **OP-1 / B3** | Conductor bring-up + `CONDUCTOR_ENFORCEMENT_ENABLED` flip | Phase 10 hand-off | **D33–D35** post-M0, 24h NoOp soak then flip |
| **OP-2 / G2** | Directory rename `the-workflow-engine/` → `workflow-trace/` | Phase 10 hand-off | **D32** post-M0 cosmetic |

### Explicitly deferred to v0.2.0 (ADR D-S1002127-03)

`NA-GAP-07` substrate-drift canary · `NA-GAP-08` substrate test fixtures · `NA-GAP-10` substrate-mediated trust.

---

## Re-baseline note (Plan v2 step 1)

Plan v2 was authored against `6c3a5c5`. HEAD is now `968540e`; the 2 intervening commits
(`a32fa1e` Phase-4 interview fold; `968540e` session checkpoint S1004115) are **doc-only**.
The `6c3a5c5` commit (W5-closeout) had already reconciled the W4 test count (1924 → 1903)
and added an S1004115 reconciliation note to `HARDENING_FLEET_2026-05-21.md`, so the
"already-corrected" subtraction the plan called for amounts to: do not re-edit
`HARDENING_FLEET_2026-05-21.md`'s W4 row in this phase (the canonical numbers are already
authoritative there). `CHANGELOG.md:31` is a separate surface and still needs the
259/96.3 % edit (DOC-3, this phase).

— Phase 1 residual list, S1004115
