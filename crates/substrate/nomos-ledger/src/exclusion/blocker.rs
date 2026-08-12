//! What is standing between an item and being workable.

use serde::Deserialize;
use serde::Serialize;
use crate::item::ItemId;
/// Why an item cannot be worked on.
///
/// A typed reason rather than free prose, so a query can answer "what is blocked on a
/// decision?" without matching strings. The prototype stored this as a sentence, and
/// the result was that nobody could tell how much of the backlog was waiting on a
/// person versus waiting on a dependency.
///
/// # Six of `WORK-LEDGER-005`'s seven causes, and why the seventh is not here
///
/// This enum is derived from `WORK-LEDGER-005`, a normative accepted corpus requirement, and
/// declares six of the seven causes it names, in its order. The absent one is
/// `stale probe artifact`. `OD-LEDGER-017` decided it is **declined rather than missing**: the
/// seven are an inventory of what one earlier ledger's prose `blocked` field was observed to
/// contain, this build has no probe artifact for an item to wait on, and the requirement's own
/// list ends "or other typed cause" — which [`Blocker::Other`] is. The variant becomes owed the
/// day an item here waits on *somebody else* regenerating a derived artifact; a stale
/// projection is not that, because re-rendering it needs no second party.
///
/// The comparison is transcribed as `CORPUS_CAUSES` in this module's test module rather than
/// left in the record, so the next reader does not have to re-derive it against a corpus that
/// is not on their machine — `D-134`'s reason for putting a universe beside its mirror.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Blocker
{
    /// Waiting on other items.
    Dependency
    {
        /// The items that must finish first.
        items: Vec<ItemId>,
    },
    /// Waiting on a decision somebody has to make.
    Decision
    {
        /// What has to be decided.
        question: String,
    },
    /// The item's territory does not match what the work actually touches.
    TerritoryMismatch
    {
        /// What is wrong with it.
        detail: String,
    },
    /// Waiting on something outside this repository.
    ExternalResource
    {
        /// What is being waited for.
        resource: String,
    },
    /// Too large to claim as one unit.
    NeedsSplit,
    /// Something else, stated.
    Other
    {
        /// What.
        detail: String,
    },
}
