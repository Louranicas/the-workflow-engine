//! # M49: Task Graph
//!
//! DAG task representation with lifecycle FSM and cycle detection (Kahn's algorithm).
//!
//! ## Layer: L6 (Coordination)
//! ## Module: M49
//! ## Feature: `agentic`
//! ## Dependencies: `m01_core_types` (`PaneId`)

use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};

use crate::m1_core::m01_core_types::PaneId;
use crate::m1_core::m02_error_handling::{PvError, PvResult};

// ──────────────────────────────────────────────────────────────
// Constants
// ──────────────────────────────────────────────────────────────

const DEFAULT_MAX_NODES: usize = 1000;

// ──────────────────────────────────────────────────────────────
// Task node state
// ──────────────────────────────────────────────────────────────

/// Lifecycle state of a task node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskNodeState {
    /// Waiting for dependencies to complete.
    Pending,
    /// All dependencies met — ready to execute.
    Ready,
    /// Currently executing.
    InProgress,
    /// Completed successfully.
    Completed,
    /// Execution failed.
    Failed,
    /// Blocked by uncompleted dependencies.
    Blocked,
    /// Cancelled.
    Cancelled,
}

impl TaskNodeState {
    /// Whether this is a terminal state.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

impl std::fmt::Display for TaskNodeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => f.write_str("pending"),
            Self::Ready => f.write_str("ready"),
            Self::InProgress => f.write_str("in_progress"),
            Self::Completed => f.write_str("completed"),
            Self::Failed => f.write_str("failed"),
            Self::Blocked => f.write_str("blocked"),
            Self::Cancelled => f.write_str("cancelled"),
        }
    }
}

// ──────────────────────────────────────────────────────────────
// Task node
// ──────────────────────────────────────────────────────────────

/// A node in the task DAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskNode {
    /// Unique task identifier.
    pub task_id: String,
    /// Description of the task.
    pub description: String,
    /// Pane assigned to execute (if any).
    pub assigned_to: Option<PaneId>,
    /// Current lifecycle state.
    pub state: TaskNodeState,
    /// Priority score [0.0, 1.0].
    pub priority: f64,
    /// Tick at which this task was created.
    pub created_at_tick: u64,
    /// Tick at which this task completed (if terminal).
    pub completed_at_tick: Option<u64>,
}

// ──────────────────────────────────────────────────────────────
// Edge type
// ──────────────────────────────────────────────────────────────

/// Type of relationship between task nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskEdgeType {
    /// `from` must complete before `to` can start.
    DependsOn,
    /// `from` prevents `to` from starting.
    Blocks,
    /// Informational link — no execution constraint.
    RelatedTo,
}

impl std::fmt::Display for TaskEdgeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DependsOn => f.write_str("depends_on"),
            Self::Blocks => f.write_str("blocks"),
            Self::RelatedTo => f.write_str("related_to"),
        }
    }
}

/// A directed edge in the task graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEdge {
    /// Source task ID.
    pub from: String,
    /// Target task ID.
    pub to: String,
    /// Edge type.
    pub edge_type: TaskEdgeType,
}

// ──────────────────────────────────────────────────────────────
// Configuration
// ──────────────────────────────────────────────────────────────

/// Task graph configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskGraphConfig {
    /// Maximum nodes.
    pub max_nodes: usize,
}

impl Default for TaskGraphConfig {
    fn default() -> Self {
        Self { max_nodes: DEFAULT_MAX_NODES }
    }
}

// ──────────────────────────────────────────────────────────────
// Statistics
// ──────────────────────────────────────────────────────────────

/// Task graph statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskGraphStats {
    /// Total nodes.
    pub node_count: usize,
    /// Total edges.
    pub edge_count: usize,
    /// Nodes by state.
    pub pending: usize,
    /// Ready nodes.
    pub ready: usize,
    /// In-progress nodes.
    pub in_progress: usize,
    /// Completed nodes.
    pub completed: usize,
    /// Failed nodes.
    pub failed: usize,
}

// ──────────────────────────────────────────────────────────────
// Task graph
// ──────────────────────────────────────────────────────────────

/// DAG-structured task graph with lifecycle FSM.
pub struct TaskGraph {
    nodes: HashMap<String, TaskNode>,
    edges: Vec<TaskEdge>,
    config: TaskGraphConfig,
}

impl TaskGraph {
    /// Create a new task graph.
    #[must_use]
    pub fn new(config: TaskGraphConfig) -> Self {
        Self { nodes: HashMap::new(), edges: Vec::new(), config }
    }

    /// Create with default configuration.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(TaskGraphConfig::default())
    }

    /// Add a task node.
    ///
    /// # Errors
    ///
    /// Returns error if duplicate ID or max nodes reached.
    pub fn add_node(&mut self, node: TaskNode) -> PvResult<()> {
        if self.nodes.contains_key(&node.task_id) {
            return Err(PvError::ConfigValidation(
                format!("task {} already exists", node.task_id),
            ));
        }
        if self.nodes.len() >= self.config.max_nodes {
            return Err(PvError::ConfigValidation(
                format!("max nodes ({}) reached", self.config.max_nodes),
            ));
        }
        self.nodes.insert(node.task_id.clone(), node);
        Ok(())
    }

    /// Add a dependency edge.
    ///
    /// # Errors
    ///
    /// Returns error if either node doesn't exist or edge creates a cycle.
    pub fn add_edge(&mut self, from: &str, to: &str, edge_type: TaskEdgeType) -> PvResult<()> {
        if !self.nodes.contains_key(from) {
            return Err(PvError::ConfigValidation(format!("node {from} not found")));
        }
        if !self.nodes.contains_key(to) {
            return Err(PvError::ConfigValidation(format!("node {to} not found")));
        }
        if from == to {
            return Err(PvError::ConfigValidation("self-edge not allowed".into()));
        }

        self.edges.push(TaskEdge { from: from.into(), to: to.into(), edge_type });

        if edge_type == TaskEdgeType::DependsOn && self.has_cycle() {
            self.edges.pop();
            return Err(PvError::ConfigValidation(
                format!("edge {from} → {to} would create a cycle"),
            ));
        }
        Ok(())
    }

    /// Get a node by ID.
    #[must_use]
    pub fn get(&self, task_id: &str) -> Option<&TaskNode> {
        self.nodes.get(task_id)
    }

    /// Get a mutable reference to a node.
    #[must_use]
    pub fn get_mut(&mut self, task_id: &str) -> Option<&mut TaskNode> {
        self.nodes.get_mut(task_id)
    }

    /// Tasks that are ready (all `DependsOn` deps completed).
    #[must_use]
    pub fn ready_tasks(&self) -> Vec<&TaskNode> {
        self.nodes.values()
            .filter(|n| !n.state.is_terminal() && n.state != TaskNodeState::InProgress)
            .filter(|n| {
                let deps: Vec<_> = self.edges.iter()
                    .filter(|e| e.to == n.task_id && e.edge_type == TaskEdgeType::DependsOn)
                    .collect();
                deps.iter().all(|e| {
                    self.nodes.get(&e.from)
                        .map_or(true, |dep| dep.state == TaskNodeState::Completed)
                })
            })
            .collect()
    }

    /// Tasks that are blocked (have uncompleted dependencies).
    #[must_use]
    pub fn blocked_tasks(&self) -> Vec<(&TaskNode, Vec<String>)> {
        self.nodes.values()
            .filter(|n| !n.state.is_terminal() && n.state != TaskNodeState::InProgress)
            .filter_map(|n| {
                let blockers: Vec<String> = self.edges.iter()
                    .filter(|e| e.to == n.task_id && e.edge_type == TaskEdgeType::DependsOn)
                    .filter(|e| {
                        self.nodes.get(&e.from)
                            .is_some_and(|dep| !dep.state.is_terminal() || dep.state == TaskNodeState::Failed)
                    })
                    .map(|e| e.from.clone())
                    .collect();
                if blockers.is_empty() { None } else { Some((n, blockers)) }
            })
            .collect()
    }

    /// Complete a task and return newly unblocked task IDs.
    ///
    /// # Errors
    ///
    /// Returns error if task not found.
    pub fn complete(&mut self, task_id: &str, tick: u64) -> PvResult<Vec<String>> {
        let node = self.nodes.get_mut(task_id).ok_or_else(|| {
            PvError::ConfigValidation(format!("task {task_id} not found"))
        })?;
        node.state = TaskNodeState::Completed;
        node.completed_at_tick = Some(tick);

        let dependents: Vec<String> = self.edges.iter()
            .filter(|e| e.from == task_id && e.edge_type == TaskEdgeType::DependsOn)
            .map(|e| e.to.clone())
            .collect();

        let mut unblocked = Vec::new();
        for dep_id in &dependents {
            if self.is_ready(dep_id) {
                unblocked.push(dep_id.clone());
            }
        }
        Ok(unblocked)
    }

    /// Fail a task.
    ///
    /// # Errors
    ///
    /// Returns error if task not found.
    pub fn fail(&mut self, task_id: &str, tick: u64) -> PvResult<()> {
        let node = self.nodes.get_mut(task_id).ok_or_else(|| {
            PvError::ConfigValidation(format!("task {task_id} not found"))
        })?;
        node.state = TaskNodeState::Failed;
        node.completed_at_tick = Some(tick);
        Ok(())
    }

    /// Detect cycles using Kahn's algorithm (BFS topological sort).
    #[must_use]
    pub fn has_cycle(&self) -> bool {
        let dep_edges: Vec<_> = self.edges.iter()
            .filter(|e| e.edge_type == TaskEdgeType::DependsOn)
            .collect();

        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        for node_id in self.nodes.keys() {
            in_degree.insert(node_id.as_str(), 0);
        }
        for edge in &dep_edges {
            *in_degree.entry(edge.to.as_str()).or_insert(0) += 1;
        }

        let mut queue: VecDeque<&str> = in_degree.iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut visited = 0usize;
        while let Some(node_id) = queue.pop_front() {
            visited += 1;
            for edge in &dep_edges {
                if edge.from == node_id {
                    if let Some(deg) = in_degree.get_mut(edge.to.as_str()) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(edge.to.as_str());
                        }
                    }
                }
            }
        }

        visited < self.nodes.len()
    }

    /// Topological ordering of task IDs (`DependsOn` edges only).
    ///
    /// # Errors
    ///
    /// Returns error if the graph contains a cycle.
    pub fn topological_order(&self) -> PvResult<Vec<String>> {
        if self.has_cycle() {
            return Err(PvError::ConfigValidation("graph contains a cycle".into()));
        }

        let dep_edges: Vec<_> = self.edges.iter()
            .filter(|e| e.edge_type == TaskEdgeType::DependsOn)
            .collect();

        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        for node_id in self.nodes.keys() {
            in_degree.insert(node_id.as_str(), 0);
        }
        for edge in &dep_edges {
            *in_degree.entry(edge.to.as_str()).or_insert(0) += 1;
        }

        let mut queue: VecDeque<&str> = in_degree.iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut order = Vec::new();
        while let Some(node_id) = queue.pop_front() {
            order.push(node_id.to_owned());
            for edge in &dep_edges {
                if edge.from == node_id {
                    if let Some(deg) = in_degree.get_mut(edge.to.as_str()) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(edge.to.as_str());
                        }
                    }
                }
            }
        }
        Ok(order)
    }

    /// Number of nodes.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Number of edges.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Statistics.
    #[must_use]
    pub fn stats(&self) -> TaskGraphStats {
        let mut s = TaskGraphStats {
            node_count: self.nodes.len(),
            edge_count: self.edges.len(),
            ..Default::default()
        };
        for n in self.nodes.values() {
            match n.state {
                TaskNodeState::Pending | TaskNodeState::Blocked => s.pending += 1,
                TaskNodeState::Ready => s.ready += 1,
                TaskNodeState::InProgress => s.in_progress += 1,
                TaskNodeState::Completed => s.completed += 1,
                TaskNodeState::Failed | TaskNodeState::Cancelled => s.failed += 1,
            }
        }
        s
    }

    fn is_ready(&self, task_id: &str) -> bool {
        let deps: Vec<_> = self.edges.iter()
            .filter(|e| e.to == task_id && e.edge_type == TaskEdgeType::DependsOn)
            .collect();
        deps.iter().all(|e| {
            self.nodes.get(&e.from)
                .map_or(true, |dep| dep.state == TaskNodeState::Completed)
        })
    }
}

impl std::fmt::Debug for TaskGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskGraph")
            .field("nodes", &self.nodes.len())
            .field("edges", &self.edges.len())
            .finish_non_exhaustive()
    }
}

// ──────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn graph() -> TaskGraph {
        TaskGraph::with_defaults()
    }

    fn node(id: &str) -> TaskNode {
        TaskNode {
            task_id: id.into(),
            description: format!("Task {id}"),
            assigned_to: None,
            state: TaskNodeState::Pending,
            priority: 0.5,
            created_at_tick: 0,
            completed_at_tick: None,
        }
    }

    // ── Construction ──

    #[test]
    fn construction_default() {
        let g = graph();
        assert_eq!(g.node_count(), 0);
        assert_eq!(g.edge_count(), 0);
    }

    #[test]
    fn construction_custom() {
        let cfg = TaskGraphConfig { max_nodes: 5 };
        let g = TaskGraph::new(cfg);
        assert_eq!(g.node_count(), 0);
    }

    #[test]
    fn construction_empty_stats() {
        let g = graph();
        assert_eq!(g.stats().node_count, 0);
    }

    // ── Node operations ──

    #[test]
    fn add_node() {
        let mut g = graph();
        g.add_node(node("a")).unwrap();
        assert_eq!(g.node_count(), 1);
    }

    #[test]
    fn add_duplicate_node_error() {
        let mut g = graph();
        g.add_node(node("a")).unwrap();
        assert!(g.add_node(node("a")).is_err());
    }

    #[test]
    fn add_max_nodes() {
        let cfg = TaskGraphConfig { max_nodes: 2 };
        let mut g = TaskGraph::new(cfg);
        g.add_node(node("a")).unwrap();
        g.add_node(node("b")).unwrap();
        assert!(g.add_node(node("c")).is_err());
    }

    #[test]
    fn get_node() {
        let mut g = graph();
        g.add_node(node("a")).unwrap();
        assert!(g.get("a").is_some());
        assert!(g.get("b").is_none());
    }

    #[test]
    fn get_mut_node() {
        let mut g = graph();
        g.add_node(node("a")).unwrap();
        g.get_mut("a").unwrap().state = TaskNodeState::InProgress;
        assert_eq!(g.get("a").unwrap().state, TaskNodeState::InProgress);
    }

    #[test]
    fn node_assignment() {
        let mut g = graph();
        let mut n = node("a");
        n.assigned_to = Some(PaneId::new("alpha"));
        g.add_node(n).unwrap();
        assert_eq!(g.get("a").unwrap().assigned_to.as_ref().unwrap().as_str(), "alpha");
    }

    #[test]
    fn node_priority() {
        let mut g = graph();
        let mut n = node("a");
        n.priority = 0.9;
        g.add_node(n).unwrap();
        assert!((g.get("a").unwrap().priority - 0.9).abs() < f64::EPSILON);
    }

    // ── Edge operations ──

    #[test]
    fn add_depends_on_edge() {
        let mut g = graph();
        g.add_node(node("a")).unwrap();
        g.add_node(node("b")).unwrap();
        g.add_edge("a", "b", TaskEdgeType::DependsOn).unwrap();
        assert_eq!(g.edge_count(), 1);
    }

    #[test]
    fn add_blocks_edge() {
        let mut g = graph();
        g.add_node(node("a")).unwrap();
        g.add_node(node("b")).unwrap();
        g.add_edge("a", "b", TaskEdgeType::Blocks).unwrap();
        assert_eq!(g.edge_count(), 1);
    }

    #[test]
    fn add_related_edge() {
        let mut g = graph();
        g.add_node(node("a")).unwrap();
        g.add_node(node("b")).unwrap();
        g.add_edge("a", "b", TaskEdgeType::RelatedTo).unwrap();
        assert_eq!(g.edge_count(), 1);
    }

    #[test]
    fn add_edge_missing_from() {
        let mut g = graph();
        g.add_node(node("b")).unwrap();
        assert!(g.add_edge("a", "b", TaskEdgeType::DependsOn).is_err());
    }

    #[test]
    fn add_edge_missing_to() {
        let mut g = graph();
        g.add_node(node("a")).unwrap();
        assert!(g.add_edge("a", "b", TaskEdgeType::DependsOn).is_err());
    }

    #[test]
    fn add_self_edge_rejected() {
        let mut g = graph();
        g.add_node(node("a")).unwrap();
        assert!(g.add_edge("a", "a", TaskEdgeType::DependsOn).is_err());
    }

    // ── Ready / blocked ──

    #[test]
    fn ready_no_deps() {
        let mut g = graph();
        g.add_node(node("a")).unwrap();
        assert_eq!(g.ready_tasks().len(), 1);
    }

    #[test]
    fn ready_all_deps_met() {
        let mut g = graph();
        let mut a = node("a");
        a.state = TaskNodeState::Completed;
        g.add_node(a).unwrap();
        g.add_node(node("b")).unwrap();
        g.add_edge("a", "b", TaskEdgeType::DependsOn).unwrap();
        let ready = g.ready_tasks();
        assert!(ready.iter().any(|t| t.task_id == "b"));
    }

    #[test]
    fn blocked_deps_not_met() {
        let mut g = graph();
        g.add_node(node("a")).unwrap();
        g.add_node(node("b")).unwrap();
        g.add_edge("a", "b", TaskEdgeType::DependsOn).unwrap();
        let blocked = g.blocked_tasks();
        assert!(blocked.iter().any(|(t, _)| t.task_id == "b"));
    }

    #[test]
    fn ready_empty_graph() {
        let g = graph();
        assert!(g.ready_tasks().is_empty());
    }

    #[test]
    fn ready_single_node() {
        let mut g = graph();
        g.add_node(node("a")).unwrap();
        assert_eq!(g.ready_tasks().len(), 1);
    }

    // ── Complete / fail ──

    #[test]
    fn complete_unblocks_dependents() {
        let mut g = graph();
        g.add_node(node("a")).unwrap();
        g.add_node(node("b")).unwrap();
        g.add_edge("a", "b", TaskEdgeType::DependsOn).unwrap();
        let unblocked = g.complete("a", 10).unwrap();
        assert!(unblocked.contains(&"b".to_string()));
    }

    #[test]
    fn complete_not_found() {
        let mut g = graph();
        assert!(g.complete("nope", 0).is_err());
    }

    #[test]
    fn fail_task() {
        let mut g = graph();
        g.add_node(node("a")).unwrap();
        g.fail("a", 10).unwrap();
        assert_eq!(g.get("a").unwrap().state, TaskNodeState::Failed);
    }

    #[test]
    fn complete_sets_tick() {
        let mut g = graph();
        g.add_node(node("a")).unwrap();
        g.complete("a", 42).unwrap();
        assert_eq!(g.get("a").unwrap().completed_at_tick, Some(42));
    }

    // ── Cycle detection ──

    #[test]
    fn no_cycle_linear() {
        let mut g = graph();
        g.add_node(node("a")).unwrap();
        g.add_node(node("b")).unwrap();
        g.add_node(node("c")).unwrap();
        g.add_edge("a", "b", TaskEdgeType::DependsOn).unwrap();
        g.add_edge("b", "c", TaskEdgeType::DependsOn).unwrap();
        assert!(!g.has_cycle());
    }

    #[test]
    fn cycle_rejected_on_add() {
        let mut g = graph();
        g.add_node(node("a")).unwrap();
        g.add_node(node("b")).unwrap();
        g.add_edge("a", "b", TaskEdgeType::DependsOn).unwrap();
        assert!(g.add_edge("b", "a", TaskEdgeType::DependsOn).is_err());
    }

    #[test]
    fn complex_cycle_rejected() {
        let mut g = graph();
        g.add_node(node("a")).unwrap();
        g.add_node(node("b")).unwrap();
        g.add_node(node("c")).unwrap();
        g.add_edge("a", "b", TaskEdgeType::DependsOn).unwrap();
        g.add_edge("b", "c", TaskEdgeType::DependsOn).unwrap();
        assert!(g.add_edge("c", "a", TaskEdgeType::DependsOn).is_err());
    }

    #[test]
    fn no_cycle_diamond() {
        let mut g = graph();
        g.add_node(node("a")).unwrap();
        g.add_node(node("b")).unwrap();
        g.add_node(node("c")).unwrap();
        g.add_node(node("d")).unwrap();
        g.add_edge("a", "b", TaskEdgeType::DependsOn).unwrap();
        g.add_edge("a", "c", TaskEdgeType::DependsOn).unwrap();
        g.add_edge("b", "d", TaskEdgeType::DependsOn).unwrap();
        g.add_edge("c", "d", TaskEdgeType::DependsOn).unwrap();
        assert!(!g.has_cycle());
    }

    #[test]
    fn related_to_ignores_cycles() {
        let mut g = graph();
        g.add_node(node("a")).unwrap();
        g.add_node(node("b")).unwrap();
        g.add_edge("a", "b", TaskEdgeType::RelatedTo).unwrap();
        g.add_edge("b", "a", TaskEdgeType::RelatedTo).unwrap();
        assert!(!g.has_cycle());
    }

    // ── Topological sort ──

    #[test]
    fn topo_linear() {
        let mut g = graph();
        g.add_node(node("a")).unwrap();
        g.add_node(node("b")).unwrap();
        g.add_node(node("c")).unwrap();
        g.add_edge("a", "b", TaskEdgeType::DependsOn).unwrap();
        g.add_edge("b", "c", TaskEdgeType::DependsOn).unwrap();
        let order = g.topological_order().unwrap();
        let pos_a = order.iter().position(|x| x == "a").unwrap();
        let pos_b = order.iter().position(|x| x == "b").unwrap();
        let pos_c = order.iter().position(|x| x == "c").unwrap();
        assert!(pos_a < pos_b);
        assert!(pos_b < pos_c);
    }

    #[test]
    fn topo_diamond() {
        let mut g = graph();
        g.add_node(node("a")).unwrap();
        g.add_node(node("b")).unwrap();
        g.add_node(node("c")).unwrap();
        g.add_node(node("d")).unwrap();
        g.add_edge("a", "b", TaskEdgeType::DependsOn).unwrap();
        g.add_edge("a", "c", TaskEdgeType::DependsOn).unwrap();
        g.add_edge("b", "d", TaskEdgeType::DependsOn).unwrap();
        g.add_edge("c", "d", TaskEdgeType::DependsOn).unwrap();
        let order = g.topological_order().unwrap();
        assert_eq!(order.len(), 4);
        let pos_a = order.iter().position(|x| x == "a").unwrap();
        let pos_d = order.iter().position(|x| x == "d").unwrap();
        assert!(pos_a < pos_d);
    }

    #[test]
    fn topo_empty() {
        let g = graph();
        let order = g.topological_order().unwrap();
        assert!(order.is_empty());
    }

    #[test]
    fn topo_single() {
        let mut g = graph();
        g.add_node(node("a")).unwrap();
        let order = g.topological_order().unwrap();
        assert_eq!(order, vec!["a"]);
    }

    // ── Stats ──

    #[test]
    fn stats_counts() {
        let mut g = graph();
        g.add_node(node("a")).unwrap();
        g.add_node(node("b")).unwrap();
        g.add_edge("a", "b", TaskEdgeType::DependsOn).unwrap();
        let s = g.stats();
        assert_eq!(s.node_count, 2);
        assert_eq!(s.edge_count, 1);
    }

    #[test]
    fn stats_by_state() {
        let mut g = graph();
        g.add_node(node("a")).unwrap();
        g.add_node(node("b")).unwrap();
        g.complete("a", 10).unwrap();
        let s = g.stats();
        assert_eq!(s.completed, 1);
        assert_eq!(s.pending, 1);
    }

    // ── State FSM ──

    #[test]
    fn state_terminal() {
        assert!(TaskNodeState::Completed.is_terminal());
        assert!(TaskNodeState::Failed.is_terminal());
        assert!(TaskNodeState::Cancelled.is_terminal());
        assert!(!TaskNodeState::Pending.is_terminal());
        assert!(!TaskNodeState::Ready.is_terminal());
        assert!(!TaskNodeState::InProgress.is_terminal());
    }

    #[test]
    fn state_display() {
        assert_eq!(format!("{}", TaskNodeState::Pending), "pending");
        assert_eq!(format!("{}", TaskNodeState::Completed), "completed");
    }

    #[test]
    fn state_serde() {
        let s = TaskNodeState::InProgress;
        let json = serde_json::to_string(&s).unwrap();
        let back: TaskNodeState = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    // ── Edge type ──

    #[test]
    fn edge_type_display() {
        assert_eq!(format!("{}", TaskEdgeType::DependsOn), "depends_on");
        assert_eq!(format!("{}", TaskEdgeType::Blocks), "blocks");
        assert_eq!(format!("{}", TaskEdgeType::RelatedTo), "related_to");
    }

    #[test]
    fn edge_type_serde() {
        let e = TaskEdgeType::DependsOn;
        let json = serde_json::to_string(&e).unwrap();
        let back: TaskEdgeType = serde_json::from_str(&json).unwrap();
        assert_eq!(e, back);
    }

    // ── Serde ──

    #[test]
    fn node_serde() {
        let n = node("test");
        let json = serde_json::to_string(&n).unwrap();
        let back: TaskNode = serde_json::from_str(&json).unwrap();
        assert_eq!(back.task_id, "test");
    }

    #[test]
    fn edge_serde() {
        let e = TaskEdge { from: "a".into(), to: "b".into(), edge_type: TaskEdgeType::DependsOn };
        let json = serde_json::to_string(&e).unwrap();
        let back: TaskEdge = serde_json::from_str(&json).unwrap();
        assert_eq!(back.from, "a");
    }

    #[test]
    fn config_serde() {
        let cfg = TaskGraphConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let back: TaskGraphConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.max_nodes, DEFAULT_MAX_NODES);
    }

    #[test]
    fn stats_serde() {
        let s = TaskGraphStats { node_count: 5, edge_count: 3, ..Default::default() };
        let json = serde_json::to_string(&s).unwrap();
        let back: TaskGraphStats = serde_json::from_str(&json).unwrap();
        assert_eq!(back.node_count, 5);
    }

    // ── Edge cases ──

    #[test]
    fn many_nodes() {
        let mut g = graph();
        for i in 0..100 {
            g.add_node(node(&format!("n{i}"))).unwrap();
        }
        assert_eq!(g.node_count(), 100);
    }

    #[test]
    fn deep_chain() {
        let mut g = graph();
        for i in 0..50 {
            g.add_node(node(&format!("n{i}"))).unwrap();
        }
        for i in 0..49 {
            g.add_edge(&format!("n{i}"), &format!("n{}", i + 1), TaskEdgeType::DependsOn).unwrap();
        }
        assert!(!g.has_cycle());
        let order = g.topological_order().unwrap();
        assert_eq!(order.len(), 50);
    }

    #[test]
    fn wide_fan_out() {
        let mut g = graph();
        g.add_node(node("root")).unwrap();
        for i in 0..20 {
            g.add_node(node(&format!("leaf{i}"))).unwrap();
            g.add_edge("root", &format!("leaf{i}"), TaskEdgeType::DependsOn).unwrap();
        }
        assert!(!g.has_cycle());
        g.complete("root", 10).unwrap();
        assert_eq!(g.ready_tasks().len(), 20);
    }

    // ── Debug ──

    #[test]
    fn debug_format() {
        let g = graph();
        let debug = format!("{g:?}");
        assert!(debug.contains("TaskGraph"));
    }

    // ── Integration ──

    #[test]
    fn full_lifecycle() {
        let mut g = graph();
        g.add_node(node("build")).unwrap();
        g.add_node(node("test")).unwrap();
        g.add_node(node("deploy")).unwrap();
        g.add_edge("build", "test", TaskEdgeType::DependsOn).unwrap();
        g.add_edge("test", "deploy", TaskEdgeType::DependsOn).unwrap();
        assert_eq!(g.ready_tasks().len(), 1);
        assert_eq!(g.ready_tasks()[0].task_id, "build");
        let unblocked = g.complete("build", 10).unwrap();
        assert!(unblocked.contains(&"test".to_string()));
        g.get_mut("test").unwrap().state = TaskNodeState::InProgress;
        let unblocked = g.complete("test", 20).unwrap();
        assert!(unblocked.contains(&"deploy".to_string()));
        g.complete("deploy", 30).unwrap();
        assert_eq!(g.stats().completed, 3);
    }
}
