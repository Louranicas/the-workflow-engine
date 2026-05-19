//! 7-trait Ember rubric heuristic scoring.
//!
//! Per m10 spec § 5: rubric reference at
//! `~/projects/claude_code/Ember 7-Trait Gate Rubric.md` (canonical;
//! NOT embedded in this module). Heuristics are pattern-matching
//! approximations of semantic concepts; the false-positive rate is
//! acceptable per spec § 13 Q3 and the allowlist provides operator-controlled
//! escape hatches with auditable `HumanAcceptanceSignature` per row.
//!
//! The scoring function evaluates traits in **deterministic order** —
//! Equanimity → Curiosity → Diligence → Honesty → Investment → Humility →
//! Warmth — and returns the FIRST failure encountered.
//! Confidence < 0.5 → [`EmberStatus::Held`]; ≥ 0.5 → [`EmberStatus::Rejected`].

/// One of the seven Ember traits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TraitName {
    /// Steady under load — no urgency theatre on routine status.
    Equanimity,
    /// Observe before assuming — claims carry measurement anchors.
    Curiosity,
    /// Quality-gate every claim — exact counts and named scopes.
    Diligence,
    /// Radical honesty — totalising claims carry enumeration.
    Honesty,
    /// This system matters — no filler decorations.
    Investment,
    /// Drift toward complexity is real — no absolutist verdicts.
    Humility,
    /// Clinical ethics — substrate modifications cite consent boundaries.
    Warmth,
}

impl TraitName {
    /// Stable identifier for log lines, metrics, and the `EMBER-REJECT` /
    /// `EMBER-HELD` stderr emissions.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Equanimity => "Equanimity",
            Self::Curiosity => "Curiosity",
            Self::Diligence => "Diligence",
            Self::Honesty => "Honesty",
            Self::Investment => "Investment",
            Self::Humility => "Humility",
            Self::Warmth => "Warmth",
        }
    }

    /// All seven traits in deterministic scoring order.
    pub const ALL: [Self; 7] = [
        Self::Equanimity,
        Self::Curiosity,
        Self::Diligence,
        Self::Honesty,
        Self::Investment,
        Self::Humility,
        Self::Warmth,
    ];
}

/// Verdict returned by [`score_against_rubric`] for a single candidate string.
#[derive(Debug, Clone, PartialEq)]
pub enum EmberStatus {
    /// All seven traits passed.
    Approved,
    /// A trait failed with confidence < 0.5 — held for manual review and
    /// CI-FAIL by default (operator may allowlist with
    /// `HumanAcceptanceSignature` per row).
    Held {
        /// Trait that fired the verdict.
        trait_name: TraitName,
        /// Plain-text explanation suitable for a developer to act on.
        reason: String,
        /// Heuristic confidence in the violation, `[0.0, 1.0]`.
        confidence: f64,
    },
    /// A trait failed with confidence ≥ 0.5 — rejected outright; cannot be
    /// allowlisted.
    Rejected {
        /// Trait that fired the verdict.
        trait_name: TraitName,
        /// Plain-text explanation.
        reason: String,
    },
}

/// Per-trait heuristic signature. A check returns `Some((trait_name,
/// reason, confidence))` when the candidate string fails the trait;
/// `None` when the trait passes.
type TraitCheck = fn(&str) -> Option<(TraitName, String, f64)>;

/// Score a candidate string against the 7-trait rubric.
///
/// Traits are checked in [`TraitName::ALL`] order; the first failure is
/// returned. A `confidence` ≥ 0.5 produces [`EmberStatus::Rejected`];
/// `< 0.5` produces [`EmberStatus::Held`]. All traits passing produces
/// [`EmberStatus::Approved`].
#[must_use]
pub fn score_against_rubric(text: &str) -> EmberStatus {
    let checks: [TraitCheck; 7] = [
        check_equanimity,
        check_curiosity,
        check_diligence,
        check_honesty,
        check_investment,
        check_humility,
        check_warmth,
    ];
    for check in checks {
        if let Some((trait_name, reason, confidence)) = check(text) {
            if confidence >= 0.5 {
                return EmberStatus::Rejected { trait_name, reason };
            }
            return EmberStatus::Held {
                trait_name,
                reason,
                confidence,
            };
        }
    }
    EmberStatus::Approved
}

// ---- Per-trait heuristic implementations --------------------------------

/// Equanimity: urgency theatre on non-critical text — all-caps routine
/// status, `!`-suffix on greens, urgency emoji without `critical`/`P0`
/// marker.
fn check_equanimity(text: &str) -> Option<(TraitName, String, f64)> {
    let lower = text.to_lowercase();
    let is_actually_critical = lower.contains("critical") || lower.contains("p0");
    for emoji in ['\u{1F6A8}', '\u{26A0}', '\u{2757}'] {
        if text.contains(emoji) && !is_actually_critical {
            return Some((
                TraitName::Equanimity,
                format!("urgency emoji {emoji:?} on non-critical text"),
                0.7,
            ));
        }
    }
    for word in text.split_whitespace() {
        let stripped: String = word
            .chars()
            .filter(|c| c.is_alphabetic())
            .collect();
        if stripped.len() >= 3 && stripped.chars().all(char::is_uppercase) {
            let routine = ["NOMINAL", "GREEN", "HEALTHY", "OK", "FINE", "GOOD"];
            if routine.iter().any(|r| stripped.eq_ignore_ascii_case(r)) {
                return Some((
                    TraitName::Equanimity,
                    format!("all-caps routine status word: {stripped}"),
                    0.6,
                ));
            }
        }
    }
    for token in text.split_whitespace() {
        if token.ends_with('!') {
            let bare = token.trim_end_matches('!').to_lowercase();
            if ["nominal", "green", "healthy", "ok", "fine"]
                .contains(&bare.as_str())
            {
                return Some((
                    TraitName::Equanimity,
                    format!("`!`-suffix on routine status: {token}"),
                    0.55,
                ));
            }
        }
    }
    None
}

/// Curiosity: claim strings without measurement anchors.
fn check_curiosity(text: &str) -> Option<(TraitName, String, f64)> {
    let lower = text.to_lowercase();
    if (lower.contains("status: healthy") || lower.contains("status: ok"))
        && !contains_measurement_anchor(text)
    {
        return Some((
            TraitName::Curiosity,
            "status claim without measurement anchor (probe timestamp or scope)".to_owned(),
            0.6,
        ));
    }
    None
}

fn contains_measurement_anchor(text: &str) -> bool {
    let lower = text.to_lowercase();
    if lower.contains("as of ")
        || lower.contains(" at 20")
        || lower.contains("probe")
        || lower.contains("ts=")
        || lower.contains("rfc3339")
        || lower.contains("scope=")
    {
        return true;
    }
    // Crude: 8+ ASCII digits anywhere suggests a timestamp or sample size.
    text.bytes().filter(u8::is_ascii_digit).count() >= 8
}

/// Diligence: round numbers without sample sizes; "passing" without a test
/// count; "clean" / "green" without a scope declaration.
fn check_diligence(text: &str) -> Option<(TraitName, String, f64)> {
    let lower = text.to_lowercase();
    if lower.contains("passing") && !text.bytes().any(|b| b.is_ascii_digit()) {
        return Some((
            TraitName::Diligence,
            "'passing' without test count".to_owned(),
            0.6,
        ));
    }
    if (lower.contains("clean") || lower.contains(" green"))
        && !lower.contains("clippy")
        && !lower.contains("check")
        && !lower.contains("test")
        && !lower.contains("gate")
        && !text.bytes().any(|b| b.is_ascii_digit())
    {
        return Some((
            TraitName::Diligence,
            "'clean'/'green' without gate scope declaration".to_owned(),
            0.55,
        ));
    }
    if (text.contains("~3000") || text.contains("~1000") || text.contains("100%"))
        && !lower.contains("test")
        && !lower.contains("sample")
        && !lower.contains("n=")
    {
        return Some((
            TraitName::Diligence,
            "round number without sample-size citation".to_owned(),
            0.55,
        ));
    }
    None
}

/// Honesty: totalising claims without enumeration.
fn check_honesty(text: &str) -> Option<(TraitName, String, f64)> {
    let lower = text.to_lowercase();
    if lower.contains("successfully completed")
        || lower.contains("all systems operational")
    {
        let has_enum = text.contains("- ")
            || text.contains("* ")
            || text.contains("\u{2022} ")
            || text.contains("1.")
            || text.contains("1)")
            || (text.contains(": ") && text.lines().count() >= 2);
        if !has_enum {
            return Some((
                TraitName::Honesty,
                "totalising claim without enumeration".to_owned(),
                0.7,
            ));
        }
    }
    None
}

/// Investment: filler phrases lacking information density.
fn check_investment(text: &str) -> Option<(TraitName, String, f64)> {
    let lower = text.to_lowercase();
    for filler in [
        "as you can see",
        "excellent",
        "great progress",
        "great work",
        "amazing",
    ] {
        if lower.contains(filler) {
            return Some((
                TraitName::Investment,
                format!("filler phrase: {filler}"),
                0.65,
            ));
        }
    }
    None
}

/// Humility: absolutist single-frame verdicts without alternatives.
///
/// Two confidence tiers:
/// - Strong absolutists ("the only path", "clearly the right", …) →
///   confidence 0.7 → `EmberStatus::Rejected` (CI-FAIL, no allowlist).
/// - Soft hedge-absolutists ("probably the best", "likely the right call",
///   "pretty much the only", "more or less the only") → confidence 0.4 →
///   `EmberStatus::Held` (CI-FAIL by default but allowlistable with
///   `HumanAcceptanceSignature` per D-C decision, S1002127).
///
/// The soft tier exists because hedged absolutism is a real Humility-trait
/// failure mode (the speaker is still collapsing alternatives, just with
/// social cushioning) but its false-positive surface — papers, retrospectives,
/// "probably the best path GIVEN constraint X" prose — is higher than the
/// strong tier. The Held tier routes it through the operator-controlled
/// escape hatch rather than blunt rejection. Day-1 wires the otherwise-dead
/// `Held` rubric branch end-to-end (closes H2 in carry-forward S1002600).
fn check_humility(text: &str) -> Option<(TraitName, String, f64)> {
    let lower = text.to_lowercase();
    for absolutist in [
        "the only path",
        "clearly the right",
        "obviously the",
        "definitely the best",
    ] {
        if lower.contains(absolutist) {
            return Some((
                TraitName::Humility,
                format!("absolutist phrase without alternatives: {absolutist}"),
                0.7,
            ));
        }
    }
    for soft_absolutist in [
        "probably the best",
        "likely the right call",
        "pretty much the only",
        "more or less the only",
    ] {
        if lower.contains(soft_absolutist) {
            return Some((
                TraitName::Humility,
                format!("soft-absolutist hedge collapsing alternatives: {soft_absolutist}"),
                0.4,
            ));
        }
    }
    None
}

/// Warmth: substrate modifications without ratification or AP27 citation;
/// `proceeding with X` without an explicit Luke ratification signal.
fn check_warmth(text: &str) -> Option<(TraitName, String, f64)> {
    let lower = text.to_lowercase();
    if lower.contains("proceeding with") {
        let has_ratification = lower.contains("luke")
            || lower.contains("approved")
            || lower.contains("ratified")
            || lower.contains("consent");
        if !has_ratification {
            return Some((
                TraitName::Warmth,
                "'proceeding with' without ratification signal".to_owned(),
                0.55,
            ));
        }
    }
    if (lower.contains("modify substrate") || lower.contains("substrate modification"))
        && !lower.contains("ap27")
    {
        return Some((
            TraitName::Warmth,
            "substrate modification proposal without AP27 boundary citation".to_owned(),
            0.6,
        ));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{score_against_rubric, EmberStatus, TraitName};

    // ---- TraitName surface (3) -----------------------------------------

    #[test]
    fn trait_name_as_str_is_stable_across_all_seven() {
        let expected = [
            "Equanimity",
            "Curiosity",
            "Diligence",
            "Honesty",
            "Investment",
            "Humility",
            "Warmth",
        ];
        for (t, e) in TraitName::ALL.iter().zip(expected.iter()) {
            assert_eq!(t.as_str(), *e);
        }
    }

    #[test]
    fn trait_name_all_has_exactly_seven_traits() {
        assert_eq!(TraitName::ALL.len(), 7);
    }

    #[test]
    fn trait_name_all_deduplicates_to_seven() {
        use std::collections::HashSet;
        let set: HashSet<_> = TraitName::ALL.iter().collect();
        assert_eq!(set.len(), 7);
    }

    // ---- Score happy path (2) -------------------------------------------

    #[test]
    fn approves_neutral_factual_string() {
        let v = score_against_rubric(
            "POVM probe at http://127.0.0.1:8125 returned 0.067 as of 2026-05-17T10:00:00Z (scope=lib).",
        );
        assert_eq!(v, EmberStatus::Approved);
    }

    #[test]
    fn approves_empty_string() {
        // Empty text has no failure modes to score; spec § 3 says new
        // strings MUST register, but the gate over a registered empty
        // string should pass vacuously.
        assert_eq!(score_against_rubric(""), EmberStatus::Approved);
    }

    // ---- Equanimity (4 pass + reject pairs) -----------------------------

    #[test]
    fn equanimity_pass_steady_status() {
        assert_eq!(
            score_against_rubric("status: healthy at 2026-05-17T10:00:00Z (probe m05)"),
            EmberStatus::Approved
        );
    }

    #[test]
    fn equanimity_reject_all_caps_routine_word() {
        let v = score_against_rubric("Pipeline is NOMINAL");
        let EmberStatus::Rejected { trait_name, reason } = v else {
            panic!("expected Rejected");
        };
        assert_eq!(trait_name, TraitName::Equanimity);
        assert!(reason.contains("NOMINAL"));
    }

    #[test]
    fn equanimity_reject_bang_suffix_on_green() {
        let v = score_against_rubric("Build green!");
        let EmberStatus::Rejected { trait_name, .. } = v else {
            panic!("expected Rejected (high confidence at 0.55? no — should be Held)");
        };
        let _ = trait_name;
    }

    #[test]
    fn equanimity_held_bang_suffix_uses_low_confidence() {
        // `!`-suffix returns confidence 0.55 → Rejected (≥ 0.5). The
        // boundary at 0.5 maps to Rejected. We verify the boundary.
        let v = score_against_rubric("Status nominal!");
        match v {
            EmberStatus::Rejected { trait_name, .. } => {
                assert_eq!(trait_name, TraitName::Equanimity);
            }
            other => panic!("expected Rejected at conf 0.55, got {other:?}"),
        }
    }

    #[test]
    fn equanimity_urgency_emoji_on_routine_text_rejected() {
        let v = score_against_rubric("All good \u{1F6A8} continuing.");
        let EmberStatus::Rejected { trait_name, .. } = v else {
            panic!("expected Rejected");
        };
        assert_eq!(trait_name, TraitName::Equanimity);
    }

    #[test]
    fn equanimity_urgency_emoji_on_critical_text_passes() {
        // P0 / critical context legitimises urgency emoji.
        let v = score_against_rubric("\u{1F6A8} P0 incident: substrate down");
        assert_eq!(v, EmberStatus::Approved);
    }

    // ---- Curiosity (3) --------------------------------------------------

    #[test]
    fn curiosity_pass_with_measurement_anchor() {
        let v = score_against_rubric("status: healthy probe at 2026-05-17");
        assert_eq!(v, EmberStatus::Approved);
    }

    #[test]
    fn curiosity_reject_bare_status_healthy() {
        let v = score_against_rubric("status: healthy");
        let EmberStatus::Rejected { trait_name, .. } = v else {
            panic!("expected Rejected at 0.6");
        };
        assert_eq!(trait_name, TraitName::Curiosity);
    }

    #[test]
    fn curiosity_reject_bare_status_ok() {
        let v = score_against_rubric("status: ok");
        let EmberStatus::Rejected { trait_name, .. } = v else {
            panic!("expected Rejected");
        };
        assert_eq!(trait_name, TraitName::Curiosity);
    }

    // ---- Diligence (4) --------------------------------------------------

    #[test]
    fn diligence_pass_passing_with_count() {
        let v = score_against_rubric("141 tests passing");
        assert_eq!(v, EmberStatus::Approved);
    }

    #[test]
    fn diligence_reject_passing_without_count() {
        let v = score_against_rubric("tests passing");
        let EmberStatus::Rejected { trait_name, .. } = v else {
            panic!("expected Rejected");
        };
        assert_eq!(trait_name, TraitName::Diligence);
    }

    #[test]
    fn diligence_reject_round_number_no_sample_size() {
        let v = score_against_rubric("~3000 commits this quarter");
        let EmberStatus::Rejected { trait_name, .. } = v else {
            panic!("expected Rejected at 0.55");
        };
        assert_eq!(trait_name, TraitName::Diligence);
    }

    #[test]
    fn diligence_pass_round_number_with_sample_anchor() {
        let v = score_against_rubric("~3000 tests sampled (n=64)");
        assert_eq!(v, EmberStatus::Approved);
    }

    // ---- Honesty (3) ----------------------------------------------------

    #[test]
    fn honesty_pass_totalising_with_enumeration() {
        let text = "successfully completed:\n- task A\n- task B";
        assert_eq!(score_against_rubric(text), EmberStatus::Approved);
    }

    #[test]
    fn honesty_reject_bare_totalising_claim() {
        let v = score_against_rubric("successfully completed");
        let EmberStatus::Rejected { trait_name, .. } = v else {
            panic!("expected Rejected at 0.7");
        };
        assert_eq!(trait_name, TraitName::Honesty);
    }

    #[test]
    fn honesty_reject_all_systems_operational_without_list() {
        let v = score_against_rubric("all systems operational");
        let EmberStatus::Rejected { trait_name, .. } = v else {
            panic!("expected Rejected at 0.7");
        };
        assert_eq!(trait_name, TraitName::Honesty);
    }

    // ---- Investment (3) -------------------------------------------------

    #[test]
    fn investment_pass_actionable_prose() {
        let v = score_against_rubric("Edit src/lib.rs:42 to add the missing import.");
        assert_eq!(v, EmberStatus::Approved);
    }

    #[test]
    fn investment_reject_filler_as_you_can_see() {
        let v = score_against_rubric("As you can see, the build is fixed.");
        let EmberStatus::Rejected { trait_name, .. } = v else {
            panic!("expected Rejected at 0.65");
        };
        assert_eq!(trait_name, TraitName::Investment);
    }

    #[test]
    fn investment_reject_filler_excellent() {
        let v = score_against_rubric("excellent timing on the deploy.");
        let EmberStatus::Rejected { trait_name, .. } = v else {
            panic!("expected Rejected");
        };
        assert_eq!(trait_name, TraitName::Investment);
    }

    // ---- Humility (3) ---------------------------------------------------

    #[test]
    fn humility_pass_names_alternatives() {
        let v = score_against_rubric(
            "Path A is preferred over path B because it requires fewer Luke decisions.",
        );
        assert_eq!(v, EmberStatus::Approved);
    }

    #[test]
    fn humility_reject_clearly_the_right() {
        let v = score_against_rubric("clearly the right call here.");
        let EmberStatus::Rejected { trait_name, .. } = v else {
            panic!("expected Rejected at 0.7");
        };
        assert_eq!(trait_name, TraitName::Humility);
    }

    #[test]
    fn humility_reject_the_only_path() {
        let v = score_against_rubric("This is the only path forward.");
        let EmberStatus::Rejected { trait_name, .. } = v else {
            panic!("expected Rejected at 0.7");
        };
        assert_eq!(trait_name, TraitName::Humility);
    }

    // ---- Warmth (4) -----------------------------------------------------

    #[test]
    fn warmth_pass_proceeding_with_ratification() {
        let v = score_against_rubric("proceeding with deploy per Luke approval at 2026-05-17.");
        assert_eq!(v, EmberStatus::Approved);
    }

    #[test]
    fn warmth_held_proceeding_without_ratification() {
        let v = score_against_rubric("proceeding with deploy.");
        // confidence 0.55 → Rejected at boundary; verify.
        let EmberStatus::Rejected { trait_name, .. } = v else {
            panic!("expected Rejected at 0.55");
        };
        assert_eq!(trait_name, TraitName::Warmth);
    }

    #[test]
    fn warmth_reject_substrate_modification_without_ap27() {
        let v = score_against_rubric("planning a substrate modification of POVM weights.");
        let EmberStatus::Rejected { trait_name, .. } = v else {
            panic!("expected Rejected at 0.6");
        };
        assert_eq!(trait_name, TraitName::Warmth);
    }

    #[test]
    fn warmth_pass_substrate_modification_with_ap27_citation() {
        let v = score_against_rubric(
            "Substrate modification gated by AP27 boundary; awaiting Luke ratification.",
        );
        assert_eq!(v, EmberStatus::Approved);
    }

    // ---- F-Property (5) -------------------------------------------------

    #[test]
    fn property_score_is_deterministic_across_repeats() {
        let text = "tests passing";
        let first = score_against_rubric(text);
        for _ in 0..1000_u32 {
            assert_eq!(score_against_rubric(text), first);
        }
    }

    #[test]
    fn property_first_failure_in_trait_order_wins() {
        // A text triggering BOTH Equanimity (all-caps NOMINAL) AND Honesty
        // (all systems operational) returns Equanimity (earlier in order).
        let v = score_against_rubric("NOMINAL: all systems operational");
        let EmberStatus::Rejected { trait_name, .. } = v else {
            panic!("expected Rejected");
        };
        assert_eq!(
            trait_name,
            TraitName::Equanimity,
            "deterministic ordering must return the first failure"
        );
    }

    #[test]
    fn property_clean_factual_strings_pass() {
        let factuals = [
            "POVM probe at 2026-05-17T10:00:00Z returned learning_health=0.067 (scope=lib).",
            "Build successful after edit at src/lib.rs:42; 141 tests sampled.",
            "RALPH fitness gen 7622 = 0.6987 (probe ts=2026-05-17T10:00:00Z).",
        ];
        for f in factuals {
            assert_eq!(score_against_rubric(f), EmberStatus::Approved, "input: {f}");
        }
    }

    #[test]
    fn property_idempotent_on_empty_text() {
        for _ in 0..100_u32 {
            assert_eq!(score_against_rubric(""), EmberStatus::Approved);
        }
    }

    #[test]
    fn property_confidence_boundary_at_0_5_maps_to_rejected() {
        // The boundary case: Warmth `proceeding with X` returns confidence
        // 0.55, which is ≥ 0.5 → Rejected. The Held arm activates below
        // 0.5 — currently exercised by the Humility soft-absolutist tier
        // (confidence 0.4); see `humility_held_soft_absolutist_*` tests
        // below.
        let v = score_against_rubric("proceeding with substrate write");
        assert!(matches!(v, EmberStatus::Rejected { .. }));
    }

    // ---- Humility Held branch (low-confidence soft absolutists) ---------

    #[test]
    fn humility_held_soft_absolutist_probably_the_best() {
        // Soft absolutist returns confidence 0.4 → Held (< 0.5), NOT
        // Rejected. Wires the Day-1 dead branch end-to-end (H2 closure).
        let v = score_against_rubric("This is probably the best approach.");
        match v {
            EmberStatus::Held {
                trait_name,
                confidence,
                ..
            } => {
                assert_eq!(trait_name, TraitName::Humility);
                assert!((confidence - 0.4).abs() < 1e-9, "expected conf 0.4, got {confidence}");
            }
            other => panic!("expected Held at conf 0.4, got {other:?}"),
        }
    }

    #[test]
    fn humility_held_soft_absolutist_likely_the_right_call() {
        let v = score_against_rubric("This is likely the right call given the trade-offs.");
        assert!(matches!(
            v,
            EmberStatus::Held {
                trait_name: TraitName::Humility,
                ..
            }
        ));
    }

    #[test]
    fn humility_held_soft_absolutist_pretty_much_the_only() {
        let v = score_against_rubric("That is pretty much the only viable path forward.");
        assert!(matches!(
            v,
            EmberStatus::Held {
                trait_name: TraitName::Humility,
                ..
            }
        ));
    }

    #[test]
    fn humility_strong_absolutist_still_rejected_not_held() {
        // Regression guard: the strong tier ("clearly the right") MUST
        // remain at confidence 0.7 → Rejected (no soft-tier downgrade).
        let v = score_against_rubric("This is clearly the right call.");
        assert!(matches!(
            v,
            EmberStatus::Rejected {
                trait_name: TraitName::Humility,
                ..
            }
        ));
    }

    // ---- F-Regression (2) -----------------------------------------------

    #[test]
    fn regression_critical_emoji_pass_branch() {
        // Regression slot: critical/P0 context legitimises urgency emoji.
        // Removing the "critical" / "P0" check would re-flag this string.
        let v = score_against_rubric("\u{1F6A8} CRITICAL: bridge down");
        assert_eq!(v, EmberStatus::Approved);
    }

    #[test]
    fn regression_enumeration_detection_for_dash_bullet() {
        // Regression slot: dash-bullet ('- ') counts as enumeration for
        // honesty check. Removing dash detection would re-flag the example.
        let v = score_against_rubric("successfully completed:\n- a\n- b");
        assert_eq!(v, EmberStatus::Approved);
    }
}
