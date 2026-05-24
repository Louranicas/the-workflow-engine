> Back to: [[SYNTHEX-V2 Integration Master Schematic]] · [[HOME]] · [[MASTER_INDEX]] · `~/claude-code-workspace/the-workflow-engine/CLAUDE.local.md`
> Cross-vault: [[synthex-v2/MASTER_INDEX]] · [[synthex-v2/BRIDGE_TOPOLOGY_CONTINUITY]]
> Module owner: m42_stcortex_emit (Cluster H Substrate Feedback)
> Discipline: **P30 namespace prefix enforcement** (synthex-v2 vault § 5)

# Wiring 03 — stcortex Pathway Namespace Alignment

> Wire ALREADY exists (`POST :3000` via stcortex MCP / SDK / CLI). What's missing is **namespace discipline** — workflow-trace must use `workflow_trace_*` prefix so cross-loop Hebbian co-activation pairs become first-class and audit-traceable. ~80 LOC.

## Why this matters

The stcortex substrate enforces a P30 discipline through `synthex-v2/src/m6_action/m35d_povm_bridge.rs`:

- Every pathway write from synthex-v2 carries prefix `synthex_v2_<domain>_*`
- Writes without the prefix are **rejected** at the m35d bridge layer
- This makes the substrate's pathway graph auditable per-emitter

**Workflow-trace's equivalent:** every pathway written from m42_stcortex_emit must use prefix `workflow_trace_<module>_*` so that:
1. A `~/.local/bin/stcortex inspect` lookup can name the emitter
2. Hebbian co-activation pairs across the loop boundary (workflow-trace ↔ synthex-v2) are explicitly bidirectional
3. Decay/pruning policies can target one emitter without poisoning the other

## Namespace convention (workflow-trace canonical)

| Pattern | Used when | Example |
|---|---|---|
| `workflow_trace_<module>_<signal>` | engine-internal pathway (no synthex-v2 counterpart) | `workflow_trace_m20_prefixspan_pattern_3` |
| `workflow_trace_<module>_<signal>_<target>` | cross-loop boundary pair (workflow-trace → synthex-v2) | `workflow_trace_m16_heartbeat_synthex_v2_pid_adjustment` |
| `workflow_trace_<module>_to_<sx2_module>` | direct module-to-module Hebbian link | `workflow_trace_m40_nexus_emit_to_m13_ingest_router` |
| `workflow_trace_session_<sNNNN>_<artifact>` | session-scoped anchor (used by /save-session + plan-propagation) | `workflow_trace_session_s1004590_v021_hardening_assessment` (existing) |

## Bidirectional pair discipline (cross-loop Hebbian)

When workflow-trace emits an event that synthex-v2 acts on, write **TWO pathways** to seed Hebbian co-activation:

```
workflow_trace_m16_heartbeat               ──→  synthex_v2_pid_adjustment
workflow_trace_m16_heartbeat               ←──  synthex_v2_pid_adjustment   (feedback edge)
```

This is the equivalent of synthex-v2's existing pattern:

```
synthex_v2_classifier_confirmed_class_b    ──→  synthex_v2_pid_target_decrease
synthex_v2_classifier_confirmed_class_b    ←──  synthex_v2_pid_target_decrease
```

The reverse edge is what enables the Hebbian crawl to traverse the loop boundary as a single connected graph rather than two disconnected ones.

## Wire mechanics (already implemented in workflow-trace m42)

```rust
// src/m42_stcortex_emit/mod.rs  (existing v0.2.0)
pub async fn emit_pathway_pair(
    &self,
    pre: &str,
    post: &str,
    weight: f64,
) -> Result<(), RefusalToken> {
    self.client.write_pathway(self.namespace, pre, post, weight, &self.session_id).await?;
    self.client.write_pathway(self.namespace, post, pre, weight * 0.85, &self.session_id).await?;
    Ok(())
}
```

The weight asymmetry (0.95 forward / 0.85 reverse) matches the synthex-v2 convention from `HEBBIAN_DEPLOYMENT_PLAN_V3`.

## Bridge-validator (post-v0.2.2+ recommendation)

Add a workflow-trace-side validator mirroring synthex-v2's m35d enforcement:

```rust
// src/m42_stcortex_emit/namespace_validator.rs  (NEW, ~80 LOC)
pub fn validate_pathway_slugs(pre: &str, post: &str) -> Result<(), NamespaceError> {
    // workflow-trace canonical: must start with `workflow_trace_` OR `synthex_v2_` (for cross-loop pairs)
    let allowed_prefixes = ["workflow_trace_", "synthex_v2_"];
    if !allowed_prefixes.iter().any(|p| pre.starts_with(p)) {
        return Err(NamespaceError::ForeignPrefix { side: "pre", slug: pre.to_string() });
    }
    if !allowed_prefixes.iter().any(|p| post.starts_with(p)) {
        return Err(NamespaceError::ForeignPrefix { side: "post", slug: post.to_string() });
    }
    // S1001757 hyphen-munge bug: slugs must use underscores, not hyphens.
    if pre.contains('-') || post.contains('-') {
        return Err(NamespaceError::HyphenSlug { pre: pre.into(), post: post.into() });
    }
    Ok(())
}
```

## Catalogue of workflow-trace pathway slug families (canonical)

| Family | Slug template | Emitter | Pair with |
|---|---|---|---|
| **m16 heartbeat → PID adjust** | `workflow_trace_m16_heartbeat` | m16 transport | `synthex_v2_pid_adjustment` |
| **m20 PrefixSpan pattern detection** | `workflow_trace_m20_prefixspan_pattern_<id>` | m20 | `synthex_v2_classifier_class_<class>` (when m31 acts on it) |
| **m22 K-means cluster emit** | `workflow_trace_m22_kmeans_cluster_<id>` | m22 | `synthex_v2_classifier_class_<class>` (sibling-classification) |
| **m23 variant proposer** | `workflow_trace_m23_proposer_variant_<id>` | m23 | `workflow_trace_m30_bank_promoted_<id>` (engine-internal first) → then if promoted, `synthex_v2_action_emission_<kind>` |
| **m30 bank promotion** | `workflow_trace_m30_bank_promoted_<id>` | m30 | `workflow_trace_m32_dispatch_<id>` |
| **m32 dispatch verdict** | `workflow_trace_m32_dispatch_verdict_<id>` | m32 | `synthex_v2_m29_policy_routed_<kind>` |
| **m33 verifier outcomes** | `workflow_trace_m33_verifier_<kind>_<verdict>` | m33 | `workflow_trace_m30_bank_<verdict>_action_<id>` |
| **m40 NexusEvent emit** | `workflow_trace_m40_nexus_emit_<kind>` | m40 | `synthex_v2_m13_ingest_routed_<kind>` |
| **m41 LCM command result** | `workflow_trace_m41_lcm_cmd_<cmd>_exit_<code>` | m41 | `synthex_v2_m31_k_adj_<delta>` |
| **m42 self-anchor (session checkpoint)** | `workflow_trace_session_<sNNNN>_<artifact>` | m42 | (no synthex-v2 pair; session-scoped) |

## Cross-loop pair examples (post-wiring)

```bash
# Verify the bidi pairs landed (after m16 + m40 wiring goes live):
~/.local/bin/stcortex inspect workflow_trace_substrate_loop --limit 20

# Expected pairs:
#   workflow_trace_m16_heartbeat        →  synthex_v2_pid_adjustment        (weight ~0.95)
#   synthex_v2_pid_adjustment            →  workflow_trace_m16_heartbeat   (weight ~0.85)
#   workflow_trace_m40_nexus_emit_drift_canary_alert  →  synthex_v2_m13_ingest_routed_drift_canary  (~0.95)
#   ...
```

## Constraints + anti-patterns

| Constraint | Why | Mitigation |
|---|---|---|
| **S1001757 hyphen-munge bug** | stcortex substrate writes munge slugs containing hyphens | Validator above rejects hyphens; CLAUDE.md memory row 8 catalogues this trap |
| **P30 namespace prefix discipline** | Required by synthex-v2-side m35d | Validator enforces `workflow_trace_*` OR `synthex_v2_*` prefix |
| **AP09 cross-namespace bleed** | Don't write pathways into synthex-v2's `synthex_v2_*` namespace from workflow-trace | Only cross-loop **pairs** are permitted (e.g., `workflow_trace_X → synthex_v2_Y`); never write `synthex_v2_X → synthex_v2_Y` |
| **POVM CR-2 inflation** (until 2026-07-10 decommission) | Pre-CR-2 binary inflates learning_health | workflow-trace m42 ADR routes to stcortex ONLY (POVM-decoupled); this risk is structurally avoided |
| **stcortex 5274-col JSON parse error** (S1004590-observed) | write_memory fails on very long content | Split memories at ~4K char boundary; chain via `parent_ids` |
| **Refuse-write enforcement** | stcortex DB layer rejects writes without a fresh consumer | `/save-session` skill + SessionStart hook auto-register `cc-<session-id>` consumer per project; m42 must use this consumer |

## Verification

```bash
# 1. Schema validation (compile-time): unit tests in m42 validator
cargo test -p workflow-trace --lib m42_stcortex_emit::namespace_validator

# 2. Round-trip (post-write): inspect that pathways landed under correct prefix
~/.local/bin/stcortex inspect workflow_trace_completion_s1004115 --limit 5

# 3. P30 enforcement test: send a malformed slug to m42, assert Err(NamespaceError::ForeignPrefix)
cargo test -p workflow-trace --test m42_integration -- p30_rejection

# 4. Hyphen-munge regression test: send hyphenated slug, assert Err(NamespaceError::HyphenSlug)
cargo test -p workflow-trace --test m42_integration -- hyphen_rejection
```

## What this unlocks

- **Hebbian co-activation across the loop boundary** — engine-substrate feedback loop becomes a connected graph in the stcortex pathway store
- **Audit-traceable emitter attribution** — `stcortex query "WHERE pre_id LIKE 'workflow_trace_%'"` gives every engine-side pathway emit
- **Pruning + decay policies** can target one emitter without poisoning the other (synthex-v2 m11 fitness-weighted decay can decay engine-side and substrate-side pathways at different rates)
- **CC-5 Substrate Learning Loop becomes fully observable** — every event flowing engine→substrate has a co-activation pair that captures the round-trip
