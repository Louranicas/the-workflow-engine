# Workflow-Trace Hardening Fleet — 2026-05-21 (S1003529)

> **Canonical plan** for the end-to-end quality + security hardening of `the-workflow-engine`.
> **Authority:** Luke @ node 0.A — directive 2026-05-21: "develop the ultimate full end-to-end stack codebase hardening agentic team … significantly improve the quality of the codebase … security harden to the highest level … in synergy with Zen and the Zellij habitat."
> **Lead:** Command (Tab 1) · **Audit gate:** Zen (Tab 10) · **Fleet:** ALPHA/BETA/GAMMA + Agent-tool subagents.
> **Back to:** [CLAUDE.local.md](../CLAUDE.local.md) · [CLAUDE.md](../CLAUDE.md)

## Mission

Take `workflow-trace` (26 modules, ~31k LOC Rust, both binaries + `workflow_core` lib) to top-1%-of-humans
Rust quality and the highest achievable security posture. Verify every step; do not carry work forward.

## Baseline (2026-05-21, git `2026a72`)

| Dimension | State |
|-----------|-------|
| `cargo check` | clean (only vendored `spacetimedb-sdk` deprecation noise + 3 intentional `build.rs` warnings) |
| `clippy -D warnings` / pedantic | workflow-trace's own code clean — **but** 105 `#[allow(...)]` in `src/` may be suppressing lints |
| `unsafe` in `src/` | **0** |
| `todo!` / `unimplemented!` | **0** |
| Panic surface (`src/`, pre-test-strip) | ~33 unwrap · ~628 expect · ~59 panic · 2 unreachable — test-stripped census in flight |
| Tests | 1294 `#[test]` fns; `cargo test` count pending baseline |
| `cargo audit` | advisory DB malformed locally (`RUSTSEC-2026-0109.md`) — env fix needed before audit runs |

## atuin security tooling harvested

- `v8-audit` — 4-stage: cargo-audit CVEs · secrets scan · dep-freshness · unwrap/unsafe census (the working checklist)
- `v8-scan-security` — FP-calibrated prod-only unwrap/unsafe/secrets probe
- `pswarm-secure` — Prometheus-Swarm 3-agent audit pipeline (`:10002`)
- `fleet-task-loop` — Fleet ALPHA/BETA/GAMMA orchestration loop, re-queue-on-fail until convergence

## Waves (sequential; verified-complete before next; parallel subagents within each)

| Wave | Scope | Exit gate |
|------|-------|-----------|
| **W0** | Baseline · atuin harvest · this plan | baseline captured ✓ |
| **W1** | Quality floor — audit 105 `#[allow]`, remove unjustified suppressions + fix what they hid; test-gap closure | clippy+pedantic green with minimal justified allows; full gate green |
| **W2** | Security hardening — apply `v8-audit` checklist; security-auditor + silent-failure-hunter sweep; strip production unwrap/expect/panic; cargo-audit clean; deep-review KEYSTONE (m20-23), trust spine (m8-11), EscapeSurfaceProfile (m9/m30/m32) | 0 RISKY panic surfaces; 0 error-swallows; cargo-audit clean; auditor sign-off |
| **W3** | Type-design · comment accuracy · simplification (functionality-preserving) | type-design-analyzer + comment-analyzer pass |
| **W4** | Zen audit gate · `cargo-mutants` on KEYSTONE + trust modules | Zen verdict APPROVE; mutation kill-rate verified |
| **W5** | Docs reconciliation (stale planning-only language) · 4-surface persist · commit + push both remotes | clean tree; pushed; persisted |

## Fleet & agent model

- **Agent-tool subagents** = primary parallel workhorse (keeps Command's context in the sweet spot — agents return conclusions, not file dumps): `security-auditor`, `silent-failure-hunter`, `zen`, `code-reviewer`, `type-design-analyzer`, `code-simplifier`, `general-purpose` for cluster fixes.
- **Zen (Tab 10)** audits each landed wave via `~/projects/shared-context/agent-cross-talk/`.
- **Zellij Fleet ALPHA/BETA/GAMMA** kept in the loop via cross-talk; dispatched to if a wave needs more hands.

## Verification discipline (per wave)

1. Subagents do scoped work → 2. Command FP-verifies (re-run, re-grep, re-read — agent reports are evidence, not fact) → 3. Full quality gate (`check → clippy -D warnings → clippy pedantic → test`, `${PIPESTATUS[0]}` per stage) → 4. Zen audit → 5. mark wave task complete, advance.

## Continuity

In-session `/loop` self-pacing between waves; TaskList (#1–#6) = goal tracker; this doc + cross-talk = resume anchors. No `/cron` unless work crosses a session boundary (flagged before creation — autonomous billing).

## Results (live — 2026-05-21)

| Wave | Outcome | Commit | Tests |
|------|---------|--------|-------|
| W0 | Baseline + atuin security harvest + plan | — | baseline 1310 |
| W1 | Quality floor — all 26 modules to ≥50 meaningful tests | `dc25335` | 1310 → 1782 |
| W2 | 19 security findings resolved — KEYSTONE `project_after_prefix` correctness bug, 9 lock-poison panics, LIKE-injection, error-swallow, m9 namespace boundary, m8 false-gate docstrings, HTTP body caps | `c662b2d` + `5cb4822` | → 1834 |
| W3 | Type-design — `#[non_exhaustive]` ×24, `WorkflowId` + `MinSupport` encapsulation, 5 comment-accuracy fixes | `2e3113d` | → 1835 |
| W4 | `cargo-mutants` scoped to m10/m11/m21/m22. Committed baseline run (`mutants.out.old`, 2026-05-21T14:37–16:09Z): **319 mutants** — 240 caught / 20 missed / 20 timeout / 39 unviable (**85.7%** caught of 280 viable). 68 mutant-killing tests authored against the 20 missed mutants. The 20 m21 `build_variants` timeout mutants remain unscored. A post-fix verification run (S1003733) is in progress; verified kill-rate folded in on completion. *(Prior wording — "412 mutants, 80.6%" — did not reconcile with any committed artifact; corrected S1003733.)* | `5de71ac` | → 1903 |
| W5 | Docs reconciliation (CLAUDE.md charter + project & workspace CLAUDE.local.md) · 4-surface persistence · commit + push both remotes | (W5 commit) | 1903 |

Gate green every wave: `cargo check` + `clippy -D warnings` + `clippy -D clippy::pedantic` +
`cargo test --all-targets --all-features --release`. Zen audit packets W1/W2/W3 filed in
`~/projects/shared-context/agent-cross-talk/`. W1 incident (shared-tree parallel-agent file
reverts) was disclosed and fully reconciled. **Resolved S1003733** (assessment-driven
remediation): the **F2 m8-gate architecture decision** → KEEP-DORMANT (m8 retained as a
dormant tripwire; see `src/m8_povm_build_prereq/mod.rs` module doc); the **W3 #5–#10
core-type-encapsulation portfolio**
([HARDENING_W3_TYPE_DESIGN_PORTFOLIO.md](HARDENING_W3_TYPE_DESIGN_PORTFOLIO.md)) → completed
in remediation Wave C (6 representable-illegal-state holes closed).
