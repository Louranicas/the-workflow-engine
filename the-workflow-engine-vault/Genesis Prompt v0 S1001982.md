---
title: Genesis Prompt v0 — 5-Voice Co-Authored S1001982
date: 2026-05-17 (S1001982 / S1001971)
kind: vault-mirror (canonical lives in working dir)
status: planning-only · DRAFT-V1 · pre-build · superseded as binding spec by v1.2 but module-architecture content remains load-bearing
---

# Genesis Prompt v0 S1001982

> Back to: [[HOME]] · [[MASTER_INDEX]] · [[workflow-engine-code-base]] · canonical: [[GENESIS_PROMPT_V0]]
>
> Related: [[Convergence Command x Command-3 S1001982]] (informed authorship) · [[Genesis Prompt v1.2 S1001982]] (Zen-audit-locked successor; current binding spec) · [[Interview Question Bank Draft S1001982]] (G5 interview content) · [[Modules Synergy Clusters and Feature Verification S1001982]] (module-count reconciliation needed: v0=28, v1.2=11, current=26)

## Summary

**5-voice co-authored** prompt (Command + Command-2 + Command-3 + Watcher ☤ + Zen) over ~3 hours wall clock + 30 min focused final deliberation. Encodes the 15 P0 town-hall constraints + 9 Command-3 recon convergence findings + Luke's directive on cascading/battern + context-window sweet-spot.

**Distinct from v1.2** (the Command-only Zen-audit-locked version that is the current binding spec). v0 is more comprehensive but was superseded by v1.2 because Zen's audit-lane discipline applied verb-locking + gate-ordering precision that v0 didn't have.

## Mission (per v0)

Build Rust microservice `workflow-trace`:
- **Observes** cascading commands across the zellij habitat
- **Observes** Battern protocol runs as coherent lifecycle records
- **Observes** every receiving pane's context-window state at payload arrival, joined into cascade record
- **Surfaces** compositional patterns *that already exist in the substrate* (archaeological, not authorial)
- **Recommends** cascade payload compactions for context-window sweet-spot (`ideal < 100k tokens`, `ok < 300k`, `degraded < 800k`, `critical >= 800k`)
- **Accelerates** hand-driven cascades by pre-emptive right-sizing of handoff payloads

> Note: words "recommends" and "accelerates" in v0 are exactly the active verbs Zen's URGENT veto (`2026-05-16T224304Z`) caught in v1.0 review. v0 was authored BEFORE Zen's veto landed; v1.2 absorbs the veto and reframes these as Phase B activities post-gate.

## Module map (v0 — DIFFERENT from v1.2 + single-phase)

v0 has **28 modules across 8 layers**, ~5,600 LOC total, ~62% boilerplate yield. v0 uses **2-digit zero-padded naming** (m01, m02, ...) — DIFFERENT from my v1.2/structure-doc 1-digit naming (m1, m2, ...).

| Layer | LOC | Module count |
|---|---:|---:|
| L1 — Foundation | ~700 | m01-m04 (4) |
| L2 — Substrate Read (Phase A) | ~800 | m05-m08 (4) |
| L3 — Cascade & Context Observation (Phase A) | ~1,250 | (5-6 modules) |
| L4 — Crystallisation | (varies) | |
| L5 — Selection | | |
| L6 — Dispatch | | |
| L7 — Bridges | | |
| L8 — CLI / surface | | |
| **Total** | **~5,600** | **28** |

## Substrate Invariants (Watcher P0, non-negotiable)

1. Observation ≠ curation (R13 quiet 30d/100 obs gating)
2. Measurement non-interference (60s blackout window post-dispatch)
3. Substrate-trust scope (`consumer_scope: [tool_call, consumption]` only)
4. Selector self-training prohibited (hand-driven runs only)
5. No POVM writes (deprecated)
6. Gradient preservation N=3 near-miss variants alongside canonical

## Quality Gate (Zen P0)

1. PIPESTATUS-correct 4-stage gate (no `tail`-swallows-exit)
2. 50+ tests per module · **80+ for m20_path_selector** (lock-in-vulnerable)
3. Receipt-DAG observation persistence
4. Single canonical `PaneContextSnapshot` shape; no silent coercion
5. No silent failures (no `unwrap_or(true)`, no `.ok()` on Result, etc.)
6. Empirical quality claims required (`{ n_samples, mean_quality, variance, last_used, source_cascade_ids }`)
7. Doc comments on all public items
8. Genesis-eats-dogfood: workflow-trace's genesis follows its own 6-step Battern shape

## ⚠️ Module-count inconsistency across artefacts

| Artefact | Module count | Naming convention | Status |
|---|---|---|---|
| Genesis Prompt v0 (this) | **28** modules / 8 layers | m01_types, m02_errors... | DRAFT, superseded as spec by v1.2 |
| Genesis Prompt v1.2 (Zen-audit-locked) | **11** modules | m1-m11 | **Binding spec** (per Zen) |
| Module Structure (3-phase) | 25 modules | m1-m42 sparse | Architecture sketch |
| Modules Synergy Clusters (single-phase, after Luke override) | **26** modules | m1-m42 sparse + m33 added | Current under-construction shape |

**Reconciliation needed at G5 spec interview.** This is a known open issue logged in [[workflow-engine-code-base]] tracker.

## Current relevance

v0's substrate invariants + quality gate carry forward intact. v0's module map structure (L1-L8 layered) provided the template my Module Structure doc followed (renamed L1-L9). v0's active-verb framing (recommends/accelerates) was caught by Zen and superseded in v1.2; under Luke's single-phase override those verbs return as Phase B modules m20-m23, m30-m33, m40-m42.
