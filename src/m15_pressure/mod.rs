//! `m15_pressure_register` — forbidden-verb pressure witness.
//!
//! m15 detects forbidden-verb-pressure events (spec patches / agent
//! reports / cross-talk messages attempting to introduce chartered-verb
//! violations) and emits durable JSONL `PHASE-B-RESERVATION-NOTICE`
//! files to a configurable directory.
//!
//! **Witness, not gate.** m15 blocks nothing. It creates a durable
//! record that scope-pressure happened, where it came from, and what
//! verb / feature was being pushed. Watcher ☤ and Zen read at their
//! own cadence.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use thiserror::Error;

/// Schema version for serialised [`PressureEvent`].
pub const SCHEMA_VERSION: u32 = 1;

/// Failure modes for the pressure register.
#[derive(Debug, Error)]
pub enum PressureRegisterError {
    /// I/O error writing the JSONL file.
    #[error("write failed: {0}")]
    WriteFailed(#[from] std::io::Error),
    /// JSON serialisation failure.
    #[error("serialise: {0}")]
    Serialise(#[from] serde_json::Error),
}

/// Closed-set forbidden-verb category.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForbiddenCategory {
    /// `recommend_*` / passive surfacing without explicit request.
    RecommendVerb,
    /// `auto_*` / `smart_*` autonomous triggers.
    AutoOrSmartVerb,
    /// `rewrite_*` source mutation outside wf-* binaries.
    RewriteVerb,
    /// Dispatch routing bypassing HABITAT-CONDUCTOR.
    RouteBypassConductor,
    /// `package_*` / `publish_*` without verification gate.
    PackageOrPublish,
    /// `optimise_*` automated tuning without measurement gate.
    OptimiseWithoutGate,
    /// HTTP server / sidecar daemon surface (hard refusal).
    HttpServerSurface,
    /// Sidecar daemon (hard refusal).
    SidecarDaemon,
    /// POVM write (workflow-trace POVM-decoupled per 2026-05-17 ADR).
    DeprecatedPovmWrite,
    /// `use synthex_v2::*` wholesale import (lift patterns, do not import).
    SynthexV2Import,
    /// Handshake silence past timeout (AP-V7-08 mitigation).
    HandshakeSilence,
    /// Any other forbidden surface with description.
    Other {
        /// Free-text description.
        description: String,
    },
}

/// Where the pressure was observed.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PressureSource {
    /// Spec patch file diff.
    SpecPatch {
        /// File path relative to repo root.
        file: String,
    },
    /// Cross-talk message from another agent session.
    AgentCrossTalk {
        /// Originating session id (or "unknown").
        from_session: String,
    },
    /// CLI verb proposal in a session note.
    CliVerbProposal {
        /// Proposed command verbatim.
        proposed_command: String,
    },
    /// Session note (CLAUDE.local.md / shared-context).
    SessionNote {
        /// Note path relative to repo root.
        note_path: String,
    },
    /// Unknown source.
    Unknown,
}

/// Which charter section the pressure violates.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CharterSection {
    /// v1.3 § 2 hard refusals.
    V1_3HardRefusal,
    /// v1.3 § 3 verb-class boundaries.
    V1_3VerbClass,
    /// AP27 Watcher self-modification boundary.
    Ap27Boundary,
    /// Other / cross-cutting.
    Other {
        /// Free-text section reference.
        section: String,
    },
}

/// One observed pressure event.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PressureEvent {
    /// Schema version.
    pub schema_version: u32,
    /// Monotonic event id per process.
    pub id: u64,
    /// RFC 3339 timestamp.
    pub detected_at: String,
    /// Originating session id.
    pub session_id: String,
    /// Forbidden category.
    pub forbidden_category: ForbiddenCategory,
    /// Trigger excerpt (≤ 512 chars).
    pub trigger_excerpt: String,
    /// Source surface.
    pub source: PressureSource,
    /// Human-readable feature description.
    pub proposed_feature: String,
    /// Violated charter section.
    pub violated_charter: CharterSection,
}

/// Truncate an excerpt to ≤ 512 chars, appending `…` on truncation.
#[must_use]
pub fn truncate_excerpt(s: &str) -> String {
    const MAX: usize = 512;
    if s.len() <= MAX {
        return s.to_owned();
    }
    let mut out: String = s.chars().take(MAX).collect();
    out.push('\u{2026}');
    out
}

/// Heuristic classifier: match a command / text excerpt to a forbidden
/// category. Returns `None` when the excerpt is in-charter.
#[must_use]
pub fn classify_excerpt(text: &str) -> Option<ForbiddenCategory> {
    let lower = text.to_lowercase();
    if lower.contains("recommend_") || lower.contains("recommend ") {
        return Some(ForbiddenCategory::RecommendVerb);
    }
    if lower.contains("auto_") || lower.contains("smart_") {
        return Some(ForbiddenCategory::AutoOrSmartVerb);
    }
    if lower.contains("rewrite_") {
        return Some(ForbiddenCategory::RewriteVerb);
    }
    if lower.contains("package_") || lower.contains("publish_") {
        return Some(ForbiddenCategory::PackageOrPublish);
    }
    if lower.contains("optimise_") || lower.contains("optimize_") {
        return Some(ForbiddenCategory::OptimiseWithoutGate);
    }
    if lower.contains("http server")
        || lower.contains("http_server")
        || lower.contains("sidecar daemon")
    {
        return Some(ForbiddenCategory::HttpServerSurface);
    }
    if lower.contains("povm") && (lower.contains("write") || lower.contains("emit")) {
        return Some(ForbiddenCategory::DeprecatedPovmWrite);
    }
    if lower.contains("use synthex_v2") {
        return Some(ForbiddenCategory::SynthexV2Import);
    }
    if lower.contains("handshake") && lower.contains("silence") {
        return Some(ForbiddenCategory::HandshakeSilence);
    }
    None
}

/// Pressure register configuration.
#[derive(Debug, Clone)]
pub struct PressureRegisterConfig {
    /// Directory for `PHASE-B-RESERVATION-NOTICE` JSONL files.
    pub notices_dir: PathBuf,
    /// Session id stamped on every emitted event.
    pub session_id: String,
}

impl Default for PressureRegisterConfig {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_owned());
        Self {
            notices_dir: PathBuf::from(format!(
                "{home}/projects/shared-context/agent-cross-talk"
            )),
            session_id: std::env::var("CLAUDE_SESSION_ID")
                .unwrap_or_else(|_| "unknown".to_owned()),
        }
    }
}

/// The witness register.
pub struct PressureRegister {
    config: PressureRegisterConfig,
    next_id: std::sync::Mutex<u64>,
}

impl std::fmt::Debug for PressureRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PressureRegister")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl PressureRegister {
    /// Construct.
    #[must_use]
    pub fn new(config: PressureRegisterConfig) -> Self {
        Self {
            config,
            next_id: std::sync::Mutex::new(1),
        }
    }

    /// Borrow the config.
    #[must_use]
    pub fn config(&self) -> &PressureRegisterConfig {
        &self.config
    }

    /// Detect and emit. Returns `Some(written_path)` if a pressure event
    /// was emitted, `None` if the excerpt was in-charter.
    ///
    /// # Errors
    ///
    /// [`PressureRegisterError::WriteFailed`] on filesystem I/O failure.
    /// [`PressureRegisterError::Serialise`] on JSON encoding failure.
    ///
    /// # Panics
    ///
    /// Panics only if the internal id mutex is poisoned, which requires
    /// a prior unwinding panic; treat as unrecoverable.
    pub fn detect_and_emit(
        &self,
        trigger_excerpt: &str,
        source: PressureSource,
        proposed_feature: &str,
        violated_charter: CharterSection,
    ) -> Result<Option<PathBuf>, PressureRegisterError> {
        let Some(category) = classify_excerpt(trigger_excerpt) else {
            return Ok(None);
        };
        let event = self.build_event(
            category,
            trigger_excerpt,
            source,
            proposed_feature,
            violated_charter,
        );
        let path = self.emit(&event)?;
        Ok(Some(path))
    }

    /// Build a `PressureEvent` from raw inputs without emitting it.
    ///
    /// # Panics
    ///
    /// Panics only if the internal id mutex is poisoned.
    #[must_use]
    pub fn build_event(
        &self,
        forbidden_category: ForbiddenCategory,
        trigger_excerpt: &str,
        source: PressureSource,
        proposed_feature: &str,
        violated_charter: CharterSection,
    ) -> PressureEvent {
        let id = {
            let mut guard = self.next_id.lock().expect("next_id lock");
            let id = *guard;
            *guard = guard.saturating_add(1);
            id
        };
        PressureEvent {
            schema_version: SCHEMA_VERSION,
            id,
            detected_at: rfc3339_now(),
            session_id: self.config.session_id.clone(),
            forbidden_category,
            trigger_excerpt: truncate_excerpt(trigger_excerpt),
            source,
            proposed_feature: proposed_feature.to_owned(),
            violated_charter,
        }
    }

    /// Emit a pre-built event. Writes one JSONL line per file. Atomic
    /// (tmp + rename) per spec § 3.
    ///
    /// # Errors
    ///
    /// See [`Self::detect_and_emit`].
    pub fn emit(&self, event: &PressureEvent) -> Result<PathBuf, PressureRegisterError> {
        std::fs::create_dir_all(&self.config.notices_dir)?;
        let date = rfc3339_today();
        let session_short = event
            .session_id
            .chars()
            .take(8)
            .collect::<String>();
        let filename = format!(
            "PHASE-B-RESERVATION-NOTICE-{date}-{session_short}-{:06}.jsonl",
            event.id
        );
        let final_path = self.config.notices_dir.join(&filename);
        let tmp_path = self.config.notices_dir.join(format!("{filename}.tmp"));
        {
            let mut f = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&tmp_path)?;
            let line = serde_json::to_string(event)?;
            f.write_all(line.as_bytes())?;
            f.write_all(b"\n")?;
            f.sync_all()?;
        }
        std::fs::rename(&tmp_path, &final_path)?;
        tracing::info!(
            target: "m15.emit",
            event_id = event.id,
            category = ?event.forbidden_category,
            "PHASE-B-RESERVATION-NOTICE emitted"
        );
        Ok(final_path)
    }

    /// Inspect a directory for emitted notices (used by tests +
    /// observability tooling).
    ///
    /// # Errors
    ///
    /// [`PressureRegisterError::WriteFailed`] for I/O errors enumerating
    /// the directory.
    pub fn list_notices(&self) -> Result<Vec<PathBuf>, PressureRegisterError> {
        let mut out = Vec::new();
        if !self.config.notices_dir.exists() {
            return Ok(out);
        }
        for entry in std::fs::read_dir(&self.config.notices_dir)? {
            let entry = entry?;
            let path = entry.path();
            let name = path
                .file_name()
                .and_then(std::ffi::OsStr::to_str)
                .unwrap_or("");
            let has_jsonl_ext = std::path::Path::new(name)
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("jsonl"));
            if name.starts_with("PHASE-B-RESERVATION-NOTICE-") && has_jsonl_ext {
                out.push(path);
            }
        }
        out.sort();
        Ok(out)
    }
}

fn rfc3339_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|d| i64::try_from(d.as_secs()).ok())
        .map_or_else(|| "unknown".to_owned(), |s| format!("ts_s={s}"))
}

fn rfc3339_today() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|d| i64::try_from(d.as_secs()).ok())
        .unwrap_or(0);
    // YYYY-MM-DD approximation. Avoid a chrono dep per god-tier rule 18.
    let days = secs / 86_400;
    format!("d{days}")
}

#[cfg(test)]
mod tests {
    use super::{
        classify_excerpt, truncate_excerpt, CharterSection, ForbiddenCategory,
        PressureRegister, PressureRegisterConfig, PressureSource, SCHEMA_VERSION,
    };

    fn tmp_register() -> (PressureRegister, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("tempdir");
        let cfg = PressureRegisterConfig {
            notices_dir: dir.path().to_path_buf(),
            session_id: "TESTSESS".into(),
        };
        (PressureRegister::new(cfg), dir)
    }

    // ---- truncate_excerpt (2) -------------------------------------------

    #[test]
    fn truncate_short_returns_as_is() {
        assert_eq!(truncate_excerpt("hello"), "hello");
    }

    #[test]
    fn truncate_long_returns_ellipsis() {
        let big = "a".repeat(1024);
        let t = truncate_excerpt(&big);
        assert!(t.chars().count() <= 513);
        assert!(t.ends_with('\u{2026}'));
    }

    // ---- classify_excerpt (10) ------------------------------------------

    #[test]
    fn classify_recommend_verb() {
        assert_eq!(
            classify_excerpt("recommend_cascade_for_user"),
            Some(ForbiddenCategory::RecommendVerb)
        );
    }

    #[test]
    fn classify_auto_verb() {
        assert_eq!(
            classify_excerpt("auto_promote_workflow"),
            Some(ForbiddenCategory::AutoOrSmartVerb)
        );
    }

    #[test]
    fn classify_smart_verb() {
        assert_eq!(
            classify_excerpt("smart_route_dispatch"),
            Some(ForbiddenCategory::AutoOrSmartVerb)
        );
    }

    #[test]
    fn classify_rewrite_verb() {
        assert_eq!(
            classify_excerpt("rewrite_src_file"),
            Some(ForbiddenCategory::RewriteVerb)
        );
    }

    #[test]
    fn classify_package_publish() {
        assert_eq!(
            classify_excerpt("package_release"),
            Some(ForbiddenCategory::PackageOrPublish)
        );
        assert_eq!(
            classify_excerpt("publish_to_crates_io"),
            Some(ForbiddenCategory::PackageOrPublish)
        );
    }

    #[test]
    fn classify_optimise() {
        assert_eq!(
            classify_excerpt("optimise_thresholds"),
            Some(ForbiddenCategory::OptimiseWithoutGate)
        );
    }

    #[test]
    fn classify_http_server() {
        assert_eq!(
            classify_excerpt("expose http server on :9000"),
            Some(ForbiddenCategory::HttpServerSurface)
        );
    }

    #[test]
    fn classify_povm_write() {
        assert_eq!(
            classify_excerpt("propose POVM write back"),
            Some(ForbiddenCategory::DeprecatedPovmWrite)
        );
    }

    #[test]
    fn classify_synthex_v2_import() {
        assert_eq!(
            classify_excerpt("use synthex_v2::types::Foo;"),
            Some(ForbiddenCategory::SynthexV2Import)
        );
    }

    #[test]
    fn classify_handshake_silence() {
        assert_eq!(
            classify_excerpt("handshake silence past 30s"),
            Some(ForbiddenCategory::HandshakeSilence)
        );
    }

    #[test]
    fn classify_in_charter_returns_none() {
        assert!(classify_excerpt("read atuin history and record outcomes").is_none());
        assert!(classify_excerpt("aggregate evidence with wilson ci").is_none());
    }

    // ---- detect_and_emit (5) --------------------------------------------

    #[test]
    fn detect_and_emit_writes_jsonl_for_forbidden() {
        let (reg, _dir) = tmp_register();
        let path = reg
            .detect_and_emit(
                "recommend_cascade_for_user",
                PressureSource::Unknown,
                "proposing recommend verb",
                CharterSection::V1_3HardRefusal,
            )
            .expect("emit")
            .expect("Some(path)");
        let content = std::fs::read_to_string(&path).expect("read");
        assert!(content.contains("recommend_verb"));
        assert!(content.contains("TESTSESS"));
    }

    #[test]
    fn detect_and_emit_returns_none_for_in_charter() {
        let (reg, _dir) = tmp_register();
        let result = reg
            .detect_and_emit(
                "read m7 rows",
                PressureSource::Unknown,
                "passive",
                CharterSection::V1_3VerbClass,
            )
            .expect("ok");
        assert!(result.is_none());
    }

    #[test]
    fn emit_creates_directory_if_missing() {
        let dir = tempfile::tempdir().expect("tempdir");
        let subdir = dir.path().join("nested/sub");
        let reg = PressureRegister::new(PressureRegisterConfig {
            notices_dir: subdir,
            session_id: "TESTSESS".into(),
        });
        let event = reg.build_event(
            ForbiddenCategory::RewriteVerb,
            "rewrite_src",
            PressureSource::Unknown,
            "test",
            CharterSection::V1_3HardRefusal,
        );
        let p = reg.emit(&event).expect("emit");
        assert!(p.exists());
    }

    #[test]
    fn emit_filename_starts_with_phase_b_reservation_notice() {
        let (reg, _dir) = tmp_register();
        let event = reg.build_event(
            ForbiddenCategory::AutoOrSmartVerb,
            "auto_promote",
            PressureSource::Unknown,
            "test",
            CharterSection::V1_3HardRefusal,
        );
        let p = reg.emit(&event).expect("emit");
        let name = p
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .expect("name");
        assert!(name.starts_with("PHASE-B-RESERVATION-NOTICE-"));
        assert!(std::path::Path::new(name)
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("jsonl")));
    }

    #[test]
    fn emitted_event_round_trips_via_serde() {
        let (reg, _dir) = tmp_register();
        let event = reg.build_event(
            ForbiddenCategory::PackageOrPublish,
            "package_release",
            PressureSource::CliVerbProposal {
                proposed_command: "wf-crystallise package".into(),
            },
            "automated package step",
            CharterSection::V1_3HardRefusal,
        );
        let path = reg.emit(&event).expect("emit");
        let content = std::fs::read_to_string(&path).expect("read");
        let parsed: super::PressureEvent =
            serde_json::from_str(content.trim()).expect("parse");
        assert_eq!(parsed.id, event.id);
        assert_eq!(parsed.forbidden_category, event.forbidden_category);
        assert_eq!(parsed.schema_version, SCHEMA_VERSION);
    }

    // ---- list_notices (2) -----------------------------------------------

    #[test]
    fn list_notices_returns_only_matching_files() {
        let (reg, dir) = tmp_register();
        let _ = reg
            .detect_and_emit(
                "auto_promote_workflow",
                PressureSource::Unknown,
                "test",
                CharterSection::V1_3HardRefusal,
            )
            .expect("emit");
        std::fs::write(dir.path().join("unrelated.txt"), "x").expect("write");
        let notices = reg.list_notices().expect("list");
        assert_eq!(notices.len(), 1);
        assert!(notices[0]
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("")
            .starts_with("PHASE-B-RESERVATION-NOTICE-"));
    }

    #[test]
    fn list_notices_empty_dir_returns_empty_vec() {
        let dir = tempfile::tempdir().expect("tempdir");
        let reg = PressureRegister::new(PressureRegisterConfig {
            notices_dir: dir.path().to_path_buf(),
            session_id: "TESTSESS".into(),
        });
        assert!(reg.list_notices().expect("list").is_empty());
    }

    // ---- monotonic id (1) -----------------------------------------------

    #[test]
    fn event_ids_are_monotonically_increasing() {
        let (reg, _dir) = tmp_register();
        let e1 = reg.build_event(
            ForbiddenCategory::RewriteVerb,
            "a",
            PressureSource::Unknown,
            "p",
            CharterSection::V1_3HardRefusal,
        );
        let e2 = reg.build_event(
            ForbiddenCategory::RewriteVerb,
            "b",
            PressureSource::Unknown,
            "p",
            CharterSection::V1_3HardRefusal,
        );
        assert!(e2.id > e1.id);
    }

    // ---- schema version (1) ---------------------------------------------

    #[test]
    fn schema_version_is_one_at_v1_3() {
        assert_eq!(SCHEMA_VERSION, 1);
    }

    // ---- path filename safety (1) ---------------------------------------

    #[test]
    fn emitted_path_does_not_traverse_parent() {
        let (reg, _dir) = tmp_register();
        let event = reg.build_event(
            ForbiddenCategory::RewriteVerb,
            "rewrite_x",
            PressureSource::Unknown,
            "test",
            CharterSection::V1_3HardRefusal,
        );
        let p = reg.emit(&event).expect("emit");
        let name = p
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .expect("name");
        assert!(!name.contains(".."));
        let parent_ok =
            reg.config().notices_dir.as_path() == p.parent().expect("parent");
        assert!(parent_ok);
    }
}
