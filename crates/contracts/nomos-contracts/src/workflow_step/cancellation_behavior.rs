//! What happens when cancellation is requested while a step is executing.

use serde::{Deserialize, Serialize};

const UNCANCELLABLE_LABEL: &str = "Uncancellable";
const CANCELLABLE_BEFORE_SIDE_EFFECTS_LABEL: &str = "CancellableBeforeSideEffects";
const CANCELLABLE_WITH_COMPENSATION_LABEL: &str = "CancellableWithCompensation";

/// What happens when cancellation is requested while a step is executing.
///
/// `WF-012`'s own text asks execution to "report which side effects completed and
/// what remains reversible" -- that reporting is the engine's job, over a real run
/// this workspace does not have yet. What a step's own contract can state today is
/// the coarser, static question this enum answers: is a mid-flight cancellation even
/// a state this step's declaration permits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CancellationBehavior
{
    /// Cancellation is not honored once execution starts.
    Uncancellable,
    /// Cancellation is only honored before the first side effect.
    CancellableBeforeSideEffects,
    /// Cancellation is honored at any point, because a [`super::Compensation`] can
    /// undo whatever ran before the cancellation was observed.
    CancellableWithCompensation,
}

impl CancellationBehavior
{
    /// The variant's stable `PascalCase` name.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Uncancellable => UNCANCELLABLE_LABEL,
            Self::CancellableBeforeSideEffects => CANCELLABLE_BEFORE_SIDE_EFFECTS_LABEL,
            Self::CancellableWithCompensation => CANCELLABLE_WITH_COMPENSATION_LABEL,
        };
    }

    /// Whether cancellation is ever honored, at any point.
    #[must_use]
    pub const fn Is_Cancellable(self) -> bool
    {
        return !matches!(self, Self::Uncancellable);
    }
}

impl core::fmt::Display for CancellationBehavior
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
    fn Test_Uncancellable_Should_Not_Be_Cancellable()
    {
        assert!(!CancellationBehavior::Uncancellable.Is_Cancellable());
    }

    #[test]
    fn Test_Is_Cancellable_Should_Be_True_For_Both_Cancellable_Variants()
    {
        assert!(CancellationBehavior::CancellableBeforeSideEffects.Is_Cancellable());
        assert!(CancellationBehavior::CancellableWithCompensation.Is_Cancellable());
    }

    /// `Label` is the `Display` form every variant renders through.
    #[test]
    fn Test_Label_Should_Spell_Every_Variant_Distinctly()
    {
        let all = [
            CancellationBehavior::Uncancellable,
            CancellationBehavior::CancellableBeforeSideEffects,
            CancellationBehavior::CancellableWithCompensation,
        ];

        let mut labels: Vec<&str> = all.iter().map(|behavior| return behavior.Label()).collect();
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), count, "two cancellation behaviors share a wire spelling");
    }
}
