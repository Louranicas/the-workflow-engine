---
title: CC-5-decomposed — substrate-substrate edges hidden in the substrate-learning loop
cc_id: CC-5
parent_synergy: ../synergies/CC-5.md
date: 2026-05-17
status: SPEC
session: S1002127
addresses: [NA-GAP-03, NA-GAP-09]
substrates_touched: [S-C stcortex, S-E synthex, S-B injection.db, S-F lcm, S-G operator (digest), S-watcher (subscriber)]
edges: 5
hold_v2_compliant: true
authority: Luke @ node 0.A — S1002127 "as per proposal"
---

# CC-5 Decomposed — Substrate-Substrate Edges in the Learning Loop

> **Back to:** [`INDEX.md`](INDEX.md) · parent synergy [`../synergies/CC-5.md`](../synergies/CC-5.md) (SPECIAL DEPTH) · [`../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md) § NA-GAP-03 + NA-GAP-09 · substrates [[stcortex]](../substrates/stcortex.md) · [[injection_db]](../substrates/injection_db.md) · [[synthex]](../substrates/synthex.md) · [[lcm]](../substrates/lcm.md) · [[operator]](../substrates/operator.md) · [[watcher]](../substrates/watcher.md) · [`../cross-cutting/refusal-taxonomy.md`](../cross-cutting/refusal-taxonomy.md) · [`../cross-cutting/substrate-drift.md`](../cross-cutting/substrate-drift.md)

## § 1 — What CC-5 hides

The engine-side CC-5 contract is "G → H → back to F via stcortex pathways". The Watcher pre-position is **Class I (Hebbian silence — PRIMARY)**, monitoring the rolling 7-day stcortex pathway-weight delta on `workflow_trace_*` IDs.

That single observation surface (Class I on S-C) hides **five substrate-substrate edges**. CC-5 fails closed-loop ONLY if all five fire correctly; the engine today observes only E1. The remaining four are substrate-internal and require their own observability contracts.

The hidden cascade (Frame A — substrates-as-actors):

```
m32 ──E1──> S-C stcortex                          (Hebbian-grain pathway delta)
m32 ──E2──> S-E synthex ──Hebbian-coord──> S-C    (since S226, S-E owns coordinator)
S-C ──E3──> habitat-memory daemon ──> S-B injection.db   (reinforcement_count++)
m32 ──E4──> S-F LCM ──deploy-loop──> S-V3 deploy-partner  (deploy-shaped only)
S-C ──E5──> weekly-digest ──> S-G operator        (digest cadence; substrate-side)
```

Each edge has its own latency, its own failure modes, its own silent-failure shape, and its own remediation.

## § 2 — Per-edge dossier

The format follows [`INDEX.md`](INDEX.md) § Verification discipline. Each edge is independently observable and independently broken.

---

### E1 — `m32 → S-C stcortex` (pathway-weight delta — PRIMARY)

- **Owner module:** m42 (via m13 stcortex writer)
- **Trigger:** every Cluster H emit (one per dispatched workflow with known outcome)
- **Latency expected:** sub-second (synchronous reducer call, idempotent within 1h window)
- **Engine-observable:** PARTIAL — m42 sees the receipt confirming ingest; does NOT see pathway-propagation (which is substrate-internal Hebbian math)
- **Substrate-confirmable:** PROPOSED — stcortex writes `cc5_closed_at` field when it observes N+1-dispatch reinforce on the same pathway (substrate-side change request in [`../../ai_docs/decisions/`](../../ai_docs/decisions/))
- **Verification surface:** Watcher Class-I rolling 7-day delta on `workflow_trace_*` pathway weights (m42 §11 closure-test surrogate)
- **Silent-failure shape:**
  - stcortex returns HTTP 200 + reducer succeeds, but pathway-write hits a no-op branch (e.g. `confidence == 0` short-circuit, or `co_activation_pair` existence filter from CR-2b mis-rejecting the write) → S-C accepts but pathway-weight never moves
  - `RefuseWriteNoConsumer` returned because Claude session forgot `register_consumer` step (per workspace [`CLAUDE.md`](../../CLAUDE.md) memory row 8); engine surfaces `SubstrateUnavailable` (wrong category — should be `SubstrateAuthored::RefuseWriteNoConsumer` per [`../cross-cutting/refusal-taxonomy.md`](../cross-cutting/refusal-taxonomy.md))
- **Substrate-drift shape:** CR-2 inflation (POVM-analogue) — substrate semantic-change recalibrates the threshold definition; engine continues to emit but Class-I window definition stops representing the metric. Per [`../cross-cutting/substrate-drift.md`](../cross-cutting/substrate-drift.md) § canary contract.
- **Remediation hint:**
  - `RefuseWriteNoConsumer` → per-session `call register_consumer` (NOT a service restart)
  - Class-I silence > 4 weeks → enable per-edge probe (read stcortex `:3000` pathway table directly, NOT engine bridge) to falsify Class-I
  - CR-2 style drift → reconciliation note (see Hebbian v3 reconciliation pattern)

---

### E2 — `m32 → S-E synthex → S-C stcortex` (Hebbian coordinator path, S226+)

- **Owner module:** m40 (NexusEvent emit to `:8092/v3/nexus/push`)
- **Trigger:** every m32 dispatch with a workflow_id
- **Latency expected:** sub-second push; coordinator action is async over minutes (SYNTHEX v2 Hebbian-coord computes pathway deltas based on workflow correlations across all consumers, not just workflow-trace)
- **Engine-observable:** NO — engine emits the NexusEvent; the coordinator action on S-E is internal to SYNTHEX v2's m8_watcher loop (per [`../substrates/synthex.md`](../substrates/synthex.md) § lifecycle). The S-E → S-C side-channel is substrate-internal.
- **Substrate-confirmable:** SYNTHEX v2 emits `WireEvent::CoordinatorApplied { workflow_id, delta_applied }` if subscribed; engine does NOT subscribe today (Wave 4 entry)
- **Verification surface:** Watcher Class-I must subscribe to BOTH S-C AND S-E coordinator events to disambiguate E1-direct from E2-indirect
- **Silent-failure shape:**
  - SYNTHEX v2 R13 quiet period active → m40 push accepted but coordinator deferred until R13 expires (per [`../substrates/synthex.md`](../substrates/synthex.md) § R13)
  - SYNTHEX v2 dispatch backlog > threshold → coordinator queued indefinitely; engine never sees it didn't run
  - SYNTHEX v2 schema rejection (E.g. `WorkflowEvent` payload version skew) → coordinator silently drops events; m40 sees ACK but coordinator state never updates
- **Substrate-drift shape:** S-E's Hebbian-coord formula changes between versions (analogous to CR-2 on S-C); m40 emits the same payload but coordinator outcomes shift unobservably
- **Remediation hint:**
  - R13 active → wait for window expiry (`watcher status` shows remaining time)
  - Schema rejection → bump version constants in m40 (and write a `bridge-contract` skill F-Contract test)
  - Coordinator drift → reconcile via paired S-E + S-C reading (cross-substrate delta should match the formula)

---

### E3 — `S-C stcortex → habitat-memory daemon → S-B injection.db` (next-session pre-warm)

- **Owner module:** NONE engine-side — this edge is **substrate-mediated entirely**
- **Trigger:** stcortex pathway-weight delta crosses a threshold the habitat-memory daemon polls (currently ~1 minute cadence per habitat-memory daemon configuration)
- **Latency expected:** minutes to hours (daemon poll cadence + injection.db write latency)
- **Engine-observable:** NO — engine does not control the daemon, does not directly observe S-B writes, and habitat-memory daemon is shared across all habitat services
- **Substrate-confirmable:** PROPOSED — habitat-memory daemon writes `cc5_propagation_observed_at` on injection.db row when it reinforces a `workflow_trace_*` chain from an stcortex pathway-weight delta (substrate-side change request)
- **Verification surface:** next-session m3 read sees `injection.db.causal_chain.reinforcement_count` incremented OR `last_seen_session` advanced for the `workflow_trace_*` chain. m3 emits a structured trace event capturing this.
- **Silent-failure shape:**
  - **TTL sweep race (the canonical case):** stcortex pathway-weight delta lands on S-C at session-end; before the next session opens, habitat-memory daemon's hourly TTL sweep deletes the reinforcement row because the `last_seen_session` timestamp was 0..5 (per [`feedback_ttl_sweep_test_timestamps.md`](../../../../.claude/projects/-home-louranicas-claude-code-workspace/memory/feedback_ttl_sweep_test_timestamps.md) — S117 crystallisation pattern). Engine sees Class-I fire correctly on S-C delta, but S-B never pre-warms; next session m3 sees no row; CC-5 closed-loop never closes.
  - habitat-memory daemon crashed (no PID file) → engine has no signal until next session attempts m3 read and finds stale row
  - injection.db schema migration between versions (column added/renamed) → daemon writes but m3 reader fails to deserialize; engine sees the row but it's silently dropped
- **Substrate-drift shape:** habitat-memory daemon's polling threshold definition changes (e.g. delta required for reinforce); engine's downstream observations stop matching what S-B holds
- **Remediation hint:**
  - TTL sweep race → ensure timestamps use `now_ms() + i` discipline (m42 already enforces this for idempotency cache; the cross-substrate analogue requires habitat-memory daemon to do the same)
  - Daemon crash → restart habitat-memory daemon AND replay the missed sweeps via daemon `--replay-from-snapshot`
  - Schema migration → version-pin in m3 + `bridge-contract` F-Contract test on injection.db schema

---

### E4 — `m32 → S-F LCM → S-V3 deploy-partner` (deploy-shaped only)

- **Owner module:** m41 (LCM `lcm.loop.create` RPC, NOT `lcm.deploy`)
- **Trigger:** dispatched workflows where step kind is deploy-shaped (a subset of all dispatches, possibly zero for non-deploy workflows)
- **Latency expected:** seconds (LCM RPC) + minutes (LCM supervisor → V3 partner deploy)
- **Engine-observable:** PARTIAL — m41 sees the LCM RPC receipt (`loop_id` returned). LCM's downstream V3 partner deploy is not engine-visible.
- **Substrate-confirmable:** LCM supervisor writes deploy completion events; engine does NOT subscribe today. V3 partner emits no direct CC-5-tagged event.
- **Verification surface:**
  - LCM supervisor logs (`~/.local/share/lcm/supervisor.log` if available) for loop_id state transitions
  - V3 dev-ops-engine-v3 `/health` for partner readiness
  - injection.db chain `lcm_deploy_observed` if habitat-memory daemon tags LCM deploys (proposed)
- **Silent-failure shape:**
  - LCM supervisor not live (M0_VERIFIED tag absent on master) → m41 receives `SupervisorNotLive` typed error per [`../cross-cutting/refusal-taxonomy.md`](../cross-cutting/refusal-taxonomy.md); engine surfaces `SubstrateAuthored::SupervisorNotLive`
  - LCM `deploy_cancel_handler` returns pending → V3 partner deploy never starts; m41 has no signal
  - V3 partner port (8082) bound but not accepting → LCM supervisor's health probe shows "stuck"; m41 sees `loop_id` but downstream never advances
- **Substrate-drift shape:** LCM RPC method signature change (e.g. `lcm.loop.create` → `lcm.loops.create`) — schema-rejection class
- **Remediation hint:**
  - `SupervisorNotLive` → check `lcm-supervisor` PID at `~/.local/bin/lcm-supervisor` (per [`../substrates/lcm.md`](../substrates/lcm.md))
  - Deploy stuck → V3 partner health-200 vs bind audit; if "stuck" (per WEB v4 L7 layer) → restart V3 service
  - Schema change → bump LCM method version in m41 + `bridge-contract` test

---

### E5 — `S-C stcortex → weekly-digest → S-G operator` (digest cadence — substrate-side)

- **Owner module:** NONE engine-side — this edge is **substrate + operator coupling, no engine code path**
- **Trigger:** stcortex's own weekly digest cadence (substrate-internal cron / scheduler; currently ~weekly per habitat convention)
- **Latency expected:** week-scale; the operator reads the digest at their own cadence
- **Engine-observable:** NO — digest emission is substrate-internal; the engine does not control it and does not observe its fire
- **Substrate-confirmable:** PROPOSED — stcortex emits `digest_fired_at` event into the substrate-event log AND writes a digest summary memory under `weekly_digest_*` namespace
- **Verification surface:**
  - watcher-notices file drops (`~/projects/shared-context/watcher-notices/*digest*.md`) when Watcher subscribes to the digest channel and produces a notice
  - m12 operator-reports include a "weeks-since-last-digest-read" metric if the operator-as-substrate dossier captures this signal (see [`../substrates/operator.md`](../substrates/operator.md))
- **Silent-failure shape:**
  - **Digest fired but operator did not read it:** operator-substrate consent-fatigue or off-shift state; m23's next selection cycle continues but with stale operator priors → CC-5 effective loop closes engine-side but never feeds operator priors → m23's substrate-shaping never reaches the operator-feedback loop in CC-7
  - **Digest scheduler crashed:** stcortex's internal scheduler is substrate-internal; no `digest_fired_at` event appears in event log → engine has no signal
  - **Digest content drift:** digest schema changes between versions (e.g. new fields added); operator sees old format from memory and skips reading → soft drift
- **Substrate-drift shape:** S-C's digest formula recalibrates (CR-2-analogue); operator's mental model of "what high pathway weight means" stops matching reality
- **Remediation hint:**
  - Operator non-read → m12 surfaces "you haven't read the digest in N weeks" banner; ConsentFatigue token if N > 4
  - Scheduler crash → check stcortex MCP `stcortex_status` for digest-scheduler health
  - Schema drift → operator reconciliation note (cite as substrate-drift class per [`../cross-cutting/substrate-drift.md`](../cross-cutting/substrate-drift.md))

## § 3 — Substrate-confirmable receipt summary (NA-GAP-09)

Five proposed receipts (one per edge); none exist today; each is a substrate-side change request to be tracked in [`../../ai_docs/decisions/`](../../ai_docs/decisions/):

| Edge | Receipt field | Written by | Read by |
|---|---|---|---|
| E1 | `cc5_closed_at` on stcortex pathway | stcortex reducer (substrate-side) | Watcher Class-I confirmation; m12 reports |
| E2 | `WireEvent::CoordinatorApplied` | SYNTHEX v2 m8_watcher coordinator | engine subscriber (proposed) |
| E3 | `cc5_propagation_observed_at` on injection.db row | habitat-memory daemon | m3 reader (engine) |
| E4 | LCM supervisor loop-state event | LCM supervisor | engine subscriber (proposed) + V3 health probe |
| E5 | `digest_fired_at` event + memory | stcortex internal scheduler | Watcher subscriber + m12 operator-reports |

The pattern across all five: **substrate writes the confirmation; engine reads it**. This inverts the engine-as-authority assumption that NA-GAP-09 surfaced and is a structural shift, not a code change. Pre-G9, these are planning-only proposals; post-G9, they become integration-test prerequisites.

## § 4 — Class-I supplementation (NA-GAP-09)

Watcher Class-I today fires on absence of stcortex pathway-weight delta. The decomposed view shows Class-I is **necessary but not sufficient** — it observes E1 only.

Proposed Class-I supplementation (deferred to v0.2.0 ADR per [`../../ai_docs/decisions/2026-05-17-substrate-as-actor-deferrals.md`](../../ai_docs/decisions/2026-05-17-substrate-as-actor-deferrals.md)):

- **Class-I-E2** — fires on E2 silence (S-E coordinator never confirms within window)
- **Class-I-E3** — fires on E3 race-loss (injection.db row TTL-swept before pre-warm)
- **Class-I-E4** — fires on E4 deploy-stuck (LCM loop opened but V3 partner never advances)
- **Class-I-E5** — fires on E5 digest-skipped (operator non-read > 4 weeks)

Each is independently configurable; the canonical Class-I remains the umbrella.

## § 5 — Test surface (post-G9)

`tests/integration/cc5_substrate_decomposition.rs` — `#[ignore = "requires live S-C + S-E + S-F + habitat-memory daemon + operator presence"]`:

1. Dispatch a known test workflow via m32; assert `DispatchOutcome::PassVerified`.
2. **E1 assert:** `stcortex_query_pathway_weight_delta(workflow_trace_*, window=2s) > 0`.
3. **E2 assert** (if synthex subscriber wired): `WireEvent::CoordinatorApplied` emitted within window.
4. **E3 assert:** wait N seconds for habitat-memory daemon poll; assert `injection_db.causal_chain` row exists with matching `workflow_trace_*` chain id; assert `reinforcement_count > 0`.
5. **E4 assert** (deploy-shaped workflows only): LCM `loop_id` reported by m41; assert LCM supervisor logs show `loop_state: created`.
6. **E5 assert** (week-scale): NOT runnable inline; must use captured fixture from weekly digest log.

Test enables / disables per assertion based on substrate availability — partial pass acceptable (e.g. E1+E3+E4 if no LCM supervisor or operator).

## § 6 — Refusal-token observability (NA-GAP-11 closure for this contract)

When CC-5 fan-out encounters a refusal at any edge, the refusing module emits `WireEvent::Refusal { token: ... }` per [`../cross-cutting/refusal-taxonomy.md`](../cross-cutting/refusal-taxonomy.md):

| Edge | Refusal class | Token | Emitting module |
|---|---|---|---|
| E1 | `S-C stcortex RefuseWriteNoConsumer` | `SubstrateAuthored { S-C, RefuseWriteNoConsumer }` | m42 |
| E1 | `S-C stcortex InvalidSlug` | `SubstrateAuthored { S-C, InvalidSlug }` | m42 (or m9 pre-empt) |
| E2 | `S-E synthex R13QuietPeriod` | `SubstrateAuthored { S-E, R13QuietPeriod }` | m40 |
| E2 | `S-E synthex SchemaRejected` | `SubstrateAuthored { S-E, SchemaRejected }` | m40 |
| E3 | (no direct refusal — daemon poll race; surface via Class-I-E3) | — | — |
| E4 | `S-F lcm SupervisorNotLive` | `SubstrateAuthored { S-F, SupervisorNotLive }` | m41 |
| E5 | (no direct refusal — substrate-internal scheduler; surface via Class-I-E5) | — | — |
| any | `m32 5-check failure` | `EngineAuthored { invariant: <check>, refusal_class: <class> }` | m32 (then m40 emit) |

Edges E3 and E5 are **substrate-mediated** with no direct engine refusal path; their failure modes surface as silence rather than typed refusal. This is the structural asymmetry NA-GAP-11 identified for habitat-memory and digest scheduler.

## § 7 — Connection to substrate-drift (NA-GAP-07 closure for this contract)

Each edge participates in the canary contract defined in [`../cross-cutting/substrate-drift.md`](../cross-cutting/substrate-drift.md). The canary kinds applicable here:

- **E1:** stcortex pathway-weight canary (substrate-CR-2 inflation-style detection)
- **E2:** S-E coordinator-formula canary (parallel-version reading)
- **E3:** habitat-memory daemon poll-cadence canary (timing-based)
- **E4:** LCM RPC method-signature canary (`bridge-contract` style)
- **E5:** digest-schema canary (memory-content matching)

Substrate-drift detector fires `SubstrateDriftDetected` events on canary mismatch; CC-5 verification must be considered **suspect-until-canary-confirms** during drift episodes.

---

> **Back to:** [`INDEX.md`](INDEX.md) · parent [`../synergies/CC-5.md`](../synergies/CC-5.md) · [`../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md) § NA-03 / NA-09 · canonical [`../../ai_docs/optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md`](../../ai_docs/optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md)

*Filed 2026-05-17 (S1002127 · Wave 4.B closeout) · Command · planning-only · HOLD-v2 compliant · five substrate-side change requests pending.*
