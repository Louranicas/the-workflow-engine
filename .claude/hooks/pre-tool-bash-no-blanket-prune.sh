#!/usr/bin/env bash
# PreToolUse hook (Bash matcher) — refuse blanket docker prune commands.
# Per S102 incident: `docker container prune -f` lost openclaw-gateway despite an earlier
# named-exclusion rule. Blanket commands rebuild their own filter; named-rm exclusion
# upstream does NOT propagate. Enumerate before any blanket.
# Per feedback memory `feedback_preserve_list_discipline.md` (weight 0.75 ACTIVE).
#
# Patterns blocked:
#   - docker container prune <any flags>
#   - docker volume prune <any flags>
#   - docker network prune <any flags>
#   - docker system prune <any flags>
#   - docker image prune --all|-a <any flags>
#   - docker prune (alias / generic)
#
# Exit 0 = allow; Exit 2 = block.

payload=$(cat)
cmd=$(echo "$payload" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d.get('tool_input',{}).get('command',''))" 2>/dev/null)

# Substring match for any docker prune variant
if echo "$cmd" | /usr/bin/grep -qE 'docker[[:space:]]+(container|volume|network|system)[[:space:]]+prune'; then
  blocked=1
elif echo "$cmd" | /usr/bin/grep -qE 'docker[[:space:]]+image[[:space:]]+prune.*(-a|--all)'; then
  blocked=1
elif echo "$cmd" | /usr/bin/grep -qE 'docker[[:space:]]+prune'; then
  blocked=1
else
  exit 0
fi

cat >&2 <<EOF
REFUSED: pre-tool-bash-no-blanket-prune.sh
  Tool: Bash
  Command: $(echo "$cmd" | head -c 200)
  Reason: blanket docker prune destroys preserved-by-name containers
  Incident: S102 lost openclaw-gateway to `docker container prune -f` despite an earlier
            named-rm exclusion. Named exclusions do NOT propagate to prune commands.
  AP-ID: AP-Hab-04 (preserve-list discipline; weight 0.75 ACTIVE)
  Recovery:
    - Enumerate first: docker ps -a --filter <criteria> --format '{{.Names}}'
    - Then targeted rm: docker rm <name1> <name2> ...
    - Explicit per-resource confirmation from Luke before any prune
  Reference: ~/.claude/projects/-home-louranicas-claude-code-workspace/memory/feedback_preserve_list_discipline.md
EOF
exit 2
