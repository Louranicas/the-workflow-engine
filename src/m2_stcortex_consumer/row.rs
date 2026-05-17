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
}
