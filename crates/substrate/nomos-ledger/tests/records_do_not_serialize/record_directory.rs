//! The rule: no open item reserves the directory every record lives in.

use crate::board::{Constructed_Writers, Is_Open, RECORD_DIRECTORY, Real_Ledger};

/// The straggler guard, and the reason this file reads the real ledger.
///
/// One item left claiming `docs/records` contains every record anyone else could write, so
/// it re-blocks the whole board by itself. The fix is therefore not incremental and cannot
/// be kept by review alone.
#[test]
fn Test_No_Open_Item_Should_Reserve_The_Whole_Record_Directory()
{
    let offenders = Directory_Reservers(&Real_Ledger());

    assert!(
        offenders.is_empty(),
        "an open item reserving `{RECORD_DIRECTORY}` excludes every other record-writing \
         item, which is the defect P10-RECORD-LOCK closed. Reserve the record the item will \
         write instead. Offenders: {offenders:?}"
    );
}

/// The control that keeps the assertion above from passing over an empty search.
///
/// It used to demand the board hold two open record writers, which `OD-LEDGER-032` measured
/// as red on 26 of the last 30 commits: a board at rest is the normal end state of finished
/// work, and CI checks out a committed ledger. So the teeth are proved on a subject built
/// for the purpose instead of demanded of the live one — the shape
/// `tests/contract/tests/rule_contract_citation.rs` already uses, where the real records are
/// checked and a fixture record proves the comparison can fail.
///
/// The assertion above is honest over an empty board: no open item reserving the directory
/// is exactly true when no item is open. What would not be honest is never finding out
/// whether it can fail, and that is what this answers.
#[test]
fn Test_An_Item_Reserving_The_Whole_Record_Directory_Should_Be_Found()
{
    use nomos_ledger::Territory;

    let mut document = Constructed_Writers(2);
    let straggler = document
        .items
        .first_mut()
        .expect("a board built with two writers holds a first one");
    straggler.territory = Territory::Of_Files([RECORD_DIRECTORY]);
    let offender = straggler.id.clone();

    let found = Directory_Reservers(&document);

    assert_eq!(
        found,
        vec![offender.As_Str().to_owned()],
        "an item reserving `{RECORD_DIRECTORY}` outright must be reported by the search the \
         assertion above makes, or that assertion is green because it looks at nothing"
    );
}

/// Every open item reserving the bare record directory.
///
/// Shared by the assertion and its control, so the control exercises the search the
/// assertion makes rather than a second one written beside it.
fn Directory_Reservers(document: &nomos_ledger::LedgerDocument) -> Vec<String>
{
    use nomos_ledger::Normalize_Path;

    return document
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
}
