---
title: The Workflow Engine — End-to-End Stack Deployment Plan (/scaffold × atuin × DevOps V3 × CodeSynthor V8)
date: 2026-05-17 (S1001982)
authority: Luke @ node 0.A — "develop a detailed and comprehensive plan for how /scaffold atuin dev ops engine v3 and the code synthor v8 can work in collaboration and synergy for a full end to end stack deployment of the-workflow-engine"
emitter: Command (Tab 1 Orchestrator top-left)
kind: PLANNING-ARTEFACT (end-to-end orchestration plan — no code, no scaffold, no execution)
status: planning-only · HOLD-v2 active (G1-G9 gated; none green) · this plan ACTIVATES at G9 "start coding" signal, not before
priors:
  - the-workflow-engine/THE_WORKFLOW_ENGINE_GENESIS_PROMPT_V1_S1001982.md (v1.2, Zen-audit-locked invariant)
  - the-workflow-engine/THE_WORKFLOW_ENGINE_TOWN_HALL_S1001982.md (15 P0 constraints, vote 11/1/0)
  - the-workflow-engine/THE_WORKFLOW_ENGINE_BOILERPLATE_HUNT_S1001982.md (9-fleet, 63 candidates)
  - the-workflow-engine/THE_WORKFLOW_ENGINE_MODULE_STRUCTURE_S1001982.md (26 modules, single-phase override)
  - the-workflow-engine/the-workflow-engine-vault/boilerplate modules/Gold Standard Exemplars — Synthesis.md
  - the-workflow-engine/the-workflow-engine-vault/boilerplate modules/Maintenance Engine V2 — Gold Standard Reference.md
  - the-workflow-engine/the-workflow-engine-vault/boilerplate modules/Habitat Loop Engine — Gold Standard Reference.md
  - the-workflow-engine/the-workflow-engine-vault/boilerplate modules/ORAC Sidecar — Gold Standard Reference.md
sources:
  - dev-ops-engine-v3 @ port 8082 — 8L/40M, NAM-03 confidence, T1-T6 tier executor
  - code-synthor-v8 @ port 8111 — Rust+TS+Elixir Holy Trinity, 848 tests, /v8:scaffold + /v8:deploy
  - scaffold-mastery skill — plan.toml-driven Rust scaffold generation (3 install paths)
  - atuin scripts — 20+ habitat scripts (habitat-intel, habitat-sweep, fzf-service, habitat-autopilot, etc.)
back_to: CLAUDE.md · CLAUDE.local.md · the-workflow-engine/
---

# The Workflow Engine — End-to-End Stack Deployment Plan

> Back to: [[the-workflow-engine-vault/HOME]] · [[the-workflow-engine-vault/MASTER_INDEX]] · [[CLAUDE.md]] · [[CLAUDE.local.md]]
>
> Vault mirror: drop a thin pointer in `the-workflow-engine-vault/` after Luke acks the path interpretation (see §0).

---

## §0 — Path note (flag → redirect if needed)

Luke wrote: *"save your plan as a .md file here `/home/louranicas/claude-code-workspace/the-workflow-engine/the-workflow-engine`"*.

The literal path has no `.md` extension and the trailing segment duplicates the project root name. I interpreted this as **save at the project root with a descriptive filename** matching the existing `THE_WORKFLOW_ENGINE_*_S1001982.md` convention. If Luke meant a different location (e.g. the vault subfolder, or literally `the-workflow-engine/the-workflow-engine.md`), redirect and I'll move it in one operation.

---

## §1 — Frame, scope, and what this plan IS / IS NOT

### What this plan IS

A **deployment recipe** describing how four habitat surfaces collaborate to take `the-workflow-engine` from **planning-pilot (today)** to **running service registered in devenv** when the build freeze lifts. The four surfaces:

| Surface | Role in the pipeline | Activation gate |
|---|---|---|
| **`/scaffold` skill** (scaffold-mastery) | plan.toml → directory tree + module stubs + tests + Cargo workspace | G9 fires; G2 (rename) green |
| **atuin scripts** | history-driven monitoring, fleet probes, habit-of-the-machine telemetry across every step | continuous — already operational |
| **DevOps Engine V3** (`:8082`) | end-to-end orchestrator: 6-tier executor (T1→T6), NAM-03 confidence gating, quality gate, devenv registration, episodic learning, cancel handler | continuous — already running |
| **CodeSynthor V8** (`:8111`) | spec-driven code emission, pattern lookup, NAM agent routing, AST refactor, bug hunt with STDP saturation auto-stop | continuous — already running |

This plan tells the future scaffold-orchestrator session **which surface to call when, what they hand off, how their outputs compose, and where the gates and refusals sit**.

### What this plan IS NOT

- **Not a build trigger.** HOLD-v2 envelope holds. No `cargo init`. No `mkdir workflow-trace/src/`. No new substrate writes for the workflow-trace spec.
- **Not a Phase B/C plan.** Watcher's R6 (substrate-frame engine) and Phase B (develop/iterate verbs) are out of scope here. This plan operates strictly within Phase A measure-only invariant (verbs: **ingest, record, correlate, consume, guard, refuse**).
- **Not a v1.3 spec patch.** Genesis Prompt v1.2 remains binding. Anything below that contradicts v1.2 is wrong and must be flagged.
- **Not a unilateral commit.** G7 Zen audit + G1 Watcher close-notice still gate any persistence of this plan beyond local file.

### The four-tier collaboration thesis

```
   Luke directive (G9 "start coding")
              │
              ▼
   ┌──────────────────────────────┐
   │  DevOps V3 (:8082)           │  ← orchestrator: owns the T1..T6 trajectory,
   │  POST /deploy {goal}         │     confidence gate, episodic memory, cancel
   └──────────────────────────────┘
              │
       ┌──────┴──────┬──────────────┬───────────────┐
       ▼             ▼              ▼               ▼
   /scaffold     CodeSynthor V8   atuin          gold-standard
   (T1-T2)       (T2-T4)          (continuous)   exemplars (boilerplate
   plan.toml →   spec → modules + reads + writes  modules/ — read-only)
   tree skeleton tests + patterns  history; emits
                 + NAM routing     telemetry to RM
       │             │              │
       └──────┬──────┴──────────────┘
              ▼
       ┌──────────────────────────────┐
       │  DevOps V3 T4-T6             │  ← gate(4-stage clippy/test) →
       │  + m29 devenv_bridge         │     register service → start →
       │  + m28 habitat_bridge        │     health probe → soak → promote
       └──────────────────────────────┘
              │
              ▼
        Service live :PORT
        on devenv batch N
```

V3 is the **conductor**. V8 is the **generative organ**. `/scaffold` is the **bone scaffold** (deterministic file-tree synthesis). atuin is the **proprioception** (sense organ for "what is the habitat currently doing").

---

## §2 — Surface deep-profile (what each brings to the table)

### §2.1 — `/scaffold` skill (scaffold-mastery)

**Location:** `~/.claude/skills/scaffold-mastery/` (also installed at `pane-vortex-v2/.claude/skills/`, `orac-sidecar/.claude/skills/`)

**Trigger phrases:** scaffold, generate scaffold, new project, create microservice, init codebase

**Inputs:**
- A `plan.toml` (preferred — driven by spec) OR
- Default fallback: 8-layer/41-module Rust scaffold (the ME v2 / ORAC pattern)

**Capabilities:**
| Capability | Detail |
|---|---|
| Plan-driven scaffold | Parses `plan.toml` → emits Cargo workspace OR single-crate with feature-gated layers |
| 8-layer default | `src/m1_foundation/` … `src/m8_evolution/` matching ORAC/ME convention |
| Custom layers | Override layer count, layer names, layer→module mapping |
| Custom modules | Per-module: name, layer, deps, est-LOC, est-test-count, feature flag |
| Test kinds | `unit` (in-module `mod tests`) + `integration` (`tests/`) + `doctests` |
| Feature gates | Per-module `[features]` map in Cargo.toml |
| Consent config | Optional `consent.toml` for boundary modules |
| Per-module deps | Crate-level deps declared per module, resolved into workspace deps |

**Output:**
- Directory tree (`Cargo.toml`, `src/lib.rs`, `src/m{N}_*/mod.rs`, `tests/`, `migrations/`, `.bacon-locations`, `bacon.toml`)
- `CLAUDE.md` skeleton + `MASTER_INDEX.md` skeleton + vault `HOME.md` skeleton
- No tests yet implemented — empty `#[cfg(test)] mod tests {}` blocks per module
- A "scaffold receipt" — JSONL of what was generated, hashed for verifier downstream

**What `/scaffold` does NOT do:**
- Does not write business logic. Stubs only.
- Does not run `cargo check`. That's V3's T4.
- Does not register with devenv. That's V3's T6.
- Does not commit. Caller commits.

### §2.2 — atuin scripts (the proprioception)

**Location:** `~/.local/share/atuin/scripts.db` (SQLite-backed)

**Inventory (relevant to deployment):**
| Script | Role in this pipeline |
|---|---|
| `habitat-intel` | 17ms pulse (r, gen, fitness, LTP/LTD, thermal) — call before/after every V3 tier |
| `habitat-sweep` | Parallel 11-service health sweep (~200ms) — call after V3 T6 to verify no neighbor regression |
| `habitat-bootstrap` | 7-layer L0-L6 session injection — call on every fresh session that resumes the pipeline |
| `habitat-loop` | Probe ORAC + PV2 baseline — call between V3 tiers for cheap field-coherence check |
| `habitat-metabolic` | ME × ORAC × PV2 composite — call after T6 to confirm metabolic health |
| `habitat-bridge-check` | 7-service bridge health — call before T6 to ensure registration target is healthy |
| `habitat-fingerprint` | Response-time + key-count fingerprint for all 11 services — baseline before/after |
| `habitat-autopilot` | Autonomous 5-tool health loop — runs as background soak harness post-T6 |
| `habitat-evolution-delta` | RALPH evolution delta with snapshot persistence — call across T1↔T6 to capture deployment-induced evolution shift |
| `fzf-service`, `fzf-endpoint-matcher` | Interactive service selection during cancel/inspect |
| `habitat-density` | Service-integration density from shell history — post-deploy, verify new service is being called |

**Capacity:** every shell command run during the deployment is **automatically recorded** in atuin (`~/.local/share/atuin/history.db`). This is the **single most important property** — atuin is the only surface that captures the actual command trajectory across all four tools. Post-deploy, `atuin search --workspace the-workflow-engine` gives a perfect provenance trail without any new instrumentation.

**What atuin does NOT do:**
- Does not generate code.
- Does not register services.
- Does not enforce gates (it observes, it does not refuse).

### §2.3 — DevOps Engine V3 (`:8082`) — the orchestrator

**Source:** `/home/louranicas/claude-code-workspace/dev-ops-engine-v3` (already profiled at length in `boilerplate modules/`-adjacent material; this section is the deployment-specific extract)

**Tier executor (T1→T6):**

| Tier | Theme | Confidence gate | What happens | V3 modules used |
|---|---|:---:|---|---|
| **T1** | Specify | none (start) | Parse NL goal → `StructuredSpec` → `Plan` (layers, modules, deps, gaps, dissent) | m07 (parser), m08 (plan gen), m10 (gap), m11 (dissent) |
| **T2** | Scaffold | C ≥ 0.80 | 6-facet parallel emit: code-stubs, doc-stubs, meta-tree, vault-skeleton, test-harnesses, config | m12 (dispatch), m13 (code), m14 (vault), m15 (meta), m16 (tests/config) |
| **T3** | Implement | C ≥ 0.80 | Fleet dispatch (7 implement panes) + critic review | m33 (fleet dispatcher), m34 (critic) |
| **T4** | Harden | C ≥ 0.85 | 4-stage gate: `cargo check` → `clippy -D warnings` → `clippy::pedantic` → `cargo test --lib --release` | m19 (quality gate), m21 (SOR tracker, target ≥18.0) |
| **T5** | Document | C ≥ 0.85 | Generate Obsidian vault, MASTER_INDEX, CLAUDE.md sections, 12D episodic signature | m14, m15, m23 (episodic memory) |
| **T6** | Deploy | C ≥ 0.90 | Register in `devenv-container.toml`, run `devenv up`, fire `/hooks/deployment_started`, probe health | m29 (devenv_bridge), m28 (habitat_bridge), m37 (HTTP handler) |

**Critical endpoints:**
- `POST /deploy {"goal": "..."}` — kick off workflow, returns `{workflow_id, status, tier}` (or 409 if MAX_ACTIVE_WORKFLOWS=1 is full)
- `GET /deploy/{id}` — poll progress
- `POST /deploy/{id}/cancel` — abort (resolved in S1001883 Wave-3; handler exists at `m37_http_server.rs:737`)
- `GET /confidence/{id}` — NAM-03 breakdown (primary/historical/health/thermal/coherence)
- `GET /self-model` — 30s introspection loop
- `POST /hooks/{event}` — ORAC pushes (`tier_completed`, `deployment_started`, `field_update`, `evolution_tick`)
- `GET /bridge-signals` — latest cached PV2 r / SYNTHEX thermal / RALPH gen+fitness+phase

**Persistence:** 3 SQLite DBs — `workflow_state.db`, `deployment_history.db` (12D episodes), `hebbian_pulse.db` (16 seeded pathways, LTP/LTD).

**What V3 does NOT do directly:**
- Does not run `cargo build`. Delegates to caller (or to `/forge` skill).
- Does not own the binary cp. m29 writes devenv config; the binary symlink is post-T6 caller responsibility.
- Does not generate Rust code. That's `/scaffold` + V8.

### §2.4 — CodeSynthor V8 (`:8111`) — the generative organ

**Source:** `/home/louranicas/claude-code-workspace/code-synthor-v8` (Holy Trinity: Rust backbone + TypeScript MCP bridge + Elixir OTP supervision)

**Generative commands (plugin-slash):**

| Command | Purpose | Output |
|---|---|---|
| `/v8:scaffold <goal>` | NL goal → `plan.toml` | TOML with modules (heuristic: √features ≈ module_count), layer assignment, dep DAG, owner-agent per module |
| `/v8:deploy <plan.toml>` | Full pipeline: parse → NAM route → scaffold → gate emit | Ready-for-V3-T4 artefact tree |
| `/v8:architect` | Architecture analysis on existing tree | Quality scores per module, anti-pattern detections |
| `/v8:pattern <lang> <concept>` | Pattern DB lookup (28 gold standards) | Top-K patterns by 11D tensor similarity |
| `/v8:hunt <target>` | Bug hunt loop with STDP saturation auto-stop | Hunt log, fixes proposed, weight delta |
| `/v8:gate <lang>` | Multi-stage quality gate | Same 4-stage Rust gate as V3 T4 (parity) |
| `/v8:refactor <target>` | Treesitter-AST refactor (rename/extract/restructure) | Diff per file, gate-validated |
| `/v8:trinity-gen` | Cross-language scaffold (Rust + TS + Elixir parallel) | Three-language tree (not needed here — workflow-engine is Rust-only) |

**Capabilities matrix:**
| Capacity | Layer | Backed by |
|---|---|---|
| 11D tensor similarity, FMA, cosine | Rust | `tensor.rs` (54K) |
| 28 gold-standard pattern DB, fuzzy search | Rust | `patterns.rs` (51K) |
| STDP LTP/LTD/decay, saturation detection | Rust | `hebbian.rs` (60K) |
| AST-aware refactor via treesitter | Rust | `ast_refactor.rs` (42K) |
| Cross-session adaptive memory | Rust | `memory_store.rs` (46K) |
| MCP wrapping, Zod-validated inputs | TypeScript | `mcp-server/src/` |
| OTP supervision, retry, GenServer state | Elixir | `application.ex` + 6 GenServers |
| Bandit HTTP `:8111` + health endpoints | Elixir | `health_plug.ex` |

**Critical insight for collaboration:** V8 and V3 **already speak to each other** through `POST /api/v8/confidence` and `POST /api/v8/learning` on V3's side. The bidirectional Hebbian feedback loop exists at the wire level. This plan exploits that.

---

## §3 — Synergy matrix (who does what, when, with which output handed to whom)

### §3.1 — The composition principle

Each surface has **one thing it does best**. Avoid overlap. The synergy comes from **clean handoffs at well-defined output boundaries**:

```
SCAFFOLD-MASTERY          CODESYNTHOR V8         DEVOPS V3              ATUIN
─────────────────         ──────────────         ───────────            ─────
deterministic file        spec→code emission     orchestration +        proprioception
tree synthesis            + pattern-aware        gating + lifecycle     + provenance
                          generation             + persistence
        │                       │                       │                  │
        │  plan.toml            │  modules + tests      │  workflow_id    │
        │  (input)              │  (artefacts)          │  (state)         │  (history)
        ▼                       ▼                       ▼                  ▼
     ──────────────────────────────────────────────────────────────────────────
                              SHARED OUTPUTS
       1. Cargo workspace tree (scaffold owns; V8 fills)
       2. Module skeletons with #[cfg(test)] (scaffold owns; V8 fills tests)
       3. plan.toml (V8 generates from NL; scaffold consumes)
       4. Gate results (V3 owns; V8 verifies; atuin records)
       5. Devenv registration (V3 owns; atuin records the command)
```

### §3.2 — The 12-step end-to-end pipeline (G9-armed)

When Luke fires G9 "start coding" (and G1-G8 are green), the following sequence runs. Each step names the **invoker**, the **callee**, the **input**, the **output**, and where the **gate** sits.

#### Step 1 — Pre-flight: habitat baseline

**Invoker:** orchestrator session (Claude on Tab 1)
**Callee:** atuin
**Command:** `atuin scripts run habitat-bootstrap && atuin scripts run habitat-intel && atuin scripts run habitat-fingerprint`
**Output:** TSV baseline of all 11 services' health + RALPH state + field r + thermal + per-service response-time fingerprint, recorded in atuin history.
**Gate:** None (baseline only). If any service is DOWN, refuse to proceed (`workflow-engine` must not deploy onto a wounded habitat).
**Verbs honored:** *consume, guard*.

#### Step 2 — Spec authoring (V8 generative organ)

**Invoker:** orchestrator
**Callee:** CodeSynthor V8 — `/v8:scaffold "ingest atuin history + stcortex consumers + injection.db chains + nvim diagnostics; correlate by opaque_id; record into workflow_trace_* namespace; emit human-readable reports; refuse all dispatch/control verbs"`
**V8 internals:** parses goal → builds 11D task tensor → NAM-routes architect agent → generates `plan.toml` (heuristic: √features → ~26 modules — aligns with current single-phase 26-module plan from `THE_WORKFLOW_ENGINE_MODULE_STRUCTURE_S1001982.md`)
**Output:** `plan.toml` written to `the-workflow-engine/plan.toml` (NEW FILE — first persistence-side write under G9 envelope).
**Gate:** **G7 Zen audit** of the generated plan.toml. Zen rejects if any forbidden verb appears (recommend, rewrite, route, package, dispatch, optimise). Watcher receives a one-line cross-talk notice (per G2.E). On reject, loop back to Step 2 with verb-list reinforcement.
**Verbs honored:** *record*.

#### Step 3 — Scaffold materialisation

**Invoker:** orchestrator (post-Zen-pass)
**Callee:** `/scaffold` skill with `plan.toml`
**Action:** scaffold-mastery walks plan.toml → emits:
- `Cargo.toml` (single crate, feature-gated per layer — ORAC pattern; rationale per Synthesis doc §"Divergent patterns")
- `src/lib.rs` + `src/m{1..7}_<theme>/mod.rs` (26 modules across 7 layers — Phase A only)
- `tests/` directory with empty harness files
- `migrations/0001_init.sql` (single `workflow_trace_events` table, ORAC blackboard shape)
- `bacon.toml` (LCM on_success chain: check → clippy → pedantic → test)
- `CLAUDE.md` skeleton (charter — copy ME v2 structure)
- `CLAUDE.local.md` skeleton (session-state delta)
- `MASTER_INDEX.md` (LCM partition style)
- `the-workflow-engine-vault/HOME.md` updated with code-side anchor

**Output:** receipt-JSONL of every file created, SHA-anchored (LCM substrate-emit pattern).
**Gate:** **G2 (naming)** must already be green for the directory rename to `workflow-trace/` to have taken effect — `/scaffold` MUST target the renamed path. If G2 not green, abort.
**Verbs honored:** *record* (passive emission, no execution).

#### Step 4 — Plan ratification + pre-DevOps-V3 handshake

**Invoker:** orchestrator
**Callee:** DevOps V3 — `POST /deploy {"goal": "scaffold workflow-trace from plan.toml at <path>", "resume_from": "T2", "plan_toml_hash": "<sha>"}`
**Note:** V3 starts at **T2**, not T1, because T1 (specify) has already been done by V8 + Zen. V3's m07 parser reads `plan.toml` directly (V3 supports TOML plans, see m07_goal_parser).
**Output:** `workflow_id` UUID. V3 writes initial row to `workflow_state.db`. atuin records the curl. Cancel handle armed (`POST /deploy/{id}/cancel` ready).
**Gate:** V3 enforces **MAX_ACTIVE_WORKFLOWS=1**. If another workflow is in-flight, refuse. Confidence gate at T2 entry: C ≥ 0.80 (initial calc uses Step 1 baseline as `C_health`).
**Verbs honored:** *consume, guard*.

#### Step 5 — V3 T2 Scaffold facet (cross-check with `/scaffold` output)

**Invoker:** V3 (now in T2)
**Callee:** V3 internal modules m12-m16 (6-facet dispatcher) — runs IN PARALLEL with V8 quality cross-check
**Action:** m12 dispatches:
- m13 (code): reads `/scaffold` output, verifies file-count and per-module stub presence
- m14 (vault): cross-checks vault scaffold (HOME + MASTER_INDEX + module pages)
- m15 (meta): checks tree integrity (Cargo.toml deps DAG = plan.toml dep DAG)
- m16 (tests): counts empty test stubs (must be ≥ plan.toml estimated_test_count × 0.8)
- + `/v8:architect` is invoked sideways for an early structural audit
**Output:** T2 receipt, confidence calc, episodic 12D snapshot.
**Gate:** **G3 Watcher close-notice required**. V3 fires `POST /hooks/tier_completed` to ORAC, ORAC pushes to Watcher cross-talk, Watcher must ack scaffold receipt. C ≥ 0.80 OR retry T2 once (V3 m18 adaptive threshold; after 1 retry, escalate to human).
**Verbs honored:** *correlate* (between two scaffold sources), *record*.

#### Step 6 — Implementation (V3 T3 + V8 per-module generation)

**Invoker:** V3
**Callee:** V3 m33 (fleet dispatcher) — but for Phase A this is **single-pane only**, NOT 7-pane fleet (Phase A is measure-only; no parallel-development verb-creep)
**Process per module:**
1. V3 m33 picks next module from plan.toml DAG (topological order)
2. V3 calls `/v8:pattern rust <module_theme>` → V8 returns top-3 gold-standard patterns
3. V3 calls `/v8:scaffold add <module_name> <description>` (mid-stream module gen) → V8 emits module body + 50+ test scaffolds
4. V8 internally NAM-routes to executor agent (resonance score on 11D task↔agent tensor)
5. V3 fires `POST /api/v8/learning` with outcome → V8 Hebbian engine LTP/LTD on pathway
6. atuin records every shell action (cargo edits, file diffs)

**Output:** per-module Rust source + tests, all under Phase-A verbs only.
**Gate:** After EACH module: `/v8:gate rust` runs the 4-stage gate. If a module fails, V8 `/v8:hunt <module>` runs bug-hunt loop with STDP saturation auto-stop (typically 3-5 rounds). Confidence C calculated after each module. If C drops below 0.80, **CANCEL** the workflow (`POST /deploy/{id}/cancel`) and emit Watcher cross-talk notice.
**Verbs honored:** *ingest, record*. **REFUSED:** any verb that emits dispatch or control toward another habitat service.

#### Step 7 — Harden (V3 T4 + V8 critic agent)

**Invoker:** V3
**Callee:** V3 m19 (quality gate) — full 4-stage on the WHOLE tree
**Parallel callee:** `/v8:architect` for final structural audit + `/v8:critic` for diff review
**Atuin record:** `habitat-intel` before T4 + after T4 (capture LTP/LTD delta caused by gate-pass).
**Gate:** C ≥ 0.85. SOR ≥ 18.0 (E33 formula: artifacts/sec × quality_factor). If SOR < 18.0, log warning but DO NOT block (Phase A is bootstrapping; first deployment has no SOR baseline).
**Verbs honored:** *guard*.

#### Step 8 — Document (V3 T5)

**Invoker:** V3
**Callee:** V3 m14-m16 + V8 doc-emission
**Action:**
- Generate canonical `THE_WORKFLOW_ENGINE_*.md` planning artefact updates (mirror to vault)
- Generate vault module pages (one per module, bidirectional `Back to: [[HOME]]` anchor)
- Generate `Bugs & Known Issues.md` skeleton (empty — to be populated on first bug)
- Generate `Diagnostics.md` (atuin commands + V3 endpoint reference)
- Generate `MASTER_INDEX.md` updates
- Persist 12D episodic signature to V3 `deployment_history.db`
**Gate:** C ≥ 0.85. **Four-surface persistence** verified (ai_docs canonical + vault mirror + stcortex namespace `workflow_trace_*` + CLAUDE.local.md anchor).
**Verbs honored:** *record*.

#### Step 9 — Deploy (V3 T6 + atuin verification chain)

**Invoker:** V3
**Callee:**
- V3 m29 (devenv_bridge): writes service entry to `~/.config/devenv/devenv.toml` and `devenv-container.toml`. Adds to **batch 4** (depends on B3: orac-sidecar).
- Binary cp: `/usr/bin/cp -f target/release/workflow-trace ~/.local/bin/` (note: **never bare `cp -f` — alias trap from habitat memory `feedback_binary_deployment.md`**)
- V3 m28: fires `POST http://localhost:8133/hooks/deployment_started` (ORAC notification)
- V3 backgrounds health-probe loop
- atuin `habitat-bridge-check` — verify ORAC, PV2, SYNTHEX, POVM, RM all healthy before service start

**ACTUAL service start:** **NEVER from Claude session** — sandbox reaps children (`feedback_terminal_service_start.md`). Luke runs from terminal: `~/.local/bin/devenv -c ~/.config/devenv/devenv.toml start workflow-trace` OR full `devenv start`.
**Gate:** C ≥ 0.90.
**Verbs honored:** *consume, guard, refuse* (refuses to start the service from agent context).

#### Step 10 — Verify (atuin + V3 background)

**Invoker:** orchestrator
**Callee:** atuin chain — `habitat-sweep` then `habitat-loop` then `habitat-bridge-check`
**Action:** Confirm new service responds on its assigned port + health endpoint. Probe for unexpected side-effects on neighbor services (regression detection).
**Output:** TSV soak record in `/tmp/workflow-trace-postdeploy-<timestamp>.tsv`.
**Gate:** all 11 services + new service = 12 healthy. If <12, emit cross-talk notice and human-escalate (do NOT auto-rollback in Phase A — refuse-write discipline; manual rollback only).
**Verbs honored:** *correlate, guard*.

#### Step 11 — Soak (atuin habitat-autopilot, 24h)

**Invoker:** orchestrator (out-of-band; can fire from terminal)
**Callee:** `atuin scripts run habitat-autopilot` (5-tool autonomous loop) OR the LCM-style soak harness if available
**Action:** Sustained 24h probe at 30s intervals, capturing fitness drift, thermal trend, breaker-state, response-time. Use the LCM soak pattern from `boilerplate modules/Habitat Loop Engine — Gold Standard Reference.md` § "Test Discipline" → P3-ignored + soak.
**Output:** Soak verdict — PASS/FAIL. PASS = no regression in any habitat metric over 24h.
**Gate:** This is the **final** gate. PASS → service promoted to "live"; CLAUDE.md updated; vault `STATUS: LIVE` set.
**Verbs honored:** *ingest, record, correlate*.

#### Step 12 — Promote + memorise

**Invoker:** orchestrator
**Callee:**
- V3: episodic-memory commit (m23/m24) — 12D signature persisted to `deployment_history.db` with outcome
- V8: Hebbian LTP on used pathways via `POST :8082/api/v8/learning`
- atuin: log final completion
- Watcher: cross-talk notice to `watcher-notices/<timestamp>_workflow_trace_promoted.md`
- Vault: bidirectional anchor pair updated; CLAUDE.local.md slim-file row added for the workflow-engine
- stcortex: write pathway `workflow_trace_genesis_complete` (consumer-freshness must be < 60s — known stcortex bug; if stale, refuse write and emit cross-talk)

**Output:** the workflow-engine is now a **first-class habitat citizen**.
**Gate:** None (terminal).
**Verbs honored:** *record*.

---

## §4 — Gate integration map (G1-G9 × pipeline steps)

| Gate | Owner | What it requires | Pipeline step it unblocks |
|---|---|---|---|
| **G1** | Watcher | close-notice on planning artefacts | Step 0 (this plan is read-ready) |
| **G2** | naming | directory rename to `workflow-trace/` | Step 3 (`/scaffold` target) |
| **G3** | Watcher | scaffold receipt ack | Step 5 (T2 → T3) |
| **G4** | Zen | spec audit pass on plan.toml | Step 2→3 boundary |
| **G5** | interview | 12-question / 3-round structured interview complete | Step 2 (V8 needs interview-derived constraints) |
| **G6** | Ember | 7-trait unanimity rubric (vault-canonical) | Step 9 (T6 deploy) |
| **G7** | Zen | post-implementation audit (per F2: n≥20 + CI/error bars per report type) | Step 11→12 boundary |
| **G8** | substrate | stcortex consumer-freshness < 60s confirmed | Step 12 (final stcortex write) |
| **G9** | Luke | "start coding" signal | Step 2 (kicks off V8 spec authoring) |

**Key invariant:** **G9 is NOT step 0. G9 fires AFTER G1-G8 are green.** This plan describes what happens once G9 lands.

---

## §5 — Verb discipline (Phase A invariant)

Every step above is annotated with which verbs it honors. Cross-checked against the v1.2 invariant (Phase A: ingest, record, correlate, consume, guard, refuse):

| Step | Verbs touched | Forbidden verbs (any?) |
|---|---|---|
| 1 baseline | consume, guard | none |
| 2 spec | record | none (V8 generates a plan, not a recommendation — plan.toml is data) |
| 3 scaffold | record | none (passive emission only) |
| 4 V3 handshake | consume, guard | none |
| 5 V3 T2 | correlate, record | none |
| 6 V3 T3 | ingest, record | **REFUSED:** any agent-to-agent dispatch verb |
| 7 V3 T4 | guard | none |
| 8 V3 T5 | record | none |
| 9 V3 T6 | consume, guard, refuse | **REFUSED:** service start from agent context |
| 10 verify | correlate, guard | none |
| 11 soak | ingest, record, correlate | none |
| 12 promote | record | none |

**No "recommend / rewrite / route / package / dispatch / optimise" verbs surface in any step.** V3's tier-executor naming (m33 "fleet dispatcher") is internal to V3 itself, not exposed as a workflow-engine capability. V8's NAM router is internal to V8.

---

## §6 — Failure modes + escape hatches

| Failure mode | Detection | Escape hatch |
|---|---|---|
| V8 generates plan.toml with forbidden verb | G7 Zen audit at Step 2 | Loop back to Step 2 with verb-list reinforcement; cross-talk notice to Watcher |
| `/scaffold` output disagrees with plan.toml (file count, dep DAG) | V3 T2 m15 (meta-tree builder) | Abort at T2, regenerate from scratch (idempotent step) |
| V3 T4 gate fails after V8 hunt-loop saturation | V3 m19 + V8 STDP saturation flag | Cancel workflow (`POST /deploy/{id}/cancel`), file Bug entry, human-escalate |
| `MAX_ACTIVE_WORKFLOWS=1` blocks Step 4 | V3 returns 409 | Inspect via `GET /status`, cancel the in-flight workflow if it's stale, retry |
| Habitat regression detected at Step 10 | atuin `habitat-sweep` non-200 on neighbor | Refuse promotion. Human-rollback (no auto-rollback in Phase A). |
| stcortex consumer-freshness > 60s at Step 12 | `stcortex consumers` query | Skip stcortex write, file cross-talk, retry on next session |
| Service start ANY agent attempt | hookify pre-tool-use hook | Hook refuses; remind operator: only Luke runs `devenv start` |
| Cancel handler 404 | shouldn't happen (LCM Drift #11 RETRACTED — handler verified at `m37_http_server.rs:737`) | If it does, V3 deploy is broken; abort everything; file P0 bug |
| Drift #N: agent claims gate-clean falsely | `/gate` re-run by orchestrator + git log verification | LCM drift-register discipline: orchestrator re-runs `--workspace --all-targets --all-features`; agent report is evidence, not fact |

---

## §7 — Verification matrix (what to check, when)

| Checkpoint | Verifier | Verification command |
|---|---|---|
| Pre-flight habitat baseline | atuin | `atuin scripts run habitat-fingerprint` |
| plan.toml verb-clean | Zen | manual audit + grep for forbidden verb list |
| Scaffold tree integrity | scaffold-receipt + V3 m15 | hash check vs plan.toml DAG |
| Per-module gate | V3 m19 + V8 /v8:gate | `cargo check && clippy -D warnings && clippy -W clippy::pedantic && cargo test --lib --release` (PIPESTATUS-aware) |
| Confidence trajectory | V3 m17 + GET /confidence/{id} | non-decreasing across T2→T6 |
| Hebbian feedback wired | V3 hebbian_pulse.db | `sqlite3 hebbian_pulse.db "SELECT COUNT(*) FROM pulse_events WHERE workflow_id=?"` non-zero after T6 |
| Four-surface persistence | filesystem + stcortex + vault + CLAUDE.local.md | each surface independently readable |
| Habitat regression-free | atuin habitat-sweep | 11/11 services 200 OK before & after |
| Soak PASS | atuin habitat-autopilot | TSV verdict line |
| stcortex write fresh | stcortex inspect | freshness < 60s |
| Vault back-links | grep | `> Back to:` line exists on every new vault note |
| atuin provenance trail | atuin search | `atuin search --workspace the-workflow-engine` returns every command from Steps 1-12 |

---

## §8 — What the orchestrator session must know (cold-start crib)

A fresh Claude session resuming this pipeline at any point should be able to read **this document + `THE_WORKFLOW_ENGINE_GENESIS_PROMPT_V1_S1001982.md` + the boilerplate modules synthesis doc** and pick up. Mandatory minimum reads:

1. This plan (§§1-7)
2. Genesis Prompt v1.2 (binding spec invariants)
3. `boilerplate modules/Gold Standard Exemplars — Synthesis.md` (architectural template)
4. `boilerplate modules/Maintenance Engine V2 — Gold Standard Reference.md` (8-layer mN_ convention)
5. `boilerplate modules/ORAC Sidecar — Gold Standard Reference.md` (single-crate feature-gated pattern)
6. `boilerplate modules/Habitat Loop Engine — Gold Standard Reference.md` (`bacon.toml` + `local-ci/` + soak)
7. CLAUDE.md (habitat charter)
8. CLAUDE.local.md (current session state, drift register)
9. `atuin scripts list` (capability inventory)
10. `curl -s :8082/status` + `curl -s :8082/self-model` (V3 live state)
11. `curl -s :8111/status` (V8 live state)

---

## §9 — Why this composition (versus alternatives)

| Alternative | Why rejected |
|---|---|
| V3 alone (no V8) | V3 doesn't generate Rust source; it orchestrates. Need V8 for module bodies + tests + pattern lookup. |
| V8 alone (no V3) | V8 doesn't own confidence gating, episodic memory, devenv registration, or workflow-state SQLite. V3 owns the trajectory. |
| `/scaffold` alone | Deterministic file tree only; no spec parsing, no code bodies, no quality gate, no devenv. |
| atuin alone | Observation surface; no generation, no gating. |
| V3 + `/scaffold` (no V8) | Loses pattern-aware code synthesis, NAM agent routing, AST refactor, hunt-loop saturation. |
| V8 + `/scaffold` (no V3) | Loses workflow state machine, confidence gating, cancel handler, devenv bridge, episodic memory. |
| V3 + V8 (no `/scaffold`) | V8's emit is template-driven but not necessarily aligned with the 8-layer mN_ convention; `/scaffold` enforces the convention deterministically. |
| Everything except atuin | No provenance trail. Atuin is the only surface that captures the cross-tool command trajectory. |

**Composition is minimal.** Drop any one of the four and a property is lost.

---

## §10 — Open questions (pre-G9)

These need answering before the plan can fire:

1. **Crate organisation:** single-crate feature-gated (ORAC) OR workspace-of-crates (LCM)? Per Synthesis doc divergent-axis table, recommendation is **single-crate feature-gated** at v0 since 26 modules don't yet need independent release cadence.
2. **Persistence model:** single SQLite `workflow_trace.db` with multi-table OR several DBs (one per concern, ME v2 pattern)? Recommendation: **single DB**, ORAC blackboard shape; revisit at sunset if concurrent-write contention surfaces.
3. **Port assignment:** workflow-trace needs a port. Available range: 8084-8089 (not in ULTRAPLATE active table). Recommendation: **8084** (next contiguous after 8083 Nerve Center).
4. **Devenv batch placement:** B4 (depends on ORAC B3) is the natural slot. Confirm B4 has capacity (current B4: codesynthor-v8 + nerve-center + habitat-telegram + prometheus-swarm-v2 + synthex-v2-shadow = 5 services).
5. **Stcortex namespace:** `workflow_trace_*` per convention. Need Watcher + Zen sign-off on the namespace shape before first write.
6. **Path interpretation for THIS plan file:** Luke clarifies whether save location is correct (§0).

---

## §11 — Hand-off snippets (copy-paste-ready when G9 fires)

These are **not** active commands. They are illustrative — to be reviewed and adapted by the future orchestrator after G1-G9 clear:

```bash
# Step 1 — baseline
atuin scripts run habitat-bootstrap
atuin scripts run habitat-intel
atuin scripts run habitat-fingerprint

# Step 2 — V8 generates plan.toml
# (invoke /v8:scaffold via Claude Code slash command in V8 plugin context)

# Step 3 — /scaffold materialises tree
# (invoke /scaffold skill with plan.toml as arg)

# Step 4 — V3 handshake
WORKFLOW_ID=$(curl -s -X POST localhost:8082/deploy \
  -H 'Content-Type: application/json' \
  -d '{"goal":"materialise workflow-trace from plan.toml at <path>","resume_from":"T2","plan_toml_hash":"<sha>"}' \
  | jq -r .workflow_id)

# Steps 5-8 — V3 drives, V8 generates per-module on demand
# (orchestrator polls GET /deploy/$WORKFLOW_ID, reacts to ORAC hook events)

# Step 9 — register
# V3 m29 writes devenv config; Luke runs from terminal:
#   ~/.local/bin/devenv -c ~/.config/devenv/devenv.toml start workflow-trace

# Step 10 — verify
atuin scripts run habitat-sweep
atuin scripts run habitat-bridge-check

# Step 11 — soak (24h)
atuin scripts run habitat-autopilot &

# Step 12 — promote
curl -s -X POST localhost:8082/api/v8/learning \
  -H 'Content-Type: application/json' \
  -d '{"workflow_id":"'$WORKFLOW_ID'","outcome":"success","pathways":["workflow_trace_genesis"]}'
```

---

## §12 — Update history

| Date | Action |
|---|---|
| 2026-05-17 (S1001982) | v1 plan authored. Four-surface composition mapped. 12-step pipeline + gate × verb matrix. Synthesis with `boilerplate modules/` deep references. |

---

## §13 — See also

**Planning artefacts (canonical pairs in `the-workflow-engine/`):**
- [[THE_WORKFLOW_ENGINE_GENESIS_PROMPT_V1_S1001982]] — binding spec invariant
- [[THE_WORKFLOW_ENGINE_TOWN_HALL_S1001982]] — 15 P0 constraints
- [[THE_WORKFLOW_ENGINE_BOILERPLATE_HUNT_S1001982]] — 9-fleet candidates
- [[THE_WORKFLOW_ENGINE_MODULE_STRUCTURE_S1001982]] — 26-module single-phase architecture
- [[THE_WORKFLOW_ENGINE_CIRCLE_OF_EXPERTS_S1001982]] — 8-persona disputation
- [[CONVERGENCE_COMMAND_X_COMMAND3_S1001982]] — peer convergence

**Gold-standard references (`the-workflow-engine-vault/boilerplate modules/`):**
- [[Gold Standard Exemplars — Synthesis]] — cross-cutting patterns + scaffolding checklist
- [[Maintenance Engine V2 — Gold Standard Reference]] — 8-layer 40-module exemplar
- [[Habitat Loop Engine — Gold Standard Reference]] — 10-crate workspace + plan.toml exemplar
- [[ORAC Sidecar — Gold Standard Reference]] — 8-layer sidecar + wire-protocol exemplar
- [[BOILERPLATE_INDEX]] — 48-file per-file lift map

**Habitat substrate:**
- `CLAUDE.md` § Service Map (port 8082 V3, 8111 V8)
- `CLAUDE.local.md` § Open Escalations (V3 bind fix `b7d4abb` shipped; LCM drift #11 RETRACTED)
- `~/projects/claude_code/ULTRAPLATE Master Index.md` § Service Map + Memory Systems

**Cross-talk channels:**
- `~/projects/shared-context/agent-cross-talk/` (peer comms)
- `~/projects/shared-context/watcher-notices/` (Watcher inbox)

---

*Luke @ node 0.A | Command @ Tab 1 | The Watcher @ observing | Zen @ Tab 10 audit lane | 2026-05-17 (S1001982)*
