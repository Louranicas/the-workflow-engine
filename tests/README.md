# tests/ — PLACEHOLDER (no test files pre-G9)

> **Status:** EMPTY by design until G9 fires.
> **Test budget:** 1,562-1,599 across 26 modules per [`../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md`](../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md)

---

## What lands here at G9-fire

Integration tests live in `tests/` (Cargo convention — separately compiled per file). Unit tests live alongside source in `src/m<N>_<name>/tests.rs`.

```
tests/
├── cc1_cascade_cost_coupling.rs           # CC-1: m4 ↔ m6 via m7 JSONB
├── cc2_trust_layer_woven.rs               # CC-2: D → all
├── cc3_evidence_driven_iteration.rs       # CC-3: E → F
├── cc4_proposal_bank_dispatch.rs          # CC-4: m23 → m30 → m32 → CONDUCTOR
├── cc5_substrate_learning_loop.rs         # CC-5: m32 → H → F read-back
├── cc6_verification_gated_dispatch.rs     # CC-6: m33 → m32
├── cc7_pressure_driven_evolution.rs       # CC-7: m15 → operator
├── e2e_wf_crystallise.rs                  # Binary 1 end-to-end
├── e2e_wf_dispatch.rs                     # Binary 2 end-to-end
└── outbox_survival.rs                     # H cluster: substrate outage recovery
```

Test kinds per [`../ai_specs/MODULE_MATRIX.md`](../ai_specs/MODULE_MATRIX.md) column "Test kind".

---

> **Back to:** [`../README.md`](../README.md) · [`../GOLD_STANDARDS.md`](../GOLD_STANDARDS.md) · [`../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md`](../ai_docs/optimisation-v7/STANDARDS/TEST_DISCIPLINE.md)
