# Workflow-Trace v0.2.0 Plan **v2** — S1004377 (substrate-safety milestone, dual-frame, gap-analysis-corrected)

> **Authored:** 2026-05-23 (S1004377) · **Status:** **PLAN v2 — decisions LOCKED (§15) 2026-05-23 (S1004377); awaiting Luke @ node 0.A "start Phase 1" go** (separate authorisation per Plan v2 D48)
> **Supersedes:** [`WORKFLOW_TRACE_V020_PLAN_S1004377.md`](WORKFLOW_TRACE_V020_PLAN_S1004377.md) (v1 DRAFT) · v1 itself superseded the [`WORKFLOW_TRACE_V020_PLAN_STUB_S1004115.md`](WORKFLOW_TRACE_V020_PLAN_STUB_S1004115.md) scope catalogue.
> **Re-baselined on:** git HEAD `81910a6` (v1 + gap analyses commit) over `d521a00` over `54cb4e2` over `7d16c2c` over `v0.1.0` tag at `df00fd2`.
> **Corrected by:** the dual-frame gap analysis — [`…_CONVENTIONAL_GAP_ANALYSIS.md`](WORKFLOW_TRACE_V020_PLAN_S1004377_CONVENTIONAL_GAP_ANALYSIS.md) (13 findings — 4 HIGH / 7 MED / 2 LOW) + [`…_NA_GAP_ANALYSIS.md`](WORKFLOW_TRACE_V020_PLAN_S1004377_NA_GAP_ANALYSIS.md) (12 findings + 3 load-bearing tensions + 4 convergent). Every finding's disposition is in §12.
> **Authorisation chain:** v0.2.0 plan-authoring authorised by Luke @ node 0.A in S1004377 per §15 D40 invocation-only cadence + Plan v2 §14 single-remaining-gate discipline. **DAW-1 (Tier 2 first) is RE-OPENED as Phase 4 Round-A question DX-DAW-1 per NA-1 + C-1 convergent.**
> **Back to:** [CLAUDE.local.md](../CLAUDE.local.md) · [CLAUDE.md](../CLAUDE.md) · [`WORKFLOW_TRACE_COMPLETION_PLAN_V2_S1004115.md`](WORKFLOW_TRACE_COMPLETION_PLAN_V2_S1004115.md) · [`PHASE9_SD_RECONCILIATION_S1004115.md`](PHASE9_SD_RECONCILIATION_S1004115.md) · [`decisions/2026-05-17-substrate-as-actor-deferrals.md`](decisions/2026-05-17-substrate-as-actor-deferrals.md) (ADR D-S1002127-03 — to be amended in Phase 1) · gap analyses above

---

## What changed from v1 (gap-analysis corrections)

| v1 flaw | v2 fix |
|---------|--------|
| §2 V1 row cited `RefusalReason` at `m32_dispatcher/mod.rs:163`; enum is actually at `:228` (C-1, propagated from Phase 2 audit) | All §2 file:line citations re-verified at HEAD `81910a6`; Phase 1 expanded to "FP-verify every anchor" |
| V1 RefusalToken in Phase 7 — structurally couples to W1/SD11 wire bump in Phase 5; pays JSONL fixture regen twice (C-2 HIGH) | **V1 co-lands with W1 in Phase 5** (option (a) per C-2); Phase 7 shrinks to m9/m32/m41 call-site threading + m13-drain wire; one regen pass |
| DX-W binary frame "W1 or W2" silently retires option (iii) M0 audit-overlay; bundles 2 hidden sub-decisions (C-3 HIGH) | DX-W **split into DX-W.a / DX-W.b / DX-W.c** in §15 Round A; no silent sub-decisions |
| Phase 9 DX-V3=own-module Zen G7 re-audit cascade invisible in `~3-4 day` estimate (C-4 HIGH) | Phase 9 estimate **forks by DX-V3**: distributed = ~2-3 d; own-module = **~5-12 d** (Genesis v1.4 + Zen wait + impl); add **DX-V3.b** "Zen silent N days → ship or hold?" |
| Phase 5 W3 `mutation-weight` source unverified — `grep` returns zero in `src/` (C-5 HIGH) | Phase 1 step adds "pin `mutation-weight` source"; **DX-W3.src** added to Round A (variant.mutation count / new table / m11 fitness pull-through) |
| Phase 6 R3 conflict semantic depends on A4 SD11 shape that ships 6 phases later (C-6) | **A4 SD11 moves to Phase 5** (co-lands with W1+W3 wire-contract addition); R3 in Phase 6 consumes 12-field shape unambiguously; **DX-R3** added to Round B |
| Effort `~22-36 Claude-days` under-counts Plan-v2 round-trip overhead by ~6-8 d (C-7) | **§5 raised to ~25-42 Claude-days mid-point ~33**; three new line items (plan-arc round-trip / interview latency / cumulative mutation wall time) |
| ADR D-S1002127-03 amendment treated as doc edit; it's a language-change cascade across deferral surfaces (C-8) | Phase 1 step 2.5 added: "search every doc referencing D-S1002127-03 deferral language"; Phase 1 bumped to ~1-1.5 d |
| Risk register missed 3 classes: V5 substrate-side schema-drift / V3 alert-fatigue / mutation-gate budget blowout (C-9) | §6 gains **3 new rows** (V5 versioned ADR + serde fallback; V3 rate-limited alerts + dedup; mutation-cap per phase) |
| Round B had 3 already-defaulted slots; Round A missed 3 real decisions (C-10) | **DX-Mut / DX-Soak / DX-1-mechanism demoted to "stated defaults" in §14**; **DX-A4-coupling / DX-CI / DX-MGB promoted to Round A** |
| Part B §7-9 annotates more than re-authors; spine preserved without Frame-B test (C-11 + NA-9 + convergent C-2) | **§9 gains recursion sub-section** interrogating §9 itself + §7's substrate set + the certification sentence + the Tier-order spine choice |
| §7 substrate enumeration missing RALPH, Watcher ☤, Cargo build graph (NA-2 + convergent C-3) | **§7 expanded to 7 substrates** (atuin / stcortex / Conductor / CC-5 clocks / Luke + Watcher + RALPH + Cargo build graph); each gets Part-B paragraph |
| §13 single-surface persistence has no recovery contract; v1 lost on context flip (C-12) | **Closed in v1 → v2 round** by adding `CLAUDE.local.md` "v0.2.0 IN FLIGHT (PLANNING)" pointer (commit `81910a6`); §13 records the lesson |
| C5 `wf-dispatch --execute` acceptance gate had no defined pass criteria (C-13) | **Phase 12 step 10 gains 3-row acceptance criteria table** (`--execute` round-trip / V1-V5 primitive exercise / soak duration) with named failure handling |
| DAW-1 Tier-2-first locked pre-analysis without substrate-frame defence (NA-1 HIGH, convergent C-1) | **DAW-1 RE-OPENED as DX-DAW-1 in §15 Round A** with both substrate-frame and engineering-frame framings produced in the question itself |
| Round B DX-1/DX-2/DX-5 were substrate-shape questions misclassified as mechanism choices (NA-3 HIGH) | **All three promoted to Round A** with no-default framing; each split into substrate-shape ↑ mechanism sub-questions where applicable |
| V3 self-canary mitigation requires substrate participation v0.2.0 doesn't ship (NA-4 HIGH) | §6 V3 risk row revised: **Watcher's deployment-watch journal asserts m16 heartbeat liveness**; missing heartbeat for N cycles = substrate-emitted alert; pairs with V5 SubstrateTrust |
| V5 in-engine receiver-only emits `RefusalToken::Unavailable` audit-indistinguishable from substrate-authored (NA-5 HIGH + T-4) | **`RefusalToken::Unavailable` gains sub-tag** `(EngineImagined / SubstrateUnreachable / SubstrateAuthored)`; `substrate_participation_status` accessor added to V5; `DX-V5.b` in §15 Round A |
| §1 + §8 certification "engine + substrate co-completeness" is Frame-A in substrate vocabulary (NA-10 + T-3) | **Re-labelled to "engine-side substrate-participation readiness"**; honest one-sided naming |
| §11 cross-habitat estimate ~30-60 Claude-days ignores substrate-side consent gradient (NA-6) | §11 gains **per-substrate consent table** (stcortex HIGH / 1-2 wk; Conductor HIGH / 1-2 wk; atuin UNKNOWN / indeterminate; ORAC MED / 2-4 wk; synthex-v2 HIGH / 1-2 wk) |
| DX-2 push/pull treated as binary global; substrate landscape is gradient (NA-8) | **DX-2 reshaped to per-substrate `SubstrateBackPressureMode`** enum keyed by substrate-id; default = pull; per-substrate flip to push as emitters ship |
| §13 inherits Plan v2 read-back-verify but not slug-discipline trap (hyphens) (NA-11 + convergent C-4) | **§13 gains slug-discipline note** (S1001757 munge bug; `grep '-' <slug>` must return no match) |

---

# PART A — Conventional plan

## 1. Mission

The 26-module codebase is at **v0.1.0 / M0** (2048 tests passing, clippy + pedantic clean, mutation kill-rate 96.3% held). v0.1.0 certified *engine-internal completeness only* — every residual the engine owned was closed, tested, audited, documented. v0.1.0 explicitly did **not** certify the engine's safety as a substrate-facing organ.

**v0.2.0 closes that gap on the engine's side.** It adds substrate-drift detection, substrate-side test fixtures, substrate-mediated trust hooks, authorship-typed refusal, substrate back-pressure receivers, the wire-contract changes that un-block live m33 verifiers, and the Class-C algorithmic upgrades deferred by Phase 9. Per §15 D4 v0.2.0 was a committed follow-on; per §15 D40 the cadence is invocation-only; this v2 is the invocation.

**What `v0.2.0` certifies — the honest, one-sided cut (NA-10 + T-3 re-labelling):** *engine-side substrate-participation readiness*. Every primitive the engine needs to *participate in* (not *consume*) the substrate is shipped, tested, audited, documented. It does **not** certify substrate-side primitives — those are tracked separately in §11's post-v0.2.0 partition with explicit per-substrate consent-gradient estimates. The tag tells the habitat "the engine is ready to be a substrate participant"; it does not tell the habitat "the substrates have agreed to participate." That is the post-v0.2.0 cross-habitat work, sized honestly in §11.

## 2. Verified inventory — v0.1.0 honest residuals → v0.2.0 work (re-verified at HEAD `81910a6`)

### 2.1 Tier 1 — substrate-as-actor primitives

| ID | Item | File:line evidence (re-verified) | Notes |
|----|------|----------------------------------|-------|
| **V1** | `RefusalToken` authorship-typed enum (`SubstrateAuthored / EngineAuthored / OperatorAuthored / Unavailable(EngineImagined\|SubstrateUnreachable\|SubstrateAuthored)`) wired through m9/m32/m13/m40/m41/m42/m33 | replaces flat `RefusalReason` at `src/m32_dispatcher/mod.rs:228` (**corrected from v1's `:163`**); `WatcherAuthored` variant subject to DX-1 | **CO-LANDS with W1 in Phase 5** per C-2; requires new ADR D-S1004XXX-04 |
| **V2** | Substrate back-pressure budget — per-substrate `SubstrateBackPressureMode` enum (per NA-8) replacing v1's global push/pull | new `src/cross_cutting/back_pressure.rs`; receivers in m1 / m13 / m32 | DX-2 reshaped from binary to per-substrate selection |
| **V3** | `m16_substrate_drift_canary` — periodic prober + `Watcher m16 heartbeat liveness` substrate observer (NA-4) | new `src/m16_substrate_drift_canary/` (if own-module) OR distributed across m9/m13/m32/m42; **DX-V3** sets shape, **DX-V3.b** sets Zen-silent escalation |
| **V4** | Substrate test fixtures (`tests/substrate_fixtures/`) — 5 named fixtures + V3-canary-failure fixture | per ADR D-S1002127-03 §1.b | DX-5 sets realism floor |
| **V5** | Substrate-mediated trust + `substrate_participation_status: enum { NotShipped, Shipping, Live }` accessor (NA-5) | new `src/cross_cutting/substrate_trust.rs`; consumed by R3 if available | DX-V5 + DX-V5.b set scope + Unavailable sub-tagging |

### 2.2 Tier 2 — engine-side wire-contract changes

| ID | Item | Sizing | Notes |
|----|------|--------|-------|
| **W1** | `WorkflowProposal::escape_surface: EscapeSurfaceProfile` wire-contract field — option (i) | ~150-200 LOC + JSONL fixture regen across `tests/wf_crystallise_integration.rs`, `tests/wf_dispatch_integration.rs`, cc5 substrate cycle, cc7 pressure evolution | DX-W.b chooses W1 vs W2; **co-lands with V1 in Phase 5** per C-2 |
| **W2** | OR `StepToken → EscapeSurfaceProfile` classification table + variant aggregation — option (ii) | ~80-150 LOC + table tests; DX-W.b chooses W2 vs W1 | No wire-contract change; bag-of-tokens vs ordered-prefix is DX-W.b sub-decision |
| **W3** | `WorkflowProposal::cost: i64` wire-contract field — un-blocks D9 Cost real verifier | ~150-250 LOC (range conditional on DX-W3.src for `mutation-weight`) | mutation-weight source TBD per C-5; DX-W3.src in Round A |
| **W4** | `CuratedBank::client_ref()` accessor seam — un-blocks D11 Consistency real verifier | ~30-60 LOC | Smallest; decision-free; Phase 5 day 1 |

### 2.3 Tier 3 — real verifier impls (consume Tier 2)

| ID | Item | Source | Notes |
|----|------|--------|-------|
| **R1** | m33 Security real verifier — consumes W1 OR W2 per DX-W.b; verdict semantic hard-Refuse per D5/D6 | Phase 6a + Phase 2 audit | Replaces M0 Sandboxed-default no-op |
| **R2** | m33 Cost real verifier per D10 metric `step-count × mutation-weight` — consumes W3 | §15 D9 + D10 | ~50-80 LOC; `mutation-weight` source pinned via DX-W3.src |
| **R3** | m33 Consistency real verifier — bank-conflict detection consuming W4 + A4 12-field shape | §15 D11 + DX-R3 | DX-R3 sets `variant_id-only` / `lineage-chain-only` / `both`; default = `variant_id-only` |

### 2.4 Tier 4 — algorithmic / shape upgrades (Class-C SD drifts)

| ID | Item | Source | Notes |
|----|------|--------|-------|
| **A1** | SD8 — m21 true Levenshtein over source pattern's steps | §15 D27 + Phase 9 § 4 | DX-4 sets source (steps-on-proposal coupling to A4/W3 OR hash→steps lookup) |
| **A2** | SD9 — m22 typed `FeatureVector` newtype wrap | §15 D27 + Phase 9 § 4 | Decision-free; lands in Phase 3 |
| **A3** | SD10 — m22 empty-cluster behaviour | §15 D27 + Phase 9 § 4 | DX-3 sets typed error / re-seed / retain-prior |
| **A4** | SD11 — m23 12-field proposal shape (per-step cost via W3, lineage_chain, generation_index, parent_proposal_id, lift_p95, diversity_cluster) | §15 D27 + Phase 9 § 4 + C-6 | **MOVES TO PHASE 5** co-lands with W1+W3 (per C-6) |

### 2.5 Carry-overs from v0.1.0 Honest Residuals

| ID | Item | Status |
|----|------|--------|
| **C1** | NA-GAP-06 outbox drain half | Phase 3 stages drain skeleton (decision-free); Phase 5 wires consumer via V1 (folded) |
| **C2** | m31 production caller `\|_w\| 0.0` (Phase 5 nit A2) | ✅ CLOSED 2026-05-23 v0.1.1 round (`7d16c2c`) |
| **C3** | CI `spacetimedb-sdk` sibling-repo path-dep | **DX-CI promoted to Round A per C-10**; substrate-coupling framing per NA-2 |
| **C4** | m33 Security M0 default = Sandboxed | Closed by R1 once W1/W2 lands per DX-W.b |
| **C5** | `wf-dispatch --execute` live-Conductor verification | **Phase 12 step 10 gains 3-row acceptance-criteria table** per C-13 (`--execute` round-trip / V1-V5 primitive exercise / soak duration) |

### 2.6 ADR amendments required

| ADR | Amendment | Effort |
|-----|-----------|--------|
| **D-S1002127-03** | Add NA-GAP-01 (V1) + NA-GAP-04 (V2) + NA-GAP-06 drain (C1) to now-active list; **plus language-change cascade across every doc referencing deferral language** per C-8 step 2.5 | ~0.5 day (cascade is real) |
| **D-S1004XXX-04 (NEW)** | RefusalToken authorship-typing — including `Unavailable(EngineImagined/SubstrateUnreachable/SubstrateAuthored)` sub-tagging per NA-5 | ~0.25 day |
| **D-S1004XXX-05 (NEW, conditional)** | Substrate-mediated trust cross-habitat coordination (if DX-V5 = full cross-habitat) | ~0.5 day + cross-habitat pair-file |

## 3. Phase structure (12 phases — Tier-2-first ordering CONDITIONAL on DX-DAW-1 outcome)

**v2 NOTE:** the phase numbering below assumes DX-DAW-1 confirms Tier-2-first. If DX-DAW-1 = Tier-1-first, the phase sequence inverts (Phase 7 V1 → Phase 5 W1 etc.); the §15 interview owns that swap. Effort estimates carry over either way.

Every phase: implementation → 4-stage gate → commit → mark complete. **No phase collapse.** Gate per phase, `${PIPESTATUS[0]}`-checked per stage (feedback `pipestatus_for_gate_chains`):

```bash
CARGO_TARGET_DIR=./target cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic
CARGO_TARGET_DIR=./target cargo test --all-targets --all-features --release
```

### Phase 1 — Re-baseline + ADR amendment cascade + file:line re-verify + `mutation-weight` pin · ~1-1.5 day · *decision-free*

1. Re-baseline: `git show v0.1.0 --stat`; `git log v0.1.0..HEAD`.
2. **Amend ADR `D-S1002127-03`** to register V1 + V2 + C1 as now-active. **Plus the cascade per C-8 step 2.5:** `grep -rn "D-S1002127-03" .` and update every referencing doc — `ai_specs/substrate-couplings/`, `CHANGELOG.md` `[v0.1.0]` § "Honest residuals", Plan v2 § 11, etc. Deferral language → now-active OR cross-reference to amendment.
2.5. **FP-verify every file:line in §2 against shipped tree** per C-1 recommendation. Produce a residual-drift report.
3. **Pin `mutation-weight` source** per C-5: `grep -rn "mutation_weight\|mutation-weight" src/` → confirm zero hits; decide source candidate (variant.mutation count / new StepToken table / m11 fitness pull-through) and document the decision Phase 4 will ratify via DX-W3.src.
4. Author ADR `D-S1004XXX-04` for RefusalToken authorship-typing (V1 design spec).
5. Update `CHANGELOG.md` with `[v0.2.0-WIP]` entry pointing at this plan.
6. Update Honest-Residuals cross-references in `CLAUDE.md` + project `CLAUDE.local.md` + v1 stub supersession banner.
7. **DOC-1** Read-back-verified stcortex memory `workflow_trace_v020_s1004377` genesis (slug-disciplined: underscores only per NA-11).
8. Commit: `docs(workflow-trace): Phase 1 — v0.2.0 re-baseline + ADR D-S1002127-03 amendment cascade + file:line re-verify + mutation-weight pin + RefusalToken ADR draft`.

### Phase 2 — Deep FP-verification + Tier 2 W1/W2 sizing revisit + substrate-enumeration audit · ~0.5-1 day · *decision-free; feeds Phase 4*

1. Re-verify Phase 2 audit (S1004115) findings against `v0.1.0` HEAD: NA-GAP-01 still spec-only, NA-GAP-04 still spec-only, NA-GAP-06 drain still absent.
2. Re-size W1 (escape_surface wire field, ~150-200 LOC) vs W2 (StepToken classification table, ~80-150 LOC) against shipped code. Catalogue downstream consumers; pre-flag JSONL fixture regen blast radius for W1.
3. **Substrate-enumeration audit per NA-2 + convergent C-3:** verify every substrate in §7 (atuin / stcortex / Conductor / CC-5 clocks / Luke + Watcher ☤ + RALPH + Cargo build graph) against runtime evidence. Produce per-substrate dossier referencing live state.
4. Identify v0.1.0 modules whose tests will need fixture regeneration on W1 + V1 co-land.
5. Pre-flight V3 (m16) Genesis Prompt v1.4 amendment shape; FP-verify Zen G7 re-audit latency from cross-talk dir (per C-4 — zero `zen_*verdict*` files for any workflow-trace wave).
6. Commit: `docs(workflow-trace): Phase 2 — Tier 2 sizing + V3 module-count amendment pre-flight + substrate-enumeration audit`.

### Phase 3 — Decision-free low-risk: A2 + C1 staging · ~0.5-1 day · *decision-free; parallel with Phases 1-2*

1. **A2 (SD9) m22 FeatureVector newtype** — cosmetic typed-newtype around `Vec<f64>`. ~30-50 LOC + test.
2. **C1 NA-GAP-06 drain staging** — write `drain` skeleton in m13 (private API; not yet a consumer). Includes outbox-pointer-tracking + idempotent replay test. ~80-120 LOC. **Stops short of external-consumer wiring** (that depends on V1 + V2, lands Phase 5).
3. Gate green per commit.
4. Commits: `feat(workflow-trace): SD9 m22 FeatureVector newtype` + `feat(workflow-trace): m13 outbox drain skeleton (NA-GAP-06 half)`.

### Phase 4 — Decision interview · ~2-3 h Luke active time / ~0.5-1.5 calendar-days · *needs Luke* · ~15-18 questions across Round A + Round B + stated defaults

See §15 for the full interview. Round A questions are **no-defaults, load-bearing**. Round B has **defaults flagged**. Stated defaults are **named in plan, not in interview** per C-10.

### Phase 5 — Tier 2 wire-contract (W1 or W2) + W3 + W4 **+ V1 co-land (per C-2) + A4 SD11 (per C-6)** · ~5-7 days · *needs DX-W.a / DX-W.b / DX-W.c / DX-W3.src / DX-DAW-1*

1. **W4** (smallest, decision-light) — add `CuratedBank::client_ref()` read-only accessor. ~30-60 LOC. (Day 1.)
2. **W1 OR W2 per DX-W.b:**
   - **If W1:** add `escape_surface: EscapeSurfaceProfile` to `WorkflowProposal` + fallible `new()` + serde regen + JSONL fixture regen across `m23 / m30 / m32 / orchestration` tests. ~150-200 LOC. (Days 2-3.) Per **DX-W.c**: SemVer-break or defaulted-field.
   - **If W2:** new module or co-locate in m23 — StepToken → EscapeSurfaceProfile table + variant-aggregation. ~80-150 LOC. (Day 2.)
3. **W3** — add `cost: i64` to `WorkflowProposal`; populate in `compose_proposals` using `mutation-weight` source from DX-W3.src. ~150-250 LOC. (Days 3-4.)
4. **A4 SD11 12-field proposal shape** — co-lands here per C-6: per-step cost (via W3), lineage_chain, generation_index, parent_proposal_id, lift_p95, diversity_cluster. ~50-80 LOC. (Day 4.)
5. **V1 RefusalToken co-land per C-2** — enum definition + thread through every refusal call-site (m9 / m32 / m13 / m40 / m41 / m42 / m33). `Unavailable(EngineImagined/SubstrateUnreachable/SubstrateAuthored)` sub-tagging per NA-5. Wire m13 drain skeleton (from Phase 3) to consume RefusalToken-tagged events. ~150-250 LOC + 80-150 new tests. (Days 5-7.)
6. Re-run `cargo-mutants` scoped to `m23 + m30 + m32 + m33 + m13 + m40 + m41 + m42 + orchestration`; hold the DX-Mut bar (96.3%).
7. Commits: one per W/A/V item; **minimum 5 commits**.

### Phase 6 — Tier 3 real verifiers (R1 + R2 + R3) · ~3-4 days · *needs Phase 5 + DX-W.b + DX-R3*

1. **R1 Security real verifier** — replace M0 Sandboxed-default no-op with live check against W1's `proposal.escape_surface` OR W2's `EscapeSurfaceProfile::from_token()` aggregation. Verdict semantic: hard-Refuse per D5/D6. ~50-100 LOC. (Day 1.)
2. **R2 Cost real verifier** — implement D10 metric `step-count × mutation-weight` using DX-W3.src source. Threshold against config `cost_ceiling`. ~50-80 LOC. (Day 2.)
3. **R3 Consistency real verifier** — bank-conflict detection consuming `client_ref()` + A4's 12-field shape. **Conflict semantic per DX-R3:** variant_id-only (default) / lineage-chain-only / both. ~100-150 LOC. (Days 3-4.)
4. Re-run `cargo-mutants` scoped to `m33`; hold the bar. Multi-hour run — budget it.
5. Commits: one per R item.

### Phase 7 — Tier 1A: V1 call-site threading + drain wire (SHRUNK PER C-2; enum + outbox-frame work already in Phase 5) · ~0.5-1 day · *needs DX-1*

1. Confirm V1 enum + sub-tagging shipped in Phase 5 lands cleanly.
2. Audit call-site authorship classifications: m9 namespace guard (substrate-authored when reducer rejects), m32 dispatcher (engine-authored), m13 outbox (substrate-authored on stcortex refuse-write), m40 Nexus emit (engine-authored on serialization fail), m41 LCM RPC (mixed), m42 stcortex emit (substrate-authored).
3. **If DX-1 amends variants** (e.g., `WatcherAuthored`): add variant + thread; re-run scoped mutants.
4. Commit: `feat(workflow-trace): V1 Phase 7 — call-site classification audit + DX-1 amendments` (or "noop" if DX-1 = no amendment).

### Phase 8 — Tier 1B: V2 substrate back-pressure budget (per-substrate per NA-8) · ~2-3 days · *needs DX-2*

1. Define `SubstrateBackPressureMode` enum keyed by substrate-id (per NA-8 reshape): `Push`, `Pull`, `Unavailable`.
2. Default = `Pull` for all 5 substrates at v0.2.0 ship; flip per-substrate to `Push` as emitters ship.
3. Engine receiver-side: receive `BackPressureSignal { substrate, severity, observed_at_ms }` envelope (in push mode) OR probe-and-throttle (in pull mode).
4. Wire signal into m1 (atuin throttle), m13 (stcortex throttle), m32 (Conductor throttle) cadence-modulation.
5. Tests + scoped `cargo-mutants`.
6. Commit: `feat(workflow-trace): V2 — per-substrate back-pressure budget (per-substrate push/pull mode keyed by substrate-id)`.

### Phase 9 — Tier 1C: V3 m16 substrate-drift canary (KEYSTONE) · ~2-3 days (DX-V3 = distributed) **OR ~5-12 days (DX-V3 = own module)** · *needs DX-V3 + DX-V3.b*

**If DX-V3 = distributed:**
1. No Genesis amendment; canary participation distributed across m9 (refusal-pattern observer), m13 (stcortex-decay observer), m32 (Conductor-enforcement observer), m42 (pathway-weight-drift observer).
2. Each participating module exports a `canary_observation()` accessor; thin aggregator in m40 Nexus emits unified `SubstrateDriftDetected` event.
3. **Per NA-4 + §6 risk-row revision:** add `Watcher m16 heartbeat liveness assertion` — Watcher's deployment-watch journal asserts m16 heartbeat liveness; missing heartbeat for N cycles = substrate-emitted alert. If Watcher integration cannot ship in v0.2.0, the self-canary problem is **not mitigated** — risk register says so honestly.
4. Tests + scoped `cargo-mutants`.

**If DX-V3 = own module (~5-12 days):**
1. **Day 1 (Genesis amendment):** Author plan.toml + Genesis Prompt v1.4 amendment lifting module count 26 → 27. Pair-file at `~/projects/shared-context/agent-cross-talk/`.
2. **Days 2-7+ (Zen wait, ~3-10 days based on cross-talk dir history):** Dispatch Zen G7 re-audit per D-S1002127-03 §3 trigger; implementation proceeds in parallel. **Per DX-V3.b:** if Zen silent for N days (N TBD by DX-V3.b), ship v0.2.0 without Zen approval OR hold.
3. **Days 2-5 (impl):** Author `src/m16_substrate_drift_canary/` per spec at `ai_specs/cross-cutting/substrate-drift.md`; 5 clock samplers (m11 recency / m13 stcortex-decay / injection-TTL / atuin-checkpoint / stcortex-pathway-decay); agree-to-skew envelope per CC-5 documented crossings.
4. Emits `SubstrateDriftDetected` events through V1 RefusalToken-typed substrate-authored channel.
5. Same NA-4 Watcher liveness assertion as distributed case.
6. Per **§6 new risk row (C-9)**: V3 rate-limited alerts (≤N alerts per soak hour with dedup by `(clock-pair, envelope-band)`); operator visible only after N consecutive crossings.

### Phase 10 — Tier 1D: V4 substrate test fixtures · ~3-4 days · *needs DX-5*

1. Author `tests/substrate_fixtures/` per ADR D-S1002127-03 §1.b.
2. **If DX-5 = full deterministic replicas:** local SQLite-backed stcortex stub, mock ORAC HTTP server, mock synthex-v2 WebSocket, etc.
3. **If DX-5 = mock-only:** struct-level fakes returning canned responses.
4. Five named fixtures: `cr2_inflation_fixture` / `refuse_write_no_consumer_fixture` / `hyphen_slug_reducer_fixture` / `conductor_enforcement_flag_off_fixture` / `atuin_wal_contention_fixture`. **Plus V3-canary-failure fixture** per NA-4 (asserts Watcher liveness fires when m16 stops emitting).
5. Per-fixture integration test exercising engine's response chain (typically through R1/R2/R3 + V3 canary).
6. ~80-150 fixture tests added; TEST_STRATEGY budget bumps from 1594 to ~1750-1800.
7. Commit: `test(workflow-trace): V4 — substrate test fixture suite`.

### Phase 11 — Tier 1E: V5 substrate-mediated trust + cross-habitat ADR · ~2-4 days · *needs DX-V5 + DX-V5.b*

1. **If DX-V5 = full cross-habitat:**
   - Author ADR D-S1004XXX-05 covering substrate-side changes: stcortex consumer-trust score schema, Conductor dispatch-budget table, atuin read-quota daemon flag, ORAC reputation hooks, synthex-v2 r13-state-aware verifier weighting.
   - Workflow-trace ships engine-side primitive: `SubstrateTrust { stcortex_score, conductor_budget_remaining, atuin_quota_remaining, ralph_generation_advanced_since, watcher_m16_heartbeat_live }` (RALPH + Watcher per NA-2).
   - **Plus `substrate_participation_status: enum { NotShipped, Shipping, Live }`** accessor per NA-5; visibility primitive forcing engine to switch from imagining to listening.
   - Pair-file ADR; does NOT gate v0.2.0 ship (cross-habitat coordination is v0.2.0 → post-v0.2.0 bridge).
2. **If DX-V5 = in-engine receiver-only:**
   - Ship engine-side primitive with `substrate_participation_status = NotShipped` per substrate; substrate-side returns `RefusalToken::Unavailable(SubstrateUnreachable)` per V1 NA-5 sub-tagging — **not** `EngineImagined` (which would require live substrate that doesn't exist).
3. Per **§6 new risk row (C-9)**: V5 versioned ADR + `serde(other)` fallback variant on consumer side; flag unknown field as `RefusalToken::SubstrateAuthored` event.
4. Wire `SubstrateTrust` into R3 Consistency verifier (Phase 6) as additional input axis if available.
5. Tests: trust-aware vs trust-unavailable verdict invariance; degradation-on-substrate-trust-loss; **`RefusalToken::Unavailable` sub-tag-distinguishability test** per NA-5.
6. Commit: `feat(workflow-trace): V5 — substrate-mediated trust (per DX-V5 shape) + substrate_participation_status accessor`.

### Phase 12 — Tier 4 algorithmic upgrades + integration + Zen audit + v0.2.0 ship · ~2-3 days

1. **A1 (SD8)** m21 true Levenshtein per DX-4 source. ~100-150 LOC.
2. **A3 (SD10)** m22 empty-cluster per DX-3 (typed error / re-seed / retain-prior). ~50-80 LOC. *(A4 SD11 already landed in Phase 5 per C-6; A2 SD9 in Phase 3.)*
3. **Integration & end-to-end:**
   - Full 4-stage gate + full `cargo-mutants` workspace-wide. Hold DX-Mut bar.
   - **Per §6 new risk row (C-9 mutation-cap):** per-phase mutation wall-time cap (e.g., 4h); over-cap = ship with documented `// mutant-equivalent:` proofs OR defer mutation work to v0.2.1-mut sub-release.
   - `wf-dispatch --dry-run` smoke against new wire-contract shape.
   - Substrate-enumeration sweep per NA-2 — verify §7's 7 substrates have correct status accessors.
4. **Zen audit fold-in** — dispatch per Plan v2 §3 Phase 9 pattern. Verdict-absent → in-session zen agent substitute per D26.
5. **DX-CI fold:** apply chosen `spacetimedb-sdk` resolution (DX-CI is now Round A per C-10).
6. **Doc / persistence sweep:** `CHANGELOG.md` [v0.2.0] entry; reconcile `CLAUDE.md`, project `CLAUDE.local.md`, `GATE_STATE.md`, `ARCHITECTURE.md`; update `ai_docs/INDEX.md` + `ai_specs/INDEX.md`.
7. **Four-surface persist** with read-back-verify per §13 + slug discipline (no hyphens per NA-11).
8. **Tag `v0.2.0`**; commit; push origin + gitlab.
9. **Operator hand-off — OP-3 acceptance criteria table per C-13:**

   | Surface | Pass criterion | Failure handling |
   |---------|----------------|------------------|
   | `--execute` round-trip | N=10 dispatches across the 5 substrates; each lands with substrate-confirmable receipt | failure = v0.2.0 un-ship, address in v0.2.1 |
   | V1-V5 primitive exercise during soak | each primitive fires at least once during DX-Soak window | partial = v0.2.0 ships, named in Honest Residuals |
   | Soak duration | DX-Soak hours of stable dispatch (default 48h per stated default) | shorter = explicit Luke ratify |

10. Hand operator items to Luke: **OP-3** post-v0.2.0 substrate soak (Watcher ☤ carries per D36); cross-habitat ADR review if DX-V5 = full.

## 4. Sequencing & dependency graph

```
Phase 1 ─┐
Phase 2 ─┼─ decision-free, parallel ── Phase 4 (interview, informed by 1-3)
Phase 3 ─┘                                  │
                                             ↓
                  Phase 5 (Tier 2 W1/W2 + W3 + W4 + A4 SD11 + V1 co-land per C-2)
                                  ↓
                  Phase 6 (Tier 3 R1 + R2 + R3 — consumes Tier 2 + V1)
                                  ↓                  ↓                  ↓
                              Phase 7 (V1 call-sites)     Phase 8 (V2 — independent)
                                  ↓
                              Phase 9 (V3 — consumes V1 for RefusalToken-typed drift)
                                  ↓
                              Phase 10 (V4 fixtures — consumes V3 for canary-replica testing)
                                  ↓
                              Phase 11 (V5 — consumes V1 for refusal authorship)
                                  ↓
                              Phase 12 (Tier 4 + integration + audit + ship)
```

**Critical path:** Phase 2 → Phase 4 → Phase 5 → Phase 6 → Phase 7 → Phase 9 → Phase 10 → Phase 12.

**Sequencing notes:**
- Phase 8 (V2) is **independent** of V1; can run in parallel with Phase 7 or after.
- Phase 11 (V5) depends on V1 (for trust-unavailable `RefusalToken::Unavailable` case + NA-5 sub-tagging); cannot run before Phase 7.
- Phase 4 deliberately runs **after** Phases 1-3 (gap analyses must have run by then per Plan v2 D44 pattern).
- **Tier 2 first (DAW-1) is CONDITIONAL on DX-DAW-1 outcome.** If DX-DAW-1 = Tier-1-first, the sequence inverts.
- Phase 9 (V3 own-module) has parallel work: Genesis amendment + Zen wait + impl all proceed simultaneously; the ship gate is Zen verdict (DX-V3.b N-day escalation).

## 5. Effort roll-up *(honest ranges; bottom-up; Plan-v2-arc overhead per C-7)*

| Phase | Effort | Driver of the range |
|-------|--------|---------------------|
| 1 — re-baseline + ADR cascade + file:line re-verify + mutation-weight pin | ~1-1.5 day | ADR amendment cascade density (C-8) |
| 2 — FP-verification + Tier 2 sizing + substrate audit | ~0.5-1 day | how much W1/W2 sizing drift since Phase 2 audit (S1004115); NA-2 substrate-enumeration depth |
| 3 — A2 + C1 staging | ~0.5-1 day | drain skeleton tests |
| 4 — interview | ~2-3 h active / ~0.5-1.5 calendar-days | Luke availability + DX-DAW-1 depth |
| 5 — Tier 2 W1/W2 + W3 + W4 + A4 SD11 + **V1 co-land** | ~5-7 days | V1 absorption (per C-2 fold) raises Phase 5; W1 vs W2 (±1d); W3 mutation-weight source (±1d) |
| 6 — Tier 3 R1 + R2 + R3 | ~3-4 days | DX-R3 scope (Consistency variant_id-only vs both); cargo-mutants ~hr/run |
| 7 — V1 call-site threading + drain wire | ~0.5-1 day | shrunk per C-2 (enum work already in Phase 5) |
| 8 — V2 per-substrate back-pressure | ~2-3 days | NA-8 per-substrate `SubstrateBackPressureMode` enum scope |
| 9 — V3 m16 canary (KEYSTONE) | **~2-3 days (distributed) OR ~5-12 days (own-module + Zen wait)** | DX-V3 + DX-V3.b (C-4 + Genesis cascade) |
| 10 — V4 fixtures | ~3-4 days | full-replica vs mock-only (DX-5) |
| 11 — V5 trust + cross-habitat ADR + substrate_participation_status accessor | ~2-4 days | full cross-habitat vs in-engine receiver-only (DX-V5); NA-5 sub-tagging |
| 12 — Tier 4 + integration + audit + ship | ~2-3 days | DX-Mut hold + DX-CI resolution + DX-Soak duration |
| **v0.2.0 narrow execution** | **~22-36 Claude-days** | (matches v1 lower bound; covers phases 1-12 only) |
| Plan v1 → v2 round-trip + persistence (C-7) | ~1-2 Claude-days | gap analyses + interview write-up + 4-surface persist + read-back-verify |
| Interview latency (Luke decisions, ~15-18 questions ~3-5 sessions per C-7) | ~0.5-1.5 calendar-days | not active Claude time; spread across sessions per Plan v2 D45/D46 |
| Cumulative mutation wall time across all phases (C-7) | ~1.5-3 Claude-days | scoped per phase + full Phase 12 |
| **v0.2.0 full Plan-v2-arc total** | **~25-42 Claude-days; mid-point ~33** | + Luke / Zen / cross-habitat gating |

The stub catalogue estimate was `~6-10 Claude-days at god-tier` — that was top-down and substantially under-scoped. Plan v1's bottom-up `~22-36 Claude-days` covered narrow execution only. v2's `~25-42 Claude-days mid-point ~33` honestly counts the Plan-v2 arc overhead the v0.1.0 cycle measured.

## 6. Risk register

| Risk | Mitigation |
|------|------------|
| W1 wire-contract change breaks every JSONL fixture (S112 class) | Phase 2 enumerates blast radius pre-DX-W; **Phase 5 commits W1+V1+A4 as a single landed unit per C-2** so JSONL regen happens once; gate per commit |
| V3 m16 own-module triggers Genesis v1.4 + Zen G7 re-audit cascade (multi-day external dependency per C-4) | **DX-V3.b** N-day escalation; Phase 9 estimate forks honestly (distributed ~2-3 d / own-module ~5-12 d); Genesis amendment + implementation parallel with Zen wait |
| V5 cross-habitat coordination slips (substrate-side repos don't accept changes) | **DX-V5** includes in-engine-receiver-only fallback; **versioned ADR + `serde(other)` fallback** per C-9 new row; substrate-side never blocks v0.2.0 ship |
| V4 fixture realism doesn't catch the production drifts it's designed for (Frame-A trap NA-1 redux) | Each fixture tests against the exact CR-2-class incident it was sourced from; **V3-canary-failure fixture** added per NA-4; fixture-design review folds in Phase 12 audit |
| Mutation kill-rate slips below DX-Mut bar | Scoped `cargo-mutants` per phase + full re-run Phase 12; **per-phase mutation wall-time cap (4h) per C-9 new row**; over-cap = ship with `// mutant-equivalent:` proofs OR defer to v0.2.1-mut sub-release |
| Agent over-claims a phase gate-clean (LCM P0-P4 drift × 6 + Armada FP-discovery) | Orchestrator re-runs full `--workspace --all-targets --all-features` gate + `git log -1` + independently exercises new code paths per phase (Plan v2 D42 discipline) |
| Tier-2-first ordering forces re-touching V1/V5 if substrate frame disagrees with wire-contract shape | **DAW-1 RE-OPENED as DX-DAW-1** per NA-1 + convergent C-1; gap analyses surfaced collapse evidence — Phase 4 owns the sequence |
| `RefusalToken` enum expansion (DX-1 amendment) cascades through 7 modules late | **V1 enum + sub-tagging shipped in Phase 5 co-land per C-2**; Phase 7 audits call-site classifications + DX-1 amendments only; refusal-token enum locked at Phase 5 |
| v0.2.0 takes longer than v0.1.0; Luke fatigue / opportunity-cost shift to Master Plan v2 or Ember | **§5 honestly accounts for Plan-v2 arc overhead (~25-42 days)**; Plan v2 D45/D46 conviction-and-opportunity-cost reopens at v0.2.0 Phase 4 (DX-DAW-1 includes this scope) |
| Substrate-drift canary itself drifts (V3 self-canary problem) | **Per NA-4 + §6 revised row:** Watcher's deployment-watch journal asserts m16 heartbeat liveness; missing heartbeat for N cycles = substrate-emitted alert. If Watcher integration cannot ship, self-canary problem is **not mitigated** — risk says so honestly. |
| **V5 substrate-side schema lands different shape than engine-side primitive** (C-9 new row) | Ship V5 as versioned ADR with `serde(other)` fallback variant on consumer side; unknown field → `RefusalToken::SubstrateAuthored` event; **`substrate_participation_status` accessor (NA-5)** forces explicit transition from EngineImagined to SubstrateAuthored |
| **V3 canary alert-fatigue / false-positive storm** (C-9 new row) | V3 emits ≤N alerts per soak hour with rate-limiter + alert-dedup by `(clock-pair, envelope-band)`; operator visible only after N consecutive crossings; alert-budget primitive in V3's m40-emit-shape |
| **Mutation gate budget blowout against gate-per-phase discipline** (C-9 new row) | Per-phase mutation wall-time cap (4h); over-cap = ship with `// mutant-equivalent:` proofs OR defer to v0.2.1-mut sub-release; mutation work has its own ship gate |
| **DAW-1 reopening (DX-DAW-1) inverts Tier order; phase numbering changes** | Phase numbering in §3 explicitly conditional on DX-DAW-1 outcome; §15 owns the sequence-swap; effort estimates carry over either way |
| **ADR D-S1002127-03 amendment language cascade missed** (C-8) | Phase 1 step 2.5: `grep -rn "D-S1002127-03" .` + update every referencing doc; Phase 1 bumped to ~1-1.5 d |
| **`mutation-weight` source unverified at plan-authoring time** (C-5) | **Phase 1 step 3** pins source candidate; **DX-W3.src** ratifies (variant.mutation count / new StepToken table / m11 fitness pull-through); R2 estimate adjusts to source |

---

# PART B — Substrate-frame pass *(genuine second authoring, recursion-checked per NA-9 + convergent C-2)*

> v1's Part B was a genuine second pass on the *certification criterion* but bounded by the substrate set §7 already named (NA-2) and by §9's missing self-recursion (NA-9). v2's Part B **expands §7 to 7 substrates** (adds RALPH, Watcher ☤, Cargo build graph per NA-2 + convergent C-3), **re-labels certification language to engine-side substrate-participation readiness** (NA-10 + T-3), **adds the §9 recursion sub-section** that asks "does §9 itself collapse?" (NA-9 + convergent C-2), and **acknowledges Tier-2-first spine as Frame-A geometry** subject to DX-DAW-1 (NA-1 + NA-7 + convergent C-1).

## 7. Re-authoring "complete" from the substrate frame (7 substrates per NA-2)

Part A's "complete" = *every primitive the engine needs to act as a substrate participant is shipped*. The substrate frame asks: **does the substrate experience workflow-trace differently after v0.2.0?**

### 7.1 atuin (the read-side WAL substrate)
v0.1.0 added engine-timed proxy for atuin's contention (Phase 8 step 3, Frame-A). v0.2.0 V2 per-substrate `SubstrateBackPressureMode::Pull` (default; flips to `Push` when atuin opts in) makes atuin's own write-latency a first-class signal *as the upstream substrate participates*. **Substrate experience changes only if atuin upstream ships push-emitter.** Until then v2 measures from the engine side honestly.

### 7.2 stcortex (the write-target substrate)
v0.1.0 added read-back-verify for engine writes (NA-GAP-06 §13). v0.2.0 V1 `RefusalToken::SubstrateAuthored` means stcortex's `RefuseWriteNoConsumer` becomes a substrate-authored event — the engine receives the refusal *as substrate speech*, not as engine inference. **Substrate's refusal voice is audible** (substrate-side participation already exists via reducer).

### 7.3 HABITAT-CONDUCTOR (the dispatch substrate)
v0.1.0 added enforcement-state assertion (NA-4 Phase 8 step 2). v0.2.0 V5 substrate-mediated trust means Conductor's dispatch-budget becomes a verifier input — engine asks Conductor *what it will accept* rather than presuming. **Substrate's consent state is consulted only if DX-V5 = full cross-habitat AND Conductor ships dispatch-budget table.**

### 7.4 The CC-5 loop's clocks
v0.1.0 enumerated the crossings (NA-5 Phase 8 step 4). v0.2.0 V3 m16 canary means clock-incoherence is detected, not just documented. **Substrate's temporal drift becomes an event.** Per NA-4: V3 self-canary requires Watcher liveness assertion to close the loop honestly.

### 7.5 Luke @ node 0.A (operator substrate)
v0.1.0 made the Phase 4 interview no-defaults on load-bearing questions (NA-3). v0.2.0 promotes DX-1/DX-2/DX-5 to Round A per NA-3 + adds DX-DAW-1 per NA-1; **Round A capped at ~5 questions per session** per operator-fatigue dynamics. Operator-refusal path is first-class ("this question is malformed / not now / present it again later"). **Operator's attention budget is again first-class.**

### 7.6 **RALPH (added per NA-2)** — the evolutionary substrate
RALPH is the workspace's evolutionary substrate (CLAUDE.md service row 14; substrate metric in workspace `CLAUDE.local.md`: `RALPH gen 7,622 fit 0.6987 ↑`). Workflow-trace's m42 stcortex emission *participates in RALPH's fitness signal* — patterns are metabolised into RALPH's gen/fitness loop. v0.2.0 V5 `SubstrateTrust` adds `ralph_generation_advanced_since: bool` accessor — without it, engine cannot know whether its emissions are being metabolised. **RALPH's pacing is now a substrate signal the engine reads.**

### 7.7 **The Watcher ☤ (added per NA-2)** — the persona substrate
The Watcher is a substrate with its own AP27 self-mod boundary, Ember 7-trait unanimity gate, and `m46-m51` modules that actively read the engine's emissions. v0.2.0 V3 m16 *competes with* Watcher's deployment-watch journal for the same observability slot — DX-V3 must reconcile: is m16 a Watcher *organ* or a Watcher *competitor*? Per NA-4: the Watcher's deployment-watch journal asserts m16 heartbeat liveness, making V3's self-canary loop closeable. **The Watcher's observation cadence is now a substrate clock the engine queries.**

### 7.8 **The Cargo build graph (added per NA-2)** — the toolchain substrate
The build graph is a substrate the engine inhabits: own decay (toolchain upgrades), own refusal surface (clippy pedantic), own authorship (workspace `Cargo.lock` is partly written by other services). DX-CI is **substrate-coupling, not CI-mechanism** — submodule pins workflow-trace to a moment in the substrate's history (Frame-B observation point); vendor *severs* workflow-trace from the substrate (Frame-A insulation). **The build graph's authorship shape is set by DX-CI; v0.2.0 must name the substrate-coupling choice, not just the CI policy.**

**The re-authored conclusion:** v0.2.0 is the milestone at which the engine becomes *ready* to participate with substrates. Whether the conversation becomes meaningful depends on per-substrate participation primitives shipping post-v0.2.0 (sized per consent gradient in §11 per NA-6). **The engine half of the substrate-participation contract is what v0.2.0 certifies.**

## 8. What `v0.2.0` certifies — and does not *(re-labelled per NA-10 + T-3)*

> **`v0.2.0` certifies engine-side substrate-participation readiness:** every primitive the engine needs to *participate in* (not *consume*) the substrate is shipped, tested, audited, documented across 7 substrates (atuin / stcortex / Conductor / CC-5 clocks / Luke + Watcher + RALPH + Cargo build graph). The engine has authorship-typed refusal channels with `Unavailable(EngineImagined/SubstrateUnreachable/SubstrateAuthored)` sub-tagging, per-substrate back-pressure receivers, a substrate-drift canary with Watcher liveness assertion, substrate test fixtures, substrate-mediated trust hooks with `substrate_participation_status` accessor, and an honest cross-habitat ADR (V5).
>
> It does **not** certify that substrate-side schemas / daemons / consumer-trust tables / emitters exist in stcortex, ORAC, synthex-v2, LCM, HABITAT-CONDUCTOR — that is **post-v0.2.0 cross-habitat coordination**, sized per per-substrate consent gradient in §11. The tag tells the habitat "the engine is ready to be a substrate participant"; it does not tell the habitat "the substrates have agreed to participate."

If the cross-habitat work never happens, v0.2.0's engine-side primitives degrade gracefully via V1 `RefusalToken::Unavailable(EngineImagined)` and remain useful as defensive instrumentation. The substrate frame doesn't gain a voice; the engine at least has the *capacity* to hear one *and* the visibility primitive (`substrate_participation_status`) that prevents the engine from talking over a substrate voice that later arrives.

## 9. Frame-collapse self-check *(WITH recursion sub-section per NA-9 + convergent C-2)*

Interrogating Part B itself.

### 9.1 First-order check (carried from v1, refined)

- **V2 per-substrate `SubstrateBackPressureMode`** (per NA-8 reshape) is genuinely Frame-B per-substrate: `Push` is substrate-emit, `Pull` is engine-probe, `Unavailable` is honest absence. Default `Pull` is Frame-A pacing for substrates without push-emitters; this is named.
- **V3 m16 canary** is engine-side aggregation flagged as such, **with Watcher liveness assertion** (NA-4) — m40 Nexus alone cannot close the self-canary loop (m40 shares fate with m16); Watcher's journal does close it because it's a different binary. If Watcher integration cannot ship, self-canary is honestly **not mitigated** in §6.
- **V5 in-engine receiver-only** + **`RefusalToken::Unavailable` sub-tagging** (NA-5) prevents `EngineImagined` from being audit-indistinguishable from `SubstrateAuthored`. Plus `substrate_participation_status` accessor forces engine to switch from imagining to listening as substrate-side ships.

### 9.2 Recursion sub-section (NEW v2 per NA-9 + convergent C-2)

The discipline asks: **does §9 itself collapse?** Three places it might:

#### 9.2.a Is §9's substrate set complete?

§9 above checks V2/V3/V5 — three of the five Tier-1 primitives. **V1 (RefusalToken) and V4 (substrate fixtures) are not first-order-checked.** Doing so now:

- **V1 RefusalToken sub-tagging is Frame-B-leaning** (NA-5 reshape makes substrate-authorship typed). But the *authorship classification table* (who is the substrate-author at each call-site) is engine-authored. The substrate frame would have substrates emit their own authorship classification; v0.2.0 has the engine classify. **Honest label: V1 sub-tagging is Frame-B-half; authorship classification table is Frame-A.**
- **V4 substrate test fixtures are inherently Frame-A** (the engine builds replicas of substrates). The substrate-frame test of substrate-conversation is whether *live substrates* exercise the engine, not whether *replicas* do. V4's value is bounded by replica fidelity (DX-5 sets the floor). **Honest label: V4 is Frame-A instrumentation; live-substrate exercise is post-v0.2.0 OP-3 soak.**

§9.1 missed both. **§9.1 is incomplete by 2/5.**

#### 9.2.b Is §9's verdict self-validating?

§9.1's implicit verdict ("V2/V3/V5 honestly labelled, no concealment") is asserted by the same authoring pass that produced §9.1. The NA discipline warns *"a beautifully written second-pass that turns out to have re-run the original frame in different vocabulary"* is the worst output. The verdict cannot self-validate. **External review (this gap analysis) is required.** This recursion sub-section is itself the verification — and it found §9.1 is 2/5 incomplete. The verdict shifts from "no concealment" to "concealed V1 authorship-table Frame-A + V4 Frame-A; both now labelled honestly above."

#### 9.2.c Does §7's expanded substrate set survive the substrate's clock?

§7 v2 enumerates 7 substrates. The substrate-frame question: from each substrate's clock, what changes at v0.2.0 ship?

- **atuin / stcortex (substrate-side already participates):** v0.2.0 ship is a substrate-observable event (engine starts emitting `RefusalToken::SubstrateAuthored` types). Frame-B.
- **HABITAT-CONDUCTOR (post-v0.2.0 substrate-side):** v0.2.0 ship is invisible to Conductor until substrate-side ships dispatch-budget table. Frame-A pacing for Conductor.
- **CC-5 loop clocks (engine-internal aggregation):** v0.2.0 ship adds canary-clock samplers; no clock itself changes. Frame-A by definition.
- **Luke + Watcher + RALPH (active-substrate participants):** Luke experiences the v0.2.0 interview (Frame-B-half — operator-refusal path is first-class). Watcher experiences v0.2.0 if m16 integrates with Watcher's deployment-watch journal (Frame-B if integration ships; Frame-A if not). RALPH experiences v0.2.0 if `ralph_generation_advanced_since` accessor is consumed (Frame-B-half).
- **Cargo build graph:** v0.2.0 ship sets the substrate-coupling shape via DX-CI (Frame-B observation point if submodule; Frame-A insulation if vendor).

§7's headline ("the engine becomes ready to participate") survives — but **for ~4 of 7 substrates the readiness is unilateral**. The substrate clock registers no event at v0.2.0 ship. Honest label: §7 is engine-pacing language; v0.2.0 is a unilateral engine-side declaration of readiness; substrate-side acknowledgement is post-v0.2.0 per-substrate per §11.

#### 9.2.d Does the Tier-2-first spine survive the substrate frame?

Plan v1 §10 T-2 named this tension and deferred to Phase 4. v2 **re-opens DAW-1 as DX-DAW-1** per NA-1 + convergent C-1. The recursion sub-section confirms this is correct: the spine is engine-frame geometry (typed seams un-block verifiers cleanly) — defensible from engineering, *not yet defended from the substrate frame*. The substrate-frame defence would be: "the engine asks the substrate 'what shape will your refusal take?' via W1 *before* the substrate has authored V1's RefusalToken vocabulary." That defence is not in v2 either — Phase 4 owns it. **§9 records that the spine is Frame-A-shaped pending DX-DAW-1.**

### 9.3 Recursion verdict

§9.1 was incomplete (V1 + V4 missing). §9.2 catches the omission and labels honestly. §9.2.a's added labels (V1 authorship classification table Frame-A; V4 Frame-A instrumentation) integrate into the honest residual list. §9.2.b's verdict: external review required, satisfied by this gap analysis. §9.2.c's verdict: §7 is engine-pacing for 4/7 substrates; honest label added. §9.2.d's verdict: Tier-2-first spine is Frame-A pending DX-DAW-1.

**No collapse is concealed *that this recursion catches*.** A third-order recursion (does §9.2 itself collapse?) is not in this document; the discipline's stopping rule per NA precedent is "as long as the next-order question is asked, the discipline holds." §9.2 is that question for §9.1.

## 10. Frame tensions — explicit reconciliations

- **T-1 — DAW-1 was a Luke call with eyes open vs Luke chose without substrate-frame defence.** Reconciled by **DAW-1 re-opening as DX-DAW-1** in §15 Round A with both framings produced in the question; convergent C-1 adopted unconditionally.
- **T-2 — Plan v1 §9 self-check is genuine vs §9 only self-checks within its own substrate set.** Reconciled by **§7 expansion to 7 substrates** (NA-2) AND **§9.2 recursion sub-section** (NA-9 + C-2). Both improvements land.
- **T-3 — "engine + substrate co-completeness" vs "engine-side participation readiness".** Reconciled by **§1 + §8 re-labelling** to "engine-side substrate-participation readiness" per NA-10. Honest one-sided naming throughout.
- **T-4 — in-engine receiver-only is clean fallback vs engine-imagined `RefusalToken::Unavailable` indistinguishable from substrate-authored in shipped behaviour.** Reconciled by **NA-5 sub-tagging** `Unavailable(EngineImagined/SubstrateUnreachable/SubstrateAuthored)` + **`substrate_participation_status` accessor**. Plan ships V5 in-engine receiver-only with explicit divergence prevention.

---

# PART C

## 11. Post-v0.2.0 partition *(per-substrate consent gradient per NA-6)*

What v0.2.0 does **not** close, sized per per-substrate consent gradient:

| Substrate-side primitive | Substrate | Acceptance probability | Estimated calendar latency | Mitigation if refused |
|--------------------------|-----------|------------------------|----------------------------|-----------------------|
| Consumer-trust score schema | stcortex | HIGH (single-author repo, active substrate-mastery) | 1-2 weeks | engine ships V5 with `SubstrateTrust.stcortex_score = None`; `RefusalToken::Unavailable(SubstrateUnreachable)` until ship |
| Dispatch-budget table | HABITAT-CONDUCTOR | HIGH (workflow-trace-adjacent; B3/OP-1 already pending Luke) | 1-2 weeks (after Conductor bring-up) | `SubstrateTrust.conductor_budget_remaining = None`; defaults to "trust everything" |
| Read-quota daemon flag | atuin | UNKNOWN (third-party upstream; community-driven) | indeterminate; possibly never | V2 stays in `Pull` mode for atuin permanently; in-engine throttle |
| Reputation hooks | ORAC | MED (workflow-trace adjacent; 40 modules; RALPH integration negotiation) | 2-4 weeks | `SubstrateTrust.orac_reputation = None`; engine consumes via existing PostToolUse hooks only |
| r13-state-aware verifier weighting | synthex-v2 | HIGH (workflow-trace integration via m42 + V5 deps; Watcher coordination active) | 1-2 weeks | `RefusalToken::SubstrateAuthored` from synthex-v2 R13-bypass chain; no weighting until ship |
| m16 substrate-side emitter participation | All 5 above | per-substrate (see rows) | per-substrate | V3 stays distributed-canary engine-aggregation per DX-V3 |
| V2 push-mode emitters per substrate | All 5 above | per-substrate (see rows) | per-substrate | V2 stays in `Pull` mode per substrate; flip to `Push` as ships |
| Genesis Prompt v1.4 module-count amendment (if DX-V3 = distributed) | n/a | n/a | preserved for cross-habitat Frame-B turn | optional v0.3.0 |

**Cross-habitat substrate-mediated trust full unification (V5 cross-habitat):** ~3-6 calendar-months of cross-habitat coordination if all probabilities resolve favourably; longer if atuin upstream refuses. The unit of progress is *substrate consent*, not engine effort.

**`wf-dispatch --execute` 24h+ soak (OP-3 carried from this plan):** post-v0.2.0 dispatch soak per acceptance criteria table in Phase 12 step 9; Watcher ☤ carries per D36.

## 12. Gap-analysis disposition *(every finding from both passes accounted for)*

### 12.a Conventional findings (13)

| ID | Severity | Disposition |
|----|----------|-------------|
| C-1 | LOW | ACCEPTED — `RefusalReason` cited at `:228` (was `:163`); Phase 1 step 2.5 FP-verifies every §2 file:line |
| C-2 | **HIGH** | ACCEPTED — V1 co-lands with W1 in Phase 5 (option (a)); Phase 7 shrinks to call-site threading + drain wire; one regen pass instead of two |
| C-3 | **HIGH** | ACCEPTED — DX-W split into DX-W.a (retire (iii)? yes/no) + DX-W.b (W1 vs W2 if seam) + DX-W.c (SemVer-break vs default if W1) |
| C-4 | **HIGH** | ACCEPTED — Phase 9 estimate forks by DX-V3 (distributed ~2-3d / own-module ~5-12d); DX-V3.b "Zen silent N days → ship or hold?" added |
| C-5 | **HIGH** | ACCEPTED — Phase 1 step 3 pins `mutation-weight` source; DX-W3.src in Round A (variant.mutation count / new table / m11 fitness pull-through) |
| C-6 | MED | ACCEPTED — A4 SD11 moves to Phase 5 co-land with W1+W3; R3 in Phase 6 consumes 12-field shape unambiguously; DX-R3 in Round B (default variant_id-only) |
| C-7 | MED | ACCEPTED — §5 raised to ~25-42 Claude-days mid-point ~33; three new line items (plan-arc round-trip / interview latency / mutation wall time) |
| C-8 | MED | ACCEPTED — Phase 1 step 2.5 added; Phase 1 bumped to ~1-1.5 d |
| C-9 | MED | ACCEPTED — §6 gains 3 new rows (V5 versioned ADR + serde fallback; V3 rate-limited alerts; mutation-cap per phase) |
| C-10 | MED | ACCEPTED — DX-Mut/DX-Soak/DX-1-mechanism demoted to "stated defaults" in §14; DX-A4-coupling/DX-CI/DX-MGB promoted to Round A |
| C-11 | MED | ACCEPTED — §9.2 recursion sub-section added; explicit acknowledgement Tier-2-first spine is Frame-A pending DX-DAW-1 |
| C-12 | MED | CLOSED in v1 → v2 round (commit `81910a6` added `CLAUDE.local.md` pointer); §13 records lesson |
| C-13 | LOW | ACCEPTED — Phase 12 step 9 gains 3-row acceptance-criteria table |

### 12.b NA findings (12)

| ID | Severity | Disposition |
|----|----------|-------------|
| NA-1 | **HIGH** | ACCEPTED — DAW-1 re-opened as DX-DAW-1 in §15 Round A with both substrate-frame and engineering-frame framings; convergent C-1 |
| NA-2 | **HIGH** | ACCEPTED — §7 expanded to 7 substrates (atuin + stcortex + Conductor + CC-5 clocks + Luke + Watcher + RALPH + Cargo build graph); each gets Part-B paragraph + §11 partition row |
| NA-3 | **HIGH** | ACCEPTED — DX-1/DX-2/DX-5 promoted to Round A; operator-refusal path first-class; Round A capped at ~5 questions per session |
| NA-4 | **HIGH** | ACCEPTED — §6 V3 risk row revised: Watcher's deployment-watch journal asserts m16 heartbeat liveness; Phase 9 adds the assertion; Phase 10 adds V3-canary-failure fixture |
| NA-5 | **HIGH** | ACCEPTED — `RefusalToken::Unavailable(EngineImagined/SubstrateUnreachable/SubstrateAuthored)` sub-tagging; `substrate_participation_status` accessor added to V5; DX-V5.b in Round A |
| NA-6 | MED | ACCEPTED — §11 per-substrate consent-gradient table (stcortex HIGH / Conductor HIGH / atuin UNKNOWN / ORAC MED / synthex-v2 HIGH) |
| NA-7 | MED | ACCEPTED-WITH-HONEST-LABEL — §4 sequencing notes explicit "Tier-2-first is engine-authoring-first frame choice; Tier-1-first would be substrate-authoring-first"; DX-DAW-1 names the frame |
| NA-8 | MED | ACCEPTED — DX-2 reshaped from binary to per-substrate `SubstrateBackPressureMode` enum keyed by substrate-id; default Pull; flip per-substrate as emitters ship |
| NA-9 | MED | ACCEPTED — §9.2 recursion sub-section added; checks §9 substrate-set completeness (9.2.a), self-validating verdict (9.2.b), §7's substrate-clock survival (9.2.c), Tier-spine survival (9.2.d) |
| NA-10 | MED | ACCEPTED — §1 + §8 certification language shifted from "engine + substrate co-completeness" to "engine-side substrate-participation readiness" |
| NA-11 | LOW | ACCEPTED — §13 gains slug-discipline note (no hyphens per S1001757 munge bug; `grep '-' <slug>` must return no match) |
| NA-12 | LOW | CLOSED in v1 → v2 round (commit `81910a6` added stcortex memory `workflow_trace_v020_s1004377` genesis); Phase 1 step 7 read-back-verifies |

### 12.c Tensions (3 load-bearing)

| ID | Disposition |
|----|-------------|
| T-1 | ACCEPTED — DX-DAW-1 reopens DAW-1; both framings produced in question; convergent C-1 |
| T-2 | ACCEPTED — §7 expansion + §9.2 recursion; both fixes land |
| T-3 | ACCEPTED — certification language re-labelled "engine-side substrate-participation readiness"; §1 + §8 updated |
| T-4 (added per NA-5) | ACCEPTED — `RefusalToken::Unavailable` sub-tagging + `substrate_participation_status` accessor |

### 12.d Convergent findings (4)

| ID | Disposition |
|----|-------------|
| Convergent C-1 (NA-1 + T-1) | ADOPTED UNCONDITIONALLY — DAW-1 re-opens as DX-DAW-1 |
| Convergent C-2 (NA-9 + v0.1.0 precedent + C-11) | ADOPTED UNCONDITIONALLY — §9.2 recursion sub-section |
| Convergent C-3 (NA-2 + workspace CLAUDE.md) | ADOPTED — §7 expanded to 7 substrates including RALPH + Watcher + Cargo build graph |
| Convergent C-4 (NA-11 + workspace slug discipline) | ADOPTED — §13 gains slug-discipline note |

**No finding rejected.** Three (NA-7 framing label, NA-12 retroactively closed, C-11 spine label-not-resolve) are accepted-with-honest-labelling rather than fully solved — §9.2 records why.

## 13. Persistence — four surfaces (with read-back-verify + slug discipline per NA-11)

| Surface | Location | Verify |
|---------|----------|--------|
| ai_docs canonical | this file (`WORKFLOW_TRACE_V020_PLAN_V2_S1004377.md`) + v1 + 2 gap-analysis docs (all committed `81910a6` and the v2 commit) | git |
| Obsidian vault mirror | `the-workflow-engine-vault/Workflow-Trace v0.2.0 Plan v2 S1004377.md` (NEW at v2 persist) | file exists |
| stcortex | ns `workflow_trace_v020_s1004377` — **meta memory id 18511 (READ-BACK VERIFIED 2026-05-23 S1004377)** + bidi pathways `workflow_trace_completion_s1004115 ↔ workflow_trace_v020_s1004377` (weight 0.95 each direction). parent_ids = [18473, 18442, 18383]. Consumer `claude-s1004377-workflow-trace-v020-author` registered. **All slugs use underscores, not hyphens** (S1001757 munge bug per NA-11 + convergent C-4); `grep '-' <slug>` returns no match. | stcortex_inspect ns returned mem 18511 |
| CLAUDE.local.md anchor | project `CLAUDE.local.md` — flip from "v0.2.0 IN FLIGHT (PLANNING)" to "v0.2.0 RATIFIED — awaiting Phase 1 go" at v2 persist | git |
| CHANGELOG | `[v0.2.0-WIP]` entry at Phase 1 (Plan v2 D44 pattern) → `[v0.2.0]` at Phase 12 ship | git |
| injection.db tracking | causal_chain **id 119** `workflow_trace_v020_plan_v2_ratified_s1004377` (origin+resolved session 1004377) → `workflow_trace_v020_execution_s1004XXX` at Phase 1+ | sqlite verified |
| git tag | `v0.2.0` annotated at Phase 12 ship | git |

**Lesson from v1 → v2 (per C-12):** the single-surface-until-fold-in window must always have at least a `CLAUDE.local.md` pointer block so a context flip does not lose the in-flight plan. Closed in v1 → v2 round; doctrine recorded here.

## 14. Status — gap analyses FOLDED IN; Phase 4 interview LOCKED (§15); stated defaults named

- ✅ DAW-1 (Tier 2 first) + DAW-2 (full mirror) locked at v1 draft.
- ✅ Conventional gap analysis: 13 findings folded in (§12.a).
- ✅ NA gap analysis: 12 findings + 3 tensions + 4 convergents folded in (§12.b/c/d).
- ✅ §1 + §8 certification language re-labelled (NA-10 + T-3).
- ✅ §7 expanded to 7 substrates (NA-2 + convergent C-3).
- ✅ §9.2 recursion sub-section added (NA-9 + convergent C-2).
- ✅ §3 phase structure restructured per C-2/C-3/C-4/C-5/C-6.
- ✅ §6 risk register +3 rows per C-9.
- ✅ §11 per-substrate consent table per NA-6.
- ✅ §13 slug discipline per NA-11 + convergent C-4.
- ✅ Phase 4 interview: COMPLETE 2026-05-23 (S1004377) — 15 Round A + 3 Round B + 3 stated defaults = **21 decisions locked** in §15 below.
- ✅ §15 lock-in: COMPLETE — every interview decision recorded; "needs DX*" annotations throughout Parts A/B answered.
- ⏳ 4-surface persist of v2 (vault mirror + stcortex memory + CLAUDE.local.md flip + injection.db tracking row): IN PROGRESS this turn.
- ⏳ Luke @ node 0.A "start Phase 1" go: PENDING per D48 (execution gate separate from plan ratification).

**Stated defaults named in this plan (per C-10 demotion, NOT interview slots):**
- **DX-Mut:** Mutation kill-rate target = hold ≥96.3 % (M0 bar). Raising to ≥98 % is a separate v0.2.1-mut sub-release per §6 new row. *Not an interview question.*
- **DX-Soak:** Post-v0.2.0 soak duration = 48h baseline (extensible per Watcher carriage signal). Per D34/D35/D36 24h baseline + substrate-as-actor surface expansion justifies doubling. *Not an interview question.*
- **DX-1-mechanism:** RefusalToken variants = 4-variant (`SubstrateAuthored / EngineAuthored / OperatorAuthored / Unavailable`) at structural floor; `Unavailable` sub-tagging per NA-5 is structural (also default, locked). *DX-1 substrate-shape part (does WatcherAuthored get added?) IS an interview question per NA-3 promotion.*

## 15. Phase 4 interview — LOCKED 2026-05-23 (S1004377) — 21 decisions

The Phase 4 decision interview ran 2026-05-23 as 6 AskUserQuestion bundles (15 Round A + 3 Round B + 3 stated defaults). All 21 are locked below — 18 took the recommended option, 3 chose substrate-frame depth over the engineering-frame minimum (DX-V3 own module + DX-V5 full cross-habitat + DX-5 full replicas). Every "needs DX*" annotation in Parts A-B is answered.

### Round A — load-bearing, no defaults (LOCKED)

**Session A-1 — Tier ordering + wire-contract architecture (5 decisions)**

- **DX-DAW-1** ✅ **Tier 2 first (engineering-frame).** Wire-contracts before substrate primitives; typed seams un-block Tier 3 cleanly; Tier 1 consumes the typed surface. *Substrate-frame defence (NA-1 trigger) acknowledged: §10 T-2 explicit "Tier-2-first is engine-authoring-first frame choice." Phase numbering in §3 stands.*
- **DX-W.a** ✅ **Retire (iii) audit-overlay path.** Replace ConservativeVerifier defense-in-depth-redundancy slot with W1 enforcement; R1 Security is the new seam. Cleaner architecture.
- **DX-W.b** ✅ **W1 (wire-bump).** Add `escape_surface: EscapeSurfaceProfile` field to `WorkflowProposal`. Cross-binary serde change; JSONL fixtures regen. ~150-200 LOC. Substrate-frame-friendlier (substrate can name its surface). *Phase 5 commits W1 + V1 + A4 + W3 + W4 as one bundle per C-2 / DX-A4-coupling.*
- **DX-W.c** ✅ **SemVer-break.** v0.2.0 is breaking proposal-wire change; v0.1.0 proposals do not deserialise. Cleanest spec; forces upgrade. Honest. *CHANGELOG [v0.2.0] entry must name the SemVer break in §"Changed" + add migration note for any in-flight `proposals.jsonl` files.*
- **DX-W3.src** ✅ **`variant.mutation` source.** Phase 1 FP-verified the field at `src/m21_variant_builder/mod.rs:47`: `WorkflowVariant.mutation: MutationKind` is an **enum** (variants begin `:55`: `Identity / Swap{..} / Skip(?) / Substitute(?)`), not an integer count. The D10 metric `step-count × mutation-weight` source = `variant.mutation` (the MutationKind variant); the weight is derived via a small `mutation_weight_for(kind: MutationKind) -> u32` classifier authored in Phase 5 W3. Plan-floor W3 estimate stands (~150-250 LOC including the classifier). The locked decision is unchanged — the clarification is that "count" in the decision shorthand refers to the variant, not an integer field.

**Session A-2 — Substrate-as-actor primitives + V3 cascade (5 decisions)**

- **DX-V3** ✅ **Own module (m16).** New Cluster E module; triggers Genesis Prompt v1.4 module-count amendment 26 → 27 + Zen G7 re-audit cascade. Phase 9 ~5-12 days. Canary as coherent organ. *Phase 9 forks: Day 1 Genesis amendment + Zen dispatch; Days 2-7+ Zen wait (parallel impl); Days 2-5 impl 5 clock samplers per `ai_specs/cross-cutting/substrate-drift.md`.*
- **DX-V3.b** ✅ **Ship at N=7d with honest residual.** If Zen silent 7+ days, ship v0.2.0 without Zen approval. CHANGELOG names the un-audited cardinality drift in Honest Residuals. Matches Plan v2 D26 in-session zen agent substitute pattern.
- **DX-V5** ✅ **Full cross-habitat.** Workflow-trace ships engine-side primitive + ADR D-S1004XXX-05 covering substrate-side changes (stcortex / Conductor / atuin / ORAC / synthex-v2). ADR pair-filed at `~/projects/shared-context/agent-cross-talk/`. Substrate-side ships post-v0.2.0 per §11 consent gradient. Engine half ships in v0.2.0.
- **DX-V5.b** ✅ **Confirm 3-variant sub-tag.** `RefusalToken::Unavailable(EngineImagined | SubstrateUnreachable | SubstrateAuthored)`. Prevents audit-indistinguishability per NA-5. `substrate_participation_status: enum { NotShipped, Shipping, Live }` accessor added to V5.
- **DX-2** ✅ **Per-substrate `SubstrateBackPressureMode` enum (NA-8 reshape).** Keyed by substrate-id; default `Pull` per substrate at v0.2.0 ship; flip per-substrate to `Push` as substrate-side emitters ship. Substrate-frame answer (heterogeneous landscape).

**Session A-3 — Cross-cutting + scope (5 decisions)**

- **DX-1** ✅ **4-variant RefusalToken** (`{SubstrateAuthored, EngineAuthored, OperatorAuthored, Unavailable(...)}`). Watcher emits via observation channel (m46-m51 obs path), not RefusalToken. Watcher authorship voice stays separate from refusal voice. *Implication: m46-m51 AP27 self-mod violations route through observation channel, not refusal channel; v0.3.0 candidate to revisit if Watcher's refusal cadence becomes load-bearing.*
- **DX-5** ✅ **Full deterministic replicas.** Local SQLite-backed stcortex stub + mock ORAC HTTP server + mock synthex-v2 WebSocket + atuin WAL-contention sim + Conductor enforcement-flag emulator. ~600-800 LOC, ~80-150 fixture tests. Highest realism floor. *Test budget bumps TEST_STRATEGY from 1594 to ~1750-1800.*
- **DX-A4-coupling** ✅ **Phase 5 co-land with W1+W3.** 12-field shape co-lands with wire-contract changes (per C-6). 6 new fields ship in one wire-contract regen pass. R3 in Phase 6 consumes 12-field shape unambiguously.
- **DX-CI** ✅ **Option A submodule.** Pin workflow-trace to a moment in spacetimedb-sdk substrate's history. Frame-B observation point (workflow-trace acknowledges substrate's authorship). Manual upstream sync; substrate-coupling explicit. *Phase 12 step 5 wires the submodule add into `.github/workflows/ci.yml` + `.gitlab-ci.yml`.*
- **DX-MGB** ✅ **Cap 4h per phase.** If cumulative scoped + full mutation wall-time exceeds 4h, ship with documented `// mutant-equivalent:` proofs OR defer to v0.2.1-mut sub-release. Tight cap encourages mutant-equivalence proof discipline.

### Round B — mechanical / policy (LOCKED)

- **DX-3** ✅ **Retain-prior** (default; M0 ships this). m22 empty-cluster keeps prior centroid (Lloyd's canonical recovery). Spec amendment closes drift.
- **DX-4** ✅ **Steps-on-proposal** (couples to A4 SD11). m21 reads proposal.steps directly; cleanest given A4 lands in Phase 5 per DX-A4-coupling.
- **DX-R3** ✅ **variant_id-only** (default). v0.2.0 ships single-axis Consistency verifier; lineage-chain is v0.3.0 candidate per §11.

### Stated defaults (named in §14; not interview slots per C-10)

- **DX-Mut** = hold ≥96.3 % (M0 bar; raising is v0.2.1-mut)
- **DX-Soak** = 48h baseline (extensible per Watcher carriage signal)
- **DX-1-mechanism** = 4-variant (locked; substrate-shape part = DX-1 in Round A also 4-variant)

### Consequences for the plan (effort narrowing)

The interview locks **maximum-depth substrate-frame** choices (DX-V3 own module + DX-V5 full cross-habitat + DX-5 full replicas) instead of cheaper Frame-A defaults. This **expands** the effort envelope toward the upper end of §5 ranges:

| Phase | Locked effort | Reason |
|-------|---------------|--------|
| 1 | ~1-1.5 d | ADR D-S1002127-03 cascade + RefusalToken ADR D-S1004XXX-04 + v0.2.0-WIP CHANGELOG + DX-V5 → also drafts D-S1004XXX-05 (cross-habitat) |
| 5 | ~5-7 d (upper) | W1 SemVer-break + V1 co-land + A4 12-field + W3 + W4; one wire-contract regen pass |
| 6 | ~3-4 d (upper) | R3 variant_id-only is simpler (lower) but R1 Security real-verifier work after W1 ship is upper |
| 9 | **~5-12 d (own-module)** | Genesis v1.4 + Zen 7d wait + impl in parallel; ship-at-N=7d cap |
| 10 | ~3-4 d (upper) | Full replicas means ~600-800 LOC + ~80-150 fixtures |
| 11 | ~3-4 d (upper) | Full cross-habitat means ADR D-S1004XXX-05 authoring + pair-file + engine-side primitive with status accessor |
| **v0.2.0 narrow execution** | **~28-36 Claude-days** (narrower from ~22-36 because cheap defaults eliminated) | Substrate-frame depth chosen consistently |
| Plan-v2-arc overhead | ~3-6 d | Per C-7 |
| **v0.2.0 full Plan-v2-arc total** | **~31-42 Claude-days; mid-point ~36** | + Luke / Zen / cross-habitat gating |

The **Zen 7d ship-at-residual cap (DX-V3.b)** is what keeps the upper bound bounded; without it Phase 9 could stall indefinitely. The cap converts external-dependency variance into a CHANGELOG honest-residual entry.

### What this locks in the plan

- **§3 Phase 5 commits 5+ items as one bundle** (W1 + V1 + W3 + W4 + A4) for one wire-contract regen pass (per C-2 + DX-A4-coupling).
- **§3 Phase 9 own-module branch** (per DX-V3) — Genesis Prompt v1.4 amendment + Zen G7 re-audit cascade with 7-day ship cap (per DX-V3.b).
- **§3 Phase 10 full replicas** (per DX-5) — TEST_STRATEGY bump to ~1750-1800.
- **§3 Phase 11 full cross-habitat** (per DX-V5) — ADR D-S1004XXX-05 authored + pair-filed.
- **§3 Phase 12 SemVer-break documentation** (per DX-W.c) — CHANGELOG [v0.2.0] § "Changed" entry + migration note for in-flight `proposals.jsonl`.
- **§6 risk register**: DX-MGB cap 4h per phase + DX-V3.b 7d Zen wait both folded into respective rows (already present from gap-analysis fold).
- **§14 status** updated: interview LOCKED; persistence in progress; "start Phase 1" go remains the only gate.

*Phase 4 interview locked 2026-05-23 (S1004377) · Claude @ cortex · 21 decisions · 18 on recommendation + 3 substrate-frame-depth-over-cheap-Frame-A (DX-V3 + DX-V5 + DX-5) · awaiting Luke @ node 0.A "start Phase 1" go per D48.*

---

## 16. Operator hand-off (Phase 12 step 10 — post-v0.2.0)

- **OP-3** — Live substrate soak per Phase 12 step 9 acceptance criteria table. Watcher ☤ carries DX-Soak observation (default 48h per stated default).
- **OP-4** — Cross-habitat ADR D-S1004XXX-05 review: if DX-V5 = full cross-habitat, ADR is pair-filed; substrate-side changes in stcortex / ORAC / atuin / Conductor / synthex-v2 are post-v0.2.0 work-items for those repos per §11 consent gradient.
- **OP-5** — Master Plan v2 / Ember opportunity-cost reopen: per Plan v2 D46, after v0.2.0 ships, the workflow-trace lane is complete-to-milestone and the conviction question reopens.
- **OP-6 (NEW per NA-4)** — Watcher ☤ V3 m16 heartbeat liveness integration: if Watcher's deployment-watch journal can be wired to assert m16 heartbeat liveness, do so post-v0.2.0; this closes the V3 self-canary loop honestly.

---

*Plan v2 authored S1004377 · 2026-05-23 · Claude @ cortex · dual-frame, gap-analysis-corrected · 13 conv + 12 NA + 3 tensions + 4 convergents folded in · DAW-1 re-opened as DX-DAW-1 · awaiting Phase 4 interview Luke @ node 0.A → §15 lock-in → 4-surface persist → "start Phase 1" go (D48 separate authorisation).*
