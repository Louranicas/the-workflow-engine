#!/usr/bin/env bash
# PreToolUse hook (Write|Edit matcher) — refuse writes to Cargo.toml while G9 has not fired.
# Per CLAUDE.md PRIME DIRECTIVE: no Cargo.toml at root until `start coding workflow-trace`.
# Per AP-V7-12: Cargo.toml-via-cargo-init inside workspace is a structural antipattern.
# Per PRIME_DIRECTIVE_WAIVER.md: metadata captured in plan.toml + ai_docs/CARGO_LAYOUT_SPEC.md
# instead of Cargo.toml until G9.
#
# Exit 0 = allow; Exit 2 = block.

ROOT="/home/louranicas/claude-code-workspace/the-workflow-engine"
STATUS_FILE="$ROOT/.claude/status.json"

payload=$(cat)
file_path=$(echo "$payload" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d.get('tool_input',{}).get('file_path',''))" 2>/dev/null)

# Only enforce on Cargo.toml at root or in subdirs (NOT inside vault/ai_docs as those are markdown only)
base=$(basename "$file_path")
case "$base" in
  Cargo.toml|Cargo.lock) ;;
  *) exit 0 ;;
esac

# Check g9_fired
g9_fired=$(python3 -c "import json; print(json.load(open('$STATUS_FILE')).get('g9_fired', False))" 2>/dev/null)
if [[ "$g9_fired" = "True" ]]; then
  exit 0
fi

# Block
cat >&2 <<EOF
REFUSED: pre-write-no-cargo-pre-g9.sh
  Tool: Write/Edit
  Path: $file_path
  Reason: G9 has not fired. Per CLAUDE.md PRIME DIRECTIVE + PRIME_DIRECTIVE_WAIVER.md,
          no Cargo.toml until Luke types: start coding workflow-trace
  Rationale: Cargo.toml at root would let any subagent or hook trigger `cargo build`
             and silently begin implementation. Metadata lives in plan.toml + ai_docs/
             CARGO_LAYOUT_SPEC.md pre-G9.
  Recovery:
    - Edit plan.toml [dependencies], [features], [[bin_targets]] sections instead
    - Document Cargo layout intent in ai_docs/CARGO_LAYOUT_SPEC.md
  AP-ID: AP24 + AP-V7-12
EOF
exit 2
