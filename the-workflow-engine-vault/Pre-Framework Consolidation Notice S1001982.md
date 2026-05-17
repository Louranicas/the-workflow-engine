---
title: Pre-Framework Consolidation Notice
date: 2026-05-17 (S1001982 / S1002029 executor)
kind: audit-notice · structural-move-record
status: planning-only · HOLD-v2 respected (no code, no scaffold; pure filesystem reorganisation)
authority: Luke @ node 0.A — "move all files and folders ... into a new folder called pre-framework-consolidation"
executor: Claude (workspace primary session S1002029)
---

# Pre-Framework Consolidation Notice

> Back to: [[HOME]] · [[MASTER_INDEX]] · [[Vault Save Status S1001982]]

This note records the **2026-05-17 filesystem consolidation** of all pre-framework planning artefacts at the project root into the new `pre-framework-consolidation/` subfolder. The move is non-destructive (filesystem `mv`), fully reversible, and was executed under explicit Luke directive.

---

## §1 — What moved

10 markdown files moved from `the-workflow-engine/` (project root) into `the-workflow-engine/pre-framework-consolidation/`:

| # | File | Size | Role |
|--:|---|---:|---|
| 1 | `CONVERGENCE_COMMAND_X_COMMAND3_S1001982.md` | 17 KB | Peer convergence — Command × Command-3 |
| 2 | `GENESIS_PROMPT_V0.md` | 22 KB | 5-voice co-authored genesis (superseded by v1.2) |
| 3 | `INTERVIEW_QUESTION_BANK_DRAFT.md` | 12 KB | G5 interview draft (12 questions / 3 rounds) |
| 4 | `THE_WORKFLOW_ENGINE_BOILERPLATE_HUNT_S1001982.md` | 18 KB | 9-Explore-fleet, 63 candidates → 9 top-picks |
| 5 | `THE_WORKFLOW_ENGINE_CIRCLE_OF_EXPERTS_S1001982.md` | 35 KB | 8-persona disputation + 3 syntheses |
| 6 | `THE_WORKFLOW_ENGINE_END_TO_END_DEPLOYMENT_PLAN_S1001982.md` | 39 KB | /scaffold × atuin × V3 × V8 12-step pipeline plan (S1002029-authored) |
| 7 | `THE_WORKFLOW_ENGINE_GENESIS_PROMPT_V1_S1001982.md` | 40 KB | Zen-audit-locked v1.2 binding spec |
| 8 | `THE_WORKFLOW_ENGINE_MODULE_STRUCTURE_S1001982.md` | 37 KB | 3-phase 9-layer architecture sketch (superseded by single-phase override) |
| 9 | `THE_WORKFLOW_ENGINE_TOWN_HALL_S1001982.md` | 24 KB | 12-persona finals + 15 P0 constraints |
| 10 | `WATCHER_DEPLOYMENT_WATCH_JOURNAL_S1001982.md` | 28 KB | Watcher's deployment-watch journal |

**Total moved:** 272 KB across 10 files.

---

## §2 — What did NOT move (exclusions)

Per Luke's directive, the following were preserved at their existing locations:

1. **`the-workflow-engine-vault/`** — the entire Obsidian vault (active surface, unchanged)
2. **`the-workflow-engine-vault/deployment framework/`** — the new framework-era subfolder (preserved per explicit exclusion; contains `phase-0-pre-genesis-gates.md` + `phase-1-genesis-day-0-3.md`)

---

## §3 — One judgment call flagged

- **`.obsidian/` dotfolder at project root** was **NOT moved**. Rationale: it is Obsidian editor configuration (analog of `.git/`), not a planning artefact. Keeping it at the project root keeps the Obsidian vault scope spanning both `pre-framework-consolidation/` and `the-workflow-engine-vault/`, so all `[[wikilink]]` references continue to resolve cleanly via Obsidian's recursive search. If Luke / Command wants `.obsidian/` moved too, it can be done in one operation without breaking content.

---

## §4 — Post-move structure

```
the-workflow-engine/
├── .obsidian/                            (untouched — editor config)
├── pre-framework-consolidation/          (NEW — 10 archived planning artefacts)
│   ├── CONVERGENCE_COMMAND_X_COMMAND3_S1001982.md
│   ├── GENESIS_PROMPT_V0.md
│   ├── INTERVIEW_QUESTION_BANK_DRAFT.md
│   ├── THE_WORKFLOW_ENGINE_BOILERPLATE_HUNT_S1001982.md
│   ├── THE_WORKFLOW_ENGINE_CIRCLE_OF_EXPERTS_S1001982.md
│   ├── THE_WORKFLOW_ENGINE_END_TO_END_DEPLOYMENT_PLAN_S1001982.md
│   ├── THE_WORKFLOW_ENGINE_GENESIS_PROMPT_V1_S1001982.md
│   ├── THE_WORKFLOW_ENGINE_MODULE_STRUCTURE_S1001982.md
│   ├── THE_WORKFLOW_ENGINE_TOWN_HALL_S1001982.md
│   └── WATCHER_DEPLOYMENT_WATCH_JOURNAL_S1001982.md
└── the-workflow-engine-vault/            (untouched — Obsidian vault)
    ├── deployment framework/             (untouched — framework-era artefacts)
    │   ├── phase-0-pre-genesis-gates.md
    │   └── phase-1-genesis-day-0-3.md
    ├── boilerplate modules/              (untouched — reference archive)
    ├── module specs/                     (untouched — cluster deep dives)
    ├── HOME.md                            (updated — external-references section)
    ├── MASTER_INDEX.md                    (updated — fs-path-summary line)
    └── Pre-Framework Consolidation Notice S1001982.md  (THIS NOTE)
```

---

## §5 — Wikilink resolution check

The vault uses canonical-pair wikilinks like `[[Genesis Prompt v1.2 S1001982]] ↔ [[THE_WORKFLOW_ENGINE_GENESIS_PROMPT_V1_S1001982]]`. The right-pole pointers (UPPER_SNAKE_CASE) **still resolve** because:

- Obsidian's vault scope is rooted at `the-workflow-engine/` (where `.obsidian/` lives)
- Vault scope includes ALL subfolders recursively (default behaviour)
- `pre-framework-consolidation/` is inside the scope
- Files moved into it are still indexed by filename, so wikilinks find them

**No broken-link sweep needed.** Confirmed by checking that the `.obsidian/` config sits at project root (line above).

---

## §6 — What this move means semantically

The folder name **`pre-framework-consolidation`** signals that everything inside it is the **planning surface that PRECEDED the framework consolidation**. The framework itself now lives at `the-workflow-engine-vault/deployment framework/` and is the new authoring surface.

This implies (but does not enforce — that's the Command/Watcher/Zen call):

- New planning artefacts authored at the framework level should go into `the-workflow-engine-vault/deployment framework/`, not back at the project root.
- The 10 archived files are **read-only history** unless explicitly re-opened (e.g., v1.3 patch on v1.2 genesis spec, or new revisions of the deployment plan).
- The phase-0 and phase-1 framework files are the live surface; phase-2+ is implied.

---

## §7 — Vault index updates landed

1. **HOME.md** § "Key external references" — replaced stale `~/claude-code-workspace/the-workflow-engine/*.md` with split pointer (pre-framework + active framework)
2. **MASTER_INDEX.md** § header note — clarified that pre-framework canonical fs paths are now in the new subfolder; framework artefacts live in `deployment framework/`
3. **This note** added as a top-level vault entry for audit-trail durability

---

## §8 — What was NOT changed

- ❌ No content edits to the 10 moved files — only their parent directory changed
- ❌ No edits to wikilinks anywhere (vault scope handles them)
- ❌ No edits to the canonical-pair table in HOME.md (wikilinks resolve)
- ❌ No deletion of any file
- ❌ No four-surface persistence trigger (this is a vault-local audit notice; stcortex/POVM untouched)
- ❌ No edits to existing cross-talk files

---

## §9 — How to revert (if needed)

```bash
cd /home/louranicas/claude-code-workspace/the-workflow-engine
mv pre-framework-consolidation/*.md ./
rmdir pre-framework-consolidation
# Then revert the HOME.md / MASTER_INDEX.md edits manually
# Then delete this notice file
```

---

## §10 — Execution receipt

- **Executed by:** Claude (workspace primary session S1002029)
- **Timestamp:** 2026-05-17T01:?? local (just after the cross-talk drops at T015900Z)
- **Method:** `mkdir -p` + `mv -v` (10 files, all verbose-logged)
- **Verification:** `ls -la` on `the-workflow-engine/`, `pre-framework-consolidation/`, and both exclusion paths — all confirmed
- **Reversibility:** full; non-destructive move within one project directory

---

*Notice authored under Luke directive · respects HOLD-v2 envelope · planning-only artefact · workspace session S1002029*
