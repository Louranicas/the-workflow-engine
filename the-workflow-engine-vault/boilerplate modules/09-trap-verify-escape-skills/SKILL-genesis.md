---
name: genesis
description: |
  End-to-end new Rust microservice creation for the ULTRAPLATE Habitat. Takes a service idea from
  zero to running: interactive plan.toml generation, scaffold via scaffold-gen, CLAUDE.md creation,
  git init, devenv.toml registration (port conflict detection, batch ordering, env vars, resource limits),
  release build, binary deploy, start, and health verification. Encodes 15+ trap points.
  Eliminates the 4-hour manual process (plan.toml authoring + scaffold + devenv registration + deploy).
  Triggers on: "genesis", "create new service", "new microservice", "new codebase", "start from scratch",
  "bootstrap a new project", "create from zero", or when user wants to create a new Rust service.
allowed-tools:
  - Bash
  - Read
  - Write
  - Edit
  - Grep
  - Glob
---

# /genesis — New Rust Service from Zero to Running

Creates a complete new Rust microservice: plan.toml, scaffold, CLAUDE.md, git, devenv registration, build, deploy, verify.

## Quick Start

```
/genesis                                              # Interactive — prompts for everything
/genesis --name my-service --port 8200 --batch 3      # Quick mode
/genesis --plan ./plan.toml                            # Plan-driven from existing plan
```

## 10-Phase Pipeline

### Phase 1: Gather Service Metadata

Claude asks for (or derives from arguments):
- **Service name** (kebab-case, unique across Habitat)
- **Description** (one-line purpose)
- **Port** (auto-assign from free range 8200-8250 if not specified)
- **Dependency batch** (1-5, based on which services it depends on)
- **Dependencies** (list of devenv service IDs)
- **Layer count** (default 8 for ORAC-style, or custom)
- **Module count** (typically 5-8 per layer)
- **Feature groups** (api, persistence, bridges, intelligence, monitoring, evolution)

**Port conflict detection:**
```bash
# Find first free port in 8200-8250 range
for p in $(seq 8200 8250); do
  if ! ss -tlnp "sport = :$p" 2>/dev/null | /usr/bin/grep -q "$p"; then
    # Also check devenv.toml for reserved ports
    if ! /usr/bin/grep -q "\"$p\"" ~/.config/devenv/devenv.toml 2>/dev/null; then
      PORT=$p; break
    fi
  fi
done
```

**Known port allocations (from devenv.toml):**
```
8082-V3  8083-Nerve  8090-SYNTHEX  8111-V8  8120-VMS  8125-POVM
8130-RM  8132-PV2  8133-ORAC  8180-ME  10002-Pswarm-V2
```

### Phase 2: Generate plan.toml

Claude creates a plan.toml following the proven schema (from ORAC's 528-line plan.toml):

```toml
[metadata]
name = "SERVICE_NAME"
description = "DESCRIPTION"
version = "0.1.0"
edition = "2021"
port = PORT
service_id = "SERVICE_ID"
devenv_batch = BATCH

[dependencies]
tokio = { version = "1", features = ["full"] }
axum = { version = "0.8", features = ["json"], optional = true }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
parking_lot = "0.12"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
ureq = "2"
uuid = { version = "1", features = ["v4"] }

[quality]
min_tests_per_module = 50
deny_unwrap = true
deny_unsafe = true
pedantic = true

[consent]
modulation_not_command = true
implementation_order = "recommended"

[[layers]]
key = "L1"
dir_name = "m1_core"
name = "Core"
description = "Foundation types, errors, config, constants, traits, validation"
depends_on = []

# ... additional layers based on user input
```

### Phase 3: Scaffold

```bash
scaffold-gen --from-plan "$PROJECT_DIR/plan.toml" "$PROJECT_DIR" 2>&1 | tail -10
scaffold-gen --verify "$PROJECT_DIR" 2>&1
```

### Phase 4: Create CLAUDE.md

Claude generates a project-specific CLAUDE.md with:
- Project header (name, status, port, LOC target, test target)
- Bootstrap protocol (generic, not ORAC-specific)
- Quality gate commands (using `/tmp/cargo-{name}` target dir)
- Architecture table (from plan.toml layers/modules)
- Rules (Rust gold standard from ORAC CLAUDE.md)
- Anti-patterns table
- Module organisation conventions
- Key constants (port, paths)
- Habitat Slash Commands table (from /propagate)
- Traps to avoid section

### Phase 5: Create .claude/ Scaffolding

```
.claude/
├── commands/        # Empty — service-specific commands added later
├── skills/          # Empty — service-specific skills added later
├── hooks/           # Empty — hooks wired via /integrate
├── schemas/         # Empty — JSON schemas added as needed
├── queries/         # Empty — SQL query templates added as needed
├── context.json     # Machine-readable module inventory from plan.toml
└── status.json      # Build phase tracking stub
```

### Phase 6: Git Init

```bash
cd "$PROJECT_DIR"
git init
cat > .gitignore << 'EOF'
/target
/tmp/cargo-*
*.db
*.db-journal
*.db-wal
*.log
.DS_Store
EOF
git add -A
git commit -m "Initial scaffold from plan.toml via /genesis

Layers: $LAYER_COUNT
Modules: $MODULE_COUNT
Port: $PORT
Batch: $BATCH

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>"
```

### Phase 7: Register with devenv.toml

Claude appends a `[[services]]` entry following the canonical schema (from fleet-gtr analysis):

```toml
[[services]]
id = "SERVICE_ID"
name = "SERVICE_NAME v0.1.0"
description = "DESCRIPTION (Port PORT)"
working_dir = "/home/louranicas/claude-code-workspace/PROJECT_DIR"
command = "./bin/SERVICE_BIN"
args = ["--port", "PORT"]
auto_start = true
auto_restart = true
max_restart_attempts = 5
restart_delay_secs = 3
health_check_interval_secs = 30
startup_timeout_secs = 30
shutdown_timeout_secs = 15
dependencies = ["DEP1", "DEP2"]

[services.env]
RUST_LOG = "SERVICE_NAME=info"
PORT = "PORT"

[services.resource_limits]
max_memory_mb = 256
max_cpu_percent = 25
```

**Validations before appending:**
1. `id` not already in devenv.toml
2. Port not claimed by another service
3. All dependencies exist as registered service IDs
4. Batch ordering is valid (deps must be in lower batches)

### Phase 8: Quick Gate + Build

```bash
cd "$PROJECT_DIR"
echo "━━━ Quick Gate ━━━"
CARGO_TARGET_DIR="/tmp/cargo-$SERVICE_ID" cargo check 2>&1 | tail -5
CARGO_TARGET_DIR="/tmp/cargo-$SERVICE_ID" cargo clippy -- -D warnings 2>&1 | tail -5

echo "━━━ Build Release ━━━"
CARGO_TARGET_DIR="/tmp/cargo-$SERVICE_ID" cargo build --release 2>&1 | tail -5
```

### Phase 9: Deploy + Start

```bash
# Deploy
/usr/bin/cp -f "/tmp/cargo-$SERVICE_ID/release/$BIN_NAME" "$HOME/.local/bin/$BIN_NAME"

# Start
nohup "$HOME/.local/bin/$BIN_NAME" > "/tmp/$BIN_NAME.log" 2>&1 &
echo "PID: $!"
```

### Phase 10: Verify + Report

```bash
sleep 3
HTTP_CODE=$(curl -s -o /dev/null -w '%{http_code}' "localhost:$PORT/health" 2>/dev/null)
echo ""
echo "━━━ GENESIS COMPLETE ━━━"
echo "Service:     $SERVICE_NAME"
echo "Port:        $PORT"
echo "Binary:      ~/.local/bin/$BIN_NAME"
echo "Health:      localhost:$PORT/health → $HTTP_CODE"
echo "Layers:      $LAYER_COUNT"
echo "Modules:     $MODULE_COUNT"
echo "DevEnv ID:   $SERVICE_ID (batch $BATCH)"
echo ""
echo "Next steps:"
echo "  1. Implement L1 Core first (bottom-up)"
echo "  2. /gate after each module"
echo "  3. /forge to redeploy after changes"
echo "  4. /integrate to wire into ORAC + PV2 + memories"
echo "  5. /propagate to push commands to this service"
```

## Dependencies

- `scaffold-gen` at ~/.local/bin/ (Python 3, 425 LOC)
- `devenv` at ~/.local/bin/ (Rust, 4.4MB)
- Rust toolchain (cargo, clippy)
- `python3`, `jq` for parsing

## Known Patterns (from fleet-gtr analysis)

### Resource Limit Tiers
| Tier | Memory | CPU | When to Use |
|------|--------|-----|-------------|
| Light | 64MB | 10% | Proxies, sidecars (ORAC) |
| Small | 128MB | 15% | Data stores (POVM, RM, PV2) |
| Medium | 256MB | 25% | Standard services (DevOps, NAIS, ME) |
| Large | 512MB | 50% | Heavy compute (SYNTHEX, K7, CS7) |

### Startup Timeout Tiers
| Tier | Timeout | When |
|------|---------|------|
| Fast | 15s | Simple health endpoint |
| Standard | 30s | DB initialization |
| Slow | 60s | Complex initialization (ORAC, SYNTHEX) |
| Heavy | 120s | Full model loading (VMS, CS7) |

## Metrics

| Metric | Without /genesis | With /genesis |
|--------|-----------------|---------------|
| Time to first health check | ~4 hours | ~15 minutes |
| Manual steps | 15+ | 1 command |
| Files manually created | 12+ | 0 (all generated) |
| Devenv registration errors | Common | Impossible (validated) |
| Port conflicts | Discovered at start time | Prevented at creation |
| Convention drift | Frequent | Zero (template-driven) |

## Cross-References

- **scaffold-gen:** ~/.local/bin/scaffold-gen (proven on ORAC, 40 modules)
- **devenv.toml schema:** fleet-gtr-devenv-registration.md (553 lines analyzed)
- **plan.toml schema:** orac-sidecar/plan.toml (528 lines, canonical reference)
- **CLAUDE.md template:** orac-sidecar/CLAUDE.md (211 lines, gold standard)
- **Fleet research:** fleet-alpha-lifecycle-gaps.md (31K, 14-step analysis)


---

> Vault navigation: [[../../BOILERPLATE_INDEX|BOILERPLATE_INDEX]] · [[../../README|boilerplate modules README]] · [[../../../HOME|HOME]] · [[../../../MASTER_INDEX|MASTER_INDEX]]
> Reference-only clone — see [[../../BOILERPLATE_INDEX]] for upstream source + target-module mapping.
