//! Whether a step's own effect can be undone, and by what.

use serde::{Deserialize, Serialize};

const NONE_LABEL: &str = "None";
const SELF_COMPENSATING_LABEL: &str = "SelfCompensating";
const EXTERNALLY_COMPENSATED_LABEL: &str = "ExternallyCompensated";

/// Whether a step's own effect can be undone, and by what.
///
/// Deliberately does not name *which* step provides external compensation. Wiring one
/// step's failure to another step's compensating run is a workflow **definition**
/// concern (`WF-003`'s composition, `WF-011`'s versioned graph), not this crate's --
/// a step's own contract only needs to say whether compensation exists, not where it
/// lives.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Compensation
{
    /// This step has no side effect to undo, or accepts a failure after it runs as
    /// final.
    None,
    /// This step can undo its own effect when re-invoked in a compensating mode -- no
    /// second step is needed.
    SelfCompensating,
    /// A separate compensating step exists, composed by whatever assembles the
    /// workflow this step belongs to.
    ExternallyCompensated,
}

impl Compensation
{
    /// The variant's stable `PascalCase` name.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::None => NONE_LABEL,
            Self::SelfCompensating => SELF_COMPENSATING_LABEL,
            Self::ExternallyCompensated => EXTERNALLY_COMPENSATED_LABEL,
        };
    }

    /// Whether a compensation of either shape exists.
    #[must_use]
    pub const fn Is_Present(self) -> bool
    {
        return !matches!(self, Self::None);
    }
}

impl core::fmt::Display for Compensation
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
    fn Test_None_Should_Not_Exist()
    {
        assert!(!Compensation::None.Is_Present());
    }

    #[test]
    fn Test_Is_Present_Should_Be_True_For_Both_Compensating_Variants()
    {
        assert!(Compensation::SelfCompensating.Is_Present());
        assert!(Compensation::ExternallyCompensated.Is_Present());
    }

    /// `Label` is the `Display` form every variant renders through.
    #[test]
    fn Test_Label_Should_Spell_Every_Variant_Distinctly()
    {
        let all = [
            Compensation::None,
            Compensation::SelfCompensating,
            Compensation::ExternallyCompensated,
        ];

        let mut labels: Vec<&str> = all.iter().map(|compensation| return compensation.Label()).collect();
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), count, "two compensations share a wire spelling");
    }
}
