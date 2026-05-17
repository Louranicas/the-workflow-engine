---
title: Phase 7 — Security + Compliance Hardening (CROSS-CUTTING)
kind: deployment-framework-recipe
status: planning-only · HOLD-v2 active · no code authorized
date: 2026-05-17 (S1001982)
emitter: Command (Tab 1 Orchestrator top-left) — security-auditor role
authority: Luke @ node 0.A
binding_spec: Genesis Prompt v1.2 (Zen-audit-locked); v1.3 patch pending
cross_cutting: true — runs in parallel with Phases 1-6, not after them
---

# Phase 7 — Security + Compliance Hardening (CROSS-CUTTING)

> Back to: [[../HOME]] · [[../GOD_TIER_CONSOLIDATION_S1001982]]
>
> Related: [[../module specs/cluster-D-trust-cross-cutting]] · [[../module specs/cluster-G-bank-select-dispatch-verify]] · [[../boilerplate modules/09-trap-verify-escape-skills/hookify.preserve-blanket-guard.local.md]] · [[../boilerplate modules/09-trap-verify-escape-skills/feedback_preserve_list_discipline]] · [[../Genesis Prompt v1.2 S1001982]]

---

## Framing

Phase 7 is not a final checklist run after deployment. It is a **parallel security lens** that fires specific gates at each of Phases 1-6. Every domain in this document maps to one or more modules (Clusters A-H), one or more phase-gate checkpoints (T1-T6), and one or more Watcher flag classes (A-I).

The engine is a CLI tool, not an HTTP service. Its attack surface is correspondingly narrow — no inbound network listener, no credential processing, no multi-user boundary. The security posture flows from that single-user constraint into practical, actionable controls. Where formally optional (no SOC2 obligation, no GDPR PII), defensive practices apply anyway because habits formed here propagate to future services, and because Luke's habitat substrate already stores sensitive-adjacent data (atuin shell history, stcortex observations, causal chains).

**Five explicit risk waivers ride on Command's head** per the single-phase override (S1001982). The security audit verifies that mitigations are adequate for the accepted risk class of each waiver. Where mitigations are thin, this document names the residual risk explicitly.

---

## Cross-Cutting Trust Architecture Overview

Before the seven domains, four architectural trust structures require first-class treatment because they are security properties embedded in the module design itself.

### W1 — Watcher Narrowed Consumer Scope = Trust Boundary

m2 (`stcortex_consumer_reader`) registers as a stcortex consumer scoped to `tool_call` and `consumption` tables only. It reads context — what tools the user invoked, what was consumed — but never reads memory or pathway tables belonging to other consumers. This is F8 mitigation (Watcher feedback-loop poisoning).

The stcortex DB layer enforces refuse-write for namespaces without a fresh registered consumer. m2's read scope is a narrowing of what the consumer registration permits, enforced by convention (m9) rather than by the DB layer. This is adequate for a single-user single-binary tool; in a multi-user deployment it would require DB-layer column-level ACLs, which stcortex does not currently expose. The single-user constraint must be documented explicitly as a deployment prerequisite (see Domain 6, Data Privacy).

**Security implication:** if a future version of m2 inadvertently adds a read from another consumer's memory table, there is no DB-layer hard stop — only the m9 write guard and the W1 architectural discipline. Any PR touching m2 must include a reviewer check that the stcortex query SELECT list does not expand beyond `tool_call` and `consumption` tables.

### AP30 — Namespace Prefix = Privilege Boundary in stcortex

All stcortex writes from workflow-trace use the `workflow_trace_*` prefix. This is not stylistic — it is the application-layer privilege boundary. ORAC writes to `orac_*`, Pane-Vortex to `pane_vortex_*`, and so on. The namespace IS the separation boundary. AP30 collision avoidance prevents a workflow-trace Hebbian pathway reinforce from accidentally over-writing ORAC's P01..P16 fitness pathway weights, which would corrupt RALPH evolution.

CWE-284 (Improper Access Control) is the relevant class. The mitigation is m9's `assert_workflow_trace_namespace()` guard, which fires as a runtime assertion before every reducer call.

### m9 — Namespace Guard as Runtime Write Gate

`assert_workflow_trace_namespace(namespace)` blocks any stcortex write where the namespace does not start with `workflow_trace`. The function emits a `tracing::error!` with structured fields and returns `WorkflowError::NamespaceViolation` before the SpacetimeDB reducer is called. The DB layer provides a second enforcement layer (refuse-write without fresh consumer), but m9 provides an earlier, more legible error.

**Gap acknowledged:** m9 is a write gate; it does not gate reads. An m2 implementation bug could read from a foreign namespace without m9 catching it. The only controls on read scope are architectural discipline and code review.

### m10 — Ember Gate as Output Security

Every user-facing string in the workflow-trace CLI is scored against the Watcher's 7-trait Ember rubric in `tests/ember_gate.rs`. Traits 4 (Honesty) and 5 (Investment) directly address inadvertent information disclosure: a string that claims false certainty about substrate state, or one that leaks the internal structure of a workspace credential path, would fail Honesty or Investment respectively. Trait 7 (Warmth) prevents strings that attempt social engineering of Luke by proposing irreversible actions without explicit Luke ratification.

This is OWASP A05:2021 (Security Misconfiguration) at the output layer — the gate prevents the CLI from emitting text that misleads the operator about system state or nudges toward unsafe action.

**Current status:** Ember §5.1 Held-semantics amendment is pending Watcher review. Until it lands, Held verdicts are CI failures (stricter than rubric default). This is the correct posture for a security gate.

### Cipher Escape-Surface — Explicit Privilege Declaration

Cipher's `EscapeSurfaceProfile` ordinal enum (`ReadOnly < HostWrite < Network < SandboxEscape < Destructive`) gives every accepted workflow an explicit privilege declaration. This is OWASP A04:2021 (Insecure Design) prevention — the insecure design would be a system where destructive operations can be dispatched without the operator knowing their privilege class. m32's display-before-step gate makes the escape surface mandatory-visible output before the Conductor receives the request.

The ordinal ordering is security-relevant: if a workflow declares `HostWrite` but one of its steps is actually `Network`, m33's mechanical gate catches the inconsistency at verification time (before dispatch, not at execution time).

### Substrate Condition Risk

LTP/LTD = 0.043 (LTD-dominant; 35× below healthy Hebbian ratio). CR-2 fixed the measurement, not the substrate condition. If the engine ships Cluster H feedback (m40/m41/m42 reinforcement) onto a degraded substrate, the learning loop adds new pathway writes to an LTD-dominant regime. The risk is not data corruption — the namespace guard and DB refuse-write prevent foreign writes. The risk is that Hebbian reinforcement signals from m42 are immediately decayed by the substrate's LTD-dominant regime, making the learning loop decorative. Watcher Class-I flag is pre-positioned for this failure mode.

---

## Domain 1 — Supply Chain Security

### Threat Model

- **Attack surface:** the `Cargo.lock` file and crate registry. A compromised or vulnerable crate in the transitive dependency tree could introduce memory safety bugs (Rust memory model provides significant protection but not against logical vulnerabilities), arbitrary code execution via build scripts, or cryptographic weaknesses.
- **Who could exploit:** a supply chain attacker (SLSA threat model: malicious maintainer, compromised registry, malicious build); or an automated vulnerability scanner that finds a known-bad crate version before the maintainer does.
- **Harm class:** remote code execution (if build script is compromised), data exfiltration (if a CLI dependency phones home), or silent behavioral corruption (if a cryptographic crate has a known weakness used in m7's FNV-1a or m33's definition_hash).

### Mitigations in workflow-trace

- **Cargo.lock committed** (P0 constraint, binding spec). Pinned lockfile means no silent dependency upgrades between runs. This is the single most important supply chain control for a Rust project.
- **FNV-1a hash usage is for integrity detection, not cryptographic purposes.** m33 uses FNV-1a to detect definition drift (CWE-354, Improper Validation of Integrity Check Value). FNV-1a is not collision-resistant for adversarial inputs — it is a fast non-cryptographic hash. Because m33's definition_hash is compared only against the hash stored at verification time (not against an adversary-provided value), this is acceptable. If a future version of workflow-trace compares definition hashes across trust boundaries, SHA-256 (from the `sha2` crate) must replace FNV-1a.
- **`forbid(unsafe_code)`** in every module (P0 constraint). Eliminates the surface area where a compromised crate could inject unsafe behavior through an FFI boundary.

### Specific Commands

Run before every release tag and in CI on every PR that touches `Cargo.toml` or `Cargo.lock`:

```bash
# Audit against RUSTSEC advisory database
cargo audit

# Deny list: known-vulnerable categories to block at CI
# Place in deny.toml at workspace root (from cargo-deny)
cargo deny check advisories

# Verify Cargo.lock is committed and unmodified
git diff --exit-code Cargo.lock

# Inspect build scripts in the dependency tree (build.rs files can execute arbitrary code)
cargo metadata --format-version 1 | jq '[.packages[] | select(.build_script != null) | .name]'

# Check for crates pulling in network access in build scripts (manual review required)
cargo metadata --format-version 1 | jq '[.packages[] | select(.build_script != null) | {name, build_script}]'
```

A `deny.toml` at workspace root should include:

```toml
[advisories]
vulnerability = "deny"
unmaintained = "warn"
yanked = "deny"
notice = "warn"

[licenses]
allow = ["MIT", "Apache-2.0", "ISC", "BSD-2-Clause", "BSD-3-Clause", "Unicode-DFS-2016"]
deny = ["GPL-2.0", "AGPL-3.0"]  # Copyleft may conflict with habitat redistribution

[bans]
# Deny any crate with a known CVE-affecting-version in the build
multiple-versions = "warn"
```

Relevant RUSTSEC advisories to monitor for the expected dependency profile: any advisory touching `rusqlite`, `tokio`, `serde_json`, `thiserror`, or `tracing` — these are the highest-probability dependencies based on the boilerplate ancestry.

### Integration with Phases 0-6

| Phase | Gate fires |
|---|---|
| Phase 0 (G5 spec interview) | Confirm `cargo audit` is in the CI pipeline spec before G7 Zen audit |
| Phase 1 (T1 Specify) | `deny.toml` authored as part of workspace scaffold |
| Phase 3 (T3 Test) | `cargo audit` + `cargo deny check` run in CI alongside `cargo test` |
| Phase 5 (T5 Validate) | Final supply chain audit before promotion from shadow to live |

### Watcher Flag Class

Class F (AP24 violation adjacency) if any dependency changes land without a corresponding `cargo audit` pass. Substrate-frame relevant: Class G if a build script in the transitive closure touches stcortex or habitat services at build time.

### Failure Mode

A vulnerable crate version is shipped. The crate is later flagged in RUSTSEC. Because `Cargo.lock` is committed, the vulnerability is deterministically reproducible and auditable. The mitigation is a `cargo audit` run in CI that fails the build when a RUSTSEC advisory matches any crate in the lockfile.

### Acceptance Criteria

- `cargo audit` returns exit code 0 with zero vulnerability findings
- `cargo deny check` passes with the `deny.toml` policy in place
- `Cargo.lock` is committed and tracked in git
- `cargo metadata` build-script audit shows no unexpected network-calling build scripts
- CI pipeline includes both checks on every PR

---

## Domain 2 — Authentication + Authorisation

### Threat Model

workflow-trace is a CLI, not a service. There is no inbound network listener (P0 hard refusal on HTTP server in `wf-dispatch`). The authentication surface is confined to the three outbound wire calls in Cluster H and the stcortex consumer registration in m2.

- **SYNTHEX v2 :8092 (m40):** no auth on the inbound NexusEvent endpoint. A local-only service binding (loopback). m40 must NOT include user credentials, PAT fragments, or atuin command content in the `NexusEvent.data` field. The risk is information leakage in the event payload, not authentication bypass.
- **LCM :8141 UDS (m41):** Unix Domain Socket. The permission model is filesystem permissions on the socket file. If the UDS is world-writable, any local user (or a compromised subprocess) could inject arbitrary LCM requests. This is OWASP A01:2021 (Broken Access Control) at the IPC layer.
- **POVM :8125 (m42):** pre-cutover HTTP endpoint, loopback only, no auth. AP30 namespace prefix prevents privilege collision. The threat is namespace confusion, not authentication bypass.
- **stcortex :3000 (m2 + m13):** consumer registration is the auth surface. The stcortex DB-layer refuse-write enforces that only registered consumers can write to their namespace. m13 must register as a fresh consumer before any write is accepted — this is the database-layer authentication primitive.

### Mitigations in workflow-trace

- m40 event payload construction must exclude any field that could contain a PAT, file path with credential content, or raw atuin command string. The `NexusEvent.data` field type must be a structured enum (not `serde_json::Value` open bag) so that the compiler prevents accidental credential embedding.
- m41 UDS: the socket file at the LCM address must be verified to be `0600` (owner-only read-write) before m41 attempts a connection. If the socket is world-readable, m41 logs a warning and proceeds (degraded security) with a `tracing::warn!` that names the socket path and its current permissions. This follows the fail-open posture appropriate for a single-user habitat tool — blocking is too disruptive; alerting is correct.
- m42 namespace prefix enforcement (m9 gate) is the primary auth control. No foreign namespace writes are possible without m9 producing a `NamespaceViolation` error.
- stcortex consumer registration: m2 uses the consumer onboarding documented in `02-stcortex-consumer/CONSUMER-ONBOARDING.md`. A stale consumer registration causes subsequent writes to be refused at the DB layer. The `wf-crystallise` binary startup sequence must include a consumer registration heartbeat before any substrate write is attempted.

### Specific Commands

```bash
# Verify LCM UDS socket permissions before each dispatch session
stat -c "%a %U %G %n" /path/to/lcm.sock  # expect: 600 <user> <group>

# Confirm stcortex consumer is fresh before write operations
curl -s http://127.0.0.1:3000/consumers | jq '.[] | select(.namespace | startswith("workflow_trace"))'

# Check NexusEvent payload does not contain PAT-pattern strings before emission (grep audit)
rg --type rust 'NexusEvent.*data.*=.*env\|PAT\|TOKEN\|glpat\|github_pat' src/
```

### Integration with Phases 0-6

| Phase | Gate fires |
|---|---|
| Phase 1 (T1 Specify) | m40 event payload field type locked to structured enum |
| Phase 2 (T2 Build) | m41 UDS permission check implemented in m41 startup |
| Phase 3 (T3 Test) | Test that `NexusEvent` type rejects open-bag `Value` fields |
| Phase 4 (T4 Deploy) | Consumer registration verified live before shadow phase |

### Watcher Flag Class

Class B (handoff boundary crossing) if m40 emits a NexusEvent to SYNTHEX v2 with data fields that expand beyond the structured enum. Class I (Hebbian silence) if stcortex consumer registration fails silently and m13 writes are refused without alerting.

### Failure Mode

m40 includes a raw atuin command string in `NexusEvent.data`. SYNTHEX v2 stores it in its Nexus event log. The atuin command contained a PAT fragment (e.g., the user ran `git push https://user:TOKEN@repo`). The event log is now a credential store. Mitigation: structured enum type prevents this at compile time.

### Acceptance Criteria

- `NexusEvent.data` is a `pub enum NexusEventData` (not `serde_json::Value`); PR reviewer confirms no `Value` wrapping
- m41 UDS permission check logs `tracing::warn!` when socket is not `0600`; test confirms warning fires
- stcortex consumer registration is verified at binary startup; test confirms startup fails-loudly if registration is refused
- `cargo audit` clean on all Cluster H dependencies (`tokio`, `rusqlite`, `serde_json`)

---

## Domain 3 — Secret Handling

### Threat Model

workflow-trace reads atuin shell history (m1), stcortex tool_call + consumption tables (m2), and habitat injection chains (m3). These sources contain sensitive-adjacent data:

- atuin history rows may contain command-line arguments with embedded credentials (e.g., `curl -H "Authorization: Bearer TOKEN"`, `git push https://user:TOKEN@repo`)
- stcortex tool_call records may contain file paths to credential files that were read during a Claude session
- injection.db causal chains may contain resolved command templates that include environment variable values

The engine must not:
1. Log or persist raw atuin command content (CWE-532, Insertion of Sensitive Information into Log File)
2. Exfiltrate stcortex memories outside the `workflow_trace_*` namespace
3. Embed credential-pattern strings in NexusEvent payloads (Domain 2)
4. Persist GitHub/GitLab PATs from atuin history into the workflow bank (m30)

Additionally: the S1001883 Bug Hunter Armada found PAT strings in three locations across the habitat. The lesson is that PAT-pattern strings propagate via copy-paste into planning docs, build scripts, and anywhere command output is recorded verbatim. workflow-trace, as a tool that processes command history, is a novel vector for this pattern.

### Mitigations in workflow-trace

- **m1 atuin ingestion:** step 1 of m1's ingest pipeline must apply a PAT-pattern redaction pass before any command string is written to m7's correlation tables. The redaction patterns to cover (based on S1001883 findings):

  - `glpat-[A-Za-z0-9_-]{20,}` — GitLab PATs
  - `github_pat_[A-Za-z0-9_]{36,}` — GitHub fine-grained PATs
  - `ghp_[A-Za-z0-9]{36}` — GitHub legacy PATs
  - `gho_[A-Za-z0-9]{36}` — GitHub OAuth tokens
  - `Authorization: Bearer [A-Za-z0-9._-]{20,}` — generic Bearer tokens in HTTP headers
  - `:[A-Za-z0-9_-]{20,}@` — credentials embedded in Git remote URLs

  These patterns must be applied as an explicit allowlist-based sanitisation: rather than trying to detect all secret patterns (CWE-312 is broad), m1 replaces the matched substring with `[REDACTED:<pattern-class>]` and emits a `tracing::info!` event noting that redaction fired on a row (without logging the original). The redaction is applied to the `argv` field only, not to timing metadata.

- **m2 stcortex reader:** m2 must read only `tool_call` and `consumption` tables (W1 scope). It must NOT select columns that contain file contents or full argument strings from tool calls that may have read credential files. The SELECT list is hardened: only `tool_call_id`, `tool_name`, `timestamp`, `session_id` — no argument blobs.

- **m30 workflow bank:** the `curator_note` field is a human-authored string at accept time. Any curator note that matches a PAT pattern must be rejected at `BankDb::accept()` with a `WorkflowError::SecretInCuratorNote` error variant. This is a belt-and-suspenders check — the curator is Luke; but automated proposal pipelines could generate notes containing command fragments.

- **No PAT in Cargo.toml or .env files:** any `.env` file in the workflow-trace workspace must be in `.gitignore`. The build system must not require environment variables containing credentials to function (the `POVM_CR2_DEPLOYED=1` env var in m8 is a binary marker, not a secret).

### Specific Commands

```bash
# Grep for PAT-pattern strings in the workspace before committing
rg --type rust 'glpat-|github_pat_|ghp_[A-Za-z0-9]{36}|gho_[A-Za-z0-9]{36}' src/ tests/

# Confirm .env is gitignored
grep -n '\.env' .gitignore

# Verify no PAT strings in Cargo.toml or build.rs
rg 'glpat-|github_pat_|ghp_|gho_' Cargo.toml build.rs 2>/dev/null && echo "FAIL: credentials found" || echo "OK"

# Audit m1 ingest code for raw atuin arg logging before redaction pass
rg --type rust 'argv\|command\|history' src/m1_atuin_ingest/ | grep -v 'redact\|REDACTED'
```

RUSTSEC advisory to monitor: any advisory for crates that handle string pattern matching (`regex` crate) or string sanitisation that workflow-trace depends on indirectly.

### Integration with Phases 0-6

| Phase | Gate fires |
|---|---|
| Phase 1 (T1 Specify) | PAT redaction patterns specified in m1 module spec |
| Phase 2 (T2 Build) | m1 redaction pass implemented; m2 SELECT list hardened |
| Phase 3 (T3 Test) | Test that atuin rows containing PAT-pattern strings are redacted; test that m30::accept rejects PAT-containing curator notes |
| Phase 5 (T5 Validate) | Pre-release grep audit via commands above |

### Watcher Flag Class

Class D (four-surface drift) if PAT-pattern strings appear in any of the four persistence surfaces (m7 SQLite, stcortex pathways, m30 bank, m15 JSONL ledger). Class F (AP24 violation adjacency) if any commit introduces a PAT string without the above grep audit having been run.

### Failure Mode

m1 ingests an atuin row containing `git push https://user:glpat-abcd@gitlab.com/repo.git`. The PAT passes through without redaction. m7 stores it in `workflow_arc_record.command_fragment`. A later m12 report surfaces it to stdout. A third party with access to the terminal session log now has a valid PAT. Mitigation: pattern-matched redaction in m1 before any write to m7.

### Acceptance Criteria

- m1 redaction unit tests: 100% of listed PAT patterns are caught and replaced with `[REDACTED:<class>]`
- m2 SELECT list contains no `args`, `argv`, `content`, or `value` columns from tool_call table
- `BankDb::accept()` test: curator note with embedded PAT returns `SecretInCuratorNote` error
- Pre-release grep audit returns zero matches for PAT patterns in source and SQLite migration files

---

## Domain 4 — Sandboxing / Privilege Containment

### Threat Model

workflow-trace runs as the user (Luke, UID `louranicas`). No setuid, no container escape, no daemon. The privilege containment concern is not escalation but **lateral movement** within the habitat: a dispatched workflow that was declared `ReadOnly` at acceptance time but actually executes a `HostWrite` or `Network` step. This is CWE-284 (Improper Access Control) at the workflow execution layer.

The second concern is the **blanket-command surface**: if the engine generates workflow steps containing `rm -rf`, `docker container prune`, `pkill -f`, or equivalent, the preserve-list discipline (Domain 5) must catch them. But at the sandbox level, the question is whether m32 could be tricked into dispatching a workflow where the Conductor-executed step escapes the habitat boundary (e.g., by SSHing to a remote host, or by running a Docker exec into the openclaw-gateway container without Cipher's knowledge).

- **SandboxEscape profile:** any workflow that touches `docker exec`, `ssh`, `kubectl exec`, or similar subprocess invocations that cross the container/VM boundary must carry `EscapeSurfaceProfile::SandboxEscape`. Watcher witness is required at dispatch for this profile class.
- **Destructive profile:** any workflow that touches `rm`, `trash`, `prune`, `--force` on container or volume operations must carry `EscapeSurfaceProfile::Destructive`. Explicit Luke confirmation at m32's display-before-step gate is required.

### Mitigations in workflow-trace

- `wf-dispatch` binary has a hard refusal on any code path that calls `tokio::process::Command` on a workflow step (P0 #3). m32 routes exclusively to HABITAT-CONDUCTOR. CONDUCTOR is the sole execution authority and carries its own privilege model.
- m33 `workflow_verifier` mechanical gate validates that the declared `escape_surface_profile` in the `AcceptedWorkflow` is consistent with the step kinds in `steps_json`. A step of kind `docker_exec` in a workflow declared as `ReadOnly` is a mechanical gate failure at verification time.
- m32's `display_before_step` outputs the escape surface banner before the Conductor request is sent. The banner for `SandboxEscape` reads: `[SANDBOX-ESCAPE] This step may exit the sandbox boundary. Verify intent.` This is a hard stdout requirement, not a log line.
- The step-kind registry in `workflow-core` (shared library) maintains an allowlist of known step kinds and their minimum escape surface requirement. An unknown step kind causes m33's mechanical gate to return `VerificationVerdict::Fail` (unknown step type = unverified surface = blocked).

### Specific Commands

```bash
# Verify wf-dispatch binary contains no direct process::Command calls on step execution
rg --type rust 'tokio::process::Command\|std::process::Command' src/m32_dispatcher.rs
# Expected output: zero matches

# Verify step-kind registry covers all expected kinds with declared surfaces
cat src/workflow_core/step_registry.rs  # review for completeness and surface correctness

# After dispatch, verify Conductor audit log shows correct operator=wf-dispatch/human
sqlite3 ~/.local/share/workflow-trace/dispatch_log.db \
  "SELECT dispatch_id, workflow_id, operator, escape_surface FROM dispatch_audit ORDER BY dispatched_at DESC LIMIT 5;"
```

### Integration with Phases 0-6

| Phase | Gate fires |
|---|---|
| Phase 1 (T1 Specify) | Step-kind registry specified; escape surface ordinal locked |
| Phase 2 (T2 Build) | m33 mechanical gate implements escape surface consistency check |
| Phase 3 (T3 Test) | Test: `docker_exec` step in `ReadOnly` workflow fails m33 mechanical gate |
| Phase 4 (T4 Deploy) | Watcher witness confirmed available for SandboxEscape-profile workflows |
| Phase 5 (T5 Validate) | Dispatch audit log reviewed for any `operator` field != `wf-dispatch/human` |

### Watcher Flag Class

Class A (activation transition) if any `SandboxEscape` workflow is dispatched — Watcher witnesses and timestamps verbatim. Class G (substrate-frame confusion) if a workflow step writes to stcortex tables outside the `workflow_trace_*` namespace via Conductor execution.

### Failure Mode

m33's mechanical gate has a bug in the escape surface consistency check: it reads `workflow.escape_surface_profile` but does not iterate `steps_json` to validate per-step surfaces. A workflow declared `ReadOnly` with one `docker_exec` step passes verification. m32 dispatches it. Conductor executes the docker exec. The step escapes the sandbox. Mitigation: m33 mechanical gate test `docker_exec_in_readonly_fails` is a required test in the 55-test coverage target.

### Acceptance Criteria

- m32 contains zero `tokio::process::Command` calls on step data; confirmed by `rg` audit
- m33 mechanical gate test: every known step kind with a higher surface than declared workflow surface fails verification
- `display_before_step` integration test confirms banner text is printed to stdout (not logged) for each surface level
- dispatch_audit `operator` field is always `wf-dispatch/human`; test asserts this

---

## Domain 5 — S102 Preserve-List Discipline

### Threat Model

The S102 openclaw-gateway incident is the origin scar. `docker container prune -f` removed a named container despite an earlier named-exclusion being in force, because named exclusions do not propagate through blanket-scope operations. The structural pattern is: **safety rules enforced per-call-site dissolve under `all` / `--all` / `prune` / `-rf *` operations.**

For workflow-trace, the threat is that the engine generates and then dispatches a workflow containing a blanket step — for example, a workflow that emerged from observing a user run `cargo clean --all-targets` followed by `cargo build`. The engine canonicalises this sequence. The next time the workflow is dispatched, Conductor executes `cargo clean --all-targets` on the current working directory, which may have a different scope than the session where it was originally observed. If the user's preserve list has changed, the blanket wipes what the new preserve list was intended to protect.

This is not a theoretical concern. The boilerplate source `feedback_preserve_list_discipline.md` documents the exact mechanism, and the `hookify.preserve-blanket-guard.local.md` hook was built specifically to intercept this pattern class at the bash-tool level. workflow-trace must apply the same discipline at the workflow-generation and workflow-dispatch levels.

### Mitigations in workflow-trace

- **m33 Security agent (one of the four parallel verification agents)** is responsible for blanket-command detection. Its analysis of `AcceptedWorkflow::steps_json` must flag any step where the `conductor_params` contain the blanket signatures:

  Blanket signatures to flag: `prune`, `--all-targets` on destructive ops, `rm -rf` with glob expansion, `pkill -f`, `docker system prune`, `cargo clean --all-targets`, `git clean -fd`, `trash-empty`, `docker rm $(...)`.

  A step containing a blanket signature triggers a `VerificationAgent::Security` REJECT unless the `TrapAnnotation` for that step explicitly documents the mitigation (e.g., explicit enumeration of targets was verified at proposal time, or the workflow carries `EscapeSurfaceProfile::Destructive` AND has a Luke-confirmation annotation).

- **m30 EscapeSurfaceProfile::Destructive** is the mandatory profile for any workflow containing a blanket step. There is no lower profile that is acceptable — a blanket `rm -rf *` with a `HostWrite` profile declaration is a m33 mechanical gate failure (escape surface inconsistency).

- **m32 display-before-step** for `Destructive` profile includes the additional line: `[DESTRUCTIVE] This step runs a potentially irreversible operation. Enumerate targets before proceeding.` This matches the S102 discipline verbatim: before executing a blanket command, enumerate, diff against preserve list, name explicitly.

- **m23 `workflow_proposer`** (Cluster F) must not auto-promote a workflow containing a blanket step without attaching a deviation evidence note explaining why the blanket scope is correct. m23's `ProposalBuilder` must reject any step containing the blanket signatures unless `deviation_evidence` contains at least one `DeviationNote` with `kind: BlanketScope` and a non-empty `justification` field. This gate fires at proposal time (before human review), not at dispatch time.

- If `Destructive` profile AND blanket signature are detected in m33's Security agent review: the verdict is REJECT unconditionally. No mitigation annotation overrides a Destructive + blanket combination — only Luke's explicit manual override at the CLI level (`wf-dispatch --override-destructive-blanket`) can proceed, and that flag must be logged in the dispatch_audit row.

### Specific Commands

```bash
# Audit proposed workflow steps for blanket signatures before bank acceptance
# (manual step; m23 should automate this, but CLI audit is belt-and-suspenders)
sqlite3 workflow_bank.db \
  "SELECT id, steps_json FROM accepted_workflows WHERE escape_surface_profile = 'destructive';" \
  | python3 -c "
import sys, json, re
blanket_re = re.compile(r'prune|--all-targets|rm\s+-rf|pkill\s+-f|git\s+clean|trash-empty|cargo\s+clean')
for line in sys.stdin:
    try:
        id_, steps = line.split('|', 1)
        for step in json.loads(steps):
            params = json.dumps(step.get('conductor_params', {}))
            if blanket_re.search(params):
                print(f'BLANKET-STEP-FOUND id={id_} step={step.get(\"id\",\"?\")}')
    except Exception as e:
        pass
"

# Check hookify guard is active and covering the expected patterns
grep -n 'pattern' /home/louranicas/claude-code-workspace/the-workflow-engine/the-workflow-engine-vault/boilerplate\ modules/09-trap-verify-escape-skills/hookify.preserve-blanket-guard.local.md
```

### Integration with Phases 0-6

| Phase | Gate fires |
|---|---|
| Phase 1 (T1 Specify) | m23 `ProposalBuilder` blanket-rejection gate specified |
| Phase 2 (T2 Build) | m33 Security agent blanket detection implemented |
| Phase 3 (T3 Test) | Test: workflow with `cargo clean --all-targets` step REJECTS in m33 Security agent unless Destructive + override flag |
| Phase 4 (T4 Deploy) | Pre-shadow audit: all Destructive-profile workflows in bank reviewed against blanket-signature list |
| Phase 5 (T5 Validate) | dispatch_audit reviewed for any `Destructive` dispatch that did not have `BlanketScope` deviation evidence |

### Watcher Flag Class

Class C (Confidence-gate refusal) if m33 Security agent rejects a blanket step and the workflow is re-submitted without the blanket signature being removed. This indicates scope-pressure on the preserve-list discipline. Class E (ancestor-rhyme) if the proportion of `Destructive` profile workflows in the bank grows beyond 10% of the total — this would suggest the engine is systematically proposing irreversible operations, which is the computational analog of the planning-sprawl risk.

### Failure Mode

m23 generates a workflow from an observed `docker container prune -f` run. The workflow is accepted into the bank without deviation evidence because the curator did not notice the blanket signature. m32 dispatches it six months later against a different set of containers. Conductor executes the prune. A container that was on the implicit preserve list (but not explicitly named) is removed. Mitigation: the m33 Security agent REJECT verdict on blanket + Destructive (or insufficient profile) at verification time.

### Acceptance Criteria

- m33 Security agent test: every known blanket signature without a `BlanketScope` deviation note produces a REJECT verdict
- m23 `ProposalBuilder::build()` returns `Err` when any step contains a blanket signature without a `DeviationNote { kind: BlanketScope }`
- m30 bank: no `ReadOnly` or `HostWrite` profile workflow has steps containing blanket signatures (verified by the SQLite audit query above, returning zero rows)
- `--override-destructive-blanket` flag usage is logged in dispatch_audit; CI test confirms no dispatch proceeds silently past a blanket + Destructive combination

---

## Domain 6 — Data Privacy / GDPR-Adjacent

### Threat Model

workflow-trace processes three categories of potentially privacy-sensitive data:

1. **Atuin shell history** (m1): command-line history is personal data. It can contain usernames, URLs with query parameters, API responses embedded in command output, or file paths to personal documents. In a single-user habitat, this is not a GDPR concern (GDPR applies to personal data of data subjects other than the operator). But if the workflow-trace output were shared — for example, if a workflow bank were exported to a team instance — the atuin history rows embedded in workflow lineage would become PII under GDPR Article 4(1).

2. **stcortex observations** (m2): tool_call records include which files were read, which URLs were fetched, and which commands were executed during Claude sessions. These are digital behavioral records — not currently sensitive as personal data, but adjacent to the concept of GDPR Article 4(1) if associated with a natural person in a shared context.

3. **Causal chains** (m3 injection.db): resolved command templates from the habitat memory injection system. These have lower privacy risk than atuin history but may contain workspace-specific paths that should not be shared.

**Current constraint: single-user, single-machine habitat.** GDPR does not apply. The threat is forward-looking: if the tool is ever made multi-user or its output is shared.

### Mitigations in workflow-trace

- **Single-user constraint as a first-class deployment prerequisite.** The `wf-crystallise` and `wf-dispatch` binaries must include a startup assertion that the stcortex consumer namespace is `workflow_trace_<hostname>_<uid>` — incorporating both the hostname and the UID. This makes the namespace unique to the current user on the current machine, and any attempt to share the binary between users without re-registration will fail at the DB-layer refuse-write check.

- **Workflow lineage is opaque.** m4 cluster IDs are derived via FNV-1a XOR (F11 mitigation, semantically destroyed before ID assignment). Pane labels (ALPHA-LEFT, BETA-TR) and command content are NOT stored in the bank's lineage field. The `lineage: LineageId` in `AcceptedWorkflow` is an opaque bytes string. If a workflow bank were exported, the lineage IDs would reveal nothing about which commands produced which workflow.

- **m12 reports are scoped.** The `workflow-trace correlation report` produced by m12 includes only workflow-level aggregate metrics (success rate, cost lift, execution count) — not individual command strings or atuin rows. This limits the data surface area of exported reports.

- **If multi-user: escalation required.** Any PR or issue that proposes multi-user workflow sharing must trigger a documented security review. This requirement should appear in `CONTRIBUTING.md` at the workflow-trace workspace root: `"Multi-user deployment changes require a security review covering GDPR Article 4(1) compliance, stcortex namespace isolation, and atuin history PII classification."`

- **Data retention.** m11's sunset law (120-day default) provides implicit data minimisation — workflows and their associated lineage data are pruned after sunset. This aligns with GDPR Article 5(1)(e) (storage limitation) for any future compliance context.

### Specific Commands

```bash
# Verify consumer namespace includes hostname and UID
sqlite3 ~/.local/share/workflow-trace/state.db \
  "SELECT namespace FROM stcortex_consumer_registration ORDER BY registered_at DESC LIMIT 1;"
# Expected: workflow_trace_<hostname>_<uid>

# Audit m12 report output for presence of raw command strings (should be zero)
wf-crystallise report --window 7 --format json | jq '.workflows[].steps'
# Expected: steps contain step kinds and IDs only, not command argv

# Confirm m7 does not store raw atuin argv in correlation tables
sqlite3 ~/.local/share/workflow-trace/correlations.db \
  "PRAGMA table_info(workflow_arc_record);" | grep -i 'argv\|command\|args'
# Expected: no such columns; if present, review m1 ingest schema
```

### Integration with Phases 0-6

| Phase | Gate fires |
|---|---|
| Phase 1 (T1 Specify) | Consumer namespace format specified with hostname+UID requirement |
| Phase 1 (T1 Specify) | m7 schema review: confirm no raw command content columns |
| Phase 3 (T3 Test) | Test that m12 report JSON contains no raw atuin command strings |
| Phase 6 (T6 Live) | Pre-live check: stcortex consumer namespace contains current hostname + UID |

### Watcher Flag Class

Class D (four-surface drift) if any of the four persistence surfaces (m7 SQLite, stcortex, m30 bank, m15 JSONL) is found to contain raw atuin command strings post-build. Class G (substrate-frame confusion) if consumer registration uses a namespace that omits hostname or UID, creating collision risk in a future shared deployment.

### Failure Mode

m1 stores the raw atuin `argv` field in m7's `workflow_arc_record` table for debugging purposes. A month later, a team deployment is proposed. The audit reveals that the atuin history — containing terminal commands with embedded credentials and personal paths — is stored verbatim in the correlation database. Migration is required before the multi-user deployment can proceed. Mitigation: m7 schema review at Phase 1 confirms no raw `argv` columns exist.

### Acceptance Criteria

- m7 schema contains no `argv`, `command_text`, `args`, or `content` column that stores raw atuin data
- Consumer namespace is `workflow_trace_<hostname>_<uid>`; test asserts this at startup
- m12 JSON report output contains zero raw command strings (jq audit confirms)
- `CONTRIBUTING.md` includes the multi-user escalation requirement

---

## Domain 7 — Compliance Frameworks + Audit Trail

### Threat Model

workflow-trace is not subject to SOC2, HIPAA, or GDPR obligations (single-user, no PII processing, no regulated data). However, the following compliance-adjacent risks exist:

- **Tamper detection:** if workflow lineage data or the receipt ledger can be silently modified after the fact, the audit trail is untrustworthy. A corrupt audit trail is the computational equivalent of a clinical record modification — it removes the ability to investigate what happened.
- **Accountability gap:** if the engine dispatches a workflow that causes habitat damage, the operator (Luke) needs a complete audit trail to reconstruct: what workflow was dispatched, when, by what authority, with what verification state. If any link in that chain is missing, the incident is uninvestigable.
- **Runbook theatre:** CLAUDE.local.md warns explicitly about runbooks with stale probe timestamps. A compliance document that claims `cargo audit` is run in CI but has no evidence that it was actually executed is a form of compliance theatre that creates false confidence. This domain requires evidence, not claims.

The LCM Drift #11 pattern (S1002029 learning #5) generalises directly: orchestrator MUST independently re-exercise gate claims. Agent reports are evidence to verify, not facts to trust. Applied to compliance: every compliance acceptance criterion must have an independently verifiable check command, not just an affirmation that a human reviewed the process.

### Mitigations in workflow-trace

**Audit trail architecture** (three overlapping surfaces):

1. **dispatch_log.db** (m32, audit-first guarantee): every dispatch is written to the SQLite dispatch log before the Conductor request is sent. Schema includes `dispatch_id`, `workflow_id`, `dispatched_at`, `conductor_addr`, `step_count`, `escape_surface`, `definition_hash`, `operator`, `outcome`. The audit-first write means a failed Conductor call is still recorded — the audit trail is complete even for failed dispatches. This is OWASP A09:2021 (Security Logging and Monitoring Failures) prevention.

2. **m15 pressure_register JSONL ledger** (one-event-per-file `PHASE-B-RESERVATION-NOTICE-*.jsonl`): scope-pressure events from m15 are recorded in JSONL format. Per LCM Drift #11's lesson, a canonical SHA-256 over the receipt JSONL directory should be computed and stored at each session close. This makes the ledger tamper-evident: if any JSONL file is modified after the fact, the SHA-256 comparison reveals the modification.

   Implementation: at session close (binary exit), compute `find /path/to/m15-ledger/ -name '*.jsonl' | sort | xargs sha256sum` and store the aggregate hash in a `m15_ledger_seal.sha256` file. On next session open, verify the seal before trusting ledger contents. This is analogous to the receipt-DAG-as-work pattern from CLAUDE.local.md's substrate preferences.

3. **atuin trajectory + Watcher journal**: m1 reads from the canonical atuin history, which is itself an audit log of all terminal commands. The Watcher deployment journal (S1001982) provides a timestamped observation record of the engine's deployment trajectory. These surfaces provide external corroboration of what the engine dispatched and when.

**Tamper detection for workflow bank (m30):**

The `definition_hash` field in `VerificationResult` (FNV-1a of `steps_json`) is computed at verification time and stored. m32 re-computes the hash at dispatch time and compares. If the steps were modified after verification (curator edited the bank row directly), the hash mismatch produces `DispatchError::DefinitionDrifted`. This is not cryptographic tamper detection (FNV-1a is not collision-resistant against adversarial modification) — but for a single-user local tool where the threat is accidental modification, not adversarial, FNV-1a is adequate. If workflow-trace is ever exposed to multi-user or network access, the definition_hash must be replaced with HMAC-SHA256 keyed to a per-installation secret.

**Incident response runbook:**

If the engine misbehaves (dispatches a workflow that causes unintended habitat changes):

```
1. Immediate containment:
   m32 has a dispatch cooldown gate (300s default). Remaining dispatches in the
   cooldown window are blocked. No manual intervention needed for the next 5 minutes.

   If containment is insufficient:
   - Set CONDUCTOR_DISPATCH_ENABLED=0 (env var drop)
   - Restart wf-dispatch: the binary re-initialises in refuse mode
   - wf-dispatch now returns ConductorDispatchDisabled on every call

2. Investigation:
   sqlite3 dispatch_log.db \
     "SELECT dispatch_id, workflow_id, escape_surface, outcome, dispatched_at \
      FROM dispatch_audit WHERE dispatched_at > (strftime('%s','now') - 3600) * 1000 \
      ORDER BY dispatched_at DESC;"
   # Review last hour of dispatches

   Review Watcher deployment journal:
   cat ~/projects/shared-context/watcher-notices/*.md | grep -A5 'Class-A\|Class-I'

3. Verification (m11 startup-refusal gate):
   If the investigation reveals that the engine dispatched from a workflow whose
   sunset_at has passed, this is an m11 failure mode. Run:
   sqlite3 workflow_bank.db \
     "SELECT id, sunset_at, dispatch_count FROM accepted_workflows \
      WHERE dispatch_count > 0 AND sunset_at < strftime('%s','now') * 1000;"
   # Expected: zero rows (m11 should exclude expired workflows from dispatch)

4. Recovery:
   - If atuin history corruption: `atuin import auto` re-imports from shell history
   - If stcortex namespace pollution: run namespace audit (m9 compliance command)
   - If bank corruption: restore from last `dispatch_log.db` + `workflow_bank.db` WAL checkpoint
   - Notify Watcher (WCP notice at ~/projects/shared-context/watcher-notices/)
```

**Compliance-by-default practices** (even without regulatory obligation):

- Minimum log retention: 90 days for `dispatch_log.db` (matching the m11 sunset window default of 120 days minus buffer).
- No `DELETE` from dispatch_audit except via explicit `m11` consolidation sweep with a `sunset_at`-based filter (not a blanket delete). This is preserve-list discipline applied to audit records.
- Every four-stage quality gate run (check → clippy → pedantic → test) is the compliance testing equivalent of a SOC2 CC7.1 control (logical access monitoring). The gate proves the codebase is in a known-good state before promotion.

### Specific Commands

```bash
# Generate and verify the m15 ledger SHA-256 seal
find ~/.local/share/workflow-trace/m15-ledger/ -name '*.jsonl' | sort | xargs sha256sum > /tmp/m15_ledger_current.sha256
diff /tmp/m15_ledger_current.sha256 ~/.local/share/workflow-trace/m15_ledger_seal.sha256
# Expected: no differences; if differences found, specific files are named

# Verify dispatch audit trail is not missing records (gap detection)
sqlite3 dispatch_log.db \
  "SELECT COUNT(*) as total_dispatches, \
          COUNT(CASE WHEN outcome = 'Pending' THEN 1 END) as stuck_pending \
   FROM dispatch_audit;"
# stuck_pending > 0 means a dispatch started but was never resolved

# Run cargo audit as compliance evidence (save output for audit trail)
cargo audit 2>&1 | tee /tmp/cargo-audit-$(date +%Y%m%d).txt
echo "cargo audit exit: $?"

# Verify 4-stage gate output is saved as compliance evidence in CI
# (CI should archive the gate output artifact; manual check for local runs)
ls -la /tmp/quality-gate-*.txt 2>/dev/null || echo "No gate output artifacts found — ensure CI archives them"
```

### Integration with Phases 0-6

| Phase | Gate fires |
|---|---|
| Phase 0 (G5 spec) | Incident response runbook specified; m15 ledger seal mechanism specified |
| Phase 1 (T1 Specify) | dispatch_log.db schema includes all required audit fields |
| Phase 2 (T2 Build) | Audit-first write guarantee implemented in m32 |
| Phase 3 (T3 Test) | Test: dispatch that fails at Conductor step still writes to dispatch_audit; test that failed Conductor does not produce missing audit row |
| Phase 5 (T5 Validate) | m15 ledger seal verified; dispatch_log gap detection run |
| Phase 6 (T6 Live) | `cargo audit` output saved as baseline compliance artifact |

### Watcher Flag Class

Class A (activation transition) when `CONDUCTOR_DISPATCH_ENABLED` is set — this is the highest-risk state change in the system's lifecycle, warranting verbatim timestamp. Class D (four-surface drift) if any of the audit surfaces (dispatch_log, m15 ledger, atuin, Watcher journal) becomes inconsistent with the others. Class I (Hebbian silence) if the m15 JSONL ledger stops emitting events during active dispatch sessions — this indicates scope-pressure monitoring has silently stopped.

### Failure Mode

An m32 dispatch request is sent to Conductor but times out at the network layer. Because the audit-first write is not implemented correctly (the write is attempted after the Conductor call, not before), no dispatch_audit row is created. The next session, the operator cannot determine whether the workflow was dispatched. If the Conductor did execute the workflow, the habitat change is unaccounted for. Mitigation: m32 audit-first guarantee test (`audit_row_written_before_conductor_call`) is a required test in the 60-test coverage target.

### Acceptance Criteria

- dispatch_audit `audit_row_written_before_conductor_call` test passes
- m15 JSONL ledger seal computed and stored at binary exit; verification check at binary startup returns OK
- `cargo audit` produces zero vulnerability findings and the output is saved as a dated artifact
- Incident response runbook is reachable from the workflow-trace `ai_docs/` canonical directory
- `dispatch_audit.outcome` contains zero `Pending` rows after a 24-hour shadow soak (no stuck dispatches)

---

## Domain Matrix — Security Gates per Phase

| Domain | Phase 0 | Phase 1 (T1) | Phase 2 (T2) | Phase 3 (T3) | Phase 4 (T4) | Phase 5 (T5) | Phase 6 (T6) |
|---|---|---|---|---|---|---|---|
| 1. Supply Chain | `deny.toml` specified | Authored | CI integration | `cargo audit` in test run | Pre-shadow audit | Final audit | Baseline artifact saved |
| 2. Auth + Authz | — | Payload enum type locked | UDS permission check | Type-system test | Consumer reg verified live | — | Consumer ns validated |
| 3. Secret Handling | — | Redaction patterns specified | m1 redaction + m2 SELECT hardened | PAT pattern tests | — | Grep audit | — |
| 4. Sandboxing | — | Step registry + ordinal locked | m33 surface consistency | `docker_exec`/`ReadOnly` test | Watcher witness confirmed | — | Audit log operator field reviewed |
| 5. Preserve-List | — | m23 blanket gate specified | m33 Security agent | Blanket signature tests | Destructive bank audit | dispatch_audit reviewed | — |
| 6. Data Privacy | — | Schema review (no argv cols) | ns format implemented | m12 report content test | — | — | Consumer ns startup check |
| 7. Compliance | Runbook specified | dispatch_log schema | Audit-first impl | Audit-first tests | — | Ledger seal verified | `cargo audit` artifact |

---

## Waiver Security Audit (5 Explicit Waivers)

The five single-phase override waivers are assessed here for adequacy of mitigations.

**Waiver 1 — Watcher R6 frame separation (partial).** Security implication: if the substrate-frame engine is eventually built on top of workflow-trace observations, the Watcher's observation scope (W1 narrowed consumer: tool_call + consumption only) becomes the trust boundary for the substrate-frame layer. Current mitigation: W1 is architecture, not code; m9 enforces writes but not reads. Residual risk: LOW for current scope (no substrate-frame engine planned). Mitigation adequate for accepted risk class.

**Waiver 2 — Fossil evidence-based scope discipline (full).** Security implication: shipping ~5,200 LOC without measure-first phasing creates a larger attack surface before the code has been exercised in production. CWE-1357 (Reliance on Insufficiently Trustworthy Component) applies if boilerplate is lifted without audit. Mitigation: m33's 4-agent verification gate (Security agent) provides post-build surface review. Residual risk: MEDIUM — the attack surface is wider than a phased approach would produce. Mitigation partially adequate; Zen G7 audit is the compensating control.

**Waiver 3 — RALPH selector-without-measurement safety (partial).** Security implication: if m31 selection is biased by a noisy substrate (LTP/LTD = 0.043), the engine could systematically select workflows that appear high-fitness but are artefacts of the LTD-dominant regime. This is not a confidentiality or integrity threat; it is a reliability threat that affects the usefulness of the engine's security review (if m33's Security agent is never selected for high-surface workflows, the gate is decorative). Mitigation: m11 sunset law prevents any single workflow from locking in selection. Residual risk: LOW for security impact specifically; MEDIUM for engine utility.

**Waiver 4 — Skeptic pain-source verification (full).** Security implication: building without demonstrated pain means the engine may be solving a problem that doesn't exist, resulting in dead code that accumulates security debt without providing value. Unmaintained dead code is CWE-561 (Dead Code) — not a direct vulnerability, but a maintenance burden that increases attack surface over time. Mitigation: m11's 120-day sunset + dispatch_count feedback loop means unused workflows are pruned; the engine itself is not dead code if used. Residual risk: LOW.

**Waiver 5 — Substrate exploration-protection (partial).** Security implication: F10 (m6 baseline EMA excluding Converged outcomes) and F11 (opaque cluster IDs) remain in architecture but are no longer phase-gate-protected. If F10 is misconfigured (Converged sessions contaminate the baseline), the cost model becomes unreliable. An unreliable cost model could cause m33 to underestimate the resource cost of high-surface workflows, leading to under-review of expensive operations. Mitigation: F10 is a compile-time constant in m6 (not configurable at runtime), making accidental reconfiguration impossible. Residual risk: LOW.

**Overall waiver security verdict:** mitigations are adequate for the accepted risk class of waivers 1, 3, 4, and 5. Waiver 2 (Fossil scope discipline) carries the highest residual security risk due to widened unexercised attack surface. The Zen G7 audit is the non-negotiable compensating control. Until G7 clears, this risk is unmitigated.

---

*Phase 7 authored 2026-05-17 (S1001982) by Command (security-auditor role). Cross-cutting: this phase runs in parallel with all others, not after them. HOLD-v2 active — no code authorized until G1-G9 gates clear and Luke emits explicit start-coding signal.*

*Back to: [[../HOME]] · [[../GOD_TIER_CONSOLIDATION_S1001982]]*
