---
title: cross-cutting/feature-gating — default/full/api/intelligence/monitoring/evolution
date: 2026-05-17
status: SPEC
axes: [feature-flags, build-profile]
---

# Feature Gating — Module-Side Guidance

> **Back to:** [`../INDEX.md`](../INDEX.md) · [`../../README.md`](../../README.md) · [`../../ARCHITECTURE.md`](../../ARCHITECTURE.md) · per-cluster gate posture in `layers/cluster-*.md`

## Feature matrix

| Feature | Default? | Modules gated | Rationale |
|---|---|---|---|
| `api` | ON | m7, m12, m13 (Cluster C); m30, m31, m32, m33 (Cluster G) | CLI surface + external write paths |
| `intelligence` | ON in `full` | m14, m15 (Cluster E); m20, m21, m22, m23 (Cluster F) | iteration KEYSTONE + evidence/pressure |
| `monitoring` | ON in `full` | m40, m41, m42 (Cluster H) | substrate-feedback emit + metrics |
| `evolution` | ON in `full` | (reserved for post-D120 L9 substrate-frame engine) | not yet allocated |
| `full` | composite | api + intelligence + monitoring (+ evolution when allocated) | production profile |
| `default` | composite | api only (minimal "headless ingest" build) | debug / dev profile |

**Cluster D is NOT feature-gated** — m8/m9/m10/m11 are the trust layer; turning them off breaks F5/F11/AP30 mitigations habitat-wide. This is a structural fact locked in Genesis v1.3 § 1 + § 4.

**Cluster A is NOT feature-gated** — substrates are mandatory inputs; gating an ingest module means the engine cannot read its own evidence.

**Cluster B is NOT feature-gated** — observation is mandatory infrastructure.

## Cargo.toml shape (post-G9)

```toml
[features]
default = ["api"]
full = ["api", "intelligence", "monitoring"]
api = []
intelligence = []
monitoring = []
evolution = []  # reserved post-D120
```

## Module-level `#[cfg(feature = "...")]` pattern

```rust
// in workflow_core/src/lib.rs
#[cfg(feature = "intelligence")]
pub mod m20_prefixspan_miner;

#[cfg(feature = "intelligence")]
pub mod m21_variant_builder;

#[cfg(feature = "monitoring")]
pub mod m40_nexusevent_emit;

// Cluster D is never #[cfg]-gated:
pub mod m8_povm_build_prereq;
pub mod m9_watcher_namespace_guard;
pub mod m10_ember_ci_gate;
pub mod m11_fitness_weighted_decay;
```

## What turning each feature OFF means

| Feature OFF | Result |
|---|---|
| `api` off | no CLI; no external writes; effectively unusable binary |
| `intelligence` off | iteration KEYSTONE unlinked; engine becomes "passive recorder" |
| `monitoring` off | no Cluster H emit; CC-5 substrate-learning loop dead; no observability |
| `evolution` off (currently always off) | no effect; reserved |

The `default` profile (api only) is intended for debug builds where iteration + emit are too noisy. **Production runs MUST use `--features full`.**

## Build profile and CI gates

CI gates run against all relevant feature combinations:

```bash
# Stage 1: default profile compiles
cargo check --workspace --all-targets

# Stage 2: full profile compiles + tests pass
cargo check --workspace --all-targets --features full
cargo test --workspace --all-targets --features full --release

# Stage 3: each feature in isolation compiles (catches missing #[cfg] guards)
cargo check --workspace --no-default-features --features api
cargo check --workspace --no-default-features --features "api,intelligence"
cargo check --workspace --no-default-features --features "api,monitoring"
```

## Verify-sync invariants

- **#6** — every feature gate has ≥1 module with `#[cfg(feature = X)]`. `rg '#\[cfg\(feature' src/` per gate count > 0.
- Cluster D `#[cfg(feature)]` density check: `rg '#\[cfg\(feature' src/m{8,9,10,11}*/` returns 0 (never gated).

## Anti-patterns

- **Feature-gating a Cluster D module.** Trust layer must always be present.
- **Hidden feature-gate behaviour.** A module that silently behaves differently under `monitoring` off vs on without compile-time check is a runtime trap. Use `#[cfg]` to make the dependency explicit.
- **Mutually-exclusive features.** `api` + `no-api` style flags are confusing; prefer "off by default, opt-in".

---

> **Back to:** [`../INDEX.md`](../INDEX.md)
