# ultramap — Code Logic Contextual Flow Map

> **Canonical (authoritative):** [`../ai_docs/optimisation-v7/ULTRAMAP.md`](../ai_docs/optimisation-v7/ULTRAMAP.md) — View 1 (layer) + View 2 (module table)
> **This folder:** dynamic flow maps that complement the canonical structural ULTRAMAP — data flow, control flow, contextual flow, invariant map, dependency graph.
> **Status:** scaffolding-stage. Deep authoring deferred to Wave 2 (parallel `ultramap-deep-author` subagent).

---

## What lives here

`ultramap/` is the **operational view** of how 26 modules behave when wired together. The canonical [`ULTRAMAP.md`](../ai_docs/optimisation-v7/ULTRAMAP.md) gives the **structural view** (what depends on what); this folder gives the **runtime view** (what data flows where, what control transfers when, what invariants must hold).

Compare:

| View | Lives at | Question answered |
|---|---|---|
| Structural | [`../ai_docs/optimisation-v7/ULTRAMAP.md`](../ai_docs/optimisation-v7/ULTRAMAP.md) | What is the dependency graph? |
| Data flow | [`DATA_FLOW.md`](DATA_FLOW.md) (TBD W2) | Where do `ToolCallRow`s travel? |
| Control flow | [`CONTROL_FLOW.md`](CONTROL_FLOW.md) (TBD W2) | When does dispatch fire? |
| Contextual flow | [`CONTEXTUAL_FLOW.md`](CONTEXTUAL_FLOW.md) (TBD W2) | What context attends each emit? |
| Invariant map | [`INVARIANT_MAP.md`](INVARIANT_MAP.md) (TBD W2) | What must always be true? |
| Master | [`ULTRAMAP.md`](ULTRAMAP.md) (TBD W2 — synthesis) | All of the above, indexed |

---

## Files (Wave 2 author target)

| File | Purpose | Status |
|---|---|---|
| `MODULE_DEPENDENCY_GRAPH.md` | Mermaid: full 26-module DAG with cross-cluster edges | TBD W2 |
| `DATA_FLOW.md` | Substrate-in → typed-rows → cluster-B interpretation → m7 hub → m20-m23 iteration → m30 bank → m32 dispatch → m40/m41/m42 emit | TBD W2 |
| `CONTROL_FLOW.md` | Trigger conditions: cron on m1 scan, event on m2 reducer-callback, operator on m23→m30, m32 5-check sequence | TBD W2 |
| `CONTEXTUAL_FLOW.md` | What metadata travels with each row (SessionId, fitness, context-cost band, CC-N marker) | TBD W2 |
| `INVARIANT_MAP.md` | Invariants enforced at compile-time (newtype, namespace.rs constants) + runtime (m9 guard, m32 5-check, EscapeSurfaceProfile ordinal) | TBD W2 |
| `ULTRAMAP.md` | Master synthesis (mirror of canonical + linked flow maps) | TBD W2 |
| `schematics/` | Per-flow Mermaid + sequence diagrams | TBD W2 |

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
3. For runtime behaviour, jump to [`DATA_FLOW.md`](DATA_FLOW.md) → [`CONTROL_FLOW.md`](CONTROL_FLOW.md) → [`CONTEXTUAL_FLOW.md`](CONTEXTUAL_FLOW.md) → [`INVARIANT_MAP.md`](INVARIANT_MAP.md).
4. Use [`ULTRAMAP.md`](ULTRAMAP.md) (this folder's synthesis) when you need everything in one view.

---

> **Back to:** [`../README.md`](../README.md) · [`../ARCHITECTURE.md`](../ARCHITECTURE.md) · [`../ai_specs/INDEX.md`](../ai_specs/INDEX.md) · [`../ai_docs/INDEX.md`](../ai_docs/INDEX.md)
