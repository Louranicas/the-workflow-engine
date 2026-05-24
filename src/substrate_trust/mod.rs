#![allow(clippy::doc_markdown)] // habitat-conventional per workspace clippy config
//! V5 substrate-mediated trust — engine-side primitive (Plan v2 v0.2.0
//! §3 Phase 11 per DX-V5 = full cross-habitat + DX-V5.b 3-variant
//! Unavailable sub-tag shipped V1).
//!
//! v0.2.0 ships the ENGINE HALF: a typed `SubstrateTrust` accumulator
//! reading per-substrate trust signals + a `substrate_participation_status`
//! accessor enforcing the NA-5 audit-distinguishability contract (so a
//! substrate that has NOT YET shipped its participation primitive
//! cannot be mistaken for one that has).
//!
//! Substrate-side schemas (stcortex consumer-trust score, Conductor
//! dispatch-budget table, atuin read-quota, ORAC reputation hooks,
//! synthex-v2 r13-state-aware verifier weighting) are post-v0.2.0
//! per ADR D-S1004XXX-05 + §11 per-substrate consent gradient.
//!
//! ## NA-5 audit-distinguishability primary contract
//!
//! When `substrate_participation_status == NotShipped`, the engine
//! MUST emit `RefusalToken::Unavailable(EngineImagined)` rather than
//! synthesise a `SubstrateAuthored` value. This prevents v0.2.0's
//! in-engine-receiver-only fallback from looking substrate-authored in
//! audit. The `is_substrate_imagined_for` helper makes the
//! audit-distinguishability check first-class.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::refusal_token::{RefusalToken, SubstrateId};

/// Per-substrate participation lifecycle — per NA-5 the engine MUST
/// distinguish a substrate that has not shipped its participation
/// primitive (`NotShipped`) from one that is in active rollout
/// (`Shipping`) from one that is fully live (`Live`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubstrateParticipationStatus {
    /// Substrate-side primitive does not exist. Engine queries return
    /// `RefusalToken::Unavailable(EngineImagined)` — engine
    /// fabricating substrate silence per NA-5 audit-distinguishability
    /// contract.
    NotShipped,
    /// Substrate-side primitive exists but is in shadow / soak phase.
    /// Engine queries may return real values OR `EngineImagined`
    /// depending on per-substrate signal availability; engines SHOULD
    /// log the distinction.
    Shipping,
    /// Substrate-side primitive is live. Engine queries return
    /// substrate-authored values via the appropriate channel
    /// (`SubstrateAuthored` or `Unavailable::SubstrateAuthored`).
    Live,
}

impl Default for SubstrateParticipationStatus {
    /// Default at v0.2.0 ship = `NotShipped`. Operators flip per
    /// substrate as substrate-side primitives land per §11 consent
    /// gradient (stcortex HIGH / Conductor HIGH / synthex-v2 HIGH /
    /// ORAC MED / atuin UNKNOWN).
    fn default() -> Self {
        Self::NotShipped
    }
}

/// Per-substrate trust signal as received (Live) or imagined
/// (NotShipped/Shipping) by the engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustValue {
    /// Substrate-authored trust score in `[0.0, 1.0]` (Live status only).
    Score(f64),
    /// Substrate-authored binary trust flag (Live status only).
    Flag(bool),
    /// Substrate-authored budget remainder (Live status only).
    BudgetRemaining(i64),
    /// Engine-imagined silence: substrate-side primitive not shipped /
    /// not reachable. Per NA-5, this must be audit-distinguishable
    /// from substrate-authored values.
    Unavailable,
}

/// Per-substrate trust entry: (status, value).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrustEntry {
    /// Lifecycle status (NotShipped / Shipping / Live).
    pub status: SubstrateParticipationStatus,
    /// Current trust value (Unavailable while NotShipped).
    pub value: TrustValue,
}

impl TrustEntry {
    /// Convenience constructor for the v0.2.0 default state:
    /// `NotShipped` + `Unavailable`.
    #[must_use]
    pub fn not_shipped() -> Self {
        Self {
            status: SubstrateParticipationStatus::NotShipped,
            value: TrustValue::Unavailable,
        }
    }

    /// Convenience constructor for a Live substrate emitting a score.
    #[must_use]
    pub fn live_score(score: f64) -> Self {
        Self {
            status: SubstrateParticipationStatus::Live,
            value: TrustValue::Score(score),
        }
    }

    /// Convenience constructor for a Live substrate emitting a flag.
    #[must_use]
    pub fn live_flag(flag: bool) -> Self {
        Self {
            status: SubstrateParticipationStatus::Live,
            value: TrustValue::Flag(flag),
        }
    }

    /// Convenience constructor for a Live substrate emitting a budget.
    #[must_use]
    pub fn live_budget(remaining: i64) -> Self {
        Self {
            status: SubstrateParticipationStatus::Live,
            value: TrustValue::BudgetRemaining(remaining),
        }
    }
}

/// V5 engine-side substrate trust accumulator. Per ADR D-S1004XXX-05 + Plan v2 §3 Phase 11 step 1: workflow-trace ships the consumer side; substrate-side schemas (stcortex / Conductor / atuin / ORAC / synthex-v2) are post-v0.2.0 per §11.
#[derive(Debug, Clone, Default)]
pub struct SubstrateTrust {
    entries: HashMap<SubstrateId, TrustEntry>,
}

impl SubstrateTrust {
    /// Construct an empty trust accumulator. Per-substrate queries
    /// return `NotShipped` + `Unavailable` until `set` is called.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the trust entry for a substrate. Returns the previous entry
    /// (if any) so callers can detect overwrites — M3 post-v0.2.0
    /// hardening (parity with `BackPressureRegistry::set_mode`). When
    /// `Some`, two emitters set conflicting trust for the same
    /// substrate; downstream observers SHOULD log the conflict.
    ///
    /// **N1 silent-failure-hunter regression fix:** `#[must_use]`
    /// forces callers to acknowledge the overwrite signal — the exact
    /// failure mode M3 was authored to prevent (a caller writing
    /// `t.set(id, entry);` would silently discard the conflict
    /// indicator). Compiler-enforced now.
    #[must_use = "set() returns the previous TrustEntry; conflicting emitters must be logged per M3 contract"]
    pub fn set(
        &mut self,
        substrate: SubstrateId,
        entry: TrustEntry,
    ) -> Option<TrustEntry> {
        self.entries.insert(substrate, entry)
    }

    /// Look up the trust entry for a substrate. Returns the v0.2.0
    /// default (`NotShipped` + `Unavailable`) when the substrate has
    /// not been explicitly set — per NA-5 the consumer cannot
    /// distinguish "never set" from "explicitly NotShipped" by design;
    /// both route through the EngineImagined refusal path.
    #[must_use]
    pub fn get(&self, substrate: SubstrateId) -> TrustEntry {
        self.entries
            .get(&substrate)
            .cloned()
            .unwrap_or_else(TrustEntry::not_shipped)
    }

    /// Lifecycle status for a substrate (per NA-5 audit-distinguishability
    /// primary check). `NotShipped` means engine queries route through
    /// `RefusalToken::Unavailable(EngineImagined)`; `Live` means
    /// substrate-authored values are honoured.
    #[must_use]
    pub fn substrate_participation_status(
        &self,
        substrate: SubstrateId,
    ) -> SubstrateParticipationStatus {
        self.entries
            .get(&substrate)
            .map_or(SubstrateParticipationStatus::default(), |e| e.status)
    }

    /// True iff the engine MUST emit `RefusalToken::Unavailable(EngineImagined)`
    /// for queries to this substrate. Equivalent to
    /// `substrate_participation_status(s) == NotShipped`.
    ///
    /// This is the executable NA-5 audit-distinguishability primary
    /// check.
    #[must_use]
    pub fn is_substrate_imagined_for(&self, substrate: SubstrateId) -> bool {
        matches!(
            self.substrate_participation_status(substrate),
            SubstrateParticipationStatus::NotShipped
        )
    }

    /// Convenience: construct a `RefusalToken` for an unavailable
    /// substrate trust query, choosing the correct NA-5 sub-tag per
    /// participation status. **Zen #2 post-v0.2.0 hardening:** the
    /// `reason` String is prefixed with the participation-status
    /// provenance tag (`engine_imagined:` / `substrate_unreachable:` /
    /// `substrate_authored:`) so log-grep audits can distinguish the
    /// three branches without losing the variant information that
    /// NA-5 was built to preserve. The structural distinction via the
    /// enum variant is preserved unchanged; the prefix is operator-
    /// observability additive.
    ///
    /// `NotShipped` → `Unavailable(EngineImagined)` (engine fabricating
    /// silence). `Shipping` → `Unavailable(SubstrateUnreachable)`
    /// (substrate exists but transient-unavailable). `Live` →
    /// `Unavailable(SubstrateAuthored)` (substrate explicitly declined
    /// to answer).
    #[must_use]
    pub fn refusal_for_unavailable(
        &self,
        substrate: SubstrateId,
        reason: &str,
    ) -> RefusalToken {
        // Zen #2 post-v0.2.0 hardening: prefix the reason with the
        // status tag so the three branches are distinguishable in
        // log-grep + textual-audit even though the structural enum
        // variant already differentiates them.
        match self.substrate_participation_status(substrate) {
            SubstrateParticipationStatus::NotShipped => {
                RefusalToken::unavailable_engine_imagined(
                    substrate,
                    format!("engine_imagined:{reason}"),
                )
            }
            SubstrateParticipationStatus::Shipping => {
                RefusalToken::unavailable_substrate_unreachable(
                    substrate,
                    format!("substrate_unreachable:{reason}"),
                )
            }
            SubstrateParticipationStatus::Live => {
                RefusalToken::unavailable_substrate_authored(
                    substrate,
                    format!("substrate_authored:{reason}"),
                )
            }
        }
    }

    /// Number of explicitly-set substrate entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True iff no substrate has been explicitly set.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests;
