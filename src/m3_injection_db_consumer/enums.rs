//! Closed-set enum translators for injection.db string columns.
//!
//! Per m3 spec § 1 invariant 2: raw string columns `chain_type` and
//! `consent` are mapped to closed [`ChainType`] / [`ConsentLevel`] enums
//! at parse time. This prevents downstream modules from drifting into
//! stringly-typed handling.
//!
//! # Live-schema discovery
//!
//! `chain_type` values are LOWERCASE in the live schema's `CHECK` clause:
//! `('bug', 'trap', 'plan', 'pattern')`. The paper spec § 2 capitalised
//! them; this module uses the live schema as source of truth.
//! `consent` values are capitalised: `('Emit', 'Store', 'Forget')`.

use super::error::InjectionDbError;

/// Chain category enumerated by injection.db's `chain_type` CHECK
/// constraint. Closed set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChainType {
    /// `bug` — defects under active or potential remediation.
    Bug,
    /// `trap` — antipatterns / repeated-mistake markers.
    Trap,
    /// `plan` — multi-session work items.
    Plan,
    /// `pattern` — recurring shape (positive or negative) worth surfacing.
    Pattern,
}

impl ChainType {
    /// Stable wire-form (mirrors the DB CHECK constraint exactly).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bug => "bug",
            Self::Trap => "trap",
            Self::Plan => "plan",
            Self::Pattern => "pattern",
        }
    }
}

/// Consent posture per `Ember` substrate ethics: whether the engine may
/// emit / store / must forget the row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConsentLevel {
    /// `Emit` — downstream consumers may freely read and act on the row.
    Emit,
    /// `Store` — substrate may retain the row but downstream emission is
    /// restricted (preserve-list discipline AP-Hab-04).
    Store,
    /// `Forget` — preserve-list says do-not-propagate; m3 filters these
    /// at SQL level so they never appear downstream.
    Forget,
}

impl ConsentLevel {
    /// Stable wire-form (mirrors the DB CHECK constraint exactly).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Emit => "Emit",
            Self::Store => "Store",
            Self::Forget => "Forget",
        }
    }
}

/// Parse a `chain_type` string into a [`ChainType`].
///
/// # Errors
///
/// [`InjectionDbError::UnknownChainType`] preserving the bad input string
/// for diagnostic display.
pub fn parse_chain_type(value: &str) -> Result<ChainType, InjectionDbError> {
    match value {
        "bug" => Ok(ChainType::Bug),
        "trap" => Ok(ChainType::Trap),
        "plan" => Ok(ChainType::Plan),
        "pattern" => Ok(ChainType::Pattern),
        other => Err(InjectionDbError::UnknownChainType(other.to_owned())),
    }
}

/// Parse a `consent` string into a [`ConsentLevel`].
///
/// # Errors
///
/// [`InjectionDbError::UnknownConsent`] preserving the bad input string.
pub fn parse_consent(value: &str) -> Result<ConsentLevel, InjectionDbError> {
    match value {
        "Emit" => Ok(ConsentLevel::Emit),
        "Store" => Ok(ConsentLevel::Store),
        "Forget" => Ok(ConsentLevel::Forget),
        other => Err(InjectionDbError::UnknownConsent(other.to_owned())),
    }
}

#[cfg(test)]
mod tests {
    use super::super::error::InjectionDbError;
    use super::{parse_chain_type, parse_consent, ChainType, ConsentLevel};

    // ---- ChainType happy paths (4) --------------------------------------

    #[test]
    fn chain_type_bug_parses_from_lowercase() {
        assert_eq!(parse_chain_type("bug").unwrap(), ChainType::Bug);
    }

    #[test]
    fn chain_type_trap_parses_from_lowercase() {
        assert_eq!(parse_chain_type("trap").unwrap(), ChainType::Trap);
    }

    #[test]
    fn chain_type_plan_parses_from_lowercase() {
        assert_eq!(parse_chain_type("plan").unwrap(), ChainType::Plan);
    }

    #[test]
    fn chain_type_pattern_parses_from_lowercase() {
        assert_eq!(parse_chain_type("pattern").unwrap(), ChainType::Pattern);
    }

    // ---- ChainType rejections (3) ---------------------------------------

    #[test]
    fn chain_type_uppercase_is_rejected() {
        let err = parse_chain_type("BUG").unwrap_err();
        let InjectionDbError::UnknownChainType(value) = err else {
            panic!("expected UnknownChainType");
        };
        assert_eq!(value, "BUG");
    }

    #[test]
    fn chain_type_empty_string_is_rejected() {
        assert!(matches!(
            parse_chain_type(""),
            Err(InjectionDbError::UnknownChainType(ref v)) if v.is_empty()
        ));
    }

    #[test]
    fn chain_type_unknown_value_preserves_input() {
        let err = parse_chain_type("incident").unwrap_err();
        let InjectionDbError::UnknownChainType(value) = err else {
            panic!("expected UnknownChainType");
        };
        assert_eq!(value, "incident");
    }

    // ---- ConsentLevel happy paths (3) -----------------------------------

    #[test]
    fn consent_emit_parses_from_capitalised() {
        assert_eq!(parse_consent("Emit").unwrap(), ConsentLevel::Emit);
    }

    #[test]
    fn consent_store_parses_from_capitalised() {
        assert_eq!(parse_consent("Store").unwrap(), ConsentLevel::Store);
    }

    #[test]
    fn consent_forget_parses_from_capitalised() {
        assert_eq!(parse_consent("Forget").unwrap(), ConsentLevel::Forget);
    }

    // ---- ConsentLevel rejections (2) ------------------------------------

    #[test]
    fn consent_lowercase_is_rejected() {
        assert!(matches!(
            parse_consent("emit"),
            Err(InjectionDbError::UnknownConsent(ref v)) if v == "emit"
        ));
    }

    #[test]
    fn consent_unknown_value_preserves_input() {
        let err = parse_consent("Decide").unwrap_err();
        let InjectionDbError::UnknownConsent(value) = err else {
            panic!("expected UnknownConsent");
        };
        assert_eq!(value, "Decide");
    }

    // ---- as_str round-trip (3) ------------------------------------------

    #[test]
    fn chain_type_as_str_round_trips() {
        for t in [
            ChainType::Bug,
            ChainType::Trap,
            ChainType::Plan,
            ChainType::Pattern,
        ] {
            assert_eq!(parse_chain_type(t.as_str()).unwrap(), t);
        }
    }

    #[test]
    fn consent_level_as_str_round_trips() {
        for c in [ConsentLevel::Emit, ConsentLevel::Store, ConsentLevel::Forget] {
            assert_eq!(parse_consent(c.as_str()).unwrap(), c);
        }
    }

    #[test]
    fn as_str_values_match_db_check_constraints() {
        // Live schema CHECK clause: ('bug', 'trap', 'plan', 'pattern')
        // and ('Emit', 'Store', 'Forget'). Drift here would silently
        // break SQL-level filtering.
        assert_eq!(ChainType::Bug.as_str(), "bug");
        assert_eq!(ChainType::Trap.as_str(), "trap");
        assert_eq!(ConsentLevel::Forget.as_str(), "Forget");
    }
}
