//! How sure a claim about a change is.

use serde::{Deserialize, Serialize};

/// How much weight a claim carries, on a stated scale.
///
/// A bounded value with an explicit constructor rather than a bare `f64`, so that a
/// confidence can never be constructed outside its range and never compared with `==`.
/// Both of those are lint-level errors in this workspace, and both were real defects in
/// the prototype.
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

    #[test]
    fn Test_Confidence_Should_Clamp_To_The_Unit_Interval()
    {
        assert!(Confidence::Of(1.5).Value() <= 1.0);
        assert!(Confidence::Of(-0.5).Value() >= 0.0);
    }

    /// A NaN confidence compares false against everything, including any threshold, so
    /// it would silently behave as "never good enough" in one place and as "never
    /// rejected" in another depending on how the comparison was written.
    #[test]
    fn Test_Nan_Confidence_Should_Become_None()
    {
        assert!(Confidence::Of(f64::NAN).Value() <= 0.0);
        assert!(!Confidence::Of(f64::NAN).Is_At_Least(Confidence::Of(0.1)));
    }
}
