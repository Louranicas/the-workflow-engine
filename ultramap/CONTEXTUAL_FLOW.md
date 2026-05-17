---
title: CONTEXTUAL_FLOW — what metadata travels with each row
kind: planning-only · operational view · complements canonical V7 ULTRAMAP
status: Wave-2 author deliverable; no source authored
date: 2026-05-17
---

# CONTEXTUAL_FLOW — what metadata travels with each row

> **Back to:** [`README.md`](README.md) · [`../ARCHITECTURE.md`](../ARCHITECTURE.md) · canonical [`../ai_docs/optimisation-v7/ULTRAMAP.md`](../ai_docs/optimisation-v7/ULTRAMAP.md) · siblings [`MODULE_DEPENDENCY_GRAPH.md`](MODULE_DEPENDENCY_GRAPH.md) · [`DATA_FLOW.md`](DATA_FLOW.md) · [`CONTROL_FLOW.md`](CONTROL_FLOW.md) · [`INVARIANT_MAP.md`](INVARIANT_MAP.md)
>
> **Purpose:** [DATA_FLOW.md](DATA_FLOW.md) tells you the *type* on each edge; this file tells you the *attendant metadata* — what context-fields each row carries from the moment it enters at m1 to the moment it lands as a substrate-feedback pathway weight delta. Read this when you need to answer "if I'm at m23 looking at a Proposal, what provenance can I trust?" or "does the `WorkflowDispatchEvent` carry the original SessionId?".

---

## One row per data transformation

The engine has 11 major transformation stages. Each row in the table below shows one stage's *primary* row type plus the metadata it gains on entry to that stage. A field is **carried** if it survives the transformation (passed through), **derived** if computed at the stage, or **dropped** if intentionally stripped for downstream privacy/opacity (e.g., F11 cascade-monoculture mitigation drops human-readable cluster labels at m4).

| # | Stage | Substrate / source | Primary row | Carries | Notes |
|---|---|---|---|---|---|
| 1 | atuin ingest | `~/.local/share/atuin/history.db` | `AtuinHistoryRow` | `id`, `command`, `session: SessionId`, `hostname`, `timestamp_ms`, `exit: Option<i32>`, `duration_ms: Option<i64>`, `cwd: Option<String>` | byte-preserving; no normalisation (m1 invariant 2) |
| 2 | stcortex narrowed | `stcortex :3000` reducer-callback | narrowed `ToolCallRow + ConsumerSignal` | row fields + `reducer_id`, `trust_signal: ConsumerFreshness` | narrowed-scope filter — only `tool_call` + `consumption` accepted at boundary |
| 3 | injection.db read | `~/.local/share/habitat/injection.db` | `CausalChainRow` | `chain_id`, `label`, `reinforcement_count`, `resolved_session: Option<i64>`, `partition: {Resolved, Unresolved}` | derived `partition` is m3's classification on top of substrate fields |
| 4 | cascade correlate | m1 stream | `CascadeCluster` | `cluster_id: u64` (opaque FNV-1a XOR), `session_id: SessionId`, `step_count: usize`, `steps: Vec<StepToken>`, `cluster_size: usize` | **Drops:** human-readable labels (F11). `cluster_id` is XOR of constituent step hashes — non-invertible without the alphabet, so the ID itself is information-preserving but not human-meaningful. |
| 5 | battern record | m1 stream | `BatternStep` | `id`, `label: Option<String>`, `sequence_position`, `ts_ms`, `session_id: SessionId` | `label = None` for unlabelled batterns; m20 PrefixSpan downstream treats labelled and unlabelled identically at token level |
| 6 | context cost EMA | m1 stream + historical EMA state | `ContextCostBand` | `session_type`, `ema_mean: f64`, `ema_variance: f64`, `n: usize`, `baseline_excludes: ["Converged"]` | derived from 20-session rolling window; **excludes** Converged outcomes (F10) so the baseline is uncertain-cost not confident-cost |
| 7 | hub join | m4 + m6 + m3 | `WorkflowRunRow` | `run_id`, `session_id: SessionId`, `ts_ms`, `consumer_inputs: JSONB { cascade, cost, injection }`, `outcome: Option<RunOutcome>` | F9 zero-weight: JSONB column is additive; never removes fields; new consumers join by adding a sub-object |
| 8 | lift evidence | m7 aggregate | `Option<Lift>` carrying `{ point_estimate: f64, wilson_low: f64, wilson_high: f64, n: usize }` | when `n < 20`, m14 emits **`None`** — explicitly NOT a numeric stand-in. CC-3 gate propagates as `ProposalError::LiftEvidenceMissing` at m23 |
| 9 | KEYSTONE pattern | m5 stream + m14 gate | `Pattern` → `WorkflowVariant` → `FeatureCluster` → `WorkflowProposal` | `Pattern { steps:Vec<StepToken>, support, gap_bounds:(usize,usize) }` → `WorkflowVariant { id, base_pattern, similar:Vec<Pattern>, representative_score }` → `FeatureCluster { centroid, members, inertia }` → `WorkflowProposal { id, steps, provenance:Pattern, confidence:f64, deviation_rationale:String, expected_lift:Lift }` | `provenance` carries the original Pattern reference so an operator reviewing the proposal can trace it back to the cascade/Battern source |
| 10 | accepted workflow | human consent + m23 proposal | `AcceptedWorkflow` | `workflow_id: WorkflowId`, `steps_json: String`, `escape_surface: EscapeSurfaceProfile`, `sunset_at: i64`, `definition_hash: String` (FNV-1a hex), `accepted_at: i64`, `accepted_by: HumanAcceptanceSignature` | `HumanAcceptanceSignature { signed_at, terminal_fingerprint, accepted_by: String }` — **AP-V7-07 mandatory**. `escape_surface` is the Gap 3 ordinal — `Sandboxed < SandboxEscape < ProcessMutate < FileWrite < NetworkEgress < DataExfil` |
| 11 | dispatch event | m32 5-check + Conductor accept | `WorkflowDispatchEvent` | `dispatch_id: Uuid v7` (idempotency key), `workflow_id`, `lineage: LineageId`, `step_count: usize`, `escape_surface: EscapeSurfaceProfile`, `dispatched_at: i64`, `conductor_session_id: String`, `conductor_accepted: bool`, `ttl_remaining_days: f64` (from m33 receipt) | `lineage` carries trace back to the originating `WorkflowProposal.id` so substrate-feedback can credit the right Pattern |
| 12 | substrate feedback | m32 outcome → m40/m41/m42 | `SubstrateFeedback { outcome: PassVerified \| Pass \| Blocked \| Fail, fitness_delta: f64, retrieval_ids:Vec<String> }` | `fitness_delta` derived per m42 module-level constants: PV +0.25 / P +0.15 / B -0.05 / F -0.10, clamped `[-1.0, 1.0]`. `retrieval_ids` prefixed `workflow_trace_*` (AP30); hyphens slug-encoded as underscores (AP-Hab-11) |

---

## Provenance walks

The most useful query against a CONTEXTUAL_FLOW table is: "given an artifact at stage N, can I walk back to stage 1?" Three illustrative provenance walks:

### Walk A — Proposal back to atuin row

```
WorkflowProposal { id: P-abc, provenance: Pattern { steps: [S-12, S-7, S-29], support: 47 } }
  → Pattern was emitted by m20 from m5 BatternSteps grouped by session_id ∈ {S-aa, S-bb, ...}
  → each BatternStep carries session_id back to AtuinHistoryRow.session
  → m1 byte-preservation guarantees the original command bytes are still queryable in atuin.db
```

This walk is preserved **only because** every intermediate carries `session_id: SessionId` (a newtype-wrapped String, not a raw String). If any stage dropped session_id, the chain breaks at that boundary.

### Walk B — Dispatch event back to human signature

```
WorkflowDispatchEvent { dispatch_id: D-xyz, workflow_id: WF-123, lineage: L-456 }
  → workflow_id resolves to AcceptedWorkflow row in m30 bank
  → AcceptedWorkflow carries accepted_by: HumanAcceptanceSignature
  → HumanAcceptanceSignature carries terminal_fingerprint, signed_at, accepted_by user identifier
  → audit trail: dispatch_log.db row carries dispatch_id + workflow_id pair
```

This walk is the engine's **accountability surface** — every dispatch is traceable to the human who accepted the workflow into the bank.

### Walk C — Substrate weight delta back to Pattern

```
stcortex.pathway.weight delta on workflow_trace_wf_123_outcome_pass
  → m42 ReinforcePayload carried retrieval_ids = ["workflow_trace_wf_123"]
  → m42 was invoked from WorkflowDispatchEvent { workflow_id: WF-123, lineage: L-456 }
  → lineage L-456 resolves to original WorkflowProposal at proposal table
  → WorkflowProposal carries provenance: Pattern → m20 mined from BatternSteps → session_ids → atuin rows
```

This walk is **the** substrate-grain provenance chain — it tells you not just "this proposal worked" but "this proposal was mined from these specific cascades observed in these specific atuin sessions, and the substrate now weights its successors higher". Per [`CROSS_CLUSTER_SYNERGIES.md`](../ai_docs/optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md) § CC-5 in depth: this is the only loop in the engine that closes substrate-grain.

---

## Metadata that intentionally **does not** travel

Some fields are *deliberately* stripped — not lost, stripped — at specific boundaries. Each strip is a substrate-frame design decision:

| Stripped at | What is dropped | Why |
|---|---|---|
| **m1 → m4** | original sessionId is NOT used as cluster_id input | F11 cascade-monoculture mitigation. `cluster_id = FNV-1a XOR of step hashes` is content-derived, not identity-derived. Two sessions running the same cascade get the *same* cluster_id. |
| **m4 → m7** | human-readable step labels | F11. m4 emits opaque `StepToken(u32)`; m12 resolves labels for display only via `StepTypeRegistry`. |
| **m23 → m30** | proposal `confidence: f64` numerics | F5 mitigation. m30 admits on **human signature**, not on confidence. The confidence is preserved in the proposal record for reference but not carried as a bank field. |
| **m32 → m40** | full `steps_json` body | size minimisation. m40 NexusEvent carries `workflow_id + step_count + escape_surface`; if SYNTHEX needs the steps it can re-read from m30 by id. |
| **m42 → stcortex** | hyphens in slug fields | AP-Hab-11 (S1001757 munge bug). Hyphens are replaced with underscores at the slug boundary. |
| **m15 → agent-cross-talk** | redacted secret material | per [m15 spec invariants — see cluster-E plan in CROSS_CLUSTER_SYNERGIES § CC-7] — pressure events are JSONL with structured-key only, no raw-text from user prompts or shell history. |

---

## Metadata that travels invisibly (substrate-grain)

CC-5 carries one piece of metadata that *appears* not to travel because it has no in-process representation: the **pathway weight** at stcortex. Each dispatch emits a `fitness_delta`, and after enough deltas the cumulative weight on the `workflow_trace_*` pathway shifts. m31 reads the *current* weight at its next selection cycle, and the difference between selection cycle N and cycle N+1 is the "metadata" CC-5 carries — but it lives only in stcortex, not in any in-process struct.

Per [`CROSS_CLUSTER_SYNERGIES.md`](../ai_docs/optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md) § CC-5 substrate-frame distinction:

> CC-5 is the only one whose absence would cause substrate to silently degrade. Other CCs failing produces obvious test failures. CC-5 failing produces **invisible non-learning** — engine appears functional but substrate-weight never moves.

The implication: an in-process `CONTEXTUAL_FLOW` audit would miss CC-5's payload entirely. Detection lives in Watcher Class-I, monitoring rolling 7-day delta on `stcortex.pathway.weight` for `workflow_trace_*` IDs externally.

---

## Newtype discipline guarantees provenance

The engine's `workflow_core::types` module wraps every long-lived identifier in a newtype: `SessionId(String)`, `WorkflowId(String)`, `LineageId(String)`, `StepToken(u32)`, `ConsumerId(String)`, etc. (per [`../ARCHITECTURE.md`](../ARCHITECTURE.md) § Canonical src/ layout `types.rs`). The discipline gives three contextual-flow benefits:

1. **Type-level provenance.** A function that takes `&WorkflowId` cannot be called with a `SessionId` — the compiler enforces that an upstream identifier is not silently substituted at a downstream layer.
2. **No accidental string-fold.** `SessionId` does not implement `Add` or `+=`; concatenation requires explicit conversion. This prevents the class of bug where `format!("{}_{}", session, suffix)` silently corrupts identity.
3. **Serde round-trips preserve identity.** `serde::Serialize + Deserialize` on the newtype guarantees that the wire form is the wrapped string with no transformation — so a `SessionId` written to JSONL by m15 reads back as the same `SessionId` at any consumer.

---

## The `consumer_inputs` JSONB column

m7 owns the engine's central correlation surface — `workflow_runs.consumer_inputs` is a JSONB column with three currently-known top-level keys:

```jsonc
{
  "cascade": { "cluster_id": 1234567890, "session_id": "...", "step_count": 7 },
  "cost":    { "session_type": "...", "ema_mean": 0.42, "ema_variance": 0.013, "n": 20 },
  "injection": { "chain_id": "...", "label": "...", "reinforcement_count": 3 }
}
```

The **F9 zero-weight** rule per [`../ai_docs/optimisation-v7/ULTRAMAP.md`](../ai_docs/optimisation-v7/ULTRAMAP.md) View 2 m7: the JSONB column is additive. New consumers join by adding a new top-level key; existing consumers never have keys removed (deprecated keys are tombstoned with a null but kept in schema for old readers). This is the "stable schema as coupling surface" pattern that makes CC-1 work — m4 and m6 don't have to know about each other; they both know about m7's JSONB shape.

---

## Cross-references

| Question | Answer | File |
|---|---|---|
| What rows travel each edge (typed view)? | per-edge type table | [`DATA_FLOW.md`](DATA_FLOW.md) |
| When does each module fire? | trigger taxonomy | [`CONTROL_FLOW.md`](CONTROL_FLOW.md) |
| What invariants must hold? | per-cluster + cross-cluster | [`INVARIANT_MAP.md`](INVARIANT_MAP.md) |
| What does the build graph look like? | Mermaid graph TD | [`MODULE_DEPENDENCY_GRAPH.md`](MODULE_DEPENDENCY_GRAPH.md) |
| What is the per-CC contract surface? | inventory + path | [`../ai_docs/optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md`](../ai_docs/optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md) |
| What is the canonical layer view? | View 1 Mermaid | [`../ai_docs/optimisation-v7/ULTRAMAP.md`](../ai_docs/optimisation-v7/ULTRAMAP.md) |

---

> **Back to:** [`README.md`](README.md) · [`../ARCHITECTURE.md`](../ARCHITECTURE.md) · canonical [`../ai_docs/optimisation-v7/ULTRAMAP.md`](../ai_docs/optimisation-v7/ULTRAMAP.md) · [`ULTRAMAP.md`](ULTRAMAP.md) (this folder's master synthesis)
