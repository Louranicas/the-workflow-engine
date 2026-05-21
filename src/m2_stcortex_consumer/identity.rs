//! Consumer identity newtypes — `ConsumerName`, `Namespace`, `Transport`,
//! and the aggregate `ConsumerIdentity`.
//!
//! Per m2 spec § 1 invariant 2: `ConsumerIdentity` rejects any namespace
//! not equal to `workflow_trace_*`; reserved names (`scratch`,
//! `claude-code`) and empty are rejected at construction. AP30
//! enforcement lives in the type system.

use crate::m9_watcher_namespace_guard::WORKFLOW_TRACE_NS_PREFIX;

use super::error::StcortexConsumerError;

/// Required prefix for any namespace m2 will register/subscribe against.
///
/// Re-exports the single source of truth from
/// [`crate::m9_watcher_namespace_guard::WORKFLOW_TRACE_NS_PREFIX`] so the
/// AP30 invariant has exactly one literal site in `src/`.
pub const WORKFLOW_TRACE_PREFIX: &str = WORKFLOW_TRACE_NS_PREFIX;

/// Maximum length for a validated [`ConsumerName`].
pub const CONSUMER_NAME_MAX_LEN: usize = 64;

/// Validated consumer name. Newtype enforces alphanumeric + `_-` chars,
/// non-empty, length ≤ 64.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConsumerName(String);

impl ConsumerName {
    /// Construct a validated `ConsumerName`.
    ///
    /// # Errors
    ///
    /// [`StcortexConsumerError::InvalidConsumerName`] if empty, over 64
    /// chars, or containing chars outside `[A-Za-z0-9_-]`.
    pub fn new(value: impl Into<String>) -> Result<Self, StcortexConsumerError> {
        let s: String = value.into();
        if s.is_empty() {
            return Err(StcortexConsumerError::InvalidConsumerName(
                "consumer name is empty".into(),
            ));
        }
        if s.len() > CONSUMER_NAME_MAX_LEN {
            return Err(StcortexConsumerError::InvalidConsumerName(format!(
                "consumer name {s:?} length {} > {CONSUMER_NAME_MAX_LEN}",
                s.len()
            )));
        }
        for c in s.chars() {
            if !(c.is_ascii_alphanumeric() || c == '_' || c == '-') {
                return Err(StcortexConsumerError::InvalidConsumerName(format!(
                    "consumer name {s:?} contains invalid character {c:?}"
                )));
            }
        }
        Ok(Self(s))
    }

    /// Borrow the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ConsumerName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Validated stcortex namespace. Newtype enforces:
///
/// - non-empty,
/// - starts with [`WORKFLOW_TRACE_PREFIX`],
/// - not equal to reserved tags (`scratch`, `claude-code`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Namespace(String);

impl Namespace {
    /// Construct a validated `Namespace`.
    ///
    /// # Errors
    ///
    /// [`StcortexConsumerError::InvalidNamespace`] for empty input,
    /// reserved names (`scratch`, `claude-code`), or anything not
    /// starting with [`WORKFLOW_TRACE_PREFIX`].
    pub fn new(value: impl Into<String>) -> Result<Self, StcortexConsumerError> {
        let s: String = value.into();
        if s.is_empty() {
            return Err(StcortexConsumerError::InvalidNamespace(
                "namespace is empty".into(),
            ));
        }
        if s == "scratch" || s == "claude-code" {
            return Err(StcortexConsumerError::InvalidNamespace(format!(
                "namespace {s:?} is reserved"
            )));
        }
        if !s.starts_with(WORKFLOW_TRACE_PREFIX) {
            return Err(StcortexConsumerError::InvalidNamespace(format!(
                "namespace {s:?} does not start with {WORKFLOW_TRACE_PREFIX:?}"
            )));
        }
        Ok(Self(s))
    }

    /// Borrow the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Namespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Transport discriminant. Day-1 locks at [`Self::Subscription`] per
/// spec § 2; future-flagged variants would carry their own wire-shape
/// invariants and would require a spec amendment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Transport {
    /// SpacetimeDB SDK subscription transport.
    Subscription,
}

impl Transport {
    /// Stable wire-form passed to `register_consumer` reducer.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Subscription => "subscription",
        }
    }
}

/// Aggregate identity passed to `register_narrowed_consumer`.
#[derive(Debug, Clone)]
pub struct ConsumerIdentity {
    /// Validated consumer name.
    pub name: ConsumerName,
    /// Validated namespace.
    pub namespace: Namespace,
    /// Transport (always `Subscription` Day-1).
    pub transport: Transport,
}

/// Absolute paths searched, in order, for the `git` executable used by
/// [`ConsumerIdentity::from_git_sha`].
///
/// # SEC3 — `$PATH`-hijack hardening
///
/// `from_git_sha` shells out to `git rev-parse --short HEAD` purely to
/// derive a cosmetic identity string. A bare `Command::new("git")`
/// resolves the binary through `$PATH`, so a process started with a
/// hostile `$PATH` (a `git` shim earlier on the search path) would run
/// attacker-controlled code. The output is *only* an identity label, so
/// the blast radius is small — but a subprocess that runs whatever
/// `$PATH` points at is still a needless trust dependency.
///
/// Reimplementing git's repository discovery in-process (parent-dir
/// walk + `.git`-file / worktree `gitdir:` indirection + symbolic-ref
/// resolution + `packed-refs` fallback) was considered and rejected:
/// `the-workflow-engine/` is a *subdirectory* of the workspace repo
/// (`.git` lives at the workspace root, not in CWD), so a naive
/// "read `./.git/HEAD`" would silently yield the `unknown` fallback.
/// A correct hand-rolled walk is ~100 LOC of fragile path logic for a
/// LOW-severity cosmetic string — a poor trade. Instead we keep the
/// subprocess but resolve `git` via these fixed absolute paths first,
/// which removes the `$PATH`-hijack vector entirely on a standard
/// Linux host. The bare `"git"` name is retained only as the last
/// resort (non-standard install layout); that residual `$PATH` trust
/// is documented and accepted because the output cannot escape the
/// `ConsumerName` validator.
const GIT_ABSOLUTE_PATHS: &[&str] = &["/usr/bin/git", "/bin/git", "/usr/local/bin/git"];

impl ConsumerIdentity {
    /// Construct from the current process's git short SHA if available;
    /// falls back to `"workflow-trace-unknown"` if `git` is absent or
    /// fails. The resulting name always starts with `workflow-trace-`.
    ///
    /// The `git` executable is resolved via the fixed absolute paths in
    /// [`GIT_ABSOLUTE_PATHS`] before falling back to a `$PATH` lookup —
    /// see that constant's docs for the SEC3 `$PATH`-hijack rationale.
    #[must_use]
    pub fn from_git_sha(namespace: Namespace) -> Self {
        let sha = Self::resolve_git_short_sha().unwrap_or_else(|| "unknown".to_owned());
        // ConsumerName allows alphanumeric + `_-`; short SHA is hex.
        // Fall back to the literal "workflow-trace-unknown" on validation
        // failure so the identity is always constructible. We construct
        // the fallback via direct field access (valid inside this module)
        // to avoid any chance of `expect`-panic if a future validator
        // change rejects the literal.
        let name_str = format!("workflow-trace-{sha}");
        let name = ConsumerName::new(&name_str)
            .unwrap_or_else(|_| Self::unknown_name_fallback());
        Self {
            name,
            namespace,
            transport: Transport::Subscription,
        }
    }

    /// Hardcoded fallback consumer name when git is absent or the
    /// derived SHA-based name fails validation. Uses direct field
    /// construction (allowed inside this module) so it cannot panic.
    fn unknown_name_fallback() -> ConsumerName {
        ConsumerName(String::from("workflow-trace-unknown"))
    }

    /// Run `git rev-parse --short HEAD` and return the trimmed short SHA,
    /// or `None` if `git` is unavailable / the command fails / the
    /// output is empty.
    ///
    /// SEC3: `git` is resolved via the absolute paths in
    /// [`GIT_ABSOLUTE_PATHS`] first; the bare `"git"` name is tried only
    /// as a last resort. This eliminates the `$PATH`-hijack vector on a
    /// standard host while keeping `git`'s own (correct) repository
    /// discovery — `the-workflow-engine/` is a subdirectory of the
    /// workspace repo, so `git` must walk up to find `.git`.
    fn resolve_git_short_sha() -> Option<String> {
        // Try each fixed absolute path, then the bare name as a final
        // fallback. The first program that both spawns AND exits 0 wins.
        let candidates = GIT_ABSOLUTE_PATHS
            .iter()
            .copied()
            .chain(std::iter::once("git"));
        for program in candidates {
            let output = std::process::Command::new(program)
                .args(["rev-parse", "--short", "HEAD"])
                .output();
            let Ok(out) = output else {
                // This program path did not spawn (absent / not
                // executable) — try the next candidate.
                continue;
            };
            if !out.status.success() {
                // Spawned but git itself failed (e.g., not a repo).
                // Re-trying another git binary will not change that, so
                // stop here and report no SHA.
                return None;
            }
            let sha = String::from_utf8(out.stdout).ok()?.trim().to_owned();
            if sha.is_empty() {
                return None;
            }
            return Some(sha);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::super::error::StcortexConsumerError;
    use super::{
        ConsumerIdentity, ConsumerName, Namespace, Transport, CONSUMER_NAME_MAX_LEN,
        WORKFLOW_TRACE_PREFIX,
    };

    // ---- ConsumerName (6) ------------------------------------------------

    #[test]
    fn consumer_name_accepts_simple_alphanumeric() {
        assert_eq!(
            ConsumerName::new("workflow-trace-abc123").unwrap().as_str(),
            "workflow-trace-abc123"
        );
    }

    #[test]
    fn consumer_name_rejects_empty() {
        assert!(matches!(
            ConsumerName::new(""),
            Err(StcortexConsumerError::InvalidConsumerName(_))
        ));
    }

    #[test]
    fn consumer_name_rejects_over_64_chars() {
        let long = "x".repeat(CONSUMER_NAME_MAX_LEN + 1);
        assert!(matches!(
            ConsumerName::new(long),
            Err(StcortexConsumerError::InvalidConsumerName(_))
        ));
    }

    #[test]
    fn consumer_name_accepts_64_chars_exactly() {
        let s = "a".repeat(CONSUMER_NAME_MAX_LEN);
        assert!(ConsumerName::new(s).is_ok());
    }

    #[test]
    fn consumer_name_rejects_invalid_characters() {
        for bad in ["bad name!", "with space", "with.dot", "with/slash"] {
            assert!(
                matches!(
                    ConsumerName::new(bad),
                    Err(StcortexConsumerError::InvalidConsumerName(_))
                ),
                "did not reject {bad}"
            );
        }
    }

    #[test]
    fn consumer_name_display_is_inner() {
        let n = ConsumerName::new("xyz").unwrap();
        assert_eq!(format!("{n}"), "xyz");
    }

    // ---- Namespace (6) --------------------------------------------------

    #[test]
    fn namespace_accepts_canonical_prefix_form() {
        assert!(Namespace::new("workflow_trace_foo").is_ok());
    }

    #[test]
    fn namespace_accepts_bare_prefix() {
        assert!(Namespace::new(WORKFLOW_TRACE_PREFIX).is_ok());
    }

    #[test]
    fn namespace_rejects_empty() {
        assert!(matches!(
            Namespace::new(""),
            Err(StcortexConsumerError::InvalidNamespace(_))
        ));
    }

    #[test]
    fn namespace_rejects_scratch_reserved() {
        let err = Namespace::new("scratch").unwrap_err();
        let StcortexConsumerError::InvalidNamespace(msg) = err else {
            panic!("expected InvalidNamespace");
        };
        assert!(msg.contains("reserved"));
    }

    #[test]
    fn namespace_rejects_claude_code_reserved() {
        let err = Namespace::new("claude-code").unwrap_err();
        let StcortexConsumerError::InvalidNamespace(msg) = err else {
            panic!("expected InvalidNamespace");
        };
        assert!(msg.contains("reserved"));
    }

    #[test]
    fn namespace_rejects_other_service_prefixes() {
        for foreign in ["orac_x", "pane_vortex_y", "synthex_v2_z"] {
            assert!(
                matches!(
                    Namespace::new(foreign),
                    Err(StcortexConsumerError::InvalidNamespace(_))
                ),
                "did not reject {foreign}"
            );
        }
    }

    // ---- Transport + ConsumerIdentity (4) -------------------------------

    #[test]
    fn transport_as_str_is_lowercase_subscription() {
        assert_eq!(Transport::Subscription.as_str(), "subscription");
    }

    #[test]
    fn from_git_sha_yields_workflow_trace_prefix() {
        let ns = Namespace::new("workflow_trace_identity").unwrap();
        let id = ConsumerIdentity::from_git_sha(ns);
        assert!(id.name.as_str().starts_with("workflow-trace-"));
        assert_eq!(id.transport, Transport::Subscription);
    }

    // rationale: SEC3 — `from_git_sha` resolves `git` via fixed absolute
    // paths before any `$PATH` lookup. Whatever the resolution outcome,
    // the produced ConsumerName MUST always be a valid, prefix-correct
    // identity: either `workflow-trace-<sha>` (git found) or the
    // `workflow-trace-unknown` fallback (git absent). It must never
    // panic and never yield a name failing the ConsumerName validator.
    #[test]
    fn from_git_sha_always_yields_valid_prefixed_name() {
        let ns = Namespace::new("workflow_trace_sec3").unwrap();
        let id = ConsumerIdentity::from_git_sha(ns);
        let name = id.name.as_str();
        assert!(
            name.starts_with("workflow-trace-"),
            "SEC3: name must carry workflow-trace- prefix, got {name:?}"
        );
        // Round-trips through the validator — proves the subprocess
        // output (or the fallback) is always a legal ConsumerName.
        assert!(
            ConsumerName::new(name).is_ok(),
            "SEC3: derived name {name:?} must satisfy ConsumerName validator"
        );
        // The suffix is either a non-empty hex-ish short SHA or the
        // literal "unknown" — never empty.
        let suffix = name.strip_prefix("workflow-trace-").unwrap();
        assert!(!suffix.is_empty(), "SEC3: identity suffix must be non-empty");
    }

    // rationale: SEC3 — the absolute-path allowlist is the hardening
    // surface. Document-as-test: the constant must list at least the
    // canonical `/usr/bin/git` location and every entry must be an
    // absolute path (a relative entry would reintroduce `$PATH` risk).
    #[test]
    fn git_absolute_paths_are_all_absolute_and_include_usr_bin() {
        use super::GIT_ABSOLUTE_PATHS;
        assert!(
            GIT_ABSOLUTE_PATHS.contains(&"/usr/bin/git"),
            "SEC3: canonical /usr/bin/git must be in the allowlist"
        );
        for p in GIT_ABSOLUTE_PATHS {
            assert!(
                p.starts_with('/'),
                "SEC3: every git path must be absolute, got {p:?}"
            );
        }
    }

    #[test]
    fn consumer_identity_clone_preserves_fields() {
        let ns = Namespace::new("workflow_trace_clone").unwrap();
        let id = ConsumerIdentity {
            name: ConsumerName::new("workflow-trace-x").unwrap(),
            namespace: ns.clone(),
            transport: Transport::Subscription,
        };
        let c = id.clone();
        assert_eq!(c.name, id.name);
        assert_eq!(c.namespace, id.namespace);
        assert_eq!(c.transport, id.transport);
    }

    #[test]
    fn workflow_trace_prefix_const_matches_m9_invariant() {
        // m2 and m9 share the same prefix constant; cross-cluster
        // discipline test.
        assert_eq!(
            WORKFLOW_TRACE_PREFIX,
            crate::m9_watcher_namespace_guard::WORKFLOW_TRACE_NS_PREFIX
        );
    }
}
