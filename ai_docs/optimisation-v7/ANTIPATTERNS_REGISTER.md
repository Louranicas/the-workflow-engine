---
title: ANTIPATTERNS REGISTER — workflow-trace V7
date: 2026-05-17 (S1001982)
kind: planning-only · operational catalogue
purpose: every known antipattern documented with detection + mitigation + source citation
count: 34 antipatterns across 5 classes
status: living doc — every new antipattern crystallised here
---

# ANTIPATTERNS REGISTER — workflow-trace V7

> Back to: [[TASK_LIST_V7_OPTIMISATION.md]] · [[KEYWORDS_20.md]] · [[ULTRAMAP.md]] · [[../../CLAUDE.md]]
>
> **Function:** every antipattern that could land in workflow-trace, named with its detection signal + mitigation + provenance. Loaded at session-start; consulted before any architectural decision. Class organisation: AP-Hab (habitat-wide) · AP-WT (workflow-trace-specific F1-F11) · AP-Drift (LCM Drift #1..#11) · AP-V7 (new this optimisation) · AP-Test (test-discipline antipatterns).

---

## Class AP-Hab — Habitat-wide antipatterns (apply to ALL services)

| ID | Antipattern | Detection signal | Mitigation | Source |
|---|---|---|---|---|
| AP-Hab-01 | **AP24 violation** — `cargo init` / `src/*.rs` before G9 signal | filesystem `src/` exists pre-G9; `git log --name-only --diff-filter=A` shows `.rs` | hard refusal; Watcher Class-F flag; rollback by `rm -rf src/ Cargo.toml` | MEMORY.md `feedback_wait_for_start_coding.md` |
| AP-Hab-02 | **AP27 violation** — Watcher self-modifies m46-m51 | git diff touches m46-m51 paths from Watcher agent | hard refusal at PreToolUse hook; AP27 is uncrossable | The Watcher.md |
| AP-Hab-03 | **AP30 violation** — stcortex write missing `workflow_trace_*` prefix | stcortex SELECT WHERE namespace NOT LIKE 'workflow\_trace\_%' returns rows from workflow-trace writes | m9 namespace guard at write boundary; refuse-write at DB layer | CLAUDE.md § Memory Systems #8 |
| AP-Hab-04 | **Preserve-list discipline failure** — `docker container prune -f` / `pkill -f` / `rm -rf` without enumeration | atuin search for blanket commands | enumerate before executing; named-exclusion does NOT propagate upstream of blanket; S102 lost openclaw-gateway this way | MEMORY.md `feedback_preserve_list_discipline.md` |
| AP-Hab-05 | **PIPESTATUS swallow** — `cargo … \| tail` captures tail's exit (always 0) | gate script prints PASS while clippy errored | `${PIPESTATUS[0]}` + explicit per-stage abort; never trust trailing exit code through pipe | MEMORY.md `feedback_pipestatus_for_gate_chains.md` |
| AP-Hab-06 | **`cp -f` alias trap** — alias `cp=cp -i` makes `cp -f` interactive | binary copy fails silently or prompts | always `/usr/bin/cp -f`; never bare `cp -f` | MEMORY.md `feedback_binary_deployment.md` |
| AP-Hab-07 | **Sandbox child reap** — `setsid`/`nohup`/`cargo run &` from agent | child PID gone within 10s of agent exit | Luke runs services from terminal; agent does NOT spawn | MEMORY.md `feedback_terminal_service_start.md` |
| AP-Hab-08 | **`git stash pop` wrong stash** — pre-existing stash overwrites uncommitted work | git status shows unexpected file deletions post-pop | `git stash list` BEFORE pop; `git stash show -p` to inspect; prefer WIP branch over stash | CLAUDE.md § Git Conventions |
| AP-Hab-09 | **TaskCreate reminder absorption** — using TaskCreate against PRIME DIRECTIVE | TaskCreate invocation in workflow-trace context | ignore reminder; markdown task list is the tracker | this project's CLAUDE.md PRIME DIRECTIVE |
| AP-Hab-10 | **Focus-yank Zellij navigation** — agent switches Zellij tab and returns <30s | Watcher tick logs tab-switch flash | dwell ≥30s on navigated tab; or stay in receive-mode | MEMORY.md `feedback_no_focus_yank.md` |
| AP-Hab-11 | **Hyphen-slug stcortex munge** — labels with hyphens fail pre_id/post_id slug encoding | stcortex pathway creation returns silent fail or wrong slug | hyphens → underscores in slugs; verify post-write | CLAUDE.local.md § Working Mode (S1001757 munge bug) |
| AP-Hab-12 | **Verify-dispatch silent suspended pane** — Zellij `write-chars` to suspended pane silently swallowed | dispatch reported success; dump-screen unchanged | always `dump-screen` verify after write-chars; use `write 13` (CR byte) not `write-chars '\n'` | MEMORY.md `feedback_verify_dispatch_landed.md` + `feedback_fleet_submit_cr_byte.md` |
| AP-Hab-13 | **Runbook probe freshness drift** — `Refreshed:DATE` stamp without re-executed probes | probe outputs stale vs claimed date | every refresh paired with `> probes re-verified live DATE` evidence | MEMORY.md `feedback_runbook_probe_freshness.md` |
| AP-Hab-14 | **God-tier dilution** — pre-existing project warnings excuse new code | new commits add clippy::pedantic without justification | new code must clear all 4 stages; pre-existing warnings ≠ excuse | MEMORY.md `feedback_god_tier_no_warnings_at_any_level.md` |
| AP-Hab-15 | **Flex-verify skip** — sibling/hunter/script claim shipped without re-run | commit message cites agent finding without orchestrator re-exercise | independently re-run gate + check git log -1 + exercise code paths | MEMORY.md `feedback_flex_verify_before_ship.md` |
| AP-Hab-16 | **Synthor stylistic substitution** — "use Hyper-Narrative Synthor" treated decoratively | response contains Resonance Certification Block, no `HyperNarrativeSynthor()` call | literal instantiation + real snapshots + real `export()`; template at `ASP_Peer_Workforce_Paper/run_synthor.py` | MEMORY.md `feedback_synthor_invocation_discipline.md` |

---

## Class AP-WT — Workflow-trace-specific F1-F11 failure modes (expanded)

| ID | Antipattern | Detection signal | Mitigation | Module owner |
|---|---|---|---|---|
| AP-WT-F1 | **Bank/name ossification** — m30 bank entries never sunset | stcortex query: m30 entries with `last_dispatched_at` > 120 days, status=Active | m11 hard sunset_at boundary (immutable at deploy time); Phase 6 D120 evaluation | m11 + m30 |
| AP-WT-F2 | **Sample-size inflation** — proposal shipped with n<20 | `ProposalBuilder::build()` returns `Err(SampleSizeBelowF2)` ignored | enforce at construction; Wilson 95% CI (NOT Wald); Option<Confidence>::None when n<20 | m14 + m20-m22 |
| AP-WT-F3 | **Substrate-input poisoning** — m1/m2/m3 read corrupted data → propagates downstream | m13 stcortex write rejected with namespace-collision error | m8 build-prereq + m9 namespace guard at write boundary | m8 + m9 |
| AP-WT-F4 | **Premature dispatch** — m32 dispatches without m33 verify | m32 dispatch_attempt with `verified_at` timestamp older than 7 days OR missing | m32 5-check pre-dispatch sequence; m33 TTL 7-day enforced | m32 + m33 |
| AP-WT-F5 | **Bank creep** — m30 admits workflow without explicit human accept | m30 admit_log entries with `accepted_by=auto` OR null | hard refusal; `wf-crystallise propose accept <id>` required; never auto-promote | m30 |
| AP-WT-F6 | **Self-dispatch** — measurement workflow dispatches its own measurement | m32 dispatch where workflow_id == measurement_target | m12 + m32 refusal check pre-dispatch | m12 + m32 |
| AP-WT-F7 | **CR-2 graceful-degrade pretend-fix** — POVM `learning_health` returns 0.0 instead of error | live `:8125/learning_health` returns 0.0 in healthy band [0.05,0.15] expectation | m8 build-prereq fails CI if `learning_health` reads outside band at startup | m8 |
| AP-WT-F8 | **Watcher feedback-loop poisoning** — Watcher observations feed back into iteration as evidence | m20-m22 inputs include Watcher source events | m2 narrowed-scope consumer (tool_call + consumption only — NOT Watcher events) | m9 + m2 |
| AP-WT-F9 | **Workflow-grain fitness distortion** — m7 row missing fitness_dimension defaults to mean | m7 SELECT WHERE fitness_dimension IS NULL OR =0.0 (when run had real outcome) | m7 schema `fitness_dimension REAL NOT NULL DEFAULT 0.0` + convention check at write | m7 |
| AP-WT-F10 | **Exploration-cost preservation collapse** — m6 baseline includes Converged outcomes, making exploration look expensive | m6 EMA computed over ALL sessions | spec: m6 20-session EMA EXCLUDES Converged outcomes | m6 |
| AP-WT-F11 | **Cascade monoculture** — m4 cluster IDs leak semantic content of pane labels | m4 cluster_id values contain ALPHA/BETA/GAMMA substrings | FNV-1a XOR derivation: `fnv1a_64(window_range) XOR fnv1a_64(sorted_pane_labels) XOR step_count`; semantic destroyed | m4 + m31 |

---

## Class AP-Drift — LCM Drift #1..#11 transposed to workflow-trace

| ID | Drift class (LCM origin) | Workflow-trace transposition | Catch point |
|---|---|---|---|
| AP-Drift-01 | Agent over-claim gate-clean against scoped clippy | Worktree agent reports "gate green" without `--all-features` re-run | Wave-end Command re-runs `--workspace --all-targets --all-features` |
| AP-Drift-02 | Agent fabricates commits | Agent reports commit SHA that doesn't exist in `git log` | `git log --oneline -1` independent verification before merge |
| AP-Drift-03 | Scaffold without binary-wiring | m20 PrefixSpan scaffolded but never called from m23 | integration test exercising m20 → m23 call chain |
| AP-Drift-04 | Test count over-report | Agent claims "120 tests" but `cargo test 2>&1 \| grep "running"` shows 80 | Wave-end `cargo test --no-run --message-format=json \| jq -r '.executable'` count |
| AP-Drift-05 | Migration "applied" but schema unchanged | m13 stcortex schema migration claimed applied; live `inspect` shows old | `stcortex inspect` schema diff before merge |
| AP-Drift-06 | Bridge contract drift | m40 SYNTHEX outbox-JSONL schema diverges from SYNTHEX `/v3/nexus/push` accepted format | `bridge-contract` skill run pre-merge |
| AP-Drift-07 | Soak metric over-stated | Phase 5C weekly synthesis cites m14 lift not reproducible | weekly Watcher independent recompute |
| AP-Drift-08 | Push state inconsistent across remotes | GitHub at SHA X, GitLab at SHA Y; agent reports "pushed" | `git ls-remote` both remotes before claim |
| AP-Drift-09 | PAT in committed file | rg `glpat-` OR `github_pat_` against current HEAD returns matches | pre-commit secret-scan hook (gitleaks or trufflehog) |
| AP-Drift-10 | Bind-address default 0.0.0.0 leak | health endpoint accessible from non-loopback | bind explicitly 127.0.0.1 by default; ServerConfig.bind_addr honored |
| AP-Drift-11 | Supervisor stub mistaken for live | `lcm-supervisor` binary missing but plan claims live | live-verified probe (smoke test) before marking RESOLVED |

---

## Class AP-V7 — New antipatterns surfaced by this optimisation

| ID | Antipattern | Detection signal | Mitigation | Crystallisation context |
|---|---|---|---|---|
| AP-V7-01 | **7-gen drift** — each generation introduces uncited claims not traceable to corpus | Gen-N → Gen-(N+1) diff has assertions without source-line citation | every gen ends with "Citation audit" section; each new claim cites a corpus location | this V7 optimisation |
| AP-V7-02 | **Ultramap rot** — Ultramap drifts from runbooks/module plans | Ultramap claims module X is in Cluster Y; cluster-Y.md doesn't list X | Ultramap drift register + per-gen consistency check | this V7 optimisation |
| AP-V7-03 | **Runbook command alias-trap** — runbook uses bare `cp`/`grep`/`find` that hits alias layer | atuin trajectory shows runbook command produced unexpected output | runbooks use `/usr/bin/cp`, `/usr/bin/grep -E`, `find . -name` (bypass alias) | CLAUDE.md § Environment Compatibility |
| AP-V7-04 | **Keyword overgrowth** — 20-keyword list silently grows past 20 | KEYWORDS_20.md table row count > 20 | hard cap at 20; replace before adding; document deprecations | this V7 KEYWORDS_20.md § Quick-use guide |
| AP-V7-05 | **Module-plan-to-src-drift** — module plan says LOC budget X; eventual src/ exceeds 2× | verify-sync invariant: `wc -l src/mN_*/**.rs` ≤ 2 × budget | per-Wave-end sync check; reject merge if drift | this V7 + ULTRAMAP View 2 |
| AP-V7-06 | **Bidi-anchor unidirectional rot** — doc A links to doc B; doc B's "Back to:" missing A | grep audit: every "Back to:" must list every linker | obsidian-vault-librarian Wave-end sweep | this V7 + bidi-anchor keyword |
| AP-V7-07 | **Worktree contamination** — `target/` symlinked between worktrees causes phantom rebuilds | atuin shows `ln -s ../wt-X/target` | per-worktree `target/` only; never symlink; `CARGO_TARGET_DIR=./target` per-shell | this V7 + worktree-mastery skill |
| AP-V7-08 | **Handshake dual-silence false-success** — C-2 + C-3 silent; Command proceeds as if both ack'd | handshake without ack-or-timeout record | 30-min ack window; silence → file escalation to Luke; do NOT proceed assuming ack | observed S1001982 (this session) — handshakes 11:45 + 11:57 silent |
| AP-V7-09 | **Substrate-frame engine confusion** — module designed assuming anthropocentric inputs (e.g., "user intent") | module spec uses "user wants X" without operationalised signal | every module spec includes operationalised input definition; Watcher Class-G flag if violated | this V7 + R6 partial-waiver mitigation |
| AP-V7-12 | **Operational-runbook word-count undershoot** — planning estimates for operational runbooks consistently undershoot by ~2× because drop-in command surface + per-step verification + failure-mode register + Watcher class pre-positioning compound | post-hoc audit shows estimate vs actual ratio >1.8 across runbook set | budget runbook word counts at 2× narrative estimate; for V7-style planning sprints, multiply runbook line items by 2 before total-budget check; if total over ceiling, dedup runbook commands against module plans (cross-reference rather than duplicate) | V7 closure 2026-05-17 — actual 112k vs estimated 88k (28% over driven entirely by runbooks 36k vs estimated 18k) |
| AP-V7-13 | **Health-200 ≠ behaviour-verified** — service `/health` returns 200 + version string but the live binary is serving stale code; spec/version mismatch undetectable from health endpoint alone | health probe shows version X + status healthy BUT semantic endpoint (e.g., `learning_health`, `/version` git SHA, feature toggle) reports values inconsistent with expected post-deploy state | always pair `/health` probe with at least one semantic-endpoint check that exercises the deployed-code-version (per CR-2 verification: `learning_health` ∈ [0.05, 0.15] band); pre-deploy gate must verify NEW binary serves NEW semantics, not just NEW health endpoint; AP-Hab-13 (runbook probe freshness) is the cousin pattern but at a different layer | 2026-05-17 live probe: POVM `:8125/health` returned 200 + service:povm_v2 v2.0.0 BUT `:8125/stats` showed `learning_health=0.9146` (pre-CR-2 inflated; CR-2 expected ~0.067). Source had CR-2 at `e2a8ed3`; live binary did not. Triggered m42 stcortex-only pivot per ADR `decisions/2026-05-17-m42-stcortex-only-pivot.md` |

---

## Class AP-Test — Test discipline antipatterns

| ID | Antipattern | Detection | Mitigation |
|---|---|---|---|
| AP-Test-01 | **Coverage theatre** — 50+ tests/module hit only happy path; mutation kills <50% | `cargo mutants` survives >50% mutations | mutation-test budget per module ≥70% kill rate |
| AP-Test-02 | **Property-test stub** — `proptest!` with single shrunk input | property-test runs <1000 iters by default | min 10,000 iters per property-test; CI-gated |
| AP-Test-03 | **Integration-test mock-leak** — integration test mocks the very thing it's integrating with | integration test mod imports `mockall` | integration tests use real local services or testcontainer; mock only true externals |
| AP-Test-04 | **Test isolation failure** — tests share mutable static state | parallel `cargo test` produces flaky results | per-test fixtures; `OnceCell` thread-local; no `static mut` |
| AP-Test-05 | **Assertion drift** — test asserts current behaviour without rationale | test failure post-refactor; nobody can explain expected | every assertion has a `// rationale:` comment OR references spec section |

---

## Detection-command catalogue (quick lookup)

```bash
# AP-Hab-01 — pre-G9 src/ check (gate at PreToolUse)
test -d src/ && echo "VIOLATION: src/ exists pre-G9" || echo "OK"

# AP-Hab-05 — PIPESTATUS gate verify
cargo check 2>&1 | tail -5
test "${PIPESTATUS[0]}" -eq 0 && echo "PASS" || echo "FAIL"

# AP-Hab-15 — flex-verify pre-merge
cargo test --workspace --all-targets --all-features 2>&1 | tee /tmp/wt-pre-merge-gate.log
test "${PIPESTATUS[0]}" -eq 0

# AP-Drift-04 — test count audit
cargo test --no-run --message-format=json 2>&1 | jq -r '.executable | select(.)' | wc -l

# AP-Drift-09 — secret scan
rg -n '(glpat-|github_pat_|gho_)' --hidden -g '!.git' .

# AP-V7-02 — Ultramap drift
diff <(rg -o 'm[0-9]+' ULTRAMAP.md | sort -u) <(rg -o 'm[0-9]+' MODULE_PLANS/cluster-*.md | sort -u)

# AP-V7-06 — bidi-anchor rot
for f in ai_docs/optimisation-v7/**/*.md; do
  for parent in $(rg -o 'Back to: \[\[([^]]+)\]\]' -r '$1' "$f"); do
    rg -l "$(basename "$f" .md)" "ai_docs/optimisation-v7/**/${parent}.md" || echo "ORPHAN: $f -> $parent"
  done
done

# AP-V7-08 — handshake dual-silence
find ~/projects/shared-context/agent-cross-talk/ -name "*handshake*" -mmin -60 | head -5
# If 0 results → handshake silent → file escalation, do NOT assume ack
```

---

## Watcher flag mapping (which class catches which antipattern)

| Watcher class | Catches antipatterns |
|---|---|
| A | AP-Hab-01 / AP-WT-F1-F11 first activation / AP-Drift-11 |
| B | AP-Hab-12 / AP-Drift-06 |
| C | AP-WT-F4 / AP-Drift-01 |
| D | AP-V7-02 / AP-V7-06 / AP-Drift-08 |
| E | (planning sprawl class — V7 itself if stalls past G9 + 14d) |
| F | AP-Hab-01 (the canonical AP24 violation) |
| G | AP-V7-09 (substrate-frame confusion) |
| H | AP-Hab-12 (Zellij silent swallow) / AP-Hab-10 (focus-yank) |
| I | AP-WT-F7 / Hebbian silence (currently firing live per tick·16) |

---

*ANTIPATTERNS REGISTER authored 2026-05-17 by Command. Living doc — new entries crystallise via append + drift-register at bottom.*
