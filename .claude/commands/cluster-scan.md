---
description: Show files in a workflow-trace cluster (specs + vault + post-G9 source)
argument-hint: <cluster letter A-H>
---

# /cluster-scan — enumerate files in a cluster

Show all files belonging to a workflow-trace cluster: per-module specs, vault cluster note, and (post-G9) source modules.

```bash
#!/usr/bin/env bash
ROOT="/home/louranicas/claude-code-workspace/the-workflow-engine"
C="$1"   # cluster letter A-H

[[ -z "$C" ]] && { echo "Usage: /cluster-scan <A|B|C|D|E|F|G|H>"; exit 1; }
[[ ! "$C" =~ ^[A-Ha-h]$ ]] && { echo "cluster must be A-H"; exit 1; }
C=$(echo "$C" | tr '[:lower:]' '[:upper:]')

echo "=== Cluster $C — workflow-trace ==="

# Resolve module list + role from context.json
python3 -c "
import json
d = json.load(open('$ROOT/.claude/context.json'))
info = d['clusters'].get('$C')
if not info:
    print('cluster $C not found in context.json'); exit(2)
print(f'  Name: {info[\"name\"]}')
print(f'  Modules: {\", \".join(info[\"modules\"])}')
print(f'  LOC budget: {info[\"loc\"]}')
print(f'  Role: {info[\"role\"]}')
"
echo ""

# Per-module specs
echo "--- ai_specs/modules/cluster-$C/ ---"
spec_dir="$ROOT/ai_specs/modules/cluster-$C"
if [[ -d "$spec_dir" ]]; then
  ls -la "$spec_dir" 2>/dev/null | tail -n +2 | sed 's/^/  /'
else
  echo "  (directory not yet created)"
fi
echo ""

# Vault cluster note
echo "--- vault cluster-$C ---"
find "$ROOT/the-workflow-engine-vault/module specs/" -iname "cluster-${C}*.md" 2>/dev/null \
  | head -5 \
  | sed 's|.*/||; s/^/  /'
echo ""

# Post-G9 source
echo "--- src/ modules (post-G9) ---"
if [[ -d "$ROOT/src" ]]; then
  python3 -c "
import json
d = json.load(open('$ROOT/.claude/context.json'))
for m in d['clusters']['$C']['modules']:
    print(f'  expected: src/{m}_*/')
"
  find "$ROOT/src" -type d -name "m*_*" 2>/dev/null | sed 's|.*/||; s/^/  found: /'
else
  echo "  (src/ does not exist — pre-G9)"
fi
```
