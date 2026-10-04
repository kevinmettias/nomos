//! What an item waits on, for `work show`: the refusal a claim on it would be given when no line
//! above already explains it, and the items it was authored to depend on.
//!
//! # Why a refusal line and a dependency list, and not one of them
//!
//! They answer different questions. The refusal is the one [`Claim_Refusal`] returns, so it names
//! the first thing a claim would meet right now -- an unfinished dependency, a declined one, or an
//! overlap nobody can decide -- and it is the sentence `work audit` already prints for the same
//! item. The list is the item's own terms, every dependency `work add` recorded, each with what
//! `list` would call it now, so a reader sees the whole chain an item sits behind and not only its
//! head. Neither computes readiness: the first is the claim check's answer and the second is its
//! label, the division `super::report::blocking`'s module doc argues for the holder's case.

use super::listing::Listing_Label;
use nomos_ledger::{ClaimRefusal, Claim_Refusal, LedgerDocument, LedgerItem};
use nomos_platform::Timestamp;

/// The refusal a claim on `item` would be given right now, described, when it is one the lines
/// around the state word do not already explain.
///
/// `None` for an item nothing refuses; for one a claim of its own or a finished state refuses,
/// which the state word and the claim line already say; for one somebody else's claim holds,
/// whose blocker lines `super::report::blocking::Blocked_Lines` print in full; and for a lapsed
/// one, whose claim line says it lapsed. Everything left is a plan fact or an undecidable overlap
/// -- `waiting`, `stranded`, `snagged` -- and until this, `show` printed the word and not the
/// reason.
pub(super) fn Refusal_Line(document: &LedgerDocument, item: &LedgerItem, now: Timestamp) -> Option<String>
{
    return match Claim_Refusal(document, &item.id, now)?
    {
        ClaimRefusal::NotClaimable { .. } | ClaimRefusal::HeldBy { .. } | ClaimRefusal::Lapsed { .. } => None,
        refusal => Some(refusal.Describe()),
    };
}

/// The items `item` depends on, counted and then listed in the order they were authored, each
/// with what `list` would call it now.
///
/// After the predicate, as the last of the item's terms. A dependency the board does not carry
/// is said to be absent rather than skipped: `validate` refuses such a document, but a reader
/// holding one needs the name more than anyone.
pub(super) fn Print_Dependencies(
    document: &LedgerDocument,
    item: &LedgerItem,
    now: Timestamp,
    output: &mut impl std::io::Write,
)
{
    let _ = writeln!(output);
    if item.depends_on.is_empty()
    {
        let _ = writeln!(output, "depends on: none");

        return;
    }

    let _ = writeln!(output, "depends on: {} item(s)", item.depends_on.len());
    for dependency in &item.depends_on
    {
        let label = document
            .items
            .iter()
            .find(|candidate| return &candidate.id == dependency)
            .map_or("not on the board", |found| return Listing_Label(document, found, now));
        let _ = writeln!(output, "  {dependency} {label}");
    }
}
