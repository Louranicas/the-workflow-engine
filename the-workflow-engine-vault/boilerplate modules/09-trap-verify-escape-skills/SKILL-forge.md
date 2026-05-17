---
name: forge
description: |
  Generic build + deploy + verify pipeline for ANY Rust service in the ULTRAPLATE Habitat.
  Auto-detects project from CWD via Cargo.toml, resolves binary name, features, port, health path,
  and environment variables from devenv.toml. Encodes 8 Habitat traps (cp alias, pkill exit 144,
  SIGPIPE, port conflicts, workspace handling, feature detection, health path variance, stale PIDs).
  Replaces the ORAC-specific /deploy-orac with a universal command.
  Triggers on: "forge", "build and deploy", "deploy service", "release build", "redeploy",
  "build release", or when user wants to build+deploy any Rust service.
allowed-tools:
  - Bash
  - Read
  - Grep
  - Glob
---

# /forge — Generic Build + Deploy + Verify for ANY Rust Service

Build, deploy, and verify any Rust service in the Habitat. Works from any project directory.

## Quick Start

Run `/forge` from any Rust project directory. It auto-detects everything.

## Resolution Algorithm

Before executing, resolve these 8 parameters from Cargo.toml + devenv.toml:

### Step 1: Find Cargo.toml

```bash
# Check CWD, then walk up to 3 parents
CARGO_TOML=""
for d in . .. ../.. ../../..; do
  [[ -f "$d/Cargo.toml" ]] && CARGO_TOML="$(cd "$d" && pwd)/Cargo.toml" && break
done
[[ -z "$CARGO_TOML" ]] && echo "ERROR: No Cargo.toml found" && exit 1
PROJECT_DIR=$(dirname "$CARGO_TOML")
```

### Step 2: Extract binary name

Read Cargo.toml. Priority order:
1. `[[bin]]` section with `name` field (first entry)
2. `[package].name` (automatic binary name)
3. Fall back to `cargo metadata --format-version 1` (slow but authoritative)

**Known variances:**
- pane-vortex-v2: package=`pane-vortex-v2`, binary=`pane-vortex`
- the_code_synthor_v7: package=`codesynthor`, binary=`codesynthor`
- Binary name != package name != directory name — ALWAYS check `[[bin]]` first

### Step 3: Detect workspace membership

```bash
# Check if parent has [workspace] with this crate as member
PARENT_CARGO="$(dirname "$PROJECT_DIR")/Cargo.toml"
IS_WORKSPACE=false
PACKAGE_FLAG=""
if [[ -f "$PARENT_CARGO" ]]; then
  if /usr/bin/grep -q '\[workspace\]' "$PARENT_CARGO" 2>/dev/null; then
    IS_WORKSPACE=true
    PACKAGE_FLAG="-p $PACKAGE_NAME"
    PROJECT_DIR=$(dirname "$PARENT_CARGO")  # Build from workspace root
  fi
fi
```

**Known workspace members:** povm_engine (nexus_forge workspace, 11 crates)

### Step 4: Determine CARGO_TARGET_DIR

```bash
if $IS_WORKSPACE; then
  CARGO_TARGET="/tmp/cargo-$(basename "$(dirname "$PARENT_CARGO")")"
else
  CARGO_TARGET="/tmp/cargo-$PACKAGE_NAME"
fi
```

### Step 5: Detect feature flags

Read `[features]` section from Cargo.toml:
- Has `full` key → `--features full`
- Has `api` key NOT in default features → `--features api` (SYNTHEX pattern)
- Otherwise → no feature flag

**Known patterns:**
| Service | Features |
|---------|----------|
| orac-sidecar | `--features full` (6 groups) |
| codesynthor-v7 | `--features full` (sandbox+ai-native+deep-intel) |
| synthex | `--features api` (binary requires explicit api feature) |
| pane-vortex-v2 | default (full adds evolution) |
| All others | default |

### Step 6: Match against devenv.toml

```bash
DEVENV="$HOME/.config/devenv/devenv.toml"
```

Scan `[[services]]` entries for matching `working_dir` or `id` containing the package name. Extract:
- `port` — for health verification
- `dependencies` — for pre-deploy health check
- `env` section — for environment variable injection at start
- `health_check_url` — if present, use instead of default

### Step 7: Determine health path

| Port | Service | Health Path |
|------|---------|-------------|
| 8180 | maintenance-engine | `/api/health` |
| 8090 | synthex | `/api/health` |
| All others | * | `/health` |

If `health_check_url` exists in devenv.toml, use it verbatim.

### Step 8: Extract environment variables

Read `[services.env]` from devenv.toml. Critical for:
- SYNTHEX: `REST_PORT=8090`, `WS_PORT=8091`
- POVM: `POVM_DB_PATH=povm_data.db`, `POVM_PORT=8125`
- ORAC: `PORT=8133`, `PV2_ADDR=127.0.0.1:8132`, etc.

## 6-Stage Pipeline

### Stage 1: Quick Gate

```bash
echo "━━━ STAGE 1: Quick gate ━━━"
cd "$PROJECT_DIR"
CARGO_TARGET_DIR="$CARGO_TARGET" cargo check $PACKAGE_FLAG 2>&1 | tail -5
CARGO_TARGET_DIR="$CARGO_TARGET" cargo clippy $PACKAGE_FLAG -- -D warnings 2>&1 | tail -5
```

Skip with `--skip-gate`. Full pedantic with `--full-gate` (adds `-W clippy::pedantic` + tests).

### Stage 2: Release Build

```bash
echo "━━━ STAGE 2: Build release ━━━"
CARGO_TARGET_DIR="$CARGO_TARGET" cargo build --release $PACKAGE_FLAG $FEATURES_FLAG 2>&1 | tail -5
BIN_SIZE=$(du -h "$CARGO_TARGET/release/$BIN_NAME" 2>/dev/null | cut -f1)
echo "Binary: $BIN_NAME ($BIN_SIZE)"
```

### Stage 3: Stop Old Process (Safe)

```bash
echo "━━━ STAGE 3: Stop old process ━━━"
if [[ -n "$PORT" ]]; then
  # Port-based kill — avoids pkill exit 144 trap
  OLD_PID=$(ss -tlnp "sport = :$PORT" 2>/dev/null | /usr/bin/grep -oP 'pid=\K[0-9]+' | head -1)
  if [[ -n "$OLD_PID" ]]; then
    kill "$OLD_PID" 2>/dev/null
    echo "Killed PID $OLD_PID on port $PORT"
    sleep 2
  else
    echo "No process on port $PORT"
  fi
else
  # Fallback: pkill (NEVER chain with &&)
  pkill -f "$BIN_NAME" 2>/dev/null; true
  sleep 2
fi
```

### Stage 4: Deploy Binary

```bash
echo "━━━ STAGE 4: Deploy ━━━"
/usr/bin/cp -f "$CARGO_TARGET/release/$BIN_NAME" "$HOME/.local/bin/$BIN_NAME"
# Verify non-zero size
[[ -s "$HOME/.local/bin/$BIN_NAME" ]] && echo "OK: ~/.local/bin/$BIN_NAME" || echo "ERROR: Binary is empty!"
```

**TRAP:** Always `/usr/bin/cp -f` — never bare `cp` (aliased to `cp -i` in this environment).

### Stage 5: Start Service

```bash
echo "━━━ STAGE 5: Start ━━━"
# Inject environment variables from devenv.toml
ENV_VARS=""  # Claude constructs this from devenv.toml [services.env]
eval "$ENV_VARS nohup $HOME/.local/bin/$BIN_NAME > /tmp/$BIN_NAME.log 2>&1 &"
echo "PID: $!"
```

**TRAP:** Always `nohup ... > file 2>&1 &` — never bare stdout (SIGPIPE kills daemons).

### Stage 6: Health Verify

```bash
echo "━━━ STAGE 6: Verify ━━━"
sleep 3  # Most services need 3s; ORAC needs 5s
HTTP_CODE=$(curl -s -o /dev/null -w '%{http_code}' "localhost:$PORT$HEALTH_PATH" 2>/dev/null)
if [[ "$HTTP_CODE" == "200" ]]; then
  echo "HEALTHY: localhost:$PORT$HEALTH_PATH"
  curl -s "localhost:$PORT$HEALTH_PATH" 2>/dev/null | python3 -c "
import sys,json
d=json.load(sys.stdin)
for k in list(d.keys())[:6]:
    print(f'  {k}: {d[k]}')
" 2>/dev/null
else
  echo "UNHEALTHY: HTTP $HTTP_CODE"
  echo "Check log: tail -20 /tmp/$BIN_NAME.log"
fi
```

## Claude Intelligence Layer

Beyond the bash pipeline, Claude handles:

1. **Workspace detection** — reads parent Cargo.toml for `[workspace]` section
2. **Feature selection** — reads `[features]` to decide `--features full` vs `--features api` vs none
3. **Multiple bin targets** — if >1 `[[bin]]`, asks which to deploy (default: first daemon target)
4. **Environment variables** — reads devenv.toml `[services.env]`, injects at start
5. **Dependency pre-check** — before deploy, `curl` each dependency's health endpoint
6. **Post-deploy devenv sync** — runs `devenv restart {id}` to update PID tracking
7. **Service-specific health interpretation** — knows ORAC returns `ralph_gen`, ME returns `fitness`, etc.

## 8 Encoded Traps

| # | Trap | How /forge Handles It |
|---|------|----------------------|
| 1 | `cp` aliased to `cp -i` | Always `/usr/bin/cp -f` |
| 2 | `pkill` exit 144 kills `&&` chains | Uses `kill PID` via `ss`, never pkill in chains |
| 3 | SIGPIPE kills daemons on stdout | `nohup ... > file 2>&1 &` always |
| 4 | CARGO_TARGET_DIR conflicts | Per-service `/tmp/cargo-{name}` |
| 5 | Port already occupied by old process | Finds PID via `ss`, kills before deploy |
| 6 | Health path variance | Auto-detects `/api/health` vs `/health` |
| 7 | Feature flag requirements | Auto-detected from Cargo.toml `[features]` |
| 8 | Stale PID files from devenv | Uses `ss` (live port check), not PID files |

## CLI Options

```
/forge                          # Auto-detect everything from CWD
/forge --service synthex        # Explicit service (matches devenv.toml id)
/forge --port 8090              # Override port
/forge --features "api"         # Override features
/forge --skip-gate              # Skip quality gate (emergency)
/forge --full-gate              # Full pedantic gate + tests
/forge --dry-run                # Show resolution, don't execute
```

## Test Matrix (Minimum Validation Set)

| Service | Tests | Why |
|---------|-------|-----|
| orac-sidecar | features=full, 4 bins, standalone | Multi-bin + features |
| synthex | features=api, env vars, /api/health | Env injection + health variance |
| povm_engine | workspace member (nexus_forge) | Workspace handling |
| maintenance-engine | /api/health | Health path variance |

## Metrics

| Metric | Manual Deploy | /forge |
|--------|--------------|--------|
| Time per deploy | ~20 min | ~3 min |
| Commands to type | 5-7 | 1 |
| Trap exposure per deploy | 8 | 0 |
| Deploys per session | ~5 | ~5 |
| **Time saved per session** | | **~85 min** |

## Cross-References

- **Replaces:** `/deploy-orac` (ORAC-specific, 5 traps) with universal command (8 traps)
- **Integrates with:** `/gate` (full gate mode), `/sweep` (post-deploy verification)
- **Source analysis:** `fleet-abr-forge-design.md` (388 lines, 14K)
- **Agent analysis:** Background Agent 1 (95K tokens, independently proposed same design)


---

> Vault navigation: [[../../BOILERPLATE_INDEX|BOILERPLATE_INDEX]] · [[../../README|boilerplate modules README]] · [[../../../HOME|HOME]] · [[../../../MASTER_INDEX|MASTER_INDEX]]
> Reference-only clone — see [[../../BOILERPLATE_INDEX]] for upstream source + target-module mapping.
