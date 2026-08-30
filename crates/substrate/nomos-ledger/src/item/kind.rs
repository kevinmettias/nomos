//! What sort of work an item is.

use serde::Deserialize;
use serde::Serialize;

/// What kind of work an item names.
///
/// A closed set, for the reason [`crate::ItemState`]'s is: a kind that exists only in one
/// tool's imagination is a kind no query can filter on, and serde refuses an unrecognized
/// variant on read the same way [`crate::LedgerItem`]'s `#[serde(deny_unknown_fields)]`
/// refuses an unrecognized key — a build that predates a sixth kind cannot silently accept
/// it and cannot silently drop it either; it refuses the document. `OD-LEDGER-024` is the
/// record this closes and names why these five and not a free-form tag list.
///
/// [`Self::Capability`] is not one of the four kinds that record names in its own
/// prose — validation, decision, correction, cleanup — because this ledger's earliest
/// batches (`P1` through `P8`, per `OD-LEDGER-002`'s own table) built crates that did not
/// exist yet rather than fixing, checking or simplifying ones that did. A defect taxonomy
/// with nothing for the thing it corrects against has nowhere to put the corpus's own
/// first half.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind
{
    /// Builds a capability that did not exist: a crate, a command, a data shape. Nothing
    /// was wrong beforehand; there was simply nothing there yet.
    Capability,
    /// Settles an architectural question and records the answer — a seam, a boundary, an
    /// ownership rule, a policy — whether or not it also touches code.
    Decision,
    /// Adds or repairs a check: a test, a gate step, a snapshot, an assertion that a
    /// property already meant to hold is actually enforced.
    Validation,
    /// Fixes something that is wrong: a defect, a gap, a stale statement, a divergence
    /// between what a record claims and what the tree does.
    Correction,
    /// Reshapes existing, working code or prose without changing what it does: a rename,
    /// a split, a decomposition, a named constant standing in for a bare literal.
    Cleanup,
}
