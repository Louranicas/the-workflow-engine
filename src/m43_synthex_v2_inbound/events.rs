//! m43 inbound event types — 5-variant SX2 → WFE event catalogue per
//! [[Wiring 02b — NexusEvent Inbound (SX2 → WFE)]].

use serde::{Deserialize, Serialize};

/// SX2-side emitter identification. Each variant maps to a specific
/// synthex-v2 module that can push an inbound event to WFE.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum InboundSource {
    /// synthex-v2 m46 Observer (1 Hz tensor-snapshot consumer).
    M46Observer,
    /// synthex-v2 m47 Critic (hypothesis generator over m46 anomalies).
    M47Critic,
    /// synthex-v2 m51 Ember Protector (7-trait gate, blocks proposals).
    M51EmberProtector,
    /// synthex-v2 m10 HTTP poller (liveness watcher for WFE heartbeats).
    M10HttpPoller,
}

/// Event-kind taxonomy. Each kind has a per-kind payload schema (carried
/// as `serde_json::Value` today; future refinement = typed-per-kind).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum InboundEventKind {
    /// SX2 m46 observed WFE clock drift exceeding baseline. Payload:
    /// `{ severity, observed_drift_ms, since_ms }`.
    WfeDriftObserved,
    /// SX2 m10 has not received a WFE heartbeat for N intervals.
    /// Payload: `{ missed_count, last_heartbeat_at_ms, last_boot_id }`.
    WfeSilenceObserved,
    /// SX2 m47 Critic generated a hypothesis about the WFE drift.
    /// Payload: `{ hypothesis_text, supporting_observations_count }`.
    WfeDriftHypothesis,
    /// SX2 m51 Ember Protector blocked a WFE-affecting proposal at the
    /// 7-trait gate. Payload: `{ proposal_id, blocking_traits, summary }`.
    WfeProposalBlocked,
    /// SX2 m10 confirms WFE silence has persisted past tolerance.
    /// Payload: `{ status: "Crashed" | "Drained" | "NeverStarted", duration_ms }`.
    /// Drives bilateral V5 `WorkflowTraceParticipationStatus` update on
    /// SX2 side (substrate-authorship request per NA-3').
    WfeUnreachablePersisting,
}

/// Inbound event envelope — SX2 emits one of these per push.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InboundEvent {
    /// Which SX2-side module emitted this event.
    pub source: InboundSource,
    /// What kind of event (semantic shape of `data`).
    pub kind: InboundEventKind,
    /// SX2 wall-clock at emit time (substrate's frame).
    pub emitted_at_ms: u64,
    /// SX2 boot identifier (NA-10' replay-safety — WFE tracks
    /// `(boot_id, last_seen_at_ms)` per SX2 emitter so cycle resets are
    /// distinguishable from replay attacks).
    pub boot_id: String,
    /// Opaque payload per `kind` schema. Typed at handler dispatch time;
    /// untyped on the wire today (future refinement: per-kind structs).
    pub data: serde_json::Value,
}
