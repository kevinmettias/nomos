//! One package as cargo's dependency graph describes it.
//!
//! The data half of what [`crate::Workspace`] reads. It is a record rather than a reader:
//! everything that runs `cargo metadata` and interprets its output lives next door, and a
//! test that only needs to ask a package what it depends on does not have to read that.

use std::collections::BTreeSet;
use std::path::PathBuf;

/// One package in the workspace's dependency graph.
#[derive(Clone, Debug)]
pub struct Package
{
    /// The package name.
    pub name: String,
    /// Whether it is a member of this workspace, as opposed to a registry dependency.
    pub is_workspace_member: bool,
    /// The directory containing its manifest.
    pub root: PathBuf,
    /// The names it depends on directly, excluding dev-dependencies.
    ///
    /// Dev-dependencies are excluded from the band ordering deliberately. They do not
    /// ship, cargo permits them to be cyclic, and an upward one is legitimate — a low
    /// crate's tests may reasonably use a higher-level fixture. Counting them would
    /// force that fixture to be duplicated downward, which trades a real architectural
    /// property for a bookkeeping one.
    pub direct_dependencies: BTreeSet<String>,
}
