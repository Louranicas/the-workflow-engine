//! Error types for the m9 application-layer namespace guard.
//!
//! Per m9 spec § 4 — error-band assignment per
//! `ERROR_TAXONOMY.md § E3xxx` (Trust-layer violations):
//!
//! - [`NamespaceViolation::WrongPrefix`]                     = `E3101`
//! - [`NamespaceViolation::Empty`]                           = `E3102`
//! - [`NamespaceViolation::Whitespace`]                      = `E3103`
//! - [`NamespaceViolation::ScratchForbidden`]                = `E3104`
//! - [`NamespaceViolation::ControlChar`]                     = `E3105`
//! - [`NamespaceViolation::CapabilityNotAcknowledged`]       = `E3106`
//!
//! Every variant Display text names the violated invariant and (where
//! applicable) the offending input and the expected prefix so operators can
//! recover without log-hunting. This mirrors m8's error-message discipline.

use thiserror::Error;

/// Application-layer refusal at any substrate-write boundary in the
/// workflow-trace binaries. This is the defense-in-depth complement to the
/// SpacetimeDB reducer-level refuse-write invariant (per
/// `CONSUMER-ONBOARDING.md`).
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum NamespaceViolation {
    /// The munged namespace did not start with
    /// [`super::validator::WORKFLOW_TRACE_NS_PREFIX`]. AP30 mitigation: this
    /// is the dominant violation kind — every other registered habitat
    /// service has its own prefix.
    #[error(
        "stcortex write blocked: namespace '{namespace}' does not start with \
         '{expected_prefix}_' — workflow-trace must not write to other services' \
         namespaces (AP30 mitigation)"
    )]
    WrongPrefix {
        /// Namespace as the validator saw it AFTER hyphen-slug munge.
        namespace: String,
        /// The required prefix (static; mirrors
        /// [`super::validator::WORKFLOW_TRACE_NS_PREFIX`]).
        expected_prefix: &'static str,
    },

    /// The input namespace was the empty string. Distinct from
    /// [`Self::WrongPrefix`] because an empty input is almost always a
    /// programming error (uninitialised builder, missing struct field) rather
    /// than a foreign-prefix attempt.
    #[error("stcortex write blocked: namespace is empty")]
    Empty,

    /// The input namespace contained a whitespace character. This is almost
    /// always a hyphen-slug-munge failure upstream (S1001757); m9 names the
    /// antipattern in the error message so operators can grep for it.
    #[error(
        "stcortex write blocked: namespace '{namespace}' contains whitespace; \
         expected hyphen-slug munge to underscore form (AP-Hab-11 mitigation)"
    )]
    Whitespace {
        /// Namespace as the validator received it (BEFORE munge — whitespace
        /// is rejected before the munge step per spec § 5).
        namespace: String,
    },

    /// The input namespace was exactly the string `"scratch"`. The bare
    /// `"scratch"` namespace is reserved at the habitat layer for ad-hoc /
    /// non-attributable writes and is forbidden for workflow-trace.
    #[error(
        "stcortex write blocked: 'scratch' namespace forbidden for workflow-trace \
         (use workflow_trace_scratch or a domain prefix)"
    )]
    ScratchForbidden,

    /// The input namespace contained a control character (`\0`, ASCII
    /// `< 0x20` not already whitespace, or a Unicode BOM `\u{FEFF}`).
    /// These would silently corrupt downstream slug logging / SQL bindings
    /// without firing the whitespace check (which only catches
    /// `char::is_whitespace`).
    #[error(
        "stcortex write blocked: namespace contains non-printable / control \
         character (U+{codepoint:04X} at byte offset {byte_offset}) — \
         likely encoding contamination or BOM slip"
    )]
    ControlChar {
        /// Unicode code point of the offending character.
        codepoint: u32,
        /// Byte offset within the input string.
        byte_offset: usize,
    },

    /// The required [`EscapeSurfaceProfile`] for this write exceeds the
    /// operator's acknowledged ceiling (monotone destructiveness gate).
    ///
    /// This is the m9 application-layer mirror of m32's
    /// [`RefusalReason::EscapeSurfaceNotAcknowledged`]: the same
    /// monotone ladder is enforced at the namespace-write boundary so
    /// that a `PrivilegeEscalation`-shaped or `DataExfil`-shaped write
    /// cannot leak past m9 while the operator has only acknowledged
    /// `Sandboxed`. Phase 6e — m9 ↔ m32 seam (gap C-8 / NA-GAP-11
    /// fold).
    ///
    /// Carries the required profile and the actually-acknowledged
    /// ceiling for operator triage; both are
    /// [`EscapeSurfaceProfile`] values so the wire round-trip is
    /// closed-set and serde-stable.
    ///
    /// [`EscapeSurfaceProfile`]: crate::m32_dispatcher::EscapeSurfaceProfile
    /// [`RefusalReason::EscapeSurfaceNotAcknowledged`]: crate::m32_dispatcher::RefusalReason::EscapeSurfaceNotAcknowledged
    #[error(
        "stcortex write blocked: required EscapeSurfaceProfile {required:?} \
         (ord {required_ord}) exceeds operator's acknowledged ceiling \
         {acknowledged:?} (ord {acknowledged_ord}) — m9 application-layer \
         capability gate (Phase 6e seam, mirrors m32 monotone gate)"
    )]
    CapabilityNotAcknowledged {
        /// The [`crate::m32_dispatcher::EscapeSurfaceProfile`] the write requires.
        required: crate::m32_dispatcher::EscapeSurfaceProfile,
        /// Ordinal of `required` (denormalised for the operator log).
        required_ord: u8,
        /// The ceiling the operator has acknowledged.
        acknowledged: crate::m32_dispatcher::EscapeSurfaceProfile,
        /// Ordinal of `acknowledged` (denormalised for the operator log).
        acknowledged_ord: u8,
    },
}

#[cfg(test)]
mod tests {
    use super::NamespaceViolation;

    // ---- Display shape (4 tests; F-Contract candidates) -----------------

    #[test]
    fn wrong_prefix_display_names_namespace_prefix_and_ap30() {
        let err = NamespaceViolation::WrongPrefix {
            namespace: "orac_learn".into(),
            expected_prefix: "workflow_trace",
        };
        let msg = err.to_string();
        assert!(msg.contains("orac_learn"), "missing namespace: {msg}");
        assert!(msg.contains("workflow_trace"), "missing prefix: {msg}");
        assert!(msg.contains("AP30"), "missing AP30 marker: {msg}");
    }

    #[test]
    fn empty_display_is_stable() {
        assert_eq!(
            NamespaceViolation::Empty.to_string(),
            "stcortex write blocked: namespace is empty"
        );
    }

    #[test]
    fn whitespace_display_names_input_and_ap_hab_11() {
        let err = NamespaceViolation::Whitespace {
            namespace: "wf trace x".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("wf trace x"), "missing input: {msg}");
        assert!(msg.contains("AP-Hab-11"), "missing AP-Hab-11 marker: {msg}");
    }

    #[test]
    fn scratch_forbidden_display_names_scratch_and_suggests_alternative() {
        let msg = NamespaceViolation::ScratchForbidden.to_string();
        assert!(msg.contains("scratch"));
        assert!(msg.contains("workflow_trace_scratch"));
    }

    // ---- Trait obligations (4 tests) ------------------------------------

    #[test]
    fn implements_std_error() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<NamespaceViolation>();
    }

    #[test]
    fn variants_are_send_sync_static() {
        fn assert_send_sync_static<T: Send + Sync + 'static>() {}
        assert_send_sync_static::<NamespaceViolation>();
    }

    #[test]
    fn implements_clone_partial_eq_eq() {
        let a = NamespaceViolation::Empty;
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn partial_eq_distinguishes_variants() {
        assert_ne!(
            NamespaceViolation::Empty,
            NamespaceViolation::ScratchForbidden
        );
        assert_eq!(
            NamespaceViolation::WrongPrefix {
                namespace: "x".into(),
                expected_prefix: "p"
            },
            NamespaceViolation::WrongPrefix {
                namespace: "x".into(),
                expected_prefix: "p"
            }
        );
        assert_ne!(
            NamespaceViolation::WrongPrefix {
                namespace: "x".into(),
                expected_prefix: "p"
            },
            NamespaceViolation::WrongPrefix {
                namespace: "y".into(),
                expected_prefix: "p"
            }
        );
    }

    // ---- Debug format snapshots (2 tests; F-Contract) -------------------

    #[test]
    fn debug_format_stable_for_empty() {
        assert_eq!(format!("{:?}", NamespaceViolation::Empty), "Empty");
    }

    #[test]
    fn debug_format_stable_for_scratch_forbidden() {
        assert_eq!(
            format!("{:?}", NamespaceViolation::ScratchForbidden),
            "ScratchForbidden"
        );
    }

    // ---- Day-1 exhaustiveness contract (1 test; F-Regression) -----------

    #[test]
    fn day_1_variant_set_is_exhaustively_matched() {
        // Per m9 spec § 2 capability table the EscapeSurfaceProfile-aware
        // capability gate landed Phase 6e (gap C-8 / NA-GAP-11 fold) as
        // `CapabilityNotAcknowledged`. The full variant set is the six
        // below — adding a new variant breaks this test, prompting an
        // explicit spec amendment.
        let err = NamespaceViolation::Empty;
        match err {
            NamespaceViolation::Empty
            | NamespaceViolation::WrongPrefix { .. }
            | NamespaceViolation::Whitespace { .. }
            | NamespaceViolation::ScratchForbidden
            | NamespaceViolation::ControlChar { .. }
            | NamespaceViolation::CapabilityNotAcknowledged { .. } => {}
        }
    }

    #[test]
    fn control_char_display_names_codepoint_and_offset() {
        let err = NamespaceViolation::ControlChar {
            codepoint: 0,
            byte_offset: 7,
        };
        let msg = err.to_string();
        assert!(msg.contains("U+0000"), "missing codepoint hex: {msg}");
        assert!(msg.contains("offset 7"), "missing byte offset: {msg}");
        assert!(msg.contains("BOM") || msg.contains("control"), "missing diagnostic: {msg}");
    }
}
