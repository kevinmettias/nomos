//! How sure a claim about a change is.

use serde::{Deserialize, Serialize};

/// How much weight a claim carries, on a stated scale.
///
/// A bounded value with an explicit constructor rather than a bare `f64`, so that a
/// confidence can never be constructed outside its range and never compared with `==`.
/// Both of those are lint-level errors in this workspace, and both were real defects in
/// the prototype.
///
/// # Why this is still an `f64`, and what would decide otherwise
///
/// This is the workspace's one bounded-ratio type and the obvious candidate for a rational
/// or a fixed-point integer instead. That question is deliberately left open, and the reason
/// is not that nobody has looked: it is that the one consumer this workspace has actually
/// named does not exercise the representation at all.
///
/// [`crate::IdentityTransition`]'s own doc records that consumer — `SUP-*`'s revalidation
/// triggers, via `OD-GATE-015`. A trigger is an *event*: "an identity transition occurred"
/// is a yes or a no, and a suppression that must be revalidated is revalidated whatever
/// number graded the transition. Nothing there compares two confidences, aggregates them, or
/// round-trips one through a serialized form and expects the same value back — and those are
/// the uses a float would be wrong for.
///
/// So the representation is answerable only against a consumer that needs a *degree* rather
/// than an event. Picking one now would be choosing a shape for a use nobody has stated,
/// which is the move this repository declines elsewhere for the same reason.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Confidence(f64);

impl Confidence
{
    /// Complete confidence.
    pub const CERTAIN: Self = Self(1.0);

    /// No confidence at all.
    pub const NONE: Self = Self(0.0);

    /// Constructs a confidence, clamping to the unit interval.
    ///
    /// Clamps rather than rejecting: a provider that computed 1.0000001 through
    /// floating-point accumulation has not made an error worth failing a run over, and
    /// the alternative is every caller writing its own clamp slightly differently.
    #[must_use]
    pub fn Of(value: f64) -> Self
    {
        if value.is_nan()
        {
            return Self::NONE;
        }
        let clamped = value.clamp(0.0, 1.0);

        return Self(clamped);
    }

    /// The value, in the unit interval.
    #[must_use]
    pub const fn Value(self) -> f64
    {
        return self.0;
    }

    /// Whether this confidence is at least `threshold`.
    #[must_use]
    pub fn Is_At_Least(self, threshold: Self) -> bool
    {
        return self.0 >= threshold.0;
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    const ARBITRARY_POSITIVE_THRESHOLD: f64 = 0.1;

    const FLOAT_TOLERANCE: f64 = 1e-9;

    #[test]
    fn Test_Of_Should_Clamp_To_The_Unit_Interval()
    {
        assert!(Confidence::Of(1.5).Value() <= 1.0);
        assert!(Confidence::Of(-0.5).Value() >= 0.0);
    }

    /// A NaN confidence compares false against everything, including any threshold, so
    /// it would silently behave as "never good enough" in one place and as "never
    /// rejected" in another depending on how the comparison was written.
    #[test]
    fn Test_Is_At_Least_Should_Reject_A_Nan_Confidence_Against_Any_Threshold()
    {
        assert!(Confidence::Of(f64::NAN).Value() <= 0.0);
        assert!(!Confidence::Of(f64::NAN).Is_At_Least(Confidence::Of(ARBITRARY_POSITIVE_THRESHOLD)));
    }

    /// `Value` is read out everywhere else as a side effect of asserting on `Of` or
    /// `Is_At_Least`; this is the one test for which it is the actual subject.
    #[test]
    fn Test_Value_Should_Return_The_Number_A_Confidence_Was_Constructed_With()
    {
        assert!((Confidence::Of(0.3).Value() - 0.3).abs() < FLOAT_TOLERANCE);
    }
}
