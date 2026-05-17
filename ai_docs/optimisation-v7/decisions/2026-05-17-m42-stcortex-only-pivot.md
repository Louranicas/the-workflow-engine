---
title: ADR — m42 stcortex-only pivot (POVM dual-path retired pre-deployment)
date: 2026-05-17 (S1001982)
kind: architectural-decision-record
authority: Luke @ node 0.A — 12-round AskUserQuestion grilling; 48/48 Command recommendations accepted
trigger: live-probe finding that POVM `:8125` is health-200 but serving pre-CR-2 binary (learning_health=0.9146 vs expected [0.05,0.15])
status: APPROVED; v1.3 amendment in progress per D-B6 AMEND-loop
supersedes: v1.3 § m42 (pre-amendment dual-path spec)
preserves: substrate-feedback featureset; CC-5 closure; outbox+breaker patterns; fitness-delta constants
---

# ADR — m42 stcortex-only pivot

> Back to: [[../OPTIMISATION_FRAMEWORK_V7_FINAL.md]] · [[../DECISION_REGISTER.md]] · [[../../../GENESIS_PROMPT_V1_3.md]]
>
> **Function:** captures the 48-decision grilling outcome that pivots m42 from POVM dual-path to stcortex-only. Every load-bearing alternative considered + rejected is documented here. Future Claude sessions resuming cold can read this ADR for the full reasoning without re-traversing 12 rounds.

---

## Context

**Pre-pivot state (v1.3 as authored 2026-05-17 ~04:30Z):** m42 was specified as POVM-dual-path with cutover ~D25 mid-soak (per workspace charter "POVM V1+V2 decommission target: 2026-07-10"). Required POVM `:8125` live + CR-2 binary deployed for G3 gate to pass. D-B5 was a Luke physical action to restart POVM.

**Live probe finding (2026-05-17 ~14:45Z):**
- POVM `:8125/health` returns 200 + service:povm_v2 v2.0.0 ✅
- `:8125/stats` shows `learning_health = 0.9146` (pre-CR-2 inflated value — CR-2 expected reduction to ~0.067)
- CR-2 + CR-2b merged to source at `e2a8ed3` + `76ea4d6` but NOT deployed in live binary
- stcortex `:3000` operational (HTTP 200 ping; 5+ active consumers: stcortex-subscriber-rust + save-session + 3× cc-* MCP)

**Luke directive 2026-05-17 ~14:45Z:** *"D-B5 POVM :8125 restart does not need a restart it is currently operational IF not then use stCortex"* — surfaced the operational ambiguity (POVM up but stale) and the IF-not fallback.

## Decision

**Pivot m42 from POVM dual-path to stcortex-only routing for substrate feedback, effective M0 from G9-fire.** Apply via D-B6 AMEND-loop (Command edits v1.3 § m42 + appendix amendment note; Zen re-fires G7 audit; AMEND-loop precedent applies; no Luke waiver needed).

## Rationale (48-decision summary)

### Pivot foundation (R1)
- Pivot YES — eliminates D-B5 + D25 cutover + F7 antipattern; aligns with workspace POVM-decommission charter
- Scope: m42 only (m40 SYNTHEX-emit + m41 LCM-router unchanged)
- Effective: M0 from G9-fire (no migration; workflow-trace has zero deployed pathways)
- Authority: Command autonomous via D-B6 AMEND-loop

### Substrate-feedback semantic preservation (R2)
- Fitness-delta constants preserved (PassVerified +0.25 / Pass +0.15 / Blocked -0.05 / Fail -0.10) — Hebbian-grain rebased on stcortex pathway.weight semantics
- Outbox-first JSONL durability preserved — substrate down NEVER blocks dispatch
- Circuit-breaker pattern preserved — shared `m40_42_common::Breaker`; now 2 peers instead of 3 (synthex-v2 + stcortex; POVM peer dropped)
- CC-5 closure-test preserved — assertion changes from "POVM learning_health moved" to "stcortex pathway.weight delta observable"

### Migration + risk (R3)
- POVM hard-cut: no read + no write from M0 (Luke's "change POVM to stcortex" answer)
- No migration: workflow-trace starts fresh on stcortex (workflow_trace_* namespace begins empty per AP30)
- Single-substrate risk acknowledged with offline JSONL snapshot fallback (per workspace charter discipline; no silent POVM fallback)
- POVM EOL revisit: 2026-07-10 hard per workspace charter

### stcortex consumer architecture (R4)
- Two consumers: wf-crystallise + wf-dispatch (one per binary)
- Single flat namespace: `workflow_trace_*` (no per-cluster sub-namespaces; minimal m9 validation surface)
- Refuse-write strict per stcortex DB-layer (POVM-cure discipline preserved)
- Pathway naming: `workflow_trace_<workflow_id>_<outcome>` underscored (AP-Hab-11 hyphen-slug discipline)

### Failure-mode posture (R5)
- stcortex degraded: outbox-buffer + retry; m32 dispatch continues (cluster-H invariant)
- stcortex unreachable at build time: build succeeds; runtime uses offline JSONL snapshot
- Schema migration: pin to stcortex current schema; explicit migration step on bump (AP-Drift-05 catch)
- m42 health surface: atuin-only (CLI-first decision G4 Axis 3 holds)

### Observability (R6)
- Telemetry: full per-write success/fail + latency histogram + outbox queue depth
- Watcher Class flag: extend Class-I (Hebbian silence) to cover stcortex (no new class; taxonomy stays at 9 A-I)
- Pushgateway: `wf_m42_stcortex_writes_total` per Phase 8 Track 2
- Phase 5C soak metric: stcortex pathway.weight delta over rolling 7d window (replaces POVM learning_health as primary)

### Test discipline (R7)
- Test budget: preserve 60 tests; reallocate dual-path tests to stcortex edge cases (schema-pin + offline-snapshot + outbox replay)
- Integration test: stcortex :3000 required; `#[ignore = "requires stcortex"]` if absent (per V7 G6 matrix)
- Mutation kill rate: preserve 80% per G6 (substrate-coherence still critical)
- Property tests: add 5 (write-then-read identity / AP30 prefix / hyphen-slug / fitness-delta bounds / outbox replay idempotency)

### Spec governance (R8)
- Amendment path: in-place edit of v1.3 § m42 + dated amendment note (D-B6 AMEND-loop)
- Re-audit timing: immediate (today)
- Audit scope: amendment-only delta + cluster-H integration
- Documentation: both inline v1.3 + this ADR

### Watcher coverage (R9)
- Class-I scope: stcortex pathway.weight delta rolling 7d; trigger if delta=0 for 4+ weeks
- Pivot itself: Class-D (four-surface drift) + Class-A (activation transition) at amendment-landing timestamp
- Pre-positioning: Phase 5C tick journal entry per stcortex health probe
- Authority: advisory (Watcher observes + flags; no veto; AP27 preserved)

### Long-horizon (R10)
- Post-2026-07-10: stcortex-only forever; revisit only on stcortex EOL signal
- API contract: pin to stcortex schema version; `bridge-contract` skill per Wave-end (AP-Drift-06)
- Second substrate: NO feature gate (G4 Axis 2 single-DB stands; future v2.0 redesign if ever)
- m40 SYNTHEX-emit: unchanged (SYNTHEX v2 + stcortex serve different concerns — coordination vs memory)

### Implementation sequencing (R11)
- Wave-3 fit: same Day 17-21 slot; reduced complexity = same/faster
- m41 LCM-router: unchanged
- m40 SYNTHEX-emit: unchanged
- Module rename: `src/m42_povm_dual/` → `src/m42_stcortex_emit/` (theme name reflects content)

### Knowledge preservation (R12)
- Keyword cap: replace #18 'two-binary' with 'stcortex-only-m42' (cap stays at 20)
- New antipattern AP-V7-13: 'Health-200 ≠ behaviour-verified' (lesson from POVM-up-but-stale-binary)
- Propagation: targeted set (v1.3 + cluster-H + ULTRAMAP V2 m42 row + KEYWORDS_20 #18 + this ADR)
- Session reflection: filed at `~/projects/claude_code/Sessions/Session S1001982 m42 pivot grilling.md`

## Featureset preservation matrix

| Featureset | Pre-pivot | Post-pivot |
|---|---|---|
| Substrate-feedback loop (CC-5) | POVM + stcortex dual write | stcortex-only write |
| Fitness-delta constants | preserved | preserved (rebased semantic) |
| Outbox-first JSONL | preserved | preserved |
| Circuit-breaker pattern | shared 3 peers | shared 2 peers (POVM dropped) |
| m32 dispatch never blocks on substrate | preserved | preserved (outbox fallback) |
| Watcher Class-I monitoring | POVM learning_health | stcortex pathway.weight delta |
| Substrate-condition acceptance (D-Substrate) | POVM LTP/LTD-degraded | stcortex (substrate condition unchanged at habitat-wide level) |
| Hebbian-grain reinforcement | preserved | preserved (stcortex IS Hebbian-grain) |

**No featureset loss.** Substrate-feedback semantic preserved 1:1 via stcortex. Dual-path overlap-window-safety-net dropped (the lost item) — mitigated by offline JSONL snapshot fallback discipline per workspace charter.

## Risk surface delta

**Eliminated risks:**
- F7 (CR-2 graceful-degrade pretend-fix at POVM) — no longer relevant; we don't depend on POVM
- D-B5 Luke physical action — no POVM restart needed
- D25 mid-soak POVM cutover dance — no cutover; M0 ships stcortex-only
- POVM binary-version drift across services — workflow-trace decoupled

**New risks (mitigated):**
- stcortex single-substrate degradation → mitigated by offline JSONL snapshot + outbox retry
- stcortex schema bump → mitigated by pinned version + bridge-contract per Wave-end + AP-Drift-05 audit
- stcortex EOL → re-evaluate per R10 trigger (currently no signal)

**Net risk surface: reduced.**

## Alternatives considered + rejected

| Alternative | Why rejected |
|---|---|
| Keep POVM dual-path (Q1.1 option b) | Requires POVM CR-2 binary deploy (D-B5 stays); higher complexity at D25 cutover; F7 antipattern remains |
| Defer pivot decision (Q1.1 option c) | Reintroduces a deferred decision (violates no-deferrals directive) |
| Whole Cluster H reviewed (Q1.2 option b) | Expands amendment scope; bigger Zen re-audit; not warranted |
| M0 dual-path + auto-cutover at D25 (Q1.3 option b) | Preserves original spec but doesn't address the underlying CR-2 binary mismatch |
| M2+ feature-gated povm-legacy (Q1.3 option c) | Adds dead code surface; opposite of single-substrate clarity |
| Re-open G5 spec interview (Q1.4 option b) | Blocks gate progress; arguably overkill for single-module decision |
| Re-derive fitness-delta constants (Q2.1 option b) | Risks AP-WT-F1 if magnitudes mismatch; over-engineering |
| Drop outbox / drop breaker (Q2.2 / Q2.3 option b) | Breaks cluster-H invariant; AP-Hab silent-failure surface |
| Add second substrate (Q3.3 option b) | Violates G4 Axis 2 single-DB decision |
| Bust 20-keyword cap (Q12.1 option c) | Violates KEYWORDS_20 § quick-use guide |

## Implementation actions (executing now)

1. ✅ This ADR authored
2. → v1.3 § m42 amended in-place + appendix amendment note appended
3. → cluster-H.md updated (m42 row + spec)
4. → ULTRAMAP V2 m42 row updated (rename + paths + LOC)
5. → KEYWORDS_20 #18 replaced
6. → ANTIPATTERNS_REGISTER AP-V7-13 added
7. → DECISION_REGISTER appended with 48-decision grilling outcome
8. → LUKE_ACTION_NEEDED escalation amended (drop action 1 / D-B5; 3 actions remain ~10min)
9. → Zen AUDIT-REQUEST v2 filed (amendment delta + cluster-H integration)
10. → DECISIONS_LANDED v2 peer notice filed
11. → Session reflection note filed at `~/projects/claude_code/Sessions/...`

## Sign-off

**Decision authority:** Luke @ node 0.A via "always make best recommendation" filter applied across 48 sub-decisions in 12 rounds; 48/48 Command recommendations accepted.

**Substrate-condition statement:** workflow-trace ships m42 stcortex-only from M0 with offline JSONL snapshot discipline. POVM is not a dependency. Substrate-feedback featureset preserved. Risk surface reduced.

*ADR authored 2026-05-17 by Command. Canonical reference for any future Claude session evaluating "should we re-introduce POVM dependency in m42?" — answer: no, this ADR rejects it.*
