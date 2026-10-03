//! Derivation, not duplication.


/// The test that fails if somebody writes the clippy line into `nomos-ledger` as a
/// constant. A derived step follows the workflow; a copied one silently disagrees with it
/// the day the workflow changes, and two guards for one rule is how they come to disagree.
#[test]
fn Test_Changing_The_Workflow_Should_Change_What_Finish_Runs()
{
    use crate::launcher::{Bench, Bench_At, Finish_In, Scripted, WORKFLOW};

    let altered = WORKFLOW.replace("--all-targets", "--lib");
    let scripted = Scripted::New(0, 0);
    let Bench {
        directory,
        mut ledger,
        launcher,
    } = Bench_At("derived", Some(&altered), scripted);

    let record = Finish_In(&mut ledger, &directory, &launcher, "T-1")
    .expect("the altered workflow still lints");

    let gate = record.gate.expect("the gate ran");
    assert!(
        gate.argv.contains(&"--lib".to_owned()),
        "finish must run what the workflow says, got {:?}",
        gate.argv
    );
    assert!(!gate.argv.contains(&"--all-targets".to_owned()));
}

/// The same, for the `Rules` step: what a finish runs and records is what the workflow's step
/// of that name says today, by the derivation the lint step uses, and not a copy of it.
#[test]
fn Test_Changing_The_Workflows_Rules_Step_Should_Change_What_Finish_Runs()
{
    use crate::launcher::{Bench, Bench_At, Finish_In, Is_Rules_Step, RULES_RUN, Scripted, WORKFLOW};
    use nomos_ledger::OptionalStepOutcome;

    let rewritten = "cargo run --quiet -p nomos-cli --bin nomos -- gate run --root crates";
    let altered = WORKFLOW.replace(RULES_RUN, rewritten);
    let Bench { directory, mut ledger, launcher } = Bench_At("derived-rules", Some(&altered), Scripted::New(0, 0));

    let record = Finish_In(&mut ledger, &directory, &launcher, "T-1")
        .unwrap_or_else(|refusal| panic!("the altered workflow still declares a Rules step: {}", refusal.Describe()));

    let expected: Vec<String> = rewritten.split_whitespace().map(str::to_owned).collect();
    let ran = launcher.Calls().into_iter().find(|argv| return Is_Rules_Step(argv));
    assert_eq!(ran.as_ref(), Some(&expected), "finish must run what the workflow's Rules step says");
    let Some(OptionalStepOutcome::Ran(recorded)) = record.rules
    else
    {
        panic!("a declared Rules step that passed must be recorded as run, got {:?}", record.rules);
    };
    assert_eq!(recorded.argv, expected, "and record what it ran");
}
