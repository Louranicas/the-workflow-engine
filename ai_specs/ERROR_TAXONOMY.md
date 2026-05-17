---
title: ERROR_TAXONOMY — thiserror enums per cluster + cross-cluster propagation
date: 2026-05-17
status: SPEC
---

# ERROR_TAXONOMY — workflow-trace

> **Back to:** [`INDEX.md`](INDEX.md) · [`MODULE_MATRIX.md`](MODULE_MATRIX.md) · [`../README.md`](../README.md) · [`../ARCHITECTURE.md`](../ARCHITECTURE.md) · [`cross-cutting/error-handling.md`](cross-cutting/error-handling.md)

## Cluster-level error enums

Every cluster owns a public error enum (or one per relevant module). All derive `thiserror::Error` + are `#[non_exhaustive]` (per C10 invariant). Variants are structured (named fields) where multiple values matter; `#[from]` for 1:1 conversions from upstream errors.

### Cluster A — Substrate ingest errors

```rust
// m1
pub enum AtuinIngestError {
    DatabaseOpenFailed { path: PathBuf, reason: String },
    BusyTimeout { timeout_ms: u64 },
    QueryFailed(String),
    SubprocessFailed(String),
    RowParseFailed { row_id: i64, reason: String },
}

// m2
pub enum ConsumerError {
    RegistrationFailed { reason: String },
    Disconnected { last_event_at_ms: i64 },
    DedupOverflow,
    Sqlite(#[from] rusqlite::Error),
}

// m3
pub enum InjectionReadError {
    DatabaseOpenFailed { path: PathBuf },
    QueryFailed(String),
    Sqlite(#[from] rusqlite::Error),
}
```

### Cluster B — Observation errors

```rust
// m4
pub enum CascadeError {
    EmptyTrajectory,
    CorrelationOverflow,
}

// m5
pub enum BatternError {
    MissingBegin,
    MissingEnd,
    NestedBattern,
}

// m6
pub enum CostError {
    EmaConvergenceFailed { iters: u32 },
    NegativeVariance,
}
```

### Cluster C — Correlation + output errors

```rust
// m7
pub enum WorkflowRunsError {
    Sqlite(#[from] rusqlite::Error),
    Serde(#[from] serde_json::Error),
    SchemaDrift { column: String, expected: String, got: String },
    F9ZeroWeightViolation { fitness_dimension: f64 },
}

// m12
pub enum ReportError {
    Sqlite(#[from] rusqlite::Error),
    Serde(#[from] serde_json::Error),
    InvalidRange { since: i64, until: i64 },
}

// m13
pub enum StcortexWriterError {
    Bridge(reqwest::Error),
    Timeout { peer: String, elapsed_ms: u64 },
    Namespace(#[from] NamespaceError),
    SubstrateUnavailable,
    Serde(#[from] serde_json::Error),
}
```

### Cluster D — Trust aspect errors

```rust
// m8 (build-time; no runtime error)
// m9
pub enum NamespaceError {
    PrefixMismatch { got: String },
    EmptyId,
}

// m10 (CI-time; produces non-zero exit, not Result)
// m11
pub enum DecayError {
    InvalidInput { field: &'static str, value: f64 },
}
```

### Cluster E — Evidence + pressure errors

```rust
// m14
pub enum LiftError {
    Sqlite(#[from] rusqlite::Error),
    InvalidSampleCount,
}

// m15
pub enum ReservationError {
    Io(#[from] std::io::Error),
    Serde(#[from] serde_json::Error),
    DedupCacheOverflow,
}
```

### Cluster F — Iteration KEYSTONE errors

```rust
// m20
pub enum PrefixSpanError {
    EmptyCorpus,
    GapExceeded { gap: u32, max: u32 },
    Sqlite(#[from] rusqlite::Error),
}

// m21
pub enum VariantBuilderError {
    EmptyCandidate,
    TopKExceedsCorpus { k: u32, corpus: u32 },
}

// m22
pub enum KMeansError {
    InsufficientFeatures { n: u32 },
    ConvergenceFailed { iters: u32 },
}

// m23
pub enum ProposalError {
    LiftEvidenceMissing,
    LineageMismatch,
    NamespaceViolation(#[from] NamespaceError),
    Serde(#[from] serde_json::Error),
}
```

### Cluster G — Bank / select / dispatch / verify errors

```rust
// m30 (per m30 spec)
pub enum BankError {
    AlreadyExists(WorkflowId),
    AutoPromoteRefused,
    SunsetInvalid { accepted_at: i64, sunset_at: i64 },
    EscapeSurfaceInconsistent { declared: EscapeSurfaceProfile, derived: EscapeSurfaceProfile },
    Namespace(#[from] NamespaceError),
    Sqlite(#[from] rusqlite::Error),
    Serde(#[from] serde_json::Error),
}

// m31
pub enum SelectorError {
    EmptyBank,
    SubstrateDegraded { pv2_r: f64, ralph_fitness: f64, thermal: f64 },
    Sqlite(#[from] rusqlite::Error),
}

// m32
pub enum DispatchError {
    ConductorDispatchDisabled,
    VerificationStale { workflow_id: WorkflowId, expired_at: i64 },
    DefinitionDrifted { workflow_id: WorkflowId, expected: String, got: String },
    WorkflowSunset { workflow_id: WorkflowId, sunset_at: i64 },
    CooldownActive { workflow_id: WorkflowId, remaining_ms: u64 },
    SelfDispatchRefused { step_kind: String },
    Bridge(#[from] BridgeError),
    EmptyBank,
}

// m33
pub enum VerifyError {
    AgentDispatchFailed { agent: String, reason: String },
    PartialQuorum { passed: Vec<String>, failed: Vec<String> },
    Bridge(#[from] BridgeError),
    Sqlite(#[from] rusqlite::Error),
}
```

### Cluster H — Substrate feedback errors

```rust
// m40
pub enum NexusEmitError {
    OutboxIo(#[from] std::io::Error),
    Bridge(reqwest::Error),
    Timeout { peer: String, elapsed_ms: u64 },
    CircuitOpen,
}

// m41
pub enum LcmRpcError {
    OutboxIo(#[from] std::io::Error),
    JsonRpc(String),
    Timeout { elapsed_ms: u64 },
    CircuitOpen,
    InvalidShape,
}

// m42
pub enum ReinforceError {
    OutboxIo(#[from] std::io::Error),
    Bridge(reqwest::Error),
    Timeout { peer: String, elapsed_ms: u64 },
    Namespace(#[from] NamespaceError),
    CircuitOpen,
    SubstrateUnavailable,
}
```

### Cross-cluster bridge errors

```rust
pub enum BridgeError {
    ConductorUnreachable,
    SynthexUnreachable,
    LcmUnreachable,
    StcortexUnreachable,
    Timeout { peer: String, elapsed_ms: u64 },
    Http { peer: String, status: u16, body_snippet: String },
}
```

## Cross-cluster propagation

When an error crosses a cluster boundary, wrap with context:

```rust
// m32 in Cluster G calling Cluster H emit
m40_emitter.emit(event).await
    .map_err(|e| DispatchError::Bridge(BridgeError::from(e)))?;
```

The pattern: each cluster's error enum wraps `BridgeError` for cross-cluster failures. `BridgeError` itself wraps the specific lower-level errors (`reqwest::Error`, etc.). This means the call-graph upward preserves both the cluster source AND the lower-level cause.

## RefusalToken — typing refusal by authorship (NA-GAP-02 closure)

Some variants in the cluster-level enums above are **refusals** (a substrate, the engine, or the operator CHOSE to refuse a request) rather than **failures** (an IO error, a parse error, a transient timeout). Per [`cross-cutting/refusal-taxonomy.md`](cross-cutting/refusal-taxonomy.md), refusals carry a `RefusalToken` distinguishing **who authored the refusal**:

```rust
// Defined in cross-cutting/refusal-taxonomy.md § RefusalToken — three top-level kinds
pub enum RefusalToken {
    SubstrateAuthored { substrate_id, refusal_class, recovery_hint, observed_at },
    EngineAuthored    { invariant_id, refusal_class, recovery_hint, observed_at },
    OperatorRefusal   { operator_id, refusal_class, attention_remaining, recovery_hint, observed_at },
}
```

### Which variants above are refusals (vs failures)

Each "refusal" variant carries `Option<RefusalToken>` so the caller can render `recovery_hint` and the `WireEvent::Refusal` emit-path can fire with correct attribution. Variants that are **true failures** (IO, parse, timeout) do NOT carry a token — they carry their original cause and surface as `Unavailability` per the refusal-taxonomy distinction.

| Enum variant | Class | Token kind | Notes |
|---|---|---|---|
| `NamespaceError::PrefixMismatch` | Engine-authored refusal | `EngineAuthored { invariant: m9::namespace_guard, AP30 }` | Engine refuses to write outside `workflow_trace_*` |
| `BankError::AutoPromoteRefused` | Engine-authored refusal | `EngineAuthored { invariant: m30::AP-V7-07, AcceptanceRequiresHumanSignature }` | Engine refuses auto-promote without operator signature |
| `BankError::SunsetInvalid` | Engine-authored refusal | `EngineAuthored { invariant: m30::sunset_window }` | Engine refuses to admit sunset-invalid workflow |
| `BankError::EscapeSurfaceInconsistent` | Engine-authored refusal | `EngineAuthored { invariant: m30::escape_surface_declared_vs_derived }` | Mismatch between declared + derived (7-variant cardinality per D-S1002127-02) |
| `DispatchError::ConductorDispatchDisabled` | Substrate-authored refusal | `SubstrateAuthored { S-D, EnforcementDisabled }` | Conductor refused dispatch (NoOp mode) |
| `DispatchError::VerificationStale` | Engine-authored refusal | `EngineAuthored { invariant: m32::check[2] }` | m32 5-check #2 refused — verification expired |
| `DispatchError::DefinitionDrifted` | Engine-authored refusal | `EngineAuthored { invariant: m32::check[3] }` | m32 5-check #3 refused — definition_hash mismatch |
| `DispatchError::WorkflowSunset` | Engine-authored refusal | `EngineAuthored { invariant: m32::check[4] }` | m32 5-check #4 refused — sunset_at exceeded |
| `DispatchError::CooldownActive` | Engine-authored refusal | `EngineAuthored { invariant: m32::check[5] }` | m32 5-check #5 refused — cooldown active |
| `DispatchError::SelfDispatchRefused` | Engine-authored refusal | `EngineAuthored { invariant: m32::AP-V7-08, SelfDispatch }` | m32 refuses to dispatch a workflow whose steps target m32 |
| `ReinforceError::Namespace(NamespaceError::PrefixMismatch)` | Engine-authored refusal | `EngineAuthored { invariant: m9::namespace_guard }` | m42 wraps m9 refusal at AP30 boundary |
| `ReinforceError::SubstrateUnavailable` | **Unavailability (NOT refusal)** | — (no token; carry `UnavailableReason`) | stcortex not present to respond — not refusing |
| `StcortexWriterError::SubstrateUnavailable` | **Unavailability (NOT refusal)** | — | Same — m13 cannot reach `:3000` |
| `BridgeError::*Unreachable` | **Unavailability (NOT refusal)** | — | Network-level unavailability |

The remaining variants (`*Sqlite`, `*Serde`, `*Bridge(reqwest::Error)`, `*Io`, `Timeout`, `*Failed`) are **true failures** — IO / parse / transient. They do NOT carry tokens and do NOT fire `WireEvent::Refusal`.

### Refusal emission discipline

Every refusal variant must, at its emit site:

1. **Return the typed error** with `Option<RefusalToken>` populated.
2. **Emit `WireEvent::Refusal { token, workflow_id, emitted_by, emitted_at }`** via m40 to `:8092/v3/nexus/push` (per [`cross-cutting/refusal-taxonomy.md`](cross-cutting/refusal-taxonomy.md) § Emission discipline).
3. **Surface `recovery_hint`** through m12 reports + m32 banner (operator-facing).

This makes "successful refusal" first-class wire-protocol traffic, not Watcher-inferred from absence (NA-GAP-11 closure).

### Substrate-authored refusal classes per substrate

Each substrate dossier in [`substrates/`](substrates/) enumerates its own `SubstrateRefusalClass` variants. See:

- [`substrates/atuin.md`](substrates/atuin.md) § 3 — `SqliteBusy`, `DatabaseLocked`, `KvNamespaceMissing`, `SchemaDetectFailure`
- [`substrates/stcortex.md`](substrates/stcortex.md) § 3 — `RefuseWriteNoConsumer`, `InvalidSlug`, `NamespacePolicy`, `ConsumerTokenExpired`, `SchemaMismatch`
- [`substrates/conductor.md`](substrates/conductor.md) § 3 — `EnforcementDisabled`, `SemanticEndpointFailed`, `WeaverZenEnforcerNotStarted`
- [`substrates/synthex.md`](substrates/synthex.md) § 3 — `R13QuietPeriod`, `SchemaRejected`, `RateLimited`, `ConsumerRevoked`
- [`substrates/lcm.md`](substrates/lcm.md) § 3 — `SupervisorNotLive`, `DeployCancelPending`, `SchemaRejected`, `RpcTimeout`, `M0Unverified`
- [`substrates/watcher.md`](substrates/watcher.md) § 3 — `R13QuietPeriod`, `AP27SelfModRefused`, `EmberUnanimityFailed`, `ScopeViolationOutsideM8M51`
- [`substrates/injection_db.md`](substrates/injection_db.md) § 3 — `SqliteBusy`, `SchemaMissing`, `TtlSweepActive`
- [`substrates/operator.md`](substrates/operator.md) § 3 — operator-as-substrate modes (modelled as `OperatorRefusal`, not `SubstrateAuthored`)

### Cross-reference to substrate-drift

A third class of "looks-like-refusal" event is **substrate-drift** — substrate is Available, returns HTTP 200, but its semantics have drifted (CR-2 POVM `learning_health` 13.6× inflation is canonical). This is neither refusal nor unavailability; it is detected via the canary contract in [`cross-cutting/substrate-drift.md`](cross-cutting/substrate-drift.md). When canaries fire `SubstrateDriftDetected`, downstream Result chains MUST treat affected substrate responses as **suspect-until-canary-confirms** even though no Result::Err was returned.

## CLI bin error handling

At the `wf-crystallise` / `wf-dispatch` `main()` level:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.run().await {
        Ok(()) => Ok(()),
        Err(e) => {
            tracing::error!(error = %e, "command failed");
            // map specific error variants to exit codes per API_SPEC
            std::process::exit(map_to_exit_code(&e));
        }
    }
}
```

Exit codes are pinned in [`API_SPEC.md`](API_SPEC.md) per command.

## Per-variant test coverage

Per [`cross-cutting/error-handling.md`](cross-cutting/error-handling.md) § Error-test pattern: every variant of every enum has at least one production-path test asserting it can be constructed via expected failure mode.

```rust
#[test]
fn m30_auto_promote_refused_blocks_agent_signature() {
    let err = bank.accept(&wf, &agent_sig()).unwrap_err();
    assert!(matches!(err, BankError::AutoPromoteRefused));
}
```

## Verify-sync

- **#12** — every error enum derives `thiserror::Error`.
- **#10** — every public error enum is `#[non_exhaustive]`.
- Per-variant test count audit: enumerate variants per enum; assert at least one test exists per variant.

---

> **Back to:** [`INDEX.md`](INDEX.md) · [`cross-cutting/error-handling.md`](cross-cutting/error-handling.md)
