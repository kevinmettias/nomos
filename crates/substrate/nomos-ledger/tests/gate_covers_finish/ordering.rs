//! Ordering. The gate runs first and short-circuits.

use crate::launcher::{Bench, Bench_At, Finish_In, Scripted, GATE_FAILED_EXIT, WORKFLOW};

/// How many steps a finish runs when the gate is green: the gate first, the predicate second.
const STEPS_WHEN_THE_GATE_IS_GREEN: usize = 2;

#[test]
fn Test_The_Gate_Should_Run_Before_The_Predicate_And_Short_Circuit()
{
    let scripted = Scripted::New(GATE_FAILED_EXIT, 0);
    let Bench {
        directory,
        mut ledger,
        launcher,
    } = Bench_At("ordering", Some(WORKFLOW), scripted);

    Finish_In(&mut ledger, &directory, &launcher, "T-1")
        .expect_err("a red gate must refuse the finish before the predicate runs");

    let calls = launcher.Calls();
    assert_eq!(
        calls.len(),
        1,
        "a red gate must stop before the predicate, so the author is not told their \
         tests passed in the same breath as being told they cannot land: {calls:?}"
    );
    assert!(calls.iter().flatten().any(|argument| return argument == "clippy"));
}

/// The control for the ordering test: when the gate is green both run, gate first.
#[test]
fn Test_A_Green_Gate_Should_Still_Run_The_Predicate_Second()
{
    let scripted = Scripted::New(0, 0);
    let Bench {
        directory,
        mut ledger,
        launcher,
    } = Bench_At("ordering-green", Some(WORKFLOW), scripted);

    Finish_In(&mut ledger, &directory, &launcher, "T-1")
        .expect("a green gate and a passing predicate finish the item");

    let calls = launcher.Calls();
    assert_eq!(
        calls.len(),
        STEPS_WHEN_THE_GATE_IS_GREEN,
        "both steps must run: {calls:?}"
    );
    assert!(calls.first().is_some_and(|first| {
        return first.iter().any(|argument| return argument == "clippy");
    }));
    assert!(calls.get(1).is_some_and(|second| {
        return second.iter().any(|argument| return argument == "test");
    }));
}
