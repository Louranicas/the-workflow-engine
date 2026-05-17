---
title: Phase 3 — Integration + Conductor Wiring (Days 21-26)
date: 2026-05-17 (S1001982)
kind: deployment-framework-recipe
status: planning-only · HOLD-v2 active · activates at G9 + Phase 2B complete
phase: 3 of 6 (Days 21-26 · 5 integration days)
authority: Luke @ node 0.A
emitter: Command (Tab 1 Orchestrator top-left)
watcher_flags_pre-positioned: B (cross-substrate handoff) · D (four-surface drift) · I (Hebbian silence)
---

# Phase 3 — Integration + Conductor Wiring (Days 21-26)

> Back to: [[../HOME]] · [[../GOD_TIER_CONSOLIDATION_S1001982]] · [[phase-2B-build-clusters-F-G-H]]
>
> Upstream: Phase 2B delivers built-and-tested Cluster G (m30/m31/m32/m33) + Cluster H (m40/m41/m42) with all modules gate-clean and substrate interfaces declared but not yet live-wired.
>
> Downstream: Phase 4 Hardening receives 5 integration tracks live, CC-5 loop closure measured, substrate snapshot taken, and the first Watcher Class-I flag either cleared or promoted to an open finding.

---

## Phase overview

Phase 3 wires the workflow engine into the living habitat across five integration tracks in parallel. By Day 26, every Cluster H module is communicating with an external substrate; the CC-5 substrate learning loop has closed for the first time (or the failure to close is a measured finding); and the dispatch wire to HABITAT-CONDUCTOR is either live (if Waves 1B/1C/2/3 have been started) or in refuse-mode with the blocker explicitly named.

**What does NOT happen in Phase 3:**
- No new module code is written. Modules are closed from Phase 2B.
- No service is started by Claude. Luke starts services at the terminal. Claude wires, tests, and observes.
- No POVM writes outside `workflow_trace_*`. AP30 is enforced at the m9 namespace-guard wire, not as a convention.
- No silent fallback from stcortex to POVM after 2026-07-10 cutover. The dual-path flag is explicit.

**Watcher observation posture:** Watcher observes all five integration tracks and will Class-B flag every cross-substrate handoff boundary crossing. The CC-5 loop closure attempt on Day 26 is the single most important observation event; Watcher timestamps it verbatim.

---

## Critical-path blocker — Conductor maturity gate

**This section gates all of Phase 3, Track 2. Read before planning any Day 22-23 work.**

As of 2026-05-17 (session S1001982), HABITAT-CONDUCTOR status per `CLAUDE.local.md` is:

| Wave | Status | Required for m32 |
|---|---|---|
| Wave 0/−1/0.5/0.75/1A | LIVE | Not blocking |
| Wave 1B (enforcer daemon) | BUILT, INSTALLED, **`auto_start=false`** | Enforcement gate active |
| Wave 1C (zen daemon) | BUILT, INSTALLED, **`auto_start=false`** | Zen review loop active |
| Wave 2 (WASM plugin) | BUILT, INSTALLED, **`auto_start=false`** | Zellij Conductor integration |
| Wave 3 (CONDUCTOR_ENFORCEMENT_ENABLED=1) | BUILT, not flipped | Rollback + audit-first writes |

**Current result:** m32 initialises in refuse-mode. Every `wf-dispatch dispatch` call returns `DispatchError::ConductorDispatchDisabled`. This is not a bug — it is the intended production behaviour until Luke executes the Wave bring-up sequence from a terminal.

**Luke's bring-up sequence (terminal, not Claude):**
```bash
devenv start weaver
devenv start zen
devenv start enforcer
curl -s http://127.0.0.1:8141/health
# After health PASS → Wave 2 WASM deploy → 24h NoOp soak
# → CONDUCTOR_ENFORCEMENT_ENABLED=1 in conductor.env
# → CONDUCTOR_DISPATCH_ENABLED=1 for wf-dispatch
```

**Phase 3 consequence:** Track 2 (m32 Conductor wiring) can be scaffolded, connection-probed, and integration-tested in dry-run mode during Days 22-23. Live dispatch integration tests require Conductor Wave 1B/1C to be started. Phase 3 can complete with m32 in refuse-mode; Track 2 is flagged as "integration scaffolded, live wire pending Luke Wave bring-up."

**Watcher Class-A pre-positioned:** when Luke starts Wave 1B and the Conductor health endpoint returns ok, this is a gate-flip activation transition. Watcher timestamps it verbatim per Class-A rubric. m32 transitions from `Dispatcher::refuse_mode` to `Dispatcher::live_mode` on the same session.

---

## Five integration tracks

---

### Track 1 — stcortex consumer registration (m2 + m9 + m13)

**Days 21-22 · Modules: m2, m9, m13 · Namespace: `workflow_trace_*`**

#### Prereqs

- stcortex running at `127.0.0.1:3000` (SpacetimeDB module, confirmed reachable)
- stcortex CLI at `~/.local/bin/stcortex` — status probe: `stcortex status`
- Namespace `the_workflow_engine` already exists (2 memories: id 16477 semantic + 16479 procedural, opened S1001971)
- m2 (`stcortex_consumer`) and m9 (`watcher_namespace_guard`) are built and gate-clean from Phase 2B
- m13 (`stcortex_writer_narrowed`) is built and gate-clean from Phase 2B

#### Refuse-write contract

stcortex enforces consumer presence at the database layer (not application layer). From `CONSUMER-ONBOARDING.md`:

```
Every write_pathway / write_memory call executes this reducer check:
  filter consumers where namespace = 'workflow_trace_*' AND stale = false
  if count == 0 AND namespace != 'scratch': return Err(530 refuse-write)
```

There is no way to soft-fail past this. A refused write returns HTTP 530 with a descriptive body. m13 must treat a 530 as an architectural signal (not a bug) and log at `ERROR` level, not `WARN`.

#### Wire-protocol contract

stcortex exposes two paths: the SpacetimeDB reducer call surface (via `stcortex call` CLI or Rust SDK) and a REST SQL query surface at `http://127.0.0.1:3000/v1/database/stcortex/sql`.

For consumer registration and write operations, the reducer path is authoritative:

```bash
# Registration (idempotent — safe to call on every m2 startup)
stcortex call register_consumer 'workflow-trace' 'workflow_trace_main' 'cli'

# Freshness maintenance (call during active sessions to prevent stale flag after 30d)
stcortex call access_memory <memory_id> 'workflow-trace'

# Write a pathway (after registration)
stcortex call write_pathway 'workflow_trace_<src>' 'workflow_trace_<dst>' \
  'workflow_trace_main' <weight_f64> '<session_id>' null

# Query pathway weights (m31 reads these during selection)
stcortex sql "SELECT pre_id, post_id, weight FROM pathway \
  WHERE namespace = 'workflow_trace_main' ORDER BY weight DESC LIMIT 20"
```

m13's Rust-side write pattern (following `subscriber_main.rs` from `02-stcortex-consumer/`):
- Transport: Rust SDK WebSocket for subscriptions; `stcortex call` (subprocess) for reducer calls
- Namespace convention: all IDs prefixed `workflow_trace_` (AP30 enforcement)
- Scope narrowing: m13 writes only `tool_call` and `consumption` event types — not cascade clusters, not Battern steps (those stay in the opaque-id layer of m4/m5)

#### Connection ceremony

Day 21 begins with the registration ceremony. The sequence must be Watcher-witnessed (cross-talk notice filed before and after):

```bash
# Step 1 — probe stcortex reachability
stcortex status
# Expected: UP + consumer count ≥ 1

# Step 2 — register workflow-trace consumer for workflow_trace_main namespace
stcortex call register_consumer 'workflow-trace' 'workflow_trace_main' 'cli'
# Expected: no error output; consumer row created

# Step 3 — verify registration
stcortex sql "SELECT name, namespace, transport, stale FROM consumer \
  WHERE namespace = 'workflow_trace_main'"
# Expected: 1 row, stale=false

# Step 4 — F8 namespace-guard verification (m9 wire check)
# m9 reads registered namespaces at startup and refuses writes to unknown ns
# Verify m9 config lists 'workflow_trace_main' as authorised:
grep -r 'workflow_trace' the-workflow-engine/src/m2_substrate/m9_watcher_namespace_guard.rs

# Step 5 — first workflow_trace_* memory write
stcortex call write_memory 'workflow_trace_main' 'semantic' \
  'workflow-trace consumer registered S1001982' '1.0' 'null' \
  '<session_uuid>' 'null' '[]'
# Expected: memory_id returned; no 530

# Step 6 — consumer freshness probe (refuse-write triggers if not registered)
stcortex sql "SELECT name, last_read_at, stale FROM consumer \
  WHERE name = 'workflow-trace'"
```

After Step 6, atuin records the full command trajectory. The registration timestamp is the T0 anchor for the 30-day stale window.

#### Watcher cross-talk notice (before + after)

File at `~/projects/shared-context/watcher-notices/` before starting the ceremony:

```
WCP-NOTICE-PHASE3-TRACK1-BEGIN
stcortex consumer registration ceremony starting for 'workflow-trace' 
namespace workflow_trace_main. Refuse-write contract in force.
T0: <timestamp>
```

After Step 6 succeeds, file:

```
WCP-NOTICE-PHASE3-TRACK1-COMPLETE
Consumer registered. Freshness probe passed. First memory written.
stcortex memory_id: <id>
Watcher Class-B handoff boundary crossed: Claude session → stcortex DB layer.
```

#### Test sequence

```bash
# Smoke test 1: consumer registration is idempotent
stcortex call register_consumer 'workflow-trace' 'workflow_trace_main' 'cli'
stcortex call register_consumer 'workflow-trace' 'workflow_trace_main' 'cli'
stcortex sql "SELECT COUNT(*) FROM consumer WHERE name = 'workflow-trace'"
# Expected: 1 (not 2)

# Smoke test 2: unregistered write is refused
stcortex call register_consumer 'temp-test' 'workflow_trace_ORPHAN' 'cli'
stcortex call unregister_consumer 'temp-test'
stcortex call write_memory 'workflow_trace_ORPHAN' 'semantic' 'should-fail' \
  '1.0' 'null' 'test' 'null' '[]'
# Expected: 530 error — "no fresh consumer registered"

# Smoke test 3: m9 namespace guard rejects non-workflow_trace_ prefix
# (unit test in m9 module; verify by invoking m9's guard function with a
# synthetic pathway source 'orac_learn_something')
cargo test -p workflow-trace m9_watcher_namespace_guard -- --nocapture
# Expected: guard rejects non-prefixed IDs, passes prefixed IDs

# Smoke test 4: m13 write → stcortex pathway appears
# (integration test using a test namespace in 'scratch' space)
cargo test -p workflow-trace m13_stcortex_writer -- --nocapture
```

#### Failure modes + Watcher flags

| Failure | Watcher class | Response |
|---|---|---|
| stcortex unreachable (`:3000` down) | Class B | Log `ERROR`, surface in Phase 3 status report; do not proceed to Track 1 writes |
| 530 refuse-write on first memory write | Class B | Architectural signal — recheck registration; never bypass with `scratch` namespace |
| m9 allows non-prefixed pathway ID | Class D | Four-surface drift: AP30 spec (vault) vs implementation (code) diverge; fix before proceeding |
| Consumer goes stale before Phase 4 | Class I | `access_memory` not being called in active sessions; add to session startup hook |

#### Atuin trajectory

Every `stcortex call` and `stcortex sql` command is automatically recorded. Additionally:

```bash
# Tag Phase 3 Track 1 in atuin KV store
atuin kv set 'wf-phase3-track1-registered' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
atuin kv set 'wf-phase3-track1-consumer' 'workflow-trace'
```

#### Verification: integration is live

- `stcortex sql "SELECT name, stale FROM consumer WHERE name = 'workflow-trace'"` returns 1 row with `stale=false`
- `stcortex sql "SELECT id, content FROM memory WHERE namespace = 'workflow_trace_main'"` returns ≥ 1 memory
- `cargo test -p workflow-trace m2_ m9_ m13_ -- --nocapture` passes with 0 failures

---

### Track 2 — HABITAT-CONDUCTOR dispatch wiring (m32)

**Days 22-23 · Module: m32 · Gate: Conductor Wave 1B/1C must be started by Luke**

#### Prereqs

- m32 (`dispatcher`) is built and gate-clean from Phase 2B
- Conductor Wave 1B (`enforcer`) and 1C (`zen`) have been started by Luke at the terminal
- `CONDUCTOR_DISPATCH_ENABLED=1` is set in the wf-dispatch process environment
- m33 has produced at least one `VerificationResult` for the test workflow
- `dispatch_log.db` file has been created (m32 migration applied)

**If Conductor is not started:** Track 2 proceeds through the connection probe and dry-run scaffold only. Live dispatch integration tests are deferred. m32 remains in refuse-mode and this is flagged explicitly in the Day 26 status snapshot.

#### Wire-protocol contract

m32 communicates with HABITAT-CONDUCTOR via a probe-then-dispatch pattern. The Conductor's health endpoint shape (from `conductor_api.rs`, reference only — m32 has no HTTP server):

```
GET http://127.0.0.1:8141/health
Expected response:
  HTTP 200
  {"status":"ok","service":"weaver","port":8141,...}

m32 probe call (internal raw_http_get, not curl):
  raw_http_get("127.0.0.1:8141", "/health")
  → matches body contains `"status":"ok"` → probe passes
  → anything else → Err(DispatchError::ConductorNotLive)
```

The dispatch request shape (outbound from m32 to Conductor):

```json
{
  "dispatch_id": "<uuid_v7>",
  "workflow_id": "<workflow_id_from_m30>",
  "steps": [
    {
      "id": "<step_id>",
      "kind": "cargo_check",
      "step_surface": "read_only",
      "traps": [],
      "conductor_params": {"service": "workflow-trace", "timeout_secs": 120}
    }
  ],
  "escape_surface": "read_only",
  "dispatched_at": 1716000000000,
  "operator": "wf-dispatch/human",
  "verification_receipt": {
    "last_verified_at": 1715913600000,
    "definition_hash": "<fnv1a_64_hex>"
  },
  "dry_run": true
}
```

The `dry_run: true` flag is mandatory during Days 22-23 integration testing. Conductor logs the request but does not execute steps. This is the integration proof without live execution.

#### Connection ceremony

The 5-check pre-dispatch sequence (from m32 module spec) is the ceremony. Each check is a distinct probe:

```bash
# Check 1 — Conductor live probe
curl -s http://127.0.0.1:8141/health | python3 -c \
  "import sys,json; d=json.load(sys.stdin); print('PASS' if d.get('status')=='ok' else 'FAIL')"

# Check 2 — m33 TTL freshness (workflow must have been verified within 7 days)
sqlite3 workflow_bank.db \
  "SELECT id, last_verified_at, \
   CASE WHEN (strftime('%s','now')*1000 - last_verified_at) < 604800000 \
        THEN 'FRESH' ELSE 'STALE' END AS ttl_status \
   FROM accepted_workflows LIMIT 5"

# Check 3 — definition_hash match (m32 computes FNV-1a of steps_json at dispatch time;
#            must match VerificationResult::definition_hash stored by m33)
#            This is a code-level assertion — run the m32 unit test:
cargo test -p workflow-trace dispatcher_definition_hash -- --nocapture

# Check 4 — sunset guard
sqlite3 workflow_bank.db \
  "SELECT id, sunset_at, \
   CASE WHEN sunset_at > (strftime('%s','now')*1000) THEN 'LIVE' ELSE 'SUNSET' END \
   FROM accepted_workflows"

# Check 5 — dispatch cooldown (300s default)
sqlite3 dispatch_log.db \
  "SELECT workflow_id, dispatched_at, \
   CASE WHEN (strftime('%s','now')*1000 - dispatched_at) > 300000 \
        THEN 'COOLDOWN_OK' ELSE 'COOLDOWN_ACTIVE' END \
   FROM dispatch_log ORDER BY dispatched_at DESC LIMIT 5"
```

**Display-before-step gate** (Cipher P0 #11): m32 prints this to stdout before every dispatch. The integration test must capture stdout and assert the banner is present:

```
═══════════════════════════════════════════════════════════════
  wf-dispatch: about to dispatch N steps via HABITAT-CONDUCTOR
  workflow:  <workflow_id>
  verified:  <ISO8601> (<N> days ago; TTL ok)
═══════════════════════════════════════════════════════════════
  Step 1/N · kind=... · surface=READ_ONLY
  [READ-ONLY] No writes or network in this step.
  ...
═══════════════════════════════════════════════════════════════
  Handing off to Conductor. wf-dispatch does NOT execute steps.
═══════════════════════════════════════════════════════════════
```

#### Test sequence

```bash
# Test 1 — refuse-mode when CONDUCTOR_DISPATCH_ENABLED not set
CONDUCTOR_DISPATCH_ENABLED= wf-dispatch dispatch <workflow_id>
# Expected: ERROR: dispatcher: dispatch disabled — set CONDUCTOR_DISPATCH_ENABLED=1

# Test 2 — live-probe fails when Conductor not running (regression guard)
# Kill conductor temporarily, probe, restart
CONDUCTOR_DISPATCH_ENABLED=1 wf-dispatch dispatch <workflow_id>
# Expected: ERROR: dispatcher: Conductor not live at 127.0.0.1:8141: ...

# Test 3 — dry-run dispatch (Conductor live, ENABLED=1, dry_run=true)
CONDUCTOR_DISPATCH_ENABLED=1 wf-dispatch dispatch --dry-run <workflow_id>
# Expected: display-before-step printed; Conductor logs receipt; no execution

# Test 4 — audit-first guarantee
# Verify dispatch_log row exists BEFORE Conductor receives the request
# (Use sqlite3 polling after trigger, before Conductor response):
cargo test -p workflow-trace dispatcher_audit_first -- --nocapture

# Test 5 — fan-out fires after Conductor accept
# Verify WorkflowDispatchEvent is enqueued (m40 outbox has a new row)
CONDUCTOR_DISPATCH_ENABLED=1 wf-dispatch dispatch --dry-run <workflow_id>
grep '"conductor_accepted":true' data/workflow_trace_outbox.jsonl | tail -1
```

#### Failure modes + Watcher flags

| Failure | Watcher class | Response |
|---|---|---|
| Conductor health returns non-ok with Wave 1B started | Class B | Check enforcer daemon logs; `devenv logs enforcer`; surface to Luke |
| `CONDUCTOR_DISPATCH_ENABLED=1` set but Conductor not started | Class B | Hard error returned by m32 — expected; do not soft-fail |
| Display-before-step banner absent in dry-run test | Class D | Implementation drifts from Cipher P0 #11 spec; fix before Track 2 closes |
| audit_first test fails (dispatch log written after Conductor call) | Class D | Critical correctness bug; blocks Track 2 close |
| Fan-out (m40 outbox) not populated after dry-run | Class B | CC-5 loop cannot close; m32→m40 channel broken; diagnose before Day 24 |

**Conductor maturity dependency (critical-path blocker restatement):** if Waves 1B/1C/2/3 remain `auto_start=false` by Day 26, Track 2 closes as "integration scaffolded; live dispatch wire pending Conductor Wave bring-up (Luke terminal action required)." Phase 4 hardening does NOT inherit a live dispatch wire in this case.

#### Atuin trajectory

```bash
atuin kv set 'wf-phase3-track2-conductor-probed' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
atuin kv set 'wf-phase3-track2-dry-run-status' "PASS"
# If live dispatch lands:
atuin kv set 'wf-phase3-track2-live-dispatch' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
```

#### Verification: integration is live

- `curl -s http://127.0.0.1:8141/health | python3 -c "..."` returns PASS
- `wf-dispatch dispatch --dry-run <id>` prints display-before-step banner, exits 0
- `dispatch_log.db` has a row with `outcome = 'Accepted'` and `dry_run = true`
- `data/workflow_trace_outbox.jsonl` has an entry with `"kind":"run"` and `"conductor_accepted":true`

---

### Track 3 — SYNTHEX v2 NexusEvent emission (m40)

**Days 23-24 · Module: m40 · Endpoint: `127.0.0.1:8092/v3/nexus/push`**

#### Prereqs

- SYNTHEX v2 running at `:8092` (confirmed by `curl -s http://127.0.0.1:8092/health`)
- m40 (`nexus_event_emitter`) built and gate-clean from Phase 2B
- Outbox JSONL file path configured: `{data_dir}/workflow_trace_outbox.jsonl`
- At least one `WorkflowDispatchEvent` available (from Track 2 dry-run test) for first emission
- Circuit breaker starts in `Closed` state

#### Wire-protocol contract

m40 uses Option A (untyped JSON — no co-change on SYNTHEX v2 side required). The local `NexusEvent` re-declaration (not imported from synthex_v2 crate):

```json
POST http://127.0.0.1:8092/v3/nexus/push
Content-Type: application/json

{
  "events": [
    {
      "type": "workflow_promote",
      "ts": 1716000000,
      "data": {
        "kind": "promote",
        "id": "wf-abc123",
        "lineage": ["wf-parent-456"]
      }
    }
  ]
}
```

For a `WorkflowEvent::Run`:

```json
{
  "events": [
    {
      "type": "workflow_run",
      "ts": 1716000000,
      "data": {
        "kind": "run",
        "id": "wf-abc123",
        "outcome": "pass_verified"
      }
    }
  ]
}
```

For a `WorkflowEvent::Decay`:

```json
{
  "events": [
    {
      "type": "workflow_decay",
      "ts": 1716000000,
      "data": {
        "kind": "decay",
        "id": "wf-abc123",
        "weight": 0.72
      }
    }
  ]
}
```

SYNTHEX v2 acknowledges with HTTP 200 and an optional response body (ignored by m40 — fire-and-forget). A 4xx or 5xx response increments the circuit breaker failure counter.

#### Dual-transport: outbox-first, HTTP fire-and-forget

The outbox-first guarantee means the JSONL file is written before any HTTP attempt. The emission order:

1. Serialize `WorkflowEvent` → `OutboxEnvelope` (set `posted=false`, `attempts=0`)
2. Append to `{data_dir}/workflow_trace_outbox.jsonl`
3. If circuit breaker is `Closed`: attempt `POST /v3/nexus/push` with the event batch
4. On HTTP 200: update envelope in JSONL to `posted=true`
5. On non-200 or timeout: log `WARN`, leave `posted=false`, increment failure counter
6. If circuit breaker is `Open`: skip HTTP, log `WARN`, return `EmitOutcome { appended: true, posted: false }`

Background retry sweep (60s interval): reads all `posted=false` envelopes where `attempts < 5`, batches them, retries.

#### Connection ceremony

```bash
# Step 1 — SYNTHEX v2 reachability
curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:8092/health
# Expected: 200

# Step 2 — manual NexusEvent push (verifies endpoint shape before m40 wires)
curl -s -X POST http://127.0.0.1:8092/v3/nexus/push \
  -H "Content-Type: application/json" \
  -d '{"events":[{"type":"workflow_promote","ts":'"$(date +%s)"',"data":{"kind":"promote","id":"wf-test-001","lineage":[]}}]}'
# Expected: 200; observe SYNTHEX logs for Hebbian processing of the event

# Step 3 — outbox file initialisation check
ls -la data/workflow_trace_outbox.jsonl 2>/dev/null || echo "MISSING — m40 creates on first emit"

# Step 4 — first WorkflowEvent::Promote emitted via m40 (integration test)
cargo test -p workflow-trace nexus_event_emitter_first_emit -- --nocapture
# Expected: outbox has 1 entry (posted=true if SYNTHEX reachable; posted=false if not)

# Step 5 — circuit breaker initial state verification
cargo test -p workflow-trace m40_circuit_breaker_starts_closed -- --nocapture
```

#### Schema evolution risk check

Because the `NexusEvent.data` field is untyped JSON, SYNTHEX v2 may silently ignore events if the `"type"` discriminator field name changes. The S118-pattern round-trip test:

```bash
# After first emission succeeds, poll SYNTHEX for the workflow event
curl -s "http://127.0.0.1:8092/v3/nexus/pull?event_type=workflow_promote&limit=5" | \
  python3 -c "import sys,json; events=json.load(sys.stdin); \
  print('FOUND' if any(e.get('type')=='workflow_promote' for e in events) else 'MISSING')"
# Expected: FOUND — if MISSING, schema drift is active
```

If MISSING: file a `D-class` Watcher drift notice. The outbox's `posted=false` entries will confirm this independently (HTTP 200 is returned by SYNTHEX even when the event is unrecognised — check SYNTHEX logs directly).

#### Test sequence

```bash
# Test 1 — WorkflowEvent::Run emission after dispatch dry-run
CONDUCTOR_DISPATCH_ENABLED=1 wf-dispatch dispatch --dry-run <workflow_id>
# m40 receives dispatch event fan-out → emits WorkflowEvent::Run
tail -1 data/workflow_trace_outbox.jsonl | python3 -c \
  "import sys,json; e=json.load(sys.stdin); assert e['event']['kind']=='run'; print('OK')"

# Test 2 — outbox retry sweep fires on posted=false envelope
# Inject a synthetic envelope with posted=false, wait 60s
cargo test -p workflow-trace m40_outbox_retry_sweep -- --nocapture

# Test 3 — circuit breaker Open state skips HTTP but writes outbox
# Simulate 5 consecutive 503s from SYNTHEX mock endpoint
cargo test -p workflow-trace m40_circuit_breaker_open_still_writes_outbox -- --nocapture

# Test 4 — NexusEvent serde rename ("type" not "event_type")
cargo test -p workflow-trace m40_nexus_event_serde_rename -- --nocapture
# Must assert: serialized JSON has key "type", not "event_type"
```

#### Failure modes + Watcher flags

| Failure | Watcher class | Response |
|---|---|---|
| SYNTHEX v2 at `:8092` unreachable | Class B | Circuit breaker opens; outbox receives all events; not a blocking failure for dispatch |
| `NexusEvent` serialized with `event_type` key instead of `type` | Class D | Schema drift vs SYNTHEX wire contract; fix serde rename annotation before Day 24 close |
| Outbox JSONL not written before HTTP attempt | Class D | Outbox-first guarantee violated; critical correctness bug |
| SYNTHEX receives events but Hebbian LTP/LTD does not move | Class I | Partial Hebbian silence; SYNTHEX is receiving but not reinforcing; note in Day 26 snapshot |

#### Atuin trajectory

```bash
atuin kv set 'wf-phase3-track3-first-promote-emitted' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
atuin kv set 'wf-phase3-track3-outbox-path' "$(realpath data/workflow_trace_outbox.jsonl)"
atuin kv set 'wf-phase3-track3-circuit-breaker-state' "closed"
```

#### Verification: integration is live

- `tail -5 data/workflow_trace_outbox.jsonl` shows entries with `"posted":true` (SYNTHEX reachable) or `"posted":false` with `"attempts":1` (SYNTHEX down but outbox alive)
- `cargo test -p workflow-trace m40_ -- --nocapture` passes with 0 failures
- S118-pattern round-trip test returns FOUND for `workflow_promote` event type

---

### Track 4 — LCM RPC integration (m41)

**Days 24-25 · Module: m41 · Socket: `$XDG_RUNTIME_DIR/lcm/supervisor.sock`**

#### Prereqs

- LCM supervisor daemon running (from `loop-engine-v2/` project; binary `lcm-supervisor`)
- M0-verified (commit `97218d1`; `deploy_cancel_handler` shipped per S1001883)
- UDS socket at `$XDG_RUNTIME_DIR/lcm/supervisor.sock` (permissions `0o600`, owner-only)
- m41 (`lcm_rpc_client`) built and gate-clean from Phase 2B
- At least one deploy-shaped workflow step available (from m30 bank)
- `lcm.ping` method confirmed working before `lcm.loop.create` is tested

**LCM supervisor startup (Luke at terminal, not Claude):**
```bash
# Verify supervisor is running
ls -la $XDG_RUNTIME_DIR/lcm/supervisor.sock
echo '{"jsonrpc":"2.0","id":1,"method":"lcm.ping","params":{}}' | \
  socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/lcm/supervisor.sock
# Expected: {"jsonrpc":"2.0","id":1,"result":{"pong":true,"version":"..."}}
```

#### Wire-protocol contract

JSON-RPC 2.0 over Unix domain socket, newline-framed. One request per `\n`-terminated line, one response per `\n`-terminated line. 30-second read timeout.

**Health probe (used by HalfOpen circuit breaker state):**
```json
{"jsonrpc":"2.0","id":1,"method":"lcm.ping","params":{}}\n
→
{"jsonrpc":"2.0","id":1,"result":{"pong":true,"version":"<semver>"}}\n
```

**Loop creation (deploy-shaped workflow step):**
```json
{"jsonrpc":"2.0","id":2,"method":"lcm.loop.create",
 "params":{"caller_id":"wf-abc123","name":"deploy:weaver:0.3.1",
           "max_iters":1,"survives_session_death":false}}\n
→
{"jsonrpc":"2.0","id":2,"result":{"loop_id":"<uuid>"}}\n
```

**Status poll (verification path, if needed):**
```json
{"jsonrpc":"2.0","id":3,"method":"lcm.loop.status","params":{"loop_id":"<uuid>"}}\n
→
{"jsonrpc":"2.0","id":3,"result":{"loop_id":"<uuid>","state":"Running",
  "created_at_unix_nanos":1716000000000000000}}\n
```

**RPC error response shape:**
```json
{"jsonrpc":"2.0","id":4,"error":{"code":-32601,"message":"Method not found"}}\n
```

#### Deploy-shape detection

m32 marks each workflow step with `StepKind` before the dispatch event fan-out. m41 only activates when `StepKind::Deploy` is present. Non-deploy steps return `LcmDispatchOutcome::NotApplicable` immediately (no UDS connection opened). The detection table:

| Step kind | Activates m41 | Example step name |
|---|---|---|
| `cargo install <binary>` | YES | `install:workflow-trace:0.1.0` |
| `devenv start <service>` | YES | `deploy:weaver:0.3.1` |
| Binary deploy to `~/.local/bin/` | YES | `deploy:wf-dispatch:0.1.0` |
| `devenv restart <service>` | YES | `restart:workflow-trace` |
| `cargo check` / `cargo test` | NO | quality gate steps |
| File edit / read | NO | — |
| HTTP probe | NO | — |

#### Connection ceremony

```bash
# Step 1 — UDS socket exists and permissions are 0o600
ls -la $XDG_RUNTIME_DIR/lcm/supervisor.sock
stat -c "%a" $XDG_RUNTIME_DIR/lcm/supervisor.sock
# Expected: 600

# Step 2 — lcm.ping succeeds (manual)
echo '{"jsonrpc":"2.0","id":1,"method":"lcm.ping","params":{}}' | \
  socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/lcm/supervisor.sock
# Expected: pong response with version

# Step 3 — first lcm.loop.create (max_iters: 1; survives_session_death: false)
echo '{"jsonrpc":"2.0","id":2,"method":"lcm.loop.create","params":{
  "caller_id":"wf-phase3-test","name":"deploy:test:0.0.1",
  "max_iters":1,"survives_session_death":false}}' | \
  socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/lcm/supervisor.sock
# Expected: result with loop_id

# Step 4 — verify loop registered in LCM supervisor state
echo '{"jsonrpc":"2.0","id":3,"method":"lcm.loop.status","params":{"loop_id":"<id>"}}' | \
  socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/lcm/supervisor.sock
# Expected: state=Running or state=Completed (if max_iters=1 already ran)

# Step 5 — deploy-shape detection in m32 verified
cargo test -p workflow-trace m32_deploy_shape_detection -- --nocapture
```

#### Test sequence

```bash
# Test 1 — LcmDispatchOutcome::NotApplicable for non-deploy step
cargo test -p workflow-trace m41_not_applicable_for_non_deploy -- --nocapture
# Must NOT open a UDS connection

# Test 2 — lcm.loop.create succeeds for deploy step (mock UDS server)
cargo test -p workflow-trace m41_loop_create_success -- --nocapture

# Test 3 — 30-second read timeout returns Timeout outcome
cargo test -p workflow-trace m41_read_timeout -- --nocapture

# Test 4 — circuit breaker Open state skips UDS (after 5 consecutive failures)
cargo test -p workflow-trace m41_circuit_breaker_open -- --nocapture

# Test 5 — newline framing: request ends with \n, response parsed line-by-line
cargo test -p workflow-trace m41_newline_framing -- --nocapture

# Test 6 — reconnect on each call (no persistent connection)
# Verify UDS fd is opened and closed per dispatch call
cargo test -p workflow-trace m41_reconnect_per_call -- --nocapture
```

#### Failure modes + Watcher flags

| Failure | Watcher class | Response |
|---|---|---|
| UDS socket missing (`$XDG_RUNTIME_DIR/lcm/supervisor.sock` absent) | Class B | LCM supervisor not started; m41 logs `WARN`; advisory tracking gap only, does not block dispatch |
| `lcm.loop.create` returns RPC error code | Class B | LCM supervisor internal error; log at `WARN`; return `LcmDispatchOutcome::RpcError` |
| Read timeout after 30s | Class B | m41 returns `Timeout`; circuit breaker failure incremented |
| m41 opens persistent connection (bug) | Class D | Spec requires reconnect-per-call; persistent connection adds keepalive complexity |

**LCM does not block dispatch.** A failed `lcm.loop.create` is an advisory tracking gap. The Conductor wire (m32) has already accepted the dispatch. m41's role is supplementary lifecycle tracking, not execution authority.

#### Atuin trajectory

```bash
atuin kv set 'wf-phase3-track4-lcm-ping-verified' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
atuin kv set 'wf-phase3-track4-first-loop-create' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
atuin kv set 'wf-phase3-track4-first-loop-id' "<loop_id>"
```

#### Verification: integration is live

- `socat - UNIX-CONNECT:...` ping returns pong
- `lcm.loop.create` with `max_iters: 1` returns a `loop_id`
- `cargo test -p workflow-trace m41_ -- --nocapture` passes with 0 failures
- m41 does not open connections for non-deploy steps (verified in unit test)

---

### Track 5 — POVM Hebbian feedback (m42)

**Day 25 · Module: m42 · Endpoint: `127.0.0.1:8125/reinforce` · Pre-cutover path**

#### Prereqs

- POVM Engine at `:8125` is live (Batch 1 service; `curl -s http://127.0.0.1:8125/health` returns 200)
- m42 (`hebbian_feedback`) built and gate-clean from Phase 2B
- `povm_overlap_active = true` in m42 config (safe default — no POVM write before explicit flag)
- AP30 namespace collision check passed (see below)
- At least one `WorkflowDispatchEvent` with `outcome` field available (from Track 2 + Track 3 tests)

**POVM status:** DEPRECATED 2026-07-10. m42 dual-path is active. During Phase 3, POVM is the write target. After 2026-07-10, `povm_overlap_active` is flipped to false and m42 routes to stcortex via m13. This flip is an explicit operator action; m42 never infers it automatically.

#### AP30 namespace collision check (mandatory before first POVM write)

V3 (DevOps Engine V3) owns POVM pathway IDs `P01..P16`. m42 must not touch these. Before the first `POST /reinforce` call, verify the namespace is clear:

```bash
# Query POVM for existing workflow_trace_ pathways (should be 0 before first write)
curl -s http://127.0.0.1:8125/pathways | python3 -c "
import sys, json
pathways = json.load(sys.stdin)
wf_paths = [p for p in pathways if p.get('source','').startswith('workflow_trace_')]
v3_paths = [p for p in pathways if p.get('source','').startswith('P')]
print(f'workflow_trace_ pathways: {len(wf_paths)} (expected 0 before first write)')
print(f'V3 P01..P16 pathways: {len(v3_paths)}')
conflicts = [p for p in wf_paths if p in v3_paths]
print(f'Collisions: {len(conflicts)} (must be 0)')
"
# Expected: 0 workflow_trace_ pathways, 0 collisions
```

If any `workflow_trace_` pathways exist from a prior session, that is expected (stale prior write). If any V3 pathway (`P01..P16`) is also prefixed `workflow_trace_` — that is an AP30 collision and must be escalated before proceeding.

#### Wire-protocol contract

Following `m24_povm_bridge.rs` gold standard exactly. BUG-033 fix applies: socket address is raw `host:port`, no `http://` prefix. BUG-034 fix applies: endpoint is `/reinforce`, not `/pathways` or `/memories`.

```
POST to raw TCP socket "127.0.0.1:8125"
Path: /reinforce
Content-Type: application/json

{
  "session_id": "wf-abc123",
  "fitness_delta": 0.25,
  "retrieval_ids": [
    "workflow_trace_wf-abc123",
    "workflow_trace_outcome_pass_verified"
  ],
  "request_id": "<uuid_v4_1h_idempotency_key>"
}
```

`fitness_delta` constants:
- `RunOutcome::PassVerified` → `+0.25`
- `RunOutcome::Pass` → `+0.15`
- `RunOutcome::Blocked` → `-0.05`
- `RunOutcome::Fail` → `-0.10`

All values clamped to `[-1.0, 1.0]` before sending.

Expected POVM response: HTTP 200 with body `{"status":"reinforced","session_id":"...",...}` or equivalent. F-001 silent-swallow fix: non-200 status is logged at `WARN`, not silently dropped.

#### Connection ceremony

```bash
# Step 1 — POVM health probe (raw http, following gold standard pattern)
curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:8125/health
# Expected: 200

# Step 2 — AP30 namespace collision check (above)

# Step 3 — First /reinforce call (manual, before m42 wires)
curl -s -X POST http://127.0.0.1:8125/reinforce \
  -H "Content-Type: application/json" \
  -d '{
    "session_id":"wf-phase3-test",
    "fitness_delta":0.15,
    "retrieval_ids":["workflow_trace_test_001","workflow_trace_outcome_pass"],
    "request_id":"'$(python3 -c "import uuid; print(uuid.uuid4())")'"}' | \
  python3 -c "import sys,json; d=json.load(sys.stdin); print('ACCEPTED' if d.get('status') else 'ERROR')"
# Expected: ACCEPTED

# Step 4 — Verify pathway weight movement in POVM learning_health
curl -s http://127.0.0.1:8125/learning_health | python3 -c "
import sys, json
d = json.load(sys.stdin)
print(f'learning_health: {d.get(\"learning_health\", \"missing\")}')
"
# Observation: note the value BEFORE and AFTER the first /reinforce call
# Expected: value changes (even slightly) — this is the Hebbian signal

# Step 5 — m42 unit tests
cargo test -p workflow-trace hebbian_feedback -- --nocapture
```

#### POVM cutover transition plan

m42 implements dual-path routing via the `povm_overlap_active` config flag:

```
povm_overlap_active = true  (Phase 3, now → 2026-07-10)
  → POST /reinforce to 127.0.0.1:8125

povm_overlap_active = false (after 2026-07-10 cutover)
  → write_memory via m13 stcortex path
  → namespace: workflow_trace_<workflow_id>_<outcome>
  → NO silent fallback to POVM after cutover
```

The cutover flip is an explicit operator action (Luke updates m42 config). Claude does not infer cutover timing. If stcortex is unreachable and `povm_overlap_active = false`, m42 logs at `ERROR` and returns `ReinforceOutcome::SubstrateUnavailable`. Per `CLAUDE.md` stcortex policy: no silent fallback.

During the POVM overlap window, m42 reads are not attempted (m42 is write-only). m31 reads pathway weights from stcortex during the overlap (stcortex receives POVM pathways via the existing stcortex-POVM bridge). This is the handoff architecture.

#### Test sequence

```bash
# Test 1 — AP30 namespace: all retrieval_ids start with workflow_trace_
cargo test -p workflow-trace m42_ap30_namespace_prefix -- --nocapture

# Test 2 — fitness_delta values match spec table constants
cargo test -p workflow-trace m42_fitness_delta_constants -- --nocapture

# Test 3 — fitness_delta clamped to [-1.0, 1.0]
cargo test -p workflow-trace m42_fitness_delta_clamp -- --nocapture

# Test 4 — overlap active: routes to POVM
cargo test -p workflow-trace m42_overlap_active_routes_povm -- --nocapture

# Test 5 — overlap inactive: routes to stcortex (not POVM fallback)
cargo test -p workflow-trace m42_post_cutover_no_povm_fallback -- --nocapture

# Test 6 — BUG-033: socket addr has no http:// prefix
cargo test -p workflow-trace m42_bug033_no_http_prefix -- --nocapture

# Test 7 — F-001: non-200 status logged, not silently dropped
cargo test -p workflow-trace m42_f001_non200_logged -- --nocapture
```

#### Failure modes + Watcher flags

| Failure | Watcher class | Response |
|---|---|---|
| POVM at `:8125` unreachable | Class B | Circuit breaker opens; log `WARN`; not a blocking failure for dispatch |
| `learning_health` does not move after /reinforce | Class I | **Hebbian silence** — the CC-5 learning loop may be decorative; Watcher Class-I flag fires; document in Day 26 snapshot |
| POVM responds 200 but pathway not created | Class D | Silent swallow; check POVM internal logs; F-001 mitigation should have surfaced this |
| m42 writes `P01..P16` pathways (AP30 collision) | Class D | Critical namespace violation; hard block on Phase 4 until fixed |

**Watcher Class-I pre-positioned:** if `learning_health` does not show any movement (even delta < 0.001) within 5 minutes of the first `/reinforce` call, Watcher flags Class-I "Hebbian silence" for this pathway. This is the strongest signal that the CC-5 loop cannot close. It does not block Phase 4 but must appear in the hand-off snapshot.

#### Atuin trajectory

```bash
atuin kv set 'wf-phase3-track5-first-reinforce' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
atuin kv set 'wf-phase3-track5-learning-health-before' "<value>"
atuin kv set 'wf-phase3-track5-learning-health-after' "<value>"
atuin kv set 'wf-phase3-track5-povm-overlap-active' "true"
```

#### Verification: integration is live

- `curl -s http://127.0.0.1:8125/health` returns 200
- `/reinforce` call returns ACCEPTED status
- AP30 namespace check shows 0 collisions
- `learning_health` shows measurable movement after first reinforcement (even 0.001)
- `cargo test -p workflow-trace m42_ -- --nocapture` passes with 0 failures

---

## CC-5 substrate learning loop — first measured closure

**Day 26 · The moment the Watcher Class-I "Hebbian silence" flag clears**

### What CC-5 closure looks like

By Day 26, the five integration tracks are either live or in known-refuse-mode with explicit blockers documented. The CC-5 loop closure test dispatches a synthetic test workflow end-to-end and measures whether each hop in the loop leaves a verifiable footprint:

```
m32 dispatches (Track 2) →
  m40 emits WorkflowEvent::Run (Track 3) →
    SYNTHEX v2 receives + Hebbian LTP fires (observed via SYNTHEX logs) →
  m42 /reinforce to POVM (Track 5) →
    POVM pathway weight increases (observed via learning_health delta) →
    stcortex pathway weight updates (via POVM↔stcortex bridge) →
      m31 reads updated pathway weight next selection cycle →
        selection weight shifts (observed by comparing m31 ranked output before/after)
```

Each hop is an independently verifiable claim. The "first measured closure" is the moment ALL hops produce verifiable evidence in the same session.

### Day 26 integration test procedure

```bash
# Pre-test baseline snapshot
atuin scripts run habitat-intel > /tmp/wf-phase3-day26-baseline.tsv
curl -s http://127.0.0.1:8125/learning_health > /tmp/wf-phase3-day26-lh-before.json
stcortex sql "SELECT pre_id, post_id, weight FROM pathway \
  WHERE namespace = 'workflow_trace_main' ORDER BY weight DESC LIMIT 10" \
  > /tmp/wf-phase3-day26-stcortex-weights-before.tsv

# Dispatch synthetic test workflow (dry-run with fan-out enabled)
CONDUCTOR_DISPATCH_ENABLED=1 wf-dispatch dispatch --dry-run <test_workflow_id>

# Allow 60-90s for Hebbian substrate to propagate
# (SYNTHEX Hebbian reinforcement is not instantaneous — it fires in next consolidation cycle)

# Post-dispatch measurement
curl -s http://127.0.0.1:8125/learning_health > /tmp/wf-phase3-day26-lh-after.json
stcortex sql "SELECT pre_id, post_id, weight FROM pathway \
  WHERE namespace = 'workflow_trace_main' ORDER BY weight DESC LIMIT 10" \
  > /tmp/wf-phase3-day26-stcortex-weights-after.tsv
atuin scripts run habitat-intel > /tmp/wf-phase3-day26-post.tsv

# Measure pathway weight delta
python3 -c "
import json, sys
before = json.load(open('/tmp/wf-phase3-day26-lh-before.json'))
after  = json.load(open('/tmp/wf-phase3-day26-lh-after.json'))
before_lh = before.get('learning_health', 0)
after_lh  = after.get('learning_health', 0)
delta = after_lh - before_lh
print(f'learning_health before: {before_lh:.4f}')
print(f'learning_health after:  {after_lh:.4f}')
print(f'delta: {delta:+.4f}')
if abs(delta) > 0.0005:
    print('CLASS-I FLAG: CLEARED — Hebbian learning loop is receiving signals')
else:
    print('CLASS-I FLAG: ACTIVE — learning_health did not move; Hebbian silence confirmed')
"
```

### Watcher witness requirement

The CC-5 closure (or failure to close) must be Watcher-witnessed. File a WCP notice with exact timestamp and weight values:

```
WCP-NOTICE-PHASE3-CC5-LOOP-CLOSURE-ATTEMPT
Timestamp: <ISO8601>
learning_health before: <value>
learning_health after:  <value>
delta: <value>
stcortex pathway count before: <n>
stcortex pathway count after: <n>
SYNTHEX logs confirm Hebbian reinforcement: YES/NO
CC-5 loop closure status: FIRST-MEASURED-CLOSURE / PARTIAL / FAILED
Watcher Class-I flag status: CLEARED / ACTIVE (Hebbian silence confirmed)
```

If loop closure is achieved, this WCP notice is the four-surface persistence event:
1. WCP notice file (vault surface)
2. atuin KV entry `wf-cc5-first-closure` with timestamp (CLI surface)
3. stcortex memory in `workflow_trace_main` namespace: "CC-5 first closure Day 26" (stcortex surface)
4. This document (deployment framework surface — updated with timestamp in hand-off section below)

---

## V8 ↔ V3 bidirectional wire — inheritance

Per S1002029 substrate-level learning 1: "V8 and V3 already speak bidirectionally — `POST :8082/api/v8/confidence` + `/api/v8/learning` exist. Hebbian feedback exists at protocol level. Don't reinvent."

workflow-trace inherits this wire. m40's SYNTHEX v2 emissions feed into the same Hebbian substrate that V8 and V3 share. workflow-trace does not need to wire V3 or V8 directly — it contributes to the shared Hebbian field through SYNTHEX v2. The bidirectional V8↔V3 wire is a pre-existing substrate condition, not a Phase 3 work item.

The implication for Day 26: when m40 posts a `WorkflowEvent::Run` to SYNTHEX v2, and SYNTHEX reinforces via Hebbian LTP/LTD, that reinforcement signal propagates through the same substrate that V3's T4-T6 confidence calculations read. workflow-trace's pathway weights become part of the shared habitat Hebbian field. This is not speculative — it follows from SYNTHEX v2's existing architecture.

---

## Atuin proprioception — Phase 3 as a whole

Atuin is the only cross-tool provenance surface. Every shell command run during Phase 3 is recorded automatically. The Phase 3 trajectory is queryable post-deployment:

```bash
# All Phase 3 commands
atuin search --workspace the-workflow-engine --after "2026-05-XX"

# Substrate integration commands only
atuin search "stcortex call" --after "2026-05-XX"
atuin search "lcm.loop.create" --after "2026-05-XX"
atuin search "/reinforce" --after "2026-05-XX"
atuin search "/v3/nexus/push" --after "2026-05-XX"
```

The KV entries set during each track provide structured anchors within the raw history:

```bash
atuin kv list | grep 'wf-phase3'
```

Expected entries at Day 26 close:
- `wf-phase3-track1-registered`
- `wf-phase3-track2-conductor-probed` + optionally `wf-phase3-track2-live-dispatch`
- `wf-phase3-track3-first-promote-emitted`
- `wf-phase3-track4-lcm-ping-verified` + `wf-phase3-track4-first-loop-id`
- `wf-phase3-track5-first-reinforce` + learning_health before/after
- `wf-cc5-first-closure` (if loop closed)

---

## Substrate condition snapshot (Day 26 close)

The substrate condition snapshot is the final artefact of Phase 3. It is filed before the hand-off to Phase 4.

```bash
# Generate substrate snapshot
echo "=== Phase 3 Day 26 Close — Substrate Snapshot ===" > /tmp/wf-phase3-substrate-snapshot.txt
echo "Timestamp: $(date -u +%Y-%m-%dT%H:%M:%SZ)" >> /tmp/wf-phase3-substrate-snapshot.txt

# stcortex state
echo "--- stcortex ---" >> /tmp/wf-phase3-substrate-snapshot.txt
stcortex sql "SELECT name, namespace, stale FROM consumer WHERE namespace LIKE 'workflow_trace%'" \
  >> /tmp/wf-phase3-substrate-snapshot.txt
stcortex sql "SELECT COUNT(*) AS memory_count, namespace FROM memory \
  WHERE namespace LIKE 'workflow_trace%' GROUP BY namespace" \
  >> /tmp/wf-phase3-substrate-snapshot.txt
stcortex sql "SELECT COUNT(*) AS pathway_count FROM pathway \
  WHERE namespace LIKE 'workflow_trace%'" \
  >> /tmp/wf-phase3-substrate-snapshot.txt

# POVM learning health
echo "--- POVM ---" >> /tmp/wf-phase3-substrate-snapshot.txt
curl -s http://127.0.0.1:8125/learning_health >> /tmp/wf-phase3-substrate-snapshot.txt

# LCM supervisor state
echo "--- LCM ---" >> /tmp/wf-phase3-substrate-snapshot.txt
echo '{"jsonrpc":"2.0","id":1,"method":"lcm.ping","params":{}}' | \
  socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/lcm/supervisor.sock \
  >> /tmp/wf-phase3-substrate-snapshot.txt

# SYNTHEX v2 health
echo "--- SYNTHEX v2 ---" >> /tmp/wf-phase3-substrate-snapshot.txt
curl -s http://127.0.0.1:8092/health >> /tmp/wf-phase3-substrate-snapshot.txt

# Outbox status
echo "--- m40 outbox ---" >> /tmp/wf-phase3-substrate-snapshot.txt
wc -l data/workflow_trace_outbox.jsonl >> /tmp/wf-phase3-substrate-snapshot.txt
python3 -c "
import json
posted = sum(1 for l in open('data/workflow_trace_outbox.jsonl') if json.loads(l).get('posted'))
total = sum(1 for _ in open('data/workflow_trace_outbox.jsonl'))
print(f'{posted}/{total} envelopes posted')
" >> /tmp/wf-phase3-substrate-snapshot.txt

# Conductor maturity status
echo "--- Conductor ---" >> /tmp/wf-phase3-substrate-snapshot.txt
curl -s -o /dev/null -w "health: %{http_code}" http://127.0.0.1:8141/health \
  >> /tmp/wf-phase3-substrate-snapshot.txt
echo "" >> /tmp/wf-phase3-substrate-snapshot.txt
echo "CONDUCTOR_DISPATCH_ENABLED: ${CONDUCTOR_DISPATCH_ENABLED:-NOT_SET}" \
  >> /tmp/wf-phase3-substrate-snapshot.txt

cat /tmp/wf-phase3-substrate-snapshot.txt
```

---

## Hand-off to Phase 4 hardening

Phase 4 receives the following state when Day 26 closes:

**Required to be true (Phase 4 cannot start without these):**

1. Track 1 complete: stcortex consumer `workflow-trace` registered with `stale=false`; at least one memory in `workflow_trace_main` namespace; m2/m9/m13 unit tests pass
2. Track 3 complete: m40 outbox JSONL exists; at least one `WorkflowEvent::Promote` emitted; m40 unit tests pass
3. Track 5 complete: `/reinforce` call reaches POVM; AP30 collision check clean; m42 unit tests pass
4. Track 4 complete: `lcm.ping` returns pong; at least one `lcm.loop.create` succeeded; m41 unit tests pass
5. CC-5 loop closure attempted and documented (either FIRST-MEASURED-CLOSURE or CLASS-I-ACTIVE noted)
6. Substrate condition snapshot filed at `/tmp/wf-phase3-substrate-snapshot.txt`

**May remain deferred (Track 2 Conductor dependency):**

- Track 2 live dispatch wire may be in refuse-mode if Conductor Waves 1B/1C/2/3 have not been started by Luke. Phase 4 hardening can begin; live dispatch integration tests are Phase 4 stretch goals pending Luke's Wave bring-up.

**CC-5 loop closure timestamp (fill in when measured):**

```
CC-5 first measured closure: _______________
learning_health delta: _______________
Watcher Class-I flag: CLEARED / ACTIVE
```

**Open Watcher flags entering Phase 4:**

| Flag | Class | Status |
|---|---|---|
| Conductor Wave bring-up | Class A | Pre-positioned; fires when Luke starts Wave 1B |
| Hebbian silence (if CC-5 failed) | Class I | Active or cleared; documented in closure notice |
| Any four-surface drift found | Class D | Must be resolved before Phase 4 closes |
| Any cross-substrate handoff gap | Class B | Documented in substrate snapshot |

---

*Phase 3 recipe · S1001982 · planning-only · HOLD-v2 active · authored 2026-05-17*
*Command-3 librarian lane · back to [[../HOME]] · [[../GOD_TIER_CONSOLIDATION_S1001982]] · [[phase-2B-build-clusters-F-G-H]]*
