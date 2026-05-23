# CHANGELOG — workflow-trace

All notable changes to the spec, structure, and decisions for `workflow-trace` are recorded here. Versioning is **spec-versioned** pre-G9 (no Cargo SemVer until first commit + `cargo check` green).

The format is loosely based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) but versions track binding-spec revisions, not code.

---

## [v0.2.0-WIP] — 2026-05-23 (S1004377) — Plan v2 Phase 1 (re-baseline + ADR cascade)

v0.2.0 execution opened per Luke @ node 0.A "begin V2" Phase 1 go (D48 execution
gate fired). Phase 1 is doc-only (decision-free per Plan v2 §3); no source code
touched. Engine at `v0.1.0` tag (`df00fd2`); tests 2048; clippy + pedantic clean;
mutation kill-rate 96.3% held.

#### Added (Phase 1)

- **ADR `D-S1002127-03` Amendment 1** (`ai_docs/decisions/2026-05-17-substrate-as-actor-deferrals.md`
  § 7) — registers **NA-GAP-01 (V1 RefusalToken) + NA-GAP-04 (V2 substrate
  back-pressure) + NA-GAP-06-drain (C1 m13 outbox drain consumer)** as
  **now-active v0.2.0 work-items** alongside the original NA-GAP-07/08/10
  deferrals. Per Phase 2 audit (S1004115) §2 recommendations + Plan v2 §2.5
  carry-overs. Cascade per C-8 step 2.5: per-phase responsibility (language
  updates co-located with implementation), not single Phase-1 sweep.
- **ADR `D-S1004XXX-04` NEW** (`ai_docs/decisions/2026-05-23-refusal-token-authorship-typing.md`)
  — V1 RefusalToken authorship-typing design spec. 4-variant outer enum
  (`SubstrateAuthored / EngineAuthored / OperatorAuthored / Unavailable`) per
  DX-1 lock. 3-variant `Unavailable` sub-tag (`EngineImagined /
  SubstrateUnreachable / SubstrateAuthored`) per DX-V5.b lock + NA-5
  recommendation. Call-site classification table for 7 emit-sites. Phase 5
  co-land with W1 (one wire-contract regen pass per C-2). Phase 7 call-site
  audit + drain wire. SemVer-break per DX-W.c — v0.1.0 `proposals.jsonl` files
  do not deserialise; migration note will ship with v0.2.0 § "Changed".

#### Changed (Phase 1)

- **Phase 1 step 3 `mutation-weight` source clarification** — DX-W3.src locked
  as "variant.mutation count"; FP-verified `WorkflowVariant.mutation: MutationKind`
  is an enum at `src/m21_variant_builder/mod.rs:47`, not an integer count.
  Phase 5 W3 will derive weight via a small `mutation_weight_for(kind:
  MutationKind) -> u32` classifier consuming the MutationKind variant. The
  D10 metric `step-count × mutation-weight` stands; the source is
  `variant.mutation` and the classifier is W3's contribution. `grep -rn
  "mutation_weight" src/` returns one hit — a comment placeholder at
  `src/orchestration/dispatch.rs:555` ("budget projection (per D10 metric)").
  No primitive exists yet; Phase 5 W3 authors it.
- **§2 file:line re-verify (per C-1 fold-in)** — `pub enum RefusalReason`
  re-verified at `src/m32_dispatcher/mod.rs:228` (Plan v2 §2.1 V1 citation
  was already corrected from v1 DRAFT's `:163`); `WorkflowVariant.mutation`
  field confirmed at `src/m21_variant_builder/mod.rs:47`; `MutationKind`
  variants begin at line `:55` (`Identity / Swap{..} / ...`).

#### Resolved (Phase 1)

- **C-1 (citation drift)** — all §2 file:line anchors re-verified at HEAD.
- **C-5 (mutation-weight source unverified)** — pinned: variant.mutation
  (MutationKind enum) → classifier function (Phase 5 W3 authors).
- **C-8 (ADR amendment cascade)** — Amendment 1 § 7.3 names per-phase
  responsibility; cascade discipline is co-located with implementation, not
  swept in Phase 1.
- **Phase 1 step 7 stcortex genesis memory** — read-back-verified mem **id 18511**
  in namespace `workflow_trace_v020_s1004377` (already landed at v2
  ratification persist S1004377; Phase 1 confirms presence + writes a
  Phase-1-specific follow-up memory).

#### Phase 1 done-evidence (per Plan v2 §15 D43)

- **Gate (4-stage, all green):**
  - `cargo check --all-targets --all-features` ✅
  - `cargo clippy --all-targets --all-features -- -D warnings` ✅
  - `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic` ✅
  - `cargo test --all-targets --all-features --release` ✅ **2048 passed / 0 failed / 1 ignored across 38 suites** (matches v0.1.0 baseline; +0 delta)
- **Test-count delta:** +0 (doc-only phase as expected)
- **Cargo-mutants:** N/A (no source code change)
- **stcortex:** Phase 1 progress memory written + read-back-verified — mem **id 18517** in namespace `workflow_trace_v020_s1004377` (parent_ids = [18511]) per Plan v2 §13 + NA-6 discipline
- **Build warnings (out of Phase 1 scope):** spacetimedb-sdk vendored upstream emits 3 `try_next` deprecation warnings (handed by DX-CI Option A submodule per Plan v2 §15; upstream fix tracked separately); `build.rs` emits 3 expected POVM CR-2 warning lines per `D-S1001982-01` design (POVM_CR2_DEPLOYED env-flag not set; downstream m8 dormant POVM-gate is KEEP-DORMANT per Plan v2 R4)

---

## [v0.1.0] — 2026-05-23 (S1004115) — M0 / Workflow-Trace Completion Plan v2

Completion Plan v2 (S1004115) — closed all outstanding tasks → v0.1.0 / M0 tag. The plan
ran ten phases (Phase 4 was the locked-decision interview, executed S1004115 with 48
decisions locked); Phases 1, 2, 3, 5, 6 (six sub-phases 6a–6f), 7, 8, 9, 10 shipped as
named commits on `main`. Per § 15 D30, the prior `v0.1.0-s1003733` working tag is
superseded by this clean `v0.1.0`; tests **1967 → 2043+** (final count = STAGE 4 of the
M0 ship gate; recorded below), clippy + pedantic clean every phase, mutation kill-rate
held at **≥ 96.3 %** per § 15 D28.

#### What M0 / v0.1.0 certifies (per Plan v2 § 8, the named cut)

> **`v0.1.0` / M0 certifies engine-internal completeness:** every residual the engine
> owns is closed, tested, audited, and documented. It does **NOT** certify the
> engine's safety as a substrate-facing organ — substrate-drift detection,
> substrate-side test fixtures, and substrate-mediated trust are absent by deliberate
> deferral (`NA-GAP-07/08/10`, ADR `D-S1002127-03` + amendments per Phase 2 audit
> recommending NA-GAP-01 and NA-GAP-04 also land in v0.2.0). The tag tells the
> habitat "the engine is done to a milestone." It does not tell the habitat "the
> engine is safe to run continuously against a live substrate." That assurance is
> `v0.2.0`.

#### Added

- **Plan v2 ten-phase execution** (canonical: `ai_docs/WORKFLOW_TRACE_COMPLETION_PLAN_V2_S1004115.md`).
- **Phase 1 doc set:** `PHASE1_RESIDUAL_LIST_S1004115.md` (authoritative successor to the
  `HARDENING_FLEET_CARRY_FORWARD_S1002600.md` carry-forward).
- **Phase 2 audit doc:** `PHASE2_AUDIT_S1004115.md` (wire-contract verification, 8-NA-gap
  code audit reframing "8/11 closed" → 3 code-backed + 2 partial + 3 spec-only).
- **Phase 8 integration doc:** `PHASE8_INTEGRATION_S1004115.md` (env matrix + clock-coherence
  enumeration of the 6 CC-5 loop clocks + Frame-A self-check on the substrate-load proxy).
- **Phase 9 audit fold-in doc:** `PHASE9_SD_RECONCILIATION_S1004115.md` (SD1–SD12 disposition
  + honest residuals consolidated).
- **CI machinery (D29):** `.github/workflows/ci.yml` + `.gitlab-ci.yml` running the
  canonical 4-stage gate. **Known M0 limitation** documented in-file: the
  `spacetimedb-sdk` sibling-repo path-dep makes standalone-checkout CI non-trivial; v0.2.0
  resolution by vendor / submodule / crates.io.
- **m33 four-named-verifier set:** Phase 6a `SecurityVerifier` (D5/D6/D7), Phase 6b
  `EmberVerifier` (D13/D14/D15/D16 — scores via m10's 7-trait rubric per D15), Phase 6c
  `CostVerifier` documented stub (D9), Phase 6d `ConsistencyVerifier` documented stub
  (D11). `ConservativeVerifier` retired from the production builder.
- **m9 ↔ m32 `AcceptanceSignatureReader` trait seam** (Phase 6e — gap C-8 / NA-GAP-11
  fold). m9 namespace guard reads the EscapeSurfaceProfile capability table through the
  trait; new `NamespaceViolation::CapabilityNotAcknowledged` typed refusal.
- **Substrate-confirmable verdict receipts** (Phase 6f — D8 + NA-GAP-09 fold):
  `NexusEventKind::WorkflowRefused` + `RefusalReceipt` payload + `emit_refusal_receipts`
  helper. Operator-visible `nexus_refusal_emit_failures` counter.
- **CC-7 PressureEvent → m23 wire** (Phase 7 — D21–D24): m15
  `read_pressure_level()` + `pressure_scalar_from_count()` feeds m23
  `compose_proposals_with_pressure` via additive-bounded
  (`MAX_PRESSURE_CONTRIBUTION = 0.5`) modulation; CC-7 no longer a dead edge.
- **m22 K-means CLI wiring** (Phase 5 — D17–D20): `extract_variant_features` 5-dim
  feature vector (step-count-norm, mutation one-hot ×3, Levenshtein-from-identity proxy
  per D17); `recommended_k_for_variant_count` adaptive `k = sqrt(n/2)` clamped per D19;
  `build_variant_cluster_map` rewires `compose_proposals` from `|_v| None` to a real
  variant→cluster lookup; `diversity_cluster` emitted via the JSONL bridge.
- **NA-4 Conductor enforcement-state assertion** (Phase 8 step 2): `wf-dispatch`
  warns when `CONDUCTOR_ENFORCEMENT_ENABLED` is unset or `"0"`; flag exposed on
  `Report::conductor_enforcement_advisory`.
- **NA-2 substrate-side load observation** (Phase 8 step 3): m1 atuin ingest timed
  (`m1_read_latency_ms` + `m1_read_perturbation_observed` on the report); honestly
  labelled as a Frame-A engine-timed proxy per § 15 D37.
- **MUT-2 unit-test kill** (Phase 3): two new direct unit tests
  (`project_after_prefix_full_embedding_*` + `project_after_prefix_gap_restart_*`)
  with exact-value `(suffix, right_gap)` assertions covering the `embed_from`
  `==` base case. Caught the `==` → `!=` mutation that prior `is_some()`-only
  tests missed.
- **T4-LIB**: `self_dispatch_guard` re-exported from `lib.rs`.

#### Changed

- **Mutation kill-rate truth arc** (W4 → Wave G → S1003733-final):
  - W4 commit message ("412 mutants / 80.6 %") was unreproducible — self-detected.
  - W4-verify (`046e955`) reset to 324 mutants / 254 caught / 94.4 % (artefact-backed).
  - Wave G (`c0ec95c`) killed 5 of the 15 W4-close survivors and proved 9 m21
    `build_variants` survivors equivalent (defense-in-depth via
    `MAX_LOOP_ITERATIONS` + redundant `out.len()` guard renders the mutated
    operators unobservable).
  - Final (`2096fd0` + S1004115 fold): **324 mutants — 259 caught / 10 missed / 0
    timeout / 55 unviable → 96.3 % kill rate**; 10 survivors all proven-equivalent
    (9 m21 `build_variants` + 1 m22 `kmeans_plus_plus_seed:310` FNV-collision-required)
    with in-code `// mutant-equivalent:` proofs.
- **m33 SD reconciliation** (Phase 9 — D27): 8 Class-A/B drifts reconciled (code is
  canonical; spec-doc amendments are documentation-only follow-up); 4 Class-C drifts
  (SD8/9/10/11) deferred to v0.2.0.
- **Doc supersession chain**: `HARDENING_FLEET_CARRY_FORWARD_S1002600.md` superseded by
  `PHASE1_RESIDUAL_LIST_S1004115.md` (Phase 1 DOC-2); vault `Hardening Fleet 2026-05-21.md`
  + `Assessment Remediation S1003733.md` W4 rows folded to 259/96.3 % (Phase 2 DOC-3 fold).

#### Audit

- **In-session zen verdicts** (per § 15 D25/D26 — substitute = in-session `zen` agent
  because no external Zen verdict file landed for any workflow-trace wave):
  - Phase 1: APPROVE-WITH-NITS (folded in Phase 2)
  - Phase 2: APPROVE
  - Phase 3: APPROVE
  - Phase 5: APPROVE-WITH-NITS (A2 — m31 caller still uses `|_w| 0.0`; substrate-side
    diversity contract satisfied, consumer-side wire is v0.2.0)
  - Phase 9 hardening-campaign audit: **APPROVE-WITH-NITS, recommend ship v0.1.0
    as-is.** Per-commit verdicts on all 13 audited commits (W1/W2/W3/W4/Wave-G/W5 +
    S1003733 + C22 + Phases 6e/6f/7/8 + final mutation fold) returned APPROVE or
    APPROVE-WITH-NITS. Honest residuals consolidated in
    `PHASE9_SD_RECONCILIATION_S1004115.md` § 4. Recommendation: add m33 doc-test
    locking `workflow_escape_surface` to Sandboxed-pending-wire-contract — **landed
    this commit** as `security_verifier_workflow_escape_surface_locked_to_sandboxed_pending_wire_contract`.

#### Resolved

- R1 m33 dispatch verifier policy logic (split 6a–6f).
- R2 m22 K-means diversity CLI wiring.
- MUT-2 m20_prefixspan `==` survivor.
- T4-LIB `self_dispatch_guard` re-export.
- m9-TODO m9 ↔ m32 trait seam (Phase 6e).
- CC-7 / H5 dead edge (Phase 7 wires).
- Doc surfaces: DOC-1 stcortex S1003733 memory (Phase 1 read-back-verified id 18410);
  DOC-2 carry-forward supersession; DOC-3 CHANGELOG / vault W4 figures.

#### Honest residuals — v0.2.0 candidates

Per Plan v2 § 11 + Phase 9 § 4 (named, not silenced):
- NA-GAP-01 `RefusalToken`, NA-GAP-04 substrate back-pressure, NA-GAP-07 substrate-drift
  canary `m16`, NA-GAP-08 substrate test fixtures, NA-GAP-10 substrate-mediated trust.
- Phase 5 nit A2: m31 production caller `|_w| 0.0` (m22 diversity consumer-side wire).
- SD8/9/10/11 Class-C algorithmic / shape upgrades.
- m33 Security M0 default surface = Sandboxed (gate shape correct; per-workflow
  surface determination is v0.2.0).
- R3 m22 K-means CLI batch-path tests, R4 m8 POVM-gate (KEEP-DORMANT) — see
  `the-workflow-engine/CLAUDE.local.md`.
- `wf-dispatch --execute` live-Conductor verification is post-M0 dispatch soak (D34/D35/D36;
  Watcher ☤ carries the 24h NoOp soak).
- CI `.github/workflows/ci.yml` + `.gitlab-ci.yml` ship with M0 per D29, but the
  `spacetimedb-sdk` sibling-path dep makes standalone-checkout CI non-trivial (vendor
  / submodule / crates.io resolution is v0.2.0).

#### Operator hand-off (Plan §3 Phase 10 step 8)

- **OP-1 / B3** — Conductor bring-up + 24h NoOp soak + flip
  `CONDUCTOR_ENFORCEMENT_ENABLED=1` per D33/D35.
- **OP-2 / G2** — directory rename `the-workflow-engine/` → `workflow-trace/` is
  post-M0 cosmetic per D32.

### [v0.1.0-s1003733] — 2026-05-22 (S1003733) — assessment remediation + binary wiring

Assessment-driven remediation. A god-tier 7-facet code-quality assessment scored
the-workflow-engine 80/100; Luke @ node 0.A directed "fix all identified issues". **21 of 22
findings closed + the C22 binary wiring + Wave G mutant-kill.** Commits `0cc7be3..ae7d460` on
`main`, pushed both remotes. Tests **1903 → 1967**, clippy + pedantic clean every wave.

#### Added
- **`wf-crystallise` + `wf-dispatch` are real CLI programs** (were one-line `println!` stubs).
  Pipeline logic lives in the new `workflow_core::orchestration` module (`crystallise` +
  `dispatch` sub-modules — each a `Config`, hand-rolled `parse_args`, and a `run()` driver);
  the binaries are thin `main()` wrappers. JSONL `WorkflowProposal` bridge crystallise→dispatch.
  `--offline` / `--dry-run` safe-default modes; graceful degradation when habitat services are
  down. 22 new integration tests for the lib↔binary seam.
- Operator/developer docs: `QUICKSTART.md`, `docs/COMMAND_MAPPING.md`, `docs/DIAGNOSTICS.md`,
  `ai_docs/API_MAP.md`, `ultramap/WF_{CRYSTALLISE,DISPATCH}_PIPELINE.md`.

#### Changed
- Documentation integrity: the W4 mutation-testing over-claim ("412 mutants / 80.6%",
  unreproducible) corrected to artifact-backed + independently re-verified numbers
  (324 mutants, 259 caught, 96.3% — post-Wave-G + S1003733-final; 10 surviving mutants all
  proven-equivalent with in-code `// mutant-equivalent:` proofs); the "W4 in progress /
  1835 tests" doc-drift swept.
- 8 contained code fixes: typed `MinerError::MaxLengthZero` (was silent coercion); `MaxGap`
  encapsulation; `DispatcherError::WireFormat` detail surfaced in `DispatchOutcome`;
  `KMeansError::NonFiniteCoordinate` variant (removed the `usize::MAX` sentinel); `unwrap_or(0)`
  audit; SEC3 `$PATH` fallback removed; `StcortexWriter::new` → `new_unchecked`.
- Core-type encapsulation (W3 #5–#10 portfolio): 6 representable-illegal-state holes closed —
  `Pattern` (KEYSTONE — `canonical_hash` can no longer desync), `WorkflowProposal` (F2-gated
  fallible constructor), `AcceptedWorkflow`, `NexusEvent.kind` → `NexusEventKind` enum, the
  `BatternId`/`CascadeClusterId`/`ChainId` newtypes, `WorkflowRunRow` → `RunState{Open,Closed}`.
- CC-4 diversity threaded through `compose_proposals`; CC-5 canonical `workflow_pathway_id`;
  `EscapeSurfaceProfile` acknowledgement gate made **monotone** (security MEDIUM —
  `FileWrite`/`NetworkEgress` no longer dispatch unacknowledged).
- m21 `build_variants` iteration cap (timeout mutants → bounded/catchable). m32/m22 oversized
  modules split into `tests.rs` siblings; m13 test-helper deduplication.

#### Resolved
- F2 m8 POVM-gate architecture decision → **KEEP-DORMANT** (dormant tripwire; static `build.rs`
  enforcement; no in-tree POVM read site post-m42-pivot).

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
