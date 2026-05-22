# Non-Anthropocentric Gap Analysis — Workflow-Trace Completion Plan S1004115

> **Discipline:** second-pass NA gap analysis (`na-gap-analyst`).
> **Subject:** `the-workflow-engine/ai_docs/WORKFLOW_TRACE_COMPLETION_PLAN_S1004115.md` (HEAD `2096fd0`).
> **Mode:** RECURSION CHECK — the plan already contains its own NA pass at §8. This document
> does **not** re-derive §8; it interrogates §8 and the frame §8 itself collapses.
> **Filed:** 2026-05-23 · `na-gap-analyst` lane · advisory; does not gate M0.
> **Companion reads:** `ai_docs/decisions/2026-05-17-substrate-as-actor-deferrals.md` (D-S1002127-03) ·
> `ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md` (the 11-gap scaffold NA pass §8 inherits).

---

## 1. Input

- **Path:** `the-workflow-engine/ai_docs/WORKFLOW_TRACE_COMPLETION_PLAN_S1004115.md`
- **Length:** ~337 lines / ~3,100 words, 11 sections.
- **Dominant frame:** A (Conventional / tooling-oriented) — *with a Frame-B section bolted on at §8*.
- **Frame call:** This is the load-bearing finding of the recursion check. The plan is
  **not** `BALANCED_INPUT`. §§1–7 and §§9–11 are pure Frame A. §8 is a Frame-B *appendix*,
  not a Frame-B *authoring*. The tell: §8 closes with *"It does not change the phase count"*
  — a Frame-B pass that re-authored the plan would change the phase structure (the scaffold
  NA pass §3 re-authored the cluster table, the layer view, the artefact list, the gate, and
  the refusal architecture; §8 re-authors nothing). §8 is the conventional plan annotating
  itself, not the second pass.

### Evidence for the frame call (quoted)

- §1: *"What remains is **not** core architecture — it is a bounded set of honest
  residuals, decision-gated wiring, audit fold-ins, and operator actions."* — residuals as
  a task list; the unit of work is the engine artefact.
- §8 (its own framing): *"§§ 1–7 above are the **conventional frame** … This section is the
  **substrate frame**."* — the plan *names* §8 as the whole second pass. One section is
  asserted to discharge the dual-frame discipline.
- §8 closer: *"the NA pass adds two concrete Phase-3/4 requirements … It does not change the
  phase count."* — a second pass that produces only additive requirements inside the
  existing skeleton has been run *from the first frame's vocabulary*.
- §2.7 / §9: the three deepest substrate gaps (NA-GAP-07/08/10) are **partitioned out** by
  citing an ADR — the plan defers the substrate frame by reference rather than running it.

---

## 2. What §8 actually is (and is not)

§8 is a **conscientious Frame-A self-audit**. It does three real and useful things:
it folds two requirements into Phases 3–4 (substrate-confirmable verdict receipts; cluster
emission) and reframes Q6 as "drop the substrate's backpressure channel." Those are
genuine improvements and should ship. **This analysis does not retract them.**

But §8 is not the second pass the discipline asks for, for three structural reasons:

1. **It re-authors nothing.** The discipline ("write it again from the frame you didn't
   take") produces a *parallel plan*. §8 produces three bullet-point annotations on the
   existing plan. The scaffold NA pass (`NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md` §3) is what
   a real second pass looks like: it re-drew the cluster table, the layers, the gate, the
   artefact set. §8 has no §3-equivalent.
2. **Its "substrate frame" is the engine's model of the substrate, not the substrate's.**
   §8's three substrate concerns — *receipts*, *emission*, *backpressure channel* — are all
   things the **engine** does *to make itself legible*. None of them is a substrate acting
   on its own clock, budget, or refusal. This is the central frame collapse; see NA-1.
3. **It treats the §9 partition as settled.** §8 says the M0 NA pass "installs the seams"
   v0.2.0 builds on. But whether the M0/v0.2.0 cut is itself a frame choice is never
   interrogated — §8 inherits the partition as a given. See NA-7.

The remainder of this document is the interrogation of §8 — numbered NA-1…NA-9.

---

## 3. Second-frame gaps — interrogating §8

### NA-1 [SUBSTRATE] — §8's "substrate frame" is anthropocentric projection: receipts/voice/emission are the *engine's* categories of legibility, not the substrate's categories of agency

**The frame collapse.** §8's three substrate moves are: (1) "substrate-confirmable receipt"
for verdicts, (2) "cluster emission so selection rationale is substrate-visible", (3)
"`PressureEvent` is the substrate's backpressure channel / voice". Read closely, all three
are the engine *publishing more about itself*. A receipt is the engine narrating its own
verdict; an emission is the engine narrating its own cluster choice; a "backpressure
channel" the engine *listens on* is still the engine deciding what counts as pressure and
what shape it must arrive in. §8 has dressed "the engine should log more legibly" in
substrate vocabulary. This is precisely the **Resonance-Certification-Block pattern**
(`feedback_synthor_invocation_discipline.md`): substrate-flavoured prose that does not
change who holds agency.

The genuine substrate frame — the one the scaffold NA pass §3.1 actually ran — is that
atuin, stcortex, injection.db, the Conductor, and Luke are **actors with their own
lifecycle, attention budget, decay clock, and refusal capacity**. "Receipt" is not a
substrate category; it is an engine-accountability category. The substrate's categories
are: *can I afford this read right now; do I refuse this write; has my interpretation of
your last write drifted; am I being starved by your cadence.*

**Second-order failure it hides.** M0 ships believing it has "honoured the substrate
frame" because §8 exists and Phase 3/4 emit receipts. The actual substrate-as-actor gaps
(NA-2…NA-6 below) ship unaddressed *and now harder to see*, because the plan carries a
checked box labelled "NA pass done." A future analyst reads §8, sees "substrate frame
covered," and does not re-run it. The frame collapse is self-concealing.

**Recommendation.** Rename §8 honestly: it is an **"Engine-legibility pass,"** not a
substrate-frame pass. Either (a) downgrade §8 to "Frame-A self-audit — additive receipt
requirements" and add a genuine §8-bis that re-authors the *completion* from the substrate
frame (what does "complete" mean when atuin, stcortex, and Luke each get a vote?), or
(b) state explicitly in §8 that the true substrate frame is deferred wholesale to v0.2.0
and §8 only installs engine-side seams — which is honest, and makes NA-7's partition
question unavoidable. The worst outcome is leaving §8 as-is, labelled "substrate frame."
*Confidence: 0.85.*

### NA-2 [SUBSTRATE] — the substrates workflow-trace *reads* (atuin, stcortex, injection.db) are themselves agents with attention budgets; the plan models them only as sinks for the engine's reads

**The frame collapse.** §8 discusses substrates only as *recipients of the engine's
verdicts and emissions* (write-side). The read-side — Cluster A (m1/m2/m3) consuming
atuin / stcortex / injection.db — is entirely absent from §8. Yet Phase 6 explicitly
schedules *"`wf-crystallise` real report path on **live atuin data**"* and the v0.2.0
partition keeps the K-means feature extraction (Phase 3) running over substrate reads.
The conventional frame made the read-side invisible because, under Frame A, a database is
a source you query — it has no say. Scaffold gap **NA-GAP-S1002127-04** ("Engine assumes
substrates have no own attention budget") named exactly this; the completion plan never
re-checks it and §8 does not mention it.

**Second-order failure it hides.** Phase 6's "exercise binaries beyond compile/test … on
live atuin data" runs the engine's read path against the *real* atuin WAL-mode SQLite
**that the user's shell is concurrently writing to**. m1's paginated cursor reads hold
read locks; m4 + m6 read atuin in parallel. A Phase-6 exercise — or worse, any future
continuous-cadence mode — can starve the operator's own shell-history writes invisibly.
The engine has no substrate-back-pressure metric (scaffold NA-GAP-04, unclosed). M0 ships
with a "verified end-to-end" stamp earned by a run that may have degraded the substrate it
was reading. "Complete" silently means "complete from the engine's side of the read."

**Recommendation.** Add to Phase 6 an explicit substrate-side observation: while exercising
`wf-crystallise` on live atuin, measure atuin's *own* foreground write latency before /
during / after the run (e.g. time a trivial `atuin` shell insert). If the run perturbs the
substrate, that is a finding, not a pass. Carry the scaffold's `NA-GAP-04` ("substrate-side
load benchmarks") forward into the M0 risk register §7 rather than leaving it stranded in a
scaffold doc the completion plan supersedes. *Confidence: 0.8.*

### NA-3 [SUBSTRATE] — Luke @ node 0.A is modelled as an oracle (Phase 0 interview), never as a substrate with an attention budget and a refusal/fatigue mode

**The frame collapse.** Phase 0 (Q1–Q7) treats the operator as a *decision oracle*: ask
seven questions, receive seven answers (or take the flagged defaults), proceed. §11
likewise enumerates "Open decisions for node 0.A." The operator appears as a question sink.
This is the exact gap the scaffold NA pass raised as **NA-GAP-S1002127-05** ("Operator is
modelled as oracle, not as substrate") — and the completion plan reproduces it *one layer
up*: the scaffold modelled Luke as a `HumanAcceptanceSignature` field; the completion plan
models Luke as a 7-row interview table. The operator-substrate's own dynamics — attention
budget, frame-switching cost, consent fatigue, the cost of *seven* simultaneous
architectural questions in one round-set — are unmodelled.

**Second-order failure it hides.** §3 bundles Q1–Q7 into "one AskUserQuestion round-set"
and provides "default-if-no-answer" for every question. Under operator-fatigue dynamics,
the *predictable* outcome is that Luke accepts most defaults to clear the round — and the
defaults are explicitly the *cheapest* option for six of seven questions (Q2 keep stub,
Q4 defer Consistency, Q6 observability-only). The plan has built a consent surface whose
path of least resistance is the minimal M0. §8 even flags Q6 ("drop the substrate's
backpressure channel — not as a tidy-up") — but having flagged it, still leaves it as one
of seven questions with a cheap default in a single round. Flagging a question while
burying it in a fatigue-inducing batch is consent theatre. The substrate-frame failure:
the operator's *refusal capacity* (the ability to say "this question is malformed" or
"not now") has no representation; only Approve/Amend/Refuse-of-content does.

**Recommendation.** Split Phase 0. The three load-bearing questions that genuinely change
the engine's relationship to the substrate — **Q4** (Consistency verifier: real or
deferred), **Q6** (CC-7: keep or sever the substrate's only compositional voice), and the
*meta-question* of whether v0.2.0 NA-GAPs enter M0 (§11.3) — should be a *separate, small*
decision surface, presented before the four mechanical policy questions (Q1/Q2/Q3/Q5).
Do not give Q6 a cheap default; §8 itself says it must be "a *seen* cut" — a default makes
it an unseen one. Add an explicit operator-refusal path: "this question is not answerable
as posed" must be a first-class response, not silence routed to a default. *Confidence: 0.8.*

### NA-4 [SUBSTRATE] — HABITAT-CONDUCTOR is treated as a dispatch sink with a health endpoint, never as an actor that refuses, queues, or has its own enforcement clock

**The frame collapse.** The Conductor appears in the plan as **OP-1/B3**: *"Conductor
bring-up … then `curl :8141/health`. Non-blocking until a live dispatch-plane soak is
wanted."* That is the entire treatment — an operator action and a health probe. m32 (the
dispatcher) and m33 (the verifier) are scheduled in Phase 4 as engine-internal verdict
logic. But the Conductor is a **substrate-actor**: it has `CONDUCTOR_ENFORCEMENT_ENABLED`
as a substrate-side flag the engine does not own, a 24h NoOp soak before enforcement
flips, `auto_start=false` on Waves 1B/1C/2/3, and its own refusal surface. Scaffold NA pass
S-D ("Conductor substrate") modelled exactly this; §8 does not carry it. The conventional
frame made it invisible because Frame A sees a dispatch target as a place verdicts *go*.

**Second-order failure it hides.** Phase 4 builds four real m33 verifiers and routes
`ack_ceiling` into the Security verifier. The plan validates these against *engine-internal*
policy. But the engine's `Approve` verdict is not the dispatch — the Conductor's
enforcement state is. M0 can ship four perfectly-tested verifiers and a `wf-dispatch`
binary that, against a Conductor with `CONDUCTOR_ENFORCEMENT_ENABLED=0` (its documented
default), dispatches into a substrate that is *not enforcing the engine's verdicts at all*.
The engine's `DispatchError::ConductorDispatchDisabled` is a *translation* of a
substrate-authored refusal (scaffold NA-GAP-S1002127-02). M0 declares "dispatch verifiers
complete" while the substrate that consumes those verdicts may be in a state that ignores
them. "Complete" means "the engine emits verdicts," not "the substrate honours them."

**Recommendation.** Phase 4 (or Phase 6's `wf-dispatch` dry-run) must assert against the
Conductor's *enforcement state*, not just its `/health`. A verdict path that emits `Approve`
into a non-enforcing Conductor should itself produce an engine-side warning ("dispatched to
a Conductor not in enforcement mode — verdict is advisory"). Add to §7 the risk: "M0 m33
verifiers validated against engine policy only; Conductor enforcement-state coupling
unmodelled." Treat the Conductor's enforcement clock as a substrate-owned input to
`wf-dispatch`, not an operator setup step. *Confidence: 0.8.*

### NA-5 [SUBSTRATE] — time, decay, and recency are engine-owned clocks; the plan never asks whose clock `frequency × fitness × recency` runs on

**The frame collapse.** The project's named structural-gap authorship #2 is the
*`frequency × fitness × recency` compound decay* in m11. The completion plan touches m11
only glancingly (MUT-2 mutant, `chrono_now_ms` unused in T4-LIB) and §8 never raises it.
But "recency" is a *clock question*, and the plan silently assumes the engine owns the
clock. The substrates have their own clocks: atuin's WAL checkpoint cadence, stcortex's
pathway-weight decay schedule, injection.db's *hourly TTL sweep* (which — per the workspace
feedback memory `feedback_ttl_sweep_test_timestamps` — deletes low-timestamp rows), the
Conductor's 24h soak, RALPH's generation tick. When m11 computes "recency," it computes it
on the engine's wall clock — but the *pattern it is decaying was observed through a
substrate that is itself decaying that pattern on a different clock*.

**Second-order failure it hides.** Scaffold NA-GAP-S1002127-03 already named the specific
mechanism: the CC-5 loop runs m42→stcortex→habitat-inject→injection.db→m3, and "injection.db
ttl-sweep [can delete] reinforced rows before next-session reads." m11's recency decay and
injection.db's TTL sweep are **two decay functions on the same data with no shared clock**.
A pattern can be simultaneously "recent" to m11 (engine clock) and "swept" by injection.db
(substrate clock). M0 ships a compound-decay primitive that is correct in isolation and
*incoherent in the loop* — and Phase 6's "end-to-end exercise" will not catch it because
Phase 6 runs in a single session, shorter than any substrate's decay horizon. "Complete"
means "the decay formula compiles and unit-tests pass," not "decay is coherent across the
substrate clocks the data actually lives on."

**Recommendation.** Add a Phase-6 (or §7 risk) item: enumerate every clock the CC-5 loop
data crosses (m11 recency, injection.db TTL, stcortex pathway decay, atuin checkpoint) and
state, per crossing, whether the engine assumes monotonic agreement. Where it does, that is
a documented coupling risk for M0 and a v0.2.0 work-item (it is the substrate-clock half of
NA-GAP-07's drift detection). At minimum m11's spec must state "recency is engine-clock;
substrate-side decay of the same pattern is not modelled." *Confidence: 0.75.*

### NA-6 [SUBSTRATE] — the substrates can *refuse the engine's reads*, and Phase 1's FP-verification is itself a substrate read that can be refused or drifted

**The frame collapse.** Phase 1 ("Verify & reconcile") is framed as *pure prep, no
decision, no gate — run immediately*. It FP-verifies carry-forward items by "grep/read for
… the integration files." The plan treats verification as a deterministic read of a static
tree. But Phase 1 also writes **DOC-1: "write the missing S1003733 stcortex memory"** — a
*substrate write*. And stcortex enforces refuse-write at the DB layer for unregistered
consumers (workspace `CLAUDE.md` memory row 8), and refuses hyphen-bearing slugs at the
reducer (S1001757, restated in this very analyst's own protocol). §8 does not mention that
the plan's own Phase 1 touches a substrate that can refuse it. Scaffold NA-GAP-S1002127-02
(substrate-authored vs engine-authored refusal conflated) applies *to the plan's own
execution*, not just to the engine it builds.

**Second-order failure it hides.** Phase 1 is labelled "no gate, decision-free, start
immediately." If the DOC-1 stcortex write silently fails — refuse-write because the
executing session did not register a consumer, or a hyphen in the namespace slug
`workflow_trace_hardening_2026_05_21` (underscores here, but the *pathway* `pre_id`/`post_id`
in §10 are the hazard) — Phase 1 reports "complete" with one of its four sub-tasks
write-only-failed. The four-surface persistence the plan promises (§10) silently becomes
three-surface. This is the **POVM write-only trap** reproduced in the completion plan's own
persistence step. The plan that exists to verify substrate observation does not verify its
own substrate write.

**Recommendation.** Phase 1 step 4 (DOC-1) and Phase 8 step 3 (four-surface persist) must
each *read back* the stcortex write they perform (`stcortex sql "SELECT id FROM memory
WHERE namespace=…"`) and treat absence as a phase failure, not a silent pass. Add to §7:
"persistence steps are substrate writes subject to refuse-write / slug-reducer refusal;
verify-read-back required." This is the convergent finding with the conventional frame —
see C-1. *Confidence: 0.85.*

### NA-7 [SUBSTRATE] — the M0 / v0.2.0 partition (§2.7, §9) is itself a frame choice the plan never interrogates: "complete" is defined as "engine-internal items closed"

**The frame collapse.** §9 hard-partitions NA-GAP-07/08/10 out of M0 by citing ADR
D-S1002127-03, and §1 states M0's purpose is to reach *"zero undocumented debt."* But look
at *what* the partition keeps and cuts. M0 keeps: verifier policy logic, K-means wiring,
dead-code cleanup, doc reconciliation — every item is **engine-internal**. v0.2.0 gets:
substrate-drift detection (m16), substrate-side test fixtures, substrate-mediated trust —
every item is **substrate-as-actor**. The partition is not neutral. It is the
anthropocentric frame *drawing the milestone boundary*: "complete v0.1.0" has been silently
defined as "everything the engine owns is closed." The substrate-frame items were not
weighed and deprioritised — they were placed in a different milestone *by virtue of being
substrate-frame items*. §8 endorses this ("the M0 NA pass already installs the seams") and
thereby ratifies the collapse instead of catching it.

**Second-order failure it hides.** "v0.1.0 / M0 with zero undocumented debt" (§1) is the
plan's headline claim. But a milestone whose completion criterion is "engine-internal items
closed" can be 100% complete while the engine's *core purpose* — to safely observe and
propose back into a living substrate — remains structurally unvalidated (no drift canary,
no substrate fixtures, no substrate-side trust). M0 will be *declared* "complete" and
*tagged* `v0.1.0`. The tag is a strong social signal: it tells the habitat "this organ is
done to a milestone." The substrate-facing risk ships under a "complete" label. This is the
recurring habitat failure mode the discipline exists to catch — *"a plan looks complete
under one frame … and the unexamined frame quietly contains the failure that ships."* The
unexamined frame here is the **partition itself**.

**Recommendation.** §9 must add a paragraph that names the partition as a frame choice and
defends it *on the merits*, not by citing the ADR. The honest framing: "M0 is defined as
*engine-internal completeness*; it is explicitly **not** *substrate-relationship
completeness*. The `v0.1.0` tag therefore certifies the engine, not the engine's safety as
a substrate-facing organ." If that sentence is uncomfortable to write, that discomfort is
the finding. Additionally: the §11.3 question ("confirm NA-GAP-07/08/10 stay out of M0")
must be presented to node 0.A *with NA-7's framing* — not as "confirm the defer" (which
primes the cheap answer) but as "M0 will ship the substrate-facing organ without
substrate-drift detection; ratify or pull forward." *Confidence: 0.85.*

### NA-8 [SUBSTRATE] — the plan inherits the scaffold NA pass's 11 gaps but only re-checks 3; the other 8 (closed as spec text, never as code) are silently assumed discharged

**The frame collapse.** The scaffold NA pass surfaced 11 gaps. D-S1002127-03 deferred 3
(07/08/10) and recorded 8 as "absorbed into Wave 4.B." But Wave 4.B's absorption was
**spec-document authoring** — `ai_specs/substrates/` dossiers, a `RefusalToken` *section*
in `ERROR_TAXONOMY.md`, `substrate-couplings/` *markdown*. The completion plan is the
plan that takes the project to *shipped code*. It never asks: did the 8 "absorbed" NA gaps
become *code*, or only *spec*? `RefusalToken` (NA-GAP-02/11) — is it a Rust type in the
shipped tree, or a paragraph in a spec? NA-GAP-09's "substrate-confirmable receipt" — §8
*re-introduces* this as a Phase-3/4 *new* requirement, which is itself evidence that
NA-GAP-09 was **never implemented**, only spec'd. §8 silently re-derives a gap the scaffold
pass already raised and the project already claimed "absorbed."

**Second-order failure it hides.** The drift discipline in this workspace
(`feedback_flex_verify_before_ship`, "FP-verify the wiring, not just the contract") is
explicit: scaffolded spec text is not shipped behaviour. The completion plan FP-verifies
*carry-forward residuals* (Phase 1) meticulously but does **not** FP-verify whether the
8 "absorbed" NA gaps are wired. M0 ships and is tagged `v0.1.0`; the project record says
"8/11 NA gaps closed Wave 4.B." A future session reads that and trusts it. If
`RefusalToken` is spec-only, every substrate-refusal the engine meets at runtime is still
miscategorised (the exact NA-GAP-02 failure: operator restarts the wrong service). The gap
was "closed" on the documentation surface and "open" on the code surface — and the
completion plan, the one document positioned to catch that, does not look.

**Recommendation.** Add a Phase-1 sub-step (or a Phase 1.5): FP-verify the **code status**
of all 8 Wave-4.B "absorbed" NA gaps. For each, grep the shipped `src/` tree: is
`RefusalToken` a type? Is the m42 outbox drain-policy (NA-GAP-06) implemented? Is the
CC-5 substrate-confirmable receipt (NA-GAP-09) wired — and if §8 is adding it as new in
Phase 3/4, mark NA-GAP-09 as **was-spec-only**, correcting the project record. Any
absorbed-as-spec-only gap is either an M0 work item or an honest amendment to the
"8/11 closed" claim. This is a `correction`-class finding: the project record currently
over-states substrate-frame completion. *Confidence: 0.8.*

### NA-9 [SUBSTRATE] — §8's own self-check is absent: the plan runs an NA pass but never frame-collapse-checks that NA pass

**The frame collapse.** The scaffold NA pass ended with **§6 "Frame-collapse self-check"** —
it explicitly caught two of its own near-drifts (NA-GAP-05 drafted in Frame-D vocabulary
then corrected; NA-GAP-07 drafted as an engine-side TODO then re-rooted). That self-check
is *the discipline's recursion step*. The completion plan's §8 has **no self-check**. It
asserts its own conclusions ("the NA pass adds two concrete requirements … does not change
the phase count") and stops. A second pass with no frame-collapse self-check has no
mechanism to notice it has collapsed — which is exactly what NA-1 finds happened.

**Second-order failure it hides.** Without §8 carrying its own self-check, the plan ships
an NA section that *looks* like it satisfies the dual-frame discipline (it is even quoted
the Working Mode rule at its head) while having quietly re-run Frame A in Frame-B
vocabulary. The discipline document warns of exactly this: *"The worst output is a
beautifully written second-pass that turns out to have re-run the original frame in
different vocabulary."* §8 is well-written, cites the right ADRs, uses the right words —
and re-ran Frame A. The missing self-check is *why nobody caught it inside the plan.*

**Recommendation.** §8 must gain a closing self-check sub-section that asks, of itself:
"is this the substrate's frame or the engine's model of the substrate?" Applied honestly,
that question produces NA-1. The cheapest correct fix to the whole recursion problem is to
add the self-check; it is self-correcting. If a self-check is added and §8's three findings
survive it, fine — but they will not survive unedited, because "receipts/emission/channel"
do not pass "is this a substrate category or an engine category." *Confidence: 0.8.*

---

## 4. Load-bearing tensions (frame conflicts requiring explicit reconciliation)

### T-1 — "smallest honest M0" (Frame A) vs "substrate-facing organ must validate its substrate relationship before claiming a milestone" (Frame B)

Frame A (§3, §6, §9): the defaults yield "the smallest honest M0"; v0.2.0 substrate work
is a clean separate milestone; bounding scope is a virtue. Frame B (NA-7): a milestone that
certifies an engine *whose entire purpose is substrate-facing* without validating the
substrate relationship is not "small and honest" — it is *mis-scoped*, because it draws the
"done" line around the half of the system the engine controls. **Reconciliation:** the plan
must explicitly choose and *name* what `v0.1.0` certifies. Recommended wording in §9:
"`v0.1.0` certifies engine-internal completeness and is **not** a substrate-safety
milestone; substrate-safety is `v0.2.0`." That sentence reconciles the tension by making
the cut visible. Refusing to write it leaves the tension papered over.

### T-2 — "Phase 0 is one efficient interview round" (Frame A) vs "the operator is a substrate with a fatigue budget and Q6 is a load-bearing severance" (Frame B / NA-3, §8's own Q6 flag)

Frame A wants one round-set of seven questions for scheduling efficiency. Frame B (and §8's
*own* Q6 flag) says Q6 severs the substrate's compositional voice and must be a *seen*
choice — which a cheap default in a seven-item batch structurally prevents.
**Reconciliation:** split the interview (NA-3 recommendation). The plan cannot both
"flag Q6 as load-bearing" (§8) and "give Q6 a cheap default in a batch of seven" (§3)
without contradiction. Pick one. Recommended: Q4/Q6 + the §11.3 meta-question become a
separate, no-default decision surface.

### T-3 — "§8 satisfies the dual-frame discipline" (the plan's claim) vs "§8 is a Frame-A self-audit" (NA-1, NA-9)

The plan asserts §8 *is* the second pass. This analysis finds it is not. This is not a
matter of degree to paper over — either the plan adds a genuine substrate-frame
re-authoring (or honestly relabels §8), or it ships claiming a discipline it did not run.
**Reconciliation:** there is no middle option. Relabel §8 honestly OR add §8-bis. The
recursion check exists precisely to force this choice.

---

## 5. Convergent findings (both frames agree — strongest signal)

### C-1 [CONVERGENT] — the plan's own persistence steps (Phase 1 DOC-1, Phase 8 four-surface persist) must read-back-verify their stcortex writes

Frame A reaches this via FP-verify discipline (`feedback_flex_verify_before_ship`: agent
reports of "done" are evidence, not fact; verify the write landed). Frame B reaches it via
NA-6 (stcortex is an actor that can refuse the write; refuse-write and slug-reducer refusal
are substrate-authored). Both frames produce the identical action: **after every stcortex
write the plan performs, `SELECT` it back and treat absence as failure.** This is the
highest-confidence recommendation in this document — adopt it unconditionally.

### C-2 [CONVERGENT] — the 8 "absorbed" Wave-4.B NA gaps must be FP-verified for *code* status, not assumed from the project record

Frame A: the workspace drift discipline ("FP-verify the wiring, not just the contract")
demands it — scaffolded spec is not shipped code. Frame B (NA-8): §8 re-introducing
NA-GAP-09 as a *new* Phase-3/4 requirement is direct evidence that "absorbed" meant
"spec'd," not "wired." Both frames converge on: **Phase 1 must grep `src/` for the 8
absorbed gaps and correct the "8/11 closed" record where they are spec-only.**

---

## 6. Summary table

| ID | Frame collapse in §8 / the plan | Ships if unaddressed |
|----|--------------------------------|----------------------|
| NA-1 | §8's "substrate frame" = engine legibility (receipts/emission/channel), not substrate agency | "NA pass done" checkbox conceals the real substrate gaps |
| NA-2 | Read-side substrates (atuin et al.) modelled as sinks; no attention budget | Phase 6 "live atuin" exercise can starve the user's shell, invisibly |
| NA-3 | Operator = decision oracle (Phase 0), not a substrate with fatigue/refusal | 7-question batch + cheap defaults → minimal M0 by path-of-least-resistance |
| NA-4 | Conductor = dispatch sink + health probe, not an actor with enforcement clock | m33 verifiers "complete" while Conductor may not enforce them at all |
| NA-5 | Recency/decay on engine clock; substrate clocks (TTL sweep etc.) unmodelled | m11 compound decay correct in isolation, incoherent in the CC-5 loop |
| NA-6 | Phase 1 & 8 stcortex writes can be refused; plan doesn't verify its own writes | Four-surface persistence silently degrades to three (POVM write-only trap) |
| NA-7 | M0/v0.2.0 partition is itself a frame choice; "complete" = engine-internal only | `v0.1.0` tag certifies substrate-facing organ without substrate validation |
| NA-8 | 8 "absorbed" scaffold NA gaps assumed wired; only spec'd | "8/11 NA gaps closed" over-states; runtime refusal still miscategorised |
| NA-9 | §8 has no frame-collapse self-check (scaffold pass §6 did) | The NA pass cannot notice it collapsed — which is why NA-1 happened |

---

## 7. Verdict

The completion plan S1004115 is a strong **conventional** plan — well-sequenced, honest
about residuals, correctly partitioning by ADR, and disciplined on FP-verification of
carry-forward items. Its §8 is a conscientious Frame-A self-audit that produces two
genuinely useful additive requirements.

But as a **dual-frame** plan it fails the recursion check: §8 is not the second pass. It
re-authors nothing (NA-1, NA-9), models every substrate — read-side databases (NA-2), the
operator (NA-3), the Conductor (NA-4) — as a sink/oracle rather than an actor, runs decay
on an engine-owned clock the substrate does not share (NA-5), does not verify its own
substrate writes (NA-6), and ratifies an M0/v0.2.0 partition that silently defines
"complete" as "engine-internal" (NA-7) while assuming 8 spec-only NA gaps are shipped code
(NA-8).

**The single highest-leverage fix:** add a frame-collapse self-check to §8 (NA-9). Applied
honestly it forces NA-1, which forces an honest relabel of §8 or a genuine §8-bis, which
forces NA-7's partition question to be named. The recursion is self-correcting once the
self-check exists. Everything else in this document follows from that one missing step.

**This is not BALANCED_INPUT.** The plan is single-frame with a Frame-A appendix mislabelled
as the second pass.

*Filed 2026-05-23 · `na-gap-analyst` · recursion check on WORKFLOW_TRACE_COMPLETION_PLAN_S1004115.md §8 · advisory; does not gate M0.*
