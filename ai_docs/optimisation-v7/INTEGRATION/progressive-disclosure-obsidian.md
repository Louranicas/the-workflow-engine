---
title: Progressive Disclosure with Obsidian Deep-Dive — workflow-trace V7
date: 2026-05-17 (S1001982)
kind: planning-only · integration deep-dive · expands G5 § Progressive disclosure with Obsidian
parent: GENERATIONS/G5-tooling.md
owner: obsidian-vault-librarian (per-Wave-end sweep); Command (MOC + MASTER_INDEX); per-author bidi anchor discipline
scope: the-workflow-engine-vault/ — TIER-1/TIER-2/TIER-3 hierarchy + bidi-anchor + file-size invariants
---

# Progressive Disclosure with Obsidian — workflow-trace V7

> Back to: [[../TASK_LIST_V7_OPTIMISATION.md]] · [[../GENERATIONS/G5-tooling.md]] · [[../ULTRAMAP.md]]
>
> Siblings: [[scaffold-integration.md]] · [[atuin-integration.md]] · [[devops-v3-integration.md]] · [[codesynthor-v8-integration.md]] · [[json-claude-code-optimisation.md]]

---

## Overview

Progressive disclosure with Obsidian is the **cognitive-load shape** for the workflow-trace vault — a three-tier hierarchy (Landing / Per-cluster + Per-phase / Per-module) that ensures any fresh Claude session can navigate from `HOME.md` to the exact piece of information needed in **at most three hops**, with file-size invariants that keep each tier's working memory footprint bounded. Per G5 § Progressive disclosure with Obsidian, the structure is: TIER-1 always-loaded landing docs (≤200 words; HOME, MASTER_INDEX, KEYWORDS_20 symlink); TIER-2 on-demand cluster + phase + integration deep-dives (≤2500 words; cluster-A through cluster-H, runbook-00 through runbook-11, INTEGRATION/*); TIER-3 deep-load per-module overviews (≤500 words; m1-overview through m42-overview, sliced from the cluster files for surgical loads). Bidi-anchor discipline mandates every TIER-N doc carries `> Back to: [[TIER-N-1 parent]]` AND every TIER-N+1 child link is present in the parent's manifest section — round-trip navigation in one hop, no orphan files. The obsidian-vault-librarian agent runs a per-Wave-end sweep against a 4-point checklist (size invariants + bidi anchors + orphan files + child-link presence). The vault path is `~/claude-code-workspace/the-workflow-engine/the-workflow-engine-vault/` (pre-G2; moves to `workflow-trace/workflow-trace-vault/` post-G2 rename). Activation: continuous from Phase 0 (vault already exists per project CLAUDE.local.md); per-Wave-end sweep starts Phase 1 Day 3; weekly synthesis Phase 5C onward.

---

## 3-tier disclosure structure

Per G5 § Progressive disclosure with Obsidian "3-tier disclosure" line, with V7-specific expansions:

```text
TIER 1 — Landing (always loaded; ≤200 words each)
├── HOME.md                          — "what is this; where to go" (728w currently — TRIM target)
├── MASTER_INDEX.md                  — comprehensive catalogue (~1,200w — split into MOC + appendix)
└── KEYWORDS_20.md (symlinked)       — 20-keyword context anchor (≤160w; lives at ai_docs/optimisation-v7/)

TIER 2 — Per-cluster + Per-phase + Per-integration (on-demand; ≤2500 words each)
├── module specs/                    — pre-existing 8 cluster specs (41,508w total — over budget; per-cluster sweep needed)
│   ├── cluster-A.md
│   ├── cluster-B.md
│   ├── …
│   └── cluster-H.md + MODULE_SPECS_INDEX.md
├── deployment framework/             — pre-existing 10 phase docs (66,576w — split per-phase)
│   ├── phase-0.md … phase-8.md + cross-cutting.md
├── ai_docs/optimisation-v7/RUNBOOKS/  — V7-authored 12 runbooks (R-00 through R-11)
└── ai_docs/optimisation-v7/INTEGRATION/  — V7-authored 6 deep-dives (THIS FILE + 5 siblings)

TIER 3 — Per-module (deep-load only when implementing; ≤500 words each)
├── m1-overview.md … m42-overview.md  — sliced from cluster-N.md per-module sections
│   (not yet authored; deferred to post-Wave-2 when modules stable)
└── boilerplate-overview/             — per-source-clone lift map (one card per source file)
    ├── ME-v2-overview.md
    ├── LCM-overview.md
    ├── ORAC-overview.md
    └── CSv8-overview.md (gold-standard exemplars per G4)
```

**Tier-load rationale:**
- **TIER 1** is what's loaded when a Claude session opens any file under `the-workflow-engine/` (per CLAUDE.local.md Resume protocol step 1 + step 8 reading order). Total budget: ~600 words across 3 files = budget-tight but achievable.
- **TIER 2** is loaded **on-demand** — typically one cluster + one phase + one integration at a time during a build sprint. Total per-session-load budget: ≤7,500 words (3 docs × 2,500w).
- **TIER 3** is loaded **only when editing the specific module's src/**, typically one module-card at a time. Per-session-load budget: ≤500 words.

**Total cognitive-load budget per typical sprint session:** TIER 1 (600w) + TIER 2 (~5,000w cluster + phase) + TIER 3 (~500w module) = ~6,100w. Compare to dumping the full vault (~115,000 words across 88 files) — that's a 19× reduction in working-memory pressure.

---

## Vault structure at `/home/louranicas/claude-code-workspace/the-workflow-engine/the-workflow-engine-vault/`

```text
the-workflow-engine-vault/                                    [project root subfolder]
├── HOME.md                                  TIER 1            — landing (target ≤200w; currently 728w)
├── MASTER_INDEX.md                          TIER 1            — catalogue (target ≤200w MOC + appendix)
├── GOD_TIER_CONSOLIDATION_S1001982.md       TIER 2 (special)  — 9-agent synthesis (~7,000w — keep at TIER 2 due to status)
├── ULTIMATE_DEPLOYMENT_FRAMEWORK_S1001982.md TIER 2 (special) — canonical recipe (~3,500w)
├── workflow-engine-code-base.md             TIER 2            — workflow tracker (~3,500w; over but stable)
├── Modules Synergy Clusters and Feature Verification S1001982.md  TIER 2 — architecture (~5,500w; over)
├── Watcher Deployment Watch Journal S1001982.md TIER 2        — operator journal (ongoing append)
├── module specs/                            TIER 2 set        — 8 cluster files
│   ├── MODULE_SPECS_INDEX.md (TIER 1 MOC subfolder; ≤200w)
│   ├── cluster-A.md (target ≤2,500w; currently over — slice TIER 3 cards)
│   ├── cluster-B.md … cluster-H.md
├── deployment framework/                    TIER 2 set        — 10 phase docs
│   ├── DEPLOYMENT_FRAMEWORK_INDEX.md (TIER 1 MOC subfolder; ≤200w; author at G7)
│   ├── phase-0.md … phase-8.md, cross-cutting.md
├── boilerplate modules/                     TIER 3 set        — 48 source clones
│   ├── BOILERPLATE_INDEX.md (TIER 1 MOC subfolder; ≤200w; currently ~2,000w — split into MOC + appendix)
│   └── <10 subdirs × source files>
└── gold-standard exemplars/                 TIER 3 set        — per-exemplar cards
    ├── ME-v2.md (≤500w per G4 13-pattern card)
    ├── LCM.md
    ├── ORAC.md
    └── CSv8.md
```

**External MOC reach-out:**
- `~/projects/claude_code/` (main Obsidian vault) has its own `[[ULTRAPLATE Master Index]]` that links workflow-trace as one of 14 services
- `the-workflow-engine-vault/MASTER_INDEX.md` reciprocally links back to the main vault
- Bidi-anchor: both files carry `> Back to:` lines pointing at each other (per workspace CLAUDE.md § Obsidian Vault navigation pattern)

---

## Bidi-anchor discipline (every TIER-N has Back-to TIER-N-1)

Per G5 § Progressive disclosure "Bidi-anchor discipline" line + workspace CLAUDE.local.md § Bidirectional anchors discipline.

### The contract

Every authored markdown file in `the-workflow-engine-vault/` MUST begin (typically line 2 or 3, after title) with a single-line bidi anchor:

```markdown
> Back to: [[parent-doc]] · [[sibling-1]] · [[sibling-2]]
```

Where:
- `parent-doc` is the TIER-N-1 navigation entry that contains a link TO this file (mandatory)
- `sibling-N` are TIER-N peers commonly co-read with this one (recommended, ≤3)

**Round-trip invariant:** if A contains `> Back to: [[B]]`, then B's body MUST contain a `[[A]]` link somewhere. Without bidi, A is an orphan (auto-prunes at next vault sweep per workspace CLAUDE.local.md "orphan docs that auto-prune when their navigation parent rots").

### TIER-by-TIER anchor rules

| Tier | Anchor structure |
|---|---|
| **TIER 1** HOME | no parent (top of vault); peers = MASTER_INDEX, KEYWORDS_20 |
| **TIER 1** MASTER_INDEX | parent = HOME; peers = subfolder MOCs |
| **TIER 1** subfolder MOC | parent = MASTER_INDEX; peers = sibling subfolder MOCs |
| **TIER 2** cluster/phase/integration | parent = subfolder MOC OR MASTER_INDEX; peers = co-clustered TIER-2 docs |
| **TIER 3** module-card | parent = cluster file; peers = adjacent module-cards (m20 ↔ m21 ↔ m22 ↔ m23) |
| **External** main vault link | parent = workspace `[[ULTRAPLATE Master Index]]`; peers = sibling services |

### Bidi-anchor enforcement

Per per-Wave-end sweep (below) — every authored file has its `> Back to:` line checked against its parent's child-link manifest. Mismatch → librarian files drift in `~/projects/shared-context/agent-cross-talk/{TS}_obsidian_vault_librarian_bidi_drift.md`.

---

## On-demand reading order

Per G5 § Progressive disclosure "On-demand reading order" line. The exact navigation sequence a fresh Claude session follows:

```text
[1] Open the-workflow-engine-vault/HOME.md     (TIER 1; always-loaded; ≤200w)
    ├─→ Anchor: HOME has zero parent; lists peers MASTER_INDEX + KEYWORDS_20
    └─→ Content: "what is this; where to go" (single paragraph)

[2] Follow link to MASTER_INDEX.md             (TIER 1; ≤200w MOC body + appendix below fold)
    ├─→ Anchor: parent = HOME
    └─→ Content: MOC table; one row per subfolder + per top-level TIER-2 doc

[3] Decide context-of-need:
    ├─ Building a module?   → MASTER_INDEX → cluster-N.md (TIER 2)
    ├─ Operating a phase?   → MASTER_INDEX → runbook-NN.md (TIER 2)
    ├─ Wiring a tool?       → MASTER_INDEX → INTEGRATION/<tool>.md (TIER 2)
    ├─ Auditing the V7 stack? → MASTER_INDEX → ai_docs/optimisation-v7/TASK_LIST_V7_OPTIMISATION.md (TIER 2)
    └─ Substrate-level query? → MASTER_INDEX → ULTRAMAP.md (TIER 2)

[4] Follow TIER-2 link.                        (~2,500w each)
    ├─→ Anchor: parent = MASTER_INDEX or subfolder MOC; peers = co-clustered docs
    └─→ Content: deep section material; refer to TIER 3 module cards as needed

[5] If editing specific module's src/:
    ├─→ TIER 2 cluster file → TIER 3 module-card (≤500w)
    └─→ Anchor: parent = cluster; peers = adjacent module-cards (m20 ↔ m21 etc)

[6] If cross-vault query (e.g., gold-standard exemplar):
    ├─→ TIER 2 G4-gold-standard.md → TIER 3 gold-standard exemplars/<service>.md
    └─→ External link to main vault [[ULTRAPLATE Master Index]] for cross-service context
```

**Worst case: 3 hops (HOME → MASTER_INDEX → TIER 2).** Average: 2 hops for return visits where MASTER_INDEX is already mental model.

---

## obsidian-vault-librarian per-Wave-end sweep checklist

Per G5 § Progressive disclosure "obsidian-vault-librarian agent runs per-Wave-end sweep" line. The librarian agent (per workspace CLAUDE.md § Habitat Personas — invoked via Agent tool with `obsidian-vault-librarian` subagent type) runs at every Wave-end (Days 3, 12, 21) + every Phase 5C weekly synthesis.

### 4-point checklist

```text
[1] FILE-SIZE INVARIANTS
    ├─ TIER 1 files: each ≤ 200 words
    │   ├─ HOME.md (currently 728w — TRIM target)
    │   ├─ MASTER_INDEX.md (currently ~1,200w — split into ≤200w MOC + appendix)
    │   ├─ KEYWORDS_20.md (currently ~160w — OK)
    │   └─ Each subfolder INDEX.md ≤ 200w
    └─ TIER 2 files: each ≤ 2500 words
        ├─ Cluster A-H specs (currently over — slice TIER 3 cards out)
        ├─ Phase 0-8 runbooks (some over — slice cross-cutting out)
        └─ INTEGRATION/*.md (V7-authored at ≤2500w by spec; verify)
    └─ TIER 3 files: each ≤ 500 words
        └─ Module-cards (not yet authored; budget enforced at author time)

[2] BIDI-ANCHOR ROUND-TRIP
    For every .md file in vault:
      grep "> Back to:" first 10 lines
      For each [[parent]] in anchor: confirm parent .md contains [[this file]] link
      Report MISMATCH per file in librarian drift drop

[3] ORPHAN FILES (no inbound link from any other vault doc)
    rg -l '' the-workflow-engine-vault/*.md the-workflow-engine-vault/**/*.md | while read f; do
      basename=$(basename "$f" .md)
      ! rg -q "\\[\\[$basename\\]\\]" the-workflow-engine-vault/ && echo "ORPHAN: $f"
    done
    Report orphans for Command review (delete or link)

[4] CHILD-LINK MANIFEST PRESENCE
    For every parent doc with subfolder children:
      MOC section MUST list every child file with link
      e.g., MASTER_INDEX lists every TIER 2 doc; module specs/INDEX lists every cluster-N
    Missing entries → librarian drift drop
```

### Sweep output

Single markdown drop at `~/projects/shared-context/agent-cross-talk/{TS}_obsidian_vault_librarian_sweep_wave_{N}.md`:

```text
# Vault sweep — Wave {N} — {TS}

## File-size violations
| File | Tier | Current words | Budget | Action |
|---|---|---|---|---|
| HOME.md | TIER 1 | 728 | ≤200 | TRIM (split details to MASTER_INDEX appendix) |
| ... |

## Bidi-anchor mismatches
| File | Anchor claims parent | Parent missing child link |
|---|---|---|
| cluster-A.md | [[MASTER_INDEX]] | yes — MASTER_INDEX has no [[cluster-A]] link |

## Orphan files
- (list)

## Missing child-link manifests
- (list)

## Summary: PASS | DEGRADED | FAIL
```

DEGRADED = ≤5 violations of any kind. FAIL = >5 OR any orphan. Wave-end checklist (per G4 § Wave-end orchestrator checklist) treats FAIL as gate-block.

---

## File-size invariants (TIER-1 < 200w, TIER-2 < 2500w, TIER-3 < 500w)

Per G5 § Progressive disclosure "File-size invariants" line. Restated with rationale:

| Tier | Word budget | Rationale | Enforcement |
|---|---|---|---|
| TIER 1 | ≤ 200 | always-loaded; total Tier-1 across 3 files = 600w = ~3000 tokens; fits in any session-start context budget | librarian per-Wave-end sweep; author-time `wc -w` discipline |
| TIER 2 | ≤ 2500 | on-demand load; typical sprint loads 2-3 TIER-2 docs concurrently = 5,000-7,500w = ~25k-37k tokens; fits comfortably in any context window | librarian per-Wave-end sweep; over-budget docs split into TIER 3 cards |
| TIER 3 | ≤ 500 | surgical module-card load; typical sprint loads 1-2 cards = 500-1,000w = ~2.5k-5k tokens; minimal context pressure | author-time `wc -w` at card-authoring |

**Token-to-word ratio approximation:** ~1.33 tokens per word for English-with-code (standard CL100k_base ratio per workspace CLAUDE.md context-discipline pattern). Per-tier budgets aim for ≤10% of typical 200k-context-window working set.

**Author-time enforcement script:**
```bash
# scripts/check-vault-sizes.sh (run by librarian; or invoked by author)
for tier in 1 2 3; do
  case $tier in
    1) budget=200;  files="HOME.md MASTER_INDEX.md */INDEX.md" ;;
    2) budget=2500; files="$(find . -path '*module specs*' -o -path '*deployment framework*' -o -path '*RUNBOOKS*' -o -path '*INTEGRATION*' -name '*.md' -not -name 'INDEX.md')" ;;
    3) budget=500;  files="$(find . -name 'm*-overview.md' -o -path '*gold-standard*' -name '*.md')" ;;
  esac
  for f in $files; do
    w=$(wc -w < "$f" 2>/dev/null || echo 0)
    [[ $w -gt $budget ]] && echo "TIER $tier OVER: $f ($w > $budget)"
  done
done
```

---

## MOC + MASTER_INDEX update protocol

Per G5 § Progressive disclosure no explicit MOC update line (V7 expansion). Map-of-Content files are the navigation backbone; updates discipline:

### Protocol

```text
[1] When a new TIER-2 or TIER-3 file is authored:
    ├─ Author the file with `> Back to: [[parent]]` anchor
    ├─ Open parent (typically MASTER_INDEX or subfolder MOC)
    ├─ Add child link in MOC table with one-line description
    ├─ Commit both files in single git commit (atomic discipline)

[2] When a TIER-N file is moved (e.g., promoted from boilerplate to cluster):
    ├─ git mv the file
    ├─ Update old parent (remove child link)
    ├─ Update new parent (add child link)
    ├─ Update file's own `> Back to:` anchor
    ├─ rg -l '[[old-name]]' the-workflow-engine-vault/ — update all back-references
    ├─ Commit atomically

[3] When a TIER-N file is renamed (G2 rename of project):
    ├─ git mv old new
    ├─ rg -l '[[old-name]]' the-workflow-engine-vault/ — search & replace all back-references
    ├─ Commit atomically; verify with full vault sweep

[4] When a file is deleted:
    ├─ rg -l '[[deleted-name]]' the-workflow-engine-vault/ — orphan-link audit
    ├─ Either repoint or remove all back-references
    ├─ git rm + commit atomically
```

### Per-Wave-end MOC drift check

The librarian sweep checks every MOC against its actual subfolder contents:

```bash
# Conceptual check per subfolder MOC
declare -A moc_links
while read -r f; do
  # extract [[link]] entries from MOC
  moc_links["$f"]=$(rg -o '\[\[[^]]+\]\]' "$f")
done < <(find . -name 'INDEX.md' -o -name 'MASTER_INDEX.md')

# Cross-check: every sibling .md should appear in MOC
for moc in "${!moc_links[@]}"; do
  dir=$(dirname "$moc")
  for sibling in "$dir"/*.md; do
    [[ "$sibling" == "$moc" ]] && continue
    basename=$(basename "$sibling" .md)
    if ! echo "${moc_links[$moc]}" | grep -q "\\[\\[$basename"; then
      echo "MOC drift: $moc missing link to [[$basename]]"
    fi
  done
done
```

---

## Failure modes (≥3)

| ID | Failure | Detection | Mitigation |
|---|---|---|---|
| **OBS-01** | Orphan file (authored doc with no inbound link) | librarian per-Wave-end sweep step 3 | author per protocol: every new file requires parent MOC update in same commit; orphan → delete or repoint within 1 Wave |
| **OBS-02** | Bidi-anchor drift (A points at B; B has no link to A) | librarian sweep step 2 | librarian files drift drop; author repoints; gate-block at Wave-end checklist step 8 if violation persists |
| **OBS-03** | TIER 1 over-budget (HOME or MASTER_INDEX grows past 200w) | librarian sweep step 1 | split details into MASTER_INDEX appendix (below-fold) or into subfolder MOC; do NOT inflate TIER 1 |
| **OBS-04** | TIER 2 over-budget (cluster-N.md grows past 2,500w through Wave-2 author addition) | librarian sweep step 1 + author-time `wc -w` | slice TIER 3 module-cards out per cluster; pre-authored card protocol |
| **OBS-05** | MOC drift (new TIER 2 file authored, MASTER_INDEX not updated) | librarian MOC drift check | atomic commit discipline (file + MOC in single commit); per-Wave-end audit |
| **OBS-06** | Cross-vault link rot (main-vault `[[ULTRAPLATE Master Index]]` link breaks after main-vault reorg) | per-Phase 5C weekly synthesis: librarian probes external links | repair link or note external-vault drift; bidi-anchor invariant relaxed for external (cross-vault links cannot be enforced bilaterally) |
| **OBS-07** | G2 rename breaks half the vault links (find/replace incomplete) | post-G2 vault sweep — many bidi mismatches | pre-G2 inventory of all `[[the-workflow-engine` references; post-G2 rg search-and-replace + full sweep before marking G2 green |
| **OBS-08** | Concurrent file edit (two agents both edit cluster-A.md in different worktrees) | merge conflict at Wave-merge | per AGENT_VIEW_GITWORKTREES.md WT-04 file-ownership; vault writes confined to main worktree where possible |

---

## Atuin trajectory

```bash
# Vault sweep history
ls -la ~/projects/shared-context/agent-cross-talk/*obsidian_vault_librarian* | head -20

# Per-Wave-end vault file count
git ls-files the-workflow-engine-vault/ | wc -l                    # baseline
git diff --name-only wave-1-complete..wave-2-complete -- the-workflow-engine-vault/ | wc -l

# File-size trend
for f in $(find the-workflow-engine-vault -name '*.md'); do
  echo "$(wc -w < "$f") $f"
done | sort -rn | head -20

# Orphan file detection
for f in $(find the-workflow-engine-vault -name '*.md'); do
  base=$(basename "$f" .md)
  ! rg -q "\[\[$base\]\]" the-workflow-engine-vault/ && echo "ORPHAN: $f"
done
```

---

## Verification commands

```bash
# Vault file count + size
find the-workflow-engine-vault -name '*.md' | wc -l                # current count
du -sh the-workflow-engine-vault/                                   # total bytes

# TIER 1 budget enforcement
for f in the-workflow-engine-vault/{HOME,MASTER_INDEX}.md \
         the-workflow-engine-vault/**/INDEX.md; do
  [[ -f "$f" ]] && w=$(wc -w < "$f") && echo "$f: $w words (budget 200)"
done

# TIER 2 budget enforcement
find the-workflow-engine-vault \( -path '*module specs*' -o -path '*deployment framework*' -o -path '*RUNBOOKS*' -o -path '*INTEGRATION*' \) -name '*.md' -not -name 'INDEX.md' \
  -exec sh -c 'w=$(wc -w < "$1"); [[ $w -gt 2500 ]] && echo "OVER: $1 ($w > 2500)"' _ {} \;

# Bidi-anchor presence audit
for f in $(find the-workflow-engine-vault -name '*.md'); do
  head -10 "$f" | grep -q '> Back to:' || echo "MISSING ANCHOR: $f"
done

# Cross-vault link probe
rg -n '\[\[ULTRAPLATE Master Index\]\]' the-workflow-engine-vault/

# Librarian sweep dispatch (post-G9; via Agent tool with obsidian-vault-librarian subagent)
# (planning-only; actual dispatch is operational, not specified here)

# G2 rename safety pre-check
rg -l 'the-workflow-engine' the-workflow-engine-vault/ | wc -l     # baseline count for post-G2 diff
```

---

## Sign-off

✅ progressive-disclosure-obsidian spec complete. 3-tier hierarchy (Landing / Per-cluster + Per-phase / Per-module) with explicit word budgets (TIER 1 ≤200w, TIER 2 ≤2500w, TIER 3 ≤500w). Vault structure documented for `~/claude-code-workspace/the-workflow-engine/the-workflow-engine-vault/` (G2-rename-safe). Bidi-anchor discipline specified with `> Back to:` round-trip invariant. On-demand reading order at most 3 hops from HOME. obsidian-vault-librarian per-Wave-end 4-point checklist defined (size + bidi + orphan + child-link manifest). File-size invariants with token-to-word ratio rationale. MOC + MASTER_INDEX update protocol (4 cases: new / move / rename / delete). 8 failure modes with mitigations. Atuin trajectory + verification commands deterministic.

*Authored 2026-05-17 (S1001982) — Command for V7 G5 expansion. Per-Wave-end sweep starts Wave 1 Day 3; weekly synthesis Phase 5C onward. HOLD-v2 respected: planning-only.*
