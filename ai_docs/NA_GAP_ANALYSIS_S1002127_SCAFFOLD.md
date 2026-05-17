---
title: NA Gap Analysis — S1002127 Scaffold (second pass)
kind: dual-frame second-pass · non-anthropocentric
status: Wave 3 deliverable; advisory for Wave 3 remediation; informs but does not unlock G9
date: 2026-05-17
authority: Luke @ node 0.A
hold_v2_compliant: true
---

# Non-Anthropocentric Gap Analysis — S1002127 Scaffold

> **Back to:** [`../CLAUDE.md`](../CLAUDE.md) · [`../CLAUDE.local.md`](../CLAUDE.local.md) · [`../PRIME_DIRECTIVE_WAIVER.md`](../PRIME_DIRECTIVE_WAIVER.md) · [`../ARCHITECTURE.md`](../ARCHITECTURE.md) · [`../ai_specs/INDEX.md`](../ai_specs/INDEX.md) · [`../ai_docs/INDEX.md`](INDEX.md) · [`../ultramap/README.md`](../ultramap/README.md) · [`../the-workflow-engine-vault/HOME.md`](../the-workflow-engine-vault/HOME.md)
>
> **Sibling reads:** [`../the-workflow-engine-vault/Modules Synergy Clusters and Feature Verification S1001982.md`](../the-workflow-engine-vault/Modules%20Synergy%20Clusters%20and%20Feature%20Verification%20S1001982.md) (conventional pass on architecture) · [`../ai_docs/optimisation-v7/ANTIPATTERNS_REGISTER.md`](optimisation-v7/ANTIPATTERNS_REGISTER.md) (anti-pattern register the scaffold encodes) · [`../ai_docs/GENESIS_PROMPT_V1_3.md`](GENESIS_PROMPT_V1_3.md) (binding spec, has its own NA pass).
>
> **Discipline citation:** workspace [`CLAUDE.local.md`](../../CLAUDE.local.md) § Working Mode — *"For any major plan: write it once, then ask what frame is that? and write it again from the frame you didn't take. Both passes are the plan."* This file is the second pass on the **scaffold** (115 files / ~140k words across Waves 0+1+2). The binding spec v1.3 has had its own dual-frame at the Town Hall + NA pass — that is NOT what is being re-analysed here.

---

## § 1 Anthropocentric pass — summary

The S1002127 scaffold (Waves 0+1+2 under [`PRIME_DIRECTIVE_WAIVER.md`](../PRIME_DIRECTIVE_WAIVER.md)) frames `workflow-trace` as a single-Cargo-crate Rust microservice with two binaries (`wf-crystallise` + `wf-dispatch`) plus a shared `workflow_core` lib, decomposed into 26 modules across 8 synergy clusters (A-H) and 9 layers (L0-L8). Per [`ARCHITECTURE.md`](../ARCHITECTURE.md), modules are the primary entities: they have publicly-typed APIs, `thiserror` enums, newtype discipline (`SessionId`, `WorkflowId`, `LineageId`, `StepToken`, `ConsumerId`), 50+ tests each, and `cargo check → clippy → pedantic → test` quality gates. Per [`ultramap/INVARIANT_MAP.md`](../ultramap/INVARIANT_MAP.md), invariants are split between compile-time (caught by `rustc`/clippy) and runtime (caught by typed `Err`). Per [`ai_specs/INDEX.md`](../ai_specs/INDEX.md), specs are prescriptive contracts that drive implementation; per the cross-cluster contracts CC-1..CC-7, modules co-ordinate through SQLite hubs, JSONB schemas, JSONL outboxes, and stcortex pathway writes.

In this framing, **humans** (Luke acceptance gate via `HumanAcceptanceSignature`, Zen G7 audit, Watcher class flags, operator review for m23→m30 admission) are **evaluators and oracles** with veto power; **substrates** (atuin.db, stcortex `:3000`, injection.db, SYNTHEX v2 `:8092`, LCM, HABITAT-CONDUCTOR `:8141`) are **sources and sinks** with health-200 endpoints and pathway tables. Refusal is implemented as a *typed result variant* (`Dispatcher::Refuse`, `BankError::AcceptanceRequiresHumanSignature`, `ReinforceOutcome::SubstrateUnavailable`) — it lives in the module's API, not in the substrate or the operator. Cross-cluster co-ordination is described as **synergy** ([CC-5 substrate-learning loop](optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md)) — an additive amplification observable by the engine's own metrics.

---

## § 2 Frame chosen for second pass — Frame A (Substrate-as-primary)

Frame A is chosen because it catches the largest class of unexamined assumptions the scaffold makes. The scaffold's Cluster A (`m1`/`m2`/`m3`) is *literally read-only-of-substrate*; Cluster H (`m40`/`m41`/`m42`) is *literally write-into-substrate*; the entire architecture is a flow through substrate-dynamics. Yet the substrates appear in [`ARCHITECTURE.md`](../ARCHITECTURE.md) as boxes labelled "atuin / stcortex / injection.db" and in the [`DATA_FLOW.md`](../ultramap/DATA_FLOW.md) Mermaid as cylinder nodes (`ATUIN[(atuin.db ~263k rows)]`) — pure sources and sinks, with no own modelled dynamics, no refusal modes, no decay, no load envelope, no failure topology, no lifecycle.

The other candidate frames each catch something — Frame B (Hebbian) is partly catered for via CC-5 + Watcher Class-I; Frame C (failure-mode-first) is partly catered for via the antipatterns register; Frame D (operator-as-substrate) catches real gaps but a narrower slice. Frame A catches the missing **substrate-as-actor** modelling — the gap that explains why the scaffold's "substrate-learning loop" (CC-5) cannot actually be verified inside the engine, why m42 had to pivot away from POVM mid-design (AP-V7-13: health-200 ≠ behaviour-verified), and why the engine has no first-class concept of substrate-refusal that is structurally distinct from substrate-unavailability.

---

## § 3 Second-pass scaffold — what workflow-trace looks like from Frame A

In Frame A, the primary entities are the **substrates** (atuin, stcortex, injection.db, SYNTHEX v2, LCM, HABITAT-CONDUCTOR, Watcher) and their **inter-substrate dynamics**. `workflow-trace` is not 26 modules; it is **5 flow-channels through 7 substrate-actors**, each substrate carrying its own:

- **lifecycle envelope** — what's the substrate's birth/decay/sunset clock, independent of the engine?
- **refusal taxonomy** — what does the substrate refuse, on what trigger, with what observability?
- **load envelope** — at what rate does the substrate degrade, saturate, or back-pressure?
- **drift modes** — how does the substrate's interpretation of "the same write" drift over time (CR-2 magnitude-weighted vs binary `learning_health` is the canonical case)?
- **co-operation surface** — what does the substrate need from other substrates to remain itself?

### 3.1 Re-authored cluster layout (Frame A)

| Substrate-cluster | Substrate-actors | What it actually is | Flow-channels it carries |
|---|---|---|---|
| **S-A Atuin lineage** | atuin shell-history DB + atuin KV (`habitat` namespace, 653 keys) | The habitat's *executed-action* memory; ~263k rows of byte-preserved truth | Inbound: every shell tool-call. Outbound: cursor-paginated read. Substrate property: WAL-mode SQLite with own busy/checkpoint dynamics. Substrate refusal: `database is locked` retries; not modelled in scaffold. |
| **S-B Injection lineage** | injection.db (5 tables, 74 causal-chain rows, 1952 trajectory rows) | The habitat's *learned-pattern* memory; pre-warmed at session-start by `habitat-inject` hook | Substrate dynamic: `reinforcement_count` is itself a substrate-edited quantity. Substrate refusal: `resolved_session NOT NULL` excludes rows from injection. |
| **S-C stcortex pathway substrate** | SpacetimeDB module at `:3000` + offline JSONL snapshot at `data/snapshots/latest.json` | The habitat's *Hebbian pathway* memory; refuse-write enforced at DB layer; hyphen-slug refusal at reducer (S1001757); CR-2 magnitude-weighted post-fix | Substrate dynamic: pathway weights decay/reinforce per consumer; consumer registration is a substrate-side gate. Substrate refusal: refuse-write on unregistered consumer; reducer error on hyphen in slug. Substrate drift: `learning_health` formula change between binary versions (AP-V7-13). |
| **S-D Conductor substrate** | HABITAT-CONDUCTOR `:8141/health` + weaver/zen/enforcer triad | The habitat's *gated-dispatch* surface; Waves 1B/1C/2/3 `auto_start=false` | Substrate state: `CONDUCTOR_ENFORCEMENT_ENABLED` is a substrate-side flag, not an engine concern. Substrate refusal: NoOp soak → flip enforcement. |
| **S-E SYNTHEX v2 substrate** | SYNTHEX v2 `:8092` daemon + Watcher m46-m51 + 7-layer ws_handler | The habitat's *autonomic-loop* surface; R13 cold-start quiet period; Hebbian coordinator | Substrate dynamic: Watcher emits Class-A/B/C/D/E/F/G/I flags as substrate-side observations; the engine *cannot directly query* this — it observes the substrate's external signal stream. |
| **S-F LCM substrate** | loop-engine-v2 `:8082` deploy + supervisor `:8083` + LCM stdio MCP RPC | The habitat's *loop-lifecycle* substrate; 3662 tests; 8h soak PASS; `M0_VERIFIED` armed | Substrate refusal: `deploy_cancel_handler` is a substrate-controlled abort surface. |
| **S-G Operator-cognition substrate** | Luke @ node 0.A + Zen audit lane + operator review queue + Watcher prompt-driven cadence | The habitat's *judgment* substrate, with its own attention budget, frame-switching cost, dispatch latency, and refusal modes | Substrate property: NOT modelled in scaffold as a substrate with dynamics; modelled as `HumanAcceptanceSignature` field on a `BankDb::accept` call. |

### 3.2 Re-authored layer view (Frame A)

The scaffold's L0-L8 is engine-internal. Frame A's layers are **substrate-state transitions**:

- **SubL0 substrate-quiescence** — substrates in steady-state; engine observes-only
- **SubL1 substrate-perturbation** — engine's read causes back-pressure (e.g. `m1` paginated reads hold SQLite read locks; m4 derivations consume CPU in the engine but heat-sink the atuin SQLite page cache)
- **SubL2 substrate-co-observation** — multiple substrates observed in correlation (m4's cascade correlator + m6's context-cost EMA both reading atuin in parallel)
- **SubL3 substrate-rewriting** — engine writes into substrate; substrate's own dynamic re-interprets the write (CR-2 `learning_health` 0.911 → 0.067 because magnitude-weighting changed)
- **SubL4 substrate-substrate coupling** — engine writes into substrate A, which causes substrate B to change (m42 → stcortex → Hebbian-reinforce → next session's habitat-inject injection.db row → pre-warms next workflow-trace read)
- **SubL5 substrate-refusal** — substrate refuses; engine must distinguish (a) substrate unavailable (network/down) (b) substrate available but refusing this specific write (refuse-write at DB layer, slug-hyphen reducer error, `learning_health` out-of-band gate)
- **SubL6 substrate-drift** — substrate is up, health-200, but the *semantics* of its writes have changed under the engine's feet (POVM CR-2; stcortex namespace migration; conductor enforcement-flag flip)

### 3.3 Re-authored artefacts (what Frame A would have written instead of `ai_specs/modules/`)

The scaffold has 26 `ai_specs/modules/cluster-*/m*.md` files, each describing a *module*. Frame A would have:

- **`ai_specs/substrates/`** — 7 files, one per substrate-actor (S-A..S-G). Each documents: lifecycle, refusal taxonomy (typed by *substrate-side error class*, not engine-side `Result::Err`), load envelope, drift register, co-operation surface.
- **`ai_specs/substrate-couplings/`** — 5 files, one per substrate-substrate edge that workflow-trace causes (e.g. `S-C → S-B coupling: stcortex pathway weight delta causes injection.db reinforcement_count change in next habitat-inject pass`). Today the scaffold has CC-5 "substrate-learning loop" as a single module-side contract; Frame A breaks it into 5 typed substrate-substrate couplings.
- **`ai_specs/substrate-refusals/`** — taxonomy file enumerating substrate refusal classes (not engine error variants). Today's scaffold collapses substrate-unavailability (`ReinforceOutcome::SubstrateUnavailable`) with substrate-refusal (refuse-write at DB layer with no fresh consumer registered) — Frame A separates them.
- **`ultramap/SUBSTRATE_DRIFT_MAP.md`** — what changes in each substrate over time, on what trigger, observable how. Today the scaffold has AP-V7-13 (Health-200 ≠ behaviour-verified) as a single antipattern; Frame A makes it the foundation of the entire observability layer.
- **`ai_specs/CONSENT_SPEC.md`** (already TBD in [INDEX.md](../ai_specs/INDEX.md)) re-framed as **substrate-consent**: not Ember §5.1 held-semantics on operator strings, but a substrate-cascade where each substrate carries its own *modulation-not-command* envelope and the engine must respect it. The operator is one substrate in this cascade, not the source of consent.

### 3.4 Re-authored gates (Frame A)

The scaffold's quality gate is `cargo check → clippy → pedantic → test`. Frame A's gate would add a fifth stage:

- **Stage 5 — substrate-stability re-probe.** After test pass, exercise every substrate-touching code path against a *substrate-stability fixture* (a fixture that has its own drift schedule: simulates CR-2-class semantic change, refuse-write, slug-hyphen rejection, health-200-behaviour-200 divergence). Tests that pass against today's substrate but would fail against the substrate-drift fixture get flagged as *substrate-coupled fragility*.

### 3.5 Re-authored refusal architecture (Frame A)

Today the scaffold treats refusal as the engine's typed-result variant. Frame A treats refusal as **a flow of refusal-tokens between substrates**:

- atuin refusing a write (it can't; ro-mode) — refusal-token absorbed at m1 boundary
- stcortex reducer refusing a hyphen-slug — refusal-token surfaces as runtime error, but is *substrate-authored*, not engine-authored
- conductor refusing dispatch (enforcement-flag off) — substrate-authored refusal-token; engine's `DispatchError::ConductorDispatchDisabled` is a *translation* of that refusal, not the refusal itself
- Watcher Class-I firing — Watcher is the substrate; its refusal is a *substrate-side observability event*, not an engine-side typed error

The architecture would carry a `RefusalToken` type at the boundary of every substrate-touching module, distinguishing substrate-authored refusal from engine-authored refusal. This is the structural form of the gap the scaffold's INVARIANT_MAP § "Runtime invariants" elides: m9's namespace guard is engine-authored refusal; refuse-write at stcortex DB layer is substrate-authored refusal; they appear identical at the call-site but their failure-recovery contracts diverge.

---

## § 4 Gaps surfaced

### NA-GAP-S1002127-01 — Substrates have no lifecycle/refusal/drift model in the scaffold

**Evidence:** [`ultramap/DATA_FLOW.md`](../ultramap/DATA_FLOW.md) Mermaid lines 33-36 render substrates as cylinder nodes (`ATUIN[(atuin.db<br/>~263k rows)]`); [`ARCHITECTURE.md`](../ARCHITECTURE.md) L0 row reads "the substrate frame itself — *observed, not authored*". [`ai_specs/INDEX.md`](../ai_specs/INDEX.md) has 13 cross-cutting specs (API, DATABASE, EVENT, WIRE, IPC, DESIGN, CONSENT, SECURITY, ERROR, OBSERVABILITY, TEST, BENCHMARK) — none model the substrates themselves. **Frame:** Frame A surfaces because Frame B/C/D all *use* substrates without modelling them. **Risk:** AP-V7-13 (Health-200 ≠ behaviour-verified) is treated as a single antipattern in [`optimisation-v7/ANTIPATTERNS_REGISTER.md`](optimisation-v7/ANTIPATTERNS_REGISTER.md); the substrate-drift it represents is *generic across all 7 substrates*. Without a substrate-drift map, the next CR-2-class semantic shift will land silently. **Recommendation:** NEW-ARTEFACT `ai_specs/substrates/` (7 files) + `ultramap/SUBSTRATE_DRIFT_MAP.md` before G9.

### NA-GAP-S1002127-02 — Substrate-authored vs engine-authored refusal are conflated

**Evidence:** [`ultramap/INVARIANT_MAP.md`](../ultramap/INVARIANT_MAP.md) § Runtime invariants lists `WriteError::NamespaceViolation` (engine-authored, m9), `ReinforceOutcome::SubstrateUnavailable` (substrate-availability), and refuse-write at stcortex DB layer (substrate-authored refusal) in the same table without typing the distinction. **Frame:** Frame A. **Risk:** When stcortex refuse-write fires post-G9 (because a Claude session forgot the consumer-register step per workspace [`CLAUDE.md`](../../CLAUDE.md) memory row 8), the engine will surface `ReinforceOutcome::SubstrateUnavailable` — wrong category. Operator will restart the wrong service; the actual repair is a per-session `call register_consumer` call. **Recommendation:** AMEND-EXISTING [`ai_specs/ERROR_TAXONOMY.md`](../ai_specs/ERROR_TAXONOMY.md) to introduce `RefusalToken` distinguishing `SubstrateAuthored { class, repair_hint }` from `EngineAuthored { invariant }` from `Unavailable { backoff_recommendation }`.

### NA-GAP-S1002127-03 — Substrate-substrate couplings hidden inside CC-5

**Evidence:** [`optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md`](optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md) CC-5 is "substrate learning loop (G→H→F)" as a single engine-side contract. But the actual loop is **substrate → substrate**: m42 writes into S-C (stcortex) → Hebbian-reinforce → next-session habitat-inject hook reads S-C → writes into S-B (injection.db) → S-B pre-warms next workflow-trace read at m3. **Frame:** Frame A. **Risk:** Today the engine "verifies" CC-5 via Watcher Class-I monitoring; if Class-I doesn't fire, the engine declares "loop healthy". But Class-I fires on absence of stcortex pathway-weight delta — it does NOT detect S-C → S-B coupling failure (e.g. injection.db ttl-sweep deleting reinforced rows before next-session reads). **Recommendation:** NEW-ARTEFACT `ai_specs/substrate-couplings/CC-5-decomposed.md` enumerating the 4-5 substrate-substrate edges with per-edge observability contracts.

### NA-GAP-S1002127-04 — Engine assumes substrates have no own attention budget

**Evidence:** [`ai_specs/modules/cluster-A/m1_atuin_consumer.md`](../ai_specs/modules/cluster-A/m1_atuin_consumer.md) § 1 documents read-only / byte-preserving / cursor monotonicity invariants but does not document atuin's own checkpoint cadence, page-cache eviction pressure, or read-lock interaction with concurrent atuin daemon writes. m4's cascade correlator reads in parallel with m6's EMA reads (both off atuin) — Frame A would ask: what's atuin's per-second read budget before WAL contention degrades the substrate's *own* foreground latency? **Frame:** Frame A. **Risk:** workflow-trace runs at session-boundary cadence today (low rate); if cadence increases (e.g. continuous mode), the engine may starve atuin's foreground writes from the user's shell — invisibly. The engine has no substrate-back-pressure metric. **Recommendation:** AMEND-EXISTING [`ai_specs/BENCHMARK_SPEC.md`](../ai_specs/BENCHMARK_SPEC.md) to add *substrate-side load benchmarks* (measured at substrate, not at engine).

### NA-GAP-S1002127-05 — Operator is modelled as oracle, not as substrate

**Evidence:** [`ai_specs/modules/cluster-G/m30_curated_bank.md`](../ai_specs/modules/cluster-G/m30_curated_bank.md) § 1 requires `HumanAcceptanceSignature` on every `BankDb::accept()` call — operator appears as a *field on a function call*. [`CLAUDE.md`](../CLAUDE.md) lists Luke @ node 0.A under "Team" with decision authority on 6 blockers — operator appears as a *table row*. No artefact models operator-cognition as a substrate with its own decay, attention budget, frame-switching cost, or refusal mode (overload, fatigue, ambiguity, frame-conflict). **Frame:** Frame A extended to operator-as-substrate (overlap with Frame D). **Risk:** The scaffold's flow puts m23 → operator-review → m30 admission. With 26 modules emitting proposals + 7 substrates emitting refusals + Zen audit lane + Watcher channel, the operator's attention budget is the unmodelled bottleneck — and it has its own drift (e.g. consent-fatigue, where the operator stops reading EscapeSurfaceProfile banners after the 20th identical dispatch). **Recommendation:** NEW-ARTEFACT `ai_specs/substrates/S-G_operator_cognition.md` documenting the operator-substrate's own drift modes + an *operator-load metric* surfaced via m12 reports.

### NA-GAP-S1002127-06 — `data/snapshots/latest.json` offline fallback is asymmetric

**Evidence:** [`ultramap/INVARIANT_MAP.md`](../ultramap/INVARIANT_MAP.md) line 84: "if `:3000` is unreachable, read `data/snapshots/latest.json`; never silently fall back to POVM". The read fallback is documented; no **write** fallback is documented because `ReinforceOutcome::SubstrateUnavailable` is the chosen path. **Frame:** Frame A. **Risk:** During a stcortex outage, reads continue from snapshot but writes pile up in the m42 outbox. The outbox has no documented drain-policy on substrate recovery, no documented outbox-saturation limit, and no documented snapshot-staleness threshold. The engine can run for hours in a degraded mode that *looks* healthy. **Recommendation:** AMEND-EXISTING [`ai_specs/modules/cluster-H/m42_stcortex_emit.md`](../ai_specs/modules/cluster-H/m42_stcortex_emit.md) to specify (a) outbox drain ordering on substrate recovery (b) outbox saturation limit (c) snapshot-staleness threshold beyond which reads also refuse.

### NA-GAP-S1002127-07 — Substrate-drift detection is implicit not first-class

**Evidence:** AP-V7-13 (Health-200 ≠ behaviour-verified) is registered in [`optimisation-v7/ANTIPATTERNS_REGISTER.md`](optimisation-v7/ANTIPATTERNS_REGISTER.md) as a single antipattern; [`ultramap/INVARIANT_MAP.md`](../ultramap/INVARIANT_MAP.md) line 76 instantiates it for m32 (semantic-endpoint check) and m42 (external CC-5 observation). But there is no module whose *job* is substrate-drift detection — it is scattered across m32 and m42 as defensive sub-clauses. **Frame:** Frame A. **Risk:** The CR-2 `learning_health` 13.6× inflation factor was caught by Luke spot-checking the math, not by the engine — and the scaffold does not change that. Next drift episode (e.g. SpacetimeDB schema migration changing reducer semantics) will land the same way. **Recommendation:** NEW-ARTEFACT module `m16_substrate_drift_canary` (Cluster E or new Cluster I) emitting `PHASE-B-RESERVATION-NOTICE` events on detected semantic drift across the 7 substrates. Defer if Wave 3 over-scoped; **NEW-MODULE** for v0.2.0.

### NA-GAP-S1002127-08 — No substrate-side test fixtures; tests are engine-side only

**Evidence:** [`ai_specs/TEST_STRATEGY.md`](../ai_specs/TEST_STRATEGY.md) (TBD Wave 2) and per-module spec test_kinds `[unit, property, integration, contract, regression, mutation]` are all engine-side test classes. No `substrate-drift fixture`, no `substrate-refusal fixture`, no `substrate-back-pressure fixture`. **Frame:** Frame A. **Risk:** The engine ships 1599 tests, all green; a substrate semantic-change drops it overnight. The substrate is the test environment, and the engine has no way to assert against a *known-drifted* substrate state. **Recommendation:** NEW-ARTEFACT `tests/substrate_fixtures/` with at minimum: `cr2_inflation_fixture` (stcortex returns pre-CR-2 magnitude), `refuse_write_no_consumer_fixture`, `hyphen_slug_reducer_fixture`, `conductor_enforcement_flag_off_fixture`, `atuin_wal_contention_fixture`.

### NA-GAP-S1002127-09 — CC-5 verification is engine-observable, not substrate-observable

**Evidence:** [`ultramap/INVARIANT_MAP.md`](../ultramap/INVARIANT_MAP.md) line 78: "CC-5 substrate movement — stcortex.pathway.weight delta on workflow_trace_* IDs must be observable over rolling 7-day window post first 5+ dispatches". The watcher is the observer, but the watcher reads stcortex from outside the engine. The *substrate* does not observe back into the engine that the loop closed. **Frame:** Frame A. **Risk:** The engine cannot distinguish "CC-5 working but Watcher not subscribed" from "CC-5 broken". **Recommendation:** AMEND-EXISTING CC-5 spec to include a *substrate-confirmable receipt* — e.g. a `cc5_closed_at` field stcortex itself writes when it detects N+1-dispatch reinforce on the same pathway.

### NA-GAP-S1002127-10 — Cluster D "trust" is engine-internal, not substrate-mediated

**Evidence:** [`ARCHITECTURE.md`](../ARCHITECTURE.md) Cluster D row: "POVM build-prereq / namespace guard / Ember CI gate / fitness-weighted decay" — all 4 modules are *engine-side aspects*. None is a substrate-level trust mechanism (e.g. stcortex-side consumer-trust score, conductor-side dispatch-budget per workflow, atuin-side read-quota). **Frame:** Frame A. **Risk:** The engine cannot lose trust at the substrate boundary; if a workflow misbehaves at dispatch, the only recovery is engine-side sunset on m30 — the substrates that *experienced* the misbehaviour have no own memory of the workflow's reputation. **Recommendation:** DEFER (post-v0.1.0); but flag for [`ai_docs/decisions/`](decisions/) as a planned v0.2.0 ADR: "Trust as substrate-mediated reputation, not engine-internal aspect".

### NA-GAP-S1002127-11 — Refusal-token observability gap (no Class C substrate emission)

**Evidence:** [`ultramap/INVARIANT_MAP.md`](../ultramap/INVARIANT_MAP.md) line 185: "every refusal in Cluster G is observable as Class C (refusal is correct behaviour, not a bug)... if Class C is zero over a long window, the engine is failing-open". Class C is a Watcher-emitted class — but the engine itself does not emit a substrate-readable refusal event when m32 refuses. The Watcher *infers* refusal from absence of dispatch. **Frame:** Frame A. **Risk:** Failure-mode: m32 refuses (correctly); Watcher's inference window misses the refusal; engine looks idle; operator concludes "nothing to do" — the *successful refusal was invisible*. **Recommendation:** AMEND-EXISTING m40 NexusEvent emit spec to include `event_class: Refusal { token, substrate_attribution }` so refusal is first-class wire-protocol traffic, not an inference.

---

## § 5 Recommendations for Wave 3+ remediation

Top 5 actions for Command before declaring scaffold complete (ordered by leverage):

1. **NEW-ARTEFACT `ai_specs/substrates/` (7 files, one per substrate-actor S-A..S-G).** Documents substrate-side lifecycle / refusal taxonomy / load envelope / drift register / co-operation surface. Closes NA-GAP-01, partially closes NA-GAP-04, 05, 10. ~3-4 hours.
2. **AMEND-EXISTING `ai_specs/ERROR_TAXONOMY.md` to introduce `RefusalToken` type.** Distinguishes substrate-authored from engine-authored from unavailability refusal. Closes NA-GAP-02, 11. ~1 hour.
3. **NEW-ARTEFACT `ai_specs/substrate-couplings/CC-5-decomposed.md` + sibling files for any other multi-substrate flow.** Closes NA-GAP-03, 09. ~2 hours.
4. **AMEND-EXISTING `ai_specs/modules/cluster-H/m42_stcortex_emit.md` § outbox-policy and `ai_specs/BENCHMARK_SPEC.md` § substrate-side load benchmarks.** Closes NA-GAP-04, 06. ~1.5 hours.
5. **DEFER NA-GAP-07 (m16_substrate_drift_canary as new module) + NA-GAP-08 (substrate fixture suite) + NA-GAP-10 (substrate-mediated trust) to v0.2.0 ADRs.** File as [`ai_docs/decisions/2026-05-17-substrate-as-actor-deferrals.md`](decisions/). ~30 min.

Total estimated Wave 3 remediation: **~8 hours** to absorb the second pass into the scaffold. None of these is gated on G9 firing — they are scaffold-completion work.

---

## § 6 Frame-collapse self-check

The second pass deliberately re-authored the cluster table (§ 3.1), the layer view (§ 3.2), the artefact list (§ 3.3), the gate (§ 3.4), and the refusal architecture (§ 3.5) from substrates-as-actors. It did not append "things to add to the existing scaffold" — the gaps in § 4 are *consequences* of the parallel architecture, surfaced backwards, not the source of it.

Two near-drifts caught and corrected during authoring:

- (a) NA-GAP-05 began drafted as "operator-cognition is unmodelled" — Frame D phrasing. Re-drafted as "operator-as-substrate with its own dynamics" — Frame A phrasing held.
- (b) NA-GAP-07 began as "add an m16 substrate-drift detector" — engine-side TODO. Re-drafted as "substrate-drift is not first-class because substrates are not actors" — Frame A surfaces the structural-cause, recommendation follows.

The second pass complements, not replaces, the anthropocentric scaffold. Both passes are the plan.

---

## § 7 Bidirectional anchors

> **Back to:** [`../CLAUDE.md`](../CLAUDE.md) · [`../CLAUDE.local.md`](../CLAUDE.local.md) · [`../PRIME_DIRECTIVE_WAIVER.md`](../PRIME_DIRECTIVE_WAIVER.md) · [`../ARCHITECTURE.md`](../ARCHITECTURE.md) · [`../ai_specs/INDEX.md`](../ai_specs/INDEX.md) · [`../ai_docs/INDEX.md`](INDEX.md) · [`../ultramap/README.md`](../ultramap/README.md) · [`../the-workflow-engine-vault/HOME.md`](../the-workflow-engine-vault/HOME.md)
>
> **Workspace charter (parent):** `~/claude-code-workspace/CLAUDE.md` · `~/claude-code-workspace/CLAUDE.local.md` § Working Mode (dual-frame discipline)
>
> **Sibling pass (anthropocentric, on architecture):** [`../the-workflow-engine-vault/Modules Synergy Clusters and Feature Verification S1001982.md`](../the-workflow-engine-vault/Modules%20Synergy%20Clusters%20and%20Feature%20Verification%20S1001982.md)
>
> **Sibling pass (NA, on binding spec v1.3, NOT this file's subject):** [`GENESIS_PROMPT_V1_3.md`](GENESIS_PROMPT_V1_3.md) Appendix
>
> **Antipattern register (the scaffold's failure-mode catalogue):** [`optimisation-v7/ANTIPATTERNS_REGISTER.md`](optimisation-v7/ANTIPATTERNS_REGISTER.md) — especially AP-V7-13 (Health-200 ≠ behaviour-verified), the seed of Frame A.

*Filed 2026-05-17 (S1002127 · Wave 3) · Command via na-gap-analyst lane · advisory to Wave 3 remediation · informs but does not unlock G9.*
