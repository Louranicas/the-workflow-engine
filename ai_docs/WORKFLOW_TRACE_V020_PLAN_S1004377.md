# Workflow-Trace v0.2.0 Plan **v1** — S1004377 (substrate-safety milestone after v0.1.0 / M0)

> **Authored:** 2026-05-23 (S1004377) · **Status:** PLAN v1 DRAFT — gap analyses NOT YET RUN; Phase 4 interview NOT YET RUN; awaiting dual-frame gap-analysis fold-in + 12-round decision interview before v2 + ratification
> **Supersedes:** [`WORKFLOW_TRACE_V020_PLAN_STUB_S1004115.md`](WORKFLOW_TRACE_V020_PLAN_STUB_S1004115.md) (scope catalogue)
> **Re-baselined on:** git HEAD `d521a00` (workspace doc fold) over `54cb4e2` (v0.1.1 + v0.2.0 prep multi-surface save) over `7d16c2c` (v0.2.0 prep) over `v0.1.0` tag at `df00fd2`
> **Authorisation chain:** v0.2.0 plan-authoring authorised by Luke @ node 0.A in S1004377 per §15 D40 invocation-only cadence + Plan v2 §14 single-remaining-gate discipline
> **Draft anchor decisions (DAW):** DAW-1 Tier 2 (wire-contracts) lands before Tier 1 (substrate primitives) · DAW-2 full Plan v2 mirror — all 5 Tiers in scope (V1-V5 + W1-W4 + R1-R3 + A1-A4). Locked at draft authorship per Luke; both eligible for revisit during Phase 4 interview.
> **Back to:** [CLAUDE.local.md](../CLAUDE.local.md) · [CLAUDE.md](../CLAUDE.md) · [`WORKFLOW_TRACE_COMPLETION_PLAN_V2_S1004115.md`](WORKFLOW_TRACE_COMPLETION_PLAN_V2_S1004115.md) · [`PHASE9_SD_RECONCILIATION_S1004115.md`](PHASE9_SD_RECONCILIATION_S1004115.md) · [`decisions/2026-05-17-substrate-as-actor-deferrals.md`](decisions/2026-05-17-substrate-as-actor-deferrals.md) (ADR D-S1002127-03 — to be amended in Phase 1)

---

## What changed from the stub

| Stub aspect | Plan v1 elevation |
|-------------|-------------------|
| Scope catalogue (5 tiers, no phase structure) | 12-phase structure with explicit gate per phase (mirrors Plan v2 §3) |
| "Awaiting node-0.A authorization" | **Authorised S1004377 by Luke @ node 0.A** — plan-authoring greenlit, execution still gated on a separate "start Phase 1" per Plan v2 D48 pattern |
| Effort estimate `~6-10 Claude-days at god-tier` (top-down) | Bottom-up roll-up after phase decomposition: `~10-14 Claude-days` (still pre-interview; range narrows after Phase 4 locks the W1-vs-W2 choice + the cross-habitat ADR scope for V5) |
| No tier sequencing | **Tier 2 first** per DAW-1 — wire-contracts un-block Tier 3 verifiers; Tier 1 substrate primitives consume the typed seams cleanly |
| No conventional gap analysis | **Pending Phase 4 prep** — dual-frame gap analyses to be dispatched in parallel after v1 lands; outputs feed v1 → v2 |
| No interview seeds | **§ 15 carries 4 seed questions from stub + 8 candidate Round-B questions** awaiting expansion from gap-analysis findings |

---

# PART A — Conventional plan

## 1. Mission

The 26-module codebase is at **v0.1.0 / M0** (2048 tests passing, clippy + pedantic clean, mutation kill-rate 96.3 % held). v0.1.0 certified **engine-internal completeness only** — every residual the engine owned was closed, tested, audited, and documented. v0.1.0 explicitly did **not** certify the engine's safety as a substrate-facing organ.

**v0.2.0 closes that gap.** It adds substrate-drift detection, substrate-side test fixtures, substrate-mediated trust, authorship-typed refusal, substrate back-pressure, the wire-contract changes that un-block live m33 verifiers, and the Class-C algorithmic upgrades deferred by Phase 9. After v0.2.0 the tag will tell the habitat both *"the engine is done to a milestone"* and *"the engine is safe to run continuously against a live substrate."*

**Per §15 D4** v0.2.0 was a committed follow-on, not a shelf. Per §15 D40 the cadence is invocation-only; this v1 is the invocation.

**What v0.2.0 / `v0.2.0` certifies** (the honest scope line — to be re-derived in Part B with frame-collapse self-check): *engine + substrate co-completeness* — every primitive the engine needs to participate in the substrate as an actor (not a sink-querying user of it) is shipped, tested, audited, and documented across both wire-contract and substrate-as-actor surfaces. It is **not** a cross-habitat trust unification (V5 ships the workflow-trace half; habitat-wide trust unification is post-v0.2.0).

## 2. Verified inventory — v0.1.0 honest residuals → v0.2.0 work

Sourced from `CHANGELOG.md` [v0.1.0] § "Honest residuals", `PHASE9_SD_RECONCILIATION_S1004115.md` § 4, Plan v2 § 11, and the Phase 2 audit ([`PHASE2_AUDIT_S1004115.md`](PHASE2_AUDIT_S1004115.md)). Re-verified at HEAD `d521a00`. Five tiers per stub + carry-overs from Phase 9 Honest Residuals not yet in the stub.

### 2.1 Tier 1 — substrate-as-actor primitives (KEYSTONE; ~1450-1700 LOC)

| ID | Item | Source | Notes |
|----|------|--------|-------|
| **V1** | `RefusalToken` authorship-typed enum (`SubstrateAuthored / EngineAuthored / OperatorAuthored / Unavailable`) wired through m9/m32/m13/m40/m41/m42/m33 | Plan v2 §11 + Phase 2 audit; Honest Residual #1 | Requires new ADR D-S1004XXX-04; replaces flat `RefusalReason` enum at `src/m32_dispatcher/mod.rs:163` |
| **V2** | Substrate back-pressure budget — substrate-emitted contention signal replacing Phase 8 step 3 Frame-A proxy | Plan v2 §11 + Phase 2 audit + §15 D37; Honest Residual #2 | Requires substrate-side participation; in-engine receiver only for v0.2.0 if cross-habitat coordination slips |
| **V3** | `m16_substrate_drift_canary` — periodic prober sampling each CC-5 clock (m11 / m13 / injection-TTL / stcortex-decay / atuin) + asserting agree-to-skew envelope | Plan v2 §11 (ADR D-S1002127-03 W1) + §15 D38; Honest Residual #3 | **KEYSTONE.** Triggers Genesis Prompt v1.4 module-count amendment (26 → 27) + Zen G7 re-audit cascade per D-S1002127-03 § 3 |
| **V4** | Substrate test fixtures — deterministic local replicas of stcortex, ORAC, synthex-v2, LCM, HABITAT-CONDUCTOR | Plan v2 §11 (ADR D-S1002127-03 W2); Honest Residual #4 | `tests/substrate_fixtures/`; ~80-150 fixture tests; expands TEST_STRATEGY budget |
| **V5** | Substrate-mediated trust — substrate participates in verification (not just substrate-confirmable-receipt) | Plan v2 §11 (ADR D-S1002127-03 W3); Honest Residual #5 | Cross-habitat ADR; touches stcortex consumer-trust + Conductor dispatch-budget + atuin read-quota — sequencing risk |

### 2.2 Tier 2 — engine-side wire-contract changes (un-blocks Tier 3)

| ID | Item | Source | Notes |
|----|------|--------|-------|
| **W1** | `WorkflowProposal::escape_surface: EscapeSurfaceProfile` wire-contract field — option (i) of Phase 2 audit § 3 | Phase 2 audit option (i) + Phase 6a invariant lock | Cross-binary serde change; JSONL fixtures regenerate; un-blocks m33 Security from M0 Sandboxed default |
| **W2** | OR `StepToken → EscapeSurfaceProfile` classification table + variant aggregation — option (ii) of Phase 2 audit § 3 | Phase 2 audit option (ii) | No wire-contract change; smaller blast radius; **W1 vs W2 is an interview question** (DX-W) |
| **W3** | `WorkflowProposal::cost: i64` (or similar) wire-contract field — un-blocks D9 Cost real verifier per D10 metric `step-count × mutation-weight` | §15 D9 + Phase 2 wire-contract sizing | ~150-250 LOC cross-binary |
| **W4** | `CuratedBank::client_ref()` accessor seam — un-blocks D11 Consistency real verifier per §15 D12 on-demand discipline | §15 D11 + D12 + T4-API #1 | ~30-60 LOC; smallest of the four |

### 2.3 Tier 3 — real verifier impls (consume Tier 2)

| ID | Item | Source | Notes |
|----|------|--------|-------|
| **R1** | m33 Security real verifier — consumes W1 OR W2 (per DX-W) | Phase 6a + Phase 2 audit | Replaces M0 Sandboxed-default defense-in-depth-redundancy slot; verdict semantic stays hard-Refuse per D5/D6 |
| **R2** | m33 Cost real verifier per D10 metric `step-count × mutation-weight` — consumes W3 | §15 D9 + D10 | ~50-80 LOC; replaces M0 documented-Approve-stub |
| **R3** | m33 Consistency real verifier — bank-conflict detection consuming W4 | §15 D11 | ~100-150 LOC; replaces M0 documented-defer-stub |

### 2.4 Tier 4 — algorithmic / shape upgrades (Class-C SD drifts)

| ID | Item | Source | Notes |
|----|------|--------|-------|
| **A1** | SD8 — m21 true Levenshtein over source pattern's steps (replaces M0 closed-form proxy) | §15 D27 + Phase 9 § 4 | Requires step-lookup; proposal carries `source_pattern_hash` not steps — adds a hash→steps lookup or carries steps |
| **A2** | SD9 — m22 typed `FeatureVector` newtype wrap | §15 D27 + Phase 9 § 4 | Cosmetic; lands early in decision-free Phase 3 candidate |
| **A3** | SD10 — m22 empty-cluster typed-error OR re-seed (spec amendment + impl) | §15 D27 + Phase 9 § 4 | Code retains-prior; spec amendment is the M0 fold; new behaviour is DX-3 |
| **A4** | SD11 — m23 12-field proposal shape (ties to W1 + W3) | §15 D27 + Phase 9 § 4 | Six new fields per Phase 9 § 2b SD11 row — naturally co-lands with W1+W3 |

### 2.5 Carry-overs from v0.1.0 Honest Residuals not yet in stub

| ID | Item | Source | Notes |
|----|------|--------|-------|
| **C1** | NA-GAP-06 **outbox drain** half — write half landed (m13 `outbox_path`); drain consumer absent | Phase 2 audit §2 NA-GAP-06 row + Phase 9 § 4 #9 | Phase 2 audit recommended pair with NA-GAP-09 in 6f; only the emit half shipped at M0. Pairs naturally with V1 RefusalToken authorship typing. |
| **C2** | m31 production caller `\|_w\| 0.0` (Phase 5 nit A2) | Phase 9 § 4 #6 | ✅ **CLOSED 2026-05-23 v0.1.1 round** (`7d16c2c` Phase 5 zen A2 wired). Carried here only to mark stub Tier 5 C1 as fully resolved. |
| **C3** | CI `spacetimedb-sdk` sibling-repo path-dep — workflow-trace cannot CI-green standalone-checkout | Phase 9 § 4 #12 + CHANGELOG [v0.1.0] | Three concrete recipe options in `.github/workflows/ci.yml` + `.gitlab-ci.yml` (Option A submodule / B vendor / C crates.io). DX-CI selects. |
| **C4** | m33 Security M0 default = Sandboxed (gate shape correct; per-workflow surface determination is v0.2.0) | Phase 9 § 4 #8 | Closed by R1 once W1 OR W2 lands; this row is the cross-reference. |
| **C5** | `wf-dispatch --execute` live-Conductor verification | Phase 9 § 4 #11 | Post-M0 dispatch soak. **Cannot be agent-closed** — Luke @ terminal runs Conductor bring-up (B3 / OP-1). v0.2.0 acceptance gate, not v0.2.0 work-item. |

### 2.6 ADR amendments required

| ADR | Amendment | Trigger |
|-----|-----------|---------|
| **D-S1002127-03** | Add NA-GAP-01 (V1) + NA-GAP-04 (V2) + NA-GAP-06 drain (C1) to the deferred-now-v0.2.0-active list | Phase 2 audit recommended this in S1004115; not yet authored |
| **D-S1004XXX-04 (NEW)** | RefusalToken authorship-typing as v0.2.0 substrate-as-actor primitive | V1 is structural enough to warrant its own ADR |
| **D-S1004XXX-05 (NEW, candidate)** | Substrate-mediated trust cross-habitat coordination | V5 — depends on Phase 4 DX-V5 (full v0.2.0 vs in-engine-receiver-only) |

## 3. Phase structure (12 phases)

Every phase: implementation → 4-stage gate → commit → mark complete. **No phase collapse.** Gate per phase, `${PIPESTATUS[0]}`-checked per stage (per Plan v2 §3 + feedback `pipestatus_for_gate_chains`):

```bash
CARGO_TARGET_DIR=./target cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic
CARGO_TARGET_DIR=./target cargo test --all-targets --all-features --release
```

### Phase 1 — Re-baseline + ADR amendments + v0.1.0 supersession sweep · ~0.5-1 day · *decision-free*

1. Re-baseline: `git show v0.1.0` + diff `v0.1.0..HEAD`; subtract anything already reconciled.
2. Amend ADR `D-S1002127-03` to register V1 (NA-GAP-01) + V2 (NA-GAP-04) + C1 (NA-GAP-06 drain) as now-active v0.2.0 work-items.
3. Author ADR `D-S1004XXX-04` for RefusalToken authorship-typing (V1 design spec).
4. Update `CHANGELOG.md` with `[v0.2.0-WIP]` entry pointing at this plan.
5. Update Honest-Residuals cross-references in `CLAUDE.md` + project `CLAUDE.local.md` + the v0.2.0 plan stub (this file's predecessor) supersession banner.
6. Commit: `docs(workflow-trace): Phase 1 — v0.2.0 re-baseline + ADR D-S1002127-03 amendment + RefusalToken ADR draft`.

### Phase 2 — Deep FP-verification + Tier 2 W1/W2 sizing revisit · ~0.5-1 day · *decision-free; feeds Phase 4*

1. Re-verify Phase 2 audit findings against `v0.1.0` HEAD: NA-GAP-01 spec-only still (no `RefusalToken` Rust type), NA-GAP-04 spec-only still, NA-GAP-06 drain absent still. Any drift = update inventory § 2.
2. Re-size W1 (escape_surface wire field, ~150-200 LOC) vs W2 (StepToken classification table, ~80-150 LOC) against shipped code. Catalogue downstream consumers of `WorkflowProposal.escape_surface` (if W1) or `EscapeSurfaceProfile::from_token()` (if W2).
3. Identify v0.1.0 modules whose tests will need fixture regeneration on W1 (JSONL bridge) — pre-flag the blast radius.
4. Pre-flight V3 (m16) Genesis Prompt v1.4 amendment shape — what does the 26 → 27 module-count touch in plan.toml + spec docs?
5. Commit: `docs(workflow-trace): Phase 2 — Tier 2 sizing + V3 module-count amendment pre-flight`.

### Phase 3 — Decision-free low-risk: A2 + C1 staging · ~0.5-1 day · *decision-free; parallel with Phases 1-2*

1. **A2 (SD9) m22 FeatureVector newtype** — purely cosmetic typed-newtype around `Vec<f64>`; no behaviour change. ~30-50 LOC + test.
2. **C1 NA-GAP-06 drain staging** — write a `drain` skeleton in m13 (private API; not yet a consumer) that the V1 RefusalToken work in Phase 7 can hook into. Includes outbox-pointer-tracking + idempotent replay test. ~80-120 LOC. **Stops short of wiring an external consumer** (that depends on V1 + V2).
3. Gate green per commit.
4. Commits: `feat(workflow-trace): SD9 m22 FeatureVector newtype` + `feat(workflow-trace): m13 outbox drain skeleton (NA-GAP-06 half)`.

### Phase 4 — Decision interview · ~2-3 h · *needs Luke*

Per gap NA-3 (operator-as-fatigue-budget; load-bearing questions get no defaults), interview is **split into Round A + Round B**.

**Round A — load-bearing (no defaults; each presented with full framing):**

- **DX-W** Wire-contract option for V1's Security verifier — W1 (escape_surface wire field, ~150-200 LOC, cross-binary serde) **or** W2 (StepToken classification table, ~80-150 LOC, no wire change)? Substrate frame asks: should the proposal carry its own surface declaration, or should the engine classify per-step? No default.
- **DX-V3** V3 m16 substrate-drift canary deployment shape — own module (Cluster E expansion, triggers Genesis Prompt v1.4 module-count amendment + Zen G7 re-audit) **or** distributed-canary participation across m9/m13/m32/m42 (preserves 26-module lock, no Genesis re-audit, but harder to test as a coherent organ)? No default.
- **DX-V5** V5 substrate-mediated trust scope — full cross-habitat coordination (touches stcortex + Conductor + atuin substrate-side schemas, depends on those repos accepting changes) **or** in-engine receiver-only (workflow-trace ships the consumer-side primitive, substrate-side trust remains opaque)? No default.

**Round B — mechanical / policy (defaults acceptable, each flagged):**

- **DX-1** V1 RefusalToken authorship variants — confirm `{SubstrateAuthored, EngineAuthored, OperatorAuthored, Unavailable}` 4-variant or amend (e.g., add `WatcherAuthored` for the m46-m51 lane).
- **DX-2** V2 substrate back-pressure signal shape — push (substrate emits to engine) **or** pull (engine probes substrate)?
- **DX-3** A3 (SD10) m22 empty-cluster behaviour — typed error, re-seed, or retain-prior (M0 shipped retain-prior; SD10 calls for error/re-seed)?
- **DX-4** Tier 4 SD8 m21 true Levenshtein source — add steps to `WorkflowProposal` (couples to W1/A4) **or** maintain a hash→steps lookup table (lookup-on-demand)?
- **DX-CI** CI `spacetimedb-sdk` resolution — Option A submodule, Option B vendor, Option C wait-for-crates.io?
- **DX-5** V4 substrate test fixture realism floor — full deterministic replicas of all 5 substrates **or** mock-only (struct-level fakes returning canned responses)?
- **DX-Mut** Mutation kill-rate target — hold ≥96.3 % (M0 bar) or raise (e.g., ≥98 %) since v0.2.0 ships a larger surface?
- **DX-Soak** Post-v0.2.0 soak duration — 24h (matches v0.1.0 OP-1) or longer (substrate-as-actor changes warrant a deeper observation window)?

*Naming (G2 directory rename): decided post-M0 by §15 D32; still post-v0.2.0 ship.*

### Phase 5 — Tier 2 wire-contract (W1 or W2) + W3 + W4 · ~3-5 days · *needs DX-W*

1. **W4** (smallest, decision-light) — add `CuratedBank::client_ref()` read-only accessor; tests for the seam shape. ~30-60 LOC. (Day 1.)
2. **W1 OR W2 per DX-W**:
   - **If W1:** add `escape_surface: EscapeSurfaceProfile` to `WorkflowProposal` + fallible `new()` signature update + accessor + serde regen + JSONL fixture regen across `m23 / m30 / m32 / orchestration` tests. ~150-200 LOC + test regen. (Days 2-3.)
   - **If W2:** new module `m25_step_token_classifier` (or co-locate in m23) carrying the StepToken → EscapeSurfaceProfile table + variant-aggregation max-fold function. ~80-150 LOC + table tests. (Day 2.)
3. **W3** — add `cost: i64` to `WorkflowProposal`; populate in `compose_proposals` from `step-count × mutation-weight` per D10; JSONL fixture regen. ~150-250 LOC. (Days 3-4.)
4. Re-run `cargo-mutants` scoped to `m23 + m30 + orchestration`; hold the DX-Mut bar.
5. One commit per W item: `feat(workflow-trace): Tier 2 Wx — …`. **Three commits minimum.**

### Phase 6 — Tier 3 real verifiers (R1 + R2 + R3) · ~3-4 days · *needs Phase 5 + DX-W*

1. **R1 Security real verifier** — replace M0 Sandboxed-default no-op with the live check against W1's `proposal.escape_surface` OR W2's `EscapeSurfaceProfile::from_token()` aggregation. Verdict semantic: hard-Refuse per D5/D6 (unchanged from M0). ~50-100 LOC. (Day 1.)
2. **R2 Cost real verifier** — implement D10 metric `step-count × mutation-weight`, threshold against config `cost_ceiling`. ~50-80 LOC. (Day 2.)
3. **R3 Consistency real verifier** — bank-conflict detection consuming `client_ref().contains_workflow_id()` or similar. Define what "conflict" means — overlapping variant_id, lineage-chain duplication, or both? Locks in Phase 4 if surfaced; otherwise local sub-decision. ~100-150 LOC. (Days 3-4.)
4. Re-run `cargo-mutants` scoped to `m33`; hold the bar. Phase 6 mutation run is multi-hour — budget it.
5. Commits: one per R item.

### Phase 7 — Tier 1A: V1 RefusalToken (KEYSTONE substrate-as-actor primitive) · ~2-3 days · *needs DX-1*

1. Define `RefusalToken` enum in a new module (`src/cross_cutting/refusal_token.rs` or co-locate in m32) with the 4 variants per DX-1.
2. Replace flat `RefusalReason` enum at `m32_dispatcher/mod.rs:163` with typed-by-authorship `RefusalToken`.
3. Thread through every refusal call-site: m9 namespace guard (substrate-authored when reducer rejects), m32 dispatcher (engine-authored), m13 outbox (substrate-authored on stcortex refuse-write), m40 Nexus emit (engine-authored on serialization fail), m41 LCM RPC (mixed), m42 stcortex emit (substrate-authored).
4. Wire the m13 drain skeleton (from Phase 3 C1) to consume RefusalToken-tagged events when it sees them.
5. Tests: ~80-150 new tests covering authorship-classification per call-site + round-trip with substrate-confirmable receipts (Phase 6f from M0).
6. Re-run `cargo-mutants` scoped to refusal paths.
7. Commit: `feat(workflow-trace): V1 — RefusalToken authorship-typing across refusal call-sites`.

### Phase 8 — Tier 1B: V2 substrate back-pressure budget · ~2-3 days · *needs DX-2*

1. **If DX-2 = push:** define a substrate-emit envelope (e.g., `BackPressureSignal { substrate, severity, observed_at_ms }`) + a substrate-side opt-in for each of the 5 substrates (stcortex, ORAC, synthex-v2, LCM, HABITAT-CONDUCTOR). Workflow-trace ships the receiver-only; substrate-side participation tracked in V5 cross-habitat ADR.
2. **If DX-2 = pull:** define a substrate-probe rhythm (e.g., m1 atuin probes its own write-latency every N reads; m13 stcortex probes refuse-write-rate). Honest Frame-A but at least bounded.
3. Wire the signal into m1 (atuin throttle), m13 (stcortex throttle), m32 (Conductor throttle) cadence-modulation.
4. Tests + scoped `cargo-mutants`.
5. Commit: `feat(workflow-trace): V2 — substrate back-pressure budget (per DX-2 mode)`.

### Phase 9 — Tier 1C: V3 m16 substrate-drift canary (KEYSTONE) · ~3-4 days · *needs DX-V3*

1. **If DX-V3 = own module:**
   - Author plan.toml + Genesis Prompt v1.4 amendment lifting module count 26 → 27.
   - Dispatch Zen G7 re-audit (cardinality drift) per D-S1002127-03 § 3 trigger.
   - Author `src/m16_substrate_drift_canary/` per spec at `ai_specs/cross-cutting/substrate-drift.md`; 5 clock samplers (m11 recency / m13 stcortex-decay / injection-TTL / atuin-checkpoint / stcortex-pathway-decay); agree-to-skew envelope per CC-5 documented crossings.
   - Emits `SubstrateDriftDetected` events through V1 RefusalToken-typed substrate-authored channel.
2. **If DX-V3 = distributed:**
   - No Genesis amendment; canary participation distributed across m9 (refusal-pattern observer), m13 (stcortex-decay observer), m32 (Conductor-enforcement observer), m42 (pathway-weight-drift observer).
   - Each participating module exports a `canary_observation()` accessor; a thin aggregator in m40 Nexus emits the unified `SubstrateDriftDetected` event.
3. Tests: ~40-80 covering each clock-pair crossing + agree-to-skew envelope boundaries + drift-event emission.
4. Re-run `cargo-mutants` scoped to the canary code path (whichever shape).
5. Commit(s): single if distributed; multi if own-module (one per clock sampler).

### Phase 10 — Tier 1D: V4 substrate test fixtures · ~3-4 days · *needs DX-5*

1. Author `tests/substrate_fixtures/` per ADR D-S1002127-03 § 1.b. **If DX-5 = full deterministic replicas:** local SQLite-backed stcortex stub, mock ORAC HTTP server, mock synthex-v2 WebSocket, etc. **If DX-5 = mock-only:** struct-level fakes returning canned responses.
2. Five named fixtures per ADR: `cr2_inflation_fixture` (stcortex returns pre-CR-2 magnitude), `refuse_write_no_consumer_fixture`, `hyphen_slug_reducer_fixture`, `conductor_enforcement_flag_off_fixture`, `atuin_wal_contention_fixture`.
3. Per-fixture integration test exercising the engine's response chain (typically through R1/R2/R3 verifiers + V3 canary).
4. ~80-150 fixture tests added; TEST_STRATEGY budget bumps from 1594 to ~1750-1800.
5. Commit: `test(workflow-trace): V4 — substrate test fixture suite`.

### Phase 11 — Tier 1E: V5 substrate-mediated trust + cross-habitat ADR · ~2-4 days · *needs DX-V5*

1. **If DX-V5 = full cross-habitat:**
   - Author ADR D-S1004XXX-05 covering substrate-side changes: stcortex consumer-trust score schema, Conductor dispatch-budget table, atuin read-quota daemon flag.
   - Workflow-trace ships the engine-side primitives: `SubstrateTrust { stcortex_score, conductor_budget_remaining, atuin_quota_remaining }` + accessors.
   - Pair-file the ADR for habitat-wide review at `~/projects/shared-context/agent-cross-talk/`; **does not gate v0.2.0 ship** (cross-habitat coordination is the v0.2.0 → post-v0.2.0 bridge).
2. **If DX-V5 = in-engine receiver-only:**
   - Ship the engine-side primitive (`SubstrateTrust` accessor); substrate-side participation is post-v0.2.0; substrate side returns `Unavailable` per V1 RefusalToken pattern.
3. Wire `SubstrateTrust` into R3 Consistency verifier (Phase 6) as an additional input axis if available.
4. Tests: trust-aware vs trust-unavailable verdict invariance + degradation-on-substrate-trust-loss.
5. Commit: `feat(workflow-trace): V5 — substrate-mediated trust (per DX-V5 shape)`.

### Phase 12 — Tier 4 algorithmic upgrades + integration + Zen audit + v0.2.0 ship · ~2-3 days

1. **A1 (SD8)** m21 true Levenshtein per DX-4 — implement the chosen source (steps-on-proposal coupling to A4 OR hash→steps lookup). Re-run scoped mutants. ~100-150 LOC.
2. **A3 (SD10)** m22 empty-cluster — implement DX-3 (typed error / re-seed / retain-prior). ~50-80 LOC.
3. **A4 (SD11)** m23 12-field proposal shape — co-lands with W1+W3 from Phase 5 (six new fields: per-step cost via W3, lineage_chain, generation_index, parent_proposal_id, lift_p95, and the existing six). ~50-80 LOC.
4. **Integration & end-to-end:**
   - Re-run full 4-stage gate + full `cargo-mutants` workspace-wide. Hold DX-Mut bar.
   - Update `wf-dispatch --dry-run` smoke against the new wire-contract shape.
   - DX-Soak duration soak: Watcher ☤ carries (per D36).
5. **Zen audit fold-in** — dispatch per Plan v2 §3 Phase 9 pattern. Verdict-absent → in-session zen agent substitute per D26 (habitat-native; no /ultrareview billing per D26).
6. **DX-CI fold:** apply chosen `spacetimedb-sdk` resolution to `.github/workflows/ci.yml` + `.gitlab-ci.yml`.
7. **Doc / persistence sweep:** `CHANGELOG.md` [v0.2.0] entry (mirrors [v0.1.0] structure — Added / Changed / Resolved / Honest residuals — § "Honest residuals" of v0.2.0 ideally is *empty* or short); reconcile `CLAUDE.md`, project `CLAUDE.local.md`, `GATE_STATE.md`, `ARCHITECTURE.md`; update `ai_docs/INDEX.md` + `ai_specs/INDEX.md`.
8. **Four-surface persist** with read-back-verify per Plan v2 § 13: ai_docs canonical + vault mirror + stcortex namespace `workflow_trace_v020_s1004XXX` + project CLAUDE.local.md anchor.
9. **Tag `v0.2.0`**; commit; push origin + gitlab.
10. **Operator hand-off:** OP-3 (post-v0.2.0 substrate soak) — Luke @ terminal exercises live substrates against the new V1-V5 surfaces; Watcher carries DX-Soak observation.

## 4. Sequencing & dependency graph

```
Phase 1 ─┐
Phase 2 ─┼─ decision-free, parallel ── Phase 4 (interview, informed by 1-3)
Phase 3 ─┘                                  │
                  ┌─────────────────────────┴───────────────┐
                  Phase 5 (Tier 2 W1/W2 + W3 + W4)
                                  ↓
                  Phase 6 (Tier 3 R1 + R2 + R3 — consumes Tier 2)
                                  ↓                  ↓
                              Phase 7 (V1)      (Phase 8 V2 — independent of V1)
                                  ↓
                              Phase 9 (V3 — consumes V1 for RefusalToken-typed drift events)
                                  ↓
                              Phase 10 (V4 fixtures — consumes V3 for canary-replica testing)
                                  ↓
                              Phase 11 (V5 — consumes V1 for refusal authorship in trust-unavailable case)
                                  ↓
                              Phase 12 (Tier 4 + integration + audit + ship)
```

**Critical path:** Phase 2 → Phase 4 → Phase 5 → Phase 6 → Phase 7 → Phase 9 → Phase 10 → Phase 12.

**Sequencing notes:**
- Phase 8 (V2) is **independent** of V1; can run in parallel with Phase 7 or after.
- Phase 11 (V5) depends on V1 (for the trust-unavailable `RefusalToken::Unavailable` case); cannot run before Phase 7.
- Phase 4 deliberately runs **after** Phases 1-3 (DAW-2 = full mirror; the gap analyses must have run by then per Plan v2 D44 pattern).
- **Tier 2 first** per DAW-1 — wire-contracts un-block Tier 3 cleanly; Tier 1 then consumes typed seams.

## 5. Effort roll-up *(honest ranges; bottom-up; range narrows after Phase 4 interview)*

| Phase | Effort | Driver of the range |
|-------|--------|---------------------|
| 1 — re-baseline + ADRs | ~0.5-1 day | ADR amendment density |
| 2 — FP-verification + Tier 2 sizing | ~0.5-1 day | how much W1/W2 sizing drift since Phase 2 audit |
| 3 — A2 + C1 staging | ~0.5-1 day | drain skeleton tests |
| 4 — interview | ~2-3 h | Luke availability + gap-analysis depth |
| 5 — Tier 2 W1/W2 + W3 + W4 | ~3-5 days | W1 vs W2 choice (±1 d); W3 fixture-regen blast radius |
| 6 — Tier 3 R1 + R2 + R3 | ~3-4 days | Consistency-verifier scope (DX-Mut hold + cargo-mutants ~hr/run) |
| 7 — V1 RefusalToken | ~2-3 days | new tests + scoped mutants on 7 call-sites |
| 8 — V2 back-pressure | ~2-3 days | push vs pull (DX-2) |
| 9 — V3 m16 canary (KEYSTONE) | ~3-4 days | own-module vs distributed (DX-V3) + Genesis amendment cascade if own-module |
| 10 — V4 fixtures | ~3-4 days | full-replica vs mock-only (DX-5) |
| 11 — V5 trust + cross-habitat ADR | ~2-4 days | full cross-habitat vs in-engine receiver only (DX-V5) |
| 12 — Tier 4 + integration + audit + ship | ~2-3 days | DX-Mut hold + DX-CI resolution + DX-Soak duration |
| **v0.2.0 total** | **~22-36 Claude-days** of effort (pre-interview range; narrows to ~22-30 once DX-V3/V5/W are locked) | + Luke / Zen / cross-habitat gating |

The stub catalogue estimate was `~6-10 Claude-days at god-tier` — that was top-down and substantially under-scoped (it counted ~LOC effort, not the full Plan v2 cycle overhead of ADRs, gap analyses, 4-surface persist, scoped mutants per phase, audit fold-in). The bottom-up number is **~22-36 Claude-days**; mid-point ~28. This is roughly double v0.1.0/M0's ~10-13 Claude-days — proportional to the LOC delta (~2200-2900 vs ~600).

## 6. Risk register

| Risk | Mitigation |
|------|------------|
| W1 wire-contract change breaks every JSONL fixture (S112 class) | Phase 2 enumerates the blast radius pre-DX-W; Phase 5 commits W1 and the fixture regen as a single landed unit, gate per commit |
| V3 m16 own-module triggers Genesis v1.4 + Zen G7 re-audit cascade (multi-day external dependency) | DX-V3 explicit; if "own module" chosen, Phase 9 starts with the Genesis amendment file and dispatches re-audit early so the audit runs in parallel with implementation |
| V5 cross-habitat coordination slips (substrate-side repos don't accept changes in v0.2.0 window) | DX-V5 includes the "in-engine receiver-only" fallback; v0.2.0 ships the engine half regardless; cross-habitat unification is post-v0.2.0 |
| V4 fixture realism doesn't actually catch the production drifts it's designed for (Frame-A trap NA-1 redux) | Each of the 5 named fixtures tests against the *exact* CR-2-class incident it was sourced from; fixture-design review folds in Phase 9 gap analysis on the fixture suite itself |
| Mutation kill-rate slips below DX-Mut bar with V1-V5 LOC delta | Scoped `cargo-mutants` per phase + full re-run Phase 12; if DX-Mut is raised to 98 %, budget for ≥2 mutation-fix cycles per Tier-1 phase |
| Agent over-claims a phase gate-clean (LCM P0-P4 drift × 6 + Armada FP-discovery) | Orchestrator re-runs full `--workspace --all-targets --all-features` gate + `git log -1` + independently exercises new code paths per phase (Plan v2 D42 discipline) |
| Tier-2-first ordering forces re-touching V1/V5 if substrate frame disagrees with wire-contract shape | DAW-1 was a Luke call; if the conventional gap analysis or NA gap analysis surfaces frame-collapse evidence, DX-W and the Tier sequence are revisited in Phase 4 |
| `RefusalToken` enum expansion (DX-1 amendment) cascades through 7 modules late | Phase 7 commits per call-site; refusal-token enum locked at Phase 7 step 1; subsequent variants additive only |
| v0.2.0 takes longer than v0.1.0; Luke fatigue / opportunity-cost shift to Master Plan v2 or Ember | Phase 4 DX-Soak + acceptance of ~22-36 Claude-day envelope is an explicit Luke decision; Plan v2 D45/D46 conviction-and-opportunity-cost questions reopen at v0.2.0 Phase 4 |
| Substrate-drift canary itself drifts (V3 self-canary problem) | V3 emits its own canary — m16 health pings stcortex / atuin / injection.db every cycle; if it fails to emit, the *absence* of canary events is itself a substrate-frame observable (m40 Nexus operator-visible alert) |

---

# PART B — Substrate-frame pass *(the genuine second authoring — Plan v2 §8/9 discipline)*

> Per CLAUDE.md Working Mode: *"write it once, then ask what frame is that? and write it again from the frame you didn't take. Both passes are the plan."* Part A is v0.2.0 *from the engine's frame* — the engine adds substrate-as-actor primitives. Part B re-authors v0.2.0 *from the substrate's frame* — atuin, stcortex, injection.db, HABITAT-CONDUCTOR, Luke. Part B will be **further re-authored** when na-gap-analyst dispatches in the next turn; this draft is the seed.

## 7. Re-authoring "complete" from the substrate frame

Part A's "complete" = *every primitive the engine needs to act as a substrate participant is shipped*. The substrate frame asks a different question: **does the substrate experience workflow-trace differently after v0.2.0?**

- **atuin (the read-side WAL substrate).** v0.1.0 added an engine-timed proxy for atuin's contention (Phase 8 step 3, honestly labelled Frame-A). v0.2.0 V2 (push or pull per DX-2) makes atuin's *own* write-latency a first-class signal. Substrate-frame test: atuin's foreground writers do not see slowdown delta > N ms during a workflow-trace read sweep. **Substrate experience changes.**
- **stcortex (the write-target substrate).** v0.1.0 added read-back-verify for engine writes (NA-GAP-06 § 13). v0.2.0 V1 (RefusalToken authorship) means stcortex's `RefuseWriteNoConsumer` becomes a substrate-authored event — the engine receives the refusal *as substrate speech*, not as engine inference. **Substrate's refusal voice is now audible.**
- **HABITAT-CONDUCTOR (the dispatch substrate).** v0.1.0 added enforcement-state assertion (NA-4 Phase 8 step 2). v0.2.0 V5 (substrate-mediated trust) means the Conductor's dispatch-budget becomes a verifier input — the engine asks Conductor *what it will accept* rather than presuming. **Substrate's consent state is now consulted.**
- **The CC-5 loop's clocks.** v0.1.0 enumerated the crossings (NA-5 Phase 8 step 4). v0.2.0 V3 (m16 canary) means clock-incoherence is detected, not just documented. **Substrate's temporal drift is now an event.**
- **Luke @ node 0.A (operator substrate).** v0.1.0 made the Phase 4 interview no-defaults on load-bearing questions (NA-3). v0.2.0 does the same — and adds DX-V3/DX-V5 to Round A because those are substrate-shape decisions, not mechanism choices. **Operator's attention budget is again first-class.**

**The re-authored conclusion:** v0.2.0 is the milestone at which the substrate stops being a thing the engine queries and starts being a thing the engine *converses with*. Each substrate has at least one channel through which it speaks back (refusal-authored events, back-pressure signals, dispatch-budget, drift events). Whether that conversation is meaningful depends on whether the substrate-side primitives ship — which is exactly the cross-habitat dependency V5 names.

## 8. What `v0.2.0` certifies — and does not *(the named cut)*

> **`v0.2.0` certifies engine + substrate co-completeness:** every primitive the engine needs to *participate in* (not *consume*) the substrate is shipped, tested, audited, documented. The engine has authorship-typed refusal channels, substrate-back-pressure receivers, a substrate-drift canary, substrate test fixtures, and substrate-mediated trust hooks. It does **not** certify that the substrate-side schemas / daemons / consumer-trust tables exist in stcortex, ORAC, synthex-v2, LCM, HABITAT-CONDUCTOR — that is **post-v0.2.0 cross-habitat coordination** (V5's ADR is the seed; live unification is its own milestone).

If the cross-habitat work never happens, v0.2.0's substrate-as-actor primitives degrade gracefully via V1 RefusalToken `::Unavailable` and remain useful as engine-side defensive instrumentation. The substrate frame doesn't gain a voice; the engine at least has the *capacity* to hear one.

The NA discipline asks: if that paragraph is uncomfortable, that is the finding. It is uncomfortable. v0.2.0 is honestly half-substrate — the engine half — and the named cut is the cross-habitat coordination boundary. DX-V5 ratifies the boundary or pulls forward.

## 9. Frame-collapse self-check *(the recursion step — gap NA-9)*

Interrogating Part B itself — *is this the substrate's frame, or the engine's model of it?*

- **V2 "substrate emits back-pressure" (Phase 8 push mode) is still the substrate doing what the engine asked it to do.** A true substrate-frame mechanism has the substrate emit back-pressure on its own behalf because back-pressure is *its* signal. v0.2.0's V2 makes the receiver real; the substrate-side emitter is opt-in per substrate. **Honest label: V2's engine half is Frame-B; substrate emitter participation is per-substrate Frame-B opt-in.** The pull mode is more honestly Frame-A.
- **V3 m16 canary is the engine sampling the substrates' clocks.** That is the engine measuring substrate state. A truly substrate-frame canary would have *each substrate* announce its own clock health. v0.2.0's V3 is the *aggregator*; per-substrate canary participation is V5-ADR-cross-habitat territory. **Honest label: V3 is engine-side aggregation of substrate-observable state; the substrate-emitted version is post-v0.2.0.**
- **V5 substrate-mediated trust depends on substrate-side changes the engine cannot author.** v0.2.0's DX-V5 honestly partitions: in-engine receiver-only is Frame-A defensive (the engine pretends to hear); full cross-habitat is the start of Frame-B (substrates actually speak). **Honest label: V5 ships the listening side; the speaking side is a cross-habitat ratification gate.**
- **Does Part B re-author or annotate?** It re-authors the *certification criterion* (§ 7-8) and the re-authoring changes the plan: DX-V5 in Round A no-defaults, V5's in-engine-receiver-only fallback as a first-class option, the named cross-habitat-coordination cut. The phase count's spine — Tier 2 → Tier 3 → Tier 1 → Tier 4 — is unchanged because the engine-side work is real and needs doing regardless. That is correct and not a collapse: the engine *needs* refusal-typing and back-pressure-receiving and canary-aggregating to be Frame-B-ready.

**Self-check verdict:** Part B is a genuine second pass on the *certification criterion*. Three primitives (V2 emitter, V3 canary, V5 trust speaker) are explicitly flagged as substrate-side dependencies that don't fully ship in v0.2.0. No collapse is concealed. The honest sentence — "v0.2.0 is half-substrate" — is in § 8 by design.

## 10. Frame tensions — explicit reconciliations

- **T-1 — "engine is done after V1-V5" vs "the substrate has not yet spoken back."** Reconciled by § 8's named cross-habitat cut; ratified at DX-V5.
- **T-2 — "DAW-1 Tier-2-first is the right engineering ordering" vs "the substrate frame asks for substrate primitives first because they define the contract the wire encodes."** Tension is real. Reconciled by Phase 4 reopening the Tier order if the gap analyses surface frame-collapse evidence; Luke chose Tier-2-first with eyes open. If the NA gap analyst returns "this is Frame collapse", DX-W escalates to "revisit DAW-1 first."
- **T-3 — "DX-V3 distributed canary preserves the 26-module lock" vs "the substrate-drift detection function deserves an organ, not capillaries."** Reconciled by DX-V3 being a no-default Round A question; either choice is defensible; the choice itself is the substrate-frame ratification.

---

# PART C

## 11. Post-v0.2.0 partition *(named as a milestone choice)*

What v0.2.0 does **not** close, sized for awareness:

- **Cross-habitat substrate-mediated trust full unification** — stcortex consumer-trust schema, Conductor dispatch-budget table, atuin read-quota daemon flag, ORAC reputation hooks, synthex-v2 r13-state-aware verifier weighting. Estimated ~30-60 Claude-days of cross-habitat coordination + per-substrate-repo PRs. Cross-habitat ADR (V5 in v0.2.0) is the seed; live unification is a habitat-wide milestone.
- **m16 canary substrate-side emitter participation** — each substrate (stcortex, atuin, injection.db) ships its own clock-health-emit so V3's distributed/own-module aggregator becomes Frame-B all the way down. Post-v0.2.0.
- **V2 push-mode substrate-side emitters** — each substrate (atuin, ORAC, etc.) ships back-pressure emission on its own behalf. Post-v0.2.0.
- **`wf-dispatch --execute` live-Conductor 24h+ soak** — post-v0.2.0 dispatch soak (carries DX-Soak from this plan).
- **Genesis Prompt v1.4 module-count amendment if DX-V3 = distributed** — preserved for the cross-habitat Frame-B turn.

These are named as a *frame choice* (NA-7 discipline): every post-v0.2.0 item above is a substrate-side primitive the engine cannot author alone. The cut is principled, not arbitrary.

## 12. Gap-analysis disposition *(placeholder — fills after dual-frame gap analyses run)*

| Finding | Disposition in v1 → v2 |
|---------|------------------------|
| *Conventional gap analysis findings* | TBD — dispatch pending |
| *NA gap analysis findings* | TBD — dispatch pending |
| *Frame tensions surfaced* | TBD |
| *Convergent findings* | TBD |

To be filled in v2 after `na-gap-analyst` + a conventional gap-analysis agent return. Every finding gets one of: ACCEPTED-FOLDED-IN / ACCEPTED-WITH-HONEST-LABEL / REJECTED-WITH-REASON. No silent drops.

## 13. Persistence — four surfaces (this plan)

| Surface | Location | Verify |
|---------|----------|--------|
| ai_docs canonical | this file (v1) → `WORKFLOW_TRACE_V020_PLAN_S1004XXX.md` (v2 after interview, where XXX is the v2-authoring session id) + 2 gap-analysis docs (next turn) | git |
| Obsidian vault mirror | `the-workflow-engine-vault/Workflow-Trace v0.2.0 Plan S1004XXX.md` (NEW at v2) | file exists |
| stcortex | ns `workflow_trace_v020_s1004XXX` — genesis memory + bidi pathways to `workflow_trace_completion_s1004115` (v0.1.0 ship) + `workflow_trace_hardening_2026_05_21` (S1003733) | `SELECT id` read-back; absence = failure per Plan v2 NA-6 |
| CLAUDE.local.md anchor | project `CLAUDE.local.md` — "v0.2.0 IN FLIGHT (planning)" pointer row added next turn after v1 + gap analyses; "v0.2.0 IN FLIGHT (execution)" block on Phase 1 start; supersedes "v0.1.0 / M0 SHIPPED" / "v0.1.1 + v0.2.0 PREP" blocks once Phase 12 lands | git |
| CHANGELOG | `[v0.2.0-WIP]` entry on Phase 1 (Plan v2 D44 pattern) → `[v0.2.0]` on Phase 12 ship | git |
| injection.db tracking | causal_chain `workflow_trace_v020_planning_s1004377` (v1 authoring) → `workflow_trace_v020_execution_s1004XXX` (Phase 1+) | sqlite |
| git tag | `v0.2.0` annotated at Phase 12 ship | git |

This v1 itself is **single-surface (ai_docs only)** until next turn. That is intentional — gap analyses fold in before any 4-surface persist so the persisted plan is the integrated v2, not the un-audited v1.

## 14. Status — v1 DRAFT; gap analyses + interview pending

- ✅ DAW-1 (Tier 2 first) + DAW-2 (full mirror) locked at draft-authorship by Luke.
- ⏳ Conventional gap analysis: PENDING (next turn dispatch).
- ⏳ NA gap analysis: PENDING (next turn dispatch).
- ⏳ Phase 4 interview: PENDING (after gap analyses fold in; expected ~10-15 questions across Round A + Round B + whatever the gap analyses surface).
- ⏳ v2 authoring (integrating gap-analysis findings + interview-locked decisions): PENDING.
- ⏳ 4-surface persist of v2: PENDING.
- ⏳ Luke @ node 0.A "start Phase 1" go: PENDING (per Plan v2 D48 — execution is a separate authorisation from plan approval).

**The remaining gates:** (1) gap-analysis fold-in → v2, (2) interview lock-in, (3) v2 4-surface persist, (4) Luke "start Phase 1" go. None are this turn.

## 15. Phase 4 interview — DRAFT seed questions (expand after gap analyses)

### Round A — load-bearing, no defaults

- **DX-W** Wire-contract option for V1's Security verifier — W1 (escape_surface wire field) **or** W2 (StepToken classification table)?
- **DX-V3** V3 m16 deployment shape — own module (Genesis v1.4 + Zen re-audit) **or** distributed-canary?
- **DX-V5** V5 substrate-mediated trust scope — full cross-habitat **or** in-engine receiver-only?

### Round B — mechanical / policy, defaults acceptable

- **DX-1** V1 RefusalToken variants — confirm 4-variant or amend?
- **DX-2** V2 back-pressure signal shape — push or pull?
- **DX-3** A3 m22 empty-cluster — typed error, re-seed, or retain-prior?
- **DX-4** A1 m21 Levenshtein source — steps-on-proposal or hash→steps lookup?
- **DX-CI** spacetimedb-sdk — submodule, vendor, or wait-for-crates.io?
- **DX-5** V4 fixture realism — full replicas or mock-only?
- **DX-Mut** Mutation kill-rate target — hold ≥96.3 % or raise?
- **DX-Soak** Post-v0.2.0 soak duration — 24h or longer?

### Pending questions (to be surfaced by gap analyses)

The Plan v2 interview was 12 rounds / 48 questions; v0.2.0's pre-gap-analysis seed is 11 questions. Gap analyses will likely surface 8-16 more across topics like: substrate-side error path coverage (NA frame), refusal-token observability hooks (operator-as-substrate), substrate-fixture realism floor (V4 cross-checks), V3 alert-fatigue mitigation, cross-habitat coordination commitment device.

---

## 16. Operator hand-off (Phase 12 step 10 — post-v0.2.0)

- **OP-3** — Live substrate soak: Luke @ terminal exercises live substrates against the new V1-V5 surfaces; Watcher ☤ carries DX-Soak observation duration (24h or longer per Phase 4).
- **OP-4** — Cross-habitat ADR D-S1004XXX-05 review: if DX-V5 = full cross-habitat, the ADR is pair-filed; substrate-side changes in stcortex / ORAC / atuin / Conductor / synthex-v2 are post-v0.2.0 work-items for those repos.
- **OP-5** — Master Plan v2 / Ember opportunity-cost reopen: per Plan v2 D46, after v0.2.0 ships, the workflow-trace lane is complete-to-milestone and the conviction question reopens.

---

*Plan v1 authored S1004377 · 2026-05-23 · Claude @ cortex · single-frame draft pending dual-frame gap analyses + Phase 4 interview · DRAFT — supersedes the v0.2.0 stub catalogue · v2 follows after gap analyses + interview lock-in · execution awaiting Luke @ node 0.A "start Phase 1" go.*
