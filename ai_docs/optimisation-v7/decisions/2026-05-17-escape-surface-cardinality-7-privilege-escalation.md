---
title: ADR — EscapeSurfaceProfile cardinality 7 (PrivilegeEscalation inserted at ordinal 30)
date: 2026-05-17
status: PROPOSED-AT-G7 (awaits Zen verdict via D-B6 AMEND-loop)
authors: [Luke @ node 0.A directive, Command]
session: S1002127
supersedes: v1.3 § 1 cardinality-6 (Sandboxed/SandboxEscape/ProcessMutate/FileWrite/NetworkEgress/DataExfil)
gates_required: G7 re-audit verdict
companion_adrs:
  - 2026-05-17-m42-stcortex-only-pivot.md
  - 2026-05-17-g8-stcortex-persistence-plan.md
register_entry: D-S1002127-02
cardinality_amendment: "S1002127 — PrivilegeEscalation inserted at ordinal 30 (D-S1002127-02 ADR)"
---

# ADR — EscapeSurfaceProfile cardinality 7 (PrivilegeEscalation @ ord 30)

> **Back to:** [`../../../CLAUDE.md`](../../../CLAUDE.md) · [`../../../CLAUDE.local.md`](../../../CLAUDE.local.md) · [`../DECISION_REGISTER.md`](../DECISION_REGISTER.md) · [`../STANDARDS/GOD_TIER_RUST.md`](../STANDARDS/GOD_TIER_RUST.md) · [`../../../ai_specs/DESIGN_CONSTRAINTS.md`](../../../ai_specs/DESIGN_CONSTRAINTS.md) · [`../../../ai_specs/SECURITY_SPEC.md`](../../../ai_specs/SECURITY_SPEC.md) · [`../../../ultramap/schematics/gap3-escape-surface-ordinal.md`](../../../ultramap/schematics/gap3-escape-surface-ordinal.md)
> **Companion ADRs:** [`2026-05-17-m42-stcortex-only-pivot.md`](2026-05-17-m42-stcortex-only-pivot.md) · [`2026-05-17-g8-stcortex-persistence-plan.md`](2026-05-17-g8-stcortex-persistence-plan.md)
> **Decision Register entry:** D-S1002127-02 (decision #63 in V7 register: 13 V7 + 48 grilling + D-S1002127-01 G8 persistence + this entry)

---

## § 0 — Context

The original V7 framework specified `EscapeSurfaceProfile` cardinality as **5** (Read / IdempotentWrite / Mutate / Destructive / Catastrophic per the pre-v1.3 `escape_surface_profile.schema.json`). v1.3 binding spec consolidated this to **6** with the operationally-grounded names `Sandboxed / SandboxEscape / ProcessMutate / FileWrite / NetworkEgress / DataExfil` per the m30 + m32 + m9 unified destructiveness schema (Gap 3 structural-gap authorship). The Wave 3 `agent-claim-verifier` flagged the V7 `=5` vs v1.3 `=6` cardinality drift as a pending V7 GOD_TIER_RUST amendment.

Luke directed at S1002127 (2026-05-17): **`=7`** — cardinality bumped to 7 with a new variant inserted between `ProcessMutate` and `FileWrite`. The chosen variant is `PrivilegeEscalation` at ordinal 30, with numeric ordinal gaps reserved (steps of 10) so future inserts do not perturb existing ordinals. This ADR captures the decision, embeds the canonical `PrivilegeEscalation` definition verbatim where each spec defines the enum, and lists the ~12 amended files plus the verification protocol.

The decision follows the D-B6 AMEND-loop (Command authors the amendment-only delta; Zen re-fires G7 audit; Luke override authority preserved). G7 verdict is **pending**; this ADR is filed at G7 audit and is the binding amendment if Zen ACCEPTs.

---

## § 1 — Decision

`EscapeSurfaceProfile` cardinality is amended from 6 to **7**. The full 7-variant ladder, with numeric ordinals using steps of 10 to reserve gaps for future inserts, is:

| Variant | Ordinal | Banner | Distinguishing dimension |
|---|---:|---|---|
| `Sandboxed` | 0 | `SANDBOXED` | no observable side-effects outside step container |
| `SandboxEscape` | 10 | `SANDBOX-ESCAPE` | step writes outside its sandbox |
| `ProcessMutate` | 20 | `PROCESS-MUTATE` | kills/respawns processes WITHIN current privilege envelope |
| **`PrivilegeEscalation`** | **30** | **`PRIVILEGE-ESCALATION!`** | **NEW — capability gain or role elevation** |
| `FileWrite` | 40 | `FILE-WRITE` | writes to user/system files under EXISTING write permission |
| `NetworkEgress` | 50 | `NETWORK-EGRESS!` | outbound network calls |
| `DataExfil` | 60 | `DATA-EXFIL!` | transmits potentially-sensitive data; rm -rf / drop database |

Existing 6 variants retain their ordinal positions across the cardinality-6→7 bump; only the new variant occupies a previously-reserved gap. The Rust enum carries `#[repr(u8)]` with explicit discriminants so the ordinal-to-numeric mapping is binding at the type level.

### `PrivilegeEscalation` canonical definition (embed verbatim where each spec defines the enum)

> Capability gain or role elevation that grants the calling process new abilities beyond its pre-call state. Examples: invoking `sudo`; setuid/setgid; capability acquisition (`cap_set_proc`, `setcap`); ACL add; container privilege escalation (Docker `--privileged`, `cap-add`); SELinux/AppArmor profile escape. Distinguished from `ProcessMutate` (modifying another process WITHIN current privilege envelope) and `FileWrite` (which requires existing write permission but does NOT acquire new capabilities). Habitat-relevant: openclaw container UID-1337 escape, sudo gates, role elevations in nerve-center / Conductor.

---

## § 2 — Rationale

`PrivilegeEscalation` captures the **structurally-missing tier** between "modify another process" and "write to disk". The previous 6-variant ordering left a semantic gap: `ProcessMutate` is bounded by the calling process's privilege envelope (you can `kill` a process you already have authority over); `FileWrite` requires existing write permission but does not acquire new capabilities. **Capability gain** — `sudo`, setuid/setgid, `cap_set_proc`, Docker `--privileged`, SELinux/AppArmor escape — sits between those: it does not write to disk directly, but once acquired, enables further escalation along all higher tiers.

The habitat has direct production-incident lineage for this class. The openclaw container UID-1337 escape, sudo gates in nerve-center, and role elevations in the Conductor are all `PrivilegeEscalation` events that the original 6-variant ladder either lumped under `SandboxEscape` (semantically wrong — capability gain ≠ sandbox boundary breach) or `ProcessMutate` (semantically wrong — gaining new privileges ≠ mutating within existing ones). Classifying these as their own variant lets m9, m30, m32, and m33 enforce policy distinctly.

The numeric-ordinal-with-gap-reservation approach (steps of 10) closes the Cluster G "ordinal stability across versions" G7 concern (formerly m30 Open Q1 / m32 Open Q1 / m33 Open Q4). Adding a future variant at, e.g., ordinal 25 (between `ProcessMutate` and `PrivilegeEscalation`) does not perturb existing `>=` comparisons or invalidate stored serde rows. Existing storage / wire / index data remains valid.

---

## § 3 — Affected spec amendments

The following spec files were amended at this ADR's authorship (~12 files):

1. `ai_specs/DESIGN_CONSTRAINTS.md` — compile-time invariant C4 cardinality 6→7; runtime invariant R9 cardinality 6→7; cross-reference to this ADR
2. `ai_specs/SECURITY_SPEC.md` — threat-model table 7-row with PrivilegeEscalation row; m9 capability gate row; m33 composition row; PrivilegeEscalation canonical definition embedded
3. `ai_specs/modules/cluster-G/m30_curated_bank.md` — Public Surface enum 7-variant; banner_line() table 7-row; ordinal numeric table; tests 8 (Ord agrees with stated ordinal) updated for 7 variants
4. `ai_specs/modules/cluster-G/m32_conductor_dispatcher.md` — cooldown ladder 7-row (PrivilegeEscalation 25 min between ProcessMutate 20 and FileWrite 30); test 12 amended
5. `ai_specs/modules/cluster-G/m33_verifier.md` — composition table: PrivilegeEscalation = 3-of-4 zero-REJECT (same band as NetworkEgress + SandboxEscape + ProcessMutate); DataExfil remains UNANIMOUS-no-nits; tests 11a/11b/11c added
6. `ai_specs/modules/cluster-D/m9_watcher_namespace_guard.md` — capability table by EscapeSurfaceProfile 7-row; PrivilegeEscalation requires `HumanAcceptanceSignature.privilege_escalation_acknowledged = true`
7. `.claude/schemas/escape_surface_profile.schema.json` — enum array 7-entries (order = ordinal); ordinal_mapping object 7-entries; m9_refuse_threshold const updated to 30
8. `ai_docs/optimisation-v7/STANDARDS/GOD_TIER_RUST.md` — verify-sync invariant 19 cardinality 5→7 (was 5 in original V7, 6 in v1.3, now 7)
9. `ARCHITECTURE.md` (root) — Gap 3 row notes cardinality 7 + 7-variant list at ordinals 0/10/20/30/40/50/60
10. `ultramap/schematics/gap3-escape-surface-ordinal.md` — Mermaid diagram 7-node with PrivilegeEscalation edges; cooldown ladder 7-row; change-discipline updated
11. `ultramap/INVARIANT_MAP.md` — compile-time invariant row updated for 7-variant + `#[repr(u8)]` discriminants
12. `ai_docs/optimisation-v7/DECISION_REGISTER.md` — append D-S1002127-02

Skipped (no EscapeSurfaceProfile mention to amend, per surgical-edit policy): `ai_specs/synergies/CC-6.md`, `ai_specs/cross-cutting/feature-gating.md`, `ANTIPATTERNS.md`.

---

## § 4 — Backward compatibility

The existing 6 variants keep their ordinal positions:

| Variant | Pre-ADR ordinal | Post-ADR ordinal | Δ |
|---|---:|---:|---:|
| `Sandboxed` | 0 | 0 | 0 |
| `SandboxEscape` | 1 | 10 | +9 (numeric only; relative position unchanged) |
| `ProcessMutate` | 2 | 20 | +18 (numeric only) |
| `FileWrite` | 3 | 40 | +37 (numeric only) |
| `NetworkEgress` | 4 | 50 | +46 (numeric only) |
| `DataExfil` | 5 | 60 | +55 (numeric only) |
| **`PrivilegeEscalation`** | — | **30** | NEW |

Because the project has **0 LOC of Rust code** (HOLD-v2 active) and **0 stcortex `workflow_trace_*` writes** (G8 not yet fired per D-S1002127-01), there is no on-disk serialised data carrying pre-ADR ordinal values to migrate. The serde-rename attribute uses `snake_case` variant names (not numeric ordinals) for stored representations, so wire / DB rows would round-trip correctly even across the numeric ordinal change. The `Ord`-based comparisons in m30 / m32 / m9 / m33 use the derived `Ord` trait (based on declaration order) which is preserved.

The only structural change for downstream consumers is a clippy non-exhaustive-match warning if a `match` on `EscapeSurfaceProfile` does not enumerate `PrivilegeEscalation`. This is desired — it catches drift at the source-of-truth level.

---

## § 5 — m33 verification composition for PrivilegeEscalation

`PrivilegeEscalation` is composed at the **3-of-4 zero-REJECT** band (same as `NetworkEgress`, `SandboxEscape`, `ProcessMutate`). Rationale:

- Capability gain is high-stakes but **bounded** — acquiring a new capability does not by itself transmit data or destroy state; it enables further escalation. Defense-in-depth means the m33 gate is one of several layers (m9 capability flag + m32 cooldown + Conductor routing + operator banner).
- Requiring UNANIMOUS (DataExfil's band) would over-gate routine privilege-bearing operations (e.g. `sudo apt update` in a workflow), creating operator-fatigue auto-Y pressure.
- Allowing majority (Sandboxed/FileWrite's band) would under-gate capability-gain, missing the structural distinction this ADR is built around.

3-of-4 zero-REJECT preserves the gating discipline of the middle band while ensuring any agent's structural finding blocks PASS. DEGRADED is permissible (3 APPROVE + 1 APPROVE-WITH-NITS) and triggers the same downstream behaviour as DEGRADED at other middle-band surfaces.

Composition table (post-ADR; canonical at m33 spec § 5):

```text
Sandboxed | FileWrite:
    ≥3 APPROVE && zero REJECT → PASS
    ≥3 APPROVE && any REJECT  → FAIL
    else                       → FAIL

SandboxEscape | ProcessMutate | PrivilegeEscalation | NetworkEgress:
    ≥3 APPROVE && zero REJECT && zero NITS  → PASS
    ≥3 APPROVE && zero REJECT && any NITS   → DEGRADED
    else                                     → FAIL

DataExfil:
    4 APPROVE && zero NITS → PASS
    else                    → FAIL  (UNANIMOUS no-nits)
```

---

## § 6 — m32 cooldown ladder amendment

`PrivilegeEscalation` is assigned a 25-minute cooldown, sitting monotonically between `ProcessMutate` (20 min) and `FileWrite` (30 min):

```text
EscapeSurfaceProfile::Sandboxed            →   5 min
EscapeSurfaceProfile::SandboxEscape        →  10 min
EscapeSurfaceProfile::ProcessMutate        →  20 min
EscapeSurfaceProfile::PrivilegeEscalation  →  25 min   (NEW)
EscapeSurfaceProfile::FileWrite            →  30 min
EscapeSurfaceProfile::NetworkEgress        →  45 min
EscapeSurfaceProfile::DataExfil            →  60 min
```

Rationale: capability-gain workflows should be rate-limited more than ProcessMutate (which acts within existing privilege) but less than FileWrite (which is the gateway to most observable system state changes). The 25-minute slot preserves monotonicity and respects the "more destructive ⇒ longer cooldown" invariant.

Operator override: `--bypass-cooldown` flag still works but is audited as a separate `DispatchAuditRow` event. For `PrivilegeEscalation`, the bypass additionally requires `HumanAcceptanceSignature.privilege_escalation_acknowledged = true` per m9 capability table.

---

## § 7 — Zen G7 re-audit flag (filed via AMEND-loop)

This ADR is filed at G7 audit per the D-B6 AMEND-loop. Filed alongside `~/projects/shared-context/agent-cross-talk/2026-05-17T160500Z_command_g7_audit_request_v1_3_amendment.md` (the v1.3 amendment AUDIT-REQUEST v2) as a delta amendment.

**Next action recommended:** Command files an amended AUDIT-REQUEST (v3) to Zen citing this ADR as the cardinality-7 amendment delta and listing the 12 affected files. Zen's G7 verdict on the AMEND must cover:

1. PrivilegeEscalation semantic distinctness vs SandboxEscape and ProcessMutate (Q: does the new variant introduce ambiguity or close a gap?)
2. m33 3-of-4 zero-REJECT band placement (Q: is the verification depth proportionate to capability-gain risk?)
3. m32 25-minute cooldown placement (Q: is 25 min appropriate or should it bias toward FileWrite at 30 min?)
4. m9 `privilege_escalation_acknowledged` HumanAcceptanceSignature field addition (Q: is the field necessary or does the existing isatty check suffice?)
5. Numeric-gap reservation (steps of 10) (Q: is the future-proofing pattern adequate, or should ordinals be larger gaps / sparse-IDs?)

If Zen ACCEPTs: cardinality 7 is ratified; the 12 amended files become the binding spec; v1.3 § 1 + Appendix A reference this ADR by ID; D-S1002127-02 lands as decision #63.

If Zen objects: D-B6 AMEND-loop iterates — Command revises this ADR per Zen's verdict; Zen re-audits; repeat until ACCEPT. Luke override authority preserved.

---

## § 8 — Bidirectional anchors

- **Project charter:** [`../../../CLAUDE.md`](../../../CLAUDE.md) → links here via [`../DECISION_REGISTER.md`](../DECISION_REGISTER.md)
- **Session state:** [`../../../CLAUDE.local.md`](../../../CLAUDE.local.md) → S1002127 scaffold-wave-3 row
- **Companion ADRs:** [`2026-05-17-m42-stcortex-only-pivot.md`](2026-05-17-m42-stcortex-only-pivot.md) (substrate-pivot) · [`2026-05-17-g8-stcortex-persistence-plan.md`](2026-05-17-g8-stcortex-persistence-plan.md) (persistence plan)
- **Decision Register:** [`../DECISION_REGISTER.md`](../DECISION_REGISTER.md) — D-S1002127-02
- **Standards manifest:** [`../STANDARDS/GOD_TIER_RUST.md`](../STANDARDS/GOD_TIER_RUST.md) — verify-sync invariant 19
- **Cluster-G specs:** [`../../../ai_specs/modules/cluster-G/m30_curated_bank.md`](../../../ai_specs/modules/cluster-G/m30_curated_bank.md) · [`../../../ai_specs/modules/cluster-G/m32_conductor_dispatcher.md`](../../../ai_specs/modules/cluster-G/m32_conductor_dispatcher.md) · [`../../../ai_specs/modules/cluster-G/m33_verifier.md`](../../../ai_specs/modules/cluster-G/m33_verifier.md)
- **Cluster-D spec:** [`../../../ai_specs/modules/cluster-D/m9_watcher_namespace_guard.md`](../../../ai_specs/modules/cluster-D/m9_watcher_namespace_guard.md)
- **Schematic:** [`../../../ultramap/schematics/gap3-escape-surface-ordinal.md`](../../../ultramap/schematics/gap3-escape-surface-ordinal.md)
- **Invariant map:** [`../../../ultramap/INVARIANT_MAP.md`](../../../ultramap/INVARIANT_MAP.md)
- **JSON schema:** [`../../../.claude/schemas/escape_surface_profile.schema.json`](../../../.claude/schemas/escape_surface_profile.schema.json)
- **Threat model:** [`../../../ai_specs/SECURITY_SPEC.md`](../../../ai_specs/SECURITY_SPEC.md) § EscapeSurfaceProfile threat model
- **Compile-time + runtime invariants:** [`../../../ai_specs/DESIGN_CONSTRAINTS.md`](../../../ai_specs/DESIGN_CONSTRAINTS.md) C4 + R9

---

> **Back to:** [`../../../CLAUDE.md`](../../../CLAUDE.md) · [`../../../CLAUDE.local.md`](../../../CLAUDE.local.md) · [`../DECISION_REGISTER.md`](../DECISION_REGISTER.md) · companion ADRs [`2026-05-17-m42-stcortex-only-pivot.md`](2026-05-17-m42-stcortex-only-pivot.md) · [`2026-05-17-g8-stcortex-persistence-plan.md`](2026-05-17-g8-stcortex-persistence-plan.md)

*ADR authored 2026-05-17 (S1002127) by Command per Luke `=7` directive. Awaits Zen G7 verdict via D-B6 AMEND-loop. D-S1002127-02 in Decision Register.*
