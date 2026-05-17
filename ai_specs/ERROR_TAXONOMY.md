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
