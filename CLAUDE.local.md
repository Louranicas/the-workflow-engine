# the-workflow-engine — Local Session State (delta)

> **Back to:** [CLAUDE.md](CLAUDE.md) — project charter (structural facts; do not duplicate here)
> **Session checkpoint:** [`~/projects/shared-context/sessions/2026-05-17T124122_workflow-engine-ultimate-framework.md`](file:///home/louranicas/projects/shared-context/sessions/2026-05-17T124122_workflow-engine-ultimate-framework.md)
> **🟢 S1002209 ACTIVE — Luke task-cascade 1-6:** see "S1002209 — Luke Task-Cascade 1-6 Execution" section below for live status.
> **Vault home:** [the-workflow-engine-vault/HOME.md](the-workflow-engine-vault/HOME.md)
> **God-tier synthesis:** [the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md](the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md)
> **Deployment recipe:** [the-workflow-engine-vault/ULTIMATE_DEPLOYMENT_FRAMEWORK_S1001982.md](the-workflow-engine-vault/ULTIMATE_DEPLOYMENT_FRAMEWORK_S1001982.md)
> **Workflow tracker:** [the-workflow-engine-vault/workflow-engine-code-base.md](the-workflow-engine-vault/workflow-engine-code-base.md) — 15 phases / 13 decisions / 13 open issues
>
> **Synergy with [CLAUDE.md](CLAUDE.md):** the charter holds the rules (how to behave, which protocols govern the fleet). This file holds the state those rules operate over — live workstream, pending decisions, persisted surfaces, cold-start pointers. **Charter answers _how_; this file answers _what the world looks like right now_.** Do NOT duplicate charter content here.

---

## 🔧 HARDENING FLEET — 2026-05-21 (S1003529) · COMPLETE

End-to-end quality + security hardening of the 26-module codebase, directed by Luke @ node
0.A in collaboration with Zen (audit lane). 6 waves — all committed on `main`, pushed both
remotes. **Tests 1310 → 1903; clippy + pedantic clean every wave.**

- **W1** `dc25335` — quality floor: every module to 50+ meaningful tests (1310 → 1782).
- **W2** `c662b2d` + `5cb4822` — security: 19 findings (KEYSTONE `project_after_prefix`
  correctness bug, 9 lock-poison panics, LIKE-injection, m9 namespace boundary, m8
  false-gate docstrings, HTTP body caps) (→ 1834).
- **W3** `2e3113d` — type-design: `#[non_exhaustive]` ×24, `WorkflowId` + `MinSupport`
  encapsulation, comment accuracy (→ 1835).
- **W4** `5de71ac` — mutation testing: `cargo-mutants` scoped to m10/m11/m21/m22.
  Committed baseline run = 319 mutants (240 caught / 20 missed / 20 timeout / 39 unviable;
  85.7% caught of 280 viable). 68 mutant-killing tests authored targeting the 20 missed
  mutants; the 20 m21 `build_variants` timeout mutants remain unscored. (→ 1903.)
- **W5** `e8f6dd3` — docs reconciliation + 4-surface persistence + push.

Gate every wave: `cargo check` + `clippy -D warnings` + `clippy -D clippy::pedantic` +
`cargo test --all-targets --all-features --release`. **Resolved S1003733** (assessment-driven
remediation): F2 m8 POVM-gate → **KEEP-DORMANT** (see `m8_povm_build_prereq` module doc §);
W3 #5–#10 core-type-encapsulation portfolio → **completed** (remediation Wave C — 6
representable-illegal-state holes closed across Pattern/WorkflowProposal/AcceptedWorkflow/
NexusEvent/WorkflowRunRow/ID-newtypes).

### 🔵 COLD-START — RESUME HERE

The Hardening Fleet is persisted across every memory substrate, each bidirectionally anchored
to this block (every surface embeds this file's path as its reverse-anchor):

| Surface | Anchor |
|---------|--------|
| git | 6 commits `dc25335..e8f6dd3` on `main` — pushed origin (GitHub) + gitlab |
| ai_docs (canonical) | [`ai_docs/HARDENING_FLEET_2026-05-21.md`](ai_docs/HARDENING_FLEET_2026-05-21.md) · [`ai_docs/HARDENING_W2_FINDINGS.md`](ai_docs/HARDENING_W2_FINDINGS.md) · [`ai_docs/HARDENING_W3_TYPE_DESIGN_PORTFOLIO.md`](ai_docs/HARDENING_W3_TYPE_DESIGN_PORTFOLIO.md) |
| Obsidian vault | [`the-workflow-engine-vault/Hardening Fleet 2026-05-21.md`](the-workflow-engine-vault/Hardening%20Fleet%202026-05-21.md) |
| stcortex | namespace `workflow_trace_hardening_2026_05_21`, memory id **17939** (meta) — `~/.local/bin/stcortex inspect workflow_trace_hardening_2026_05_21` |
| POVM (deprecated mirror; stcortex canonical) | namespace `workflow_trace_hardening_2026_05_21`, id `2c8427fa-d87d-432e-9821-c6c7512c4d71` |
| tracking DB | `~/.local/share/habitat/injection.db` → `causal_chain` id **113**, label `workflow_trace_hardening_fleet_2026_05_21` |
| Zen audit packets | `~/projects/shared-context/agent-cross-talk/2026-05-21T*_command_zen_review_request_hardening_w[1-4]*.md` |

A fresh context window opens **`the-workflow-engine/CLAUDE.local.md`**, reads this block, and
reaches all hardening state from the table above.

---

## 🟢 CURRENT STATUS SNAPSHOT — S1002209+M0/M1 implementation wave (verified 2026-05-20 08:19 +1000)

**Project state:** `workflow-trace` is no longer planning-only. G9 fired on 2026-05-17; HOLD-v2 is lifted. The full 26-module Rust architecture is now present in `src/` with both binaries and the shared `workflow_core` library.

**Latest verified git anchor:** `9db534d hardening(the-workflow-engine): Cluster H — m40/m41/m42 god-tier pass` on branch `main`.

**Implemented clusters/modules:**
- Cluster A: m1/m2/m3 substrate ingest — Atuin, stcortex narrowed consumer, injection.db.
- Cluster B: m4/m5/m6 habitat observation — cascade, battern, context cost.
- Cluster C: m7/m12/m13 correlation/output — workflow runs, CLI reports, stcortex writer.
- Cluster D: m8/m9/m10/m11 trust spine — POVM prereq, namespace guard, Ember CI gate, fitness-weighted decay.
- Cluster E: m14/m15 evidence/pressure — lift and pressure register.
- Cluster F: m20/m21/m22/m23 KEYSTONE iteration — PrefixSpan, variant builder, k-means features, proposer.
- Cluster G: m30/m31/m32/m33 bank/select/dispatch/verify.
- Cluster H: m40/m41/m42 substrate feedback — Nexus emit, LCM RPC, stcortex emit.

**Verification receipts from this snapshot:**
- `cargo check --all-targets --all-features` = PASS.
- `cargo test --all-targets --all-features -- --format terse` = **1090 passed, 0 failed, 1 ignored**.
- Active Rust surface: `src` 119 files / 118 `.rs`; `tests` 11 files / 9 `.rs`.
- Implemented module directories: 26/26.
- Docs/spec surface still large and useful: `ai_docs` 64 md, `ai_specs` 76 md, `ultramap` 14 md, vault 103 files.

**Known drift / dirty state:**
- Git working tree is not pristine: `.obsidian/workspace.json`, `pre-framework-consolidation/WATCHER_DEPLOYMENT_WATCH_JOURNAL_S1001982.md`, `src/m30_bank/mod.rs`, and vault Watcher journal files are modified.
- Some older sections below still preserve historical planning-only/HOLD-v2 language as archaeology. Treat this snapshot + `GATE_STATE.md` G9-fired table + git history as current authority.
- Build warning persists by design until live POVM CR-2 is explicitly verified: `POVM_CR2_DEPLOYED=1` not set. Workflow-trace is stcortex-routed per m42 ADR, so this is a trust-gate warning, not a blocker for stcortex-only paths.

**Current next moves:**
1. Inspect/resolve the dirty `src/m30_bank/mod.rs` delta.
2. Run full gate including clippy pedantic again after any m30 edit is accepted.
3. Exercise binaries beyond compile/test: `wf-crystallise` report path and `wf-dispatch` dry-run/verification path.
4. Bring up Conductor/weaver/zen/enforcer when Luke wants live dispatch-plane soak; keep real dispatch human-ratified.

---

## 🟢 RESUME FROM HERE — S1002127 Cold-Start Entry (Workflow-Trace Scaffold Closeout)

> **For a fresh Claude session:** read this section FIRST, then drop into the workstream rich-block below (`## S1002127 — Scaffold Waves 0/1/2/3/4 (LIVE)`) for Wave-by-Wave detail.

**Git anchor:** commit `2536f4a` on `main`, pushed `origin` (GitHub) + `gitlab`. Walk with `git show 2536f4a --stat | head` for the 326-file scaffold inventory; `git log --oneline 2536f4a..HEAD` for any post-closeout deltas.

**stcortex anchor** (Surface 3 — substrate persistence; namespace `workflow_trace_scaffold_s1002127`):

```bash
# Inspect via CLI:
~/.local/bin/stcortex inspect workflow_trace_scaffold_s1002127 --limit 20

# Or via MCP from inside Claude:
mcp__stcortex-mcp__stcortex_inspect(namespace="workflow_trace_scaffold_s1002127", limit=20)
mcp__stcortex-mcp__stcortex_recall(namespace="workflow_trace_scaffold_s1002127", anchors=["workflow_trace_scaffold_s1002127_genesis"])
```

Stored **6 memories** (IDs 16603-16608) + **14 bidi pathways** (7 pairs) — written 2026-05-17 S1002127:

| Memory ID | Slug | Modality | Role |
|---|---|---|---|
| **16603** | `workflow_trace_scaffold_s1002127_genesis` | meta | **PRIMARY ENTRY** — full scaffold state + RESUME ENTRY pointer back to this file |
| 16604 | `workflow_trace_decision_v1_3_binding` | semantic | v1.3 binding spec (26 modules · 8 clusters · 2 binaries · ORAC pattern) |
| 16605 | `workflow_trace_decision_m42_pivot` | semantic | D-S1001982-01 m42 stcortex-only pivot ADR |
| 16606 | `workflow_trace_decision_prime_directive_waiver` | semantic | Luke S1002127 scope-override verbatim + tight/wide table |
| 16607 | `workflow_trace_decision_g8_persistence` | semantic | D-S1002127-01 G8 stcortex persistence plan ADR |
| 16608 | `workflow_trace_decision_escape_surface_cardinality_7` | semantic | D-S1002127-02 EscapeSurfaceProfile cardinality 6→7 ADR |

Pathways (slug-based bidi, 7 pairs; weights 0.80-0.95): `genesis ↔ v1_3_binding` (0.95) · `genesis ↔ prime_directive_waiver` (0.95) · `genesis ↔ g8_persistence` (0.95) · `genesis ↔ escape_surface_7` (0.90) · `v1_3_binding ↔ m42_pivot` (0.85) · `v1_3_binding ↔ escape_surface_7` (0.85) · `m42_pivot ↔ g8_persistence` (0.80). Reverse-anchors (`ai_docs:<path> ; vault:[[<wikilink>]] ; claude_local:<heading>`) embedded in every memory `content` field per [G8 persistence ADR](ai_docs/optimisation-v7/decisions/2026-05-17-g8-stcortex-persistence-plan.md) rule § 1.d.

**POVM (Surface 3-alternate — MIRRORED 2026-05-17 per Luke S1002127 override):** Initial Command interpretation was to skip POVM per the m42 stcortex-only ADR. Luke explicitly overrode with `also execute the POVM mirror` directive. Command position recorded:

> _Luke override accepted. Per CLAUDE.md "make the call, act, flag": I'm mirroring to POVM as instructed but the m42 stcortex-only ADR stays in force — stcortex remains canonical source of truth; POVM serves as historical mirror only (not a write-back source). Loading POVM MCP tools._

6 memories ingested under POVM v2 namespace `workflow_trace_scaffold_s1002127` (`source: command-s1002127-scaffold-closeout`):

| stcortex ID | POVM UUID | Role |
|---|---|---|
| 16603 | `6a479092-96a7-4eec-b390-a847db8f4455` | GENESIS |
| 16604 | `14a16b73-04ae-4636-8413-cafb221d1e6f` | v1.3 binding spec |
| 16605 | `79ea0e65-92e6-41d1-b754-fb58a0ff7591` | D-S1001982-01 m42 ADR (mirror is Luke-override; does NOT retract ADR) |
| 16606 | `2c2a5c9f-dd74-447b-b928-35f80d4bbebb` | PRIME_DIRECTIVE_WAIVER |
| 16607 | `6ce671a6-4d16-4340-a5e7-82230314d253` | D-S1002127-01 G8 persistence ADR |
| 16608 | `5e0f15a5-f72e-497d-8753-16e99a3eb18c` | D-S1002127-02 EscapeSurfaceProfile cardinality-7 ADR |

POVM v2 has no native bidi-pathway primitive (different paradigm); the 14 bidi pathways in stcortex are NOT mirrored — namespace grouping (`workflow_trace_scaffold_s1002127`) serves as the implicit relation surface in POVM. m42 ADR routing (stcortex-only for substrate-feedback writes) **remains in force**; POVM mirror is historical-anchor-only.

**🔴 INCIDENTAL DRIFT FLAG (AP-V7-13 firing live):** `povm_stats` at write time reported `learning_health=0.9162` (pre-CR-2 inflated value) — workspace post-CR-2 expected ~0.067. POVM v2 health-200 but serving pre-CR-2 binary. This is the EXACT condition that triggered the m42 ADR. Flagged for separate Luke action; does not affect this mirror's integrity (POVM is just a static store for our purposes).

**Obsidian anchors** (Surface 2 — vault mirror; bidi):

- Project-local vault (`the-workflow-engine-vault/`):
  - [[Scaffold Wave 0-2 — Session S1002127]] — session summary note (Wave 2E authored; Wave 3/4 closeout appended)
  - [[Cluster A Scaffold — Module Specs S1002127]] … [[Cluster H Scaffold — Module Specs S1002127]] — 8 per-cluster scaffold notes
  - [[HOME]] · [[MASTER_INDEX]] · [[GOD_TIER_CONSOLIDATION_S1001982]] · [[ULTIMATE_DEPLOYMENT_FRAMEWORK_S1001982]]
- Main habitat vault (`~/projects/claude_code/`):
  - [[workflow-trace — S1002127 Scaffold Closeout]] — new anchor note (this closeout; bidi-linked here)
  - [[stcortex — Pioneer Capability Dossier 2026-05-10]] — substrate workflow-trace routes m42 to
  - [[POVM Engine]] — substrate workflow-trace is DECOUPLED from (m42 ADR)

**ai_docs anchors** (Surface 1 — canonical):

- [`PRIME_DIRECTIVE_WAIVER.md`](PRIME_DIRECTIVE_WAIVER.md) — scope override authorising the scaffold
- [`CHANGELOG.md`](CHANGELOG.md) — versioned spec deltas v0.0.0-spec.0/1/2/3 + Wave 4
- [`GATE_STATE.md`](GATE_STATE.md) — G3 dropped · G4 CLOSED · G7 PENDING · G9 BLOCKED · B4 CLOSED
- [`ARCHITECTURE.md`](ARCHITECTURE.md) — 26-module / 8-cluster / 9-layer canonical map
- [`ai_specs/INDEX.md`](ai_specs/INDEX.md) · [`ai_specs/MODULE_MATRIX.md`](ai_specs/MODULE_MATRIX.md)
- [`ai_docs/INDEX.md`](ai_docs/INDEX.md) · [`ultramap/README.md`](ultramap/README.md)
- [`ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md) — Frame-A dual-pass analysis (11 NA gaps)
- 3 ADRs in [`ai_docs/optimisation-v7/decisions/`](ai_docs/optimisation-v7/decisions/):
  - `2026-05-17-m42-stcortex-only-pivot.md` (D-S1001982-01)
  - `2026-05-17-g8-stcortex-persistence-plan.md` (D-S1002127-01)
  - `2026-05-17-escape-surface-cardinality-7-privilege-escalation.md` (D-S1002127-02)

**State at closeout** (2026-05-17 S1002127):

- HOLD-v2 envelope INTACT (`0 .rs` files in active scope; `0 Cargo.toml`)
- G9 NOT fired — Luke types `start coding workflow-trace` to unlock; first wave will be Cluster D Day 1 (m8 build-cfg → m9 namespace guard → m10 Ember CI → m11 decay) per non-negotiable phase-1 framework
- Zen G7 AMEND-loop: AUDIT-REQUEST v2 filed (2026-05-17T160500Z); v3 owed for D-S1002127-02 cardinality bump
- 3 Luke physical actions standing: B1 G7 verdict · B2 v1.3 patch green-light · B3 Conductor `auto_start=false` → `devenv -c ~/.config/devenv/devenv.toml start --services weaver,zen,enforcer`
- Workspace-root `~/claude-code-workspace/CLAUDE.local.md` "The Workflow Engine" row stale (project charter forbids Command from amending; Luke action required OR explicit waiver to authorise Command)

**One-line resume for a fresh Claude context window:**

```bash
cd /home/louranicas/claude-code-workspace/the-workflow-engine && \
  git log --oneline -1 && \
  ~/.local/bin/stcortex inspect workflow_trace_scaffold_s1002127 --limit 6 2>/dev/null && \
  head -120 CLAUDE.local.md
```

---

## How this file maps to CLAUDE.md

| CLAUDE.md / workspace protocol | What this file provides |
|---|---|
| §0 multi-agent context | **State surfaces** table below — actual blackboard rows, broadcast topics, Atuin keys, vault mirror, stcortex namespace |
| §1 think before coding | **Pending decisions** table — open ambiguities, surface (don't pick silently) |
| §2 simplicity first | **Scope constraints** — active HOLD-v2 envelope, planning-only flag, PRIME_DIRECTIVE_WAIVER scope-tight/wide table |
| §4 goal-driven | **Success criteria** per in-flight workstream block (S1002127 scaffold + V7 + m42 pivot) |
| §5.2 broadcast etiquette | **In-flight** handshakes — what's outstanding, who owes a reply (Command-2/Command-3 5× silent; Zen G7 verdict pending) |
| §5.3 blackboard etiquette | **Active state** table — which rows this pane owns vs watches; gate state owned by Command |
| §5.4 RALPH loop | Current generation + drift flags (RALPH gen 7,622 fitness 0.6987 trending up; LTP/LTD 0.043 35× below target) |
| §5.5 recovery / handoff | **Resume protocol** — full cold-start sequence (auto-load → checkpoint → re-probe → re-activate persona → re-apply scope) |

---

## Last saved session

- **Date:** 2026-05-17 (S1001982)
- **Label:** `workflow-engine-ultimate-framework`
- **Pane:** Tab 1 Orchestrator top-left (Command)
- **Persisted across 6 surfaces:** primary file + Obsidian vault mirror + stcortex memory id 16526 (`habitat_sessions` ns) + POVM pathway (overlap mirror) + RM id `r6a092b6e00e0` + atuin KV (`habitat.last_session*`)
- **Resume:** `atuin kv get --namespace habitat habitat.last_session` → opens checkpoint path

---

## Active state (delta from charter)

| State | Value |
|---|---|
| **Phase** | ACTIVE implementation + hardening — G9 fired 2026-05-17, HOLD-v2 lifted. 26-module Rust codebase (~31k LOC); Hardening Fleet 2026-05-21 complete (W0–W5); assessment-driven remediation S1003733 in progress |
| **Gates** | G1–G9 all resolved; **G9 FIRED 2026-05-17** — live record in `GATE_STATE.md` |
| **Last spec version** | v1.2 binding (Zen-audit-locked); **v1.3 patch pending** (single-phase override absorption) |
| **Vault** | 88 files / 2.4MB across root + `module specs/` (9 files) + `boilerplate modules/` (10 subdirs + 4 gold-standard exemplars) + `deployment framework/` (10 phase docs) |
| **Git** | branch `main` at `2fbfbd1` (Hardening Fleet W5 docs persist); assessment-remediation S1003733 in flight |
| **Services** | 11/11 healthy at last probe (8082, 8083, 8092, 8111, 8120, 8125, 8130, 8132, 8133, 8180, 10002) |
| **Watcher** | ready · eligible · 48,723 observations · proposals_submitted 0 · R13 elapsed |
| **Substrate** | LTP/LTD = 0.043 (35× below healthy); substrate_LTP_density 0.018 (Phase 1 PASSING) |

---

## Pending Luke decisions (6 critical-path blockers)

| # | Blocker | Resolution |
|---|---|---|
| **B1** | G7 Zen URGENT block on G9 out-of-sequence | Per-gate waiver OR drive G1-G8 in sequence |
| **B2** | v1.3 patch not yet authored | Luke green-lights Command to author (1-2 days) |
| **B3** | Conductor Waves 1B/1C/2/3 `auto_start=false` | Luke @ terminal: `~/.local/bin/devenv -c ~/.config/devenv/devenv.toml start weaver/zen/enforcer` |
| **B4** | Ember rubric §5.1 Held-semantics amendment | Watcher's lane; awaits Luke direction |
| **B5** | POVM `:8125` redeploy verify (G3) | Luke `devenv restart povm-engine` (~hour) |
| **B6** | Power-structure ambiguity (Luke override vs Zen G7 audit precedence) | Luke clarifies in 1 decision |

**4 of 6 sequenceable; 2 are single-Luke-action.** See [GOD_TIER_CONSOLIDATION Part VIII](the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md) for full context.

---

## S1002209 — Luke Task-Cascade 1-6 Execution

**Authorisation:** Luke S1002209 directive verbatim — _"continue plan for and then complete each task 1. 2. 3. 4. 5. 6. in logical order to the highest level of excellence and impact proceed seamlessly"_

**Interpretation (logged for Zen audit-trail):**
- Task 1 (file Zen v3 AUDIT-REQUEST) — Command lane; EXECUTED
- Task 2 (Luke clears B1 verdict) — Luke directive = drive-G1-G8-sequence path elected (NOT per-gate waiver — preserves Zen audit authority); GATE_STATE updated
- Task 3 (B2 green-light → Command authors v1.3 patch) — green-lit; v1.3 binding at `ai_docs/GENESIS_PROMPT_V1_3.md` (46K, Appendix A amendment record) already authored; v3 AUDIT-REQUEST covers full amendment scope; DELIVERED
- Task 4 (Luke runs `devenv -c ~/.config/devenv/devenv.toml start --services weaver,zen,enforcer`) — Luke @ terminal action; project rule forbids agent service-start; STANDING-LUKE
- Task 5 (workspace-root CLAUDE.local.md row) — Luke directive = "complete each task" = Command-amend waiver for this row only; EXECUTED at `~/claude-code-workspace/CLAUDE.local.md` § "The Workflow Engine"
- Task 6 (G1-G8 green → Luke types `start coding workflow-trace` → G9 fires → Cluster D Day 1) — G9 fire requires literal phrase; not in Luke's S1002209 message; STAGED (Cluster D Day-1 specs verified 4/4 present + boilerplate clones available); STANDING-LUKE phrase

**Status table:**

| Step | Action | Owner | State | Receipt |
|---|---|---|---|---|
| 1 | File Zen v3 AUDIT-REQUEST | Command | ✅ EXECUTED | [`2026-05-17T093800Z_command_g7_audit_request_v3_cardinality_7_plus_wave4b_amendment.md`](../../projects/shared-context/agent-cross-talk/2026-05-17T093800Z_command_g7_audit_request_v3_cardinality_7_plus_wave4b_amendment.md) |
| 2 | B1 path-elected (drive G1-G8 sequence) | Luke directive → Command | ✅ EXECUTED | [`GATE_STATE.md`](GATE_STATE.md) B1 row + S1002209 directive header |
| 3 | B2 green-light + v1.3 delivery confirmation | Luke directive → Command | ✅ DELIVERED | [`ai_docs/GENESIS_PROMPT_V1_3.md`](ai_docs/GENESIS_PROMPT_V1_3.md) (binding) + v3 AUDIT-REQUEST (amendment scope) + [`GATE_STATE.md`](GATE_STATE.md) B2 row |
| 4 | Conductor `devenv -c ~/.config/devenv/devenv.toml start --services weaver,zen,enforcer` | Luke @ terminal | ⏳ STANDING-LUKE | Non-blocking pre-G9; binaries already in `~/.local/bin/` |
| 5 | Workspace-root CLAUDE.local.md "Workflow Engine" row | Luke directive → Command | ✅ EXECUTED | `~/claude-code-workspace/CLAUDE.local.md` § "The Workflow Engine" amended; project-charter forbidance carved out for this row only |
| 6 | G9 fire / Cluster D Day-1 m8 | Luke phrase ✓ → Command | 🔥 **G9 FIRED + m8 LIVE** | Luke typed `start coding workflow-trace` S1002209. HOLD-v2 envelope LIFTED. m8 module IMPLEMENTED + tests GREEN: Cargo.toml + build.rs + src/lib.rs + src/m8_povm_build_prereq/{mod,cfg,error,health}.rs + src/bin/{wf_crystallise,wf_dispatch}.rs + tests/m8_integration.rs. **69 tests pass** (64 lib + 5 integration; 1 `#[ignore]` live POVM probe). 4-stage gate GREEN: cargo check ✓ · clippy -D warnings ✓ · clippy pedantic -D warnings ✓ · cargo test ✓. Next: m9 namespace guard, m10 Ember CI, m11 compound decay per Cluster D Day-1 build order |

**v3 AUDIT-REQUEST scope (filed 2026-05-17T093800Z):** Group A (v2 absorbed: m42 stcortex-only pivot) + Group B (D-S1002127-02 EscapeSurfaceProfile cardinality 6→7 `PrivilegeEscalation` @ ord 30; ~12 file amendments) + Group C (D-S1002127-03 substrate-as-actor v0.2.0 deferrals NA-GAP-07/08/10) + Group D (Wave 4.B substrate-as-actor remediation; 8/11 NA gaps closed; 4 sub-groups: substrate-couplings/ · refusal taxonomy · m42 § 5.1 outbox · BENCHMARK_SPEC substrate-side). Drift flags carried: test-budget 1,562/1,594/**1,599** (Command recommends 1,599 per G6 latest); 12 substrate-confirmable receipts catalogued as cross-habitat ADR work-items.

**Critical path forward (gate sequence):**
1. **G7** Zen verdict on v3 AUDIT-REQUEST → APPROVE / AMEND / PARTIAL APPROVE per D-B6
2. **G5** Interview / F2 round (Command lane; ~half-session if needed; v1.3 binding obviates if Zen APPROVE)
3. **G6** Dual-frame gap analysis (Conventional + NA; Wave 4.B substrate-frame already authored; Conventional gap analysis remains)
4. **G8** stcortex persistence per [`ai_docs/optimisation-v7/decisions/2026-05-17-g8-stcortex-persistence-plan.md`](ai_docs/optimisation-v7/decisions/2026-05-17-g8-stcortex-persistence-plan.md) — mechanical, ~46 memories + ~60 pathways under `workflow_trace_*` namespace
5. **G9** Luke types `start coding workflow-trace` → HOLD-v2 envelope lifts → Cluster D Day 1 begins

---

## In-flight (no response from peers since prior handshake)

- **Command-2 / Command-3 handshakes** filed 5× now (11:45 · 11:57 · 04:12Z V7-close · 16:00Z m42-amendment · 2026-05-17T163100Z S1002127 v3 state-delta refresh). 6th handshake S1002209 task-cascade filed concurrent with v3 AUDIT-REQUEST. Receive-mode v2 standing per AP-V7-08 (silence ≠ consent). Luke directed to wake panes (Action 2 in LUKE_ACTION_NEEDED v2).
- **POST-ARMADA-HYGIENE workstream** (this morning's first handshake): repo push-state assignments still unacked.

## S1002127 — Scaffold Waves 0/1/2/3/4 (LIVE) · workstream rich-block

**Authorisation:** Luke S1002127 PRIME_DIRECTIVE_WAIVER (scaffold-only override — see [`PRIME_DIRECTIVE_WAIVER.md`](PRIME_DIRECTIVE_WAIVER.md)). G9 NOT fired; HOLD-v2 envelope intact for code authoring; scaffold scope: structure + markdown specs + .claude config + plan.toml only. Luke S1002127 follow-on directives: D-A m23 Pure Rust · D-B m1 page_size 2_000 · D-C m10 Hybrid CI-FAIL+allowlist (closes G4/B4) · D-D m11 dual-read soak · D-E m13 threshold >0.015 · D-S1002127-02 EscapeSurfaceProfile cardinality 6→7 with `PrivilegeEscalation` · NA-GAP-01..11 "as per proposal".

**Success criteria (CLAUDE.md §4 — verifiable checks):**
- ✅ 26/26 per-module Rust spec files present at `ai_specs/modules/cluster-{A-H}/m<N>_<name>.md`; all carry YAML frontmatter + bidi anchors + 13 logical sections (heading-form variance accepted across 3 forms)
- ✅ 0 `.rs` files in active scope (38 in vault `boilerplate modules/` are intentional paste-templates)
- ✅ 0 `Cargo.toml` in active scope
- ✅ 8/8 `.claude/*.json` parse via `python3 -m json.tool`
- ✅ 4/4 `.claude/hooks/*.sh` executable
- ✅ 16 Mermaid diagrams across `ultramap/`
- ✅ All facts preserved: m42 POVM-decoupled · AP-V7-07 m23 no-auto-promote · AP-V7-08 m32 no-self-dispatch · `lcm.loop.create` (not `lcm.deploy`) · Cluster D `ship_first: true` × 4 · CC-1b resolved as `CC-1.subA` · EscapeSurfaceProfile 7-variant ordinal (rg shows 84 hits / 12 files)
- ✅ Four-surface persistence: ai_docs ✓ · vault ✓ · stcortex namespace RESERVED (G8 ADR pre-specifies writes) · CLAUDE.local.md anchor (this section)
- ✅ Wave 4.B NA-GAP substrate-as-primary remediation **CLOSED** 2026-05-17 (S1002127 continuation) — 8/11 NA gaps fully closed; 3/11 (NA-07 module / NA-08 / NA-10) deferred to v0.2.0 via D-S1002127-03 ADR with compensating controls. CHANGELOG `v0.0.0-spec.4` records full deltas.

**What landed (Waves 0+1+2+3 + part of 4):** ~210 files / ~145k+ words.
- Wave 0 (22 root anchors); Wave 1 (26 per-module specs, ~70k words); Wave 2A (`.claude/` 28 files); Wave 2B (ai_docs deep 11 files / ~19k); Wave 2C (ai_specs cross-cutting 33 files / ~29k; CC-1b → `CC-1.subA`); Wave 2D (ultramap deep 13 files / 16 Mermaid); Wave 2E (Obsidian sync 16 vault file changes + 14 repo files)
- Wave 3 verifier reports: agent-claim-verifier PASS-WITH-AMENDMENTS (20/20 hard); four-surface-persistence-verifier PARTIAL → addressed; na-gap-analyst Frame-A 11 NA gaps surfaced
- Wave 4.0: 5 decisions D-A..D-E surgically applied across 9 files (incl. GATE_STATE G4/B4 CLOSED)
- Wave 4.A: EscapeSurfaceProfile 6→7 (`PrivilegeEscalation` at ord 30); 12 file amendments + new ADR `2026-05-17-escape-surface-cardinality-7-privilege-escalation.md` + DECISION_REGISTER D-S1002127-02
- Wave 4.B: NA-GAP-01..11 substrate-as-primary remediation in flight (~10 new + 5 amended target)
- See [`CHANGELOG.md`](CHANGELOG.md) for full per-wave deltas. NO `.rs` files. NO Cargo.toml.

**In flight (CLAUDE.md §5.2 broadcast etiquette):**
- ~~Wave 4.B `na-gap-analyst → substrate-as-primary remediation`~~ — **CLOSED 2026-05-17** with full 5-item closeout: `substrate-couplings/` directory (4 files) + ERROR_TAXONOMY.md amendment (RefusalToken) + m42_stcortex_emit.md § 5.1 outbox-policy amendment + BENCHMARK_SPEC.md substrate-side benchmarks amendment + `ai_docs/decisions/2026-05-17-substrate-as-actor-deferrals.md` ADR (D-S1002127-03). 8/11 NA gaps closed; 3/11 deferred to v0.2.0 with compensating controls.
- Zen G7 AMEND-loop: amended AUDIT-REQUEST v3 owed to Zen — **scope expanded** to include D-S1002127-02 (cardinality 7) + D-S1002127-03 (substrate-as-actor deferrals) + 4 amended files + 4 new substrate-couplings/ files (~`2026-05-17T<utc>_command_g7_audit_request_v3_cardinality_7_plus_wave4b_amendment.md`)
- Workspace-root CLAUDE.local.md "The Workflow Engine" row stale — project charter forbids me from editing; Luke action standing

**Flagged for Luke / Zen / Watcher escalation:**
- EscapeSurfaceProfile cardinality bump (6 → 7) requires Zen G7 re-audit via D-B6 AMEND-loop (filing pending Wave 4.B completion)
- Test budget drift 1,562 / 1,594 / 1,599 (TEST_STRATEGY locked at **1,594** per G6 latest)
- NA-GAP-01..11 (substrate-as-primary frame) ~8h remediation in progress; Wave 4.B output will surface 3 most important spec-level changes for future per-module spec fold-in
- Template heading-form variance across 3 forms (`## N.` / `## N —` / `## §N`) — accepted as canonical; document in `ai_specs/INDEX.md` (Wave 5 cosmetic)
- 11 module specs (Cluster B/C/E/F) missing bottom `Back to:` anchor — accepted (top anchor sufficient); re-author if Luke prefers

**Next on G9 fire (Luke types `start coding workflow-trace`):**
1. Cluster D ships Day 1 in this order: m8 build-script cfg → m9 namespace guard → m10 Ember CI gate → m11 decay (per non-negotiable phase-1 framework)
2. Cluster A readers Day 2 (m1 atuin · m2 stcortex consumer · m3 injection_db)
3. Cluster B/C build-out Day 3
4. Cluster F KEYSTONE iteration Days 5–7 (m20 PrefixSpan ~280 LOC pure-Rust per D-A; bench targets locked)
5. Cluster G/H thereafter
6. Per [`G8 stcortex persistence ADR`](ai_docs/optimisation-v7/decisions/2026-05-17-g8-stcortex-persistence-plan.md): on G8-green, write ~46 memories + ~60 pathways under `workflow_trace_*` namespace (mechanical, not interpretive)

**Canonical entry points (single-load):**
- [`README.md`](README.md) — project landing
- [`ARCHITECTURE.md`](ARCHITECTURE.md) — 26-module / 8-cluster / 9-layer / 2-binary map (EscapeSurfaceProfile 7-variant ordinal locked here)
- [`GATE_STATE.md`](GATE_STATE.md) — live G1-G9 + B1-B6 (G3 dropped · G4/B4 CLOSED · B5 dropped · B6 resolved)
- [`PRIME_DIRECTIVE_WAIVER.md`](PRIME_DIRECTIVE_WAIVER.md) — scaffold-only scope override + 4-wave record
- [`CHANGELOG.md`](CHANGELOG.md) — versioned spec deltas (v0.0.0-spec.0/1/2/3/+ Wave 4)
- [`plan.toml`](plan.toml) — machine-readable architecture + `[scaffold_meta].four_surfaces`
- [`ai_docs/INDEX.md`](ai_docs/INDEX.md) · [`ai_specs/INDEX.md`](ai_specs/INDEX.md) · [`ai_specs/MODULE_MATRIX.md`](ai_specs/MODULE_MATRIX.md) · [`ultramap/README.md`](ultramap/README.md)
- [`ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md) — Frame-A dual-pass analysis
- [`ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md`](ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md) — D-S1001982-01
- [`ai_docs/optimisation-v7/decisions/2026-05-17-g8-stcortex-persistence-plan.md`](ai_docs/optimisation-v7/decisions/2026-05-17-g8-stcortex-persistence-plan.md) — D-S1002127-01
- [`ai_docs/optimisation-v7/decisions/2026-05-17-escape-surface-cardinality-7-privilege-escalation.md`](ai_docs/optimisation-v7/decisions/2026-05-17-escape-surface-cardinality-7-privilege-escalation.md) — D-S1002127-02

**Vault mirrors (bidi-linked):**
- [[Scaffold Wave 0-2 — Session S1002127]] — session summary
- [[Cluster A Scaffold — Module Specs S1002127]] … [[Cluster H Scaffold — Module Specs S1002127]] — 8 per-cluster scaffold notes
- [[HOME]] · [[MASTER_INDEX]] — vault landing + catalogue (both updated with Wave-0/1/2/3 §7b section)

---

## V7 Optimisation + m42 stcortex-only pivot (2026-05-17 ~14:00–16:00 local)

**Status:** V7 author wave CLOSED · 45 deliverables / ~115k words / planning-only / HOLD-v2 respected · m42 pivot AMENDMENT landed via D-B6 AMEND-loop · awaiting Zen G7 verdict.

### Canonical (single-load entry points)

- **V7 framework:** [`ai_docs/optimisation-v7/OPTIMISATION_FRAMEWORK_V7_FINAL.md`](ai_docs/optimisation-v7/OPTIMISATION_FRAMEWORK_V7_FINAL.md) — table of contents for all 44 ai_docs deliverables
- **m42 ADR:** [`ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md`](ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md) — 48-decision grilling outcome
- **v1.3 spec patch:** [`ai_docs/GENESIS_PROMPT_V1_3.md`](ai_docs/GENESIS_PROMPT_V1_3.md) — binding spec + Appendix A amendment
- **Decision register:** [`ai_docs/optimisation-v7/DECISION_REGISTER.md`](ai_docs/optimisation-v7/DECISION_REGISTER.md) — 61 decisions made (13 V7 + 48 grilling); 0 deferred

### Vault mirrors (bidi-linked)

- [[the-workflow-engine-vault/optimisation-v7/HOME|V7 vault HOME]] — landing for V7 subtree
- [[the-workflow-engine-vault/optimisation-v7/V7 Optimisation Framework]] — mirror of FINAL canonical
- [[the-workflow-engine-vault/optimisation-v7/m42 stcortex-only pivot ADR]] — mirror of ADR
- [[the-workflow-engine-vault/optimisation-v7/Session S1001982 m42 pivot grilling]] — 12-round substrate-pivot doctrine

### Substrate notes (bidi from V7 → external vault)

- [[stcortex — Pioneer Capability Dossier 2026-05-10]] — the substrate workflow-trace routes m42 to exclusively (M0 onward)
- [[POVM Engine]] — DECOUPLED from workflow-trace per 2026-05-17 ADR (workspace charter 2026-07-10 decommission unaffected for other services)

### What the m42 pivot did

m42 module pivoted from POVM-dual-path to stcortex-only routing for substrate-feedback, effective M0 from G9-fire. Module renamed `src/m42_povm_dual/` → `src/m42_stcortex_emit/`. Triggered by live-probe finding: POVM `:8125` health-200 but serving pre-CR-2 binary (`learning_health=0.9146` vs expected ~0.067). Crystallised as AP-V7-13 (Health-200 ≠ behaviour-verified). Luke directed 12-round AskUserQuestion grilling; 48/48 Command recommendations accepted.

**Featureset preserved 1:1:** CC-5 substrate-learning loop / fitness-delta constants / outbox-first JSONL / circuit-breaker (2 peers) / Watcher Class-I (extended to stcortex pathway.weight delta) / substrate-condition acceptance.

**Risk surface reduced:** F7 antipattern eliminated · D-B5 POVM restart Luke action dropped · D25 mid-soak cutover dance dropped · POVM binary-drift no longer a workflow-trace concern.

### Luke physical actions remaining (3 items, ~10 min — was 4 pre-pivot)

Filed at [`~/projects/shared-context/agent-cross-talk/2026-05-17T160300Z_command_luke_action_needed_v2.md`](file:///home/louranicas/projects/shared-context/agent-cross-talk/2026-05-17T160300Z_command_luke_action_needed_v2.md):

1. Conductor `devenv -c ~/.config/devenv/devenv.toml start --services weaver,zen,enforcer` (D-B3 — unblocks Phase 3 Track 2)
2. Wake Tab-1 C-2 + C-3 panes (D-Handshake — 4 handshakes silent now)
3. Approve hybrid CI-FAIL+allowlist OR file own Ember §5.1 direction (D-B4 — Watcher amends per AP27)

**Dropped:** D-B5 POVM `:8125` restart (workflow-trace POVM-decoupled per m42 pivot).

### Zen G7 verdict pending

AUDIT-REQUEST v2 filed at [`~/projects/shared-context/agent-cross-talk/2026-05-17T160500Z_command_g7_audit_request_v1_3_amendment.md`](file:///home/louranicas/projects/shared-context/agent-cross-talk/2026-05-17T160500Z_command_g7_audit_request_v1_3_amendment.md). Scope: amendment-only delta + cluster-H integration (per D-B6 AMEND-loop). Drift flagged: test-budget figure (1,562 / 1,594 / 1,599 across V7 docs) — Command recommends 1,599 (G6 latest with mutation allocation).

---

## Resume protocol (next session cold-start)

1. **Auto-load:** opening any file under `the-workflow-engine/` auto-loads this CLAUDE.local.md + [CLAUDE.md](CLAUDE.md) + workspace-root `~/claude-code-workspace/CLAUDE.md` + workspace-root `~/claude-code-workspace/CLAUDE.local.md`.
2. **Find the latest checkpoint:**
   ```bash
   latest=$(atuin kv get --namespace habitat habitat.last_session 2>/dev/null)
   [ -z "$latest" ] && latest=$(ls -t ~/projects/shared-context/sessions/*.md | head -1)
   echo "$latest"
   ```
   Expected: `~/projects/shared-context/sessions/2026-05-17T124122_workflow-engine-ultimate-framework.md`
3. **Read checkpoint** for full session summary + resume instructions.
4. **Re-probe 11 services** to confirm habitat healthy:
   ```bash
   for port in 8082 8083 8092 8111 8120 8125 8130 8132 8133 8180 10002; do
     curl -sS -o /dev/null -w "$port=%{http_code}\n" --max-time 1 "http://localhost:$port/health" 2>/dev/null
   done
   ```
5. **Git delta since checkpoint:**
   ```bash
   git log --oneline 76ea4d6..HEAD
   ```
6. **Re-activate Command persona** — Tab 1 Orchestrator top-left; receive-mode for C-2/C-3; no Tab navigation; channel-based comms only.
7. **Re-apply scope constraints** (planning-only; HOLD-v2; no code; no cargo; no rename; ignore TaskCreate).
8. **Read in order:**
   - [HOME.md](the-workflow-engine-vault/HOME.md)
   - [MASTER_INDEX.md](the-workflow-engine-vault/MASTER_INDEX.md)
   - [GOD_TIER_CONSOLIDATION_S1001982.md](the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md)
   - [ULTIMATE_DEPLOYMENT_FRAMEWORK_S1001982.md](the-workflow-engine-vault/ULTIMATE_DEPLOYMENT_FRAMEWORK_S1001982.md)
   - Individual phase docs / cluster specs / boilerplate clones as needed
9. **Check inbox for new C-2/C-3/Watcher drops:**
   ```bash
   find ~/projects/shared-context/agent-cross-talk -name "*.md" -newer ~/projects/shared-context/agent-cross-talk/2026-05-17T115700Z_command_handshake_to_c2_c3_state_delta_refresh.md 2>/dev/null
   find ~/projects/shared-context/watcher-notices -name "*.md" -newer ~/projects/shared-context/watcher-notices/2026-05-17T023139_notify_5d20aaed98b0.md 2>/dev/null
   ```
10. **Continue from Pending Luke decisions** (table above) — none have moved; all 6 blockers still standing unless a fresh drop changes state.

---

## Session-specific Working Mode

- **Receive-mode v2** for peers (C-2 + C-3) — no new outbound on workflow-engine planning until their drops land
- **Watcher carriage** active — deployment-watch journal continuous; Watcher cadence is prompt-driven or cross-talk-delta-driven; NO autonomous loop
- **Zen audit lane** active — G7 audit gates everything downstream; v1.3 patch will trigger G7 re-audit
- **Power-structure ambiguity standing** — Luke override vs Zen G7 audit precedence; B6 must clarify before v1.3 patch authoring begins

---

## What's been added since the prior session checkpoint

Per [workflow-engine-code-base.md](the-workflow-engine-vault/workflow-engine-code-base.md) tracker (P10 → P15):

- **P10** Vault Save v1 + v2 (HOME + MASTER_INDEX + tracker + Vault Save Status + 8 mirror notes)
- **P11** Boilerplate Modules Clone (48 source files across 10 categories)
- **P12** Detailed Module Specs (8 parallel rust-pro agents; 41,508 words across 8 cluster specs)
- **P13** GOD_TIER_CONSOLIDATION (9 parallel Explore agents; ~7,000-word synthesis of all 77 vault files)
- **P14** Watcher Deployment Watch Journal absorbed (T0 baseline + 3 yellow signals: Class E ancestor-rhyme, Class I Hebbian silence, Class A G7 highest-leverage)
- **P15** ULTIMATE DEPLOYMENT FRAMEWORK (9 parallel specialist agents — 7 rust-pro + 1 security-auditor + 1 observability-engineer — 66,576 words across 10 phase docs + canonical synthesis)

Plus continuous: CLAUDE.local.md (workspace-root) Hebbian v3 row + CR-2 ship status updated via Zen-audited reconciliation; FP-discipline self-correction on Phase A count (15 not 13); bidirectional links audited and patched across all vault notes.

---

## State surfaces (reference — actual addresses)

Per CLAUDE.md §0 multi-agent context. Update when surfaces are added or rotated.

| Surface | Address | Purpose |
|---|---|---|
| **ai_docs canonical** | `ai_docs/` (61 files; V7 framework + GENESIS_v1_3 + Wave-2B deep + 3 ADRs + NA gap analysis) | Prescriptive structural + decisions persistence |
| **ai_specs prescriptive** | `ai_specs/` (61 files; INDEX + MODULE_MATRIX + 26 per-module + 8 layers + 8 synergies + 12 cross-cutting + 5 axes + substrates/ Wave 4.B) | Per-module Rust god-tier specs (HOLD-v2 markdown only) |
| **ultramap operational** | `ultramap/` (14 files; 16 Mermaid diagrams across DATA/CONTROL/CONTEXTUAL/INVARIANT/master + 7 schematics) | Runtime flow maps complementing canonical V7 ULTRAMAP |
| **Obsidian vault mirror** | `the-workflow-engine-vault/` (88 pre-existing + 9 new Wave-2E + 2 updated + 6 audited; ~2.4MB+) | Bidi-linked human-readable mirror |
| **stcortex namespace** | `workflow_trace_*` — **RESERVED, NOT WRITTEN pre-G8** per [`G8 persistence ADR`](ai_docs/optimisation-v7/decisions/2026-05-17-g8-stcortex-persistence-plan.md) (~46 mem + ~60 pathways planned) | Post-G8 substrate persistence |
| **CLAUDE.local.md anchor** | this file (project-local) + `~/claude-code-workspace/CLAUDE.local.md` (workspace; row stale → Luke action) | Live session-state delta |
| **plan.toml machine-readable** | [`plan.toml`](plan.toml) `[scaffold_meta].four_surfaces` enumerates all 4 | Scaffold-mastery input + machine-readable architecture |
| **`.claude/` config** | `.claude/{settings,context,status,anti_patterns,patterns}.json` + 6 agents + 6 commands + 4 hooks + 4 schemas + 3 queries (29 files) | Claude Code runtime + machine-readable registers |
| **Blackboard (SQLite)** | `~/.local/share/devenv/*.db` (workspace-scope; tables `fleet_status` / `pane_status`) | Shared work claims, milestone state (cross-pane) |
| **Broadcast (PV2 Kuramoto)** | `:8132` (PV2 sphere registration; r=0.924 at S1002127 boot) | Inter-pane mutual visibility |
| **Hot state (Atuin KV)** | namespace `habitat`; keys `habitat.last_session*`, `habitat.last_session_path` | Session-scoped variables |
| **Cross-talk inbox** | `~/projects/shared-context/agent-cross-talk/` | Peer handshakes (Command ↔ C-2 / C-3 / Zen) |
| **Watcher notices** | `~/projects/shared-context/watcher-notices/` (drop via `~/.local/bin/watcher notify`) | Watcher journal drops |
| **Session checkpoints** | `~/projects/shared-context/sessions/` | Cold-start anchors |
| **POVM (decoupled)** | DECOUPLED per [2026-05-17 m42 ADR](ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md) | Workflow-trace no longer depends |

---

## Bidirectional anchor footer

> **This file ↔ [CLAUDE.md](CLAUDE.md)** — project charter (structural)
> **Session checkpoint ↔ [`~/projects/shared-context/sessions/2026-05-17T124122_workflow-engine-ultimate-framework.md`](file:///home/louranicas/projects/shared-context/sessions/2026-05-17T124122_workflow-engine-ultimate-framework.md)**
> **Vault home ↔ [the-workflow-engine-vault/HOME.md](the-workflow-engine-vault/HOME.md)**
> **God-tier synthesis ↔ [the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md](the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md)**
> **Deployment recipe ↔ [the-workflow-engine-vault/ULTIMATE_DEPLOYMENT_FRAMEWORK_S1001982.md](the-workflow-engine-vault/ULTIMATE_DEPLOYMENT_FRAMEWORK_S1001982.md)**
> **Workflow tracker ↔ [the-workflow-engine-vault/workflow-engine-code-base.md](the-workflow-engine-vault/workflow-engine-code-base.md)**
> **Watcher journal ↔ [the-workflow-engine-vault/Watcher Deployment Watch Journal S1001982.md](the-workflow-engine-vault/Watcher%20Deployment%20Watch%20Journal%20S1001982.md)**
> **V7 vault subtree ↔ [the-workflow-engine-vault/optimisation-v7/HOME.md](the-workflow-engine-vault/optimisation-v7/HOME.md)** — 4 vault mirrors of V7 optimisation work
> **V7 canonical framework ↔ [ai_docs/optimisation-v7/OPTIMISATION_FRAMEWORK_V7_FINAL.md](ai_docs/optimisation-v7/OPTIMISATION_FRAMEWORK_V7_FINAL.md)** — 44 markdown deliverables / single-load entry
> **m42 ADR ↔ [ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md](ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md)** — 48-decision grilling outcome
> **v1.3 spec patch ↔ [ai_docs/GENESIS_PROMPT_V1_3.md](ai_docs/GENESIS_PROMPT_V1_3.md)** — binding spec + Appendix A amendment
> **Substrate (routed-to) ↔ [[stcortex — Pioneer Capability Dossier 2026-05-10]]** — main vault dossier (`~/projects/claude_code/`)
> **Substrate (decoupled) ↔ [[POVM Engine]]** — main vault note; workflow-trace no longer depends post 2026-05-17 ADR
> **Workspace charter (parent) ↔ `~/claude-code-workspace/CLAUDE.md` + `~/claude-code-workspace/CLAUDE.local.md`**

*Local session state last updated: 2026-05-17 ~16:15 (post-V7 closure + post-m42 stcortex-only pivot amendment + vault mirroring). Updates land here on every substantive session boundary.*
