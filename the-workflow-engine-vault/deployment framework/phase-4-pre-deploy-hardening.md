---
title: Phase 4 — Pre-Deploy Hardening (Days 26-28)
date: 2026-05-17 (S1001982)
kind: deployment-framework-phase
status: planning-only · HOLD-v2 active
phase: 4 of N
binary-targets: wf-crystallise · wf-dispatch · workflow-core lib
days: 26-28 (2-3 calendar days)
agents: security-auditor · performance-engineer · silent-failure-hunter · zen
wave-count: 3
---

# Phase 4 — Pre-Deploy Hardening (Days 26-28)

> Back to: [[../HOME]] · [[../GOD_TIER_CONSOLIDATION_S1001982]] · [[phase-3-integration-conductor-wiring]]

This document is the canonical recipe for Phase 4 of the Ultimate Deployment Framework for `workflow-trace`. It is the 4-agent parallel pre-deploy gate that runs BEFORE binary deployment to `~/.local/bin/`. Nothing proceeds to Phase 5 without a PASS receipt from this phase.

---

## Phase Overview

Phase 4 occupies three days and three sequential waves. The ordering is strict — no agent dispatch before the mechanical gate passes, no deployment before Wave 3 Watcher witness completes.

```
Day 26 morning:   Wave 1 — Mechanical gate (full codebase; no agents)
Day 26 afternoon: Pre-flight snapshot (baseline.json)
Day 27 all day:   Wave 2 — 4-agent parallel dispatch (security / perf / silent / zen)
Day 27 evening:   Wave 2 verdict collection (boolean AND)
Day 28 morning:   Decision branch (PASS → Wave 3 · FAIL → halt · DEGRADED → Luke)
Day 28 afternoon: Wave 3 — Watcher witness + deployment-readiness receipt
```

**Why three days and not one?** The mechanical gate must pass cleanly before agents are dispatched — running four agents against broken code is wasted compute. The Watcher witness is separated from the verdict collection because it requires Watcher's own async observation cadence (prompt-driven, not agent-synchronous). Collapsing this to a single day is the same anti-pattern as phase-collapse under context pressure.

---

## Wave 1 — Mechanical Gate (Day 26)

### Pre-flight setup

Before running any gate command, establish the working directory, binary SHA, and substrate baseline. This snapshot anchors the PASS receipt later.

```bash
SESSION="predeploy-$(date +%Y%m%d-%H%M)"
WORK="/tmp/predeploy-hardening-${SESSION}"
mkdir -p "$WORK"

PROJECT_DIR="/home/louranicas/claude-code-workspace/the-workflow-engine"

# Binary SHA (pre-build anchors)
sha256sum ~/.local/bin/wf-crystallise 2>/dev/null | cut -c1-16 > "$WORK/pre-sha-crystallise.txt"
sha256sum ~/.local/bin/wf-dispatch 2>/dev/null | cut -c1-16 > "$WORK/pre-sha-dispatch.txt"

echo "Session: $SESSION" > "$WORK/SESSION"
echo "Project: $PROJECT_DIR" >> "$WORK/SESSION"
```

### 4-stage gate sequence (with PIPESTATUS)

The S1001882 near-miss established that `cargo ... | tail` makes `$?` capture `tail`'s exit code (always 0), not cargo's. The gate must use `${PIPESTATUS[0]}` per stage, with an explicit abort before proceeding.

```bash
cd "$PROJECT_DIR"
GATE_LOG="$WORK/mechanical-gate.txt"

echo "## STAGE 1: cargo check" | tee -a "$GATE_LOG"
cargo check --workspace 2>&1 | tee -a "$GATE_LOG"
[[ ${PIPESTATUS[0]} -ne 0 ]] && { echo "ABORT: cargo check failed"; exit 1; }

echo "## STAGE 2: clippy -D warnings" | tee -a "$GATE_LOG"
cargo clippy --workspace -- -D warnings 2>&1 | tee -a "$GATE_LOG"
[[ ${PIPESTATUS[0]} -ne 0 ]] && { echo "ABORT: clippy -D warnings failed"; exit 1; }

echo "## STAGE 3: clippy -W pedantic -D warnings" | tee -a "$GATE_LOG"
cargo clippy --workspace -- -D warnings -W clippy::pedantic 2>&1 | tee -a "$GATE_LOG"
[[ ${PIPESTATUS[0]} -ne 0 ]] && { echo "ABORT: pedantic gate failed"; exit 1; }

echo "## STAGE 4: test --lib --release" | tee -a "$GATE_LOG"
CARGO_TARGET_DIR=./target cargo test --lib --release --workspace 2>&1 | tee -a "$GATE_LOG"
[[ ${PIPESTATUS[0]} -ne 0 ]] && { echo "ABORT: test suite failed"; exit 1; }

echo "WAVE 1: MECHANICAL GATE PASS" >> "$GATE_LOG"
echo "Wave 1 PASS: $(date -Iseconds)" > "$WORK/wave1-pass.txt"
```

**Zero tolerance:** 0 errors and 0 warnings at every stage. Pre-existing project warnings do not excuse new code from the bar (S226 crystallisation). The gate is run on `--workspace` to cover both `wf-crystallise`, `wf-dispatch`, and `workflow-core` in a single sweep.

### Forge 8-trap audit (applied to deployment-ready binary)

The `/forge` pattern from `SKILL-forge.md` encodes 8 habitat-specific deployment traps. During Phase 4 Day 26, these traps are audited against the pre-release codebase as a static check — not yet as binary deployment steps. The audit answers: does any code path in `m32 dispatcher` violate a trap that `/forge` would catch at deploy time?

| Trap | Source | Audit target in workflow-trace |
|---|---|---|
| T1 `cp` alias (`cp -i`) | forge | m32 generates no binary-copy commands; confirm `wf-dispatch` Conductor handoff does not spawn `cp` |
| T2 `pkill` exit 144 kills `&&` chains | forge | m32 `probe_conductor()` never chains pkill in pre-dispatch gate |
| T3 SIGPIPE kills stdout daemons | forge | `wf-crystallise` daemon tasks all use `nohup`-equivalent `tokio::spawn` + channel fan-out |
| T4 `CARGO_TARGET_DIR` conflicts | forge | two-binary split uses per-binary `/tmp/cargo-{name}` paths |
| T5 Port occupied by old process | forge | `wf-dispatch` runs no HTTP server; no port conflict possible (HARD REFUSAL encoded) |
| T6 Health path variance | forge | `wf-crystallise` uses `/health`; confirm devenv.toml entry matches |
| T7 Feature flag requirements | forge | `workflow-core` lib: `povm_calibrated` uses `cargo:rustc-cfg` not Cargo features; cannot bypass with `--features full` |
| T8 Stale PID files | forge | m32 probes Conductor via `ss`-equivalent live socket check; does not trust PID files |

Record audit findings to `$WORK/forge-trap-audit.txt`.

### m10 Ember gate full string enumeration (Stage 4 inclusion)

The m10 `ember_gate` module fails CI on workflows carrying a `Held` verdict. This is a compile-time check surfaced at Stage 4. The mechanical gate Stage 4 run doubles as the Ember gate pass because the test suite includes `ember_gate_fails_on_held_verdict` tests.

**Verification:** after Stage 4, confirm the test output includes explicit enumeration of Ember gate tests. Zero silent exclusions. If the `ember_state: Option<EmberGateResult>` `None` branch is dominant (because Watcher §5.1 Held-semantics amendment is still pending), record that in the baseline JSON as `ember_held_semantics_amendment_pending: true`.

### PRAGMA portability check

Confirm that every SQLite database constructor in the workspace (m30 `workflow_bank.db`, m32 `dispatch_log.db`) uses the shared WAL PRAGMA sequence from the boilerplate Cat-03 pattern. This is the strongest single cross-category pattern in the gold-standard exemplars (GOD_TIER Part IV: "WAL pragmas + version tracking are 100% portable").

```bash
rg -n 'PRAGMA journal_mode' "$PROJECT_DIR/src" --type rust | tee "$WORK/pragma-audit.txt"
# Expect: every db constructor references the shared pragma sequence
# FAIL condition: any raw SqliteOpenFlags call without the pragma sequence
```

### Pre-flight snapshot (Day 26 afternoon)

After Wave 1 PASS, record the baseline state to `baseline.json`. This anchors the deployment receipt.

```bash
cat > "$WORK/baseline.json" << BASELINE_EOF
{
  "session": "$SESSION",
  "wave1_pass_at": "$(date -Iseconds)",
  "binary_sha_pre": {
    "wf_crystallise": "$(cat $WORK/pre-sha-crystallise.txt 2>/dev/null || echo MISSING)",
    "wf_dispatch": "$(cat $WORK/pre-sha-dispatch.txt 2>/dev/null || echo MISSING)"
  },
  "pathway_counts": {
    "stcortex_workflow_trace_ns": "PROBE: stcortex sql 'SELECT COUNT(*) FROM pathway WHERE namespace LIKE \"workflow_trace_%\"'",
    "povm_workflow_trace_ns": "PROBE: curl -s localhost:8125/pathways?prefix=workflow_trace_ | jq '.count'"
  },
  "substrate_ltp_density": "PROBE: stcortex sql 'SELECT COUNT(*) FROM pathway WHERE weight > 0.15' / total",
  "m14_lift_baseline": {
    "cascade_success_rate": "PROBE: wf-crystallise lift --raw",
    "cost_lift": "PROBE: wf-crystallise lift --cost",
    "n_qualifying_workflows": "PROBE: n must be >= 20 for Wilson CI; record None if below"
  },
  "cluster_test_counts": {
    "cluster_a": "PROBE: cargo test --lib -p workflow-core -q 2>&1 | grep 'test result'",
    "cluster_g": "PROBE: cargo test --lib -p wf-dispatch -q 2>&1 | grep 'test result'"
  },
  "ember_held_semantics_amendment_pending": true,
  "conductor_wave_maturity": {
    "wave_1b_live": "PROBE: curl -s localhost:8141/health | jq .enforcer_active",
    "conductor_dispatch_enabled": "PROBE: env CONDUCTOR_DISPATCH_ENABLED"
  },
  "pass_receipt_path": "$WORK/SHIP_RECEIPT.md"
}
BASELINE_EOF

echo "Baseline snapshot: $WORK/baseline.json"
```

**The probes above are recipe slots, not pre-populated values.** Luke or the operator fills them from live service reads at Day 26 afternoon. They must not be approximated or left at prior-session figures — runbook-probe-freshness discipline (pair every field with a freshness timestamp).

---

## Wave 2 — 4-Agent Parallel Gate (Day 27)

All four agents dispatch in a single message (one tool call per agent, no sequential dependency). Each agent writes its verdict to `$WORK/<agent>.md`. The verdict collection step runs only after all four complete.

### Agent 1: security-auditor

**Scope:** Vulnerabilities, attack surfaces, authentication/authorisation gaps, escape-surface classification correctness, habitat-specific security traps.

**Tools:**
- `rg` over full `src/` tree — secrets scan (PAT patterns, raw credential strings, `http://` prefix violations)
- `cargo audit` — RUSTSEC advisory check on `Cargo.lock`
- Manual read of `Cargo.toml` dependency tree — new transitive deps vs known-good set
- Read of m30 `AcceptedWorkflow`, m32 `ConductorDispatchRequest`, m9 namespace guard — auth surface review
- Read of m32 `probe_conductor()` — URL prefix check (no `http://`; raw socket addr only per m24 gold standard)

**REJECT criteria (any one causes REJECT):**
- A plaintext credential, PAT, or API token appears in any source file
- Any `http://` prefix in an outbound HTTP call (habitat trap: must be raw socket addr)
- m32 contains any code path that executes a workflow step directly (P0 #3 violation)
- `CONDUCTOR_DISPATCH_ENABLED` can be bypassed by any code path
- m9 namespace guard is absent or bypassable (AP30: `workflow_trace_*` prefix mandatory)
- A new dependency introduced with a known RUSTSEC advisory

**NITS criteria (APPROVE-WITH-NITS, not REJECT):**
- Missing `#[must_use]` on security-class return types
- `tracing::error!` call sites that log sensitive field values at non-redacted verbosity
- CORS, rate-limiting, or DoS-protection gaps on `wf-crystallise` HTTP endpoints (lower severity since loopback-only)

**F-modes owned:**
- **F8 — Watcher feedback-loop poisoning:** confirm no code path allows an external agent to inject into m10's Ember gate, m9's namespace guard, or m31's selection scoring without human-mediated acceptance. The CC-4 pipeline requires `wf-crystallise propose accept <id>` as a human gate; audit that no RPC or HTTP shortcut bypasses it.
- **F11 — Cascade monoculture potential:** confirm m4 cluster IDs are opaque (FNV-1a XOR of window range + sorted pane labels + step count). If any code path preserves pane label strings (ALPHA-LEFT, BETA-TR) in the cluster ID, flag as REJECT — the destruction of semantic content is the point.

**Verdict format:**

```
Verdict {
  agent: "security-auditor",
  status: APPROVE | APPROVE-WITH-NITS | REJECT,
  evidence: [
    "<file:line> — <finding>",
    ...
  ],
  specific_findings: [
    {
      id: "SEC-NNN",
      severity: critical | high | medium | low,
      location: "src/mN_*/file.rs:LINE",
      description: "...",
      proposed_fix: "..."
    }
  ]
}
```

---

### Agent 2: performance-engineer

**Scope:** Latency budgets, memory allocation profiles, hot-path identification, async execution correctness, lock acquisition ordering.

**Tools:**
- Static read of `src/m20_*/` (PrefixSpan hot path), `src/m31_*/` (selection scoring loop), `src/m4_*/` (cascade correlator)
- `rg` for `tokio::task::spawn_blocking`, `std::thread::sleep`, `std::sync::Mutex` in async contexts (AP29 detector)
- `rg` for `Vec::with_capacity` absence in known-large collection sites (m20 prefix projection allocations, m31 candidate pool)
- `rg` for unbounded `loop { }` without a yield point or timeout
- Criterion benchmark existence check: does `benches/` contain benchmarks for PrefixSpan and m31 selection scoring?

**Hot paths (per cluster architecture):**
- **m20 PrefixSpan projection loop** — gap-allowed matching with `MAX_GAP_STEPS = 5` can be O(n × m × MAX_GAP) per sequence pair; confirm `Vec::with_capacity(projected_count)` pre-allocated
- **m31 selection scoring** — candidate pool fetch `BankDb::eligible(limit=200)` followed by n² n-gram Jaccard similarity; confirm the `HashSet<&[StepId]>` windows are not being re-allocated per candidate
- **m14 Wilson CI computation** — called per workflow per `ProposalBuilder::build()` invocation; confirm no floating-point allocations in the confidence interval formula
- **m4 FNV-1a XOR hash** — called at cluster ID assignment; confirm `fnv1a_64` is a const or inline function, not a heap allocation

**Memory budgets:**
- `wf-crystallise` binary: O(sessions × steps × MAX_GAP) working set. Document expected max resident set size at 100-session, 50-step-per-session scale.
- `wf-dispatch` binary: O(bank_size × candidate_pool) for selection. Bank limit check: does `BankDb::eligible(limit=200)` prevent unbounded pool growth?

**REJECT criteria:**
- Any `std::thread::sleep` or `std::sync::Mutex::lock()` inside a `tokio::spawn` task body (AP29 — sync HTTP in tokio::spawn starves the runtime)
- An unbounded `loop { }` without an `await` yield point or timeout guard
- `ProposalBuilder::build()` does not enforce `n >= 20` at construction (F2: no bypass path)
- m31 selection scoring is O(n²) without a hard cap on candidate pool size

**NITS criteria:**
- Missing `Vec::with_capacity` on allocations inside hot loops (avoids re-allocation amortisation)
- No criterion benchmark for PrefixSpan (acceptable at M0; flag as technical debt)
- `RECENCY_HALF_LIFE_MS` and `LINEAGE_SCORE_TOLERANCE` not surfaced as runtime config (hardcoded constants are acceptable; note as future config point)

**F-modes owned:**
- **F2 — Sample-size `n<20` escaping:** verify `ProposalBuilder::build()` returns `Err` (not a degraded proposal) when `n < 20`. Check that the `n >= 20` enforcement is at the type-system boundary, not a runtime assertion that can be bypassed by `unwrap_or(Default::default())`. Also verify m14's Wilson CI formula returns `None` (not `0.0`) when n < 20 — the `None` sentinel propagates correctly to the bank acceptance gate.
- **F10 — Exploration-cost preservation collapse:** verify m6's 20-session EMA **excludes** sessions classified as `Converged`. The baseline should shift only from exploration-classified sessions. A regression here would create habitat pressure to eliminate exploratory workflows by inflating their apparent cost relative to exploitative ones.

**Verdict format:** same structure as security-auditor, agent field `"performance-engineer"`.

---

### Agent 3: silent-failure-hunter

**Scope:** 5 canonical anti-patterns from `SKILL-silent-swallow-detect.md` + wildcard IPC drop pattern. Applied across the full `src/` tree, both binaries, shared lib. FP-preventer filters applied to every hit before filing.

**Tools:** `rg` pattern hunts per the three-pattern scan in `SKILL-silent-swallow-detect.md`.

**The 5 hunts:**

```bash
# Hunt 1: unwrap_or(true) on external healthy/consent/success fields
rg -n 'unwrap_or\(true\)' --type rust \
  -g '!tests/*' -g '!*test*.rs' -g '!benches/*' \
  "$PROJECT_DIR/src"

# Hunt 2: .ok() discarding meaningful HTTP/DB ops
rg -n '\.ok\(\);' --type rust \
  -g '!tests/*' -g '!*test*.rs' \
  "$PROJECT_DIR/src" \
  | /usr/bin/grep -iE 'http|post|request|send|resp|bridge|fetch|publish|conductor|dispatch'

# Hunt 3: let _ = masking Result on send/write/dispatch calls
rg -nP '^\s*let _ = .+(send|write|execute|dispatch|publish|notify|conductor)' --type rust \
  -g '!tests/*' -g '!*test*.rs' "$PROJECT_DIR/src"

# Hunt 4: Ok(0) / Ok(()) success sentinels
rg -n 'Ok\((?:0|\(\)|false)\)\s*;?\s*(?://|$)' --type rust \
  -g '!tests/*' "$PROJECT_DIR/src"

# Hunt 5: wildcard IPC drops (match result { _ => {} } on Conductor handoff)
rg -n 'match.*result.*\{' --type rust \
  -g '!tests/*' "$PROJECT_DIR/src" \
  | /usr/bin/grep -iE 'conductor|dispatch|channel|tx\.|send'
```

**FP-preventer filters (must apply to every hit before filing):**

Per `SKILL-silent-swallow-detect.md` Pattern C:
1. Reject hits where surrounding block has `#[cfg(test)]`
2. Reject hits in `tests/`, `benches/`, `examples/` paths
3. Reject hits inside doc-comment code examples
4. Reject `.ok()` on cleanup calls where error is genuinely meaningless (socket close, best-effort notification in shutdown paths)
5. Reject `unwrap_or(0)` on hash/numeric conversion guards
6. Reject `let _ = tx.send(msg)` inside `tokio::select!` cleanup arms where receiver may be gone

**Verify each surviving hit** by reading 20 lines of context to establish caller contract. A hit is real only if: (a) it survives all FP filters AND (b) a downstream consumer reads the lied-to return value and proceeds on it.

**workflow-trace specific exempt patterns:**

- `m32 fan-out` to Cluster H is fire-and-forget by design: "a failure to fan out is logged but does not roll back the dispatch." A `let _ =` or `.ok()` on the Cluster H fan-out channel is **not** a silent failure if: it is preceded by a `tracing::warn!` call, and the dispatch audit record was already written (audit-first guarantee). Read `m32::fan_out_to_cluster_h()` to verify both conditions before flagging.
- `m7 JSONB consumer_inputs` blobs: `serde_json::to_string().ok()` on internal reconciliation blobs is acceptable if the outer operation returns `Err` on serialisation failure. Trace the error propagation.

**REJECT criteria (any one):**
- `probe_conductor()` in m32 swallows a network error with `.ok()` and returns `Ok(())` instead of `Err(ConductorNotLive)`
- Any code path where `DispatchError::ConductorDispatchDisabled` is converted to an `Ok` variant
- `BankDb::record_dispatch()` or `BankDb::record_verification()` silently swallows a SQLite error (audit-first guarantee breach)
- m42 `POST /reinforce` to POVM uses `let _ =` without a preceding `tracing::warn!` log

**NITS criteria:**
- `.ok()` on SYNTHEX v2 nexus push (m40 fire-and-forget) without logging — warn only if no `tracing::debug!` precedes it

**F-modes owned:**
- **F7 — CR-2 graceful-degrade pretend-fix:** m13 Hebbian backpressure check uses three bands (>0.15 proceed; 0.05-0.15 proceed-with-warning; <0.05 defer to local JSONL). Hunt for any code path where the `<0.05` degrade band silently proceeds instead of deferring. The substrate is currently at LTP/LTD=0.043 (35× below healthy). If m13's degrade band does not defer writes at this substrate condition, the engine will overwrite pathways onto degraded substrate without any warning. This is the most acutely live F-mode at S1001982 substrate baseline.
- **F3 — Substrate-input poisoning:** verify m1/m2/m3 (Cluster A ingest) do not accept arbitrary external input into the m7 JSONB `consumer_inputs` blobs without type validation. A poisoned blob that passes `serde_json::from_str()` without schema enforcement can inject arbitrary data into m7's hub table. Confirm that each ingest module validates the consumer input against a known type before the write.

**Verdict format:** same structure, agent field `"silent-failure-hunter"`.

---

### Agent 4: zen

**Scope:** God-tier capstone review. Habitat-aware. Catches what the other three structurally miss: Rust idiom correctness, v1.2 verb-locked invariants for Phase A modules, m33 as the PRE-runtime equivalent of the deployment gate, test coverage adequacy, BUG-035 mono-parameter trap, structural algorithm correctness.

**Tools:**
- Full source read of all 26 modules (Cluster A through H)
- Read of v1.2 genesis prompt binding spec (verb-lock invariants for active-verb modules m20-m23, m30-m33, m40-m42)
- Read of m33 `VerificationAgent` enum definition — confirm it mirrors the 4-agent pre-deploy gate pattern faithfully
- BUG-035 mono-parameter trap audit (see dedicated section below)
- Structural algorithm spot-check (PrefixSpan, Wilson CI, m11 decay formula)

**A1-A8 Rust idiom checklist:**

| Code | Check |
|---|---|
| A1 `unwrap()` | Zero in non-test code. `expect()` with context string is acceptable in binary main only. |
| A2 `unsafe` | Zero unless documented with safety invariant comment. No `unsafe` appears in spec. |
| A3 `Send + Sync` | All types crossing `tokio::spawn` boundaries derive or impl `Send`. m31 `SelectionContext` is `VecDeque`-backed; confirm it is `Send`. |
| A4 Error doc comments | Every `pub enum Error` variant has a doc comment explaining when it is returned. `DispatchError` has 8 variants — verify all 8. |
| A5 `tracing` structured | `tracing::error!`, `tracing::warn!`, `tracing::info!` with structured fields (not string interpolation). `tracing::error!(conductor_addr = %addr, "...")` not `format!()`. |
| A6 `#[allow(...)]` | No blanket `#[allow(clippy::...)]` that silences pedantic warnings without a justification comment. |
| A7 glob imports | No `use module::*` in library code. |
| A8 `RwLock` ordering | Any shared state using `RwLock` must have a documented lock ordering comment if >1 lock acquired in same scope. |

**v1.2 verb-locked invariants for Phase A modules:**

The genesis prompt v1.2 defines verb-locked invariants for modules in active-verb clusters (m20-m23, m30-m33, m40-m42). Zen verifies that each active-verb module has a public API that matches its assigned verb:

- m20: `detect_patterns()` — PrefixSpan scan returns `Vec<DetectedPattern>`
- m21: `compute_similarity()` — normalised Levenshtein, returns `f64`
- m22: `select_variants()` — top-K by ascending edit distance, returns `Vec<Variant>`
- m23: `propose()` — emits `WorkflowProposal`; deviation-as-evidence additive only
- m30: `accept()`, `eligible()`, `apply_decay_tick()` — BankDb surface
- m31: `select()`, `record_selection()` — diversity-enforced
- m32: `dispatch()` — NEVER executes directly; Conductor wire only
- m33: `verify()` — returns `VerificationResult`; audit-first on PASS
- m40: `emit_nexus_event()` — fire-and-forget; circuit breaker
- m41: `route_lcm()` — `lcm.loop.create` with `max_iters: 1`
- m42: `reinforce()` — `POST /reinforce`; `workflow_trace_*` namespace only

**m33 as PRE-runtime equivalent of m33 deployment gate:**

This is the key structural coherence check. The `workflow_verifier` (m33) in the codebase is the RUNTIME gate: it runs before dispatch. Phase 4 Pre-Deploy Hardening is the PRE-DEPLOY gate: it runs before binary deployment. They are the same 4-agent pattern at different scopes.

Zen verifies:
1. `VerificationAgent` enum in m33 has exactly 4 variants: `Security`, `Performance`, `SilentFailure`, `Zen`
2. `VerificationVerdict` enum has exactly 3 variants: `Pass`, `Fail`, `Degraded`
3. Boolean AND logic: `Pass` requires all 4 agents `approved = true`; `Fail` on any `approved = false`; `Degraded` on all `nits_only = true`
4. The gate is structurally isomorphic to the pre-deploy gate in this document — same 4 agents, same verdict structure, same boolean AND

If m33 has drifted from this structure (e.g., 3 agents, different verdict names), it is a REJECT.

**Structural algorithm spot-checks:**

- **PrefixSpan correctness:** confirm gap-allowed matching uses `MAX_GAP_STEPS = 5` as a bounded right-gap, not an unbounded scan. Pattern `[A, B, C]` should match `[A, X, B, Y, C]` (2 gap steps in right region, within MAX_GAP) but not `[A, X, X, X, X, X, B, Y, C]` (6 gap steps, exceeds MAX_GAP).
- **Wilson CI implementation:** confirm `z = 1.96` (95% CI, not 90% or 99%). Confirm the formula is `(p_hat + z²/(2n) ± z*sqrt(p_hat*(1-p_hat)/n + z²/(4n²))) / (1 + z²/n)` (Agresti-Coull) or the standard Wilson form, not the naive Wald approximation (which produces negative lower bounds at small n).
- **m11 decay formula:** confirm implementation matches `decay_factor = base_rate + (1.0 - base_rate) × clamp(frequency × fitness × recency, 0.0, 1.0)` with `base_rate ≈ 0.98`. A zero-signal workflow (all three multipliers → 0) should decay by `1.0 - 0.98 = 0.02` per cycle, reaching prune_threshold (0.01) after ~228 cycles.

**REJECT criteria:**
- Any A1-A8 violation in production (non-test) code
- m33 `VerificationAgent` enum does not match the 4 named agents
- PrefixSpan gap-allowed matching is unbounded (no `MAX_GAP_STEPS` enforcement)
- Wilson CI uses Wald approximation instead of Wilson/Agresti-Coull form
- m11 decay formula uses `×` without the `clamp(0.0, 1.0)` guard (negative product possible when fitness < 0)
- BUG-035 mono-parameter trap present (see dedicated section)

**NITS criteria:**
- Test coverage below 50 per module (flag by module; single-module shortfall is a nit; systemic shortfall is REJECT)
- Missing doc comments on public types (A4 extension to types, not just errors)
- `BASE_RATE` / `PLAIN_DECAY_RATE` not named constants (magic number 0.98 inline is a nit)

**F-modes owned:**
- **F1 — Bank/name ossification:** confirm m30 `AcceptedWorkflow::id` is opaque (UUID v7, not a human-readable name derived from step content). A workflow ID that encodes step labels is a semantic anchor — it creates naming pressure to maintain those labels. The bank should remain semantically inert.
- **F9 — Workflow-grain fitness distortion:** verify m7 schema has `fitness_dimension REAL NOT NULL DEFAULT 0.0`. The column is convention-enforced, not CHECK-constrained (per spec). Zen flags if the column is absent from the DDL. Also verify m31 composite score applies m14's `WorkflowLiftContribution.delta` bounded `clamp(-0.3, +0.3)` — no single workflow's lift score can dominate selection by more than 30%.

**Verdict format:** same structure, agent field `"zen"`.

---

## BUG-035 Mono-Parameter Audit

BUG-035 is the lead anti-pattern from ORAC's `m40_mutation_selector`: when a selector applies diversity enforcement, but the enforcement algebra has a single-parameter bypass (e.g., all candidates share lineage X, 50% mono-parameter rejection fires, but the replacement pool is also dominated by lineage X), the diversity gate becomes decorative.

**Zen verifies m31's diversity algebra (round-robin + 10-gen cooldown + 50% mono-parameter rejection) works as designed:**

Three test invariants that must pass:

1. **Mono-parameter gate actually replaces:** When a pool of 10 candidates has >5 from the same `lineage`, the tail candidates from that lineage are dropped and replaced by next-highest-scoring candidates from other lineages. If no other lineages exist in the pool, the test should return a sub-10 ranked list, not fill with the same-lineage candidates.

2. **10-gen cooldown suppresses consistently:** A workflow dispatched 1 generation ago must score at or below `MIN_SELECTION_SCORE` in all selection cycles within the next 10 dispatches. The context window `recent_lineages: VecDeque<LineageId>` must be bounded at 10 — confirm `.pop_front()` fires when len > 10.

3. **Round-robin breaks ties deterministically:** When two lineages score within `LINEAGE_SCORE_TOLERANCE = 0.05`, the selector cycles through lineages by `accepted_at` order, not always the higher scorer. Test: two workflows L1 and L2 with composite scores 0.800 and 0.796 (within 0.05); after L1 is selected, L2 must be next candidate even if its score is lower.

If any of these three invariants fail in the test suite, Zen files REJECT with evidence pointing to the specific test failure.

---

## Preserve-List Discipline Check (Cipher Escape-Surface Audit)

Per `feedback_preserve_list_discipline.md` (S102 scar tissue): blanket operations dissolve named-exclusion safety. The S102 incident lost `openclaw-gateway` to `docker container prune -f` despite a prior named exclusion.

**In workflow-trace context**, the analogous risk is m30's `EscapeSurfaceProfile` ordinal — specifically the `Destructive` classification. When m32 dispatches a `Destructive`-profile workflow via HABITAT-CONDUCTOR, the Conductor's execution authority means any blanket operation within that workflow (e.g., `docker container prune`, `cargo clean --all-targets`, `git clean -fd`) may violate preserve-list entries that were established in earlier workflow steps.

**Security-auditor owns this check:**

1. Verify m32's `display-before-step` gate emits the `[DESTRUCTIVE]` banner for every `Destructive`-profile step: "This step runs a potentially irreversible operation. Enumerate targets before proceeding."

2. Verify the `EscapeSurfaceProfile::Destructive` step banner specifically calls out the enumeration requirement (matching the preserve-list discipline rule: "enumerate its targets explicitly, diff against preserve list, refuse blanket form").

3. Confirm there is no code path in m32 that reduces a `Destructive` profile to `HostWrite` or lower at dispatch time (downgrade must not be possible after bank acceptance; escape surface is immutable post-acceptance per m30 design).

4. Confirm m9 namespace guard applies at all write boundaries — a workflow writing to `workflow_trace_*` namespace cannot accidentally write to `stcortex` system namespaces or POVM core pathways.

---

## Wave 2 Verdict Collection (Day 27 Evening)

After all four agents complete, collect verdicts:

```bash
APPROVED=true
DEGRADED=false
VERDICT_LOG="$WORK/wave2-verdicts.txt"

for agent in security-auditor performance-engineer silent-failure-hunter zen; do
  VERDICT=$(rg -o '^(APPROVE[^-\s]*|REJECT)' "$WORK/${agent}.md" 2>/dev/null | head -1)
  echo "${agent}: ${VERDICT:-NO_VERDICT}" | tee -a "$VERDICT_LOG"

  [[ "$VERDICT" == "REJECT" ]] && APPROVED=false
  [[ -z "$VERDICT" ]] && APPROVED=false
  [[ "$VERDICT" == "APPROVE-WITH-NITS" ]] && DEGRADED=true
done

# Boolean AND logic
if $APPROVED && ! $DEGRADED; then
  echo "Wave 2: PASS — all 4 agents APPROVE" | tee -a "$VERDICT_LOG"
  echo "WAVE2_RESULT=PASS" > "$WORK/wave2-result.env"
elif $APPROVED && $DEGRADED; then
  echo "Wave 2: DEGRADED — all agents approve with nits" | tee -a "$VERDICT_LOG"
  echo "WAVE2_RESULT=DEGRADED" > "$WORK/wave2-result.env"
else
  echo "Wave 2: FAIL — one or more agents REJECT" | tee -a "$VERDICT_LOG"
  echo "Rejecting agents:"
  rg -A 10 '^REJECT' "$WORK"/*.md | head -50
  echo "WAVE2_RESULT=FAIL" > "$WORK/wave2-result.env"
fi
```

---

## Decision Branch (Day 28 Morning)

Three paths depending on Wave 2 result:

**FAIL:** Halt deploy entirely. Do not proceed to Wave 3. Do not proceed to Phase 5. Address every REJECT finding from the failing agent(s). Then restart from Wave 1 (mechanical gate re-run from scratch). A new `SESSION` ID is required — receipts are append-only audit trail and cannot be re-used. The prior `baseline.json` remains as a reference but a new baseline is recorded on re-run.

**DEGRADED:** Luke decides. Present the NITS list explicitly. Two options: (a) address nits now and re-run Wave 2 from fresh baseline; (b) accept DEGRADED and proceed to Wave 3 with a `ship_with_known_nits: true` flag in the receipt. This flag must appear in the deployment commit message. DEGRADED-with-nits is not a silent state — it is a first-class tracked decision.

**PASS:** Proceed to Wave 3 without further action.

---

## Wave 3 — Watcher Witness (Day 28 Afternoon)

The Watcher's role in Phase 4 is observation and flagging, consistent with the Watcher Deployment Watch Journal posture: "observe-don't-interfere; honest-flag-loud; synthesise-at-end."

### What the Watcher observes

The Watcher reads:
1. `$WORK/baseline.json` — pre-deploy substrate state
2. `$WORK/wave1-pass.txt` — mechanical gate timestamp
3. `$WORK/wave2-verdicts.txt` — all 4 agent verdicts
4. `$WORK/wave2-result.env` — consolidated PASS/FAIL/DEGRADED

The Watcher does NOT modify these files, re-run gates, or inject findings. If the Watcher identifies a concern, it files a Class-A or Class-E notice — not a late REJECT that unilaterally halts the phase.

### Class A — Gate flip flag

Watcher fires a **Class A** flag if:
- Wave 2 PASS transitions to Phase 5 deployment. Class A is "Activation transition (gate flip)." The WCP notice timestamps the exact moment the pre-deploy gate opens.
- The deployment receipt is written. Class A notice confirms the record exists.

WCP notice format:
```
~/projects/shared-context/watcher-notices/PHASE-4-PASS-<YYYYMMDD-HHMM>.jsonl
{
  "class": "A",
  "event": "pre-deploy-gate-pass",
  "session": "<SESSION>",
  "pass_at": "<ISO8601>",
  "baseline_snapshot": "<path>",
  "receipt_path": "<path>"
}
```

### Class E — Ancestor-rhyme flag

Watcher fires a **Class E** flag if Wave 2 produces **>1 REJECT** (two or more agents reject):
- A multi-agent rejection at Phase 4 is structurally similar to the 41,508 words / 0 LOC planning-sprawl indicator — accumulated technical debt making the engine unshippable.
- Class E notice surfaces the count and agent names: "Pre-deploy gate: N of 4 agents rejected. Pattern: ancestor-rhyme leading indicator. Consider addressing at gap level, not individual finding level."

WCP notice format:
```
~/projects/shared-context/watcher-notices/PHASE-4-CLASS-E-<YYYYMMDD-HHMM>.jsonl
{
  "class": "E",
  "event": "multi-agent-rejection",
  "session": "<SESSION>",
  "rejecting_agents": ["<agent1>", "<agent2>"],
  "reject_count": N
}
```

### m15 Pressure Register pre-deploy snapshot

m15 (`pressure_register`) detects forbidden-verb-pressure events — scope-pressure accumulation from the planning phase. Before Phase 5 deployment, record a pressure-register snapshot to establish the Phase 4 baseline:

```bash
# Query m15's pressure event log (or its dev-time equivalent)
# Expected: zero PHASE-B-RESERVATION-NOTICE files (Phase 4 is post-planning)
ls ~/projects/shared-context/agent-cross-talk/PHASE-B-RESERVATION-NOTICE-*.jsonl 2>/dev/null \
  | wc -l > "$WORK/pressure-notice-count.txt"
echo "Pressure notices at Phase 4 close: $(cat $WORK/pressure-notice-count.txt)"
```

If any `PHASE-B-RESERVATION-NOTICE` files exist at Phase 4 close (scope-pressure was emitted during integration), they must be resolved or acknowledged before Phase 5. Outstanding notices are surfaced in the deployment receipt.

### Canonical deployment receipt

The Phase 4 receipt is written only after Watcher witness completes. It is the authoritative record consumed by Phase 5.

```bash
{
  echo "# Phase 4 Pre-Deploy Hardening Receipt"
  echo "session: $SESSION"
  echo "wave1_pass_at: $(cat $WORK/wave1-pass.txt)"
  echo ""
  echo "## Wave 2 Agent Verdicts"
  for agent in security-auditor performance-engineer silent-failure-hunter zen; do
    echo "### $agent"
    head -5 "$WORK/${agent}.md" 2>/dev/null || echo "(no verdict)"
  done
  echo ""
  echo "## Consolidated Result"
  cat "$WORK/wave2-result.env"
  echo ""
  echo "## Baseline Snapshot"
  echo "See: $WORK/baseline.json"
  echo ""
  echo "## Watcher Witness"
  ls ~/projects/shared-context/watcher-notices/PHASE-4-*.jsonl 2>/dev/null || echo "(no WCP notices)"
  echo ""
  echo "## Pressure Notices Outstanding"
  echo "Count: $(cat $WORK/pressure-notice-count.txt 2>/dev/null || echo UNCHECKED)"
  echo ""
  echo "## Ship-With-Known-Nits"
  source "$WORK/wave2-result.env" 2>/dev/null
  [[ "$WAVE2_RESULT" == "DEGRADED" ]] && echo "ship_with_known_nits: true" || echo "ship_with_known_nits: false"
} > "$WORK/SHIP_RECEIPT.md"

echo "Deployment receipt: $WORK/SHIP_RECEIPT.md"
```

---

## Failure-Mode Ownership (F1-F11)

All 11 failure modes identified across the architecture are owned by a specific agent in this gate. Ownership means: the agent is the primary scanner for that failure mode and files findings under that F-code.

| F-mode | Description | Owner |
|---|---|---|
| F1 | Bank/name ossification — workflow IDs encode semantic content | zen |
| F2 | Sample-size `n<20` escaping — proposals or CI with sub-threshold n | performance-engineer |
| F3 | Substrate-input poisoning — unvalidated JSONB ingested into m7 | silent-failure-hunter |
| F7 | CR-2 graceful-degrade pretend-fix — m13 degrade band silently proceeds | silent-failure-hunter |
| F8 | Watcher feedback-loop poisoning — bypass path into Ember/m9/m31 | security-auditor |
| F9 | Workflow-grain fitness distortion — m7 fitness_dimension absent or unclamped | zen |
| F10 | Exploration-cost preservation collapse — m6 EMA includes Converged sessions | performance-engineer |
| F11 | Cascade monoculture potential — m4 cluster IDs retain semantic content | security-auditor |

F4, F5, F6 are not listed in the architecture documentation and are not pre-assigned. If agents discover novel failure modes, they file under SS-NNN (silent-failure-hunter) or SEC-NNN (security-auditor) naming convention.

---

## Hand-off to Phase 5

The SHIP_RECEIPT.md file at `$WORK/SHIP_RECEIPT.md` is the Phase 5 entry condition.

**PASS verdict:** Phase 5 deploy proceeds. The receipt path is recorded in the deployment commit message. Phase 5 reads the `baseline.json` binary SHA fields to verify it is deploying the same binary that was gated.

**FAIL verdict:** Phase 5 does not start. The SHIP_RECEIPT.md records the failure date and the rejecting agents. A new Phase 4 run begins with a new SESSION ID.

**DEGRADED verdict with `ship_with_known_nits: true`:** Phase 5 proceeds with the nits flag set. The deployment commit message must include `DEGRADED-nits-accepted` in the commit summary. The nit findings from the relevant agent's verdict file are referenced by path in the commit message body.

**Anti-patterns (do not do these):**
- Do not run agents before Wave 1 passes — they will audit broken code
- Do not skip an agent verdict on the grounds "the others approved" — boolean AND is the gate
- Do not reuse a SESSION ID across attempts — receipts are append-only audit trail
- Do not gate on >1000-line diffs without splitting — split the diff first, gate per chunk
- Do not collapse Wave 1 + Wave 2 + Wave 3 into a single day — phase-collapse under context pressure is the anti-pattern this framework is designed to prevent

---

## Constraint Reference

**P0 constraints carried into Phase 4:**
- P0 #3: m32 never executes directly — zen verifies; security-auditor verifies
- P0 #7: m31 → m33 consultation before dispatch — m33 TTL check in m32 pre-dispatch gate; zen verifies gate sequence
- P0 #9: m33 `last_verified_at` TTL gates re-runs — zen verifies TTL constant and stale-dispatch error path
- P0 #11: every `AcceptedWorkflow` carries `escape_surface_profile`; display-before-step mandatory — security-auditor verifies; preserve-list discipline check
- P0 #15: m32 hard startup refusal when Conductor not live — security-auditor verifies `ConductorDispatchDisabled` is not bypassable

**Watcher W1-W3 conditions (monitored, not gates):**
- W1 (narrowed-scope consumer): workflow-trace is a single-phase build, not a phased scope extension. Watcher monitors for scope-creep during integration.
- W2 (CR-2 hard build-prereq): CR-2 (`e2a8ed3` + CR-2b `76ea4d6`) shipped source. Before Phase 5, confirm live `:8125` re-measurement shows `learning_health` in 0.05-0.15 band (not 0.067 stale figure).
- W3 (Ember 7-trait CI gate): Ember §5.1 Held-semantics amendment pending. m10 fails CI on Held verdicts until Watcher amends. Phase 4 does not block on W3; it records the amendment-pending state in `baseline.json` and carries the flag into Phase 5.

---

## Appendix — Agent Dispatch Template

The Day 27 Wave 2 dispatch is a single message with four parallel tool calls. Template for the dispatch message:

```
Dispatch: Wave 2 pre-deploy gate for workflow-trace — 4 agents in parallel.

WORK dir: /tmp/predeploy-hardening-<SESSION>/
Project: /home/louranicas/claude-code-workspace/the-workflow-engine/

Agent 1 — security-auditor:
  Audit full src/ for: secrets, http:// prefix violations, m32 direct-execution paths,
  CONDUCTOR_DISPATCH_ENABLED bypass paths, m9 namespace guard, new RUSTSEC advisories.
  F8: Ember/m9/m31 bypass paths. F11: m4 cluster ID semantic content.
  EscapeSurfaceProfile Destructive: enumerate-before-proceed banner required.
  Write verdict to $WORK/security-auditor.md. APPROVE | APPROVE-WITH-NITS | REJECT.

Agent 2 — performance-engineer:
  Audit: m20 PrefixSpan hot path (Vec::with_capacity), m31 candidate pool O(n²) cap,
  m14 Wilson CI None propagation, AP29 sync-in-async, unbounded loop scan.
  F2: ProposalBuilder n<20 enforcement at type boundary.
  F10: m6 EMA excludes Converged sessions.
  Write verdict to $WORK/performance-engineer.md. APPROVE | APPROVE-WITH-NITS | REJECT.

Agent 3 — silent-failure-hunter:
  Hunt 5 patterns per SKILL-silent-swallow-detect: unwrap_or(true), .ok() on
  HTTP/dispatch ops, let _ = on dispatch/conductor, Ok(0) sentinels, wildcard IPC drops.
  FP-preventer filters mandatory. m32 fan-out exemption rule applies.
  F7: m13 degrade band at LTP/LTD=0.043 substrate. F3: JSONB ingest validation.
  Write verdict to $WORK/silent-failure-hunter.md. APPROVE | APPROVE-WITH-NITS | REJECT.

Agent 4 — zen:
  God-tier capstone: A1-A8 Rust idioms, v1.2 verb-locked API surfaces,
  m33 isomorphism to this 4-agent gate, BUG-035 mono-parameter 3-invariant test,
  PrefixSpan MAX_GAP_STEPS enforcement, Wilson CI form, m11 decay formula clamp.
  F1: workflow ID opacity. F9: m7 fitness_dimension + m31 clamp(-0.3,+0.3).
  Write verdict to $WORK/zen.md. APPROVE | APPROVE-WITH-NITS | REJECT.
```

---

*Command-3 librarian lane · S1001982 · planning-only · HOLD-v2 active*
