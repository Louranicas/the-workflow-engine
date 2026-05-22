# Hardening Fleet — 2026-05-21

> Back to: [[HOME]] · [[MASTER_INDEX]] · [[Assessment Remediation S1003733]] · charter `../CLAUDE.md` · session-state `../CLAUDE.local.md`
> Canonical (ai_docs): `../ai_docs/HARDENING_FLEET_2026-05-21.md`

End-to-end quality + security hardening of `workflow-trace` (26 modules, ~31k LOC),
directed by Luke @ node 0.A, in collaboration with Zen (audit lane). Six waves.

> **Successor note:** the assessment-driven remediation **S1003733** (2026-05-22) followed
> this Hardening Fleet — it closed 21 assessment findings, wired both binaries (C22), and
> took tests 1903 → **1967**. See [[Assessment Remediation S1003733]] for the current reality.

## Waves

| Wave | Outcome | Commit | Tests |
|------|---------|--------|-------|
| W0 | Baseline · atuin security-prompt harvest · plan | — | 1310 baseline |
| W1 | Quality floor — every module to ≥50 meaningful tests | `dc25335` | 1310 → 1782 |
| W2 | 19 security findings — KEYSTONE `project_after_prefix` correctness bug, 9 lock-poison panics, LIKE-injection, error-swallow, m9 namespace boundary, m8 false-gate docstrings, HTTP body caps | `c662b2d`+`5cb4822` | → 1834 |
| W3 | Type-design — `#[non_exhaustive]` ×24, `WorkflowId`+`MinSupport` encapsulation, comment accuracy | `2e3113d` | → 1835 |
| W4 | `cargo-mutants` scoped to m10/m11/m21/m22 + 68 mutant-killing tests. **Verified post-remediation run (S1003733, frozen tree `@0cc7be3`): 324 mutants — 254 caught / 15 missed / 0 timeout / 55 unviable → 94.4% kill rate.** *(The earlier "412 mutants / 80.6%" headline did not reconcile with any committed artifact and was corrected by remediation Wave A.)* | `5de71ac`+`0cc7be3` | → 1921 |
| W5 | Docs reconciliation (charter + both `CLAUDE.local.md`) · 4-surface persistence · commit + push both remotes | `e8f6dd3` | 1903 |

## Headline outcomes

- **KEYSTONE correctness bug fixed** — `m20_prefixspan::project_after_prefix` was a greedy
  single pass that under-counted gap-bounded pattern support; rewritten as a correct
  backtracking matcher with failure-memoisation. The detector now computes true support.
- **0 production panic surfaces** — 9 `Mutex::lock().expect()` poison panics converted to
  `PoisonError::into_inner` recovery; 0 `unwrap`/`panic!`/`unsafe` in hand-written prod code.
- **Security** — LIKE-injection allowlist, HTTP body caps, m9 namespace-boundary leak closed,
  `WorkflowId` m9-gate bypass closed, `cargo-audit` clean.
- **Honesty** — m8 trust-gate docstrings corrected (claimed an implemented gate that never
  existed); the SF1/SF2 recon finding was caught as a false positive by FP-verification.
- Gate green every wave: `check` + `clippy -D warnings` + `clippy -D clippy::pedantic` +
  `cargo test --all-targets --all-features --release`.

## Open for node 0.A — RESOLVED by S1003733 remediation

- **F2** — m8's POVM gate → **KEEP-DORMANT** (remediation Wave D; m8 retained as a dormant
  build.rs tripwire, see `src/m8_povm_build_prereq/mod.rs` module doc).
- **W3 #5–#10** — core-domain-type encapsulation portfolio (`AcceptedWorkflow`, `Pattern`,
  `WorkflowProposal`, `NexusEvent`, opaque IDs, `WorkflowRunRow`) → **completed** in
  remediation Wave C (6 representable-illegal-state holes closed). See
  [[Assessment Remediation S1003733]] § "5 remediation waves" and
  `../ai_docs/HARDENING_W3_TYPE_DESIGN_PORTFOLIO.md`.

Current open follow-ups (post-C22): see [[Assessment Remediation S1003733]] § "Open follow-ups".

## Anchors — all surfaces (bidirectional)

- **Cold-start hub:** `../CLAUDE.local.md` § "HARDENING FLEET" → the COLD-START / RESUME-HERE table.
- **ai_docs (canonical):** `../ai_docs/HARDENING_FLEET_2026-05-21.md` · `../ai_docs/HARDENING_W2_FINDINGS.md` ·
  `../ai_docs/HARDENING_W3_TYPE_DESIGN_PORTFOLIO.md`
- **stcortex:** namespace `workflow_trace_hardening_2026_05_21`, memory id 17939 (meta);
  bidi Hebbian pathway ↔ `workflow_trace_scaffold_s1002127`.
- **POVM** (deprecated mirror — stcortex is canonical): namespace `workflow_trace_hardening_2026_05_21`,
  id `2c8427fa-d87d-432e-9821-c6c7512c4d71`.
- **tracking DB:** `~/.local/share/habitat/injection.db` → `causal_chain` id 113,
  label `workflow_trace_hardening_fleet_2026_05_21`.
- **Zen audit packets:** `~/projects/shared-context/agent-cross-talk/2026-05-21T*_command_zen_review_request_hardening_w[1-4]*.md`
- **git:** 6 commits `dc25335..e8f6dd3` on `main` — pushed origin (GitHub) + gitlab.
