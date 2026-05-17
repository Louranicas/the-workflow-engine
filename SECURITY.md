# SECURITY — workflow-trace

> **Status:** Pre-G9 (no executable code). Security posture is **spec-level** and **scaffold-level** until G9 fires.
> **Disclosure contact:** Luke @ node 0.A — direct via habitat channels.
> **Threat-model spec:** [`ai_specs/SECURITY_SPEC.md`](ai_specs/SECURITY_SPEC.md) (TBD Wave 2)
> **Antipattern register:** [`ai_docs/optimisation-v7/ANTIPATTERNS_REGISTER.md`](ai_docs/optimisation-v7/ANTIPATTERNS_REGISTER.md)

---

## Trust boundaries (architectural)

| Boundary | Modules involved | Defense |
|---|---|---|
| **L0 substrate (atuin / stcortex / injection.db)** ⇄ **L1 readers (m1/m2/m3)** | Read-only; cursor pagination; PRAGMA query_only; subprocess fallback for SQLite WAL lock | m1 5s busy-timeout; m2 reducer-callback dedup; m3 partition-on-resolved |
| **Proposer (m23)** ⇄ **Bank (m30)** | Operator review **mandatory** between m23 emit and m30 accept (CC-4) | AP-V7-07 refusal: NO auto-promote |
| **Selector (m31)** ⇄ **Dispatcher (m32)** | EscapeSurfaceProfile display-before-step (operator sees destructiveness band) | Ordinal enum (Sandboxed < SandboxEscape < ProcessMutate < FileWrite < NetworkEgress < DataExfil); m9 namespace guard |
| **Dispatcher (m32)** ⇄ **HABITAT-CONDUCTOR** | NEVER direct dispatch; always via Conductor; 5-check pre-dispatch sequence | m33 verifier 7-day TTL PASS; m9 namespace; EscapeSurfaceProfile; Conductor breaker; m32-self-dispatch refusal (AP-V7-08) |
| **Verifier (m33)** ⇄ **4-agent gate** | Zen + security-auditor + silent-failure-hunter + performance-engineer; unanimous PASS required | 7-day TTL on `last_verified_at`; PASS/FAIL/DEGRADED tri-state |
| **Substrate feedback (m40/m41/m42)** ⇄ **SYNTHEX/LCM/stcortex** | Outbox-first JSONL buffer; circuit breaker on 2 consecutive failures; Watcher Class-I log on breaker OPEN | m41 `lcm.loop.create` NOT `lcm.deploy`; m42 POVM-decoupled |

---

## Compile-time invariants (must hold)

| Invariant | Enforced by |
|---|---|
| **`povm_calibrated` cfg gate** | m8 build.rs `cargo:rustc-cfg=povm_calibrated` + `compile_error!`; env-only (cannot be bypassed by `--features full`) |
| **Newtype discipline** | SessionId, ConsumerId, NamespaceId — no raw `String` in domain APIs |
| **AP30 namespace constants** | `namespace.rs` constants only; never literal `"workflow_trace_*"` strings inline |
| **Zero `unwrap()` outside tests** | clippy gate + manual review |
| **Zero `unsafe`** | clippy gate + denied at crate level |
| **EscapeSurfaceProfile ordinal** | Enum derive(Ord); comparison via ordinal for gating |

---

## Runtime invariants (must hold)

| Invariant | Enforced by |
|---|---|
| **m9 namespace guard** | Asserts `workflow_trace_*` prefix on every stcortex write |
| **m10 Ember CI gate** | Watcher 7-trait rubric on every user-facing string |
| **m32 5-check pre-dispatch** | (1) m33 verifier PASS within TTL; (2) m9 namespace; (3) EscapeSurfaceProfile display; (4) Conductor reachable + breaker not OPEN; (5) self-dispatch check (m32 itself is not a workflow) |
| **Outbox-first JSONL** | m40/m41/m42 write to outbox before RPC — survives substrate outage |
| **Circuit breaker** | m40/m41/m42 open breaker on 2 consecutive failures; Watcher Class-I log |
| **3-band LTP/LTD gate (m13)** | substrate_LTP_density backpressure 0.15 threshold; deferred-write JSONL buffer when below |

---

## Secrets & PAT discipline

- **No secrets in repo.** Use environment variables; reference in `config/*.toml.template` only.
- **PATs rotate on exposure.** Reference [workspace CLAUDE.local.md Open Escalations E1-E7](../CLAUDE.local.md) for the S1001883 rotation precedent.
- **`.git-credentials` lives at `~/.git-credentials`** — never copied into repo.
- **gh CLI device-flow re-auth** preferred over long-lived PATs.
- **Triple-location PAT leak** is a Habitat-wide pattern (S1001883): scan source + vault + ai_docs together before any commit.

---

## Pre-G9 security work

- ✅ Scaffold-only waiver explicitly forbids `.rs` source files (see [`PRIME_DIRECTIVE_WAIVER.md`](PRIME_DIRECTIVE_WAIVER.md))
- ✅ Cluster D ships Day 1 (CC-2 trust layer woven first; phase-1 framework non-negotiable)
- ✅ EscapeSurfaceProfile schema spec'd in cluster-G spec
- ⏳ `ai_specs/SECURITY_SPEC.md` — TBD Wave 2 (formal threat model)
- ⏳ Bash-pattern block hooks in [`.claude/hooks/`](.claude/hooks/) — TBD Wave 2
- ⏳ `.claude/anti_patterns.json` machine-readable register — TBD Wave 2

---

## Post-G9 security work

- m8 build-script gate FIRST commit (Day 1)
- m9 namespace guard SECOND
- m10 Ember CI gate THIRD
- Then Cluster A readers (CC-2 trust woven into every reader)
- Security audit (Zen + security-auditor agent) at end of Phase 1 (Day 3)

---

## Disclosure

Vulnerabilities, weaknesses, or near-misses: file at `~/projects/shared-context/agent-cross-talk/YYYY-MM-DDThhmmssZ_security_<slug>.md` and notify Watcher via `~/.local/bin/watcher notify`. The Watcher is **the substrate-aware security observer** and will Class-A flag systemic threats.

---

> **Back to:** [`README.md`](README.md) · [`CLAUDE.md`](CLAUDE.md) · [`ARCHITECTURE.md`](ARCHITECTURE.md) · [`ANTIPATTERNS.md`](ANTIPATTERNS.md) · [`GOLD_STANDARDS.md`](GOLD_STANDARDS.md)
