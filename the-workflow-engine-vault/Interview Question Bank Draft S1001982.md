---
title: Genesis Interview Question Bank (DRAFT) S1001982
date: 2026-05-17 (S1001982)
kind: vault-mirror (canonical lives in working dir)
status: planning-only · DRAFT pre-staged · awaiting Action 1 (pain-source verify) + Action 2 (CR-2 ruling) before run
---

# Interview Question Bank S1001982

> Back to: [[HOME]] · [[MASTER_INDEX]] · [[workflow-engine-code-base]] · canonical: [[INTERVIEW_QUESTION_BANK_DRAFT]]
>
> Related: [[Town Hall S1001982]] (interview is G5 content from town-hall pre-build sequence) · [[Genesis Prompt v0 S1001982]] (5-voice draft references this bank) · [[Modules Synergy Clusters and Feature Verification S1001982]] (single-phase override altered Q1.3 + Q2.4 semantics — v0.1 patch flagged)

## Summary

Pre-staged 12-question genesis interview (3 rounds × 4 questions; AskUserQuestion-compatible). Authored by Command + Command-3 per S117 crystallisation pattern (`feedback_structured_interview_before_code.md`). Runs only when Luke green-lights Action 1 (Skeptic pain-source verification, ≥3 hits criterion) + Action 2 (CR-2 ruling — now SHIPPED).

This is the **G5 gate's content** in the pre-genesis sequence.

## Round 1 — Foundation (load-bearing structural)

### Q1.1 Naming + framing
- `workflow-cartographer` (recommended; archaeological per Fossil)
- `workflow-librarian` (Command-3 framing)
- `workflow-archaeologist` (Fossil first-alternative)
- `the-workflow-engine` (original; control-plane framing)

### Q1.2 Binary topology
- Two binaries + library (recommended; Architect position)
- One binary, internal modules
- One binary now, split later

### Q1.3 Conductor integration timing
- Wait (recommended; town-hall consensus; P0 #15)
- Plan + scaffold; defer wiring (`CONDUCTOR_ENFORCEMENT_ENABLED` startup-refusal)
- Bypass Conductor (NOT recommended; violates P0 #3)

### Q1.4 Decay law shape
- `frequency × fitness × recency × co-activation` (recommended; integrates RALPH P0 #8 + Command-3 Wave 2 F3)
- `frequency × fitness × recency` (RALPH town-hall position)
- `frequency × recency` only (Command-3 Wave 2 default)
- `co-activation × recency` only (Hebbian-only)

## Round 2 — Substrate boundaries (read/write scope, trust profile)

### Q2.1 stcortex consumer scope
- `tool_call` + `consumption` only (recommended; Substrate P0 #2 narrowest)
- Add `memory` read-only (cross-domain pattern detection; enormous read amplification)
- Add `pathway` read-only (Hebbian-aware crystallisation; more surveillance surface)

### Q2.2 Write surface boundary
- Own `workflow_state.db` + own stcortex `workflow_engine_*` namespace (recommended; town-hall consensus)
- Above + atuin KV (`workflow.last_state.*` keys)
- Above + injection.db `causal_chain` table

### Q2.3 Stale-workflow gate threshold (Operator P0 #7)
- Substrate evidence decayed below threshold in last 60 days (recommended; concrete, time-bounded)
- N consecutive failed runs (e.g., 3)
- Manual `workflow refresh <name>` only
- Adaptive: function of decay law output

### Q2.4 Sunset clause trigger
- ALL of: selector accuracy <60% AND no Hebbian gain vs baseline AND Conductor-violation >5% (recommended; three-axis AND)
- ANY of the three (stricter)
- Selector accuracy alone
- Hebbian gain alone

## Round 3 — Crystalliser internals + first-workflow choice

(8 more questions per canonical doc — Q3.1 sub-graph algorithm, Q3.2 R13 quiet period, Q3.3 first workflow, Q3.4 build sequence)

## Current relevance

Interview is the G5 gate's content. **G5 is currently ⏸ NOT GREEN.** Interview runs synchronously with Watcher + Zen in their respective participation/audit slots. Output of interview informs G6 (dual-frame gap analysis) and G7 (Zen spec audit).

**Luke's single-phase override (2026-05-17)** may simplify some interview questions (e.g., Q1.3 Conductor posture and Q2.4 sunset are now altered semantics). Interview question bank may need v0.1 patch before G5 fires — flagged for review.
