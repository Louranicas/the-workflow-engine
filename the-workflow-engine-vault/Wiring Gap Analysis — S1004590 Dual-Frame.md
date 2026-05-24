> Back to: [[SYNTHEX-V2 Integration Master Schematic]] · [[HOME]] · [[MASTER_INDEX]] · `~/claude-code-workspace/the-workflow-engine/CLAUDE.local.md`
> Source reports: `/tmp/synthex-v2-deep-evidence-for-gap-analysis.md` (conventional, 324 lines) · `/tmp/synthex-v2-wiring-plan-na-gap.md` (NA pass, 305 lines)
> Discipline: Plan v2 §6 dual-frame pattern (write once, ask *what frame is that?*, write again from the frame not taken)

# Wiring Gap Analysis — S1004590 (Dual-Frame)

> **Verdict: AMEND** — the 5 wiring schematics ([[SYNTHEX-V2 Integration Master Schematic]] + [[Wiring 01 — m16 Heartbeat Consumer (NA-4 Closure)]] through [[Wiring 04 — Watcher (m46–m51) Integration Hooks]]) are an **honest first pass**, not plan-ratified. Conventional pass finds 1 CRITICAL drift + 2 speculative claims. NA pass finds 4 HIGH-severity frame-collapse findings + 3 load-bearing tensions. Wiring 03 (stcortex namespace) lands as authored; the other four need amendment before ratification.

## TL;DR — what each schematic actually is

| Schematic | Authored as | Reality (post-evidence) | Action |
|---|---|---|---|
| Master | "umbrella + 4 wiring schematics" | umbrella + 4 wiring schematics + **substrate-generalisation gap NA-4'** + 7-substrate generalisability audit table missing | AMEND — add §"7-substrate generalisability audit" |
| Wiring 01 | "NA-4 closure via m10_http_poller landing" | landing surface correct, **but NA-4 NOT actually closed** — m46's `signal_bus_recent` cap of 50 evicts heartbeats before m47 Critic detects drift pattern | AMEND — add `HeartbeatBuffer` ring-buffer requirement (~600 entries, 10min @ 1Hz); add bilateral V5 (NA-3'); add WFE-down liveness contract (NA-8') |
| Wiring 02 | "NexusEvent Bidirectional Push — receiver LIVE" | **`/v3/nexus/push` is a 501 STUB** (commit `c9eeb75` 2026-05-24, W1-005 Bug Hunt Armada hardening); **AND** "bidirectional" is unidirectional (body line 144-150 defers SX2→WFE to T2+) | RENAME to "Wiring 02 — NexusEvent Outbound" + author NEW [[Wiring 02b — NexusEvent Inbound (SX2 → WFE)]] |
| Wiring 03 | "stcortex namespace alignment, P30 enforcement" | **VERIFIED CORRECT** — m35d_povm_bridge enforces compile-time + serde-layer validation per AP09 | LANDS AS AUTHORED (minor: NA-9' undocumented `parent_namespace` field) |
| Wiring 04 | "Watcher integration hooks — substrate-side actor flow" | Watcher persona + m49 PBFT-eligibility verified, **but plan body line 120 admits proposals "land on Luke, not WFE engine"** — agency is over SX2's own state, not WFE's | RENAME to "Wiring 04 — Watcher Observation of WFE Heartbeats (read-only, no WFE feedback loop)" |

## Methodology

Two parallel sub-agents dispatched 2026-05-24 (S1004590):

1. **Explore agent (very thorough)** — read-only ground-truth gathering: 324-line evidence report verifying every plan claim against synthex-v2 source (src/daemon/http.rs, src/m6_action/m35d_povm_bridge.rs, src/m8_watcher/, src/m2_ingest/m10_http_poller.rs) + vault notes (MASTER_INDEX, CLAUDE.local.md, S116/S117 specs).
2. **na-gap-analyst** — frame-classification pass: catalogues Frame A vs Frame B assumptions in each schematic, surfaces second-frame gaps (NA-1' through NA-10'), identifies 3 load-bearing tensions + 2 convergent findings.

**Confidence:** evidence-side 0.95 (file:line citations); NA-side 0.80 (NA-1' is 0.85-conditional on direct read of `m46_observer.rs` to confirm `signal_bus_recent` cap — currently inferred from architectural docs).

---

## Part A — Conventional gap analysis (evidence-based)

### A.1 Verified claims (6 hold as authored)

| Claim | Status | Evidence (file:line) |
|---|---|---|
| `:8092/v3/heartbeat` does NOT exist | ✅ VERIFIED | `synthex-v2/src/daemon/http.rs:399-411` route table — confirmed absence |
| `m10_http_poller` is L2 ingest tier, recommended landing | ✅ VERIFIED | `synthex-v2/src/m2_ingest/m10_http_poller.rs` module + doc comments |
| R13 cold-start arm elapsed 2026-05-19; m49 PBFT-eligible | ✅ VERIFIED | `synthex-v2/CLAUDE.local.md` § "The Watcher ☤" — 663,619+ obs ≥ 100 threshold |
| P30 namespace enforcement in m35d_povm_bridge | ✅ VERIFIED | `synthex-v2/src/m6_action/m35d_povm_bridge.rs:334-839` — newtype `PovmPathwayId` + serde rejection + test `payload_pathway_id_must_be_namespaced` (line 352) |
| m46–m51 Watcher modules ACTIVE/library-ready, AP27 enforced | ✅ VERIFIED | `synthex-v2/CLAUDE.local.md` § Watcher — m46 ACTIVE, m51 ACTIVE feature-gated, m47/48/50 library-ready |
| WsMessage 14-variant wire contract canonical | ✅ VERIFIED (reference) | S117 spec doc; full enum not re-read but referenced canonically |

### A.2 Drift findings (CRITICAL — plan claims false)

#### CONV-1 [CRITICAL] — `/v3/nexus/push` is a 501 STUB, not LIVE

**Plan claim:** Wiring 02 states "/v3/nexus/push is LIVE with routing m13→m29→m31/m33/m35g/m45".

**Evidence:** `synthex-v2/src/daemon/http.rs:253-267`:
```rust
async fn nexus_push_handler(
    axum::extract::Json(_body): axum::extract::Json<serde_json::Value>,
) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED,
     Json(NotImplementedBody {
         error: "not_implemented",
         endpoint: "/v3/nexus/push",
         detail: "stub handler — wiring lands in Phase 3 (m35g_orac_bridge)",
     }))
}
```

**Commit history:** `c9eeb75` (2026-05-24, Bug Hunt Armada W1-005 HIGH `dead_auth`): "stub handlers now return 501 Not Implemented (was lying-200)". The plan was authored before this hardening landed; pre-c9eeb75 the endpoint was lying-200 (AP01 silent-swallow).

**Severity:** **CRITICAL.** Wiring 02's headline contract is false. Workflow-trace client code is forward-compatible (transport will succeed once synthex-v2 Phase 3 lands), but the plan misleads any reader who treats it as a wire-ready surface.

**Remediation:**
1. Wiring 02 § "Current state" — replace "synthex-v2 side (LIVE)" with "synthex-v2 side (501 STUB; Phase 3 pending per commit `c9eeb75`)".
2. Add new "Pre-flight" section naming the synthex-v2 Phase 3 landing as a hard precondition.
3. Workflow-trace transport client MUST detect 501 and route through `RefusalToken::Unavailable(SubstrateUnreachable { transport_reason: "synthex_v2_phase3_pending" })` — not retry indefinitely.

#### CONV-2 [HIGH] — m13→m29→4-subscriber routing chain is INTERNAL only

**Plan claim:** Wiring 02 § 4.3 shows the routing chain as if HTTP request triggers downstream subscribers.

**Evidence:** `m13_ingest_router.rs` and `m29_policy_router.rs` modules exist (filesystem confirmed) but are **internal signal processors**, NOT wired to the HTTP POST handler. The HTTP layer is the stub from CONV-1; the routers run on internal signal bus events only.

**Severity:** HIGH — promotes architectural fiction. The routers exist; the wire-up does not.

**Remediation:** Wiring 02 § 4.3 — distinguish "current internal flow (working)" from "future HTTP-triggered flow (Phase 3 pending)".

### A.3 Speculative claims (cannot verify without further reads)

#### CONV-3 [SPECULATIVE] — `WorkflowTraceHeartbeat` wire schema match

**Plan claim:** Wiring 01 § 3.2 specifies `WorkflowTraceHeartbeat { sequence_number, timestamp_ms, skew_envelope, alert_budget, heartbeat_source }`.

**Evidence:** Plan specifies JSON schema; the agent did NOT read `the-workflow-engine/src/m16_substrate_drift_canary/mod.rs` to verify the actual emitted type matches byte-for-byte.

**Severity:** SPECULATIVE — schema may drift from what m16 actually emits.

**Remediation:** Before any synthex-v2-side endpoint code is authored, read `src/m16_substrate_drift_canary/mod.rs` and reconcile the wire schema against the actual `Heartbeat` struct definition. Update Wiring 01 § 3.2 with the verified shape.

#### CONV-4 [SPECULATIVE] — 10 `workflow_trace_*` slug families

**Plan claim:** Wiring 03 catalogues 10 slug families emitted by m42.

**Evidence:** Plan lists 4 naming patterns + 10 example families. The actual `src/m42_stcortex_emit/mod.rs` emission roster was not verified.

**Severity:** SPECULATIVE.

**Remediation:** Read `src/m42_stcortex_emit/mod.rs` + tests; reconcile the slug catalogue against actual emission sites; mark each as IMPLEMENTED / SPECIFIED / DEFERRED.

### A.4 Missed surfaces (plan should mention but doesn't)

| Surface | Path | Plan coverage gap |
|---|---|---|
| `/api/ingest` POST (currently 501 stub) | `synthex-v2/src/daemon/http.rs` | Alternative m16 landing if ORAC's `CascadeHeartbeat` shape fits — Wiring 01 should evaluate or rule out |
| `/v3/nexus/pull` GET (501) | same | Reverse-direction NexusEvent could be SX2→WFE channel — Wiring 02b should consider |
| `/v3/decay/trigger` POST (501) | same | STDP decay invocation — out of scope but worth naming |
| `/notify/rm_entry` POST (LIVE) | Nerve Center bridge | WFE could write RM entries directly bypassing m42 — out of scope but worth naming as a rejected alternative |
| `/ws/orac` WebSocket (LIVE) | S116 stream B | Already noted in Wiring 02 as deferred; should be cross-linked in Master schematic |

### A.5 Resilience policy gaps (plan silent)

| Gap | Plan coverage | Recommendation |
|---|---|---|
| Retry backoff strategy | none | Specify exponential (100ms → 500ms → 2s) cap at 5 retries |
| Circuit-breaker thresholds | none | 5 consecutive 503s → trip; 30s cooldown |
| Max heartbeat age tolerance | none | >500ms latency = degradation signal; >2s = circuit-trip |
| Watcher $50/day Opus budget (AP30) | implicit only | `AlertBudget` engine-side rate-limit must be documented as the upstream mitigation |

---

## Part B — Non-anthropocentric gap analysis (frame-collapse pass)

**Dominant frame across all 5 schematics: A (engineer-as-actor / substrate-as-tool).** Frame B vocabulary (NA-5, RefusalToken, SubstrateAuthored) is inherited from Plan v2 but applied **asymmetrically** — only on the WFE→SX2 leg. Wiring 03 is the strongest substrate-frame work and lands largely as authored. The other four read as RPC-and-LOC engineering.

### B.1 HIGH-severity findings

#### NA-1' [SUBSTRATE-FRAME · HIGH] — NA-4 is not closed, it is displaced one layer deeper

**Plan claim:** Wiring 01 closes NA-4 by landing heartbeats at m10 → m13 → SignalBus → m46.

**Substrate-frame reality:** m46 reads `signal_bus_recent: Vec<Signal>` capped at **50 events** (per Wiring 04 line 36-37 ObservableState). At sustained 1 Hz from m16, with competing signals from m29/m31/m33/m34/5 heat sources/6 daemon tasks, m16 heartbeats **evict from the buffer before m46 can detect the "3+ events in last 10 ticks" drift pattern** (per Wiring 04 line 47 m46 trigger condition).

**The wire exists; the consumption shape does not match.** NA-4 is not closed — the silence migrates from m16's outbound channel to m46's signal_bus_recent eviction frontier. The substrate frame sees this: the substrate cannot reason about a signal it never observes.

**Remediation:** Wiring 01 must specify a dedicated `HeartbeatBuffer` (ring buffer ≥600 entries, 10 min at 1 Hz) inside m46 OR a downsampler at m13 that aggregates heartbeats before they reach signal_bus_recent. Acceptance test must demonstrate accumulation under realistic load (concurrent 5-heat-source emissions for 60s).

#### NA-2' [SUBSTRATE-FRAME · HIGH] — "Bidirectional Push" is unidirectional; title hides it

**Plan claim:** Wiring 02 title says "Bidirectional Push".

**Substrate-frame reality:** Body lines 144-150 defer SX2→WFE direction to T2+ (post-S117). The actual wire is **unidirectional outbound from WFE only**. From the substrate's frame, "bidirectional" implies a reciprocal channel — but the substrate cannot push events to WFE today (no `m43_synthex_v2_inbound` module in WFE).

**Remediation:** Rename to "Wiring 02 — NexusEvent Outbound (WFE → SX2)" and author NEW sibling [[Wiring 02b — NexusEvent Inbound (SX2 → WFE)]] specifying the m43 WFE-side consumer module needed.

#### NA-3' [SUBSTRATE-FRAME · HIGH] — No `WorkflowTraceParticipationStatus`; V5 is asymmetric

**Plan claim:** Wiring 01 uses WFE's V5 `SubstrateTrust::is_substrate_imagined_for(SubstrateId::SynthexV2)` to gate emission.

**Substrate-frame reality:** V5 is **engine-only**. The substrate has no equivalent `WorkflowTraceParticipationStatus` per WFE module — m46 cannot distinguish "WFE drained gracefully" from "WFE crashed mid-burst" from "WFE never started". When m16 heartbeats stop, the substrate has no NA-5 audit-distinguishability for the reason.

**Remediation:** Add `WorkflowTraceParticipationStatus { NotShipped, Shipping, Live }` to synthex-v2-side (parity with V5). ~40 LOC. The substrate uses this to refuse-distinguishably when WFE silence persists.

#### NA-4' [SUBSTRATE-FRAME · HIGH] — Single-substrate completion theatre; pattern doesn't generalise

**Plan claim:** Master schematic + Wiring 01-04 focus on synthex-v2; implicit assumption that the pattern is repeatable across the 7 NA-2 substrates.

**Substrate-frame reality:** The 7 NA-2 substrates (atuin · stcortex · HABITAT-CONDUCTOR · CC-5 loop clocks · Luke + Watcher · RALPH · Cargo build graph) have **radically different consumption surfaces**:
- **atuin** is a shell-history store, not a daemon — no HTTP endpoint, no signal bus; heartbeat envelope is a category error
- **HABITAT-CONDUCTOR** is per-pane dispatch coordination — uses Zellij IPC + file-drop, not REST
- **Luke + Watcher** is human + persona pair — no machine consumer at all
- **RALPH** is in-process fitness loop on ORAC — internal, no external surface
- **Cargo build graph** is the build-time substrate — observable via `cargo metadata` not runtime HTTP

The synthex-v2 pattern (REST POST + m46 Observer) is NOT generalisable. Master schematic implies generalisability without auditing this.

**Remediation:** Master schematic must add §"7-substrate generalisability audit" — for each NA-2 substrate, name: (a) what consumption surface exists, (b) whether m16 heartbeat shape fits, (c) what would need to be authored to wire it, (d) honest residual if no integration path. Author this audit BEFORE Wiring 01 ships per-substrate.

### B.2 MED-severity findings

#### NA-5' [SUBSTRATE-FRAME · MED] — Watcher integration claims agency that doesn't reach WFE

**Plan claim:** Wiring 04 frames the Watcher as a feedback loop ("substrate-side actor flow").

**Substrate-frame reality:** Wiring 04 line 120 admits proposals "land on Luke, not WFE engine". The Watcher's agency is over **synthex-v2's own internal state** (PID setpoint, heat source weighting) — not WFE's. Even if m46 observes WFE drift and m49 proposes a correction, the correction is to SX2's PID, not WFE's m16 emission rate or m11 decay weight. There is no feedback to WFE.

**Remediation:** Rename Wiring 04 to "Wiring 04 — Watcher Observation of WFE Heartbeats (read-only, no WFE feedback loop)". Add explicit § "What this does NOT do" naming the absent feedback path. If true feedback is wanted, design `m44_watcher_pathway_crawl` in WFE (Wiring 04 § honest residual already names this — promote to first-class follow-up).

#### NA-6' [SUBSTRATE-FRAME · MED] — Cross-habitat reciprocity invoked but not honoured

**Plan claim:** ADR D-S1004XXX-05 cited multiple times as "coordinated cross-habitat ship".

**Substrate-frame reality:** All 5 schematics are **unilateral prescription from WFE → SX2**, not reciprocal. ADR D-S1004XXX-05 specifies a cross-habitat substrate-mediated trust pattern requiring SX2-side schemas. The schematics specify what SX2 must ship without parallel "what WFE must ship to receive SX2-authored refusals" — that's the missing reciprocal half.

**Remediation:** File a substrate-authorship request to the synthex-v2 maintainer alongside the WFE schematics. The Watcher-Notices file-drop protocol (`~/projects/shared-context/watcher-notices/`) is the natural channel.

#### NA-7' [SUBSTRATE-FRAME · MED] — Opportunity cost: implicit re-prioritisation

**Plan claim:** ~400 LOC synthex-v2-side + ~80 LOC WFE-side estimated for Wiring 01.

**Substrate-frame reality:** The 7-facet assessment (S1004590, 91/100) named 3 explicit v0.2.2+ priorities: (1) close NA-4, (2) ship one substrate-side schema (Conductor dispatch-budget cheapest), (3) drop `strum::EnumCount`. Wiring 01 IS priority (1), but the cost was characterised as "~80 LOC WFE-side" — the substrate-side ~400 LOC is operator-only and **outside WFE's agency**. The schematics implicitly bet on synthex-v2 prioritising the heartbeat receiver over its own v0.2.2+ horizon items (Phase G shadow window, Watcher Phase J reinforce loop, Ember 8th trait Restraint).

**Remediation:** Master schematic § Honest residuals — add explicit opportunity-cost frame: "this wiring assumes synthex-v2 prioritises a WFE-driven endpoint over [list 3 SX2-side v0.2.2+ items]. If SX2 does not, m16 heartbeats remain in `RefusalToken::Unavailable(SubstrateUnreachable)` indefinitely — which is honest but not loop-closing."

#### NA-8' [SUBSTRATE-FRAME · MED] — WFE-down-mid-burst story is absent

**Plan claim:** Wiring 01 specifies emission semantics + transport semantics.

**Substrate-frame reality:** Both sides silent on what happens when WFE process dies mid-burst. The substrate observes: heartbeat seq=42 arrives at t=0; nothing at t=1, t=2, ..., t=N. Is WFE crashed? Throttling? Network partitioned? No timeout/liveness contract. m46 cannot distinguish.

**Remediation:** Add liveness contract — m16 transport should emit a `Goodbye` event on graceful shutdown; SX2 m10 should track `last_heartbeat_seq_seen_at_ms` and emit its own internal `WfeSilencePersisting` signal after N missed heartbeats (e.g., 3 × poll_interval).

### B.3 LOW-severity findings

#### NA-9' [LOW] — `parent_namespace` field in Wiring 02 pathway_learned is undocumented

**Plan claim:** Wiring 02 § (source, kind) schema lists `data: { pre_id, post_id, weight, parent_namespace }` for `pathway_learned` kind.

**Substrate-frame reality:** `parent_namespace` is not in the existing m42 emit signature; appears speculative.

**Remediation:** Either drop the field OR document its semantics + verify against `src/m42_stcortex_emit/mod.rs`.

#### NA-10' [LOW] — Sequence_number recovery / replay semantics undefined

**Plan claim:** Wiring 01 § 3.2 specifies `sequence_number: u64` monotonic per m16 instance.

**Substrate-frame reality:** What happens on WFE restart? Sequence resets to 0? m46 observes seq drop from 1000 to 0 and concludes... what? Replay attack? Restart? Bug?

**Remediation:** Specify monotonic-per-`(instance_id, boot_id)` semantics; include `boot_id` in the envelope; SX2 tracks `(instance_id, boot_id, last_seen_seq)` per emitter.

### B.4 Load-bearing tensions (3 — require explicit reconciliation)

| ID | Tension | Resolution |
|---|---|---|
| **T1'** | "Bidirectional Push" title vs unidirectional body (Wiring 02) | Rename + author Wiring 02b |
| **T2'** | "NA-4 closure" claim vs `signal_bus_recent` eviction (Wiring 01) | NA-1' ring buffer fix |
| **T3'** | Cross-habitat coordination prescriptive vs reciprocal (D-S1004XXX-05) | NA-6' watcher-notice + reverse V5 |

### B.5 Convergent findings (both frames agree — 2)

#### C1' [CONVERGENT] — m43_synthex_v2_inbound / m43_synthex_v2_ws / m44_watcher_pathway_crawl modules needed

Both passes name new WFE-side modules. **Promote from "Wiring 04 honest residual" to first-class v0.3.0+ planning items.**

#### C2' [CONVERGENT] — Plan v2 §6 NA-4 row update gate

Both passes recommend the Plan v2 §6 NA-4 row update from "mitigated structurally, NOT loop-closed" → "loop-closed" be **gated on an acceptance test** (drift detected end-to-end under load), not on wire existence. Author the acceptance test spec before claiming closure.

---

## Part C — Unified disposition + remediation roadmap

### C.1 Per-schematic dispositions

| Schematic | Disposition | Required amendments | Estimated effort |
|---|---|---|---|
| [[SYNTHEX-V2 Integration Master Schematic]] | AMEND | Add §"7-substrate generalisability audit" (NA-4'); cross-link new Wiring 02b + reverse-V5 (NA-3') + opportunity-cost § (NA-7') | ~1 hr authoring |
| [[Wiring 01 — m16 Heartbeat Consumer (NA-4 Closure)]] | AMEND-BLOCKING | `HeartbeatBuffer` ring requirement (NA-1' MUST FIX) + bilateral V5 (NA-3') + WFE-down liveness contract (NA-8') + sequence_number replay semantics (NA-10') + speculative schema reconciliation (CONV-3) | ~2 hr authoring + read of m16 source |
| [[Wiring 02 — NexusEvent Bidirectional Push]] | RENAME + SPLIT | Rename to "Wiring 02 — NexusEvent Outbound (WFE → SX2)"; update §"Current state" with 501 stub honesty (CONV-1); flag m13→m29 as INTERNAL-only (CONV-2); pre-flight gate on synthex-v2 Phase 3 land | ~1 hr authoring |
| **NEW** [[Wiring 02b — NexusEvent Inbound (SX2 → WFE)]] | AUTHOR NEW | m43 WFE-side consumer module spec (NA-2'); ws/rest receiver shape; refusal channel design | ~2 hr authoring |
| [[Wiring 03 — stcortex Pathway Namespace Alignment]] | LANDS AS AUTHORED | Minor: drop or document `parent_namespace` (NA-9'); reconcile 10 slug families against m42 source (CONV-4) | ~30 min |
| [[Wiring 04 — Watcher (m46–m51) Integration Hooks]] | RENAME | Rename to "Wiring 04 — Watcher Observation of WFE Heartbeats (read-only, no WFE feedback loop)"; add §"What this does NOT do" naming absent feedback path (NA-5'); promote m44_watcher_pathway_crawl to first-class v0.3.0+ item (C1') | ~1 hr authoring |

### C.2 Acceptance gate for "NA-4 loop-closed" claim (C2')

Plan v2 §6 NA-4 row must NOT update from "mitigated structurally" to "loop-closed" until ALL of:

1. ✅ synthex-v2 ships POST `/v3/heartbeat` endpoint at m10
2. ✅ `HeartbeatBuffer` ring (≥600 entries) lives inside m46
3. ✅ End-to-end acceptance test demonstrates: WFE emits 60 heartbeats over 60s under realistic SX2 load (5 heat sources + 6 daemon tasks); m46 detects drift pattern in `signal_bus_recent` within 10s of pattern onset
4. ✅ Bilateral V5 `WorkflowTraceParticipationStatus` ships on SX2-side
5. ✅ WFE-down-mid-burst liveness contract (m16 `Goodbye` + m10 `WfeSilencePersisting` signal)
6. ✅ Reverse-channel module `m43_synthex_v2_inbound` exists in WFE (for SX2→WFE refusal channel)
7. ✅ 48h DX-Soak per OP-3 with the full chain LIVE (per Plan v2 §15 D44 reproducibility gate)

Only ALL 7 closes NA-4 honestly. Wire existence is necessary but not sufficient.

### C.3 v0.2.2+ horizon (revised, gap-informed)

Original 7-facet assessment (S1004590, 91/100) named:
1. Close NA-4 (m16 Watcher consumer)
2. Ship one substrate-side schema (Conductor dispatch-budget cheapest)
3. Drop `strum::EnumCount`

**Gap analysis revises to (leverage-ordered):**

| # | Item | Source | Why |
|---|---|---|---|
| 1 | **Author Wiring 02b + amend Wiring 01/04 per dispositions above** | this gap analysis | Restores planning honesty; ungates v0.2.2+ |
| 2 | **Read m16 + m42 source; verify CONV-3 + CONV-4 + NA-9' + NA-10' speculative claims** | CONV-3, CONV-4 | Wire schemas can't be authored against fictional types |
| 3 | **Close CONV-1 dependency: track synthex-v2 Phase 3 landing** | CONV-1 | Wiring 02 ships nothing until 501 stub is replaced |
| 4 | **Author m43_synthex_v2_inbound + m44_watcher_pathway_crawl scaffolds** | C1' | New WFE-side modules; ~600 LOC + tests |
| 5 | **Bilateral V5 (`WorkflowTraceParticipationStatus`) on SX2 side** | NA-3' | Substrate-authorship request via watcher-notice; ~40 LOC SX2 |
| 6 | **7-substrate generalisability audit** | NA-4' | Master schematic addition; informs whether to author parallel schematic sets per substrate or accept synthex-v2 as the only integration target for v0.2.2+ |
| 7 | NA-4 acceptance test spec | C2' | Block any premature §6 NA-4 row update |
| 8 | Original assessment items 2 + 3 (Conductor schema, strum) | 7-facet | Unchanged priority |

### C.4 Persistence

| Surface | Anchor |
|---|---|
| ai_docs canonical | source reports `/tmp/synthex-v2-deep-evidence-for-gap-analysis.md` + `/tmp/synthex-v2-wiring-plan-na-gap.md` |
| Obsidian vault | THIS NOTE (`the-workflow-engine-vault/Wiring Gap Analysis — S1004590 Dual-Frame.md`) |
| stcortex | ns `workflow_trace_completion_s1004115` — pathway pair: `wiring_gap_analysis_s1004590 ↔ project_session_checkpoint_s1004590` |
| CLAUDE.local.md | top banner § "Wiring Gap Analysis S1004590" with disposition headline |

## Honest residuals

- **Both reports are agent-generated.** Conventional pass cited file:line evidence (high confidence). NA pass is interpretive (NA-1' marked 0.85-conditional on direct m46 source read; the cap-of-50 claim should be verified against `src/m8_watcher/m46_observer.rs:ObservableState.signal_bus_recent` before NA-4 acceptance test is authored).
- **This gap analysis is itself unratified by Luke @ node 0.A.** It is a candidate for ratification, not a ratified plan.
- **Wiring 03 lands as authored** — the only schematic the dual-frame pass agrees on.
