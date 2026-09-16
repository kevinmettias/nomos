//! How long a step may run before it is no longer waited on.

use serde::{Deserialize, Serialize};
use core::num::NonZeroU32;

/// How long a step may run before it is no longer waited on.
///
/// Seconds, deliberately -- the same resolution `nomos_platform::Timestamp`'s own
/// module doc already commits to elsewhere in this workspace: ordering finer than a
/// second is carried explicitly where it matters, not implied by a clock's precision.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Timeout
{
    /// No bound is declared.
    Unbounded,
    /// The step is no longer waited on after this many seconds.
    Seconds(NonZeroU32),
}

impl Timeout
{
    /// Whether this step declares any bound at all.
    #[must_use]
    pub const fn Is_Bounded(self) -> bool
    {
        return matches!(self, Self::Seconds(_));
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The declared second count this case builds a bounded timeout around.
    const DECLARED_SECONDS: u32 = 30;

    #[test]
    fn Test_Unbounded_Should_Not_Be_Bounded()
    {
        assert!(!Timeout::Unbounded.Is_Bounded());
    }

    #[test]
    fn Test_Is_Bounded_Should_Be_True_When_Seconds_Are_Declared()
    {
        assert!(Timeout::Seconds(NonZeroU32::new(DECLARED_SECONDS).expect("DECLARED_SECONDS is nonzero")).Is_Bounded());
    }
}
