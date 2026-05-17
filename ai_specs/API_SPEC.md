---
title: API_SPEC — public APIs across wf-crystallise + wf-dispatch + workflow_core
date: 2026-05-17
status: SPEC
binaries: [wf-crystallise, wf-dispatch]
lib: workflow_core
---

# API_SPEC — workflow-trace

> **Back to:** [`INDEX.md`](INDEX.md) · [`MODULE_MATRIX.md`](MODULE_MATRIX.md) · [`../README.md`](../README.md) · [`../ARCHITECTURE.md`](../ARCHITECTURE.md) · [`../ai_docs/GENESIS_PROMPT_V1_3.md`](../ai_docs/GENESIS_PROMPT_V1_3.md)

## Surface inventory

Three public-API surfaces:

1. **`wf-crystallise` CLI** — ingest, observation, iteration KEYSTONE, substrate-feedback emit.
2. **`wf-dispatch` CLI** — bank admission, selection, dispatch, verification.
3. **`workflow_core` library** — shared types, schemas, namespace constants, error taxonomy.

No HTTP API as default. Optional `feature = "serve"` for live `wf-status` data per Genesis v1.3 § 6 Axis 3, default off.

---

## 1. `wf-crystallise` CLI (commands)

### `wf-crystallise ingest`

Run a single ingest tick: m1 pages atuin, m2 polls stcortex consumer, m3 partitions injection.db.

```
USAGE:
    wf-crystallise ingest [--since-id <i64>] [--max-rows <usize>] [--dry-run]

OPTIONS:
    --since-id   only ingest atuin rows with id > N (resume cursor)
    --max-rows   cap rows per source (default 10_000)
    --dry-run    parse + classify but do not write to m7

EXIT:
    0  success
    2  substrate read error
    3  m9 namespace violation (refuse-mode; correct behaviour)
```

### `wf-crystallise propose`

Run the iteration KEYSTONE: m20 PrefixSpan → m21 variants → m22 K-means → m23 ProposalBuilder.

```
USAGE:
    wf-crystallise propose [--cluster-id <hex>] [--n-min <u32>=20] [--max-proposals <u32>]

OPTIONS:
    --cluster-id  restrict to a single opaque m4 cluster_id (16-hex)
    --n-min       Wilson-CI sample-size floor (default 20; below = no propose)
    --max-proposals  cap on emissions per tick

EXIT:
    0  success (proposals emitted to review queue)
    3  LiftEvidenceMissing (correct refusal — below n_min)
```

### `wf-crystallise report`

Emit a CLI report (stdout default; `--json` structured).

```
USAGE:
    wf-crystallise report [--since <sha|iso8601>] [--until <sha|iso8601>] [--json] [--sample-pack]

OPTIONS:
    --since     start of report window
    --until     end of report window
    --json      machine-readable output (for downstream tools)
    --sample-pack  emit Ember-rubric-testable output samples to target/output-samples/

EXIT:
    0  success
```

### `wf-crystallise reservation-notices`

List recent m15 pressure events from agent-cross-talk/.

```
USAGE:
    wf-crystallise reservation-notices [--since <iso8601>] [--kind <event_kind>]

EXIT:
    0  success
```

---

## 2. `wf-dispatch` CLI (commands)

### `wf-dispatch propose accept <proposal_id>`

Interactive subcommand — admits a `WorkflowProposal` into the curated bank. Requires `isatty()`.

```
USAGE:
    wf-dispatch propose accept <PROPOSAL_ID> [--curator-note <text>] [--sunset-days <u32>=120]

INTERACTIVE PROMPTS:
    "Accept proposal {id} into curated bank? [y/N]"
    "Operator username: " (cannot be empty, "agent", or "auto")

WRITES:
    m30 BankDb::accept(wf, HumanAcceptanceSignature { interactive_terminal: true, ... })

EXIT:
    0  accepted
    2  user-cancelled (typed N at prompt)
    3  AutoPromoteRefused (operator typed "agent" / "auto" / empty)
    4  EscapeSurfaceInconsistent
    5  SunsetInvalid
```

### `wf-dispatch bank list`

List eligible bank entries (sunset not expired).

```
USAGE:
    wf-dispatch bank list [--limit <usize>=50] [--include-sunset]
```

### `wf-dispatch verify <workflow_id>`

Run the m33 4-agent verifier dry-run. Produces PASS / FAIL / DEGRADED with 7-day `last_verified_at` TTL.

```
USAGE:
    wf-dispatch verify <WORKFLOW_ID> [--ttl-days <u32>=7] [--agents <list>]

EXIT:
    0  PASS
    1  FAIL (refusal; correct)
    2  DEGRADED (refusal; correct)
```

### `wf-dispatch select`

Run m31 selector once; emit selected workflow (or refuse-mode flag).

```
USAGE:
    wf-dispatch select [--limit <usize>=1] [--explain]

OUTPUT:
    selected workflow_id + composite_score breakdown
    OR refuse-mode: "no eligible workflows" / "degraded substrate"
```

### `wf-dispatch dispatch <workflow_id>`

Run m32's 5-check pre-dispatch sequence then dispatch via Conductor.

```
USAGE:
    wf-dispatch dispatch <WORKFLOW_ID>

CHECKS:
    1. Conductor :8141/health
    2. m33 VerifyResult TTL fresh
    3. definition_hash matches
    4. sunset_at > now
    5. dispatch_cooldown elapsed

EXIT:
    0  dispatched (DispatchOutcome captured)
    3  ConductorDispatchDisabled
    4  VerificationStale
    5  DefinitionDrifted
    6  WorkflowSunset
    7  CooldownActive
```

---

## 3. `workflow_core` library export surface

```rust
// public module structure
pub mod types {
    pub struct WorkflowId(String);     // UUIDv7
    pub struct ProposalId(String);     // UUIDv4
    pub struct LineageId(String);
    pub struct SessionId(String);
    pub struct BatternId(String);
    pub struct OpaqueClusterId(String); // FNV-1a-16-hex
    pub struct ConsumerId(String);
    pub struct StepId(String);
    pub struct StepDef { /* ... */ }
}

pub mod schemas {
    pub struct ConsumerInputs { /* JSONB shape for m7.consumer_inputs */ }
    pub struct WorkflowRunRow { /* m7 hub row */ }
    pub struct WorkflowProposal { /* m23 emit type */ }
    pub struct AcceptedWorkflow { /* m30 row */ }
    pub struct VerifyResult { /* m33 emit type */ }
    pub struct DispatchOutcome { /* m32 emit type */ }
    // ...
}

pub mod namespace {
    pub const WORKFLOW_TRACE_PREFIX: &str = "workflow_trace_";
    pub const WORKFLOW_TRACE_OUTCOME_PREFIX: &str = "workflow_trace_outcome_";
    pub const CONSUMER_PREFIX: &str = "workflow_trace_consumer_";
    // ... AP30 constants only; no inline strings allowed downstream
}

pub mod errors {
    pub enum BankError { /* ... */ }
    pub enum DispatchError { /* ... */ }
    pub enum BridgeError { /* ... */ }
    pub enum LiftError { /* ... */ }
    pub enum ProposalError { /* ... */ }
    pub enum ReinforceError { /* ... */ }
    pub enum NamespaceError { /* ... */ }
    // ... per-module enums re-exported
}

pub mod stages {
    // re-exports of Phase A vs Phase B verbs per Genesis v1.3 § 3
    pub enum VerbClass { Passive, Active }
}
```

## API stability commitments

- **All public types** carry semver guarantees per Genesis v1.3 § 1 lock.
- **Newtype-only domain values** — no raw `String` / `i64` for identity-bearing fields.
- **`#[non_exhaustive]`** on every public error enum to allow variant addition without semver-major.
- **Doc comments on every public item** with `# Errors` section on fallible fns (god-tier rule 9).
- **`#[must_use]`** on every public fn returning `Result` / `Option` / builder.

## CLI stability commitments

- Command names locked at v1.3 (`ingest` / `propose` / `report` / `propose accept` / `bank list` / `verify` / `select` / `dispatch`).
- Exit codes locked at v1.3.
- Option flags stable; new flags additive only.
- Help text generated from clap derive.
- `--version` outputs SemVer + commit SHA + build profile per god-tier per-binary discipline.

---

> **Back to:** [`INDEX.md`](INDEX.md) · [`MODULE_MATRIX.md`](MODULE_MATRIX.md) · [`../README.md`](../README.md) · [`../ai_docs/GENESIS_PROMPT_V1_3.md`](../ai_docs/GENESIS_PROMPT_V1_3.md)
