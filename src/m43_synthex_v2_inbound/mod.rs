//! m43 — synthex-v2 → WFE inbound event consumer (v0.2.2+).
//!
//! Per Plan v2 — Source-Verified Integration S1004590 § S4 (NA-2'
//! resolution) + [[Wiring 02b — NexusEvent Inbound (SX2 → WFE)]].
//! Library-only scaffold; HTTP listener deferred to future bin-level
//! integration per Zen "HOLD source expansion" guidance (no axum/actix
//! runtime dep added here).
//!
//! ## What this module ships (v0.2.2+ first cut)
//!
//! - [`InboundEvent`] envelope shape SX2 emits to WFE
//! - [`InboundEventKind`] 5-variant catalogue of event semantics
//! - [`InboundSource`] 4-variant catalogue of SX2-side emitters
//! - [`InboundHandler`] dispatch trait + default impl
//! - [`HandlerOutcome`] + [`HandlerError`] typed results
//!
//! ## What this module DOES NOT ship (deferred)
//!
//! - HTTP listener (no axum/actix dep) — future bin-level wire
//! - Live integration with m11 fitness decay (handler routes are stubs
//!   per NA-4 acceptance cond 6 SPEC; production m11 hook is a separate
//!   change)
//! - Bilateral V5 update path (NA-3' depends on substrate-side
//!   `WorkflowTraceParticipationStatus` shipping; handler logs the event
//!   today, doesn't update engine V5 state)
//!
//! ## Bilateral V5 symmetry (NA-3' + NA-5 reciprocity)
//!
//! Per ADR D-S1004XXX-04, refusals from WFE-side back to SX2 follow the
//! same NA-5 sub-tag taxonomy used outbound. Handler can synthesise
//! [`crate::refusal_token::RefusalToken::SubstrateAuthored`] (from SX2's
//! perspective, WFE is the substrate) when WFE refuses an inbound event
//! (draining / malformed / oversized / hypothesis-rejected).

use serde::{Deserialize, Serialize};

pub use events::{InboundEvent, InboundEventKind, InboundSource};
pub use handler::{HandlerError, HandlerOutcome, InboundHandler};

mod events;
mod handler;

#[cfg(test)]
mod tests;

/// Default WFE-side inbound port reservation per Wiring 02b spec.
/// Provisional — confirm no collision with the 14-service habitat table
/// in `~/claude-code-workspace/CLAUDE.md` before binding.
pub const DEFAULT_INBOUND_PORT: u16 = 8094;

/// Server configuration. Lives here as a typed contract today; the
/// concrete listener implementation is a future bin-level change per
/// Zen "HOLD source expansion".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InboundServerConfig {
    /// Port to bind. Default [`DEFAULT_INBOUND_PORT`].
    pub port: u16,
    /// Maximum JSON body size accepted (DoS guard). Default 64 KiB.
    pub max_body_bytes: usize,
    /// Whether to log full event payloads (verbose; off by default for
    /// production).
    pub log_payloads: bool,
}

impl Default for InboundServerConfig {
    fn default() -> Self {
        Self {
            port: DEFAULT_INBOUND_PORT,
            max_body_bytes: 64 * 1024,
            log_payloads: false,
        }
    }
}
