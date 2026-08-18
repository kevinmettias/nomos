//! Where a change was observed.

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
