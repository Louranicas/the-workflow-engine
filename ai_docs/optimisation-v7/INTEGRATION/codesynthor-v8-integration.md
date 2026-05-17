---
title: CodeSynthor V8 Integration Deep-Dive — workflow-trace V7
date: 2026-05-17 (S1001982)
kind: planning-only · integration deep-dive · expands G5 § CSv8 + Zellij plugin
parent: GENERATIONS/G5-tooling.md
owner: Command (Phase 3 Track 5 wire-up); workflow-trace Rust client + V8 Elixir OTP server
v8_role: Holy Trinity (Rust + TypeScript + Elixir OTP); workflow-trace Rust-only client; reuses existing V8 endpoints
---

# CodeSynthor V8 Integration — workflow-trace V7

> Back to: [[../TASK_LIST_V7_OPTIMISATION.md]] · [[../GENERATIONS/G5-tooling.md]] · [[../GENERATIONS/G4-gold-standard.md]] · [[../ULTRAMAP.md]]
>
> Siblings: [[scaffold-integration.md]] · [[atuin-integration.md]] · [[devops-v3-integration.md]] · [[json-claude-code-optimisation.md]] · [[progressive-disclosure-obsidian.md]]

---

## Overview

CodeSynthor V8 (`codesynthor-v8`, port `:8111`, `/health`, batch 4 per workspace CLAUDE.md, 848 tests, Holy Trinity Rust + TypeScript + Elixir OTP) is the Habitat's spec-generation + confidence-tracking service — the upstream author for T1 Specify in V3's pipeline (per [[devops-v3-integration.md]] T1 row + G4 GAP-Gold-04). For workflow-trace, V8 plays three roles: (1) **confidence sink** — m40 emits a single `POST :8082/api/v8/learning {confidence_delta}` (routed via V3, NOT direct V8 — per G4 § V8 ↔ V3 wire reuse, the existing wire is reused); (2) **confidence source** — m31 selector consults `GET :8111/api/v8/confidence/{workflow_id}` when selecting between candidates (advisory, not authoritative); (3) **plugin host** — proposed `wf-status` Zellij plugin reads V8 sphere registration to surface workflow-trace's PV2 sphere state alongside V8's own sphere display. The Holy Trinity context matters because: workflow-trace is Rust-only; V8's TypeScript layer (UI) and Elixir OTP layer (self-healing supervisor tree per CodeSynthor lineage notes in MEMORY.md `reference_codesynthor_lineage_v5_v7_v8.md`) are out of scope — we touch only the HTTP REST surface that all three Trinity sides serve. Activation: Phase 3 Track 5 wire-up (Day 22-26); sphere registration on first `wf-crystallise propose accept` (Phase 5A); plugin proposal opt-in post-D60.

---

## HTTP REST wire spec (`:8111`)

Per G5 § CodeSynthor V8 Zellij plugin integration "Wire" line. Two endpoints — both already exist (workflow-trace is consumer, V8 is provider; zero new V8 server work needed per G5 closure):

### Endpoint 1 — POST :8082/api/v8/learning (via V3, NOT direct V8)

Per G4 wire diagram: workflow-trace m40 emits to **V3's** `/api/v8/learning` proxy, which routes to V8 internally. This is the **single outbound** integration point.

```text
HTTP POST http://localhost:8082/api/v8/learning
Headers:
  content-type: application/json
  authorization: <FNV-1a single-user token>   (Phase 5C+; HMAC-SHA256 in multi-user Phase 7)
Body:
  {
    "workflow_id":       "<m32 dispatch_id>",
    "outcome":           "PassVerified" | "Pass" | "Blocked" | "Fail" | "Refused" | "ConfidenceGateFail",
    "confidence_delta":  <float in [-0.10, +0.10]>,
    "source":            "workflow-trace m32",
    "ts":                <utc ms epoch>
  }
Response: 200 OK { "accepted": true, "v8_confidence_new": <float> }
         | 4xx if auth fails or shape drifts (per [[devops-v3-integration.md]] V3-06)
         | 5xx if V8 internal model unavailable — m40 outbox replay loop handles
```

The delta-to-outcome map is owned by m32 dispatcher (per [[devops-v3-integration.md]] § Outcome → confidence_delta map). m40 is a dumb emitter — it serialises and POSTs.

### Endpoint 2 — GET :8111/api/v8/confidence/{workflow_id}

Per G5 § CSv8 wire (direct V8, not via V3 — the read-path is direct). Used by m31 selector as one input to its weighted decision:

```text
HTTP GET http://localhost:8111/api/v8/confidence/<workflow_id>
Headers: none required (read-only)
Response: 200 OK
  {
    "workflow_id":       "<id>",
    "v8_confidence":     <float in [0.0, 1.0]>,
    "n_observations":    <int>,
    "last_updated_ts":   <utc ms epoch>,
    "source_count":      { "workflow-trace": <n>, "user-direct": <n>, … }
  }
  | 404 if V8 has no model for this workflow_id (m31 treats as confidence=0.5 prior)
```

**m31 weighting (per ULTRAMAP m31 row formula):**
```text
score = α·fitness + β·recency + γ·frequency + δ·diversity
      = 0.40·fitness + 0.25·recency + 0.20·frequency + 0.15·diversity
```
**v8_confidence is NOT in the weighting formula.** Per G4 Axis 4 (no evolution layer at M0), V8 confidence is **advisory** — surfaced in CLI output and Zellij plugin pane, but selector decisions read m14 lift + m11 decay only. Rationale: m14+m11 are workflow-trace's own substrate measurements; V8 confidence is upstream and could drift independently. M2+ optional `feature = "v8-confidence-weighted"` could promote v8_confidence into the formula post-substrate-readiness check.

---

## Rust client → Elixir OTP server (Holy Trinity context)

Per G5 § CSv8 "CSv8 Holy Trinity (Rust + TS + Elixir OTP): workflow-trace Rust-only client; reuses existing V8 endpoints; no new V8 work needed".

### Why this matters

The V5→V7→V8 lineage (per MEMORY.md `reference_codesynthor_lineage_v5_v7_v8.md`) shows ~98% mass reduction was the cure — V5's self-healing stub became Elixir OTP's supervisor tree in V8. **Workflow-trace must NOT touch the Elixir layer** — that's V8's substrate-integrity. workflow-trace touches only the HTTP boundary that the OTP supervisor tree backs.

### Rust client shape (workflow-trace side; m40 + m31)

m40_synthex_emit's bridge to V8 (per G4 convergent pattern #10 `mN_bridges/<peer>.rs` with circuit breaker):

```text
src/m40_bridges/codesynthor_v8.rs    (~80 LOC + tests)
  - struct CodesynthorV8Client { http: reqwest::Client, breaker: tokio_circuit_breaker, base_url: String }
  - async fn post_learning(&self, req: LearningRequest) -> Result<LearningResponse, V8Error>
  - Breaker config: 5 failures / 30s window → OPEN for 60s → HALF_OPEN with 1 probe
  - Timeout: 2s connect + 5s response (Elixir OTP supervisor recovery time budget)
```

m31_selector's read-side bridge:

```text
src/m31_bridges/codesynthor_v8.rs    (~50 LOC + tests; thinner — read-only)
  - async fn get_confidence(&self, workflow_id: &str) -> Result<V8Confidence, V8Error>
  - Cache: in-memory LRU 256 entries, 30s TTL (m31 selector calls in tight loop)
  - On breaker OPEN: returns cached value if fresh, else V8Confidence::Unknown (treated as 0.5 prior)
```

### Holy Trinity invariant: never touch TypeScript / Elixir source

If a Phase 5C bug surfaces in V8 (e.g., confidence model drifts off), workflow-trace files an incident_war_room invocation (per workspace `/incident-war-room` skill) — does NOT patch V8. Workflow-trace owns its m40+m31 client only.

---

## m32 PassVerified → V8 confidence delta path (the full chain)

End-to-end illustration (synthesises G4 § V8 ↔ V3 wire reuse + ULTRAMAP V2 m32+m40 rows):

```text
[1] User-authored workflow dispatched by wf-dispatch
       │
[2] m32 dispatches via Conductor (B3 blocker resolved per project CLAUDE.md)
       │
[3] Workflow executes; m33 verifier 4-agent gate runs (7-day TTL)
       │
[4] m32 receives DispatchOutcome::PassVerified (all 4 agents APPROVE)
       │
[5] m32 calls m40_synthex_emit.emit_nexus_event(workflow_id, "PassVerified", +0.10)
       │
       ├── (5a) m40 writes outbox JSONL: src/m40_outbox/<ts>_<workflow_id>.json
       │       (outbox-first per ULTRAMAP m40 row — durable before any network call)
       │
       ├── (5b) m40 POSTs to SYNTHEX :8092/v3/nexus/push (existing wire)
       │
       └── (5c) m40 POSTs to V3 :8082/api/v8/learning (NEW Phase 3 Track 5)
              │
              ├── V3 forwards to V8 internal confidence model
              │
              ├── V8 updates Hebbian-grain confidence
              │
              └── V8 calls V3 :8082/api/v8/confidence (existing wire — cache update)
                       │
                       └── Future T1 Specify calls inherit updated V8 confidence
[6] m40 outbox row marked sent_at = <ts>
[7] On reconnect after V3/V8 unavailability: outbox replay loop scans for sent_at IS NULL rows
```

**Why outbox-first:** per G4 LCM Drift #6 (bridge contract drift) + LCM Drift #2 (fabricate commits) — the durable outbox is the audit ledger. If V3/V8 ever loses a delta, the outbox proves workflow-trace emitted it.

---

## Sphere registration (PV2 :8132)

Per G5 § CSv8 sphere registration line. Workflow-trace registers a PV2 sphere on first `wf-crystallise propose accept` (i.e., first m23→m30 admission):

```text
Sphere ID:    workflow_trace_proposer
Sphere kind:  workflow-author
Coupling:     freq_band=0.05  (slow oscillation; workflow proposals are days-scale, not seconds)
Initial state: Working (per habitat-sphere-heartbeat pattern — convergence fix + LTP enabler)
Frequency:    spans 0.05-0.40 (wide spread per habitat-sphere-persistent script)
```

Registration request (one-shot at first admission):
```text
HTTP POST http://localhost:8132/spheres/register
Body: { id: "workflow_trace_proposer", kind: "workflow-author", freq_band: 0.05, initial_state: "Working" }
Response: 201 Created
```

Heartbeat (per habitat-sphere-heartbeat reuse):
```text
HTTP POST http://localhost:8132/spheres/<id>/heartbeat
Body: { state: "Working" | "Idle" | "Reading", ts: <utc_ms> }
Cadence: 30s while wf-crystallise running; on-stop sets Idle
```

**Why sphere registration matters:** the V8 Zellij plugin (below) reads PV2 sphere state alongside V8's own internal state; sphere registration makes workflow-trace's existence **legible** to the visualisation layer without coupling to V8's internals.

---

## Zellij plugin proposal — `wf-status` pane

Per G5 § CSv8 Zellij plugin "wf-status" line — full specification:

### Plugin name + binary

- **Name:** `wf-status`
- **Source language:** Rust (workflow-trace owns it as Rust-only — Holy Trinity invariant preserved)
- **Compilation target:** WebAssembly (WASM) per Zellij plugin requirement (per workspace `/zellij-mastery` skill notes: 14 WASM plugins live, 7 floating + 2 background + 5 available)
- **Output binary:** `~/.local/bin/wf-status-plugin.wasm` (~150KB stripped target)
- **KDL inclusion:** opt-in pane in `swarm-orchestrator.kdl` — NOT default; Luke enables via `/zellij-mastery` reconfiguration
- **Build command (post-G9):** `cargo build --release --target wasm32-wasi --bin wf-status-plugin`

### Display surfaces (5)

Per G5 § CSv8 plugin "Displays" line:

```text
┌─ wf-status ──────────────────────────────────────────────┐
│ G-gates: G1✅ G2✅ G3✅ G4✅ G5✅ G6✅ G7✅ G8✅ G9✅      │   ← gate state
│ Wave: wave-3-complete (sha 4f3a9…)                       │   ← current Wave SHA
│ Watcher 5m EMA: A=1 B=0 C=2 D=0 E=0 F=0 G=0 H=0 I=0     │   ← Watcher A-I flag counts
│ m14 lift: mean=0.073 n=47 wilson_lower=0.041             │   ← evidence layer
│ m11 decay floor: 0.018 (Phase 1 PASSING > 0.015)         │   ← trust layer health
│ V8 conf (last dispatch): 0.847                           │   ← upstream confidence (advisory)
│ Bridges UP: 5/5 (V8 V3 SX LCM POVM)                      │   ← peer health
└──────────────────────────────────────────────────────────┘
```

### Update cadence

- **5s** via plugin pipe (per Zellij WASM plugin contract; `/zellij-mastery` skill describes pipe protocol)
- Backend: plugin polls localhost endpoints + reads SQLite — all reads are local IPC, no network

### Data sources (per surface)

| Surface | Source |
|---|---|
| G-gates | `the-workflow-engine/.gate-state.json` (V7-defined; updated by per-gate authors) |
| Wave SHA | `git log --oneline -1` in workflow-trace/ |
| Watcher EMA | `watcher list --since 5m --format json` (per workspace CLAUDE.md persona row) |
| m14 lift | local SQLite `~/.local/share/workflow-trace/db.sqlite` — query m14 view |
| m11 decay | same DB — query m11_decay_floor view |
| V8 confidence | `GET :8111/api/v8/confidence/<last_dispatch_id>` |
| Bridges UP | `wt-bridge-check` script invocation (cached 30s) |

### Plugin permissions (Zellij plugin manifest)

```text
permissions: [
  "ReadApplicationState",     # for current tab/pane context
  "WriteToStdout",            # for rendering surfaces
  "RunCommands",              # for spawning wt-bridge-check / curl / sqlite3
  "ReadOnlyFs"                # for reading .gate-state.json + db.sqlite
  # NO WriteToFs, NO Network — plugin is render-only
]
```

---

## WASM compilation target for plugin

Per G5 § CSv8 "WASM compilation target" line. The plugin compiles to `wasm32-wasi`:

```text
Cargo target:        wasm32-wasi
Feature gate:        --features zellij-plugin   (excludes native deps not WASM-compatible)
Excluded crates:     reqwest (HTTP), sqlx (SQLite native) — plugin uses zellij_tile bindings for IPC and local file IO
Substitutes:         zellij_tile::run_command for spawning wt-bridge-check; std::fs::read for SQLite-via-CLI
Bundle size budget:  ≤ 200 KB stripped (per Zellij plugin convention)
```

**Why this works:** plugin doesn't need direct HTTP / SQLite — it shells out to existing CLI tools (sqlite3, curl, watcher, wt-bridge-check) and reads JSON output. Same pattern as the existing 14 Zellij plugins (`/zellij-mastery` skill catalog).

---

## Failure modes (≥3)

| ID | Failure | Detection | Mitigation |
|---|---|---|---|
| **CS-01** | V8 `:8111` unreachable (devenv stopped V8) | curl timeout in m31 get_confidence | breaker OPEN; m31 falls back to last cached confidence OR V8Confidence::Unknown (0.5 prior); Class-C Watcher flag fires (degraded but safe) |
| **CS-02** | V8 confidence model drifts off (returns nonsensical values) | Phase 5C weekly Watcher independent recompute (per G4 Drift #7) | Watcher files Class-D flag; incident_war_room invocation; workflow-trace does NOT patch V8 (Holy Trinity invariant); workflow-trace's m31 reverts to non-V8-weighted formula (already the M0 default) |
| **CS-03** | Sphere registration race (two `wf-crystallise propose accept` calls in same second) | PV2 returns 409 Conflict | idempotent registration — workflow-trace's first registration wins; subsequent calls observe 409 and proceed |
| **CS-04** | Zellij plugin pipe stale (plugin crashed or pipe broken) | wf-status pane shows last-known surface for >30s | Zellij auto-restart on plugin crash (per `/zellij-mastery` skill); plugin reads .gate-state.json fresh on restart |
| **CS-05** | WASM bundle bloat (plugin >200 KB) | `wasm-strip wf-status-plugin.wasm && ls -la` | exclude reqwest/sqlx; use `run_command` for native shell-outs; if still oversize, split into 2 plugins |
| **CS-06** | V8 outcome shape drifts (V8 adds field; m40 sends old shape) | bridge-contract drift report at Wave-end | per [[devops-v3-integration.md]] V3-06 mitigation; ALL V3+V8 calls go through V3 proxy, so V3-version pinning protects |

---

## Atuin trajectory

```bash
# Phase 3 Track 5 wire-up verification
atuin search "POST.*api/v8/learning" --before 1d
atuin search "GET.*api/v8/confidence" --before 1d

# Sphere registration verification
atuin search "POST.*spheres/register" --before 30d | grep workflow_trace_proposer

# Plugin install + KDL inclusion
atuin search "wf-status-plugin.wasm" --before 30d
atuin search "swarm-orchestrator.kdl" --before 30d | grep wf-status

# Bridge contract drift retrospective
~/.local/bin/bridge-contract workflow-trace codesynthor-v8
```

---

## Verification commands

```bash
# V8 health pre-wire-up
curl -s -o /dev/null -w "%{http_code}\n" --max-time 1 http://localhost:8111/health  # expect 200

# V3 + V8 wire smoke (post-Wave-3, post-binary-deploy)
wf-dispatch --workflow-id "test_v8_smoke_001" --dry-run --simulate-passverified
# Expect: outbox JSONL row; V3 200; V8 confidence query shows delta applied

# m31 confidence read
curl -sS http://localhost:8111/api/v8/confidence/test_v8_smoke_001 | jq

# Sphere registration verification
curl -sS http://localhost:8132/spheres/workflow_trace_proposer | jq

# Zellij plugin smoke (post-D60, after plugin authored)
ls -la ~/.local/bin/wf-status-plugin.wasm                          # expect ≤200KB
wasm-objdump -x ~/.local/bin/wf-status-plugin.wasm | head          # validate WASM

# Plugin runtime in Zellij
zellij action launch-or-focus-plugin file:~/.local/bin/wf-status-plugin.wasm

# Bridge contract static check
~/.local/bin/bridge-contract workflow-trace codesynthor-v8
```

---

## Sign-off

✅ codesynthor-v8-integration spec complete. HTTP REST wire spec (POST :8082/api/v8/learning via V3 + GET :8111/api/v8/confidence direct) defined. Rust client → Elixir OTP server boundary respected (Holy Trinity invariant: workflow-trace touches only HTTP REST surface; never Elixir OTP / TypeScript source). m32 PassVerified → V8 confidence delta full 7-step chain documented with outbox-first durability. PV2 sphere registration (workflow_trace_proposer) specified with heartbeat cadence. Zellij `wf-status` plugin proposed: 5 surfaces, 5s cadence, opt-in KDL, WASM wasm32-wasi target ≤200KB. 6 failure modes (≥3 target met) with mitigations. Atuin trajectory + verification commands deterministic.

*Authored 2026-05-17 (S1001982) — Command for V7 G5 expansion. Phase 3 Track 5 wire-up + sphere registration Phase 5A; Zellij plugin opt-in post-D60. HOLD-v2 respected: planning-only.*
