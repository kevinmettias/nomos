//! [`BlockChangeResponse`], shared by [`super::preview_response::PreviewResponse::Previewed`] and
//! [`super::committed_preview_response::CommittedPreviewResponse`] -- the same "what changed at the
//! block level" vocabulary both an uncommitted and a just-committed edit report.

use nomos_spec_store::BlockChange;
use serde::Serialize;

/// A serializable twin of [`nomos_spec_store::BlockChange`], which does not derive
/// `Serialize`. Tagged `change` rather than the more usual `kind`, since `Added` and
/// `Removed` already carry a field of their own named `kind`.
#[derive(Debug, Serialize)]
#[serde(tag = "change", rename_all = "snake_case")]
pub enum BlockChangeResponse
{
    Added
    {
        ordinal: u32, kind: String
    },
    Removed
    {
        ordinal: u32, kind: String
    },
    /// Same position, different wording.
    Reworded
    {
        ordinal: u32, before: String, after: String
    },
    /// Same wording, different position.
    Moved
    {
        from: u32, to: u32
    },
    /// Same wording, different whitespace.
    Reflowed
    {
        ordinal: u32
    },
}

impl BlockChangeResponse
{
    pub(crate) fn From(change: BlockChange) -> Self
    {
        return match change
        {
            BlockChange::Added { ordinal, kind } => Self::Added { ordinal, kind },
            BlockChange::Removed { ordinal, kind } => Self::Removed { ordinal, kind },
            BlockChange::Reworded { ordinal, before, after } => Self::Reworded { ordinal, before, after },
            BlockChange::Moved { from, to } => Self::Moved { from, to },
            BlockChange::Reflowed { ordinal } => Self::Reflowed { ordinal },
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The ordinal the added block sits at, distinct from every other ordinal below so a
    /// mapping that crossed two variants' ordinals could not pass.
    const ADDED_BLOCK_ORDINAL: u32 = 3;
    /// The removed block's ordinal.
    const REMOVED_BLOCK_ORDINAL: u32 = 4;
    /// The reworded block's ordinal -- the same position as its domain counterpart, which is
    /// what "same position, different wording" means.
    const REWORDED_BLOCK_ORDINAL: u32 = 5;
    /// Where the moved block lands. Its origin is `1`, which stays literal since it is the
    /// identity end of the move rather than a choice.
    const MOVED_TO_ORDINAL: u32 = 2;
    /// The reflowed block's ordinal.
    const REFLOWED_BLOCK_ORDINAL: u32 = 6;

    #[test]
    fn Test_From_Should_Map_Every_Domain_Variant_To_Its_Own_Response_Variant()
    {
        assert!(matches!(
            BlockChangeResponse::From(BlockChange::Added { ordinal: ADDED_BLOCK_ORDINAL, kind: "table".to_owned() }),
            BlockChangeResponse::Added { ordinal: ADDED_BLOCK_ORDINAL, kind } if kind == "table"
        ));
        assert!(matches!(
            BlockChangeResponse::From(BlockChange::Removed { ordinal: REMOVED_BLOCK_ORDINAL, kind: "list".to_owned() }),
            BlockChangeResponse::Removed { ordinal: REMOVED_BLOCK_ORDINAL, kind } if kind == "list"
        ));
        assert!(matches!(
            BlockChangeResponse::From(BlockChange::Reworded {
                ordinal: REWORDED_BLOCK_ORDINAL,
                before: "old".to_owned(),
                after: "new".to_owned(),
            }),
            BlockChangeResponse::Reworded { ordinal: REWORDED_BLOCK_ORDINAL, before, after } if before == "old" && after == "new"
        ));
        assert!(matches!(
            BlockChangeResponse::From(BlockChange::Moved { from: 1, to: MOVED_TO_ORDINAL }),
            BlockChangeResponse::Moved { from: 1, to: MOVED_TO_ORDINAL }
        ));
        assert!(matches!(
            BlockChangeResponse::From(BlockChange::Reflowed { ordinal: REFLOWED_BLOCK_ORDINAL }),
            BlockChangeResponse::Reflowed { ordinal: REFLOWED_BLOCK_ORDINAL }
        ));
    }
}
