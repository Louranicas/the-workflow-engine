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
