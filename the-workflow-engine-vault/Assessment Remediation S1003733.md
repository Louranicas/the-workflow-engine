---
title: Assessment Remediation — S1003733
date: 2026-05-22
session: S1003733
kind: remediation-record
status: COMPLETE — 21 findings closed · C22 binaries wired · Wave G mutation closeout · 1967 tests · clippy+pedantic clean
authority: Luke @ node 0.A
---

# Assessment Remediation — S1003733

> Back to: [[HOME]] · [[MASTER_INDEX]] · [[Hardening Fleet 2026-05-21]] · [[workflow-engine-code-base]] · [[CLAUDE.md]] · [[CLAUDE.local.md]]
> Canonical codebase map: `/tmp/wfe-assessment-S1003733/CODEBASE_MAP.md` (989-line authoritative module map)
> Repo siblings: [`../CLAUDE.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/CLAUDE.md) · [`../CLAUDE.local.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/CLAUDE.local.md) · [`../GATE_STATE.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/GATE_STATE.md) · [`../ai_docs/HARDENING_FLEET_2026-05-21.md`](file:///home/louranicas/claude-code-workspace/the-workflow-engine/ai_docs/HARDENING_FLEET_2026-05-21.md)

This note records the **assessment-driven remediation** of `workflow-trace` carried out under
session **S1003733** (2026-05-22). It is the current-reality companion to [[Hardening Fleet 2026-05-21]]:
the Hardening Fleet took the codebase to 1903 tests; this remediation took it to **1967**,
wired both binaries, and closed every assessment finding.

---

## 1. The 7-facet assessment — 80/100

A god-tier 7-facet code-quality assessment scored the implemented 26-module `workflow-trace`
codebase **80/100**. The assessment produced a 989-line authoritative codebase map
(`/tmp/wfe-assessment-S1003733/CODEBASE_MAP.md`) and **22 findings**, of which **21 were
remediated** in five gated waves; the 22nd (C22 — wiring the two binary stubs) was flagged
as a product-feature build requiring explicit node-0.A authorisation, then executed
separately once authorised.

The seven facets assessed: **correctness · security · type-design · documentation integrity ·
synergy wiring · structure / module hygiene · test rigour (mutation kill-rate)**.

---

## 2. The 5 remediation waves (commit `0cc7be3`)

All 21 contained findings were closed in five gated waves. Each wave passed the full quality
gate (`cargo check` + `clippy -D warnings` + `clippy -D clippy::pedantic` +
`cargo test --all-targets --all-features --release`). Tests 1903 → 1921.

| Wave | Theme | What changed |
|------|-------|--------------|
| **A** | Documentation integrity | Corrected the W4 mutation over-claim in `HARDENING_FLEET_2026-05-21.md` ("412 mutants / 80.6%" did not reconcile with any committed artifact — replaced with artifact-backed numbers); fixed "W4 in progress / 1835 tests" doc-drift in `CLAUDE.md` + `CLAUDE.local.md`; removed a stale m11 `error.rs` comment ("m7/m14/m42 don't exist yet" — they do). |
| **B** | Contained code fixes | m20 `max_length==0` → typed `MinerError::MaxLengthZero` (was silent coercion); m20 `MaxGap` encapsulated (private field + `new`/`get`, consistent with `MinSupport`); m32 `DispatcherError::WireFormat` detail now surfaced in the structured `DispatchOutcome`; m22 dedicated `KMeansError::NonFiniteCoordinate` variant (removed the `usize::MAX` sentinel); `unwrap_or(0)` audit (m15 `unix_ms_now` made explicitly saturating + observable; m31/m6 documented); m2 SEC3 — bare `git` `$PATH` fallback removed (PATH-hijack vector eliminated); m13 `StcortexWriter::new` → `new_unchecked` (loud SEC1-opt-out at every call site); Cargo.toml reserved feature flags documented. |
| **C** | Core-type encapsulation (W3 #5–#10 portfolio) | **6 representable-illegal-state holes closed** — `Pattern` (KEYSTONE; `canonical_hash` can no longer desync from `steps`), `WorkflowProposal` (F2-gated fallible constructor), `AcceptedWorkflow` (validated weight + controlled mutators), `NexusEvent.kind` (`String` → `NexusEventKind` enum), `BatternId`/`CascadeClusterId`/`ChainId` (private inner field + `new`/accessor), `WorkflowRunRow` (coupled `ended_at`/`outcome` optionals → `RunState{Open,Closed}` enum; outcome now typed `Outcome`). |
| **D** | Synergy wiring + security | **CC-4** — `compose_proposals` now threads the m22 diversity cluster (was hardcoded `None`); **CC-5** — canonical `workflow_pathway_id()` namespaced pathway id (not a bare decimal); **m32 security MEDIUM** — `EscapeSurfaceProfile` acknowledgement gate made **MONOTONE** (`FileWrite`/`NetworkEgress`/`ProcessMutate`/`SandboxEscape` no longer dispatch unacknowledged; `HumanAcceptanceSignature` carries an `acknowledged_ceiling`); **F2 m8 POVM-gate decision** resolved → **KEEP-DORMANT**; m21 `build_variants` iteration cap (the ~20 timeout mutants are now bounded/catchable). |
| **E** | Structure | m32/m22 oversized modules — test blocks extracted to `tests.rs` siblings (consistent with m10/m11); m13 test-helper deduplication; `GOLD_STANDARDS.md` — newtype accessor naming convention + fixed stale "no cargo build until G9" rule. |

---

## 3. C22 — binaries wired (commit `ae7d460`)

The 22nd assessment finding: the two binaries (`wf-crystallise`, `wf-dispatch`) were one-line
`println!` stubs — the last open finding, and the reason the lib↔binary seam was entirely
unexercised. C22 turned them into **real CLI programs**.

- **New library module** `workflow_core::orchestration` (`src/orchestration/{mod,crystallise,dispatch}.rs`,
  ~60K of source). Pipeline logic lives in the **library**, not the binary source files —
  so the lib↔binary seam is integration-testable (`run()` can be exercised against
  temp-file fixtures with `--offline` / `--dry-run` without launching a subprocess).
- **Binaries are thin `main()` wrappers** — parse `std::env::args()` → `run()` → print
  `Report` → process exit code (`0` success, `1` pipeline fault, `2` CLI arg error).
- **`wf-crystallise`** — `--atuin-db --injection-db --runs-db --proposals-out --min-support
  --max-gap --offline --format text|json`. Drives m1 atuin + m3 injection.db ingest →
  m4 cascade correlate → m7 run record → m14 lift → m20 mine → m21/m23 compose →
  `WorkflowProposal` JSONL out → m12 report. Live-service stages (m2/m13/m40/m41/m42)
  attempted only without `--offline` and **degrade gracefully** — a down service is logged
  via `tracing` and the stage is skipped, never a crash.
- **`wf-dispatch`** — `--proposals-in --top-k --conductor-url --dry-run --execute
  --ack-ceiling`. Reads the proposals JSONL → m30 `bank.accept` → m31 select → m33 verifier
  gate → m32 `ConductorDispatcher` (only under `--execute`; **`--dry-run` is the default-safe
  mode**). Ships a minimal `reqwest`-blocking `HttpConductorClient` impl of m32's
  `ConductorClient` trait.
- **JSONL bridge** — `WorkflowProposal` (serde) is the crystallise→dispatch handoff,
  closing the bank-persistence gap the assessment flagged (the `CuratedBank` was previously
  in-memory with no path between the two binaries).

Tests 1924 → 1967 (+43): orchestration unit tests + `tests/wf_{crystallise,dispatch}_integration.rs`
exercising `run()` against temp-SQLite fixtures in offline/dry-run mode. clippy + pedantic clean.

---

## 4. Wave G — mutation closeout (commit `c0ec95c`)

Post-remediation mutation verification (`cargo-mutants`, frozen tree) surfaced 15 surviving
mutants. Wave G resolved all 15 — **honestly, not by inflation**:

- **6 KILLED** by 3 new discriminating-input tests:
  - m11 `consolidation.rs:335` (`< → <=`) — exact-prune-threshold-boundary test.
  - m22 `kmeans_plus_plus_seed:238` (`delete -`) — unary-minus seed-selection freeze test.
  - m22 `kmeans_plus_plus_seed:303` (`> → >=` / `> → ==` / `> → <`, ×3) — FNV-tiebreak
    resolution test sweeping both tiebreak branches.
- **9 PROVEN EQUIVALENT** — all m21 `build_variants` loop-condition mutants. The Wave-D+
  iteration-cap defense-in-depth (independent `MAX_LOOP_ITERATIONS` break + redundant
  per-push `out.len()` guard) renders these mutations **output-equivalent**. Each line
  carries a `// mutant-equivalent:` proof comment in source. Not fake-killed — proven.

The verified post-remediation mutation run (S1003733, frozen tree `@0cc7be3`): **324 mutants
— 254 caught / 15 missed / 0 timeout / 55 unviable → 94.4 % kill rate** (254 of 269 viable).
The S1003733 iteration-cap fix eliminated all 20 prior m21 `build_variants` *timeout* mutants
(now scored; 0 timeout). After Wave G the 15 survivors are resolved: 6 killed + 9 proven-equivalent.

---

## 5. What changed per facet

| Facet | Assessment state | After remediation |
|-------|------------------|-------------------|
| **Correctness** | m20 silent `max_length==0` coercion; m22 `usize::MAX` sentinel | Typed `MinerError::MaxLengthZero`; dedicated `KMeansError::NonFiniteCoordinate`. |
| **Security** | m2 bare `git` `$PATH` fallback (PATH-hijack); m32 non-monotone escape-surface ack; m13 quiet SEC1 opt-out | PATH fallback removed; `EscapeSurfaceProfile` ack gate **monotone** with `acknowledged_ceiling`; `StcortexWriter::new_unchecked` makes the SEC1 opt-out loud at every call site. |
| **Type-design** | 6 representable-illegal-state holes (Pattern hash desync, coupled `WorkflowRunRow` optionals, stringly `NexusEvent.kind`, un-newtyped IDs, ungated `WorkflowProposal`/`AcceptedWorkflow` constructors) | All 6 closed (Wave C) — fallible/gated constructors, `RunState` enum, `NexusEventKind` enum, private-inner ID newtypes. |
| **Documentation integrity** | W4 mutation over-claim ("412 mutants / 80.6%") not artifact-backed; "W4 in progress / 1835 tests" doc-drift; stale m11 comment | Corrected to artifact-backed numbers; charter + both `CLAUDE.local.md` reconciled; stale comment removed. |
| **Synergy wiring** | CC-4 diversity cluster hardcoded `None`; CC-5 pathway id was a bare decimal | CC-4 threads the real m22 cluster index through `compose_proposals`; CC-5 uses canonical namespaced `workflow_pathway_id()`. |
| **Structure / hygiene** | m32/m22 oversized modules; m13 duplicated test helpers; stale `GOLD_STANDARDS.md` rule | Test blocks extracted to `tests.rs` siblings; helpers deduplicated; `GOLD_STANDARDS.md` newtype convention added + stale "no cargo build" rule fixed. |
| **Test rigour** | mutation kill-rate unverified post-remediation; binary `main()` seam unexercised | 94.4 % verified kill-rate; Wave G resolved all 15 survivors; C22 added 43 tests exercising the lib↔binary seam — **1967 total**. |

---

## 6. Bugs & Known Issues — post-remediation

All 22 assessment findings are **CLOSED** (21 remediated + C22 wired). The honest residuals
below are scope notes, not regressions — disclosed verbatim in the C22 commit body.

### Resolved (assessment findings — S1003733)

| ID | Severity | Finding | Resolution |
|----|----------|---------|------------|
| F-A1 | LOW | W4 mutation over-claim not artifact-backed | Wave A — replaced with artifact-backed numbers |
| F-A2 | LOW | "W4 in progress / 1835 tests" doc-drift | Wave A — charter + both `CLAUDE.local.md` reconciled |
| F-A3 | LOW | Stale m11 `error.rs` comment | Wave A — removed |
| F-B1 | MED | m20 silent `max_length==0` coercion | Wave B — typed `MinerError::MaxLengthZero` |
| F-B2 | LOW | m20 `MaxGap` unencapsulated | Wave B — private field + `new`/`get` |
| F-B3 | MED | m32 `WireFormat` detail not surfaced | Wave B — surfaced in structured `DispatchOutcome` |
| F-B4 | MED | m22 `usize::MAX` sentinel | Wave B — `KMeansError::NonFiniteCoordinate` variant |
| F-B5 | LOW | `unwrap_or(0)` ambiguity (m15/m31/m6) | Wave B — m15 explicitly saturating+observable; m31/m6 documented |
| F-B6 | HIGH | m2 bare `git` `$PATH` fallback (PATH-hijack) | Wave B — fallback removed |
| F-B7 | MED | m13 quiet SEC1 opt-out | Wave B — `new` → `new_unchecked` |
| F-C1..C6 | MED | 6 representable-illegal-state type holes | Wave C — all 6 closed |
| F-D1 | MED | CC-4 diversity cluster hardcoded `None` | Wave D — threads real m22 cluster |
| F-D2 | LOW | CC-5 pathway id a bare decimal | Wave D — canonical `workflow_pathway_id()` |
| F-D3 | MED | m32 non-monotone escape-surface ack | Wave D — monotone gate + `acknowledged_ceiling` |
| F-D4 | — | m8 POVM-gate architecture undecided | Wave D — RESOLVED: KEEP-DORMANT |
| F-D5 | MED | m21 `build_variants` ~20 timeout mutants | Wave D — iteration cap; bounded/catchable |
| F-E1..E3 | LOW | m32/m22 oversized modules; m13 helper dup; stale gold-standard rule | Wave E — extracted/deduplicated/fixed |
| C22 | — | Both binaries are stubs; lib↔binary seam unexercised | Wired — `workflow_core::orchestration` module + JSONL bridge |
| Wave G | — | 15 surviving mutants post-remediation | 6 killed + 9 proven-equivalent |

### Open residuals (honest scope notes — not regressions)

| ID | Severity | Residual | Notes |
|----|----------|----------|-------|
| R1 | MED | **m33 verifiers are conservative-default (`Approve`) placeholders.** Real per-kind policy logic (Security/Consistency/Cost/Ember) needs inputs the binary does not yet receive. | The m33 `aggregate` gate is wired and exercised; the four `Verifier` impls return `Approve` by default. A workflow is never *auto-rejected*, but never *adversarially screened* on the CLI path either. Real policy logic is a follow-up product build. |
| R2 | LOW | **m22 K-means diversity not assembled on the CLI batch paths.** `wf-crystallise` passes an honest `\|_\| None` diversity closure into `compose_proposals` rather than a faked signal. | The CC-4 *wiring* exists (the closure is threaded); the CLI does not yet *compute* feature vectors to feed m22. `WorkflowProposal.diversity_cluster` is `None` on CLI-generated proposals. Honest `None`, not a fabricated cluster. |
| R3 | INFO | **9 m21 `build_variants` mutants proven output-equivalent.** | Not killable — the defense-in-depth iteration cap makes the loop-condition mutation inert. Each carries a `// mutant-equivalent:` proof comment. Recorded as closed-as-equivalent, not as a test gap. |
| R4 | INFO | **m8 POVM trust gate is a dormant build.rs tripwire (KEEP-DORMANT).** | Architecture decision, not a defect. m8 fires `compile_error!` only on a future POVM-read code site; workflow-trace is stcortex-routed (m42 ADR) so no live POVM-read site exists. |

---

## 7. Diagnostics — workflow-trace

> Probes re-verified live 2026-05-22 (S1003733) against the post-`ae7d460` tree.

### Build + test

```bash
cd /home/louranicas/claude-code-workspace/the-workflow-engine
CARGO_TARGET_DIR=./target cargo check --all-targets --all-features 2>&1 | tail -5
cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tail -5
cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic 2>&1 | tail -5
CARGO_TARGET_DIR=./target cargo test --all-targets --all-features --release 2>&1 | tail -10
# Expected: 1967 tests passing, 0 failed; clippy + pedantic clean.
```

### Binary smoke tests (offline / dry-run — no live services)

```bash
# wf-crystallise — offline pipeline against temp SQLite fixtures
cargo run --bin wf-crystallise -- --help
cargo run --bin wf-crystallise -- --offline --atuin-db /tmp/atuin.db \
  --injection-db /tmp/injection.db --runs-db /tmp/runs.db \
  --proposals-out /tmp/proposals.jsonl --format text

# wf-dispatch — dry-run is default-safe; --execute required for real Conductor dispatch
cargo run --bin wf-dispatch -- --help
cargo run --bin wf-dispatch -- --proposals-in /tmp/proposals.jsonl --top-k 5 --dry-run
```

Exit codes: `0` pipeline ran to completion (or `--help`/`--version`); `1` pipeline fault
(missing DB, unwritable output); `2` CLI argument error.

### Live-service degradation contract

Every stage touching a live habitat service degrades gracefully — an unreachable service is
logged via `tracing` and recorded as a skipped/degraded `Report` entry; the pipeline always
runs end-to-end. A down service never panics or aborts the run.

| Service | Port / URL | Used by | Behaviour if down |
|---------|-----------|---------|-------------------|
| stcortex (SpacetimeDB) | `ws://127.0.0.1:3000` | m2 consumer, m13 writer | stage skipped, logged, degraded `Report` entry |
| ORAC sidecar | `http://127.0.0.1:8133/blackboard/substrate_LTP_density` | m13 LTP probe | m13 defers to JSONL outbox (3-band gate) |
| synthex-v2 | `http://127.0.0.1:8092/v3/nexus/push` | m40 nexus emit | stage skipped, logged |
| LCM | `http://127.0.0.1:8082/rpc` | m41 loop-create | stage skipped, logged |
| HABITAT-CONDUCTOR | `:8141` (`--conductor-url`) | m32 dispatch | refused with `RefusalReason::ConductorUnreachable`; only attempted under `--execute` |

### mutation testing

```bash
cd /home/louranicas/claude-code-workspace/the-workflow-engine
cargo mutants --in-place 2>&1 | tail -20   # scoped to m10/m11/m21/m22 per W4 config
# Verified S1003733: 324 mutants — 254 caught / 15 missed / 0 timeout / 55 unviable
# (94.4% kill rate). After Wave G: 6 of the 15 killed, 9 proven-equivalent.
```

### Git anchor

```bash
git log --oneline dc25335..ae7d460   # Hardening Fleet W1-W5 + S1003733 remediation + C22 + Wave G
# main @ ae7d460 — pushed origin (GitHub) + gitlab
```

---

## 8. Open follow-ups (for node 0.A)

1. **R1 — m33 verifier policy logic.** The four `Verifier` impls are conservative-default
   `Approve` placeholders. Implementing real per-kind Security/Consistency/Cost/Ember policy
   is a product-feature build needing inputs the CLI does not yet plumb through.
2. **R2 — m22 diversity on CLI paths.** Wire `wf-crystallise` to compute feature vectors and
   feed m22 k-means so `WorkflowProposal.diversity_cluster` is populated on CLI-generated
   proposals (the CC-4 closure is already threaded; only the feature-vector assembly is missing).
3. **Conductor live-dispatch soak.** `wf-dispatch --execute` against a live HABITAT-CONDUCTOR
   (`:8141`) needs Conductor Waves 1B/1C/2/3 brought up (`auto_start=false`) — Luke @ terminal.
   Keep real dispatch human-ratified (`HumanAcceptanceSignature`).
4. **Directory rename** `the-workflow-engine/` → `workflow-trace/` — deferred post-M0, cosmetic.

---

## 9. Provenance

- **Assessment + remediation:** Command, session S1003733, 2026-05-22.
- **Commits:** `0cc7be3` (5-wave remediation), `046e955` (W4 doc fold-in), `c0ec95c` (Wave G),
  `ae7d460` (C22 binary wiring) — all on `main`, range `dc25335..ae7d460`, pushed origin + gitlab.
- **Canonical codebase map:** `/tmp/wfe-assessment-S1003733/CODEBASE_MAP.md` (989 lines).
- **Companion record:** [[Hardening Fleet 2026-05-21]] (W0–W5; the predecessor hardening pass).

---

*Note authored 2026-05-22 (S1003733) by the Obsidian Vault Librarian — reflecting the
post-`ae7d460` reality. Edit only if the codebase state changes.*
