//! stcortex-capacity — SDK/WebSocket capacity probe for stcortex.
//!
//! This is the first-class pressure-test path for stcortex. It uses generated
//! SpacetimeDB Rust bindings and WebSocket subscription deltas, not CLI loops.
//!
//! Usage:
//!   stcortex-capacity [--namespace stcortex-bench] [--n 300] [--dim 32]
//!                     [--host ws://127.0.0.1:3000] [--db stcortex]
//!                     [--access 200] [--confirmed-reads]
//!
//! Cleanup:
//!   After publishing a module with `cleanup_test_namespace`, remove test rows:
//!   stcortex call cleanup_test_namespace stcortex-bench delete:stcortex-bench

#![allow(clippy::disallowed_macros)]

mod module_bindings;

use module_bindings::*;
use spacetimedb_sdk::{DbContext, Table, TableWithPrimaryKey};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicUsize, Ordering},
    mpsc,
};
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
struct Config {
    host: String,
    db: String,
    namespace: String,
    n: usize,
    dim: usize,
    access: usize,
    confirmed_reads: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: std::env::var("STCORTEX_HOST_WS")
                .unwrap_or_else(|_| "ws://127.0.0.1:3000".to_string()),
            db: std::env::var("STCORTEX_DB").unwrap_or_else(|_| "stcortex".to_string()),
            namespace: std::env::var("STCORTEX_BENCH_NS")
                .unwrap_or_else(|_| "stcortex-bench".to_string()),
            n: std::env::var("STCORTEX_BENCH_N")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(300),
            dim: std::env::var("STCORTEX_BENCH_DIM")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(32),
            access: std::env::var("STCORTEX_BENCH_ACCESS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(200),
            confirmed_reads: std::env::var("STCORTEX_CONFIRMED_READS")
                .is_ok_and(|v| matches!(v.as_str(), "1" | "true" | "yes")),
        }
    }
}

fn parse_args() -> Result<Config, String> {
    let mut cfg = Config::default();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--host" => cfg.host = take_value(&mut args, "--host")?,
            "--db" => cfg.db = take_value(&mut args, "--db")?,
            "--namespace" | "--ns" => cfg.namespace = take_value(&mut args, "--namespace")?,
            "--n" => cfg.n = parse_value(&mut args, "--n")?,
            "--dim" => cfg.dim = parse_value(&mut args, "--dim")?,
            "--access" => cfg.access = parse_value(&mut args, "--access")?,
            "--confirmed-reads" => cfg.confirmed_reads = true,
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    if cfg.n == 0 {
        return Err("--n must be > 0".to_string());
    }
    if cfg.dim == 0 {
        return Err("--dim must be > 0".to_string());
    }
    cfg.namespace = normalize_namespace(&cfg.namespace)?;
    Ok(cfg)
}

fn normalize_namespace(raw: &str) -> Result<String, String> {
    let ns = raw
        .chars()
        .map(|c| match c {
            'A'..='Z' => c.to_ascii_lowercase(),
            'a'..='z' | '0'..='9' | '_' | '-' => c,
            _ => '-',
        })
        .collect::<String>()
        .trim_matches('-')
        .chars()
        .take(64)
        .collect::<String>();
    if ns.is_empty() {
        return Err("--namespace must contain at least one [a-z0-9_-] character".to_string());
    }
    Ok(ns)
}

fn take_value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    args.next()
        .filter(|v| !v.is_empty())
        .ok_or_else(|| format!("{flag} requires a value"))
}

fn parse_value<T>(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<T, String>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    take_value(args, flag)?
        .parse::<T>()
        .map_err(|e| format!("invalid {flag}: {e}"))
}

fn print_help() {
    println!(
        "stcortex-capacity --namespace stcortex-bench --n 300 --dim 32 --access 200 [--confirmed-reads]"
    );
}

fn wait_count(counter: &AtomicUsize, target: usize, deadline: Duration) -> (usize, f64) {
    let start = Instant::now();
    while start.elapsed() < deadline {
        let n = counter.load(Ordering::Relaxed);
        if n >= target {
            return (n, start.elapsed().as_secs_f64());
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    (
        counter.load(Ordering::Relaxed),
        start.elapsed().as_secs_f64(),
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Warn)
        .parse_env("RUST_LOG")
        .init();

    let cfg = parse_args().map_err(|e| format!("{e}\ntry --help"))?;
    let consumer = format!("sdk-capacity-{}", std::process::id());
    let session = format!("sdk-capacity-session-{}", std::process::id());

    let mem_inserts = Arc::new(AtomicUsize::new(0));
    let path_inserts = Arc::new(AtomicUsize::new(0));
    let mem_updates = Arc::new(AtomicUsize::new(0));
    let memory_ids = Arc::new(Mutex::new(Vec::<u64>::new()));
    let reducer_ok = Arc::new(AtomicUsize::new(0));
    let reducer_err = Arc::new(AtomicUsize::new(0));
    let (tx, rx) = mpsc::channel::<()>();

    let conn = DbConnection::builder()
        .with_uri(cfg.host.clone())
        .with_database_name(cfg.db.clone())
        .with_confirmed_reads(cfg.confirmed_reads)
        .on_connect({
            let cfg = cfg.clone();
            let consumer = consumer.clone();
            let mem_inserts = mem_inserts.clone();
            let path_inserts = path_inserts.clone();
            let mem_updates = mem_updates.clone();
            let memory_ids = memory_ids.clone();
            move |ctx, identity, _token| {
                println!(
                    "connected identity={identity} namespace={} n={} dim={} confirmed_reads={}",
                    cfg.namespace, cfg.n, cfg.dim, cfg.confirmed_reads
                );
                ctx.db.memory().on_insert({
                    let namespace = cfg.namespace.clone();
                    let mem_inserts = mem_inserts.clone();
                    let memory_ids = memory_ids.clone();
                    move |_ctx, row| {
                        if row.namespace == namespace {
                            memory_ids.lock().expect("memory id lock").push(row.id);
                            mem_inserts.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                });
                ctx.db.pathway().on_insert({
                    let namespace = cfg.namespace.clone();
                    let path_inserts = path_inserts.clone();
                    move |_ctx, row| {
                        if row.namespace == namespace {
                            path_inserts.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                });
                ctx.db.memory().on_update({
                    let namespace = cfg.namespace.clone();
                    let mem_updates = mem_updates.clone();
                    move |_ctx, _old, row| {
                        if row.namespace == namespace {
                            mem_updates.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                });
                if let Err(e) = ctx.reducers.register_consumer(
                    consumer.clone(),
                    cfg.namespace.clone(),
                    "subscription".to_string(),
                ) {
                    eprintln!("register_consumer error: {e:?}");
                }
                ctx.subscription_builder()
                    .on_applied({
                        let tx = tx.clone();
                        move |_ctx| {
                            let _ = tx.send(());
                        }
                    })
                    .on_error(|_ctx, e| eprintln!("subscription error: {e:?}"))
                    .subscribe([
                        format!("SELECT * FROM memory WHERE namespace = '{}'", cfg.namespace),
                        format!(
                            "SELECT * FROM pathway WHERE namespace = '{}'",
                            cfg.namespace
                        ),
                    ]);
            }
        })
        .on_connect_error(|_ctx, err| {
            eprintln!("connect error: {err:?}");
            std::process::exit(2);
        })
        .build()?;

    let runner = conn.run_threaded();
    rx.recv_timeout(Duration::from_secs(5))
        .map_err(|_| "subscription did not apply within 5s")?;

    let base_mem = mem_inserts.load(Ordering::Relaxed);
    let base_path = path_inserts.load(Ordering::Relaxed);
    let base_upd = mem_updates.load(Ordering::Relaxed);

    let start = Instant::now();
    for i in 0..cfg.n {
        let tensor: Vec<f32> = (0..cfg.dim).map(|j| ((i + j + 1) as f32).sin()).collect();
        let okc = reducer_ok.clone();
        let errc = reducer_err.clone();
        conn.reducers.write_memory_then(
            cfg.namespace.clone(),
            "semantic".to_string(),
            format!("sdk capacity memory {i} dim {}", cfg.dim),
            1.0,
            Some(tensor),
            session.clone(),
            Some("sdk-capacity".to_string()),
            vec![],
            move |_ctx, res| {
                if matches!(res, Ok(Ok(()))) {
                    okc.fetch_add(1, Ordering::Relaxed);
                } else {
                    errc.fetch_add(1, Ordering::Relaxed);
                    eprintln!("write_memory error: {res:?}");
                }
            },
        )?;
    }
    let enqueue_mem_s = start.elapsed().as_secs_f64();
    let (mi_seen, mem_wait_s) = wait_count(&mem_inserts, base_mem + cfg.n, Duration::from_secs(30));

    let start_p = Instant::now();
    for i in 0..cfg.n {
        let okc = reducer_ok.clone();
        let errc = reducer_err.clone();
        conn.reducers.write_pathway_then(
            format!("bench-pre-{}", i % 64),
            format!("bench-post-{i}"),
            cfg.namespace.clone(),
            0.2,
            session.clone(),
            Some("sdk-capacity".to_string()),
            move |_ctx, res| {
                if matches!(res, Ok(Ok(()))) {
                    okc.fetch_add(1, Ordering::Relaxed);
                } else {
                    errc.fetch_add(1, Ordering::Relaxed);
                    eprintln!("write_pathway error: {res:?}");
                }
            },
        )?;
    }
    let enqueue_path_s = start_p.elapsed().as_secs_f64();
    let (pi_seen, path_wait_s) =
        wait_count(&path_inserts, base_path + cfg.n, Duration::from_secs(30));

    let ids: Vec<u64> = memory_ids
        .lock()
        .expect("memory id lock")
        .iter()
        .copied()
        .rev()
        .take(cfg.n.min(cfg.access))
        .collect();
    let access_target = ids.len();
    let start_access = Instant::now();
    for id in ids {
        let okc = reducer_ok.clone();
        let errc = reducer_err.clone();
        conn.reducers
            .access_memory_then(id, consumer.clone(), move |_ctx, res| {
                if matches!(res, Ok(Ok(()))) {
                    okc.fetch_add(1, Ordering::Relaxed);
                } else {
                    errc.fetch_add(1, Ordering::Relaxed);
                    eprintln!("access_memory error: {res:?}");
                }
            })?;
    }
    let enqueue_access_s = start_access.elapsed().as_secs_f64();
    let (upd_seen, access_wait_s) = wait_count(
        &mem_updates,
        base_upd + access_target,
        Duration::from_secs(20),
    );

    let nn_done = Arc::new(AtomicBool::new(false));
    let nn_ok = Arc::new(AtomicBool::new(false));
    let start_nn = Instant::now();
    let q: Vec<f32> = (0..cfg.dim).map(|j| ((j + 1) as f32).sin()).collect();
    conn.reducers.nearest_neighbor_then(
        format!("sdk-capacity-q-{}", std::process::id()),
        q,
        10,
        cfg.namespace.clone(),
        {
            let nn_done = nn_done.clone();
            let nn_ok = nn_ok.clone();
            move |_ctx, res| {
                nn_ok.store(matches!(res, Ok(Ok(()))), Ordering::Relaxed);
                nn_done.store(true, Ordering::Relaxed);
            }
        },
    )?;
    while !nn_done.load(Ordering::Relaxed) && start_nn.elapsed() < Duration::from_secs(20) {
        std::thread::sleep(Duration::from_millis(2));
    }
    let nn_s = start_nn.elapsed().as_secs_f64();

    println!("RESULT namespace={}", cfg.namespace);
    println!(
        "RESULT memory_enqueued={} enqueue_s={enqueue_mem_s:.6} enqueue_rate_per_s={:.1}",
        cfg.n,
        cfg.n as f64 / enqueue_mem_s.max(1e-9)
    );
    println!(
        "RESULT memory_delta_seen={} delta_wait_s={mem_wait_s:.6} effective_delta_rate_per_s={:.1}",
        mi_seen.saturating_sub(base_mem),
        mi_seen.saturating_sub(base_mem) as f64 / mem_wait_s.max(1e-9)
    );
    println!(
        "RESULT pathway_enqueued={} enqueue_s={enqueue_path_s:.6} enqueue_rate_per_s={:.1}",
        cfg.n,
        cfg.n as f64 / enqueue_path_s.max(1e-9)
    );
    println!(
        "RESULT pathway_delta_seen={} delta_wait_s={path_wait_s:.6} effective_delta_rate_per_s={:.1}",
        pi_seen.saturating_sub(base_path),
        pi_seen.saturating_sub(base_path) as f64 / path_wait_s.max(1e-9)
    );
    println!(
        "RESULT access_enqueued={access_target} enqueue_s={enqueue_access_s:.6} access_updates_seen={} access_wait_s={access_wait_s:.6} access_update_rate_per_s={:.1}",
        upd_seen.saturating_sub(base_upd),
        upd_seen.saturating_sub(base_upd) as f64 / access_wait_s.max(1e-9)
    );
    println!(
        "RESULT nn_ok={} nn_callback_s={nn_s:.6}",
        nn_ok.load(Ordering::Relaxed)
    );
    println!(
        "RESULT reducer_ok={} reducer_err={}",
        reducer_ok.load(Ordering::Relaxed),
        reducer_err.load(Ordering::Relaxed)
    );

    let _ = conn.disconnect();
    let _ = runner.join();

    if reducer_err.load(Ordering::Relaxed) > 0 || !nn_ok.load(Ordering::Relaxed) {
        std::process::exit(1);
    }
    Ok(())
}
