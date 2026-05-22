# Workflow-Trace Completion Plan — S1004115 (close all outstanding tasks → v0.1.0 / M0)

> ⚠️ **SUPERSEDED 2026-05-23 by [`WORKFLOW_TRACE_COMPLETION_PLAN_V2_S1004115.md`](WORKFLOW_TRACE_COMPLETION_PLAN_V2_S1004115.md).**
> This v1 is retained only as the record the dual-frame gap analysis was run against.
> v1's flaws (stale HEAD, mis-scoped Q1, a Cost verifier resting on a non-existent field,
> Zen-verdict optimism, and a §8 "NA pass" that re-authored nothing) are catalogued in
> [`…_CONVENTIONAL_GAP_ANALYSIS.md`](WORKFLOW_TRACE_COMPLETION_PLAN_S1004115_CONVENTIONAL_GAP_ANALYSIS.md)
> + [`…_NA_GAP_ANALYSIS.md`](WORKFLOW_TRACE_COMPLETION_PLAN_S1004115_NA_GAP_ANALYSIS.md)
> and corrected in v2. **Do not execute from this document — use v2.**
>
> **Authored:** 2026-05-23 (S1004115) · **Status:** SUPERSEDED by v2
> **Back to:** [CLAUDE.local.md](../CLAUDE.local.md) · [CLAUDE.md](../CLAUDE.md) · [GATE_STATE.md](../GATE_STATE.md)
> **Scope:** every outstanding recommendation / residual / deferred item for `the-workflow-engine`
> as of HEAD `2096fd0` (Hardening Fleet W0–W5 + assessment-remediation S1003733 both COMPLETE).
> **Persistence target:** four surfaces — this file (canonical) · vault mirror · stcortex
> `workflow_trace_completion_s1004115` · `CLAUDE.local.md` anchor.

---

## 1. Mission

The 26-module codebase is implemented, hardened (1967 tests, clippy + pedantic clean,
96.3 % mutation kill-rate), and shipped through C22. What remains is **not** core
architecture — it is a bounded set of honest residuals, decision-gated wiring, audit
fold-ins, and operator actions. This plan takes the project from "hardened scaffold +
documented residuals" to a **clean v0.1.0 / M0 milestone** with zero undocumented debt.

It does **not** smuggle the v0.2.0 NA-GAP deferrals into M0 — those are explicitly
deferred by ADR `D-S1002127-03` and are partitioned in § 9 as a separate milestone that
only a node-0.A decision un-defers.

---

## 2. Verified inventory of outstanding items

Two reconnaissance passes (doc enumeration + code scan) produced this list. Items marked
✅ in the source docs but **superseded by later commits** are excluded. Items whose status
is *uncertain because the source doc predates the Hardening Fleet* are marked ⚠ VERIFY —
Phase 1 resolves them.

### 2.1 Code residuals — need a decision input first

| ID | Item | Source | Effort | Decision needed |
|----|------|--------|--------|-----------------|
| **R1** | m33 dispatch verifiers — 4 `ConservativeVerifier` kinds unconditionally `Approve`; gate structure is real, the *verdict* is a documented placeholder (`src/orchestration/dispatch.rs:311–335`) | CLAUDE.local.md S1003733 §R1 | ~310–490 LOC | Policy inputs ×4 (see Phase 0 Q1–Q4) |
| **R2** | m22 K-means diversity never assembled on the `wf-crystallise` CLI batch path — `compose_proposals(..., \|_v\| None)` (`src/orchestration/crystallise.rs:504–509`); m22 runs + is tested, but its cluster signal never reaches proposals | CLAUDE.local.md S1003733 §R2 | ~135–235 LOC | Feature-vector dimensions (Phase 0 Q5) |

### 2.2 Code residuals — no decision needed (low-risk)

| ID | Item | Source | Effort |
|----|------|--------|--------|
| **m9-TODO** | Wire the `EscapeSurfaceProfile` 7-variant capability table into the m9 validator (`src/m9_watcher_namespace_guard/validator.rs:169–177`) — was gated on m32 shipping; m32 has shipped, so now unblocked | code scan | ~50 LOC, ~4 h |
| **m2-DOC** | `src/m2_stcortex_consumer/module_bindings/mod.rs:1059` — doc-debt TODO on the subscription API | code scan | ~1 h |
| **MUT-2** ⚠ | `m20/mod.rs:251` gap-restart `==`→`!=` mutant — m20 was **not** in the final W4 scope (`m10/m11/m21/m22`); confirm whether W2's m20 rewrite already kills it, else add one unit test | carry-forward §MUT-2 | ~1 test |
| **T4-batch** ⚠ | ~6 genuinely-minor items: `T4-PORT` (spacetimedb-sdk absolute-path dep → portable), `T4-AP30` (m42 anti-property grep misses child modules), `T4-LIB` (m32 `self_dispatch_guard` not re-exported; m11 `chrono_now_ms` unused), `T4-API` (4 test-seam gaps), `T4-DEAD-ERR` (~15 dead error variants → keep+test or `#[deprecated]`), `T4-MISC` (7 cosmetic) | carry-forward §T4 | ~1–1.5 days **total**, after FP-verify |

> ⚠ The carry-forward doc (`HARDENING_FLEET_CARRY_FORWARD_S1002600.md`) is a **scout pass
> from before the Hardening Fleet ran**. Waves B/C/D landed integration tests
> (`f8ab952` = m12/m21/m22/m31; `1c9b809`/`c4bfed4`/`e2acf65`/`711a662` = CC-1/CC-3/CC-4/CC-6
> + m13/m40/m41) and W4 added 68 mutant-killing tests. So `H8-rem`/`H9-rem`/`MUT-1` and much
> of `T4-DEAD-SERDES` are **probably already closed**. Phase 1 FP-verifies each before any
> work is scheduled — re-doing closed work is the failure mode here.

### 2.3 Architectural decision — node 0.A / Zen

| ID | Item | Source | Decision |
|----|------|--------|----------|
| **H5 / CC-7** | `PressureEvent` (m15) has **zero downstream consumers** — the CC-7 "Pressure-Driven Evolution" edge is dead. Either wire m15 → m23 (pressure influences proposal composition) **or** formally document CC-7 as observability-only and drop the synergy claim | carry-forward §H5; filed to Zen 2026-05-20 | Phase 0 Q6 |
| **OI-5** | Naming: keep working name `workflow-trace`, or rename, or scope-honest rename | vault tracker OI-5 | Phase 0 Q7 (cosmetic) |

### 2.4 Audit fold-in — Zen-paced

| ID | Item | Status |
|----|------|--------|
| **ZEN-W** | Zen audit packets filed for Hardening W1/W2/W3/W4 + S1003733 remediation + C22 docs (`agent-cross-talk/2026-05-21T*` … `2026-05-22T135545Z`). **No verdict files observed.** Verdicts must be folded in when they land — APPROVE closes the gate; AMEND spawns work | cross-talk scan |
| **SD1–SD12** | 12 spec-drift items filed to Zen (V4 audit, `2026-05-20T080000Z`). 8/12 Class A/B = *code ahead of spec* → reconcile the spec docs to the shipped code. 4/12 Class C = algorithmic divergence → v0.2.0 fold-in. Not a live blocker | carry-forward §SD |

### 2.5 Doc / persistence debt — Claude-code, no decision

| ID | Item |
|----|------|
| **DOC-1** | S1003733 remediation is persisted on 3 surfaces (ai_docs, vault, CLAUDE.local.md) but **not stcortex** — write the missing memory in ns `workflow_trace_hardening_2026_05_21` |
| **DOC-2** | `HARDENING_FLEET_CARRY_FORWARD_S1002600.md` is stale (items closed by later waves not marked) — supersede it with a one-line pointer to this plan + Phase 1's verified residual list |

### 2.6 Operator actions — Luke @ terminal (cannot be done by an agent)

| ID | Item |
|----|------|
| **OP-1 / B3** | Conductor bring-up: `~/.local/bin/devenv -c ~/.config/devenv/devenv.toml start --services weaver,zen,enforcer`; then `curl :8141/health`. Non-blocking until a live dispatch-plane soak is wanted |
| **OP-2 / G2** | Directory rename `the-workflow-engine/` → `workflow-trace/` — cosmetic, deferred to post-M0 by GATE_STATE |

### 2.7 Explicitly deferred — v0.2.0 milestone (NOT in M0 scope)

| ID | Item | LOC |
|----|------|-----|
| **NA-GAP-07** | First-class substrate-drift detection — new `m16_substrate_drift_canary` module | ~300 |
| **NA-GAP-08** | Substrate-side test fixtures (`tests/substrate_fixtures/`) | ~500 |
| **NA-GAP-10** | Substrate-mediated trust — cross-habitat ADR + Cluster-D rework | ~400 + ADR |

Deferred by ADR `D-S1002127-03` with compensating controls already in place. Un-deferring
is a conscious node-0.A decision — see § 9.

---

## 3. Phase 0 — Decision interview (node 0.A gate)

Per `feedback_structured_interview_before_code`: there are ≥ 5 architectural decisions
below; they are resolved in **one AskUserQuestion round-set before any R1/R2/CC-7 code is
written**. Phases 1–2 do **not** depend on these and can run immediately.

| Q | Decision | Why it gates code |
|---|----------|-------------------|
| **Q1** | **R1 Security verifier** policy source — operator `--ack-ceiling` CLI flag (already threaded to m32) routed into m33? a policy file? both? | Determines the Security verifier's input contract |
| **Q2** | **R1 Cost verifier** — what is the budget model? static ceiling, per-run estimate vs constant, or no real cost gate for M0 (keep Approve, documented)? | Determines whether Cost is implemented or consciously stays a stub |
| **Q3** | **R1 Ember verifier** — apply the 7-trait rubric to the proposal artefact, or a reduced M0 subset (e.g. clarity + safe-naming only)? | Determines Ember verifier scope |
| **Q4** | **R1 Consistency verifier** — implement bank-conflict detection now, or document-and-defer to v0.2.0? | Consistency is the heaviest of the four (~80–120 LOC + bank access) |
| **Q5** | **R2 feature-vector dimensions** — confirm the m22 feature set: step-count (norm), mutation-kind one-hot ×3, Levenshtein-from-identity. Add/remove dimensions? | m22 has no canonical feature-extraction fn; dims must be fixed before wiring |
| **Q6** | **CC-7 / H5** — wire `PressureEvent` → m23 proposal composition, or document CC-7 as observability-only and drop the synergy from the architecture? | Either ~0.5 day of wiring or a doc-only change |
| **Q7** | **OI-5 naming** — keep `workflow-trace`, or rename. Cosmetic; affects only the post-M0 G2 directory rename | Low stakes; can default to "keep" |

**Default-if-no-answer (flagged, not silent):** Q1 → `--ack-ceiling` only; Q2 → keep Cost
as a documented Approve-stub for M0; Q3 → reduced subset; Q4 → defer Consistency to v0.2.0;
Q5 → the 5-dimension set above; Q6 → document observability-only (cheapest, honest); Q7 →
keep `workflow-trace`. These defaults yield the smallest honest M0; the interview exists to
let node 0.A buy more.

---

## 4. Execution phases

Each phase is self-contained: implementation → 4-stage quality gate → commit → mark
complete. **No phase collapse** (`feedback_no_shortcuts`). The gate every phase:

```bash
CARGO_TARGET_DIR=./target cargo check --all-targets --all-features 2>&1 | tail -20
cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tail -20
cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic 2>&1 | tail -20
CARGO_TARGET_DIR=./target cargo test --all-targets --all-features --release 2>&1 | tail -30
```
(`${PIPESTATUS[0]}` checked per stage — `feedback_pipestatus_for_gate_chains`.)

### Phase 1 — Verify & reconcile *(no decision needed; run immediately)*  · ~0.5 day
1. FP-verify every ⚠ item in § 2.2 against the live tree: grep/read for `H8-rem`, `H9-rem`,
   `MUT-1`, `T4-DEAD-SERDES`, the m12/m21/m22/m31 + CC-1/3/4/6 integration files. Mark each
   genuinely-closed or genuinely-open.
2. Produce the **true residual list** — the authoritative successor to the stale
   carry-forward doc.
3. **DOC-2:** prepend a supersession banner to `HARDENING_FLEET_CARRY_FORWARD_S1002600.md`
   pointing at this plan + the verified list.
4. **DOC-1:** write the missing S1003733 stcortex memory.
5. Commit: `docs(workflow-trace): Phase 1 — verified residual list + carry-forward supersede`.

### Phase 2 — Low-risk code cleanup *(no decision needed)*  · ~0.5–1 day
1. **m9-TODO** — wire the `EscapeSurfaceProfile` 7-variant capability table into the m9
   validator (m32 has shipped). ~50 LOC + tests.
2. **MUT-2** — confirm/kill the m20 gap-restart mutant with a direct unit test.
3. **T4-batch** — only the items Phase 1 confirmed open: `T4-PORT`, `T4-AP30`, `T4-LIB`,
   `T4-API` test seams, `T4-DEAD-ERR` resolution, `T4-MISC`. Each small; bundle into ≤ 3
   commits by theme (portability / test-seams / dead-code).
4. Gate green per commit.

### Phase 3 — R2: m22 K-means CLI wiring *(needs Q5)*  · ~1–2 days
1. Add `extract_variant_features(&WorkflowVariant) -> Vec<f64>` in `m22_kmeans` (dims per Q5).
2. In `crystallise.rs`: assemble feature vectors across all patterns → `m22::kmeans` →
   build the variant→cluster map → replace `\|_v\| None` with the real lookup closure.
3. Tests: unit (feature extraction determinism) + integration (`wf-crystallise` proposals
   carry non-`None` `diversity_cluster`).
4. Re-run `cargo-mutants` scoped to m22 — confirm no new survivors.
5. Commit: `feat(workflow-trace): R2 — wire m22 K-means diversity into wf-crystallise`.

### Phase 4 — R1: m33 per-kind verifier policy logic *(needs Q1–Q4)*  · ~2–4 days
1. Replace `ConservativeVerifier` with four real verifiers (or documented-stub for any kind
   Q2/Q4 chose to defer):
   - **Security** — compare `EscapeSurfaceProfile` against the operator ceiling (Q1).
   - **Cost** — validate `cost_estimate` against the budget model (Q2).
   - **Ember** — score the proposal artefact against the rubric subset (Q3).
   - **Consistency** — bank-conflict detection (Q4), or documented-defer.
2. Route `ack_ceiling` from the dispatch `Config` into the Security verifier.
3. Tests: per-kind verdict logic (Approve / Amend / Refuse paths), 50+/verifier; the m33
   `aggregate()` ordering invariant already covered.
4. Re-run `cargo-mutants` scoped to m33.
5. Commit: `feat(workflow-trace): R1 — real per-kind m33 verifier policy logic`.

### Phase 5 — CC-7 / H5 resolution *(needs Q6)*  · ~0.5 day
- **If wire:** thread `PressureEvent` from m15 into m23 proposal composition (pressure
  raises/lowers a pattern's compose priority); tests + the CC-7 integration test.
- **If observability-only:** update `CLAUDE.md` cross-cluster-synergies § + `ARCHITECTURE.md`
  to state CC-7 is observability-only; remove the dead-edge claim; keep m15 as a register.
- Commit accordingly.

### Phase 6 — Integration & end-to-end exercise  · ~0.5 day
1. Exercise both binaries beyond compile/test: `wf-crystallise` real report path on live
   atuin data; `wf-dispatch` dry-run + verification path against a `proposals.jsonl`.
2. Full 4-stage gate on the whole workspace.
3. Full `cargo-mutants` re-run on every module touched in Phases 2–5; record kill-rate.
4. Commit any test gaps the mutation run exposes.

### Phase 7 — Zen audit fold-in & SD reconciliation *(Zen-paced)*  · ~1–2 days
1. When Zen verdicts land for W1–W4 / S1003733 / C22 / the V4 SD audit: APPROVE → record;
   AMEND → schedule the amendment as a tracked sub-task with its own gate.
2. **SD1–SD12** — reconcile the 8 Class-A/B items (code ahead of spec) by updating the
   spec docs to the shipped code. The 4 Class-C items are recorded as v0.2.0 fold-ins.
3. Commit: `docs(workflow-trace): Phase 7 — Zen verdicts folded + SD1–SD12 spec reconcile`.

### Phase 8 — M0 / v0.1.0 ship  · ~0.5 day
1. Final 4-stage gate + final mutation run — record exact counts.
2. Update `CHANGELOG.md` → `v0.1.0`; reconcile `CLAUDE.md` / `CLAUDE.local.md` /
   `GATE_STATE.md` / `ARCHITECTURE.md` to the M0 state.
3. **Four-surface persist** this completion (ai_docs + vault + stcortex + CLAUDE.local.md).
4. Tag `v0.1.0` (M0); commit; push origin + gitlab.
5. Hand operator items to Luke: **OP-1** Conductor bring-up, **OP-2** the G2 directory
   rename (`workflow-trace/`), now formally post-M0.

---

## 5. Sequencing & dependency graph

```
Phase 1  ──┐ (no gate)
Phase 2  ──┤── run in parallel; both decision-free
           │
Phase 0 (interview) ──┬── Phase 3 (R2)   ──┐
                      ├── Phase 4 (R1)   ──┤── Phase 6 ── Phase 7 ── Phase 8
                      └── Phase 5 (CC-7) ──┘
```

- **Critical path:** Phase 0 → Phase 4 (R1, the largest) → Phase 6 → 7 → 8.
- Phases 1, 2 are pure prep — start them the moment this plan is approved, before the
  interview even runs.
- Phase 7 is Zen-paced; if verdicts are slow it must not block Phase 8's *code* freeze —
  fold AMEND work as a v0.1.1 point release rather than holding M0 hostage.

---

## 6. Effort roll-up

| Phase | Effort | Gated on |
|-------|--------|----------|
| 0 — interview | ~1–2 h | Luke availability |
| 1 — verify & reconcile | ~0.5 day | — |
| 2 — low-risk cleanup | ~0.5–1 day | — |
| 3 — R2 | ~1–2 days | Q5 |
| 4 — R1 | ~2–4 days | Q1–Q4 |
| 5 — CC-7 | ~0.5 day | Q6 |
| 6 — integration | ~0.5 day | 3,4,5 |
| 7 — Zen fold-in + SD | ~1–2 days | Zen |
| 8 — M0 ship | ~0.5 day | 1–7 |
| **M0 total** | **~7–12 working days** of Claude-code effort | + Luke/Zen gating |
| v0.2.0 (NA-GAP-07/08/10) | ~5–8 days, ~1,200 LOC | separate node-0.A decision |

---

## 7. Risk register

| Risk | Mitigation |
|------|------------|
| Re-doing already-closed carry-forward items | Phase 1 FP-verifies every ⚠ item before scheduling — the carry-forward doc is treated as evidence, not fact |
| R1/R2 built against guessed policy → rework | Phase 0 interview is a hard gate before Phase 3/4 code |
| Mutation regressions from new R1/R2 code | Per-phase scoped `cargo-mutants` (Phases 3, 4) + full re-run Phase 6 |
| Zen AMEND verdict arrives after M0 tag | Phase 7 note: AMEND work ships as v0.1.1, M0 code-freeze not held hostage |
| Scope creep — v0.2.0 NA-GAPs pulled into M0 under "complete everything" | § 9 hard-partitions them; un-deferring needs an explicit node-0.A decision |
| Drift: agent over-claims a phase gate-clean | Orchestrator re-runs the full `--workspace --all-targets --all-features` gate + `git log -1` independently per phase (`feedback_flex_verify_before_ship`) |

---

## 8. Frame check — non-anthropocentric pass

> *Working Mode: "write it once, then ask what frame is that? and write it again from the
> frame you didn't take."* §§ 1–7 above are the **conventional frame**: workflow-trace as an
> engineering artefact, residuals as tasks to close. This section is the **substrate frame**:
> workflow-trace exists to *observe the habitat substrate and propose workflows back into it*.
> Re-read from that frame, three things change:

1. **R1 done engine-internally re-opens NA-GAP-10.** The conventional plan implements the
   m33 verifiers with engine-internal policy (a ceiling flag, a budget constant, a rubric).
   But NA-GAP-10 ("Cluster-D trust is engine-internal, not substrate-mediated") was
   *deferred*, not solved. If R1's Security verifier hard-codes a ceiling the engine owns,
   M0 ships a trust gate the substrate cannot see or contest. **Mitigation folded into
   Phase 4:** every verifier verdict must emit a **substrate-confirmable receipt** (the
   convention already exists from NA-GAP-09) — Refuse/Amend verdicts become observable
   `WireEvent`s, not just engine-internal returns. This keeps M0's engine-internal policy
   *honest about being engine-internal* and leaves a clean seam for v0.2.0's
   substrate-mediated trust.

2. **R2 K-means clusters are an engine-private signal unless they are emitted.** The
   conventional plan threads `diversity_cluster` into proposals so m31 can use it. Substrate
   frame: the cluster assignment is a *judgement the engine makes about the substrate's
   observed behaviour* — if it never surfaces in a receipt, the substrate cannot tell why
   one variant was selected over another. **Phase 3 addition:** include `diversity_cluster`
   in the proposal's stcortex/Nexus emission so the selection rationale is substrate-visible.

3. **CC-7 is the substrate's backpressure channel — Q6 is not cosmetic.** Conventionally
   CC-7 is a "dead synergy edge". Substrate frame: `PressureEvent` is *how the habitat tells
   the engine it is under load*. Documenting it observability-only (the cheap default)
   silently removes the substrate's only voice in proposal composition. The interview must
   present Q6 with that framing — "drop the substrate's backpressure channel" — not as a
   tidy-up. If node 0.A picks observability-only, that is a legitimate M0 scope cut, but it
   must be a *seen* cut.

**Net:** the NA pass adds two concrete Phase-3/4 requirements (substrate-confirmable
verdict receipts; cluster emission) and reframes Q6. It does not change the phase count.

---

## 9. v0.2.0 partition (out of M0 scope — un-defer only by node-0.A decision)

`NA-GAP-07` (substrate-drift canary module), `NA-GAP-08` (substrate test fixtures), and
`NA-GAP-10` (substrate-mediated trust) are deferred by ADR
`ai_docs/decisions/2026-05-17-substrate-as-actor-deferrals.md` (`D-S1002127-03`) with
compensating controls live. "Complete *all* outstanding tasks" *could* be read to include
them — this plan deliberately does **not**, because:

- they are a ~1,200-LOC, ~5–8-day milestone with their own ADR and gap analysis;
- folding them into M0 would make the milestone unbounded and break the v0.1.0 / v0.2.0 line;
- the M0 NA pass (§ 8) already installs the *seams* (verdict receipts, cluster emission)
  that v0.2.0 builds on.

**If node 0.A wants them in scope**, this plan gains a Phase 9 (v0.2.0) appended after
Phase 8 — say so at Phase-0 interview time and it is added.

---

## 10. Persistence (this plan, four surfaces)

On approval, this plan is persisted per the Working Mode four-surface rule:

| Surface | Location |
|---------|----------|
| ai_docs canonical | this file — `ai_docs/WORKFLOW_TRACE_COMPLETION_PLAN_S1004115.md` |
| Obsidian vault mirror | `the-workflow-engine-vault/Workflow-Trace Completion Plan S1004115.md` |
| stcortex | ns `workflow_trace_completion_s1004115` — meta memory + bidi pathway to `workflow_trace_hardening_2026_05_21` |
| CLAUDE.local.md anchor | project `CLAUDE.local.md` — a "Completion Plan" pointer row |

---

## 11. Open decisions for node 0.A (summary)

1. **Approve this plan** as the path to M0 — or redirect scope.
2. **Phase 0 interview** — answer Q1–Q7 (or accept the flagged defaults in § 3).
3. **v0.2.0** — confirm NA-GAP-07/08/10 stay out of M0 (§ 9), or request a Phase 9.
4. **Execution authorisation** — Phases 1–2 are decision-free and can start on approval;
   Phases 3–5 need the interview; confirm "start coding" covers R1/R2 (G9 already fired,
   so code work is charter-authorised — this is a courtesy confirmation, not a gate).

*Plan authored S1004115 · 2026-05-23 · Claude @ cortex · awaiting Luke @ node 0.A.*
