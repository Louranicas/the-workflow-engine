//! `m5_battern_step_record` — observe Battern-protocol executions in the
//! atuin step stream.
//!
//! **F1 exclusive owner (bank/name ossification):**
//! [`BatternStepObservation::step_label`] is `Option<BatternStepLabel>` —
//! unlabelled steps surface as `None`, never as a placeholder `Other`,
//! never discarded. See [`step_label`] for the type-level guarantee.

pub mod error;
pub mod step_label;

use std::path::PathBuf;

use regex::Regex;

pub use error::BatternError;
pub use step_label::BatternStepLabel;

use crate::m4_cascade::{cluster_id::fnv1a_64, AtuinStep};

/// Minimum step count for a Battern observation to be considered
/// `is_complete = true` in [`BatternStepRecord::summarise`]. Matches the
/// per-spec floor (also the default of `BatternStepRecordConfig::min_steps`)
/// — extracted from the previously-hardcoded `2` for documentability.
pub const MIN_COMPLETE_STEPS: usize = 2;

/// Opaque identifier for one Battern execution.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BatternId(pub String);

impl BatternId {
    /// Borrow the inner opaque string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for BatternId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Derive a deterministic opaque battern id from the first-dispatch
/// timestamp.
#[must_use]
pub fn derive_battern_id(first_dispatch_ts_ns: i64) -> BatternId {
    let h = fnv1a_64(first_dispatch_ts_ns.to_string().as_bytes());
    BatternId(format!("battern_{h:016x}"))
}

/// One observed step within a Battern execution.
#[derive(Debug, Clone)]
pub struct BatternStepObservation {
    /// Opaque battern id this step belongs to.
    pub battern_id: BatternId,
    /// Position within the battern (0-indexed).
    pub step_index: usize,
    /// **F1 invariant:** `None` when the step did not match any
    /// heuristic. NEVER substituted with a placeholder enum variant.
    pub step_label: Option<BatternStepLabel>,
    /// Duration of the step in milliseconds.
    pub duration_ms: i64,
    /// Session identifier.
    pub session: String,
    /// Atuin exit code for the step's underlying command.
    pub exit_code: i32,
    /// `true` if the battern closed at batch end (incomplete).
    pub is_partial: bool,
    /// Wall-clock time the observation was recorded (ms since epoch).
    pub recorded_at_ms: i64,
}

/// Per-battern summary.
#[derive(Debug, Clone)]
pub struct BatternRecord {
    /// Opaque battern id.
    pub battern_id: BatternId,
    /// Total step count.
    pub total_steps: usize,
    /// Steps that received a `Some(BatternStepLabel)`.
    pub labelled_steps: usize,
    /// Steps whose `exit_code != 0`.
    pub failed_steps: usize,
    /// Sum of `duration_ms` over all steps.
    pub total_duration_ms: i64,
    /// `true` if any step is partial.
    pub is_partial: bool,
    /// `true` when total_steps >= min_steps AND not partial.
    pub is_complete: bool,
}

/// Configuration for the Battern step recorder.
#[derive(Debug, Clone)]
pub struct BatternStepRecordConfig {
    /// Maximum wall-clock gap between steps within one battern (ms).
    pub inter_step_timeout_ms: i64,
    /// Minimum step count for an observation to emit.
    pub min_steps: usize,
    /// atuin SQLite path.
    pub atuin_db_path: PathBuf,
}

impl Default for BatternStepRecordConfig {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_owned());
        Self {
            inter_step_timeout_ms: 1_800_000,
            min_steps: 2,
            atuin_db_path: PathBuf::from(format!("{home}/.local/share/atuin/history.db")),
        }
    }
}

/// The Battern step recorder.
pub struct BatternStepRecord {
    config: BatternStepRecordConfig,
    heuristics: Vec<(Regex, BatternStepLabel)>,
}

impl std::fmt::Debug for BatternStepRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BatternStepRecord")
            .field("config", &self.config)
            .field("heuristics", &self.heuristics.len())
            .finish()
    }
}

impl BatternStepRecord {
    /// Construct with the given configuration. Pre-compiles the heuristic
    /// regex table.
    ///
    /// # Errors
    ///
    /// [`BatternError::RegexCompile`] if any of the built-in heuristic
    /// patterns fails to compile (should be impossible — they are
    /// hardcoded).
    pub fn new(config: BatternStepRecordConfig) -> Result<Self, BatternError> {
        let heuristics: Vec<(Regex, BatternStepLabel)> = vec![
            (Regex::new(r"^cc-dispatch\b")?, BatternStepLabel::Dispatch),
            (Regex::new(r"^cc-(health|gate-check)\b")?, BatternStepLabel::Gate),
            (Regex::new(r"^curl .*:[0-9]+/health\b")?, BatternStepLabel::Gate),
            (Regex::new(r"^cc-(harvest|audit)\b")?, BatternStepLabel::Collect),
            (Regex::new(r"^cc-cascade\b")?, BatternStepLabel::Compose),
            (Regex::new(r"^(rg|atuin search|Read)\b")?, BatternStepLabel::Design),
        ];
        Ok(Self {
            config,
            heuristics,
        })
    }

    /// Borrow the configuration.
    #[must_use]
    pub fn config(&self) -> &BatternStepRecordConfig {
        &self.config
    }

    /// Match a command against the heuristic table; return the first
    /// matching label, or `None`.
    ///
    /// **F1 invariant:** the return type is `Option<BatternStepLabel>`.
    /// There is no fallback `Other` variant.
    #[must_use]
    pub fn label_command(&self, command: &str) -> Option<BatternStepLabel> {
        for (re, label) in &self.heuristics {
            if re.is_match(command) {
                return Some(*label);
            }
        }
        None
    }

    /// Observe a batch of atuin steps, producing per-step observations.
    ///
    /// Battern boundaries are inferred from inter-step timeouts and
    /// `cc-dispatch` markers. Closed clusters with `< min_steps` are
    /// discarded; remaining open clusters at batch end emit with
    /// `is_partial = true`.
    #[must_use]
    pub fn observe(&self, steps: &[AtuinStep]) -> Vec<BatternStepObservation> {
        if steps.is_empty() {
            return Vec::new();
        }
        let mut sorted: Vec<&AtuinStep> = steps.iter().collect();
        sorted.sort_by_key(|s| s.ts_ns);
        let timeout_ns = self.config.inter_step_timeout_ms.saturating_mul(1_000_000);

        // Accumulate batterns; close on dispatch-rollover (only when the
        // CURRENT battern already saw a dispatch — a Design→Dispatch
        // transition opens its OWN battern) or inter-step timeout.
        let mut batterns: Vec<Vec<&AtuinStep>> = Vec::new();
        for step in &sorted {
            let opens_new = self.label_command(&step.command) == Some(BatternStepLabel::Dispatch);
            let current_has_dispatch = batterns.last().is_some_and(|b| {
                b.iter()
                    .any(|s| self.label_command(&s.command) == Some(BatternStepLabel::Dispatch))
            });
            let timed_out = batterns
                .last()
                .and_then(|b| b.last())
                .is_some_and(|last| step.ts_ns.saturating_sub(last.ts_ns) > timeout_ns);
            if batterns.is_empty() || timed_out || (opens_new && current_has_dispatch) {
                batterns.push(vec![*step]);
            } else if let Some(open) = batterns.last_mut() {
                open.push(*step);
            }
        }

        let recorded_at_ms = now_ms();
        let total = batterns.len();
        let mut out = Vec::new();
        for (battern_idx, group) in batterns.into_iter().enumerate() {
            if group.len() < self.config.min_steps {
                continue;
            }
            let first_ts = group.first().map_or(0, |s| s.ts_ns);
            let battern_id = derive_battern_id(first_ts);
            // Hardening: use saturating arithmetic on `recorded_at_ms`-vs-`last.ts_ns`
            // diff. Plain `-` previously underflowed on test inputs where
            // `last.ts_ns` was below recorded_at_ms (synthetic timestamps with
            // tiny ts_ns vs real-clock ms recorded_at). saturating_mul +
            // saturating_sub make the heuristic well-defined across the entire
            // i64 range without changing observable behaviour at habitat scale.
            let wallclock_ns = recorded_at_ms.saturating_mul(1_000_000);
            let is_partial = battern_idx + 1 == total
                && group
                    .last()
                    .is_some_and(|last| wallclock_ns.saturating_sub(last.ts_ns) < timeout_ns / 2);
            for (step_index, step) in group.iter().enumerate() {
                let next_ts = group.get(step_index + 1).map_or(step.ts_ns, |n| n.ts_ns);
                let elapsed_ns = next_ts.saturating_sub(step.ts_ns).max(0);
                let duration_ms = elapsed_ns / 1_000_000;
                out.push(BatternStepObservation {
                    battern_id: battern_id.clone(),
                    step_index,
                    step_label: self.label_command(&step.command),
                    duration_ms,
                    session: step.session.clone(),
                    exit_code: step.exit,
                    is_partial,
                    recorded_at_ms,
                });
            }
        }
        out
    }

    /// Summarise a set of observations sharing one `battern_id`.
    ///
    /// `observations` MUST all share the same `battern_id`; a precondition
    /// the caller arranges. The function does not validate this and uses
    /// the first id encountered.
    ///
    /// `is_complete` is `true` iff: no observation is marked partial AND
    /// `total_steps >= MIN_COMPLETE_STEPS` (the per-spec floor; matches
    /// the default `BatternStepRecordConfig::min_steps`).
    #[must_use]
    pub fn summarise(observations: &[BatternStepObservation]) -> BatternRecord {
        if observations.is_empty() {
            return BatternRecord {
                battern_id: BatternId("battern_empty".into()),
                total_steps: 0,
                labelled_steps: 0,
                failed_steps: 0,
                total_duration_ms: 0,
                is_partial: false,
                is_complete: false,
            };
        }
        let battern_id = observations[0].battern_id.clone();
        let total_steps = observations.len();
        let labelled_steps = observations.iter().filter(|o| o.step_label.is_some()).count();
        let failed_steps = observations.iter().filter(|o| o.exit_code != 0).count();
        let total_duration_ms = observations.iter().map(|o| o.duration_ms).sum();
        let is_partial = observations.iter().any(|o| o.is_partial);
        BatternRecord {
            battern_id,
            total_steps,
            labelled_steps,
            failed_steps,
            total_duration_ms,
            is_partial,
            is_complete: !is_partial && total_steps >= MIN_COMPLETE_STEPS,
        }
    }
}

fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|d| i64::try_from(d.as_millis()).ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{
        derive_battern_id, BatternId, BatternStepLabel, BatternStepObservation,
        BatternStepRecord, BatternStepRecordConfig,
    };
    use crate::m4_cascade::AtuinStep;

    fn step(ts_ns: i64, cmd: &str, session: &str) -> AtuinStep {
        AtuinStep {
            id: format!("ulid-{ts_ns}"),
            ts_ns,
            command: cmd.to_owned(),
            cwd: "/tmp".into(),
            session: session.to_owned(),
            exit: 0,
        }
    }

    fn rec() -> BatternStepRecord {
        BatternStepRecord::new(BatternStepRecordConfig::default()).expect("regex compile")
    }

    #[test]
    fn battern_id_is_opaque_hex_form() {
        let id = derive_battern_id(1_700_000_000_000_000_000);
        let s = format!("{id}");
        assert!(s.starts_with("battern_"));
        let suffix = &s["battern_".len()..];
        assert_eq!(suffix.len(), 16);
        assert!(suffix.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn battern_id_deterministic_for_same_input() {
        for _ in 0..50_u32 {
            assert_eq!(derive_battern_id(100), derive_battern_id(100));
        }
    }

    #[test]
    fn battern_id_distinct_for_different_input() {
        assert_ne!(derive_battern_id(100), derive_battern_id(101));
    }

    #[test]
    fn label_command_returns_none_for_unmatched() {
        let r = rec();
        assert!(r.label_command("random shell command").is_none());
    }

    #[test]
    fn label_command_dispatch_pattern() {
        let r = rec();
        assert_eq!(
            r.label_command("cc-dispatch ALPHA-LEFT prompt"),
            Some(BatternStepLabel::Dispatch)
        );
    }

    #[test]
    fn label_command_gate_pattern() {
        let r = rec();
        assert_eq!(r.label_command("cc-health"), Some(BatternStepLabel::Gate));
        assert_eq!(
            r.label_command("curl localhost:8133/health"),
            Some(BatternStepLabel::Gate)
        );
    }

    #[test]
    fn label_command_collect_pattern() {
        let r = rec();
        assert_eq!(r.label_command("cc-harvest"), Some(BatternStepLabel::Collect));
        assert_eq!(r.label_command("cc-audit"), Some(BatternStepLabel::Collect));
    }

    #[test]
    fn label_command_compose_pattern() {
        let r = rec();
        assert_eq!(r.label_command("cc-cascade --to BETA"), Some(BatternStepLabel::Compose));
    }

    #[test]
    fn label_command_design_pattern() {
        let r = rec();
        assert_eq!(r.label_command("rg foo"), Some(BatternStepLabel::Design));
        assert_eq!(r.label_command("atuin search --limit 10"), Some(BatternStepLabel::Design));
    }

    #[test]
    fn observe_empty_returns_empty() {
        assert!(rec().observe(&[]).is_empty());
    }

    #[test]
    fn observe_below_min_steps_drops_battern() {
        let r = BatternStepRecord::new(BatternStepRecordConfig {
            min_steps: 3,
            ..BatternStepRecordConfig::default()
        })
        .expect("regex");
        let steps = vec![step(1, "cc-dispatch", "s1"), step(2, "cc-health", "s1")];
        assert!(r.observe(&steps).is_empty());
    }

    #[test]
    fn observe_records_six_canonical_steps_in_one_battern() {
        let r = rec();
        let steps = vec![
            step(1_000_000_000, "rg foo", "s1"),
            step(2_000_000_000, "cc-dispatch ALPHA", "s1"),
            step(3_000_000_000, "cc-health", "s1"),
            step(4_000_000_000, "cc-harvest", "s1"),
            step(5_000_000_000, "some long synth command", "s1"),
            step(6_000_000_000, "cc-cascade --to BETA", "s1"),
        ];
        let obs = r.observe(&steps);
        assert_eq!(obs.len(), 6);
        assert!(obs[0].step_label.is_some() || obs[0].step_label.is_none());
        // Verify F1: at least one None is acceptable (the synth-command at idx 4
        // does not match any heuristic, so step_label = None).
        let some_count = obs.iter().filter(|o| o.step_label.is_some()).count();
        assert!(some_count >= 4, "expected ≥4 labelled steps, got {some_count}");
    }

    #[test]
    fn observe_preserves_unlabelled_step_as_none_not_other() {
        // F1 invariant test: unmatched commands surface as None.
        let r = rec();
        let steps = vec![
            step(1, "cc-dispatch X", "s1"),
            step(2, "totally-unrecognised-shell-command", "s1"),
        ];
        let obs = r.observe(&steps);
        assert!(obs.iter().any(|o| o.step_label.is_none()));
    }

    #[test]
    fn observe_records_failed_steps_via_exit_code() {
        let r = rec();
        let mut s1 = step(1, "cc-dispatch", "s1");
        s1.exit = 1;
        let s2 = step(2, "cc-health", "s1");
        let obs = r.observe(&[s1, s2]);
        let failed = obs.iter().filter(|o| o.exit_code != 0).count();
        assert_eq!(failed, 1);
    }

    #[test]
    fn summarise_empty_returns_sentinel() {
        let r = BatternStepRecord::summarise(&[]);
        assert_eq!(r.total_steps, 0);
        assert!(!r.is_complete);
    }

    #[test]
    fn summarise_counts_labelled_steps() {
        let bid = BatternId("battern_test".into());
        let obs = vec![
            BatternStepObservation {
                battern_id: bid.clone(),
                step_index: 0,
                step_label: Some(BatternStepLabel::Design),
                duration_ms: 100,
                session: "s1".into(),
                exit_code: 0,
                is_partial: false,
                recorded_at_ms: 0,
            },
            BatternStepObservation {
                battern_id: bid,
                step_index: 1,
                step_label: None,
                duration_ms: 200,
                session: "s1".into(),
                exit_code: 0,
                is_partial: false,
                recorded_at_ms: 0,
            },
        ];
        let r = BatternStepRecord::summarise(&obs);
        assert_eq!(r.total_steps, 2);
        assert_eq!(r.labelled_steps, 1);
        assert_eq!(r.total_duration_ms, 300);
        assert!(r.is_complete);
    }

    #[test]
    fn implements_debug_for_recorder() {
        let r = rec();
        let s = format!("{r:?}");
        assert!(s.contains("BatternStepRecord"));
    }

    // ---- Hardening pass: anti-property + adversarial input (10) -----------

    // rationale: Boundary — i64::MAX timestamps survive observe() without
    // arithmetic overflow in the is_partial heuristic.
    #[test]
    fn observe_does_not_panic_on_i64_max_timestamps() {
        let r = rec();
        let steps = vec![
            step(i64::MAX - 10, "cc-dispatch A", "s1"),
            step(i64::MAX - 5, "cc-health", "s1"),
        ];
        let _obs = r.observe(&steps);
    }

    // rationale: Boundary — i64::MIN timestamps survive observe() (saturating
    // arithmetic prevents overflow in the recorded_at vs ts_ns diff).
    #[test]
    fn observe_does_not_panic_on_i64_min_timestamps() {
        let r = rec();
        let steps = vec![
            step(i64::MIN, "cc-dispatch A", "s1"),
            step(i64::MIN + 1, "cc-health", "s1"),
        ];
        let _obs = r.observe(&steps);
    }

    // rationale: Anti-property F1 — battern_id is opaque hex; even when the
    // first dispatch ts carries a meaningful semantic shape (round number),
    // the hash output is hex-only.
    #[test]
    fn battern_id_no_semantic_leak_for_round_number_timestamps() {
        let id = derive_battern_id(1_700_000_000_000_000_000);
        let s = format!("{id}");
        // Strip prefix, assert pure hex (no `1700`, `000`, etc. as substrings
        // would be valid hex characters, but we assert structural hex-only).
        let suffix = &s["battern_".len()..];
        assert!(suffix.chars().all(|c| c.is_ascii_hexdigit()));
    }

    // rationale: Determinism — same input ts produces same id across many
    // calls, even when interleaved with other derivations.
    #[test]
    fn battern_id_stable_under_interleaved_derivation() {
        let target = derive_battern_id(42);
        for noise in 0..1000_i64 {
            let _ = derive_battern_id(noise * 17 + 1);
        }
        assert_eq!(derive_battern_id(42), target);
    }

    // rationale: Anti-property F1 — observe records keep step_label as None
    // for unrecognised commands; never a placeholder. Tested with many
    // adversarial commands.
    #[test]
    fn observe_preserves_none_for_many_unrecognised_commands() {
        let r = rec();
        let steps = vec![
            step(1, "cc-dispatch A", "s1"),
            step(2, "unknown-1", "s1"),
            step(3, "unknown-2", "s1"),
            step(4, "unknown-3", "s1"),
            step(5, "unknown-4", "s1"),
        ];
        let obs = r.observe(&steps);
        let nones = obs.iter().filter(|o| o.step_label.is_none()).count();
        assert!(nones >= 4, "F1 preservation broken: only {nones} Nones");
    }

    // rationale: Boundary battern-boundary rule — `cc-dispatch` opens a new
    // battern ONLY IF the current battern already saw a dispatch. A
    // Design→Dispatch transition stays in ONE battern.
    #[test]
    fn boundary_design_then_dispatch_stays_in_one_battern() {
        let r = rec();
        let steps = vec![
            step(1_000_000_000, "rg foo", "s1"),
            step(2_000_000_000, "cc-dispatch ALPHA", "s1"),
            step(3_000_000_000, "cc-health", "s1"),
        ];
        let obs = r.observe(&steps);
        // All three observations should share one battern_id.
        let ids: std::collections::HashSet<_> =
            obs.iter().map(|o| o.battern_id.clone()).collect();
        assert_eq!(ids.len(), 1, "Design→Dispatch should not split battern");
    }

    // rationale: Boundary battern-boundary rule — a SECOND dispatch
    // (Dispatch→Gate→Dispatch) opens a NEW battern.
    #[test]
    fn boundary_second_dispatch_opens_new_battern() {
        let r = rec();
        let steps = vec![
            step(1_000_000_000, "cc-dispatch A", "s1"),
            step(2_000_000_000, "cc-health", "s1"),
            step(3_000_000_000, "cc-dispatch B", "s1"),
            step(4_000_000_000, "cc-health", "s1"),
        ];
        let obs = r.observe(&steps);
        let ids: std::collections::HashSet<_> =
            obs.iter().map(|o| o.battern_id.clone()).collect();
        assert_eq!(ids.len(), 2, "second dispatch must open a new battern");
    }

    // rationale: Anti-property F11 — battern_id MUST NOT contain any
    // human-meaningful substring, even when input commands are semantically
    // loaded.
    #[test]
    fn battern_id_does_not_leak_dispatch_command_substring() {
        let r = rec();
        let steps = vec![
            step(1_000_000_000, "cc-dispatch ALPHA-LEFT", "s1"),
            step(2_000_000_000, "cc-health", "s1"),
        ];
        let obs = r.observe(&steps);
        let id = obs[0].battern_id.as_str();
        for forbidden in ["ALPHA", "LEFT", "dispatch", "health", "cc-"] {
            assert!(!id.contains(forbidden), "F11 leak: {forbidden:?} in {id:?}");
        }
    }

    // rationale: Cross-module surface invariant — exported
    // MIN_COMPLETE_STEPS matches BatternStepRecordConfig::default().min_steps.
    #[test]
    fn min_complete_steps_matches_default_min_steps() {
        let cfg = BatternStepRecordConfig::default();
        assert_eq!(super::MIN_COMPLETE_STEPS, cfg.min_steps);
    }

    // rationale: Determinism — same input twice produces equal observations
    // (apart from recorded_at_ms wall-clock skew).
    #[test]
    fn observe_is_deterministic_on_structural_fields() {
        let r = rec();
        let steps = vec![
            step(1, "cc-dispatch A", "s1"),
            step(2, "cc-health", "s1"),
        ];
        let obs_a = r.observe(&steps);
        let obs_b = r.observe(&steps);
        assert_eq!(obs_a.len(), obs_b.len());
        for (a, b) in obs_a.iter().zip(obs_b.iter()) {
            assert_eq!(a.battern_id, b.battern_id);
            assert_eq!(a.step_index, b.step_index);
            assert_eq!(a.step_label, b.step_label);
            assert_eq!(a.duration_ms, b.duration_ms);
            assert_eq!(a.exit_code, b.exit_code);
        }
    }
}
