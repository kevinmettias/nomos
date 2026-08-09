//! The one door.
//!
//! Five things change a workspace and there is one way in. The alternative — a method per
//! source, or a public map somebody can reach into — is not a style preference: it is two
//! answers to what the workspace currently is, and the second one is always discovered
//! after something has been built on the first.

use nomos_contracts::Digest128;
use nomos_model::Content_Digest;

/// Who submitted a change.
///
/// Recorded because provenance is a fact about the change, and because the sources behave
/// differently in ways somebody will eventually need to see: a git checkout arrives as
/// hundreds of changes at once, an IDE save as one, and an agent's edit is the one a
/// person will want to find again.
///
/// It is deliberately **not** part of the workspace's identity. Two workspaces holding the
/// same files are the same workspace however the files got there, and keying the snapshot
/// on the source would make an agent's edit and a human's edit of identical content two
/// different states to analyze.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ChangeSource
{
    /// A correction applied by the system to its own recorded state.
    Correction,
    /// An editor wrote a file.
    IdeEdit,
    /// The working tree moved to another revision.
    GitCheckout,
    /// An agent edited a file.
    AgentEdit,
    /// A generator produced a file.
    CodeGenerator,
}

impl ChangeSource
{
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Correction => "Correction",
            Self::IdeEdit => "IdeEdit",
            Self::GitCheckout => "GitCheckout",
            Self::AgentEdit => "AgentEdit",
            Self::CodeGenerator => "CodeGenerator",
        };
    }

    #[must_use]
    pub const fn All() -> &'static [Self]
    {
        return &[
            Self::Correction,
            Self::IdeEdit,
            Self::GitCheckout,
            Self::AgentEdit,
            Self::CodeGenerator,
        ];
    }
}

impl core::fmt::Display for ChangeSource
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}

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

/// A batch of changes from one source, applied as one step.
///
/// A batch rather than a change, because a checkout that moved four hundred files is one
/// event. Applying them one at a time would produce four hundred generations, and every
/// intermediate one would describe a tree that never existed — a half-applied checkout is
/// not a state anybody should be able to ask questions about.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceChangeSet
{
    source: ChangeSource,
    changes: Vec<Change>,
}

/// The name this type is known by where the distinction from [`Change`] is already clear.
pub type ChangeSet = WorkspaceChangeSet;

impl WorkspaceChangeSet
{
    #[must_use]
    pub const fn From(source: ChangeSource) -> Self
    {
        return Self {
            source,
            changes: Vec::new(),
        };
    }

    #[must_use]
    pub fn Present(mut self, path: impl Into<String>, content: impl Into<String>) -> Self
    {
        self.changes.push(Change::Present {
            path: path.into(),
            content: content.into(),
        });

        return self;
    }

    #[must_use]
    pub fn Absent(mut self, path: impl Into<String>) -> Self
    {
        self.changes.push(Change::Absent { path: path.into() });

        return self;
    }

    #[must_use]
    pub const fn Source(&self) -> ChangeSource
    {
        return self.source;
    }

    #[must_use]
    pub fn Changes(&self) -> &[Change]
    {
        return &self.changes;
    }

    #[must_use]
    pub fn Is_Empty(&self) -> bool
    {
        return self.changes.is_empty();
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_A_Change_Set_Should_Carry_Its_Source_And_Its_Changes()
    {
        let set = WorkspaceChangeSet::From(ChangeSource::GitCheckout)
            .Present("src/lib.rs", "pub fn a() {}")
            .Absent("src/old.rs");

        assert_eq!(set.Source(), ChangeSource::GitCheckout);
        assert_eq!(set.Changes().len(), 2);
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
}
