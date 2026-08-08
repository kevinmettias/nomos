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
