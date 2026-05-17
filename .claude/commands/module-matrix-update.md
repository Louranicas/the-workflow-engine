---
description: Refresh ai_specs/MODULE_MATRIX.md from per-module frontmatter (read-only audit pre-G9; write post-G9)
argument-hint: (no args)
---

# /module-matrix-update — refresh MODULE_MATRIX.md from per-module frontmatter

Scan every `ai_specs/modules/cluster-*/m<N>.md` per-module spec frontmatter and emit a fresh `MODULE_MATRIX.md` table. Pre-G9: print to stdout for audit only (no write). Post-G9: write to file.

```bash
#!/usr/bin/env bash
ROOT="/home/louranicas/claude-code-workspace/the-workflow-engine"
MATRIX="$ROOT/ai_specs/MODULE_MATRIX.md"
OUT="/tmp/wf-module-matrix-$(date +%s).md"

echo "=== Module Matrix Refresh — workflow-trace ==="
echo ""

# Collect frontmatter from every per-module spec
python3 <<'PYEOF'
import os, re, glob, json
root = "/home/louranicas/claude-code-workspace/the-workflow-engine"
spec_glob = f"{root}/ai_specs/modules/cluster-*/m*.md"
rows = []
for path in sorted(glob.glob(spec_glob)):
    with open(path) as f:
        body = f.read()
    m = re.match(r'^---\n(.*?)\n---', body, re.S)
    if not m:
        continue
    fm = {}
    for line in m.group(1).splitlines():
        if ':' in line:
            k, v = line.split(':', 1)
            fm[k.strip()] = v.strip()
    rows.append(fm)

if not rows:
    print("(no per-module specs found yet under ai_specs/modules/cluster-*/m*.md)")
    print("(dispatch cluster-spec-author agent per cluster to populate)")
else:
    # Sort by module_id natural order (m1, m2, ..., m42)
    rows.sort(key=lambda r: int(r.get('module_id', 'm0').lstrip('m') or 0))
    print("| module | cluster | binary | LOC | tests | features | deps |")
    print("|---|---|---|---|---|---|---|")
    for r in rows:
        print(f"| {r.get('module_id','-')} | {r.get('cluster','-')} | {r.get('binary_owner','-')} | {r.get('loc_target','-')} | {r.get('test_target','-')} | {r.get('features','-')} | {r.get('deps','-')} |")
    print()
    print(f"Total per-module specs found: {len(rows)} (expected 26)")
PYEOF
echo ""

# Diff against current MODULE_MATRIX.md
if [[ -f "$MATRIX" ]]; then
  echo "--- diff against current MODULE_MATRIX.md ---"
  echo "(audit only — write deferred to post-G9 explicit invocation)"
  echo ""
  echo "Current $MATRIX exists ($(wc -l < "$MATRIX") lines). Use the table above to update by hand pre-G9."
else
  echo "MODULE_MATRIX.md missing at $MATRIX — author from the table above."
fi
```
