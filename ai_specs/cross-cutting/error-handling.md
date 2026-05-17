---
title: cross-cutting/error-handling — thiserror + ? propagation + cross-cluster propagation
date: 2026-05-17
status: SPEC
axes: [error-taxonomy, propagation, anti-patterns]
consolidates: ERROR_TAXONOMY.md (module-side guidance)
---

# Error Handling — Module-Side Guidance

> **Back to:** [`../INDEX.md`](../INDEX.md) · [`../ERROR_TAXONOMY.md`](../ERROR_TAXONOMY.md) · [`../../README.md`](../../README.md) · [`../../ARCHITECTURE.md`](../../ARCHITECTURE.md)

## Purpose

This axis spec is the **module-side translation** of [`../ERROR_TAXONOMY.md`](../ERROR_TAXONOMY.md). The root taxonomy enumerates the cluster-level error enums; here is what every module MUST do for propagation, conversion, and the `?` operator.

## thiserror per-module discipline

Every module owns an error enum:

```rust
#[derive(Debug, thiserror::Error)]
pub enum BankError {
    #[error("bank: workflow {0} already exists")]
    AlreadyExists(WorkflowId),

    #[error("bank: acceptance requires interactive human signature (AP-V7-07)")]
    AutoPromoteRefused,

    #[error("bank: sunset_at ({sunset_at}) must be > accepted_at ({accepted_at})")]
    SunsetInvalid { accepted_at: i64, sunset_at: i64 },

    #[error("bank: sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("bank: serde: {0}")]
    Serde(#[from] serde_json::Error),
}
```

- **Variants enumerate failure modes**, never loosely-typed `Generic(String)`.
- **`#[error("...")]` messages include operational guidance**, not just description.
- **Named-field variants** (`{ accepted_at, sunset_at }`) over tuple variants where multiple values matter — programmatic matching.
- **`#[from]`** for upstream errors that are 1:1 wrappable (rusqlite, serde_json).

## Propagation — the `?` operator and `.map_err`

```rust
pub fn accept(&self, wf: &AcceptedWorkflow) -> Result<(), BankError> {
    // ? propagates rusqlite::Error → BankError::Sqlite (via #[from])
    let conn = self.pool.get()?;

    // .map_err for context-adding when no #[from] conversion exists
    let parsed: ParsedSteps = serde_json::from_str(&wf.steps_json)
        .map_err(BankError::Serde)?;

    // .map_err with structured context for cross-cluster boundaries
    bridge_client.dispatch(payload)
        .await
        .map_err(|e| BridgeError::ConductorDispatchFailed {
            source: e,
            workflow_id: wf.id.clone(),
        })?;

    Ok(())
}
```

## Cross-cluster propagation pattern

When an error crosses a cluster boundary, wrap with context:

```rust
// in m32 (Cluster G), calling into Cluster H emit
m40_emitter.emit(event).await
    .map_err(|e| DispatchError::DownstreamEmitFailed {
        source: e,
        downstream: "m40_synthex",
        workflow_id: outcome.workflow_id,
    })?;
```

The **cluster-G error knows about its cluster-H caller**, not vice versa. Errors flow upward through the call graph; cross-cluster boundaries add cluster-name context for log correlation.

## Anti-patterns

- **`unwrap()` outside tests.** Hard-banned by `deny(clippy::unwrap_used)`. The only exception is FFI hot paths with documented `// rationale:` comment.
- **`Box<dyn Error>` in lib types.** Loses variant pattern-matching. Reserved for bin top-level `anyhow::Result<()>` only.
- **`.ok()` discarding Result without rationale.** Hard-banned by clippy + `silent-swallow-detect` skill. Allowed only with `// rationale:` comment.
- **`let _ = ...` for sync side-effects without rationale.** Same as `.ok()`.
- **`Generic(String)` variants.** Use named-field structured variants instead.
- **`panic!` outside startup / build-time invariant checks.** Programmatic recovery impossible.
- **Sentinel-Ok returns** like `Ok(0)`, `Ok(())` on a path that should have surfaced an error — `silent-swallow-detect` catches these.

## `anyhow` vs `thiserror`

- **`thiserror`** in libs and modules — typed error enums callers pattern-match on.
- **`anyhow`** at bin entry-points only — `fn main() -> anyhow::Result<()>` for ergonomic top-level error formatting.

## Error-test pattern

Every variant of every error enum has at least one test asserting the variant is producible:

```rust
#[test]
fn bank_auto_promote_refused_when_not_interactive() {
    let bank = BankDb::open_in_memory().unwrap();
    let wf = test_workflow();
    let sig = HumanAcceptanceSignature {
        accepted_by: "test".into(),
        accepted_at: 0,
        interactive_terminal: false,
    };
    let err = bank.accept(&wf, &sig).unwrap_err();
    assert!(matches!(err, BankError::AutoPromoteRefused));
}
```

## Verify-sync invariants

- **#12** — every error enum derives `thiserror::Error`.
- Variant coverage: every enum variant has at least one production-path test asserting it can be constructed.

---

> **Back to:** [`../INDEX.md`](../INDEX.md) · [`../ERROR_TAXONOMY.md`](../ERROR_TAXONOMY.md)
