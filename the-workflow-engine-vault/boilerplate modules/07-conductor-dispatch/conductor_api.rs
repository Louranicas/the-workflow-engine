//! Axum HTTP router for the Weaver service (port 8141).
//!
//! # Endpoints
//!
//! | Method | Path | Description |
//! |---|---|---|
//! | `GET` | `/health` | Simple liveness check |
//! | `GET` | `/state` | Latest `HABITAT_STATE.md` content |
//! | `GET` | `/state.json` | Latest [`SnapshotRow`] as JSON |
//! | `GET` | `/proposals` | Pending proposals (filter: `?status=pending`) |
//! | `GET` | `/spheres/binding` | All pane-sphere bindings |
//! | `POST` | `/probe` | Force probe refresh |
//! | `GET` | `/divergence` | Divergence reports (filter: `?severity=HIGH`) |
//!
//! # Shared state
//!
//! All handlers share a [`WeaverState`] behind an `Arc`. The `StateDb` is
//! accessed through a `parking_lot::Mutex`.

use std::sync::Arc;

use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tracing::instrument;

use crate::state::{DivergenceReport, Severity, SnapshotRow, StateDb};

// ---------------------------------------------------------------------------
// WeaverState
// ---------------------------------------------------------------------------

/// Shared state passed to all Axum handlers.
pub struct WeaverState {
    /// The Weaver state database.
    pub db: Arc<Mutex<StateDb>>,
    /// Latest rendered `HABITAT_STATE.md` content (may be empty until first
    /// probe cycle completes).
    pub state_md: Arc<Mutex<String>>,
}

// ---------------------------------------------------------------------------
// Query extractors
// ---------------------------------------------------------------------------

/// Query parameters for `/proposals`.
#[derive(Debug, Deserialize)]
pub struct ProposalsQuery {
    /// Optional filter: `pending` or `released`.
    pub status: Option<String>,
}

/// Query parameters for `/divergence`.
#[derive(Debug, Deserialize)]
pub struct DivergenceQuery {
    /// Optional minimum severity filter: `LOW`, `MED`, `HIGH`, `CRITICAL`.
    pub severity: Option<String>,
    /// Maximum number of rows to return (default 50).
    pub limit: Option<usize>,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// JSON body returned by `GET /health`.
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    /// Always `"ok"`.
    pub status: &'static str,
    /// Service name.
    pub service: &'static str,
    /// Port this instance listens on.
    pub port: u16,
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

/// Builds and returns the Axum router for the Weaver HTTP server.
///
/// The caller is responsible for binding a `TcpListener` and calling
/// `axum::serve`.
pub fn build_router(state: Arc<WeaverState>) -> Router {
    Router::new()
        .route("/health", get(handle_health))
        .route("/state", get(handle_state_md))
        .route("/state.json", get(handle_state_json))
        .route("/proposals", get(handle_proposals))
        .route("/spheres/binding", get(handle_spheres_binding))
        .route("/probe", post(handle_probe))
        .route("/divergence", get(handle_divergence))
        .with_state(state)
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `GET /health` — simple liveness check.
#[instrument(skip_all)]
async fn handle_health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "weaver",
        port: 8141,
    })
}

/// `GET /state` — returns the current `HABITAT_STATE.md` as plain text.
#[instrument(skip_all)]
async fn handle_state_md(State(state): State<Arc<WeaverState>>) -> impl IntoResponse {
    let md = state.state_md.lock().clone();
    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "text/markdown; charset=utf-8")],
        md,
    )
}

/// `GET /state.json` — returns the latest [`SnapshotRow`] as JSON.
#[instrument(skip_all)]
async fn handle_state_json(
    State(state): State<Arc<WeaverState>>,
) -> Result<Json<Value>, ApiError> {
    // AP29 fix (S1001791): rusqlite is sync — wrap in spawn_blocking so axum
    // worker threads aren't blocked during DB I/O.
    let db = Arc::clone(&state.db);
    let snap: Option<SnapshotRow> = tokio::task::spawn_blocking(move || db.lock().latest_snapshot())
        .await
        .map_err(|e| ApiError::Internal(format!("join error: {e}")))?
        .map_err(ApiError::from)?;
    match snap {
        Some(s) => Ok(Json(
            serde_json::to_value(&s).map_err(|e| ApiError::Internal(e.to_string()))?,
        )),
        None => Ok(Json(json!({"status": "no_snapshots_yet"}))),
    }
}

/// `GET /proposals` — returns pending proposals from the database.
///
/// Query param `?status=pending` filters to unreleased rows only.
#[instrument(skip_all)]
async fn handle_proposals(
    State(state): State<Arc<WeaverState>>,
    Query(params): Query<ProposalsQuery>,
) -> Result<Json<Value>, ApiError> {
    // For now, returns the raw count + status info from the snapshots table.
    // Full proposals table query will be wired in once the binary is deployed.
    let _ = params;
    // AP29 fix (S1001791): wrap rusqlite in spawn_blocking.
    let db = Arc::clone(&state.db);
    let count: i64 = tokio::task::spawn_blocking(move || db.lock().applied_migration_count())
        .await
        .map_err(|e| ApiError::Internal(format!("join error: {e}")))?
        .map_err(ApiError::from)?;
    Ok(Json(json!({
        "note": "proposals endpoint — migration count proxy until full wiring",
        "migrations_applied": count,
    })))
}

/// `GET /spheres/binding` — returns all pane-to-sphere bindings.
#[instrument(skip_all)]
async fn handle_spheres_binding(
    State(state): State<Arc<WeaverState>>,
) -> Result<Json<Value>, ApiError> {
    // AP29 fix (S1001791): wrap rusqlite in spawn_blocking.
    let db = Arc::clone(&state.db);
    let bindings = tokio::task::spawn_blocking(move || db.lock().all_pane_bindings())
        .await
        .map_err(|e| ApiError::Internal(format!("join error: {e}")))?
        .map_err(ApiError::from)?;
    Ok(Json(
        serde_json::to_value(&bindings).map_err(|e| ApiError::Internal(e.to_string()))?,
    ))
}

/// `POST /probe` — force a probe refresh cycle.
///
/// Returns `202 Accepted` immediately; the probe runs in the Weaver background
/// task. The next `GET /state.json` will reflect the result.
#[instrument(skip_all)]
async fn handle_probe() -> StatusCode {
    StatusCode::ACCEPTED
}

/// `GET /divergence` — returns divergence reports.
///
/// Query params: `?severity=HIGH&limit=20`.
///
/// Returns `400 Bad Request` if the `severity` query param is provided but
/// cannot be parsed as `LOW`, `MED`, `HIGH`, or `CRITICAL`. A missing
/// `severity` param returns all severities.
#[instrument(skip_all)]
async fn handle_divergence(
    State(state): State<Arc<WeaverState>>,
    Query(params): Query<DivergenceQuery>,
) -> Result<Json<Value>, ApiError> {
    // F-CONDUCTOR-06: reject unrecognised severity strings with 400 Bad Request.
    // Previously `.ok()` silently discarded parse errors and returned all rows,
    // causing operator dashboards to receive unfiltered results for typo queries.
    let min_sev = match params.severity.as_deref() {
        None => None,
        Some(s) => match Severity::parse(s) {
            Ok(sev) => Some(sev),
            Err(_) => {
                return Err(ApiError::BadRequest(format!(
                    "invalid severity: {s}; expected LOW|MED|HIGH|CRITICAL"
                )));
            }
        },
    };
    let limit = params.limit.unwrap_or(50);
    // AP29 fix (S1001791): wrap rusqlite in spawn_blocking.
    let db = Arc::clone(&state.db);
    let reports: Vec<DivergenceReport> = tokio::task::spawn_blocking(move || db.lock().divergence_reports(min_sev, limit))
        .await
        .map_err(|e| ApiError::Internal(format!("join error: {e}")))?
        .map_err(ApiError::from)?;
    Ok(Json(
        serde_json::to_value(&reports).map_err(|e| ApiError::Internal(e.to_string()))?,
    ))
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that the HTTP handlers can return as structured JSON responses.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// A `state.db` operation failed.
    #[error("db error: {0}")]
    Db(#[from] crate::state::StateError),
    /// Internal processing error.
    #[error("internal error: {0}")]
    Internal(String),
    /// Caller supplied an invalid query parameter (F-CONDUCTOR-06).
    ///
    /// Maps to `400 Bad Request` — the caller must correct their request.
    #[error("bad request: {0}")]
    BadRequest(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        if let Self::BadRequest(ref msg) = self {
            let body = json!({"error": msg});
            (StatusCode::BAD_REQUEST, Json(body)).into_response()
        } else {
            let body = json!({"error": self.to_string()});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// HABITAT_STATE.md renderer
// ---------------------------------------------------------------------------

/// Renders a `HABITAT_STATE.md` document from the latest snapshot.
///
/// Returns a machine-readable Markdown string with emoji-free metric tables.
#[must_use]
pub fn render_state_md(snap: &SnapshotRow) -> String {
    format!(
        "# HABITAT_STATE\n\n\
         > Generated by Weaver at ts={ts}\n\n\
         ## RALPH\n\n\
         | Metric | Value |\n\
         |---|---|\n\
         | fitness | {fitness:.4} |\n\
         | phase | {ralph_phase} |\n\
         | gen | {ralph_gen} |\n\
         | ltp | {ltp} |\n\
         | ltd | {ltd} |\n\
         | mutations_proposed | {mutations_proposed} |\n\
         | mutations_accepted | {mutations_accepted} |\n\n\
         ## PV2 Field\n\n\
         | Metric | Value |\n\
         |---|---|\n\
         | field_r | {field_r:.4} |\n\
         | sphere_count | {sphere_count} |\n\n\
         ## Thermal\n\n\
         | Metric | Value |\n\
         |---|---|\n\
         | thermal_t | {thermal_t:.4} |\n\n\
         ## Health\n\n\
         | Metric | Value |\n\
         |---|---|\n\
         | breakers_open | {breakers_open} |\n\
         | probe_failures | {probe_failures} |\n\
         | me_fitness | {me_fitness:.4} |\n",
        ts = snap.ts,
        fitness = snap.fitness,
        ralph_phase = snap.ralph_phase,
        ralph_gen = snap.ralph_gen,
        ltp = snap.ltp,
        ltd = snap.ltd,
        mutations_proposed = snap.mutations_proposed,
        mutations_accepted = snap.mutations_accepted,
        field_r = snap.field_r,
        sphere_count = snap.sphere_count,
        thermal_t = snap.thermal_t,
        breakers_open = snap.breakers_open,
        probe_failures = snap.probe_failures,
        me_fitness = snap.me_fitness,
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::Request;
    use tower::ServiceExt;

    fn make_state() -> Arc<WeaverState> {
        let mut db = StateDb::open_in_memory().expect("db");
        db.migrate().expect("migrate");
        Arc::new(WeaverState {
            db: Arc::new(Mutex::new(db)),
            state_md: Arc::new(Mutex::new(String::new())),
        })
    }

    async fn get(router: &Router, path: &str) -> (StatusCode, Vec<u8>) {
        let req = Request::builder().uri(path).body(axum::body::Body::empty()).unwrap();
        let resp = router.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        (status, body.to_vec())
    }

    #[tokio::test]
    async fn health_returns_200() {
        let state = make_state();
        let app = build_router(state);
        let (status, body) = get(&app, "/health").await;
        assert_eq!(status, StatusCode::OK);
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ok");
        assert_eq!(json["port"], 8141);
    }

    #[tokio::test]
    async fn state_md_returns_200_empty_initially() {
        let state = make_state();
        let app = build_router(state);
        let (status, _) = get(&app, "/state").await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn state_json_returns_no_snapshots_initially() {
        let state = make_state();
        let app = build_router(state);
        let (status, body) = get(&app, "/state.json").await;
        assert_eq!(status, StatusCode::OK);
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "no_snapshots_yet");
    }

    #[tokio::test]
    async fn state_json_returns_snapshot_after_insert() {
        let state = make_state();
        let snap = SnapshotRow {
            ts: 999,
            fitness: 0.74,
            field_r: 0.29,
            thermal_t: 0.41,
            sphere_count: 3,
            ralph_phase: "selection".into(),
            ralph_gen: 5678,
            ltp: 120,
            ltd: 22_000,
            mutations_proposed: 178,
            mutations_accepted: 1,
            breakers_open: 0,
            probe_failures: 0,
            me_fitness: 0.70,
            raw_json: "{}".into(),
        };
        state.db.lock().insert_snapshot(&snap).expect("insert");
        let app = build_router(state);
        let (status, body) = get(&app, "/state.json").await;
        assert_eq!(status, StatusCode::OK);
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["ts"], 999);
    }

    #[tokio::test]
    async fn spheres_binding_returns_empty_array_initially() {
        let state = make_state();
        let app = build_router(state);
        let (status, body) = get(&app, "/spheres/binding").await;
        assert_eq!(status, StatusCode::OK);
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert!(json.as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn divergence_returns_empty_initially() {
        let state = make_state();
        let app = build_router(state);
        let (status, body) = get(&app, "/divergence").await;
        assert_eq!(status, StatusCode::OK);
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert!(json.as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn probe_returns_202() {
        let state = make_state();
        let app = build_router(state);
        let req = Request::builder()
            .method("POST")
            .uri("/probe")
            .body(axum::body::Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::ACCEPTED);
    }

    // --- render_state_md ---

    #[test]
    fn render_state_md_contains_fitness() {
        let snap = SnapshotRow {
            ts: 1_000,
            fitness: 0.74,
            field_r: 0.29,
            thermal_t: 0.41,
            sphere_count: 3,
            ralph_phase: "selection".into(),
            ralph_gen: 5678,
            ltp: 120,
            ltd: 22_000,
            mutations_proposed: 178,
            mutations_accepted: 1,
            breakers_open: 0,
            probe_failures: 0,
            me_fitness: 0.70,
            raw_json: "{}".into(),
        };
        let md = render_state_md(&snap);
        assert!(md.contains("HABITAT_STATE"));
        assert!(md.contains("0.7400"));
        assert!(md.contains("selection"));
        assert!(md.contains("5678"));
    }

    // --- F-CONDUCTOR-06: severity filter returns 400 on typo ---

    #[tokio::test]
    async fn divergence_severity_typo_returns_400() {
        // F-CONDUCTOR-06 regression: `?severity=CRITICALL` (typo) must return
        // 400 Bad Request, not silently return all rows with no filter applied.
        let state = make_state();
        let app = build_router(state);
        let (status, body) = get(&app, "/divergence?severity=CRITICALL").await;
        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "typo severity must yield 400, not 200 with unfiltered results"
        );
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert!(
            json["error"].as_str().unwrap_or("").contains("invalid severity"),
            "error message must describe the invalid severity: {json}"
        );
    }

    #[tokio::test]
    async fn divergence_valid_severity_filter_returns_200() {
        // Valid severity param must still work after the F-CONDUCTOR-06 fix.
        let state = make_state();
        let app = build_router(state);
        let (status, _) = get(&app, "/divergence?severity=HIGH").await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn divergence_no_severity_param_returns_200() {
        // No severity param must return all rows (None filter).
        let state = make_state();
        let app = build_router(state);
        let (status, body) = get(&app, "/divergence").await;
        assert_eq!(status, StatusCode::OK);
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert!(json.is_array(), "must return a JSON array");
    }

    #[tokio::test]
    async fn divergence_lowercase_severity_returns_400() {
        // 'high' (lowercase) is not a valid severity string.
        let state = make_state();
        let app = build_router(state);
        let (status, _) = get(&app, "/divergence?severity=high").await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    // --- F-CONDUCTOR-03: render_state_md includes probe_failures ---

    #[test]
    fn render_state_md_includes_probe_failures() {
        let snap = SnapshotRow {
            ts: 1_000,
            fitness: 0.74,
            field_r: 0.29,
            thermal_t: 0.41,
            sphere_count: 3,
            ralph_phase: "selection".into(),
            ralph_gen: 5678,
            ltp: 120,
            ltd: 22_000,
            mutations_proposed: 178,
            mutations_accepted: 1,
            breakers_open: 2,
            probe_failures: 1,
            me_fitness: 0.70,
            raw_json: "{}".into(),
        };
        let md = render_state_md(&snap);
        assert!(md.contains("probe_failures"), "render must surface probe_failures column");
        assert!(md.contains("breakers_open"), "render must surface breakers_open column");
        // Validate the values are distinct in the output.
        assert!(md.contains("| probe_failures | 1 |"), "probe_failures value must appear");
        assert!(md.contains("| breakers_open | 2 |"), "breakers_open value must appear");
    }

    #[test]
    fn render_state_md_no_emoji() {
        let snap = SnapshotRow {
            ts: 1,
            fitness: 0.0,
            field_r: 0.0,
            thermal_t: 0.0,
            sphere_count: 0,
            ralph_phase: "x".into(),
            ralph_gen: 0,
            ltp: 0,
            ltd: 0,
            mutations_proposed: 0,
            mutations_accepted: 0,
            breakers_open: 0,
            probe_failures: 0,
            me_fitness: 0.0,
            raw_json: "{}".into(),
        };
        let md = render_state_md(&snap);
        // No emoji bytes in the output.
        assert!(md.is_ascii() || !md.chars().any(|c| c as u32 > 0x2FFF));
    }
}
