//! Across what environment a determinism guarantee holds.

use serde::{Deserialize, Serialize};

const SINGLE_RUN_LABEL: &str = "SingleRun";
const CROSS_RUN_LABEL: &str = "CrossRun";
const CROSS_PLATFORM_LABEL: &str = "CrossPlatform";
const CROSS_BINARY_LABEL: &str = "CrossBinary";

/// Scope of environment over which a determinism claim is valid.
///
/// Each step up costs real verification work, and the variants are ordered so that
/// "at least this strong" is a comparison rather than a lookup table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ReproducibilityScope
{
    /// Reproducible only within one process invocation. Verified by repeating the
    /// operation in the same process.
    SingleRun,
    /// Reproducible across separate runs of the same binary on the same machine.
    /// Verified by two independent processes — which is what catches hash-seed
    /// randomization, address-dependent iteration and racing initialization.
    CrossRun,
    /// Reproducible across every supported platform. Verified by capturing reference
    /// output on one platform and comparing from the others; the practical floor is
    /// Linux, macOS and Windows.
    CrossPlatform,
    /// Reproducible across compiler versions and optimization levels. The strongest and
    /// most expensive claim, and the one a stored baseline needs if it must remain
    /// comparable after the analyzer itself is rebuilt.
    CrossBinary,
}

impl ReproducibilityScope
{
    /// The variant's stable `PascalCase` name, for display and diagnostics.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::SingleRun => SINGLE_RUN_LABEL,
            Self::CrossRun => CROSS_RUN_LABEL,
            Self::CrossPlatform => CROSS_PLATFORM_LABEL,
            Self::CrossBinary => CROSS_BINARY_LABEL,
        };
    }

    /// Whether verifying this scope requires more than one process.
    ///
    /// [`ReproducibilityScope::SingleRun`] has nothing to compare across environments;
    /// everything above it does, and the verification owed grows with the scope.
    #[must_use]
    pub const fn Is_Cross_Environment_Verification_Required(self) -> bool
    {
        return !matches!(self, Self::SingleRun);
    }
}

impl core::fmt::Display for ReproducibilityScope
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
    fn Test_Scope_Should_Order_Narrowest_First()
    {
        assert!(ReproducibilityScope::SingleRun < ReproducibilityScope::CrossRun);
        assert!(ReproducibilityScope::CrossRun < ReproducibilityScope::CrossPlatform);
        assert!(ReproducibilityScope::CrossPlatform < ReproducibilityScope::CrossBinary);
    }

    #[test]
    fn Test_Is_Cross_Environment_Verification_Required_Should_Be_False_For_Single_Run_Only()
    {
        assert!(!ReproducibilityScope::SingleRun.Is_Cross_Environment_Verification_Required());
        assert!(ReproducibilityScope::CrossRun.Is_Cross_Environment_Verification_Required());
        assert!(ReproducibilityScope::CrossPlatform.Is_Cross_Environment_Verification_Required());
        assert!(ReproducibilityScope::CrossBinary.Is_Cross_Environment_Verification_Required());
    }

    /// `Label` is the `Display` form every variant renders through.
    #[test]
    fn Test_Label_Should_Spell_Every_Variant_Distinctly()
    {
        let all = [
            ReproducibilityScope::SingleRun,
            ReproducibilityScope::CrossRun,
            ReproducibilityScope::CrossPlatform,
            ReproducibilityScope::CrossBinary,
        ];

        let mut labels: Vec<&str> = all.iter().map(|scope| return scope.Label()).collect();
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), count, "two scopes share a wire spelling");
    }
}
