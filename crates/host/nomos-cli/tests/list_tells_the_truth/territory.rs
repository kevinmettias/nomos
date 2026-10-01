//! The same word was lying about territory too.

use crate::authored::{Board, Held_By, Item, NO_CLAIM, Standing};

/// Held ground is the other way `ready` was false, and it is the one that was measured
/// widest: eight items, one claim. It costs nothing extra to report, because it comes from
/// the same refusal the dependency case does.
#[test]
fn Test_An_Item_On_Held_Ground_Should_Not_Be_Listed_Ready()
{
    let board = Board::New(
        "held-ground",
        &format!(
            "{},{}",
            Item("T-1", "\"src/shared.rs\"", Standing {
                state: "Claimed",
                depends_on: "",
                tail: &Held_By("agent-a"),
            }),
            Item("T-2", "\"src/shared.rs\"", Standing {
                state: "Ready",
                depends_on: "",
                tail: NO_CLAIM,
            })
        ),
    );

    assert_eq!(
        board.Label_Of("T-2"),
        "held",
        "agent-a holds overlapping ground:\n{}",
        board.List(None)
    );
}

/// The control for the one above. Move the territory apart and the same board reports the
/// same item as claimable.
#[test]
fn Test_An_Item_On_Free_Ground_Should_Still_Be_Listed_Ready()
{
    let board = Board::New(
        "free-ground",
        &format!(
            "{},{}",
            Item("T-1", "\"src/a.rs\"", Standing {
                state: "Claimed",
                depends_on: "",
                tail: &Held_By("agent-a"),
            }),
            Item("T-2", "\"src/b.rs\"", Standing {
                state: "Ready",
                depends_on: "",
                tail: NO_CLAIM,
            })
        ),
    );

    assert_eq!(board.Label_Of("T-2"), "ready");
}
