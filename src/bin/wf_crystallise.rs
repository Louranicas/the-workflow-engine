//! `wf-crystallise` — primary workflow-trace binary owning modules m1-m23
//! and m40-m42 (Substrate Ingest, Habitat Observers, Correlation, Trust,
//! Evidence, Iteration, Substrate Feedback).
//!
//! Per Genesis Prompt v1.3 § 1 (two-binary split locked: `wf-crystallise`
//! owns m1-m23 + m40-m42; `wf-dispatch` owns m30-m33; both share the
//! `workflow_core` lib in the same Cargo crate per the ORAC single-crate
//! pattern, NOT the LCM 10-crate workspace pattern).
//!
//! This entrypoint is a stub at Cluster D Day-1 — Cluster D (m8/m9/m10/m11)
//! ships in `workflow_core` first as the trust-regime floor. Subsequent
//! modules (m1-m7, m12-m23, m40-m42) land in their own commits per the
//! non-negotiable phase-1 framework.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

fn main() {
    println!("wf-crystallise · workflow-trace v0.1.0 (Cluster D Day-1 scaffold)");
}
