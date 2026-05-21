# GOLD_STANDARDS — workflow-trace Rust god-tier reference

> **Canonical (authoritative):** [`ai_docs/optimisation-v7/STANDARDS/GOD_TIER_RUST.md`](ai_docs/optimisation-v7/STANDARDS/GOD_TIER_RUST.md) · [`ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md`](ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md)
> **Workspace charter (parent):** [`~/claude-code-workspace/CLAUDE.md`](../CLAUDE.md) § Quality Gate Protocol · § God-Tier Standards
> **Auto-memory:** [feedback_coding_standards_godtier](../../.claude/projects/-home-louranicas-claude-code-workspace/memory/feedback_coding_standards_godtier.md) · [feedback_god_tier_no_warnings_at_any_level](../../.claude/projects/-home-louranicas-claude-code-workspace/memory/feedback_god_tier_no_warnings_at_any_level.md)
> **This file:** summary mirror with workflow-trace specifics. The canonical file rules; this file routes.

---

## The 15 mandatory god-tier rules

1. **Zero `unwrap()` outside tests** — use `?` propagation + `thiserror` taxonomy (per [m1_foundation/error.rs](../the_maintenance_engine_v2/src/m1_foundation/error.rs))
2. **Zero `unsafe`** — if FFI required, isolate to one audited module with safety doc
3. **Zero clippy warnings** at `-D warnings` (default)
4. **Zero clippy::pedantic warnings** at `-D warnings -W clippy::pedantic`
5. **50+ tests per module minimum** (KEYSTONE Cluster F: 250+; budget table at [TEST_DISCIPLINE.md](ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md))
6. **Doc comments on all public items** — backticked identifiers (clippy `doc_markdown`)
7. **Structured tracing emit** — `tracing-subscriber` not `println!`/`eprintln!` (per [m1_foundation/logging.rs](../the_maintenance_engine_v2/src/m1_foundation/logging.rs))
8. **Newtype discipline** — `SessionId(String)` not raw `String`; per cluster A spec.
   **Accessor naming convention:** string-backed newtypes expose `as_str(&self) -> &str`;
   `Copy` scalar newtypes expose `get(self) -> T`; multi-field domain types expose one
   accessor per field, named after the field (`Pattern::support()`, `AcceptedWorkflow::weight()`).
   `as_*` signals a cheap borrow/view; `get`/field-name signals a returned `Copy` value.
9. **thiserror error enums** — no `Box<dyn Error>` in public APIs
10. **No `#![allow]` suppressions** — fix the actual code; never suppress pedantic
11. **`PIPESTATUS[0]` in gate scripts** — never `cargo … | tail` (AP-V7-13 cousin)
12. **4-stage zero-tolerance gate** — check → clippy → pedantic → test, abort on first failure
13. **Doc-comment style (//!)**: Layer / Deps / Tests / Features / Platform / Impl Notes / Related Docs (per ME v2 `resources.rs`)
14. **Full 4-stage gate on every change** — G9 fired 2026-05-17; HOLD-v2 lifted; `cargo build` / `cargo test` are authorised and required
15. **No new pre-existing-project-warnings excuse** — even if the project has pre-existing warnings, new code must be clean (per [feedback_god_tier_no_warnings_at_any_level](../../.claude/projects/-home-louranicas-claude-code-workspace/memory/feedback_god_tier_no_warnings_at_any_level.md))

---

## Quality gate (run before every commit; G9 unlock condition)

```bash
CARGO_TARGET_DIR=./target cargo check 2>&1 | tail -20 && \
cargo clippy -- -D warnings 2>&1 | tail -20 && \
cargo clippy -- -D warnings -W clippy::pedantic 2>&1 | tail -20 && \
CARGO_TARGET_DIR=./target cargo test --lib --release 2>&1 | tail -30
```

For workflow-trace specifically (post-G9):
```bash
~/.local/bin/scaffold-gen --verify .                   # plan.toml ↔ src/ alignment
cargo --features=full check
cargo --features=full clippy -- -D warnings -W clippy::pedantic
cargo --features=full test
cargo --features=full bench --no-run               # F cluster KEYSTONE
```

---

## Test discipline (per `STANDARDS/TEST_DISCIPLINE.md`)

| Cluster | Modules | Min tests | KEYSTONE bonus |
|---|---|---:|---|
| A Substrate Ingest | 3 | 150 | — |
| B Habitat Observation | 3 | 180 | property tests for opaque-ID determinism |
| C Correlation + Output | 3 | 180 | integration tests for CC-1 |
| D Trust (cross-cutting) | 4 | 250 | property tests for m11 decay formula |
| E Evidence + Pressure | 2 | 140 | Wilson-CI boundary tests |
| **F Iteration (KEYSTONE)** | 4 | **250+** | bench + property + fuzz |
| G Bank/Select/Dispatch/Verify | 4 | 220 | integration for CC-4/CC-5/CC-6 |
| H Substrate Feedback | 3 | 200 | async + integration for outbox-survival |
| **Total** | **26** | **~1,562-1,599** | |

---

## File header convention (per ME v2 `m1_foundation`)

```rust
//! # m20_prefixspan_miner
//!
//! - **Layer**: L6 (Iteration KEYSTONE, Cluster F)
//! - **Deps**: workflow_core::types::{ToolCallRow, SessionId}, workflow_core::errors::IterError
//! - **Tests**: unit (PrefixSpan invariants) + property (mining determinism) + bench (10k rows)
//! - **Features**: intelligence
//! - **Platform**: any
//! - **Impl Notes**: PrefixSpan algorithm chosen over Apriori/n-gram per Gap 1 spec ([cluster-F](../../../ai_specs/modules/cluster-F/m20_prefixspan_miner.md)); operates on session-grouped ToolCallRow iterator
//! - **Related Docs**: [cluster-F spec](../../../ai_specs/modules/cluster-F/m20_prefixspan_miner.md) · [ULTRAMAP](../../../ai_docs/optimisation-v7/ULTRAMAP.md) View 2 row m20
```

---

> **Back to:** [`README.md`](README.md) · [`ANTIPATTERNS.md`](ANTIPATTERNS.md) · [`PATTERNS.md`](PATTERNS.md) · [`ARCHITECTURE.md`](ARCHITECTURE.md)
