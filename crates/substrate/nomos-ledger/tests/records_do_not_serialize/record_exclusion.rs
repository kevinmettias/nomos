//! The acceptance criterion: a record excludes nobody but its own writer.

use crate::board::{
    Claimed, Project_Onto_Records, Saved, SavedBoard, Unclaimed_Copy, Writer_Ids,
};
use nomos_ledger::Holder;

/// What `P10-RECORD-LOCK` actually bought, stated so that it can hold.
///
/// The original acceptance test asserted that two record writers on the board could be
/// claimed at once, over their whole territories. That was true on the day it was written
/// and stopped being true on 2026-08-09 without any code changing: every open item that
/// writes a record must also edit `governing.rs` and the surface snapshots, so they all
/// share two paths and no pair is independent. The property was a fact about who had
/// authored what.
///
/// This is the property the mechanism can keep. Every record writer is reduced to the
/// records it reserves, and the pair is claimed through the real ledger on that
/// projection. If it passes, the records contribute no exclusion — which is the whole of
/// what reserving `docs/records/<ID>` instead of `docs/records` was for. What the items
/// *also* share is measured separately, by the two tests in [`super::serializers`].
///
/// `OD-LEDGER-007` records the restatement and why the stronger property was given up
/// rather than manufactured.
///
/// It no longer refuses a board with fewer than two writers. "Every record writer can be
/// claimed at once" is true of a board with one and true of a board with none, and
/// `OD-LEDGER-032` measured what demanding otherwise cost: red on 26 of the last 30 commits,
/// for a reason no commit contained. The teeth are proved on a constructed subject instead
/// — `super::sharing_a_record::Test_Two_Items_Writing_One_Record_Should_Still_Be_Refused`
/// builds two writers on one record and requires the refusal, so this passing over an empty
/// board is not the same as nothing being checked anywhere.
#[test]
fn Test_A_Record_Should_Exclude_Nobody_But_Its_Own_Writer()
{
    let mut document = Unclaimed_Copy();
    let writers = Writer_Ids(&document);
    Project_Onto_Records(&mut document, &writers);

    // Every one of them, not a pair. A pair could be independent by accident; all of them
    // being claimable at once is the property, and it is the one that survives an item being
    // added to the board tomorrow.
    let SavedBoard {
        scratch: _scratch,
        mut ledger,
    } = Saved("records-only", &document);
    for (ordinal, writer) in writers.iter().enumerate()
    {
        let agent = format!("agent-{ordinal}");
        let blame = format!(
            "{writer} was refused on its record alone, so two records still exclude each other"
        );
        Claimed(&mut ledger, writer, Holder::from(&agent), &blame);
    }
    ledger
        .Validate_Current()
        .expect("independent claims are a valid ledger");
}
