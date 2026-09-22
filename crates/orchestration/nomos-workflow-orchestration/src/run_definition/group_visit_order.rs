//! The order a run visits the members of one dispatch group in.

/// The order a definition run visits the members of one [`crate::DispatchGroup`] in.
///
/// A group's members are mutually independent by construction, so there is no order
/// between them that the definition states -- which means a run has to pick one, and which
/// one it picks must not be visible in what it reports. That is the determinism claim, and
/// this type is what makes it testable rather than asserted: the same definition run under
/// both orders must report the same node sequence, the same completed outcomes and the
/// same attempts, and a run that sorted its report by anything other than the declaration
/// would fail that comparison.
///
/// It stands in for the arrival order a real bounded-parallel executor would produce.
/// This crate spawns no thread, so without it there would be no second order to run
/// against and the determinism guarantee would be a claim with no falsifier -- a test that
/// passes over the ordered implementation and would have passed without the ordering.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupVisitOrder
{
    /// Visit a group's members in the order the definition declared them.
    Declared,
    /// Visit a group's members in the reverse of the order the definition declared them.
    Reversed,
}
