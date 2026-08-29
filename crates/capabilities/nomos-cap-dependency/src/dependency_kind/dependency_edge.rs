//! One first-party dependency edge.

use super::DependencyKind;

/// One edge: this package names `target` as a dependency of kind `kind`, optionally.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DependencyEdge
{
    /// The dependency's own package name, as Cargo resolved it — not the manifest's
    /// `package = "..."` rename, if one was given, because a rule comparing this against
    /// `bands.rs`'s table must compare against the same name the table is written in.
    pub target: String,
    pub kind: DependencyKind,
    pub optional: bool,
}
