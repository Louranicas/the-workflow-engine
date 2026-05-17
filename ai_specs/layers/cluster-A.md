---
title: Cluster A — Substrate Ingest (Layer L1) — Layer Spec
cluster: A
layer: L1
module_count: 3
modules: [m1_atuin_consumer, m2_stcortex_consumer, m3_injection_db_consumer]
binary: wf-crystallise
feature_gates: [none]
cc_owns: []
cc_consumes: [CC-1, CC-2, CC-3]
ship_priority: Day 1 (after Cluster D)
status: SPEC
date: 2026-05-17
hold_v2_compliant: true
---

# Cluster A — Substrate Ingest (L1)

> **Back to:** [`../INDEX.md`](../INDEX.md) · [`../MODULE_MATRIX.md`](../MODULE_MATRIX.md) · [`../../README.md`](../../README.md) · [`../../ARCHITECTURE.md`](../../ARCHITECTURE.md) · vault [[cluster-A-substrate-ingest]] · [`../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-A.md`](../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-A.md)

## Role

Cluster A IS the **read-only data-intake boundary** of workflow-trace. Its sole responsibility is to surface tool-call rows, consumer events, and resolved/unresolved causal-chain entries from three external substrates as typed iterators consumable by Clusters B and C. Nothing in this cluster writes, proposes, selects, classifies, or routes. The Phase A verb-set applies without exception: **ingest · correlate · record · emit · refuse**. The Genesis v1.3 single-phase override does NOT relax Cluster A's verb-lock — active verbs are reserved exclusively for Clusters F / G / H modules whose Town Hall P0 commitments require them.

The three modules read three distinct substrates. **m1** paginates `~/.local/share/atuin/history.db` via cursor-based queries with a 5,000 ms WAL busy-timeout, falling back to `atuin history list --format json` subprocess if SQLite open fails. **m2** registers as a narrowed-scope SpacetimeDB consumer at `127.0.0.1:3000` with a reducer-callback dedup ring and emits `ConsumerEvent` iterators; the registration side-effect itself is the runtime trust signal CC-2 (m9 namespace-guard) keys on. **m3** opens `~/.local/share/habitat/injection.db` and partitions the `causal_chain` table by `resolved_session IS NULL`, surfacing unresolved-vs-resolved iterators that drive Cluster B causal correlation. No row produced by m1, m2, or m3 is interpreted or filtered for relevance inside this cluster — interpretation begins at Cluster B.

## Modules

| Module | Spec | LOC | Tests | Substrate | Iterator type |
|---|---|---:|---:|---|---|
| `m1_atuin_consumer` | [`../modules/cluster-A/m1_atuin_consumer.md`](../modules/cluster-A/m1_atuin_consumer.md) | 80 | 50 | atuin SQLite | `ToolCallRow` |
| `m2_stcortex_consumer` | [`../modules/cluster-A/m2_stcortex_consumer.md`](../modules/cluster-A/m2_stcortex_consumer.md) | 80 | 50 | stcortex `:3000` | `ConsumerEvent` + trust signal |
| `m3_injection_db_consumer` | [`../modules/cluster-A/m3_injection_db_consumer.md`](../modules/cluster-A/m3_injection_db_consumer.md) | 70 | 50 | injection.db SQLite | `CausalChainRow` |

## Cross-cluster contracts

- **OWNS:** none (Cluster A is a pure source; CC ownership lives downstream).
- **CONSUMES** (via aspect-weave only):
  - **CC-2** — m8 `povm_calibrated` build-prereq cfg-gate applies to the whole crate at build.rs time; m9 namespace-guard activates the moment m2 registration succeeds (CC-2 trust signal published).
- **PROVIDES TO:**
  - **CC-1** — m1 `ToolCallRow` iterator feeds m4 (cascade correlator) and m6 (context-cost EMA); m4 and m6 join through m7's JSONB `consumer_inputs` column.
  - **CC-3** — m1 + m3 supply the trajectory + lifecycle context that m14 turns into Wilson-CI `Lift` for Cluster F iteration gating.

## Binary placement

All three modules live in **`wf-crystallise`** (the read-heavy ingest binary). `wf-dispatch` does not link Cluster A — dispatch never touches raw substrate iterators, only `AcceptedWorkflow` bank rows. This is the canonical two-binary split locked in Genesis v1.3 § 1.

## Feature-gate posture

Cluster A is **un-gated** (`feature = "none"`). The substrates are mandatory inputs; gating an ingest module would mean a build profile in which the engine cannot read its own evidence. This is intentional. Cluster D's aspect modules wrap Cluster A; Cluster A does not wrap Cluster D.

## Ship priority

Cluster A is **Day 1 of Wave 1 / Phase 1**, but ships **after Cluster D** (CC-2 trust layer woven first per phase-1 framework). Implementation order inside Wave 1: m8 → m9 → m10 → (Cluster D commit point) → m1 → m2 → m3. This ordering means the namespace-guard is live before the first substrate read; reads then emit no trust violations by construction.

## Operational invariants

1. **No writes.** Every module opens its substrate with the appropriate read-only pragma (SQLite `PRAGMA query_only = ON`; stcortex consumer registration is a structural read-side subscription, not a write). m1's subprocess fallback is read-only by command shape (`atuin history list`).
2. **No interpretation.** Rows are typed but un-labelled. Cluster A does not classify a `ToolCallRow` as a "dispatch attempt" or a "battern step"; that is m4/m5/m6's job in Cluster B.
3. **No coupling between m1, m2, m3.** Each reads a distinct substrate; intra-cluster invocation is forbidden. The only side-effect cross-link is m2 registration → m9 trust signal, which is published via the CC-2 aspect surface, not a direct call.
4. **Cursor monotonicity.** m1's pagination cursor is `id ASC`; m3's partition cursor is `causal_chain.id ASC`. m2's reducer-callback dedup ring is FIFO with capacity 256. None of the three modules emit a row twice within a single ingest pass.
5. **Graceful fallback is structural, not silent.** m1 falls back to subprocess via a *typed* error path (`AtuinIngestError::DatabaseOpenFailed → fallback_subprocess_ingest`); m2 surfaces stcortex disconnect as `ConsumerError::Disconnected` with a re-registration retry policy. Neither falls back to a sentinel value (no `Ok(vec![])` on substrate failure).

## Failure modes the cluster structurally refuses

- **Anthropocentric labelling.** No `ToolCallRow.intent: String`, no `ConsumerEvent.is_dispatch: bool` — these are downstream Cluster B classifications. Adding such fields to a Cluster A type is an AP-V7-09 violation (substrate-frame confusion) and rejected at spec review.
- **Inline interpretation in pagination logic.** m1's `next_page` does not skip rows based on command content; it surfaces every row in `id ASC` order. Filtering happens in m4.
- **Cross-substrate joins inside Cluster A.** A "session-id joined with stcortex consumer-id" view is a Cluster B/C concern (m7's JSONB join). Cluster A modules never see each other's row types.
- **Substrate writes via fall-through.** If m13 (stcortex writer) accidentally were placed in Cluster A's dependency graph, verify-sync invariant #15 would catch it — Cluster A's `Cargo.toml`-equivalent module declaration lists zero downstream Cluster C/H writer dependencies.

## Performance envelope

| Operation | Target | Notes |
|---|---|---|
| m1 `next_page(page_size=2000)` | < 50 ms (cold) / < 5 ms (warm) | atuin.db on local SSD; WAL replay dominates cold path |
| m1 fallback subprocess | < 5,000 ms total | hard timeout via `AtuinIngestConfig.subprocess_timeout_ms` |
| m2 reducer-callback dispatch | < 10 µs per event | dedup ring is `parking_lot::Mutex<VecDeque>`; no allocation in hot path |
| m3 partition query | < 30 ms (full re-scan) | injection.db is small (<5 MB typical); single-pass acceptable |

## Verify-sync invariants

(Per [`../../ai_docs/optimisation-v7/STANDARDS/GOD_TIER_RUST.md`](../../ai_docs/optimisation-v7/STANDARDS/GOD_TIER_RUST.md) § Verify-sync invariants 1-20):

- **#1** — `src/m1_atuin_consumer/`, `src/m2_stcortex_consumer/`, `src/m3_injection_db_consumer/` all exist post-G9.
- **#7** — `rg '\bunsafe\b' src/m1*/ src/m2*/ src/m3*/` returns 0.
- **#8** — `rg '\.unwrap\(\)' src/m1*/ src/m2*/ src/m3*/ | grep -v 'tests/'` returns 0.
- **#14** — every async fn in m2 (consumer subscribe) has `tokio::time::timeout` on the registration `await`.

## Per-module cross-links

- [`../modules/cluster-A/m1_atuin_consumer.md`](../modules/cluster-A/m1_atuin_consumer.md) — cursor-based atuin ingest with WAL busy-timeout
- [`../modules/cluster-A/m2_stcortex_consumer.md`](../modules/cluster-A/m2_stcortex_consumer.md) — narrowed-scope SpacetimeDB consumer + dedup ring
- [`../modules/cluster-A/m3_injection_db_consumer.md`](../modules/cluster-A/m3_injection_db_consumer.md) — partitioned causal-chain reader

## Antipatterns specific to Cluster A

- **AP-V7-09** (substrate-frame confusion) — every Cluster A type is operationalised at its substrate; no anthropocentric labels.
- **AP-Hab-03** (AP30 namespace violation) — m2's consumer-id slug uses `workflow_core::namespace::CONSUMER_PREFIX` constant; never literal `"workflow_trace_consumer"` strings.
- **AP-WT-F8** (no feedback-loop poisoning) — m1/m3 never read from `workflow_trace_*` namespace; m2 reads ONLY stcortex consumer-event surface, not workflow-trace pathway writes.

---

> **Back to:** [`../INDEX.md`](../INDEX.md) · [`../MODULE_MATRIX.md`](../MODULE_MATRIX.md) · [`../../README.md`](../../README.md) · vault [[cluster-A-substrate-ingest]]
