# ARCHITECTURE — workflow-trace

> **Canonical:** [`ai_docs/optimisation-v7/ULTRAMAP.md`](ai_docs/optimisation-v7/ULTRAMAP.md) (View 1 = layer; View 2 = module table) · [`ai_docs/API_MAP.md`](ai_docs/API_MAP.md) (complete public-API reference) · [`ai_docs/GENESIS_PROMPT_V1_3.md`](ai_docs/GENESIS_PROMPT_V1_3.md) § 1
> **This file:** post-remediation structural summary of the **implemented** 26-module codebase. Verified against `src/lib.rs` and the `orchestration` module as of git `ae7d460` (S1003733 closeout: binaries wired).
> **Status:** ACTIVE — G9 fired 2026-05-17; Hardening Fleet W0–W5 complete; assessment-driven remediation S1003733 landed (binaries wired via `workflow_core::orchestration`; CC-4 diversity threading + CC-5 pathway-id + the `EscapeSurfaceProfile` monotone gate; core-type encapsulation portfolio closed). 1835+ tests passing; clippy + pedantic clean.

---

## 1. What this is

`workflow-trace` is a single Rust crate (package name `workflow-trace`, library `workflow_core`)
that observes cascading-command, Battern-protocol, and context-window activity across the
Zellij habitat, mines compositional patterns from those observations, composes workflow
variants for **human review**, and dispatches ratified workflows via HABITAT-CONDUCTOR — never
by spawning a process directly.

It is **26 modules** organised into **8 synergy clusters** (A–H), packaged as one shared
library plus **two thin binaries**. The library holds every pipeline stage; the binaries hold
only argument parsing, report printing, and exit-code translation. This split is what makes
the pipelines integration-testable — a test calls the library `run` function directly against
temp-file fixtures with `--offline` / `--dry-run`, no subprocess required.

---

## 2. The crate: one library, two binaries, one orchestration module

| Artefact | Path | Role |
|---|---|---|
| **`workflow_core`** (lib) | `src/lib.rs` | All 26 modules + the `orchestration` pipeline drivers; the entire public API surface ([`ai_docs/API_MAP.md`](ai_docs/API_MAP.md)) |
| **`wf-crystallise`** (bin) | `src/bin/wf_crystallise.rs` | Owns m1–m23 + m40–m42. Thin: parses argv, calls `orchestration::crystallise::run`, prints `Report`, sets exit code |
| **`wf-dispatch`** (bin) | `src/bin/wf_dispatch.rs` | Owns m30–m33. Thin: parses argv, calls `orchestration::dispatch::run`, prints `Report`, sets exit code |
| **`orchestration`** (module) | `src/orchestration/` | The pipeline drivers behind the two binaries — `crystallise.rs` + `dispatch.rs` + `mod.rs` |

Single Cargo crate, **not** a multi-crate workspace (the ORAC single-crate pattern, not the
LCM 10-crate workspace). `workflow_core` is the `[lib]`; the two binaries are `[[bin]]`
targets that depend on it internally.

### 2.1 The `orchestration` module (S1003733 — the wiring that closed the binary-stub gap)

Before S1003733 both binary `main()` functions were one-line `println!` stubs and the library
sat unwired. The remediation introduced `src/orchestration/`, which moved the end-to-end
pipeline logic **into the library** so it is testable:

- `orchestration::crystallise` — the m1→m23 + m40–m42 ingest → mine → propose →
  substrate-feedback pipeline driver. Backs `wf-crystallise`.
- `orchestration::dispatch` — the m30→m33 bank → select → verify → dispatch pipeline driver.
  Backs `wf-dispatch`.

Each sub-module exposes the same shape: a `Config` (parsed CLI options), `parse_args`, a `run`
function, a serialisable `Report`, an `ArgError`, and an `OrchestrationError`. Because both
sub-modules name those types identically, `lib.rs` re-exports them under disambiguating
prefixes: `CrystalliseConfig` / `run_crystallise` / `CrystalliseReport` / `CrystalliseError`
and `DispatchConfig` / `run_dispatch` / `DispatchReport` / `DispatchError`.

**Live-service degradation contract.** Every stage that touches a live habitat service
(stcortex `:3000`, ORAC `:8133`, synthex-v2 `:8092`, LCM `:8082`, HABITAT-CONDUCTOR `:8141`)
degrades gracefully: an unreachable service is logged via `tracing` and recorded as a
skipped/degraded entry on the `Report`. The pipelines always run end-to-end to completion — a
down service never panics or aborts the run. `OrchestrationError` is reserved for **true
faults** only (a missing required file, an unwritable output, an invalid miner/selector
parameter).

---

## 3. The 26 modules · 8 clusters · 9 layers

| Cluster | Layer | Modules | Role |
|---|---|---|---|
| **A** Substrate Ingest | L1 | m1, m2, m3 | atuin SQLite reader / stcortex narrowed-scope WebSocket consumer / injection.db reader |
| **B** Habitat Observers | L2 | m4, m5, m6 | cascade correlator (opaque FNV-1a IDs) / Battern step record / context-cost EMA |
| **C** Correlation + Output | L3 | m7, m12, m13 | `workflow_runs` SQLite hub / CLI report formatters / stcortex writer (3-band LTP gate) |
| **D** Trust (cross-cutting) | L4 | m8, m9, m10, m11 | POVM build-prereq cfg-gate / namespace guard / Ember CI gate / fitness-weighted decay |
| **E** Evidence + Pressure | L5 | m14, m15 | Wilson-CI lift aggregator / forbidden-verb pressure register (JSONL) |
| **F** Iteration (KEYSTONE) | L6 | m20, m21, m22, m23 | PrefixSpan miner / variant builder / k-means clustering / workflow proposer |
| **G** Bank/Select/Dispatch/Verify | L7 | m30, m31, m32, m33 | curated bank / weighted selector / Conductor dispatcher / 4-agent verifier |
| **H** Substrate Feedback | L8 | m40, m41, m42 | NexusEvent emit / LCM RPC / stcortex-only Hebbian reinforce |

- **L0** = the substrate frame itself (atuin, stcortex, injection.db, SYNTHEX, LCM, Conductor,
  Watcher) — observed, not authored.
- **L9** = a substrate-frame engine — intentionally absent; placeholder reserved.
- **Module naming:** unpadded `m1`–`m42`. Module directory order in `src/` is the build order:
  Cluster D ships first (m8 → m9 → m10 → m11), then A, then B/C, then E, then F, then G/H.

### Cluster A — Substrate Ingest

- **m1 `m1_atuin_consumer`** — read-only paginated ingress to atuin's shell-history SQLite
  (`~/.local/share/atuin/history.db`). Cursor-based `id > last_id` paging; `PRAGMA query_only`;
  byte-for-byte command/session/cwd preservation for FNV-1a determinism. Never writes.
- **m2 `m2_stcortex_consumer`** — narrowed-scope SpacetimeDB WebSocket consumer. Subscribes to
  exactly two SQL queries (`tool_invocation`, `consumption_event`) filtered to the
  `workflow_trace_*` namespace; LIKE-injection-safe. `RegistrationHandle::is_fresh` is the
  CC-2 freshness signal consumed by m13.
- **m3 `m3_injection_db_consumer`** — read-only ingress to the habitat `causal_chain` ledger
  (`~/.local/share/habitat/injection.db`). `Forget`-consent rows are filtered at the SQL layer
  and never emitted downstream.

### Cluster B — Habitat Observers

- **m4 `m4_cascade`** — correlates atuin steps into opaque multi-pane cascade-cluster IDs
  (FNV-1a-XOR). Owns the shared `fnv1a_64` hash primitive used across m20/m21/m23/m30.
- **m5 `m5_battern`** — observes Battern-protocol executions in the step stream; unlabelled
  steps surface as `None`, never discarded.
- **m6 `m6_cost`** — per-session token-cost proxy + exploration-rate baseline EMA. The
  baseline EMA tracks **only** `Explored` / `Diverged` / `Unknown` outcomes (F10
  exploration-cost preservation).

### Cluster C — Correlation + Output

- **m7 `m7_workflow_runs`** — central WAL SQLite hub for workflow trace records. `WorkflowRunRow`
  carries `run_state: RunState` (an `Open` / `Closed{ended_at,outcome}` enum — illegal mixed
  Some/None states are unrepresentable), a JSONB `consumer_inputs` blob (the CC-1 join
  surface), and `fitness_dimension: f64 DEFAULT 0.0` (F9 zero-weight — only m11 writes it).
- **m12 `m12_cli_reports`** — pure formatter: `WorkflowRunRow` slices → human- or
  machine-readable strings. No DB writes, no side effects, all `#[must_use]`.
- **m13 `m13_stcortex_writer`** — promotes `WorkflowRunRow`s to stcortex under a 3-band
  LTP/LTD gate. Every write passes the m9 `ValidatedNamespace` evidence. Below
  `LTP_PHASE_1_FLOOR` (0.015) → defer to a JSONL outbox; `[0.015, 0.10)` → write
  `under_pressure`; `>= 0.10` → write normally; ORAC unreachable → defer.

### Cluster D — Trust (cross-cutting — woven through every other cluster, CC-2)

Cluster D is **not feature-gated** — these are aspect-layer invariants that every other
module routes through.

- **m8 `m8_povm_build_prereq`** — compile-time + runtime CR-2 verification gate. `build.rs`
  emits `cargo:rustc-cfg=povm_calibrated` **iff** `POVM_CR2_DEPLOYED=1`. This is a custom
  rustc-cfg, **not** a Cargo feature — it cannot be activated by `--features full`. Any
  in-tree POVM-read site guarded by `#[cfg(not(povm_calibrated))]` would emit a
  `compile_error!` tombstone when the env var is absent.
  **Decision (S1003733 F2): KEEP-DORMANT.** No in-tree POVM-read site remains — m42 is
  stcortex-routed per the 2026-05-17 ADR — so the gate has nothing to guard. The trust regime
  is enforced statically by the `build.rs` cfg; m8's runtime probe is **not** invoked as a
  pipeline stage. `orchestration::crystallise::run` documents this explicitly at Stage 0.
- **m9 `m9_watcher_namespace_guard`** — single source of truth for the `workflow_trace_`
  prefix. `assert_workflow_trace_namespace` validates and munges hyphens → underscores,
  returning an opaque `ValidatedNamespace` newtype that m13/m42 consume as write-side proof.
  Defense-in-depth complement to SpacetimeDB's DB-layer refuse-write rule.
- **m10 `m10_ember_ci_gate`** — output-time 7-trait Ember rubric CI gate. Hybrid
  CI-FAIL + allowlist posture: `Rejected` (confidence ≥ 0.5) always fails CI and is not
  allowlistable; `Held` (confidence < 0.5) fails CI unless an unexpired allowlist row exists
  in `tests/ember_held_approvals.tsv`.
- **m11 `m11_fitness_weighted_decay`** — Gap 2 NEW PRIMITIVE: the `frequency × fitness ×
  recency` compound-decay formula. Lifecycle checkpoint of the CC-2 trust regime —
  `run_consolidation_cycle` runs decay → reinforce-read → prune → auto-sunset against the
  curated bank, catching workflow ossification at lifecycle time.

### Cluster E — Evidence + Pressure

- **m14 `m14_lift`** — Wilson 95% CI aggregator + composite lift. `wilson_ci_half` returns
  `None` below `MIN_SAMPLE_SIZE` (20) — the F2 sample-size-inflation mitigation. Produces the
  `LiftSnapshot` that gates m23.
- **m15 `m15_pressure`** — forbidden-verb pressure **witness** (not a gate — it blocks
  nothing). Detects forbidden-verb-pressure excerpts and emits durable JSONL
  `PHASE-B-RESERVATION-NOTICE` files.

### Cluster F — Iteration (KEYSTONE)

- **m20 `m20_prefixspan`** — Gap 1 NEW PRIMITIVE: gap-allowed PrefixSpan sequential pattern
  mining over cascade/Battern step sequences. Deterministic — output sorted by
  `(support DESC, len DESC, canonical_hash ASC)`. `MinSupport::new` enforces the F2 floor at
  the type level.
- **m21 `m21_variant_builder`** — enumerates bounded `Swap`/`Skip` mutations on top-supported
  patterns into `WorkflowVariant`s; fair-emission interleave so neither mutation class is
  starved. `MAX_LOOP_ITERATIONS` caps the loop independently (mutation-testing hardening).
- **m22 `m22_kmeans`** — Lloyd's k-means with k-means++ seeding, bounded iterations,
  deterministic seed. Feeds the CC-4 diversity signal.
- **m23 `m23_proposer`** — composes `WorkflowVariant`s + evidence into `WorkflowProposal`s for
  human review. **Never auto-promotes** (AP-V7-07). F2 hard refusal: a proposal is rejected
  when `snapshot.n < PROPOSAL_F2_THRESHOLD` (20) or `snapshot.lift` is `None`.

### Cluster G — Bank / Select / Dispatch / Verify

- **m30 `m30_bank`** — `CuratedBank`: accepted-proposal storage (`BTreeMap` behind a `Mutex`)
  with `sunset_at` + decay weights. `accept` rejects the m32 self-dispatch sentinel
  (AP-V7-08). `workflow_pathway_id` produces the CC-5 stcortex pathway identifier.
- **m31 `m31_selector`** — weighted-composite scorer: `α·fitness + β·recency + γ·frequency +
  δ·diversity` (weights sum to 1.0, compile-time-asserted). The δ-diversity component is
  externally supplied via closure — the F11 monoculture anti-property.
- **m32 `m32_dispatcher`** — dispatches selected workflows via the HABITAT-CONDUCTOR endpoint.
  **Hard refusal: never spawns a process/shell/fleet pane directly.** Routes to
  `lcm.loop.create` (never `lcm.deploy`). 5-check sequence: (1) AP-V7-08 self-dispatch,
  (2) escape-surface acknowledgement, (3) routing-method enforcement, (4) m33 verifier gate,
  (5) Conductor submit.
- **m33 `m33_verifier`** — 4-agent verification gate (Security / Consistency / Cost / Ember).
  `aggregate(&[&dyn Verifier], &AcceptedWorkflow) -> Result<AggregateVerdict, VerifierError>`
  requires exactly one verifier of each `VerifierKind`; any blocking verdict →
  `AggregateVerdict::Blocked { per_verifier }` (sorted by ordinal), else `AllApprove`.

### Cluster H — Substrate Feedback

- **m40 `m40_nexus_emit`** — pushes `NexusEvent`s to synthex-v2 `:8092/v3/nexus/push`.
  AP-V7-13 body-shape check (a 2xx response carrying an `"error"` field is **not** success).
- **m41 `m41_lcm_rpc`** — `lcm.loop.create` JSON-RPC 2.0 client to LCM `:8082/rpc`; verifies
  the id-echo. NOT `lcm.deploy`.
- **m42 `m42_stcortex_emit`** — substrate-feedback (Hebbian reinforce). **m42 stcortex-only
  pivot (2026-05-17 ADR):** all substrate-feedback writes route through
  `m13::StcortexWriter::promote_run`; no direct POVM call. POVM is decoupled.

---

## 4. The two end-to-end pipelines

### 4.1 `wf-crystallise` — observe → mine → propose

Driver: `orchestration::crystallise::run(&Config) -> Result<Report, OrchestrationError>`.
The pipeline always runs to completion; live stages are attempted only when
`config.offline == false`.

| Stage | Module(s) | What happens | Live? |
|---|---|---|---|
| 0 | m8 | Trust floor — **KEEP-DORMANT**; no runtime probe. Static `build.rs` cfg only | — |
| 1 | m1 | atuin ingest — `open_atuin_readonly(...).collect_all()` → `Vec<AtuinHistoryRow>` | file |
| 1b | m3 | injection.db ingest — `read_unresolved()` → `Vec<CausalChainRow>` | file |
| 2 | m2 | stcortex narrowed consumer registration (skipped under `--offline`) | **live** |
| 3 | m4 | cascade correlation — rows → `AtuinStep`s → `Vec<CascadeCluster>` | pure |
| 4 | m7 / m9 | record run — `insert_run` → `merge_observation` (one per cluster + per chain) | file |
| 5 | m14 | lift snapshot over open runs → `LiftSnapshot`; then `close_run` | file |
| 6 | m20 / m21 / m23 | KEYSTONE — `mine_sequences` → `compose_proposals` (with diversity closure) | pure |
| 7 | m12 / m13 | write proposals JSONL output (the cross-binary bridge) | file |
| 8 | m40 | nexus emit `WorkflowCompleted` event (skipped under `--offline`) | **live** |

Output: a `proposals.jsonl` file (one `WorkflowProposal` per line) + a printed `Report`.

### 4.2 `wf-dispatch` — bank → select → verify → dispatch

Driver: `orchestration::dispatch::run(&Config) -> Result<Report, OrchestrationError>`.
`--dry-run` is the **default-safe** mode (verify + select, no Conductor contact); a real
dispatch requires the explicit `--execute` flag.

| Stage | Module(s) | What happens | Live? |
|---|---|---|---|
| 1 | — | load proposals from the JSONL bridge → `Vec<WorkflowProposal>` | file |
| 2 | m30 | `CuratedBank::accept` each proposal; a rejected proposal is logged + skipped | pure |
| 3 | m31 | `select_top_k` over the bank's active set → `Vec<ScoredCandidate>` | pure |
| 4a | m33 | per-candidate `aggregate` 4-verifier gate → `AggregateVerdict` | pure |
| 4b | m32 | under `--execute`: `ConductorDispatcher::dispatch` (5-check sequence) | **live** |

Output: a printed `Report` carrying a per-candidate disposition
(`dry-run` / `dispatched` / `refused` / `verifier-blocked`).

### 4.3 The cross-binary bridge

`wf-crystallise` writes `proposals.jsonl`; `wf-dispatch` reads it. The JSONL file **is** the
handoff — there is no shared in-process state, no shared bank database. One `WorkflowProposal`
per line; a blank line is skipped, a malformed line aborts `wf-dispatch` with
`OrchestrationError::ProposalsParse`. The `CuratedBank` itself is in-memory and rebuilt fresh
on every `wf-dispatch` invocation from the proposals file.

---

## 5. Cross-cluster synergies (CC-1 … CC-7) — as wired now

| ID | Contract | How it is wired |
|---|---|---|
| **CC-1** | Cascade-Cost Coupling | `ClusterBObservation` enum holds both `Cascade` and `ContextCost`/`InjectionChain` variants under the same `consumer_inputs` JSONB blob on `WorkflowRunRow`; `merge_observation` JSON-patches each discriminant in. m14 reads the combined blob. |
| **CC-2** | Trust Layer Woven (D → all) | m9 `assert_workflow_trace_namespace` is called inside `StcortexWriter::promote_run` (m13) and `WorkflowId::new` (m14); `MinSupport::new` enforces F2 at the type level (m20); `WorkflowProposal` constructor enforces F2 (m23); the bank validates weights + rejects NaN factors (m30); `select_top_k` sanitises NaN components (m31); the m8 `build.rs` cfg statically gates any future POVM read. |
| **CC-3** | Evidence-Driven Iteration (E → F) | `compose_proposals(patterns, &snapshot, diversity_of)` takes the m14 `LiftSnapshot`; the m23 F2 gate reads `MIN_SAMPLE_SIZE` re-exported from m14. |
| **CC-4** | Proposal → Bank → Dispatch | `compose_proposals`'s `diversity_of: impl Fn(&WorkflowVariant) -> Option<usize>` closure carries the m22 k-means cluster index into `WorkflowProposal::diversity_cluster`. **S1003733 threaded this:** `diversity_cluster: Some(idx)` now flows m22 → m23 → `WorkflowProposal` → `CuratedBank::accept` → `AcceptedWorkflow` → `select_top_k`. (The orchestration CLI path passes a documented conservative `\|_\| None` — the m22 signal is genuinely absent in the batch path, not faked.) |
| **CC-5** | Substrate Learning Loop (G → H → F) | `m42::emit_feedback` → `m13::StcortexWriter::promote_run` → stcortex `write_pathway`; `m30::workflow_pathway_id` produces the namespace-key (**S1003733 — the CC-5 pathway-id wiring**); `m11::PathwayWeightReader` reads those pathway weights back into the decay formula; `m41` re-triggers the next crystallise iteration. |
| **CC-6** | Verification-Gated Dispatch (G internal) | `ConductorDispatcher::with_verifiers` attaches the m33 set; `dispatch()` calls `m33::aggregate` at step 4 of the 5-check sequence; `RefusalReason::VerifierGateBlocked` carries the blocking `VerifierKind`s in ordinal order. |
| **CC-7** | Pressure-Driven Evolution (E → spec interviews) | `m15::PressureRegister::record` emits JSONL `PHASE-B-RESERVATION-NOTICE` files; the Watcher ☤ and Zen read them asynchronously. Architectural file-drop coupling — no in-code wiring. |

---

## 6. Core data types and how they flow

| Type | Producer | Consumer(s) | Note |
|---|---|---|---|
| `AtuinHistoryRow` | m1 | m4, m5, orchestration | raw atuin shell-history row |
| `AtuinStep` | orchestration (`row_to_step`) | m4 | ms→ns timestamp conversion |
| `CascadeCluster` | m4 | m7 (via `ClusterBObservation::Cascade`) | opaque FNV-1a cluster id |
| `CausalChainRow` | m3 | m7 (via `ClusterBObservation::InjectionChain`) | `Forget` rows pre-filtered |
| `WorkflowRunRow` | m7 | m12, m13, m14, m42 | `run_state` enum; JSONB `consumer_inputs`; `fitness_dimension` F9 zero-weight |
| `RunState` | m7 | m12, m13, m42 | `Open` \| `Closed{ended_at,outcome}` — illegal states unrepresentable |
| `LiftSnapshot` | m14 | m23 (F2 gate) | `lift: Option<f64>` — `None` below n=20 |
| `Pattern` | m20 | m21, m23 | encapsulated; `.steps()`/`.support()`/`.canonical_hash()` accessors |
| `WorkflowVariant` | m21 | m23 | `Swap`/`Skip`/`Identity` mutation |
| `WorkflowProposal` | m23 | m30 (`accept`) | encapsulated; the JSONL bridge payload |
| `AcceptedWorkflow` | m30 | m31, m32, m33, m42 | encapsulated; sunset state machine |
| `EscapeSurfaceProfile` | caller / CLI `--ack-ceiling` | m32 | 7-variant closed enum, ordinals 0/10/…/60 |
| `HumanAcceptanceSignature` | operator (CLI flag) | m32 | monotone destructiveness gate |
| `PromoteOutcome` | m13 | m42, caller | `Written` \| `WrittenUnderPressure` \| `Deferred` |

The S1003733 core-type-encapsulation portfolio (Wave C) closed six representable-illegal-state
holes: `WorkflowRunRow`→`RunState`, plus full field encapsulation on `Pattern`,
`WorkflowProposal`, `AcceptedWorkflow`, `NexusEvent`, and the ID newtypes.

---

## 7. The trust spine (Cluster D — what enforces correctness)

The trust regime is woven through every cluster (CC-2) and is **not feature-gated**:

1. **m8 — POVM build-prereq (KEEP-DORMANT).** A `build.rs` custom rustc-cfg
   (`povm_calibrated`, gated on `POVM_CR2_DEPLOYED=1`) — not a Cargo feature, so unbypassable
   by `--features full`. With m42 stcortex-routed there is no in-tree POVM read to guard, so
   the gate is dormant: present, statically enforced, but with no runtime pipeline stage.
2. **m9 — namespace guard.** The single source of truth for the `workflow_trace_` prefix.
   Every substrate write carries a `ValidatedNamespace` newtype as compile-checked evidence.
3. **m10 — Ember CI gate.** Every user-facing string is scored against the 7-trait Ember
   rubric; hybrid CI-FAIL + allowlist posture.
4. **m11 — fitness-weighted decay.** The lifecycle checkpoint — `frequency × fitness ×
   recency` compound decay run periodically over the curated bank, sunsetting ossified
   workflows so the bank cannot become a monoculture.

---

## 8. Feature gate matrix

```toml
[features]
default = ["full"]
full         = ["api", "intelligence", "monitoring", "evolution"]
api          = []   # reserved
intelligence = []   # reserved
monitoring   = []   # reserved
evolution    = []   # reserved
```

The four sub-features are currently RESERVED and enable nothing — the 26 modules are too
interdependent for clean partitioning at this stage. Cluster D is deliberately **not**
gated. The only real conditional-compilation switch is the m8 `povm_calibrated` rustc-cfg.

---

## 9. Structural-gap authorship (net-new; cannot be lifted from boilerplate)

| Gap | Owner | What it is |
|---|---|---|
| **Gap 1** N-step compositional sub-graph detection | F (m20 + m23) | gap-allowed PrefixSpan + Wilson CI — the KEYSTONE |
| **Gap 2** `frequency × fitness × recency` compound decay | D (m11) | a NEW PRIMITIVE decay formula |
| **Gap 3** Unified `EscapeSurfaceProfile` 7-variant schema | G (m30 + m32) + D (m9) | ordinal enum + monotone destructiveness gate + namespace guard |

---

> **Back to:** [`README.md`](README.md) · [`CLAUDE.md`](CLAUDE.md) · [`CLAUDE.local.md`](CLAUDE.local.md) · [`GATE_STATE.md`](GATE_STATE.md) · [`ai_docs/API_MAP.md`](ai_docs/API_MAP.md) · [`ai_docs/INDEX.md`](ai_docs/INDEX.md) · [`ai_specs/INDEX.md`](ai_specs/INDEX.md) · [`ultramap/README.md`](ultramap/README.md)
