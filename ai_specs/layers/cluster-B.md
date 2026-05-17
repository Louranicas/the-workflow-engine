---
title: Cluster B — Habitat Observation (Layer L2) — Layer Spec
cluster: B
layer: L2
module_count: 3
modules: [m4_cascade_correlator, m5_battern_step_record, m6_context_cost]
binary: wf-crystallise
feature_gates: [none]
cc_owns: [CC-1.subA (m5↔m6 via m7 JSONB battern_id↔session_id)]
cc_consumes: [CC-1, CC-2, CC-3]
ship_priority: Day 1 Wave 2 (after Cluster A)
status: SPEC
date: 2026-05-17
hold_v2_compliant: true
---

# Cluster B — Habitat Observation (L2)

> **Back to:** [`../INDEX.md`](../INDEX.md) · [`../MODULE_MATRIX.md`](../MODULE_MATRIX.md) · [`../../README.md`](../../README.md) · [`../../ARCHITECTURE.md`](../../ARCHITECTURE.md) · vault [[cluster-B-habitat-observers]] · [`../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-B.md`](../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-B.md)

## Role

Cluster B IS the **habitat-observation layer** — three modules that take typed Cluster A iterators and classify, correlate, and quantify them into the four observation primitives downstream Clusters E/F/G/H reason over: **cascade clusters** (m4), **battern step trajectories** (m5), and **context-cost EMA bands** (m6). The classification is observational only — Cluster B never names a pattern with a human-meaningful label (F11 cascade-monoculture mitigation), never proposes, never selects, never dispatches. Cluster B's job is to turn raw substrate rows into typed observation records that m7 can join and that Cluster E (m14, m15) can derive evidence and pressure from.

The three modules read one Cluster A iterator each and emit one observation primitive each. **m4** consumes `m1::ToolCallRow` and emits `CascadeCluster { cluster_id: OpaqueId, session_id, step_count, fnv_xor_signature }` — the `cluster_id` is opaque per F11 (FNV-1a XOR over normalised step tokens; never a label like `"deploy-dance"`). **m5** consumes `m1::ToolCallRow` and emits `BatternStepObservation { battern_id, session_id, step_index, step_token }` — the battern boundary is detected via Battern protocol step-markers, not heuristic. **m6** consumes `m1::ToolCallRow` and emits `ContextCostBand { session_type, ema_mean, ema_variance, n }` — the EMA is computed per session-type with the gradient-preservation rule (per Genesis v1.3 § 3 m6 row) and never collapses exploration-cost variance into a single point estimate (F10 mitigation).

## Modules

| Module | Spec | LOC | Tests | Reads | Emits |
|---|---|---:|---:|---|---|
| `m4_cascade_correlator` | [`../modules/cluster-B/m4_cascade_correlator.md`](../modules/cluster-B/m4_cascade_correlator.md) | 160 | 60 | m1 iter | `CascadeCluster` |
| `m5_battern_step_record` | [`../modules/cluster-B/m5_battern_step_record.md`](../modules/cluster-B/m5_battern_step_record.md) | 130 | 60 | m1 iter | `BatternStepObservation` |
| `m6_context_cost` | [`../modules/cluster-B/m6_context_cost.md`](../modules/cluster-B/m6_context_cost.md) | 170 | 60 | m1 iter | `ContextCostBand` |

## Cross-cluster contracts

- **OWNS:**
  - **CC-1.subA (Battern-Cost Coupling)** — m5's `(battern_id, session_id)` and m6's `(session_id, ema_band)` join through m7's JSONB `consumer_inputs` for cost-per-step approximations. **The sub-A scoping is decided per the synergies/README.md resolution: CC-1b is a sub-pattern of CC-1, not a net-new 8th contract** — same join surface (m7 JSONB), same `accept m4 + m6 schemas via shared persistent column` discipline. See [`../synergies/CC-1.md`](../synergies/CC-1.md) § CC-1.subA.
- **CONSUMES:**
  - **CC-1** — m4's `CascadeCluster` is the canonical input to m7's `consumer_inputs` JSONB column alongside m6's `ContextCostBand`; the join surface is owned by m7 (Cluster C).
  - **CC-2** — m9 namespace-guard wraps any string emission that lands in m13/m42; cluster-internal types use newtype IDs (`OpaqueId`, `SessionId`, `BatternId`) without literal namespace strings.
  - **CC-3** — m4 + m5 + m6 outputs are the trajectory inputs that m14 (Cluster E) computes Wilson-CI Lift over for downstream Cluster F evidence-gating.

## Binary placement

All three modules live in **`wf-crystallise`**. m4/m5/m6 are observation modules; observation belongs in the ingest binary, not the dispatcher.

## Feature-gate posture

**Un-gated** (`feature = "none"`). Observation is mandatory infrastructure; gating m4 would mean a build profile in which the engine cannot see cascades. Genesis v1.3 § 1 locks Cluster B as default-on.

## Ship priority

**Day 1 Wave 2** (after Cluster A reads land). Implementation order: m4 → m6 → m5 (m4 publishes the cascade-id schema first; m6 publishes the cost-band schema before m5 since m5's tests reference both for CC-1.subA join contract assertions).

## Operational invariants

1. **Opaque IDs only (F11 mitigation).** `CascadeCluster.cluster_id` is `FNV-1a(canonical_step_token_sequence)` rendered as 16-hex lowercase, NEVER a name like `"deploy-dance"`. Test invariant: `rg '"[a-z]+-[a-z]+-[a-z]+"' src/m4_cascade_correlator/` returns 0 string-literals matching the cascade-name shape outside `tests/`.
2. **No direct intra-cluster calls.** m4 does not call m5; m5 does not call m6. Cross-module observation joins happen through m7's JSONB column (CC-1 + CC-1.subA). Intra-cluster invocation is rejected at code review and caught by verify-sync invariant #2 (cluster-internal use-graph audit).
3. **EMA gradient preservation (F10 mitigation).** m6's EMA carries both `ema_mean` and `ema_variance`. Collapsing to a single scalar is an F10 violation — the gradient (variance) is what downstream m20-m23 use to weight exploration vs exploitation.
4. **Battern boundaries are protocol-defined, not heuristic.** m5 detects battern start/end via the Battern protocol's explicit step-marker tokens (`battern_begin`, `battern_end`); it does not heuristically chunk based on time gaps or command similarity. Heuristic chunking is a heuristic-as-truth antipattern and rejected.
5. **No labels survive the cluster.** Any human-readable string a Cluster A module might carry (e.g., `command: String`) is hashed or summarised before crossing the Cluster B → C boundary. The `cluster_id` and `battern_id` newtypes are the canonical handles.

## Failure modes the cluster structurally refuses

- **Cascade label collapse.** Naming a cascade-cluster `"deploy-dance"` would let downstream m23 emit a proposal with a human-meaningful label — banned per F11.
- **EMA single-scalar collapse.** Returning `ContextCostBand { ema: f64 }` instead of `{ ema_mean, ema_variance, n }` would erase the gradient m22's K-means feature-clusterer relies on.
- **Direct m5 → m6 invocation.** `m5::record_step()` calling `m6::lookup_cost(session_id)` would create the cross-coupling spaghetti CC-1.subA's "join through m7 JSONB" discipline explicitly prevents.
- **Inline `INSERT` into m7.** m4/m5/m6 NEVER write to m7's SQLite directly; they return `Vec<Observation>` and m7's own write API (`workflow_arc_record`) does the insert. Cluster B is emit-only.

## Performance envelope

| Operation | Target | Notes |
|---|---|---|
| m4 `correlate(batch_size=2000)` | < 30 ms | FNV-1a XOR is O(n); single pass over the page |
| m5 `record_steps(batch)` | < 20 ms | step-marker parsing is regex-free (token equality) |
| m6 `update_ema(row)` | < 5 µs / row | parking_lot lock + arithmetic only |
| m6 EMA convergence | < 100 rows | `α = 2 / (n + 1)` with `n=100` half-life |

## Verify-sync invariants

- **#2** — every src/ module in Cluster B has an entry in [`../modules/cluster-B/`](../modules/cluster-B/) (3 entries, all present).
- **#4** — `cargo test --package workflow_trace --no-run --message-format=json` per module returns ≥60 tests for each.
- **#8** — `rg '\.unwrap\(\)' src/m4_cascade_correlator/ src/m5_battern_step_record/ src/m6_context_cost/ | grep -v 'tests/'` returns 0.
- **#16** — F11 mitigation tests assert no human-readable string lands in `CascadeCluster.cluster_id`.

## Per-module cross-links

- [`../modules/cluster-B/m4_cascade_correlator.md`](../modules/cluster-B/m4_cascade_correlator.md) — FNV-1a XOR opaque-id correlator
- [`../modules/cluster-B/m5_battern_step_record.md`](../modules/cluster-B/m5_battern_step_record.md) — Battern protocol step trajectory recorder
- [`../modules/cluster-B/m6_context_cost.md`](../modules/cluster-B/m6_context_cost.md) — F10-aware EMA cost-band recorder

## Antipatterns specific to Cluster B

- **AP-WT-F10** (exploration-cost preservation collapse) — m6 returns `ema_mean + ema_variance + n`, never a single scalar.
- **AP-WT-F11** (cascade monoculture / label collapse) — m4 `cluster_id` is opaque FNV; m5 `battern_id` derives from protocol markers, not similarity.
- **AP-V7-09** (substrate-frame confusion) — no anthropocentric labels on any Cluster B type.

---

> **Back to:** [`../INDEX.md`](../INDEX.md) · [`../MODULE_MATRIX.md`](../MODULE_MATRIX.md) · [`../../README.md`](../../README.md) · vault [[cluster-B-habitat-observers]]
