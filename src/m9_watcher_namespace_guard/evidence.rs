//! Newtype evidence that a namespace string has been validated.
//!
//! Per m9 spec § 3: [`ValidatedNamespace`] is the type that downstream
//! writers (m13, m42) accept in place of `&str`, so the type system enforces
//! that any stcortex write call has already passed through the prefix check.
//! This is the AP30 compile-time mitigation — code that bypasses the
//! validator fails to type-check at the writer-call boundary.
//!
//! The single legal constructor is
//! [`super::validator::assert_workflow_trace_namespace`]; the inner field is
//! `pub(super)`-visible so no code outside the module can fabricate evidence.

/// Proof that a namespace string:
///
/// 1. is non-empty,
/// 2. contains no whitespace,
/// 3. is not the forbidden bare `"scratch"`,
/// 4. starts with [`super::validator::WORKFLOW_TRACE_NS_PREFIX`] AFTER any
///    hyphens have been munged to underscores (AP-Hab-11 mitigation, applied
///    exactly once at the validator boundary).
///
/// Construct only via
/// [`super::validator::assert_workflow_trace_namespace`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValidatedNamespace(pub(super) String);

impl ValidatedNamespace {
    /// Borrow the munged namespace as a `&str`. The returned slice is
    /// suitable for direct use as a stcortex consumer namespace, pathway-id
    /// prefix, or any other substrate identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume the evidence and return the owned munged string.
    ///
    /// This is occasionally useful at writer boundaries that already require
    /// an owned `String`. Prefer [`Self::as_str`] when a borrow suffices.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl std::fmt::Display for ValidatedNamespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for ValidatedNamespace {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::ValidatedNamespace;

    // ---- F-Unit (7 tests) ------------------------------------------------

    #[test]
    fn as_str_roundtrips() {
        let v = ValidatedNamespace("workflow_trace_x".into());
        assert_eq!(v.as_str(), "workflow_trace_x");
    }

    #[test]
    fn into_inner_yields_owned_munged_string() {
        let v = ValidatedNamespace("workflow_trace_alpha".into());
        let s: String = v.into_inner();
        assert_eq!(s, "workflow_trace_alpha");
    }

    #[test]
    fn display_produces_munged_form() {
        let v = ValidatedNamespace("workflow_trace_alpha_beta".into());
        assert_eq!(format!("{v}"), "workflow_trace_alpha_beta");
    }

    #[test]
    fn as_ref_str_works() {
        fn takes_ref(s: &str) -> usize {
            s.len()
        }
        let v = ValidatedNamespace("workflow_trace_y".into());
        assert_eq!(takes_ref(v.as_ref()), "workflow_trace_y".len());
    }

    #[test]
    fn equal_when_inner_equal() {
        assert_eq!(
            ValidatedNamespace("x".into()),
            ValidatedNamespace("x".into())
        );
    }

    #[test]
    fn unequal_when_inner_differs() {
        assert_ne!(
            ValidatedNamespace("a".into()),
            ValidatedNamespace("b".into())
        );
    }

    #[test]
    fn hashable_and_dedups_in_hashset() {
        let mut set: HashSet<ValidatedNamespace> = HashSet::new();
        set.insert(ValidatedNamespace("workflow_trace_x".into()));
        set.insert(ValidatedNamespace("workflow_trace_y".into()));
        set.insert(ValidatedNamespace("workflow_trace_x".into()));
        assert_eq!(set.len(), 2);
    }

    // ---- F-Contract (2 tests) --------------------------------------------

    #[test]
    fn debug_format_contains_inner() {
        let v = ValidatedNamespace("workflow_trace_z".into());
        let s = format!("{v:?}");
        assert!(s.contains("workflow_trace_z"));
    }

    #[test]
    fn clone_preserves_value() {
        let v = ValidatedNamespace("workflow_trace_clone".into());
        let c = v.clone();
        assert_eq!(v, c);
    }
}
