---
title: modules/ — per-module landing pages (reserved post-G9)
date: 2026-05-17
status: PLACEHOLDER (HOLD-v2 active; no per-module landings until G9 fires)
session: S1002127 Wave 4.B audit
hold_v2_compliant: true
---

# modules/ — per-module operational landing pages

> **Back to:** [`../README.md`](../README.md) · [`../CLAUDE.md`](../CLAUDE.md) · [`../ARCHITECTURE.md`](../ARCHITECTURE.md) · sister [`../ai_specs/MODULE_MATRIX.md`](../ai_specs/MODULE_MATRIX.md) · sister [`../ai_specs/modules/`](../ai_specs/modules/) (26 prescriptive spec files) · sister [`../layers/`](../layers/) (per-cluster landings)

## § 1 — Purpose & scope

This directory is reserved for **per-module operational landing pages** — one short README per module describing the module's runtime concerns (deploy/operate/observe, NOT design). It is the operational counterpart to [`../ai_specs/modules/cluster-{A-H}/m<N>_<name>.md`](../ai_specs/modules/) which carries the **prescriptive Rust god-tier spec** for each module.

Pre-G9, this directory is **empty by design** under HOLD-v2 — per-module landings cannot meaningfully describe runtime behaviour until source files exist and a binary deploys. The directory is scaffolded to make the post-G9 author-pass mechanical.

## § 2 — Why this directory is empty pre-G9

A per-module operational landing would document, per module:

- **Runtime metrics emitted** — Prometheus / tracing field names + cardinality
- **Health-probe endpoint** — what does m<N> contribute to a binary's `/health` response
- **Operational toggles** — env vars + config-file fields that change the module's behaviour
- **Failure-mode escalation** — which alerts fire when, who's paged, what's the runbook link
- **Capacity envelope** — per-module load characteristics (refresh of [`../ai_specs/BENCHMARK_SPEC.md`](../ai_specs/BENCHMARK_SPEC.md) measurements)
- **Last-deploy observation** — git ref + binary SHA + observed-rate-since-deploy

None of these are knowable pre-G9 (no source, no binary, no deploy, no observation). Authoring them now would be fabrication, violating CLAUDE.md § Integrity rules. Therefore: HOLD until G9.

## § 3 — Relationship to sibling directories

| Directory | Concern | Per-module count | When authored |
|---|---|---|---|
| [`../ai_specs/modules/cluster-{A-H}/`](../ai_specs/modules/) | **Prescriptive design specs** — what MUST be true at code level | 26 files (one per module) | Wave 1 (LIVE — already authored) |
| [`../ai_specs/layers/`](../ai_specs/layers/) | **Cluster-level architectural specs** — synthesis from vault + V7 plan | 8 files (one per cluster) | Wave 1 (LIVE) |
| [`../layers/cluster-{A-H}/`](../layers/) | **Per-cluster operational landings** — runtime view of each cluster | 8 sub-dirs, README each | Wave 4.B audit (LIVE — landings authored) |
| **`./modules/`** (this dir) | **Per-module operational landings** — runtime view of each module | 26 reserved | **Post-G9 (HOLD-v2)** |

The asymmetry (8 layer landings live, 26 module landings deferred) is deliberate: **cluster-level operational view can be authored pre-G9** because it reflects design intent (8 clusters × Watcher pre-position × cross-cluster contracts — all known from `ai_specs/`); **module-level operational view requires the module to actually exist and run** before it can be meaningfully described.

## § 4 — Authoring trigger (post-G9)

When Luke fires G9 and Cluster D Day 1 ships (`m8 → m9 → m10 → m11` per [`../CLAUDE.md`](../CLAUDE.md) § Cluster D ships Day 1), the per-module README for each shipped module SHOULD be authored as part of the module's commit. Suggested template (DO NOT instantiate pre-G9):

```markdown
---
module_id: m<N>
name: <module_name>
cluster: <A..H>
binary: <wf-crystallise | wf-dispatch>
status: ALIVE | DRAFT | DEPRECATED
last_deploy: <git-sha> · <date>
---

# m<N> — <name>

> **Spec:** [`../ai_specs/modules/cluster-<C>/m<N>_<name>.md`](../ai_specs/modules/cluster-<C>/m<N>_<name>.md)
> **Substrate dossier(s):** [`../ai_specs/substrates/<S-X>.md`](../ai_specs/substrates/) (if applicable)
> **Coupling decomposition:** [`../ai_specs/substrate-couplings/CC-<N>-decomposed.md`](../ai_specs/substrate-couplings/) (if applicable)

## Metrics emitted
| name | type | cardinality | trigger |

## Health contribution
- HTTP /health probe returns: `m<N>.status` ∈ {ready, degraded, refusing, dead}

## Operational toggles
| env var | default | effect |

## Failure-mode escalation
| signal | severity | runbook |

## Capacity envelope
- baseline: <op/s @ <p99 ms>
- saturation: <op/s @ <p99 ms>
- back-pressure: <signal>

## Last-deploy observation
- deploy SHA: <hash>
- observed rate since deploy: <op/s>
- known issues: <list>
```

Anchor the README in the module's `src/m<N>/` directory once Cargo source exists.

## § 5 — Compliance assertion

This README is markdown only; **0** `.rs` files, **0** `Cargo.toml`, **0** code under `modules/`. HOLD-v2 envelope preserved.

The deliberate emptiness of this directory is the correct state pre-G9 — adding per-module operational documentation pre-G9 would be theatre (fabricating runtime descriptions for non-existent modules), the anti-pattern AP-V7-13 has trained us to refuse (cf. POVM `learning_health` displays before CR-2 — false health signals).

---

> **Back to:** [`../README.md`](../README.md) · [`../ARCHITECTURE.md`](../ARCHITECTURE.md) · [`../GATE_STATE.md`](../GATE_STATE.md) · [`../ai_specs/MODULE_MATRIX.md`](../ai_specs/MODULE_MATRIX.md)

*Filed 2026-05-17 (S1002127 · Wave 4.B audit follow-up) · Command · placeholder declaration · HOLD-v2 compliant.*
