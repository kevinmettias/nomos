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
    pub const fn Disturbs_Wording(&self) -> bool
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
