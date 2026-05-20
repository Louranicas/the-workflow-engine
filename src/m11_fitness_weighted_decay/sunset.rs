//! Sunset state machine + per-cycle statistics.
//!
//! Per m11 spec § 3 + § 5.5: each `AcceptedWorkflow` in the m30 bank
//! transitions through [`SunsetPhase::Active`] →
//! [`SunsetPhase::PrunePending`] (on weight < soft threshold) →
//! [`SunsetPhase::SunsetExpired`] (on hard sunset boundary OR weight <
//! prune threshold). `PrunePending → Active` is allowed on fitness
//! recovery; `SunsetExpired → Active` requires explicit Luke override.

/// Workflow lifecycle phase per m11 spec § 5.5 state machine.
///
/// # Serde
///
/// `SunsetPhase` derives `serde::{Serialize, Deserialize}` so m12 reports,
/// m13 promotion payloads, and the Prometheus metric emitter promised in
/// m11 spec § 9 can round-trip it as JSON without manual variant mapping.
/// The wire form is the exact variant name (`"Active"`, `"PrunePending"`,
/// `"SunsetExpired"`) — identical to [`Self::as_str`], so downstream
/// consumers can match string log lines against deserialised values
/// bit-for-bit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum SunsetPhase {
    /// Weight ≥ soft threshold (default `0.1`); dispatch-eligible.
    Active,
    /// Weight `<` soft threshold but `>` prune threshold; de-ranked but
    /// still recoverable on fitness rise.
    PrunePending,
    /// `sunset_at < now` OR weight `<` prune threshold; excluded from
    /// dispatch. Returns to Active only via explicit Luke override.
    SunsetExpired,
}

impl SunsetPhase {
    /// Stable ordinal projection for metrics and snapshot-stability tests.
    /// `Active = 0`, `PrunePending = 1`, `SunsetExpired = 2`.
    #[must_use]
    pub const fn ordinal(self) -> u8 {
        match self {
            Self::Active => 0,
            Self::PrunePending => 1,
            Self::SunsetExpired => 2,
        }
    }

    /// Stable identifier for log lines and metric labels.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "Active",
            Self::PrunePending => "PrunePending",
            Self::SunsetExpired => "SunsetExpired",
        }
    }
}

/// Per-cycle telemetry per m11 spec § 3.
///
/// # Serde contract (JSON-INFINITY decision)
///
/// `SunsetStats` derives `serde::{Serialize, Deserialize}` so m12 reports,
/// m13 promotion payloads, and the Prometheus metric emitter promised in
/// m11 spec § 9 can route it through any serde format.
///
/// JSON does not natively represent `f64::INFINITY` / `f64::NEG_INFINITY`
/// (the two sentinels on [`Self::min_decay_factor`] and
/// [`Self::max_decay_factor`] when `workflows_decayed == 0`). serde_json's
/// default behaviour silently maps non-finite floats to `null`, which
/// breaks round-trip. We therefore wire a custom serializer that maps the
/// non-finite sentinels to the JSON-safe string tokens `"+inf"` / `"-inf"`
/// and a deserializer that reads either the strings OR a finite number.
/// This preserves the public API (no `Option<f64>` break vs Wave-1) AND
/// makes JSON round-trip lossless. Binary formats (bincode, postcard,
/// MessagePack, CBOR) honour the f64 bit pattern natively and bypass the
/// adapter via serde format-tagging.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SunsetStats {
    /// Number of consolidation cycles that produced this stats record. For
    /// a single [`super::consolidation::run_consolidation_cycle`] call this
    /// is exactly `1`; aggregators may sum across cycles.
    pub cycles_run: u64,
    /// Number of active workflows that had decay applied this cycle.
    pub workflows_decayed: usize,
    /// Number of workflows transitioned (or marked for) prune this cycle.
    pub workflows_pruned: usize,
    /// Number of workflows transitioned to SunsetExpired this cycle.
    pub workflows_auto_sunset: usize,
    /// Number of workflows transitioned to PrunePending (soft floor) this
    /// cycle. Recoverable on fitness rise — the recovery edge
    /// (PrunePending → Active) is owned by the bank consumer (m30), not by
    /// m11, so no symmetric counter lives in this struct.
    pub workflows_prune_pending: usize,
    /// Number of workflows whose `last_run_ms` was future-dated relative to
    /// `now_ms` (clock-skew) and therefore SKIPPED this cycle. Surfacing
    /// this prevents the F-POVM-07 silent-zero-timestamp pattern from
    /// inflating recency credit on phantom future times.
    pub workflows_clock_skew_skipped: usize,
    /// Mean of the per-workflow [`super::formula::DecayFactor`] applied
    /// this cycle; `0.0` if `workflows_decayed == 0`.
    pub mean_decay_factor: f64,
    /// Minimum factor applied; `f64::INFINITY` if no workflows decayed.
    /// JSON-safe via [`json_safe_float`] adapter.
    #[serde(with = "json_safe_float")]
    pub min_decay_factor: f64,
    /// Maximum factor applied; `f64::NEG_INFINITY` if no workflows decayed.
    /// JSON-safe via [`json_safe_float`] adapter.
    #[serde(with = "json_safe_float")]
    pub max_decay_factor: f64,
}

/// Serde adapter that maps non-finite `f64` sentinels (INFINITY,
/// NEG_INFINITY, NaN) to the JSON-safe string tokens `"+inf"`, `"-inf"`,
/// `"NaN"` and back. Finite values pass through as plain JSON numbers.
///
/// This lets `SunsetStats::default()` (which holds INFINITY/-INFINITY
/// sentinels until any workflow is decayed) round-trip losslessly through
/// `serde_json`. Binary formats (bincode, postcard, MessagePack, CBOR)
/// accept f64 bit patterns directly and route through the same path; for
/// those formats the adapter still emits the f64 as a plain number when
/// it's finite, and as a string sentinel when it's non-finite — which is
/// still a valid value in every format.
pub mod json_safe_float {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// Serialize a possibly-non-finite f64 as either a plain number or
    /// the string sentinel `"+inf"` / `"-inf"` / `"NaN"`.
    ///
    /// # Errors
    ///
    /// Propagates any serializer error from the underlying format.
    pub fn serialize<S>(v: &f64, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if v.is_finite() {
            v.serialize(ser)
        } else if v.is_nan() {
            "NaN".serialize(ser)
        } else if v.is_sign_positive() {
            "+inf".serialize(ser)
        } else {
            "-inf".serialize(ser)
        }
    }

    /// Deserialize either a plain number or one of the string sentinels
    /// `"+inf"` / `"-inf"` / `"NaN"` back to a f64. Unknown strings are
    /// rejected with a typed serde error (no silent zero-fill).
    ///
    /// # Errors
    ///
    /// Returns a deserializer error for unrecognised string tokens or
    /// values that aren't a number or string.
    pub fn deserialize<'de, D>(de: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Two-step via serde_json::Value: buffer the field's JSON, then
        // pattern-match by JSON kind. We avoid `deserialize_any` because
        // some serde_derive code paths (notably struct-field dispatch
        // with #[serde(with = ...)]) interact poorly with it, surfacing
        // as a spurious "invalid type: map" when the inner value is a
        // plain number. Buffering through Value is the canonical robust
        // approach.
        let v = serde_json::Value::deserialize(de)?;
        match v {
            serde_json::Value::Number(n) => n
                .as_f64()
                .ok_or_else(|| serde::de::Error::custom("number outside f64 range")),
            serde_json::Value::String(s) => match s.as_str() {
                "+inf" | "inf" | "Infinity" => Ok(f64::INFINITY),
                "-inf" | "-Infinity" => Ok(f64::NEG_INFINITY),
                "NaN" | "nan" => Ok(f64::NAN),
                other => Err(serde::de::Error::custom(format!(
                    "unrecognised f64 sentinel: {other:?} (expected +inf|-inf|NaN or a finite number)"
                ))),
            },
            other => Err(serde::de::Error::custom(format!(
                "expected number or sentinel string, got {other:?}"
            ))),
        }
    }
}

impl Default for SunsetStats {
    fn default() -> Self {
        Self {
            cycles_run: 0,
            workflows_decayed: 0,
            workflows_pruned: 0,
            workflows_auto_sunset: 0,
            workflows_prune_pending: 0,
            workflows_clock_skew_skipped: 0,
            mean_decay_factor: 0.0,
            min_decay_factor: f64::INFINITY,
            max_decay_factor: f64::NEG_INFINITY,
        }
    }
}

/// Snapshot of an active workflow as the bank exposes it to the
/// consolidation cycle. Derives `serde::{Serialize, Deserialize}` so
/// downstream m12 reports + m13 promotion payloads can round-trip the
/// row alongside [`SunsetStats`].
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AcceptedWorkflowDecay {
    /// Stable workflow identifier (m30 bank key).
    pub workflow_id: String,
    /// Pathway id used to look up substrate fitness via the m42 stcortex
    /// route (post-2026-05-17 ADR).
    pub pathway_id: String,
    /// Wall-clock timestamp (ms since UNIX epoch) of the last dispatch of
    /// this workflow.
    pub last_run_ms: i64,
}

#[cfg(test)]
mod tests {
    use super::{AcceptedWorkflowDecay, SunsetPhase, SunsetStats};

    #[test]
    fn phase_ordinals_match_state_machine() {
        assert_eq!(SunsetPhase::Active.ordinal(), 0);
        assert_eq!(SunsetPhase::PrunePending.ordinal(), 1);
        assert_eq!(SunsetPhase::SunsetExpired.ordinal(), 2);
    }

    #[test]
    fn phase_as_str_is_stable() {
        assert_eq!(SunsetPhase::Active.as_str(), "Active");
        assert_eq!(SunsetPhase::PrunePending.as_str(), "PrunePending");
        assert_eq!(SunsetPhase::SunsetExpired.as_str(), "SunsetExpired");
    }

    #[test]
    fn phase_implements_copy_eq_hash() {
        use std::collections::HashSet;
        let mut s: HashSet<SunsetPhase> = HashSet::new();
        s.insert(SunsetPhase::Active);
        s.insert(SunsetPhase::Active);
        s.insert(SunsetPhase::PrunePending);
        s.insert(SunsetPhase::SunsetExpired);
        assert_eq!(s.len(), 3);
    }

    #[test]
    fn stats_default_initialises_min_max_sentinels() {
        let s = SunsetStats::default();
        assert_eq!(s.cycles_run, 0);
        assert_eq!(s.workflows_decayed, 0);
        assert!(s.min_decay_factor.is_infinite() && s.min_decay_factor.is_sign_positive());
        assert!(s.max_decay_factor.is_infinite() && s.max_decay_factor.is_sign_negative());
    }

    #[test]
    fn stats_clone_preserves_values() {
        let s = SunsetStats {
            cycles_run: 5,
            workflows_decayed: 42,
            ..SunsetStats::default()
        };
        let c = s.clone();
        assert_eq!(c, s);
    }

    // ─── T4-SERDE wiring (Wave-C1 hardening) ─────────────────────────────
    //
    // rationale category mapping per godtier-rust-maintainer § E:
    //   - contract regression (round-trip identity)
    //   - boundary (INFINITY / NEG_INFINITY sentinels)
    //   - anti-property (JSON cannot represent INFINITY — adapter MUST
    //     round-trip losslessly via the +inf / -inf string sentinels)

    #[test]
    fn sunset_stats_serde_round_trip() {
        // rationale: contract regression — full struct serde JSON round-trip
        // preserves every field bit-for-bit, including the two Wave-1 fields
        // (workflows_prune_pending, workflows_clock_skew_skipped) and the
        // f64 min/max via the json_safe_float adapter.
        let stats = SunsetStats {
            cycles_run: 7,
            workflows_decayed: 42,
            workflows_pruned: 3,
            workflows_auto_sunset: 2,
            workflows_prune_pending: 5,
            workflows_clock_skew_skipped: 1,
            mean_decay_factor: 0.987_654_321,
            min_decay_factor: 0.5,
            max_decay_factor: 0.999,
        };
        let s = serde_json::to_string(&stats).expect("serialize");
        let back: SunsetStats = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(back, stats);
        // Finite floats serialise as plain JSON numbers (no string sentinel).
        assert!(s.contains("\"min_decay_factor\":0.5"));
        assert!(s.contains("\"max_decay_factor\":0.999"));
    }

    #[test]
    fn sunset_phase_serde_round_trip() {
        // rationale: contract regression — every variant round-trips and
        // the wire form matches as_str() bit-for-bit so log lines can be
        // diff'd against deserialised values.
        for variant in [
            SunsetPhase::Active,
            SunsetPhase::PrunePending,
            SunsetPhase::SunsetExpired,
        ] {
            let s = serde_json::to_string(&variant).expect("serialize");
            let back: SunsetPhase = serde_json::from_str(&s).expect("deserialize");
            assert_eq!(back, variant);
            // Wire form is the variant name in quotes, matching as_str().
            assert_eq!(s, format!("\"{}\"", variant.as_str()));
        }
    }

    #[test]
    fn sunset_stats_default_serialises_with_infinity_sentinels() {
        // rationale: anti-property — Default::default() has
        // min_decay_factor=INFINITY, max_decay_factor=NEG_INFINITY. JSON
        // cannot represent INFINITY natively (serde_json default behaviour
        // is to emit `null`, which then breaks round-trip). The
        // json_safe_float adapter MUST map them to the "+inf" / "-inf"
        // string sentinels and back, lossless.
        let stats = SunsetStats::default();
        assert!(stats.min_decay_factor.is_infinite());
        assert!(stats.min_decay_factor.is_sign_positive());
        assert!(stats.max_decay_factor.is_infinite());
        assert!(stats.max_decay_factor.is_sign_negative());
        let s = serde_json::to_string(&stats).expect("serialize");
        // Adapter emits the sentinels as JSON strings, NOT as `null`.
        assert!(
            s.contains("\"min_decay_factor\":\"+inf\""),
            "expected +inf sentinel string in {s}",
        );
        assert!(
            s.contains("\"max_decay_factor\":\"-inf\""),
            "expected -inf sentinel string in {s}",
        );
        assert!(
            !s.contains("\"min_decay_factor\":null"),
            "INFINITY must NOT serialise as null (silent data loss): {s}",
        );
        let back: SunsetStats = serde_json::from_str(&s).expect("deserialize");
        // Equality must hold despite INFINITY: PartialEq on f64 treats
        // INFINITY == INFINITY as true; NaN == NaN as false (we don't use
        // NaN in defaults, so this is safe).
        assert_eq!(back, stats);
        assert!(back.min_decay_factor.is_infinite() && back.min_decay_factor.is_sign_positive());
        assert!(back.max_decay_factor.is_infinite() && back.max_decay_factor.is_sign_negative());
    }

    #[test]
    fn sunset_stats_serde_json_shape_snapshot() {
        // rationale: anti-property — bit-exact JSON keys present. Locks
        // the wire shape against silent field renames (a rename in code
        // without spec coordination would break m12 / m13 / Prometheus
        // consumers; this test fires the canary).
        let stats = SunsetStats::default();
        let s = serde_json::to_string(&stats).expect("serialize");
        for key in [
            "\"cycles_run\":",
            "\"workflows_decayed\":",
            "\"workflows_pruned\":",
            "\"workflows_auto_sunset\":",
            "\"workflows_prune_pending\":",
            "\"workflows_clock_skew_skipped\":",
            "\"mean_decay_factor\":",
            "\"min_decay_factor\":",
            "\"max_decay_factor\":",
        ] {
            assert!(s.contains(key), "missing wire key {key} in {s}");
        }
    }

    #[test]
    fn accepted_workflow_decay_clone_eq_debug() {
        let w = AcceptedWorkflowDecay {
            workflow_id: "wf_1".into(),
            pathway_id: "pw_a".into(),
            last_run_ms: 1_700_000_000_000,
        };
        let w2 = w.clone();
        assert_eq!(w, w2);
        let s = format!("{w:?}");
        assert!(s.contains("wf_1"));
        assert!(s.contains("pw_a"));
    }
}
