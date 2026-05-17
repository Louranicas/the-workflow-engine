//! `synthex-v2` — habitat autonomic regulator daemon entry point.
//!
//! Thin CLI shim. All logic lives in the library
//! ([`synthex_v2::daemon`]) so integration tests in `tests/daemon/`
//! can drive the daemon in-process without touching this binary.
//!
//! Phase 2 of `DAEMON_INTEGRATION_PLAN.md`:
//! - `clap` CLI parse
//! - `#[global_allocator] = jemalloc` behind the `jemalloc` feature
//! - `tracing-appender` non-blocking stdout writer (logging never
//!   blocks the PID tick)
//! - `habitat-pidlock` acquire + stale-PID orphan cleanup
//! - Tokio multi-thread runtime via [`synthex_v2::daemon::build_runtime`]
//! - Delegate to [`synthex_v2::daemon::run`]

#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use std::io::Write;

use clap::Parser;
use synthex_v2::daemon::{self, DaemonConfig};
use tracing::{error, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

fn main() -> std::process::ExitCode {
    let config = DaemonConfig::parse();

    let _tracing_guard = match init_tracing() {
        Ok(guard) => guard,
        Err(err) => {
            // Tracing subscriber failed to install — fall back to a direct
            // stderr write (not the `eprintln!` macro, which clippy's
            // `print_stderr` lint rightly denies). If stderr itself is
            // unavailable we surface via exit code 2 regardless.
            let _ = writeln!(std::io::stderr(), "synthex-v2: tracing init failed: {err}");
            return std::process::ExitCode::from(2);
        }
    };

    let runtime = match daemon::runtime::build_runtime(config.effective_workers()) {
        Ok(rt) => rt,
        Err(err) => {
            error!(target = "main", %err, "tokio runtime build failed");
            return std::process::ExitCode::from(3);
        }
    };

    info!(target = "main", version = env!("CARGO_PKG_VERSION"), "synthex-v2 boot");

    match runtime.block_on(daemon::run(config)) {
        Ok(()) => {
            info!(target = "main", "synthex-v2 clean exit");
            std::process::ExitCode::SUCCESS
        }
        Err(err) => {
            error!(target = "main", %err, "synthex-v2 fatal");
            std::process::ExitCode::from(1)
        }
    }
}

/// Install the tracing subscriber with a non-blocking JSON writer.
///
/// The returned guard keeps the background flusher alive for the rest of
/// the program. Dropping it flushes any queued log lines.
fn init_tracing() -> Result<tracing_appender::non_blocking::WorkerGuard, String> {
    let (non_blocking, guard) = tracing_appender::non_blocking(std::io::stdout());
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("synthex_v2=info,warn"))
        .map_err(|err| format!("env filter: {err}"))?;
    let layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(non_blocking)
        .with_target(true)
        .with_thread_ids(true)
        .with_current_span(false)
        .with_span_list(false);
    tracing_subscriber::registry()
        .with(env_filter)
        .with(layer)
        .try_init()
        .map_err(|err| format!("subscriber init: {err}"))?;
    Ok(guard)
}
