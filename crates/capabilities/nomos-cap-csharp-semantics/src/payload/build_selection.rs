//! [`BuildSelection`], which build an answer is about.

/// The build an answer judges its branches under: one project, one configuration, one target
/// framework.
///
/// A conditional region is compiled or skipped only relative to a build, and one project builds
/// differently per configuration and per target framework, so an answer that did not name all
/// three would be a claim about no build in particular.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildSelection
{
    /// The project file, as its path relative to the repository root.
    pub project: String,
    /// The build configuration, such as `Debug` or `Release`.
    pub configuration: String,
    /// The target framework moniker, such as `net8.0`.
    pub target_framework: String,
}
