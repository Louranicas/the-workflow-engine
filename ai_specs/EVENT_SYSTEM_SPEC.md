---
title: EVENT_SYSTEM_SPEC — NexusEvent (m40) + pressure register (m15) + Hebbian emit (m42)
date: 2026-05-17
status: SPEC
event_taxonomy: [WorkflowEvent (m40), ReservationNotice (m15), ReinforceEvent (m42)]
---

# EVENT_SYSTEM_SPEC — workflow-trace

> **Back to:** [`INDEX.md`](INDEX.md) · [`MODULE_MATRIX.md`](MODULE_MATRIX.md) · [`../README.md`](../README.md) · [`../ARCHITECTURE.md`](../ARCHITECTURE.md) · [`WIRE_PROTOCOL_SPEC.md`](WIRE_PROTOCOL_SPEC.md)

## Event taxonomy

Three typed event families. Each owns its emit module, its serde envelope, its target substrate, and its consumer protocol.

| Family | Owner module | Emitted to | Consumer | Purpose |
|---|---|---|---|---|
| **WorkflowEvent** | m40 | SYNTHEX v2 `:8092/v3/nexus/push` | NexusEvent bus | coordination-grain observability across fleet |
| **ReservationNotice** | m15 | `~/projects/shared-context/agent-cross-talk/*.jsonl` | Watcher tick + Zen G7 audit | spec-grain pressure for CC-7 evolution |
| **ReinforceEvent** | m42 (via m13) | stcortex `:3000` `workflow_trace_*` pathways | substrate-side Hebbian update | substrate-grain CC-5 learning loop |

---

## 1. `WorkflowEvent` (m40)

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WorkflowEvent {
    Run {
        id: WorkflowId,
        outcome: RunOutcome,
        fitness_delta: f64,
        fields: serde_json::Value,
    },
    Verify {
        id: WorkflowId,
        verdict: VerifyVerdict,
        agents_passed: Vec<String>,
    },
    DispatchRefused {
        id: WorkflowId,
        check_failed: u8,        // 1..=5
        reason: String,
    },
    BankAccepted {
        id: WorkflowId,
        accepted_by: String,
        escape_surface: EscapeSurfaceProfile,
    },
    PathwayReinforced {
        pre: String,             // workflow_trace_* prefixed
        post: String,             // workflow_trace_* prefixed
        fitness_delta: f64,
    },
}
```

### Envelope

```rust
pub struct NexusEnvelope {
    pub event_id: uuid::Uuid,     // UUIDv4 per emit
    pub source: &'static str,     // "workflow-trace"
    pub emitted_at: i64,          // unix ms
    pub event: WorkflowEvent,
}
```

### Emit shape

- **Outbox-first JSONL** — `outbox/m40/{event_id}.jsonl` written + fsynced + renamed BEFORE network POST.
- **HTTP POST** to `http://localhost:8092/v3/nexus/push` with `Content-Type: application/json`.
- **Best-effort** — circuit-breaker after 2 consecutive failures; outbox carries the record.
- **Idempotency** — `event_id` deduplicates within SYNTHEX v2's idempotency window.

### Consumer protocol

SYNTHEX v2 accepts `NexusEnvelope` and routes to subscribed fleet consumers. workflow-trace is producer-only; it does not subscribe to its own emits.

---

## 2. `ReservationNotice` (m15)

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReservationNotice {
    ForbiddenVerb {
        attempted_verb: String,
        attempted_by_module: String,
        context: String,
    },
    SampleSizeRelaxation {
        from: u32,
        to: u32,
        attempted_by_module: String,
    },
    ScopeRelaxation {
        v1_invariant: String,
        proposed_relaxation: String,
    },
    HandshakeSilence {
        peer: String,
        timeout_ms: u64,
        last_ack_at: i64,
    },
    EscapeSurfaceEscalation {
        workflow_id: String,
        from: EscapeSurfaceProfile,
        to: EscapeSurfaceProfile,
    },
}
```

### Envelope (filename + content)

```rust
pub struct ReservationEnvelope {
    pub event_id: String,             // hex-digest
    pub ts_ms: i64,
    pub count: u32,                   // dedup-window collapsed count
    pub kind: ReservationNoticeKind,  // discriminant for filename
    pub notice: ReservationNotice,
}
```

**Filename:** `PHASE-B-RESERVATION-NOTICE-{ts_ms}_{event_id}.jsonl`
**Path:** `~/projects/shared-context/agent-cross-talk/`
**Format:** single-line JSONL (one event per file, no append mode).

### Emit shape

- **Atomic tmp + rename** — `.jsonl.tmp` → `.jsonl`. Watcher reads canonical name only.
- **60s de-dup window** — same `(kind, context_hash)` within 60s collapses to one file with `count: N`.
- **No fsync required** — Watcher reads on cadence, not real-time.

### Consumer protocol

- **Watcher** reads `~/projects/shared-context/agent-cross-talk/` on tick cadence per `~/.local/bin/watcher` schedule; classifies; emits WCP notices via `~/.local/bin/watcher notify`.
- **Zen** reads at G7-style audit cadence; emits AUDIT-REQUEST drops if pattern is structural.
- **Luke** receives accumulated Watcher + Zen surfaces.

See [`synergies/CC-7.md`](synergies/CC-7.md) for the 8-step pressure-driven evolution sequence.

---

## 3. `ReinforceEvent` (m42)

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReinforceEvent {
    pub request_id: uuid::Uuid,           // UUIDv4 for idempotency
    pub pre_id: String,                    // workflow_trace_<workflow_id>
    pub post_id: String,                   // workflow_trace_outcome_<outcome>
    pub fitness_delta: f64,                // clamped [-1.0, 1.0]
    pub outcome: RunOutcome,
    pub emitted_at: i64,                   // unix ms
}
```

### Emit shape

- **Outbox-first JSONL** — `outbox/m42/{request_id}.jsonl`.
- **HTTP POST via m13** — m13 is the transport; m42 hands `(namespace, payload)` to `m13::write`.
- **AP30 prefix enforced at m9** — `m9::assert_namespace(&pre_id)` called before m13 write.
- **AP-Hab-11 hyphen-slug** — pre_id/post_id convert `-` → `_`.

### Fitness constants

```rust
pub const FITNESS_PASS_VERIFIED: f64 =  0.25;
pub const FITNESS_PASS:          f64 =  0.15;
pub const FITNESS_BLOCKED:       f64 = -0.05;
pub const FITNESS_FAIL:          f64 = -0.10;
```

### Consumer protocol

stcortex-side Hebbian update; m31 reads updated pathway weights at next selection cycle (timescale: days/weeks). See [`synergies/CC-5.md`](synergies/CC-5.md).

---

## Internal event channels (cross-binary)

`wf-crystallise` and `wf-dispatch` are separate binaries. `m32` lives in `wf-dispatch`; `m40/m41/m42` live in `wf-crystallise`. Cross-binary event flow:

- **In-process invocation** (`wf-crystallise --serve` mode): m32's `DispatchOutcome` posts to an in-process `tokio::sync::mpsc` channel; `wf-crystallise` event-loop drains.
- **CLI invocation** (`wf-dispatch dispatch <id>`): m32 writes `DispatchOutcome` to a JSONL outbox at `~/.local/state/workflow-trace/dispatch_outbox/`; next `wf-crystallise ingest` tick drains and fans into m40/m41/m42 emit.

This means **`wf-crystallise` and `wf-dispatch` have no in-process IPC bus** — see [`IPC_BUS_SPEC.md`](IPC_BUS_SPEC.md).

## Event coalescing rules (m15 only)

m40 + m42 events are NOT coalesced — every dispatch produces one m40 + one m42 emit.

m15 events ARE coalesced — 60s window same-kind same-context coalesces. The rationale: m40/m42 are substrate-grain (lost detail = lost learning); m15 is pressure-grain (flood = signal loss).

## Verify-sync invariants

- Wire-contract tests pin every event family's serde shape at `tests/contract/event_*.rs`.
- Outbox-first invariant tested via integration: kill process mid-emit; assert outbox file exists, no `.tmp` left.
- Idempotency: same `event_id` / `request_id` re-emitted does not double-write at consumer.

---

> **Back to:** [`INDEX.md`](INDEX.md) · [`WIRE_PROTOCOL_SPEC.md`](WIRE_PROTOCOL_SPEC.md) · [`synergies/CC-5.md`](synergies/CC-5.md) · [`synergies/CC-7.md`](synergies/CC-7.md)
