# synthex-v2 Integration Map — workflow-engine ↔ synthex-v2

> Back to: [[HOME]] · [[MASTER_INDEX]] · `../CLAUDE.md` · `../CLAUDE.local.md`
>
> **Cross-vault canonical:** [External Engines Wire-In — workflow-engine + loop-engine-v2](../../synthex-v2/obsidian-synthex-v2/synthex-v2/schematics/External%20Engines%20Wire-In%20%E2%80%94%20workflow-engine%20%2B%20loop-engine-v2.md) (synthex-v2 vault) — full map with API tables, wire-in locations, and Mermaid diagrams. This note is the workflow-engine-side stub for round-trip discoverability.

## Wires (summary)

| Wire | Direction | Status | Source-of-truth |
|---|---|---|---|
| **W1** workflow-engine `m40_nexus_emit` → synthex-v2 `POST :8092/v3/nexus/push` | wfe → sxv2 | **LIVE** | `src/m40_nexus_emit/` ↔ synthex-v2 `src/daemon/http.rs:404` |
| **W2** synthex-v2 `GET :8092/v3/nexus/pull` ← workflow-engine | sxv2 → wfe (poll) | AVAILABLE, no consumer | synthex-v2 `src/daemon/http.rs:405` |
| **W3** synthex-v2 → workflow-engine subprocess driver | sxv2 → wfe | UNBUILT (recommended next) | _proposed:_ synthex-v2 `src/m6_action/m35k_workflow_engine_bridge.rs` invoking `wf-crystallise` / `wf-dispatch` |
| **W4** workflow-engine `m41_lcm_rpc` → LCM (endpoint cutover pending) | wfe → LCM | DESIGNED | `src/m41_lcm_rpc/` — current target `:8082/rpc`, will cut over to LCM `:8200/rpc` when LCM serves |

W1 is the production wire — `m40_nexus_emit::HttpNexusClient` posts `NexusEvent` records to synthex-v2 with outbox-first JSONL durability and a circuit breaker. Spec: `ai_specs/EVENT_SYSTEM_SPEC.md`.

## synthex-v2 expose surface (port 8092)

Full table in the cross-vault canonical. Key consumers used by workflow-engine:

- `POST /v3/nexus/push` — workflow-engine writes here (W1, LIVE)
- `GET /v3/nexus/pull` — available for polling (no consumer yet)
- `GET /metrics`, `GET /v3/thermal`, `GET /health` — read-only observability

## workflow-engine expose surface

CLI-only (no HTTP server). synthex-v2 can integrate via:

- Subprocess: `wf-crystallise [--proposals-out PATH]` (m1-m23 + m40-m42 pipeline) · `wf-dispatch [--proposals-in PATH] [--execute]` (m30-m33 bank/select/verify/dispatch)
- File-based: read `./workflow_runs.db` (m7 SQLite hub) and `./proposals.jsonl` (m23 output)
- Library: re-export from `workflow_core::*` (m1-m42 modules)

## Recommended landing for W3 (synthex-v2 side)

- Path: `synthex-v2/src/m6_action/m35k_workflow_engine_bridge.rs`
- Pattern: subprocess (precedent: `m35j_tlv2_bridge` — `hb` CLI)
- Error shape: `WfBridgeError::{CliNotFound, Timeout, NonZeroExit, InvalidJson, EmptyOutput}`
- Concurrency: `tokio::sync::Semaphore(2)`; timeout 120s on crystallise, 60s on dispatch
- AP discipline: AP32/AP33/AP34/AP36 (V8 hardening lineage)

## See also

- workflow-engine: [[Assessment Remediation S1003733]] · [[Hardening Fleet 2026-05-21]] · `../ai_specs/EVENT_SYSTEM_SPEC.md` · `../ai_docs/API_MAP.md`
- synthex-v2: cross-vault [External Engines Wire-In](../../synthex-v2/obsidian-synthex-v2/synthex-v2/schematics/External%20Engines%20Wire-In%20%E2%80%94%20workflow-engine%20%2B%20loop-engine-v2.md) · [Bridge Topology](../../synthex-v2/obsidian-synthex-v2/synthex-v2/schematics/Bridge%20Topology.md) (m35k slot reserved)
- LCM: cross-vault [HOME](../../loop-engine-v2/obsidian-lcm/HOME.md)

---

*Authored 2026-05-23 alongside the synthex-v2 vault's canonical wire-in schematic. Update when W3 lands or W4 cuts over to `:8200`.*
