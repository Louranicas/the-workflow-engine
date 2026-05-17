---
title: m15 — pressure_register (per-module spec)
date: 2026-05-17 (S1001982)
kind: per-module-spec · planning-only · HOLD-v2 active
status: SPEC (no .rs, no Cargo.toml, no cargo)
cluster: E — Evidence + Pressure
layer: L5
binary: wf-crystallise
feature_gate: [intelligence]
verb_class: emit
loc_est: 90
test_budget: 60 (G6 mutation kill ≥ 70%)
boilerplate_lift: ~30% (Cat 09 trap-classification + agent-cross-talk file-drop convention)
authority: Command · workflow-trace V7 optimisation · v1.3 binding
---

# m15 — `pressure_register` · forbidden-verb pressure witness

> **Back to:** [`../../INDEX.md`](../../INDEX.md) · [`../../MODULE_MATRIX.md`](../../MODULE_MATRIX.md) · [`../../../CLAUDE.md`](../../../CLAUDE.md) · [`../../../CLAUDE.local.md`](../../../CLAUDE.local.md)
>
> **Sister modules:** [m14](m14_habitat_outcome_lift.md) · [m15](m15_pressure_register.md)
>
> **Cluster spec (vault):** [[cluster-E-evidence-pressure]] (canonical at `the-workflow-engine-vault/module specs/cluster-E-evidence-pressure.md`)
>
> **V7 plan:** [cluster-E plan](../../../ai_docs/optimisation-v7/MODULE_PLANS/cluster-E.md) · [ULTRAMAP View 2 m15 row](../../../ai_docs/optimisation-v7/ULTRAMAP.md) · [G3 bidi-flow § Cluster E CC-7](../../../ai_docs/optimisation-v7/GENERATIONS/G3-bidi-flow.md)
>
> **Genesis spec:** [`ai_docs/GENESIS_PROMPT_V1_3.md`](../../../ai_docs/GENESIS_PROMPT_V1_3.md) § 1 (Cluster E row), § 2 (hard refusals — m15 trigger surface)
>
> **Framework references:** [`PATTERNS.md`](../../../PATTERNS.md) · [`GOLD_STANDARDS.md`](../../../GOLD_STANDARDS.md) · [`ANTIPATTERNS.md`](../../../ANTIPATTERNS.md) · runbook [`runbook-00-pre-genesis-gates.md`](../../../ai_docs/optimisation-v7/RUNBOOKS/runbook-00-pre-genesis-gates.md)

---

## §1 Purpose

m15 is the engine's **institutional immune system** against the Fossil's ancestor-rhyme failure mode: ambition-shaped codebases that drift past their chartered verb-set die. m15 detects forbidden-verb-pressure events — observable signals that someone is attempting to add a capability outside `wf-crystallise` / `wf-dispatch`'s chartered verbs — and emits a durable `PHASE-B-RESERVATION-NOTICE` JSONL file to `~/projects/shared-context/agent-cross-talk/` for Watcher ☤ and Zen to observe.

m15 is a **witness module**, not a gating module. It blocks nothing. It creates a durable record that scope-pressure happened, where it came from, and what verb / feature was being pushed. Verb class: `emit`. m15 IS the **CC-7 closure entry-point** — without it the engine has no way to surface its own constraint-pressure and spec drift becomes invisible (GAP-Bidi-02).

The decoupling from m14 is deliberate (per cluster spec § Intra-cluster): m14 lives on the read-path (m7 → m14 → m23 / m31); m15 lives on the side-band path (m32 / m20-m23 → m15 → agent-cross-talk/). Cross-coupling would create circular evidence-pressure feedback ("m14 refused n<20 → m15 emits pressure → spec relaxes n<20 → m14 emits with n<20 → bias"). They share a cluster and an L5 layer; they do not share state.

## §2 What is a forbidden-verb-pressure event?

Any observable signal — spec patch, agent report, cross-talk message, CLI verb proposal, session note, or user request — attempting to add a capability outside the chartered verb-set.

**Chartered verb-sets (per v1.3 §2 hard refusals):**

| Binary | Permitted verbs |
|---|---|
| `wf-crystallise` | read · correlate · record · emit reports · aggregate evidence · log pressure |
| `wf-dispatch` | bank · select · verify · dispatch (Conductor-only) |

**Forbidden verbs / surfaces (trigger surface for m15 — per v1.3 §2):**

- `recommend_*` — functions that recommend to a user without explicit request
- `auto_*` / `smart_*` — autonomous trigger without explicit invocation
- `rewrite_*` — modifying source files outside the `wf-*` binaries themselves
- `route_*` (Conductor-bypass) — routing decisions bypassing HABITAT-CONDUCTOR (P0 #3)
- `package_*` / `publish_*` — packaging without verification gate
- `optimise_*` — automated tuning without measurement gate
- `promote_*` — auto-promoting without human gate (unified into dispatch in v1.3)
- HTTP server / sidecar daemon (Hard Refusal per v1.3 §2; `feature = "serve"` post-D60 read-only excepted)
- POVM writes — workflow-trace POVM-decoupled per 2026-05-17 ADR (Appendix A); any proposed POVM write fires m15
- `use synthex_v2::*` — wholesale SYNTHEX v2 import (lift patterns, do not import)
- Self-modification of m46-m51 (Watcher substrate, AP27 hard boundary)

**Example pressure events (per cluster spec):**

1. Spec patch proposes `fn recommend_cascade_for_next_session()` on m23 → `RecommendVerb`.
2. Agent report describes auto-promote of workflows above 0.85 fitness → `AutoOrSmartVerb`.
3. Cross-talk request that m14 expose live-poll HTTP endpoint → `HttpServerSurface`.
4. Devenv batch proposal to add `workflow_engine_sidecar` daemon → `SidecarDaemon`.

Detection at v1.3 is **necessarily heuristic** (no runtime detection machinery yet). Primary surfaces: spec patches diff'd against `GENESIS_PROMPT_V*.md` / `cluster-*.md`; cross-talk files appearing under `~/projects/shared-context/agent-cross-talk/`; CLI verb proposals in `CLAUDE.local.md` / session notes; scanned at crystalliser sweep time. False-positive bounding lives in §11 test contract.

## §3 Edge contract (per G3 § m15)

**Upstream-IN**
- Forbidden-verb pressure (m32 dispatch attempts of non-allow-listed verbs).
- Scope-relaxation pressure (m20-m23 internal pressure to weaken invariants — e.g., relax `MIN_SAMPLE_SIZE < 20`).
- Spec-patch diff scans (cluster spec / Genesis prompt edits).
- Cross-talk file content scans (`agent-cross-talk/*.md` arrivals).
- CLI verb proposals lifted from session notes / `CLAUDE.local.md`.

**Downstream-OUT**
- JSONL one-event-per-file emit to `~/projects/shared-context/agent-cross-talk/`.
- Filename pattern: `PHASE-B-RESERVATION-NOTICE-{iso8601-date}-{session_id_prefix}-{event_id}.jsonl`.
- Each file contains exactly **one** `PressureEvent` JSON object on a single line.
- Atomic write (`tmp + rename`); no shared lock; append-only directory protocol.

**Aspect-IN**
- m8 build-prereq (compile-time `povm_calibrated` cfg — refused-to-build defence-in-depth).
- m9 namespace_guard (filename + content prefix discipline).

**Aspect-OUT**
- Watcher ☤ + Zen READ from `agent-cross-talk/` at their own cadence. m15 does **not** push. Silence from observers does NOT indicate the notice was missed.

**Failure mode mitigated**
- GAP-Bidi-02 — CC-7 closure missing without m15; the engine becomes blind to its own constraint pressure.
- AP-Hab-04 (preserve-list discipline) — scope-relaxation pressure events explicitly include "blanket-action attempted" patterns; surfaces S102-class incidents at planning time, not execution.
- AP-V7-08 (handshake dual-silence) — a `HandshakeSilence` PressureKind variant is emitted when an expected peer ack is missing past timeout, making the silence itself observable.

## §4 Public surface (spec, NOT compileable Rust)

```rust
// src/m15_pressure/mod.rs (SPEC — markdown only, not source)

/// A single forbidden-verb-pressure event observed by m15.
///
/// Schema version: 1.  Any schema change bumps `schema_version` and the notice
/// filename includes the version: `PHASE-B-RESERVATION-NOTICE-v{N}-...`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PressureEvent {
    pub schema_version: u32,                       // = 1 at v1.3
    pub id: u64,                                   // monotonic per engine instance
    pub detected_at: String,                       // RFC 3339
    pub session_id: String,                        // $CLAUDE_SESSION_ID or "unknown"
    pub forbidden_category: ForbiddenCategory,
    pub trigger_excerpt: String,                   // ≤ 512 chars, truncated with "…"
    pub source: PressureSource,
    pub proposed_feature: String,                  // human-readable summary
    pub violated_charter: CharterSection,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForbiddenCategory {
    RecommendVerb,
    AutoOrSmartVerb,
    RewriteVerb,
    RouteBypassConductor,
    PackageOrPublish,
    OptimiseWithoutGate,
    HttpServerSurface,
    SidecarDaemon,
    DeprecatedPovmWrite,   // workflow-trace POVM-decoupled per v1.3 Appendix A
    SynthexV2Import,
    HandshakeSilence,      // AP-V7-08 mitigation
    Other { description: String },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PressureSource {
    SpecPatch        { file: String },
    AgentCrossTalk   { from_session: String },
    CliVerbProposal  { proposed_command: String },
    SessionNote      { note_path: String },
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CharterSection {
    Crystalliser,   // wf-crystallise verb-set
    Dispatcher,     // wf-dispatch verb-set
    HardRefusal,    // engine-wide (HTTP, POVM-write, synthex import, etc.)
}

pub fn register_pressure(event: PressureEvent) -> Result<PathBuf, PressureRegisterError>;
```

The PressureKind cap is **4 variants in v1.3** (per cluster plan AP-V7-04 hard cap, mirrored by `ForbiddenCategory` whose breadth is structural rather than expressive — adding categories requires spec amendment + G7 re-audit). Variants are de-duplicated by `forbidden_category + trigger_excerpt`-hash within a 60-second window (per §5 de-dup filter).

## §5 JSONL one-event-per-file pattern (atomic emit · no shared lock)

**File naming:**
```
PHASE-B-RESERVATION-NOTICE-{iso8601-date}-{session_id_prefix}-{event_id}.jsonl
```
- `iso8601-date` — `YYYY-MM-DDTHH-MM-SSZ` (colons replaced with hyphens for filesystem safety)
- `session_id_prefix` — first 12 chars of `$CLAUDE_SESSION_ID`
- `event_id` — zero-padded 8-digit monotonic counter

**Location:** `~/projects/shared-context/agent-cross-talk/` (the shared-context filesystem layer — always available regardless of which 14 habitat services are healthy).

**Format:** JSONL — one `PressureEvent` per file, on a single line. This makes `grep`, `jq`, `rg` trivial; concatenation across files is `cat *.jsonl`. **Multiple events in a session produce multiple files** (one-event-per-file invariant; never append to an existing file).

**Atomic write protocol:**
1. Serialise event to JSON string.
2. Write to `{filename}.tmp` in same directory.
3. `fsync(tmp_fd)`.
4. `rename(tmp_fd, final_path)` — atomic on POSIX.
5. `fsync(parent_dir_fd)` — directory entry durability.

**No shared lock.** The one-event-per-file invariant means writers never contend; the rename atomicity provides the write-visibility barrier. Readers (Watcher, Zen) tail the directory with `inotify` / atuin `wt-pressure-tail` and see only fully-formed files.

**De-dup filter (60-second window):**
- A hash of `(forbidden_category, trigger_excerpt)` is kept in-memory for 60s.
- Repeat events within the window are coalesced: m15 increments a `count` field on the *next* emitted notice (carrying forward de-dup state in memory; the JSONL files themselves remain immutable).
- Window expiry → next event emits fresh notice. No retroactive editing of on-disk JSONL.

**Example notice file:**
```jsonl
{"schema_version":1,"id":1,"detected_at":"2026-05-17T14:33:02Z","session_id":"S1001999","forbidden_category":"recommend_verb","trigger_excerpt":"fn recommend_cascade_for_next_session(cluster_id: ClusterId) -> RecommendedWorkflow {","source":{"spec_patch":{"file":"cluster-F-iteration.md"}},"proposed_feature":"Auto-recommend cascade workflows for next session","violated_charter":"crystalliser"}
```

## §6 Structured tracing output (parallel observability channel)

m15 emits `tracing::warn!` for every detected event, following the ME v2 gold-standard logging pattern (`logging.rs`):

```
level=WARN module=m15_pressure_register event=forbidden_verb_pressure
       category=recommend_verb source=spec_patch file="cluster-F-iteration.md"
       session_id=S1001999 proposed_feature="Auto-recommend cascade workflows…"
```

This surfaces in the `wf-crystallise` daemon log and is visible to any log monitor without requiring the shared filesystem mount. The two channels — JSONL files + tracing — are **redundant on purpose**: filesystem unavailable → tracing carries the signal; tracing collector down → JSONL persists. Either channel surviving = the pressure event survives.

## §7 The `PHASE-B-RESERVATION-NOTICE` name (legacy continuity)

The term is preserved from the original phased architecture where m15 reserved features for a future Phase B activation. Under the single-phase override (v1.3) the notice is semantically "forbidden-verb-pressure noticed". The name is kept for:

1. Continuity with the feature-verification matrix row 30 ([[Modules Synergy Clusters and Feature Verification S1001982]]).
2. Recognisability for Watcher ☤ and Zen who already match against `PHASE-B-RESERVATION-NOTICE-*` glob.
3. Atuin script `wt-pressure-tail` already keyed on the prefix.

Renaming would require coordinated edits across Watcher consumer + Zen consumer + atuin script + spec rows + observer documentation. Not worth the cost; the name is a label, not a semantic constraint.

## §8 Observer contract (Watcher + Zen — read-only, pull cadence)

| Side | Behaviour |
|---|---|
| m15 (writer) | WRITES to `agent-cross-talk/` — shared filesystem, always available |
| Watcher ☤ | READS at its own cadence (~tick-driven); no confirmation back to m15 |
| Zen | READS at audit-cadence; no confirmation back to m15 |
| Silence | Does NOT indicate the notice was missed; m15 emits and forgets |

This is the **preserve-blanket-guard pattern** analogue for scope-pressure: the S102 lesson (`feedback_preserve_list_discipline.md`) is that named exclusions upstream do not propagate through blanket operations. m15 applies the same principle to scope-pressure — the engine's charter named which verbs are permitted, but that exclusion must be re-asserted at every pressure site. m15 IS that re-assertion mechanism: the per-site enumeration-before-execution check, applied to spec-level scope creep rather than filesystem operations.

When a pattern of notices accumulates (e.g., three `recommend_*` attempts in one session, or ≥10 same-kind events in 14 days per cluster plan Class-E escalation), Watcher MAY initiate a spec amendment interview. m15 never decides whether the amendment proceeds; it only witnesses and records.

## §9 CC-7 cross-cluster contract (E → spec interview)

m15 owns the **first step** of CC-7 (pressure-driven evolution). Full loop:

```
m15 emits JSONL → agent-cross-talk/ → Watcher tick observes → Zen audit observes
   → accumulation threshold tripped → spec amendment interview → v1.4 patch
   → G7 re-audit → merged → m1.config (cursor, scope filters) updates
   → loop continues at next session
```

All subsequent steps are human-in-loop (Watcher synthesis, Zen audit, Luke decision). The intentional asymmetry: m15 emits cheaply (one JSONL file per event) but loop closure is slow (days / weeks). This is the design — fast detection, slow consensus.

## §10 Error taxonomy (thiserror, no unwrap)

```rust
// src/m15_pressure/mod.rs (SPEC)

#[derive(Debug, thiserror::Error)]
pub enum PressureRegisterError {
    #[error("failed to write notice file: {path}: {source}")]
    NoticeWrite { path: String, source: std::io::Error },

    #[error("agent-cross-talk directory not found or not writable: {path}")]
    DirectoryUnavailable { path: String },

    #[error("event serialization failed: {0}")]
    Serialize(#[from] serde_json::Error),

    #[error("fsync on parent directory failed: {0}")]
    DirSyncFailed(std::io::Error),

    #[error("trigger excerpt exceeded 512 chars and truncation failed: {0}")]
    ExcerptOversize(usize),
}
```

All paths return `Result<PathBuf, PressureRegisterError>`. Zero `.unwrap()` / `.expect()` outside `#[cfg(test)]`. `#![forbid(unsafe_code)]`. `DirectoryUnavailable` is recoverable (tracing-only fallback path); other errors propagate.

## §11 Test budget (60 tests · G6 mutation kill ≥ 70%)

Per [TEST_DISCIPLINE](../../../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md) matrix:

| Family | Count | Notes |
|---|---:|---|
| F-Unit | 25 | per `ForbiddenCategory` arm; filename convention; atomic-write semantics; truncation |
| F-Property | 5 | filename uniqueness across 10k concurrent emits; de-dup window monotonic; one-event-per-file invariant |
| F-Fuzz | 0 | no parser surface |
| F-Integration | 15 | end-to-end emit → real `agent-cross-talk/` dir → tail-based pickup latency < 60s |
| F-Contract | 3 | JSONL schema parity with Watcher reader; serde round-trip for every category arm |
| F-Regression | 2 | reserved |
| F-Mutation | budget | ≥ **70%** kill rate (G6 standard floor — m15 is orchestration, not algorithm) |

Canonical tests (must exist by name):

- `notice_file_created_on_pressure_event`
- `notice_filename_includes_session_and_event_id_and_iso8601_date`
- `notice_filename_uses_hyphens_not_colons_in_timestamp`
- `jsonl_round_trip_preserves_all_fields`
- `recommend_verb_detected_from_spec_patch_excerpt`
- `auto_or_smart_verb_detected`
- `http_server_surface_detected_as_hard_refusal`
- `deprecated_povm_write_detected_post_pivot`
- `trigger_excerpt_truncated_at_512_chars_with_ellipsis`
- `missing_directory_returns_error_not_panic_does_not_block_caller`
- `multiple_events_produce_multiple_files`
- `dedup_window_60s_coalesces_repeats`
- `dedup_window_expiry_emits_fresh_notice`
- `tracing_warn_emitted_with_structured_fields_on_every_event`
- `atomic_write_visible_only_after_rename`
- `handshake_silence_variant_emitted_on_timeout`

## §12 Boilerplate-lift map (≈30% lifted, ~30 LOC fresh authorship)

| Source | What is lifted | Reuse % |
|---|---|---:|
| `SKILL-genesis.md` trap-classification (Cat 09) | `ForbiddenCategory` enum mirrors genesis trap-classes | 95% |
| Existing `agent-cross-talk/` exemplar drops | Filename + atomic-write protocol lifted directly | 90% |
| `logging.rs` (ME v2, Cat 06) | `tracing::warn!` structured-emission pattern; correlation ID propagation | 60% |
| `hookify.preserve-blanket-guard.local.md` | Conceptual model: per-site named-exclusion vs blanket bypass; 3-step enumeration-diff-rewrite | conceptual |
| `feedback_preserve_list_discipline.md` | Structural root cause of m15's existence | reference |

Fresh authorship (~30 LOC, NEW): `ForbiddenCategory` enum specialised for workflow-trace charter; 60s de-dup filter window; `register_pressure()` API surface; atomic-write protocol applied to `agent-cross-talk/`. m15 owns no structural-gap fresh-primitive LOC — but m15 IS the GAP-Bidi-02 closure mechanism (CC-7 closure point); that gap is a *bidi-flow* gap closed by spec discipline, not a fresh-primitive gap.

## §13 Open questions, gaps, antipatterns

**Open questions (deferred to v1.4 or post-soak)**
1. **Runtime detection machinery.** v1.3 detection is heuristic — diff scans + content scans at sweep time. Open: AST-level scan of `wf-*` binary source pre-build? Risk: would couple m15 to build pipeline; current decoupling is a feature. Probably defer until ≥10 false-negatives accumulate.
2. **`HandshakeSilence` timeout default.** Cluster plan suggests 60s peer-ack timeout. Open: per-peer override (Watcher slower than Zen)? Defer until Phase 3 integration soak shows actual ack latencies.
3. **`Other { description }` escape hatch.** Permits the operator to flag pressure that doesn't fit other variants. Risk: becomes a dumping ground that collapses category discipline. Open: cap `Other` to ≤10% of total events per 30-day window — auto-fire AP-V7-04 keyword overgrowth flag if exceeded.
4. **Atuin trajectory anchor `wt-pressure-tail`.** Per [INTEGRATION/atuin-integration.md](../../../ai_docs/optimisation-v7/INTEGRATION/atuin-integration.md) (proposed) tails the directory and writes one-line summaries to history. Hook contract not yet locked.

**Antipatterns explicitly avoided (per [ANTIPATTERNS.md](../../../ANTIPATTERNS.md) + cluster plan)**
- AP-V7-04 (keyword overgrowth) — `ForbiddenCategory` capped at the v1.3 variants; new variants require spec amendment + G7 re-audit.
- AP-Hab-04 (preserve-list discipline) — m15 IS the per-site re-assertion mechanism; blanket exclusions are explicitly flagged via dedicated trigger patterns.
- AP-V7-08 (handshake dual-silence) — `HandshakeSilence` PressureKind variant makes the silence itself observable within 60s.
- AP-V7-13 (Health-200 ≠ behaviour-verified) — m15 does not consume health endpoints; pressure detection is content-based, not status-based.
- AP30 (POVM namespace collision) — m15 writes to filesystem only; `DeprecatedPovmWrite` variant explicitly catches any module proposing POVM writes post 2026-05-17 pivot.

**Standing constraints carried into m15**

| Constraint | Enforcement point |
|---|---|
| W1 narrowed-scope (no service-wide reads) | m15 reads filesystem + in-process signals only; no HTTP, no SQLite-of-other-services |
| AP30 namespace prefix discipline | filename + JSONL content tagged under `workflow_trace_*` semantic namespace |
| Hard refusal: no HTTP server | m15 exposes no listening socket |
| Hard refusal: no POVM writes | filesystem JSONL only; `DeprecatedPovmWrite` catches violators |
| `#![forbid(unsafe_code)]` + zero `.unwrap()` outside tests | All paths return `Result<PathBuf, PressureRegisterError>` |
| 50+ tests minimum (m15 budget: 60) | Test family table §11 |
| Tracing only (no `println!`) | `tracing::warn!` per event; `tracing::error!` on `NoticeWrite` / `DirSyncFailed` |
| One-event-per-file (never append) | Property test `multiple_events_produce_multiple_files` |
| Atomic write (tmp + rename + fsync parent) | Integration test `atomic_write_visible_only_after_rename` |

---

> *m15 spec authored 2026-05-17 (S1001982) · planning-only · HOLD-v2 active · gates G1-G9 required before build · CC-7 owner · witness, not gate; emit-and-forget; silence ≠ missed.*
> *Sister anchor: [m14](m14_habitat_outcome_lift.md).*
