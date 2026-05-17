---
title: CC-4-decomposed — substrate edges in proposal → bank → dispatch → Conductor
cc_id: CC-4
parent_synergy: ../synergies/CC-4.md
date: 2026-05-17
status: SPEC
session: S1002127
addresses: [NA-GAP-03 (secondary), AP-V7-13 enrichment]
substrates_touched: [S-D HABITAT-CONDUCTOR (primary), S-G operator (m30 acceptance signature)]
edges: 3
hold_v2_compliant: true
authority: Luke @ node 0.A — S1002127 "as per proposal"
---

# CC-4 Decomposed — Substrate Edges in Proposal → Bank → Dispatch → Conductor

> **Back to:** [`INDEX.md`](INDEX.md) · parent [`../synergies/CC-4.md`](../synergies/CC-4.md) · [`../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md) · substrates [[conductor]](../substrates/conductor.md) · [[operator]](../substrates/operator.md) · [`../cross-cutting/refusal-taxonomy.md`](../cross-cutting/refusal-taxonomy.md) · [`../cross-cutting/substrate-drift.md`](../cross-cutting/substrate-drift.md)

## § 1 — What CC-4 hides

CC-4 is the **dispatch pipeline** — engine-side `m23 → m30 → m31 → m32`, then **out** to Conductor (`S-D` substrate). Most of the chain is engine-internal SQLite (`bank.db`), but the tail two edges cross substrate boundaries: m30's acceptance gate touches the **operator substrate (S-G)**, and m32's dispatch crosses into the **Conductor substrate (S-D)**.

The Watcher pre-position is **Class A (activation), Class B (hand-off boundary), Class C (refusal)**. None of these directly observes S-D's enforcement state nor S-G's consent budget — both are substrate-internal until they fail.

The hidden cascade:

```
m23 ──(engine)──> m30 acceptance gate ──E1──> S-G operator (HumanAcceptanceSignature required)
m30 ──(engine)──> m31 ──(engine)──> m32 dispatch
m32 ──E2──> S-D HABITAT-CONDUCTOR (Wave 1B/1C/2/3 panes — weaver/zen/enforcer)
S-D ──E3──> m32 refusal-path (5-check returns; auth/enforcement state propagation)
```

## § 2 — Per-edge dossier

---

### E1 — `m30 → S-G operator` (HumanAcceptanceSignature gate, AP-V7-07)

- **Owner module:** m30 (`bank.accept()`)
- **Trigger:** every m23-proposed workflow that requires bank admission (i.e. not auto-promoted, since AP-V7-07 forbids auto-promotion)
- **Latency expected:** seconds to days (operator-attention-bound; depends on consent budget per [`../substrates/operator.md`](../substrates/operator.md))
- **Engine-observable:** YES at signature-receipt (engine has the signed `HumanAcceptanceSignature` payload); NO at pre-signature operator state
- **Substrate-confirmable:** YES — operator signature is the receipt; signature is durable on `bank.db`
- **Verification surface:** `bank.db` row inspection; m12 acceptance-report shows signed vs pending
- **Silent-failure shape:**
  - **Consent fatigue:** operator stops signing after N proposals/session (per [`../cross-cutting/refusal-taxonomy.md`](../cross-cutting/refusal-taxonomy.md) `OperatorRefusal::ConsentFatigue`); proposals accumulate in queue; m23's variant-generation continues; pressure-register fills with un-acted proposals. Engine sees the queue grow but does not type the cause.
  - **Frame conflict:** operator is in cluster-spec frame; m30 surfacing dispatch frame → `OperatorRefusal::FrameConflict` token; banner ignored or deferred indefinitely
  - **Off-shift:** no operator presence > 30 min; signature requests time out (or rather, never receive a signature); engine has no signal that the operator is structurally absent
- **Substrate-drift shape:** operator's signing threshold drifts (e.g. operator becomes more conservative after a near-miss incident); engine cannot distinguish drift from temporary fatigue
- **Remediation hint:**
  - ConsentFatigue → cool-down period (provisional 10 min); m12 reports surface "you've signed N proposals today — pause?" banner
  - FrameConflict → m30 banner includes context-switch warning; defer or batch
  - Off-shift → mark request as `pending_off_shift` with no timeout; surface on operator next-session m12 report

---

### E2 — `m32 → S-D HABITAT-CONDUCTOR` (Wave 1B/1C/2/3 dispatch)

- **Owner module:** m32 (`dispatch_via_conductor`)
- **Trigger:** every accepted workflow dispatch
- **Latency expected:** sub-second (m32 → Conductor HTTP) + variable (Conductor → weaver/zen/enforcer pane handoff)
- **Engine-observable:** PARTIAL — m32 sees Conductor HTTP 200 / receipt; does NOT observe pane-handoff success on weaver/zen/enforcer
- **Substrate-confirmable:** PROPOSED — Conductor writes `wave_dispatch_received_at` on successful enforcement-pane handoff (substrate-side change request in [`../../ai_docs/decisions/`](../../ai_docs/decisions/))
- **Verification surface:**
  - Conductor `/health` endpoint
  - Conductor wave-pane state (substrate-internal; not engine-visible)
  - `CONDUCTOR_ENFORCEMENT_ENABLED` env flag (engine reads but does not control)
- **Silent-failure shape:**
  - **AP-V7-13 canonical:** Conductor returns HTTP 200 on `/health` but Wave 1B/1C/2/3 panes have `auto_start=false` and were never brought up by `devenv start weaver/zen/enforcer` (per workspace [`CLAUDE.local.md`](../../../CLAUDE.local.md) Active Workstreams). m32 dispatches, gets 200, considers dispatch confirmed — but enforcement never fires.
  - **`CONDUCTOR_ENFORCEMENT_ENABLED=0`** (the soak state) → dispatches accepted at Conductor in NoOp mode; m32 has no signal that enforcement is off; engine looks healthy but every dispatch is a NoOp
  - **Conductor backlog overflow** → 200 accepted but enqueued; long-tail latency hidden from m32
- **Substrate-drift shape:** Conductor wave schema changes (e.g. `wave_id` semantics); m32 dispatches under old shape, Conductor accepts but pane-handoff fails on parse
- **Remediation hint:**
  - Wave panes not up → Luke action B3 standing: `~/.local/bin/devenv -c ~/.config/devenv/devenv.toml start weaver/zen/enforcer`
  - Enforcement off → check `CONDUCTOR_ENFORCEMENT_ENABLED`; flip after NoOp soak passes
  - Schema drift → bump dispatch payload version in m32; F-Contract test against Conductor payload schema

---

### E3 — `S-D → m32 refusal-path` (enforcement state feedback)

- **Owner module:** m32 (5-check sequence — particularly check[0] audit-first and check[2] verification)
- **Trigger:** every m32 dispatch attempt (5-check runs pre-emission)
- **Latency expected:** sub-second (Conductor `/health` HTTP)
- **Engine-observable:** YES — m32 receives Conductor refusal as typed error; per [`../cross-cutting/refusal-taxonomy.md`](../cross-cutting/refusal-taxonomy.md) emits `SubstrateAuthored { S-D, EnforcementDisabled | SemanticEndpointFailed | WeaverZenEnforcerNotStarted }`
- **Substrate-confirmable:** YES — Conductor's HTTP response IS the refusal receipt; no proposed extension required
- **Verification surface:** m32 `DispatchError::ConductorDispatchDisabled` (per [`../ERROR_TAXONOMY.md`](../ERROR_TAXONOMY.md) Cluster G); Watcher Class-C count from `WireEvent::Refusal` emissions
- **Silent-failure shape:**
  - **Refusal collapse:** Conductor's refusal-vs-unavailability response coding inconsistent across versions; m32 surfaces `ConductorUnreachable` for both → wrong category, wrong remediation
  - **5-check check[3] (definition_hash) mismatch** but Conductor's `/health` reports OK → engine cannot distinguish "Conductor accepts but definition stale" from "Conductor refuses on definition" → propagates as wrong refusal class
- **Substrate-drift shape:** Conductor response shape changes (e.g. error body schema); m32 parser fails silently and produces `Bridge::Http` rather than typed refusal
- **Remediation hint:**
  - Refusal collapse → tighten m32 parser to require typed Conductor error body
  - 5-check mismatch → re-run 5-check after definition re-load; if persistent, surface `SubstrateDrift` per [`../cross-cutting/substrate-drift.md`](../cross-cutting/substrate-drift.md)

## § 3 — Substrate-confirmable receipt summary

| Edge | Receipt field | Written by | Read by |
|---|---|---|---|
| E1 | `HumanAcceptanceSignature` on `bank.db` row | operator (signature flow) + m30 (durable write) | m12 reports + Watcher Class-A |
| E2 | `wave_dispatch_received_at` on Conductor pane state | Conductor (substrate-side) | engine subscriber (proposed) |
| E3 | Conductor refusal-body schema | Conductor (existing) | m32 parser |

## § 4 — AP-V7-13 enrichment (the live-bootstrap case)

Per the live habitat-bootstrap snapshot at session-start, POVM `:8125` returned health-200 while serving a pre-CR-2 binary with `learning_health = 0.9162` (post-CR-2 expected ~0.067). The Conductor analogue applies to E2:

- Conductor `:8141` returns health-200 (Wave 0/−1/0.5/0.75/1A LIVE per [`../../CLAUDE.local.md`](../../CLAUDE.local.md))
- Wave 1B/1C/2/3 panes have `auto_start=false` — `enforcement_enabled` is silently 0 in m32's view
- m32 dispatches, sees 200, declares success

This is the **canonical AP-V7-13 case for the dispatch path**: health-200 is liveness, not enforcement-readiness. Per [`../substrates/conductor.md`](../substrates/conductor.md) § lifecycle, E2 requires a **behaviour-verification probe** (e.g. semantic-endpoint check that asks Conductor "is enforcement live?" rather than just "are you alive?").

## § 5 — Test surface (post-G9)

`tests/integration/cc4_substrate_decomposition.rs` — `#[ignore = "requires S-D Conductor + S-G operator"]`:

1. **E1:** drive m30 acceptance flow with mock operator signature; assert signature stored on `bank.db`.
2. **E2:** dispatch m32 with mock Conductor (NoOp mode); assert `wave_dispatch_received_at` if extension live, otherwise health-200 receipt.
3. **E3:** force Conductor to refuse (mock `EnforcementDisabled`); assert m32 emits `WireEvent::Refusal { token: SubstrateAuthored { S-D, EnforcementDisabled } }`.
4. **AP-V7-13 regression:** mock Conductor returning health-200 but `enforcement_check` returning false; assert m32 detects and refuses dispatch.

## § 6 — Refusal-token observability (NA-GAP-11 closure for this contract)

| Edge | Refusal class | Token | Emitting module |
|---|---|---|---|
| E1 | operator non-signature within window | `OperatorRefusal { Luke, ConsentFatigue \| Ambiguity \| OffShift }` | m30 (queue surface) + m12 (report) |
| E1 | operator Ember Held verdict | `OperatorRefusal { Luke, EmberUnanimityHeld }` | m30 (passed through Ember CI gate via m10) |
| E2 | Conductor enforcement disabled | `SubstrateAuthored { S-D, EnforcementDisabled }` | m32 → m40 |
| E2 | Conductor wave panes not started | `SubstrateAuthored { S-D, WeaverZenEnforcerNotStarted }` | m32 → m40 |
| E3 | Conductor semantic-endpoint failure | `SubstrateAuthored { S-D, SemanticEndpointFailed }` | m32 → m40 |

---

> **Back to:** [`INDEX.md`](INDEX.md) · parent [`../synergies/CC-4.md`](../synergies/CC-4.md) · [`../substrates/conductor.md`](../substrates/conductor.md) · [`../substrates/operator.md`](../substrates/operator.md)

*Filed 2026-05-17 (S1002127 · Wave 4.B closeout) · Command · planning-only · HOLD-v2 compliant.*
