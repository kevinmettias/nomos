//! The negative control: sharing a record still excludes.

use crate::board::{Contest, Board_Contested_By_Two_Agents, RECORD_DIRECTORY, Two_Record_Writers, TwoWriters};
use nomos_ledger::{ClaimRefusal, Territory};

/// The control that keeps the repair from being a blanket exemption.
///
/// Take the two record writers the acceptance test claims concurrently on their records,
/// and point both at one record. If that still claims twice, the fix has not made record
/// reservations finer — it has stopped them mattering, and two agents will write one file.
///
/// Stated over the records-only projection for the same reason the acceptance test is: the
/// two items very likely share code as well, and a refusal caused by `governing.rs` would
/// let this pass while proving nothing about records.
#[test]
fn Test_Two_Items_Writing_One_Record_Should_Still_Be_Refused()
{
    let TwoWriters {
        mut document,
        first,
        second,
    } = Two_Record_Writers();
    let contested = format!("{RECORD_DIRECTORY}/OD-CONTESTED-001");
    for item in &mut document.items
    {
        if item.id == first || item.id == second
        {
            item.territory = Territory::Of_Files([contested.clone()]);
        }
    }
    let Contest {
        scratch: _scratch,
        refusal,
    } = Board_Contested_By_Two_Agents("contested-record", &document, &first, &second);

    assert!(
        matches!(refusal, ClaimRefusal::HeldBy { .. }),
        "the refusal must name the holder: {}",
        refusal.Describe()
    );
    assert!(refusal.Describe().contains("agent-a"), "{}", refusal.Describe());
}

/// The spelling control. A finer grain must not reintroduce the hole
/// `Normalize_Path` exists to close: two spellings of one record are one record.
#[test]
fn Test_Two_Spellings_Of_One_Record_Should_Still_Collide()
{
    let lower = Territory::Of_Files(["docs/records/OD-LEDGER-004"]);
    let shouted = Territory::Of_Files([r"docs\Records\OD-LEDGER-004"]);

    assert!(
        !lower.Intersect(&shouted).Permits_Concurrency(),
        "case and separator folding must survive the move to record-level reservations"
    );
}
