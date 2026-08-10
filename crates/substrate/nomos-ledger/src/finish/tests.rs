//! What this module promises, exercised.

use super::*;

#[test]
fn Test_Short_Output_Should_Be_Kept_Whole()
{
    assert_eq!(Tail_Of("all good", 100), "all good");
}

#[test]
fn Test_Long_Output_Should_Keep_The_End()
{
    let long = "a".repeat(50) + "the failure";

    let tail = Tail_Of(&long, 11);

    assert_eq!(tail, "the failure");
}

/// Slicing a multi-byte string at an arbitrary byte offset panics. A predicate that
/// prints a non-ASCII character must not be able to crash the ledger.
#[test]
fn Test_Truncation_Should_Survive_Multi_Byte_Characters()
{
    let text = "é".repeat(100);

    let tail = Tail_Of(&text, 51);

    assert!(tail.len() <= 51);
    assert!(tail.chars().all(|character| character == 'é'));
}

/// The distinction the whole enum exists for.
#[test]
fn Test_Only_A_Failing_Predicate_Should_Judge_The_Work()
{
    let item = ItemId::New("T-1");

    assert!(
        FinishRefusal::PredicateFailed {
            item: item.clone(),
            exit_code: 1,
            output_tail: String::new(),
        }
        .Judged_The_Work()
    );

    // A red gate is an answer about the work too. The author's next move differs
    // from a failing test, which is why it is a separate variant, but "nobody found
    // out" it is not.
    assert!(
        FinishRefusal::GateFailed {
            item: item.clone(),
            argv: vec!["cargo".to_owned(), "clippy".to_owned()],
            exit_code: 101,
            output_tail: String::new(),
        }
        .Judged_The_Work()
    );

    for prevented in [
        FinishRefusal::GateUndetermined {
            item: item.clone(),
            cause: GateUnknown::NoSuchStep {
                step: LINT_STEP.to_owned(),
            },
        },
        FinishRefusal::NoPredicate {
            item: item.clone(),
            done_when: "when it works".to_owned(),
        },
        FinishRefusal::CouldNotRun {
            item: item.clone(),
            cause: "no such program".to_owned(),
        },
        FinishRefusal::NoVerdict {
            item: item.clone(),
            outcome: ExitOutcome::TimedOut,
        },
        FinishRefusal::NotRecorded {
            cause: "locked".to_owned(),
        },
    ]
    {
        assert!(
            !prevented.Judged_The_Work(),
            "{} is not a statement about the work",
            prevented.Describe()
        );
    }
}

#[test]
fn Test_Every_Refusal_Should_Describe_Itself_Usefully()
{
    let item = ItemId::New("T-1");
    let refusals = [
        FinishRefusal::NotHeld {
            refusal: ClaimRefusal::NoSuchItem { item: item.clone() },
        },
        FinishRefusal::NoPredicate {
            item: item.clone(),
            done_when: "when it works".to_owned(),
        },
        FinishRefusal::CouldNotRun {
            item: item.clone(),
            cause: "no such program".to_owned(),
        },
        FinishRefusal::NoVerdict {
            item: item.clone(),
            outcome: ExitOutcome::TimedOut,
        },
        FinishRefusal::PredicateFailed {
            item: item.clone(),
            exit_code: 101,
            output_tail: "assertion failed".to_owned(),
        },
        FinishRefusal::GateUndetermined {
            item: item.clone(),
            cause: GateUnknown::NotASingleCommand {
                step: LINT_STEP.to_owned(),
                run: "a && b".to_owned(),
            },
        },
        FinishRefusal::GateFailed {
            item,
            argv: vec!["cargo".to_owned(), "clippy".to_owned()],
            exit_code: 101,
            output_tail: "indexing may panic".to_owned(),
        },
        FinishRefusal::NotRecorded {
            cause: "locked".to_owned(),
        },
    ];

    for refusal in &refusals
    {
        assert!(
            refusal.Describe().len() > 15,
            "{} is too terse to act on",
            refusal.Describe()
        );
    }
}
