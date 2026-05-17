---
description: Print live G1-G9 + B1-B6 gate / blocker state for workflow-trace
argument-hint: (no args)
---

# /gate-status — workflow-trace gate + blocker register

Print the live G1-G9 + B1-B6 table for the workflow-trace project. Sources from `GATE_STATE.md`, `CLAUDE.local.md`, and `.claude/context.json`.

```bash
#!/usr/bin/env bash
# Read three sources and produce a unified table.
ROOT="/home/louranicas/claude-code-workspace/the-workflow-engine"

echo "=== workflow-trace Gate + Blocker State ==="
echo "Source: $ROOT/GATE_STATE.md + CLAUDE.local.md + .claude/context.json"
echo ""

# Gates G1-G9 from GATE_STATE.md
echo "--- Gates (G1-G9) ---"
rg -n '^\| \*\*G[1-9]\*\*' "$ROOT/GATE_STATE.md" 2>/dev/null \
  | sed 's/|/ | /g'
echo ""

# Blockers B1-B6 from GATE_STATE.md
echo "--- Blockers (B1-B6) ---"
rg -n '^\| \*\*B[1-6]\*\*' "$ROOT/GATE_STATE.md" 2>/dev/null \
  | sed 's/|/ | /g'
echo ""

# Luke physical actions remaining
echo "--- Luke physical actions remaining ---"
rg -A 4 '## Luke physical actions remaining' "$ROOT/GATE_STATE.md" 2>/dev/null \
  | head -7
echo ""

# Cross-check with context.json
echo "--- context.json mirror ---"
python3 -c "
import json
d = json.load(open('$ROOT/.claude/context.json'))
print('  HOLD-v2 active:', d['hold_v2_active'])
print('  G9 signal needed:', d['g9_signal'])
print('  Module count:', d['module_count'])
print('  Luke actions remaining:', d['luke_actions_remaining'])
for k, v in d['gates'].items():
    note = v.get('note', '')
    print(f'  {k}: {v[\"state\"]:20s} owner={v[\"owner\"] or \"-\":10s} {note}')
"
echo ""

# Any newer cross-talk drops since GATE_STATE.md edit
echo "--- Newer cross-talk drops (since GATE_STATE.md last edit) ---"
gs_ts=$(stat -c %Y "$ROOT/GATE_STATE.md" 2>/dev/null)
find ~/projects/shared-context/agent-cross-talk/ -name "*.md" -newer "$ROOT/GATE_STATE.md" 2>/dev/null \
  | head -5 \
  | sed 's|.*/||; s/^/  /'
```
