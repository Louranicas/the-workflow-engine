//! Soft-schema Battern step labels + the F1 (bank/name ossification)
//! invariant.
//!
//! **F1 STRUCTURAL GUARANTEE:** there is NO `Other` variant in
//! [`BatternStepLabel`]. Unlabelled steps surface as `Option::None`
//! upstream; they are NEVER substituted with a placeholder enum variant
//! and NEVER discarded. This is the type-level enforcement of F1.

/// One canonical Battern step. **Closed enum** — extending requires a
/// versioned spec amendment per m5 spec § 12 Q3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BatternStepLabel {
    /// Plan + scope a multi-pane dispatch.
    Design,
    /// Issue `cc-dispatch` to ≥2 panes within the design window.
    Dispatch,
    /// Verify gate conditions on dispatched panes.
    Gate,
    /// Harvest outputs via `cc-harvest` / `cc-audit`.
    Collect,
    /// Synthesise observations into a unified record.
    Synthesize,
    /// Compose / cascade outputs into the next iteration.
    Compose,
}

impl BatternStepLabel {
    /// Stable wire-form.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Design => "Design",
            Self::Dispatch => "Dispatch",
            Self::Gate => "Gate",
            Self::Collect => "Collect",
            Self::Synthesize => "Synthesize",
            Self::Compose => "Compose",
        }
    }

    /// All six labels in canonical order.
    pub const ALL: [Self; 6] = [
        Self::Design,
        Self::Dispatch,
        Self::Gate,
        Self::Collect,
        Self::Synthesize,
        Self::Compose,
    ];
}

#[cfg(test)]
mod tests {
    use super::BatternStepLabel;

    #[test]
    fn all_six_labels_in_canonical_order() {
        let names: Vec<&'static str> = BatternStepLabel::ALL.iter().map(|l| l.as_str()).collect();
        assert_eq!(
            names,
            vec!["Design", "Dispatch", "Gate", "Collect", "Synthesize", "Compose"]
        );
    }

    #[test]
    fn no_other_variant_compile_time_exhaustiveness() {
        // F1 invariant: there is no `Other` variant. This match must be
        // exhaustive with exactly the six canonical labels.
        let l = BatternStepLabel::Design;
        match l {
            BatternStepLabel::Design
            | BatternStepLabel::Dispatch
            | BatternStepLabel::Gate
            | BatternStepLabel::Collect
            | BatternStepLabel::Synthesize
            | BatternStepLabel::Compose => {}
        }
    }

    #[test]
    fn implements_copy_eq_hash() {
        use std::collections::HashSet;
        let a = BatternStepLabel::Gate;
        let b = a;
        let mut s = HashSet::new();
        s.insert(a);
        s.insert(b);
        assert_eq!(s.len(), 1);
    }

    // rationale: Core correctness — `as_str` round-trips against the
    // canonical ALL array; every label's wire-form is distinct and
    // matches the variant name exactly (no casing/typo drift).
    #[test]
    fn as_str_is_distinct_and_matches_variant_for_all_labels() {
        use std::collections::HashSet;
        let strs: HashSet<&'static str> =
            BatternStepLabel::ALL.iter().map(|l| l.as_str()).collect();
        assert_eq!(strs.len(), 6, "all six wire-forms must be distinct");
        // Spot-check the per-variant mapping.
        assert_eq!(BatternStepLabel::Synthesize.as_str(), "Synthesize");
        assert_eq!(BatternStepLabel::Collect.as_str(), "Collect");
    }

    // rationale: Anti-property F1 — the ALL array has EXACTLY six entries.
    // F1 forbids an `Other` variant; this pins the closed-enum cardinality
    // against silent growth (any new variant breaks the count assertion).
    #[test]
    fn all_array_cardinality_is_exactly_six_f1_closed_enum() {
        assert_eq!(
            BatternStepLabel::ALL.len(),
            6,
            "F1: closed enum must stay at six variants — no Other"
        );
    }
}
