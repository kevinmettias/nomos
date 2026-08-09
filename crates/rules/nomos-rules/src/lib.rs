//! Band 3 — rules that judge source.
//!
//! The first thing in this workspace that judges code rather than judging the
//! workspace's own paperwork. `P10-FIRST-CHECK` opened for that reason: four types in
//! `nomos-contracts` described enforcement and nothing implemented them, and two
//! consecutive batches of work had produced audit rather than capability.
//!
//! # A rule is a pure function over text
//!
//! Nothing in this crate opens a file, walks a directory, or knows where the workspace
//! is. A rule takes [`SourceFile`]s and returns [`Finding`]s, and the caller supplies
//! the tree — the binary walks the real one, and a test hands it three files it wrote
//! by hand.
//!
//! That is not a style preference. `P10-FIRST-CHECK` requires the three instances
//! `OD-COMPLETENESS-001` analyses to fail this rule *as originally written*, and none of
//! the three can be replayed from git: each was repaired at the site. The only way to
//! judge code that no longer exists is for the rule to accept code as an argument. A
//! rule that reads the filesystem can only ever be tested against the tree it is
//! standing in.
//!
//! # What is here
//!
//! One rule: [`Check_Completeness_Mirrors`]. Scope is one rule and not three —
//! `P10-FIRST-CHECK` says so explicitly, because a single check that is honest end to
//! end is worth more than three that are nearly wired.

#![forbid(unsafe_code)]

mod mirror;
mod universe;

pub use mirror::{Check_Completeness_Mirrors, COMPLETENESS_MIRROR};
pub use universe::{DeclaredUniverse, Read_Universes, Reading, UniverseKind, Universes_In};

/// One file of source, as the caller found it.
///
/// `path` is repo-relative with forward slashes, and it is reporting only. Nothing in
/// this crate keys anything on it: `identity.rs` states the rule for the whole system,
/// and a finding identified by its path is a finding that closes and reopens every time
/// somebody moves a file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceFile
{
    /// Repo-relative, forward slashes. For reporting.
    pub path: String,
    /// The file's full text.
    pub text: String,
}

impl SourceFile
{
    /// Builds one, for callers that have two strings.
    #[must_use]
    pub fn New(path: impl Into<String>, text: impl Into<String>) -> Self
    {
        return Self {
            path: path.into(),
            text: text.into(),
        };
    }
}
