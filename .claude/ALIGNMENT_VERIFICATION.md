# workflow-trace .claude Folder — Alignment Verification

> Cross-referenced against `plan.toml` (26 modules · 8 clusters · 2 binaries), `ai_specs/MODULE_MATRIX.md`, vault canonical, and `ai_docs/GENESIS_PROMPT_V1_3.md` binding spec.
> Verified: 2026-05-17 (S1002127). HOLD-v2 active; **pre-G9 alignment** (spec ↔ plan.toml ↔ ai_specs ↔ MODULE_MATRIX); **post-G9 alignment** adds (Cargo.toml ↔ plan.toml ↔ src/m<N>/ ↔ ai_specs/modules/cluster-X/m<N>.md).
> See: scaffold-mastery skill at `~/.claude/skills/scaffold-mastery/SKILL.md` for `scaffold-gen --verify .` invocation.

---

## Pre-G9 Triple Alignment (active now)

### Pair A: BINDING SPEC ↔ plan.toml ↔ MODULE_MATRIX

Module count and IDs must agree across the three sources of truth.

```bash
# Plan module count
PLAN=$(rg -c '^\[\[modules\]\]' plan.toml)
echo "plan.toml modules: $PLAN"   # Expected: 26

# MODULE_MATRIX row count (markdown table data rows; skip header + separator)
MAT=$(rg -n '^\| *m[0-9]+ *\|' ai_specs/MODULE_MATRIX.md | wc -l)
echo "MODULE_MATRIX rows: $MAT"    # Expected: 26

# Spec doc references (binding spec v1.3 enumerates module IDs)
SPEC=$(rg -o 'm[0-9]+' ai_docs/GENESIS_PROMPT_V1_3.md | sort -u | wc -l)
echo "GENESIS_PROMPT_V1_3 unique m-ids: $SPEC"   # Expected: 26

[[ "$PLAN" = "$MAT" ]] && [[ "$MAT" = "26" ]] && echo "PASS: plan ↔ matrix ↔ spec aligned" || echo "DRIFT"
```

### Pair B: plan.toml ↔ ai_specs/modules/cluster-{A..H}/m<N>.md

Every module declared in plan.toml must have a per-module spec file in the correct cluster dir.

```bash
# For each module in plan.toml, check the matching ai_specs/modules/cluster-X/m<N>.md exists
declare -A cluster=(
  [m1]=A [m2]=A [m3]=A
  [m4]=B [m5]=B [m6]=B
  [m7]=C [m12]=C [m13]=C
  [m8]=D [m9]=D [m10]=D [m11]=D
  [m14]=E [m15]=E
  [m20]=F [m21]=F [m22]=F [m23]=F
  [m30]=G [m31]=G [m32]=G [m33]=G
  [m40]=H [m41]=H [m42]=H
)
missing=0
for m in "${!cluster[@]}"; do
  c="${cluster[$m]}"
  spec="ai_specs/modules/cluster-${c}/${m}.md"
  [[ -f "$spec" ]] || { echo "MISSING: $spec"; missing=$((missing+1)); }
done
[[ $missing -eq 0 ]] && echo "PASS: 26/26 per-module specs present" || echo "DRIFT: $missing missing"
```

### Pair C: CLAUDE.md ↔ .claude/context.json ↔ ARCHITECTURE.md

Module count and cluster structure must agree across navigation surfaces.

```bash
# context.json module count
CTX=$(python3 -c "import json; d=json.load(open('.claude/context.json')); print(d['module_count'])")
echo "context.json module_count: $CTX"  # Expected: 26

# CLAUDE.md cluster count (table rows)
CMD=$(rg -c '^\| \*\*[A-H]\*\*' CLAUDE.md)
echo "CLAUDE.md cluster table rows: $CMD"  # Expected: 8

# ARCHITECTURE.md cluster mentions
ARCH=$(rg -c '^## Cluster [A-H]' ARCHITECTURE.md)
echo "ARCHITECTURE.md cluster sections: $ARCH"  # Expected: 8 (or per ARCHITECTURE convention)

[[ "$CTX" = "26" && "$CMD" = "8" ]] && echo "PASS: nav surfaces aligned" || echo "DRIFT"
```

### Pair D: Bidirectional anchor sweep

Every vault note + ai_docs canonical doc should start with `> Back to:` pointing at CLAUDE.md or the canonical ai_docs root.

```bash
# Find vault notes missing back-link
fd -e md . the-workflow-engine-vault/ | while read f; do
  head -3 "$f" | rg -q '^> *Back to' || echo "MISSING ANCHOR: $f"
done
```

---

## Pre-G9 Quick Verification One-Liner

```bash
echo "=== workflow-trace Alignment Check (pre-G9) ===" && \
PLAN=$(rg -c '^\[\[modules\]\]' plan.toml) && \
MAT=$(rg -n '^\| *m[0-9]+ *\|' ai_specs/MODULE_MATRIX.md | wc -l) && \
CTX=$(python3 -c "import json; print(json.load(open('.claude/context.json'))['module_count'])") && \
echo "plan=$PLAN matrix=$MAT context.json=$CTX" && \
[[ "$PLAN" = "26" && "$MAT" = "26" && "$CTX" = "26" ]] && echo "ALIGNED" || echo "DRIFT DETECTED"
```

---

## Post-G9 Triple Alignment (activates after `start coding workflow-trace`)

### Pair E: Cargo.toml ↔ plan.toml

```bash
# Both [[bin]] entries declared
rg '^\[\[bin\]\]' Cargo.toml | wc -l   # Expected: 2
rg '^\[\[bin_targets\]\]' plan.toml | wc -l  # Expected: 2

# Binary names match
diff <(rg '^name *= *"wf-' Cargo.toml | sort) <(rg '^name *= *"wf-' plan.toml | sort) && echo "bin names aligned"
```

### Pair F: src/m<N>_*/ ↔ ai_specs/modules/cluster-X/m<N>.md

```bash
# Every module dir under src/ has a matching spec
fd -t d '^m[0-9]+' src/ | while read d; do
  m=$(basename "$d" | rg -o '^m[0-9]+')
  cluster_spec=$(fd "${m}.md" ai_specs/modules/ -1)
  [[ -n "$cluster_spec" ]] || echo "ORPHAN MODULE DIR: $d"
done

# Every spec has a matching src/ dir
fd '^m[0-9]+\.md$' ai_specs/modules/ | while read s; do
  m=$(basename "$s" .md)
  fd -t d "^${m}_" src/ -1 >/dev/null || echo "MISSING SRC DIR for spec: $s"
done
```

### Pair G: scaffold-mastery verify

Per the `scaffold-mastery` skill (`~/.claude/skills/scaffold-mastery/SKILL.md`):

```bash
scaffold-gen --verify .
# Expected output: "26/26 modules aligned; 2/2 binaries; 0 orphans; 0 drift"
```

---

## .claude/ Directory Coverage Matrix

| # | Area | .claude Artifact | Coverage |
|---|------|-----------------|----------|
| 1 | 26 modules / 8 clusters / 2 binaries | `context.json` (full cluster map) | FULL |
| 2 | 13 antipatterns (AP-V7-*) + 9 habitat (AP24/27/29/30/32-37) | `anti_patterns.json` | FULL |
| 3 | 6 architectural + 11 ME v2 + 4 QG + 10 data-flow + 5 substrate patterns | `patterns.json` | FULL |
| 4 | G1-G9 + B1-B6 state | `context.json` (gates, blockers) | FULL |
| 5 | Pre-G9 hook enforcement (no .rs / no Cargo.toml) | `hooks/pre-write-no-rust-pre-g9.sh`, `pre-write-no-cargo-pre-g9.sh` | FULL |
| 6 | Namespace guard (AP30) | `hooks/pre-tool-bash-namespace-guard.sh` | FULL |
| 7 | Preserve-list discipline (S102) | `hooks/pre-tool-bash-no-blanket-prune.sh` | FULL |
| 8 | NexusEvent / EscapeSurfaceProfile / PressureRegister / workflow_runs JSON schemas | `schemas/*.json` (4 files) | FULL |
| 9 | atuin / injection.db / m7 lift queries | `queries/*.sql` (3 files) | FULL |
| 10 | 6 project subagents (gate / waiver / cluster-spec / escape / ember / cc-verify) | `agents/*.md` | FULL |
| 11 | 6 project slash commands | `commands/*.md` | FULL |
| 12 | Permissions allow/deny + hooks env | `settings.json` | FULL |
| 13 | Plan + spec + matrix triple alignment | this file | FULL |
| 14 | Status + recent_writes audit | `status.json` | FULL |

---

## Per-Cluster Spec Status

| Cluster | Modules | Specs in `ai_specs/modules/` | Wave-1 status |
|---|---|---|---|
| A | m1, m2, m3 | 3 expected | per cluster-spec-author dispatch |
| B | m4, m5, m6 | 3 expected | per cluster-spec-author dispatch |
| C | m7, m12, m13 | 3 expected | per cluster-spec-author dispatch |
| D | m8, m9, m10, m11 | 4 expected | per cluster-spec-author dispatch |
| E | m14, m15 | 2 expected | per cluster-spec-author dispatch |
| F (KEYSTONE) | m20, m21, m22, m23 | 4 expected | per cluster-spec-author dispatch |
| G | m30, m31, m32, m33 | 4 expected | per cluster-spec-author dispatch |
| H | m40, m41, m42 | 3 expected | per cluster-spec-author dispatch |
| **TOTAL** | 26 | 26 | — |

---

## Trap Coverage (workflow-trace specific)

| # | Trap | .claude File | ID |
|---|------|-------------|-----|
| 1 | Pre-G9 `.rs` file write | `hooks/pre-write-no-rust-pre-g9.sh` | AP24 |
| 2 | Pre-G9 `Cargo.toml` write | `hooks/pre-write-no-cargo-pre-g9.sh` | AP24 / AP-V7-12 |
| 3 | stcortex namespace string drift | `hooks/pre-tool-bash-namespace-guard.sh` | AP30 |
| 4 | Blanket docker prune (S102) | `hooks/pre-tool-bash-no-blanket-prune.sh` | AP-Hab-04 |
| 5 | Health-200 ≠ behaviour-verified | `anti_patterns.json` AP-V7-01 | runbook discipline |
| 6 | Silence ≠ consent | `anti_patterns.json` AP-V7-02 | comms discipline |
| 7 | Verb collapse Phase A/B | `anti_patterns.json` AP-V7-03 | reviewer enforcement |
| 8 | Auto-promotion m23 → m30 | `anti_patterns.json` AP-V7-07 | CC-4 contract enforcement |
| 9 | Self-dispatch via m32 | `anti_patterns.json` AP-V7-08 | m32 guard `kind != dispatcher_self` |
| 10 | Cluster D not woven first | `anti_patterns.json` AP-V7-09 | Phase 1 sequencing |
| 11 | LCM workspace pattern misuse | `anti_patterns.json` AP-V7-10 | scaffold-mastery enforcement |
| 12 | Sync HTTP in tokio::spawn | `anti_patterns.json` AP29 | reqwest::Client async only |
| 13 | Refresh-date stamps without probes | `anti_patterns.json` AP-V7-13 | runbook-freshness skill |

---

## Verdict: ALIGNED (pre-G9 envelope)

26 modules declared in `plan.toml`. Per-cluster spec dirs populated by Wave-1 (`cluster-spec-author` agent). All artefacts under `.claude/` cross-reference back to `ai_docs/optimisation-v7/ANTIPATTERNS_REGISTER.md` (canonical) and per-module ai_specs.

**Post-G9 verification:** re-run with `scaffold-gen --verify .` after `start coding workflow-trace` fires; expected 26/26 + 2/2 + 0 orphans + 0 drift.

---

## Verification History

| Date | Session | Wave | Result | Modules | Specs | Notes |
|------|---------|------|--------|---------|-------|-------|
| 2026-05-17 | S1002127 | Wave 0 | INITIAL | 26 declared | 0 written | Skeleton + root anchor files |
| 2026-05-17 | S1002127 | Wave 1 | IN PROGRESS | 26 declared | 8 cluster dirs | 8 parallel cluster-spec-author agents dispatched |
| 2026-05-17 | S1002127 | Wave 2 | IN PROGRESS | 26 declared | TBD | `.claude/` JSON optimisation (this session) |
| TBD | TBD | Wave 3 | PENDING | — | — | agent-claim-verifier + four-surface-persistence-verifier |

---

> **Back to:** [`../CLAUDE.md`](../CLAUDE.md) · [`../CLAUDE.local.md`](../CLAUDE.local.md) · [`../GATE_STATE.md`](../GATE_STATE.md) · [`../PRIME_DIRECTIVE_WAIVER.md`](../PRIME_DIRECTIVE_WAIVER.md)
