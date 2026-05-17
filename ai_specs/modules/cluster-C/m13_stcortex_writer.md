---
title: m13 — stcortex_writer_narrowed (substrate emit with 3-band LTP/LTD gate)
module_id: m13
module_name: stcortex_writer
cluster: C — Correlation + Output
layer: L3
binary: wf-crystallise
verb_class: emit
feature_gate: [api]
loc_estimate: 90-120
test_budget: 60
boilerplate_lift: ~45-50%
gap_owner: none (3-band LTP/LTD gate owner; AP30 namespace-prefix write-side enforcer)
cc_contracts_owned: []
cc_contracts_consumed: [CC-2, CC-5]
status: SPEC · planning-only · HOLD-v2 active · no code until G1-G9 clear
authority: Luke @ node 0.A
date: 2026-05-17 (S1001982)
decisions_applied: [D-E]
---

# m13 — `stcortex_writer_narrowed`

> Back to: [`../../INDEX.md`](../../INDEX.md) · [`../../MODULE_MATRIX.md`](../../MODULE_MATRIX.md) · [`../../../CLAUDE.md`](../../../CLAUDE.md) · [`../../../CLAUDE.local.md`](../../../CLAUDE.local.md) · vault [[cluster-C-correlation-output]] · canonical V7 [cluster-C plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-C.md) · v1.3 spec [`../../../ai_docs/GENESIS_PROMPT_V1_3.md`](../../../ai_docs/GENESIS_PROMPT_V1_3.md) § 1
>
> Sister modules: [m7](m7_workflow_runs.md) · [m12](m12_cli_reports.md) · [m13](m13_stcortex_writer.md)
> Cluster peers in-flow: m7 (read-only source) · m9 (namespace guard write-time call site) · m2 (trust signal dependency, CC-2) · m11 (decay factor consumer) · m42 (downstream substrate-feedback emitter; H cluster)

---

## 1 — Role + one-line purpose

m13 is the **sole stcortex writer** in the workflow-trace engine. It promotes completed `WorkflowRunRow` rows to the `workflow_trace_*` namespace in stcortex, subject to two gates: (a) **m9 namespace-prefix validation** at every write boundary, and (b) the **3-band LTP/LTD substrate-headroom check** that defers writes when the Hebbian substrate is in LTD dominance.

Cluster C verb-class is `record` / `emit`; m13 is the **passive substrate-emitter**. m13 does not transform, route, or decide — it stages, gates, and either promotes or defers.

## 2 — Cluster + layer + binary placement

| Axis | Value |
|---|---|
| Cluster | **C — Central Correlation + Output** |
| Layer | **L3** |
| Binary | **`wf-crystallise`** (substrate-feedback emit owned at the periphery of the read-heavy binary) |
| Feature gate | **`api`** — m13's public surface is part of `workflow-core`; the binary wires it via `main.rs` |
| Verb class | **`emit`** (passive; stages, gates, promotes-or-defers) |
| src/ path | `src/m13_stcortex_writer/` with `mod.rs`, `gate.rs` (3-band LTP/LTD logic), `namespace.rs` (m9-coupled prefix validator + AP-Hab-11 hyphen-slug munge), `write.rs` (stcortex write + retry), `outbox.rs` (deferred-write JSONL buffer), `error.rs` |

## 3 — Upstream-IN (what arrives)

| Source | Wire shape | Notes |
|---|---|---|
| **m7** `workflow_runs` | `&WorkflowRunRow` (single row at promotion time) | m13 reads m7's SQLite file in **read-only mode** using the same `configure_connection()` pragma block as m7 |
| **m2** `stcortex_consumer` (CC-2 trust signal) | success of `register_consumer` call on m13 startup | **m13 MUST NOT write to stcortex until m2 has confirmed a fresh consumer registration** in the `workflow_trace_*` namespace. The DB-layer refuse-write reducer (per `CONSUMER-ONBOARDING.md`) blocks otherwise |
| **m11** `fitness_weighted_decay` | `DecayFactor` for prune-marker computation | Gap 2 owner; m13 consumes the value but does not author the formula |
| **ORAC blackboard `:8133/blackboard/substrate_LTP_density`** | floating-point ratio (Hebbian v3 telemetry) | lightweight HTTP GET before every promotion attempt; 5s timeout; defer on timeout |
| **m8** `povm_build_prereq` (CC-2) | compile-time `cfg(povm_calibrated)` — gates the crate build | per v1.3 § 1 m8; runtime band-check + compile-time gate must both agree (F7 mitigation) |

## 4 — Downstream-OUT (what departs)

| Destination | Wire shape | Notes |
|---|---|---|
| **stcortex `:3000`** — namespace `workflow_trace_*` | `CorrelationMemory` payload (§ 7) written via `write_memory_then` reducer-callback pattern | **AP30 prefix discipline** enforced at the m9 boundary; namespace constants live in `workflow-core::namespace` (never literals; per AP30) |
| **`~/.local/share/workflow-trace/deferred_writes.jsonl`** | newline-delimited JSON of un-promoted `CorrelationMemory` payloads | written when LTP/LTD ratio is below threshold OR stcortex unreachable. Background task in `wf-crystallise` re-attempts on 60s interval |
| **m31** `selector` (CC-5 loop closure) | stcortex pathway weights written by m13 (and downstream m42) flow back into m31's selection at the next cycle | indirect; m13 emits; m31 reads-back |

## 5 — Aspect-IN (Cluster D trust-layer wraps)

| Aspect | Wrapping point |
|---|---|
| **m8** `povm_build_prereq` | compile-time gate — the whole crate refuses to build if `cargo:rustc-cfg=povm_calibrated` is not set (env-only; not bypassable by `--features full`). Defense-in-depth alongside DB-layer refuse-write |
| **m9** `watcher_namespace_guard` | **write-time namespace-prefix validation at every m13 write boundary** — this is the **canonical m9 call site**. Any key not prefixed `workflow_trace_` is rejected with `NamespaceViolation` |
| **m10** `ember_ci_gate` | m13's write semantics are spec-bound; trait audit applies to module-level docstrings and error messages |
| **m11** `fitness_weighted_decay` | m13 reads `DecayFactor` for `prune-marker` computation; m11 reads m7 stats to produce the factor — m13 is a consumer, not a producer, of the decay value |

## 6 — Public API (lifted verbatim from vault canonical)

```rust
/// Build an m13 writer, registering the stcortex consumer on construction.
///
/// # Errors
///
/// Returns `WorkflowError::StcortexUnavailable` if the connection cannot
/// be established within `connect_timeout`. The caller should log and
/// continue without stcortex writes rather than panicking.
pub fn StcortexWriter::new(
    stcortex_uri: &str,
    orac_uri: &str,
    connect_timeout: Duration,
) -> Result<Self, WorkflowError>;

/// Attempt to promote a completed `WorkflowRunRow` to stcortex.
///
/// Performs:
///   1. m9 namespace-prefix validation (`namespace_key` must start with `workflow_trace_`).
///   2. LTP/LTD 3-band headroom check (HTTP GET to ORAC blackboard).
///   3. Either write immediately (band 1 or 2) or defer to local JSONL (band 3).
///
/// # Errors
///
/// - `WorkflowError::NamespaceViolation` if m9 rejects the key.
/// - `WorkflowError::WriteDeferred` if the LTP/LTD ratio is below threshold
///   or ORAC is unreachable; the caller should treat this as informational,
///   not fatal.
#[tracing::instrument(skip(self, run))]
pub fn promote_run(
    &self,
    run: &WorkflowRunRow,
    namespace_key: &str,
) -> Result<PromoteOutcome, WorkflowError>;

/// Retry all rows in the local JSONL defer buffer.
/// Called by the `wf-crystallise` background task on a 60-second interval.
#[tracing::instrument(skip(self))]
pub async fn flush_deferred(&self) -> Result<u32, WorkflowError>;
```

### Outcome enum

```rust
pub enum PromoteOutcome {
    Written { memory_id: i64 },
    WrittenUnderPressure { memory_id: i64, ltp_density: f64 },
    Deferred { jsonl_offset: u64, reason: DeferReason },
}

pub enum DeferReason {
    LtpBelowFloor { density: f64 },
    OracUnreachable,
    StcortexUnreachable,
}
```

All network-touching functions carry `#[tracing::instrument]` (god-tier rule — structured tracing on I/O paths).

## 7 — Wire shape (CorrelationMemory)

Each promoted row is written as a stcortex `memory` with structured content + optional vector:

```rust
pub struct CorrelationMemory {
    /// Namespace — MUST be prefixed `workflow_trace_`; enforced by m9.
    pub namespace: String,
    /// Memory type: always "semantic" for correlation rows in Phase A.
    pub memory_type: String,
    /// Content: JSON string encoding run_id, outcome, cost_tokens, cluster digest.
    pub content: String,
    /// Relevance: derived from outcome (ok=1.0, fail=0.5, abort=0.3, unknown=0.1).
    pub relevance: f32,
    /// Session ID: the atuin session-id of the originating run.
    pub session_id: String,
    /// Source tag: always "workflow-trace-m13".
    pub source_tag: Option<String>,
    /// Tensor: None in Phase A (F9 mitigation; fitness_dimension is zero-weight).
    pub tensor: Option<Vec<f32>>,
}
```

The `relevance` mapping is a **fixed heuristic for Phase A** — `ok=1.0, fail=0.5, abort=0.3, unknown=0.1`. It is NOT derived from `fitness_dimension` (F9 mitigation; that column is at zero weight). In a future phase relevance would derive from Hebbian feedback; in Phase A it is a static outcome ordinal.

The `tensor` field is **always `None`** in Phase A — populating it would constitute a fitness signal at workflow granularity, which is the F9 forbidden distortion.

## 8 — The 3-band LTP/LTD gate (the load-bearing invariant)

m13 reads `substrate_LTP_density` from ORAC's blackboard via lightweight HTTP GET before each promotion attempt. The thresholds align with the **Hebbian v3 reconciliation** (`~/projects/claude_code/Hebbian Deployment Plan v3 — Post-CR-2 Threshold Reconciliation.md`) — Phase 1 floor `substrate_LTP_density > 0.015` (workspace post-CR-2 metric was 0.018 at S1002127 boot, **Phase 1 PASSING**). The 3-band gate is **reconciled to the substrate_LTP_density scale** per D-E (Luke S1002127): Phase 1 floor = **0.015**, Phase 2 target = 0.05, Phase 3 target = 0.10 (Phase 2/3 provisional pending 30-day baseline from 2026-05-17, ending 2026-06-16).

| Condition | Band | Action |
|---|---|---|
| `substrate_LTP_density >= 0.10` (Phase 3 target) | **proceed** | Write proceeds normally; `PromoteOutcome::Written { memory_id }` |
| `0.015 <= substrate_LTP_density < 0.10` (above Phase 1 floor, below Phase 3) | **warn-and-proceed** | Write proceeds; `tracing::warn!` emitted; row tagged `promoted_under_pressure = true` in payload; `PromoteOutcome::WrittenUnderPressure { memory_id, ltp_density }` |
| `substrate_LTP_density < 0.015` (below Phase 1 floor) | **defer** | Row appended to `~/.local/share/workflow-trace/deferred_writes.jsonl`; no stcortex write; `PromoteOutcome::Deferred { ... LtpBelowFloor }` |
| `substrate_LTP_density` unavailable (ORAC 5xx, 5s timeout) | **defer** | Same JSONL path; `tracing::error!` emitted; `PromoteOutcome::Deferred { ... OracUnreachable }` — never blocks the calling thread |

**Why these thresholds:** promoting correlation rows when the substrate is in LTD dominance amplifies suppression signals — the pathway weights being written would themselves be subject to LTD decay before the next learning step. The backpressure check ensures m13 writes only when the substrate is in a receptive (LTP-dominant or balanced) state. (vault canonical § m13 Why this threshold.)

**Reconciliation note (UPDATED per D-E, Luke S1002127):** CLAUDE.local.md (workspace) Hebbian v3 row reports `substrate_LTP_density ≈ 0.018` (Phase 1 PASSING > 0.015). m13's 3-band gate is now **directly aligned with the substrate_LTP_density scale** — the legacy LTP/LTD ratio scale (`>0.15`) is **deprecated** (it was POVM `learning_health` pre-CR-2; 13.6× inflation factor confirmed by S1001971 live re-measurement). Phase 2 (>0.05) + Phase 3 (>0.10) thresholds remain provisional pending 30-day baseline observation window ending 2026-06-16. m13 reads `substrate_LTP_density` from ORAC blackboard and applies the reconciled floor of `0.015`.

References: [[Hebbian Deployment Plan v3 — Post-CR-2 Threshold Reconciliation]] (Watcher-authored, Zen-audited PASS-WITH-MINOR-AMEND 2026-05-17 `agent-cross-talk/2026-05-16T224430Z_zen_hebbian_v3_reconciliation_audit.md`).

**LTP/LTD scale reconciliation — DECIDED (drift originally tracked as vault `>0.15` vs workspace `0.018`; no dedicated § 12 Open Questions in m13, anchor lands here in § 8):** **DECIDED 2026-05-17 (S1002127, Luke directive "best practice + impactful performance"):** Threshold reconciled to **substrate_LTP_density > 0.015** (Phase 1 floor per workspace post-CR-2 metric). Deprecate the old > 0.15 scale (was POVM `learning_health` pre-CR-2; 13.6× inflation factor). Phase 2/3 thresholds (>0.05/>0.10) remain provisional pending 30-day Hebbian v3 reconciliation baseline observation window ending 2026-06-16. References [[Hebbian Deployment Plan v3 — Post-CR-2 Threshold Reconciliation]] for full reconciliation context.

### Deferred-write JSONL buffer

Append-only JSONL at `~/.local/share/workflow-trace/deferred_writes.jsonl`:

```
{"ts":"2026-05-17T11:04:12Z","memory":{"namespace":"workflow_trace_correlations","content":"...","relevance":0.5,"session_id":"abc"},"reason":{"LtpBelowFloor":{"density":0.012}}}
{"ts":"2026-05-17T11:09:33Z","memory":{...},"reason":"OracUnreachable"}
```

A background task in `wf-crystallise` polls every 60s. When ORAC becomes reachable AND `substrate_LTP_density >= 0.015` (Phase 1 floor per D-E), the task replays the JSONL line-by-line, removing lines on successful promotion. SHA-fingerprinted writes (AP-Drift-08 mitigation).

## 9 — CC-2 trust-signal dependency (consumer registration before any write)

Per `CONSUMER-ONBOARDING.md` and v1.3 § 2 hard-refusals, the DB-layer refuse-write reducer blocks any write from an un-registered consumer. m13 satisfies this by registering on startup:

```rust
impl StcortexWriter {
    fn new(...) -> Result<Self, WorkflowError> {
        let conn = DbConnection::builder()
            .with_uri(stcortex_uri)
            .with_database_name("stcortex")
            .on_connect(|conn, _ident, _addr| {
                // CC-2: register consumer idempotently.
                // Stable name: "workflow-trace-m13"
                // Namespace: "workflow_trace_*"
                // Transport: "cli"
                register_consumer_idempotent(conn, "workflow-trace-m13", "workflow_trace_*", "cli")
            })
            .build()
            .map_err(|e| WorkflowError::StcortexUnavailable(e.to_string()))?;
        // ...
    }
}
```

m13 RELIES ON the DB-layer refuse-write architectural commitment — it does NOT re-implement the check. What m13 owns is the call to `register_consumer_idempotent` on construction, following the onboarding checklist verbatim (stable name, namespace, transport).

## 10 — Tests (60 minimum; per `TEST_DISCIPLINE.md` row m13)

Allocation lifted from V7 cluster-C plan § m13 Test-pattern allocation:

| Pattern | Count | Coverage |
|---|---|---|
| F-Unit | 30 | 3-band gate per band · namespace-prefix validation per pass/fail · **AP-Hab-11 hyphen-slug munge** (S1001757 regression slot) · outbox JSONL write per success/fail · retry-with-jitter per attempt · relevance mapping per outcome arm |
| F-Property | 5 | namespace invariant: `for_all (payload: Payload) -> namespace_of(write(payload)).starts_with("workflow_trace_")` · outbox roundtrip · 3-band gate monotonic in `substrate_LTP_density` · `tensor` always `None` (F9) · `register_consumer` idempotency |
| F-Fuzz | 0 | — |
| F-Integration | 15 | m13 ↔ stcortex `:3000` live · m13 ↔ ORAC blackboard `:8133/blackboard/substrate_LTP_density` · 3-band gate end-to-end per band · outbox-replay after stcortex recovery · graceful degradation (ORAC 5xx → defer; not panic) · CC-2 register-then-write happy path |
| F-Contract | 5 | stcortex write-payload schema vs `stcortex_API.md` accepted format · outbox JSONL schema insta snapshot · namespace string-format stability · `CorrelationMemory` insta snapshot · ORAC blackboard read-path tolerance to schema additions |
| F-Regression | 4 | **AP30** regression slot · **AP-Hab-11 hyphen-slug munge** regression (S1001757) · 3-band-flip regression (CR-2-fixed thresholds) · outbox-loss regression (SHA fingerprint) |
| F-Mutation | budget | ≥70% on `gate.rs` + `namespace.rs` |

Key edge cases (from vault § Test density target):
- **Namespace guard:** `promote_run` with a key not prefixed `workflow_trace_` returns `Err(NamespaceViolation)`.
- **LTP/LTD threshold branches:** mock the ORAC probe; assert all three bands produce the correct `PromoteOutcome` variant.
- **Deferred write JSONL:** after a deferred write, the JSONL file contains exactly one line; `flush_deferred()` re-attempts and removes the line on success.
- **Relevance mapping:** `ok→1.0`, `fail→0.5`, `abort→0.3`, `unknown→0.1`.
- **Graceful degradation:** when ORAC returns 5xx or times out, `promote_run` returns `Deferred { ... OracUnreachable }`, never propagates the network error, never panics.
- **F9 invariant:** every constructed `CorrelationMemory.tensor == None`.

Integration tests use **real local stcortex** + **real local ORAC**, never mocks (per TEST_DISCIPLINE § Integration-test pattern; AP-Test-03 avoidance).

## 11 — Reuse + boilerplate lift

| Source | Lift % | What |
|---|---|---|
| `02-stcortex-consumer/capacity.rs` | **90%** | `DbConnection::builder()` + `with_uri()` + `with_database_name()` · `on_connect()` callback registration · `write_memory_then()` / `write_pathway_then()` reducer-callback with `Ok(Ok(()))` vs `Err(...)` discrimination · capacity-probe atomic counter pattern for success/failure accounting |
| `CONSUMER-ONBOARDING.md` refuse-write reducer template | architectural | DB-layer enforcement — m13 does not re-implement; m13 just registers correctly |
| `03-sqlite-multi-db/m06_schema.rs` `open_database()` | reused | configure_connection pragma block to open m7's SQLite file read-only when constructing payload |
| ME v2 outbox-first JSONL pattern | ~70% | `outbox.rs` (per S106 CC-5 substrate learning loop discipline) |
| POVM `/learning_health` HTTP client → adapted to ORAC `/blackboard/...` | ~80% | shared `HealthClient`-like pattern from `m40_42_common` |
| 3-band gate | **0% (fresh)** | ~25 LOC novel; thresholds align with CR-2-reconciled Hebbian v3 substrate_LTP_density ranges |
| AP-Hab-11 hyphen-slug munge | **0% (fresh)** | ~5 LOC novel; hyphen→underscore at write-time; verified post-write via `stcortex inspect` |

Net lift across m13: ~45-50%. Fresh authorship in the 3-band gate, the hyphen-slug munge, and the AP30 prefix coupling to m9.

## 12 — Graceful degradation (substrate may be down; m7 is source of truth)

m13 is **designed to degrade gracefully** when stcortex or ORAC is unavailable. The entire stcortex write path is **optional** for the core recording loop — m7 is the source of truth, not stcortex.

If stcortex is down:
1. `StcortexWriter::new()` returns `Err(StcortexUnavailable)`.
2. `wf-crystallise` logs the error at `tracing::warn!` level and proceeds **without an m13 instance**.
3. m7 rows are still written; m12 reports still render; m14 evidence aggregation still runs.
4. When stcortex recovers, the deferred JSONL file (if it exists) is flushed on the next 60s interval.

If ORAC is down (substrate-headroom probe unreachable):
1. Every `promote_run` returns `PromoteOutcome::Deferred { ... OracUnreachable }`.
2. Rows accumulate in the deferred JSONL buffer.
3. When ORAC recovers and the band check passes, the background task drains the buffer.

This matches the "**probe, don't block**" operational principle (workspace CLAUDE.md § Habitat Operations).

## 13 — Cross-cluster contracts owned + consumed

m13 owns **no** cross-cluster contract directly (m7 owns CC-1; m11 owns Gap 2 decay; m9 owns AP30). m13 is the **canonical participant** in CC-2 and CC-5:

### CC-2 — Trust Layer Woven (consumed, with the canonical m9 call site)

m13 is the **prime call site for m9 namespace-prefix validation** — every stcortex write goes through `namespace::check(key)` before the reducer-callback fires. The DB-layer refuse-write enforces the CC-2 register-then-write architectural commitment; m13's job is to register correctly on startup (stable name `workflow-trace-m13`, namespace `workflow_trace_*`, transport `cli`).

### CC-5 — Substrate Learning Loop (canonical write-side participant)

m13's stcortex writes feed back into m31's selector at the next cycle via stcortex pathway.weight delta. m13 is therefore a **critical CC-5 participant**. The loop closure: m7 records → m13 promotes → stcortex pathway.weight updates → m31 reads weights at selection → m32 dispatches → m40/m41/m42 emit results → m7 records the next run. m13 is the canonical promotion gate.

Synergy file: [`../../synergies/CC-5.md`](../../synergies/CC-5.md) (TBD Wave 2).

## Failure-modes covered (subset of `ANTIPATTERNS_REGISTER`)

- **AP30 (namespace prefix discipline)** — **prime owner at the write layer.** m9-coupled validator refuses any write missing `workflow_trace_` prefix. Tested by F-Property invariant.
- **AP-Hab-11 (hyphen-slug stcortex munge)** — **prime owner.** Hyphens → underscores at write-time; verified post-write via `stcortex inspect`. S1001757 regression slot.
- **3-band LTP/LTD gate** — F-WT-Substrate failure mode (engine writing to a degraded substrate); deferred to outbox below 0.05.
- **F7 (CR-2 graceful-degrade pretend-fix)** — m13 hard-couples to m8's compile-time `cfg(povm_calibrated)` AND runtime band-check; both must agree.
- **AP-Drift-06 (bridge contract drift)** — `bridge-contract` skill run pre-merge on m13's outbox JSONL schema vs stcortex accepted format.
- **AP-Drift-08 (push state inconsistent across remotes)** — SHA-fingerprinted outbox writes.
- **AP-V7-01 (Health-200 ≠ behaviour-verified)** — m13 does NOT trust a 200 on stcortex `/health`; it relies on the DB-layer reducer-callback `Ok(Ok(()))` discrimination as the success signal.
- **AP-V7-04 (POVM dual-path)** — RESOLVED per 2026-05-17 ADR; m13 has no POVM dependency. workflow-trace is POVM-decoupled per Genesis v1.3 § 2.

## Atuin trajectory anchor

- `wt-substrate-pulse` (proposed; reads m13 outbox depth + last-successful-write timestamp).
- `stcortex-probe` (atuin script — m13's write is the inverse of m2's read; shares probe pattern).

## Watcher class pre-position

- **Class A** — first stcortex write post-Genesis (the substrate-write activation).
- **Class B** — hand-off boundary crossing — every stcortex write is a cross-substrate handoff (per Phase 3 cross-substrate calls).
- **Class D** — four-surface drift if m13 writes succeed but corresponding ai_docs / vault / CLAUDE.local.md anchors are missing.
- **Class I** — Hebbian silence — m13's 3-band gate is the direct measure; if `substrate_LTP_density < 0.05` for N consecutive write attempts, Class-I escalates from "firing" to "sustained".

## Implementation order within Cluster C

m13 **last** — depends on m7 (`WorkflowRunRow` schema), m9 (namespace guard), and the ORAC blackboard probe. The deferred-write JSONL path is testable with mock ORAC responses; the live stcortex write needs the real `:3000` endpoint. (Vault canonical § Implementation sequence.)

---

*m13 spec authored 2026-05-17 (S1001982) by Command for the Cluster C author wave. Planning-only; HOLD-v2 active; no code until G1-G9 clear. Substrate condition note (post-D-E reconciliation per Luke S1002127): workspace CLAUDE.local.md reports `substrate_LTP_density ≈ 0.018` (Phase 1 PASSING > 0.015 floor); legacy LTP/LTD ratio 0.043 (35× below healthy 1.5-4.0; deprecated as gate metric per D-E — 13.6× inflation factor versus reconciled substrate_LTP_density). The 3-band gate is now aligned with substrate_LTP_density directly: at 0.018, current state routes writes to **band 2 (warn-and-proceed)** — above the 0.015 Phase 1 floor, below the 0.10 Phase 3 target.*

> Sister-module bottom anchors: [m7](m7_workflow_runs.md) · [m12](m12_cli_reports.md) · [m13](m13_stcortex_writer.md) · vault [[cluster-C-correlation-output]] · canonical V7 [cluster-C plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-C.md)
