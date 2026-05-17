//! Allowlist TSV reader with strict ISO-8601 expiry checks.
//!
//! Per m10 spec § 5 (hybrid CI-FAIL + allowlist, D-C decision, S1002127):
//! Held verdicts may be allowlisted with operator sign-off (`approved_by`
//! field is the `HumanAcceptanceSignature` anchor) and a non-expired
//! `expiry` timestamp. Rejected verdicts can NEVER be allowlisted — that
//! enforcement lives in [`super::gate`] (the gate only consults this
//! allowlist for Held verdicts).
//!
//! # TSV schema
//!
//! Tab-separated columns:
//!
//! ```text
//! artefact_key   approved_by   approved_at   expiry
//! m12.report     luke@node.0A  2026-05-17T10:00:00Z  2026-06-17T10:00:00Z
//! ```
//!
//! First line MUST start with `artefact_key` and is treated as a header.
//! Blank lines and lines starting with `#` are ignored.
//! Timestamps are RFC 3339 (UTC).
//!
//! File missing → empty allowlist (Ok). Malformed → typed parse error.

use std::fs;
use std::io;
use std::path::Path;

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use super::error::EmberGateError;

/// A single allowlist row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeldApproval {
    /// `<module>.<context>.<variant>` key matching a row in
    /// `crate::user_facing_strings::ALL`.
    pub artefact_key: String,
    /// Operator identity authoring the approval — this is the
    /// `HumanAcceptanceSignature` anchor per D-C decision.
    pub approved_by: String,
    /// When the approval was made (audit trail).
    pub approved_at: OffsetDateTime,
    /// When the approval ceases to permit the Held string through the gate.
    /// Strict inequality at check time: `now > expiry` → not approved.
    pub expiry: OffsetDateTime,
}

/// Load the allowlist from a TSV file.
///
/// Missing file is treated as an empty allowlist per spec § 5 (so a
/// brand-new repo without an `ember_held_approvals.tsv` boots in the
/// strictest configuration — every Held verdict CI-FAILs).
///
/// # Errors
///
/// - [`EmberGateError::AllowlistRead`] for any I/O error other than the file
///   being absent.
/// - [`EmberGateError::AllowlistParse`] for any line that does not contain
///   exactly four tab-separated fields, or where an RFC 3339 timestamp
///   fails to parse.
pub fn load_approvals<P: AsRef<Path>>(path: P) -> Result<Vec<HeldApproval>, EmberGateError> {
    let p = path.as_ref();
    let display = p.display().to_string();
    let content = match fs::read_to_string(p) {
        Ok(c) => c,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => return Err(EmberGateError::AllowlistRead { path: display, source }),
    };
    parse_tsv(&content, &display)
}

fn parse_tsv(content: &str, path: &str) -> Result<Vec<HeldApproval>, EmberGateError> {
    let mut out = Vec::new();
    for (idx, raw) in content.lines().enumerate() {
        let line = idx + 1;
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if line == 1 && trimmed.starts_with("artefact_key") {
            continue;
        }
        let fields: Vec<&str> = raw.split('\t').collect();
        if fields.len() != 4 {
            return Err(EmberGateError::AllowlistParse {
                path: path.to_owned(),
                line,
                reason: format!(
                    "expected 4 tab-separated fields, found {}",
                    fields.len()
                ),
            });
        }
        let artefact_key = fields[0].trim().to_owned();
        let approved_by = fields[1].trim().to_owned();
        let approved_at = OffsetDateTime::parse(fields[2].trim(), &Rfc3339).map_err(|e| {
            EmberGateError::AllowlistParse {
                path: path.to_owned(),
                line,
                reason: format!("approved_at parse failed: {e}"),
            }
        })?;
        let expiry = OffsetDateTime::parse(fields[3].trim(), &Rfc3339).map_err(|e| {
            EmberGateError::AllowlistParse {
                path: path.to_owned(),
                line,
                reason: format!("expiry parse failed: {e}"),
            }
        })?;
        out.push(HeldApproval {
            artefact_key,
            approved_by,
            approved_at,
            expiry,
        });
    }
    Ok(out)
}

/// True iff `approvals` contains a row whose `artefact_key == key` AND
/// whose `expiry > now` (strict).
#[must_use]
pub fn is_approved_at(approvals: &[HeldApproval], key: &str, now: OffsetDateTime) -> bool {
    approvals
        .iter()
        .any(|a| a.artefact_key == key && a.expiry > now)
}

/// Convenience: [`is_approved_at`] with `now = OffsetDateTime::now_utc()`.
#[must_use]
pub fn is_approved(approvals: &[HeldApproval], key: &str) -> bool {
    is_approved_at(approvals, key, OffsetDateTime::now_utc())
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::NamedTempFile;
    use time::macros::datetime;

    use super::{
        is_approved, is_approved_at, load_approvals, parse_tsv, HeldApproval,
    };
    use super::super::error::EmberGateError;

    fn write_tsv(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().expect("temp file");
        write!(file, "{content}").expect("write");
        file
    }

    // ---- TSV parsing happy paths (3) ------------------------------------

    #[test]
    fn parse_header_only_yields_empty_vec() {
        let content = "artefact_key\tapproved_by\tapproved_at\texpiry\n";
        let rows = parse_tsv(content, "test").expect("header only");
        assert!(rows.is_empty());
    }

    #[test]
    fn parse_one_row_after_header() {
        let content = "artefact_key\tapproved_by\tapproved_at\texpiry\n\
                       m12.report\tluke@0A\t2026-05-17T10:00:00Z\t2026-06-17T10:00:00Z\n";
        let rows = parse_tsv(content, "test").expect("one row");
        assert_eq!(rows.len(), 1);
        let r = &rows[0];
        assert_eq!(r.artefact_key, "m12.report");
        assert_eq!(r.approved_by, "luke@0A");
        assert_eq!(r.approved_at, datetime!(2026-05-17 10:00:00 UTC));
        assert_eq!(r.expiry, datetime!(2026-06-17 10:00:00 UTC));
    }

    #[test]
    fn parse_ignores_blank_lines_and_comments() {
        let content = "artefact_key\tapproved_by\tapproved_at\texpiry\n\
                       \n\
                       # comment line\n\
                       m12.report\tluke\t2026-05-17T10:00:00Z\t2026-06-17T10:00:00Z\n\
                       \n\
                       # trailing comment\n";
        let rows = parse_tsv(content, "test").expect("blanks/comments");
        assert_eq!(rows.len(), 1);
    }

    // ---- TSV parsing errors (3) -----------------------------------------

    #[test]
    fn parse_wrong_field_count_yields_typed_error() {
        let content = "artefact_key\tapproved_by\tapproved_at\texpiry\n\
                       m12.report\tluke\ttoo_few_fields\n";
        let err = parse_tsv(content, "test").expect_err("wrong field count");
        let EmberGateError::AllowlistParse { line, reason, .. } = err else {
            panic!("expected AllowlistParse");
        };
        assert_eq!(line, 2);
        assert!(reason.contains("expected 4"));
    }

    #[test]
    fn parse_malformed_approved_at_yields_typed_error() {
        let content = "artefact_key\tapproved_by\tapproved_at\texpiry\n\
                       m12.report\tluke\tnot-a-timestamp\t2026-06-17T10:00:00Z\n";
        let err = parse_tsv(content, "test").expect_err("bad timestamp");
        let EmberGateError::AllowlistParse { reason, .. } = err else {
            panic!("expected AllowlistParse");
        };
        assert!(reason.contains("approved_at parse failed"));
    }

    #[test]
    fn parse_malformed_expiry_yields_typed_error() {
        let content = "artefact_key\tapproved_by\tapproved_at\texpiry\n\
                       m12.report\tluke\t2026-05-17T10:00:00Z\tnever\n";
        let err = parse_tsv(content, "test").expect_err("bad expiry");
        let EmberGateError::AllowlistParse { reason, .. } = err else {
            panic!("expected AllowlistParse");
        };
        assert!(reason.contains("expiry parse failed"));
    }

    // ---- File loading (3) -----------------------------------------------

    #[test]
    fn load_approvals_from_temp_file_succeeds() {
        let f = write_tsv(
            "artefact_key\tapproved_by\tapproved_at\texpiry\n\
             m12.report\tluke\t2026-05-17T10:00:00Z\t2026-06-17T10:00:00Z\n",
        );
        let rows = load_approvals(f.path()).expect("load");
        assert_eq!(rows.len(), 1);
    }

    #[test]
    fn load_approvals_missing_file_returns_empty_vec() {
        let rows = load_approvals("/tmp/definitely-does-not-exist-9f3e7a1b").expect("missing");
        assert!(rows.is_empty());
    }

    #[test]
    fn load_approvals_propagates_io_error_other_than_not_found() {
        // Pointing at a directory yields an I/O error of kind that isn't
        // NotFound on most platforms.
        let result = load_approvals("/");
        assert!(result.is_err(), "expected I/O error reading a directory");
    }

    // ---- Expiry checks (5) ----------------------------------------------

    #[test]
    fn is_approved_at_future_expiry_true() {
        let approvals = vec![HeldApproval {
            artefact_key: "m12.report".into(),
            approved_by: "luke".into(),
            approved_at: datetime!(2026-05-17 10:00:00 UTC),
            expiry: datetime!(2030-01-01 00:00:00 UTC),
        }];
        let now = datetime!(2026-05-18 10:00:00 UTC);
        assert!(is_approved_at(&approvals, "m12.report", now));
    }

    #[test]
    fn is_approved_at_past_expiry_false() {
        let approvals = vec![HeldApproval {
            artefact_key: "m12.report".into(),
            approved_by: "luke".into(),
            approved_at: datetime!(2024-01-01 00:00:00 UTC),
            expiry: datetime!(2024-02-01 00:00:00 UTC),
        }];
        let now = datetime!(2026-05-17 10:00:00 UTC);
        assert!(!is_approved_at(&approvals, "m12.report", now));
    }

    #[test]
    fn is_approved_at_exact_expiry_false_strict() {
        // Strict: now == expiry is NOT approved (expiry is exclusive).
        let approvals = vec![HeldApproval {
            artefact_key: "m12.report".into(),
            approved_by: "luke".into(),
            approved_at: datetime!(2026-05-17 10:00:00 UTC),
            expiry: datetime!(2026-06-17 10:00:00 UTC),
        }];
        let exact = datetime!(2026-06-17 10:00:00 UTC);
        assert!(!is_approved_at(&approvals, "m12.report", exact));
    }

    #[test]
    fn is_approved_at_unknown_key_false() {
        let approvals = vec![HeldApproval {
            artefact_key: "m12.report".into(),
            approved_by: "luke".into(),
            approved_at: datetime!(2026-05-17 10:00:00 UTC),
            expiry: datetime!(2030-01-01 00:00:00 UTC),
        }];
        let now = datetime!(2026-05-18 10:00:00 UTC);
        assert!(!is_approved_at(&approvals, "m99.unknown", now));
    }

    #[test]
    fn is_approved_uses_now_utc_convenience() {
        // Smoke: convenience wrapper uses OffsetDateTime::now_utc().
        let approvals = vec![HeldApproval {
            artefact_key: "m12.report".into(),
            approved_by: "luke".into(),
            approved_at: datetime!(2024-01-01 00:00:00 UTC),
            expiry: datetime!(2024-02-01 00:00:00 UTC),
        }];
        // Past expiry → not approved at any plausible "now".
        assert!(!is_approved(&approvals, "m12.report"));
    }

    // ---- HeldApproval trait surface (2) ---------------------------------

    #[test]
    fn held_approval_implements_clone_eq_debug() {
        let a = HeldApproval {
            artefact_key: "k".into(),
            approved_by: "by".into(),
            approved_at: datetime!(2026-05-17 10:00:00 UTC),
            expiry: datetime!(2026-06-17 10:00:00 UTC),
        };
        let b = a.clone();
        assert_eq!(a, b);
        let s = format!("{a:?}");
        assert!(s.contains("HeldApproval"));
    }

    #[test]
    fn held_approval_unequal_when_key_differs() {
        let a = HeldApproval {
            artefact_key: "k1".into(),
            approved_by: "by".into(),
            approved_at: datetime!(2026-05-17 10:00:00 UTC),
            expiry: datetime!(2026-06-17 10:00:00 UTC),
        };
        let mut b = a.clone();
        b.artefact_key = "k2".into();
        assert_ne!(a, b);
    }
}
