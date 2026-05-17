> Back to: [[HOME]]

---
**Source repository:** `/home/louranicas/claude-code-workspace/habitat-loop-engine`  
**Commit SHA:** 568d9e3  
**Document version:** v1.0 (2026-05-17)  
**Scope:** Comprehensive gold-standard scaffold exemplar for workflow engine codebases  
**Purpose:** Reference profile for the-workflow-engine scaffolding based on HLE patterns

---

## 1. One-Line Essence

Habitat Loop Engine (HLE) is a **bounded local workflow substrate** implementing executor/verifier separation with durable evidence receipts, runbook-aware AwaitingHuman semantics, and trait-first cross-service contracts. Role: foundational M0 framework for one-shot, foreground, locally-bounded workflow operations under Habitat deployment governance. Status: M0 runtime active (4/50 modules built; 50-module planned topology spec'd); scaffold + M0 quality gates PASS.

---

## 2. Top-Level Structure

```
habitat-loop-engine/
├── crates/                          # 10-crate workspace (3 substrate + 7 HLE)
│   ├── substrate-types/             # M001 L01: foundational types (ProjectStatus, Authorization, StepState, Workflow, Receipt)
│   ├── substrate-verify/            # M002 L04: verifier authority (verify_authorization, verify_step, verify_report)
│   ├── substrate-emit/              # M003 L02/L03: executor + ledger (JSONL receipts, local runner, SHA receipt verification)
│   ├── hle-cli/                     # M004 L06: CLI surface (hle run, verify, daemon --once, scan, audit, status, taxonomy)
│   ├── hle-core/                    # Foundation types for M005-M009 (receipt_hash, claims_store, claim_authority, workflow_state)
│   ├── hle-storage/                 # L02 persistence (migrations, pool, stores: workflow_runs, ticks, evidence, receipts, blockers)
│   ├── hle-executor/                # L03 executor (state_machine, phase_executor, local_runner, timeout/retry policies)
│   ├── hle-verifier/                # L04 verifier (SHA verifier, claim authority audit, false-pass auditor, anti-pattern scanner)
│   ├── hle-runbook/                 # L07 runbook semantics (schema, parser, AwaitingHuman, manual evidence, incident replay)
│   └── hle-bridge/                  # L05 dispatch bridges (zellij, atuin, devops-v3, stcortex, watcher—read-only until M2+)
├── migrations/                      # 2 SQL schemas (0001 scaffold, 0002 M025-M031 persistence ledger)
├── docs/                            # Operations, plans, quality, reviews, workflows
├── ai_specs/                        # 50 formal module specs (MEv2 L1 gold standard)
├── ai_docs/                         # Narrative layer/module/cluster docs (L01-L07, M001-M004 LITE)
├── schematics/                      # 11+ mermaid diagrams (layer-dag, executor-verifier-sequence, receipt-graph)
├── schemas/                         # Machine contracts (receipt.schema.json, status.schema.json, plan.schema.json)
├── scripts/                         # 22 verify scripts + quality-gate.sh orchestrator
├── bin/                             # 24 CLI wrappers (hle-verify-sync, hle-quality-gate, etc.)
├── vault/                           # CONVENTIONS.md (markdown authority, JSONL substrate, no execution PASS without round-trip verification)
├── tests/                           # Python unit/integration tests + fixtures
├── runbooks/                        # Operator runbooks (m0-authorization-boundary, verification-and-sync-pipeline)
├── plan.toml                        # Machine-readable config: layers, modules, planned_modules (M005-M054), clusters
├── CLAUDE.md                        # Project authority
├── CLAUDE.local.md                  # Local operator overlay, start-here checklist
├── README.md                        # Mission, boundary summary, verification pipeline, planned topology
├── QUICKSTART.md                    # Executable command path
├── MASTER_INDEX.md                  # Single navigational entry point
├── ULTRAMAP.md                      # Layer/module/source alignment authority
├── ARCHITECTURE.md                  # 7-layer DAG + forbidden edges
├── QUALITY_BAR.md                   # MEv2 L1 standard + semantic predicate bars
├── HARNESS_CONTRACT.md              # Scaffold receipt contract (split hash anchors, S13 substrate)
├── .deployment-work/status/         # scaffold-status.json, m0-status.json (JSON PASS/FAIL authority)
├── .deployment-work/receipts/       # Authorization receipts
├── .deployment-work/runtime/        # Local JSONL receipt ledgers
└── SHA256SUMS.txt                   # Manifest verification
```

---

## 3. Workspace Crate Architecture — 10 Crates, 47 KLoC

| Crate | Layer | Purpose | Dependencies | LOC |
|-------|-------|---------|--------------|-----|
| substrate-types | L01 | Foundation types (ProjectStatus, Authorization, StepState, Workflow, Receipt, ExecutionReport, HleError) | None | 682 |
| substrate-verify | L04 | Verifier gates (verify_authorization, verify_step, verify_report) | substrate-types | 743 |
| substrate-emit | L02/L03 | Executor + JSONL ledger (execute_local_workflow, append_jsonl_receipt, receipt SHA verification) | substrate-types, substrate-verify, sha2 | 1015 |
| hle-cli | L06 | CLI surface (hle run, verify, daemon --once, scan, audit, status, taxonomy) | All upstream | 8,391 |
| hle-core | L01 | Foundation for M005-M009 (receipt_hash, claims_store, claim_authority, workflow_state) | substrate-types, sha2 | 36 |
| hle-storage | L02 | SQLite persistence (migrations, pool, stores: workflow_runs, ticks, evidence, receipts, blockers) | hle-core, substrate-types, rusqlite | 6,916 |
| hle-executor | L03 | Executor primitives (state_machine, phase_executor, local_runner, timeout/retry policies) | hle-core, hle-storage, substrate-types, sha2 | 5,818 |
| hle-verifier | L04 | Verification (receipt SHA, claim authority audit, false-pass auditor, anti-pattern scanner) | hle-core, hle-storage, substrate-types | 6,882 |
| hle-runbook | L07 | Runbook semantics (schema, parser, AwaitingHuman, manual evidence, incident replay, safety policy) | hle-core, hle-executor, hle-storage, substrate-types | 7,583 |
| hle-bridge | L05 | Dispatch bridges (zellij, atuin, devops-v3, stcortex, watcher—read-only until M2+) | hle-core, hle-storage, substrate-types, ureq | 6,337 |

**Key constraints:**
- All crates: `forbid(unsafe_code); deny unwrap_used/expect_used/panic/todo/dbg_macro`
- hle-verifier intentionally does NOT depend on hle-executor (UP_EXECUTOR_VERIFIER_SPLIT enforced at cargo level)
- Total: 3,165 test markers (all stubs; M0 authorization gates implementation)

---

## 4. Substrate Layer — Typed Events & Verification Contract

**substrate-types (M001, L01):** Zero-dependency foundational types.
- ProjectStatus: ScaffoldOnly, M0Runtime, LiveIntegrated, Deployed
- Authorization: m0_runtime, live_integrations, cron_daemons flags
- StepState: Pending, Running, AwaitingHuman, Passed, Failed, RolledBack
- Workflow, WorkflowStep, Receipt, ExecutionReport, HleError

**substrate-verify (M002, L04):** Verifier-exclusive authority gates.
- `verify_authorization(auth: Authorization) -> Result<(), HleError>` — rejects unauthorized scopes
- `verify_step(workflow, step_id, draft_state) -> Result<Receipt, HleError>` — converts executor draft to final verdict (Passed/Failed/AwaitingHuman)
- `verify_report(execution_report) -> Result<VerificationReport, HleError>` — audits entire execution report

**Key invariant:** Executor cannot emit PASS; only verifier can promote receipt from draft to final verdict.

**substrate-emit (M003, L02/L03):** Local executor + ledger substrate.
- `execute_local_workflow(workflow) -> Result<ExecutionReport, HleError>` — bounded CLI command runner
- `append_jsonl_receipt(path, receipt) -> Result<(), HleError>` — append verifier receipt to local JSONL ledger
- `receipt_to_json_line(receipt) -> Result<String, HleError>` — serialize receipt with canonical SHA-256 hash

All three stubs today; M0 implementation gated by `begin M0` phrase.

---

## 5. Capabilities Catalogue — 45+ HLE Surfaces

**L03 Executor (C03):** bounded local runner, phase executor, timeout policy (TERM→KILL), retry policy, output limiter, state machine

**L04 Verifier (C01/C02/C04):** authorization verifier, receipt SHA verifier, claim authority verifier, test taxonomy verifier, false-pass auditor, anti-pattern scanner

**L02 Storage (C05):** connection pool, migration runner, workflow runs store, ticks store, evidence store, step receipts store, verifier results store, blockers store, anti-pattern events store

**L07 Runbook (C06):** runbook schema, parser, phase mapper, AwaitingHuman confirmation, manual evidence attachment, incident replay, safety policy, scaffold generator

**L05 Bridges (C07):** zellij dispatch, atuin QI chain, devops-v3 probe, stcortex anchor bridge (gated), watcher notice writer

**L06 CLI (C08):** hle run, hle verify, hle daemon --once, hle scan, hle audit, hle status, hle taxonomy

**C09 DevOps/QI Lane:** verify-sync.sh, quality-gate.sh, verify-module-map.sh, verify-layer-dag.sh, verify-antipattern-registry.sh, verify-receipt-schema.sh, and 16 more

---

## 6. Plan-Driven Scaffolding & Layered Phase Model

**plan.toml** drives generation:
```toml
[authorization]
scaffold = true          # Docs/types/specs authorized
m0_runtime = true       # Bounded local runtime authorized
live_integrations = false  # NOT authorized
cron_daemons = false     # NOT authorized

[[modules]]
id = "M001"
name = "substrate-types"
layer = "L01"
crate = "crates/substrate-types"

[[planned_modules]]
id = "M005"
name = "receipt_hash"
layer = "L01"
cluster = "C01_EVIDENCE_INTEGRITY"
source_path = "crates/hle-core/src/evidence/receipt_hash.rs"

[[full_codebase_clusters]]
id = "C01_EVIDENCE_INTEGRITY"
layers = "L01/L02/L04"
module_surfaces = 5
synergy = "receipt hash → claim store → receipt store → verifier recompute → final claim evaluator"
```

**Phase model (P0–P5+):**
- P0: Scaffold only (docs/types, no runtime)
- P1: M0 local bounded (foreground, verifier-gated, JSONL ledger) ← **CURRENT**
- P2: Live Habitat write integration
- P3: Operator loop (recurring dispatch, no unbounded daemon)
- P4: Distributed execution
- P5+: Deployment claims

**Scaffold practice pattern:**
1. Write spec at `ai_specs/modules/cXX-slug/M0XX_NAME.md` (MEv2 L1: header, Types-at-a-Glance, Rust signatures, Design Notes, cluster invariants)
2. Create stub Rust file at `crates/hle-xyz/src/module_name.rs` (compile-safe, `#[allow(dead_code)]` until M0)
3. Link layer doc, module doc, source file (bidirectional cross-references)
4. Run `scripts/quality-gate.sh --scaffold --json` → validate alignment
5. When `begin M0` issued: implement module; re-run `scripts/quality-gate.sh --m0 --json`

---

## 7. Quality Gate Stack — Compose 28 Checks into PASS/FAIL

**Scaffold gate (`--scaffold`):**
- 22 verify scripts (sync, docs, claude, patterns, receipts, safety, local-M0)
- cargo fmt --check (Rust formatting)
- cargo check --workspace --all-targets (skeleton compile)
- cargo test --workspace --all-targets (skeleton tests)
- cargo clippy --workspace --all-targets -- -D warnings (zero warnings)
- python3 tests/unit/test_manifest.py (manifest presence)
- python3 tests/integration/test_scaffold.py (sync + negative controls)

**Local M0 gate (`--m0`):**
- All of above, PLUS:
- Verifier scripts that probe M0 runtime surfaces (CLI markers, SQLite schema presence, JSONL boundedness)
- Negative controls proving false passes fail

**Semantic predicates (HLE-SP-001..003):**
- SP-001: anti-pattern docs require detector semantics, negative controls, remediation expectations
- SP-002: S01–S13 specs require acceptance gates rejecting premature PASS claims
- SP-003: verifier scripts map checklist bars to explicit predicate IDs and evaluator paths

**Output:** `.deployment-work/status/scaffold-status.json` is sole PASS/FAIL authority (no prose).

---

## 8. Persistence & Storage — Local-Only SQLite Ledger

**Migrations:**
- `0001_scaffold_schema.sql`: workflow_runs, step_receipts with indexes
- `0002_m028_m029_m030_m031.sql`: workflow_ticks, evidence_store, verifier_results_store, blockers_store, anti_pattern_events

**Persisted:** Workflow runs (name, status, timestamps), step receipts (immutable append-only), evidence blobs, verifier results, blocker state, anti-pattern scanner events.

**Invariant:** Local-only (`.deployment-work/runtime/`); no live Habitat database mutations during M0.

---

## 9. Bridges + Integrations (Read-Only Scaffolding, L05)

**Read-only bridges (M2+ for write-gating):**
- zellij_dispatch: read pane/layout state
- atuin_qi_bridge: read shell history
- devops_v3_probe: read DevOps V3 cluster state
- stcortex_anchor_bridge: read memory anchors (write-gated)
- watcher_notice_writer: emit receipts to Watcher dashboard

**Wire formats:** JSON-RPC over stdio (Zellij), REST w/ Bearer token (external), SQLite pragma (local).

---

## 10. Test Discipline — 3,165 Markers, Negative Controls, P3-Ignored

**Test count:** grep reveals 3,165 test markers across 10 crates (all stubs pending M0 authorization).

**Taxonomy:** `[test:unit]` / `[test:integration]` / `[test:smoke]` / `[test:soak]` / `[test:negative]` (false-pass traps that must stay negative).

**P3-ignored pattern:** `#[ignore]` gates tests blocked by P3+ authorization. Current tests are P0–P1 only.

**Negative controls:** `tests/fixtures/negative/` (e.g., tampered_receipt.jsonl with modified field → SHA mismatch → FAIL).

---

## 11. Docs Discipline — Partitioned Authority Surfaces

| Surface | Purpose |
|---------|---------|
| `docs/operations/SCAFFOLD_RECEIPT.md` | Receipt anatomy, split hash anchor meaning, counter-evidence locators |
| `docs/plans/full-codebase-deployment-cluster-synergy-plan.md` | 9-cluster synergy model |
| `docs/quality/semantic-predicates.md` | HLE-SP-001..003 detailed specs |
| `docs/reviews/watcher-scaffold-assessment-20260510.md` | External observer review + boundary warnings |
| `docs/workflows/scaffold-m0-broadening-review-20260510T112840Z.md` | Weaver change review + receipts |
| `docs/SCRIPT_SPEC_PREDICATE_MAP.md` | Maps 22 verify scripts to predicates (single source of truth for Watcher/CI ingestion) |
| `ai_specs/modules/c01-c09/` | 50 formal module specs (MEv2 L1: header, Types-at-a-Glance, Rust signatures, Design Notes, cluster invariants) |
| `ai_docs/layers/L01_FOUNDATION.md..L07_RUNBOOK_SEMANTICS.md` | Layer narratives |
| `MASTER_INDEX.md` | Single navigational entry point; every linked surface backlinks here |

---

## 12. Session-Resume + Drift Discipline

**Cold-start anchor:** `SESSION_RESUME_2026-05-15.md` (readiness state, drift register, supervisor protocol).

**Drift catalogue (11 dimensions):**
1. plan.toml ↔ ULTRAMAP.md alignment
2. Layer doc count and structure
3. Module spec count and format
4. Anti/use-pattern registry breadth
5. Cargo compile-safety + lints
6. Script wrapper parity (bin/hle-* ↔ scripts/)
7. Receipt schema + split hash anchor presence
8. JSONL ledger boundedness + verifier authority
9. Forbidden runtime patterns (live writes, cron, daemons)
10. Manifest freshness (SHA256SUMS.txt)
11. Cross-reference parity (reciprocal backlinks)

**Supervisor protocol:** On resume, run `scripts/verify-sync.sh` (fast check), inspect `.deployment-work/status/scaffold-status.json`, reconcile authority files, refresh SHA256SUMS.txt, re-run gates.

---

## 13. Patterns to Emulate

**Workspace-of-crates:** 3 substrate + 7 feature crates; dependencies flow upward (substrate → core → storage/executor/verifier → bridge/runbook/cli).

**Substrate-types zero-dep:** Enables cross-service types without circular deps.

**Plan.toml-driven:** Declare all 50 modules upfront (planned_modules); gate module-count changes with `scripts/verify-source-topology.sh --strict`.

**Quality-bar enforcement:** Forbid unsafe/unwrap/panic/todo/dbg at workspace lint level; no "fill later" stubs.

**Drift register:** Document 11 known drift modes in `.claude/rules/drift-register.md`; audit on session resume.

**Executor/verifier separation enforced at cargo-dep level:** hle-verifier does NOT depend on hle-executor.

**MEv2 L1 specs as gold standard:** 50 modules × 9 clusters with header, Types-at-a-Glance, Rust signatures, Design Notes, cluster invariants (copy from `/the_maintenance_engine_v2/ai_specs/m1-foundation-specs/`).

---

## 14. Anti-Patterns & Known Traps

**Drift register + retracted claims:** PASS/FAIL claims MUST cite JSON output, not prose. RCA: early HLE retracted "complete" during next session when drift detected.

**Cancel handler RCA:** Foreground-only + `--once` flag prevents unbounded daemons. Cleanup must be synchronous, bounded.

**Executor self-certification:** Cargo-dep enforcement: hle-verifier does NOT depend on hle-executor. Verifier is sole PASS/FAIL authority.

**Receipt hash forgery:** JSONL ledger includes canonical SHA-256. hle verify independently recomputes hash.

**Runbook becoming a second engine:** Runbooks delegate to executor/verifier; they do NOT implement state machines. UP_RUNBOOK_AWAITING_HUMAN restricts scope.

**Local helpers becoming daemons:** verify-local-loop-helpers.sh scans for loop/sleep/cron patterns. Helpers must return within 30s.

**Manifest stale after docs edit:** Always refresh SHA256SUMS.txt after doc/status changes. Checklist in CLAUDE.local.md.

---

## Conclusion

HLE demonstrates:
- **Layered architecture** (L01–L07) with forbidden edges enforced via cargo deps
- **Trait-first boundaries** (substrate-types zero-dep, verifier authority isolated)
- **Plan-driven generation** (plan.toml → specs → stubs → implementation)
- **Verification-first culture** (22 verify scripts, JSON authority, negative controls)
- **Drift discipline** (11-dimension register, cross-reference parity, session-resume protocol)
- **Quality enforcement** (MEv2 L1 semantic predicates, workspace-wide lints, split hash anchors)

For **the-workflow-engine:** Copy 10-crate structure, plan.toml layout, 22+ verify scripts, MASTER_INDEX discipline, session-resume protocol, JSONL ledgers, executor/verifier separation, MEv2 L1 specs.

---

*Document version 1.0 · Source: 568d9e3 · Generated 2026-05-17 · Comprehensive gold-standard reference for workflow engine scaffolding*
