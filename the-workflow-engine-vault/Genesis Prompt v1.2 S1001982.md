---
title: Genesis Prompt v1.2 — Zen-audit-locked S1001982
date: 2026-05-17 (S1001982)
kind: vault-mirror (canonical lives in ai_docs)
status: planning-only · binding spec until v1.3 patch lands · single-phase override needs v1.3 + Zen re-audit
---

# Genesis Prompt v1.2 S1001982

> Back to: [[HOME]] · [[MASTER_INDEX]] · [[workflow-engine-code-base]] · [[CLAUDE.md]] · [[CLAUDE.local.md]] · canonical: [[THE_WORKFLOW_ENGINE_GENESIS_PROMPT_V1_S1001982]]
>
> Related: [[Genesis Prompt v0 S1001982]] (5-voice co-authored predecessor) · [[Town Hall S1001982]] (15 P0 source) · [[Modules Synergy Clusters and Feature Verification S1001982]] (single-phase override → v1.3 patch pending)

## Summary

The genesis prompt v1.2 is the **binding spec** for the workflow-engine codebase. It is the recipe that, when issued to a fresh CC session **after all G1-G9 gates clear**, scaffolds the build.

History: v1.0 (initial) → v1.1 (Zen URGENT veto on active verbs absorbed) → v1.2 (Zen AMEND-THEN-FORWARD: ratification language gated, vault-first rubric, F2 hard-gate, G2 typo, Phase-B-reservation observability). v1.2 tail-clean per Zen PASS-FOR-FORWARD-AFTER-TAIL-CLEANUP.

**Single-phase override (Luke 2026-05-17) will require v1.3 patch** absorbing: 26 modules (was 11 in v1.2), no Phase B activation gate, explicit waiver record. v1.3 will need Zen re-audit (G7).

## Core invariant — Zen-audit-locked verbs (v1.2)

**Phase A RECORDS.**

Phase A does NOT: recommend · rewrite · route · package · dispatch · optimise · select · bank workflows · name patterns with human-meaningful labels · auto-* / smart-* anything.

Phase A DOES: read · correlate · record · emit reports · refuse to start at sunset.

**v1.3 patch will relax this** for the active-verb modules m20-m23, m30-m33, m40-m42 since single-phase requires them buildable at genesis — but the Zen-discipline of explicit verb mapping per module name carries forward.

## 9 pre-genesis gates (Zen-prescribed ordering)

| # | Gate | State |
|---|---|---|
| G1 | RATIFICATION (Watcher close-notice) | ⏸ |
| G2 | NAMING (directory rename) | ⏸ |
| G3 | :8125 REDEPLOY VERIFY | ⏸ |
| G4 | WATCHER NOTES (Hebbian v3 ✅ / Ember rubric ⚠ Held amendment pending) | partial |
| G5 | GENESIS INTERVIEW + F2 hard gate | ⏸ |
| G6 | DUAL-FRAME GAP ANALYSIS | ⏸ |
| G7 | ZEN SPEC AUDIT (APPROVE/REFUSE/AMEND) | ⏸ — will re-fire on v1.3 |
| G8 | FOUR-SURFACE PERSISTENCE | ⏸ |
| G9 | EXPLICIT START-CODING SIGNAL | ⚠ queued-intent-only (Zen blocked out-of-sequence observation) |

## Hard refusals (v1.2 list)

Forbidden verbs, nouns, surfaces (~30 items). Forbidden surfaces include: daemon, HTTP server, NexusEvent emission, LCM RPC, Conductor dispatch path, POVM writes, writes outside `workflow_trace_*` namespace, force-push, --no-verify, `use synthex_v2::*`.

**Single-phase override relaxes**: NexusEvent emission (now m40), LCM RPC (now m41), Conductor dispatch (now m32), workflow bank (now m30), selector (now m31), dispatcher (now m32). These move from forbidden-in-Phase-A to permitted-in-single-phase. **v1.3 patch documents this transition explicitly.**

## What stays forbidden even in single-phase

- POVM writes (deprecated → 2026-07-10)
- stcortex writes outside `workflow_trace_*` namespace (AP30 collision avoidance)
- force-push, --no-verify, --no-gpg-sign (CLAUDE.md hard rules)
- `use synthex_v2::*` (lift patterns, don't import)
- Self-modification of m46-m51 (AP27 hard boundary)

## Pre-build sequence (unchanged in single-phase)

1. Pain-source verification (Skeptic's gate) — **WAIVED by Luke 2026-05-17**
2. Structured genesis interview (G5)
3. Design doc + dual-frame gap analysis (G6)
4. 4-surface persistence (G8)
5. Eat the dogfood (engine's genesis follows the 6-step shape it encodes)
6. MVP cuts → each module gated by Zen audit during build

## Failure-mode table (F1-F11, all P0, all owned across module map)

See [[Modules Synergy Clusters and Feature Verification S1001982]] §"Flagged Features Verification Matrix" rows 1-11.

## Current relevance

v1.2 is the binding spec; v1.3 patch absorbing single-phase override is required before any G7 audit can re-fire. The 26-module architecture in [[Modules Synergy Clusters and Feature Verification S1001982]] IS the v1.3 module-list draft; v1.3 needs to explicitly waive the v1.2 invariant verb-restrictions for active-verb modules and document Luke's per-constraint waivers from the convergence's recommendations.
