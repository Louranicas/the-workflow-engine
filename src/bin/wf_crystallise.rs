//! `wf-crystallise` — primary workflow-trace binary owning modules m1-m23
//! and m40-m42 (Substrate Ingest, Habitat Observers, Correlation, Trust,
//! Evidence, Iteration, Substrate Feedback).
//!
//! Per Genesis Prompt v1.3 § 1 (two-binary split: `wf-crystallise` owns
//! m1-m23 + m40-m42; `wf-dispatch` owns m30-m33; both share the
//! `workflow_core` lib in the same Cargo crate per the ORAC single-crate
//! pattern).
//!
//! This binary is intentionally thin: it parses `std::env::args()`, hands
//! off to [`workflow_core::orchestration::crystallise`], prints the
//! resulting report, and sets a process exit code. All pipeline logic
//! lives in the library so it is integration-testable.
//!
//! # Exit codes
//!
//! - `0` — pipeline ran to completion (or `--help` / `--version`).
//! - `1` — a pipeline fault (missing DB, unwritable output, ...).
//! - `2` — a CLI argument error.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::process::ExitCode;

use workflow_core::orchestration::crystallise::{
    parse_args, run, Config, ReportFormat, HELP_TEXT,
};

/// Crate version, surfaced by `--version`.
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    // `tracing` events are emitted throughout the pipeline. No subscriber
    // is installed here (`tracing-subscriber` is a dev-dependency only);
    // without one the events are silently dropped, which is the correct
    // default for a CLI whose canonical output is the printed report.
    let args: Vec<String> = std::env::args().skip(1).collect();
    let config: Config = match parse_args(&args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("wf-crystallise: {e}");
            eprint!("{HELP_TEXT}");
            return ExitCode::from(2);
        }
    };

    if config.show_help {
        print!("{HELP_TEXT}");
        return ExitCode::SUCCESS;
    }
    if config.show_version {
        println!("wf-crystallise {VERSION}");
        return ExitCode::SUCCESS;
    }

    match run(&config) {
        Ok(report) => {
            match config.format {
                ReportFormat::Text => println!("{report}"),
                ReportFormat::Json => match serde_json::to_string(&report) {
                    Ok(json) => println!("{json}"),
                    Err(e) => {
                        eprintln!("wf-crystallise: report serialisation failed: {e}");
                        return ExitCode::FAILURE;
                    }
                },
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("wf-crystallise: pipeline fault: {e}");
            ExitCode::FAILURE
        }
    }
}
