//! Graceful shutdown orchestration with per-task budgets (R15, P0-4).
//!
//! Shutdown is driven by a single [`CancellationToken`] the daemon shares
//! with every spawned task. On signal the token is cancelled; tasks are
//! expected to observe it within their `.await` budget and exit cleanly.
//! Any task that over-runs its budget is abandoned — we prefer correct
//! shutdown of the critical plane over waiting for a stuck dependency.
//!
//! Budgets (from `DAEMON_INTEGRATION_PLAN.md` Phase 2):
//!
//! | Plane                  | Budget  | Rationale                              |
//! |------------------------|---------|----------------------------------------|
//! | Ingest                 |  5 s    | Drain in-flight HTTP poll responses    |
//! | Regulation             |  2 s    | PID tick must not starve shutdown      |
//! | Memory consolidation   | 10 s    | Flush working tier to POVM/STDB        |
//! | Watcher                | 30 s    | In-flight shadow tests may be long     |
//! | HTTP server            |  2 s    | Drain axum connections                 |

use std::time::Duration;

use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

/// Per-task shutdown budget table.
#[derive(Debug, Clone, Copy)]
pub struct ShutdownBudgets {
    /// Drain in-flight ingest poll responses.
    pub ingest: Duration,
    /// PID-tick drain.
    pub regulation: Duration,
    /// Working-tier → POVM/STDB flush.
    pub memory: Duration,
    /// Watcher in-flight shadow tests.
    pub watcher: Duration,
    /// Axum drain.
    pub http: Duration,
}

impl Default for ShutdownBudgets {
    fn default() -> Self {
        Self {
            ingest: Duration::from_secs(5),
            regulation: Duration::from_secs(2),
            memory: Duration::from_secs(10),
            watcher: Duration::from_secs(30),
            http: Duration::from_secs(2),
        }
    }
}

/// Named task handle — used for diagnostic logging when a task exceeds its budget.
pub struct NamedHandle {
    /// Human-readable name written into shutdown log lines.
    pub name: &'static str,
    /// Per-task budget from [`ShutdownBudgets`].
    pub budget: Duration,
    /// Join handle from `tokio::spawn`.
    pub handle: JoinHandle<()>,
}

impl NamedHandle {
    /// Build a named handle with explicit budget.
    #[must_use]
    pub const fn new(name: &'static str, budget: Duration, handle: JoinHandle<()>) -> Self {
        Self { name, budget, handle }
    }
}

/// Install SIGINT + SIGTERM handlers that trigger the cancel token.
///
/// Returns a future that resolves when the first signal arrives. The
/// caller is expected to `tokio::spawn` this future and rely on the
/// cancel token for coordination.
pub async fn install_signal_handlers(cancel: CancellationToken) {
    #[cfg(unix)]
    let sigterm_fut = async {
        use tokio::signal::unix::{signal, SignalKind};
        match signal(SignalKind::terminate()) {
            Ok(mut stream) => {
                stream.recv().await;
                info!(target = "shutdown", "SIGTERM received");
            }
            Err(err) => {
                error!(target = "shutdown", %err, "failed to install SIGTERM handler");
                std::future::pending::<()>().await;
            }
        }
    };
    #[cfg(not(unix))]
    let sigterm_fut = std::future::pending::<()>();

    let sigint_fut = async {
        if let Err(err) = tokio::signal::ctrl_c().await {
            error!(target = "shutdown", %err, "ctrl_c handler failed");
            std::future::pending::<()>().await;
        } else {
            info!(target = "shutdown", "SIGINT received");
        }
    };

    tokio::select! {
        () = sigint_fut => {},
        () = sigterm_fut => {},
        () = cancel.cancelled() => {
            info!(target = "shutdown", "cancellation already triggered");
        }
    }
    cancel.cancel();
}

/// Await a batch of named handles with per-task budgets **concurrently**.
///
/// All tasks are joined in parallel — each with its own individual
/// timeout — so total shutdown time is `max(budgets)` instead of
/// `sum(budgets)`.
///
/// Returns the number of tasks that completed within their budget.
/// Over-running tasks are logged via `tracing::warn!` and abandoned —
/// the runtime is shutting down anyway.
pub async fn join_with_budgets(handles: Vec<NamedHandle>) -> usize {
    let futures: Vec<_> = handles
        .into_iter()
        .map(|nh| {
            let NamedHandle { name, budget, handle } = nh;
            async move {
                match tokio::time::timeout(budget, handle).await {
                    Ok(Ok(())) => {
                        info!(target = "shutdown", task = name, "clean exit");
                        true
                    }
                    Ok(Err(err)) => {
                        error!(target = "shutdown", task = name, %err, "task panicked");
                        false
                    }
                    Err(_) => {
                        warn!(
                            target = "shutdown",
                            task = name,
                            budget_ms = u64::try_from(budget.as_millis()).unwrap_or(u64::MAX),
                            "exceeded shutdown budget — abandoning"
                        );
                        false
                    }
                }
            }
        })
        .collect();
    let results = futures::future::join_all(futures).await;
    results.into_iter().filter(|ok| *ok).count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU8, Ordering};

    #[test]
    fn default_budgets_match_plan() {
        let b = ShutdownBudgets::default();
        assert_eq!(b.ingest, Duration::from_secs(5));
        assert_eq!(b.regulation, Duration::from_secs(2));
        assert_eq!(b.memory, Duration::from_secs(10));
        assert_eq!(b.watcher, Duration::from_secs(30));
        assert_eq!(b.http, Duration::from_secs(2));
    }

    #[test]
    fn budgets_implement_copy_clone() {
        let b = ShutdownBudgets::default();
        let c = b;
        assert_eq!(c.memory, Duration::from_secs(10));
    }

    #[tokio::test]
    async fn join_with_budgets_returns_zero_for_empty_input() {
        assert_eq!(join_with_budgets(vec![]).await, 0);
    }

    #[tokio::test]
    async fn join_with_budgets_counts_clean_exits() {
        let counter = Arc::new(AtomicU8::new(0));
        let counter2 = Arc::clone(&counter);
        let handle = tokio::spawn(async move {
            counter2.fetch_add(1, Ordering::SeqCst);
        });
        let nh = NamedHandle::new("test", Duration::from_secs(1), handle);
        assert_eq!(join_with_budgets(vec![nh]).await, 1);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn join_with_budgets_abandons_over_budget_task() {
        let handle = tokio::spawn(async {
            tokio::time::sleep(Duration::from_secs(10)).await;
        });
        let nh = NamedHandle::new("slow", Duration::from_millis(10), handle);
        let start = std::time::Instant::now();
        let completed = join_with_budgets(vec![nh]).await;
        let elapsed = start.elapsed();
        assert_eq!(completed, 0, "over-budget task must not count as completed");
        assert!(
            elapsed < Duration::from_millis(500),
            "abandoning must not wait for the task"
        );
    }

    #[tokio::test]
    async fn install_signal_handlers_cancels_on_external_signal() {
        let cancel = CancellationToken::new();
        let cancel_for_spawn = cancel.clone();
        let handler = tokio::spawn(async move {
            install_signal_handlers(cancel_for_spawn).await;
        });
        // External trigger (simulating the daemon proper calling cancel()).
        cancel.cancel();
        let result = tokio::time::timeout(Duration::from_secs(1), handler).await;
        assert!(result.is_ok(), "handler must observe external cancel");
        assert!(cancel.is_cancelled());
    }
}
