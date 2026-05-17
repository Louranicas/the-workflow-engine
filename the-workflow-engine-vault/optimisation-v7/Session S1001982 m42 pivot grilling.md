---
title: Session S1001982 — m42 pivot 12-round grilling crystallisation
date: 2026-05-17 (S1001982)
kind: vault-mirror-tier-2 · session-reflection
canonical: ~/projects/claude_code/Sessions/Session S1001982 m42 pivot grilling.md
status: pattern crystallisation — 12-round substrate-pivot interview template
---

# Session S1001982 — m42 pivot grilling

> **Back to:** [[HOME|V7 vault HOME]] · [[V7 Optimisation Framework]] · [[m42 stcortex-only pivot ADR]] · [[the-workflow-engine CLAUDE.local.md]]
>
> **Substrate notes (bidi):** [[stcortex — Pioneer Capability Dossier 2026-05-10]] · [[POVM Engine]]
>
> **Main vault sibling:** `~/projects/claude_code/Sessions/Session S1001982 m42 pivot grilling.md` (canonical)

---

## What happened

In the second half of S1001982 (~14:00-16:00 local 2026-05-17), Luke directed a deep architectural pivot via 12-round AskUserQuestion grilling on the [[m42 stcortex-only pivot ADR|m42 routing decision]]. **48 sub-decisions made; 48/48 Command recommendations accepted.**

## The 12-round structure (reusable template)

| Round | Theme | Hit-rate |
|---|---|---|
| R1 | Pivot foundation (YES/scope/date/authority) | 4/4 |
| R2 | Substrate-feedback semantic preservation | 4/4 |
| R3 | Migration + cutover + risk | 4/4 |
| R4 | [[stcortex — Pioneer Capability Dossier 2026-05-10\|stcortex]] consumer architecture | 4/4 |
| R5 | Failure-mode posture | 4/4 |
| R6 | Observability | 4/4 |
| R7 | Test discipline | 4/4 |
| R8 | Spec governance | 4/4 |
| R9 | Watcher coverage | 4/4 |
| R10 | Long-horizon | 4/4 |
| R11 | Implementation sequencing | 4/4 |
| R12 | Knowledge preservation | 4/4 |

**Total: 48/48 hit rate.**

## What made the hit rate work

1. **Clear filter** — Luke's prior directive "preserve capacity + featureset" made the "Recommended" option in each question the substrate-preserving + risk-reducing choice.
2. **Genuine alternatives** — options (b) and (c) per Q were real considerations with documented rationale, not strawmen.
3. **Citations linked back to V7 corpus** — every option referenced existing G2 / G3 / G4 / G6 / ULTRAMAP / KEYWORDS_20 / ANTIPATTERNS_REGISTER content.
4. **Tight scoping** — Q1.2 explicitly bounded scope to m42-only; subsequent rounds inherited.
5. **Recommendation-first** — alternatives served as falsifiability checks.

## Reusable doctrine: substrate-pivot interview template

For any future "swap substrate X for Y" decision (SQLite → DuckDB / HTTP → gRPC / monolith → split / etc.), the 12-round template:

1. **Foundation** — pivot YES/NO; scope; effective date; authority
2. **Semantic preservation** — what does the dependent featureset need from the new substrate?
3. **Migration + risk** — overlap-window? data migration? risk acceptance?
4. **Consumer architecture** — registration grain; namespace; refuse-write; naming convention
5. **Failure-mode posture** — degraded? unreachable? schema bump? health surface?
6. **Observability** — telemetry; flag class; metrics endpoint; soak metric
7. **Test discipline** — budget; integration live-services; mutation kill; property tests
8. **Spec governance** — amendment path; audit timing; audit scope; ADR
9. **Watcher coverage** — class scope; flag at landing; pre-positioning; authority
10. **Long-horizon** — post-EOL posture; API contract stability; optionality
11. **Implementation sequencing** — wave fit; sibling-module impact; rename
12. **Knowledge preservation** — keyword; antipattern; cross-doc propagation; reflection note

## Validated against MEMORY rule

Per `feedback_structured_interview_before_code.md` MEMORY entry: "For cross-repo or any task with ≥5 architectural decisions, run an N-round AskUserQuestion interview FIRST." This pivot had 48 architectural sub-decisions. The 12-round interview was correctly shaped.

## What was crystallised

- **AP-V7-13** in ANTIPATTERNS_REGISTER: "Health-200 ≠ behaviour-verified"
- **KEYWORDS_20 #18** replaced: 'two-binary' → 'stcortex-only-m42'
- **Substrate-pivot doctrine** (this 12-round template) — first canonical authority
- **D-B6 AMEND-loop precedent** validated end-to-end (Command edits in-place + Zen re-audits delta; no impasse)

## Cross-references

- ADR: [[m42 stcortex-only pivot ADR]]
- V7 framework: [[V7 Optimisation Framework]]
- Project state: [[the-workflow-engine CLAUDE.local.md]]
- Substrate (routed-to): [[stcortex — Pioneer Capability Dossier 2026-05-10]]
- Substrate (decoupled): [[POVM Engine]]
- Canonical session reflection: `~/projects/claude_code/Sessions/Session S1001982 m42 pivot grilling.md`

---

*Vault mirror authored 2026-05-17. The 48/48 hit rate validates that featureset-preserving recommendations align with operator intent when the filter is explicit. Pattern reusable for future substrate-pivots.*
