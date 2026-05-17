//! `m3_aggregator` — Thread-safe Habitat state aggregator.
//!
//! [`Aggregator`] maintains the most recent [`HabitatState`] snapshot and
//! server uptime.  It is the single source of truth queried by the HTTP
//! handlers.  All access is through a [`parking_lot::RwLock`] so handlers
//! can hold the lock briefly and return owned values.
//!
//! # Usage
//!
//! ```rust
//! use habitat_nerve_center::m1_types::{MetricSnapshot, ProbeResult};
//! use habitat_nerve_center::m3_aggregator::Aggregator;
//! use std::time::Duration;
//!
//! let agg = Aggregator::new();
//! let probes = vec![ProbeResult::healthy("svc-a", 8080, 200, Duration::from_millis(12))];
//! agg.update_with_metrics(probes, MetricSnapshot::default()).expect("ok");
//! let state = agg.snapshot_state();
//! assert!(state.overall_health > 0.0);
//! ```

pub mod povm_bridge;

use crate::m1_types::{HabitatState, HealthStatus, MetricSnapshot, ProbeResult};
use parking_lot::RwLock;
use std::{sync::Arc, time::Instant};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors produced by [`Aggregator`] operations.
#[derive(Debug, Error)]
pub enum AggregatorError {
    /// No snapshot is available — [`Aggregator::update_with_metrics`] or
    /// [`Aggregator::update`] has not been called yet.
    #[error("no snapshot available: aggregator has not been updated yet")]
    NoSnapshot,
    /// An empty probe list was supplied; aggregation requires at least one
    /// result to produce a meaningful state.
    #[error("empty probe list supplied to aggregator")]
    EmptyProbes,
}

/// Thread-safe aggregator that stores the latest [`HabitatState`] snapshot.
///
/// Clone is `O(1)` — the inner `Arc` is simply reference-counted, so both the
/// original and the clone share the same lock and observe each other's writes.
///
/// # Thread safety
///
/// All reads go through `parking_lot::RwLock::read()` and all writes through
/// `parking_lot::RwLock::write()`.  Guards are dropped inside brace blocks so
/// they are never held across a function boundary.
#[derive(Debug, Clone)]
pub struct Aggregator {
    inner: Arc<AggregatorInner>,
}

#[derive(Debug)]
struct AggregatorInner {
    state: RwLock<Option<HabitatState>>,
    start: Instant,
}

impl Aggregator {
    /// Create a new [`Aggregator`] with no initial state.
    ///
    /// Call [`Aggregator::update`] or [`Aggregator::update_with_metrics`]
    /// before reading any state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(AggregatorInner {
                state: RwLock::new(None),
                start: Instant::now(),
            }),
        }
    }

    // ── Write paths ──────────────────────────────────────────────────────────

    /// Ingest a fresh list of [`ProbeResult`]s and update the stored snapshot.
    ///
    /// Metrics default to [`MetricSnapshot::default`].  Use
    /// [`Aggregator::update_with_metrics`] to attach enriched metric data.
    pub fn update(&self, probes: Vec<ProbeResult>) {
        let state = HabitatState::from_probes(probes);
        {
            let mut guard = self.inner.state.write();
            *guard = Some(state);
        }
    }

    /// Ingest probe results together with an enriched [`MetricSnapshot`] and
    /// compute a new [`HabitatState`].
    ///
    /// This is the preferred write path when Kuramoto `r`, SYNTHEX thermal,
    /// and ORAC RALPH metrics are available alongside the probe results.
    ///
    /// # Errors
    ///
    /// Returns [`AggregatorError::EmptyProbes`] when `probes` is empty.
    pub fn update_with_metrics(
        &self,
        probes: Vec<ProbeResult>,
        metrics: MetricSnapshot,
    ) -> Result<(), AggregatorError> {
        if probes.is_empty() {
            return Err(AggregatorError::EmptyProbes);
        }
        let mut state = HabitatState::from_probes(probes);
        state.metrics = metrics;
        {
            let mut guard = self.inner.state.write();
            *guard = Some(state);
        }
        Ok(())
    }

    /// Ingest a pre-built [`HabitatState`] directly.
    ///
    /// Useful when the state has been computed externally or reconstructed from
    /// a persistence layer.
    pub fn set_state(&self, state: HabitatState) {
        let mut guard = self.inner.state.write();
        *guard = Some(state);
    }

    // ── Read paths ───────────────────────────────────────────────────────────

    /// Return the most recently stored snapshot, or `None` if none has been
    /// ingested yet.
    #[must_use]
    pub fn snapshot(&self) -> Option<HabitatState> {
        self.inner.state.read().clone()
    }

    /// Return the most recently stored snapshot as an owned value.
    ///
    /// Returns an empty (zero-service) [`HabitatState`] when no update has
    /// been performed yet.  Use [`Aggregator::snapshot_result`] when you need
    /// to distinguish "no data" from "empty probe list".
    #[must_use]
    pub fn snapshot_state(&self) -> HabitatState {
        self.state_or_empty()
    }

    /// Return the most recently stored snapshot or an error if none exists.
    ///
    /// # Errors
    ///
    /// Returns [`AggregatorError::NoSnapshot`] when no update has been
    /// performed yet.
    pub fn snapshot_result(&self) -> Result<HabitatState, AggregatorError> {
        self.inner
            .state
            .read()
            .clone()
            .ok_or(AggregatorError::NoSnapshot)
    }

    /// Return a valid [`HabitatState`] in all cases.
    ///
    /// Returns the stored snapshot if present; otherwise returns an empty
    /// state produced from an empty probe list (all counts zero).
    #[must_use]
    pub fn state_or_empty(&self) -> HabitatState {
        let guard = self.inner.state.read();
        match guard.as_ref() {
            Some(s) => s.clone(),
            None => HabitatState::from_probes(vec![]),
        }
    }

    /// Return `true` when at least one snapshot has been stored.
    #[must_use]
    pub fn has_snapshot(&self) -> bool {
        self.inner.state.read().is_some()
    }

    /// Number of seconds since this [`Aggregator`] was created (proxy for
    /// server uptime).
    #[must_use]
    pub fn uptime_seconds(&self) -> u64 {
        self.inner.start.elapsed().as_secs()
    }

    // ── Query helpers ────────────────────────────────────────────────────────

    /// Look up the [`ProbeResult`] for a single service by name.
    ///
    /// Returns `None` when no snapshot exists or the named service is not
    /// present in the most recent probe batch.  The returned value is a clone;
    /// the lock is released before this function returns.
    #[must_use]
    pub fn service_status(&self, id: &str) -> Option<ProbeResult> {
        let guard = self.inner.state.read();
        guard.as_ref().and_then(|s| s.service(id).cloned())
    }

    /// Return all probe results whose status is not [`HealthStatus::Healthy`].
    ///
    /// In the current binary `Healthy`/`Unhealthy` model this includes every
    /// service with [`HealthStatus::Unhealthy`].  Returns an empty `Vec` when
    /// there is no snapshot or all services are healthy.
    #[must_use]
    pub fn degraded_services(&self) -> Vec<ProbeResult> {
        let guard = self.inner.state.read();
        match guard.as_ref() {
            None => Vec::new(),
            Some(s) => s
                .services
                .iter()
                .filter(|p| !p.status.is_healthy())
                .cloned()
                .collect(),
        }
    }

    /// Return the weighted health score for the current snapshot (0.0 – 1.0).
    ///
    /// Per-status weights:
    /// - [`HealthStatus::Healthy`]   → `1.0`
    /// - [`HealthStatus::Degraded`]  → `0.5`
    /// - [`HealthStatus::Unhealthy`] → `0.0`
    /// - [`HealthStatus::Unknown`]   → `0.0`
    ///
    /// Returns `0.0` when no snapshot exists or the probe list is empty.
    #[must_use]
    pub fn health_score(&self) -> f64 {
        let guard = self.inner.state.read();
        match guard.as_ref() {
            None => 0.0,
            Some(s) if s.total_count == 0 => 0.0,
            Some(s) => {
                let weight_sum: f64 = s
                    .services
                    .iter()
                    .map(|p| match p.status {
                        HealthStatus::Healthy => 1.0_f64,
                        HealthStatus::Degraded => 0.5_f64,
                        HealthStatus::Unhealthy | HealthStatus::Unknown => 0.0_f64,
                    })
                    .sum();
                #[allow(clippy::cast_precision_loss)]
                let score = weight_sum / s.total_count as f64;
                score
            }
        }
    }

    /// Return `true` when the system is in a critical state.
    ///
    /// Critical is defined as **either**:
    /// - More than **3** services unhealthy, **or**
    /// - `overall_health < 0.5`
    ///
    /// Returns `false` when no snapshot exists.
    #[must_use]
    pub fn is_critical(&self) -> bool {
        let guard = self.inner.state.read();
        match guard.as_ref() {
            None => false,
            Some(s) => {
                let unhealthy = s.total_count.saturating_sub(s.healthy_count);
                unhealthy > 3 || s.overall_health < 0.5
            }
        }
    }
}

impl Default for Aggregator {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m1_types::{HealthStatus, MetricSnapshot, ProbeResult};
    use chrono::Utc;
    use std::{sync::Arc, time::Duration};

    // ── helpers ──────────────────────────────────────────────────────────────

    fn healthy(name: &str) -> ProbeResult {
        ProbeResult::healthy(name, 8080, 200, Duration::from_millis(5))
    }

    fn unhealthy(name: &str) -> ProbeResult {
        ProbeResult::unhealthy(name, 8080, None, Duration::from_millis(3000), "timeout")
    }

    fn ten_healthy() -> Vec<ProbeResult> {
        (0..10_u16)
            .map(|i| ProbeResult::healthy(&format!("svc-{i}"), 8080 + i, 200, Duration::from_millis(10)))
            .collect()
    }

    fn metrics_with_values() -> MetricSnapshot {
        MetricSnapshot {
            pv2_r: Some(0.871),
            synthex_thermal: Some(0.424),
            synthex_temperature: None,
            orac_ralph_gen: Some(23_993),
            orac_ralph_fitness: Some(0.576),
            captured_at: Some(Utc::now()),
        }
    }

    // ── construction / defaults ───────────────────────────────────────────────

    #[test]
    fn new_aggregator_has_no_snapshot() {
        let agg = Aggregator::new();
        assert!(agg.snapshot().is_none());
    }

    #[test]
    fn default_equals_new_no_snapshot() {
        let agg = Aggregator::default();
        assert!(agg.snapshot().is_none());
    }

    #[test]
    fn has_snapshot_false_before_update() {
        let agg = Aggregator::new();
        assert!(!agg.has_snapshot());
    }

    #[test]
    fn has_snapshot_true_after_update() {
        let agg = Aggregator::new();
        agg.update(vec![healthy("svc-a")]);
        assert!(agg.has_snapshot());
    }

    // ── update (probes-only) ──────────────────────────────────────────────────

    #[test]
    fn update_stores_snapshot() {
        let agg = Aggregator::new();
        agg.update(vec![healthy("svc-a"), healthy("svc-b")]);
        let snap = agg.snapshot().expect("snapshot should exist after update");
        assert_eq!(snap.total_count, 2);
        assert_eq!(snap.healthy_count, 2);
    }

    #[test]
    fn update_overwrites_previous_snapshot() {
        let agg = Aggregator::new();
        agg.update(vec![healthy("svc-a")]);
        agg.update(vec![healthy("svc-a"), unhealthy("svc-b")]);
        let snap = agg.snapshot().expect("snapshot present after second update");
        assert_eq!(snap.total_count, 2);
        assert_eq!(snap.healthy_count, 1);
    }

    #[test]
    fn update_single_healthy_service() {
        let agg = Aggregator::new();
        agg.update(vec![healthy("lone")]);
        let snap = agg.snapshot().expect("snap");
        assert_eq!(snap.total_count, 1);
        assert_eq!(snap.healthy_count, 1);
        assert!((snap.overall_health - 1.0).abs() < 1e-9);
    }

    // ── update_with_metrics ───────────────────────────────────────────────────

    #[test]
    fn update_with_metrics_stores_snapshot() {
        let agg = Aggregator::new();
        agg.update_with_metrics(vec![healthy("svc-a")], MetricSnapshot::default())
            .expect("update_with_metrics must succeed");
        assert!(agg.has_snapshot());
    }

    #[test]
    fn update_with_metrics_attaches_metrics() {
        let agg = Aggregator::new();
        let m = metrics_with_values();
        agg.update_with_metrics(vec![healthy("svc-a")], m.clone())
            .expect("update_with_metrics must succeed");
        let snap = agg.snapshot().expect("snap");
        assert_eq!(snap.metrics.pv2_r, m.pv2_r);
        assert_eq!(snap.metrics.synthex_thermal, m.synthex_thermal);
        assert_eq!(snap.metrics.orac_ralph_gen, m.orac_ralph_gen);
        assert_eq!(snap.metrics.orac_ralph_fitness, m.orac_ralph_fitness);
    }

    #[test]
    fn update_with_metrics_empty_probes_is_err() {
        let agg = Aggregator::new();
        let result = agg.update_with_metrics(vec![], MetricSnapshot::default());
        assert!(matches!(result, Err(AggregatorError::EmptyProbes)));
    }

    #[test]
    fn update_with_metrics_does_not_modify_state_on_error() {
        let agg = Aggregator::new();
        agg.update(vec![healthy("prior")]);
        // Empty probes — must fail and leave prior state intact.
        let _ = agg.update_with_metrics(vec![], MetricSnapshot::default());
        let snap = agg.snapshot().expect("prior state must still exist");
        assert_eq!(snap.total_count, 1);
    }

    #[test]
    fn update_with_metrics_ten_services() {
        let agg = Aggregator::new();
        agg.update_with_metrics(ten_healthy(), MetricSnapshot::default())
            .expect("update must succeed");
        let snap = agg.snapshot().expect("snap");
        assert_eq!(snap.total_count, 10);
        assert_eq!(snap.healthy_count, 10);
    }

    // ── set_state ─────────────────────────────────────────────────────────────

    #[test]
    fn set_state_stores_prebuilt_state() {
        let agg = Aggregator::new();
        let state = HabitatState::from_probes(vec![healthy("x")]);
        agg.set_state(state);
        let snap = agg.snapshot().expect("snapshot after set_state");
        assert_eq!(snap.total_count, 1);
    }

    #[test]
    fn set_state_overwrites_existing() {
        let agg = Aggregator::new();
        agg.update(vec![healthy("a"), healthy("b")]);
        agg.set_state(HabitatState::from_probes(vec![healthy("only")]));
        let snap = agg.snapshot().expect("snap");
        assert_eq!(snap.total_count, 1);
    }

    // ── snapshot / state_or_empty / snapshot_state / snapshot_result ──────────

    #[test]
    fn state_or_empty_returns_empty_when_no_snapshot() {
        let agg = Aggregator::new();
        let state = agg.state_or_empty();
        assert_eq!(state.total_count, 0);
    }

    #[test]
    fn state_or_empty_returns_snapshot_when_present() {
        let agg = Aggregator::new();
        agg.update(vec![healthy("svc-a")]);
        let state = agg.state_or_empty();
        assert_eq!(state.total_count, 1);
    }

    #[test]
    fn snapshot_state_zero_before_update() {
        let agg = Aggregator::new();
        assert_eq!(agg.snapshot_state().total_count, 0);
    }

    #[test]
    fn snapshot_state_matches_last_update() {
        let agg = Aggregator::new();
        agg.update(vec![healthy("a"), unhealthy("b")]);
        let s = agg.snapshot_state();
        assert_eq!(s.total_count, 2);
        assert_eq!(s.healthy_count, 1);
    }

    #[test]
    fn snapshot_result_err_before_update() {
        let agg = Aggregator::new();
        assert!(matches!(agg.snapshot_result(), Err(AggregatorError::NoSnapshot)));
    }

    #[test]
    fn snapshot_result_ok_after_update() {
        let agg = Aggregator::new();
        agg.update(vec![healthy("svc")]);
        assert!(agg.snapshot_result().is_ok());
    }

    // ── overall_health counts ────────────────────────────────────────────────

    #[test]
    fn all_unhealthy_overall_health_is_zero() {
        let agg = Aggregator::new();
        agg.update(vec![unhealthy("a"), unhealthy("b")]);
        let snap = agg.snapshot().expect("snap");
        assert_eq!(snap.healthy_count, 0);
        assert!((snap.overall_health).abs() < 1e-9);
    }

    #[test]
    fn mixed_healthy_overall_health_fraction() {
        let agg = Aggregator::new();
        agg.update(vec![healthy("a"), unhealthy("b"), healthy("c"), healthy("d")]);
        let snap = agg.snapshot().expect("snap");
        assert!((snap.overall_health - 0.75).abs() < 1e-9);
    }

    #[test]
    fn all_healthy_overall_health_is_one() {
        let agg = Aggregator::new();
        agg.update(ten_healthy());
        let snap = agg.snapshot().expect("snap");
        assert!((snap.overall_health - 1.0).abs() < 1e-9);
    }

    // ── service_status ────────────────────────────────────────────────────────

    #[test]
    fn service_status_found_healthy() {
        let agg = Aggregator::new();
        agg.update(vec![healthy("maintenance-engine"), unhealthy("orac-sidecar")]);
        let result = agg.service_status("maintenance-engine");
        assert!(result.is_some());
        assert_eq!(result.expect("present").status, HealthStatus::Healthy);
    }

    #[test]
    fn service_status_found_unhealthy() {
        let agg = Aggregator::new();
        agg.update(vec![healthy("a"), unhealthy("broken")]);
        let result = agg.service_status("broken");
        assert!(result.is_some());
        assert_eq!(result.expect("present").status, HealthStatus::Unhealthy);
    }

    #[test]
    fn service_status_not_found_returns_none() {
        let agg = Aggregator::new();
        agg.update(vec![healthy("svc-a")]);
        assert!(agg.service_status("no-such-service").is_none());
    }

    #[test]
    fn service_status_before_update_is_none() {
        let agg = Aggregator::new();
        assert!(agg.service_status("anything").is_none());
    }

    #[test]
    fn service_status_returns_independent_clone() {
        let agg = Aggregator::new();
        agg.update(vec![healthy("svc")]);
        let r1 = agg.service_status("svc");
        let r2 = agg.service_status("svc");
        assert!(r1.is_some() && r2.is_some());
        assert_eq!(r1.expect("r1").name, r2.expect("r2").name);
    }

    // ── degraded_services ─────────────────────────────────────────────────────

    #[test]
    fn degraded_services_empty_when_all_healthy() {
        let agg = Aggregator::new();
        agg.update(ten_healthy());
        assert!(agg.degraded_services().is_empty());
    }

    #[test]
    fn degraded_services_returns_non_healthy_probes() {
        let agg = Aggregator::new();
        agg.update(vec![
            healthy("a"),
            unhealthy("b"),
            unhealthy("c"),
            healthy("d"),
        ]);
        let degraded = agg.degraded_services();
        assert_eq!(degraded.len(), 2);
        let names: Vec<&str> = degraded.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"b"));
        assert!(names.contains(&"c"));
    }

    #[test]
    fn degraded_services_empty_before_update() {
        let agg = Aggregator::new();
        assert!(agg.degraded_services().is_empty());
    }

    #[test]
    fn degraded_services_all_unhealthy() {
        let agg = Aggregator::new();
        agg.update(vec![unhealthy("x"), unhealthy("y"), unhealthy("z")]);
        assert_eq!(agg.degraded_services().len(), 3);
    }

    // ── health_score ──────────────────────────────────────────────────────────

    #[test]
    fn health_score_all_healthy_is_one() {
        let agg = Aggregator::new();
        agg.update(ten_healthy());
        assert!((agg.health_score() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn health_score_all_unhealthy_is_zero() {
        let agg = Aggregator::new();
        agg.update(vec![unhealthy("a"), unhealthy("b"), unhealthy("c")]);
        assert!(agg.health_score().abs() < 1e-9);
    }

    #[test]
    fn health_score_before_update_is_zero() {
        let agg = Aggregator::new();
        assert!(agg.health_score().abs() < 1e-9);
    }

    #[test]
    fn health_score_partial_match() {
        let agg = Aggregator::new();
        agg.update(vec![healthy("a"), healthy("b"), unhealthy("c"), unhealthy("d")]);
        let expected = 2.0 / 4.0;
        assert!((agg.health_score() - expected).abs() < 1e-9);
    }

    #[test]
    fn health_score_three_out_of_four_healthy() {
        let agg = Aggregator::new();
        agg.update(vec![healthy("a"), healthy("b"), healthy("c"), unhealthy("d")]);
        let expected = 3.0 / 4.0;
        assert!((agg.health_score() - expected).abs() < 1e-9);
    }

    // ── is_critical ───────────────────────────────────────────────────────────

    #[test]
    fn is_critical_false_when_all_healthy() {
        let agg = Aggregator::new();
        agg.update(ten_healthy());
        assert!(!agg.is_critical());
    }

    #[test]
    fn is_critical_true_when_four_or_more_unhealthy() {
        let agg = Aggregator::new();
        // 4 unhealthy out of 12 → unhealthy_count 4 > 3 → critical
        let mut probes = ten_healthy();
        for p in probes.iter_mut().take(4) {
            p.status = HealthStatus::Unhealthy;
        }
        agg.update(probes);
        assert!(agg.is_critical());
    }

    #[test]
    fn is_critical_false_when_exactly_three_unhealthy() {
        let agg = Aggregator::new();
        // 3 unhealthy out of 12 → overall_health 0.75, unhealthy 3 (not > 3)
        let mut probes = ten_healthy();
        for p in probes.iter_mut().take(3) {
            p.status = HealthStatus::Unhealthy;
        }
        agg.update(probes);
        assert!(!agg.is_critical());
    }

    #[test]
    fn is_critical_true_when_overall_health_below_half() {
        let agg = Aggregator::new();
        // 1 healthy, 3 unhealthy → overall_health 0.25 < 0.5 → critical
        agg.update(vec![
            healthy("a"),
            unhealthy("b"),
            unhealthy("c"),
            unhealthy("d"),
        ]);
        assert!(agg.is_critical());
    }

    #[test]
    fn is_critical_false_when_overall_health_exactly_half() {
        let agg = Aggregator::new();
        // 2 healthy, 2 unhealthy → overall_health 0.5, unhealthy 2 (not > 3)
        // 0.5 is NOT < 0.5 → not critical
        agg.update(vec![healthy("a"), healthy("b"), unhealthy("c"), unhealthy("d")]);
        assert!(!agg.is_critical());
    }

    #[test]
    fn is_critical_false_before_update() {
        let agg = Aggregator::new();
        assert!(!agg.is_critical());
    }

    // ── uptime ────────────────────────────────────────────────────────────────

    #[test]
    fn uptime_seconds_does_not_panic() {
        let agg = Aggregator::new();
        let _ = agg.uptime_seconds();
    }

    // ── clone shares inner state ──────────────────────────────────────────────

    #[test]
    fn clone_shares_inner_state() {
        let agg = Aggregator::new();
        let agg2 = agg.clone();
        agg.update(vec![healthy("shared")]);
        let snap = agg2.snapshot().expect("clone sees the update");
        assert_eq!(snap.total_count, 1);
    }

    #[test]
    fn clone_write_visible_to_original() {
        let agg = Aggregator::new();
        let agg2 = agg.clone();
        agg2.update(vec![healthy("via-clone"), healthy("also-via-clone")]);
        assert_eq!(agg.snapshot().expect("snap").total_count, 2);
    }

    // ── snapshot isolation ────────────────────────────────────────────────────

    #[test]
    fn snapshot_is_independent_clone() {
        let agg = Aggregator::new();
        agg.update(vec![healthy("svc")]);
        let s1 = agg.snapshot().expect("snap1");

        // Overwrite with a different state.
        agg.update(vec![healthy("svc"), unhealthy("svc2"), unhealthy("svc3")]);
        let s2 = agg.snapshot().expect("snap2");

        // s1 still reflects the original single-probe state.
        assert_eq!(s1.total_count, 1);
        assert_eq!(s2.total_count, 3);
    }

    // ── thread safety ─────────────────────────────────────────────────────────

    #[test]
    fn concurrent_updates_are_safe() {
        let agg = Arc::new(Aggregator::new());
        let handles: Vec<_> = (0..8)
            .map(|i| {
                let agg = Arc::clone(&agg);
                std::thread::spawn(move || {
                    agg.update(vec![healthy(&format!("svc-{i}"))]);
                })
            })
            .collect();
        for h in handles {
            h.join().expect("thread panicked");
        }
        assert!(agg.snapshot().is_some());
    }

    #[test]
    fn concurrent_reads_after_single_write() {
        let agg = Arc::new(Aggregator::new());
        agg.update(ten_healthy());

        let handles: Vec<_> = (0..8)
            .map(|_| {
                let agg = Arc::clone(&agg);
                std::thread::spawn(move || {
                    let s = agg.snapshot().expect("snapshot must exist");
                    assert_eq!(s.healthy_count, 10);
                })
            })
            .collect();

        for h in handles {
            h.join().expect("thread panicked");
        }
    }

    #[test]
    fn concurrent_update_with_metrics_are_safe() {
        let agg = Arc::new(Aggregator::new());
        let handles: Vec<_> = (0..4_u16)
            .map(|i| {
                let agg = Arc::clone(&agg);
                std::thread::spawn(move || {
                    agg.update_with_metrics(
                        vec![ProbeResult::healthy(
                            &format!("svc-{i}"),
                            8080 + i,
                            200,
                            Duration::from_millis(5),
                        )],
                        MetricSnapshot::default(),
                    )
                    .expect("update must succeed");
                })
            })
            .collect();

        for h in handles {
            h.join().expect("thread panicked");
        }

        assert!(agg.has_snapshot());
    }
}
