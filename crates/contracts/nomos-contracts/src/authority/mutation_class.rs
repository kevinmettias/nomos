use serde::{Deserialize, Serialize};

use super::AuthorityClass;

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
}
