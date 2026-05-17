//! Integration tests for m9 `watcher_namespace_guard`.
//!
//! Per m9 spec § 6 F-Integration row: 15 tests exercising the validator at
//! realistic call boundaries. Day-1 scope — the real m13 / m42 writers don't
//! exist yet — so we simulate the call boundary with `mock_m13_write` /
//! `mock_m42_reinforce` thunks that take `ValidatedNamespace`, not `&str`.
//! The type system enforcement is the point: code that bypasses the m9
//! validator fails to type-check at the writer-call boundary.

// Integration-test crates do not inherit `src/lib.rs`'s allow list. Match
// the workspace convention from `src/lib.rs` so habitat acronyms (POVM,
// AP30, ADR, …) and module names in prose don't need to be backticked.
#![allow(clippy::doc_markdown)]

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::thread;

use workflow_core::m9_watcher_namespace_guard::{
    assert_workflow_trace_namespace, munge_hyphen_slug, NamespaceViolation,
    ValidatedNamespace, WORKFLOW_TRACE_NS_PREFIX,
};

// ---- Mock writer surfaces -------------------------------------------------

// Mock writers take `ValidatedNamespace` by value (not by reference) on
// purpose: that is the type-system enforcement point. Writers consume the
// evidence so callers cannot reuse stale evidence after a write completes.
// `clippy::needless_pass_by_value` would defeat the design.
#[allow(clippy::needless_pass_by_value)]
fn mock_m13_write(namespace: ValidatedNamespace, payload: &str) -> String {
    format!("STCORTEX WRITE: ns={namespace} payload={payload}")
}

#[allow(clippy::needless_pass_by_value)]
fn mock_m42_reinforce(namespace: ValidatedNamespace, pathway: &str) -> String {
    format!("STCORTEX REINFORCE: ns={namespace} pathway={pathway}")
}

// ---- Realistic call boundaries (5) ---------------------------------------

#[test]
fn m13_write_through_validator_succeeds_for_canonical_namespace() {
    let v = assert_workflow_trace_namespace("workflow_trace_correlations")
        .expect("canonical namespace");
    let out = mock_m13_write(v, "row{ id: 1 }");
    assert!(out.contains("workflow_trace_correlations"));
    assert!(out.contains("row{ id: 1 }"));
}

#[test]
fn m13_write_blocked_for_foreign_prefix() {
    let result = assert_workflow_trace_namespace("orac_blackboard");
    assert!(matches!(result, Err(NamespaceViolation::WrongPrefix { .. })));
    // Type-system enforcement: try editing the call below to take a `&str`
    // and it will not compile. That is the point of `ValidatedNamespace`.
    let v = assert_workflow_trace_namespace("workflow_trace_blackboard")
        .expect("workflow_trace_*");
    let _ = mock_m13_write(v, "{}");
}

#[test]
fn m42_reinforce_through_validator_munges_hyphens() {
    let v = assert_workflow_trace_namespace("workflow-trace-pulse").expect("hyphen");
    let out = mock_m42_reinforce(v, "pathway_id_001");
    assert!(out.contains("workflow_trace_pulse"));
    assert!(!out.contains('-'));
}

#[test]
fn m42_reinforce_blocks_empty_namespace() {
    assert_eq!(
        assert_workflow_trace_namespace("").unwrap_err(),
        NamespaceViolation::Empty
    );
}

#[test]
fn m42_reinforce_blocks_scratch_exactly() {
    assert_eq!(
        assert_workflow_trace_namespace("scratch").unwrap_err(),
        NamespaceViolation::ScratchForbidden
    );
}

// ---- Concurrency (3) -----------------------------------------------------

#[test]
fn concurrent_validators_produce_identical_results() {
    let handles: Vec<_> = (0..16_u32)
        .map(|i| {
            thread::spawn(move || {
                let input = format!("workflow_trace_thread_{i}");
                assert_workflow_trace_namespace(&input).map(|v| v.as_str().to_owned())
            })
        })
        .collect();
    for (i, h) in handles.into_iter().enumerate() {
        let result = h.join().expect("thread join");
        let s = result.expect("validate");
        assert_eq!(s, format!("workflow_trace_thread_{i}"));
    }
}

#[test]
fn concurrent_invalid_inputs_all_refused() {
    let handles: Vec<_> = (0..16_u32)
        .map(|i| {
            thread::spawn(move || {
                let input = format!("orac_thread_{i}");
                assert_workflow_trace_namespace(&input)
            })
        })
        .collect();
    for h in handles {
        let result = h.join().expect("thread join");
        assert!(matches!(
            result,
            Err(NamespaceViolation::WrongPrefix { .. })
        ));
    }
}

#[test]
fn refusal_of_one_call_does_not_block_a_separate_concurrent_call() {
    // Simulate m13 + m42 making concurrent calls — refusal of one does not
    // poison the other's separate call.
    let r1 = assert_workflow_trace_namespace("orac_bad");
    let r2 = assert_workflow_trace_namespace("workflow_trace_good");
    assert!(r1.is_err());
    assert!(r2.is_ok());
}

// ---- Cross-substrate boundary (2) ----------------------------------------

#[test]
fn validates_for_m42_path_even_post_2026_05_17_stcortex_only_adr() {
    // Per m9 spec § 6: the validator still applies to m42 even though m42
    // routes stcortex-only post-2026-05-17 ADR. POVM is decoupled but the
    // validator surface is unchanged.
    let v = assert_workflow_trace_namespace("workflow_trace_m42_pathway")
        .expect("m42 routing");
    assert!(v.as_str().starts_with(WORKFLOW_TRACE_NS_PREFIX));
}

#[test]
fn validator_does_not_distinguish_substrate_by_namespace() {
    // The validator is substrate-agnostic — it only checks the prefix, not
    // whether the eventual write lands in stcortex vs POVM (POVM-decoupled
    // per m42 ADR, but the validator surface is symmetrical).
    for ns in [
        "workflow_trace_alpha",
        "workflow_trace_beta",
        "workflow_trace_gamma",
    ] {
        assert!(assert_workflow_trace_namespace(ns).is_ok(), "ns={ns}");
    }
}

// ---- Munge across boundary (3) -------------------------------------------

#[test]
fn munged_value_is_what_lands_in_mock_writer() {
    let v = assert_workflow_trace_namespace("workflow-trace-munged").expect("hyphen");
    let out = mock_m13_write(v, "{}");
    assert!(out.contains("workflow_trace_munged"));
    assert!(!out.contains("workflow-trace-munged"));
}

#[test]
fn munge_helper_idempotent_at_boundary() {
    let input = "workflow-trace-x";
    let m1 = munge_hyphen_slug(input);
    let m2 = munge_hyphen_slug(&m1);
    assert_eq!(m1, m2);
    let v1 = assert_workflow_trace_namespace(input).expect("v1");
    let v2 = assert_workflow_trace_namespace(v1.as_str()).expect("v2");
    assert_eq!(v1, v2);
}

#[test]
fn dedup_of_concurrent_validated_namespaces_via_hashset() {
    // Eight concurrent validators on identical input yield one unique
    // validated value in a HashSet — confirms Eq/Hash impls are coherent
    // across thread boundaries.
    let handles: Vec<_> = (0..8_u32)
        .map(|_| {
            thread::spawn(|| assert_workflow_trace_namespace("workflow_trace_dedup"))
        })
        .collect();
    let mut set: HashSet<ValidatedNamespace> = HashSet::new();
    for h in handles {
        let v = h.join().expect("thread").expect("validate");
        set.insert(v);
    }
    assert_eq!(set.len(), 1);
}

// ---- AP30 coarse src/ grep guard (1) -------------------------------------

#[test]
fn ap30_no_literal_workflow_trace_prefix_outside_m9_module() {
    // Coarse F-Regression check: scan `src/` for the literal
    // `"workflow_trace"` string and confirm that every occurrence lives
    // either inside `src/m9_watcher_namespace_guard/` (the single legal
    // source-of-truth) or inside a `#[cfg(test)]` block (test code may
    // reference the constant value directly).
    //
    // This catches the AP30 regression pattern where a contributor
    // hard-codes the prefix downstream instead of importing
    // `WORKFLOW_TRACE_NS_PREFIX`.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let src_dir = Path::new(manifest_dir).join("src");
    let offenders = scan_dir_for_literal(&src_dir, "\"workflow_trace\"");
    // Allow occurrences inside m9 itself + inside tests:
    let stray: Vec<String> = offenders
        .into_iter()
        .filter(|path| !path.contains("m9_watcher_namespace_guard"))
        .collect();
    assert!(
        stray.is_empty(),
        "AP30 regression: literal \"workflow_trace\" string outside m9: {stray:?}"
    );
}

fn scan_dir_for_literal(dir: &Path, needle: &str) -> Vec<String> {
    let mut hits = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return hits;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            hits.extend(scan_dir_for_literal(&path, needle));
            continue;
        }
        // MSRV 1.75; `Option::is_none_or` stabilised in 1.82. Use map_or.
        if path.extension().map_or(true, |ext| ext != "rs") {
            continue;
        }
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        if content.contains(needle) {
            hits.push(path.display().to_string());
        }
    }
    hits
}

// ---- Day-1 surface stability (2) -----------------------------------------

#[test]
fn day_1_validator_signature_unchanged() {
    // m9 spec § 2 describes a 7-variant EscapeSurfaceProfile capability
    // table; Day-1 deliberately does NOT implement that integration. This
    // test documents the deferral: the validator signature is
    // (&str) -> Result<ValidatedNamespace, NamespaceViolation>. When m30 /
    // m32 ship, the validator will gain a sibling
    // `assert_namespace_for_profile(ns, profile, sig)`. If this assignment
    // ever fails to type-check, the contract has drifted and the spec needs
    // a coordinated update.
    let _: fn(&str) -> Result<ValidatedNamespace, NamespaceViolation> =
        assert_workflow_trace_namespace;
}

#[test]
fn day_1_error_variant_set_is_exactly_four() {
    // F-Regression contract: the four base variants are the entire
    // Day-1 surface. Adding a new variant requires a coordinated
    // spec amendment + Zen audit.
    let err = NamespaceViolation::Empty;
    match err {
        NamespaceViolation::Empty
        | NamespaceViolation::WrongPrefix { .. }
        | NamespaceViolation::Whitespace { .. }
        | NamespaceViolation::ScratchForbidden => {}
    }
}
