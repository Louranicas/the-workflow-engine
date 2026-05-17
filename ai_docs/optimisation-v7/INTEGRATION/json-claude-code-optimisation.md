---
title: JSON Claude Code Optimisation Deep-Dive — workflow-trace V7
date: 2026-05-17 (S1001982)
kind: planning-only · integration deep-dive · expands G5 § JSON Claude Code optimisation
parent: GENERATIONS/G5-tooling.md
owner: Command (settings.json author); per-hook command-binaries owned by hookify+plugin-dev skills
scope: .claude/settings.json (workspace-scoped) + .mcp.json (post-G9) + hook payload contracts
---

# JSON Claude Code Optimisation — workflow-trace V7

> Back to: [[../TASK_LIST_V7_OPTIMISATION.md]] · [[../GENERATIONS/G5-tooling.md]] · [[../ULTRAMAP.md]]
>
> Siblings: [[scaffold-integration.md]] · [[atuin-integration.md]] · [[devops-v3-integration.md]] · [[codesynthor-v8-integration.md]] · [[progressive-disclosure-obsidian.md]]

---

## Overview

The `.claude/settings.json` + `.mcp.json` pair is the workspace-scoped Claude Code harness configuration — permissions, env vars, hooks, MCP server registrations — and it is the **single highest-leverage substrate-policy surface** for protecting workflow-trace from AP24 (pre-G9 cargo init), AP-Hab-04 (preserve-list violations per S102 docker container prune), AP-Hab-06 (cp alias trap), and Drift #1 (over-claim gate-clean against scoped clippy per [[../GENERATIONS/G4-gold-standard.md]] § LCM Drift transposition). Per G5 § JSON Claude Code optimisation, three planes: (1) **permissions** — allow workflow-trace-specific Bash patterns (cargo, wt-* scripts, gate.sh, wave-end-checklist.sh, verify-sync.sh) and deny categorically dangerous patterns (`rm -rf`, `docker container prune`, `git push --force`, `cp -f`); (2) **hooks** — Tier-1 fast PostToolUse cargo check on edited .rs (`rust-postedit-gate`), Tier-2 full 4-stage QG at Stop (`./scripts/gate.sh`), Tier-3 PreToolUse Bash guard (`cc-guard-bash`); (3) **env** — `CARGO_TARGET_DIR=./target`, `RUST_BACKTRACE=1`, `RUSTFLAGS=-D warnings` to project workflow-trace's quality bar into the substrate-frame. The file is **workspace-scoped, pre-G2-rename in current dir** (`the-workflow-engine/.claude/settings.json`); after G2 rename to `workflow-trace/`, the file follows automatically (path is relative to project root). `.mcp.json` additions are post-G9 only (no MCP tools exist for workflow-trace until then). Plugin manifest is empty for M0 — Zellij `wf-status` plugin is the only proposed plugin and is opt-in (per [[codesynthor-v8-integration.md]]).

---

## Full `.claude/settings.json` spec

Per G5 § JSON Claude Code optimisation literal JSON. Annotated with rationale:

```json
{
  "$schema": "https://claude.com/schemas/settings.json",
  "permissions": {
    "allow": [
      "Read",
      "Edit",
      "Write",
      "Glob",
      "Grep",
      "Bash(cargo *)",
      "Bash(atuin scripts run wt-*)",
      "Bash(atuin scripts run habitat-*)",
      "Bash(atuin scripts run hab-*)",
      "Bash(atuin search *)",
      "Bash(atuin kv get *)",
      "Bash(atuin kv set --namespace wt *)",
      "Bash(./scripts/verify-sync.sh*)",
      "Bash(./scripts/gate.sh)",
      "Bash(./scripts/wave-end-checklist.sh*)",
      "Bash(./scripts/wave-merge.sh*)",
      "Bash(curl -s -o /dev/null -w * http://localhost:*)",
      "Bash(curl -sS http://localhost:*)",
      "Bash(curl -sS -X POST http://localhost:*)",
      "Bash(sqlite3 -header -column*)",
      "Bash(sqlite3 ~/.local/share/workflow-trace/db.sqlite*)",
      "Bash(git status*)",
      "Bash(git diff*)",
      "Bash(git log*)",
      "Bash(git branch*)",
      "Bash(git worktree list)",
      "Bash(~/.local/bin/scaffold check*)",
      "Bash(~/.local/bin/bridge-contract*)",
      "Bash(~/.local/bin/watcher notify*)",
      "Bash(/usr/bin/cp -f target/release/wf-* ~/.local/bin/)"
    ],
    "deny": [
      "Bash(rm -rf*)",
      "Bash(docker container prune*)",
      "Bash(docker volume prune*)",
      "Bash(docker network prune*)",
      "Bash(docker system prune*)",
      "Bash(git push --force*)",
      "Bash(git push -f*)",
      "Bash(git stash pop*)",
      "Bash(cp -f*)",
      "Bash(cargo init*)",
      "Bash(cargo new*)",
      "Bash(~/.local/bin/scaffold sync*)",
      "Bash(setsid*)",
      "Bash(nohup*)",
      "Bash(* &)"
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
            "comment": "Tier-3 guard: block AP-Hab-04 preserve-list violations, AP-Hab-06 cp alias, AP24 pre-G9 cargo init, AP-Hab-05 setsid/nohup child-reaping"
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
            "comment": "Tier-1 fast: per-file cargo check on edited .rs files; emits diagnostic JSON for cc-nvim-context"
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
            "comment": "Tier-2 full 4-stage QG before session-stop (per project CLAUDE.md Working Mode); PIPESTATUS-disciplined per feedback_pipestatus_for_gate_chains.md"
          }
        ]
      }
    ]
  },
  "env": {
    "CARGO_TARGET_DIR": "./target",
    "RUST_BACKTRACE": "1",
    "RUSTFLAGS": "-D warnings",
    "WORKFLOW_TRACE_STCORTEX_NAMESPACE": "workflow_trace_",
    "WORKFLOW_TRACE_PROJECT_ROOT": "."
  }
}
```

### Per-section rationale

**`allow` list rationale:**
- `Read/Edit/Write/Glob/Grep` — core file operations
- `Bash(cargo *)` — full cargo surface (build/test/clippy/doc) post-G9
- `Bash(atuin *)` — full atuin surface; the proprioception layer per [[atuin-integration.md]] § Provenance principle
- `Bash(./scripts/*)` — local scripts only; workspace-scope discipline
- `Bash(curl -s* http://localhost:*)` — local-only HTTP probes (bridge checks, health probes); no external network
- `Bash(sqlite3 *)` — DB inspection per workspace CLAUDE.md Essential Patterns § SQLite
- `Bash(git status/diff/log/branch/worktree list)` — read-only git (no commit/push allowed without explicit per-task elevation)
- `Bash(~/.local/bin/scaffold check*)` — read-only scaffold drift check ONLY; sync is denied (see deny list)
- `Bash(~/.local/bin/bridge-contract*)` — static bridge contract verification (per [[devops-v3-integration.md]])
- `Bash(~/.local/bin/watcher notify*)` — Watcher Communication Protocol per workspace CLAUDE.md persona row
- `Bash(/usr/bin/cp -f target/release/wf-* ~/.local/bin/)` — explicit `/usr/bin/cp` (bypasses `cp → trash` alias trap)

**`deny` list rationale:**
- `Bash(rm -rf*)` — preserve-list discipline (per MEMORY.md `feedback_preserve_list_discipline.md`, S102 lesson)
- `Bash(docker * prune*)` — blanket prune destroys preserved-by-name containers (S102 openclaw-gateway loss)
- `Bash(git push --force*|--f)` — per workspace CLAUDE.md Git Conventions
- `Bash(git stash pop*)` — pre-existing stashes silently wiped uncommitted work (per workspace CLAUDE.md Git Conventions)
- `Bash(cp -f*)` — alias trap to `trash`; force explicit `/usr/bin/cp -f`
- `Bash(cargo init*|cargo new*)` — AP24 enforcement (planning-only pilot)
- `Bash(~/.local/bin/scaffold sync*)` — sync requires Command in main worktree only with explicit confirmation; deny in default settings forces audit trail
- `Bash(setsid*|nohup*|* &)` — sandbox child-reaping per workspace CLAUDE.md Habitat Operations rule "Do NOT spawn services manually"

**`env` rationale:**
- `CARGO_TARGET_DIR=./target` — per-worktree isolation (per AGENT_VIEW_GITWORKTREES.md resource sharing table: target/ NOT shared)
- `RUST_BACKTRACE=1` — diagnostic clarity in test failures
- `RUSTFLAGS=-D warnings` — projects god-tier no-warnings discipline (per MEMORY.md `feedback_god_tier_no_warnings_at_any_level.md`) into the substrate-frame
- `WORKFLOW_TRACE_STCORTEX_NAMESPACE` — AP30 namespace prefix as env constant; m9 namespace guard reads this at runtime
- `WORKFLOW_TRACE_PROJECT_ROOT` — relative path for cross-script consistency

---

## Hook payload contracts

Per G5 § JSON Claude Code optimisation Hook payload contract line. Each hook receives Claude Code's standard JSON payload on stdin; spec what each hook MUST do and SHOULD do.

### PreToolUse Bash → `~/.local/bin/cc-guard-bash`

**Input (Claude Code stdin):**
```json
{
  "tool_name": "Bash",
  "tool_input": {
    "command": "<the bash command>",
    "description": "<active-voice description>",
    "timeout": <optional ms>
  },
  "session_id": "<uuid>",
  "transcript_path": "/path/to/transcript",
  "cwd": "<current working directory>"
}
```

**MUST do:**
1. Parse `tool_input.command`
2. Check against AP24 (cargo init/new + pre-G9 check via `.gate-state.json`)
3. Check against AP-Hab-04 (preserve-list patterns)
4. Check against AP-Hab-06 (`cp -f` without `/usr/bin/` prefix)
5. Check against AP-Hab-05 (`setsid`/`nohup`/bare `&`)
6. Check against PreToolUse hook source skill rules (`hookify:writing-rules`)

**SHOULD do:**
- Log every block to `~/.local/share/cc-guard-bash/blocks.jsonl` for audit
- Emit one-line block reason on stderr for operator clarity

**Exit codes:**
- `0` — allow tool use to proceed
- `2` — block tool use (Claude Code refuses tool invocation; user sees stderr message)
- Any other non-zero — error in guard itself; Claude Code falls back to ask-permission

### PostToolUse Edit|Write → `~/.local/bin/rust-postedit-gate`

**Input (Claude Code stdin):**
```json
{
  "tool_name": "Edit" | "Write",
  "tool_input": {
    "file_path": "/path/to/edited/file",
    ...
  },
  "tool_response": { ... },
  "session_id": "<uuid>",
  "cwd": "<cwd>"
}
```

**MUST do:**
1. If `tool_input.file_path` does NOT end in `.rs`, exit 0 (no-op)
2. If `.rs`, identify owning module (parse `src/m<N>_<theme>/` from path)
3. Run `cargo check -p workflow-trace --bin <bin> 2>&1 | tail -20` scoped to module
4. Emit diagnostic JSON to `~/.local/share/workflow-trace/postedit-diagnostics.jsonl`
5. Update `cc-nvim-context` KV key `nvim.diag.last` with structured diagnostic per workspace CLAUDE.md § Nvim Context

**SHOULD do:**
- Time budget: ≤ 2s (Tier-1 fast — should NOT run full clippy)
- On time-budget exceeded: emit warning + abort cleanly (don't block session flow)
- Use `${PIPESTATUS[0]}` discipline (per workspace CLAUDE.local.md Shell & Git Hygiene)

**Exit codes:** always 0 (advisory only — does not block Edit/Write).

### Stop → `./scripts/gate.sh`

**Input (Claude Code stdin):**
```json
{
  "hook_event_name": "Stop",
  "session_id": "<uuid>",
  "transcript_path": "/path/to/transcript",
  "cwd": "<cwd>",
  "stop_hook_active": <bool>
}
```

**MUST do:**
1. Run full 4-stage QG (per project CLAUDE.md Quality Gate Protocol):
   ```text
   cargo check  --workspace --all-targets --all-features  | tail -20
   cargo clippy --workspace --all-targets --all-features  -- -D warnings | tail -20
   cargo clippy --workspace --all-targets --all-features  -- -D warnings -W clippy::pedantic | tail -20
   cargo test   --workspace --all-targets --all-features  --release | tail -30
   ```
2. PIPESTATUS discipline at every stage (per MEMORY.md `feedback_pipestatus_for_gate_chains.md`):
   ```bash
   cargo check ... | tee /tmp/wf-stop-check.log
   [[ ${PIPESTATUS[0]} -ne 0 ]] && { echo "STOP-HOOK FAIL: check"; exit 2; }
   ```
3. On all-green: exit 0 (session-stop proceeds)
4. On any-red: exit 2 with reason on stderr (session-stop refused — operator must address)

**SHOULD do:**
- Time budget: full QG can take 2-5 minutes; acceptable for Stop (per project CLAUDE.md Working Mode "report exact test counts in completion summaries")
- Honor `stop_hook_active=true` flag to prevent recursive invocation
- Skip when in HOLD-v2 (no src/, no Cargo.toml) — emit "QG skipped: HOLD-v2 active" + exit 0

---

## `.mcp.json` additions (post-G9, when MCP tools exist)

Per G5 § JSON Claude Code optimisation `.mcp.json` line. Pre-G9: empty (no MCP tools for workflow-trace). Post-G9, one server registered:

```json
{
  "mcpServers": {
    "workflow-trace-status": {
      "command": "~/.local/bin/wf-mcp-server",
      "args": ["--mode", "status"],
      "env": {
        "WORKFLOW_TRACE_DB": "~/.local/share/workflow-trace/db.sqlite"
      }
    }
  }
}
```

**Tools exposed by `wf-mcp-server --mode status`** (post-G9 author task):
- `mcp__workflow_trace_status__gates` — return G1-G9 state JSON
- `mcp__workflow_trace_status__wave` — return current Wave + worktrees
- `mcp__workflow_trace_status__lift` — return m14 lift mean + n + Wilson CI
- `mcp__workflow_trace_status__decay` — return m11 decay floor
- `mcp__workflow_trace_status__bridges` — return wt-bridge-check result
- `mcp__workflow_trace_status__cc5` — return last CC-5 closure timestamp + duration

**Mode discipline:** MCP server has `--mode status` only at M0 — read-only, no writes. M2+ optional `--mode dispatch` could expose dispatch primitives but only after Phase 7 security gate. Per workspace MEMORY.md `session-111.md` — blanket MCP rejected; this is a narrow, single-purpose server.

**Permissions integration:** add to `.claude/settings.json` `permissions.allow`:
```json
"mcp__workflow_trace_status__*"
```

---

## Plugin manifest

**M0 state: empty.** No plugin manifest at `.claude/plugin.json` until/unless workflow-trace ever ships as a Claude Code plugin. The proposed `wf-status` Zellij plugin (per [[codesynthor-v8-integration.md]]) is a **Zellij WASM plugin**, NOT a Claude Code plugin — different ecosystem, different manifest.

**Post-D60 hypothetical:** if workflow-trace's M0 succeeds and warrants plugin packaging, the manifest would live at `.claude-plugin/plugin.json` per `plugin-dev:plugin-structure` skill. Out of M0 scope.

---

## Workspace-scoped + survives G2 rename

Per G5 § JSON Claude Code optimisation "(workspace-scoped, pre-G2-rename; moves on rename)" line. The file lives at:

```text
the-workflow-engine/.claude/settings.json    # pre-G2 (current)
workflow-trace/.claude/settings.json         # post-G2 rename
```

**Rename safety:** `git mv the-workflow-engine workflow-trace` preserves `.claude/` subtree. No content changes needed — all paths in settings.json are relative (`./scripts/*`, `./target`) or absolute-but-stable (`~/.local/bin/*`).

**Pre-G2 verification:** the file must NOT contain hardcoded path `the-workflow-engine` anywhere — check with `rg -n 'the-workflow-engine' .claude/settings.json` → should return zero results.

---

## Failure modes (≥3)

| ID | Failure | Detection | Mitigation |
|---|---|---|---|
| **JSON-01** | Allow list too permissive — agent runs unrelated `cargo run` on wrong project | atuin trajectory shows cargo invocation outside `cwd=workflow-trace` | settings.json IS workspace-scoped; allow `Bash(cargo *)` matches only when cwd is the project; verify via `--cwd` filter |
| **JSON-02** | Hook binary missing — `~/.local/bin/cc-guard-bash` not installed | PreToolUse fails open (Claude Code falls back to ask-permission) | install per `hookify:writing-rules` skill; CI check `[[ -x ~/.local/bin/cc-guard-bash ]]` at session-start; Watcher Class-H flag if missing |
| **JSON-03** | Stop-hook gate.sh false positive (PIPESTATUS bug → red on green clippy) | session-stop blocked unexpectedly; operator overrides | per MEMORY.md `feedback_pipestatus_for_gate_chains.md` — exact PIPESTATUS handling; unit-test gate.sh independently; bypass once via `--skip-gate` flag with audit log |
| **JSON-04** | Deny pattern bypass via shell expansion (`r''m -rf` or `\\rm -rf`) | atuin shows command that should have been denied | hook matching uses simple substring (not regex) by default; supplement with `cc-guard-bash` runtime parse via real shell tokenizer; periodic per-Wave-end audit of cc-guard-bash blocks.jsonl for missed patterns |
| **JSON-05** | env var override not respected (CARGO_TARGET_DIR ignored by child cargo) | per-worktree builds collide on shared target/ | Claude Code env injects at process exec time; if cargo runs via shell-out from non-cargo binary, env may not propagate; explicit `CARGO_TARGET_DIR=./target cargo ...` in scripts (already workspace CLAUDE.md Quality Gate convention) |
| **JSON-06** | MCP server crashes mid-session (post-G9) | tool calls to `mcp__workflow_trace_status__*` time out | restart via MCP infrastructure; status server is stateless (reads from SQLite each call); workflow-trace operation unaffected (server is observational) |
| **JSON-07** | G2 rename breaks settings (someone hardcoded `the-workflow-engine` in allow path) | post-rename Bash commands denied | pre-G2 verification grep (above); post-rename smoke: `atuin scripts run wt-gate-status` from new cwd |

---

## Atuin trajectory

```bash
# Hook firing audit
ls -la ~/.local/share/cc-guard-bash/blocks.jsonl
wc -l    ~/.local/share/cc-guard-bash/blocks.jsonl                  # block count over time
tail -20 ~/.local/share/cc-guard-bash/blocks.jsonl | jq             # recent blocks

# Postedit diagnostics
ls -la ~/.local/share/workflow-trace/postedit-diagnostics.jsonl
tail -20 ~/.local/share/workflow-trace/postedit-diagnostics.jsonl | jq

# Stop-hook history
atuin search "scripts/gate.sh" --before 7d                          # Stop-hook invocations

# Per-Wave-end audit
atuin search "cc-guard-bash" --before 7d | wc -l
atuin search "rust-postedit-gate" --before 7d | wc -l

# Settings.json change audit
git log --oneline -- .claude/settings.json
```

---

## Verification commands

```bash
# Validate settings.json shape
jq . .claude/settings.json > /dev/null && echo "valid JSON"

# Validate against schema (if available)
# jsonschema-cli .claude/settings.json --schema https://claude.com/schemas/settings.json

# Hook binary presence
for b in cc-guard-bash rust-postedit-gate; do
  [[ -x ~/.local/bin/$b ]] && echo "$b ✅" || echo "$b ❌ MISSING"
done
[[ -x ./scripts/gate.sh ]] && echo "gate.sh ✅" || echo "gate.sh ❌ MISSING (Phase 1)"

# Deny pattern dry-run (cc-guard-bash should block)
echo '{"tool_name":"Bash","tool_input":{"command":"rm -rf /"},"cwd":"."}' \
  | ~/.local/bin/cc-guard-bash; echo "exit=$?"            # expect exit 2

# Env var smoke
env | grep -E "CARGO_TARGET_DIR|RUST_BACKTRACE|RUSTFLAGS|WORKFLOW_TRACE_"

# G2 rename safety (must return zero matches)
rg -n 'the-workflow-engine' .claude/settings.json

# MCP post-G9 smoke
[[ -f .mcp.json ]] && jq .mcpServers.workflow-trace-status .mcp.json
```

---

## Sign-off

✅ json-claude-code-optimisation spec complete. `.claude/settings.json` full spec (permissions allow/deny lists, hooks PreToolUse/PostToolUse/Stop, env vars CARGO_TARGET_DIR/RUST_BACKTRACE/RUSTFLAGS/WORKFLOW_TRACE_*) defined with per-section rationale. `.mcp.json` post-G9 addition specified (single workflow-trace-status server, status-mode only at M0). Hook payload contracts for each event (PreToolUse stdin shape + MUST/SHOULD + exit codes; PostToolUse advisory ≤2s budget; Stop full 4-stage QG with PIPESTATUS discipline). Plugin manifest documented as M0-empty. G2 rename safety verified (relative paths only). 7 failure modes with mitigations. Atuin trajectory + verification commands deterministic.

*Authored 2026-05-17 (S1001982) — Command for V7 G5 expansion. Settings.json activated Phase 1 Genesis Day 0; .mcp.json post-G9. HOLD-v2 respected: planning-only.*
