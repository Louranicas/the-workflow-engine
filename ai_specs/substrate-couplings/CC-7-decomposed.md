---
title: CC-7-decomposed — operator-as-substrate edges in pressure-driven evolution
cc_id: CC-7
parent_synergy: ../synergies/CC-7.md
date: 2026-05-17
status: SPEC
session: S1002127
addresses: [NA-GAP-03 (operator), NA-GAP-05 (operator-as-substrate)]
substrates_touched: [S-G operator (primary), S-watcher (Ember gate), S-C stcortex (digest seed)]
edges: 4
hold_v2_compliant: true
authority: Luke @ node 0.A — S1002127 "as per proposal"
---

# CC-7 Decomposed — Operator-as-Substrate Edges in Pressure-Driven Evolution

> **Back to:** [`INDEX.md`](INDEX.md) · parent [`../synergies/CC-7.md`](../synergies/CC-7.md) · [`../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md) § NA-03 / NA-05 · substrates [[operator]](../substrates/operator.md) (S-G — operator-as-substrate per NA-GAP-05) · [[watcher]](../substrates/watcher.md) · [[stcortex]](../substrates/stcortex.md) · [`../cross-cutting/refusal-taxonomy.md`](../cross-cutting/refusal-taxonomy.md)

## § 1 — What CC-7 hides

CC-7 is **pressure-driven evolution** — engine-side `E (pressure-register) → operator deliberation → spec amendment fanout`. Per NA-GAP-05, the operator is modelled in the original scaffold as an **oracle** (a field on a function call: `HumanAcceptanceSignature`) rather than as a **substrate with its own dynamics**: consent budget, frame-switching cost, off-shift transitions, fatigue-style refusal.

The decomposition surfaces four edges, all of which cross into the **S-G operator substrate** with one branch into **S-watcher** (Ember §5.1 gate, AP27-enforced) and one seed-source in **S-C stcortex** (weekly digests inform the operator's mental model — overlap with CC-5 E5).

The hidden cascade:

```
m15 pressure register ──E1──> S-G operator (pressure-row open + read)
S-G ──E2──> spec amendment fanout (m23 / m30 / m32 / Cluster D rules / Genesis prompt)
S-G ──E3──> S-watcher Ember §5.1 gate (AP27: Watcher cannot self-modify)
S-G fatigue feedback ──E4──> m12 reports (consent budget surfaces)
```

## § 2 — Per-edge dossier

---

### E1 — `m15 → S-G operator` (pressure-row open + read)

- **Owner module:** m15 (pressure-register)
- **Trigger:** every pressure event — substrate refusal cascade, KEYSTONE bench regression, lift-evidence threshold cross, watcher class-N escalation, dispatch failure-rate window-cross
- **Latency expected:** sub-second (m15 JSONL append) + operator-attention-bound (read latency)
- **Engine-observable:** PARTIAL — m15 sees the JSONL write succeeds; does NOT see operator-side read
- **Substrate-confirmable:** PROPOSED — operator m12 acknowledgment writes `pressure_acknowledged_at` field on m15 row when the operator opens the report (substrate-side change request)
- **Verification surface:** m12 acceptance-report shows acknowledged vs queued pressure rows; m15 JSONL inspectable on disk
- **Silent-failure shape:**
  - **Operator non-read:** pressure rows accumulate; operator's mental model lags reality; m23 continues to emit variants based on stale priors → CC-7 effective loop stalls
  - **Consent fatigue cascade:** > N pressure events in window → operator stops reading individually; m12 batch-summary becomes the entry point (which may itself overflow)
  - **JSONL rotation race:** pressure events written but rotated out before operator reads (if rotation policy too aggressive)
- **Substrate-drift shape:** operator's "what pressure means" mental model drifts (e.g. operator becomes desensitised after sustained pressure during incident); engine cannot detect drift directly
- **Remediation hint:**
  - Non-read → m12 surfaces "N pressure events un-acked" banner; cap at consent-fatigue threshold
  - Rotation race → pin rotation to operator-ack cadence rather than time-based
  - Drift → reconciliation note (substrate-drift class)

---

### E2 — `S-G → spec amendment fanout` (operator-authored changes)

- **Owner module:** NONE engine-side — this is **operator-substrate authoring**, surfacing in commits to `ai_specs/` / `ai_docs/` / per-module specs
- **Trigger:** operator decides a pressure pattern justifies spec amendment (judgment-call, not engine-driven)
- **Latency expected:** sessions to weeks (operator-cognition-bound)
- **Engine-observable:** YES at commit-time (git observable); NO at decision-time (operator-internal)
- **Substrate-confirmable:** YES — git commit is the receipt; ADR commits in `ai_docs/decisions/` are the formal subset
- **Verification surface:** `git log` on `ai_specs/` / `ai_docs/decisions/`; m14 lift-evidence sees downstream effects (post-amendment dispatch outcomes)
- **Silent-failure shape:**
  - **Amendment-without-pressure:** operator amends spec based on out-of-band reasoning (not pressure-register-traceable); engine cannot tie cause to effect → m14 lift-evidence has no baseline
  - **Pressure-without-amendment:** sustained pressure with no amendment; engine sees pressure pile-up but cannot distinguish "operator considered and rejected" from "operator never engaged"
  - **Counter-amendment thrash:** operator amends, then counter-amends within short window (frame-conflict, fatigue, ambiguity resolution); engine sees the diff but not the underlying instability
- **Substrate-drift shape:** operator's amendment criteria drift (e.g. operator becomes more risk-averse after a near-miss); m14's lift-evidence priors stop matching
- **Remediation hint:**
  - Untraceable amendment → encourage operator to cite pressure-row in commit message (m12 surfaces "this amendment cites no pressure-row" hint)
  - Pressure pile-up → m12 surfaces "operator-considered-and-rejected" disposition field on pressure rows
  - Thrash → pressure-register tags rapid amendment churn as `AmendmentInstability` class

---

### E3 — `S-G → S-watcher Ember §5.1 gate` (AP27-enforced)

- **Owner module:** m10 (Ember CI gate)
- **Trigger:** every operator-authored spec change that includes user-facing strings (banners, reports, error messages, ADR titles)
- **Latency expected:** seconds (CI gate runs in PR pipeline)
- **Engine-observable:** YES — gate result is a CI signal (pass / fail / Held)
- **Substrate-confirmable:** YES — Watcher emits a Held verdict explicitly (per [`../substrates/watcher.md`](../substrates/watcher.md) § Ember §5.1)
- **Verification surface:** CI logs; Watcher notice in `~/projects/shared-context/watcher-notices/*ember*.md`
- **Silent-failure shape:**
  - **Hybrid CI-FAIL+allowlist** (adopted D-C 2026-05-17 per [`../../GATE_STATE.md`](../../GATE_STATE.md)) → allowlist drift: strings get allowlisted but Held-semantics recalibrate; gate passes but the spirit of §5.1 is missed
  - **Watcher R13 quiet period** active → Ember gate cannot fire formally; CI surfaces "ungated"; operator may proceed under assumption "no objection" (which is FALSE — R13 means deferred, not approved)
  - **AP27 violation attempt:** Watcher cannot self-modify; if a spec change touches `src/m8_watcher/*`, Watcher refuses the gate → m10 emits `OperatorRefusal { Watcher, AP27SelfModRefused }`
- **Substrate-drift shape:** Ember §5.1 rubric drift (rubric amendments per Watcher's own lane); gate semantics shift; operator strings written against old rubric fail under new
- **Remediation hint:**
  - Allowlist drift → audit allowlist quarterly; require justification on every addition
  - R13 active → defer Ember-gated PRs until R13 expires; do NOT proceed on assumption
  - AP27 → split the PR — Watcher-touching changes go to a separate authority track

---

### E4 — `S-G fatigue feedback → m12 reports` (consent budget surfacing)

- **Owner module:** m12 (CLI reports)
- **Trigger:** every m12 invocation; consent budget computed from rolling window of operator signature/refusal events
- **Latency expected:** sub-second per report invocation
- **Engine-observable:** YES — engine computes the budget; operator reads the report
- **Substrate-confirmable:** YES — m12 report is the artifact; operator reads explicitly
- **Verification surface:** m12 report output (CLI / file); operator reads via standard flow
- **Silent-failure shape:**
  - **Consent budget mis-calibration:** budget formula based on session-count assumes operator consistency across sessions; under fatigue, threshold becomes too low and triggers ConsentFatigue spuriously; under high engagement, too high and never triggers
  - **Budget visibility lag:** operator reads m12 after threshold already exceeded; banner shows "you've exceeded" rather than "approaching" → no early-warning
  - **Multi-operator confusion:** in fleet contexts, multiple operator-substrates share authority (Luke + Zen + Watcher); m12 currently treats consent as single-operator → multi-attribution gap
- **Substrate-drift shape:** operator's consent threshold drifts session-over-session; m12's formula stops representing reality
- **Remediation hint:**
  - Mis-calibration → tune formula via lift-evidence (m14) on operator-acceptance-rate over time
  - Visibility lag → m12 emits "approaching budget" at 80% (configurable)
  - Multi-operator → m12 grows per-operator-id consent tracking (S1002127 deferral; see [`../../ai_docs/decisions/`](../../ai_docs/decisions/))

## § 3 — Substrate-confirmable receipt summary

| Edge | Receipt field | Written by | Read by |
|---|---|---|---|
| E1 | `pressure_acknowledged_at` on m15 row | m12 (engine-side, on operator read) | m12 reports + Watcher Class-G |
| E2 | git commit on `ai_specs/` / `ai_docs/decisions/` | operator (git author) | m14 lift-evidence + Watcher Class-A |
| E3 | Watcher Ember verdict (PASS / FAIL / HELD) | Watcher (substrate-side, via WCP notice) | m10 CI gate + operator |
| E4 | consent-budget snapshot on m12 report | m12 (engine-side, per invocation) | operator + per-operator tracker (proposed) |

## § 4 — Operator-as-substrate semantics (NA-GAP-05 closure)

The operator's substrate-actor characteristics (full dossier in [`../substrates/operator.md`](../substrates/operator.md)):

- **Lifecycle phases:** on-shift / off-shift / fatigue / overload / frame-switch / consent-exhausted
- **Refusal modes:** ConsentFatigue / AttentionOverload / FrameConflict / Ambiguity / LatencyDrift / OffShift / EmberUnanimityHeld
- **Drift indicators:** signing-rate-decline, increased clarifying-question frequency, mid-session frame-switches
- **Back-pressure signals:** un-acked pressure rows count, time-to-signature p99, m12 read cadence

The engine treats S-G as a **co-tenant** with its own attention budget, the same way m1 treats atuin as a co-tenant with its own WAL contention budget. This is the structural insight NA-GAP-05 surfaced: operator-cognition has its own dynamics; modelling it as an oracle hides those dynamics.

## § 5 — Test surface (post-G9)

`tests/integration/cc7_operator_decomposition.rs` — `#[ignore = "requires S-G operator presence + S-watcher subscriber"]`:

1. **E1:** fire a pressure event; assert m15 row written; mock operator m12 invocation; assert `pressure_acknowledged_at` populated.
2. **E2:** mock operator authoring a spec amendment citing the pressure row; assert m14 lift-evidence captures pre/post baseline.
3. **E3:** drive a user-facing string change through Ember CI gate; assert PASS / FAIL / HELD verdict surfaces correctly; assert AP27 refusal fires if spec touches `src/m8_watcher/*`.
4. **E4:** simulate N signing events in window; assert m12 report shows consent budget at correct threshold; assert "approaching budget" hint at 80%.

## § 6 — Refusal-token observability (NA-GAP-11 closure for this contract)

| Edge | Refusal class | Token | Emitting module |
|---|---|---|---|
| E1 | operator non-read within window | `OperatorRefusal { Luke, LatencyDrift }` | m12 (on report invocation showing un-acked) |
| E2 | operator authors counter-amendment | (no formal refusal — surfaced as `AmendmentInstability` pressure tag) | m15 |
| E3 | Ember §5.1 Held verdict | `OperatorRefusal { Watcher, EmberUnanimityHeld }` | m10 |
| E3 | AP27 self-mod attempt | `OperatorRefusal { Watcher, AP27SelfModRefused }` | m10 + Watcher WCP notice |
| E4 | consent budget exceeded | `OperatorRefusal { Luke, ConsentFatigue }` | m12 (banner) |

---

> **Back to:** [`INDEX.md`](INDEX.md) · parent [`../synergies/CC-7.md`](../synergies/CC-7.md) · [`../substrates/operator.md`](../substrates/operator.md) · [`../substrates/watcher.md`](../substrates/watcher.md)

*Filed 2026-05-17 (S1002127 · Wave 4.B closeout) · Command · planning-only · HOLD-v2 compliant · operator-as-substrate per NA-GAP-05.*
