//! W1 — m16 Heartbeat transport client (v0.2.2+).
//!
//! Per Plan v2 — Source-Verified Integration S1004590 (Zen PARTIAL APPROVE
//! / AMEND, 2026-05-24T102556Z), this module ships the **engine-side
//! enrichment wrapper** + **transport client** for m16 substrate-drift
//! heartbeats to synthex-v2 `:8092/v3/heartbeat`.
//!
//! ## Why an enrichment wrapper
//!
//! The minimal m16 [`Heartbeat`] (`{ emitted_at_ms, cycle }`) carries only
//! liveness. The wire envelope adds engine-side context — `boot_id`
//! (NA-10' replay semantics), `heartbeat_source`, `instance_id`,
//! `skew_summary` (derived from the last `DetectionResult`),
//! `alert_budget_remaining` — without bloating the in-process `Heartbeat`
//! struct itself.
//!
//! ## NA-5 audit-distinguishability contract
//!
//! Per ADR D-S1004XXX-04 + Plan v2 §15 DX-V5.b, every transport failure
//! routes through a typed [`RefusalToken`] sub-tag:
//!
//! | HTTP outcome | RefusalToken sub-tag | Audit meaning |
//! |---|---|---|
//! | 200 | (none — Ok) | Substrate accepted |
//! | 501 | `SubstrateUnreachable("synthex_v2_phase3_pending")` | Honest-501 source (per ZA-2: source is `c9eeb75` honest-501; running daemon may still be lying-200 — drift check is acceptance-gate cond 0, not transport-client concern) |
//! | 503 | `SubstrateUnreachable("r13_cold_start_503")` | Substrate signalled deferral |
//! | 4xx/5xx other | `SubstrateAuthored(http_<code>)` | Substrate explicitly refused |
//! | Transport error | `SubstrateUnreachable(transport_<err>)` | Network/socket failure |
//! | V5 gate `NotShipped` | `EngineImagined(synthex_v2_engine_imagined_not_shipped)` | No HTTP call made — substrate participation primitive not shipped |
//!
//! ## What this module does NOT do
//!
//! - Does NOT call the network in tests by default (tests use mock endpoints).
//! - Does NOT retry on failure — caller's responsibility per
//!   [`AlertBudget`](super::AlertBudget) rate-limiting upstream.
//! - Does NOT include the synthex-v2 source/deploy drift check
//!   (Plan v2 ZA-3 NA-4 acceptance gate cond 0) — that lives in the
//!   live-test harness, not transport.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::Heartbeat;
use crate::refusal_token::{RefusalToken, SubstrateId};
use crate::substrate_trust::SubstrateTrust;

/// Engine-side enrichment wrapper for the minimal [`Heartbeat`] wire shape.
///
/// Wraps the in-process [`Heartbeat`] with operator-observability context
/// (boot ID, source tag, skew summary, alert budget) per Plan v2 § S2
/// amendment 2 (ZA-1 + CONV-3 escalation).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeartbeatWireEnvelope {
    /// The minimal in-process heartbeat from [`Heartbeat`].
    pub heartbeat: Heartbeat,
    /// Emitter tag (canonical: `workflow-trace::m16_substrate_drift_canary`).
    /// Owned `String` so deserialized envelopes do not pin caller lifetimes.
    pub heartbeat_source: String,
    /// Process-boot identifier (NA-10' replay semantics — substrate tracks
    /// `(boot_id, last_seen_cycle)` per emitter so cycle resets are
    /// distinguishable from replay attacks).
    pub boot_id: String,
    /// Instance identifier (host/pid/lane).
    pub instance_id: String,
    /// Last observed skew envelope summary (derived engine-side from the
    /// most recent [`DetectionResult`](super::DetectionResult)).
    pub skew_summary: SkewSummary,
    /// Alert budget remaining at emit time (engine-side rate-limiter state).
    pub alert_budget_remaining: u16,
}

/// Compact skew snapshot for the wire — derived from the last detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SkewSummary {
    /// Maximum observed pair-skew across all clock sources in ms.
    pub max_observed_skew_ms: u64,
    /// Number of pairs sampled in the last detection cycle.
    pub samples_observed: u16,
    /// Whether the last detection emitted any refusal-token events.
    pub had_refusals: bool,
}

/// Substrate-side ACK shape returned on successful 200.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeartbeatAck {
    /// Substrate echoes the emitter cycle for end-to-end correlation.
    pub cycle_acked: u64,
    /// Substrate wall-clock at receive time (substrate's frame; engine
    /// vs substrate clock skew is itself observable via this field).
    pub synthex_v2_observed_at_ms: u64,
}

/// Blocking HTTP transport client for m16 → synthex-v2 `/v3/heartbeat`.
///
/// V0.2.2+ contained module: one new file + tests. Does NOT depend on the
/// synthex-v2-side endpoint landing — until it does, every emit will route
/// through [`RefusalToken::Unavailable`] honestly per the table above.
pub struct HeartbeatTransport {
    client: reqwest::blocking::Client,
    endpoint: String,
    substrate_trust: Arc<SubstrateTrust>,
}

impl HeartbeatTransport {
    /// Construct with explicit endpoint URL + shared V5 substrate-trust gate.
    ///
    /// The V5 gate enforces NA-5 audit-distinguishability: if
    /// `substrate_trust.is_substrate_imagined_for(SynthexV2) == true`
    /// (i.e., substrate participation status = `NotShipped`), `send()`
    /// short-circuits to [`RefusalToken::EngineImagined`] without any
    /// HTTP call. This prevents fabricated SubstrateAuthored noise from
    /// engine-side fallback when the substrate primitive doesn't exist.
    #[must_use]
    pub fn new(endpoint: String, substrate_trust: Arc<SubstrateTrust>) -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
            endpoint,
            substrate_trust,
        }
    }

    /// Send a wire envelope to synthex-v2. Returns `Ok(HeartbeatAck)` on
    /// 200, or `Err(RefusalToken)` with NA-5-distinguished sub-tag per
    /// the failure-table in module docs.
    ///
    /// # Errors
    ///
    /// Returns `Err(RefusalToken::Unavailable(_))` for every non-200
    /// outcome — see module-level docs for the full table. Caller MUST
    /// match on the sub-tag (NA-5 audit-distinguishability) rather than
    /// treating all errors as equivalent.
    pub fn send(&self, envelope: &HeartbeatWireEnvelope) -> Result<HeartbeatAck, RefusalToken> {
        // V5 gate: short-circuit if substrate is NotShipped (engine-imagined
        // fallback per NA-5). Avoids fabricating SubstrateAuthored noise.
        if self.substrate_trust.is_substrate_imagined_for(SubstrateId::SynthexV2) {
            return Err(RefusalToken::unavailable_engine_imagined(
                SubstrateId::SynthexV2,
                "synthex_v2_engine_imagined_not_shipped".to_owned(),
            ));
        }

        match self.client.post(&self.endpoint).json(envelope).send() {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    resp.json::<HeartbeatAck>().map_err(|e| {
                        // 200 but body unparseable — substrate-authored
                        // malformation (substrate sent something but it
                        // doesn't match the contract).
                        RefusalToken::unavailable_substrate_authored(
                            SubstrateId::SynthexV2,
                            format!("substrate_authored:ack_parse_{e}"),
                        )
                    })
                } else if status.as_u16() == 501 {
                    // ZA-2 + CONV-1: synthex-v2 source ships honest-501
                    // since c9eeb75; running daemon may still be lying-200
                    // (operator must run source/deploy drift check per
                    // ZA-3 NA-4 acceptance gate cond 0).
                    Err(RefusalToken::unavailable_substrate_unreachable(
                        SubstrateId::SynthexV2,
                        "synthex_v2_phase3_pending".to_owned(),
                    ))
                } else if status.as_u16() == 503 {
                    // Substrate signalled deferral (R13 cold-start or
                    // overload).
                    Err(RefusalToken::unavailable_substrate_unreachable(
                        SubstrateId::SynthexV2,
                        "r13_cold_start_503".to_owned(),
                    ))
                } else {
                    // 4xx/5xx other — substrate explicitly refused.
                    Err(RefusalToken::unavailable_substrate_authored(
                        SubstrateId::SynthexV2,
                        format!("substrate_authored:http_{}", status.as_u16()),
                    ))
                }
            }
            Err(e) => {
                // Transport-layer failure (network, socket, DNS, timeout).
                Err(RefusalToken::unavailable_substrate_unreachable(
                    SubstrateId::SynthexV2,
                    format!("substrate_unreachable:transport_{e}"),
                ))
            }
        }
    }
}

/// Glue: convert a `DetectionResult` from `DriftDetector::detect()` plus
/// process-wide context into a wire envelope, send via the transport,
/// return the typed outcome.
///
/// Per Plan v2 — Source-Verified Integration S1004590 § "v0.2.2+ horizon
/// item 1" (call-site integration). Bridges the call-driven `detect()`
/// surface (m16 v0.2.0 KEYSTONE) with the wire-driven `HeartbeatTransport`
/// surface (W1, v0.2.2+ commit `2e9edff`).
///
/// `boot_id` + `instance_id` are caller-injected (m16 has no
/// process-wide UUID generator; this is the integration point).
/// `alert_budget_remaining` is caller-derived from the budget state at
/// emit time (m16's `AlertBudget` does not expose this directly per
/// design; callers compute via `(min_interval_ms.saturating_sub(now -
/// last_fired))` clamped to `u16` if a budget primitive is needed).
///
/// # Errors
///
/// Returns `Err(RefusalToken::Unavailable(_))` per the failure-table in
/// the [`HeartbeatTransport::send`] docs — V5 gate, transport, 501, 503,
/// 4xx/5xx, parse-fail all route through NA-5-distinguishable sub-tags.
pub fn emit_detection_to_transport(
    result: &super::DetectionResult,
    transport: &HeartbeatTransport,
    boot_id: String,
    instance_id: String,
    alert_budget_remaining: u16,
) -> Result<HeartbeatAck, RefusalToken> {
    // Derive SkewSummary from the DetectionResult's samples + events
    // (engine-side enrichment per Plan v2 § S2 amendment 2).
    let max_observed_skew_ms = result
        .samples
        .iter()
        .flat_map(|a| {
            result
                .samples
                .iter()
                .filter(move |b| a.source != b.source)
                .map(move |b| super::pair_skew_ms(a.clock_value_ms, b.clock_value_ms))
        })
        .max()
        .unwrap_or(0);
    let skew_summary = SkewSummary {
        max_observed_skew_ms,
        samples_observed: u16::try_from(result.samples.len()).unwrap_or(u16::MAX),
        had_refusals: !result.events.is_empty(),
    };

    let envelope = HeartbeatWireEnvelope {
        heartbeat: result.heartbeat,
        heartbeat_source: "workflow-trace::m16_substrate_drift_canary".to_owned(),
        boot_id,
        instance_id,
        skew_summary,
        alert_budget_remaining,
    };

    transport.send(&envelope)
}

/// Convenience tick wrapper: invoke `DriftDetector::detect()` for the
/// given `now_ms`, then emit the result via `emit_detection_to_transport`.
///
/// Per Plan v2 — Source-Verified Integration S1004590 § "v0.2.2+ horizon
/// item 1" extension (call-loop wire). m16 has no internal scheduler
/// today (`detect()` is call-driven); this wrapper is the smallest
/// integration that combines per-cycle detection + transport emission
/// into one operator-facing call.
///
/// The caller is expected to drive the scheduling (`tick(now_ms)` per
/// cycle on whatever cadence the operator chooses — typically 1 Hz).
/// Boot ID and instance ID are caller-injected (same contract as
/// [`emit_detection_to_transport`]).
///
/// # Errors
///
/// Same failure-table as [`HeartbeatTransport::send`] — V5 gate,
/// transport, 501, 503, 4xx/5xx, parse-fail all route through
/// NA-5-distinguishable sub-tags.
pub fn tick_and_emit(
    detector: &mut super::DriftDetector,
    now_ms: u64,
    transport: &HeartbeatTransport,
    boot_id: String,
    instance_id: String,
    alert_budget_remaining: u16,
) -> Result<HeartbeatAck, RefusalToken> {
    let result = detector.detect(now_ms);
    emit_detection_to_transport(
        &result,
        transport,
        boot_id,
        instance_id,
        alert_budget_remaining,
    )
}

#[cfg(test)]
mod tests;
