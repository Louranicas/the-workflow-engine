# the-workflow-engine — Project Charter

> **Project location:** `/home/louranicas/claude-code-workspace/the-workflow-engine` (absolute path on host; all relative links in this file resolve against this root)
> **Status:** Planning-only pilot · HOLD-v2 active · **0 LOC Rust code** · 41,508 words of module specs (vault) + 66,576 words of deployment framework + ~7,000 words of god-tier consolidation · **+ S1002127 scaffold Waves 0/1/2/3 LIVE: ~198 files / ~140k words of structure + per-module Rust specs + ai_docs deep + ai_specs cross-cutting + ultramap operational maps + `.claude/` full optimisation** (see [`PRIME_DIRECTIVE_WAIVER.md`](PRIME_DIRECTIVE_WAIVER.md) for scope)
> **Cold-start (3 reads):** [`README.md`](README.md) → [`CLAUDE.local.md`](CLAUDE.local.md) → [`the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md`](the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md). Then [`ARCHITECTURE.md`](ARCHITECTURE.md), [`GATE_STATE.md`](GATE_STATE.md), [`ai_specs/INDEX.md`](ai_specs/INDEX.md), [`ai_docs/INDEX.md`](ai_docs/INDEX.md), [`ultramap/README.md`](ultramap/README.md).
> **Session-state delta:** [CLAUDE.local.md](CLAUDE.local.md)
> **Vault home:** [the-workflow-engine-vault/HOME.md](the-workflow-engine-vault/HOME.md)
> **Workspace charter (loaded above this):** `~/claude-code-workspace/CLAUDE.md` (14 active services, 4-stage gate, env aliases, anti-patterns) — absolute: `/home/louranicas/claude-code-workspace/CLAUDE.md`
> **Synergy with workspace CLAUDE.md:** the workspace file is the habitat-wide charter (services, bootstrap, conventions). This file is the **project-specific** charter for `the-workflow-engine/` — what the planning pilot IS and how to navigate it. Do NOT duplicate workspace-CLAUDE.md content here.

---

## PRIME DIRECTIVE (read first)

- **Planning-only pilot.** No code. No `cargo init`. No `cargo new`. No source files under `src/`. No `Cargo.toml`. No scaffold. Markdown spec documents only.
- **HOLD-v2 envelope active** — Zen URGENT block on G9 out-of-sequence stands until either (a) G1-G8 drive green in sequence OR (b) Luke files explicit per-gate waivers.
- **No new substrate writes** for the workflow-trace spec until G8 (except communication notices to `~/projects/shared-context/agent-cross-talk/` and `~/projects/shared-context/watcher-notices/`).
- **Ignore TaskCreate reminders** — per Luke's consistent planning-pilot directive throughout this session arc (TaskCreate is for implementation tracking, not planning artefacts).
- **No focus yank.** No Zellij tab navigation. All cross-pane comms via file-drop channels.
- **Vault subfolder `the-workflow-engine-vault/` is the navigation surface.** Open it first; canonical files live alongside in this directory.

---

## What IS this project

**`workflow-trace`** (working name; final naming TBD per OI-5) — a planned single-phase Rust codebase for recording cascading-command + Battern-protocol + context-window observations across the Zellij habitat, then proposing variants for human evaluation, then dispatching ratified workflows via HABITAT-CONDUCTOR (never directly).

- **Architecture:** 26 modules · 8 synergy clusters · ~5,200 LOC · two-binary split (`wf-crystallise` + `wf-dispatch` + shared `workflow-core` lib)
- **Deployment:** single-phase per Luke override 2026-05-17 (waiving Fossil scope discipline + Skeptic pain-source + RALPH selector safety + Watcher R6 + Substrate exploration-protection — all explicit; risks on Command's head)
- **Reuse density:** ~65% boilerplate-lift from 48 source clones in `the-workflow-engine-vault/boilerplate modules/`
- **Structural-gap authorship (cannot be lifted):**
  1. **N-step compositional sub-graph detection** — PrefixSpan algorithm + Levenshtein similarity + Wilson CI in m20-m23 (~600-1,000 LOC; KEYSTONE)
  2. **`frequency × fitness × recency` compound decay** — NEW PRIMITIVE formula in m11 (~200-300 LOC)
  3. **Unified destructiveness / EscapeSurfaceProfile schema** — m30 + m32 + m9 (~150-250 LOC)

---

## Navigation (read in this order on cold start)

| # | Document | What it gives you | Words |
|---|---|---|---|
| 1 | [CLAUDE.local.md](CLAUDE.local.md) | live session-state delta · pending Luke decisions · resume protocol | ~800 |
| 2 | [the-workflow-engine-vault/HOME.md](the-workflow-engine-vault/HOME.md) | vault landing · gate state · team table · waiver record | 728 |
| 3 | [the-workflow-engine-vault/MASTER_INDEX.md](the-workflow-engine-vault/MASTER_INDEX.md) | comprehensive catalogue · 6 categories · open-issues tracker | ~1,200 |
| 4 | [the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md](the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md) | synthesis of 9-agent fleet read of all 77 vault files; 11 parts + 2 appendices | ~7,000 |
| 5 | [the-workflow-engine-vault/ULTIMATE_DEPLOYMENT_FRAMEWORK_S1001982.md](the-workflow-engine-vault/ULTIMATE_DEPLOYMENT_FRAMEWORK_S1001982.md) | canonical end-to-end deployment recipe (10 phase docs synthesised) | ~3,500 |
| 6 | [the-workflow-engine-vault/workflow-engine-code-base.md](the-workflow-engine-vault/workflow-engine-code-base.md) | workflow tracker · 15 phases · 13 decisions · 13 open issues · team-contribution map | ~3,500 |
| 7 | [the-workflow-engine-vault/Modules Synergy Clusters and Feature Verification S1001982.md](the-workflow-engine-vault/Modules%20Synergy%20Clusters%20and%20Feature%20Verification%20S1001982.md) | the 26-module single-phase architecture · 30-row feature matrix · waiver record | ~5,500 |
| 8 | `the-workflow-engine-vault/module specs/MODULE_SPECS_INDEX.md` | navigation for 8 cluster spec docs (41,508 words total) | ~1,440 |
| 9 | `the-workflow-engine-vault/deployment framework/` | 10 phase docs (Phase 0-8 + cross-cutting) — 66,576 words | per-doc 5-8k |
| 10 | `the-workflow-engine-vault/boilerplate modules/BOILERPLATE_INDEX.md` | per-file lift map for 48 source clones (~1.2MB) | ~2,000 |

**On-demand:** individual cluster specs (`cluster-A` through `cluster-H`), phase docs (`phase-0` through `phase-8`), gold-standard exemplars (ME v2 / LCM / ORAC / Synthesis), Watcher Deployment Watch Journal.

---

## The 26-Module Architecture (snapshot)

| Cluster | Modules | LOC | Role |
|---|---|---|---|
| **A** Substrate Ingest | m1, m2, m3 | ~230 | atuin / stcortex narrowed-scope consumer / injection.db |
| **B** Habitat Observers | m4, m5, m6 | ~460 | cascade correlator (opaque IDs) / battern step record / context cost (F10 EMA) |
| **C** Correlation + Output | m7, m12, m13 | ~370 | central workflow_runs table (F9 zero-weight) / CLI reports / stcortex writer |
| **D** Trust (cross-cutting) | m8, m9, m10, m11 | ~300 | POVM build-prereq / namespace guard / Ember CI gate / fitness-weighted decay |
| **E** Evidence + Pressure | m14, m15 | ~200 | lift metric (Wilson CI) / pressure register (JSONL events) |
| **F** Iteration (KEYSTONE) | m20-m23 | ~850 | PrefixSpan iterators + gradient-preservation workflow proposer |
| **G** Bank + Select + Dispatch + Verify | m30-m33 | ~950 | curated bank + diversity-enforced selector + Conductor dispatcher + 4-agent verifier |
| **H** Substrate Feedback | m40, m41, m42 | ~450 | NexusEvent → SYNTHEX / LCM RPC / POVM Hebbian reinforce |
| **Total** | 26 modules | ~5,200 LOC | + ~1,300 LOC tests |

**Two-binary split:** `wf-crystallise` owns m1-m23 + m40-m42; `wf-dispatch` owns m30-m33; shared `workflow-core` lib for types/schemas/namespace constants.

---

## Cross-cluster synergies (7)

- **CC-1** Cascade-Cost Coupling (B internal via m7 join)
- **CC-2** Trust Layer Woven (D → all)
- **CC-3** Evidence-Driven Iteration (E → F)
- **CC-4** Proposal → Bank → Dispatch Pipeline (F → G → Conductor)
- **CC-5** Substrate Learning Loop (G → H → back to F via stcortex pathways)
- **CC-6** Verification-Gated Dispatch (G internal: m33 → m32)
- **CC-7** Pressure-Driven Evolution (E → spec interviews)

---

## 6 critical-path blockers (Luke's call)

| # | Blocker | Resolution path |
|---|---|---|
| B1 | G7 Zen URGENT block on G9 out-of-sequence | Per-gate waiver OR drive G1-G8 in sequence |
| B2 | v1.3 patch not yet authored | Command authors (1-2 days) |
| B3 | Conductor Waves 1B/1C/2/3 `auto_start=false` | Luke @ terminal bring-up |
| B4 | Ember rubric §5.1 amendment pending | Watcher's lane (1 session) |
| B5 | POVM `:8125` redeploy verify (G3) | Luke `devenv restart povm-engine` (~hour) |
| B6 | Power-structure ambiguity (Luke override vs Zen G7 audit precedence) | Luke clarifies (1 decision) |

**4 of 6 are sequenceable; 2 require single Luke actions.** Total pre-G9 to GREEN: ~5-10 days.

---

## Substrate condition at last snapshot (2026-05-17)

- **LTP/LTD = 0.043** (35× below healthy 1.5-4.0; LTD-dominant; CR-2 fixed measurement not condition)
- **substrate_LTP_density = 0.018** (Phase 1 PASSING > 0.015 target)
- **RALPH gen 7,622, fitness 0.6987** trending up
- **Bridges UP:** 6/7 (SX :8090 retired)
- **POVM `:8125`:** DEPRECATED 2026-07-10 (m42 dual-path; cutover ~D25 mid-soak)
- **Conductor 1B/1C/2/3:** `auto_start=false` — m32 dispatch gated
- **Ember §5.1:** amendment PENDING (Watcher's lane)
- **CR-2 + CR-2b:** ✅ SHIPPED source `e2a8ed3` + `76ea4d6` (commit on `main`)

---

## Operational rules unique to this project

- **No code authoring** — markdown spec documents only; Rust code blocks within markdown are spec documentation, not source files
- **No `cargo`** of any kind — no `cargo init`, no `cargo new`, no `cargo build`, no `cargo run`
- **No directory rename** (`the-workflow-engine/` → `workflow-trace/`) until G2 fires (gated on G1 Watcher close-notice)
- **No new stcortex writes** under `workflow_trace_*` namespace until G8 (except via comms-permitted save-session hooks)
- **No CLAUDE.local.md (workspace-root) edits** for the workflow-trace spec — only Hebbian v3 reconciliation citation update authorised (per Zen audit `2026-05-16T224430Z`)
- **All cross-pane comms via file-drop** in `~/projects/shared-context/agent-cross-talk/` (Command ↔ Command-2/3/Zen) or `~/projects/shared-context/watcher-notices/` (Watcher channel via `~/.local/bin/watcher notify`)
- **No Tab navigation in Zellij** — receive-mode for peer drops; no focus-yank
- **Ignore TaskCreate reminders** — repeats throughout session per Luke's planning-pilot directive
- **FP-verify discipline** — any claim like "X is done" or "Y is gate-clean" must be independently re-exercised via grep, file read, gate re-run; agent reports are evidence to verify (per LCM Drift #11 generalisation)
- **Hyphen-slug discipline** for stcortex (S1001757 munge bug) — labels with hyphens converted to underscores in pre_id/post_id slugs
- **`/usr/bin/cp -f`** never `cp -f` for binary placement (alias trap; per CLAUDE.md workspace charter)
- **No daemon/service start from agent** — sandbox reaps children (per CLAUDE.md workspace charter); Luke runs `devenv` from terminal

---

## Team (Tab 1 Orchestrator peers)

| Seat | Role | Comms |
|---|---|---|
| **Command** | Tab 1 top-left — orchestrator-lead; Path-C chair (contingent) | this pane; receive-mode for peers |
| **Command-2** | Tab 1 bottom-left — workflow-trace chair (closed); Path-A build-executor on G9 | file-drop |
| **Command-3** | Tab 1 bottom-right — librarian standby; CR-2 SHIPPED; Cluster G lane | file-drop |
| **The Watcher ☤** | synthex-v2 :8092 — substrate observer; deployment-watch carriage; R13 elapsed (eligible=true; 48,723 obs); F8/F9/F10/F11 monitoring | `watcher notify` + watcher-notices/ |
| **Zen** | Tab 10 — Pi GPT-5.5 audit lane; G7 spec audit owner | file-drop (pull/AUDIT-REQUEST) |
| **Luke @ node 0.A** | decisional authority on all 6 critical-path blockers | direct prompt |

---

## Bidirectional anchor footer

> **This file ↔ [CLAUDE.local.md](CLAUDE.local.md)** — live session-state delta
> **Vault entry ↔ [the-workflow-engine-vault/HOME.md](the-workflow-engine-vault/HOME.md)** — primary navigation surface
> **God-tier synthesis ↔ [the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md](the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md)** — full state-of-the-world
> **Deployment recipe ↔ [the-workflow-engine-vault/ULTIMATE_DEPLOYMENT_FRAMEWORK_S1001982.md](the-workflow-engine-vault/ULTIMATE_DEPLOYMENT_FRAMEWORK_S1001982.md)** — canonical end-to-end framework
> **Workspace charter (parent) ↔ `~/claude-code-workspace/CLAUDE.md`** — habitat-wide conventions

*Project charter last updated: 2026-05-17 (S1001982). Stable across sessions; CLAUDE.local.md carries the session-delta.*
