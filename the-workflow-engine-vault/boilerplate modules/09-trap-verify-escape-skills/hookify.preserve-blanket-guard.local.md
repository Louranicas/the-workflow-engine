---
name: preserve-blanket-guard
enabled: true
event: bash
pattern: ^(?!GUARD_OVERRIDE=1\s).*(docker\s+(container\s+|system\s+|image\s+|volume\s+|network\s+)?prune|rm\s+-rf\s+.*[\*\$]|pkill\s+-f\b|cargo\s+clean\s+--all-targets|git\s+clean\s+-fd|trash-empty|docker\s+rm\s+\$\()
action: block
---

⚠️ **BLANKET-SCOPE COMMAND DETECTED — preserve-list check required**

Before running this command:

1. **Enumerate** what it will actually affect:
   - `docker ps -a --format '{{.ID}} {{.Names}}'` before any `docker * prune`
   - `ls <glob>` before `rm -rf` with expansion
   - `pgrep -af <pattern>` before `pkill -f`
   - `docker volume ls` before volume prune
2. **Diff** that enumeration against every "do not delete X / preserve Y / keep Z / we may yet resurrect Z" instruction in this conversation. Scan the last N user messages explicitly.
3. **If any overlap**, replace the blanket form with an explicit named list (e.g., `docker rm <id1> <id2> ...`) that includes only the intended targets.

**Why this guard exists:** S102 (2026-04-18). `docker container prune -f` removed the openclaw-gateway stopped container despite an explicit "we may yet resurrect Cipher" instruction from Luke. The Step-4 named-exclusion did not propagate through the Step-7 blanket prune. Image + volumes survived so Cipher is recoverable, but the instruction was not honored.

The pattern generalises: safety rules enforced per-call-site (named exclusions, per-constructor strips, per-service health paths) dissolve under "all" / `--all` / `prune` / `-rf *` operations. See also BUG-033 (http:// strip per-bridge-constructor bypassed by direct helper calls) and S101 path-drift.

**To bypass** (ONLY after the three checks above): prefix the command with `GUARD_OVERRIDE=1 `. Example: `GUARD_OVERRIDE=1 docker container prune -f`.

Reference: `~/.claude/projects/-home-louranicas-claude-code-workspace/memory/feedback_preserve_list_discipline.md`


---

> Vault navigation: [[../../BOILERPLATE_INDEX|BOILERPLATE_INDEX]] · [[../../README|boilerplate modules README]] · [[../../../HOME|HOME]] · [[../../../MASTER_INDEX|MASTER_INDEX]]
> Reference-only clone — see [[../../BOILERPLATE_INDEX]] for upstream source + target-module mapping.
