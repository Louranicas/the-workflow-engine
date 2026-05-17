---
title: cross-cutting/concurrency — tokio runtime, spawn discipline, AP29
date: 2026-05-17
status: SPEC
axes: [runtime, spawn, channels, cancellation, timeouts, AP29]
---

# Concurrency — Module-Side Guidance

> **Back to:** [`../INDEX.md`](../INDEX.md) · [`../../README.md`](../../README.md) · [`../../ARCHITECTURE.md`](../../ARCHITECTURE.md) · [`../../ai_docs/optimisation-v7/STANDARDS/GOD_TIER_RUST.md`](../../ai_docs/optimisation-v7/STANDARDS/GOD_TIER_RUST.md) § Async discipline

## Tokio runtime — single choice

- **Tokio multi-thread runtime** exclusively (no async-std, no smol).
- **Binary entry points** declare runtime explicitly:

```rust
#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> anyhow::Result<()> { ... }
```

- **Library code** is runtime-agnostic — never calls `tokio::runtime::Builder` inside lib modules; accepts the caller's runtime.

## `tokio::spawn` discipline

```rust
// GOOD — JoinHandle captured + awaited
let handle = tokio::spawn(async move { process(payload).await });
handle.await??;

// BAD — JoinHandle silently dropped (god-tier rule #5)
tokio::spawn(async move { process(payload).await });  // BANNED by review
```

If fire-and-forget IS the intent (Cluster H emit fan-out), the pattern is:

```rust
// GOOD — fire-and-forget with explicit rationale + tracing span
// rationale: m40 emit is fire-and-forget by CC-5 design; outbox covers durability
let span = tracing::Span::current();
tokio::spawn(async move {
    let _enter = span.enter();
    if let Err(e) = m40_emit(event).await {
        tracing::warn!(error = %e, "m40 emit failed; outbox carries record");
    }
});
```

## AP29 — sync HTTP inside `tokio::spawn`

**Catalogued S225:** synchronous HTTP libraries (`ureq`, blocking `reqwest`) inside `tokio::spawn` starve the runtime because each blocking call holds a worker thread for the full RTT. The fleet-wide hot-path under load collapses.

**Rule:** any synchronous IO operation > 100 µs inside a tokio context MUST be wrapped in `tokio::task::spawn_blocking`:

```rust
// BAD — AP29 violation
tokio::spawn(async move {
    let resp = ureq::get(url).call();  // BLOCKS WORKER THREAD
    process(resp).await;
});

// GOOD — spawn_blocking for the sync IO
tokio::spawn(async move {
    let resp = tokio::task::spawn_blocking(move || ureq::get(url).call()).await?;
    process(resp).await;
});
```

In workflow-trace, the relevant modules are **m13** (stcortex HTTP write) and **m40/m41/m42** (Cluster H emit). All four use async HTTP (`reqwest` async) and avoid AP29 by construction. Any future module that needs sync HTTP — wrap in `spawn_blocking`.

## Channel discipline

| Channel | Use case |
|---|---|
| `tokio::sync::mpsc` | backpressure-needed; producers wait when consumer slow |
| `tokio::sync::broadcast` | fanout to multiple consumers; lossy on slow consumer |
| `tokio::sync::oneshot` | request/response single-fire |
| `flume` | bridges sync + async producers (rare in workflow-trace) |

Default to `mpsc` with bounded capacity; unbounded is a memory-leak vector.

## Cancellation — `CancellationToken`

- **Propagate cancellation downward** — every long-running spawn accepts a `CancellationToken` clone.
- **Never `task.abort()` abruptly** — `abort` leaves resources mid-mutation; use cooperative cancellation via the token's `cancelled()` future.
- **SIGINT/SIGTERM** at bin entry point translates to root token cancellation:

```rust
let root_token = CancellationToken::new();
let signal_token = root_token.clone();
tokio::spawn(async move {
    tokio::signal::ctrl_c().await.ok();
    signal_token.cancel();
});

// downstream spawns inherit clones of root_token
```

## Timeout discipline

Every `await` on external IO has `tokio::time::timeout`:

```rust
let resp = tokio::time::timeout(Duration::from_secs(5), client.post(url).send())
    .await
    .map_err(|_| BridgeError::Timeout { peer: "stcortex", elapsed_ms: 5000 })??;
```

- **m13** stcortex write: 5s timeout
- **m40** SYNTHEX push: 2s timeout (best-effort; outbox covers failure)
- **m41** LCM RPC: 10s timeout (LCM-side latency is higher)
- **m42** stcortex reinforce (via m13): 5s timeout
- **m32** Conductor `:8141`: 30s timeout

## Lock scoping

```rust
// GOOD — drop guard before acquiring next lock
{
    let guard = self.bank_lock.read();
    do_thing(&*guard);
} // guard dropped here
let other_guard = self.other_lock.write();

// BAD — lock-acquisition-while-holding-lock (deadlock vector)
let bank_guard = self.bank_lock.read();
let other_guard = self.other_lock.write();  // RISKY
```

Per ORAC pattern: **acquire `AppState` before `BusState`** if both needed (lock-order discipline).

## Verify-sync invariants

- **#5** — `tokio::spawn(.*); *$` (silent JoinHandle drop) returns 0 via grep audit.
- **#14** — every external IO has `tokio::time::timeout`; rg density check per module.

---

> **Back to:** [`../INDEX.md`](../INDEX.md) · [`../../ai_docs/optimisation-v7/STANDARDS/GOD_TIER_RUST.md`](../../ai_docs/optimisation-v7/STANDARDS/GOD_TIER_RUST.md)
