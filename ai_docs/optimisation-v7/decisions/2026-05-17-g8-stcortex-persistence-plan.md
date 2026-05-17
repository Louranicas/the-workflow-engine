---
title: ADR — G8 stcortex persistence plan for workflow-trace scaffold
date: 2026-05-17
status: PROPOSED (awaits G8-green to fire)
authors: Command (Tab 1 Orchestrator top-left), four-surface-persistence-verifier (Wave 3 dispatch)
session: S1002127
gates_required: G8 (Persistence — 4-surface plan crystallisation)
supersedes: none
authorising_session: S1002127 PRIME_DIRECTIVE_WAIVER scaffold-only override
audit_lane: Zen G7 (verdict-pending on v1.3; this ADR follows D-B6 AMEND-loop if Zen objects)
---

# ADR — G8 stcortex persistence plan for workflow-trace scaffold

> **Back to:** [`../../../CLAUDE.md`](../../../CLAUDE.md) · [`../../../CLAUDE.local.md`](../../../CLAUDE.local.md) · [`../../../GATE_STATE.md`](../../../GATE_STATE.md) · [`../../../PRIME_DIRECTIVE_WAIVER.md`](../../../PRIME_DIRECTIVE_WAIVER.md) · [`../../../plan.toml`](../../../plan.toml) · vault [[Scaffold Wave 0-2 — Session S1002127]]
> **Companion ADR:** [`2026-05-17-m42-stcortex-only-pivot.md`](2026-05-17-m42-stcortex-only-pivot.md) (decouples workflow-trace from POVM; this ADR is consistent with that pivot)
> **Surfaces this ADR governs:** Surface 3 of the 4-surface persistence rule (stcortex namespace `workflow_trace_*`)

---

## § 0 — Context

The S1002127 scaffold persists across the **four surfaces** declared in workspace [`CLAUDE.local.md`](../../../../CLAUDE.local.md) Working Mode:

1. **`ai_docs/` canonical** — present (LIVE; Waves 0+2B+2C cover it)
2. **Obsidian vault mirror** — present (LIVE; pre-existing 88 vault files + Wave-2E adds 9 new + 2 updates + 6 audits)
3. **stcortex `workflow_trace_*` namespace** — **RESERVED, NOT WRITTEN** pre-G8 per [`GATE_STATE.md`](../../../GATE_STATE.md) G8 NOT GREEN
4. **`CLAUDE.local.md` anchor** — present (LIVE; project-local + workspace anchors)

The Wave-3 `four-surface-persistence-verifier` flagged that Surface 3 has **no documented post-G8 write-plan** — the future writes themselves are not pre-specified, so G8-fire would be interpretive rather than mechanical. This ADR closes that gap by pre-specifying the writes that fire when G8 goes green.

---

## § 1 — Decision

When Luke fires G8 (`Persistence — 4-surface plan crystallisation`), Command (or an authorised stcortex-implementer agent) writes the following memories + pathways under namespace `workflow_trace_*` in a single atomic batch.

### § 1.a — Namespace shape (locked)

| Prefix | Purpose | Cardinality (anticipated) |
|---|---|---|
| `workflow_trace_scaffold_s1002127` | Genesis memory for the S1002127 scaffold session | 1 |
| `workflow_trace_module_m<N>` | Per-module spec anchors (m1, m2, m3, m4, m5, m6, m7, m8, m9, m10, m11, m12, m13, m14, m15, m20, m21, m22, m23, m30, m31, m32, m33, m40, m41, m42) | 26 |
| `workflow_trace_cluster_<a..h>` | Per-cluster spec anchors | 8 |
| `workflow_trace_decision_<adr-slug>` | ADR anchors (m42 pivot, this ADR, future ADRs) | growing (≥2 at G8-fire) |
| `workflow_trace_gate_<g1..g9>` | Gate-state snapshots at each gate-fire moment | 9 (filled progressively as gates fire) |
| `workflow_trace_meta` | Cross-reference index memory | 1 |
| `workflow_trace_synergy_cc<1..7>` | CC contract anchors (CC-1, CC-1.subA, CC-2, …, CC-7) | 8 |
| **Total at G8-fire** | | **~46 memories + pathways below** |

### § 1.b — Minimum 4 memories at G8-fire (atomic batch)

| # | Memory id | tensor | modality | Content head | Reverse-anchors embedded |
|---|---|---|---|---|---|
| 1 | `workflow_trace_scaffold_s1002127` | 0.95 | `meta` | "S1002127 scaffold-wave-0-1-2 complete (~163 files / ~140k words; HOLD-v2 respected; G1-G9 still gating G9)" | `ai_docs:CHANGELOG.md` + `vault:[[Scaffold Wave 0-2 — Session S1002127]]` + `claude_local:## S1002127 — Scaffold Waves 0+1+2 (LIVE)` |
| 2 | `workflow_trace_decision_v1_3_binding` | 0.90 | `spec` | GENESIS_PROMPT_V1_3 spec hash + canonical path | `ai_docs:GENESIS_PROMPT_V1_3.md` + `vault:[[Genesis Prompt v1.2 S1001982]]` (superseded notice) |
| 3 | `workflow_trace_decision_prime_directive_waiver_s1002127` | 0.85 | `governance` | Verbatim Luke S1002127 prompt + scope tight/wide table | `ai_docs:PRIME_DIRECTIVE_WAIVER.md` |
| 4 | `workflow_trace_gate_g8` | 0.95 | `gate` | Gate snapshot at G8-fire moment (G1-G7 state, B1-B6 remaining, substrate condition) | `ai_docs:GATE_STATE.md` + `claude_local:## Active Workstreams workflow-trace row` |

### § 1.c — Pathways (bidirectional weights)

| Pre-id | Post-id | Weight | Direction | Rationale |
|---|---|---|---|---|
| `workflow_trace_scaffold_s1002127` | `workflow_trace_decision_v1_3_binding` | 0.95 | bidi | scaffold instantiates the binding spec |
| `workflow_trace_decision_v1_3_binding` | `workflow_trace_cluster_<a..h>` | 0.85 | bidi (× 8) | binding spec specifies 8 clusters |
| `workflow_trace_cluster_<X>` | `workflow_trace_module_m<N>` | 0.80 | bidi (per cluster-module pair, 26 total) | cluster contains modules |
| `workflow_trace_module_m<N>` | `workflow_trace_synergy_cc<K>` | 0.75 | bidi (multi-edge per CC contract) | modules participate in CC contracts |
| `workflow_trace_decision_m42_pivot` | `workflow_trace_module_m42` | 0.95 | bidi | m42 ADR governs m42 spec |
| `workflow_trace_decision_g8_persistence` | `workflow_trace_meta` | 0.90 | bidi | this ADR governs the persistence index |
| `workflow_trace_gate_g<N>` | `workflow_trace_gate_g<N+1>` | 0.80 | forward-only | gate sequencing (G1 → G2 → … → G9) |
| `workflow_trace_meta` | every namespace anchor above | 0.50 | forward | cross-reference index |

**Total pathways at G8-fire: ~50 bidi + ~10 forward-only = ~60 edges.**

### § 1.d — Reverse-anchor embedding rule (mandatory per memory)

Every `content` field MUST embed three strings (separator: ` ; `):
- `ai_docs:<absolute path under ai_docs/>`
- `vault:[[<vault note name>]]`
- `claude_local:<heading-anchor under CLAUDE.local.md>`

This makes recovery round-trip: if surfaces 1, 2, 4 vanish, an SQL query against stcortex memories yields the paths to re-author them.

---

## § 2 — Verification (post-G8 fire)

Run `stcortex-reviewer` agent to verify:
1. All 46 memories present at correct ids
2. All ~60 pathways present at correct weights
3. Reverse-anchor embedding holds on every memory
4. `stcortex inspect workflow_trace_*` reports clean
5. `agent-claim-verifier` re-run sees Surface 3 score rise from 0.6 (RESERVED) to 1.0 (LIVE)

Filed at: `~/projects/shared-context/agent-cross-talk/<utc>_stcortex_reviewer_workflow_trace_g8_persistence_verify.md` (post-G8-fire).

---

## § 3 — Non-decisions (explicitly out of scope)

- **POVM mirror writes:** NOT performed. m42 stcortex-only ADR (2026-05-17) decoupled workflow-trace from POVM; G8 persistence writes are stcortex-only.
- **Pre-G8 writes:** REFUSED at DB layer (stcortex refuse-write); the namespace MAY be probed for emptiness pre-G8 (read-only).
- **Future-gate snapshots beyond G8:** G9, G10+ snapshots are written when those gates fire, not at G8.
- **Workspace-root CLAUDE.local.md amendment:** project charter forbids; Luke action required separately if workspace anchor needs update.

---

## § 4 — Risk surface

| Risk | Probability | Mitigation |
|---|---|---|
| Pre-G8 accidental write attempt | low | DB-layer refuse-write + `.claude/hooks/pre-tool-bash-namespace-guard.sh` |
| stcortex `:3000` unreachable at G8-fire | low | Offline JSONL snapshot at `stcortex/data/snapshots/` per workspace CLAUDE.md memory row 8; retry on connection restore |
| Namespace-name collision with another project | low | `workflow_trace_*` prefix is project-unique; verify before write |
| Pathway weight drift over time (Hebbian decay) | medium | Pathways reinforced on every per-module spec read post-G8 (CC-5 substrate-learning loop) |
| Reverse-anchor strings rot (file moves) | medium | `four-surface-persistence-verifier` agent re-runs on every session boundary; broken anchors → Class-I Watcher flag |

---

## § 5 — Status flow

| State | Trigger | Action |
|---|---|---|
| PROPOSED | This ADR filed at G7 audit | Awaits Zen G7 verdict on v1.3 amendment (which this ADR is consistent with) |
| RATIFIED | Zen G7 ACCEPT + Luke G8 signal | Command executes § 1.b + § 1.c atomically |
| LIVE | All 46 memories + 60 pathways present, `stcortex-reviewer` confirms | Surface 3 score rises 0.6 → 1.0 |
| MAINTAINED | Continuous | `four-surface-persistence-verifier` on session boundaries; CC-5 substrate-learning loop reinforces weights |

---

## § 6 — Decision register entry

This ADR is logged in [`../DECISION_REGISTER.md`](../DECISION_REGISTER.md) as decision D-S1002127-01 (G8 stcortex persistence plan). It is the **62nd decision** in the V7 register (13 V7 + 48 grilling + 1 m42 pivot ADR + this ADR).

---

> **Back to:** [`../../../CLAUDE.md`](../../../CLAUDE.md) · [`../../../CLAUDE.local.md`](../../../CLAUDE.local.md) · [`../../../GATE_STATE.md`](../../../GATE_STATE.md) · [`../../../PRIME_DIRECTIVE_WAIVER.md`](../../../PRIME_DIRECTIVE_WAIVER.md) · [`../../../plan.toml`](../../../plan.toml) · [`../DECISION_REGISTER.md`](../DECISION_REGISTER.md) · companion [`2026-05-17-m42-stcortex-only-pivot.md`](2026-05-17-m42-stcortex-only-pivot.md) · vault [[Scaffold Wave 0-2 — Session S1002127]]

*ADR last updated: 2026-05-17 S1002127 (Wave 3 closure)*
