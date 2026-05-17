# the-workflow-engine — Local Session State (delta)

> **Back to:** [CLAUDE.md](CLAUDE.md) — project charter (structural facts; do not duplicate here)
> **Session checkpoint:** [`~/projects/shared-context/sessions/2026-05-17T124122_workflow-engine-ultimate-framework.md`](file:///home/louranicas/projects/shared-context/sessions/2026-05-17T124122_workflow-engine-ultimate-framework.md)
> **Vault home:** [the-workflow-engine-vault/HOME.md](the-workflow-engine-vault/HOME.md)
> **God-tier synthesis:** [the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md](the-workflow-engine-vault/GOD_TIER_CONSOLIDATION_S1001982.md)
> **Deployment recipe:** [the-workflow-engine-vault/ULTIMATE_DEPLOYMENT_FRAMEWORK_S1001982.md](the-workflow-engine-vault/ULTIMATE_DEPLOYMENT_FRAMEWORK_S1001982.md)
> **Workflow tracker:** [the-workflow-engine-vault/workflow-engine-code-base.md](the-workflow-engine-vault/workflow-engine-code-base.md) — 15 phases / 13 decisions / 13 open issues
>
> **Synergy with [CLAUDE.md](CLAUDE.md):** the charter holds the rules (how to behave, which protocols govern the fleet). This file holds the state those rules operate over — live workstream, pending decisions, persisted surfaces, cold-start pointers. **Charter answers _how_; this file answers _what the world looks like right now_.** Do NOT duplicate charter content here.

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
| **Phase** | Planning-only · HOLD-v2 active · 0 LOC code · 41,508 words module specs + 66,576 words deployment framework + ~7,000 words consolidation |
| **Gates** | G1-G9 all NOT GREEN; G9 fired out-of-sequence (Zen URGENT block) |
| **Last spec version** | v1.2 binding (Zen-audit-locked); **v1.3 patch pending** (single-phase override absorption) |
| **Vault** | 88 files / 2.4MB across root + `module specs/` (9 files) + `boilerplate modules/` (10 subdirs + 4 gold-standard exemplars) + `deployment framework/` (10 phase docs) |
| **Git** | branch `main` at `76ea4d6` (CR-2b coactivation pair-loop existence-filter); 479 dirty files |
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

## In-flight (no response from peers since prior handshake)

- **Command-2 / Command-3 handshakes** filed 5× now (11:45 · 11:57 · 04:12Z V7-close · 16:00Z m42-amendment · 2026-05-17T163100Z S1002127 v3 state-delta refresh). Receive-mode v2 standing per AP-V7-08 (silence ≠ consent). Luke directed to wake panes (Action 2 in LUKE_ACTION_NEEDED v2).
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
- ⏳ Wave 4.B NA-GAP substrate-as-primary remediation in flight (closes NA-GAP-01..11)

**What landed (Waves 0+1+2+3 + part of 4):** ~210 files / ~145k+ words.
- Wave 0 (22 root anchors); Wave 1 (26 per-module specs, ~70k words); Wave 2A (`.claude/` 28 files); Wave 2B (ai_docs deep 11 files / ~19k); Wave 2C (ai_specs cross-cutting 33 files / ~29k; CC-1b → `CC-1.subA`); Wave 2D (ultramap deep 13 files / 16 Mermaid); Wave 2E (Obsidian sync 16 vault file changes + 14 repo files)
- Wave 3 verifier reports: agent-claim-verifier PASS-WITH-AMENDMENTS (20/20 hard); four-surface-persistence-verifier PARTIAL → addressed; na-gap-analyst Frame-A 11 NA gaps surfaced
- Wave 4.0: 5 decisions D-A..D-E surgically applied across 9 files (incl. GATE_STATE G4/B4 CLOSED)
- Wave 4.A: EscapeSurfaceProfile 6→7 (`PrivilegeEscalation` at ord 30); 12 file amendments + new ADR `2026-05-17-escape-surface-cardinality-7-privilege-escalation.md` + DECISION_REGISTER D-S1002127-02
- Wave 4.B: NA-GAP-01..11 substrate-as-primary remediation in flight (~10 new + 5 amended target)
- See [`CHANGELOG.md`](CHANGELOG.md) for full per-wave deltas. NO `.rs` files. NO Cargo.toml.

**In flight (CLAUDE.md §5.2 broadcast etiquette):**
- Wave 4.B `na-gap-analyst → substrate-as-primary remediation` agent — authoring `ai_specs/substrates/{atuin,stcortex,injection_db,synthex,lcm,conductor,watcher,operator}.md` + `cross-cutting/refusal-taxonomy.md` + `cross-cutting/substrate-drift.md`
- Zen G7 AMEND-loop: amended AUDIT-REQUEST v3 owed to Zen (citing D-S1002127-02 ADR + 12 affected files; ~`2026-05-17T<utc>_command_g7_audit_request_v3_cardinality_7_amendment.md`)
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

1. Conductor `devenv start weaver/zen/enforcer` (D-B3 — unblocks Phase 3 Track 2)
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
