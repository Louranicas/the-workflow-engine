# COMMAND_MAPPING — workflow-trace CLI reference

> **Back to:** [`../README.md`](../README.md) · [`../QUICKSTART.md`](../QUICKSTART.md) · [`../CLAUDE.md`](../CLAUDE.md)

Complete reference for both `workflow-trace` binaries — every CLI flag, its argument, default value, and behaviour; exit-code semantics; and the environment variables that affect runtime. Flags are sourced directly from the `parse_args` functions in `src/orchestration/crystallise.rs` and `src/orchestration/dispatch.rs`.

Both binaries take **flags only** — no positional arguments are accepted; a positional argument is a CLI error (exit 2). Flags that take a value use a separate following token (`--flag value`); there is no `--flag=value` form.

---

## `wf-crystallise`

Observe habitat workflows and crystallise proposals. Owns modules m1–m23 + m40–m42 (clusters A–F + H).

```
wf-crystallise [OPTIONS]
```

### Flags

| Flag | Argument | Default | Description |
|---|---|---|---|
| `--atuin-db` | `<PATH>` | `$HOME/.local/share/atuin/history.db` | Path to the atuin shell-history SQLite DB. Opened **read-only**; never written. |
| `--injection-db` | `<PATH>` | `$HOME/.local/share/habitat/injection.db` | Path to the habitat `injection.db` causal-chain ledger. Opened **read-only**; never written. |
| `--runs-db` | `<PATH>` | `./workflow_runs.db` | Path to the `workflow_runs` SQLite store. Created if absent. Reuse the same path across invocations to accumulate lift evidence (see the F2 gate note below). |
| `--proposals-out` | `<PATH>` | `./proposals.jsonl` | Output path for the proposals JSONL. Each composed proposal is written as one compact JSON object per line. The file is created/truncated. This file is the handoff to `wf-dispatch --proposals-in`. |
| `--min-support` | `<N>` | `3` | Minimum support for the m20 PrefixSpan miner — a pattern must appear in at least `N` sequences to be mined. |
| `--max-gap` | `<N>` | `5` | Maximum gap for the m20 miner — the largest allowed skip between matched steps in a gap-allowed pattern. |
| `--offline` | *(none)* | off (online) | Skip all live-service stages: the m2 stcortex consumer registration and the m40 SYNTHEX nexus emit. Skipped stages are recorded in the report under `stages skipped`. The default-safe mode. |
| `--format` | `text` \| `json` | `text` | Report rendering format. `text` is the human-readable multi-line report; `json` is a single-line JSON object. Any other value is a CLI error. |
| `--help`, `-h` | *(none)* | — | Print the help text to stdout and exit `0`. |
| `--version`, `-V` | *(none)* | — | Print `wf-crystallise <version>` and exit `0`. |

### Notes

- **The F2 evidence gate.** The m14 lift snapshot is computed over the accumulated *open* runs in `--runs-db`. A single fresh-DB run sees `n == 1`, below the 20-sample floor (`MIN_SAMPLE_SIZE`), so m23 honestly skips every proposal — a fresh run typically writes **0 proposals**. Reuse the same `--runs-db` until ≥20 runs accumulate for proposals to appear.
- **Graceful degradation.** A down live service (stcortex, SYNTHEX) is logged via `tracing` and recorded as a skipped stage — it is never a fault and never produces exit `1`.
- **`tracing` output.** The binary emits `tracing` events but installs no subscriber, so events are dropped by default; the printed report is the canonical output.

---

## `wf-dispatch`

Verify and dispatch crystallised workflow proposals. Owns modules m30–m33 (cluster G).

```
wf-dispatch [OPTIONS]
```

### Flags

| Flag | Argument | Default | Description |
|---|---|---|---|
| `--proposals-in` | `<PATH>` | `./proposals.jsonl` | Input path of the proposals JSONL produced by `wf-crystallise`. One JSON-encoded proposal per line; blank lines are skipped; a malformed line aborts with exit `1`. |
| `--top-k` | `<N>` | `5` | Number of workflows to select from the bank via the m31 weighted-composite scorer. |
| `--conductor-url` | `<URL>` | `http://127.0.0.1:8141` | HABITAT-CONDUCTOR base URL. The dispatch endpoint is this base plus `/dispatch`. Only contacted under `--execute`. |
| `--dry-run` | *(none)* | **on (default)** | Verify + select but never contact the Conductor. The default-safe mode. Each verified candidate is reported with disposition `dry-run`. |
| `--execute` | *(none)* | off | Perform a real dispatch via the Conductor. Overrides `--dry-run`. `--dry-run` / `--execute` are last-wins: the final occurrence decides the mode. |
| `--ack-ceiling` | `<PROFILE>` | `sandboxed` | The escape-surface ceiling the operator acknowledges. A workflow whose escape-surface profile exceeds the acknowledged ceiling is refused by m32. One of the 7 profiles listed below. |
| `--help`, `-h` | *(none)* | — | Print the help text to stdout and exit `0`. |
| `--version`, `-V` | *(none)* | — | Print `wf-dispatch <version>` and exit `0`. |

### `--ack-ceiling` profile values

The accepted `<PROFILE>` values map to the m32 `EscapeSurfaceProfile` enum, in ascending order of destructiveness (ordinal):

| Value | Ordinal | Meaning |
|---|---|---|
| `sandboxed` | 0 | Fully sandboxed; no escape surface. *(default)* |
| `sandbox-escape` | 10 | May escape the sandbox. |
| `process-mutate` | 20 | May mutate processes. |
| `privilege-escalation` | 30 | May escalate privilege. |
| `file-write` | 40 | May write to the filesystem. |
| `network-egress` | 50 | May make outbound network connections. |
| `data-exfil` | 60 | May exfiltrate data. |

Any other value is a CLI error (exit 2).

### Notes

- **Default-safe mode.** `wf-dispatch` defaults to `--dry-run`; you need not pass it. `--execute` is required to contact the Conductor.
- **Graceful degradation.** Under `--execute`, an unreachable Conductor / non-2xx response / unparseable body all collapse into a `refused` candidate disposition — never a fault, never exit `1`.
- **No direct execution.** The dispatcher never spawns a process, shell, or pane. A real dispatch is a single HTTP POST routed through the Conductor's `lcm.loop.create` method.
- **Per-candidate dispositions** in the report: `dry-run`, `dispatched`, `refused`, `verifier-blocked`.

---

## Exit codes (both binaries)

| Code | Meaning |
|---|---|
| `0` | Pipeline ran to completion, or `--help` / `--version` was requested. |
| `1` | A pipeline fault. For `wf-crystallise`: a missing/unreadable atuin or injection DB, a workflow-runs store that cannot be opened, an invalid miner/lift parameter, an unwritable proposals output. For `wf-dispatch`: a missing/malformed proposals file, an unrecoverable bank fault, an invalid selector configuration. A **down live service is not a fault**. |
| `2` | A CLI argument error — unknown flag, missing flag value, unparseable value, or an unexpected positional argument. The help text is printed to stderr. |

---

## Environment variables

| Variable | Phase | Affects | Effect |
|---|---|---|---|
| `HOME` | runtime | `wf-crystallise` | Resolves the default `--atuin-db` (`$HOME/.local/share/atuin/history.db`) and `--injection-db` (`$HOME/.local/share/habitat/injection.db`). If unset, both defaults fall back to a `/tmp`-rooted path. Explicit `--atuin-db` / `--injection-db` flags override regardless. |
| `POVM_CR2_DEPLOYED` | build | `build.rs` | When set to `1`, `build.rs` emits the `povm_calibrated` rustc-cfg and suppresses the CR-2 build warning. Set this only after confirming the POVM CR-2 (magnitude-weighted `learning_health`) fix is live. It is a `rustc-cfg`, **not** a Cargo feature — `--features full` / `--all-features` cannot activate it. |
| `POVM_HEALTH_URL` | runtime | m8 `m8_povm_build_prereq::resolve_health_url` | Overrides the m8 POVM health-probe URL (default `http://127.0.0.1:8125/learning_health`). The m8 runtime probe is KEEP-DORMANT in the current pipeline — no in-tree POVM-read site is reached — so this variable affects only a direct m8 probe call, not the standard `wf-crystallise` / `wf-dispatch` runs. |

### Hard-coded endpoints (not env-configurable)

These are baked into the library as constants. Some are overridable only through a constructor argument when the library is used directly, not via a CLI flag or environment variable:

| Endpoint | Default | Override |
|---|---|---|
| stcortex WebSocket (m2) | `ws://127.0.0.1:3000` | none — hard-coded constant. |
| ORAC LTP-density probe (m13) | `http://127.0.0.1:8133/blackboard/substrate_LTP_density` | constructor argument (`OracHttpReader::new`) only; no CLI flag. |
| SYNTHEX nexus push (m40) | `http://127.0.0.1:8092/v3/nexus/push` | constructor argument (`HttpNexusClient::new`) only; no CLI flag. |
| LCM JSON-RPC (m41) | `http://127.0.0.1:8082/rpc` | constructor argument (`HttpLcmClient::new`) only; no CLI flag. |
| HABITAT-CONDUCTOR (m32) | `http://127.0.0.1:8141` | `wf-dispatch --conductor-url`. |

---

## Feature flags

`Cargo.toml` declares `default = ["full"]` and `full = ["api", "intelligence", "monitoring", "evolution"]`. The four sub-features are **reserved** — they currently enable nothing; the 26 modules are too interdependent for clean partitioning. A normal `cargo build --release` builds the full crate.

---

> **Back to:** [`../README.md`](../README.md) · [`../QUICKSTART.md`](../QUICKSTART.md) · [`../ARCHITECTURE.md`](../ARCHITECTURE.md) · [`../CLAUDE.md`](../CLAUDE.md)
