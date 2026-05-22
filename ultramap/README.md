# ultramap — Code Logic Contextual Flow Map

> **Canonical (authoritative):** [`../ai_docs/optimisation-v7/ULTRAMAP.md`](../ai_docs/optimisation-v7/ULTRAMAP.md) — View 1 (layer) + View 2 (module table)
> **API reference:** [`../ai_docs/API_MAP.md`](../ai_docs/API_MAP.md) — complete `workflow_core` public surface
> **This folder:** dynamic flow maps that complement the canonical structural ULTRAMAP — data flow, control flow, contextual flow, invariant map, dependency graph, and the two **runtime binary-pipeline maps**.
> **Status:** flow maps authored; the two binary-pipeline maps added post-S1003733 (binaries wired via `workflow_core::orchestration`).

---

## What lives here

`ultramap/` is the **operational view** of how 26 modules behave when wired together. The canonical [`ULTRAMAP.md`](../ai_docs/optimisation-v7/ULTRAMAP.md) gives the **structural view** (what depends on what); this folder gives the **runtime view** (what data flows where, what control transfers when, what invariants must hold).

Compare:

| View | Lives at | Question answered |
|---|---|---|
| Structural | [`../ai_docs/optimisation-v7/ULTRAMAP.md`](../ai_docs/optimisation-v7/ULTRAMAP.md) | What is the dependency graph? |
| Data flow | [`DATA_FLOW.md`](DATA_FLOW.md) | Where do typed rows travel? |
| Control flow | [`CONTROL_FLOW.md`](CONTROL_FLOW.md) | When does dispatch fire? |
| Contextual flow | [`CONTEXTUAL_FLOW.md`](CONTEXTUAL_FLOW.md) | What context attends each emit? |
| Invariant map | [`INVARIANT_MAP.md`](INVARIANT_MAP.md) | What must always be true? |
| **wf-crystallise pipeline** | [`WF_CRYSTALLISE_PIPELINE.md`](WF_CRYSTALLISE_PIPELINE.md) | What runs, in order, on `wf-crystallise`? |
| **wf-dispatch pipeline** | [`WF_DISPATCH_PIPELINE.md`](WF_DISPATCH_PIPELINE.md) | What runs, in order, on `wf-dispatch`? |
| Master | [`ULTRAMAP.md`](ULTRAMAP.md) | All of the above, indexed |

---

## Files

| File | Purpose |
|---|---|
| `MODULE_DEPENDENCY_GRAPH.md` | Mermaid: full 26-module DAG with cross-cluster edges |
| `DATA_FLOW.md` | Substrate-in → typed-rows → cluster-B interpretation → m7 hub → m20-m23 iteration → m30 bank → m32 dispatch → m40/m41/m42 emit |
| `CONTROL_FLOW.md` | Trigger conditions: cron on m1 scan, event on m2 reducer-callback, operator on m23→m30, m32 5-check sequence |
| `CONTEXTUAL_FLOW.md` | What metadata travels with each row (SessionId, fitness, context-cost band, CC-N marker) |
| `INVARIANT_MAP.md` | Invariants enforced at compile-time (newtype, namespace constants) + runtime (m9 guard, m32 5-check, EscapeSurfaceProfile ordinal) |
| **`WF_CRYSTALLISE_PIPELINE.md`** | **Runtime stage flow of the `wf-crystallise` binary** — Mermaid flowchart of the 9-stage observe→mine→propose sequence + prose; driver `orchestration::crystallise::run` |
| **`WF_DISPATCH_PIPELINE.md`** | **Runtime stage flow of the `wf-dispatch` binary** — Mermaid flowchart of the bank→select→verify→dispatch sequence + prose; driver `orchestration::dispatch::run` |
| `ULTRAMAP.md` | Master synthesis (mirror of canonical + linked flow maps) |
| `schematics/` | Per-flow Mermaid + sequence diagrams |

---

## Bridge to canonical

Canonical V7 ULTRAMAP contains:
- **View 1 (Layer)** — L0 substrate → L1 ingest → L2 observation → L3 correlation → L4 trust (aspect) → L5 evidence → L6 iteration → L7 bank/dispatch → L8 feedback
- **View 2 (Module Table)** — 26 rows × architectural columns (module name, layer, LOC, tests, owns, depends, feature, verb-class, CC contracts, gap-owner)

This folder ADDS:
- **View 3 (Data Flow)** — what rows/structs travel which edges (Mermaid + struct names)
- **View 4 (Control Flow)** — when each edge fires (triggers, timers, events)
- **View 5 (Contextual Flow)** — what metadata attends each emit
- **View 6 (Invariant Map)** — what must hold at every snapshot

---

## How to read this folder

1. Start at canonical [`../ai_docs/optimisation-v7/ULTRAMAP.md`](../ai_docs/optimisation-v7/ULTRAMAP.md) for the structural map.
2. Then [`MODULE_DEPENDENCY_GRAPH.md`](MODULE_DEPENDENCY_GRAPH.md) for the visual DAG.
3. For the **runtime binary behaviour**, jump to [`WF_CRYSTALLISE_PIPELINE.md`](WF_CRYSTALLISE_PIPELINE.md) and [`WF_DISPATCH_PIPELINE.md`](WF_DISPATCH_PIPELINE.md) — what runs, in order, when each binary is invoked.
4. For cross-module behaviour, jump to [`DATA_FLOW.md`](DATA_FLOW.md) → [`CONTROL_FLOW.md`](CONTROL_FLOW.md) → [`CONTEXTUAL_FLOW.md`](CONTEXTUAL_FLOW.md) → [`INVARIANT_MAP.md`](INVARIANT_MAP.md).
5. Use [`ULTRAMAP.md`](ULTRAMAP.md) (this folder's synthesis) when you need everything in one view; [`../ai_docs/API_MAP.md`](../ai_docs/API_MAP.md) for the per-function API reference.

---

> **Back to:** [`../README.md`](../README.md) · [`../ARCHITECTURE.md`](../ARCHITECTURE.md) · [`../ai_specs/INDEX.md`](../ai_specs/INDEX.md) · [`../ai_docs/INDEX.md`](../ai_docs/INDEX.md)
