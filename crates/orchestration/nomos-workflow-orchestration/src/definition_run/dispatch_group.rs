//! One group of nodes a definition run visited together, under the caller's bound.

/// One group of nodes a definition run visited together.
///
/// A definition's nodes fall into dependency *waves*: wave zero is every node that depends
/// on nothing, and a node's wave is one past the highest wave it depends on. Every member
/// of one wave is independent of every other member, which is the condition
/// `OD-ROADMAP-006` decision 2's `WF-010` clause names -- steps that declare no dependence
/// on one another.
///
/// A wave is then cut into groups of at most the bound the caller stated, and this is one
/// of those cuts. The bound is honored rather than advisory in the only sense a report can
/// be checked on: no group a run produces ever holds more nodes than the bound, and a wave
/// wider than the bound arrives as several groups rather than one oversized one.
///
/// **What this is not.** This crate spawns no thread and composes no executor, so the
/// members of a group are dispatched one after another rather than at the same time.
/// Nothing here claims otherwise. What the grouping buys is the two things a bound is
/// actually for -- an explicit statement of how much independent work may be in flight at
/// once, and a report a caller can check that statement against -- without this crate
/// taking a position on a concurrency runtime it does not have. A real executor consuming
/// these groups would find the independence already established and the bound already
/// applied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DispatchGroup
{
    /// The dependency wave this group was cut from.
    ///
    /// Two groups can share a wave, which is exactly what a wave wider than the bound
    /// produces, and that is the case a caller checking the bound wants to see.
    pub wave: usize,
    /// The nodes in this group, by name, in the definition's declared order.
    pub nodes: Vec<String>,
}
