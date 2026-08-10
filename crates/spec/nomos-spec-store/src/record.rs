use nomos_spec_model::BlockKind;

#[must_use]
pub const fn Kind_Label(kind: BlockKind) -> &'static str
{
    return match kind
    {
        BlockKind::Heading => "heading",
        BlockKind::Prose => "prose",
        BlockKind::Code => "code",
    };
}

/// The kind a stored label names.
///
/// [`Kind_Label`]'s inverse, and its counterpart rather than a second opinion: reading a
/// block back out of the store needs the label to mean what writing it meant, and
/// `Test_A_Block_Kind_Should_Survive_The_Label` is what says the pair is one mapping.
/// `None` for a label this build does not know, so a store written by a newer one reads as
/// unknown rather than as prose.
#[must_use]
pub fn Kind_Of(label: &str) -> Option<BlockKind>
{
    return [BlockKind::Heading, BlockKind::Prose, BlockKind::Code]
        .into_iter()
        .find(|kind| return Kind_Label(*kind) == label);
}

/// What became of a source block.
///
/// `PreservedVerbatim` and `PreservedNormalized` are the only dispositions that satisfy
/// NSV-PRESERVE-006. `Omitted` requires a row in `omissions`, which carries the reason
/// and the decision record; it is the sanctioned exit, not a free one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Disposition
{
    PreservedVerbatim,
    PreservedNormalized,
    Superseded,
    RegressionFiller,
    Omitted,
}

impl Disposition
{
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::PreservedVerbatim => "preserved-verbatim",
            Self::PreservedNormalized => "preserved-normalized",
            Self::Superseded => "superseded",
            Self::RegressionFiller => "regression-filler",
            Self::Omitted => "omitted",
        };
    }

    #[must_use]
    pub const fn Preserves_Content(self) -> bool
    {
        return matches!(self, Self::PreservedVerbatim | Self::PreservedNormalized);
    }

    #[must_use]
    pub fn Parse(label: &str) -> Option<Self>
    {
        return [
            Self::PreservedVerbatim,
            Self::PreservedNormalized,
            Self::Superseded,
            Self::RegressionFiller,
            Self::Omitted,
        ]
        .into_iter()
        .find(|disposition| disposition.Label() == label);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Dispositions_Should_Round_Trip()
    {
        for disposition in [
            Disposition::PreservedVerbatim,
            Disposition::PreservedNormalized,
            Disposition::Superseded,
            Disposition::RegressionFiller,
            Disposition::Omitted,
        ]
        {
            assert_eq!(Disposition::Parse(disposition.Label()), Some(disposition));
        }
        assert_eq!(Disposition::Parse("invented"), None);
    }

    #[test]
    fn Test_A_Block_Kind_Should_Survive_The_Label()
    {
        for kind in [BlockKind::Heading, BlockKind::Prose, BlockKind::Code]
        {
            assert_eq!(Kind_Of(Kind_Label(kind)), Some(kind));
        }
        assert_eq!(Kind_Of("paragraph"), None);
    }

    /// Filler is what v15.0 shipped in place of real content. If it ever counts as
    /// preservation, the rule that would have caught that regression stops working.
    #[test]
    fn Test_Only_Preserved_Dispositions_Should_Count_As_Preservation()
    {
        assert!(Disposition::PreservedVerbatim.Preserves_Content());
        assert!(Disposition::PreservedNormalized.Preserves_Content());
        assert!(!Disposition::Superseded.Preserves_Content());
        assert!(!Disposition::RegressionFiller.Preserves_Content());
        assert!(!Disposition::Omitted.Preserves_Content());
    }
}
