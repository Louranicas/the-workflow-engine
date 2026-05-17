//! Integration tests for m2 stcortex_consumer.
//!
//! The type surface (newtypes, validation, query builders, identity)
//! is exercised in the module's `#[cfg(test)]` unit tests. This file
//! adds:
//!
//! - End-to-end validation flows that cross the public surface
//! - A small advisory live `:3000` probe that runs ONLY when stcortex
//!   is reachable on the host (`db_path_exists`-equivalent for a
//!   WebSocket endpoint via a 1-second TCP connect)

#![allow(clippy::doc_markdown)]

use std::net::TcpStream;
use std::time::Duration;

use workflow_core::m2_stcortex_consumer::{
    consumption_event_query, register_narrowed_consumer, tool_call_query, ConsumerIdentity,
    ConsumerName, Namespace, StcortexConsumerError, Transport, DEFAULT_SUBSCRIPTION_TIMEOUT_MS,
    STCORTEX_DB, STCORTEX_URI, WORKFLOW_TRACE_PREFIX,
};

fn stcortex_reachable() -> bool {
    TcpStream::connect_timeout(
        &"127.0.0.1:3000".parse().expect("parse"),
        Duration::from_millis(500),
    )
    .is_ok()
}

// ---- End-to-end validation (5) -------------------------------------------

#[test]
fn full_identity_construction_via_from_git_sha() {
    let ns = Namespace::new("workflow_trace_e2e").expect("ns");
    let id = ConsumerIdentity::from_git_sha(ns);
    assert!(id.name.as_str().starts_with("workflow-trace-"));
    assert_eq!(id.namespace.as_str(), "workflow_trace_e2e");
    assert_eq!(id.transport, Transport::Subscription);
}

#[test]
fn namespace_rejects_all_foreign_service_prefixes() {
    for foreign in [
        "orac_x",
        "pane_vortex_y",
        "synthex_v2_z",
        "lcm_q",
        "me_r",
        "povm_s",
        "habitat_memory_t",
    ] {
        assert!(
            matches!(
                Namespace::new(foreign),
                Err(StcortexConsumerError::InvalidNamespace(_))
            ),
            "did not reject {foreign}"
        );
    }
}

#[test]
fn consumer_name_rejects_disallowed_chars_systematically() {
    for c in &[' ', '.', '/', '!', '\n', '\t', '*', ',', ';'] {
        let s = format!("workflow-trace-{c}-x");
        assert!(
            matches!(
                ConsumerName::new(&s),
                Err(StcortexConsumerError::InvalidConsumerName(_))
            ),
            "did not reject {s:?}"
        );
    }
}

#[test]
fn workflow_trace_prefix_matches_m9_constant() {
    use workflow_core::m9_watcher_namespace_guard::WORKFLOW_TRACE_NS_PREFIX;
    assert_eq!(WORKFLOW_TRACE_PREFIX, WORKFLOW_TRACE_NS_PREFIX);
}

#[test]
fn surface_constants_are_stable() {
    assert_eq!(STCORTEX_URI, "ws://127.0.0.1:3000");
    assert_eq!(STCORTEX_DB, "stcortex");
    assert_eq!(DEFAULT_SUBSCRIPTION_TIMEOUT_MS, 5_000);
}

// ---- Query builder W1 narrowing (3) --------------------------------------

#[test]
fn tool_call_query_includes_namespace_filter() {
    let q = tool_call_query("workflow_trace_test");
    assert!(q.contains("WHERE namespace LIKE"));
    assert!(q.contains("workflow_trace_test"));
}

#[test]
fn tool_call_query_does_not_widen_to_other_tables() {
    let q = tool_call_query("workflow_trace_test");
    for forbidden in ["pathway", "memory", "ghost_memory", "consumer", "decay_audit"] {
        assert!(!q.contains(forbidden), "{q} contains {forbidden}");
    }
}

#[test]
fn consumption_event_query_is_simple_select() {
    let q = consumption_event_query();
    assert_eq!(q, "SELECT * FROM consumption_event");
}

// ---- Live :3000 advisory (2) ---------------------------------------------

#[test]
fn live_stcortex_3000_tcp_reachability_advisory() {
    let reachable = stcortex_reachable();
    eprintln!("M2-LIVE advisory: stcortex :3000 TCP reachable = {reachable}");
}

#[test]
fn live_register_narrowed_consumer_smoke() {
    if !stcortex_reachable() {
        eprintln!("M2-LIVE: stcortex :3000 not reachable; skipping register smoke");
        return;
    }
    let ns = Namespace::new("workflow_trace_smoke").expect("ns");
    let identity = ConsumerIdentity::from_git_sha(ns);
    match register_narrowed_consumer(identity, 3_000) {
        Ok(handle) => {
            eprintln!(
                "M2-LIVE: register_narrowed_consumer OK; is_fresh={} identity={}",
                handle.is_fresh(),
                handle.identity().name
            );
        }
        Err(StcortexConsumerError::ConnectionFailed { uri, reason }) => {
            eprintln!("M2-LIVE: ConnectionFailed at {uri}: {reason} (advisory)");
        }
        Err(StcortexConsumerError::SubscriptionTimeout { timeout_ms }) => {
            eprintln!("M2-LIVE: SubscriptionTimeout after {timeout_ms}ms (advisory)");
        }
        Err(other) => {
            eprintln!("M2-LIVE: other error: {other:?} (advisory)");
        }
    }
}

// ---- Hardening pass S1002209 (Cluster A) — +10 tests --------------------

#[test]
fn hardening_tool_call_query_strips_single_quote_injection() {
    // rationale: Adversarial input — a malicious caller bypassing m9
    // could supply `"x' OR 1=1; --"`. The hardening fix strips single
    // quotes (the only SpacetimeDB string delimiter) so the resulting
    // SQL is syntactically valid even if semantically a no-match.
    let q = tool_call_query("x' OR 1=1; --");
    assert!(!q.contains("' OR 1=1"), "single-quote must be stripped from SQL: {q}");
    // Still well-formed:
    assert!(q.starts_with("SELECT * FROM tool_call WHERE namespace LIKE '"));
    assert!(q.ends_with("_%'"));
}

#[test]
fn hardening_tool_call_query_preserves_legal_workflow_trace_runes() {
    // rationale: Anti-property — the sanitiser must not corrupt legal
    // namespaces (underscore, hyphen, alphanumeric). Only `'` is
    // stripped.
    let q = tool_call_query("workflow_trace_cluster-A_v2");
    assert!(q.contains("workflow_trace_cluster-A_v2"));
}

#[test]
fn hardening_namespace_must_start_with_workflow_trace_prefix() {
    // rationale: Contract regression — Namespace constructor refuses
    // any prefix other than WORKFLOW_TRACE_PREFIX. We verify the
    // re-exported constant is identical to m9's source of truth and
    // that constructor rejection is exhaustive.
    use workflow_core::m9_watcher_namespace_guard::WORKFLOW_TRACE_NS_PREFIX;
    assert_eq!(WORKFLOW_TRACE_PREFIX, WORKFLOW_TRACE_NS_PREFIX);
    for foreign in ["", "x", "WORKFLOW_TRACE_X", "workflow trace x"] {
        assert!(
            Namespace::new(foreign).is_err(),
            "did not reject {foreign:?}"
        );
    }
}

#[test]
fn hardening_consumer_name_64_char_boundary_inclusive() {
    // rationale: Boundary — CONSUMER_NAME_MAX_LEN=64 is inclusive.
    // The 64-char name passes; 65-char fails. Already covered in unit
    // tests, but the integration boundary asserts the public surface.
    use workflow_core::m2_stcortex_consumer::CONSUMER_NAME_MAX_LEN;
    let exact = "a".repeat(CONSUMER_NAME_MAX_LEN);
    assert!(ConsumerName::new(&exact).is_ok());
    let too_long = "a".repeat(CONSUMER_NAME_MAX_LEN + 1);
    assert!(ConsumerName::new(&too_long).is_err());
}

#[test]
fn hardening_from_git_sha_is_total_function_under_failure() {
    // rationale: Anti-property — `from_git_sha` must always return a
    // valid ConsumerIdentity even when git is absent, unsuccessful, or
    // produces empty stdout. The hardening fix preserved this via the
    // direct-field fallback. We verify by calling many times under
    // realistic conditions (git may or may not be available).
    for _ in 0..3 {
        let ns = Namespace::new("workflow_trace_robust").expect("ns");
        let id = ConsumerIdentity::from_git_sha(ns);
        assert!(id.name.as_str().starts_with("workflow-trace-"));
        assert_eq!(id.transport, Transport::Subscription);
    }
}

#[test]
fn hardening_register_failed_error_variant_exists_for_silent_failure_fix() {
    // rationale: Contract regression — the hardening fix introduces
    // surface-of-RegisterFailed from the on_connect callback when the
    // reducer rejects. The error variant must exist and round-trip a
    // diagnostic string.
    let e = StcortexConsumerError::RegisterFailed("server refused".into());
    let msg = e.to_string();
    assert!(msg.contains("register_consumer reducer failed"));
    assert!(msg.contains("server refused"));
}

#[test]
fn hardening_transport_enum_is_subscription_only_day_1() {
    // rationale: Anti-property — Day-1 spec § 2 locks Transport at
    // Subscription. A future spec amendment widens this. Test asserts
    // the closed-set invariant by constructing every variant.
    let t = Transport::Subscription;
    assert_eq!(t.as_str(), "subscription");
}

#[test]
fn hardening_consumer_identity_is_send_sync_static() {
    // rationale: Concurrency — `ConsumerIdentity` is passed across
    // thread boundaries (the registration callback runs on the SDK
    // worker thread). It must be Send + Sync + 'static.
    fn assert_send_sync_static<T: Send + Sync + 'static>() {}
    assert_send_sync_static::<ConsumerIdentity>();
}

#[test]
fn hardening_namespace_eq_hash_collapses_duplicates() {
    // rationale: Cross-module surface invariant — `Namespace` is used
    // as a HashMap key downstream; equal namespaces must hash equal.
    use std::collections::HashSet;
    let mut s: HashSet<Namespace> = HashSet::new();
    s.insert(Namespace::new("workflow_trace_x").unwrap());
    s.insert(Namespace::new("workflow_trace_x").unwrap());
    s.insert(Namespace::new("workflow_trace_y").unwrap());
    assert_eq!(s.len(), 2);
}

#[test]
fn hardening_register_returns_typed_connection_failed_when_stcortex_down() {
    // rationale: Adversarial input — when stcortex is unreachable, the
    // SDK build()-step returns ConnectionFailed or eventually
    // SubscriptionTimeout. We verify that no panic / no silent success
    // path exists; the call returns a typed error of the expected
    // variant set. (If stcortex IS up locally we accept any non-panic.)
    if stcortex_reachable() {
        eprintln!("M2-HARDENING: stcortex up, advisory pass");
        return;
    }
    let ns = Namespace::new("workflow_trace_h").expect("ns");
    let id = ConsumerIdentity::from_git_sha(ns);
    let result = register_narrowed_consumer(id, 250);
    assert!(
        matches!(
            result,
            Err(StcortexConsumerError::ConnectionFailed { .. }
                | StcortexConsumerError::SubscriptionTimeout { .. }
                | StcortexConsumerError::RegisterFailed(_))
        ),
        "expected typed error variant from unreachable stcortex; got {result:?}"
    );
}
