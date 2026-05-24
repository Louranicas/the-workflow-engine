//! m43 inbound event handler — typed dispatch + outcome routing.
//!
//! Today's handler logs each event and returns a typed outcome — the
//! production hooks into m11 fitness decay / m16 budget hint / V5
//! bilateral mirror are stubs per Wiring 02b spec, awaiting follow-up
//! amendments to the consumer modules. See [[Wiring 02b — NexusEvent
//! Inbound (SX2 → WFE)]] § "Honest residuals".

use serde::{Deserialize, Serialize};

use super::events::{InboundEvent, InboundEventKind};
use crate::refusal_token::{RefusalToken, SubstrateId};

/// Typed handler outcome. Each variant represents a side-effect that
/// the handler MAY have taken (today most are `Logged` no-ops; future
/// amendments wire production effects per Wiring 02b spec).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandlerOutcome {
    /// Event logged for operator visibility; no production effect.
    Logged,
    /// m11 fitness decay weight hint applied (production wire — stub today).
    M11DecayHintApplied,
    /// m16 alert-budget reset hint applied (production wire — stub today).
    M16BudgetHintApplied,
    /// Engine-side V5 bilateral mirror updated (substrate told us SX2's
    /// view of WFE participation status; production wire — stub today).
    V5BilateralMirrorUpdated,
    /// Event was a no-op (e.g., advisory hypothesis with no production
    /// action).
    NoOpAdvisory,
}

/// Typed handler error — distinct from a [`RefusalToken`] (which the
/// handler MAY synthesise via [`InboundHandler::refusal_for`] when WFE
/// refuses an inbound event per NA-5 reciprocity).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandlerError {
    /// Event payload did not match the schema expected for its `kind`.
    PayloadShapeMismatch {
        /// Which kind was expected.
        kind: InboundEventKind,
        /// Free-form description of the mismatch.
        detail: String,
    },
    /// Event was structurally valid but rejected by handler policy
    /// (e.g., m47 hypothesis that contradicts known invariants).
    PolicyRejected {
        /// Which kind was rejected.
        kind: InboundEventKind,
        /// Reason for rejection.
        reason: String,
    },
    /// Handler is in drain mode (graceful shutdown); event NOT processed.
    Draining,
}

/// Default inbound handler. Library-only today; production wires (m11
/// decay hint, m16 budget hint, V5 bilateral) are stubs that return the
/// typed outcome without taking the named effect.
#[derive(Debug, Clone, Default)]
pub struct InboundHandler {
    /// Whether the handler is in drain mode (graceful shutdown).
    draining: bool,
}

impl InboundHandler {
    /// Construct a fresh handler (not draining).
    #[must_use]
    pub fn new() -> Self {
        Self { draining: false }
    }

    /// Mark the handler as draining. Subsequent [`Self::dispatch`] calls
    /// return `Err(HandlerError::Draining)`.
    pub fn drain(&mut self) {
        self.draining = true;
    }

    /// True iff the handler is in drain mode.
    #[must_use]
    pub const fn is_draining(&self) -> bool {
        self.draining
    }

    /// Dispatch an inbound event to its typed handler arm. Returns the
    /// typed [`HandlerOutcome`] on success, or [`HandlerError`] on
    /// dispatch failure.
    ///
    /// # Errors
    ///
    /// - [`HandlerError::Draining`] if the handler is in drain mode.
    /// - [`HandlerError::PayloadShapeMismatch`] if the event's payload
    ///   does not match the schema expected for its kind. (Today this
    ///   variant fires only when explicit per-kind validators land —
    ///   most kinds accept any `serde_json::Value` opaquely.)
    /// - [`HandlerError::PolicyRejected`] if a per-kind policy rejects
    ///   the event (today no policies are configured).
    pub fn dispatch(&self, event: &InboundEvent) -> Result<HandlerOutcome, HandlerError> {
        if self.draining {
            return Err(HandlerError::Draining);
        }
        match event.kind {
            InboundEventKind::WfeDriftObserved => Self::route_drift_observed(event),
            InboundEventKind::WfeSilenceObserved => Self::route_silence_observed(event),
            InboundEventKind::WfeDriftHypothesis => Self::route_drift_hypothesis(event),
            InboundEventKind::WfeProposalBlocked => Self::route_proposal_blocked(event),
            InboundEventKind::WfeUnreachablePersisting => Self::route_unreachable_persisting(event),
        }
    }

    /// Synthesise a RefusalToken when WFE refuses an inbound event per
    /// NA-5 reciprocity (substrate-authored sub-tag, from SX2's
    /// perspective WFE IS the substrate when SX2 → WFE).
    #[must_use]
    pub fn refusal_for(err: &HandlerError) -> RefusalToken {
        let reason = match err {
            HandlerError::Draining => "wfe_draining".to_owned(),
            HandlerError::PayloadShapeMismatch { detail, .. } => {
                format!("wfe_malformed_inbound:{detail}")
            }
            HandlerError::PolicyRejected { reason, .. } => {
                format!("wfe_hypothesis_rejected:{reason}")
            }
        };
        // SubstrateId::SynthexV2 from SX2's perspective is the SENDER,
        // not the substrate — but the RefusalToken needs A substrate id;
        // we use SynthexV2 to attribute the refusal-source. (Per NA-5
        // reciprocity, the substrate_authored sub-tag's `substrate_id`
        // names the substrate that AUTHORED the refusal. When WFE
        // refuses an SX2 event, WFE IS the substrate from SX2's POV;
        // SynthexV2 is the closest available variant until a `WFE`
        // variant lands in v0.3.0+.)
        RefusalToken::unavailable_substrate_authored(SubstrateId::SynthexV2, reason)
    }

    // Route functions are associated (no &self) by design — handler
    // state (drain flag) is checked in dispatch(); per-kind routing is
    // event-data-driven only today. Result return preserved because
    // future per-kind validation will fail (PayloadShapeMismatch /
    // PolicyRejected) without changing the public signature.

    #[allow(clippy::unnecessary_wraps)]
    fn route_drift_observed(_event: &InboundEvent) -> Result<HandlerOutcome, HandlerError> {
        // Production wire (stub): would call m11.apply_decay_hint(...)
        // when m11 exposes that API per Wiring 02b residual.
        Ok(HandlerOutcome::M11DecayHintApplied)
    }

    #[allow(clippy::unnecessary_wraps)]
    fn route_silence_observed(_event: &InboundEvent) -> Result<HandlerOutcome, HandlerError> {
        // Production wire (stub): would call m16.budget_reset_hint(...)
        // when m16 exposes that API per Wiring 02b residual.
        Ok(HandlerOutcome::M16BudgetHintApplied)
    }

    #[allow(clippy::unnecessary_wraps)]
    fn route_drift_hypothesis(_event: &InboundEvent) -> Result<HandlerOutcome, HandlerError> {
        // Advisory; no production wire today.
        Ok(HandlerOutcome::NoOpAdvisory)
    }

    #[allow(clippy::unnecessary_wraps)]
    fn route_proposal_blocked(_event: &InboundEvent) -> Result<HandlerOutcome, HandlerError> {
        // Audit log only; no production wire today.
        Ok(HandlerOutcome::Logged)
    }

    #[allow(clippy::unnecessary_wraps)]
    fn route_unreachable_persisting(
        _event: &InboundEvent,
    ) -> Result<HandlerOutcome, HandlerError> {
        // Production wire (stub): would update engine-side V5 bilateral
        // mirror when substrate-side WorkflowTraceParticipationStatus
        // ships per NA-3'.
        Ok(HandlerOutcome::V5BilateralMirrorUpdated)
    }
}
