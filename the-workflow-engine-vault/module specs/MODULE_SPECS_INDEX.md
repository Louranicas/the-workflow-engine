---
title: Module Specs Index — 8 cluster specs / 26 modules
date: 2026-05-17 (S1001982)
kind: index
status: planning-only · detailed module design · HOLD-v2 active on build
authority: Luke @ node 0.A — "for each planned module plan in detail ... use agent view in parallel"
---

# Module Specs Index

> Back to: [[../HOME]] · [[../MASTER_INDEX]] · [[../workflow-engine-code-base]] · [[../CLAUDE.md]] · [[../CLAUDE.local.md]]

Detailed planning specs for all 26 modules of the single-phase `workflow-trace` codebase. Authored 2026-05-17 by 8 parallel `rust-pro` agents (one per cluster) over ~6 minutes. Each spec includes purpose, public surface (Rust code blocks), internal data structures, data flow, boilerplate lifts, ME v2 `m1_foundation` patterns referenced, constraint satisfaction, tests, open questions, and LOC estimates.

**Total: ~41,500 words across 8 spec documents.**

## Cluster spec catalogue

| Cluster | Modules | Spec file | Words | Highlight |
|---|---|---|---:|---|
| **A** Substrate Ingest | m1, m2, m3 | [[cluster-A-substrate-ingest]] | 5,883 | m1 cursor pagination = novel ~30 LOC (5-absence flag); m2 narrows stcortex to tool_call+consumption with reducer-callback dedup |
| **B** Habitat Observers | m4, m5, m6 | [[cluster-B-habitat-observers]] | 6,471 | F11 opaque-cluster-id via FNV-1a XOR; m5 unlabelled batterns preserved (no shape coercion); m6 20-session EMA explicitly excludes Converged outcomes |
| **C** Correlation + Output | m7, m12, m13 | [[cluster-C-correlation-output]] | 4,068 | m7 JSONB consumer_inputs (no internal join — CC-1 contract); m13 substrate_LTP_density backpressure 0.15 threshold; deferred-write JSONL buffer |
| **D** Trust (cross-cutting) | m8, m9, m10, m11 | [[cluster-D-trust-cross-cutting]] | 5,835 | m8 `cargo:rustc-cfg=povm_calibrated` + `compile_error!`; m11 `freq × fitness × recency` formula (3-signal composite — NEW PRIMITIVE) |
| **E** Evidence + Pressure | m14, m15 | [[cluster-E-evidence-pressure]] | 3,564 | m14 Wilson-CI lift (returns None < n=20); m15 JSONL one-event-per-file PHASE-B-RESERVATION-NOTICE |
| **F** Iteration (KEYSTONE) | m20, m21, m22, m23 | [[cluster-F-iteration]] | 6,135 | PrefixSpan over Apriori/n-gram; Wilson CI; normalized Levenshtein (<0.25 same / 0.25-0.60 near-miss); top-K-by-distance N=3 variants |
| **G** Bank/Select/Dispatch/Verify | m30, m31, m32, m33 | [[cluster-G-bank-select-dispatch-verify]] | 5,033 | m30 EscapeSurfaceProfile ordinal enum; m31 composite α/β/γ/δ + diversity gate; m32 5-check pre-dispatch sequence; m33 4-agent gate + 7-day TTL |
| **H** Substrate Feedback | m40, m41, m42 | [[cluster-H-substrate-feedback]] | 4,519 | m40 Option A untyped JSON for MVP; m41 `lcm.loop.create` not hypothetical lcm.deploy; m42 fitness_delta `PassVerified=+0.25, Pass=+0.15, Blocked=-0.05, Fail=-0.10` |
| **Total** | **26 modules** | 8 specs | **41,508** | |

## Cross-cluster synergy contracts (verified across specs)

| Contract | Path | Status across specs |
|---|---|---|
| **CC-1** Cascade-Cost Coupling | m4 ↔ m6 via m7 JSONB consumer_inputs | ✅ A+B+C aligned (B uses m7 join-as-stable-contract; C exposes JSONB schema) |
| **CC-2** Trust Layer Woven | D → all | ✅ D specifies cross-cutting; A/B/C reference m8/m9 trust hooks |
| **CC-3** Evidence-Driven Iteration | E → F | ✅ E14 contributes to F's iterator inputs; m20-m22 read lift metric |
| **CC-4** Proposal→Bank→Dispatch | F → G → Conductor | ✅ F23 outputs to G30; G32 wires Conductor |
| **CC-5** Substrate Learning Loop | G → H → back to F | ✅ G32 fan-outs to H40/H41/H42; m31 reads back stcortex pathway weights |
| **CC-6** Verification-Gated Dispatch | G internal m33→m32 | ✅ G32 5-check sequence enforces |
| **CC-7** Pressure-Driven Evolution | E → spec interviews | ✅ E15 PHASE-B-RESERVATION-NOTICE emit |

## Structural gaps owned (from Boilerplate Hunt; addressed in specs)

| Gap | Owner cluster + module | Spec addresses |
|---|---|---|
| **Gap 1** N-step compositional sub-graph detection | F (m20 + m23) | F: PrefixSpan algorithm + sketch + Levenshtein similarity + gradient preservation |
| **Gap 2** `frequency × fitness × recency` compound decay | D (m11) | D: composite formula `base_rate + (1.0 - base_rate) × clamp(frequency × fitness × recency, 0.0, 1.0)` |
| **Gap 3** Unified destructiveness / escape-surface schema | G (m30 + m32) + D (m9) | G: `EscapeSurfaceProfile` ordinal enum + m32 display-before-step + m9 namespace guard |

## ME v2 m1_foundation patterns referenced (across all specs)

| ME v2 file | Pattern lifted | Used in clusters |
|---|---|---|
| `resources.rs` | `//!` docstring style (Layer/Deps/Tests/Features/Platform/Impl Notes/Related Docs) | ALL 8 clusters |
| `error.rs` | thiserror error-enum taxonomy + propagation | A, C, D, G, H |
| `logging.rs` | tracing-subscriber + structured emit | A, C, D, H |
| `metrics.rs` | counter/gauge/histogram + rolling window | B, C, E, H |
| `signals.rs` | signal-observation pattern | B, E, H |
| `shared_types.rs` | newtype discipline (SessionId, ConsumerId, etc.) | A |
| `state.rs` | central state-table + transitions | C, D, G |
| `config.rs` | feature flags + env override + TOML | D (m8 build-script) |
| `tensor_registry.rs` | 12D tensor framing | B (m6 cost bands), F (per-edge confidence) |
| `self_model.rs` | engine-knows-about-its-own-state | G (m31 selection criteria) |
| `nam.rs` | substrate concepts | F (proposal-as-substrate-frame language) |

## Compliance posture

- ✅ NO source code files (markdown spec documents only)
- ✅ NO Cargo.toml, NO `cargo init`, NO `cargo` invocation
- ✅ Rust code blocks within markdown are spec documentation (types, schemas, algorithm pseudo-code) — NOT compileable source
- ✅ Honors v1.2 verb-locked invariant for Phase A modules (m1-m15: passive verbs)
- ✅ Active verbs permitted in m20-m23, m30-m33, m40-m42 per Luke single-phase override
- ✅ All boilerplate references map to actual files in `boilerplate modules/` subfolder
- ✅ All ME v2 references map to actual files in `~/claude-code-workspace/the_maintenance_engine_v2/src/m1_foundation/`
- ✅ Bidirectional `[[wikilinks]]` to HOME, MASTER_INDEX, workflow-engine-code-base

## Module ID consistency

Single-phase architecture uses `m1-m42 sparse` naming (1-digit unpadded). Spec docs follow this convention.

**OI-3 (module-count inconsistency 28/11/25/26 across artefacts)** and **OI-4 (m01-padded vs m1-unpadded naming convention)** from `workflow-engine-code-base.md` open-issue tracker are RECONCILED in these specs (26 modules using m1-m42 unpadded). G5 spec interview may revisit if v0's 28-module map is preferred.

## What still needs Luke / Watcher / Zen direction

1. **v1.3 spec patch** absorbing single-phase + 26 modules + waiver record (replaces v1.2 binding spec)
2. **Zen G7 re-audit** on v1.3 patch + these 8 cluster specs as supporting material
3. **Watcher G1 ratification close-notice** (Path A formally ratified)
4. **G3-G8 sequence** (Conductor `:8125` redeploy verify + Watcher notes + interview + dual-frame gap + Zen audit + persistence)
5. **G9 explicit `start coding workflow-trace` signal** (currently queued-intent-only per Zen URGENT block)

## See also

- [[../HOME]] — vault landing
- [[../MASTER_INDEX]] — comprehensive catalogue
- [[../workflow-engine-code-base]] — workflow tracker + decision log
- [[../Modules Synergy Clusters and Feature Verification S1001982]] — single-phase 26-module architecture (parent of these specs)
- [[../boilerplate modules/README]] — REFERENCE-ONLY archive (48 files study material)
- [[../boilerplate modules/BOILERPLATE_INDEX]] — per-file lift map

---

*MODULE_SPECS_INDEX last refreshed: 2026-05-17 ~11:30 (8 parallel rust-pro agents complete)*
