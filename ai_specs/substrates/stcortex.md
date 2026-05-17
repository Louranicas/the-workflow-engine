---
substrate_id: S-C-stcortex
kind: spacetimedb
read_endpoints: ["127.0.0.1:3000 (SpacetimeDB module HTTP/WS)", "~/claude-code-workspace/stcortex/data/snapshots/latest.json (offline fallback)"]
write_endpoints: ["127.0.0.1:3000 reducer calls (consumer-gated)"]
lifecycle_phases: [cold-start, warming, steady-state, degraded, refusing, dead]
refusal_modes: [refuse_write_no_consumer, hyphen_slug_rejected, namespace_drift, schema_migration, reducer_panic, ws_disconnect]
drift_indicators: [learning_health_formula_change, pathway_weight_semantic_shift, reducer_signature_change, namespace_relocation, snapshot_export_format_change]
back_pressure_signals: [ws_send_buffer_depth, reducer_queue_depth, consumer_token_revoked, snapshot_staleness_seconds]
consent_dimensions: [n/a — substrate is not an operator]
status: SPEC
date: 2026-05-17
authority: Luke @ node 0.A — S1002127 "as per proposal"
hold_v2_compliant: true
addresses: [NA-GAP-01, NA-GAP-02, NA-GAP-03, NA-GAP-04, NA-GAP-06, NA-GAP-07, NA-GAP-08, NA-GAP-09, NA-GAP-10]
---

# S-C — stcortex (SpacetimeDB pathway substrate)

> **Back to:** [`INDEX.md`](../INDEX.md) · [`../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md) · [`../cross-cutting/persistence.md`](../cross-cutting/persistence.md) · [`../cross-cutting/refusal-taxonomy.md`](../cross-cutting/refusal-taxonomy.md) · [`../cross-cutting/substrate-drift.md`](../cross-cutting/substrate-drift.md) · [`../synergies/CC-5.md`](../synergies/CC-5.md) · sister substrates [`atuin.md`](atuin.md) · [`injection_db.md`](injection_db.md) · [`synthex.md`](synthex.md) · [`lcm.md`](lcm.md) · [`conductor.md`](conductor.md) · [`watcher.md`](watcher.md) · [`operator.md`](operator.md)
>
> Engine consumers: **m2** (read), **m13** (write writer), **m42** (Hebbian reinforce via m13)

## 1. Purpose & boundary

stcortex is the habitat's **pathway-of-record substrate** — the SpacetimeDB module at `:3000` that holds Hebbian pathway weights, memory rows, and consumer registration tables. It is the canonical successor to POVM v2 (overlap window → 2026-07-10). For `workflow-trace`, stcortex is the **only** write target for substrate-feedback per the m42 stcortex-only pivot ADR (2026-05-17); POVM is decoupled.

**IN scope:** consumer-gated reducer calls (`write_memory`, `write_pathway`, `register_consumer`); WebSocket subscription for live pathway updates; HTTP read of frozen snapshots; offline JSONL fallback at `data/snapshots/latest.json` on substrate down.

**OUT of scope:** modifying SpacetimeDB schema; running migrations; restarting the daemon; touching POVM (decoupled).

## 2. Lifecycle phases

| Phase | Indicator | Engine action |
|---|---|---|
| cold-start | daemon recently up; consumer table possibly empty | `register_consumer` first; defer reads/writes until ACK |
| warming | consumer registered; first pathway weights loading | reads OK; writes throttled to avoid reducer queue spikes |
| steady-state | live WS subscription stable; pathway weights flowing | normal read+write cadence |
| degraded | reducer queue depth > 100; WS reconnects > 1/min | m42 backs off writes; falls to outbox-only mode |
| refusing | reducer returns `RefuseWrite::NoConsumer` or `InvalidSlug` | typed `ReinforceOutcome::Refused { class }`; surfaces to operator |
| dead | `:3000` unreachable; daemon down | m42 reads `data/snapshots/latest.json` (read-only); writes SKIPPED (never POVM fall-through) |

## 3. Refusal modes (substrate-authored)

stcortex is the canonical **substrate-authored-refusal** substrate. Distinct from `Unavailable`:

- **`RefuseWrite::NoConsumer`** — engine wrote without first calling `register_consumer` in the current session. Substrate-side gate. Recovery: call `register_consumer` and retry.
- **`RefuseWrite::InvalidSlug`** — slug contains `-` (hyphen). Substrate-side reducer error. Recovery: encode hyphens as underscores at the slug boundary (AP-Hab-11; S1001757).
- **`RefuseWrite::NamespacePolicy`** — slug prefix not in the substrate's allowed-namespace list. Recovery: surface to operator; do not auto-rename.
- **`RefuseRead::ConsumerTokenExpired`** — substrate has revoked the engine's consumer token. Recovery: re-register; surface to operator if persistent.
- **`InvalidArgument::SchemaMismatch`** — reducer expected `{weight: f64, ts: i64}` but received `{w: f64, ts: i64}`. Substrate-authored (substrate upgraded). Recovery: pin substrate version; emit drift event.

These map to `RefusalToken::SubstrateAuthored { substrate_id: "stcortex", class, repair_hint }` per [`../cross-cutting/refusal-taxonomy.md`](../cross-cutting/refusal-taxonomy.md). Distinct from `Unavailable { backoff_recommendation }` which is `:3000` connection refused.

## 4. Drift indicators (closes NA-GAP-07; **canonical case**)

stcortex is the **canonical case** for substrate-drift because of the CR-2 POVM incident: POVM `learning_health` returned 0.9146 pre-fix and 0.067 post-fix on the same input data — a 13.6× inflation factor caused by switching from binary (any-write counted) to magnitude-weighted (weight > 0.5 threshold) aggregation. POVM `:8125` was returning HTTP 200 on `/health` throughout. This is the seed of AP-V7-13 (Health-200 ≠ behaviour-verified). See [`../cross-cutting/substrate-drift.md`](../cross-cutting/substrate-drift.md) for the full treatment.

Drift indicators specific to stcortex:

- **`learning_health` formula change** — substrate upgrades the aggregation algorithm without bumping schema version. Detector: substrate-drift canary that re-derives `learning_health` from raw pathway rows and compares against substrate's reported value.
- **Pathway weight semantic shift** — substrate changes the meaning of `weight` (e.g. from `[0,1]` to `[-1,1]`). Detector: substrate-drift canary checks min/max bounds across a frozen baseline pathway set.
- **Reducer signature change** — substrate upgrades `write_pathway` argument list. Detector: schema-hash comparison against frozen baseline at session-start.
- **Namespace relocation** — substrate moves `workflow_trace_*` to `wf_trace_*`. Detector: namespace-presence canary on read at session-start.
- **Snapshot export format change** — `data/snapshots/latest.json` schema evolves; offline fallback breaks silently. Detector: snapshot-schema-hash check on every fallback read.

## 5. Back-pressure signals (closes NA-GAP-04)

- **WS send-buffer depth** — engine-side observable; substrate-side congestion.
- **Reducer queue depth** — substrate's own observable (exposed via `:3000/metrics` if enabled); engine should self-throttle writes if depth > 50.
- **Consumer token nearing revocation** — substrate may surface a `consumer.ttl_remaining` field; engine should re-register before TTL hits zero.
- **Snapshot staleness seconds** — for offline-fallback reads, the timestamp on `latest.json` must be checked; > 24h is yellow flag, > 7d is red.

## 6. Receipts (closes NA-GAP-09)

stcortex CAN emit substrate-confirmable receipts. Per NA-GAP-09 recommendation, the engine should:
- Subscribe via WS to `workflow_trace_*` pathway updates.
- When m42 writes via reducer, the WS subscription receives a `PathwayUpdated` event with the same pathway slug — this is the substrate-confirmable receipt.
- Compare the receipt event's `weight_delta` against m42's expected `fitness_delta`. If they diverge, drift event fires.
- Optionally request substrate-side `cc5_closed_at` write per NA-GAP-09 amendment to CC-5 (see [`../synergies/CC-5.md`](../synergies/CC-5.md) — pending Wave 4 amendment).

## 7. Capabilities & namespaces (closes NA-GAP-10)

stcortex authorises writes per substrate-side **consumer-trust contract** (refuse-write at DB layer):
- A consumer must call `register_consumer { id, namespace_glob }` before any write.
- The substrate maintains a consumer table; unregistered writes are refused.
- Namespace globs are enforced substrate-side: `workflow_trace_*` is the engine's authorised glob (per AP30).
- **This is the only substrate in the habitat that gates writes per-consumer.** Trust is substrate-mediated, not engine-internal — the engine cannot bypass this gate even with elevated privileges.

NA-GAP-10's "trust as substrate-mediated reputation" v0.2.0 ADR would build on this primitive: weight per-consumer reputation by recent fitness-delta accuracy; revoke or down-rate misbehaving consumers at the substrate.

## 8. Substrate-internal couplings (closes NA-GAP-03)

stcortex's substrate-internal edges that affect `workflow-trace`:

- **stcortex → injection.db (S-B)** — when stcortex pathway weights cross a threshold, the `habitat-inject` SessionStart hook may translate pathway hits into causal-chain reinforcements that update `injection.db.causal_chain.reinforcement_count`. This is the **hidden CC-5 edge** that NA-GAP-03 surfaces — m42 → S-C → habitat-inject hook → S-B.
- **stcortex → operator (S-G)** — substrate emits weekly digest reports that surface to Luke via watcher-notices; this shapes operator's prior beliefs over time.
- **stcortex → Watcher (S-F watcher)** — Watcher classes I/D consume stcortex's substrate-side observability (pathway-weight delta over rolling window).

These edges are NOT internal to the engine; they are substrate-side dynamics the engine triggers but does not control.

## 9. Test-fixture sketch (closes NA-GAP-08)

Fixtures at `tests/substrate_fixtures/stcortex/` (post-G9):

- **`cr2_inflation_fixture`** — stcortex emulator returns pre-CR-2 magnitude-weighted formula on `learning_health`. Asserts substrate-drift canary fires.
- **`refuse_write_no_consumer_fixture`** — emulator returns `RefuseWrite::NoConsumer` on `write_pathway`. Asserts m42 surfaces `Refused`, not `Unavailable`.
- **`hyphen_slug_reducer_fixture`** — emulator returns `RefuseWrite::InvalidSlug`. Asserts m13 slug-encoder converts hyphens before retry.
- **`namespace_drift_fixture`** — emulator moves `workflow_trace_*` to `wf_trace_*`. Asserts namespace-presence canary catches it.
- **`snapshot_stale_fixture`** — fallback file timestamp 14 days old. Asserts fallback-staleness threshold triggers Class-D.

## 10. Watcher class pre-positions

- **Class I (Hebbian silence)** — PRIMARY for stcortex; pathway-weight delta zero over 4+ weeks
- **Class D (drift)** — substrate-drift canary fires (formula change, schema-hash mismatch)
- **Class B (boundary)** — refuse-write at write site
- **Class C (refusal)** — refuse-write as substrate-authored-correct behaviour (consumer registration missing is operator-recoverable)

---

> **Back to:** [`INDEX.md`](../INDEX.md) · [`../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md)

*Filed 2026-05-17 (S1002127 · Wave 4 NA-remediation) · Luke "as per proposal" · planning-only · HOLD-v2 compliant.*
