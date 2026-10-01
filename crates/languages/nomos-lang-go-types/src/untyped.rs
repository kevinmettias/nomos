//! [`Untyped`], a part of what was asked about that no fact answers.

use crate::TypesFailure;

/// A part of the Go sources asked about that no fact answers, and why -- reported by whoever holds
/// the answer, and never read as a file that discards nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Untyped
{
    /// A module the helper gave no answer for at all.
    Module
    {
        /// The module's directory, relative to the root, empty for the root itself.
        path: String,
        /// Which kind of failure.
        failure: TypesFailure,
        /// What happened, in the tool's own words where it gave any.
        reason: String,
    },
    /// A package that did not load or type-check, so none of its files was judged.
    Package
    {
        /// Its import path, as `go list` names it.
        package: String,
        /// The sources asked about that it holds, relative to the root.
        files: Vec<String>,
        /// The toolchain's own words.
        reason: String,
    },
    /// Sources inside a module that no package the module compiles on this host holds: a build
    /// constraint excludes them, they sit where `./...` does not look -- `testdata`, `vendor`, a
    /// directory named with a leading `_` or `.` -- or they are cgo sources, which the compiler
    /// sees only as generated files.
    Unchecked
    {
        /// The module's directory, relative to the root, empty for the root itself.
        module: String,
        /// Each such source, relative to the root.
        files: Vec<String>,
    },
    /// Go sources that sit under no `go.mod`, so no module holds them and `go` has no package to
    /// load them in.
    Outside
    {
        /// Each such source, relative to the root.
        files: Vec<String>,
    },
}
