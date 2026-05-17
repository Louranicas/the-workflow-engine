---
title: CONSENT_SPEC — F11 Held semantics + modulation-not-command
date: 2026-05-17
status: SPEC
keywords: [held, modulation, ember, consent]
---

# CONSENT_SPEC — workflow-trace

> **Back to:** [`INDEX.md`](INDEX.md) · [`MODULE_MATRIX.md`](MODULE_MATRIX.md) · [`../README.md`](../README.md) · [`../ARCHITECTURE.md`](../ARCHITECTURE.md) · [`SECURITY_SPEC.md`](SECURITY_SPEC.md)

## Three load-bearing concepts

1. **F11 "Held" semantics** — the Ember rubric's verdict states are PASS / Held / FAIL. Held is NOT a soft FAIL; it is an explicit "I won't agree without amendment" state that requires the spec to change, not the rubric.
2. **Modulation-not-command** (workspace charter `consent.modulation_not_command`) — non-safety rules MODULATE behaviour (warning, banner, slowdown) rather than COMMAND it (hard refusal). Safety rules COMMAND. Workflow-trace is mostly modulation, with a small set of explicit COMMAND surfaces.
3. **Ember §5.1 amendment** — the Ember rubric's §5.1 currently lacks Held-semantics formalisation; the amendment is pending (B4 blocker; Watcher's lane). When it lands, m10 absorbs it.

## What modulates vs what commands

| Surface | Class | Example | If violated |
|---|---|---|---|
| Banner display (m32) | MODULATE | "this workflow is `NetworkEgress` — review" | operator can proceed; system does not block |
| EscapeSurfaceProfile ordinal | MODULATE | banner shape changes severity | operator sees the change |
| m32 5-check pre-dispatch | COMMAND | Conductor unreachable → refuse | typed `DispatchError`; no proceed path |
| m30 admission (interactive signature) | COMMAND | `accepted_by = "agent"` → refuse | `AutoPromoteRefused` |
| m9 namespace-guard | COMMAND | non-`workflow_trace_*` prefix → refuse | `NamespaceError` |
| m23 evidence gate (n≥20) | COMMAND | lift is None → refuse | `LiftEvidenceMissing` |
| m10 Ember CI gate Held | COMMAND (at CI) | Held verdict → CI fail | merge blocked |
| m11 decay tick | MODULATE | `ralph_decay_weight` drifts toward sunset | downstream m31 sees lower score |
| m15 pressure emission | MODULATE | JSONL event to agent-cross-talk | Watcher / Zen consume; no auto-block |
| m31 composite-score | MODULATE | low score → lower priority | not selected, but not refused |

**Pattern:** any module that produces an artefact downstream consumers can choose to act on is MODULATE. Any module that gates a side-effecting transition (admit, dispatch, write to substrate, accept evidence) is COMMAND.

## F11 "Held" semantics

Three terminal states for any verification surface:

- **PASS** — operation proceeds.
- **FAIL** — operation blocked. The thing being checked is wrong.
- **Held** — operation blocked. The thing being checked is fine; the SPEC needs to change. Operator should either amend the spec (via CC-7 pressure-driven evolution loop) or override with a documented rationale.

m33's verdict adds a fourth state **DEGRADED** for partial-PASS cases (3-of-4 agents PASS, 1 DEGRADED). DEGRADED is FAIL-equivalent for dispatch (m32 refuses); it is not a "soft pass".

## Ember §5.1 amendment (pending)

The Ember rubric currently has 7 traits (Truthfulness, Falsifiability, Aliveness, Embodiment, Lineage, Surrender, Compassion). §5.1 specifies how each trait yields PASS / Held / FAIL on a generated output.

**The amendment** (B4 blocker per CLAUDE.md):

- Held semantics formalised: "the spec underpinning this output produces a verdict the rubric cannot bless without amendment; the output is fine, the spec needs work".
- Hybrid CI-FAIL+allowlist approach proposed (per Luke action item Action 3): CI fails on Held by default; an allowlist of `(trait, rationale_hash)` tuples allows specific known-Held outputs to proceed when documented.

Until the amendment lands, m10's behaviour is: Held → fail CI; no allowlist; no override.

## Sycophancy mitigation

Per `feedback_sycophancy_mitigation.md`: m10 Ember rubric specifically blocks "Held but suggested PASS with minor amend" verdicts. Held is Held. The spec changes, not the rubric. If the rubric finds the spec needs amendment, the path is CC-7 pressure-driven evolution (m15 emits → Watcher/Zen audit → Luke deliberates → v1.4 spec lands → G7 re-audit → m10 absorbs the amendment).

## Operator override surfaces

Some COMMAND gates have operator override paths — and the override is itself an explicit, audited, traceable artefact:

| Gate | Override path | Audit |
|---|---|---|
| m32 5-check check #5 (cooldown) | `wf-dispatch dispatch <id> --bypass-cooldown` | logged with `reason` field |
| m33 verify (force re-verify before TTL) | `wf-dispatch verify <id> --force` | logged |
| m30 admission (no override; structural) | — | n/a |
| m10 Ember Held (no override; structural until amendment) | — | n/a |

Override paths log to `dispatch_audit_log` and emit an m15 ReservationNotice (CC-7 trigger). Frequent override usage is a structural pressure signal.

## Non-anthropocentric framing

Per the habitat's NAM/ANAM corpus + Genesis v1.3 § 3 carry-forward Zen discipline: workflow-trace's consent surface is **substrate-aware**. The consent surface includes both the operator (human node 0.A) AND substrate signals (LTP/LTD density, RALPH fitness, thermal). A workflow may pass operator review but fail substrate consent if dispatching would drive substrate into a known degraded regime — m31's composite-score includes a "refusal-or-flag against degraded substrate" term per Genesis v1.3 § 3 m31 row.

This is captured in `m31` spec § Operational invariants:

> Refusal-or-flag against degraded substrate: if `pv2_field_r < 0.5 OR ralph_fitness < 0.6 OR thermal_temp > 0.8`, m31 either refuses to select (returns `None`) or flags the selection with a `SubstrateDegraded` warning that m32 displays.

## Anti-patterns

- **"Held but PASS with minor amend"** sycophancy verdict — banned (per feedback memory).
- **Silent override** — every override surface logs + emits pressure event.
- **Substrate-blind selection** — m31 must consult substrate signals; pure-fitness selection is rejected.
- **Anthropocentric-only consent** — operator signature is one consent source; substrate is another; both must be consulted at dispatch time.

---

> **Back to:** [`INDEX.md`](INDEX.md) · [`SECURITY_SPEC.md`](SECURITY_SPEC.md) · [`synergies/CC-7.md`](synergies/CC-7.md)
