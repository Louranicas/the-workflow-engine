//! `m9_watcher_namespace_guard` — application-layer namespace prefix validator.
//!
//! See [m9 spec](../../../ai_specs/modules/cluster-D/m9_watcher_namespace_guard.md)
//! for the canonical contract.
//!
//! # Purpose
//!
//! Defense-in-depth complement to the SpacetimeDB reducer-level refuse-write
//! invariant (per `CONSUMER-ONBOARDING.md`). m9 surfaces the assertion at the
//! **application layer** immediately before the reducer call, so the failure
//! mode appears as a typed [`NamespaceViolation`] with a human-readable
//! `tracing::error!` at the call site — not as an opaque SpacetimeDB 530
//! HTTP error several stack frames downstream.
//!
//! # Observer read-deny convention
//!
//! The Watcher (synthex-v2 m46-m51) MAY read `workflow_trace_*` namespace
//! via SQL (Observer role per the Watcher persona doc) but MUST NOT write.
//! m9 has no runtime enforcement on reads — reads cannot be gated at the
//! application-layer validator — but this module docstring is the
//! authoritative documentation site for the architectural convention. The
//! enforcement is architectural (Watcher R13 scope discipline + AP27
//! self-modification boundary).
//!
//! # Gap 3 co-ownership
//!
//! m9 is the **namespace dimension** co-owner of Gap 3 (Unified
//! destructiveness / `EscapeSurfaceProfile` schema — shared with m30 +
//! m32). The 7-variant `EscapeSurfaceProfile` capability table per spec § 2
//! is integrated post-Cluster-G build (see TODO in `validator.rs`).
//!
//! # Public surface
//!
//! - [`WORKFLOW_TRACE_NS_PREFIX`] — single source of truth for the prefix.
//! - [`assert_workflow_trace_namespace`] — validator + munge + typed refusal.
//! - [`munge_hyphen_slug`] — hyphen → underscore helper (idempotent).
//! - [`assert_namespace_capability`] — Phase 6e m9 ↔ m32 capability gate
//!   (monotone [`crate::m32_dispatcher::EscapeSurfaceProfile`] ladder, read
//!   via the shared [`crate::m32_dispatcher::AcceptanceSignatureReader`]
//!   trait).
//! - [`required_signature_ceiling`] — m9 spec § 2 capability table; identity
//!   on the single-axis monotone ladder.
//! - [`ValidatedNamespace`] — newtype evidence consumed by m13 / m42 writers.
//! - [`NamespaceViolation`] — 6-variant error enum (`WrongPrefix` / `Empty`
//!   / `Whitespace` / `ScratchForbidden` / `ControlChar` /
//!   `CapabilityNotAcknowledged`).

pub mod error;
pub mod evidence;
pub mod validator;

pub use error::NamespaceViolation;
pub use evidence::ValidatedNamespace;
pub use validator::{
    assert_namespace_capability, assert_workflow_trace_namespace, munge_hyphen_slug,
    required_signature_ceiling, WORKFLOW_TRACE_NS_PREFIX,
};

#[cfg(test)]
mod tests {
    use super::{
        assert_workflow_trace_namespace, munge_hyphen_slug, NamespaceViolation,
        ValidatedNamespace, WORKFLOW_TRACE_NS_PREFIX,
    };

    #[test]
    fn reexports_prefix_constant() {
        assert_eq!(WORKFLOW_TRACE_NS_PREFIX, "workflow_trace");
    }

    #[test]
    fn reexports_validator_returns_validated_namespace() {
        let v: ValidatedNamespace =
            assert_workflow_trace_namespace("workflow_trace_x").expect("happy");
        assert_eq!(v.as_str(), "workflow_trace_x");
    }

    #[test]
    fn reexports_munge_helper() {
        assert_eq!(munge_hyphen_slug("a-b-c"), "a_b_c");
    }

    #[test]
    fn reexports_error_enum_for_wrong_prefix() {
        let err: NamespaceViolation = assert_workflow_trace_namespace("orac_x").unwrap_err();
        assert!(matches!(err, NamespaceViolation::WrongPrefix { .. }));
    }
}
