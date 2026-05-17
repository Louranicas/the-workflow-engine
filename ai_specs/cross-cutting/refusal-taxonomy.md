---
title: cross-cutting/refusal-taxonomy — RefusalToken across substrate / engine / operator authorship
date: 2026-05-17
status: SPEC
axes: [refusal-typing, substrate-vs-engine, operator-as-substrate, observability]
authority: Luke @ node 0.A — S1002127 "as per proposal"
hold_v2_compliant: true
addresses: [NA-GAP-02, NA-GAP-05, NA-GAP-11]
---

# Refusal Taxonomy — RefusalToken

> **Back to:** [`../INDEX.md`](../INDEX.md) · [`../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md) · [`../ERROR_TAXONOMY.md`](../ERROR_TAXONOMY.md) · [`./substrate-drift.md`](substrate-drift.md) · [`../substrates/`](../substrates/) (8 dossiers) · [`../EVENT_SYSTEM_SPEC.md`](../EVENT_SYSTEM_SPEC.md) · [`../../ultramap/INVARIANT_MAP.md`](../../ultramap/INVARIANT_MAP.md)

## Purpose

The scaffold's existing `ERROR_TAXONOMY.md` types every fallible operation as a `Result<T, ModuleError>` with thiserror enums. INVARIANT_MAP § Runtime invariants lists `WriteError::NamespaceViolation` (engine-authored, m9), `ReinforceOutcome::SubstrateUnavailable` (substrate-availability), and refuse-write at stcortex DB layer (substrate-authored refusal) in the **same table without typing the authorship distinction** (NA-GAP-02). When stcortex refuse-write fires because a Claude session forgot the consumer-register step, the engine surfaces `ReinforceOutcome::SubstrateUnavailable` — wrong category — and the operator restarts the wrong service.

This spec introduces `RefusalToken` — a top-level taxonomy that types refusal by **who authored it**: substrate, engine, or operator. The taxonomy is consumed by:

1. **`Result<T, E>` typed errors** — module-level error enums carry a `token: RefusalToken` field when the error class is a refusal (vs. a true failure like IO error).
2. **`WireEvent::Refusal`** — Class-C envelope on the NexusEvent bus so refusals are engine-emitted, not Watcher-inferred (NA-GAP-11).
3. **m12 operator reports** — refusal summaries grouped by token kind so the operator sees substrate-vs-engine-vs-self refusals distinctly.

## RefusalToken — three top-level kinds

```rust
// SPEC ONLY — not compileable code
pub enum RefusalToken {
    SubstrateAuthored {
        substrate_id: SubstrateId,             // S-A | S-B | S-C | S-D | S-E | S-F | S-watcher | S-G
        refusal_class: SubstrateRefusalClass,  // RefuseWriteNoConsumer | InvalidSlug | EnforcementDisabled | ...
        recovery_hint: Cow<'static, str>,      // substrate-side remediation
        observed_at: i64,
    },
    EngineAuthored {
        invariant_id: InvariantId,             // m9::namespace_guard | m32::five_check[2] | m33::dataexfil_unanimous
        refusal_class: EngineRefusalClass,     // NamespaceViolation | VerificationStale | SelfDispatchRefused | ...
        recovery_hint: Cow<'static, str>,      // engine-side remediation (often "amend spec")
        observed_at: i64,
    },
    OperatorRefusal {
        operator_id: OperatorId,               // Luke | Zen | future-review-queue-member
        refusal_class: OperatorRefusalClass,   // ConsentFatigue | AttentionOverload | FrameConflict | Ambiguity | LatencyDrift | OffShift | EmberUnanimityHeld
        attention_remaining: Option<i64>,      // milliseconds; None for OffShift
        recovery_hint: Cow<'static, str>,      // operator-side remediation (often "wait" or "rephrase")
        observed_at: i64,
    },
}
```

Each enum kind carries a per-substrate / per-invariant / per-operator-class enum for the refusal class:

### SubstrateRefusalClass (per substrate)

Per-substrate refusal classes are defined in each substrate dossier (see [`../substrates/atuin.md`](../substrates/atuin.md) § 3, [`stcortex.md`](../substrates/stcortex.md) § 3, etc.). Canonical examples:

- `S-A atuin`: `SqliteBusy`, `DatabaseLocked`, `KvNamespaceMissing`, `SchemaDetectFailure`
- `S-B injection.db`: `SqliteBusy`, `SchemaMissing`, `TtlSweepActive`
- `S-C stcortex`: `RefuseWriteNoConsumer`, `InvalidSlug`, `NamespacePolicy`, `ConsumerTokenExpired`, `SchemaMismatch`
- `S-D conductor`: `EnforcementDisabled`, `SemanticEndpointFailed`, `WeaverZenEnforcerNotStarted`
- `S-E synthex`: `R13QuietPeriod`, `SchemaRejected`, `RateLimited`, `ConsumerRevoked`
- `S-F lcm`: `SupervisorNotLive`, `DeployCancelPending`, `SchemaRejected`, `RpcTimeout`, `M0Unverified`
- `S-watcher`: `R13QuietPeriod`, `AP27SelfModRefused`, `EmberUnanimityFailed`, `ScopeViolationOutsideM8M51`
- `S-G operator`: (modelled as `OperatorRefusal`, not `SubstrateAuthored`, even though operator IS a substrate per NA-GAP-05)

### EngineRefusalClass

Engine-authored refusals are the engine's own invariant enforcement:

- `NamespaceViolation` (m9) — write attempted under non-`workflow_trace_*` prefix
- `VerificationStale` (m32 check 2) — m33 verification older than TTL
- `DefinitionDrifted` (m32 check 3) — definition_hash mismatch
- `Sunset` (m32 check 4) — workflow sunset_at exceeded
- `CooldownActive` (m32 check 5) — per-workflow dispatch cooldown active
- `SelfDispatchRefused` (m32 AP-V7-08) — workflow steps target m32 itself
- `AcceptanceRequiresHumanSignature` (m30 AP-V7-07) — auto-promotion attempted
- `BankAuditWriteFailed` (m32 check 0) — audit-first write failed

### OperatorRefusalClass (per operator-as-substrate per NA-GAP-05)

- `ConsentFatigue` — banner-count exceeded per-session cap (provisional 5)
- `AttentionOverload` — mid-frame-switch; engine should defer
- `FrameConflict` — operator in cluster-spec frame; engine surfacing dispatch frame
- `Ambiguity` — operator asked clarifying question
- `LatencyDrift` — operator non-response beyond expected window
- `OffShift` — no operator presence for > 30 min
- `EmberUnanimityHeld` — Ember §5.1 Held verdict on a string

## Unavailable is NOT a refusal

The taxonomy explicitly separates **refusal** (substrate / engine / operator CHOSE to refuse) from **unavailability** (substrate is not present to respond):

```rust
pub enum SubstrateAvailability {
    Available,
    Unavailable {
        substrate_id: SubstrateId,
        reason: UnavailableReason,           // ConnectionRefused | DnsFail | TimeoutNoResponse | ProcessNotRunning
        backoff_recommendation: Duration,
        observed_at: i64,
    },
    Recovering,                              // breaker HALF_OPEN
}
```

When stcortex `:3000` refuses connection (process not running), it is **Unavailable**, not refusing. When stcortex `:3000` accepts connection and the reducer returns `RefuseWrite::NoConsumer`, it is **SubstrateAuthored refusal**.

The CR-2 incident's distinct symptom: POVM `:8125` was Available AND returning HTTP 200, but the *semantics* were drifted — neither refusal nor unavailability, but **substrate-drift**. See [`./substrate-drift.md`](substrate-drift.md).

## WireEvent::Refusal — Class-C envelope (closes NA-GAP-11)

Per NA-GAP-11, refusal must be substrate-readable as a first-class wire-protocol event, not Watcher-inferred from absence:

```rust
// SPEC ONLY
pub enum WireEvent {
    Run { workflow_id: WorkflowId, outcome: DispatchOutcome },
    Refusal {
        token: RefusalToken,
        workflow_id: Option<WorkflowId>,     // None for substrate-level refusals not tied to a workflow
        emitted_by: ModuleId,                // m32 | m42 | m30 | m41
        emitted_at: i64,
    },
    Capability { /* ... */ },
}
```

Emission discipline:
- **m32 refusal** (any of the 5-check failures) emits `WireEvent::Refusal { token: EngineAuthored { ... } }` via m40 to `/v3/nexus/push`.
- **m42 stcortex refusal** emits `WireEvent::Refusal { token: SubstrateAuthored { substrate_id: S-C, ... } }`.
- **m30 acceptance-signature-missing** emits `WireEvent::Refusal { token: EngineAuthored { invariant: AP-V7-07, ... } }`.
- **Operator refusal** (consent-fatigue, ambiguity, etc.) emits `WireEvent::Refusal { token: OperatorRefusal { ... } }` — surfaced from m12 reports or m32 banner-feedback channel.

The Watcher Class-C count is then sourced from `WireEvent::Refusal` emissions, NOT from absence-of-dispatch heuristics. This makes the "successful refusal" observable.

## Recovery-hint discipline

Every `RefusalToken` carries a `recovery_hint` (Cow<'static, str>) so the operator's downstream surface (m12 report, m32 banner) can render an actionable message instead of just an error class. Examples:

| Token | recovery_hint |
|---|---|
| `SubstrateAuthored { S-C, RefuseWriteNoConsumer, ... }` | "Call `stcortex register_consumer` for this session before retry." |
| `SubstrateAuthored { S-D, EnforcementDisabled, ... }` | "Conductor enforcement off. Luke: flip `CONDUCTOR_ENFORCEMENT_ENABLED=1` after NoOp soak passes." |
| `EngineAuthored { m9, NamespaceViolation, ... }` | "Write must be under `workflow_trace_*` prefix (AP30). Use `workflow_core::namespace` constants." |
| `OperatorRefusal { Luke, ConsentFatigue, ... }` | "Banner cap (5) exceeded. Cooldown 10 min before next dispatch." |
| `OperatorRefusal { Luke, EmberUnanimityHeld, ... }` | "Ember §5.1 Held — re-author user-facing string before re-submission." |

## Invariants

| # | Invariant | Enforcement |
|---|---|---|
| 1 | Every fallible API returns `Result<T, E>` where `E` carries `Option<RefusalToken>` for the refusal case | clippy `disallowed_methods` ban on `?` returning untyped error; CI grep gate |
| 2 | `RefusalToken` is serialisable to JSON for outbox + wire emission | `#[derive(serde::Serialize)]` on enum + variant fields |
| 3 | `WireEvent::Refusal` MUST be emitted for every refusal — no silent typed-error-only path | integration test: refuse a write at each substrate, assert wire emission fires |
| 4 | `SubstrateAuthored` vs `EngineAuthored` vs `OperatorRefusal` mutually exclusive at any call site | property test on classification function |
| 5 | `recovery_hint` non-empty for every refusal variant | exhaustive match test in unit suite |

## Closure test (post-G9)

`tests/integration/refusal_taxonomy_round_trip.rs` — for each of the 8 substrate dossiers, drive a refusal scenario and assert: (a) typed error returned with correct `RefusalToken`; (b) `WireEvent::Refusal` emitted to outbox; (c) operator-surface render of `recovery_hint` non-empty; (d) Class-C taxonomy correctly assigned by downstream Watcher.

---

> **Back to:** [`../INDEX.md`](../INDEX.md) · [`../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md) · [`../substrates/`](../substrates/)

*Filed 2026-05-17 (S1002127 · Wave 4 NA-remediation) · Luke "as per proposal" · planning-only · HOLD-v2 compliant.*
