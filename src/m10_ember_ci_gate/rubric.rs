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

    // ====================================================================
    // W4 FINAL mutation-kill pass (S1003529) — `check_warmth` survivors.
    //
    // `check_warmth` builds `has_ratification` from a 4-way `||` chain:
    //   lower.contains("luke")            // line 342
    //   || lower.contains("approved")     // line 343
    //   || lower.contains("ratified")     // line 344
    //   || lower.contains("consent")      // line 345
    // then `if !has_ratification { ... Warmth finding ... }`  // line 346
    //
    // The pre-existing Warmth tests never exercised a SINGLE ratification
    // keyword in isolation: `warmth_pass_proceeding_with_ratification`
    // uses "...per Luke approval..." which contains BOTH "luke" AND
    // "approved", so an `||`->`&&` mutation on any single link still
    // leaves the chain true (`luke` alone or `approved` alone keeps it
    // satisfied via the surviving `||`s). That is why 343/344/345 survive.
    // Each test below uses EXACTLY ONE ratification keyword so the
    // corresponding `&&` mutation flips `has_ratification` to false ->
    // the real-Approved string becomes Rejected under the mutant.
    // ====================================================================

    // rationale: KILLS `rubric.rs:343` `||` -> `&&`.
    // Text contains ONLY "approved" (no luke / ratified / consent).
    // Real chain: `luke(F) || approved(T) || ratified(F) || consent(F)`
    //   = true  -> has_ratification -> no Warmth finding -> Approved.
    // `343 &&` mutant: `(luke(F) && approved(T)) || ratified(F)
    //   || consent(F)` = false -> !false -> Warmth Rejected.
    #[test]
    fn mutkill_343_final_proceeding_with_only_approved_keyword_passes() {
        let v = score_against_rubric("proceeding with the rollout, approved.");
        assert_eq!(
            v,
            EmberStatus::Approved,
            "the lone ratification keyword 'approved' must satisfy \
             has_ratification; a 343 `||`->`&&` mutant rejects it"
        );
    }

    // rationale: KILLS `rubric.rs:344` `||` -> `&&`.
    // Text contains ONLY "ratified".
    // Real chain: `luke(F) || approved(F) || ratified(T) || consent(F)`
    //   = true -> Approved.
    // `344 &&` mutant: `luke(F) || (approved(F) && ratified(T))
    //   || consent(F)` = false -> Warmth Rejected.
    #[test]
    fn mutkill_344_final_proceeding_with_only_ratified_keyword_passes() {
        let v = score_against_rubric("proceeding with the rollout, ratified.");
        assert_eq!(
            v,
            EmberStatus::Approved,
            "the lone ratification keyword 'ratified' must satisfy \
             has_ratification; a 344 `||`->`&&` mutant rejects it"
        );
    }

    // rationale: KILLS `rubric.rs:345` `||` -> `&&`.
    // Text contains ONLY "consent".
    // Real chain: `luke(F) || approved(F) || ratified(F) || consent(T)`
    //   = true -> Approved.
    // `345 &&` mutant: `luke(F) || approved(F) || (ratified(F)
    //   && consent(T))` = false -> Warmth Rejected.
    #[test]
    fn mutkill_345_final_proceeding_with_only_consent_keyword_passes() {
        let v = score_against_rubric("proceeding with the rollout, consent.");
        assert_eq!(
            v,
            EmberStatus::Approved,
            "the lone ratification keyword 'consent' must satisfy \
             has_ratification; a 345 `||`->`&&` mutant rejects it"
        );
    }

    // rationale: KILLS `rubric.rs:346` delete `!` in `if !has_ratification`.
    // Text has the `proceeding with` trigger but NO ratification keyword.
    // Real code: `!has_ratification` = `!false` = true -> Warmth finding
    //   at confidence 0.55 -> Rejected.
    // `346` `!`-deletion mutant: `if has_ratification` = `if false` ->
    //   no Warmth finding -> second warmth block (substrate) also misses
    //   -> Approved.
    // We BOTH assert Rejected on the un-ratified text AND assert Approved
    // on a ratified counterpart, so a trivially-inverted gate cannot pass
    // by accident.
    #[test]
    fn mutkill_346_final_unratified_proceeding_is_rejected_ratified_is_approved() {
        // Un-ratified: real -> Rejected (Warmth); `!`-deletion -> Approved.
        let unratified = score_against_rubric("proceeding with the substrate rollout.");
        let EmberStatus::Rejected { trait_name, .. } = unratified else {
            panic!(
                "un-ratified 'proceeding with' must be Rejected by Warmth; \
                 a 346 `!`-deletion mutant would Approve it"
            );
        };
        assert_eq!(trait_name, TraitName::Warmth);

        // Ratified counterpart: real -> Approved (pins the gate is not
        // simply always-Rejected).
        let ratified = score_against_rubric(
            "proceeding with the rollout per Luke approval at 2026-05-17.",
        );
        assert_eq!(
            ratified,
            EmberStatus::Approved,
            "a ratified 'proceeding with' must still pass"
        );
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

    // ====================================================================
    // W4 mutation-kill pass (S1003529) — pins surviving cargo-mutants
    // mutants in rubric.rs. `contains_measurement_anchor`,
    // `check_diligence`, and `check_honesty` are private; they are
    // exercised through `score_against_rubric`. Curiosity is the only
    // trait that consults `contains_measurement_anchor`, so we drive it
    // via `status: healthy` strings that fail Curiosity UNLESS an
    // anchor is present.
    // ====================================================================

    // ---- contains_measurement_anchor (rubric.rs:190-203) ----------------

    // rationale: kills `rubric.rs:191` replace
    // `contains_measurement_anchor -> bool` with `false`. If the helper
    // always returned `false`, a `status: healthy` string carrying a
    // genuine anchor ("scope=") would STILL be rejected by Curiosity.
    // The real code returns `true` → the string is Approved.
    #[test]
    fn measurement_anchor_present_makes_status_healthy_pass() {
        // "scope=" is one anchor token; without the helper returning true
        // this would be a Curiosity Rejected.
        let v = score_against_rubric("status: healthy scope=lib");
        assert_eq!(
            v,
            EmberStatus::Approved,
            "a real measurement anchor must suppress the Curiosity rejection"
        );
    }

    // rationale: kills `rubric.rs:193-197` replace `||` with `&&` in the
    // anchor-token chain. Each clause must INDEPENDENTLY make the helper
    // return true. With `&&`, a string carrying only ONE token would no
    // longer count as anchored → Curiosity would reject it. We test each
    // of the six tokens in isolation, on a `status: healthy` carrier
    // that has NO ASCII-digit run (so the digit-count fallback at line
    // 202 cannot mask the result).
    #[test]
    fn measurement_anchor_each_token_independently_satisfies() {
        // Each carrier: `status: healthy` + exactly one anchor token,
        // no 8-digit run. Each must independently be Approved.
        let carriers = [
            ("status: healthy as of yesterday", "as of "),
            ("status: healthy at 20zz", " at 20"),
            ("status: healthy verified by probe run", "probe"),
            ("status: healthy ts=recent", "ts="),
            ("status: healthy per rfc3339 stamp", "rfc3339"),
            ("status: healthy scope=lib", "scope="),
        ];
        for (text, token) in carriers {
            assert_eq!(
                score_against_rubric(text),
                EmberStatus::Approved,
                "anchor token {token:?} alone must satisfy contains_measurement_anchor"
            );
        }
    }

    // rationale: kills `rubric.rs:202` replace `>=` with `<` in
    // `text.bytes().filter(u8::is_ascii_digit).count() >= 8`. EXACTLY 8
    // ASCII digits must count as an anchor (boundary). With `< 8` the
    // 8-digit case would FAIL the anchor check → Curiosity rejects.
    #[test]
    fn measurement_anchor_exactly_eight_digits_counts() {
        // "status: healthy" + exactly 8 digits, no other anchor token.
        let v = score_against_rubric("status: healthy 12345678");
        assert_eq!(
            v,
            EmberStatus::Approved,
            "exactly 8 ASCII digits must satisfy the anchor (>= 8 boundary)"
        );
    }

    // rationale: complements the above — pins the FALSE side of the
    // digit-count anchor. SEVEN digits is below the threshold and (with
    // no token anchor) must NOT count → Curiosity rejects the string.
    // Guards against a `< 8` mutant masquerading as correct on the
    // 8-digit case alone, and against the helper being stuck at `true`.
    #[test]
    fn measurement_anchor_seven_digits_does_not_count() {
        let v = score_against_rubric("status: healthy 1234567");
        let EmberStatus::Rejected { trait_name, .. } = v else {
            panic!("7 digits must NOT anchor → Curiosity Rejected, got {v:?}");
        };
        assert_eq!(trait_name, TraitName::Curiosity);
    }

    // ---- check_diligence (rubric.rs:207-241) ---------------------------

    // rationale: kills `rubric.rs:208` replace `check_diligence` with
    // `None`. A `passing` claim with no digit must fire Diligence. If
    // the whole function returned `None`, this would be Approved.
    #[test]
    fn check_diligence_fires_for_passing_without_count() {
        let v = score_against_rubric("the suite is passing");
        let EmberStatus::Rejected { trait_name, .. } = v else {
            panic!("'passing' without count must be Diligence Rejected, got {v:?}");
        };
        assert_eq!(trait_name, TraitName::Diligence);
    }

    // rationale: pins `rubric.rs:209` `&&` and the `!` on the digit
    // check — `passing` WITH a digit must NOT fire (digits present →
    // negated clause false → block skipped). Together with the test
    // above this brackets the Diligence block-1 boolean.
    #[test]
    fn check_diligence_passing_with_digit_does_not_fire() {
        // "412 tests passing" — has a digit; block 1 must not fire.
        let v = score_against_rubric("412 tests passing");
        assert_eq!(v, EmberStatus::Approved);
    }

    // rationale: kills `rubric.rs:217-221` delete `!` in the
    // `!lower.contains("clippy"/"check"/"test"/"gate")` exception chain
    // of Diligence block 2. The block fires for a digitless "clean"
    // string ONLY when NONE of those scope words appear. Deleting any
    // `!` inverts that clause: a string WITHOUT the word would then be
    // considered as-if it had the word and the block would not fire.
    // We test the firing case + one exception word per `!`.
    #[test]
    fn check_diligence_block2_fires_for_bare_clean() {
        // "clean" with no digit and no scope word → block 2 fires.
        let v = score_against_rubric("the working tree is clean");
        let EmberStatus::Rejected { trait_name, .. } = v else {
            panic!("bare 'clean' must be Diligence Rejected, got {v:?}");
        };
        assert_eq!(trait_name, TraitName::Diligence);
    }

    #[test]
    fn check_diligence_block2_suppressed_by_each_scope_word() {
        // Each scope word individually suppresses the bare-'clean'
        // rejection. Deleting the `!` on that word's clause (217-221)
        // would re-flag the string → Rejected. Real code → Approved.
        // No digits in any carrier so only block 2 is in play.
        for scope in ["clippy", "check", "test", "gate"] {
            let text = format!("the {scope} run came back clean");
            assert_eq!(
                score_against_rubric(&text),
                EmberStatus::Approved,
                "scope word {scope:?} must suppress the bare-'clean' Diligence rejection"
            );
        }
    }

    // rationale: pins the `!text.bytes().any(...is_ascii_digit)` clause
    // of Diligence block 2 (line 221's digit guard) — a digit present in
    // a "clean" string suppresses block 2.
    #[test]
    fn check_diligence_block2_suppressed_by_digit() {
        // "clean" with a digit and no scope word: block 2's digit clause
        // suppresses it. (No round-number token, so block 3 stays quiet.)
        let v = score_against_rubric("7 files clean");
        assert_eq!(v, EmberStatus::Approved);
    }

    // rationale: kills `rubric.rs:229` replace `||` with `&&` in
    // `(text.contains("~3000") || text.contains("~1000") ||
    // text.contains("100%"))`. Each round-number token must
    // INDEPENDENTLY arm block 3. With `&&` a string carrying only one
    // token would no longer fire. We test all three in isolation. No
    // "test"/"sample"/"n=" present so the negated guards stay true.
    #[test]
    fn check_diligence_block3_each_round_number_token_fires() {
        for token in ["~3000", "~1000", "100%"] {
            let text = format!("about {token} commits this quarter");
            let v = score_against_rubric(&text);
            let EmberStatus::Rejected { trait_name, .. } = v else {
                panic!("round-number token {token:?} must fire Diligence, got {v:?}");
            };
            assert_eq!(
                trait_name,
                TraitName::Diligence,
                "round-number token {token:?} alone must arm Diligence block 3"
            );
        }
    }

    // rationale: kills `rubric.rs:230-232` replace `&&` with `||` and
    // `230:12`/`231:12` delete `!`. Block 3 fires only when a round
    // number is present AND none of "test"/"sample"/"n=" appear. Each
    // of those words individually suppresses the rejection. With `&&`→
    // `||` the chain would collapse; with a deleted `!` the suppression
    // would invert. Real code → each carrier is Approved.
    #[test]
    fn check_diligence_block3_suppressed_by_each_sample_word() {
        // "~3000" round number + one suppressor word each.
        let carriers = [
            "~3000 cases under test",
            "~3000 rows in the sample",
            "~3000 events (n=64)",
        ];
        for text in carriers {
            assert_eq!(
                score_against_rubric(text),
                EmberStatus::Approved,
                "a sample-size word must suppress the round-number Diligence rejection: {text:?}"
            );
        }
    }

    // ---- check_honesty (rubric.rs:244-264) -----------------------------

    // rationale: kills `rubric.rs:247` replace `||` with `&&` in
    // `lower.contains("successfully completed") ||
    // lower.contains("all systems operational")`. Each totalising
    // phrase must INDEPENDENTLY arm the Honesty check. With `&&` a
    // string with only one phrase would no longer be examined.
    #[test]
    fn check_honesty_each_totalising_phrase_fires_independently() {
        for phrase in ["successfully completed", "all systems operational"] {
            let v = score_against_rubric(phrase);
            let EmberStatus::Rejected { trait_name, .. } = v else {
                panic!("totalising phrase {phrase:?} must fire Honesty, got {v:?}");
            };
            assert_eq!(
                trait_name,
                TraitName::Honesty,
                "totalising phrase {phrase:?} alone must arm the Honesty check"
            );
        }
    }

    // rationale: kills `rubric.rs:250-254` replace `||` with `&&` in the
    // `has_enum` chain. Each enumeration marker must INDEPENDENTLY make
    // `has_enum` true, which SUPPRESSES the Honesty rejection. With `&&`
    // a string carrying only one marker would still be flagged. Real
    // code → each carrier is Approved.
    #[test]
    fn check_honesty_each_enumeration_marker_suppresses() {
        // "successfully completed" + exactly one enumeration marker.
        let carriers = [
            "successfully completed\n- item one",          // "- "
            "successfully completed\n* item one",          // "* "
            "successfully completed\n\u{2022} item one",   // bullet "• "
            "successfully completed\n1. item one",         // "1."
            "successfully completed\n1) item one",         // "1)"
        ];
        for text in carriers {
            assert_eq!(
                score_against_rubric(text),
                EmberStatus::Approved,
                "an enumeration marker must suppress the Honesty rejection: {text:?}"
            );
        }
    }

    // rationale: kills `rubric.rs:254` replace `&&` with `||` AND
    // replace `>=` with `<` in the final `has_enum` clause
    // `(text.contains(": ") && text.lines().count() >= 2)`. This clause
    // is true only when a colon-space is present AND the text spans at
    // least 2 lines. We pin both halves:
    //  - colon-space + 2 lines → suppressed (Approved). A `>=`→`<`
    //    mutant would make the 2-line case false → Rejected.
    //  - colon-space on a SINGLE line → NOT suppressed (Rejected). A
    //    `&&`→`||` mutant would suppress on the colon alone → Approved.
    #[test]
    fn check_honesty_colon_clause_requires_colon_and_two_lines() {
        // Colon-space + 2 lines → has_enum true → suppressed.
        let two_line = "successfully completed\nresult: ok";
        assert_eq!(
            score_against_rubric(two_line),
            EmberStatus::Approved,
            "colon-space across >= 2 lines must suppress the Honesty rejection"
        );
        // Colon-space but only ONE line → has_enum false → still flagged.
        let one_line = "successfully completed: result ok";
        let v = score_against_rubric(one_line);
        let EmberStatus::Rejected { trait_name, .. } = v else {
            panic!("single-line colon must NOT suppress Honesty, got {v:?}");
        };
        assert_eq!(trait_name, TraitName::Honesty);
    }
}
