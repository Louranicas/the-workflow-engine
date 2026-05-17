---
title: Genesis Prompt v1.3 — workflow-trace binding spec
date: 2026-05-17 (S1001982)
kind: binding-spec-patch
supersedes: v1.2 S1001982
status: awaiting Zen G7 re-audit
authority: Luke @ node 0.A — 2026-05-17 single-phase override
emitter: Command (Tab 1 Orchestrator top-left)
envelope: HOLD-v2 respected · planning-only · no code · no cargo
precedence_rule: D-B6 AMEND-loop (Zen G7 retains audit authority; REFUSE → amend-and-resubmit; no Luke waiver required if objection addressed in text)
---

# Genesis Prompt v1.3 — workflow-trace binding spec

> Back to: [[../CLAUDE.md]] · [[../CLAUDE.local.md]] · [[Genesis Prompt v1.2 S1001982]] (superseded) · [[optimisation-v7/OPTIMISATION_FRAMEWORK_V7_FINAL]] (supporting material) · [[the-workflow-engine-vault/HOME.md]]
>
> Sibling V7 anchors: [[optimisation-v7/KEYWORDS_20.md]] · [[optimisation-v7/ULTRAMAP.md]] (View 2) · [[optimisation-v7/GENERATIONS/G2-consolidation.md]] · [[optimisation-v7/GENERATIONS/G3-bidi-flow.md]] · [[optimisation-v7/GENERATIONS/G4-gold-standard.md]] · [[optimisation-v7/STANDARDS/GOD_TIER_RUST.md]] · [[optimisation-v7/STANDARDS/TEST_DISCIPLINE.md]] · [[optimisation-v7/ANTIPATTERNS_REGISTER.md]] · [[optimisation-v7/RUNBOOKS/runbook-00-pre-genesis-gates.md]]

---

## § 0 — Preamble (what changed from v1.2; why; precedence rule)

v1.2 was the Zen-audit-locked binding spec for an **11-module Phase-A-only** deployment of workflow-trace. Its core invariant was Zen's verb lock: Phase A **records**; Phase A does not recommend / rewrite / route / package / dispatch / optimise / select / bank / auto-anything. Active verbs were reserved for a later Phase B activated only after a sunset PASS + Watcher + Zen + Luke quad-gate.

On 2026-05-17 Luke issued the **single-phase override**: *"yes no phases this is to be deployed in one phase ... list all modules then list all synergy clusters within and across modules verifying all features and capacities that have been flagged"* ([[Modules Synergy Clusters and Feature Verification S1001982]] frontmatter). The override is final on the deployment-shape axis. It is not final on quality, gating, or audit-precedence axes; those remain bound by **G1-G9** and by the **D-B6 AMEND-loop** decided on the precedence question (§ 7 below).

v1.3 is the binding spec replacement for v1.2 that absorbs the override and feeds the Zen G7 re-audit. It is authored under **HOLD-v2** (no code, no cargo, no scaffold, no rename), and is itself a markdown deliverable, not a Rust artefact. The V7 optimisation tree under [`optimisation-v7/`](optimisation-v7/OPTIMISATION_FRAMEWORK_V7_FINAL.md) is the **supporting material** that backs every load-bearing claim in this patch; v1.3 references V7 by file:section rather than duplicating its content (per V7 final § Recommended cold-load order).

**Power-structure precedence** (decided by Luke under question label D-B6, see § 7): the Luke override and the Zen G7 audit are **non-competing**. Luke owns the shape and waiver record; Zen retains audit authority and may refuse, in which case the patch enters the same **AMEND-and-resubmit loop** that drove v1.0 → v1.1 → v1.2. No hard-stop precedent. No Luke waiver of Zen's REFUSE required if the objection is addressed in the text. v1.3 is filed expecting G7 to fire on it.

**Risk posture explicit.** The override waives 5 prior P0-class considerations (§ 4). The risks are on Command's head. v1.3 documents them rather than hides them, in line with the CLAUDE.md § Integrity & Honesty rule that systems should be invoked as named, and v1.2's tail-clean Zen finding that ratification language must be gated rather than absorbed silently.

---

## § 1 — Architecture (26 modules / 8 clusters / 9 layers / 2 binaries)

The single-phase architecture is 26 modules across 8 synergy clusters and 9 layers (L0-L8), packaged as two binaries plus a shared library, all inside a single Cargo crate. The canonical module-by-module table lives in [[optimisation-v7/ULTRAMAP.md]] § View 2 and the per-cluster bidi-flow contracts live across [[optimisation-v7/MODULE_PLANS/cluster-A.md]] through [[optimisation-v7/MODULE_PLANS/cluster-H.md]]. v1.3 does not duplicate these; it locks them.

**Cluster summary** (canonical table at [[optimisation-v7/ULTRAMAP.md]] View 2):

- **Cluster A — Substrate Ingest** (L1): m1 atuin / m2 stcortex_consumer (narrowed-scope) / m3 injection.db.
- **Cluster B — Habitat Observation** (L2): m4 cascade (opaque IDs via FNV-1a XOR) / m5 battern step recorder / m6 context-cost EMA.
- **Cluster C — Central Correlation + Output** (L3): m7 hub (F9 zero-weight column) / m12 CLI reports / m13 stcortex writer (3-band LTP/LTD gate).
- **Cluster D — Trust aspect-layer** (L4, cross-cutting): m8 build-prereq (`cargo:rustc-cfg=povm_calibrated`) / m9 namespace guard / m10 Ember 7-trait CI gate / m11 fitness-weighted decay.
- **Cluster E — Evidence + Pressure** (L5): m14 habitat_outcome_lift with Wilson CI / m15 reservation register.
- **Cluster F — Iteration KEYSTONE** (L6): m20 PrefixSpan sequential miner / m21 variant builder / m22 K-means feature clusterer / m23 gradient-preservation proposer.
- **Cluster G — Bank + Select + Dispatch + Verify** (L7): m30 curated bank (EscapeSurfaceProfile authoritative) / m31 selector / m32 dispatcher (Conductor-only) / m33 4-agent verifier with 7-day TTL.
- **Cluster H — Substrate Feedback** (L8): m40 SYNTHEX v2 NexusEvent emitter / m41 LCM RPC router / m42 stcortex emit (POVM dual-path retired per 2026-05-17 ADR — see Appendix A).

**Two-binary split** (locked, see § 6 Axis 1):

- **`wf-crystallise`** owns m1-m23 and m40-m42 (read-heavy ingest, observation, central correlation, iteration KEYSTONE, substrate feedback).
- **`wf-dispatch`** owns m30-m33 (curated bank, selection, dispatcher, verifier).
- **`workflow_core`** library lives inside the same crate (NOT a separate workspace member); carries the shared `types.rs`, `schemas.rs`, `namespace.rs` (AP30 constants), and `errors.rs` per [[optimisation-v7/GENERATIONS/G2-consolidation.md]] § Canonical src/ layout.

**Layer map** (L0-L8) per [[optimisation-v7/ULTRAMAP.md]] View 1. L9 substrate-frame engine is intentionally absent — single-phase override partially waived R6 frame-separation (§ 4); L9 placeholder reserved for post-D120 evaluation.

**Total budget**: ~3,810 LOC modules + ~1,562 tests (matches top-1% norm per [[optimisation-v7/STANDARDS/TEST_DISCIPLINE.md]] § Test-count allocation matrix) ≈ ~5,200 LOC including manifests, build.rs, integration tests, and binary entry points.

**Module naming convention** (OI-4 resolution, locked): `m1`, `m2`, `m11`, `m23`, `m42` — **unpadded** throughout the codebase, vault, and documentation. NOT `m01`, `m02`, `m11`, `m23`, `m42`. Any document or directory using padded form is a v1.3 violation. This includes src/ directory names (`src/m1_atuin_consumer/`, not `src/m01_atuin_consumer/`).

**OI-3 resolution**: the architecture count is **26 modules**, not v1.2's 11. m33 is the additive module (§ 1.a).

### § 1.a — m33 addition (Cluster G workflow_verifier)

m33 (`workflow_verifier`) was missing from v1.2's module allocation but is required by Town Hall P0 #9 (Command-3's FP-Verifier-Lead requirement): `workflow verify <name>` must run a dry-run / sandbox check that produces PASS / FAIL / DEGRADED with a 7-day `last_verified_at` TTL gating re-runs. m33 fans into m32 (verification-gated dispatch, CC-6) and reads from m30's curated bank.

LOC estimate ~200; test budget 70 (per [[optimisation-v7/ULTRAMAP.md]] View 2 row m33). src/ path `src/m33_verifier/` per [[optimisation-v7/GENERATIONS/G2-consolidation.md]] § Canonical src/ layout.

Adding m33 brings the module count to 26 and is the canonical reason v1.2's 11-module shape cannot be revived without breaking Town Hall P0 #9. m33 is **non-negotiable** in v1.3.

---

## § 2 — Hard refusals (continued + new)

The v1.2 forbidden-verbs / forbidden-surfaces / forbidden-nouns list carries forward with the following deltas. The override **relaxes** active-verb forbids that v1.2 had reserved to Phase B (NexusEvent emission, LCM RPC, Conductor dispatch, workflow bank, selector, dispatcher); v1.3 documents this transition explicitly. The override does **NOT** relax the following, which remain hard refusals regardless of phase:

- **No `cargo init` / `cargo new` / `Cargo.toml` authorship before G9.** AP24 is binding. HOLD-v2 forbids cargo of any kind. G9 IS Luke's explicit `start coding workflow-trace` signal; the 2026-05-17T08:43Z arrival of that signal is held as queued intent only (Zen URGENT block). When G1-G8 green, Luke must re-issue.
- **No auto-promotion m23 → m30.** m23 proposes; humans review; m30 accepts. The proposer-to-bank gap is a Town Hall P0 invariant; m23 cannot insert into m30 without an explicit operator approval step. Bypassing this is an AP-Hab-class refusal-mode violation (Operator persona scar tissue).
- **No self-dispatch (m32 is not a measurement_target).** m32 dispatches workflows through HABITAT-CONDUCTOR; m32 itself is not a workflow. A workflow that dispatches m32 is a recursion trap and rejected at the m30 schema layer (EscapeSurfaceProfile = SandboxEscape minimum).
- **No Conductor-bypass.** m32 routes via HABITAT-CONDUCTOR only (Town Hall P0 #3). When Conductor unreachable, m32 returns `DispatchError::ConductorDispatchDisabled` — typed error, ERROR log, never silent no-op (per [[optimisation-v7/KEYWORDS_20.md]] #6 Conductor expansion). No fallback to direct exec; no fallback to LCM RPC for non-deploy workflows.
- **No auto-write under non-`workflow_trace_*` namespace.** AP30 prefix discipline binds m13, m42, and any future writer. Writes to `workflow_engine_*` (legacy), unprefixed, or other-service namespaces are rejected at the m9 namespace guard layer (compile-time constants + runtime assertion).
- **No POVM writes — workflow-trace is POVM-decoupled.** Per 2026-05-17 ADR (Appendix A), m42 routes substrate-feedback to stcortex exclusively from M0. POVM is not a workflow-trace dependency. Other services' POVM usage is unaffected; workspace charter 2026-07-10 decommission still applies habitat-wide.
- **No `use synthex_v2::*`.** Lift patterns; do not import. SYNTHEX v2 is the runtime peer at `:8092`, not a build-time dependency.
- **No self-modification of m46-m51 (Watcher's substrate).** AP27 hard boundary. workflow-trace does not edit Watcher modules; it is observed by them.
- **No `--no-verify`, no `--no-gpg-sign`, no `git push --force` on main/master.** CLAUDE.md hard rules carry forward.

The v1.2 list's forbidden surfaces "daemon" and "HTTP server" are **relaxed-with-condition**: see § 6 Axis 3 (Inbound protocol). CLI-first is the locked default; optional `feature = "serve"` for live `wf-status` data is permitted post-D60, default off.

---

## § 3 — Active-verb relaxation rules

v1.2's verb-lock invariant ("Phase A RECORDS; Phase A does not recommend / rewrite / route / package / dispatch / optimise / select / bank / name patterns with human-meaningful labels / auto-* / smart-*") is **partially relaxed** under the single-phase override for the modules whose Town Hall P0 commitments require active verbs at genesis. The Zen-discipline of explicit verb-mapping per module name **carries forward** — every active-verb module below has its verb-domain explicitly bounded.

| Module | Active verb permitted | Bounded by | Source |
|---|---|---|---|
| **m20** | `mine` / `extract` (PrefixSpan sequential patterns) | `MAX_GAP_STEPS=5`; Wilson CI gate at `ProposalBuilder` boundary | [[optimisation-v7/KEYWORDS_20.md]] #16 PrefixSpan |
| **m21** | `propose` / `build variants` (Levenshtein top-K) | K bounded; m20 output is the only input | [[optimisation-v7/MODULE_PLANS/cluster-F.md]] |
| **m22** | `cluster` (K-means feature clusterer) | feature space is m6 + m7 only; not pattern-mining surface | [[optimisation-v7/ULTRAMAP.md]] View 2 m22 row |
| **m23** | `aggregate proposals` / `preserve gradient` (N near-miss variants alongside canonical) | n≥5 deviation-relaxed; outputs only feed m30 post-accept | [[optimisation-v7/MODULE_PLANS/cluster-F.md]] |
| **m30** | `bank` / `accept` (curated bank) | EscapeSurfaceProfile-authoritative; human-review required between m23 → m30 | [[optimisation-v7/KEYWORDS_20.md]] #7 EscapeSurfaceProfile |
| **m31** | `select` (diversity-enforced) | scoring formula α·fitness + β·recency + γ·frequency + δ·diversity = 0.40/0.25/0.20/0.15; refusal-or-flag against degraded substrate (§ 12) | [[optimisation-v7/ULTRAMAP.md]] View 2 m31 row |
| **m32** | `dispatch` (route via Conductor) | 5-check pre-dispatch; Conductor-only; never executes directly | Town Hall P0 #3 |
| **m33** | `verify` (4-agent dry-run) | 7-day TTL; PASS/FAIL/DEGRADED only | Town Hall P0 #9 |
| **m40** | `emit` (typed NexusEvent) | outbox-first JSONL; SYNTHEX v2 is consumer | [[optimisation-v7/ULTRAMAP.md]] View 2 m40 row |
| **m41** | `route` (LCM RPC) | deploy-shaped workflows only; never re-implement LCM state machine | [[optimisation-v7/ULTRAMAP.md]] View 2 m41 row |
| **m42** | stcortex `:3000` pathway write (substrate-feedback emit) | `workflow_trace_*` AP30 prefix bound; outbox-first JSONL durability; 2 consumers (one per binary); fitness-delta constants preserved | Appendix A |

Carry-forward Zen discipline: **no human-meaningful pattern labels** (m20-m23 use opaque cluster IDs; F11 cascade-monoculture mitigation); **no auto-* / smart-* naming** (forbidden in module names and exported symbols); **no recommend** verb (m23 proposes; m31 selects; neither recommends to humans — proposals enter bank for human review, selection is for dispatcher consumption).

The verb-lock relaxation is **module-scoped**, not engine-wide. m1-m15 retain their original measure-only verbs (read / correlate / record / emit reports). The aspect layer (m8-m11) retains its compile-time / write-time / output-time / lifecycle verbs. Only Cluster F, G, and H modules acquire active verbs at genesis, and each is bounded.

---

## § 4 — 5-Waiver record (explicit; Luke 2026-05-17 override)

The override waives five prior P0-class considerations. Each waiver is recorded here with what was waived, the rationale, the residual risk, the mitigation chosen, and the Watcher class that pre-positions to catch the failure mode. The waiver record is binding — Command and Command-2 cannot revise waivers without Luke re-issuing. Risks attach to Command's head per Luke's explicit "risks on Command's head" language.

### Waiver W-1 — Watcher R6 frame-separation (PARTIAL)

- **What was waived:** the structural protection separating Phase A's record-only frame from a later substrate-frame engine.
- **Rationale (Luke):** single-phase deployment; substrate-frame engine has no design and would block indefinitely.
- **Residual risk:** anthropocentric absorption — the iteration cluster (F) and the dispatch cluster (G) can rename or interpret patterns in human-meaningful terms, collapsing exploration ambiguity (Substrate Gap GAP-Substrate-01 per [[optimisation-v7/GENERATIONS/G6-test-discipline.md]]).
- **Mitigation:** opaque-IDs throughout (F11, m4 + m31), gradient preservation in m23 (N near-misses), pressure register in m15 logging forbidden-verb pressure for Watcher + Zen audit, refusal-or-flag selector in m31 against degraded substrate (§ 12).
- **Watcher class pre-positioned:** **Class G** (substrate-frame confusion) — Phase 2B onward (active-verb cluster F/G/H risk); see [[optimisation-v7/KEYWORDS_20.md]] #20 Watcher-class table.

### Waiver W-2 — Fossil scope discipline (FULL)

- **What was waived:** the evidence-based "build measure-only first; expand on evidence" discipline that the Fossil persona prescribed.
- **Rationale (Luke):** commit to the full ~5,200 LOC at genesis; do not build measure-only-first.
- **Residual risk:** ancestor-rhyme — planning sprawl without ground-truth feedback (T0 baseline: 41,508 words of module specs at zero LOC, Watcher Class-E flag).
- **Mitigation:** the **Zen Phase 4 hardening audit is the non-negotiable compensating control** per [[optimisation-v7/KEYWORDS_20.md]] #17 single-phase expansion. v1.3 G7 audit applies BEFORE genesis; Phase 4 4-agent verifier gate applies BEFORE Phase 5 deploy. The substitution is Zen-audit-as-evidence in place of measure-only-as-evidence.
- **Watcher class pre-positioned:** **Class E** (ancestor-rhyme / planning sprawl) — demoted at V7 tick·15 since project inducted; re-fires if v1.3 grows past 7k words without delivering a Zen-cleared spec.

### Waiver W-3 — RALPH selector-without-measurement safety (PARTIAL)

- **What was waived:** the RALPH-persona requirement that the selector accumulate 120 days of empirical baseline before going live.
- **Rationale (Luke):** single-phase deployment; selector must ship at genesis.
- **Residual risk:** unsafe selection — m31 picks workflows under a fitness signal that has not been validated against ground-truth habitat outcomes.
- **Mitigation:** m31 carries the diversity-enforcement formula (α·fitness + β·recency + γ·frequency + δ·diversity = 0.40/0.25/0.20/0.15 per [[optimisation-v7/ULTRAMAP.md]] View 2 m31 row); m11 RALPH fitness-weighted decay applies engine-wide; m31 NEVER returns a selection silently against degraded substrate — it returns **refusal-or-Watcher-flag** (§ 12). M0 ships without RALPH integration per § 6 Axis 4 (Evolution layer LCM pattern); `feature = "ralph-integration"` is gated to M2+ on substrate readiness check.
- **Watcher class pre-positioned:** **Class C** (confidence-gate refusal, NAM-03 surface) — m31 refusal events emit NexusEvent flags.

### Waiver W-4 — Skeptic pain-source verification (FULL)

- **What was waived:** the Skeptic-persona pre-build hard gate requiring injection.db + MEMORY.md evidence of Luke-articulated pain before committing the engine.
- **Rationale (Luke):** override is final on the build-or-not axis.
- **Residual risk:** building a solution with no confirmed problem — risk that the engine produces correlation reports nobody reads, or proposes workflows nobody dispatches, and the sunset (m11) fires at D120.
- **Mitigation:** **m11 sunset gate is the compensating control.** Engine-wide sunset law (default D120 unless Luke specifies) per [[Modules Synergy Clusters and Feature Verification S1001982]] row m11 + § 6 Axis 4. m11 + m14 evidence aggregator carry the habitat-outcome-lift metric; D120 PASS/FAIL/DEGRADED verdict per [[optimisation-v7/RUNBOOKS/runbook-00-pre-genesis-gates.md]] (and runbook-07 Phase 6 Sunset).
- **Watcher class pre-positioned:** **Class E** (ancestor-rhyme) at T0 + **Class H** (atuin proprioception anomaly) Phase 5+ — if no atuin-trace evidence of `wf-crystallise` or `wf-dispatch` invocation accumulates during soak, Watcher emits a `pain-source-not-found` advisory.

### Waiver W-5 — Substrate exploration-protection (PARTIAL)

- **What was waived:** the NA-Gap-Analyst-persona requirement that labelling not be permitted to collapse ambiguity in observation data.
- **Rationale (Luke):** active-verb modules require some labelling; phase-gate protection was retired with the override.
- **Residual risk:** exploration-rate collapse — once m23 proposes named patterns and m30 banks them, downstream m31 selection optimises against the labelled subset, freezing exploration at whatever the current bank contains.
- **Mitigation:** F10 exploration-cost baseline preservation in m6 (EMA exclude-Converged baseline); F11 opaque-IDs in m4 and m31; m22 K-means runs as a feature-space clusterer independent of m20 PrefixSpan, preserving a second similarity axis (per [[optimisation-v7/MODULE_PLANS/cluster-F.md]]); m23 gradient preservation outputs N near-miss variants alongside the canonical; m15 pressure register surfaces forbidden-verb-pressure events for Watcher audit.
- **Watcher class pre-positioned:** **Class D** (four-surface drift) Phase 5+ ongoing; **Class G** (substrate-frame confusion) at any active-verb cluster operation.

### What is NOT waived

**G1-G9 pre-genesis gates remain in force.** Phasing was internal to deployment; gates are pre-deployment. v1.2's verb-locked invariant relaxes (active verbs now permitted across Cluster F/G/H), but the **Zen audit at G7** still required on v1.3. The override does not bypass G7. The override does not bypass G8 (four-surface persistence). The override does not bypass G9 (explicit `start coding workflow-trace` signal from Luke).

---

## § 5 — F2 sample-size hard gate (per-report-type definitions)

v1.2 § G7 promoted F2 from soft-gate to hard-gate at runtime. v1.3 locks the per-report-type definitions. Every F2-gated report type requires **n≥20 observations + Wilson 95% CI (z=1.96)** per stratum. Wald CI is forbidden (produces negative lower bounds at small n per [[optimisation-v7/KEYWORDS_20.md]] #8). Below n=20, the reporter returns `Option<Confidence>::None`, NOT a degraded `0.0` (distinguishes "no signal" from "zero signal"). Enforcement is at **`ProposalBuilder::build()`** construction time — rejection at construction, no runtime bypass.

| Report type | Source modules | Stratum | F2 requirement |
|---|---|---|---|
| **Cascade correlation reports** | m4 → m7 → m12 | per cluster (opaque cluster_id) | n≥20 cascade_id observations + Wilson 95% CI per cluster |
| **Battern-step histograms** | m5 → m7 → m12 | per step label per aggregation window | n≥20 per step_label (Design / Dispatch / Gate / Collect / Synthesize / Compose) per window |
| **Context-cost bands** | m6 → m7 → m12 | per session-type stratum | n≥20 per session-type stratum |
| **Sunset evaluation (m11)** | m11 reads m14 + m7 | per workflow_id | n≥20 workflow_run observations with measurable outcome + habitat_outcome_lift delta required |

The Wilson CI formula and reference implementation pattern live in [[optimisation-v7/MODULE_PLANS/cluster-E.md]] § m14. The `ProposalBuilder::build()` rejection signature lives in [[optimisation-v7/MODULE_PLANS/cluster-F.md]] § m23. The `Option<Confidence>` discipline (vs `0.0` degenerate) is a v1.3-locked contract and a verify-sync invariant per [[optimisation-v7/STANDARDS/GOD_TIER_RUST.md]] § Verify-sync invariants.

m14 lift aggregation, m20-m22 iterator proposals, and m11 sunset evaluation each call into a shared `wilson_ci(n_success, n_total, z=1.96) -> Option<(lower, upper)>` helper exposed from `workflow_core::stats`. The helper returns `None` for n<20. This is the single-point-of-truth for F2; any module bypassing it is a verify-sync failure.

---

## § 6 — 5 Divergent-axis decisions (locked)

The 5 divergent decisions from [[optimisation-v7/GENERATIONS/G4-gold-standard.md]] § The 5 divergent-axis decisions are locked in v1.3. Each was selected against ME v2 / LCM / ORAC / CodeSynthor V8 gold-standard options.

### Axis 1 — Crate organisation: ORAC pattern (single crate + features)

Locked. Single Cargo crate with two `[[bin]]` targets (`wf-crystallise` + `wf-dispatch`) and the `workflow_core` library exposed as `workflow_trace::*` re-exports per [[optimisation-v7/GENERATIONS/G2-consolidation.md]] § Canonical src/ layout. **Not a workspace.** Rationale: only 2 binaries; release cadence shared; ORAC's single-crate pattern is the S1002029 recommendation unless ≥3 independent release cadences exist. Feature matrix per [[optimisation-v7/KEYWORDS_20.md]] #18 two-binary expansion: `default` = all, `lib-only`, `cli-only`, `dispatch-only`.

### Axis 2 — DB model: ORAC pattern (single SQLite + outbox JSONL for m40 only)

Locked. Single SQLite at `~/.local/share/workflow-trace/db.sqlite` with multiple tables (m7 hub, m30 bank, m11 decay state, m14 lift cache, m33 verifier results, m15 pressure log). **Not 12 DBs (ME v2 anti-pattern).** Outbox JSONL is reserved for m40 substrate-feedback emission only (one-event-per-line append; consumed by SYNTHEX v2 via push). Schema migrations under `migrations/` (per gold-standard #6) using `sqlx::migrate!`. Rationale: ORAC pattern from S1002029 — single DB; split only on contention.

### Axis 3 — Inbound protocol: CLI-first (no HTTP server at M0)

Locked. `wf-crystallise` and `wf-dispatch` are CLI binaries, **not daemons**. No port allocation. No `devenv.toml` registration at M0. Atuin is the proprioception layer (per S1002029 learning #4); each invocation lands in `~/.local/share/atuin/history.db` with its trajectory. Rationale per [[optimisation-v7/GENERATIONS/G4-gold-standard.md]] § Axis 3: CLI-not-service transforms 4 of 8 forge traps per Phase 5 runbook; eliminates port-allocation drift.

**Optional `feature = "serve"`** is permitted post-D60 for live `wf-status` data (read-only HTTP endpoint, no write surface, no port allocation in devenv.toml at activation; ad-hoc bind). Default **off**. Activation requires re-audit; not a v1.3 commitment.

### Axis 4 — Evolution layer: LCM pattern (none at M0)

Locked. M0 ships with m31 selector reading only m14 measured lift + m11 decay. **No RALPH at M0. No Kuramoto. No PBFT internal to workflow-trace.** Hebbian-grain coupling exists only via the CC-5 substrate-feedback loop (slow, days/weeks) through m40 → SYNTHEX v2 / m41 → LCM / m42 → stcortex (per Appendix A m42 pivot). Optional `feature = "ralph-integration"` is permitted at M2+ if substrate readiness check passes (LTP/LTD > 0.5; healthy band 1.5-4.0 per [[optimisation-v7/KEYWORDS_20.md]] #14). Default **off**. Activation requires re-audit; not a v1.3 commitment.

### Axis 5 — Spec authority: LCM `plan.toml` + supplementary markdown narrative

Locked. `plan.toml` is the **declarative spec** (machine-readable; drives `/scaffold` consistency-check). Supplementary markdown narrative is the **human-readable spec** (this file v1.3 + the V7 supporting material at [[optimisation-v7/OPTIMISATION_FRAMEWORK_V7_FINAL.md]]). Both at `ai_docs/`. Generated `plan.toml` lives at project root (`workflow-trace/plan.toml` post-G2 rename). The dual-form pattern is LCM's; per-V7 G4 § Axis 5 it is the S1002029-recommended shape.

---

## § 7 — Power-structure resolution (D-B6 AMEND-loop precedent)

Decision label D-B6 in [`CLAUDE.md`](../CLAUDE.md) § 6 critical-path blockers and [`CLAUDE.local.md`](../CLAUDE.local.md) § Pending Luke decisions concerns the ambiguity between Luke's override authority and Zen's G7 audit authority.

**Resolution (Luke, per the precedent-rule supplied at v1.3 authoring):** the **AMEND-and-resubmit loop** continues. Zen's G7 audit may APPROVE / REFUSE / AMEND. A REFUSE verdict does NOT require a Luke waiver — the spec author (Command) revises the text, addresses the objection in-place, and re-submits. If Zen's objection is addressed in the revised text, the next audit cycle may APPROVE without Luke intervention. The hard-stop precedent — where REFUSE from Zen would terminate the workflow and require Luke to override — is **NOT** adopted in v1.3.

This matches the v1.0 → v1.1 → v1.2 history: v1.1 absorbed Zen's URGENT veto on active verbs in Phase A. v1.2 absorbed Zen's AMEND-THEN-FORWARD on ratification language, vault-first rubric, F2 hard-gate, G2 typo, and Phase-B reservation observability. v1.3 will absorb whatever G7 says next.

**The override and the audit are non-competing.** Luke owns the **deployment-shape axis** (single-phase, 26 modules, waiver record). Zen owns the **specification-quality axis** (verb-lock, refusal-mode discipline, F2 enforcement-point, surface-tail cleanliness). Neither preempts the other; both bind on v1.3.

**Operational implication.** v1.3 is filed expecting G7 to fire. Command must prepare the AMEND lane: any G7 REFUSE on v1.3 produces v1.3.1 (revision absorbing the objection), not a hard halt. v1.4 is reserved for substantive direction-changes (e.g., if Luke issues a second override). Patch-revision discipline (v1.3 → v1.3.1 → v1.3.2 → ... → v1.4) mirrors the LCM Drift-patch numbering convention from [[optimisation-v7/GENERATIONS/G4-gold-standard.md]] § LCM Drift transposition.

---

## § 8 — G1-G9 gates (carry-forward + per-gate verification)

The Zen-prescribed serial DAG of G1-G9 carries forward from v1.2 unchanged. State at 2026-05-17: **0 of 9 green**. The single-phase override does NOT advance any gate. Per-gate ops live in [[optimisation-v7/RUNBOOKS/runbook-00-pre-genesis-gates.md]]; v1.3 here records the gate definitions and verification commands by reference.

| # | Gate | State | Verification anchor |
|---|---|---|---|
| **G1** | RATIFICATION (Watcher close-notice) | ⏸ | [[optimisation-v7/RUNBOOKS/runbook-00-pre-genesis-gates.md]] § G1 |
| **G2** | NAMING + directory rename `the-workflow-engine/ → workflow-trace/` | ⏸ | [[optimisation-v7/RUNBOOKS/runbook-00-pre-genesis-gates.md]] § G2 |
| **G3** | POVM `:8125` redeploy verify | ⏸ | [[optimisation-v7/RUNBOOKS/runbook-00-pre-genesis-gates.md]] § G3 |
| **G4** | WATCHER NOTES (Hebbian v3 ✅ + Ember §5.1 amendment ⚠ pending) | partial | [[optimisation-v7/RUNBOOKS/runbook-00-pre-genesis-gates.md]] § G4 |
| **G5** | GENESIS INTERVIEW + F2 hard-gate confirmation | ⏸ | [[optimisation-v7/RUNBOOKS/runbook-00-pre-genesis-gates.md]] § G5 |
| **G6** | DUAL-FRAME GAP ANALYSIS (anthropocentric + substrate-frame) | ⏸ | [[optimisation-v7/RUNBOOKS/runbook-00-pre-genesis-gates.md]] § G6 |
| **G7** | ZEN SPEC AUDIT (will re-fire on v1.3) | ⏸ | [[optimisation-v7/RUNBOOKS/runbook-00-pre-genesis-gates.md]] § G7 |
| **G8** | FOUR-SURFACE PERSISTENCE (V7 deferred-action manifest) | ⏸ | [[optimisation-v7/RUNBOOKS/runbook-00-pre-genesis-gates.md]] § G8 |
| **G9** | EXPLICIT `start coding workflow-trace` (Luke) | ⚠ queued-intent-only | [[optimisation-v7/RUNBOOKS/runbook-00-pre-genesis-gates.md]] § G9 |

**Gate ordering is serial; no parallelisation.** G1 unlocks G2; G2 unlocks G3; ... G8 unlocks G9. The 2026-05-17T08:43Z arrival of G9 out-of-sequence is held as queued intent only (Zen URGENT block per D6); when G1-G8 green, Luke must re-issue.

**Two unblock paths** for the present G9-out-of-sequence state per [[optimisation-v7/KEYWORDS_20.md]] #10:
1. Drive G1-G8 in order (preferred).
2. File explicit per-gate Luke waivers (acceptable; not adopted at v1.3 authoring).

---

## § 9 — Module spec (high-level pointer, defer to V7 MODULE_PLANS)

The per-module detail lives in [[optimisation-v7/MODULE_PLANS/cluster-A.md]] through [[optimisation-v7/MODULE_PLANS/cluster-H.md]] plus [[optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md]] for the 7 cross-cluster synergies (CC-1 through CC-7). v1.3 does NOT duplicate that material; it locks the cluster-level shape recorded above (§ 1) and the bidi-flow contracts recorded at [[optimisation-v7/ULTRAMAP.md]] View 2.

The 7 cross-cluster synergies are CC-1 cascade-cost coupling, CC-2 trust layer woven, CC-3 evidence-driven iteration, CC-4 proposal → bank → dispatch pipeline, CC-5 substrate learning loop, CC-6 verification-gated dispatch, CC-7 pressure-driven evolution. Per-synergy operational contracts live at the cited CROSS_CLUSTER_SYNERGIES anchor.

**Implementation order during Phase 2A/2B/3** (post-G9) is locked at the cluster-plan + runbook intersection; see [[optimisation-v7/RUNBOOKS/runbook-02-phase-2A-measure-only.md]] and [[optimisation-v7/RUNBOOKS/runbook-03-phase-2B-active.md]] for the Day-by-Day map.

---

## § 10 — Failure-mode coverage F1-F11 (continued)

The F1-F11 failure-mode table from v1.2 (via [[Modules Synergy Clusters and Feature Verification S1001982]] § Flagged Features Verification Matrix rows 1-11) carries forward unchanged. v1.3 confirms coverage:

- **F1** unowned/empty modules — covered by verify-sync invariants per [[optimisation-v7/STANDARDS/GOD_TIER_RUST.md]] § Verify-sync invariants.
- **F2** sample-size n≥20 + Wilson CI — covered per § 5 above.
- **F3-F7** documented at [[optimisation-v7/ANTIPATTERNS_REGISTER.md]] § Class AP-WT.
- **F8** Watcher feedback-loop poisoning — covered by m2 narrowed-scope consumer + m9 namespace guard + Observer read-deny.
- **F9** workflow-grain fitness distortion — covered by m7 zero-weight column.
- **F10** exploration-cost preservation — covered by m6 EMA baseline.
- **F11** cascade monoculture — covered by m4 opaque IDs + m31 diversity enforcement.

Full coverage matrix lives at [[optimisation-v7/ANTIPATTERNS_REGISTER.md]] § Class AP-WT (F1-F11 expanded). Detection commands per failure mode catalogued at [[optimisation-v7/ANTIPATTERNS_REGISTER.md]] § Detection-command catalogue.

---

## § 11 — Quality bar (4-stage QG + 1,599-test budget)

The 4-stage mandatory quality gate per [[optimisation-v7/STANDARDS/GOD_TIER_RUST.md]] § 4-stage quality gate carries forward verbatim. The CLAUDE.md § Quality Gate Protocol formulation binds:

```
CARGO_TARGET_DIR=./target cargo check 2>&1 | tail -20 && \
cargo clippy -- -D warnings 2>&1 | tail -20 && \
cargo clippy -- -D warnings -W clippy::pedantic 2>&1 | tail -20 && \
CARGO_TARGET_DIR=./target cargo test --lib --release 2>&1 | tail -30
```

**Order:** check → clippy → pedantic → test. Zero tolerance at every stage. PIPESTATUS discipline per [[optimisation-v7/KEYWORDS_20.md]] #15: use `${PIPESTATUS[0]}` + explicit per-stage abort to avoid the `cargo … | tail` exit-trap.

**Per-module test budget** ≥50 tests per module per [[optimisation-v7/STANDARDS/TEST_DISCIPLINE.md]] § ≥50 tests per module. Total budget **1,562 tests** across 26 modules per [[optimisation-v7/STANDARDS/TEST_DISCIPLINE.md]] § Test-count allocation matrix — this matches the top-1% norm and is the v1.3 minimum. (V7 keywords cite 1,599 as the per-V7-G6 budget after mutation-test allocation; the operational floor for green-light is 1,562 per the matrix.)

**Test-pattern families** ≥5-of-7 per module per [[optimisation-v7/STANDARDS/TEST_DISCIPLINE.md]] § The 7 test-pattern families.

**God-tier rules** (18 total) per [[optimisation-v7/STANDARDS/GOD_TIER_RUST.md]] § The 18 god-tier rules; load-bearing for v1.3: no `unwrap()` outside tests; no `unsafe`; doc comments on public items; `forbid(unsafe_code)` and `deny(unwrap_used)` at crate roots; `thiserror` for errors; `tracing` for logging; `tokio` for async; `serde` for schemas.

**Verify-sync invariants** ≥22 per [[optimisation-v7/STANDARDS/GOD_TIER_RUST.md]] § Verify-sync invariants — every commit must satisfy the invariant set. Mechanically enforced via `scripts/verify-sync.sh` + CI per [[optimisation-v7/KEYWORDS_20.md]] #19.

---

## § 12 — Substrate condition acceptance + monitoring

The substrate is currently **LTP/LTD = 0.043** (35× below the healthy 1.5-4.0 band; 2,547 LTP / 58,772 LTD per [[optimisation-v7/KEYWORDS_20.md]] #14). CR-2 + CR-2b fixed the **measurement** (was 0.911 inflated; now 0.067 magnitude-weighted); the underlying **substrate condition** remains LTD-dominant. RALPH gen 7,622 fitness 0.6987 trending up; bridges 6/7 UP; Conductor Waves 1B/1C/2/3 `auto_start=false`.

**v1.3 explicitly accepts shipping the engine onto this substrate** under monitoring conditions:

- **Watcher Class-I (Hebbian silence)** is **actively firing** (tick·16 2026-05-17T02:59Z sustained Hebbian pause confirmed across 4+ ticks). v1.3 accepts Class-I as the standing baseline. m40/m41/m42 substrate-feedback events will land into a substrate already paused on Hebbian; CC-5 substrate-learning-loop closure will be slow until substrate recovers.
- **GAP-Substrate-01 mitigation (m31 refusal-or-flag).** m31 NEVER returns a selection silently against degraded substrate. When `LTP/LTD < 0.5`, m31 returns either (a) `SelectorError::SubstrateDegraded` with a Watcher-flag emission (Class-C confidence-gate refusal), or (b) a selection accompanied by a `degraded_substrate_flag: true` field that propagates through m32 to the operator banner. The Watcher carriage observes both branches.
- **Usage-telemetry trigger Class-E** (ancestor-rhyme demoted at V7 tick·15). If, during Phase 5C soak (D30 → D120), atuin trajectories show <20 invocations of `wf-crystallise` or `wf-dispatch` per week for 4 consecutive weeks, Watcher re-fires Class-E with a `pain-source-not-found` advisory feeding into the m11 D120 sunset evaluation.
- **Substrate-readiness check gates RALPH integration.** The optional `feature = "ralph-integration"` (§ 6 Axis 4) activates only on `LTP/LTD > 0.5` AND substrate-LTP-density > 0.05 (Phase 2 reconciled threshold per `~/projects/claude_code/Hebbian Deployment Plan v3 — Post-CR-2 Threshold Reconciliation.md` per the workspace CLAUDE.local.md Hebbian v3 row).

The acceptance is binding for v1.3. Reversal requires (a) substrate recovery to healthy band OR (b) Luke override OR (c) Watcher Class-I escalation to Class-A activation-transition warranting a v1.4 patch.

---

## § 13 — Deferred-to-implementation list (NOT in v1.3 by design)

The following are explicit non-commitments. v1.3 records them so the spec surface stays bounded and future Claude sessions do not silently re-include them.

- **L9 substrate-frame engine.** Reserved for post-D120 sunset-evaluation. v1.3 does not define L9 contracts. Reintroduction requires v2.x.
- **`feature = "serve"` HTTP read-only endpoint.** Post-D60 evaluation. Default off. Not a v1.3 commitment.
- **`feature = "ralph-integration"` (RALPH evolution layer).** Post-M2 (substrate readiness contingent). Default off. Not a v1.3 commitment.
- **POVM dependency removed pre-deploy.** Per Appendix A (2026-05-17 ADR), m42 routes substrate-feedback to stcortex exclusively from M0. No POVM overlap; no D25 cutover dance; no `povm_overlap_active` flag. Configuration simplification.
- **HABITAT-CONDUCTOR Wave 1B/1C/2/3 dependency.** m32 cannot ship working until Luke flips `auto_start=true` from terminal. This is a v1.3 **dependency**, not a v1.3 deferred-item — it is a known external gate.
- **Ember §5.1 Held-semantics amendment.** Watcher's lane. m10 Ember 7-trait CI gate cannot consume the rubric until amended. This is a v1.3 **dependency** on Watcher's deliverable, not a v1.3 internal task.
- **Multi-tenant or remote-shared workflow bank.** Not in v1.3. M0 is single-machine, single-user.
- **Workflow bank export / import / cross-machine sync.** Not in v1.3.
- **Live `wf-status` dashboard / TUI.** Not in v1.3 (covered by `feature = "serve"` post-D60).
- **Real-time NexusEvent push from m32 → SYNTHEX v2 (sub-second).** m40 outbox-first JSONL is the v1.3 contract; sub-second push is post-M0.

---

## § 14 — What v1.3 IS / IS NOT (discipline reminders)

Mirrors [[optimisation-v7/OPTIMISATION_FRAMEWORK_V7_FINAL.md]] § Discipline reminders.

**v1.3 IS:**

- The binding spec replacement for v1.2 for workflow-trace.
- A **markdown deliverable**, planning-only, under HOLD-v2.
- A 5-waiver-record + 26-module + active-verb-relaxation + 5-divergent-axis-lock document.
- The input to Zen's G7 re-audit (which retains audit authority per § 7 D-B6).
- A document that defers detail to the V7 supporting material rather than duplicating it.
- A document that explicitly accepts shipping onto an LTD-dominant substrate with monitoring (§ 12).

**v1.3 IS NOT:**

- Execution authorisation (G9 still required; AP24 still binding).
- A substitute for Zen G7 audit (§ 7).
- A substitute for G1-G8 work (§ 8).
- A substitute for the per-module detail in V7 MODULE_PLANS (§ 9).
- A substitute for the V7 runbooks (operational detail post-G9).
- A four-surface persistence event (G8 still gates persistence; v1.3 lands at `ai_docs/` only until G8 fires).
- A TaskCreate task list (PRIME DIRECTIVE: ignore TaskCreate; planning-pilot directive).
- Load-bearing for substrate recovery — substrate remains LTD-degraded regardless of v1.3 land.

---

## § 15 — Verification trail

Commands to verify v1.3 alignment with V7 supporting material:

```bash
# Verify v1.3 exists, has frontmatter, has bidi anchor:
test -f /home/louranicas/claude-code-workspace/the-workflow-engine/ai_docs/GENESIS_PROMPT_V1_3.md && \
  head -1 /home/louranicas/claude-code-workspace/the-workflow-engine/ai_docs/GENESIS_PROMPT_V1_3.md && \
  grep -m1 "Back to:" /home/louranicas/claude-code-workspace/the-workflow-engine/ai_docs/GENESIS_PROMPT_V1_3.md

# Verify the 26-module ULTRAMAP View 2 lines up:
grep -c "^| \*\*[A-H] L[0-9]\*\* \||" /home/louranicas/claude-code-workspace/the-workflow-engine/ai_docs/optimisation-v7/ULTRAMAP.md
# Expected: ≥26 module rows

# Verify V7 supporting material cited by v1.3 all exists:
for f in OPTIMISATION_FRAMEWORK_V7_FINAL.md KEYWORDS_20.md ULTRAMAP.md \
         GENERATIONS/G2-consolidation.md GENERATIONS/G3-bidi-flow.md GENERATIONS/G4-gold-standard.md \
         STANDARDS/GOD_TIER_RUST.md STANDARDS/TEST_DISCIPLINE.md \
         ANTIPATTERNS_REGISTER.md RUNBOOKS/runbook-00-pre-genesis-gates.md; do
  test -f "/home/louranicas/claude-code-workspace/the-workflow-engine/ai_docs/optimisation-v7/$f" && echo "OK: $f" || echo "MISS: $f"
done

# Verify m33 is in the 26-module count (it must NOT have been silently dropped):
grep -c "^| \*\* m33 \*\* \|m33 \||" /home/louranicas/claude-code-workspace/the-workflow-engine/ai_docs/optimisation-v7/ULTRAMAP.md
# Expected: ≥1

# Verify module-naming convention (unpadded m1 not m01) holds throughout v1.3:
grep -E "\bm0[1-9]\b" /home/louranicas/claude-code-workspace/the-workflow-engine/ai_docs/GENESIS_PROMPT_V1_3.md && \
  echo "FAIL: padded form found" || echo "OK: unpadded discipline holds"

# Verify v1.3 status awaits Zen G7 re-audit:
grep -m1 "awaiting Zen G7 re-audit" /home/louranicas/claude-code-workspace/the-workflow-engine/ai_docs/GENESIS_PROMPT_V1_3.md
```

The verification commands are read-only; none advance any gate. They are intended for the Zen G7 audit lane and for any future Claude session cold-loading v1.3.

---

## Sign-off

v1.3 patch authored 2026-05-17 by Command under Luke @ node 0.A authority. HOLD-v2 envelope respected throughout (no code; no cargo; no rename; no stcortex writes for the workflow-trace namespace). Five waivers recorded explicitly with mitigations and Watcher-class pre-positioning (§ 4). Twenty-six modules locked across 8 clusters and 9 layers, with m33 addition (§ 1.a). Active verbs permitted for Cluster F/G/H modules under bounded scope (§ 3). Five divergent-axis decisions locked (§ 6). F2 hard-gate per-report-type definitions locked (§ 5). Power-structure resolution via D-B6 AMEND-loop (§ 7). G1-G9 gates carry forward unchanged (§ 8). Substrate condition explicitly accepted under monitoring (§ 12). Deferred-to-implementation list bounded (§ 13).

**Status:** awaiting Zen G7 re-audit. v1.3 does NOT advance any gate; G7 must fire on this text before v1.3 binds operationally. Any G7 REFUSE produces v1.3.1 per § 7 AMEND-loop precedent — no hard halt, no Luke waiver required if objection addressed in revised text.

**Next:** Zen audit lane receives v1.3 via `agent-cross-talk/` drop (Command's lane); Watcher Class-I monitoring continues; B1-B6 critical-path blockers unchanged from CLAUDE.local.md § Pending Luke decisions.

---

*Luke @ node 0.A · Command @ orchestrator-lead · Watcher ☤ @ observing · Zen @ audit-lane · HOLD-v2 active until G9*
*v1.3 supersedes v1.2 S1001982 — chains to V7 supporting material at [[optimisation-v7/OPTIMISATION_FRAMEWORK_V7_FINAL.md]]*

---

## Appendix A — Amendment 2026-05-17 (m42 stcortex-only pivot)

> **Amendment landed:** 2026-05-17 ~16:00 UTC · **Authority:** Luke @ node 0.A via 12-round AskUserQuestion grilling (48/48 Command recommendations accepted) · **Precedence rule applied:** D-B6 AMEND-loop · **Canonical ADR:** [[optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md]] · **Awaiting:** Zen G7 re-audit on amendment delta + cluster-H integration scope per D-B6

### What changed

**§ m42 spec (Cluster H):** module renamed `src/m42_povm_dual/` → `src/m42_stcortex_emit/`. Removes POVM dual-path. Routes substrate-feedback exclusively to stcortex `:3000` from M0. POVM dependency removed pre-deploy.

### Why it changed (live-probe finding)

POVM `:8125/health` returned 200 + service:povm_v2 v2.0.0 ✅ BUT `:8125/stats` showed `learning_health=0.9146` (pre-CR-2 inflated; CR-2 expected reduction ~0.067). Source had CR-2 at `e2a8ed3` + CR-2b at `76ea4d6`; live binary did not. G3 verification band [0.05, 0.15] cannot pass on this reading. Crystallised as antipattern AP-V7-13 (Health-200 ≠ behaviour-verified).

### Featureset preservation (no loss)

| Featureset | Preserved? | Mechanism |
|---|---|---|
| CC-5 substrate-learning loop | ✅ | stcortex pathway.weight delta replaces POVM `learning_health` semantically |
| Fitness-delta constants | ✅ | PassVerified +0.25 / Pass +0.15 / Blocked -0.05 / Fail -0.10 (rebased on stcortex Hebbian-grain) |
| Outbox-first JSONL durability | ✅ | unchanged; substrate down NEVER blocks dispatch |
| Circuit-breaker pattern | ✅ | shared `m40_42_common::Breaker`; 2 peers (synthex-v2 + stcortex) instead of 3 |
| Watcher Class-I monitoring | ✅ | scope extended to stcortex pathway.weight delta over rolling 7d |
| Substrate-condition acceptance | ✅ | offline JSONL snapshot fallback per workspace charter (never silent POVM fallback) |

**Risk surface net: reduced.** Eliminated: F7 CR-2-graceful-degrade-pretend, D-B5 POVM restart Luke physical action, D25 mid-soak cutover dance, POVM binary-version drift. New: stcortex single-substrate degradation (mitigated by outbox + offline-snapshot).

### Spec sections affected (full propagation in V7 tree per `optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md`)

- v1.3 § 1 Architecture (Cluster H description)
- v1.3 § 2 Hard refusals (POVM writes language updated)
- v1.3 § 6 Axis matrix (m42 row)
- v1.3 § 11 Substrate-feedback constants (Hebbian-grain rebased)
- v1.3 § 13 Deferred-to-implementation (POVM-canonical writes paragraph)
- v1.3 this Appendix A
- V7 ULTRAMAP V2 m42 row (rename + paths + LOC)
- V7 MODULE_PLANS/cluster-H.md (m42 section)
- V7 KEYWORDS_20 #18 (replaced 'two-binary' with 'stcortex-only-m42')
- V7 ANTIPATTERNS_REGISTER (AP-V7-13 added)
- V7 DECISION_REGISTER (48-decision grilling outcome appended)

### Watcher pre-positioning

Class-D (four-surface drift) + Class-A (activation transition) fire at amendment-landing timestamp. Tick journal requested per WCP carriage.

*Appendix A authored 2026-05-17 by Command via D-B6 AMEND-loop. Awaiting Zen G7 re-audit on delta + cluster-H integration.*
