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
/// Order of checks (per m9 spec § 5, extended for non-printable rejection):
///
/// 1. empty → [`NamespaceViolation::Empty`]
/// 2. any whitespace char → [`NamespaceViolation::Whitespace`]
/// 3. any control char or BOM → [`NamespaceViolation::ControlChar`]
/// 4. exactly `"scratch"` → [`NamespaceViolation::ScratchForbidden`]
/// 5. munge hyphens → underscores
/// 6. munged form is *exactly* [`WORKFLOW_TRACE_NS_PREFIX`] OR begins with
///    `"{WORKFLOW_TRACE_NS_PREFIX}_"` (prefix + `_` separator) → ok, else
///    [`NamespaceViolation::WrongPrefix`]. A bare `starts_with` would leak
///    the boundary by accepting `workflow_traceXYZ`; the `_`-separator
///    requirement enforces the AP30 contract `workflow_trace` |
///    `workflow_trace_<suffix>`.
///
/// The hyphen munge (AP-Hab-11 / S1001757 mitigation) happens exactly once
/// at this boundary; downstream writers operate on the munged form.
///
/// # Errors
///
/// - [`NamespaceViolation::Empty`] if `namespace` is the empty string.
/// - [`NamespaceViolation::Whitespace`] if `namespace` contains any
///   whitespace character.
/// - [`NamespaceViolation::ControlChar`] if `namespace` contains a control
///   character (`is_control() && !is_whitespace()`) or a BOM (U+FEFF).
/// - [`NamespaceViolation::ScratchForbidden`] if `namespace == "scratch"`.
/// - [`NamespaceViolation::WrongPrefix`] if the munged form is neither
///   exactly [`WORKFLOW_TRACE_NS_PREFIX`] nor begins with the prefix
///   followed by an `_` separator (i.e. `workflow_traceXYZ` is rejected).
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
    // Reject any control character or U+FEFF (BOM) that slipped past the
    // whitespace check. `char::is_whitespace` does NOT cover NUL (`\0`),
    // non-whitespace ASCII control bytes (`\x01`-`\x1F` except whitespace),
    // DEL (`\x7F`), or the Unicode BOM. Without this gate, a stcortex slug
    // can carry an invisible control byte all the way to substrate
    // logging / SQL — silent contamination.
    // (Fix: silent-failure-hunter LIKELY finding — NUL/BOM bypass.)
    if let Some((byte_offset, c)) = namespace
        .char_indices()
        .find(|&(_, c)| c == '\u{FEFF}' || (c.is_control() && !c.is_whitespace()))
    {
        tracing::error!(
            target: "m9.validator",
            namespace = %namespace.escape_debug().to_string(),
            codepoint = c as u32,
            byte_offset,
            "stcortex write blocked: namespace contains control character"
        );
        return Err(NamespaceViolation::ControlChar {
            codepoint: c as u32,
            byte_offset,
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
    // AP30 boundary enforcement: a legal namespace is *exactly* the prefix
    // OR the prefix followed by an underscore separator. A bare
    // `starts_with(prefix)` accepts `workflow_traceXYZ` (the 14-char prefix
    // immediately followed by non-separator content) — a boundary leak into
    // adjacent/foreign-shaped namespaces (e.g. `workflow_tracexploit`,
    // `workflow_trace2`). The contract is `workflow_trace` |
    // `workflow_trace_<suffix>`; we therefore require the next character
    // after the prefix (if any) to be the `_` separator.
    // (Fix: W2/F3 HIGH security-boundary finding.)
    let prefix_boundary_ok = munged == WORKFLOW_TRACE_NS_PREFIX
        || munged.starts_with(&format!("{WORKFLOW_TRACE_NS_PREFIX}_"));
    if !prefix_boundary_ok {
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

// Phase 6e — m9 ↔ m32 EscapeSurfaceProfile seam (gap C-8 / NA-GAP-11 fold).
// Closes the TODO that previously occupied this site. The m9 spec § 2
// 7-variant capability table (D-S1002127-02 ADR at
// ai_docs/optimisation-v7/decisions/2026-05-17-escape-surface-cardinality-7-privilege-escalation.md)
// is wired below via the single shared
// [`crate::m32_dispatcher::AcceptanceSignatureReader`] trait. m9 reads only
// `acknowledged_ceiling()` — the minimum slice of the signature surface the
// monotone gate needs — so we share m32's contract without coupling to the
// rest of `HumanAcceptanceSignature`'s layout.
//
// Monotonicity (mirrors m32's
// [`EscapeSurfaceProfile::is_acknowledged_by`]): a write at profile X is
// permitted iff `X.ordinal() <= reader.acknowledged_ceiling().ordinal()`.
// PrivilegeEscalation (ord 30) demands a ceiling ≥ PrivilegeEscalation;
// DataExfil (ord 60) one ≥ DataExfil. All 7 variants are covered by the
// same single comparison — there is no per-variant special case here.
//
// The function is a pure validator: a single ordinal comparison + (on
// refusal) a `tracing::error!` event in the same shape as the rest of the
// validator surface. No allocation on the happy path.

use crate::m32_dispatcher::{AcceptanceSignatureReader, EscapeSurfaceProfile};

/// Capability table per m9 spec § 2 — the minimum
/// [`EscapeSurfaceProfile`] ceiling an acceptance signature must
/// acknowledge for a write at the given `required` profile to be admitted
/// at the m9 application layer.
///
/// The mapping is the **identity** function: the m32 monotone destructiveness
/// ladder is single-axis, so the minimum ceiling for a write at profile X is
/// exactly X. Implementations of the table that introduce slack (e.g.
/// "`SandboxEscape` requires only `Sandboxed`") would silently widen the
/// gate — by exposing the table as a `const fn` we make the lift visible to
/// every caller and locked at the type system.
///
/// Used by [`assert_namespace_capability`]; exposed publicly so callers can
/// preflight the gate without raising a refusal event.
#[must_use]
#[allow(
    clippy::needless_match,
    reason = "Explicit per-variant arms force any future EscapeSurfaceProfile \
              cardinality bump (8th variant) to visit this site and audit the \
              capability table; collapsing to `required` would silently \
              inherit identity for unseen variants, defeating the m9 ↔ m32 \
              seam discipline (Phase 6e, gap C-8)."
)]
pub const fn required_signature_ceiling(required: EscapeSurfaceProfile) -> EscapeSurfaceProfile {
    // Identity (single-axis monotone ladder). All 7 variants enumerated
    // explicitly so a future cardinality bump (8th variant) is forced to
    // visit this site rather than silently inheriting a wildcard default.
    match required {
        EscapeSurfaceProfile::Sandboxed => EscapeSurfaceProfile::Sandboxed,
        EscapeSurfaceProfile::SandboxEscape => EscapeSurfaceProfile::SandboxEscape,
        EscapeSurfaceProfile::ProcessMutate => EscapeSurfaceProfile::ProcessMutate,
        EscapeSurfaceProfile::PrivilegeEscalation => EscapeSurfaceProfile::PrivilegeEscalation,
        EscapeSurfaceProfile::FileWrite => EscapeSurfaceProfile::FileWrite,
        EscapeSurfaceProfile::NetworkEgress => EscapeSurfaceProfile::NetworkEgress,
        EscapeSurfaceProfile::DataExfil => EscapeSurfaceProfile::DataExfil,
    }
}

/// Assert that the operator's acknowledged ceiling, as exposed by any
/// [`AcceptanceSignatureReader`], covers a write at the given `required`
/// [`EscapeSurfaceProfile`].
///
/// This is the m9 application-layer mirror of m32's monotone
/// dispatch-time gate ([`EscapeSurfaceProfile::is_acknowledged_by`] /
/// [`EscapeSurfaceProfile::is_acknowledged_by_reader`]). m9 surfaces the
/// gate **before the substrate write** so a write at `PrivilegeEscalation`
/// shape (ord 30) or `DataExfil` shape (ord 60) cannot leak past the
/// namespace guard while the operator has only acknowledged a lower
/// ceiling.
///
/// The function is pure beyond a single `tracing::error!` on refusal: no
/// allocation on the happy path, single comparison, returns a typed
/// [`NamespaceViolation::CapabilityNotAcknowledged`] on violation.
///
/// Phase 6e — m9 ↔ m32 EscapeSurfaceProfile seam (gap C-8 / NA-GAP-11
/// fold). The shared trait is defined ONCE in
/// [`crate::m32_dispatcher`]; m9 imports it (read-only) and consumes only
/// `acknowledged_ceiling()`.
///
/// # Errors
///
/// Returns [`NamespaceViolation::CapabilityNotAcknowledged`] when the
/// signature reader's `acknowledged_ceiling()` ordinal is strictly less
/// than the `required` profile's ordinal.
pub fn assert_namespace_capability<R>(
    required: EscapeSurfaceProfile,
    signature: &R,
) -> Result<(), NamespaceViolation>
where
    R: AcceptanceSignatureReader + ?Sized,
{
    let acknowledged = signature.acknowledged_ceiling();
    // Use the shared trait-generic gate on EscapeSurfaceProfile to keep the
    // m9 ↔ m32 check the SAME comparison — one ordinal, one direction. Any
    // future divergence between m32's dispatch gate and m9's namespace gate
    // would force a change here.
    if required.is_acknowledged_by_reader(signature) {
        return Ok(());
    }
    tracing::error!(
        target: "m9.validator",
        required = ?required,
        required_ord = required.ordinal(),
        acknowledged = ?acknowledged,
        acknowledged_ord = acknowledged.ordinal(),
        "stcortex write blocked: EscapeSurfaceProfile capability gate"
    );
    Err(NamespaceViolation::CapabilityNotAcknowledged {
        required,
        required_ord: required.ordinal(),
        acknowledged,
        acknowledged_ord: acknowledged.ordinal(),
    })
}

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
    fn property_prefix_boundary_iff_ok_modulo_munge() {
        // for_all ns: validate(ns).is_ok() iff
        //   munge(ns) == prefix OR munge(ns).starts_with("{prefix}_")
        //   AND ns is non-empty
        //   AND ns is whitespace-free
        //   AND ns != "scratch"
        //
        // Note the prefix check is the *boundary-aware* form per the W2/F3
        // fix: a bare `starts_with(prefix)` would wrongly accept
        // `workflow_traceXYZ`.
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
            "workflow_traceXYZ",
            "workflow_trace2",
        ];
        let sep_prefixed = format!("{WORKFLOW_TRACE_NS_PREFIX}_");
        for input in cases {
            let result = assert_workflow_trace_namespace(input);
            let munged = munge_hyphen_slug(input);
            let prefix_boundary_ok =
                munged == WORKFLOW_TRACE_NS_PREFIX || munged.starts_with(&sep_prefixed);
            let expected_ok = !input.is_empty()
                && !input.chars().any(char::is_whitespace)
                && input != "scratch"
                && prefix_boundary_ok;
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

    // ====================================================================
    // Hardening pass (S1002388) — +10 tests for the m9 namespace guard.
    // Closes the NUL/BOM/control-char silent bypass + raises adversarial
    // coverage on the validator boundary.
    // ====================================================================

    // rationale: Anti-property — NUL byte slipped past `is_whitespace`
    // pre-fix; the new ControlChar check rejects it loudly.
    #[test]
    fn rejects_embedded_nul_byte() {
        let err = assert_workflow_trace_namespace("workflow_trace\0x").unwrap_err();
        let NamespaceViolation::ControlChar { codepoint, byte_offset } = err else {
            panic!("expected ControlChar, got {err:?}");
        };
        assert_eq!(codepoint, 0, "expected U+0000 (NUL)");
        assert_eq!(byte_offset, 14, "NUL is at byte offset 14 in input");
    }

    // rationale: Anti-property — Unicode BOM (U+FEFF) at the start of a
    // namespace is invisible in most terminals and would silently
    // contaminate the slug downstream. m9 rejects it now.
    #[test]
    fn rejects_leading_bom() {
        let err = assert_workflow_trace_namespace("\u{FEFF}workflow_trace_x").unwrap_err();
        let NamespaceViolation::ControlChar { codepoint, byte_offset } = err else {
            panic!("expected ControlChar, got {err:?}");
        };
        assert_eq!(codepoint, 0xFEFF, "expected U+FEFF (BOM)");
        assert_eq!(byte_offset, 0);
    }

    // rationale: Boundary — DEL character (U+007F) is control but not
    // whitespace; must be rejected by ControlChar.
    #[test]
    fn rejects_embedded_del_character() {
        let err = assert_workflow_trace_namespace("workflow_trace_\x7Fx").unwrap_err();
        assert!(matches!(err, NamespaceViolation::ControlChar { codepoint: 0x7F, .. }));
    }

    // rationale: Adversarial input — Bell (U+0007) and other low-ASCII
    // controls must all surface ControlChar with the right codepoint.
    #[test]
    fn rejects_all_low_ascii_control_chars_individually() {
        for cp in 0_u32..0x20 {
            // Skip whitespace control chars — they're caught by the
            // earlier Whitespace check.
            let c = char::from_u32(cp).expect("ascii cp");
            if c.is_whitespace() {
                continue;
            }
            let input = format!("workflow_trace_{c}x");
            let err = assert_workflow_trace_namespace(&input).unwrap_err();
            assert!(
                matches!(err, NamespaceViolation::ControlChar { codepoint, .. }
                    if codepoint == cp),
                "expected ControlChar(cp={cp:#04X}), got {err:?} for input {input:?}"
            );
        }
    }

    // rationale: Anti-property — printable Unicode (e.g. CJK, accented
    // chars, emoji) does NOT trigger ControlChar; the prefix check (or
    // its absence) governs the verdict.
    #[test]
    fn printable_unicode_does_not_trigger_control_char_check() {
        // Non-prefix Unicode → WrongPrefix, not ControlChar.
        for input in ["héllo", "测试_namespace", "wf_\u{1F600}"] {
            let err = assert_workflow_trace_namespace(input).unwrap_err();
            assert!(
                !matches!(err, NamespaceViolation::ControlChar { .. }),
                "printable unicode {input:?} wrongly triggered ControlChar: {err:?}"
            );
        }
    }

    // rationale: Contract regression — ordering invariant. Empty wins over
    // whitespace wins over control-char wins over scratch wins over
    // prefix. Test the WHITESPACE / CONTROL-CHAR boundary (most likely
    // to drift on refactor).
    #[test]
    fn whitespace_wins_over_control_char_when_both_present() {
        // String contains a space AND a NUL. Whitespace check fires first.
        let err = assert_workflow_trace_namespace("workflow_trace \0x").unwrap_err();
        assert!(matches!(err, NamespaceViolation::Whitespace { .. }),
            "whitespace check must precede control-char check");
    }

    // rationale: Determinism — validator returns identical Err shapes for
    // identical bad inputs across repeated calls.
    #[test]
    fn validator_error_is_deterministic_for_control_char() {
        let first = assert_workflow_trace_namespace("workflow_trace\0x").unwrap_err();
        for _ in 0..100_u32 {
            let again = assert_workflow_trace_namespace("workflow_trace\0x").unwrap_err();
            assert_eq!(first, again);
        }
    }

    // rationale: Resource accounting — munge_hyphen_slug does NOT
    // allocate on the no-hyphen happy path (the common case for already-
    // canonicalised namespaces). Tested observationally via the fact
    // that munge of a string identical to its munged form returns a
    // String equal to the input.
    //
    // (We can't directly test no-alloc, but we can verify functional
    // equivalence + idempotence on the hot path.)
    #[test]
    fn munge_hot_path_no_hyphens_preserves_input_exactly() {
        let input = "workflow_trace_some_long_canonical_form_no_hyphens";
        let out = munge_hyphen_slug(input);
        assert_eq!(out, input);
        // Idempotence: a second pass through munge is a no-op.
        assert_eq!(munge_hyphen_slug(&out), input);
    }

    // rationale: Boundary — the workflow_trace_ prefix with a trailing
    // underscore is the canonical "well-formed" namespace shape; verify
    // the validator accepts BOTH bare prefix and prefix-with-underscore
    // (per spec § 13 Q1 open question — current behaviour: both ok).
    #[test]
    fn accepts_prefix_with_explicit_trailing_underscore() {
        let v = assert_workflow_trace_namespace("workflow_trace_").expect("trailing _");
        assert_eq!(v.as_str(), "workflow_trace_");
    }

    // ====================================================================
    // W2/F3 hardening — AP30 prefix-boundary leak. A bare
    // `starts_with(prefix)` accepts `workflow_traceXYZ` (prefix immediately
    // followed by non-separator content). The contract is exactly
    // `workflow_trace` OR `workflow_trace_<suffix>`.
    // ====================================================================

    // rationale: Security boundary — the F3 leak. `workflow_traceXYZ` is the
    // 14-char prefix followed by non-separator content; it MUST be rejected
    // as WrongPrefix, not silently admitted into an adjacent namespace.
    #[test]
    fn rejects_prefix_with_non_separator_suffix_f3_leak() {
        let err = assert_workflow_trace_namespace("workflow_traceXYZ").unwrap_err();
        let NamespaceViolation::WrongPrefix {
            namespace,
            expected_prefix,
        } = err
        else {
            panic!("expected WrongPrefix, got {err:?}");
        };
        assert_eq!(namespace, "workflow_traceXYZ");
        assert_eq!(expected_prefix, "workflow_trace");
    }

    // rationale: Happy path — the bare prefix `workflow_trace` (exact match)
    // is a legal namespace and must be accepted.
    #[test]
    fn accepts_bare_prefix_exact_match_f3() {
        let v = assert_workflow_trace_namespace("workflow_trace").expect("bare prefix");
        assert_eq!(v.as_str(), "workflow_trace");
    }

    // rationale: Happy path — `workflow_trace_foo` (prefix + `_` separator +
    // suffix) is the canonical well-formed namespace and must be accepted.
    #[test]
    fn accepts_prefix_with_separator_and_suffix_f3() {
        let v = assert_workflow_trace_namespace("workflow_trace_foo").expect("prefix_suffix");
        assert_eq!(v.as_str(), "workflow_trace_foo");
    }

    // rationale: Adversarial — a digit immediately after the prefix
    // (`workflow_trace2`) is the same boundary leak class as the XYZ case
    // and must be rejected.
    #[test]
    fn rejects_prefix_with_digit_suffix_f3_leak() {
        assert!(matches!(
            assert_workflow_trace_namespace("workflow_trace2"),
            Err(NamespaceViolation::WrongPrefix { .. })
        ));
    }

    // rationale: Adversarial — a hyphenated near-miss munges to a
    // non-separator suffix (`workflow-tracex` → `workflow_tracex`) and must
    // still be rejected after the munge.
    #[test]
    fn rejects_hyphenated_prefix_boundary_leak_after_munge_f3() {
        let err = assert_workflow_trace_namespace("workflow-tracex").unwrap_err();
        let NamespaceViolation::WrongPrefix { namespace, .. } = err else {
            panic!("expected WrongPrefix, got {err:?}");
        };
        // Error carries the munged form per the existing contract.
        assert_eq!(namespace, "workflow_tracex");
    }

    // rationale: Cross-module surface invariant — a control-char rejection
    // must include the byte offset to help operators locate the bad
    // codepoint in their input. The offset MUST be the START of the
    // multi-byte sequence (UTF-8 boundary), not a midpoint.
    #[test]
    fn control_char_byte_offset_is_utf8_boundary() {
        // Put BOM after a 3-byte CJK char (测=3 bytes UTF-8).
        let input = "workflow_trace测\u{FEFF}";
        let err = assert_workflow_trace_namespace(input).unwrap_err();
        let NamespaceViolation::ControlChar { byte_offset, codepoint } = err else {
            panic!("expected ControlChar");
        };
        assert_eq!(codepoint, 0xFEFF);
        // The byte offset must be precisely where the U+FEFF starts in
        // the UTF-8-encoded input — NOT at a midpoint inside the CJK char.
        assert_eq!(byte_offset, "workflow_trace测".len(),
            "byte_offset must land on a UTF-8 boundary, got {byte_offset}");
    }

    // ====================================================================
    // Phase 6e — m9 ↔ m32 EscapeSurfaceProfile seam (gap C-8 / NA-GAP-11).
    //
    // The new `assert_namespace_capability` function closes the prior
    // TODO at the bottom of `validator.rs` by wiring the m9 spec § 2
    // capability table to m32's monotone destructiveness ladder via the
    // shared `AcceptanceSignatureReader` trait. Tests below exercise:
    //   - the identity capability table (all 7 variants),
    //   - the trait-generic gate across every (required, ceiling) pair,
    //   - the boundary at-or-below-ceiling Approve / above-ceiling
    //     Refuse semantics,
    //   - the typed `CapabilityNotAcknowledged` shape,
    //   - the trait blanket impl on references,
    //   - a custom test-only `AcceptanceSignatureReader` impl confirming
    //     the trait — not the concrete struct — is the contract,
    //   - the m9 ↔ m32 cross-consistency invariant (the same comparison
    //     runs through both `is_acknowledged_by_reader` and
    //     `assert_namespace_capability`).
    // ====================================================================

    use super::{assert_namespace_capability, required_signature_ceiling};
    use crate::m32_dispatcher::{
        AcceptanceSignatureReader, EscapeSurfaceProfile, HumanAcceptanceSignature,
    };

    /// Test-only [`AcceptanceSignatureReader`] that carries ONLY the
    /// minimum data point — proves the m9 ↔ m32 seam contracts on the
    /// trait, not on the concrete [`HumanAcceptanceSignature`] layout.
    struct CeilingOnly(EscapeSurfaceProfile);

    impl AcceptanceSignatureReader for CeilingOnly {
        fn acknowledged_ceiling(&self) -> EscapeSurfaceProfile {
            self.0
        }
    }

    // rationale: Capability-table contract — m9 spec § 2 lifts to identity
    // on the single-axis monotone ladder. Any per-variant slack would
    // silently widen the gate; this test pins all 7 mappings.
    #[test]
    fn required_signature_ceiling_is_identity_on_all_seven_variants() {
        for &p in &EscapeSurfaceProfile::VARIANTS {
            assert_eq!(
                required_signature_ceiling(p),
                p,
                "capability table must be identity for {p:?} (single-axis monotone ladder)"
            );
        }
    }

    // rationale: Happy path — when ceiling ≥ required, the m9 capability
    // gate Approves. Exercises every (required, ceiling) pair in the
    // upper triangle + the diagonal (7 × 7 = 49 pairs, 28 satisfy the
    // gate).
    #[test]
    fn capability_gate_approves_when_ceiling_meets_or_exceeds_required() {
        let mut ok_count = 0;
        for &required in &EscapeSurfaceProfile::VARIANTS {
            for &ceiling in &EscapeSurfaceProfile::VARIANTS {
                let sig = HumanAcceptanceSignature {
                    interactive_terminal: true,
                    acknowledged_ceiling: ceiling,
                };
                let result = assert_namespace_capability(required, &sig);
                if required.ordinal() <= ceiling.ordinal() {
                    assert!(
                        result.is_ok(),
                        "required={required:?} ceiling={ceiling:?} must Approve, got {result:?}"
                    );
                    ok_count += 1;
                } else {
                    assert!(
                        result.is_err(),
                        "required={required:?} ceiling={ceiling:?} must Refuse, got Ok"
                    );
                }
            }
        }
        // binomial(7, 2) + 7 = 21 + 7 = 28 (lower-or-equal triangle including diagonal).
        assert_eq!(ok_count, 28, "expected 28 at-or-below-ceiling pairs to Approve");
    }

    // rationale: Anti-property — when ceiling < required, the m9
    // capability gate refuses with the typed
    // `CapabilityNotAcknowledged` variant carrying both ordinals AND
    // both profile enums. Operator triage relies on the structured
    // error, not just the tracing event.
    #[test]
    fn capability_gate_refuses_with_typed_error_above_ceiling() {
        let mut refuse_count = 0;
        for &required in &EscapeSurfaceProfile::VARIANTS {
            for &ceiling in &EscapeSurfaceProfile::VARIANTS {
                if required.ordinal() <= ceiling.ordinal() {
                    continue;
                }
                let sig = HumanAcceptanceSignature {
                    interactive_terminal: true,
                    acknowledged_ceiling: ceiling,
                };
                let err = assert_namespace_capability(required, &sig).unwrap_err();
                let NamespaceViolation::CapabilityNotAcknowledged {
                    required: r,
                    required_ord,
                    acknowledged,
                    acknowledged_ord,
                } = err
                else {
                    panic!(
                        "expected CapabilityNotAcknowledged for required={required:?} \
                         ceiling={ceiling:?}, got {err:?}"
                    );
                };
                assert_eq!(r, required);
                assert_eq!(required_ord, required.ordinal());
                assert_eq!(acknowledged, ceiling);
                assert_eq!(acknowledged_ord, ceiling.ordinal());
                refuse_count += 1;
            }
        }
        // binomial(7, 2) = 21 strict-greater-than pairs.
        assert_eq!(refuse_count, 21, "expected 21 above-ceiling pairs to Refuse");
    }

    // rationale: Boundary — equal ordinals Approve (the comparison is
    // `<=` not `<`; a write at exactly the ceiling is acknowledged).
    #[test]
    fn capability_gate_approves_on_exact_ceiling_match() {
        for &p in &EscapeSurfaceProfile::VARIANTS {
            let sig = HumanAcceptanceSignature {
                interactive_terminal: true,
                acknowledged_ceiling: p,
            };
            assert!(
                assert_namespace_capability(p, &sig).is_ok(),
                "required={p:?} ceiling={p:?} must Approve (gate is <=, not <)"
            );
        }
    }

    // rationale: m9 ↔ m32 cross-consistency. The same monotone comparison
    // must drive m32's `is_acknowledged_by`, m32's
    // `is_acknowledged_by_reader`, AND m9's `assert_namespace_capability`
    // — if any of the three drifts, this test fires.
    #[test]
    fn capability_gate_matches_m32_dispatch_gate_on_every_pair() {
        for &required in &EscapeSurfaceProfile::VARIANTS {
            for &ceiling in &EscapeSurfaceProfile::VARIANTS {
                let sig = HumanAcceptanceSignature {
                    interactive_terminal: true,
                    acknowledged_ceiling: ceiling,
                };
                let m32_concrete = required.is_acknowledged_by(&sig);
                let m32_trait = required.is_acknowledged_by_reader(&sig);
                let m9_gate = assert_namespace_capability(required, &sig).is_ok();
                assert_eq!(
                    m32_concrete, m32_trait,
                    "m32 concrete vs trait gate diverge at required={required:?} \
                     ceiling={ceiling:?}"
                );
                assert_eq!(
                    m32_concrete, m9_gate,
                    "m9 vs m32 gates diverge at required={required:?} ceiling={ceiling:?} \
                     — m9 ↔ m32 single-source-of-truth invariant breached"
                );
            }
        }
    }

    // rationale: Trait-abstraction contract. m9 must depend on the
    // `AcceptanceSignatureReader` trait, NOT the concrete
    // `HumanAcceptanceSignature`. A test-only reader that exposes only
    // the ceiling (no `interactive_terminal`, no other fields) must
    // drive the gate identically.
    #[test]
    fn capability_gate_consumes_trait_not_concrete_struct() {
        for &required in &EscapeSurfaceProfile::VARIANTS {
            for &ceiling in &EscapeSurfaceProfile::VARIANTS {
                let reader = CeilingOnly(ceiling);
                let result = assert_namespace_capability(required, &reader);
                if required.ordinal() <= ceiling.ordinal() {
                    assert!(result.is_ok(), "CeilingOnly Approve for ({required:?}, {ceiling:?})");
                } else {
                    assert!(result.is_err(), "CeilingOnly Refuse for ({required:?}, {ceiling:?})");
                }
            }
        }
    }

    // rationale: Trait blanket impl over references — `&R` where
    // `R: AcceptanceSignatureReader` must forward the call. This
    // guarantees callers can hand m9 either `sig` or `&sig` without
    // changing the gate semantics.
    #[test]
    fn capability_gate_accepts_reference_to_reader() {
        let sig = HumanAcceptanceSignature {
            interactive_terminal: true,
            acknowledged_ceiling: EscapeSurfaceProfile::FileWrite,
        };
        // `&sig` (already an `&HumanAcceptanceSignature`) is one level of
        // reference; `&&sig` exercises the blanket impl for `&R`.
        let r1 = assert_namespace_capability(EscapeSurfaceProfile::FileWrite, &sig);
        let outer: &HumanAcceptanceSignature = &sig;
        let r2 = assert_namespace_capability(EscapeSurfaceProfile::FileWrite, &outer);
        assert!(r1.is_ok());
        assert!(r2.is_ok());
        // Above-ceiling case via the blanket impl on a borrowed reader.
        let refuse = assert_namespace_capability(EscapeSurfaceProfile::DataExfil, &outer);
        assert!(matches!(
            refuse,
            Err(NamespaceViolation::CapabilityNotAcknowledged { .. })
        ));
    }

    // rationale: Error Display surface — the `CapabilityNotAcknowledged`
    // message must name both profiles, both ordinals, and the Phase 6e
    // seam tag so an operator triaging from logs alone can identify the
    // gate.
    #[test]
    fn capability_not_acknowledged_display_names_required_and_acknowledged_and_seam() {
        let sig = HumanAcceptanceSignature {
            interactive_terminal: true,
            acknowledged_ceiling: EscapeSurfaceProfile::Sandboxed,
        };
        let err =
            assert_namespace_capability(EscapeSurfaceProfile::PrivilegeEscalation, &sig)
                .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("PrivilegeEscalation"), "missing required: {msg}");
        assert!(msg.contains("Sandboxed"), "missing acknowledged: {msg}");
        assert!(msg.contains("ord 30"), "missing required_ord: {msg}");
        assert!(msg.contains("ord 0"), "missing acknowledged_ord: {msg}");
        assert!(msg.contains("Phase 6e"), "missing seam tag: {msg}");
        assert!(msg.contains("m9"), "missing m9 layer tag: {msg}");
    }

    // rationale: Default-signature behaviour — `HumanAcceptanceSignature`
    // defaults to `Sandboxed` ceiling; m9's gate must therefore refuse
    // any required profile above `Sandboxed` under the default.
    #[test]
    fn capability_gate_refuses_above_sandboxed_under_default_signature() {
        let sig = HumanAcceptanceSignature::default();
        assert_eq!(sig.acknowledged_ceiling, EscapeSurfaceProfile::Sandboxed);
        for &required in &EscapeSurfaceProfile::VARIANTS {
            let result = assert_namespace_capability(required, &sig);
            if required.ordinal() == 0 {
                assert!(result.is_ok(), "Sandboxed required must Approve under default sig");
            } else {
                assert!(
                    matches!(result, Err(NamespaceViolation::CapabilityNotAcknowledged { .. })),
                    "{required:?} required must Refuse under default Sandboxed sig"
                );
            }
        }
    }

    // rationale: Determinism — repeated calls return identical results
    // (no internal mutation; pure validator beyond tracing).
    #[test]
    fn capability_gate_is_deterministic() {
        let sig = HumanAcceptanceSignature {
            interactive_terminal: true,
            acknowledged_ceiling: EscapeSurfaceProfile::FileWrite,
        };
        let first = assert_namespace_capability(EscapeSurfaceProfile::DataExfil, &sig);
        for _ in 0..100_u32 {
            let again = assert_namespace_capability(EscapeSurfaceProfile::DataExfil, &sig);
            assert_eq!(format!("{first:?}"), format!("{again:?}"));
        }
    }
}
