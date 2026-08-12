//! An item that cannot start until another finishes.

use crate::board::*;

/// A dependency edge that only `validate` reads is a comment. Claiming has to refuse an
/// item whose prerequisite is unfinished, or the ordering is advice.
#[test]
fn Test_Claiming_An_Item_With_An_Unfinished_Dependency_Should_Be_Refused()
{
    let directory = Temp_Dir("claim-dependency");
    let mut ledger = Ledger_At(&directory, &AT_NOW);

    let mut dependent = Item("T-2", &["src/b.rs"]);
    dependent.depends_on = vec![ItemId::New("T-1")];

    ledger
        .Save(&Document(vec![Item("T-1", &["src/a.rs"]), dependent]))
        .expect("a fresh ledger is valid");

    let refusal = Refused(&mut ledger, "T-2", "agent-a");

    assert!(matches!(refusal, ClaimRefusal::DependencyUnmet { .. }), "{}", refusal.Describe());
    assert!(refusal.Is_Retryable(), "finishing T-1 is what resolves this");
    assert!(refusal.Describe().contains("T-1"), "{}", refusal.Describe());
}

/// The negative control. A satisfied dependency must not stand in the way.
#[test]
fn Test_A_Finished_Dependency_Should_Not_Block_A_Claim()
{
    let directory = Temp_Dir("claim-dependency-met");
    let mut ledger = Ledger_At(&directory, &AT_NOW);

    let finished = Finished("T-1", &["src/a.rs"]);
    let mut dependent = Item("T-2", &["src/b.rs"]);
    dependent.depends_on = vec![ItemId::New("T-1")];

    ledger
        .Save(&Document(vec![finished, dependent]))
        .expect("a fresh ledger is valid");

    Take(&mut ledger, "T-2", "agent-a");
}

// ---------------------------------------------------------------------------
// A lapse must not brick the board. P10-LAPSE-BRICKS.
// ---------------------------------------------------------------------------
