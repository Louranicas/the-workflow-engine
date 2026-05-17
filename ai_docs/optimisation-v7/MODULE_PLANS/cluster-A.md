---
title: MODULE PLAN — Cluster A (Substrate Ingest)
date: 2026-05-17
kind: planning-only · per-module deep plan · V7 T3.A deliverable
cluster: A
layer: L1
modules: [m1, m2, m3]
status: V7 author-wave subagent draft (Command)
---

# Cluster A — Substrate Ingest (L1)

> Back to: [[../TASK_LIST_V7_OPTIMISATION.md]] · [[../ULTRAMAP.md]] · [[../GENERATIONS/G3-bidi-flow.md]]
>
> Peers: [[cluster-B.md]] (downstream observers consume A outputs) · [[cluster-C.md]] (m7 central hub joins A streams) · [[cluster-D.md]] (aspect-wrap of all A writes/reads)

---

## Overview

Cluster A is the substrate-ingest layer L1 — the engine's only read-port onto the host habitat's three primary trajectory substrates: atuin shell history, stcortex SpacetimeDB memory, and the habitat injection.db causal-chain store. Per ULTRAMAP View 1, L1 sits directly above L0 (`workflow-core` types/schemas/namespace constants) and below L2 (Cluster B habitat observation). Cluster A is **read-only by mandate** — the engine never writes back into atuin/stcortex via these modules; the write-side belongs to m13 (stcortex writer in Cluster C) and m42 (POVM dual-path in Cluster H). The aspect layer L4 (Cluster D: m8 build-prereq + m9 namespace guard) wraps every A module at compile/write/output time per G3 § Aspect-layer arrow inventory; m8 enforces `cfg(povm_calibrated)` on the whole crate, m9 validates the stcortex/POVM namespace prefix on writes derived from A reads (downstream).

Cluster A modules together account for **~230 LOC** (per ULTRAMAP View 2: m1 ~80 + m2 ~70 + m3 ~80) and **150 tests** (50 each per TEST_DISCIPLINE matrix). Boilerplate-lift density is high (~80% average) because two of the three modules clone from `memory-injection/m06_schema.rs` (SQLite WAL pragma + open helpers) and one clones from `stcortex/clients/rust-subscriber/src/main.rs`. The structural novelty in Cluster A is small: cursor-based pagination for atuin (~30 LOC of fresh authorship in m1) and narrowed-scope subscription queries for stcortex (~10 LOC variant authorship in m2). No Cluster A module owns a structural gap — Gaps 1/2/3 all live in Clusters D/F.

The cluster's substrate-frame distinction (per G3): A is the unique read-only ingress surface. Watcher Class-G (substrate-frame confusion) is pre-positioned here because mis-treating atuin rows as anthropocentric "user intent" — rather than as opaque substrate trajectory — would silently corrupt all downstream evidence in Clusters E/F.

---

## m1 — atuin_consumer

**Purpose.** Stream atuin shell-history rows (cursor-paginated, read-only) into the engine for cascade/battern/cost reconstruction.

**Upstream-IN** (per G3 § m1).
- `~/.local/share/atuin/history.db` — SQLite read-only (URI mode `?mode=ro`).
- `m1.config.cursor` — internal monotonic `last_id: i64` persisted across `next_page()` calls.
- Page-size config — bounded `[100, 10_000]`; default 1_000.

**Downstream-OUT.**
- `Vec<AtuinHistoryRow>` → m4 cascade (CascadeCluster derivation), m5 battern (BatternStepRow ordering), m6 cost (EMA input).
- `PageResult { rows, last_id, exhausted }` — exhaustion when returned page < page_size (per cluster-A spec § Pagination).

**Aspect-IN** (per G3 § m1).
- m8 build-prereq — compile-time `cfg(povm_calibrated)` gate (no functional effect on m1, but the whole crate refuses to build if POVM `learning_health` out of band).
- m9 namespace guard — N/A on the read path (m1 emits nothing into stcortex/POVM); m9 *will* wrap any downstream m13/m42 write derived from m1 data.

**src/ path** (per ULTRAMAP View 2 + G2 § src layout): `src/m1_atuin_consumer/` with sub-modules `mod.rs` (public API), `config.rs` (`AtuinConsumerConfig`), `cursor.rs` (cursor state), `row.rs` (`AtuinHistoryRow` + `parse_row`), `error.rs` (typed errors).

**LOC budget** (per ULTRAMAP View 2): **~80 LOC** (m1 spec § LOC estimate). ~50 LOC lifted from `memory-injection/m06_schema.rs` (open + WAL pragmas) + ~30 LOC fresh (cursor pagination).

**Test budget** (per TEST_DISCIPLINE matrix row m1): **50 tests**.

**Test-pattern allocation.**
- F-Unit 25 — per-arm coverage of `AtuinConsumerConfig` defaults, `open_readonly()` error paths, `parse_row()` per-column extraction, `next_page()` cursor advance / exhaustion branch, `PageResult` construction.
- F-Property 5 — pagination invariants: monotonic-cursor-non-decrease (`prop_assume!(p2.last_id >= p1.last_id)`), idempotent re-read of same cursor returns same rows, page_size ≤ returned ≤ 0, exhaustion sticky.
- F-Fuzz 0 (no parser surface — SQLite handles row decoding).
- F-Integration 15 — real `~/.local/share/atuin/history.db` clone in `tests/fixtures/`; full-iteration scan; concurrent-pagination interleave.
- F-Contract 3 — `AtuinHistoryRow` schema snapshot (insta); column-list stability vs atuin v18.10.
- F-Regression 2 — pre-seeded slots for first bugs (e.g., cursor-rollback-on-error, page_size=0 panic).
- F-Mutation 1 budget — ≥70% kill on `cursor.rs` + `row.rs` mutations.

**Mutation kill threshold** (per G6 § Mutation budget): **≥70%** (standard non-KEYSTONE).

**Boilerplate-lift source** (per cluster-A spec § Boilerplate lifts).
- `memory-injection/m06_schema.rs::configure_connection` (lines per spec — WAL/busy_timeout/foreign_keys/synchronous/wal_autocheckpoint batch) — ~90% lift, add `PRAGMA query_only = ON`.
- `memory-injection/m06_schema.rs::open_database` — ~85% lift, strip migration logic, open as read-only URI.
- `memory-injection/m11_parallel_query.rs::QueryResult<T>` — ~80% lift, rename to `PageResult`, parametrize over `Vec<AtuinHistoryRow>`, staleness threshold 200ms.
- `memory-injection/m18_atuin_cache.rs` graceful-degrade wrapper — ~70% lift, abstract into `FallbackIngest<T>` trait.

**Structural-gap LOC.** None for m1. Cursor pagination is novel-but-trivial (~30 LOC); it does not constitute a Gap-class structural primitive.

**Failure-modes covered** (per ANTIPATTERNS_REGISTER § AP-WT).
- F11 (cascade monoculture) — m1 emits raw rows; opaque-ID derivation lives in m4. m1's contract preserves verbatim pane labels for m4 to FNV-1a-XOR-fold.
- F3 (substrate-input poisoning) — read-only SQLite URI + `PRAGMA query_only` enforces no accidental write; m1 can never poison atuin.
- AP-Hab-04 (preserve-list discipline) — m1 never executes blanket commands; pagination bounded by page_size; cursor exhaustion deterministic.

**Atuin trajectory anchor.** Scripts that touch this module:
- `habitat-bootstrap` (reads atuin history.db on session start — m1 inherits the read pattern).
- `wt-soak-pulse` (proposed per T5.2; would read m1's exposed `PageResult` for soak telemetry).
- `wt-gate-status` (proposed per T5.2; reads m1's `last_id` to confirm engine is current with atuin).

**Watcher class pre-position** (per KEYWORDS_20 § 20).
- **Class A** — m1's first successful `next_page()` post-Genesis = activation moment.
- **Class G** — substrate-frame confusion if `AtuinHistoryRow` is treated as user-intent rather than opaque trajectory.
- **Class H** — atuin proprioception anomaly if cursor diverges from atuin's live `MAX(id)` by >1h.

---

## m2 — stcortex_consumer

**Purpose.** Subscribe to the narrowed-scope stcortex SpacetimeDB feed (`tool_call` + `consumption_event` only — W1 narrowing) to capture cross-session memory consumption events that signal workflow boundaries.

**Upstream-IN** (per G3 § m2).
- stcortex `:3000` SpacetimeDB module (WebSocket subscription via Rust SDK).
- Two narrowed subscription queries: `SELECT * FROM tool_call WHERE namespace LIKE 'workflow_trace_%'` and `SELECT * FROM consumption_event WHERE namespace LIKE 'workflow_trace_%'`.
- `m2.config.timeout_ms` — `wait_count` pattern for subscription-applied confirmation (per `stcortex/clients/rust-subscriber/src/capacity.rs`).

**Downstream-OUT.**
- `Vec<StcortexRow>` → m4 cascade, m5 battern, m13 stcortex writer (m13 reads m2's stream to dedupe write attempts).
- `SubscriptionApplied(())` mpsc signal — emitted exactly once when both subscriptions confirmed.

**Aspect-IN** (per G3 § m2).
- m8 — must be `povm_calibrated` for stcortex correctness (CR-2 magnitude-weighted formula upstream).
- m9 — read-side narrow-scope validator: every incoming row's namespace MUST start with `workflow_trace_` (per AP30); m9 rejects out-of-namespace rows at the m2 boundary.

**src/ path:** `src/m2_stcortex_consumer/` with `mod.rs`, `config.rs`, `subscription.rs` (subscription-applied state machine), `row.rs` (`StcortexRow` enum: `ToolCall { … }` + `ConsumptionEvent { … }`), `error.rs`.

**LOC budget** (per ULTRAMAP View 2): **~70 LOC**. ~60 LOC lifted from `stcortex/clients/rust-subscriber/src/main.rs` (DbConnection builder + subscription registration) + ~10 LOC fresh (narrowing handler filters).

**Test budget** (per TEST_DISCIPLINE matrix row m2): **50 tests**.

**Test-pattern allocation.**
- F-Unit 25 — subscription-applied state transitions, `StcortexRow::parse`, namespace prefix filter, timeout-expiry branch.
- F-Property 5 — narrowing invariant: `for_all (row: StcortexRow) -> row.namespace.starts_with("workflow_trace_")`; row-ordering preservation across reconnect.
- F-Fuzz 0 (Rust SDK handles wire decoding).
- F-Integration 15 — local stcortex `:3000` running; subscription-applied within 2s; reconnect-on-disconnect; refuse-write enforcement (m2 has no consumer registered → expected refuse).
- F-Contract 3 — `StcortexRow` schema snapshot vs `stcortex_API.md` `register_consumer`/`access_memory` signatures.
- F-Regression 2 — reserved (e.g., hyphen-slug munge regression per AP-Hab-11).
- F-Mutation 1 budget — ≥70%.

**Mutation kill threshold:** ≥70%.

**Boilerplate-lift source** (per cluster-A spec § Boilerplate lifts).
- `stcortex/clients/rust-subscriber/src/main.rs::DbConnection::builder` chain — ~80% lift, strip `pathway`/`memory` handlers.
- `stcortex/clients/rust-subscriber/src/main.rs` subscription query strings — ~85% lift, hardcode `workflow_trace_*` namespace LIKE.
- `stcortex/clients/rust-subscriber/src/capacity.rs::wait_count` pattern — ~90% lift for subscription-applied confirmation.
- `stcortex/clients/rust-subscriber/src/capacity.rs::AtomicBool` state pattern — ~90% lift for `subscription_applied` flag.
- `stcortex/docs/CONSUMER-ONBOARDING.md` refuse-write reducer — **reference only**: m2 must `register_consumer` before m13 attempts any write (documented in m2 docstring).

**Structural-gap LOC.** None for m2. Narrowed-scope subscription is configuration-level, not a Gap-class primitive.

**Failure-modes covered.**
- F8 (Watcher feedback-loop poisoning) — narrowed scope (only `tool_call` + `consumption_event`) structurally excludes Watcher event tables from m2's feed; Watcher observations cannot feed back into iteration evidence.
- F3 (substrate-input poisoning) — m9 namespace prefix filter at the m2 read boundary.
- AP-Hab-11 (hyphen-slug munge) — hyphens in labels converted to underscores at `parse_row` time; verified post-write by m13.
- AP30 (namespace prefix discipline) — enforced at read filter; out-of-namespace rows rejected before propagating to m4/m5.

**Atuin trajectory anchor.**
- `stcortex-probe` (atuin script — m2 inherits its consumer-count probe pattern).
- `wt-substrate-pulse` (proposed per T5.2; queries m2's `subscription_applied` + last-row timestamp).

**Watcher class pre-position.**
- **Class A** — first `SubscriptionApplied(())` signal post-Genesis.
- **Class D** — four-surface drift if m2 reads namespace-prefixed rows but corresponding ai_docs/vault entries are missing.
- **Class I** — Hebbian silence (currently firing per tick·16) — m2 will observe substrate non-movement directly; pre-positioning is mandatory.

---

## m3 — injection_consumer

**Purpose.** Read the habitat causal-chain store (`~/.local/share/habitat/injection.db`) for known cause→effect chains that bound workflow boundaries.

**Upstream-IN** (per G3 § m3).
- `~/.local/share/habitat/injection.db` SQLite — `causal_chain` table primary; secondary tables (per `memory-injection/m07_causal_chain.rs` schema).
- Filter: `resolved_session IS NULL` for unresolved chains; `consent NOT IN ('Forget')` for ethical exclusion.
- `m3.config.limit` — bounded `[100, 5_000]`; default 500.

**Downstream-OUT.**
- `Vec<InjectionEvent>` → m4 (cascade — chains anchor cascade boundaries), m5 (battern — chain labels seed battern step labels), m7 (central correlation hub — direct join on `chain_id`).

**Aspect-IN** (per G3 § m3).
- m8 — compile-time gate.
- m9 — read-side namespace validator (m3 enforces that `chain_type` is in the allowed taxonomy set; rejects unknown chain_types).

**src/ path:** `src/m3_injection_consumer/` with `mod.rs`, `config.rs`, `causal_chain.rs` (`CausalChainRow` mirror — workflow-trace-local type, no upstream import), `enums.rs` (`ChainType`, `ConsentLevel`), `query.rs` (`find_unresolved` + `find_recently_resolved`), `error.rs`.

**LOC budget** (per ULTRAMAP View 2): **~80 LOC**. ~65 LOC lifted (m06_schema + m07_causal_chain) + ~15 LOC fresh (enum mapping + workflow-trace-local error domain).

**Test budget** (per TEST_DISCIPLINE matrix row m3): **50 tests**.

**Test-pattern allocation.**
- F-Unit 25 — `configure_connection` pragma application, `parse_row` per-column, `ChainType`/`ConsentLevel` parse + invalid-string rejection, `find_unresolved` query construction, limit-clamp.
- F-Property 5 — query-result-bound invariants: returned rows ≤ limit; ordering DESC by `reinforcement_count`; consent filter never returns `Forget`.
- F-Fuzz 0.
- F-Integration 15 — real `injection.db` fixture; `find_unresolved` × `find_recently_resolved` interleave; consent filter end-to-end.
- F-Contract 3 — `InjectionEvent` schema snapshot; column-list stability vs current injection.db v4 schema.
- F-Regression 2 — reserved.
- F-Mutation 1 budget — ≥70%.

**Mutation kill threshold:** ≥70%.

**Boilerplate-lift source** (per cluster-A spec § Boilerplate lifts).
- `memory-injection/m06_schema.rs::configure_connection` — ~90% lift, add `PRAGMA query_only = ON`.
- `memory-injection/m06_schema.rs::open_database` — ~85% lift, drop migration logic, read-only URI.
- `memory-injection/m07_causal_chain.rs::CausalChainRow` struct — ~70% lift, rename to workflow-trace-local type, enum-ify string columns.
- `memory-injection/m07_causal_chain.rs::parse_row()` — ~75% lift, typed enums via parse functions, return `InjectionDbError::RowParseFailed`.
- `memory-injection/m07_causal_chain.rs::find_unresolved()` — ~80% lift, add `consent NOT IN ('Forget')` filter, parametrise limit.
- `memory-injection/m11_parallel_query.rs` timing harness — ~60% lift, emit to tracing span (not wrapper).

**Structural-gap LOC.** None for m3. Direct row reflection of an existing schema; no novel algorithm.

**Failure-modes covered.**
- F3 (substrate-input poisoning) — m9 validates row structure pre-emit downstream; injection.db corruption surfaces as `RowParseFailed` not silent propagation.
- AP-Hab-04 (preserve-list) — `consent NOT IN ('Forget')` filter is the read-side preserve-list; never bypass.
- AP-Drift-05 (migration "applied" but schema unchanged) — m3 never migrates; pure read; mitigated by construction.

**Atuin trajectory anchor.**
- `injection-pulse` (proposed; reads m3's `find_unresolved` count for habitat-health probe).
- `habitat-bootstrap` (reads `causal_chain` indirectly via the L4 layer of bootstrap injection — m3 inherits the read pattern).

**Watcher class pre-position.**
- **Class A** — first successful `find_unresolved` post-Genesis.
- **Class D** — four-surface drift if injection.db `causal_chain` rows exist without matching ai_docs/CLAUDE.local.md anchors.
- **Class G** — substrate-frame confusion if `chain.label` is treated as natural-language intent rather than opaque tag.

---

## Cluster-level synergies (which CC-1..CC-7 Cluster A participates in)

Per G3 § Cross-cluster synergies bidi diagrams:

- **CC-1 Cascade-Cost Coupling (B internal via m7 join).** Cluster A is the data source: m1 → m4/m6 + m3 → m4. CC-1 closes inside Cluster B; Cluster A's role is the join-source guarantee — m1/m3 must emit consistent `session_id` framing across both downstream consumers so m7's join column never NULLs.
- **CC-2 Trust Layer Woven (D → all).** All three A modules are aspect-wrapped by m8 (compile) + m9 (read-side AP30 prefix validator on m2, structural validator on m3, no-op on m1). Cluster A is therefore the read-side surface of CC-2.
- **CC-3 Evidence-Driven Iteration (E → F).** Cluster A is two hops upstream (A → B → C → E → F); Cluster A's role is provenance preservation — `AtuinHistoryRow`/`StcortexRow`/`InjectionEvent` carry session/chain IDs that survive the join in m7 and reach m14's evidence layer.
- **CC-7 Pressure-Driven Evolution (E → spec).** Cluster A is the closure target: per G3 § CC-7 closure, m15's reservation register escalates to Watcher/Zen → spec amendment → `m1.config.cursor` (or `m2.config.timeout`/`m3.config.limit`) updates next session. Cluster A is therefore the **only cluster whose config receives pressure feedback** — CC-7's bidi loop closes at A.

Cluster A does NOT directly participate in CC-4 (proposal→bank→dispatch — that's F→G→Conductor), CC-5 (substrate learning loop — that's G→H→F), or CC-6 (verification-gated dispatch — G internal).

---

## Cluster-level antipatterns (subset of ANTIPATTERNS_REGISTER relevant to Cluster A)

From ANTIPATTERNS_REGISTER.md, the antipatterns most likely to land in Cluster A:

- **AP-Hab-03 (AP30 violation)** — m2's narrowing filter + m9's read-side prefix validator are the dual-defence; failing either lets non-`workflow_trace_*` rows propagate to m4/m5/m13.
- **AP-Hab-04 (preserve-list discipline)** — m3's `consent NOT IN ('Forget')` is the canonical preserve-filter at read; never enumerate-then-blanket.
- **AP-Hab-11 (hyphen-slug stcortex munge)** — m2's `parse_row` converts hyphens to underscores in slug fields per S1001757 munge bug; m13 (write-side) verifies post-write.
- **AP-WT-F3 (substrate-input poisoning)** — Cluster A is the ingress surface; all three modules pre-emit-validate.
- **AP-WT-F8 (Watcher feedback-loop poisoning)** — m2 narrowed-scope subscription structurally excludes Watcher event tables; if narrowing widens in a future spec amendment, F8 becomes live.
- **AP-WT-F11 (cascade monoculture)** — Cluster A's contract preserves verbatim pane labels for m4 to opaque-fold; if any A module pre-folds (mistakenly trying to "help"), F11 fires.
- **AP-V7-09 (substrate-frame engine confusion)** — Cluster A is the substrate-frame boundary; any module-author drift toward "user wants X" framing flags Watcher Class-G immediately.
- **AP-Drift-05 (migration "applied" but schema unchanged)** — m3 never migrates injection.db; m1 never migrates atuin history.db; mitigated by construction (read-only URIs). m2 cannot migrate stcortex; refuse-write at DB layer per `stcortex/docs/CONSUMER-ONBOARDING.md`.

The cluster's overall risk surface is **read-side validation rigour** — every byte that crosses the L1→L2 boundary must be enum-typed, prefix-validated, and preserve-filtered before any downstream module touches it.

---

*cluster-A authored 2026-05-17 by Command (V7 author wave subagent)*
