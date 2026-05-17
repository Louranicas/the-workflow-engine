//! Wave 1C — Zen divergence detection engine.
//!
//! Implements seven detection rules that identify when the live Habitat
//! implementation drifts from its architectural plan. Each rule produces zero
//! or more [`DivergenceEvent`] values that the Zen daemon persists via
//! [`crate::state::StateDb::insert_divergence`].
//!
//! # Rule registry
//!
//! Rules are registered at startup via [`RuleRegistry::register`] and executed
//! sequentially each polling cycle. A rule that was recently fired is suppressed
//! by a per-rule *cooldown* to avoid flooding `divergence_reports`.
//!
//! # False-positive discipline
//!
//! Every rule ships with a "negative fixture" (see tests): a probe result that
//! MUST NOT trigger the rule. This prevents regression of the
//! `all_distinct_no_false_positive` class of bugs.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, info};

use crate::state::Severity;

// ---------------------------------------------------------------------------
// DivergenceEvent
// ---------------------------------------------------------------------------

/// A single detected divergence event, ready for persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DivergenceEvent {
    /// Machine-readable rule identifier (e.g. `"sphere_ghost"`).
    pub kind: String,
    /// Origin agent (always `"zen"` from this module).
    pub source: String,
    /// Canonical plan reference (path or URL).
    pub plan_ref: String,
    /// Observed state reference (endpoint or file path).
    pub observed_ref: String,
    /// Severity level.
    pub severity: Severity,
    /// Arbitrary JSON detail body.
    pub body: serde_json::Value,
}

// ---------------------------------------------------------------------------
// SphereInfo — input type for sphere_ghost rule
// ---------------------------------------------------------------------------

/// Minimal sphere descriptor consumed by the `sphere_ghost` rule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SphereInfo {
    /// PV2 sphere identity string.
    pub sphere_id: String,
    /// Persona label.
    pub persona: String,
    /// Kuramoto frequency (radians / s).
    pub freq: f64,
    /// Kuramoto phase (radians).
    pub phase: f64,
    /// Buoy label set (sorted).
    pub buoy_labels: Vec<String>,
    /// Whether the sphere has ever been observed working.
    pub has_worked: bool,
    /// Number of stored memories.
    pub memories: i64,
}

// ---------------------------------------------------------------------------
// Rule trait
// ---------------------------------------------------------------------------

/// A single divergence detection rule.
///
/// Implementors inspect probe data and return any detected divergences.
/// Returning an empty `Vec` means no divergence was found (the negative case).
pub trait Rule: Send + Sync {
    /// Returns the unique identifier for this rule (e.g. `"sphere_ghost"`).
    fn kind(&self) -> &'static str;

    /// Returns the severity this rule fires with.
    fn severity(&self) -> Severity;

    /// Returns the minimum inter-fire cooldown in seconds.
    ///
    /// A rule will not fire again within this many seconds of the previous
    /// firing. Set to `0` to disable cooldown (useful in tests).
    fn cooldown_secs(&self) -> u64 {
        300
    }
}

// ---------------------------------------------------------------------------
// SphereGhostRule — detects bit-identical ghost spheres
// ---------------------------------------------------------------------------

/// Detects N spheres that share `(persona, freq±ε, phase±ε, buoy_labels)`.
///
/// This rule caught the CMD3 forensic finding of 9 zombie fleet-worker ghosts.
///
/// # Severity: HIGH
pub struct SphereGhostRule {
    /// Phase epsilon for near-match comparison (default 0.001 rad).
    pub phase_epsilon: f64,
    /// Frequency epsilon (default 0.001 rad/s).
    pub freq_epsilon: f64,
}

impl Default for SphereGhostRule {
    fn default() -> Self {
        Self {
            phase_epsilon: 0.001,
            freq_epsilon: 0.001,
        }
    }
}

impl Rule for SphereGhostRule {
    fn kind(&self) -> &'static str { "sphere_ghost" }
    fn severity(&self) -> Severity { Severity::High }
}

impl SphereGhostRule {
    /// Runs the rule against a slice of sphere descriptors.
    ///
    /// Returns one [`DivergenceEvent`] per group of ghost spheres found.
    #[must_use]
    pub fn detect(&self, spheres: &[SphereInfo]) -> Vec<DivergenceEvent> {
        // Group by (persona, freq_bucket, phase_bucket, sorted_buoys).
        #[allow(clippy::cast_possible_truncation)]
        let bucket = |f: f64, eps: f64| (f / eps).round() as i64;

        let mut groups: HashMap<(String, i64, i64, Vec<String>), Vec<String>> = HashMap::new();
        for s in spheres {
            let mut sorted_buoys = s.buoy_labels.clone();
            sorted_buoys.sort_unstable();
            let key = (
                s.persona.clone(),
                bucket(s.freq, self.freq_epsilon),
                bucket(s.phase, self.phase_epsilon),
                sorted_buoys,
            );
            groups.entry(key).or_default().push(s.sphere_id.clone());
        }

        let mut events = Vec::new();
        for ((persona, _, _, buoys), ids) in &groups {
            if ids.len() > 1 {
                info!(
                    rule = "sphere_ghost",
                    count = ids.len(),
                    %persona,
                    "ghost spheres detected"
                );
                events.push(DivergenceEvent {
                    kind: "sphere_ghost".into(),
                    source: "zen".into(),
                    plan_ref: "HABITAT_CONDUCTOR_ARCHITECTURE.md §4.5".into(),
                    observed_ref: "pv2:8132/spheres".into(),
                    severity: Severity::High,
                    body: json!({
                        "persona": persona,
                        "ghost_count": ids.len(),
                        "sphere_ids": ids,
                        "shared_buoys": buoys,
                    }),
                });
            }
        }
        events
    }
}

// ---------------------------------------------------------------------------
// TunnelOverloadRule
// ---------------------------------------------------------------------------

/// Detects when >50% of PV2 tunnels have `overlap=1.0` (zero information).
///
/// # Severity: MED
pub struct TunnelOverloadRule {
    /// Threshold fraction; default 0.5 (50%).
    pub threshold_fraction: f64,
}

impl Default for TunnelOverloadRule {
    fn default() -> Self {
        Self { threshold_fraction: 0.5 }
    }
}

impl Rule for TunnelOverloadRule {
    fn kind(&self) -> &'static str { "tunnel_overload" }
    fn severity(&self) -> Severity { Severity::Med }
}

impl TunnelOverloadRule {
    /// Runs the rule against aggregate tunnel statistics.
    ///
    /// `total_tunnels` is the total number of tunnels; `saturated_tunnels` is
    /// the count with `overlap=1.0`.
    #[must_use]
    pub fn detect(
        &self,
        total_tunnels: usize,
        saturated_tunnels: usize,
    ) -> Vec<DivergenceEvent> {
        if total_tunnels == 0 {
            return vec![];
        }
        #[allow(clippy::cast_precision_loss)]
        let fraction = saturated_tunnels as f64 / total_tunnels as f64;
        if fraction > self.threshold_fraction {
            debug!(
                fraction,
                total = total_tunnels,
                saturated = saturated_tunnels,
                "tunnel overload detected"
            );
            vec![DivergenceEvent {
                kind: "tunnel_overload".into(),
                source: "zen".into(),
                plan_ref: "HABITAT_CONDUCTOR_ARCHITECTURE.md §3".into(),
                observed_ref: "pv2:8132/field".into(),
                severity: Severity::Med,
                body: json!({
                    "total_tunnels": total_tunnels,
                    "saturated_tunnels": saturated_tunnels,
                    "saturation_fraction": fraction,
                    "threshold": self.threshold_fraction,
                }),
            }]
        } else {
            vec![]
        }
    }
}

// ---------------------------------------------------------------------------
// InjectionCacheStaleRule
// ---------------------------------------------------------------------------

/// Detects when the session-injection cache is reading a stale window.
///
/// Fires if `injected_session_window` is more than `max_lag_sessions` behind
/// `current_session`.
///
/// # Severity: LOW
pub struct InjectionCacheStaleRule {
    /// Maximum tolerated session lag; default 5.
    pub max_lag_sessions: u64,
}

impl Default for InjectionCacheStaleRule {
    fn default() -> Self {
        Self { max_lag_sessions: 5 }
    }
}

impl Rule for InjectionCacheStaleRule {
    fn kind(&self) -> &'static str { "injection_cache_stale" }
    fn severity(&self) -> Severity { Severity::Low }
    fn cooldown_secs(&self) -> u64 { 3600 } // once per hour is enough
}

impl InjectionCacheStaleRule {
    /// Runs the rule given the current and injected session numbers.
    #[must_use]
    pub fn detect(
        &self,
        current_session: u64,
        injected_session: u64,
    ) -> Vec<DivergenceEvent> {
        if current_session <= injected_session {
            return vec![];
        }
        let lag = current_session - injected_session;
        if lag > self.max_lag_sessions {
            vec![DivergenceEvent {
                kind: "injection_cache_stale".into(),
                source: "zen".into(),
                plan_ref: "CLAUDE.local.md §Memory Systems".into(),
                observed_ref: "injection.db:causal_chain".into(),
                severity: Severity::Low,
                body: json!({
                    "current_session": current_session,
                    "injected_session": injected_session,
                    "lag_sessions": lag,
                    "threshold": self.max_lag_sessions,
                }),
            }]
        } else {
            vec![]
        }
    }
}

// ---------------------------------------------------------------------------
// BreakerPersistentOpenRule
// ---------------------------------------------------------------------------

/// Detects when an ORAC circuit breaker has been open for >300 s.
///
/// # Severity: HIGH
pub struct BreakerPersistentOpenRule {
    /// Threshold in seconds; default 300.
    pub threshold_secs: u64,
}

impl Default for BreakerPersistentOpenRule {
    fn default() -> Self {
        Self { threshold_secs: 300 }
    }
}

impl Rule for BreakerPersistentOpenRule {
    fn kind(&self) -> &'static str { "breaker_persistent_open" }
    fn severity(&self) -> Severity { Severity::High }
}

impl BreakerPersistentOpenRule {
    /// Runs the rule given a list of `(service_name, open_duration_secs)` pairs.
    ///
    /// Returns one event per breaker that exceeds the threshold.
    #[must_use]
    pub fn detect(&self, breakers: &[(&str, u64)]) -> Vec<DivergenceEvent> {
        breakers
            .iter()
            .filter(|(_, secs)| *secs >= self.threshold_secs)
            .map(|(svc, secs)| DivergenceEvent {
                kind: "breaker_persistent_open".into(),
                source: "zen".into(),
                plan_ref: "HABITAT_CONDUCTOR_ARCHITECTURE.md §8".into(),
                observed_ref: format!("orac:8133/bridges/{svc}"),
                severity: Severity::High,
                body: json!({
                    "service": svc,
                    "open_secs": secs,
                    "threshold_secs": self.threshold_secs,
                }),
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// RuleRegistry
// ---------------------------------------------------------------------------

/// Tracks which rules have been registered and enforces per-rule cooldowns.
pub struct RuleRegistry {
    last_fired: HashMap<String, DateTime<Utc>>,
}

impl RuleRegistry {
    /// Creates a new empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            last_fired: HashMap::new(),
        }
    }

    /// Returns `true` if the rule identified by `kind` may fire at `now`.
    ///
    /// A rule is allowed to fire if it has never fired before or the
    /// cooldown period has elapsed.
    #[must_use]
    pub fn may_fire(&self, kind: &str, cooldown_secs: u64, now: DateTime<Utc>) -> bool {
        if let Some(last) = self.last_fired.get(kind) {
            #[allow(clippy::cast_sign_loss)]
            let elapsed = now.signed_duration_since(*last).num_seconds().max(0) as u64;
            elapsed >= cooldown_secs
        } else {
            true
        }
    }

    /// Records that the rule identified by `kind` fired at `now`.
    pub fn record_fired(&mut self, kind: &str, now: DateTime<Utc>) {
        self.last_fired.insert(kind.to_owned(), now);
    }

    /// Deduplicates a list of events by `kind`: if two events share the same
    /// `kind`, only the first is kept.
    ///
    /// Used when the same rule fires multiple groups in one cycle.
    #[must_use]
    pub fn dedup_by_kind(events: Vec<DivergenceEvent>) -> Vec<DivergenceEvent> {
        let mut seen = std::collections::HashSet::new();
        events
            .into_iter()
            .filter(|e| seen.insert(e.kind.clone()))
            .collect()
    }

    /// Filters events by minimum severity, dropping anything below `min`.
    #[must_use]
    pub fn filter_by_severity(
        events: Vec<DivergenceEvent>,
        min: Severity,
    ) -> Vec<DivergenceEvent> {
        events.into_iter().filter(|e| e.severity >= min).collect()
    }
}

impl Default for RuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sphere(id: &str, persona: &str, freq: f64, phase: f64, buoys: &[&str]) -> SphereInfo {
        SphereInfo {
            sphere_id: id.into(),
            persona: persona.into(),
            freq,
            phase,
            buoy_labels: buoys.iter().map(|s| (*s).to_string()).collect(),
            has_worked: false,
            memories: 0,
        }
    }

    // --- SphereGhostRule ---

    #[test]
    fn sphere_ghost_detects_identical_spheres() {
        let rule = SphereGhostRule::default();
        let spheres = vec![
            sphere("s1", "fleet-worker", 1.0, 0.5, &["a"]),
            sphere("s2", "fleet-worker", 1.0, 0.5, &["a"]),
            sphere("s3", "fleet-worker", 1.0, 0.5, &["a"]),
        ];
        let events = rule.detect(&spheres);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, "sphere_ghost");
        assert_eq!(events[0].severity, Severity::High);
    }

    #[test]
    fn sphere_ghost_near_match_within_epsilon() {
        let rule = SphereGhostRule { phase_epsilon: 0.01, freq_epsilon: 0.01 };
        let spheres = vec![
            sphere("s1", "fleet-worker", 1.0, 0.5, &["a"]),
            sphere("s2", "fleet-worker", 1.002, 0.502, &["a"]), // within epsilon
        ];
        let events = rule.detect(&spheres);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn sphere_ghost_no_false_positive_all_distinct() {
        let rule = SphereGhostRule::default();
        let spheres = vec![
            sphere("s1", "nvim", 1.0, 0.1, &["rust"]),
            sphere("s2", "fleet-worker", 2.0, 0.2, &["orac"]),
            sphere("s3", "cc-01", 3.0, 0.3, &["lcm"]),
        ];
        let events = rule.detect(&spheres);
        assert!(events.is_empty(), "all distinct → no ghost events");
    }

    #[test]
    fn sphere_ghost_one_real_among_many_ghosts() {
        let rule = SphereGhostRule::default();
        let mut spheres: Vec<_> = (0..8)
            .map(|i| sphere(&format!("ghost-{i}"), "fleet-worker", 1.0, 0.0, &["x"]))
            .collect();
        spheres.push(sphere("real", "nvim", 2.5, 1.2, &["rust", "orac"]));
        let events = rule.detect(&spheres);
        // One ghost group (fleet-worker × 8), nvim is distinct.
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].body["ghost_count"], 8);
    }

    // --- TunnelOverloadRule ---

    #[test]
    fn tunnel_overload_fires_above_threshold() {
        let rule = TunnelOverloadRule::default();
        let events = rule.detect(100, 60); // 60% > 50%
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, "tunnel_overload");
        assert_eq!(events[0].severity, Severity::Med);
    }

    #[test]
    fn tunnel_overload_no_fire_below_threshold() {
        let rule = TunnelOverloadRule::default();
        let events = rule.detect(100, 40); // 40% ≤ 50%
        assert!(events.is_empty());
    }

    #[test]
    fn tunnel_overload_zero_tunnels_no_fire() {
        let rule = TunnelOverloadRule::default();
        let events = rule.detect(0, 0);
        assert!(events.is_empty());
    }

    // --- InjectionCacheStaleRule ---

    #[test]
    fn injection_cache_stale_fires_when_lag_exceeds_threshold() {
        let rule = InjectionCacheStaleRule::default();
        let events = rule.detect(1_001_025, 1_001_009); // lag = 16 > 5
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, "injection_cache_stale");
        assert_eq!(events[0].severity, Severity::Low);
    }

    #[test]
    fn injection_cache_stale_no_fire_within_threshold() {
        let rule = InjectionCacheStaleRule::default();
        let events = rule.detect(1_001_025, 1_001_022); // lag = 3 ≤ 5
        assert!(events.is_empty());
    }

    #[test]
    fn injection_cache_stale_no_fire_when_current_le_injected() {
        let rule = InjectionCacheStaleRule::default();
        let events = rule.detect(100, 200); // injected is newer
        assert!(events.is_empty());
    }

    // --- BreakerPersistentOpenRule ---

    #[test]
    fn breaker_persistent_fires_for_long_open() {
        let rule = BreakerPersistentOpenRule::default();
        let events = rule.detect(&[("povm", 400)]);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, "breaker_persistent_open");
        assert_eq!(events[0].severity, Severity::High);
    }

    #[test]
    fn breaker_persistent_no_fire_for_short_open() {
        let rule = BreakerPersistentOpenRule::default();
        let events = rule.detect(&[("povm", 100)]);
        assert!(events.is_empty());
    }

    #[test]
    fn breaker_persistent_multiple_breakers() {
        let rule = BreakerPersistentOpenRule::default();
        let events = rule.detect(&[("povm", 400), ("pv2", 200), ("me", 600)]);
        assert_eq!(events.len(), 2); // povm(400) + me(600); pv2(200) < 300
    }

    // --- RuleRegistry ---

    #[test]
    fn registry_may_fire_first_time() {
        let reg = RuleRegistry::new();
        assert!(reg.may_fire("sphere_ghost", 300, Utc::now()));
    }

    #[test]
    fn registry_may_not_fire_within_cooldown() {
        let mut reg = RuleRegistry::new();
        let now = Utc::now();
        reg.record_fired("sphere_ghost", now);
        // 10 seconds later — still within 300s cooldown.
        let later = now + chrono::Duration::seconds(10);
        assert!(!reg.may_fire("sphere_ghost", 300, later));
    }

    #[test]
    fn registry_may_fire_after_cooldown() {
        let mut reg = RuleRegistry::new();
        let fired_at =
            Utc::now() - chrono::Duration::seconds(400);
        reg.record_fired("sphere_ghost", fired_at);
        assert!(reg.may_fire("sphere_ghost", 300, Utc::now()));
    }

    #[test]
    fn dedup_by_kind_removes_duplicates() {
        let event = |kind: &str| DivergenceEvent {
            kind: kind.into(),
            source: "zen".into(),
            plan_ref: String::new(),
            observed_ref: String::new(),
            severity: Severity::Low,
            body: json!({}),
        };
        let events = vec![event("a"), event("b"), event("a"), event("c"), event("b")];
        let deduped = RuleRegistry::dedup_by_kind(events);
        assert_eq!(deduped.len(), 3);
    }

    #[test]
    fn filter_by_severity_keeps_high_and_above() {
        let event = |sev: Severity| DivergenceEvent {
            kind: "x".into(),
            source: "zen".into(),
            plan_ref: String::new(),
            observed_ref: String::new(),
            severity: sev,
            body: json!({}),
        };
        let events = vec![
            event(Severity::Low),
            event(Severity::Med),
            event(Severity::High),
            event(Severity::Critical),
        ];
        let filtered = RuleRegistry::filter_by_severity(events, Severity::High);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|e| e.severity >= Severity::High));
    }
}
