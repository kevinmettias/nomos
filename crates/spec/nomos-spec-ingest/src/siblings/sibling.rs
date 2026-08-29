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
