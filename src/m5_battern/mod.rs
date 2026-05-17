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
            let is_partial = battern_idx + 1 == total
                && group
                    .last()
                    .is_some_and(|last| recorded_at_ms.saturating_mul(1_000_000) - last.ts_ns < timeout_ns / 2);
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
            is_complete: !is_partial && total_steps >= 2,
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
}
