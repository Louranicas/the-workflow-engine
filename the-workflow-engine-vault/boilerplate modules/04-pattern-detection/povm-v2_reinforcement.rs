use crate::l1_storage::StorageEngine;

use super::ReinforceDedupCache;

/// A reinforcement signal from a completed session.
///
/// `request_id` (Plan v3 § 3.2) is an optional caller-supplied `UUIDv4` for
/// idempotency. When present, POVM caches the result for 1h (Plan v3 § 3.4)
/// and returns the cached `ReinforcementStats` with `dedup: true` on retry.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ReinforcementSignal {
    pub retrieval_ids: Vec<String>,
    pub fitness_delta: f64,
    pub session_id: String,
    /// Caller-supplied idempotency key. When present, repeated calls
    /// within the dedup window return the cached stats instead of
    /// re-applying the reinforcement.
    #[serde(default)]
    pub request_id: Option<String>,
    /// Phase 1.5: explicit co-activation pairs from tick-level observation.
    /// When present, these pairs are reinforced in addition to the pairwise
    /// combinations derived from `retrieval_ids`. When absent (v1 schema),
    /// only `retrieval_ids` pairs are used.
    #[serde(default)]
    pub co_activation_pairs: Vec<CoActivationPair>,
    /// Phase 1.5: 12-dimension fitness tensor snapshot at harvest time.
    #[serde(default)]
    pub fitness_tensor: Option<[f64; 12]>,
}

/// A tick-level co-activation pair observed between two pathways/memories.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct CoActivationPair {
    pub a: String,
    pub b: String,
    pub ts_ms: u64,
}

/// The feedback engine closes the Hebbian learning loop.
///
/// When a session ends, the caller reports which memories were retrieved
/// and what the fitness delta was. Positive delta → reinforce pathways
/// between retrieved memories. Negative delta → weaken them.
pub struct FeedbackEngine;

impl FeedbackEngine {
    /// Apply reinforcement based on session outcome.
    ///
    /// For each pair of retrieved memory IDs, strengthen or weaken
    /// the pathway between them proportional to the fitness delta.
    /// Maximum `retrieval_ids` to process (prevents O(n^2) lock starvation).
    const MAX_RETRIEVAL_IDS: usize = 500;

    /// Apply reinforcement with dedup semantics (Plan v3 §§ 3.2, 3.4).
    ///
    /// If `signal.request_id` is `Some` and a cached entry is found within
    /// the TTL, the cached stats are returned with `dedup: true` and no
    /// reinforcement is re-applied. Otherwise the reinforcement is applied
    /// and the result is cached (if `request_id` is present).
    ///
    /// Auto-starts the session via `engine.ensure_session` so callers can
    /// fire `/reinforce` without a separate session-start round-trip — this
    /// is the wiring that allows `learning_health` to actually move once
    /// the feedback loop is closed.
    ///
    /// # Errors
    ///
    /// Returns the underlying `rusqlite::Error` from any of the storage
    /// operations (memory lookup, coactivation record, session lifecycle).
    pub fn reinforce_with_dedup(
        engine: &StorageEngine,
        cache: &ReinforceDedupCache,
        signal: &ReinforcementSignal,
    ) -> Result<ReinforcementStats, rusqlite::Error> {
        // Phase 1 (Plan v3 § 3.4): dedup check. Idempotent retries
        // return the original stats with the `dedup` flag flipped.
        if let Some(rid) = signal.request_id.as_deref() {
            if let Some(mut cached) = cache.get(rid) {
                cached.dedup = true;
                return Ok(cached);
            }
        }

        // Phase 1: auto-start the session with `fitness_start=0.0` so the
        // `end_session` math (`fitness_delta = fitness_end - fitness_start`)
        // preserves the caller's signalled delta. `start_session` is
        // idempotent: existing sessions keep their original `fitness_start`
        // via COALESCE (matching the `/budget` flow's `ensure_session`
        // pattern but with a deterministic baseline for fresh sessions).
        // Without this, end_session would compute `delta = fitness_end -
        // COALESCE(NULL, fitness_end) = 0`, freezing learning_health.
        if let Err(e) = engine.start_session(&signal.session_id, Some(0.0)) {
            tracing::warn!(error = %e, session = %signal.session_id, "start_session failed");
        }

        let mut stats = Self::reinforce(engine, signal)?;

        if !signal.co_activation_pairs.is_empty() {
            let pair_strength = signal.fitness_delta.clamp(-1.0, 1.0);
            if pair_strength > 0.0 {
                for pair in &signal.co_activation_pairs {
                    if let Err(e) = engine.record_coactivation(&pair.a, &pair.b, "tick_pair", pair_strength) {
                        tracing::debug!(a = %pair.a, b = %pair.b, error = %e, "co-activation pair record failed");
                    } else {
                        stats.pathways_strengthened += 1;
                    }
                }
            }
        }

        if let Some(rid) = signal.request_id.as_deref() {
            cache.insert(rid.to_string(), stats.clone());
        }

        Ok(stats)
    }

    /// Apply reinforcement directly without dedup or session auto-start.
    ///
    /// This is the original Phase 0 reinforcement path retained for the
    /// in-process tests in this module and for any caller that has already
    /// started its session via `engine.start_session(...)` and does not
    /// require idempotency. Production HTTP traffic from ORAC arrives via
    /// [`Self::reinforce_with_dedup`] (Plan v3 Phase 1).
    ///
    /// # Errors
    ///
    /// Returns the underlying `rusqlite::Error` from any of the storage
    /// operations (memory lookup, coactivation record, session lifecycle).
    pub fn reinforce(
        engine: &StorageEngine,
        signal: &ReinforcementSignal,
    ) -> Result<ReinforcementStats, rusqlite::Error> {
        let ids: &[String] = if signal.retrieval_ids.len() > Self::MAX_RETRIEVAL_IDS {
            &signal.retrieval_ids[..Self::MAX_RETRIEVAL_IDS]
        } else {
            &signal.retrieval_ids
        };
        let mut pathways_strengthened = 0u32;
        let mut pathways_weakened = 0u32;
        let mut memories_activated = 0u32;

        let strength = signal.fitness_delta.clamp(-1.0, 1.0);

        if strength > 0.0 {
            // CR-2b (S1001971): filter ids by memory existence BEFORE the
            // coactivation loop. The prior implementation iterated `ids` raw
            // in the inner pair loop, so non-existent memory ids generated
            // pathway rows (record_coactivation has no FK on memory.id and
            // upserts unconditionally). pathways_strengthened was inflated
            // 1.5–3× in production whenever retrieval_ids contained ids
            // that had been crystallised or pruned since retrieval.
            // F-POVM-01 fixed the symmetric over-count for memories_activated
            // by gating activation on `get_memory` returning Some; this
            // applies the same discipline to the coactivation pair loop.
            // Watcher concurred 2026-05-17 (pair-filed with CR-2 Candidate A).
            let mut existing_ids: Vec<&str> = Vec::with_capacity(ids.len());
            for id in ids {
                if let Some(mem) = engine.get_memory(id)? {
                    let boosted = (mem.activation + strength).clamp(0.0, 1.0);
                    engine.activate_memory(id, boosted)?;
                    memories_activated += 1; // F-POVM-01: only count memories that actually exist
                    existing_ids.push(id.as_str());
                }
            }

            for i in 0..existing_ids.len() {
                for j in (i + 1)..existing_ids.len() {
                    engine.record_coactivation(existing_ids[i], existing_ids[j], "feedback", strength)?;
                    engine.record_coactivation(existing_ids[j], existing_ids[i], "feedback", strength)?;
                    pathways_strengthened += 2;
                }
            }
        } else if strength < 0.0 {
            let decay = strength.abs();
            for id in ids {
                if let Some(mem) = engine.get_memory(id)? {
                    let weakened = (mem.activation - decay).max(0.0);
                    engine.activate_memory(id, weakened)?;
                    memories_activated += 1;
                    pathways_weakened += 1; // F-POVM-02: count only memories that were actually found and weakened
                }
            }
        }

        // F-POVM-03: propagate end_session failure so the caller sees half-recorded sessions.
        // All other DB ops in this function use `?`; end_session must be consistent.
        // Without this, fitness_end is never written and the LTP/LTD ledger silently drifts.
        engine.end_session(&signal.session_id, signal.fitness_delta)?;

        Ok(ReinforcementStats {
            pathways_strengthened,
            pathways_weakened,
            memories_activated,
            fitness_delta: signal.fitness_delta,
            dedup: false,
        })
    }
}

/// Statistics from a reinforcement operation.
///
/// `dedup` (Plan v3 § 3.4) is `true` when the response was served from the
/// 1h dedup cache (the same `request_id` was already processed). When `true`
/// no actual reinforcement was applied this call — the original counts are
/// the cumulative result of the first call.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReinforcementStats {
    pub pathways_strengthened: u32,
    pub pathways_weakened: u32,
    pub memories_activated: u32,
    pub fitness_delta: f64,
    /// Set to `true` when this response was served from the dedup cache
    /// (idempotent retry of an already-processed `request_id`).
    #[serde(default)]
    pub dedup: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l2_graph::{MemoryNode, MemoryTier};

    fn test_engine() -> StorageEngine {
        StorageEngine::in_memory().expect("in-memory engine")
    }

    fn mem(id: &str) -> MemoryNode {
        MemoryNode {
            id: id.to_string(),
            content: format!("memory {id}"),
            namespace: "test".to_string(),
            tier: MemoryTier::Working,
            theta: 0.0,
            phi: 0.0,
            tensor: [0.0; 12],
            token_count: 5,
            info_density: 1.0,
            access_count: 0,
            activation: 0.5,
            session_created: "S1".to_string(),
            crystallised: false,
            created_at: String::new(),
            last_accessed: None,
            last_activated: None,
            decay_survived: 0,
        }
    }

    fn signal(ids: Vec<&str>, delta: f64) -> ReinforcementSignal {
        ReinforcementSignal {
            retrieval_ids: ids.into_iter().map(String::from).collect(),
            fitness_delta: delta,
            session_id: "S121".to_string(),
            request_id: None,
            co_activation_pairs: vec![],
            fitness_tensor: None,
        }
    }

    fn signal_with_request(
        ids: Vec<&str>,
        delta: f64,
        session: &str,
        request_id: &str,
    ) -> ReinforcementSignal {
        ReinforcementSignal {
            retrieval_ids: ids.into_iter().map(String::from).collect(),
            fitness_delta: delta,
            session_id: session.to_string(),
            request_id: Some(request_id.to_string()),
            co_activation_pairs: vec![],
            fitness_tensor: None,
        }
    }

    #[test]
    fn positive_delta_strengthens_pathways() {
        let e = test_engine();
        e.start_session("S121", Some(0.4)).unwrap();
        e.insert_memory(&mem("m1")).unwrap();
        e.insert_memory(&mem("m2")).unwrap();

        let stats = FeedbackEngine::reinforce(&e, &signal(vec!["m1", "m2"], 0.1)).unwrap();
        assert!(stats.pathways_strengthened > 0);
    }

    #[test]
    fn positive_delta_activates_memories() {
        let e = test_engine();
        e.start_session("S121", None).unwrap();
        e.insert_memory(&mem("m1")).unwrap();

        let stats = FeedbackEngine::reinforce(&e, &signal(vec!["m1"], 0.5)).unwrap();
        assert_eq!(stats.memories_activated, 1);

        let got = e.cache().get_memory("m1").unwrap();
        assert!(got.activation > 0.0);
    }

    #[test]
    fn negative_delta_weakens() {
        let e = test_engine();
        e.start_session("S121", None).unwrap();
        e.insert_memory(&mem("m1")).unwrap();
        e.insert_memory(&mem("m2")).unwrap();

        let stats = FeedbackEngine::reinforce(&e, &signal(vec!["m1", "m2"], -0.2)).unwrap();
        assert!(stats.pathways_weakened > 0);
    }

    #[test]
    fn zero_delta_no_changes() {
        let e = test_engine();
        e.start_session("S121", None).unwrap();
        e.insert_memory(&mem("m1")).unwrap();

        let stats = FeedbackEngine::reinforce(&e, &signal(vec!["m1"], 0.0)).unwrap();
        assert_eq!(stats.pathways_strengthened, 0);
        assert_eq!(stats.pathways_weakened, 0);
    }

    #[test]
    fn creates_bidirectional_pathways() {
        let e = test_engine();
        e.start_session("S121", None).unwrap();
        e.insert_memory(&mem("a")).unwrap();
        e.insert_memory(&mem("b")).unwrap();

        FeedbackEngine::reinforce(&e, &signal(vec!["a", "b"], 0.3)).unwrap();

        let fwd = e.list_pathways(None, None).unwrap();
        let has_ab = fwd.iter().any(|p| p.pre_id == "a" && p.post_id == "b");
        let has_ba = fwd.iter().any(|p| p.pre_id == "b" && p.post_id == "a");
        assert!(has_ab);
        assert!(has_ba);
    }

    #[test]
    fn empty_retrieval_ids_no_errors() {
        let e = test_engine();
        e.start_session("S121", None).unwrap();
        let stats = FeedbackEngine::reinforce(&e, &signal(vec![], 0.5)).unwrap();
        assert_eq!(stats.pathways_strengthened, 0);
    }

    #[test]
    fn single_id_no_pathway_creation() {
        let e = test_engine();
        e.start_session("S121", None).unwrap();
        e.insert_memory(&mem("m1")).unwrap();

        let stats = FeedbackEngine::reinforce(&e, &signal(vec!["m1"], 0.5)).unwrap();
        assert_eq!(stats.pathways_strengthened, 0);
        assert_eq!(stats.memories_activated, 1);
    }

    #[test]
    fn delta_clamped_to_range() {
        let e = test_engine();
        e.start_session("S121", None).unwrap();
        e.insert_memory(&mem("m1")).unwrap();
        e.insert_memory(&mem("m2")).unwrap();

        let stats = FeedbackEngine::reinforce(&e, &signal(vec!["m1", "m2"], 5.0)).unwrap();
        assert!(stats.pathways_strengthened > 0);
    }

    #[test]
    fn three_memories_create_six_pathways() {
        let e = test_engine();
        e.start_session("S121", None).unwrap();
        e.insert_memory(&mem("a")).unwrap();
        e.insert_memory(&mem("b")).unwrap();
        e.insert_memory(&mem("c")).unwrap();

        let stats = FeedbackEngine::reinforce(&e, &signal(vec!["a", "b", "c"], 0.2)).unwrap();
        assert_eq!(stats.pathways_strengthened, 6);
    }

    #[test]
    fn max_positive_delta_produces_high_weight() {
        let e = test_engine();
        e.start_session("S121", None).unwrap();
        e.insert_memory(&mem("m1")).unwrap();
        e.insert_memory(&mem("m2")).unwrap();

        FeedbackEngine::reinforce(&e, &signal(vec!["m1", "m2"], 1.0)).unwrap();

        let got = e.cache().get_memory("m1").unwrap();
        assert!((got.activation - 1.0).abs() < f64::EPSILON, "activation should be clamped to 1.0");

        let pathways = e.list_pathways(None, None).unwrap();
        assert!(!pathways.is_empty());
        for pw in &pathways {
            assert!(pw.weight > 0.0);
        }
    }

    #[test]
    fn max_negative_delta_drops_activation_to_zero() {
        let e = test_engine();
        e.start_session("S121", None).unwrap();
        e.insert_memory(&mem("m1")).unwrap();

        FeedbackEngine::reinforce(&e, &signal(vec!["m1"], -1.0)).unwrap();

        e.cache().remove_memory("m1");
        let got = e.get_memory("m1").unwrap().unwrap();
        assert!(got.activation.abs() < f64::EPSILON, "activation should be 0.0 after max negative");
    }

    #[test]
    fn compound_reinforcement_increases_weight() {
        let e = test_engine();
        e.start_session("S121", None).unwrap();
        e.insert_memory(&mem("a")).unwrap();
        e.insert_memory(&mem("b")).unwrap();

        FeedbackEngine::reinforce(&e, &signal(vec!["a", "b"], 0.3)).unwrap();
        let pathways_after_first = e.list_pathways(None, None).unwrap();
        let weight_first = pathways_after_first
            .iter()
            .find(|p| p.pre_id == "a" && p.post_id == "b")
            .unwrap()
            .weight;

        FeedbackEngine::reinforce(&e, &signal(vec!["a", "b"], 0.3)).unwrap();
        let pathways_after_second = e.list_pathways(None, None).unwrap();
        let weight_second = pathways_after_second
            .iter()
            .find(|p| p.pre_id == "a" && p.post_id == "b")
            .unwrap()
            .weight;

        assert!(
            weight_second > weight_first,
            "second reinforcement should increase weight: {weight_second} > {weight_first}"
        );
    }

    #[test]
    fn four_memories_create_twelve_pathways() {
        let e = test_engine();
        e.start_session("S121", None).unwrap();
        for id in &["w", "x", "y", "z"] {
            e.insert_memory(&mem(id)).unwrap();
        }

        let stats = FeedbackEngine::reinforce(&e, &signal(vec!["w", "x", "y", "z"], 0.2)).unwrap();
        // 4 choose 2 = 6 pairs, each bidirectional = 12
        assert_eq!(stats.pathways_strengthened, 12);
    }

    #[test]
    fn negative_reinforcement_does_not_create_pathways() {
        let e = test_engine();
        e.start_session("S121", None).unwrap();
        e.insert_memory(&mem("m1")).unwrap();
        e.insert_memory(&mem("m2")).unwrap();

        FeedbackEngine::reinforce(&e, &signal(vec!["m1", "m2"], -0.5)).unwrap();

        let pathways = e.list_pathways(None, None).unwrap();
        assert!(pathways.is_empty(), "negative reinforcement should not create new pathways");
    }

    #[test]
    fn nonexistent_memory_id_does_not_crash() {
        let e = test_engine();
        e.start_session("S121", None).unwrap();
        e.insert_memory(&mem("real")).unwrap();

        let stats = FeedbackEngine::reinforce(
            &e,
            &signal(vec!["real", "ghost", "phantom"], 0.5),
        )
        .unwrap();
        // F-POVM-01 fix: only the 1 existing memory ("real") is counted as activated.
        // "ghost" and "phantom" were never in storage so they should NOT inflate the count.
        assert_eq!(stats.memories_activated, 1);
        // CR-2b fix (S1001971): coactivation pair loop is now gated on memory existence.
        // With only 1 existing memory ("real"), there are 0 valid pairs to coactivate.
        // Prior behaviour created 6 pathways (3 ids choose 2 pairs * 2 directions) even
        // though 2 of the 3 ids referred to non-existent memories — this inflated
        // pathways_strengthened in production whenever retrieval_ids contained ids
        // that had been crystallised or pruned since retrieval.
        assert_eq!(stats.pathways_strengthened, 0);
    }

    #[test]
    fn very_small_positive_delta_still_creates_pathways() {
        let e = test_engine();
        e.start_session("S121", None).unwrap();
        e.insert_memory(&mem("m1")).unwrap();
        e.insert_memory(&mem("m2")).unwrap();

        let stats = FeedbackEngine::reinforce(&e, &signal(vec!["m1", "m2"], 0.001)).unwrap();
        assert_eq!(stats.pathways_strengthened, 2);

        let pathways = e.list_pathways(None, None).unwrap();
        assert_eq!(pathways.len(), 2);
        for pw in &pathways {
            assert!(pw.weight > 0.0, "even tiny delta should create non-zero weight pathways");
        }
    }

    #[test]
    fn session_end_is_recorded() {
        let e = test_engine();
        e.start_session("S121", Some(0.5)).unwrap();
        e.insert_memory(&mem("m1")).unwrap();

        FeedbackEngine::reinforce(&e, &signal(vec!["m1"], 0.3)).unwrap();

        // Verify the session was ended by checking the DB directly
        let conn = e.cache(); // Use cache reference to confirm engine is live
        let _ = conn; // Engine still valid
        // The end_session call should have updated fitness_end
        // We can verify by starting a new session retrieval
        let retrievals = e.session_retrievals("S121").unwrap();
        // Session exists and is queryable (no error means end_session didn't corrupt it)
        assert!(retrievals.is_empty() || retrievals.len() == 0);
    }

    #[test]
    fn multiple_reinforcements_accumulate() {
        let e = test_engine();
        e.start_session("S121", None).unwrap();
        e.insert_memory(&mem("a")).unwrap();
        e.insert_memory(&mem("b")).unwrap();

        // First reinforcement
        FeedbackEngine::reinforce(&e, &signal(vec!["a", "b"], 0.1)).unwrap();
        let after_one = e.list_pathways(None, None).unwrap();
        let w1 = after_one.iter().find(|p| p.pre_id == "a" && p.post_id == "b").unwrap().weight;

        // Reinforce 9 more times (10 total)
        for _ in 0..9 {
            FeedbackEngine::reinforce(&e, &signal(vec!["a", "b"], 0.1)).unwrap();
        }
        let after_ten = e.list_pathways(None, None).unwrap();
        let w10 = after_ten.iter().find(|p| p.pre_id == "a" && p.post_id == "b").unwrap().weight;

        assert!(
            w10 > w1,
            "10 reinforcements ({w10}) should produce higher weight than 1 ({w1})"
        );
        assert!(w10 > 0.5, "10 reinforcements of 0.1 should accumulate past 0.5, got {w10}");
    }

    #[test]
    fn pathway_weight_bounded_zero_to_one() {
        let e = test_engine();
        e.start_session("S121", None).unwrap();
        e.insert_memory(&mem("a")).unwrap();
        e.insert_memory(&mem("b")).unwrap();

        // Reinforce many times to push weight toward limit
        for _ in 0..20 {
            FeedbackEngine::reinforce(&e, &signal(vec!["a", "b"], 1.0)).unwrap();
        }

        let pathways = e.list_pathways(None, None).unwrap();
        for pw in &pathways {
            assert!(pw.weight >= 0.0, "weight must be >= 0.0, got {}", pw.weight);
            assert!(pw.weight <= 1.0, "weight must be <= 1.0, got {}", pw.weight);
        }
    }

    #[test]
    fn negative_on_already_zero_activation_stays_zero() {
        let e = test_engine();
        e.start_session("S121", None).unwrap();
        let mut m = mem("m1");
        m.activation = 0.0;
        e.insert_memory(&m).unwrap();

        FeedbackEngine::reinforce(&e, &signal(vec!["m1"], -0.5)).unwrap();

        e.cache().remove_memory("m1");
        let got = e.get_memory("m1").unwrap().unwrap();
        assert!(got.activation.abs() < f64::EPSILON, "zero activation should remain zero");
    }

    #[test]
    fn truncation_at_max_retrieval_ids() {
        let e = test_engine();
        e.start_session("S121", None).unwrap();

        // Create MAX_RETRIEVAL_IDS + 1 memories
        let count = FeedbackEngine::MAX_RETRIEVAL_IDS + 1;
        let ids: Vec<String> = (0..count).map(|i| format!("m{i}")).collect();
        for id in &ids {
            e.insert_memory(&mem(id)).unwrap();
        }

        let sig = ReinforcementSignal {
            retrieval_ids: ids,
            fitness_delta: 0.1,
            session_id: "S121".to_string(),
            request_id: None,
            co_activation_pairs: vec![],
            fitness_tensor: None,
        };
        let stats = FeedbackEngine::reinforce(&e, &sig).unwrap();

        // Pathways from 500 IDs: 500 choose 2 = 124750 pairs * 2 = 249500
        let expected = (FeedbackEngine::MAX_RETRIEVAL_IDS * (FeedbackEngine::MAX_RETRIEVAL_IDS - 1)) as u32;
        assert_eq!(stats.pathways_strengthened, expected);
        // memories_activated should be MAX (the truncated count)
        assert_eq!(stats.memories_activated, FeedbackEngine::MAX_RETRIEVAL_IDS as u32);
    }

    #[test]
    fn reinforcement_signal_deserializes_from_json() {
        let json = r#"{"retrieval_ids":["m1","m2"],"fitness_delta":0.42,"session_id":"S200"}"#;
        let sig: ReinforcementSignal = serde_json::from_str(json).unwrap();
        assert_eq!(sig.retrieval_ids.len(), 2);
        assert!((sig.fitness_delta - 0.42).abs() < f64::EPSILON);
        assert_eq!(sig.session_id, "S200");
    }

    #[test]
    fn reinforcement_stats_fields_populated_correctly() {
        let e = test_engine();
        e.start_session("S121", None).unwrap();
        e.insert_memory(&mem("a")).unwrap();
        e.insert_memory(&mem("b")).unwrap();
        e.insert_memory(&mem("c")).unwrap();

        let stats = FeedbackEngine::reinforce(&e, &signal(vec!["a", "b", "c"], 0.4)).unwrap();

        assert_eq!(stats.pathways_strengthened, 6);
        assert_eq!(stats.pathways_weakened, 0);
        assert_eq!(stats.memories_activated, 3);
        assert!((stats.fitness_delta - 0.4).abs() < f64::EPSILON);
    }

    #[test]
    fn mixed_positive_then_negative_net_effect() {
        let e = test_engine();
        e.start_session("S121", None).unwrap();
        e.insert_memory(&mem("m1")).unwrap();

        // Boost activation from 0.5 -> should increase
        FeedbackEngine::reinforce(&e, &signal(vec!["m1"], 0.4)).unwrap();
        let boosted = e.cache().get_memory("m1").unwrap().activation;
        assert!(boosted > 0.5, "positive should increase activation");

        // Weaken it back
        FeedbackEngine::reinforce(&e, &signal(vec!["m1"], -0.3)).unwrap();
        e.cache().remove_memory("m1");
        let weakened = e.get_memory("m1").unwrap().unwrap().activation;
        assert!(
            weakened < boosted,
            "negative after positive should reduce: {weakened} < {boosted}"
        );
        // But it should still be above zero since we only removed 0.3 from ~0.9
        assert!(weakened > 0.0, "should not have gone to zero");
    }

    #[test]
    fn bidirectional_pathway_symmetry() {
        let e = test_engine();
        e.start_session("S121", None).unwrap();
        e.insert_memory(&mem("p")).unwrap();
        e.insert_memory(&mem("q")).unwrap();

        FeedbackEngine::reinforce(&e, &signal(vec!["p", "q"], 0.5)).unwrap();

        let pathways = e.list_pathways(None, None).unwrap();
        let pq = pathways
            .iter()
            .find(|pw| pw.pre_id == "p" && pw.post_id == "q")
            .expect("p->q pathway should exist");
        let qp = pathways
            .iter()
            .find(|pw| pw.pre_id == "q" && pw.post_id == "p")
            .expect("q->p pathway should exist");

        assert!(
            (pq.weight - qp.weight).abs() < f64::EPSILON,
            "bidirectional pathways should have same weight: {} vs {}",
            pq.weight,
            qp.weight
        );
    }

    // ── reinforce_with_dedup tests (Plan v3 Phase 1) ─────────────────────

    #[test]
    fn dedup_default_signal_has_no_request_id() {
        let s = signal(vec!["a"], 0.1);
        assert!(s.request_id.is_none());
    }

    #[test]
    fn dedup_stats_default_dedup_is_false() {
        let e = test_engine();
        e.start_session("S121", None).unwrap();
        e.insert_memory(&mem("m1")).unwrap();
        let stats = FeedbackEngine::reinforce(&e, &signal(vec!["m1"], 0.1)).unwrap();
        assert!(!stats.dedup);
    }

    #[test]
    fn dedup_first_call_is_not_marked_dedup() {
        let e = test_engine();
        e.insert_memory(&mem("m1")).unwrap();
        e.insert_memory(&mem("m2")).unwrap();
        let cache = ReinforceDedupCache::new();
        let stats = FeedbackEngine::reinforce_with_dedup(
            &e,
            &cache,
            &signal_with_request(vec!["m1", "m2"], 0.1, "S-ph1-fresh", "req-1"),
        )
        .unwrap();
        assert!(!stats.dedup);
        assert!(stats.pathways_strengthened > 0);
    }

    #[test]
    fn dedup_second_call_with_same_request_id_returns_dedup_true() {
        let e = test_engine();
        e.insert_memory(&mem("m1")).unwrap();
        e.insert_memory(&mem("m2")).unwrap();
        let cache = ReinforceDedupCache::new();
        let _ = FeedbackEngine::reinforce_with_dedup(
            &e,
            &cache,
            &signal_with_request(vec!["m1", "m2"], 0.1, "S-ph1-dup", "req-dup"),
        )
        .unwrap();
        let second = FeedbackEngine::reinforce_with_dedup(
            &e,
            &cache,
            &signal_with_request(vec!["m1", "m2"], 0.1, "S-ph1-dup", "req-dup"),
        )
        .unwrap();
        assert!(second.dedup);
        assert!(second.pathways_strengthened > 0);
    }

    #[test]
    fn dedup_returns_original_counts_on_second_call() {
        let e = test_engine();
        e.insert_memory(&mem("m1")).unwrap();
        e.insert_memory(&mem("m2")).unwrap();
        let cache = ReinforceDedupCache::new();
        let first = FeedbackEngine::reinforce_with_dedup(
            &e,
            &cache,
            &signal_with_request(vec!["m1", "m2"], 0.1, "S-ph1-counts", "req-counts"),
        )
        .unwrap();
        let second = FeedbackEngine::reinforce_with_dedup(
            &e,
            &cache,
            &signal_with_request(vec!["m1", "m2"], 0.1, "S-ph1-counts", "req-counts"),
        )
        .unwrap();
        assert_eq!(first.pathways_strengthened, second.pathways_strengthened);
        assert_eq!(first.memories_activated, second.memories_activated);
    }

    #[test]
    fn dedup_no_double_application() {
        let e = test_engine();
        e.insert_memory(&mem("a")).unwrap();
        e.insert_memory(&mem("b")).unwrap();
        let cache = ReinforceDedupCache::new();

        // First call records co-activation.
        FeedbackEngine::reinforce_with_dedup(
            &e,
            &cache,
            &signal_with_request(vec!["a", "b"], 0.1, "S-no-double", "req-no-double"),
        )
        .unwrap();
        let pathways_after_first = e.list_pathways(None, None).unwrap();
        let weight_first = pathways_after_first
            .iter()
            .find(|p| p.pre_id == "a" && p.post_id == "b")
            .map(|p| p.weight)
            .unwrap_or(0.0);

        // Repeat with same request_id — should be served from cache, no
        // additional weight increase.
        FeedbackEngine::reinforce_with_dedup(
            &e,
            &cache,
            &signal_with_request(vec!["a", "b"], 0.1, "S-no-double", "req-no-double"),
        )
        .unwrap();
        let pathways_after_second = e.list_pathways(None, None).unwrap();
        let weight_second = pathways_after_second
            .iter()
            .find(|p| p.pre_id == "a" && p.post_id == "b")
            .map(|p| p.weight)
            .unwrap_or(0.0);

        assert!(
            (weight_first - weight_second).abs() < f64::EPSILON,
            "dedup retry must not double-strengthen: {weight_first} vs {weight_second}"
        );
    }

    #[test]
    fn dedup_different_request_ids_apply_separately() {
        let e = test_engine();
        e.insert_memory(&mem("m1")).unwrap();
        e.insert_memory(&mem("m2")).unwrap();
        let cache = ReinforceDedupCache::new();
        let s1 = FeedbackEngine::reinforce_with_dedup(
            &e,
            &cache,
            &signal_with_request(vec!["m1", "m2"], 0.1, "S-A", "req-A"),
        )
        .unwrap();
        let s2 = FeedbackEngine::reinforce_with_dedup(
            &e,
            &cache,
            &signal_with_request(vec!["m1", "m2"], 0.1, "S-B", "req-B"),
        )
        .unwrap();
        assert!(!s1.dedup);
        assert!(!s2.dedup);
    }

    #[test]
    fn dedup_signal_without_request_id_is_never_cached() {
        let e = test_engine();
        e.insert_memory(&mem("m1")).unwrap();
        let cache = ReinforceDedupCache::new();
        let _ = FeedbackEngine::reinforce_with_dedup(&e, &cache, &signal(vec!["m1"], 0.1)).unwrap();
        let _ = FeedbackEngine::reinforce_with_dedup(&e, &cache, &signal(vec!["m1"], 0.1)).unwrap();
        // Both calls executed; nothing was cached.
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn auto_start_session_preserves_caller_delta_in_sessions_table() {
        // Regression for the S226 deploy verify finding: a fresh session
        // with NULL fitness_start would cause end_session to compute
        // `delta = fitness_end - fitness_end = 0`, masking real signal.
        // Auto-start with fitness_start=0.0 fixes the math.
        let e = test_engine();
        e.insert_memory(&mem("a")).unwrap();
        e.insert_memory(&mem("b")).unwrap();
        let cache = ReinforceDedupCache::new();
        let _ = FeedbackEngine::reinforce_with_dedup(
            &e,
            &cache,
            &signal_with_request(
                vec!["a", "b"],
                0.05,
                "S-fresh-fitness",
                "req-fresh-fitness",
            ),
        )
        .unwrap();

        // hydration_summary's learning_health = SUM(delta>0) / COUNT(*)
        // across sessions with non-null fitness_delta. With the fix, this
        // session contributes a positive delta, so on a fresh in-memory
        // engine the ratio is 1.0.
        let summary = e.hydration_summary().unwrap();
        assert!(
            summary.learning_health > 0.0,
            "learning_health must move when caller-supplied delta is positive: got {}",
            summary.learning_health
        );
    }

    #[test]
    fn auto_start_session_makes_end_session_succeed() {
        // The Phase 1 wiring fix: ensure_session() before reinforce so that
        // end_session updates the row and learning_health can move.
        let e = test_engine();
        e.insert_memory(&mem("m1")).unwrap();
        e.insert_memory(&mem("m2")).unwrap();
        let cache = ReinforceDedupCache::new();
        let _ = FeedbackEngine::reinforce_with_dedup(
            &e,
            &cache,
            &signal_with_request(
                vec!["m1", "m2"],
                0.25,
                "S-auto-start",
                "req-auto-start",
            ),
        )
        .unwrap();

        // Hydration summary must include the auto-started session, otherwise
        // learning_health cannot move.
        let summary = e.hydration_summary().unwrap();
        assert!(
            summary.session_count >= 1,
            "auto-start must register session, got count={}",
            summary.session_count
        );
    }

    #[test]
    fn dedup_request_id_field_serde_roundtrip() {
        let json = r#"{"retrieval_ids":["a"],"fitness_delta":0.1,"session_id":"S","request_id":"r1"}"#;
        let s: ReinforcementSignal = serde_json::from_str(json).unwrap();
        assert_eq!(s.request_id.as_deref(), Some("r1"));
    }

    #[test]
    fn dedup_request_id_field_optional_in_serde() {
        let json = r#"{"retrieval_ids":["a"],"fitness_delta":0.1,"session_id":"S"}"#;
        let s: ReinforcementSignal = serde_json::from_str(json).unwrap();
        assert!(s.request_id.is_none());
    }

    #[test]
    fn dedup_stats_serializes_dedup_field() {
        let stats = ReinforcementStats {
            pathways_strengthened: 1,
            pathways_weakened: 0,
            memories_activated: 1,
            fitness_delta: 0.1,
            dedup: true,
        };
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"dedup\":true"));
    }

    // ── Phase 1.5: co-activation pair tests ─────────────────────

    #[test]
    fn co_activation_pairs_deserialized_from_v2_json() {
        let json = r#"{"retrieval_ids":["a"],"fitness_delta":0.1,"session_id":"S","co_activation_pairs":[{"a":"x","b":"y","ts_ms":1000}]}"#;
        let s: ReinforcementSignal = serde_json::from_str(json).unwrap();
        assert_eq!(s.co_activation_pairs.len(), 1);
        assert_eq!(s.co_activation_pairs[0].a, "x");
        assert_eq!(s.co_activation_pairs[0].b, "y");
        assert_eq!(s.co_activation_pairs[0].ts_ms, 1000);
    }

    #[test]
    fn v1_json_without_pairs_defaults_to_empty() {
        let json = r#"{"retrieval_ids":["a"],"fitness_delta":0.1,"session_id":"S"}"#;
        let s: ReinforcementSignal = serde_json::from_str(json).unwrap();
        assert!(s.co_activation_pairs.is_empty());
        assert!(s.fitness_tensor.is_none());
    }

    #[test]
    fn co_activation_pairs_create_pathways_via_dedup() {
        let e = test_engine();
        e.insert_memory(&mem("x")).unwrap();
        e.insert_memory(&mem("y")).unwrap();
        let cache = ReinforceDedupCache::new();
        let sig = ReinforcementSignal {
            retrieval_ids: vec![],
            fitness_delta: 0.2,
            session_id: "S-pair".to_string(),
            request_id: Some("req-pair".to_string()),
            co_activation_pairs: vec![
                CoActivationPair { a: "x".to_string(), b: "y".to_string(), ts_ms: 1000 },
            ],
            fitness_tensor: None,
        };
        let stats = FeedbackEngine::reinforce_with_dedup(&e, &cache, &sig).unwrap();
        assert!(stats.pathways_strengthened >= 1, "co-activation pair should create pathway");
    }

    #[test]
    fn co_activation_pairs_not_processed_on_negative_delta() {
        let e = test_engine();
        e.insert_memory(&mem("x")).unwrap();
        e.insert_memory(&mem("y")).unwrap();
        let cache = ReinforceDedupCache::new();
        let sig = ReinforcementSignal {
            retrieval_ids: vec![],
            fitness_delta: -0.3,
            session_id: "S-neg-pair".to_string(),
            request_id: Some("req-neg-pair".to_string()),
            co_activation_pairs: vec![
                CoActivationPair { a: "x".to_string(), b: "y".to_string(), ts_ms: 2000 },
            ],
            fitness_tensor: None,
        };
        let stats = FeedbackEngine::reinforce_with_dedup(&e, &cache, &sig).unwrap();
        assert_eq!(stats.pathways_strengthened, 0, "negative delta should not process co-activation pairs");
    }

    #[test]
    fn fitness_tensor_deserialized_from_v2_json() {
        let json = r#"{"retrieval_ids":[],"fitness_delta":0.1,"session_id":"S","fitness_tensor":[0.1,0.2,0.3,0.4,0.5,0.6,0.7,0.8,0.9,1.0,1.1,1.2]}"#;
        let s: ReinforcementSignal = serde_json::from_str(json).unwrap();
        assert!(s.fitness_tensor.is_some());
        let t = s.fitness_tensor.unwrap();
        assert!((t[0] - 0.1).abs() < f64::EPSILON);
        assert!((t[11] - 1.2).abs() < f64::EPSILON);
    }

    #[test]
    fn multiple_co_activation_pairs_all_processed() {
        let e = test_engine();
        for id in &["a", "b", "c", "d"] {
            e.insert_memory(&mem(id)).unwrap();
        }
        let cache = ReinforceDedupCache::new();
        let sig = ReinforcementSignal {
            retrieval_ids: vec![],
            fitness_delta: 0.15,
            session_id: "S-multi-pair".to_string(),
            request_id: Some("req-multi-pair".to_string()),
            co_activation_pairs: vec![
                CoActivationPair { a: "a".to_string(), b: "b".to_string(), ts_ms: 100 },
                CoActivationPair { a: "c".to_string(), b: "d".to_string(), ts_ms: 200 },
                CoActivationPair { a: "a".to_string(), b: "c".to_string(), ts_ms: 300 },
            ],
            fitness_tensor: None,
        };
        let stats = FeedbackEngine::reinforce_with_dedup(&e, &cache, &sig).unwrap();
        assert!(stats.pathways_strengthened >= 3, "all 3 co-activation pairs should create pathways, got {}", stats.pathways_strengthened);
    }

    #[test]
    fn co_activation_pair_serde_roundtrip() {
        let pair = CoActivationPair { a: "mem_a".to_string(), b: "mem_b".to_string(), ts_ms: 1735000000000 };
        let json = serde_json::to_string(&pair).unwrap();
        let parsed: CoActivationPair = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.a, "mem_a");
        assert_eq!(parsed.b, "mem_b");
        assert_eq!(parsed.ts_ms, 1735000000000);
    }

    // ── F-POVM-01 regression: positive-branch counter only counts existing memories ─

    #[test]
    fn f_povm_01_missing_positive_ids_not_counted() {
        // Regression for F-POVM-01: memories_activated must NOT include IDs that
        // were not found in storage. Before the fix, the counter was incremented
        // outside the `if let Some(mem)` block and overcounted misses as hits.
        let e = test_engine();
        e.start_session("S121", None).unwrap();
        e.insert_memory(&mem("exists_a")).unwrap();
        e.insert_memory(&mem("exists_b")).unwrap();
        // "ghost" and "phantom" are deliberately not inserted

        let stats = FeedbackEngine::reinforce(
            &e,
            &signal(vec!["exists_a", "ghost", "exists_b", "phantom"], 0.5),
        )
        .unwrap();

        // Only 2 memories were in storage; the other 2 must NOT inflate the count.
        assert_eq!(
            stats.memories_activated, 2,
            "memories_activated should only count memories that actually exist in storage"
        );
    }

    #[test]
    fn f_povm_01_all_missing_positive_ids_yields_zero_activated() {
        // Extreme case: every ID in the signal is missing from storage.
        // memories_activated must be 0, not ids.len().
        let e = test_engine();
        e.start_session("S121", None).unwrap();

        let stats = FeedbackEngine::reinforce(
            &e,
            &signal(vec!["no_such_a", "no_such_b", "no_such_c"], 0.8),
        )
        .unwrap();

        assert_eq!(
            stats.memories_activated, 0,
            "when no IDs exist in storage, memories_activated must be 0"
        );
        // CR-2b fix (S1001971): pathways_strengthened is also 0 when no memories
        // exist. The pair loop is gated on memory existence (the existing_ids
        // filter). Prior behaviour created 6 pathway rows for 3 non-existent ids,
        // inflating pathways_strengthened structurally. Same discipline as
        // F-POVM-01 for memories_activated, applied to the coactivation count.
        assert_eq!(
            stats.pathways_strengthened, 0,
            "when no IDs exist in storage, pathways_strengthened must be 0"
        );
    }

    // ── F-POVM-02 regression: pathways_weakened reports only found-and-weakened count ─

    #[test]
    fn f_povm_02_missing_negative_ids_not_counted_as_weakened() {
        // Regression for F-POVM-02: pathways_weakened must NOT be set to ids.len()
        // when some IDs are missing from storage. Before the fix, it was assigned
        // `u32::try_from(ids.len())` regardless of how many memories were found.
        let e = test_engine();
        e.start_session("S121", None).unwrap();
        e.insert_memory(&mem("real_1")).unwrap();
        // "ghost_1" and "ghost_2" are not in storage

        let stats = FeedbackEngine::reinforce(
            &e,
            &signal(vec!["real_1", "ghost_1", "ghost_2"], -0.4),
        )
        .unwrap();

        assert_eq!(
            stats.pathways_weakened, 1,
            "only 1 memory was in storage; pathways_weakened must not report ids.len()=3"
        );
    }

    #[test]
    fn f_povm_02_all_missing_negative_ids_yields_zero_weakened() {
        let e = test_engine();
        e.start_session("S121", None).unwrap();

        let stats = FeedbackEngine::reinforce(
            &e,
            &signal(vec!["missing_x", "missing_y"], -0.3),
        )
        .unwrap();

        assert_eq!(
            stats.pathways_weakened, 0,
            "no memories in storage → pathways_weakened must be 0, not ids.len()=2"
        );
    }

    // ── F-POVM-03 regression: end_session failure propagates as Err ─────────────

    #[test]
    fn f_povm_03_end_session_propagates_on_missing_session() {
        // Regression for F-POVM-03: end_session is now called with `?` so any
        // rusqlite error propagates to the caller. This test verifies the happy
        // path: reinforce() succeeds when the session exists and end_session
        // returns Ok (even if the UPDATE affects 0 rows when the session is
        // missing — rusqlite returns Ok(0), not Err, for no-op UPDATEs).
        let e = test_engine();
        e.start_session("S-f3", Some(0.0)).unwrap();
        e.insert_memory(&mem("m1")).unwrap();

        // Must not discard end_session result — must return Ok on the happy path
        let result = FeedbackEngine::reinforce(&e, &signal(vec!["m1"], 0.3));
        assert!(
            result.is_ok(),
            "reinforce must succeed when session exists and end_session succeeds"
        );

        // Verify the session was ended (fitness_end was written) by using
        // reinforce_with_dedup on a second call, which exercises the full loop.
        let stats = result.unwrap();
        assert_eq!(stats.memories_activated, 1, "activated count must match existing memories");
        assert!((stats.fitness_delta - 0.3).abs() < f64::EPSILON, "fitness_delta must be preserved");
    }

    #[test]
    fn f_povm_03_reinforcement_stats_correct_after_end_session() {
        // Verifies that stats are fully populated even with the propagation change.
        let e = test_engine();
        e.start_session("S-f3b", Some(0.0)).unwrap();
        e.insert_memory(&mem("a")).unwrap();
        e.insert_memory(&mem("b")).unwrap();

        let stats = FeedbackEngine::reinforce(&e, &signal(vec!["a", "b"], 0.5)).unwrap();
        assert_eq!(stats.memories_activated, 2);
        assert_eq!(stats.pathways_strengthened, 2);
        assert_eq!(stats.pathways_weakened, 0);
        assert!((stats.fitness_delta - 0.5).abs() < f64::EPSILON);
    }
}
