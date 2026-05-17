//! `m6_context_cost` — per-session token-cost proxy + exploration-rate
//! baseline EMA. F10 (exploration-cost preservation) exclusive owner.
//!
//! The baseline EMA exclusively tracks `Explored` / `Diverged` / `Unknown`
//! outcomes. Including `Converged` or `Repeated` would pull the baseline
//! toward exploitation, defeating the F10 mitigation. This exclusion is
//! the F10 core and is property-tested.

pub mod error;

use std::path::PathBuf;
use std::sync::Mutex;

pub use error::ContextCostError;

/// Workflow outcome classification (read back from m7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkflowOutcome {
    /// Converged on a known good workflow (exploitation).
    Converged,
    /// Repeated an existing pattern (exploitation).
    Repeated,
    /// Explored new ground (exploration).
    Explored,
    /// Diverged from prior patterns (exploration).
    Diverged,
    /// Outcome not yet classified by m7 (first-sweep bootstrap).
    Unknown,
}

impl WorkflowOutcome {
    /// **F10 invariant:** only `Explored`, `Diverged`, and `Unknown`
    /// contribute to the baseline EMA. `Converged` and `Repeated` are
    /// structurally excluded.
    #[must_use]
    pub const fn is_exploration(self) -> bool {
        matches!(self, Self::Explored | Self::Diverged | Self::Unknown)
    }

    /// Stable wire-form.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Converged => "Converged",
            Self::Repeated => "Repeated",
            Self::Explored => "Explored",
            Self::Diverged => "Diverged",
            Self::Unknown => "Unknown",
        }
    }
}

/// Cost band relative to the exploration baseline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CostBand {
    /// `total_cost < ema * below_threshold`.
    BelowBaseline,
    /// `ema * below_threshold <= total_cost < ema * above_threshold`.
    NearBaseline,
    /// `total_cost >= ema * above_threshold`.
    AboveBaseline,
}

impl CostBand {
    /// Stable identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BelowBaseline => "BelowBaseline",
            Self::NearBaseline => "NearBaseline",
            Self::AboveBaseline => "AboveBaseline",
        }
    }
}

/// Running exploration-rate baseline (EMA).
#[derive(Debug, Clone)]
pub struct ExplorationBaseline {
    /// Current EMA value; `None` until `n >= bootstrap_n`.
    pub ema: Option<f64>,
    /// Number of exploration sessions absorbed so far.
    pub n: usize,
    /// Smoothing factor: `alpha = 2 / (window + 1)`.
    pub alpha: f64,
}

impl ExplorationBaseline {
    /// Construct with the given EMA window (defaults to 20-session
    /// window per spec).
    #[must_use]
    pub fn new(window: usize) -> Self {
        let w = window.max(1);
        // f64 cast is safe for window < 2^53; habitat windows are < 1000.
        #[allow(
            clippy::cast_precision_loss,
            reason = "window is u-bounded < 2^53; precision irrelevant"
        )]
        let alpha = 2.0 / (w as f64 + 1.0);
        Self {
            ema: None,
            n: 0,
            alpha,
        }
    }

    /// **F10 gate:** update the EMA ONLY if `outcome.is_exploration()`.
    /// No-op for `Converged` / `Repeated`.
    pub fn update(&mut self, cost: i64, outcome: WorkflowOutcome) {
        if !outcome.is_exploration() {
            return;
        }
        #[allow(
            clippy::cast_precision_loss,
            reason = "cost is bounded by tool-call count per session; precision irrelevant"
        )]
        let sample = cost as f64;
        let next = match self.ema {
            Some(prev) => self.alpha.mul_add(sample - prev, prev),
            None => sample,
        };
        self.ema = Some(next);
        self.n = self.n.saturating_add(1);
    }

    /// Classify a cost relative to the current baseline. Returns `None`
    /// during bootstrap (`n < bootstrap_n`) or when the baseline is the
    /// degenerate zero EMA (which would otherwise collapse the band
    /// boundary to 0 and make every non-negative cost look `AboveBaseline`).
    /// Hardening: pre-CR-2 a zero EMA would have classified cost=0 as
    /// `AboveBaseline` since `0 >= 0` is true — that's a silent
    /// division-by-zero-class defect. We now refuse to classify when the
    /// EMA is non-positive.
    #[must_use]
    pub fn classify(
        &self,
        cost: i64,
        bootstrap_n: usize,
        below_threshold: f64,
        above_threshold: f64,
    ) -> Option<CostBand> {
        if self.n < bootstrap_n {
            return None;
        }
        let ema = self.ema?;
        // Degenerate baseline guard: a non-positive or non-finite EMA cannot
        // produce a meaningful band — refuse to classify.
        if !ema.is_finite() || ema <= 0.0 {
            return None;
        }
        #[allow(
            clippy::cast_precision_loss,
            reason = "see update()"
        )]
        let c = cost as f64;
        if c < ema * below_threshold {
            Some(CostBand::BelowBaseline)
        } else if c >= ema * above_threshold {
            Some(CostBand::AboveBaseline)
        } else {
            Some(CostBand::NearBaseline)
        }
    }
}

/// Per-session cost record.
#[derive(Debug, Clone)]
pub struct SessionCostRecord {
    /// Opaque session id.
    pub session_id: String,
    /// Proxy for input tokens: count of read/bash/grep/glob tool calls.
    pub token_cost_input_proxy: i64,
    /// Proxy for output tokens: count of write/edit tool calls.
    pub token_cost_output_proxy: i64,
    /// Sum of input + output proxies.
    pub total_cost_proxy: i64,
    /// Outcome from m7 join; `None` when session not yet classified.
    pub outcome: Option<WorkflowOutcome>,
    /// EMA value at record time; `None` during bootstrap.
    pub exploration_baseline: Option<f64>,
    /// Band classification; `None` during bootstrap.
    pub cost_band: Option<CostBand>,
    /// Wall-clock time of recording (ms).
    pub recorded_at_ms: i64,
}

/// Configuration.
#[derive(Debug, Clone)]
pub struct ContextCostRecordConfig {
    /// EMA window size; default 20.
    pub baseline_ema_window: usize,
    /// Bootstrap threshold; default 5 (no classification before n >= 5).
    pub baseline_bootstrap_n: usize,
    /// Below-baseline threshold; default 0.8.
    pub below_threshold: f64,
    /// Above-baseline threshold; default 1.2.
    pub above_threshold: f64,
    /// stcortex snapshot fallback path.
    pub stcortex_snapshot_path: Option<PathBuf>,
}

impl Default for ContextCostRecordConfig {
    fn default() -> Self {
        Self {
            baseline_ema_window: 20,
            baseline_bootstrap_n: 5,
            below_threshold: 0.8,
            above_threshold: 1.2,
            stcortex_snapshot_path: None,
        }
    }
}

/// The context-cost recorder.
pub struct ContextCostRecord {
    config: ContextCostRecordConfig,
    baseline: Mutex<ExplorationBaseline>,
}

impl std::fmt::Debug for ContextCostRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContextCostRecord")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl ContextCostRecord {
    /// Construct with the given configuration.
    #[must_use]
    pub fn new(config: ContextCostRecordConfig) -> Self {
        let baseline = ExplorationBaseline::new(config.baseline_ema_window);
        Self {
            config,
            baseline: Mutex::new(baseline),
        }
    }

    /// Borrow the configuration snapshot.
    #[must_use]
    pub fn config(&self) -> &ContextCostRecordConfig {
        &self.config
    }

    /// Update the baseline (if the outcome is exploration) and fill in
    /// `cost_band` + `exploration_baseline` on the record.
    ///
    /// Hardening: this method holds the baseline lock for the entire
    /// update-then-classify window, so the band returned in the record
    /// always reflects the same EMA that was just written. Previously two
    /// separate `lock()` calls left a race window in which a concurrent
    /// writer could move the EMA between update and classify.
    #[must_use]
    pub fn record_and_update_baseline(&self, mut record: SessionCostRecord) -> SessionCostRecord {
        // Single-lock atomic update-then-read. A poisoned mutex is treated
        // as a no-op (no baseline movement, no classification) rather than
        // silently replaced with a fresh baseline (which would mask the
        // failure mode).
        if let Ok(mut b) = self.baseline.lock() {
            if let Some(outcome) = record.outcome {
                b.update(record.total_cost_proxy, outcome);
            }
            record.exploration_baseline = b.ema;
            record.cost_band = b.classify(
                record.total_cost_proxy,
                self.config.baseline_bootstrap_n,
                self.config.below_threshold,
                self.config.above_threshold,
            );
        }
        record
    }

    /// Point-in-time snapshot of the baseline.
    #[must_use]
    pub fn baseline_snapshot(&self) -> ExplorationBaseline {
        self.baseline
            .lock()
            .map_or_else(|_| ExplorationBaseline::new(self.config.baseline_ema_window), |b| b.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ContextCostRecord, ContextCostRecordConfig, CostBand, ExplorationBaseline,
        SessionCostRecord, WorkflowOutcome,
    };

    fn now_ms() -> i64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .and_then(|d| i64::try_from(d.as_millis()).ok())
            .unwrap_or(0)
    }

    fn cost(session_id: &str, cost: i64, outcome: Option<WorkflowOutcome>) -> SessionCostRecord {
        SessionCostRecord {
            session_id: session_id.to_owned(),
            token_cost_input_proxy: cost / 2,
            token_cost_output_proxy: cost - cost / 2,
            total_cost_proxy: cost,
            outcome,
            exploration_baseline: None,
            cost_band: None,
            recorded_at_ms: now_ms(),
        }
    }

    // ---- WorkflowOutcome F10 gate (5) -----------------------------------

    #[test]
    fn is_exploration_truth_table() {
        assert!(!WorkflowOutcome::Converged.is_exploration());
        assert!(!WorkflowOutcome::Repeated.is_exploration());
        assert!(WorkflowOutcome::Explored.is_exploration());
        assert!(WorkflowOutcome::Diverged.is_exploration());
        assert!(WorkflowOutcome::Unknown.is_exploration());
    }

    #[test]
    fn outcome_as_str_stability() {
        assert_eq!(WorkflowOutcome::Explored.as_str(), "Explored");
        assert_eq!(WorkflowOutcome::Converged.as_str(), "Converged");
    }

    #[test]
    fn cost_band_as_str_stability() {
        assert_eq!(CostBand::BelowBaseline.as_str(), "BelowBaseline");
        assert_eq!(CostBand::NearBaseline.as_str(), "NearBaseline");
        assert_eq!(CostBand::AboveBaseline.as_str(), "AboveBaseline");
    }

    // ---- ExplorationBaseline F10 invariant (10) -------------------------

    #[test]
    fn new_window_zero_uses_window_one_for_safety() {
        let b = ExplorationBaseline::new(0);
        // alpha = 2 / (1 + 1) = 1.0 (degenerate but finite)
        assert!((b.alpha - 1.0).abs() < 1e-12);
    }

    #[test]
    fn new_window_twenty_yields_canonical_alpha() {
        let b = ExplorationBaseline::new(20);
        // alpha = 2 / 21
        assert!((b.alpha - (2.0 / 21.0)).abs() < 1e-12);
    }

    #[test]
    fn update_converged_is_no_op_f10_core() {
        let mut b = ExplorationBaseline::new(20);
        for _ in 0..100_u32 {
            b.update(1_000_000, WorkflowOutcome::Converged);
        }
        assert!(b.ema.is_none(), "F10: Converged must not move EMA");
        assert_eq!(b.n, 0);
    }

    #[test]
    fn update_repeated_is_no_op_f10_core() {
        let mut b = ExplorationBaseline::new(20);
        for _ in 0..100_u32 {
            b.update(1_000_000, WorkflowOutcome::Repeated);
        }
        assert!(b.ema.is_none(), "F10: Repeated must not move EMA");
        assert_eq!(b.n, 0);
    }

    #[test]
    fn update_explored_moves_ema() {
        let mut b = ExplorationBaseline::new(20);
        b.update(100, WorkflowOutcome::Explored);
        assert_eq!(b.ema, Some(100.0));
        assert_eq!(b.n, 1);
    }

    #[test]
    fn update_diverged_moves_ema() {
        let mut b = ExplorationBaseline::new(20);
        b.update(100, WorkflowOutcome::Diverged);
        assert_eq!(b.ema, Some(100.0));
        assert_eq!(b.n, 1);
    }

    #[test]
    fn update_unknown_moves_ema_for_bootstrap() {
        // First-sweep behaviour: Unknown counts toward exploration.
        let mut b = ExplorationBaseline::new(20);
        b.update(50, WorkflowOutcome::Unknown);
        assert!(b.ema.is_some());
        assert_eq!(b.n, 1);
    }

    #[test]
    fn ema_converges_to_constant_input_mean() {
        let mut b = ExplorationBaseline::new(20);
        for _ in 0..200_u32 {
            b.update(100, WorkflowOutcome::Explored);
        }
        let ema = b.ema.expect("ema");
        assert!((ema - 100.0).abs() < 1.0, "ema {ema} should approach 100");
    }

    #[test]
    fn classify_returns_none_during_bootstrap() {
        let mut b = ExplorationBaseline::new(20);
        b.update(100, WorkflowOutcome::Explored);
        assert!(b.classify(100, 5, 0.8, 1.2).is_none());
    }

    #[test]
    fn classify_returns_some_after_bootstrap() {
        let mut b = ExplorationBaseline::new(20);
        for _ in 0..6_u32 {
            b.update(100, WorkflowOutcome::Explored);
        }
        // ema ≈ 100; cost=100 → NearBaseline
        let band = b.classify(100, 5, 0.8, 1.2);
        assert_eq!(band, Some(CostBand::NearBaseline));
    }

    #[test]
    fn classify_below_above_boundaries() {
        let mut b = ExplorationBaseline::new(20);
        for _ in 0..10_u32 {
            b.update(100, WorkflowOutcome::Explored);
        }
        let ema = b.ema.unwrap();
        // Construct integer test costs derived from the f64 ema; safe at
        // habitat scale (ema is bounded by tool-call count).
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "ema is bounded by tool-call count; truncation is intentional integer construction"
        )]
        let (below, above) = ((ema * 0.5) as i64, (ema * 2.0) as i64);
        assert_eq!(b.classify(below, 5, 0.8, 1.2), Some(CostBand::BelowBaseline));
        assert_eq!(b.classify(above, 5, 0.8, 1.2), Some(CostBand::AboveBaseline));
    }

    // ---- F10 property invariants (3) -----------------------------------

    #[test]
    fn property_f10_gate_idempotent_under_repeat() {
        let mut b1 = ExplorationBaseline::new(20);
        b1.update(100, WorkflowOutcome::Converged);
        let b2 = ExplorationBaseline::new(20);
        // F10: Converged update is a no-op, so b1 and b2 are identical.
        assert_eq!(b1.ema, b2.ema);
        assert_eq!(b1.n, b2.n);
    }

    #[test]
    fn property_100_converged_then_5_explored_baseline_reflects_only_explored() {
        let mut b = ExplorationBaseline::new(20);
        for _ in 0..100_u32 {
            b.update(1_000_000, WorkflowOutcome::Converged);
        }
        for _ in 0..5_u32 {
            b.update(50, WorkflowOutcome::Explored);
        }
        let ema = b.ema.expect("ema");
        assert!(
            ema < 1_000_000.0 / 2.0,
            "F10: Converged cost {} should NOT have leaked into EMA {ema}",
            1_000_000
        );
        assert!(ema > 10.0, "ema close to exploration mean");
    }

    #[test]
    fn property_cost_band_none_iff_baseline_none() {
        let b = ExplorationBaseline::new(20);
        assert!(b.classify(100, 5, 0.8, 1.2).is_none());
        let mut c = ExplorationBaseline::new(20);
        for _ in 0..5_u32 {
            c.update(100, WorkflowOutcome::Explored);
        }
        assert!(c.classify(100, 5, 0.8, 1.2).is_some());
    }

    // ---- ContextCostRecord wiring (5) ----------------------------------

    #[test]
    fn record_and_update_no_outcome_does_not_move_baseline() {
        let r = ContextCostRecord::new(ContextCostRecordConfig::default());
        let _ = r.record_and_update_baseline(cost("s1", 100, None));
        assert!(r.baseline_snapshot().ema.is_none());
    }

    #[test]
    fn record_and_update_converged_does_not_move_baseline() {
        let r = ContextCostRecord::new(ContextCostRecordConfig::default());
        let _ = r.record_and_update_baseline(cost("s1", 1_000_000, Some(WorkflowOutcome::Converged)));
        assert!(r.baseline_snapshot().ema.is_none());
    }

    #[test]
    fn record_and_update_explored_moves_baseline() {
        let r = ContextCostRecord::new(ContextCostRecordConfig::default());
        let out = r.record_and_update_baseline(cost("s1", 100, Some(WorkflowOutcome::Explored)));
        assert!(out.exploration_baseline.is_some());
    }

    #[test]
    fn cost_band_appears_after_bootstrap() {
        let r = ContextCostRecord::new(ContextCostRecordConfig::default());
        for i in 0..6_u32 {
            let _ = r.record_and_update_baseline(cost(&format!("s{i}"), 100, Some(WorkflowOutcome::Explored)));
        }
        let out = r.record_and_update_baseline(cost("final", 200, Some(WorkflowOutcome::Explored)));
        assert!(out.cost_band.is_some());
    }

    #[test]
    fn baseline_snapshot_is_independent_of_record() {
        let r = ContextCostRecord::new(ContextCostRecordConfig::default());
        let _ = r.record_and_update_baseline(cost("s1", 100, Some(WorkflowOutcome::Explored)));
        let snap1 = r.baseline_snapshot();
        let _ = r.record_and_update_baseline(cost("s2", 200, Some(WorkflowOutcome::Explored)));
        let snap2 = r.baseline_snapshot();
        // Both have ema; snap2.n > snap1.n.
        assert!(snap2.n > snap1.n);
    }

    // ---- Defaults (2) ---------------------------------------------------

    #[test]
    fn default_config_matches_spec() {
        let c = ContextCostRecordConfig::default();
        assert_eq!(c.baseline_ema_window, 20);
        assert_eq!(c.baseline_bootstrap_n, 5);
        assert!((c.below_threshold - 0.8).abs() < 1e-12);
        assert!((c.above_threshold - 1.2).abs() < 1e-12);
    }

    #[test]
    fn debug_shows_record() {
        let r = ContextCostRecord::new(ContextCostRecordConfig::default());
        let s = format!("{r:?}");
        assert!(s.contains("ContextCostRecord"));
    }

    // ---- Hardening pass: anti-property F10 + adversarial + concurrency (10)

    // rationale: Adversarial input — NaN cost is never produced by the
    // codebase, but the EMA must not be poisoned by a deliberate or
    // wrap-induced NaN. (We cast `cost: i64` → f64 so NaN can't enter via
    // sample, but a future ExplorationBaseline mutation could; this
    // regression-pins the current well-defined behaviour.)
    #[test]
    fn classify_rejects_non_finite_ema() {
        // Manually construct a baseline with NaN ema (can only happen via
        // future bug; this pins the classify guard).
        let mut b = ExplorationBaseline::new(20);
        for _ in 0..6_u32 {
            b.update(100, WorkflowOutcome::Explored);
        }
        b.ema = Some(f64::NAN);
        assert!(b.classify(100, 5, 0.8, 1.2).is_none(), "NaN ema must refuse classify");
        b.ema = Some(f64::INFINITY);
        assert!(b.classify(100, 5, 0.8, 1.2).is_none(), "Inf ema must refuse classify");
    }

    // rationale: Anti-property F10 — empty-cohort / zero-cost EMA must NOT
    // be classified as `AboveBaseline` for a zero cost. The pre-hardening
    // code would have, because `0 >= 0 * 1.2` is `0 >= 0` is `true`.
    #[test]
    fn classify_rejects_zero_ema_to_prevent_division_by_zero_collapse() {
        let mut b = ExplorationBaseline::new(20);
        for _ in 0..6_u32 {
            b.update(0, WorkflowOutcome::Explored);
        }
        // EMA = 0.0 (degenerate). Pre-hardening this returned Some(AboveBaseline)
        // for cost=0; post-hardening it returns None.
        assert!(b.classify(0, 5, 0.8, 1.2).is_none(), "zero-ema must refuse classify");
    }

    // rationale: Anti-property F10 — A burst of 1000 Converged updates
    // followed by a single Explored update must produce an EMA equal to
    // that one Explored sample (Converged never contributed).
    #[test]
    fn property_burst_converged_then_one_explored_baseline_is_explored_sample() {
        let mut b = ExplorationBaseline::new(20);
        for _ in 0..1000_u32 {
            b.update(999_999, WorkflowOutcome::Converged);
        }
        b.update(42, WorkflowOutcome::Explored);
        assert_eq!(b.ema, Some(42.0));
        assert_eq!(b.n, 1);
    }

    // rationale: Anti-property F10 — Repeated never contributes regardless
    // of how cost trends.
    #[test]
    fn property_repeated_never_contributes_even_with_huge_cost() {
        let mut b = ExplorationBaseline::new(20);
        b.update(i64::MAX, WorkflowOutcome::Repeated);
        assert!(b.ema.is_none());
        assert_eq!(b.n, 0);
    }

    // rationale: Boundary — i64::MIN cost on an Explored outcome casts to
    // a finite f64 (no overflow); EMA is finite and bounded.
    #[test]
    fn update_handles_i64_min_cost_without_nan_or_inf() {
        let mut b = ExplorationBaseline::new(20);
        b.update(i64::MIN, WorkflowOutcome::Explored);
        let ema = b.ema.expect("ema");
        assert!(ema.is_finite(), "i64::MIN cost should produce finite ema");
    }

    // rationale: Boundary — i64::MAX cost on Explored outcome stays finite.
    #[test]
    fn update_handles_i64_max_cost_without_nan_or_inf() {
        let mut b = ExplorationBaseline::new(20);
        b.update(i64::MAX, WorkflowOutcome::Explored);
        let ema = b.ema.expect("ema");
        assert!(ema.is_finite(), "i64::MAX cost should produce finite ema");
    }

    // rationale: Concurrency — record_and_update_baseline is atomic
    // across threads (post-hardening). N threads contributing N samples
    // each yield exactly N*N exploration counts (no lost updates).
    #[test]
    fn concurrent_record_and_update_no_lost_updates() {
        use std::sync::Arc;
        use std::thread;
        let r = Arc::new(ContextCostRecord::new(ContextCostRecordConfig::default()));
        let threads = 8_u32;
        let per_thread = 50_u32;
        let mut handles = Vec::with_capacity(threads as usize);
        for t in 0..threads {
            let r2 = Arc::clone(&r);
            handles.push(thread::spawn(move || {
                for i in 0..per_thread {
                    let _ = r2.record_and_update_baseline(cost(
                        &format!("t{t}s{i}"),
                        100,
                        Some(WorkflowOutcome::Explored),
                    ));
                }
            }));
        }
        for h in handles {
            h.join().expect("thread");
        }
        let snap = r.baseline_snapshot();
        let expected = (threads * per_thread) as usize;
        assert_eq!(snap.n, expected, "expected {expected} updates, got {}", snap.n);
    }

    // rationale: Anti-property F10 — recorded band reflects the EMA
    // recorded in the SAME atomic update (no time-of-check/time-of-use gap).
    #[test]
    fn record_and_update_band_matches_ema_atomically() {
        let r = ContextCostRecord::new(ContextCostRecordConfig::default());
        for i in 0..6_u32 {
            let _ = r.record_and_update_baseline(cost(
                &format!("s{i}"),
                100,
                Some(WorkflowOutcome::Explored),
            ));
        }
        let out = r.record_and_update_baseline(cost(
            "probe",
            100,
            Some(WorkflowOutcome::Explored),
        ));
        // EMA is some, band is some, and band classification matches the
        // ema we got back in the same record.
        let ema = out.exploration_baseline.expect("ema");
        let band = out.cost_band.expect("band");
        // cost == 100, ema near 100 → NearBaseline
        assert_eq!(band, CostBand::NearBaseline);
        assert!((ema - 100.0).abs() < 50.0);
    }

    // rationale: Contract regression — WorkflowOutcome::is_exploration is
    // const-fn and pure; the compile-time evaluation below proves the
    // function is usable in `const` context (the surface-stability
    // invariant). Runtime calls then re-confirm the runtime path agrees.
    #[test]
    fn is_exploration_is_const_pure() {
        const E: bool = WorkflowOutcome::Explored.is_exploration();
        const C: bool = WorkflowOutcome::Converged.is_exploration();
        assert_eq!(E, WorkflowOutcome::Explored.is_exploration());
        assert_eq!(C, WorkflowOutcome::Converged.is_exploration());
        assert!(WorkflowOutcome::Explored.is_exploration());
        assert!(!WorkflowOutcome::Converged.is_exploration());
    }

    // rationale: Cross-module surface invariant — CostBand variants enum
    // is exactly three (defensive against silent enum growth).
    #[test]
    fn cost_band_has_exactly_three_variants() {
        // Exhaustive match; compile error if a new variant is added.
        for b in [CostBand::BelowBaseline, CostBand::NearBaseline, CostBand::AboveBaseline] {
            match b {
                CostBand::BelowBaseline | CostBand::NearBaseline | CostBand::AboveBaseline => {}
            }
        }
    }
}
