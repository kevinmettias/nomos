//! What an actor is permitted to do, and what an operation does to the world.

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
/// Separated from [`MutationClass`] because they answer different questions and an
/// actor's grants are not a function of what an operation touches. Approving someone
/// else's change touches nothing and requires more authority than making one.
///
/// Not ordered. These are not a ladder — an agent may legitimately hold `Mutate` and
/// not `Approve`, which is the whole point of separating duties — so there is
/// deliberately no `PartialOrd` to invite `>=` comparisons that would grant approval to
/// anything allowed to write.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuthorityClass
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

impl AuthorityClass
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
    pub const fn Requires_Explicit_Grant(self) -> bool
    {
        return matches!(
            self,
            Self::Mutate | Self::Execute | Self::Approve | Self::Publish
        );
    }
}

impl core::fmt::Display for AuthorityClass
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}

const MUTATION_READ_LABEL: &str = "Read";
const MUTATION_PREVIEW_LABEL: &str = "Preview";
const MUTATION_VALIDATE_LABEL: &str = "Validate";
const MUTATION_APPLY_LABEL: &str = "Apply";
const MUTATION_ROLLBACK_LABEL: &str = "Rollback";

/// What an operation does to workspace state.
///
/// The sequence `Preview → Validate → Apply → Rollback` is the shape every mutating
/// operation takes. It exists so that no API can combine analysis and mutation in one
/// opaque call — a caller must be able to see what would happen, check it, and undo it,
/// and an operation that offers only `Apply` has removed all three.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MutationClass
{
    /// Changes nothing.
    Read,
    /// Computes the effect of a change without applying it.
    Preview,
    /// Checks a previewed change against its obligations.
    Validate,
    /// Commits a previously previewed and validated change.
    Apply,
    /// Reverses a previously applied change.
    Rollback,
}

impl MutationClass
{
    /// The variant's stable `PascalCase` name.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Read => MUTATION_READ_LABEL,
            Self::Preview => MUTATION_PREVIEW_LABEL,
            Self::Validate => MUTATION_VALIDATE_LABEL,
            Self::Apply => MUTATION_APPLY_LABEL,
            Self::Rollback => MUTATION_ROLLBACK_LABEL,
        };
    }

    /// Whether this operation changes workspace state.
    #[must_use]
    pub const fn Writes(self) -> bool
    {
        return matches!(self, Self::Apply | Self::Rollback);
    }

    /// The authority an operation of this class requires by default.
    #[must_use]
    pub const fn Required_Authority(self) -> AuthorityClass
    {
        return match self
        {
            Self::Read => AuthorityClass::Read,
            Self::Preview | Self::Validate => AuthorityClass::Preview,
            Self::Apply | Self::Rollback => AuthorityClass::Mutate,
        };
    }
}

impl core::fmt::Display for MutationClass
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
    fn Test_Only_Apply_And_Rollback_Should_Write()
    {
        assert!(MutationClass::Apply.Writes());
        assert!(MutationClass::Rollback.Writes());
        assert!(!MutationClass::Read.Writes());
        assert!(!MutationClass::Preview.Writes());
        assert!(!MutationClass::Validate.Writes());
    }

    /// A preview that needed write authority would push callers to skip previewing,
    /// which defeats the sequence the class exists to enforce.
    #[test]
    fn Test_Preview_Should_Not_Require_Mutate_Authority()
    {
        assert_eq!(
            MutationClass::Preview.Required_Authority(),
            AuthorityClass::Preview
        );
        assert_eq!(
            MutationClass::Validate.Required_Authority(),
            AuthorityClass::Preview
        );
    }

    #[test]
    fn Test_Writing_Operations_Should_Require_An_Explicit_Grant()
    {
        for class in [MutationClass::Apply, MutationClass::Rollback]
        {
            assert!(class.Required_Authority().Requires_Explicit_Grant());
        }
    }

    #[test]
    fn Test_Reading_Should_Not_Require_An_Explicit_Grant()
    {
        assert!(!AuthorityClass::Read.Requires_Explicit_Grant());
        assert!(!AuthorityClass::Propose.Requires_Explicit_Grant());
        assert!(!AuthorityClass::Preview.Requires_Explicit_Grant());
    }
}
