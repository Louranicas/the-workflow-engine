-- workflow-trace m1 (Substrate Ingest) — atuin recent-session probe
-- Reads last 100 tool calls / shell commands from the current pane's atuin history.
-- Per P-DATA-01: lazy cursor + PRAGMA query_only ON; busy_timeout 5000ms.
-- DB: ~/.local/share/atuin/history.db  (read-only — never UPDATE/DELETE)
--
-- Usage:
--   sqlite3 -header -column ~/.local/share/atuin/history.db < .claude/queries/atuin_recent_session.sql
--
-- Note: PRAGMA query_only must be set per-connection; sqlite3 CLI handles it inline.

PRAGMA query_only = ON;
PRAGMA busy_timeout = 5000;

-- Last 100 commands in this session, ordered most-recent-first.
-- session = atuin's session uuid; resolved via env at probe time.
-- cwd filter limits to the workflow-trace project root (escape-resistant via REPLACE for ~).
SELECT
  h.id,
  h.timestamp,
  substr(h.command, 1, 120) AS command_excerpt,
  h.exit,
  h.duration,
  h.cwd
FROM history h
WHERE h.session = (SELECT session FROM history WHERE timestamp = (SELECT MAX(timestamp) FROM history))
  AND h.cwd LIKE '%/the-workflow-engine%'
ORDER BY h.timestamp DESC
LIMIT 100;
