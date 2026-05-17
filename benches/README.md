# benches/ — PLACEHOLDER (no benchmark files pre-G9)

> **Status:** EMPTY by design until G9 fires.
> **Bench framework:** [criterion](https://docs.rs/criterion) — declared in [`../plan.toml`](../plan.toml) `[dev-dependencies]`.

---

## What lands here at G9-fire

Per [`../ai_specs/MODULE_MATRIX.md`](../ai_specs/MODULE_MATRIX.md) "Test kind = bench" + Cluster F KEYSTONE bench requirement:

```
benches/
├── prefixspan_10k.rs              # m20 — 10k atuin rows
├── prefixspan_100k.rs             # m20 — 100k stress
├── cascade_correlator_scale.rs    # m4 — 1M pairwise correlations
├── workflow_runs_hub_ingest.rs    # m7 — JSONB insert rate
├── m23_proposer_top_k.rs          # m23 — top-K-by-distance N=3 at scale
└── m32_dispatcher_5check.rs       # m32 — 5-check pre-dispatch latency
```

Performance budgets: see [`../ai_docs/PERFORMANCE.md`](../ai_docs/PERFORMANCE.md) (TBD Wave 2).

---

> **Back to:** [`../README.md`](../README.md) · [`../GOLD_STANDARDS.md`](../GOLD_STANDARDS.md)
