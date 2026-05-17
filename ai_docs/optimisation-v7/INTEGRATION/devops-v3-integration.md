---
title: DevOps Engine V3 Integration Deep-Dive — workflow-trace V7
date: 2026-05-17 (S1001982)
kind: planning-only · integration deep-dive · expands G5 § DevOps V3
parent: GENERATIONS/G5-tooling.md
owner: Command (Phase 3 Track 5 wire-up); Phase 4 4-agent gate hardening; Phase 5A/B/C operations
contract: T1-T6 confidence-gated; resume_from honoured; V8↔V3 wire reused
---

# DevOps Engine V3 Integration — workflow-trace V7

> Back to: [[../TASK_LIST_V7_OPTIMISATION.md]] · [[../GENERATIONS/G5-tooling.md]] · [[../GENERATIONS/G4-gold-standard.md]] · [[../ULTRAMAP.md]]
>
> Siblings: [[scaffold-integration.md]] · [[atuin-integration.md]] · [[codesynthor-v8-integration.md]] · [[json-claude-code-optimisation.md]] · [[progressive-disclosure-obsidian.md]]

---

## Overview

DevOps Engine V3 (port `:8082`, `/health`, batch 1 per workspace CLAUDE.md service table) is the Habitat's canonical deploy-orchestration backbone. Its six-stage pipeline T1 Specify → T2 Scaffold → T3 Implement → T4 Harden → T5 Document → T6 Deploy is the **shape into which workflow-trace plugs** for both (a) self-deployment (workflow-trace itself ships through V3 at Phase 5A) and (b) downstream dispatch (when `wf-dispatch` invokes a deploy workflow, m41 LCM router uses `lcm.loop.create {max_iters: 1, resume_from: "T2"}` so V8+Zen own T1 and V3 owns T2-T6, per G4 GAP-Gold-04 closure). The V8↔V3 wire is already live (per G4 § V8 ↔ V3 wire reuse) — workflow-trace adds exactly one outbound call (`POST :8082/api/v8/learning {confidence_delta}` from m40_synthex_emit on m32 PassVerified) — zero reinvention. NAM-03 confidence gates per stage (T1≥0.80, T2≥0.80, T3≥0.85, T4≥0.85, T5≥0.90) are the hard halt conditions for any V3 invocation by workflow-trace, halting on any sub-threshold. Activation: Phase 3 Track 5 wire-up (Day 22-26); Phase 5A self-deploy (Day 28-30); Phase 5C downstream-dispatch becomes routine (D30→D120).

---

## T1-T6 mapping to wf-crystallise / wf-dispatch / Conductor

Per G5 § DevOps V3 integration T1-T6 line + G4 GAP-Gold-04. Each T-stage maps to a workflow-trace verb + the agent + the verification:

| T-stage | V3 phase | Workflow-trace verb | Owner | Output | Conductor involvement |
|---|---|---|---|---|---|
| **T1** | Specify | `wf-crystallise propose accept <proposal_id>` | m23 proposer → m30 bank | proposal admitted to curated bank with confidence ≥ 0.80 | none — V8+Zen author T1 spec |
| **T2** | Scaffold | `/scaffold check --plan plan.toml --src src/` | Command (foreground; per [[scaffold-integration.md]]) | drift-clean src/ tree with confidence ≥ 0.80 | none |
| **T3** | Implement | `cargo build --workspace --release --features full` (worktree-isolated per AGENT_VIEW_GITWORKTREES.md) | Command-2 (Waves 1-3 foreground) | release binaries `target/release/wf-crystallise` + `target/release/wf-dispatch` with confidence ≥ 0.85 | none — but worktree per layer per Wave |
| **T4** | Harden | 4-stage QG + 4-agent pre-deploy hardening (per [[../RUNBOOKS/runbook-05-phase-4-hardening.md]]) | Zen + 4 specialist agents | APPROVE verdict from all four with confidence ≥ 0.85 | none |
| **T5** | Document | `cargo doc --no-deps --workspace --all-features` + vault MOC update (per [[progressive-disclosure-obsidian.md]]) | obsidian-vault-librarian + Command | docs at `target/doc/workflow_trace/` + vault TIER-1/2/3 bidi-clean with confidence ≥ 0.90 | none |
| **T6** | Deploy | `/usr/bin/cp -f target/release/wf-* ~/.local/bin/` then `wf-dispatch <id>` | Command + Luke @ terminal (Phase 5A) | binaries deployed; dispatch routes through Conductor | **m32 → Conductor only** — `wf-dispatch` NEVER executes a workflow directly (KEYWORDS_20 #6); Conductor `:8141` is the authoritative dispatch coordinator |

**Trap caught (per workspace CLAUDE.md anti-pattern table + project CLAUDE.md operational rule):** T6 binary placement uses `/usr/bin/cp -f` — never `cp -f` — to bypass shell alias to `trash` which would silently move the binary.

---

## NAM-03 confidence gates per stage

Per G5 § DevOps V3 integration NAM-03 line. Each T-stage emits a confidence value; V3 thresholds (sourced from V3 spec):

```text
T1 ≥ 0.80    Specify          if < → halt; spec revision required
T2 ≥ 0.80    Scaffold         if < → halt; /scaffold drift report; plan.toml or src/ amendment required
T3 ≥ 0.85    Implement        if < → halt; build failure; quality-gate stage 1 (cargo check) referenced
T4 ≥ 0.85    Harden           if < → halt; any of 4 agents (zen / security-auditor / silent-failure-hunter / performance-engineer) REJECT
T5 ≥ 0.90    Document         if < → halt; missing doc comments on public items; broken bidi anchors; vault sweep failure
                              (T5 is the strictest gate — documentation drift is a Class-D Watcher flag)
                              T6 inherits T5 confidence; if T5 = 0.91, T6 starts at 0.91; deploy gated on T6 own checks
```

**Halt semantics:** V3 halt = `POST :8082/deploy/<id>/cancel` → workflow-trace observes via m41 LCM router → m32 marks DispatchOutcome::ConfidenceGateFail → m40 emits NexusEvent (confidence_delta = -0.10 per V8 wire) → V8 lowers internal confidence → next T1 will face higher bar.

**Class-C Watcher flag:** confidence-gate refusal per KEYWORDS_20 #20 — this is the **safe path**, not a failure. Pre-positioned at every T-stage transition.

---

## resume_from contract (V8+Zen own T1; V3 owns T2-T6)

Per G4 V3 `resume_from` integration line + G5 § DevOps V3 integration resume_from line. The contract: when workflow-trace `wf-dispatch` invokes a deploy workflow on behalf of a user-authored proposal, T1 has already been authored elsewhere (typically V8 spec generation + Zen audit). m41 LCM router calls:

```text
JSON-RPC: lcm.loop.create
Body: {
  "spec_id": "<workflow_trace_proposal_id>",
  "max_iters": 1,
  "resume_from": "T2",                     // ← critical: skip T1 (already done by V8+Zen)
  "context": {
    "v8_confidence":     <inherited from V8 last update>,
    "zen_verdict":       "APPROVE" | "AMEND" | "REFUSE",
    "workflow_trace_id": "<m23 proposal_id>"
  }
}
```

LCM then dispatches V3 starting at T2; V3's `resume_from` field is honoured (per V3 spec; live since S1001882 V3 `b7d4abb`). If V3 refuses resume_from (e.g., `:8082` returns 4xx), m41 falls back to refuse-mode: ERROR log + typed `DispatchError::ResumeFromRejected`, NOT silent re-do (per project CLAUDE.md FP-verify discipline).

**Why workflow-trace doesn't author T1:** per G4 Axis 5 spec authority decision — `plan.toml` IS workflow-trace's spec authority; T1 in V3's nomenclature corresponds to a different domain (deploy spec generation). V8 + Zen own that.

---

## V8↔V3 wire reuse (the inherit-don't-reinvent contract)

Per G4 § V8 ↔ V3 wire reuse (the full diagram). Workflow-trace adds **exactly one** outbound call to this existing wire. The diagram:

```text
m32 DispatchOutcome::PassVerified
    │
    ▼
m40_synthex_emit (existing — NexusEvent emit to SYNTHEX v2 :8092)
    │
    ▼ (NEW — Phase 3 Track 5)
HTTP POST :8082/api/v8/learning
{
  "workflow_id": "<m32 dispatch_id>",
  "outcome": "PassVerified",
  "confidence_delta": +0.10,             // outcome-to-delta map below
  "source": "workflow-trace m32"
}
    │
    ▼
V3 routes to V8 internal confidence model (Hebbian-grain)
    │
    ▼ (existing wire — V8 → V3 already speaks)
V8 calls V3 POST :8082/api/v8/confidence  (cache update)
    │
    ▼
V3 future T1 Specify calls inherit updated V8 confidence
```

### Outcome → confidence_delta map (m32 → V8)

Per G4 wire reuse description:

| m32 DispatchOutcome | confidence_delta | rationale |
|---|---|---|
| `PassVerified` | **+0.10** | full 4-agent gate cleared; strongest signal |
| `Pass` | **+0.05** | dispatched and completed; lighter reinforce |
| `Blocked` | **−0.05** | refuse-mode (Conductor unreachable or AP30 violation); not a failure but no positive evidence |
| `Fail` | **−0.10** | dispatched and failed; full negative signal |
| `Refused` | **0.00** | m33 verifier refused dispatch; signal is for m33, not for V8 |
| `ConfidenceGateFail` | **−0.05** | NAM-03 halt during deploy; lighter penalty (was caught upstream) |

**Why only m32 writes this delta:** per AGENT_VIEW_GITWORKTREES.md per-layer ownership — L7 dispatch (m30-m33) is the **only** layer that emits to V8. Other layers' confidence-relevant signals flow into m31 selector (read-side); only m32 closes the loop.

---

## Bridge contract (FNV-1a single-user; HMAC-SHA256 multi-user)

Per V7 Phase 7 security runbook reference (per ULTRAMAP View 3 Phase 7) + KEYWORDS_20 #18 single-phase override.

### Phase 0 (current) through Phase 5C — single-user FNV-1a

Workflow-trace is a single-user system (Luke @ node 0.A) for the duration of Phase 5C soak (D30→D120). The m4 cascade correlator uses **FNV-1a XOR for opaque IDs** (per ULTRAMAP View 2 m4 row "F11 opaque IDs via FNV-1a XOR"). For V3 bridge: a per-dispatch FNV-1a hash of `(workflow_id + timestamp + secret)` serves as the integrity token attached to `POST :8082/api/v8/learning`. V3 verifies; if mismatch, returns 401 (bridge contract drift per G4 LCM Drift #6).

### Phase 6+ (D120 sunset evaluation onward) — multi-user HMAC-SHA256

If workflow-trace passes sunset and becomes multi-user, the FNV-1a single-user token becomes inadequate (collision space too small for adversarial use). Phase 7 security runbook mandates upgrade to **HMAC-SHA256** with per-consumer secret:

```text
Header: Authorization: HMAC-SHA256 <consumer_id>:<hmac_sha256(body + ts, consumer_secret)>
Body:   { workflow_id, outcome, confidence_delta, source, ts }
```

V3 verifies HMAC; tracks consumer_id; rejects on mismatch. Migration plan: dual-accept (FNV-1a AND HMAC) for 30 days; then FNV-1a-only-deprecated; then HMAC-only.

**Bridge contract verification:** `~/.local/bin/bridge-contract workflow-trace dev-ops-engine-v3` (per G4 § Wave-end orchestrator checklist step 7) — runs static grep of both sides for port + path + serde shape + auth scheme; reports drift without needing live services. Cited by [[atuin-integration.md]] § wt-bridge-check script (script also probes live `:8082/health` round-trip).

---

## Failure modes (≥3)

| ID | Failure | Detection | Mitigation |
|---|---|---|---|
| **V3-01** | T-stage confidence sub-threshold | V3 returns halt; m41 observes; m32 marks ConfidenceGateFail | Class-C Watcher flag fires (safe path); user re-issues with revised spec; m40 emits −0.05 delta to V8 |
| **V3-02** | V3 `:8082` unreachable (devenv stopped V3) | curl timeout on m40 outbound POST | outbox-first JSONL durable (per ULTRAMAP m40 row) — message persists; replay loop on reconnect; refuse-mode at m32 dispatcher (ERROR log + typed `DispatchError::DownstreamUnreachable`, NOT silent no-op) |
| **V3-03** | resume_from rejected by V3 (4xx) | LCM RPC response shape mismatch | m41 falls back to refuse-mode; DispatchError::ResumeFromRejected; Class-C Watcher flag; user manually re-runs at T1 |
| **V3-04** | V3 bind drift — `:8082` binds to 0.0.0.0 instead of 127.0.0.1 (LCM Drift #10) | `ss -tlnp 'sport = :8082'` shows 0.0.0.0 | per V3 `b7d4abb` (S1001882): V3 ServerConfig.bind_addr honoured; if regression detected, immediate `devenv restart dev-ops-engine-v3` and incident_war_room invocation |
| **V3-05** | HMAC secret leak (Phase 6+) | gitleaks pre-commit hook catches `HMAC_SECRET=` pattern | rotate secret via Luke @ terminal; per E1-E2 PAT rotation pattern (CLAUDE.local.md Open Escalations); update both ends atomically |
| **V3-06** | Wire-contract drift — V3 changes `/api/v8/learning` shape; m40 sends old shape silently | bridge-contract drift report at Wave-end + Phase 5C weekly | static grep both sides via `~/.local/bin/bridge-contract`; per Phase 4 hardening 4-agent ownership of bridges (security-auditor owns wire shape) |
| **V3-07** | NAM-03 confidence values mis-calibrated (workflow-trace emits +0.10 too generously and V8 over-confident as a result) | Phase 5C weekly Watcher independent recompute (per G4 Drift #7 transposition) | Watcher files Class-D flag; calibration audit; outcome-to-delta map amended via decision-record |

---

## Atuin trajectory

```bash
# Phase 3 Track 5 wire-up verification
atuin search "POST.*api/v8/learning" --before 1d           # confirms outbound calls landing
atuin search "lcm.loop.create" --before 7d | grep resume_from   # confirms resume_from honoured

# Phase 5A deploy verification
atuin search "wf-crystallise" --before 1d --cwd "$HOME/claude-code-workspace/workflow-trace"
atuin search "wf-dispatch"    --before 1d --cwd "$HOME/claude-code-workspace/workflow-trace"

# Phase 5C continuous
atuin search "wt-bridge-check" --before 7d --limit 50         # weekly bridge probes
atuin search "DispatchError"   --before 7d                    # refuse-mode invocations (Class-C signal)

# Bridge contract drift retrospective
atuin search "bridge-contract workflow-trace" --before 30d
```

---

## Verification commands

```bash
# Live V3 health (pre-wire-up confirmation)
curl -s -o /dev/null -w "%{http_code}\n" --max-time 1 http://localhost:8082/health   # expect 200

# Static bridge contract (no live services needed)
~/.local/bin/bridge-contract workflow-trace dev-ops-engine-v3

# Phase 3 Track 5 wire smoke (post-Wave-3, post-binary-deploy)
wf-dispatch --workflow-id "test_dispatch_001" --dry-run
# Expect: m32 emits, m40 outbox writes, POST :8082/api/v8/learning lands with 200, V8 confidence delta visible

# resume_from contract verification
curl -sS -X POST http://localhost:8082/deploy \
  -H 'content-type: application/json' \
  -d '{"spec_id":"test_resume_t2","resume_from":"T2"}' | jq '.honoured_resume_from'   # expect true

# Confidence-gate halt smoke
# (deliberately submit a spec V3 will fail at T3 implement; observe halt + workflow-trace observability)

# Atuin trajectory audit
atuin search "/api/v8/learning" --before 7d | wc -l        # should be > 0 once Phase 3 Track 5 wired
```

---

## Sign-off

✅ devops-v3-integration spec complete. T1-T6 mapped to workflow-trace verbs with explicit confidence thresholds. resume_from contract specified (m41 LCM router calls lcm.loop.create with resume_from: "T2"). V8↔V3 wire reuse documented (single outbound POST :8082/api/v8/learning from m40 on m32 PassVerified). Bridge contract specified for both single-user (FNV-1a) and multi-user (HMAC-SHA256) regimes. 7 failure modes (≥3 target met) with mitigations. Atuin trajectory + verification commands deterministic. NAM-03 thresholds (T1≥0.80, T2≥0.80, T3≥0.85, T4≥0.85, T5≥0.90) locked.

*Authored 2026-05-17 (S1001982) — Command for V7 G5 expansion. Phase 3 Track 5 wire-up post-G9. HOLD-v2 respected: planning-only.*
