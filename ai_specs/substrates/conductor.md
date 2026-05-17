---
substrate_id: S-D-conductor
kind: http
read_endpoints: ["http://127.0.0.1:8141/health"]
write_endpoints: ["http://127.0.0.1:8141/dispatch (m32 only)"]
lifecycle_phases: [cold-start, warming, steady-state, degraded, refusing, dead]
refusal_modes: [enforcement_disabled, conductor_dispatch_disabled_env, semantic_endpoint_failed, weaver_zen_enforcer_not_started, breaker_open]
drift_indicators: [dispatch_envelope_change, enforcement_flag_semantics, wave_promotion_silent, banner_format_change]
back_pressure_signals: [breaker_state, recent_429_count, weaver_inflight_count]
consent_dimensions: [n/a — substrate is not an operator]
status: SPEC
date: 2026-05-17
authority: Luke @ node 0.A — S1002127 "as per proposal"
hold_v2_compliant: true
addresses: [NA-GAP-01, NA-GAP-02, NA-GAP-04, NA-GAP-07, NA-GAP-08, NA-GAP-09, NA-GAP-10, NA-GAP-11]
---

# S-D — HABITAT-CONDUCTOR (gated-dispatch substrate)

> **Back to:** [`INDEX.md`](../INDEX.md) · [`../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md) · [`../cross-cutting/persistence.md`](../cross-cutting/persistence.md) · [`../cross-cutting/refusal-taxonomy.md`](../cross-cutting/refusal-taxonomy.md) · [`../cross-cutting/substrate-drift.md`](../cross-cutting/substrate-drift.md) · sister substrates [`atuin.md`](atuin.md) · [`stcortex.md`](stcortex.md) · [`injection_db.md`](injection_db.md) · [`synthex.md`](synthex.md) · [`lcm.md`](lcm.md) · [`watcher.md`](watcher.md) · [`operator.md`](operator.md)
>
> Engine consumer: **m32** ([`../modules/cluster-G/m32_conductor_dispatcher.md`](../modules/cluster-G/m32_conductor_dispatcher.md))

## 1. Purpose & boundary

HABITAT-CONDUCTOR is the habitat's **gated-dispatch substrate** — the weaver/zen/enforcer triad at `:8141` plus 4 binaries in `~/.local/bin/`. Currently in Wave-3 LANDED state; auto_start=false in Batch 5. Luke @ terminal action required to bring weaver/zen/enforcer up. The `CONDUCTOR_ENFORCEMENT_ENABLED` env var is a substrate-side flag that gates whether the substrate refuses or executes dispatches.

**IN scope for `workflow-trace`:** m32 is the **only** dispatch path; every dispatch goes through `:8141`. m32 NEVER executes via `Command::*` (AP-V7-08 self-dispatch refusal).

**OUT of scope:** running Conductor; modifying its WASM plugins; touching weaver/zen/enforcer internals; flipping `CONDUCTOR_ENFORCEMENT_ENABLED` (Luke's hand only).

## 2. Lifecycle phases

| Phase | Indicator | Engine action |
|---|---|---|
| cold-start | weaver/zen/enforcer not yet started; `:8141/health` unreachable | m32 returns `DispatchError::ConductorNotLive` with remediation banner |
| warming | started but in NoOp soak (24h required pre-enforcement) | m32 dispatches succeed but enforcer is NoOp; Watcher Class-A pre-positioned for first dispatch |
| steady-state | NoOp soak passed; `CONDUCTOR_ENFORCEMENT_ENABLED=1` flipped | m32 dispatches succeed with full enforcement; banner emitted |
| degraded | breaker HALF_OPEN; or semantic endpoint flake | m32 backs off; circuit half-opens after 30s |
| refusing | `CONDUCTOR_DISPATCH_ENABLED != "1"` (env) or `:8141/health` body says enforcement_active=false | m32 returns `DispatchError::ConductorDispatchDisabled`; hard-refuse, no fall-through |
| dead | `:8141` connection refused | m32 returns `DispatchError::ConductorNotLive`; surfaces remediation banner with `devenv start weaver/zen/enforcer` |

## 3. Refusal modes (substrate-authored)

This substrate is the canonical case for NA-GAP-02 (substrate-authored vs engine-authored refusal):

- **`EnforcementDisabled`** (substrate-authored) — substrate responds 200 to `/health` but body says enforcement is off. m32 must NOT dispatch even though substrate is "up". This is a **substrate-side refusal**, not unavailability. Recovery hint: Luke flips `CONDUCTOR_ENFORCEMENT_ENABLED=1` post-soak.
- **`ConductorDispatchDisabled`** (engine-authored adjacent) — `CONDUCTOR_DISPATCH_ENABLED` env on engine side; m32 reads. Hybrid: env is engine-read but the policy is operator-mediated.
- **`SemanticEndpointFailed`** (substrate-authored) — health 200 but `body.status != "ok"`. AP-V7-13 mitigation. Substrate-authored "I'm sick".
- **`WeaverZenEnforcerNotStarted`** (substrate-authored) — health 200 from a partial bring-up where one of three is missing. Recovery: surface to operator.
- **`BreakerOpen`** (engine-authored) — 2 consecutive failures opened the breaker.

The crucial distinction NA-GAP-02 surfaces: today INVARIANT_MAP collapses these into one row; `RefusalToken` typing makes the recovery semantics distinct.

## 4. Drift indicators (closes NA-GAP-07)

- **Dispatch envelope change** — `ConductorDispatchRequest` schema bumps without notice. Detector: envelope-hash check at m32 startup.
- **Enforcement-flag semantics shift** — substrate redefines `CONDUCTOR_ENFORCEMENT_ENABLED` from "all-or-nothing" to "per-step". Detector: flag-semantics contract check.
- **Wave promotion silent** — substrate advances from Wave-1A to Wave-1B autonomously (e.g. on first NoOp soak pass). Engine may assume wave-1A semantics; reads break. Detector: wave-version probe.
- **Banner format change** — substrate-side log format breaks operator's eye scan. Detector: banner-format check on `/dispatch` response.

## 5. Back-pressure signals (closes NA-GAP-04)

- **Breaker state** — m40_42_common::breaker tracks consecutive failures.
- **Recent 429 count** — substrate may rate-limit in future.
- **Weaver inflight count** — observable via `:8141/metrics` if enabled; engine should slow if > 16.

## 6. Receipts (closes NA-GAP-09)

Conductor returns `{ accepted: bool, dispatch_id: String, enforcement_mode: "NoOp" | "Enforcing" }` — substrate-side receipt. m32 records this in `dispatch_log.db` as `audit-first → patch on response`. The receipt confirms **substrate ingestion** of the dispatch; it does NOT confirm step execution (that lives in Conductor's downstream lanes).

Per NA-GAP-11, when m32 refuses (any 5-check failure), m32 MUST emit `WireEvent::Refusal { token: ... }` via m40 to NexusEvent push so refusal is substrate-readable, not Watcher-inferred-from-absence. See [m32 spec](../modules/cluster-G/m32_conductor_dispatcher.md) amendment.

## 7. Capabilities & namespaces (closes NA-GAP-10)

Conductor accepts dispatches from any client with valid request shape; no per-consumer ACL today. Namespace gating happens at the dispatch payload level: `workflow_id` carries the engine's namespace prefix (`workflow_trace_*` via AP30).

NA-GAP-10's v0.2.0 extension: Conductor could maintain a per-engine reputation score (correlating workflow outcomes with engine-source); revoke or down-rate misbehaving engines. Not in scope today.

## 8. Substrate-internal couplings (closes NA-GAP-03)

Conductor's substrate-internal edges:
- **Conductor → weaver/zen/enforcer triad** — substrate-internal; engine triggers but does not observe.
- **Conductor → step execution lanes** — substrate-internal; engine receives accept/reject only.
- **Conductor ← Luke @ terminal** — `devenv start` / env flag flip is operator-mediated substrate control.
- **Conductor → operator** — banner output to operator's terminal (substrate writes to operator's attention substrate via m32's stdout banner).

The Conductor-to-operator edge is the **display-before-step** invariant (m32 Gap 3 display half). This is a substrate-substrate edge: S-D writes to S-G via stdout banner channel.

## 9. Test-fixture sketch (closes NA-GAP-08)

Fixtures at `tests/substrate_fixtures/conductor/`:

- **`conductor_enforcement_flag_off_fixture`** — emulator returns 200 + body `enforcement=false`; asserts m32 refuses dispatch with structured error.
- **`conductor_not_started_fixture`** — emulator returns connection-refused; asserts remediation banner emitted.
- **`semantic_endpoint_flake_fixture`** — emulator returns 200 + body `status=degraded`; asserts m32 surfaces typed refusal.
- **`weaver_only_started_fixture`** — emulator returns 200 + body indicating zen/enforcer absent; asserts m32 holds dispatch.
- **`wave_promotion_silent_fixture`** — emulator advances wave version; asserts version-probe catches drift.

## 10. Watcher class pre-positions

- **Class A (activation)** — first successful workflow-trace dispatch through Conductor
- **Class B (boundary)** — every m32 dispatch attempt
- **Class C (refusal)** — substrate-authored EnforcementDisabled or SemanticEndpointFailed; THIS IS HEALTHY behaviour, Class-C count > 0 in steady-state
- **Class D (drift)** — envelope/wave/flag-semantics hash drift

---

> **Back to:** [`INDEX.md`](../INDEX.md) · [`../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md)

*Filed 2026-05-17 (S1002127 · Wave 4 NA-remediation) · Luke "as per proposal" · planning-only · HOLD-v2 compliant.*
