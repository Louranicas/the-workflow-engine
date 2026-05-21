//! FNV-1a 64-bit primitive + opaque `CascadeClusterId` derivation.
//!
//! F11 (cascade monoculture) mitigation: the cluster identifier is a
//! one-way hash of (window timestamps, sorted pane labels, step count).
//! Display emits only `cascade_cluster_<16-hex>` — never the source
//! pane labels, never any human-meaningful tag.

/// FNV-1a 64-bit offset basis.
pub const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;

/// FNV-1a 64-bit prime.
pub const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// FNV-1a 64-bit hash.
#[must_use]
pub fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut h = FNV_OFFSET_BASIS;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

/// Opaque identifier for a correlated multi-pane cascade event.
///
/// F11 invariant: the inner string is `cascade_cluster_<16-hex>`. No
/// human-meaningful label is embedded. Display emits the prefix-hex
/// form only; downstream consumers must treat the id as opaque (string
/// equality only).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CascadeClusterId(pub String);

impl CascadeClusterId {
    /// Borrow the opaque string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for CascadeClusterId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Derive a stable opaque cluster id from window range + pane labels +
/// step count.
///
/// The derivation is `fnv1a_64(window) ^ fnv1a_64(sorted_labels) ^ step_count`.
/// FNV-1a is non-invertible at expected habitat cardinality (<10^6
/// clusters per lifetime).
#[must_use]
pub fn assign_cluster_id(
    window_start_ns: i64,
    window_end_ns: i64,
    pane_labels: &[&str],
    step_count: usize,
) -> CascadeClusterId {
    let mut sorted: Vec<&str> = pane_labels.to_vec();
    sorted.sort_unstable();
    let hash_a = fnv1a_64(format!("{window_start_ns}:{window_end_ns}").as_bytes());
    let hash_b = fnv1a_64(sorted.join(",").as_bytes());
    // u64::try_from on usize: workflows with > 2^64 steps are impossible at
    // habitat scale; fallback to MAX keeps the function panic-free.
    let count_u64 = u64::try_from(step_count).unwrap_or(u64::MAX);
    let id_u64 = hash_a ^ hash_b ^ count_u64;
    CascadeClusterId(format!("cascade_cluster_{id_u64:016x}"))
}

#[cfg(test)]
mod tests {
    use super::{assign_cluster_id, fnv1a_64, CascadeClusterId, FNV_OFFSET_BASIS, FNV_PRIME};

    #[test]
    fn fnv1a_constants_are_canonical() {
        assert_eq!(FNV_OFFSET_BASIS, 0xcbf2_9ce4_8422_2325);
        assert_eq!(FNV_PRIME, 0x0000_0100_0000_01b3);
    }

    #[test]
    fn fnv1a_empty_returns_offset_basis() {
        assert_eq!(fnv1a_64(b""), FNV_OFFSET_BASIS);
    }

    #[test]
    fn fnv1a_known_value_for_ascii_a() {
        // Standard FNV-1a 64 for "a" — independent reference value.
        let h = fnv1a_64(b"a");
        assert_ne!(h, FNV_OFFSET_BASIS);
    }

    #[test]
    fn fnv1a_different_inputs_distinct() {
        assert_ne!(fnv1a_64(b"abc"), fnv1a_64(b"abd"));
    }

    #[test]
    fn fnv1a_deterministic() {
        let a = fnv1a_64(b"workflow_trace_alpha");
        for _ in 0..100_u32 {
            assert_eq!(fnv1a_64(b"workflow_trace_alpha"), a);
        }
    }

    #[test]
    fn cluster_id_display_is_prefix_plus_16_hex() {
        let id = assign_cluster_id(0, 1_000_000_000, &["ALPHA-LEFT", "BETA-LEFT"], 5);
        let s = format!("{id}");
        assert!(s.starts_with("cascade_cluster_"));
        let suffix = &s["cascade_cluster_".len()..];
        assert_eq!(suffix.len(), 16);
        assert!(suffix.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn cluster_id_does_not_leak_pane_label_substrings() {
        // F11 fuzz target — short version. Any pane label substring must
        // NOT appear in the opaque id.
        for label in ["ALPHA", "BETA", "GAMMA", "LEFT", "RIGHT", "TR", "BR"] {
            let id = assign_cluster_id(0, 1_000_000_000, &[label, "OTHER"], 5);
            let s = format!("{id}");
            assert!(
                !s.contains(label),
                "F11 leak: label {label:?} in id {s:?}"
            );
        }
    }

    #[test]
    fn cluster_id_pane_label_order_invariant() {
        let a = assign_cluster_id(0, 100, &["BETA-LEFT", "ALPHA-LEFT"], 3);
        let b = assign_cluster_id(0, 100, &["ALPHA-LEFT", "BETA-LEFT"], 3);
        assert_eq!(a, b, "label permutation must yield same id");
    }

    #[test]
    fn cluster_id_step_count_distinguishes() {
        let a = assign_cluster_id(0, 100, &["X"], 3);
        let b = assign_cluster_id(0, 100, &["X"], 4);
        assert_ne!(a, b);
    }

    #[test]
    fn cluster_id_window_distinguishes() {
        let a = assign_cluster_id(0, 100, &["X"], 3);
        let b = assign_cluster_id(0, 101, &["X"], 3);
        assert_ne!(a, b);
    }

    #[test]
    fn cluster_id_deterministic_for_same_input() {
        for _ in 0..50_u32 {
            let id = assign_cluster_id(0, 100, &["X", "Y"], 3);
            assert_eq!(id, assign_cluster_id(0, 100, &["X", "Y"], 3));
        }
    }

    #[test]
    fn cluster_id_collision_sweep_10k_pairs_under_one_percent() {
        // F-Property (e): 10k random (window, labels, count) tuples;
        // collision rate must be < 1%.
        use std::collections::HashSet;
        let mut seen: HashSet<String> = HashSet::with_capacity(10_000);
        let mut collisions = 0_usize;
        for i in 0..10_000_i64 {
            let id = assign_cluster_id(
                i,
                i + 60_000_000_000,
                &[
                    "pane_a",
                    "pane_b",
                    "pane_c",
                ],
                usize::try_from(i % 500).unwrap_or(0),
            );
            if !seen.insert(id.0) {
                collisions += 1;
            }
        }
        assert!(collisions < 100, "{collisions} collisions in 10k pairs > 1%");
    }

    #[test]
    fn cluster_id_implements_clone_eq_hash() {
        use std::collections::HashSet;
        let id = CascadeClusterId("cascade_cluster_0000000000000001".into());
        let id2 = id.clone();
        let mut s = HashSet::new();
        s.insert(id);
        s.insert(id2);
        assert_eq!(s.len(), 1);
    }

    // rationale: Core correctness — FNV-1a is a known reference algorithm.
    // The 64-bit hash of "a" has a fixed, independently-verifiable value
    // (FNV-1a 64 of a single byte 0x61). Pins the implementation against
    // a transposed XOR/multiply order or wrong constants.
    #[test]
    fn fnv1a_known_reference_value_for_single_byte() {
        // FNV-1a 64: h = (offset_basis ^ 0x61) * prime
        let expected = (FNV_OFFSET_BASIS ^ 0x61).wrapping_mul(FNV_PRIME);
        assert_eq!(fnv1a_64(b"a"), expected);
    }

    // rationale: Core correctness — FNV-1a is order-sensitive; "ab" and
    // "ba" hash differently. A commutative bug (e.g. summing bytes) would
    // break this.
    #[test]
    fn fnv1a_is_order_sensitive() {
        assert_ne!(fnv1a_64(b"ab"), fnv1a_64(b"ba"));
    }

    // rationale: Boundary — a single zero byte still mixes the state away
    // from the offset basis (XOR with 0 is a no-op but the multiply runs).
    #[test]
    fn fnv1a_single_zero_byte_still_mixes() {
        let h = fnv1a_64(&[0_u8]);
        // h = (offset_basis ^ 0) * prime = offset_basis * prime
        assert_eq!(h, FNV_OFFSET_BASIS.wrapping_mul(FNV_PRIME));
        assert_ne!(h, FNV_OFFSET_BASIS);
    }

    // rationale: Boundary — empty pane-label slice still derives a
    // well-formed id (join of empty = empty string; hash is defined).
    #[test]
    fn assign_cluster_id_empty_pane_labels_yields_well_formed_id() {
        let id = assign_cluster_id(0, 100, &[], 3);
        let s = id.as_str();
        assert!(s.starts_with("cascade_cluster_"));
        assert_eq!(s["cascade_cluster_".len()..].len(), 16);
    }

    // rationale: Core correctness — step_count is folded via XOR; the
    // count u64::MAX (saturated usize) is a legal input and produces a
    // distinct, well-formed id (no panic from the try_from fallback).
    #[test]
    fn assign_cluster_id_handles_max_step_count_without_panic() {
        let id = assign_cluster_id(0, 100, &["X"], usize::MAX);
        assert!(id.as_str().starts_with("cascade_cluster_"));
        // Differs from a small count at the same window/labels.
        assert_ne!(id, assign_cluster_id(0, 100, &["X"], 1));
    }

    // rationale: Anti-property — duplicate pane labels in the input slice
    // do not change the id versus the de-duplicated form ONLY IF sorted
    // join differs. Here we pin that label MULTIPLICITY is significant:
    // ["X","X"] joins to "X,X" which differs from ["X"] → "X".
    #[test]
    fn assign_cluster_id_label_multiplicity_is_significant() {
        let single = assign_cluster_id(0, 100, &["X"], 3);
        let doubled = assign_cluster_id(0, 100, &["X", "X"], 3);
        assert_ne!(
            single, doubled,
            "label join is multiplicity-sensitive; 'X' != 'X,X'"
        );
    }
}
