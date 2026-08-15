//! Giving up a claim, as the next person meets it: the real binary, a real ledger.
//!
//! P10-ABANDON-REASON is about a reason that was required and then dropped. `abandon`
//! refuses without `--reason`, [`ReleaseOutcome::Abandoned`] carries it non-optionally, and
//! the store matched it with `{ .. }` and wrote the state alone — so the one thing the
//! holder was compelled to write down was the one thing nothing kept.
//!
//! These tests run the program rather than calling a function. Keeping the reason in the
//! ledger and never reporting it would be most of the way back to not keeping it: a record
//! only somebody willing to read JSON can find is not a record the next agent will read.
//!
//! [`ReleaseOutcome::Abandoned`]: nomos_ledger::ReleaseOutcome::Abandoned

#[path = "scratch_ledger/board.rs"]
mod board;

use board::{Board, Ran};

/// The reason these tests assert on, as text.
///
/// Asserting that a field is present would pass over an empty string, which is the shape
/// the defect would come back as.
const REASON: &str = "the corpus this needs is on no runner here; stopped before inventing one";

/// A scratch board holding one claimable item.
///
/// The item text is what this suite is about and stays here; the directory it is written
/// into is `scratch_ledger/board.rs`, shared with the suites beside this one.
fn A_Board(name: &str) -> Board
{
    return Board::New(
        "abandon",
        name,
        "{\n  \"schema_version\": 1,\n  \"items\": [\
         {\"id\":\"T-1\",\"title\":\"item T-1\",\"why\":\"because\",\
         \"done_when\":\"the tests pass\",\
         \"territory\":{\"resolution\":\"File\",\"paths\":[\"src/a.rs\"],\"patterns\":[]},\
         \"state\":\"Ready\",\"depends_on\":[],\"blocked\":null,\
         \"claim\":null,\"verification\":null,\"verified\":null}\
         ]\n}\n",
    );
}

/// Claims and then gives up, which is the cycle every test here needs.
///
/// A free function rather than a method, because the board is shared scaffolding and this
/// cycle is what only this suite is about.
fn Claim_Then_Abandon(board: &Board, holder: &str, reason: &str)
{
    let Ran { said, code } = board.Work(&["claim", "--item", "T-1", "--holder", holder]);
    assert_eq!(code, 0, "the claim must be granted: {said}");

    let Ran { said, code } = board.Work(&[
        "abandon", "--item", "T-1", "--holder", holder, "--reason", reason,
    ]);
    assert_eq!(code, 0, "a holder may give up its own claim: {said}");
}

/// The assertion the item exists for, made where a person would actually read it.
#[test]
fn Test_An_Abandoned_Item_Should_Report_Who_Stopped_And_Why()
{
    let board = A_Board("reports-reason");
    Claim_Then_Abandon(&board, "agent-a", REASON);

    let Ran { said: shown, code } = board.Work(&["show", "--item", "T-1"]);

    assert_eq!(code, 0, "{shown}");
    assert!(
        shown.contains(REASON),
        "the reason the holder was required to give is not readable:\n{shown}"
    );
    assert!(
        shown.contains("agent-a"),
        "the report does not say who stopped:\n{shown}"
    );
}

/// The other half of `done_when`: a record of an abandonment must not go on excluding.
///
/// Asserted through `claim` rather than through the state word, because being takeable is
/// the thing that matters and the column has been wrong about it before — which is the
/// whole of P9-DEPENDS-ON.
#[test]
fn Test_An_Abandoned_Item_Should_Be_Claimable_By_Somebody_Else()
{
    let board = A_Board("still-claimable");
    Claim_Then_Abandon(&board, "agent-a", REASON);

    let Ran { said, code } = board.Work(&["claim", "--item", "T-1", "--holder", "agent-b"]);
    assert_eq!(code, 0, "an abandoned item must be takeable: {said}");

    let Ran { said: shown, .. } = board.Work(&["show", "--item", "T-1"]);
    assert!(
        shown.contains(REASON),
        "the next claim erased the record of the last one:\n{shown}"
    );
}

/// Two abandonments are two reasons, and the surface reports both.
#[test]
fn Test_Every_Abandonment_Should_Be_Reported_And_Not_Only_The_Last()
{
    let board = A_Board("reports-both");
    Claim_Then_Abandon(&board, "agent-a", "went to look at something else");
    Claim_Then_Abandon(&board, "agent-b", REASON);

    let Ran { said: shown, .. } = board.Work(&["show", "--item", "T-1"]);

    assert!(
        shown.contains("went to look at something else"),
        "the earlier reason was overwritten by the later one:\n{shown}"
    );
    assert!(shown.contains(REASON), "{shown}");
}

/// The negative control for all three above.
///
/// Everything here is satisfied by a `show` that prints the whole ledger file, or by one
/// that never prints anything about abandonment at all — the first would pass by saying
/// too much, and the assertions would be measuring nothing. An item nobody abandoned must
/// report no abandonment.
#[test]
fn Test_An_Item_Nobody_Abandoned_Should_Report_None()
{
    let board = A_Board("nothing-to-report");

    let Ran { said, code } = board.Work(&["claim", "--item", "T-1", "--holder", "agent-a"]);
    assert_eq!(code, 0, "{said}");

    let Ran { said: shown, code } = board.Work(&["show", "--item", "T-1"]);

    assert_eq!(code, 0, "{shown}");
    assert!(
        !shown.contains("abandoned"),
        "an item nobody gave up reports an abandonment:\n{shown}"
    );
    assert!(
        shown.contains("agent-a"),
        "the live claim is missing, so this test would pass on a `show` that prints \
         nothing at all:\n{shown}"
    );
}

/// A `show` for an item that is not there must say so rather than report an empty one.
#[test]
fn Test_Showing_An_Item_That_Is_Not_There_Should_Refuse()
{
    let board = A_Board("no-such-item");

    let Ran { said: shown, code } = board.Work(&["show", "--item", "T-9"]);

    assert_ne!(code, 0, "a missing item reported as success: {shown}");
    assert!(shown.contains("T-9"), "{shown}");
}
