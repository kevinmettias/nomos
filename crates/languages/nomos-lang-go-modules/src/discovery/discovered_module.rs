//! One Go module as [`Discover_Workspace`](super::Discover_Workspace) found it.

use nomos_cap_dependency::DependencyPayload;

/// One Go module as this reader found it: its own dependency payload, and the
/// repository-relative directory its `go.mod` lives in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscoveredModule
{
    pub payload: DependencyPayload,
    /// Repository-relative, forward slashes — the same convention
    /// `nomos_lang_rust_cargo::DiscoveredPackage::manifest_relative_root` takes, because
    /// this is what a caller derives this module's subject from.
    pub manifest_relative_root: String,
}
