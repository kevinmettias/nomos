//! [`BranchState`], whether a build compiles a branch.

/// Whether the named build compiles one branch's lines.
///
/// Three states, not a `bool`: a branch whose condition could not be evaluated is neither
/// compiled nor skipped as far as this answer knows, and reading it as either would be a guess
/// presented as a fact.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BranchState
{
    /// The compiler reads this branch's lines.
    Compiled,
    /// The compiler skips this branch's lines: its condition is false, an earlier branch of its
    /// chain was taken, or it sits inside a branch that is itself skipped.
    Skipped,
    /// This branch's condition, or one it depends on, is not a well-formed preprocessor
    /// expression, which the compiler reports as an error rather than resolving either way.
    Unevaluated,
}

impl BranchState
{
    /// This state's canonical label, as the payload spells it.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Compiled => "compiled",
            Self::Skipped => "skipped",
            Self::Unevaluated => "unevaluated",
        };
    }

    /// The state [`Self::Label`] spelled `label`, or `None` for any other string.
    #[must_use]
    pub fn From_Label(label: &str) -> Option<Self>
    {
        return match label
        {
            "compiled" => Some(Self::Compiled),
            "skipped" => Some(Self::Skipped),
            "unevaluated" => Some(Self::Unevaluated),
            _ => None,
        };
    }
}
