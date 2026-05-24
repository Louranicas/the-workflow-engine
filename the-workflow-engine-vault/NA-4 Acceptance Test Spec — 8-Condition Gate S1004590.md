> Back to: [[Wiring Plan v2 — Source-Verified Integration S1004590]] · [[Wiring 01 — m16 Heartbeat Consumer (NA-4 Closure)]] · [[Wiring Gap Analysis — S1004590 Dual-Frame]] · [[HOME]] · [[MASTER_INDEX]] · `~/claude-code-workspace/the-workflow-engine/CLAUDE.local.md`
> Status: **SPEC AUTHORED, IMPLEMENTATION DEFERRED.** Authored per Plan v2 horizon item 8 (C2' acceptance gate). Test scaffolding lives in `tests/na4_acceptance_*.rs` files when implemented; specification is canonical here.

# NA-4 Acceptance Test Spec — 8-Condition Closure Gate

> Plan v2 §6 NA-4 row **must NOT update** from "mitigated structurally" → "loop-closed" until ALL 8 conditions below pass an automated test (NOT just "wire exists"). Per Zen ZA-3: source/deploy drift check (cond 0) is hard prerequisite — otherwise live-test only proves stale binary still returns lying-200.

## The 8 conditions

### Cond 0 — Source/Deploy Drift Check (Zen ZA-3, NEW)

**Test:** `tests/na4_acceptance/cond0_source_deploy_drift_check.rs` (proposed)

**Assertion:** `git rev-parse HEAD` on `~/claude-code-workspace/synthex-v2/` ≤ `mtime(bin/synthex-v2)` — running daemon binary post-dates the W1-005 honest-501 source fix (commit `c9eeb75` 2026-05-24).

**Why:** Otherwise the live smoke test (cond 3) only proves the stale binary still returns lying-200 (AP01), not that the source fix actually shipped.

**Skeleton:**
```rust
#[test]
fn cond0_source_deploy_drift_check() {
    let synthex_v2_repo = "/home/louranicas/claude-code-workspace/synthex-v2";
    let binary_path = format!("{synthex_v2_repo}/bin/synthex-v2");

    let head_commit_unix = git_commit_unix_time(synthex_v2_repo, "HEAD");
    let binary_mtime_unix = file_mtime_unix(&binary_path);

    assert!(
        binary_mtime_unix >= head_commit_unix,
        "binary mtime ({binary_mtime_unix}) predates source HEAD ({head_commit_unix}); \
         redeploy synthex-v2 before NA-4 acceptance runs — \
         otherwise smoke only proves stale binary still returns lying-200"
    );
}
```

**Pass criteria:** binary mtime ≥ HEAD commit time. **Fail criteria:** drift > 0 seconds; NA-4 acceptance MUST halt before cond 1 attempted.

---

### Cond 1 — synthex-v2 ships POST `/v3/heartbeat` endpoint at m10

**Test:** `tests/na4_acceptance/cond1_endpoint_exists.rs`

**Assertion:** `curl -X POST localhost:8092/v3/heartbeat -H 'Content-Type: application/json' -d '{}'` returns NOT 404 (anything else acceptable for cond 1 — 400/422 indicate endpoint exists but rejected our empty body, which is correct).

**Why:** CONV-1 unblock. Without this endpoint, transport always routes through `RefusalToken::Unavailable(SubstrateUnreachable)` — honest but not loop-closing.

**Skeleton:**
```rust
#[test]
fn cond1_endpoint_exists_post_v3_heartbeat() {
    let response = reqwest::blocking::Client::new()
        .post("http://localhost:8092/v3/heartbeat")
        .json(&serde_json::json!({}))
        .send()
        .expect("synthex-v2 alive at :8092");
    assert_ne!(
        response.status().as_u16(),
        404,
        "cond 1 FAILED: /v3/heartbeat returns 404; synthex-v2 has not shipped the endpoint"
    );
}
```

---

### Cond 2 — TensorSnapshot integration path chosen (NA-1'')

**Test:** `tests/na4_acceptance/cond2_tensor_integration_path.rs`

**Assertion:** synthex-v2's `m07_tensor_registry::TensorSnapshot` has either (Option A) a `wfe_clock_skew_ms` dimension (12th dim) OR (Option B) `m22 capability_trace` is wired to consume m16 alerts. Verified via `cargo check` or doctest against synthex-v2 source.

**Why:** m46 consumes `TensorSnapshot` via `tick()` — heartbeats need to influence a tensor dimension to reach m46's anomaly-detection baseline (NA-1' REFUTED + NA-1'' substituted; signal_bus path is dead end).

**Skeleton:**
```rust
#[test]
fn cond2_tensor_integration_path_present() {
    // Option A: cargo check that synthex-v2 has 12 dimensions
    let dims = synthex_v2_introspect::tensor_snapshot_dimension_count();
    // OR Option B: m22 has wfe_capability_trace consumer
    let has_m22_wfe_path = synthex_v2_introspect::m22_consumes_wfe_capability();
    assert!(
        dims >= 12 || has_m22_wfe_path,
        "cond 2 FAILED: neither Option A (12th tensor dim) nor Option B (m22 wfe capability) wired"
    );
}
```

**Pre-condition:** Luke + Zen decide Option A vs B (D7 distinct decision). This test cannot pass until that decision lands.

---

### Cond 3 — End-to-end drift detection under load (REAL gate)

**Test:** `tests/na4_acceptance/cond3_e2e_drift_detection_under_load.rs`

**Assertion:** with WFE emitting 60 heartbeats over 60s under realistic SX2 load (5 heat sources + 6 daemon tasks producing competing signals), m46's 60-sample rolling baseline on the chosen dimension (Option A or B per cond 2) exhibits a measurable drift anomaly z-score, AND m47 Critic detects the pattern within 10s of pattern onset.

**Why:** This is the **real loop-closure gate**. Wire existence (cond 1) + integration path (cond 2) are necessary but not sufficient — the actual question is "does the substrate actually observe WFE drift under realistic operating conditions?"

**Skeleton:**
```rust
#[tokio::test]
async fn cond3_e2e_drift_detection_under_load() {
    // 1. Spawn realistic SX2 load (5 heat sources + 6 daemon tasks; mock or live)
    let sx2_load = spawn_realistic_sx2_load();
    sleep(Duration::from_secs(5)).await; // warmup baseline

    // 2. WFE m16 emits 60 heartbeats over 60s with INDUCED clock skew
    let wfe_transport = HeartbeatTransport::new(
        "http://localhost:8092/v3/heartbeat".to_owned(),
        shipped_trust(),
    );
    for cycle in 0..60 {
        let envelope = induced_skew_envelope(cycle, 25); // 25ms skew (above 60-sample baseline noise)
        let _ack = wfe_transport.send(&envelope).expect("substrate accepts");
        sleep(Duration::from_secs(1)).await;
    }

    // 3. Assert m46 anomaly detection fired within 10s of pattern onset
    let observations = sx2_introspect::recent_m46_observations(Duration::from_secs(70));
    let drift_anomaly = observations.iter().find(|o| o.kind == "wfe_clock_drift");
    assert!(
        drift_anomaly.is_some(),
        "cond 3 FAILED: m46 did not detect the induced drift pattern under load \
         within the 70s window"
    );
    let detect_lag = drift_anomaly.unwrap().detected_at - induction_start_at;
    assert!(
        detect_lag <= Duration::from_secs(10),
        "cond 3 FAILED: m46 detected after {detect_lag:?}; bar is <=10s"
    );

    sx2_load.shutdown().await;
}
```

**Pre-conditions:** cond 0 + cond 1 + cond 2 all green. Requires synthex-v2 introspection API (not yet specified).

---

### Cond 4 — Bilateral V5 `WorkflowTraceParticipationStatus` ships on SX2-side

**Test:** `tests/na4_acceptance/cond4_bilateral_v5_substrate_side.rs`

**Assertion:** synthex-v2 source contains `pub enum WorkflowTraceParticipationStatus { NotShipped, Shipping, Live }` with serde derives matching the WFE-side `SubstrateParticipationStatus` shape (parity per NA-3').

**Skeleton:**
```rust
#[test]
fn cond4_bilateral_v5_substrate_side_present() {
    let synthex_v2_grep = grep_synthex_v2_source(
        "pub enum WorkflowTraceParticipationStatus"
    );
    assert!(
        !synthex_v2_grep.is_empty(),
        "cond 4 FAILED: bilateral V5 primitive not present in synthex-v2 source"
    );
}
```

---

### Cond 5 — WFE-down-mid-burst liveness contract (NA-8' + NA-10')

**Test:** `tests/na4_acceptance/cond5_wfe_down_liveness_contract.rs`

**Assertion:** (a) WFE m16 emits a `Goodbye { boot_id, final_cycle, reason }` envelope on graceful SIGTERM. (b) SX2 m10 tracks `last_heartbeat_at_ms` per `boot_id` and emits internal `WfeSilencePersisting` signal after 3× poll interval missed.

**Skeleton:**
```rust
#[test]
fn cond5a_wfe_emits_goodbye_on_sigterm() {
    let wfe_proc = spawn_wfe_test_process();
    sleep(Duration::from_secs(2)).await;
    wfe_proc.sigterm();
    // capture stcortex tail
    let final_event = stcortex_tail_until(|e| e.kind == "WfeGoodbye");
    assert!(final_event.is_some());
}

#[test]
fn cond5b_sx2_emits_silence_persisting_after_3_missed() {
    // Halt WFE; assert SX2 internal signal after 3 × poll_interval
}
```

---

### Cond 6 — Reverse-channel `m43_synthex_v2_inbound` exists in WFE

**Test:** `tests/na4_acceptance/cond6_m43_inbound_exists.rs`

**Assertion:** `src/m43_synthex_v2_inbound/` exists with `InboundServer` + `InboundHandler` types as specified in [[Wiring 02b — NexusEvent Inbound (SX2 → WFE)]].

**Skeleton:**
```rust
#[test]
fn cond6_m43_inbound_module_present() {
    assert!(std::path::Path::new("src/m43_synthex_v2_inbound/mod.rs").exists());
    // type-check at compile time
    let _ = workflow_core::m43_synthex_v2_inbound::InboundServer;
}
```

**Pre-condition:** Luke "start coding m43" phrase (held per Zen "HOLD source expansion").

---

### Cond 7 — 48h DX-Soak with the full chain LIVE

**Test:** `tests/na4_acceptance/cond7_48h_dx_soak.rs` (manual / operator-orchestrated)

**Assertion:** with cond 0-6 all green + WFE + SX2 + Watcher all live, run 48h soak per OP-3. During soak:
- ≥1 induced drift detected end-to-end (cond 3 repeated under soak load)
- ≥1 WFE-restart `boot_id` transition observed (cond 5b)
- m11 fitness decay weight measurably influenced by inbound SX2 events (cond 6 wire active)
- No silent failures, no SubstrateAuthored noise from EngineImagined fallback paths

**Why:** Closes the §6 NA-4 row honestly — wire existence ≠ operational closure.

**Pass criteria:** 48h continuous run with the 4 observable signals above all green and zero silent-failure regressions.

## Implementation tier

This spec is **authoring-only**. The actual test files (`tests/na4_acceptance/cond*.rs`) require:
1. synthex-v2 introspection API (not yet specified) — for cond 2/3/4
2. WFE-side `Goodbye` primitive in m16 (NEW; v0.3.0+) — for cond 5a
3. m43_synthex_v2_inbound module (NEW; held) — for cond 6
4. 48h soak orchestration script (operator-only) — for cond 7

The W1 transport client (shipped commit `2e9edff` 2026-05-24) is the **first piece** of the cond-3 chain — but cond 3 itself cannot run until cond 0-2 land.

## Plan v2 §6 NA-4 row update gate

The §6 row **MUST NOT** update to "loop-closed" until this acceptance test suite — all 8 conditions in sequence — passes a single end-to-end run. Honesty discipline carried recursively: the gate-on-the-gate-update is itself part of the plan.

## Persistence anchors

| Surface | Anchor |
|---|---|
| ai_docs canonical | (this spec) + Wiring Plan v2 § "Revised NA-4 closure 8-condition acceptance test" |
| Obsidian vault | `the-workflow-engine-vault/NA-4 Acceptance Test Spec — 8-Condition Gate S1004590.md` |
| Code (when implemented) | `tests/na4_acceptance/cond[0-7]_*.rs` (~8 files, ~200 LOC each) |
| stcortex | ns `workflow_trace_completion_s1004115` mem with pathway to `wiring_plan_v2_s1004590` (weight 0.95) |
| CLAUDE.local.md | top banner reference to this spec when section "NA-4 acceptance" lands |
