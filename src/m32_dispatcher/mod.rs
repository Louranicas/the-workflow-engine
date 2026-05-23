//! `m32_conductor_dispatcher` — dispatch selected workflows via the
//! HABITAT-CONDUCTOR endpoint (no direct exec). Cluster G · L7.
//!
//! **Hard refusal:** m32 NEVER spawns a process / shell / fleet pane
//! directly. Every dispatch surfaces as a structured request to
//! Conductor, which performs the actual execution under its own gates
//! (escape-surface profile, human-acceptance signature, audit).
//!
//! # Routing
//!
//! Production routing target is the Conductor RPC method
//! [`CONDUCTOR_DISPATCH_METHOD`] (`"lcm.loop.create"`), NOT `lcm.deploy`.
//! The trait [`ConductorClient::submit`] is the only egress; tests mock it.
//!
//! # AP-V7-08 self-dispatch refusal
//!
//! m32 MUST NOT execute the dispatch itself; it composes the
//! [`ConductorClient`] call. Additionally, m32 refuses to dispatch any
//! workflow whose `proposal_id` matches the `forbidden_step_targets` list
//! supplied by the caller. See [`self_dispatch_guard`].
//!
//! # `EscapeSurfaceProfile` cardinality
//!
//! Compile-time-asserted at 7 variants per D-S1002127-02 ADR. Adding or
//! removing a variant fails `cargo check` via [`EscapeSurfaceProfile::VARIANTS`].

use thiserror::Error;

use crate::m30_bank::AcceptedWorkflow;
use crate::m33_verifier::{aggregate, AggregateVerdict, Verifier, VerifierKind};

/// Production Conductor RPC method routed by [`ConductorClient`]
/// implementations. Locked at `"lcm.loop.create"` — `lcm.deploy` is a
/// regression target documented in the v1.3 spec.
pub const CONDUCTOR_DISPATCH_METHOD: &str = "lcm.loop.create";

/// Closed-set escape-surface profile (7 variants per D-S1002127-02 ADR).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscapeSurfaceProfile {
    /// Sandboxed (ord 0).
    Sandboxed,
    /// `SandboxEscape` (ord 10).
    SandboxEscape,
    /// `ProcessMutate` (ord 20).
    ProcessMutate,
    /// `PrivilegeEscalation` (ord 30; D-S1002127-02 amendment).
    PrivilegeEscalation,
    /// `FileWrite` (ord 40).
    FileWrite,
    /// `NetworkEgress` (ord 50).
    NetworkEgress,
    /// `DataExfil` (ord 60).
    DataExfil,
}

impl EscapeSurfaceProfile {
    /// Canonical enumeration of all variants in ordinal-ascending order.
    ///
    /// Used by the compile-time cardinality assertion below and by tests
    /// that exhaustively exercise each profile. Adding a variant requires
    /// updating this array, which is then enforced to be the same length
    /// (7) by the const-context assertion.
    pub const VARIANTS: [Self; 7] = [
        Self::Sandboxed,
        Self::SandboxEscape,
        Self::ProcessMutate,
        Self::PrivilegeEscalation,
        Self::FileWrite,
        Self::NetworkEgress,
        Self::DataExfil,
    ];

    /// Stable ordinal.
    #[must_use]
    pub const fn ordinal(self) -> u8 {
        match self {
            Self::Sandboxed => 0,
            Self::SandboxEscape => 10,
            Self::ProcessMutate => 20,
            Self::PrivilegeEscalation => 30,
            Self::FileWrite => 40,
            Self::NetworkEgress => 50,
            Self::DataExfil => 60,
        }
    }

    /// `true` if dispatching this profile is covered by `signature` — the
    /// operator's acknowledged ceiling is at least this profile's severity.
    ///
    /// The destructiveness ladder is monotone: an operator who has
    /// acknowledged a higher rung (e.g. [`Self::DataExfil`], ordinal 60)
    /// implicitly covers every lower rung. The default ceiling is
    /// [`Self::Sandboxed`] (ordinal 0) — nothing beyond the sandbox is
    /// acknowledged unless the operator says so.
    #[must_use]
    pub const fn is_acknowledged_by(self, signature: &HumanAcceptanceSignature) -> bool {
        self.ordinal() <= signature.acknowledged_ceiling.ordinal()
    }

    /// `true` if dispatching this profile is covered by the
    /// `acknowledged_ceiling` exposed by any
    /// [`AcceptanceSignatureReader`]. Trait-generic counterpart of
    /// [`Self::is_acknowledged_by`].
    ///
    /// The monotone destructiveness gate is identical: a profile at ordinal
    /// X is permitted iff the reader-supplied ceiling has ordinal ≥ X. The
    /// dedicated trait-generic method exists so callers that hold an
    /// arbitrary `&impl AcceptanceSignatureReader` (e.g. the m9 namespace
    /// guard, which must not depend on the concrete
    /// [`HumanAcceptanceSignature`] layout) can perform the gate check
    /// without an explicit `signature.acknowledged_ceiling.ordinal()` poke.
    ///
    /// Phase 6e — m9 ↔ m32 EscapeSurfaceProfile seam (gap C-8 fold).
    #[must_use]
    pub fn is_acknowledged_by_reader<R: AcceptanceSignatureReader + ?Sized>(
        self,
        reader: &R,
    ) -> bool {
        self.ordinal() <= reader.acknowledged_ceiling().ordinal()
    }
}

/// Read the operator's acknowledged
/// [`EscapeSurfaceProfile`] ceiling from any acceptance-signature carrier.
///
/// The trait is the **single shared seam** between m32's dispatch-time
/// acknowledgement gate and m9's write-time namespace-capability gate
/// (Phase 6e, gap C-8 / NA-GAP-11 fold). It is implemented for
/// [`HumanAcceptanceSignature`] here at its definition site and consumed
/// by [`EscapeSurfaceProfile::is_acknowledged_by_reader`] (m32) and
/// [`crate::m9_watcher_namespace_guard::assert_namespace_capability`]
/// (m9). Defining the trait ONCE — and implementing it ONCE on the
/// canonical [`HumanAcceptanceSignature`] — keeps the m9 ↔ m32 contract a
/// single source of truth.
///
/// # Monotonicity
///
/// The destructiveness ladder is monotone: an operator who has acknowledged
/// a higher rung (e.g. [`EscapeSurfaceProfile::DataExfil`], ordinal 60)
/// implicitly covers every lower rung. The trait surfaces exactly the
/// minimum data point that gate check needs — the highest ordinal the
/// operator has explicitly accepted — so neither implementor nor consumer
/// must agree on any other field of the underlying signature struct.
pub trait AcceptanceSignatureReader {
    /// The highest [`EscapeSurfaceProfile`] the operator has acknowledged.
    /// Dispatch / write of any profile at or below this severity is
    /// permitted.
    fn acknowledged_ceiling(&self) -> EscapeSurfaceProfile;
}

impl AcceptanceSignatureReader for HumanAcceptanceSignature {
    fn acknowledged_ceiling(&self) -> EscapeSurfaceProfile {
        self.acknowledged_ceiling
    }
}

impl<T> AcceptanceSignatureReader for &T
where
    T: AcceptanceSignatureReader + ?Sized,
{
    fn acknowledged_ceiling(&self) -> EscapeSurfaceProfile {
        (**self).acknowledged_ceiling()
    }
}

// Compile-time cardinality enforcement (D-S1002127-02): the variant count
// must remain 7. Any drift breaks `cargo check`.
const _: () = {
    assert!(
        EscapeSurfaceProfile::VARIANTS.len() == 7,
        "EscapeSurfaceProfile cardinality drift — D-S1002127-02 ADR locks at 7"
    );
};

/// Human acceptance signature accompanying a dispatch (D-S1002127-02).
///
/// The acknowledgement gate is **monotone**: rather than carrying one bool
/// per acknowledgeable profile (which left `SandboxEscape` / `ProcessMutate`
/// / `FileWrite` / `NetworkEgress` ungated), the operator records a single
/// `acknowledged_ceiling` — the *highest* [`EscapeSurfaceProfile`] severity
/// they have accepted. A dispatch at profile X is permitted iff X's ordinal
/// is `<= acknowledged_ceiling.ordinal()`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HumanAcceptanceSignature {
    /// Operator confirmed interactive terminal.
    pub interactive_terminal: bool,
    /// The highest [`EscapeSurfaceProfile`] the operator has acknowledged.
    /// Dispatch of any profile at or below this severity is permitted.
    pub acknowledged_ceiling: EscapeSurfaceProfile,
}

impl Default for HumanAcceptanceSignature {
    fn default() -> Self {
        Self {
            interactive_terminal: true,
            // Ordinal 0 — nothing beyond the sandbox is acknowledged by
            // default; the operator must raise the ceiling explicitly.
            acknowledged_ceiling: EscapeSurfaceProfile::Sandboxed,
        }
    }
}

/// Outcome of a dispatch attempt.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum DispatchOutcome {
    /// Conductor accepted the request; carries a Conductor-assigned id.
    Accepted {
        /// Conductor's dispatch id.
        conductor_dispatch_id: String,
    },
    /// Refused by the 5-check sequence.
    Refused {
        /// Reason variant.
        reason: RefusalReason,
    },
}

/// Refusal reasons.
///
/// `#[non_exhaustive]`: this is an evolving wire enum — new refusal classes
/// may be added as the dispatch check-sequence grows. Within-crate matches
/// stay exhaustive; external consumers must include a wildcard arm.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum RefusalReason {
    /// The dispatched escape-surface profile exceeds the operator's
    /// acknowledged ceiling (monotone destructiveness gate).
    EscapeSurfaceNotAcknowledged {
        /// The profile the workflow requires.
        required: EscapeSurfaceProfile,
        /// The ceiling the operator actually acknowledged.
        acknowledged: EscapeSurfaceProfile,
    },
    /// Workflow not in the curated bank.
    WorkflowNotBanked,
    /// Conductor endpoint unreachable.
    ConductorUnreachable,
    /// Self-dispatch (AP-V7-08) refused.
    SelfDispatchRefused,
    /// Generic spec-bound refusal.
    SpecBoundRefusal,
    /// `ConductorClient::dispatch_method()` does not match
    /// [`CONDUCTOR_DISPATCH_METHOD`]. A misconfigured client (e.g. one routing
    /// to `lcm.deploy` instead of `lcm.loop.create`) is refused before egress
    /// — the defensive constant is now enforced, not merely documented.
    /// Carries the expected (canonical) and actual (client-reported) RPC
    /// method names for operator triage.
    ///
    /// Both fields are `String` (rather than `&'static str`) so this variant
    /// participates in `serde::Deserialize` without a lifetime bound — the
    /// wire-format must round-trip cleanly across the IPC bus.
    ///
    /// Additive public-API surface (C3 hardening, S1002600 carry-forward).
    RoutingMethodMismatch {
        /// The canonical RPC method (the value of
        /// [`CONDUCTOR_DISPATCH_METHOD`] at dispatch time).
        expected: String,
        /// The method the supplied [`ConductorClient`] reports.
        actual: String,
    },
    /// m33 verifier gate blocked the workflow. Carries the set of verifier
    /// kinds whose verdicts were blocking (Refuse or Amend), in
    /// [`crate::m33_verifier::VerifierKind::ordinal`] order.
    ///
    /// Additive public-API surface (H6 hardening, S1002600 carry-forward).
    VerifierGateBlocked {
        /// Verifier kinds whose verdicts were blocking, ordinal-ordered.
        blocking_kinds: Vec<VerifierKind>,
    },
    /// The Conductor response could not be parsed (or the transport framing
    /// failed); carries the wire-format detail.
    ///
    /// Distinct from [`Self::ConductorUnreachable`]: a `WireFormat` refusal
    /// means the Conductor endpoint *was* contacted but returned an
    /// unparseable / mis-framed response, whereas `ConductorUnreachable`
    /// (reserved for genuine connection failure) carries no detail.
    ///
    /// The `detail` field is `String` (not `&'static str`) so this variant
    /// participates in `serde::Deserialize` without a lifetime bound — the
    /// wire-format must round-trip cleanly across the IPC bus.
    ///
    /// Additive public-API surface (C4 hardening — previously the wire-format
    /// detail was only surfaced via `tracing::warn!` and lost from the
    /// structured outcome).
    WireFormat {
        /// Human-readable parser/transport detail for operator triage.
        detail: String,
    },
}

/// Dispatcher errors.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DispatcherError {
    /// Conductor wire-format failure.
    #[error("wire format: {0}")]
    WireFormat(String),
}

/// Trait for the Conductor client (mocked in tests; production hits
/// `:8141`).
pub trait ConductorClient: Send + Sync {
    /// Submit a dispatch request; return the Conductor-assigned id.
    ///
    /// # Errors
    ///
    /// [`DispatcherError::WireFormat`] for unparseable response.
    fn submit(
        &self,
        workflow_id: u64,
        profile: EscapeSurfaceProfile,
        signature: &HumanAcceptanceSignature,
    ) -> Result<String, DispatcherError>;

    /// Production RPC method this client invokes. Default is
    /// [`CONDUCTOR_DISPATCH_METHOD`]; tests and adapters may override
    /// (but the production wire MUST route `lcm.loop.create`).
    fn dispatch_method(&self) -> &'static str {
        CONDUCTOR_DISPATCH_METHOD
    }
}

/// AP-V7-08 self-dispatch guard.
///
/// Returns `true` if the workflow is allowed to dispatch (no forbidden
/// proposal ids match); `false` if the workflow targets m32 itself.
///
/// Callers compose this with [`ConductorDispatcher::dispatch`]; the
/// dispatcher also invokes the guard internally when a `forbidden_proposals`
/// list is configured.
#[must_use]
pub fn self_dispatch_guard(workflow: &AcceptedWorkflow, forbidden_proposals: &[u64]) -> bool {
    let pid = workflow.proposal().proposal_id();
    !forbidden_proposals.contains(&pid)
}

/// The dispatcher.
pub struct ConductorDispatcher<C: ConductorClient> {
    client: C,
    /// Caller-supplied list of `proposal_id` values that target m32 itself.
    forbidden_proposals: Vec<u64>,
    /// Optional m33 verifier set. When non-empty, [`Self::dispatch`] runs
    /// [`aggregate`] before egress; any blocking verdict refuses with
    /// [`RefusalReason::VerifierGateBlocked`]. When empty, the verifier gate
    /// is skipped (backward-compatible legacy path for callers that have not
    /// adopted the m31/m33 → m32 bridge).
    verifiers: Vec<Box<dyn Verifier>>,
}

impl<C: ConductorClient> std::fmt::Debug for ConductorDispatcher<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // `Verifier` is not `Debug`-bound by contract; surface just the count
        // so operators can confirm wiring without coupling Debug to the
        // trait surface.
        f.debug_struct("ConductorDispatcher")
            .field("forbidden_count", &self.forbidden_proposals.len())
            .field("verifier_count", &self.verifiers.len())
            .field("dispatch_method", &self.client.dispatch_method())
            .finish_non_exhaustive()
    }
}

impl<C: ConductorClient> ConductorDispatcher<C> {
    /// Construct with the given client and no self-dispatch list.
    pub fn new(client: C) -> Self {
        Self {
            client,
            forbidden_proposals: Vec::new(),
            verifiers: Vec::new(),
        }
    }

    /// Construct with explicit AP-V7-08 forbidden-proposal ids. The
    /// dispatcher refuses any workflow whose `proposal_id` matches one of
    /// these values.
    pub fn with_forbidden_proposals(client: C, forbidden: Vec<u64>) -> Self {
        Self {
            client,
            forbidden_proposals: forbidden,
            verifiers: Vec::new(),
        }
    }

    /// Attach an m33 verifier set to be aggregated before egress.
    ///
    /// Builder method (consumes `self`). Passing an empty `verifiers` vec
    /// disables the gate — equivalent to the legacy no-verifier behaviour.
    /// A non-empty set MUST satisfy [`aggregate`]'s "exactly one of each
    /// [`VerifierKind`]" precondition at dispatch time; if it does not, the
    /// gate refuses with [`RefusalReason::VerifierGateBlocked`] carrying the
    /// kinds whose verdicts could be collected (operationally: a malformed
    /// verifier set is a block, never a silent pass).
    ///
    /// Additive public-API surface (H6 hardening, S1002600 carry-forward).
    #[must_use]
    pub fn with_verifiers(mut self, verifiers: Vec<Box<dyn Verifier>>) -> Self {
        self.verifiers = verifiers;
        self
    }

    /// Run the dispatch check sequence + submit if all checks pass.
    ///
    /// 1. AP-V7-08 self-dispatch refusal.
    /// 2. Monotone signature acknowledgement: the profile's severity must not
    ///    exceed the operator's acknowledged ceiling.
    /// 3. **Routing-method enforcement** (C3, S1002600): the underlying
    ///    [`ConductorClient::dispatch_method`] MUST match
    ///    [`CONDUCTOR_DISPATCH_METHOD`]; a misconfigured client routing to
    ///    `lcm.deploy` (or anything else) is refused before egress with
    ///    [`RefusalReason::RoutingMethodMismatch`].
    /// 4. **m33 verifier gate** (H6, S1002600): when [`Self::with_verifiers`]
    ///    has supplied a non-empty set, [`aggregate`] is run and any blocking
    ///    verdict refuses with [`RefusalReason::VerifierGateBlocked`].
    /// 5. Conductor reachable.
    ///
    /// # Errors
    ///
    /// This method only returns [`DispatcherError`] for explicit caller
    /// faults; all refusals (including transport errors) are folded into
    /// [`DispatchOutcome::Refused`].
    pub fn dispatch(
        &self,
        workflow: &AcceptedWorkflow,
        profile: EscapeSurfaceProfile,
        signature: &HumanAcceptanceSignature,
    ) -> Result<DispatchOutcome, DispatcherError> {
        // Check 1 — AP-V7-08 self-dispatch refusal.
        if !self_dispatch_guard(workflow, &self.forbidden_proposals) {
            tracing::warn!(
                workflow_id = workflow.workflow_id(),
                proposal_id = workflow.proposal().proposal_id(),
                "m32: AP-V7-08 self-dispatch refused"
            );
            return Ok(DispatchOutcome::Refused {
                reason: RefusalReason::SelfDispatchRefused,
            });
        }
        // Check 2 — monotone signature acknowledgement. The destructiveness
        // ladder implies monotone severity: a dispatch at profile X is
        // permitted iff the operator's acknowledged ceiling is at least X.
        // This closes the pre-fix gap where SandboxEscape / ProcessMutate /
        // FileWrite / NetworkEgress dispatched with no acknowledgement at all
        // — FileWrite (ord 40) and NetworkEgress (ord 50) outranking the
        // gated PrivilegeEscalation (ord 30) yet sailing through.
        if !profile.is_acknowledged_by(signature) {
            tracing::warn!(
                workflow_id = workflow.workflow_id(),
                required = profile.ordinal(),
                acknowledged = signature.acknowledged_ceiling.ordinal(),
                "m32: escape-surface profile exceeds acknowledged ceiling — refusing"
            );
            return Ok(DispatchOutcome::Refused {
                reason: RefusalReason::EscapeSurfaceNotAcknowledged {
                    required: profile,
                    acknowledged: signature.acknowledged_ceiling,
                },
            });
        }
        // Check 3 — C3 routing-method enforcement. The defensive const
        // CONDUCTOR_DISPATCH_METHOD existed since v1.3 but was never
        // compared against the client's reported method. A misconfigured
        // ConductorClient that routes to "lcm.deploy" (the documented
        // regression target) would otherwise dispatch silently. We refuse
        // before egress and surface both names for triage.
        let client_method = self.client.dispatch_method();
        if client_method != CONDUCTOR_DISPATCH_METHOD {
            tracing::warn!(
                workflow_id = workflow.workflow_id(),
                expected = CONDUCTOR_DISPATCH_METHOD,
                actual = %client_method,
                "m32: routing-method mismatch — refusing pre-egress"
            );
            return Ok(DispatchOutcome::Refused {
                reason: RefusalReason::RoutingMethodMismatch {
                    expected: CONDUCTOR_DISPATCH_METHOD.to_owned(),
                    actual: client_method.to_owned(),
                },
            });
        }
        // Check 4 — H6 m33 verifier gate. Empty verifier set means the gate
        // is intentionally disabled (legacy callers); non-empty means run
        // aggregate(). A malformed set (missing kind / duplicate kind /
        // any blocking verdict) refuses with VerifierGateBlocked carrying
        // the blocking kinds (or, for the malformed-set case, the empty
        // vec — the operator triages from the tracing event).
        if !self.verifiers.is_empty() {
            let refs: Vec<&dyn Verifier> = self
                .verifiers
                .iter()
                .map(std::convert::AsRef::as_ref)
                .collect();
            match aggregate(&refs, workflow) {
                Ok(AggregateVerdict::AllApprove) => {
                    // Fall through to submit.
                }
                Ok(AggregateVerdict::Blocked { per_verifier }) => {
                    let blocking_kinds: Vec<VerifierKind> = per_verifier
                        .iter()
                        .filter(|(_, v)| v.is_blocking())
                        .map(|(k, _)| *k)
                        .collect();
                    tracing::warn!(
                        workflow_id = workflow.workflow_id(),
                        blocking_count = blocking_kinds.len(),
                        "m32: m33 verifier gate blocked — refusing pre-egress"
                    );
                    return Ok(DispatchOutcome::Refused {
                        reason: RefusalReason::VerifierGateBlocked { blocking_kinds },
                    });
                }
                Err(err) => {
                    // Malformed verifier set is operator misuse — fail
                    // closed (refuse) rather than fall through to egress.
                    tracing::warn!(
                        workflow_id = workflow.workflow_id(),
                        error = %err,
                        "m32: m33 verifier set malformed — refusing pre-egress"
                    );
                    return Ok(DispatchOutcome::Refused {
                        reason: RefusalReason::VerifierGateBlocked {
                            blocking_kinds: Vec::new(),
                        },
                    });
                }
            }
        }
        // Subsequent spec-bound refusal hooks (bank membership / sunset gate)
        // are reserved for upstream callers per m32 spec § 5.
        // The workflow id is read here so that the upstream contract is
        // explicit at this site; it is the only egress argument the
        // Conductor sees aside from profile + signature.
        let workflow_id = workflow.workflow_id();
        // Check 5 — submit via Conductor.
        match self.client.submit(workflow_id, profile, signature) {
            Ok(id) => Ok(DispatchOutcome::Accepted {
                conductor_dispatch_id: id,
            }),
            Err(DispatcherError::WireFormat(detail)) => {
                // Surface the detail via tracing for operator triage at the
                // log surface...
                tracing::warn!(
                    workflow_id,
                    method = %self.client.dispatch_method(),
                    detail = %detail,
                    "m32: conductor wire-format failure"
                );
                // ...and also carry it in the structured outcome so an
                // operator triaging from the DispatchOutcome alone does not
                // lose the failure cause (C4 hardening).
                Ok(DispatchOutcome::Refused {
                    reason: RefusalReason::WireFormat { detail },
                })
            }
        }
    }

    /// Production routing target inspected by adapters and tests.
    pub fn dispatch_method(&self) -> &'static str {
        self.client.dispatch_method()
    }
}

#[cfg(test)]
mod tests;
