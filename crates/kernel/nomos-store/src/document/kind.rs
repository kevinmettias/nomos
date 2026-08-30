//! What sort of thing a stored document is.

use serde::{Deserialize, Serialize};

use crate::Authority;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Kind
{
    Commit,
    Fact,
    Finding,
    Run,
    Specification,
    Record,
    Projection,
}

impl Kind
{
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Commit => "commit",
            Self::Fact => "fact",
            Self::Finding => "finding",
            Self::Run => "run",
            Self::Specification => "specification",
            Self::Record => "record",
            Self::Projection => "projection",
        };
    }

    #[must_use]
    pub const fn Authority(self) -> Authority
    {
        return match self
        {
            Self::Commit | Self::Fact | Self::Finding | Self::Run => Authority::Observed,
            Self::Specification | Self::Record | Self::Projection => Authority::Authored,
        };
    }

    /// Every kind a document can be.
    ///
    /// Mirrored by `Test_Every_Kind_Should_Be_Matched_Exhaustively`, an exhaustive
    /// match over every variant with no wildcard arm, in this file. It fails to compile,
    /// not merely to pass, if a variant is added here without being added there.
    #[must_use]
    pub const fn All() -> &'static [Self]
    {
        return &[
            Self::Commit,
            Self::Fact,
            Self::Finding,
            Self::Run,
            Self::Specification,
            Self::Record,
            Self::Projection,
        ];
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    // Each position below is hand-typed and independent of `Kind::All()`'s own order --
    // naming them does not derive one from the other, which would defeat the point of an
    // independent mirror.
    const FINDING_POSITION: usize = 2;
    const RUN_POSITION: usize = 3;
    const SPECIFICATION_POSITION: usize = 4;
    const RECORD_POSITION: usize = 5;
    const PROJECTION_POSITION: usize = 6;

    /// `Kind::All()`'s own mirror, named in the doc comment above it.
    ///
    /// The match has no wildcard arm. A variant added to `Kind` without a matching
    /// arm added here fails this file to *compile*, not merely to pass — the property
    /// D-134 asks a closed enum's mirror to have.
    #[test]
    fn Test_All_Should_Enumerate_Every_Kind_Exhaustively()
    {
        fn Expected_Ordinal(kind: Kind) -> usize
        {
            return match kind
            {
                Kind::Commit => 0,
                Kind::Fact => 1,
                Kind::Finding => FINDING_POSITION,
                Kind::Run => RUN_POSITION,
                Kind::Specification => SPECIFICATION_POSITION,
                Kind::Record => RECORD_POSITION,
                Kind::Projection => PROJECTION_POSITION,
            };
        }

        for (index, kind) in Kind::All().iter().enumerate()
        {
            assert_eq!(
                Expected_Ordinal(*kind),
                index,
                "{} is not matched at the position Kind::All() puts it, so the \
                 exhaustive match and the universe have drifted apart",
                kind.Label()
            );
        }
    }

    #[test]
    fn Test_Label_Should_Return_A_Stable_Lowercase_Name_For_Every_Kind()
    {
        assert_eq!(Kind::Commit.Label(), "commit");
        assert_eq!(Kind::Fact.Label(), "fact");
        assert_eq!(Kind::Finding.Label(), "finding");
        assert_eq!(Kind::Run.Label(), "run");
        assert_eq!(Kind::Specification.Label(), "specification");
        assert_eq!(Kind::Record.Label(), "record");
        assert_eq!(Kind::Projection.Label(), "projection");
    }

    #[test]
    fn Test_Authority_Should_Assign_Every_Kind_To_Exactly_One_Authority()
    {
        assert_eq!(Kind::Commit.Authority(), Authority::Observed);
        assert_eq!(Kind::Fact.Authority(), Authority::Observed);
        assert_eq!(Kind::Finding.Authority(), Authority::Observed);
        assert_eq!(Kind::Run.Authority(), Authority::Observed);
        assert_eq!(Kind::Specification.Authority(), Authority::Authored);
        assert_eq!(Kind::Record.Authority(), Authority::Authored);
        assert_eq!(Kind::Projection.Authority(), Authority::Authored);
    }
}
