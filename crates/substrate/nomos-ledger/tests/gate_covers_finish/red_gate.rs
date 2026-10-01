//! The instance. A passing predicate and a red gate must not finish an item.

use crate::launcher::{
    Bench, Bench_At, Finish_In, Scripted, Standing, State_Of, GATE_FAILED_EXIT, WORKFLOW,
};
use nomos_ledger::{FinishRefusal, ItemState, VerificationRecord};

#[test]
fn Test_A_Passing_Predicate_Should_Not_Finish_An_Item_While_The_Gate_Is_Red()
{
    let scripted = Scripted::New(GATE_FAILED_EXIT, 0);
    let Bench {
        directory,
        mut ledger,
        launcher,
    } = Bench_At("red-gate", Some(WORKFLOW), scripted);

    let refusal = Finish_In(&mut ledger, &directory, &launcher, "T-1")
    .expect_err("a red gate must refuse the finish");

    assert!(
        matches!(refusal, FinishRefusal::GateFailed { .. }),
        "expected GateFailed, got {}",
        refusal.Describe()
    );

    let Standing { state, verified } = State_Of(&directory);
    assert_eq!(state, ItemState::Claimed, "the item must not have finished");
    assert!(
        verified.is_none(),
        "an item refused by the gate must carry no verification record"
    );
}

/// The negative control. Without it, a `GateFailed` that fired unconditionally would pass
/// the test above and nothing would ever be finishable.
#[test]
fn Test_A_Green_Gate_And_A_Passing_Predicate_Should_Finish_The_Item()
{
    let scripted = Scripted::New(0, 0);
    let Bench {
        directory,
        mut ledger,
        launcher,
    } = Bench_At("green-gate", Some(WORKFLOW), scripted);

    let record = Finish_In(&mut ledger, &directory, &launcher, "T-1")
    .expect("a green gate and a passing predicate finish the item");
    let Standing { state, verified } = State_Of(&directory);

    Assert_The_Derived_Step_Ran(&record);
    assert_eq!(state, ItemState::Done);
    assert!(
        verified.and_then(|record| return record.gate).is_some(),
        "the ledger must record what the gate did, or a reader cannot tell an item \
         finished under the gate from one finished before it existed"
    );
}

/// The step the record names must be the one derived from the workflow, not a guess.
fn Assert_The_Derived_Step_Ran(record: &VerificationRecord)
{
    let gate = record
        .gate
        .as_ref()
        .expect("the record must say the gate ran");

    assert_eq!(gate.exit_code, 0);
    assert!(
        gate.argv.iter().any(|argument| return argument == "clippy"),
        "the recorded gate step must be the derived one, got {:?}",
        gate.argv
    );
}
