-- workflow-trace m3 (Substrate Ingest — injection.db) — unresolved chains
-- Reads ~/.local/share/habitat/injection.db: 74 patterns + 40 causal chains.
-- Per workspace CLAUDE.md "Memory Injection" pattern: causal_chain rows with
-- resolved_session IS NULL are the "still-open" learning opportunities — these
-- feed m3 substrate-ingest and downstream m22 confidence calculation.
--
-- Usage:
--   sqlite3 -header -column ~/.local/share/habitat/injection.db < .claude/queries/injection_db_open_chains.sql

PRAGMA query_only = ON;
PRAGMA busy_timeout = 5000;

-- Top 20 unresolved causal chains by reinforcement_count desc.
-- Chains with high reinforcement_count are highest-leverage learning opportunities;
-- m3 prioritises these for workflow-trace ingestion + m23 proposer seeding.
SELECT
  cc.id,
  cc.label,
  cc.reinforcement_count,
  cc.first_observed_ts,
  cc.last_observed_ts,
  cc.confidence,
  cc.source_pattern_id,
  substr(cc.summary, 1, 100) AS summary_excerpt
FROM causal_chain cc
WHERE cc.resolved_session IS NULL
ORDER BY cc.reinforcement_count DESC, cc.last_observed_ts DESC
LIMIT 20;
