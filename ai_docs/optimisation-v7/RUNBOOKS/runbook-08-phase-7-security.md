---
title: Runbook 08 — Phase 7 Security + Compliance (CROSS-CUTTING continuous)
date: 2026-05-17 (S1001982)
kind: planning-only · operational runbook · cross-cutting continuous (runs in PARALLEL with phases 1-6)
phase: 7 of 6 (cross-cutting; not sequential)
domains: 7 — supply-chain · auth/authz · secrets · sandboxing · preserve-list · data-privacy · audit-trail
owner: security-auditor (per phase) + Command (orchestration) + Zen (G7 audit) + Watcher (waiver tracking)
waivers_security_relevant: 5 (Watcher R6, Fossil scope, RALPH selector, Skeptic pain-source, Substrate exploration-protection)
compensating_control_waiver_2: Zen G7 audit + Phase 4 Zen audit (non-negotiable for Fossil scope waiver)
authority: Luke @ node 0.A
status: planning-only · HOLD-v2 active · NOT executable until G1-G9 GREEN
---

# Runbook 08 — Phase 7 Security + Compliance (CROSS-CUTTING)

> Back to: [[../TASK_LIST_V7_OPTIMISATION.md]] · [[../ANTIPATTERNS_REGISTER.md]] · sibling [[runbook-06-phase-5-deploy-soak.md]] · [[runbook-09-phase-8-observability.md]] · [[runbook-10-cross-cutting.md]] · [[runbook-11-emergency-rollback.md]]
>
> Source phase doc: [[../../the-workflow-engine-vault/deployment framework/phase-7-security-compliance.md]] (covers W1/W2/AP30/m9/m10/Cipher-escape-surface architecture + 7 domains × 7 phases matrix + waiver security audit)

---

## Overview

Phase 7 is **not a final checklist** run after deployment — it is a **parallel security lens** that fires specific gates at each of Phases 1-6. Seven domains × seven phases produces a 49-cell responsibility matrix where each cell names the gate, the owner, and the check command. Because workflow-trace is a CLI (no inbound listener, no multi-user boundary, no PII processing), the attack surface is narrow; defensive practices apply anyway because **habits formed here propagate to future services** and the habitat substrate already stores sensitive-adjacent data (atuin shell history with embedded credentials, stcortex observations, causal chains). Five risk waivers ride on Command's head per the single-phase override (S1001982); Waiver 2 (Fossil scope-discipline FULL waiver) carries the highest residual risk and **the Zen G7 audit + Phase 4 Zen audit is its non-negotiable compensating control**.

---

## Pre-flight checklist

Run continuously throughout the deployment lifecycle. Per-phase items must complete before the corresponding phase gate fires.

- `cargo audit` clean baseline established at Phase 1 close
- `deny.toml` authored at workspace root (Phase 1 T1 specify)
- PAT-pattern redaction rules specified in m1 module spec (Phase 1)
- m33 4-agent verification gate scaffold present (Security agent + Performance + Silent-Failure + Zen)
- LCM UDS socket permission check planned for m41 startup (Phase 2)
- `EscapeSurfaceProfile` ordinal enum locked at workspace root (Phase 1)
- `tests/ember_gate.rs` present and passing (m10 output security gate)
- m9 namespace guard `assert_workflow_trace_namespace()` implemented (Phase 2)
- Zen G7 audit verdict captured for v1.3 spec patch (compensating control for Waiver 2)
- `~/projects/shared-context/wf-retirement/` directory created for compliance artefacts

If any item misses: HALT the corresponding phase. Phase 7 gates are cross-cutting — they are not optional after-the-fact checks.

---

## Step 1 — 7 security domains × 7 phases matrix (canonical)

This is the operational core. Each cell names what fires when. Drop-in commands follow.

| Domain | Phase 0 (pre-G9) | Phase 1 T1 Specify | Phase 2 T2 Build | Phase 3 T3 Test | Phase 4 T4 Hardening | Phase 5 T5 Validate | Phase 6 T6 Sunset |
|---|---|---|---|---|---|---|---|
| **1. Supply Chain** | `deny.toml` spec authored; `cargo audit` planned for CI | `deny.toml` materialised | `cargo audit` + `cargo deny check` in CI | `cargo audit` in test run | pre-shadow audit | final pre-deploy audit | baseline artefact saved (`/tmp/cargo-audit-D120.txt`) |
| **2. Auth + Authz** | — | m40 `NexusEvent.data` enum type locked (no `serde_json::Value`) | m41 UDS permission check in startup | type-system test confirms enum constraint | consumer-reg verified live before shadow | — | consumer-ns validated `workflow_trace_<host>_<uid>` |
| **3. Secret Handling** | — | PAT redaction patterns in m1 spec | m1 redaction pass + m2 `SELECT` list hardened | tests confirm PAT rows redacted; `BankDb::accept()` rejects PAT in curator note | — | pre-release grep audit `rg 'glpat-\|github_pat_\|gho_\|ghp_'` zero matches | — |
| **4. Sandboxing** | — | step-kind registry + ordinal locked | m33 mechanical gate surface consistency | `docker_exec`-in-`ReadOnly` test FAILS m33 | Watcher witness confirmed available for `SandboxEscape` profile | dispatch audit `operator` field reviewed (all `wf-dispatch/human`) | — |
| **5. S102 Preserve-List** | — | m23 `ProposalBuilder` blanket-rejection gate spec | m33 Security agent blanket detection implemented | test: `cargo clean --all-targets` step REJECTS in m33 unless `Destructive` + override | `Destructive`-profile bank audit | `dispatch_audit` reviewed for any `Destructive` lacking `BlanketScope` deviation evidence | — |
| **6. Data Privacy** | — | consumer-ns format spec includes `<hostname>_<uid>`; m7 schema review (no `argv` cols) | namespace format implemented | m12 report content test (no raw command strings) | — | — | consumer-ns startup check |
| **7. Compliance + Audit** | runbook spec; m15 ledger seal mechanism spec | `dispatch_log.db` schema all required audit fields | audit-first write guarantee in m32 | `audit_row_written_before_conductor_call` test passes; failed Conductor still writes audit row | — | m15 JSONL ledger seal verified; gap-detection run | `cargo audit` artefact saved as baseline |

**Inputs:** all module specs · Phase 1-6 runbooks · workspace `Cargo.toml` + `Cargo.lock` · `WAIVER_REGISTER.md`.

**Outputs:** per-cell evidence artefact (TXT/JSONL/SHA file); per-phase security-auditor APPROVE/REFUSE; cumulative compliance artefact bundle in `~/projects/shared-context/wf-retirement/`.

---

## Step 2 — Per-domain detail commands (drop-in)

### Domain 1 — Supply chain

```bash
set -o pipefail
cd /home/louranicas/claude-code-workspace/the-workflow-engine

# 1.1 — RUSTSEC audit
cargo audit 2>&1 | tee /tmp/wt-cargo-audit-$(date +%Y%m%d).txt
[[ ${PIPESTATUS[0]} -ne 0 ]] && { echo "FAIL: cargo audit"; exit 1; }

# 1.2 — Policy enforcement (advisories + license + ban)
cargo deny check advisories 2>&1 | tail -20
[[ ${PIPESTATUS[0]} -ne 0 ]] && { echo "FAIL: cargo deny advisories"; exit 2; }
cargo deny check licenses 2>&1 | tail -20
cargo deny check bans 2>&1 | tail -20

# 1.3 — Lockfile committed
git diff --exit-code Cargo.lock || { echo "FAIL: Cargo.lock dirty"; exit 3; }

# 1.4 — Build-script audit (manual review trigger)
cargo metadata --format-version 1 \
  | jq '[.packages[] | select(.build_script != null) | .name]' \
  > /tmp/wt-build-scripts.json
```

### Domain 2 — Auth + authz

```bash
# 2.1 — m40 NexusEvent type discipline (no open serde_json::Value bag)
rg --type rust 'NexusEvent.*Value' src/m40_synthex_emit/ && \
  { echo "FAIL: NexusEvent.data is open Value"; exit 1; }

# 2.2 — m41 UDS socket permissions check (at startup)
stat -c "%a %U %G %n" /path/to/lcm.sock 2>/dev/null
# expected: 600 <user> <group>; if not, m41 logs tracing::warn and proceeds (fail-open by design)

# 2.3 — stcortex consumer freshness (m2 + m13 dependency)
curl -s --max-time 2 http://127.0.0.1:3000/consumers \
  | jq '.[] | select(.namespace | startswith("workflow_trace"))'

# 2.4 — PAT pattern in NexusEvent emission paths
rg --type rust 'NexusEvent.*data.*=.*env|PAT|TOKEN|glpat|github_pat' src/
```

### Domain 3 — Secret handling

```bash
# 3.1 — PAT pattern grep in workspace (BLOCKING — zero matches required)
rg --hidden -n '(glpat-[A-Za-z0-9_-]{20,}|github_pat_[A-Za-z0-9_]{36,}|ghp_[A-Za-z0-9]{36}|gho_[A-Za-z0-9]{36}|Authorization:\s*Bearer\s+[A-Za-z0-9._-]{20,}|:[A-Za-z0-9_-]{20,}@)' \
  -g '!.git' -g '!target' .
[[ $? -eq 0 ]] && { echo "FAIL: PAT pattern found"; exit 1; }   # rg exit 0 = match found = FAIL here

# 3.2 — .env in .gitignore
grep -n '\.env' .gitignore || { echo "FAIL: .env not gitignored"; exit 2; }

# 3.3 — Cargo.toml + build.rs free of credentials
rg 'glpat-|github_pat_|ghp_|gho_' Cargo.toml build.rs 2>/dev/null \
  && { echo "FAIL: credentials in Cargo.toml or build.rs"; exit 3; }

# 3.4 — m1 ingest code redacts BEFORE logging argv
rg --type rust 'argv|command|history' src/m1_atuin_consumer/ \
  | grep -v 'redact\|REDACTED' \
  && echo "REVIEW: m1 has raw argv refs without redaction marker"
```

### Domain 4 — Sandboxing

```bash
# 4.1 — m32 has ZERO direct process::Command calls (P0 #3 — Conductor-only dispatch)
rg --type rust 'tokio::process::Command|std::process::Command' src/m32_dispatcher.rs \
  && { echo "FAIL: m32 spawns processes directly (P0 violation)"; exit 1; }

# 4.2 — Step-kind registry covers all expected kinds + correct surface
cat src/workflow_core/step_registry.rs   # manual review for completeness

# 4.3 — Dispatch audit operator field is always wf-dispatch/human
sqlite3 ~/.local/share/workflow-trace/dispatch_log.db \
  "SELECT DISTINCT operator FROM dispatch_audit;" \
  | grep -v 'wf-dispatch/human' \
  && { echo "FAIL: rogue operator found in dispatch_audit"; exit 2; }
```

### Domain 5 — S102 preserve-list discipline

```bash
# 5.1 — Bank scan for blanket steps in non-Destructive profile (SHOULD return zero rows)
sqlite3 workflow_bank.db \
  "SELECT id, steps_json FROM accepted_workflows WHERE escape_surface_profile != 'destructive';" \
  | python3 -c "
import sys, json, re
blanket_re = re.compile(r'prune|--all-targets|rm\s+-rf|pkill\s+-f|git\s+clean|trash-empty|cargo\s+clean')
fails = 0
for line in sys.stdin:
    try:
        id_, steps = line.split('|', 1)
        for step in json.loads(steps):
            params = json.dumps(step.get('conductor_params', {}))
            if blanket_re.search(params):
                print(f'FAIL: blanket-step in non-destructive workflow id={id_}')
                fails += 1
    except Exception: pass
sys.exit(1 if fails else 0)
"

# 5.2 — Hookify preserve-blanket-guard active
grep -n 'pattern' ~/claude-code-workspace/the-workflow-engine/the-workflow-engine-vault/boilerplate\ modules/09-trap-verify-escape-skills/hookify.preserve-blanket-guard.local.md
```

### Domain 6 — Data privacy

```bash
# 6.1 — Consumer namespace includes hostname + UID
sqlite3 ~/.local/share/workflow-trace/state.db \
  "SELECT namespace FROM stcortex_consumer_registration ORDER BY registered_at DESC LIMIT 1;"
# expected pattern: workflow_trace_<hostname>_<uid>

# 6.2 — m12 report contains zero raw command strings
wf-crystallise report --window 7 --format json \
  | jq '[.workflows[].steps[] | select(.argv? // empty)] | length'
# expected: 0

# 6.3 — m7 schema has no argv-shaped columns
sqlite3 ~/.local/share/workflow-trace/correlations.db \
  "PRAGMA table_info(workflow_arc_record);" \
  | grep -iE 'argv|command|args'
# expected: empty output
```

### Domain 7 — Compliance + audit trail

```bash
# 7.1 — m15 JSONL ledger seal
find ~/.local/share/workflow-trace/m15-ledger/ -name '*.jsonl' | sort | xargs sha256sum \
  > /tmp/wt-m15-current.sha256
diff /tmp/wt-m15-current.sha256 ~/.local/share/workflow-trace/m15_ledger_seal.sha256
[[ $? -ne 0 ]] && echo "WARN: m15 ledger seal mismatch — investigate"

# 7.2 — Dispatch audit gap detection (no stuck Pending rows)
sqlite3 ~/.local/share/workflow-trace/dispatch_log.db \
  "SELECT COUNT(*) as stuck FROM dispatch_audit WHERE outcome = 'Pending' AND dispatched_at < strftime('%s','now')*1000 - 86400000;"
# expected: 0

# 7.3 — cargo audit as compliance artefact (dated)
cargo audit 2>&1 | tee /tmp/wt-cargo-audit-$(date +%Y%m%d).txt
[[ ${PIPESTATUS[0]} -eq 0 ]] && echo "Compliance artefact saved"
```

---

## Step 3 — FNV-1a vs HMAC-SHA256 single-user / multi-user discriminator

workflow-trace currently uses **FNV-1a** for:

- m4 cluster IDs (F11 opaque ID mitigation; semantic-destruction XOR)
- m33 `definition_hash` (drift detection at dispatch time)
- m31 `wf_m31_selection_weight` label (FNV-1a u32 cardinality cap)

FNV-1a is **not cryptographic** — it's collision-resistant only against accidental modification, not adversarial. This is **acceptable for single-user local tool** where the threat model is accidental drift, not active adversary.

**Multi-user upgrade rule:** if workflow-trace is ever exposed to multi-user OR network access, replace FNV-1a with **HMAC-SHA256** keyed to a per-installation secret. Specifically:

| Use site | Single-user (current) | Multi-user (future) |
|---|---|---|
| m4 cluster ID | FNV-1a 64-bit XOR | HMAC-SHA256(per-install-key, window\|labels\|step_count) truncated 128-bit |
| m33 definition_hash | FNV-1a 64-bit of `steps_json` | HMAC-SHA256(per-install-key, `steps_json`) |
| m31 metric label | FNV-1a u32 of workflow_id | HMAC-SHA256(scrape-key, workflow_id) truncated u32 |

The per-installation secret lives in `~/.local/share/workflow-trace/install.key` (mode 0600, owner-only). The key is generated once at `wf-crystallise --init` and never logged.

---

## Step 4 — Waiver security audit (5 explicit waivers, S1001982)

Per [[../../the-workflow-engine-vault/deployment framework/phase-7-security-compliance.md]] § Waiver Security Audit, mitigations are assessed for adequacy:

| Waiver | Security implication | Residual risk | Compensating control |
|---|---|---|---|
| **W1** Watcher R6 frame separation (partial) | W1 narrowed-consumer scope becomes substrate-frame trust boundary if L9 built | LOW | architectural discipline + m9 write guard; m2 SELECT-list reviewer check on PR |
| **W2** Fossil evidence-based scope discipline (FULL) | larger attack surface before live exercise; CWE-1357 (insufficient trustworthy component) for boilerplate-lift | **MEDIUM** — highest of the 5 | **Zen G7 audit + Phase 4 Zen audit — NON-NEGOTIABLE compensating control** |
| **W3** RALPH selector-without-measurement safety (partial) | m31 selection biased by noisy substrate; potentially decorative m33 Security agent if never selected for high-surface workflows | LOW (security); MEDIUM (engine utility) | m11 sunset law prevents single-workflow lock-in |
| **W4** Skeptic pain-source verification (FULL) | unmaintained dead code (CWE-561) increases attack surface over time | LOW | m11 120-day sunset + `dispatch_count` feedback loop prunes unused |
| **W5** Substrate exploration-protection (partial) | F10 misconfiguration could under-estimate resource cost of high-surface workflows | LOW | F10 is compile-time constant in m6 (no runtime reconfiguration possible) |

**Overall verdict:** waivers 1, 3, 4, 5 mitigations adequate for accepted risk class. **Waiver 2 mitigation is conditional on Zen G7 audit clearing.** Until G7 clears, residual security risk is unmitigated.

---

## Phase-end gate

Each Phase 0-6 has its security-domain cells (see Step 1 matrix). Phase 7 itself has no "end" — it terminates with the workflow-trace deployment terminating (Phase 6 retirement OR continuation extension). At retirement:

```bash
# Compliance artefact bundle
TS=$(date -u +%Y-%m-%dT%H:%M:%SZ)
mkdir -p ~/projects/shared-context/wf-retirement/compliance-${TS}/
cp /tmp/wt-cargo-audit-*.txt   ~/projects/shared-context/wf-retirement/compliance-${TS}/
cp /tmp/wt-m15-current.sha256   ~/projects/shared-context/wf-retirement/compliance-${TS}/
cp ~/.local/share/workflow-trace/m15_ledger_seal.sha256 \
  ~/projects/shared-context/wf-retirement/compliance-${TS}/
sqlite3 ~/.local/share/workflow-trace/dispatch_log.db \
  ".dump dispatch_audit" > ~/projects/shared-context/wf-retirement/compliance-${TS}/dispatch_audit_full.sql

atuin kv set "workflow_trace.phase7.close.timestamp"      "$TS"
atuin kv set "workflow_trace.phase7.close.audit_artefact_dir" \
  "~/projects/shared-context/wf-retirement/compliance-${TS}/"
```

---

## Failure modes register

| ID | Trigger | Detection | Mitigation | Antipattern |
|---|---|---|---|---|
| F-SEC-1 | `cargo audit` returns CVE finding | exit non-zero | HALT build; update offending dep; re-run | AP-Drift-09 (secret/CVE in committed file) |
| F-SEC-2 | PAT pattern found in source (Domain 3 grep) | rg returns match | redact + force-push purge; rotate PAT (S1001883 E1/E2 pattern) | AP-Drift-09 |
| F-SEC-3 | m32 spawns `process::Command` directly | rg returns match in m32 | refuse merge; route via Conductor | (P0 #3 violation) |
| F-SEC-4 | Blanket step in non-Destructive workflow | Domain 5 query returns rows | m33 Security agent REJECT verdict; remove from bank | AP-Hab-04 preserve-list discipline |
| F-SEC-5 | Consumer namespace missing hostname or UID | Domain 6 query shows generic ns | refuse startup; m9 namespace guard upgrade | AP-Hab-03 AP30 violation |
| F-SEC-6 | m15 JSONL seal mismatch | Domain 7 diff non-empty | investigate which file modified; tamper detection real | (tamper detection working as designed) |
| F-SEC-7 | Zen G7 audit verdict missing for Waiver 2 compensating control | `WAIVER_REGISTER.md` lacks Zen sign-off line | HALT Phase 5A deploy; Waiver 2 mitigation incomplete | (compensating control failure) |

---

## Watcher flag pre-positioning

| Class | Pre-position trigger | What it captures |
|---|---|---|
| A | Domain 7 `CONDUCTOR_DISPATCH_ENABLED` toggle | activation transitions on highest-risk state changes |
| B | m40 NexusEvent emission to SYNTHEX with non-enum data field | hand-off boundary type violation |
| C | m33 Security agent REJECT verdict | confidence-gate refusal (blanket-step or surface inconsistency) |
| **D** | any of 4 persistence surfaces (m7 SQLite / stcortex / m30 bank / m15 JSONL) shows PAT pattern post-build | four-surface drift on secret containment |
| E | DEGRADED + blanket-step bank growth beyond 10% of total | ancestor-rhyme via irreversible-operation proposals |
| F | Domain 7 `wf_runs_total` increments while G9 not green in HOME.md | AP24 violation surface |
| G | consumer namespace omits hostname or UID (collision risk in future shared deploy) | substrate-frame confusion |
| H | atuin trajectory missing expected `cargo audit` runs | atuin proprioception anomaly on compliance evidence |
| I | m15 JSONL ledger stops emitting during active dispatch | Hebbian-silence analog on scope-pressure monitoring |

---

## Atuin trajectory anchors

```bash
# Continuous security audit trajectory
atuin search "cargo audit"            --before 7d
atuin search "cargo deny"             --before 7d
atuin search "rg.*glpat"              --before 7d
atuin search "stcortex.*consumers"    --before 7d
atuin scripts run wt-bridge-check
atuin scripts run wt-substrate-pulse
atuin kv get  "workflow_trace.phase7.close.timestamp"
```

---

## Sign-off

This runbook is **planning-only** (HOLD-v2). The 7×7 matrix becomes operational only when the corresponding Phase runbook fires. Phase 7 is **cross-cutting** — it does not have its own dedicated execution window; it runs in parallel with every other phase. The compensating control for Waiver 2 (Fossil scope, FULL waiver) is the Zen G7 audit + Phase 4 Zen audit — without these clearing, the deployment proceeds with unmitigated residual risk on Command's head.

*Runbook 08 authored 2026-05-17 by Command (V7 optimisation, parallel author). 7 security domains × 7 phases matrix encoded. FNV-1a vs HMAC-SHA256 multi-user discriminator documented. 5-waiver security audit attached. Waiver 2 compensating control named explicitly. ~1,690 words. Source: phase-7-security-compliance.md. Sibling: runbook-06 / runbook-09 / runbook-10 / runbook-11.*
