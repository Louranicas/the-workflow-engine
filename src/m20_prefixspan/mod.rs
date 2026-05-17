//! `m20_prefixspan_miner` — **KEYSTONE / Gap 1 NEW PRIMITIVE**.
//!
//! PrefixSpan-based gap-allowed sequential pattern mining over cascade
//! and Battern step sequences. The engine's N-step compositional
//! sub-graph detection — the structural-gap authorship called out in
//! the Boilerplate Hunt (POVM's `CoActivationPair` covers two elements
//! at a time; m20 generalises to N steps under bounded right-gap).
//!
//! **Determinism contract:** identical input → identical output. Output
//! patterns are sorted by `(support DESC, len DESC, canonical_hash ASC)`.
//! Property-test invariant; downstream m21/m23 rely on stable ordering
//! for variant generation.

use std::collections::HashMap;

use thiserror::Error;

use crate::m4_cascade::cluster_id::fnv1a_64;

/// Opaque step token (F11 cascade-monoculture: no human-readable label
/// in the inner value).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub struct StepToken(pub u32);

/// Newtype for the minimum-support floor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MinSupport(pub usize);

/// Newtype for max-gap config (right-gap bound during matching).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaxGap(pub usize);

/// **F2 hard floor** — `MinSupport` below this value is rejected outright.
pub const MIN_SUPPORT_FLOOR: usize = 2;

/// Maximum pattern length (depth cap on recursion).
pub const DEFAULT_MAX_LENGTH: usize = 8;

/// Default right-gap bound.
pub const DEFAULT_MAX_GAP: usize = 5;

/// A frequent sequential pattern emitted by [`mine_sequences`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Pattern {
    /// Ordered token sequence.
    pub steps: Vec<StepToken>,
    /// Number of input sequences containing this pattern.
    pub support: usize,
    /// `(min_left_gap, max_right_gap)` observed during matching.
    pub gap_bounds: (usize, usize),
    /// Stable hash for cross-module identity (FNV-1a 64 over
    /// `step_id:gap_min:gap_max` concatenation).
    pub canonical_hash: u64,
}

impl Pattern {
    /// Construct + compute canonical hash.
    #[must_use]
    pub fn new(steps: Vec<StepToken>, support: usize, gap_bounds: (usize, usize)) -> Self {
        use std::fmt::Write;
        let mut buf = String::new();
        for s in &steps {
            let _ = write!(buf, "{}:", s.0);
        }
        let _ = write!(buf, "{}-{}", gap_bounds.0, gap_bounds.1);
        let canonical_hash = fnv1a_64(buf.as_bytes());
        Self {
            steps,
            support,
            gap_bounds,
            canonical_hash,
        }
    }
}

/// Miner failure modes.
#[derive(Debug, Error)]
pub enum MinerError {
    /// Input sequence slice was empty.
    #[error("empty database: no sequences to mine")]
    EmptyDatabase,
    /// Pattern length exceeded the configured maximum.
    #[error("pattern too long: {len} > {max}")]
    PatternTooLong {
        /// Observed length.
        len: usize,
        /// Configured max.
        max: usize,
    },
    /// `min_support` is below [`MIN_SUPPORT_FLOOR`].
    #[error("min_support floor {0} below F2 hard minimum of 2")]
    MinSupportBelowFloor(usize),
}

/// PrefixSpan with gap-allowed matching.
///
/// Mines frequent sequential patterns from `sequences` under the given
/// support floor and right-gap bound. Output is sorted by
/// `(support DESC, len DESC, canonical_hash ASC)`.
///
/// # Errors
///
/// - [`MinerError::MinSupportBelowFloor`] if `min_support < MIN_SUPPORT_FLOOR`.
/// - [`MinerError::EmptyDatabase`] if `sequences` is empty.
pub fn mine_sequences(
    sequences: &[Vec<StepToken>],
    min_support: MinSupport,
    max_gap: MaxGap,
    max_length: usize,
) -> Result<Vec<Pattern>, MinerError> {
    if min_support.0 < MIN_SUPPORT_FLOOR {
        return Err(MinerError::MinSupportBelowFloor(min_support.0));
    }
    if sequences.is_empty() {
        return Err(MinerError::EmptyDatabase);
    }
    let max_length = max_length.max(1);

    let mut results: Vec<Pattern> = Vec::new();
    let mut frequencies: HashMap<StepToken, usize> = HashMap::new();
    for seq in sequences {
        let mut seen: std::collections::HashSet<StepToken> =
            std::collections::HashSet::new();
        for tok in seq {
            if seen.insert(*tok) {
                *frequencies.entry(*tok).or_insert(0) += 1;
            }
        }
    }
    let mut frequent_items: Vec<StepToken> = frequencies
        .iter()
        .filter(|(_, &c)| c >= min_support.0)
        .map(|(t, _)| *t)
        .collect();
    frequent_items.sort();

    for &item in &frequent_items {
        let prefix = vec![item];
        let support = frequencies[&item];
        let pattern = Pattern::new(prefix.clone(), support, (0, 0));
        results.push(pattern);
        if max_length > 1 {
            recurse_prefix(sequences, &prefix, min_support, max_gap, max_length, &mut results);
        }
    }

    results.sort_by(|a, b| {
        b.support
            .cmp(&a.support)
            .then_with(|| b.steps.len().cmp(&a.steps.len()))
            .then_with(|| a.canonical_hash.cmp(&b.canonical_hash))
    });
    Ok(results)
}

fn recurse_prefix(
    sequences: &[Vec<StepToken>],
    prefix: &[StepToken],
    min_support: MinSupport,
    max_gap: MaxGap,
    max_length: usize,
    out: &mut Vec<Pattern>,
) {
    if prefix.len() >= max_length {
        return;
    }
    // Project: for each sequence containing `prefix` under gap-allowed
    // matching, retain the suffix after the FIRST matching occurrence.
    let mut suffixes: Vec<Vec<StepToken>> = Vec::new();
    let mut max_right_gap = 0_usize;
    for seq in sequences {
        if let Some(suffix_info) = project_after_prefix(seq, prefix, max_gap) {
            max_right_gap = max_right_gap.max(suffix_info.right_gap);
            suffixes.push(suffix_info.suffix);
        }
    }
    // Count length-1 extensions in the projected suffixes.
    let mut ext_freq: HashMap<StepToken, usize> = HashMap::new();
    for suf in &suffixes {
        let mut seen: std::collections::HashSet<StepToken> =
            std::collections::HashSet::new();
        for tok in suf {
            if seen.insert(*tok) {
                *ext_freq.entry(*tok).or_insert(0) += 1;
            }
        }
    }
    let mut frequent_exts: Vec<(StepToken, usize)> = ext_freq
        .into_iter()
        .filter(|(_, c)| *c >= min_support.0)
        .collect();
    frequent_exts.sort_by_key(|(t, _)| *t);

    for (ext, support) in frequent_exts {
        let mut new_prefix = prefix.to_vec();
        new_prefix.push(ext);
        let pattern = Pattern::new(new_prefix.clone(), support, (0, max_right_gap));
        out.push(pattern);
        recurse_prefix(sequences, &new_prefix, min_support, max_gap, max_length, out);
    }
}

struct ProjectedSuffix {
    suffix: Vec<StepToken>,
    right_gap: usize,
}

/// Find the FIRST occurrence of `prefix` in `seq` under gap-allowed
/// semantics: between successive prefix tokens, at most `max_gap.0`
/// non-matching tokens may be skipped. Returns the suffix AFTER the last
/// matched prefix token, plus the maximum gap observed.
fn project_after_prefix(
    seq: &[StepToken],
    prefix: &[StepToken],
    max_gap: MaxGap,
) -> Option<ProjectedSuffix> {
    if prefix.is_empty() {
        return Some(ProjectedSuffix {
            suffix: seq.to_vec(),
            right_gap: 0,
        });
    }
    let mut p_idx = 0_usize;
    let mut last_match_idx: Option<usize> = None;
    let mut max_gap_observed = 0_usize;
    for (i, tok) in seq.iter().enumerate() {
        if *tok == prefix[p_idx] {
            if let Some(last) = last_match_idx {
                let gap = i.saturating_sub(last).saturating_sub(1);
                if gap > max_gap.0 {
                    // Gap too large; restart matching at this token.
                    if *tok == prefix[0] {
                        p_idx = 1;
                        last_match_idx = Some(i);
                        max_gap_observed = 0;
                        continue;
                    }
                    p_idx = 0;
                    last_match_idx = None;
                    max_gap_observed = 0;
                    continue;
                }
                if gap > max_gap_observed {
                    max_gap_observed = gap;
                }
            }
            last_match_idx = Some(i);
            p_idx += 1;
            if p_idx == prefix.len() {
                let after = i.saturating_add(1);
                let suffix = if after >= seq.len() {
                    Vec::new()
                } else {
                    seq[after..].to_vec()
                };
                return Some(ProjectedSuffix {
                    suffix,
                    right_gap: max_gap_observed,
                });
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{
        mine_sequences, project_after_prefix, MaxGap, MinSupport, MinerError, Pattern,
        StepToken, DEFAULT_MAX_GAP, DEFAULT_MAX_LENGTH, MIN_SUPPORT_FLOOR,
    };

    fn tok(n: u32) -> StepToken {
        StepToken(n)
    }
    fn seq(ns: &[u32]) -> Vec<StepToken> {
        ns.iter().copied().map(tok).collect()
    }

    // ---- Constants + types (4) ------------------------------------------

    #[test]
    fn min_support_floor_is_two() {
        assert_eq!(MIN_SUPPORT_FLOOR, 2);
    }

    #[test]
    fn default_max_gap_is_five() {
        assert_eq!(DEFAULT_MAX_GAP, 5);
    }

    #[test]
    fn default_max_length_is_eight() {
        assert_eq!(DEFAULT_MAX_LENGTH, 8);
    }

    #[test]
    fn step_token_is_ord_and_hash() {
        use std::collections::HashSet;
        let mut s = HashSet::new();
        s.insert(tok(1));
        s.insert(tok(1));
        s.insert(tok(2));
        assert_eq!(s.len(), 2);
    }

    // ---- Pattern canonical hash (3) -------------------------------------

    #[test]
    fn pattern_canonical_hash_deterministic() {
        let a = Pattern::new(seq(&[1, 2]), 3, (0, 0));
        let b = Pattern::new(seq(&[1, 2]), 3, (0, 0));
        assert_eq!(a.canonical_hash, b.canonical_hash);
    }

    #[test]
    fn pattern_canonical_hash_distinguishes_steps() {
        let a = Pattern::new(seq(&[1, 2]), 3, (0, 0));
        let b = Pattern::new(seq(&[1, 3]), 3, (0, 0));
        assert_ne!(a.canonical_hash, b.canonical_hash);
    }

    #[test]
    fn pattern_canonical_hash_distinguishes_gap_bounds() {
        let a = Pattern::new(seq(&[1, 2]), 3, (0, 0));
        let b = Pattern::new(seq(&[1, 2]), 3, (0, 5));
        assert_ne!(a.canonical_hash, b.canonical_hash);
    }

    // ---- Refusal modes (3) ----------------------------------------------

    #[test]
    fn rejects_min_support_below_floor() {
        let r = mine_sequences(&[seq(&[1])], MinSupport(1), MaxGap(5), 8);
        assert!(matches!(r, Err(MinerError::MinSupportBelowFloor(1))));
    }

    #[test]
    fn rejects_empty_database() {
        let r = mine_sequences(&[], MinSupport(2), MaxGap(5), 8);
        assert!(matches!(r, Err(MinerError::EmptyDatabase)));
    }

    #[test]
    fn empty_sequences_are_accepted_but_yield_no_patterns() {
        let r = mine_sequences(
            &[Vec::<StepToken>::new(), Vec::<StepToken>::new()],
            MinSupport(2),
            MaxGap(5),
            8,
        )
        .expect("ok");
        assert!(r.is_empty());
    }

    // ---- Projection helper (3) ------------------------------------------

    #[test]
    fn project_returns_suffix_after_first_match() {
        let s = seq(&[1, 2, 3, 4, 5]);
        let p = project_after_prefix(&s, &[tok(1), tok(3)], MaxGap(5)).expect("match");
        assert_eq!(p.suffix, seq(&[4, 5]));
        assert_eq!(p.right_gap, 1);
    }

    #[test]
    fn project_returns_none_when_pattern_absent() {
        let s = seq(&[1, 2, 3]);
        assert!(project_after_prefix(&s, &[tok(4)], MaxGap(5)).is_none());
    }

    #[test]
    fn project_respects_max_gap_bound() {
        let s = seq(&[1, 9, 9, 9, 9, 9, 2]);
        // 5 intervening tokens > max_gap=4 → no match (under strict gap bound)
        let r = project_after_prefix(&s, &[tok(1), tok(2)], MaxGap(4));
        assert!(r.is_none() || r.unwrap().right_gap <= 4);
    }

    // ---- mine_sequences happy paths (6) ---------------------------------

    #[test]
    fn mine_finds_single_frequent_item() {
        let seqs = vec![seq(&[1, 2]), seq(&[1, 3]), seq(&[1, 4])];
        let p = mine_sequences(&seqs, MinSupport(3), MaxGap(5), 8).expect("ok");
        assert!(p.iter().any(|pat| pat.steps == seq(&[1]) && pat.support == 3));
    }

    #[test]
    fn mine_filters_below_min_support() {
        let seqs = vec![seq(&[1, 2]), seq(&[3, 4])];
        let p = mine_sequences(&seqs, MinSupport(2), MaxGap(5), 8).expect("ok");
        // No single item appears in BOTH sequences.
        for pat in &p {
            assert!(
                pat.support >= 2,
                "below-support pattern leaked: {pat:?}"
            );
        }
    }

    #[test]
    fn mine_finds_2_step_patterns() {
        let seqs = vec![
            seq(&[1, 2, 3]),
            seq(&[1, 5, 2]),
            seq(&[1, 2, 7]),
        ];
        let p = mine_sequences(&seqs, MinSupport(2), MaxGap(5), 8).expect("ok");
        assert!(
            p.iter().any(|pat| pat.steps == seq(&[1, 2])),
            "did not find [1,2] in {p:?}"
        );
    }

    #[test]
    fn mine_handles_gap_allowed_matching() {
        let seqs = vec![
            seq(&[1, 9, 2]),   // [1,2] under gap=1
            seq(&[1, 9, 9, 2]), // [1,2] under gap=2
            seq(&[1, 2]),       // [1,2] under gap=0
        ];
        let p = mine_sequences(&seqs, MinSupport(2), MaxGap(5), 8).expect("ok");
        let found_12 = p.iter().any(|pat| pat.steps == seq(&[1, 2]));
        assert!(found_12, "gap-allowed [1,2] missed in {p:?}");
    }

    #[test]
    fn mine_respects_max_length_cap() {
        let seqs: Vec<Vec<StepToken>> = (0..3).map(|_| seq(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10])).collect();
        let p = mine_sequences(&seqs, MinSupport(3), MaxGap(10), 3).expect("ok");
        for pat in &p {
            assert!(pat.steps.len() <= 3, "pattern exceeds max_length: {pat:?}");
        }
    }

    #[test]
    fn mine_is_deterministic_across_runs() {
        let seqs = vec![seq(&[3, 1, 2]), seq(&[1, 2, 3]), seq(&[2, 3, 1])];
        let p1 = mine_sequences(&seqs, MinSupport(2), MaxGap(5), 8).expect("ok");
        for _ in 0..10_u32 {
            let p2 = mine_sequences(&seqs, MinSupport(2), MaxGap(5), 8).expect("ok");
            assert_eq!(
                p1.iter().map(|p| (p.steps.clone(), p.support)).collect::<Vec<_>>(),
                p2.iter().map(|p| (p.steps.clone(), p.support)).collect::<Vec<_>>()
            );
        }
    }

    // ---- Output ordering invariant (2) ----------------------------------

    #[test]
    fn output_sorted_by_support_desc_then_length_desc() {
        let seqs = vec![
            seq(&[1, 2, 3]),
            seq(&[1, 2, 3]),
            seq(&[1, 2, 3]),
            seq(&[1, 5]),
            seq(&[1, 5]),
        ];
        let p = mine_sequences(&seqs, MinSupport(2), MaxGap(5), 8).expect("ok");
        // First entries should have highest support; ties broken by length.
        for w in p.windows(2) {
            let a = &w[0];
            let b = &w[1];
            let ok = a.support > b.support
                || (a.support == b.support && a.steps.len() >= b.steps.len())
                || (a.support == b.support
                    && a.steps.len() == b.steps.len()
                    && a.canonical_hash <= b.canonical_hash);
            assert!(ok, "ordering violation: {a:?} before {b:?}");
        }
    }

    #[test]
    fn output_supports_are_non_increasing_first_few_patterns() {
        let seqs = vec![
            seq(&[1, 2, 3, 4]),
            seq(&[1, 2, 3, 4]),
            seq(&[1, 2]),
        ];
        let p = mine_sequences(&seqs, MinSupport(2), MaxGap(5), 8).expect("ok");
        for w in p.windows(2) {
            assert!(w[0].support >= w[1].support);
        }
    }

    // ---- F11 token opacity (2) ------------------------------------------

    #[test]
    fn step_token_inner_is_pure_u32_no_string_inside() {
        // F11 compliance smoke: StepToken does not carry a String.
        let t = tok(0);
        assert_eq!(std::mem::size_of_val(&t), std::mem::size_of::<u32>());
    }

    #[test]
    fn pattern_serde_round_trip_preserves_opaque_token() {
        let p = Pattern::new(seq(&[7, 11, 13]), 5, (1, 3));
        let s = serde_json::to_string(&p).expect("ser");
        let back: Pattern = serde_json::from_str(&s).expect("de");
        assert_eq!(back, p);
    }
}
