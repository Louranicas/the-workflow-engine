# the-workflow-engine — `workflow-trace`

> **Status:** G9 fired · HOLD-v2 lifted · 26-module Rust codebase implemented (~31k LOC) · `workflow_core` lib + **4 binaries** (`wf-crystallise`, `wf-dispatch`, `wf-poller`, `wf-daemon`) · **habitat-managed service on port 8142** (S1005032 Wave-16) · 1967 tests passing · clippy + pedantic clean.
> **Cargo package:** `workflow-trace` v0.1.0 — library `workflow_core`; binaries `wf-crystallise` + `wf-dispatch` (invoke-and-exit CLIs), `wf-poller` (operator-launched continuous CLI, Wave-15), `wf-daemon` (habitat-managed service shape on `:8142` with `/health` + embedded poller subsystem, Wave-16).
> **Directory:** `the-workflow-engine/` (a rename to `workflow-trace/` is deferred — cosmetic).
> **Habitat-service grid:** visible as `WFE` in the Zellij habitat-plugin (14-service grid: V3 Nerve TL SX V8 VMS POVM RM PV2 ORAC Inj **WFE** ME PSw). Registered in `~/.config/devenv/devenv.toml` as `id = "workflow-trace"` with `auto_start = true`, `auto_restart = true`.

---

## What workflow-trace IS

`workflow-trace` is a **cascading-command + Battern-protocol + context-window observation engine** for the Zellij habitat. It does three things, in order:

1. **Observes.** It reads the habitat's own activity substrates — atuin shell history, the stcortex memory substrate, and the `injection.db` causal-chain ledger — and correlates them into structured workflow-run records (multi-pane cascade clusters, Battern-protocol step records, per-session context-cost records).
2. **Mines and proposes.** It mines repeated sequential patterns out of the observed history (PrefixSpan, gap-allowed), enumerates bounded mutations of the top patterns, scores them against Wilson-CI lift evidence, and composes **workflow variants as proposals for human review**. It never auto-promotes a proposal.
3. **Dispatches.** Ratified proposals are banked, scored by a weighted composite, gated through a four-agent verifier, and **dispatched via HABITAT-CONDUCTOR** — never by spawning a process, shell, or pane directly.

It is an observation-and-suggestion engine with a human in the loop and a Conductor between it and any real action. Two structural disciplines run through every cluster: the **trust regime** (namespace-prefix guards, an Ember-rubric output gate, an evidence floor that refuses proposals below 20 accumulated runs) and **graceful degradation** (a down live service is logged and skipped, never a panic).

---

## The 26-module / 8-cluster architecture

The 26 modules group into 8 synergy clusters (A–H). One line each:

| Cluster | Modules | Role |
|---|---|---|
| **A** — Substrate Ingest | m1, m2, m3 | Read-only ingress: atuin history SQLite (m1), narrowed-scope stcortex WebSocket consumer (m2), `injection.db` causal-chain ledger (m3). |
| **B** — Habitat Observers | m4, m5, m6 | Correlate atuin steps into opaque cascade clusters (m4), observe Battern-protocol step records (m5), track per-session context-cost + exploration-rate baseline (m6). |
| **C** — Correlation + Output | m7, m12, m13 | Central `workflow_runs` SQLite hub (m7), pure CLI report formatters (m12), the LTP/LTD-gated stcortex writer (m13). |
| **D** — Trust (cross-cutting) | m8, m9, m10, m11 | POVM CR-2 build-prereq gate (m8), `workflow_trace_` namespace guard (m9), 7-trait Ember-rubric output CI gate (m10), `frequency × fitness × recency` compound-decay primitive (m11). |
| **E** — Evidence + Pressure | m14, m15 | Wilson 95% CI lift aggregator with a 20-sample floor (m14), forbidden-verb pressure witness emitting durable JSONL events (m15). |
| **F** — Iteration (KEYSTONE) | m20, m21, m22, m23 | PrefixSpan gap-allowed sequential pattern mining (m20), bounded variant mutation (m21), k-means feature clustering (m22), evidence-gated proposal composition (m23). |
| **G** — Bank + Select + Dispatch + Verify | m30, m31, m32, m33 | Accepted-proposal curated bank with sunset/decay (m30), weighted-composite top-K selector (m31), HABITAT-CONDUCTOR dispatcher (m32), 4-agent verification gate (m33). |
| **H** — Substrate Feedback | m40, m41, m42 | `NexusEvent` push to SYNTHEX (m40), `lcm.loop.create` JSON-RPC re-trigger (m41), Hebbian substrate-feedback via the m13 stcortex writer (m42). |

Full canonical map: [`ARCHITECTURE.md`](ARCHITECTURE.md). Per-module prescriptive specs: [`ai_specs/modules/cluster-{A-H}/`](ai_specs/modules/).

---

## The two binaries

`workflow-trace` ships as two thin CLI binaries over the shared `workflow_core` library. Each `main()` parses arguments, hands off to a library orchestration module, prints a report, and sets an exit code — all pipeline logic is library code so it is integration-testable.

### `wf-crystallise` — observe → mine → propose

Owns clusters A–F + H (modules m1–m23 + m40–m42). It ingests atuin history and the `injection.db` ledger, correlates cascade clusters, records a workflow run into a `workflow_runs` SQLite store, computes a lift snapshot, mines sequential patterns, enumerates variants, composes proposals, and **writes each proposal as one JSON line to a proposals JSONL file**. Live-service stages (stcortex registration, SYNTHEX nexus emit) are attempted only when `--offline` is unset and degrade gracefully when a service is down.

### `wf-dispatch` — bank → select → verify → dispatch

Owns cluster G (modules m30–m33). It reads the proposals JSONL produced by `wf-crystallise`, accepts each proposal into an in-memory curated bank, scores and selects the top-K, runs the four-verifier gate, and — only under `--execute` — dispatches the selected workflows to HABITAT-CONDUCTOR. `--dry-run` is the default-safe mode: it verifies and selects but never contacts the Conductor.

The `wf-crystallise` → `wf-dispatch` handoff is a **proposals JSONL file**: `wf-crystallise --proposals-out` writes it, `wf-dispatch --proposals-in` reads it.

---

## Build

Standard Cargo. The release binaries land in `target/release/`:

```bash
cargo build --release
# binaries: target/release/wf-crystallise, target/release/wf-dispatch
```

Prerequisites and concrete runnable examples are in [`QUICKSTART.md`](QUICKSTART.md).

---

## Quality gate

Every change passes the mandatory 4-stage zero-tolerance gate, in order:

```bash
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic
cargo test  --all-targets --all-features --release
```

`check → clippy → pedantic → test`, zero tolerance at every stage. No `unwrap()`/`expect()` outside tests, no `unsafe` (the binaries `#![forbid(unsafe_code)]`), doc comments on public items, 50+ tests per module.

**Test count: 1967 passing** (inline unit suites + integration tests in [`tests/`](tests/)), clippy and pedantic clean.

---

## Documentation

| Document | What it covers |
|---|---|
| [`QUICKSTART.md`](QUICKSTART.md) | Hands-on getting started — prerequisites, build, runnable `wf-crystallise` / `wf-dispatch` examples, the JSONL handoff, the F2 evidence gate, exit codes. |
| [`ARCHITECTURE.md`](ARCHITECTURE.md) | The 26-module / 8-cluster / 9-layer canonical architecture map. |
| [`docs/COMMAND_MAPPING.md`](docs/COMMAND_MAPPING.md) | Complete CLI reference for both binaries — every flag, default, exit-code semantics, environment variables. |
| [`GATE_STATE.md`](GATE_STATE.md) | Live G1–G9 gate + B1–B6 blocker record (G9 FIRED). |
| [`GOLD_STANDARDS.md`](GOLD_STANDARDS.md) | God-tier Rust reference standards the codebase is held to. |

Deeper navigation surfaces: [`CLAUDE.md`](CLAUDE.md) (project charter), [`CLAUDE.local.md`](CLAUDE.local.md) (live session-state delta), [`ai_docs/INDEX.md`](ai_docs/INDEX.md), [`ai_specs/INDEX.md`](ai_specs/INDEX.md), [`ultramap/README.md`](ultramap/README.md), and the Obsidian vault at [`the-workflow-engine-vault/HOME.md`](the-workflow-engine-vault/HOME.md).

---

## License

`MIT OR Apache-2.0` (see [`LICENSE`](LICENSE)). Not published to crates.io (`publish = false`).

---

> **Back to:** [`CLAUDE.md`](CLAUDE.md) · [`CLAUDE.local.md`](CLAUDE.local.md) · [`ai_docs/QUICKSTART.md`](ai_docs/QUICKSTART.md) · [`ai_docs/INDEX.md`](ai_docs/INDEX.md) · [`ai_specs/INDEX.md`](ai_specs/INDEX.md) · [`the-workflow-engine-vault/HOME.md`](the-workflow-engine-vault/HOME.md) · [`~/claude-code-workspace/CLAUDE.md`](../CLAUDE.md) (workspace charter)

*README rewritten 2026-05-22 (S1003733) — verified against the implemented `workflow_core` codebase and the S1003733 codebase map.*
