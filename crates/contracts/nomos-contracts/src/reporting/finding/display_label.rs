//! The compact form an applicability state is rendered as, and nothing more.

use serde::{Deserialize, Serialize};

const NATIVE_LABEL: &str = "Native";
const FALLBACK_LABEL: &str = "Fallback";
const PARTIAL_LABEL: &str = "Partial";
const DISPLAY_NOT_APPLICABLE_LABEL: &str = "N/A";
const UNAVAILABLE_LABEL: &str = "Unavailable";
const FAILED_LABEL: &str = "Failed";
const AGENT_LABEL: &str = "Agent";

/// The compact presentation form of an [`Applicability`](crate::Applicability), for
/// matrices and summaries.
///
/// This is a projection, never a source. It intentionally loses information — several
/// applicability states collapse to `Unavailable` — which is exactly why a gate
/// decision must never be taken from a `DisplayLabel`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DisplayLabel
{
    /// Evaluated at the requested guarantee.
    Native,
    /// Evaluated, by a weaker provider than requested.
    Fallback,
    /// Evaluated over part of the subject.
    Partial,
    /// The rule does not bind this subject.
    NotApplicable,
    /// Something needed was absent. Not a pass.
    Unavailable,
    /// Something needed was present and broke. Not a pass.
    Failed,
    /// A model is needed to judge this. Available work, not a pass and not a gap.
    ///
    /// This one does not collapse. Folding it into `Unavailable` would tell a reader to
    /// install something that does not exist, which is the mis-filing
    /// [`Applicability::AgentRequired`](crate::Applicability::AgentRequired) was added to
    /// end — reintroducing it here would undo the correction at the only layer most
    /// readers ever see.
    AgentRequired,
}

impl DisplayLabel
{
    /// The variant's stable display string.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Native => NATIVE_LABEL,
            Self::Fallback => FALLBACK_LABEL,
            Self::Partial => PARTIAL_LABEL,
            Self::NotApplicable => DISPLAY_NOT_APPLICABLE_LABEL,
            Self::Unavailable => UNAVAILABLE_LABEL,
            Self::Failed => FAILED_LABEL,
            Self::AgentRequired => AGENT_LABEL,
        };
    }
}

impl core::fmt::Display for DisplayLabel
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

    /// `Label` is the `Display` form every variant renders through. `NotApplicable`'s
    /// `N/A` is deliberately not distinct in *shape* from a longer word, only in
    /// spelling, so this checks the spelling rather than any format convention.
    #[test]
    fn Test_Label_Should_Spell_Every_Variant_Distinctly()
    {
        let all = [
            DisplayLabel::Native,
            DisplayLabel::Fallback,
            DisplayLabel::Partial,
            DisplayLabel::NotApplicable,
            DisplayLabel::Unavailable,
            DisplayLabel::Failed,
            DisplayLabel::AgentRequired,
        ];

        let mut labels: Vec<&str> = all.iter().map(|label| return label.Label()).collect();
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), count, "two display labels share a spelling");
    }
}
