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
///
/// The inner value is **private**: a `MinSupport` cannot exist unless it
/// carries a value `>= MIN_SUPPORT_FLOOR`. Construct via the fallible
/// [`MinSupport::new`]; this hoists the F2 hard-floor check from a runtime
/// guard inside [`mine_sequences`] up into the type system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MinSupport(usize);

impl MinSupport {
    /// Construct a `MinSupport`, enforcing the **F2 hard floor**.
    ///
    /// # Errors
    ///
    /// Returns [`MinerError::MinSupportBelowFloor`] if `value` is below
    /// [`MIN_SUPPORT_FLOOR`].
    pub fn new(value: usize) -> Result<Self, MinerError> {
        if value < MIN_SUPPORT_FLOOR {
            return Err(MinerError::MinSupportBelowFloor(value));
        }
        Ok(Self(value))
    }

    /// Borrow the inner support count (always `>= MIN_SUPPORT_FLOOR`).
    #[must_use]
    pub fn get(self) -> usize {
        self.0
    }
}

/// Newtype for max-gap config (right-gap bound during matching).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaxGap(usize);

impl MaxGap {
    /// Construct a `MaxGap` right-gap bound. Any `usize` is a valid
    /// gap bound (0 = adjacent-only); construction is infallible.
    #[must_use]
    pub fn new(value: usize) -> Self {
        Self(value)
    }

    /// Borrow the inner right-gap bound.
    #[must_use]
    pub fn get(self) -> usize {
        self.0
    }
}

/// **F2 hard floor** — `MinSupport` below this value is rejected outright.
pub const MIN_SUPPORT_FLOOR: usize = 2;

/// Maximum pattern length (depth cap on recursion).
pub const DEFAULT_MAX_LENGTH: usize = 8;

/// Default right-gap bound.
pub const DEFAULT_MAX_GAP: usize = 5;

/// A frequent sequential pattern emitted by [`mine_sequences`].
///
/// All fields are **private**: a `Pattern` cannot exist with a
/// `canonical_hash` that disagrees with its `steps`/`gap_bounds`. The
/// canonical hash is the cross-module identity (m21/m23 key off it), so it
/// must always be derived by [`Pattern::new`]. Read state through the
/// accessors ([`Pattern::steps`], [`Pattern::support`],
/// [`Pattern::gap_bounds`], [`Pattern::canonical_hash`]).
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Pattern {
    /// Ordered token sequence.
    steps: Vec<StepToken>,
    /// Number of input sequences containing this pattern.
    support: usize,
    /// `(min_left_gap, max_right_gap)` observed during matching.
    gap_bounds: (usize, usize),
    /// Stable hash for cross-module identity (FNV-1a 64 over
    /// `step_id:gap_min:gap_max` concatenation).
    canonical_hash: u64,
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

    /// Borrow the ordered token sequence.
    #[must_use]
    pub fn steps(&self) -> &[StepToken] {
        &self.steps
    }

    /// Number of input sequences containing this pattern.
    #[must_use]
    pub const fn support(&self) -> usize {
        self.support
    }

    /// `(min_left_gap, max_right_gap)` observed during matching.
    #[must_use]
    pub const fn gap_bounds(&self) -> (usize, usize) {
        self.gap_bounds
    }

    /// Stable cross-module identity hash (FNV-1a 64 over
    /// `step_id:gap_min:gap_max`). Always consistent with
    /// [`Self::steps`] / [`Self::gap_bounds`] because both are derived
    /// together in [`Self::new`].
    #[must_use]
    pub const fn canonical_hash(&self) -> u64 {
        self.canonical_hash
    }
}

/// Miner failure modes.
#[derive(Debug, Error)]
#[non_exhaustive]
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
    /// `max_length` was 0; pattern length must be at least 1.
    #[error("max_length must be >= 1, got 0")]
    MaxLengthZero,
}

/// PrefixSpan with gap-allowed matching.
///
/// Mines frequent sequential patterns from `sequences` under the given
/// support floor and right-gap bound. Output is sorted by
/// `(support DESC, len DESC, canonical_hash ASC)`.
///
/// The F2 hard floor on `min_support` is enforced at the type level by
/// [`MinSupport::new`] — a `MinSupport` value reaching this function is
/// already guaranteed `>= MIN_SUPPORT_FLOOR`.
///
/// # Errors
///
/// - [`MinerError::EmptyDatabase`] if `sequences` is empty.
/// - [`MinerError::MaxLengthZero`] if `max_length` is 0.
pub fn mine_sequences(
    sequences: &[Vec<StepToken>],
    min_support: MinSupport,
    max_gap: MaxGap,
    max_length: usize,
) -> Result<Vec<Pattern>, MinerError> {
    if sequences.is_empty() {
        return Err(MinerError::EmptyDatabase);
    }
    if max_length == 0 {
        return Err(MinerError::MaxLengthZero);
    }

    // Per spec § 5: L1 frequency scan is one pass over the database
    // counting per-sequence occurrence (not per-token), which makes the
    // L1 support count equal to the number of input sequences that
    // contain the token at least once.
    let mut frequencies: HashMap<StepToken, usize> =
        HashMap::with_capacity(sequences.len().min(64));
    for seq in sequences {
        // Capacity hint: most cascade sequences carry ≤16 distinct tokens.
        let mut seen: std::collections::HashSet<StepToken> =
            std::collections::HashSet::with_capacity(seq.len().min(16));
        for tok in seq {
            if seen.insert(*tok) {
                *frequencies.entry(*tok).or_insert(0) += 1;
            }
        }
    }
    let mut frequent_items: Vec<StepToken> = frequencies
        .iter()
        .filter(|(_, &c)| c >= min_support.get())
        .map(|(t, _)| *t)
        .collect();
    frequent_items.sort();

    // Pre-allocate a generous-but-bounded result buffer. Each frequent
    // L1 item yields at most `max_length` patterns under perfect
    // extension; in practice the projection prunes harshly so this is
    // an upper-bound hint, not a guarantee.
    let mut results: Vec<Pattern> =
        Vec::with_capacity(frequent_items.len().saturating_mul(max_length));
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
        b.support()
            .cmp(&a.support())
            .then_with(|| b.steps().len().cmp(&a.steps().len()))
            .then_with(|| a.canonical_hash().cmp(&b.canonical_hash()))
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
    // Capacity hint: at most one projected suffix per input sequence.
    let mut suffixes: Vec<Vec<StepToken>> = Vec::with_capacity(sequences.len());
    let mut max_right_gap = 0_usize;
    for seq in sequences {
        if let Some(suffix_info) = project_after_prefix(seq, prefix, max_gap) {
            max_right_gap = max_right_gap.max(suffix_info.right_gap);
            suffixes.push(suffix_info.suffix);
        }
    }
    // Count length-1 extensions in the projected suffixes.
    let mut ext_freq: HashMap<StepToken, usize> =
        HashMap::with_capacity(suffixes.len().min(64));
    for suf in &suffixes {
        let mut seen: std::collections::HashSet<StepToken> =
            std::collections::HashSet::with_capacity(suf.len().min(16));
        for tok in suf {
            if seen.insert(*tok) {
                *ext_freq.entry(*tok).or_insert(0) += 1;
            }
        }
    }
    let mut frequent_exts: Vec<(StepToken, usize)> = ext_freq
        .into_iter()
        .filter(|(_, c)| *c >= min_support.get())
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

/// Find the first gap-bounded occurrence of `prefix` in `seq`.
///
/// Gap-allowed semantics: between two successively matched prefix
/// tokens at most `max_gap.get()` non-matching tokens may be skipped.
/// "First" means the earliest start index (a `seq` position equal to
/// `prefix[0]`) that admits a *complete* embedding of the whole prefix;
/// within that start every prefix token is matched as early as
/// possible, falling back to a backtracked choice only when the
/// earliest choice cannot be completed.
///
/// Returns the suffix AFTER the last matched prefix token, plus the
/// maximum inter-token gap observed in the chosen embedding.
///
/// A greedy single pass is *incomplete* for this problem — an
/// over-gapped first candidate, or an earliest-but-dead-ending
/// intermediate match, can hide a valid embedding — so both the start
/// position and each intermediate match are allowed to backtrack.
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
    // `(p_idx, prev_idx)` states already proven to admit no embedding —
    // shared across start positions so the search stays polynomial.
    let mut failed: std::collections::HashSet<(usize, usize)> =
        std::collections::HashSet::new();
    for (start, tok) in seq.iter().enumerate() {
        if *tok != prefix[0] {
            continue;
        }
        if let Some((last_idx, max_observed)) =
            embed_from(seq, prefix, 1, start, max_gap.get(), &mut failed)
        {
            let after = last_idx.saturating_add(1);
            let suffix = if after >= seq.len() {
                Vec::new()
            } else {
                seq[after..].to_vec()
            };
            return Some(ProjectedSuffix {
                suffix,
                right_gap: max_observed,
            });
        }
    }
    None
}

/// Depth-first, earliest-position-first embedding of `prefix[p_idx..]`
/// into `seq`, given the previous prefix token was matched at
/// `prev_idx`. Returns `(last_matched_idx, max_gap_observed)` for the
/// first complete embedding found. Earliest-first traversal yields the
/// greedy embedding when greedy succeeds and the earliest backtracked
/// embedding otherwise. Failed `(p_idx, prev_idx)` states are memoised
/// in `failed` so the search is polynomial against adversarial input.
fn embed_from(
    seq: &[StepToken],
    prefix: &[StepToken],
    p_idx: usize,
    prev_idx: usize,
    max_gap: usize,
    failed: &mut std::collections::HashSet<(usize, usize)>,
) -> Option<(usize, usize)> {
    if p_idx == prefix.len() {
        return Some((prev_idx, 0));
    }
    if failed.contains(&(p_idx, prev_idx)) {
        return None;
    }
    let first = prev_idx.saturating_add(1);
    if first >= seq.len() {
        failed.insert((p_idx, prev_idx));
        return None;
    }
    // prefix[p_idx] may land in `prev_idx+1 ..= prev_idx+1+max_gap`,
    // clamped to the last valid index.
    let last = first
        .saturating_add(max_gap)
        .min(seq.len().saturating_sub(1));
    for cand in first..=last {
        if seq[cand] != prefix[p_idx] {
            continue;
        }
        // `cand >= first == prev_idx + 1`, so the subtraction is safe.
        let gap = cand.saturating_sub(prev_idx).saturating_sub(1);
        if let Some((deep_last, deep_max)) =
            embed_from(seq, prefix, p_idx + 1, cand, max_gap, failed)
        {
            return Some((deep_last, gap.max(deep_max)));
        }
    }
    failed.insert((p_idx, prev_idx));
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
        assert_eq!(a.canonical_hash(), b.canonical_hash());
    }

    #[test]
    fn pattern_canonical_hash_distinguishes_steps() {
        let a = Pattern::new(seq(&[1, 2]), 3, (0, 0));
        let b = Pattern::new(seq(&[1, 3]), 3, (0, 0));
        assert_ne!(a.canonical_hash(), b.canonical_hash());
    }

    #[test]
    fn pattern_canonical_hash_distinguishes_gap_bounds() {
        let a = Pattern::new(seq(&[1, 2]), 3, (0, 0));
        let b = Pattern::new(seq(&[1, 2]), 3, (0, 5));
        assert_ne!(a.canonical_hash(), b.canonical_hash());
    }

    // ---- Refusal modes (3) ----------------------------------------------

    // rationale: F2 hard floor is now enforced at the type level —
    // MinSupport::new(1) is rejected before a MinSupport value can exist,
    // so mine_sequences can no longer be reached with an under-floor value.
    #[test]
    fn min_support_new_rejects_below_floor() {
        let r = MinSupport::new(1);
        assert!(matches!(r, Err(MinerError::MinSupportBelowFloor(1))));
        // 0 is also rejected (degenerate floor case).
        assert!(matches!(
            MinSupport::new(0),
            Err(MinerError::MinSupportBelowFloor(0))
        ));
    }

    #[test]
    fn rejects_empty_database() {
        let r = mine_sequences(
            &[],
            MinSupport::new(2).expect("at floor"),
            MaxGap::new(5),
            8,
        );
        assert!(matches!(r, Err(MinerError::EmptyDatabase)));
    }

    #[test]
    fn empty_sequences_are_accepted_but_yield_no_patterns() {
        let r = mine_sequences(
            &[Vec::<StepToken>::new(), Vec::<StepToken>::new()],
            MinSupport::new(2).expect("min_support >= floor"),
            MaxGap::new(5),
            8,
        )
        .expect("ok");
        assert!(r.is_empty());
    }

    // ---- Projection helper (3) ------------------------------------------

    #[test]
    fn project_returns_suffix_after_first_match() {
        let s = seq(&[1, 2, 3, 4, 5]);
        let p = project_after_prefix(&s, &[tok(1), tok(3)], MaxGap::new(5)).expect("match");
        assert_eq!(p.suffix, seq(&[4, 5]));
        assert_eq!(p.right_gap, 1);
    }

    #[test]
    fn project_returns_none_when_pattern_absent() {
        let s = seq(&[1, 2, 3]);
        assert!(project_after_prefix(&s, &[tok(4)], MaxGap::new(5)).is_none());
    }

    #[test]
    fn project_respects_max_gap_bound() {
        let s = seq(&[1, 9, 9, 9, 9, 9, 2]);
        // 5 intervening tokens > max_gap=4 → no match (under strict gap bound)
        let r = project_after_prefix(&s, &[tok(1), tok(2)], MaxGap::new(4));
        assert!(r.is_none() || r.unwrap().right_gap <= 4);
    }

    // ---- mine_sequences happy paths (6) ---------------------------------

    #[test]
    fn mine_finds_single_frequent_item() {
        let seqs = vec![seq(&[1, 2]), seq(&[1, 3]), seq(&[1, 4])];
        let p = mine_sequences(&seqs, MinSupport::new(3).expect("min_support >= floor"), MaxGap::new(5), 8).expect("ok");
        assert!(p.iter().any(|pat| pat.steps() == seq(&[1]) && pat.support() == 3));
    }

    #[test]
    fn mine_filters_below_min_support() {
        let seqs = vec![seq(&[1, 2]), seq(&[3, 4])];
        let p = mine_sequences(&seqs, MinSupport::new(2).expect("min_support >= floor"), MaxGap::new(5), 8).expect("ok");
        // No single item appears in BOTH sequences.
        for pat in &p {
            assert!(
                pat.support() >= 2,
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
        let p = mine_sequences(&seqs, MinSupport::new(2).expect("min_support >= floor"), MaxGap::new(5), 8).expect("ok");
        assert!(
            p.iter().any(|pat| pat.steps() == seq(&[1, 2])),
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
        let p = mine_sequences(&seqs, MinSupport::new(2).expect("min_support >= floor"), MaxGap::new(5), 8).expect("ok");
        let found_12 = p.iter().any(|pat| pat.steps() == seq(&[1, 2]));
        assert!(found_12, "gap-allowed [1,2] missed in {p:?}");
    }

    #[test]
    fn mine_respects_max_length_cap() {
        let seqs: Vec<Vec<StepToken>> = (0..3).map(|_| seq(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10])).collect();
        let p = mine_sequences(&seqs, MinSupport::new(3).expect("min_support >= floor"), MaxGap::new(10), 3).expect("ok");
        for pat in &p {
            assert!(pat.steps().len() <= 3, "pattern exceeds max_length: {pat:?}");
        }
    }

    #[test]
    fn mine_is_deterministic_across_runs() {
        let seqs = vec![seq(&[3, 1, 2]), seq(&[1, 2, 3]), seq(&[2, 3, 1])];
        let p1 = mine_sequences(&seqs, MinSupport::new(2).expect("min_support >= floor"), MaxGap::new(5), 8).expect("ok");
        for _ in 0..10_u32 {
            let p2 = mine_sequences(&seqs, MinSupport::new(2).expect("min_support >= floor"), MaxGap::new(5), 8).expect("ok");
            assert_eq!(
                p1.iter().map(|p| (p.steps().to_vec(), p.support())).collect::<Vec<_>>(),
                p2.iter().map(|p| (p.steps().to_vec(), p.support())).collect::<Vec<_>>()
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
        let p = mine_sequences(&seqs, MinSupport::new(2).expect("min_support >= floor"), MaxGap::new(5), 8).expect("ok");
        // First entries should have highest support; ties broken by length.
        for w in p.windows(2) {
            let a = &w[0];
            let b = &w[1];
            let ok = a.support() > b.support()
                || (a.support() == b.support() && a.steps().len() >= b.steps().len())
                || (a.support() == b.support()
                    && a.steps().len() == b.steps().len()
                    && a.canonical_hash() <= b.canonical_hash());
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
        let p = mine_sequences(&seqs, MinSupport::new(2).expect("min_support >= floor"), MaxGap::new(5), 8).expect("ok");
        for w in p.windows(2) {
            assert!(w[0].support() >= w[1].support());
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

    // ---- Cluster F hardening pass — adversarial / boundary / resource ----

    #[test]
    // rationale: Boundary — MIN_SUPPORT_FLOOR-1 must refuse at construction
    // (F2 hard floor is enforced by MinSupport::new, not by mine_sequences).
    fn boundary_min_support_floor_minus_one_refuses() {
        let r = MinSupport::new(MIN_SUPPORT_FLOOR - 1);
        assert!(matches!(r, Err(MinerError::MinSupportBelowFloor(_))));
    }

    #[test]
    // rationale: Boundary — exactly at MIN_SUPPORT_FLOOR must be accepted.
    fn boundary_min_support_at_floor_accepted() {
        let r = mine_sequences(
            &[seq(&[1]), seq(&[1])],
            MinSupport::new(MIN_SUPPORT_FLOOR).expect("at floor accepted"),
            MaxGap::new(5),
            8,
        );
        assert!(r.is_ok());
    }

    #[test]
    // rationale: Boundary — max_length=0 is rejected with a typed error
    // (no silent coercion to 1).
    fn boundary_max_length_zero_rejected() {
        let r = mine_sequences(
            &[seq(&[1, 2, 3]), seq(&[1, 2, 4])],
            MinSupport::new(2).expect("min_support >= floor"),
            MaxGap::new(5),
            0,
        );
        assert!(matches!(r, Err(MinerError::MaxLengthZero)));
    }

    #[test]
    // rationale: Adversarial input — single-step sequence is valid.
    fn adversarial_single_step_sequence_handled() {
        let p = mine_sequences(
            &[seq(&[42]), seq(&[42]), seq(&[42])],
            MinSupport::new(2).expect("min_support >= floor"),
            MaxGap::new(5),
            8,
        )
        .expect("ok");
        assert!(p.iter().any(|pat| pat.steps() == seq(&[42]) && pat.support() == 3));
    }

    #[test]
    // rationale: Adversarial input — pathological 10k-element repeated pattern
    // must not panic, must not OOM (Vec::with_capacity hints), must complete.
    fn adversarial_long_repeated_sequence_does_not_panic() {
        let long: Vec<StepToken> = (0..10_000_u32).map(|i| tok(i % 4)).collect();
        let seqs = vec![long.clone(), long.clone(), long];
        let p = mine_sequences(&seqs, MinSupport::new(2).expect("min_support >= floor"), MaxGap::new(2), 3).expect("ok");
        // Just assert it returned; the algorithm's bounded-depth guarantees
        // termination at max_length=3.
        assert!(!p.is_empty());
        for pat in &p {
            assert!(pat.steps().len() <= 3);
        }
    }

    #[test]
    // rationale: Adversarial input — empty sequence interleaved with non-empty.
    fn adversarial_mixed_empty_and_non_empty_sequences() {
        let seqs = vec![
            Vec::<StepToken>::new(),
            seq(&[1, 2]),
            Vec::<StepToken>::new(),
            seq(&[1, 2]),
        ];
        let p = mine_sequences(&seqs, MinSupport::new(2).expect("min_support >= floor"), MaxGap::new(5), 8).expect("ok");
        assert!(p.iter().any(|pat| pat.steps() == seq(&[1, 2])));
    }

    #[test]
    // rationale: Boundary — max u32 StepToken value must not overflow counts.
    fn boundary_max_u32_step_token_handled() {
        let big = StepToken(u32::MAX);
        let seqs = vec![vec![big], vec![big]];
        let p = mine_sequences(&seqs, MinSupport::new(2).expect("min_support >= floor"), MaxGap::new(5), 8).expect("ok");
        assert!(p.iter().any(|pat| pat.steps() == vec![big] && pat.support() == 2));
    }

    #[test]
    // rationale: Determinism — output bit-stable across iteration order
    // permutations of the input (per spec § 5 ordering invariant).
    fn determinism_output_stable_under_input_permutation() {
        let a = vec![seq(&[1, 2]), seq(&[1, 3]), seq(&[1, 2, 3])];
        let b = vec![seq(&[1, 2, 3]), seq(&[1, 2]), seq(&[1, 3])];
        let pa = mine_sequences(&a, MinSupport::new(2).expect("min_support >= floor"), MaxGap::new(5), 8).expect("a");
        let pb = mine_sequences(&b, MinSupport::new(2).expect("min_support >= floor"), MaxGap::new(5), 8).expect("b");
        // The full output multiset must be identical (HashMap iteration is
        // non-deterministic across permutations, but the final sort is total).
        let fa: Vec<_> = pa.iter().map(|p| (p.steps().to_vec(), p.support())).collect();
        let fb: Vec<_> = pb.iter().map(|p| (p.steps().to_vec(), p.support())).collect();
        assert_eq!(fa, fb);
    }

    #[test]
    // rationale: Anti-property — F11 opacity: Pattern serde JSON must NOT
    // contain any human-readable substring (cluster names, "pane", "tab",
    // or the workflow-trace namespace prefix).
    fn anti_property_f11_pattern_serde_carries_no_human_label() {
        let p = Pattern::new(seq(&[1, 2]), 5, (0, 0));
        let s = serde_json::to_string(&p).expect("ser");
        // Use the m9 namespace constant — AP30 forbids re-hardcoding the
        // namespace-prefix literal anywhere outside of the m9 module.
        let ns_prefix = crate::m9_watcher_namespace_guard::WORKFLOW_TRACE_NS_PREFIX;
        for forbidden in ["pane", "tab", "cluster_pane"] {
            assert!(
                !s.contains(forbidden),
                "F11 violation: serde output contains '{forbidden}' in {s}"
            );
        }
        assert!(
            !s.contains(ns_prefix),
            "F11 violation: namespace prefix '{ns_prefix}' leaked into Pattern serde: {s}"
        );
    }

    #[test]
    // rationale: Contract regression — MinerError variants stable across edits.
    fn contract_miner_error_variants_stable() {
        // Trip every public variant once to lock the taxonomy.
        let floor = MinerError::MinSupportBelowFloor(1);
        let empty = MinerError::EmptyDatabase;
        let too_long = MinerError::PatternTooLong { len: 10, max: 8 };
        let max_len_zero = MinerError::MaxLengthZero;
        // Display impls must not panic.
        assert!(!format!("{floor}").is_empty());
        assert!(!format!("{empty}").is_empty());
        assert!(!format!("{too_long}").is_empty());
        assert!(!format!("{max_len_zero}").is_empty());
    }

    #[test]
    // rationale: Resource accounting — projection helper does not allocate on
    // a no-match path (empty-prefix-after returns full seq).
    fn projection_empty_prefix_returns_full_seq() {
        let s = seq(&[1, 2, 3]);
        let p = project_after_prefix(&s, &[], MaxGap::new(5)).expect("empty prefix");
        assert_eq!(p.suffix, s);
    }

    #[test]
    // rationale: Cross-module surface — Pattern is consumed by m21
    // (variant_builder); hash stability across re-mines must hold.
    fn cross_module_pattern_canonical_hash_survives_remine() {
        let seqs = vec![seq(&[1, 2, 3]), seq(&[1, 2, 3]), seq(&[1, 2, 4])];
        let a = mine_sequences(&seqs, MinSupport::new(2).expect("min_support >= floor"), MaxGap::new(5), 8).expect("a");
        let b = mine_sequences(&seqs, MinSupport::new(2).expect("min_support >= floor"), MaxGap::new(5), 8).expect("b");
        for (pa, pb) in a.iter().zip(b.iter()) {
            assert_eq!(pa.canonical_hash(), pb.canonical_hash());
        }
    }

    #[test]
    // rationale: Boundary — MAX_GAP=0 still emits patterns; gap discipline
    // applies to prefix-projection cursor advancement, not to extension
    // counting in the projected suffix (per spec § 5 "Gap-Allowed Matching
    // Model": bounded right-gap on prefix items; extensions counted in
    // projected suffix). Both [1] and [2] L1 are frequent under gap=0;
    // [1,2] L2 is built by extension within the projected suffix.
    fn boundary_max_gap_zero_completes_without_panic() {
        let seqs = vec![seq(&[1, 9, 2]), seq(&[1, 9, 2])];
        let p = mine_sequences(&seqs, MinSupport::new(2).expect("min_support >= floor"), MaxGap::new(0), 8).expect("ok");
        // L1 [1] and [2] both have support 2 — emission MUST be present.
        assert!(p.iter().any(|pat| pat.steps() == seq(&[1])));
        assert!(p.iter().any(|pat| pat.steps() == seq(&[2])));
    }

    #[test]
    // rationale: Concurrency — mine_sequences is `Send + Sync` reentrant
    // (pure function on borrowed inputs) — runs on two threads in parallel.
    fn concurrency_pure_function_is_send_safe() {
        use std::sync::Arc;
        use std::thread;
        let seqs = Arc::new(vec![seq(&[1, 2, 3]), seq(&[1, 2, 4]), seq(&[1, 2, 5])]);
        let s1 = Arc::clone(&seqs);
        let s2 = Arc::clone(&seqs);
        let h1 = thread::spawn(move || {
            mine_sequences(&s1, MinSupport::new(2).expect("min_support >= floor"), MaxGap::new(5), 8).expect("a")
        });
        let h2 = thread::spawn(move || {
            mine_sequences(&s2, MinSupport::new(2).expect("min_support >= floor"), MaxGap::new(5), 8).expect("b")
        });
        let a = h1.join().expect("t1");
        let b = h2.join().expect("t2");
        let fa: Vec<_> = a.iter().map(|p| (p.steps().to_vec(), p.support())).collect();
        let fb: Vec<_> = b.iter().map(|p| (p.steps().to_vec(), p.support())).collect();
        assert_eq!(fa, fb);
    }

    // ====================================================================
    // KEYSTONE hardening pass — known-input/known-output, monotonic-support
    // invariant, projection correctness, backtracking projection /
    // gap-bounded embedding, depth caps.
    // ====================================================================

    /// Helper: does the result contain a pattern with exactly `steps`?
    fn has(p: &[Pattern], steps: &[u32]) -> bool {
        p.iter().any(|pat| pat.steps() == seq(steps))
    }
    /// Helper: support of the pattern with exactly `steps` (panics if absent).
    fn support_of(p: &[Pattern], steps: &[u32]) -> usize {
        p.iter()
            .find(|pat| pat.steps() == seq(steps))
            .unwrap_or_else(|| panic!("pattern {steps:?} not found in {p:?}"))
            .support()
    }

    #[test]
    // rationale: KIO — three sequences all sharing [1,2,3]; the L1 [1],
    // L2 [1,2], L3 [1,2,3] patterns must each have support exactly 3, and
    // L2/L3 must be present (compositional sub-graph detection).
    fn kio_three_identical_sequences_full_pattern_lattice() {
        let seqs = vec![seq(&[1, 2, 3]), seq(&[1, 2, 3]), seq(&[1, 2, 3])];
        let p = mine_sequences(&seqs, MinSupport::new(3).expect("min_support >= floor"), MaxGap::new(5), 8).expect("ok");
        assert_eq!(support_of(&p, &[1]), 3);
        assert_eq!(support_of(&p, &[2]), 3);
        assert_eq!(support_of(&p, &[3]), 3);
        assert_eq!(support_of(&p, &[1, 2]), 3);
        assert_eq!(support_of(&p, &[2, 3]), 3);
        assert_eq!(support_of(&p, &[1, 2, 3]), 3);
    }

    #[test]
    // rationale: Algorithmic invariant — PrefixSpan support is anti-monotone:
    // a pattern's support can never exceed any of its prefixes' support.
    // (Apriori / downward-closure property.)
    fn invariant_support_anti_monotone_extension_never_gains() {
        let seqs = vec![
            seq(&[1, 2, 3, 4]),
            seq(&[1, 2, 3]),
            seq(&[1, 2]),
            seq(&[1]),
        ];
        let p = mine_sequences(&seqs, MinSupport::new(2).expect("min_support >= floor"), MaxGap::new(5), 8).expect("ok");
        // For every length-n pattern, its length-(n-1) prefix must exist
        // with support >= the longer pattern's support.
        for pat in &p {
            if pat.steps().len() < 2 {
                continue;
            }
            let prefix_steps: Vec<u32> =
                pat.steps()[..pat.steps().len() - 1].iter().map(|t| t.0).collect();
            let prefix_sup = support_of(&p, &prefix_steps);
            assert!(
                prefix_sup >= pat.support(),
                "anti-monotonicity violated: prefix {prefix_steps:?} sup={prefix_sup} \
                 < extension {pat:?}"
            );
        }
    }

    #[test]
    // rationale: KIO — a token frequent in only 1 of 3 sequences must be
    // pruned at min_support=2 (the L1 frequency floor).
    fn kio_l1_floor_prunes_rare_token() {
        let seqs = vec![seq(&[1, 2]), seq(&[1, 3]), seq(&[1, 9])];
        // token 9 appears once; tokens 2,3 once each; token 1 thrice.
        let p = mine_sequences(&seqs, MinSupport::new(2).expect("min_support >= floor"), MaxGap::new(5), 8).expect("ok");
        assert!(has(&p, &[1]), "frequent token 1 missing");
        assert!(!has(&p, &[9]), "rare token 9 leaked past min_support=2");
        assert!(!has(&p, &[2]), "rare token 2 leaked past min_support=2");
    }

    #[test]
    // rationale: KIO — L1 support counts SEQUENCES not OCCURRENCES (per
    // spec § 5: per-sequence occurrence). A token repeated 5× in one
    // sequence still contributes support 1 from that sequence.
    fn kio_l1_support_counts_sequences_not_occurrences() {
        let seqs = vec![seq(&[7, 7, 7, 7, 7]), seq(&[7])];
        let p = mine_sequences(&seqs, MinSupport::new(2).expect("min_support >= floor"), MaxGap::new(5), 8).expect("ok");
        // 7 appears in both sequences → support 2, NOT 6.
        assert_eq!(support_of(&p, &[7]), 2);
    }

    #[test]
    // rationale: Projection correctness — the suffix after a matched
    // multi-token prefix is exactly the tail past the LAST matched token.
    fn projection_suffix_is_tail_past_last_prefix_token() {
        let s = seq(&[5, 1, 6, 2, 7, 3, 8]);
        let p = project_after_prefix(&s, &[tok(1), tok(2), tok(3)], MaxGap::new(5))
            .expect("match");
        assert_eq!(p.suffix, seq(&[8]));
        // gaps: 1→2 skip [6] (gap 1); 2→3 skip [7] (gap 1) → max right gap 1.
        assert_eq!(p.right_gap, 1);
    }

    #[test]
    // rationale: Projection correctness — exhausting the sequence on the
    // final prefix token yields an EMPTY suffix, not None.
    fn projection_empty_suffix_when_prefix_ends_at_sequence_end() {
        let s = seq(&[1, 2, 3]);
        let p = project_after_prefix(&s, &[tok(1), tok(3)], MaxGap::new(5)).expect("match");
        assert!(p.suffix.is_empty(), "expected empty suffix, got {:?}", p.suffix);
    }

    #[test]
    // rationale: Projection correctness — FIRST occurrence wins. With two
    // disjoint [1,2] matches, the suffix is taken after the FIRST.
    fn projection_takes_first_occurrence_not_last() {
        let s = seq(&[1, 2, 99, 1, 2, 100]);
        let p = project_after_prefix(&s, &[tok(1), tok(2)], MaxGap::new(5)).expect("match");
        // First [1,2] ends at idx 1 → suffix is everything from idx 2.
        assert_eq!(p.suffix, seq(&[99, 1, 2, 100]));
        assert_eq!(p.right_gap, 0);
    }

    #[test]
    // rationale: Gap semantics — exactly max_gap intervening tokens is
    // ACCEPTED (boundary inclusive); one more is rejected.
    fn projection_gap_exactly_at_bound_is_accepted() {
        // 3 intervening tokens between 1 and 2; max_gap=3 → accepted.
        let s = seq(&[1, 9, 9, 9, 2]);
        let p = project_after_prefix(&s, &[tok(1), tok(2)], MaxGap::new(3)).expect("match");
        assert_eq!(p.right_gap, 3);
        // max_gap=2 → 3 > 2 → rejected (no other start position admits an embedding).
        assert!(project_after_prefix(&s, &[tok(1), tok(2)], MaxGap::new(2)).is_none());
    }

    #[test]
    // rationale: F1 REGRESSION GUARD — the KEYSTONE over-gap restart bug.
    // Before the W2 fix, project_after_prefix was a greedy single pass: a
    // fresh `prefix[0]` appearing after an over-gap, while mid-match
    // expecting a non-first token, was neither matched nor used to
    // re-anchor — so [1, <over-gap noise>, 1, 2] failed to match [1,2]
    // and pattern support was UNDER-COUNTED. The matcher now backtracks
    // both the start position and intermediate matches; this test pins
    // the recovered (correct) behaviour.
    fn projection_recovers_when_first_token_recurs_after_overgap() {
        // First 1 at idx 0; 2 is 6 positions away (over max_gap=2). Second
        // 1 at idx 6 with 2 immediately after at idx 7 — the clean gap-0
        // match the old greedy pass missed.
        let s = seq(&[1, 9, 9, 9, 9, 9, 1, 2]);
        let p = project_after_prefix(&s, &[tok(1), tok(2)], MaxGap::new(2))
            .expect("[1,2] embeds at indices 6,7 within MaxGap(2)");
        assert!(p.suffix.is_empty(), "match ends at idx 7 — empty suffix");
        assert_eq!(p.right_gap, 0, "the recovered embedding (6,7) is gap-0");
    }

    #[test]
    // rationale: self-repeating prefix [1,1] — exercises the backtracking
    // matcher when the only second-token candidate over-gaps (no
    // embedding) and when a later candidate completes within gap.
    fn projection_restart_reachable_for_self_repeating_prefix() {
        // [1, 9,9,9,9,9, 1] — the only second 1 (idx 6) is over-gap
        // (gap 5 > max_gap 2) from the first 1 and no other 1 follows,
        // so [1,1] has no gap-bounded embedding → None.
        let s = seq(&[1, 9, 9, 9, 9, 9, 1]);
        let r = project_after_prefix(&s, &[tok(1), tok(1)], MaxGap::new(2));
        assert!(r.is_none(), "expected None — no second 1 within gap");
        // [1, 9,9, 1, 1] completes [1,1] within gap: first 1 at idx0,
        // second 1 at idx3 (gap = 3-0-1 = 2, within MaxGap(2)). The match
        // ends at idx3, so the suffix is the tail past idx3 = [1].
        let s2 = seq(&[1, 9, 9, 1, 1]);
        let p = project_after_prefix(&s2, &[tok(1), tok(1)], MaxGap::new(2)).expect("match");
        assert_eq!(p.suffix, seq(&[1]));
        assert_eq!(p.right_gap, 2, "the single observed inter-prefix gap is 2");
    }

    #[test]
    // rationale: F1 — earliest-greedy from a start can dead-end; the
    // matcher must backtrack the intermediate match. prefix [1,2,3] in
    // [1,2,2,9,3] MaxGap(1): greedy picks 2@1 then cannot reach 3, so it
    // must fall back to 2@2 to complete via 3@4.
    fn projection_backtracks_dead_ending_intermediate_match() {
        let s = seq(&[1, 2, 2, 9, 3]);
        let p = project_after_prefix(&s, &[tok(1), tok(2), tok(3)], MaxGap::new(1))
            .expect("[1,2,3] embeds at (0,2,4) within MaxGap(1)");
        assert!(p.suffix.is_empty(), "match ends at idx 4");
        assert_eq!(p.right_gap, 1, "both inter-token gaps are 1");
    }

    #[test]
    // rationale: F1 — a stray copy of prefix[0] mid-match must NOT force a
    // re-anchor that abandons a still-completable match. prefix [1,2,3] in
    // [1,2,1,3]: the 1 at idx 2 must be skipped, not re-anchored to.
    fn projection_does_not_falsely_reanchor_on_stray_first_token() {
        let s = seq(&[1, 2, 1, 3]);
        let p = project_after_prefix(&s, &[tok(1), tok(2), tok(3)], MaxGap::new(5))
            .expect("[1,2,3] embeds at (0,1,3)");
        assert!(p.suffix.is_empty(), "match ends at idx 3");
    }

    #[test]
    // rationale: F1 — the first start may over-gap while a later start
    // succeeds; the matcher returns the later start's embedding.
    fn projection_uses_later_start_when_first_start_overgaps() {
        // 1@0 then 2@8 over-gaps (MaxGap 2); 1@7, 2@8 is a clean match.
        let s = seq(&[1, 9, 9, 9, 9, 9, 9, 1, 2, 2]);
        let p = project_after_prefix(&s, &[tok(1), tok(2)], MaxGap::new(2))
            .expect("[1,2] embeds at (7,8)");
        assert_eq!(p.suffix, seq(&[2]), "match ends at idx 8, tail is [2]");
        assert_eq!(p.right_gap, 0);
    }

    #[test]
    // rationale: F1 — a genuinely absent token still returns None after
    // the all-starts search (exercises the failure memo across starts).
    fn projection_absent_pattern_returns_none_across_all_starts() {
        let s = seq(&[1, 1, 1, 1, 1]);
        assert!(
            project_after_prefix(&s, &[tok(1), tok(2)], MaxGap::new(9)).is_none(),
            "no token 2 anywhere — no embedding from any start"
        );
    }

    #[test]
    // rationale: F1 — token order matters: [1,2] cannot embed in [2,1]
    // (the 2 precedes the 1) regardless of the gap budget.
    fn projection_respects_token_order() {
        let s = seq(&[2, 1]);
        assert!(
            project_after_prefix(&s, &[tok(1), tok(2)], MaxGap::new(9)).is_none(),
            "[1,2] requires a 1 before a 2"
        );
    }

    #[test]
    // rationale: KIO — gap-allowed mining: [1,3] is frequent across
    // sequences where 1 and 3 are separated by varying gaps within bound.
    fn kio_gap_allowed_two_step_pattern_mined() {
        let seqs = vec![
            seq(&[1, 8, 3]),       // gap 1
            seq(&[1, 8, 8, 3]),    // gap 2
            seq(&[1, 3]),          // gap 0
        ];
        let p = mine_sequences(&seqs, MinSupport::new(3).expect("min_support >= floor"), MaxGap::new(5), 8).expect("ok");
        assert_eq!(support_of(&p, &[1, 3]), 3);
    }

    #[test]
    // rationale: Boundary — max_length exactly 1 yields ONLY L1 patterns
    // even when longer patterns exist in the data.
    fn boundary_max_length_one_yields_only_l1() {
        let seqs = vec![seq(&[1, 2, 3]), seq(&[1, 2, 3])];
        let p = mine_sequences(&seqs, MinSupport::new(2).expect("min_support >= floor"), MaxGap::new(5), 1).expect("ok");
        for pat in &p {
            assert_eq!(pat.steps().len(), 1, "max_length=1 yielded {pat:?}");
        }
        assert!(has(&p, &[1]) && has(&p, &[2]) && has(&p, &[3]));
    }

    #[test]
    // rationale: Boundary — max_length=2 caps at L2 even with a clean
    // length-4 repeated pattern.
    fn boundary_max_length_two_caps_at_l2() {
        let seqs = vec![seq(&[1, 2, 3, 4]), seq(&[1, 2, 3, 4]), seq(&[1, 2, 3, 4])];
        let p = mine_sequences(&seqs, MinSupport::new(3).expect("min_support >= floor"), MaxGap::new(5), 2).expect("ok");
        for pat in &p {
            assert!(pat.steps().len() <= 2, "L2 cap violated: {pat:?}");
        }
        assert!(has(&p, &[1, 2]), "L2 [1,2] must still be present");
    }

    #[test]
    // rationale: Boundary — min_support exactly equal to the sequence count
    // means a pattern must appear in EVERY sequence to survive.
    fn boundary_min_support_equals_count_requires_universal_pattern() {
        let seqs = vec![seq(&[1, 2]), seq(&[1, 3]), seq(&[1, 4])];
        let p = mine_sequences(&seqs, MinSupport::new(3).expect("min_support >= floor"), MaxGap::new(5), 8).expect("ok");
        // Only token 1 is in all 3 sequences.
        assert!(has(&p, &[1]));
        assert!(!has(&p, &[2]) && !has(&p, &[3]) && !has(&p, &[4]));
        assert_eq!(p.len(), 1, "exactly one universal pattern expected: {p:?}");
    }

    #[test]
    // rationale: KIO — interleaved noise. [1,2,3] is a sub-sequence of each
    // input despite heavy noise insertion; gap-allowed mining recovers it.
    fn kio_compositional_pattern_under_interleaved_noise() {
        let seqs = vec![
            seq(&[1, 90, 2, 91, 3]),
            seq(&[1, 92, 2, 93, 3]),
            seq(&[1, 2, 94, 3]),
        ];
        let p = mine_sequences(&seqs, MinSupport::new(3).expect("min_support >= floor"), MaxGap::new(5), 8).expect("ok");
        assert_eq!(
            support_of(&p, &[1, 2, 3]),
            3,
            "compositional [1,2,3] not recovered under noise"
        );
    }

    #[test]
    // rationale: Determinism — output ordering total even with hash-colliding
    // support ties: equal-support, equal-length patterns are ordered by
    // canonical_hash ascending. Verify the tie-break leg directly.
    fn determinism_equal_support_equal_length_ordered_by_hash() {
        let seqs = vec![
            seq(&[1, 5]),
            seq(&[1, 5]),
            seq(&[2, 6]),
            seq(&[2, 6]),
        ];
        let p = mine_sequences(&seqs, MinSupport::new(2).expect("min_support >= floor"), MaxGap::new(5), 8).expect("ok");
        for w in p.windows(2) {
            if w[0].support() == w[1].support() && w[0].steps().len() == w[1].steps().len() {
                assert!(
                    w[0].canonical_hash() <= w[1].canonical_hash(),
                    "hash tie-break violated: {:?} before {:?}",
                    w[0],
                    w[1]
                );
            }
        }
    }

    #[test]
    // rationale: Anti-property — no pattern below min_support may EVER be
    // emitted, at ANY depth (L1 or recursive extension).
    fn anti_property_no_pattern_below_min_support_at_any_depth() {
        let seqs = vec![
            seq(&[1, 2, 3, 4, 5]),
            seq(&[1, 2, 3, 6, 7]),
            seq(&[1, 2, 8, 9, 10]),
        ];
        let p = mine_sequences(&seqs, MinSupport::new(2).expect("min_support >= floor"), MaxGap::new(5), 8).expect("ok");
        for pat in &p {
            assert!(
                pat.support() >= 2,
                "below-floor pattern leaked at depth {}: {pat:?}",
                pat.steps().len()
            );
        }
    }

    #[test]
    // rationale: KIO — the L1 pattern always carries gap_bounds (0,0); only
    // recursively-extended patterns observe a non-trivial right gap.
    fn kio_l1_pattern_gap_bounds_are_zero() {
        let seqs = vec![seq(&[1]), seq(&[1])];
        let p = mine_sequences(&seqs, MinSupport::new(2).expect("min_support >= floor"), MaxGap::new(5), 8).expect("ok");
        let l1 = p.iter().find(|pat| pat.steps() == seq(&[1])).expect("L1");
        assert_eq!(l1.gap_bounds(), (0, 0));
    }

    #[test]
    // rationale: Determinism — duplicate identical input sequences do not
    // perturb pattern identity; support scales with the duplication count.
    fn determinism_duplicate_sequences_scale_support() {
        let one = vec![seq(&[1, 2]), seq(&[1, 2])];
        let two = vec![seq(&[1, 2]), seq(&[1, 2]), seq(&[1, 2]), seq(&[1, 2])];
        let p1 = mine_sequences(&one, MinSupport::new(2).expect("min_support >= floor"), MaxGap::new(5), 8).expect("a");
        let p2 = mine_sequences(&two, MinSupport::new(2).expect("min_support >= floor"), MaxGap::new(5), 8).expect("b");
        assert_eq!(support_of(&p1, &[1, 2]), 2);
        assert_eq!(support_of(&p2, &[1, 2]), 4);
    }
}
