//! Workflow-trace-local mirrors of stcortex delta row types.
//!
//! Per m2 spec § 2: m2 emits a typed enum [`StcortexRow`] to downstream
//! callbacks. We use **local** mirror types rather than re-exporting the
//! module_bindings types so that any future stcortex schema change
//! produces a localised parse error rather than a silent shape drift
//! through the engine.

use super::identity::{ConsumerName, Namespace};

/// One `tool_call` delta row in the narrowed window.
#[derive(Debug, Clone)]
pub struct ToolCallRow {
    /// stcortex primary key.
    pub call_id: u64,
    /// Session UUID / opaque trajectory identifier.
    pub session_id: String,
    /// Validated workflow_trace_* namespace.
    pub namespace: Namespace,
    /// The tool / command name verbatim.
    pub command: String,
    /// Wall-clock recording time (ms since epoch).
    pub recorded_at_ms: i64,
}

/// One `consumption_event` delta row (substrate access-gradient signal).
#[derive(Debug, Clone)]
pub struct ConsumptionEventRow {
    /// Memory primary key consumed.
    pub memory_id: u64,
    /// Which consumer recorded the access.
    pub consumer_name: ConsumerName,
    /// Wall-clock consumption time (ms since epoch).
    pub consumed_at_ms: i64,
}

/// Typed delta row enum delivered to consumer callbacks.
#[derive(Debug, Clone)]
pub enum StcortexRow {
    /// A `tool_call` delta.
    ToolCall(ToolCallRow),
    /// A `consumption_event` delta.
    ConsumptionEvent(ConsumptionEventRow),
}

impl StcortexRow {
    /// Helper for log/metrics emit.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::ToolCall(_) => "tool_call",
            Self::ConsumptionEvent(_) => "consumption_event",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::identity::{ConsumerName, Namespace};
    use super::{ConsumptionEventRow, StcortexRow, ToolCallRow};

    #[test]
    fn tool_call_row_clone_preserves_fields() {
        let r = ToolCallRow {
            call_id: 7,
            session_id: "session-a".into(),
            namespace: Namespace::new("workflow_trace_x").unwrap(),
            command: "echo".into(),
            recorded_at_ms: 1,
        };
        let c = r.clone();
        assert_eq!(c.call_id, r.call_id);
        assert_eq!(c.session_id, r.session_id);
        assert_eq!(c.namespace, r.namespace);
    }

    #[test]
    fn consumption_event_row_clone_preserves_fields() {
        let r = ConsumptionEventRow {
            memory_id: 42,
            consumer_name: ConsumerName::new("workflow-trace-c").unwrap(),
            consumed_at_ms: 1,
        };
        let c = r.clone();
        assert_eq!(c.memory_id, r.memory_id);
        assert_eq!(c.consumer_name, r.consumer_name);
    }

    #[test]
    fn stcortex_row_kind_disambiguates() {
        let tc = StcortexRow::ToolCall(ToolCallRow {
            call_id: 1,
            session_id: "s".into(),
            namespace: Namespace::new("workflow_trace_x").unwrap(),
            command: "c".into(),
            recorded_at_ms: 0,
        });
        let ce = StcortexRow::ConsumptionEvent(ConsumptionEventRow {
            memory_id: 1,
            consumer_name: ConsumerName::new("workflow-trace-c").unwrap(),
            consumed_at_ms: 0,
        });
        assert_eq!(tc.kind(), "tool_call");
        assert_eq!(ce.kind(), "consumption_event");
    }

    // rationale: Contract regression — `kind()` is a const-fn (used in
    // log/metric hot paths). The match must be exhaustive over exactly two
    // variants; this pins the wire-string set against silent enum growth.
    #[test]
    fn stcortex_row_kind_is_one_of_exactly_two_wire_strings() {
        let tc = StcortexRow::ToolCall(ToolCallRow {
            call_id: 1,
            session_id: "s".into(),
            namespace: Namespace::new("workflow_trace_k").unwrap(),
            command: "c".into(),
            recorded_at_ms: 1_700_000_000_000,
        });
        let ce = StcortexRow::ConsumptionEvent(ConsumptionEventRow {
            memory_id: 2,
            consumer_name: ConsumerName::new("workflow-trace-k").unwrap(),
            consumed_at_ms: 1_700_000_000_001,
        });
        for row in [tc, ce] {
            let k = row.kind();
            assert!(
                k == "tool_call" || k == "consumption_event",
                "unexpected kind wire-string {k:?}"
            );
        }
    }

    // rationale: Boundary — row mirrors must carry adversarial epoch-ms
    // extremes (negative, i64::MAX) without mangling. The mirror is a
    // value type; it is the engine's localised parse surface.
    #[test]
    fn tool_call_row_preserves_extreme_timestamp_values() {
        for ts in [i64::MIN, -1, 0, 1_700_000_000_000, i64::MAX] {
            let r = ToolCallRow {
                call_id: u64::MAX,
                session_id: "session-extreme".into(),
                namespace: Namespace::new("workflow_trace_ts").unwrap(),
                command: "cargo test".into(),
                recorded_at_ms: ts,
            };
            assert_eq!(r.recorded_at_ms, ts);
            assert_eq!(r.call_id, u64::MAX);
        }
    }

    // rationale: Boundary — a consumption-event row with memory_id 0 and
    // an extreme consumed_at is preserved verbatim; 0 is a legal stcortex
    // primary key edge and must not be conflated with "absent".
    #[test]
    fn consumption_event_row_preserves_zero_memory_id() {
        let r = ConsumptionEventRow {
            memory_id: 0,
            consumer_name: ConsumerName::new("workflow-trace-z").unwrap(),
            consumed_at_ms: i64::MAX,
        };
        assert_eq!(r.memory_id, 0);
        assert_eq!(r.consumed_at_ms, i64::MAX);
    }

    // rationale: Contract — the typed enum wraps the row payload without
    // copying-by-value drift; the wrapped ToolCallRow round-trips through
    // a clone of the enum with all fields intact.
    #[test]
    fn stcortex_row_clone_preserves_wrapped_tool_call_payload() {
        let original = StcortexRow::ToolCall(ToolCallRow {
            call_id: 99,
            session_id: "session-clone".into(),
            namespace: Namespace::new("workflow_trace_clone").unwrap(),
            command: "git status".into(),
            recorded_at_ms: 1_700_000_123_456,
        });
        let cloned = original.clone();
        let (StcortexRow::ToolCall(a), StcortexRow::ToolCall(b)) = (&original, &cloned) else {
            panic!("clone changed the enum variant");
        };
        assert_eq!(a.call_id, b.call_id);
        assert_eq!(a.session_id, b.session_id);
        assert_eq!(a.command, b.command);
        assert_eq!(a.recorded_at_ms, b.recorded_at_ms);
    }
}
