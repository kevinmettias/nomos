//! Where a change was observed.
//!
//! # Why this file sits at the crate root rather than inside `change/`
//!
//! It was `change/source.rs`, declaring `Source`, and `change.rs` published it as
//! `pub use source::Source as ChangeSource;`. Three rules pull against that shape, and
//! moving the file here is the only arrangement that satisfies all three at once:
//!
//! - `file-name-matches-declared-type` requires a file to be named after the public type it
//!   declares, so a type called `ChangeSource` forces the file to be `change_source`.
//! - `check-tree-legibility` forbids a file repeating its parent folder, so
//!   `change/change_source.rs` is out.
//! - `check-facade-surface` counts `pub use x::Y as Z;` a second public name for one item, so
//!   keeping `Source` and qualifying it at the facade is out too.
//!
//! The public path is unchanged: `lib.rs` re-exports this, so
//! `nomos_workspace::ChangeSource` is what it always was.

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

    /// Every source a change can be observed from.
    ///
    /// Mirrored by `Test_Every_ChangeSource_Should_Be_Matched_Exhaustively`, an exhaustive
    /// match over every variant with no wildcard arm, in
    /// `crates/substrate/nomos-workspace/src/change.rs`. It fails to compile, not merely
    /// to pass, if a variant is added here without being added there.
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

#[cfg(test)]
mod tests
{
    use super::*;

    /// One per variant `ChangeSource::All` lists. `Label`'s exhaustive match over every variant is
    /// what keeps that list and the enum in step, so this pins the count.
    const SOURCE_COUNT: usize = 5;

    #[test]
    fn Test_Label_Should_Spell_Every_Source_Distinctly()
    {
        assert_eq!(ChangeSource::Correction.Label(), "Correction");
        assert_eq!(ChangeSource::AgentEdit.Label(), "AgentEdit");
        assert_ne!(ChangeSource::IdeEdit.Label(), ChangeSource::GitCheckout.Label());
    }

    #[test]
    fn Test_All_Should_List_Every_Source_Exactly_Once()
    {
        let all = ChangeSource::All();

        assert_eq!(all.len(), SOURCE_COUNT);
        assert!(all.contains(&ChangeSource::Correction));
        assert!(all.contains(&ChangeSource::CodeGenerator));

        let unique: std::collections::BTreeSet<_> = all.iter().collect();
        assert_eq!(unique.len(), all.len());
    }
}
