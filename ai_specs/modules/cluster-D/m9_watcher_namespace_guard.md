---
title: m9 — watcher_namespace_guard (write-time AP30 prefix validator)
module_id: m9
cluster: D — Trust (cross-cutting)
layer: L4
binary: wf-crystallise
feature_gate: [none]
verb_class: refuse
ship_first: true
gap_owner: [Gap 3 — partial]
status: SPEC · planning-only · HOLD-v2 · NO CODE · NO CARGO
loc_budget: 50
test_budget: 50
mutation_kill: 70
boilerplate_lift: 20
date: 2026-05-17 (S1001982)
authority: Luke @ node 0.A — AP24 gate (G9 "start coding workflow-trace")
binding_spec: Genesis Prompt v1.3 § 1, § 2
primary_contract: CC-2 (Trust Layer Woven — D → all)
co_owns: Gap 3 (Unified destructiveness / EscapeSurfaceProfile schema — namespace dimension; shared with m30 + m32)
cardinality_amendment: "S1002127 — PrivilegeEscalation inserted at ordinal 30 (D-S1002127-02 ADR)"
---

> **🏷 v0.1.0 — SD A/B reconciliation (S1004115 Phase 9 / § 15 D27):**
> The shipped m9 implementation is canonical. **SD1** (ControlChar
> surface) was a code-ahead-of-spec drift — see
> `src/m9_watcher_namespace_guard/` for the authoritative behaviour.
> Phase 6e added the `AcceptanceSignatureReader` trait seam +
> `NamespaceViolation::CapabilityNotAcknowledged` typed refusal. Spec
> amendments mirror the shipped surface; no behavioural divergence
> remains. Full disposition: [`PHASE9_SD_RECONCILIATION_S1004115.md`](../../../ai_docs/PHASE9_SD_RECONCILIATION_S1004115.md).

# m9 — `watcher_namespace_guard`

> **Back to:** [`cluster-D/INDEX`](./) · [`ai_specs/INDEX`](../../INDEX.md) · [`MODULE_MATRIX`](../../MODULE_MATRIX.md) · vault [[cluster-D-trust-cross-cutting]] · [cluster-D plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-D.md) · [phase-1](../../../the-workflow-engine-vault/deployment%20framework/phase-1-genesis-day-0-3.md) · [Genesis v1.3](../../../ai_docs/GENESIS_PROMPT_V1_3.md)
>
> **Sister modules (Cluster D):** [m8](m8_povm_build_prereq.md) · [m9](m9_watcher_namespace_guard.md) · [m10](m10_ember_ci_gate.md) · [m11](m11_fitness_weighted_decay.md)

---

## 1. Purpose

A pure-validator runtime module that asserts every substrate write originating from the workflow-trace binaries carries the `workflow_trace_` namespace prefix, and documents the Observer read-deny convention that prevents the Watcher (synthex-v2 m46-m51) from poisoning the feedback loop. Single public constant + single assertion function + one typed error variant. m9 is structurally exclusive (`allowed_prefixes = ["workflow_trace_"]`); there is exactly one valid namespace prefix in workflow-trace's universe, and any deviation is a refusal-mode violation.

m9 does NOT reimplement stcortex's DB-layer refuse-write — that invariant is enforced at the SpacetimeDB reducer per [CONSUMER-ONBOARDING § refuse-write](../../../the-workflow-engine-vault/boilerplate%20modules/02-stcortex-consumer/). m9 surfaces the assertion at the **application layer** immediately before the reducer call, so the failure mode appears as a typed `WorkflowError::NamespaceViolation` with a human-readable `tracing::error!` at the call site — not as an opaque SpacetimeDB 530 HTTP error several stack frames downstream. This is defense-in-depth, not duplication: the DB layer remains the substrate-grain backstop; m9 is the application-grain validator.

m9 is the **co-owner of Gap 3** (Unified destructiveness / EscapeSurfaceProfile schema — shared with m30 and m32 per [CLAUDE.md § structural-gap authorship](../../../CLAUDE.md)). m9 specifically validates the **namespace dimension** of the unified schema: any EscapeSurfaceProfile that involves a write target must carry a validated namespace, and m9 is the only producer of `ValidatedNamespace` evidence. m30 and m32 own the EscapeSeverity ordinal and trap-surface dimensions respectively; m9 owns the namespace dimension. The three together are the unified destructiveness schema.

---

## 2. Contracts (CC-2 primary)

| Surface | Direction | Detail |
|---|---|---|
| **CC-2 Trust Layer Woven** (PRIMARY) | OUT → m13, m42, any future writer | Called immediately before stcortex/POVM/injection.db write attempts. |
| **Gap 3 — Unified destructiveness / EscapeSurfaceProfile** (co-owner; namespace dimension) | OUT → m30, m32 | m9 produces `ValidatedNamespace` newtype evidence; m30 bank schema and m32 dispatch surface consume it. |
| m13 write request | IN | `StcortexWriteRequest { namespace, payload, … }` passes through `assert_workflow_trace_namespace(&namespace)` before the reducer call. |
| m42 reinforce request | IN | `StcortexEmitRequest { namespace, pathway, … }` passes through the same gate before the substrate-feedback emit. |
| `WORKFLOW_TRACE_NS_PREFIX` constant | OUT | Single source of truth for the prefix string; AP30 mitigation (no literal string usage downstream). |
| `tracing::error!` structured event | OUT (side-effect) | Emitted on violation with `namespace` + `expected_prefix` fields. |

**Observer read-deny convention:** documented in m9's module docstring. The Watcher MAY read `workflow_trace_*` namespace via SQL (Observer role per [synthex-v2/obsidian-synthex-v2/synthex-v2/The Watcher.md](../../../synthex-v2/obsidian-synthex-v2/synthex-v2/The%20Watcher.md)) but MUST NOT write. m9 has no runtime enforcement on reads — reads cannot be gated at the application-layer validator — but it is the authoritative documentation site for the architectural convention. The enforcement is architectural (Watcher R13 scope discipline + AP27 self-mod boundary). m9 cites F8 (Watcher feedback-loop poisoning) as the failure mode the convention prevents.

### Capability table by `EscapeSurfaceProfile` (7-variant; D-S1002127-02)

m9's namespace-write capability gate by `EscapeSurfaceProfile`. Default policy is permissive for low-stakes surfaces and DENY-with-acknowledgement for capability-gain and destructive surfaces:

| Variant | Ordinal | m9 namespace-write gate | Required `HumanAcceptanceSignature` field |
|---|---:|---|---|
| `Sandboxed` | 0 | allow (no signature beyond default `interactive_terminal=true`) | — |
| `SandboxEscape` | 10 | allow + extra logging | — |
| `ProcessMutate` | 20 | allow with operator confirm | — |
| **`PrivilegeEscalation`** | **30** | **DENY by default; only allowed for Conductor-routed dispatches with explicit `HumanAcceptanceSignature.privilege_escalation_acknowledged = true`** | **`privilege_escalation_acknowledged: bool`** (must be `true`; absent or `false` returns `NamespaceViolation::PrivilegeEscalationNotAcknowledged`) |
| `FileWrite` | 40 | allow with operator confirm | — |
| `NetworkEgress` | 50 | allow with operator confirm + audit | — |
| `DataExfil` | 60 | DENY by default; operator override required + audit | `data_exfil_acknowledged: bool` (must be `true`) |

`PrivilegeEscalation` is the only NEW capability-gate row introduced by D-S1002127-02; the other rows preserve their existing policy. The `HumanAcceptanceSignature` struct (private constructor in m30) carries the `privilege_escalation_acknowledged` flag; m9 reads it via the same code path that already reads `data_exfil_acknowledged`. m9 itself does not construct the signature — it validates the field at the namespace-write boundary.

m9 has **no upstream aspect**. It is a pure synchronous validator with no I/O.

---

## 3. Public surface

```rust
// src/m9_watcher_namespace_guard/mod.rs
pub mod validator;
pub mod evidence;
pub mod error;

pub use validator::{assert_workflow_trace_namespace, munge_hyphen_slug};
pub use evidence::ValidatedNamespace;
pub use error::NamespaceViolation;

/// Single source of truth for the workflow-trace stcortex namespace prefix.
///
/// AP30 collision avoidance: `workflow_trace_*` is reserved; it does not
/// collide with `orac_*`, `pane_vortex_*`, `synthex_v2_*`, `lcm_*`, `me_*`,
/// or any other registered habitat namespace per the stcortex consumer registry.
pub const WORKFLOW_TRACE_NS_PREFIX: &str = "workflow_trace";
```

Newtype evidence (`ValidatedNamespace`) carries proof that a string has passed the prefix check; downstream writers accept `ValidatedNamespace` not `&str`. This is the AP30 compile-time mitigation: any code path constructing a stcortex write call without first calling the validator fails to type-check at the writer-call boundary (m13 / m42 signatures take `ValidatedNamespace`).

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValidatedNamespace(String);

impl ValidatedNamespace {
    pub fn as_str(&self) -> &str { &self.0 }
}

impl std::fmt::Display for ValidatedNamespace { /* … */ }
```

---

## 4. Errors

```rust
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum NamespaceViolation {
    #[error("stcortex write blocked: namespace '{namespace}' does not start with '{expected_prefix}_' — workflow-trace must not write to other services' namespaces")]
    WrongPrefix { namespace: String, expected_prefix: &'static str },

    #[error("stcortex write blocked: namespace is empty")]
    Empty,

    #[error("stcortex write blocked: namespace '{namespace}' contains whitespace; expected hyphen-slug munge to underscore form")]
    Whitespace { namespace: String },

    #[error("stcortex write blocked: 'scratch' namespace forbidden for workflow-trace (use workflow_trace_scratch or a domain prefix)")]
    ScratchForbidden,
}
```

Hoisted into `WorkflowError::NamespaceViolation(NamespaceViolation)` per [m02_error_taxonomy § E3xxx](../../../the-workflow-engine-vault/deployment%20framework/phase-1-genesis-day-0-3.md). Error-band assignment: `WrongPrefix = E3101`, `Empty = E3102`, `Whitespace = E3103`, `ScratchForbidden = E3104`.

---

## 5. Implementation sketch

```rust
//! # m9_watcher_namespace_guard
//!
//! - **Layer**: L4 (Trust aspect-layer, Cluster D)
//! - **Deps**: workflow_core::errors::WorkflowError
//! - **Tests**: unit (prefix arms) + integration (m13 + m42 call boundaries)
//! - **Features**: none (aspect-layer invariant; not feature-gated)
//! - **Platform**: any
//! - **Impl Notes**: pure validator; no I/O; tracing::error! on violation;
//!                   hyphen-slug munge applied exactly once at boundary (AP-Hab-11)
//! - **Related Docs**: [cluster-D spec](../../../the-workflow-engine-vault/module%20specs/cluster-D-trust-cross-cutting.md) § m9
//!                     · [cluster-D plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-D.md) § m9
//!                     · [CONSUMER-ONBOARDING.md] — refuse-write reducer (DB-layer backstop)

pub fn assert_workflow_trace_namespace(namespace: &str) -> Result<ValidatedNamespace, NamespaceViolation> {
    if namespace.is_empty() {
        tracing::error!(target = "m9.validator", namespace = %namespace, "namespace empty");
        return Err(NamespaceViolation::Empty);
    }
    if namespace.chars().any(char::is_whitespace) {
        tracing::error!(target = "m9.validator", namespace = %namespace, "namespace whitespace");
        return Err(NamespaceViolation::Whitespace { namespace: namespace.to_owned() });
    }
    if namespace == "scratch" {
        tracing::error!(target = "m9.validator", namespace = %namespace, "scratch namespace forbidden");
        return Err(NamespaceViolation::ScratchForbidden);
    }
    let munged = munge_hyphen_slug(namespace);  // hyphens → underscores; AP-Hab-11 mitigation
    if !munged.starts_with(WORKFLOW_TRACE_NS_PREFIX) {
        tracing::error!(
            target = "m9.validator",
            namespace = %munged,
            expected_prefix = %WORKFLOW_TRACE_NS_PREFIX,
            "stcortex write blocked: AP30 collision avoidance — workflow-trace must not write to other services' namespaces"
        );
        return Err(NamespaceViolation::WrongPrefix {
            namespace: munged,
            expected_prefix: WORKFLOW_TRACE_NS_PREFIX,
        });
    }
    Ok(ValidatedNamespace(munged))
}

/// Convert hyphens to underscores per stcortex slug convention (S1001757 munge bug).
/// Called exactly once at the m9 boundary; downstream writers operate on the munged form.
pub fn munge_hyphen_slug(input: &str) -> String { input.replace('-', "_") }
```

The function is pure: no allocations on the happy path beyond the munge result, single `tracing::error!` on violation, returns `Result`. Per [GOLD_STANDARDS rule 7](../../../GOLD_STANDARDS.md), tracing uses structured key-value fields, never string interpolation in the format string. The call site (m13 / m42) wraps the validator call inline; no async, no spawn, no I/O.

---

## 6. Test plan (50 tests, mutation ≥70%)

Per [TEST_DISCIPLINE matrix row m9](../../../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md) and [cluster-D plan § m9 test-pattern allocation](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-D.md):

| Pattern | Count | Examples |
|---|---:|---|
| **F-Unit** | 25 | `workflow_trace_correlations` → Ok; `workflow_trace_battern_runs_2026` → Ok; `workflow_trace` (prefix only) → Ok; `orac_learn` → `WrongPrefix`; `pane_vortex_pulse` → `WrongPrefix`; `""` → `Empty`; `"scratch"` → `ScratchForbidden`; `"workflow trace x"` → `Whitespace`; `"workflow-trace-x"` → munged to `workflow_trace_x` → Ok; idempotent munge (`munge(munge(x)) == munge(x)`); each error carries correct `namespace` field; each error carries correct `expected_prefix` field; `ValidatedNamespace::as_str` round-trips; `Display` impl produces munged form. |
| **F-Property** | 5 | `for_all ns: &str, ns.starts_with("workflow_trace") iff validate(ns).is_ok()` (modulo munge); munge idempotent (`munge(munge(x)) == munge(x)`); `ValidatedNamespace::as_str` round-trip; whitespace rejection is closed under any whitespace char; emptiness rejection is exclusive on length 0. |
| **F-Integration** | 15 | m13 → m9 write boundary (mock stcortex client); m42 → m9 write boundary; concurrent write attempts from both writers; refuse-pass through both consumers (rejection at one does not block the other's separate call); cross-substrate boundary (stcortex vs POVM-decoupled — m9 still validates m42 even though m42 routes stcortex-only post-2026-05-17 ADR). |
| **F-Contract** | 3 | `NamespaceViolation` error shape stability (Debug + Display snapshot); `ValidatedNamespace` Display stability; tracing event shape (field names `namespace`, `expected_prefix`). |
| **F-Regression** | 2 | AP30 regression slot (literal `"workflow_trace_"` string anywhere outside `WORKFLOW_TRACE_NS_PREFIX` constant — grep test in CI); AP-Hab-11 hyphen-slug regression (S1001757 munge bug — `workflow-trace-foo` writes must land as `workflow_trace_foo` in stcortex). |
| **F-Mutation** | budget | ≥70% kill rate concentrated on `validator.rs` prefix-check and munge logic. |

The 50-test count matches the cluster-D spec's "15 tests for m9" target plus the ULTRAMAP View 2 row m9 minimum-50-per-module discipline. The cluster-D spec's smaller count reflected the function's thin surface; the matrix's 50 minimum forces additional edge-case coverage on munge + integration paths.

---

## 7. Boilerplate lift map

| Source | Lift % | Use |
|---|---:|---|
| `CONSUMER-ONBOARDING.md` (refuse-write principle) | reference only | Architectural anchor; documents DB-layer enforcement m9 mirrors |
| ME v2 `m1_foundation/logging.rs` | 30% (pattern) | tracing structured-field discipline |
| Newtype + `as_str` Rust idiom | std | `ValidatedNamespace` shape |
| Hyphen-slug munge | 0% (fresh) | ~5 LOC for `str::replace('-', "_")` wrap |
| Prefix check `str::starts_with` | std | no lift needed |

**Structural-gap LOC:** m9 is the namespace-dimension co-owner of Gap 3 (~20 LOC of validator + evidence-newtype contribute to the ~150-250 LOC unified destructiveness/EscapeSurfaceProfile schema authored across m9 + m30 + m32). m9 itself contributes the namespace validation surface; m30 contributes the EscapeSeverity ordinal; m32 contributes the trap-surface dispatch profile.

---

## 8. Failure modes addressed

| ID | Mode | How m9 addresses |
|---|---|---|
| **AP30** | Namespace string drift (literal `"workflow_trace_"` scattered through code) | Single `WORKFLOW_TRACE_NS_PREFIX` constant; downstream call-sites import the constant; CI grep guard ensures no literal usage outside m9. |
| **AP-Hab-11** | Hyphen-slug munge (S1001757) | `munge_hyphen_slug` applied exactly once at validator boundary; downstream operates on munged form. |
| **F3** | Substrate-input poisoning (write-side complement) | m9 is the write-side prefix discipline; m2/m3 (Cluster A) are the read-side complement. |
| **F8** | Watcher feedback-loop poisoning | Documented in module docstring as Observer read-deny convention; architectural enforcement only (Watcher R13 + AP27). |
| **AP-V7-09** | Substrate-frame engine confusion | Namespace prefix IS the substrate-frame boundary; mixing namespaces is a Class-G drift; m9 makes the boundary type-system-visible via `ValidatedNamespace`. |
| **W1** | Narrowed-scope consumer violation | m2 subscribes only to `workflow_trace_*`; m9 ensures the write side matches the read side. |

---

## 9. Observability

```rust
tracing::error!(
    target = "m9.validator",
    namespace = %namespace,
    expected_prefix = %WORKFLOW_TRACE_NS_PREFIX,
    "stcortex write blocked: AP30 collision avoidance"
);
```

No happy-path tracing emission — the validator is on the hot path of every write and should not generate log spam on success. Metric `m9_namespace_violations_total{kind}` counter exposed via [m05_metrics_collector](../../../the-workflow-engine-vault/deployment%20framework/phase-1-genesis-day-0-3.md) (kinds: `wrong_prefix`, `empty`, `whitespace`, `scratch`).

---

## 10. Pre-conditions / post-conditions

**Pre:** call-site has a `&str` candidate namespace. No I/O state, no async context required.

**Post:** Either (a) `Ok(ValidatedNamespace)` evidence is returned and the caller may proceed to the stcortex/POVM reducer with the munged namespace, or (b) `Err(NamespaceViolation)` is returned, the caller MUST NOT proceed to the write, and the tracing event has been emitted. The validator is pure beyond the tracing emission; calling it N times produces N tracing events on violation but N identical results.

---

## 11. Watcher class pre-positions

| Class | Triggers when |
|---|---|
| **Class A** (activation) | First `assert_workflow_trace_namespace(...)` call post-Genesis (Day 2 m2 consumer registration). |
| **Class D** (four-surface drift) | m9 passes a namespace but downstream surfaces (vault, ai_docs, CLAUDE.local.md) lack the corresponding anchor; namespace exists in stcortex but not in any documentation surface. |
| **Class F** (false-binding) | Hyphen-slug munge produces a valid prefix from an input that operator intent was non-workflow-trace (e.g. `workflow-trace-debug` from accidental copy-paste of another service's namespace munged into valid form). |

WCP notify on Class D or Class F → file drop to `~/projects/shared-context/watcher-notices/`.

---

## 12. Atuin trajectory anchor

Proposed atuin scripts (post-G9):
- `wt-namespace-audit` — greps stcortex namespaces for non-`workflow_trace_*` entries that should belong to workflow-trace (orphan detection).
- `stcortex-probe` (existing atuin script; m9 enforces what stcortex-probe surfaces).

History rows during normal authoring: `~/.local/bin/stcortex sql "SELECT name, namespace FROM consumer WHERE namespace LIKE 'workflow_trace_%'"`, `cargo test -- m9`. Queryable via `atuin search --workspace workflow-trace 'namespace'`.

---

## 13. Open questions

1. **Subprefix collision.** Future habitat services might register `workflow_trace_analytics` (legitimate sub-component) vs `workflow_traceback` (accidental prefix match). Should the validator require a trailing `_` separator (`workflow_trace_`) to be strict? Current spec accepts `workflow_trace` as prefix-only; tightening to require `_` separator is one-line change. Question for G7.
2. **Gap 3 schema unification.** m9 owns the namespace dimension; m30 and m32 own the EscapeSeverity / trap-surface dimensions. The schema is not yet authored as a unified type. Should it live in `workflow_core::schemas::EscapeSurfaceProfile`, or be assembled at the m32 dispatch boundary? Coordinate with Cluster G specs.
3. **Observer read-deny enforcement vs documentation-only.** Currently architectural-only. If F8 fires (Watcher writes to workflow_trace_*), we will know post-hoc via Class-F or post-hoc audit. Should m9 grow a runtime guard reading the writer's process identity? Question for G7 + Watcher.

---

> **Back to:** [`cluster-D/INDEX`](./) · [`ai_specs/INDEX`](../../INDEX.md) · [`MODULE_MATRIX`](../../MODULE_MATRIX.md) · vault [[cluster-D-trust-cross-cutting]] · [cluster-D plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-D.md) · [phase-1](../../../the-workflow-engine-vault/deployment%20framework/phase-1-genesis-day-0-3.md)
>
> **Sister modules (Cluster D):** [m8](m8_povm_build_prereq.md) · [m9](m9_watcher_namespace_guard.md) · [m10](m10_ember_ci_gate.md) · [m11](m11_fitness_weighted_decay.md)

*Spec authored 2026-05-17 (S1001982). HOLD-v2 active. No code, no Cargo, no scaffold until G1-G9 clear and Luke emits explicit start-coding signal.*
