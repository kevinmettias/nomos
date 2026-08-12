//! Ordering. The gate runs first and short-circuits.

use crate::common::{Bench, Bench_At, Finish_In, Scripted, WORKFLOW};

#[test]
fn Test_The_Gate_Should_Run_Before_The_Predicate_And_Short_Circuit()
{
    let scripted = Scripted::New(101, 0);
    let Bench {
        directory,
        mut ledger,
        launcher,
    } = Bench_At("ordering", Some(WORKFLOW), scripted);

    let _ = Finish_In(&mut ledger, &directory, &launcher, "T-1");

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

    let _ = Finish_In(&mut ledger, &directory, &launcher, "T-1");

    let calls = launcher.Calls();
    assert_eq!(calls.len(), 2, "both steps must run: {calls:?}");
    assert!(calls.first().is_some_and(|first| {
        return first.iter().any(|argument| return argument == "clippy");
    }));
    assert!(calls.get(1).is_some_and(|second| {
        return second.iter().any(|argument| return argument == "test");
    }));
}
