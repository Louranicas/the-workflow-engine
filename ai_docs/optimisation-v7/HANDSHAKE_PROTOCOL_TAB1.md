---
title: HANDSHAKE PROTOCOL — Tab 1 Orchestrator (Command + Command-2 + Command-3)
date: 2026-05-17 (S1001982)
kind: planning-only · session-start protocol
purpose: deterministic 4-step handshake at session start; failure-mode register; dual-silence escalation
inheritance: Battern protocol (Design→Dispatch→Gate→Collect→Synthesize→Compose) for build waves
---

# HANDSHAKE PROTOCOL — Tab 1 Orchestrator

> Back to: [[TASK_LIST_V7_OPTIMISATION.md]] · [[AGENT_VIEW_GITWORKTREES.md]] · [[ULTRAMAP.md]]
>
> **Function:** ensure all 3 Tab-1 CC instances (Command top-left, Command-2 bottom-left, Command-3 bottom-right) are alive + aware + at-known-state before any coordinated work. Live observation 2026-05-17: handshakes filed at 11:45 + 11:57; both silent. This protocol exists to make silence operational (not a black hole).

---

## Roles (canonical seat map)

| Seat | Role | Lane | Comms |
|---|---|---|---|
| **Command** | Tab 1 top-left | orchestrator-lead; Path-C chair (contingent); Wave-merge | this pane; receive-mode for peers |
| **Command-2** | Tab 1 bottom-left | workflow-trace chair (closed); Path-A build-executor on G9; primary author Wave 1+2+3 modules | file-drop |
| **Command-3** | Tab 1 bottom-right | librarian standby; CR-2 SHIPPED; Cluster G lane (m30-m33) | file-drop |

**Channel:** `~/projects/shared-context/agent-cross-talk/` — append-only file drops, UTC-timestamped filenames (`YYYY-MM-DDTHHMMSSZ_{sender}_{recipient}_{topic}.md`).

---

## The 4-step protocol

### Step 1 — Command emits handshake-OPEN

```bash
# Command at session start (Tab 1 top-left):
TS=$(date -u +%Y-%m-%dT%H%M%SZ)
cat > ~/projects/shared-context/agent-cross-talk/${TS}_command_handshake_open.md <<EOF
HANDSHAKE-OPEN — session $(date -u +%Y-%m-%d_%H%M)
From: Command (Tab 1 top-left)
To:   Command-2 (Tab 1 bottom-left), Command-3 (Tab 1 bottom-right)
CC:   Watcher ☤ (synthex-v2 :8092), Zen (Tab 10)
Ack window: 30 minutes (HHMM+30 in local time)
Session context: <one-line — current workstream + last gate state>
Open blockers (B1-B6): <subset relevant to this session>
Request: each peer reply with HANDSHAKE-ACK file containing:
  1. peer seat identifier
  2. current state (idle / working-on-X / waiting-for-Y)
  3. last action timestamp
  4. any blockers to receiving Wave dispatch
EOF
echo "Filed: ${TS}_command_handshake_open.md"
```

### Step 2 — Peers reply HANDSHAKE-ACK

```bash
# Command-2 OR Command-3 within 30-minute window:
TS=$(date -u +%Y-%m-%dT%H%M%SZ)
cat > ~/projects/shared-context/agent-cross-talk/${TS}_{c2_or_c3}_handshake_ack.md <<EOF
HANDSHAKE-ACK — session $(date -u +%Y-%m-%d_%H%M)
From: Command-{2|3} (Tab 1 bottom-{left|right})
To:   Command (Tab 1 top-left)
Re:   {TS_of_open}_command_handshake_open.md

State: {idle | working-on-<workstream> | waiting-for-<X>}
Last action: <timestamp + brief>
Blockers to Wave dispatch: <list OR none>
Ready for: <Wave-1 modules m{N-M} | Wave-2 modules m{N-M} | Wave-3 modules m{N-M} | observation only>
EOF
```

### Step 3 — Command files HANDSHAKE-VERIFIED

After both ACKs received (or 30-min window expires — see Step 4):

```bash
TS=$(date -u +%Y-%m-%dT%H%M%SZ)
cat > ~/projects/shared-context/agent-cross-talk/${TS}_command_handshake_verified.md <<EOF
HANDSHAKE-VERIFIED — session $(date -u +%Y-%m-%d_%H%M)
From: Command (Tab 1 top-left)
To:   Command-2, Command-3
CC:   Watcher, Zen

Peer states:
  Command-2: <ack content summary>
  Command-3: <ack content summary>

Session work plan: <ordered list of what each peer will do>
Wave allocation: <which peer gets which Wave>
Ack-window resolved at: $(date -u +%Y-%m-%dT%H%M%SZ)
EOF
```

### Step 4 — Silence handling (dual-silence escalation)

**30-min ack window expires; one or both peers silent.** Three branches:

**Branch A — single silence (one peer ack'd, one silent):**
```bash
TS=$(date -u +%Y-%m-%dT%H%M%SZ)
cat > ~/projects/shared-context/agent-cross-talk/${TS}_command_handshake_partial.md <<EOF
HANDSHAKE-PARTIAL — session $(date -u +%Y-%m-%d_%H%M)
ACK received from: Command-{X}
Silent: Command-{Y}

Decision: proceed with single-peer work plan; do NOT dispatch to silent peer.
Re-handshake attempt: file new HANDSHAKE-OPEN at $(date -u -d '+1 hour' +%Y-%m-%dT%H%M%SZ)
Class-A flag candidate: silent peer may be in suspended state — Watcher: please observe Tab-1 dump-screen at next tick
EOF
```

**Branch B — dual silence (both peers silent):**
```bash
# 30-min window expires; neither C-2 nor C-3 responded.
# Live observation 2026-05-17 S1001982: this happened twice (11:45 + 11:57 handshakes).

TS=$(date -u +%Y-%m-%dT%H%M%SZ)
cat > ~/projects/shared-context/agent-cross-talk/${TS}_command_handshake_dual_silence.md <<EOF
HANDSHAKE-DUAL-SILENCE — session $(date -u +%Y-%m-%d_%H%M)
Both peers silent through 30-min window.

Possible causes (ranked):
  1. Both panes suspended (Zellij focus-state; common after Luke navigation)
  2. Both panes in long-running task — not actually idle
  3. /clear ran in both panes without restart
  4. Atuin agent-cross-talk/ scan hook absent in C-2 + C-3 startup

Mitigation actions:
  - Do NOT dispatch any Wave to silent peers (AP-V7-08 — silence ≠ consent)
  - Continue Command-only work (orchestration, planning, vault writes)
  - File escalation to Luke at $(date -u +%Y-%m-%dT%H%M%SZ)
  - Watcher: tick journal entry requested for Tab-1 dump-screen state

Escalation to Luke:
EOF

# Then file ESCALATION:
TS2=$(date -u +%Y-%m-%dT%H%M%SZ)
cat > ~/projects/shared-context/agent-cross-talk/${TS2}_command_escalation_dual_silence.md <<EOF
ESCALATION — Luke @ node 0.A
Issue: Command-2 + Command-3 both unresponsive through 30-min handshake window
Filed handshakes: <list timestamps>
Action requested: please check Tab-1 pane state; resume if suspended; type 'hi' in each to wake
Workaround: Command proceeds with orchestration + Command-side-only work
EOF
```

**Branch C — handshake response after window (late ack):**
Treat as new HANDSHAKE-OPEN from peer-side initiating; re-run from Step 1 with peer as initiator.

---

## Session-start checklist (Command runs this every session)

```bash
# 1. Verify own pane identity
echo "I am: Command (Tab 1 top-left)"
echo "Session: $(date -u +%Y-%m-%dT%H%M%SZ)"
echo "ZELLIJ_PANE_ID: ${ZELLIJ_PANE_ID:-unset}"

# 2. Check for in-flight handshakes (last 24h)
find ~/projects/shared-context/agent-cross-talk/ -name "*handshake*" -mmin -1440 -type f | head -10

# 3. Check for peer drops since last Command file
last_command_drop=$(ls -t ~/projects/shared-context/agent-cross-talk/*command*.md 2>/dev/null | head -1)
if [[ -n "$last_command_drop" ]]; then
  find ~/projects/shared-context/agent-cross-talk/ -newer "$last_command_drop" -name "*.md" | head -20
fi

# 4. Probe Watcher
~/.local/bin/watcher status 2>/dev/null | head -5

# 5. Emit HANDSHAKE-OPEN (Step 1)
# (...as above...)
```

---

## Battern protocol for build waves (post-G9)

Once G9 fires + handshakes verified, Wave dispatch uses Battern (6-step):

| Battern step | Tab-1 action | Output |
|---|---|---|
| 1. **Design** | Command composes Wave plan (which modules, which peer, time budget) | `WAVE_{N}_PLAN.md` in agent-cross-talk/ |
| 2. **Dispatch** | Command file-drops per-peer module assignment | `WAVE_{N}_DISPATCH_{C2\|C3}.md` |
| 3. **Gate** | Each peer files ACK + estimated completion | `WAVE_{N}_GATE_{C2\|C3}.md` |
| 4. **Collect** | Peers file completion notice + per-module deliverable list | `WAVE_{N}_COLLECT_{C2\|C3}.md` |
| 5. **Synthesize** | Command runs cross-peer integration check (4-stage QG on integrated branch) | `WAVE_{N}_SYNTHESIZE.md` |
| 6. **Compose** | Command merges Wave to main; tags `wave-N-complete` | git tag + `WAVE_{N}_COMPOSE.md` |

**Source:** Battern protocol skill (`/battern`). Used for V1 ULTIMATE_DEPLOYMENT_FRAMEWORK 9-agent author wave; will be used for V7 module-plans + runbooks parallel author waves AND for post-G9 implementation waves.

---

## Failure-mode register

| ID | Failure | Detection | Mitigation |
|---|---|---|---|
| H-01 | Peer's pane suspended at handshake time | dual-silence per Branch B | Luke wake-up; do NOT proceed assuming ack |
| H-02 | Peer ack but state stale (says "idle" but actually working on prior) | ack content contradicts most-recent peer file | re-handshake; ack from peer must reference recent file |
| H-03 | Battern Step 5 synthesize finds merge conflict | Wave-end 4-stage QG fails | resolve, re-run; do NOT auto-resolve |
| H-04 | Peer files unsolicited (no preceding OPEN) | drop without `Re:` reference | accept as out-of-band info; ack receipt; do NOT treat as ACK to non-existent OPEN |
| H-05 | Watcher tick logs Tab-1 silent >2h despite live work | atuin shows Command file activity but no peer drops | escalate to Luke per Branch B even outside 30-min window |
| H-06 | TZ drift in filenames (local time labelled as Z) | filename `113200Z` vs file mtime `01:34:03 UTC` | use `date -u +%Y-%m-%dT%H%M%SZ` exclusively |

**Tracked at:** AP-V7-08 in ANTIPATTERNS_REGISTER.md.

---

## Watcher integration (WCP)

Watcher observes Tab-1 via its journal (already active per S1001982 watcher journal). Watcher tick·n entries log:
- Tab-1 silent duration
- Peer-pane suspended state (via Zellij dump-screen)
- Class-A activation transitions (handshake ACK received)
- Class-H atuin proprioception anomalies (silent peer despite active work)

Watcher does NOT initiate handshakes (observer-only per WCP); Command initiates.

WCP commands available from any pane:
```bash
watcher status
watcher hello                    # explicit ping to Watcher
watcher observe <event>          # log observation
watcher recent                   # last N WCP events
```

---

## Atuin trajectory (provenance)

Every handshake leaves traceable atuin history:

```bash
# Verify session handshakes:
atuin search "handshake" --before 1h --limit 20
atuin search "agent-cross-talk" --before 1h --limit 20

# Per-day handshake count (Tab-1 health metric):
atuin stats --period day | grep "handshake"
```

---

## Live state (2026-05-17 S1001982)

**Last handshakes from Command:**
- 11:45 local — silent
- 11:57 local — silent

**Per Branch B mitigation:** Command continues V7 optimisation single-handed. Dispatch to C-2 / C-3 deferred until they ack. **All V7 deliverables Command-authored** (no peer-dispatch in V7 author wave). Post-V7, fresh handshake attempt before any Wave-1 dispatch.

---

*HANDSHAKE_PROTOCOL_TAB1 authored 2026-05-17 by Command. 4-step protocol + 6-failure-mode register + dual-silence escalation operational.*
