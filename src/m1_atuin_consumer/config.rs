//! `AtuinConsumerConfig` — page-size / row-cap / fallback-timeout knobs.
//!
//! Per m1 spec § 2 + D-B (Luke S1002127): default `page_size = 2_000`
//! (fewer round-trips on hot session-start path; ~263k atuin rows →
//! ~131 pages vs ~263 pages; ~400 KB per page negligible).

use std::path::PathBuf;

/// Lower bound on `page_size` per V7 cluster-A plan.
pub const PAGE_SIZE_MIN: usize = 100;

/// Upper bound on `page_size` per V7 cluster-A plan.
pub const PAGE_SIZE_MAX: usize = 10_000;

/// Default `page_size` per D-B (Luke S1002127): 2_000.
pub const PAGE_SIZE_DEFAULT: usize = 2_000;

/// Default subprocess fallback timeout (ms).
pub const SUBPROCESS_TIMEOUT_DEFAULT_MS: u64 = 5_000;

/// Pagination + fallback configuration for [`super::AtuinConsumer`].
#[derive(Debug, Clone)]
pub struct AtuinConsumerConfig {
    /// Page size; clamped to `[PAGE_SIZE_MIN, PAGE_SIZE_MAX]` at
    /// effective-use time. Default `PAGE_SIZE_DEFAULT`.
    pub page_size: usize,
    /// Total ingest cap across all pages. `None` = unlimited.
    pub row_cap: Option<usize>,
    /// `atuin history list` subprocess fallback timeout (ms).
    pub subprocess_timeout_ms: u64,
    /// Override the default DB path (`~/.local/share/atuin/history.db`).
    pub db_path_override: Option<PathBuf>,
}

impl Default for AtuinConsumerConfig {
    fn default() -> Self {
        Self {
            page_size: PAGE_SIZE_DEFAULT,
            row_cap: None,
            subprocess_timeout_ms: SUBPROCESS_TIMEOUT_DEFAULT_MS,
            db_path_override: None,
        }
    }
}

impl AtuinConsumerConfig {
    /// `page_size` clamped to the canonical `[100, 10_000]` interval.
    #[must_use]
    pub fn effective_page_size(&self) -> usize {
        self.page_size.clamp(PAGE_SIZE_MIN, PAGE_SIZE_MAX)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        AtuinConsumerConfig, PAGE_SIZE_DEFAULT, PAGE_SIZE_MAX, PAGE_SIZE_MIN,
        SUBPROCESS_TIMEOUT_DEFAULT_MS,
    };

    #[test]
    fn default_matches_d_b_decision() {
        let c = AtuinConsumerConfig::default();
        assert_eq!(c.page_size, PAGE_SIZE_DEFAULT);
        assert_eq!(c.page_size, 2_000);
        assert!(c.row_cap.is_none());
        assert_eq!(c.subprocess_timeout_ms, SUBPROCESS_TIMEOUT_DEFAULT_MS);
        assert!(c.db_path_override.is_none());
    }

    #[test]
    fn effective_page_size_clamps_below_min() {
        let c = AtuinConsumerConfig {
            page_size: 0,
            ..Default::default()
        };
        assert_eq!(c.effective_page_size(), PAGE_SIZE_MIN);
    }

    #[test]
    fn effective_page_size_clamps_above_max() {
        let c = AtuinConsumerConfig {
            page_size: 1_000_000,
            ..Default::default()
        };
        assert_eq!(c.effective_page_size(), PAGE_SIZE_MAX);
    }

    #[test]
    fn effective_page_size_passes_through_when_in_range() {
        let c = AtuinConsumerConfig {
            page_size: 1234,
            ..Default::default()
        };
        assert_eq!(c.effective_page_size(), 1234);
    }

    #[test]
    fn db_path_override_is_honoured_via_struct_field() {
        let c = AtuinConsumerConfig {
            db_path_override: Some(PathBuf::from("/tmp/x.db")),
            ..Default::default()
        };
        assert_eq!(c.db_path_override.as_deref().map(std::path::Path::display).map(|d| d.to_string()), Some("/tmp/x.db".to_string()));
    }
}
