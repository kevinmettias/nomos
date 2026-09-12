

use serde::{Deserialize, Serialize};

const BLOCKING_LABEL: &str = "Blocking";
const ADVISORY_LABEL: &str = "Advisory";
const UNREACHABLE_LABEL: &str = "Unreachable";
const REVIEW_LABEL: &str = "Review";

/// What a rule's declared enforcers can actually do to a build.
///
/// This value is **derived**, never authored. It is computed by walking the real gate
/// wiring — from the gate roots outward — and asking what would happen if the rule were
/// violated. A rule may state its expectation, and a mismatch between the expectation
/// and this value is itself a finding.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GateCategory
{
    /// No mechanical enforcer is claimed. Honest, and the correct declaration for
    /// anything a machine cannot judge.
    Review,
    /// An enforcer is named, and nothing invokes it. The rule is declared enforced and
    /// never runs.
    Unreachable,
    /// A gate invokes the enforcer and discards its result. It reports; it cannot fail
    /// the build.
    Advisory,
    /// A gate invokes the enforcer and honors its result.
    Blocking,
}

impl GateCategory
{
    /// The variant's stable `PascalCase` name.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Blocking => BLOCKING_LABEL,
            Self::Advisory => ADVISORY_LABEL,
            Self::Unreachable => UNREACHABLE_LABEL,
            Self::Review => REVIEW_LABEL,
        };
    }

    /// Whether a violation of a rule in this category can fail a build.
    #[must_use]
    pub const fn Can_Fail_A_Build(self) -> bool
    {
        return matches!(self, Self::Blocking);
    }

    /// The category of a rule whose enforcers land in several categories.
    ///
    /// The strongest wins: running advisory in one place does not undo being enforced
    /// in another. Ordering the enum weakest-first is what makes this a `max` rather
    /// than a table nobody maintains.
    #[must_use]
    pub fn Strongest_Of(self, other: Self) -> Self
    {
        return core::cmp::max(self, other);
    }
}

impl core::fmt::Display for GateCategory
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
    fn Test_Can_Fail_A_Build_Should_Be_True_For_Blocking_Only()
    {
        assert!(GateCategory::Blocking.Can_Fail_A_Build());
        assert!(!GateCategory::Advisory.Can_Fail_A_Build());
        assert!(!GateCategory::Unreachable.Can_Fail_A_Build());
        assert!(!GateCategory::Review.Can_Fail_A_Build());
    }

    /// Running advisory in one place does not undo being enforced in another.
    #[test]
    fn Test_Strongest_Of_Should_Pick_The_Stronger_Category()
    {
        assert_eq!(
            GateCategory::Advisory.Strongest_Of(GateCategory::Blocking),
            GateCategory::Blocking
        );
        assert_eq!(
            GateCategory::Unreachable.Strongest_Of(GateCategory::Advisory),
            GateCategory::Advisory
        );
        assert_eq!(
            GateCategory::Review.Strongest_Of(GateCategory::Unreachable),
            GateCategory::Unreachable
        );
    }

    /// `Label` is the `Display` form every variant renders through.
    #[test]
    fn Test_Label_Should_Spell_Every_Variant_Distinctly()
    {
        let all = [
            GateCategory::Review,
            GateCategory::Unreachable,
            GateCategory::Advisory,
            GateCategory::Blocking,
        ];

        let mut labels: Vec<&str> = all.iter().map(|category| return category.Label()).collect();
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), count, "two gate categories share a wire spelling");
    }
}
