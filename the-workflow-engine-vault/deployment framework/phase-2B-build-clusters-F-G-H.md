---
title: "Phase 2B — Build Clusters F + G + H (Days 12-21)"
date: 2026-05-17 (S1001982)
kind: deployment-framework-recipe
status: planning-only · HOLD-v2 active · pre-G9 gate
phase: 2B
days: 12-21
modules: m20 m21 m22 m23 m30 m31 m32 m33 m40 m41 m42
binary-owners: wf-crystallise (m20-m23 m40-m42) · wf-dispatch (m30-m33)
authority: Luke @ node 0.A (single-phase override 2026-05-17)
---

# Phase 2B — Build Clusters F + G + H (Days 12-21)

> Back to: [[../HOME]] · [[../GOD_TIER_CONSOLIDATION_S1001982]] · [[phase-2A-build-clusters-B-C-E]]

---

## Overview

Phase 2B spans 10 days and delivers 11 modules across three clusters. It is the densest build phase in the deployment plan. Cluster F contains the engine's structural keystone — the PrefixSpan-based N-step sequential pattern miner that no boilerplate in the vault covers beyond 30%. Cluster G closes the CC-4 contract (Proposal → Bank → Dispatch) and introduces the HUMAN-IN-THE-LOOP gate that governs every accepted workflow's entry to the bank. Cluster H closes CC-5, the slow substrate learning loop that feeds reinforcement signals back to m31's selection weights.

By the end of Day 21, three structural gaps will be owned by working code: N-step compositional sub-graph detection (Gap 1), the verification-gated dispatch pipeline (part of Gap 3), and the substrate learning close (CC-5). Cross-cluster contracts CC-4, CC-5, and CC-6 will be demonstrable via integration tests.

**Cluster assignments:**

| Cluster | Modules | Binary | Total LOC est. | Boilerplate lift |
|---|---|---|---|---|
| F — Iteration KEYSTONE | m20 m21 m22 m23 | wf-crystallise | ~850 | ~30% |
| G — Bank/Select/Dispatch/Verify | m30 m31 m32 m33 | wf-dispatch | ~950 | ~60-70% |
| H — Substrate Feedback | m40 m41 m42 | wf-crystallise | ~450 | ~85-95% |

**Three core algorithms introduced in this phase:**

1. PrefixSpan sequential pattern mining with gap-allowed matching (m20) — the keystone
2. Normalized Levenshtein distance on StepToken sequences for near-miss variant selection (m23)
3. Composite selection score with diversity algebra (m31) — `α × fitness + β × recency + γ × frequency + δ × diversity`

---

## KEYSTONE GAP Authorship Strategy

### Why this is genuinely new authorship

The Boilerplate Hunt confirms: Cat 04 (Pattern Detection) provides 30% reuse toward m20. The existing `m20_heat_source_hebbian.rs` covers pairwise (2-step) co-activation only. Generalising from pairwise to N-step sub-graph detection is not incremental — it requires a different data structure (projected database), a different algorithm (PrefixSpan vs. frequency map), and a different similarity metric (Levenshtein on token sequences vs. Jaccard on pairs). This is ~600-1,000 LOC of fresh authorship.

### Build order within the keystone (Days 12-15)

Build m20 in four internal passes rather than writing the full ~200 LOC at once:

**Pass 1 — Skeleton (Day 12):** Lay the module structure. Define `StepToken`, `SequentialPattern`, `PatternConfidence`, `SequentialPatternMiner` trait, `CascadeIterator` struct, `ProposalBuilder`, `ClusterFError` taxonomy. All methods return `todo!()`. Mechanical gate must be green on the skeleton before any algorithm code is written. This catches structural issues (trait bounds, lifetime annotations, Serialize/Deserialize derives) early.

**Pass 2 — PrefixSpan core (Day 13):** Implement `project()` and the recursive `prefix_span()` function. Write tests for the algorithm in isolation against synthetic sequence databases: linear sequence, diamond graph, gap-allowed matching at various `MAX_GAP_STEPS` values, patterns longer than `max_length` rejected. The PrefixSpan implementation is self-contained in `cascade_iterator/miner.rs` (~350 LOC) and does not touch m21/m22/m23.

**Pass 3 — Wilson CI gate (Day 14):** Implement `wilson_95()` with property tests (`ci_lower <= ci_upper` always; both in `[0, 1]`). Wire `ProposalBuilder::build()` to reject `n < 20` at construction. Add the CC-3 stabilization gate (m14 evidence variance check). At this point m20 can mine patterns and build proposals but has no variant selection.

**Pass 4 — Variant selection top-K (Day 15):** Implement `normalized_edit_distance()` on `StepToken` slices and `select_variants()`. Wire the full `iterate()` method. Run the complete m20 test suite (50+ tests). Push a `cargo check && cargo clippy -D warnings -W clippy::pedantic && cargo test --lib` before moving to m21.

### m23 Levenshtein as second keystone piece (~600-1,000 LOC total across m20+m23)

The edit distance function is implemented in m20 (where it gates variant selection) and then re-used by m23 for cross-type proposal deduplication. The implementation lives in `workflow_core::similarity` so both m20 and m23 import from the shared library without duplication. Write this once at the end of Day 15 when m20 is complete and m23's aggregation loop begins.

---

## Day-by-Day Breakdown

### Days 12-15: Cluster F KEYSTONE

**Day 12: m20 skeleton + PrefixSpan data structures**
- Atuin trajectory: `cargo new --lib workflow_core` (or extend existing structure) → `cargo check` → skeleton tests green
- Boilerplate lift: `m49_task_graph.rs` Kahn's topological sort and `ProjectedDatabase` adjacency list shape (50% reuse → ~100 LOC scaffold)
- Key deliverable: `StepToken`, `SequentialPatternMiner` trait, `ClusterFError` taxonomy compilable

**Day 13: PrefixSpan algorithm + gap-allowed projection**
- Atuin trajectory: `bacon` (on_success chaining) → `cargo test -- miner` → fix gap boundary conditions
- The projection loop is the hardest part: enforcing `MAX_GAP_STEPS` after each matched item without over-scanning. Write tests first (TDD for the projection logic).
- Key deliverable: `prefix_span()` recursion correct on 5+ synthetic sequence databases

**Day 14: Wilson CI + F2 gate + CC-3 stabilization**
- Atuin trajectory: `cargo test -- wilson` → `cargo test -- proposal_builder`
- `ProposalBuilder::build()` is the single enforcement point. Test that construction fails on `n < 20` and succeeds at `n = 20`. Test that CI bounds are non-negative and within `[0, 1]`.
- Key deliverable: F2 hard gate wired; no bypass path exists

**Day 15: Variant selection + m21 + m22 start**
- Atuin trajectory: `cargo test -- select_variants` → `cargo test -- m20` (full suite, 50+ pass) → begin m21 skeleton
- m21 (battern_iterator) lifts topological sort from m49 (50%) and dedup pattern from povm-v2_reinforcement (60%). New authorship: IQR-based step-duration outlier detection and wallclock-vs-cost Pareto frontier (~150 LOC).
- m22 (prompt_pattern_iterator) lifts rolling-mean smoothing from m39_fitness_tensor (70%) and dedup from povm-v2_reinforcement (60%). New authorship: feature vector construction and K-means clustering K=5 (~130 LOC).
- Key deliverable: m20 fully tested (50+); m21 and m22 skeletons compilable

### Days 15-17: Cluster G

**Day 15 (overlap): m30 workflow_bank**
- Atuin trajectory: `cargo new --bin wf-dispatch` → wire m30 migrations → `cargo test -- bank`
- Boilerplate lift: `conductor_state.rs` WAL constructor and migration framework (70%); `conductor_divergence.rs` Rule trait adapted as step-validator (50%)
- Key deliverable: `BankDb::accept`, `BankDb::eligible`, `BankDb::apply_decay_tick` all tested; SQLite WAL-mode confirmed

**Day 16: m31 selector + m33 workflow_verifier**
- m31 composite score formula wired. Bigram Jaccard similarity (`ngram_similarity()`) implements the Command-3 librarian shape. ORAC `m40_mutation_selector` diversity algebra: 10-gen cooldown + 50% mono-parameter rejection + round-robin cycling (~300 LOC total; ~70% boilerplate lift from m39_fitness_tensor + m10_pattern).
- m33 4-agent gate lifted from `SKILL-pre-deploy-hardening.md` (80%). Mechanical Wave 1 → parallel agent Wave 2 → boolean AND Wave 3 → audit-first write. Definition hash via FNV-1a 64-bit of `steps_json`. TTL: 7 days.
- Atuin trajectory: `cargo test -- selector` → `cargo test -- verifier` → confirm CC-6 contract (m33 must update `last_verified_at` before returning PASS)

**Day 17: m32 dispatcher**
- Boilerplate lift: `conductor_enforcement.rs` (80%) for `EnforcerAction` enum shape, audit-first write, cooldown tracking; `conductor_state.rs` (70%) for WAL, `kv_get/kv_set` dispatch-cooldown; `m32_tier_executor.rs` (60%) for step-outcome gating; `m24_povm_bridge.rs` (gold standard) for outbound HTTP wire shape — raw `host:port`, no `http://` prefix (BUG-033 fix).
- Wire the 5-check pre-dispatch sequence in order: Conductor probe → m33 TTL fresh → definition hash match → sunset guard → dispatch cooldown.
- Wire Conductor refuse-mode: when `CONDUCTOR_DISPATCH_ENABLED != 1`, every `dispatch()` call returns `DispatchError::ConductorDispatchDisabled`. Not a warn log — hard error with full message to stdout.
- Display-before-step banner: mandatory stdout, not optional log. Each step shows surface profile banner and trap annotations.
- Atuin trajectory: `cargo test -- dispatcher` → manually confirm refuse-mode message legibility

### Days 17-19: Cluster H

**Day 17 (overlap): m40 nexus_event_emitter skeleton**
- Boilerplate lift: `m22_synthex_bridge.rs` (90%) for dual-transport outbox-first + HTTP fire-and-forget; `m22_synthex_async.rs` (95%) for circuit breaker FSM.
- Local re-declaration of `NexusEvent` (Option A untyped JSON). Do NOT import synthex_v2. The struct is re-declared in `workflow_core::nexus_types` with identical wire shape.
- Atuin trajectory: `cargo test -- nexus_event` → `cargo test -- outbox_envelope` → confirm `posted=true` only after HTTP 2xx

**Day 18: m40 complete + m41 lcm_rpc_client**
- m40 outbox sweep task: retry `posted=false` envelopes up to `max_attempts=5` with exponential backoff ±25% jitter. Compaction removes entries older than 24h where `posted=true`.
- m41 builds on `m22_synthex_async.rs` (95%) circuit breaker and `lcm_supervisor.rs` client-side mirror. JSON-RPC 2.0 newline-framed, Unix domain socket, 30s read timeout. Only `lcm.loop.create` with `max_iters: 1` — no hypothetical `lcm.deploy`. Deploy-shape detection lives in m32; m41 only activates when `StepKind::Deploy` flag is set.
- Atuin trajectory: `cargo test -- lcm_rpc` → `cargo test -- m41` (50+)

**Day 19: m42 hebbian_feedback + CC-5 integration**
- m42 is the smallest module (~100 LOC). Boilerplate lift: `m24_povm_bridge.rs` (85% gold standard). Fitness delta constants wired as named constants, not magic numbers. POVM dual-path: `povm_overlap_active = true` default routes to POVM; `false` routes to stcortex via m13. Post-cutover stcortex unavailability logs at `ERROR`, returns `SubstrateUnavailable` — no silent POVM fallback (per CLAUDE.md stcortex policy).
- Wire CC-5 fan-out in m32: after Conductor accepts the request, m32 fires `WorkflowDispatchEvent` to m40/m41/m42 channel. Fan-out failure is non-fatal (log + continue).
- Atuin trajectory: `cargo test -- hebbian_feedback` → `cargo test -- m42` (50+) → confirm AP30 namespace prefix on all `retrieval_ids`

### Days 19-21: Cross-Cluster Integration Testing

**Day 20: CC-4 and CC-6 integration**
- CC-4 integration test: author a `Proposal` via m23, persist to `proposals` table with `status = AwaitingReview`, simulate `wf-crystallise propose accept <id>`, verify m30 `BankDb::accept` is called (m23 never calls it directly — the CLI subcommand calls m30). Confirm m23's `proposals` table row transitions to `Accepted` only after human command.
- CC-6 integration test: verify workflow via m33 → record `last_verified_at` → retrieve m30 row → m32 5-check gate passes → modify `steps_json` → re-run m32 5-check → `DispatchError::DefinitionDrifted` returned. Definition drift detection working.
- Atuin trajectory: `cargo test -- integration_cc4` → `cargo test -- integration_cc6`

**Day 21: CC-5 integration + gap demonstration**
- CC-5 integration test: dispatch a workflow via m32 (mock Conductor accepting) → assert `WorkflowDispatchEvent` emitted → assert m40 outbox contains event → assert m42 `ReinforcePayload` contains correct `fitness_delta` per outcome → assert AP30 namespace prefix present on all pathway IDs.
- Gap demonstration: run m20 PrefixSpan on a synthetic sequence database → assert top-K variants ordered by ascending edit distance from canonical → assert Wilson CI bounds non-negative → assert `deviation_shaped: false` on undeviated proposal (m23).
- Full workspace quality gate before phase close: `cargo check --workspace && cargo clippy --workspace -- -D warnings -W clippy::pedantic && cargo test --workspace --lib --release`.
- Atuin trajectory: `cargo test -- integration_cc5` → full workspace gate → record test counts

---

## Module Specifications

---

### m20 — cascade_iterator

**Cluster:** F — Iteration KEYSTONE  
**Binary:** wf-crystallise  
**LOC estimate:** ~200 (new authorship ~170 + lifted scaffold ~30)

#### Inputs

| Source | Table / metric | Via |
|---|---|---|
| m4 cascade_correlator | `workflow_trace.db` → `cascade_clusters` | Direct DB read |
| m6 context_cost_record | `workflow_trace.db` → `context_cost_records` | Direct DB read |
| m14 evidence_aggregator | `habitat_outcome_lift` metric (shared metric store) | CC-3 gate |

#### Outputs

| Target | What is written |
|---|---|
| `workflow_trace.db` → `cascade_proposals` | `Vec<Proposal>` with `source: IteratorKind::Cascade` |

#### Build steps

```bash
# 1. Scaffold — Day 12
cargo check  # skeleton must compile clean

# 2. Algorithm — Day 13
bacon  # continuous check while writing PrefixSpan

# 3. Tests — Day 14-15 (target 80+ per spec lock-in note)
cargo test --lib -- m20

# 4. Quality gate — end of Day 15
cargo clippy -- -D warnings -W clippy::pedantic
```

#### Boilerplate lifts

| Source file | LOC lifted | Reuse % | What is lifted |
|---|---|---|---|
| `04-pattern-detection/m49_task_graph.rs` | ~50 | 50% | Kahn's topological sort for DAG step ordering pre-PrefixSpan; `ProjectedDatabase` adjacency list shape |
| `04-pattern-detection/m20_heat_source_hebbian.rs` | ~30 | 30% | `CoActivationPair` API shape → generalized to N-step; `PatternConfidence` struct derivation |
| `04-pattern-detection/povm-v2_reinforcement.rs` | ~20 | 60% | Idempotency dedup cache (prevents same cluster-id double-counted across iteration cycles) |

#### ME v2 m1_foundation patterns referenced

- `forbid(unsafe_code)` and `deny(unwrap_used)` at crate root — applied to `cascade_iterator/mod.rs`
- `tracing::instrument` on `CascadeIterator::iterate()` — structured spans for observability
- `thiserror` error taxonomy — `ClusterFError` follows the ME v2 `error.rs` pattern exactly

#### Algorithm pseudo-code: PrefixSpan with gap-allowed matching

```
Input:
  sequence_db: [(cluster_id: u64, steps: Vec<StepToken>)]
  min_support: usize          -- default 20 (F2 hard gate)
  max_length:  usize          -- default 8
  max_gap:     usize          -- default 5 (MAX_GAP_STEPS)

fn prefix_span(db, prefix, depth, min_support, max_length, max_gap):
  if depth > max_length: return []

  item_counts: HashMap<StepToken, usize> = {}
  for (cluster_id, seq) in db:
    seen_in_this_seq: HashSet<StepToken> = {}
    for token in seq:
      if token not in seen_in_this_seq:
        item_counts[token] += 1
        seen_in_this_seq.insert(token)

  results = []
  for (item, support) in item_counts.items():
    if support < min_support: continue
    new_prefix = prefix + [item]
    projected_db = project(db, item, max_gap)
    results.push((new_prefix, support, cluster_ids(projected_db)))
    results.extend(prefix_span(projected_db, new_prefix, depth+1, ...))

  return results

fn project(db, item, max_gap):
  result = []
  for (cluster_id, seq) in db:
    for (i, token) in seq.enumerate():
      if token == item and i <= max_gap:  -- gap constraint: within window of last match
        result.push((cluster_id, seq[i+1..]))
        break  -- leftmost-first occurrence
  return result
```

The `max_gap` constraint is tracked per-sequence across recursion levels by passing the remaining suffix (suffixes implicitly encode how many steps remain after the last match).

#### Tests required

**Minimum: 80 tests** (elevated from 50 per spec note: lock-in-vulnerable because PrefixSpan correctness determines all downstream proposals).

| Category | Count | Examples |
|---|---|---|
| PrefixSpan correctness | 15 | Linear sequence; diamond DAG; gap-allowed [A,_,B,C]; deep chain depth=8; empty DB → InsufficientData |
| Gap constraint enforcement | 8 | Pattern [A,B] matched with gap=3 OK; gap=6 > MAX_GAP_STEPS blocked; gap=exactly-5 OK; gap=0 still matches adjacent |
| F2 gate | 10 | `n=19` → `InsufficientSamples`; `n=20` → OK; `n=0` → `InsufficientSamples`; overflow-safe |
| Wilson CI bounds | 10 | `successes=0, n=20`; `successes=20, n=20`; `successes=10, n=20`; non-negative lower; ≤ 1.0 upper |
| Variant selection | 8 | Top-3 by ascending edit distance; near-miss band [0.25, 0.60] enforced; patterns > 0.60 excluded |
| CC-3 stabilization gate | 6 | Variance above threshold → `StabilizationGateNotMet`; below threshold → proceeds |
| Opaque ID discipline | 5 | `StepToken(u32)` serializes as integer, not string; display resolution not in proposal layer |
| Error taxonomy | 5 | All `ClusterFError` variants constructable; `Display` non-empty; source chain correct |
| Property tests | 8 | `ci_lower <= ci_upper` always; `confidence in [0,1]`; edit distance symmetric; `select_variants` len ≤ max_variants |
| DB failure handling | 5 | Empty DB → `EmptyDatabase`; corrupted row → `DatabaseRead` |

#### Quality gate

```bash
cargo check
cargo clippy -- -D warnings
cargo clippy -- -D warnings -W clippy::pedantic
cargo test --lib -- m20  # must show 80+ passed, 0 failed
```

Additionally: Ember gate — the PROPOSE-never-act invariant must be demonstrable by test: assert that `CascadeIterator::iterate()` does not write to `cascade_proposals` in m30 bank or `workflow_trace.db` tables owned by other modules.

#### Cross-cluster touch-points

- **CC-3 (E → F):** m14 stabilization gate checked at the top of `iterate()`. If `habitat_outcome_lift` variance exceeds threshold, return `Err(StabilizationGateNotMet)`. This is not a silent skip — the error is returned and logged.
- **CC-5 (H → F feedback):** m20 does not directly consume CC-5 signals. The loop closes via m31: pathway weights updated by m42 shift m31's selection distribution, which over time changes which clusters produce the most dispatched workflows, which changes which sequences appear most frequently in m20's sequence database.

#### Failure modes + Watcher flag classes

| Mode | Watcher class | Symptom |
|---|---|---|
| Proposals never emitted | Class I (Hebbian silence) | `cascade_proposals` table empty after 7+ days of cascade cluster observation |
| All proposals rejected at F2 gate | Class C (confidence-gate refusal) | `n_samples` always < 20; observation volume insufficient |
| PrefixSpan produces zero patterns | Class E (ancestor-rhyme) | min_support set too high relative to observation volume; reduces to config diagnosis |
| CC-3 gate always firing | Class C | m14 metric never stabilizes; may indicate upstream substrate instability |

#### Verification gate — m20 complete when

1. `cargo test --lib -- m20` shows 80+ passed, 0 failed, 0 warnings at pedantic level
2. PrefixSpan mines at least 3 patterns from the 20-sequence synthetic test database embedded in the test suite
3. F2 gate rejects `n=19` and accepts `n=20` in the same test run
4. `cascade_proposals` table receives exactly the expected number of proposals from the integration fixture

#### Atuin trajectory

```bash
atuin scripts run wf-crystallise-init    # scaffold
bacon                                     # continuous check during algorithm authorship
cargo test --lib -- m20::tests            # targeted suite
cargo clippy -- -D warnings -W clippy::pedantic && cargo test --lib
```

---

### m21 — battern_iterator

**Cluster:** F  
**Binary:** wf-crystallise  
**LOC estimate:** ~200 (new authorship ~150 + lifted ~50)

#### Inputs

| Source | Table | Via |
|---|---|---|
| m5 battern_step_record | `workflow_trace.db` → `battern_step_records` | Direct DB read |
| m6 context_cost_record | `workflow_trace.db` → `context_cost_records` | Direct DB read |
| m14 evidence_aggregator | `habitat_outcome_lift` | CC-3 gate (same as m20) |

#### Outputs

| Target | What is written |
|---|---|
| `workflow_trace.db` → `battern_proposals` | `Vec<Proposal>` with `source: IteratorKind::Battern`, two variants: `ProposalKind::WallclockOptimized` and `ProposalKind::CostOptimized` |

#### Build steps

```bash
# Day 15 — skeleton after m20 complete
cargo test --lib -- m21  # target 50+ passed
cargo clippy -- -D warnings -W clippy::pedantic
```

#### Boilerplate lifts

| Source file | LOC lifted | Reuse % | What is lifted |
|---|---|---|---|
| `04-pattern-detection/m49_task_graph.rs` | ~50 | 50% | Topological sort for valid Battern ordering enumeration (respects Design→Dispatch→Gate→Collect→Synthesize→Compose DAG) |
| `04-pattern-detection/povm-v2_reinforcement.rs` | ~30 | 60% | Dedup pattern — prevents same `battern_id` counted twice across iteration cycles |

#### Algorithm notes: IQR-based step-duration outlier detection

```
For each Battern step position s in {0..5}:
  durations_s = [duration_ms for all runs where step_position == s]
  Q1 = percentile(durations_s, 25)
  Q3 = percentile(durations_s, 75)
  IQR = Q3 - Q1
  outliers_s = [d for d in durations_s if d > Q3 + 1.5 * IQR]

A step position with outlier_rate > 0.10 (>10% of runs are outliers)
generates a ProposalKind::WallclockOptimized proposal noting that
step s runs long in certain cluster contexts.
```

Two independent proposals per cycle: one for wallclock, one for token cost. The human reviewer sees both and chooses which variant to promote.

#### Tests required (50 minimum)

Topological enumeration of valid Battern orderings; ordering constraint enforcement (invalid reorderings rejected); IQR outlier detection (thresholds correct for known distributions); wallclock and cost proposals generated independently; dedup on battern-id; F2 gate enforced; Wilson CI for Battern success rates (success = step met wallclock budget); CC-3 gate blocks premature iteration; `IteratorKind::Battern` in every emitted proposal.

#### Verification gate — m21 complete when

`cargo test --lib -- m21` shows 50+ passed; both `ProposalKind::WallclockOptimized` and `ProposalKind::CostOptimized` proposals generated from the integration fixture; topological ordering enumeration correct for standard 6-step DAG.

---

### m22 — prompt_pattern_iterator

**Cluster:** F  
**Binary:** wf-crystallise  
**LOC estimate:** ~200 (new authorship ~130 + lifted ~70)

#### Inputs

| Source | Table | Via |
|---|---|---|
| m6 context_cost_record | `workflow_trace.db` → `context_cost_records` | Direct DB read |
| m7 correlation_hub | `workflow_trace.db` → `workflow_runs` | Direct DB read |
| m14 evidence_aggregator | `habitat_outcome_lift` | CC-3 gate |

#### Outputs

| Target | What is written |
|---|---|
| `workflow_trace.db` → `prompt_proposals` | `Vec<Proposal>` with `source: IteratorKind::PromptPattern` |

#### Boilerplate lifts

| Source file | LOC lifted | Reuse % | What is lifted |
|---|---|---|---|
| `04-pattern-detection/povm-v2_reinforcement.rs` | ~30 | 60% | Dedup on run IDs |
| From m39_fitness_tensor (via GOD_TIER synthesis) | ~40 | 70% | 6-sample rolling-mean smoothing for volatile `cost_tier` feature; `VOLATILE_DIMENSIONS` mask pattern |

#### Algorithm note: K-means NOT PrefixSpan

Prompt patterns are feature vectors, not ordered sequences. K-means on `[token_cost_tier, tool_call_count, distinct_tool_types, has_verification_step]` with K=5. PrefixSpan is not applicable here because the structural invariant PrefixSpan exploits — ordered occurrence of items across sequences — is absent in feature vectors.

Wilson CI success definition: a run is a success if `workflow_runs.fitness_delta > 0.0`. Clusters with fewer than 20 runs are dropped (F2 gate at cluster level, not run level).

#### Tests required (50 minimum)

Feature vector construction; K-means convergence; cluster membership stability (same inputs → same clusters); small-cluster filtering (< 20 runs → dropped); Wilson CI for success rates; volatile feature smoothing; dedup on run IDs; CC-3 gate; `IteratorKind::PromptPattern` in every emitted proposal; cost/outcome ratio distinguishes clusters.

#### Verification gate — m22 complete when

`cargo test --lib -- m22` shows 50+ passed; K-means converges in < 100 iterations on test fixtures; all clusters with < 20 members are absent from output.

---

### m23 — workflow_proposer

**Cluster:** F — single output surface  
**Binary:** wf-crystallise  
**LOC estimate:** ~250 (new authorship ~180 + lifted ~70)

#### Inputs

| Source | Table | Via |
|---|---|---|
| m20 cascade_iterator | `workflow_trace.db` → `cascade_proposals` | DB read |
| m21 battern_iterator | `workflow_trace.db` → `battern_proposals` | DB read |
| m22 prompt_pattern_iterator | `workflow_trace.db` → `prompt_proposals` | DB read |
| m32 dispatcher | `workflow_trace.db` → `deviation_events` | DB read (CC feedback loop) |

#### Outputs

| Target | What is written |
|---|---|
| `workflow_trace.db` → `proposals` | Final aggregated proposals; `status = 'awaiting_review'` |

**NEVER writes to:** m30 bank, stcortex, atuin, injection.db. m23 is write-blind beyond `workflow_trace.db`.

#### Build steps

```bash
# Day 15-16 — after m20/m21/m22 skeletons exist
cargo test --lib -- m23  # target 50+
cargo clippy -- -D warnings -W clippy::pedantic
```

#### Boilerplate lifts

| Source file | LOC lifted | Reuse % | What is lifted |
|---|---|---|---|
| `09-trap-verify-escape-skills/SKILL-pre-deploy-hardening.md` | ~50 | 80% | Deviation event schema: `Verdict { agent, APPROVE/REJECT, evidence }` pattern adapted as `DeviationEvent { bypassed_step, rationale, cluster_id, ts_ms }` |
| From m39_fitness_tensor (via GOD_TIER synthesis) | ~20 | 70% | Per-dimension confidence weighting for `ConfidenceBand` computation |

#### Algorithm pseudo-code: normalized Levenshtein for near-miss variant selection

```
fn normalized_edit_distance(a: &[StepToken], b: &[StepToken]) -> f64:
  max_len = max(a.len(), b.len())
  if max_len == 0: return 0.0
  dist = levenshtein_dp(a, b)   -- standard O(|a| * |b|) DP over u32 comparisons
  return dist as f64 / max_len as f64

fn select_variants(canonical, candidates, max_variants=3, lo=0.25, hi=0.60):
  near_misses = [(normalized_edit_distance(canonical.steps, p.steps), p)
                 for p in candidates
                 if lo <= normalized_edit_distance(canonical.steps, p.steps) < hi]
  near_misses.sort_by(|(d_a, _), (d_b, _)| d_a.partial_cmp(d_b))
  return near_misses[:max_variants].map(|(_, p)| p)
  -- Deterministic: always selects the N structurally nearest alternatives
  -- Preserves solution-space topology gradient
```

#### Cluster F deviation-as-evidence loop

This is the feedback mechanism that closes Cluster F internally:

1. m32 presents a step to the operator before Conductor dispatch (display-before-step gate, P0 #11)
2. Operator bypasses a step with an explicit rationale text
3. m32 writes a `deviation_events` row to `workflow_trace.db`
4. m23 reads `deviation_events` and joins them to proposals whose canonical pattern contains the bypassed step
5. If the same step is bypassed across multiple runs with rationale-similarity ≥ 0.8 cosine, m23 creates a "step X removed" variant with relaxed support threshold (n ≥ 5, not 20)
6. The proposal's `deviation_shaped: true` flag is set; `deviation_evidence` array populated
7. Human reviewer sees the deviation-shaped variant alongside standard near-misses
8. If the reviewer accepts the deviation-shaped variant → promoted to m30 → dispatched → m40/m41/m42 learn → m31 selection shifts

The loop is additive: deviation evidence adds a variant but never removes the canonical or its standard near-misses. The human always sees the full picture.

#### Tests required (50 minimum)

Aggregation merges from all three iterator tables; cross-type deduplication (same cluster referenced by cascade + prompt-pattern → one canonical); variant selection (top-3 near-misses, ascending edit distance ordering); deviation event join (step matching, consistent-rationale detection); deviation-shaped flag set only with sufficient bypass evidence; relaxed support threshold (n ≥ 5) for deviation-shaped variants; empty iterator input → `ProposerError::NoIteratorOutput`; proposals written with `status = AwaitingReview`; m23 never writes to m30 (bank table unchanged in test); `ConfidenceBand` computation; property test: variants always ordered by ascending edit distance.

#### Quality gate for Cluster F (all four modules)

```bash
cargo check
cargo clippy -- -D warnings
cargo clippy -- -D warnings -W clippy::pedantic
cargo test --lib -- m20  # 80+ passed
cargo test --lib -- m21  # 50+ passed
cargo test --lib -- m22  # 50+ passed
cargo test --lib -- m23  # 50+ passed
```

Plus: Ember gate — the PROPOSE-never-act invariant checked by a test that asserts m20-m23 produce zero writes to `workflow_bank.db` or any stcortex namespace during an iteration cycle.

#### Wilson CI + diversity gate for Cluster F quality assessment

Before Cluster F is declared Phase-complete:
- Wilson CI lower bound > 0.0 on the synthesis fixture (20 cascade clusters, 20 runs each)
- Diversity gate: at least 2 distinct lineage IDs represented in the `cascade_proposals` output (monoculture check)
- `cargo test --lib -- cluster_f_integration` passes (CC-3 and the deviation-as-evidence loop demonstrated)

#### Verification gate — m23 complete when

`cargo test --lib -- m23` shows 50+ passed; deviation-shaped variant appears when 3+ bypass events share rationale similarity > 0.8; m30 bank table is unmodified after a full `aggregate_and_write()` call in tests.

#### Atuin trajectory (Cluster F complete)

```bash
cargo test --workspace --lib --release -- cluster_f  # all 230+ Cluster F tests
cargo clippy --workspace -- -D warnings -W clippy::pedantic
atuin scripts run wf-trace-cluster-f-gate
```

---

### m30 — workflow_bank

**Cluster:** G — Bank/Select/Dispatch/Verify  
**Binary:** wf-dispatch  
**LOC estimate:** ~200

#### Inputs

| Source | Via |
|---|---|
| m23 `proposals` table (human-mediated) | CLI `wf-crystallise propose accept <id>` triggers the write path |
| m11 engine_sunset_lifecycle decay tick | `BankDb::apply_decay_tick()` called by m11's sweep |

#### Outputs

| Target | Schema |
|---|---|
| `workflow_bank.db` → `accepted_workflows` | `AcceptedWorkflow` struct; WAL-mode SQLite |

The human-in-the-loop gate is architectural: m23 writes `proposals` with `status = AwaitingReview`. The CLI subcommand `wf-crystallise propose accept <id>` transitions status and calls `BankDb::accept()`. m23 never calls `BankDb::accept()` directly — the bank is opaque to the proposer.

#### Boilerplate lifts

| Source file | LOC lifted | Reuse % | What is lifted |
|---|---|---|---|
| `07-conductor-dispatch/conductor_state.rs` | ~80 | 70% | `StateDb` WAL constructor; migration framework; `kv_get/kv_set` pattern |
| `07-conductor-dispatch/conductor_divergence.rs` | ~40 | 50% | `Rule` trait adapted as `AcceptanceValidator` for step-type registry checks |
| `09-trap-verify-escape-skills/feedback_preserve_list_discipline.md` | schema rationale | — | `EscapeSurfaceProfile` ordinal enum motivation (S102 openclaw incident) |

#### EscapeSurfaceProfile enum

```rust
// Ordinal: ReadOnly < HostWrite < Network < SandboxEscape < Destructive
// A workflow touching BOTH HostWrite and Network steps carries Network (higher ordinal)
// Rationale: S102 preserve-list-discipline scar tissue
pub enum EscapeSurfaceProfile {
    ReadOnly,
    HostWrite,
    Network,
    SandboxEscape,
    Destructive,
}
```

The `step_banner()` method returns the mandatory display string that m32 prints before each step. Not optional. Not a log line. Mandatory stdout.

#### Sunset semantics from m11

m30 exposes `BankDb::apply_decay_tick(now_ms)` which m11's engine-wide sweep calls. m30 never initiates decay itself. The formula applied per tick: `ralph_decay_weight *= 0.98`. Workflows past `sunset_at` are marked ineligible (not deleted — the audit trail is preserved).

#### Tests required (55 minimum)

Schema migrations (idempotent; 3 migrations applied in order; `sunset_at > accepted_at` CHECK enforced; escape_surface roundtrip); accept/get (accept inserts row; get returns None for sunset; None for missing); eligible query (excludes sunset; ordered by weight; respects limit; empty bank returns empty vec); dispatch recording (increments count; updates timestamp); verification recording (sets `last_verified_at`; None before first verify; TTL freshness correctly computed); decay tick (weight decrements by 0.98; never below 0.0; sunset marking; count returned); escape surface (all 5 variants serde roundtrip; `step_banner()` non-empty for all variants).

#### Verification gate — m30 complete when

`cargo test --lib -- workflow_bank` shows 55+ passed; `apply_decay_tick` produces `ralph_decay_weight = 0.98^n` after n ticks (floating-point tolerance ≤ 1e-9); `eligible()` excludes workflows where `sunset_at <= now_ms`.

---

### m31 — selector

**Cluster:** G  
**Binary:** wf-dispatch  
**LOC estimate:** ~300

#### Inputs

| Source | Via |
|---|---|
| m30 `workflow_bank.db` | `BankDb::eligible(now_ms, limit=200)` |
| stcortex `workflow_trace_*` pathway weights | Read at start of each selection cycle (CC-5 feedback) |
| `SelectionContext` (in-memory) | Rolling window of 10 recent dispatch sequences and lineages |

#### Outputs

| Target | Type |
|---|---|
| m33 `workflow_verifier` | `Vec<RankedWorkflow>` (top-N, default 5) for verification |

#### Composite score formula

```
score(w) = α × fitness_weight(w)
         + β × recency_score(w)
         + γ × frequency_penalty(w)
         + δ × diversity_bonus(w, recently_selected)

where:
  α = 0.40 (fitness weight — primary signal)
  β = 0.25 (recency)
  γ = 0.20 (frequency penalty — suppresses over-dispatched)
  δ = 0.15 (diversity bonus — monoculture mitigation)

  fitness_weight(w)      = w.ralph_decay_weight  [0.0, 1.0]
  recency_score(w)       = exp(-(now_ms - w.last_verified_at_or_accepted) / RECENCY_HALF_LIFE_MS)
  frequency_penalty(w)   = 1.0 / (1.0 + w.dispatch_count × FREQ_SCALE)
  diversity_bonus(w, S)  = 1.0 - max(ngram_similarity(w.steps, s.steps) for s in S)
```

The stcortex pathway weight feedback from CC-5 modulates `α`: a workflow in `workflow_trace_*` namespace with strong SYNTHEX v2 reinforcement gets an amplified `fitness_weight` contribution.

#### Diversity algebra (ORAC m40_mutation_selector adapted)

Three diversity enforcement mechanisms, in order of application:

**10-generation cooldown:** A workflow dispatched within the last 10 dispatch events is suppressed (diversity_bonus = 0.0, composite score floored at `MIN_SELECTION_SCORE`). Prevents immediate re-selection.

**50% mono-parameter rejection gate:** If more than 50% of the top-10 candidates share the same `lineage` id, the tail candidates from that lineage are dropped and replaced by the next-highest-scoring candidates from different lineages. Prevents a single well-performing cascade cluster from monopolising all selections. This is BUG-035 (ORAC mono-parameter mutation trap) carried forward.

**Round-robin cycling across lineages:** When multiple lineages score within `LINEAGE_SCORE_TOLERANCE = 0.05` of each other, the selector cycles through lineages in first-acceptance order rather than always picking the highest scorer.

#### Bigram Jaccard similarity (Command-3 librarian shape)

```
fn ngram_similarity(a: &[StepId], b: &[StepId], n=2) -> f64:
  a_grams = Set(a.windows(n))
  b_grams = Set(b.windows(n))
  if a_grams.empty() and b_grams.empty(): return 1.0
  if a_grams.empty() or  b_grams.empty(): return 0.0
  return |a_grams ∩ b_grams| / |a_grams ∪ b_grams|  -- Jaccard
```

Step IDs are opaque tokens (F11 compliance). Bigram windows over integer sequences are fast to compute and produce stable Jaccard similarity for sequences of 4-8 steps (the typical workflow step count).

#### Tests required (60 minimum)

N-gram similarity (identical → 1.0; disjoint → 0.0; partial overlap; empty sequences; unigram vs bigram); composite score (fitness applied; recency decays; frequency penalty suppresses over-dispatched; diversity bonus computed correctly); mono-parameter gate (>50% same lineage triggers replacement; ≤50% passes; all-same-lineage handled); 10-gen cooldown (recently dispatched suppressed; after 10 events re-eligible); round-robin cycling (similar-score lineages cycle, not always top scorer); empty bank (empty `eligible()` → empty `RankedWorkflow` vec); feedback loop (record_selection updates context; context window bounded at 10).

#### Verification gate — m31 complete when

`cargo test --lib -- selector` shows 60+ passed; mono-parameter gate fires when synthetic bank has 7/10 candidates from the same lineage; 10-gen cooldown correctly suppresses the most recently dispatched workflow in a synthetic 10-dispatch sequence.

---

### m33 — workflow_verifier

**Cluster:** G  
**Binary:** wf-dispatch  
**LOC estimate:** ~200

#### Inputs

| Source | Via |
|---|---|
| m30 `workflow_bank.db` | Reads `AcceptedWorkflow` by id |
| m31 `RankedWorkflow` | Ordered candidates provided by m31 (CC-6) |

#### Outputs

| Target | What is written |
|---|---|
| m30 `workflow_bank.db` | `BankDb::record_verification(id, now_ms)` on PASS or DEGRADED |
| `VerificationResult` struct | In-memory; returned to m32 |

The verification result carries `definition_hash: FNV-1a 64-bit hex of steps_json`. m32 re-computes this hash at dispatch time to detect definition drift.

#### Boilerplate lifts

| Source file | LOC lifted | Reuse % | What is lifted |
|---|---|---|---|
| `09-trap-verify-escape-skills/SKILL-pre-deploy-hardening.md` | ~90 | 80% | 4-agent parallel gate: `Verdict { agent, APPROVE/REJECT, evidence }` structure; Wave 1 mechanical → Wave 2 parallel agents → Wave 3 boolean AND |
| `07-conductor-dispatch/conductor_enforcement.rs` | ~40 | 80% | `EnforcerAction` enum adapted as `VerificationVerdict`; audit-first write guarantee; cooldown logic adapted as 7-day TTL |

#### 4-agent gate structure (from SKILL-pre-deploy-hardening)

```
Wave 1 (mechanical — Rust, no agents):
  - All StepDef references resolve in workflow-core type registry
  - EscapeSurfaceProfile consistent with step definitions
  - sunset_at > now_ms
  → If any fails: return VerificationVerdict::Fail immediately

Wave 2 (4 agents, parallel — in single-phase these are sub-agent Task dispatches):
  - Security auditor:     secrets, injection, info-disclosure, fail-closed DoS,
                          error-type leaks, habitat traps (URL prefix, escape surface)
  - Performance engineer: allocations in hot paths, blocking I/O in async (AP29),
                          unbounded loops, lock acquisition order
  - Silent-failure hunter: P1-P7 patterns from bridge-silent-failure-hunt
  - Zen (capstone):       Rust idiom, architecture coherence, test coverage, smell test

Wave 3 (boolean AND — Rust):
  - All 4 APPROVE or APPROVE-WITH-NITS → VerificationVerdict::Pass
  - All APPROVE-WITH-NITS, no REJECT → VerificationVerdict::Degraded
  - Any REJECT → VerificationVerdict::Fail

Wave 4 (audit-first write):
  - On PASS or DEGRADED: BankDb::record_verification(id, now_ms) BEFORE returning
  - On FAIL: no verification timestamp written; workflow remains unverified
```

The build-time implementation choice for Wave 2 agent dispatch (Task tool, sub-agent, or structured prompt chain) is left open for Zen G7 audit. The structural contract (4 agents, parallel, boolean AND) is not negotiable.

#### Tests required (55 minimum)

Verdict computation (PASS when all 4 approve; FAIL on any reject; DEGRADED on all nits); TTL gating (fresh within 7 days; stale after TTL; None `last_verified_at` → stale; exact boundary); mechanical gate (unknown step type fails; escape surface inconsistency fails; past sunset fails; all valid passes); definition hash (same steps → same hash; mutated steps → different hash; FNV-1a deterministic); agent verdicts (all 4 agents represented; `approved + nits_only` roundtrip; evidence list preserved); audit-first guarantee (record_verification called on PASS; not called on FAIL; DEGRADED records).

#### Verification gate — m33 complete when

`cargo test --lib -- workflow_verifier` shows 55+ passed; `definition_hash` changes when one step's `kind` field is mutated; `BankDb::record_verification` is called in the PASS test and NOT called in the FAIL test (confirmed by test harness asserting the bank row's `last_verified_at` value).

---

### m32 — dispatcher

**Cluster:** G  
**Binary:** wf-dispatch  
**LOC estimate:** ~250

#### Inputs

| Source | Via |
|---|---|
| m33 `VerificationResult` | Passed by m32 caller after m31 selection + m33 verification (CC-6) |
| m30 `AcceptedWorkflow` | Retrieved by workflow_id |
| HABITAT-CONDUCTOR health probe | `GET /health` on `127.0.0.1:8141` |

#### Outputs

| Target | What is written |
|---|---|
| `dispatch_log.db` | `DispatchAuditRow` — audit-first, written BEFORE Conductor request |
| m30 `workflow_bank.db` | `BankDb::record_dispatch(id, now_ms)` after Conductor accepts |
| m40/m41/m42 event fan | `WorkflowDispatchEvent` — fire-and-forget, non-blocking |

#### NEVER executes directly

m32 never calls `std::process::Command` or `tokio::process::Command` on a workflow step. P0 #3 violation if any such call appears in m32's code path. All execution goes through HABITAT-CONDUCTOR's dispatch endpoint.

#### Conductor maturity check — refuse-mode startup

```
if CONDUCTOR_DISPATCH_ENABLED != "1":
  tracing::error!("CONDUCTOR_DISPATCH_ENABLED not set — dispatcher initialised in REFUSE mode")
  Dispatcher::refuse_mode()    // every dispatch() returns ConductorDispatchDisabled
  // Does NOT panic; does NOT exit; does NOT silently no-op
  // Prints hard error to stdout on every dispatch() call

else:
  probe_conductor("127.0.0.1:8141")   // live health check
  // If probe fails → Err(ConductorNotLive) → binary exits with non-zero
```

This is not a configuration smell — it is the production behaviour until Luke flips the env var after Conductor Wave 3 bring-up.

#### Conductor wire shape

```rust
// ConductorDispatchRequest (outbound, NDJSON to Conductor endpoint)
// Adapted from conductor_api.rs DeployRequest + m24_povm_bridge gold standard
pub struct ConductorDispatchRequest {
    pub dispatch_id: String,           // UUID v7; idempotency key
    pub workflow_id: WorkflowId,
    pub steps: Vec<ResolvedStep>,
    pub escape_surface: EscapeSurfaceProfile,
    pub dispatched_at: i64,
    pub operator: String,              // always "wf-dispatch/human"
    pub verification_receipt: VerificationReceipt,
    pub dry_run: bool,
}
```

Wire: raw HTTP POST to `127.0.0.1:8141/dispatch` (NO http:// prefix — BUG-033 applied from m24_povm_bridge gold standard). Newline-delimited JSON body.

#### Pre-dispatch gate sequence (5 checks, in order)

```
1. probe_conductor(addr)                    → Err(ConductorNotLive) if fails
2. is_verification_fresh(wf, now_ms)        → Err(VerificationStale) if fails  [CC-6]
3. definition_hash_matches(wf, stored)      → Err(DefinitionDrifted) if fails
4. wf.sunset_at > now_ms                   → Err(Sunset) if fails
5. dispatch_cooldown_ok(wf.id, now_ms)      → Err(CooldownActive) if fails
```

Any failure in steps 1-5 returns the error immediately. No partial execution. No silent degradation.

#### Display-before-step banner (mandatory stdout)

```
═══════════════════════════════════════════════════════════════
  wf-dispatch: about to dispatch <N> steps via HABITAT-CONDUCTOR
  workflow:  <workflow_id>
  verified:  <ISO8601 timestamp> (<N days> ago; TTL ok)
═══════════════════════════════════════════════════════════════

  Step 1/<N> · kind=<step_kind> · surface=<ESCAPE_SURFACE>
  <EscapeSurfaceProfile::step_banner()>
  Traps: <trap list or "none">

  [... per step ...]

═══════════════════════════════════════════════════════════════
  Handing off to Conductor. wf-dispatch does NOT execute steps.
═══════════════════════════════════════════════════════════════
```

Not a log line. Mandatory stdout before the Conductor request is sent. If the operator reads nothing else, they see this.

#### Boilerplate lifts

| Source file | LOC lifted | Reuse % | What is lifted |
|---|---|---|---|
| `07-conductor-dispatch/conductor_enforcement.rs` | ~90 | 80% | `EnforcerAction` enum shape; `COOLDOWN_SECS` adapted as `DISPATCH_COOLDOWN_MS`; audit-first write |
| `07-conductor-dispatch/conductor_state.rs` | ~50 | 70% | `StateDb` WAL; migration; `kv_get/kv_set` for dispatch-cooldown tracking |
| `07-conductor-dispatch/m32_tier_executor.rs` | ~40 | 60% | `Tier` enum; step-outcome gating; checkpoint/resume |
| `08-nexus-lcm-rpc/m24_povm_bridge.rs` | ~20 | gold standard | raw `host:port` HTTP shape; BUG-033 fix; BUG-034 endpoint discipline |

#### Tests required (60 minimum)

Conductor probe (live → Ok; health non-ok → ConductorNotLive; network error → ConductorNotLive); refuse mode (dispatch returns ConductorDispatchDisabled; every call, not just first); pre-dispatch gate order (each of 5 gates tested individually + combined; correct error variant returned); audit-first guarantee (audit row written before Conductor call; failed write aborts dispatch); display-before-step (banner printed for each surface level; traps listed; step count correct); fan-out (WorkflowDispatchEvent emitted; fan-out failure non-fatal); error taxonomy (all 8 `DispatchError` variants constructable; Display impl non-empty).

#### Verification gate — m32 complete when

`cargo test --lib -- dispatcher` shows 60+ passed; refuse-mode message appears on stdout in the `ConductorDispatchDisabled` test; audit row exists in `dispatch_log.db` before the mock Conductor call is made (confirmed via test that injects a Conductor stub that panics — the audit row must already exist when the panic fires).

#### Atuin trajectory (Cluster G complete)

```bash
cargo test --workspace --lib --release -- cluster_g  # 230+ Cluster G tests
cargo clippy --workspace -- -D warnings -W clippy::pedantic
atuin scripts run wf-dispatch-refuse-mode-check  # manual confirm refuse-mode message
```

---

### m40 — nexus_event_emitter

**Cluster:** H — Substrate Feedback  
**Binary:** wf-crystallise  
**LOC estimate:** ~150

#### Inputs

| Source | Via |
|---|---|
| m32 `WorkflowDispatchEvent` | Channel fan-out from dispatcher after Conductor acceptance |

#### Outputs

| Target | What is written |
|---|---|
| `{data_dir}/workflow_trace_outbox.jsonl` | `OutboxEnvelope` — durable; append-only |
| SYNTHEX v2 `:8092/v3/nexus/push` | HTTP POST — fire-and-forget; circuit-breaker guarded |

#### Boilerplate lifts

| Source file | LOC lifted | Reuse % | What is lifted |
|---|---|---|---|
| `08-nexus-lcm-rpc/m22_synthex_bridge.rs` | ~90 | 90% | Dual-transport outbox-first + HTTP fire-and-forget; `OutboxEnvelope`; retry sweep (60s interval, max_attempts=5); outbox compaction (24h TTL on posted=true entries) |
| `08-nexus-lcm-rpc/m22_synthex_async.rs` | ~95 | 95% | Circuit breaker FSM (Closed → Open → HalfOpen); exponential backoff ±25% jitter; `spawn_blocking` for sync HTTP (AP29 mitigation, 2s cap not 10s) |

#### WorkflowEvent enum and Option A wire strategy

```
WorkflowEvent serialized into serde_json::Value placed in NexusEvent.data
NexusEvent.event_type carries the discriminator: "workflow_promote", "workflow_run", "workflow_decay"

NexusEvent struct is re-declared locally in workflow_core::nexus_types
DO NOT import from synthex_v2::* (compile-time dependency forbidden)
Wire shape is identical to SYNTHEX v2's inbound NexusEvent
```

Option B (typed enum via shared crate) is a future migration gated on an S118-pattern round-trip integration test. Build Option A only.

#### Circuit breaker discipline (shared across H)

```
CLOSED:   5 failures → OPEN
OPEN:     60 seconds → HALF-OPEN (HTTP skipped; outbox still written)
HALF-OPEN: 1 probe → success: CLOSED / failure: OPEN (restart timer)

Substrate down NEVER blocks dispatch:
  EmitOutcome { appended: true, posted: false, failed: false }
  // appended = true; the event is durable in the outbox
  // failed = false; this is not an error from the engine's perspective
```

#### Tests required (55 minimum)

`WorkflowEvent` enum serialization roundtrips (all 3 variants: Promote/Run/Decay); `OutboxEnvelope` roundtrip; `EmitOutcome` accumulation; dual-transport order (outbox append before HTTP attempt — inject a panicking HTTP stub and verify outbox is already written); circuit breaker state transitions; circuit open skips HTTP but still appends outbox; retry sweep (posted envelopes not re-attempted; failed envelopes re-attempted up to max_attempts); NexusEvent local re-declaration matches wire shape; schema drift: `event_type` field serializes with serde rename `"type"`; Option A untyped JSON preserves `kind` discriminator; `Send + Sync` bounds; concurrent emit calls don't corrupt outbox (test with 10 concurrent threads).

#### Verification gate — m40 complete when

`cargo test --lib -- nexus_event_emitter` shows 55+ passed; circuit breaker correctly transitions through all 3 states in a single test scenario; outbox JSONL file grows by exactly 1 line per `emit()` call in the outbox-first test.

---

### m41 — lcm_rpc_client

**Cluster:** H  
**Binary:** wf-crystallise  
**LOC estimate:** ~200

#### Inputs

| Source | Via |
|---|---|
| m32 `WorkflowDispatchEvent` | Channel fan-out from dispatcher |

m41 only activates when `event.step_kind == StepKind::Deploy`. All other step kinds return `LcmDispatchOutcome::NotApplicable` immediately without attempting a UDS connection.

#### Outputs

| Target | What is written |
|---|---|
| LCM supervisor Unix socket | `lcm.loop.create` JSON-RPC 2.0 request |

#### Boilerplate lifts

| Source file | LOC lifted | Reuse % | What is lifted |
|---|---|---|---|
| `08-nexus-lcm-rpc/m22_synthex_async.rs` | ~95 | 95% | Circuit breaker FSM; 30s read timeout |
| `08-nexus-lcm-rpc/_see_lcm_supervisor.txt` | ~50 | client mirror | JSON-RPC 2.0 newline-framing; request/response envelope shapes; `lcm.ping` for HalfOpen probe |
| `08-nexus-lcm-rpc/m38_deployment_api.rs` | ~40 | 50% | Deploy-step classification and step metadata shapes |

#### JSON-RPC 2.0 wire: lcm.loop.create (NOT lcm.deploy)

```
Request (newline-terminated):
{"jsonrpc":"2.0","id":1,"method":"lcm.loop.create",
 "params":{"caller_id":"wf-<run_id>","name":"deploy:<step_name>",
           "max_iters":1,"survives_session_death":false}}\n

Response (newline-terminated):
{"jsonrpc":"2.0","id":1,"result":{"loop_id":"<uuid>"}}\n
```

`max_iters: 1` maps a deploy workflow step to a single-iteration LCM loop. This is compatible with LCM's existing 9-RPC surface without requiring new methods. The LCM supervisor owns the deploy state machine; m41 is a thin client.

m41 has no persistent connection. Each `dispatch()` call opens a new UDS connection, sends one request, reads one response (30s read timeout), closes. Deploy steps are infrequent; keepalive complexity is not warranted.

#### Tests required (50 minimum)

`LcmLoopCreateRequest` serialization (JSON-RPC 2.0 shape verified against expected JSON string); `LcmLoopCreateResponse` deserialization (result.loop_id extracted); deploy-shape detection (`StepKind::Deploy` routes to LCM; non-deploy → `NotApplicable` without UDS attempt); circuit breaker Closed/Open/HalfOpen; circuit open returns `ServiceUnavailable`, not an error; read timeout returns `Timeout`; RPC error response parsed into `RpcError { code, message }`; newline framing (request ends with `\n`; response parsed line-by-line); `lcm.ping` used in HalfOpen probe; `Send + Sync` bounds; reconnect on each call.

#### Verification gate — m41 complete when

`cargo test --lib -- lcm_rpc_client` shows 50+ passed; `NotApplicable` returned without UDS connection attempt for non-deploy step (confirmed by test that panics on UDS connect for non-deploy paths).

---

### m42 — hebbian_feedback

**Cluster:** H  
**Binary:** wf-crystallise  
**LOC estimate:** ~100

#### Inputs

| Source | Via |
|---|---|
| m32 `WorkflowDispatchEvent` | Channel fan-out from dispatcher; carries `RunOutcome` |

#### Outputs

| Target | Via |
|---|---|
| POVM Engine `:8125/reinforce` (overlap window) | HTTP POST — raw TCP, BUG-033 socket address |
| stcortex via m13 (post-cutover 2026-07-10) | `m13::stcortex_writer_narrowed` |

#### Boilerplate lifts

| Source file | LOC lifted | Reuse % | What is lifted |
|---|---|---|---|
| `08-nexus-lcm-rpc/m24_povm_bridge.rs` | ~85 | 85% (GOLD STANDARD per Command-3 E3) | `raw_http_post` call pattern; BUG-033 socket address (no `http://` prefix); BUG-034 endpoint path (`/reinforce` not `/pathways`); F-001 silent-swallow fix (log HTTP status, don't ignore); 1h idempotency dedup via `request_id` |

#### fitness_delta table

| Outcome | fitness_delta | Rationale |
|---|---|---|
| `RunOutcome::PassVerified` | `+0.25` | Dispatched AND independently verified by m33 — best signal |
| `RunOutcome::Pass` | `+0.15` | Standard success |
| `RunOutcome::Blocked` | `-0.05` | Conductor refused — mild LTD; may be appropriate refusal |
| `RunOutcome::Fail` | `-0.10` | Step failed — stronger LTD signal |

Constants: `FITNESS_PASS_VERIFIED`, `FITNESS_PASS`, `FITNESS_BLOCKED`, `FITNESS_FAIL`. Named constants, not magic numbers in call sites. Clamped to `[-1.0, 1.0]` before sending (Hebbian v3 clamp pattern).

#### AP30 namespace prefix discipline

All `retrieval_ids` in `ReinforcePayload` must begin with `workflow_trace_`. Examples:
- `workflow_trace_wf-abc123` → `workflow_trace_outcome_pass_verified`
- `workflow_trace_wf-abc123` → `workflow_trace_outcome_fail`

m9 (`watcher_namespace_guard`) enforces this at write boundaries. m42 generates the prefixed IDs; m9 validates them. The AP30 anti-pattern (collision with V3's `P01..P16` Hebbian pathway IDs) is prevented by this prefix convention.

#### POVM deprecation cutover — dual-path

```rust
// Default: povm_overlap_active = true (safe — prefers POVM until explicitly flipped)
// Post-cutover 2026-07-10: set povm_overlap_active = false in config

fn route_reinforcement(payload) -> ReinforceOutcome:
  if self.config.povm_overlap_active:
    self.post_to_povm(payload)       // BUG-033 + BUG-034 applied
  else:
    self.write_to_stcortex_via_m13(payload)

// CRITICAL: if stcortex unreachable after cutover:
//   log at ERROR level
//   return ReinforceOutcome::SubstrateUnavailable
//   DO NOT silently fall back to POVM (CLAUDE.md stcortex policy)
```

#### Tests required (55 minimum per CC-5 spec; 50 minimum per cluster-H spec)

`ReinforcePayload` serialization roundtrip; AP30 namespace: all `retrieval_ids` start with `workflow_trace_`; fitness_delta constants match spec table; fitness_delta clamping (values outside [-1.0, 1.0] clamped); overlap window active → routes to POVM; overlap window inactive → routes to stcortex; post-cutover stcortex unavailable → `SubstrateUnavailable`, NOT POVM fallback; circuit breaker transitions; POVM unreachable → returns `SubstrateUnavailable`, does not block; idempotency dedup (duplicate `request_id` within 1h → `Skipped`); BUG-033 (no `http://` in socket addr); BUG-034 (endpoint is `/reinforce`); F-001 (non-200 status logged, not silently dropped); all `ReinforceOutcome` variants constructed in tests; `Send + Sync` bounds.

#### Verification gate — m42 complete when

`cargo test --lib -- hebbian_feedback` shows 55+ passed; `ReinforcePayload.retrieval_ids[0]` starts with `workflow_trace_` in every test; post-cutover test confirms stcortex path is taken and POVM is not contacted (verified by injecting a panicking POVM stub when `povm_overlap_active = false`).

#### Atuin trajectory (Cluster H complete)

```bash
cargo test --workspace --lib --release -- cluster_h   # 165+ Cluster H tests
cargo clippy --workspace -- -D warnings -W clippy::pedantic
atuin scripts run wf-trace-ap30-namespace-check       # verify no non-prefixed pathway IDs
```

---

## Cross-Cluster Integration Testing (Days 19-21)

### CC-4 verification: Proposal → Bank → Dispatch pipeline

Integration test scenario:
1. Populate `cascade_proposals` with 3 proposals (n ≥ 20, Wilson CI valid)
2. Simulate `wf-crystallise propose accept <id>` → confirm `BankDb::accept()` called
3. Assert m23's `proposals` table row transitions to `Accepted`
4. Assert m23 never directly called `BankDb::accept()` (structural guarantee)
5. Assert `workflow_bank.db` has the corresponding `AcceptedWorkflow` row

### CC-6 verification: definition drift detection

Integration test scenario:
1. Store `AcceptedWorkflow` with steps S1, S2, S3
2. Run m33 verification → `PASS` → `last_verified_at` written
3. Retrieve workflow from m30 → modify `steps_json` (change S2's kind)
4. Run m32 5-check gate → `DispatchError::DefinitionDrifted`
5. Assert no audit row written to `dispatch_log.db`

### CC-5 verification: substrate learning loop

Integration test scenario:
1. Mock Conductor accepting a dispatch request
2. Assert m40 `workflow_trace_outbox.jsonl` contains the `WorkflowEvent::Run`
3. Assert m41 received a `lcm.loop.create` call (for a deploy-shaped step)
4. Assert m42 `ReinforcePayload` carries `fitness_delta = +0.15` for `RunOutcome::Pass`
5. Assert `retrieval_ids` all prefixed with `workflow_trace_`
6. Inject circuit-open condition on m40 → assert dispatch completes anyway (substrates never block)

---

## Overview Sections

### Cluster F deviation-as-evidence loop (full trace)

The loop is closed when all five links are wired:

- **Link 1:** m32 presents a step to the operator via `display-before-step` banner
- **Link 2:** Operator types a bypass rationale; m32 writes `deviation_events` row
- **Link 3:** m23 reads `deviation_events` on next aggregation cycle; joins to proposals containing the bypassed step
- **Link 4:** If rationale similarity ≥ 0.8 cosine across 3+ bypass events: m23 creates deviation-shaped variant (n ≥ 5 relaxed threshold); sets `deviation_shaped = true`
- **Link 5:** Human reviewer sees deviation-shaped variant in proposal surface → may accept → m30 bank → dispatched → m40/m41/m42 learn → m31 selection shifts

Never auto-promotes. The loop is purely advisory: it generates evidence and presents it to the human. The human decides. If no bypass rationale is ever collected, `deviation_evidence` remains empty and `deviation_shaped` remains false — the system is correct and complete in both cases.

### Cluster G HUMAN-IN-THE-LOOP gate

The human-in-the-loop gate is enforced by three structural guarantees:

1. **m23 is write-blind to m30:** m23's `aggregate_and_write()` only writes to `workflow_trace.db` table `proposals`. The bank (`workflow_bank.db`) is a different database file. m23 has no connection string to `workflow_bank.db`. This is not a convention — it is a compile-time isolation.

2. **CLI owns the transition:** The subcommand `wf-crystallise propose accept <id>` is the only code path that calls `BankDb::accept()`. No background task, sweep, timer, or agent may call it.

3. **Human command = explicit intent:** The accept command requires `<id>` — a specific proposal identifier. There is no `--all` flag, no batch accept, no automatic acceptance when confidence exceeds a threshold. The human sees a specific proposal and decides.

This is the CC-4 contract enforced: F → human review → G. The Conductor never receives a workflow that a human has not explicitly ratified.

### Cluster H circuit breaker discipline

All three Cluster H modules share the same circuit breaker FSM (lifted from m22_synthex_async.rs). The critical invariant:

**Substrates being down NEVER blocks dispatch.**

This means:
- m40 with SYNTHEX unreachable: outbox appended (durable), HTTP skipped, `EmitOutcome { failed: false }` returned
- m41 with LCM unreachable: `LcmDispatchOutcome::ServiceUnavailable` returned, no error propagated to m32
- m42 with POVM unreachable: `ReinforceOutcome::SubstrateUnavailable` returned, no error propagated to m32

m32's fan-out is fire-and-forget: if any of the three H modules returns an error, m32 logs at WARN and continues. The workflow has already been handed to the Conductor.

The only post-cutover exception: if stcortex is unreachable after 2026-07-10 and `povm_overlap_active = false`, m42 logs at ERROR (not WARN). This is a substrate misconfiguration that warrants attention, not just a transient outage.

### Conductor maturity check

m32 will NOT silently degrade. Two states:

**REFUSE mode** (when `CONDUCTOR_DISPATCH_ENABLED != "1"`): Every `Dispatcher::dispatch()` call returns `DispatchError::ConductorDispatchDisabled`. The binary prints a hard error to stdout naming the env var and the expected Conductor Wave sequence. The binary does not panic and does not exit — it remains running so that other wf-dispatch subcommands (bank list, verify, etc.) remain functional. Only the dispatch subcommand is blocked.

**LIVE mode** (when `CONDUCTOR_DISPATCH_ENABLED = "1"`): The Conductor health probe runs at startup and before every dispatch. A health probe failure in live mode returns `DispatchError::ConductorNotLive` — also not a silent no-op.

Luke's terminal sequence to unblock live mode (from CLAUDE.local.md HABITAT-CONDUCTOR row):
```bash
devenv start weaver && devenv start zen && devenv start enforcer
curl :8141/health          # must return {"status":"ok"}
# → Wave 2 WASM deploy → 24h NoOp soak
# → export CONDUCTOR_ENFORCEMENT_ENABLED=1
# → export CONDUCTOR_DISPATCH_ENABLED=1
```

### POVM deprecation cutover plan (2026-07-10)

m42 is the only module that requires active cutover management. The dual-path is built at genesis; cutover is a configuration flip:

**Pre-cutover (now → 2026-07-10):**
- `povm_overlap_active = true` in m42 config (safe default)
- m42 `POST /reinforce` → POVM at `127.0.0.1:8125`
- m31 reads pathway weights from stcortex (overlap: read both; write stcortex only per CLAUDE.md stcortex policy)

**Post-cutover (2026-07-10+):**
- Set `povm_overlap_active = false` in m42 config
- m42 routes to stcortex via `m13::stcortex_writer_narrowed`
- m31 reads pathway weights directly from stcortex (POVM decommissioned)
- If stcortex unreachable: `ERROR` log + `SubstrateUnavailable` — NO silent POVM fallback

The stcortex write path in m42 maps `ReinforcePayload` to `mcp__stcortex-mcp__stcortex_write_pathway` in the `workflow_trace_*` namespace. The namespace convention carries from POVM to stcortex unchanged.

### Hand-off to Phase 3 integration

Phase 2B is complete when the following state is true on Day 21 close:

1. **11 modules with tests green:**
   - m20: 80+ tests; PrefixSpan mines patterns correctly on synthetic sequence DB
   - m21: 50+ tests; IQR outlier detection + Pareto frontier proposals generated
   - m22: 50+ tests; K-means converges; < 20-member clusters dropped
   - m23: 50+ tests; deviation-evidence loop wired; m30 bank untouched
   - m30: 55+ tests; WAL-mode SQLite; decay tick correct; sunset marking
   - m31: 60+ tests; composite score formula correct; diversity enforcement firing
   - m32: 60+ tests; refuse-mode message legible; audit-first guarantee
   - m33: 55+ tests; 4-agent gate structure; definition hash correct; TTL gating
   - m40: 55+ tests; outbox-first dual-transport; circuit breaker correct
   - m41: 50+ tests; `lcm.loop.create` wire correct; non-deploy → NotApplicable
   - m42: 55+ tests; AP30 prefix; fitness_delta table; dual-path cutover

2. **Three structural gaps demonstrated working:**
   - Gap 1 (PrefixSpan N-step detection): m20 test suite demonstrates gap-allowed matching at MAX_GAP_STEPS=5 with Wilson CI bounds
   - Gap 3 (EscapeSurfaceProfile schema): `EscapeSurfaceProfile::Destructive.step_banner()` renders correct banner; m32 integration test shows banner on stdout before mock Conductor call
   - CC-5 closure: end-to-end test traces dispatch → m40 outbox appended → m42 `ReinforcePayload` with correct namespace prefix

3. **CC-4, CC-5, CC-6 contracts demonstrable:**
   - CC-4: human accept CLI is the only path from m23 → m30 (structural isolation confirmed by test)
   - CC-5: m42 `ReinforceOutcome::Accepted` in integration test + outbox entry in m40
   - CC-6: definition drift detection fires when `steps_json` is mutated post-verification

4. **Full workspace quality gate green:**
   ```bash
   cargo check --workspace
   cargo clippy --workspace -- -D warnings -W clippy::pedantic
   cargo test --workspace --lib --release
   # Expected: 630+ tests passed (230 F + 230 G + 165 H), 0 failed, 0 warnings
   ```

5. **Ready for Phase 3:** The next phase receives m30-m33 and m20-m23 as stable foundations with known integration test seams at CC-4, CC-5, CC-6.

---

## Substrate state note

This phase ships on an LTD-dominant substrate (LTP/LTD = 0.043; target 1.5-4.0). Watcher Class-I flag is pre-positioned. The m31 selector operates on RALPH decay weights that may not have had time to converge. The 10-gen cooldown and 50% mono-parameter rejection gate provide structural protection against selection lock-in while the substrate matures. The m11 120-day sunset law provides the outer hard boundary.

If `learning_health` does not move during Cluster H's first pipeline runs, Watcher will flag Class-I and recommend injecting a cluster-H probe into T3 or T4 integration testing.

---

*Phase 2B deployment framework authored 2026-05-17 (S1001982) · planning-only · HOLD-v2 active · pre-G9 gate*
*Source material: GOD_TIER_CONSOLIDATION (77 files / 1.9MB), cluster-F/G/H specs, boilerplate Cat 04/07/08/09*
*Test count target: 630+ across 11 modules (230 Cluster F + 230 Cluster G + 165 Cluster H)*
