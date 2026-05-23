# Conventional (Engineering / PM) Gap Analysis — Workflow-Trace v0.2.0 Plan v1 S1004377

> **Subject:** [`ai_docs/WORKFLOW_TRACE_V020_PLAN_S1004377.md`](WORKFLOW_TRACE_V020_PLAN_S1004377.md)
> **Analyst pass:** skeptical engineering review · 2026-05-23 · sibling to the NA pass dispatched in parallel
> **Method:** read Plan v1 end-to-end; FP-verified `git log`, `git tag`, the v0.1.0 honest-residual surface
> in `CHANGELOG.md`, the Phase 2 audit ([`PHASE2_AUDIT_S1004115.md`](PHASE2_AUDIT_S1004115.md)), the SD
> reconciliation doc ([`PHASE9_SD_RECONCILIATION_S1004115.md`](PHASE9_SD_RECONCILIATION_S1004115.md)),
> ADR `D-S1002127-03` ([`decisions/2026-05-17-substrate-as-actor-deferrals.md`](decisions/2026-05-17-substrate-as-actor-deferrals.md)),
> shipped source in `src/`, `Cargo.toml`, `.github/workflows/ci.yml`, `.gitlab-ci.yml`, the cross-talk
> directory, and the Plan v2 precedent gap-analysis
> ([`WORKFLOW_TRACE_COMPLETION_PLAN_S1004115_CONVENTIONAL_GAP_ANALYSIS.md`](WORKFLOW_TRACE_COMPLETION_PLAN_S1004115_CONVENTIONAL_GAP_ANALYSIS.md)).
> **Verdict in brief:** Plan v1 is structurally sound, honestly Plan-v2-shaped, and unusually
> good at flagging cross-habitat dependencies as no-default interview slots. It still ships
> with **one wrong file:line citation** on a load-bearing claim (Q1 / V1 anchor), **at least one
> mis-classified "decision-free" item that quietly couples to the keystone**, an **interview-question
> set that conflates orthogonal decisions in DX-W and DX-V5**, **two phase effort estimates that
> ignore well-known overheads** (Phase 9 Genesis cascade external dependency; Phase 6 mutation-run
> wall time), **three missing risk-register classes** (V5 schema-drift across substrate boundary,
> V3 alert-fatigue / canary-of-canary, CI / mutation-gate budget blowout), and **its own §13
> single-surface drift risk** flagged as defensible but unaccompanied by a mechanical recovery
> contract. The plan should not start until C-2, C-3, and C-7 are resolved — each one mis-sizes
> or mis-directs the Tier-1 / Tier-2 critical path or lets Plan v2's NA-1 trap re-occur in v1's
> Part B.

---

## C-1 — § 2 inventory cites RefusalReason at `src/m32_dispatcher/mod.rs:163`; the enum is at `:228`. **Severity: LOW**

**Gap.** § 2.1 V1 row says the flat `RefusalReason` enum is "at `src/m32_dispatcher/mod.rs:163`".
The shipped tree has the enum declaration at **line 228** (`pub enum RefusalReason {`); line 163
is unrelated. Phase 7 step 2 repeats the same file:line. Phase 2 audit also cites `:163`
(inherited from the original PHASE2_AUDIT_S1004115 doc), so this is propagated.

**Why it matters.** It is a small citation drift — but the plan is meant to be the document a
fresh Claude window reads and follows. A misleading file:line means Phase 7 step 2's "replace flat
`RefusalReason` enum at `m32_dispatcher/mod.rs:163`" will not pattern-match the actual code on
the first read, and the v1 → v2 lineage will silently inherit the wrong number. Same class as
C-1 in the Plan v2 gap analysis (stale HEAD), at smaller magnitude.

**Evidence.** `grep -n "pub enum RefusalReason" src/m32_dispatcher/mod.rs` → `228`. Plan v1
§ 2.1 V1 row + Phase 7 step 2 + Phase 2 audit § 2 row 1 all cite `:163`.

**Recommendation.** Update V1's anchor to `src/m32_dispatcher/mod.rs:228` (where the enum is
declared), and Phase 7 step 2 likewise. Add a Phase 1 task "re-verify every file:line in
inventory § 2 against shipped tree" — the plan already calls Phase 2 a "decision-free FP
verification"; expand its step 1 explicitly to *all* anchors in § 2, not just the three
named NA gaps.

---

## C-2 — V1 RefusalToken is **structurally coupled** to the Tier 4 SD11 / W3 wire bump; the plan sequences them three phases apart. **Severity: HIGH**

**Gap.** Plan v1 ships V1 RefusalToken in Phase 7, but the actual production refusal sites that
need re-typing are not the just-the-`m32_dispatcher::RefusalReason` enum — the substrate-frame
half of refusal lives in `m13_stcortex_writer` (outbox writes — `outbox_path`/`outbox_lock`),
`m40_nexus_emit` (`ServerRejected` events), and the new substrate-confirmable receipts that
shipped in Phase 6f (verified at `src/m40_nexus_emit/mod.rs:51,248,572,1860` + `src/orchestration/
dispatch.rs:223`). Each one of those sites already carries a refusal-shaped *event* that v0.1.0
emits **without** authorship typing. When V1 lands, *every existing serialised event with a
RefusalReason* gets its serde shape bumped — the JSONL outbox at `m13` is the most painful
consumer because v0.1.0 *already* writes refusal-shaped lines via the outbox-first defer path.

That makes V1 not "an enum replacement" but a **second JSONL wire-contract change** that lands
**after** W1 (Phase 5) and W3 (Phase 5). Plan v1 treats them as orthogonal — they are not. A
proposal serialised with the W1 `escape_surface` field in Phase 5 will still encode any V1
refusal via the *old* `RefusalReason` shape; Phase 7 then re-bumps the same lines two phases
later. The Phase 5 JSONL-fixture regen is paid twice.

**Why it matters.** Plan v1's Phase 5 risk-register entry calls out the W1 blast radius and
schedules the regen as a single landed unit. But Phase 7's re-bump of every refusal-shaped
serialised line is not mentioned in either Phase 5 or Phase 7's mitigation — it just appears
inside Phase 7's "Wire the m13 drain skeleton to consume RefusalToken-tagged events" step.
The actual *re-regen* of all the `outbox_*.jsonl` test fixtures (and any committed
`tests/m13_integration.rs` / `cc5_substrate_cycle.rs` fixture lines) is invisible.

**Evidence.** `src/m13_stcortex_writer/mod.rs:307–369` carries `outbox_path` + `outbox_lock`;
m13 tests `:472–955` write refusal-shaped lines to the outbox (per Phase 2 audit § 2 NA-GAP-06
row). `src/m40_nexus_emit/mod.rs:51,248,572,1860` emit substrate-confirmable receipts that include
refusal envelopes. `src/orchestration/dispatch.rs:223` carries the receipt emission seam.

**Recommendation.** Either (a) **co-land V1 with Phase 5 W1** so the JSONL wire-contract regen
happens once (cost: Phase 5 grows by ~1-1.5 days; Phase 7 shrinks to just the m9/m32/m41 call-site
threading + the m13-drain wire), or (b) **explicitly add a Phase 7.0 "JSONL re-regen pass" sub-step**
naming the outbox fixtures + integration-test fixtures that need re-baselining, and budget the
mutation re-run on the *combined* `m13 + m32 + m40 + m41 + m42` surface. As written, the cost
is hidden inside Phase 7's "~80-150 new tests" line. This is the Plan-v2-conventional C-2/C-7
class re-occurring.

---

## C-3 — DX-W in Phase 4 conflates two orthogonal decisions: *where the surface lives* and *whether the wire breaks*. **Severity: HIGH**

**Gap.** Phase 4 Round A frames DX-W as a binary: "W1 (escape_surface wire field) **or** W2
(StepToken classification table)". Reading the Phase 2 audit ([`PHASE2_AUDIT_S1004915.md`](PHASE2_AUDIT_S1004115.md))
§ 3, the option-space is *three* (i / ii / iii); v0.1.0/M0 shipped option (iii) — defense-in-depth
redundancy against m32's existing monotone gate, with `ConservativeVerifier::verify → Approve`
as the placeholder. DX-W's binary frame implicitly retires (iii). That is a real decision —
"is the v0.2.0 R1 Security verifier a *new* enforcement seam or an *audit overlay* on an
existing one?" — and it is being decided silently by virtue of DX-W's two-option framing.

There are also **two coupled sub-decisions** hidden inside DX-W:
1. **Variant aggregation function** — W2 needs to define the StepToken → EscapeSurfaceProfile
   aggregation (max-fold per Plan v1 Phase 5 step 2; but bag-of-tokens vs. ordered-prefix
   matters for collision-prone token semantics) — that is its own decision the plan defers
   to "table tests" with no spec slot.
2. **Backwards-compat for in-flight proposals** — if W1 is chosen, every existing JSONL
   proposal-fixture (in `tests/wf_crystallise_integration.rs`, `tests/wf_dispatch_integration.rs`,
   the cc5 substrate cycle, the cc7 pressure evolution) needs either a default value at
   the field-new-on-deserialize boundary, or a wire-bump that breaks read of any v0.1.0
   proposal sitting in a developer's `proposals.jsonl`. That is a SemVer-visible decision.

**Why it matters.** A "no-default load-bearing" interview question that secretly bundles
three decisions burns Luke's 3-decisions-per-week budget on a single slot and risks the
answer locking-in two sub-decisions that were never aired. DX-W as written cannot be answered
without surfacing (iii) and the SemVer compat question.

**Evidence.** Plan v1 § 15 Round A DX-W (two options); Phase 2 audit § 3 Phase 6a Security
re-sizing (three options); shipped `src/orchestration/dispatch.rs:333`
(`ConservativeVerifier::verify → Approve` is the (iii) M0 surface).

**Recommendation.** Split DX-W into three rolled sub-questions inside Round A:
- **DX-W.a** — Retire option (iii) audit-overlay path, or keep it as the v0.2.0 default and
  add W1/W2 as additive enforcement? (yes/no)
- **DX-W.b** — If new enforcement seam: W1 wire-bump or W2 in-engine classifier? (i / ii)
- **DX-W.c** — If W1: SemVer-bump (`v0.2.0` is a breaking proposal-wire change) or
  defaulted-field for backwards-compat read of v0.1.0 proposals? (break / default)

Three answers, ~3-5 min of interview time, no silent sub-decisions.

---

## C-4 — Phase 9's V3 own-module branch has a **multi-day external dependency** invisible in the effort estimate. **Severity: HIGH**

**Gap.** Phase 9 step 1 (DX-V3 = own module) lists in passing: "Dispatch Zen G7 re-audit
(cardinality drift) per D-S1002127-03 § 3 trigger." ADR `D-S1002127-03` § 3 confirms: adding
m16 *triggers* Genesis Prompt v1.4 module-count amendment (26 → 27) **and** Zen G7 re-audit
cascade. The same ADR also confirms the v0.1.0 module-count is "locked at 26 (per v1.3-amendment
OI-3 resolution)".

A Zen G7 re-audit is structurally a multi-day external dependency:
- Plan v2's Phase 9 absorbed a Zen verdict that took *unbounded* time. The Plan-v2-conventional
  gap-analysis C-4 caught that "Zen has not engaged with workflow-trace hardening at all"
  (no `zen_*verdict*` file in cross-talk for W1/W2/W3/W4 / S1003733 / C22; the closest is the
  Command-authored `2026-05-23T030611Z_zen_command_wfe_c22_waveg_verdict.md` notice).
- v1's Plan v1 then proposes to trigger *another* Zen G7 cascade as a single bullet in
  Phase 9 with the rest of Phase 9 budgeted at "~3-4 days".

Phase 9 step 5's "Re-run `cargo-mutants` scoped to the canary code path" is also non-trivial:
m16 is a periodic-prober with 5 clock samplers and a coordinator. Mutant runs against a
periodic-prober module are slow because each mutant rebuilds in release mode (per Plan v2
Phase 6 risk-register precedent); a scoped mutation run with the DX-Mut "hold ≥96.3 %" bar
plausibly costs 2-6 hours of wall time per iteration.

Combined: Phase 9's "~3-4 days" implicitly assumes the Genesis amendment + Zen re-audit
will land *within* the phase window. The Plan v2 evidence is that Zen response times
on workflow-trace hardening have been measured in *days-to-weeks*, not hours.

**Why it matters.** Phase 9 is on the critical path (Plan v1 § 4: "Phase 2 → Phase 4 →
Phase 5 → Phase 6 → Phase 7 → Phase 9 → Phase 10 → Phase 12"). If DX-V3 = own module, the
estimate is honest only if the cascade is *concurrent* with implementation — but the plan
language ("starts with the Genesis amendment file and dispatches re-audit early") makes
that a hope, not a contract.

**Evidence.** ADR `D-S1002127-03` § 3 W1 row (m16 triggers Genesis v1.4 + Zen G7 re-audit);
cross-talk dir (no `zen_*verdict*` file for any workflow-trace hardening wave); Plan v2
conventional gap analysis C-4 ("verdict absent, not slow").

**Recommendation.** Phase 9's effort range should explicitly fork by DX-V3:
- **DX-V3 = distributed canary:** ~2-3 days (no Genesis amendment, no Zen cascade, but
  harder testability).
- **DX-V3 = own module:** **~5-12 days** (Genesis v1.4 authoring ~1 day + Zen G7 re-audit
  external wait ~3-10 days depending on Zen engagement + implementation ~3-4 days; partially
  parallelisable but the ship gate is "Zen verdict returns").

Add a Phase 4 sub-question DX-V3.b — "if DX-V3 = own module and Zen verdict is silent for
N days (e.g., N=7), do we ship v0.2.0 without Zen approval of the cardinality drift, or hold?"
— mirrors Plan v2 conventional gap-analysis C-4's escalation requirement. Without this, v0.2.0
inherits Plan v2's "ship without audit" risk on its own keystone primitive.

---

## C-5 — Phase 5 W3 cost-population source is **unverified** and may force a substrate-side change. **Severity: HIGH**

**Gap.** Plan v1 Phase 5 step 3 says W3 "populate[s] in `compose_proposals` from `step-count ×
mutation-weight` per D10". The Plan v2 § 15 D10 metric is `step-count × mutation-weight`.
FP-verifying against shipped code:
- `step-count` is available — `m22_kmeans::RECOMMENDED_K_MAX = 8`; `WorkflowVariant.steps`
  carries the step list (m21).
- `mutation-weight` is **not a defined primitive in the live tree**. `grep -rn
  "mutation_weight\|mutation-weight" src/` returns zero. The only "mutation" primitive in
  the codebase is `WorkflowVariant.mutation` (m21 variant-builder; integer mutation count
  per variant) and `cargo-mutants` (test infra).

If D10's "mutation-weight" is *the WorkflowVariant.mutation count*, fine — but Plan v2 § 15
D10 doesn't say so and Plan v1 doesn't pin it. If it's *something else* (e.g., a per-StepToken
class-weight table, or an m11 fitness-weighted score), Phase 5 W3 needs to author a new
weighting primitive before it can land — that is unbudgeted work.

This is the same class of finding as Plan v2 conventional C-3 ("Cost verifier rests on an
unverified `cost_estimate` field"). The Plan v2 doc closed C-3 by deferring D9 to a Cost
stub. Plan v1's R2 Cost verifier (Phase 6) needs W3, and W3 needs a `mutation-weight`
definition that — at HEAD `d521a00` — does not exist in source.

**Why it matters.** Phase 5 is on the critical path. If `mutation-weight` requires either
(a) a new lookup table (added scope, ~50-100 LOC + tests), (b) wiring m11 fitness weight
through m23 compose (touches multiple modules), or (c) per-StepToken hand-built weights
(habitat-wide policy decision), Phase 5's "~3-5 days" range is a floor, not a ceiling.

**Evidence.** `grep -rn "mutation_weight\|mutation-weight" src/` → empty; Plan v1 § 15 DX-4
(only mentions Levenshtein source, not mutation-weight source); Plan v2 § 15 D10 ("step-count
× mutation-weight"); Plan v2 conventional C-3 precedent.

**Recommendation.** Add a Phase 1 / Phase 2 verification step: define what `mutation-weight`
*is* against the live tree, and pin its source (variant.mutation count vs. new table vs.
m11 fitness pull-through). If new, raise the Phase 5 W3 estimate to ~250-400 LOC and add
DX-W3.src to Round B ("mutation-weight source: variant.mutation count / new StepToken table /
m11 fitness pull-through"). Without this, Phase 6 R2 will run into the same blocker.

---

## C-6 — Phase 6 R3 Consistency "bank-conflict detection" has a hidden sub-decision the plan flags but never resolves. **Severity: MED**

**Gap.** Plan v1 Phase 6 step 3 (R3) admits: "Define what 'conflict' means — overlapping
variant_id, lineage-chain duplication, or both? Locks in Phase 4 if surfaced; otherwise local
sub-decision." That admission is honest but operationally fragile:
- Phase 4 § 15 has no DX slot for it.
- "Local sub-decision" means an implementing agent picks one of three semantics and lands it
  in code; the choice becomes ratified-by-existence the moment R3 ships.
- *Each semantic implies a different schema*. `variant_id` overlap only needs the existing
  `WorkflowVariant.variant_id` field. `lineage-chain duplication` needs the `lineage_chain`
  field (A4 SD11 12-field proposal shape — not landed until Phase 12). `Both` needs both,
  with a precedence rule.

So R3's "real verifier" depends on the A4 SD11 shape — which lands in Phase 12, *after* Phase 6.
Phase 6 R3 ships before SD11, meaning either R3 is a single-axis (`variant_id`-only) shim that
gets re-shaped in Phase 12, or A4 SD11 moves to Phase 5/6.

**Why it matters.** A "real verifier" that needs to be re-shaped two phases later is a
documented-stub by another name. Plan v1's tier sequencing has R3 in Phase 6 and SD11 in
Phase 12; one of them is in the wrong phase.

**Evidence.** Plan v1 Phase 6 step 3 ("Define what 'conflict' means … Locks in Phase 4 if
surfaced; otherwise local sub-decision"); Plan v1 Phase 12 step 3 (A4 SD11 "co-lands with
W1+W3 from Phase 5: six new fields per Phase 9 § 2b SD11 row").

**Recommendation.** Move A4 SD11 *into* Phase 5 (it's a wire-contract addition; pairs with
W1/W3 naturally; the Plan v1 § 2.4 notes "naturally co-lands with W1+W3"). Then R3 in Phase 6
can use the 12-field shape unambiguously. Add a new Round-B question DX-R3: "Consistency
conflict semantic: variant_id-only / lineage-chain-only / both with precedence" with a default
(variant_id-only) for v0.2.0 and an explicit deferral note that lineage-chain is post-v0.2.0
if too costly.

---

## C-7 — § 5 effort roll-up `~22-36 Claude-days` ignores the Plan-v2-class overhead the v1 itself was created from. **Severity: MED**

**Gap.** Plan v1 § 5 honestly raises the stub catalogue's `~6-10 Claude-days` to bottom-up
`~22-36 Claude-days`, ascribing the gap to ADRs / gap analyses / 4-surface persist / scoped
mutants / audit fold-in. It correctly observes this is "roughly double v0.1.0/M0's ~10-13
Claude-days — proportional to the LOC delta (~2200-2900 vs ~600)".

The bottom-up still under-counts three categories the Plan-v2 arc made evident:
1. **Plan-v2-arc overhead.** Plan v2 itself authored `Plan v1 → Conventional GA → NA GA →
   Interview → Plan v2 → 4-surface persist`. Plan v1's § 14 commits to the same arc for v0.2.0,
   but the effort roll-up (§ 5) prices only the *implementation* phases. The plan-authoring
   round-trip is real ~2-4 Claude-days that should be a line item.
2. **Interview cascade.** § 15 has 11 seed questions + "8-16 more across topics" surfaced by
   gap analyses. A Round A + Round B interview at Plan-v2's depth was ~12 rounds / 48 questions
   — and Plan v1 explicitly says the v0.2.0 interview "will likely surface 8-16 more". A
   ~20-question interview, at the Plan-v2 measured depth, is ~2-3 hr *per session* across ~3-5
   sessions, often spread over multiple days while Luke considers (Plan v2 D45/D46 conviction
   cycles took the rest of a session). Plan v1 line-items this at "~2-3 h" — that is the
   *active* time, not the calendar time.
3. **Mutation gate budget under-counted.** Plan v1 schedules scoped `cargo-mutants` on
   Phases 5/6/7/8/9/12 + full re-run in Phase 12. Per Plan v2 W4 evidence, a scoped m33 mutants
   run (per Phase 6) is ~1-3 hours; full workspace mutation (Phase 12) is ~6-12 hours and
   includes mutant-triage time. Cumulative mutation wall time across v0.2.0 is plausibly
   1.5-3 full Claude-days.

Add those three categories conservatively and the realistic top of `~22-36` becomes more like
`~28-44`. The plan should at least line-item the gap.

**Why it matters.** Effort estimates that under-count Plan-v2 cycle overhead let v0.2.0 fall
into the "fatigue / opportunity-cost shift" risk that Plan v1 § 6 already flags but does not
quantify.

**Evidence.** Plan v2 Conventional GA C-7 precedent (4 verifiers non-uniform); Plan v2 effort
roll-up "~10-13 working days" *after* 48 interview decisions locked, which is itself ~10-15
hr of interview time spread across ≥2 sessions; Plan v1 § 5 phase decomposition.

**Recommendation.** Add three line items to § 5:
- "Plan v1 → v2 round-trip + persistence (this analyst pass + NA + interview write-up) — ~1-2
  Claude-days"
- "Interview rounds (~12-20 questions) + Luke decision latency — ~0.5-1.5 calendar-days"
- "Cumulative scoped + full mutation wall time across all phases — ~1.5-3 Claude-days"

New total: **~25-42 Claude-days**, mid-point ~33. State that the ~22-36 quoted lower bound
applies only to phases 1-12 *narrow execution*, not the full Plan-v2 arc.

---

## C-8 — Phase 1 ADR D-S1002127-03 amendment is treated as a *doc edit*; it is a **language change** with downstream consequences. **Severity: MED**

**Gap.** Phase 1 step 2 says: "Amend ADR `D-S1002127-03` to register V1 (NA-GAP-01) + V2
(NA-GAP-04) + C1 (NA-GAP-06 drain) as now-active v0.2.0 work-items." That sounds simple, but
ADR `D-S1002127-03` is currently the *deferral charter* — its language across §1 and §3
deliberately states these gaps are deferred and *out of scope* for v0.1.0. Multiple downstream
docs reference the deferral language:
- v0.1.0 CHANGELOG entry "Honest residuals — v0.2.0 candidates" cites these gaps by reference
  to the ADR.
- Plan v2 § 11 names the ADR as the authority for the deferral.
- The substrate-coupling spec docs ([`ai_specs/substrate-couplings/`](../ai_specs/substrate-couplings/),
  per ADR § 3) carry "to be re-activated in v0.2.0" language.

Amending the ADR shifts those gaps from "deferred" to "active" — which requires *removing the
deferral language*, not just appending an amendment section. That has the same kind of churn
risk Plan v2's gap analysis C-1 caught (stale HEAD): any doc that copied the deferral verbatim
now disagrees with the ADR.

**Why it matters.** Plan v1 Phase 1 budgets `~0.5-1 day` for re-baseline + ADR amendments. A
proper ADR amendment cascade across the deferral surfaces is plausibly half a day on its own;
combined with the other Phase 1 work (re-baseline diff, three new ADRs, CHANGELOG update,
CLAUDE.md / project CLAUDE.local.md / stub supersession), the budget is tight.

**Evidence.** ADR `D-S1002127-03` § 3 W1 row; CHANGELOG `[v0.1.0]` "Honest residuals — v0.2.0
candidates" entry; Plan v2 § 11 partition language; `ai_specs/substrate-couplings/` directory.

**Recommendation.** Add a Phase 1 step 2.5: "Search every doc that references
`D-S1002127-03` deferral language; either update the language to 'now-active v0.2.0' or
add a cross-reference to the amendment". Bump Phase 1 effort to `~1-1.5 days` to be honest.

---

## C-9 — Risk register misses three classes: V5 cross-habitat schema-drift, V3 canary-of-canary observability, mutation-run budget overflow. **Severity: MED**

**Gap.** Plan v1 § 6's 10-row risk register covers the obvious ones (W1 fixture regen,
V3 own-module cascade, V5 cross-habitat slip, V4 fixture realism, mutation kill-rate slip,
agent over-claim, Tier-2-first ordering, RefusalToken cascade, Luke fatigue, V3 self-canary).
Three classes are missing:

1. **V5 substrate-side schema-drift after engine-side ships.** If DX-V5 = full cross-habitat,
   workflow-trace ships the *engine consumer* of `SubstrateTrust { stcortex_score,
   conductor_budget_remaining, atuin_quota_remaining }`. The substrate-side ADRs land later
   and the substrate-side teams may pick a different field shape (different name, type,
   semantics — recall S112 port-drift, S117 schema-shape changes). Workflow-trace's V5
   consumer then silently desyncs.
2. **V3 canary alert-fatigue / canary-of-canary problem.** V3 m16 emits `SubstrateDriftDetected`
   events. The plan's mitigation for "canary itself drifts" is "*absence* of canary events is
   itself observable in m40 Nexus operator-visible alert" — but operator-visible alerts have
   a fatigue cost. If V3 fires several times per soak period (5 substrate clocks × hourly
   pulse × typical envelope sloppiness), the alert stream may be ignored. The plan has no
   alert-budget primitive.
3. **Mutation-run wall-time budget blowout against gate-per-phase discipline.** Plan v1
   commits to "gate per commit … cargo-mutants scoped per phase + full re-run Phase 12".
   If DX-Mut is raised from 96.3 % to 98 %, every Tier-1 phase plausibly needs ≥2 mutation-fix
   cycles (per § 6 risk row 5 — "budget for ≥2 mutation-fix cycles per Tier-1 phase"). Cumulative
   wall time can blow past the per-phase day budget; the plan has no fallback ("if mutation
   bar is missed by ≤1 mutant, ship with `// mutant-equivalent:` proof; > 1 mutant blocks").

**Why it matters.** Each missing class is a v0.2.0-killing risk the plan does not have a
mitigation for. The Plan v2 era is rife with each (S117 schema drift; S112 wire drift; Plan
v2 W4 mutation iteration count).

**Evidence.** Plan v1 § 6 risk register (10 rows); Plan v2 W4 final-mutation-fold history
(multiple iterations across Wave G); S112 / S117 precedents in CLAUDE.local.md workspace
charter.

**Recommendation.** Add three rows to § 6:
- *"V5 substrate-side schema lands different shape than engine-side primitive"* → mitigation:
  ship V5 as a versioned ADR with a `serde(other)` fallback variant on the consumer side; flag
  any unknown field as a substrate-authored `RefusalToken::SubstrateAuthored` event.
- *"V3 canary alert-fatigue / false-positive storm"* → mitigation: V3 emits at most N alerts
  per soak hour with rate-limiter + alert-dedup by (clock-pair, envelope-band); operator
  visible only after N consecutive crossings.
- *"Mutation gate budget blowout"* → mitigation: per-phase mutation wall-time cap (e.g., 4 h);
  > cap = either ship with documented `// mutant-equivalent:` proofs or defer mutation work to
  a v0.2.1-mut sub-release.

---

## C-10 — § 15 Round B has at least three questions that are **already-answered defaults**, not real interview slots. **Severity: MED**

**Gap.** Round B questions should be "mechanical / policy with defaults acceptable". Several
are already answered or have a clearly load-bearing default that does not need a Luke slot:
- **DX-Mut** ("hold ≥96.3 % or raise to ≥98 %?") — the v0.1.0/M0 bar is 96.3 % and the
  "raise" arm requires unbudgeted work (per C-9). The honest default is "hold; raising is a
  separate v0.2.1 sub-release". This does not need an interview slot — it needs an honest
  statement in the plan.
- **DX-Soak** ("24h or longer?") — D34/D35/D36 already lock the 24h baseline (Plan v2). v0.2.0
  adds substrate-as-actor primitives, so a longer soak is justified — but "longer" is
  unbounded. Honest defaulted answer: "48h, extensible per Watcher carriage signal".
- **DX-1** ("V1 RefusalToken variants — confirm 4-variant or amend with `WatcherAuthored`?")
  — the v0.1.0 RefusalReason taxonomy + Watcher's m46-m51 separate emission lane suggest
  `WatcherAuthored` should *not* fold into the operator/substrate/engine triad because
  Watcher emits *observations*, not *refusals*. Default: "4-variant; Watcher emits via the
  observation channel, not RefusalToken".

Conversely, three additional questions belong in **Round A** but are not there:
- **DX-A4-coupling** — A4 SD11 12-field proposal shape (per C-6) needs to fold *into* Phase 5
  (with W1+W3) or stay in Phase 12. That coupling is a real Round-A call.
- **DX-CI** elevation — Plan v1 Round B has DX-CI as "submodule / vendor / wait-for-crates.io".
  C-5 of Plan v2 already flagged this as Q8 territory; for v0.2.0, choosing wrong has
  CI-doesn't-run risk that masks regressions for the substrate-as-actor surfaces. Should be
  Round A.
- **Mutation-gate budget cap** — per C-9 row 3. Operationally load-bearing; should be Round A.

**Why it matters.** Plan v1's interview is the v0.2.0 fatigue-budget gate. Spending Round B
slots on questions with clearly-correct defaults while putting load-bearing decisions in the
"will surface from gap analyses" pile means the interview won't actually cover the right ground.

**Evidence.** Plan v1 § 15 Round A (3 slots) + Round B (8 slots); D34/D35/D36 in Plan v2;
C-6 + C-9 + C-5 of this gap analysis.

**Recommendation.** Promote DX-A4-coupling + DX-CI + Mutation-gate-cap into Round A. Demote
DX-Mut + DX-Soak + DX-1 into "stated defaults" in § 14 status block (not interview slots). Net
interview length unchanged but slots used for real decisions.

---

## C-11 — Part B (§ 7-10) **annotates rather than re-authors** in places; Plan v2 NA-1 trap is partially recursing. **Severity: MED**

**Gap.** Plan v1 § 9 has a frame-collapse self-check that asks "does Part B re-author or
annotate?" and answers: "It re-authors the *certification criterion* … the phase count's spine —
Tier 2 → Tier 3 → Tier 1 → Tier 4 — is unchanged because the engine-side work is real and needs
doing regardless."

Reading § 7-8 critically:
- § 7 lists each substrate (atuin / stcortex / Conductor / CC-5 clocks / Luke) and asks "does
  the substrate experience workflow-trace differently after v0.2.0?". Each gets a one-line
  answer of the form "the substrate's X is now Y". That is *annotation* of v0.2.0's primitives,
  not a re-author of the certification criterion.
- § 8 re-states "v0.2.0 certifies engine + substrate co-completeness" — that is just § 1
  rephrased.
- § 9 honestly admits V2 push-mode is Frame-B-half, V3 is engine-side aggregation, V5 ships
  listening-side. But § 9 does not interrogate whether the *Tier 2 → Tier 3 → Tier 1 → Tier 4*
  spine itself is engine-frame. It justifies preserving the spine ("engine work needs doing
  regardless"), which is a defensible move but is also exactly the *kind* of move Plan v2
  NA-1 caught (preserving the v1 spine while claiming a substrate-frame self-check).

The cleaner test: would a substrate-frame author *start* with Tier 2 (engine-side wire-contracts)
or with Tier 1 (substrate primitives)? Plan v1's DAW-1 ("Tier 2 first") is Luke's call — but
the substrate frame asks for Tier 1 first because primitives define the contract the wire
encodes. § 10 T-2 names this tension but reconciles by deferring to Phase 4. So the spine is
*known to be Frame-A-shaped*, with the Frame-B alternative deferred but not authored.

**Why it matters.** Plan v2 NA-1 caught the same recursion: "§ 8 was Frame-A self-audit
mislabelled as substrate frame". Plan v1's § 9 is honest about three primitives being
Frame-B-half — but the **spine** is the Frame-A authoring choice that survived. The recursion
isn't fully closed.

**Evidence.** Plan v1 § 8 ("§ 8 re-authors the certification criterion"); Plan v2 NA gap
analysis NA-1 ("Frame-A self-audit mislabelled"); Plan v1 § 10 T-2 ("DAW-1 Tier-2-first is the
right engineering ordering" vs "substrate frame asks for substrate primitives first").

**Recommendation.** Either (a) explicitly state Part B is *partial* re-author (certification
criterion re-authored; spine preserved by engineering call), with a § 9 line "Part B did
*not* re-author the tier sequence; the substrate-frame alternative is the Tier-1-first ordering,
which is the DX-W escalation per § 10 T-2", or (b) actually re-author Part B's spine and let
the NA-gap-analyst pass decide whether to fold the alternative. The current text claims
re-authoring without quite delivering it.

---

## C-12 — § 13 single-surface-until-v2 has no mechanical recovery contract if v1 is lost. **Severity: MED**

**Gap.** § 13 says "This v1 itself is single-surface (ai_docs only) until next turn." That
is defensible (the analyst pass + NA pass + interview should fold in before the 4-surface
persist) — but the workspace charter is explicit ("Persist major plans across four surfaces
with bidirectional links: ai_docs/ canonical + Obsidian vault mirror + stcortex … + CLAUDE.local.md
anchor. One surface survives = plan survives.")

Single-surface for one turn is fine in principle but has no fallback contract:
- If the current Claude session terminates between v1 author and v2 author, the orchestrator
  starts cold without injection.db / stcortex / vault anchors for v0.2.0. The cold-start
  sequence in `CLAUDE.local.md` (§ "Cold-start sequence for a fresh Claude window") still
  points at v0.1.0 / M0 + v0.1.1 + v0.2.0 prep stub — *not* at this v1.
- The stub at `WORKFLOW_TRACE_V020_PLAN_STUB_S1004115.md` is still the v0.2.0 entry point a
  new window will hit; it does not yet supersede to this v1.

**Why it matters.** "One surface survives = plan survives" assumes one of the four surfaces
is touched. Zero surfaces touched (single-surface ai_docs file with no other anchor) means a
context-window flip loses the v1 entirely and the orchestrator restarts from the stub.

**Evidence.** Plan v1 § 13 "single-surface (ai_docs only)"; project `CLAUDE.local.md` cold-start
sequence still points at v0.1.0/v0.1.1 anchors; CLAUDE.md workspace § "Working Mode" four-surface
discipline.

**Recommendation.** Either (a) treat v1 as 1.5-surface — add a one-line pointer in project
`CLAUDE.local.md` immediately after authoring ("v0.2.0 PLAN v1 DRAFT — see
[`ai_docs/WORKFLOW_TRACE_V020_PLAN_S1004377.md`](ai_docs/WORKFLOW_TRACE_V020_PLAN_S1004377.md);
v2 follows"), or (b) state explicitly in § 13 that if the session terminates, the new window
should re-read the stub and re-author v1 from scratch using this gap analysis + NA pass + the
stub as inputs. Option (a) is ~1 minute of work.

---

## C-13 — C5 (`wf-dispatch --execute`) is disposed as "acceptance gate, not work-item"; the disposition is *correct* but the **acceptance criteria are absent**. **Severity: LOW**

**Gap.** § 2.5 C5 row says: "Post-M0 dispatch soak. Cannot be agent-closed — Luke @ terminal
runs Conductor bring-up (B3 / OP-1). v0.2.0 acceptance gate, not v0.2.0 work-item." That is
right — agent cannot start Conductor — but the disposition leaves open what *passing* the
acceptance gate looks like. Phase 12 step 10 references OP-3 (post-v0.2.0 substrate soak) and
DX-Soak (24h or longer), but neither defines what "verified" means for `--execute`:
- Does `--execute` pass if 1 dispatch round-trip lands cleanly?
- Does it pass only if N rounds across X substrate-as-actor primitives (V1 refusal-typed
  dispatch fail, V2 back-pressure throttle, V3 drift event, V5 trust accessor) all exercise
  cleanly?
- What's the failure semantic — if `--execute` finds a bug after v0.2.0 ships, does that
  retroactively un-ship v0.2.0, or land as v0.2.1?

**Why it matters.** Plan v2 absorbed this risk implicitly (v0.1.0 shipped with `--execute`
unverified-live, and that was named honestly in CHANGELOG honest residuals). v0.2.0 *cannot*
do the same honestly because the entire substrate-as-actor surface is what `--execute` would
exercise. An acceptance gate without acceptance criteria is just a flag.

**Evidence.** Plan v1 § 2.5 C5 row; Phase 12 step 10 OP-3; § 15 DX-Soak; CHANGELOG `[v0.1.0]`
§ "Honest residuals — v0.2.0 candidates".

**Recommendation.** Add to Phase 12 step 10 a 3-row acceptance-criteria table:
| Surface | Pass criterion | Failure handling |
|---------|----------------|------------------|
| `--execute` round-trip | N=10 dispatches across the 5 substrates; each lands with substrate-confirmable receipt | failure = v0.2.0 un-ship, address in v0.2.1 |
| V1-V5 primitive exercise | each primitive fires at least once during soak | partial = v0.2.0 ships, named in Honest Residuals |
| Soak duration | DX-Soak hours of stable dispatch | shorter = explicit Luke ratify |

Without this, OP-3 is a deferred decision masquerading as an acceptance test.

---

## Strengths (noted after the gaps, per brief)

1. **Honest about the substrate frame in § 8.** "v0.2.0 is honestly half-substrate — the
   engine half" is the right named cut. The Plan v2 NA pass had to fight to get its equivalent
   sentence; Plan v1 has it in the draft.
2. **DX-V3 / DX-V5 correctly identified as no-default load-bearing.** These are the right
   Round A questions; the substrate-frame partition lives in their answers.
3. **Tier sequencing has a real reason.** DAW-1 "Tier-2-first" is engineering-defensible (wire
   contracts un-block verifiers cleanly; Tier 1 then consumes typed seams) — even if the
   substrate-frame author would invert it, the rationale is on the page.
4. **Phase 3 A2 + C1 staging is genuinely decision-free and parallelisable.** SD9 FeatureVector
   newtype is a one-line type change; m13 drain skeleton without consumer is genuinely safe to
   stage. This is a real scheduling win.
5. **§ 11 post-v0.2.0 partition is principled.** Each post-v0.2.0 item is *correctly* identified
   as substrate-side primitive the engine cannot author alone — exactly the NA-7 discipline.
6. **Risk register row "V3 self-canary problem"** anticipates the canary-of-canary regress and
   gives a real Frame-B observable ("absence of canary events is itself a substrate-frame
   observable") — even though C-9 above flags the alert-fatigue dimension still uncovered.

---

## Severity roll-up

| ID | Gap | Severity |
|----|-----|----------|
| C-1 | RefusalReason file:line wrong (`:163` vs actual `:228`); propagated from Phase 2 audit | LOW |
| C-2 | V1 RefusalToken structurally coupled to W1/SD11 wire bump; sequenced 3 phases apart | **HIGH** |
| C-3 | DX-W conflates 3 decisions (audit-overlay retire / seam choice / SemVer-compat) | **HIGH** |
| C-4 | Phase 9 own-module V3 hides Zen G7 re-audit multi-day external dependency | **HIGH** |
| C-5 | Phase 5 W3 `mutation-weight` source unverified — may force new primitive | **HIGH** |
| C-6 | Phase 6 R3 conflict semantic depends on A4 SD11 shape that ships in Phase 12 | MED |
| C-7 | Effort roll-up ~22-36 days under-counts Plan-v2 round-trip / interview / mutation wall time | MED |
| C-8 | ADR D-S1002127-03 amendment is language change w/ doc cascade; budgeted as edit | MED |
| C-9 | Risk register misses V5 schema-drift / V3 alert-fatigue / mutation wall-time blowout | MED |
| C-10 | Round B has 3 questions that are already-defaulted; Round A missing 3 real decisions | MED |
| C-11 | Part B § 7-8 annotates more than re-authors; spine preserved without Frame-B test | MED |
| C-12 | § 13 single-surface has no mechanical recovery contract; v1 lost on context flip | MED |
| C-13 | C5 `wf-dispatch --execute` acceptance gate has no defined pass criteria | LOW |

**4 HIGH, 7 MED, 2 LOW.** The plan is executable but should not start Phase 1 until C-2,
C-3, C-4, and C-5 are resolved — each one mis-sizes or mis-directs the Tier-1 / Tier-2
critical path. C-7 / C-9 / C-10 are scope-hygiene fixes that should fold in via v1 → v2
re-author; C-11 should trigger a genuine Part B re-author pass (or an honest § 9 amendment
saying the spine is preserved by engineering call, not by substrate-frame derivation);
C-12 is a 1-minute fix.

> **Back to:** [`WORKFLOW_TRACE_V020_PLAN_S1004377.md`](WORKFLOW_TRACE_V020_PLAN_S1004377.md) · [`WORKFLOW_TRACE_COMPLETION_PLAN_S1004115_CONVENTIONAL_GAP_ANALYSIS.md`](WORKFLOW_TRACE_COMPLETION_PLAN_S1004115_CONVENTIONAL_GAP_ANALYSIS.md) · [`PHASE2_AUDIT_S1004115.md`](PHASE2_AUDIT_S1004115.md) · [`PHASE9_SD_RECONCILIATION_S1004115.md`](PHASE9_SD_RECONCILIATION_S1004115.md) · [`decisions/2026-05-17-substrate-as-actor-deferrals.md`](decisions/2026-05-17-substrate-as-actor-deferrals.md) · [CLAUDE.local.md](../CLAUDE.local.md) · [CLAUDE.md](../CLAUDE.md)

*Conventional gap analysis · 2026-05-23 (S1004377) · Claude @ cortex · sibling to NA-gap-analyst pass · 13 findings (4 HIGH / 7 MED / 2 LOW) · to be folded into Plan v2 next turn alongside the NA pass per Plan v1 § 12.*
