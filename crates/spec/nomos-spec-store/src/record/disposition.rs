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
    pub const fn Is_Preserving_Content(self) -> bool
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
    fn Test_Label_Should_Name_Every_Known_Disposition()
    {
        assert_eq!(Disposition::PreservedVerbatim.Label(), "preserved-verbatim");
        assert_eq!(Disposition::PreservedNormalized.Label(), "preserved-normalized");
        assert_eq!(Disposition::Superseded.Label(), "superseded");
        assert_eq!(Disposition::RegressionFiller.Label(), "regression-filler");
        assert_eq!(Disposition::Omitted.Label(), "omitted");
    }

    #[test]
    fn Test_Parse_Should_Recover_The_Disposition_Its_Own_Text_Names()
    {
        for disposition in All_Dispositions()
        {
            assert_eq!(Disposition::Parse(disposition.Label()), Some(disposition));
        }
        assert_eq!(Disposition::Parse("invented"), None);
    }

    fn All_Dispositions() -> [Disposition; 5]
    {
        return [
            Disposition::PreservedVerbatim,
            Disposition::PreservedNormalized,
            Disposition::Superseded,
            Disposition::RegressionFiller,
            Disposition::Omitted,
        ];
    }

    /// Filler is what v15.0 shipped in place of real content. If it ever counts as
    /// preservation, the rule that would have caught that regression stops working.
    #[test]
    fn Test_Is_Preserving_Content_Should_Be_True_Only_For_Preserved_Dispositions()
    {
        assert!(Disposition::PreservedVerbatim.Is_Preserving_Content());
        assert!(Disposition::PreservedNormalized.Is_Preserving_Content());
        assert!(!Disposition::Superseded.Is_Preserving_Content());
        assert!(!Disposition::RegressionFiller.Is_Preserving_Content());
        assert!(!Disposition::Omitted.Is_Preserving_Content());
    }
}
