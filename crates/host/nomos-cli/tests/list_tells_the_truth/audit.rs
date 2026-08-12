//! P10-AUDIT-STATE: `audit` answers the same question, so it must give the same answer.
//!
//! `list` and `audit` are two readings of one rule — what would refuse a claim right now.
//! `audit` had its own implementation of it, `ExclusionLedger::Conflicts`, which compares
//! territory and knows nothing about state or `depends_on`. It therefore answered for
//! finished work nobody is queued behind, and said nothing at all about the one refusal an
//! agent looking for work most needs to hear. These tests hold the two readings together:
//! the board below carries a terminal item, a dependency-blocked item and a
//! territory-blocked item at once, which is the only shape in which both halves of the
//! disagreement are visible in a single command.

use crate::common::{Audit_Line, Board, FINISHED, Held_By, Item, NO_CLAIM, Standing};

/// The board both halves of the defect are visible on.
///
/// * `T-1` holds `src/shared.rs`, so `T-2` is refused for territory.
/// * `T-3` depends on an unfinished `T-4`, so it is refused for a dependency — the refusal
///   `Conflicts` cannot express.
/// * `T-4` is claimable, and is the control against a report that prints everything.
/// * `T-5` is `Done` on the *same* held ground as `T-2`. Territory alone cannot tell them
///   apart, which is exactly how the old audit came to describe finished work as blocked.
fn Mixed_Board(name: &str) -> Board
{
    return Board::New(
        name,
        &format!(
            "{},{},{},{},{}",
            Item("T-1", "\"src/shared.rs\"", Standing {
                state: "Claimed",
                depends_on: "",
                tail: &Held_By("agent-a"),
            }),
            Item("T-2", "\"src/shared.rs\"", Standing {
                state: "Ready",
                depends_on: "",
                tail: NO_CLAIM,
            }),
            Item("T-3", "\"src/c.rs\"", Standing {
                state: "Ready",
                depends_on: "\"T-4\"",
                tail: NO_CLAIM,
            }),
            Item("T-4", "\"src/d.rs\"", Standing {
                state: "Ready",
                depends_on: "",
                tail: NO_CLAIM,
            }),
            Item("T-5", "\"src/shared.rs\"", Standing {
                state: "Done",
                depends_on: "",
                tail: FINISHED,
            })
        ),
    );
}

/// The pin. One board, one command, both halves of the disagreement asserted at once.
///
/// It goes red if a terminal item comes back into the report, and it goes red if a
/// dependency refusal drops out of it. Those are the two failures the old implementation
/// had simultaneously, and either one alone is the defect returning.
#[test]
fn Test_Audit_Should_Report_Every_Live_Refusal_And_Only_Those()
{
    let board = Mixed_Board("audit-mixed");

    let audit = board.Audit();

    Assert_Both_Live_Refusals_Are_Reported(&audit);

    assert!(
        Audit_Line(&audit, "T-5").is_none(),
        "T-5 is done; nobody is queued behind it, so it must be absent rather than \
         sorted below:\n{audit}"
    );
    assert!(
        Audit_Line(&audit, "T-4").is_none(),
        "T-4 is claimable, so the audit has nothing to say about it:\n{audit}"
    );
    assert!(
        Audit_Line(&audit, "T-1").is_none(),
        "T-1 is the holder, not a blocked item:\n{audit}"
    );
}

/// Territory and dependency are two refusals the board has simultaneously, and either one
/// alone is the defect returning.
fn Assert_Both_Live_Refusals_Are_Reported(audit: &str)
{
    let held = Audit_Line(audit, "T-2")
        .unwrap_or_else(|| panic!("T-2 is on ground agent-a holds:\n{audit}"));
    let waiting = Audit_Line(audit, "T-3").unwrap_or_else(|| {
        panic!(
            "T-3 is refused for an unfinished dependency, which the audit used to be \
             silent about:\n{audit}"
        )
    });

    assert!(
        held.contains("held") && held.contains("T-1") && held.contains("agent-a"),
        "the territory refusal must name what holds the ground and who:\n{held}"
    );
    assert!(
        waiting.contains("waiting") && waiting.contains("T-4"),
        "the dependency refusal must name the dependency:\n{waiting}"
    );
}

/// The audit and the listing must use the same word for the same item.
///
/// The line count was only how the disagreement was noticed. This is the disagreement
/// itself: two guards for one rule, asserted against each other rather than against a
/// number. Any item the audit answers for is one the listing must already call unclaimable,
/// with the same label.
#[test]
fn Test_Audit_Should_Agree_With_The_Listing_Item_For_Item()
{
    let board = Mixed_Board("audit-agrees");

    let audit = board.Audit();

    let mut checked = 0_u32;
    for line in audit.lines()
    {
        let named = Assert_The_Listing_Agrees(&board, line, &audit);
        checked = checked.saturating_add(named);
    }

    assert_eq!(
        checked, 2,
        "the board has exactly two blocked items; a different count means the audit is \
         answering for something else:\n{audit}"
    );
}

/// One audit line against what the listing calls the same item. Answers 1 where a line named
/// an item at all, which is what the count above measures.
fn Assert_The_Listing_Agrees(board: &Board, line: &str, audit: &str) -> u32
{
    let mut fields = line.split_whitespace();
    let (Some(item), Some(label)) = (fields.next(), fields.next())
    else
    {
        return 0;
    };

    assert_eq!(
        board.Label_Of(item),
        label,
        "`audit` and `list` disagree about {item}:\n{audit}"
    );

    return 1;
}

/// The control against a repair that simply prints less.
///
/// Every assertion above is satisfied by an `audit` that has stopped reporting anything at
/// all except two hardcoded lines. Move the ground apart and finish the dependency, and the
/// same command must fall silent — and say so, because an empty report and a report that
/// cannot express the blockage look identical otherwise.
#[test]
fn Test_Audit_Should_Say_So_When_Nothing_Is_Blocked()
{
    let board = A_Board_Where_Nothing_Refuses();

    let audit = board.Audit();

    assert!(
        Audit_Line(&audit, "T-2").is_none() && Audit_Line(&audit, "T-3").is_none(),
        "nothing on this board refuses a claim:\n{audit}"
    );
    assert!(
        audit.contains("nothing"),
        "an empty report must say it found nothing rather than print nothing:\n{audit}"
    );
}

/// The same four items as the mixed board, with the ground moved apart and the dependency
/// finished, so that nothing on it refuses a claim.
fn A_Board_Where_Nothing_Refuses() -> Board
{
    return Board::New(
        "audit-clear",
        &format!(
            "{},{},{},{}",
            Item("T-1", "\"src/a.rs\"", Standing {
                state: "Claimed",
                depends_on: "",
                tail: &Held_By("agent-a"),
            }),
            Item("T-2", "\"src/b.rs\"", Standing {
                state: "Ready",
                depends_on: "",
                tail: NO_CLAIM,
            }),
            Item("T-3", "\"src/c.rs\"", Standing {
                state: "Ready",
                depends_on: "\"T-4\"",
                tail: NO_CLAIM,
            }),
            Item("T-4", "\"src/d.rs\"", Standing {
                state: "Done",
                depends_on: "",
                tail: FINISHED,
            })
        ),
    );
}
