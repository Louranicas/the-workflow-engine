# scripts/

> **Back to:** [`../README.md`](../README.md) · [`../CLAUDE.md`](../CLAUDE.md) · [`../CLAUDE.local.md`](../CLAUDE.local.md)

**Status:** placeholder. Empty under the S1002127 scaffold waiver — populated post-G9.

This directory is the home for **helper bash scripts** that operate on the workflow-trace habitat surface (not Rust code). Post-G9 it carries: (a) habitat-side probe and deploy scripts (e.g. `probe-workflow-trace-health.sh`, `deploy-workflow-trace.sh`, `soak-workflow-trace.sh`); (b) `watcher notify` invocations for cross-pane communication (`watcher-notify-pre-deploy.sh` etc); (c) integration-test harnesses that exercise the Conductor dispatch path end-to-end; (d) ULTRAPLATE-style 4-stage gate wrappers per the workspace charter `~/claude-code-workspace/CLAUDE.md` § Quality Gate Protocol. **All scripts MUST avoid `set -e` traps** per workspace-charter [Shell Scripting Conventions](../CLAUDE.md), use `${PIPESTATUS[0]}` for cargo|tail chains, and never run `docker container prune -f` or other blanket destructive operations.

> **Back to:** [`../README.md`](../README.md) · [`../CLAUDE.md`](../CLAUDE.md) · [`../CLAUDE.local.md`](../CLAUDE.local.md)
