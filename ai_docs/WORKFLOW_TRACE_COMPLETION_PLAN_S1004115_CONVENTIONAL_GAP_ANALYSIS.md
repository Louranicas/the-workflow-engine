# Conventional (Engineering / PM) Gap Analysis — Workflow-Trace Completion Plan S1004115

> **Subject:** `the-workflow-engine/ai_docs/WORKFLOW_TRACE_COMPLETION_PLAN_S1004115.md`
> **Analyst pass:** skeptical engineering review · 2026-05-23
> **Method:** read plan in full; spot-checked against live tree at HEAD `6c3a5c5`;
> cross-checked GATE_STATE.md, CLAUDE.local.md S1003733 block, HARDENING_FLEET_2026-05-21.md,
> HARDENING_FLEET_CARRY_FORWARD_S1002600.md, CHANGELOG.md, Cargo.toml, the cross-talk dir.
> **Verdict in brief:** the plan is structurally sound and unusually honest about residuals,
> but it ships on a stale HEAD premise, mis-scopes at least one interview question that is
> already-done work, contains an unverified factual claim that would mis-size Phase 4, omits
> several real outstanding items (CI, tags, repo hygiene, the dirty working tree, the
> CHANGELOG mutation-number contradiction), and its Phase 7 "ship M0 anyway" rule rests on
> Zen verdicts that **do not exist** rather than ones that are merely slow.

---

## C-1 — Plan is authored against a stale HEAD; `2096fd0` is not HEAD. **Severity: MED**

**Gap.** The plan's scope line (§ Scope, line 6) and § 2 both fix the world "as of HEAD
`2096fd0`". The live tree is one commit further on:

- `git rev-parse HEAD` → `6c3a5c57…` = commit `6c3a5c5` *"docs(workflow-trace): W5
  closeout — reconcile docs to post-S1003733 truth"*, authored 2026-05-23 — i.e. **the same
  day the plan was authored**, and the plan did not pick it up.
- `6c3a5c5` is materially relevant: its commit message says it *"fill[s] the `(W5 commit)`
  placeholder … correct W4 test count 1924 → 1903 … Adds an S1004115 reconciliation note"*.
  So a *prior* reconciliation pass for S1004115 already touched HARDENING_FLEET_2026-05-21.md
  and the two CLAUDE files.

**Why it matters.** Phase 1 step 3 (DOC-2 supersede the carry-forward doc) and Phase 8 step 2
("reconcile CLAUDE.md / CLAUDE.local.md / GATE_STATE.md / ARCHITECTURE.md") are partly
*already done* by `6c3a5c5`. The plan will re-walk reconciliation work that landed hours
earlier, and its own § 2 inventory is sourced from a tree-state that no longer exists.

**Evidence.** `git log --oneline -1` = `6c3a5c5`; plan line 6 = `2096fd0`.

**Recommendation.** Re-baseline the plan on `6c3a5c5`. Before Phase 1 runs, `git show 6c3a5c5
--stat` and subtract anything it already reconciled from Phase 1/Phase 8's doc list. Make
"latest HEAD" a variable the plan reads at execution time, not a frozen literal.

---

## C-2 — Q1 is mis-scoped: the `--ack-ceiling` plumbing it asks about already exists. **Severity: HIGH**

**Gap.** Phase 0 Q1 asks node 0.A to *decide* the R1 Security-verifier policy source and
frames the default as "`--ack-ceiling` only (already threaded to m32)". The plan treats the
*threading* as the open work. It is not open — it is **shipped**:

- `src/orchestration/dispatch.rs:116` — `--ack-ceiling <PROFILE>` is a real, documented CLI
  flag in the `wf-dispatch` help text.
- `dispatch.rs:55` — `pub ack_ceiling: EscapeSurfaceProfile` is a real field on `Config`.
- `dispatch.rs:124-130` — `parse_ack_ceiling()` maps the raw arg to `EscapeSurfaceProfile`.
- `src/m32_dispatcher/mod.rs:98,125,134` — `acknowledged_ceiling` is consumed by the m32
  monotone gate; `dispatch.rs:25-26` imports `HumanAcceptanceSignature`.

So the operator ceiling input *already reaches m32*. What is genuinely missing is only that
the **m33 `ConservativeVerifier::verify` body returns `Approve` unconditionally** — it never
*reads* `ack_ceiling`. That is a code task, not a decision. Q1's real content collapses to
"should the Security verifier compare the proposal's `EscapeSurfaceProfile` against the
already-plumbed `ack_ceiling`?" — which is nearly self-answering (yes, that is the whole
point of the field) and does not need a node-0.A interview slot.

**Why it matters.** A decision interview that spends a question on already-built plumbing
burns Luke's stated 3-decisions/week budget and risks the answer being "do the thing that is
already done", producing zero delta. It also signals the § 2 recon under-verified the code.

**Evidence.** `dispatch.rs:55,116,124-130`; `m32_dispatcher/mod.rs:98,125,134`;
`dispatch.rs:333` (`ConservativeVerifier::verify` → `VerifierVerdict::Approve`).

**Recommendation.** Re-write Q1 as a code task in Phase 4, not a Phase-0 decision: "Security
verifier compares proposal `EscapeSurfaceProfile.ordinal()` ≤ `Config.ack_ceiling.ordinal()`,
emitting Refuse above ceiling." Keep an interview slot only for genuinely-open policy (e.g.
whether Refuse-above-ceiling is hard or soft). Free the slot for a real decision (see C-7).

---

## C-3 — Phase 4 "Cost verifier" rests on an unverified `cost_estimate` field. **Severity: HIGH**

**Gap.** Phase 4 step 1 says the Cost verifier will "validate `cost_estimate` against the
budget model". Q2 likewise presupposes a per-run cost figure on the proposal artefact. A
spot-check shows **no `cost_estimate` field on the dispatch artefact**:

- `grep cost_estimate src/m30_bank/` → nothing.
- The only cost primitive is `m14_lift::cost_lift(baseline, actual) -> Result<f64>`
  (`src/m14_lift/mod.rs:252`) — a *lift ratio* computed in the crystallise pipeline, not a
  cost figure carried on an `AcceptedWorkflow` into `wf-dispatch`.
- `wf-dispatch` consumes a JSONL `WorkflowProposal` bridge; whether that struct carries any
  cost field at all is unverified by the plan.

So Q2's three options ("static ceiling / per-run estimate vs constant / no real gate") and
Phase 4's Cost verifier are sized against a data field that may not exist on the dispatch
side. If it does not, the Cost verifier is not "~80-120 LOC of policy" — it is a
**cross-binary schema change** (add the field to `WorkflowProposal`, populate it in
`crystallise`, serialise it through the JSONL bridge, deserialise in `dispatch`), which is a
materially larger and riskier change than the plan budgets.

**Why it matters.** Phase 4 is the critical path and the largest phase (~2-4 days). A hidden
schema change inside it blows the estimate and the sequencing — and a JSONL wire-contract
change is exactly the bug class the habitat has been bitten by (S112 port drift).

**Evidence.** `m14_lift/mod.rs:252` (only cost primitive); absence of `cost_estimate` in
`src/m30_bank/`; plan Phase 4 step 1 + Q2.

**Recommendation.** Add a Phase-1 verification line: confirm what cost data (if any) the
`WorkflowProposal` JSONL bridge carries. If none, Q2 must explicitly surface "Cost verifier
requires a new wire field" as its own decision with its own LOC/risk, or honestly default to
"Cost stays an Approve-stub for M0" — and the plan must say *why* (no data plumbed), not just
"keep the stub".

---

## C-4 — Phase 7 "ship M0 anyway" rests on Zen verdicts that do not exist, not slow ones. **Severity: HIGH**

**Gap.** § 2.4 says Zen audit packets were filed for W1/W2/W3/W4 + S1003733 + C22 and "**No
verdict files observed.**" Phase 7 + the risk register treat this as a *latency* problem
("if verdicts are slow … fold AMEND as v0.1.1"). A cross-talk scan shows the situation is
worse than slow:

- Review **requests** exist: `2026-05-21T111329Z_command_zen_review_request_hardening_w1.md`,
  `…114931Z…w2.md`, `…121000Z…w3.md`, `2026-05-21T163238Z…w4.md`.
- There is **no `zen_*verdict*` file for any workflow-trace hardening wave**. The only Zen
  *verdicts* in the dir are for LCM sub-wave 1A and the Restraint 8th-trait — different
  projects. `2026-05-22135545Z_command_zen_workflow_engine_c22_docs_complete.md` is a
  Command-authored *notice*, not a Zen verdict.
- Worse: the carry-forward doc records the **V4 spec-drift audit was filed 2026-05-20T08:00Z
  and "No Zen reply yet"** three days later. That is not a slow verdict; that is an audit
  channel that may be dead or de-prioritised.

**Why it matters.** "Zen-paced, ship M0 anyway, fold AMEND as v0.1.1" is only safe if a
verdict is *coming*. If Zen has not engaged with workflow-trace hardening at all, M0 ships
with **zero independent audit** of W1-W4 + S1003733 + C22 — the entire hardening campaign —
and the v0.1.1 escape hatch becomes a euphemism for "we never got audited". The whole
project's quality story (per the Zen 87/100 assessment) leans on Zen as the audit gate.

**Evidence.** `~/projects/shared-context/agent-cross-talk/` — 4 `command_zen_review_request_
hardening_w[1-4]` files, 0 corresponding `zen_*verdict` files; carry-forward §"Cross-pane Zen
verdicts" V4 = "PENDING … No Zen reply yet".

**Recommendation.** Phase 7 must distinguish *verdict slow* from *verdict absent*. Add an
explicit pre-M0 gate: if no Zen verdict has landed for W1-W4/S1003733/C22 by Phase 6 close,
**escalate to node 0.A as a decision** — "ship M0 un-audited, or hold for audit, or
substitute an independent reviewer". Do not let "v0.1.1 point release" silently absorb the
absence of any audit at all.

---

## C-5 — Missing work: CI, version-tag hygiene, repo hygiene, and the dirty working tree. **Severity: MED**

**Gap.** Several real outstanding engineering items appear nowhere in the plan:

1. **No CI exists.** `ls .github/workflows .gitlab-ci.yml` → nothing. The project pushes to
   GitHub + GitLab and runs a 4-stage gate manually every phase. There is no automated gate
   on push, no protection against a regression landing un-gated. A v0.1.0/M0 milestone with
   no CI is an incomplete engineering deliverable; the plan should at minimum decide
   (Phase 0 or Phase 8) whether M0 ships with a CI pipeline.
2. **Version drift.** `Cargo.toml` already says `version = "0.1.0"`, but `CHANGELOG.md`'s
   top entry is `[v0.1.0-s1003733]` under an `## [Unreleased]` heading, and **`git tag -l`
   is empty** — no `v0.1.0` tag yet. Phase 8 step 4 tags `v0.1.0`, but the plan never
   notices Cargo.toml *already* claims 0.1.0 (so the project has been at "0.1.0" through all
   of S1003733), nor that the CHANGELOG version label (`v0.1.0-s1003733`) will collide /
   confuse with the Phase-8 `v0.1.0` tag. This needs a deliberate version-string decision.
3. **`mutants.out` / `mutants.out.old` dirs are in the working tree.** They are
   `.gitignore`d (verified: `git check-ignore` exits 0) so not a commit risk, but
   `mutants.out.old` is 2.8 MB of stale artefact and the plan's repeated `cargo-mutants`
   re-runs (Phases 3,4,6,8) will keep regenerating/rotating them. The plan should say to
   clean them, and confirm `.gitignore` coverage as part of Phase 8 "clean tree".
4. **Dirty working tree, untracked + modified.** `git status` shows `M
   .obsidian/workspace.json`, `M pre-framework-consolidation/WATCHER_DEPLOYMENT_WATCH_
   JOURNAL_S1001982.md`, `M the-workflow-engine-vault/Watcher Deployment Watch Journal
   S1001982.md`, and `D "Pasted image 20260517154304.png"`. The CLAUDE.local.md S1003733
   snapshot itself flags `src/m30_bank/mod.rs` as dirty (its "Known drift" §). Phase 8 says
   "clean tree" as an outcome but the plan never lists *resolving the existing dirt* as a
   task. A "clean v0.1.0" cannot be tagged on top of uncommitted journal/workspace edits
   without first deciding commit-or-discard for each.

**Why it matters.** These are exactly the "version/tag hygiene, CI" items the review brief
asked to check, and a milestone plan that omits them produces a v0.1.0 that is not actually a
clean release point.

**Evidence.** No `.github/workflows` or `.gitlab-ci.yml`; `Cargo.toml:` `version="0.1.0"`;
`git tag -l` empty; CHANGELOG top entry `[v0.1.0-s1003733]` under `[Unreleased]`;
`git status --short`; CLAUDE.local.md §"Known drift / dirty state".

**Recommendation.** Add a Phase-1 (or new Phase 1.5) task: resolve the dirty tree
(commit-or-discard each modified/deleted file), clean `mutants.out*`, and verify `.gitignore`.
Add a Phase-0 or Phase-8 decision on (a) whether M0 ships a CI pipeline and (b) the canonical
version string (reconcile Cargo.toml `0.1.0`, CHANGELOG `v0.1.0-s1003733`, and the planned
`v0.1.0` git tag into one coherent story).

---

## C-6 — CHANGELOG carries a mutation number that contradicts the plan's headline. **Severity: MED**

**Gap.** The plan's § 1 headline says "96.3 % mutation kill-rate". HARDENING_FLEET_2026-05-21
W4 row agrees: "324 mutants — 259 caught / 10 missed … 96.3 %". But `CHANGELOG.md:31`
(written by `6c3a5c5`/`ce0d77b`) says: *"324 mutants, 254 caught, **94.4 %**"*. Same
mutant-total (324), **different caught count (254 vs 259) and different kill-rate (94.4 % vs
96.3 %)**. One of these is wrong, and they are in two canonical surfaces the plan itself
intends to leave as M0 truth.

This is precisely the doc-drift class S1003733 was supposed to *close* — the W4 history is
littered with corrected numbers (412/80.6 % → 1924 → 1903 → 96.3 %). The plan's Phase 8 step
2 "reconcile CLAUDE.md/CLAUDE.local.md/GATE_STATE.md/ARCHITECTURE.md" does **not** list
CHANGELOG.md, so this contradiction would survive into M0.

**Why it matters.** Shipping a milestone with two different official kill-rates undermines
the project's central quality claim and repeats the exact failure (mutation-number drift)
that the hardening fleet kept having to correct.

**Evidence.** `CHANGELOG.md:31` ("254 caught, 94.4 %") vs `HARDENING_FLEET_2026-05-21.md:65`
+ `:90` ("259 caught … 96.3 %") vs plan § 1 ("96.3 %").

**Recommendation.** Phase 1 (verify & reconcile) must add CHANGELOG.md to the reconciliation
set, resolve the 254-vs-259 discrepancy against the actual `mutants.out` artefact
(`caught.txt`), and make every surface cite one number. Phase 8's reconcile list must include
CHANGELOG.md explicitly.

---

## C-7 — Effort estimate for R1 (Phase 4) is optimistic; the 4 verifiers are not uniform. **Severity: MED**

**Gap.** The plan estimates R1 at "~310-490 LOC, ~2-4 days" and Phase 0 itself notes
Consistency is "the heaviest … ~80-120 LOC + bank access". But the estimate still treats the
four verifiers as a roughly uniform batch. They are not:

- **Security** — small *if* it just compares two ordinals (C-2). Real.
- **Ember** — Q3 itself admits scope is undecided ("7-trait rubric vs reduced subset"). A
  full 7-trait rubric applied to a proposal artefact is not 80 LOC; the m10 Ember CI gate is
  a whole module. "Reduced subset (clarity + safe-naming)" is cheap but is itself a
  judgement about what an Ember verdict on a *workflow proposal* even means — under-specified.
- **Cost** — possibly a cross-binary schema change (C-3), i.e. *unbounded* until Phase 1
  verifies the wire contract.
- **Consistency** — "bank-conflict detection" needs read access to the curated bank from
  inside `wf-dispatch`; the carry-forward doc T4-API flags that `ConductorDispatcher`
  *"exposes no `client_ref()` accessor"* and tests already work around missing seams. So
  Consistency may also need a new accessor / API-seam change.

Each verifier also carries the plan's own "50+/verifier" test requirement = **200+ new
tests** plus per-kind `cargo-mutants` re-runs. `cargo-mutants` debug-rebuilds per mutant; the
carry-forward doc explicitly notes modules were *skipped* in earlier runs purely on time
budget. A scoped m33 mutation run after adding 4 verifiers is plausibly a multi-hour job by
itself.

**Why it matters.** Phase 4 is the critical path. If two of its four verifiers can each
trigger a schema/API-seam change, "2-4 days" is a floor, not a range. The roll-up's "~7-12
days" upper bound is likely the realistic *median* once C-3/C-7 unknowns resolve adversely.

**Evidence.** plan Phase 4 + Q1-Q4; `dispatch.rs:333`; carry-forward §T4-API ("no
`client_ref()` accessor"); carry-forward §Wave-D ("m21/m22/m23/m10/m13 SKIPPED — time
budget").

**Recommendation.** Split Phase 4 into 4a (Security — small, ordinal compare), 4b (Ember —
reduced subset only for M0), 4c (Cost — gated on the C-3 wire-contract verification; default
documented-stub), 4d (Consistency — gated on whether a bank accessor seam is needed). Give
each its own estimate. Revise the roll-up upper bound upward, or explicitly say the upper
bound assumes Q2+Q4 both choose "defer".

---

## C-8 — False parallelism: Phases 1-2 are not as decision-free as claimed. **Severity: MED**

**Gap.** § 3 and § 5 assert Phases 1 and 2 are "decision-free" and can "start the moment
this plan is approved, before the interview even runs". Two cracks:

1. **m9-TODO (Phase 2 step 1) is not decision-free.** The plan calls it "~50 LOC, no
   decision needed". The actual TODO in `src/m9_watcher_namespace_guard/validator.rs:169-177`
   says wiring the `EscapeSurfaceProfile` 7-variant table requires reading the
   `HumanAcceptanceSignature` struct *"via a trait abstraction once that module ships"* —
   m9 reaching into m32's type. That is a cross-module coupling decision (define the trait,
   decide its surface), and it is *the same EscapeSurfaceProfile machinery* that R1's
   Security verifier (Phase 4) touches. Doing m9-TODO in Phase 2 *before* Q1/Q4 are
   answered risks designing the m9↔m32 seam one way, then re-doing it when Phase 4 settles
   the verifier's relationship to that profile. There is a hidden dependency Phase 2→Phase 4.
2. **T4-DEAD-ERR (Phase 2 step 3) explicitly needs a decision.** Carry-forward §T4-DEAD-ERR:
   *"Decision: keep + test, or drop with `#[deprecated]`."* The plan folds it into the
   "no decision needed" Phase 2. ~15 dead error variants across 9 modules being kept vs
   deprecated is a public-API decision (deprecation is a SemVer-visible change). It is small,
   but it is not *no decision* — it is an un-interviewed default.

**Why it matters.** "Phases 1-2 run before the interview" is the plan's headline scheduling
win. If m9-TODO actually couples to the R1 verifier design, the win is partly illusory and
Phase 2 should wait for Q1/Q4 — or m9-TODO should move into Phase 4.

**Evidence.** `src/m9_watcher_namespace_guard/validator.rs:169-177` (m9 must read m32's
`HumanAcceptanceSignature` via a not-yet-defined trait); carry-forward §T4-DEAD-ERR
("Decision: keep + test, or drop").

**Recommendation.** Either (a) move m9-TODO into Phase 4 so the EscapeSurfaceProfile seam is
designed once, or (b) explicitly state in Phase 2 that m9-TODO only wires the *table* and
does not touch the m9↔m32 trait, deferring that to Phase 4 — and verify that split is
actually possible. Add T4-DEAD-ERR's keep-vs-deprecate to the Phase-0 interview as a Q8 (or
explicitly flag it as an accepted default with SemVer implications noted).

---

## C-9 — m2-DOC task points at auto-generated SpacetimeDB code that must not be hand-edited. **Severity: MED**

**Gap.** § 2.2 lists **m2-DOC**: *"`src/m2_stcortex_consumer/module_bindings/mod.rs:1059` —
doc-debt TODO on the subscription API … ~1 h"*. That file is **machine-generated**. Its first
lines: *"THIS FILE IS AUTOMATICALLY GENERATED BY SPACETIMEDB. EDITS TO THIS FILE WILL NOT BE
SAVED."* (verified, `mod.rs:1-2`), and `#![allow(unused, clippy::all)]` at line 5. The
`TODO: Document this better` at line 1059 is upstream SpacetimeDB SDK boilerplate, not
workflow-trace's debt.

**Why it matters.** Scheduling a 1-hour "fix the doc TODO" task against a regenerated file
is wasted work — any edit is destroyed on the next `spacetimedb generate`. Worse, if an agent
edits it and the file is regenerated later, the edit silently vanishes (the file even *says*
so). It also indicates the § 2 code-scan recon flagged a generated-code TODO without checking
provenance — which lowers confidence in the rest of the "code scan" inventory.

**Evidence.** `src/m2_stcortex_consumer/module_bindings/mod.rs:1-5` (generation banner +
`allow(unused, clippy::all)`); generated by "spacetimedb cli version 2.1.0".

**Recommendation.** Drop m2-DOC from the plan entirely, or re-scope it as "confirm
`module_bindings/` is vendored generated code; add a `// generated — do not edit` note to its
parent `mod.rs` if not already present; no edit to the generated file." Re-audit the rest of
the § 2.2 code-scan list for other generated-code false positives.

---

## C-10 — Phase 6 "exercise binaries on live atuin data" has an undeclared environment dependency. **Severity: LOW**

**Gap.** Phase 6 step 1 says exercise `wf-crystallise` on "real report path on live atuin
data" and `wf-dispatch` against a `proposals.jsonl`. This is the only phase that needs a live
substrate, yet the plan never lists what must be up: atuin DB readable, stcortex `:3000`
reachable (m2 consumer), and — for any real `wf-dispatch --execute` path — HABITAT-CONDUCTOR
on `:8141`, which OP-1 explicitly notes is **not yet started** (`auto_start=false`). The plan
correctly defers OP-1 to Luke, but then schedules Phase 6 as if the binaries can be
end-to-end exercised without it. `--dry-run`/`--offline` cover most of it, but "exercise
beyond compile/test" against a down Conductor only ever tests the degradation path, never the
real dispatch path — so Phase 6 cannot actually validate `wf-dispatch --execute`.

**Why it matters.** Phase 6 is billed as the integration/end-to-end gate before M0. If it
structurally cannot exercise the real dispatch path (Conductor down), M0 ships with the
`--execute` path never having run once. That should be a stated, accepted limitation, not a
silent one.

**Evidence.** Plan Phase 6 step 1 vs OP-1 (Conductor `auto_start=false`, Luke action,
"non-blocking until a live dispatch-plane soak is wanted"); GATE_STATE B3.

**Recommendation.** Phase 6 should explicitly state its environment matrix and split into
"offline/dry-run exercise (always runs)" vs "live `--execute` smoke (requires OP-1; deferred
to post-M0 dispatch soak)". M0's definition of done should record that `--execute` is
unverified-against-live-Conductor at ship.

---

## C-11 — The v0.2.0 partition (§ 9) is defensible — but the boundary leaks. **Severity: LOW**

**Gap (in favour of the plan, with one caveat).** Partitioning NA-GAP-07/08/10 out of M0 is
*not* scope-dodging: they are covered by a real ADR (`D-S1002127-03`), they are a coherent
~1,200-LOC milestone, and § 9 honestly says un-deferring is a node-0.A decision. That is
sound PM. **However**, § 8 (the NA frame-check) then *adds* two requirements into M0 —
"substrate-confirmable verdict receipts" in Phase 4 and "cluster emission" in Phase 3 —
explicitly because R1/R2 "re-open NA-GAP-10" / touch substrate visibility. So the plan
*does* pull a slice of the deferred substrate-trust work back into M0, under the § 8 banner,
without sizing it. "every verifier verdict must emit a substrate-confirmable receipt … become
observable `WireEvent`s" is real additional code in Phase 4 (new event emission, new wire
shape, tests) and is not in the Phase 4 LOC estimate or the effort roll-up.

**Why it matters.** The partition is clean on paper but § 8 quietly re-imports a fragment of
NA-GAP-10 into the critical-path phase without an estimate — the exact "scope creep via a
framing section" pattern the risk register claims to guard against.

**Evidence.** § 9 (defensible partition) vs § 8 point 1 ("Mitigation folded into Phase 4:
every verifier verdict must emit a substrate-confirmable receipt … observable `WireEvent`s")
+ § 8 point 2 (Phase 3 cluster emission) — neither reflected in § 6 effort roll-up.

**Recommendation.** Either cost the § 8 additions explicitly inside the Phase 3/Phase 4
estimates (and raise the roll-up), or move "substrate-confirmable verdict receipts" wholly
into v0.2.0 with the rest of NA-GAP-10 and keep M0's verifiers honestly engine-internal.
Pick one; do not have § 9 defer NA-GAP-10 while § 8 silently does a piece of it.

---

## C-12 — Phase 0 is missing two questions; Q7 is correctly a default. **Severity: LOW**

**Gap.** Beyond the Q1/Q2 mis-scoping (C-2, C-3), the interview is missing decisions that
*are* genuine node-0.A calls:

- **CI** — does M0 ship a CI pipeline? (C-5) — a real, currently-unmade decision.
- **Version string reconciliation** — Cargo.toml `0.1.0` vs CHANGELOG `v0.1.0-s1003733` vs
  the planned `v0.1.0` tag (C-5) — a real call.
- **T4-DEAD-ERR keep-vs-deprecate** — SemVer-visible (C-8).

Conversely, **Q7 (naming) is correctly scoped as a default** — it is genuinely cosmetic and
"keep `workflow-trace`" as a flagged default is the right call; it arguably should not
consume an interview slot at all and could just be stated as a decision already made.

**Why it matters.** The interview's job is to surface the *real* decisions. It currently
spends slots on a done task (Q1) and a cosmetic (Q7) while omitting CI and version hygiene.

**Recommendation.** Drop Q1 to a code task (C-2), demote Q7 to a stated default, and add Q8
(CI for M0?) and Q9 (canonical version string). Net interview size stays ~7 questions but
they are all real.

---

## Strengths (noted after the gaps, per brief)

1. **Honest residual framing.** The plan does not pretend the codebase is finished — R1/R2
   are correctly identified as documented placeholders, and the `ConservativeVerifier`
   doc-comment in `dispatch.rs:311-335` genuinely matches the plan's description (verified).
   That honesty is rare and correct.
2. **Phase 1 FP-verify discipline.** Treating the carry-forward doc as "evidence, not fact"
   and FP-verifying every ⚠ item before scheduling is exactly right — the carry-forward doc
   *is* stale (it pre-dates the Hardening Fleet) and the plan correctly flags it.
3. **No phase collapse.** Each phase gets its own gate + commit, `${PIPESTATUS[0]}`-aware,
   matching `feedback_no_shortcuts` and `feedback_pipestatus_for_gate_chains`.
4. **The v0.2.0 partition is principled** (modulo the § 8 leak in C-11) — it cites a real
   ADR and makes un-deferring an explicit decision rather than silently dropping work.
5. **The integration-test commits it cites are real.** `f8ab952`, `1c9b809`, `c4bfed4`,
   `e2acf65`, `711a662` all exist with the described content (verified `git cat-file`).

---

## Severity roll-up

| ID | Gap | Severity |
|----|-----|----------|
| C-1 | Plan authored against stale HEAD `2096fd0`; real HEAD is `6c3a5c5` | MED |
| C-2 | Q1 mis-scoped — `--ack-ceiling` plumbing already shipped | **HIGH** |
| C-3 | Phase 4 Cost verifier rests on an unverified `cost_estimate` field | **HIGH** |
| C-4 | Phase 7 "ship anyway" assumes slow Zen verdicts; they are absent, not slow | **HIGH** |
| C-5 | No CI, version-tag drift, `mutants.out` cruft, dirty tree — all omitted | MED |
| C-6 | CHANGELOG says 94.4 % kill-rate vs plan's 96.3 % — contradiction survives to M0 | MED |
| C-7 | R1 effort optimistic — 4 verifiers non-uniform, 2 may need schema/API changes | MED |
| C-8 | False parallelism — m9-TODO couples to R1; T4-DEAD-ERR needs a decision | MED |
| C-9 | m2-DOC targets auto-generated SpacetimeDB code that must not be edited | MED |
| C-10 | Phase 6 cannot exercise `--execute` (Conductor down) — unstated limitation | LOW |
| C-11 | § 8 silently re-imports a slice of deferred NA-GAP-10 into M0 unsized | LOW |
| C-12 | Phase 0 missing CI + version decisions; Q7 over-scoped as a question | LOW |

**3 HIGH, 6 MED, 3 LOW.** The plan is executable but should not start until C-2, C-3, and
C-4 are resolved — each can mis-size or mis-direct the critical-path Phase 4, or let M0 ship
with no audit at all.
