//! Whether one portion of a work result was substantiated, and if not, why.

use crate::UnsubstantiatedReason;

/// Whether the producing executor could substantiate one portion of a
/// [`WorkResult`](crate::WorkResult).
///
/// Two variants rather than a boolean and rather than a presence flag. A boolean says *that*
/// a producer could not ground a portion and never *why*, so a caller still cannot choose a
/// reply — and it is individually unverifiable, since nothing distinguishes a producer that
/// honestly declares a limit from one that sets the field and moves on. A presence flag is
/// the same absence with a different spelling, and the question here is not whether a value
/// is present but what an absent one means.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PortionSubstantiation
{
    /// The producer grounded this portion, so its value describes the task itself.
    ///
    /// An empty value under this variant means the task produced none.
    Substantiated,
    /// The producer could not ground this portion, so its value says nothing about the task,
    /// whatever it holds.
    Unsubstantiated(UnsubstantiatedReason),
}

impl PortionSubstantiation
{
    /// Whether the value beside this declaration describes the task.
    ///
    /// This is the rule a consumer applies, so it is asked rather than re-derived: a
    /// substantiated and empty portion means the task produced none, and an unsubstantiated
    /// portion means the value carries no claim either way.
    #[must_use]
    pub fn Is_Substantiated(&self) -> bool
    {
        return match self
        {
            Self::Substantiated => true,
            Self::Unsubstantiated(_) => false,
        };
    }

    /// A one-line description of what this declaration says about its portion.
    #[must_use]
    pub fn Describe(&self) -> String
    {
        return match self
        {
            Self::Substantiated =>
            {
                "the producing dispatch grounded this portion, so an empty value means the \
                 task produced none"
                    .to_owned()
            }
            Self::Unsubstantiated(reason) => reason.Describe(),
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_A_Substantiated_Portion_Should_Answer_That_It_Is()
    {
        assert!(PortionSubstantiation::Substantiated.Is_Substantiated());
    }

    #[test]
    fn Test_An_Unsubstantiated_Portion_Should_Answer_That_It_Is_Not()
    {
        let unsubstantiated =
            PortionSubstantiation::Unsubstantiated(UnsubstantiatedReason::ProducerCannotGround);

        assert!(!unsubstantiated.Is_Substantiated());
    }

    /// The description of an unsubstantiated portion is its reason's own, so a caller that
    /// reads one learns why rather than only that the portion is absent.
    #[test]
    fn Test_An_Unsubstantiated_Portion_Should_Describe_Its_Reason()
    {
        let reason = UnsubstantiatedReason::ProducerCannotGround;
        let unsubstantiated = PortionSubstantiation::Unsubstantiated(reason.clone());

        assert_eq!(unsubstantiated.Describe(), reason.Describe());
    }
}
