# QUICKSTART — workflow-trace

> **Back to:** [`README.md`](README.md) · [`CLAUDE.md`](CLAUDE.md) · full CLI reference: [`docs/COMMAND_MAPPING.md`](docs/COMMAND_MAPPING.md)

A hands-on, copy-pasteable getting-started guide for the two `workflow-trace` binaries: `wf-crystallise` (observe → mine → propose) and `wf-dispatch` (bank → select → verify → dispatch).

---

## 1. Prerequisites

### Required

- **Rust toolchain** — stable, `rustc 1.75` or newer (the crate sets `rust-version = "1.75"`). Install via [rustup](https://rustup.rs/). `cargo build --release` needs nothing else: SQLite is vendored (`rusqlite` with `bundled`), TLS is `rustls` (no system OpenSSL).

### Optional — habitat live services

Both binaries run end-to-end **without any habitat services**. The default-safe modes (`--offline` for `wf-crystallise`, `--dry-run` for `wf-dispatch`) deliberately avoid every live service. Live mode contacts these:

| Service | Used by | Default endpoint | When needed |
|---|---|---|---|
| stcortex | `wf-crystallise` m2 consumer | `ws://127.0.0.1:3000` | only without `--offline` |
| SYNTHEX v2 | `wf-crystallise` m40 nexus emit | `http://127.0.0.1:8092/v3/nexus/push` | only without `--offline` |
| HABITAT-CONDUCTOR | `wf-dispatch` m32 dispatcher | `http://127.0.0.1:8141` | only with `--execute` |

A down service is **logged and skipped** (recorded as a skipped stage / refused candidate), never a crash. You can ignore this whole table for a first run.

### Input data sources (read-only)

`wf-crystallise` reads two SQLite files. It never writes to either:

- `~/.local/share/atuin/history.db` — atuin shell history (override with `--atuin-db`).
- `~/.local/share/habitat/injection.db` — the habitat causal-chain ledger (override with `--injection-db`).

If you do not have these files, point the flags at a copy or an empty SQLite DB — the pipeline still runs (it simply observes zero rows).

---

## 2. Build

```bash
cd /home/louranicas/claude-code-workspace/the-workflow-engine
cargo build --release
```

This produces two binaries in `target/release/`:

- `target/release/wf-crystallise`
- `target/release/wf-dispatch`

> **Note — a build warning is expected.** Unless `POVM_CR2_DEPLOYED=1` is set, `build.rs` prints a `cargo:warning` about POVM CR-2 not being verified. This is by design: `workflow-trace` routes substrate feedback through stcortex, not POVM, so the warning is informational and does not fail the build or affect any stcortex-only path.

Quality gate (run before any commit):

```bash
cargo check  --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic
cargo test   --all-targets --all-features --release   # 1967 tests
```

---

## 3. Step 1 — `wf-crystallise`: observe and crystallise proposals

```bash
target/release/wf-crystallise --offline --proposals-out /tmp/proposals.jsonl
```

What this does, stage by stage:

1. **Ingest (m1, m3).** Opens `~/.local/share/atuin/history.db` and `~/.local/share/habitat/injection.db` read-only and pulls atuin history rows and unresolved causal chains.
2. **Correlate (m4).** Groups atuin steps into opaque multi-pane cascade clusters.
3. **Record (m7).** Opens the workflow-runs SQLite store (`./workflow_runs.db` by default), inserts a new run, and merges every cascade cluster and causal chain onto it as an observation.
4. **Lift evidence (m14).** Computes a Wilson-CI lift snapshot over the accumulated open runs.
5. **Mine → propose (m20 → m21 → m23).** Mines per-session sequential patterns with PrefixSpan, enumerates bounded variants, composes proposals.
6. **Write JSONL.** Writes each composed proposal as **one compact JSON object per line** to `--proposals-out` (here `/tmp/proposals.jsonl`).

`--offline` skips the stcortex consumer (m2) and the SYNTHEX nexus emit (m40); those appear in the report under `stages skipped`. The pipeline always runs to completion.

The printed report ends with `proposals written` and `completed : yes`. Add `--format json` for a machine-readable single-line report.

> **The F2 evidence gate.** A *single fresh run* against a new `--runs-db` sees `n == 1` open runs — far below the 20-sample floor (m14 `MIN_SAMPLE_SIZE`). The m23 proposer honestly refuses every proposal below that floor, so the first run typically writes **0 proposals**. Evidence accumulates across invocations when you reuse the same `--runs-db` (or its default `./workflow_runs.db`); once **≥20 runs** have accumulated, the lift snapshot becomes significant and proposals start to appear. This is intentional: no proposal is made on thin evidence.

---

## 4. Step 2 — `wf-dispatch`: verify and (dry-run) dispatch

```bash
target/release/wf-dispatch --proposals-in /tmp/proposals.jsonl --dry-run
```

What this does:

1. **Load.** Reads `/tmp/proposals.jsonl` — one JSON-encoded proposal per line (blank lines skipped; a malformed line aborts).
2. **Bank (m30).** Accepts each proposal into an in-memory curated bank.
3. **Select (m31).** Scores the bank by a weighted composite (`α·fitness + β·recency + γ·frequency + δ·diversity`) and picks the top-K (default 5).
4. **Verify (m33).** Runs the four-verifier gate (Security / Consistency / Cost / Ember) on each selected candidate — all four must agree.
5. **Plan (dry-run).** In `--dry-run` mode the Conductor is **never contacted**; each verified candidate is printed with disposition `dry-run`. Verifier-rejected candidates show `verifier-blocked`.

The printed report shows `mode : dry-run`, the per-candidate dispositions, and `dispatched : 0` (dry-run never dispatches).

> **The crystallise → dispatch handoff** is the proposals JSONL file. `wf-crystallise --proposals-out FILE` writes it; `wf-dispatch --proposals-in FILE` reads it. There is no shared database or socket between the two binaries — the file *is* the contract.

---

## 5. Safe defaults and live mode

`--offline` and `--dry-run` are the **safe defaults** — they are what you want for exploration, testing, and CI:

- `wf-crystallise --offline` — skips every live-service stage; touches only local SQLite files.
- `wf-dispatch --dry-run` — verifies and selects but never contacts HABITAT-CONDUCTOR. This is the *default* for `wf-dispatch`; you do not even need to pass the flag.

To run for real:

- **`wf-crystallise` live mode** — simply omit `--offline`. The m2 stcortex consumer and m40 nexus emit are then attempted; an unreachable service degrades to a skipped stage, so live mode is still safe to try.
- **`wf-dispatch` live dispatch** — pass `--execute`. This flips off the dry-run default and performs a **real dispatch to HABITAT-CONDUCTOR**. It requires:
  - a reachable Conductor at `--conductor-url` (default `http://127.0.0.1:8141`), and
  - an `--ack-ceiling` matching or exceeding each workflow's escape-surface profile — the operator's explicit acknowledgement of the most-destructive surface a dispatch may touch (default `sandboxed`, the least-destructive).

  An unreachable Conductor under `--execute` degrades to a `refused` candidate, never a panic. The dispatcher still never spawns a process directly — every action goes through the Conductor's `lcm.loop.create` route.

---

## 6. Exit codes

Both binaries use the same scheme:

| Code | Meaning |
|---|---|
| `0` | Pipeline ran to completion — or `--help` / `--version` was requested. |
| `1` | A pipeline fault: a missing/unreadable input DB, an unwritable proposals output, a malformed proposals JSONL line, an unrecoverable bank fault, an invalid miner/selector configuration. A *down live service is not a fault* and does not produce exit 1. |
| `2` | A CLI argument error — an unknown flag, a missing flag value, an unparseable value, or a positional argument (none are accepted). The help text is printed to stderr. |

---

## 7. End-to-end in one block

```bash
cd /home/louranicas/claude-code-workspace/the-workflow-engine
cargo build --release

# Observe → mine → propose (offline, safe). First fresh run may write 0
# proposals until ≥20 runs have accumulated in ./workflow_runs.db.
target/release/wf-crystallise --offline --proposals-out /tmp/proposals.jsonl

# Bank → select → verify → plan (dry-run, safe). Prints the plan; no dispatch.
target/release/wf-dispatch --proposals-in /tmp/proposals.jsonl --dry-run
```

For the complete flag-by-flag reference of both binaries — every default, every environment variable — see [`docs/COMMAND_MAPPING.md`](docs/COMMAND_MAPPING.md).

---

> **Back to:** [`README.md`](README.md) · [`ARCHITECTURE.md`](ARCHITECTURE.md) · [`docs/COMMAND_MAPPING.md`](docs/COMMAND_MAPPING.md) · [`CLAUDE.md`](CLAUDE.md)
