//! What `nomos work list --touching` and `--mentions` print, held against one board built so
//! each filter has an item on each side of it.

use super::super::{Bounds, LedgerDocument, ListingFilters, ListingScope, Territory};
use super::listing::{Done, Label_In, Listing_Item, Printed};

/// The path every `--touching` case below asks about: a file inside `crates/a`, which `T-1` and
/// `T-3` reserve as a directory.
const A_FILE_IN_A: &str = "crates/a/src/lib.rs";

/// The board each filter is read against.
///
/// - `T-1`, live, reserves the directory `crates/a`, so it touches a file beneath it.
/// - `T-2`, live, reserves another crate and is the only item whose `why` names `OD-LEDGER-041`.
/// - `T-3` has ended and reserves `crates/a` too: withheld from the live board, admitted by
///   `--touching`.
/// - `T-4`, live, carries an unexpanded pattern, so whether it touches anything is undecided.
/// - `T-5` has ended and reserves a third crate: neither filter admits it.
fn A_Board_To_Filter() -> LedgerDocument
{
    let mut touching_by_directory = Listing_Item("T-1");
    touching_by_directory.territory = Territory::Of_Files(["crates/a"]);
    let mut mentioning = Listing_Item("T-2");
    mentioning.territory = Territory::Of_Files(["crates/b/src/lib.rs"]);
    mentioning.why = "OD-LEDGER-041 refused the archive".to_owned();
    let mut ended_touching = Done("T-3");
    ended_touching.territory = Territory::Of_Files(["crates/a"]);
    let mut undecided = Listing_Item("T-4");
    undecided.territory = Territory::Of_Files(["crates/c"]).With_Pattern("crates/**/lib.rs");
    let mut ended_elsewhere = Done("T-5");
    ended_elsewhere.territory = Territory::Of_Files(["crates/z"]);

    return LedgerDocument {
        schema_version: nomos_ledger::SCHEMA_VERSION,
        items: vec![touching_by_directory, mentioning, ended_touching, undecided, ended_elsewhere],
    };
}

fn Touching(path: &str) -> ListingFilters
{
    return ListingFilters { touching: Some(path.to_owned()), mentions: None };
}

fn Mentioning(text: &str) -> ListingFilters
{
    return ListingFilters { touching: None, mentions: Some(text.to_owned()) };
}

/// The live board under `filters`, as `nomos work list` prints it.
fn Live(filters: &ListingFilters) -> String
{
    return Printed(&A_Board_To_Filter(), Bounds { state: None, scope: ListingScope::Live, filters });
}

/// Exactly the items whose territory overlaps the path, by the claim check's containment: the
/// directory reservation is listed for a file beneath it, and a disjoint one is not. Path
/// equality would list nothing here, which is the mutation this answers.
#[test]
fn Test_Touching_Should_List_Exactly_The_Items_Whose_Territory_Overlaps_The_Path()
{
    let printed = Live(&Touching(A_FILE_IN_A));

    assert_eq!(Label_In(&printed, "T-1"), Some("ready"), "{printed}");
    assert_eq!(Label_In(&printed, "T-2"), None, "T-2 reserves another crate:\n{printed}");
    assert_eq!(Label_In(&printed, "T-5"), None, "{printed}");
}

/// An overlap that cannot be decided is printed and says so, never left out.
#[test]
fn Test_Touching_Should_Print_And_Mark_An_Item_Whose_Overlap_Is_Undecided()
{
    let printed = Live(&Touching(A_FILE_IN_A));

    let row = printed.lines().find(|line| return line.starts_with("T-4")).unwrap_or_else(|| {
        panic!("an undecided overlap must be listed, not left out:\n{printed}")
    });
    assert!(row.contains("[overlap undecided]"), "{row}");
    let decided = printed.lines().find(|line| return line.starts_with("T-1")).unwrap_or_default();
    assert!(!decided.contains("undecided"), "only the undecided row is marked:\n{printed}");
}

#[test]
fn Test_Mentions_Should_List_Exactly_The_Items_That_Mention_The_Text()
{
    let printed = Printed(
        &A_Board_To_Filter(),
        Bounds { state: None, scope: ListingScope::Whole, filters: &Mentioning("OD-LEDGER-041") },
    );

    assert_eq!(Label_In(&printed, "T-2"), Some("ready"), "{printed}");
    for other in ["T-1", "T-3", "T-4", "T-5"]
    {
        assert_eq!(Label_In(&printed, other), None, "{other} mentions nothing:\n{printed}");
    }
}

/// The filters narrow what each bound admits and change none of them: `--state` still picks
/// its one bucket, `--all` still reaches the ended rows, and the two filters narrow each other.
#[test]
fn Test_The_Filters_Should_Narrow_Every_Bound_Without_Changing_It()
{
    let board = A_Board_To_Filter();
    let touching = Touching(A_FILE_IN_A);

    let done = Printed(&board, Bounds { state: Some("done"), scope: ListingScope::Live, filters: &touching });
    assert_eq!(Label_In(&done, "T-3"), Some("done"), "{done}");
    assert_eq!(Label_In(&done, "T-5"), None, "a done item the filter leaves out:\n{done}");
    assert_eq!(Label_In(&done, "T-1"), None, "a match outside the bucket:\n{done}");

    let whole = Printed(&board, Bounds { state: None, scope: ListingScope::Whole, filters: &touching });
    assert_eq!(Label_In(&whole, "T-3"), Some("done"), "{whole}");
    assert_eq!(Label_In(&whole, "T-1"), Some("ready"), "{whole}");
    assert_eq!(Label_In(&whole, "T-5"), None, "{whole}");

    let both = ListingFilters { touching: Some(A_FILE_IN_A.to_owned()), mentions: Some("item T-1".to_owned()) };
    let narrowed = Printed(&board, Bounds { state: None, scope: ListingScope::Whole, filters: &both });
    assert_eq!(Label_In(&narrowed, "T-1"), Some("ready"), "{narrowed}");
    assert_eq!(Label_In(&narrowed, "T-3"), None, "T-3 touches the path and does not mention it:\n{narrowed}");
}

/// The withheld line counts only the ended items the filters admit -- `T-3`, and not `T-5` --
/// and says nothing when there are none.
#[test]
fn Test_A_Filtered_Listing_Should_Count_Only_The_Ended_Items_Its_Filters_Admit()
{
    let printed = Live(&Touching(A_FILE_IN_A));
    assert!(printed.contains("1 ended items match and are not shown"), "{printed}");

    let none_ended = Live(&Touching("crates/b/src/lib.rs"));
    assert!(!none_ended.contains("ended items"), "nothing ended touches crates/b:\n{none_ended}");
}

/// A filtered listing names no item to claim next, for the reason `--state` does not; the
/// unfiltered control does.
#[test]
fn Test_A_Filtered_Listing_Should_Not_Print_The_Next_Line()
{
    assert!(!Live(&Touching(A_FILE_IN_A)).contains("next:"));
    assert!(!Live(&Mentioning("OD-LEDGER-041")).contains("next:"));
    assert!(Live(&ListingFilters::default()).contains("next:"), "the control must name one");
}

/// A filter that matched nothing says which filter it was, rather than that the board is empty.
#[test]
fn Test_A_Filter_That_Matched_Nothing_Should_Name_Itself()
{
    let printed = Live(&Mentioning("absent"));

    assert!(printed.contains("no live item matches --mentions absent"), "{printed}");
    assert!(!printed.contains("nothing on the board is live"), "{printed}");
}
