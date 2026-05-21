# Hardening Fleet W2 — Security Finding Ledger

> Workflow-trace security hardening wave. Baseline commit `dc25335`.
> Sources: W1 batch agents + W2 silent-failure-hunter + W2 OWASP/Rust audit + W2 zen deep-review.
> Back to: [HARDENING_FLEET_2026-05-21.md](HARDENING_FLEET_2026-05-21.md)

## Known (carried from W1)

| ID | Location | Sev | Issue | Status |
|----|----------|-----|-------|--------|
| K1 | `m20_prefixspan/mod.rs:251` | HIGH | Over-gap restart branch is dead code for ordinary prefixes — under-counts KEYSTONE pattern support when first token recurs after a wide gap. | OPEN |
| K2 | `m30_bank` ×8 + `m15_pressure` ×1 `lock().expect()` | MED | Mutex poison → panic. Convert to `PoisonError::into_inner` / typed `BankError`. | OPEN |
| K3 | `RUSTSEC-2024-0436` (`paste` unmaintained, transitive via spacetimedb-sdk) | LOW | — | ✅ DONE — `.cargo/audit.toml` ignore + rationale; `cargo audit` clean. |

## Silent-failure sweep (6)

| ID | Location | Sev | Issue |
|----|----------|-----|-------|
| SF1 | `m12_cli_reports/mod.rs:221` | HIGH | `to_string_pretty(runs).unwrap_or_else(\|_\|"[]")` — NaN fitness collapses the whole JSON report to `[]`, indistinguishable from empty DB. |
| SF2 | `m12_cli_reports/mod.rs:227` | HIGH | `.filter_map(\|r\| to_string(r).ok())` — NdJson silently drops un-serializable rows, no count/warn. |
| SF3 | `m4_cascade/mod.rs:250` | LOW | `now_ms()` silent-zeros to epoch 0 (1970) on clock fault — m4 missed by the m11/m13 hardening. |
| SF4 | `m2_stcortex_consumer/subscription.rs:281` | MED | `if let Ok(slot)=lock()` no-else — poison skips storing the register-error → misleading "fresh" handle. |
| SF5 | `m2_stcortex_consumer/subscription.rs:359` | MED | symmetric to SF4 — poison skips the error check → false `Ok(RegistrationHandle)`. |
| SF6 | `m2_stcortex_consumer/subscription.rs:289` | MED | `tx.lock().ok()` discards poison → readiness signal never fires → masked as `SubscriptionTimeout`. |

## OWASP / Rust audit (6)

| ID | Location | Sev | Issue |
|----|----------|-----|-------|
| SEC1 | `m13_stcortex_writer/mod.rs:264-310` | MED | `promote_run` never checks `RegistrationHandle::is_fresh()` despite docstrings; `DeferReason::StcortexUnreachable` is dead code — refuse-write-on-stale invariant unenforced. |
| SEC2 | `m2_stcortex_consumer/subscription.rs:45-52` | MED | `tool_call_query` strips `'` but not LIKE metachars `%` `_` `\`; m9 validator permits `%`. |
| SEC3 | `m2_stcortex_consumer/identity.rs:156` | LOW | `Command::new("git")` resolved via `$PATH` — PATH-hijack (identity-string only). |
| SEC4 | `m40:99` (+ `m13`, `m8`, `m41` HTTP clients) | LOW | `resp.text()`/`.json()` with no response-body size cap — unbounded `String` alloc. |
| SEC5 | `m13_stcortex_writer/mod.rs:200-209` | LOW | `body.as_f64()` density accepted with no `[0,1]`/finite range check → can defeat the 3-band LTP gate. |
| SEC6 | `m2_stcortex_consumer/module_bindings/` | INFO | Generated SDK deserialization surface is `#[allow]`-blanketed — unreviewed. Pin SDK; treat regen as security event. |

## Zen deep-review (KEYSTONE / trust / dispatch) — 9 findings

| ID | Location | Sev | Issue |
|----|----------|-----|-------|
| F1 | `m20_prefixspan/mod.rs:251` | HIGH | (= K1) KEYSTONE over-gap restart dead code — confirmed in depth; detector computes a *lower bound* on support, under-count compounds with pattern depth. |
| F2 | `m8_povm_build_prereq/` (whole module) | HIGH | Trust gate is doc-only — compile-time `#[cfg(povm_calibrated)]`/`compile_error!` tombstones AND runtime startup-refusal both unimplemented. `StartupRefused`/`OutOfBand` constructed only in tests. |
| F3 | `m9_watcher_namespace_guard/validator.rs:127` | HIGH | `starts_with("workflow_trace")` accepts `workflow_traceXYZ` (no separator) — namespace boundary leak. Fix: `== PREFIX \|\| starts_with("{PREFIX}_")`. |
| F4 | `m11 consolidation.rs:40-41` vs `m30_bank/mod.rs:43,47` | MED | Divergent lifecycle threshold constants for the same 3-phase state machine — m11 telemetry under-reports vs m30 eviction. Fix: single source of truth. |
| F5 | `m21_variant_builder/mod.rs:79-112` | MED | `MAX_VARIANTS_PER_PATTERN=8` cap starves all `Skip` variants for patterns ≥8 steps — narrows m31 diversity input. Fix: interleave emission / per-class budget. |
| F6 | `m22_kmeans/mod.rs:267-268` | MED | k-means++ tiebreak bias added to all candidates, scaled by `d` — perturbs farthest-point selection rather than only breaking exact ties. |
| F7 | `m31_selector/mod.rs:216` vs `m11 inputs.rs:27` | LOW | Never-run workflow recency = 0.5 (m31) vs 1.0 (m11) — divergent neutral value (may be context-intentional; align or document). |
| F8 | `m30_bank` ×8 + `m15_pressure` ×1 | LOW | (= K2) 9× `lock().expect()` poison panics. |
| F9 | `m23_proposer/mod.rs:122-144` | INFO | `compose_proposals` `EmptyPattern` arm is unreachable dead handling. |

Zen verdict: KEYSTONE **REQUEST-CHANGES** (F1); trust spine **REQUEST-CHANGES, BLOCK on m8** (F2).
Zen "done well": m11 formula (FMA + 10 proptests), m32 dispatcher refusal sequence, m33 aggregate
duplicate-before-missing ordering, consistent NaN/non-finite discipline, m9 NUL/BOM rejection.

## Execution & progress

**Command-applied + verified (committed `c662b2d` / in flight):**
- ✅ **K1/F1** — m20 KEYSTONE `project_after_prefix` rewritten as a correct backtracking
  matcher; m20 61 tests pass. (`c662b2d`)
- ✅ **K2/F8** — 9 lock-poison panics → `PoisonError::into_inner`. (`c662b2d`)
- ✅ **K3** — `.cargo/audit.toml`; `cargo audit` clean. (`c662b2d`)
- ✅ **F2** — m8 trust-gate false docstrings corrected (`mod.rs` + `error.rs`); gate honestly
  documented as dormant-by-construction post-m42-pivot. **ARCHITECTURE FLAG → node 0.A:** m8's
  POVM gate has nothing to protect (workflow-trace is POVM-decoupled); keep-dormant / wire /
  retire is an open decision, NOT a hardening fix.

**Dispatched to 5 disjoint-file parallel agents (incident-proofed):**
- SF4/SF5/SF6/SEC2/SEC3 → m2 · SF1/SF2/SF3 → m12+m4 · SEC1/SEC5/SEC4 → m13+m40+m41 ·
  F5/F6/F9/F7 → m21+m22+m23+m31 · F3/F4/F7 → m9+m11.

### Final status — all 19 findings resolved

| Finding | Resolution |
|---------|-----------|
| K1/F1 | FIXED — m20 backtracking matcher (`c662b2d`) |
| K2/F8 | FIXED — 9 lock-poison → `into_inner` (`c662b2d`) |
| K3 | FIXED — `.cargo/audit.toml` (`c662b2d`) |
| F2 | FIXED — m8 doc-honesty; gate dormant-by-construction; **architecture flag → node 0.A** |
| SF1/SF2 | FIXED (code = defense-in-depth) — recon premise was a **false positive** (`serde_json` renders NaN as `null`, never errors); 3 tests rewritten against true behaviour |
| SF3 | FIXED — m4 `now_ms` → `Option<i64>` |
| SF4/SF5/SF6 | FIXED — m2 subscription poison-recovery |
| SEC1 | FIXED — m13 `FreshnessGate` machinery + `promote_run` enforcement; `DeferReason::StcortexUnreachable` now live. Binary-wiring is future work (binaries are Day-1 stubs; no production `StcortexWriter` construction site exists yet) |
| SEC2 | FIXED — m2 `tool_call_query` → allowlist-reject `Result` |
| SEC3 | FIXED — m2 git absolute-path resolution (PATH-hijack removed) |
| SEC4 | FIXED — 1 MiB capped HTTP body reads (m13/m40/m41) |
| SEC5 | FIXED — m13 density finite/range validation |
| F3 | FIXED — m9 namespace boundary (`== PREFIX \|\| starts_with("PREFIX_")`) |
| F4 | FIXED — m11 thresholds single-source from m30 |
| F5 | FIXED — m21 round-robin variant emission |
| F6 | FIXED — m22 exact-tie-only k-means++ tiebreak |
| F7 | NOT A BUG — m11/m31 recency divergence judged intentional by 2 independent agents; documented |
| F9 | FIXED — m23 `debug_assert` guard on the unreachable arm |

W2 reconciliation also fixed 3 sibling integration tests (`tests/m2_integration.rs` ×4,
`tests/m30_integration.rs`, `tests/m11_integration.rs`) for the F4/SEC2 API+threshold changes.
Gate: check + clippy + pedantic green; final test run in progress. Zen audit packet on close.

## Fix plan

Applied sequentially by Command (no parallel writers). Batched by file:
- `m20_prefixspan` — K1
- `m30_bank` + `m15_pressure` — K2
- `m12_cli_reports` — SF1, SF2
- `m4_cascade` — SF3
- `m2_stcortex_consumer/subscription.rs` — SF4, SF5, SF6, SEC2
- `m2_stcortex_consumer/identity.rs` — SEC3
- `m13_stcortex_writer` — SEC1, SEC5
- HTTP body caps — SEC4 (m40/m13/m8/m41)
- SEC6 — no code change; documented.
- + zen findings.

Each fix FP-verified against source first; full gate after; Zen audit packet on W2 close.
