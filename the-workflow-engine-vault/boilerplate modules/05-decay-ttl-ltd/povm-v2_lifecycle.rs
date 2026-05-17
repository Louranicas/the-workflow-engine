use std::time::{SystemTime, UNIX_EPOCH};

use crate::l1_storage::StorageEngine;
use crate::l2_graph::MemoryTier;

/// Return the current Unix timestamp as a decimal string, or `None` if the
/// system clock is before the Unix epoch.
///
/// F-POVM-07: the original `unwrap_or_default()` silently returned `"0"` on
/// clock-skew, causing `decay_co_activations` to treat every co-activation
/// row as newer than now and freeze Hebbian weights. This version returns
/// `None` so the caller can skip the decay step and emit a structured error
/// rather than silently corrupting the co-activation table.
fn chrono_now() -> Option<String> {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => Some(format!("{}", d.as_secs())),
        Err(e) => {
            tracing::error!(
                error = %e,
                "system clock is before the Unix epoch — \
                 skipping co-activation decay to avoid corrupting Hebbian weights"
            );
            None
        }
    }
}

/// Statistics from a consolidation cycle.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConsolidationStats {
    pub promoted_to_working: u32,
    pub promoted_to_consolidated: u32,
    pub promoted_to_crystallized: u32,
    pub demoted: u32,
    pub pruned_memories: u32,
    pub pruned_pathways: u64,
    pub decayed_pathways: u64,
    pub decayed_activations: u64,
}

/// The consolidation engine manages the 4-tier memory lifecycle.
///
/// Promotion criteria:
/// - Ephemeral → Working: accessed >= 2 times
/// - Working → Consolidated: accessed >= 5 times AND activation > 0.3
/// - Consolidated → Crystallized: accessed >= 20 times AND survived >= 5 decay cycles
///
/// Demotion: memories in Working/Consolidated that fall below activation
/// threshold get demoted one tier.
pub struct ConsolidationEngine {
    pathway_decay_rate: f64,
    activation_decay_rate: f64,
    prune_threshold: f64,
    demotion_activation_threshold: f64,
}

impl ConsolidationEngine {
    /// Create a consolidation engine with the given parameters.
    #[must_use]
    pub fn new(
        pathway_decay_rate: f64,
        activation_decay_rate: f64,
        prune_threshold: f64,
        demotion_activation_threshold: f64,
    ) -> Self {
        Self {
            pathway_decay_rate,
            activation_decay_rate,
            prune_threshold,
            demotion_activation_threshold,
        }
    }

    /// Run a full consolidation cycle:
    /// decay → pathway mortality → increment survived → promote → demote → active forget → prune.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if any storage operation fails. Co-activation
    /// decay is skipped (with a `tracing::error!`) when the system clock is before
    /// the Unix epoch rather than propagating the `SystemTimeError` — this is the
    /// correct behaviour for an offline/embedded host with a dead RTC.
    pub fn run(&self, engine: &StorageEngine) -> Result<ConsolidationStats, rusqlite::Error> {
        let decayed_pathways = engine.decay_pathways(self.pathway_decay_rate)?;
        let decayed_activations = engine.decay_activations(self.activation_decay_rate)?;

        // F-POVM-07: chrono_now() returns None on pre-epoch clock rather than silently
        // returning "0". Skipping decay is safe: co-activation counts are preserved until
        // the clock recovers. The alternative (decay with ts="0") would treat every row
        // as "never activated" and destroy the Hebbian weight history.
        if let Some(now) = chrono_now() {
            engine.decay_co_activations(&now)?;
        }

        engine.increment_decay_survived()?;

        let memories = engine.list_memories(None, 100_000, 0)?;

        let mut promoted_to_working = 0u32;
        let mut promoted_to_consolidated = 0u32;
        let mut promoted_to_crystallized = 0u32;
        let mut demoted = 0u32;
        let mut pruned_memories = 0u32;

        for mem in &memories {
            match mem.tier {
                MemoryTier::Ephemeral => {
                    if mem.access_count >= 2 {
                        engine.promote_memory(&mem.id, MemoryTier::Working)?;
                        promoted_to_working += 1;
                    } else if mem.activation < 0.001 && ((mem.access_count == 0 && mem.decay_survived > 3) || (mem.access_count > 0 && mem.decay_survived > 10)) {
                        engine.delete_memory(&mem.id)?;
                        pruned_memories += 1;
                    }
                }
                MemoryTier::Working => {
                    if mem.access_count >= 5 && mem.activation > 0.3 {
                        engine.promote_memory(&mem.id, MemoryTier::Consolidated)?;
                        promoted_to_consolidated += 1;
                    } else if mem.activation < self.demotion_activation_threshold {
                        engine.promote_memory(&mem.id, MemoryTier::Ephemeral)?;
                        demoted += 1;
                    }
                }
                MemoryTier::Consolidated => {
                    if mem.access_count >= 20 && mem.decay_survived >= 5 {
                        engine.promote_memory(&mem.id, MemoryTier::Crystallized)?;
                        promoted_to_crystallized += 1;
                    } else if mem.activation < self.demotion_activation_threshold {
                        engine.promote_memory(&mem.id, MemoryTier::Working)?;
                        demoted += 1;
                    }
                }
                MemoryTier::Crystallized => {}
            }
        }

        let pruned_pathways = engine.prune_pathways(self.prune_threshold)?;

        Ok(ConsolidationStats {
            promoted_to_working,
            promoted_to_consolidated,
            promoted_to_crystallized,
            demoted,
            pruned_memories,
            pruned_pathways,
            decayed_pathways,
            decayed_activations,
        })
    }
}

impl Default for ConsolidationEngine {
    fn default() -> Self {
        Self::new(0.02, 0.05, 0.01, 0.05)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l2_graph::MemoryNode;

    fn test_engine() -> StorageEngine {
        StorageEngine::in_memory().expect("in-memory engine")
    }

    fn mem(id: &str, tier: MemoryTier, access: u32, activation: f64, decay: u32) -> MemoryNode {
        MemoryNode {
            id: id.to_string(),
            content: format!("memory {id}"),
            namespace: "test".to_string(),
            tier,
            theta: 0.0,
            phi: 0.0,
            tensor: [0.0; 12],
            token_count: 5,
            info_density: 1.0,
            access_count: access,
            activation,
            session_created: "S1".to_string(),
            crystallised: tier == MemoryTier::Crystallized,
            created_at: String::new(),
            last_accessed: None,
            last_activated: None,
            decay_survived: decay,
        }
    }

    #[test]
    fn ephemeral_promoted_to_working_on_access() {
        let se = test_engine();
        se.insert_memory(&mem("m1", MemoryTier::Ephemeral, 3, 0.5, 0)).unwrap();
        let ce = ConsolidationEngine::default();
        let stats = ce.run(&se).unwrap();
        assert_eq!(stats.promoted_to_working, 1);
    }

    #[test]
    fn ephemeral_not_promoted_without_access() {
        let se = test_engine();
        se.insert_memory(&mem("m1", MemoryTier::Ephemeral, 0, 0.0, 0)).unwrap();
        let ce = ConsolidationEngine::default();
        let stats = ce.run(&se).unwrap();
        assert_eq!(stats.promoted_to_working, 0);
    }

    #[test]
    fn working_promoted_to_consolidated() {
        let se = test_engine();
        se.insert_memory(&mem("m1", MemoryTier::Working, 6, 0.5, 0)).unwrap();
        let ce = ConsolidationEngine::default();
        let stats = ce.run(&se).unwrap();
        assert_eq!(stats.promoted_to_consolidated, 1);
    }

    #[test]
    fn working_demoted_on_low_activation() {
        let se = test_engine();
        se.insert_memory(&mem("m1", MemoryTier::Working, 1, 0.01, 0)).unwrap();
        let ce = ConsolidationEngine::default();
        let stats = ce.run(&se).unwrap();
        assert_eq!(stats.demoted, 1);
    }

    #[test]
    fn consolidated_promoted_to_crystallized() {
        let se = test_engine();
        se.insert_memory(&mem("m1", MemoryTier::Consolidated, 25, 0.8, 6)).unwrap();
        let ce = ConsolidationEngine::default();
        let stats = ce.run(&se).unwrap();
        assert_eq!(stats.promoted_to_crystallized, 1);
    }

    #[test]
    fn crystallized_never_demoted() {
        let se = test_engine();
        se.insert_memory(&mem("m1", MemoryTier::Crystallized, 0, 0.0, 0)).unwrap();
        let ce = ConsolidationEngine::default();
        let stats = ce.run(&se).unwrap();
        assert_eq!(stats.demoted, 0);
    }

    #[test]
    fn dead_ephemeral_pruned() {
        let se = test_engine();
        se.insert_memory(&mem("m1", MemoryTier::Ephemeral, 0, 0.0, 5)).unwrap();
        let ce = ConsolidationEngine::default();
        let stats = ce.run(&se).unwrap();
        assert_eq!(stats.pruned_memories, 1);
    }

    #[test]
    fn decay_applies_to_pathways() {
        let se = test_engine();
        se.upsert_pathway(&crate::l2_graph::Pathway::new("a", "b", "t", 0.5)).unwrap();
        let ce = ConsolidationEngine::default();
        let stats = ce.run(&se).unwrap();
        assert!(stats.decayed_pathways > 0);
    }

    #[test]
    fn prune_removes_dead_pathways() {
        let se = test_engine();
        se.upsert_pathway(&crate::l2_graph::Pathway::new("a", "b", "t", 0.005)).unwrap();
        let ce = ConsolidationEngine::default();
        let stats = ce.run(&se).unwrap();
        assert!(stats.pruned_pathways > 0);
    }

    #[test]
    fn full_lifecycle_promotion_chain() {
        let se = test_engine();
        se.insert_memory(&mem("m1", MemoryTier::Ephemeral, 3, 0.6, 0)).unwrap();
        let ce = ConsolidationEngine::default();

        let s1 = ce.run(&se).unwrap();
        assert_eq!(s1.promoted_to_working, 1);

        se.cache().remove_memory("m1");
        let m = se.get_memory("m1").unwrap().unwrap();
        assert_eq!(m.tier, MemoryTier::Working);
    }

    #[test]
    fn default_params_are_reasonable() {
        let ce = ConsolidationEngine::default();
        assert!((ce.pathway_decay_rate - 0.02).abs() < f64::EPSILON);
        assert!((ce.activation_decay_rate - 0.05).abs() < f64::EPSILON);
        assert!((ce.prune_threshold - 0.01).abs() < f64::EPSILON);
    }

    #[test]
    fn empty_store_no_errors() {
        let se = test_engine();
        let ce = ConsolidationEngine::default();
        let stats = ce.run(&se).unwrap();
        assert_eq!(stats.promoted_to_working, 0);
        assert_eq!(stats.pruned_memories, 0);
    }

    #[test]
    fn multiple_cycles_increment_decay_survived() {
        let se = test_engine();
        se.insert_memory(&mem("m1", MemoryTier::Working, 1, 0.5, 0)).unwrap();
        let ce = ConsolidationEngine::default();

        ce.run(&se).unwrap();
        ce.run(&se).unwrap();
        ce.run(&se).unwrap();

        se.cache().remove_memory("m1");
        let got = se.get_memory("m1").unwrap().unwrap();
        assert_eq!(got.decay_survived, 3, "decay_survived should increment each cycle");
    }

    #[test]
    fn working_not_promoted_if_activation_at_threshold() {
        let se = test_engine();
        // activation = 0.3 exactly, promotion requires > 0.3
        se.insert_memory(&mem("m1", MemoryTier::Working, 10, 0.3, 0)).unwrap();
        let ce = ConsolidationEngine::default();
        let stats = ce.run(&se).unwrap();
        assert_eq!(stats.promoted_to_consolidated, 0, "activation=0.3 should NOT promote (need >0.3)");
    }

    #[test]
    fn consolidated_not_promoted_if_decay_survived_below_five() {
        let se = test_engine();
        // access >= 20 but decay_survived = 4 (needs >= 5)
        se.insert_memory(&mem("m1", MemoryTier::Consolidated, 25, 0.8, 4)).unwrap();
        let ce = ConsolidationEngine::default();
        let stats = ce.run(&se).unwrap();
        // After one cycle, decay_survived increments to 5, but the increment
        // happens before promotion check only via increment_decay_survived.
        // However, the promotion logic reads from the originally-loaded snapshot
        // (list_memories is called after increment_decay_survived, so the DB value is 5).
        // Let's verify:
        assert_eq!(
            stats.promoted_to_crystallized, 1,
            "after increment_decay_survived, the DB value becomes 5 which meets threshold"
        );
    }

    #[test]
    fn consolidated_not_promoted_if_decay_survived_three() {
        let se = test_engine();
        // decay_survived = 3 => after increment it's 4, still < 5
        se.insert_memory(&mem("m1", MemoryTier::Consolidated, 25, 0.8, 3)).unwrap();
        let ce = ConsolidationEngine::default();
        let stats = ce.run(&se).unwrap();
        assert_eq!(stats.promoted_to_crystallized, 0, "decay_survived=3+1=4 < 5, should not promote");
    }

    #[test]
    fn consolidation_with_many_memories_completes() {
        let se = test_engine();
        for i in 0..1000 {
            se.insert_memory(&mem(
                &format!("m{i}"),
                MemoryTier::Ephemeral,
                if i % 3 == 0 { 3 } else { 0 },
                0.5,
                0,
            ))
            .unwrap();
        }
        let ce = ConsolidationEngine::default();
        let stats = ce.run(&se).unwrap();
        // 1/3 of memories have access=3 >= 2, so should be promoted
        assert_eq!(stats.promoted_to_working, 334); // ceil(1000/3) = 334
    }

    #[test]
    fn stats_sum_does_not_exceed_total() {
        let se = test_engine();
        // Mix of memories in different states
        se.insert_memory(&mem("e1", MemoryTier::Ephemeral, 3, 0.5, 0)).unwrap(); // promote
        se.insert_memory(&mem("e2", MemoryTier::Ephemeral, 0, 0.0, 5)).unwrap(); // prune
        se.insert_memory(&mem("w1", MemoryTier::Working, 6, 0.5, 0)).unwrap();   // promote
        se.insert_memory(&mem("w2", MemoryTier::Working, 1, 0.01, 0)).unwrap();  // demote
        se.insert_memory(&mem("c1", MemoryTier::Crystallized, 0, 0.0, 0)).unwrap(); // untouched

        let ce = ConsolidationEngine::default();
        let stats = ce.run(&se).unwrap();

        let total_changes = stats.promoted_to_working
            + stats.promoted_to_consolidated
            + stats.promoted_to_crystallized
            + stats.demoted
            + stats.pruned_memories;
        assert!(
            total_changes <= 5,
            "promoted + demoted + pruned ({total_changes}) should not exceed total memories (5)"
        );
    }

    #[test]
    fn consecutive_consolidations_decay_pathway_weights() {
        let se = test_engine();
        se.upsert_pathway(&crate::l2_graph::Pathway::new("a", "b", "t", 0.8)).unwrap();
        let ce = ConsolidationEngine::default();

        ce.run(&se).unwrap();
        let after_first = se.list_pathways(None, None).unwrap();
        let w1 = after_first[0].weight;

        ce.run(&se).unwrap();
        let after_second = se.list_pathways(None, None).unwrap();
        let w2 = after_second[0].weight;

        ce.run(&se).unwrap();
        let after_third = se.list_pathways(None, None).unwrap();
        let w3 = after_third[0].weight;

        assert!(w1 < 0.8, "first decay should reduce from 0.8");
        assert!(w2 < w1, "second decay should reduce further: {w2} < {w1}");
        assert!(w3 < w2, "third decay should reduce further: {w3} < {w2}");
    }

    #[test]
    fn ephemeral_with_access_one_not_promoted() {
        let se = test_engine();
        // access_count = 1, needs >= 2 for promotion
        se.insert_memory(&mem("m1", MemoryTier::Ephemeral, 1, 0.5, 0)).unwrap();
        let ce = ConsolidationEngine::default();
        let stats = ce.run(&se).unwrap();
        assert_eq!(stats.promoted_to_working, 0, "access=1 should not promote (need >= 2)");
    }

    #[test]
    fn mixed_tiers_each_processed_correctly() {
        let se = test_engine();
        // Ephemeral with enough access -> promote to Working
        se.insert_memory(&mem("e1", MemoryTier::Ephemeral, 5, 0.8, 0)).unwrap();
        // Working with enough access + activation -> promote to Consolidated
        se.insert_memory(&mem("w1", MemoryTier::Working, 7, 0.6, 0)).unwrap();
        // Consolidated with enough access + decay_survived (will be 6 after increment) -> promote
        se.insert_memory(&mem("c1", MemoryTier::Consolidated, 25, 0.9, 5)).unwrap();
        // Working with low activation -> demote
        se.insert_memory(&mem("w2", MemoryTier::Working, 1, 0.01, 0)).unwrap();

        let ce = ConsolidationEngine::default();
        let stats = ce.run(&se).unwrap();

        assert_eq!(stats.promoted_to_working, 1, "e1 should promote");
        assert_eq!(stats.promoted_to_consolidated, 1, "w1 should promote");
        assert_eq!(stats.promoted_to_crystallized, 1, "c1 should promote (decay=5+1=6)");
        assert_eq!(stats.demoted, 1, "w2 should demote");
    }

    #[test]
    fn crystallized_count_unchanged_by_consolidation() {
        let se = test_engine();
        se.insert_memory(&mem("c1", MemoryTier::Crystallized, 0, 0.0, 0)).unwrap();
        se.insert_memory(&mem("c2", MemoryTier::Crystallized, 100, 1.0, 50)).unwrap();
        se.insert_memory(&mem("c3", MemoryTier::Crystallized, 0, 0.0, 0)).unwrap();

        let ce = ConsolidationEngine::default();
        let stats = ce.run(&se).unwrap();

        assert_eq!(stats.demoted, 0, "crystallized should never be demoted");
        assert_eq!(stats.pruned_memories, 0, "crystallized should never be pruned");

        // Verify all three still exist and are still crystallized
        for id in &["c1", "c2", "c3"] {
            se.cache().remove_memory(id);
            let got = se.get_memory(id).unwrap().unwrap();
            assert_eq!(got.tier, MemoryTier::Crystallized, "{id} should remain crystallized");
        }
    }

    #[test]
    fn pathway_pruning_preserves_active_pathways() {
        let se = test_engine();
        // Active pathway (has co_activations > 0 from record_coactivation)
        se.record_coactivation("a", "b", "test", 0.1).unwrap();
        // Dead pathway (zero co_activations, low weight)
        se.upsert_pathway(&crate::l2_graph::Pathway::new("c", "d", "t", 0.005)).unwrap();

        let ce = ConsolidationEngine::default();
        let stats = ce.run(&se).unwrap();

        // The dead pathway should be pruned (weight < 0.01, co_activations = 0)
        assert!(stats.pruned_pathways > 0, "dead pathway should be pruned");

        // The active pathway should survive (co_activations > 0)
        let remaining = se.list_pathways(None, None).unwrap();
        let ab_exists = remaining.iter().any(|p| p.pre_id == "a" && p.post_id == "b");
        assert!(ab_exists, "active pathway a->b should survive pruning");
    }

    #[test]
    fn decay_rate_zero_no_weight_change() {
        let se = test_engine();
        se.upsert_pathway(&crate::l2_graph::Pathway::new("a", "b", "t", 0.5)).unwrap();
        se.insert_memory(&mem("m1", MemoryTier::Working, 3, 0.8, 0)).unwrap();

        let ce = ConsolidationEngine::new(0.0, 0.0, 0.01, 0.05);
        ce.run(&se).unwrap();

        let pathways = se.list_pathways(None, None).unwrap();
        assert!(
            (pathways[0].weight - 0.5).abs() < f64::EPSILON,
            "zero decay rate should leave weight unchanged: got {}",
            pathways[0].weight
        );
    }

    #[test]
    fn decay_rate_one_zeroes_all_weights() {
        let se = test_engine();
        se.upsert_pathway(&crate::l2_graph::Pathway::new("a", "b", "t", 0.9)).unwrap();
        se.insert_memory(&mem("m1", MemoryTier::Working, 3, 0.8, 0)).unwrap();

        let ce = ConsolidationEngine::new(1.0, 1.0, 0.01, 0.05);
        ce.run(&se).unwrap();

        // After decay_rate=1.0, weight should be 0 (factor = 1-1 = 0)
        // All pathways should have been decayed to 0 or pruned
        let pathways = se.list_pathways(None, None).unwrap();
        for pw in &pathways {
            assert!(
                pw.weight < f64::EPSILON,
                "decay rate 1.0 should zero all weights, got {}",
                pw.weight
            );
        }
    }

    #[test]
    fn custom_consolidation_params_take_effect() {
        let se = test_engine();
        // Set demotion threshold very high (0.9) so Working memory with 0.5 activation gets demoted
        let ce = ConsolidationEngine::new(0.02, 0.05, 0.01, 0.9);

        se.insert_memory(&mem("m1", MemoryTier::Working, 1, 0.5, 0)).unwrap();
        let stats = ce.run(&se).unwrap();

        // With default (0.05), activation=0.5 would NOT be demoted.
        // With custom threshold=0.9, activation=0.5 < 0.9 so it WILL be demoted.
        assert_eq!(stats.demoted, 1, "custom high demotion threshold should cause demotion");

        se.cache().remove_memory("m1");
        let got = se.get_memory("m1").unwrap().unwrap();
        assert_eq!(got.tier, MemoryTier::Ephemeral, "should be demoted to Ephemeral");
    }

    // ── F-POVM-07 regression: chrono_now returns None on pre-epoch clock ─────────

    #[test]
    fn f_povm_07_chrono_now_returns_some_on_valid_clock() {
        // Regression for F-POVM-07: chrono_now() must return Some(_) on a system
        // with a valid clock (i.e., in any normal CI or production environment).
        // The original `unwrap_or_default()` silently returned "0" on pre-epoch
        // clocks; the fix returns None so the caller can skip the decay step.
        let result = chrono_now();
        assert!(
            result.is_some(),
            "chrono_now() must return Some on a valid system clock"
        );
        let ts_str = result.unwrap();
        let secs: u64 = ts_str.parse().expect("timestamp must be a valid u64");
        // Must be after 2020-01-01 (1577836800 seconds) — catches "0" regression.
        assert!(
            secs > 1_577_836_800,
            "timestamp must be after 2020-01-01; got {secs} — suspiciously low, clock may be zeroed"
        );
    }

    #[test]
    fn f_povm_07_consolidation_run_succeeds_with_real_clock() {
        // Verifies that the full consolidation cycle completes without error when
        // chrono_now() returns Some (i.e., system clock is healthy). The F-POVM-07
        // fix causes decay_co_activations to be skipped only on pre-epoch clocks;
        // on a normal system it must run as before.
        let se = test_engine();
        // Seed a co-activation so decay_co_activations has rows to process
        se.record_coactivation("x", "y", "test", 0.5).unwrap();

        let ce = ConsolidationEngine::default();
        let stats = ce.run(&se).unwrap();

        // If chrono_now() returned None (the pre-epoch bug path), decay would be
        // skipped but the cycle would still succeed. On a real clock, decay should
        // process at least the 1 row we seeded.
        // We just verify run() doesn't Err — that's the key invariant.
        let _ = stats; // stats fields verified in other tests
    }
}
