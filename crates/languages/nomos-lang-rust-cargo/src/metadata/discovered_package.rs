//! One workspace member as [`Discover_Workspace`](super::Discover_Workspace) found it.

use nomos_cap_dependency::DependencyPayload;

/// One workspace member as this reader found it: its own dependency payload, and the
/// repository-relative path its manifest lives at.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscoveredPackage
{
    pub payload: DependencyPayload,
    /// Repository-relative, forward slashes — the same convention `Subject_Of_Path`
    /// takes, because this is what a caller derives this package's subject from.
    pub manifest_relative_root: String,
}
