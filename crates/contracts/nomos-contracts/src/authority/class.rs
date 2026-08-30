use serde::{Deserialize, Serialize};

const READ_LABEL: &str = "Read";
const PROPOSE_LABEL: &str = "Propose";
const PREVIEW_LABEL: &str = "Preview";
const MUTATE_LABEL: &str = "Mutate";
const EXECUTE_LABEL: &str = "Execute";
const APPROVE_LABEL: &str = "Approve";
const PUBLISH_LABEL: &str = "Publish";

/// The authority an actor must hold to invoke an operation.
///
/// Separated from [`MutationClass`](super::MutationClass) because they answer different
/// questions and an actor's grants are not a function of what an operation touches.
/// Approving someone else's change touches nothing and requires more authority than
/// making one.
///
/// Not ordered. These are not a ladder — an agent may legitimately hold `Mutate` and
/// not `Approve`, which is the whole point of separating duties — so there is
/// deliberately no `PartialOrd` to invite `>=` comparisons that would grant approval to
/// anything allowed to write.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Class
{
    /// Observe state.
    Read,
    /// Suggest a change without staging it.
    Propose,
    /// Compute and inspect the effect of a change without applying it.
    Preview,
    /// Change repository or workspace state.
    Mutate,
    /// Run something with side effects outside the workspace.
    Execute,
    /// Authorize someone else's change.
    Approve,
    /// Make something externally visible and durable.
    Publish,
}

impl Class
{
    /// The variant's stable `PascalCase` name.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Read => READ_LABEL,
            Self::Propose => PROPOSE_LABEL,
            Self::Preview => PREVIEW_LABEL,
            Self::Mutate => MUTATE_LABEL,
            Self::Execute => EXECUTE_LABEL,
            Self::Approve => APPROVE_LABEL,
            Self::Publish => PUBLISH_LABEL,
        };
    }

    /// Whether holding this authority requires an explicit, auditable grant.
    ///
    /// Reading and proposing are ordinary. Everything that changes the world, runs
    /// something, blesses someone else's work or makes a thing durable is not.
    #[must_use]
    pub const fn Is_Explicit_Grant_Required(self) -> bool
    {
        return matches!(
            self,
            Self::Mutate | Self::Execute | Self::Approve | Self::Publish
        );
    }
}

impl core::fmt::Display for Class
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

    #[test]
    fn Test_Reading_Should_Not_Require_An_Explicit_Grant()
    {
        assert!(!Class::Read.Is_Explicit_Grant_Required());
        assert!(!Class::Propose.Is_Explicit_Grant_Required());
        assert!(!Class::Preview.Is_Explicit_Grant_Required());
    }
}
