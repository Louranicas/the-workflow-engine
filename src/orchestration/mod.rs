//! `orchestration` — the pipeline drivers behind the two binaries.
//!
//! The `wf-crystallise` and `wf-dispatch` binaries are intentionally thin:
//! they parse `std::env::args()`, hand off to one of the [`crystallise`] /
//! [`dispatch`] `run` functions in *this* library module, print the
//! resulting [`crystallise::Report`] / [`dispatch::Report`], and translate
//! it into a process exit code.
//!
//! Keeping the pipeline logic in the library (rather than in the binary
//! source files) is what makes it integration-testable: a test can call
//! [`crystallise::run`] / [`dispatch::run`] directly against temp-file
//! fixtures with `--offline` / `--dry-run`, without launching a subprocess.
//!
//! # Sub-modules
//!
//! - [`crystallise`] — owns the m1-m23 + m40-m42 ingest → mine → propose →
//!   substrate-feedback pipeline; backs the `wf-crystallise` binary.
//! - [`dispatch`] — owns the m30-m33 bank → select → verify → dispatch
//!   pipeline; backs the `wf-dispatch` binary.
//!
//! # Live-service degradation
//!
//! Every stage that touches a live habitat service (stcortex `:3000`,
//! ORAC `:8133`, synthex-v2 `:8092`, LCM `:8082`, HABITAT-CONDUCTOR
//! `:8141`) degrades gracefully: an unreachable service is logged via
//! `tracing` and the stage is skipped, recorded as a skipped/degraded
//! entry in the `Report`. The pipelines always run end-to-end to
//! completion — a down service never panics or aborts the run.

/// Pipeline driver + CLI parser for the `wf-crystallise` binary.
pub mod crystallise;
/// Pipeline driver + CLI parser for the `wf-dispatch` binary.
pub mod dispatch;
