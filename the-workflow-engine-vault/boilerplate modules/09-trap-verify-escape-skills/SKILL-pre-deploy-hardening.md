---
name: pre-deploy-hardening
description: |
  Four-agent parallel pre-deploy gate: security-auditor + performance-engineer +
  silent-failure-hunter + zen, all run on the staged diff before /forge or
  /deploy-orac fires. Each catches a class the others structurally miss. APPROVE
  from all four required before binary cp to ~/.local/bin/.
  Triggers on: "pre-deploy hardening", "before deploy", "pre-ship gate", "harden
  before forge", "ship gate", or before any binary cp to live habitat.
allowed-tools:
  - Task
  - Bash
  - Read
  - Write
---

# /pre-deploy-hardening — 4-Lens Pre-Deploy Gate

Run between `cargo build --release` and `/usr/bin/cp -f`. Catches the regressions
post-deploy soak finds 30 minutes too late.

## Pre-flight

```bash
SHIP_ID="ship-$(date +%Y%m%d-%H%M)"
WORK="/tmp/${SHIP_ID}"
mkdir -p "$WORK"

SERVICE="${1:?USAGE: /pre-deploy-hardening <service-dir-name>}"
SVC_PATH="$HOME/claude-code-workspace/$SERVICE"
[ ! -d "$SVC_PATH" ] && { echo "ERR: $SVC_PATH missing"; exit 1; }

# Stage the diff against last deploy (use the binary backup as anchor if available)
LAST_DEPLOY=$(ls -t /tmp/*.bak-* 2>/dev/null | rg "$SERVICE" | head -1)
git -C "$SVC_PATH" diff > "$WORK/staged.diff"
git -C "$SVC_PATH" diff --stat > "$WORK/staged.stat"

# Snapshot binary SHA pre-build for audit trail
BIN_NAME=$(basename "$SVC_PATH" | tr - _)
PRE_SHA=$(sha256sum ~/.local/bin/$BIN_NAME 2>/dev/null | cut -c1-16)
echo "Pre-deploy SHA: $PRE_SHA"

# Snapshot live service state pre-deploy (so we can compare post-deploy)
LIVE_HEALTH=$(curl -s -o /dev/null -w '%{http_code}' "http://localhost:$(rg -o "port = (\d+)" "$SVC_PATH/devenv-service.toml" | head -1)/health" 2>/dev/null)
echo "Live health pre-deploy: $LIVE_HEALTH"

echo "Ship gate $SHIP_ID for $SERVICE → $WORK"
```

## Wave 1 — Mechanical 4-stage gate (FIRST, before agent dispatch)

```bash
cd "$SVC_PATH"
{
  echo "## check"; cargo check 2>&1 | tail -10
  echo "## clippy"; cargo clippy -- -D warnings 2>&1 | tail -10
  echo "## pedantic"; cargo clippy -- -D warnings -W clippy::pedantic 2>&1 | tail -10
  echo "## test"; cargo test --lib --release 2>&1 | tail -20
} > "$WORK/mechanical-gate.txt"

if rg -q '(error|FAILED)' "$WORK/mechanical-gate.txt"; then
  echo "ABORT: mechanical gate failed. Fix before agent gate."
  exit 1
fi
```

## Wave 2 — Four agents in parallel (single message, 4 tool calls)

### `security-scanning:security-auditor`
```
Audit this diff for security regressions in $SERVICE pre-deploy:
- B1 secrets in code/logs
- B2 injection (SQL, shell, deserialize)
- B3 info disclosure (panic msgs, stack traces in responses)
- B4 fail-closed DoS (unbounded queues, missing timeouts)
- B5 error type leaks (PvError → public response)
- New deps or version bumps?
- Habitat traps: URL prefix (no http://), POVM write-only, lock ordering
Verdict: APPROVE / APPROVE-WITH-NITS / REJECT, file:line evidence.

DIFF:
<paste $WORK/staged.diff>
```

### `observability-monitoring:performance-engineer`
```
Audit this diff for performance regressions:
- New allocations in hot paths (hooks, RALPH tick, IPC dispatch)
- New blocking IO in async contexts (sync HTTP in tokio::spawn = AP29)
- New unbounded loops or recursion
- Lock acquisition order changes (deadlock risk)
- Test latencies if any new bench in this diff
Verdict: APPROVE/REJECT, with hot-path file:line refs.

DIFF:
<paste $WORK/staged.diff>
```

### `pr-review-toolkit:silent-failure-hunter`
```
Hunt this diff for silent-failure regressions per /bridge-silent-failure-hunt
patterns P1-P7. Pre-deploy gate, so HIGH-confidence findings only.
Verdict: APPROVE / REJECT (with file:line per finding).

DIFF:
<paste $WORK/staged.diff>
```

### `zen` — god-tier capstone
```
Final ULTRAPLATE god-tier review pre-deploy for $SERVICE. The other 3 reviewers
have audited security, perf, silent-failure. You catch what they don't:
- A1-A8 Rust idiom (unwrap, unsafe, Send-Sync, Errors docs, tracing, allow, glob, RwLock)
- Architecture coherence with rest of codebase
- Test coverage adequacy for the change scope
- Anything that "smells" wrong to a senior reviewer
Verdict: APPROVE / APPROVE-WITH-NITS / REJECT.

DIFF:
<paste $WORK/staged.diff>
MECHANICAL_GATE:
<paste $WORK/mechanical-gate.txt>
```

Each writes to `$WORK/<agent>.md`.

## Wave 3 — Gate decision (bash, no agent — boolean AND)

```bash
APPROVED=true
for a in security-auditor performance-engineer silent-failure-hunter zen; do
  V=$(rg -o '^(APPROVE[^-\s]*|REJECT)' "$WORK/$a.md" 2>/dev/null | head -1)
  echo "$a: ${V:-NO_VERDICT}"
  [[ "$V" == "REJECT" ]] && APPROVED=false
  [[ -z "$V" ]] && APPROVED=false
done

if $APPROVED; then
  echo "GATE OPEN — proceed to /forge or binary cp"
  echo "$SHIP_ID APPROVED $(date -Iseconds)" >> "$WORK/SHIP_RECEIPT.md"
else
  echo "GATE CLOSED — address rejections before deploy"
  echo "Rejections:"
  rg -A 5 '^REJECT' "$WORK"/*.md | head -30
  exit 1
fi
```

## Post-flight (only if APPROVE)

```bash
# Build receipt for the deploy log
{
  echo "# Pre-deploy gate $SHIP_ID — $SERVICE"
  echo "Diff: $(cat $WORK/staged.stat | tail -1)"
  echo "Mechanical: PASS"
  for a in security-auditor performance-engineer silent-failure-hunter zen; do
    echo ""
    echo "## $a"
    head -30 "$WORK/$a.md"
  done
} > "$WORK/SHIP_RECEIPT.md"

echo "Receipt: $WORK/SHIP_RECEIPT.md"
echo "Hand off to /forge $SERVICE or your deploy chain."
```

## Anti-patterns

- Don't run agents before the mechanical gate passes — wastes their time on broken code
- Don't skip a verdict on grounds "the others approved" — boolean AND is the gate
- Don't deploy with NITS unaddressed unless explicitly time-pressured (incident)
- Don't gate on >1000-line diffs — split first, gate per chunk
- Don't reuse a SHIP_ID across attempts — receipts are append-only audit trail

## Reference

CLAUDE.md `Quality Gate Protocol` (mandatory). Companion: `/forge` (build+deploy+verify),
`/soak` (post-deploy verifier). S099 deploy reference: 5 fixes shipped via this gate
class with zero rollbacks.


---

> Vault navigation: [[../../BOILERPLATE_INDEX|BOILERPLATE_INDEX]] · [[../../README|boilerplate modules README]] · [[../../../HOME|HOME]] · [[../../../MASTER_INDEX|MASTER_INDEX]]
> Reference-only clone — see [[../../BOILERPLATE_INDEX]] for upstream source + target-module mapping.
