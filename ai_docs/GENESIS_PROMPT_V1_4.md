# Genesis Prompt v1.4 — m16 substrate-drift canary amendment

> **Authored:** 2026-05-24 (S1004377, Plan v2 v0.2.0 §3 Phase 9 step 1)
> **Status:** AMENDMENT to v1.3 (`GENESIS_PROMPT_V1_3.md`) — lifts module count 26 → 27; adds m16 to Cluster E
> **Trigger:** ADR D-S1002127-03 Amendment 1 (W4) — V3 m16 substrate-drift canary as v0.2.0 active work-item; DX-V3 locked = own module (per Plan v2 §15)
> **Authorisation:** Luke @ node 0.A "begin V2" S1004377 Phase 1 go + Plan v2 §15 DX-V3 locked-as-own-module
> **Zen G7 re-audit:** DISPATCHED via pair-file at `~/projects/shared-context/agent-cross-talk/2026-05-24T*_zen_genesis_v1_4_audit_request.md`; **DX-V3.b ship-cap = 7 days from this commit** — if Zen verdict is silent at N=7 days, ship v0.2.0 with the cardinality drift named in CHANGELOG `[v0.2.0]` § "Honest residuals". Otherwise fold the verdict per APPROVE / APPROVE-WITH-NITS / AMEND.
> **Back to:** [`GENESIS_PROMPT_V1_3.md`](GENESIS_PROMPT_V1_3.md) · [`WORKFLOW_TRACE_V020_PLAN_V2_S1004377.md`](WORKFLOW_TRACE_V020_PLAN_V2_S1004377.md) §3 Phase 9 · [`decisions/2026-05-17-substrate-as-actor-deferrals.md`](decisions/2026-05-17-substrate-as-actor-deferrals.md) Amendment 1 · [`WORKFLOW_TRACE_V020_PHASE2_AUDIT_S1004377.md`](WORKFLOW_TRACE_V020_PHASE2_AUDIT_S1004377.md) § 5 Genesis pre-flight

---

## What this amendment changes from v1.3

Per Phase 2 audit (S1004377) § 5 Genesis v1.4 pre-flight, four v1.3 anchors are amended:

| v1.3 line | v1.3 text | v1.4 amendment |
|-----------|-----------|----------------|
| `:35` | `## § 1 — Architecture (26 modules / 8 clusters / 9 layers / 2 binaries)` | **27 modules / 8 clusters / 9 layers / 2 binaries** (m16 joins Cluster E per § 1 below) |
| `:37` | `The single-phase architecture is 26 modules across 8 synergy clusters and 9 layers (L0-L8)` | **27 modules** across 8 synergy clusters and 9 layers (L0-L8) |
| `:62` | `**OI-3 resolution**: the architecture count is **26 modules**, not v1.2's 11. m33 is the additive module (§ 1.a).` | (unchanged — v1.3 resolution stays as-historical) **NEW: OI-3.b resolution** — v1.4 lifts module count to 27; m16 is the v0.2.0 additive module per ADR D-S1002127-03 Amendment 1 + Plan v2 v0.2.0 §2.1 V3 + DX-V3 own-module lock |
| `:288` | `Total budget **1,562 tests** across 26 modules` | **Total budget 1,602 tests** across 27 modules (+40 for m16 per ADR D-S1002127-03 §3 W1 row estimate) |

Cluster count unchanged (8). m16 joins Cluster E per the Phase 2 audit recommendation: m16 is a substrate-observation module homologous to m14 (lift) and m15 (pressure) which already inhabit Cluster E.

## § 1 — Architecture (27 modules / 8 clusters / 9 layers / 2 binaries)

The single-phase architecture is **27 modules** across 8 synergy clusters and 9 layers (L0-L8), packaged as two binaries plus a shared library, all inside a single Cargo crate. v1.3 § 1's canonical module-by-module table is amended to add m16 to Cluster E; all other rows unchanged.

### § 1.a (unchanged from v1.3) — OI-3 resolution: 26 modules + m33 additive

(v1.3 text preserved verbatim.)

### § 1.b (NEW) — OI-3.b resolution: v1.4 lifts to 27 modules + m16 additive

Per ADR D-S1002127-03 Amendment 1 (registers NA-GAP-01 / NA-GAP-04 / NA-GAP-06-drain as v0.2.0 active work-items) + Plan v2 v0.2.0 §2.1 V3 (substrate-drift canary KEYSTONE) + §15 DX-V3 (locked = own module): m16 is the **v0.2.0 additive module**. Cluster placement = Cluster E (Evidence + Pressure → Evidence + Pressure + Substrate-Observation). Cluster count unchanged at 8.

## § 1.c (NEW) — Cluster E expansion table

| Module | Role (per Plan v2 §2.4 cluster-E rows) | LOC est |
|--------|-----------------------------------------|---------|
| m14 (existing) | Habitat outcome lift (Wilson CI) | ~120 |
| m15 (existing) | Pressure register (JSONL events) | ~80 |
| **m16 (NEW)** | **Substrate-drift canary — 5 clock samplers + agree-to-skew envelope per `ai_specs/cross-cutting/substrate-drift.md`. Emits `SubstrateDriftDetected` via V1 `RefusalToken::SubstrateAuthored Cc5LoopClocks` channel per ADR D-S1004XXX-04 §1.2.** | ~250-350 + ~40 tests |
| Cluster E total | (Evidence + Pressure + Substrate-Observation) | ~450-550 + ~120 tests |

## § 1.d — Test budget revision

Per ADR D-S1002127-03 §3 W1 row estimate ("~250-350 LOC + ~40 tests"), the v1.4 test budget lifts:

| v1.3 | v1.4 |
|------|------|
| 1,562 tests across 26 modules (per § 6 STANDARDS/TEST_DISCIPLINE.md `≥50 tests per module` × 26 modules baseline) | **1,602 tests** across 27 modules (1,562 + 40 m16 tests) |

The minimum-floor matrix (1,562 v1.3 / 1,599 v1.3 V7-G6) lifts proportionally to 1,602 v1.4. The actual M0 test count at v0.1.0 ship was 2044 (well above the v1.3 floor); the v1.4 floor lifts to 2044 + 40 = ~2084 (still well below the running v0.2.0 count which has reached 2118+ as of Phase 8 close). The v1.4 amendment does not affect the live test count's headroom over the floor.

## § 2 — Backwards-compatibility with v1.3

This amendment is **additive** at the architecture level (one new module + one cluster row) and the test-budget level (+40 tests). No v1.3 module is removed, renamed, or restructured. All v1.3 binding-spec rules (every Verb-Lock; F2 enforcement-point; F9 zero-weight; F10 EMA window; F12 sunset taxonomy; etc.) are preserved verbatim.

## § 3 — Pair-file dispatch for Zen G7 re-audit (DX-V3.b 7-day clock)

Per Plan v2 §3 Phase 9 step 1 (own-module branch) + DX-V3.b ship-cap, this amendment is pair-filed at:

```
~/projects/shared-context/agent-cross-talk/2026-05-24T_zen_genesis_v1_4_audit_request.md
```

The pair-file requests Zen's verdict on the cardinality drift (26 → 27 modules + Cluster E expansion). Per Plan v2 D26 D-substitute-in-session-zen and the Phase 2 audit § 4 confirmed Zen-verdict-absent precedent for workflow-trace, the 7-day silent-ship cap fires automatically at N=7 days from this commit. CHANGELOG `[v0.2.0]` § "Honest residuals" will name the un-audited cardinality drift if Zen remains silent.

## § 4 — m16 implementation spec reference

m16 implementation lands in this same Phase 9 commit at `src/m16_substrate_drift_canary/`. Spec compliance per `ai_specs/cross-cutting/substrate-drift.md`:

- **5 clock samplers**: m11 recency / m13 stcortex-decay / injection-TTL / atuin-checkpoint / stcortex-pathway-decay
- **Agree-to-skew envelope**: configurable `max_skew_ms` per clock-pair crossing (default 5s for substrate-stable systems; operator-tunable)
- **Watcher m16 heartbeat liveness assertion** (per NA-4 self-canary mitigation): Watcher's deployment-watch journal asserts m16 emits heartbeat per cycle; missing heartbeat for N cycles = substrate-emitted alert via Watcher's separate clock (closing the self-canary loop honestly per NA-4)
- **Rate-limited alerts** (per C-9 alert-fatigue mitigation): ≤N alerts per soak hour with dedup by `(clock-pair, envelope-band)`; operator visible only after N consecutive crossings
- **Emission via V1 RefusalToken::SubstrateAuthored Cc5LoopClocks**: detected drift events route through the authorship-typed envelope shipped Phase 5 V1 (the substrate IS speaking when its clocks disagree)

## § 5 — Standing for node 0.A (post-Zen-wait)

- **Zen verdict folded** (within 7 days): per APPROVE / APPROVE-WITH-NITS / AMEND, optionally amend m16 implementation before v0.2.0 ship.
- **Zen silent at N=7d**: per DX-V3.b lock, ship v0.2.0 with cardinality drift named as honest residual in CHANGELOG `[v0.2.0]`. No further Luke escalation required (cap is in-plan-locked).

---

*Genesis Prompt v1.4 amendment authored S1004377 · 2026-05-24 · Claude @ cortex · per Plan v2 v0.2.0 §3 Phase 9 step 1 + DX-V3 own-module + DX-V3.b 7-day ship-cap · Zen G7 re-audit pair-filed (dispatch is in-this-commit) · m16 implementation lands in this same Phase 9 commit.*
