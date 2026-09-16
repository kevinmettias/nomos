//! What became of one block.

/// What became of one block.
///
/// [`BlockChange::Reflowed`] is separate from [`BlockChange::Reworded`] on the normalizer's
/// authority: the normalized hashes are equal, so v14's own definition of same-content says
/// the wording did not change. Collapsing the two would make every reflowed paragraph
/// report as moved wording, and a preview that cries wolf is a preview people stop reading.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockChange
{
    Added
    {
        ordinal: u32,
        kind: String,
    },
    Removed
    {
        ordinal: u32,
        kind: String,
    },
    /// Same position, different wording.
    Reworded
    {
        ordinal: u32,
        before: String,
        after: String,
    },
    /// Same wording, different position.
    Moved
    {
        from: u32,
        to: u32,
    },
    /// Same wording, different whitespace.
    Reflowed
    {
        ordinal: u32,
    },
}

impl BlockChange
{
    /// Whether this change moves, rewrites or removes wording that was already there.
    #[must_use]
    pub const fn Is_Disturbing_Wording(&self) -> bool
    {
        return matches!(
            self,
            Self::Removed { .. } | Self::Reworded { .. } | Self::Moved { .. }
        );
    }

    #[must_use]
    pub fn Describe(&self) -> String
    {
        return match self
        {
            Self::Added { ordinal, kind } => format!("block {ordinal} added ({kind})"),
            Self::Removed { ordinal, kind } => format!("block {ordinal} removed ({kind})"),
            Self::Reworded {
                ordinal,
                before,
                after,
            } => format!("block {ordinal} reworded, {before} -> {after}"),
            Self::Moved { from, to } => format!("block {from} moved to {to}, wording unchanged"),
            Self::Reflowed { ordinal } =>
            {
                format!("block {ordinal} reflowed, wording unchanged under the normalizer")
            }
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The ordinal a moved block reports as its destination.
    const MOVED_TO_ORDINAL: u32 = 2;

    /// The ordinal an added block reports.
    const ADDED_ORDINAL: u32 = 3;

    #[test]
    fn Test_Is_Disturbing_Wording_Should_Be_True_Only_When_Wording_Actually_Changed()
    {
        assert!(BlockChange::Removed { ordinal: 1, kind: "prose".to_owned() }.Is_Disturbing_Wording());
        assert!(
            BlockChange::Reworded {
                ordinal: 1,
                before: "a".to_owned(),
                after: "b".to_owned()
            }
            .Is_Disturbing_Wording()
        );
        assert!(BlockChange::Moved { from: 1, to: MOVED_TO_ORDINAL }.Is_Disturbing_Wording());
        assert!(!BlockChange::Added { ordinal: 1, kind: "prose".to_owned() }.Is_Disturbing_Wording());
        assert!(!BlockChange::Reflowed { ordinal: 1 }.Is_Disturbing_Wording());
    }

    #[test]
    fn Test_Describe_Should_Name_The_Ordinal_And_What_Happened_To_It()
    {
        assert_eq!(
            BlockChange::Added { ordinal: ADDED_ORDINAL, kind: "prose".to_owned() }.Describe(),
            "block 3 added (prose)"
        );
        assert_eq!(BlockChange::Moved { from: 1, to: MOVED_TO_ORDINAL }.Describe(), "block 1 moved to 2, wording unchanged");
    }
}
