//! The build this suite ingests as, and the one door everything arrives through.
//!
//! Held in one place because a permutation compared against a baseline built from a
//! different variant would be comparing two workspaces, not two arrival orders.

pub(crate) use nomos_contracts::{ConfigurationId, Digest128};
pub(crate) use nomos_workspace::{ChangeSource, Workspace, WorkspaceChangeSet};

pub(crate) fn Variant() -> nomos_workspace::BuildVariant
{
    return nomos_workspace::BuildVariant::New(
        "x86_64-pc-windows-msvc",
        "dev",
        "1.85",
        ["analysis", "telemetry"],
    );
}

pub(crate) fn Configuration() -> ConfigurationId
{
    return ConfigurationId::From_Digest(Digest128::From_Bytes([0x2f; 16]));
}

pub(crate) fn Fresh() -> Workspace
{
    return Workspace::Empty(Variant(), Configuration());
}

/// Ingests a corpus in the order given, as one change set per batch of `stride` files.
///
/// Batched rather than one set per file, because that is what a real source does: a
/// checkout arrives as one event covering hundreds of paths. It also means the permutation
/// test permutes across change-set boundaries rather than only within one.
pub(crate) fn Ingest(workspace: &mut Workspace, members: &[(String, String)], stride: usize)
{
    for batch in members.chunks(stride.max(1))
    {
        let set = Change_Set(batch);

        workspace
            .Apply(&set)
            .expect("every path in the corpus is workspace-relative");
    }
}

/// One batch of files, as the single change set a checkout would arrive as.
pub(crate) fn Change_Set(batch: &[(String, String)]) -> WorkspaceChangeSet
{
    let mut set = WorkspaceChangeSet::From(ChangeSource::GitCheckout);
    for (path, content) in batch
    {
        set = set.Present(path.clone(), content.clone());
    }

    return set;
}
