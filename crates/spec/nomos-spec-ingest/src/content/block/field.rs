//! Which field of a block disagreed.

/// Which field of a block disagreed.
///
/// The two hashes are separate answers rather than one, because the two disagreeing is
/// the whole signal: a block whose content moved but whose normalization did not is a
/// reformatting, and one where both moved is an edit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field
{
    Kind,
    ContentHash,
    NormalizedHash,
}

impl Field
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

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Label_Should_Spell_Each_Field_As_The_Manifest_Does()
    {
        assert_eq!(Field::Kind.Label(), "kind");
        assert_eq!(Field::ContentHash.Label(), "content_hash");
        assert_eq!(Field::NormalizedHash.Label(), "normalized_hash");
    }
}
