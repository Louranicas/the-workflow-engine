---
title: Runbook 10 — Cross-Cutting Concerns CC-1..CC-7
date: 2026-05-17 (S1001982)
kind: planning-only · operational runbook · 7 concerns running continuously across phases 0-6
phase: cross-cutting (parallel to phases 1-6; not sequential)
concerns: 7 — Drift Register · Rollback · Watcher Observability · Atuin Proprioception · V8↔V3 Wire · /scaffold Bound · Power-Structure Resolution
owner: Command (mediator) + Luke @ node 0.A (authority) + Watcher (recorder) + Zen (audit) — per-concern
cadence: continuous OR on-event (per concern)
authority: Luke @ node 0.A
status: planning-only · HOLD-v2 active · NOT executable until G1-G9 GREEN
---

# Runbook 10 — Cross-Cutting Concerns CC-1..CC-7

> Back to: [[../TASK_LIST_V7_OPTIMISATION.md]] · [[../ULTRAMAP.md]] · sibling [[runbook-06-phase-5-deploy-soak.md]] · [[runbook-08-phase-7-security.md]] · [[runbook-09-phase-8-observability.md]] · [[runbook-11-emergency-rollback.md]]
>
> Source phase doc: [[../../the-workflow-engine-vault/deployment framework/phase-6-sunset-and-cross-cutting.md]] (7 cross-cutting concerns section + appendix); auxiliary [[../../the-workflow-engine-vault/deployment framework/phase-7-security-compliance.md]] · [[../../the-workflow-engine-vault/deployment framework/phase-8-observability-operations.md]]

---

## Overview

Seven cross-cutting concerns run continuously across the workflow-trace lifecycle. They are **not also-rans** — each addresses a failure mode directly observed in a prior ancestor or peer codebase: planning sprawl (CC-1 Drift Register), ungated rollback chaos (CC-2 Rollback), invisible degradation (CC-3 Watcher Observability), cross-tool provenance loss (CC-4 Atuin Proprioception), reinvented protocols (CC-5 V8↔V3 Wire), convention drift (CC-6 `/scaffold` Bound), ambiguous authority (CC-7 Power-Structure Resolution). Encoding these as **living procedures with owners, cadences, and failure modes** is what separates this deployment from its predecessors. Each concern below carries: purpose · cadence · owner · phase integration · commands · failure mode · Watcher flag class.

---

## Pre-flight checklist

- `~/.local/share/atuin/wf-drift-audit.tsv` writable (CC-1 audit log)
- `~/projects/shared-context/watcher-notices/` writable (CC-3 Watcher channel)
- `~/projects/shared-context/agent-cross-talk/` writable (CC-2 + CC-7 file-drop)
- atuin daemon running (CC-4)
- `WAIVER_REGISTER.md` initialised at workspace root (CC-7)
- DevOps Engine V3 `:8082` reachable (CC-5)
- `/scaffold` CLI available at `~/.local/bin/scaffold` (CC-6)

---

## CC-1 — Drift Register (LCM-inspired 11-dimension audit)

**Purpose:** prevent the "verify contract not wiring" failure mode (LCM Drift #11 generalisation) — agent drift catches at orchestrator level via re-executed checks, not read-only review.

**Cadence:** on every session resume that involves workflow-trace work. **Not** on schedule — on human intent. Session that doesn't touch workflow-trace = no audit needed. Session that makes any code/doc change = audit before close.

**Owner:** Command (Tab 1 Orchestrator). Watcher observes but does not run. Zen receives audit result at each G7 gate re-evaluation.

**Phase integration:**

- Phase 0 pre-genesis: dimensions 1-6 (architecture-layer) after each G1-G8 gate flip
- Phase 1 genesis: dimensions 2, 5, 9 primary (scaffold alignment, script parity, manifest freshness)
- Phase 2A/B build: **all 11 dimensions active**; dim 11 (wiring not contract) is highest-frequency failure mode
- Phase 3 integration: dims 6, 7, 10 active (JSONL bounded, receipt-DAG schema, cross-ref parity)
- Phase 4 hardening: dim 8 (forbidden patterns absent) is quality gate primary surface
- Phase 5 deploy/soak: dims 3, 4 (module-spec vs binary export parity; Cargo safety invariants)

**Commands (drop-in 11-dimension audit):**

```bash
set -o pipefail
cd /home/louranicas/claude-code-workspace/the-workflow-engine

# Dim 1 — plan.toml ↔ module src/ alignment
grep -c 'id = "M' plan.toml
find src -name 'mod.rs' -path '*/m[0-9]*/*' | wc -l
# These two numbers must match

# Dim 2 — layer-doc count matches src/mN_ directories
ls ai_docs/layers/ 2>/dev/null | wc -l
find src -maxdepth 1 -name 'm[0-9]*' -type d | wc -l

# Dim 3 — module-spec count matches binary export
ls the-workflow-engine-vault/module\ specs/cluster-*.md | wc -l

# Dim 4 — Cargo safety invariants in lib.rs
grep -c 'forbid(unsafe_code)' src/lib.rs    # ≥ 1
grep -c 'deny.*unwrap' src/lib.rs           # ≥ 1

# Dim 5 — script parity scripts/ vs scaffold-status.json
ls scripts/ | sort > /tmp/wt-scripts-actual.txt
jq -r '.scripts[]' .deployment-work/status/scaffold-status.json 2>/dev/null | sort > /tmp/wt-scripts-claimed.txt
diff /tmp/wt-scripts-actual.txt /tmp/wt-scripts-claimed.txt
# Empty diff required

# Dim 6 — m7 workflow_runs JSONB schema unchanged (SHA against reference)
sqlite3 workflow_trace.db "SELECT sql FROM sqlite_master WHERE name='workflow_runs'" | sha256sum

# Dim 7 — m15 reservation JSONL bounded (< 10 files in 7d)
ls ~/projects/shared-context/agent-cross-talk/PHASE-B-RESERVATION-NOTICE-*.jsonl 2>/dev/null | wc -l
# > 10 in 7d → Watcher Class-E

# Dim 8 — forbidden patterns absent
rg --hidden -l 'recommend_|auto_start|smart_|rewrite_|route_bypass|optimise_without' src/
# Zero matches required

# Dim 9 — manifest freshness
sha256sum -c SHA256SUMS.txt 2>&1 | grep -c FAILED
# Zero failures

# Dim 10 — vault wikilinks resolve
rg '\[\[' the-workflow-engine-vault/ -o --no-filename | grep -oP '(?<=\[\[)[^\]]+' | sort -u > /tmp/wt-wikilinks-claimed.txt

# Dim 11 — verify WIRING not contract (THE LOAD-BEARING ONE — full gate, NOT scoped)
CARGO_TARGET_DIR=./target cargo check --workspace --all-targets --all-features 2>&1 | tail -5
[[ ${PIPESTATUS[0]} -ne 0 ]] && { echo "DIM11 FAIL: check"; exit 11; }
cargo clippy --workspace --all-targets --all-features -- -D warnings 2>&1 | tail -5
[[ ${PIPESTATUS[0]} -ne 0 ]] && { echo "DIM11 FAIL: clippy"; exit 11; }
cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic 2>&1 | tail -5
[[ ${PIPESTATUS[0]} -ne 0 ]] && { echo "DIM11 FAIL: pedantic"; exit 11; }
cargo test --workspace --all-targets --release 2>&1 | tail -10
[[ ${PIPESTATUS[0]} -ne 0 ]] && { echo "DIM11 FAIL: test"; exit 11; }

# Independent verification (NEVER skip — AP-Drift-01 prevention)
git log -1 --oneline                       # confirm SHA you THINK you tested
# Exercise at least one new code path, NOT just compile it (LCM Drift #11)
```

**Outputs:** TSV append to `~/.local/share/atuin/wf-drift-audit.tsv`:

```
timestamp  session_id  dims_pass  dims_fail  dim11_status  notes
```

**Failure mode:** F-CC1-1 — agent reports gate-clean against scoped invocation; orchestrator ships green summary while clippy was screaming (S1001882 PIPESTATUS near-miss). **Mitigation: dim 11 mandates `--all-features` full gate; do NOT trust agent reports.**

**Watcher class:** Class-D (four-surface drift) on any dim 1-10 failure; Class-G on dim 11 failure (substrate-frame engine vs binary divergence).

---

## CC-2 — Rollback Procedure

**Purpose:** each phase has scripted, documented rollback so failed phase reverses without destructive improvisation.

**Cadence:** on-event — when phase gate fails AND decision is roll back (not fix-forward).

**Owner:** Command executes; Luke authorises; Watcher records Class-B (hand-off boundary reversed).

**Phase integration & commands:** see [[runbook-11-emergency-rollback.md]] for the per-phase 3-command rollback sequences (Phase 1 / 5A / 5B cutover are the canonical scenarios). Phase 0, 2A/B, 3, 4 rollback scripts also documented there.

**Global rollback guard:** before any rollback that touches git, `git stash list` and confirm no uncommitted work in current session is in stash. **AP-Hab-08** (stash pop wrong stash). Per S1001882 generalisation: never trust rollback succeeded without independently verifying resulting state.

**Failure mode:** F-CC2-1 — silent rollback success claim while binary still present in `~/.local/bin/`. Mitigation: post-rollback `ls -la ~/.local/bin/wf-*` independent verification.

**Watcher class:** Class-B always (hand-off reversed).

---

## CC-3 — Watcher Observability

**Purpose:** Watcher records the full deployment in a structured watch journal so workflow-level improvement candidates (IC-N) can be mined and handed to the next codebase generation — **improvement loop closure, not a debugging tool**.

**Cadence:** **prompt-driven**, not autonomous. Flag recording continuous (every tick). Synthesis is periodic: phase boundaries + D60 + D90 + D120. **No unsolicited synthesis during active build phases** (avoids interrupting flow).

**Owner:** the Watcher ☤. Command receives WCP notices when Watcher dispatches them. Zen receives synthesis at gate boundaries.

**Phase integration table:**

| Phase | Watcher activity | Flag classes active |
|---|---|---|
| Phase 0 | T0 baseline captured; yellow signals documented | A, E |
| Phase 1 | **Class-F guard active** (pre-G9 src/*.rs hard violation); Class-A on G9 flip | A, E, F |
| Phase 2A/B | Class-D primary (four-surface drift); each cluster ship → Class-B; first Cluster H stub | A, B, D, I |
| Phase 3 | Class-B integration hand-offs; Class-H atuin proprio; **Class-I live** | B, H, I |
| Phase 4 | Class-C if confidence gate refuses; Class-G if substrate-frame confusion in test design | C, G |
| Phase 5 | Class-I primary (Cluster H must show Hebbian activity); Class-A successful deploy | A, I |

**Synthesis format (IC-N candidates):**

```markdown
Improvement candidate IC-<N>:
  Observed:        <evidence>
  Phase:           <which phase>
  Flag class:      <A-I>
  Candidate:       <what should differ in next-generation codebase>
  Confidence:      <High|Medium|Low>
  Evidence:        <dispatch counts, metric values, atuin trajectory IDs>
```

**Commands (Watcher synthesis trigger from Command):**

```bash
TS=$(date -u +%Y-%m-%dT%H%M%SZ)
WEEK_N="<N>"
cat > ~/projects/shared-context/watcher-notices/${TS}_workflow_trace_week_${WEEK_N}_synthesis_request.md <<EOF
# WCP — workflow-trace Week ${WEEK_N} Synthesis Request

**To:** Watcher ☤
Cover: Class-E new flags · Class-I status · m15 pressure notices · m14 lift trajectory · m11 decay state · Cluster H circuit breaker
EOF
```

**Failure mode:** F-CC3-1 — Watcher synthesis cadence unfired; journal accumulates ticks but no fold. Mitigation: Command weekly WCP synthesis trigger (Step 3.2 in [[runbook-06-phase-5-deploy-soak.md]]).

**Watcher class:** all classes — this concern IS the Watcher.

---

## CC-4 — Atuin Proprioception (Cross-Tool Provenance)

**Purpose:** atuin is the **ONLY** cross-tool provenance record (S1002029 finding #4). V3 deploys, V8 module generations, `/scaffold` materialisations, and `wf-*` CLI invocations all converge in `~/.local/share/atuin/history.db`. Without atuin tagging, forensic replay of "how did this deployment go" is impossible.

**Cadence:** continuous (atuin records by default). Explicit audits at phase boundaries + D120.

**Owner:** Command ensures `--workspace the-workflow-engine` tag applied to build/deploy commands.

**Commands:**

```bash
# Set workspace context at session start
atuin kv set wf-session-id "S$(date +%Y%m%d%H%M)"
atuin kv set wf-workspace  "the-workflow-engine"

# Post-build provenance audit (end of each phase)
atuin search --workspace the-workflow-engine --limit 200 \
  | grep -E '(cargo build|cargo test|cargo clippy|wf-|scaffold|devenv)' \
  > /tmp/wt-trajectory-phase-N.txt

# D120 trajectory replay (full retirement record)
atuin search --workspace the-workflow-engine \
  > ~/projects/shared-context/wf-retirement/atuin-trajectory-D120.txt

# Dispatch trajectory consistency check (Phase 3+)
DISPATCH_COUNT=$(atuin search --workspace the-workflow-engine | grep 'wf-dispatch execute' | wc -l)
M7_COUNT=$(sqlite3 ~/.local/share/workflow-trace/correlations.db "SELECT COUNT(*) FROM workflow_runs;")
# Must match within ±2 for same-session events
```

**Failure mode:** F-CC4-1 — session made build/deploy changes but atuin history shows no matching commands (atuin daemon was not running). Mitigation: session-start probe `atuin status` BEFORE any build operation. F-CC4-2 — `wf-dispatch dispatch` in atuin but no row in m7 → silent dispatch failure. Mitigation: dispatch_audit consistency check above.

**Watcher class:** Class-H (atuin proprioception anomaly) — fires when atuin gap detected; cross-tool gap = missing forensic record.

---

## CC-5 — V8 ↔ V3 Bidirectional Wire (Existing Protocol — Inherit, Don't Reinvent)

**Purpose:** V8 ↔ V3 bidirectional Hebbian feedback protocol (`POST :8082/api/v8/confidence` and `POST :8082/api/v8/learning`) **already exists** and carries habitat-level Hebbian feedback. workflow-trace inherits via m42; it does NOT reinvent.

**Cadence:** continuous during Phase 5 soak. m42 fires on each dispatch outcome.

**Owner:** m42 (`hebbian_feedback`) is call site within workflow-trace. DevOps Engine V3 `:8082` is receiving service. Neither is workflow-trace's responsibility to operate.

**Commands (Phase 5 wire verification at start):**

```bash
# Probe (200 or 400 = alive; 000/404 = drift; resolve before Phase 5 proceeds)
curl -s -o /dev/null -w '%{http_code}' \
  -X POST http://localhost:8082/api/v8/confidence \
  -H 'Content-Type: application/json' \
  -d '{"workflow_id":"__probe__","confidence":0.0,"session_id":"probe"}'

# m42 outbound call pattern
# (Rust pseudocode — actual call in m42 module)
# POST /api/v8/confidence {workflow_id, confidence, session_id, source: "workflow-trace/m42"}
# POST /api/v8/learning   {workflow_id, outcome: "ltp"|"ltd", delta, source: "workflow-trace/m42"}
```

**Failure mode:** F-CC5-1 — wire down; m42 falls back to outbox JSONL + circuit breaker (Closed → Open → HalfOpen; 5 fail → Open; 60s → HalfOpen). Does NOT block dispatch (correct fail-open posture). Watcher Class-I flags if sustained > 5 invocations.

**Watcher class:** Class-D if wire-document says endpoints exist but `:8082` says they don't; Class-I if m42 silent for sustained period.

---

## CC-6 — /scaffold as V8's Bound

**Purpose:** `/scaffold` enforces 8-layer `mN_<theme>/` convention regardless of what V8's plan.toml generates. If V8 drifts, `/scaffold` is the corrective. **The two tools compose** — V8 generates intent; `/scaffold` materialises structure; V8 fills module bodies against materialised tree. Each is the other's bound.

**Cadence:** on-event — Phase 1 genesis (initial tree); any Phase 2A/B cluster implementation adding new `src/mN_*/` directory. **NOT** invoked on every session resume.

**Owner:** Command invokes `/scaffold`. V8 generates `plan.toml`. Zen audits both at G7 (spec audit gate).

**Commands:**

```bash
# Phase 1 — initial scaffold from plan.toml
/scaffold --plan ~/claude-code-workspace/the-workflow-engine/plan.toml \
          --output ~/claude-code-workspace/the-workflow-engine/src/
# /scaffold ERRORS — does NOT silently generate non-conforming tree — if V8 plan.toml deviates

# Phase 2A/B — per-cluster verification after each cluster implementation
/scaffold --verify --cluster D --plan plan.toml
# Verifies: all modules in cluster have source files; no extra files outside tree
```

**Failure mode:** F-CC6-1 — V8 generates `m_trust_cluster` instead of `m8_povm_build_prereq` → tree deviation → Dim 2 of Drift Register fails on first audit; convention drift becomes load-bearing scar tissue. Mitigation: `/scaffold` hard-error.

**Watcher class:** Class-D (drift between V8 intent and `/scaffold` enforcement).

---

## CC-7 — Power-Structure Resolution Protocol

**Purpose:** define resolution path when Luke's unilateral override conflicts with Zen's G7 spec audit verdict — preserve gate credibility AND document override (rather than silently absorb).

**Cadence:** on-event — conflict between Luke override and Zen audit identified. Pre-identified as "power-structure ambiguity" in Part VIII of GOD_TIER_CONSOLIDATION.

**Owner:** Luke @ node 0.A final authority. Watcher records. Zen audits. Command mediates. **Not a veto mechanism** — Luke can override — but documentation mechanism preventing invisible overrides.

**Commands (4-step protocol):**

```bash
# Step 1 — Luke issues explicit per-gate waiver (NOT implicit continuation)
# Statement must be explicit: "I am overriding Zen's G7 objection to [element] on the basis of [reason]. Risk class accepted: [class]."

cat >> ~/claude-code-workspace/the-workflow-engine/WAIVER_REGISTER.md <<EOF
- gate: G7
  date: $(date -u +%Y-%m-%dT%H:%M:%SZ)
  session: S<id>
  waived_element: "<specific Zen objection>"
  luke_rationale: "<explicit reason>"
  risk_class: "<Watcher class covering this risk>"
  accepted_by: "Luke @ node 0.A"
EOF

# Step 2 — Watcher records via WCP notice (structured, not editorial; does NOT argue, only records)
TS=$(date -u +%Y-%m-%dT%H%M%SZ)
cat > ~/projects/shared-context/watcher-notices/${TS}_waiver_g7_<element>.md <<EOF
# WCP — G7 Waiver Recorded
Waiver YAML entry: WAIVER_REGISTER.md at <line N>
Associated Watcher class: <A-I>
EOF

# Step 3 — m15 emits PHASE-B-RESERVATION-NOTICE if waived element involves forbidden-verb pressure
# (auto on m15 detection; manual seed only if Watcher synthesis identifies pattern)

# Step 4 — Next spec amendment (v1.4 G5 interview) MUST reference waiver
# Precedent cannot be silently absorbed; named + re-audited at next gate
```

**Failure mode:** F-CC7-1 — Luke overrides without documentation; override absorbed silently into next spec patch; Zen's next audit approves without knowing previously contested; objection reasoning lost. **This is planning-sprawl equivalent at governance level.** Mitigation: Step 1 explicit YAML entry is non-negotiable; Watcher refuses to ratify a waiver without it.

**Watcher class:** Class-A (waiver = activation transition on gate timeline); Class-E if waiver pattern shows ancestor-rhyme accumulation (3+ G7 overrides in single session).

---

## Phase-end gate

Cross-cutting concerns have no discrete end. Per-phase rollups:

```bash
TS=$(date -u +%Y-%m-%dT%H:%M:%SZ)
PHASE="<1|2A|2B|3|4|5A|5B|5C|6>"

# CC-1 audit hash for this phase
echo "phase=${PHASE} ts=${TS} drift_audit=<dims_pass>/<11>" \
  >> ~/.local/share/atuin/wf-drift-audit.tsv

# CC-3 Watcher tick count
WATCHER_TICKS=$(ls ~/projects/shared-context/watcher-notices/*workflow_trace* 2>/dev/null | wc -l)

# CC-4 atuin trajectory count
ATUIN_COUNT=$(atuin search --workspace the-workflow-engine --before 30d | wc -l)

# CC-7 waiver count this phase
WAIVER_COUNT=$(grep -c '^- gate:' ~/claude-code-workspace/the-workflow-engine/WAIVER_REGISTER.md 2>/dev/null || echo 0)

atuin kv set "workflow_trace.cc.phase${PHASE}.ts"        "$TS"
atuin kv set "workflow_trace.cc.phase${PHASE}.watcher_ticks"   "$WATCHER_TICKS"
atuin kv set "workflow_trace.cc.phase${PHASE}.atuin_count"     "$ATUIN_COUNT"
atuin kv set "workflow_trace.cc.phase${PHASE}.waiver_count"    "$WAIVER_COUNT"
```

---

## Failure modes register (consolidated across all 7 concerns)

| ID | Concern | Trigger | Detection | Mitigation |
|---|---|---|---|---|
| F-CC1-1 | Drift Register | scoped clippy claim shipped | dim 11 full gate disagrees | mandate `--workspace --all-targets --all-features` |
| F-CC2-1 | Rollback | silent rollback success claim | `ls -la ~/.local/bin/wf-*` shows binary still present | post-rollback independent verification |
| F-CC3-1 | Watcher | synthesis unfired | journal accumulates ticks without fold | Command weekly WCP trigger |
| F-CC4-1 | Atuin proprio | atuin daemon down during session | atuin history shows no `cargo`/`wf-*` for changed session | session-start `atuin status` probe |
| F-CC4-2 | Atuin proprio | wf-dispatch in atuin no m7 row | dispatch consistency check shows gap | investigate m7 schema or m32 audit-first ordering |
| F-CC5-1 | V8↔V3 wire | wire down sustained | m42 outbox accumulates 7d+ | circuit breaker correct fail-open; investigate `:8082` |
| F-CC6-1 | /scaffold bound | V8 generates non-conforming module name | `/scaffold --verify` hard-error | refuse merge; V8 plan.toml correction |
| F-CC7-1 | Power structure | Luke override sans YAML entry | `WAIVER_REGISTER.md` lacks gate/date/rationale | Watcher refuses to ratify; require structured waiver entry |

---

## Watcher flag pre-positioning (consolidated)

| Class | CC sources |
|---|---|
| A | CC-3 (gate flips) · CC-7 (waiver = activation transition) |
| B | CC-2 (rollback = hand-off reversed) · CC-3 (cluster boundary) |
| C | CC-3 (confidence gate refusal during build) |
| **D** | CC-1 (drift dim 1-10) · CC-3 (four-surface drift) · CC-5 (wire-doc divergence) · CC-6 (V8 vs scaffold drift) |
| E | CC-3 (ancestor-rhyme) · CC-7 (waiver accumulation) |
| F | CC-3 (AP24 violation) |
| G | CC-1 (dim 11 wiring not contract) · CC-3 (substrate-frame confusion) |
| **H** | CC-4 (atuin proprioception anomaly — primary owner) |
| **I** | CC-3 (Hebbian silence) · CC-5 (m42 wire silent sustained) |

---

## Atuin trajectory anchors

```bash
# Cross-concern audit
atuin scripts run wt-substrate-pulse                  # CC-3 Watcher flag counts
atuin scripts run wt-bridge-check                     # CC-5 wire health
atuin search --workspace the-workflow-engine --before 30d  # CC-4 full trajectory
atuin kv get  "workflow_trace.cc.phase5C.watcher_ticks"
atuin kv get  "workflow_trace.cc.phase5C.atuin_count"
atuin kv get  "workflow_trace.cc.phase5C.waiver_count"
```

---

## Sign-off

This runbook is **planning-only** (HOLD-v2). The 7 cross-cutting concerns are continuous (CC-3, CC-4, CC-5) or on-event (CC-1, CC-2, CC-6, CC-7). Each concern names its owner + cadence + integration point so no concern silently lapses. **CC-1 dim 11 is load-bearing** — the wiring-not-contract verification is the canonical defence against agent drift (S1001882 + LCM Drift #11 generalisation).

*Runbook 10 authored 2026-05-17 by Command (V7 optimisation, parallel author). 7 cross-cutting concerns operational. Per-concern: purpose · cadence · owner · phase integration · commands · failure mode · Watcher class. ~1,940 words. Source: phase-6-sunset-and-cross-cutting.md § 7 concerns. Sibling: runbook-06 / runbook-08 / runbook-09 / runbook-11.*
