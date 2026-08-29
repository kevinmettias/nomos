//! An item that cannot start until another finishes.

use crate::board::*;

/// A dependency edge that only `validate` reads is a comment. Claiming has to refuse an
/// item whose prerequisite is unfinished, or the ordering is advice.
#[test]
fn Test_Claiming_An_Item_With_An_Unfinished_Dependency_Should_Be_Refused()
{
    let directory = Temp_Dir("claim-dependency");
    let mut ledger = Ledger_At(directory.As_Path(), &AT_NOW);

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

/// A fresh board with `T-1` declined and `T-2` depending on it — the fixture
/// `Test_Claiming_An_Item_With_A_Declined_Dependency_Should_Be_Refused_Non_Retryably`
/// exercises.
fn Declined_Dependency_Board() -> (Scratch, FileLedger<StdFileSystem, &'static FixedClock, FileLock>)
{
    let directory = Temp_Dir("claim-dependency-declined");
    let ledger = Ledger_At(directory.As_Path(), &AT_NOW);

    let mut dependent = Item("T-2", &["src/b.rs"]);
    dependent.depends_on = vec![ItemId::New("T-1")];

    ledger
        .Save(&Document(vec![
            Declined("T-1", &["src/a.rs"], "superseded by T-3"),
            dependent,
        ]))
        .expect("a fresh ledger is valid");

    return (directory, ledger);
}

/// A declined dependency is a dead end, not a queue — `OD-LEDGER-020`. Reporting it as
/// `DependencyUnmet` tells the caller that waiting resolves this, and no amount of waiting
/// does: the dependency has already answered and will not answer again.
#[test]
fn Test_Claiming_An_Item_With_A_Declined_Dependency_Should_Be_Refused_Non_Retryably()
{
    let (_directory, mut ledger) = Declined_Dependency_Board();

    let refusal = Refused(&mut ledger, "T-2", "agent-a");

    assert!(
        matches!(refusal, ClaimRefusal::DependencyDeclined { .. }),
        "{}",
        refusal.Describe()
    );
    assert!(
        !refusal.Is_Retryable(),
        "finishing a declined dependency is not a thing that happens: {}",
        refusal.Describe()
    );
    assert!(refusal.Describe().contains("T-1"), "{}", refusal.Describe());
    assert!(refusal.Describe().contains("T-2"), "{}", refusal.Describe());
    assert!(
        refusal.Describe().contains("decline"),
        "the refusal must name the remedy, not just the dead end: {}",
        refusal.Describe()
    );
}

/// The negative control. A satisfied dependency must not stand in the way.
#[test]
fn Test_A_Finished_Dependency_Should_Not_Block_A_Claim()
{
    let directory = Temp_Dir("claim-dependency-met");
    let mut ledger = Ledger_At(directory.As_Path(), &AT_NOW);

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
