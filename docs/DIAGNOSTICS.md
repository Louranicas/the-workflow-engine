# DIAGNOSTICS — workflow-trace troubleshooting + operator guide

> **Back to:** [`../README.md`](../README.md) · [`../CLAUDE.md`](../CLAUDE.md) · [`../CLAUDE.local.md`](../CLAUDE.local.md) · [`../GATE_STATE.md`](../GATE_STATE.md)
> **Sister docs:** [`../QUICKSTART.md`](../QUICKSTART.md) (5-minute start) · [`COMMAND_MAPPING.md`](COMMAND_MAPPING.md) (flag reference) · [`../ai_docs/API_MAP.md`](../ai_docs/API_MAP.md) (public-API surface) · [`../ai_docs/HARDENING_FLEET_2026-05-21.md`](../ai_docs/HARDENING_FLEET_2026-05-21.md) (hardening record)
> **Scope:** problem → cause → fix for operators running `wf-crystallise` / `wf-dispatch`, and developers running the quality gate. Every claim is verifiable against `src/` at git `ae7d460` (C22 binary wiring) and the Hardening Fleet record.

---

## 0. How to read this guide

Each entry is a **symptom → cause → fix** triple. Symptoms are exactly what you see on
stdout/stderr or in the `Report`. The two binaries **never panic on a down service** — a
service outage degrades into a logged-and-skipped stage, not a fault. So the first
diagnostic question is always: *is this a real fault, or graceful degradation?* §3 and §4
draw that line for each binary.

Quick triage:

| You see… | Go to |
|---|---|
| `cargo` errors during build/test | [§1 Build & quality gate](#1-the-4-stage-quality-gate) |
| `wf-crystallise` printed a report but `proposals written: 0` | [§3.1 Zero proposals](#31-zero-proposals-produced) |
| `wf-crystallise: pipeline fault:` on stderr | [§3.2 Crystallise faults](#32-crystallise-true-faults-exit-1) |
| `wf-crystallise` report shows `stages skipped: m2-stcortex, …` | [§3.3 Skipped stages](#33-skipped-live-service-stages-expected) |
| `wf-dispatch` shows `proposals loaded: 0` | [§4.1 Empty proposals JSONL](#41-empty-or-missing-proposals-jsonl) |
| `wf-dispatch` candidate disposition `verifier-blocked` | [§4.2 Verifier gate](#42-verifier-gate-blocking-a-candidate) |
| `wf-dispatch` candidate disposition `refused` under `--execute` | [§4.3 Conductor unreachable](#43-conductor-unreachable-under---execute) |
| `cargo:warning=POVM CR-2 … not verified` | [§6 m8 build warnings](#6-the-m8-povm-gate-buildrs-warnings-keep-dormant) |
| `error: failed to load source for dependency spacetimedb-sdk` | [§8 Build/env issues](#8-common-build--env-issues) |

---

## 1. The 4-stage quality gate

The mandatory gate (per [`../GOLD_STANDARDS.md`](../GOLD_STANDARDS.md)) is four stages, run
in order, **zero tolerance at every stage**:

```bash
CARGO_TARGET_DIR=./target cargo check --all-targets --all-features 2>&1 | tail -20 && \
cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tail -20 && \
cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic 2>&1 | tail -20 && \
CARGO_TARGET_DIR=./target cargo test --all-targets --all-features --release 2>&1 | tail -30
```

Stage-by-stage reading:

| Stage | Command | What a failure means | Where to fix |
|---|---|---|---|
| 1 — check | `cargo check` | Type/borrow/resolution error. Compilation is broken. | The `error[E…]` file:line. Fix before going further — later stages can't run. |
| 2 — clippy | `cargo clippy -- -D warnings` | A default-level lint fired and `-D warnings` promoted it to an error. | The lint name in brackets, e.g. `[clippy::needless_borrow]`. Fix the code; do **not** `#[allow]` it (god-tier rule 10). |
| 3 — pedantic | `cargo clippy -- -D warnings -W clippy::pedantic` | A pedantic lint fired. Common: `clippy::doc_markdown` (un-backticked identifier in a doc comment), `clippy::cast_possible_truncation`, `clippy::must_use_candidate`. | Same — fix, don't suppress. The two `#[allow(...)]` that survive in `src/` (e.g. the intentional 64→32-bit fold in `command_to_token`) carry a `reason = "…"` justification; new code is held to the same bar. |
| 4 — test | `cargo test --all-targets --all-features --release` | A `#[test]` assertion failed, or a test panicked. | The `---- … stdout ----` block names the failing test and assertion. Run just it: `cargo test <test_name> -- --exact --nocapture`. |

### 1.1 The `${PIPESTATUS[0]}` trap (god-tier rule 11)

**Symptom:** the gate prints a green-looking final line, but clippy was actually screaming.

**Cause:** `cargo clippy … | tail -20` makes `$?` capture **`tail`'s** exit code (always `0`),
not clippy's. A chained `&&` then sees `0` and proceeds. The summary looks clean while a real
error scrolled past.

**Fix:** never trust `$?` after a pipe. Use `${PIPESTATUS[0]}` and abort explicitly per
stage:

```bash
set +e
CARGO_TARGET_DIR=./target cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tail -20
[ "${PIPESTATUS[0]}" -ne 0 ] && { echo "GATE FAIL: clippy"; exit 1; }
```

Do **not** wrap the gate in `set -e` + a single pipe and assume it caught the failure. Test
both the pass and fail path of your gate script before relying on it. (This near-missed a bad
commit in S1001882 — `clippy::doc_markdown` on an un-backticked `tracing::error` slipped past
a `| tail` gate.)

### 1.2 "It was green yesterday" — toolchain drift

**Symptom:** the gate fails on code you did not touch, usually new clippy lints.

**Cause:** a `rustc` / `clippy` upgrade introduced lints that did not exist before. Pre-existing
project warnings do **not** excuse new code (god-tier rule 15), and an upgrade can surface
lints in old code too.

**Fix:** run the full 4-stage gate, fix every new lint, and confirm `rustup show` /
`rustc --version` matches what your gate script's `PATH` resolves. `rust-version = "1.75"` is
the declared MSRV in `Cargo.toml`; building on an older toolchain is unsupported.

### 1.3 Reading the test count

The reported figure (≈1900+ `#[test]` items across `src/` inline modules + `tests/`
integration files) is **higher** than a naive `grep -c '#\[test\]'` because `proptest` cases
(in `m11_integration.rs`) generate many internal cases per `#[test]`. There are **0**
`#[ignore]` tests in the current tree — the m8 live-POVM probe was previously `#[ignore]`d and
is now exercised against a mock. A test count regression between waves is itself a signal:
cross-check against [`../ai_docs/HARDENING_FLEET_2026-05-21.md`](../ai_docs/HARDENING_FLEET_2026-05-21.md)
§ Results (W0 baseline 1310 → W5 1903).

---

## 2. The two binaries — mental model

Both binaries are **thin wrappers**. `src/bin/wf_crystallise.rs` (76 lines) and
`src/bin/wf_dispatch.rs` (63 lines) only parse `std::env::args()`, hand off to a `run()`
function in `src/orchestration/`, print the `Report`, and set an exit code. **All pipeline
logic lives in the library** (`workflow_core::orchestration::crystallise` /
`::dispatch`) so it is integration-testable without spawning a subprocess
(`tests/wf_crystallise_integration.rs`, `tests/wf_dispatch_integration.rs`).

> *Note for readers of an older `CODEBASE_MAP.md`:* any text describing the binary `main()`
> as a "one-line stub" is stale archaeology — it was true before commit `ae7d460` (C22). The
> binaries are wired.

**Exit codes (identical scheme for both):**

| Code | Meaning |
|---|---|
| `0` | Pipeline ran to completion, or `--help` / `--version` was requested. |
| `1` | A pipeline **fault** (`OrchestrationError`) — a missing/unwritable file, an unrecoverable store error. A down *live service* is **not** this. |
| `2` | A CLI **argument** error (`ArgError`) — unknown flag, missing value, bad value, unexpected positional. |

The fault/degradation split is enforced in the type system: `OrchestrationError`
(in `crystallise.rs` and `dispatch.rs`) is `#[non_exhaustive]` and its variants name only
true faults. A down stcortex/ORAC/synthex/LCM/Conductor never constructs an
`OrchestrationError`.

---

## 3. `wf-crystallise` diagnostics

`wf-crystallise` is the observe → mine → propose pipeline (m1-m23 + m40-m42). It reads three
substrates, records a workflow run, computes lift evidence, mines patterns, composes
proposals, and writes them to a JSONL file.

### 3.1 Zero proposals produced

**Symptom:** the binary exits `0`, the report prints, but:

```
  patterns mined             : 0
  proposals written          : 0
```

This is **the most common "is it broken?" question** and is almost always **correct
behaviour, not a fault.** There are three independent reasons it happens; check them in
order.

#### Cause A — F2 evidence gate: not enough accumulated runs

The F2 sample-size floor is `MIN_SAMPLE_SIZE = 20` (`m14_lift`, re-exported as
`m23_proposer::PROPOSAL_F2_THRESHOLD`). `compose_proposals` **silently skips every variant**
whose `LiftSnapshot.n < 20` *or* whose `snapshot.lift` is `None` — and `wilson_ci_half`
itself returns `None` below 20 samples. This is a deliberate anti-property: the engine
**refuses to propose a workflow on thin evidence.** A fresh run on a fresh `--runs-db` sees
`n == 1`, which is honestly below the floor, so 0 proposals is the *correct* answer.

**Fix — accumulate runs across invocations.** The lift window is computed over the **open
runs** in the `--runs-db` *before* the current run is closed. Evidence therefore accumulates
across invocations **only if you reuse the same `--runs-db` file**:

```bash
# WRONG — fresh DB every run, n always small, never crosses the F2 floor
wf-crystallise --runs-db /tmp/throwaway-$RANDOM.db

# RIGHT — persistent DB, evidence accumulates run-over-run until n >= 20
wf-crystallise --runs-db ~/.local/share/habitat/workflow_runs.db
```

The default is `./workflow_runs.db` (`DEFAULT_RUNS_DB`), created if absent — but only useful
if you run from the **same working directory** every time so the relative path resolves to
the same file. For a real operator deployment, pass an absolute `--runs-db` path and keep it.
Inspect accumulated runs directly:

```bash
sqlite3 -header -column ~/.local/share/habitat/workflow_runs.db \
  "SELECT COUNT(*) AS total_runs FROM workflow_runs;"
```

You need ≥20 rows before m23 will emit anything.

> The report's `lift window runs` line tells you `n` for the current invocation. If it stays
> below 20, you have not accumulated enough history yet — keep running against the persistent
> DB.

#### Cause B — `min_support` set too high for the history

m20 PrefixSpan only emits a `Pattern` supported by at least `--min-support` sessions
(default `3`; floor `MIN_SUPPORT_FLOOR = 2`, enforced by `MinSupport::new`). If your atuin
history has few repeated command sequences, no pattern clears the support threshold and
`patterns mined : 0`.

**Fix:** lower it (but not below 2): `wf-crystallise --min-support 2`. If even `2` yields
nothing, the input genuinely has no repeated sequences — `0` is correct.

#### Cause C — empty / tiny atuin history

If `atuin rows ingested : 0` (or a very small number), there is nothing to mine. See §3.2
for the "DB not found" case; if the file *is* found but small, the history is simply sparse.

**Bottom line:** `0 proposals` with a `completed: yes` report is the engine being honest. It
is a fault **only** if you have a persistent `--runs-db` with ≥20 rows, a sensible
`--min-support`, and non-trivial atuin history — and *still* get 0. In that case capture the
report JSON (`--format json`) and the `--runs-db` and escalate.

### 3.2 `wf-crystallise` true faults (exit 1)

These print `wf-crystallise: pipeline fault: <detail>` on stderr and exit `1`. The
`OrchestrationError` variants:

| Fault | `Report` detail | Cause | Fix |
|---|---|---|---|
| `Atuin(...)` — *"atuin ingest failed"* | — | The atuin `history.db` could not be **opened**. | See "atuin DB not found" below. |
| `Injection(...)` — *"injection.db ingest failed"* | — | `~/.local/share/habitat/injection.db` could not be opened. | Confirm the path; pass `--injection-db <PATH>` if it lives elsewhere. The file is owned by the `memory-injection` service — workflow-trace only reads it. |
| `WorkflowRuns(...)` — *"workflow-runs store failed"* | — | The `--runs-db` SQLite could not be opened/created/written (bad path, no write permission, disk full, corrupt file). | Check the directory exists and is writable; check disk space; if the file is corrupt, move it aside and let a fresh one be created (you lose accumulated F2 evidence — see §3.1A). |
| `Miner(...)` — *"pattern miner failed"* | — | `MinSupport::new` rejected `--min-support` (it was `< 2`). | Pass `--min-support 2` or higher. |
| `Lift(...)` — *"lift aggregator config invalid"* | — | The internal `LiftAggregatorConfig` weights were non-finite — only reachable if the defaults are edited. | Restore `LiftAggregatorConfig::default()` (cascade 0.6 / cost 0.4). |
| `ProposalsOutput { path, source }` — *"proposals output write failed at …"* | — | The `--proposals-out` file could not be **created or written** (bad directory, no permission, disk full), or a proposal failed to serialise. | Confirm the parent directory exists and is writable; check disk space. |

#### "atuin DB not found"

**Symptom:** `wf-crystallise: pipeline fault: atuin ingest failed: …` mentioning the
history.db path.

**Cause:** m1 (`m1_atuin_consumer::open_readonly`) could not open
`$HOME/.local/share/atuin/history.db`. Either atuin is not installed, `$HOME` is unset (the
binary falls back to `/tmp`, which won't have the DB), or the file is at a non-default path.

**Fix:**

```bash
# 1. Confirm the file exists
ls -la ~/.local/share/atuin/history.db

# 2. If it's elsewhere, point the binary at it explicitly
wf-crystallise --atuin-db /custom/path/history.db

# 3. If $HOME is unset in your runtime (cron, systemd), set it
HOME=/home/louranicas wf-crystallise
```

m1 opens the DB **read-only** with `PRAGMA query_only = ON` — it never writes atuin's
history and cannot corrupt it. A `BUSY_TIMEOUT_MS = 5_000` covers atuin writing concurrently.

> An atuin DB that exists but is **locked beyond 5 s** surfaces as an `Atuin(...)` fault, not
> a hang. If atuin itself is mid-heavy-write, retry.

### 3.3 Skipped live-service stages (expected)

**Symptom:** the report's last data line reads e.g.:

```
  stages skipped             : m2-stcortex, m40-nexus-emit
```

…and the binary still exits `0` with `completed : yes`.

**Cause — this is graceful degradation, not a fault.** `wf-crystallise` has two live-service
stages, and each degrades independently:

| Stage | Service | Port | When it skips |
|---|---|---|---|
| `m2-stcortex` | stcortex (SpacetimeDB) | `ws://127.0.0.1:3000` | `--offline` set, **or** stcortex unreachable, **or** the m2 identity newtype rejected its inputs (cannot happen with the built-in literals). |
| `m40-nexus-emit` | synthex-v2 | `http://127.0.0.1:8092/v3/nexus/push` | `--offline` set, **or** synthex-v2 unreachable / returned a non-2xx / returned a 2xx body carrying an `"error"` field (AP-V7-13). |

A skipped stage is logged via `tracing` at `WARN` (`"m2 stcortex unreachable — skipping
(graceful degradation)"`) and recorded in `Report.stages_skipped`. **The mining/proposal core
(m1, m3, m4, m7, m14, m20-m23) does not depend on either live stage** — proposals are still
produced. stcortex registration (m2) is a *trust-signal* read; the nexus emit (m40) is an
*outbound notification*. Neither gates crystallisation.

**When it IS a problem:** if you *intended* substrate registration/notification to happen
and the services are *supposed* to be up, the skip tells you the service is actually down.
Confirm:

```bash
# stcortex — note: SpacetimeDB returns 404 on `/`, that's still "up"
~/.local/bin/stcortex status        # or: curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:3000
# synthex-v2
curl -s -o /dev/null -w '%{http_code}\n' http://127.0.0.1:8092/health
```

If you *do not* want live stages attempted at all (CI, sandboxed runs), pass `--offline` —
both stages are then skipped deliberately and the report says so. `--offline` is the honest,
deterministic way to run the pipeline with no network.

**Real-fault vs graceful-skip decision rule:**

- Report `completed: yes` + stage in `stages skipped` → **graceful degradation.** Exit `0`.
  Investigate the *service* only if it should have been up.
- Report `completed: no` / exit `1` + `wf-crystallise: pipeline fault:` on stderr → **real
  fault.** It is always a *file/store* problem (atuin, injection.db, runs-db, proposals-out),
  never a down HTTP service.

### 3.4 Reading the `wf-crystallise` `Report`

Text format (`--format text`, the default) prints one labelled line per field. JSON
(`--format json`) prints the same `Report` struct as a single line for machine consumption.

| Report field | Meaning | Healthy value |
|---|---|---|
| `atuin rows ingested` | m1 history rows read | > 0 (else see §3.2) |
| `injection chains read` | m3 unresolved `causal_chain` rows | ≥ 0 (0 is fine — may genuinely be no open chains) |
| `cascade clusters` | m4 correlated multi-pane clusters | ≥ 0 |
| `workflow run id` | the m7 run row id just recorded | > 0 |
| `observations merged` | cluster + chain observations patched onto the run | ≥ 0 |
| `lift window runs` | `n` for the m14 snapshot — **the F2 number** | ≥ 20 before m23 proposes (§3.1A) |
| `patterns mined` | m20 PrefixSpan patterns | ≥ 0 (0 → §3.1B/C) |
| `proposals written` | m23 proposals written to `--proposals-out` | ≥ 0 (0 → §3.1) |
| `stages skipped` | live stages skipped (`--offline` or service down) | `none`, or a known service list (§3.3) |
| `completed` | reached the end without aborting | `yes` |

A `completed: yes` report with all-zero counts is a *valid* run on an empty/young habitat —
not a bug.

---

## 4. `wf-dispatch` diagnostics

`wf-dispatch` is the select → verify → dispatch pipeline (m30-m33). It reads the proposals
JSONL produced by `wf-crystallise`, accepts each into an in-memory curated bank (m30), scores
and selects the top-K (m31), runs the 4-verifier gate (m33), and — **only under
`--execute`** — dispatches to HABITAT-CONDUCTOR (m32).

### 4.0 `--dry-run` vs `--execute`

| Mode | Flag | Behaviour | Contacts Conductor? |
|---|---|---|---|
| **Dry-run** (default-safe) | `--dry-run`, or no flag | Loads, accepts, selects, verifies. Each approved candidate's disposition is `dry-run`. | **No.** |
| **Execute** | `--execute` | Same, then for each *verifier-approved* candidate performs a real m32 dispatch to the Conductor. | **Yes.** |

`--dry-run` is the **default** — `Config::default().dry_run == true`. Last-flag-wins:
`--execute --dry-run` ends up in dry-run; `--dry-run --execute` ends up in execute. Use
dry-run freely; it is side-effect-free with respect to the Conductor. Use `--execute` only
when you intend a real dispatch *and* the Conductor is up *and* you have set an appropriate
`--ack-ceiling` (§4.3).

### 4.1 Empty or missing proposals JSONL

**Symptom (empty):** the report shows `proposals loaded : 0` and everything downstream is
`0`. Exit `0`.

**Cause:** the `--proposals-in` file (default `./proposals.jsonl`) exists but is empty or
contains only blank lines. Almost always: the upstream `wf-crystallise` run produced 0
proposals (§3.1). Blank lines are skipped, not errors.

**Fix:** this is correct — there is nothing to dispatch. Go fix the *upstream* side: re-run
`wf-crystallise` against a persistent `--runs-db` with ≥20 accumulated runs (§3.1A) and a
sensible `--min-support`, then point `wf-dispatch --proposals-in` at the *same path*
`wf-crystallise --proposals-out` wrote.

**Symptom (missing):** `wf-dispatch: pipeline fault: proposals input read failed at …` and
exit `1`.

**Cause:** the `--proposals-in` file does not exist or is unreadable. The two binaries hand
off **only** through this JSONL file — there is no shared process or database. If
`wf-crystallise` wrote `./proposals.jsonl` from directory A and you run `wf-dispatch` from
directory B, the relative default resolves to a different (missing) file.

**Fix:** pass matching absolute paths to both binaries:

```bash
OUT=~/.local/share/habitat/proposals.jsonl
wf-crystallise --proposals-out "$OUT" --runs-db ~/.local/share/habitat/workflow_runs.db
wf-dispatch    --proposals-in  "$OUT"
```

**Symptom (malformed):** `wf-dispatch: pipeline fault: proposals input parse failed at <path>
line <N>: <detail>` and exit `1`.

**Cause:** line `N` of the JSONL is not a valid serialised `WorkflowProposal`. This means the
file was hand-edited, truncated mid-write, or produced by an incompatible build of
`wf-crystallise`.

**Fix:** regenerate the file from a current `wf-crystallise` build. Do not hand-edit the
JSONL — each line is one compact `serde_json` object and the schema is the `WorkflowProposal`
struct.

### 4.2 Verifier gate blocking a candidate

**Symptom:** a candidate line in the report reads:

```
    workflow            12345678901234567890 : verifier-blocked
```

…and `verifier approved` is lower than `candidates selected`.

**Cause:** the m33 4-verifier gate did **not** return `AllApprove`. m33 requires **all four**
verifiers (`Security`, `Consistency`, `Cost`, `Ember` — `VerifierKind::VARIANTS`) to
`Approve`; any `Refuse` or `Amend` makes `aggregate` return `AggregateVerdict::Blocked`. A
blocked candidate is **never dispatched**, even under `--execute`.

**Important nuance — the CLI's verifiers are conservative placeholders.** `wf-dispatch`'s
`build_verifiers()` wires four `ConservativeVerifier`s that each return
`VerifierVerdict::Approve` unconditionally. This is **documented in the source**
(`ConservativeVerifier` doc comment): the m33 gate is structurally present, deterministic,
and exercised end-to-end, but its verdict is a *placeholder* — not a real security/cost
audit. Real per-kind logic needs policy inputs the binary does not currently receive.

**Therefore:**

- With the **current CLI build**, `verifier-blocked` should essentially never appear — the
  conservative verifiers always approve. If you see it, something structural is wrong (the
  m33 gate did not get exactly one verifier per kind — see the
  `build_verifiers_yields_one_per_kind` test).
- If real verifier logic has been swapped in, `verifier-blocked` means that real audit
  refused. The disposition string alone does not name *which* verifier — to see the blocking
  `VerifierKind`s, raise `tracing` verbosity (§7) and look for the m33 aggregation span, or
  read `RefusalReason::VerifierGateBlocked { blocking_kinds }` from m32 (the blocking kinds
  are ordinal-sorted).

### 4.3 Conductor unreachable under `--execute`

**Symptom:** under `--execute`, a candidate's disposition is `refused`:

```
    workflow            12345678901234567890 : refused
```

…and `dispatched : 0` despite candidates being `verifier approved`.

**Cause — graceful degradation, not a fault.** m32's `dispatch` folds an unreachable
HABITAT-CONDUCTOR into `DispatchOutcome::Refused`, which `wf-dispatch` reports as `refused`.
The Conductor at `http://127.0.0.1:8141` is down, returned a non-2xx, or returned a 2xx body
carrying an `"error"` field (AP-V7-13 — a 2xx with an `error` field is **not** success). The
binary logs `"m32 dispatch refused (graceful degradation)"` at `WARN` and continues; exit is
still `0`.

**Other reasons m32 can refuse** (`RefusalReason`, surfaced in `tracing` at `WARN`):

| `RefusalReason` | Meaning | Fix |
|---|---|---|
| `ConductorUnreachable` / `WireFormat { detail }` | Conductor down, non-2xx, or unparseable/`error`-bearing body. | Bring the Conductor up (below). |
| `EscapeSurfaceNotAcknowledged` | The workflow's escape-surface profile exceeds the `--ack-ceiling` the operator acknowledged. | Raise `--ack-ceiling` to a profile that covers the workflow's destructiveness — *only if you genuinely accept that surface*. See the 7-variant ladder below. |
| `RoutingMethodMismatch { expected, actual }` | The Conductor client's dispatch method is not the canonical `lcm.loop.create`. | Indicates a wiring regression; `CONDUCTOR_DISPATCH_METHOD` must stay `"lcm.loop.create"` (never `lcm.deploy`). |
| `SelfDispatchRefused` | AP-V7-08 — the workflow's `proposal_id` matched the m32 self-dispatch sentinel. | Expected refusal of a workflow that would dispatch the engine itself. Not a fault. |
| `WorkflowNotBanked` | The workflow id is not in the curated bank. | Internal — should not happen via the CLI path. |

**Fix — bring the Conductor up.** Per project rule, **do not start services from an agent** —
Luke runs `devenv` from the terminal:

```bash
~/.local/bin/devenv -c ~/.config/devenv/devenv.toml start --services weaver,zen,enforcer
curl -s -o /dev/null -w 'weaver=%{http_code}\n' http://127.0.0.1:8141/health
```

The Conductor (weaver) Waves 1B/1C/2/3 ship `auto_start=false` (blocker B3 in
[`../GATE_STATE.md`](../GATE_STATE.md)) — so a fresh habitat has it **down by design** until
explicitly started. `wf-dispatch --execute` against a down Conductor is therefore a *normal*
state, correctly handled as `refused`.

**`--ack-ceiling` — the escape-surface ladder.** `EscapeSurfaceProfile` is a 7-variant
monotone ordinal; the operator must acknowledge a ceiling **≥** the workflow's surface:

| `--ack-ceiling` value | Profile | Ordinal |
|---|---|---|
| `sandboxed` (default) | `Sandboxed` | 0 |
| `sandbox-escape` | `SandboxEscape` | 10 |
| `process-mutate` | `ProcessMutate` | 20 |
| `privilege-escalation` | `PrivilegeEscalation` | 30 |
| `file-write` | `FileWrite` | 40 |
| `network-egress` | `NetworkEgress` | 50 |
| `data-exfil` | `DataExfil` | 60 |

The default `sandboxed` is deliberately the least-destructive — a workflow needing a higher
surface is **refused** until you raise the ceiling. The flag *is* the human acknowledgement;
raise it only when you genuinely accept that destructiveness class.

### 4.4 `wf-dispatch` true faults (exit 1)

Printed as `wf-dispatch: pipeline fault: <detail>`. `OrchestrationError` variants:

| Fault | Cause | Fix |
|---|---|---|
| `ProposalsInput { path, source }` | `--proposals-in` missing/unreadable. | §4.1 (missing). |
| `ProposalsParse { path, line, detail }` | A JSONL line is not a valid `WorkflowProposal`. | §4.1 (malformed) — regenerate. |
| `Bank(...)` | The curated bank rejected a proposal **unrecoverably**. (A *single* rejected proposal — e.g. the AP-V7-08 self-dispatch sentinel — is logged and skipped, not a fault; this variant is for an unrecoverable bank fault.) | Capture the report + proposals file; escalate. |
| `Selector(...)` | m31 was given an invalid `SelectorConfig` (weights not summing to 1.0 / non-finite). | Only reachable if `SelectorConfig::default()` was edited — restore α 0.4 / β 0.25 / γ 0.2 / δ 0.15. |

A down Conductor is **never** an `OrchestrationError` — it is a `refused` candidate (§4.3).

### 4.5 Reading the `wf-dispatch` `Report`

| Report field | Meaning | Notes |
|---|---|---|
| `mode` | `dry-run` or `execute` | Confirms which mode actually ran (last-flag-wins). |
| `proposals loaded` | lines parsed from `--proposals-in` | 0 → §4.1 |
| `bank accepted` | proposals accepted into m30 | May be `< proposals loaded` if m30 skipped some (logged at `WARN`). |
| `candidates selected` | top-K chosen by m31 | ≤ `--top-k` (default 5) and ≤ `bank accepted`. |
| `verifier approved` | candidates the m33 gate approved | With conservative verifiers, equals `candidates selected`. |
| `dispatched` | candidates a real Conductor accepted | **Always 0 in dry-run.** In execute, < `verifier approved` if the Conductor refused some. |
| per-candidate lines | `workflow <id> : <disposition>` | disposition ∈ `dry-run` / `dispatched` / `refused` / `verifier-blocked`. |
| `completed` | reached the end without aborting | `yes` |

---

## 5. External-service matrix

From [CODEBASE_MAP §6]. Every HTTP/WebSocket dependency, what needs it, and what happens when
it is down. **The binaries degrade gracefully on every live-service outage** — a down service
is a skipped/refused stage, never a panic or a non-zero exit.

| Service | Port / URL | Modules | Used for | Binary | When DOWN |
|---|---|---|---|---|---|
| **atuin** (SQLite file) | `$HOME/.local/share/atuin/history.db` | m1 | Shell-history ingest — the raw observation input. | `wf-crystallise` | **Hard fault** (`Atuin(...)`, exit 1). Not a "service" — a required file. §3.2. |
| **injection.db** (SQLite file) | `~/.local/share/habitat/injection.db` | m3 | Habitat `causal_chain` ledger ingest. | `wf-crystallise` | **Hard fault** (`Injection(...)`, exit 1). Required file. §3.2. |
| **workflow_runs.db** (SQLite file) | `--runs-db`, default `./workflow_runs.db` | m7 | Run record + F2 evidence store. **Created if absent.** | `wf-crystallise` | **Hard fault** if unwritable (`WorkflowRuns(...)`, exit 1). |
| **stcortex** (SpacetimeDB) | `ws://127.0.0.1:3000` | m2, m13, m42 | m2: trust-signal consumer registration. m13/m42: substrate-feedback pathway writes. | `wf-crystallise` | **Graceful skip.** m2 → `stages skipped: m2-stcortex`. m13 → `PromoteOutcome::Deferred` (writes to JSONL outbox instead). §3.3. SpacetimeDB returns **404 on `/`** — that is still "up"; use `stcortex status`, not an HTTP-200 probe. |
| **ORAC** | `http://127.0.0.1:8133/blackboard/substrate_LTP_density` | m13 | LTP-density probe driving the m13 3-band write/defer gate. | `wf-crystallise` | **Graceful.** ORAC unreachable → m13 **defers** the write to the JSONL outbox (`DeferReason::OracUnreachable`). No data lost; the outbox is the durable record. |
| **synthex-v2** | `http://127.0.0.1:8092/v3/nexus/push` | m40 | Outbound `NexusEvent` notification (`workflow.completed`). | `wf-crystallise` | **Graceful skip.** → `stages skipped: m40-nexus-emit`. Pure notification — does not gate anything. §3.3. |
| **HABITAT-CONDUCTOR** (weaver) | `http://127.0.0.1:8141/dispatch` | m32 | The **only** sanctioned dispatch path — m32 never spawns a process/shell/pane directly. | `wf-dispatch --execute` | **Graceful.** → candidate disposition `refused`, `RefusalReason::ConductorUnreachable`. Exit still 0. §4.3. Down by design until `weaver` started (B3). |
| **LCM** | `http://127.0.0.1:8082/rpc` | m41 | `lcm.loop.create` JSON-RPC — the CC-5 F-iteration re-trigger. | (m41 library; not on the current CLI happy-path) | **Graceful** (`LcmRpcError::Transport`) when invoked. |
| **POVM** | `http://127.0.0.1:8125/learning_health` | m8 | CR-2 trust probe — **KEEP-DORMANT**, not on any pipeline path. | neither (m8 runtime probe unused) | N/A — see §6. |

**The LTP 3-band gate (m13)** decides what happens to a substrate write based on the ORAC
`substrate_LTP_density` reading:

| LTP density | m13 outcome |
|---|---|
| `< 0.015` (`LTP_PHASE_1_FLOOR`) | `Deferred` — written to the JSONL outbox, not stcortex. |
| `[0.015, 0.10)` | `WrittenUnderPressure` — written to stcortex with `under_pressure = true`. |
| `>= 0.10` (`LTP_PHASE_3_TARGET`) | `Written` — written normally. |
| ORAC unreachable | `Deferred` (`OracUnreachable`) — outbox. |
| stcortex unreachable | `Deferred` (`StcortexUnreachable`) — outbox. |

So even with the whole substrate down, m13 loses nothing — the JSONL outbox at the
configured `outbox_path` is the durable fallback (atomic tmp+rename write). A growing outbox
file is the signal that substrate writes are being deferred.

---

## 6. The m8 POVM-gate `build.rs` warnings (KEEP-DORMANT)

**Symptom:** every `cargo build` / `cargo check` prints three warnings:

```
warning: POVM CR-2 (magnitude-weighted learning_health) not verified.
warning: Set POVM_CR2_DEPLOYED=1 after confirming povm-v2 commit e2a8ed3 is live.
warning: See: ~/projects/claude_code/Hebbian Deployment Plan v3 — Post-CR-2 Threshold Reconciliation.md
```

**This is intentional. It is NOT an error and NOT a gate failure.** It does not fail
`cargo check`, clippy, or the quality gate.

**Cause:** `build.rs` emits `cargo:rustc-cfg=povm_calibrated` **only** when the environment
variable `POVM_CR2_DEPLOYED=1` is set. When it is absent, `build.rs` prints these three
`cargo:warning=` lines instead. The warnings are the m8 trust regime announcing that the POVM
CR-2 (magnitude-weighted `learning_health`) fix has not been *verified live* in this build
environment.

**Why `povm_calibrated` is a `rustc-cfg`, not a Cargo feature:** so that
`cargo --features full`, `--all-features`, or any feature combination **cannot** activate it
(F7 / AP-V7-09 defense). Only the explicit env var flips it. `tests/m8_integration.rs`
asserts this — `--all-features` does not enable `povm_calibrated`.

**Why it stays dormant (S1003733 F2 decision — KEEP-DORMANT):** the m8 gate's job is to fail
the build (via `compile_error!` tombstones at `#[cfg(not(povm_calibrated))]` POVM-read sites)
if any code reads POVM `learning_health` before CR-2 is verified. Per the m42 stcortex-only
ADR, workflow-trace routes all substrate-feedback through stcortex — **there are no in-tree
POVM-read sites** for the gate to guard. m8 is therefore retained as a **dormant tripwire**:
it costs nothing, fires only if future code adds a POVM read, and the three warnings are its
"I am armed and watching" signal. See the `src/m8_povm_build_prereq/mod.rs` module doc and
[`../ai_docs/HARDENING_FLEET_2026-05-21.md`](../ai_docs/HARDENING_FLEET_2026-05-21.md) (F2
resolution).

**What to do:** nothing. Do **not** set `POVM_CR2_DEPLOYED=1` to silence the warnings unless
you have actually confirmed povm-v2 commit `e2a8ed3` (the CR-2 magnitude-weighted formula) is
live on `:8125`. Silencing a trust gate without the underlying condition being true is
exactly the regression the gate exists to prevent. Treat the three warnings as expected
build noise.

> Distinguish from the **vendored `spacetimedb-sdk` deprecation noise** — that is unrelated
> third-party-crate churn from `src/m2_stcortex_consumer/module_bindings/`, also harmless and
> clippy-suppressed at the module boundary. Neither set of warnings fails the gate.

---

## 7. Logs — where they go, how to get more

Both binaries emit structured **`tracing`** events throughout the pipeline (`tracing::info!`
on each stage completion, `tracing::warn!` on every graceful degradation). Targets are
`wf_crystallise` and `wf_dispatch`.

**The default is silence.** Per `src/bin/wf_crystallise.rs`, **no `tracing` subscriber is
installed** by the binaries — `tracing-subscriber` is a *dev-dependency only*. Without a
subscriber, every `tracing` event is silently dropped. This is the correct default for a CLI
whose canonical, intentional output is the printed `Report`.

**Consequence:** out of the box you see *only* the `Report` on stdout and `ArgError` /
`OrchestrationError` messages on stderr. The per-stage `tracing` events
(`"m2 stcortex unreachable — skipping"`, `"m32 dispatch refused"`, the m33 aggregation
detail, etc.) are **not visible**.

**To raise verbosity** you need a subscriber. Options:

1. **Run the integration tests with output** — the test harness can install a subscriber and
   `--nocapture` shows the events:
   ```bash
   cargo test --test wf_crystallise_integration -- --nocapture
   cargo test --test wf_dispatch_integration -- --nocapture
   ```
2. **Add a subscriber to the binary** (the supported extension point). In `main()`, before
   `run(&config)`, install one and honour `RUST_LOG`:
   ```rust
   tracing_subscriber::fmt()
       .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
       .init();
   ```
   Then `tracing-subscriber` must move from `[dev-dependencies]` to `[dependencies]`, and:
   ```bash
   RUST_LOG=wf_crystallise=debug,wf_dispatch=debug wf-crystallise
   ```
   `EnvFilter` syntax: `RUST_LOG=warn` (warnings only), `RUST_LOG=wf_dispatch=trace` (one
   target, max detail), `RUST_LOG=info` (all targets, stage-completion lines).

Until a subscriber is wired, the `Report` itself is your primary diagnostic surface — it is
deliberately rich enough (skipped stages, per-candidate dispositions, counts) to diagnose the
common cases in §3-§4 *without* logs. Use `--format json` to capture a `Report` verbatim for
an escalation.

---

## 8. Common build & env issues

### 8.1 `CARGO_TARGET_DIR` — keep it consistent

The gate scripts and the project convention use `CARGO_TARGET_DIR=./target`. If you build
once *with* it and once *without*, you get **two separate target directories** (`./target`
and the cargo default), doubling disk use and disk-cache misses — and a "stale binary" trap
where you rebuilt one tree but ran the binary from the other.

**Fix:** pick one and be consistent. The project convention is `CARGO_TARGET_DIR=./target`
(matches [`../GOLD_STANDARDS.md`](../GOLD_STANDARDS.md) gate). Export it for the session:

```bash
export CARGO_TARGET_DIR=./target
```

Release binaries land at `./target/release/wf-crystallise` and `./target/release/wf-dispatch`.

### 8.2 The `spacetimedb-sdk` relative-path dependency

**Symptom:** `error: failed to load source for dependency 'spacetimedb-sdk'` /
`failed to read … /spacetimedb/sdks/rust/Cargo.toml`.

**Cause:** `Cargo.toml` declares `spacetimedb-sdk = { path = "../spacetimedb/sdks/rust" }`.
This is a **relative** path (deliberately — T4-PORT / Zen assessment improvement #4: a
relative path resolves on any host without a hard-coded `/home/louranicas/...` prefix). It
resolves as a **sibling directory of `the-workflow-engine/`** under
`claude-code-workspace/`. The build fails if:

- the `spacetimedb/` sibling repo is not checked out next to `the-workflow-engine/`, **or**
- you copied/moved `the-workflow-engine/` out of `claude-code-workspace/` so the sibling no
  longer exists at `../spacetimedb/sdks/rust`.

**Fix:** ensure the directory layout is intact:

```
claude-code-workspace/
├── the-workflow-engine/      ← you are here
└── spacetimedb/
    └── sdks/rust/Cargo.toml  ← the path dep resolves here
```

```bash
ls ../spacetimedb/sdks/rust/Cargo.toml   # must exist
```

If `spacetimedb/` is genuinely absent, check it out as a sibling. Do **not** "fix" this by
hard-coding an absolute path in `Cargo.toml` — that re-introduces the host-specific breakage
the relative path was chosen to avoid. The m2 SpacetimeDB *bindings* themselves are vendored
in-tree (`src/m2_stcortex_consumer/module_bindings/`); only the SDK crate is the path dep.

### 8.3 `cargo audit` — malformed advisory DB

**Symptom:** `cargo audit` errors out parsing the local advisory database (e.g. a malformed
`RUSTSEC-2026-0109.md`).

**Cause:** a corrupt entry in the locally-cached RustSec advisory DB — an environment
problem, not a workflow-trace problem.

**Fix:** refresh the advisory DB (`cargo audit fetch`, or remove `~/.cargo/advisory-db` and
let it re-clone). The Hardening Fleet W2 ran `cargo audit` clean after this env fix; the
crate itself has no flagged advisories.

### 8.4 Feature flags do nothing — by design

**Symptom:** building with `--features intelligence` (or `api` / `monitoring` / `evolution`)
changes nothing.

**Cause:** `default = ["full"]` and `full = ["api", "intelligence", "monitoring",
"evolution"]`, but **all four sub-features are RESERVED and gate no code** — the 26 modules
form an interdependent graph that does not partition cleanly, so module-level
`#[cfg(feature = …)]` gating is deferred pending an architecture decision. `cfg!(feature =
"full")` is also used by `tests/m8_integration.rs` as a fixture to confirm `--all-features`
does **not** activate `povm_calibrated`.

**Fix:** none needed — this is expected. Build with defaults. `substrate-load` is the one
*functional* opt-in (off by default; enables substrate-side benchmarks — keep it off in
production to avoid substrate co-tenant traffic).

### 8.5 Pipeline runs but writes nothing useful in CI / cron

**Symptom:** in an automated environment the binary runs but every count is 0.

**Causes & fixes:**

- **`$HOME` unset** → atuin/injection.db paths resolve wrong. Set `HOME` explicitly (§3.2).
- **Fresh `--runs-db` each run** → F2 never satisfied. Use a persistent absolute path (§3.1A).
- **Relative `proposals.jsonl`** → crystallise and dispatch resolve different files. Use
  matching absolute paths (§4.1).
- **Network-isolated runner** → pass `--offline` to `wf-crystallise` so live stages are
  *deterministically* skipped rather than each incurring a connect-timeout.

---

## 9. Escalation checklist

Before escalating a suspected bug, confirm it is not one of the expected behaviours above.
Attach:

1. The exact command line (all flags).
2. The full `Report` — run with `--format json` and capture stdout verbatim.
3. stderr verbatim (the `wf-…: pipeline fault:` / `wf-…:` line, if any) and the exit code
   (`echo $?`).
4. For `0 proposals`: `sqlite3 <runs-db> "SELECT COUNT(*) FROM workflow_runs;"` and the
   `--min-support` used (§3.1).
5. For a live-service issue: the health probe output for the relevant service from the §5
   matrix.
6. Toolchain: `rustc --version`, `cargo --version`, and whether `CARGO_TARGET_DIR` was set.
7. Git revision: `git rev-parse HEAD` (the binaries were wired at `ae7d460` — C22).

A `completed: yes` report with zero counts, or a `stages skipped` entry, or the three m8
`build.rs` warnings, are **expected** — none of those alone is a bug.

---

> **Back to:** [`../README.md`](../README.md) · [`../CLAUDE.md`](../CLAUDE.md) · [`../GATE_STATE.md`](../GATE_STATE.md) · [`../ai_docs/INDEX.md`](../ai_docs/INDEX.md) · sister [`COMMAND_MAPPING.md`](COMMAND_MAPPING.md) · [`../QUICKSTART.md`](../QUICKSTART.md)
