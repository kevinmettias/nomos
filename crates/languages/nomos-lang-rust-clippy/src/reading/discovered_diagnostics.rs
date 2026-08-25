//! One workspace member as [`Discover_Workspace`](super::Discover_Workspace) found it.

use nomos_cap_lint::DiagnosticsPayload;

/// One workspace member as this reader found it: its own diagnostics payload, and the
/// repository-relative path its manifest lives at.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscoveredDiagnostics
{
    pub payload: DiagnosticsPayload,
    /// Repository-relative, forward slashes — the same convention `Subject_Of_Path`
    /// takes, because this is what a caller derives this member's subject from.
    pub manifest_relative_root: String,
}
