# workflow-trace — Error Taxonomy

> **Back to:** [`README.md`](../README.md) · [`CLAUDE.md`](../CLAUDE.md) · [`ARCHITECTURE_DEEP_DIVE.md`](ARCHITECTURE_DEEP_DIVE.md) · [`CODE_MODULE_MAP.md`](CODE_MODULE_MAP.md) · [`MESSAGE_FLOWS.md`](MESSAGE_FLOWS.md) · per-module specs at `../ai_specs/modules/cluster-{A-H}/m<N>_<name>.md`
>
> **Function:** Per-cluster thiserror taxonomy. All errors implement `Display`, `Error`, derive `thiserror::Error`, and carry structured context. Synthesised from per-module spec `## 2 Public surface` blocks and the canonical `src/errors.rs` consolidation in `workflow_core`. Status: planning-only · 0 LOC · names + structure locked, exact field shapes finalise at G9.

---

## 1. Top-level enum (consolidation pattern)

Per [`GENERATIONS/G2-consolidation.md`](optimisation-v7/GENERATIONS/G2-consolidation.md) § Canonical src/ layout, `workflow_core/src/errors.rs` consolidates per-module errors into a single workspace-public `WorkflowTraceError`:

```text
pub enum WorkflowTraceError {
    Atuin(m1::AtuinError),                     // Cluster A
    Stcortex(m2::Stc2Error),
    Injection(m3::InjectionError),
    Cascade(m4::CascadeError),                 // Cluster B
    Battern(m5::BatternError),
    Cost(m6::CostError),
    Hub(m7::HubError),                         // Cluster C
    Report(m12::ReportError),
    Writer(m13::WriterError),
    BuildPrereq(m8::BuildPrereqError),         // Cluster D (aspect)
    Namespace(m9::NamespaceError),
    Ember(m10::EmberError),
    Decay(m11::DecayError),
    Lift(m14::LiftError),                      // Cluster E
    Pressure(m15::PressureError),
    Miner(m20::MinerError),                    // Cluster F (KEYSTONE)
    Builder(m21::BuilderError),
    KMeans(m22::KMeansError),
    Proposal(m23::ProposalError),
    Bank(m30::BankError),                      // Cluster G
    Selector(m31::SelectorError),
    Dispatch(m32::DispatchError),
    Verify(m33::VerifyError),
    Nexus(m40::NexusError),                    // Cluster H
    Lcm(m41::LcmError),
    Emit(m42::EmitError),
}
```

Per-module errors are listed by cluster below.

---

## 2. Cluster A — Substrate Ingest (L1)

### 2.1 `m1::AtuinError`

| Variant | Description | Recovery |
|---|---|---|
| `DbOpen { path, source }` | atuin SQLite open failed | Verify path; subprocess fallback to `atuin search` |
| `CursorInvalid { cursor }` | malformed pagination cursor | Restart from cursor 0 |
| `RowParse { row, source }` | row schema mismatch | Skip row + emit m15 pressure event |
| `SchemaMismatch { expected, found }` | atuin schema version drift | Hard fail; surface to operator |

### 2.2 `m2::Stc2Error`

| Variant | Description | Recovery |
|---|---|---|
| `ConnectFailed { endpoint, source }` | stcortex `:3000` unreachable | Offline JSON fallback at `data/snapshots/latest.json` |
| `ReducerRegistry { name }` | reducer not in narrowed scope (tool_call, consumption only) | Refuse + emit m15 pressure event |
| `WireSerde { source }` | SpacetimeDB wire decode failed | Log + skip frame |
| `OfflineFallbackUsed` | non-fatal sentinel — Watcher Class-A consumer of this | continue |

### 2.3 `m3::InjectionError`

| Variant | Description | Recovery |
|---|---|---|
| `DbMissing { path }` | injection.db not at `~/.local/share/habitat/injection.db` | Hard fail |
| `RowParse { row_id }` | causal_chain row malformed | Skip + warn |
| `ResolvedPartitionEmpty` | both resolved/unresolved buckets empty | Continue with no-data result |

---

## 3. Cluster B — Habitat Observation (L2)

### 3.1 `m4::CascadeError`

| Variant | Description | Recovery |
|---|---|---|
| `WindowEmpty` | no rows in correlation window | Continue with empty `CascadeCluster` |
| `OpaqueIdCollision { id }` | FNV-1a XOR collision (statistically rare) | Re-derive with sorted pane labels |
| `UpstreamFailed { source }` | m1/m2/m3 read failed | Propagate |
| `F11Violation { cluster_id }` | cluster_id contains semantic pane label substring (ALPHA/BETA/GAMMA) | Hard fail — AP-WT-F11 |

### 3.2 `m5::BatternError`

| Variant | Description | Recovery |
|---|---|---|
| `StepLabelInvalid { raw }` | `step_label` Option deserialisation failed | Skip step + warn |
| `RingFull` | append-only ring buffer overflowed | Drop oldest with metric increment |
| `UpstreamFailed { source }` | m1/m3 read failed | Propagate |

### 3.3 `m6::CostError`

| Variant | Description | Recovery |
|---|---|---|
| `EmaWindowUninitialised` | <20 sessions observed yet | Return `ContextCostBand { n: <20 }` for caller to gate |
| `ConvergedLeak { session_id }` | F10 violation — Converged outcome included in baseline | Hard fail — AP-WT-F10 |
| `UpstreamFailed { source }` | m1/m3 read failed | Propagate |

---

## 4. Cluster C — Correlation + Output (L3)

### 4.1 `m7::HubError`

| Variant | Description | Recovery |
|---|---|---|
| `SchemaVersionMismatch { expected, found }` | hub schema drift | Migrate via `migrations/` |
| `JsonbSerde { source }` | `consumer_inputs` JSONB encode/decode failed | Skip row + emit m15 pressure |
| `FitnessDimensionNull { row_id }` | F9 zero-weight violated (fitness_dimension NULL) | Hard fail — AP-WT-F9 |
| `InsertFailed { source }` | SQLite insert failed | Retry once; then propagate |

### 4.2 `m12::ReportError`

| Variant | Description | Recovery |
|---|---|---|
| `EmberHeldVerdict { trait, score }` | m10 Ember rubric returned Held for user-facing string | Block emit; require revision |
| `JsonSerde { source }` | JSON report serialisation failed | Fall back to human report |
| `UpstreamFailed { source }` | m7 read failed | Propagate |

### 4.3 `m13::WriterError`

| Variant | Description | Recovery |
|---|---|---|
| `BandGateBlocked { band: LtpLtdBand }` | 3-band gate blocked write (LTD-dominated regime) | Defer to JSONL buffer |
| `NamespaceViolation` | m9 namespace guard rejected (delegates to `m9::NamespaceError`) | Hard fail — AP-Hab-03 |
| `JsonlBufferOverflow { current_bytes }` | deferred buffer exceeded cap | Spill to disk |
| `StcortexWire { source }` | upstream stcortex write failed | Retry + circuit-break |

---

## 5. Cluster D — Trust ASPECT (L4)

### 5.1 `m8::BuildPrereqError` (build-time; surfaces via `compile_error!`)

| Variant | Description | Recovery |
|---|---|---|
| `EnvUnset { var: "POVM_CALIBRATED_BAND" }` | env var not present at `cargo check` | Set env; re-build |
| `BandOutOfRange { lo, hi, observed }` | `learning_health` reading outside calibrated band [0.05, 0.15] | Update calibration OR re-deploy POVM (note: POVM DECOUPLED from m42 per 2026-05-17 ADR; m8 retains for `substrate_LTP_density` display only) |
| `BuildScriptIO { source }` | build.rs IO failed | Surface build error |

### 5.2 `m9::NamespaceError`

| Variant | Description | Recovery |
|---|---|---|
| `PrefixMissing { namespace }` | not `workflow_trace_*` prefixed | Hard refusal — AP30 |
| `HyphenSlugMunge { raw }` | hyphen-slug stcortex munge (S1001757 bug) | Auto-convert `-` → `_`; warn |
| `EscapeSurfaceUnclassified { workflow_id }` | missing EscapeSurfaceProfile classification | Block + require classification |

### 5.3 `m10::EmberError`

| Variant | Description | Recovery |
|---|---|---|
| `RubricLoadFailed { path, source }` | Ember rubric file unreadable | Fail closed (CI red) |
| `HeldVerdict { trait_name, score, threshold }` | 7-trait audit returned Held | Block CI; require revision |
| `Sect5_1AmendmentMissing` | Ember §5.1 Held-semantics amendment not yet landed (B4) | Soft-fail with warning (current default until B4 closes) |

### 5.4 `m11::DecayError`

| Variant | Description | Recovery |
|---|---|---|
| `OutOfRange { value }` | `DecayFactor` ∉ [0.0, 1.0] or non-finite | Clamp + emit metric |
| `RecencyEpochInvalid { last_run_at }` | last_run_at in future | Treat as `now`; warn |
| `FitnessSignalMissing { workflow_id }` | stcortex pathway weight unreadable | Use base_rate floor only |
| `FrequencyDivByZero` | observation window empty | Frequency factor = 0.0 |
| `PruneEmitFailed { source }` | m13 prune-marker emit failed (delegates to `m13::WriterError`) | Retry + circuit-break |

---

## 6. Cluster E — Evidence + Pressure (L5)

### 6.1 `m14::LiftError`

| Variant | Description | Recovery |
|---|---|---|
| `SampleSizeBelowF2 { n }` | n<20 — Wilson CI uncomputable | Return `Option::None` (NOT an error to caller; m23 catches as `ProposalError::LiftEvidenceMissing`) |
| `WilsonCIDivergent` | numerical instability in CI compute | Log + return None |
| `UpstreamFailed { source }` | m7 read failed | Propagate |

### 6.2 `m15::PressureError`

| Variant | Description | Recovery |
|---|---|---|
| `JsonlWriteFailed { path, source }` | atomic tmp+rename failed | Retry to fallback path |
| `DedupWindowCollision { key }` | 60s dedup window match (coalesce expected) | Increment count on existing file |
| `AgentCrossTalkUnreachable { path }` | `~/projects/shared-context/agent-cross-talk/` not writable | Spill to `/tmp/`; emit metric |

---

## 7. Cluster F — Iteration KEYSTONE (L6)

### 7.1 `m20::MinerError`

| Variant | Description | Recovery |
|---|---|---|
| `MinSupportInvalid { value }` | min_support ∉ (0.0, 1.0] | Hard fail |
| `SequenceEmpty` | no sequences to mine | Return empty pattern set |
| `WilsonGateBlocked { pattern, n }` | m14 evidence missing for pattern | Skip pattern |
| `MemoryBudgetExceeded { current_bytes, cap }` | PrefixSpan memory cap hit | Spill to disk + warn |

### 7.2 `m21::BuilderError`

| Variant | Description | Recovery |
|---|---|---|
| `LevenshteinDistanceInfinite { a, b }` | normalisation div-by-zero (empty inputs) | Skip pair |
| `TopKEmpty { k }` | fewer than K variants buildable | Return what's available; log |
| `UpstreamFailed { source }` | m20 returned empty | Propagate |

### 7.3 `m22::KMeansError`

| Variant | Description | Recovery |
|---|---|---|
| `KOutOfRange { k, max }` | k > n samples | Hard fail |
| `ConvergenceTimeout { iters }` | did not converge in max iters | Return best-so-far + warn |
| `FeatureVectorEmpty` | upstream produced empty feature vectors | Skip |

### 7.4 `m23::ProposalError`

| Variant | Description | Recovery |
|---|---|---|
| `LiftEvidenceMissing` | m14 returned `Option::None` (n<20); **CC-3 gate** | Refuse construction — F2 mitigation |
| `GradientPreservationViolation { workflow_id }` | proposal would invert gradient | Hard refusal |
| `DeviationBelowFloor { n, floor: 5 }` | n<5 relaxed-deviation gate | Refuse |
| `UpstreamFailed { source }` | m20/m21/m22 failure | Propagate |

---

## 8. Cluster G — Bank / Select / Dispatch / Verify (L7)

### 8.1 `m30::BankError`

| Variant | Description | Recovery |
|---|---|---|
| `AutoPromotionAttempted { workflow_id }` | F5 violation — admit without `accepted_by: HumanAcceptanceSignature` | Hard refusal |
| `EscapeSurfaceMissing { workflow_id }` | Gap 3 violation | Block admit |
| `DefinitionHashCollision { hash }` | FNV-1a hash collision | Re-hash with HMAC-SHA256 upgrade path |
| `SunsetExpired { workflow_id, sunset_at }` | admit past sunset_at | Refuse |
| `DbInsert { source }` | SQLite insert failed | Retry once; then propagate |

### 8.2 `m31::SelectorError`

| Variant | Description | Recovery |
|---|---|---|
| `BankEmpty` | no admitted entries | Return `Option::None` |
| `CompositeScoreNaN { workflow_id }` | α/β/γ/δ component produced NaN | Skip entry + emit metric |
| `DiversityFloorViolated` | δ component below floor | Re-select with relaxed diversity |
| `DecayReadFailed { source }` | m11 decay factor unavailable | Use base_rate floor |

### 8.3 `m32::DispatchError`

| Variant | Description | Recovery |
|---|---|---|
| `ConductorDispatchDisabled` | B3 blocker — `auto_start=false` for Waves 1B/1C/2/3 | Refuse-mode (NOT panic, NOT exit) |
| `ConductorHealthFail { status }` | 5-check #1 failed | Refuse-mode |
| `VerifyTtlExpired { ttl_expires_at }` | 5-check #2 failed | Refuse-mode; caller re-runs m33 |
| `DefinitionHashDrift { expected, found }` | 5-check #3 failed | Refuse-mode; re-verify |
| `SunsetExpired { sunset_at }` | 5-check #4 failed | Refuse-mode; m30 sunsets entry |
| `DispatchCooldownActive { remaining_ms }` | 5-check #5 failed | Refuse-mode; backoff |
| `SelfDispatchRefused { workflow_id }` | F6 violation — m32 dispatching itself | Hard refusal |
| `NamespaceViolation` | delegates to `m9::NamespaceError` | Hard refusal |
| `ConductorWire { source }` | HTTP POST `/dispatch` failed | Retry once; circuit-break on second fail |

### 8.4 `m33::VerifyError`

| Variant | Description | Recovery |
|---|---|---|
| `AgentUnavailable { agent, source }` | one of 4 verifier agents unreachable | Mark DEGRADED + emit metric |
| `VerdictDisagreement { verdicts }` | 4-agent quorum split | DEGRADED + Watcher notify |
| `TtlCacheCorrupt { source }` | SQLite verify-cache unreadable | Rebuild from m30 |

---

## 9. Cluster H — Substrate Feedback (L8)

### 9.1 `m40::NexusError`

| Variant | Description | Recovery |
|---|---|---|
| `OutboxWriteFailed { source }` | local JSONL outbox write failed | Hard fail (durability MUST hold) |
| `WirePostFailed { status, source }` | `:8092/v3/nexus/push` non-2xx | Best-effort — leave in outbox; retry on next cycle |
| `SerdeRenameTypeTrap { source }` | serde `rename = "type"` trap (most likely silent failure mode) | Hard fail at deploy verify; not a runtime path |
| `CircuitOpen` | breaker open after 5 failures | Hold + retry after 30s |

### 9.2 `m41::LcmError`

| Variant | Description | Recovery |
|---|---|---|
| `RpcConnectFailed { source }` | UDS connect failed | Reconnect-per-call (no persistent connection) |
| `MaxItersInvalid { value, expected: 1 }` | called with anything other than `max_iters: 1` | Hard fail (we are NOT `lcm.deploy`) |
| `JsonRpcInvalid { source }` | LCM returned malformed JSON-RPC | Log + skip |

### 9.3 `m42::EmitError`

| Variant | Description | Recovery |
|---|---|---|
| `StcortexUnreachable { source }` | `:3000` unreachable | **NO silent POVM fallback** per 2026-05-17 ADR; spill to outbox + alert |
| `FitnessDeltaOutOfRange { value, expected: [-0.10, +0.25] }` | invalid fitness_delta | Hard fail |
| `NamespaceViolation` | delegates to `m9::NamespaceError` | Hard refusal |
| `PathwayWeightStale { age_ms }` | read-back stale beyond freshness threshold | Warn + Watcher Class-I candidate |

---

## 10. Cross-cluster error propagation paths

```
Cluster A failures (DbOpen, ConnectFailed) ─┐
                                            ├─► m4/m5/m6 UpstreamFailed
                                            └─► m7 InsertFailed (partial row skip)

Cluster B failures (F10/F11) ───────────────► hard fail at compile/CI gate

m7 HubError::FitnessDimensionNull ──────────► AP-WT-F9 hard fail (write boundary)

m9 NamespaceError ──────────────────────────► writers in m13 + m42 refuse-mode
                                              + AP-Hab-03 alert

m11 DecayError::FitnessSignalMissing ───────► m31 fallback to base_rate (NOT a hard fail)

m14 LiftError::SampleSizeBelowF2 ───────────► m23 ProposalError::LiftEvidenceMissing
                                              (CC-3 gate — refuse-construction)

m32 DispatchError (any 5-check fail) ───────► refuse-mode (Watcher Class-C)
                                              ─► caller re-runs (m33 / m30 / m31)
                                              ─► NEVER panic or exit process
                                              ─► NEVER silent-success path

m40/m41/m42 wire failures ──────────────────► outbox-first JSONL preserves durability
                                              ─► circuit-break on persistent failure
                                              ─► CC-5 read-back will detect silence
                                              ─► Watcher Class-I primary detector
```

---

## 11. Anti-pattern guardrails

Per [`ANTIPATTERNS_REGISTER.md`](optimisation-v7/ANTIPATTERNS_REGISTER.md):

| AP | Error variant that catches it |
|---|---|
| AP-Hab-03 namespace violation | `m9::NamespaceError::PrefixMissing` |
| AP-Hab-05 PIPESTATUS swallow | not a runtime error — gate-script discipline |
| AP-WT-F2 sample-size inflation | `m14::LiftError::SampleSizeBelowF2` + `m23::ProposalError::LiftEvidenceMissing` |
| AP-WT-F5 bank creep | `m30::BankError::AutoPromotionAttempted` |
| AP-WT-F6 self-dispatch | `m32::DispatchError::SelfDispatchRefused` |
| AP-WT-F7 CR-2 graceful-degrade pretend-fix | `m8::BuildPrereqError::BandOutOfRange` (build-time) |
| AP-WT-F9 fitness_dimension default-zero | `m7::HubError::FitnessDimensionNull` |
| AP-WT-F10 exploration-cost collapse | `m6::CostError::ConvergedLeak` |
| AP-WT-F11 cascade monoculture | `m4::CascadeError::F11Violation` |
| AP-V7-13 health-200 ≠ behaviour-verified | not a runtime error — Phase 5A deploy gate discipline |

---

## 12. Cross-references

- **Per-module specs (Public surface § 2 blocks):** `../ai_specs/modules/cluster-{A-H}/m<N>_<name>.md`
- **Antipatterns:** [`ANTIPATTERNS_REGISTER.md`](optimisation-v7/ANTIPATTERNS_REGISTER.md)
- **Cross-cluster contracts:** [`CROSS_CLUSTER_SYNERGIES.md`](optimisation-v7/MODULE_PLANS/CROSS_CLUSTER_SYNERGIES.md)
- **Message flows (where errors surface):** [`MESSAGE_FLOWS.md`](MESSAGE_FLOWS.md)
- **State machines (refuse-mode visualisations):** [`STATE_MACHINES.md`](STATE_MACHINES.md)

> **Back to:** [`README.md`](../README.md) · [`CLAUDE.md`](../CLAUDE.md) · [`ARCHITECTURE_DEEP_DIVE.md`](ARCHITECTURE_DEEP_DIVE.md) · [`CODE_MODULE_MAP.md`](CODE_MODULE_MAP.md)

*ERROR_TAXONOMY authored 2026-05-17 (S1001982) by Command. Synthesised from per-module spec Public-surface blocks; preserves m42 stcortex-only fact (no silent POVM fallback), AP-WT-F1..F11 mapping, and refuse-mode-not-panic m32 discipline.*
