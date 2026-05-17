---
title: schematic — Gap 1 PrefixSpan algorithm flowchart (m20)
kind: planning-only · Mermaid-only · focused operational schematic
---

# Gap 1 — m20 PrefixSpan Algorithm

> **Back to:** [`../README.md`](../README.md) · [`../ULTRAMAP.md`](../ULTRAMAP.md) · [`../DATA_FLOW.md`](../DATA_FLOW.md) · per-module [`../../ai_specs/modules/cluster-F/m20_prefixspan_miner.md`](../../ai_specs/modules/cluster-F/m20_prefixspan_miner.md)

m20 is the engine's **KEYSTONE** structural-gap authorship — N-step compositional sub-graph detection that cannot be lifted from existing habitat sources (the habitat's pairwise infrastructure in POVM only covers two elements). PrefixSpan was chosen over Apriori (candidate explosion) and N-gram sliding window (no gap-allowed matching).

## Algorithm flowchart

```mermaid
flowchart TD
    classDef gate fill:#fff3cd,stroke:#a67c00,color:#000
    classDef proc fill:#e6f0ff,stroke:#1a4d99,color:#000
    classDef out fill:#cce6cc,stroke:#1a7a1a,color:#000
    classDef refuse fill:#ffd9d9,stroke:#a00000,color:#000

    Start([m20::mine invoked]):::proc

    Start --> CC3{CC-3 evidence gate<br/>m14.Lift variance < threshold?<br/>n ≥ 20?}:::gate
    CC3 -- no --> RefuseCC3[Err MinerError<br/>StabilizationGateNotMet]:::refuse
    CC3 -- yes --> LoadDB[load session-grouped Vec~StepToken~<br/>from cascade_clusters + battern_step_records]:::proc

    LoadDB --> L1Scan[scan database once<br/>find L1 candidates meeting min_support]:::proc

    L1Scan --> L1Filter{any candidates<br/>meet min_support?}
    L1Filter -- no --> EmptyOut[Ok empty Vec~Pattern~]:::out
    L1Filter -- yes --> InitFrontier[initialise frontier<br/>= L1 frequent items]:::proc

    InitFrontier --> Recurse[/recursive call:<br/>extend_pattern prefix=P/]:::proc

    Recurse --> Project[project database<br/>for each sequence containing P<br/>retain suffix following first match<br/>under gap-allowed semantics]:::proc

    Project --> CountExt[count frequent extensions<br/>under MAX_GAP_STEPS = 5]:::proc

    CountExt --> ExtMin{any extensions<br/>meet min_support?}
    ExtMin -- no --> EmitPattern[emit Pattern steps support gap_bounds<br/>add to result set]:::out
    ExtMin -- yes --> DepthCheck{prefix length<br/>< MAX_DEPTH=8?}

    DepthCheck -- no --> EmitPattern
    DepthCheck -- yes --> ExtendLoop[for each frequent extension e<br/>recurse with prefix P + e]:::proc

    ExtendLoop --> Recurse

    EmitPattern --> AllDone{all branches<br/>explored?}
    AllDone -- no --> Recurse
    AllDone -- yes --> Sort[sort patterns<br/>by support DESC then length DESC]:::proc

    Sort --> DiffTest[differential test gate<br/>vs naive O n^3 reference impl]:::gate
    DiffTest --> Out[Ok Vec~Pattern~]:::out
```

## Why PrefixSpan over alternatives

| Algorithm | Why rejected / chosen |
|---|---|
| **Apriori** | REJECTED — level-wise candidate generation; O(80⁴) ≈ 40M candidates at 4-step patterns; database pass per level; structural to algorithm, not tunable |
| **N-gram sliding window** | REJECTED — treats sequences as strings; **cannot skip gaps**; cascade `[read_file → bash → edit → bash → cargo_check]` won't match pattern `[read_file → edit → cargo_check]` because bash interleavings break window; gap-allowed matching is hard requirement |
| **PrefixSpan** (Pei et al. 2001) | CHOSEN — projection-based; bounds search to actually-observed extensions; gap-allowed matching intrinsic to projection step; exact frequency support; reference implementations enable differential testing |

## Output type

```text
Pattern {
    steps: Vec<StepToken>,        // opaque u32 alphabet (F11)
    support: usize,               // exact frequency count
    gap_bounds: (usize, usize),   // min/max gap observed within MAX_GAP_STEPS=5
}
```

Sort order is deterministic: `support DESC`, then `length DESC`. Same input → same output (property test invariant).

## Complexity

O(|D| × L × W) where |D| = number of sequences, L = max pattern length (capped at 8), W = average sequence width (typically 6-20 tool calls per cascade). Bounded depth + bounded width → practical single-pass per recursion level.

## Test discipline (mutation kill ≥80%)

Per [m20 spec § 8](../../ai_specs/modules/cluster-F/m20_prefixspan_miner.md) — m20 is one of two modules (alongside m32) with the engine's highest mutation kill thresholds. Property tests verify:

- determinism (same input → same output)
- monotonicity (raising min_support cannot add patterns)
- gap-bound respect (no pattern with gap > MAX_GAP_STEPS=5)
- depth-bound respect (no pattern with length > 8)
- differential equivalence vs O(n³) naive reference

## Downstream consumers

```
m20.Pattern → m21.WorkflowVariant (Levenshtein clustering)
                    → m22.FeatureCluster (K-means)
                              → m23.WorkflowProposal (gradient-preservation top-K N=3)
```

If m20 is broken, every downstream proposal is garbage. The engine's reason-for-existing.

---

> **Back to:** [`../ULTRAMAP.md`](../ULTRAMAP.md) · [`../../ai_specs/modules/cluster-F/m20_prefixspan_miner.md`](../../ai_specs/modules/cluster-F/m20_prefixspan_miner.md)
