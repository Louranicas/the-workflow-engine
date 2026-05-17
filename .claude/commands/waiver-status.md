---
description: Print active waiver records governing workflow-trace HOLD-v2 envelope
argument-hint: (no args)
---

# /waiver-status — active waiver records register

Print active waivers governing the workflow-trace HOLD-v2 envelope. Sources from `PRIME_DIRECTIVE_WAIVER.md` and any per-gate waiver drops in `~/projects/shared-context/agent-cross-talk/`.

```bash
#!/usr/bin/env bash
ROOT="/home/louranicas/claude-code-workspace/the-workflow-engine"

echo "=== workflow-trace Active Waivers ==="
echo ""

# Canonical waiver
if [[ -f "$ROOT/PRIME_DIRECTIVE_WAIVER.md" ]]; then
  echo "--- S1002127 PRIME_DIRECTIVE_WAIVER (canonical) ---"
  head -45 "$ROOT/PRIME_DIRECTIVE_WAIVER.md"
  echo ""
  echo "(full text at $ROOT/PRIME_DIRECTIVE_WAIVER.md)"
else
  echo "PRIME_DIRECTIVE_WAIVER.md MISSING — HOLD-v2 envelope is at full strictness"
fi
echo ""

# Per-gate waiver drops
echo "--- Per-gate waiver drops (cross-talk) ---"
find ~/projects/shared-context/agent-cross-talk/ -name "*waiver*" -type f 2>/dev/null \
  | head -10 \
  | sed 's|.*/||; s/^/  /'
echo ""

# What's still in force
echo "--- HOLD-v2 envelope status ---"
python3 -c "
import json
d = json.load(open('$ROOT/.claude/context.json'))
print('  hold_v2_active:', d['hold_v2_active'])
print('  g9_signal_required:', d['g9_signal'])
"
echo ""

# What this waiver explicitly denies
echo "--- Explicitly NOT waived (refuse-mode for these actions) ---"
echo "  - .rs source files under src/"
echo "  - Cargo.toml at root"
echo "  - cargo init / cargo new / cargo build"
echo "  - directory rename the-workflow-engine -> workflow-trace (G2)"
echo "  - stcortex writes under workflow_trace_* namespace (G8)"
```
