//! The rule: no open item reserves the directory every record lives in.

use crate::board::{Is_Open, RECORD_DIRECTORY, Real_Ledger, Reserved_Records};

/// The straggler guard, and the reason this file reads the real ledger.
///
/// One item left claiming `docs/records` contains every record anyone else could write, so
/// it re-blocks the whole board by itself. The fix is therefore not incremental and cannot
/// be kept by review alone.
#[test]
fn Test_No_Open_Item_Should_Reserve_The_Whole_Record_Directory()
{
    use nomos_ledger::Normalize_Path;

    let document = Real_Ledger();

    let offenders: Vec<String> = document
        .items
        .iter()
        .filter(|item| Is_Open(item))
        .filter(|item| {
            item.territory
                .paths
                .iter()
                .any(|path| Normalize_Path(path) == RECORD_DIRECTORY)
        })
        .map(|item| item.id.As_Str().to_owned())
        .collect();

    assert!(
        offenders.is_empty(),
        "an open item reserving `{RECORD_DIRECTORY}` excludes every other record-writing \
         item, which is the defect P10-RECORD-LOCK closed. Reserve the record the item will \
         write instead. Offenders: {offenders:?}"
    );
}

/// Guards the test above against passing because there is nothing to check. A board with
/// no open record-writing items would satisfy it vacuously.
#[test]
fn Test_The_Board_Should_Have_Record_Writing_Items_To_Talk_About()
{
    let document = Real_Ledger();

    let writers = document
        .items
        .iter()
        .filter(|item| Is_Open(item))
        .filter(|item| !Reserved_Records(item).is_empty())
        .count();

    assert!(
        writers >= 2,
        "fewer than two open items reserve a record, so the concurrency this file measures \
         cannot be observed; got {writers}"
    );
}
