//! `workflow_core` — shared library for the `wf-crystallise` and `wf-dispatch`
//! binaries.
//!
//! Per Genesis Prompt v1.3 binding spec § 1 (Cluster-D Trust cross-cutting),
//! Cluster D ships first on G9 fire. m8 is the **floor of the trust regime**:
//! a compile-time gate that ensures no code reads POVM `learning_health` until
//! the CR-2 magnitude-weighted formula is verified live.
//!
//! # Module map (Cluster D Day-1 — m8 → m9 → m10 → m11)
//!
//! - [`m8_povm_build_prereq`] — compile-time + runtime CR-2 verification gate
//!   (ships first; all other Cluster D modules transitively depend on it).
//!
//! Subsequent Cluster D modules (m9 namespace guard, m10 Ember CI gate, m11
//! compound decay) land in this lib in their own commits per the
//! non-negotiable phase-1-framework order.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
// `BuildPrereqError` lives in `error` module; `HealthClient` in `health`. The
// repeated stem is intentional habitat convention (matches LCM / ORAC /
// synthex-v2 module-typing pattern). See CLAUDE.md god-tier standards § 8
// (newtype discipline).
#![allow(clippy::module_name_repetitions)]
// Acronyms (POVM, CR-2, URL, JSON, HTTP) in prose doc comments are habitat
// convention — backticking every occurrence would harm readability for
// minimal correctness gain (the spec docs in `ai_specs/` are the canonical
// reference, not the rustdoc).
#![allow(clippy::doc_markdown)]

pub mod m8_povm_build_prereq;
