# Workflow-Trace Completion Plan **v2** — S1004115 (close all outstanding tasks → v0.1.0 / M0)

> **Authored:** 2026-05-23 (S1004115) · **Status:** PLAN v2 — decisions LOCKED (§ 15), awaiting node-0.A go for Phase 1
> **Supersedes:** [`WORKFLOW_TRACE_COMPLETION_PLAN_S1004115.md`](WORKFLOW_TRACE_COMPLETION_PLAN_S1004115.md) (v1)
> **Re-baselined on:** git HEAD `6c3a5c5` (v1 was authored against the stale `2096fd0` — gap C-1).
> **Corrected by:** the dual-frame gap analysis —
> [`…_CONVENTIONAL_GAP_ANALYSIS.md`](WORKFLOW_TRACE_COMPLETION_PLAN_S1004115_CONVENTIONAL_GAP_ANALYSIS.md)
> (3 HIGH / 6 MED / 3 LOW) + [`…_NA_GAP_ANALYSIS.md`](WORKFLOW_TRACE_COMPLETION_PLAN_S1004115_NA_GAP_ANALYSIS.md)
> (9 frame gaps / 3 tensions / 2 convergent). Every finding's disposition is in § 12.
> **Back to:** [CLAUDE.local.md](../CLAUDE.local.md) · [CLAUDE.md](../CLAUDE.md) · [GATE_STATE.md](../GATE_STATE.md)

---

## What changed from v1 (gap-analysis corrections)

| v1 flaw | v2 fix |
|---------|--------|
| Authored against stale HEAD `2096fd0` (C-1) | Re-baselined on `6c3a5c5`; HEAD is now read at execution time, not frozen |
| §8 "NA pass" re-authored nothing + had no self-check (NA-1, NA-9) | §8 honestly relabelled; **Part B** is a genuine substrate-frame re-authoring **with** a frame-collapse self-check (§ 9) |
| Q1 asked node 0.A to decide already-shipped `--ack-ceiling` plumbing (C-2) | Q1 deleted; the genuinely-open bit (hard vs soft Refuse) is interview DB1; the wiring is a Phase-6a code task |
| Phase 4 "Cost verifier" assumed a `cost_estimate` field that does not exist (C-3) | **Verified:** `WorkflowProposal` has `evidence_lift`, no cost field. Cost is now an explicit decision (DB2): wire a new cost field cross-binary, or honest M0 Approve-stub |
| Phase 7 treated absent Zen verdicts as merely slow (C-4) | Phase 9 distinguishes verdict-**absent** from verdict-**slow**; adds a pre-M0 node-0.A escalation |
| Phase 4 treated 4 verifiers as a uniform batch (C-7) | Phase 6 split into 6a/6b/6c/6d with per-verifier estimates + ranges |
| Phases 1–2 claimed decision-free but m9-TODO couples to R1 (C-8) | m9 EscapeSurfaceProfile seam moved **into** Phase 6; decision-free work genuinely is |
| m2-DOC targeted auto-generated SpacetimeDB code (C-9) | m2-DOC deleted from scope |
| No CI / version-tag / dirty-tree / CHANGELOG-drift items (C-5, C-6) | New Phase 1 + Phase 2 tasks; CI + version-string are interview questions DB5/DB6 |
| §8 silently re-imported deferred NA-GAP-10 work unsized (C-11) | Substrate-frame M0 inclusions are named **and costed** in Part B + § 5 |
| Operator modelled as a 7-question oracle with cheap defaults (NA-3) | Interview split: Round A = 3 load-bearing questions, **no defaults**; Round B = mechanical, defaults flagged |
| `v0.1.0` would certify a substrate-facing organ without substrate validation (NA-7) | § 8 states explicitly what `v0.1.0` certifies and what it does **not** |

---

# PART A — Conventional plan

## 1. Mission

The 26-module codebase is implemented and hardened (1967 tests, clippy + pedantic clean,
96.3 % mutation kill-rate). This plan closes the remaining honest residuals, decision-gated
wiring, audit fold-ins, doc debt, and operator hand-offs, and reaches a **clean v0.1.0 / M0
tag**.

**What M0 / `v0.1.0` certifies** (the honest scope line — gap NA-7): *engine-internal
completeness* — every residual the engine owns is closed, tested, audited, and documented.
It is **not** a substrate-safety milestone. The engine's safety *as a substrate-facing
organ* — substrate-drift detection, substrate-side fixtures, substrate-mediated trust — is
the explicit subject of **v0.2.0** (§ 11). Part B re-derives this from the substrate frame
and § 8 states it as a single uncomfortable sentence, on purpose.

## 2. Verified inventory of outstanding items

Sourced from two recon passes + the gap analysis's spot-checks, re-verified at HEAD
`6c3a5c5`. Items the gap analysis corrected are marked.

### 2.1 Code residuals — decision-gated

| ID | Item | Evidence | Effort |
|----|------|----------|--------|
| **R1** | m33 dispatch verifiers — 4 `ConservativeVerifier` kinds unconditionally `Approve` (`src/orchestration/dispatch.rs:311–335`). Gate structure is real; the verdict is a documented placeholder | verified `dispatch.rs:333` | see Phase 6 (split) |
| **R2** | m22 K-means diversity never assembled on the `wf-crystallise` CLI path — `compose_proposals(&patterns,&snapshot,\|_v\|None)` (`src/orchestration/crystallise.rs:504–509`) | verified | ~135–235 LOC |

### 2.2 Code residuals — low-risk, no decision

| ID | Item | Note |
|----|------|------|
| **MUT-2** ⚠ | `m20/mod.rs:251` gap-restart mutant — m20 was **not** in the final W4 scope (`m10/m11/m21/m22`); Phase 2 confirms whether W2's m20 rewrite already kills it | ~1 test |
| **T4-batch** ⚠ | `T4-PORT` (spacetimedb-sdk absolute-path dep), `T4-AP30` (m42 anti-property grep misses child modules), `T4-LIB` (m32 `self_dispatch_guard` not re-exported; m11 `chrono_now_ms` unused), `T4-API` (4 test-seam gaps) | FP-verify each first |
| ~~m2-DOC~~ | **DELETED** (gap C-9) — `m2_stcortex_consumer/module_bindings/mod.rs` is auto-generated SpacetimeDB code; its TODO is upstream boilerplate and any edit is destroyed on regeneration | — |

### 2.3 Code residuals — needs an interview answer

| ID | Item | Interview |
|----|------|-----------|
| **m9-TODO** | Wire the `EscapeSurfaceProfile` 7-variant capability table into the m9 validator (`src/m9_watcher_namespace_guard/validator.rs:169–177`). It must read m32's `HumanAcceptanceSignature` via a not-yet-defined trait — the **same EscapeSurfaceProfile seam R1's Security verifier touches** (gap C-8). Designed once, in Phase 6 | folded into Phase 6 |
| **T4-DEAD-ERR** | ~15 dead error variants across ~9 modules — keep+test, or `#[deprecated]` (a SemVer-visible change, gap C-8) | DB7 |

### 2.4 Architectural decision — node 0.A

| ID | Item | Interview |
|----|------|-----------|
| **CC-7 / H5** | `PressureEvent` (m15) has **zero downstream consumers**. Wire m15 → m23 proposal composition, or formally document CC-7 observability-only. The substrate frame (Part B) reframes this as "keep or sever the substrate's only compositional voice" | DA1 (no default) |

### 2.5 Audit — Zen-paced *(gap C-4: verdicts absent, not slow)*

| ID | Item | Status |
|----|------|--------|
| **ZEN-W** | Review **requests** for Hardening W1–W4 filed (`agent-cross-talk/2026-05-21T*`). **Zero verdict files exist** for any workflow-trace wave. The S1003733/C22 packets are Command-authored notices, not verdicts | absent |
| **SD1–SD12** | 12 spec-drift items filed to Zen 2026-05-20T08:00Z — "no Zen reply" 3 days later. 8/12 Class A/B (code ahead of spec → reconcile spec); 4/12 Class C (algorithmic → v0.2.0) | absent |

### 2.6 Doc / persistence debt — no decision

| ID | Item |
|----|------|
| **DOC-1** | S1003733 has no stcortex surface — write the memory in ns `workflow_trace_hardening_2026_05_21` (**read-back-verified** — § 13, gap NA-6) |
| **DOC-2** | `HARDENING_FLEET_CARRY_FORWARD_S1002600.md` is a stale pre-Fleet scout pass — supersede it |
| **DOC-3** | `CHANGELOG.md:31` says "254 caught, 94.4 %"; the verified figure is "259 caught, 96.3 %" (gap C-6). Reconcile — and add `CHANGELOG.md` to every reconcile set |

### 2.7 Repo hygiene — no decision *(gap C-5)*

| ID | Item |
|----|------|
| **HYG-1** | Dirty working tree: `M .obsidian/workspace.json`, `M` two Watcher journal files, `D "Pasted image …png"`, and CLAUDE.local.md flags `src/m30_bank/mod.rs` dirty. Commit-or-discard each before any `v0.1.0` tag |
| **HYG-2** | `mutants.out/` + `mutants.out.old/` (2.8 MB stale) — `.gitignore`d (verified), clean local dirs; the repeated `cargo-mutants` runs rotate them |

### 2.8 Operator actions — Luke @ terminal (cannot be agent-done)

| ID | Item |
|----|------|
| **OP-1 / B3** | Conductor bring-up: `~/.local/bin/devenv -c ~/.config/devenv/devenv.toml start --services weaver,zen,enforcer` → `curl :8141/health`. **Plus** the enforcement-state coupling (gap NA-4): `CONDUCTOR_ENFORCEMENT_ENABLED` defaults `0`; `wf-dispatch` verdicts are advisory until it flips |
| **OP-2 / G2** | Directory rename `the-workflow-engine/` → `workflow-trace/` — cosmetic, post-M0 |

### 2.9 Explicitly deferred — v0.2.0 (§ 11)

`NA-GAP-07` substrate-drift canary (~300 LOC) · `NA-GAP-08` substrate test fixtures (~500) ·
`NA-GAP-10` substrate-mediated trust (~400 + ADR). Deferred by ADR `D-S1002127-03`.

## 3. Phase structure (10 phases)

Every phase: implementation → 4-stage gate → commit → mark complete. **No phase collapse.**
Gate per phase, `${PIPESTATUS[0]}`-checked per stage:
```bash
CARGO_TARGET_DIR=./target cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic
CARGO_TARGET_DIR=./target cargo test --all-targets --all-features --release
```

### Phase 1 — Re-baseline + verify & reconcile  *(decision-free; immediate)* · ~0.5 day
1. Re-baseline: `git show 6c3a5c5 --stat`; subtract anything `6c3a5c5` already reconciled
   from this plan's doc tasks (it already corrected the W4 count + added an S1004115 note).
2. FP-verify every ⚠ item in § 2.2 against the live tree (`MUT-1`, `H8-rem`, `H9-rem`,
   `T4-DEAD-SERDES`, the m12/m21/m22/m31 + CC-1/3/4/6 integration files) — produce the
   **true residual list**, the authoritative successor to the carry-forward doc.
3. **DOC-3** — reconcile `CHANGELOG.md:31` (254/94.4 %) against the `mutants.out` artefact;
   make every surface cite **259 / 96.3 %**.
4. **DOC-2** — prepend a supersession banner to `HARDENING_FLEET_CARRY_FORWARD_S1002600.md`.
5. **HYG-1 / HYG-2** — resolve the dirty tree (commit-or-discard each file, including
   `src/m30_bank/mod.rs`); clean `mutants.out*`; verify `.gitignore`.
6. **DOC-1** — write the S1003733 stcortex memory; **read it back** (`stcortex sql "SELECT
   id FROM memory WHERE namespace='workflow_trace_hardening_2026_05_21'"`) — absence = phase
   failure (gap NA-6 / convergent C-1).
7. Commit: `docs(workflow-trace): Phase 1 — re-baseline, residual list, CHANGELOG reconcile`.

### Phase 2 — Deep FP-verification  *(decision-free; immediate; feeds the interview)* · ~0.5–1 day
1. **Wire-contract verification (settles gap C-3):** trace the `WorkflowProposal` JSONL
   bridge end-to-end (m23 → `wf-crystallise` write → `wf-dispatch` read → m30
   `AcceptedWorkflow`). Confirmed already: `WorkflowProposal` carries `evidence_lift`, **no
   cost field**. Determine precisely what a Cost verifier would need added and the
   cross-binary blast radius. **Output feeds DB2.**
2. **8-absorbed-NA-gap code audit (gap NA-8 / convergent C-2):** for each of the 8 Wave-4.B
   "absorbed" scaffold NA gaps, grep the shipped `src/` tree — is `RefusalToken` a Rust
   type? is the m42 outbox drain-policy (NA-GAP-06) implemented? is the CC-5
   substrate-confirmable receipt (NA-GAP-09) wired? Any spec-only gap becomes either an M0
   work-item or an honest correction to the "8/11 closed" project record. **Output feeds
   the interview + Part B.**
3. Catalogue the m33 verifier inputs each kind actually needs (Security ← `ack_ceiling`
   ✓ shipped; Consistency ← bank read access, check for a `client_ref()` seam; Ember ←
   rubric source; Cost ← step 1). **Output feeds DB1–DB3.**
4. Commit: `docs(workflow-trace): Phase 2 — wire-contract + NA-gap code audit`.

### Phase 3 — Low-risk code cleanup  *(decision-free; parallel with Phases 1–2)* · ~0.5–1 day
1. **MUT-2** — confirm/kill the m20 gap-restart mutant with a direct unit test.
2. **T4-batch** — only the items Phase 1 confirmed genuinely open: `T4-PORT`, `T4-AP30`,
   `T4-LIB`, `T4-API` test seams. Bundle into ≤ 2 thematic commits.
   (`T4-DEAD-ERR` is **excluded** — it needs DB7; executed in Phase 6/10.)
3. Gate green per commit.

### Phase 4 — Decision interview  ✅ COMPLETE (S1004115) — 48 decisions locked in § 15  · ~1–2 h
Per gap NA-3, the interview is **split** so the load-bearing questions are not buried in a
fatigue-inducing batch with cheap defaults.

**Round A — load-bearing (no defaults; each presented with its full framing):**
- **DA1 — CC-7 / H5.** Wire `PressureEvent` → m23, or sever it. Framed (per Part B) as
  "keep or remove the substrate's only voice in proposal composition." No default.
- **DA2 — Consistency verifier (R1 6d).** Implement bank-conflict detection for M0, or
  defer to v0.2.0. Heaviest verifier; may need a bank-accessor seam. No default.
- **DA3 — M0 certification scope.** Ratify that `v0.1.0` certifies *engine-internal*
  completeness and is explicitly **not** a substrate-safety milestone (§ 8, gap NA-7), or
  pull v0.2.0 substrate work forward into M0 (adds Phase 11). No default.

**Round B — mechanical / policy (defaults acceptable, each flagged):**
- **DB1** R1 Security verifier — Refuse-above-ceiling **hard**, or **soft** (Amend)?
- **DB2** R1 Cost verifier — given Phase 2's wire-contract finding (no cost field): add a
  cost field cross-binary, **or** ship Cost as a documented Approve-stub for M0?
- **DB3** R1 Ember verifier — full 7-trait rubric, or reduced M0 subset (clarity +
  safe-naming)?
- **DB4** R2 m22 feature-vector dimensions — confirm `{step-count-norm, mutation-kind
  one-hot ×3, Levenshtein-from-identity}` or amend.
- **DB5** CI — does M0 ship a CI pipeline (GitHub Actions + GitLab CI running the 4-stage
  gate on push)? (gap C-5)
- **DB6** Canonical version string — reconcile `Cargo.toml` `0.1.0`, CHANGELOG
  `v0.1.0-s1003733`, and the planned `v0.1.0` tag into one story. (gap C-5)
- **DB7** `T4-DEAD-ERR` — keep + test the ~15 dead error variants, or `#[deprecated]` them
  (SemVer-visible).

*Naming (old Q7): decided — keep `workflow-trace`; the G2 directory rename stays post-M0.
Stated, not asked (gap C-12).*

### Phase 5 — R2: m22 K-means CLI wiring + cluster emission  *(needs DB4)* · ~1.5–2 days
1. Add `extract_variant_features(&WorkflowVariant) -> Vec<f64>` in `m22_kmeans` (dims per DB4).
2. In `crystallise.rs`: assemble feature vectors across patterns → `m22::kmeans` → build
   the variant→cluster map → replace `\|_v\|None` with the real lookup closure.
3. **Substrate-frame addition (Part B / gap C-11, costed here):** include `diversity_cluster`
   in the proposal's stcortex / Nexus emission so the selection rationale is
   substrate-visible — ~20–30 LOC, in this estimate.
4. Tests: feature-extraction determinism (unit) + integration (`wf-crystallise` proposals
   carry non-`None` `diversity_cluster`). Re-run `cargo-mutants` scoped to m22.
5. Commit: `feat(workflow-trace): R2 — m22 K-means diversity wired + cluster emission`.

### Phase 6 — R1: m33 verifier policy logic, split  *(needs DA2, DB1–DB3)* · ~2–7 days
Split per gap C-7; each sub-phase its own commit + gate.

- **6a Security** — compare proposal `EscapeSurfaceProfile.ordinal()` ≤
  `Config.ack_ceiling.ordinal()`; Refuse/Amend per DB1. `ack_ceiling` is already plumbed.
  ~40–70 LOC. **~0.5 day.**
- **6b Ember** — score the proposal artefact against the rubric scope chosen in DB3.
  ~80–150 LOC. **~1–1.5 days.**
- **6c Cost** — per DB2: either ship a documented Approve-stub (~10 LOC + doc, **~0.25
  day**) **or** the cross-binary cost-field change (add field to `WorkflowProposal`,
  populate in `crystallise`, serialise through JSONL, deserialise in `dispatch`, verifier
  logic; ~150–250 LOC, **~2 days**, JSONL wire-contract risk — S112 class).
- **6d Consistency** — per DA2: implement bank-conflict detection (~100–150 LOC + a bank
  accessor seam if Phase 2 found none; **~1.5–2 days**) **or** documented-defer to v0.2.0
  (**~0.25 day** doc).
- **6e m9 EscapeSurfaceProfile seam** — wire the m9 validator's 7-variant table; define the
  m9 ↔ m32 `HumanAcceptanceSignature` trait **once**, shared with 6a (gap C-8). ~60–90 LOC.
  **~0.5 day.**
- **6f Substrate-confirmable verdict receipts** (Part B, costed here per gap C-11) — every
  Refuse/Amend verdict emits an observable `WireEvent`, not just an engine-internal return.
  ~60–100 LOC + tests. **~1 day.**
- Tests: ≥ 50/verifier; `cargo-mutants` scoped to m33 (multi-hour — budget it).
- Commits: one per sub-phase, `feat(workflow-trace): R1 6x — …`.

### Phase 7 — CC-7 resolution  *(needs DA1)* · ~0.25–0.5 day
- **If wire:** thread `PressureEvent` from m15 into m23 compose-priority; CC-7 integration
  test. **If observability-only:** update `CLAUDE.md` synergies § + `ARCHITECTURE.md`;
  drop the dead-edge claim; keep m15 as a register.

### Phase 8 — Integration & end-to-end  · ~0.5–1 day
1. **Environment matrix (gap C-10):** state what must be up. Split into *offline / dry-run
   exercise* (always runs) vs *live `--execute` smoke* (requires OP-1; deferred to the
   post-M0 dispatch soak — M0's definition-of-done records `--execute` as
   unverified-against-live-Conductor).
2. **Conductor enforcement-state assertion (gap NA-4):** `wf-dispatch` warns when it emits
   `Approve` into a Conductor with `CONDUCTOR_ENFORCEMENT_ENABLED=0` ("verdict advisory").
3. **Substrate-side load observation (gap NA-2):** while exercising `wf-crystallise` on live
   atuin, measure atuin's *own* foreground write latency before/during/after — perturbation
   is a finding, not a pass.
4. **Clock-coherence enumeration (gap NA-5):** document every clock the CC-5 loop data
   crosses (m11 recency, injection.db TTL sweep, stcortex pathway decay, atuin checkpoint);
   state per crossing whether the engine assumes monotonic agreement. Coupling risks → § 6
   + v0.2.0.
5. Full 4-stage gate + full `cargo-mutants` re-run on every module touched in Phases 3–7.
6. Commit.

### Phase 9 — Zen audit fold-in + SD reconciliation  *(Zen-paced)* · ~1–2 days
1. **Absent-vs-slow gate (gap C-4):** if no Zen verdict has landed for W1–W4 / S1003733 /
   C22 by Phase 8 close, **escalate to node 0.A as a decision** — ship M0 un-audited / hold
   for audit / substitute an independent reviewer (`/ultrareview` or the `zen` agent). Do
   not let "v0.1.1 point release" silently absorb the absence of *any* audit.
2. Fold landed verdicts: APPROVE → record; AMEND → tracked sub-task with its own gate.
3. **SD1–SD12** — reconcile the 8 Class-A/B items (update spec docs to shipped code); the
   4 Class-C items are recorded as v0.2.0 fold-ins.
4. Commit.

### Phase 10 — M0 / v0.1.0 ship  · ~0.5 day
1. Final 4-stage gate + final mutation run — record exact counts.
2. **DB5** — if CI chosen: add `.github/workflows/` + `.gitlab-ci.yml` running the gate.
3. **DB6** — apply the canonical version string; `CHANGELOG.md` → `v0.1.0` (in the reconcile
   set — gap C-6).
4. **DB7** — apply the dead-error-variant decision.
5. Reconcile `CLAUDE.md` / `CLAUDE.local.md` / `GATE_STATE.md` / `ARCHITECTURE.md` /
   **`CHANGELOG.md`** to the M0 state.
6. **Four-surface persist** this completion, each stcortex write **read-back-verified**
   (§ 13, gap NA-6).
7. Tag `v0.1.0`; commit; push origin + gitlab.
8. Hand operator items to Luke: **OP-1** Conductor bring-up + enforcement flip, **OP-2**
   the G2 directory rename.

## 4. Sequencing & dependency graph

```
Phase 1 ─┐
Phase 2 ─┼─ decision-free, run first/parallel ── Phase 4 (interview, informed by P2)
Phase 3 ─┘                                          │
                                  ┌─────────────────┼─────────────────┐
                                  Phase 5 (R2)   Phase 6 (R1, 6a-6f)  Phase 7 (CC-7)
                                  └─────────────────┼─────────────────┘
                                              Phase 8 ── Phase 9 ── Phase 10
```
- **Critical path:** Phase 2 → Phase 4 → Phase 6 → Phase 8 → 9 → 10.
- Phase 4 deliberately runs **after** Phase 2 (gap C-8): the interview is informed by the
  wire-contract + NA-gap-code findings, so DB2/DA2 are answered with evidence, not guesses.
- Phase 9 is Zen-paced; its absent-vs-slow gate (step 1) is itself a node-0.A decision and
  must not silently slip M0.

## 5. Effort roll-up *(honest ranges — gap C-7; range driven by interview outcomes)*

| Phase | Effort | Driver of the range |
|-------|--------|---------------------|
| 1 — re-baseline + reconcile | ~0.5 day | — |
| 2 — deep FP-verification | ~0.5–1 day | size of the 8-NA-gap audit |
| 3 — low-risk cleanup | ~0.5–1 day | how many T4 items survive FP-verify |
| 4 — interview | ~1–2 h | Luke availability |
| 5 — R2 | ~1.5–2 days | — |
| 6 — R1 | **~2–7 days** | DB2 stub vs cost-wire (±2 d); DA2 defer vs Consistency (±2 d) |
| 7 — CC-7 | ~0.25–0.5 day | wire vs document |
| 8 — integration | ~0.5–1 day | — |
| 9 — Zen + SD | ~1–2 days | Zen pace |
| 10 — M0 ship | ~0.5 day | + CI if DB5=yes (~0.5 d) |
| **M0 total** | **~10–13 working days** of Claude-code effort (decisions locked — § 15) | + Luke/Zen gating |

v1 quoted ~7–12 days; v2's pre-interview range was ~9–18. The Phase 4 interview (§ 15) locked
DB2 → Cost stub and DA2 → Consistency defer, so Phase 6 lands near its low end and the **M0
roll-up narrows to ~10–13 Claude-days**.

## 6. Risk register

| Risk | Mitigation |
|------|------------|
| Re-doing already-closed carry-forward items | Phase 1 FP-verifies every ⚠ item — carry-forward doc is evidence, not fact |
| DB2 cost-wire change is a JSONL wire-contract change (S112 class) | Phase 2 sizes the blast radius *before* DB2 is answered; 6c isolated as its own sub-phase + gate |
| Interview answered without evidence | Phase 4 runs after Phase 2; DA2/DB2 informed by the wire-contract + accessor-seam findings |
| Mutation regressions from R1/R2 | Per-sub-phase scoped `cargo-mutants` (5, 6) + full re-run Phase 8 |
| **Zen never audits the hardening campaign** | Phase 9 step 1 escalates verdict-absence to node 0.A as an explicit decision — not silently absorbed |
| M0 tagged on engine-internal completeness only | § 8 names exactly what `v0.1.0` certifies; v0.2.0 owns substrate-safety |
| §8/Part B re-imports v0.2.0 work unsized | The two substrate-frame M0 inclusions are costed in Phase 5 step 3 + Phase 6f |
| Agent over-claims a phase gate-clean | Orchestrator re-runs the full `--workspace --all-targets --all-features` gate + `git log -1` independently per phase |
| `--execute` path never run against a live Conductor | Phase 8 states it as an accepted M0 limitation; covered by the post-M0 dispatch soak |
| Substrate clock-incoherence in the CC-5 loop (m11 recency vs injection.db TTL) | Phase 8 step 4 enumerates + documents; live coherence is a v0.2.0 item |

---

# PART B — Substrate-frame pass *(the genuine second authoring — gap NA-1, NA-9)*

> v1's §8 was a Frame-A self-audit mislabelled as the substrate frame: it re-authored
> nothing and had no self-check. This Part B is the second pass the dual-frame discipline
> asks for — it re-authors the question *"what does complete mean?"* from the frame where
> atuin, stcortex, injection.db, HABITAT-CONDUCTOR, and Luke are **actors with their own
> lifecycle, attention budget, decay clock, and refusal capacity** — not sinks the engine
> queries and dispatches into.

## 7. Re-authoring "complete" from the substrate frame

Part A's "complete" = *every residual the engine owns is closed*. The substrate frame asks
a different question: **complete to whom?** Five substrate actors each have a stake:

- **atuin / injection.db / stcortex (read-side substrates).** The engine's Cluster-A
  consumers hold read locks on a WAL-mode SQLite the user's shell is concurrently writing.
  "Complete" in their frame means *the engine's reads do not starve the substrate's own
  writes* — which Part A never measures. → folded in as **Phase 8 step 3** (substrate-side
  load observation). The deeper fix — a substrate-side back-pressure budget the substrate
  itself emits — is genuinely v0.2.0 (NA-GAP-04/07).
- **HABITAT-CONDUCTOR (dispatch substrate).** Its `CONDUCTOR_ENFORCEMENT_ENABLED` flag,
  24 h NoOp soak, and `auto_start=false` are a substrate-owned enforcement clock the engine
  does not control. "Complete" means *the engine knows whether its verdicts are being
  honoured*. → folded in as **Phase 8 step 2** (enforcement-state assertion).
- **The CC-5 loop's clocks.** m11 recency runs on the engine wall-clock; injection.db runs
  an hourly TTL sweep; stcortex decays pathway weights on its own schedule. The same
  pattern can be "recent" to m11 and "swept" by injection.db. "Complete" means *the engine
  has named which clock crossings it assumes agree*. → **Phase 8 step 4** (enumeration);
  live reconciliation is v0.2.0 (NA-GAP-07).
- **Luke @ node 0.A (operator substrate).** Has an attention budget and a refusal capacity
  ("this question is malformed", "not now"). "Complete" means *the consent surface does not
  route fatigue to the cheapest answer*. → folded in as the **Phase 4 split** (Round A,
  no defaults) + an explicit operator-refusal path: "not answerable as posed" is a
  first-class interview response.
- **stcortex as a write target.** Enforces refuse-write for unregistered consumers and
  refuses hyphen-bearing slugs at the reducer. "Complete" means *the engine verifies its
  own writes landed*. → **§ 13 read-back-verify**, applied to Phase 1 DOC-1 and Phase 10.

**The re-authored conclusion:** a substrate-frame "complete" requires substrate-drift
detection, substrate-side fixtures, and substrate-mediated trust — i.e. v0.2.0. M0 can
honestly install only the **seams** above (load observation, enforcement-state assertion,
clock enumeration, read-back-verify). It cannot honestly call itself substrate-complete.

## 8. What `v0.1.0` certifies — and does not *(gap NA-7, the named cut)*

> **`v0.1.0` / M0 certifies engine-internal completeness: every residual the engine owns is
> closed, tested, audited, and documented. It does NOT certify the engine's safety as a
> substrate-facing organ — substrate-drift detection, substrate-side test fixtures, and
> substrate-mediated trust are absent by deliberate deferral. The tag tells the habitat
> "the engine is done to a milestone." It does not tell the habitat "the engine is safe to
> run continuously against a live substrate." That assurance is `v0.2.0`.**

The NA analysis said: if that sentence is uncomfortable to write, the discomfort is the
finding. It is written. DA3 asks node 0.A to ratify it or to pull v0.2.0 forward.

## 9. Frame-collapse self-check *(gap NA-9 — the recursion step v1 omitted)*

Interrogating Part B itself — *is this the substrate's frame, or the engine's model of it?*

- **Phase 8's "substrate-side load observation" is still the engine measuring.** A true
  substrate-frame mechanism has atuin emit its *own* contention signal. v2's version is the
  engine timing a proxy write. **Honest label: a Frame-A proxy for a Frame-B property.**
  Adequate for M0 as a tripwire; the real mechanism is v0.2.0 (NA-GAP-04). Not hidden.
- **"Substrate-confirmable verdict receipts" (Phase 6f) — engine legibility or substrate
  agency?** Per gap NA-1, a receipt the engine emits is the engine narrating itself. It is
  *not* the substrate acting. v2 keeps Phase 6f because operator-visible Refuse/Amend
  events are genuinely useful — but labels it honestly: **6f is engine-legibility, not
  substrate-mediated trust.** Substrate-mediated trust stays NA-GAP-10 / v0.2.0.
- **Does Part B re-author, or annotate?** It re-authors the *completion criterion* (§ 7–8)
  and that re-authoring changed the plan: the Phase 4 split, the named cut, four folded-in
  seams, the v0.2.0 boundary defended on merits. That is more than v1's §8 did. But it did
  **not** change the phase count's core — Phases 5/6/7 are still engine-internal work. That
  is correct and not a collapse: the *engine-internal residuals are real and do need
  closing*. The collapse v1 committed was calling that work "substrate frame." v2 calls it
  what it is and partitions honestly.

**Self-check verdict:** Part B is a genuine second pass on the *completion criterion* and
an honest Frame-A/Frame-B split on the *work items*. Two mechanisms (load observation,
verdict receipts) are explicitly flagged as Frame-A proxies, not substrate agency. No
collapse is concealed.

## 10. Frame tensions — explicit reconciliations

- **T-1 — "smallest honest M0" vs "a substrate-facing organ must validate its substrate
  relationship before a milestone."** Reconciled by § 8: `v0.1.0` is named as an
  engine-internal certificate, not a substrate-safety one. The cut is visible.
- **T-2 — "one efficient interview round" vs "the operator is a substrate with a fatigue
  budget; DA1/CC-7 is a load-bearing severance."** Reconciled by the Phase 4 split: Round A
  is 3 no-default questions; CC-7 cannot be cleared by a cheap default.
- **T-3 — "§8 satisfies the dual-frame discipline" vs "§8 was a Frame-A self-audit."**
  Reconciled by relabelling v1's §8 and writing this Part B with a self-check (§ 9).

---

# PART C

## 11. v0.2.0 partition *(named as a frame choice — gap NA-7)*

`NA-GAP-07` (substrate-drift canary `m16`), `NA-GAP-08` (substrate test fixtures),
`NA-GAP-10` (substrate-mediated trust) are deferred by ADR `D-S1002127-03`. v2 keeps the
partition **and names it honestly as a frame choice**: every v0.2.0 item is a
substrate-as-actor item, every M0 item is engine-internal — the milestone boundary *is* the
frame boundary. That is a legitimate scope decision (a ~1,200-LOC milestone with its own
ADR), but it is now a *stated* one, ratified at DA3, not an implicit one. If node 0.A pulls
it forward, this plan gains **Phase 11 (v0.2.0)** after Phase 10.

## 12. Gap-analysis disposition *(every finding accounted for)*

| Finding | Disposition in v2 |
|---------|-------------------|
| C-1 stale HEAD | ACCEPTED — re-baselined `6c3a5c5`; HEAD read at execution |
| C-2 Q1 mis-scoped | ACCEPTED — Q1 deleted; wiring → Phase 6a code task; hard/soft → DB1 |
| C-3 Cost field absent | ACCEPTED — verified; Phase 2 sizes it; DB2 decides; 6c isolated |
| C-4 Zen verdicts absent | ACCEPTED — Phase 9 step 1 absent-vs-slow node-0.A escalation |
| C-5 CI / version / hygiene | ACCEPTED — Phase 1 HYG-1/2; DB5 CI; DB6 version string |
| C-6 CHANGELOG 94.4 % | ACCEPTED — DOC-3, Phase 1; CHANGELOG in every reconcile set |
| C-7 R1 effort optimistic | ACCEPTED — Phase 6 split 6a–6f; roll-up range raised to 9–18 d |
| C-8 false parallelism | ACCEPTED — m9-TODO → Phase 6e; interview after Phase 2 |
| C-9 m2-DOC generated code | ACCEPTED — m2-DOC deleted from scope |
| C-10 Phase 6 `--execute` | ACCEPTED — Phase 8 environment matrix + stated M0 limitation |
| C-11 §8 re-imports unsized | ACCEPTED — Phase 5 step 3 + Phase 6f costed explicitly |
| C-12 interview questions | ACCEPTED — Q1 dropped, Q7 demoted, DB5/DB6/DB7 added |
| NA-1 §8 engine-legibility | ACCEPTED — v1 §8 relabelled; Part B re-authors; 6f labelled honestly |
| NA-2 read-side substrates | ACCEPTED — Phase 8 step 3 load observation |
| NA-3 operator as oracle | ACCEPTED — Phase 4 split; Round A no defaults; refusal path |
| NA-4 Conductor as sink | ACCEPTED — Phase 8 step 2 enforcement-state assertion |
| NA-5 engine vs substrate clocks | ACCEPTED — Phase 8 step 4 clock enumeration; live recon → v0.2.0 |
| NA-6 unverified own writes | ACCEPTED — § 13 read-back-verify (Phase 1, Phase 10) |
| NA-7 partition is a frame choice | ACCEPTED — § 8 names the cut; DA3 ratifies; § 11 honest |
| NA-8 8 absorbed gaps spec-only | ACCEPTED — Phase 2 step 2 code audit; corrects the record |
| NA-9 no self-check | ACCEPTED — § 9 frame-collapse self-check |
| T-1/T-2/T-3 | ACCEPTED — § 10 reconciliations |
| Convergent (read-back; 8-gap code audit) | ACCEPTED — § 13; Phase 2 step 2 — highest confidence |

No finding rejected. Two (NA-1 6f, NA-2 load-obs) are **accepted-with-honest-labelling**
rather than fully solved — § 9 records why.

## 13. Persistence (this plan, four surfaces — with read-back-verify)

| Surface | Location | Verify |
|---------|----------|--------|
| ai_docs canonical | this file + the two gap-analysis docs | git |
| Obsidian vault mirror | `the-workflow-engine-vault/Workflow-Trace Completion Plan v2 S1004115.md` | file exists |
| stcortex | ns `workflow_trace_completion_s1004115` — meta memory + bidi pathway to `workflow_trace_hardening_2026_05_21` | **`SELECT id` read-back; absence = failure** |
| CLAUDE.local.md anchor | project `CLAUDE.local.md` — a "Completion Plan v2" pointer row | git |

Every stcortex write (here and in Phases 1/10) is read back; a silently-failed write
degrades four-surface persistence to three (the POVM write-only trap — gap NA-6).

## 14. Status — all design decisions locked

The Phase 4 interview is **complete** (S1004115, 2026-05-23) — all 48 decisions are locked
in **§ 15**. Every "needs DAx / DBx" annotation in Parts A–B is now answered; the plan is
fully specified.

**The single remaining gate:** Luke @ node 0.A gives the explicit go for **Phase 1**. Per
D48, execution is a separate authorisation from plan approval — nothing starts until that
word. Phases 1–3 are decision-free; Phases 5–7 are unblocked by § 15.

---

## 15. Phase 4 interview — locked decisions (S1004115)

The Phase 4 decision interview ran 2026-05-23 as a 12-round / 48-question grilling
(node 0.A). All 48 are locked below — 47 took the recommended option, 1 deviated (**D26**).
The phase bodies' "needs DAx / DBx" annotations are answered here.

**Round 1 — Scope & milestone**
- **D1** M0 certifies **engine-internal completeness only** — substrate-safety is v0.2.0.
- **D2** workflow-trace is an **internal habitat milestone** — no crates.io / external users.
- **D3** **No deadline** — quality-first; every phase its own gate.
- **D4** **v0.2.0 is a real committed follow-on milestone**, not a shelf.

**Round 2 — m33 Security verifier & gate semantics**
- **D5** Security verdict above ceiling = **hard Refuse**.
- **D6** The m33 gate is **blocking** — a Refuse stops dispatch.
- **D7** Default `--ack-ceiling` = **Sandboxed** (most restrictive; fail-safe).
- **D8** Every Refuse/Amend verdict **emits a substrate-confirmable WireEvent** (Phase 6f).

**Round 3 — m33 Cost & Consistency verifiers**
- **D9** **Cost = documented Approve-stub for M0** (no cost field on the wire).
- **D10** If a real cost signal is ever wired: metric = **step-count × mutation-weight**.
- **D11** **Consistency verifier deferred to v0.2.0** — rules unspecified; M0 ships a stub.
- **D12** Bank-accessor seam: **add a read-only accessor on-demand**, not speculatively.

**Round 4 — m33 Ember verifier**
- **D13** Ember scope = **reduced M0 subset** (clarity + safe-naming + ambiguity).
- **D14** Ember assesses **proposal-artefact quality**.
- **D15** Ember **reuses m10's Ember CI machinery** where it fits.
- **D16** Ember below-bar verdict = **Amend** (quality issue, not a Refuse).

**Round 5 — R2 / m22 K-means**
- **D17** Features = **5-dim**: step-count-norm, mutation one-hot ×3, Levenshtein-from-identity.
- **D18** Diversity cluster **influences m31 selection** (not observability-only).
- **D19** Cluster count **k is adaptive** — derived from variant count.
- **D20** `diversity_cluster` is **emitted to stcortex/Nexus**.

**Round 6 — CC-7**
- **D21** **CC-7 is wired** — `PressureEvent` → m23 for M0.
- **D22** Pressure **modulates m23 compose-priority** (additive, bounded).
- **D23** **m15 is audited in Phase 2** before CC-7 wiring.
- **D24** The NA framing is **accepted** — CC-7 is the substrate's voice in composition.

**Round 7 — Audit governance**
- **D25** If no Zen verdict by M0: **substitute an independent reviewer**.
- **D26** ⚠ *DEVIATION* — the substitute is **the in-session `zen` agent**, not /ultrareview
  (habitat-native; no external billing).
- **D27** SD drift: **reconcile the 8 Class-A/B items now**; defer the 4 Class-C to v0.2.0.
- **D28** Mutation bar: **hold ≥96.3%**; re-verify every module touched in Phases 5–7.

**Round 8 — Release engineering**
- **D29** **CI ships with M0** — GitHub Actions + GitLab CI running the 4-stage gate on push.
- **D30** Version: **tag a clean `v0.1.0`**; drop the CHANGELOG `-s1003733` suffix.
- **D31** Dead error variants: **keep + test** (construct each).
- **D32** Directory rename `workflow-trace/`: **post-M0**.

**Round 9 — Operator / deployment plane**
- **D33** Conductor bring-up: **post-M0**, before the dispatch soak.
- **D34** `wf-dispatch --execute`: **dry-run verification is enough for M0**; live `--execute`
  is a documented post-M0 item.
- **D35** Enforcement: **24h NoOp soak**, then flip `CONDUCTOR_ENFORCEMENT_ENABLED=1`.
- **D36** The post-M0 dispatch soak is **carried by the Watcher ☤**.

**Round 10 — Substrate frame**
- **D37** Substrate-load: the **engine-timed proxy is adequate for M0** (honestly labelled);
  a true substrate-emitted signal is v0.2.0.
- **D38** Clock-coherence: **document the CC-5 clock crossings for M0**; fix in v0.2.0.
- **D39** Absorbed NA gaps: **correct the project record** where spec-only; implement in M0
  **only** what the M0 verifiers load-bearingly need.
- **D40** Cadence: **invocation-only** — M0 and v0.2.0; no daemon mode.

**Round 11 — Execution model**
- **D41** Executor: **Claude solo, sequential, in-session** (subagents for Phase 2 audits +
  the Phase 9 zen review only).
- **D42** Drift: **full independent re-verification every phase** (gate + `git log` + exercise).
- **D43** Per-phase done-evidence: **gate output + test-count delta + commit SHA** (+ scoped
  `cargo-mutants` for code phases) in the commit message + a running ledger.
- **D44** **Harness Tasks** track the 10 phases; **one commit per phase/sub-phase**.

**Round 12 — Strategic premise**
- **D45** Conviction: **complete it** — workflow-trace deserves completion.
- **D46** Opportunity cost: **workflow-trace M0 first**, then Master Plan v2 / Ember.
- **D47** Premise: **believed** — the mined-workflow signal is real; Phase 8's live-atuin
  exercise shows first evidence.
- **D48** After the interview: **fold in, persist, stop for explicit node-0.A approval**
  before Phase 1.

### Consequences for the plan

- **Effort narrows.** D9 (Cost stub) + D11 (Consistency defer) take Phase 6 to its low end:
  6a Security ~0.5 d · 6b Ember ~1–1.5 d · 6c Cost stub ~0.25 d · 6d Consistency stub
  ~0.25 d · 6e m9 seam ~0.5 d · 6f verdict receipts ~1 d ≈ **~3.5–4 days**. With CI (+0.5 d)
  the **M0 roll-up narrows from ~9–18 to ~10–13 Claude-days**.
- **The m33 gate at M0:** 4 verifier kinds — Security + Ember **real**, Cost + Consistency
  **documented stubs** — blocking, fail-safe (D5/D6/D7), verdict-emitting (D8).
- **R2 m22 is fully real** (D17–D20): not decorative diversity. CC-7 is wired (D21–D24).
- **D26 is the one deviation** — the audit substitute is the habitat-native `zen` agent;
  /ultrareview is excluded.
- **§ 14 is closed** — every design decision is locked; the only remaining gate is Luke's
  explicit "start Phase 1" (D48).

*Plan v2 authored S1004115 · 2026-05-23 · Claude @ cortex · dual-frame, gap-analysis-corrected · 48 decisions locked · awaiting Luke @ node 0.A go for Phase 1.*
