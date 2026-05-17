---
title: G5 — Tooling Integration Pass (Generation 5 of 7)
date: 2026-05-17 (S1001982)
kind: planning-only · deep-dive on /scaffold + atuin + V3 + CSv8 + JSON + Obsidian
purpose: close GAP-Tool-01..06; produce 6 integration deep-dives (separate INTEGRATION/*.md files referenced)
inputs: G1-G4 + ULTRAMAP + workspace CLI inventory (atuin + cc-* + scripts)
output: integration timing + invocation patterns + new atuin scripts + JSON optimisation
---

# G5 — Tooling Integration Pass

> Back to: sibling [[G4-gold-standard.md]] (input) · [[G6-test-discipline.md]] (next)
> Children (full deep-dives): [[../INTEGRATION/scaffold-integration.md]] · [[../INTEGRATION/atuin-integration.md]] · [[../INTEGRATION/devops-v3-integration.md]] · [[../INTEGRATION/codesynthor-v8-integration.md]] · [[../INTEGRATION/json-claude-code-optimisation.md]] · [[../INTEGRATION/progressive-disclosure-obsidian.md]]

---

## Gap closure

| Gap | Closure |
|---|---|
| GAP-Tool-01 (`/scaffold` integration timing) | ✅ invoked Genesis Day 0 + per-Wave-end consistency drift + on-demand |
| GAP-Tool-02 (atuin scripts proposed) | ✅ 7 proposed: wt-gate-status / wt-soak-pulse / wt-substrate-pulse / wt-bridge-check / wt-wave-status / wt-cc5-trace / wt-keystone-bench |
| GAP-Tool-03 (V3 T1-T6 mapping) | ✅ T1→propose / T2→/scaffold / T3→cargo build / T4→QG / T5→cargo doc / T6→wf-dispatch+Conductor |
| GAP-Tool-04 (CSv8 plugin deep spec) | ✅ HTTP REST + Rust↔Elixir wire + sphere registration + Phase 3 Track 5 m32→/api/v8/learning |
| GAP-Tool-05 (Zellij plugin proposal) | ✅ `wf-status` plugin pane with 5 surfaces (gate / Wave SHA / Watcher flags / m14 lift / m11 decay) |
| GAP-Tool-06 (JSON schema for .claude/settings.json) | ✅ proposed settings.json + hook config + .mcp.json additions |

---

## Integration summary tables (full deep-dives in INTEGRATION/*.md)

### 1. `/scaffold` integration
- **Owner:** Command at Genesis Day 0; Command at Wave-end (consistency drift); on-demand any session
- **Cadence:** Day 0 (initial) + per Wave-end (drift check) + on-demand (manual audit)
- **Input:** `plan.toml`
- **Output:** scaffolded src/ tree consistent with plan.toml; drift report if mismatch
- **Invocation:** `/scaffold` (Claude Code skill); or `~/.local/bin/scaffold` direct CLI
- **Failure mode:** plan.toml drifts from src/ — scaffold reports diff; Command files D-drift in agent-cross-talk/
- **Source skill:** `scaffold-mastery` skill, `genesis` skill

### 2. Atuin integration
**7 proposed workflow-trace scripts:**

| Script | Purpose | Invocation | Phase |
|---|---|---|---|
| `wt-gate-status` | report G1-G9 state + current Wave SHA + active worktrees | `atuin scripts run wt-gate-status` | continuous |
| `wt-soak-pulse` | 30s-interval probe of m14 lift mean + n + Wilson CI | `atuin scripts run wt-soak-pulse` | Phase 5C |
| `wt-substrate-pulse` | LTP/LTD + RALPH gen + fitness + Watcher class flag counts | `atuin scripts run wt-substrate-pulse` | continuous |
| `wt-bridge-check` | probe 5 substrate peers (V8/V3/SYNTHEX/LCM/POVM) | `atuin scripts run wt-bridge-check` | per-Wave-end + post-deploy |
| `wt-wave-status` | active worktrees + lock files + time-budget consumption | `atuin scripts run wt-wave-status` | Wave 1-3 |
| `wt-cc5-trace` | CC-5 first-closure trace (m32 → m40 → SYNTHEX → pathway.weight delta) | `atuin scripts run wt-cc5-trace` | Phase 3 Day 26 |
| `wt-keystone-bench` | m20 PrefixSpan criterion bench delta (gen N vs gen N-1) | `atuin scripts run wt-keystone-bench` | post-Wave-3 |

**Existing reuse:** `habitat-intel`, `habitat-sweep`, `habitat-bridge-check`, `hab-quality-gate`, `habitat-evolution-delta` — all referenced from workflow-trace runbooks.

**Provenance principle (S1002029 #4):** every workflow-trace shell invocation lands in `~/.local/share/atuin/history.db`. atuin is the ONLY cross-tool ledger.

### 3. DevOps V3 integration
- **T1 Specify** → `wf-crystallise propose accept <id>` (m23 → m30 admission)
- **T2 Scaffold** → `/scaffold` consistency check (drift report if any)
- **T3 Implement** → `cargo build --workspace --release` (worktree-isolated)
- **T4 Harden** → 4-stage QG + 4-agent pre-deploy hardening (per Phase 4 runbook)
- **T5 Document** → `cargo doc --no-deps --workspace` + vault MOC update
- **T6 Deploy** → `wf-dispatch <id>` (m32 → Conductor)

**NAM-03 confidence gates:** each T-stage emits confidence; V3 thresholds (T1≥0.80, T2≥0.80, T3≥0.85, T4≥0.85, T5≥0.90) per V3 spec. Failure at any → halt.

**`resume_from` usage:** if m32 dispatches a deploy workflow, m41 LCM router uses `lcm.loop.create {max_iters: 1, resume_from: "T2"}` so V8+Zen own T1 (spec), V3 owns T2-T6 trajectory.

### 4. CodeSynthor V8 Zellij plugin integration
- **Wire:** HTTP REST `:8111`
  - `POST /api/v8/learning` — receive confidence delta (workflow-trace → V8)
  - `GET /api/v8/confidence/{workflow_id}` — read current V8 confidence (workflow-trace ← V8)
- **Sphere registration:** workflow-trace registers a PV2 sphere on first `wf-crystallise propose accept`; sphere ID = `workflow_trace_proposer`
- **Zellij plugin `wf-status` proposal:**
  - Pane name: `wf-status`
  - Source: `~/.local/bin/wf-status-plugin.wasm` (compiled to WASM target)
  - Updates: every 5s via plugin pipe
  - Displays: G-gate state, current Wave SHA, Watcher A-I flag counts (5-min EMA), m14 lift mean, m11 decay floor
  - KDL inclusion: `swarm-orchestrator.kdl` opt-in pane (NOT default — Luke enables via `/zellij-mastery`)
- **CSv8 Holy Trinity** (Rust + TS + Elixir OTP): workflow-trace Rust-only client; reuses existing V8 endpoints; no new V8 work needed

### 5. JSON Claude Code optimisation
**`.claude/settings.json` (workspace-scoped, pre-G2-rename in current dir; moves on rename):**
```json
{
  "$schema": "https://claude.com/schemas/settings.json",
  "permissions": {
    "allow": [
      "Read",
      "Edit",
      "Write",
      "Bash(cargo *)",
      "Bash(atuin scripts run wt-*)",
      "Bash(./scripts/verify-sync.sh*)",
      "Bash(./scripts/gate.sh)",
      "Bash(./scripts/wave-end-checklist.sh*)",
      "Bash(./scripts/wave-merge.sh*)"
    ],
    "deny": [
      "Bash(rm -rf*)",
      "Bash(docker container prune*)",
      "Bash(git push --force*)",
      "Bash(cp -f*)"
    ]
  },
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "~/.local/bin/cc-guard-bash",
            "comment": "Tier-3 guard: block AP-Hab-04 preserve-list violations, AP-Hab-06 cp alias, AP24 pre-G9 cargo init"
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": "Edit|Write",
        "hooks": [
          {
            "type": "command",
            "command": "~/.local/bin/rust-postedit-gate",
            "comment": "Tier-1: per-file cargo check on edited .rs files (fast)"
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "./scripts/gate.sh",
            "comment": "Tier-2: full 4-stage QG before session-stop (per Working Mode)"
          }
        ]
      }
    ]
  },
  "env": {
    "CARGO_TARGET_DIR": "./target",
    "RUST_BACKTRACE": "1",
    "RUSTFLAGS": "-D warnings"
  }
}
```

**`.mcp.json` additions (post-G9, when MCP tools exist):**
```json
{
  "mcpServers": {
    "workflow-trace-status": {
      "command": "~/.local/bin/wf-mcp-server",
      "args": ["--mode", "status"],
      "env": {}
    }
  }
}
```

**Hook payload contract:**
- PostToolUse for Edit/Write on `*.rs`: triggers per-file `cargo check` on touched module only
- Stop hook: runs full 4-stage QG; refuses session-stop if any fail
- PreToolUse Bash: per-pattern guard (see deny list above)

### 6. Progressive disclosure with Obsidian
**3-tier disclosure for `the-workflow-engine-vault/`:**

```
TIER 1 — Landing (always loaded; ≤200 words each)
├── HOME.md                       — "what is this; where to go"
├── MASTER_INDEX.md               — comprehensive catalogue (already exists)
└── KEYWORDS_20.md (symlinked)    — 20-keyword context anchor

TIER 2 — Per-cluster + per-phase (on-demand; ~2000 words each)
├── cluster-A.md ... cluster-H.md    — module plans
├── runbook-00 ... runbook-11.md      — operational
└── INTEGRATION/*.md                  — tooling deep-dives

TIER 3 — Per-module (deep-load only when implementing; ~500 words each)
├── m1-overview.md ... m42-overview.md
└── (per-module bidi contract from G3, sliced from cluster-N.md)
```

**Bidi-anchor discipline:** every TIER-N doc has `> Back to: [[TIER-N-1 parent]]` AND every TIER-N+1 child link present in parent.

**On-demand reading order:**
1. Open vault → HOME.md loads
2. HOME.md links to MASTER_INDEX
3. Navigate to cluster / phase / integration as needed
4. Per-module: only when about to edit that module's src/

**Tooling:** obsidian-vault-librarian agent runs per-Wave-end sweep verifying:
- Every TIER-1 doc < 200 words
- Every TIER-2 doc < 2500 words
- Every TIER-N+1 has bidi link back to TIER-N
- Zero orphan files (every file linked from at least one other)

---

## Tooling integration time matrix (Phase × Tool)

| Phase | /scaffold | atuin | V3 | CSv8 | settings.json | Obsidian |
|---|---|---|---|---|---|---|
| Phase 0 | (none — HOLD-v2) | session-start `habitat-bootstrap` | (none) | (none) | settings.json planning-spec authored | vault navigation refresh |
| Phase 1 Genesis Day 0 | initial scaffold | session-start | T2 mapping confirmed | (none) | settings.json activated | per-Wave-end sweep |
| Phase 1 Genesis Day 1-3 | (none) | wt-gate-status | T3 builds | (none) | (active) | (active) |
| Phase 2A Build B/C/E | per-Wave-end drift check | wt-wave-status | T3+T4 | (none) | (active) | per-Wave-end |
| Phase 2B Build F/G/H | per-Wave-end | wt-keystone-bench | T3+T4 | (none) | (active) | per-Wave-end |
| Phase 3 Integration | per-Wave-end | wt-cc5-trace | T4 + resume_from contract | Phase 3 Track 5 wire active | (active) | per-Wave-end |
| Phase 4 Hardening | (none) | wt-bridge-check | T4 | (none — read-only confidence) | (active) | per-Wave-end |
| Phase 5A Deploy | (none) | wt-substrate-pulse | T6 + Conductor | sphere registration | (active) | per-Wave-end |
| Phase 5B Cutover | (none) | wt-substrate-pulse | T6 monitor | (active) | (active) | per-Wave-end |
| Phase 5C Soak (Day 30→D120) | weekly drift check | wt-soak-pulse 30s | (continuous) | (continuous) | (active) | weekly Watcher synthesis |
| Phase 6 Sunset D120 | (drift retrospective) | wt-substrate-pulse trend | (final report) | (final report) | (frozen) | D120 retrospective |

---

## G5 substrate-frame pass

**Second-frame question:** what does the substrate "see" of these integrations?

From substrate-frame:
- `/scaffold` produces a substrate-trace (file-tree state captured in `~/.local/share/scaffold/history.db` if any)
- atuin scripts produce substrate-pulse (shell history rows; each script run is an event)
- V3 produces substrate-state-change (deploy DB rows)
- CSv8 produces substrate-confidence-update (V8 internal model)
- settings.json produces substrate-policy (hook firings + permission denials)
- Obsidian produces substrate-document-state (vault file mtimes)

**Substrate-frame distinction:** each integration tool has its own substrate ledger. atuin is the only cross-tool one. The substrate-grain question: do these ledgers cross-correlate?

**Currently NO.** That's GAP-Substrate (catalogue at G6). Mitigation: every workflow-trace shell invocation through atuin (so atuin-derived workflow ID can be looked up in V3 / CSv8 / Obsidian state by correlating timestamps + atuin session ID).

---

## G5 Watcher pre-positioning

**Class H activated.** Atuin proprioception anomaly — workflow-trace shell commands NOT landing in atuin (e.g., if Luke runs `cargo build` outside an atuin-active shell). Mitigated by:
- Every runbook command prefixed with explicit atuin context note
- Per-Wave-end `atuin search "workflow-trace OR wt-*" --before 7d` audit

---

## G5 close

✅ G5 PASS. 6 GAP-Tool entries closed. 7 atuin scripts proposed (will be authored as separate `crystallize` task post-G9). V3 T1-T6 mapping locked. CSv8 wire + Zellij plugin proposed. settings.json schema authored. Obsidian 3-tier progressive disclosure structure defined.

**Output for G6:** tooling integration spec. G6 reads G5 + TEST_DISCIPLINE + ANTIPATTERNS and produces per-module test allocation + mutation kill rates + property-test strategies.

---

*G5 authored 2026-05-17 by Command. 6 integrations + 7 atuin scripts + 3-tier vault + JSON optimisation. atuin = cross-tool provenance.*
