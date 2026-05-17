# ANTIPATTERNS — workflow-trace

> **Canonical (authoritative, growing):** [`ai_docs/optimisation-v7/ANTIPATTERNS_REGISTER.md`](ai_docs/optimisation-v7/ANTIPATTERNS_REGISTER.md)
> **This file:** summary index + cross-link. Do NOT add a new entry here; add it to the canonical register and update this index.
> **Machine-readable (for `.claude/` hooks):** [`.claude/anti_patterns.json`](.claude/anti_patterns.json)

---

## Workflow-trace antipatterns (AP-V7-*)

| ID | Title | Severity | One-line | Canonical entry |
|---|---|---|---|---|
| **AP-V7-01** | Health-200 ≠ behaviour-verified | HIGH | A 200 on `/health` does not prove the binary is the calibrated one | [register](ai_docs/optimisation-v7/ANTIPATTERNS_REGISTER.md#ap-v7-13) |
| **AP-V7-02** | Silence-as-consent in multi-pane orchestration | MEDIUM | If peer panes don't reply, you don't have consent — you have silence (AP-V7-08) | [register](ai_docs/optimisation-v7/ANTIPATTERNS_REGISTER.md#ap-v7-08) |
| **AP-V7-03** | Verb collapse across Phase A/B boundary | HIGH | Phase A passive verbs (record, ingest) must not leak into Phase B active verbs (recommend, dispatch) | [register](ai_docs/optimisation-v7/ANTIPATTERNS_REGISTER.md) |
| **AP-V7-04** | POVM dual-path coupling | RESOLVED | m42 was POVM-dual-path; pivoted to stcortex-only 2026-05-17 ([ADR](ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md)) | [ADR](ai_docs/optimisation-v7/decisions/2026-05-17-m42-stcortex-only-pivot.md) |
| **AP-V7-05** | Module-count drift (28/11/25/26 across artefacts) | RESOLVED | OI-3 reconciled at **26** modules with m1-m42 unpadded | [v1.3 § 1](ai_docs/GENESIS_PROMPT_V1_3.md) |
| **AP-V7-06** | Padded vs unpadded module IDs (`m01` vs `m1`) | RESOLVED | OI-4 locked at **unpadded** throughout (`m1`, `m2`, `m11`, `m42`) | [v1.3 § 1](ai_docs/GENESIS_PROMPT_V1_3.md) |
| **AP-V7-07** | Auto-promotion m23 → m30 (proposer-to-bank without human review) | HIGH | m23 proposes; humans review; m30 accepts — bypass = AP-Hab refusal-mode violation | [register](ai_docs/optimisation-v7/ANTIPATTERNS_REGISTER.md) |
| **AP-V7-08** | Self-dispatch via m32 | HIGH | m32 dispatches workflows; m32 itself is not a workflow; recursion trap | [register](ai_docs/optimisation-v7/ANTIPATTERNS_REGISTER.md) |
| **AP-V7-09** | Cluster D not woven first | HIGH | Trust aspect-layer (m8/m9/m10/m11) must ship Day 1 before any reader; retrofit costs audit of every call site | [phase-1](the-workflow-engine-vault/deployment%20framework/phase-1-genesis-day-0-3.md) |
| **AP-V7-10** | LCM workspace-of-crates pattern when ORAC single-crate is correct | MEDIUM | Single deployment target → ORAC pattern; LCM pattern only when independently-releasable concerns | [G2-consolidation](ai_docs/optimisation-v7/GENERATIONS/G2-consolidation.md) |
| **AP-V7-11** | Active verbs in Phase A modules (m1-m15) | RESOLVED | v1.2 verb-lock active; Phase A is passive: ingest · correlate · record · emit · refuse | [v1.3 § 1](ai_docs/GENESIS_PROMPT_V1_3.md) |
| **AP-V7-12** | `Cargo.toml`-via-`cargo init` inside workspace | MEDIUM | Add `[workspace]` to isolate; even better — do not `cargo init` pre-G9 | [scaffold-mastery](../.claude/skills/scaffold-mastery/SKILL.md) |
| **AP-V7-13** | Diagnostics theatre: refresh-date stamps without re-executed probes | MEDIUM | Refreshed:DATE on runbooks/docs MUST be paired with `> probes re-verified live DATE` evidence | [auto-memory feedback_runbook_probe_freshness](../../.claude/projects/-home-louranicas-claude-code-workspace/memory/feedback_runbook_probe_freshness.md) |

> **AP-V7-13 scope expansion (Wave 4, NA-GAP-07):** Health-200 ≠ behaviour-verified is no longer scattered as defensive sub-clauses inside m32 and m42. It is now elevated to first-class substrate-drift detection per [`ai_specs/cross-cutting/substrate-drift.md`](ai_specs/cross-cutting/substrate-drift.md), with per-substrate drift indicators enumerated in each substrate dossier at [`ai_specs/substrates/`](ai_specs/substrates/) § 4, a `SubstrateDriftCanary` trait contract, and `SubstrateDriftDetected` event wiring into m15 pressure register. CR-2 POVM `learning_health` 13.6× inflation is the canonical case (S-C dossier § 4).

---

## Habitat antipatterns (AP-Hab-*; carries from workspace CLAUDE.md)

| ID | Title | Reference |
|---|---|---|
| **AP24** | Coding without explicit `start coding <project>` signal | Workspace [CLAUDE.md](../CLAUDE.md) · [feedback_wait_for_start_coding](../../.claude/projects/-home-louranicas-claude-code-workspace/memory/feedback_wait_for_start_coding.md) |
| **AP27** | Watcher self-modification | Workspace [CLAUDE.md](../CLAUDE.md) · [The Watcher persona](../synthex-v2/obsidian-synthex-v2/synthex-v2/The%20Watcher.md) |
| **AP29** | Sync HTTP in `tokio::spawn` (starves runtime) | Workspace auto-memory [session-225](../../.claude/projects/-home-louranicas-claude-code-workspace/memory/session-225-orac-restoration.md) |
| **AP30** | Namespace string drift (use `namespace.rs` constants, never literals) | [G2-consolidation](ai_docs/optimisation-v7/GENERATIONS/G2-consolidation.md) § Canonical src/ layout |
| **AP32-37** | V8 OTP integration gotchas (CLI hang / binary absence / JSON fallback / restart storm / FD exhaustion / `/health` path) | Workspace auto-memory [session-104](../../.claude/projects/-home-louranicas-claude-code-workspace/memory/session-104.md) |

---

## How to add a new antipattern

1. Add the entry to the **canonical register** [`ai_docs/optimisation-v7/ANTIPATTERNS_REGISTER.md`](ai_docs/optimisation-v7/ANTIPATTERNS_REGISTER.md) with: ID, title, observed-in (session/sub-agent/incident), root cause, mitigation, related-patterns
2. Update the machine-readable [`.claude/anti_patterns.json`](.claude/anti_patterns.json) so `.claude/hooks/` can detect future occurrences
3. Add the one-line summary row above with link to canonical
4. If the antipattern is a refusal-mode violation, add a Bash-pattern block hook in [`.claude/hooks/`](.claude/hooks/)

---

> **Back to:** [`README.md`](README.md) · [`PATTERNS.md`](PATTERNS.md) · [`GOLD_STANDARDS.md`](GOLD_STANDARDS.md) · [`ARCHITECTURE.md`](ARCHITECTURE.md)
