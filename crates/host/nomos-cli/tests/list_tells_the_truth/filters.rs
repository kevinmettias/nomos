//! The questions agents scripted against the raw ledger, asked of the real binary instead.
//!
//! `OD-LEDGER-041` version 2 measured 1,482 shell calls running a script against
//! `work/ledger.json`, most asking which items touch a path, which mention a record, or what one
//! item is without its hundred-character id. These run the program for the same reason the rest
//! of this suite does: what an agent acts on is what the command prints and exits with.

use crate::authored::{Board, Item, Item_Declined, NO_CLAIM, Standing};

const READY: Standing<'static> = Standing { state: "Ready", depends_on: "", tail: NO_CLAIM };

/// `T-1` reserves the directory `crates/a`, `T-2` another crate, and `T-3` was declined over
/// `crates/a` with a reason naming a record.
fn A_Board_To_Ask(name: &str) -> Board
{
    return Board::New(
        name,
        &format!(
            "{},{},{}",
            Item("T-1", "\"crates/a\"", READY),
            Item("T-2", "\"crates/b\"", READY),
            Item_Declined("T-3", "\"crates/a\"", "superseded by OD-LEDGER-041"),
        ),
    );
}

#[test]
fn Test_Touching_Should_Answer_Which_Live_Items_A_Claim_On_A_File_Would_Overlap()
{
    let board = A_Board_To_Ask("touching");

    let ran = board.Work(&["list", "--touching", "crates/a/src/lib.rs"]);

    assert_eq!(ran.code, 0, "{}", ran.said);
    assert!(ran.said.lines().any(|line| return line.starts_with("T-1 ")), "{}", ran.said);
    assert!(!ran.said.contains("T-2 "), "{}", ran.said);
    assert!(ran.said.contains("1 ended items match"), "T-3 touches it and has ended:\n{}", ran.said);
    assert!(!ran.said.contains("next:"), "{}", ran.said);
}

#[test]
fn Test_Mentions_Should_Find_An_Item_By_The_Reason_It_Was_Declined_For()
{
    let board = A_Board_To_Ask("mentions");

    let ran = board.Work(&["list", "--mentions", "OD-LEDGER-041", "--all"]);

    assert_eq!(ran.code, 0, "{}", ran.said);
    assert!(ran.said.lines().any(|line| return line.starts_with("T-3 ")), "{}", ran.said);
    assert!(!ran.said.contains("T-1 ") && !ran.said.contains("T-2 "), "{}", ran.said);
}

/// A prefix several ids begin with names them all, shows none, and exits non-zero, so an agent
/// branching on the code does not read a list of ids as an item.
#[test]
fn Test_Show_Should_Name_Every_Id_A_Prefix_Begins_And_Exit_Non_Zero()
{
    let board = A_Board_To_Ask("show-prefix");

    let ran = board.Work(&["show", "--item", "T"]);

    assert_ne!(ran.code, 0, "{}", ran.said);
    for id in ["T-1", "T-2", "T-3"]
    {
        assert!(ran.said.contains(&format!("  {id}\n")), "{id}:\n{}", ran.said);
    }
}
