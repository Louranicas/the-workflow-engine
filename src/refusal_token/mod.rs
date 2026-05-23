//! `RefusalToken` — authorship-typed refusal envelope (V1, Plan v2 v0.2.0
//! §3 Phase 5 step 5 per C-2 co-land discipline + ADR D-S1004XXX-04).
//!
//! v0.2.0 introduces authorship-typed refusal: every refusal carries
//! *who* refused (substrate / engine / operator / unavailable) alongside
//! *why*. Per Plan v2 §15 DX-1 = 4-variant outer enum (Watcher emits via
//! observation channel, not refusal channel) + DX-V5.b = 3-variant
//! `Unavailable` sub-tag preventing in-engine-receiver-only V5 fallback
//! from being audit-indistinguishable from substrate-authored refusal.
//!
//! ## Co-land discipline (per C-2)
//!
//! This module **defines** the types. Call-site classification —
//! replacing every flat `RefusalReason` emission with the
//! authorship-typed envelope — is the Phase 7 work (per Plan v2 §3
//! Phase 7 step 2: "Audit call-site authorship classifications"). v0.2.0
//! Phase 5 ships the type + a couple of canonical conversion seams; the
//! 65-occurrence call-site cascade across 7 files is the deliberate
//! Phase 7 scope. This is the lean execution of "co-land" per C-2 —
//! the type lands, the consumers wire in subsequent phases.
//!
//! ## Stacks the W1+W3+A4 SemVer-break (per DX-W.c semantics extended)
//!
//! `RefusalToken` and `RefusalPayload` derive serde Serialize +
//! Deserialize so substrate-confirmable receipts (NexusEvent
//! `WorkflowRefused` + `RefusalReceipt` shipped at M0 Phase 6f) can
//! carry typed refusals on the wire. v0.2.0 `RefusalReceipt` adds an
//! optional `token: Option<RefusalToken>` field (Some on v0.2.0,
//! `None`-fallback for cross-version receipts pre-cutover).
//!
//! ## Type layout
//!
//! ```text
//! RefusalToken                         (the outer envelope)
//!   ├── SubstrateAuthored              (substrate explicitly declined)
//!   │     { substrate_id, substrate_reason, payload }
//!   ├── EngineAuthored                 (engine declined on its own behalf)
//!   │     { module_id, engine_reason, payload }
//!   ├── OperatorAuthored               (Luke @ node 0.A declined via consent surface)
//!   │     { operator_reason, payload }
//!   └── Unavailable(UnavailableReason)
//!         ├── EngineImagined           (engine fabricated the silence — DX-V5.b)
//!         │     { substrate_id, reason }
//!         ├── SubstrateUnreachable     (substrate exists but cannot be contacted)
//!         │     { substrate_id, transport_reason }
//!         └── SubstrateAuthored        (substrate emitted "unavailable")
//!               { substrate_id, substrate_reason }
//! ```

use serde::{Deserialize, Serialize};

/// Authorship-typed refusal envelope. Per ADR D-S1004XXX-04 §1.1 + Plan
/// v2 §15 DX-1 (4-variant outer) + DX-V5.b (3-variant Unavailable
/// sub-tag).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefusalToken {
    /// Substrate explicitly declined.
    ///
    /// Example call-sites: stcortex `RefuseWriteNoConsumer`,
    /// HABITAT-CONDUCTOR dispatch-budget exhausted, atuin read-quota
    /// refused. The substrate has spoken; the engine listened.
    SubstrateAuthored {
        /// Which substrate authored the refusal.
        substrate_id: SubstrateId,
        /// Substrate-side classification (substrate's own taxonomy).
        substrate_reason: String,
        /// Optional carried payload (e.g., the `EscapeSurfaceProfile`
        /// the substrate would have accepted instead).
        payload: Option<RefusalPayload>,
    },
    /// Engine declined on its own behalf.
    ///
    /// Example call-sites: m32 dispatcher rejection, m9 namespace guard
    /// `NamespaceViolation::CapabilityNotAcknowledged`, m33 verifier
    /// hard-Refuse per D5/D6. The engine is the author.
    EngineAuthored {
        /// Which engine module emitted.
        module_id: ModuleId,
        /// Engine-side classification — preserves the v0.1.0
        /// `RefusalReason` taxonomy under a new alias name.
        engine_reason: EngineRefusalReason,
        /// Optional payload (e.g., verifier verdict context).
        payload: Option<RefusalPayload>,
    },
    /// Operator (Luke @ node 0.A) declined via consent surface.
    ///
    /// Includes "this question is malformed", "not now", "present it
    /// again later" per Plan v2 §15 Round A NA-3 first-class
    /// operator-refusal response.
    OperatorAuthored {
        /// Operator-side classification.
        operator_reason: OperatorRefusalReason,
        /// Optional payload (e.g., requested re-framing).
        payload: Option<RefusalPayload>,
    },
    /// Substrate response is unavailable. THREE distinct sub-tags per
    /// NA-5 + DX-V5.b prevent audit-indistinguishability.
    Unavailable(UnavailableReason),
}

/// Three-variant `Unavailable` sub-tag — DX-V5.b lock.
///
/// Critical NA-5 distinction: `EngineImagined` is the engine
/// *fabricating* substrate silence (the in-engine-receiver-only V5
/// fallback for substrates whose substrate-side schema has not yet
/// shipped); `SubstrateUnreachable` is substrate-side actually exists
/// but cannot be contacted; `SubstrateAuthored` is the substrate
/// emitted "unavailable" itself.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnavailableReason {
    /// Engine fabricated the silence — substrate has no participation
    /// primitive (DX-V5 = in-engine receiver-only fallback). MUST be
    /// observable in telemetry as a first-class event per NA-5
    /// recommendation; downstream observers SHOULD treat this as a
    /// post-v0.2.0 substrate-coupling gap, not as a substrate refusal.
    EngineImagined {
        /// Substrate whose silence is fabricated.
        substrate_id: SubstrateId,
        /// Free-form reason the engine had to imagine
        /// (e.g., "stcortex consumer-trust schema not shipped").
        reason: String,
    },
    /// Substrate exists but cannot be contacted (network / port / lock /
    /// timeout).
    SubstrateUnreachable {
        /// Substrate that is unreachable.
        substrate_id: SubstrateId,
        /// Underlying transport error description.
        transport_reason: String,
    },
    /// Substrate explicitly emitted "unavailable" (e.g., stcortex
    /// `RefuseWriteNoConsumer`, atuin read-quota-exceeded). Distinct
    /// from outer `SubstrateAuthored` because the substrate refused
    /// **availability** rather than refused **the operation**.
    SubstrateAuthored {
        /// Substrate that authored the unavailability.
        substrate_id: SubstrateId,
        /// Substrate-side classification.
        substrate_reason: String,
    },
}

/// Substrate identifier — the 7 v2 §7 substrates per NA-2 expansion
/// (atuin / stcortex / Conductor / CC-5 clocks / Luke + Watcher +
/// RALPH + Cargo build graph) plus the v0.1.0-era `HabitatInjection`
/// substrate for the injection.db channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubstrateId {
    /// atuin WAL shell-history substrate.
    Atuin,
    /// stcortex (SpacetimeDB) substrate.
    Stcortex,
    /// HABITAT-CONDUCTOR dispatch substrate.
    HabitatConductor,
    /// injection.db sqlite substrate.
    HabitatInjection,
    /// CC-5 loop clocks (m11 recency / m13 stcortex-decay / injection-TTL
    /// / atuin-checkpoint / stcortex-pathway-decay).
    Cc5LoopClocks,
    /// The Watcher ☤ persona substrate (synthex-v2 m8 m46-m51).
    Watcher,
    /// RALPH evolutionary substrate.
    Ralph,
    /// Cargo build graph substrate (spacetimedb-sdk sibling-repo +
    /// workspace `Cargo.lock`).
    CargoBuildGraph,
    /// LCM (Loop Engine V2) substrate — for m41 LCM RPC paths.
    Lcm,
    /// SYNTHEX v2 substrate — Nexus emit path.
    SynthexV2,
}

/// Engine module identifier (Cluster D/G/H + cross-cutting emitting
/// modules).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModuleId {
    /// m9 namespace guard.
    M9,
    /// m13 stcortex writer (outbox + write path).
    M13,
    /// m32 dispatcher.
    M32,
    /// m33 verifier.
    M33,
    /// m40 Nexus emit.
    M40,
    /// m41 LCM RPC.
    M41,
    /// m42 stcortex emit (Hebbian coordinator).
    M42,
}

/// Engine refusal taxonomy — alias of the v0.1.0 m32 `RefusalReason`
/// enum, preserved under a v0.2.0-coherent name. v0.1.0 call-sites
/// continue to construct `RefusalReason` directly; v0.2.0 wraps each
/// instance into `RefusalToken::EngineAuthored { engine_reason, ... }`
/// at the Phase 7 call-site classification audit.
pub type EngineRefusalReason = crate::m32_dispatcher::RefusalReason;

/// Operator refusal taxonomy — per Plan v2 §15 Round A NA-3
/// first-class operator-refusal response. Three variants matching the
/// NA-3 documented operator-refusal vocabulary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperatorRefusalReason {
    /// Operator declined because the question / dispatch is malformed
    /// (e.g., wire-contract violation, unknown surface).
    Malformed {
        /// Free-form context explaining the malformation.
        context: String,
    },
    /// Operator declined for now ("not now"; defer to a later session).
    NotNow {
        /// Optional context (e.g., why deferred).
        context: Option<String>,
    },
    /// Operator declined and requested a re-framing of the dispatch.
    RequestReframing {
        /// Operator's suggested re-framing direction (free-form).
        suggested_reframing: String,
    },
}

/// Optional payload for refusals — typed envelopes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefusalPayload {
    /// The escape surface the substrate / engine would have accepted
    /// instead (pairs with `EscapeSurfaceProfile` per ADR
    /// D-S1002127-02).
    AcceptableEscapeSurface(crate::m32_dispatcher::EscapeSurfaceProfile),
    /// Verifier verdict context for engine-authored refusals.
    VerifierContext(String),
    /// Re-framing suggestion for operator-authored refusals.
    SuggestedReframing(String),
    /// Free-form passthrough (escape hatch; use sparingly).
    Freeform(String),
}

impl RefusalToken {
    /// Convenience constructor for substrate-authored refusals.
    #[must_use]
    pub fn substrate_authored(substrate_id: SubstrateId, substrate_reason: String) -> Self {
        Self::SubstrateAuthored {
            substrate_id,
            substrate_reason,
            payload: None,
        }
    }

    /// Convenience constructor for engine-authored refusals — wraps a
    /// flat `EngineRefusalReason` (= v0.1.0 `RefusalReason`) under the
    /// authorship envelope. Phase 7 call-site audit uses this to
    /// classify every existing `RefusalReason` emission.
    #[must_use]
    pub fn engine_authored(module_id: ModuleId, engine_reason: EngineRefusalReason) -> Self {
        Self::EngineAuthored {
            module_id,
            engine_reason,
            payload: None,
        }
    }

    /// Convenience constructor for operator-authored refusals.
    #[must_use]
    pub fn operator_authored(operator_reason: OperatorRefusalReason) -> Self {
        Self::OperatorAuthored {
            operator_reason,
            payload: None,
        }
    }

    /// Convenience constructor for the **EngineImagined** unavailable
    /// case — the NA-5-flagged case where the engine fabricates
    /// substrate silence. First-class telemetry per NA-5
    /// recommendation; downstream observers SHOULD treat as a
    /// post-v0.2.0 substrate-coupling gap, not as a real substrate
    /// refusal.
    #[must_use]
    pub fn unavailable_engine_imagined(substrate_id: SubstrateId, reason: String) -> Self {
        Self::Unavailable(UnavailableReason::EngineImagined {
            substrate_id,
            reason,
        })
    }

    /// Convenience constructor for substrate-unreachable refusals.
    #[must_use]
    pub fn unavailable_substrate_unreachable(
        substrate_id: SubstrateId,
        transport_reason: String,
    ) -> Self {
        Self::Unavailable(UnavailableReason::SubstrateUnreachable {
            substrate_id,
            transport_reason,
        })
    }

    /// Convenience constructor for the substrate-authored unavailable
    /// case — distinct from outer `SubstrateAuthored` (substrate
    /// refused availability vs refused the operation).
    #[must_use]
    pub fn unavailable_substrate_authored(
        substrate_id: SubstrateId,
        substrate_reason: String,
    ) -> Self {
        Self::Unavailable(UnavailableReason::SubstrateAuthored {
            substrate_id,
            substrate_reason,
        })
    }

    /// Return the authoring `SubstrateId` for substrate-authored or
    /// unavailable variants; `None` for engine-authored or
    /// operator-authored.
    ///
    /// Useful for downstream filters (e.g., "show me only stcortex
    /// substrate refusals across all authorship types").
    #[must_use]
    pub const fn substrate_id(&self) -> Option<SubstrateId> {
        match self {
            Self::SubstrateAuthored { substrate_id, .. }
            | Self::Unavailable(
                UnavailableReason::EngineImagined { substrate_id, .. }
                | UnavailableReason::SubstrateUnreachable { substrate_id, .. }
                | UnavailableReason::SubstrateAuthored { substrate_id, .. },
            ) => Some(*substrate_id),
            Self::EngineAuthored { .. } | Self::OperatorAuthored { .. } => None,
        }
    }

    /// Return `true` if this refusal originated from substrate speech
    /// (either outer `SubstrateAuthored` or any `Unavailable` with a
    /// substrate-speech sub-tag = `SubstrateAuthored`).
    ///
    /// Use this to distinguish *substrate-said-no* from
    /// *engine-said-no-for-the-substrate* (the NA-5
    /// audit-distinguishability check).
    #[must_use]
    pub const fn is_substrate_authored(&self) -> bool {
        matches!(
            self,
            Self::SubstrateAuthored { .. }
                | Self::Unavailable(UnavailableReason::SubstrateAuthored { .. })
        )
    }

    /// Return `true` if this refusal is engine-imagined (the NA-5 case
    /// where the engine fabricated substrate silence). Downstream
    /// telemetry MUST flag these as substrate-coupling gaps.
    #[must_use]
    pub const fn is_engine_imagined(&self) -> bool {
        matches!(
            self,
            Self::Unavailable(UnavailableReason::EngineImagined { .. })
        )
    }
}

#[cfg(test)]
mod tests;
