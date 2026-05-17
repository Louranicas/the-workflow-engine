---
name: Preserve-List Discipline for Blanket Commands
description: When the user has said "do not delete X" or "preserve Y", named exclusions upstream do not propagate through blanket commands (prune/--all/rm -rf/pkill -f). Every blanket-scope command must rebuild its own filter and be diff'd against the preserve list before execution.
type: feedback
originSessionId: 1cb1aa74-93ae-4828-be36-97b58506dd90
---
**Rule:** When a preserve list exists (any "do not delete", "we may yet resurrect", "keep X"), every blanket-scope command must (1) enumerate its targets explicitly, (2) diff the enumeration against the preserve list, (3) if there's any overlap, rewrite into a named form. Named-exclusion on an earlier named command does not propagate through.

**Why:** S102 (2026-04-18). Luke said "openclaw-gateway we may yet resurect Cipher". I ran the retirement doc's Step 4 (named `docker rm`) with openclaw excluded correctly. Then I ran Step 7 (`docker container prune -f`) — a blanket command whose scope is "all stopped containers". It removed openclaw's stopped-container snapshot despite the earlier exclusion. Image + volumes survived so Cipher is recoverable, but the instruction was not honored and the recovery is luck, not design. Same structural pattern as BUG-033 (http:// rule enforced per-bridge-constructor, bypassed by direct raw_http_* calls) and S101 path-drift (deploy-path rule enforced for `~/.local/bin`, bypassed by `./bin/` services). Safety enforced per-call-site dissolves under "all" operations.

**How to apply:**
- Blanket-command signatures to flag: `prune`, `prune -f`, `--all`, `-a` on destructive ops, `rm -rf <expanding-glob>`, `pkill -f`, `cargo clean --all-targets`, `git clean -fd`, `docker system prune`, `trash-empty`.
- Before executing any blanket command, run the enumeration first (`docker ps -a --format ...`, `ls <path-glob>`, `pgrep -af <pattern>`). Show the list. Then mentally or literally diff against every preserve entry in the conversation.
- If overlap exists, refuse the blanket form. Replace with an explicit named list (e.g., `docker rm <id1> <id2> ...`) that includes only the intended targets.
- If a doc generated earlier in the session contains both named-exclusions AND blanket steps, treat the doc as self-inconsistent. Either edit the doc to remove the blanket step, or treat the exclusion as a hard stop for every subsequent step.
- "Proceed seamlessly" instruction means skip permission-asking on approved work. It does NOT mean skip pre-execution constraint checks. Blanket scope always warrants the check regardless of pacing instruction.


---

> Vault navigation: [[../../BOILERPLATE_INDEX|BOILERPLATE_INDEX]] · [[../../README|boilerplate modules README]] · [[../../../HOME|HOME]] · [[../../../MASTER_INDEX|MASTER_INDEX]]
> Reference-only clone — see [[../../BOILERPLATE_INDEX]] for upstream source + target-module mapping.
