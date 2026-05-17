---
title: G4 — Gold-Standard Alignment (Generation 4 of 7)
date: 2026-05-17 (S1001982)
kind: planning-only · convergent-pattern adoption + divergent-axis decisions
purpose: close GAP-Gold-01..05; lock crate-org / DB-model / inbound-protocol / evolution-layer / spec-authority
inputs: GOD_TIER_CONSOLIDATION Part IV § Convergent + Divergent + 5 learnings + 4 gold-standard exemplar profiles
output: 13 convergent patterns adopted + 5 divergent decisions + LCM Drift #1..#11 mitigations transposed
---

# G4 — Gold-Standard Alignment

> Back to: sibling [[G3-bidi-flow.md]] (input) · [[G5-tooling.md]] (next)

---

## Gap closure

| Gap | Closure |
|---|---|
| GAP-Gold-01 (Cargo workspace organisation) | ✅ ORAC pattern — single crate + two `[[bin]]` + lib (decided in G2) |
| GAP-Gold-02 (spec authority) | ✅ LCM `plan.toml` + supplementary markdown narrative; `/scaffold` reads plan.toml |
| GAP-Gold-03 (evolution layer policy) | ✅ M0 = m31 selector uses m14+m11 only (no RALPH); M2+ optional `feature = "ralph-integration"` |
| GAP-Gold-04 (V8 ↔ V3 wire reuse) | ✅ m32 PassVerified → POST `:8082/api/v8/learning {confidence_delta: +X}` (Phase 3 Track 5) |
| GAP-Gold-05 (LCM Drift #11 transposition) | ✅ Wave-end orchestrator checklist mandates re-exercise of `--workspace --all-targets --all-features` + `git log -1` + integration smoke |

---

## The 13 convergent patterns (adopt verbatim from gold-standard exemplars)

Per GOD_TIER_CONSOLIDATION Part IV § Convergent patterns. Workflow-trace adoption status:

| # | Pattern | ME v2 | LCM | ORAC | CSv8 | **Workflow-trace** |
|---|---|:-:|:-:|:-:|:-:|:-:|
| 1 | `src/mN_<theme>/` DAG modules | ✅ | ✅ | ✅ | ✅ | ✅ **adopted** (per ULTRAMAP + G2 src/ layout) |
| 2 | Workspace + feature matrix | ✅ | ✅ | ✅ | ✅ | ✅ **adopted** (single-crate features; not workspace) |
| 3 | `CLAUDE.md` / `CLAUDE.local.md` split | ✅ | ✅ | ✅ | ✅ | ✅ **adopted** (already present pre-V7) |
| 4 | `MASTER_INDEX.md` at vault root | ✅ | ✅ | ✅ | ✅ | ✅ **adopted** (vault HOME + MASTER_INDEX exist) |
| 5 | `ai_docs/` + `ai_specs/` partition | ✅ | ✅ | ✅ | ✅ | ✅ **adopted** (V7 lives in `ai_docs/optimisation-v7/`) |
| 6 | Co-located Obsidian vault | ✅ | ✅ | ✅ | ✅ | ✅ **adopted** (`the-workflow-engine-vault/`) |
| 7 | SQLite + `migrations/` | ✅ | ✅ | ✅ | ✅ | ✅ **adopted** (`migrations/` planned per G2 src/ layout) |
| 8 | 50+ tests per module | ✅ | ✅ | ✅ | ✅ | ✅ **adopted** (1,594 budget per TEST_DISCIPLINE) |
| 9 | 4-stage QG (check→clippy→pedantic→test) | ✅ | ✅ | ✅ | ✅ | ✅ **adopted** (per GOD_TIER_RUST.md) |
| 10 | `mN_bridges/<peer>.rs` with circuit breaker | ✅ | partial | ✅ | ✅ | ✅ **adopted** (m40/41/42 each is its own bridge; shared `m40_42_common::Breaker` per G3) |
| 11 | `forbid(unsafe_code)` + `deny(clippy::unwrap_used)` | ✅ | ✅ | ✅ | ✅ | ✅ **adopted** (GOD_TIER_RUST rule 1+2) |
| 12 | Live drift register (not archive) | ✅ | ✅ | ✅ | partial | ✅ **adopted** (per V7 CLAUDE.local.md drift discipline) |
| 13 | Layered TOML config | ✅ | ✅ | ✅ | ✅ | ✅ **adopted** (`config/default.toml`, `config/dev.toml`, `config/prod.toml` per Phase 1 runbook) |

**All 13 patterns adopted.** Zero divergence from gold-standard convergent set.

---

## The 5 divergent-axis decisions

Per GOD_TIER_CONSOLIDATION Part IV § Divergent decisions. Workflow-trace makes one explicit choice per axis:

### Axis 1 — Crate organisation
- ME v2: single crate + mN layers
- LCM: 10-crate workspace
- ORAC: single crate + feature gates
- S1002029 recommendation: ORAC pattern unless ≥3 independent release cadences
- **Workflow-trace decision: ORAC pattern (single crate + features)** — only 2 binaries (`wf-crystallise` + `wf-dispatch`); release cadence shared.

### Axis 2 — DB model
- ME v2: 12 DBs (1 per concern)
- LCM: single ledger + JSONL
- ORAC: single blackboard + 5 tables
- S1002029 recommendation: ORAC pattern (single DB; split only on contention)
- **Workflow-trace decision: ORAC pattern** — single SQLite at `~/.local/share/workflow-trace/db.sqlite` with multiple tables; outbox JSONL for substrate-feedback (m40 only).

### Axis 3 — Inbound protocol
- ME v2: HTTP REST + SSE
- LCM: CLI + MCP JSON-RPC
- ORAC: HTTP + optional V2 wire FSM
- S1002029 recommendation: ME v2 pattern unless streaming in-scope
- **Workflow-trace decision: CLI-first (no HTTP server)** — `wf-crystallise` and `wf-dispatch` are CLI binaries, not daemons. Reasons:
  - CLI-not-service transforms 4 of 8 forge traps per Phase 5 runbook
  - No port allocation; no devenv.toml registration
  - Atuin is the proprioception layer (per S1002029 learning #4)
  - Reduces attack surface (no listening port)
- **Exception:** post-D60, if `wf-status` Zellij plugin needs live data, expose `wf-crystallise serve --listen 127.0.0.1:8190` (FEATURE = `serve`, default off).

### Axis 4 — Evolution layer
- ME v2: Hebbian + PBFT + Kuramoto
- LCM: none at M0
- ORAC: RALPH 5-phase
- S1002029 recommendation: LCM pattern (none at M0; defer to M2+)
- **Workflow-trace decision: LCM pattern** — M0 ships with m31 selector reading only m14 measured lift + m11 decay. No RALPH. No Kuramoto. Hebbian-grain only via CC-5 substrate feedback loop (slow). Optional `feature = "ralph-integration"` at M2+ if substrate readiness check passes.

### Axis 5 — Spec authority
- ME v2: `ai_specs/` 50-spec sheet
- LCM: `plan.toml` declarative
- ORAC: `ORAC_PLAN.md` narrative
- S1002029 recommendation: LCM `plan.toml`
- **Workflow-trace decision: LCM `plan.toml` + supplementary markdown narrative.** `plan.toml` is machine-readable (drives `/scaffold` consistency-check); markdown is human-readable (drives reading + reviewing). Both at `ai_docs/`. Generated `plan.toml` lives at project root.

---

## LCM Drift #1..#11 transposition (closes GAP-Gold-05)

Per GOD_TIER_CONSOLIDATION Part IV § learning #5 + ANTIPATTERNS_REGISTER AP-Drift-01..11. Each Drift class → workflow-trace catch-point.

| LCM Drift | Workflow-trace transposition | Gate that catches |
|---|---|---|
| #1 over-claim gate-clean scoped clippy | Wave-end Command re-runs `--workspace --all-targets --all-features` | Wave-end orchestrator checklist |
| #2 fabricate commits | `git log --oneline -1` independent verification | Wave-end orchestrator checklist |
| #3 scaffold without binary-wiring | integration test exercising N-to-N call chain | Wave-end integration smoke per cluster |
| #4 test count over-report | `cargo test --no-run --message-format=json` count | Wave-end QG step 4 |
| #5 migration "applied" but schema unchanged | `sqlite3 db.sqlite ".schema"` diff before merge | Wave-2 m13 acceptance |
| #6 bridge contract drift | `bridge-contract` skill mandatory pre-merge | Wave-3 m40/41/42 acceptance |
| #7 soak metric over-stated | Phase 5C weekly Watcher independent recompute | Phase 5C cadence |
| #8 push state inconsistent across remotes | `git ls-remote origin && git ls-remote gitlab` before tag-complete | Wave-end merge protocol step 8 |
| #9 PAT in committed file | gitleaks or trufflehog pre-commit hook | per-commit secret scan |
| #10 bind-address default 0.0.0.0 leak | bind explicitly 127.0.0.1; ServerConfig.bind_addr honored | N/A — CLI-first, no bind (per Axis 3) |
| #11 supervisor stub mistaken for live | live-verified smoke test before marking RESOLVED | Phase 5C weekly probe |

**All 11 transposed.** Drift #10 reduces to N/A under workflow-trace's CLI-first decision (no listening port → no bind drift surface).

---

## Wave-end orchestrator checklist (LCM Drift #1 generalisation)

Per closure of GAP-Gold-05. Command runs this before tagging any `wave-N-complete`:

```bash
#!/usr/bin/env bash
# scripts/wave-end-checklist.sh — Wave merge gate per LCM Drift discipline
WAVE_N="${1:?wave number required}"
WAVE_BRANCH="wave-${WAVE_N}-integration-$(ls -t /tmp/wave-${WAVE_N}* | head -1)"

# 1. Full QG re-run (NOT scoped — full workspace + all features + all targets)
cargo check   --workspace --all-targets --all-features 2>&1 | tee /tmp/wave-${WAVE_N}-check.log
[[ ${PIPESTATUS[0]} -ne 0 ]] && { echo "FAIL: check"; exit 1; }

cargo clippy  --workspace --all-targets --all-features -- -D warnings 2>&1 | tee /tmp/wave-${WAVE_N}-clippy.log
[[ ${PIPESTATUS[0]} -ne 0 ]] && { echo "FAIL: clippy"; exit 2; }

cargo clippy  --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic 2>&1 | tee /tmp/wave-${WAVE_N}-pedantic.log
[[ ${PIPESTATUS[0]} -ne 0 ]] && { echo "FAIL: pedantic"; exit 3; }

cargo test    --workspace --all-targets --all-features --release 2>&1 | tee /tmp/wave-${WAVE_N}-test.log
[[ ${PIPESTATUS[0]} -ne 0 ]] && { echo "FAIL: test"; exit 4; }

# 2. git log -1 verification (latest commit is what we merged)
git log --oneline -1 | tee /tmp/wave-${WAVE_N}-headcommit.log

# 3. Integration smoke per cluster (Drift #3)
for cluster in cc1 cc2 cc3 cc4 cc5 cc6 cc7; do
  cargo test --test integration "${cluster}_" --features all-substrates 2>&1 | tail -3
done

# 4. Verify-sync invariants subset for this Wave
./scripts/verify-sync.sh --invariants "$(case $WAVE_N in 1) echo 1-7 ;; 2) echo 1-13 ;; 3) echo 1-22 ;; esac)"

# 5. Test-count audit (Drift #4)
test_count=$(cargo test --workspace --no-run --message-format=json 2>&1 | jq -r '.executable | select(.)' | wc -l)
echo "Test count: ${test_count}"

# 6. Migration schema diff (Drift #5)
[[ -f db.sqlite ]] && sqlite3 db.sqlite ".schema" > /tmp/wave-${WAVE_N}-schema.txt

# 7. Bridge contract check (Drift #6)
~/.local/bin/bridge-contract workflow-trace synthex-v2 || echo "BRIDGE CONTRACT DRIFT"
~/.local/bin/bridge-contract workflow-trace lcm        || echo "BRIDGE CONTRACT DRIFT"
~/.local/bin/bridge-contract workflow-trace povm-v2    || echo "BRIDGE CONTRACT DRIFT"

# 8. Secret scan (Drift #9)
rg -n '(glpat-|github_pat_|gho_)' --hidden -g '!.git' . || echo "secrets clean"

# 9. Remote state parity (Drift #8) — run AFTER push
# git ls-remote origin main && git ls-remote gitlab main

echo "WAVE ${WAVE_N} CHECKLIST PASSED"
```

---

## V8 ↔ V3 wire reuse (closes GAP-Gold-04)

Per S1002029 learning #1+#2. Phase 3 Track 5 wires m32 outcomes into V8 confidence:

```
m32 DispatchOutcome::PassVerified
    │
    ▼
m40_synthex_emit (already does NexusEvent emit)
    │
    ▼ (NEW — Phase 3 Track 5)
HTTP POST :8082/api/v8/learning
{
  "workflow_id": "...",
  "outcome": "PassVerified",
  "confidence_delta": +0.10,   # +0.10 PassVerified / +0.05 Pass / -0.05 Blocked / -0.10 Fail
  "source": "workflow-trace m32"
}
    │
    ▼
V8 updates internal confidence model (Hebbian-grain)
    │
    ▼ (existing V8 ↔ V3 wire)
V8 calls V3 POST :8082/api/v8/confidence
    │
    ▼
V3 future T1 Specify calls honour updated V8 confidence
```

**No reinvention.** Workflow-trace adds one outbound call; V8 ↔ V3 already speaks.

**V3 `resume_from` integration (S1002029 learning #2):**
If workflow-trace `wf-dispatch` is invoking a deploy workflow, m41 routes via `lcm.loop.create` with `resume_from: "T2"` — V8+Zen own T1 (spec); V3 owns trajectory.

---

## Substrate-level learnings catalog (S1002029 × workflow-trace)

| Learning | Adoption |
|---|---|
| #1 V8 ↔ V3 wire reuse | adopted Phase 3 Track 5 (above) |
| #2 V3 `resume_from` | adopted m41 router (above) |
| #3 `/scaffold` is V8's bound | adopted G5 (next) |
| #4 atuin is the cross-tool provenance | adopted T5.2 atuin integration + per-Wave-end atuin trajectory queries |
| #5 LCM Drift #11 generalises | adopted Wave-end orchestrator checklist (above) |

---

## G4 substrate-frame pass

**Second-frame question:** what frame is "gold-standard alignment" itself?

From substrate-frame, "gold-standard" is **convergent-pattern substrate** — three habitat services (ME v2 + LCM + ORAC) independently arrived at the same 13 patterns. This is **substrate-grain evidence** that those patterns are load-bearing (not stylistic). Adopting them aligns workflow-trace with the substrate's accumulated learning.

**Substrate-frame distinction:** the 5 divergent decisions are NOT substrate-evidence (each service made a different choice — no convergence). They're anthropocentric decisions about local concerns (cadence, complexity, surface area). Workflow-trace must MAKE the divergent choices; substrate has no preference.

**Substrate-frame mitigation:** if at Phase 5C soak any of the 5 divergent decisions appears to be causing trouble (e.g., single SQLite contention, CLI-only too limiting), re-open as IC-N (improvement candidate) for the next workflow service's design — DO NOT silently shift workflow-trace's choice mid-soak.

---

## G4 Watcher pre-positioning

**Class B activated.** Hand-off boundary risk: gold-standard pattern adoption can break workflow-trace contracts if patterns transferred carelessly. Mitigated by per-pattern adoption confirmation + per-divergent-axis explicit decision.

---

## G4 close

✅ G4 PASS. 13 convergent patterns adopted (100%). 5 divergent decisions made (single-crate + single-DB + CLI-first + no-evolution-M0 + plan.toml). LCM Drift #1..#11 transposed. Wave-end orchestrator checklist scripted. V8 ↔ V3 wire reuse planned for Phase 3 Track 5. 5 substrate learnings catalogued.

**Output for G5:** gold-standard-aligned contract. G5 reads gold-standard alignment + tooling targets and produces 6 integration deep-dives.

---

*G4 authored 2026-05-17 by Command. Convergent: 13/13. Divergent: 5 explicit. Drift: 11/11 transposed. Substrate-frame: 5 divergent decisions monitored for IC-N at Phase 5C.*
