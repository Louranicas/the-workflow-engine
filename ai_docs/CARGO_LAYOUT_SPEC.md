# workflow-trace — Cargo Layout Specification (planned Cargo.toml)

> **Back to:** [`README.md`](../README.md) · [`CLAUDE.md`](../CLAUDE.md) · [`ARCHITECTURE.md`](../ARCHITECTURE.md) · [`CODE_MODULE_MAP.md`](CODE_MODULE_MAP.md) · [`plan.toml`](../plan.toml) · [`G2-consolidation`](optimisation-v7/GENERATIONS/G2-consolidation.md)
>
> **Function:** Specification for the Cargo.toml workspace + binary + feature layout that the post-G9 scaffold will generate. This document REPLACES the actual `Cargo.toml` until G9 fires (`Cargo.toml` does not exist; authoring it pre-G9 is AP-Hab-01 / AP24 violation). The single source-of-truth for module list + feature matrix is [`plan.toml`](../plan.toml); this document documents the SHAPE the scaffold will produce. Status: planning-only · 0 LOC · Cargo.toml does NOT exist.

---

## 1. Pattern: ORAC single-crate (NOT LCM 10-crate workspace)

Per [Genesis v1.3 § 1](GENESIS_PROMPT_V1_3.md) and [G2-consolidation](optimisation-v7/GENERATIONS/G2-consolidation.md) § Canonical src/ layout:

- **One Cargo crate** (`workflow-trace`)
- **Two binary targets** (`wf-crystallise`, `wf-dispatch`)
- **In-crate library** (`workflow_core` lives in `src/lib.rs`, NOT a separate workspace member)
- **No `[workspace]` section** — single-crate keeps Cargo.lock simple and aligns with ORAC's pattern

**Why not LCM 10-crate?** The LCM 10-crate workspace generates per-phase Cargo coordination overhead. workflow-trace's 26 modules are tightly coupled through `workflow_core` shared types; splitting into 10 workspace members produces no compile-time isolation benefit and creates cross-crate version drift surface.

---

## 2. `[package]` section

```toml
[package]
name         = "workflow-trace"
version      = "0.1.0"                  # bumps from 0.0.0-spec.0 on first commit + cargo check green
edition      = "2021"
rust-version = "1.83"                   # workspace toolchain pin
authors      = ["Luke @ node 0.A", "Command (Tab 1)", "The Watcher ☤", "Zen (G7 audit)"]
license      = "MIT OR Apache-2.0"      # workspace default (TBD at G9 license decision)
description  = "Record cascading-command + Battern + context-window observations; propose workflow variants; dispatch ratified workflows via HABITAT-CONDUCTOR."
repository   = "https://github.com/Louranicas/workflow-trace"  # TBD; GitLab mirror also planned
readme       = "README.md"
keywords     = ["workflow", "battern", "habitat", "rust", "cli"]
categories   = ["development-tools", "command-line-utilities"]
```

---

## 3. `[lib]` section

```toml
[lib]
name = "workflow_core"
path = "src/lib.rs"
```

`workflow_core` re-exports all 26 `m<N>_<name>` modules + shared types/schemas/namespace/errors. The library is in-crate so both binaries (`wf-crystallise`, `wf-dispatch`) consume the same compiled `workflow_core` without rebuild churn.

---

## 4. `[[bin]]` × 2

```toml
[[bin]]
name = "wf-crystallise"
path = "src/bin/wf-crystallise/main.rs"
required-features = []                  # all features behind --features full; binary works at minimal feature set

[[bin]]
name = "wf-dispatch"
path = "src/bin/wf-dispatch/main.rs"
required-features = ["api"]             # m32 dispatch CLI surface gated behind api feature
```

**Per [`plan.toml`](../plan.toml) `[[bin_targets]]`:**

- `wf-crystallise` owns m1-m23 + m40-m42 (18 modules)
- `wf-dispatch` owns m30-m33 (4 modules)
- workflow_core lib owns shared types/schemas/namespace/errors

Both binaries link the same `workflow_core` lib; cluster-D aspects (m8/m9/m10/m11) are routed through `workflow_core` and apply to both.

---

## 5. `[features]` matrix

```toml
[features]
default      = ["full"]
full         = ["api", "intelligence", "monitoring", "evolution"]

api          = []   # m12 CLI reports, m32 dispatch CLI surface
intelligence = []   # m14, m15 evidence + m20-m23 iteration KEYSTONE
monitoring   = []   # m40, m41, m42 substrate feedback
evolution    = []   # m11 lifecycle/sunset decay
```

**Per [`CLAUDE.md`](../CLAUDE.md) Architecture § Feature gate matrix:**

- **Cluster D is NOT feature-gated.** m8/m9/m10/m11 are aspect-layer invariants that EVERY other module routes through. Feature-gating Cluster D would create paths where the trust regime can be bypassed.
- **m8's `cargo:rustc-cfg=povm_calibrated` is ENV-only, not a Cargo feature.** This is intentional — Cargo features can be toggled via `--features`, but env-only cfg cannot. Once compiled, the gate is baked in.
- **`evolution` feature gates m11's lifecycle cycle CLI but NOT m11's compile-time existence.** m11 always compiles; only the cron-driven `run_consolidation_cycle` CLI subcommand is gated.

### 5.1 Feature flag invariants

| Invariant | Reason |
|---|---|
| `default = ["full"]` | Production deployments always run all features; minimum-cargo use case is contrib local-dev only |
| All features additive (no exclusivity) | Standard Cargo convention; no `cfg(any(feature_a, not(feature_b)))` antipatterns |
| Cluster D never feature-gated | aspect-layer invariants; bypass = AP-Hab-03 / AP30 violation |
| m8 env cfg `povm_calibrated` not Cargo feature | cannot be toggled at link time; baked in at build |

---

## 6. `[dependencies]` (locked per `plan.toml`)

```toml
[dependencies]
serde              = { version = "1", features = ["derive"] }
serde_json         = "1"
thiserror          = "2"
tracing            = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tokio              = { version = "1", features = ["full"] }
rusqlite           = { version = "0.31", features = ["bundled"] }
reqwest            = { version = "0.12", features = ["json"] }
clap               = { version = "4", features = ["derive"] }
chrono             = { version = "0.4", features = ["serde"] }
uuid               = { version = "1", features = ["v4", "serde"] }
```

**Rationale per dependency:**

| Dep | Used by | Reason |
|---|---|---|
| `serde` + `serde_json` | all modules with JSON I/O | newtype derives + JSON ser/de |
| `thiserror` v2 | every module's `error.rs` | structured error taxonomy ([`ERROR_TAXONOMY.md`](ERROR_TAXONOMY.md)) |
| `tracing` + `tracing-subscriber` | every module | no `println!` in daemons (BUG-018 SIGPIPE); env-filter for runtime log control |
| `tokio` | m2/m13/m40/m41/m42 (async substrate I/O) | async runtime; `["full"]` includes net+fs+macros |
| `rusqlite` `["bundled"]` | m1/m3/m7/m30/m33 | bundled sqlite avoids system lib version drift |
| `reqwest` `["json"]` | m32 (Conductor), m40 (SYNTHEX) | HTTP client; JSON feature |
| `clap` `["derive"]` | both bin targets | CLI surface |
| `chrono` `["serde"]` | every module with timestamps | serializable timestamps + duration arithmetic |
| `uuid` `["v4", "serde"]` | m23/m30 (workflow_id), m31, m32 | v4 random; serde for persistence |

### 6.1 Deliberately absent from `[dependencies]`

| Crate | Why absent |
|---|---|
| `anyhow` | hard rule per [`CLAUDE.md`](../CLAUDE.md) — typed `thiserror` only; never `anyhow::Result<T>` in lib code |
| `eyre` | same — typed errors only |
| `parking_lot` | first-pass uses `std::sync::RwLock`; consider `parking_lot::RwLock` in Phase 4 hardening if contention warrants |
| `axum` | workflow-trace is CLI-not-service (Phase 5A); no HTTP daemon — no Axum |
| `tower-http` | n/a (no HTTP daemon) |
| `prometheus` | metrics via Pushgateway since CLI exits; lighter `prometheus_exporter` consideration deferred to Phase 8 |
| `sqlx` | rusqlite preferred for synchronous SQLite (no async DB needs) |

---

## 7. `[dev-dependencies]`

```toml
[dev-dependencies]
proptest   = "1"
criterion  = "0.5"
mockito    = "1"
assert_cmd = "2"
```

| Dep | Used by | Test kind |
|---|---|---|
| `proptest` v1 | m14 (Wilson CI), m11 (Gap 2 compound decay), m20-m22 (PrefixSpan + variant + K-means) | property-based; min 10,000 iters per AP-Test-02 |
| `criterion` 0.5 | `benches/m20_prefixspan.rs`, `benches/m32_dispatcher.rs`, etc. | benchmark harness (see [`PERFORMANCE.md`](PERFORMANCE.md)) |
| `mockito` v1 | m32 (Conductor HTTP mock), m40 (SYNTHEX HTTP mock) | HTTP server mocks; integration tests use real local services where possible (AP-Test-03 — mocks only for true externals) |
| `assert_cmd` v2 | tests/cli/*.rs | end-to-end CLI integration tests |

---

## 8. `[build-dependencies]` (for `build.rs`)

```toml
[build-dependencies]
# m8 build.rs uses no external crates; pure std::env + emit cargo: directives
```

m8 `build.rs` reads `POVM_CALIBRATED_BAND` env var, validates band, emits `cargo:rustc-cfg=povm_calibrated`. No external build deps required.

---

## 9. `build.rs` (m8 contract)

The presence of `build.rs` at crate root is REQUIRED. Auto-discovered by Cargo (does not need `[package] build = "build.rs"` declaration — Cargo finds it). Per [m8 spec](../ai_specs/modules/cluster-D/m8_povm_build_prereq.md):

```text
build.rs (PLANNED — does not exist pre-G9):

1. Read env var POVM_CALIBRATED_BAND (e.g., "0.05,0.15")
2. Validate band is well-formed
3. If env unset OR band malformed → compile_error!() OR println!("cargo:warning=…") + emit nothing
4. If band valid → println!("cargo:rustc-cfg=povm_calibrated")
5. Emit println!("cargo:rerun-if-env-changed=POVM_CALIBRATED_BAND")
```

**Why env-only (not Cargo feature):** Cargo features are link-time toggleable via `--features`. Env-cfg is baked at compile and CANNOT be flipped after. This is intentional for AP-WT-F7 (CR-2 graceful-degrade pretend-fix) mitigation — gate is immutable post-compile.

**Note:** Per 2026-05-17 m42 stcortex-only ADR, POVM is DECOUPLED from m42. m8 retains the `cargo:rustc-cfg=povm_calibrated` gate for `substrate_LTP_density` display-only code paths (m11 read-side), but POVM is no longer a runtime substrate target for the engine.

---

## 10. `[profile.*]`

```toml
[profile.dev]
opt-level     = 0
debug         = true
overflow-checks = true

[profile.release]
opt-level     = 3
debug         = false
lto           = "fat"           # full LTO for binary size + perf (per ORAC release pattern)
codegen-units = 1
strip         = "symbols"
panic         = "abort"          # binaries crash on panic — better signal than catch+continue

[profile.bench]
inherits = "release"             # benchmarks build with release profile
debug = true                     # but keep debug symbols for flamegraph

[profile.test]
inherits = "dev"
```

---

## 11. `[workspace]` — INTENTIONALLY ABSENT

There is **no `[workspace]` section** in workflow-trace's `Cargo.toml`. This is intentional:

- workflow-trace is a single crate (ORAC pattern, not LCM 10-crate)
- The parent `~/claude-code-workspace/` is NOT a Cargo workspace either — each project under it is independently published
- Per [`Cargo book`](https://doc.rust-lang.org/cargo/reference/workspaces.html), absence of `[workspace]` means the crate is standalone

**If you find yourself wanting `[workspace]`,** consider whether you actually want LCM's 10-crate pattern. The cost (~3-5x more Cargo.toml + Cargo.lock divergence + per-crate version churn) does not pay for itself at workflow-trace's scale (26 modules in one crate).

---

## 12. `[[bench]]` entries

```toml
[[bench]]
name    = "m20_prefixspan"
path    = "benches/m20_prefixspan.rs"
harness = false                  # criterion uses its own harness

[[bench]]
name    = "m4_cascade"
path    = "benches/m4_cascade.rs"
harness = false

[[bench]]
name    = "m7_hub"
path    = "benches/m7_hub.rs"
harness = false

[[bench]]
name    = "m32_dispatcher"
path    = "benches/m32_dispatcher.rs"
harness = false

[[bench]]
name    = "m11_decay"
path    = "benches/m11_decay.rs"
harness = false

[[bench]]
name    = "m14_lift"
path    = "benches/m14_lift.rs"
harness = false

[[bench]]
name    = "m22_kmeans"
path    = "benches/m22_kmeans.rs"
harness = false

[[bench]]
name    = "m31_selector"
path    = "benches/m31_selector.rs"
harness = false

[[bench]]
name    = "cluster_h_emit"
path    = "benches/cluster_h_emit.rs"
harness = false

[[bench]]
name    = "m33_verifier"
path    = "benches/m33_verifier.rs"
harness = false
```

See [`PERFORMANCE.md`](PERFORMANCE.md) for bench targets + critical thresholds.

---

## 13. Lint policy (planned `[lints]`)

```toml
[lints.rust]
unsafe_code = "forbid"

[lints.clippy]
unwrap_used      = "deny"     # no .unwrap() outside tests
expect_used      = "deny"     # no .expect() outside tests
panic            = "warn"     # panic! discouraged
pedantic         = { level = "deny", priority = -1 }
missing_docs_in_private_items = "warn"

# Module-level: per [GOD_TIER_RUST.md](optimisation-v7/STANDARDS/GOD_TIER_RUST.md), each module
# crate root carries:
#   #![forbid(unsafe_code)]
#   #![deny(missing_docs)]    on public items
#   #![warn(clippy::pedantic)]
```

Quality gate (per [`CLAUDE.md`](../CLAUDE.md)) is enforced at CI:
```
check → clippy -D warnings → clippy -D warnings -W clippy::pedantic → test
```

Each stage gated by `${PIPESTATUS[0]}` (AP-Hab-05 PIPESTATUS swallow mitigation).

---

## 14. Source-of-truth precedence

If `plan.toml` and `Cargo.toml` diverge:

1. **`plan.toml` wins on:** module list, feature matrix, layer ordering, dependency versions, test budget, LOC budget. `plan.toml` is the spec.
2. **`Cargo.toml` wins on:** build syntax, `[lib]` vs `[[bin]]` declarations, profile knobs. `Cargo.toml` is the build manifest.
3. **Drift detection:** AP-V7-05 (Module-plan-to-src-drift) + per-Wave-end sync check. Reject merge if budget exceeded by >2x.

Pre-G9, `Cargo.toml` does not exist — `plan.toml` IS the canonical declaration. Post-G9, scaffold-gen materialises `Cargo.toml` from `plan.toml`; drift is caught at every Wave-end.

---

## 15. Scaffold-gen invocation (post-G9 only)

```bash
# DO NOT RUN PRE-G9 — AP-Hab-01 / AP24 violation
scaffold-gen --from-plan plan.toml .
```

This will materialise:

- `Cargo.toml` (per this spec)
- `Cargo.lock` (after first `cargo check`)
- `src/lib.rs` + `src/types.rs` + `src/schemas.rs` + `src/namespace.rs` + `src/errors.rs`
- `src/m<N>_<name>/` directories for all 26 modules (with `mod.rs` stubs)
- `src/bin/wf-crystallise/main.rs` + `src/bin/wf-dispatch/main.rs`
- `build.rs` (m8 contract)
- `benches/` (10 bench file stubs)
- `tests/integration/cc<N>_*.rs` (7 CC closure test stubs)
- `migrations/` (SQLite schema migrations)

---

## 16. Cross-references

- **plan.toml (machine-readable source of truth):** [`../plan.toml`](../plan.toml)
- **Genesis v1.3 § 1 (architecture lock):** [`GENESIS_PROMPT_V1_3.md`](GENESIS_PROMPT_V1_3.md)
- **G2 consolidation (canonical src/ layout):** [`G2-consolidation`](optimisation-v7/GENERATIONS/G2-consolidation.md)
- **GOD_TIER_RUST standards:** [`STANDARDS/GOD_TIER_RUST.md`](optimisation-v7/STANDARDS/GOD_TIER_RUST.md)
- **TEST_DISCIPLINE:** [`STANDARDS/TEST_DISCIPLINE.md`](optimisation-v7/STANDARDS/TEST_DISCIPLINE.md)
- **Module map:** [`CODE_MODULE_MAP.md`](CODE_MODULE_MAP.md)
- **Performance targets:** [`PERFORMANCE.md`](PERFORMANCE.md)
- **Error taxonomy:** [`ERROR_TAXONOMY.md`](ERROR_TAXONOMY.md)

> **Back to:** [`README.md`](../README.md) · [`CLAUDE.md`](../CLAUDE.md) · [`ARCHITECTURE.md`](../ARCHITECTURE.md) · [`plan.toml`](../plan.toml)

*CARGO_LAYOUT_SPEC authored 2026-05-17 (S1001982) by Command. Documents the SHAPE the post-G9 scaffold will materialise; Cargo.toml does NOT exist pre-G9 (AP-Hab-01 / AP24); plan.toml is canonical source of truth. ORAC single-crate pattern locked; m8 env-cfg not Cargo feature preserved; POVM-decoupled m42 fact carried forward.*
