//! `nomos work show` naming what an item waits on: the refusal under its state word when no holder
//! is the reason, and every dependency the item was authored with.

use super::super::{ExitCode, ItemId, LedgerDocument, LedgerItem, Render_Show, ShowView};
use super::listing::{LISTING_NOW, Listing_Item};
use nomos_ledger::{Claim, ItemState, Territory};
use nomos_platform::Timestamp;

/// How far from [`LISTING_NOW`] a fixture's claim was taken, and how far its lease runs past it
/// when the fixture wants it live.
const CLAIM_SECONDS: i64 = 1_000;

/// What `show --item <id>` prints on a board of `items`, which must show it.
fn Shown(items: Vec<LedgerItem>, id: &str) -> String
{
    let view = ShowView {
        document: LedgerDocument {
            schema_version: nomos_ledger::SCHEMA_VERSION,
            items,
        },
        now: Timestamp::From_Unix_Seconds(LISTING_NOW),
        current_revision: None,
    };
    let mut output = Vec::new();

    let code = Render_Show(&ItemId::New(id), Ok(view), &mut output);
    let shown = String::from_utf8(output).expect("Render_Show writes only str into the buffer");
    assert_eq!(code, ExitCode::Ok, "{shown}");

    return shown;
}

/// An item whose `depends_on` names each of `dependencies`, in that order.
fn Depending(id: &str, dependencies: &[&str]) -> LedgerItem
{
    let mut item = Listing_Item(id);
    item.depends_on = dependencies.iter().map(|dependency| return ItemId::New(*dependency)).collect();

    return item;
}

/// An item in `state`.
fn In_State(id: &str, state: ItemState) -> LedgerItem
{
    let mut item = Listing_Item(id);
    item.state = state;

    return item;
}

/// A claimed item held by somebody else, its lease live or run out as of [`LISTING_NOW`], over
/// `path`.
fn Claimed(id: &str, live: bool, path: &str) -> LedgerItem
{
    let expires = if live { LISTING_NOW + CLAIM_SECONDS } else { LISTING_NOW - 1 };
    let mut item = In_State(id, ItemState::Claimed);
    item.territory = Territory::Of_Files([path]);
    item.claim = Some(Claim {
        holder: "agent-b".to_owned(),
        acquired_at: Timestamp::From_Unix_Seconds(LISTING_NOW - CLAIM_SECONDS),
        lease_expires_at: Timestamp::From_Unix_Seconds(expires),
    });

    return item;
}

/// The lines between the state word and the item's terms: what `show` says about the state.
fn Under_The_State(shown: &str) -> Vec<&str>
{
    return shown
        .lines()
        .skip_while(|line| return !line.starts_with("state: "))
        .skip(1)
        .take_while(|line| return !line.is_empty())
        .collect();
}

/// The dependency list, which is the last of the item's terms.
fn Dependency_Section(shown: &str) -> Vec<&str>
{
    return shown.lines().skip_while(|line| return !line.starts_with("depends on:")).collect();
}

/// A waiting item says which dependency it waits for, directly under the word, in the sentence
/// the claim check's own refusal gives and `work audit` prints.
#[test]
fn Test_Show_Should_Name_The_Unfinished_Dependency_A_Waiting_Item_Waits_For()
{
    let shown = Shown(vec![Depending("T-2", &["T-1"]), Listing_Item("T-1")], "T-2");

    assert!(shown.contains("state: waiting\n"), "{shown}");
    assert!(
        Under_The_State(&shown).iter().any(|line| return line.starts_with("T-2 depends on T-1, which is ")),
        "{shown}"
    );
}

/// A stranded item names the declined dependency, and the refusal's own remedy with it.
#[test]
fn Test_Show_Should_Name_The_Declined_Dependency_A_Stranded_Item_Will_Never_Get_Past()
{
    let declined = In_State("T-1", ItemState::Declined { reason: "superseded".to_owned() });
    let shown = Shown(vec![Depending("T-2", &["T-1"]), declined], "T-2");

    assert!(shown.contains("state: stranded\n"), "{shown}");
    assert!(
        Under_The_State(&shown).iter().any(|line| {
            return line.starts_with("T-2 depends on T-1, which is ") && line.contains("nomos work decline");
        }),
        "{shown}"
    );
}

/// Every dependency is listed in the order it was authored, each with its own label, and one the
/// board does not carry is said to be absent rather than left out.
#[test]
fn Test_Show_Should_List_Every_Dependency_With_What_List_Would_Call_It()
{
    let items = vec![Depending("T-3", &["T-1", "T-2", "T-9"]), In_State("T-1", ItemState::Done), Listing_Item("T-2")];

    let shown = Shown(items, "T-3");

    assert_eq!(
        Dependency_Section(&shown),
        vec!["depends on: 3 item(s)", "  T-1 done", "  T-2 ready", "  T-9 not on the board"],
        "{shown}"
    );
}

/// An item authored with no dependency says so, rather than leaving the reader to wonder whether
/// the list was not printed.
#[test]
fn Test_Show_Should_Say_An_Item_Depends_On_Nothing_When_It_Does()
{
    let shown = Shown(vec![Listing_Item("T-1")], "T-1");

    assert_eq!(Dependency_Section(&shown), vec!["depends on: none"], "{shown}");
}

/// A ready item and a finished one have nothing to explain, a held one is explained by its blocker
/// lines and a lapsed one by its claim line, so none of them gains a refusal line.
#[test]
fn Test_Show_Should_Add_No_Refusal_Line_Where_The_State_Or_A_Claim_Already_Explains_It()
{
    let kind = "kind: Correction  origin: Proposed";
    let mut held = Listing_Item("T-1");
    held.territory = Territory::Of_Files(["src/shared.rs"]);

    let ready = Shown(vec![Listing_Item("T-1")], "T-1");
    let done = Shown(vec![In_State("T-1", ItemState::Done)], "T-1");
    let blocked = Shown(vec![held, Claimed("T-2", true, "src/shared.rs")], "T-1");
    let lapsed = Shown(vec![Claimed("T-1", false, "src/T-1.rs")], "T-1");

    assert_eq!(Under_The_State(&ready), vec![kind], "{ready}");
    assert_eq!(Under_The_State(&done), vec![kind], "{done}");
    assert!(blocked.contains("state: held\n"), "{blocked}");
    assert_eq!(Under_The_State(&blocked).len(), 2, "the kind and one blocker line:\n{blocked}");
    assert!(lapsed.contains("state: lapsed\n"), "{lapsed}");
    assert_eq!(Under_The_State(&lapsed).len(), 2, "the kind and the claim line:\n{lapsed}");
}
