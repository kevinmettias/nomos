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

    /// `Kind::All()`'s own mirror, named in the doc comment above it.
    ///
    /// The match has no wildcard arm. A variant added to `Kind` without a matching
    /// arm added here fails this file to *compile*, not merely to pass — the property
    /// D-134 asks a closed enum's mirror to have.
    #[test]
    fn Test_Every_Kind_Should_Be_Matched_Exhaustively()
    {
        fn Expected_Ordinal(kind: Kind) -> usize
        {
            // Each position below is hand-typed and independent of `Kind::All()`'s own
            // order -- naming them does not derive one from the other, which would defeat the
            // point of an independent mirror.
            const FINDING_POSITION: usize = 2;
            const RUN_POSITION: usize = 3;
            const SPECIFICATION_POSITION: usize = 4;
            const RECORD_POSITION: usize = 5;
            const PROJECTION_POSITION: usize = 6;

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
}
