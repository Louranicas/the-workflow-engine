#!/usr/bin/env bash
# PreToolUse hook (Write|Edit matcher) — refuse writes to *.rs files while G9 has not fired.
# Reads .claude/status.json `g9_fired` flag; if false, blocks any .rs write with exit 2.
# Per CLAUDE.md PRIME DIRECTIVE: no Rust source until Luke types `start coding workflow-trace`.
# Per PRIME_DIRECTIVE_WAIVER.md: S1002127 waiver scope is markdown + .claude config, NOT .rs.
#
# Hook input on stdin: standard Claude Code JSON payload with tool_input.file_path.
# Exit 0 = allow; Exit 2 = block (Claude Code refuses tool invocation; user sees stderr message).
#
# Note: no `set -e` — jq / python may exit non-zero on missing keys legitimately; we handle inline.

ROOT="/home/louranicas/claude-code-workspace/the-workflow-engine"
STATUS_FILE="$ROOT/.claude/status.json"

payload=$(cat)
file_path=$(echo "$payload" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d.get('tool_input',{}).get('file_path',''))" 2>/dev/null)

# Only enforce on .rs files
case "$file_path" in
  *.rs) ;;
  *) exit 0 ;;
esac

# Check g9_fired
g9_fired=$(python3 -c "import json; print(json.load(open('$STATUS_FILE')).get('g9_fired', False))" 2>/dev/null)
if [[ "$g9_fired" = "True" ]]; then
  exit 0
fi

# Block
cat >&2 <<EOF
REFUSED: pre-write-no-rust-pre-g9.sh
  Tool: Write/Edit
  Path: $file_path
  Reason: G9 has not fired. Per CLAUDE.md PRIME DIRECTIVE + PRIME_DIRECTIVE_WAIVER.md,
          no .rs source files until Luke types: start coding workflow-trace
  Recovery:
    - If this is a spec doc: use .md extension under ai_specs/ or vault/
    - If you genuinely need .rs: ask Luke to fire G9 or file a per-gate waiver
  AP-ID: AP24 (workspace) + AP-V7-12 (workflow-trace) + AP-V7-11 (verb-lock)
EOF
exit 2
