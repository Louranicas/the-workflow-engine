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
