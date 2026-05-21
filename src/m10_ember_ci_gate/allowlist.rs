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

    // ====================================================================
    // W4 mutation-kill pass (S1003529) — pins surviving cargo-mutants
    // mutants in allowlist.rs.
    // ====================================================================

    // rationale: kills `allowlist.rs:77` replace `+` with `*`/`-` in
    // `let line = idx + 1`. A parse error on the SECOND data row (file
    // line 3) must report `line == 3`. With `idx * 1` the third line
    // (idx 2) reports 2; with `idx - 1` it reports 1; only `idx + 1`
    // reports 3. The header is line 1, first data row line 2, the
    // malformed row line 3.
    #[test]
    fn parse_error_line_number_is_idx_plus_one_not_times_or_minus() {
        let content = "artefact_key\tapproved_by\tapproved_at\texpiry\n\
                       m12.report\tluke\t2026-05-17T10:00:00Z\t2026-06-17T10:00:00Z\n\
                       m13.report\tonly_two_fields\n";
        let err = parse_tsv(content, "test").expect_err("malformed third line");
        let EmberGateError::AllowlistParse { line, .. } = err else {
            panic!("expected AllowlistParse");
        };
        // idx of the malformed row is 2 → line must be 3 (2 + 1).
        assert_eq!(line, 3, "line must be idx + 1, not idx * 1 (=2) or idx - 1 (=1)");
    }

    // rationale: kills `allowlist.rs:79` replace `||` with `&&` in
    // `trimmed.is_empty() || trimmed.starts_with('#')`. A blank line is
    // empty-but-not-#; a comment line is #-but-not-empty. With `&&`,
    // NEITHER would be skipped — both would fall through to the
    // field-count check and raise AllowlistParse. With `||`, both are
    // correctly skipped and parsing succeeds.
    #[test]
    fn parse_skips_blank_line_which_is_empty_but_not_hash() {
        // A lone blank line: is_empty() == true, starts_with('#') == false.
        // Under `&&` this would NOT be skipped → parse error. Under `||`
        // it is skipped → Ok.
        let content = "artefact_key\tapproved_by\tapproved_at\texpiry\n\
                       \n\
                       m12.report\tluke\t2026-05-17T10:00:00Z\t2026-06-17T10:00:00Z\n";
        let rows = parse_tsv(content, "test").expect("blank line must be skipped");
        assert_eq!(rows.len(), 1);
    }

    #[test]
    fn parse_skips_comment_line_which_is_hash_but_not_empty() {
        // A comment line: is_empty() == false, starts_with('#') == true.
        // Under `&&` this would NOT be skipped → parse error (the comment
        // text has no tabs). Under `||` it is skipped → Ok.
        let content = "artefact_key\tapproved_by\tapproved_at\texpiry\n\
                       # this is a non-empty comment line\n\
                       m12.report\tluke\t2026-05-17T10:00:00Z\t2026-06-17T10:00:00Z\n";
        let rows = parse_tsv(content, "test").expect("comment line must be skipped");
        assert_eq!(rows.len(), 1);
    }

    // rationale: kills `allowlist.rs:82` replace `&&` with `||` in
    // `line == 1 && trimmed.starts_with("artefact_key")`. A genuine
    // DATA row whose artefact_key column literally equals
    // `artefact_key` (on file line 2, not line 1) must NOT be treated
    // as a header — it must parse into a row. With `||` the line-1
    // condition is dropped, so any line starting with "artefact_key"
    // (including this real data row) is silently skipped → 0 rows.
    #[test]
    fn parse_does_not_skip_data_row_keyed_artefact_key_on_line_two() {
        let content = "artefact_key\tapproved_by\tapproved_at\texpiry\n\
                       artefact_key\tluke\t2026-05-17T10:00:00Z\t2026-06-17T10:00:00Z\n";
        let rows = parse_tsv(content, "test").expect("data row must parse");
        assert_eq!(
            rows.len(),
            1,
            "a data row literally keyed 'artefact_key' on line 2 must NOT be header-skipped"
        );
        assert_eq!(rows[0].artefact_key, "artefact_key");
    }

    // rationale: also pins `allowlist.rs:82` — the header on line 1
    // that DOESN'T start with "artefact_key" must NOT be skipped (it
    // would be parsed as data). Under `||` a line-1 non-header would be
    // skipped by the dropped `line == 1` arm. This complements the
    // above: header detection requires BOTH conditions.
    #[test]
    fn parse_line_one_without_artefact_key_prefix_is_treated_as_data() {
        // First line does not start with "artefact_key" → it is a data
        // row, not a header. Under `||`, `line == 1` alone would skip it.
        let content = "m12.report\tluke\t2026-05-17T10:00:00Z\t2026-06-17T10:00:00Z\n";
        let rows = parse_tsv(content, "test").expect("line-1 non-header parses as data");
        assert_eq!(rows.len(), 1, "line-1 non-header row must parse as data");
        assert_eq!(rows[0].artefact_key, "m12.report");
    }

    // rationale: kills `allowlist.rs:134` replace `is_approved -> bool`
    // with `false`. With a future-dated, key-matching approval,
    // `is_approved` must return TRUE. The `false`-replacement mutant
    // would fail this; the real code passes.
    #[test]
    fn is_approved_returns_true_for_future_dated_matching_row() {
        let approvals = vec![HeldApproval {
            artefact_key: "m12.report".into(),
            approved_by: "luke".into(),
            approved_at: datetime!(2026-01-01 00:00:00 UTC),
            // Far-future expiry so the wall-clock `now` is always before it.
            expiry: datetime!(2999-01-01 00:00:00 UTC),
        }];
        assert!(
            is_approved(&approvals, "m12.report"),
            "is_approved must return true for a far-future matching approval"
        );
    }

    // rationale: kills `allowlist.rs:134` replace `is_approved -> bool`
    // with `true`. With NO approvals, `is_approved` must return FALSE.
    // The `true`-replacement mutant would fail this; the real code
    // (and `is_approved_uses_now_utc_convenience` above for the
    // expired case) passes.
    #[test]
    fn is_approved_returns_false_for_empty_allowlist() {
        assert!(
            !is_approved(&[], "m12.report"),
            "is_approved must return false when the allowlist is empty"
        );
    }

    // ====================================================================
    // W4 FINAL mutation-kill pass (S1003529) — re-verification found
    // `load_approvals` (64:5) + the NotFound match guard (68:19) still
    // surviving. The pre-existing `load_approvals_from_temp_file_succeeds`
    // and `load_approvals_propagates_io_error_other_than_not_found` tests
    // DO kill them when run against the manually-applied mutations
    // (verified). They survive in the cargo-mutants report because the
    // `propagates_io_error` test reads `/`, whose error kind is
    // environment-dependent (some sandboxes return Ok or NotFound for a
    // directory read). The replacements below are deterministic: they
    // trigger a guaranteed non-NotFound `NotADirectory` error by reading
    // a path whose parent component is a regular file.
    // ====================================================================

    // rationale: KILLS `allowlist.rs:64:5` replace `load_approvals -> ...`
    // with `Ok(vec![])`. A real, populated TSV file with exactly two
    // approval rows must load as TWO `HeldApproval`s with the exact
    // field values. The `Ok(vec![])` mutant returns an empty Vec and
    // fails the length assertion; a `Ok(vec![Default::default()])`
    // mutant (were `HeldApproval: Default`, which it is NOT — no
    // `Default` derive, `OffsetDateTime` has none) would fail the field
    // assertions. This pins the function actually parses the file.
    #[test]
    fn mutkill_64_final_load_approvals_returns_real_parsed_rows_not_empty() {
        let f = write_tsv(
            "artefact_key\tapproved_by\tapproved_at\texpiry\n\
             m12.report\tluke@0A\t2026-05-17T10:00:00Z\t2026-06-17T10:00:00Z\n\
             m13.report\tzen@audit\t2026-05-18T09:30:00Z\t2026-07-01T00:00:00Z\n",
        );
        let rows = load_approvals(f.path()).expect("populated file must load");
        // `Ok(vec![])` mutant -> len 0 -> fails here.
        assert_eq!(rows.len(), 2, "two data rows must parse into two approvals");
        // Exact field values -> a constant-vec mutant cannot satisfy these.
        assert_eq!(rows[0].artefact_key, "m12.report");
        assert_eq!(rows[0].approved_by, "luke@0A");
        assert_eq!(rows[1].artefact_key, "m13.report");
        assert_eq!(rows[1].approved_by, "zen@audit");
        assert_eq!(rows[1].expiry, datetime!(2026-07-01 00:00:00 UTC));
    }

    // rationale: KILLS `allowlist.rs:68:19` replace the match guard
    // `e.kind() == io::ErrorKind::NotFound` with `true`. With the guard
    // forced to `true`, EVERY I/O error — not just file-absent — is
    // swallowed into `Ok(Vec::new())`, so a genuine read failure is
    // silently masked as an empty allowlist (the strictest-but-WRONG
    // configuration: a corrupt/unreadable allowlist would let every
    // Held verdict CI-FAIL with no error surfaced).
    //
    // We trigger a DETERMINISTIC non-NotFound error: read a path whose
    // parent component is a regular file. `<regular_file>/child.tsv`
    // yields `io::ErrorKind::NotADirectory` (ENOTDIR) on Unix — which is
    // NOT `NotFound`. Real code: the guard is false -> propagates
    // `Err(EmberGateError::AllowlistRead)`. The `true` mutant: guard
    // true -> returns `Ok(vec![])`.
    #[test]
    fn mutkill_68_final_non_notfound_io_error_propagates_not_swallowed() {
        // A real regular file...
        let f = write_tsv("artefact_key\tapproved_by\tapproved_at\texpiry\n");
        // ...used as if it were a directory: ENOTDIR, not NotFound.
        let not_a_dir = f.path().join("child_allowlist.tsv");
        let result = load_approvals(&not_a_dir);
        // Real code: non-NotFound error propagates as a typed error.
        // `true`-guard mutant: swallowed into Ok(empty).
        let err = result.expect_err(
            "a non-NotFound I/O error (ENOTDIR) must propagate as Err, \
             not be swallowed into Ok(empty) by a `true` match guard",
        );
        assert!(
            matches!(err, EmberGateError::AllowlistRead { .. }),
            "non-NotFound I/O failure must surface as AllowlistRead, got {err:?}"
        );
    }

    // rationale: also pins `allowlist.rs:68:19` from the other side —
    // a genuinely ABSENT file (NotFound) MUST still be treated as an
    // empty allowlist (Ok), so the guard is not merely "always false".
    // Together with the test above this pins the guard to EXACTLY
    // `kind == NotFound`. (A `false`-guard mutant would fail here by
    // turning a missing file into an error; the `true`-guard mutant is
    // killed by the test above.)
    #[test]
    fn mutkill_68_final_absent_file_still_yields_empty_ok() {
        let absent = std::env::temp_dir().join("wf_m10_definitely_absent_a7f3e9c1.tsv");
        // Make sure it truly does not exist.
        let _ = std::fs::remove_file(&absent);
        let rows = load_approvals(&absent)
            .expect("an absent allowlist file must load as an empty Ok, not Err");
        assert!(rows.is_empty(), "absent file -> empty allowlist");
    }
}
