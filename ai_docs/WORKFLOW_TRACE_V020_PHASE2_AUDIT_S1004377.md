# Phase 2 Audit — v0.2.0 deep FP-verify + Tier 2 W1 sizing + 7-substrate enumeration + V3 Genesis v1.4 pre-flight

> **Authored:** 2026-05-23 (S1004377, Plan v2 v0.2.0 Phase 2 — decision-free)
> **Status:** decision-free audit; output feeds Phase 5 W1+V1+A4 co-land + Phase 9 V3 m16 own-module + Phase 10 V4 fixtures
> **Per Plan v2 §3 Phase 2:** evidence-cited audit; orchestrator-direct (no subagent fan-out this phase per D41 solo discipline; subagents reserved for Phase 5 W1 dispatch fan-out)
> **Back to:** [`WORKFLOW_TRACE_V020_PLAN_V2_S1004377.md`](WORKFLOW_TRACE_V020_PLAN_V2_S1004377.md) · [CLAUDE.local.md](../CLAUDE.local.md) · [`PHASE2_AUDIT_S1004115.md`](PHASE2_AUDIT_S1004115.md) (v0.1.0 era; Plan v2 §3 Phase 2 precedent)

---

## 1. Re-verify v0.1.0-era Phase 2 audit findings against HEAD `39e71a7`

The v0.1.0/M0 Phase 2 audit (`PHASE2_AUDIT_S1004115.md` §2) flagged 3 of 8 absorbed NA gaps as "spec-only" (NA-GAP-01 / NA-GAP-04 / NA-GAP-09) and 2 as partial (NA-GAP-06 drain absent / NA-GAP-11 trait seam TODO). Phase 6e/6f at M0 closed NA-GAP-09 + NA-GAP-11. v0.2.0 Plan v2 §2.1 + §2.5 + ADR D-S1002127-03 Amendment 1 register the three remaining spec-only / partial items as v0.2.0 active.

Re-verification at HEAD `39e71a7`:

| Item | Phase 2 (S1004115) verdict | Re-verify at HEAD `39e71a7` | Status |
|------|----------------------------|------------------------------|--------|
| **NA-GAP-01 `RefusalToken` Rust type** | ❌ spec-only (zero hits in `src/`) | `rg -n "RefusalToken\|substrate_authored\|engine_authored\|operator_authored" src/` → **1 hit** at `src/orchestration/dispatch.rs:356` (a comment referencing v0.2.0 plan: "v0.2.0 work alongside NA-GAP-01 (RefusalToken)"). **No actual `RefusalToken` type defined.** | ✅ Phase 2 audit confirmed; V1 still ungrounded. ADR D-S1004XXX-04 (Phase 1) is the design spec; Phase 5 lands the type. |
| **NA-GAP-04 substrate back-pressure** | ❌ spec-only (zero hits) | `rg -n "back_pressure\|backpressure\|BackPressure\|throttle" src/` → **zero hits**. | ✅ Phase 2 audit confirmed; V2 still ungrounded. Phase 8 lands per-substrate `SubstrateBackPressureMode` enum per DX-2. |
| **NA-GAP-06 m13 outbox drain** | ⚠ partial (write done, drain absent) | `rg -n "drain_outbox\|drain(\|outbox_drain\|reap_outbox" src/m13_stcortex_writer/` → **zero hits**. Write half still at `m13_stcortex_writer/mod.rs:307–369` (`outbox_path` + `outbox_lock`); drain consumer side absent. | ✅ Phase 2 audit confirmed; C1 still partial. Phase 3 stages drain skeleton; Phase 5 wires V1-typed consumer. |

**Aggregate:** all three flagged items at v0.1.0 remain ungrounded at HEAD; the Plan v2 v0.2.0 work-item registry is faithful to shipped state.

---

## 2. W1 wire-contract blast-radius enumeration (per Plan v2 §3 Phase 5 + C-2 co-land)

DX-W.b locked = **W1 wire-bump** (`escape_surface: EscapeSurfaceProfile` field on `WorkflowProposal`). DX-W.c locked = **SemVer-break** (v0.1.0 proposals.jsonl do not deserialise). Per C-2 fold-in, V1 RefusalToken co-lands with W1 in Phase 5 — one JSONL fixture regen pass.

### 2.1 `src/` consumers (6 files; per `rg -l "WorkflowProposal" src/`)

| File | Role | W1 touch | V1 co-land touch (Phase 5 step 5) |
|------|------|----------|------------------------------------|
| `src/m23_proposer/mod.rs` | Declares `WorkflowProposal` (struct at `:27`) + `new()` + accessors | **Add `escape_surface: EscapeSurfaceProfile` field; update fallible `new()` signature; accessor; serde regen via derive (auto)** | NA — V1 is a separate enum; m23 doesn't refuse |
| `src/m30_bank/mod.rs` | `AcceptedWorkflow` wraps `WorkflowProposal` as-is (`:63`) | Auto via wrapper; no struct change | NA — bank doesn't refuse |
| `src/m42_stcortex_emit/mod.rs` | Emits proposals to stcortex (Hebbian-coordinator path) | Auto via serde | **Replaces flat `RefusalReason` usage at stcortex refuse-write path with `RefusalToken::SubstrateAuthored Stcortex`** |
| `src/orchestration/crystallise.rs` | Creates proposals + `write_proposals_jsonl` at `:512` | **Populate `escape_surface` in `compose_proposals` (line 504-509 area; needs source per Phase 5 step 5 — likely m23 caller passes it)** | NA — crystallise doesn't refuse |
| `src/orchestration/dispatch.rs` | Deserializes via `serde_json::from_str` at `:594–595`; `Verifier::verify` consumes at `:331`; comment at `:356` references NA-GAP-01 | Auto via serde; verifier consumes new field | **Replaces flat `RefusalReason` usage at m32 dispatch path with `RefusalToken::EngineAuthored M32` (per ADR D-S1004XXX-04 § 1.2 row)** |
| `src/lib.rs` | Re-exports `WorkflowProposal` + helpers | No change (re-export passes through) | No change (V1 also re-exported once defined) |

**Per ADR D-S1004XXX-04 § 1.2 call-site classification table — additional V1 touch sites in `src/` not coupled to W1:**

| File | V1 touch (Phase 5 + Phase 7) |
|------|------------------------------|
| `src/m9_watcher_namespace_guard/validator.rs` | `NamespaceViolation::CapabilityNotAcknowledged` typed as `RefusalToken::SubstrateAuthored Stcortex` (reducer-origin) OR `RefusalToken::EngineAuthored M9` (local-trait-origin) — FP-verify per call origin in Phase 7 step 2 |
| `src/m32_dispatcher/mod.rs:228` | Flat `RefusalReason` enum removed; its variants become `EngineRefusalReason` nested in `RefusalToken::EngineAuthored M32` |
| `src/m33_verifier/mod.rs` (any module) | All Refuse verdicts typed `RefusalToken::EngineAuthored M33` |
| `src/m40_nexus_emit/mod.rs:51,248,572,1860` | `ServerRejected` events typed `RefusalToken::SubstrateAuthored {substrate_id}` (already substrate-frame in semantics; just gets typed enum) |
| `src/m41_lcm_rpc/mod.rs` | Mixed: `RefusalToken::Unavailable::SubstrateUnreachable Lcm` (transport) vs `RefusalToken::SubstrateAuthored Lcm` (refused) — FP-verify per call origin in Phase 7 step 2 |
| `src/m13_stcortex_writer/mod.rs` | `outbox_path` write failures typed `RefusalToken::SubstrateAuthored Stcortex` (refuse-write-no-consumer); rare fail-on-disk = `RefusalToken::EngineAuthored M13` |

### 2.2 Test fixture regen surface (10 integration test files; per `rg -l "WorkflowProposal|proposals.jsonl|proposal_id" tests/`)

W1 SemVer-break regenerates proposal-bearing JSONL fixtures across:

| File | What it fixtures |
|------|------------------|
| `tests/wf_crystallise_integration.rs` | End-to-end crystallise → proposals.jsonl; 12 tests last run |
| `tests/wf_dispatch_integration.rs` | End-to-end dispatch reading proposals.jsonl; 13 tests last run |
| `tests/cc5_substrate_cycle.rs` | CC-5 substrate learning loop — proposal+receipt round-trip |
| `tests/cc4_proposal_to_dispatch_pipeline.rs` | CC-4 proposal → bank → dispatch pipeline |
| `tests/cc7_pressure_evolution.rs` | CC-7 PressureEvent → m23 compose-priority |
| `tests/m11_integration.rs` | Fitness-weighted decay over proposals |
| `tests/m23_integration.rs` | m23 proposal construction + serde |
| `tests/m30_integration.rs` | Bank acceptance of proposals |
| `tests/m31_integration.rs` | Selector consuming proposals |
| `tests/m32_integration.rs` | Dispatcher acceptance signature path |

**Phase 5 step 5 commit budget:** one JSONL fixture regen pass touching ~10 integration test files. Per the C-2 co-land discipline, this regen lands with W1+V1+A4 in a single Phase 5 commit cluster (per D44 one-commit-per-sub-phase).

### 2.3 Backwards-compat surface (DX-W.c = SemVer-break)

Files outside `src/` and `tests/` that may carry v0.1.0-shape proposal JSONL:

| Surface | Risk | Mitigation |
|---------|------|------------|
| Developer-local `proposals.jsonl` (uncommitted, anywhere on disk) | Will fail to deserialise post-Phase 5 | CHANGELOG [v0.2.0] § "Changed" migration note; explicit `--migrate` flag NOT added per D9 lock + Plan v2 §2.2 (W3 is the only Phase 5 wire-format-bumping decision that touches Cost; the rest are additive — but escape_surface is also bump-with-default-on-deserialize per DX-W.c which chose SemVer-break) |
| In-flight CI artefacts | None at v0.2.0; CI ships with v0.2.0 per D29 + DX-CI submodule | N/A |

---

## 3. Substrate-enumeration audit per NA-2 + convergent C-3 — 7 substrates verified

Plan v2 v2 §7 expanded the substrate set from v1's 4 to 7 substrates per NA-2 + convergent C-3. Phase 2 verifies each against runtime evidence + names the v0.2.0 participation surface.

| # | Substrate | Runtime evidence | v0.2.0 participation surface | §11 consent-gradient |
|---|-----------|------------------|------------------------------|----------------------|
| 1 | **atuin** | Shell history WAL at `~/.local/share/atuin/`; m1 reads | V2 `SubstrateBackPressureMode::Pull` default per DX-2; flip to Push if upstream emitter ships | UNKNOWN / indeterminate (third-party upstream) |
| 2 | **stcortex** | SpacetimeDB at `127.0.0.1:3000`; namespace `workflow_trace_v020_s1004377` populated mem 18511 + 18517; bidi pathways 21178 + 21180 | V1 `RefusalToken::SubstrateAuthored Stcortex` for refuse-write; m13 outbox drain consumer (C1); V3 m16 sampler for stcortex-decay clock | HIGH / 1-2 weeks |
| 3 | **HABITAT-CONDUCTOR** | Port `:8141` (NOT yet bound up — pending OP-1); auto_start=false in devenv.toml | V5 `SubstrateTrust.conductor_budget_remaining`; m32 dispatch path queries Conductor enforcement-state | HIGH / 1-2 weeks (after OP-1 Conductor bring-up) |
| 4 | **CC-5 loop clocks** (5 sub-clocks) | m11 recency (engine wall-clock); m13 stcortex-decay (stcortex schedule); injection-TTL (injection.db sqlite TTL sweep); atuin-checkpoint (atuin WAL checkpoint); stcortex-pathway-decay (stcortex Hebbian decay) | V3 m16 5-clock sampler + agree-to-skew envelope | n/a (clocks are engine-observable; substrate-side is implicit in each individual substrate) |
| 5 | **Luke @ node 0.A + Watcher ☤** | Luke @ keyboard (session S1004377); Watcher in `synthex-v2/src/m8_watcher/m46-m51` (~2410 LOC); 48,723 observations per workspace `CLAUDE.local.md` | DX-1 (4-variant; Watcher emits via observation channel per §15 lock); V3 OP-6 Watcher m16 heartbeat liveness assertion (closes V3 self-canary loop per NA-4); V5 `watcher_m16_heartbeat_live` accessor | Luke = active (interview); Watcher = active (synthex-v2 :8092) |
| 6 | **RALPH** | Gen 7,622 fit 0.6987 per workspace `CLAUDE.local.md`; fitness signal received via m42 stcortex emit pathway | V5 `SubstrateTrust.ralph_generation_advanced_since: bool` accessor — engine reads RALPH fitness-signal back as substrate speech | MED / 2-4 weeks (RALPH lane integration via ORAC) |
| 7 | **Cargo build graph** | `Cargo.toml` deps incl `spacetimedb-sdk` (sibling-repo); `Cargo.lock` partly written by other services; clippy + pedantic constraints | DX-CI Option A submodule per §15 lock — Frame-B observation point (workflow-trace acknowledges substrate's authorship); Phase 12 step 5 wires submodule | n/a (substrate-coupling shape, not consent-gradient) |

**Verdict:** all 7 substrates have concrete v0.2.0 participation surfaces. §7 enumeration in Plan v2 v2 is faithful; no substrate has been promoted-without-anchor.

---

## 4. C-4 Zen-verdict-absent confirmation (per Plan v2 §3 Phase 9 + DX-V3.b 7-day ship cap)

Plan v2 v2 Conventional gap analysis C-4 (HIGH) claimed: "zero `zen_*verdict*` files exist for any workflow-trace hardening wave (W1-W4 / S1003733 / C22)" — the Plan v2 era precedent inherited from v0.1.0 Conventional C-4.

Re-verification at HEAD `39e71a7`:

`ls ~/projects/shared-context/agent-cross-talk/ | grep -iE "zen.*workflow|workflow.*zen|zen.*wfe|wfe.*zen"` returns **0 strict matches** for the `zen_*verdict*` pattern targeting workflow-trace.

Broader search for any Zen-authored verdict pointing at workflow-trace returned:
- `2026-05-17T094500Z_luke_as_zen_g7_verdict_approve_v3.md` — **Luke acting as Zen substitute** per D26 in-session-zen-agent discipline; this is a workflow-trace G7 verdict for v1.3 amendment scaffold. **Not external Zen.**
- Several `2026-05-21T*command_zen_*` files — Command-authored *notices* about Zen assessment workflow ("mission start", "context", "complete") — **Command-authored signals about Zen work, not Zen verdicts.**

**Verdict:** C-4 claim confirmed at HEAD. Phase 9 V3 own-module (DX-V3 lock) **will trigger Zen G7 re-audit cascade** per ADR D-S1002127-03 § 3 + Genesis Prompt v1.4 amendment; the **DX-V3.b 7-day ship-with-honest-residual cap** (locked S1004377) is the mitigation. If Zen remains silent past 7 days, Phase 9 ships with the Genesis v1.4 + m16 own-module integration named in CHANGELOG [v0.2.0] § "Honest residuals" as un-audited cardinality drift.

---

## 5. V3 m16 Genesis Prompt v1.4 amendment pre-flight (per Plan v2 §3 Phase 9 + DX-V3)

DX-V3 locked = **own module (m16)**. Phase 9 owns the Genesis v1.4 amendment. Per ADR D-S1002127-03 § 3 W1 row, the trigger is module-count change 26 → 27 + test budget bump + Zen G7 re-audit.

### 5.1 Genesis Prompt v1.3 anchors that v1.4 must amend

Read at HEAD `39e71a7` from `ai_docs/GENESIS_PROMPT_V1_3.md`:

| Line | Anchor | v1.4 amendment shape |
|------|--------|----------------------|
| `:35` | `## § 1 — Architecture (26 modules / 8 clusters / 9 layers / 2 binaries)` | **Lift to 27 modules.** Cluster count unchanged (m16 lands in Cluster E or new Cluster I per DX-V3 sub-choice — recommend Cluster E expansion; Cluster I would amend the cluster count too). |
| `:37` | `The single-phase architecture is 26 modules across 8 synergy clusters and 9 layers (L0-L8)` | Lift to 27 modules; clusters/layers unchanged |
| `:62` | `**OI-3 resolution**: the architecture count is **26 modules**, not v1.2's 11. m33 is the additive module (§ 1.a).` | Add OI-3.b resolution row: "v1.4 lifts count to 27 modules; m16 is the v0.2.0 additive module (per ADR D-S1002127-03 Amendment 1 + Plan v2 v0.2.0 §2.1 V3 + DX-V3 own-module lock)" |
| `:288` | `Total budget **1,562 tests** across 26 modules` | Lift to **1,602 tests** across 27 modules (add ~40 tests for m16 per ADR D-S1002127-03 § 3 estimate). Recompute the per-V7-G6 budget proportionally. |

### 5.2 Cluster placement (DX-V3 sub-decision NOT in §15)

DX-V3 locks "own module" but does NOT lock cluster placement. Phase 9 step 1 owns this sub-decision. The v1.4 amendment will name one of:

- **Option A: Cluster E expansion** — m16 joins {m14 lift, m15 pressure} as a substrate-observation module. Preserves 8-cluster lock. **Recommended** (smaller v1.4 amendment surface; Cluster E is the natural home per the substrate-drift cross-cutting spec).
- **Option B: new Cluster I** — m16 alone in a new Cluster I "substrate-observation". Lifts cluster count 8 → 9. Triggers Cluster-I row in every ULTRAMAP / per-cluster spec / docs surface. **Larger amendment surface; only if Cluster E's role-coherence would be diluted.**

Phase 9 step 1 will pick A (recommended) unless evidence emerges otherwise during Genesis v1.4 authoring.

### 5.3 Zen G7 re-audit dispatch shape

Per ADR D-S1002127-03 § 3 W1 row + Plan v2 §3 Phase 9 (DX-V3 = own module branch):

1. Phase 9 Day 1: author Genesis v1.4 amendment file `ai_docs/GENESIS_PROMPT_V1_4.md`; pair-file at `~/projects/shared-context/agent-cross-talk/` with cover note.
2. Phase 9 Days 2-7+: Zen wait (3-10 days per cross-talk history; bounded by DX-V3.b 7-day cap → ship with honest residual at day 7 if silent).
3. Parallel Days 2-5: m16 implementation per `ai_specs/cross-cutting/substrate-drift.md`.
4. Phase 9 ship-gate: Zen verdict landed OR 7-day cap expired.

---

## 6. Phase 2 done-evidence (per Plan v2 §15 D43)

- **Gate (4-stage, all green):**
  - `cargo check --all-targets --all-features` ✅ (carried from Phase 1; no source code touched in Phase 2)
  - `cargo clippy --all-targets --all-features -- -D warnings` ✅
  - `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic` ✅
  - `cargo test --all-targets --all-features --release` ✅ 2048 passed / 0 failed / 1 ignored across 38 suites (Phase 1 baseline; Phase 2 no-delta)
- **Test-count delta:** +0 (doc-only)
- **Cargo-mutants:** N/A (no source code change)
- **Surfaces:** this Phase 2 audit doc (NEW); no edits to other surfaces (Phase 2 is read-only)
- **Stcortex memory:** Phase 2 progress write + read-back will land with the commit

---

## 7. Net Phase 2 disposition

- All 3 v0.1.0 Phase 2 audit findings re-verified at HEAD: NA-GAP-01 / NA-GAP-04 / NA-GAP-06-drain remain ungrounded. v0.2.0 Plan v2 work-item registry is faithful.
- W1 blast radius sized: 6 src files (immediate) + 10 integration test fixture files (JSONL regen); V1 co-lands per C-2.
- 7-substrate enumeration verified against runtime evidence; all substrates have concrete v0.2.0 participation surfaces.
- C-4 Zen-verdict-absent claim confirmed for workflow-trace hardening waves at HEAD; DX-V3.b 7-day cap is the right mitigation.
- Genesis v1.4 amendment pre-flighted: 4 v1.3 anchors enumerated for amendment; recommended Cluster E placement for m16; Zen re-audit dispatch shape per ADR D-S1002127-03 § 3.

**No surprises surfaced.** Phase 2 is **clean** — proceed to Phase 3 (A2 + C1 staging, decision-free Rust code) on Phase 2 commit.

*Phase 2 audit, S1004377 · 2026-05-23 · Claude @ cortex · decision-free; no source code touched · feeds Phase 3 / Phase 5 / Phase 9.*
