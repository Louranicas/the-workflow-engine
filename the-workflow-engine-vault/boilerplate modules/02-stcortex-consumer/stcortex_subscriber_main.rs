//! stcortex-subscriber — reference SpacetimeDB SDK consumer.
//!
//! Connects to the local stcortex module via WebSocket, registers as a consumer,
//! subscribes to pathway + memory tables, and prints every delta as it arrives.
//! This is the architectural-commitment proof: subscribers register naturally as
//! the act of subscribing, and the database pushes them updates — no polling,
//! no consumption gradient.
//!
//! Usage:
//!   stcortex-subscriber [namespace]
//!     namespace defaults to "claude-code"
//!
//! Behaviour:
//!   1. Connect to ws://127.0.0.1:3000, module name "stcortex"
//!   2. On connect: call register_consumer reducer
//!   3. Subscribe to: SELECT * FROM pathway WHERE namespace = '<ns>'
//!   4. Subscribe to: SELECT * FROM memory   WHERE namespace = '<ns>'
//!   5. Print every insert/update/delete delivered via WebSocket
//!   6. Run forever — Ctrl-C to exit

#![allow(clippy::disallowed_macros)]

mod module_bindings;

use module_bindings::*;
use spacetimedb_sdk::{DbContext, Table, TableWithPrimaryKey};

const HOST: &str = "ws://127.0.0.1:3000";
const DB: &str = "stcortex";
const CONSUMER_NAME: &str = "stcortex-subscriber-rust";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .parse_env("RUST_LOG")
        .init();

    let namespace = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "claude-code".to_string());
    println!("[stcortex-subscriber] connecting to {HOST} / {DB} (namespace={namespace})");

    let ns_for_callback = namespace.clone();
    let conn = DbConnection::builder()
        .with_uri(HOST)
        .with_database_name(DB)
        .on_connect(move |ctx, identity, _token| {
            println!("[stcortex-subscriber] connected as identity {identity}");

            // Step 1 — register ourselves as a consumer (idempotent).
            //          This is what unlocks writes to the namespace.
            if let Err(e) = ctx.reducers.register_consumer(
                CONSUMER_NAME.to_string(),
                ns_for_callback.clone(),
                "subscription".to_string(),
            ) {
                eprintln!("[stcortex-subscriber] register_consumer failed: {e:?}");
            } else {
                println!("[stcortex-subscriber] registered as consumer '{CONSUMER_NAME}' for namespace '{}'", ns_for_callback);
            }

            // Step 2 — table change handlers (fire on every delta after subscription applied)
            ctx.db.pathway().on_insert(|_ctx, row| {
                println!(
                    "[delta] PATHWAY INSERT  ns={}  {} -> {}  weight={:.4}  reinforce_count={}",
                    row.namespace, row.pre_id, row.post_id, row.weight, row.reinforce_count
                );
            });
            ctx.db.pathway().on_update(|_ctx, old, new| {
                println!(
                    "[delta] PATHWAY UPDATE  ns={}  {} -> {}  weight={:.4} -> {:.4}",
                    new.namespace, new.pre_id, new.post_id, old.weight, new.weight
                );
            });
            ctx.db.pathway().on_delete(|_ctx, row| {
                println!(
                    "[delta] PATHWAY DELETE  ns={}  {} -> {}  (final weight {:.4})",
                    row.namespace, row.pre_id, row.post_id, row.weight
                );
            });

            ctx.db.memory().on_insert(|_ctx, row| {
                println!(
                    "[delta] MEMORY INSERT   ns={}  modality={}  intensity={:.3}  content={:?}",
                    row.namespace, row.modality, row.intensity,
                    row.content.chars().take(80).collect::<String>()
                );
            });
            ctx.db.memory().on_update(|_ctx, old, new| {
                println!(
                    "[delta] MEMORY UPDATE   id={}  access_count={} -> {}  intensity={:.3} -> {:.3}  crystallized={} -> {}",
                    new.id, old.access_count, new.access_count,
                    old.intensity, new.intensity,
                    old.crystallized, new.crystallized
                );
            });
            ctx.db.memory().on_delete(|_ctx, row| {
                println!(
                    "[delta] MEMORY DELETE   id={}  ns={}  (final intensity {:.3})",
                    row.id, row.namespace, row.intensity
                );
            });

            ctx.db.consumption_event().on_insert(|_ctx, row| {
                println!(
                    "[delta] CONSUMPTION     memory_id={}  consumer={}",
                    row.memory_id, row.consumer_name
                );
            });

            // Step 3 — subscribe to the queries we care about
            let q_path = format!("SELECT * FROM pathway WHERE namespace = '{}'", ns_for_callback);
            let q_mem = format!("SELECT * FROM memory WHERE namespace = '{}'", ns_for_callback);
            let q_event = "SELECT * FROM consumption_event".to_string();
            ctx.subscription_builder()
                .on_applied(|_ctx| println!("[stcortex-subscriber] initial subscription state delivered"))
                .on_error(|_ctx, e| eprintln!("[stcortex-subscriber] subscription error: {e:?}"))
                .subscribe([q_path, q_mem, q_event]);
        })
        .on_connect_error(|_ctx, error| {
            eprintln!("[stcortex-subscriber] connect error: {error:?}");
            std::process::exit(2);
        })
        .on_disconnect(|_ctx, err| {
            if let Some(e) = err {
                eprintln!("[stcortex-subscriber] disconnected with error: {e:?}");
            } else {
                println!("[stcortex-subscriber] disconnected cleanly");
            }
        })
        .build()?;

    println!("[stcortex-subscriber] running — Ctrl-C to exit");
    conn.run_threaded()
        .join()
        .map_err(|e| format!("subscriber background thread panicked: {e:?}"))?;
    Ok(())
}
