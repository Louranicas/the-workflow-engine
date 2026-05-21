# Hardening Fleet W3 — Type-Design Portfolio (core-type refactors — needs node-0.A call)

> Back to: [HARDENING_FLEET_2026-05-21.md](HARDENING_FLEET_2026-05-21.md)
> Source: W3 type-design-analyzer review (2026-05-21). Overall verdict: **"strong, disciplined codebase."**

## What W3 applied directly (clear, bounded, low-risk wins)

1. `#[non_exhaustive]` on the public error enums (+ `RefusalReason`, `DispatchOutcome`) —
   additive, within-crate matches unaffected, future-proofs the `workflow_core` public surface.
2. **`WorkflowId`** (m14) — private field + `#[serde(transparent)]` + fallible `new()` running the
   m9 namespace gate. Closes a real hole: `pub String` let callers bypass m9 entirely.
3. **`MinSupport`** (m20) — private field + fallible `new()` enforcing `MIN_SUPPORT_FLOOR`; makes
   an illegal state unrepresentable and lets `mine_sequences` drop a runtime guard.
4. 5 comment-accuracy fixes (stale line-number / term references).

## Deferred — 6 core-domain-type refactors (your decision)

Each is a genuine improvement, but each refactors a **core state type** with multi-module
ripple and/or a real design tradeoff. Per CLAUDE.md ("ask before major refactor of core state
types") these are surfaced rather than applied unilaterally. None is a security defect — they
are robustness / illegal-state-prevention improvements. All "breaking" labels are *intra-crate
only* (no external consumers; binaries are stubs).

| # | Type | Gap | Recommended fix | Blast radius / tradeoff |
|---|------|-----|-----------------|--------------------------|
| 5 | `AcceptedWorkflow` (m30) | `pub weight: f64` documented `[0,1]` but settable to `NaN`, poisoning m31's ranking (m31's `sanitise()` exists only to defend this) | private `weight` + `weight()` accessor + `set_weight() -> Result`; or `#[non_exhaustive]` + "construct only via `CuratedBank::accept`" | m30 + m11 + m31 call sites + tests |
| 6 | `Pattern` (m20) | `canonical_hash` is derived in `new()` but all fields `pub` → `steps.push()` desyncs the hash that m21/m23 use as identity → silent determinism break (KEYSTONE) | private fields, immutable-after-construction, accessors | m20/m21/m23 + heavy test `.steps`/`.support` access |
| 7 | `WorkflowProposal` (m23) | 6 `pub` fields; a hand-built `WorkflowProposal{evidence_n:0,..}` defeats the F2 gate; mismatched `proposal_id` breaks m32 `self_dispatch_guard` | private fields + accessors, construct only via `build_proposal`; min: `#[non_exhaustive]` | m23 + m30 + m32 + tests |
| 8 | `NexusEvent.kind` (m40) | `kind: String` is really a closed set (`workflow.dispatched`/`.completed`/…); a typo is silent wire drift | `enum NexusEventKind` with `#[serde(rename)]` to keep the dotted wire form + a `custom` escape hatch | m40 + wire-format consideties; **tradeoff:** loses free-form `kind` |
| 9 | `BatternId`/`CascadeClusterId`/`ChainId` (m5/m4/m3) | `pub` tuple fields; opacity is a doc promise, not a type guarantee | private field + `as_str`/`as_i64` accessor, single construction path | per-type `.0` access; `StepToken`/`ChainId` low-risk to leave |
| 10 | `WorkflowRunRow` (m7) | `ended_at: Option` + `outcome: Option` independently optional but semantically coupled — illegal `(Some,None)`/`(None,Some)` representable; `outcome` stringly-typed though `Outcome` enum exists | replace both with `run_state: RunState { Open, Closed{ ended_at, outcome: Outcome } }` | m7 + m12 + DB-row mapping; **biggest ripple** |

## Recommendation

#5, #6, #7, #10 are genuine illegal-state-prevention wins (#6 is a KEYSTONE determinism
hazard). #8, #9 are lower-value / carry tradeoffs. Suggested: greenlight #5/#6/#7/#10 as a
focused "W3.1 core-type encapsulation" sub-wave; treat #8/#9 as optional. Awaiting node-0.A go.
