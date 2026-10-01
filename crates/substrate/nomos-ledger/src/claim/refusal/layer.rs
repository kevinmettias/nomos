//! Which of two owners a [`crate::ClaimRefusal`] is a fact about.

/// Which of two owners a [`crate::ClaimRefusal`] is a fact about.
///
/// `ARC-HARNESS-001`'s sentence names the two owners this splits: "the scheduler decides what
/// should run, coordination decides whether it can run now". A refusal answers one of those
/// questions, and this type is what lets a caller above the ledger ask *which* one it answered
/// without reading [`crate::ClaimRefusal::Describe`]'s prose or reconstructing the CLI's own
/// match arm — `OD-LEDGER-022` is the record of why that reconstruction was the defect.
///
/// The one-way rule this exists to make askable: coordination may withhold work the plan
/// calls ready, and coordination alone can never make ready what the plan calls not ready.
/// Nothing in this type enforces the rule — [`crate::ClaimRefusal::Layer`] only classifies what
/// a refusal already is — but the classification is what lets the rule be tested at all, over
/// the whole set of refusals rather than the two variants a caller happens to remember.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer
{
    /// A fact about the plan: whether this item can be worked at all, by anybody, regardless
    /// of when they ask. Stays true until the plan itself changes — a dependency finishes, an
    /// item's state changes, an item is authored.
    Readiness,
    /// A fact about this moment: the plan says the work is ready, and coordination — a lease,
    /// a lock, an unprovable independence, a store it could not reach — stands between a
    /// caller and starting it right now. A later attempt, against the same plan, can find it
    /// already resolved.
    Dispatch,
}
