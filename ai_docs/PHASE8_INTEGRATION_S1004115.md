# Phase 8 — Integration + End-to-End + Substrate-Frame Folds

> **Authored:** 2026-05-23 (S1004115, Plan v2 Phase 8)
> **Per plan §3 Phase 8:** environment matrix · Conductor enforcement assertion ·
> substrate-side load observation · clock-coherence enumeration · gate + mutants · commit
> **§15 decisions cited:** D34 (live `--execute` post-M0) · D37 (engine-timed proxy) · D38 (clock enumeration)
> **Back to:** [`WORKFLOW_TRACE_COMPLETION_PLAN_V2_S1004115.md`](WORKFLOW_TRACE_COMPLETION_PLAN_V2_S1004115.md) · [CLAUDE.local.md](../CLAUDE.local.md)

---

## 1. Environment matrix (Plan v2 §3 Phase 8 step 1 — gap C-10) — DOC

Two execution modes; M0 ships **only** the first as fully-verified.

### 1a. Offline / dry-run — always runs, no live services required

`wf-crystallise` + `wf-dispatch --dry-run` operate against **file-backed fixtures only**:

| Stage | What it reads / writes | Live-service dependence |
|-------|------------------------|-------------------------|
| m1 atuin consumer | reads atuin `history.db` (SQLite) | none (file-backed) |
| m3 injection.db consumer | reads `injection.db` (SQLite) | none (file-backed) |
| m4 cascade correlator | in-process | none |
| m7 workflow runs | reads/writes `workflow_runs.db` (SQLite) | none |
| m13 stcortex writer | writes outbox file when `--offline` | none (outbox-first; ORAC unreachable accepted) |
| m14 lift aggregator | in-process | none |
| m15 pressure register | in-process | none (Phase 7 CC-7 wiring) |
| m20 / m21 / m22 / m23 | in-process | none |
| m30 / m31 bank + selector | in-process; loads JSONL | none |
| m33 verifier | in-process (Phase 6a–6d named verifiers) | none |
| m32 dispatcher (`--dry-run`) | verifies + selects but **does NOT contact Conductor** | none (default-safe) |
| m40 nexus emit | writes outbox when offline | none |
| m42 stcortex emit | writes via m13 outbox | none |

**Verification gate:** the existing `tests/wf_crystallise_integration.rs` + `tests/wf_dispatch_integration.rs` + the
~33 module/cross-cluster integration suites exercise this path. Gate-green on every commit = full
verification of the offline / dry-run mode at M0.

### 1b. Live `--execute` — requires OP-1, deferred to post-M0 soak (per §15 D34)

| Service | Default endpoint | Required for `--execute` | Live-verified at M0? |
|---------|------------------|--------------------------|----------------------|
| HABITAT-CONDUCTOR | `http://127.0.0.1:8141` | YES (`wf-dispatch` posts here) | **NO** (D34: dry-run sufficient for M0; `--execute` is a documented post-M0 limitation) |
| ORAC / m40 nexus | `http://127.0.0.1:8133/nexus/push` | NO (offline path uses outbox) | n/a |
| stcortex `:3000` | SpacetimeDB module | NO (offline path uses m13 outbox) | n/a |
| LCM `:8082` | RPC | NO (offline path no-op) | n/a |
| synthex-v2 `:8092` | nexus / pulses | NO (offline path no-op) | n/a |

**M0 acceptance per §15 D34:** the `--execute` wire-contract (`POST {conductor_url}/dispatch` with workflow-id +
escape-surface profile) is exercised by the unit tests in `src/orchestration/dispatch.rs` against a mock
`ConductorClient`; the **production HTTP client** (`HttpConductorClient`) and the round-trip against a live
Conductor at `:8141` are explicitly NOT verified at M0. This is recorded here as a known M0 limitation —
not a regression.

Live `--execute` verification happens during the post-M0 dispatch soak (per §15 D33 — Conductor bring-up
post-M0, D35 — 24-hour NoOp soak, D36 — Watcher ☤ carries the soak).

---

## 2. Conductor enforcement-state assertion (Plan v2 §3 Phase 8 step 2 — gap NA-4) — CODE (post-merge of 6e/6f)

**Requirement:** `wf-dispatch` must emit an observable warning when it dispatches `Approve` verdicts into a
Conductor instance with `CONDUCTOR_ENFORCEMENT_ENABLED=0` — those verdicts are **advisory, not enforced**,
and the operator must know.

**Design:**

- At `wf-dispatch::run`, after the m33 aggregate yields `AllApprove` AND before m32 dispatches, read the
  `CONDUCTOR_ENFORCEMENT_ENABLED` environment variable.
- If unset OR `"0"` → emit `tracing::warn!` once per `run` invocation: "Conductor enforcement disabled
  (CONDUCTOR_ENFORCEMENT_ENABLED=0); m33 Approve verdicts are advisory only; dispatch proceeds but
  enforcement is the Conductor's responsibility".
- If `"1"` → silent pass-through (the production case).

**Why not a hard refuse:** Per §15 D34, the dispatch is allowed even when enforcement is disabled —
the dry-run-by-default discipline makes the default safe. The warning is **observability**, not a gate.

**Implementation file:** `src/orchestration/dispatch.rs::run` — after the verifier aggregate, before the
dispatch loop. Estimated +15–25 LOC including 2–3 unit tests (env-var Some("1") = silent;
Some("0") = warn; None = warn).

---

## 3. Substrate-side load observation (Plan v2 §3 Phase 8 step 3 — gap NA-2) — CODE (post-merge of 6e/6f)

**Requirement (per §15 D37):** measure atuin's own foreground write latency **before / during / after**
`wf-crystallise` reads, to detect any read-side contention the engine introduces. **D37 explicitly
labels this an engine-timed proxy** — a true substrate-emitted contention signal is v0.2.0 (NA-GAP-04).

**Design:**

- At `crystallise::run`, immediately before and immediately after the m1 atuin read, time a small
  synthetic write against the atuin DB (an INSERT into a `_workflow_trace_probe` table that we create
  + drop in a transaction so the production atuin shape is untouched).
- Compare `before_latency_ms` vs `after_latency_ms`. If `after - before > THRESHOLD_MS` (initial:
  100 ms), record in the `Report` as a `substrate_load_observation: SubstrateLoadObservation`
  field with `{ before_ms, after_ms, delta_ms, threshold_exceeded: bool }`.
- The observation is recorded but **does not block** — it is an observability data point per D37
  ("engine-timed proxy adequate for M0").

**Why this is a proxy (per Phase 2 audit §3 self-check — Frame-A measurement of a Frame-B property):**
A true substrate-frame mechanism would have atuin emit its own contention signal; our timing is the
engine measuring its own latency, not the substrate's. This is honestly labelled in the docstring.

**Implementation file:** `src/orchestration/crystallise.rs::run` — wrapping the m1 stage. Estimated
+40–60 LOC including a new `SubstrateLoadObservation` type in `Report`, plus 2 unit tests
(observation populated; threshold flag fires).

---

## 4. Clock-coherence enumeration (Plan v2 §3 Phase 8 step 4 — gap NA-5) — DOC

The CC-5 substrate-learning loop (proposal → dispatch → outcome → feedback → proposal-fitness-decay)
crosses **at least five distinct clocks**. The engine assumes each crossing is monotone and
agree-with-each-other; per §15 D38 we **document** the assumptions here for M0 and **defer live
reconciliation** to v0.2.0.

| # | Clock | Source file:line | What it measures | Monotonicity guarantee |
|---|-------|------------------|------------------|------------------------|
| C1 | m11 wall-clock (`chrono_now_ms`) | `src/m11_fitness_weighted_decay/consolidation.rs:125` | `SystemTime::now()` ms since epoch, returns `Option<i64>` (None on clock-fault) | None (system wall clock; subject to NTP adjustment); gate at consolidation.rs:204 explicitly skips future-dated `last_run_ms` to handle skew. |
| C2 | m13 outbox / stcortex writer wall-clock | `src/m13_stcortex_writer/mod.rs:587` | Independent `now_ms() -> Option<i64>` (parallel to C1) | None; tag-and-defer on `None` (C1 pattern). |
| C3 | injection.db TTL sweep | `~/.local/share/habitat/injection.db` hourly sweep (process-external) | Sweeps causal-chain rows older than the TTL window | Not engine-controlled; the engine ASSUMES rows survive between read cycles within the sweep interval. |
| C4 | stcortex pathway decay | SpacetimeDB module on `:3000` (process-external) | Tick-based pathway-weight decay (per stcortex API) | Independent ticking; engine ASSUMES per-pathway decay rates are stable across consolidation cycles. |
| C5 | atuin checkpoint | `~/.local/share/atuin/history.db` (process-external) | atuin writes new rows asynchronously as the user types | Append-only WAL semantics; engine reads the snapshot at `m1::read_atuin_rows`. |
| C6 | m15 pressure register | `src/m15_pressure/mod.rs` (Phase 7 wiring) | In-process pressure level | Monotone within a single `wf-crystallise` invocation. |

**Engine assumptions per crossing (documented per D38; live reconciliation = v0.2.0 NA-GAP-07):**

- **C1 vs C2** — both wall-clocks; on the same host they agree to clock-skew (gate at C1 handles
  future-dated `last_run_ms` by skipping that workflow's decay this cycle — see m11 consolidation
  step 0). **Assumption: |C1 − C2| < 1 second across a single `wf-crystallise` invocation.**
- **C1 vs C3** — injection.db sweep is hourly; m11 decay runs per `wf-crystallise` invocation
  (sub-second). **Assumption: an injection-row read by m3 will not be swept between read and decay
  cycle's reference of it.**
- **C1 vs C4** — stcortex's per-pathway decay runs independently; m13 writes new pathways with
  C1-stamped timestamps. **Assumption: stcortex decay does not invalidate a pathway in less time
  than a single `wf-crystallise` invocation takes (current bound: < 1 minute end-to-end).**
- **C1 vs C5** — atuin rows ingested by m1 carry their own `timestamp` field (atuin's clock); m11's
  recency is computed against the engine's `chrono_now_ms` (C1). **Assumption: atuin timestamps and
  engine wall-clock agree to clock-skew (same host, same NTP).**
- **C5 vs C2** — m1 reads atuin under a snapshot; m13 writes outbox under C2. If atuin's
  background-writer thread interleaves with m1's read, the engine sees a partial snapshot.
  **Assumption: atuin's WAL semantics make the snapshot consistent at read time (atuin's own
  guarantee — not engine-enforced).**

**Coupling risks (named, not solved at M0):**

- A failed NTP sync on the host could cause C1 to jump backwards; the m11 gate skips the
  consolidation cycle (no decay, no corruption), but recency-factor effects on selection are
  silently distorted for one cycle.
- An injection.db TTL sweep mid-`wf-crystallise` would silently drop rows m3 had already read; the
  read-side rows survive in memory but the on-disk lineage is broken (only matters for cross-run
  causal correlation, not for the current run's proposals).
- stcortex's pathway decay is the only clock NOT directly readable from the engine; the engine
  emits pathways and assumes they persist for at least one further consolidation cycle.

**Live reconciliation = v0.2.0 (NA-GAP-07 substrate-drift canary):** a periodic prober that
samples each clock and asserts the agree-to-skew envelope. Out of M0 scope per §15 D38; documented
here so the assumption is visible.

---

## 5. Phase 8 gate + cargo-mutants (Plan v2 §3 Phase 8 step 5) — EXEC (post-merge)

Per §15 D28 (hold ≥ 96.3 % mutation kill-rate; re-verify every module touched in Phases 5–7):

- After 6e + 6f + Phase 7 are merged into `main`:
  - Full 4-stage gate (`check → clippy → pedantic → test --release --all-targets --all-features`).
  - `cargo mutants` scoped to modules touched in Phases 3–7:
    `m20_prefixspan, m22_kmeans, m23_proposer, m15_pressure, m9_watcher_namespace_guard, m32_dispatcher, m33_verifier, m40_nexus_emit, m10_ember_ci_gate, orchestration/{crystallise,dispatch}`.
  - Hold ≥ 96.3 % kill-rate; record exact `caught/missed/timeout/unviable` counts.

Estimated runtime: cargo-mutants per-module budget is multi-hour at god-tier. May launch in background
and proceed with Phase 9/10 paperwork while it runs.

---

## 6. Phase 8 commit (Plan v2 §3 Phase 8 step 6) — EXEC

Bundle steps 1+4 (docs) + 2+3 (code) + 5 (gate evidence) into ONE Phase 8 commit per D44 (one commit per
phase). Commit message includes the D43 done-evidence block + the mutation-kill summary.

---

## Bidirectional anchor

> Back to: [`WORKFLOW_TRACE_COMPLETION_PLAN_V2_S1004115.md`](WORKFLOW_TRACE_COMPLETION_PLAN_V2_S1004115.md) · [`PHASE2_AUDIT_S1004115.md`](PHASE2_AUDIT_S1004115.md) · [CLAUDE.local.md](../CLAUDE.local.md)

— Phase 8 design doc, S1004115. Steps 1 + 4 ratified at authoring time; steps 2 + 3 implementation pending merges of 6e + 6f.
