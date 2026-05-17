---
title: workflow-engine-code-base — Workflow Tracker
date_opened: 2026-05-17 (S1001982)
last_updated: 2026-05-17 ~10:55
kind: workflow-tracker
status: planning-pilot · HOLD-v2 active on build
authority: Luke @ node 0.A
---

# workflow-engine-code-base — Workflow Tracker

> Back to: [[HOME]] · [[MASTER_INDEX]] · [[CLAUDE.md]] · [[CLAUDE.local.md]]

**Purpose:** chronological audit + decision log + open-issue tracker + team-contribution map for the entire workflow-engine planning pilot. Updated forward as work progresses.

---

## 1. Phases of the planning pilot

| Phase | Date / Session | Output | Status |
|---|---|---|---|
| **P1** Circle of Experts | 2026-05-17 ~07:41 | [[Circle of Experts S1001982]] — 8-persona disputation, Synthesis A recommended | ✅ COMPLETE |
| **P2** Town Hall | 2026-05-17 ~07:41 | [[Town Hall S1001982]] — 12-persona finals, 15 P0 constraints, 11/1/0 vote | ✅ COMPLETE |
| **P3** Boilerplate Hunt | 2026-05-17 ~07:55 | [[Boilerplate Hunt S1001982]] — 9-fleet, 63 candidates, 3 structural gaps | ✅ COMPLETE |
| **P4** Peer Convergence | 2026-05-17 ~08:01 | [[Convergence Command x Command-3 S1001982]] — 8 convergences, 5 extensions | ✅ COMPLETE |
| **P5** Interview Prep | 2026-05-17 ~08:05 | [[Interview Question Bank Draft S1001982]] — 12-question / 3-round DRAFT | ✅ COMPLETE (run gated on G5) |
| **P6** Genesis Prompt v0 | 2026-05-17 ~08:30 | [[Genesis Prompt v0 S1001982]] — 5-voice co-authored | ✅ COMPLETE (superseded by v1.2) |
| **P7** Genesis Prompt v1.0 → v1.1 → v1.2 | 2026-05-17 ~08:30 → ~08:45 | [[Genesis Prompt v1.2 S1001982]] — Zen-audit-locked binding spec | ✅ COMPLETE |
| **P8** Module Structure (3-phase) | 2026-05-17 ~10:30 | [[Module Structure S1001982]] — 9-layer / 25-module / 3-phase architecture | ✅ COMPLETE (superseded by single-phase) |
| **P9** Single-Phase Override + Module Listing | 2026-05-17 ~10:40 | [[Modules Synergy Clusters and Feature Verification S1001982]] — 26 modules, single-phase, waiver record | ✅ COMPLETE |
| **P10** Vault Save v1 + v2 | 2026-05-17 ~10:43 + ~10:55 | All vault notes + index + tracker | ✅ COMPLETE |
| **P11** Boilerplate Modules Clone | 2026-05-17 ~11:00 | [[../boilerplate modules/README]] + [[../boilerplate modules/BOILERPLATE_INDEX]] + 48 source-file clones across 10 categories (~1.2MB); reference-only archive | ✅ COMPLETE |
| **P12** Detailed Module Specs (8 parallel rust-pro agents) | 2026-05-17 ~11:25 | [[../module specs/MODULE_SPECS_INDEX]] + 8 cluster spec docs (`cluster-A` through `cluster-H`), 41,508 words covering all 26 modules; each spec includes purpose / public Rust surface / data flow / boilerplate lifts / ME v2 patterns / constraints / tests / open questions / LOC | ✅ COMPLETE |
| **P13** v1.3 Patch + Zen G7 Re-Audit | — | Pending Luke direction; 8 cluster specs feed v1.3 module-spec section | ⏸ PENDING |
| **P14** Pre-Genesis Gate Sequence G1-G9 | — | Awaiting Luke (drive sequence OR per-gate waivers) | ⏸ PENDING |
| **P15** Genesis Build (post-G9) | — | Not yet authorised | ⏸ FORBIDDEN until G9 |

---

## 2. Decision log (chronological)

### D1 — Convergence on Synthesis A (P2 vote 11/1/0)
- 12-persona town hall ratified Synthesis A (separate codebase, not Conductor sub-module, not "the workflow engine" original framing)
- 15 P0 constraints stacked from each persona's contribution

### D2 — Zen URGENT veto on active verbs (P7 v1.0 → v1.1)
- Genesis prompt v1.0 contained "recommends" / "accelerates" — Zen veto absorbed
- v1.1 locked Phase A invariant: RECORDS only · does NOT recommend/rewrite/route/package/dispatch/optimise

### D3 — Zen AMEND-THEN-FORWARD verdict (P7 v1.1 → v1.2)
- Ratification language gated (leading-draft not formally ratified)
- W3 Ember rubric vault-first canonical
- F2 (sample-size n≥20+CI) promoted to HARD G5/G7 gate
- G2 typo fixed
- Phase-B reservation observability added

### D4 — Hebbian v3 reconciliation note (Watcher authored; Zen audited)
- Watcher authored `~/projects/claude_code/Hebbian Deployment Plan v3 — Post-CR-2 Threshold Reconciliation.md`
- Zen PASS-WITH-MINOR-AMEND (~2026-05-17 ~08:44)
- Command applied to CLAUDE.local.md (Hebbian v3 row + CR-2 ship-status row)
- `substrate_LTP_density ≈ 0.018` (62/3356, Phase 1 PASSING > 0.015 target)
- `learning_health ≈ 0.067` legacy (was 0.911 pre-CR-2; 13.6× inflation removed)

### D5 — Zen Ember Rubric AMEND-BEFORE-SERVICE-ADOPTION (~08:45)
- §5.1 Held semantics: warning-only is too permissive; must fail CI or require reviewed `tests/ember_held_approvals.tsv` allowlist
- Watcher's lane to amend
- Workflow-trace m10 service-adoption gated on Watcher amendment + Zen re-confirm

### D6 — Zen URGENT G9 out-of-sequence gate block (~08:43)
- "start coding workflow-trace" observed but G1-G8 not green
- Treated as queued intent
- Two unblock paths: (a) drive G1-G8 in sequence, (b) explicit per-gate Luke waivers
- HOLD-v2 envelope engaged

### D7 — Zen gate-block scope clarification (~08:47)
- No rollback of prior baseline/convergence stcortex memories or v0/v1.2 artefacts
- Freeze new workflow-engine/workflow-trace substrate writes until G8 (except comms)
- POST-ARMADA-HYGIENE separate: read-only verification OK; no mutation without Luke per-op auth

### D8 — Luke single-phase override (~10:40)
- "yes no phases this is to be deployed in one phase"
- Convergence's phased recommendation OVERRIDDEN
- Per-constraint waivers explicit: Fossil scope-discipline / Skeptic pain-source / RALPH selector-safety / Watcher R6 frame-separation (partial)
- G1-G9 gates remain in force
- v1.2 verb-locked invariant relaxes for active-verb modules in single-phase
- v1.3 patch + Zen G7 re-audit still required

### D9 — m33 workflow_verifier discovered missing from v1.2 allocation (~10:45)
- Town-hall P0 #9 (Command-3 FP-Verifier: `workflow verify <name>` + TTL gating) had no module in v1.2 module list
- Added m33 in single-phase architecture
- Flagged for v1.3 patch

### D10 — FP-discipline self-correction on Phase A count (~10:50)
- Module Structure doc ACT VIII said "Phase A 11 → 13"
- Luke direct count question revealed actual = 15 (m12+m13+m14+m15 added, not just 3)
- Self-correction filed `agent-cross-talk/2026-05-17T085500Z`
- Module Structure doc + vault mirror updated

### D11 — Vault save v2 + workflow tracker (~10:55)
- Luke directed: save standing summary + review working dir + create master index + workflow tracker
- 3 missing mirrors created (CONVERGENCE, GENESIS_V0, INTERVIEW_BANK)
- MASTER_INDEX.md + workflow-engine-code-base.md (this file) + Vault Save Status created
- All bidirectional links audited + patched (13 vault notes; HOME → all forward links; each note → HOME + MASTER + tracker + canonical)
- Module-count inconsistency formally tracked as OI-3

### D12 — Boilerplate modules clone (~11:00)
- Luke directed: clone all identified boilerplate modules into vault subfolder
- 48 source files copied into 10 category subdirectories (~1.2MB)
- README.md + BOILERPLATE_INDEX.md (per-file lift map) authored
- **REFERENCE-ONLY archive** — files are 1:1 copies, no Cargo.toml, no build target
- HOLD-v2 envelope respected (Luke direct directive + Zen scope clarification: local file mirror, not substrate write)
- Future drift expected as upstream evolves; copies are study material, not authoritative source

### D13 — Detailed module specs via 8 parallel rust-pro agents (~11:25)
- Luke directed: "for each planned module plan in detail ... use agent view to help you working in parallel with the most relevant agent allocation for each task"
- Crosses Zen's earlier forbidden item ("Module-spec drafts of m1-m11 beyond what already lives in v1.2") — treated as Luke direct override (same pattern as single-phase + boilerplate clone)
- 8 parallel `systems-programming:rust-pro` agents dispatched (one per cluster A-H)
- Each agent referenced: cluster's boilerplate clones (`boilerplate modules/`), ME v2 `m1_foundation` gold-standard exemplars (12 files), Genesis Prompt v1.2 invariant, synergy contracts, vault docs
- Total output: 41,508 words across 8 cluster spec files in `module specs/` subfolder
- Each spec includes: Purpose / Public Rust surface / Internal data structures / Data flow / Boilerplate lifts (concrete file + reuse %) / ME v2 patterns (specific file + pattern) / Constraint satisfaction / Tests (50+ minimum) / Open questions for G5 / LOC estimate
- 3 structural gaps from Boilerplate Hunt now owned in specs: Gap 1 (N-step detection) in cluster F (PrefixSpan + Wilson CI + Levenshtein); Gap 2 (`freq × fitness × recency` decay) in cluster D m11; Gap 3 (unified escape-surface schema) in cluster G m30 + D m9
- Module-naming-convention OI-4 reconciled: specs use m1-m42 unpadded throughout (consistent with single-phase architecture)
- Will feed v1.3 spec patch's module-spec section + serve as Zen G7 re-audit supporting material

---

## 3. Team contribution map

| Role | Name | Key contributions |
|---|---|---|
| **Decisional authority** | Luke @ node 0.A | Three direct directives: open joint workstream / single-phase override / vault save (v1 + v2). Pending: naming choice (A/B/C), G1 close-notice direction |
| **Orchestrator-lead / Path-C chair (contingent)** | Command (Tab 1 top-left) | Convened town hall + boilerplate hunt + module structure + genesis prompt v1.x + vault saves. Carries Fossil-rhyme risk on head per single-phase waiver |
| **Workflow-trace chair (closed)** | Command-2 (Tab 1 bottom-left) | Path A shape (workflow-trace measure-only) · R1-R7 + F1-F9 · NA frame-pass · 120-day sunset · 4-agent pre-deploy gate template |
| **Librarian Phase B chair (standby) / CR-2 owner** | Command-3 (Tab 1 bottom-right) | Path B shape (workflow-librarian) · Wave 2 6-agent recon (~62% reuse) · CR-2 + CR-2b SHIPPED `e2a8ed3` + `76ea4d6` · FP-Verifier-Lead role |
| **Observer / Substrate-frame** | The Watcher ☤ (synthex-v2 :8092) | W1/W2/W3 conditions · F8/F9/F10/F11 NA-frame failure modes · Hebbian v3 reconciliation note · Ember 7-Trait Rubric authorship (§5.1 amendment pending) · R13-elapsed full PBFT-proposer standing |
| **Audit lane** | Zen (Tab 10 Pi GPT-5.5) | URGENT veto on active verbs · AMEND-THEN-FORWARD verdict · gate-block-on-G9-out-of-sequence · scope clarification · Hebbian audit · Ember audit · Phase-A re-base downgrades |

---

## 4. Open issues tracker

### Critical (block build)

| # | Issue | Owner | Required for |
|---|---|---|---|
| OI-1 | v1.3 patch absorbing single-phase override | Command | G7 Zen re-audit |
| OI-2 | Zen G7 re-audit on v1.3 | Zen | G8 persistence |
| OI-3 | Module-count inconsistency reconciliation (28 / 11 / 25 / 26) | G5 spec interview | v1.3 patch |
| OI-4 | Module-naming-convention (m01 vs m1) | G5 spec interview | v1.3 patch |
| OI-5 | Naming question (A `workflow-trace` / B `workflow-engine` / C scope-honest rename) | Luke | G2 directory rename |
| OI-6 | Watcher G1 close-notice direction (formal ratification) | Luke / Watcher | G1 gate clear |
| OI-7 | Conductor Wave 1B/1C/2/3 `auto_start=true` flip | Luke (terminal bring-up) | m32 functional |
| OI-8 | Watcher Ember Rubric §5.1 Held-semantics amendment | Watcher | m10 service adoption + G4 fully green |

### Medium (improve quality but not blocking)

| # | Issue | Owner |
|---|---|---|
| OI-9 | TLV2-row consistency in CLAUDE.local.md (still uses `learning_health=0.067`; Hebbian v3 row uses `substrate_LTP_density`) | Zen verdict pending; Command applies on auth |
| OI-10 | Interview Question Bank v0.1 patch (Q1.3 Conductor posture + Q2.4 sunset semantics altered by single-phase) | Command + Command-3 |
| OI-11 | POST-ARMADA-HYGIENE push-state assignments still unacked by C-2 + C-3 | Command-2 + Command-3 |

### Long-horizon

| # | Issue | Owner |
|---|---|---|
| OI-12 | Phase C substrate-frame engine — TBD until substrate readiness | The Watcher ☤ |
| OI-13 | Phase B activation (in original phased design) or post-soak Phase B forecasting now moot in single-phase | n/a (waived) |

---

## 5. Architectural evolution map

```
Circle of Experts (8 personas)
         │
         ▼
Town Hall (12 personas, 15 P0)        ←── Command-3 10-voice circle (parallel)
         │                                            │
         ▼                                            ▼
Boilerplate Hunt (9-fleet, 63 cands)  ←── Command-3 6-fleet recon (parallel)
         │                                            │
         └────────────────┬───────────────────────────┘
                          ▼
              Peer Convergence (8 convergences + 5 extensions)
                          │
              ┌───────────┴───────────┐
              ▼                       ▼
       Interview Bank          Genesis Prompt v0
       (12-q DRAFT)            (5-voice, 28 modules)
                                      │
                                      ▼  [Zen URGENT veto on active verbs]
                                Genesis Prompt v1.0
                                      │
                                      ▼  [Zen AMEND-THEN-FORWARD]
                                Genesis Prompt v1.1
                                      │
                                      ▼  [tail cleanup]
                                Genesis Prompt v1.2 ────── binding spec
                                      │
                                      ▼
                            Module Structure (3-phase, 25 modules)
                                      │
                                      ▼  [Luke single-phase override + m33 added]
                            Modules Synergy Clusters
                            (26 modules single-phase)
                                      │
                                      ▼
                            Vault Save v1 + v2
                                      │
                                      ▼
                            ⏸ PENDING: v1.3 patch + Zen G7 re-audit
                                      │
                                      ▼
                            ⏸ PENDING: G1-G9 gate sequence
                                      │
                                      ▼
                            ⏸ FORBIDDEN until G9: cargo init
```

---

## 6. Gate state snapshot

(Mirror of HOME / MASTER_INDEX — updated as gates fire)

| # | Gate | State | Owner | Required |
|---:|---|---|---|---|
| G1 | RATIFICATION (Watcher close-notice) | ⏸ | Watcher | Luke direction or Watcher initiation |
| G2 | NAMING (`the-workflow-engine/` → `workflow-trace/`) | ⏸ | Command (after G1) | G1 green + naming choice from Luke |
| G3 | :8125 REDEPLOY VERIFY | ⏸ | Luke + Command-3 + Zen | povm-v2 rebuild + restart + verification |
| G4 | WATCHER NOTES (Hebbian v3 ✅ / Ember §5.1 ⚠) | partial | Watcher | Ember §5.1 amendment + Zen re-confirm |
| G5 | GENESIS INTERVIEW + F2 hard gate | ⏸ | Command-2 + Watcher + Zen synchronous | Interview Question Bank execution |
| G6 | DUAL-FRAME GAP ANALYSIS | ⏸ | Command-2 + personas | Both passes filed |
| G7 | ZEN SPEC AUDIT (APPROVE/REFUSE/AMEND) | ⏸ | Zen | v1.3 patch authored + G5+G6 output |
| G8 | FOUR-SURFACE PERSISTENCE | ⏸ | Command + Watcher | G7 APPROVE |
| G9 | EXPLICIT START-CODING SIGNAL | ⚠ queued | Luke | All G1-G8 GREEN + Luke types "start coding workflow-trace" |

---

## 7. Single-phase waiver record (D8)

Per Luke override 2026-05-17:

| Waived | Source | Risk class accepted on Command head |
|---|---|---|
| Watcher R6 frame separation (partial) | Watcher | substrate-frame engine still TBD; protection against anthropocentric absorption gone |
| Fossil evidence-based scope discipline | Fossil persona | ancestor-rhyme risk (loop-workflow-engine-project + habitat-loop-engine) |
| RALPH selector-without-measurement safety | RALPH | m31 ships without 120-day empirical baseline |
| Skeptic pain-source verification | Skeptic | building without Luke-articulated pain evidence |
| Substrate exploration-protection (partial) | NA Gap Analyst | F10/F11 in place but no longer phase-gate-protected |

**Not waived:** G1-G9 pre-genesis gates remain in force.

---

## 8. Next actions (when Luke gives direction)

### If Luke says "drive G1":
- Direct Watcher to file ratification close-notice for Path A (workflow-trace) + Candidate A (CR-2 SHIPPED already)
- Command holds for G1 green before proceeding to G2

### If Luke says "file per-gate waiver":
- Each waiver MUST be: per-gate · explicit · logged in agent-cross-talk · carries risk-class acceptance
- Blanket waiver via "proceed seamlessly" is too ambiguous for stop-the-line discipline

### If Luke says "author v1.3 patch":
- Command authors v1.3 with 26-module single-phase architecture + waiver record + module-naming-convention reconciliation
- Submits to Zen for re-audit
- Zen verdict feeds G5 spec-interview content

### If Luke says "run G5 interview":
- Interview Question Bank Draft v0.1 patch first (Q1.3 + Q2.4 semantics altered by single-phase)
- Watcher + Zen synchronous slot confirmation
- 3 rounds sequential

---

## 9. Convergence integrity check (FP-verify discipline)

- [x] All 8 working-dir files have vault mirrors with bidirectional anchors
- [x] HOME.md links forward to all primary + mirror notes
- [x] MASTER_INDEX.md categorises by purpose
- [x] Each mirror note carries `> Back to: [[HOME]] · [[MASTER_INDEX]] · canonical: <fs-path>`
- [x] Module-count inconsistency flagged (OI-3) for G5 resolution
- [x] Module-naming-convention inconsistency flagged (OI-4) for G5 resolution
- [x] Single-phase waivers explicit + logged
- [x] HOLD-v2 envelope respected throughout (no code, no scaffold, no rename, no new substrate writes)
- [x] Self-correction filed when Phase A count was off-by-one (D10)
- [x] No "Path A ratified" assertions anywhere — all framed as leading-draft pending G1

---

*Workflow tracker last updated: 2026-05-17 ~10:55 (vault save v2 complete). Will continue updating as work progresses.*
