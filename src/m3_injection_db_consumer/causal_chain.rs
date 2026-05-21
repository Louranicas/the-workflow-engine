//! `CausalChainRow` + `ChainId` + `ChainLabel` newtypes + row parser.
//!
//! Per m3 spec § 1: a workflow-trace-local mirror of injection.db's
//! `causal_chain` row. Distinct type — workflow-trace never imports the
//! `memory-injection` crate's struct, preventing accidental write-side
//! coupling.
//!
//! # Live-schema discovery
//!
//! - `id INTEGER PRIMARY KEY AUTOINCREMENT` — i64 in rusqlite, exposed
//!   verbatim through the [`ChainId`] newtype.
//! - `origin_session INTEGER NOT NULL` — paper spec said `u32`; live
//!   schema is just `INTEGER` (i64). We narrow to `u32` at parse time;
//!   negative or overflowing values surface as
//!   [`InjectionDbError::RowParseFailed`].
//! - `created_at TEXT` and `updated_at TEXT` exist in the live schema
//!   but the spec § 2 does not list them — m3 ignores them at SELECT
//!   time (not in the column list).

use rusqlite::Row;

use super::enums::{parse_chain_type, parse_consent, ChainType, ConsentLevel};
use super::error::InjectionDbError;

/// Primary-key newtype for `causal_chain.id`.
///
/// The inner value is **private**: the newtype's opacity is enforced by
/// the type system. Construct via [`ChainId::new`] and read via
/// [`ChainId::get`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChainId(i64);

impl ChainId {
    /// Construct a `ChainId` from a `causal_chain.id` primary key.
    ///
    /// The value is stored verbatim; `causal_chain.id` is
    /// `INTEGER PRIMARY KEY AUTOINCREMENT`, so no invariant is enforced.
    #[must_use]
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    /// Return the inner primary-key value.
    #[must_use]
    pub const fn get(self) -> i64 {
        self.0
    }
}

impl std::fmt::Display for ChainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Stable, opaque chain identifier (the `label` column). Newtype enforces
/// "this is a stable habitat tag, not natural-language prose for the
/// engine to reason about" — m3 spec § 1 Watcher Class-G mitigation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChainLabel(String);

impl ChainLabel {
    /// Construct without validation. The label is a stable cross-session
    /// identifier; m3 preserves it byte-for-byte.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Borrow the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ChainLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// One `causal_chain` row.
#[derive(Debug, Clone)]
pub struct CausalChainRow {
    /// Primary key (newtype-wrapped).
    pub id: ChainId,
    /// Session in which the chain was first reinforced (`u32`).
    pub origin_session: u32,
    /// Session in which the chain was resolved (`None` for open chains).
    pub resolved_session: Option<u32>,
    /// Closed-set chain category.
    pub chain_type: ChainType,
    /// Stable habitat tag.
    pub label: ChainLabel,
    /// Free-form description verbatim.
    pub description: String,
    /// Reinforcement count (≥ 1 by schema default).
    pub reinforcement_count: u32,
    /// Last session in which the chain was reinforced.
    pub last_reinforced_session: Option<u32>,
    /// Consent posture.
    pub consent: ConsentLevel,
}

/// Parse one rusqlite row into a [`CausalChainRow`].
///
/// Column order MUST match the SELECT in `query.rs`:
///
/// ```text
/// SELECT id, origin_session, resolved_session, chain_type, label,
///        description, reinforcement_count, last_reinforced_session, consent
/// FROM causal_chain
/// ```
///
/// # Errors
///
/// - [`InjectionDbError::RowParseFailed`] on column type-mismatch or
///   `u32` overflow.
/// - [`InjectionDbError::UnknownChainType`] / [`InjectionDbError::UnknownConsent`]
///   on values outside the schema's CHECK set.
pub fn parse_causal_chain_row(row: &Row<'_>) -> Result<CausalChainRow, InjectionDbError> {
    let id: i64 = row
        .get(0)
        .map_err(|e| InjectionDbError::RowParseFailed {
            row_id: -1,
            reason: format!("id column: {e}"),
        })?;
    let origin_session_i: i64 =
        row.get(1)
            .map_err(|e| InjectionDbError::RowParseFailed {
                row_id: id,
                reason: format!("origin_session column: {e}"),
            })?;
    let origin_session = u32::try_from(origin_session_i).map_err(|_| {
        InjectionDbError::RowParseFailed {
            row_id: id,
            reason: format!("origin_session {origin_session_i} does not fit in u32"),
        }
    })?;
    let resolved_session_i: Option<i64> =
        row.get(2)
            .map_err(|e| InjectionDbError::RowParseFailed {
                row_id: id,
                reason: format!("resolved_session column: {e}"),
            })?;
    let resolved_session = match resolved_session_i {
        None => None,
        Some(v) => {
            Some(u32::try_from(v).map_err(|_| InjectionDbError::RowParseFailed {
                row_id: id,
                reason: format!("resolved_session {v} does not fit in u32"),
            })?)
        }
    };
    let chain_type_str: String =
        row.get(3)
            .map_err(|e| InjectionDbError::RowParseFailed {
                row_id: id,
                reason: format!("chain_type column: {e}"),
            })?;
    let chain_type = parse_chain_type(&chain_type_str)?;
    let label_str: String = row
        .get(4)
        .map_err(|e| InjectionDbError::RowParseFailed {
            row_id: id,
            reason: format!("label column: {e}"),
        })?;
    let description: String = row
        .get(5)
        .map_err(|e| InjectionDbError::RowParseFailed {
            row_id: id,
            reason: format!("description column: {e}"),
        })?;
    let reinforcement_count_i: i64 =
        row.get(6)
            .map_err(|e| InjectionDbError::RowParseFailed {
                row_id: id,
                reason: format!("reinforcement_count column: {e}"),
            })?;
    let reinforcement_count = u32::try_from(reinforcement_count_i).map_err(|_| {
        InjectionDbError::RowParseFailed {
            row_id: id,
            reason: format!("reinforcement_count {reinforcement_count_i} does not fit in u32"),
        }
    })?;
    let last_reinforced_session_i: Option<i64> =
        row.get(7)
            .map_err(|e| InjectionDbError::RowParseFailed {
                row_id: id,
                reason: format!("last_reinforced_session column: {e}"),
            })?;
    let last_reinforced_session = match last_reinforced_session_i {
        None => None,
        Some(v) => {
            Some(u32::try_from(v).map_err(|_| InjectionDbError::RowParseFailed {
                row_id: id,
                reason: format!("last_reinforced_session {v} does not fit in u32"),
            })?)
        }
    };
    let consent_str: String =
        row.get(8)
            .map_err(|e| InjectionDbError::RowParseFailed {
                row_id: id,
                reason: format!("consent column: {e}"),
            })?;
    let consent = parse_consent(&consent_str)?;
    Ok(CausalChainRow {
        id: ChainId::new(id),
        origin_session,
        resolved_session,
        chain_type,
        label: ChainLabel::new(label_str),
        description,
        reinforcement_count,
        last_reinforced_session,
        consent,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{CausalChainRow, ChainId, ChainLabel};
    use super::super::enums::{ChainType, ConsentLevel};

    #[test]
    fn chain_id_display_emits_integer() {
        assert_eq!(format!("{}", ChainId::new(42)), "42");
    }

    #[test]
    fn chain_id_implements_copy_eq_hash() {
        let a = ChainId::new(1);
        let b = a;
        let mut s = HashSet::new();
        s.insert(a);
        s.insert(b);
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn chain_label_round_trip_via_as_str() {
        let l = ChainLabel::new("BUG-001-devenv-stop");
        assert_eq!(l.as_str(), "BUG-001-devenv-stop");
        assert_eq!(format!("{l}"), "BUG-001-devenv-stop");
    }

    #[test]
    fn chain_label_eq_hash() {
        let mut s = HashSet::new();
        s.insert(ChainLabel::new("x"));
        s.insert(ChainLabel::new("x"));
        s.insert(ChainLabel::new("y"));
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn causal_chain_row_clone_preserves_all_fields() {
        let r = CausalChainRow {
            id: ChainId::new(7),
            origin_session: 100,
            resolved_session: Some(110),
            chain_type: ChainType::Bug,
            label: ChainLabel::new("BUG-007"),
            description: "desc".into(),
            reinforcement_count: 4,
            last_reinforced_session: Some(108),
            consent: ConsentLevel::Emit,
        };
        let c = r.clone();
        assert_eq!(c.id, r.id);
        assert_eq!(c.chain_type, r.chain_type);
        assert_eq!(c.consent, r.consent);
        assert_eq!(c.label, r.label);
    }
}
