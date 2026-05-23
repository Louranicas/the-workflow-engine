---
title: ADR — RefusalToken authorship-typing (V1 design spec)
date: 2026-05-23 (S1004377)
status: ACTIVE (v0.2.0 V1 — design spec for Phase 5 co-land with W1)
adr_id: D-S1004XXX-04
authors: Claude @ cortex (orchestrator, S1004377)
session: S1004377
authorising_session: S1004377 Luke "begin V2" → Plan v2 §3 Phase 1 step 4 + §2.6 ADR amendment row
audit_lane: in-session zen agent per D26 (no external Zen verdicts on file for workflow-trace; substitute per Plan v2 D25/D26)
gates_required: Plan v2 §3 Phase 5 co-land with W1 (the source-code landing); Phase 7 call-site threading; v0.2.0 ship
supersedes: none (introduces a new primitive; flat `RefusalReason` enum at `src/m32_dispatcher/mod.rs:228` becomes the V1 base case before being replaced)
companion_adrs:
  - 2026-05-17-substrate-as-actor-deferrals.md (D-S1002127-03) Amendment 1 registers V1/W4 as v0.2.0 active; this ADR is the V1 design spec
  - 2026-05-17-escape-surface-cardinality-7-privilege-escalation.md (D-S1002127-02) — `EscapeSurfaceProfile` 7-variant ordinal stability; RefusalToken `SubstrateAuthored` may carry an `EscapeSurfaceProfile`-typed payload field per NA-GAP-09 substrate-confirmable receipt linkage
addresses: [NA-GAP-01 (RefusalToken authorship-typing — Plan v2 V1; was spec-only at v0.1.0 per Phase 2 audit S1004115 §2 NA-GAP-01 row), NA-5 (Unavailable sub-tagging to prevent in-engine-receiver-only audit-indistinguishability per v0.2.0 NA gap analysis)]
ratified_decisions: DX-1 (4-variant; Watcher emits via observation channel) + DX-V5.b (3-variant `Unavailable` sub-tag confirmed) per v0.2.0 Plan v2 §15
---

# ADR — RefusalToken Authorship-Typing (V1 Design Spec)

> **Back to:** [`../../CLAUDE.md`](../../CLAUDE.md) · [`../../CLAUDE.local.md`](../../CLAUDE.local.md) · [`../WORKFLOW_TRACE_V020_PLAN_V2_S1004377.md`](../WORKFLOW_TRACE_V020_PLAN_V2_S1004377.md) §2.1 V1 row + §3 Phase 5 + §15 DX-1 / DX-V5.b · companion [`./2026-05-17-substrate-as-actor-deferrals.md`](./2026-05-17-substrate-as-actor-deferrals.md) § 7 Amendment 1

---

## § 0 — Context

The v0.1.0 / M0 codebase carries a flat `RefusalReason` enum at `src/m32_dispatcher/mod.rs:228` that classifies refusal *by cause* but not *by authorship*. Per Phase 2 audit (S1004115) §2 NA-GAP-01 row, the substrate-as-actor frame requires refusals to be typed by **who refused** (substrate / engine / operator / unavailable), not just *why*. Per v0.2.0 NA gap analysis NA-5, the `Unavailable` variant in particular must be sub-tagged to prevent the in-engine-receiver-only V5 fallback from emitting refusals that audit-indistinguishably look substrate-authored.

This ADR specifies the `RefusalToken` design that V1 lands. It is the **companion ADR to D-S1002127-03 Amendment 1** which registers V1 as a v0.2.0 active work-item.

---

## § 1 — Type design

### § 1.1 — Variants (DX-1 locked: 4-variant)

```rust
/// Authorship-typed refusal. Replaces the flat `RefusalReason` enum at
/// `src/m32_dispatcher/mod.rs:228` at v0.2.0 Phase 5 co-land with W1.
///
/// Per Plan v2 §15 DX-1: 4-variant. Watcher ☤ emits via the observation
/// channel (m46-m51 obs path), NOT via RefusalToken; Watcher's authorship
/// voice is separate from refusal voice.
///
/// Per Plan v2 §15 DX-V5.b: 3-variant `Unavailable` sub-tag prevents
/// in-engine-receiver-only V5 fallback from emitting refusals that
/// audit-indistinguishably look substrate-authored.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefusalToken {
    /// Substrate explicitly declined (e.g., stcortex `RefuseWriteNoConsumer`,
    /// HABITAT-CONDUCTOR dispatch-budget exhausted, atuin read-quota refused).
    /// The substrate has spoken; the engine listened.
    SubstrateAuthored {
        /// Which substrate authored the refusal (stcortex / conductor / atuin / etc.)
        substrate_id: SubstrateId,
        /// Substrate-side classification (substrate's own taxonomy)
        substrate_reason: String,
        /// Optional carried payload — e.g., the `EscapeSurfaceProfile` the substrate
        /// would have accepted instead. Pairs with NA-GAP-09 substrate-confirmable
        /// receipt shipped in M0 Phase 6f.
        payload: Option<RefusalPayload>,
    },
    /// Engine declined on its own behalf (e.g., m32 dispatcher rejection,
    /// m9 namespace guard `NamespaceViolation::CapabilityNotAcknowledged`,
    /// m33 verifier hard-Refuse per D5/D6).
    /// The engine is the author; substrate did not refuse.
    EngineAuthored {
        /// Which module emitted (m9 / m32 / m33 / m40 / m41 / m42)
        module_id: ModuleId,
        /// Engine-side classification — typically the original `RefusalReason` variant
        engine_reason: EngineRefusalReason,
        /// Optional payload (e.g., verifier verdict context)
        payload: Option<RefusalPayload>,
    },
    /// Operator (Luke @ node 0.A) declined via consent surface
    /// (operator-refusal path per Plan v2 §15 Round A NA-3 first-class response).
    /// Includes "this question is malformed", "not now", "present it again later".
    OperatorAuthored {
        /// Operator-side classification
        operator_reason: OperatorRefusalReason,
        /// Optional payload (e.g., requested re-framing)
        payload: Option<RefusalPayload>,
    },
    /// Substrate response is unavailable. THREE distinct sub-tags per NA-5 +
    /// DX-V5.b prevent audit-indistinguishability.
    Unavailable(UnavailableReason),
}

/// Per NA-5 + DX-V5.b: 3-variant sub-tagging is required because v0.2.0 ships
/// V5 in three substrate-participation states (per DX-V5 = full cross-habitat
/// AND per-substrate `SubstrateBackPressureMode` per DX-2 / NA-8). Without the
/// sub-tag, an engine-imagined silence is audit-indistinguishable from a
/// substrate-authored refusal.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnavailableReason {
    /// The engine fabricated this silence: substrate has no participation
    /// primitive (DX-V5 in-engine receiver-only fallback for substrates whose
    /// substrate-side schema has not yet shipped per §11 consent gradient).
    /// MUST be a first-class telemetry event per NA-5 recommendation.
    EngineImagined {
        /// Substrate whose silence is fabricated
        substrate_id: SubstrateId,
        /// Free-form reason the engine had to imagine (e.g., "stcortex consumer-trust schema not shipped")
        reason: String,
    },
    /// Substrate exists but cannot be contacted (network / port / lock / timeout / etc.)
    SubstrateUnreachable {
        substrate_id: SubstrateId,
        /// Underlying transport error
        transport_reason: String,
    },
    /// The substrate explicitly emitted "unavailable" — e.g., stcortex returned
    /// `RefuseWriteNoConsumer`, atuin returned read-quota-exceeded. The substrate
    /// has spoken; the engine listened. Distinct from `SubstrateAuthored` because
    /// the substrate refused availability rather than refused the operation.
    SubstrateAuthored {
        substrate_id: SubstrateId,
        substrate_reason: String,
    },
}

/// Substrate identifier — the 7 v2 §7 substrates plus expansion slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubstrateId {
    Atuin,
    Stcortex,
    HabitatConductor,
    HabitatInjection,  // injection.db
    Cc5LoopClocks,
    Watcher,
    Ralph,
    CargoBuildGraph,
}

/// Engine module identifier (Cluster D/G/H emitting modules).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModuleId {
    M9,   // namespace guard
    M32,  // dispatcher
    M33,  // verifier
    M40,  // Nexus emit
    M41,  // LCM RPC
    M42,  // stcortex emit
    M13,  // stcortex writer (outbox)
}

/// Optional payload for refusals — typed envelopes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum RefusalPayload {
    /// The escape surface the substrate / engine would have accepted instead
    /// (pairs with EscapeSurfaceProfile per D-S1002127-02).
    AcceptableEscapeSurface(crate::m32_dispatcher::EscapeSurfaceProfile),
    /// Verifier verdict context for engine-authored refusals
    VerifierContext(String),
    /// Re-framing suggestion for operator-authored refusals
    SuggestedReframing(String),
    /// Free-form passthrough (escape hatch; use sparingly)
    Freeform(String),
}

// EngineRefusalReason + OperatorRefusalReason taxonomies omitted from this ADR;
// they preserve the v0.1.0 RefusalReason variant set + Operator-NA-3 vocabulary
// respectively, and are defined in the Phase 5 implementation per the Phase 7
// call-site classification audit (Plan v2 §3 Phase 7 step 2).
```

### § 1.2 — Call-site classification (Phase 7 step 2 spec)

Per Plan v2 §3 Phase 7 step 2, each refusal call-site gets a *default authorship classification* that the implementer must FP-verify against the call's actual cause:

| Module:site | Default classification | Notes |
|-------------|-------------------------|-------|
| `m9_watcher_namespace_guard/validator.rs` `NamespaceViolation::CapabilityNotAcknowledged` | **SubstrateAuthored** (Stcortex if from reducer refusal; EngineAuthored M9 if from local-trait check) | Conditional on call origin; FP-verify |
| `m32_dispatcher/mod.rs:228` (`RefusalReason` → `RefusalToken::EngineAuthored M32`) | **EngineAuthored** M32 | All current variants are engine-authored |
| `m13_stcortex_writer/mod.rs` `outbox_path` write failures | **SubstrateAuthored** Stcortex | Substrate refused write (refuse-write-no-consumer); rare fail-on-disk = EngineAuthored M13 |
| `m40_nexus_emit/mod.rs:51,248,572,1860` (`ServerRejected` events) | **SubstrateAuthored** (substrate-id from event) | Already substrate-frame in semantics; just gets typed enum |
| `m41_lcm_rpc/mod.rs` RPC failures | **mixed** — `Unavailable::SubstrateUnreachable` LCM (transport) vs `SubstrateAuthored::Lcm` (refused) | FP-verify per call |
| `m42_stcortex_emit/mod.rs` `RefuseWriteNoConsumer` | **SubstrateAuthored** Stcortex | The canonical case |
| `m33_verifier` Refuse verdicts | **EngineAuthored** M33 | All m33 Refuse verdicts are engine-authored gate decisions |
| `wf-dispatch` operator-interactive ack-refusal | **OperatorAuthored** | Per HumanAcceptanceSignature monotone gate |

### § 1.3 — Backwards-compat with v0.1.0 `RefusalReason`

Per DX-W.c locked = **SemVer-break**: v0.2.0 is a breaking proposal-wire change. The flat `RefusalReason` enum at `src/m32_dispatcher/mod.rs:228` is **removed** at Phase 5 co-land with W1; its variants become `EngineRefusalReason` variants nested inside `RefusalToken::EngineAuthored { engine_reason: ... }`. v0.1.0 `proposals.jsonl` files do not deserialise into v0.2.0; CHANGELOG [v0.2.0] § "Changed" carries the migration note.

### § 1.4 — Substrate-confirmable receipt linkage

The v0.1.0 / M0 Phase 6f shipped `NexusEventKind::WorkflowRefused` + `RefusalReceipt` per Plan v2 §8 D8. v0.2.0 V1 extends those receipts to carry the typed `RefusalToken` so receipt consumers can read authorship without re-classifying. Receipt format becomes:

```rust
pub struct RefusalReceipt {
    pub refused_at_ms: u64,
    pub workflow_id: WorkflowId,
    pub token: RefusalToken,  // v0.2.0 addition; v0.1.0 had string `reason` field
    pub substrate_attestation: Option<String>,  // v0.2.0 addition; optional substrate-side echo
}
```

---

## § 2 — Phase landings

| Phase | Work | LOC |
|-------|------|-----|
| **Phase 5 (co-land with W1)** | Define `RefusalToken` + `UnavailableReason` + `SubstrateId` + `ModuleId` + `RefusalPayload` enums (this ADR specifies); thread through every refusal call-site (replace `RefusalReason` usage in m32 + m9 + m13 + m40 + m41 + m42 + m33); JSONL serde regen (one wire-contract regen pass per C-2); `RefusalReceipt` shape update | ~150-250 LOC + ~80-150 tests |
| **Phase 7** | Call-site classification audit + drain wire (per § 1.2 table); FP-verify each default classification against actual call origin; m13 drain skeleton from Phase 3 wires to consume `RefusalToken::SubstrateAuthored Stcortex` events | ~50-100 LOC (just classification corrections + drain wire) |
| **Phase 9 V3 m16** | V3 emits `SubstrateDriftDetected` events through `RefusalToken::SubstrateAuthored` channel (per Plan v2 §3 Phase 9 step 4) | (consumed; no new LOC for this ADR) |
| **Phase 11 V5** | V5 in-engine receiver-only emits `RefusalToken::Unavailable(EngineImagined{...})` per § 1.1; `substrate_participation_status: enum { NotShipped, Shipping, Live }` accessor consumed by `EngineImagined` reason field | (consumed; no new LOC for this ADR) |

---

## § 3 — Risks specific to this ADR

| Risk | Mitigation |
|------|------------|
| Phase 7 call-site classification table (§ 1.2) wrong on multiple sites | FP-verify each call origin before threading; per-site test asserts the classification |
| `EngineRefusalReason` taxonomy drifts from v0.1.0 `RefusalReason` (silent breakage) | Phase 5 includes round-trip test: v0.1.0-shape JSONL fails to deserialise with explicit SemVer error message |
| `RefusalPayload::AcceptableEscapeSurface` introduces a cross-module dependency m32 → cross_cutting (or wherever EscapeSurfaceProfile lives) | Plan v2 §3 Phase 6e m9 ↔ m32 trait seam (shipped M0) already establishes the pattern; reuse |
| Substrate's `substrate_reason` field is free-form string — no validation | Per NA discipline, the substrate's own taxonomy is the source of truth; engine must not impose schema on substrate's refusal vocabulary |
| `RefusalToken::Unavailable(SubstrateAuthored{...})` and `RefusalToken::SubstrateAuthored{...}` are similar — possible confusion | Doc-comment-clear distinction: the outer-SubstrateAuthored is refusal of operation; Unavailable-SubstrateAuthored is refusal of availability. Phase 5 includes contrast test. |

---

## § 4 — Decision register fields

- **Decision ID:** D-S1004XXX-04 (XXX = sequential per habitat ADR convention; final ID assigned at landing)
- **Status:** ACTIVE (v0.2.0 V1 design spec)
- **Decision-makers:** Claude @ cortex orchestrator (S1004377 Plan v2 Phase 1 step 4); Luke @ node 0.A authorised via DX-1 + DX-V5.b interview locks (S1004377 Plan v2 §15)
- **Affected surfaces:**
  - `src/m32_dispatcher/mod.rs` (RefusalReason enum removal — Phase 5)
  - all refusal call-sites in m9 / m32 / m33 / m40 / m41 / m42 / m13 (Phase 5 + Phase 7)
  - `RefusalReceipt` struct + every JSONL fixture asserting against it (Phase 5 wire-contract regen pass)
  - `ai_specs/cross-cutting/refusal-taxonomy.md` (language update Phase 5 + cascade per D-S1002127-03 Amendment 1 § 7.3)
- **Reversal cost:** HIGH. The wire-contract change is SemVer-breaking; reverting requires another SemVer-break to restore the v0.1.0 shape. The classification choice (4-variant vs 5-variant w/ WatcherAuthored) is reversible by additive variant addition.

---

## § 5 — Acceptance discipline

This ADR is accepted when:
1. ✅ Type design § 1.1 produced with all 4 outer variants + 3 Unavailable sub-variants + payload typed-enum.
2. ✅ Call-site classification table § 1.2 enumerated with default classifications + FP-verify-required marker for ambiguous sites.
3. ✅ SemVer-break documented in § 1.3 with v0.1.0 → v0.2.0 migration note pointer.
4. ✅ Substrate-confirmable receipt linkage § 1.4 named.
5. ⏳ Phase 5 commit lands the actual type definitions + call-site threading + receipt-shape update.
6. ⏳ Phase 7 commit lands call-site classification audit + drain wire.
7. ⏳ V0.2.0 ship CHANGELOG [v0.2.0] § "Changed" carries the SemVer-break migration note.

---

> **Back to:** [`../../CLAUDE.md`](../../CLAUDE.md) · [`../../CLAUDE.local.md`](../../CLAUDE.local.md) · [`../WORKFLOW_TRACE_V020_PLAN_V2_S1004377.md`](../WORKFLOW_TRACE_V020_PLAN_V2_S1004377.md) · companion [`./2026-05-17-substrate-as-actor-deferrals.md`](./2026-05-17-substrate-as-actor-deferrals.md) § 7 Amendment 1

*Filed 2026-05-23 (S1004377 · Plan v2 Phase 1 step 4) · Claude @ cortex · workflow-trace v0.2.0 execution begin · V1 design spec · in-session zen agent audit per D26.*
