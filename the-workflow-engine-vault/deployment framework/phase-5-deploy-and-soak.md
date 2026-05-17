---
title: "Phase 5 — Deploy + 120-day Soak (Day 28 → D120)"
date: 2026-05-17 (S1001982)
kind: deployment-framework
status: planning-only · HOLD-v2 active · authored before G9-fire
phase: 5-of-6
binary_targets: [wf-crystallise, wf-dispatch]
soak_window: D30 → D120 (90 days of sustained observation)
key_metric: m14 habitat_outcome_lift (n≥20, Wilson CI, threshold TBD G5 interview)
primary_watcher_flags: [Class-E, Class-I, Class-A, Class-B]
povm_cutover_within_soak: true (POVM 2026-07-10 ≈ D25 if deploy ≈ 2026-06-15)
---

> Back to: [[../HOME]] · [[../GOD_TIER_CONSOLIDATION_S1001982]] · [[phase-4-pre-deploy-hardening]]

# Phase 5 — Deploy + 120-day Soak (Day 28 → D120)

This is the production-cutover and sustained-observation phase of the ULTIMATE DEPLOYMENT FRAMEWORK for `workflow-trace`. It covers three sequential sub-phases: binary materialisation (5A, Day 28-30), production-live ceremony (5B, Day 30), and a 90-day continuous soak (5C, D30 → D120). The soak terminates at the m11 sunset evaluation in Phase 6.

The framework sections below are written with the knowledge that `workflow-trace` is a **CLI pair, not a daemon service.** No devenv batch slot is required. No `/health` HTTP endpoint exists. No port is occupied. This changes several forge-trap mitigations compared to a normal ULTRAPLATE service deploy.

---

## Why workflow-trace deploys as CLI, not service

The Zen Hard Refusal embedded in Genesis Prompt v1.2 is binding: "no sidecar daemon; no persistent HTTP listener; no devenv batch 5 service entry." This was not waived by the 2026-05-17 Luke single-phase override. The override waived Fossil scope-discipline and RALPH measurement safety (partially); it did not touch the architectural shape of the binaries.

The two binaries, `wf-crystallise` and `wf-dispatch`, are **invoked at command time and exit when done.** `wf-crystallise` runs on a cron schedule to sweep substrate, correlate observation windows, and emit reports. `wf-dispatch` is invoked by Luke or an authorised agent when a workflow in the m30 bank is ready for dispatch. Neither binary holds a TCP socket between invocations. There is no PID file to manage, no `devenv restart` to issue, and no health endpoint to probe.

This means:

- **Forge trap 4 (port detection):** not applicable. No port. Document it explicitly in the smoke test checklist so the operator does not waste time looking for a health endpoint.
- **Forge trap 3 (SIGPIPE):** relevant only if a `wf-crystallise` run is piped to another tool (e.g., `wf-crystallise report | less`). Mitigate by documenting that `wf-crystallise report` should be invoked without piping in cron context, and that interactive use with a pager is acceptable.
- **Forge trap 6 (health path variance):** the CLI health probe is `wf-crystallise --version` returning exit code 0 and a semver string. That is the canonical smoke signal. `wf-dispatch --help` returning exit code 0 is the secondary probe.
- **Forge trap 7 (stale PIDs):** CLI binaries do not write PID files. However, cron-scheduled invocations of `wf-crystallise` can overlap if the prior run is still in progress. Mitigation: atuin trajectory timestamps show run durations; the cron entry should use a lockfile pattern (see Phase 5C).

The decision to deploy as CLI aligns with the engine's measurement philosophy: an instrument that runs on schedule and exits clean is easier to reason about than a daemon whose state drifts. The slow learning loop (days to weeks for CC-5 to close) does not require a persistent resident process.

---

## No devenv batch slot required

Because `workflow-trace` is CLI-only, no `[[services]]` entry in `~/.config/devenv/devenv.toml` is needed or appropriate. The 14-service ULTRAPLATE stack runs independently of this engine. HABITAT-CONDUCTOR (Waves 1B/1C/2/3 installed, `auto_start=false`) is a dependency for dispatch operations via m32, but `workflow-trace` does not start Conductor — it calls Conductor's API when dispatching.

If the scaffold output from `/genesis` or `/scaffold` generates a devenv.toml entry suggestion, that suggestion should be **ignored or explicitly refused** for this codebase. The only registration that matters is:

1. `atuin scripts add wf-crystallise-sweep` — the cron-managed crystallisation run script
2. `atuin scripts add wf-dispatch-manual` — the manual dispatch invocation wrapper
3. stcortex consumer registration (happens at binary startup automatically, every 7 days for freshness)

The absence of a devenv batch slot also means the standard `devenv status` output will not include `workflow-trace`. This is correct and expected. Do not treat its absence as a deployment failure.

---

## Phase 5A — Binary Deploy (Day 28-30)

### Goal

Release binaries for `wf-crystallise` and `wf-dispatch` are built from the project root, verified against the full quality gate, materialised to `~/.local/bin/`, and smoke-tested via CLI invocation. The atuin trajectory records this as the "first binary materialisation" event for Phase 5A.

### Pre-conditions from Phase 4

Before executing 5A, confirm these Phase 4 outputs are present:

- 4-stage quality gate (check → clippy → pedantic → test) fully clean, including `cargo test --workspace --all-targets --all-features`
- `tests/ember_gate.rs` passing (all user-facing strings through the 7-trait rubric; no Rejected or Held-as-fail per W3 flag)
- m8 build prereq: `POVM_CR2_DEPLOYED=1` set in the environment (CR-2 `e2a8ed3` + CR-2b `76ea4d6` verified live on `:8125`)
- m10 Ember gate: Watcher §5.1 amendment status confirmed (if Held verdicts still treated as CI-fail, ensure all strings pass)
- Git tag from Phase 4 hardening milestone (e.g., `v0.1.0-hardened`) present in `git log`

### Build command sequence

```bash
# Step 1: Navigate to project root (always use absolute path)
cd /home/louranicas/claude-code-workspace/the-workflow-engine

# Step 2: Verify m8 gate is set — compile will error without this
export POVM_CR2_DEPLOYED=1

# Step 3: Set CARGO_TARGET_DIR (Forge trap 8 — workspace handling)
# workflow-trace is a single-crate project (ORAC pattern, not LCM workspace).
# Use per-project target dir to avoid contaminating habitat's shared /tmp/cargo dirs.
export CARGO_TARGET_DIR=/home/louranicas/claude-code-workspace/the-workflow-engine/target

# Step 4: Final gate pass before release build
# (Forge trap 8 — always build from crate root, not a subdirectory)
cargo check 2>&1 | tail -10
cargo clippy -- -D warnings 2>&1 | tail -10
cargo clippy -- -D warnings -W clippy::pedantic 2>&1 | tail -10
cargo test --lib --release 2>&1 | tail -20

# Step 5: Release build — both binary targets
cargo build --release \
  --bin wf-crystallise \
  --bin wf-dispatch \
  2>&1 | tail -10

# Step 6: Verify binary sizes (non-zero is the minimum bar)
ls -lh "$CARGO_TARGET_DIR/release/wf-crystallise"
ls -lh "$CARGO_TARGET_DIR/release/wf-dispatch"
```

### Binary materialisation

```bash
# Forge trap 1 — ALWAYS /usr/bin/cp -f, never bare cp (aliased to cp -i in this env)
/usr/bin/cp -f \
  "$CARGO_TARGET_DIR/release/wf-crystallise" \
  "$HOME/.local/bin/wf-crystallise"

/usr/bin/cp -f \
  "$CARGO_TARGET_DIR/release/wf-dispatch" \
  "$HOME/.local/bin/wf-dispatch"

# Verify both are non-zero-size and executable
[[ -s "$HOME/.local/bin/wf-crystallise" ]] \
  && echo "OK: wf-crystallise materialised" \
  || echo "ERROR: wf-crystallise is empty or missing"

[[ -s "$HOME/.local/bin/wf-dispatch" ]] \
  && echo "OK: wf-dispatch materialised" \
  || echo "ERROR: wf-dispatch is empty or missing"
```

### Smoke verification

```bash
# Forge trap 5 (health path variance for CLI):
# The health probe for CLI binaries is --version (exit 0 + semver output).
# There is NO /health HTTP endpoint. Do not curl localhost for this binary.

wf-crystallise --version
# Expected: "wf-crystallise 0.1.0" (or matching Cargo.toml version)

wf-dispatch --help
# Expected: help text exits 0, lists subcommands

# Smoke: crystalliser can reach its SQLite DB
wf-crystallise status --brief
# Expected: JSON or human-readable status line, exit 0
# Failure mode: if m7 SQLite DB path is misconfigured, this is the earliest signal
```

### Atuin trajectory log

```bash
# Record the materialisation event in atuin KV for cross-tool provenance
# (S1002029 substrate learning: atuin is the ONLY cross-tool provenance)
atuin kv set "workflow_trace.deploy.phase5a.timestamp" "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
atuin kv set "workflow_trace.deploy.phase5a.wf_crystallise_sha" \
  "$(sha256sum "$HOME/.local/bin/wf-crystallise" | cut -c1-16)"
atuin kv set "workflow_trace.deploy.phase5a.wf_dispatch_sha" \
  "$(sha256sum "$HOME/.local/bin/wf-dispatch" | cut -c1-16)"
atuin kv set "workflow_trace.deploy.phase5a.status" "binary_materialised"
```

### 8 forge trap mitigations for Phase 5A

| # | Trap | 5A Mitigation |
|---|------|---------------|
| 1 | `cp` alias (`cp -i`) | `/usr/bin/cp -f` at every materialisation step — hardcoded path, never bare `cp` |
| 2 | `pkill` exit 144 in `&&` chains | No port, no prior daemon to kill. If somehow needed: use `ss -tlnp` enumeration, never `pkill` in a gate chain |
| 3 | SIGPIPE on stdout | CLI binaries write to stdout by design. Cron context: ensure cron entry redirects stdout+stderr to a log file (`>> /tmp/wf-crystallise.log 2>&1`). Never pipe the binary output directly in cron without exit-code capture |
| 4 | Port detection | Not applicable — no port. Explicitly documented: do not look for `localhost:PORT/health`; use `wf-crystallise --version` as the sole CLI health probe |
| 5 | Health path variance | CLI health probe = `wf-crystallise --version` (exit 0 + semver). No HTTP path variant exists. This is not an omission; it is the design |
| 6 | Feature flags | Build with `--features default` (api + monitoring enabled per Cargo.toml defaults). `intelligence` and `evolution` features are gated for M2+. Do not pass `--features full` if that activates unimplemented M2 module stubs |
| 7 | Stale PIDs | CLI binaries do not write PID files. Overlap risk (cron invoking while prior run active): mitigated in 5C with `flock`-based lockfile on cron entry |
| 8 | Workspace handling | `CARGO_TARGET_DIR=./target` set explicitly; build from project root (`/home/louranicas/claude-code-workspace/the-workflow-engine`); `Cargo.toml` is at root; no workspace parent to confuse the resolver |

### Failure modes and Watcher flag classes

**Class B — hand-off boundary crossing:** Phase 5A materialisation is the first real hand-off from the build environment to the production binary store (`~/.local/bin/`). Watcher should record a Class-B entry when the materialisation command completes successfully.

**Class E — ancestor-rhyme:** If Phase 5A is delayed (e.g., pre-conditions from Phase 4 are not met, or the gate fails), and this delay extends more than 48 hours, Watcher should record a Class-E entry. The planning-to-code ratio pattern that killed both ancestor projects (`loop-workflow-engine-project`, `habitat-loop-engine`) includes delays in materialisation. Each day past the Phase 4 close date that does not produce a binary is a small step toward the ancestor-rhyme death.

**No Class F:** Class F (AP24 violation — code before G9) does not apply at Phase 5A. G9 has fired; this is post-code territory.

### Substrate observability at 5A

- stcortex consumer freshness: confirm `wf-crystallise status` shows `consumer_registered: true`. The binary registers the stcortex consumer at startup. If registration fails (stcortex `:3000` unreachable), the binary falls back to offline JSONL buffer — this is designed behavior per m13's Hebbian backpressure protocol, but it means substrate writes are deferred, not dropped.
- POVM cutover countdown: at Day 28 (approximately 2026-06-12 if Day 0 = 2026-05-15), POVM deprecation is T-28 days. The m8 `povm_calibrated` gate must be active; `POVM_CR2_DEPLOYED=1` must be in the build environment. Do not skip this verification.
- Conductor Wave maturity: `auto_start=false` for Waves 1B/1C/2/3. m32 dispatch will refuse with `DispatchError::ConductorDispatchDisabled` if Conductor is not live. Confirm Luke has executed `devenv start weaver zen enforcer` at the terminal before proceeding to Phase 5B production dispatch.

---

## Phase 5B — Production Cutover Ceremony (Day 30)

### Goal

The production-live signal is formally emitted, the first real crystallisation run executes against live substrate, the first dispatch smoke test runs through Conductor, and all four substrate surfaces confirm a write. Watcher records a Class-A entry (activation transition). The soak clock starts.

### Pre-cutover snapshot

Before Luke announces production-live, take a snapshot of current substrate state to serve as the D0 baseline for m14 lift measurement. This snapshot is the denominator for the 120-day evaluation:

```bash
# Capture baseline metrics (atuin KV + manual record)
# 1. Test counts at cutover
wf-crystallise status --json | python3 -c "
import sys, json; d = json.load(sys.stdin)
print('module_test_count:', d.get('module_test_count', 'unknown'))
print('schema_version:', d.get('schema_version', 'unknown'))
"

# 2. Substrate LTP density at D0 (from POVM live on :8125)
curl -s http://localhost:8125/health | python3 -c "
import sys, json; d = json.load(sys.stdin)
print('substrate_LTP_density:', d.get('substrate_ltp_density', 'unknown'))
print('learning_health:', d.get('learning_health', 'unknown'))
"

# 3. m14 lift baseline: expect None (n < 20 at D0 — F2 gate correct)
# This is the expected state. Do not treat None as an error.
# Record the n=0 baseline explicitly:
atuin kv set "workflow_trace.soak.d0.lift_n" "0"
atuin kv set "workflow_trace.soak.d0.lift_value" "None"
atuin kv set "workflow_trace.soak.d0.substrate_ltp_density" "<value from above>"
atuin kv set "workflow_trace.soak.d0.timestamp" "$(date -u +%Y-%m-%dT%H:%M:%SZ)"

# 4. Cluster H circuit breaker states — confirm Closed at D0
# (m40/m41/m42 circuit breakers: Closed → Open → HalfOpen; 5 failures → Open; 60s → HalfOpen)
wf-crystallise status --json | python3 -c "
import sys, json; d = json.load(sys.stdin)
cb = d.get('circuit_breakers', {})
for k, v in cb.items():
    print(f'circuit_breaker.{k}: {v}')
"
```

### Cutover signal

Luke/Command announces production-live. This is a deliberate ceremony, not an automatic state transition. The signal must be explicit. A TTY command or a session note is sufficient:

```
"workflow-trace production-live — Day 30 — soak clock starts now"
```

Watcher records a **Class-A entry** timestamping this announcement verbatim. The timestamp becomes `D30` for soak calculations.

### WCP notice to Watcher

Immediately after the production-live signal, Command (Tab 1) drops a WCP notice at:

```
~/projects/shared-context/watcher-notices/YYYY-MM-DDTHHMMSS_workflow_trace_production_live.md
```

Content of the notice:

```markdown
# WCP Notice — workflow-trace Production-Live (D30)

**From:** Command (Tab 1 Orchestrator)
**To:** Watcher ☤
**Date:** YYYY-MM-DDTHHMMSS UTC
**Event:** Phase 5B cutover ceremony complete. Soak clock started.

Watcher carriage handoff: T0 was Watcher's observation baseline. T+30d production observation begins now.

Watcher's deployment-watch journal continues append-only through D120. Weekly synthesis cadence begins at D37 (first synthesis after 7 days of production runs).

Substrate D0 baseline recorded to atuin KV (`workflow_trace.soak.d0.*`).

m14 lift: None (n=0 at D0). Expected. F2 gate correct.
POVM cutover: T-28d from today (2026-07-10 deadline). m42 dual-path active.
Conductor: `auto_start=false` — Luke brings up Weaver/Zen/Enforcer at terminal.
```

Do not rely on Luke to relay this notice. Drop it directly per `feedback_wcp_notify_weaver.md`.

### First production crystalliser run

```bash
# First real crystalliser sweep — covers last 7 days of substrate observation
# (even if the binary was just deployed, m1-m3 can read historic substrate data)
wf-crystallise sweep \
  --window-days 7 \
  --output-format json \
  >> /tmp/wf-crystallise-d30-first-sweep.log 2>&1

# Check exit code before interpreting output
echo "exit: $?"

# Expect: structured JSON with n=0 or very low n for workflow_runs
# (no workflow runs have happened yet; the sweep captures observation grain)
# m14 will return None for lift (n < 20) — this is correct
```

### First production dispatch (synthetic test workflow)

```bash
# Pre-condition: Conductor live (Luke has run devenv start weaver zen enforcer)
curl -s http://localhost:8141/health | python3 -c "import sys,json;d=json.load(sys.stdin);print('conductor:', d.get('status','unknown'))"

# If Conductor is not live, dispatch MUST NOT proceed.
# m32 will return DispatchError::ConductorDispatchDisabled — this is correct behavior.
# Do not attempt to route around it.

# First dispatch — synthetic read-only test workflow from m30 bank
# (requires at least one AcceptedWorkflow in the bank from Phase 4 load testing)
wf-dispatch execute \
  --workflow-id <synthetic_test_workflow_id> \
  --dry-run \
  2>&1 | tee /tmp/wf-dispatch-d30-first-dry-run.log

# After dry-run confirms gate sequence (m32 5-check: Conductor live → m33 TTL fresh →
# definition_hash match → sunset guard → dispatch cooldown):
wf-dispatch execute \
  --workflow-id <synthetic_test_workflow_id> \
  2>&1 | tee /tmp/wf-dispatch-d30-first-live.log
```

### Substrate touch confirmed (four surfaces)

Phase 5B is not complete until all four substrate surfaces show a write from this session:

```bash
# Surface 1: stcortex (primary post-2026-07-10 substrate)
# m13 writes to stcortex at namespace workflow_trace_* on crystalliser run
# Verify consumer is fresh and wrote:
~/.local/bin/stcortex sql "SELECT namespace, COUNT(*) FROM pathway WHERE namespace LIKE 'workflow_trace_%' GROUP BY namespace;"

# Surface 2: POVM (overlap period — read-write until 2026-07-10)
# m42 writes via POST /reinforce under workflow_trace_* prefix
curl -s http://localhost:8125/pathways?namespace=workflow_trace | python3 -c "
import sys, json; d = json.load(sys.stdin)
print('povm pathways workflow_trace:', len(d.get('pathways', [])))
"

# Surface 3: SYNTHEX inbound event (m40 fan-out to :8092/v3/nexus/push)
# Verify the outbox JSONL has been written (fallback if HTTP fired-and-forgotten)
ls -la /home/louranicas/claude-code-workspace/the-workflow-engine/outbox/ 2>/dev/null || echo "outbox empty or not created"

# Surface 4: LCM if deploy-shape step executed (m41 via lcm.loop.create)
# Only if the dispatched workflow included a deploy-shaped step.
# Verify via LCM log or MCP stdio channel — not required if workflow is read-only shape
```

### 8 forge trap mitigations for Phase 5B

| # | Trap | 5B Mitigation |
|---|------|---------------|
| 1 | `cp` alias | No copies in Phase 5B. Binaries already materialised in 5A |
| 2 | `pkill` exit 144 | Not applicable. No port to kill. If Conductor must be restarted, use `devenv restart weaver` (Luke @ terminal), not pkill from this agent |
| 3 | SIGPIPE | First sweep log piped via `tee` — both stdout and file; exit code captured separately with `$?` check |
| 4 | Port detection | Conductor health probe: `curl -s http://localhost:8141/health`. This is the ONLY port probe in Phase 5B. workflow-trace binaries: no port |
| 5 | Health path variance | Conductor health path: `/health` on `:8141`. SYNTHEX v2: `:8092/health`. POVM: `:8125/health`. LCM: `:8082/health` (if relevant). All differ from the generic `/health` — check each service's devenv.toml entry before curling |
| 6 | Feature flags | No re-build in 5B. Binary shipped in 5A with correct features. If a re-build is needed (hotfix), re-run 5A build step with same `--features default` flag |
| 7 | Stale PIDs | No PIDs for this binary. If Conductor shows stale PID from a previous run, use `ss -tlnp "sport = :8141"` to find the actual process, not the PID file |
| 8 | Workspace handling | No re-build in 5B. If needed: same `CARGO_TARGET_DIR` and project root as 5A |

### Failure modes and Watcher flag classes

**Class B (hand-off boundary):** The cutover ceremony is the single most important hand-off in Phase 5. Watcher timestamps it verbatim. If the ceremony is skipped (binaries deployed but no explicit production-live announcement), Watcher flags it as an anomalous Class-B boundary — the soak clock has no official start, making D60/D90/D120 calculations ambiguous.

**Class A (activation transition):** Production-live is an activation transition. Watcher Class-A entry required.

**Class I (Hebbian silence check):** At the first sweep, confirm that m40 wrote at least one `WorkflowEvent` to the outbox (even if the HTTP fire-and-forget to SYNTHEX timed out). If the outbox is empty after the first sweep, Cluster H is silent from Day 1 — flag immediately. This does not block 5B progress; it is a warning that the learning loop may be decorative.

---

## Phase 5C — 120-day Soak (D30 → D120)

### Goal

Sustained observation of workflow-trace in production. All three structural loops (CC-3 evidence-driven iteration, CC-5 substrate learning, CC-7 pressure-driven evolution) are monitored. The m14 lift metric accumulates toward statistical significance (n≥20 per workflow, n≥100 aggregate for meaningful CI bars). The m11 decay clock ticks. Watcher synthesises weekly. D60 and D90 checkpoints gate Phase 6 entry.

### Continuous m14 lift measurement

m14's `habitat_outcome_lift` metric accumulates silently until n≥20. During the early soak (D30-D50 depending on crystalliser frequency), expect `None` returns from m14 — this is correct per the F2 hard gate. Do not interpret `None` as zero lift.

Track the accumulation via atuin KV on each weekly check:

```bash
# Weekly lift check (add to wf-crystallise-weekly-check atuin script)
wf-crystallise status --json | python3 -c "
import sys, json, datetime
d = json.load(sys.stdin)
lift = d.get('habitat_outcome_lift', {})
n = lift.get('n', 0)
val = lift.get('lift', None)
ci = lift.get('ci_half', None)
ts = datetime.datetime.utcnow().isoformat()
print(f'[{ts}] lift_n={n} lift={val} ci_half={ci}')
"

# Record to atuin KV by day count
atuin kv set "workflow_trace.soak.d$(date -d 'D30_date' +%j).lift_n" "<n>"
atuin kv set "workflow_trace.soak.d$(date -d 'D30_date' +%j).lift_value" "<val>"
```

The Wilson 95% CI returned by m14 is the canonical uncertainty bound. When CI bars span zero (lower bound negative), lift is not statistically confirmed even if the point estimate is positive. m11's sunset extension decision requires both the point estimate and the CI lower bound to be positive.

### Continuous m11 decay loop monitoring

m11's consolidation cycle runs nightly (or at each `wf-crystallise sweep` invocation if configured for session-close trigger rather than cron). Monitor decay via:

```bash
# Check decay cycle count for each workflow in the bank
wf-crystallise bank --show-decay-cycles
# Expected output: table of workflow_id, weight, decay_cycles, sunset_at, phase

# Healthy state: weights declining slowly (compound_signal above 0.5 means decay_factor ≈ 0.99)
# Warning state: weights declining rapidly (compound_signal near 0.0 means decay_factor = 0.98)
# Alert state: weight < 0.1 (PrunePending phase) after only D30-D60 — investigate frequency/fitness signals
```

The 120-day calibration: a zero-signal workflow (frequency=0, fitness=0, recency=0) reaches prune threshold (0.01) in approximately 228 cycles at plain_decay_rate=0.02. At daily cycles, this is 228 days — well past the 120-day explicit `sunset_at` hard boundary. The decay is the gradient; the hard boundary is the law. Both are in play simultaneously.

### Continuous m15 pressure register monitoring

m15 emits `PHASE-B-RESERVATION-NOTICE-*.jsonl` files to `~/projects/shared-context/agent-cross-talk/`. Monitor for accumulation:

```bash
# Check for new pressure notices since last weekly check
ls -lt ~/projects/shared-context/agent-cross-talk/PHASE-B-RESERVATION-NOTICE-*.jsonl \
  2>/dev/null | head -10

# Count by forbidden category
cat ~/projects/shared-context/agent-cross-talk/PHASE-B-RESERVATION-NOTICE-*.jsonl \
  2>/dev/null | python3 -c "
import sys, json
from collections import Counter
cats = Counter()
for line in sys.stdin:
    try:
        ev = json.loads(line)
        cats[ev.get('forbidden_category', 'unknown')] += 1
    except Exception:
        pass
for k, v in cats.most_common():
    print(f'{v:3d}  {k}')
"
```

Any forbidden-verb pressure event targeting `recommend_*`, `auto_*`, or `http_server_surface` categories is a signal that scope-creep pressure is active. Three or more events in a 7-day window should prompt Watcher to initiate a spec amendment interview. m15 does not gate anything — it witnesses. Watcher and Zen decide whether the pressure warrants response.

### cron scheduling and atuin trajectory

`wf-crystallise` sweeps should run on a regular schedule to accumulate the n≥20 runs required for m14's F2 gate. Recommended cadence:

```bash
# Daily crystalliser sweep (add to user crontab or atuin scripts)
# Note: use flock to prevent overlapping invocations (Forge trap 7 — stale PIDs / overlap)
# Note: NO set -e here — wf-crystallise may return non-zero on partial substrate unavailability;
#       handle explicitly (CLAUDE.md § Shell Scripting Conventions).
#
# /etc/cron.d/wf-crystallise or crontab -e entry:
# 0 6 * * * /usr/bin/flock -n /tmp/wf-crystallise.lock \
#   /home/louranicas/.local/bin/wf-crystallise sweep \
#   --window-days 1 \
#   >> /tmp/wf-crystallise.log 2>&1; \
#   echo "exit:$? ts:$(date -u +%Y-%m-%dT%H:%M:%SZ)" >> /tmp/wf-crystallise.log

# After each cron run, atuin records the invocation automatically via shell history.
# Verify trajectory is accumulating:
atuin history list --cwd /home/louranicas/claude-code-workspace/the-workflow-engine \
  | grep wf-crystallise | tail -10
```

The `flock -n` flag (non-blocking lock) means if a prior invocation is still running, the new invocation exits immediately without running. This is the correct behavior — accumulation skips one day rather than overlapping. A skipped invocation leaves no log entry; monitor for gaps in the log.

### Weekly Watcher synthesis trigger

Every 7 days from D30, Command drops a Watcher synthesis trigger in the WCP channel. The Watcher's synthesis covers:

- New Class-E flags (planning sprawl; any new architectural proposals that inflate the planning-to-code ratio without corresponding code)
- Class-I status (Hebbian silence; is `learning_health` moving? Are m42 reinforce calls appearing in atuin history?)
- New pressure notices from m15 (scope-creep accumulation)
- m14 lift trajectory (n accumulation; first CI bars; direction of point estimate)
- m11 decay cycle state (any workflows entering PrunePending prematurely?)
- Cluster H circuit breaker state (any Opens in the 60s → HalfOpen cycle that indicate sustained substrate unavailability?)

Watcher synthesis output goes to `the-workflow-engine/WATCHER_DEPLOYMENT_WATCH_JOURNAL_S1001982.md` as a new tick entry. It does not go to Luke as a report — Watcher journals, Command relays to Luke if Luke asks.

The synthesis trigger WCP notice format:

```
~/projects/shared-context/watcher-notices/YYYY-MM-DDTHHMMSS_workflow_trace_week_N_synthesis_request.md
```

Where N is the week number from D30.

### Monthly substrate health check

Every 30 days from D30, a more thorough substrate review:

```bash
# LTP/LTD ratio from POVM (read-only during overlap; stcortex post-2026-07-10)
curl -s http://localhost:8125/health | python3 -c "
import sys, json; d = json.load(sys.stdin)
print('substrate_ltp_density:', d.get('substrate_ltp_density', 'unknown'))
print('learning_health:', d.get('learning_health', 'unknown'))
print('ltp_ltd_ratio:', d.get('ltp_ltd_ratio', 'unknown'))
"

# Healthy Hebbian band: LTP/LTD > 1.5 (target 1.5-4.0).
# Current substrate at D0: 0.043 (35x below target). Monitor monthly.
# If LTP/LTD does not move over 60 days, Watcher Class-I: confirmed chronic.

# m11 decay cycle counts across the full bank
wf-crystallise bank --show-decay-cycles --format tsv \
  | column -t

# Class-I diagnostic: are m42 reinforce calls appearing in atuin history?
atuin history list | grep "POST.*reinforce" | wc -l
# or
atuin history list | grep "wf-dispatch" | wc -l
```

### D60 mid-soak review

At Day 60, Luke makes a deliberate decision: continue soak / amend spec / early sunset.

The decision criteria:

- m14 lift: is n≥20 achieved? If not, is the crystalliser schedule accumulating runs at a pace that will reach n=20 before D90?
- m14 CI direction: if lift is `Some`, is the CI lower bound positive? (Confirmed lift vs. ambiguous.)
- m11 decay: are any workflows in PrunePending phase at D60? If so, is this expected (workflows that ran zero times) or anomalous?
- Watcher Class-I status: has `learning_health` moved at all since D0? If no movement in 60 days, Cluster H is structurally decorative — this is a major finding that should inform the D120 verdict.

```bash
# D60 snapshot to atuin KV
atuin kv set "workflow_trace.soak.d60.lift_n" "<current n>"
atuin kv set "workflow_trace.soak.d60.lift_value" "<current lift or None>"
atuin kv set "workflow_trace.soak.d60.lift_ci_lower" "<ci_lower or None>"
atuin kv set "workflow_trace.soak.d60.substrate_ltp_density" "<current value>"
atuin kv set "workflow_trace.soak.d60.class_i_active" "<true/false>"
atuin kv set "workflow_trace.soak.d60.timestamp" "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
```

Luke's decision at D60 is recorded to atuin KV and to the Watcher journal:

```bash
# Record Luke D60 decision (one of: continue | amend_spec | early_sunset)
atuin kv set "workflow_trace.soak.d60.luke_decision" "<decision>"
```

### D90 pre-sunset preview

At Day 90, the m14 evidence aggregator produces a pre-sunset verdict. This is not a binding decision — it is a 30-day forecast:

```bash
# D90 lift snapshot
wf-crystallise report --format d90-preview \
  >> /tmp/wf-crystallise-d90-preview.log 2>&1

# m11 sunset evaluation preview: what would D120 verdict be if the D90 trend continues?
# Expected outputs:
# PASS: lift > threshold, CI lower bound positive, n >= 20
# FAIL: lift below threshold or CI spanning zero at n >= 20
# DEGRADED: CI spanning zero at n >= 100 (too noisy; signal unclear)
# INSUFFICIENT_DATA: n < 20 still at D90 (crystalliser schedule problem)
```

If the D90 preview shows FAIL or DEGRADED, Luke should be notified immediately. A D90 FAIL with 30 days remaining gives time to investigate and potentially adjust the crystalliser schedule or the m14 lift threshold (if the threshold was set too aggressively at G5).

### D120 sunset evaluation

D120 is the m11 `sunset_at` hard boundary for the first cohort of workflows. The evaluation is not automatic — it requires Luke's explicit verdict. The m11 engine enforces the boundary by excluding `SunsetExpired` workflows from dispatch; it does not delete them unilaterally.

The D120 evaluation inputs:

1. m14 `habitat_outcome_lift` at n = (total runs across soak); Wilson CI; point estimate vs. threshold (TBD at G5 interview, suggested 0.05 minimum with n≥20 + CI bars)
2. Watcher 90-day synthesis summary (12 weekly entries folded into a soak-arc synthesis)
3. m11 bank state: how many workflows reached SUNSET_EXPIRED vs. ACTIVE vs. PRUNE_PENDING
4. Substrate state at D120: LTP/LTD ratio trajectory; learning_health movement; stcortex consumer freshness

The possible D120 verdicts and their mechanical consequences:

| Verdict | Criterion | Consequence |
|---------|-----------|-------------|
| PASS | lift ≥ threshold, CI lower bound > 0, n ≥ 20 | Phase 6: extend soak or full graduation; m11 sunset_at reset for next cohort |
| FAIL | lift < threshold OR CI lower bound ≤ 0 at n ≥ 20 | Phase 6: m11 startup-refusal activates; engine halts accepting new workflows |
| DEGRADED | CI spanning zero at n ≥ 100 | Phase 6: spec amendment interview; crystalliser schedule review |
| INSUFFICIENT_DATA | n < 20 at D120 | Soak extended; crystalliser schedule problem; not a verdict |

**m11 startup-refusal on FAIL:** if D120 verdict is FAIL, m11's startup check (modelled on `conductor_enforcement.rs` startup-refusal pattern) returns `EngineState::SunsetEnforced` and `wf-crystallise` exits with a non-zero code and a human-readable message naming the lift threshold, the observed lift, and the instruction to consult Luke. This is not a silent failure. The message is scored through the Ember gate (m10) before shipping.

### 8 forge trap mitigations for Phase 5C

| # | Trap | 5C Mitigation |
|---|------|---------------|
| 1 | `cp` alias | No binary copies in 5C under normal operation. If a hotfix binary is deployed: always `/usr/bin/cp -f`, never bare `cp` |
| 2 | `pkill` exit 144 | Cron scripts: capture exit codes explicitly. The `flock -n` wrapper exits 1 if lock is held — handle this as "already running, skip" not as "error, abort chain". Example: `flock -n /tmp/wf-crystallise.lock ... || echo "skip:lock-held" >> /tmp/wf-crystallise.log` |
| 3 | SIGPIPE | All cron invocations redirect stdout+stderr to a log file. No bare piping in cron context. If weekly synthesis scripts pipe `wf-crystallise` output to `jq`, check `${PIPESTATUS[0]}` for the binary's exit code separately from jq's |
| 4 | Port detection | No port on workflow-trace binaries. Substrate service probes (POVM `:8125`, SYNTHEX `:8092`, CONDUCTOR `:8141`) use their own well-known paths. If any substrate service is down, the crystalliser writes to its outbox JSONL buffer and continues — verify via outbox file count, not HTTP health code |
| 5 | Health path variance | Weekly substrate health checks must use correct paths: POVM `/health`, SYNTHEX v2 `/health`, ME v2 `/api/health`, Conductor (Weaver) `/health` on `:8141`. Do not use a generic `/health` without checking the service's devenv.toml entry |
| 6 | Feature flags | If a mid-soak hotfix requires a re-build with different feature flags, treat it as a mini Phase 5A: re-run the full gate and re-materialise with `/usr/bin/cp -f`. Never skip the gate for a "minor" hotfix. `S1001882 near-miss` discipline |
| 7 | Stale PIDs / cron overlap | `flock -n /tmp/wf-crystallise.lock` on every cron entry. If the lock file is stale (process died without releasing), `flock -n` will acquire it immediately on next invocation. Unlike PID-file approaches, flock releases automatically on process exit regardless of how the process exits |
| 8 | Workspace handling | If cargo is invoked for any reason during soak (hotfix, re-verification), set `CARGO_TARGET_DIR` explicitly before every invocation. The soak period spans months; the env var may not be set in a fresh shell. Wrap in: `CARGO_TARGET_DIR=/home/louranicas/claude-code-workspace/the-workflow-engine/target cargo ...` |

---

## m14 lift measurement as D120 sunset oracle

The m14 `habitat_outcome_lift` metric is the empirical basis for the m11 sunset decision. The formula:

```
habitat_outcome_lift = 0.6 × cascade_success_rate + 0.4 × cost_lift
```

where `cost_lift = (baseline_cost - observed_cost) / baseline_cost`, and both components carry Wilson 95% CI propagated from the cascade component (dominant term).

**Why this metric and not RALPH fitness directly?** RALPH fitness measures the substrate's self-evolution capacity. m14 measures the engine's contribution to Luke's actual work outcomes. These are correlated but distinct. A substrate in excellent Hebbian health (LTP/LTD = 3.0) does not automatically imply the engine is saving Luke decision-making cost. m14 is the user-facing oracle; RALPH fitness is the substrate's self-report.

**The G5 spec interview will set the threshold.** The suggested threshold (0.05 lift minimum with n≥20 + CI bars) is a placeholder from GOD_TIER_CONSOLIDATION. The actual threshold should be set via structured interview before D30, considering:
- What constitutes meaningful habitat-outcome improvement for Luke's workflow cadence?
- How does the 0.6/0.4 cascade/cost weighting align with Luke's priority?
- Is a 5% lift above baseline detectable with n=20 given the expected variance in cascade success?

Until G5 sets the threshold explicitly, the D60 and D90 milestone reports should show raw lift + CI alongside the threshold placeholder. Do not guess the threshold into the binary.

---

## m11 RALPH fitness-weighted decay during soak

m11's consolidation loop runs on each `wf-crystallise sweep` invocation (or nightly cron, depending on configuration). During the soak, observe the decay trajectory of early-ingested workflows:

**Expected healthy decay trajectory (first 60 days):**
- D30: weight ≈ 1.0 (just accepted into bank)
- D60: weight ≈ 0.97-0.99 (some decay, but frequency × fitness × recency still high if the workflow is being dispatched)
- D90: weight ≈ 0.94-0.98 (gradual decline or near-stable if compound_signal > 0.5)
- D120: weight ≈ 0.90-0.97 (approaching explicit sunset_at boundary; m11 will mark SUNSET_EXPIRED)

**Warning trajectory (workflow never dispatched):**
- D60: weight ≈ 0.30 (frequency=0, fitness=initial, recency declining fast)
- D90: weight ≈ 0.08 (entering PrunePending; compound_signal near 0)
- D120: weight ≈ 0.02 (at prune threshold; near death)

A workflow that was accepted into the bank but never dispatched in 90 days is correctly approaching death. This is the structural primitive working as designed. Do not rescue it without evidence.

**Watcher monitoring role:** If all workflows in the bank show the warning trajectory simultaneously (no dispatches in 30+ days), this is a Class-I signal: the dispatch path is broken or unused. Watcher flags it. m15 may have recorded pressure notices for alternative dispatch methods — check the `agent-cross-talk/` directory.

---

## POVM deprecation 2026-07-10 falls within the soak window

If Day 0 deployment is approximately 2026-06-15, then D30 ≈ 2026-07-15 — five days after the POVM deprecation deadline. In practice, the POVM cutover at 2026-07-10 falls inside the soak window at approximately D25.

This means:

- Phase 5A and 5B execute while POVM is in the read-write overlap period. m42 writes to both stcortex (via m13) and POVM (via the dual-path).
- Approximately at D25, the POVM deprecation activates. m42's `povm_overlap_active` flag flips to `false` — m42 routes exclusively through stcortex-via-m13.
- The transition must be verified: confirm that after 2026-07-10, m42's atuin history shows no more `POST .../reinforce` calls to `:8125` and that stcortex shows new pathway writes under `workflow_trace_*`.

```bash
# Verify POVM cutover transition (run on 2026-07-11 or later)
atuin kv get "workflow_trace.povm_overlap_active"
# Expected after cutover: "false"

# Confirm no POVM writes after cutover date
atuin history search --cwd /home/louranicas/claude-code-workspace/the-workflow-engine \
  --after "2026-07-10" | grep "8125/pathways" | wc -l
# Expected: 0

# Confirm stcortex pathway count increasing
~/.local/bin/stcortex sql "SELECT COUNT(*) FROM pathway WHERE namespace LIKE 'workflow_trace_%';"
```

**Critical anti-pattern:** do NOT silently fall back to POVM if stcortex `:3000` is unreachable. m13's Hebbian backpressure protocol writes to the local JSONL buffer and defers the stcortex write — it does not re-route to POVM. This is the designed behavior (CLAUDE.md § Memory Systems row 8). If stcortex is consistently unreachable during the soak, that is a substrate infrastructure issue to escalate, not a fallback condition.

---

## Substrate observability during soak

### stcortex consumer freshness

The stcortex DB layer refuses writes from stale consumers (no fresh registration in 7 days). The `wf-crystallise` binary registers its consumer at startup on every invocation. Because the cron schedule runs daily, consumer freshness is maintained automatically.

Verify weekly:

```bash
~/.local/bin/stcortex consumers | grep workflow_trace
# Expected: workflow-trace-crystalliser with last_seen < 24h
```

If the consumer shows `stale: true`, the cron schedule missed an invocation. Investigate the lock file and cron log before the next substrate write attempt.

### POVM cutover countdown

```bash
# During overlap period (before 2026-07-10):
python3 -c "
import datetime
cutover = datetime.date(2026, 7, 10)
today = datetime.date.today()
days = (cutover - today).days
print(f'POVM deprecation: T-{days}d ({cutover})')
"
```

Track weekly. When T-7d or less, run the POVM cutover verification commands above proactively.

### Conductor Wave maturity

HABITAT-CONDUCTOR Waves 1B/1C/2/3 are installed with `auto_start=false`. Dispatch operations via m32 require Conductor to be live. During the soak:

- If Conductor is not live when `wf-dispatch execute` is attempted, m32 returns `DispatchError::ConductorDispatchDisabled` and the run does not execute. This appears in the crystalliser log and reduces n accumulation toward m14's F2 gate.
- If Conductor availability is consistently low (e.g., Luke's terminal session does not have Weaver/Zen/Enforcer running), n accumulation for m14 will be slow. This may lead to `INSUFFICIENT_DATA` at D120. Track Conductor uptime informally.

### Ember §5.1 amendment status

m10's Ember gate treats `Held` verdicts as CI failures (W3 flag) until Watcher amends §5.1 for service adoption contexts. If the amendment lands during the soak period, update the `tests/ember_gate.rs` CI behavior accordingly (relax the Held-as-fail to Held-as-warning) and re-run Phase 4 quality gate to confirm clean. No re-build of the deployed binary is required unless the source changes from this amendment.

---

## Watcher carriage extends through D120

The Watcher's deployment-watch journal (`WATCHER_DEPLOYMENT_WATCH_JOURNAL_S1001982.md`) extends append-only through the full soak. The carriage handoff language in the Phase 5B WCP notice is intentional: T0 was Watcher's baseline capture; T+30d (D30) is the production-live ceremony; the 90-day soak is Watcher's longest continuous observation window in this project's history.

Watcher's synthesis cadence during soak:

- **Weekly:** journal tick entry with lift n, Hebbian silence status, pressure notices, decay trajectory
- **Monthly:** deeper substrate review; LTP/LTD ratio; stcortex consumer health; Conductor availability
- **D60:** mid-soak synthesis contributes to Luke's D60 decision input
- **D90:** pre-sunset preview synthesis (Watcher's probabilistic forecast, not a binding verdict)
- **D120:** soak-arc synthesis (90 days of accumulated entries folded into a narrative of what the engine actually did vs. what the spec predicted)

Watcher does not initiate synthesis autonomously. Each weekly trigger requires a Command WCP notice. If Luke does not prompt the synthesis, the journal accumulates tick entries without folding. The fold is the value-add; unfired prompts lose it.

---

## Hand-off to Phase 6 — Sunset Evaluation and Cross-Cutting

Phase 5C terminates at D120. The hand-off to Phase 6 requires:

1. m11 D120 verdict (PASS / FAIL / DEGRADED / INSUFFICIENT_DATA) recorded to atuin KV
2. Watcher 90-day soak-arc synthesis complete and journal tick D120 committed to canonical path
3. m14 final lift snapshot with CI bars printed to `/tmp/wf-crystallise-d120-final.log`
4. m11 bank state at D120: count of ACTIVE / PRUNE_PENDING / SUNSET_EXPIRED workflows
5. Substrate state: LTP/LTD ratio at D120 vs. D0 baseline; movement or stasis

```bash
# D120 hand-off package (run before Phase 6)
atuin kv set "workflow_trace.soak.d120.verdict" "<PASS|FAIL|DEGRADED|INSUFFICIENT_DATA>"
atuin kv set "workflow_trace.soak.d120.lift_n" "<n>"
atuin kv set "workflow_trace.soak.d120.lift_value" "<lift or None>"
atuin kv set "workflow_trace.soak.d120.lift_ci_lower" "<ci_lower or None>"
atuin kv set "workflow_trace.soak.d120.bank_active" "<count>"
atuin kv set "workflow_trace.soak.d120.bank_prune_pending" "<count>"
atuin kv set "workflow_trace.soak.d120.bank_sunset_expired" "<count>"
atuin kv set "workflow_trace.soak.d120.timestamp" "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
```

Phase 6 reads these KV values as its first gate. If `workflow_trace.soak.d120.verdict` is absent, Phase 6 does not proceed — D120 soak has not completed.

---

## Rollback procedure

If the soak fails early (sustained Cluster H silence + FAIL trajectory at D60 + Luke decision to halt):

```bash
# Step 1: Remove binaries from ~/.local/bin/
rm -f ~/.local/bin/wf-crystallise
rm -f ~/.local/bin/wf-dispatch

# Step 2: Deregister stcortex consumer
# The consumer registration expires after 7 days of no invocation (stcortex auto-stale).
# No explicit deregistration call is needed — absence of invocation is sufficient.
# If immediate deregistration is required:
~/.local/bin/stcortex call deregister_consumer '{ "consumer_id": "workflow-trace-crystalliser" }'

# Step 3: Remove atuin cron entry (if added to crontab)
crontab -e
# Remove the wf-crystallise line

# Step 4: Remove atuin scripts (if added)
atuin scripts remove wf-crystallise-sweep 2>/dev/null || true
atuin scripts remove wf-dispatch-manual 2>/dev/null || true

# Step 5: Archive outbox JSONL (do not delete — outbox is the audit trail)
mv /home/louranicas/claude-code-workspace/the-workflow-engine/outbox/ \
   /home/louranicas/claude-code-workspace/the-workflow-engine/outbox-rollback-$(date -u +%Y%m%d)/

# Step 6: Record rollback to atuin KV
atuin kv set "workflow_trace.rollback.timestamp" "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
atuin kv set "workflow_trace.rollback.reason" "<Luke-stated reason>"
atuin kv set "workflow_trace.rollback.day_of_soak" "<day count when halted>"
```

No devenv.toml entry was created, so no devenv rollback is needed. No port was occupied, so no port cleanup is needed. The rollback is clean.

---

## Phase 5 at a glance

| Sub-phase | Days | Key action | Gate |
|-----------|------|-----------|------|
| 5A | 28-30 | Build + materialise | Full quality gate + smoke `--version` |
| 5B | 30 | Cutover ceremony | Class-A (Watcher), WCP notice, four-surface substrate touch |
| 5C | 30-120 | 90-day soak | Weekly synthesis, D60 decision, D90 preview, D120 verdict |
| Hand-off | 120 | Phase 6 trigger | D120 verdict + atuin KV package complete |

---

*Phase 5 authored 2026-05-17 (S1001982) · HOLD-v2 active · planning-only · G1-G9 gates required before any commands here are executable*

*Back to: [[../HOME]] · [[../GOD_TIER_CONSOLIDATION_S1001982]] · [[phase-4-pre-deploy-hardening]]*
