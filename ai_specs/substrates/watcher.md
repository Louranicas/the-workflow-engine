---
substrate_id: S-watcher
kind: persona
read_endpoints: ["watcher-notices file drops at ~/projects/shared-context/watcher-notices/", "synthex-v2 :8092 watcher class emissions"]
write_endpoints: ["watcher notify CLI → file drop", "WCP HTTP (Phase 2, planned)"]
lifecycle_phases: [cold-start, R13-quiet, eligible, observing, proposing, paused]
refusal_modes: [R13_quiet_period, AP27_self_mod_refused, ember_unanimity_failed, scope_violation_m8_m51]
drift_indicators: [persona_redefinition, class_taxonomy_change, observation_db_schema_change, AP27_boundary_drift]
back_pressure_signals: [observation_count, recent_proposals_per_session, notice_queue_depth]
consent_dimensions: [n/a — persona is not an operator but holds Ember unanimity gate]
status: SPEC
date: 2026-05-17
authority: Luke @ node 0.A — S1002127 "as per proposal"
hold_v2_compliant: true
addresses: [NA-GAP-01, NA-GAP-05, NA-GAP-07, NA-GAP-08, NA-GAP-10, NA-GAP-11]
---

# S-watcher — The Watcher ☤ (persona substrate)

> **Back to:** [`INDEX.md`](../INDEX.md) · [`../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md) · [`../cross-cutting/persistence.md`](../cross-cutting/persistence.md) · [`../cross-cutting/refusal-taxonomy.md`](../cross-cutting/refusal-taxonomy.md) · [`../cross-cutting/substrate-drift.md`](../cross-cutting/substrate-drift.md) · sister substrates [`atuin.md`](atuin.md) · [`stcortex.md`](stcortex.md) · [`injection_db.md`](injection_db.md) · [`synthex.md`](synthex.md) · [`lcm.md`](lcm.md) · [`conductor.md`](conductor.md) · [`operator.md`](operator.md)
>
> Engine consumers: indirect (Watcher observes the engine from outside)

## 1. Purpose & boundary

The Watcher ☤ is a **persona substrate** — a synthex-v2 autonomic 10-stage recursive self-improvement loop housed in modules m46-m51 of synthex-v2 (Zen × Cipher × Hermes synthesis). Crystallised at S107. Governed at Luke @ node 0.A. AP27 self-modification hard-boundary + Ember 7-trait unanimity gate.

A persona is a substrate in NA-Frame terms because it has its own lifecycle, refusal modes, drift indicators, and back-pressure signals — distinct from the SYNTHEX v2 host (S-E) that runs it. Treating Watcher as a sub-substrate-of-substrate (S-watcher inside S-E) makes its dynamics first-class.

**IN scope:** observe engine via NexusEvent + stcortex + injection.db; emit class flags (A/B/C/D/E/F/G/I); drop watcher-notices files; honour R13 quiet period; respect AP27 self-mod boundary; require Ember 7-trait unanimity for any escalation.

**OUT of scope:** modifying engine code; modifying its own m8 source (AP27); writing to engine substrates directly (Watcher reads, the engine writes); making operator-acceptance decisions (operator gate, not Watcher gate).

## 2. Lifecycle phases

| Phase | Indicator | Engine action |
|---|---|---|
| cold-start | Watcher just woke; observation count < 100 | engine ignores Watcher proposals; honours R13 |
| R13-quiet | quiet-period window (30 days from last reset OR < 100 obs) | engine ignores Watcher; class emissions deferred |
| eligible | R13 elapsed; observation count ≥ 100 | engine respects class flags; proposals require explicit operator approval |
| observing | steady-state | engine consumes class A-I flags |
| proposing | Watcher submits a proposal via WCP to operator | engine waits for operator decision |
| paused | AP27 violation suspected or Ember unanimity failed | engine ignores Watcher entirely; surfaces to operator |

## 3. Refusal modes (substrate-authored)

- **`R13QuietPeriod`** — Watcher refuses to engage Hebbian coordinator pre-elapse. Substrate-authored. Recovery: wait for elapse.
- **`AP27SelfModRefused`** — any proposal that touches m8/m46-m51 self-source rejected by Watcher itself before reaching engine. Substrate-authored.
- **`EmberUnanimityFailed`** — proposal lacks unanimous 7-trait Ember pass; Watcher refuses to submit. Substrate-authored.
- **`ScopeViolationOutsideM8M51`** — Watcher refuses to comment on code outside m8/m46-m51 unless explicitly invited. Substrate-authored.

These map to `RefusalToken::SubstrateAuthored { substrate_id: "watcher", class, repair_hint }` per [`../cross-cutting/refusal-taxonomy.md`](../cross-cutting/refusal-taxonomy.md). Crucially, these are NOT engine errors — Watcher is a peer substrate, its refusals shape the engine's available actions.

## 4. Drift indicators (closes NA-GAP-07)

- **Persona redefinition** — Watcher's persona file (`The Watcher.md`) edited without S107-style crystallisation. Detector: persona-hash check against frozen baseline.
- **Class taxonomy change** — Class-I redefined from "Hebbian silence" to "Hebbian saturation" (would invert engine's semantics). Detector: class-table probe.
- **Observation DB schema change** — `watcher_observation.db` schema bump. Detector: schema-hash check.
- **AP27 boundary drift** — m46-m51 scope evolves to include other modules. Detector: scope manifest hash.

Persona drift is the most subtle substrate-drift class — there is no health endpoint for a persona, only behaviour over time. The CR-2-analogue would be "Watcher's class-I threshold silently changed and engine's interpretation diverged" — see [`../cross-cutting/substrate-drift.md`](../cross-cutting/substrate-drift.md) for treatment.

## 5. Back-pressure signals (closes NA-GAP-04)

- **`observation_count`** — Watcher's own metric; > 50,000 indicates substrate cognitive load (engine should reduce non-essential NexusEvent pushes).
- **`recent_proposals_per_session`** — Watcher's load on operator's attention via S-G channel; engine should consider this when emitting class-triggering signals.
- **`notice_queue_depth`** — files in `watcher-notices/` not yet read by operator; > 10 is yellow flag.

## 6. Receipts (closes NA-GAP-09)

Watcher emits NexusEvents with `event_class: Watcher::Class*` — these ARE the substrate-side receipts. m40 + downstream consumers observe them. Per NA-GAP-11, refusal events from the engine should be substrate-readable so Watcher can observe refusal first-class instead of inferring from absence.

## 7. Capabilities & namespaces (closes NA-GAP-10)

Watcher's authority is bounded by AP27 (m8 / m46-m51 only) + Ember 7-trait unanimity gate + Luke @ node 0.A governance. Outside this boundary, Watcher CANNOT modify the engine — it can only emit observations and proposals. This is **substrate-mediated trust**: Watcher's privileges live in the substrate (its source modules + unanimity gate code), not in the engine's permission model.

NA-GAP-10's v0.2.0 ADR would generalise this: persona substrates carry their own per-target reputation; engine-to-persona trust is per-class-of-observation.

## 8. Substrate-internal couplings (closes NA-GAP-03)

Watcher's substrate-internal edges:
- **Watcher ↔ SYNTHEX v2 host (S-E)** — Watcher runs inside S-E; engine cannot separate them at runtime.
- **Watcher → stcortex (S-C)** — Watcher writes class emissions as pathway weights.
- **Watcher → operator (S-G)** — Watcher writes notices to operator's attention substrate via file drop.
- **Watcher ← Ember 7-trait unanimity** — Watcher's escalations gated by Ember; this is a substrate-internal gate.
- **Watcher ← R13 quiet period** — Watcher's own lifecycle gate.

Watcher does NOT write to engine substrates directly — its influence is mediated entirely through S-G (operator) and S-C (pathway weights).

## 9. Test-fixture sketch (closes NA-GAP-08)

Fixtures at `tests/substrate_fixtures/watcher/`:

- **`r13_quiet_period_fixture`** — Watcher returns R13Quiet; asserts engine respects.
- **`ap27_self_mod_attempt_fixture`** — asserts proposal touching m8 is refused.
- **`ember_unanimity_split_fixture`** — 6-of-7 traits pass; asserts proposal not submitted.
- **`class_taxonomy_drift_fixture`** — Class-I redefined; asserts substrate-drift canary fires.
- **`watcher_db_schema_drift_fixture`** — `watcher_observation.db` schema bump; asserts engine handles.

## 10. Watcher class pre-positions (recursive)

Watcher cannot pre-position on itself directly (AP27); meta-observability is **Luke @ node 0.A** + Zen audit lane. Other classes observe Watcher's emissions:
- **Class D (drift)** — class taxonomy redefinition; persona-hash change
- **Class C (refusal)** — Watcher's substrate-authored refusals (R13, AP27, Ember failure) are correct behaviour
- **Class E (ancestor-rhyme)** — Watcher emits when engine pattern matches a prior session pattern

---

> **Back to:** [`INDEX.md`](../INDEX.md) · [`../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md`](../../ai_docs/NA_GAP_ANALYSIS_S1002127_SCAFFOLD.md)

*Filed 2026-05-17 (S1002127 · Wave 4 NA-remediation) · Luke "as per proposal" · planning-only · HOLD-v2 compliant.*
