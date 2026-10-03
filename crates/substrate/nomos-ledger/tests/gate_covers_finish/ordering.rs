//! Ordering. The gate runs first and short-circuits.

use crate::launcher::{Bench, Bench_At, Finish_In, Is_Rules_Step, Scripted, GATE_FAILED_EXIT, WORKFLOW};

/// How many steps a finish runs when the gate is green: the gate's lint step first, its
/// rules step second, the predicate third.
const STEPS_WHEN_THE_GATE_IS_GREEN: usize = 3;

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
         tests passed in the same breath as being told they cannot land -- and before the \
         rules step, which would spend minutes on a finish already refused: {calls:?}"
    );
    assert!(calls.iter().flatten().any(|argument| return argument == "clippy"));
}

/// The control for the ordering test: when the gate is green every step runs, the gate's two
/// first and the predicate last.
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
        "every step must run: {calls:?}"
    );
    assert!(calls.first().is_some_and(|first| {
        return first.iter().any(|argument| return argument == "clippy");
    }));
    assert!(calls.get(1).is_some_and(|second| return Is_Rules_Step(second)), "{calls:?}");
    assert!(calls.get(2).is_some_and(|third| {
        return third.iter().any(|argument| return argument == "test");
    }));
}
