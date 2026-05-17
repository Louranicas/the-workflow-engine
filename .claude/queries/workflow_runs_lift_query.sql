-- workflow-trace m7 (hub) + m14 (lift evidence) — Wilson CI lift over workflow_runs
-- Computes per-workflow lift (= success_rate of variant / success_rate of baseline)
-- with Wilson confidence interval. Returns NULL CI for n < 20 (P-DATA-06 — avoids
-- point-estimate over-claim).
--
-- DB: ~/.local/share/workflow-trace/db.sqlite (post-G9 created; pre-G9 this SQL is
--     spec-only against the workflow_runs_row.schema.json shape).
--
-- Usage:
--   sqlite3 -header -column ~/.local/share/workflow-trace/db.sqlite < .claude/queries/workflow_runs_lift_query.sql

PRAGMA query_only = ON;
PRAGMA busy_timeout = 5000;

-- Wilson CI z = 1.96 (95% confidence). Lift = success(variant) / success(baseline).
-- A workflow's "baseline" is its parent_workflow_id; "variant" is the row itself.
-- Outcome flag: PassVerified or Pass = success; Fail/Blocked/Refused = failure;
-- Pending/Active = excluded (insufficient evidence).
WITH per_workflow_stats AS (
  SELECT
    wr.id,
    wr.parent_workflow_id,
    COUNT(*)                                                          AS n_total,
    SUM(CASE WHEN wr.outcome IN ('PassVerified', 'Pass') THEN 1 ELSE 0 END) AS n_success,
    SUM(CASE WHEN wr.outcome IN ('Fail', 'Blocked', 'Refused')       THEN 1 ELSE 0 END) AS n_failure
  FROM workflow_runs wr
  WHERE wr.outcome IN ('PassVerified', 'Pass', 'Fail', 'Blocked', 'Refused')
  GROUP BY wr.id, wr.parent_workflow_id
),
wilson AS (
  SELECT
    pws.id,
    pws.parent_workflow_id,
    pws.n_total,
    pws.n_success,
    CASE WHEN pws.n_total >= 20
      THEN (CAST(pws.n_success AS REAL) / pws.n_total) END                  AS rate,
    -- Wilson lower bound (z=1.96 -> z^2 = 3.8416)
    CASE WHEN pws.n_total >= 20
      THEN ((CAST(pws.n_success AS REAL) + 1.9208)
            - 1.96 * sqrt((CAST(pws.n_success AS REAL) * (pws.n_total - pws.n_success) / pws.n_total) + 0.9604))
           / (pws.n_total + 3.8416)
    END AS wilson_lower,
    CASE WHEN pws.n_total >= 20
      THEN ((CAST(pws.n_success AS REAL) + 1.9208)
            + 1.96 * sqrt((CAST(pws.n_success AS REAL) * (pws.n_total - pws.n_success) / pws.n_total) + 0.9604))
           / (pws.n_total + 3.8416)
    END AS wilson_upper
  FROM per_workflow_stats pws
)
SELECT
  v.id            AS variant_id,
  v.parent_workflow_id AS baseline_id,
  v.n_total       AS variant_n,
  v.rate          AS variant_rate,
  v.wilson_lower  AS variant_wilson_lower,
  v.wilson_upper  AS variant_wilson_upper,
  b.n_total       AS baseline_n,
  b.rate          AS baseline_rate,
  CASE WHEN b.rate IS NOT NULL AND b.rate > 0 AND v.rate IS NOT NULL
    THEN v.rate / b.rate END AS lift
FROM wilson v
LEFT JOIN wilson b ON b.id = v.parent_workflow_id
WHERE v.parent_workflow_id IS NOT NULL
ORDER BY lift DESC NULLS LAST
LIMIT 20;
