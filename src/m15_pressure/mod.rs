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
//!
//! # Invariants (preserved under refactor)
//!
//! - `PressureEvent` JSONL writer is one-event-per-file (atomic via
//!   tmp + rename); the on-disk shape is a single JSONL line.
//! - Every emitted event carries a **monotonic** `id` per
//!   [`PressureRegister`] instance and a `detected_at_ms` Unix-millis
//!   timestamp; combined, `(id, detected_at_ms)` is a strict total
//!   order even when wall clock drifts.
//! - `CharterSection` and `ForbiddenCategory` enums are CLOSED in this
//!   release. Adding a variant is a spec amendment, not a refactor;
//!   serde uses `snake_case` rename so on-disk format is stable.
//! - `classify_excerpt` returns the closed category enum only — it
//!   never folds the input excerpt into category metadata. F11
//!   (cascade-monoculture) mitigated.
//! - Hardening S1002388: timestamp + millis component; session id
//!   sanitised before filename use; tmp file carries a pid suffix so
//!   inter-process collisions cannot clobber.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use thiserror::Error;

/// Schema version for serialised [`PressureEvent`].
pub const SCHEMA_VERSION: u32 = 1;

/// Maximum length of the `trigger_excerpt` field, in CHARS, after
/// truncation. See [`truncate_excerpt`].
pub const MAX_EXCERPT_CHARS: usize = 512;

/// Failure modes for the pressure register.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum PressureRegisterError {
    /// I/O error writing the JSONL file.
    #[error("write failed: {0}")]
    WriteFailed(#[from] std::io::Error),
    /// JSON serialisation failure.
    #[error("serialise: {0}")]
    Serialise(#[from] serde_json::Error),
}

/// Closed-set forbidden-verb category.
///
/// CLOSED enum: adding a variant is a spec amendment, not a refactor.
/// Variants serialise via `rename_all = "snake_case"`.
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
    /// `use synthex_v2::*` wholesale import.
    SynthexV2Import,
    /// Handshake silence past timeout (AP-V7-08 mitigation).
    HandshakeSilence,
    /// Any other forbidden surface with description. NOTE: callers are
    /// the ONLY producers of this variant; [`classify_excerpt`] never
    /// fabricates it (F11 mitigation — no excerpt content folded into
    /// the closed category space).
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
    /// Monotonic event id per [`PressureRegister`] instance.
    pub id: u64,
    /// Pseudo-RFC3339 timestamp string (UTC seconds since epoch, no
    /// chrono dep). Format: `ts_s=<unix_seconds>`; companion
    /// [`Self::detected_at_ms`] carries millisecond precision.
    pub detected_at: String,
    /// Unix milliseconds since epoch — strict ordering signal.
    pub detected_at_ms: i64,
    /// Originating session id.
    pub session_id: String,
    /// Forbidden category (closed set; F11 mitigation — never carries
    /// input excerpt content).
    pub forbidden_category: ForbiddenCategory,
    /// Trigger excerpt (≤ [`MAX_EXCERPT_CHARS`]).
    pub trigger_excerpt: String,
    /// Source surface.
    pub source: PressureSource,
    /// Human-readable feature description.
    pub proposed_feature: String,
    /// Violated charter section.
    pub violated_charter: CharterSection,
}

/// Truncate an excerpt to ≤ [`MAX_EXCERPT_CHARS`] *characters* (not bytes),
/// appending the U+2026 ellipsis on truncation.
#[must_use]
pub fn truncate_excerpt(s: &str) -> String {
    if s.len() <= MAX_EXCERPT_CHARS {
        return s.to_owned();
    }
    let mut iter = s.chars();
    let prefix: String = iter.by_ref().take(MAX_EXCERPT_CHARS).collect();
    if iter.next().is_none() {
        prefix
    } else {
        let mut out = prefix;
        out.push('\u{2026}');
        out
    }
}

/// Heuristic classifier: match a command / text excerpt to a forbidden
/// category. Returns `None` when the excerpt is in-charter.
///
/// **F11 mitigation:** returns ONLY the closed-set variants. NEVER
/// constructs the `Other { description: <input> }` variant. The
/// `trigger_excerpt` field on [`PressureEvent`] is the faithful copy of
/// the input; the `forbidden_category` field is a pane-label-free
/// classification.
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
    /// Panics only if the internal id mutex is poisoned.
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
        let detected_at_ms = unix_ms_now();
        PressureEvent {
            schema_version: SCHEMA_VERSION,
            id,
            detected_at: pseudo_rfc3339(detected_at_ms),
            detected_at_ms,
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
    /// The tmp filename carries the current pid as a suffix so two
    /// processes that pick the same `(id, session_short, day)` triple
    /// cannot clobber each other's tmp file.
    ///
    /// # Errors
    ///
    /// See [`Self::detect_and_emit`].
    pub fn emit(&self, event: &PressureEvent) -> Result<PathBuf, PressureRegisterError> {
        std::fs::create_dir_all(&self.config.notices_dir)?;
        let day = ymd_day_bucket(event.detected_at_ms);
        let session_short = sanitise_session_id(&event.session_id);
        let filename = format!(
            "PHASE-B-RESERVATION-NOTICE-{day}-{session_short}-{:06}.jsonl",
            event.id
        );
        let final_path = self.config.notices_dir.join(&filename);
        let tmp_path = self
            .config
            .notices_dir
            .join(format!("{filename}.tmp.{}", std::process::id()));
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

    /// Inspect a directory for emitted notices.
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

/// Sanitise a session id for use in a filename: keep only
/// `[A-Za-z0-9_-]`, take first 8 chars after filtering; fall back to
/// `"unknown_"` when nothing survives. Path-traversal-safe.
fn sanitise_session_id(raw: &str) -> String {
    let filtered: String = raw
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .take(8)
        .collect();
    if filtered.is_empty() {
        "unknown_".to_owned()
    } else {
        filtered
    }
}

/// Unix milliseconds since epoch, saturating on overflow / clock skew.
fn unix_ms_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|d| i64::try_from(d.as_millis()).ok())
        .unwrap_or(0)
}

/// Pseudo-RFC3339 string — Unix-seconds carried as text (chrono out per
/// god-tier rule 18). Companion `detected_at_ms` carries ordering.
fn pseudo_rfc3339(ms: i64) -> String {
    let s = ms / 1_000;
    format!("ts_s={s}")
}

/// Day bucket label for filename use. `d<N>` where N = days since epoch.
fn ymd_day_bucket(ms: i64) -> String {
    let secs = ms / 1_000;
    let days = secs / 86_400;
    format!("d{days}")
}

#[cfg(test)]
mod tests {
    use super::{
        classify_excerpt, pseudo_rfc3339, sanitise_session_id, truncate_excerpt,
        unix_ms_now, ymd_day_bucket, CharterSection, ForbiddenCategory, PressureEvent,
        PressureRegister, PressureRegisterConfig, PressureSource, MAX_EXCERPT_CHARS,
        SCHEMA_VERSION,
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
        assert!(t.chars().count() <= MAX_EXCERPT_CHARS + 1);
        assert!(t.ends_with('\u{2026}'));
    }

    // ---- classify_excerpt (12) ------------------------------------------

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

    // ====================================================================
    // Hardening pass (S1002388) — m15 god-tier maintainer pass.
    // ====================================================================

    // rationale: Anti-property F11 — classify_excerpt NEVER returns the
    // `Other { description: <input> }` variant.
    #[test]
    fn classify_excerpt_never_returns_other_variant_f11() {
        let inputs = [
            "recommend_cascade",
            "auto_promote",
            "rewrite_src",
            "package_release",
            "optimise_x",
            "http server :9000",
            "POVM write back",
            "use synthex_v2::x",
            "handshake silence",
            "innocuous read",
            "agent reports recommend something",
        ];
        for inp in inputs {
            if let Some(cat) = classify_excerpt(inp) {
                assert!(
                    !matches!(cat, ForbiddenCategory::Other { .. }),
                    "classify_excerpt fabricated Other variant for {inp:?}"
                );
            }
        }
    }

    // rationale: Anti-property F11 — input excerpt body is NEVER a
    // substring of the category JSON.
    #[test]
    fn classify_excerpt_category_json_does_not_leak_input() {
        let needle_input = "recommend_TOPSECRETSESSIONABC123";
        let cat = classify_excerpt(needle_input).expect("classified");
        let serialised = serde_json::to_string(&cat).expect("ser");
        assert!(
            !serialised.contains("TOPSECRETSESSIONABC123"),
            "category JSON {serialised:?} leaked input substring"
        );
    }

    // rationale: Boundary — truncate_excerpt at exactly
    // MAX_EXCERPT_CHARS bytes of ASCII returns input as-is.
    #[test]
    fn truncate_excerpt_at_max_chars_returns_input_unchanged() {
        let exact: String = "a".repeat(MAX_EXCERPT_CHARS);
        assert_eq!(truncate_excerpt(&exact), exact);
    }

    // rationale: Boundary — truncate_excerpt at MAX + 1 ASCII chars
    // truncates and appends U+2026.
    #[test]
    fn truncate_excerpt_just_over_max_truncates() {
        let just_over: String = "a".repeat(MAX_EXCERPT_CHARS + 1);
        let t = truncate_excerpt(&just_over);
        assert!(t.ends_with('\u{2026}'));
        assert_eq!(t.chars().count(), MAX_EXCERPT_CHARS + 1);
    }

    // rationale: Adversarial input — multi-byte UTF-8 does NOT slice a
    // char in half. Output well-formed UTF-8.
    #[test]
    fn truncate_excerpt_does_not_split_multibyte_chars() {
        let cjk: String = "测".repeat(600);
        let t = truncate_excerpt(&cjk);
        assert!(t.ends_with('\u{2026}'));
        assert_eq!(t.chars().count(), MAX_EXCERPT_CHARS + 1);
    }

    // rationale: Contract regression — numeric fields round-trip
    // bit-exact via serde_json.
    #[test]
    fn serde_round_trip_preserves_numeric_fields_bit_exact() {
        let (reg, _dir) = tmp_register();
        let event = reg.build_event(
            ForbiddenCategory::AutoOrSmartVerb,
            "auto_x",
            PressureSource::Unknown,
            "p",
            CharterSection::V1_3HardRefusal,
        );
        let s = serde_json::to_string(&event).expect("ser");
        let back: PressureEvent = serde_json::from_str(&s).expect("de");
        assert_eq!(back.id, event.id);
        assert_eq!(back.schema_version, event.schema_version);
        assert_eq!(back.detected_at_ms, event.detected_at_ms);
        assert_eq!(back.detected_at, event.detected_at);
        assert_eq!(back.session_id, event.session_id);
    }

    // rationale: Concurrency — many threads on shared Arc produce
    // strictly monotonic ids with no duplicates.
    #[test]
    fn concurrent_build_event_ids_are_strictly_monotonic_no_duplicates() {
        use std::sync::Arc;
        let dir = tempfile::tempdir().expect("tempdir");
        let reg = Arc::new(PressureRegister::new(PressureRegisterConfig {
            notices_dir: dir.path().to_path_buf(),
            session_id: "MT".into(),
        }));
        let mut handles = Vec::new();
        for _ in 0..8_u32 {
            let r = Arc::clone(&reg);
            handles.push(std::thread::spawn(move || {
                let mut ids = Vec::with_capacity(50);
                for _ in 0..50_u32 {
                    let e = r.build_event(
                        ForbiddenCategory::RewriteVerb,
                        "x",
                        PressureSource::Unknown,
                        "p",
                        CharterSection::V1_3HardRefusal,
                    );
                    ids.push(e.id);
                }
                ids
            }));
        }
        let mut all: Vec<u64> = Vec::new();
        for h in handles {
            all.extend(h.join().expect("join"));
        }
        let mut sorted = all.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), all.len(), "duplicates emitted");
        assert_eq!(*sorted.first().expect("first"), 1);
        assert_eq!(*sorted.last().expect("last"), all.len() as u64);
    }

    // rationale: Concurrency — concurrent emit() produces N distinct
    // on-disk files.
    #[test]
    fn concurrent_emit_creates_n_distinct_files() {
        use std::sync::Arc;
        let dir = tempfile::tempdir().expect("tempdir");
        let reg = Arc::new(PressureRegister::new(PressureRegisterConfig {
            notices_dir: dir.path().to_path_buf(),
            session_id: "MT".into(),
        }));
        let mut handles = Vec::new();
        for _ in 0..4_u32 {
            let r = Arc::clone(&reg);
            handles.push(std::thread::spawn(move || {
                for _ in 0..10_u32 {
                    let e = r.build_event(
                        ForbiddenCategory::RewriteVerb,
                        "x",
                        PressureSource::Unknown,
                        "p",
                        CharterSection::V1_3HardRefusal,
                    );
                    r.emit(&e).expect("emit");
                }
            }));
        }
        for h in handles {
            h.join().expect("join");
        }
        let notices = reg.list_notices().expect("list");
        assert_eq!(notices.len(), 40);
    }

    // rationale: Resource accounting — emit() never leaves a `.tmp.*`
    // behind on happy path.
    #[test]
    fn emit_does_not_leave_tmp_files_on_happy_path() {
        let (reg, dir) = tmp_register();
        for _ in 0..5_u32 {
            let e = reg.build_event(
                ForbiddenCategory::RewriteVerb,
                "x",
                PressureSource::Unknown,
                "p",
                CharterSection::V1_3HardRefusal,
            );
            reg.emit(&e).expect("emit");
        }
        for entry in std::fs::read_dir(dir.path()).expect("read_dir") {
            let entry = entry.expect("entry");
            let name = entry.file_name().to_string_lossy().into_owned();
            assert!(!name.contains(".tmp."), "leftover tmp file: {name}");
        }
    }

    // rationale: Adversarial input — path-traversal session_id sanitised
    // before filename use.
    #[test]
    fn session_id_with_path_traversal_is_sanitised_in_filename() {
        let dir = tempfile::tempdir().expect("tempdir");
        let reg = PressureRegister::new(PressureRegisterConfig {
            notices_dir: dir.path().to_path_buf(),
            session_id: "../../etc/passwd ; rm -rf /".into(),
        });
        let e = reg.build_event(
            ForbiddenCategory::RewriteVerb,
            "x",
            PressureSource::Unknown,
            "p",
            CharterSection::V1_3HardRefusal,
        );
        let p = reg.emit(&e).expect("emit");
        let name = p
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .expect("name");
        assert!(!name.contains('/'));
        assert!(!name.contains(".."));
        assert!(!name.contains(' '));
        assert!(!name.contains(';'));
        assert!(p.parent().expect("parent") == dir.path());
    }

    // rationale: Boundary — empty / non-alphanum session_id falls back.
    #[test]
    fn sanitise_empty_session_id_falls_back() {
        assert_eq!(sanitise_session_id(""), "unknown_");
        assert_eq!(sanitise_session_id("////"), "unknown_");
    }

    // rationale: Determinism — sanitise_session_id is pure.
    #[test]
    fn sanitise_session_id_is_deterministic() {
        let first = sanitise_session_id("AbC-123_DEF456ghi");
        for _ in 0..100_u32 {
            assert_eq!(sanitise_session_id("AbC-123_DEF456ghi"), first);
        }
    }

    // rationale: Boundary — timestamp helpers produce sensible shapes.
    #[test]
    fn timestamp_helpers_produce_sensible_values() {
        let ms = unix_ms_now();
        assert!(ms >= 0);
        let s = pseudo_rfc3339(ms);
        assert!(s.starts_with("ts_s="));
        let bucket = ymd_day_bucket(ms);
        assert!(bucket.starts_with('d'));
    }

    // rationale: Anti-property F11 — category field stays identical
    // across differing excerpts; only trigger_excerpt reflects input.
    #[test]
    fn category_field_independent_of_excerpt_body() {
        let (reg, _dir) = tmp_register();
        let e1 = reg.build_event(
            classify_excerpt("auto_promote_pane1").expect("c1"),
            "auto_promote_pane1",
            PressureSource::Unknown,
            "p",
            CharterSection::V1_3HardRefusal,
        );
        let e2 = reg.build_event(
            classify_excerpt("auto_promote_pane2").expect("c2"),
            "auto_promote_pane2",
            PressureSource::Unknown,
            "p",
            CharterSection::V1_3HardRefusal,
        );
        assert_eq!(e1.forbidden_category, e2.forbidden_category);
        assert_ne!(e1.trigger_excerpt, e2.trigger_excerpt);
    }

    // rationale: Contract regression — detected_at_ms is non-decreasing
    // in emission order.
    #[test]
    fn detected_at_ms_is_non_decreasing_in_emission_order() {
        let (reg, _dir) = tmp_register();
        let mut last: i64 = 0;
        for _ in 0..50_u32 {
            let e = reg.build_event(
                ForbiddenCategory::RewriteVerb,
                "x",
                PressureSource::Unknown,
                "p",
                CharterSection::V1_3HardRefusal,
            );
            assert!(
                e.detected_at_ms >= last,
                "detected_at_ms went backwards: {} < {last}",
                e.detected_at_ms
            );
            last = e.detected_at_ms;
        }
    }

    // rationale: Cross-module surface invariant — m15 emits the
    // forbidden-verb category but never includes any namespace prefix
    // string (that surface belongs to m9 / m13 / m42). Confirms via an
    // indirect check: emit one event and assert the on-disk JSONL has no
    // `_trace` suffix substring (which would be a tell for stray
    // namespace literals leaking into the witness record).
    #[test]
    fn m15_emit_path_does_not_embed_namespace_suffix() {
        let (reg, _dir) = tmp_register();
        let e = reg.build_event(
            ForbiddenCategory::RewriteVerb,
            "rewrite_x",
            PressureSource::Unknown,
            "p",
            CharterSection::V1_3HardRefusal,
        );
        let path = reg.emit(&e).expect("emit");
        let content = std::fs::read_to_string(&path).expect("read");
        // `_trace` is a distinctive substring of the workflow_trace
        // namespace prefix; if it ever appears in an m15 event JSONL,
        // someone has folded a namespace literal into m15's wire-form.
        assert!(
            !content.contains("_trace"),
            "m15 event JSONL must not embed namespace-prefix substrings: {content}"
        );
    }

    // ====================================================================
    // Hardening pass 2 — +13 tests. Classifier precedence, build_event
    // shape, serde enum wire-form, list ordering, source variants.
    // ====================================================================

    // rationale: Correctness — classify_excerpt is case-insensitive; an
    // upper-case verb still matches its forbidden category.
    #[test]
    fn classify_excerpt_is_case_insensitive() {
        assert_eq!(
            classify_excerpt("AUTO_PROMOTE"),
            Some(ForbiddenCategory::AutoOrSmartVerb)
        );
        assert_eq!(
            classify_excerpt("Rewrite_Source"),
            Some(ForbiddenCategory::RewriteVerb)
        );
    }

    // rationale: Boundary — classifier precedence. An excerpt containing
    // BOTH a recommend-verb and an auto-verb matches the first checked
    // branch (recommend), per the function's top-to-bottom ordering.
    #[test]
    fn classify_excerpt_recommend_wins_over_auto_when_both_present() {
        let cat = classify_excerpt("recommend_ and auto_ together").expect("c");
        assert_eq!(cat, ForbiddenCategory::RecommendVerb, "recommend checked first");
    }

    // rationale: Boundary — the `recommend ` (with trailing space) branch
    // catches the natural-language form, not just the snake_case verb.
    #[test]
    fn classify_excerpt_matches_recommend_space_natural_language() {
        assert_eq!(
            classify_excerpt("the agent will recommend a cascade"),
            Some(ForbiddenCategory::RecommendVerb)
        );
    }

    // rationale: Adversarial input — POVM mention WITHOUT a write/emit
    // verb is NOT classified as DeprecatedPovmWrite (both conditions
    // required by the `&&` guard).
    #[test]
    fn classify_excerpt_povm_read_only_is_in_charter() {
        assert!(
            classify_excerpt("read from povm history").is_none(),
            "povm read without write/emit must be in-charter"
        );
        assert_eq!(
            classify_excerpt("povm write back"),
            Some(ForbiddenCategory::DeprecatedPovmWrite)
        );
    }

    // rationale: Adversarial input — "handshake" alone or "silence" alone
    // is in-charter; only the conjunction triggers HandshakeSilence.
    #[test]
    fn classify_excerpt_handshake_alone_is_in_charter() {
        assert!(classify_excerpt("send a handshake message").is_none());
        assert!(classify_excerpt("a moment of silence").is_none());
        assert_eq!(
            classify_excerpt("handshake met with silence"),
            Some(ForbiddenCategory::HandshakeSilence)
        );
    }

    // rationale: Boundary — the empty excerpt is in-charter (classifier
    // returns None; detect_and_emit emits nothing).
    #[test]
    fn classify_empty_excerpt_returns_none() {
        assert!(classify_excerpt("").is_none());
        let (reg, _dir) = tmp_register();
        let result = reg
            .detect_and_emit(
                "",
                PressureSource::Unknown,
                "empty",
                CharterSection::V1_3VerbClass,
            )
            .expect("ok");
        assert!(result.is_none(), "empty excerpt emits nothing");
    }

    // rationale: Correctness — optimize_ (US spelling) is classified the
    // same as optimise_ (UK spelling).
    #[test]
    fn classify_excerpt_optimize_us_spelling_matches() {
        assert_eq!(
            classify_excerpt("optimize_parameters"),
            Some(ForbiddenCategory::OptimiseWithoutGate)
        );
    }

    // rationale: Correctness — build_event faithfully copies all inputs
    // into the event (category, source, feature, charter section).
    #[test]
    fn build_event_copies_all_input_fields() {
        let (reg, _dir) = tmp_register();
        let src = PressureSource::SpecPatch {
            file: "ai_docs/patch.md".into(),
        };
        let charter = CharterSection::Ap27Boundary;
        let e = reg.build_event(
            ForbiddenCategory::SidecarDaemon,
            "spawn sidecar daemon",
            src.clone(),
            "daemon proposal",
            charter.clone(),
        );
        assert_eq!(e.forbidden_category, ForbiddenCategory::SidecarDaemon);
        assert_eq!(e.source, src);
        assert_eq!(e.proposed_feature, "daemon proposal");
        assert_eq!(e.violated_charter, charter);
        assert_eq!(e.session_id, "TESTSESS");
    }

    // rationale: Correctness — build_event truncates an over-long
    // trigger_excerpt to MAX_EXCERPT_CHARS+1 (ellipsis) before storing.
    #[test]
    fn build_event_truncates_over_long_excerpt() {
        let (reg, _dir) = tmp_register();
        let big = "rewrite_".to_owned() + &"x".repeat(2_000);
        let e = reg.build_event(
            ForbiddenCategory::RewriteVerb,
            &big,
            PressureSource::Unknown,
            "p",
            CharterSection::V1_3HardRefusal,
        );
        assert!(e.trigger_excerpt.chars().count() <= MAX_EXCERPT_CHARS + 1);
        assert!(e.trigger_excerpt.ends_with('\u{2026}'));
    }

    // rationale: Contract regression — ForbiddenCategory serialises with
    // snake_case rename; the Other variant carries its description.
    #[test]
    fn forbidden_category_serde_wire_form_is_snake_case() {
        let s = serde_json::to_string(&ForbiddenCategory::RouteBypassConductor)
            .expect("ser");
        assert_eq!(s, "\"route_bypass_conductor\"", "snake_case wire form");
        let other = ForbiddenCategory::Other {
            description: "custom surface".into(),
        };
        let os = serde_json::to_string(&other).expect("ser");
        let back: ForbiddenCategory = serde_json::from_str(&os).expect("de");
        assert_eq!(back, other, "Other round-trips with description");
    }

    // rationale: Contract regression — CharterSection::V1_3HardRefusal
    // serde wire-form is the documented snake_case string.
    #[test]
    fn charter_section_serde_wire_form_round_trips() {
        for cs in [
            CharterSection::V1_3HardRefusal,
            CharterSection::V1_3VerbClass,
            CharterSection::Ap27Boundary,
            CharterSection::Other {
                section: "x-cut".into(),
            },
        ] {
            let s = serde_json::to_string(&cs).expect("ser");
            let back: CharterSection = serde_json::from_str(&s).expect("de");
            assert_eq!(back, cs);
        }
    }

    // rationale: Correctness — list_notices returns paths in sorted
    // order; since filenames embed the zero-padded id, sort order equals
    // emission order.
    #[test]
    fn list_notices_returns_paths_in_sorted_emission_order() {
        let (reg, _dir) = tmp_register();
        for _ in 0..5_u32 {
            let e = reg.build_event(
                ForbiddenCategory::RewriteVerb,
                "rewrite_x",
                PressureSource::Unknown,
                "p",
                CharterSection::V1_3HardRefusal,
            );
            reg.emit(&e).expect("emit");
        }
        let notices = reg.list_notices().expect("list");
        assert_eq!(notices.len(), 5);
        let mut sorted = notices.clone();
        sorted.sort();
        assert_eq!(notices, sorted, "list_notices output must be sorted");
    }

    // rationale: Correctness — detect_and_emit's written file embeds the
    // monotonic id zero-padded to 6 digits in the filename.
    #[test]
    fn emit_filename_embeds_zero_padded_event_id() {
        let (reg, _dir) = tmp_register();
        let e = reg.build_event(
            ForbiddenCategory::RewriteVerb,
            "rewrite_x",
            PressureSource::Unknown,
            "p",
            CharterSection::V1_3HardRefusal,
        );
        let id = e.id;
        let p = reg.emit(&e).expect("emit");
        let name = p
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .expect("name");
        assert!(
            name.contains(&format!("{id:06}")),
            "filename must embed zero-padded id {id:06}: {name}"
        );
    }
}
