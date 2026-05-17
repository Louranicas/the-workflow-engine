# bin/wf-dispatch/

> **Back to:** [`../README.md`](../README.md) · [`../../README.md`](../../README.md) · [`../../CLAUDE.md`](../../CLAUDE.md) · [`../../CLAUDE.local.md`](../../CLAUDE.local.md)

**Status:** placeholder. `main.rs` lands post-G9 (HOLD-v2 forbids `.rs` files).

## Purpose

`wf-dispatch` is the **write-heavy binary** of the workflow-trace two-binary split. It owns the curated workflow bank (m30), the diversity-enforced selector (m31), the HABITAT-CONDUCTOR dispatcher (m32 — never dispatches directly to host shell), and the 4-agent verifier (m33). It does **not** ingest substrate observations — that's [`../wf-crystallise/`](../wf-crystallise/)'s job. The hard separation lets `wf-dispatch` ship with a much tighter blast radius: it reads `workflow_core` types + the m30 SQLite bank, calls Conductor, and writes back verdicts. **It cannot accidentally trigger `cargo build` of variants, cannot tail atuin, cannot mutate stcortex pathways outside m32's outbox-first JSONL path.**

## Owned modules (per [`../../ai_specs/MODULE_MATRIX.md`](../../ai_specs/MODULE_MATRIX.md))

- **Cluster G (bank + select + dispatch + verify):**
  - **m30** `m30_curated_bank` — post-operator-review bank in SQLite (~220 LOC; integration tests; **Gap 3 partial owner** for unified destructiveness schema)
  - **m31** `m31_selector` — diversity-enforced selection consuming m11 compound decay (~240 LOC; property tests)
  - **m32** `m32_conductor_dispatcher` — HABITAT-CONDUCTOR client owning the dispatch envelope (~290 LOC; integration; **Gap 3 primary owner** for EscapeSurfaceProfile schema; CC-4 + CC-6 contract surface)
  - **m33** `m33_verifier` — 4-agent verifier returning PASS/FAIL/DEGRADED to m30 (~200 LOC; integration; CC-6 contract owner)

Total: **4 of 26 modules**. Cluster G is the only cluster owned entirely by `wf-dispatch`.

## CLI entry point (planning)

```text
wf-dispatch [--config PATH] <command> [args]
  bank list [--status pending|approved|rejected]   # m30 read
  bank approve <id> | reject <id>                  # m30 operator action
  select [--strategy diversity|recency|fitness]    # m31 dry-run
  dispatch <bank_id> [--dry-run]                   # m32 (gated on m9 namespace + m33 verify)
  verify <bank_id>                                 # m33 4-agent verifier explicit invocation
  health                                           # health endpoint check (used by devenv)
```

Final CLI shape locks at G5 interview / G7 spec audit.

## Feature gate coverage

All Cluster G modules carry the `api` feature gate by default (the bank, selector, dispatcher and verifier are the API surface that operators interact with). `wf-dispatch` does NOT compile in `intelligence` (Cluster F) or `monitoring` (Cluster H) gates — those live in `wf-crystallise`. This keeps `wf-dispatch`'s binary size minimal and its dependency closure tight.

## Critical-path dependencies

`wf-dispatch` cannot ship until **Conductor Waves 1B/1C/2/3 are LIVE** (currently `auto_start=false`; see [`../../CLAUDE.local.md`](../../CLAUDE.local.md) blocker B3 — Luke @ terminal: `~/.local/bin/devenv -c ~/.config/devenv/devenv.toml start weaver/zen/enforcer`). m32 is the highest-risk module in the codebase (Gap 3 NEW schema + external Conductor dependency). Command-3 owns this lane.

## Boilerplate references

Lift from [`../../the-workflow-engine-vault/boilerplate modules/`](../../the-workflow-engine-vault/boilerplate%20modules/) categories `07-conductor-dispatch/` (habitat-conductor + dev-ops-engine-v3 dispatcher patterns), `09-trap-verify-escape-skills/` (EscapeSurfaceProfile shape from `.claude/skills/`). Total lift ~30% — lower than `wf-crystallise` because Gap 3 schema authorship is NEW.

> **Back to:** [`../README.md`](../README.md) · [`../../README.md`](../../README.md) · [`../../CLAUDE.md`](../../CLAUDE.md) · [`../../CLAUDE.local.md`](../../CLAUDE.local.md)
