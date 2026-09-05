//! What a correction run found, and what it did about it.

/// What [`crate::Run_Correction`] found and did, apart from choosing a platform, walking a
/// tree, or rendering the answer to a human or a wire.
///
/// One variant per decision `nomos-cli`'s own `correct.rs` used to make inline, carrying
/// exactly what a caller needs to reproduce that decision's own report — a renderer maps
/// each variant to text and an exit code; a wire caller maps it to JSON. Neither renders
/// here: this type is the seam's own answer, not either host's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CorrectionOutcome
{
    /// The root named by [`crate::CorrectionCommand::root`] is not a directory.
    UnreadableRoot,
    /// The walk under the root found no `.rs` or `.go` source at all.
    NoSourceFound,
    /// The tree could not be read as a workspace state.
    UnreadableWorkspaceState,
    /// This build's own capability registry is self-contradictory.
    ContradictoryRegistry(String),
    /// Source was read but no syntax fact was materialized for any of it.
    NoFactsMaterialized(usize),
    /// No blocking phantom-mirror claim was found; the tree is clean.
    Clean,
    /// A real claim was found but could not be safely corrected -- its claim line is not
    /// exactly once in the file, or the plan does not stage, validate or commit against
    /// the file's own live content.
    Refused(String),
    /// Staged and validated, not committed. `preview` is the plan's own rendered preview.
    Staged
    {
        path: String,
        claimed: String,
        preview: Vec<u8>,
    },
    /// Staged, validated and committed. The file named by `path` was written to disk at
    /// `after`; `base` and `after_snapshot` are the workspace's own snapshots before and
    /// after the commit, rendered through their own `Display`.
    Committed
    {
        path: String,
        claimed: String,
        preview: Vec<u8>,
        base: String,
        after_snapshot: String,
    },
}
