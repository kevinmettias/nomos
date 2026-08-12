//! Which field of a block disagreed.

/// Which field of a block disagreed.
///
/// The two hashes are separate answers rather than one, because the two disagreeing is
/// the whole signal: a block whose content moved but whose normalization did not is a
/// reformatting, and one where both moved is an edit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockField
{
    Kind,
    ContentHash,
    NormalizedHash,
}

impl BlockField
{
    /// The field's name as the manifest spells it.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Kind => "kind",
            Self::ContentHash => "content_hash",
            Self::NormalizedHash => "normalized_hash",
        };
    }
}
