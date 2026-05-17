---
title: Gold Standard Exemplars — Synthesis
date: 2026-05-17 (S1001982)
kind: synthesis-reference
status: planning-only · REFERENCE-ONLY
authority: Luke @ node 0.A — "develop a deep understanding of [ME v2, LCM, ORAC] … save all the capacities features and structure"
sources:
  - the_maintenance_engine_v2 @ 552b888
  - habitat-loop-engine @ 568d9e3
  - orac-sidecar @ 6224bd7
---

# Gold Standard Exemplars — Synthesis

> Back to: [[../HOME]] · [[../MASTER_INDEX]] · [[README]] · [[BOILERPLATE_INDEX]]

Three exemplar codebases, profiled in depth as scaffolding references for `the-workflow-engine`. Each is a separate document; this note is the cross-reference + shared-pattern distillation.

## The three profiles

| Exemplar | Doc | Role | Lines | Source commit |
|---|---|---|---:|---|
| **Maintenance Engine V2** | [[Maintenance Engine V2 — Gold Standard Reference]] | 8-layer autonomous orchestrator (port 8180) — PBFT consensus, Hebbian STDP, Kuramoto field, RALPH | 1,109 | `552b888` |
| **Habitat Loop Engine (LCM)** | [[Habitat Loop Engine — Gold Standard Reference]] | Bounded local workflow substrate — workspace of 10 crates, executor/verifier separation, plan-driven scaffold | 322 | `568d9e3` |
| **ORAC Sidecar** | [[ORAC Sidecar — Gold Standard Reference]] | 8-layer fleet-coordination sidecar (port 8133) — V2 wire protocol, 6 hooks, RALPH evolution chamber | 544 | `6224bd7` |

## Why these three

- **ME v2** is the deepest-stacked layered Rust service in the habitat: 8 layers, ~97k LOC, 4,083 tests, 12 SQLite DBs, 7 bridges, PBFT n=40 q=27, 12 design constraints enforced at compile-time. It is the canonical reference for `src/m{N}_*` layered organisation, Hebbian/STDP/PBFT mechanics, and CLAUDE.md governance.
- **LCM** is the canonical reference for a **workspace-of-crates** layout with strict crate-level dependency boundaries (executor vs verifier separation enforced in `Cargo.toml`), plan-driven scaffolding (`plan.toml`), and a substrate-types contract. Its `docs/{operations,plans,quality,reviews,workflows}` partition is the cleanest docs discipline in the habitat. M0 milestone state captured.
- **ORAC** is the canonical reference for an 8-layer **sidecar with a wire protocol state machine**, hook system (6 endpoints, sub-ms response), blackboard pattern (5 SQLite tables, `Arc<Mutex<>>` + owned clones), feature-gated layers (`mcp_gateway`, `full`, per-layer features), and the RALPH 5-phase evolution chamber + 12D fitness tensor.

## Convergent patterns (lift directly)

These appear in ≥2 of the three exemplars and should anchor `the-workflow-engine`'s structure:

| Pattern | ME v2 | LCM | ORAC | Notes |
|---|:-:|:-:|:-:|---|
| `src/mN_<theme>/` layered modules | ✅ m1..m7 | — (crates) | ✅ m1..m8 | Strict DAG; lower index = fewer deps |
| Workspace `Cargo.toml` + `[features]` matrix | ✅ | ✅ 10 crates | ✅ `full`/`mcp_gateway` | Feature-gate per-layer in single-crate, per-crate in workspace |
| `CLAUDE.md` + `CLAUDE.local.md` split | ✅ | ✅ | ✅ | Charter vs session-state delta |
| `MASTER_INDEX.md` at repo root | ✅ | ✅ | ✅ | Always-loaded nav |
| `ai_docs/` + `ai_specs/` | ✅ | ✅ | ✅ | Plans + formal specs separated |
| Obsidian vault co-located with source | `maintenance-engine-v2-vault/` | `vault/` + `docs/` | `orac-sidecar-vault/` | Bidirectional `Back to:` anchors |
| SQLite + `migrations/` dir | 12 DBs, 11 migrations | 2 migrations, 1 DB | `orac_blackboard.db`, layered migrations | Always `.schema` before SQL |
| 50+ tests per module convention | 4,083 total | 3,165 markers (stubs) | 2,993 verified | `#[tokio::test]` inline `mod tests` |
| Quality gate: check → clippy → pedantic → test | ✅ enforced | ✅ `QUALITY_BAR.md` + `local-ci/` | ✅ | Zero tolerance at every stage |
| Bridges as `mX_bridges/<peer>.rs` with breaker | ✅ 5 outbound | ✅ `hle-bridge` | ✅ 7 clients | Circuit breaker FSM per peer |
| `forbid(unsafe_code)` + `deny(unwrap)` | ✅ | ✅ | ✅ | Lint-as-policy |
| Drift register / known-issues catalogue | `Bugs & Known Issues.md` | `CLAUDE.local.md` drift #1..#11 | `Bugs & Known Issues.md` | Live document, not archive |
| Layered config TOML | `.config/` | `etc/` + `plan.toml` | `config/` (default/dev/prod/hooks/bridges) | Layered override |

## Divergent patterns (choose one)

Each exemplar makes a different call on these axes; `the-workflow-engine` must pick:

| Axis | ME v2 | LCM | ORAC | Recommendation for workflow-engine |
|---|---|---|---|---|
| Crate organisation | Single crate, `mN_` layers | 10-crate workspace, layer-per-crate | Single crate, `mN_` layers + feature gates | LCM's workspace **if** ≥3 independently-releasable concerns; otherwise ORAC's feature-gated single crate |
| Persistence model | Many DBs (1 per concern) | Single ledger DB, append-only | Single blackboard DB, 5 tables | Single DB with multiple tables (ORAC); split only when concurrent-write contention is real |
| Inbound protocol | HTTP REST + SSE | CLI + MCP stdio JSON-RPC | HTTP + V2 wire-protocol FSM | HTTP REST for v1; add FSM later only if streaming required |
| Evolution / learning | Hebbian + PBFT + Kuramoto | None (deterministic) | RALPH 5-phase + 12D fitness | None at M0; wire POVM-pathway reads, defer write-side |
| Spec authority | `ai_specs/` + L1 50-spec sheet | `plan.toml` drives generation | `ORAC_PLAN.md` + `ORAC_MINDMAP.md` | LCM's `plan.toml` if scaffold-gen is in scope; otherwise ME v2's spec-sheet pattern |

## Cross-references to the 48-file boilerplate clones

The three profiles above describe **whole systems**. The neighbouring `01-..10-` subdirectories hold **48 per-file source clones** identified by the Boilerplate Hunt as direct-lift candidates. The mapping:

| Clone category | Primary exemplar | Specific section to read |
|---|---|---|
| `01-cli-scaffolding/` | LCM | "Workspace Crate Architecture" → `hle-cli` |
| `02-stcortex-consumer/` | LCM | "Bridges + Integrations" → stcortex client |
| `03-sqlite-multi-db/` | ME v2 | "Persistence + Databases" (12-DB pattern) |
| `04-pattern-detection/` | ORAC | "RALPH evolution chamber" + 12D tensor |
| `05-decay-ttl-ltd/` | ME v2 + ORAC | ME v2 "Capabilities" (Hebbian decay) + ORAC "Patterns to Emulate" (POVM lifecycle) |
| `06-daemon-scaffolding/` | LCM | "Session-resume + drift" (supervisor); ORAC m6_coordination |
| `07-conductor-dispatch/` | ME v2 | "Bridges" (Cascade) — but **see anti-pattern flags in ORAC m24 ref** |
| `08-nexus-lcm-rpc/` | LCM | "Capabilities" → MCP stdio JSON-RPC; ME v2 nexus L8 |
| `09-trap-verify-escape-skills/` | All three | LCM `QUALITY_BAR.md` + ME v2 12 design constraints + ORAC bugs |
| `10-foundation-direct-clones/` | LCM substrate-types | "Substrate Layer" — zero-dep foundation types |

## Shared anti-patterns (do NOT copy)

Drawn from all three `Bugs & Known Issues` + `CLAUDE.local.md` drift catalogues:

- **Mono-parameter evolution mutation** (ORAC BUG-035) — RALPH must mutate ≥2 dims/cycle; single-param trap stalls fitness
- **Cancel-handler skipped** (LCM Drift #9) — handler scaffolded without binary wiring; verify-by-exercise, not by `git log`
- **Supervisor over-claim** (LCM Drift #11) — agent reported "stub" when binary was live; orchestrator must re-exercise, not trust report
- **EventBus 0 external publishers** (ME v2 BUG-008) — declared pub/sub with no producers wired; pattern: write the integration test before declaring the seam done
- **POVM "write-only"** (ORAC AP) — read-without-write trap; `learning_health` inflation factor 13.6× pre-CR-2
- **Thermal-gate race** (ORAC) — hook returns before thermal read settles; sub-ms response SLA conflicts with cross-service probe
- **Pipe-to-`tail` swallows `$?`** (LCM CLAUDE.local.md) — gate prints green while clippy was screaming; use `${PIPESTATUS[0]}`
- **Stash-pop wrong stash** (all three CLAUDE.md) — pre-existing stash silently overwrites current work; always `git stash list` first
- **Blanket prune** (all three CLAUDE.md) — `docker container prune -f` rebuilt its own filter, lost openclaw-gateway

## Scaffolding checklist for the-workflow-engine

When G7/G9 clear and authoring begins, this is the order to seed structure (lifted from convergent patterns above):

1. `Cargo.toml` workspace OR single-crate-with-features (decision per "Divergent" table)
2. `src/m1_foundation/`, `m2_*`, ... — layer skeletons with `mod.rs` only
3. `CLAUDE.md` (charter) + `CLAUDE.local.md` (session-state) — copy ME v2's section structure
4. `MASTER_INDEX.md` at root — copy LCM's partition style
5. `ai_docs/` (plans) + `ai_specs/` (formal specs) — empty, ready for plan.toml or spec sheet
6. `migrations/0001_init.sql` — single table for now, mirror ORAC blackboard shape
7. `the-workflow-engine-vault/` — Obsidian sibling, copy ORAC's `architecture/ modules/ schematics/ sessions/` partition
8. `bacon.toml` — copy LCM's `on_success` chain (check → clippy → pedantic → test)
9. `forbid(unsafe_code)` + `deny(clippy::unwrap_used, clippy::expect_used)` in `lib.rs`
10. `local-ci/` (LCM) OR `.github/workflows/` — quality gate enforcement
11. Bridge module `mN_bridges/<peer>.rs` per planned habitat integration — start as breaker-only stubs
12. First test: `mod tests { #[tokio::test] async fn smoke_compiles() }` per module

## Update history

| Date | Action |
|---|---|
| 2026-05-17 | v1 synthesis — three deep-exploration agents authored the three reference docs in parallel; this note ties them to the 48-file clone tree |

## See also

- [[Maintenance Engine V2 — Gold Standard Reference]] — 8-layer Rust orchestrator deep profile
- [[Habitat Loop Engine — Gold Standard Reference]] — 10-crate workspace deep profile
- [[ORAC Sidecar — Gold Standard Reference]] — 8-layer sidecar deep profile
- [[BOILERPLATE_INDEX]] — per-file lift map for the 48 clones
- [[../Boilerplate Hunt S1001982]] — 9-fleet hunt that surfaced clone candidates
- [[../Modules Synergy Clusters and Feature Verification S1001982]] — single-phase architecture this references support
