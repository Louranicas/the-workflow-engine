# PATTERNS — workflow-trace

> **Canonical sources:** [`ai_docs/optimisation-v7/GENERATIONS/G4-gold-standard.md`](ai_docs/optimisation-v7/GENERATIONS/G4-gold-standard.md) · [`the-workflow-engine-vault/boilerplate modules/Gold Standard Exemplars — Synthesis.md`](the-workflow-engine-vault/boilerplate%20modules/) · [`ai_docs/optimisation-v7/MODULE_PLANS/`](ai_docs/optimisation-v7/MODULE_PLANS/)
> **This file:** synthesis index of useful patterns lifted from deployment framework + ME v2 + LCM + ORAC.
> **Machine-readable:** [`.claude/patterns.json`](.claude/patterns.json)

---

## Architectural patterns

| Pattern | Source | Use in workflow-trace |
|---|---|---|
| **ORAC single-crate, multi-binary** | ORAC sidecar | One Cargo crate; `[[bin]]` × 2 (`wf-crystallise`, `wf-dispatch`); shared `workflow_core` lib in `src/lib.rs`. Avoids LCM 10-crate workspace overhead when there's one deployment target. |
| **Cluster D early-ship (trust before substrate)** | Phase 1 framework | m8/m9/m10/m11 on Day 1, before any Cluster A reader. Build-time invariants must exist before first reader is authored. |
| **`cargo:rustc-cfg=povm_calibrated` env-only gate** | m8 spec | Not a Cargo feature → cannot be bypassed by `--features full`. Defense-in-depth alongside DB-layer refuse-write. |
| **plan.toml as machine-readable PASS/FAIL authority** | LCM `habitat-loop-engine/plan.toml` | Phase transitions read plan.toml status field; prose declarations have no authority. Authoring block at top declares 26 modules upfront. |
| **Feature gate matrix (`api`/`intelligence`/`monitoring`/`evolution`)** | Phase 1 framework | Per-cluster control without workspace-crate complexity. Cluster D NOT gated (aspect-layer invariant). |
| **`workflow_core` shared lib INSIDE same crate** | ORAC m22 pattern | Library lives in `src/lib.rs` alongside `[[bin]]` entries; not a workspace member. Reduces gate-script complexity. |

---

## Module-level patterns (per ME v2 `m1_foundation`)

Per [`the-workflow-engine-vault/module specs/MODULE_SPECS_INDEX.md`](the-workflow-engine-vault/module%20specs/MODULE_SPECS_INDEX.md) § ME v2 patterns:

| ME v2 file | Pattern | Used in workflow-trace clusters |
|---|---|---|
| `resources.rs` | `//!` docstring style (Layer/Deps/Tests/Features/Platform/Impl Notes/Related Docs) | **ALL 8 clusters** |
| `error.rs` | thiserror error-enum taxonomy + propagation | A, C, D, G, H |
| `logging.rs` | tracing-subscriber + structured emit | A, C, D, H |
| `metrics.rs` | counter / gauge / histogram + rolling window | B, C, E, H |
| `signals.rs` | signal-observation pattern (passive) | B, E, H |
| `shared_types.rs` | newtype discipline (SessionId, ConsumerId, …) | A |
| `state.rs` | central state-table + transitions | C, D, G |
| `config.rs` | feature flags + env override + TOML | D (m8 build-script) |
| `tensor_registry.rs` | 12D tensor framing | B (m6 cost bands), F (per-edge confidence) |
| `self_model.rs` | engine-knows-about-its-own-state | G (m31 selection criteria) |
| `nam.rs` | substrate concepts | F (proposal-as-substrate-frame language) |

---

## Quality-gate patterns

| Pattern | Notes |
|---|---|
| **4-stage zero-tolerance gate** | `cargo check && cargo clippy -- -D warnings && cargo clippy -- -D warnings -W clippy::pedantic && cargo test --lib --release` — order matters; each stage aborts on first failure |
| **`PIPESTATUS[0]` in gate chains** | `cargo … \| tail` makes `$?` capture `tail`'s exit (always 0); use `${PIPESTATUS[0]}` and explicit per-stage abort (cousin to AP-V7-13 diagnostics theatre) |
| **bacon `on_success` chaining** | `check → clippy → pedantic` auto-pipeline; locations export to `.bacon-locations` for nvim integration |
| **50+ tests per module minimum** | Forces design questions that improve architecture; per [G6-test-discipline](ai_docs/optimisation-v7/GENERATIONS/G6-test-discipline.md) |
| **No `unwrap()` outside tests** | God-tier standard; use `?` propagation + thiserror taxonomy |
| **No `unsafe`** | God-tier standard; if needed, isolate to single audited module with FFI justification doc |
| **Doc comments on public items** | rustdoc-publishable; clippy `doc_markdown` requires backticked identifiers |

---

## Data-flow patterns

| Pattern | Use in workflow-trace |
|---|---|
| **Lazy cursor-based pagination (atuin SQLite)** | m1 — avoids full-table scan on 263k row history; busy-timeout 5000 ms; `PRAGMA query_only = ON` defense |
| **Subprocess CLI fallback** | m1 — if SQLite path unavailable, wrap `atuin` CLI with 5s timeout |
| **Reducer-callback dedup (stcortex consumer)** | m2 — narrows scope to tool_call + consumption events; dedup at consumer not at writer |
| **JSONL one-event-per-file (pressure register)** | m15 — atomic emit, no shared lock, append-only |
| **JSONB `consumer_inputs` column (m7 hub)** | C — F9 zero-weight column; CC-1 contract uses join, not internal struct sharing |
| **Wilson CI (m14 lift, m22 confidence)** | Returns None for n<20; avoids point-estimate over-claim |
| **EMA over Converged-excluded window (m6)** | 20-session EMA explicitly excludes Converged outcomes to avoid feedback bias |
| **Top-K-by-distance N=3 (m23 proposer)** | Normalised Levenshtein <0.25 same / 0.25-0.60 near-miss / >0.60 distinct |
| **3-band LTP/LTD gate (m13)** | substrate_LTP_density backpressure 0.15 threshold; deferred-write JSONL buffer when below |
| **EscapeSurfaceProfile ordinal enum** | m30 — unified destructiveness schema; m32 displays before each step; m9 enforces at namespace level |

---

## Test patterns (per `optimisation-v7/STANDARDS/TEST_DISCIPLINE.md`)

| Test kind | Use when | Cluster mapping |
|---|---|---|
| **unit** | Pure functions, single struct invariants | Everywhere; default |
| **integration** | Cross-module contracts (CC-1..CC-7) | Test directory `tests/` with one file per contract |
| **async** | tokio-spawned tasks | H (m40/m41/m42 substrate emit) |
| **property** | Algorithmic invariants (PrefixSpan, Wilson CI, Levenshtein) | F (m20/m21/m22/m23) |
| **doctest** | Public API examples | All public surface in `workflow_core` lib |
| **bench** | Performance-critical hot loops | F (m20 PrefixSpan), B (m4 correlator at scale) |

Test count allocation (from [`STANDARDS/TEST_DISCIPLINE.md`](ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md)): **1,562-1,599 total** across 26 modules; min 50/module; KEYSTONE Cluster F gets 250+.

---

## Substrate-write patterns (Cluster H)

| Pattern | Use |
|---|---|
| **NexusEvent typed envelope** | m40 → SYNTHEX v2 :8092 `/v3/nexus/push`; typed schema (Option A untyped JSON for MVP per Cluster H spec) |
| **LCM RPC `lcm.loop.create`** | m41 → LCM MCP stdio; NOT hypothetical `lcm.deploy` (specifically called out as wrong endpoint in vault) |
| **stcortex pathway emit** | m42 → fitness_delta `PassVerified=+0.25, Pass=+0.15, Blocked=−0.05, Fail=−0.10` |
| **outbox-first JSONL buffer** | m40/m41/m42 — write to outbox file first, then RPC; survives substrate outage |
| **circuit breaker on 2 consecutive peer failures** | m40/m41/m42 — open breaker, fall back to outbox-only, log Watcher Class-I |

---

> **Back to:** [`README.md`](README.md) · [`ANTIPATTERNS.md`](ANTIPATTERNS.md) · [`GOLD_STANDARDS.md`](GOLD_STANDARDS.md) · [`ARCHITECTURE.md`](ARCHITECTURE.md)
