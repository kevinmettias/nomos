//! The defect: an unfinished dependency is not readiness.

use crate::authored::{Board, FINISHED, Item, Item_Declined, NO_CLAIM, Standing};

#[test]
fn Test_An_Item_With_An_Unfinished_Dependency_Should_Not_Be_Listed_Ready()
{
    let board = Board::New(
        "unfinished-dependency",
        &format!(
            "{},{}",
            Item("T-1", "\"src/a.rs\"", Standing {
                state: "Ready",
                depends_on: "",
                tail: NO_CLAIM,
            }),
            Item("T-2", "\"src/b.rs\"", Standing {
                state: "Ready",
                depends_on: "\"T-1\"",
                tail: NO_CLAIM,
            })
        ),
    );

    assert_eq!(
        board.Label_Of("T-2"),
        "waiting",
        "T-2 depends on an unfinished T-1, so nothing can claim it:\n{}",
        board.List(None)
    );
}

/// A declined dependency is not a queue. `waiting` tells the reader that finishing T-1
/// resolves this, and T-1 has already answered `declined` and will not answer again —
/// `OD-LEDGER-020`.
#[test]
fn Test_An_Item_With_A_Declined_Dependency_Should_Be_Listed_Stranded_Not_Waiting()
{
    let board = Board::New(
        "declined-dependency",
        &format!(
            "{},{}",
            Item_Declined("T-1", "\"src/a.rs\"", "superseded by T-3"),
            Item("T-2", "\"src/b.rs\"", Standing {
                state: "Ready",
                depends_on: "\"T-1\"",
                tail: NO_CLAIM,
            })
        ),
    );

    assert_eq!(
        board.Label_Of("T-2"),
        "stranded",
        "T-1 is declined and will never be done, so T-2 is a dead end rather than a queue:\n{}",
        board.List(None)
    );
}

/// The negative control that stops the new label being printed for everything.
///
/// Same board, same shape, no dependency. If this said `waiting` too, the test above would
/// pass while the column had simply stopped saying `ready` at all — which is the same defect
/// with the sign flipped.
#[test]
fn Test_An_Item_With_No_Dependency_Should_Still_Be_Listed_Ready()
{
    let board = Board::New(
        "no-dependency",
        &format!(
            "{},{}",
            Item("T-1", "\"src/a.rs\"", Standing {
                state: "Ready",
                depends_on: "",
                tail: NO_CLAIM,
            }),
            Item("T-2", "\"src/b.rs\"", Standing {
                state: "Ready",
                depends_on: "",
                tail: NO_CLAIM,
            })
        ),
    );

    assert_eq!(board.Label_Of("T-2"), "ready");
    assert_eq!(board.Label_Of("T-1"), "ready");
}

/// The second control, and the sharper one. The dependency is *present* and *satisfied*, so
/// a label driven by "does this item have a `depends_on` entry" would get it wrong while
/// passing both tests above.
#[test]
fn Test_A_Satisfied_Dependency_Should_Leave_An_Item_Ready()
{
    let board = Board::New(
        "satisfied-dependency",
        &format!(
            "{},{}",
            Item("T-1", "\"src/a.rs\"", Standing {
                state: "Done",
                depends_on: "",
                tail: FINISHED,
            }),
            Item("T-2", "\"src/b.rs\"", Standing {
                state: "Ready",
                depends_on: "\"T-1\"",
                tail: NO_CLAIM,
            })
        ),
    );

    assert_eq!(
        board.Label_Of("T-2"),
        "ready",
        "T-1 is Done, so T-2 is genuinely claimable:\n{}",
        board.List(None)
    );
}

/// Filtering has to agree with the column, or an agent asking for work still gets items it
/// cannot take. This is the query the defect was actually costing round trips on.
#[test]
fn Test_Filtering_For_Ready_Should_Not_Return_An_Item_Nothing_Can_Claim()
{
    let board = A_Ready_Item_Behind_An_Unfinished_One();

    let ready = board.List(Some("ready"));

    assert!(ready.contains("T-1"), "T-1 is claimable:\n{ready}");
    assert!(
        !ready.contains("T-2"),
        "T-2 cannot be claimed, so it must not answer a request for ready work:\n{ready}"
    );
}

/// Two `Ready` items, the second of which depends on the first and so cannot be taken.
fn A_Ready_Item_Behind_An_Unfinished_One() -> Board
{
    return Board::New(
        "filter-ready",
        &format!(
            "{},{}",
            Item("T-1", "\"src/a.rs\"", Standing {
                state: "Ready",
                depends_on: "",
                tail: NO_CLAIM,
            }),
            Item("T-2", "\"src/b.rs\"", Standing {
                state: "Ready",
                depends_on: "\"T-1\"",
                tail: NO_CLAIM,
            })
        ),
    );
}
