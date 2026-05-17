---
title: m42 stcortex-only pivot — Architectural Decision Record
date: 2026-05-17 (S1001982)
kind: vault-mirror-tier-2 · ADR
canonical: ~/claude-code-workspace/the-workflow-engine/ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md
status: APPROVED · v1.3 amendment landed · awaiting Zen G7 verdict (AMEND-loop)
---

# m42 stcortex-only pivot — ADR

> **Back to:** [[HOME|V7 vault HOME]] · [[V7 Optimisation Framework]] · [[../HOME|project vault HOME]] · [[the-workflow-engine CLAUDE.local.md]]
>
> **Substrate notes (bidi — load-bearing for this decision):**
> - [[stcortex — Pioneer Capability Dossier 2026-05-10]] — the substrate workflow-trace now routes to exclusively
> - [[POVM Engine]] — the substrate workflow-trace DECOUPLED from pre-deploy
> - [[POVM V2 — Architecture Reference]] — historical context for what was being routed-to
>
> **Sibling vault notes:** [[Session S1001982 m42 pivot grilling]] (12-round interview record)
>
> **Canonical ADR:** `~/claude-code-workspace/the-workflow-engine/ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md`

---

## Decision

**Pivot m42 from POVM dual-path to [[stcortex — Pioneer Capability Dossier 2026-05-10|stcortex]]-only routing for substrate feedback, effective M0 from G9-fire.** Module renamed `src/m42_povm_dual/` → `src/m42_stcortex_emit/`. [[POVM Engine|POVM]] dependency removed pre-deployment.

## Trigger (the live-probe finding)

Live probe 2026-05-17 ~14:45Z:
- POVM `:8125/health` returned 200 + service:povm_v2 v2.0.0 ✅
- BUT `:8125/stats` showed `learning_health = 0.9146` (pre-CR-2 inflated; CR-2 expected reduction ~0.067)
- CR-2 + CR-2b merged at source (`e2a8ed3` + `76ea4d6`) but NOT in live binary
- [[stcortex — Pioneer Capability Dossier 2026-05-10|stcortex]] `:3000` operational (HTTP 200 ping; 5+ active consumers)

**Crystallised as antipattern AP-V7-13:** Health-200 ≠ behaviour-verified (sibling to AP-Hab-13 runbook probe freshness; different layer).

**Luke directive 2026-05-17 ~14:45Z:** *"D-B5 POVM :8125 restart does not need a restart it is currently operational IF not then use stCortex"* — surfaced the operational ambiguity and the IF-not fallback that became the strategic main path.

## Rationale (48-decision grilling summary)

Luke directed a 12-round AskUserQuestion deep grilling (48 sub-decisions). **48/48 Command recommendations accepted.** Full per-round transcript in canonical ADR + [[Session S1001982 m42 pivot grilling]].

12 rounds covered:
- R1 Pivot foundation · R2 Substrate-feedback semantic preservation · R3 Migration + risk · R4 stcortex consumer architecture · R5 Failure-mode posture · R6 Observability · R7 Test discipline · R8 Spec governance · R9 Watcher coverage · R10 Long-horizon · R11 Implementation sequencing · R12 Knowledge preservation

## Featureset preservation matrix (no loss)

| Featureset | Pre-pivot ([[POVM Engine|POVM]] dual-path) | Post-pivot ([[stcortex — Pioneer Capability Dossier 2026-05-10\|stcortex]]-only) |
|---|---|---|
| CC-5 substrate-learning loop | POVM + stcortex dual write | stcortex-only write |
| Fitness-delta constants | preserved | preserved (rebased on stcortex Hebbian-grain) |
| Outbox-first JSONL durability | preserved | preserved |
| Circuit-breaker pattern | shared 3 peers | shared 2 peers (POVM peer dropped) |
| m32 dispatch never blocks on substrate | preserved | preserved (outbox fallback) |
| Watcher Class-I monitoring | POVM learning_health | stcortex pathway.weight delta over rolling 7d |
| Hebbian-grain reinforcement | preserved | preserved (stcortex IS Hebbian-grain per [[stcortex — Pioneer Capability Dossier 2026-05-10\|stcortex dossier]] §"WAL pragmas + reducers") |

## Risk surface delta

**Eliminated:**
- F7 (CR-2 graceful-degrade pretend-fix at POVM) — workflow-trace no longer depends on POVM
- D-B5 Luke physical action — no POVM restart needed
- D25 mid-soak POVM cutover dance — no cutover; M0 ships stcortex-only
- POVM binary-version drift across services — workflow-trace decoupled
- 3rd circuit-breaker peer

**New (mitigated):**
- stcortex single-substrate degradation → outbox JSONL + offline snapshot fallback per workspace charter discipline
- stcortex schema bump → pinned version + bridge-contract per Wave-end + AP-Drift-05 audit
- stcortex EOL → re-evaluate per R10 trigger (currently no signal)

**Net: reduced.**

## V1.3 amendment scope (per Zen AMEND-loop)

v1.3 sections edited in-place:
- § 1 Cluster H description
- § 2 Hard refusals (POVM writes language)
- § 6 Axis matrix m42 row
- § 11 substrate-feedback list
- § 13 deferred-to-implementation
- § Appendix A (NEW — full amendment record)

V7 supporting material updated:
- KEYWORDS_20 #18: 'two-binary' → 'stcortex-only-m42'
- ANTIPATTERNS_REGISTER: AP-V7-13 added
- ULTRAMAP V2 m42 row + L8 description + flow diagram
- MODULE_PLANS/cluster-H.md title + module path

## Watcher pre-positioning at amendment landing

**Class-D** (four-surface drift coverage during sync) + **Class-A** (activation transition: amendment landed) fire at 2026-05-17 ~16:00Z. Watcher tick journal entry requested via WCP carriage. Watcher authority: advisory only (AP27 preserved; no veto on this pivot).

## Alternatives considered + rejected

| Alternative | Why rejected |
|---|---|
| Keep [[POVM Engine\|POVM]] dual-path | Requires POVM CR-2 binary deploy (D-B5 stays); higher D25 cutover complexity; F7 antipattern remains |
| Defer pivot decision | Violates no-deferrals directive |
| Whole Cluster H reviewed (m40 + m41 + m42) | Expands amendment scope; bigger Zen re-audit; not warranted (m41 + m40 unchanged per Q11.2 + Q11.3) |
| M2+ feature-gated povm-legacy | Adds dead code surface; opposite of single-substrate clarity (Q1.3 option c) |
| Add second substrate alongside stcortex | Violates G4 Axis 2 single-DB decision (Q3.3 option b) |
| Re-derive fitness-delta constants | Risks AP-WT-F1 if magnitudes mismatch; over-engineering (Q2.1 option b) |
| Drop outbox / drop breaker | Breaks cluster-H invariant; AP-Hab silent-failure surface (Q2.2 / Q2.3 option b) |

## Cross-references

- Canonical ADR: `~/claude-code-workspace/the-workflow-engine/ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md`
- v1.3 amendment: `~/claude-code-workspace/the-workflow-engine/ai_docs/GENESIS_PROMPT_V1_3.md` § Appendix A
- Session reflection: [[Session S1001982 m42 pivot grilling]]
- V7 canonical: [[V7 Optimisation Framework]]
- Project state: [[the-workflow-engine CLAUDE.local.md]]
- Substrate dossier: [[stcortex — Pioneer Capability Dossier 2026-05-10]] (the substrate routed-to)
- Decoupled substrate: [[POVM Engine]] (no longer a workflow-trace dependency)

---

*ADR vault mirror authored 2026-05-17. Canonical reference for any future Claude session evaluating "should we re-introduce POVM dependency in m42?" — answer: no, this ADR rejects it. Bidi-linked to [[stcortex — Pioneer Capability Dossier 2026-05-10]] + [[POVM Engine]] (decoupled marker) + project CLAUDE.local.md.*
