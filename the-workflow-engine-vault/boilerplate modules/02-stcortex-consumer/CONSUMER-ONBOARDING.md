# Consumer Onboarding — stcortex

> stcortex enforces consumer-presence at the database layer. Before you can write to a namespace, you must have a registered, fresh consumer for it. This is the architectural commitment that cures POVM's write-only failure mode.
>
> SpacetimeDB SDK/language support reference: [`SPACETIMEDB_LANGUAGE_SUPPORT.md`](SPACETIMEDB_LANGUAGE_SUPPORT.md). This page links back here from its bidirectional-links section.

## TL;DR

```sh
# 1. Register
stcortex call register_consumer 'my-app' 'my-project' 'cli'

# 2. Write
stcortex call write_pathway 'A' 'B' 'my-project' 0.5 'session-uuid' null

# 3. Read (for the POVM-cure invariants to fire)
stcortex call access_memory 1 'my-app'   # increments access_count + sets last_accessed
stcortex sql "SELECT * FROM memory WHERE namespace = 'my-project'"
```

## What "consumer" means

A row in the `consumer` table:

```sql
SELECT name, namespace, transport, last_read_at, stale, read_count_total FROM consumer;
```

| field | purpose |
|---|---|
| `name` | Stable identifier you choose (e.g. session UUID, skill name, service name) |
| `namespace` | Which slice of stcortex you're consuming |
| `transport` | One of `subscription`, `polling`, `mcp`, `cli` (informational; doesn't change behaviour) |
| `last_read_at` | Auto-updated whenever you call `access_memory`; if > 30 days ago you go `stale` |
| `stale` | `true` after 30 days no-read; refuse-write enforcement ignores stale consumers |

A namespace must have **at least one fresh** (non-stale) consumer for writes to succeed. The `scratch` namespace is the only exception (no consumer required, 24h TTL).

## Onboarding checklist

For any new client (script, service, agent, skill):

1. **Pick a stable name.** Avoid randomness — use `<service>-<role>` or `cc-<session-id>`.
2. **Pick a namespace.** Project namespace conventions:
   - Per-project: `sha256(git_remote || abs_path)[:12]` (the SessionStart hook does this automatically for Claude Code)
   - Habitat-shared: `claude-code`, `orac-learn`, `pane-vortex`, `obsidian` (well-known names)
   - Ephemeral: `scratch` (24h TTL, no enforcement)
3. **Register before any writes.** Idempotent — safe to call on every startup.
4. **Read regularly.** A consumer that doesn't `access_memory` for 30 days goes stale and is ignored by enforcement.
5. **Unregister cleanly on shutdown** *(optional)* — `unregister_consumer 'name'`. Stale GC handles it eventually anyway.

## Patterns by client type

### Claude Code session (auto via hooks)

You don't have to do anything. `~/.claude/hooks/stcortex-session-start.sh` runs on every SessionStart and registers `cc-<session-id>` for the project namespace. The Stop hook closes the session and writes a meta-modality summary memory.

If you're inside Claude Code, you can also use:
- The `/stcortex` slash command for inline querying
- The `stcortex_*` MCP tools (via `mcp-server-stcortex` registered in `~/claude-code-workspace/.mcp.json`)

### Bash / shell script

```bash
#!/bin/sh
NS="my-namespace"
NAME="my-script-$(hostname)"

# Register (idempotent)
stcortex call register_consumer "$NAME" "$NS" "cli"

# Write
stcortex call write_pathway 'A' 'B' "$NS" 0.5 "$$" null

# Read
stcortex sql "SELECT * FROM memory WHERE namespace = '$NS'"
```

### Python (urllib, no SDK)

```python
import json, urllib.request, subprocess, uuid

NS = "my-app"
NAME = "py-worker-1"
HOST = "http://127.0.0.1:3000"
DB = "stcortex"

def call(reducer, *args):
    """Reducer call via spacetime CLI — most reliable for typed args."""
    return subprocess.run(
        ["/home/louranicas/.local/bin/spacetime", "call", "--server", "local", DB, reducer, *args],
        check=False, capture_output=True, text=True,
    )

def query(sql):
    """SQL via REST."""
    req = urllib.request.Request(
        f"{HOST}/v1/database/{DB}/sql",
        data=sql.encode(), headers={"Content-Type": "text/plain"}, method="POST",
    )
    with urllib.request.urlopen(req, timeout=5) as r:
        return json.loads(r.read())

# Register
call("register_consumer", NAME, NS, "polling")

# Write a memory
call("write_memory", NS, "semantic", "first python memory", "1.0", "null", str(uuid.uuid4()), "null", "[]")

# Read
print(query(f"SELECT id, content FROM memory WHERE namespace = '{NS}'"))
```

### TypeScript / Node (via the MCP server, recommended)

If you can MCP, use it. Configure in your MCP client (Claude Code, Cursor, etc.):

```json
{
  "mcpServers": {
    "stcortex-mcp": {
      "command": "node",
      "args": ["/home/louranicas/.local/bin/stcortex-mcp/index.js"],
      "env": { "STCORTEX_HOST": "http://127.0.0.1:3000", "STCORTEX_DB": "stcortex" }
    }
  }
}
```

Then call tools: `stcortex_register_consumer`, `stcortex_write_memory`, `stcortex_access_memory`, `stcortex_recall`, `stcortex_context_pack`, `stcortex_query`, `stcortex_offline_snapshot`.

### Rust (SpacetimeDB SDK — for real subscriptions)

```rust
// Cargo.toml: spacetimedb-sdk = { path = "/home/louranicas/claude-code-workspace/spacetimedb/sdks/rust" }

use spacetimedb_sdk::DbContext;
use module_bindings::DbConnection;

let conn = DbConnection::builder()
    .with_uri("ws://127.0.0.1:3000")
    .with_database_name("stcortex")
    .build()?;

// Register as consumer (one-shot reducer call)
conn.reducers.register_consumer("my-rust-app".into(), "my-ns".into(), "subscription".into())?;

// Subscribe — pushes deltas as data changes
conn.subscription_builder()
    .on_applied(|_| println!("initial data loaded"))
    .subscribe(["SELECT * FROM pathway WHERE namespace = 'my-ns'"]);

conn.db.pathway().on_insert(|_, p| {
    println!("new pathway: {} -> {} (w={})", p.pre_id, p.post_id, p.weight);
});

conn.run_threaded();   // blocks; use run_async() for async
```

The Rust SDK is the only path that gets real WebSocket-pushed deltas. CLI/Python/MCP all poll. For consumers that need low-latency reactivity (e.g. ORAC Learn phase), prefer the Rust SDK.

## How the architectural commitment fires

When you call `write_pathway` or `write_memory`, the reducer body executes:

```rust
let consumer_count = ctx.db.consumer().iter()
    .filter(|c| c.namespace == namespace && !c.stale)
    .count();
if consumer_count == 0 && namespace != "scratch" {
    return Err("stcortex: refusing write to namespace 'X' — no fresh consumer registered ...");
}
```

The error propagates back to the caller as a 530 HTTP status with the descriptive message. **Treat refused writes as architectural feedback, not bugs.**

## POVM-lesson invariants you get for free

Every consumer benefits from these without doing anything:

- `access_memory` ALWAYS increments `access_count` and sets `last_accessed = now()`. POVM V1 left these null forever; here they're populated by the only read path.
- `write_memory` L2-normalizes any tensor you pass. POVM had stdev 66147 vs mean 44730 due to no normalization.
- `decay_step` (every 60s) auto-crystallises memories that meet `access_count ≥ 3 AND age > 24h`. POVM had 0/100 crystallised despite the logic existing.
- Pruned memories leave a `ghost_memory` archive (NA-4) — `stcortex ghosts` to inspect.
- Every `access_memory` writes a `consumption_event` row (NA-7) — actionable beyond aggregate counts.

## Cross-substrate references

If your memory cites an Obsidian note, a transcript line, an RM entry, or a hookify rule, record the link:

```sh
stcortex call add_cross_substrate_ref <memory_id> <substrate> <uri> <anchor?>
# substrate ∈ {obsidian, rm, auto_memory, mcp_kg, transcript, hookify, povm, external}
```

This makes inter-substrate references first-class and queryable.

## Related docs

- `API.md` — reducer signatures and SQL patterns
- `RUNBOOK.md` — operator runbook (start/stop/restore/tuning)
- `~/.claude/projects/-home-louranicas/memory/stcortex.md` — operator memory note


---

> Vault navigation: [[../../BOILERPLATE_INDEX|BOILERPLATE_INDEX]] · [[../../README|boilerplate modules README]] · [[../../../HOME|HOME]] · [[../../../MASTER_INDEX|MASTER_INDEX]]
> Reference-only clone — see [[../../BOILERPLATE_INDEX]] for upstream source + target-module mapping.
