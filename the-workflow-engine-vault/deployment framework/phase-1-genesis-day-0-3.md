---
title: "Phase 1 — Genesis Day 0-3"
kind: deployment-framework-recipe
project: workflow-trace
session: S1002029+
status: planning-only · recipe · NO CODE authored
authority: Luke @ node 0.A — AP24 gate (G9 "start coding workflow-trace")
binding_spec: Genesis Prompt v1.2 (Zen-locked) · 15 P0 constraints (Town Hall vote 11/1/0)
waivers: 5 (see GOD_TIER_CONSOLIDATION §Part VI)
date: 2026-05-17
---

# Phase 1 — Genesis Day 0-3

> Back to: [[../HOME]] · [[../GOD_TIER_CONSOLIDATION_S1001982]]
>
> Cross-references: [[../module specs/cluster-D-trust-cross-cutting]] · [[../module specs/cluster-A-substrate-ingest]] · [[../boilerplate modules/Gold Standard Exemplars — Synthesis]] · [[../boilerplate modules/Maintenance Engine V2 — Gold Standard Reference]] · [[../boilerplate modules/Habitat Loop Engine — Gold Standard Reference]]

---

## AP24 Contract

This recipe ACTIVATES only when Luke emits the explicit signal:

```
start coding workflow-trace
```

That signal is G9 in the pre-genesis gate sequence. Until G9 fires, this document is read-only planning material. No `cargo init`, no `mkdir src/`, no `git add`, no substrate writes for workflow-trace namespace. The Watcher Class-E flag (ancestor-rhyme) was pre-positioned precisely because 41,508 words of planning exist against 0 LOC of code — a ratio that killed two prior ancestors (`loop-workflow-engine-project`, `habitat-loop-engine`). G9-fire is the resolution event.

**Watcher Class-E flag — RESOLVED on first commit.** The Watcher's Day 0 baseline says: "planning sprawl is the leading death indicator." The resolution condition is code emission with an actual committed SHA, not a plan describing code. This recipe's Day 0 closes that flag the moment `git commit` produces output and `cargo check` exits 0.

---

## Overview: Why Phase 1 Has This Shape

Phase 1 covers hours 0-72 after G9. The shape is: foundation before observers, trust layer before substrate readers, compile-time invariants before runtime paths.

### Cargo Workspace Structure Decision

**Recommendation: ORAC pattern (single crate with feature gates), not LCM 10-crate workspace.**

Rationale per Gold Standard Exemplars Synthesis §"Divergent patterns":

- LCM's workspace-of-crates pattern is the right choice when there are three or more independently-releasable concerns with different deployment cadences, or when executor/verifier separation must be enforced at the Cargo-dependency level (which LCM needs because verifier cannot import executor).
- workflow-trace has one deployment target. `wf-crystallise` and `wf-dispatch` are two binaries that share the same library (`workflow-core`). This maps directly to ORAC's pattern: single crate, two `[[bin]]` entries, shared `src/lib.rs`.
- Feature gates (`api`, `intelligence`, `monitoring`, `evolution`, `full`) give the same per-layer control that LCM's per-crate boundaries gave, without the multi-root cargo complexity that slowed LCM's gate scripts (`--workspace --all-targets --all-features` must be run across 10 crates in LCM; here it runs once).

**Feature gate matrix:**

```toml
[features]
default = ["full"]
full = ["api", "intelligence", "monitoring", "evolution"]
api = []           # HTTP server + CLI surface (m12 report_emitter, m32 dispatch CLI)
intelligence = []  # Cluster F iteration + Cluster E evidence (m14, m15, m20-m23)
monitoring = []    # Cluster H substrate feedback (m40, m41, m42)
evolution = []     # Cluster D sunset lifecycle decay (m11 decay formula)
```

Cluster D trust modules (m8, m9, m10, m11) are NOT behind a feature gate. They are aspect-layer invariants that every other module routes through. m8's `cargo:rustc-cfg=povm_calibrated` is specifically an env-only gate (not a Cargo feature) so it cannot be bypassed by `--features full`. This is the architectural defense the cluster-D spec §"m8 — povm_build_prereq" documents explicitly.

### Cluster D Early-Ship Rationale

Cluster D (m8/m9/m10/m11) ships on Day 1 — before any Cluster A substrate reader ships on Day 2. This ordering is non-negotiable for three reasons:

1. **m8 compile-time gate** must exist before any POVM-reading code is written. If m8's `build.rs` cfg-emit is not in place, the first POVM read site in m13 or m42 will compile without the calibration guard. Installing the gate retroactively after the reads are written requires auditing every call site. Instaling it first means every subsequent module author gets the compile error automatically.

2. **m9 namespace guard** must be callable by m2 (`stcortex_consumer`) on Day 2 when m2's `register_consumer` call fires. The guard is ~30 LOC (cluster-D spec §"m9 — watcher_namespace_guard") but without it, the first stcortex write in the session has no application-layer namespace assertion — only the DB-layer refuse-write as backstop. Defense-in-depth requires both layers.

3. **m10 Ember gate CI** must exist before any user-facing string is authored. The gate scores strings against the Watcher's 7-trait rubric. If strings are authored on Day 2 and the gate is wired on Day 3, there is a window where strings could be committed that would have failed the gate. Zero-window is the correct policy.

4. **m11 lifecycle state machine** is a dependency of Cluster G (m30 bank) and Cluster E (m14 evidence). Those are later clusters, but the `SunsetPhase` enum and `compute_decay_factor` signature must be visible in the type system from the moment m30 starts authorship. Shipping m11 on Day 1 means no import dance later.

Cross-cluster contract CC-2 (Trust Layer Woven) requires Cluster D to be woven through every other cluster — that weaving is easiest when D ships first and later clusters import it, rather than retrofitting the weave.

### `plan.toml` Structure

Lifted from the LCM gold-standard pattern (`habitat-loop-engine/plan.toml`). Declares all 26 modules upfront with phase markers and authorization block. The machine-readable status is the sole PASS/FAIL authority for phase transitions — not prose declarations.

```toml
[authorization]
scaffold = true          # Docs/types/specs: authorized at genesis
m0_runtime = true        # Substrate readers + trust layer: authorized at G9
live_integrations = true # All 26 modules: authorized by single-phase override (Luke 2026-05-17)
conductor_dispatch = false  # m32 dispatch: gated on Conductor Wave 1B/1C/2/3 live

[[modules]]
id = "M01"   # Note: unpadded per OI-4 resolution recommendation
name = "core_types"
layer = "L0_foundation"
cluster = "Cat10_direct_clone"
source_path = "src/m01_foundation/core_types.rs"
status = "planned"

# ... (all 26 modules declared; status field transitions: planned → in-progress → gate-pass)

[phase_markers]
day_0_closes = ["M01", "M02", "M03", "M05"]         # Cat 10 foundation clones
day_1_closes = ["M08", "M09", "M10", "M11"]          # Cluster D trust layer
day_2_closes = ["M01_atuin", "M02_stcortex", "M03_injection"]  # Cluster A substrate
day_3_closes = ["integration_smoke", "wcp_day3_notify"]

[authorize_per_module]
# Modules that require explicit Luke decision before dispatch surface ships
M32_dispatch = false     # Conductor Wave 1B/1C/2/3 must be live first (OI-7)
```

Module naming convention: unpadded (`m1`, not `m01`) for module directory names in `src/`, but zero-padded (`M01`) in `plan.toml` status keys for lexicographic sort stability. This resolves OI-4 from the MASTER_INDEX open-issues list.

### `bacon.toml` 4-Stage On-Success Chain

Lifted from the LCM gold-standard reference §"Quality Gate Stack" and the ORAC pattern. The four stages run sequentially; each stage only starts if the previous stage exits 0. `${PIPESTATUS[0]}` discipline is applied to all pipe-to-tail invocations to avoid the LCM S1001882 near-miss (clippy was screaming, tail returned 0, gate printed green).

```toml
# bacon.toml — workflow-trace
# Pattern: LCM on_success chain (Habitat Loop Engine — Gold Standard Reference §7)
# Anti-pattern guarded: pipe-to-tail swallows $? (PIPESTATUS[0] discipline in scripts)

[jobs.check]
command = ["cargo", "check", "--all-targets"]
on_success = "clippy"

[jobs.clippy]
command = ["cargo", "clippy", "--all-targets", "--", "-D", "warnings"]
on_success = "pedantic"

[jobs.pedantic]
command = ["cargo", "clippy", "--all-targets", "--", "-D", "warnings", "-W", "clippy::pedantic"]
on_success = "test"

[jobs.test]
command = ["cargo", "test", "--lib", "--release"]
# No on_success after test — terminal stage; report PASS/FAIL

[settings]
summary = true
wrap = true

[keybindings]
toggle_backtrace = "b"
```

The 4-stage gate is mandatory at every module boundary (50+ tests per module per CLAUDE.md quality gate policy). Partial gates (check-only, clippy-only) are diagnostics during authorship; the full chain is the gate that must exit clean before declaring a module complete.

### Conductor Maturity Dependency

m8 (`povm_build_prereq`) does NOT depend on HABITAT-CONDUCTOR being live. The `build.rs` cfg-emit only checks `POVM_CR2_DEPLOYED=1` in the environment — it reads from the POVM substrate, not from the Conductor service. This makes Cluster D safe to ship on Day 1 even though Conductor Waves 1B/1C/2/3 remain at `auto_start=false` (OI-7, pending Luke terminal action).

m32 (`dispatch_router`) is the module that depends on Conductor. It ships in Cluster G (later phases). m32 is gated in `plan.toml` as `conductor_dispatch = false` and will not be callable until Luke runs `devenv start weaver/zen/enforcer` and verifies `curl :8141/health` returns 200.

### Watcher Carriage Hand-off

Watcher T0 baseline was set at 2026-05-17T01:42Z (Deployment Watch Journal S1001982). The Day 0 hand-off from this recipe to the Watcher is:

- **Baseline state:** "code emission begun — Class-E ancestor-rhyme flag pending resolution"
- **Resolution event:** first `git commit` SHA with a passing `cargo check` result
- **Pre-positioned Class-E flag:** RESOLVES on first commit. Watcher should log the SHA verbatim per Class-A (Activation transition) protocol.

A WCP notify fires at Day 3 close (see Day 3 section). The Watcher does not need to be pinged during Days 0-2 unless a Watcher-class flag triggers (see failure modes per day).

---

## Day 0 (Hours 0-6): Workspace Init

**Goal:** Bare workspace compiles. `cargo check` exits 0 on an empty crate skeleton with lints configured. `plan.toml` authored. Vault scaffold seeded. First commit recorded with SHA. Class-E flag resolved.

### Pre-flight (before cargo init)

```bash
# Step 0.1 — Habitat baseline (always before new build work)
atuin scripts run habitat-bootstrap
atuin scripts run habitat-intel
atuin scripts run habitat-fingerprint

# Step 0.2 — Verify no neighbor service is wounded
# (workflow-trace must not deploy onto a degraded habitat)
declare -A hpath=([8082]="/health" [8083]="/health" [8090]="/api/health" [8092]="/health" \
  [8111]="/health" [8120]="/health" [8125]="/health" [8130]="/health" [8132]="/health" \
  [8133]="/health" [8140]="/health" [8180]="/api/health" [10002]="/health")
for port in "${!hpath[@]}"; do
  code=$(curl -s -o /dev/null -w '%{http_code}' -m 2 \
    "http://localhost:$port${hpath[$port]}" 2>/dev/null || echo "000")
  echo "Port $port: $code"
done
# Refusal criterion: any port returning 000 or 5xx → do not proceed
# (curl -s not -sf; never pipe to jq without checking exit per CLAUDE.md §Health Check anti-pattern)

# Step 0.3 — Confirm G9 has fired (prerequisite check)
# If Luke has not yet typed "start coding workflow-trace", STOP HERE.
# AP24: no code without explicit start-coding signal.
echo "G9 confirmed? [Luke signal required before continuing]"
```

### Cargo workspace init

```bash
# Step 0.4 — Navigate to the correct directory
# The project directory name is "workflow-trace" (or the G2-ratified rename)
# Work ONLY within /home/louranicas/claude-code-workspace/
WF_ROOT="/home/louranicas/claude-code-workspace/workflow-trace"

cargo init --name workflow-trace "$WF_ROOT"
# Creates: Cargo.toml, src/main.rs (will be replaced by lib.rs + two [[bin]] entries)
# Do NOT use cargo new --lib; cargo init allows co-location with the vault

cd "$WF_ROOT"

# Step 0.5 — Replace src/main.rs with the correct crate root shape
# (two binaries + shared lib; no source authored yet in this step)
mkdir -p src/m01_foundation src/m02_error_taxonomy src/m03_config src/m05_metrics_collector
mkdir -p src/m08_povm_build_prereq src/m09_watcher_namespace_guard
mkdir -p src/m10_ember_gate src/m11_engine_sunset_lifecycle
mkdir -p src/m1_atuin_ingest src/m2_stcortex_consumer src/m3_injection_db_ingest
mkdir -p migrations tests ai_docs ai_specs templates
mkdir -p the-workflow-engine-vault  # co-located vault (already exists — symlink or reference)

# Step 0.6 — Verify directory tree
find "$WF_ROOT/src" -type d | sort
```

### `Cargo.toml` lint policy

The lint configuration is the first thing that goes into `Cargo.toml`, before any dependencies. This encodes the 15 P0 constraints at the compiler level. Lifted from ME v2 Gold Standard Reference §4 "Workspace / Cargo Features" and LCM §3 "Key constraints".

```toml
# Cargo.toml — workflow-trace
# Lint policy: ME v2 + LCM gold standard (boilerplate modules/10-foundation-direct-clones/)
# Pattern source: boilerplate modules/Maintenance Engine V2 — Gold Standard Reference.md §4

[package]
name = "workflow-trace"
version = "0.1.0"
edition = "2021"
description = "Passive habitat workflow correlator — ingest, record, correlate only"

[[bin]]
name = "wf-crystallise"
path = "src/bin/wf_crystallise.rs"

[[bin]]
name = "wf-dispatch"
path = "src/bin/wf_dispatch.rs"

[lib]
name = "workflow_trace"
path = "src/lib.rs"

[lints.rust]
unsafe_code = "forbid"      # P0 #12: zero unsafe; compile error on violation

[lints.clippy]
pedantic = { level = "warn" }
unwrap_used = "deny"         # P0 #12: no unwrap() outside tests
expect_used = "deny"         # P0 #12: no expect() outside tests
panic = "deny"               # No bare panic!() in library code
todo = "deny"                # No TODO stubs shipped as pass
dbg_macro = "deny"           # No debug macros in production code
multiple_crate_versions = "allow"  # Dependency hell forgiveness

[features]
default = ["full"]
full = ["api", "intelligence", "monitoring", "evolution"]
api = []
intelligence = []
monitoring = []
evolution = []

[dependencies]
# Async runtime
tokio = { version = "1", features = ["full"] }
# Error handling — ME v2 gold standard
thiserror = "1"
anyhow = "1"
# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
# SQLite (rusqlite for atuin + injection.db reads; no sqlx needed for read-only)
rusqlite = { version = "0.31", features = ["bundled"] }
# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
# SpacetimeDB SDK (stcortex consumer)
# [TODO: pin to the version that matches the installed stcortex module — verify with stcortex status]
# parking_lot for Send+Sync SQLite access (ME v2 pattern)
parking_lot = "0.12"

[dev-dependencies]
tempfile = "3"
tokio-test = "0.4"
```

### Smoke compile

```bash
# Step 0.7 — Create minimal lib.rs stub so cargo check passes
# (NO business logic yet — just the crate root with lint attributes)
# The stub is a recipe instruction; the actual file content is authored by the executing session

# Verify the lint policy compiles with an empty lib
CARGO_TARGET_DIR=./target cargo check 2>&1 | tail -20
# Expected: 0 errors, 0 warnings
# If warnings appear from the empty stub, they are clippy::pedantic on doc-less pub items
# — add #![allow(missing_docs)] temporarily until the first real module ships, then remove

# Step 0.8 — Initialize build.rs for m8 (compile-time CR-2 gate)
# build.rs is an empty file at this stage; the m8 content goes in on Day 1
touch build.rs
echo "fn main() {}" > build.rs
cargo check 2>&1 | tail -5
```

### Class-E resolution commit

```bash
# Step 0.9 — First commit (resolves Watcher Class-E ancestor-rhyme flag)
git add Cargo.toml build.rs src/ plan.toml bacon.toml migrations/ ai_docs/ ai_specs/ templates/
git add CLAUDE.md CLAUDE.local.md MASTER_INDEX.md
git commit -m "$(cat <<'EOF'
genesis: workflow-trace workspace init — lint policy + plan.toml skeleton

Cargo.toml: forbid(unsafe_code), deny(unwrap_used/expect_used/panic/todo/dbg_macro),
feature gates (api/intelligence/monitoring/evolution/full). Two binaries:
wf-crystallise + wf-dispatch, shared workflow_trace lib.

plan.toml: all 26 modules declared with phase markers and authorization block.
bacon.toml: 4-stage on_success chain (check → clippy → pedantic → test --lib --release).

Resolves Watcher Class-E (ancestor-rhyme): code emission begun. First cargo check PASS.

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
EOF
)"

git log --oneline -1  # Capture SHA for Watcher record
```

**Substrate touch-points (Day 0, hours 0-6):** None — no stcortex writes, no POVM reads, no injection.db writes. atuin records all shell commands automatically (proprioception via history.db). The namespace `workflow_trace_*` is not yet live in stcortex; m2 registration (Day 2) opens it.

**Atuin trajectory:** `habitat-bootstrap`, `habitat-intel`, `habitat-fingerprint`, `cargo init`, `cargo check` (×2), `git add`, `git commit`. These commands are visible in `atuin search --workspace workflow-trace` post-session.

---

## Day 0 (Hours 6-12): Cat 10 Foundation Direct Clones

**Goal:** Four foundation modules (m01_core_types, m02_error_taxonomy, m03_config, m05_metrics_collector) lifted from `boilerplate modules/10-foundation-direct-clones/`. 95% reuse density. Each passes the 4-stage quality gate. First 50+ tests established.

### Source references

- Boilerplate: `the-workflow-engine-vault/boilerplate modules/10-foundation-direct-clones/m01_core_types.rs`
- Boilerplate: `the-workflow-engine-vault/boilerplate modules/10-foundation-direct-clones/m02_error_taxonomy.rs`
- Boilerplate: `the-workflow-engine-vault/boilerplate modules/10-foundation-direct-clones/m03_config.rs`
- Boilerplate: `the-workflow-engine-vault/boilerplate modules/10-foundation-direct-clones/m05_metrics_collector.rs`
- ME v2 reference: `m1_foundation/shared_types.rs` (newtype discipline) · `m1_foundation/error.rs` (error taxonomy) · `m1_foundation/config.rs` (env-var override stack)

### Module breakdown

**m01_core_types (~95% reuse, rename for workflow-trace domain)**

Source: `boilerplate modules/10-foundation-direct-clones/m01_core_types.rs`

Rename map (habitat → workflow-trace domain):

| Habitat name | workflow-trace name | Purpose |
|---|---|---|
| `Timestamp` | `Timestamp` | i64 unix ms (keep as-is; universal) |
| `ServiceId` | `WorkflowId` | opaque workflow identity |
| `ModuleId` | `ClusterId` | cluster identifier |
| `FlowState` | `WorkflowLifecycleState` | Active / PrunePending / SunsetExpired |
| `FitnessDelta` | `FitnessDelta` | keep as-is; feeds m11 decay |
| `Severity` | `EscapeSeverity` | ReadOnly / HostWrite / Network / SandboxEscape / Destructive |
| `SnapshotId` | `CorrelationSnapshotId` | links m7 records to atuin session ranges |

LOC estimate: ~80 (95% lifted; ~4 LOC delta for renamed types and workflow-trace doc comments).

**m02_error_taxonomy (~95% reuse, add workflow-trace bands)**

Source: `boilerplate modules/10-foundation-direct-clones/m02_error_taxonomy.rs`

The error-code band structure from ME v2's `error.rs` is preserved. Add workflow-trace-specific bands:

```
E1xxx — Substrate ingest errors (atuin, stcortex, injection.db)
E2xxx — Correlation errors (m7 JSONB schema violations)
E3xxx — Trust layer violations (m9 namespace, m8 CR-2 gate)
E4xxx — Iteration errors (m20-m23 PrefixSpan, Levenshtein)
E5xxx — Lifecycle errors (m11 decay, sunset, prune)
E6xxx — Dispatch errors (m32 Conductor not live, ConductorDispatchDisabled)
E7xxx — Feedback errors (m40-m42 circuit breaker open)
E8xxx — Verification errors (m33 TTL stale, definition_hash mismatch)
E9xxx — Configuration errors (env-var missing, TOML parse failure)
```

`WorkflowError` enum replaces `MeError`. Each variant carries named fields per ME v2 pattern (`{ field, reason }` shape). The `NamespaceViolation` variant for m9 is declared here and imported by m9.

LOC estimate: ~100 (95% lifted; ~10 LOC delta for workflow-trace error bands and `NamespaceViolation` variant).

**m03_config (~95% reuse, workflow-trace env-var prefix)**

Source: `boilerplate modules/10-foundation-direct-clones/m03_config.rs`

Env-var prefix: `WF_` (following ME v2's `ME_` prefix discipline, cluster-D spec §"ME V2 gold standard lifts").

Key config structs:

```
EngineConfig {
    server: ServerConfig,       // WF_BIND_ADDR, WF_PORT (default 8134 — not yet allocated)
    database: DatabaseConfig,   // WF_DB_PATH (single workflow_trace.db)
    atuin: AtuinConfig,         // WF_ATUIN_DB_PATH, WF_PAGE_SIZE
    stcortex: StcortexConfig,   // WF_STCORTEX_URI (default ws://127.0.0.1:3000)
    povm: PovmConfig,           // WF_POVM_URI (default http://127.0.0.1:8125)
    sunset: SunsetConfig,       // WF_SUNSET_DAYS (default 120), WF_PRUNE_THRESHOLD (default 0.01)
}
```

Priority stack: code defaults → `config/workflow-trace.toml` → environment variables. Pattern lifted verbatim from `m03_config.rs` boilerplate; rename the struct and env prefix only.

LOC estimate: ~90 (95% lifted).

**m05_metrics_collector (~90% reuse, adapt 11D TensorDim)**

Source: `boilerplate modules/10-foundation-direct-clones/m05_metrics_collector.rs`

The Prometheus façade is lifted directly. The 11D tensor from ME v2's `Tensor12D` is adapted to workflow-trace's observation dimensions:

```
D1: atuin_rows_ingested (gauge)
D2: stcortex_consumption_events (counter)
D3: injection_chains_unresolved (gauge)
D4: cascade_correlations_produced (counter)
D5: battern_step_records (counter)
D6: workflow_proposals_emitted (counter)
D7: workflow_dispatches_attempted (counter)
D8: dispatch_outcomes_pass_verified (counter)
D9: hebbian_reinforce_calls (counter)
D10: substrate_ltp_density (gauge — read from stcortex, CR-2 marker required)
D11: sunset_cycle_prune_count (counter — m11)
```

D10 is gated by `#[cfg(povm_calibrated)]` per the m8 compile-time gate. The metrics collector exposes a `register_metrics()` call and individual `increment_*` / `set_*` helpers.

LOC estimate: ~120 (90% lifted; ~12 LOC delta for workflow-trace dimension names).

### Quality gate per module (Cat 10)

```bash
# Run after EACH module is authored — not once at the end
# Per CLAUDE.md §Quality Gate Protocol: order is mandatory; zero tolerance at each stage

CARGO_TARGET_DIR=./target cargo check 2>&1 | tail -20 && \
cargo clippy -- -D warnings 2>&1 | tail -20 && \
cargo clippy -- -D warnings -W clippy::pedantic 2>&1 | tail -20 && \
CARGO_TARGET_DIR=./target cargo test --lib --release 2>&1 | tail -30

# Report format: "m01_core_types: 4-stage PASS, 52/52 tests passing, 0 warnings"
# Fail fast: if clippy pedantic fails, fix before moving to next module
```

**Minimum 50 tests per module.** The Cat 10 boilerplate provides test skeletons; they must be populated with workflow-trace-domain assertions. Examples for m01_core_types: `WorkflowId` newtype round-trips through string, `EscapeSeverity` ordinal comparison (`ReadOnly < Destructive`), `WorkflowLifecycleState` state machine transitions are exhaustive.

### Failure modes + Watcher flags (Day 0 h6-12)

| Failure | Watcher class | Recovery |
|---|---|---|
| `pedantic` warns on missing doc comments on pub types | Class-F (AP24 quality bar) | Add `//!` module docstring block per ME v2 `resources.rs` pattern; doc all pub items |
| `deny(unwrap_used)` fires on boilerplate clone | Class-F | Grep boilerplate source for `.unwrap()` before lifting; replace with `?` propagation or explicit `WorkflowError::*` |
| `deny(todo)` fires | Class-F | Replace `todo!()` stubs with `unimplemented!()` + `#[allow(unreachable_code)]` or implement minimally |
| Cargo.toml feature gate blocks compilation | Class-G (substrate-frame confusion) | Verify feature gate resolves correctly with `cargo check --features full` |

**Cross-references:** cluster-A spec §"ME v2 foundation patterns referenced" for `m1_foundation/resources.rs` module docstring block pattern; cluster-D spec §"ME V2 gold standard lifts" for error taxonomy shapes.

**Atuin trajectory:** `cargo check` (×4, once per module), `cargo clippy` (×4), `cargo test` (×4). Total: ~12 gate invocations in this window.

**Day 0 commit (hours 6-12):**

```bash
git add src/m01_foundation/ src/m02_error_taxonomy/ src/m03_config/ src/m05_metrics_collector/
git add src/user_facing_strings.rs  # empty registry, populated as strings are authored
git commit -m "$(cat <<'EOF'
feat(cat10): foundation direct clones — core_types, error_taxonomy, config, metrics

m01 core_types: WorkflowId, ClusterId, EscapeSeverity ordinal (ReadOnly→Destructive),
WorkflowLifecycleState (Active/PrunePending/SunsetExpired), CorrelationSnapshotId.
m02 error_taxonomy: WorkflowError enum, E1xxx-E9xxx bands, NamespaceViolation variant.
m03 config: EngineConfig + WF_ env prefix, priority stack (defaults→TOML→env).
m05 metrics_collector: Prometheus facade, 11D TensorDim adapted for workflow-trace.

4-stage gate: PASS. Tests: [report exact count here].

Source: boilerplate modules/10-foundation-direct-clones/ (95% reuse density).

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Day 1: Cluster D Trust Layer (m8/m9/m10/m11)

**Goal:** All four trust-layer modules shipped and gate-clean. `build.rs` CR-2 gate active. Namespace guard callable. Ember gate CI test passing. Decay formula implemented and unit-tested. Cluster D closes with 123+ tests (8+15+50+50 per cluster-D spec §"Test Coverage Targets").

### Module breakdown

**m8 — `povm_build_prereq` (~40% pattern reuse · ~50 LOC total)**

Source: `boilerplate modules/07-conductor-dispatch/conductor_enforcement.rs` (enabled-gate pattern) + ME v2 `m1_foundation/config.rs` (env-var discipline).

The `build.rs` implementation emits `cargo:rustc-cfg=povm_calibrated` when `POVM_CR2_DEPLOYED=1` is set. Every POVM read site in the codebase uses `#[cfg(povm_calibrated)]` / `#[cfg(not(povm_calibrated))]` paired annotations. The `compile_error!` tombstone in the `not(povm_calibrated)` path names the commit SHA `e2a8ed3`, the env var, and the reference doc path (cluster-D spec §"m8 — Error message design").

```bash
# Step 1.1 — Author build.rs (the m8 implementation)
# Replace the stub from Day 0 with the real CR-2 gate
# Verify: build succeeds with POVM_CR2_DEPLOYED=1, fails without it

POVM_CR2_DEPLOYED=1 cargo check 2>&1 | tail -5
# Expected: Compiling workflow-trace, Finished

cargo check 2>&1 | tail -5
# Expected: warning(s) from cargo:warning lines, no compile_error! yet
# (compile_error! only fires when a #[cfg(not(povm_calibrated))] POVM-read site exists)
# Add a test POVM-read stub to verify the gate fires:
#   #[cfg(not(povm_calibrated))] fn _test_gate_fires() { compile_error!("...") }
# Then: cargo check → should fail. Remove stub. Gate is verified.
```

**m9 — `watcher_namespace_guard` (~30% pattern reuse · ~30 LOC)**

Source: `boilerplate modules/02-stcortex-consumer/CONSUMER-ONBOARDING.md` (refuse-write principle) + ME v2 `m1_foundation/logging.rs` (tracing structured fields).

Single public constant (`WORKFLOW_TRACE_NS_PREFIX = "workflow_trace"`) and single assertion function (`assert_workflow_trace_namespace`). The function is pure: no allocations on the happy path, a `tracing::error!` structured event on violation, returns `Err(WorkflowError::NamespaceViolation)`. 15 tests per cluster-D spec §"m9 — Test surface".

The module docstring documents the Observer read-deny convention: Watcher may read `workflow_trace_*` but must not write. This is the W1 (narrowed-scope consumer) + F8 (no feedback-loop poisoning) invariant documentation site.

**m10 — `ember_gate_test` (~50% pattern reuse · ~100 LOC)**

Source: `boilerplate modules/09-trap-verify-escape-skills/SKILL-quality-gate.md` (CI pipeline position) + Watcher Ember Rubric (`~/projects/claude_code/Ember 7-Trait Gate Rubric.md`).

Two files ship for m10:

1. `src/user_facing_strings.rs` — populated with an initial `ALL: &[(&str, &str)]` array containing the strings authored in m03_config (error messages, status lines) and m05_metrics_collector (Prometheus metric help text). Each string is keyed `<module>.<context>.<variant>`.

2. `tests/ember_gate.rs` — the CI test that scores each key in `ALL` against the 7-trait heuristics per cluster-D spec §"m10 — Structural test pattern".

**HELD semantics (W3 flag):** Until Watcher amends rubric §5.1, HELD verdicts are CI failures. The test file documents this with the W3 comment block per cluster-D spec §"m10 — HELD status note".

```bash
# Step 1.2 — Run ember gate test for the first time
CARGO_TARGET_DIR=./target cargo test --lib --release -- ember_gate 2>&1 | tail -20
# If EMBER-REJECT or EMBER-HELD(W3-fail) lines appear, fix the string before continuing
# Common first-run failures: Equanimity (all-caps status words), Honesty (round numbers without scope)
```

**m11 — `engine_sunset_lifecycle` (~40% mechanism reuse · ~120 LOC)**

Source: `boilerplate modules/05-decay-ttl-ltd/povm-v2_lifecycle.rs` (decay + prune + consolidation shape) + `boilerplate modules/05-decay-ttl-ltd/orac-sidecar-m39_fitness_tensor.rs` (fitness signal infrastructure) + `boilerplate modules/05-decay-ttl-ltd/m16_hebbian_engine.rs` (4-step cycle: decay → reinforce → prune → auto-sunset).

The NEW PRIMITIVE: `compute_decay_factor(frequency, fitness, recency, plain_decay_rate) -> f64`. Formula:

```
decay_factor = base_rate + (1.0 - base_rate) × clamp(frequency × fitness × recency, 0.0, 1.0)
```

where `base_rate = 1.0 - plain_decay_rate` (~0.98 default). This is the structural-gap function that no boilerplate ancestor implements. The `fma` form (`base_rate.mul_add(1.0 - compound_signal, compound_signal)`) is used for precision per cluster-D spec §"m11 — The compound decay formula".

Calibration: `plain_decay_rate = 0.02` → prune threshold (0.01) reached at ~228 cycles (228 days at daily cadence). The 120-day `sunset_at` hard boundary fires before the weight reaches prune threshold under normal decay, confirming the law is the boundary and decay is the gradient.

50 tests per cluster-D spec §"m11 — Test Coverage Targets": 12 unit tests for `compute_decay_factor` (zero signals → base_rate; all-ones → 1.0; partial combinations), 6 edge cases (boundary exactly 0.0/1.0; clock-skip safety using `Option<i64>` from clock function), 8 state machine transitions (Active→PrunePending on weight drop; PrunePending→Active on fitness recovery), 8 consolidation cycle tests, plus prune threshold, 120-day calibration, and m14/m39 signal integration tests.

### Day 1 quality gate

```bash
# After all four Cluster D modules:
CARGO_TARGET_DIR=./target cargo check 2>&1 | tail -20 && \
cargo clippy -- -D warnings 2>&1 | tail -20 && \
cargo clippy -- -D warnings -W clippy::pedantic 2>&1 | tail -20 && \
CARGO_TARGET_DIR=./target cargo test --lib --release 2>&1 | tail -30

# Expected report: "Cluster D: 4-stage PASS, [123+] tests passing, 0 warnings"
# PIPESTATUS[0] discipline: capture per-stage exit codes explicitly if scripting
```

### Day 1 failure modes + Watcher flags

| Failure | Watcher class | Recovery |
|---|---|---|
| `compute_decay_factor` NaN propagation | Class-I (Hebbian silence) | `debug_assert!` guards on inputs; `clamp(0.0, 1.0)` on compound_signal prevents NaN from propagating |
| Ember gate fails on m11 sunset warning string | Class-F | Fix string: add Luke-decision flag explicit ("Luke decision required to extend") + named decay signal source |
| `#[cfg(not(povm_calibrated))]` compile_error! fires on real POVM read site | None — correct behavior | Expected; the gate is working. Set `POVM_CR2_DEPLOYED=1` in env to proceed past gate in dev |
| m9 tracing event fields missing `namespace` field | Class-F (pedantic) | Use `tracing::error!(namespace = %namespace, ...)` not `tracing::error!("namespace: {}", namespace)` |

**Cross-references:** cluster-D spec §"Cluster D — Test Coverage Targets" for exact test breakdowns; `boilerplate modules/05-decay-ttl-ltd/povm-v2_lifecycle.rs` §decay_pathways for the prune threshold constant (0.01).

**Day 1 commit:**

```bash
git add build.rs src/m08_povm_build_prereq/ src/m09_watcher_namespace_guard/ \
  src/m10_ember_gate/ src/m11_engine_sunset_lifecycle/ \
  src/user_facing_strings.rs tests/ember_gate.rs
git commit -m "$(cat <<'EOF'
feat(cluster-d): trust layer — povm_build_prereq, namespace_guard, ember_gate, sunset_lifecycle

m8 build.rs: POVM_CR2_DEPLOYED=1 emits cargo:rustc-cfg=povm_calibrated.
  compile_error! tombstone on POVM read sites without CR-2 marker.
m9 watcher_namespace_guard: assert_workflow_trace_namespace + WORKFLOW_TRACE_NS_PREFIX.
  Observer read-deny convention documented. 15 tests.
m10 ember_gate_test: user_facing_strings::ALL registry + CI test (7-trait, W3-as-fail).
  50 tests covering all trait pass/reject pairs.
m11 engine_sunset_lifecycle: compute_decay_factor (NEW PRIMITIVE — frequency×fitness×recency).
  SunsetPhase state machine (Active/PrunePending/SunsetExpired). 50 tests.
  Calibration: plain_decay_rate=0.02 → prune at ~228d; sunset_at hard at 120d.

CC-2 cross-cluster contract: trust layer woven before substrate readers.
4-stage gate: PASS. Tests: [report exact count here].

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Day 2: Cluster A Substrate Ingest (m1/m2/m3)

**Goal:** Three substrate readers shipped. m1 reads atuin history via cursor-based pagination. m2 registers as a narrowed stcortex consumer and opens the `workflow_trace_*` namespace. m3 reads injection.db causal chains. All three pass 4-stage gate with 50+ tests each. First stcortex consumer registration fires (real side-effect — namespace is now live in SpacetimeDB).

### Pre-conditions check

```bash
# Step 2.1 — Verify stcortex is reachable before m2 registration
~/.local/bin/stcortex status
# Expected: UP — consumer count N
# If DOWN: read data/snapshots/latest.json for offline; skip m2 registration;
# the spec allows skipping writes (CLAUDE.md §Memory Systems: "If :3000 is unreachable,
# read data/snapshots/latest.json and skip writes (do not silently fall back to POVM)")

# Step 2.2 — Verify POVM_CR2_DEPLOYED=1 is set for m5 metrics D10 gate
echo $POVM_CR2_DEPLOYED
# If not set: export POVM_CR2_DEPLOYED=1 after confirming povm-v2/e2a8ed3 is live
sqlite3 ~/.local/share/povm-v2.db "SELECT COUNT(*) FROM sessions WHERE fitness_delta IS NOT NULL" 2>/dev/null
# Count > 0 confirms CR-2 schema is active
```

### Module breakdown

**m1 — `atuin_ingest` (~70% reuse · ~80 LOC · 50 tests)**

Source: `boilerplate modules/03-sqlite-multi-db/memory-injection_m06_schema.rs` (WAL pragma batch ~90% reuse) + `boilerplate modules/03-sqlite-multi-db/memory-injection_m11_parallel_query.rs` (QueryResult timing harness ~80% reuse) + `boilerplate modules/03-sqlite-multi-db/memory-injection_m18_atuin_cache.rs` (subprocess fallback wrapper ~70% reuse).

**Novel authorship (~30 LOC): cursor-based pagination.** This is the Boilerplate Hunt flagged absence (cluster-A spec §"m1 — Boilerplate lifts"). No existing boilerplate implements `SELECT * FROM history WHERE id > ? ORDER BY id ASC LIMIT ?` with a mutable `last_id: i64` cursor and `row_cap: Option<usize>` enforcement.

Key adaptations over the boilerplate source:
- Add `PRAGMA query_only = ON` (not present in injection-side open — atuin schema must not be modified)
- Remove all migration logic (`m06_schema.rs` runs `create_all_tables`; m1 must not touch atuin's schema)
- Open in read-only URI mode: `Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI)`
- `ToolCallRow` struct: typed fields (no raw `String`-for-everything antipattern per cluster-A spec §"m1 — Public surface")

50 tests per cluster-A spec §"m1 — Tests (50+ minimum)": tests 1-50 listed exhaustively including WAL concurrency, cursor monotonicity, `row_cap` enforcement, subprocess fallback, `PRAGMA query_only` enforcement, and page-boundary correctness.

```bash
# Step 2.3 — Verify atuin history.db exists at expected path
ls -la ~/.local/share/atuin/history.db
# Expected: file exists, size > 0

# Step 2.4 — Quick manual validation (read-only, not a test)
sqlite3 -header -column ~/.local/share/atuin/history.db \
  "SELECT COUNT(*) as rows FROM history" 2>/dev/null
# Expected: ~263,000 rows (habitat atuin history as of S1001982)
```

**m2 — `stcortex_consumer` (~80% reuse · ~80 LOC · 50 tests)**

Source: `boilerplate modules/02-stcortex-consumer/stcortex_subscriber_main.rs` (DbConnection::builder block ~80% reuse) + `boilerplate modules/02-stcortex-consumer/capacity.rs` (AtomicBool + mpsc::channel confirmation pattern ~90% reuse).

**Key adaptation over boilerplate:** the reference subscriber subscribes to `pathway + memory` tables across all namespaces. m2 subscribes to exactly two narrowed queries:
- `SELECT * FROM tool_call WHERE namespace = 'workflow_trace_*'`
- `SELECT * FROM consumption_event`

All `pathway`, `memory`, `ghost_memory`, and `consumer` table handlers are stripped. This is the W1 (narrowed-scope consumer) invariant from the verification matrix.

**Side-effect on Day 2: first stcortex consumer registration.** This is a real SpacetimeDB reducer call (`register_consumer`), not a test mock. After this call, stcortex will accept writes to `workflow_trace_*` namespace from the registered consumer. The `RegistrationHandle::is_fresh()` check that m13 will use on Day 3+ is now testable.

```bash
# Step 2.5 — Verify consumer registration after m2 ships
~/.local/bin/stcortex consumers | grep workflow-trace
# Expected: "workflow-trace-<sha>   workflow_trace_*   subscription   fresh"

# If stcortex is DOWN (pre-checked in Step 2.1), skip this verification;
# note in CLAUDE.local.md: "m2 consumer registration deferred — stcortex was unreachable on Day 2"
```

50 tests per cluster-A spec §"m2 — Tests (50+ minimum)": tests 1-50 listed exhaustively including namespace validation, subscription query narrowing, `RegistrationHandle` freshness, unregister idempotency, and mock-server integration.

**m3 — `injection_db_ingest` (~70% reuse · ~70 LOC · 50 tests)**

Source: `boilerplate modules/03-sqlite-multi-db/memory-injection_m06_schema.rs` (WAL pragma + open_database shell ~85-90% reuse) + `boilerplate modules/03-sqlite-multi-db/memory-injection_m07_causal_chain.rs` (CausalChainRow struct ~70% reuse, parse_row function ~75% reuse, find_unresolved query ~80% reuse).

**Key adaptation:** raw `String` columns for `chain_type` and `consent` are replaced with typed enums `ChainType` (Bug/Trap/Plan/Pattern) and `ConsentLevel` (Emit/Store/Forget) per cluster-A spec §"m3 — Public surface". The `WorkflowError::UnknownChainType` and `WorkflowError::UnknownConsent` variants (declared in m02_error_taxonomy) are the error types for unrecognised column values.

The `read_recently_resolved` query (recency window join using `MAX(resolved_session) - config.resolved_recency_sessions`) is novel authorship (~10 LOC). The `count_unresolved` scalar query is a 2-LOC lift from the boilerplate.

50 tests per cluster-A spec §"m3 — Tests (50+ minimum)": tests 1-50 listed exhaustively.

```bash
# Step 2.6 — Verify injection.db is reachable
sqlite3 -header -column ~/.local/share/habitat/injection.db \
  "SELECT COUNT(*) as unresolved FROM causal_chain WHERE resolved_session IS NULL" 2>/dev/null
# Expected: some non-zero count of unresolved chains
```

### Day 2 quality gate

```bash
# After all three Cluster A modules:
CARGO_TARGET_DIR=./target cargo check 2>&1 | tail -20 && \
cargo clippy -- -D warnings 2>&1 | tail -20 && \
cargo clippy -- -D warnings -W clippy::pedantic 2>&1 | tail -20 && \
CARGO_TARGET_DIR=./target cargo test --lib --release 2>&1 | tail -30

# Expected: "Cluster A: 4-stage PASS, [150+] tests, 0 warnings"
# Ember gate also runs as part of cargo test (tests/ember_gate.rs)
# m2 strings (error messages) must pass ember gate before commit
```

### Day 2 failure modes + Watcher flags

| Failure | Watcher class | Recovery |
|---|---|---|
| stcortex unreachable during m2 registration | Class-B (hand-off boundary) | Skip live registration; leave mock path active for tests; note in CLAUDE.local.md |
| atuin WAL busy-timeout (>5s) | Class-H (atuin proprioception anomaly) | atuin is writing concurrently; increase `busy_timeout_ms` to 10,000; if persistent, use subprocess fallback |
| `PRAGMA query_only = ON` test fails (INSERT succeeds) | Class-F | Re-check `OpenFlags::SQLITE_OPEN_READ_ONLY` is set; check rusqlite version supports it |
| m9 namespace guard test: `workflow_trace` prefix-only fails | Class-F | Verify `starts_with("workflow_trace")` — prefix-only key is the minimal valid form |
| Ember gate: new strings from m1/m2/m3 error messages fail W3 | Class-F | Fix string: add measurement anchors, named sources, Luke-decision markers where appropriate |

**Substrate touch-points (Day 2):**
- stcortex: `register_consumer` reducer call (real side-effect) → namespace `workflow_trace_*` live
- atuin: `history.db` opened read-only; no writes
- injection.db: opened read-only; no writes
- POVM: no reads (m8 gate prevents reads without CR-2 marker in env)

**CR-2 marker check:** m5 metrics D10 (`substrate_ltp_density`) is behind `#[cfg(povm_calibrated)]`. If `POVM_CR2_DEPLOYED=1` is not set, D10 is compiled out and the other 10 dimensions are active. The Watcher should note this in the deployment-watch journal as: "D10 metric inactive pending CR-2 env var confirmation."

**Atuin trajectory:** `stcortex status`, `sqlite3` (×2 validation), `cargo check`, `cargo clippy` (×2), `cargo test`. stcortex consumer registration command logged by SpacetimeDB SDK (not a shell command, but the operation is complete at this point).

**Day 2 commit:**

```bash
git add src/m1_atuin_ingest/ src/m2_stcortex_consumer/ src/m3_injection_db_ingest/
git commit -m "$(cat <<'EOF'
feat(cluster-a): substrate ingest — atuin, stcortex_consumer, injection_db

m1 atuin_ingest: cursor-based pagination (novel, ~30 LOC), WAL read-only,
  PRAGMA query_only=ON, subprocess fallback. ToolCallRow typed fields.
  Source: memory-injection m06_schema (90%), m11_parallel_query (80%), m18_atuin_cache (70%).
  50 tests.

m2 stcortex_consumer: narrowed subscription (tool_call + consumption_event only).
  RegistrationHandle + ConsumerIdentity from_git_sha. trust gate for m13.
  Source: stcortex/rust-subscriber (80%), capacity.rs (90%).
  50 tests. Consumer registered: workflow-trace-<sha> on workflow_trace_* namespace.

m3 injection_db_ingest: CausalChainRow with ChainType/ConsentLevel typed enums.
  read_unresolved (reinforcement_count DESC), read_recently_resolved (recency window).
  Source: memory-injection m06_schema (90%), m07_causal_chain (70-80%).
  50 tests.

CC-1 (Cascade-Cost Coupling) data path: m1 rows → m4/m5/m6 (later clusters).
CC-2 (Trust Layer Woven): m9 namespace guard called at m2 registration time.
4-stage gate: PASS. Tests: [report exact count here].

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Day 3: Integration Smoke + WCP Notify

**Goal:** m1, m2, m3 read raw substrates and produce output that can be observed in a test harness. m9 namespace guard runtime check passes against the live stcortex registration. m10 Ember gate CI run passes with all Day 0-2 strings registered. Full 4-stage gate runs across all modules authored so far. First WCP notify to Watcher fires. Hand-off state documented for Phase 2A.

### Integration smoke tests

```bash
# Step 3.1 — Integration smoke: m1 reads atuin, m2 registration is fresh, m3 reads chains
CARGO_TARGET_DIR=./target cargo test --lib --release -- integration 2>&1 | tail -30

# Integration test assertions:
# 1. m1: open_atuin_db returns Ok; first page contains > 0 rows
# 2. m2: RegistrationHandle::is_fresh() returns true (within 30-day freshness window)
# 3. m3: read_unresolved returns Vec with at least 1 CausalChainRow
# 4. m9: assert_workflow_trace_namespace("workflow_trace_test") returns Ok
# 5. m9: assert_workflow_trace_namespace("orac_learn") returns Err(NamespaceViolation)
# 6. m10: ember_gate_all_user_facing_strings passes with all registered strings
# 7. m11: compute_decay_factor(1.0, 1.0, 1.0, 0.02) returns approximately 1.0 (within f64 epsilon)

# Step 3.2 — Full gate across all modules (check → clippy → pedantic → test)
CARGO_TARGET_DIR=./target cargo check --all-targets 2>&1 | tail -20 && \
cargo clippy --all-targets -- -D warnings 2>&1 | tail -20 && \
cargo clippy --all-targets -- -D warnings -W clippy::pedantic 2>&1 | tail -20 && \
CARGO_TARGET_DIR=./target cargo test --lib --release 2>&1 | tail -30

# Record exact test count: "Day 3 close: [N] tests passing, 0 warnings, 0 errors"
```

### Watcher namespace guard runtime check

```bash
# Step 3.3 — Runtime namespace guard spot-check
# This exercises the m9 guard against a real stcortex write attempt (test mode)
# The RegistrationHandle from m2 confirms the consumer is live

~/.local/bin/stcortex sql "SELECT name, namespace, is_stale FROM consumer WHERE namespace = 'workflow_trace_*'"
# Expected: "workflow-trace-<sha>   workflow_trace_*   false"
# If is_stale = true: m2 re-registration needed
```

### First WCP notify to Watcher

The Day 3 WCP notice fires after the full gate passes and the smoke tests pass. It is a structured file dropped to `~/projects/shared-context/watcher-notices/` per the WCP convention (CLAUDE.local.md §WCP Notify Weaver pattern).

```bash
# Step 3.4 — WCP notify (Day 3 close)
WCP_TIMESTAMP=$(date -u +%Y-%m-%dT%H%M%S)
cat > ~/projects/shared-context/watcher-notices/${WCP_TIMESTAMP}_workflow_trace_phase1_day3.md << 'EOF'
---
from: Command (Tab 1 Orchestrator)
to: Watcher ☤
subject: workflow-trace Phase 1 — Day 3 close
timestamp: [fill from WCP_TIMESTAMP]
---

## Phase 1 Day 0-3 Summary

**Cluster D (trust layer):** m8/m9/m10/m11 SHIPPED. 4-stage gate PASS. [N] tests.
- m8 build.rs: POVM_CR2_DEPLOYED gate active. compile_error! tombstone in place.
- m9 namespace guard: workflow_trace_* prefix enforced at application layer.
- m10 Ember gate CI: all user-facing strings passing 7-trait rubric (W3-as-fail active).
- m11 decay formula: compute_decay_factor NEW PRIMITIVE. 228d calibration verified.

**Cluster A (substrate ingest):** m1/m2/m3 SHIPPED. 4-stage gate PASS. [N] tests.
- m1 atuin_ingest: cursor pagination live. ~263K rows readable.
- m2 stcortex_consumer: consumer registered as workflow-trace-<sha> on workflow_trace_* namespace.
- m3 injection_db_ingest: CausalChainRow typed enums. [N] unresolved chains readable.

**Cat 10 foundation:** m01/m02/m03/m05 SHIPPED. 4-stage gate PASS. [N] tests.

**Class-E flag:** RESOLVED at commit [SHA] (Day 0 first cargo check PASS).

**Substrate state:**
- stcortex namespace workflow_trace_*: LIVE (consumer fresh, not stale)
- atuin history: readable, cursor-based pagination verified
- injection.db: readable, [N] unresolved chains
- POVM: CR-2 env marker [set/not-set] — D10 metric [active/inactive accordingly]

**Open items for Phase 2A:**
- G9 gate: REMAINS THE GATE — confirm with Luke that Phase 2A can proceed
- Conductor Waves 1B/1C/2/3: still auto_start=false (OI-7) — m32 blocked until Luke runs devenv start
- Watcher §5.1 amendment: still pending — Held verdicts still CI-fail per W3 flag
- LTP/LTD substrate: 35× below target band (0.043 ratio; target 1.5-4.0). Watcher Class-I pre-positioned.

Watcher, please log this notice verbatim as your T+[N]h deployment-watch journal entry.
EOF

echo "WCP notice written: ~/projects/shared-context/watcher-notices/${WCP_TIMESTAMP}_workflow_trace_phase1_day3.md"
```

### Day 3 failure modes + Watcher flags

| Failure | Watcher class | Recovery |
|---|---|---|
| Integration smoke: m2 stale after Day 2 (freshness window expired) | Class-B (hand-off boundary) | Call `register_narrowed_consumer` again; SpacetimeDB `register_consumer` is idempotent |
| Full gate pedantic fails on integration test file | Class-F | Add `#![allow(clippy::pedantic)]` to `tests/integration_smoke.rs` only; lib code must remain pedantic-clean |
| Ember gate: Day 3 strings (WCP notice template text in constants) fail gate | Class-F | WCP notice text is not user-facing output; do not register it in `user_facing_strings::ALL` |
| Watcher Class-I: LTP/LTD < 0.05 at Day 3 close | Class-I (Hebbian silence) | Pre-positioned flag; note in Day 3 WCP notice; do not defer Phase 2A on this basis (substrate condition is a monitoring signal, not a gate) |

### Day 3 commit

```bash
git add tests/integration_smoke.rs  # integration smoke tests
git commit -m "$(cat <<'EOF'
test(integration): Phase 1 smoke — m1+m2+m3 substrate reads + cluster-D runtime checks

Integration smoke: atuin page-1 reads OK, stcortex consumer fresh, injection chains readable.
m9 namespace guard: runtime assertion passes on valid prefix, fails on orac_ prefix.
m10 Ember gate: all user-facing strings PASS (W3-as-fail mode; 0 HELD verdicts).
m11 decay formula: compute_decay_factor round-trip asserted across all Day 1-2 modules.

Full gate: check → clippy → pedantic → test --lib --release. Tests: [N] PASS, 0 warnings.

WCP notice filed: workflow-trace Phase 1 Day 3 close → Watcher.

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Hand-off to Phase 2A

Phase 2A opens when Day 3 commit is clean and the WCP notify has been filed. The following state is true when Day 3 closes:

**What exists:**

| Item | State |
|---|---|
| Cargo workspace | Single crate, two binaries (wf-crystallise, wf-dispatch), feature gates (api/intelligence/monitoring/evolution/full) |
| `plan.toml` | All 26 modules declared with phase markers; Day 0-3 modules marked `status = "gate-pass"` |
| `bacon.toml` | 4-stage on_success chain configured and verified |
| Cat 10 foundation | m01_core_types, m02_error_taxonomy, m03_config, m05_metrics_collector — 4-stage PASS |
| Cluster D | m8 build.rs (CR-2 gate), m9 (namespace guard), m10 (Ember gate CI), m11 (decay formula) — 4-stage PASS |
| Cluster A | m1 atuin_ingest (cursor pagination), m2 stcortex_consumer (registered), m3 injection_db_ingest — 4-stage PASS |
| stcortex namespace | `workflow_trace_*` LIVE, consumer `workflow-trace-<sha>` registered, fresh |
| Test count | [N] tests across all modules (target: 123 Cluster D + 150 Cluster A + ~200 Cat 10 foundation = ~470+) |
| Watcher flags | Class-E RESOLVED. Class-I pre-positioned (LTP/LTD below target — monitoring only). |
| WCP notify | Filed to `~/projects/shared-context/watcher-notices/` |

**What does NOT yet exist (Phase 2A targets):**

- Cluster B (m4 cascade_correlator, m5 battern_step_record, m6 context_cost_record) — habitat observation layer
- Cluster C (m7 workflow_arc_record, m12 report_emitter, m13 stcortex_writer_narrowed) — central correlation hub
- m7 SQLite schema (`migrations/0001_init.sql`) and the `workflow_trace_events` table
- First stcortex write (m13 is the write path; Day 2 only opened the namespace, did not write to it)
- HTTP server or CLI surface (no `main.rs` binary logic yet; only library modules)

**Gate state for Phase 2A:**

- G9 remains the activation gate for each subsequent phase. Luke must emit "proceed with Phase 2A" or equivalent signal before Cluster B/C work begins.
- OI-7 (Conductor live): m32 dispatch cannot be wired until Luke runs `devenv start weaver/zen/enforcer`. Phase 2A does not include m32.
- W3 (Ember §5.1 amendment): Held verdicts remain CI-fail until Watcher amends the rubric. Phase 2A strings must be authored with this constraint active.
- CR-2 (`POVM_CR2_DEPLOYED=1`): must be set in the environment for m8 to allow POVM-reading paths to compile. Phase 2A's m13 (stcortex writer) does not read POVM directly; the CR-2 gate is latent until m42 (Cluster H) ships.

---

## Summary: Phase 1 Atuin Trajectory

The following commands are recorded in atuin history across all four days and are queryable post-phase via `atuin search --workspace workflow-trace`:

```
habitat-bootstrap
habitat-intel
habitat-fingerprint
curl -s -o /dev/null -w '%{http_code}' ... (×13 health checks)
cargo init
cargo check (multiple)
cargo clippy (multiple)
cargo test (multiple)
git add (multiple)
git commit (multiple)
sqlite3 ~/.local/share/atuin/history.db "SELECT COUNT(*) ..."
sqlite3 ~/.local/share/habitat/injection.db "SELECT COUNT(*) ..."
~/.local/bin/stcortex status
~/.local/bin/stcortex consumers
~/.local/bin/stcortex sql "SELECT ..."
```

The atuin trajectory is the proprioceptive ground truth for Phase 1. Any discrepancy between declared progress and the atuin record is a Drift signal — orchestrator must re-exercise independently (LCM Drift #11 discipline).

---

*Phase 1 recipe authored 2026-05-17 (S1002029). Planning-only. AP24 gate (G9 signal) is prerequisite. No code in this document.*

*Back to: [[../HOME]] · [[../GOD_TIER_CONSOLIDATION_S1001982]]*
