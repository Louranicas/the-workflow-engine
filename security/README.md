# security/

> **Back to:** [`../README.md`](../README.md) · [`../SECURITY.md`](../SECURITY.md) · [`../CLAUDE.md`](../CLAUDE.md) · [`../CLAUDE.local.md`](../CLAUDE.local.md)

**Status:** placeholder. Empty under the S1002127 scaffold waiver — populated as needed.

This directory holds **security policies + disclosure procedure artefacts** that don't fit cleanly in the root [`../SECURITY.md`](../SECURITY.md) (the canonical disclosure surface; the file you should read first). Anticipated post-G9 contents: (a) `THREAT_MODEL.md` (STRIDE-style threat enumeration covering m32 Conductor dispatch path, m13/m42 stcortex write side, m9 namespace-guard escape surface); (b) `SECRETS_HANDLING.md` (PAT rotation, never-commit checklist; cross-link to workspace-charter S1001883 PAT-rotation lessons); (c) `INCIDENT_RUNBOOK.md` (what to do when m32 dispatches a workflow it shouldn't have); (d) per-CVE response notes. **All security disclosures route through the channel in [`../SECURITY.md`](../SECURITY.md) — do not file issues here directly.** ASP-vault-class sensitivity isolation per workspace-charter `~/claude-code-workspace/CLAUDE.md` § PRIME DIRECTIVE applies.

> **Back to:** [`../README.md`](../README.md) · [`../SECURITY.md`](../SECURITY.md) · [`../CLAUDE.md`](../CLAUDE.md) · [`../CLAUDE.local.md`](../CLAUDE.local.md)
