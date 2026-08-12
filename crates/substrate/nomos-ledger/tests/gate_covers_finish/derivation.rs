//! Derivation, not duplication.

use crate::launcher::{Bench, Bench_At, Finish_In, Scripted, WORKFLOW};

/// The test that fails if somebody writes the clippy line into `nomos-ledger` as a
/// constant. A derived step follows the workflow; a copied one silently disagrees with it
/// the day the workflow changes, and two guards for one rule is how they come to disagree.
#[test]
fn Test_Changing_The_Workflow_Should_Change_What_Finish_Runs()
{
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
