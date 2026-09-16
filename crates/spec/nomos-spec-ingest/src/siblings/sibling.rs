/// A specification that is not this repository's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Sibling
{
    Xvpe,
    Kwb,
    Ecosystem,
}

impl Sibling
{
    #[must_use]
    pub const fn Suite_Id(self) -> &'static str
    {
        return match self
        {
            Self::Xvpe => "xvpe-spec-seed",
            Self::Kwb => "kwb-spec-seed",
            Self::Ecosystem => "ecosystem-contracts",
        };
    }

    #[must_use]
    pub const fn Title(self) -> &'static str
    {
        return match self
        {
            Self::Xvpe => "XVPE specification seed",
            Self::Kwb => "KnowledgeWorkbench specification seed",
            Self::Ecosystem => "Ecosystem contracts",
        };
    }

    #[must_use]
    pub const fn Archive(self) -> &'static str
    {
        return match self
        {
            Self::Xvpe => "xvpe-spec-seed-v0.1.zip",
            Self::Kwb => "kwb-spec-seed-v0.1.zip",
            Self::Ecosystem => "ecosystem-contracts-v0.1.zip",
        };
    }

    /// Every specification not this repository's own.
    ///
    /// Mirrored by `Test_Every_Sibling_Should_Be_Matched_Exhaustively`, an exhaustive
    /// match over every variant with no wildcard arm, in
    /// `crates/spec/nomos-spec-ingest/src/siblings/tests.rs`. It fails to compile, not
    /// merely to pass, if a variant is added here without being added there.
    #[must_use]
    pub const fn All() -> &'static [Self]
    {
        return &[Self::Xvpe, Self::Kwb, Self::Ecosystem];
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The variants `Sibling::All()` lists. The assertion below is that the universe holds
    /// this many *and* that each appears once, so a duplicate plus a dropped variant — which
    /// leaves the length alone — still fails the uniqueness check beside it.
    const SIBLINGS_IN_ALL: usize = 3;

    #[test]
    fn Test_Suite_Id_Should_Give_Each_Sibling_Its_Own_Suite_Identifier()
    {
        assert_eq!(Sibling::Xvpe.Suite_Id(), "xvpe-spec-seed");
        assert_eq!(Sibling::Kwb.Suite_Id(), "kwb-spec-seed");
        assert_eq!(Sibling::Ecosystem.Suite_Id(), "ecosystem-contracts");
    }

    #[test]
    fn Test_Title_Should_Give_Each_Sibling_A_Human_Readable_Name()
    {
        assert_eq!(Sibling::Xvpe.Title(), "XVPE specification seed");
        assert_eq!(Sibling::Kwb.Title(), "KnowledgeWorkbench specification seed");
        assert_eq!(Sibling::Ecosystem.Title(), "Ecosystem contracts");
    }

    #[test]
    fn Test_Archive_Should_Name_Each_Siblings_Own_Zip_File()
    {
        assert_eq!(Sibling::Xvpe.Archive(), "xvpe-spec-seed-v0.1.zip");
        assert_eq!(Sibling::Kwb.Archive(), "kwb-spec-seed-v0.1.zip");
        assert_eq!(Sibling::Ecosystem.Archive(), "ecosystem-contracts-v0.1.zip");
    }

    #[test]
    fn Test_All_Should_List_Every_Sibling_With_No_Duplicate()
    {
        let all = Sibling::All();

        assert_eq!(all.len(), SIBLINGS_IN_ALL);
        let unique: std::collections::BTreeSet<_> = all.iter().collect();
        assert_eq!(unique.len(), all.len());
    }
}
