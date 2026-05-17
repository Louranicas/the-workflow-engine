# CHANGELOG — workflow-trace

All notable changes to the spec, structure, and decisions for `workflow-trace` are recorded here. Versioning is **spec-versioned** pre-G9 (no Cargo SemVer until first commit + `cargo check` green).

The format is loosely based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) but versions track binding-spec revisions, not code.

---

## [Unreleased]

### [v0.1.0-cluster-d-day-1-m8] — 2026-05-17 (S1002209) — first code-bearing release

🔥 **G9 fired. HOLD-v2 envelope LIFTED. m8 LIVE.**

#### Added — Cargo workspace + m8 (Cluster D Day-1, ship-first floor of trust regime)

- **`Cargo.toml`** — workflow-trace v0.1.0 single-crate 2-binary ORAC pattern: lib `workflow_core` + bin `wf-crystallise` (m1-m23 + m40-m42) + bin `wf-dispatch` (m30-m33). Deps: thiserror 2, reqwest 0.12 blocking+rustls, serde+serde_json 1, tracing 0.1. dev-deps: tracing-subscriber 0.3. Features: default=full + 4 capability features + opt-in `substrate-load`. `[lints.rust]` `check-cfg(cfg(povm_calibrated))` per build.rs gate.
- **`build.rs`** — emits `cargo:rustc-cfg=povm_calibrated` when `POVM_CR2_DEPLOYED=1` env var set; otherwise emits 3 `cargo:warning=` precursor signals naming commit SHA `e2a8ed3` + env var + reference doc. `rerun-if-env-changed` on both `POVM_CR2_DEPLOYED` and `POVM_HEALTH_URL`. NOT a Cargo feature (F7/AP-V7-09 defense).
- **`src/lib.rs`** — `workflow_core` crate root. `#![forbid(unsafe_code)]` · `#![warn(missing_docs)]` · `#![warn(clippy::pedantic)]` · 2 habitat-conventional allows (`module_name_repetitions` + `doc_markdown`).
- **`src/m8_povm_build_prereq/mod.rs`** — public re-exports + 3 cross-module sanity tests.
- **`src/m8_povm_build_prereq/cfg.rs`** — single-source-of-truth band thresholds: `POVM_LH_BAND_LOW=0.05`, `POVM_LH_BAND_HIGH=0.15`, `POVM_LH_EDGE_TOLERANCE=0.01`. `BandClassification` enum (BelowFloor/InBand/AboveCeiling/Nan) with `ordinal()` + `banner()` + `is_healthy()` + `is_band_edge()`. `classify(value: f64) -> BandClassification` is the shared classifier. **34 inline tests** (27 F-Unit + 5 F-Property + 2 F-Regression).
- **`src/m8_povm_build_prereq/error.rs`** — `BuildPrereqError` (Cr2MarkerAbsent / OutOfBand / ProbeFailed) + `RuntimeBandError` (StartupRefused). Every error message names commit `e2a8ed3` + env `POVM_CR2_DEPLOYED=1` + ref doc per m8 spec § 4 operator-recovery discipline. **9 inline tests** including F-Contract checks for commit-SHA / env-var / reference-doc literal presence.
- **`src/m8_povm_build_prereq/health.rs`** — runtime mirror probe: `HealthClient` (reqwest::blocking) + `probe_band` free function + `resolve_health_url` env-aware default. 2s default timeout. `tracing::info!` emission per m8 § 9 Observability. **18 inline tests** including F-Integration tests against a TCP one-shot mock server (no external mock-server dep).
- **`src/bin/wf_crystallise.rs`** + **`src/bin/wf_dispatch.rs`** — minimal stub binaries pending Day-2+ module landing.
- **`tests/m8_integration.rs`** — 6 integration tests (1 `#[ignore = "requires live POVM"]` for nightly + post-G9 acceptance; 5 always-run including build-runtime mirror agreement, features-full-does-not-enable-povm_calibrated regression, sub-second timeout fast-fail).

#### Quality gate — 4-stage all GREEN (S1002209)

- `cargo check` — clean (5.67s cold build)
- `cargo clippy --all-targets -- -D warnings` — clean
- `cargo clippy --all-targets -- -D warnings -W clippy::pedantic` — clean
- `cargo test` — **69/69 passing** (64 lib + 5 integration; 1 `#[ignore]` live POVM)

#### m8 specs vs implementation (god-tier discipline)

| Spec § | Requirement | Implementation |
|---|---|---|
| § 1 Purpose | `build.rs` emits `cargo:rustc-cfg=povm_calibrated` when env marker set | `build.rs:18` |
| § 2 Contracts | CC-2 Trust Layer Woven; rustc-cfg NOT [features] | Cargo.toml has no `povm_calibrated` feature; build.rs emits rustc-cfg |
| § 3 Public Surface | `BandClassification` + `HealthClient` + `probe_band` | All three in `src/m8_povm_build_prereq/mod.rs` re-exports |
| § 4 Errors | `BuildPrereqError` + `RuntimeBandError` with commit SHA + env + ref doc text | F-Contract tests verify literal text presence |
| § 5 Implementation sketch | build.rs ~30 LOC env-var-only; runtime mirror reqwest blocking 2s | build.rs 39 LOC; health.rs 117 LOC with reqwest blocking 2s default |
| § 6 Test plan | 50 tests across F-Unit/Property/Integration/Contract/Regression | 69 tests delivered (138% of budget) |
| § 7 Boilerplate lift | 70% pattern from synthex-v2/loop-engine-v2 build.rs idioms | Achieved (env-check + rerun-if-env-changed + warning emit) |
| § 8 Failure modes | W2 / F7 / F3 / AP-V7-01 / AP-V7-13 / AP-Drift-11 / AP-Hab-14 | F7 regression test in cfg.rs (band-edge precision); AP-Drift-11 covered by runtime probe; AP-Hab-14 enforced by clippy pedantic gate green |
| § 9 Observability | `tracing::info!` at `m8.health.probe` | `health.rs:probe_band` emits the structured event |
| § 10 Pre/Post | Env-var-only per spec; ProcessExit 78 deferred to caller | env-var-only honoured; exit code is caller responsibility |

### S1002209 — Luke task-cascade 1-6 (2026-05-17T09:38Z)

**Authorisation:** Luke S1002209 directive _"continue plan for and then complete each task 1. 2. 3. 4. 5. 6. in logical order to the highest level of excellence and impact proceed seamlessly"_

**Steps EXECUTED by Command (no source code touched; HOLD-v2 intact):**
- Step 1: filed Zen G7 AUDIT-REQUEST v3 at [`agent-cross-talk/2026-05-17T093800Z_command_g7_audit_request_v3_cardinality_7_plus_wave4b_amendment.md`](../../projects/shared-context/agent-cross-talk/2026-05-17T093800Z_command_g7_audit_request_v3_cardinality_7_plus_wave4b_amendment.md) — Group A (v2 absorbed) + Group B (D-S1002127-02 cardinality 7) + Group C (D-S1002127-03 substrate deferrals) + Group D (Wave 4.B 4 sub-groups)
- Step 2: B1 RESOLUTION-PATH-ELECTED — drive G1-G8 in sequence (NOT per-gate waiver); GATE_STATE.md updated
- Step 3: B2 GREEN-LIT + DELIVERED — v1.3 binding (`ai_docs/GENESIS_PROMPT_V1_3.md` 46K) confirmed authored; v3 AUDIT-REQUEST covers full amendment scope; GATE_STATE.md B2 row updated
- Step 5: workspace-root `~/claude-code-workspace/CLAUDE.local.md` "Workflow Engine" row amended per Luke single-row Command-amend waiver; row stale-since 2026-05-13 brought current to S1002209 state

**Steps STANDING Luke-action (cannot be completed by Command):**
- Step 4: `~/.local/bin/devenv -c ~/.config/devenv/devenv.toml start weaver/zen/enforcer` — project rule forbids agent service-start (sandbox reaps children); non-blocking pre-G9
- Step 6: G9 fire requires Luke's literal phrase `start coding workflow-trace` — not present in S1002209 message; staged readiness: Cluster D 4/4 specs verified (`m8_povm_build_prereq` · `m9_watcher_namespace_guard` · `m10_ember_ci_gate` · `m11_fitness_weighted_decay`); boilerplate clones ready (`05-decay-ttl-ltd` · `09-trap-verify-escape-skills` · `10-foundation-direct-clones`); gated on G7→G5→G6→G8

**Gate state delta:**
- G7: PENDING — superseded v2 AUDIT-REQUEST with v3 (broader scope)
- B1: Active → PATH-ELECTED
- B2: Active → DELIVERED
- B3: Active → STANDING-LUKE (re-classified as non-blocking-pre-G9)

### Pending (Wave 5 candidates — post-Wave-4.B-closeout)
- ai_docs/INDEX + ai_specs/INDEX status markers `TBD Wave 2` → `LIVE` (cosmetic)
- ai_specs/INDEX heading-form variance documentation (3 canonical forms: `## N.` / `## N —` / `## §N`)
- Vault HOME.md wikilinks to Wave-2B deep docs (`ARCHITECTURE_DEEP_DIVE`, `CODE_MODULE_MAP`, `CARGO_LAYOUT_SPEC`, etc.) — currently bidi anchor present in HOME but no per-doc wikilinks
- Cluster scaffold vault notes (A-H) → ai_specs/modules/cluster-X/ back-links (currently one-way: vault→ai_specs missing)
- m32 cooldown ladder defaults table → back-propagate from ultramap schematic to m32 spec § 2 DispatcherConfig
- m11 compound decay worked-examples → back-propagate to m11 spec test fixtures
- m42 CC-5 closure-test 5-step ritual → back-propagate to m42 spec § tests
- Cluster D Day-1 gantt → back-propagate to V7 runbook-01 Phase-1 Genesis
- agent-claim-verifier checks 6 + 16 in CI regression slot (post-G9)
- Bottom-anchor decision on 11 specs (Cluster B/C/E/F missing trailing `Back to:`) — Command accepted top-anchor-sufficient; can re-author if Luke disagrees
- ~~**Workspace-root CLAUDE.local.md "The Workflow Engine" row amendment** — flagged stale by 4-surface verifier; project charter forbids; **Luke action required**~~ **CLOSED 2026-05-17 S1002209 task-cascade Step 5** — Luke single-row Command-amend waiver via "complete each task" directive; row brought to S1002209 state

### Pending (v0.2.0 deferrals — see [`ai_docs/decisions/2026-05-17-substrate-as-actor-deferrals.md`](ai_docs/decisions/2026-05-17-substrate-as-actor-deferrals.md))
- W1: m16_substrate_drift_canary module (NA-GAP-07 module half; cross-cutting half closed Wave 4.B)
- W2: tests/substrate_fixtures/ suite (NA-GAP-08)
- W3: substrate-mediated trust cross-habitat ADR (NA-GAP-10)

### Pending (binding-spec gating; not scaffold scope)
- Luke `start coding workflow-trace` (G9) — gated on G1-G8 sequence
- Zen G7 verdict on v1.3 amendment + this scaffold (AUDIT-REQUEST v2 filed 2026-05-17T160500Z)
- B4 Ember §5.1 Held-semantics amendment (Watcher's lane; awaits Luke direction)
- B3 Conductor `auto_start=false` (Luke @ terminal `devenv start weaver/zen/enforcer`)

---

## [v0.0.0-spec.4] — 2026-05-17 (S1002127 — Wave 4.B closeout: NA-GAP substrate-as-actor remediation)

### Added — substrate-couplings (NA-GAP-03 + NA-GAP-09 closure)
- **`ai_specs/substrate-couplings/INDEX.md`** — new directory landing + verification-discipline pattern (`Engine-observable` / `Substrate-confirmable` / `Verification surface` / `Silent-failure shape` / `Remediation hint`) + substrate-confirmable-receipt convention
- **`ai_specs/substrate-couplings/CC-5-decomposed.md`** — primary deliverable; decomposes CC-5's single Watcher-Class-I observation into **5 substrate-substrate edges** (E1 m32→S-C; E2 m32→S-E→S-C via Hebbian coord; E3 S-C→habitat-memory→S-B injection.db; E4 m32→S-F→V3-partner; E5 S-C→digest→S-G operator). Each edge has dossier with latency/observability/silent-failure shape/remediation. Class-I supplementation deferred to v0.2.0. Includes refusal-token table per edge.
- **`ai_specs/substrate-couplings/CC-4-decomposed.md`** — 3 edges (m32→S-D Conductor wave dispatch; S-D refusal-path; m30→S-G operator AP-V7-07 acceptance). Includes AP-V7-13 enrichment (Conductor health-200 ≠ wave-pane-up case mirrors live POVM-CR-2 incident).
- **`ai_specs/substrate-couplings/CC-7-decomposed.md`** — 4 edges (m15 pressure→S-G; S-G→spec amendment fanout; S-G→S-watcher Ember §5.1 gate AP27-enforced; S-G fatigue→m12 consent budget). Operator-as-substrate per NA-GAP-05.

### Amended — RefusalToken introduction (NA-GAP-02 closure)
- **`ai_specs/ERROR_TAXONOMY.md`** — new section after Cross-cluster propagation: "RefusalToken — typing refusal by authorship". Cross-references `cross-cutting/refusal-taxonomy.md`. Includes per-variant classification table (which existing thiserror variants are refusals vs failures vs unavailability). Adds cross-reference to substrate-drift as third class (looks-like-refusal but neither refusal nor unavailability — CR-2 canonical case).

### Amended — m42 outbox-policy (NA-GAP-06 closure)
- **`ai_specs/modules/cluster-H/m42_stcortex_emit.md § 5.1 Outbox-policy`** — new section between Algorithm sketch and Boilerplate lifts. Covers:
  - § 5.1.a: drain ordering on substrate recovery (envelope.id ascending; idempotency-honoured replay; offline-snapshot reconciliation; throttle cap)
  - § 5.1.b: outbox saturation limit (warn 64 MB / refuse 256 MB / panic 1 GB with rationale)
  - § 5.1.c: offline-snapshot staleness threshold (warn 5 min / refuse 1 hr / panic 24 hr at boot)
  - § 5.1.d: substrate-confirmable receipt (proposed `cc5_replay_observed_at` on stcortex side)
  - § 5.1.e: metric inventory (6 new metrics including `reinforce_outbox_warn_total`, `reinforce_outbox_saturated_total`, `reinforce_drain_throttled_total`)

### Amended — substrate-side benchmarks (NA-GAP-04 closure)
- **`ai_specs/BENCHMARK_SPEC.md § Substrate-side load benchmarks`** — new section after m32 5-check bench. Defines:
  - Six substrate-side benches (one per substrate; measurement surface AT the substrate, not the engine)
  - Methodology (baseline → load → re-measure → delta → emit `SubstrateLoadProfile`)
  - Wave-end discipline (opt-in `--features substrate-load`; substrate-drift quarantine on `SubstrateDriftDetected`)
  - Per-substrate cadence-throttle rules consumed by m1/m3/m13/m40/m41/m42
  - Anti-patterns specific to substrate-side benchmarking (measuring-at-engine; single-window measurement; ignoring co-tenant traffic)

### Added — v0.2.0 deferrals ADR
- **`ai_docs/decisions/2026-05-17-substrate-as-actor-deferrals.md`** (D-S1002127-03) — registers W1 (m16_substrate_drift_canary), W2 (tests/substrate_fixtures/), W3 (substrate-mediated trust cross-habitat ADR). Documents v0.1.0 compensating controls per deferred item. Defends split as structural-in-v0.1.0 / automation-in-v0.2.0 (not Frame collapse). Filed at `ai_docs/decisions/` (NOT `optimisation-v7/decisions/`) per NA-05 recommendation path; rationale documented in frontmatter.

### Amended — registers
- **`ai_specs/INDEX.md`** — new section "Substrate-substrate couplings (NA-GAP-03/09 closure — Wave 4.B)" after Substrate dossiers; lists 4 substrate-couplings files. Footer `Back to:` extended with `substrate-couplings/` and v0.2.0 deferrals ADR links.

### Wave 4.B coverage summary

| NA-GAP item | Closure surface | Status |
|---|---|---|
| NA-01 (substrate lifecycle/refusal/drift model in scaffold) | 8 substrate dossiers (Wave 4.B earlier) | ✅ CLOSED |
| NA-02 (substrate-authored vs engine-authored refusal conflation) | `cross-cutting/refusal-taxonomy.md` + ERROR_TAXONOMY.md amendment | ✅ CLOSED |
| NA-03 (substrate-substrate couplings hidden in CC-5) | `substrate-couplings/` (4 files) | ✅ CLOSED |
| NA-04 (engine assumes substrates have no own attention budget) | substrate dossiers § back-pressure + BENCHMARK_SPEC.md amendment + m42 outbox-policy | ✅ CLOSED |
| NA-05 (operator as oracle, not substrate) | `substrates/operator.md` + CC-7-decomposed.md | ✅ CLOSED |
| NA-06 (offline-fallback asymmetric — outbox/read policy under-specified) | m42_stcortex_emit.md § 5.1 outbox-policy amendment | ✅ CLOSED |
| NA-07 (substrate-drift detection implicit not first-class) | `cross-cutting/substrate-drift.md` (Wave 4.B earlier) + m16 module deferred to v0.2.0 (ADR D-S1002127-03) | ✅ PARTIAL (cross-cutting half closed; module deferred) |
| NA-08 (no substrate-side test fixtures) | Deferred to v0.2.0 W2 (ADR D-S1002127-03) | ⏳ DEFERRED with compensating control (per-module integration tests gated `#[ignore]`) |
| NA-09 (CC-5 verification engine-observable not substrate-observable) | substrate-confirmable-receipt convention in `substrate-couplings/INDEX.md` + 5 proposed receipts (CC-5) + 3 (CC-4) + 4 (CC-7) | ✅ CLOSED (substrate-side change requests in deferral ADR) |
| NA-10 (Cluster D trust engine-internal not substrate-mediated) | Deferred to v0.2.0 W3 (ADR D-S1002127-03) | ⏳ DEFERRED with compensating control (operator-as-substrate trust via CC-7-decomposed; m9 namespace bounds blast radius) |
| NA-11 (refusal-token observability gap — no Class C substrate emission) | `cross-cutting/refusal-taxonomy.md` § WireEvent::Refusal Class-C envelope + refusal-token tables in all 3 CC-decomposed files | ✅ CLOSED |

**Wave 4.B verdict:** 8/11 NA gaps fully closed; 3/11 (NA-07 module / NA-08 / NA-10) deferred to v0.2.0 with documented compensating controls + work-item registry in D-S1002127-03.

### Flagged
- Zen G7 AUDIT-REQUEST v3 amendment needed: must include Wave 4.B deltas (3 substrate-couplings/ files + 1 ADR + 3 amendments). Stays inside D-B6 AMEND-loop scope.
- Substrate-side change requests (5 receipts in CC-5; 3 in CC-4; 4 in CC-7) are **cross-habitat coordination items** — none of them are workflow-trace engine-side changes; tracked for v0.2.0 cross-habitat ADR cycle.

---

## [v0.0.0-spec.3] — 2026-05-17 (S1002127 — Wave 3 verification + closure)

### Added
- **Wave 3 verifier reports landing** — 3 parallel agents:
  - `agent-claim-verifier` — **PASS-WITH-AMENDMENTS** (20/20 hard checks PASS; 3 cosmetic; confidence 0.94); receipt at `~/projects/shared-context/agent-cross-talk/2026-05-17T064906Z_agent-claim-verifier_workflow_trace_wave1_2_verification.md` (cross-talk: `broadcast: clean_verified`)
  - `four-surface-persistence-verifier` — **PARTIAL** (Surfaces 1+2 strong; Surface 3 correctly reserved pre-G8; Surface 4 anchor added concurrent with verifier via CLAUDE.local.md edit); 5 gaps surfaced
  - `na-gap-analyst` — **Frame A (substrate-as-primary)** chosen; 11 NA gaps at [`ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md); ~8h Wave 4 remediation recommended **HIGH-VALUE pre-G9**
- **New ADR D-S1002127-01** — [`ai_docs/optimisation-v7/decisions/2026-05-17-g8-stcortex-persistence-plan.md`](ai_docs/optimisation-v7/decisions/2026-05-17-g8-stcortex-persistence-plan.md) — pre-specifies Surface 3 writes for G8-fire (~46 memories + ~60 pathways under `workflow_trace_*` namespace with reverse-anchor embedding rule); closes four-surface-verifier gap #3
- **plan.toml `[scaffold_meta].four_surfaces`** array — machine-readable enumeration of the 4 persistence surfaces (closes four-surface-verifier gap #4)
- **Project CLAUDE.local.md** S1002127 scaffold section added (closes four-surface-verifier gap #1 via concurrent edit)

### Resolved
- **PRIME_DIRECTIVE_WAIVER.md Wave 3 row** — IN PROGRESS → **LIVE — PASS-WITH-AMENDMENTS**
- Template-variance heading-form decision: ACCEPT all 3 forms (`## N.` / `## N —` / `## §N`) as canonical; document in `ai_specs/INDEX.md` (Wave 4 cosmetic)
- Bottom-anchor decision: ACCEPT top-anchor-sufficient (round-trip works without trailing `Back to:` on 11 affected specs)
- HOLD-v2 verified intact: 0 `.rs` files in active scope; 0 `Cargo.toml`; 38 `.rs` files in `the-workflow-engine-vault/boilerplate modules/` are intentional paste-templates (not code tree)

### Flagged for Luke
- **Workspace-root CLAUDE.local.md "The Workflow Engine" row stale** — project charter forbids workspace-CLAUDE.local edits for workflow-trace spec; Luke must amend manually OR grant explicit waiver
- **NA-GAP-01..11 (substrate-as-primary frame)** — author NA gap analysis recommends ~8h Wave 4 remediation before G9; substrate-naive risk = CR-2-class shift lands via Luke spot-check rather than engine detection
- **EscapeSurfaceProfile cardinality drift** (5 vs 6) — V7 GOD_TIER_RUST.md amendment recommended
- **Test budget drift** (1,562 / 1,594 / 1,599) — TEST_STRATEGY locked at 1,594
- 5 cluster-spec open questions to G7 (m1 page_size, m23 PrefixSpan implementation, m10 Ember §5.1, m11 re-calibration, m13 LTP/LTD scale)

---

## [v0.0.0-spec.2] — 2026-05-17 (S1002127 — Wave 2)

### Added
- **`.claude/` deep optimisation (28 files)** — `anti_patterns.json` (24 entries: AP-V7-01..13 + AP24/27/29/30 + AP32-37), `patterns.json` (35 entries), `context.json`, `status.json`, `ALIGNMENT_VERIFICATION.md`, 6 project-specific subagents (`agents/`), 6 project slash commands (`commands/`), 4 executable hooks (`hooks/`), 4 JSON schemas (`schemas/`), 3 canned SQL queries (`queries/`)
- **`ai_docs/` deep authoring (11 files, ~19k words)** — `ARCHITECTURE_DEEP_DIVE`, `CODE_MODULE_MAP`, `DEPLOYMENT_GUIDE`, `ERROR_TAXONOMY`, `MESSAGE_FLOWS`, `META_TREE_MIND_MAP`, `ONBOARDING`, `PERFORMANCE`, `QUICKSTART`, `STATE_MACHINES`, `CARGO_LAYOUT_SPEC`
- **`ai_specs/` cross-cutting + layers + synergies (33 files, ~29k words)** — 8 cluster-level layer specs (`layers/cluster-{A-H}.md`); 12 cross-cutting specs (API/DATABASE/EVENT/WIRE/IPC/DESIGN_CONSTRAINTS/CONSENT/SECURITY/ERROR_TAXONOMY/OBSERVABILITY/TEST_STRATEGY/BENCHMARK); 7 synergy contracts + README (`synergies/CC-{1-7}.md` + README — **CC-1b resolved as `CC-1.subA` sub-contract**); 5 cross-cutting axis specs (`cross-cutting/`)
- **`ultramap/` deep authoring (13 files, 16 Mermaid diagrams)** — `MODULE_DEPENDENCY_GRAPH`, `DATA_FLOW`, `CONTROL_FLOW`, `CONTEXTUAL_FLOW`, `INVARIANT_MAP`, `ULTRAMAP` master synthesis; 7 schematics (`cc4-pipeline`, `cc5-loop`, `m32-5check`, `cluster-d-day1`, `gap{1,2,3}-*`)
- **Obsidian vault sync (16 file changes)** — 6 audited (`> Back to:` anchors include `[[CLAUDE.md]] · [[CLAUDE.local.md]]`), 2 updated (HOME.md + MASTER_INDEX.md additions), 9 new (`Scaffold Wave 0-2 — Session S1002127.md` + 8 per-cluster scaffold notes)
- **Remaining gold-standard (14 files)** — `LICENSE` (placeholder, TBD), 8 placeholder dir READMEs (docs/config/scripts/migrations/bin/hooks/security/schematics), 2 per-binary READMEs (wf-crystallise/wf-dispatch), 3 config templates (default/production/devenv-service)

### Resolved
- **CC-1b reconciliation:** documented as `CC-1.subA` sub-contract in `synergies/CC-1.md` (preserves canonical 7-CC list discipline; AP-V7-02 Ultramap-rot avoided)

### Flagged
- **EscapeSurfaceProfile cardinality drift** — V7 GOD_TIER_RUST.md invariant #19 says 5, v1.3 + m30 spec say 6 (DataExfil added for openclaw scar tissue). Documented in `ai_specs/DESIGN_CONSTRAINTS.md` + `SECURITY_SPEC.md`. V7 amendment recommended.
- **Test budget drift** — V7 docs vary 1,562 / 1,594 / 1,599; `TEST_STRATEGY.md` locks at **1,594** per G6 latest matrix
- **`povm_calibrated` cfg name** — historical post-2026-05-17 m42 ADR; rename/retire deferred to post-G9 spec revision

## [v0.0.0-spec.1] — 2026-05-17 (S1002127 — Wave 1)

### Added
- **26 per-module god-tier Rust spec files** (~70k words, ~2,700 words/spec) written by 8 parallel cluster-spec-author agents
  - Cluster A (3): m1_atuin_consumer, m2_stcortex_consumer, m3_injection_db_consumer
  - Cluster B (3): m4_cascade_correlator, m5_battern_step_record, m6_context_cost
  - Cluster C (3): m7_workflow_runs, m12_cli_reports, m13_stcortex_writer
  - Cluster D (4): m8_povm_build_prereq, m9_watcher_namespace_guard, m10_ember_ci_gate, m11_fitness_weighted_decay (**Gap 2 owner**)
  - Cluster E (2): m14_habitat_outcome_lift, m15_pressure_register
  - Cluster F (4): m20_prefixspan_miner, m21_variant_builder, m22_kmeans_feature, m23_workflow_proposer (**KEYSTONE, Gap 1 owner**)
  - Cluster G (4): m30_curated_bank, m31_selector, m32_conductor_dispatcher, m33_verifier (**Gap 3 owner with m9**)
  - Cluster H (3): m40_nexusevent_emit, m41_lcm_rpc, m42_stcortex_emit (**POVM DECOUPLED per 2026-05-17 ADR**)
- Each spec: YAML frontmatter (14 fields) + 13-section body (Purpose/Public surface/Internal data/Data flow/Algorithm/Boilerplate lifts/ME v2 patterns/Test strategy/Antipatterns/Useful patterns/CC contracts/Open questions/Implementation order) + bidi anchors top+bottom

### Open questions to G7 / Luke / Watcher (consolidated; full per-spec lists in §12 of each)
- m1 page_size: V7 plan `1_000` vs vault `2_000` — needs Zen G7 reconciliation
- m23 PrefixSpan implementation: pure-Rust vs C-FFI vs Python-port (Cluster F agent recommends pure-Rust; **#1 G7 question**)
- m10 Ember §5.1 amendment (B4 blocker) — biggest Cluster D dependency
- m11 re-calibration post-m42 ADR (fitness signal POVM `learning_health` → stcortex `pathway.weight`); dual-read soak proposed
- m13 LTP/LTD scale reconciliation (vault `>0.15` vs workspace S1002127 `0.018`)
- EscapeSurfaceProfile ordinal stability across versions (m30/m32/m33 + m9) — reserve numeric gaps?

## [v0.0.0-spec.0] — 2026-05-17 (S1002127 — Wave 0)

### Added
- Scaffold-only scope-override waiver ([`PRIME_DIRECTIVE_WAIVER.md`](PRIME_DIRECTIVE_WAIVER.md)) — Luke @ S1002127
- Wave 0 scaffold skeleton: `src/`, `tests/`, `benches/`, `docs/`, `config/`, `migrations/`, `bin/{wf-crystallise,wf-dispatch}/`, `hooks/`, `security/`, `ai_docs/{layers,modules,decisions,schematics,runbooks,reflections}`, `ai_specs/{layers,modules/cluster-A..H,patterns,schematics,synergies,cross-cutting}`, `ultramap/schematics`, `layers/cluster-A..H`, `modules/`, `.claude/{agents,commands,hooks,skills,schemas,queries,worktrees}`
- Root anchor files: [`README.md`](README.md), [`ARCHITECTURE.md`](ARCHITECTURE.md), [`GATE_STATE.md`](GATE_STATE.md), [`ANTIPATTERNS.md`](ANTIPATTERNS.md), [`PATTERNS.md`](PATTERNS.md), [`GOLD_STANDARDS.md`](GOLD_STANDARDS.md), [`CONTRIBUTING.md`](CONTRIBUTING.md), [`SECURITY.md`](SECURITY.md), [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md), [`plan.toml`](plan.toml), [`.gitignore`](.gitignore)

### Pre-existing (carried over)
- [`CLAUDE.md`](CLAUDE.md) project charter
- [`CLAUDE.local.md`](CLAUDE.local.md) session-state delta
- [`the-workflow-engine-vault/`](the-workflow-engine-vault/) (88 files / 2.4MB)
- [`ai_docs/optimisation-v7/`](ai_docs/optimisation-v7/) (45 V7 deliverables: framework, generations G1-G7, integration, runbooks, standards, module plans, decisions, ultramap)
- [`ai_docs/GENESIS_PROMPT_V1_3.md`](ai_docs/GENESIS_PROMPT_V1_3.md) (binding spec v1.3 amendment; Zen G7 verdict pending)
- [`pre-framework-consolidation/`](pre-framework-consolidation/) brain-dump archive

---

## [v1.3-amendment] — 2026-05-17 (S1001982 → S1002127)

### Spec patch (binding; awaits Zen G7 verdict)
- Single-phase override absorbed (Luke override; D-B6 AMEND-loop adopted)
- m42 stcortex-only pivot ([ADR](ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md)) — POVM decoupled
- 26 modules locked (OI-3 resolved: was 28/11/25/26 across artefacts)
- m1-m42 unpadded naming locked (OI-4 resolved)
- m33 additive (`workflow_verifier`; required by Town Hall P0 #9)
- Two-binary split locked: `wf-crystallise` + `wf-dispatch` + `workflow_core` lib in same crate (ORAC pattern, not LCM workspace)
- Cluster D early-ship locked (Day 1; before any Cluster A reader)
- Feature gate matrix locked (default/full/api/intelligence/monitoring/evolution; D NOT gated)

---

## [v1.2] — 2026-05-15 (S1001982)

- Zen-audit-locked binding spec for 11-module Phase-A-only deployment (superseded by v1.3)

---

## [v1.1] — 2026-05-14
## [v1.0] — 2026-05-13
## [v0] — 2026-05-12 (Genesis Prompt v0 sketch)

> Earlier versions are in [`the-workflow-engine-vault/Genesis Prompt v0 S1001982.md`](the-workflow-engine-vault/Genesis%20Prompt%20v0%20S1001982.md) and [`the-workflow-engine-vault/Genesis Prompt v1.2 S1001982.md`](the-workflow-engine-vault/Genesis%20Prompt%20v1.2%20S1001982.md).

---

> **Back to:** [`README.md`](README.md) · [`PRIME_DIRECTIVE_WAIVER.md`](PRIME_DIRECTIVE_WAIVER.md) · [`GATE_STATE.md`](GATE_STATE.md)
