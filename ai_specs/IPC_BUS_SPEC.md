---
title: IPC_BUS_SPEC — minimal (CLI handoff via JSONL outbox; no in-process bus between binaries)
date: 2026-05-17
status: SPEC
---

# IPC_BUS_SPEC — workflow-trace

> **Back to:** [`INDEX.md`](INDEX.md) · [`../README.md`](../README.md) · [`../ARCHITECTURE.md`](../ARCHITECTURE.md) · [`EVENT_SYSTEM_SPEC.md`](EVENT_SYSTEM_SPEC.md)

## TL;DR — there is no in-process IPC bus

workflow-trace has **two binaries** — `wf-crystallise` and `wf-dispatch`. They do NOT share memory; they do NOT have an in-process IPC bus; they do NOT use shared-memory channels, named pipes, or Unix domain sockets between binaries. The architectural decision is documented here so future contributors do not invent one.

## Why no IPC bus

1. **CLI-first deployment.** Genesis v1.3 § 6 Axis 3 locks CLI as the default invocation surface; `feature = "serve"` for a live HTTP `wf-status` endpoint is opt-in (default off, post-D60). The binaries are intended to be invoked sequentially — `wf-crystallise ingest` then `wf-dispatch select` then `wf-dispatch dispatch` — not run concurrently as a daemon pair.
2. **Outbox is the durability primitive.** A JSONL outbox at `~/.local/state/workflow-trace/dispatch_outbox/` (m32 → Cluster H emit) and the per-module outboxes at `outbox/m{40,41,42}/` already provide cross-binary handoff with the better property: surviving process restart. An IPC bus adds complexity without adding durability.
3. **Failure-mode reduction.** Two binaries with a shared bus introduce shared-state failure modes (bus full, bus deadlock, consumer slow → producer backpressure). Two binaries with disk-backed handoff have one failure mode: the handoff file. Disk failure surfaces the same way regardless.
4. **ORAC + ME v2 precedent.** The gold-standard services in the habitat use HTTP bridges + circuit breakers for cross-service communication, not in-process buses. workflow-trace follows the same pattern.

## CLI handoff via JSONL outbox

### m32 → Cluster H emit (cross-binary)

`m32` lives in `wf-dispatch`. When `wf-dispatch dispatch <id>` completes:

1. m32 writes `DispatchOutcome` JSONL to `~/.local/state/workflow-trace/dispatch_outbox/{outcome_id}.jsonl` (atomic tmp+rename + fsync).
2. The next `wf-crystallise ingest` invocation includes a `drain_dispatch_outbox()` step that:
   - Reads all `.jsonl` files in `dispatch_outbox/`
   - For each: fans into m40 (`outbox/m40/`) + m41 (deploy-shaped only, `outbox/m41/`) + m42 (`outbox/m42/`) emit
   - On successful fan-out: moves consumed file to `dispatch_outbox/processed/{outcome_id}.jsonl` (or deletes per config).

### `wf-crystallise --serve` (single-process mode, future)

In a hypothetical future `wf-crystallise --serve` mode that runs as a daemon and ALSO links m32 (via feature flag, NOT the default), the handoff would be in-process via `tokio::sync::mpsc::channel(capacity=1024)`. Even in that mode, the outbox would still be written — it is the durability primitive, not just a transport.

## In-process channels (within a single binary)

Inside `wf-crystallise`, async tasks may use `tokio::sync::mpsc` for backpressure or `tokio::sync::broadcast` for fanout. This is intra-binary, not cross-binary.

| Channel kind | Used by | Purpose |
|---|---|---|
| `tokio::sync::mpsc` | m1 page-iter → m4/m5/m6 observation pipeline | backpressure across the ingest pipeline |
| `tokio::sync::oneshot` | individual RPC request/response in m13/m40/m41 | per-call response |
| `tokio::sync::broadcast` | (none currently allocated) | reserved |

## Anti-patterns (banned)

- **Inventing a shared-memory IPC bus between wf-crystallise and wf-dispatch.** If a future contributor proposes this, the answer is: "Use the outbox; or merge the binaries; do not add a third primitive."
- **Named pipes / FIFOs for cross-binary communication.** Disk-backed JSONL is simpler, more debuggable, and durable across restarts.
- **DBus / dbus-rs.** No habitat service uses DBus; introducing it for workflow-trace alone is architectural drift.
- **gRPC between binaries.** Same answer — outbox is sufficient and matches habitat conventions.

## What about Conductor `:8141`?

Conductor is the **external** dispatch service. m32 makes an HTTP call to Conductor — this is a wire-protocol concern covered in [`WIRE_PROTOCOL_SPEC.md`](WIRE_PROTOCOL_SPEC.md) (not in this spec) and the synergy contract [`synergies/CC-4.md`](synergies/CC-4.md). It is not an internal IPC bus.

## Migration path if IPC bus ever becomes necessary

If a future requirement forces real-time cross-binary coordination (e.g., live `wf-status` showing m32 dispatch in-flight from m23 emit instant), the migration is:

1. Merge `wf-dispatch` into `wf-crystallise` under a single binary with `--mode=dispatch` flag — preserves CLI ergonomics, removes the cross-binary need.
2. Use in-process `tokio::sync::mpsc` for the m23 → m32 → m40 chain.
3. The outbox stays — durability primitive remains.

This is documented as a future option, NOT a current commitment.

---

> **Back to:** [`INDEX.md`](INDEX.md) · [`EVENT_SYSTEM_SPEC.md`](EVENT_SYSTEM_SPEC.md) · [`WIRE_PROTOCOL_SPEC.md`](WIRE_PROTOCOL_SPEC.md)
