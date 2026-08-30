//! The counter that keeps a stale result readable as history without ever making it current.

use serde::{Deserialize, Serialize};

/// A monotone counter marking one analysis state of a workspace.
///
/// Generations are the mechanism by which stale results stay readable as history
/// without ever being publishable as current. A result produced against generation 42
/// is not wrong when the workspace reaches 43 — it is evidence about 42, and saying so
/// is different from either discarding it or presenting it as fresh.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GenerationId(u64);

impl GenerationId
{
    /// The generation a freshly discovered workspace starts at.
    pub const INITIAL: Self = Self(0);

    /// Wraps a raw counter value.
    #[must_use]
    pub const fn From_Raw(value: u64) -> Self
    {
        return Self(value);
    }

    /// The raw counter value.
    #[must_use]
    pub const fn Raw(self) -> u64
    {
        return self.0;
    }

    /// The generation following this one.
    ///
    /// Saturating rather than wrapping. A wrapped generation counter would make an
    /// ancient result compare as current, which is precisely the failure the type
    /// exists to prevent; saturation stalls loudly instead, and `u64::MAX` generations
    /// is not a workspace anyone will reach.
    #[must_use]
    pub const fn Next(self) -> Self
    {
        return Self(self.0.saturating_add(1));
    }
}

impl core::fmt::Display for GenerationId
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "gen{}", self.0);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_From_Raw_Should_Wrap_The_Given_Counter_Value()
    {
        assert_eq!(GenerationId::From_Raw(7).Raw(), 7);
    }

    #[test]
    fn Test_Raw_Should_Return_The_Wrapped_Counter_Value()
    {
        assert_eq!(GenerationId::From_Raw(7).Raw(), 7);
        assert_eq!(GenerationId::INITIAL.Raw(), 0);
    }

    #[test]
    fn Test_Next_Should_Advance_And_Never_Wrap()
    {
        assert_eq!(GenerationId::INITIAL.Next().Raw(), 1);
        assert_eq!(GenerationId::From_Raw(u64::MAX).Next().Raw(), u64::MAX);
    }
}
