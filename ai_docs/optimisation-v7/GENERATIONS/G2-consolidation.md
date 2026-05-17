---
title: G2 — Consolidation Pass (Generation 2 of 7)
date: 2026-05-17 (S1001982)
kind: planning-only · sync-invariant + src-layout consolidation
purpose: close GAP-Sync-01..05 from G1; lock src/ layout; add verify-sync invariants 21-22
inputs: G1-baseline-audit.md + ULTRAMAP.md + GOD_TIER_RUST.md
output: canonical src/ layout + 22-invariant verify-sync set + Cargo workspace contract
---

# G2 — Consolidation Pass

> Back to: [[../TASK_LIST_V7_OPTIMISATION.md]] · sibling: [[G1-baseline-audit.md]] (input) · [[G3-bidi-flow.md]] (next)

---

## Gap closure (G1 → G2)

| Gap | Closure |
|---|---|
| GAP-Sync-01 (povm_calibrated cfg invariant) | ✅ added invariant #21 |
| GAP-Sync-02 (workflow-core lib contract) | ✅ defined § workflow-core contract |
| GAP-Sync-03 (per-Wave invariant subset) | ✅ verify-sync.sh `--invariants N-M` flag |
| GAP-Sync-04 (m11 dependency inversion) | ✅ m11 Day-1 = pure formula; wire Day 3-4 |
| GAP-Sync-05 (cargo doc clean invariant) | ✅ added invariant #22 |

---

## Canonical src/ layout (Cargo single-crate + features)

**Decision per GAP-Gold-01 (G4 preview):** single-crate, two `[[bin]]`, library exposed as `workflow_trace::*`. ORAC pattern. NOT a workspace.

```
workflow-trace/                                    # (G2-renamed from the-workflow-engine/)
├── Cargo.toml                                     # single crate
├── Cargo.lock
├── build.rs                                       # m8 `cargo:rustc-cfg=povm_calibrated`
├── README.md
├── CLAUDE.md
├── CLAUDE.local.md
├── plan.toml                                      # LCM-pattern declarative spec
├── ai_docs/
│   ├── GENESIS_PROMPT_V1_3.md                     # binding spec post-G7
│   ├── GENESIS_INTERVIEW_TRANSCRIPT_G5.md
│   ├── GAP_ANALYSIS_ANTHROPOCENTRIC_G6.md
│   ├── GAP_ANALYSIS_SUBSTRATE_FRAME_G6.md
│   └── optimisation-v7/                           # ← this V7 tree (planning artefacts)
├── ai_specs/                                      # post-G9 detailed module specs
├── src/
│   ├── lib.rs                                     # facade — re-exports per layer
│   ├── workflow_core/                             # L0 shared types
│   │   ├── mod.rs
│   │   ├── types.rs                               # WorkflowId, StepToken, EscapeSurfaceProfile, Confidence
│   │   ├── schemas.rs                             # serde schemas: m7 hub, m30 bank, m40 NexusEvent
│   │   ├── namespace.rs                           # AP30 prefix constants
│   │   └── errors.rs                              # workflow_core::Error (thiserror)
│   ├── m1_atuin_consumer/
│   ├── m2_stcortex_consumer/
│   ├── m3_injection_consumer/                     # L1 ↑
│   ├── m4_cascade/
│   ├── m5_battern/
│   ├── m6_cost/                                   # L2 ↑
│   ├── m7_central/
│   ├── m12_cli_reports/
│   ├── m13_stcortex_writer/                       # L3 ↑
│   ├── m8_build_prereq/                           # L4 (aspect-layer — also lives in build.rs)
│   ├── m9_namespace_guard/
│   ├── m10_ember_gate/
│   ├── m11_decay/                                 # L4 ↑
│   ├── m14_lift/
│   ├── m15_pressure/                              # L5 ↑
│   ├── m20_prefixspan/
│   ├── m21_variant_builder/
│   ├── m22_kmeans/
│   ├── m23_proposer/                              # L6 ↑ KEYSTONE
│   ├── m30_bank/
│   ├── m31_selector/
│   ├── m32_dispatcher/
│   ├── m33_verifier/                              # L7 ↑
│   ├── m40_synthex_emit/
│   ├── m41_lcm_router/
│   ├── m42_povm_dual/                             # L8 ↑
│   └── bin/
│       ├── wf_crystallise.rs                      # binary 1 — owns m1-m23 + m40-m42
│       └── wf_dispatch.rs                         # binary 2 — owns m30-m33
├── tests/
│   ├── integration/                               # cross-module integration
│   │   ├── cc1_cascade_cost_coupling.rs
│   │   ├── cc2_trust_aspect_routing.rs
│   │   ├── cc3_evidence_driven_iteration.rs
│   │   ├── cc4_proposal_bank_dispatch.rs
│   │   ├── cc5_substrate_learning_loop.rs
│   │   ├── cc6_verification_gated_dispatch.rs
│   │   └── cc7_pressure_driven_evolution.rs
│   ├── contract/                                  # bridge-contract parity
│   │   ├── m40_synthex_emit_schema.rs
│   │   ├── m41_lcm_rpc_frame.rs
│   │   └── m42_povm_dual_payload.rs
│   ├── regression/                                # per-bug
│   │   └── (one file per fixed bug)
│   ├── fixtures/                                  # captured real payloads
│   └── ember_held_approvals.tsv                   # m10 allowlist (Watcher+Zen signed)
├── benches/                                       # criterion
│   ├── m20_prefixspan_bench.rs
│   └── m11_decay_bench.rs
├── fuzz/                                          # cargo-fuzz nightly
│   └── fuzz_targets/
│       ├── m4_cascade_id.rs
│       ├── m7_jsonb_blob.rs
│       ├── m13_stcortex_schema.rs
│       ├── m20_prefixspan.rs
│       ├── m40_outbox_jsonl.rs
│       └── m41_jsonrpc_frame.rs
├── migrations/                                    # sqlite schema versioning (m13 stcortex writer)
│   ├── 001_initial_schema.sql
│   └── 002_central_hub_fitness_dimension.sql
├── scripts/
│   ├── gate.sh                                    # 4-stage QG (per GOD_TIER_RUST.md)
│   ├── verify-sync.sh                             # all 22 invariants
│   └── wave-merge.sh                              # per-Wave-end merge protocol
├── .bacon-locations                               # bacon-ls export (per-worktree)
├── bacon.toml                                     # bacon job config
├── deny.toml                                      # cargo-deny config
├── clippy.toml                                    # workspace-level pedantic config
└── .claude/
    ├── settings.json                              # workspace-scoped Claude settings (V7 T5.5)
    └── skills/
        └── (post-G9 — workflow-trace-specific skills)
```

---

## Cargo.toml contract (planning-spec only — not executed)

```toml
[package]
name = "workflow-trace"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"  # match ULTRAPLATE habitat baseline
license = "<TBD-Luke>"
authors = ["workflow-trace contributors"]
publish = false

[[bin]]
name = "wf-crystallise"
path = "src/bin/wf_crystallise.rs"
required-features = ["crystallise"]

[[bin]]
name = "wf-dispatch"
path = "src/bin/wf_dispatch.rs"
required-features = ["dispatch"]

[lib]
name = "workflow_trace"
path = "src/lib.rs"

[features]
default = ["crystallise", "dispatch", "all-substrates"]
crystallise = []
dispatch = []
all-substrates = ["synthex-emit", "lcm-router", "povm-dual"]
synthex-emit = []
lcm-router = []
povm-dual = []
ralph-integration = []  # M2+ optional per GAP-Gold-03

[dependencies]
tokio = { version = "1.40", features = ["full"] }
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio-rustls"] }
fnv = "1.0"
# (No chrono — see GOD_TIER_RUST.md)
time = { version = "0.3", features = ["serde"] }

[dev-dependencies]
proptest = "1.5"
insta = { version = "1.40", features = ["yaml", "redactions"] }
mockito = "1.5"  # for true-externals only; not internal modules
testcontainers = "0.21"
criterion = { version = "0.5", features = ["html_reports"] }

[build-dependencies]
# m8 build-prereq: cargo:rustc-cfg=povm_calibrated detection

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = "symbols"
debug = "line-tables-only"  # symbolicated flamegraph support

[profile.bench]
inherits = "release"
debug = true  # full symbols for flamegraph
```

**Note:** This is a planning-spec — NOT to be executed pre-G9. Lives in this G2 doc and in the future `Cargo.toml` after G9 `cargo init`.

---

## workflow_core library contract (closes GAP-Sync-02)

```rust
// src/workflow_core/mod.rs — facade
pub mod types;
pub mod schemas;
pub mod namespace;
pub mod errors;

pub use types::{WorkflowId, StepToken, EscapeSurfaceProfile, Confidence};
pub use errors::Error;
```

**types.rs:**
- `WorkflowId(String)` newtype — namespaced workflow identifier
- `StepToken { kind, content_hash, timestamp }` — step-grain unit
- `EscapeSurfaceProfile { ReadOnly, HostWrite, Network, SandboxEscape, Destructive }` — ordinal enum, derives `Ord`
- `Confidence { wilson_low: f64, mean: f64, wilson_high: f64, n: usize }` — Wilson CI quad

**schemas.rs:**
- m7 hub row: `WorkflowRunRow { id, started_at, ended_at, fitness_dimension, outcome, consumer_inputs }`
- m30 bank entry: `BankEntry { workflow_id, definition_hash, escape_surface, sunset_at, last_dispatched_at }`
- m40 NexusEvent: `WorkflowEvent::Run { id, outcome, fitness_delta, fields }`

**namespace.rs:**
- `pub const WORKFLOW_TRACE_PREFIX: &str = "workflow_trace_";`  // AP30
- `pub fn validate_namespace(s: &str) -> Result<&str, Error>` — m9 guard helper

**errors.rs:**
- `Error` enum via `thiserror::Error`: `BridgeError`, `NamespaceViolation`, `SampleSizeBelowF2`, `ConductorDispatchDisabled`, `EmptyBank`, `EmberHeldRefused`, `PovmNotCalibrated`

**Doc discipline:** every public item has rustdoc + `# Errors` section if returning `Result`.

---

## Verify-sync invariant set (updated to 22)

Per GOD_TIER_RUST.md §"Verify-sync invariants" — adding 21 and 22:

| # | Invariant | Check command | Owning Wave |
|---|---|---|---|
| 1-20 | (per GOD_TIER_RUST.md) | (see source) | per Wave |
| **21** | **No `povm_calibrated` Cargo feature** — `cfg(povm_calibrated)` only honored via env at build.rs | `grep -E 'povm_calibrated' Cargo.toml` returns 0 hits | Wave 1 (m8 build-prereq) |
| **22** | **`cargo doc --no-deps --workspace` returns 0 warnings** | `cargo doc --no-deps --workspace 2>&1 \| grep -c warning` returns 0 | Wave 1 + per Wave |

### Per-Wave invariant subset

```bash
# scripts/verify-sync.sh --invariants 1-7   # Wave 1
# scripts/verify-sync.sh --invariants 1-13  # Wave 2 (cumulative)
# scripts/verify-sync.sh --invariants 1-22  # Wave 3 (full set)
```

Wave 1 invariants: 1-7 + 21-22 (L0/L1/L4 only — measure-only modules don't yet exist)
Wave 2 invariants: 1-13 + 21-22 (adds L2/L3/L5 measure-only)
Wave 3 invariants: 1-22 (full set — active-verb modules now exist)

### `verify-sync.sh` skeleton

```bash
#!/usr/bin/env bash
# scripts/verify-sync.sh — workflow-trace V2 verify-sync gate
# Usage: ./scripts/verify-sync.sh [--invariants N-M]
set -uo pipefail   # NOT -e — fleet discipline per CLAUDE.md

INVARIANTS_RANGE="${1:-1-22}"
FAIL=0

inv() {
  local n="$1"; local desc="$2"; local cmd="$3"
  # invariant-N check
  if eval "$cmd"; then
    echo "✅ invariant-$n: $desc"
  else
    echo "❌ invariant-$n: $desc"
    FAIL=$((FAIL+1))
  fi
}

# invariant 1: every L1-L8 module has src/ entry
[[ "$INVARIANTS_RANGE" =~ ^(1|.*1.*)$ ]] && inv 1 \
  "every active module has src/ entry" \
  'for n in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 20 21 22 23 30 31 32 33 40 41 42; do test -d "src/m${n}_"* || exit 1; done'

# ... (full 22 invariants follow same pattern)

exit "$FAIL"
```

---

## Phase 1 runbook amendment (closes GAP-Sync-04)

**Day 1 — Cluster D FIRST (revised):**
- m8 build-prereq build.rs scaffold (NO live POVM probe — placeholder + env hook)
- m9 namespace guard PURE (no I/O — pure validator fn)
- m10 Ember CI scaffold (allowlist file empty; rubric path constant)
- **m11 PURE formula** — `compute_decay_factor(base: f64, f: f64, g: f64, r: f64) -> f64` — NO I/O dependencies; testable in isolation; F-Property tests assert invariants

**Day 2 — Cluster A:**
- m1 + m2 + m3 substrate ingest (consumer fn signatures; cursor types; pagination)

**Day 3-4 — Cluster C wires m11 to m7:**
- m7 schema includes `last_run_at REAL NOT NULL`
- m11 wire — read m7.last_run_at + m14.frequency + stcortex pathway.weight → call `compute_decay_factor`

**Rationale:** m11's pure-formula function ships Day 1 (cheap to test); wiring happens Day 3-4 when m7 schema concrete. Dependency inversion resolved.

---

## Executable-referent enforcement (substrate-frame G1 mitigation)

Every G2-G7 deliverable must contain:
- ≥1 executable bash command (verify-sync, gate, atuin probe, etc.), OR
- ≥1 src-tree path that will be created post-G9, OR
- ≥1 test name (BDD-flavoured) per behaviour described

**G2 self-check:** ✅ bash commands in `verify-sync.sh` skeleton + `Cargo.toml` planning-spec (concrete file path) + invariant table commands. No pure narrative.

---

## G2 Watcher pre-positioning

**Class D activated.** Four-surface drift is the prime risk of consolidation passes: src/ vs cluster-N.md vs ULTRAMAP vs verify-sync. Closed by:
- Per-Wave-end obsidian-vault-librarian sweep
- verify-sync invariants 1-3 explicitly cross-check
- G7 cross-doc verification matrix

---

## G2 substrate-frame pass

**Second-frame question:** what does this consolidation produce for the substrate?

The 22 invariants are themselves a substrate signal — they get re-exercised every Wave-end, leaving atuin trajectory entries. These constitute a Hebbian-grain reinforcement pulse: invariants that consistently green strengthen "this is the correct structure" pathway weights; invariants that flip back-and-forth weaken structural certainty.

**Substrate-frame mitigation:** never silently downgrade an invariant. If invariant N fails on a legitimate refactor, EXPLICITLY remove invariant N from the set with rationale in the drift register — never let invariants rot into perpetually-failing-but-ignored state.

---

## G2 close

✅ G2 PASS. Closed 5 GAP-Sync entries. Added invariants 21-22. Defined workflow_core lib contract + canonical src/ layout + Cargo planning-spec. Phase 1 runbook m11 dependency inversion resolved. Substrate-frame rule: invariants must succeed or be explicitly retired (never silently fail).

**Output for G3:** clear src/ paths + workflow_core contract. G3 reads ULTRAMAP V2 + this G2 + cluster-X.md specs (when written) and produces full bidi-flow contract per module.

---

*G2 authored 2026-05-17 by Command. src/ canonical. 22 invariants. Substrate-frame: invariants never silently rot.*
