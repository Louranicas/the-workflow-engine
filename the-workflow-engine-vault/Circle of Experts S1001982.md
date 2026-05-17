---
title: Circle of Experts — Workflow Engine Disputation S1001982
date: 2026-05-17 (S1001982)
kind: vault-mirror (canonical lives in ai_docs)
status: planning-only · superseded-in-shape (single-phase override) · structurally informs current spec
---

# Circle of Experts S1001982

> Back to: [[HOME]] · [[MASTER_INDEX]] · [[workflow-engine-code-base]] · canonical: [[THE_WORKFLOW_ENGINE_CIRCLE_OF_EXPERTS_S1001982]]
>
> Related: [[Town Hall S1001982]] (followed this) · [[Modules Synergy Clusters and Feature Verification S1001982]] (current architecture)

## Summary

8-persona disputation on whether to build `the-workflow-engine` and in what shape. Three syntheses tabled:

- **Synthesis A — Minimal Engine** (CLI + namespace + hooks; no sidecar service) — Command recommendation
- **Synthesis B — Conductor Wave-5 Module** (fold into HABITAT-CONDUCTOR)
- **Synthesis C — Engine As Originally Proposed** (full multi-service workflow engine) — Fossil predicted 80% mass loss in 12 months

## Circle (8 seats)

1. **The Architect** (Zen voice) — "right shape, not Airflow clone"; two-binary split + shared library
2. **The Substrate** (stcortex/POVM) — "narrowed consumer scope; bounded write rate"
3. **The Topologist** (Weaver/Conductor) — "Conductor-dispatch mandatory; never execute directly"
4. **The Skeptic** (Silent-Failure Hunter) — "habitat hasn't articulated this pain"
5. **The NA Gap Analyst** — "substrate-frame pass: labelling collapses ambiguity; gradient preservation required"
6. **The Fossil** (Code Writer ancestor) — "ambition-shaped codebases die; archaeological framing wins"
7. **The Operator** — "trap-surfacing + deviation-rationale first-class"
8. **The Watcher ☤** — recording, AP27 boundary-keeper

## Recommendation

Synthesis A with stacked constraints. 6-of-7 voting seats endorse.

## How it informs current single-phase architecture

- Architect's two-binary split → maintained (`wf-crystallise` + `wf-dispatch` + `workflow-core` lib)
- Substrate's narrowed scope → m2 + m9
- Topologist's mandatory Conductor dispatch → m32
- NA Gap's gradient preservation → m23 internal
- Fossil's archaeological framing → opaque cluster IDs in m4 + m23 + m30
- Operator's trap-surfacing + deviation → m32 + m23

## Current relevance

Synthesis A's *shape* is largely preserved in single-phase architecture. What changed (Luke 2026-05-17 override): the 120-day sunset-gate that protected against ancestor-rhyme risk is no longer a phase boundary. See [[Modules Synergy Clusters and Feature Verification S1001982]] §"What Luke's override waives" for explicit waiver record.
