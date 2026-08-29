//! What sort of thing a stored document is.

use serde::{Deserialize, Serialize};

use crate::Authority;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DocumentKind
{
    Commit,
    Fact,
    Finding,
    Run,
    Specification,
    Record,
    Projection,
}

impl DocumentKind
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
    /// Mirrored by `Test_Every_DocumentKind_Should_Be_Matched_Exhaustively`, an exhaustive
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

    /// `DocumentKind::All()`'s own mirror, named in the doc comment above it.
    ///
    /// The match has no wildcard arm. A variant added to `DocumentKind` without a matching
    /// arm added here fails this file to *compile*, not merely to pass — the property
    /// D-134 asks a closed enum's mirror to have.
    #[test]
    fn Test_Every_DocumentKind_Should_Be_Matched_Exhaustively()
    {
        fn Expected_Ordinal(kind: DocumentKind) -> usize
        {
            return match kind
            {
                DocumentKind::Commit => 0,
                DocumentKind::Fact => 1,
                DocumentKind::Finding => 2,
                DocumentKind::Run => 3,
                DocumentKind::Specification => 4,
                DocumentKind::Record => 5,
                DocumentKind::Projection => 6,
            };
        }

        for (index, kind) in DocumentKind::All().iter().enumerate()
        {
            assert_eq!(
                Expected_Ordinal(*kind),
                index,
                "{} is not matched at the position DocumentKind::All() puts it, so the \
                 exhaustive match and the universe have drifted apart",
                kind.Label()
            );
        }
    }
}
