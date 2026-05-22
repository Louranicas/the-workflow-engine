> Back to: [[CLAUDE.md]] · [[CLAUDE.local.md]] · canonical [`ai_docs/WORKFLOW_TRACE_COMPLETION_PLAN_V2_S1004115.md`](../ai_docs/WORKFLOW_TRACE_COMPLETION_PLAN_V2_S1004115.md)

# Workflow-Trace Completion Plan v2 — S1004115

**Vault mirror** (condensed) of the canonical completion plan. Authored 2026-05-23 (S1004115),
re-baselined on git HEAD `6c3a5c5`. Status: **PLAN v2 — 48 decisions LOCKED (Phase 4 interview
complete, S1004115), awaiting node-0.A go for Phase 1**.

## What this plan is

Closes every outstanding recommendation / residual / deferred item for `the-workflow-engine`
and reaches a clean **v0.1.0 / M0** tag. The 26-module codebase is already implemented and
hardened (1967 tests, clippy + pedantic clean, 96.3 % mutation kill-rate); what remains is
honest residuals, decision-gated wiring, audit fold-ins, doc debt, and operator hand-offs.

## What `v0.1.0` certifies

`v0.1.0` / M0 certifies **engine-internal completeness** — every residual the engine owns is
closed, tested, audited, documented. It is **not** a substrate-safety milestone; the engine's
safety as a substrate-facing organ (substrate-drift detection, substrate fixtures,
substrate-mediated trust) is **v0.2.0**.

## Dual-frame discipline

- **Part A — conventional plan:** 10 phases (re-baseline → verify → cleanup → interview →
  R2 → R1 → CC-7 → integration → Zen fold-in → M0 ship).
- **Part B — substrate-frame pass:** a genuine second authoring (atuin / stcortex /
  injection.db / Conductor / Luke as actors, not sinks) **with a frame-collapse self-check**.
- v1's §8 was a Frame-A self-audit mislabelled as the substrate frame; v2 corrects this.

## v1 → v2 (gap-analysis correction)

v2 supersedes [[Workflow-Trace Completion Plan S1004115]] (v1). v1 was authored against a
stale HEAD, mis-scoped an interview question on already-shipped plumbing, rested a verifier
on a non-existent field, and treated absent Zen verdicts as merely slow. The dual-frame gap
analysis ([conventional](../ai_docs/WORKFLOW_TRACE_COMPLETION_PLAN_S1004115_CONVENTIONAL_GAP_ANALYSIS.md)
3 HIGH / 6 MED / 3 LOW · [NA](../ai_docs/WORKFLOW_TRACE_COMPLETION_PLAN_S1004115_NA_GAP_ANALYSIS.md)
9 frame gaps / 3 tensions) found 24 items — v2 accepts all 24 (disposition table § 12),
zero rejected.

## The 10 phases

| Phase | What | Gate |
|-------|------|------|
| 1 | Re-baseline + verify & reconcile (CHANGELOG 94.4→96.3 %, dirty tree, residual list) | decision-free |
| 2 | Deep FP-verification (wire contract, 8 absorbed NA gaps' *code* status) — feeds interview | decision-free |
| 3 | Low-risk code cleanup (MUT-2, T4 batch) | decision-free |
| 4 | Decision interview — Round A (3 no-default) + Round B (7 mechanical) | node 0.A |
| 5 | R2 — m22 K-means diversity wired into `wf-crystallise` + cluster emission | needs DB4 |
| 6 | R1 — m33 verifier policy logic, split 6a–6f (Security / Ember / Cost / Consistency / m9 seam / verdict receipts) | needs DA2, DB1–DB3 |
| 7 | CC-7 resolution — wire `PressureEvent` → m23, or document observability-only | needs DA1 |
| 8 | Integration & end-to-end (env matrix, substrate-load observation, clock enumeration) | — |
| 9 | Zen audit fold-in + SD1–SD12 reconciliation (absent-vs-slow escalation) | Zen-paced |
| 10 | M0 / v0.1.0 ship — tag, four-surface persist, push | — |

**Effort:** ~9–18 Claude-days to M0 (range driven by interview outcomes — DB2 cost-wire vs
stub, DA2 Consistency-now vs defer). v0.2.0 (NA-GAP-07/08/10) is a separate ~1,200-LOC
milestone.

## Decisions — LOCKED (Phase 4 interview, S1004115)

The Phase 4 interview ran as a 12-round / 48-question grilling — all 48 decisions locked
(47 on recommendation, 1 deviation). Full answer key: canonical plan **§ 15**. Headlines:

- **M0** = engine-internal completeness only; substrate-safety = v0.2.0 (a real committed milestone).
- **m33 gate** = blocking, fail-safe — Security (hard-Refuse) + Ember (Amend) real; Cost +
  Consistency documented stubs; every verdict emits a WireEvent.
- **R2 m22** fully real — 5-dim features, adaptive k, influences m31 selection, cluster emitted.
- **CC-7 wired** — `PressureEvent` modulates m23 compose-priority.
- **Audit** — if Zen stays silent, the in-session `zen` agent substitutes (not /ultrareview).
- **Release** — CI ships with M0; clean `v0.1.0` tag; rename post-M0.
- **Execution** — Claude solo, sequential, full re-verify every phase; ~10–13 Claude-days.
- **The single remaining gate:** node 0.A's explicit go for Phase 1.

## Anchors — four surfaces

- **ai_docs (canonical):** [`WORKFLOW_TRACE_COMPLETION_PLAN_V2_S1004115.md`](../ai_docs/WORKFLOW_TRACE_COMPLETION_PLAN_V2_S1004115.md)
  + the two gap-analysis companions.
- **Obsidian vault:** this note.
- **stcortex:** namespace `workflow_trace_completion_s1004115` — meta memory + bidi pathway
  ↔ `workflow_trace_hardening_2026_05_21`.
- **CLAUDE.local.md:** project [`CLAUDE.local.md`](../CLAUDE.local.md) § "Completion Plan v2".

---
*Vault mirror · S1004115 · 2026-05-23 · condensed from the canonical ai_docs plan.*
