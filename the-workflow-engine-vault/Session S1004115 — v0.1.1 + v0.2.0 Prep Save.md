> **Back to:** [[CLAUDE.md]] · [[CLAUDE.local.md]] · [`../CLAUDE.local.md` § v0.1.0 / M0 SHIPPED](../CLAUDE.local.md)
> **Type:** Session save · cold-start anchor · 4-surface persistence
> **Date:** 2026-05-23 (S1004115 continuation, post-v0.1.0 / M0)

# Session S1004115 — v0.1.1 + v0.2.0 Prep Save

This note captures the **post-v0.1.0 / M0 round of cleanup + v0.2.0 prep** so a fresh Claude context window can resume seamlessly. Mirrors the stcortex memory `workflow_trace_completion_s1004115` at this save (id forthcoming when verified) and the `injection.db` causal_chain `workflow_trace_v011_hygiene_plus_v020_prep_s1004115` entry.

## v0.1.1 hygiene round — CLOSED (commits on main, both remotes)

| Task | Commit | Subject |
|------|--------|---------|
| T10 | `0f0ca8c` | T4-DEAD-ERR m5 BatternError 2 construction tests (W2/W3 already covered the other ~13 variants; CLOSED) |
| T11 | `a7c697f` | SD A/B spec doc amendments — 4 spec files (m9 m14 m15 m20) banner-pointed at PHASE9_SD_RECONCILIATION (D27) |
| T12+T13 | `d62ba80` | R3 m22 K-means CLI batch-path distribution test + R4 m8 KEEP-DORMANT cfg invariant lock test |
| T14+T15+T16 | `0787ea6` | Hygiene batch — 15 stale worktrees removed; `target-mutants/` trashed + gitignored; Watcher journals git-rm-cached + gitignored (Zen Phase-1 hygiene-asymptotic finding resolved) |

**Test count:** 2044 → 2048 (+4 across T10/T12/T13)

## v0.2.0 prep — CLOSED (commits on main, both remotes)

| Task | Commit | Subject |
|------|--------|---------|
| T17 | `7d16c2c` | `ai_docs/WORKFLOW_TRACE_V020_PLAN_STUB_S1004115.md` (scope catalogue; ratification awaits node-0.A "start v0.2.0 planning" per §15 D40) |
| T18 | `7d16c2c` | Phase 5 zen A2 m31 caller wire — `diversity_score_from_proposal` consumes `proposal.diversity_cluster()`; behavioural-loop **CLOSED** |
| T19 | `7d16c2c` | CI spacetimedb-sdk path-dep concrete recipe (Options A submodule / B vendor / C crates.io) in `.github/workflows/ci.yml` + `.gitlab-ci.yml`, with pinned upstream commit `fbec2761abbf65b90673130835c4cab6c016924c` |

**Test count post-T18:** 2048 (no new tests; closure-shape change covered by existing distribution + cc4 dual-branch tests)

## Still STANDING for node 0.A (irreducibly Luke-only)

| ID | Item | Why agent cannot |
|----|------|------------------|
| OP-1 | Conductor bring-up: `~/.local/bin/devenv -c ~/.config/devenv/devenv.toml start --services weaver,zen,enforcer` → 24h NoOp soak → flip `CONDUCTOR_ENFORCEMENT_ENABLED=1` | `devenv start` forbidden per CLAUDE.md (sandbox reaps children) |
| OP-2 | Directory rename `the-workflow-engine/` → `workflow-trace/` (Plan v2 §15 D32) | Too cross-Habitat for safe agent execution (touches ~/projects/claude_code/ vault references, every CLAUDE.md table across services) |
| v0.2.0 | Substrate-safety milestone full ratification (NA-GAP-01/04/07/08/10 + Tier-2 wire-contract + Tier-3 real verifiers + Tier-4 SD8/9/10/11) | Per Plan v2 §15 D40 invocation-only cadence; needs a v0.2.0 plan with own decision interview |

## Cold-start sequence (for fresh Claude context window)

```bash
# 1. Read project session-state delta (single source of truth for current state)
cd /home/louranicas/claude-code-workspace/the-workflow-engine
$EDITOR CLAUDE.local.md           # § "v0.1.0 / M0 SHIPPED" block

# 2. Read v0.2.0 scope (what's NEXT if/when node 0.A ratifies)
$EDITOR ai_docs/WORKFLOW_TRACE_V020_PLAN_STUB_S1004115.md

# 3. Read this vault note for the save-time snapshot
#    (Obsidian: open vault; vim/cat path below)
$EDITOR the-workflow-engine-vault/Session\ S1004115\ —\ v0.1.1\ +\ v0.2.0\ Prep\ Save.md

# 4. Read the CHANGELOG v0.1.0 honest-residuals section
$EDITOR CHANGELOG.md              # § [v0.1.0] § Honest residuals

# 5. Read the stcortex memory for the substrate-side snapshot
~/.local/bin/stcortex inspect workflow_trace_completion_s1004115 --limit 5

# 6. Verify git anchor
git log --oneline -1                # expected: 7d16c2c v0.2.0 prep
git describe --tags HEAD            # expected: v0.1.0-<N>-g7d16c2c (or similar)
git status -s -- .                  # expected: empty (only `.obsidian/workspace.json` UI churn)
```

## State-at-save snapshot

| Attribute | Value |
|-----------|-------|
| Project HEAD | `7d16c2c` (v0.2.0 prep) — v0.1.0 tag remains at `df00fd2` |
| Remotes | `origin/main` + `gitlab/main` both at `7d16c2c` |
| Tests | **2048** passing, 0 failed (38 suites) |
| Clippy | clean (default + pedantic) |
| Mutation kill-rate | **96.3 %** held (baseline from `2096fd0` final fold; 10 survivors all proven-equivalent) |
| Working tree | clean (only `.obsidian/workspace.json` UI churn) |
| Tasks completed this session | 21 (Plan v2 10 phases + v0.1.1 7 items + v0.2.0 prep 4 items) |
| Cron jobs | `dc3d06c8` cancelled at M0 ship; CronList = no scheduled jobs |
| Watcher journal | gitignored (Zen hygiene-asymptotic CLOSED); historical content through `b18ef0c` preserved |

## 4-surface persistence at this save

| Surface | Anchor |
|---------|--------|
| ai_docs | `ai_docs/WORKFLOW_TRACE_V020_PLAN_STUB_S1004115.md` (NEW) + earlier `WORKFLOW_TRACE_COMPLETION_PLAN_V2_S1004115.md` + `PHASE1/2/8/9_*` docs |
| Obsidian vault | THIS note + earlier `Workflow-Trace Completion Plan v2 S1004115.md` + `Hardening Fleet 2026-05-21.md` + `Assessment Remediation S1003733.md` |
| stcortex | namespace `workflow_trace_completion_s1004115` (memories 18376 genesis + 18383 interview-locked + 18442 M0 ship + this save's new memory) + bidi pathways to `workflow_trace_hardening_2026_05_21` (mems 17939 + 18410) |
| CLAUDE.local.md | project file § "v0.1.0 / M0 SHIPPED" block (forthcoming update with v0.1.1 + v0.2.0 prep section) |
| CHANGELOG | `[v0.1.0]` entry (canonical release record) |
| injection.db | causal_chain `workflow_trace_v011_hygiene_plus_v020_prep_s1004115` entry (NEW this save) |
| git tag | `v0.1.0` annotated at `df00fd2` |

— Session S1004115 v0.1.1 + v0.2.0 Prep Save, 2026-05-23.
