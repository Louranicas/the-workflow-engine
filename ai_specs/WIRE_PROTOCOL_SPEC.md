---
title: WIRE_PROTOCOL_SPEC — m42 stcortex pathway envelope + m41 LCM RPC + m40 NexusEvent push
date: 2026-05-17
status: SPEC
wires: [m42 stcortex pathway (POVM-DECOUPLED), m41 lcm.loop.create, m40 /v3/nexus/push]
---

# WIRE_PROTOCOL_SPEC — workflow-trace

> **Back to:** [`INDEX.md`](INDEX.md) · [`MODULE_MATRIX.md`](MODULE_MATRIX.md) · [`../README.md`](../README.md) · [`../ARCHITECTURE.md`](../ARCHITECTURE.md) · [`EVENT_SYSTEM_SPEC.md`](EVENT_SYSTEM_SPEC.md)

## Wire inventory

Three external wire contracts. Each is pinned at `tests/contract/{module}_wire.rs` and is contract-binding (drift fails CI).

| Wire | Module | Target | Protocol | POVM dep? |
|---|---|---|---|---|
| m42 stcortex pathway | m42 (via m13) | stcortex `:3000` | HTTP POST JSON | **NO — POVM-DECOUPLED per 2026-05-17 ADR** |
| m41 LCM `lcm.loop.create` | m41 | LCM MCP server | MCP stdio JSON-RPC 2.0 | NO |
| m40 NexusEvent push | m40 | SYNTHEX v2 `:8092/v3/nexus/push` | HTTP POST JSON | NO |

---

## 1. m42 stcortex pathway envelope (POVM-DECOUPLED)

**The POVM decoupling is the load-bearing fact.** Per [`../ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md`](../ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md): m42 routes substrate-feedback to stcortex exclusively from M0. **No POVM read. No POVM write. No fallback.**

### HTTP request

```http
POST http://127.0.0.1:3000/v1/pathway
Content-Type: application/json
X-Request-Id: <UUIDv4>
X-Idempotency-Key: <UUIDv4>          # same as X-Request-Id for dedup

{
  "pre_id":        "workflow_trace_wf_abc_123",       // hyphen→underscore slug
  "post_id":       "workflow_trace_outcome_pass_verified",
  "fitness_delta": 0.25,                                // clamped [-1.0, 1.0]
  "emitted_at_ms": 1747497600000,
  "metadata": {
    "outcome":     "pass_verified",
    "workflow_id": "wf-abc-123",                        // original (un-munged) for trace
    "source":      "workflow-trace"
  }
}
```

### HTTP response (success)

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "accepted": true,
  "pathway_id_canonical": "workflow_trace_wf_abc_123__workflow_trace_outcome_pass_verified",
  "weight_before": 0.4123,
  "weight_after":  0.4248
}
```

**`weight_before`/`weight_after` are advisory only.** Per AP-V7-13: HTTP 200 ≠ behaviour-verified. m42 does NOT take `weight_after > weight_before` as proof of learning; verification belongs to the Class-I Watcher 7-day rolling-window monitor.

### HTTP response (substrate unavailable)

```http
HTTP/1.1 503 Service Unavailable
```

→ m42 returns `ReinforceOutcome::SubstrateUnavailable`; outbox carries the record for offline-snapshot replay. **NO silent POVM fall-through.**

### Slug-encoding rules (AP-Hab-11)

| Raw | Encoded |
|---|---|
| `wf-abc-123` | `wf_abc_123` |
| `cluster-id-xyz` | `cluster_id_xyz` |
| `pass-verified` | `pass_verified` (variant name) |

Encoded at the slug boundary by `workflow_core::namespace::encode_slug(raw: &str) -> String` — single helper, single test point.

### Idempotency

`X-Request-Id` = `X-Idempotency-Key` = UUIDv4. stcortex de-duplicates within its idempotency window (default 1h, matching m24_povm_bridge.rs pattern lift from boilerplate). Re-emit with same UUID is a no-op at substrate.

### Timeout

5,000 ms via `tokio::time::timeout`. Exceeding → `ReinforceError::Timeout { elapsed_ms: 5000 }`.

---

## 2. m41 LCM `lcm.loop.create` RPC payload

**The correct call is `lcm.loop.create`, NOT `lcm.deploy`.** This is a frequently-confused API; the contract test pins the method name.

### MCP stdio JSON-RPC 2.0 request

```json
{
  "jsonrpc": "2.0",
  "id": "<UUIDv4>",
  "method": "lcm.loop.create",
  "params": {
    "max_iters": 1,
    "loop_kind": "workflow_trace_deploy",
    "context": {
      "workflow_id": "wf-abc-123",
      "outcome":     "pass_verified",
      "fitness_delta": 0.25,
      "steps_hash":  "0xabc...def",
      "source":      "workflow-trace"
    }
  }
}
```

### Response (success)

```json
{
  "jsonrpc": "2.0",
  "id": "<UUIDv4>",
  "result": {
    "loop_id": "<lcm-assigned-id>",
    "status":  "created",
    "max_iters": 1
  }
}
```

### Response (error — non-deploy shape rejected)

m41 itself is the gate — non-deploy-shaped DispatchOutcomes return `Ok(LcmRouteResult::NotRouted { reason: "non_deploy_shape" })` and never call LCM. LCM never sees a non-deploy event.

### Timeout

10,000 ms (LCM-side latency higher than stcortex). Exceeding → `LcmRpcError::Timeout`.

### Anti-pattern banned

**`lcm.deploy`** — this is the V3-side API. m41 calls `lcm.loop.create` exclusively. Contract test:

```rust
#[test]
fn m41_emits_lcm_loop_create_not_deploy() {
    let req = m41::build_request(test_dispatch_outcome());
    assert_eq!(req.method, "lcm.loop.create");
    assert_ne!(req.method, "lcm.deploy");
}
```

---

## 3. m40 NexusEvent push schema

### HTTP request

```http
POST http://localhost:8092/v3/nexus/push
Content-Type: application/json

{
  "event_id":    "<UUIDv4>",
  "source":      "workflow-trace",
  "emitted_at":  1747497600000,
  "event": {
    "kind": "run",
    "id":            "wf-abc-123",
    "outcome":       "pass_verified",
    "fitness_delta": 0.25,
    "fields": {
      "cluster_id":  "a1b2c3d4e5f60718",
      "session_id":  "sess-xyz"
    }
  }
}
```

### Response (success)

```http
HTTP/1.1 200 OK
Content-Type: application/json

{ "accepted": true, "event_id": "<UUIDv4>" }
```

### Event kind discriminator

The `event.kind` field is the discriminator; SYNTHEX v2's consumer dispatcher routes by kind. Supported kinds per [`EVENT_SYSTEM_SPEC.md`](EVENT_SYSTEM_SPEC.md) § WorkflowEvent: `run`, `verify`, `dispatch_refused`, `bank_accepted`, `pathway_reinforced`.

### Timeout

2,000 ms (best-effort; outbox covers failure).

### Idempotency

`event_id` UUIDv4 deduplicates within SYNTHEX v2's idempotency window. Re-emit is a no-op.

---

## Wire-contract test discipline

Every wire is pinned with `insta::assert_yaml_snapshot!` against a known-good payload:

```rust
// tests/contract/m42_stcortex_wire.rs
#[test]
fn m42_pathway_envelope_matches_snapshot() {
    let env = m42::build_envelope(test_outcome());
    insta::assert_yaml_snapshot!(env);
}
```

Snapshots stored under `tests/contract/snapshots/`. Reviewed + approved at v1.3-G7 spec audit. Drift fails CI.

## Wire-port discipline

Wire ports are constants:

```rust
pub const STCORTEX_HTTP_BASE: &str = "http://127.0.0.1:3000";
pub const SYNTHEX_V2_NEXUS_PUSH: &str = "http://localhost:8092/v3/nexus/push";
// LCM is MCP stdio — no port
```

Per Genesis v1.3 § 1 + Town Hall P0 #3: **Conductor-only routing** for m32; m32 dispatches via `:8141`; this is the dispatch wire (not a substrate-feedback wire) and is covered in [`synergies/CC-4.md`](synergies/CC-4.md) and m32's per-module spec.

## Anti-patterns (wire-specific)

- **`http://` prefix in port-only configs** — bridge URLs lift raw `SocketAddr`; the `http://` is computed at request time (BUG-033).
- **Untyped `serde_json::Value` in wire types** — every event/payload uses typed serde structs.
- **Wire-port magic numbers** — every wire port is a named constant.
- **`X-Idempotency-Key` reuse across distinct events** — UUIDv4 per emit.

---

> **Back to:** [`INDEX.md`](INDEX.md) · [`EVENT_SYSTEM_SPEC.md`](EVENT_SYSTEM_SPEC.md) · [`synergies/CC-5.md`](synergies/CC-5.md) · [`../ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md`](../ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md)
