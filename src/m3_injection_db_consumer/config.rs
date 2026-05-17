//! `InjectionDbConfig` — limits + paths for the m3 reader.

use std::path::PathBuf;

/// Lower bound on `max_unresolved` / `max_recently_resolved`.
pub const LIMIT_MIN: usize = 100;

/// Upper bound on `max_unresolved` / `max_recently_resolved`.
pub const LIMIT_MAX: usize = 5_000;

/// Default `max_unresolved` per V7 cluster-A plan.
pub const MAX_UNRESOLVED_DEFAULT: usize = 500;

/// Default `max_recently_resolved` per V7 cluster-A plan.
pub const MAX_RECENTLY_RESOLVED_DEFAULT: usize = 500;

/// Default `resolved_recency_sessions` per V7 cluster-A plan.
pub const RESOLVED_RECENCY_SESSIONS_DEFAULT: u32 = 10;

/// Configuration for [`super::open_readonly`].
#[derive(Debug, Clone)]
pub struct InjectionDbConfig {
    /// Path to `injection.db`. Default `$HOME/.local/share/habitat/injection.db`.
    pub db_path: PathBuf,
    /// Maximum unresolved rows returned per call (clamped at use time).
    pub max_unresolved: usize,
    /// Maximum recently-resolved rows returned per call.
    pub max_recently_resolved: usize,
    /// "Recently resolved" cutoff = `MAX(resolved_session) - this`.
    pub resolved_recency_sessions: u32,
}

impl Default for InjectionDbConfig {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_owned());
        Self {
            db_path: PathBuf::from(format!("{home}/.local/share/habitat/injection.db")),
            max_unresolved: MAX_UNRESOLVED_DEFAULT,
            max_recently_resolved: MAX_RECENTLY_RESOLVED_DEFAULT,
            resolved_recency_sessions: RESOLVED_RECENCY_SESSIONS_DEFAULT,
        }
    }
}

impl InjectionDbConfig {
    /// `max_unresolved` clamped to `[LIMIT_MIN, LIMIT_MAX]`.
    #[must_use]
    pub fn effective_max_unresolved(&self) -> usize {
        self.max_unresolved.clamp(LIMIT_MIN, LIMIT_MAX)
    }

    /// `max_recently_resolved` clamped to `[LIMIT_MIN, LIMIT_MAX]`.
    #[must_use]
    pub fn effective_max_recently_resolved(&self) -> usize {
        self.max_recently_resolved.clamp(LIMIT_MIN, LIMIT_MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        InjectionDbConfig, LIMIT_MAX, LIMIT_MIN, MAX_RECENTLY_RESOLVED_DEFAULT,
        MAX_UNRESOLVED_DEFAULT, RESOLVED_RECENCY_SESSIONS_DEFAULT,
    };

    #[test]
    fn defaults_match_v7_plan() {
        let c = InjectionDbConfig::default();
        assert_eq!(c.max_unresolved, MAX_UNRESOLVED_DEFAULT);
        assert_eq!(c.max_recently_resolved, MAX_RECENTLY_RESOLVED_DEFAULT);
        assert_eq!(c.resolved_recency_sessions, RESOLVED_RECENCY_SESSIONS_DEFAULT);
        assert!(c.db_path.to_string_lossy().contains(".local/share/habitat"));
    }

    #[test]
    fn effective_max_unresolved_clamps_below_min() {
        let c = InjectionDbConfig {
            max_unresolved: 0,
            ..Default::default()
        };
        assert_eq!(c.effective_max_unresolved(), LIMIT_MIN);
    }

    #[test]
    fn effective_max_unresolved_clamps_above_max() {
        let c = InjectionDbConfig {
            max_unresolved: 1_000_000,
            ..Default::default()
        };
        assert_eq!(c.effective_max_unresolved(), LIMIT_MAX);
    }

    #[test]
    fn effective_max_recently_resolved_clamps() {
        let c = InjectionDbConfig {
            max_recently_resolved: 0,
            ..Default::default()
        };
        assert_eq!(c.effective_max_recently_resolved(), LIMIT_MIN);
    }

    #[test]
    fn db_path_override_via_struct_field_is_honoured() {
        let c = InjectionDbConfig {
            db_path: "/tmp/x.db".into(),
            ..Default::default()
        };
        assert_eq!(c.db_path.to_string_lossy(), "/tmp/x.db");
    }
}
