---
title: Boilerplate Modules — Reference Clones
date: 2026-05-17 (S1001982)
kind: reference-archive
status: planning-only · REFERENCE-ONLY · NOT for direct import or compilation
authority: Luke @ node 0.A — "clone all identified boilerplate modules and save in [vault subfolder]"
---

# Boilerplate Modules — REFERENCE-ONLY Archive

> Back to: [[../HOME]] · [[../MASTER_INDEX]] · [[../workflow-engine-code-base]] · [[BOILERPLATE_INDEX]]

## ⚠️ READ FIRST

**These are REFERENCE COPIES, not project source.**

- Each file is a clone of an existing habitat-service source file, copied here for study + boilerplate-lift planning.
- Originals continue to live in their canonical service directories and evolve there.
- **These copies will drift** as upstream evolves. Do not treat them as authoritative.
- **No `cargo` operation should ever touch this directory.** No `Cargo.toml` exists here intentionally.
- When v1.3 patch + G7 Zen audit + G9 explicit signal clear and `workflow-trace` build begins, source files will be authored in the project's source tree (`workflow-trace/src/`), informed by these references but written fresh per the spec.

## What this directory IS

- Planning-pilot artefact (Luke directive)
- Snapshot of which boilerplate the Boilerplate Hunt (9-Explore-fleet, 63 candidates → 9 top-picks) identified as highest-value
- Self-contained study material — no need to leave the vault to read boilerplate
- Reference for the v1.3 spec patch and G5 spec interview

## What this directory is NOT

- ❌ Not the project source (workflow-trace will live in `~/claude-code-workspace/workflow-trace/src/` post-G2 rename)
- ❌ Not a build target — no Cargo.toml, no compilation expected
- ❌ Not authoritative — upstream sources evolve; these snapshots will drift
- ❌ Not in-scope of HOLD-v2 build-freeze — these are reference clones, not authored source
- ❌ Not a substrate write (vault is local file mirror per Zen scope clarification)

## Inventory

48 files across 10 categories. See [[BOILERPLATE_INDEX]] for per-file what-to-lift notes.

| Category | Files | Size | Boilerplate Hunt strength |
|---|---:|---:|---|
| `01-cli-scaffolding/` | 4 | 68KB | STRONG (Architect's two-binary split source) |
| `02-stcortex-consumer/` | 4 | 48KB | STRONG (W1 narrowed-scope; refuse-write) |
| `03-sqlite-multi-db/` | 6 | 228KB | STRONG (atuin + tracking-db + injection.db readers) |
| `04-pattern-detection/` | 3 | 120KB | MEDIUM (keystone gap — pairwise → N-step gap remains) |
| `05-decay-ttl-ltd/` | 4 | 136KB | STRONG (RALPH fitness-tensor + POVM lifecycle; "freq×fitness×recency" gap remains) |
| `06-daemon-scaffolding/` | 6 | 148KB | STRONG (synthex-v2 + nerve-center fusion) |
| `07-conductor-dispatch/` | 5 | 184KB | STRONG-pattern / BLOCKED-on-Wave-maturity |
| `08-nexus-lcm-rpc/` | 4 + symlink-note | 148KB | STRONG (NexusEvent + circuit breaker + JSON-RPC dispatch + ORAC m24 anti-pattern reference) |
| `09-trap-verify-escape-skills/` | 7 | 56KB | STRONG per-piece / GAP unified destructiveness schema |
| `10-foundation-direct-clones/` | 4 | 132KB | 95% reuse (per v0 module map) |
| **Total** | **48** | **~1.2MB** | |

## Per-category READMEs

Each subdirectory may grow its own README as study notes accumulate. See [[BOILERPLATE_INDEX]] for the cross-cutting per-file lift table.

## Compliance posture

This clone operation is **within the planning-pilot envelope**:

- ✅ Luke direct directive ("clone all identified boilerplate modules and save in [vault subfolder]")
- ✅ Local file copies only — no substrate write (stcortex/POVM untouched)
- ✅ No code authored — files are 1:1 mirrors of existing source
- ✅ No `cargo init`, no build target
- ✅ Vault subfolder existed pre-this-write (created earlier for this purpose)
- ✅ Within Zen scope clarification: comms + local files OK
- ⚠ Reference-only nature must be respected — these MUST NOT be edited and lifted-as-changed into the eventual project source

## Update history

| Date | Action |
|---|---|
| 2026-05-17 ~11:00 | v1 clone — 48 files from 10 categories (per Boilerplate Hunt 9 top-picks + v0 foundation-direct-clones) |

## See also

- [[../Boilerplate Hunt S1001982]] — the 9-fleet report that identified these candidates
- [[../Convergence Command x Command-3 S1001982]] — Command-3's Wave 2 recon added the foundation-direct-clone scope (~85% reuse for registry + Hebbian + schema)
- [[../Modules Synergy Clusters and Feature Verification S1001982]] — single-phase architecture this boilerplate supports
- [[../Genesis Prompt v0 S1001982]] — 5-voice prompt's 28-module map references many of these files at 95% reuse
- [[BOILERPLATE_INDEX]] — per-file lift map
