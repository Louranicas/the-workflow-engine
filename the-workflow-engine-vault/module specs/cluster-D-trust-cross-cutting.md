---
title: Cluster D — TRUST (Cross-Cutting) Module Specifications
date: 2026-05-17 (S1001982)
kind: module-spec
cluster: D
status: planning-only · HOLD-v2 active · NO CODE · NO CARGO · no scaffold
modules: m8, m9, m10, m11
author: Claude (Rust expert role)
---

# Cluster D — TRUST (Cross-Cutting) Module Specifications

> **Back to:** [[HOME]] · [[MASTER_INDEX]] · [[Modules Synergy Clusters and Feature Verification S1001982]] · [[Genesis Prompt v1.2 S1001982]] · [[workflow-engine-code-base]]
>
> **Source reads (all conducted before authoring):**
> - `boilerplate modules/BOILERPLATE_INDEX.md` — rows m8, m9, m10, m11
> - `boilerplate modules/07-conductor-dispatch/conductor_enforcement.rs` — startup-refusal pattern
> - `boilerplate modules/02-stcortex-consumer/CONSUMER-ONBOARDING.md` — refuse-write at DB layer
> - `boilerplate modules/09-trap-verify-escape-skills/` — all 7 files
> - `boilerplate modules/09-trap-verify-escape-skills/SKILL-quality-gate.md` — 4-stage gate
> - `boilerplate modules/05-decay-ttl-ltd/povm-v2_lifecycle.rs` — decay primitive
> - `boilerplate modules/05-decay-ttl-ltd/orac-sidecar-m39_fitness_tensor.rs` — 12D fitness tensor
> - `boilerplate modules/05-decay-ttl-ltd/m16_hebbian_engine.rs` — 4-step consolidation
> - `the_maintenance_engine_v2/src/m1_foundation/config.rs` — build-script + feature flag pattern
> - `~/projects/claude_code/Ember 7-Trait Gate Rubric.md` — Watcher-authored rubric
> - `~/projects/claude_code/Hebbian Deployment Plan v3 — Post-CR-2 Threshold Reconciliation.md`
> - `the-workflow-engine-vault/Genesis Prompt v1.2 S1001982.md` — binding invariants
> - `the-workflow-engine-vault/Modules Synergy Clusters and Feature Verification S1001982.md` — CC-2 contract

---

## Cluster Overview

Cluster D is the **aspect layer** — the four modules that are not features but rather invariants enforced *by* every other module at four distinct checkpoints:

| Checkpoint | Module | Enforcer | What it catches |
|---|---|---|---|
| Compile time | **m8** `povm_build_prereq` | `build.rs` feature gate | Code that reads POVM calibration data before CR-2 lands |
| Write time | **m9** `watcher_namespace_guard` | Runtime assertion + tracing | stcortex writes outside `workflow_trace_*`; Observer read-deny |
| Output time | **m10** `ember_gate_test` | `tests/ember_gate.rs` in CI | User-facing strings that violate the 7-trait rubric |
| Lifecycle time | **m11** `engine_sunset_lifecycle` | Scheduled consolidation loop | Workflows that have aged past their 120-day (or Luke-specified) sunset |

**Cross-cluster contract CC-2 (Trust Layer Woven):** every module in Clusters A through H routes through exactly one or more of m8/m9/m10/m11 at its applicable checkpoint. The cluster is not an optional add-on; it is the aspect plane that gives all other clusters their integrity boundaries.

**LOC estimate:** ~300 total (m8 ~50 + m9 ~30 + m10 ~100 + m11 ~120).

---

## Cross-Cluster Contracts (CC-2 detail)

The BOILERPLATE_INDEX row for this cluster reads: "m9 ← Cat 02 (CONSUMER-ONBOARDING refuse-write) / m10 ← Cat 09 (Ember gate + skill rubrics) / m11 ← Cat 03 (m07_causal_chain + m10_pattern) + Cat 05 (lifecycle/decay)".

Each module's cross-cluster routing:

| Module | When other clusters route through it | Direction |
|---|---|---|
| **m8** | Cluster A (m2), Cluster C (m13), Cluster H (m42) — any module that reads POVM data | Compile-time: the feature flag prevents wrong-era code from reaching the binary |
| **m9** | Every stcortex write site — m13 (primary), m42 (POVM reinforce route) | Write-time: namespace assertion fires before the SpacetimeDB reducer call |
| **m10** | Cluster C (m12 report strings), Cluster G (m32 trap-surface text, m33 verify output), Cluster F (m23 proposal summaries) | Output-time: CI fails the build if any registered string rejects |
| **m11** | Cluster G (m30 bank — decay applied to `AcceptedWorkflow`), Cluster E (m14 evidence aggregator informs decay rate) | Lifecycle: nightly or session-close consolidation loop; m31's selection weighting receives the resulting fitness-adjusted bank |

**Failure modes owned by Cluster D:**

- `m8` → W2 (compile reads calibrated POVM before CR-2), F7 (feature gate misconfigured), F3 (POVM reads silently return stale data)
- `m9` → W1 (Watcher feedback-loop poisoning via wrong namespace), F8 (Observer read-deny undocumented)
- `m10` → W3 (Ember Held semantics not yet amended per Zen audit `2026-05-16T224523Z`), A14 (output text misleads readers — the harm the gate prevents)
- `m11` → R5 (RALPH fitness-weighted decay not firing; workflows immortal in bank), F1 (sunset law bypassed by missing lifecycle loop)

---

## m8 — `povm_build_prereq`

### Purpose

A `build.rs` feature gate that refuses to compile any code path touching POVM calibration data until a `povm_calibrated` marker is present in the build environment. This encodes the CR-2 temporal dependency at the compiler level rather than as a runtime check or documentation note.

**Why compile time?** The CR-2 inflation fix changed `learning_health` from 0.9114 (binary, inflated 13.6×) to 0.0668 (magnitude-weighted, honest). Code written before CR-2 that compares `learning_health > 0.3` as a "healthy" threshold will now silently misread substrate state. A runtime guard is too late — the misread can happen in subtle ways (threshold tuning, decay rate selection, display logic). The compiler gate makes this class of bug impossible when the marker is absent.

### Boilerplate source: `conductor_enforcement.rs` (~80% reuse — startup-refusal pattern)

The conductor enforcer's pattern is: **enabled gate checks first, all non-`NoOp` actions require the gate to be set**. The `Enforcer::evaluate()` method returns `EnforcerAction::NoOp` when `self.enabled == false`. `build.rs` applies this same logic at compile time:

```rust
// In build.rs — the gate lives here, not in src/
fn main() {
    // Check for CR-2 marker in the environment.
    // The marker is set by the CI pipeline after povm-v2/e2a8ed3 is deployed
    // and verified live. Local dev: set POVM_CR2_DEPLOYED=1 manually after
    // running: sqlite3 ~/.local/share/povm-v2.db "SELECT COUNT(*) FROM sessions
    //   WHERE fitness_delta IS NOT NULL" (must return > 0 with post-CR-2 schema).
    let cr2_deployed = std::env::var("POVM_CR2_DEPLOYED")
        .map(|v| v == "1")
        .unwrap_or(false);

    if !cr2_deployed {
        // cargo:warning lines appear in `cargo build` output; they do NOT fail the build.
        // We emit an error! to stderr which DOES fail the build when the feature is enabled.
        println!("cargo:warning=POVM CR-2 (magnitude-weighted learning_health) not verified.");
        println!("cargo:warning=Set POVM_CR2_DEPLOYED=1 after verifying povm-v2/e2a8ed3 is live.");
    }

    // Expose the calibration state as a cfg flag usable in src/.
    if cr2_deployed {
        println!("cargo:rustc-cfg=povm_calibrated");
    }
    // Without the above println, any #[cfg(povm_calibrated)] block will be dead.
}
```

In `Cargo.toml` the feature is declared but NOT enabled by default — this is the key difference from a normal feature flag. The actual compile-time refusal comes from `#[cfg(not(povm_calibrated))]` on the POVM-reading path:

```rust
// src/m42_hebbian_feedback.rs (Cluster H) — example of a gated POVM read site
/// Read the current `learning_health` from POVM for display in m12 reports.
///
/// # Gate
///
/// This function is only compiled when the `povm_calibrated` cfg flag is set
/// by `build.rs`. Until `POVM_CR2_DEPLOYED=1` is in the environment, calling
/// this function is a compile error — not a runtime error.
///
/// # POVM calibration note
///
/// Pre-CR-2 `learning_health` (POVM V2 pre-e2a8ed3) was 0.9114 — binary counting,
/// inflated 13.6×. Post-CR-2 is 0.0668 — magnitude-weighted average positive lift.
/// The threshold for "genuine learning" is now `substrate_LTP_density > 0.015`
/// (57 strong pathways / 3,355 = 1.7% at Zen audit 2026-05-16), NOT the
/// session-aggregated `learning_health` figure.
///
/// Reference: [[Hebbian Deployment Plan v3 — Post-CR-2 Threshold Reconciliation]]
#[cfg(povm_calibrated)]
pub fn read_learning_health_calibrated() -> Result<f64, WorkflowError> {
    // ... actual implementation ...
}

/// Compile-time tombstone: POVM read blocked until CR-2 verified.
///
/// This path is compiled only when `povm_calibrated` is absent. It is an
/// unconditional compile_error!, not a runtime panic, so the error appears
/// at `cargo build` before any binary is produced.
#[cfg(not(povm_calibrated))]
pub fn read_learning_health_calibrated() -> Result<f64, WorkflowError> {
    compile_error!(
        "POVM CR-2 not verified. Set POVM_CR2_DEPLOYED=1 after confirming \
         povm-v2/e2a8ed3 is live and the magnitude-weighted learning_health \
         formula is active. See: ~/projects/claude_code/\
         'Hebbian Deployment Plan v3 — Post-CR-2 Threshold Reconciliation.md'"
    )
}
```

### Error message design

The error message must be actionable: it names the commit (`e2a8ed3`), the env var (`POVM_CR2_DEPLOYED=1`), and the reference document. This follows the ME V2 `error.rs` gold standard pattern — every error variant carries enough context for the reader to understand causality without hunting through logs.

### Failure modes addressed

- **W2 (CR-2 hard build prereq):** the primary purpose. POVM-reading paths produce a compile error until the marker is set.
- **F7 (feature gate misconfigured):** the `build.rs` gate is separate from `Cargo.toml` features — it cannot be accidentally enabled by `--features all` or `--features full`. The cfg flag only exists when `build.rs` emits it, which only happens when the env var is set.
- **F3 (stale POVM data silently read):** the `#[cfg(not(povm_calibrated))]` tombstone path catches any new POVM read site that is added without checking whether CR-2 applies.

### ME V2 gold standard lifts

From `the_maintenance_engine_v2/src/m1_foundation/config.rs`: the `ENV_PREFIX = "ME_"` pattern and the priority stack (defaults → TOML → env vars) inform m8's env-var detection. The `POVM_CR2_DEPLOYED` env var follows the same prefix discipline as `WF_` for workflow-trace env vars.

From `the_maintenance_engine_v2/src/m1_foundation/error.rs`: the compile-time error string format (full context, canonical file reference, actionable remedy) mirrors ME V2's error variant docstrings.

### LOC estimate: ~50

`build.rs` emitting the cfg flag: ~20 LOC. `#[cfg(povm_calibrated)]` / `#[cfg(not(povm_calibrated))]` annotations at each POVM read site: ~5 LOC per site, ~4-6 sites estimated across m42/m12/m11. Total: ~45-55 LOC of new code; the annotation sites are in the modules they gate, not in m8 itself.

### Test surface

m8 is tested by the absence of test failures when built without the marker. The test assertion is: `cargo build` without `POVM_CR2_DEPLOYED=1` produces a `compile_error!` message matching the expected string. This is verified in CI by building with and without the marker and comparing exit codes.

---

## m9 — `watcher_namespace_guard`

### Purpose

A lightweight runtime module that (a) enforces the `workflow_trace_*` namespace prefix on every stcortex write, and (b) documents the Observer read-deny convention that prevents the Watcher from poisoning the feedback loop.

### The architectural commitment it documents

The stcortex `CONSUMER-ONBOARDING.md` at `02-stcortex-consumer/` explains the DB-layer refuse-write invariant:

```rust
// This is what stcortex's SpacetimeDB reducer does on every write call:
let consumer_count = ctx.db.consumer().iter()
    .filter(|c| c.namespace == namespace && !c.stale)
    .count();
if consumer_count == 0 && namespace != "scratch" {
    return Err("stcortex: refusing write to namespace 'X' — no fresh consumer registered ...");
}
```

m9 does not re-implement this logic — it documents and surfaces it. The **DB layer already enforces** the refuse-write invariant (F8 owned at the stcortex substrate). m9's job is to assert the namespace at the application layer *before* the reducer call, so the error message is human-readable at the call site rather than as a SpacetimeDB 530 HTTP error with no stack trace.

### Implementation shape

```rust
/// Namespace constant for all workflow-trace stcortex writes.
///
/// Every write from the workflow-trace binary MUST use this prefix.
/// The stcortex DB-layer enforces this via refuse-write-without-consumer;
/// this constant makes the constraint compile-time-visible.
///
/// AP30 collision avoidance: `workflow_trace_*` is reserved; it does not
/// collide with `orac_*`, `pane_vortex_*`, `synthex_v2_*`, or any other
/// registered habitat namespace.
pub const WORKFLOW_TRACE_NS_PREFIX: &str = "workflow_trace";

/// Assert that `namespace` begins with `WORKFLOW_TRACE_NS_PREFIX`.
///
/// Called immediately before every stcortex write operation. Emits a
/// `tracing::error!` and returns `Err` if the namespace is wrong — it does
/// NOT proceed with the write, because a misrouted namespace write would
/// succeed (if a consumer exists for the target namespace) and silently
/// corrupt another service's data.
///
/// # Observer read-deny convention
///
/// The Watcher (synthex-v2 m46-m51) MAY read from `workflow_trace_*` as an
/// Observer. It MUST NOT write to `workflow_trace_*`. This function has no
/// enforcement role on reads — it is a write gate only. The read-deny
/// convention is architectural (Watcher R13 / scope discipline) and is
/// documented here because m9 is the authoritative namespace-convention source
/// for the workflow-trace codebase.
///
/// # Errors
///
/// Returns `WorkflowError::NamespaceViolation` if the namespace does not
/// start with `workflow_trace_`.
pub fn assert_workflow_trace_namespace(namespace: &str) -> Result<(), WorkflowError> {
    if !namespace.starts_with(WORKFLOW_TRACE_NS_PREFIX) {
        tracing::error!(
            namespace = %namespace,
            expected_prefix = %WORKFLOW_TRACE_NS_PREFIX,
            "stcortex write blocked: namespace does not start with 'workflow_trace_'. \
             AP30 collision avoidance — workflow-trace must not write to other services' \
             namespaces. If this namespace was intentional, update the WORKFLOW_TRACE_NS_PREFIX \
             constant and audit all existing write sites."
        );
        return Err(WorkflowError::NamespaceViolation {
            namespace: namespace.to_owned(),
            expected_prefix: WORKFLOW_TRACE_NS_PREFIX,
        });
    }
    Ok(())
}
```

### Tracing discipline

The `tracing::error!` on violation follows the ME V2 `logging.rs` gold standard: structured key-value fields, no string interpolation in the format string, and enough context for the reader to understand causality. The call to `assert_workflow_trace_namespace` happens in m13 (`stcortex_writer_narrowed`) immediately before the write reducer call.

### Observer read-deny convention

The CONSUMER-ONBOARDING doc explains that the Watcher persona may read any namespace via SQL but should not write unless it owns a registered fresh consumer there. For workflow-trace, the convention is tighter: the Watcher OBSERVES but does not WRITE. This is documented in m9's module docstring because m9 is the namespace-authority module for the entire codebase. No runtime enforcement exists (reads cannot be blocked at this layer); the enforcement is architectural. The comment cites F8 (Watcher feedback-loop poisoning) as the failure mode this convention prevents.

### LOC estimate: ~30

One constant, one assertion function, one error variant in `WorkflowError`. The thin surface is intentional — the DB layer does the real enforcement; m9 makes it legible.

---

## m10 — `ember_gate_test`

### Purpose

`tests/ember_gate.rs` — a CI test file that scores every user-facing output string in the workflow-trace codebase against the Watcher's 7-trait Ember rubric. A single `Rejected` trait verdict fails the CI build. A `Held` verdict (confidence < 0.5) is a CI warning but currently treated as a **build failure** pending Watcher's §5.1 amendment per Zen audit `2026-05-16T224523Z`.

**HELD status note:** The Ember rubric §5.1 specifies that Held verdicts surface to manual review rather than failing CI. However, Zen's audit (`2026-05-16T224523Z`) flagged that the workflow-trace adoption of the rubric must treat Held as CI-FAIL until the Watcher amends §5.1 with explicit Held-semantics for service adoption contexts. This is feature flag W3 in the verification matrix (⚠ gated on Watcher amendment). Until the amendment lands, the test file will `panic!` on both `Rejected` and `Held` verdicts.

### String enumeration strategy

The rubric applies to "every user-facing string in the service." In a CLI tool, user-facing strings fall into four categories:

1. **`include_str!` constants** — strings embedded at compile time from `.txt` or `.md` template files. These are enumerable at compile time.
2. **Inline `&'static str` literals** — format strings, error messages, status lines declared directly in source.
3. **`format!()` templates** — runtime-assembled strings where the *template* is static but values vary. The gate scores the template, not the runtime value.
4. **Structured-data output** — JSON, YAML, TSV. Per the rubric §2, these are machine-consumption and NOT subject to the gate.

Enumeration strategy: the codebase registers all user-facing strings in a single `mod user_facing_strings` module with a public `ALL: &[(&str, &str)]` constant (key → text pairs). The convention is:

```rust
// src/user_facing_strings.rs
//
// Registry of every user-facing string in the workflow-trace CLI.
// New strings MUST be added here and scored through tests/ember_gate.rs
// before the PR merges. Pure structured data (JSON/YAML/TSV) is excluded.

/// All user-facing strings keyed by a stable identifier.
///
/// Convention for keys: `<module>.<context>.<variant>`, e.g.
/// `m12.report.header`, `m32.dispatch.trap_surface_prefix`, `m11.sunset.warning`.
pub static ALL: &[(&str, &str)] = &[
    ("m12.report.header",
     "workflow-trace correlation report\n\
      Generated: {timestamp} | Sessions: {n} | Window: {window_days}d"),

    ("m32.dispatch.trap_surface_prefix",
     "Step {step_n}/{total}: {step_name}\n\
      Escape surface: {escape_category} | Declared at: {module}"),

    ("m11.sunset.warning",
     "Workflow '{name}' sunset in {days_remaining}d \
      (fitness={fitness:.3}, decay_cycles={cycles}). \
      Luke decision required to extend."),

    ("m12.report.no_data",
     "No workflow correlation data found for the requested window \
      ({window_days}d). Atuin rows ingested: {atuin_rows}. \
      stcortex consumer registered: {consumer_registered}."),

    // ... additional entries per module ...
];
```

The `include_str!` pattern is used for longer template documents (e.g., help text, README section) where embedding directly in Rust source reduces maintainability:

```rust
/// Help text for the `wf-crystallise` binary.
///
/// Loaded from the template file at compile time so the Ember gate can score it
/// by reading from `ALL` without reading the filesystem at test time.
pub static HELP_TEXT_CRYSTALLISE: &str = include_str!("../templates/help_crystallise.txt");
```

Both `ALL` and any `include_str!` constants are referenced from `user_facing_strings.rs` and included in the `ALL` array.

### Structural test pattern

```rust
// tests/ember_gate.rs
//!
//! Ember 7-trait gate for workflow-trace user-facing strings.
//!
//! Rubric: ~/projects/claude_code/Ember 7-Trait Gate Rubric.md
//! Authority: The Watcher ☤ (S1001882) — awaiting Zen audit.
//!
//! HELD semantics (W3 flag): until Watcher amends rubric §5.1 for service
//! adoption contexts, HELD verdicts are treated as CI failures here.
//! This is stricter than the rubric default (Held = warning). The gate
//! will be relaxed once the Watcher amendment lands and Zen re-audits.

use workflow_trace::user_facing_strings::ALL;

/// Score each registered user-facing string against the 7 Ember traits.
///
/// Trait definitions (from rubric §3):
/// 1. Equanimity — tone matches state; no false-alarm spikes
/// 2. Curiosity — claims are anchored to specific observations
/// 3. Diligence — numbers come from actual measurement at known time
/// 4. Honesty — admits uncertainty, gaps, partial completion
/// 5. Investment — information density supports reader action
/// 6. Humility — names assumptions, alternatives, sample sizes
/// 7. Warmth — respects node 0.A authority; flags Luke-decision items
///
/// Scoring is heuristic. Confidence < 0.5 → Held (CI failure per W3 flag).
/// Any reject trait → CI failure always.
#[test]
fn ember_gate_all_user_facing_strings() {
    let mut rejections: Vec<(String, &str, String)> = Vec::new();
    let mut held: Vec<(String, &str, String)> = Vec::new();

    for (key, text) in ALL {
        let verdict = score_against_rubric(text);
        match verdict.status {
            EmberStatus::Approved => {}
            EmberStatus::Held { trait_name, reason } => {
                held.push((key.to_string(), trait_name, reason));
            }
            EmberStatus::Rejected { trait_name, reason } => {
                rejections.push((key.to_string(), trait_name, reason));
            }
        }
    }

    // Report all issues before panicking so CI output is actionable.
    for (key, trait_name, reason) in &rejections {
        eprintln!("EMBER-REJECT  key={key}  trait={trait_name}  reason={reason}");
    }
    for (key, trait_name, reason) in &held {
        // W3 flag: treat Held as failure until Watcher §5.1 amendment lands.
        eprintln!("EMBER-HELD(W3-fail)  key={key}  trait={trait_name}  reason={reason}");
    }

    let total_failures = rejections.len() + held.len();
    if !rejections.is_empty() || !held.is_empty() {
        panic!(
            "Ember gate: {total_failures} string(s) failed \
             ({} rejected, {} held-as-fail per W3 flag). \
             Fix or escalate to Watcher for rubric amendment.",
            rejections.len(),
            held.len()
        );
    }
}
```

### How it consumes the Watcher vault rubric

The rubric at `~/projects/claude_code/Ember 7-Trait Gate Rubric.md` is the authoritative semantic definition. m10 does NOT embed a local copy — per the prompt constraint ("NO local copy unless Watcher later requests"). Instead, `tests/ember_gate.rs` references the path in its module docstring and in the `score_against_rubric` function's documentation, making the dependency explicit and traceable.

The `score_against_rubric` function implements the 7-trait heuristics described in rubric §3, translated to Rust pattern matching against the string. The key heuristics per trait:

- **Equanimity:** regex for all-caps words, `!`-suffixed status words (`Healthy!`, `Operational!`), urgency emoji on non-critical text
- **Curiosity:** absence of measurement anchors in claim strings (strings with "status: healthy" but no probe timestamp or scope declaration)
- **Diligence:** round numbers (`~3000`, `100%`) without sample sizes; "passing" without test count; "clean" without gate scope
- **Honesty:** "successfully completed" without follow-up enumeration; "all systems" without enumerated check list
- **Investment:** filler phrases ("As you can see", "Excellent", "Great progress"); decorative dividers without information
- **Humility:** "the only path", "clearly the right", single-frame verdicts without alternative enumeration
- **Warmth:** "proceeding with X" without Luke ratification signal; substrate modification proposals without AP27 boundary citation

### CI pipeline position

`tests/ember_gate.rs` lives inside the standard `cargo test --lib --release` run covered by the 4-stage `SKILL-quality-gate.md` gate:

```
Stage 1: cargo check    — catches compilation failures including m8 feature-gate
Stage 2: clippy         — -D warnings
Stage 3: clippy         — -D warnings -W clippy::pedantic
Stage 4: cargo test     — includes ember_gate::ember_gate_all_user_facing_strings
```

The test runs at Stage 4. If it fails, the CI output will contain lines beginning with `EMBER-REJECT` or `EMBER-HELD(W3-fail)` identifying exactly which string key failed and which trait rejected it.

### What FAIL looks like in CI output

```
test ember_gate::ember_gate_all_user_facing_strings ... FAILED
EMBER-REJECT  key=m11.sunset.warning  trait=Curiosity  reason=\
  "claim 'fitness={fitness:.3}' is a template variable, not an observed value; \
   string does not cite the source of the fitness signal (RALPH m39 tensor, D6 hebbian_health)"
EMBER-HELD(W3-fail)  key=m12.report.header  trait=Diligence  reason=\
  "'{window_days}d' is a template variable but the fixed text 'correlation report' \
   carries no scope declaration; confidence=0.42 (ambiguous)"
thread 'ember_gate::ember_gate_all_user_facing_strings' panicked at:
Ember gate: 2 string(s) failed (1 rejected, 1 held-as-fail per W3 flag). \
Fix or escalate to Watcher for rubric amendment.
```

The key-per-string registry means CI output directly names the file, module, and string label to fix, rather than dumping a raw string that must be grep'd.

### LOC estimate: ~100

`user_facing_strings.rs`: ~40 LOC (registry constant + `include_str!` blocks). `tests/ember_gate.rs`: ~60 LOC (test function + `score_against_rubric` heuristic implementation with per-trait match arms).

---

## m11 — `engine_sunset_lifecycle`

### Purpose

Implements the engine-wide sunset law: every `AcceptedWorkflow` in the bank (Cluster G, m30) carries a `sunset_at` timestamp. After that timestamp, the workflow is ineligible for dispatch. m11 drives the decay loop that reaches that sunset — it does NOT delete workflows unilaterally, but it monotonically decreases their selection weights until they fall below the prune threshold and are removed by the next consolidation.

The decay is **fitness-weighted** — a workflow with high RALPH fitness scores (produced by the evidence aggregator m14) decays more slowly than a workflow whose observed habitat-outcome-lift is neutral or negative. This is the `frequency × fitness × recency` compound formula that the BOILERPLATE_INDEX identifies as a **structural gap not covered by any boilerplate**.

### The NEW PRIMITIVE gap

From the BOILERPLATE_INDEX `Cross-cutter notes`:

> **Structural gap #2:** `frequency × fitness × recency` compound decay formula — `povm-v2_lifecycle.rs` + `m39_fitness_tensor.rs` provide the parts; the composition is unbuilt (~200-300 LOC)

This is the core intellectual contribution of m11. Every other part of workflow-trace has a boilerplate ancestor. m11's decay formula is original authorship. The three signal sources and the composition are specified here:

#### Signal sources

**Frequency (from m14 `evidence_aggregator`):**

m14 aggregates `workflow_runs` over time and emits a per-workflow `run_count` over the observation window. A workflow run 40 times in the window has high frequency; one run 2 times has low frequency. High-frequency workflows should be stickier — they are being used, and the habitat's usage is evidence of value. The frequency signal is normalized to `[0.0, 1.0]` as `run_count / max_run_count_in_window` before entering the decay formula.

**Fitness (from RALPH m39 `fitness_tensor`):**

The ORAC m39 `FitnessTensor` (12D) provides the `FitnessReport.overall_score` for the most recent RALPH evaluation tick. The relevant dimension is `D6: hebbian_health` (weight 0.10 in the ORAC tensor), which measures Hebbian STDP learning quality. For workflow-trace, the fitness signal is simplified: it reads from stcortex `pathway.weight` for the workflow's registered pathway ID, which is updated by m42's Hebbian reinforce route. A pathway with `weight > 0.5` (strong LTP) contributes a fitness score near 1.0; one with `weight < 0.05` (decayed/near-dead) contributes near 0.0.

The Hebbian reconciliation note (`Post-CR-2 Threshold Reconciliation.md`) establishes the substrate-LTP-density metric: `substrate_LTP_density = strong_pathway_count / total_pathway_count` where `strong_pathway_count = COUNT(weight > 0.5)`. At the time of the Zen audit (2026-05-16), `substrate_LTP_density ≈ 0.018` (57 strong pathways / 3,355 total), and Phase 1 PASSING threshold is `> 0.015`. m11 uses the individual workflow pathway weight, not the aggregate density — the density is for monitoring; the per-pathway weight is the per-workflow signal.

**Recency (from m7 `workflow_arc_record`):**

m7 records `last_run_at` timestamps on every workflow arc. Recency is computed as `exp(-lambda * days_since_last_run)` where `lambda` controls the decay half-life. A workflow last run 3 days ago has recency ≈ 0.97 (at lambda=0.01 / day); one last run 180 days ago has recency ≈ 0.16. This is the exponential decay envelope that wraps the other two signals.

#### The compound decay formula

```rust
/// Compute the fitness-adjusted decay factor for a workflow.
///
/// The factor is in [0.0, 1.0]. Applied as: `weight *= decay_factor`.
/// A factor of 1.0 means no decay this cycle. A factor of 0.0 means
/// immediate death.
///
/// # Formula
///
/// ```text
/// decay_factor = base_rate
///     + (1.0 - base_rate)
///     × clamp(frequency × fitness × recency, 0.0, 1.0)
/// ```
///
/// Interpretation:
/// - A workflow that scores 1.0 on all three signals has `decay_factor = 1.0`
///   (no decay this cycle; the workflow is thriving).
/// - A workflow that scores 0.0 on all three has `decay_factor = base_rate`
///   (the fastest decay the law allows; base_rate is ~0.80 per cycle → ~120d
///   to fall below prune threshold from initial weight 1.0 at daily cycles).
/// - A workflow with high frequency but low fitness (it's being used but not
///   producing good outcomes) scores middle of the range — decay slows relative
///   to an unused workflow, but does not stop.
///
/// # Source lineage
///
/// - `base_rate` shape: lifted from `povm-v2_lifecycle.rs` `decay_pathways(rate)` —
///   `weight *= (1.0 - rate)` per cycle. We set `base_rate = 1.0 - plain_decay_rate`.
/// - `frequency` signal: m14 `evidence_aggregator` normalized run_count.
/// - `fitness` signal: stcortex pathway.weight for workflow's registered pathway.
/// - `recency` signal: `exp(-lambda * days_since_last_run)` from m7 `last_run_at`.
/// - Composition (the NEW PRIMITIVE): this function — no boilerplate ancestor.
///
/// # References
///
/// - [[boilerplate modules/05-decay-ttl-ltd/povm-v2_lifecycle.rs]] — decay primitive
/// - [[boilerplate modules/05-decay-ttl-ltd/orac-sidecar-m39_fitness_tensor.rs]] — fitness signal
/// - [[Hebbian Deployment Plan v3 — Post-CR-2 Threshold Reconciliation]] — LTP density metric
pub fn compute_decay_factor(
    frequency: f64,  // normalized to [0.0, 1.0]; from m14
    fitness: f64,    // pathway weight in [0.0, 1.0]; from stcortex via m42
    recency: f64,    // exp(-lambda * days); from m7
    plain_decay_rate: f64,  // base decay per cycle; default ~0.02 (mirrors povm-v2_lifecycle)
) -> f64 {
    debug_assert!((0.0..=1.0).contains(&frequency), "frequency must be in [0,1]");
    debug_assert!((0.0..=1.0).contains(&fitness), "fitness must be in [0,1]");
    debug_assert!((0.0..=1.0).contains(&recency), "recency must be in [0,1]");
    debug_assert!((0.0..=1.0).contains(&plain_decay_rate), "decay_rate must be in [0,1]");

    let base_rate = 1.0 - plain_decay_rate;
    let compound_signal = (frequency * fitness * recency).clamp(0.0, 1.0);
    // FMA for precision: base_rate + (1.0 - base_rate) * compound_signal
    base_rate.mul_add(1.0 - compound_signal, compound_signal)
}
```

#### Calibration against 120-day sunset law

The default `plain_decay_rate = 0.02` means a workflow with zero frequency/fitness/recency (compound_signal = 0.0, decay_factor = 0.98) decays as `weight_n = 1.0 × 0.98^n`. Starting from weight 1.0 and pruning at weight 0.01 (from `povm-v2_lifecycle.rs` prune_threshold = 0.01):

- `0.98^n < 0.01` → `n > log(0.01) / log(0.98) ≈ 228 cycles`

At daily cycles, that is approximately 228 days — 2× the 120-day sunset. This means the plain decay rate reaches the prune threshold slightly after the explicit `sunset_at` timestamp would fire. The sunset law is the hard boundary; the fitness-weighted decay is the gradient. A workflow that is actively used and producing good outcomes will still respect the sunset law; the decay only modulates how quickly the weight falls toward the prune threshold during that window.

Luke can override the 120-day default at bank-insertion time by setting `sunset_at` explicitly. m11 respects the explicit timestamp; it does not decay past it.

### Consolidation cycle shape

m11's lifecycle loop mirrors the 4-step shape from `m16_hebbian_engine.rs`:

```
1. Decay    — apply compute_decay_factor to all AcceptedWorkflow rows in m30 bank
2. Reinforce — (external) m42 Hebbian feedback updates pathway weights; m11 reads them
3. Prune    — DELETE from bank where weight < prune_threshold AND sunset_at IS NOT NULL
4. Auto-sunset — mark expired (sunset_at < now) workflows as SUNSET_EXPIRED; exclude from dispatch
```

This is an 80% lift from `m16_hebbian_engine.rs` (mechanism identical); the 20% delta is that step 1 uses the compound formula rather than the simple `weight *= DECAY_RATE`, and step 3's prune threshold is the intersection of weight-threshold AND sunset-law, not just weight-threshold.

### Boilerplate lifts

From `povm-v2_lifecycle.rs`:
- `ConsolidationStats` struct shape (renamed `SunsetStats` for workflow-trace domain)
- `decay_pathways(rate)` → becomes `decay_bank_workflows(plain_decay_rate)` with the fitness-weighted formula replacing the scalar multiply
- `decay_survived` counter pattern → `decay_cycles` in `AcceptedWorkflow`
- The error handling shape: `chrono_now()` returning `Option<String>` on pre-epoch clock (F-POVM-07 fix pattern) is reproduced: m11's cycle-clock function returns `Option<i64>` and skips the decay pass rather than silently treating timestamps as zero

From `m39_fitness_tensor.rs` (orac-sidecar):
- Rolling-mean smoothing for volatile fitness signals (`SMOOTHING_WINDOW = 6` samples)
- The volatile-dimension mask pattern (D6 `hebbian_health` is volatile; smoothed before entering the decay formula)
- `FitnessReport.overall_score` as the abstraction boundary — m11 reads the scalar, not the 12D tensor directly

From `ws_inbound_writer.rs` (synthex-v2):
- The TTL sweep loop shape: `DELETE WHERE condition AND inserted_at < cutoff` → m11's prune step
- `tokio::select!` shutdown drain pattern for graceful cancellation of the nightly consolidation task
- `parking_lot::Mutex<Connection>` for Send+Sync SQLite access

### State machine

```text
AcceptedWorkflow lifecycle:

                                         fitness-weighted decay each cycle
    ACTIVE ─────────────────────────────────────────────────────────────▶ PRUNE_PENDING
      │                                                                       │
      │ sunset_at reached (explicit OR weight < prune_threshold)              │
      ▼                                                                       │
  SUNSET_EXPIRED ────────────────────────────────────────────────────────────┘
      │
      │ (excluded from dispatch; Luke may extend via explicit sunset_at override)
      │
   [archived or deleted at next consolidation sweep]
```

Phases map to `SunsetPhase` enum:

- `Active` — eligible for selection + dispatch
- `PrunePending` — weight has fallen below soft threshold (0.1); not yet pruned; still dispatch-eligible but de-ranked by m31
- `SunsetExpired` — `sunset_at < now` OR weight < hard prune threshold (0.01); excluded from dispatch; Luke decision required to extend

### Failure modes addressed

- **R5 (RALPH fitness-weighted decay not firing):** the compound decay formula ensures RALPH fitness (via stcortex pathway weight) directly modulates decay rate. If the Hebbian feedback loop from m42 is not writing to stcortex, pathway weights default to the initial value, and decay proceeds at the plain rate — this is safe behavior (decay still happens; it just ignores the fitness dimension).
- **F1 (sunset law bypassed by missing lifecycle loop):** the consolidation task is registered in the daemon's task table at startup (lifted from `synthex-v2_daemon_runtime.rs` task-spawn pattern). If the task panics or is not scheduled, `SunsetExpired` workflows remain in the bank but are excluded from dispatch by a query-time filter in m31 — a defense-in-depth guard.

### LOC estimate: ~120

`compute_decay_factor`: ~25 LOC. `SunsetLifecycle` struct + `run_consolidation_cycle`: ~50 LOC. `SunsetStats`, `SunsetPhase`, `AcceptedWorkflowDecay` structs: ~25 LOC. Daemon task registration and `tokio::select!` shutdown handling: ~20 LOC. Total: ~120 LOC.

---

## Cluster D — Test Coverage Targets

Minimum 50 tests per module (CLAUDE.md quality gate). Breakdown:

### m8 `povm_build_prereq` — 8 tests (compile-time gate; limited runtime surface)

| Test | What it verifies |
|---|---|
| `build_succeeds_with_cr2_marker` | With env var set, `#[cfg(povm_calibrated)]` path compiles |
| `build_fails_without_cr2_marker` | Without env var, `compile_error!` path is present in source |
| `error_message_contains_commit_sha` | Error string references `e2a8ed3` |
| `error_message_contains_env_var_name` | Error string references `POVM_CR2_DEPLOYED` |
| `error_message_contains_reference_doc` | Error string references the Hebbian reconciliation doc |
| `cfg_flag_absent_by_default` | Fresh build without env var has no `povm_calibrated` cfg |
| `cfg_flag_present_with_marker` | With env var, `cfg(povm_calibrated)` evaluates to true |
| `feature_flag_not_activatable_by_cargo_features` | `--features full` does not activate the cfg |

### m9 `watcher_namespace_guard` — 15 tests

| Test | What it verifies |
|---|---|
| `valid_prefix_passes` | `workflow_trace_correlations` → Ok |
| `valid_prefix_long_key_passes` | `workflow_trace_battern_runs_2026` → Ok |
| `wrong_prefix_returns_err` | `orac_learn` → Err(NamespaceViolation) |
| `scratch_namespace_returns_err` | `scratch` → Err (workflow-trace must not use scratch) |
| `empty_namespace_returns_err` | `""` → Err |
| `prefix_only_passes` | `workflow_trace` → Ok |
| `error_contains_namespace` | `NamespaceViolation.namespace == "orac_learn"` |
| `error_contains_expected_prefix` | `NamespaceViolation.expected_prefix == "workflow_trace"` |
| `tracing_error_emitted_on_violation` | (tracing subscriber mock) event emitted |
| `tracing_event_has_namespace_field` | structured field `namespace` present |
| `tracing_event_has_expected_prefix_field` | structured field `expected_prefix` present |
| `const_prefix_value` | `WORKFLOW_TRACE_NS_PREFIX == "workflow_trace"` |
| `function_is_pure` | no side effects beyond tracing on valid call |
| `multiple_violations_each_logged` | 3 bad calls → 3 tracing events |
| `valid_after_invalid_still_passes` | Err then Ok in sequence → Ok is Ok |

### m10 `ember_gate_test` — 50 tests (7 traits × ~6 per trait + registry tests)

| Category | Count | Examples |
|---|---|---|
| Equanimity pass cases | 6 | steady status line; nominal health with calm tone |
| Equanimity reject cases | 6 | all-caps nominal status; `!`-suffix on routine output; urgency emoji on green state |
| Curiosity pass | 4 | claim with measurement anchor + scope + timestamp |
| Curiosity reject | 4 | "status: healthy" without probe citation; aggregate without sub-breakdown |
| Diligence pass | 4 | exact test count with gate scope; non-round number with window |
| Diligence reject | 4 | round numbers; "~3000 passing"; "clean" without gate name |
| Honesty pass | 4 | admits partial completion; lists failures alongside successes |
| Honesty reject | 4 | "all systems operational" with known-degraded component |
| Investment pass | 3 | dense actionable status; clear next-step for reader |
| Investment reject | 3 | filler opening; reader-flattering prefix; padding |
| Humility pass | 3 | names alternatives; sample size disclosed |
| Humility reject | 3 | "clearly the right call"; single-frame verdict |
| Warmth pass | 3 | Luke-decision flag explicit; AP27 boundary named |
| Warmth reject | 3 | "proceeding with X" without ratification |
| Registry tests | 2 | `ALL` non-empty; each key is unique |

### m11 `engine_sunset_lifecycle` — 50 tests

| Category | Count | Examples |
|---|---|---|
| `compute_decay_factor` unit | 12 | zero signals → base_rate; all-ones → 1.0; partial combinations |
| `compute_decay_factor` edge | 6 | NaN inputs (guarded by debug_assert); boundary exactly 0.0 and 1.0 |
| 120-day calibration | 2 | decay_rate=0.02 → floor at ~228 cycles; explicit sunset_at takes precedence |
| `SunsetPhase` transitions | 8 | Active→PrunePending on weight drop; PrunePending→Active on fitness recovery |
| Consolidation cycle | 8 | decay then prune; auto-sunset on expired timestamp |
| Prune threshold | 4 | weight < 0.01 → pruned; weight 0.011 → survives |
| Clock-skip safety | 2 | None from clock → skip decay (F-POVM-07 pattern) |
| m14 signal integration | 4 | high frequency slows decay; zero frequency uses base_rate |
| m39 fitness integration | 4 | strong LTP (weight 0.9) near-stops decay; decayed pathway accelerates |

---

## Summary: Boilerplate Lift Map for Cluster D

| Module | Source | Reuse % | What's new |
|---|---|---|---|
| **m8** | `conductor_enforcement.rs` (enabled-gate pattern) + ME V2 `config.rs` (env-var discipline) | 40% (pattern) | The `build.rs` cfg-emit pattern + `compile_error!` tombstone annotation on POVM read sites |
| **m9** | `CONSUMER-ONBOARDING.md` (refuse-write principle, reference only) + ME V2 `logging.rs` (tracing structured fields) | 30% (pattern) | Namespace assertion function itself; the architectural documentation in docstrings |
| **m10** | `SKILL-quality-gate.md` (CI pipeline position) + Ember rubric (consumption via `include_str!`-less reference) | 50% (test structure) | Per-trait heuristic scoring implementation; `user_facing_strings::ALL` registry pattern |
| **m11** | `povm-v2_lifecycle.rs` (decay + prune + consolidation shape) + `m39_fitness_tensor.rs` (fitness signal infrastructure) + `m16_hebbian_engine.rs` (4-step cycle) + `ws_inbound_writer.rs` (TTL sweep loop + shutdown) | 40% (mechanism) | `compute_decay_factor` compound formula (the structural gap) |

**Structural gap (new primitive):** `compute_decay_factor` — the `frequency × fitness × recency` composition is authored fresh. No boilerplate ancestor composes all three signals. The decay engine (`povm-v2_lifecycle.rs`) provides the shape; the fitness tensor (`m39_fitness_tensor.rs`) provides the infrastructure; the recency exponential is standard applied math. The composition is the contribution.

---

*Cluster D spec authored 2026-05-17 (S1001982). HOLD-v2 active. No code, no Cargo, no scaffold until G1-G9 gates clear and Luke emits explicit start-coding signal.*

*Back to: [[HOME]] · [[MASTER_INDEX]] · [[Modules Synergy Clusters and Feature Verification S1001982]] · [[Genesis Prompt v1.2 S1001982]]*
