# Phase 9 — Zen Audit Fold-in + SD1–SD12 Reconciliation

> **Authored:** 2026-05-23 (S1004115, Plan v2 Phase 9)
> **Per plan §3 Phase 9 + §15 D25–D28:** Zen audit absent-vs-slow gate · SD reconcile · audit substitute per D26
> **Back to:** [`WORKFLOW_TRACE_COMPLETION_PLAN_V2_S1004115.md`](WORKFLOW_TRACE_COMPLETION_PLAN_V2_S1004115.md) · [`PHASE1_RESIDUAL_LIST_S1004115.md`](PHASE1_RESIDUAL_LIST_S1004115.md) · [CLAUDE.local.md](../CLAUDE.local.md)

---

## 1. Zen audit fold-in (Plan §3 Phase 9 step 1+2 — gap C-4)

**Per §15 D25/D26:** if no external Zen verdict has landed by Phase 8 close → substitute = the in-session
`zen` agent (NOT `/ultrareview` — habitat-native; no external billing).

**Status at Phase 9 entry:** zero external Zen verdict files exist in
`~/projects/shared-context/agent-cross-talk/` for ANY workflow-trace wave (W1–W4, S1003733, C22) per
Phase 1 residual `ZEN-W`. The absent-vs-slow gate fires; substitute dispatched.

**In-session zen verdicts already on record this session** (per the Phase commit history):
- Phase 1: APPROVE-WITH-NITS (vault W4 row + MUT-1 path-typo nits folded in Phase 2)
- Phase 2: APPROVE
- Phase 3: APPROVE
- Phase 5: APPROVE-WITH-NITS (A2 — m31 caller still uses `|_w| 0.0`; deferred per plan to Phase 9)

**Phase 9 zen audit dispatched** (background `a8d14566a9473bd8e`) — covers the full hardening campaign
+ S1003733 + C22 + this session's Phases 6/7/8 commits. Per D43 the verdict packet folds in here when
it returns; if APPROVE / APPROVE-WITH-NITS → record and proceed. AMEND → tracked sub-task with own gate.

**Phase 5 nit (A2) disposition:** the m31 production caller at `wf-dispatch::run` uses `|_w| 0.0` for
the `diversity_score` closure (m22 K-means signal threaded to `proposal.diversity_cluster()` is
substrate-side complete but the consumer-side wire is not). Per the plan this remains an honest
residual at M0 — the substrate-facing contract of D18 is satisfied (the proposal carries the cluster);
the behavioural loop of m22→m31 ranking is a v0.2.0 fold-in once a production caller actually
consumes it. Documented here; not blocking v0.1.0.

---

## 2. SD1–SD12 reconciliation (Plan §3 Phase 9 step 3 — §15 D27)

Per §15 D27: reconcile the 8 Class-A/B items now; defer the 4 Class-C to v0.2.0.

**Disposition at v0.1.0 / M0:**

### 2a. Class A/B — code is canonical; spec docs need amendment (reconcile NOW)

| ID | Drift | Class | Disposition for M0 |
|----|-------|-------|--------------------|
| **SD1** | m9 ControlChar surface | A (code ahead of spec) | Code is canonical (Phase 6e m9 ↔ m32 EscapeSurfaceProfile trait seam landed; `NamespaceViolation::CapabilityNotAcknowledged` typed refusal). Spec doc `ai_specs/modules/cluster-D/m9_watcher_namespace_guard.md` § 4 reconciliation note: shipped surface trumps spec. **Resolved by code; spec amendment is a documentation-only patch deferred to a separate Phase 10 doc-pass commit.** |
| **SD2** | m14 LiftError taxonomy | B (name + variant divergence) | Code is canonical. Spec amendment to mirror live taxonomy is a documentation patch. **Same disposition as SD1.** |
| **SD3** | m14 cost_lift Result return | A | Code is canonical. **Same disposition.** |
| **SD4** | m14 window-eviction direction | A | Code is canonical. **Same disposition.** |
| **SD5** | m15 CharterSection variant names | B | Code is canonical (Phase 7 m15 surfaces `read_pressure_level()` + `pressure_scalar_from_count()`). Spec mirrors live. **Same disposition.** |
| **SD6** | m15 detected_at_ms field | A | Code is canonical. **Same disposition.** |
| **SD7** | m15 pseudo_rfc3339 wire-format | B | Code uses ms-since-epoch (the m13 `now_ms()` pattern); spec mentions pseudo_rfc3339. Code wins (live habitat already serdes ms). **Same disposition.** |
| **SD12** | m20 stabilization gate absent | B/load-bearing | m20 ships without an explicit stabilization gate — the m23 F2 evidence floor (`PROPOSAL_F2_THRESHOLD = 20`) is the load-bearing gate that the spec's `stabilization` refers to. Spec amendment to point at F2 as the stabilization mechanism. **Same disposition.** |

**Reconciliation principle for all 8 A/B items:** the shipped `src/` tree is canonical (verified by
the W2 KEYSTONE rewrite, Phase 2 wire-contract audit, Phase 6 named-verifier set, Phase 7 CC-7 wire);
spec-doc amendments are documentation-only patches that do not change behaviour and can land as a
single Phase 10 doc-pass commit on M0 ship. **No behavioural risk from M0 perspective.**

### 2b. Class C — algorithmic / shape, deferred to v0.2.0

| ID | Drift | Class | v0.2.0 fold-in |
|----|-------|-------|----------------|
| **SD8** | m21 Levenshtein vs swap/skip | C (KEYSTONE algorithmic divergence) | Phase 5 D17 docstring honestly labels m22's Levenshtein-from-identity as a **closed-form proxy** for the edit-distance baseline. SD8 asks for a true Levenshtein over the source pattern's steps — that requires a step-lookup (proposal carries `source_pattern_hash`, not the steps). v0.2.0 candidate per Plan v2 §11. |
| **SD9** | m22 generic kmeans vs spec FeatureVector | C | The shipped `kmeans(&[Vec<f64>], &KMeansConfig)` is a generic-shape K-means; the spec calls for a named `FeatureVector` type wrapping the same data. Cosmetic typed-newtype wrap is v0.2.0 doc/quality work. |
| **SD10** | m22 empty-cluster retain-prior | C/A hybrid | The shipped F-m22-01 fix retains the prior centroid on empty cluster (the Lloyd's canonical recovery action — `m22_kmeans/mod.rs:188–204`). The spec called for either typed error or re-seed. **CODE wins on shape; spec should document the retain-prior choice — pending v0.2.0 doc amendment.** |
| **SD11** | m23 5-field vs spec 12-field proposal | C | Shipped `WorkflowProposal` has 6 fields {proposal_id, variant, evidence_n, evidence_lift, evidence_ci_half, diversity_cluster}; spec proposes 12. The extra 6 spec-fields are: per-step cost, per-mutation cost, lineage_chain, generation_index, parent_proposal_id, lift_p95. These are v0.2.0 features tied to NA-GAP-01 (RefusalToken) + Cost-wire (per D9, currently stub). v0.2.0 candidate. |

**Class C reconciliation principle:** these are algorithmic and shape drifts where the M0 simplification
is honest and documented; the full spec is the v0.2.0 target. No M0 behavioural risk.

---

## 3. Net Phase 9 disposition

- All 12 SD items have a disposition.
- 8 Class A/B → reconciled now (code is canonical; spec amendments deferred to a Phase 10 doc-pass
  commit which carries no behaviour change).
- 4 Class C → v0.2.0 (algorithmic / shape; documented honestly in the docstrings of the M0 code).
- ZEN-W absent-vs-slow → substituted via the in-session zen agent per D26. Verdict packet folds in
  here when the dispatched agent returns; if `APPROVE` / `APPROVE-WITH-NITS` → proceed to Phase 10
  M0 ship; `AMEND` → sub-task before tag; `BLOCK` → escalate to node 0.A.

**Phase 9 is not blocked on the zen verdict** — the SD reconciliation above stands. The verdict
modifies *which* honest residual list ships in `CHANGELOG.md`'s v0.1.0 section.

---

## 4. Honest residuals at v0.1.0 / M0 (consolidated)

Per the substrate-frame discipline of Plan v2 §8 — what M0 does **not** certify is documented here so
the v0.1.0 tag's scope is named, not implied.

1. **NA-GAP-01 `RefusalToken`** — deferred to v0.2.0 (Phase 2 audit recommended); ADR
   `D-S1002127-03` amendment required.
2. **NA-GAP-04 substrate back-pressure budget** — deferred to v0.2.0 (Phase 2 audit recommended).
3. **NA-GAP-07 substrate-drift canary `m16`** — deferred (`D-S1002127-03` original).
4. **NA-GAP-08 substrate test fixtures** — deferred (`D-S1002127-03` original).
5. **NA-GAP-10 substrate-mediated trust** — deferred (`D-S1002127-03` original).
6. **m31 production caller `|_w| 0.0`** (Phase 5 nit A2) — m22 diversity signal threaded
   substrate-side; consumer-side wire is v0.2.0.
7. **SD8 / SD9 / SD10 / SD11 Class-C drifts** — v0.2.0 algorithmic / shape upgrades.
8. **m33 Security M0 default workflow surface = Sandboxed** (Phase 6a) — gate SHAPE correct; per-workflow
   surface determination is v0.2.0 (alongside NA-GAP-01).
9. **m13 outbox drain side absent** (NA-GAP-06 partial) — pairs with NA-GAP-09 in Phase 6f's
   substrate-confirmable receipt; consumer/drain is v0.2.0.
10. **R1 m33 4-verifier set:** Security + Ember are real semantic gates; Cost + Consistency are
    documented stubs per D9 + D11.
11. **`wf-dispatch --execute` against a live Conductor** — verification is post-M0 dispatch soak per
    D34/D35/D36 (Watcher ☤ carries the 24h NoOp soak; Conductor bring-up = OP-1).
12. **CI machinery ships with M0 (D29)** but the `spacetimedb-sdk = { path = "../spacetimedb/sdks/rust" }`
    relative path-dep makes CI flow against the workflow-trace repo non-trivial. Phase 10 ships the
    workflow files; full CI-green requires resolving the sibling-repo dependence (vendor or
    submodule) — documented as the one known limitation of D29's "CI ships with M0".

---

## 5. Phase 9 commit shape

ONE Phase 9 commit per D44, bundling:
- This doc.
- Zen verdict-packet fold-in (when the agent returns).
- The SD1–SD12 disposition narrative.

`docs(workflow-trace): Phase 9 — Zen audit fold-in + SD1–SD12 reconciliation`

— Phase 9 audit fold-in + SD reconciliation, S1004115.
