---
title: CONTROL_FLOW — when each module fires + sequence diagrams for key flows
kind: planning-only · operational view · complements canonical V7 ULTRAMAP
status: Wave-2 author deliverable; no source authored
date: 2026-05-17
---

# CONTROL_FLOW — When does each module fire?

> **Back to:** [`README.md`](README.md) · [`../ARCHITECTURE.md`](../ARCHITECTURE.md) · canonical [`../ai_docs/optimisation-v7/ULTRAMAP.md`](../ai_docs/optimisation-v7/ULTRAMAP.md) · siblings [`MODULE_DEPENDENCY_GRAPH.md`](MODULE_DEPENDENCY_GRAPH.md) · [`DATA_FLOW.md`](DATA_FLOW.md) · [`CONTEXTUAL_FLOW.md`](CONTEXTUAL_FLOW.md) · [`INVARIANT_MAP.md`](INVARIANT_MAP.md)
>
> **Purpose:** the trigger taxonomy for each of the 26 modules — when does it fire, who pulls it, what cadence does it run at. Edges in [DATA_FLOW.md](DATA_FLOW.md) tell you what shape the data is in transit; edges here tell you *when* the data moves. Without this view, the static-flow diagram suggests everything runs synchronously per call — but in reality the engine is a heterogeneous mix of cron loops, substrate event callbacks, operator-driven invocations, and outbox-first retry loops with circuit breakers.

---

## Trigger taxonomy

The engine's 26 modules fall into five trigger classes. Most modules have a single primary trigger; a few (m13, m32) bridge classes.

| Class | Description | Modules in class |
|---|---|---|
| **Cron-driven** | fires on a scheduled tick (every N minutes / hours) | m1, m4, m5, m6 (every N min); m11, m14 (hourly); m30 sunset sweep (daily) |
| **Event-driven (substrate callback)** | fires on a substrate emit (reducer callback, fs watch, signal) | m2 (stcortex reducer-callback); m15 emit (on any in-engine forbidden-verb or scope-relaxation pressure); m33 verifier (on bank insert via m30) |
| **Operator-driven** | fires when a human runs a `wf-crystallise` or `wf-dispatch` CLI subcommand | m12 (`report`); m20-m23 (`propose`); m30 (`bank accept`); m32 (`dispatch`); m33 (`verify`) |
| **Sequence-driven** | fires as a fixed-order in-process sequence inside another module's call | m32 5-check (within `dispatch()`); m23 evidence-gate then build (within `ProposalBuilder::build()`); m40/m41/m42 fan-out from m32 |
| **Retry/backoff-driven** | fires from an outbox replay loop with circuit-breaker policy | m40 outbox drain; m41 outbox drain; m42 outbox drain (all share `m40_42_common::breaker`) |

---

## Per-module trigger reference

### Cluster A — Ingest

| Module | Trigger class | Cadence | Trigger source |
|---|---|---|---|
| **m1** atuin_consumer | cron | every N min (configurable; default 5 min) | `wf-crystallise` main loop scheduler |
| **m2** stcortex_consumer | event-driven | on stcortex reducer-callback (push) | stcortex SpacetimeDB module; narrowed-scope subscription only |
| **m3** injection_db_consumer | cron | every N min (default 5 min) | `wf-crystallise` main loop scheduler |

### Cluster B — Observation

| Module | Trigger class | Cadence | Trigger source |
|---|---|---|---|
| **m4** cascade_correlator | cron | downstream of m1 tick | invoked from m1 page-completion callback |
| **m5** battern_step_record | cron | downstream of m1 tick | invoked from m1 page-completion callback |
| **m6** context_cost | cron | downstream of m1 tick (plus separate EMA refresh every 1h) | dual: m1 page-completion + dedicated hourly EMA refresh |

### Cluster C — Correlation + Output

| Module | Trigger class | Cadence | Trigger source |
|---|---|---|---|
| **m7** workflow_runs hub | cron (write) | on every m4/m5/m6 emission | passive hub — receives writes from B-cluster modules |
| **m12** cli_reports | operator-driven | on demand | `wf-crystallise report [--json]` |
| **m13** stcortex_writer | event-driven + retry/backoff | on every m7 write that passes 3-band LTP/LTD gate + on m42 reinforce call + on outbox replay | three paths: direct from m7, indirect from m42, durability replay |

### Cluster D — Trust (aspect)

| Module | Trigger class | Cadence | Trigger source |
|---|---|---|---|
| **m8** build_prereq | **build-time only** | once per `cargo build` | `cargo:rustc-cfg=povm_calibrated` emitted from `build.rs` |
| **m9** namespace_guard | sequence-driven | on every m13/m42 write attempt | called inline from writers before substrate egress |
| **m10** ember_ci_gate | **CI-time only** | once per CI run | invoked by CI workflow on PR / push |
| **m11** freq×fitness×recency decay | cron | hourly + on every m31 selection | dual: hourly batch decay write + on-demand multiplier read from m31 |

### Cluster E — Evidence + Pressure

| Module | Trigger class | Cadence | Trigger source |
|---|---|---|---|
| **m14** outcome_lift | cron (Wilson CI recompute) | hourly | dedicated hourly Wilson CI refresh on `workflow_runs` aggregates |
| **m15** pressure_register | event-driven | on every in-engine pressure event | called inline from m20-m23 (sample-size pressure), m32 (forbidden-verb), watcher silence detection, m33 (escape-surface escalation) |

### Cluster F — Iteration (KEYSTONE)

| Module | Trigger class | Cadence | Trigger source |
|---|---|---|---|
| **m20** prefixspan_miner | operator-driven (primary) + cron (secondary) | on `wf-crystallise propose` + nightly batch | dual: interactive propose path + nightly batch run for proposal-bank freshness |
| **m21** variant_builder | sequence-driven | within m20-orchestrated propose | called as next step in `propose` orchestration |
| **m22** kmeans_feature | sequence-driven | within propose | next step |
| **m23** workflow_proposer | sequence-driven | within propose | final step — emits proposals to operator review queue |

### Cluster G — Bank / Select / Dispatch / Verify (`wf-dispatch` binary)

| Module | Trigger class | Cadence | Trigger source |
|---|---|---|---|
| **m30** curated_bank | operator-driven (writes) + cron (sunset sweep) | on `wf-dispatch bank accept` + daily sunset sweep at 03:00 UTC | dual: operator inserts + scheduled sunset/decay pass |
| **m31** selector | sequence-driven | inside `wf-dispatch select` and inside m32 dispatch entry | called from CLI + as preamble to m32 dispatch |
| **m32** conductor_dispatcher | operator-driven | on `wf-dispatch dispatch <id>` | operator invocation; **never auto-fires** |
| **m33** verifier | event-driven (bank insert) + operator-driven (re-verify) + cron (TTL sweep) | on m30 insert + on `wf-dispatch verify <id>` + every 6h sweep for TTL-expiring entries | triple: insert trigger, manual re-verify, scheduled expiry sweep |

### Cluster H — Substrate Feedback

| Module | Trigger class | Cadence | Trigger source |
|---|---|---|---|
| **m40** nexusevent_emit | sequence-driven (fan-out) + retry/backoff (outbox drain) | on every m32 dispatch outcome + outbox replay every 30s when wire down | dual path; circuit breaker opens after 2 consecutive failures |
| **m41** lcm_rpc | sequence-driven (fan-out) + retry/backoff | on every m32 dispatch outcome (if deploy-shaped) + outbox replay | same pattern as m40 |
| **m42** stcortex_emit | sequence-driven (fan-out) + retry/backoff | on every m32 dispatch outcome + outbox replay | same pattern; POVM path **decoupled** per 2026-05-17 ADR (m42 source has no POVM branch) |

---

## Sequence diagram (i) — m32 5-check pre-dispatch

The most-load-bearing sequence in the engine. Per [m32 spec § 1 third invariant](../ai_specs/modules/cluster-G/m32_conductor_dispatcher.md), order is contractual: failure at check N short-circuits with the correct typed `DispatchError`; checks N+1..5 do not run.

```mermaid
sequenceDiagram
    autonumber
    actor Op as Operator (CLI)
    participant M32 as m32 conductor_dispatcher
    participant M30 as m30 BankDb
    participant M33 as m33 VerifyCache
    participant Aud as dispatch_log.db (audit-first)
    participant Cnd as HABITAT-CONDUCTOR :8141

    Op->>M32: wf-dispatch dispatch <id>
    M32->>M30: get(workflow_id) → AcceptedWorkflow
    M32->>Aud: insert row (outcome: Pending) — AUDIT-FIRST
    Note over M32: 5-CHECK SEQUENCE (order contractual)

    M32->>Cnd: (1) GET :8141/health
    alt unhealthy
        Cnd-->>M32: refused / timeout
        M32-->>Op: Err(ConductorNotLive)
        M32->>Aud: patch row → outcome: Error("ConductorNotLive")
    else healthy
        Cnd-->>M32: 200 ok

        M32->>M33: (2) lookup VerifyCache(workflow_id)
        alt TTL expired or absent
            M33-->>M32: stale / missing
            M32-->>Op: Err(VerificationStale)
            M32->>Aud: patch row → outcome: Error("VerificationStale")
        else fresh
            M33-->>M32: VerificationReceipt

            M32->>M32: (3) hash(steps_json); compare vs receipt.definition_hash
            alt mismatch
                M32-->>Op: Err(DefinitionDrifted)
                M32->>Aud: patch row → outcome: Error("DefinitionDrifted")
            else match

                M32->>M32: (4) check sunset_at > now_ms
                alt sunset elapsed
                    M32-->>Op: Err(Sunset)
                    M32->>Aud: patch row → outcome: Error("Sunset")
                else not sunset

                    M32->>Aud: (5) lookup last_dispatched_at; check cooldown
                    alt cooldown active
                        M32-->>Op: Err(CooldownActive)
                        M32->>Aud: patch row → outcome: Error("CooldownActive")
                    else cooldown elapsed

                        Note over M32: ALL 5 PASS — emit Gap 3 banner per ResolvedStep
                        M32->>Op: print EscapeSurfaceProfile banner + trap annotations (stdout)
                        M32->>Cnd: POST :8141/dispatch (ConductorDispatchRequest, NDJSON)
                        alt accepted
                            Cnd-->>M32: 202 accepted
                            M32->>Aud: patch row → outcome: Accepted
                            M32-->>Op: Ok(WorkflowDispatchEvent)
                            M32->>M32: fan-out to m40/m41/m42 via tokio::mpsc
                        else rejected
                            Cnd-->>M32: 4xx { reason }
                            M32->>Aud: patch row → outcome: Rejected
                            M32-->>Op: Err(ConductorRejected)
                        end
                    end
                end
            end
        end
    end
```

**Refuse-mode** (B3 blocker active — `CONDUCTOR_DISPATCH_ENABLED != "1"`): `Dispatcher::Refuse` variant constructed at init; `dispatch()` returns `Err(ConductorDispatchDisabled)` without entering the sequence. This is enforced at the type level — the borrow checker prevents bypass (per [m32 spec § 3](../ai_specs/modules/cluster-G/m32_conductor_dispatcher.md)).

**Defense-in-depth on AP-V7-08 self-dispatch**: between check 5 and the Conductor POST, m32 inspects `resolved.iter().any(|s| s.kind == "m32_dispatch" || s.conductor_params.points_at_self())` and returns `Err(SelfDispatchRefused)` if any step targets m32 itself. m30 also refuses such workflows at admission — both gates active.

---

## Sequence diagram (ii) — m23 → m30 operator review (CC-4)

The single mandatory human-consent boundary in the engine. Per [m30 spec § 1 First invariant — AP-V7-07](../ai_specs/modules/cluster-G/m30_curated_bank.md), there is no code path that bypasses this.

```mermaid
sequenceDiagram
    autonumber
    actor Op as Operator (human terminal)
    participant CLI as wf-crystallise CLI
    participant M23 as m23 workflow_proposer
    participant Q as proposals table
    participant M30 as m30 BankDb
    participant M33 as m33 verifier
    participant M9 as m9 namespace_guard

    Op->>CLI: wf-crystallise propose
    CLI->>M23: build proposal slate
    M23->>M23: enforce CC-3 evidence gate (Option<Lift>::None → ProposalError::LiftEvidenceMissing)
    M23->>Q: insert proposals (top-K-by-distance N=3)
    M23-->>CLI: emit proposal IDs
    CLI-->>Op: render proposal slate (table + deviation rationale)

    Note over Op: human review — out-of-band; can take minutes/hours/days

    Op->>CLI: wf-crystallise propose accept <id>
    CLI->>CLI: synthesise HumanAcceptanceSignature { signed_at, terminal_fingerprint, accepted_by }
    CLI->>M9: validate namespace prefix
    M9-->>CLI: ok (workflow_trace_* OK)
    CLI->>M30: BankDb::accept(proposal_id, HumanAcceptanceSignature)

    alt accepted_by missing or invalid
        M30-->>CLI: Err(AcceptanceRequiresHumanSignature) — AP-V7-07
        CLI-->>Op: refusal message
    else valid signature
        M30->>M30: compute EscapeSurfaceProfile (Gap 3 ordinal)
        M30->>M30: set sunset_at = now + 120d
        M30->>M30: insert AcceptedWorkflow row (definition_hash frozen)
        M30->>M33: trigger 4-agent verifier (event-driven)
        M33->>M33: dispatch 4 agents (security-auditor, performance-engineer, silent-failure-hunter, zen)
        M33-->>M30: VerificationReceipt { verdict, ttl_expires_at = now + 7d, definition_hash }
        M30->>M9: validate namespace_prefix on emitted bank_id
        M9-->>M30: ok
        M30-->>CLI: Ok(workflow_id)
        CLI-->>Op: "Workflow <id> accepted into bank; verifier dispatched (TTL 7d)"
    end
```

**Why this matters:** F5 (bank creep) was identified in the gold-standards review as the engine's primary failure-mode-of-laziness. Without an explicit human signature, every confidence-thresholded auto-promote pathway eventually becomes the de-facto path — and the bank silently fills with unverified workflows. m30's `accepted_by: HumanAcceptanceSignature` parameter is the structural refusal.

---

## Sequence diagram (iii) — m40/m41/m42 outbox-first + circuit breaker

All three Cluster H modules share the `m40_42_common::breaker` shape. Per [`DATA_FLOW.md`](DATA_FLOW.md) Stage 8 and [m42 spec § 1 fourth invariant](../ai_specs/modules/cluster-H/m42_stcortex_emit.md), the wire attempt is *never* on the critical path — durability lives in the outbox JSONL, wire success is best-effort.

```mermaid
sequenceDiagram
    autonumber
    participant M32 as m32 dispatcher (fan-out)
    participant Mh as m40 / m41 / m42 (any H module)
    participant Out as outbox/m{N}/*.jsonl
    participant Brk as Breaker (state: Closed/Open/HalfOpen)
    participant Wire as substrate wire (SYNTHEX / LCM / stcortex)
    participant Met as metrics counters

    M32->>Mh: WorkflowDispatchEvent (mpsc; fire-and-forget)
    Mh->>Out: append JSONL + fsync — DURABILITY FIRST

    Mh->>Brk: check state
    alt Closed (healthy)
        Mh->>Wire: POST event
        alt success
            Wire-->>Mh: 2xx
            Mh->>Met: increment {accepted_total}
            Mh->>Out: mark line as ACKed (or delete on subsequent compaction)
            Mh->>Brk: record success
        else failure
            Wire-->>Mh: 4xx/5xx/timeout
            Mh->>Met: increment {failed_total}
            Mh->>Brk: record failure
            Note over Brk: after 2 consecutive failures → Open
        end
    else Open
        Mh->>Met: increment {circuit_open_total}
        Note over Mh: skip wire entirely; durable row already in outbox
        Mh-->>M32: (no-op return)
    else HalfOpen (probe)
        Mh->>Wire: probe POST (single event)
        alt success
            Brk->>Brk: → Closed
        else failure
            Brk->>Brk: → Open (back-off doubles)
        end
    end

    Note over Out,Wire: BACKGROUND OUTBOX DRAIN (every 30s when Brk == Closed)
    loop drain pending lines
        Out->>Wire: POST next pending event
        alt success
            Wire-->>Out: 2xx — mark ACKed
        else failure
            Out->>Brk: record failure
            Brk->>Brk: maybe → Open
        end
    end
```

**Why outbox-first:** stcortex / SYNTHEX / LCM down must **never** block the engine's dispatch loop. The outbox JSONL is the durable surface; the wire is a best-effort relay. Per [m42 spec § 1 fifth invariant](../ai_specs/modules/cluster-H/m42_stcortex_emit.md): substrate-unavailable returns `ReinforceOutcome::SubstrateUnavailable` (NOT silently falls back to POVM — POVM is decoupled).

**Why 2 consecutive failures:** the V7 framework sets the breaker threshold at 2 (not 5 or 10) because a flapping substrate is more dangerous than a definitively-down one — by opening fast, we prevent a storm of in-flight requests piling up against a degraded substrate. Half-open probes resume after a back-off period (default 30s; doubles per failed probe up to 5 min cap).

---

## Cron schedule summary

| Job | Cadence | Owner module | Why this cadence |
|---|---|---|---|
| atuin scan | every 5 min | m1 | matches typical user-cadence (~1-5 min between command sequences) |
| stcortex narrowed-scope poll | event-driven (push) | m2 | reducer-callback is push; no poll needed |
| injection.db scan | every 5 min | m3 | matches m1 cadence; aligned by main loop |
| context-cost EMA refresh | every 1 hour | m6 | EMA needs enough rows to be stable; hourly is sufficient |
| Wilson CI lift recompute | every 1 hour | m14 | downstream of m7 hub aggregates |
| m11 batch decay write | every 1 hour | m11 | compound decay updates slow enough to batch |
| m30 sunset sweep | daily at 03:00 UTC | m30 | low-traffic hour; idempotent |
| m33 TTL-expiring sweep | every 6 hours | m33 | trade-off: too-frequent wastes 4-agent budget; too-slow misses re-verify window |
| m40 outbox drain | every 30s (when breaker Closed) | m40 | balance: fast enough that backlog stays small; slow enough that idle wire doesn't churn |
| m41 outbox drain | every 30s | m41 | same |
| m42 outbox drain | every 30s | m42 | same |
| breaker half-open probe | back-off 30s → 5 min | m40_42_common | exponential back-off prevents probe-storm against a definitively-down substrate |

---

## Operator-driven CLI commands (full inventory)

Per [`../ARCHITECTURE.md`](../ARCHITECTURE.md) § Binary split and individual cluster specs:

### `wf-crystallise` subcommands

| Command | Fires modules | Trigger note |
|---|---|---|
| `wf-crystallise scan` | m1, m3 (m4/m5/m6/m7 cascade via callbacks) | manual cron-equivalent (force an ingest tick) |
| `wf-crystallise report [--json]` | m12 | reads m7 hub state |
| `wf-crystallise propose` | m20 → m21 → m22 → m23 | sequence; emits proposal slate |
| `wf-crystallise propose accept <id>` | m30 (write) → m33 (event-trigger) | **CC-4 mandatory human boundary** |
| `wf-crystallise pressure [--list]` | m15 | reads pressure register |

### `wf-dispatch` subcommands

| Command | Fires modules | Trigger note |
|---|---|---|
| `wf-dispatch bank list` | m30 (read) | eligible rows only (`sunset_at > now`) |
| `wf-dispatch select` | m31 | one-shot composite scoring without dispatch |
| `wf-dispatch verify <id>` | m33 (manual re-verify) | refreshes TTL on PASS/DEGRADED |
| `wf-dispatch dispatch <id>` | m32 (5-check sequence) → m40/m41/m42 fan-out | **operator only; never auto** |

---

## Refuse-mode catalogue (when modules fire and then refuse)

| Module | Refusal trigger | Returned error / behaviour |
|---|---|---|
| m1 | `db_path` missing | `AtuinConsumerError::DatabaseOpenFailed` (no fallback unless subprocess fallback configured) |
| m13 | LTP/LTD value outside all 3 bands | silent — write deferred to JSONL queue (not an error) |
| m20 | `m14.variance > STABILIZATION_VARIANCE_THRESHOLD` (CC-3 gate) | `MinerError::StabilizationGateNotMet` |
| m23 | `Option<Lift>::None` | `ProposalError::LiftEvidenceMissing` |
| m30 | accept without `HumanAcceptanceSignature` | `BankError::AcceptanceRequiresHumanSignature` (AP-V7-07) |
| m32 | `CONDUCTOR_DISPATCH_ENABLED != "1"` | `DispatchError::ConductorDispatchDisabled` (refuse-mode type) |
| m32 | check 1 fail | `DispatchError::ConductorNotLive` |
| m32 | check 2 fail | `DispatchError::VerificationStale` |
| m32 | check 3 fail | `DispatchError::DefinitionDrifted` |
| m32 | check 4 fail | `DispatchError::Sunset` |
| m32 | check 5 fail | `DispatchError::CooldownActive` |
| m32 | self-dispatch detected | `DispatchError::SelfDispatchRefused` (AP-V7-08) |
| m33 | non-unanimous PASS for DataExfil-class workflow | `VerifyResult::Verdict::Fail` (per [m33 spec](../ai_specs/modules/cluster-G/m33_verifier.md)) |
| m42 | substrate down (stcortex unreachable) | `ReinforceOutcome::SubstrateUnavailable` (outbox carries record) |

Refusal is a **first-class concern** in this engine. Cluster G in particular treats every "no" as a typed result, not an exception. Per [`CROSS_CLUSTER_SYNERGIES.md`](../ai_docs/optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md) Watcher Class C pre-position summary: every refusal is observable as Class C (refusal is correct behaviour, not a bug).

---

## Cross-references

| Question | Answer | File |
|---|---|---|
| What is the build-time graph? | Mermaid graph TD | [`MODULE_DEPENDENCY_GRAPH.md`](MODULE_DEPENDENCY_GRAPH.md) |
| What rows travel each edge? | per-edge type table | [`DATA_FLOW.md`](DATA_FLOW.md) |
| What metadata attends each emission? | context table | [`CONTEXTUAL_FLOW.md`](CONTEXTUAL_FLOW.md) |
| What must always hold? | invariants | [`INVARIANT_MAP.md`](INVARIANT_MAP.md) |
| What does the canonical layer view look like? | View 1 Mermaid | [`../ai_docs/optimisation-v7/ULTRAMAP.md`](../ai_docs/optimisation-v7/ULTRAMAP.md) |
| What is the per-CC closure-test contract? | inventory table | [`../ai_docs/optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md`](../ai_docs/optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md) |

---

> **Back to:** [`README.md`](README.md) · [`../ARCHITECTURE.md`](../ARCHITECTURE.md) · canonical [`../ai_docs/optimisation-v7/ULTRAMAP.md`](../ai_docs/optimisation-v7/ULTRAMAP.md) · [`ULTRAMAP.md`](ULTRAMAP.md) (this folder's master synthesis)
