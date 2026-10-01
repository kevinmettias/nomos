//! What counts as "the same output" when two runs are compared.

use serde::{Deserialize, Serialize};

const BIT_IDENTICAL_LABEL: &str = "BitIdentical";
const BEHAVIORALLY_EQUIVALENT_LABEL: &str = "BehaviorallyEquivalent";
const NOT_APPLICABLE_LABEL: &str = "NotApplicable";

/// Definition of equivalence for traces of observable events.
///
/// This axis decides what a differential test is allowed to assert. Two strategies with
/// different declared trace values are compared **at the weaker tolerance**, because a
/// comparison is only as strong as the weaker of the two contracts it spans — asserting
/// bit-identity against a strategy that never promised it would fail honestly-written
/// code.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TraceEquivalence
{
    /// No trace claim is made. Valid only alongside
    /// [`super::DeterminismStrength::None`].
    NotApplicable,
    /// Events produce equivalent observable outcomes within a documented tolerance,
    /// which the contract must define rather than leave to the reader.
    BehaviorallyEquivalent,
    /// Observable events match exactly, byte for byte.
    BitIdentical,
}

impl TraceEquivalence
{
    /// The variant's stable `PascalCase` name, for display and diagnostics.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::BitIdentical => BIT_IDENTICAL_LABEL,
            Self::BehaviorallyEquivalent => BEHAVIORALLY_EQUIVALENT_LABEL,
            Self::NotApplicable => NOT_APPLICABLE_LABEL,
        };
    }

    /// The tolerance at which two strategies with these declarations may be compared.
    ///
    /// Always the weaker of the pair. This is a method rather than a convention because
    /// leaving it to each differential test to remember produces exactly one test that
    /// forgets, and that test then encodes an assertion the contract never made.
    #[must_use]
    pub const fn Weaker_Of(self, other: Self) -> Self
    {
        return if (self as u8) <= (other as u8) { self } else { other };
    }
}

impl core::fmt::Display for TraceEquivalence
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
    fn Test_Comparison_Should_Take_The_Weaker_Tolerance()
    {
        assert_eq!(
            TraceEquivalence::BitIdentical.Weaker_Of(TraceEquivalence::BehaviorallyEquivalent),
            TraceEquivalence::BehaviorallyEquivalent
        );
        assert_eq!(
            TraceEquivalence::BehaviorallyEquivalent.Weaker_Of(TraceEquivalence::BitIdentical),
            TraceEquivalence::BehaviorallyEquivalent
        );
        assert_eq!(
            TraceEquivalence::BitIdentical.Weaker_Of(TraceEquivalence::BitIdentical),
            TraceEquivalence::BitIdentical
        );
    }

    /// `NotApplicable` is the weakest thing there is; pairing it with anything means
    /// there is nothing to compare.
    #[test]
    fn Test_Not_Applicable_Should_Dominate_Any_Comparison()
    {
        assert_eq!(
            TraceEquivalence::BitIdentical.Weaker_Of(TraceEquivalence::NotApplicable),
            TraceEquivalence::NotApplicable
        );
        assert_eq!(
            TraceEquivalence::NotApplicable.Weaker_Of(TraceEquivalence::BehaviorallyEquivalent),
            TraceEquivalence::NotApplicable
        );
    }

    #[test]
    fn Test_Weaker_Of_Should_Be_Commutative()
    {
        for left in All_Trace_Equivalences()
        {
            for right in All_Trace_Equivalences()
            {
                assert_eq!(left.Weaker_Of(right), right.Weaker_Of(left));
            }
        }
    }

    /// `Label` is the `Display` form every variant renders through.
    #[test]
    fn Test_Label_Should_Spell_Every_Variant_Distinctly()
    {
        let mut labels: Vec<&str> = All_Trace_Equivalences().iter().map(|trace| return trace.Label()).collect();
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), count, "two trace equivalences share a wire spelling");
    }

    /// Every declared trace-equivalence variant, once.
    fn All_Trace_Equivalences() -> [TraceEquivalence; 3]
    {
        return [
            TraceEquivalence::NotApplicable,
            TraceEquivalence::BehaviorallyEquivalent,
            TraceEquivalence::BitIdentical,
        ];
    }
}
