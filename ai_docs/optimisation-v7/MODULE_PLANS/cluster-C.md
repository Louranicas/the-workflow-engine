---
title: MODULE PLAN — Cluster C (Central Correlation + Output)
date: 2026-05-17
kind: planning-only · per-module deep plan · V7 T3.C deliverable
cluster: C
layer: L3
modules: [m7, m12, m13]
status: V7 author-wave subagent draft (Command)
---

# Cluster C — Central Correlation + Output (L3)

> Back to: [[../TASK_LIST_V7_OPTIMISATION.md]] · [[../ULTRAMAP.md]] · [[../GENERATIONS/G3-bidi-flow.md]]
>
> Peers: [[cluster-A.md]] (m1/m2/m3 ingest) · [[cluster-B.md]] (m4/m5/m6 observation joins) · [[cluster-D.md]] (aspect-wrap; m11 decay reads m7) · [[cluster-E.md]] (m14 lift consumes m7 rows) · [[cluster-G.md]] (m30 bank ultimately derived from m7 evidence) · [[cluster-H.md]] (m42 dual-path uses m13's writer)

---

## Overview

Cluster C is L3 — the engine's central correlation hub and only output surface. **m7** is the workflow_runs table (the F9 zero-weight column owner, the JSONB join target for B's three streams, the single source of truth for every downstream evidence query). **m12** is the human CLI report renderer (the only stdout surface; F6 self-dispatch refusal owner). **m13** is the stcortex writer (3-band LTP/LTD gate owner; the AP30 namespace prefix discipline write-side).

Per ULTRAMAP View 2, Cluster C accounts for **~370 LOC** (m7 ~200 + m12 ~80 + m13 ~90) and **180 tests** (70 + 50 + 60 per TEST_DISCIPLINE matrix). m7 has the highest test density in C because it's the schema-of-record — schema-stability tests, JSONB roundtrip property tests, and integration tests across every consumer/producer pair all land here.

Cluster C is where every Cluster A and Cluster B stream converges before fanning back out to E/F/G/H. The substrate-frame role (per G3): C is the unique persistence-of-record surface — every downstream evidence claim must trace to a `WorkflowRunRow.id` here, and every substrate write to stcortex must pass through m13's namespace guard. The aspect-layer L4 (m8 compile + m9 namespace write-boundary + m10 Ember CI + m11 decay-factor injection) wraps Cluster C completely; m13 specifically is the canonical m9 namespace-guard call site.

---

## m7 — central

**Purpose.** The workflow_runs SQLite table (schema-of-record). Single source of truth for every downstream evidence query. F9 zero-weight column owner.

**Upstream-IN** (per G3 § m7).
- `m4.CascadeCluster` rows (with opaque `cluster_id`).
- `m5.BatternStepRow` rows.
- `m6.ContextCostBand` aggregates.
- `m3.InjectionEvent` rows (direct join on `chain_id`).

**Downstream-OUT.**
- `WorkflowRunRow { id, started_at, ended_at, fitness_dimension: f64, outcome: Outcome, consumer_inputs: JsonValue }` → m11 decay (reads `last_run_at`), m12 CLI reports, m13 stcortex writer, m14 lift evidence layer.
- SQLite schema migrations under `migrations/` (sqlx convention).

**Aspect-IN.**
- m8 — compile-time gate.
- m9 — write-time validator on every row insert (namespace prefix on payloads destined for stcortex via m13).
- m10 — Ember CI gate (m7 schema changes are PR-text and audited).

**src/ path:** `src/m7_central/` with `mod.rs` (public API), `schema.rs` (the `WorkflowRunRow` type + SQL DDL constants), `migrations/` (sqlx migration files), `query.rs` (insert + select + join helpers), `consumer_inputs.rs` (JSONB join column construction from m4/m5/m6 inputs), `error.rs`.

**LOC budget** (per ULTRAMAP View 2): **~200 LOC**. Boilerplate-lift moderate (~50%) from ME v2 SQLite-table patterns; the JSONB consumer_inputs join construction is novel-but-bounded.

**Test budget** (per TEST_DISCIPLINE matrix row m7): **70 tests**.

**Test-pattern allocation.**
- F-Unit 30 — `WorkflowRunRow` field-defaults, `Outcome` enum per-arm, `consumer_inputs` JsonValue construction from each B-cluster input shape, insert/select per column, F9 default-application (`fitness_dimension REAL NOT NULL DEFAULT 0.0`).
- F-Property 5 — schema invariants: `fitness_dimension` never null after insert; `consumer_inputs` roundtrips (insert → select → deserialize → equal); `started_at ≤ ended_at` enforced; `outcome` always in canonical set.
- F-Fuzz 1 — JSONB blob roundtrip (`fuzz_targets/m7_jsonb_fuzz.rs`); assertion = serde_json::from_value(serde_json::to_value(blob)) == blob.
- F-Integration 18 — m4/m5/m6 → m7 insert; m7 → m11/m12/m13/m14 read; concurrent insert; migration up/down; m7 schema-stability across `migrate run`.
- F-Contract 5 — `WorkflowRunRow` schema snapshot (insta); SQL DDL stability snapshot; JSONB schema for `consumer_inputs`.
- F-Regression 3 — F9 regression (any commit removing NOT NULL DEFAULT 0.0); JSONB column omission regression; migration-applied-but-schema-unchanged (AP-Drift-05) regression.
- F-Mutation 1 budget — ≥70% on `schema.rs` + `query.rs`.

**Mutation kill threshold:** ≥70%.

**Boilerplate-lift source.**
- ME v2 `src/m07_*/schema.rs` — workflow-runs table pattern (~60% reuse for DDL constants + sqlx migration scaffolding).
- LCM `src/loop_runs/` insert/select helpers (~40% reuse for query.rs).
- `memory-injection/m07_causal_chain.rs::parse_row` extraction pattern (~50% reuse for `WorkflowRunRow::from_row`).
- `consumer_inputs` JSONB join column — fresh authorship (~60 LOC); no exact upstream equivalent.

**Structural-gap LOC.** None for m7 specifically. m7 is the storage-of-record; the structural-gap (Gap 1 KEYSTONE) compositional sub-graph detection lives in Cluster F m20-m23 (per CLAUDE.md § structural-gap authorship).

**Failure-modes covered** (per ANTIPATTERNS_REGISTER § AP-WT-F9).
- **F9 (workflow-grain fitness distortion) — exclusive owner.** `fitness_dimension REAL NOT NULL DEFAULT 0.0` is the schema-level guarantee; tested by F-Property invariant + F-Regression slot. Per G3 § m7 contract.
- F3 (substrate-input poisoning) — m9 namespace guard wraps every payload before m7 write that derives from a cross-substrate source.
- AP-Drift-05 (migration "applied" but schema unchanged) — `stcortex inspect` (via m13) + sqlx `__migrations` table cross-check at Wave-end.
- AP-Drift-06 (bridge contract drift) — `WorkflowRunRow` insta snapshot guards schema stability vs downstream m13/m40/m42 consumers.

**Atuin trajectory anchor.**
- `wt-pulse` (proposed per T5.2; reads `SELECT COUNT(*) FROM workflow_runs WHERE started_at > ?`).
- `wt-gate-status` (proposed; reads m7's most-recent-row timestamp).
- `habitat-bootstrap` (downstream — m7's rows are queried by the L4 bootstrap layer when constructing context).

**Watcher class pre-position.**
- **Class A** — first `INSERT INTO workflow_runs` post-Genesis.
- **Class D** — four-surface drift if m7 row exists without corresponding ai_docs/vault/stcortex/CLAUDE.local.md anchor.
- **Class I** — Hebbian silence (firing live per tick·16); m7's row count over time is the direct measure.

---

## m12 — CLI reports

**Purpose.** Human-readable CLI table/JSON renderer over `WorkflowRunRow`. The only stdout surface. F6 self-dispatch refusal owner.

**Upstream-IN** (per G3 § m12).
- `m7.WorkflowRunRow` — primary input.
- `m12.config.output_format` — `Table | Json | NdJson`.

**Downstream-OUT.**
- stdout (no other consumer).

**Aspect-IN.**
- m8 — compile.
- m9 — N/A (stdout, not substrate write).
- m10 — Ember CI gate (m12 user-facing output is PR-text; trait audit applies).

**src/ path:** `src/m12_cli_reports/` with `mod.rs`, `format.rs` (Table/Json/NdJson rendering), `refuse.rs` (the F6 self-dispatch check), `error.rs`.

**LOC budget** (per ULTRAMAP View 2): **~80 LOC**. Boilerplate-lift high (~70%) from `comfy-table` / `serde_json` patterns; fresh authorship is the F6 refusal check (~15 LOC).

**Test budget** (per TEST_DISCIPLINE matrix row m12): **50 tests**.

**Test-pattern allocation.**
- F-Unit 25 — Table rendering per-row + per-column; Json/NdJson rendering; F6 refusal per match arm; empty-result rendering; truncation behaviour for long fields.
- F-Property 5 — render-roundtrip: parse(render(row)) == row (Json/NdJson formats); table-render width-monotonic.
- F-Fuzz 0.
- F-Integration 15 — m12 ↔ m7 read; full CLI invocation; output-format switching; F6 end-to-end refusal.
- F-Contract 3 — output schema snapshot per format.
- F-Regression 2 — reserved (e.g., F6 regression if any commit silently allows self-dispatch reporting).
- F-Mutation 1 budget — ≥70%.

**Mutation kill threshold:** ≥70%.

**Boilerplate-lift source.**
- `comfy-table` crate idioms for Table rendering (~70% reuse; standard library).
- `serde_json::to_writer` / `to_writer_pretty` for Json/NdJson (~80% reuse).
- ME v2 CLI report patterns in `src/m41_*/cli.rs` (~50% reuse).
- F6 refusal check — fresh authorship (~15 LOC).

**Structural-gap LOC.** None.

**Failure-modes covered.**
- **F6 (self-dispatch) — exclusive owner at the report layer.** m12 refuses to render if `target_workflow == reporting_workflow` (the trivial self-loop that would emit causally-tainted output back into the engine via m7 readback). Per G3 § m12 contract.
- AP-Hab-14 (god-tier dilution) — m12's output discipline (no print/format/eprintln except via tracing) keeps god-tier clean.

**Atuin trajectory anchor.**
- `wf-crystallise report` (proposed binary subcommand; m12 is its renderer).
- `wt-pulse` (uses m12 NdJson output to feed atuin KV `habitat.wt.last_pulse`).

**Watcher class pre-position.**
- **Class A** — first `wf-crystallise report` invocation post-Genesis.
- **Class C** — confidence-gate refusal — m12's F6 refusal is exactly Class-C behaviour (safe-path refusal, not failure).

---

## m13 — stcortex writer

**Purpose.** Write workflow-trace records into stcortex under namespace `workflow_trace_*` with **3-band LTP/LTD gating**: >0.15 proceeds, 0.05-0.15 proceeds-with-warning, <0.05 deferred to local JSONL buffer.

**Upstream-IN** (per G3 § m13).
- `m7.WorkflowRunRow` — primary payload source.
- `m11.DecayFactor` — for prune-marker computation.
- Live POVM `:8125/learning_health` reading (gated indirectly via m8's `cfg(povm_calibrated)` compile-time check; runtime read at write-time).

**Downstream-OUT.**
- stcortex namespace `workflow_trace_*` (per AP30).
- Local JSONL buffer `outbox/m13/*.jsonl` when 3-band gate defers.

**Aspect-IN.**
- m8 — compile-time gate (the whole crate refuses to build if POVM `learning_health` outside [0.05, 0.15] at startup).
- m9 — **write-time namespace prefix validation** at every m13 write boundary. This is the canonical m9 call-site.
- m10 — Ember CI gate (m13's write semantics are spec-bound; trait audit applies).

**src/ path:** `src/m13_stcortex_writer/` with `mod.rs`, `gate.rs` (the 3-band LTP/LTD logic), `namespace.rs` (the m9-coupled prefix validator + hyphen-slug munge per AP-Hab-11), `write.rs` (the actual stcortex write + retry), `outbox.rs` (the local JSONL buffer for deferred writes), `error.rs`.

**LOC budget** (per ULTRAMAP View 2): **~90 LOC**. Boilerplate-lift moderate (~50%) from `stcortex/clients/rust-subscriber/` write patterns + ME v2 outbox-first patterns; fresh authorship is the 3-band gate + AP30 namespace coupling (~40 LOC).

**Test budget** (per TEST_DISCIPLINE matrix row m13): **60 tests**.

**Test-pattern allocation.**
- F-Unit 30 — 3-band gate per band, namespace-prefix validation per pass/fail, hyphen-slug munge (AP-Hab-11), outbox JSONL write per success/fail, retry-with-jitter per attempt.
- F-Property 5 — namespace invariant: `for_all (payload: Payload) -> namespace_of(write(payload)).starts_with("workflow_trace_")`; outbox roundtrip; 3-band gate monotonic in `learning_health`.
- F-Fuzz 0.
- F-Integration 15 — m13 ↔ stcortex `:3000` live; m13 ↔ POVM `:8125/learning_health`; 3-band gate end-to-end per band; outbox-replay after stcortex recovery.
- F-Contract 5 — stcortex write payload schema vs `stcortex_API.md` accepted format; outbox JSONL schema snapshot; namespace string-format stability.
- F-Regression 4 — AP30 regression slot; hyphen-slug munge regression (S1001757); 3-band-flip regression (CR-2-fixed thresholds); outbox-loss regression.
- F-Mutation 1 budget — ≥70% on `gate.rs` + `namespace.rs`.

**Mutation kill threshold:** ≥70%.

**Boilerplate-lift source.**
- `stcortex/clients/rust-subscriber/src/main.rs` (write-side mirror of m2 read patterns) — ~60% reuse for `DbConnection`-based write.
- ME v2 outbox-first JSONL pattern — ~70% reuse for `outbox.rs` (per S106 CC-5 substrate learning loop discipline).
- POVM `/learning_health` HTTP client — adapted from `m40_42_common::HealthClient` (Cluster H lift) — ~80% reuse.
- 3-band gate — fresh authorship (~25 LOC); the band thresholds (>0.15 proceed, 0.05-0.15 warn, <0.05 defer) align with CR-2 reconciled `substrate_LTP_density` ranges per Hebbian v3 reconciliation note.
- AP-Hab-11 hyphen-slug munge — fresh authorship (~5 LOC).

**Structural-gap LOC.** None for m13 specifically. The Gap 2 fitness-weighted decay formula lives in m11 (Cluster D); m13 consumes m11's `DecayFactor` but doesn't author the formula.

**Failure-modes covered** (per ANTIPATTERNS_REGISTER § AP-WT + AP-Hab).
- **AP30 (namespace prefix discipline) — exclusive owner at the write layer.** m9-coupled validator refuses any write missing `workflow_trace_` prefix. Per G3 § m13 contract.
- **AP-Hab-11 (hyphen-slug stcortex munge) — exclusive owner.** Hyphens in labels converted to underscores in pre_id/post_id slugs at write-time; verified post-write via `stcortex inspect`.
- **3-band LTP/LTD gate** — F-WT-Substrate failure mode (engine writing to a degraded substrate); deferred to outbox below 0.05.
- F7 (CR-2 graceful-degrade pretend-fix) — m13 hard-couples to m8's compile-time `cfg(povm_calibrated)` AND runtime band-check; both must agree.
- AP-Drift-06 (bridge contract drift) — `bridge-contract` skill run pre-merge on m13's outbox JSONL schema vs SYNTHEX/POVM accepted formats.
- AP-Drift-08 (push state inconsistent across remotes) — m13's outbox writes are SHA-fingerprinted for cross-substrate verification.

**Atuin trajectory anchor.**
- `wt-substrate-pulse` (proposed; reads m13 outbox depth + last-successful-write timestamp).
- `stcortex-probe` (atuin script — m13's write is the inverse of m2's read; shares probe pattern).

**Watcher class pre-position.**
- **Class A** — first stcortex write post-Genesis (the substrate-write activation).
- **Class B** — hand-off boundary crossing — every stcortex write is a cross-substrate hand-off; per Phase 3 cross-substrate calls.
- **Class D** — four-surface drift if m13 writes succeed but corresponding ai_docs/vault/CLAUDE.local.md anchors are missing.
- **Class I** — Hebbian silence — m13's 3-band gate is the direct measure; if `learning_health` stays <0.05 for N consecutive write attempts, Class-I escalates from "firing" to "sustained".

---

## Cluster-level synergies (which CC-1..CC-7 Cluster C participates in)

Per G3 § Cross-cluster synergies:

- **CC-1 Cascade-Cost Coupling (B internal via m7 join).** m7 is the join target; m4 and m6 never couple directly. m7's `consumer_inputs` JSONB schema is the contract that lets B's three streams converge without N×N module-module coupling.
- **CC-2 Trust Layer Woven (D → all).** Cluster C is the canonical aspect-wrap site: m8 compile-time (whole crate), m9 write-time (m13 is the prime call site), m10 Ember CI (m12 + m13 user-facing surfaces), m11 lifecycle (m13 reads `DecayFactor` for prune-marker).
- **CC-3 Evidence-Driven Iteration (E → F).** Cluster C is the immediate downstream consumer of B's observation streams; m7's `WorkflowRunRow` is the primary input to m14 lift evidence; m14 → m20 PrefixSpan gate. C's contract guarantee: every `WorkflowRunRow.id` is queryable forever (no row deletion, only sunset via m11/m30).
- **CC-4 Proposal → Bank → Dispatch (F → G → Conductor).** Cluster C is two hops upstream of CC-4; m7's rows are the evidence base m23 proposer cites; m30 bank-entry `definition_hash` is derived from m7 schema state at proposal time.
- **CC-5 Substrate Learning Loop (G → H → back to F via stcortex pathways).** m13 is the substrate-write side; m13's stcortex writes feed back into m31's selector at the next selection cycle via stcortex pathway.weight delta. m13 is therefore a critical CC-5 participant.
- **CC-7 Pressure-Driven Evolution (E → spec).** Cluster C's m7 schema evolution is one of the most likely sources of m15 reservation events (schema drift → forbidden-verb pressure).

Cluster C participates in **5 of 7** CC synergies — the highest density of any cluster. m7 specifically is the most-cited target across the bidi flow.

---

## Cluster-level antipatterns (subset of ANTIPATTERNS_REGISTER relevant to Cluster C)

- **AP-Hab-03 (AP30 violation) — m13 prime owner.** Mitigation: m9-coupled prefix validator at write boundary; tested by F-Property invariant.
- **AP-Hab-11 (hyphen-slug stcortex munge) — m13 owner.** Mitigation: hyphen → underscore at write-time; verified post-write.
- **AP-WT-F6 (self-dispatch) — m12 owner.** Mitigation: F6 refusal check at render layer.
- **AP-WT-F9 (workflow-grain fitness distortion) — m7 owner.** Mitigation: `fitness_dimension REAL NOT NULL DEFAULT 0.0`; tested by F-Property + F-Regression.
- **AP-Drift-04 (test count over-report)** — Cluster C's 180-test count must verify-count at Wave-end.
- **AP-Drift-05 (migration "applied" but schema unchanged) — m7 prime risk.** Mitigation: sqlx `__migrations` cross-check + `stcortex inspect` schema diff at Wave-end.
- **AP-Drift-06 (bridge contract drift)** — m13's outbox JSONL + m7's `WorkflowRunRow` schema both insta-snapshotted and `bridge-contract`-validated.
- **AP-Drift-08 (push state inconsistent across remotes) — m13 write semantics adjacent.** Mitigation: SHA-fingerprinted outbox writes.
- **AP-V7-09 (substrate-frame engine confusion)** — m7's `consumer_inputs` JSONB column is the substrate-frame discipline; anthropocentric naming or back-decoding is a Class-G drift.
- **AP-Test-03 (integration-test mock-leak)** — m7's integration tests use real local stcortex/POVM, not mocks; per TEST_DISCIPLINE § Integration-test pattern.

The cluster's overall risk surface is **schema-of-record discipline** — every byte that crosses L3 in either direction (insert or select) must satisfy the m7 schema invariants, namespace-prefix validation, 3-band substrate-gate, and self-dispatch refusal.

---

*cluster-C authored 2026-05-17 by Command (V7 author wave subagent)*
