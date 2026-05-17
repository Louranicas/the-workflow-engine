#!/usr/bin/env bash
# PreToolUse hook (Bash matcher) — refuse stcortex writes whose namespace is not prefixed
# `workflow_trace_*`. Enforces AP30 (namespace string drift) at the substrate-write boundary.
#
# Patterns blocked (substring grep on tool_input.command):
#   - `stcortex call write_memory <ns>` where ns starts with anything other than workflow_trace_
#   - `stcortex call write_pathway <pre> <post> <ns>` (same)
#   - `stcortex call register_consumer <name> <ns>` (same)
#   - curl/HTTP POST to :3000 with payload containing namespace key not workflow_trace_*
#
# Exit 0 = allow; Exit 2 = block.

payload=$(cat)
cmd=$(echo "$payload" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d.get('tool_input',{}).get('command',''))" 2>/dev/null)

# Quick filter: only inspect commands that mention stcortex or the SpacetimeDB port :3000
case "$cmd" in
  *stcortex*|*:3000*) ;;
  *) exit 0 ;;
esac

# Extract any positional namespace argument from `stcortex call write_memory <ns> ...`
# or `stcortex call write_pathway <pre> <post> <ns> ...`
# or `stcortex call register_consumer <name> <ns> ...`
# We use a permissive regex tokeniser.
ns=$(echo "$cmd" | python3 -c '
import sys, re
cmd = sys.stdin.read()
patterns = [
    r"stcortex\s+call\s+write_memory\s+([A-Za-z_][A-Za-z0-9_-]*)",
    r"stcortex\s+call\s+write_pathway\s+\S+\s+\S+\s+([A-Za-z_][A-Za-z0-9_-]*)",
    r"stcortex\s+call\s+register_consumer\s+\S+\s+([A-Za-z_][A-Za-z0-9_-]*)",
]
for p in patterns:
    m = re.search(p, cmd)
    if m:
        print(m.group(1))
        break
' 2>/dev/null)

# If we could not extract a namespace, allow (lets other tools run)
if [[ -z "$ns" ]]; then
  exit 0
fi

# Allow workflow_trace_* prefix
case "$ns" in
  workflow_trace_*) exit 0 ;;
esac

# Also tolerate well-known habitat-shared namespaces (read-side or comms-only)
case "$ns" in
  habitat_sessions|habitat_*) exit 0 ;;
esac

# Block
cat >&2 <<EOF
REFUSED: pre-tool-bash-namespace-guard.sh
  Tool: Bash
  Command excerpt: $(echo "$cmd" | head -c 200)
  Reason: stcortex namespace '$ns' is not prefixed workflow_trace_*
  AP-ID: AP30 (namespace string drift)
  Rationale: All workflow-trace stcortex writes MUST use a namespace string constant from
             workflow_core::namespace. Literal strings in module code drift; m9 namespace
             guard enforces at runtime, this hook enforces pre-tool.
  Recovery:
    - Prefix the namespace: workflow_trace_<domain> (e.g. workflow_trace_pathways)
    - If this is a habitat-shared comms write, use one of: habitat_sessions, habitat_*
    - If you genuinely need a new prefix, add it to this hook's allow list with rationale
EOF
exit 2
