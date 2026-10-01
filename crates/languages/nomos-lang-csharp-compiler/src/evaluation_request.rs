//! [`EvaluationRequest`], which build to ask `MSBuild` about.

use std::path::PathBuf;

/// Which build to ask `MSBuild` for the definition set of.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvaluationRequest
{
    /// The repository root, where `MSBuild` is run from.
    pub root: PathBuf,
    /// The project file, relative to [`Self::root`].
    pub project: String,
    /// The build configuration, such as `Debug`.
    pub configuration: String,
    /// The target framework to evaluate, or `None` to take the project's own when it declares
    /// exactly one. A project declaring several has no single answer, and is refused rather than
    /// answered for whichever it lists first.
    pub target_framework: Option<String>,
}
