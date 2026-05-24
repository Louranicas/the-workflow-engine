> **Back to:** [[CLAUDE.md]] · [[CLAUDE.local.md]] · [`../CLAUDE.local.md` § v0.2.0 SHIPPED](../CLAUDE.local.md) · canonical [`../ai_docs/WORKFLOW_TRACE_V020_PLAN_V2_S1004377.md`](../ai_docs/WORKFLOW_TRACE_V020_PLAN_V2_S1004377.md) · ratification companion [[Workflow-Trace v0.2.0 Plan v2 S1004377]]
> **Type:** Ship record · cold-start anchor · 4-surface persistence
> **Date:** 2026-05-24 (S1004377 Phase 12 ship close)
> **Status:** 🟢 **v0.2.0 SHIPPED** — tag `v0.2.0` at commit `5d92248` on `main`, pushed both remotes

# Session S1004377 — v0.2.0 SHIPPED

Workflow-Trace v0.2.0 — the **substrate-safety milestone** authored in Plan v2 — is shipped. All 12 phases of the Plan v2 execution arc landed; tests 2048 → 2163 (+115); 18 commits on `main` from Phase 1 `39e71a7` through the v0.2.0 tag commit `5d92248`. Engine is now **substrate-participation-ready across 7 substrates** per NA-2 expansion (atuin · stcortex · HABITAT-CONDUCTOR · CC-5 loop clocks · Luke + Watcher ☤ · RALPH · Cargo build graph).

This note is the vault-side mirror of the `CHANGELOG.md` `[v0.2.0]` entry and the `CLAUDE.local.md` § "v0.2.0 SHIPPED" cold-start block.

## State block

| Attribute | Value |
|-----------|-------|
| Tag | `v0.2.0` annotated |
| Tag commit | `5d92248` (CHANGELOG ship entry consolidating all 12 phases) |
| Final phase commit | `69e72ce` (Phase 12 SHIP — A1 SD8 Levenshtein + CLAUDE.local.md flip) |
| First phase commit | `39e71a7` (Phase 1 re-baseline + ADR D-S1002127-03 Amendment 1 + RefusalToken ADR) |
| Commit count (Phase 1 → tag) | 18 commits on `main` (16 workflow-trace phase commits + 2 workspace LCM rows interleaved by upstream merge) |
| Tests at v0.1.0 baseline | 2048 |
| Tests at v0.2.0 ship | **2163** (+115; +5.6 %) |
| Failed / ignored / suites | 0 failed · 1 ignored · 39 suites |
| Clippy + pedantic | clean every sub-phase, every 4-stage gate green |
| Mutation kill-rate | 96.3 % held (DX-Mut floor; DX-MGB 4h cap defers re-run per honest residual #11) |
| New modules | 4 — `src/refusal_token/` (V1) · `src/back_pressure/` (V2) · `src/m16_substrate_drift_canary/` (V3 KEYSTONE) · `src/substrate_trust/` (V5) |
| New test suite | 1 — `tests/substrate_fixtures.rs` (V4 catalogue) |
| New ADRs | 3 — `D-S1002127-03` Amendment 1 · `D-S1004XXX-04` RefusalToken · `D-S1004XXX-05` SubstrateTrust cross-habitat |
| Genesis amendment | v1.4 (26 → 27 modules; m16 added in Cluster E) |
| Remotes | `origin/main` + `gitlab/main` both at `5d92248` |

## Phase-by-phase commit table

| Phase | Commit | Subject |
|-------|--------|---------|
| 1 | `39e71a7` | re-baseline + ADR D-S1002127-03 Amendment 1 + RefusalToken ADR D-S1004XXX-04 + CHANGELOG `[v0.2.0-WIP]` |
| 2 | `0023f44` | deep FP-verify + Tier 2 W1 sizing + 7-substrate audit + V3 Genesis v1.4 pre-flight |
| 3 A2 | `b1aea21` | SD9 m22 `FeatureVector` newtype + `kmeans_typed` entry point |
| 3 C1 | `a4690f2` | NA-GAP-06 m13 outbox drain skeleton + Phase 3 done |
| 4 | (locked in §15) | Phase 4 interview — 21 decisions ratified at v2 §15 (no commit; design lock) |
| 5 W4 | `9a15213` | `CuratedBank::client_ref()` + `BankSnapshot` accessor seam |
| 5 W1 | `39953df` | `WorkflowProposal::escape_surface` SemVer-break wire-bump |
| 5 W3 | `d776671` | `WorkflowProposal::cost` + `mutation_weight_for` classifier |
| 5 A4 | `a25540e` | SD11 12-field proposal shape |
| 5 V1 | `f29dc5d` | RefusalToken authorship-typed envelope module + Phase 5 done |
| 6 | `91cbf9c` | Tier 3 R1 + R2 + R3 real verifiers LIVE |
| 7 | `b64bcc6` | m13 `drain_to_refusal_tokens` consumer wire |
| 8 V2 | `16cde46` | per-substrate `SubstrateBackPressureMode` + `BackPressureSignal` envelope + Registry |
| 9 KEYSTONE | `8757e50` | V3 m16 substrate-drift canary + Genesis v1.4 amendment + Zen pair-file |
| 10 V4 | `6ca7ae9` | substrate fixture catalogue (5 + V3-canary; DX-5) |
| 11 V5 | `77dc65c` | `SubstrateTrust` + `substrate_participation_status` accessor + ADR D-S1004XXX-05 |
| 12 SHIP | `69e72ce` | A1 SD8 Levenshtein + CHANGELOG + CLAUDE.local.md flip + `v0.2.0` tag |
| ratify | `5d92248` | CHANGELOG ship entry consolidating all 12 phases + 21 locked decisions + 12 honest residuals + OP-1..OP-6 hand-off |

## What v0.2.0 certifies

> **Engine-side substrate-participation readiness across 7 substrates** (atuin · stcortex · HABITAT-CONDUCTOR · CC-5 loop clocks · Luke + Watcher ☤ · RALPH · Cargo build graph) per NA-2 expansion. The engine has authorship-typed refusal channels (V1), per-substrate back-pressure receivers (V2), a substrate-drift canary (V3 KEYSTONE), substrate test fixtures (V4), and substrate-mediated trust hooks (V5) — all with the NA-5 audit-distinguishability primary check (`is_substrate_imagined_for`) preventing in-engine-receiver-only fallbacks from looking substrate-authored.
>
> It does **NOT** certify substrate-side schemas / daemons / consumer-trust tables exist in the substrate-side repos — post-v0.2.0 cross-habitat coordination per Plan v2 §11 + ADR D-S1004XXX-05.

This is the NA-10 + T-3 re-labelled honest sentence. The earlier draft language ("engine + substrate co-completeness") was Frame-A in substrate vocabulary; this re-label was load-bearing for the §9 recursion check.

## Three stacked SemVer-breaks at v0.2.0 wire level

| # | Break | Source phase | Impact |
|---|-------|--------------|--------|
| 1 | `WorkflowProposal::escape_surface` typed wire-bump (was audit-overlay-only) | Phase 5 W1 `39953df` | v0.1.0 proposals lose round-trip; substrate names its surface |
| 2 | `WorkflowProposal::cost: i64` (new field driven by `variant.mutation` count) | Phase 5 W3 `d776671` | Tier 3 R2 Cost real-verifier requires the field |
| 3 | `WorkflowProposal` SD11 12-field shape (adds `lineage_chain`, `generation_index`, `parent_proposal_id`, `lift_p95`) | Phase 5 A4 `a25540e` | full A4 lineage-vs-fitness coupling |

Net effect: `WorkflowProposal` lifts **6 fields → 12 fields**. v0.1.0 proposals do NOT deserialise at v0.2.0; re-run `wf-crystallise` to regenerate v0.2.0-shape JSONL (no `--migrate` flag).

## 21 §15 decisions ratified (S1004377 Phase 4 interview)

### Round A — load-bearing, no defaults (15)

| ID | Decision |
|----|----------|
| DX-DAW-1 | Tier-2-first sequencing (wire-contracts un-block Tier 3 cleanly) |
| DX-W.a | Retire (iii) audit-overlay (R1 Security is the new enforcement seam) |
| DX-W.b | W1 wire-bump (~150-200 LOC) — substrate can name its surface |
| DX-W.c | SemVer-break (v0.2.0 breaking; CHANGELOG migration note) |
| DX-W3.src | `variant.mutation` count as W3 mutation-weight source |
| DX-V3 | V3 m16 own module (Cluster E expansion; triggers Genesis v1.4 + Zen G7 re-audit) |
| DX-V3.b | Ship at N=7d Zen-silent cap with honest residual (cap fires 2026-05-31) |
| DX-V5 | Full cross-habitat (ADR D-S1004XXX-05 pair-filed; engine half ships in v0.2.0) |
| DX-V5.b | 3-variant `Unavailable(EngineImagined / SubstrateUnreachable / SubstrateAuthored)` sub-tag |
| DX-2 | Per-substrate `SubstrateBackPressureMode` enum (default `Pull` per substrate) |
| DX-1 | 4-variant RefusalToken (no `WatcherAuthored` — Watcher emits via observation channel) |
| DX-5 | Full deterministic replicas (TEST_STRATEGY bump to ~1,750-1,800) |
| DX-A4-coupling | Phase 5 co-land A4 with W1+W3 (one wire-contract regen pass) |
| DX-CI | Option A submodule (Frame-B observation point per NA-2) |
| DX-MGB | Cap 4 h per phase on `cargo-mutants` (encourages `// mutant-equivalent:` proof discipline) |

### Round B — mechanical / policy (3)

| ID | Decision |
|----|----------|
| DX-3 | retain-prior (default; M0 ships this) |
| DX-4 | steps-on-proposal (couples to A4 Phase 5) |
| DX-R3 | variant_id-only (default; lineage-chain is v0.3.0) |

### Stated defaults (3 — NOT interview slots per C-10)

| ID | Default |
|----|---------|
| DX-Mut | hold ≥ 96.3 % kill-rate |
| DX-Soak | 48 h baseline |
| DX-1-mechanism | 4-variant structural floor |

## 12 honest residuals → v0.3.0 / post-v0.2.0

Named, not silenced. Each bounded by gate name (OP-x or external clock).

| # | Residual | Gate |
|---|----------|------|
| 1 | 65-occurrence `RefusalReason` call-site classification cascade | Phase 7 follow-up; tracked v0.3.0 |
| 2 | A1 SD8 spec-compliant invocation (algorithm shipped; hash→steps lookup post-v0.2.0) | v0.3.0 |
| 3 | V4 detailed per-fixture-per-V1-variant test sweep (catalogue shipped; expansion post-v0.2.0) | v0.3.0 |
| 4 | V5 CH-1..CH-5 substrate-side primitives (engine consumer shipped; substrate-side at **OP-4**) | OP-4 cross-habitat ADR review |
| 5 | V3 m16 OP-6 Watcher heartbeat liveness assertion wire (closes NA-4) | **OP-6** Watcher integration |
| 6 | Production drain consumer wire (capability shipped; production forwarding post-v0.2.0) | v0.3.0 |
| 7 | V2 per-substrate cadence-modulation wire into m1/m13/m32 throttle | v0.3.0 |
| 8 | m13 drain APIs `#[allow(dead_code)]` (production consumer post-v0.2.0) | v0.3.0 |
| 9 | Zen G7 re-audit verdict on Genesis v1.4 (DX-V3.b 7-day cap fires 2026-05-31; ship stands per in-plan-locked cap if silent) | external clock 2026-05-31 |
| 10 | DX-CI Option A submodule wire to `.github/workflows/ci.yml` + `.gitlab-ci.yml` | v0.3.0 |
| 11 | `cargo-mutants` scoped run per DX-Mut ≥ 96.3 % hold (DX-MGB 4 h cap defers) | v0.3.0 |
| 12 | `wf-dispatch --execute` live-Conductor verification | **OP-3** post-soak |

## 6 operator hand-offs (OP-1..OP-6 — Plan v2 §16)

| ID | Item |
|----|------|
| **OP-1** | Conductor bring-up + 24h NoOp soak + flip `CONDUCTOR_ENFORCEMENT_ENABLED=1` |
| **OP-2** | directory rename `the-workflow-engine/` → `workflow-trace/` |
| **OP-3** | post-v0.2.0 48h DX-Soak substrate soak (Watcher ☤ carries) |
| **OP-4** | cross-habitat ADR D-S1004XXX-05 review post-v0.2.0 (per-substrate CH-1..CH-5 pair-files) |
| **OP-5** | Master Plan v2 / Ember opportunity-cost reopen per Plan v2 D46 |
| **OP-6** | Watcher m16 heartbeat liveness integration (closes V3 self-canary loop per NA-4) |

## 4-surface persistence at this save

| Surface | Anchor |
|---------|--------|
| **ai_docs canonical** | [`../ai_docs/WORKFLOW_TRACE_V020_PLAN_V2_S1004377.md`](../ai_docs/WORKFLOW_TRACE_V020_PLAN_V2_S1004377.md) (plan; 601 lines) · [`../ai_docs/WORKFLOW_TRACE_V020_PHASE2_AUDIT_S1004377.md`](../ai_docs/WORKFLOW_TRACE_V020_PHASE2_AUDIT_S1004377.md) (Phase 2 audit) · v1 DRAFT + 2 gap analyses (sibling docs) |
| **Obsidian vault** | **THIS note** (`Session S1004377 — v0.2.0 SHIPPED.md`) + ratification companion [[Workflow-Trace v0.2.0 Plan v2 S1004377]] + earlier [[Session S1004115 — Completion Plan v2 Locked]] + [[Session S1004115 — v0.1.1 + v0.2.0 Prep Save]] |
| **stcortex** | namespace `workflow_trace_v020_s1004377` — meta memory chain rooted at id 18511 (Plan v2 RATIFIED, read-back verified per NA-6) + per-phase ship memories landed across Phases 1-12 + bidi pathways `workflow_trace_completion_s1004115 ↔ workflow_trace_v020_s1004377` (weight 0.95). Consumer registered. |
| **CLAUDE.local.md anchor** | project file [`../CLAUDE.local.md`](../CLAUDE.local.md) § "🟢 v0.2.0 SHIPPED — S1004377, 2026-05-24" (cold-start anchor) |
| **CHANGELOG** | [`../CHANGELOG.md`](../CHANGELOG.md) `[v0.2.0]` entry (canonical release record; 2026-05-24) |
| **git tag** | `v0.2.0` annotated at `5d92248` on `main` (pushed origin + gitlab) |

## Cold-start sequence (fresh Claude context window)

```bash
cd /home/louranicas/claude-code-workspace/the-workflow-engine

# 1. Read project session-state delta — § "v0.2.0 SHIPPED" is the live anchor
$EDITOR CLAUDE.local.md

# 2. Read this vault note for the ship-time snapshot + phase table
$EDITOR "the-workflow-engine-vault/Session S1004377 — v0.2.0 SHIPPED.md"

# 3. Read the CHANGELOG v0.2.0 entry (canonical release record)
$EDITOR CHANGELOG.md       # § [v0.2.0]

# 4. Verify git anchor
git log --oneline -1                # expected: 5d92248
git describe --tags HEAD            # expected: v0.2.0
git rev-list -n 1 v0.2.0            # expected: 5d92248...

# 5. Verify substrate
~/.local/bin/stcortex inspect workflow_trace_v020_s1004377 --limit 10
```

— Session S1004377 v0.2.0 SHIPPED record · 2026-05-24.

> **Back to:** [[CLAUDE.md]] · [[CLAUDE.local.md]] · [[Workflow-Trace v0.2.0 Plan v2 S1004377]] · [[Session S1004115 — v0.1.1 + v0.2.0 Prep Save]] · [[Session S1004115 — Completion Plan v2 Locked]]
