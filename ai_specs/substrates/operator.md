---
substrate_id: S-G-operator
kind: persona
read_endpoints: ["operator terminal stdout (m32 banner + m12 reports)", "watcher-notices/ inbox", "Obsidian vault notes"]
write_endpoints: ["HumanAcceptanceSignature on BankDb::accept", "Luke directive lines", "explicit `start coding workflow-trace` signal", "Zen G7 audit verdict"]
lifecycle_phases: [fresh, focused, fatiguing, fatigued, recovered, off-shift]
refusal_modes: [consent_fatigue, attention_overload, frame_conflict, ambiguity, latency_drift, off-shift, ember_unanimity_held]
drift_indicators: [escape_surface_banner_familiarity_blindness, dispatch_density_normalisation, frame_switching_cost_increase, prompt_text_drift]
back_pressure_signals: [banner_count_per_session, prompt_density, attention_remaining, frame_switches_per_hour, decision_latency_p50]
consent_dimensions: [explicit_acceptance, modulation_not_command, held_semantics_ember_5_1, attention_budget, frame_context, acceptance_fatigue_cap]
status: SPEC
date: 2026-05-17
authority: Luke @ node 0.A — S1002127 "as per proposal"
hold_v2_compliant: true
addresses: [NA-GAP-01, NA-GAP-02, NA-GAP-05, NA-GAP-08, NA-GAP-09, NA-GAP-10, NA-GAP-11]
---

# S-G — Operator (operator-as-substrate)

> **Back to:** [`INDEX.md`](../INDEX.md) · [`../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md) · [`../cross-cutting/persistence.md`](../cross-cutting/persistence.md) · [`../cross-cutting/refusal-taxonomy.md`](../cross-cutting/refusal-taxonomy.md) · [`../cross-cutting/substrate-drift.md`](../cross-cutting/substrate-drift.md) · [`../CONSENT_SPEC.md`](../CONSENT_SPEC.md) · sister substrates [`atuin.md`](atuin.md) · [`stcortex.md`](stcortex.md) · [`injection_db.md`](injection_db.md) · [`synthex.md`](synthex.md) · [`lcm.md`](lcm.md) · [`conductor.md`](conductor.md) · [`watcher.md`](watcher.md)
>
> Engine touch points: m12 reports (operator inbox), m23 → m30 review gate, m30 BankDb::accept (HumanAcceptanceSignature), m32 stdout banner (Gap 3 display half), Zen G7 audit lane, Luke @ node 0.A directive surface.

## 1. Purpose & boundary

The operator (Luke + Zen audit lane + any future review queue) is **a substrate with its own dynamics, not an oracle**. Per NA-GAP-05, the scaffold collapses operator-cognition into a `HumanAcceptanceSignature` field on `BankDb::accept()` and a "Luke decides" row in a decision table — treating it as an external oracle with infinite capacity. Frame A surfaces that the operator has its own lifecycle, refusal modes, drift indicators, back-pressure signals, and consent dimensions; without modelling them, the engine's flow will overload the operator and the unmodelled bottleneck becomes invisible.

**IN scope:** model operator-attention budget, consent-fatigue cap, frame-switching cost, dispatch latency; surface operator-load metrics via m12 reports; enforce a per-session EscapeSurfaceProfile banner cooldown; treat operator refusals as first-class typed events.

**OUT of scope:** modelling Luke's personal capacity in detail (privacy + dignity); replacing operator with automation (m23 → m30 is a hard human-in-loop gate per AP-V7-07).

## 2. Lifecycle phases

| Phase | Indicator | Engine action |
|---|---|---|
| fresh | session start; banner count = 0; recent decisions ≤ 2 | normal cadence |
| focused | mid-session; banner count 1-3; decisions flowing | normal cadence |
| fatiguing | banner count 4-5; decision latency rising | engine pauses non-critical banners; surfaces "fatigue warning" event |
| fatigued | banner count > 5 per session OR decision latency p50 > 5 min | engine MANDATORY cooldown: no new banner-triggering dispatches until operator acks reset |
| recovered | operator explicitly resets (typed `reset`) OR new session starts | normal cadence |
| off-shift | no operator presence (no keystrokes for > 30 min) | engine defers all decision-requiring work; outbox accumulates |

## 3. Refusal modes (operator-authored)

The operator can refuse, and these refusals are first-class typed events per [`../cross-cutting/refusal-taxonomy.md`](../cross-cutting/refusal-taxonomy.md) (`RefusalToken::OperatorRefusal`):

- **`ConsentFatigue`** — operator explicitly says "stop showing me these" OR engine detects banner-density cap exceeded.
- **`AttentionOverload`** — operator is mid-frame-switch (e.g. reading m12 report while engine surfaces m32 banner). Engine defers.
- **`FrameConflict`** — operator is in cluster-spec frame; engine surfaces dispatch frame. Operator must context-switch; engine should batch.
- **`Ambiguity`** — operator surfaces "what frame is that?" or asks clarifying question. Engine must restate, not proceed.
- **`LatencyDrift`** — operator has not responded in expected window; engine surfaces alternate-channel notice (watcher-notices) rather than another inline banner.
- **`OffShift`** — operator gone; engine queues.
- **`EmberUnanimityHeld`** — operator explicitly invokes Ember §5.1 Held verdict on a string; engine treats as Held, not as Pass.

## 4. Drift indicators (closes NA-GAP-07)

The operator-as-substrate drifts too:

- **EscapeSurfaceProfile banner familiarity-blindness** — after the 20th identical PrivilegeEscalation banner, operator's eye stops registering them as warnings. Detector: banner-correlation with operator-decision-latency; rising latency on repeat banners signals drift.
- **Dispatch-density normalisation** — operator's threshold for "noisy" shifts as engine cadence increases; what was alarming becomes background. Detector: operator-stated "noisy" threshold vs measured dispatch rate.
- **Frame-switching cost increase** — operator switches between cluster-spec / gate-state / conversation / vault more slowly as session ages. Detector: frame-switch-latency EMA.
- **Prompt text drift** — engine subtly changes banner wording across versions; operator's mental model lags. Detector: banner-wording-hash + operator-decision-accuracy correlation.

Per the CR-2 analogue, the operator substrate can be "200 OK on the surface" (responding to banners) while semantically drifting (no longer reading them). See [`../cross-cutting/substrate-drift.md`](../cross-cutting/substrate-drift.md).

## 5. Back-pressure signals (closes NA-GAP-04, NA-GAP-05)

Quantifiable signals the engine MUST track:

| Signal | Threshold (provisional) | Engine action |
|---|---|---|
| `banner_count_per_session` | > **5** EscapeSurfaceProfile banners | enter `fatiguing` phase; mandatory cooldown of 10 min before next banner |
| `prompt_density` (m12 reports / hour) | > 3 | batch into single digest |
| `attention_remaining` (proxy: time since last decision) | < 30s for next decision | defer new decision-requiring work |
| `frame_switches_per_hour` | > 6 | engine batches by frame |
| `decision_latency_p50` | > 5 min | escalate fatigue warning |
| `acceptance_fatigue_cap` | 5 EscapeSurfaceProfile dispatches before cooldown | hard cap; engine refuses new dispatch until operator resets |

**Provisional consent-fatigue cap value:** **5 banners per session before mandatory 10-min cooldown.** This is an open question for operator review — see § Open Questions in the report. The cap is intentionally conservative; the cost of underestimation is operator overload, the cost of overestimation is slow dispatch — the asymmetry favours conservatism.

## 6. Receipts (closes NA-GAP-09, NA-GAP-11)

Operator-authored receipts:
- **`HumanAcceptanceSignature`** on `BankDb::accept` is the explicit substrate-side receipt for admission.
- **Luke @ node 0.A directive line** (e.g. "as per proposal", "start coding workflow-trace") is the explicit dispatch-of-authority receipt.
- **Zen G7 audit verdict** is the explicit audit-pass receipt.

Per NA-GAP-11, when the operator refuses, the engine MUST emit `WireEvent::Refusal { token: OperatorRefusal { ... } }` so the refusal is observable across substrates — not Watcher-inferred from absence-of-acceptance. This is the canonical operator-refusal observability gap closure.

## 7. Capabilities & namespaces (closes NA-GAP-10)

Operator is the highest-authority substrate in the habitat (Luke @ node 0.A authority + Ember 7-trait unanimity gate + Zen G7 audit lane). The engine's per-substrate trust model places operator at the top of the trust hierarchy — operator refusals are **terminal** (no override), operator approvals are **explicit-only** (no auto-promote per AP-V7-07).

The engine MUST NOT generate fake operator signatures, fake operator approvals, or assume silent consent (per `feedback_direct_team_comms` + AP-V7-08). Silence ≠ consent.

## 8. Substrate-internal couplings (closes NA-GAP-03)

Operator's substrate-internal edges:
- **Operator → Watcher (S-watcher)** — operator's directives shape Watcher's R13 window, Ember unanimity gate, AP27 boundary.
- **Operator → Conductor (S-D)** — operator's `devenv start` + env flag flip controls Conductor's lifecycle.
- **Operator → LCM (S-F)** — operator's deploy-cancel directs LCM's lifecycle.
- **Operator → engine (workflow-trace)** — `start coding`, acceptance signature, Zen audit verdict.
- **Operator ← Conductor (S-D)** via stdout banner — substrate writes to operator's attention.
- **Operator ← Watcher (S-watcher)** via watcher-notices — substrate writes to operator's inbox.
- **Operator ← engine** via m12 reports + EscapeSurfaceProfile banners.

The operator is **the only substrate with bidirectional edges to every other substrate**. This makes it the busiest substrate in the habitat — and the most likely bottleneck.

## 9. Test-fixture sketch (closes NA-GAP-08)

Fixtures at `tests/substrate_fixtures/operator/`:

- **`consent_fatigue_cap_exceeded_fixture`** — simulate 6 banners in one session; asserts engine enters mandatory cooldown.
- **`frame_conflict_fixture`** — operator state shows mid-frame-switch; asserts engine defers banner.
- **`off_shift_fixture`** — no operator keystrokes for 30 min; asserts engine queues decision-requiring work.
- **`ember_held_fixture`** — operator returns Held verdict on string; asserts engine treats as Held, not Pass.
- **`acceptance_signature_missing_fixture`** — asserts BankDb::accept refuses without signature (already enforced engine-side; this fixture verifies operator's typed refusal is observable).
- **`silence_not_consent_fixture`** — asserts engine never auto-promotes from absence-of-rejection.

## 10. Watcher class pre-positions

- **Class A (activation)** — first operator acceptance of a workflow-trace dispatch
- **Class B (boundary)** — every operator-decision-required surface (m12 / m32 banner / m30 accept)
- **Class C (refusal)** — operator refusal as correct behaviour; Class-C count must be > 0 in steady-state
- **Class D (drift)** — operator-decision-latency rising on repeat banners → familiarity-blindness suspected
- **Class E (ancestor-rhyme)** — operator's directive language echoes prior session
- **Class F (frame conflict)** — operator state changes mid-decision; engine should batch
- **Class G (governance)** — Luke @ node 0.A directive line received

## Open question: consent-fatigue cap value

**The provisional cap of 5 banners per session before mandatory 10-min cooldown is engine-internal.** This value MUST be reviewed and calibrated by the operator (Luke) before G9. Candidate calibration mechanism: instrument m32 banner count + operator-decision-latency across the first 5 Phase-2 measure-only weeks; surface the histogram via m12 report; let Luke pick the cap on observed data, not a guess.

---

> **Back to:** [`INDEX.md`](../INDEX.md) · [`../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md) · [`../CONSENT_SPEC.md`](../CONSENT_SPEC.md)

*Filed 2026-05-17 (S1002127 · Wave 4 NA-remediation) · Luke "as per proposal" · planning-only · HOLD-v2 compliant.*
