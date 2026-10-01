//! How much independent work a definition run may have in one group, and in what order it
//! visits one.

use std::num::NonZeroUsize;

use super::GroupVisitOrder;

/// The bound a caller states on how many independent nodes one dispatch group may hold.
///
/// `NonZeroUsize` because a bound of zero is not a narrower bound, it is a run that can
/// never visit anything -- the same refusal-by-type `nomos_contracts::RetryPolicy::Retry`
/// already makes of `max_attempts`. Stated by the caller rather than derived from the
/// machine, for the reason `nomos_platform::Clock` exists at all: a crate that reads an
/// ambient answer cannot be asked a reproducible question, and a bound read from the core
/// count would make a run's own report depend on which machine took it.
///
/// [`Self::visit`] is not a second bound. It says which order this run visits a group's
/// members in, and it exists so the claim that the order does not matter has something to
/// be checked against; [`GroupVisitOrder`]'s own doc carries that argument.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Parallelism
{
    /// The most nodes one dispatch group may hold.
    pub bound: NonZeroUsize,
    /// The order this run visits a group's members in.
    pub visit: GroupVisitOrder,
}

impl Parallelism
{
    /// `bound` independent nodes per group, visited in the order the definition declared
    /// them.
    #[must_use]
    pub const fn Of(bound: NonZeroUsize) -> Self
    {
        return Self { bound, visit: GroupVisitOrder::Declared };
    }
}
