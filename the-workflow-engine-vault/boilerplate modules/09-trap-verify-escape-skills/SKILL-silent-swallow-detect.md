---
name: silent-swallow-detect
description: Hunt for silent-swallow anti-patterns across Habitat Rust services — `unwrap_or(true)` on external "healthy/consent/success" flags, `.ok()` discarding meaningful HTTP/DB ops, `let _ =` masking Result, `Ok(0)`/`Ok(())` success sentinels. Filters tests/benches/docs automatically. Use when reviewing a service for hidden error-masking, during a bug hunt, or before a release. Derived from S100 hunt that found 8 Class A + 5 Class B instances including the "POVM write-only" structural cascade (BUG-034 / S099 F-001 pattern).
---

# Silent-Swallow Detect

The silent-swallow anti-pattern is where a daemon receives a real error from an external operation (HTTP, DB, IPC, syscall) and turns it into a success sentinel — `unwrap_or(true)` on a `healthy` field, `.ok()` after `client.send()`, `let _ = db.write();`, `Ok(0)` returned from a function that couldn't actually complete. Downstream consumers read "success" and proceed on a lie.

This skill encodes the repeatable hunt + FP-preventer pass used in S100 which ID'd the structural cascade behind S099 F-001 and BUG-034 (POVM write-only).

## When to use

- Reviewing a service's error handling before a release
- Post-mortem after a silent-success incident
- Cross-service audit during habitat bug hunts
- Before merging a PR that touches bridge / HTTP / DB code

## Three-pattern scan

### Pattern A — Error-masking (HIGH)
```bash
rg -n 'unwrap_or\(true\)' --type rust \
  -g '!tests/*' -g '!*test*.rs' -g '!benches/*' -g '!examples/*' \
  <service-src-dir>

rg -n '\.ok\(\);' --type rust \
  -g '!tests/*' -g '!*test*.rs' -g '!benches/*' \
  <service-src-dir> \
  | /usr/bin/grep -iE 'http|post|request|send|resp|bridge|fetch|publish'

rg -nP '^\s*let _ = .+(send|write|execute|dispatch|publish|notify)' --type rust \
  -g '!tests/*' -g '!*test*.rs' <service-src-dir>

rg -n 'Ok\((?:0|\(\)|false)\)\s*;?\s*(?://|$)' --type rust \
  -g '!tests/*' <service-src-dir>
```

### Pattern B — Silent-defaulting (MEDIUM)
```bash
rg -n '\.unwrap_or\(0\)' --type rust \
  -g '!tests/*' -g '!*test*.rs' <service-src-dir> \
  | /usr/bin/grep -iE 'healthy|ok|success|count|rate|tick|latency'

rg -n 'unwrap_or_default\(\)' --type rust \
  -g '!tests/*' <service-src-dir>
```

### Pattern C — FP-preventer filters (ALWAYS apply)
Reject hits where:
- Surrounding block has `#[cfg(test)]`
- File lives under `tests/` or `benches/` or `examples/`
- Line is inside `///`, `//`, `//!` doc block showing a code example
- Surrounding function is `#[deprecated]` or documented as "TODO: remove"
- `.ok()` follows a cleanup call (`socket.close().ok();`) where error is genuinely meaningless
- `.unwrap_or(0)` on `hash`, `u32::try_from`, numeric overflow guards (correct default)

Verify each hit by **reading 20 lines around it** to establish caller context. A hit that survives the filters AND has a downstream consumer reading the lied-to value is a real finding.

## Output template per finding

```
### SS-<service>-<NNN> — <title>
- severity: critical | high | medium | low
- class: error-mask | silent-default | success-sentinel
- location: src/path/file.rs:LINE
- pattern: <unwrap_or(true) | .ok() | let _ = | Ok(0)>
- evidence: <3-line excerpt>
- downstream: <who reads the lied-to return>
- confidence: 0.0-1.0
- FP check: confirmed production
- proposed fix: <log + return error | fail-closed | breaker_failure + tracing::warn> + LOC
```

## Known exempt patterns (saves hunter cycles)

- `consent_snapshot.get(field).unwrap_or(true)` — legitimate when the field semantically means "unknown → permissive"; flag only if it applies to a security-class field (separate `strict` variant as done in SS-A-004 fix)
- `.ok()` on cleanup / best-effort notifications in shutdown paths
- `unwrap_or_default()` on scratch-local `String` builders or counter maps
- `unwrap_or(0)` on `hash >> N as u32` conversions
- CLI binaries in `src/bin/*.rs` that are ONE-SHOT (not daemons auto-starting via devenv)

## Verified catches (S100 provenance)

| ID | Service | Pattern | LOC fix |
|----|---------|---------|---------|
| V3-001 | V3 | `unwrap_or(true)` on SYNTHEX healthy | 1 |
| SS-A-001/002/003 | VMS | `.ok()` + `let _ =` triple on POVM bridge | ~20 |
| SS-A-004 | ORAC | `unwrap_or(true)` on consent for cascade | ~8 |
| SS-A-005/006 | PV2 | Discarded `listen()` / `set_reuse_address()` returns | ~14 |
| VMS-007 | VMS | `anam_active` defaults true on missing | ~3 |

## Anti-patterns — DO NOT flag

- `.ok().and_then(|x| ...)` — chained Option handling is not a swallow
- `error.ok()` where the error is the happy path (rare, context-dependent)
- `let _ = tx.send(msg);` inside a `tokio::select!` cleanup arm where receiver may be gone (intentional)

## When the scan is "clean"

A scan returning zero Class A hits doesn't mean the service is clean — it means the fleet's favorite form of the anti-pattern isn't there. Also check for:
- Silent timer drops (`interval.tick().await` without error prop)
- Blocking retries without back-off
- `match result { _ => { /* log */ } }` wildcard arms

Those are separate hunts.


---

> Vault navigation: [[../../BOILERPLATE_INDEX|BOILERPLATE_INDEX]] · [[../../README|boilerplate modules README]] · [[../../../HOME|HOME]] · [[../../../MASTER_INDEX|MASTER_INDEX]]
> Reference-only clone — see [[../../BOILERPLATE_INDEX]] for upstream source + target-module mapping.
