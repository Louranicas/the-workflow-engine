# migrations/

> **Back to:** [`../README.md`](../README.md) · [`../CLAUDE.md`](../CLAUDE.md) · [`../CLAUDE.local.md`](../CLAUDE.local.md)

**Status:** placeholder. Empty under the S1002127 scaffold waiver — populated post-G9 (concurrent with m7 build).

This directory carries the **SQLite migrations for the m7 `workflow_runs` central hub** (Cluster C — see [[Modules Synergy Clusters and Feature Verification S1001982]] and [`../ai_specs/modules/cluster-C/m7_workflow_runs.md`](../ai_specs/modules/cluster-C/m7_workflow_runs.md)). Convention: `NNN_<snake_case>.sql` (zero-padded; e.g. `001_workflow_runs_initial.sql` · `002_cascade_correlator_jsonb_column.sql` · `003_context_cost_ema_index.sql`). Each migration is forward-only (no `down.sql`); rollback is via restore-from-backup. Apply via `sqlx-cli` or the m7 module's startup migration runner — never by hand. The workflow_runs schema lands at v1 at first deploy; future migrations track schema-versioned upgrades (F9 zero-weight row preservation, JSONB column additions for new B-cluster observers, etc). See [`../ai_docs/runbooks/`](../ai_docs/runbooks/) post-G9 for the migration runbook.

> **Back to:** [`../README.md`](../README.md) · [`../CLAUDE.md`](../CLAUDE.md) · [`../CLAUDE.local.md`](../CLAUDE.local.md)
