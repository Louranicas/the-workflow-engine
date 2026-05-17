---
title: m10 — ember_ci_gate (output-time 7-trait CI gate)
module_id: m10
cluster: D — Trust (cross-cutting)
layer: L4
binary: wf-crystallise
feature_gate: [none]
verb_class: refuse
ship_first: true
gap_owner: []
status: SPEC · planning-only · HOLD-v2 · NO CODE · NO CARGO · BLOCKED on B4 (Ember §5.1 amendment)
loc_budget: 90
test_budget: 60
mutation_kill: 70
boilerplate_lift: 30
date: 2026-05-17 (S1001982)
authority: Luke @ node 0.A — AP24 gate (G9 "start coding workflow-trace")
binding_spec: Genesis Prompt v1.3 § 1, § 2
primary_contract: CC-2 (Trust Layer Woven — D → all)
amendment_dependency: Watcher Ember Rubric §5.1 Held-semantics (B4 blocker — pending)
decisions_applied: [D-C]
---

# m10 — `ember_ci_gate`

> **Back to:** [`cluster-D/INDEX`](./) · [`ai_specs/INDEX`](../../INDEX.md) · [`MODULE_MATRIX`](../../MODULE_MATRIX.md) · vault [[cluster-D-trust-cross-cutting]] · [cluster-D plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-D.md) · [phase-1](../../../the-workflow-engine-vault/deployment%20framework/phase-1-genesis-day-0-3.md) · [Genesis v1.3](../../../ai_docs/GENESIS_PROMPT_V1_3.md)
>
> **Sister modules (Cluster D):** [m8](m8_povm_build_prereq.md) · [m9](m9_watcher_namespace_guard.md) · [m10](m10_ember_ci_gate.md) · [m11](m11_fitness_weighted_decay.md)

---

## 1. Purpose

`tests/ember_gate.rs` + `src/user_facing_strings.rs` + `src/m10_ember_gate/` — a CI test file that scores every registered user-facing output string in the workflow-trace codebase against the Watcher's 7-trait Ember rubric (`~/projects/claude_code/Ember 7-Trait Gate Rubric.md`). A single `Rejected` trait verdict fails the CI build. A `Held` verdict (confidence < 0.5) is **also a build failure** under the W3 flag, pending the Watcher's §5.1 amendment per Zen audit `2026-05-16T224523Z`.

m10 is the **output-time aspect** of the trust regime. m8 catches POVM-misreading at compile time; m9 catches namespace drift at write time; m10 catches tone/honesty/diligence violations in user-facing output before the binary ships. The three together form the CC-2 trust weaving — compile / write / output checkpoints — that prevents the engine from corrupting humans, substrate, or itself with mis-calibrated emissions.

**HELD status note (W3 flag, hard-gated on B4):** The Ember rubric §5.1 specifies that Held verdicts surface to manual review rather than failing CI. Zen's audit `2026-05-16T224523Z` flagged that workflow-trace's adoption of the rubric must treat Held as CI-FAIL until the Watcher amends §5.1 with explicit Held-semantics for service adoption contexts. This is the W3 verification-matrix flag. Until the amendment lands, `tests/ember_gate.rs` panics on both `Rejected` and `Held` verdicts. **B4 is one of 6 critical-path blockers per [CLAUDE.md § 6 blockers](../../../CLAUDE.md).** Until B4 clears, m10 ships in W3-strict mode.

**Hybrid CI-FAIL + allowlist (DECIDED 2026-05-17, S1002127, Luke directive "best practice + make the decision") — Watcher amendment unblocked:** Strings that fail the 7-trait rubric default to **CI-FAIL** (build-time block). An explicit allowlist file (`tests/ember_held_approvals.tsv`, schema below) permits known-acceptable Held strings to pass with operator-visible `tracing::warn!` ("EMBER-HELD(allowlisted)"); `Rejected` verdicts can **never** be allowlisted. Each allowlist row requires a `HumanAcceptanceSignature` (the `approved_by` column is the signature anchor). Fail-loud preserves discipline; allowlist provides operator-controlled escape hatch with auditable signatures; ships W3-strict-default behaviour matching LUKE_ACTION_NEEDED v2. **B4 closed** — Watcher Ember §5.1 amendment unblocked; rubric §5.1 receives matching service-adoption clause from Watcher in next cycle.

---

## 2. Contracts (CC-2 primary)

| Surface | Direction | Detail |
|---|---|---|
| **CC-2 Trust Layer Woven** (PRIMARY) | OUT → m12, m32, m23, m11 (any string-emitter) | Every user-facing string passes the 7-trait gate before merge. |
| Ember 7-Trait Rubric (`~/projects/claude_code/Ember 7-Trait Gate Rubric.md`) | IN (canonical reference) | NOT embedded; cited by path in module docstring + scoring function. The rubric is authoritative semantic definition; m10 implements heuristic scoring per §3. |
| `user_facing_strings::ALL: &[(&str, &str)]` registry | IN | All user-facing strings keyed by `<module>.<context>.<variant>`. New strings MUST register here before the PR merges. |
| `tests/ember_gate.rs` panic-or-pass | OUT | CI gate result — propagates to `cargo test --lib --release` exit code, which propagates to the 4-stage quality gate. |
| `tests/ember_held_approvals.tsv` allowlist | IN | TSV file: `(artefact_key, approved_by, approved_at, expiry)`. Held verdicts can be allowlisted with explicit operator sign-off + expiry; post-amendment, allowlist remains but Held no longer requires it. |
| structured tracing event | OUT (side-effect at CI runtime) | `EMBER-REJECT` and `EMBER-HELD(W3-fail)` lines emitted with `key`, `trait`, `reason` fields. |

**Aspect-IN:** m8 (must compile) — m10's gate test file is a normal Rust integration test and inherits the m8 compile-time POVM gate.

---

## 3. Public surface

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
pub static ALL: &[(&str, &str)] = &[ /* … populated as modules ship … */ ];

/// Help text for the `wf-crystallise` binary.
///
/// Loaded from the template file at compile time so the Ember gate can score it
/// by reading from `ALL` without reading the filesystem at test time.
pub static HELP_TEXT_CRYSTALLISE: &str = include_str!("../templates/help_crystallise.txt");
```

```rust
// src/m10_ember_gate/mod.rs
pub mod rubric;
pub mod allowlist;
pub mod gate;
pub mod error;

pub use rubric::{score_against_rubric, EmberStatus, TraitName};
pub use allowlist::{HeldApproval, load_approvals, is_approved};
pub use gate::{GateVerdict, evaluate_string};
pub use error::EmberGateError;
```

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmberStatus {
    Approved,
    Held { trait_name: TraitName, reason: String, confidence: f64 },
    Rejected { trait_name: TraitName, reason: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraitName {
    Equanimity, Curiosity, Diligence, Honesty, Investment, Humility, Warmth,
}
```

---

## 4. Errors

```rust
#[derive(Debug, thiserror::Error)]
pub enum EmberGateError {
    #[error("ember gate failed: {rejected} string(s) rejected, {held} string(s) held-as-fail (W3 flag active)")]
    GateFailed { rejected: usize, held: usize },

    #[error("allowlist TSV at {path} unreadable: {source}")]
    AllowlistRead { path: String, #[source] source: std::io::Error },

    #[error("allowlist TSV at {path} malformed at line {line}: {reason}")]
    AllowlistParse { path: String, line: usize, reason: String },

    #[error("rubric reference missing at {path}: cannot score without canonical rubric")]
    RubricMissing { path: String },
}
```

Error-band assignment per [`ERROR_TAXONOMY.md` § E3xxx](../../INDEX.md): `GateFailed = E3201`, `AllowlistRead = E3202`, `AllowlistParse = E3203`, `RubricMissing = E3204`.

---

## 5. Implementation sketch

The CI test file (lifted from [cluster-D spec § m10 structural test pattern](../../../the-workflow-engine-vault/module%20specs/cluster-D-trust-cross-cutting.md)):

```rust
// tests/ember_gate.rs
//!
//! Ember 7-trait gate for workflow-trace user-facing strings.
//!
//! Rubric: ~/projects/claude_code/Ember 7-Trait Gate Rubric.md
//! Authority: The Watcher ☤ (S1001882) — Zen audit `2026-05-16T224523Z`.
//!
//! HELD semantics (W3 flag): until Watcher amends rubric §5.1 for service
//! adoption contexts, HELD verdicts are CI failures here.
//! This is stricter than the rubric default (Held = warning). The gate
//! will be relaxed once the Watcher amendment lands (B4) and Zen re-audits.

use workflow_trace::user_facing_strings::ALL;
use workflow_trace::m10_ember_gate::{score_against_rubric, EmberStatus, load_approvals, is_approved};

#[test]
fn ember_gate_all_user_facing_strings() {
    let approvals = load_approvals("tests/ember_held_approvals.tsv").unwrap_or_default();
    let mut rejections: Vec<(String, &str, String)> = Vec::new();
    let mut held: Vec<(String, &str, String)> = Vec::new();

    for (key, text) in ALL {
        match score_against_rubric(text) {
            EmberStatus::Approved => {}
            EmberStatus::Held { trait_name, reason, .. } => {
                if !is_approved(&approvals, key) {
                    held.push((key.to_string(), trait_name.as_str(), reason));
                }
            }
            EmberStatus::Rejected { trait_name, reason } => {
                // Rejected can NEVER be allowlisted — operator approval cannot override Rejected.
                rejections.push((key.to_string(), trait_name.as_str(), reason));
            }
        }
    }

    for (key, t, reason) in &rejections {
        eprintln!("EMBER-REJECT  key={key}  trait={t}  reason={reason}");
    }
    for (key, t, reason) in &held {
        eprintln!("EMBER-HELD(W3-fail)  key={key}  trait={t}  reason={reason}");
    }

    if !rejections.is_empty() || !held.is_empty() {
        panic!(
            "Ember gate: {} rejected, {} held-as-fail per W3 flag. \
             Fix the string or escalate to Watcher for rubric §5.1 amendment.",
            rejections.len(), held.len()
        );
    }
}
```

The 7-trait heuristic scoring lives in `src/m10_ember_gate/rubric.rs`. Per [cluster-D spec § m10 trait heuristics](../../../the-workflow-engine-vault/module%20specs/cluster-D-trust-cross-cutting.md):

- **Equanimity:** regex for all-caps words on routine status; `!`-suffixed status words on green state; urgency emoji on non-critical text.
- **Curiosity:** absence of measurement anchors in claim strings (e.g. `"status: healthy"` without probe timestamp or scope declaration).
- **Diligence:** round numbers (`~3000`, `100%`) without sample sizes; `"passing"` without test count; `"clean"` without gate scope.
- **Honesty:** `"successfully completed"` without follow-up enumeration; `"all systems"` without enumerated check list.
- **Investment:** filler phrases (`"As you can see"`, `"Excellent"`, `"Great progress"`); decorative dividers without information density.
- **Humility:** `"the only path"`, `"clearly the right"`, single-frame verdicts without alternative enumeration.
- **Warmth:** `"proceeding with X"` without Luke ratification signal; substrate modification proposals without AP27 boundary citation.

Each trait's scoring function returns `Option<(TraitName, String, f64)>` — `None` if the trait passes, `Some((trait, reason, confidence))` if not. Confidence < 0.5 → Held; ≥ 0.5 → Rejected. The aggregator (`score_against_rubric`) runs all 7 trait checks and returns the first failure encountered (deterministic ordering: Equanimity → Curiosity → Diligence → Honesty → Investment → Humility → Warmth).

`allowlist.rs` parses `tests/ember_held_approvals.tsv` per [RM v2 TSV reader pattern](../../../the-workflow-engine-vault/boilerplate%20modules/) with columns `artefact_key`, `approved_by`, `approved_at_iso8601`, `expiry_iso8601`. Expiry strict: `now > expiry → not approved`.

---

## 6. Test plan (60 tests, mutation ≥70%)

Per [TEST_DISCIPLINE matrix row m10](../../../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md) and [cluster-D plan § m10 test-pattern allocation](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-D.md):

| Pattern | Count | Examples |
|---|---:|---|
| **F-Unit (per-trait pass/reject)** | 30 | 6 traits × ~4-5 each — Equanimity pass (steady status, nominal health calm); Equanimity reject (all-caps nominal; `!`-suffix on routine; urgency emoji on green); Curiosity pass (claim with measurement anchor); Curiosity reject (`"status: healthy"` no probe citation); Diligence pass (exact test count + gate scope); Diligence reject (round numbers; `"~3000 passing"`); Honesty pass (admits partial completion); Honesty reject (`"all systems operational"` with known-degraded component); Investment pass (dense actionable); Investment reject (filler opener); Humility pass (names alternatives); Humility reject (`"clearly the right call"`); Warmth pass (Luke-decision explicit, AP27 cited); Warmth reject (`"proceeding with X"` without ratification). |
| **F-Property** | 5 | Rubric monotonic — more passing traits → not-worse gate verdict; allowlist idempotent under reapplication; expiry strict; `score_against_rubric` deterministic on identical input; trait-ordering stable. |
| **F-Integration** | 18 | Live rubric file read; live TSV read; gate end-to-end against fixture PR diff; gate against empty allowlist; gate against fully-allowlisted Held → pass; CI-fail propagation to exit code; Day 0/1/2/3 string registration end-to-end (Day 0 m02 error messages, Day 1 m11 sunset warning, Day 2 m1/m2/m3 error messages); allowlist row expiry rejection. |
| **F-Contract** | 5 | Rubric §5.1 schema vs implementation (snapshot); TSV schema snapshot; gate-result JSON shape; tracing event shape; the Ember §5.1 amendment text (pre-amendment hash captured to detect future drift). |
| **F-Regression** | 2 | Allowlist-bypass regression slot (`Rejected` never allowlistable); pedantic-mode panic-without-context regression. |
| **F-Mutation** | budget | ≥70% kill rate on `rubric.rs` per-trait scoring. |

The 60-test count reaches the cluster-D spec's 50-test target plus matrix-additional integration coverage. Note [GOLD_STANDARDS test discipline](../../../GOLD_STANDARDS.md) sets Cluster D at 250 tests min combined across m8/m9/m10/m11; m10's 60 + m11's 70 + m8's 50 + m9's 50 = 230, with property-test breadth on m11 (decay formula) bringing total above 250.

---

## 7. Boilerplate lift map

| Source | Lift % | Use |
|---|---:|---|
| `SKILL-quality-gate.md` | 90% (CI position) | 4-stage gate stage-4 integration |
| Ember Rubric `~/projects/claude_code/Ember 7-Trait Gate Rubric.md` | reference only | Canonical semantic definition; NOT embedded |
| RM v2 `read_tsv` helper | 70% (parser) | `allowlist.rs` TSV reader |
| ME v2 `m44_quality_gate/decision.rs` | 50% (decision shape) | `GateVerdict` and aggregation pattern |
| `chrono` `DateTime<Utc>` idiom | std | Expiry check |
| 7-trait heuristic implementation | 0% (fresh) | ~60 LOC of per-trait scoring functions |

**Structural-gap LOC:** none. m10 is a CI-integration scoring module; the rubric is canonical, the heuristics are implementation.

---

## 8. Failure modes addressed

| ID | Mode | How m10 addresses |
|---|---|---|
| **F8** | Watcher feedback-loop poisoning (trait level) | Refuses to merge PRs that fail Ember audit; prevents low-Ember strings entering substrate. |
| **B4** | Ember §5.1 amendment pending | Module ships in W3-strict mode (Held = CI-FAIL) until Watcher amends; documented in test file comment + module docstring; ratchet release on amendment land. |
| **AP-V7-06** | Bidi-anchor unidirectional rot | Heuristic can include bidi-anchor rot as Honesty trait fail-mode (claims `"Back to: [[X]]"` without verifying X exists). |
| **A14** | Output text misleads readers | The harm the gate prevents — every string is scored before reaching a user. |
| **AP-V7-13** | Diagnostics theatre | Heuristic catches `"Refreshed: DATE"` strings without paired evidence; flags under Diligence. |

---

## 9. Observability

CI runtime stderr emission (deterministic format):

```
EMBER-REJECT  key=m11.sunset.warning  trait=Curiosity  reason="claim 'fitness={fitness:.3}' is a template variable, not an observed value; string does not cite the source of the fitness signal (RALPH m39 tensor, D6 hebbian_health)"
EMBER-HELD(W3-fail)  key=m12.report.header  trait=Diligence  reason="'{window_days}d' is a template variable but the fixed text 'correlation report' carries no scope declaration; confidence=0.42 (ambiguous)"
```

Metric `m10_ember_verdicts_total{verdict, trait}` counter exposed via [m05_metrics_collector](../../../the-workflow-engine-vault/deployment%20framework/phase-1-genesis-day-0-3.md). Read by [`/sweep`](../../../CLAUDE.md) and Watcher.

---

## 10. Pre-conditions / post-conditions

**Pre:** `cargo test --lib --release` invocation (4-stage gate stage 4); `tests/ember_held_approvals.tsv` readable (may be empty); canonical rubric file referenceable (path verified, content not loaded — heuristic implementation is the test surface).

**Post:** Either (a) all strings in `ALL` produce `EmberStatus::Approved` (or `Held`-allowlisted with unexpired approval) and the test passes with exit code 0, or (b) one or more strings fail and the test panics with named keys + traits + reasons; exit code non-zero; 4-stage gate aborts; PR cannot merge.

---

## 11. Watcher class pre-positions

| Class | Triggers when |
|---|---|
| **Class A** (activation) | First CI run with m10 in `Fail` mode post-B4-amendment; first allowlist entry approval. |
| **Class C** (confidence-gate refusal — safe-path) | m10's CI-FAIL on `Held` is exactly Class-C: substrate refuses to ship low-confidence output. |
| **Class D** (four-surface drift) | Allowlist (`ember_held_approvals.tsv`) carries entries not mirrored in vault / CLAUDE.local.md / ai_docs — operator approval lost the audit trail. |
| **Class F** (false-binding) | Pre-B4 amendment: Held verdict rejects a string that, post-amendment, would have been acceptable for manual review. |

WCP notify on Class C (substrate-refusal) → file drop to `~/projects/shared-context/watcher-notices/`.

---

## 12. Atuin trajectory anchor

Proposed atuin scripts (post-G9):
- `wt-ember-gate` — CI invocation surface for m10 (`cargo test --lib --release -- ember_gate`).
- `ember-audit` (existing atuin script; m10 is its CI-gate counterpart).
- `wt-ember-allowlist-audit` — diffs `tests/ember_held_approvals.tsv` against vault sign-off notes; surfaces Class-D drift.

History rows during normal authoring: `cargo test -- ember_gate`, `vim tests/ember_held_approvals.tsv`, `~/.local/bin/watcher notify "ember §5.1 amendment query"`. Queryable via `atuin search --workspace workflow-trace 'ember'`.

---

## 13. Open questions

1. **B4 blocker — Watcher Ember §5.1 amendment.** Until the Watcher amends §5.1 for service adoption contexts, m10 ships W3-strict. **m10's behaviour materially changes when the amendment lands.** Phase-1 ships W3-strict; a post-amendment refit changes panic-on-Held to warn-on-Held + allowlist-permits-Held. Question for G7: is the W3-strict default the correct posture for Phase 1 deployment, or should the gate ship in warn-mode pending amendment with explicit risk acceptance? **This is the primary open question and the largest spec dependency for Cluster D.**
2. **Hybrid CI-FAIL + allowlist semantics post-amendment — DECIDED.** **DECIDED 2026-05-17 (S1002127, Luke directive "best practice + make the decision"):** Hybrid CI-FAIL + allowlist adopted as default. Rationale: fail-loud preserves discipline; allowlist provides operator-controlled escape hatch with auditable `HumanAcceptanceSignature` per row; ships W3-strict-default behaviour, matches LUKE_ACTION_NEEDED v2 proposal. Watcher amendment to Ember §5.1 unblocked; **B4 CLOSED** (D-C decision; allowlist defaults). Rejected verdicts remain non-allowlistable.
3. **Rubric heuristic accuracy.** The trait heuristics are pattern-matching approximations of semantic concepts. False-positive rate is unknown until first CI runs land real PRs. Question for G7: acceptable FP rate; should we instrument allowlist-entry-creation as a heuristic-correction feedback channel?
4. **`include_str!` template files vs inline strings.** Help text + long-form report templates live in `templates/*.txt` and are scored as single concatenated strings. Granularity question: should each paragraph be a separate `ALL` entry?
5. **PR-text-only scope.** Cluster-D plan says "PR-text only; non-PR-text modules are exempt." The classifier for PR-text vs internal isn't fully specified. Question for G7.

---

> **Back to:** [`cluster-D/INDEX`](./) · [`ai_specs/INDEX`](../../INDEX.md) · [`MODULE_MATRIX`](../../MODULE_MATRIX.md) · vault [[cluster-D-trust-cross-cutting]] · [cluster-D plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-D.md) · [phase-1](../../../the-workflow-engine-vault/deployment%20framework/phase-1-genesis-day-0-3.md)
>
> **Sister modules (Cluster D):** [m8](m8_povm_build_prereq.md) · [m9](m9_watcher_namespace_guard.md) · [m10](m10_ember_ci_gate.md) · [m11](m11_fitness_weighted_decay.md)

*Spec authored 2026-05-17 (S1001982). HOLD-v2 active. No code, no Cargo, no scaffold until G1-G9 clear and Luke emits explicit start-coding signal.*
