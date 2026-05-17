---
title: SECURITY_SPEC — formal threat model + trust boundaries + EscapeSurfaceProfile + m32 5-check
date: 2026-05-17
status: SPEC
threats: [auto-promote attack, namespace drift, Conductor bypass, self-dispatch, substrate drift, F7 graceful-degrade, privilege escalation]
cardinality_amendment: "S1002127 — PrivilegeEscalation inserted at ordinal 30 (D-S1002127-02 ADR)"
---

# SECURITY_SPEC — workflow-trace

> **Back to:** [`INDEX.md`](INDEX.md) · [`MODULE_MATRIX.md`](MODULE_MATRIX.md) · [`../README.md`](../README.md) · [`../ARCHITECTURE.md`](../ARCHITECTURE.md) · [`../SECURITY.md`](../SECURITY.md) (operational surface)

## Threat model

workflow-trace's threat surface is **internal-only** — there is no public-facing HTTP API; no untrusted user-input ingestion path; no remote code execution surface. The threats are:

| # | Threat | Class | Mitigation |
|---|---|---|---|
| T1 | Auto-promote attack (m23 inserts into m30 without human review) | Logic | C14 + R7 (private `HumanAcceptanceSignature` constructor + `accepted_by` blocklist) |
| T2 | Namespace drift (write to non-`workflow_trace_*` prefix; collision with other services) | Logic | C2 + R1 (namespace constants + m9 assert) |
| T3 | Conductor bypass (m32 directly executes workflow) | Logic | C11 + no `exec_local` symbol + verify-sync #20 |
| T4 | Self-dispatch (workflow contains step that calls m32) | Logic | R10 (defense-in-depth at m30 + m32) |
| T5 | EscapeSurfaceProfile silent downgrade (reclassify Destructive → HostWrite to bypass m9) | Logic | R8 + R9 (admission rejects second-accept; declared >= derived) |
| T6 | F7 graceful-degrade pretend-fix (m8 emits warning instead of build-fail when prereq missing) | Operational | R4 (`compile_error!` hard-fail) |
| T7 | Substrate-drift attack (POVM serving pre-CR-2 binary; pre-ADR trap) | Operational | R20 (m42 POVM-decoupled; no POVM dependency to drift on) |
| T8 | Silent-swallow (`.ok()` discards Result on health/consent path) | Operational | `silent-swallow-detect` skill + clippy lints |
| T9 | Sycophancy override (m10 Held silently absorbed) | Logic | R3 (Held fails CI; no override until amendment) |
| T10 | Substrate-blind dispatch (m31 selects when substrate degraded) | Logic | R17 + m31 substrate-consultation invariant |
| T11 | PAT/secret leak in commits | Operational | per workspace CLAUDE.local.md Open Escalations E1-E7 protocol; scan source + vault + ai_docs together |

## Trust boundaries (deeper than SECURITY.md operational surface)

| # | Boundary | Modules | Defense layers | Failure if defense breaks |
|---|---|---|---|---|
| **B1** | L0 substrate (atuin / stcortex / injection.db) ↔ L1 readers (m1/m2/m3) | m1, m2, m3 | PRAGMA query_only=ON; cursor pagination; busy_timeout=5000ms; subprocess fallback (m1); reducer-callback dedup (m2); partition-on-resolved (m3) | unauthorised write to atuin/injection.db (catastrophic); silent consumer-id collision in stcortex |
| **B2** | Proposer (m23) ↔ Bank (m30) | m23, m30 | C14 + R7; `HumanAcceptanceSignature` private constructor; accepted_by blocklist; structural code-path absence | F5 bank creep; engine auto-evolves without human review |
| **B3** | Selector (m31) ↔ Dispatcher (m32) | m31, m32 | EscapeSurfaceProfile display-before-step (ordinal); m9 namespace-guard; substrate-consultation in m31 | operator dispatches destructive workflow unknowingly; substrate-blind dispatch |
| **B4** | Dispatcher (m32) ↔ HABITAT-CONDUCTOR | m32 | NEVER direct dispatch; 5-check sequence; AP-V7-08 self-dispatch refusal; Conductor breaker; typed errors | direct workflow exec bypasses audit; recursion trap |
| **B5** | Verifier (m33) ↔ 4-agent gate | m33 | unanimous PASS required (Zen + security-auditor + silent-failure-hunter + performance-engineer); 7-day TTL; tri-state verdict | verification theatre; stale verification accepted |
| **B6** | Substrate feedback (m40/m41/m42) ↔ SYNTHEX/LCM/stcortex | m40, m41, m42 | outbox-first JSONL; circuit breaker on 2 consecutive failures; Watcher Class-I log on OPEN; m41 calls `lcm.loop.create` (not `lcm.deploy`); m42 POVM-decoupled | silent learning loss (CC-5 dead); LCM-side substrate corruption |
| **B7** | Build-time prereq (m8) ↔ crate compile | m8 + all modules | `cargo:rustc-cfg=povm_calibrated` + `compile_error!` on missing env | F7 graceful-degrade ships degraded binary |
| **B8** | CI output (m12 sample-pack) ↔ Ember rubric (m10) | m10, m12 | 7-trait rubric over output samples; Held verdict fails CI; no sycophancy override | sycophancy verdicts ship; spec amendments slip |

## EscapeSurfaceProfile threat model

The 7-level ordinal `Sandboxed(0) < SandboxEscape(10) < ProcessMutate(20) < PrivilegeEscalation(30) < FileWrite(40) < NetworkEgress(50) < DataExfil(60)` is the destructiveness classifier consumed by m32 (banner display), m9 (namespace-write gate), and m33 (verification depth). Cardinality bumped from 6 to **7** per Luke S1002127 directive (`=7`); PrivilegeEscalation inserted at ordinal 30 between ProcessMutate (20) and FileWrite (40). See D-S1002127-02 ADR (`ai_docs/optimisation-v7/decisions/2026-05-17-escape-surface-cardinality-7-privilege-escalation.md`).

**PrivilegeEscalation (ordinal 30) — canonical definition (embed verbatim):**
> Capability gain or role elevation that grants the calling process new abilities beyond its pre-call state. Examples: invoking `sudo`; setuid/setgid; capability acquisition (`cap_set_proc`, `setcap`); ACL add; container privilege escalation (Docker `--privileged`, `cap-add`); SELinux/AppArmor profile escape. Distinguished from `ProcessMutate` (modifying another process WITHIN current privilege envelope) and `FileWrite` (which requires existing write permission but does NOT acquire new capabilities). Habitat-relevant: openclaw container UID-1337 escape, sudo gates, role elevations in nerve-center / Conductor.

**Per-level threat classes:**

| Level | Ord | Sample threats | m32 banner | m9 gate | m33 composition |
|---|---:|---|---|---|---|
| Sandboxed | 0 | none | "SANDBOXED" (green) | allow | majority APPROVE (≥3-of-4) |
| SandboxEscape | 10 | `sudo`, `setsid`, `nohup` in container-escape context | "SANDBOX-ESCAPE" (yellow) | allow with extra logging | 3-of-4 zero-REJECT |
| ProcessMutate | 20 | `kill`, `pkill`, `devenv stop`, signal-send within own privilege envelope | "PROCESS-MUTATE" (orange) | allow with operator confirm | 3-of-4 zero-REJECT |
| **PrivilegeEscalation** | **30** | `sudo`/`setuid`/`setgid`/`cap_set_proc`/`setcap`/ACL-add/Docker `--privileged`/`cap-add`/SELinux-AppArmor escape; openclaw UID-1337 escape; sudo gates; role elevations in nerve-center / Conductor | **"PRIVILEGE-ESCALATION!"** (red) | **DENY** by default; **only Conductor-routed dispatches with `HumanAcceptanceSignature.privilege_escalation_acknowledged = true`** | **3-of-4 zero-REJECT** (same gate as NetworkEgress + SandboxEscape + ProcessMutate) |
| FileWrite | 40 | `mkdir`, `mv`, `cp -f`, Edit (under existing write permission; no capability gain) | "FILE-WRITE" (orange) | allow with operator confirm | majority APPROVE (≥3-of-4) |
| NetworkEgress | 50 | `git push`, `ssh`, HTTP POST to non-allowlisted remote | "NETWORK-EGRESS!" (red) | allow with operator confirm + audit | 3-of-4 zero-REJECT |
| DataExfil | 60 | `rm -rf`, drop database, `git push --force`, `prune` | "DATA-EXFIL!" (red) | DENY by default; operator override required + audit | UNANIMOUS APPROVE-no-nits |

**Per-step classifier** (`m30::StepClassifier`) reads the step kind and returns the maximum escape-surface across all steps in a workflow. If declared profile < derived → `EscapeSurfaceInconsistent` (admission refused).

**Reorder/insert-variant risk** (formerly m30 Open Q1; now CLOSED by D-S1002127-02): inserting a new variant shifts ordinals; all `>=` comparisons across m32/m9/m33 need re-audit. **Mitigation IN FORCE:** numeric gap reservation (`Sandboxed = 0, SandboxEscape = 10, ProcessMutate = 20, PrivilegeEscalation = 30, FileWrite = 40, NetworkEgress = 50, DataExfil = 60`) — existing 6 variants keep their ordinal positions across the cardinality-6→7 bump; only the new variant occupies a previously-reserved gap. Any future variant must also occupy a reserved gap with explicit numeric value.

## m32 5-check pre-dispatch sequence as security gate

The 5-check sequence is the **principal security gate** at dispatch time:

| # | Check | Security property |
|---|---|---|
| 1 | Conductor `:8141/health` 200 | upstream service alive + breaker not OPEN |
| 2 | `m33.VerifyResult.ttl_expires_at > now_ms` | freshness — verification not stale |
| 3 | `definition_hash` matches m30 row | integrity — workflow definition unchanged since verification |
| 4 | `sunset_at > now_ms` | sunset honoured; expired workflows refused |
| 5 | `dispatch_cooldown` elapsed | rate-limiting; prevents fleet-wide dispatch storm |

All five MUST pass. Any check failure returns typed `DispatchError`. No soft-fail path; no `--force-check-N` flag (operator override exists only for cooldown via `--bypass-cooldown` with audit log).

## m9 namespace-guard defense-in-depth

m9 is the **single namespace-write gate**. Every write through m13 (which transports m42's substrate-feedback) eventually calls `m9::assert_namespace(id)`. The assert:

1. Reads `id.as_str()`.
2. Verifies `id.as_str().starts_with(workflow_core::namespace::WORKFLOW_TRACE_PREFIX)`.
3. Returns `Err(NamespaceError::PrefixMismatch { got: id })` on failure.

The `NamespaceId` newtype constructor (`pub fn from_const(c: &'static str) -> NamespaceId`) only accepts `&'static str` references to `workflow_core::namespace::*` constants. Runtime `String` cannot construct a `NamespaceId`; this is the structural prevention of dynamic namespace strings.

Defense-in-depth layering:
- **L1 type system** — `NamespaceId` constructor private to namespace module
- **L2 runtime assert** — `m9::assert_namespace` validates prefix
- **L3 SQLite CHECK constraint** — `accepted_workflows.id` column has CHECK on prefix (`migrations/0002_bank.sql`)
- **L4 Watcher monitor** — Class-D fires on any namespace drift in substrate observation

## Secrets discipline

- **No secrets in repo.** Use environment variables; reference in `config/*.toml.template` only.
- **PAT rotation on exposure.** Triple-location PAT leak is a habitat-wide pattern (S1001883 precedent); scan source + vault + ai_docs together.
- **`.git-credentials`** lives at `~/.git-credentials`; never copied into repo.
- **gh CLI device-flow re-auth** preferred over long-lived PATs.
- **`.claude/anti_patterns.json`** (TBD Wave 2) will include a pre-commit hook scanning for PAT prefixes (`glpat-*`, `ghp_*`, `github_pat_*`, `gho_*`).

## Disclosure protocol

Vulnerabilities, weaknesses, or near-misses:

1. File at `~/projects/shared-context/agent-cross-talk/YYYY-MM-DDThhmmssZ_security_<slug>.md`.
2. Notify Watcher via `~/.local/bin/watcher notify`.
3. Watcher Class-A flag fires for systemic threats.
4. Luke @ node 0.A receives WCP notice; CC-7 evolution loop runs if spec amendment needed.

## Pre-G9 security work (status)

- ✅ Scaffold-only waiver in `PRIME_DIRECTIVE_WAIVER.md` forbids `.rs` source files.
- ✅ Cluster D ships Day 1 (CC-2 trust layer woven first).
- ✅ EscapeSurfaceProfile schema spec'd in cluster-G spec.
- ✅ Formal threat model (this doc, T1-T11).
- ⏳ Bash-pattern block hooks in `.claude/hooks/` — TBD Wave 2.
- ⏳ `.claude/anti_patterns.json` machine-readable register — TBD Wave 2.

## Post-G9 security work

- m8 build-script gate FIRST commit (Day 1).
- m9 namespace guard SECOND.
- m10 Ember CI gate THIRD.
- Cluster A readers (CC-2 woven into every reader).
- Security audit (Zen + security-auditor agent) at end of Phase 1 (Day 3).
- m33 4-agent verifier wires Day 4 (depends on Conductor B3 resolution).

---

> **Back to:** [`INDEX.md`](INDEX.md) · [`../SECURITY.md`](../SECURITY.md) · [`CONSENT_SPEC.md`](CONSENT_SPEC.md)
