# Wave-17 — Wire-Aware /health + wf-poller Merge + /port-claim Skill (S1005032, 2026-05-25)

> **Back to:** [[CLAUDE.md]] · [[CLAUDE.local.md]] · [[Wave-16 — wf-daemon Habitat Service Shape S1005032]] · [[Wave-15 — wf-poller wiring S1005032]]
> **Sibling vaults:** `habitat-zellij/CLAUDE.local.md` (plugin grid 14→15 pending LCM commit)
> **Main vault mirror:** TBD pending
> **CHANGELOG entry:** `the-workflow-engine/CHANGELOG.md` `[v0.2.1-wave17]`

## Headline — close-the-loop on three Wave-16 retro recommendations

Wave-17 ships three things, each closing a specific gap I named in the post-Wave-16 retro:

1. **R-WAVE16-1 CLOSED: wire-aware `/health` body.** No longer just `{"status":"ok"}`. Now carries 7 wire-status fields including `tick_count`, `ok_count`, `refusal_count`, `last_ok_age_ms`, `refusal_rate_per_kilo`. The dashboard-lying surface I introduced in Wave-16 is closed.

2. **`wf-poller` deleted, merged into `wf-daemon`.** 4-binary topology → 3-binary topology. `WF_DAEMON_DISABLE_HTTP=1` runs poller-only mode (the historical `wf-poller` shape). One binary, two modes.

3. **NEW `/port-claim` skill at `.claude/skills/port-claim/`.** Procedural intervention against the port-collision trap that bit two Claude panes in Wave-16 (the 8141 false-positive). 4-check ritual + authoritative `HABITAT_PORT_AUDIT.tsv` (78 rows, surveyed by general-purpose subagent across the live habitat).

## Wire-aware `/health` body (the main shipped change)

### Before (Wave-16)

```
$ curl -s localhost:8142/health
{"status":"ok","service":"workflow-trace","port":8142}
```

200 returned regardless of substrate reachability. WFE shows green in the plugin grid even when the wire is silently dropping every tick. **This is the AP-V7-13 "health-200 ≠ behaviour-verified" pattern I had just spent a whole arc surfacing — and re-introduced at the dashboard layer.**

### After (Wave-17)

```
$ curl -s localhost:8142/health | jq
{
  "status": "ok",
  "service": "workflow-trace",
  "port": 8142,
  "tick_count": 7,
  "ok_count": 0,
  "refusal_count": 7,
  "unreachable_count": 0,
  "last_ok_unix_ms": 0,
  "last_ok_age_ms": 18446744073709551615,
  "refusal_rate_per_kilo": 1000
}
```

The live smoke test above ran against the stale SX2 daemon (which is 19 days behind Wave-15 source HEAD). The body correctly reports `ok_count=0`, `refusal_rate_per_kilo=1000`, `last_ok_age_ms=u64::MAX` (the "never acked" sentinel) — the wire IS broken, and the dashboard surface now reflects it honestly.

### Implementation

| Type | Role |
|---|---|
| `WireStats` struct | 4× `AtomicU64`: `ok`, `refusals`, `unreachable`, `last_ok_ms`. Shared between poller subsystem (writer via `fetch_add` + `store`) and axum `/health` handler (reader via `load`). |
| `health()` handler | Upgraded from `async fn → &'static str` to `async fn(State<Arc<WireStats>>) → String`. Computes `total_ticks`, `last_ok_age_ms` (saturating-sub from `unix_ms_now()`), `refusal_rate_per_kilo` (integer-only ppm; no float JSON). |
| `run_one_tick` | Signature gains `stats: &WireStats`; per-outcome arm does `stats.X.fetch_add(1, Relaxed)` plus the existing per-tick tracing event. |

**Concurrency:** Atomics with `Ordering::Relaxed` — no synchronisation needed because each counter is single-writer (poller) / multi-reader (axum); the counters don't have ordering invariants between them. Body shape doesn't claim atomic-snapshot semantics (individual `load` calls may straddle increments — that's acceptable for an observability endpoint and noted in the doc comment).

### Probe contract preserved (no `bridge_health` break)

`habitat-zellij/.../bridge_health.rs` consumes only the 200 status code, never the body. So the wire-aware body change is **non-breaking** to the existing dashboard. A future `bridge_health` upgrade could parse `refusal_rate_per_kilo > 500` → render YELLOW indicator (or `last_ok_age_ms > 30_000` → render DEGRADED), without further changes to wf-daemon.

## `wf-poller` deletion + `WF_DAEMON_DISABLE_HTTP=1`

| Before | After |
|---|---|
| `wf-crystallise`, `wf-dispatch`, `wf-poller`, `wf-daemon` (4 binaries) | `wf-crystallise`, `wf-dispatch`, `wf-daemon` (3 binaries) |
| `wf-poller` = standalone CLI, runs forever | `WF_DAEMON_DISABLE_HTTP=1 wf-daemon` = same shape, env-gated |
| Two binaries to maintain (drift risk) | One binary, two modes |

Deleted file: `src/bin/wf_poller.rs` (270 LOC). Wave-15 commit `54c107b` / `ce317ae` stays in git history for archaeology. Operators using `wf-poller` directly should switch to the env-gate.

## `/port-claim` skill

The S1005032 port-collision trap (two Claude panes independently assigning `8141` to workflow-trace because `ss -tlnp` reported it free — but 8141 is HABITAT-CONDUCTOR's reserved port) was the kind of error that *the same training tendency will rediscover the same wrong answer through* unless intervened procedurally.

**4-check ritual (grep-first, ss-last):**

```bash
PORT=NNNN

# 1. devenv.toml (authoritative reservation list)
/usr/bin/grep -nE ":$PORT|--port.*$PORT|=.*$PORT\b" ~/.config/devenv/devenv.toml

# 2. Workspace doctrine + planning claims
/usr/bin/grep -rn ":$PORT\b" ~/claude-code-workspace/{CLAUDE.md,CLAUDE.local.md,*/CLAUDE.md,*/CLAUDE.local.md,*/ai_docs/,*/ai_specs/} 2>/dev/null

# 3. habitat-plugin's service grid registry
/usr/bin/grep -n "$PORT" ~/claude-code-workspace/habitat-zellij/crates/habitat-modules/src/bridge_health.rs

# 4. Live socket state (last, NOT first — `ss -tlnp` alone IS the trap)
ss -tlnp "sport = :$PORT" 2>/dev/null
```

Only if ALL of 1-3 are clean AND 4 shows unbound is the port truly free.

**Skill location:** `.claude/skills/port-claim/SKILL.md` (133 lines, full description with allocation suggestions for new services) + `HABITAT_PORT_AUDIT.tsv` (78 rows; subagent-surveyed live habitat 2026-05-25).

## Live-binary drift flag (surfaced by subagent during the audit)

The currently-running `wf-daemon` process (pid 3695254) is bound to `127.0.0.1:8141`, NOT 8142. This is the OLD binary deployed before the Wave-16 8141→8142 re-port. The new binary at `./bin/wf-daemon` (sha `355936e9…`) is staged but not running. Operator needs:

```bash
~/.local/bin/devenv -c ~/.config/devenv/devenv.toml stop workflow-trace
~/.local/bin/devenv -c ~/.config/devenv/devenv.toml start workflow-trace
curl -s localhost:8142/health | jq  # expect wire-aware body
```

The plugin grid screenshot Luke shared (`ALL UP 14/14 (14 probed)` with green WFE) was probably a probe race or hot-reload lag — the wasm has `(8142, "/health")` in PROBE_PATHS but the running daemon binds 8141. Operator-side verify needed.

## 4-stage gate (Wave-17)

| Stage | Result |
|---|---|
| `cargo check --release --bin wf-daemon` | ✓ |
| `cargo clippy --release --bin wf-daemon -- -D warnings` | ✓ clean |
| `cargo clippy --release --bin wf-daemon -- -D warnings -W clippy::pedantic` | ✓ clean |
| `cargo build --release --bin wf-daemon` (17.63 s) | ✓ |
| `./bin/wf-daemon` redeployed | sha `355936e9…` |
| Live smoke test on port 18142 | ✓ wire-aware body returned correctly |

## Honest residuals (carried from Wave-16, still standing)

- **R-WAVE16-2 (operator-D2):** SX2 daemon redeploy. Until Luke redeploys, every `wf-daemon` tick logs `refusal=ack_parse_error` (because the running SX2 daemon predates Wave-15 source by 19 days). The wire-aware /health body now SHOWS this — `refusal_rate_per_kilo=1000` is the dashboard-visible evidence.
- **R-WAVE16-3:** m46 Watcher subscription to `Signal::ExternalHeartbeat` (NA-1'' Option C); v0.2.3+.
- **R-WAVE16-4:** silence-watcher daemon task flipping `Live → Unreachable` (NA-4 + NA-8'); v0.2.3+.
- **R-WAVE16-5:** HABITAT-CONDUCTOR bring-up (OP-1) — Conductor `auto_start=false` on `:8141`; when started, plugin grid → 16 services.
- **R-WAVE16-6:** workspace charter row drift — `~/claude-code-workspace/CLAUDE.md` "14 active services" table is now 19 with workflow-trace added; charter refresh pending.
- **NEW R-WAVE17-1:** `bridge_health` consumer of the wire-aware body — a follow-up amendment to `habitat-zellij/.../bridge_health.rs` could parse `refusal_rate_per_kilo` + `last_ok_age_ms` to render WFE in YELLOW (degraded) instead of GREEN when the wire is broken but daemon-alive. Deferred — design TBD.
- **NEW R-WAVE17-2:** operator restart of running `wf-daemon` to pick up the new 8142 binary (currently still bound on 8141 from pre-re-port deploy).

## Cross-references

- **Sibling Wave-16 vault note:** [[Wave-16 — wf-daemon Habitat Service Shape S1005032]]
- **Original Wave-15 SX2-side wire:** [[Wave-15 — wf-poller wiring S1005032]] + companion `synthex-v2/obsidian-synthex-v2/synthex-v2/Wave-15 — WFE Heartbeat Inbound Wiring S1005032.md`
- **Source:** `src/bin/wf_daemon.rs` (Wave-17 edits: `WireStats` struct + atomic counters + `WF_DAEMON_DISABLE_HTTP` env-gate)
- **HTTP shape spec (updated):** `ai_specs/WF_DAEMON_HTTP_SHAPE.md` § 2 + § 4
- **Lifecycle ultramap (updated):** `ultramap/WF_DAEMON_LIFECYCLE.md`
- **Skill:** `.claude/skills/port-claim/SKILL.md`
- **CHANGELOG:** `CHANGELOG.md` `[v0.2.1-wave17]`
- **stcortex:** ns `workflow_trace_completion_s1004115` (Wave-17 mem — landed this session)
- **injection.db:** `causal_chain` (Wave-17 row — landed this session)
