---
description: Open the per-module spec by module ID
argument-hint: m<N> (e.g. m7, m23, m42)
---

# /module-spec — open per-module ai_specs by m<N>

Open the per-module spec for the given workflow-trace module ID. Resolves the cluster from `.claude/context.json` and prints the spec path + first 50 lines.

```bash
#!/usr/bin/env bash
ROOT="/home/louranicas/claude-code-workspace/the-workflow-engine"
M="$1"   # e.g. m7, m23, m42

[[ -z "$M" ]] && { echo "Usage: /module-spec m<N>"; exit 1; }

# Look up cluster from context.json
cluster=$(python3 -c "
import json
d = json.load(open('$ROOT/.claude/context.json'))
m = '$M'
for c, info in d['clusters'].items():
    if m in info['modules']:
        print(c)
        break
")

if [[ -z "$cluster" ]]; then
  echo "module $M not found in any cluster — check .claude/context.json"
  exit 2
fi

spec="$ROOT/ai_specs/modules/cluster-$cluster/$M.md"
echo "=== Module: $M  Cluster: $cluster ==="
echo "Spec path: $spec"
echo ""

if [[ ! -f "$spec" ]]; then
  echo "(spec not yet authored — dispatch cluster-spec-author agent for cluster $cluster)"
  exit 0
fi

# Print frontmatter + first 50 lines of body
echo "--- frontmatter + body (first 50 lines) ---"
head -50 "$spec"
echo ""
echo "(full spec at $spec — use Read tool for the rest)"
```
