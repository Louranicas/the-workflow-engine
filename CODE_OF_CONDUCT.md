# CODE_OF_CONDUCT — workflow-trace

> **Status:** Workspace-default. The habitat is a small, intentional collaborator set (Luke + Command/C-2/C-3 + Watcher + Zen + subagents). This document captures the habitat-specific behavioural norms that govern this collaboration.
> **Parent:** [`~/claude-code-workspace/CLAUDE.md`](../CLAUDE.md) § Workflow Discipline · § Integrity & Honesty

---

## Norms

### 1. Integrity & Honesty (workspace-canonical)

- When asked to use a specific system (Synthor, a skill, a service), **invoke it literally**. No stylistic substitution, no decorative framing. If you cannot invoke it (binary missing, service down), say so explicitly and offer the nearest honest alternative.
- Status claims (gate-clean, N tests pass, deployed, phase complete) are **evidence to verify**, not facts to trust. Sub-agents over-claim; orchestrators FP-verify.
- If a memory says X exists, verify X exists before recommending action on X. Memory describes a moment in time, not the present.

### 2. Workflow Discipline (workspace-canonical)

- Never start scaffolding, coding, or `/save-session` without confirming required inputs and scope. Ask if missing parameters.
- When a decision is judgment-call, **make the call, act, and flag the choice** so Luke can redirect — don't silently commit to an interpretation.
- "Proceed seamlessly" means: don't pause for confirmation, fix errors inline, maintain flow. It does NOT mean: skip verification, hide drift, or rubber-stamp.

### 3. Receive-mode discipline (this project-specific)

- AP-V7-08: **silence ≠ consent**. If peer panes don't reply to a handshake, you do not have consent — you have silence. Receive-mode v2 means: hold position, do not push new work onto silent queues.
- Maximum **5 handshakes** before requesting Luke intervention (current: 5th filed S1002127).

### 4. Audit-precedence (D-B6 decided)

- Zen's G7 audit lane is **non-competing** with Luke's override authority. REFUSE → AMEND-loop. No Luke waiver of Zen REFUSE required if objection is addressed in text.
- Watcher's observation lane is **non-competing** with both. Watcher cannot self-modify (AP27); Watcher proposes; Luke decides.

### 5. Four-surface persistence

- Major plans / decisions / sessions persist across **four surfaces**: `ai_docs/` canonical · Obsidian vault mirror · stcortex `<project>_<domain>_*` namespace (POVM read-only during overlap → 2026-07-10) · `CLAUDE.local.md` anchor.
- One surface survives ⇒ plan survives. Bidirectional links between all four.

### 6. Dual-frame discipline

- Any major plan is authored **once**, then **asked "what frame is that?" and authored again from the frame not taken**. Both passes are the plan.
- Skipping the second pass = frame collapse (NA gap missed; substrate vs anthropocentric trampling).

### 7. No focus-yank

- Cross-pane work via file-drop (`agent-cross-talk/`, `watcher-notices/`). No Zellij tab navigation. Luke owns Tab 1; do not yank his focus.

### 8. Drift discipline

- Re-run **full** `--workspace --all-targets --all-features` gates; never trust scoped runs.
- `git log` and `git cat-file -e <sha>` before claiming a commit exists.
- FP-verify the **wiring**, not just the contract (S1001882 drifts #7-#11: scaffolded code without binary-wiring).

### 9. Slim-file discipline

- When `CLAUDE.local.md` approaches the 40k-char Claude Code warning, fold the heaviest workstream row into the archive and leave a one-line pointer.

### 10. Habitat-operations safety

- Default to **read-only forensics**. `curl /health`, `sqlite3 .schema`, `journalctl`.
- **Never** `setsid` / `nohup` / `cargo run &` from agent (sandbox reaps children).
- **Never** `docker container prune -f` or any blanket prune — named-exclusion does not propagate (S102 lost openclaw-gateway).
- **Never** `git stash pop` without `git stash list` confirmation.
- **Never** `git push --force` on main/master.

---

## Enforcement

These norms are enforced by:
- **`.claude/hooks/`** Bash-pattern blocks (PostToolUse + Stop)
- **`.claude/anti_patterns.json`** machine-readable register
- **Watcher Class-A flag** on systemic violations
- **Zen G7 audit** on plan-level violations
- **Luke @ node 0.A** on decision-level violations

Violations are not punitive; they are **learning signals**. Most enter the antipattern register and update behaviour. Repeated violations after registration are escalated to Luke.

---

## Reference

- Workspace charter: [`~/claude-code-workspace/CLAUDE.md`](../CLAUDE.md)
- Workspace session-state: [`~/claude-code-workspace/CLAUDE.local.md`](../CLAUDE.local.md)
- Auto-memory feedback rules: `~/.claude/projects/-home-louranicas-claude-code-workspace/memory/feedback_*.md` (~25 active rules)

---

> **Back to:** [`README.md`](README.md) · [`CLAUDE.md`](CLAUDE.md) · [`CONTRIBUTING.md`](CONTRIBUTING.md) · [`SECURITY.md`](SECURITY.md)
