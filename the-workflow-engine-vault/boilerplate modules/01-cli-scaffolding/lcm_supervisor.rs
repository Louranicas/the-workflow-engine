//! `lcm-supervisor` daemon — session-scoped loop supervisor.
//!
//! # Startup
//!
//! 1. Validates `LCM_VERIFIER_HMAC_KEY` (P3.2 gate).
//! 2. Resolves the UDS socket path from `$XDG_RUNTIME_DIR/lcm/supervisor.sock`
//!    (same resolution as [`SupervisorRpcClient::default_socket`]).
//! 3. Performs singleton-acquire: attempt connect first; if connection succeeds another
//!    supervisor is running → exit 1. If connect fails, remove stale socket and bind.
//! 4. Sets socket permissions to `0o600` (owner-only).
//! 5. Runs a Tokio accept loop, spawning one task per connection.
//!
//! # JSON-RPC 2.0 dispatch
//!
//! Each connection is newline-framed (one request per `\n`-terminated line, one
//! response per `\n`-terminated line). A 30-second read timeout guards against
//! stalled clients.
//!
//! # Graceful shutdown (SIGTERM)
//!
//! Installs a SIGTERM handler; on receipt calls `Supervisor::drain_shutdown` and
//! then exits. In-flight connection tasks each have their own 30-second read budget;
//! the supervisor does not forcibly kill them — the kernel cleans up on process exit.
//!
//! # RPC methods
//!
//! | Method                | Supervisor call              |
//! |-----------------------|------------------------------|
//! | `lcm.loop.create`     | `Supervisor::loop_create`    |
//! | `lcm.loop.cancel`     | `Supervisor::loop_cancel`    |
//! | `lcm.loop.status`     | `Supervisor::loop_status`    |
//! | `lcm.follow.start`    | SSE broker subscribe (stub)  |
//! | `lcm.follow.stop`     | no-op (stub)                 |
//! | `lcm.receipt.get`     | stub (M0)                    |
//! | `lcm.gate.check`      | stub (M0)                    |
//! | `lcm.schedule.register` | stub (M0)                  |
//! | `lcm.ping`            | health probe                 |

use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use serde::Deserialize;
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

use lcm::m01_types::m02_ids::{HelloEnvelope, LoopId};
use lcm::m01_types::m03_error_codes::ErrorCode;
use lcm::m01_types::m04_lcm_error::LcmError;
use lcm::m02_substrate::m09_config_toml::LcmConfig;
use lcm::m02_substrate::m13_receipt_internal::HmacKeyStore;
use lcm::m04_core::m20_loop_trait::LoopSpec;
use lcm::m06_supervisor::m27_supervisor_lifecycle::{
    LifecyclePhase, LoopCreateContext, Supervisor, SupervisorBuilder,
};
use lcm::m06_supervisor::m28_singleton::FileSingleton;
use lcm::m06_supervisor::m29_parent_death_watch::DefaultParentDeathWatch;
use lcm::m06_supervisor::m30_scope_resolver::DefaultScopeResolver;
use lcm::m06_supervisor::m31_sse_broker::{DefaultSseBroker, InMemoryEventSource};
use lcm::m07_adapter::m32_adapter_claude_mcp::JsonRpcResponse;

// ─── Constants ────────────────────────────────────────────────────────────────

/// Read timeout per connection (mirrors F-LCM-08).
const READ_TIMEOUT: Duration = Duration::from_secs(30);

/// Version string embedded in ping response.
const DAEMON_VERSION: &str = env!("CARGO_PKG_VERSION");

// ─── JSON-RPC request shape ───────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct RpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Value,
    method: String,
    #[serde(default)]
    params: Value,
}

// ─── Singleton acquire ────────────────────────────────────────────────────────

/// Resolve the supervisor socket path (mirrors `SupervisorRpcClient::default_socket`).
fn resolve_socket_path() -> PathBuf {
    let uid = current_uid();
    let base = std::env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| format!("/run/user/{uid}"));
    PathBuf::from(base).join("lcm").join("supervisor.sock")
}

/// Read effective UID from `/proc/self/status`.
fn current_uid() -> u32 {
    if let Ok(contents) = std::fs::read_to_string("/proc/self/status") {
        for line in contents.lines() {
            if let Some(rest) = line.strip_prefix("Uid:") {
                if let Some(uid_str) = rest.split_whitespace().next() {
                    if let Ok(uid) = uid_str.parse::<u32>() {
                        return uid;
                    }
                }
            }
        }
    }
    std::env::var("UID")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

/// Singleton-acquire: bind the UDS socket with stale-socket recovery.
///
/// Returns `Ok(listener)` when the socket is exclusively bound.
/// Returns `Err` when another supervisor is already running.
async fn singleton_bind(socket_path: &Path) -> Result<UnixListener, String> {
    // Ensure parent directory exists.
    if let Some(parent) = socket_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return Err(format!(
                "cannot create socket directory {}: {e}",
                parent.display()
            ));
        }
    }

    // Try to connect first — if it succeeds, another supervisor owns the socket.
    if socket_path.exists() {
        match tokio::net::UnixStream::connect(socket_path).await {
            Ok(_) => {
                return Err(format!(
                    "supervisor already running at {}",
                    socket_path.display()
                ));
            }
            Err(_) => {
                // Stale socket — remove it before binding.
                if let Err(e) = std::fs::remove_file(socket_path) {
                    return Err(format!(
                        "failed to remove stale socket {}: {e}",
                        socket_path.display()
                    ));
                }
            }
        }
    }

    let listener = UnixListener::bind(socket_path)
        .map_err(|e| format!("bind({}) failed: {e}", socket_path.display()))?;

    // Restrict to owner-only (0600).
    std::fs::set_permissions(socket_path, std::fs::Permissions::from_mode(0o600))
        .map_err(|e| format!("chmod 0600 {} failed: {e}", socket_path.display()))?;

    Ok(listener)
}

// ─── Write PID file ────────────────────────────────────────────────────────────

fn write_pid_file() {
    let pid_path = {
        let base = std::env::var("XDG_RUNTIME_DIR")
            .ok()
            .map(PathBuf::from)
            .or_else(dirs::runtime_dir)
            .unwrap_or_else(|| PathBuf::from("/tmp"));
        base.join("lcm").join("supervisor.pid")
    };
    if let Some(parent) = pid_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let pid = std::process::id();
    let _ = std::fs::write(&pid_path, format!("{pid}\n"));
}

fn remove_pid_file() {
    let base = std::env::var("XDG_RUNTIME_DIR")
        .ok()
        .map(PathBuf::from)
        .or_else(dirs::runtime_dir)
        .unwrap_or_else(|| PathBuf::from("/tmp"));
    let path = base.join("lcm").join("supervisor.pid");
    let _ = std::fs::remove_file(path);
}

// ─── Per-connection handler ────────────────────────────────────────────────────

/// Handle one incoming client connection.
async fn handle_connection(stream: UnixStream, supervisor: Arc<Supervisor>) {
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    // Read one request line, with a 30-second timeout.
    let line_result = tokio::time::timeout(READ_TIMEOUT, lines.next_line()).await;

    let line = match line_result {
        Ok(Ok(Some(l))) => l,
        Ok(Ok(None)) => return, // EOF — client disconnected
        Ok(Err(e)) => {
            // IO error
            let resp = error_response(&Value::Null, -32700, format!("read error: {e}"));
            let _ = write_value(&mut writer, &resp).await;
            return;
        }
        Err(_elapsed) => {
            let resp = error_response(&Value::Null, -32000, "read timeout after 30s");
            let _ = write_value(&mut writer, &resp).await;
            return;
        }
    };

    if line.is_empty() {
        return;
    }

    let req: RpcRequest = match serde_json::from_str(&line) {
        Ok(r) => r,
        Err(e) => {
            let resp = error_response(&Value::Null, -32700, format!("parse error: {e}"));
            let _ = write_value(&mut writer, &resp).await;
            return;
        }
    };

    let id = req.id.clone();
    let result = dispatch_method(&supervisor, &req.method, req.params).await;

    let response = match result {
        Ok(val) => JsonRpcResponse::ok(id, val),
        Err(ref e) => JsonRpcResponse::err(id, e),
    };

    let _ = write_response(&mut writer, &response).await;
}

/// Write any `serde_json::Value` as a newline-terminated JSON line.
async fn write_value(
    writer: &mut tokio::net::unix::OwnedWriteHalf,
    value: &Value,
) -> std::io::Result<()> {
    let mut bytes = serde_json::to_vec(value).unwrap_or_default();
    bytes.push(b'\n');
    writer.write_all(&bytes).await?;
    writer.flush().await
}

/// Write a `JsonRpcResponse` as a newline-terminated JSON line.
async fn write_response(
    writer: &mut tokio::net::unix::OwnedWriteHalf,
    response: &JsonRpcResponse,
) -> std::io::Result<()> {
    let val = serde_json::to_value(response).unwrap_or(Value::Null);
    write_value(writer, &val).await
}

/// Build a raw error response (used before full request parsing succeeds).
fn error_response(id: &Value, code: i32, message: impl Into<String>) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message.into(),
        }
    })
}

// ─── Method dispatch ──────────────────────────────────────────────────────────

async fn dispatch_method(
    supervisor: &Supervisor,
    method: &str,
    params: Value,
) -> Result<Value, LcmError> {
    match method {
        "lcm.ping" => Ok(handle_ping()),
        "lcm.loop.create" => handle_loop_create(supervisor, &params).await,
        "lcm.loop.cancel" => handle_loop_cancel(supervisor, &params).await,
        "lcm.loop.status" => handle_loop_status(supervisor, &params).await,
        "lcm.follow.start" => handle_follow_start(supervisor, &params).await,
        "lcm.follow.stop" => Ok(handle_follow_stop()),
        "lcm.receipt.get" => handle_receipt_get(&params),
        "lcm.gate.check" => handle_gate_check(&params),
        "lcm.schedule.register" => Ok(handle_schedule_register(&params)),
        "lcm.supervisor.status" => handle_supervisor_status(supervisor).await,
        other => Err(LcmError::Other {
            code: ErrorCode::MethodNotFound,
            detail: format!("unknown method: {other}"),
        }),
    }
}

// ─── Handler implementations ──────────────────────────────────────────────────

fn handle_ping() -> Value {
    serde_json::json!({
        "ok": true,
        "version": DAEMON_VERSION,
    })
}

async fn handle_loop_create(supervisor: &Supervisor, params: &Value) -> Result<Value, LcmError> {
    // Extract caller_id, name, max_iters, survives_session_death.
    let caller_session = params
        .get("caller_id")
        .and_then(Value::as_str)
        .unwrap_or("anonymous")
        .to_string();

    let name = params
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("unnamed")
        .to_string();

    let max_iters = params
        .get("max_iters")
        .and_then(Value::as_u64)
        .and_then(|v| u32::try_from(v).ok())
        .unwrap_or(1000_u32);

    let survives = params
        .get("survives_session_death")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let spec = LoopSpec::builder(&name)
        .max_iters(max_iters)
        .survives_session_death(survives)
        .build();

    let envelope = HelloEnvelope::new(current_uid(), std::process::id(), Some(caller_session));

    // F4 discipline (S1001883): route through SystemWallClock — the sole
    // sanctioned entry to SystemTime per m07_clock_wall.rs quarantine.
    let wall_clock = lcm::m02_substrate::m07_clock_wall::SystemWallClock::new();
    let now_nanos = <lcm::m02_substrate::m07_clock_wall::SystemWallClock as lcm::m02_substrate::m07_clock_wall::WallClock>::now_unix_nanos(&wall_clock);

    let ctx = LoopCreateContext { envelope, now_nanos };

    let loop_id = supervisor.loop_create(ctx, spec).await?;
    Ok(serde_json::json!({ "loop_id": loop_id.to_string() }))
}

async fn handle_loop_cancel(supervisor: &Supervisor, params: &Value) -> Result<Value, LcmError> {
    let loop_id = extract_loop_id(params)?;
    supervisor.loop_cancel(loop_id).await?;
    Ok(serde_json::json!({ "cancelled": true, "loop_id": loop_id.to_string() }))
}

async fn handle_loop_status(supervisor: &Supervisor, params: &Value) -> Result<Value, LcmError> {
    let loop_id = extract_loop_id(params)?;
    let report = supervisor.loop_status(loop_id).await?;
    Ok(serde_json::json!({
        "loop_id": report.loop_id.to_string(),
        "state": format!("{:?}", report.state),
        "created_at_unix_nanos": report.created_at_unix_nanos,
    }))
}

async fn handle_follow_start(supervisor: &Supervisor, params: &Value) -> Result<Value, LcmError> {
    // M0 stub: validate loop_id exists, return empty events.
    let loop_id = extract_loop_id(params)?;
    supervisor.loop_status(loop_id).await?;
    Ok(serde_json::json!({
        "loop_id": loop_id.to_string(),
        "events": [],
        "note": "SSE follow is a stub in M0; events are empty",
    }))
}

fn handle_follow_stop() -> Value {
    serde_json::json!({ "stopped": true })
}

fn handle_receipt_get(params: &Value) -> Result<Value, LcmError> {
    let hash = params
        .get("hash")
        .and_then(Value::as_str)
        .ok_or_else(|| LcmError::Other {
            code: ErrorCode::InvalidParams,
            detail: "receipt.get requires 'hash' param".into(),
        })?;
    Ok(serde_json::json!({
        "hash": hash,
        "status": "not_found",
        "note": "receipt store not wired in M0",
    }))
}

fn handle_gate_check(params: &Value) -> Result<Value, LcmError> {
    let loop_id = params
        .get("loop_id")
        .and_then(Value::as_str)
        .ok_or_else(|| LcmError::Other {
            code: ErrorCode::InvalidParams,
            detail: "gate.check requires 'loop_id' param".into(),
        })?;
    Ok(serde_json::json!({
        "loop_id": loop_id,
        "has_final_claim": false,
        "note": "claim authority not wired in M0",
    }))
}

fn handle_schedule_register(params: &Value) -> Value {
    let expr = params
        .get("expr")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    serde_json::json!({
        "registered": true,
        "expr": expr,
        "note": "scheduler not wired in M0",
    })
}

async fn handle_supervisor_status(supervisor: &Supervisor) -> Result<Value, LcmError> {
    let phase = supervisor.phase().await;
    let loop_count = supervisor.loop_count();
    Ok(serde_json::json!({
        "phase": phase.to_string(),
        "loop_count": loop_count,
        "version": DAEMON_VERSION,
    }))
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Extract `loop_id` from params as a hex string and parse it.
fn extract_loop_id(params: &Value) -> Result<LoopId, LcmError> {
    let s = params
        .get("loop_id")
        .and_then(Value::as_str)
        .ok_or_else(|| LcmError::Other {
            code: ErrorCode::InvalidParams,
            detail: "params requires 'loop_id' string".into(),
        })?;

    LoopId::from_hex(s).map_err(|_| LcmError::LoopNotFound {
        code: ErrorCode::LoopNotFound,
        id_short: s[..s.len().min(8)].to_string(),
    })
}

// ─── Accept loop ──────────────────────────────────────────────────────────────

/// Accept connections from `listener` until `shutdown` is notified.
async fn accept_loop(
    listener: UnixListener,
    supervisor: Arc<Supervisor>,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
) {
    loop {
        tokio::select! {
            accept_result = listener.accept() => {
                match accept_result {
                    Ok((stream, _addr)) => {
                        let sv = Arc::clone(&supervisor);
                        tokio::spawn(handle_connection(stream, sv));
                    }
                    Err(e) => {
                        eprintln!("lcm-supervisor: accept error: {e}");
                        // Brief back-off on transient errors.
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                }
            }
            _ = shutdown.changed() => {
                if *shutdown.borrow() {
                    break;
                }
            }
        }
    }
}

// ─── main ─────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    // ── P3.2 gate: HMAC key ─────────────────────────────────────────────────
    let _keystore = match HmacKeyStore::from_env() {
        Ok(ks) => ks,
        Err(e) => {
            eprintln!(
                "lcm-supervisor: FATAL — LCM_VERIFIER_HMAC_KEY missing or invalid: {e}\n\
                 Set LCM_VERIFIER_HMAC_KEY to a 64-character lowercase hex string (32 bytes).\n\
                 Example: export LCM_VERIFIER_HMAC_KEY=$(openssl rand -hex 32)"
            );
            std::process::exit(1);
        }
    };
    eprintln!("lcm-supervisor: LCM_VERIFIER_HMAC_KEY loaded (P3.2 gate passed)");

    // ── Logging ──────────────────────────────────────────────────────────────
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .try_init();

    // ── Socket path ──────────────────────────────────────────────────────────
    let socket_path = resolve_socket_path();
    eprintln!("lcm-supervisor: socket path = {}", socket_path.display());

    // ── Singleton acquire ────────────────────────────────────────────────────
    let listener = match singleton_bind(&socket_path).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("lcm-supervisor: {e}");
            std::process::exit(1);
        }
    };

    // ── Build Supervisor ─────────────────────────────────────────────────────
    let config = Arc::new(LcmConfig::default());
    let singleton: Arc<dyn lcm::m06_supervisor::m28_singleton::Singleton> =
        match FileSingleton::new() {
            Ok(s) => Arc::new(s),
            Err(e) => {
                eprintln!("lcm-supervisor: failed to build singleton identity: {e}");
                std::process::exit(1);
            }
        };
    let death_watch = Arc::new(DefaultParentDeathWatch::new());
    let scope_resolver = Arc::new(DefaultScopeResolver::new());
    let event_src = Arc::new(InMemoryEventSource::new(4096));
    let sse_broker = Arc::new(DefaultSseBroker::new(event_src));

    let supervisor = Arc::new(
        SupervisorBuilder::new(config, singleton, death_watch, scope_resolver, sse_broker).build(),
    );

    // Advance to Running phase.
    if let Err(e) = supervisor.set_phase(LifecyclePhase::Running).await {
        eprintln!("lcm-supervisor: failed to enter Running phase: {e}");
        std::process::exit(1);
    }

    // ── Write PID file ───────────────────────────────────────────────────────
    write_pid_file();
    eprintln!(
        "lcm-supervisor: running (PID {}, socket={})",
        std::process::id(),
        socket_path.display()
    );

    // ── Shutdown channel ─────────────────────────────────────────────────────
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    // ── SIGTERM handler ──────────────────────────────────────────────────────
    let sv_for_signal = Arc::clone(&supervisor);
    let socket_path_for_signal = socket_path.clone();
    tokio::spawn(async move {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut stream) => {
                stream.recv().await;
            }
            Err(e) => {
                eprintln!("lcm-supervisor: cannot install SIGTERM handler: {e}");
                // Wait forever so the accept loop continues running.
                std::future::pending::<()>().await;
                return;
            }
        }
        eprintln!("lcm-supervisor: SIGTERM received — draining");
        let _ = sv_for_signal.drain_shutdown().await;
        let _ = std::fs::remove_file(&socket_path_for_signal);
        remove_pid_file();
        let _ = shutdown_tx.send(true);
    });

    // ── Accept loop ──────────────────────────────────────────────────────────
    accept_loop(listener, supervisor, shutdown_rx).await;

    // Cleanup on normal exit.
    let _ = std::fs::remove_file(&socket_path);
    remove_pid_file();
    eprintln!("lcm-supervisor: shutdown complete");
}
