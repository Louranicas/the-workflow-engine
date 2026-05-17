---
title: substrate-couplings/ — substrate-substrate coupling decompositions
date: 2026-05-17
status: SPEC
session: S1002127
addresses: [NA-GAP-03, NA-GAP-09]
hold_v2_compliant: true
authority: Luke @ node 0.A — S1002127 "as per proposal" (Wave 4.B closeout)
---

# substrate-couplings/ — Substrate-Substrate Coupling Decompositions

> **Back to:** [`../INDEX.md`](../INDEX.md) · [`../MODULE_MATRIX.md`](../MODULE_MATRIX.md) · [`../../README.md`](../../README.md) · [`../../ARCHITECTURE.md`](../../ARCHITECTURE.md) · [`../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md) · sibling [`../synergies/`](../synergies/) · sibling [`../substrates/`](../substrates/) · sibling [`../cross-cutting/refusal-taxonomy.md`](../cross-cutting/refusal-taxonomy.md) · sibling [`../cross-cutting/substrate-drift.md`](../cross-cutting/substrate-drift.md)

## Purpose & relation to `synergies/`

`synergies/CC-*.md` enumerates the **engine-side** cross-cluster contracts — function-call graphs across modules, owned by engine code, verifiable via engine-internal tests. This directory enumerates the **substrate-side** coupling edges hidden inside those contracts — information channels between persistent substrate-state stores, owned by substrates (not the engine), verifiable only via cross-substrate observation.

The distinction is structural, not redundant. Per NA-GAP-03:

> Today the engine "verifies" CC-5 via Watcher Class-I monitoring on S-C (stcortex); if Class-I doesn't fire, the engine declares "loop healthy". But Class-I fires on absence of stcortex pathway-weight delta — it does NOT detect S-C → S-B coupling failure (e.g. injection.db TTL sweep deleting reinforced rows before next-session reads).

A single engine-side contract (CC-5) hides a multi-edge substrate cascade: `m32 → S-C stcortex → habitat-memory daemon → S-B injection.db → next-session m3 read`. The engine owns only the first edge. The remaining edges are **substrate-substrate** and require their own observability surface.

## What lives here

| File | Engine-side parent | Substrate edges decomposed | NA-GAP closure |
|---|---|---|---|
| [`CC-5-decomposed.md`](CC-5-decomposed.md) | [`../synergies/CC-5.md`](../synergies/CC-5.md) (SPECIAL DEPTH) | 5 edges (E1-E5): m32→S-C, m32→S-E→S-C, S-C→habitat-memory→S-B, m32→S-F→V3-partner, S-C→digest→S-G | NA-03, NA-09 (PRIMARY) |
| [`CC-4-decomposed.md`](CC-4-decomposed.md) | [`../synergies/CC-4.md`](../synergies/CC-4.md) | 3 edges (E1-E3): m32→S-D conductor wave, S-D→weaver/zen/enforcer panes, S-D refusal → m32 5-check | NA-03 (secondary) + AP-V7-13 enrichment |
| [`CC-7-decomposed.md`](CC-7-decomposed.md) | [`../synergies/CC-7.md`](../synergies/CC-7.md) | 4 edges (E1-E4): m15 pressure → S-G operator, S-G → spec amendment fanout, S-G → S-watcher Ember gate, S-G fatigue feedback → m12 reports | NA-03 (operator) + NA-05 (operator-as-substrate) |

CC-1, CC-1.subA, CC-2, CC-3, CC-6 are **engine-internal** (no substrate-substrate edges beyond the trivial atuin-read or SQLite-internal cases already documented in [`../substrates/`](../substrates/)). They are not decomposed here.

## Distinction from substrate dossiers

[`../substrates/{S-A..S-G,watcher}.md`](../substrates/) documents each substrate **as an actor**: its lifecycle, refusal modes, drift indicators, back-pressure signals. The dossier is per-substrate (one viewpoint).

[`CC-*-decomposed.md`](.) documents **edges between substrates** for a given engine-side contract. The file is per-contract (multiple substrates' viewpoints intersect).

Read order for a new Claude session studying CC-5: `synergies/CC-5.md` first (engine contract) → `substrate-couplings/CC-5-decomposed.md` (substrate cascade) → individual `substrates/{stcortex,injection_db}.md` dossiers for per-substrate state.

## Verification discipline

Each `CC-*-decomposed.md` declares **per-edge observability contracts**. The pattern:

| Field | Meaning |
|---|---|
| `Engine-observable` | YES / NO / PARTIAL — can engine-side tracing see the edge fire? |
| `Substrate-confirmable` | YES / NO — does the receiving substrate emit a confirmation? |
| `Verification surface` | Where to look (Watcher class, m12 report, atuin row, substrate-specific dashboard) |
| `Silent-failure shape` | What does this edge look like when it fails invisibly? |
| `Remediation hint` | Operator-actionable repair step |

Per NA-GAP-09, the engine should NOT collapse contract verification into a single Watcher class. Each edge gets its own surface.

## Substrate-confirmable receipts (NA-GAP-09)

Where possible, the decomposition specifies a **substrate-confirmable receipt**: a field the receiving substrate itself writes when it observes the edge fire. Examples:

- **CC-5 E1:** stcortex writes `cc5_closed_at` on N+1-dispatch reinforce of the same pathway.
- **CC-5 E3:** habitat-memory daemon writes `cc5_propagation_observed_at` when it detects an stcortex pathway-weight delta and reinforces the corresponding injection.db row.
- **CC-4 E1:** Conductor writes `wave_dispatch_received_at` on successful enforcement-pane handoff.
- **CC-7 E1:** S-G operator (via m12 acknowledgment) writes `pressure_acknowledged_at` when the operator opens a pressure-register row.

These receipts are **substrate-authored** and replace engine-side inference with substrate-confirmation. Pending substrate-side change requests are tracked in [`../../ai_docs/decisions/`](../../ai_docs/decisions/) ADRs.

## HOLD-v2 compliance

All files in this directory are markdown specs only. No `.rs` files, no `Cargo.toml`, no executable code. Receipt-field additions are **substrate-side change requests** captured in ADRs — they do NOT modify engine code pre-G9. The decomposition itself is planning-only and does not require G9 to fire.

## Closure tests (post-G9, indicative)

Each decomposed CC has a closure-test surface:

- `tests/integration/cc5_substrate_decomposition.rs` — drives a CC-5 dispatch, asserts edge E1 (stcortex delta) AND edge E3 (injection.db row pre-warm) AND edge E5 (digest fire) observable within their respective windows.
- `tests/integration/cc4_conductor_decomposition.rs` — drives a CC-4 dispatch, asserts E1 (Conductor wave receive) AND E2 (weaver/zen/enforcer pane registration) AND E3 (refusal-path correctness if enforcement disabled).
- `tests/integration/cc7_operator_decomposition.rs` — drives a CC-7 pressure-fire, asserts E1 (operator pressure-register open) AND E4 (operator fatigue feedback through m12 within 7d).

These tests are `#[ignore = "requires live substrates + operator presence"]` for PR-CI; they run in nightly + Wave-end + post-G9 acceptance gates.

---

> **Back to:** [`../INDEX.md`](../INDEX.md) · sibling [`../synergies/`](../synergies/) · sibling [`../substrates/`](../substrates/) · [`../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md) § NA-GAP-03 / NA-GAP-09 · canonical [`../../ai_docs/optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md`](../../ai_docs/optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md)

*Filed 2026-05-17 (S1002127 · Wave 4.B closeout) · Command via na-gap-analyst follow-up · planning-only · HOLD-v2 compliant.*
