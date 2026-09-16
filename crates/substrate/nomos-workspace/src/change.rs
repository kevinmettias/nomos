//! The one door.
//!
//! Five things change a workspace and there is one way in. The alternative — a method per
//! source, or a public map somebody can reach into — is not a style preference: it is two
//! answers to what the workspace currently is, and the second one is always discovered
//! after something has been built on the first.

// A change set and the source a change came from are parts of a change.
#[path = "change/set.rs"]
mod set;
#[path = "change/source.rs"]
mod source;

pub use set::Set as ChangeSet;
pub use source::Source as ChangeSource;

use nomos_contracts::Digest128;
use nomos_model::Content_Digest;

/// One change to one path.
///
/// There is no `Moved`. A move is a removal and an addition, and modelling it as its own
/// case would require this crate to decide when two paths hold "the same" content — a
/// judgement that belongs to something that can see history, not to the door.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Change
{
    /// The path now holds this content, whether or not it did before.
    ///
    /// One case rather than `Added` and `Modified`, because the submitter frequently does
    /// not know which it is — a checkout does not diff, and an editor's save hook does not
    /// consult the previous generation. Making them state it would make them guess, and a
    /// wrong guess would have to be either trusted or checked; the workspace already knows
    /// the answer and reports it in [`crate::Effect`].
    Present
    {
        path: String,
        content: String,
    },
    /// The path holds nothing.
    Absent
    {
        path: String,
    },
}

impl Change
{
    /// The path this change concerns, as submitted.
    #[must_use]
    pub fn Path(&self) -> &str
    {
        return match self
        {
            Self::Present { path, .. } | Self::Absent { path } => path,
        };
    }

    /// The digest of the content this change asserts, or `None` for a removal.
    #[must_use]
    pub fn Content_Digest(&self) -> Option<Digest128>
    {
        return match self
        {
            Self::Present { content, .. } => Some(Content_Digest(content.as_bytes())),
            Self::Absent { .. } => None,
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    use crate::WorkspaceChangeSet;

    /// The ordinal each source is recorded at, one per [`ChangeSource::All`] in that list's
    /// own order. `Test_Every_ChangeSource_Should_Be_Matched_Exhaustively` is what holds the
    /// two in step: it asserts every one of these against the position `All` puts its source
    /// at, so a reordering here is a failing assertion rather than a silent one.
    const CORRECTION_ORDINAL: usize = 0;
    const IDE_EDIT_ORDINAL: usize = 1;
    const GIT_CHECKOUT_ORDINAL: usize = 2;
    const AGENT_EDIT_ORDINAL: usize = 3;
    const CODE_GENERATOR_ORDINAL: usize = 4;

    /// How many changes `Test_A_Change_Set_Should_Carry_Its_Source_And_Its_Changes` submits
    /// through the builder — one `Present` and one `Absent`, which is the whole of what the
    /// assertions there are about.
    const CHANGES_IN_THE_SET: usize = 2;

    #[test]
    fn Test_Path_Should_Return_The_Submitted_Path()
    {
        let present = Change::Present {
            path: "src/a.rs".to_owned(),
            content: "pub fn a() {}".to_owned(),
        };
        let absent = Change::Absent {
            path: "src/gone.rs".to_owned(),
        };

        assert_eq!(present.Path(), "src/a.rs");
        assert_eq!(absent.Path(), "src/gone.rs");
    }

    #[test]
    fn Test_A_Change_Set_Should_Carry_Its_Source_And_Its_Changes()
    {
        let set = WorkspaceChangeSet::From(ChangeSource::GitCheckout)
            .Present("src/lib.rs", "pub fn a() {}")
            .Absent("src/old.rs");

        assert_eq!(set.Source(), ChangeSource::GitCheckout);
        assert_eq!(set.Changes().len(), CHANGES_IN_THE_SET);
        assert!(!set.Is_Empty());
    }

    /// A removal has no content digest, and that is not the same as having the digest of
    /// nothing. An empty file exists; a removed one does not, and an `Option` is what
    /// keeps the two from sharing a value.
    #[test]
    fn Test_A_Removal_Should_Have_No_Content_Digest()
    {
        let empty = Change::Present {
            path: "a.rs".to_owned(),
            content: String::new(),
        };
        let gone = Change::Absent {
            path: "a.rs".to_owned(),
        };

        assert!(empty.Content_Digest().is_some(), "an empty file exists");
        assert_eq!(gone.Content_Digest(), None);
    }

    #[test]
    fn Test_Identical_Content_Should_Digest_Identically()
    {
        let one = Change::Present {
            path: "a.rs".to_owned(),
            content: "pub fn shared() {}".to_owned(),
        };
        let other = Change::Present {
            path: "b.rs".to_owned(),
            content: "pub fn shared() {}".to_owned(),
        };

        assert_eq!(one.Content_Digest(), other.Content_Digest());
    }

    /// Every source has a stable spelling. These appear in recorded provenance, and a
    /// label that changes with a refactor is a history that stops joining.
    #[test]
    fn Test_Every_Source_Should_Have_A_Distinct_Label()
    {
        let labels: std::collections::BTreeSet<&str> = ChangeSource::All()
            .iter()
            .map(|source| return source.Label())
            .collect();

        assert_eq!(labels.len(), ChangeSource::All().len());
    }

    /// `ChangeSource::All()`'s own mirror, named in the doc comment above it.
    ///
    /// The match has no wildcard arm. A variant added to `ChangeSource` without a matching
    /// arm added here fails this file to *compile*, not merely to pass — the property
    /// D-134 asks a closed enum's mirror to have.
    #[test]
    fn Test_Every_ChangeSource_Should_Be_Matched_Exhaustively()
    {
        fn Ordinal_Of_Source(source: ChangeSource) -> usize
        {
            return match source
            {
                ChangeSource::Correction => CORRECTION_ORDINAL,
                ChangeSource::IdeEdit => IDE_EDIT_ORDINAL,
                ChangeSource::GitCheckout => GIT_CHECKOUT_ORDINAL,
                ChangeSource::AgentEdit => AGENT_EDIT_ORDINAL,
                ChangeSource::CodeGenerator => CODE_GENERATOR_ORDINAL,
            };
        }

        for (index, source) in ChangeSource::All().iter().enumerate()
        {
            assert_eq!(
                Ordinal_Of_Source(*source),
                index,
                "{} is not matched at the position ChangeSource::All() puts it, so the \
                 exhaustive match and the universe have drifted apart",
                source.Label()
            );
        }
    }
}
