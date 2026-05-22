//! `wf-dispatch` — secondary workflow-trace binary owning modules m30-m33
//! (Bank → Select → Verify → Dispatch; Cluster G).
//!
//! Per Genesis Prompt v1.3 § 1. This binary is intentionally thin: it
//! parses `std::env::args()`, hands off to
//! [`workflow_core::orchestration::dispatch`], prints the resulting
//! report, and sets a process exit code. All pipeline logic lives in the
//! library so it is integration-testable.
//!
//! `--dry-run` is the default-safe mode (verify + select, no Conductor
//! contact). A real dispatch requires the explicit `--execute` flag.
//!
//! # Exit codes
//!
//! - `0` — pipeline ran to completion (or `--help` / `--version`).
//! - `1` — a pipeline fault (missing/malformed proposals file, ...).
//! - `2` — a CLI argument error.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::process::ExitCode;

use workflow_core::orchestration::dispatch::{parse_args, run, Config, HELP_TEXT};

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
            eprintln!("wf-dispatch: {e}");
            eprint!("{HELP_TEXT}");
            return ExitCode::from(2);
        }
    };

    if config.show_help {
        print!("{HELP_TEXT}");
        return ExitCode::SUCCESS;
    }
    if config.show_version {
        println!("wf-dispatch {VERSION}");
        return ExitCode::SUCCESS;
    }

    match run(&config) {
        Ok(report) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("wf-dispatch: pipeline fault: {e}");
            ExitCode::FAILURE
        }
    }
}
