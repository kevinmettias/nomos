

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
    pub const fn Is_Write(self) -> bool
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
    use alloc::vec::Vec;
    use super::*;

    #[test]
    fn Test_Is_Write_Should_Be_True_For_Only_Apply_And_Rollback()
    {
        assert!(MutationClass::Apply.Is_Write());
        assert!(MutationClass::Rollback.Is_Write());
        assert!(!MutationClass::Read.Is_Write());
        assert!(!MutationClass::Preview.Is_Write());
        assert!(!MutationClass::Validate.Is_Write());
    }

    /// A preview that needed write authority would push callers to skip previewing,
    /// which defeats the sequence the class exists to enforce.
    #[test]
    fn Test_Required_Authority_Should_Not_Demand_Mutate_For_Preview_Or_Validate()
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
        for class in Write_Classes()
        {
            assert!(class.Required_Authority().Is_Explicit_Grant_Required());
        }
    }

    /// The two classes [`MutationClass::Is_Write`] reports `true` for.
    fn Write_Classes() -> [MutationClass; 2]
    {
        return [MutationClass::Apply, MutationClass::Rollback];
    }

    /// `Label` is the `Display` form every variant renders through.
    #[test]
    fn Test_Label_Should_Spell_Every_Variant_Distinctly()
    {
        let all = [
            MutationClass::Read,
            MutationClass::Preview,
            MutationClass::Validate,
            MutationClass::Apply,
            MutationClass::Rollback,
        ];

        let mut labels: Vec<&str> = all.iter().map(|class| return class.Label()).collect();
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), count, "two mutation classes share a wire spelling");
    }
}
