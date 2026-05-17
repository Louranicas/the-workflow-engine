# synergies/ — Cross-Cluster Synergy Contracts

> **Back to:** [`../INDEX.md`](../INDEX.md) · [`../MODULE_MATRIX.md`](../MODULE_MATRIX.md) · [`../../README.md`](../../README.md) · [`../../ARCHITECTURE.md`](../../ARCHITECTURE.md) · canonical [`../../ai_docs/optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md`](../../ai_docs/optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md)

## The 7 CC contracts

| CC | File | Path | Watcher class pre-position |
|---|---|---|---|
| CC-1 | [`CC-1.md`](CC-1.md) | Cascade-Cost Coupling (B internal via m7 JSONB) | Class D, Class G |
| CC-2 | [`CC-2.md`](CC-2.md) | Trust Layer Woven (D → all) | Class A, Class D |
| CC-3 | [`CC-3.md`](CC-3.md) | Evidence-Driven Iteration (E → F) | Class C, Class A |
| CC-4 | [`CC-4.md`](CC-4.md) | Proposal → Bank → Dispatch (F → G → Conductor) | Class A, Class B, Class C |
| CC-5 | [`CC-5.md`](CC-5.md) | Substrate Learning Loop (G → H → F) — **SPECIAL DEPTH** | **Class I (primary)**, Class A, Class B, Class D |
| CC-6 | [`CC-6.md`](CC-6.md) | Verification-Gated Dispatch (G-internal m33→m32) | Class C, Class A |
| CC-7 | [`CC-7.md`](CC-7.md) | Pressure-Driven Evolution (E → operator deliberation) | Class C, Class G, Class E |

## CC-1b resolution decision

**Decision: CC-1b is a sub-contract of CC-1 (CC-1.subA), NOT a net-new 8th contract.**

The Cluster B agent surfaced a putative new contract `CC-1b (m5↔m6)` covering the battern-cost coupling via shared `session_id` within `battern_id` range. On review of the per-module specs (`m5_battern_step_record.md` line 222; `m6_context_cost.md` line 286), CC-1b describes the **same structural coupling pattern** as CC-1:

- **Same coupling discipline:** m5 and m6 NEVER directly call each other; they join through m7's JSONB `consumer_inputs` column via `(battern_id, session_id)` tuples.
- **Same join surface:** m7's stable JSONB column is the persistent coupling surface for both m4↔m6 (CC-1) and m5↔m6 (CC-1.subA).
- **Same closure-test category:** `tests/integration/cc1_*.rs` covers both — adding `cc1b_*.rs` would duplicate test infrastructure for a sub-pattern, not a new contract.
- **Same Watcher pre-position:** Class D (four-surface drift) fires on any drift in m7's JSONB schema regardless of which Cluster B emitter is the source.

The Cluster B vault spec, V7 `CROSS_CLUSTER_SYNERGIES.md`, and GOD_TIER_CONSOLIDATION all enumerate **7 CCs**, not 8. Promoting CC-1b to CC-8 would diverge from the canonical 7-CC list maintained across vault + V7 + ai_docs, introducing AP-V7-02 (Ultramap rot) — different surfaces would carry different CC counts.

**Documentation location:** [`CC-1.md`](CC-1.md) § CC-1.subA. The sub-pattern is enumerated with its own data-flow diagram, modules-involved subsection, and Watcher class pre-position, all under the CC-1 contract umbrella. The per-module specs' existing CC-1b references remain valid as shorthand for "CC-1.subA".

**Recorded by:** Command (this Wave) at 2026-05-17 during root-level spec authoring. Per V7 D-B6 AMEND-loop: if Zen G7 audit disagrees, this decision is overturned and CC-8 becomes the contract name. No code is affected (planning-only); the decision lives in markdown only.

---

> **Back to:** [`../INDEX.md`](../INDEX.md) · canonical [`../../ai_docs/optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md`](../../ai_docs/optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md)
