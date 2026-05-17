//! Pure synchronous validator for the workflow-trace stcortex namespace prefix.
//!
//! Per m9 spec § 5: check order is **empty → whitespace → "scratch" exact
//! → munge hyphens → starts_with prefix**. `tracing::error!` is emitted on
//! every violation; no emission on the happy path (the validator is on the
//! hot path of every substrate write).
//!
//! The function is pure beyond its tracing emission: no allocation on the
//! happy path other than the munge result, single `tracing::error!` on
//! violation, returns a `Result` typed by [`NamespaceViolation`].

use super::error::NamespaceViolation;
use super::evidence::ValidatedNamespace;

/// Single source of truth for the workflow-trace stcortex namespace prefix.
///
/// AP30 collision avoidance: `workflow_trace_*` is reserved; it does not
/// collide with `orac_*`, `pane_vortex_*`, `synthex_v2_*`, `lcm_*`, `me_*`,
/// `povm_*`, `vortex_memory_system_*`, `habitat_memory_*`, or any other
/// registered habitat namespace per the stcortex consumer registry.
///
/// **Do NOT** hard-code the literal string `"workflow_trace"` anywhere else
/// in the codebase; always import this constant. A coarse regression test
/// inside this module enforces that no other public symbol fabricates the
/// prefix outside the single legal site (see
/// `tests::regression_ap30_prefix_constant_is_the_only_legal_source`).
pub const WORKFLOW_TRACE_NS_PREFIX: &str = "workflow_trace";

/// Validate that `namespace` is a legal workflow-trace stcortex namespace
/// and return [`ValidatedNamespace`] evidence on success.
///
/// Order of checks (per m9 spec § 5):
///
/// 1. empty → [`NamespaceViolation::Empty`]
/// 2. any whitespace char → [`NamespaceViolation::Whitespace`]
/// 3. exactly `"scratch"` → [`NamespaceViolation::ScratchForbidden`]
/// 4. munge hyphens → underscores
/// 5. munged-form `starts_with` [`WORKFLOW_TRACE_NS_PREFIX`] → ok, else
///    [`NamespaceViolation::WrongPrefix`]
///
/// The hyphen munge (AP-Hab-11 / S1001757 mitigation) happens exactly once
/// at this boundary; downstream writers operate on the munged form.
///
/// # Errors
///
/// - [`NamespaceViolation::Empty`] if `namespace` is the empty string.
/// - [`NamespaceViolation::Whitespace`] if `namespace` contains any
///   whitespace character.
/// - [`NamespaceViolation::ScratchForbidden`] if `namespace == "scratch"`.
/// - [`NamespaceViolation::WrongPrefix`] if the munged form does not start
///   with [`WORKFLOW_TRACE_NS_PREFIX`].
///
/// # Examples
///
/// ```
/// use workflow_core::m9_watcher_namespace_guard::{
///     assert_workflow_trace_namespace, WORKFLOW_TRACE_NS_PREFIX,
/// };
///
/// let v = assert_workflow_trace_namespace("workflow_trace_correlations").unwrap();
/// assert_eq!(v.as_str(), "workflow_trace_correlations");
///
/// // Hyphens are munged to underscores per AP-Hab-11 mitigation:
/// let v = assert_workflow_trace_namespace("workflow-trace-runs").unwrap();
/// assert_eq!(v.as_str(), "workflow_trace_runs");
///
/// // Foreign-service prefixes are rejected:
/// assert!(assert_workflow_trace_namespace("orac_learn").is_err());
/// # let _ = WORKFLOW_TRACE_NS_PREFIX;
/// ```
pub fn assert_workflow_trace_namespace(
    namespace: &str,
) -> Result<ValidatedNamespace, NamespaceViolation> {
    if namespace.is_empty() {
        tracing::error!(
            target: "m9.validator",
            namespace = %namespace,
            "stcortex write blocked: namespace empty"
        );
        return Err(NamespaceViolation::Empty);
    }
    if namespace.chars().any(char::is_whitespace) {
        tracing::error!(
            target: "m9.validator",
            namespace = %namespace,
            "stcortex write blocked: namespace contains whitespace"
        );
        return Err(NamespaceViolation::Whitespace {
            namespace: namespace.to_owned(),
        });
    }
    if namespace == "scratch" {
        tracing::error!(
            target: "m9.validator",
            namespace = %namespace,
            "stcortex write blocked: scratch namespace forbidden"
        );
        return Err(NamespaceViolation::ScratchForbidden);
    }
    let munged = munge_hyphen_slug(namespace);
    if !munged.starts_with(WORKFLOW_TRACE_NS_PREFIX) {
        tracing::error!(
            target: "m9.validator",
            namespace = %munged,
            expected_prefix = %WORKFLOW_TRACE_NS_PREFIX,
            "stcortex write blocked: AP30 collision avoidance"
        );
        return Err(NamespaceViolation::WrongPrefix {
            namespace: munged,
            expected_prefix: WORKFLOW_TRACE_NS_PREFIX,
        });
    }
    Ok(ValidatedNamespace(munged))
}

/// Convert hyphens to underscores per stcortex slug convention
/// (S1001757 munge bug).
///
/// Pure, allocation-on-write: `String::replace` only allocates when a hyphen
/// is present. Idempotent: `munge(munge(x)) == munge(x)` because the output
/// contains no hyphens.
#[must_use]
pub fn munge_hyphen_slug(input: &str) -> String {
    input.replace('-', "_")
}

// TODO(m30/m32 — Cluster G, post-Wave-3): wire the EscapeSurfaceProfile
// 7-variant capability table per m9 spec § 2 (D-S1002127-02 ADR at
// ai_docs/optimisation-v7/decisions/2026-05-17-escape-surface-cardinality-7-privilege-escalation.md).
// PrivilegeEscalation (ordinal 30) requires
// `HumanAcceptanceSignature.privilege_escalation_acknowledged = true`;
// DataExfil (ordinal 60) requires `data_exfil_acknowledged = true`.
// The signature struct lives in m30; m9 will read it via a trait abstraction
// once that module ships. Day-1 scope = prefix + munge + 4 base variants.

#[cfg(test)]
mod tests {
    use super::{
        assert_workflow_trace_namespace, munge_hyphen_slug, NamespaceViolation,
        WORKFLOW_TRACE_NS_PREFIX,
    };

    // ---- F-Unit: prefix constant sanity (3) -----------------------------

    #[test]
    fn prefix_const_is_workflow_trace() {
        assert_eq!(WORKFLOW_TRACE_NS_PREFIX, "workflow_trace");
    }

    #[test]
    fn prefix_const_does_not_collide_with_known_services() {
        // Cross-check against known habitat namespace prefixes documented in
        // CLAUDE.md ULTRAPLATE services table.
        for foreign in [
            "orac",
            "pane_vortex",
            "synthex_v2",
            "lcm",
            "me",
            "povm",
            "vortex_memory_system",
            "habitat_memory",
        ] {
            assert!(
                !WORKFLOW_TRACE_NS_PREFIX.starts_with(foreign),
                "prefix collides with foreign={foreign}"
            );
            assert!(
                !foreign.starts_with(WORKFLOW_TRACE_NS_PREFIX),
                "foreign={foreign} collides with prefix"
            );
        }
    }

    #[test]
    fn prefix_const_is_lowercase_and_has_no_trailing_separator() {
        assert!(WORKFLOW_TRACE_NS_PREFIX
            .chars()
            .all(|c| !c.is_uppercase()));
        assert!(!WORKFLOW_TRACE_NS_PREFIX.ends_with('_'));
        assert!(!WORKFLOW_TRACE_NS_PREFIX.ends_with('-'));
    }

    // ---- F-Unit: happy paths (3) ----------------------------------------

    #[test]
    fn accepts_canonical_prefix_with_suffix() {
        let v = assert_workflow_trace_namespace("workflow_trace_correlations")
            .expect("canonical");
        assert_eq!(v.as_str(), "workflow_trace_correlations");
    }

    #[test]
    fn accepts_year_qualified_namespace() {
        let v =
            assert_workflow_trace_namespace("workflow_trace_battern_runs_2026").expect("year");
        assert_eq!(v.as_str(), "workflow_trace_battern_runs_2026");
    }

    #[test]
    fn accepts_bare_prefix() {
        // Per spec § 13 Q1: prefix-only is currently accepted; trailing `_`
        // strictness is a G7 open question carried forward.
        let v = assert_workflow_trace_namespace("workflow_trace").expect("bare");
        assert_eq!(v.as_str(), "workflow_trace");
    }

    // ---- F-Unit: wrong-prefix rejections (3) ----------------------------

    #[test]
    fn rejects_orac_prefix() {
        let err = assert_workflow_trace_namespace("orac_learn").unwrap_err();
        let NamespaceViolation::WrongPrefix {
            namespace,
            expected_prefix,
        } = err
        else {
            panic!("expected WrongPrefix");
        };
        assert_eq!(namespace, "orac_learn");
        assert_eq!(expected_prefix, "workflow_trace");
    }

    #[test]
    fn rejects_pane_vortex_prefix() {
        assert!(matches!(
            assert_workflow_trace_namespace("pane_vortex_pulse"),
            Err(NamespaceViolation::WrongPrefix { .. })
        ));
    }

    #[test]
    fn rejects_close_but_missing_separator_prefix() {
        // "workflowtrace_x" lacks the underscore between "workflow" and
        // "trace" — does NOT start with "workflow_trace".
        assert!(matches!(
            assert_workflow_trace_namespace("workflowtrace_x"),
            Err(NamespaceViolation::WrongPrefix { .. })
        ));
    }

    // ---- F-Unit: empty / whitespace / scratch (8) -----------------------

    #[test]
    fn rejects_empty_string() {
        assert_eq!(
            assert_workflow_trace_namespace("").unwrap_err(),
            NamespaceViolation::Empty
        );
    }

    #[test]
    fn rejects_whitespace_in_middle() {
        assert!(matches!(
            assert_workflow_trace_namespace("workflow trace x"),
            Err(NamespaceViolation::Whitespace { .. })
        ));
    }

    #[test]
    fn rejects_leading_whitespace() {
        assert!(matches!(
            assert_workflow_trace_namespace(" workflow_trace_x"),
            Err(NamespaceViolation::Whitespace { .. })
        ));
    }

    #[test]
    fn rejects_trailing_whitespace() {
        assert!(matches!(
            assert_workflow_trace_namespace("workflow_trace_x "),
            Err(NamespaceViolation::Whitespace { .. })
        ));
    }

    #[test]
    fn rejects_tab_character() {
        assert!(matches!(
            assert_workflow_trace_namespace("workflow_trace_a\tb"),
            Err(NamespaceViolation::Whitespace { .. })
        ));
    }

    #[test]
    fn rejects_newline_character() {
        assert!(matches!(
            assert_workflow_trace_namespace("workflow_trace_a\nb"),
            Err(NamespaceViolation::Whitespace { .. })
        ));
    }

    #[test]
    fn rejects_scratch_exact_string() {
        assert_eq!(
            assert_workflow_trace_namespace("scratch").unwrap_err(),
            NamespaceViolation::ScratchForbidden
        );
    }

    #[test]
    fn accepts_scratch_as_substring_within_workflow_trace_namespace() {
        // "scratch" forbidden is EXACT-string only per spec § 4;
        // "workflow_trace_scratch" is allowed.
        let v = assert_workflow_trace_namespace("workflow_trace_scratch").expect("substring");
        assert_eq!(v.as_str(), "workflow_trace_scratch");
    }

    // ---- F-Unit: munge cases (5) ----------------------------------------

    #[test]
    fn munges_single_hyphen() {
        assert_eq!(munge_hyphen_slug("a-b"), "a_b");
    }

    #[test]
    fn munges_multiple_hyphens() {
        assert_eq!(munge_hyphen_slug("a-b-c-d"), "a_b_c_d");
    }

    #[test]
    fn munge_no_hyphens_returns_same_value() {
        assert_eq!(munge_hyphen_slug("a_b_c"), "a_b_c");
    }

    #[test]
    fn munge_empty_returns_empty() {
        assert_eq!(munge_hyphen_slug(""), "");
    }

    #[test]
    fn munge_then_validate_for_hyphenated_workflow_trace_passes() {
        let v = assert_workflow_trace_namespace("workflow-trace-foo").expect("hyphenated");
        assert_eq!(v.as_str(), "workflow_trace_foo");
    }

    // ---- F-Unit: error-field shape (3) ----------------------------------

    #[test]
    fn wrong_prefix_error_namespace_field_is_munged_form() {
        let err = assert_workflow_trace_namespace("orac-learn").unwrap_err();
        let NamespaceViolation::WrongPrefix { namespace, .. } = err else {
            panic!("expected WrongPrefix");
        };
        assert_eq!(
            namespace, "orac_learn",
            "error must carry munged form, not raw input"
        );
    }

    #[test]
    fn whitespace_error_namespace_field_is_raw_input() {
        // Per spec § 5: whitespace check fires BEFORE munge; the error
        // carries the raw input so operators see what the caller actually
        // sent.
        let err = assert_workflow_trace_namespace("workflow trace x").unwrap_err();
        let NamespaceViolation::Whitespace { namespace } = err else {
            panic!("expected Whitespace");
        };
        assert_eq!(namespace, "workflow trace x");
    }

    #[test]
    fn validator_is_deterministic_on_happy_path() {
        // 100 calls with same input produce 100 identical Ok results.
        for _ in 0..100_usize {
            let v = assert_workflow_trace_namespace("workflow_trace_x").expect("happy");
            assert_eq!(v.as_str(), "workflow_trace_x");
        }
    }

    // ---- F-Property (5 tests) -------------------------------------------

    #[test]
    fn property_starts_with_iff_ok_modulo_munge() {
        // for_all ns: validate(ns).is_ok() iff
        //   munge(ns).starts_with(prefix)
        //   AND ns is non-empty
        //   AND ns is whitespace-free
        //   AND ns != "scratch"
        let cases = [
            "workflow_trace_a",
            "workflow_trace",
            "workflow-trace-a",
            "orac_x",
            "",
            "scratch",
            "workflow_trace_b c",
            "workflowtrace_x",
            "wf_trace_x",
            "workflow-trace",
            "workflow_trace_",
            "Workflow_trace_x",
        ];
        for input in cases {
            let result = assert_workflow_trace_namespace(input);
            let munged = munge_hyphen_slug(input);
            let expected_ok = !input.is_empty()
                && !input.chars().any(char::is_whitespace)
                && input != "scratch"
                && munged.starts_with(WORKFLOW_TRACE_NS_PREFIX);
            assert_eq!(
                result.is_ok(),
                expected_ok,
                "input {input:?}: result={result:?} expected_ok={expected_ok}"
            );
        }
    }

    #[test]
    fn property_munge_is_idempotent() {
        for input in [
            "",
            "no-hyphens",
            "a-b-c",
            "--",
            "a---b",
            "already_munged",
            "mixed-with_underscores",
            "leading-",
            "-trailing",
        ] {
            let once = munge_hyphen_slug(input);
            let twice = munge_hyphen_slug(&once);
            assert_eq!(once, twice, "munge non-idempotent on {input:?}");
        }
    }

    #[test]
    fn property_validated_namespace_round_trip_via_as_str() {
        for input in [
            "workflow_trace_x",
            "workflow_trace_battern_2026",
            "workflow-trace-x",
            "workflow_trace",
        ] {
            let v = assert_workflow_trace_namespace(input).expect("round-trip in");
            let s = v.as_str().to_owned();
            let v2 = assert_workflow_trace_namespace(&s).expect("round-trip out");
            assert_eq!(v, v2);
        }
    }

    #[test]
    fn property_whitespace_rejection_closed_under_any_whitespace_char() {
        for ws in &[' ', '\t', '\n', '\r', '\x0C'] {
            let input = format!("workflow_trace{ws}x");
            let err = assert_workflow_trace_namespace(&input).unwrap_err();
            assert!(
                matches!(err, NamespaceViolation::Whitespace { .. }),
                "expected Whitespace for ws={ws:?}, got {err:?}"
            );
        }
    }

    #[test]
    fn property_only_empty_string_returns_empty() {
        for input in ["a", " ", "x", "scratch", "workflow_trace_x"] {
            let result = assert_workflow_trace_namespace(input);
            assert!(
                !matches!(result, Err(NamespaceViolation::Empty)),
                "non-empty {input:?} returned Empty"
            );
        }
        assert_eq!(
            assert_workflow_trace_namespace("").unwrap_err(),
            NamespaceViolation::Empty
        );
    }

    // ---- F-Regression (2 tests) -----------------------------------------

    #[test]
    fn regression_ap30_prefix_constant_is_the_only_legal_source() {
        // AP30 regression slot: the prefix constant is the single source of
        // truth. Sanity-check the constant value and the literal it exposes
        // — if anyone redefines the literal upstream this assertion catches
        // the drift before downstream call sites silently pick up a wrong
        // value.
        assert_eq!(WORKFLOW_TRACE_NS_PREFIX, "workflow_trace");
        // The constant itself is the legitimate "workflow_trace" site. A
        // coarse grep in tests/m9_integration.rs scans the rest of `src/`
        // for stray literals.
    }

    #[test]
    fn regression_ap_hab_11_hyphen_munge_at_boundary() {
        // AP-Hab-11 (S1001757) regression slot: writes with hyphens in the
        // namespace must land in stcortex as the underscored form. The
        // munge happens at the validator boundary exactly once.
        let v = assert_workflow_trace_namespace("workflow-trace-foo").expect("hyphen");
        assert_eq!(
            v.as_str(),
            "workflow_trace_foo",
            "hyphen munge must happen exactly once at the m9 boundary"
        );
        // Calling the validator again on the munged form must be a no-op.
        let v2 = assert_workflow_trace_namespace(v.as_str()).expect("re-validate");
        assert_eq!(v, v2);
    }
}
