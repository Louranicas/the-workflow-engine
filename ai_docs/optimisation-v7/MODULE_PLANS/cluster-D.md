---
title: MODULE PLAN — Cluster D (Trust — aspect layer)
date: 2026-05-17
kind: planning-only · per-module deep plan · V7 T3.D deliverable
cluster: D
layer: L4 (aspect)
modules: [m8, m9, m10, m11]
status: V7 author-wave subagent draft (Command)
---

# Cluster D — Trust (L4 aspect layer)

> Back to: [[../TASK_LIST_V7_OPTIMISATION.md]] · [[../ULTRAMAP.md]] · [[../GENERATIONS/G3-bidi-flow.md]]
>
> Peers: [[cluster-A.md]] (D wraps A reads) · [[cluster-B.md]] (D wraps B observation writes) · [[cluster-C.md]] (D wraps C central + writer; m11 reads m7) · [[cluster-E.md]] (D wraps E evidence) · [[cluster-F.md]] (D wraps F iteration) · [[cluster-G.md]] (D wraps G dispatch; m11 feeds m31 selector) · [[cluster-H.md]] (D wraps H feedback writes)

---

## Overview

Cluster D is the engine's **aspect layer** — woven through every other cluster at compile-time (m8), write-time (m9), output-time (m10), and lifecycle (m11) per ULTRAMAP View 1 dashed aspect arrows. It is NOT a sequential pipeline layer; it is the cross-cutting trust regime that holds Clusters A/B/C/E/F/G/H to the engine's invariants. Per CLAUDE.md § structural-gap authorship, Cluster D is the unique owner of **Gap 2** (`frequency × fitness × recency` compound decay formula — m11; ~200-300 LOC NEW PRIMITIVE) and the m9-side co-owner of **Gap 3** (Unified destructiveness / EscapeSurfaceProfile schema — shared with m30 + m32; ~150-250 LOC).

Per ULTRAMAP View 2, Cluster D accounts for **~450 LOC** (m8 ~60 + m9 ~50 + m10 ~90 + m11 ~250 — m11 is the largest non-KEYSTONE module by LOC because of Gap 2) and **230 tests** (50 + 50 + 60 + 70 per TEST_DISCIPLINE matrix). m11 has the highest test density in D because Gap 2 (compound decay formula) requires property-test invariants over a 4-dimensional input space (`base`, `frequency`, `fitness`, `recency`).

Cluster D itself has an **aspect-IN from m8** — m8 is the lowest aspect; it has no further upstream aspect. This is the bootstrap convention: m8 build-prereq is the floor of the trust regime. All other D modules transitively depend on m8's `cfg(povm_calibrated)` compile-time gate (per G3 § m8 contract).

The substrate-frame role (per G3 substrate-frame pass): Cluster D is the trust regime that prevents anthropocentric framing from corrupting substrate-grain measurement. m11's `frequency × fitness × recency` formula is the canonical substrate-frame measure of pathway vitality — not "did the user like this workflow", but "did the substrate weight rise". Watcher Class-I (Hebbian silence) maps directly onto m11's signal.

---

## m8 — build-prereq

**Purpose.** `build.rs` script that detects POVM at `:8125` and emits `cargo:rustc-cfg=povm_calibrated` if `learning_health` ∈ [0.05, 0.15]. Hard-fails the build if outside the band. The lowest aspect — no further upstream.

**Upstream-IN** (per G3 § m8).
- `build.rs` execution context (Cargo build invocation).
- HTTP probe to `http://127.0.0.1:8125/learning_health` (POVM health endpoint).
- `m8.config` env vars (`POVM_HEALTH_URL` override for CI; `POVM_HEALTH_TIMEOUT_MS`).

**Downstream-OUT.**
- `cfg(povm_calibrated)` available to ALL modules (compile-time).
- `build.rs` stdout messages per Cargo convention (`cargo:rerun-if-env-changed=POVM_HEALTH_URL`, `cargo:warning=...` on band-edge).

**Aspect-IN.** None — m8 IS the lowest aspect (per G3 § m8 contract).

**src/ path:** `build.rs` at workspace root + `src/m8_build_prereq/` for the runtime mirror (`HealthClient` + band-check used at startup, not just build-time). Sub-modules: `mod.rs`, `health.rs` (HTTP probe + band logic), `cfg.rs` (the band thresholds — single source of truth shared between `build.rs` and runtime), `error.rs`.

**LOC budget** (per ULTRAMAP View 2): **~60 LOC** (≈30 in `build.rs` + 30 in `src/m8_build_prereq/`).

**Test budget** (per TEST_DISCIPLINE matrix row m8): **50 tests**.

**Test-pattern allocation.**
- F-Unit 25 — band-check per band (below 0.05 / in band / above 0.15 / NaN / negative), `HealthClient::probe` per status-code arm, env-var override application, `cargo:warning` emit on band-edge.
- F-Property 5 — band-check monotonic (`learning_health` ascending → band classification ascending); idempotent (probe twice with same input → same output); deterministic on `(POVM_HEALTH_URL, learning_health)`.
- F-Fuzz 0.
- F-Integration 15 — live POVM `:8125/learning_health` probe; CI env-var override path; build-time + runtime mirror band-check agreement; `cfg(povm_calibrated)` compile-emission verification via `cargo expand`.
- F-Contract 3 — `learning_health` JSON shape vs POVM `:8125` actual response; band-threshold constants vs Hebbian v3 reconciliation note thresholds.
- F-Regression 2 — F7 (CR-2 graceful-degrade pretend-fix) regression slot; build-time/runtime mismatch regression.
- F-Mutation 1 budget — ≥70% on `health.rs` (the band-check core).

**Mutation kill threshold:** ≥70%.

**Boilerplate-lift source.**
- `reqwest::blocking::Client` for build-time HTTP probe — standard crate idiom.
- Cargo `build.rs` convention from `synthex-v2/build.rs` and `loop-engine-v2/build.rs` (~70% reuse for `println!("cargo:rustc-cfg=…")` emission pattern).
- Band-threshold constants — derived from CR-2 reconciled `learning_health` band per `~/projects/claude_code/Hebbian Deployment Plan v3 — Post-CR-2 Threshold Reconciliation.md`.
- F7 hard-fail logic — fresh authorship (~15 LOC).

**Structural-gap LOC.** None for m8 specifically. m8 is the trust regime's floor — no formula authorship.

**Failure-modes covered** (per ANTIPATTERNS_REGISTER § AP-WT-F7).
- **F7 (CR-2 graceful-degrade pretend-fix) — exclusive owner.** m8 hard-fails the build (and at startup, refuses to run) if POVM `learning_health` reads outside [0.05, 0.15] — never silently degrades. Per G3 § m8 contract.
- AP-Drift-11 (supervisor stub mistaken for live) — m8's runtime probe verifies POVM actually responds; never inferred from process-list.
- AP-Hab-14 (god-tier dilution) — m8 is the canonical no-fallback discipline; any commit adding a "soft" band-check is a god-tier dilution.

**Atuin trajectory anchor.**
- `wt-build-status` (proposed; runs `cargo check` and grep for `cfg(povm_calibrated)` emission to surface band state).
- `povm-probe` (atuin script; m8 is the build-side counterpart).

**Watcher class pre-position.**
- **Class A** — first successful `cargo build` with `cfg(povm_calibrated)` emitted.
- **Class I** — Hebbian silence; if `learning_health` drops below 0.05 mid-deploy, m8's runtime mirror refuses to run.

---

## m9 — namespace guard

**Purpose.** Validate namespace prefix `workflow_trace_*` at every stcortex/POVM write boundary (m13 + m42). Pure validator; no side effects.

**Upstream-IN** (per G3 § m9).
- Write attempts from m13 (`StcortexWriteRequest`) and m42 (`PovmReinforceRequest`).
- `m9.config.allowed_prefixes: Vec<&'static str>` = `["workflow_trace_"]` (single-element set; structurally exclusive).

**Downstream-OUT.**
- `Result<ValidatedNamespace, NamespaceViolation>` — pass-through with newtype evidence, or typed refusal.

**Aspect-IN.** None (pure validator; no further aspect needed).

**src/ path:** `src/m9_namespace_guard/` with `mod.rs`, `validator.rs` (the prefix check + hyphen-slug munge coupling), `error.rs`, `evidence.rs` (the `ValidatedNamespace` newtype + Display).

**LOC budget** (per ULTRAMAP View 2): **~50 LOC**. Almost entirely fresh authorship; no upstream equivalent for the `workflow_trace_*`-specific validator (the closest analogue is per-service namespace conventions in stcortex docs).

**Test budget** (per TEST_DISCIPLINE matrix row m9): **50 tests**.

**Test-pattern allocation.**
- F-Unit 25 — valid prefix per allowed entry; invalid prefix rejection per non-allowed entry; hyphen-slug munge per input/output pair; empty-string rejection; whitespace-trim rejection.
- F-Property 5 — `for_all (ns: &str) -> ns.starts_with("workflow_trace_") iff validate(ns).is_ok()`; munge idempotent (`munge(munge(x)) = munge(x)`); `ValidatedNamespace::as_str` round-trip.
- F-Fuzz 0.
- F-Integration 15 — m13 → m9 write boundary; m42 → m9 write boundary; concurrent write attempts; refuse-pass through both consumers; cross-substrate boundary (stcortex vs POVM).
- F-Contract 3 — `NamespaceViolation` error shape; `ValidatedNamespace` Display stability.
- F-Regression 2 — AP30 regression slot; AP-Hab-11 hyphen-slug regression.
- F-Mutation 1 budget — ≥70%.

**Mutation kill threshold:** ≥70%.

**Boilerplate-lift source.**
- Newtype evidence pattern — Rust idiom; no upstream lift needed.
- Hyphen-slug munge — fresh authorship (~5 LOC); aligns with AP-Hab-11 (S1001757) mitigation.
- Prefix-check — standard `str::starts_with`; no lift.

**Structural-gap LOC.** m9 is the **boundary-validator co-owner of Gap 3** (EscapeSurfaceProfile schema — shared with m30 + m32). m9 specifically validates the namespace dimension of the unified schema; per G3 + ULTRAMAP § Cluster D.

**Failure-modes covered.**
- **AP30 (namespace prefix discipline) — m9 prime enforcer.** m13 and m42 both delegate to m9; refuse-write at validator boundary, not at DB layer (DB-layer refuse is a backstop).
- **AP-Hab-11 (hyphen-slug munge)** — m9's `validator.rs` performs the hyphen → underscore conversion before prefix check; ensures munge happens exactly once per write.
- F3 (substrate-input poisoning) — m9 is the write-side prefix discipline; m2/m3 are the read-side complement.
- AP-V7-09 (substrate-frame engine confusion) — namespace prefix is the substrate-frame boundary; mixing namespaces is a Class-G drift.

**Atuin trajectory anchor.**
- `wt-namespace-audit` (proposed; greps stcortex namespaces for non-`workflow_trace_*` entries that should belong to workflow-trace).
- `stcortex-probe` (atuin script — m9 enforces what stcortex-probe surfaces).

**Watcher class pre-position.**
- **Class A** — first `validate(...)` call post-Genesis.
- **Class D** — four-surface drift if m9 passes a namespace but downstream surfaces (vault, ai_docs, CLAUDE.local.md) lack the corresponding anchor.

---

## m10 — Ember 7-trait CI gate

**Purpose.** CI gate that fails when Ember 7-trait audit returns `Held` on a PR-text artefact, unless the artefact is in the allowlist (`tests/ember_held_approvals.tsv`).

**Upstream-IN** (per G3 § m10).
- `~/projects/claude_code/Ember 7-Trait Gate Rubric.md` §5.1 (post-amendment: `Held = CI-FAIL` unless allowlist).
- `tests/ember_held_approvals.tsv` — TSV file of `(artefact_path, approved_by, approved_at, expiry)` rows.
- The candidate PR diff (PR-text only; non-PR-text modules are exempt).

**Downstream-OUT.**
- CI gate result: `Pass | Fail | Warning`.
- Structured event log entry (per tracing discipline).

**Aspect-IN.**
- m8 — must compile.

**src/ path:** `src/m10_ember_gate/` with `mod.rs`, `rubric.rs` (the 7-trait audit logic), `allowlist.rs` (the TSV reader + expiry check), `gate.rs` (the CI gate decision), `error.rs`.

**LOC budget** (per ULTRAMAP View 2): **~90 LOC**. ~30% boilerplate-lift (TSV parsing from RM v2 patterns); ~70% fresh (the rubric coupling — the rubric itself lives in `~/projects/claude_code/` and is canonical, but the audit-loop logic is workflow-trace-local).

**Test budget** (per TEST_DISCIPLINE matrix row m10): **60 tests**.

**Test-pattern allocation.**
- F-Unit 30 — per-trait audit per pass/fail/held arm; allowlist read + expiry-check per fresh/expired row; gate-decision per Pass/Fail/Warning combination; PR-text-vs-non-PR-text classifier.
- F-Property 5 — rubric monotonic (more passing traits → not-worse gate verdict); allowlist idempotent under reapplication; expiry strict (`now > expiry → not approved`).
- F-Fuzz 0.
- F-Integration 18 — live rubric file read; live TSV read; gate end-to-end against a fixture PR diff; gate against an empty allowlist; gate against a fully-allowlisted Held; CI-fail propagation to exit code.
- F-Contract 5 — rubric §5.1 schema vs implementation; TSV schema snapshot; gate-result JSON shape; tracing event shape; the Ember §5.1 amendment text.
- F-Regression 2 — reserved (e.g., allowlist-bypass regression).
- F-Mutation 1 budget — ≥70%.

**Mutation kill threshold:** ≥70%.

**Boilerplate-lift source.**
- TSV reader — adapted from RM v2 `read_tsv` helper (~70% reuse).
- 7-trait audit logic — fresh authorship; the rubric definition is canonical at `~/projects/claude_code/Ember 7-Trait Gate Rubric.md` §5.1 (amendment PENDING per B4 blocker).
- CI gate decision — adapted from ME v2 `src/m44_quality_gate/decision.rs` (~50% reuse).
- Expiry check — std `chrono` idiom; no lift.

**Structural-gap LOC.** None.

**Failure-modes covered.**
- **F8 (Watcher feedback-loop poisoning) at trait level** — m10 is the Ember gate itself; mitigates F8 by refusing PRs that fail Ember audit (per G3 § m10 contract).
- B4 blocker — m10's implementation is gated on Ember §5.1 amendment (Watcher's lane); pre-amendment, m10 ships in `Warning` mode; post-amendment, `Fail` is mandatory.
- AP-V7-06 (bidi-anchor unidirectional rot) — m10's audit can include bidi-anchor rot as a 7-trait fail-mode (specifically the "Internal Coherence" trait).

**Atuin trajectory anchor.**
- `wt-ember-gate` (proposed; CI invocation surface for m10).
- `ember-audit` (existing atuin script; m10 is its CI-gate counterpart).

**Watcher class pre-position.**
- **Class A** — first CI run with m10 in `Fail` mode post-amendment.
- **Class C** — confidence-gate refusal — m10's CI-FAIL is exactly Class-C (safe-path refusal).
- **Class D** — four-surface drift in the allowlist (if `ember_held_approvals.tsv` carries entries not mirrored in vault/CLAUDE.local.md/ai_docs).

**Bidi-04 closure** (per G3): m10 ↔ `tests/ember_held_approvals.tsv` (read at CI runtime); ↔ `~/projects/claude_code/Ember 7-Trait Gate Rubric.md` (canonical rubric source).

---

## m11 — decay (Gap 2 owner)

**Purpose.** Compute `DecayFactor = base + (1-base) × clamp(frequency × fitness × recency, 0, 1)` — the NEW PRIMITIVE compound decay formula (Gap 2 per CLAUDE.md § structural-gap authorship). Feeds m31 selector modulation and m30 sunset trigger.

**Upstream-IN** (per G3 § m11).
- `m7.last_run_at` per workflow (recency input — time-since-last-dispatch).
- `m14.frequency` per workflow (frequency input — dispatch-count over last 20 selection cycles).
- stcortex `pathway.weight` per workflow (fitness input — substrate-weight after Hebbian update).
- `m11.config.base: f64` (the floor; default 0.1 per Hebbian v3 reconciliation).
- `m11.config.recency_half_life_days: u32` (default 30; aligned with Phase 6 D120 sunset evaluation).

**Downstream-OUT.**
- `DecayFactor(f64)` ∈ [base, 1.0] → m31 selector (multiplicative modulation on `α·fitness + β·recency + γ·frequency + δ·diversity` composite), m30 bank (sunset trigger when `DecayFactor < sunset_threshold`).
- Prune-marker hint to m13 (write-side; m13 reads m11 for stcortex delete-marker emission).

**Aspect-IN.**
- m8 — compile-time gate.
- m9 — write-time prefix validation when m11 emits to stcortex (delete-markers).

**src/ path:** `src/m11_decay/` with `mod.rs`, `formula.rs` (the Gap 2 compound decay — the canonical NEW PRIMITIVE), `inputs.rs` (the `recency_factor`, `frequency_factor`, `fitness_factor` normalisations), `sunset.rs` (the `m30` sunset-trigger logic), `error.rs`.

**LOC budget** (per ULTRAMAP View 2): **~250 LOC** (per CLAUDE.md "Gap 2 ~200-300 LOC" target). The largest non-KEYSTONE module in the engine.

**Test budget** (per TEST_DISCIPLINE matrix row m11): **70 tests**.

**Test-pattern allocation.**
- F-Unit 35 — formula per-input-arm (base/freq/fit/rec); normalisation per input domain; sunset-trigger per threshold; recency half-life computation per Δt; multiplicative-modulation per fitness range; clamp behaviour per out-of-range input; base-floor behaviour.
- F-Property 10 — Gap 2 invariants: `decay ∈ [base, 1.0]` for all finite inputs; monotonic non-decreasing in `frequency`; monotonic non-decreasing in `fitness`; monotonic non-decreasing in `recency`; idempotent under `recency_half_life=0` (degenerate case); `base=1.0 → decay=1.0` always (sanity); `base=0.0` + zero inputs → `decay=0` (no negative); `recency` symmetric under timestamp-shift on stationary inputs.
- F-Fuzz 0.
- F-Integration 15 — m11 ↔ m7 read (`last_run_at`); m11 ↔ m14 read (`frequency`); m11 ↔ stcortex read (`pathway.weight`); m11 → m31 selector; m11 → m30 sunset; m11 → m13 prune-marker emit; multi-workflow concurrent decay.
- F-Contract 5 — `DecayFactor` newtype Display stability; sunset-threshold constant vs Phase 6 D120 spec; formula coefficient stability across spec versions.
- F-Regression 4 — Gap 2 formula regression slot; sunset threshold regression; recency half-life regression; F1 (bank/name ossification) regression (decay never approaches 1.0 indefinitely → forces sunset).
- F-Mutation 1 budget — ≥70% on `formula.rs` (the Gap 2 core).

**Mutation kill threshold:** ≥70%.

**Boilerplate-lift source.**
- Recency-half-life primitive — adapted from ME v2 telemetry decay (~50% reuse); generalised to the `recency_factor` axis.
- Frequency normalisation — adapted from POVM v2 `learning_health` magnitude-weighted formula (~30% reuse for the EMA-like cohort denominator).
- Fitness normalisation — adapted from RALPH fitness 12D tensor patterns in ORAC (~20% reuse — just the clamp(0,1) idiom).
- **Gap 2 compound formula** — fresh authorship (~120 LOC). No upstream equivalent: `base + (1-base) × clamp(f × g × r, 0, 1)`. This is the engine's structural primitive #2 per CLAUDE.md.

**Structural-gap LOC.** **Gap 2 ~200-300 LOC NEW PRIMITIVE** — per CLAUDE.md § structural-gap authorship. m11 is the exclusive owner of Gap 2.

**Failure-modes covered.**
- **F1 (bank/name ossification) — m11 prime mitigator at the lifecycle aspect.** Decay enforces hard sunset_at boundary by construction (per G3 § m11 contract); pairs with m30's sunset trigger.
- Gap-2-specific: decay never returns negative; decay never exceeds 1.0; decay never silently saturates (the `(1-base)` factor structurally bounds the upper region).
- AP-V7-09 (substrate-frame engine confusion) — m11's formula is the canonical substrate-frame measure; `fitness × frequency × recency` is operationalised (not anthropocentric) per AP-V7-09 mitigation.
- AP-Test-02 (property-test stub) — m11's 10 F-Property tests run ≥10k iters per invariant; the highest property-test count in the engine (matches Gap 2's structural weight).

**Atuin trajectory anchor.**
- `wt-decay-pulse` (proposed; samples m11's per-workflow `DecayFactor` for top-K least-decayed and bottom-K most-decayed; Phase 5C weekly synthesis input).
- `wt-sunset-watch` (proposed; lists workflows approaching sunset threshold).

**Watcher class pre-position.**
- **Class A** — first `compute_decay_factor` invocation post-Genesis.
- **Class D** — four-surface drift if m11's per-workflow `DecayFactor` diverges across stcortex/m7/CLAUDE.local.md weekly synthesis.
- **Class G** — substrate-frame confusion if `recency` is back-decoded as user-attention rather than substrate-weight time-since-update.
- **Class I** — Hebbian silence — m11's `fitness` input reads stcortex `pathway.weight`; if pathway weight never moves, m11's decay is stuck at `base × (1-base) × clamp(freq × 0 × rec, 0, 1) = base`. Class-I directly observable here.

---

## Cluster-level synergies (which CC-1..CC-7 Cluster D participates in)

Per G3 § Cross-cluster synergies:

- **CC-2 Trust Layer Woven (D → all) — Cluster D is THE source of CC-2.** Per G3 diagram:
  ```
  m8 (compile-time) ─┐
  m9 (write-time)   ├─── aspect-arrow ───► all m1-m7, m12-m15, m20-m42
  m10 (output-time) │
  m11 (lifecycle)   ┘
  ```
  Every other cluster receives CC-2 from Cluster D; D itself receives nothing (it's the aspect floor — m8 specifically has no upstream aspect).
- **CC-4 Proposal → Bank → Dispatch (F → G → Conductor).** m11 feeds m31 selector composite-score modulation (`α=0.40 fitness + β=0.25 recency + γ=0.20 frequency + δ=0.15 diversity`); m11's `DecayFactor` is the multiplicative modulator on `β` (recency) and `γ` (frequency). m30 reads m11 for sunset trigger.
- **CC-5 Substrate Learning Loop (G → H → back to F via stcortex pathways).** m11 reads stcortex `pathway.weight` as its `fitness` input; this is the substrate-grain edge per G3 § CC-5 (the unique substrate-relevant CC). m11's decay therefore moves in lock-step with substrate weight evolution; absence of substrate movement = m11 decay stuck = Class-I Hebbian silence escalation.
- **CC-7 Pressure-Driven Evolution (E → spec).** m11's threshold constants (base, recency_half_life_days, sunset_threshold) are the prime targets of m15 reservation events when Watcher/Zen audit observes systematic mis-decay. CC-7 closes at m11's config (alongside Cluster A's config) next-session.

Cluster D does NOT directly participate in CC-1 (Cascade-Cost — B internal), CC-3 (Evidence-Driven Iteration — E → F), or CC-6 (Verification-Gated Dispatch — G internal). It wraps them via CC-2 aspect-arrows but doesn't sit on the bidi flow.

---

## Cluster-level antipatterns (subset of ANTIPATTERNS_REGISTER relevant to Cluster D)

- **AP-Hab-03 (AP30 violation) — m9 exclusive enforcer.** m9 is the validator; m13/m42 are the call-sites.
- **AP-Hab-11 (hyphen-slug stcortex munge) — m9 prime owner.** Munge happens exactly once per write at m9 boundary.
- **AP-Hab-14 (god-tier dilution) — m8 prime defender.** Hard-fail discipline; never softens to a warning.
- **AP-WT-F1 (bank/name ossification) — m11 prime mitigator at the lifecycle aspect.** Decay formula structurally bounds re-promotion.
- **AP-WT-F7 (CR-2 graceful-degrade pretend-fix) — m8 exclusive owner.** Hard-fail when `learning_health` outside [0.05, 0.15].
- **AP-WT-F8 (Watcher feedback-loop poisoning) — m10 trait-level mitigator.** Ember audit is the cross-trait check.
- **AP-V7-01 (7-gen drift)** — Cluster D's invariants are the most likely surface for generation-N drift; each generation's Citation audit section must re-verify D's formula coefficients + band thresholds.
- **AP-V7-09 (substrate-frame engine confusion)** — Cluster D is the substrate-frame discipline incarnate. m11's `frequency × fitness × recency` is the canonical operationalised measure; m8/m9/m10 enforce non-anthropocentric outputs.
- **AP-Drift-01 (agent over-claim gate-clean against scoped clippy)** — m11's 70-test count + 10 F-Property invariants + mutation budget combine to make over-claim detectable at Wave-end.
- **AP-Drift-11 (supervisor stub mistaken for live) — m8 specific risk.** m8's runtime mirror probes live POVM; never inferred from process-list.

The cluster's overall risk surface is **aspect discipline rigour** — Cluster D is the cross-cutting trust regime; failure here corrupts every other cluster transitively. m11 specifically is the highest-LOC, highest-test-density module in the engine outside KEYSTONE m20 because Gap 2 is the engine's second structural primitive.

---

*cluster-D authored 2026-05-17 by Command (V7 author wave subagent)*
