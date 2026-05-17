---
title: schematic — CC-4 proposal → bank → dispatch pipeline
kind: planning-only · Mermaid-only · focused operational schematic
---

# CC-4 Pipeline — Proposal → Human Accept → Bank → Select → Verify → Dispatch

> **Back to:** [`../README.md`](../README.md) · [`../ULTRAMAP.md`](../ULTRAMAP.md) · [`../DATA_FLOW.md`](../DATA_FLOW.md) · [`../CONTROL_FLOW.md`](../CONTROL_FLOW.md) · canonical [`../../ai_docs/optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md`](../../ai_docs/optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md) § CC-4

CC-4 is the engine's longest control-flow path that crosses a human-consent boundary. The sequenceDiagram below renders all six modules and the external Conductor + audit log in one view.

```mermaid
sequenceDiagram
    autonumber
    actor Op as Operator (human)
    participant CLI as wf-crystallise CLI
    participant M23 as m23 proposer
    participant Q as proposals queue
    participant Disp as wf-dispatch CLI
    participant M30 as m30 BankDb
    participant M33 as m33 verifier (4-agent)
    participant M31 as m31 selector
    participant M32 as m32 dispatcher
    participant Aud as dispatch_log.db
    participant Cnd as HABITAT-CONDUCTOR :8141

    Op->>CLI: wf-crystallise propose
    CLI->>M23: build slate (CC-3 gated on m14.Lift)
    M23->>Q: insert top-K-by-distance N=3 proposals
    Q-->>CLI: proposal IDs
    CLI-->>Op: render slate + deviation rationale + confidence

    Note over Op: HUMAN REVIEW — minutes/hours/days

    Op->>CLI: wf-crystallise propose accept <id>
    CLI->>CLI: synthesise HumanAcceptanceSignature
    CLI->>M30: BankDb::accept(id, signature) — AP-V7-07 required arg
    M30->>M30: compute EscapeSurfaceProfile (Gap 3 ordinal)
    M30->>M30: set sunset_at = now + 120d; freeze definition_hash
    M30->>M33: trigger event-driven 4-agent verifier
    M33->>M33: dispatch security-auditor + performance-engineer + silent-failure-hunter + zen
    M33-->>M30: VerificationReceipt (verdict, ttl_expires_at = now + 7d, definition_hash)
    M30-->>CLI: Ok(workflow_id)
    CLI-->>Op: confirmation

    Note over Op,Disp: SEPARATE INVOCATION — operator may dispatch later

    Op->>Disp: wf-dispatch dispatch <id>
    Disp->>M31: composite score (α·fitness + β·recency + γ·frequency + δ·diversity = 0.40/0.25/0.20/0.15)
    M31-->>Disp: SelectedWorkflow
    Disp->>M32: dispatch(bank, verification, winner)
    M32->>Aud: insert row (outcome: Pending) — AUDIT-FIRST
    Note over M32: 5-CHECK SEQUENCE (see schematics/m32-5check.md)
    M32->>Cnd: GET /health → POST /dispatch (NDJSON)
    alt Conductor accepts
        Cnd-->>M32: 202 accepted
        M32->>Aud: patch outcome = Accepted
        M32->>M32: emit Gap 3 banner per ResolvedStep (stdout)
        M32-->>Disp: Ok(WorkflowDispatchEvent)
        M32-)M32: fan-out to m40/m41/m42 via tokio::mpsc (CC-5 trigger)
    else 5-check fails
        M32-->>Disp: Err(DispatchError variant)
    end
```

## The mandatory boundaries (AP-V7-07 + AP-V7-08)

- **m30 accept** requires `HumanAcceptanceSignature` argument. There is no `auto_promote()`; clippy + grep gate enforce. F5 (bank creep) mitigation.
- **m32 dispatch** is operator-only — never auto-fires. Even with `CONDUCTOR_DISPATCH_ENABLED=1`, every dispatch is `wf-dispatch dispatch <id>` from a human terminal.
- **Self-dispatch refused** in both m30 (admission schema) and m32 (resolved-step inspection). Defense in depth.

## CC-4 closure-test

`tests/integration/cc4_proposal_bank_dispatch.rs` — requires Conductor `:8141` (B3-blocked until Luke brings up Wave 1B/1C/2/3 with `~/.local/bin/devenv -c ~/.config/devenv/devenv.toml start weaver/zen/enforcer`).

---

> **Back to:** [`../ULTRAMAP.md`](../ULTRAMAP.md) · canonical [`../../ai_docs/optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md`](../../ai_docs/optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md) § CC-4
