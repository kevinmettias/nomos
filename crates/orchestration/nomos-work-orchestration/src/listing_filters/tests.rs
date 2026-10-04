//! [`ListingFilters::Admits`], held against items built to sit on each side of each filter.

use nomos_ledger::{Abandonment, ItemId, ItemKind, ItemOrigin, ItemState, LedgerItem, Territory};
use nomos_platform::Timestamp;

use super::{Admission, ListingFilters};

/// A ready item reserving `paths`, with prose that names nothing any test below looks for.
fn Item_Reserving(id: &str, paths: &[&str]) -> LedgerItem
{
    return LedgerItem {
        id: ItemId::New(id),
        title: "a listed item".to_owned(),
        why: "because the board needed one".to_owned(),
        done_when: "the assertion passes".to_owned(),
        kind: ItemKind::Cleanup,
        origin: ItemOrigin::Proposed,
        territory: Territory::Of_Files(paths.iter().copied()),
        state: ItemState::Ready,
        depends_on: Vec::new(),
        blocked: None,
        claim: None,
        verification: None,
        verified: None,
        abandoned: Vec::new(),
        displaced: Vec::new(),
        widened: Vec::new(),
        declined: None,
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

/// The case the claim check's containment exists for, and the one path equality gets wrong: a
/// directory reservation already reserves every file beneath it.
#[test]
fn Test_Admits_Should_Admit_A_Directory_Reservation_For_A_File_Beneath_It()
{
    let item = Item_Reserving("T-1", &["crates/a"]);

    assert_eq!(Touching("crates/a/src/lib.rs").Admits(&item), Some(Admission::Admitted));
}

/// Containment in the other direction: asking about a directory finds an item reserving a file
/// inside it.
#[test]
fn Test_Admits_Should_Admit_A_File_Reservation_For_The_Directory_Holding_It()
{
    let item = Item_Reserving("T-1", &["crates/a/src/lib.rs"]);

    assert_eq!(Touching("crates/a").Admits(&item), Some(Admission::Admitted));
}

/// A sibling whose name merely begins the same way is not inside the path, which is what a
/// prefix test would get wrong.
#[test]
fn Test_Admits_Should_Leave_Out_An_Item_Whose_Territory_Is_Disjoint()
{
    let item = Item_Reserving("T-1", &["crates/ab", "crates/b/src/lib.rs"]);

    assert_eq!(Touching("crates/a").Admits(&item), None);
}

/// An item reserving a record by identifier touches the record's file, because a record file
/// folds onto its identifier for a claim and the listing asks the claim's question.
#[test]
fn Test_Admits_Should_Fold_A_Record_File_Onto_The_Identifier_An_Item_Reserved()
{
    let item = Item_Reserving("T-1", &["docs/records/OD-LEDGER-041"]);

    assert_eq!(
        Touching("docs/records/OD-LEDGER-041-whether-terminal-items-leave-the-active-board-for-an-append-only-archive.md")
            .Admits(&item),
        Some(Admission::Admitted)
    );
}

/// An overlap nobody can decide is listed and marked, never left out: the claim check refuses
/// on it, so leaving it out would call a path free that a claim calls taken.
#[test]
fn Test_Admits_Should_Mark_An_Undecided_Overlap_Rather_Than_Leave_It_Out()
{
    let mut item = Item_Reserving("T-1", &["crates/b"]);
    item.territory = item.territory.With_Pattern("crates/**/lib.rs");

    assert_eq!(Touching("crates/a/src/lib.rs").Admits(&item), Some(Admission::Undecided));
}

/// Each place a text can be recorded on an item is read, and each is enough on its own.
#[test]
fn Test_Admits_Should_Admit_An_Item_Mentioning_The_Text_In_Any_Field_It_Reads()
{
    let base = Item_Reserving("T-1", &["src/a.rs"]);
    let mut by_title = base.clone();
    by_title.title = "names OD-LEDGER-041".to_owned();
    let mut by_why = base.clone();
    by_why.why = "OD-LEDGER-041 decided it".to_owned();
    let mut by_done_when = base.clone();
    by_done_when.done_when = "OD-LEDGER-041 version 2 says so".to_owned();
    let mut by_decline = base.clone();
    by_decline.state = ItemState::Declined { reason: "superseded by OD-LEDGER-041".to_owned() };
    let mut by_abandonment = base.clone();
    by_abandonment.abandoned.push(Abandonment {
        holder: "agent-a".to_owned(),
        reason: "OD-LEDGER-041 moved".to_owned(),
        abandoned_at: Timestamp::From_Unix_Seconds(1),
    });

    for item in [by_title, by_why, by_done_when, by_decline, by_abandonment]
    {
        assert_eq!(Mentioning("OD-LEDGER-041").Admits(&item), Some(Admission::Admitted), "{item:?}");
    }
    assert_eq!(Mentioning("T-1").Admits(&base), Some(Admission::Admitted), "the id is read too");
    assert_eq!(Mentioning("OD-LEDGER-041").Admits(&base), None, "and the control mentions nothing");
}

/// Literal and case-sensitive, and blind to the territory, which is `touching`'s question.
#[test]
fn Test_Admits_Should_Match_Mentions_Literally_And_Leave_The_Territory_Unread()
{
    let item = Item_Reserving("T-1", &["docs/records/OD-LEDGER-041"]);

    assert_eq!(Mentioning("Because").Admits(&item), None, "the why says `because`, lower case");
    assert_eq!(Mentioning("because").Admits(&item), Some(Admission::Admitted));
    assert_eq!(Mentioning("OD-LEDGER-041").Admits(&item), None, "only the territory names it");
}

/// The two filters compose by both having to admit an item.
#[test]
fn Test_Admits_Should_Require_Every_Filter_Given_To_Admit_The_Item()
{
    let item = Item_Reserving("T-1", &["crates/a"]);
    let both = |touching: &str, mentions: &str| {
        return ListingFilters { touching: Some(touching.to_owned()), mentions: Some(mentions.to_owned()) };
    };

    assert_eq!(both("crates/a/x.rs", "listed").Admits(&item), Some(Admission::Admitted));
    assert_eq!(both("crates/a/x.rs", "absent").Admits(&item), None, "mentions refuses it");
    assert_eq!(both("crates/b", "listed").Admits(&item), None, "touching refuses it");
}

/// No filter is no narrowing.
#[test]
fn Test_Admits_Should_Admit_Every_Item_When_No_Filter_Is_Given()
{
    let filters = ListingFilters::default();

    assert!(filters.Is_Empty());
    assert_eq!(filters.Admits(&Item_Reserving("T-1", &["src/a.rs"])), Some(Admission::Admitted));
    assert!(!Touching("src/a.rs").Is_Empty());
    assert!(!Mentioning("a").Is_Empty());
}
