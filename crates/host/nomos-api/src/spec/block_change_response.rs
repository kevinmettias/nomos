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

    #[test]
    fn Test_From_Should_Map_Every_Domain_Variant_To_Its_Own_Response_Variant()
    {
        assert!(matches!(
            BlockChangeResponse::From(BlockChange::Added { ordinal: 3, kind: "table".to_owned() }),
            BlockChangeResponse::Added { ordinal: 3, kind } if kind == "table"
        ));
        assert!(matches!(
            BlockChangeResponse::From(BlockChange::Removed { ordinal: 4, kind: "list".to_owned() }),
            BlockChangeResponse::Removed { ordinal: 4, kind } if kind == "list"
        ));
        assert!(matches!(
            BlockChangeResponse::From(BlockChange::Reworded {
                ordinal: 5,
                before: "old".to_owned(),
                after: "new".to_owned(),
            }),
            BlockChangeResponse::Reworded { ordinal: 5, before, after } if before == "old" && after == "new"
        ));
        assert!(matches!(
            BlockChangeResponse::From(BlockChange::Moved { from: 1, to: 2 }),
            BlockChangeResponse::Moved { from: 1, to: 2 }
        ));
        assert!(matches!(
            BlockChangeResponse::From(BlockChange::Reflowed { ordinal: 6 }),
            BlockChangeResponse::Reflowed { ordinal: 6 }
        ));
    }
}
