//! The kind of problem a record names, and the tag each kind is written with.

/// One of the five ways `crate::predicates` already reports a stale or incomplete
/// assessment — kept as five variants, not collapsed to one "stale" tag, because a rule
/// reading this payload reconstructs one `Finding` per [`Problem`] and a reader comparing
/// two runs needs to tell which of the five kinds moved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProblemKind
{
    /// A `site` line names a path or symbol that no longer resolves in this workspace.
    UnresolvedSite,
    /// A `gap` line (a `Partial` entry's own unsatisfied part) no longer resolves.
    UnresolvedGap,
    /// A `record` line names a governing record with no registration, or a registration
    /// with no document.
    UnresolvedRecord,
    /// A `Diverges` or `NotBinding` entry names no governing record at all.
    DivergenceWithNoRecord,
    /// A `Partial` entry names no gap at all.
    PartialWithNoGap,
}

impl ProblemKind
{
    const SITE_TAG: &'static str = "site";
    const GAP_TAG: &'static str = "gap";
    const RECORD_TAG: &'static str = "record";
    const DIVERGES_TAG: &'static str = "diverges";
    const PARTIAL_TAG: &'static str = "partial";

    /// The tag this kind is written with in [`Encode_Payload`]'s own tab-separated lines.
    pub(crate) const fn Tag(self) -> &'static str
    {
        return match self
        {
            Self::UnresolvedSite => Self::SITE_TAG,
            Self::UnresolvedGap => Self::GAP_TAG,
            Self::UnresolvedRecord => Self::RECORD_TAG,
            Self::DivergenceWithNoRecord => Self::DIVERGES_TAG,
            Self::PartialWithNoGap => Self::PARTIAL_TAG,
        };
    }

    /// The kind a tag names, or `None` if it names none of the five.
    pub(crate) fn Of_Tag(tag: &str) -> Option<Self>
    {
        return match tag
        {
            Self::SITE_TAG => Some(Self::UnresolvedSite),
            Self::GAP_TAG => Some(Self::UnresolvedGap),
            Self::RECORD_TAG => Some(Self::UnresolvedRecord),
            Self::DIVERGES_TAG => Some(Self::DivergenceWithNoRecord),
            Self::PARTIAL_TAG => Some(Self::PartialWithNoGap),
            _ => None,
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// Every kind, and the tag it is written with -- one row per kind. It is named and it is
    /// declared outside both tests, so a sixth kind is a diff to this table and not a second copy
    /// of a test that has to be kept in step with it.
    const KINDS_AND_TAGS: [(ProblemKind, &str); 5] = [
        (ProblemKind::UnresolvedSite, "site"),
        (ProblemKind::UnresolvedGap, "gap"),
        (ProblemKind::UnresolvedRecord, "record"),
        (ProblemKind::DivergenceWithNoRecord, "diverges"),
        (ProblemKind::PartialWithNoGap, "partial"),
    ];

    /// The tag a kind is written with, which is the whole of what a reader of an encoded payload
    /// has to go on: two kinds sharing one tag would be two kinds nobody can tell apart.
    #[test]
    fn Test_Tag_Should_Write_Each_Kind_With_Its_Own_Tag()
    {
        for (kind, tag) in KINDS_AND_TAGS
        {
            assert_eq!(kind.Tag(), tag, "{kind:?} is written with its own tag");
        }
    }

    /// ...and reads back to exactly the kind that tag names, which is what makes the kind field of
    /// an encoded payload unambiguous. A tag naming none of the five answers nothing rather than a
    /// nearest match, which is the answer [`Parse_Payload`] turns into a refusal.
    #[test]
    fn Test_Of_Tag_Should_Read_Back_The_Kind_Each_Tag_Names()
    {
        for (kind, tag) in KINDS_AND_TAGS
        {
            assert_eq!(ProblemKind::Of_Tag(tag), Some(kind), "{tag} names the kind it is written with");
        }

        assert_eq!(ProblemKind::Of_Tag("mystery"), None, "a tag naming no kind is refused, never guessed at");
    }
}
