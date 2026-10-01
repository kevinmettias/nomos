//! How much debt one [`super::BaselineDebt`] accepted.

/// How much debt one [`super::BaselineDebt`] accepted.
///
/// A state that names itself rather than an `Option<u32>` whose `None` a reader has to
/// interpret. The interpretation is load-bearing and counter-intuitive -- absence means
/// *unlimited*, not zero and not one -- so it is spelled, and every match over it has to say
/// which case it is handling.
///
/// # Why absence is unbounded rather than one
///
/// `OD-GATE-030` decides this and states the alternative it rejected. Every entry authored
/// before the quantity existed names none, and reading those as a single occurrence would
/// start blocking builds over debt a repository did adopt, with the gate claiming a number
/// nobody wrote. Reading them as unlimited keeps the meaning they were written under; what
/// makes that honest rather than a silent hole is that the state is *named*, so a run can
/// report the entry as unbounded instead of it being indistinguishable from a bounded one.
///
/// # What a count is not
///
/// It bounds capacity and establishes nothing about history. A scope that accepted five and
/// observes five is equally consistent with the same five persisting and with all five having
/// been fixed while five different violations appeared. `OD-GATE-030` says so at length, and
/// this type is deliberately not named for continuity, persistence or reintroduction so that
/// nobody reaches for it to answer one of those.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BaselineAllowance
{
    /// The entry names no count, so it tolerates however many occurrences its scope holds.
    ///
    /// The state every entry authored before `OD-GATE-030` v2 is in, and the one a run reports
    /// rather than passes over in silence.
    Unbounded,
    /// The entry accepted at most this many occurrences.
    ///
    /// Never zero: an entry accepting none tolerates nothing, which is what declining to write
    /// the entry already does, so the declared reader refuses it rather than storing a value
    /// whose only effect would be to block what it claims to permit.
    AtMost(u32),
}
