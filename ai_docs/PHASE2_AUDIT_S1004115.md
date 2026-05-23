# Phase 2 Audit — Wire-Contract + 8-NA-Gap Code Audit + m33 Verifier Input Catalog

> **Authored:** 2026-05-23 (S1004115, Plan v2 Phase 2)
> **Status:** decision-free deep FP-verification; feeds Phase 4 decision-cleanup, Phase 6 sizing
> **Per Plan v2 §3 Phase 2:** evidence-cited audit; subagent dispatched per D41 for the 8-NA-gap fan-out
> **Back to:** [`WORKFLOW_TRACE_COMPLETION_PLAN_V2_S1004115.md`](WORKFLOW_TRACE_COMPLETION_PLAN_V2_S1004115.md) · [CLAUDE.local.md](../CLAUDE.local.md)

---

## 1. Wire-contract verification (Plan v2 §3 Phase 2 step 1) — VERIFIED

End-to-end trace m23 → `wf-crystallise` → JSONL → `wf-dispatch` → m30, against HEAD `24cf6e1`:

| Stage | File:line | Shape |
|-------|-----------|-------|
| **m23 proposal type** | `src/m23_proposer/mod.rs:27` | `pub struct WorkflowProposal { proposal_id: u64, variant: WorkflowVariant, evidence_n: usize, evidence_lift: f64, evidence_ci_half: f64, diversity_cluster: Option<usize> }` — all fields private; F2 floor enforced in fallible `new()`; `#[derive(serde::Serialize, serde::Deserialize)]` |
| **Compose call site** | `src/orchestration/crystallise.rs:509` | `compose_proposals(&patterns, &snapshot, \|_v\| None)` — **confirms R2 open** (closure-lookup is unconditional `None`) |
| **JSONL write** | `src/orchestration/crystallise.rs:512` | `report.proposals_written = write_proposals_jsonl(&config.proposals_out, &proposals)?` |
| **JSONL read** | `src/orchestration/dispatch.rs:594–595` | `let proposal: WorkflowProposal = serde_json::from_str(trimmed).map_err(...)` — auto-deserialise via derive |
| **Accept** | `src/m30_bank/mod.rs:63` | `pub struct AcceptedWorkflow { workflow_id, proposal: WorkflowProposal, accepted_at_ms, sunset_at_ms, weight, last_run_ms, run_count }` — wraps the proposal as-is, no field added/dropped |
| **m33 input** | `src/orchestration/dispatch.rs:331` | `fn verify(&self, _workflow: &crate::m30_bank::AcceptedWorkflow) -> VerifierVerdict` — the leading `_` confirms R1 placeholder; verdict is `Approve` unconditional |
| **m32 dispatch** | `src/orchestration/dispatch.rs:518` | `dispatch_one(&dispatcher, &workflow, config.ack_ceiling, &signature, &mut report)` |

**Cost-field blast radius** (for D9 stub-vs-wire sizing — D9 already locked as **stub**, this sizes the path-not-taken):
The `WorkflowProposal` struct carries **no `cost` field**. Adding one to the wire would touch:
1. `WorkflowProposal` struct (add field) + `new()` signature + accessor — m23_proposer/mod.rs (~30 LOC)
2. `compose_proposals` populate (needs a cost input source — open question) — m23_proposer (~20 LOC)
3. JSONL serialisation: **auto** via `#[derive]` (zero LOC) — but every JSONL fixture in the test suite breaks until regenerated.
4. `dispatch.rs` reads: **auto** via `serde_json::from_str` (zero LOC); `AcceptedWorkflow` re-wraps the proposal so the cost is visible at `accepted.proposal().cost()` (zero LOC).
5. m33 Cost verifier logic — ~30–50 LOC (per § 15 D10 if wired: `step-count × mutation-weight` metric).
6. Tests across m23/m30/orchestration — ~50–100 LOC (round-trip, missing-field tolerance, F2-floor interaction).

**Total estimated blast radius: ~150–200 LOC** + JSONL-fixture regeneration churn. **Matches the plan §3 Phase 6c "~150–250 LOC ~2 days" estimate.** D9 stub decision is the correct choice for M0 quality-first/no-deadline (D3).

---

## 2. 8-NA-gap code audit (Plan v2 §3 Phase 2 step 2; gap NA-8 / convergent C-2)

**Subagent dispatched** per D41 (Phase 2 audits explicit subagent-allowance) — read-only fan-out across `src/`, evidence-cited findings.

The S1002127 project record claims **"8 of 11 NA gaps closed; 3 deferred to v0.2.0 via ADR D-S1002127-03"**. The audit verdict against shipped `src/`:

| # | NA Gap | Verdict | Evidence |
|---|--------|---------|----------|
| 1 | NA-GAP-01 — `RefusalToken` as Rust type | **❌ spec-only** (zero hits in `src/`) | Taxonomy lives in `ai_specs/ERROR_TAXONOMY.md:257–272` + `ai_specs/cross-cutting/refusal-taxonomy.md` only. In code: only `RefusalReason` (closed enum at `src/m32_dispatcher/mod.rs:163`), flat — no substrate-/engine-/operator-authored typing. |
| 2 | NA-GAP-02 — m11 substrate-clock-crossing tracking | **✅ code-backed** | `DecayError::ClockUnavailable` at `src/m11_fitness_weighted_decay/error.rs:39`; future-dated `last_run_ms` skip at `consolidation.rs:230`; `workflows_clock_skew_skipped` field on `SunsetStats` (`sunset.rs:96`); 3 test arms at `consolidation.rs:486,750,782`. |
| 3 | NA-GAP-03 — m4 opaque-ID discipline | **✅ code-backed** | `src/m4_cascade/cluster_id.rs:1–136` — `CascadeClusterId` wraps an FNV-1a hash, downstream-consumer "treat as opaque (string form only)" enforced in test at `:136`. |
| 4 | NA-GAP-04 — substrate back-pressure budget | **❌ spec-only** (zero hits in `src/`) | `rg "back_pressure\|backpressure\|BackPressure\|throttle\|rate_limit\|RateLimit"` returns zero. No substrate-side back-pressure mechanism in shipped code. |
| 5 | NA-GAP-05 — m9 namespace slug discipline | **✅ code-backed** | `src/m9_watcher_namespace_guard/validator.rs:131,158–167` — `munge_hyphen_slug` (idempotent `'-'→'_'`); prefix-match against `WORKFLOW_TRACE_NS_PREFIX`. Comment at `:46` cites "AP-Hab-11 / S1001757 mitigation". Matches stcortex reducer-side rule. |
| 6 | NA-GAP-06 — m42 outbox drain-policy | **⚠ partial** | **Write half done:** `m13_stcortex_writer/mod.rs:307–369` carries `outbox_path` + lock; m42 tests `:472–955` confirm outbox-first defer + append-only durability. **Drain half absent:** `rg "drain\|drain_outbox\|reap\|consume_outbox"` returns zero. Comment at `m13_stcortex_writer/mod.rs:470` explicit — "m13's fire-and-forget contract — the outbox consumer can choose..." (no consumer in-crate). |
| 7 | NA-GAP-09 — CC-5 substrate-confirmable receipt | **❌ spec-only** (zero hits in `src/`) | Only `NexusEventKind` variants are `WorkflowDispatched` + `WorkflowCompleted` (`m40_nexus_emit/mod.rs:41–48`) — no `Refusal` envelope; no confirmation-receipt event. Wire is one-way. `ServerRejected` (`:73–79`) catches AP-V7-13 health-200 lies (transport-truthing), but is not the CC-5 receipt the spec promises. |
| 8 | NA-GAP-11 — m32 `HumanAcceptanceSignature` + m9↔m32 trait | **⚠ partial** | **Type shipped:** `m32_dispatcher/mod.rs:120–137` defines `HumanAcceptanceSignature { interactive_terminal, acknowledged_ceiling: EscapeSurfaceProfile }`; monotone `is_acknowledged_by` gate; used in `dispatch(...)` `:250,363`; `EscapeSurfaceNotAcknowledged` refusal at `:391`. **Trait seam absent:** `src/m9_watcher_namespace_guard/validator.rs:169–177` is an explicit `TODO(m30/m32 — Cluster G, post-Wave-3)` block. |

### Aggregate verdict

- **3 / 8 genuinely code-backed:** NA-GAP-02, NA-GAP-03, NA-GAP-05.
- **2 / 8 partial:** NA-GAP-06 (write done, drain absent), NA-GAP-11 (type shipped, trait seam TODO).
- **3 / 8 spec-only — were claimed absorbed but have zero implementation:** NA-GAP-01, NA-GAP-04, NA-GAP-09.

### Honest project-record correction (per plan §3 step 2)

The S1002127 closeout's "8/11 closed" label is **not faithful to the shipped tree**. The three spec-only items must be either (a) folded into Plan v2 phases as M0 work-items, or (b) explicitly added to ADR `D-S1002127-03` deferral list (alongside NA-GAP-07/08/10).

**v2 disposition (recommend):**

| NA Gap | Disposition | Phase |
|--------|-------------|-------|
| **NA-GAP-09 CC-5 substrate-confirmable receipt** | **Fold into M0 via Phase 6f.** Plan v2 § 3 Phase 6f already specifies "Substrate-confirmable verdict receipts — every Refuse/Amend verdict emits an observable `WireEvent`" — same primitive. Adds a `Refusal` envelope to `NexusEventKind` + emit path. ~60–100 LOC (already in 6f estimate). The spec-only label is satisfied **only** if 6f ships — explicit cross-reference, no silent inheritance. |
| **NA-GAP-01 RefusalToken** | **Defer to v0.2.0** alongside NA-GAP-07/08/10. Touches m9, m32, m13, m40, m41, m42, m33 (~150–300 LOC + ADR amendment). Current `RefusalReason` enum is functional; the authorship-typed `RefusalToken` is a substrate-frame primitive that pairs naturally with NA-GAP-09's substrate-confirmable receipt and NA-GAP-10's substrate-mediated trust. Amend `D-S1002127-03` to include it. |
| **NA-GAP-04 substrate back-pressure budget** | **Defer to v0.2.0.** Natural v0.2.0 territory (substrate-side primitive the engine receives, not emits). Amend `D-S1002127-03` to include it. |
| **NA-GAP-06 outbox drain (partial)** | **Pair with NA-GAP-09 in Phase 6f.** Drain + confirm together is the natural unit; emission without read-back is half a loop. |
| **NA-GAP-11 m9↔m32 trait (partial)** | **Fold into Phase 6e** (m9 EscapeSurfaceProfile seam). Plan v2 § 3 Phase 6e already defines the m9↔m32 `HumanAcceptanceSignature` trait once, shared with 6a (gap C-8). This closes NA-GAP-11's trait gap as a side-effect. ~0 LOC added beyond 6e. |

**Net effect:** the v2 Phase 6 plan already contains the natural homes for the two MED items (NA-GAP-09 → 6f, NA-GAP-11 → 6e). The HIGH item NA-GAP-01 (RefusalToken) + LOW item NA-GAP-04 are honestly partitioned to v0.2.0. The "8/11 closed" claim is reframed as "3/8 fully + 2/8 partial naturally completed by Phase 6e/6f + 3/8 explicitly deferred or amended". **No silent inheritance.**

---

## 3. m33 Verifier input catalog (Plan v2 §3 Phase 2 step 3) — VERIFIED

The `Verifier` trait (`src/orchestration/dispatch.rs:323–333`) receives `&AcceptedWorkflow` and returns `VerifierVerdict`. Per-kind input audit:

| Kind | Inputs from `&AcceptedWorkflow` | Inputs from `&Config` / context | Status at HEAD `24cf6e1` |
|------|--------------------------------|---------------------------------|--------------------------|
| **Security** | `proposal().variant()` (m21 `WorkflowVariant { variant_id, steps, mutation, source_pattern_hash }`) — **but no `escape_surface` field on `WorkflowVariant` or `WorkflowProposal`** | `ack_ceiling: EscapeSurfaceProfile` (Sandboxed default — D7) from `Config` | **GAP** — Phase 6a needs to choose **how the proposal's surface is determined**: (i) add `escape_surface: EscapeSurfaceProfile` to `WorkflowProposal` (cross-binary wire change ~150–200 LOC, like Cost), OR (ii) build a `StepToken → EscapeSurfaceProfile` lookup + variant-aggregation (~80–150 LOC for table + max-aggregation), OR (iii) inherit from m32's existing monotone ack-gate which already enforces this defense-in-depth at dispatch time and document 6a as a redundancy with documented overlap. Plan §3 Phase 6a's "~40–70 LOC" estimate is contingent on choice (iii); choices (i)/(ii) are larger. **Surfacing for Phase 6 planning.** |
| **Ember** | proposal artefact text (TBD: serialised proposal? variant steps as text? rationale? — needs Phase 6b sub-decision) | m10 rubric machinery via `m10_ember_ci_gate::rubric::score_against_rubric(text)` (`src/m10_ember_ci_gate/rubric.rs:100`) — present ✓, reusable per D15 | Functional dependency met; "what gets scored" is the 6b sub-decision (artefact-formatting choice). |
| **Cost** | `proposal().cost()` — **no `cost` field on `WorkflowProposal`** | n/a | D9 stub correct; ship `Approve` returning + doc; ~10 LOC. |
| **Consistency** | `CuratedBank` accessor for conflict detection (e.g., overlapping `variant_id` already-accepted) — **no `client_ref()` accessor** at `src/m30_bank/mod.rs` (T4-API #1 still open per Phase 1 residual list) | n/a | D11 defer + D12 bank-accessor on-demand; ship `Approve` stub for M0; ~10 LOC. |

**`EscapeSurfaceProfile` cardinality verified:** 7 variants in ord-ascending order at `src/m32_dispatcher/mod.rs:40–58` — `Sandboxed(0) · SandboxEscape(10) · ProcessMutate(20) · PrivilegeEscalation(30) · FileWrite(40) · NetworkEgress(50) · DataExfil(60)`. D-S1002127-02 amendment landed.

### Phase 6a Security re-sizing recommendation

Of the three sizing options above, **option (iii)** is most consistent with the rest of § 15's quality-first stub doctrine (D9 Cost stub, D11 Consistency stub) and avoids a cross-binary wire-contract change for M0:

> **6a Security verifier ships as:** "compare `config.ack_ceiling` to the m32-monotone gate's already-enforced ceiling; verdict is `Approve` (no-op) at M0; the verifier is wired but defers to m32's existing enforcement; documented as a defense-in-depth redundancy slot that will be replaced by either choice (i) or (ii) in v0.2.0 when a per-workflow surface determination is available."

This shrinks 6a to ~30–50 LOC (verifier wiring + cross-reference docstring + tests) and keeps the **hard-Refuse semantic** (D5/D6) intact via m32's existing path. The wired m33 Security verifier is then ready to accept a real surface input in v0.2.0 without further trait reshape.

**Alternative (if node 0.A overrides for full M0 Security):** option (ii) — `StepToken → EscapeSurfaceProfile` table + variant aggregation. Estimated ~80–150 LOC. Adds a habitat-shared StepToken-classification table (small new module). Defers cross-binary wire-contract change.

This is **a finding, not a fait accompli** — the plan §3 Phase 6a estimate was made before this Phase 2 audit verified the absence of a per-workflow surface. Phase 6a sub-decision is **surfaced here** for node 0.A's awareness; it does not require an interview round because D5/D6/D7 lock the **semantic** (hard-Refuse above Sandboxed default); only the **mechanism** is now sized honestly.

---

## 4. Surfaces touched by Phase 2

- **This doc** (canonical Phase 2 audit) — `ai_docs/PHASE2_AUDIT_S1004115.md`
- **PHASE1_RESIDUAL_LIST_S1004115.md** MUT-1 path-typo (Zen Phase-1 LOW nit) — fixed in this commit
- **Vault `Hardening Fleet 2026-05-21.md`** W4 row (Zen Phase-1 MED nit; CHANGELOG-fold-missed surface) — folded in this commit (`254/94.4 @0cc7be3` → `259/96.3 post-Wave-G + C22`)
- **Vault `Assessment Remediation S1003733.md`** lines 101/117/212–213 (vault DOC-3 fold) — folded in this commit
- **`src/`** — **not touched** (Phase 2 is read-only audit; code work begins Phase 3/5/6/7)

---

## 5. Zen Phase-1 verdict fold (received 2026-05-23 in-session)

Verdict: **APPROVE-WITH-NITS → proceed to Phase 2.** Concrete findings folded:

| Severity | Finding | Disposition |
|----------|---------|-------------|
| MED | Vault `Hardening Fleet 2026-05-21.md` W4 row still cites 254/94.4 — out of Phase 1 scope per Plan §3 step 1 but real loose end | **Fold here** (§ 4 surfaces) |
| LOW | MUT-1 row in `PHASE1_RESIDUAL_LIST_S1004115.md` missing `src/m11_fitness_weighted_decay/` prefix on `inputs.rs:215–234` | **Fold here** (§ 4 surfaces) |
| Hygiene-asymptotic | Working tree dirties continuously from live Watcher journal appends; "tree-clean at HEAD" is structurally unachievable under current architecture | **Phase 10 hand-off candidate**: gitignore the Watcher journals or move them out of the workflow-engine repo. Documented; not blocking. |
| Stcortex CLI ORDER-BY broken | DOC-1 read-back not Zen-verifiable via raw-SQL; MCP path works | **No action**: MCP read-back returned `id=18410, modality=meta` — DOC-1 receipt is honest evidence. CLI breakage is a habitat-side issue (filed informally; not Phase 1 regression). |

---

## 6. Phase 2 done-evidence (per D43)

- **Gate** (re-run on this commit's HEAD): forthcoming (4-stage).
- **Test count delta**: expected +0 (docs-only phase; no `src/` changes).
- **Audit fan-out**: 1 subagent dispatched (NA-gap code-backed audit) per D41; findings folded above.
- **Surfaces**: 1 new canonical doc + 3 edits (vault + residual-list nit fix).
- **No code touched.** R1/R2/Phase 6 sizing **surfaced** for downstream phases; no semantic changes to § 15 decisions (DA1–DB7 stay locked).

— Phase 2 audit, S1004115
