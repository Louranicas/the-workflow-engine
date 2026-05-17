//! Cursor primitive for paginated atuin reads.
//!
//! Per m1 spec § 3 + S1002211 schema-discovery amendment: `ConsumerState`
//! carries the monotonic `last_id` (an empty string before the first
//! page; advances to the largest ULID seen each page), the running
//! `rows_yielded` count (to enforce `row_cap`), and the sticky `exhausted`
//! flag.
//!
//! ULIDs are lexicographically sortable by issue time, so the SQL
//! `WHERE id > ?1 ORDER BY id ASC LIMIT ?2` cursor remains correct with a
//! string key (the original integer-PK spec assumption is corrected here).

use super::config::AtuinConsumerConfig;

/// Cursor state for the m1 paginated reader.
#[derive(Debug)]
pub struct ConsumerState {
    /// Exclusive lower bound (ULID string) for the next SELECT. Empty
    /// string before the first page (lexicographically `< any ULID`).
    pub last_id: String,
    /// Running count enforcing `config.row_cap`.
    pub rows_yielded: usize,
    /// Sticky exhaustion flag.
    pub exhausted: bool,
    /// Configuration snapshot.
    pub config: AtuinConsumerConfig,
}

impl ConsumerState {
    /// Construct a fresh cursor at the start of the table.
    #[must_use]
    pub fn new(config: AtuinConsumerConfig) -> Self {
        Self {
            last_id: String::new(),
            rows_yielded: 0,
            exhausted: false,
            config,
        }
    }

    /// Effective page size for this iteration, honouring the configured
    /// page-size clamp AND the remaining `row_cap` budget (so the final
    /// page is exactly-trimmed instead of overshooting).
    #[must_use]
    pub fn effective_page_size(&self) -> usize {
        let base = self.config.effective_page_size();
        match self.config.row_cap {
            Some(cap) => base.min(cap.saturating_sub(self.rows_yielded)),
            None => base,
        }
    }

    /// Update the cursor after a successful page fetch.
    pub fn advance(&mut self, new_last_id: String, page_len: usize, base_page_size: usize) {
        self.last_id = new_last_id;
        self.rows_yielded = self.rows_yielded.saturating_add(page_len);
        if page_len < base_page_size {
            self.exhausted = true;
        }
        if let Some(cap) = self.config.row_cap {
            if self.rows_yielded >= cap {
                self.exhausted = true;
            }
        }
    }

    /// Mark exhausted without advancing — used when a SELECT returns
    /// zero rows.
    pub fn mark_exhausted(&mut self) {
        self.exhausted = true;
    }
}

#[cfg(test)]
mod tests {
    use super::super::config::AtuinConsumerConfig;
    use super::ConsumerState;

    #[test]
    fn fresh_cursor_starts_at_empty_string_not_exhausted() {
        let s = ConsumerState::new(AtuinConsumerConfig::default());
        assert_eq!(s.last_id, "");
        assert_eq!(s.rows_yielded, 0);
        assert!(!s.exhausted);
    }

    #[test]
    fn effective_page_size_without_cap_matches_config() {
        let s = ConsumerState::new(AtuinConsumerConfig::default());
        assert_eq!(s.effective_page_size(), 2_000);
    }

    #[test]
    fn effective_page_size_with_cap_trims_to_remaining() {
        let cfg = AtuinConsumerConfig {
            page_size: 1_000,
            row_cap: Some(1_500),
            ..AtuinConsumerConfig::default()
        };
        let mut s = ConsumerState::new(cfg);
        s.rows_yielded = 1_000;
        assert_eq!(s.effective_page_size(), 500);
    }

    #[test]
    fn effective_page_size_with_cap_already_reached_is_zero() {
        let cfg = AtuinConsumerConfig {
            page_size: 1_000,
            row_cap: Some(500),
            ..AtuinConsumerConfig::default()
        };
        let mut s = ConsumerState::new(cfg);
        s.rows_yielded = 500;
        assert_eq!(s.effective_page_size(), 0);
    }

    #[test]
    fn advance_updates_cursor_and_count() {
        let mut s = ConsumerState::new(AtuinConsumerConfig::default());
        s.advance("01HQA-ulid-42".to_owned(), 2_000, 2_000);
        assert_eq!(s.last_id, "01HQA-ulid-42");
        assert_eq!(s.rows_yielded, 2_000);
        assert!(!s.exhausted);
    }

    #[test]
    fn advance_with_short_page_marks_exhausted() {
        let mut s = ConsumerState::new(AtuinConsumerConfig::default());
        s.advance("01HQA-ulid-100".to_owned(), 50, 2_000);
        assert!(s.exhausted);
    }

    #[test]
    fn advance_with_cap_reached_marks_exhausted() {
        let cfg = AtuinConsumerConfig {
            row_cap: Some(10),
            ..AtuinConsumerConfig::default()
        };
        let mut s = ConsumerState::new(cfg);
        s.advance("01HQA-ulid-10".to_owned(), 10, 2_000);
        assert!(s.exhausted);
    }

    #[test]
    fn mark_exhausted_sets_flag() {
        let mut s = ConsumerState::new(AtuinConsumerConfig::default());
        s.mark_exhausted();
        assert!(s.exhausted);
    }

    #[test]
    fn cursor_monotonic_lexicographic_across_advances() {
        // ULIDs are lexicographically sortable.
        let mut s = ConsumerState::new(AtuinConsumerConfig::default());
        s.advance("01HQA-aaa".to_owned(), 2_000, 2_000);
        let id1 = s.last_id.clone();
        s.advance("01HQA-bbb".to_owned(), 2_000, 2_000);
        assert!(s.last_id > id1);
    }
}
