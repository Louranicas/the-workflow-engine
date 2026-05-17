# stcortex API

> SpacetimeDB module reducers + table read patterns. All data shapes documented as JSON Schema in `../schemas/*.schema.json`.

## Connection

> SpacetimeDB SDK/language support reference: [`SPACETIMEDB_LANGUAGE_SUPPORT.md`](SPACETIMEDB_LANGUAGE_SUPPORT.md). Back-link target from that page.

- Host: `127.0.0.1:3000`
- Database name: `stcortex`
- Database identity: `c200538073d27ef5785e77aa4355c6bbba2e132ebf03ab3f4bc434bea8cdec1e`
- Subscriptions: WebSocket via SpacetimeDB SDK (preferred for live consumers)
- Imperative calls: `~/.local/bin/spacetime call --server local stcortex <reducer> <args...>`
- Read queries: `~/.local/bin/spacetime sql --server local stcortex '<sql>'`
- Convenience CLI: `~/.local/bin/stcortex` (status/inspect/consumers/ghosts/sql/call/publish/backup)

## Reducers

### `register_consumer(name: String, namespace: String, transport: String) -> Result<(), String>`
Register as a subscriber to a namespace. **This is the precondition for writes** to the namespace.
- `name`: stable identifier (e.g., "save-session", "orac-learn", "<session-uuid>")
- `namespace`: project namespace, e.g. "claude-code"
- `transport`: one of "subscription" | "polling" | "mcp" | "cli"

Idempotent — re-registering an existing name updates `last_read_at` and clears stale flag.

### `heartbeat_consumer(name: String) -> Result<(), String>`
Refresh an existing consumer lease without creating synthetic read telemetry. This is the low-cost liveness path for polling/MCP/CLI batch writers between real `access_memory` calls. It updates `last_read_at` and clears `stale`; it does **not** increment `read_count_total` and does **not** write a `consumption_event` row.

### `unregister_consumer(name: String) -> Result<(), String>`
Explicit removal. Stale consumers (no reads in 30 days) are also auto-marked stale.

### `write_pathway(pre_id, post_id, namespace, weight: f32, session_id, instance_id: Option<String>) -> Result<(), String>`
Insert or additively merge a Hebbian pathway.
- **Refuses** if `consumer` table has zero fresh rows for `namespace` (unless `namespace == "scratch"`)
- Existing `(pre_id, post_id, namespace)` triple → reinforces (homeostatic, fights NA-1 winner-take-all)
- New triple → inserts with `reinforce_count = 1`

### `reinforce_pathway(pre_id, post_id, namespace, delta: f32) -> Result<(), String>`
Apply Hebbian LTP increment. `delta ∈ [0, 0.5]`. Errors if pathway not found.

### `write_memory(namespace, modality, content, intensity: f32, tensor: Option<Vec<f32>>, session_id, instance_id: Option<String>, parent_ids: Vec<u64>) -> Result<(), String>`
Insert a memory. **Refuses** like `write_pathway`.
- `modality`: one of `episodic` | `semantic` | `procedural` | `meta`
- `intensity ∈ [0, 2]`
- `tensor`: optional embedding (any dim) — L2-normalized, NaN/Inf rejected
- `parent_ids`: NA-14 lineage — ids of memories this was consolidated from

### `access_memory(memory_id: u64, consumer_name: String) -> Result<(), String>`
THE read path. **Always** increments `access_count` and sets `last_accessed = now()`. Records a `consumption_event` row (NA-7). Bumps consumer's `read_count_total`.

### `decay_step(_arg)` *(scheduled, every 60s)*
- Decays pathway weights by `0.995` per tick; prunes < `0.05`
- Decays memory intensities by modality (episodic 0.99, semantic 0.999, procedural 0.997, meta 1.0)
- Auto-crystallizes memories with `access_count >= 3 AND age > 24h`
- Ghost-traces pruned memories to `ghost_memory` (NA-4)
- Marks consumers stale after 30 days no reads
- Writes one `decay_audit` row

### `register_claude_session(session_id, project_path, project_namespace, transcript_path?, model_id?, parent_session_id?, zellij_session?, zellij_pane_id?)`
First-class session record with subagent lineage support.

### `end_claude_session(session_id)`
Sets `ended_at = now()` for cleanup.

### `record_tool_invocation(session_id, tool_name, args_hash, args_summary?, success, duration_ms?)`
Procedural memory. Hebbian pathways form between consecutive invocations.

### `add_cross_substrate_ref(memory_id, substrate, uri, anchor?)`
NA-9. `substrate ∈ {obsidian, rm, auto_memory, mcp_kg, transcript, hookify, povm, external}`.

### `cleanup_test_namespace(namespace, confirm)`
Restricted cleanup for integration/capacity namespaces only. Accepts `stcortex-bench`, `stcortex-bench-*`, or `it-*` namespaces when `confirm == "delete:<namespace>"`. Deletes matching `memory`, `pathway`, `pathway_lookup`, `consumer`, and `ghost_memory` rows. Intended for safe pressure-test cleanup, not production namespace pruning.

### `hydrate_pathway_lookup(namespace, max_rows)`
Bounded catch-up reducer for historical pathway rows created before the private composite lookup table existed. It inserts missing `pathway_lookup` entries for up to `max_rows` rows in `namespace`; `max_rows` must be `1..=10000`. New writes hydrate immediately, so this is only needed after migrations or imports.

## Hermes SDK hot-cache path

The fastest live integration is the generated Rust SDK sidecar:

```sh
cd ~/claude-code-workspace/stcortex/clients/rust-subscriber
cargo run --bin stcortex-hermes-cache -- --namespace hermes-agent
```

It registers `hermes-sdk-hot-cache` as a subscription consumer, subscribes over WebSocket, maintains hot memory/pathway/consumer caches, and atomically writes a compact JSON context-pack file at `data/live/hermes-agent-cache.json`. Use `--once` for a bootstrap refresh and long-lived mode for continuous Hermes/Habitat cache warming.

## Hermes MCP optimized path

The fastest Hermes-native integration is the local `stcortex-mcp` stdio bridge in `~/.local/bin/stcortex-mcp/index.js`. It uses direct local HTTP SQL for reads and the local `spacetime call --server local stcortex ...` reducer path for writes, avoiding browser/manual CLI round trips while preserving the database-level consumer gate.

Exposed tools:

| Tool | Purpose |
|---|---|
| `stcortex_status` | Reachability, consumers, recent decay audit. |
| `stcortex_register_consumer` | Fresh consumer registration; required before namespace writes. |
| `stcortex_write_memory` / `stcortex_write_pathway` | Low-level reducer wrappers. Optional Rust `Option<String>` args are encoded for the Spacetime CLI as JSON `{"some":"value"}` or `null`. |
| `stcortex_access_memory` | Meaningful read path; increments `access_count`, `last_accessed`, and consumption telemetry. |
| `stcortex_recall` | Agent-native associative recall from anchors/query; optionally performs meaningful access only for selected memories. |
| `stcortex_context_pack` | Turn-bootstrap bundle under a character budget; ranks by intensity, bounded access count, crystallization, anchor/pathway/query overlap. |
| `stcortex_write_observation` | One-call observation writer: registers consumer, writes memory, and optionally wires anchor pathways. |
| `stcortex_health_gradient` | Namespace metabolism/consumption diagnostics: write-only risk, active consumers, crystallization, strongest pathways. |
| `stcortex_query` | SELECT-only SQL passthrough for advanced inspection. |
| `stcortex_offline_snapshot` | JSON snapshot fallback when live service is down. |

Hermes use policy:

1. At session/tool bootstrap, call `stcortex_context_pack(namespace="hermes-agent", consumer_name="hermes-agent-native-mcp", anchors=[...], query=<task>, budget_chars=<context budget>)`.
2. For targeted retrieval mid-task, call `stcortex_recall` with `meaningful_access=true` only when the returned memories will be used. Use `meaningful_access=false` for exploratory previews.
3. After discovering durable procedures or stable environment facts, call `stcortex_write_observation` with `modality="procedural"` or `"semantic"` and anchors that should co-fire later.
4. Periodically call `stcortex_health_gradient` to catch stale consumers, write-only rows, and missing consumption telemetry.
5. If live SpacetimeDB is unavailable, call `stcortex_offline_snapshot` and continue read-only from the latest JSON snapshot.

Runtime result schemas added for these agent-native tools:

- `../schemas/recall-result.schema.json`
- `../schemas/context-pack.schema.json`
- `../schemas/observation-write.schema.json`
- `../schemas/health-gradient.schema.json`

## Read patterns (SQL)

Local v2.1.0 operator warning: the generic docs describe richer SQL, but the current stcortex REST/CLI SQL path has empirically rejected `ORDER BY`/`LIMIT` combinations in operator probes. Aliased aggregates such as `COUNT(*) AS n` work, but wrappers should use simple `SELECT`/`WHERE` in CLI examples and sort/limit client-side from SDK caches or JSON snapshots.

```sql
-- Memories in one namespace; sort/limit client-side
SELECT id, namespace, modality, content, access_count, crystallized
FROM memory WHERE namespace = 'claude-code';

-- Pathways from a specific anchor; sort by weight client-side
SELECT id, pre_id, post_id, namespace, weight, reinforce_count
FROM pathway WHERE pre_id = 'tool:Read';

-- Consumer health (who's actually reading?)
SELECT name, namespace, read_count_total, last_read_at, stale FROM consumer;

-- Decay history; sort by id/ts client-side
SELECT tick, pathways_decayed, memories_pruned, memories_crystallized, ts
FROM decay_audit;

-- NA-7 consumption telemetry; aggregate client-side
SELECT memory_id, consumer_name, ts
FROM consumption_event;

-- Ghost traces; sort/limit client-side
SELECT original_id, namespace, modality, content_preview, decayed_at
FROM ghost_memory;

-- Session lineage
SELECT session_id, parent_session_id, project_namespace, started_at, ended_at
FROM claude_session;
```

## Subscription patterns (SpacetimeDB SDK)

```sql
-- Live updates for a project's pathways
SUBSCRIBE TO pathway WHERE namespace = 'claude-code';

-- Live updates for memories above an intensity threshold
SUBSCRIBE TO memory WHERE intensity > 0.5;

-- Observability dashboard
SUBSCRIBE TO decay_audit;
SUBSCRIBE TO consumer;
```

## Errors

| Error message | Meaning |
|---|---|
| `stcortex: refusing write to namespace 'X' — no fresh consumer registered. ...` | The pioneer insight firing. Register a consumer or use `scratch`. |
| `weight must be finite in [0,1]` | Validation failure. |
| `tensor contains NaN/Inf` | POVM-lesson invariant — rejected before persistence. |
| `tensor norm is zero` | Cannot L2-normalize a zero vector. |
| `namespace 'X' must match [a-z0-9_-]+` | Validation. |
| `modality must be one of episodic/semantic/procedural/meta, got 'X'` | Validation. |

## SpacetimeDB SQL subset (REST `/v1/database/<id>/sql`)

Verified empirically S1001562 (cortex) + independently re-probed by Weaver. The REST endpoint accepts only a strict subset of SQL — wrappers must keep using simple `SELECT … WHERE` and do sorting/limiting/grouping client-side. **Subscription queries (via SDK)** support a slightly different subset; this table is for the REST endpoint only.

| Clause / feature | REST behaviour | Notes |
|---|---|---|
| `SELECT col, …` | ✅ accepted | |
| `WHERE col = X` | ✅ accepted | |
| compound `WHERE … AND …` | ✅ accepted | verified |
| `LIMIT N` | ✅ accepted | |
| `COUNT(*) AS alias` | ✅ accepted | **alias REQUIRED** — bare `COUNT(*)` errors `Aggregate expressions must have column aliases` |
| `ORDER BY col` | ❌ rejected | `HTTP 400: Unsupported` — sort client-side |
| `GROUP BY col` | ❌ rejected | aggregate client-side |
| `OFFSET N` | ❌ rejected | paginate via `WHERE id > last_seen` |
| `LIKE 'pattern%'` | ❌ rejected | `HTTP 400: Unsupported expression` — full scan + client filter, or use exact equals on indexed columns |
| `JOIN` | ❌ not supported | issue parallel queries + join client-side |

The Hermes MCP bridge (`stcortex_status`, `stcortex_inspect`) already does client-side sort/limit correctly. Custom callers should follow the same pattern.


---

> Vault navigation: [[../../BOILERPLATE_INDEX|BOILERPLATE_INDEX]] · [[../../README|boilerplate modules README]] · [[../../../HOME|HOME]] · [[../../../MASTER_INDEX|MASTER_INDEX]]
> Reference-only clone — see [[../../BOILERPLATE_INDEX]] for upstream source + target-module mapping.
