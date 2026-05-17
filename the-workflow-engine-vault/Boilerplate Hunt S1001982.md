---
title: Boilerplate Hunt — 9-Fleet Wave 1 Report S1001982
date: 2026-05-17 (S1001982)
kind: vault-mirror (canonical lives in ai_docs)
status: planning-only · ~65% lift density confirmed
---

# Boilerplate Hunt S1001982

> Back to: [[HOME]] · [[MASTER_INDEX]] · [[workflow-engine-code-base]] · canonical: [[THE_WORKFLOW_ENGINE_BOILERPLATE_HUNT_S1001982]]
>
> Related: [[Town Hall S1001982]] (preceded; 15 P0 constraints scoped the hunt) · [[Convergence Command x Command-3 S1001982]] (extended via Command-3 6-fleet recon, ~62% reuse) · [[Module Structure S1001982]] (consumed findings)

## Summary

9 parallel Explore agents dispatched against scoped habitat crates + Obsidian master indexes. **63 raw candidates surfaced**, consolidated to 9 top-picks, 4 cross-cutting boilerplate sources, **3 structural gaps that cannot be boilerplated**, 1 maturity-ceiling blocker (Conductor).

## 9 Top-Picks by category

| # | Category | Top-Pick | Strength |
|---|---|---|---|
| 1 | CLI binary scaffolding | habitat-conductor binaries + LCM m35_cli_root (fusion) | STRONG |
| 2 | stcortex consumer (narrowed) | stcortex/clients/rust-subscriber/capacity.rs:213-297 | STRONG |
| 3 | SQLite read-heavy multi-DB | memory-injection m06_schema + m11_parallel_query | STRONG |
| 4 | Compositional pattern detection | memory-injection m10 buoy + ORAC m49 Kahn's | MEDIUM (keystone gap) |
| 5 | Decay / TTL / LTD | povm-v2 lifecycle.rs + ORAC m39 fitness tensor (hybrid) | STRONG combined |
| 6 | Daemon scaffolding | synthex-v2 runtime+shutdown + habitat-nerve-center main | STRONG |
| 7 | HABITAT-CONDUCTOR dispatch | habitat-conductor state.rs + enforcement.rs | **MATURITY CEILING — Wave 1B+ not live** |
| 8 | NexusEvent + LCM RPC | LCM lcm_supervisor.rs + ORAC m22_synthex_async + habitat-bench-spine emitter | STRONG |
| 9 | Trap + verify + escape-surface | /forge + /pre-deploy-hardening + hookify preserve-blanket-guard | STRONG per-piece / GAP (no unified schema) |

## 4 Cross-cutting boilerplate sources

- **synthex-v2/src/daemon/** — appears in Cat 6 + 8 + indirect 3, 5 (~3,000 LOC potentially liftable)
- **loop-engine-v2/src/bin/lcm_supervisor.rs + m08_cli/m35_cli_root** — Cat 1 + 6 + 8
- **habitat-conductor/src/bin/{weaver,zen,enforcer}** — Cat 1 + 7
- **memory-injection/src/m2_schema/m10_pattern.rs** — Cat 4 + 5

## 3 Structural gaps (cannot be lifted; must be authored)

1. **N-step compositional sub-graph detection** — POVM does pairwise; m10 does linear; ORAC m49 does cycle detection; nothing does N-step pattern mining with gaps + isomorphism. Engine keystone.
2. **`frequency × fitness × recency` decay law** — POVM has frequency+recency; m10 has frequency+buoy; ORAC m39 has fitness; no compound formula exists.
3. **Unified destructiveness / escape-surface schema** — scattered across /forge, /genesis, hookify preserve-blanket-guard, security-auditor; no unified primitive.

## The Conductor Blocker

HABITAT-CONDUCTOR Waves 0/-1/0.5/0.75/1A LIVE. Waves 1B/1C/2/3 BUILT + INSTALLED + REGISTERED but NOT LIVE (`auto_start=false` Batch 5 pending Luke's terminal bring-up). m32 cannot ship working until Conductor `auto_start=true`.

## Total estimate (informed single-phase shape)

- Lift-as-is: **~2,700 LOC**
- Adapt: **~1,100 LOC**
- 3 structural gaps (new authorship): **~950-1,550 LOC**
- **Total: ~4,750-5,350 Rust** — matches single-phase ~5,200 LOC estimate

## Current relevance

Boilerplate-hunt findings directly informed the 26-module architecture in [[Modules Synergy Clusters and Feature Verification S1001982]]. ~65% lift density confirmed across the architecture. The 3 structural gaps live in:
- Gap 1 (N-step detection) → m4 + m20-m23 (Iteration cluster)
- Gap 2 (fitness-weighted decay) → m11 + m31
- Gap 3 (escape-surface schema) → m30 + m32 (Cipher constraint owned)
