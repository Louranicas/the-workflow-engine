---
substrate_id: S-E-synthex
kind: http
read_endpoints: ["http://127.0.0.1:8092/health", "http://127.0.0.1:8092/v3/nexus (read)"]
write_endpoints: ["http://127.0.0.1:8092/v3/nexus/push (NexusEvent push)"]
lifecycle_phases: [cold-start, warming, steady-state, degraded, refusing, dead]
refusal_modes: [breaker_open, schema_rejected, r13_quiet_period, ws_disconnect, http_429_rate_limit]
drift_indicators: [nexus_event_envelope_change, watcher_class_redefinition, wire_protocol_version_bump, hebbian_coordinator_formula_change]
back_pressure_signals: [breaker_state, recent_rejects_per_min, ws_send_buffer_depth, watcher_observation_count]
consent_dimensions: [n/a]
status: SPEC
date: 2026-05-17
authority: Luke @ node 0.A — S1002127 "as per proposal"
hold_v2_compliant: true
addresses: [NA-GAP-01, NA-GAP-04, NA-GAP-07, NA-GAP-08, NA-GAP-09, NA-GAP-10, NA-GAP-11]
---

# S-E — SYNTHEX v2 (autonomic-loop substrate)

> **Back to:** [`INDEX.md`](../INDEX.md) · [`../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md) · [`../cross-cutting/persistence.md`](../cross-cutting/persistence.md) · [`../cross-cutting/refusal-taxonomy.md`](../cross-cutting/refusal-taxonomy.md) · [`../cross-cutting/substrate-drift.md`](../cross-cutting/substrate-drift.md) · sister substrates [`atuin.md`](atuin.md) · [`stcortex.md`](stcortex.md) · [`injection_db.md`](injection_db.md) · [`lcm.md`](lcm.md) · [`conductor.md`](conductor.md) · [`watcher.md`](watcher.md) · [`operator.md`](operator.md)
>
> Engine consumer: **m40** ([`../modules/cluster-H/m40_synthex_emit.md`](../modules/cluster-H/m40_synthex_emit.md))

## 1. Purpose & boundary

SYNTHEX v2 is the habitat's **autonomic-loop substrate** — a Rust daemon at `:8092` that hosts the Watcher (m46-m51), the 7-layer ws_handler, regulation_cascade, the Hebbian coordinator (since S226), and the NexusEvent bus at `/v3/nexus/push`. SYNTHEX v1 (`:8090`) is RETIRED (S113); only v2 exists.

**IN scope:** push typed `WorkflowEvent` payloads to `/v3/nexus/push` (outbox-first JSONL durability per m40); observe Watcher class emissions; respect R13 cold-start quiet period.

**OUT of scope:** running SYNTHEX itself; modifying its config; pushing to `:8090` (retired); coupling tighter than NexusEvent (no shared state).

## 2. Lifecycle phases

| Phase | Indicator | Engine action |
|---|---|---|
| cold-start | R13 quiet period active (eligibility=false; observation count < 100) | m40 defers all pushes; outbox-only |
| warming | R13 elapsed; first watcher observations flowing | normal pushes after sanity probe |
| steady-state | breaker CLOSED; rate-limit headroom > 80% | normal cadence |
| degraded | breaker HALF_OPEN; or 429 rate appearing | m40 reduces push rate; respects 429 Retry-After |
| refusing | breaker OPEN after 2 consecutive failures; or schema rejected | typed `EmitError::Breaker` or `EmitError::SchemaRejected`; outbox accumulates |
| dead | `:8092/health` unreachable for > 5 min | m40 surfaces `SubstrateUnavailable`; outbox replay deferred until recovery |

## 3. Refusal modes (substrate-authored)

- **HTTP 422 `SchemaRejected`** — substrate validated NexusEvent envelope and rejected unknown field. Substrate-authored. Recovery: pin envelope version; emit drift event.
- **HTTP 429 rate-limit** — substrate signals back-pressure. Substrate-authored. Recovery: respect `Retry-After`; do not retry tight loop.
- **R13 quiet-period refusal** — substrate refuses Hebbian coordinator interactions until R13 elapses (observation count ≥ 100 OR 30-day window). Substrate-authored. Recovery: defer until quiet period ends.
- **HTTP 401 / consumer revoked** — substrate revokes engine's push token (defensive). Recovery: surface to operator; re-issue.

Plus engine-side breaker-OPEN, which is NOT a substrate refusal (`Unavailable { backoff_recommendation }` per [`../cross-cutting/refusal-taxonomy.md`](../cross-cutting/refusal-taxonomy.md)).

## 4. Drift indicators (closes NA-GAP-07)

- **NexusEvent envelope change** — substrate adds/renames/typenarrows a field. Detector: envelope-schema-hash check on m40's startup probe.
- **Watcher class redefinition** — substrate redefines Class-I from "Hebbian silence" to "Hebbian saturation". Detector: subscribe to Watcher class taxonomy table; compare against frozen baseline.
- **Wire-protocol version bump** — `/v3/nexus/push` becomes `/v4/...`. Detector: 404 on push; trigger re-handshake.
- **Hebbian coordinator formula change** — analogous to CR-2 POVM case (see [`../cross-cutting/substrate-drift.md`](../cross-cutting/substrate-drift.md)). Detector: Watcher Class-I delta vs engine's local fitness-delta sum.

## 5. Back-pressure signals (closes NA-GAP-04)

- **Breaker state** — m40_42_common breaker tracks consecutive failures; OPEN after 2.
- **Recent rejects per minute** — engine-side counter; substrate-side rate-limit observable indirectly.
- **WS send-buffer depth** — for subscribers, indicates substrate consumer lag.
- **Watcher observation count** — proxy for substrate cognitive load (high observation rate → engine should reduce non-essential pushes).

## 6. Receipts (closes NA-GAP-09)

SYNTHEX v2 returns a `nexus_event_id` on accepted push; this is the substrate-side receipt. m40 records the receipt in outbox JSONL row as `accepted_at + nexus_event_id`. The receipt closes the local emit-loop but does NOT confirm downstream Hebbian coordinator action — for that, Watcher Class-I rolling-window observation is required (per [`../synergies/CC-5.md`](../synergies/CC-5.md)).

Per NA-GAP-11, refusal events MUST also be substrate-readable: when m32 refuses, m40 emits `WireEvent::Refusal { token: ... }` to `/v3/nexus/push` so the substrate observes refusal first-class instead of Watcher inferring from absence (see [`../cross-cutting/refusal-taxonomy.md`](../cross-cutting/refusal-taxonomy.md)).

## 7. Capabilities & namespaces (closes NA-GAP-10)

SYNTHEX v2's capability model:
- Accepts pushes from any `consumer_id` with valid bearer (currently flat trust; v0.2.0 reputation TBD).
- Filters by `event.namespace`; only events with allowed namespace prefix are routed to subscribers.
- `workflow-trace` is authorised for `workflow_trace_*` namespace per AP30.

NA-GAP-10's "substrate-mediated trust" would extend this with per-consumer reputation surfaced by Watcher fitness-delta accuracy.

## 8. Substrate-internal couplings (closes NA-GAP-03)

SYNTHEX v2's substrate-internal edges:
- **SYNTHEX v2 → stcortex (S-C) via Hebbian coordinator** — since S226, SYNTHEX v2 reads NexusEvents, computes Hebbian deltas, writes to stcortex. This is a substrate-substrate edge **the engine triggers but does not control**.
- **SYNTHEX v2 → Watcher classes (S-F watcher)** — Watcher is housed inside SYNTHEX v2; class emissions are substrate-internal.
- **SYNTHEX v2 → operator (S-G)** — via watcher-notices file drops.
- **SYNTHEX v2 ↔ ORAC (out-of-scope)** — ORAC consumes Watcher class output; not directly engine-coupled.

The **hidden CC-5 edge** NA-GAP-03 surfaces: m40 → SYNTHEX v2 → stcortex pathway delta. Engine does not see the middle hop; it only observes substrate-side delta via Watcher.

## 9. Test-fixture sketch (closes NA-GAP-08)

Fixtures at `tests/substrate_fixtures/synthex/`:

- **`breaker_open_after_2_failures_fixture`** — emulator returns 503 twice; asserts breaker opens.
- **`schema_rejected_fixture`** — emulator returns 422 with new-field rejection; asserts SchemaRejected surfaces typed.
- **`r13_quiet_period_fixture`** — emulator returns "R13 active"; asserts m40 defers and outboxes.
- **`429_rate_limit_fixture`** — emulator returns 429 with Retry-After: 60; asserts m40 respects.
- **`envelope_schema_drift_fixture`** — emulator's envelope-hash differs from baseline; asserts substrate-drift canary fires.

## 10. Watcher class pre-positions

Watcher is *housed in* this substrate, so meta-observability is recursive:
- **Class B (boundary)** — every m40 push attempt
- **Class D (drift)** — envelope-hash change; class taxonomy change
- **Class I (Hebbian silence)** — downstream stcortex delta zero
- **Class C (refusal)** — substrate-authored 422/429
- **Class A (activation)** — first successful push post R13

---

> **Back to:** [`INDEX.md`](../INDEX.md) · [`../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md)

*Filed 2026-05-17 (S1002127 · Wave 4 NA-remediation) · Luke "as per proposal" · planning-only · HOLD-v2 compliant.*
