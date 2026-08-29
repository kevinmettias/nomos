//! Unknown is not permission.

use crate::launcher::{Bench, Bench_At, Finish_In, Scripted, Standing, State_Of, WORKFLOW};
use nomos_ledger::{FinishRefusal, GateUnknown, ItemState};

#[test]
fn Test_A_Missing_Workflow_Should_Refuse_Rather_Than_Finish_On_The_Predicate_Alone()
{
    let scripted = Scripted::New(0, 0);
    let Bench {
        directory,
        mut ledger,
        launcher,
    } = Bench_At("no-workflow", None, scripted);

    let refusal = Finish_In(&mut ledger, &directory, &launcher, "T-1")
    .expect_err("an underived gate must refuse");
    let Standing { state, .. } = State_Of(&directory);

    Assert_Nobody_Found_Out(&refusal);
    assert!(
        launcher.Calls().is_empty(),
        "nothing should have been run once the gate could not be established"
    );
    assert_eq!(state, ItemState::Claimed);
}

/// An undetermined gate is not a verdict on the work, and must not read as one.
fn Assert_Nobody_Found_Out(refusal: &FinishRefusal)
{
    assert!(
        matches!(
            refusal,
            FinishRefusal::GateUndetermined {
                cause: GateUnknown::Unreadable { .. },
                ..
            }
        ),
        "expected GateUndetermined, got {}",
        refusal.Describe()
    );
    assert!(
        !refusal.Has_Judged_The_Work(),
        "nobody found out whether the work passes the gate, so this must not read as \
         the work being wrong"
    );
}

/// A workflow whose lint step is a shell script cannot yield an argv without guessing.
#[test]
fn Test_A_Scripted_Gate_Step_Should_Refuse_Rather_Than_Be_Guessed_At()
{
    let workflow = WORKFLOW.replace(
        "cargo clippy --workspace --all-targets -- -D warnings",
        "cargo clippy && cargo doc",
    );
    let scripted = Scripted::New(0, 0);
    let Bench {
        directory,
        mut ledger,
        launcher,
    } = Bench_At("scripted", Some(&workflow), scripted);

    let refusal = Finish_In(&mut ledger, &directory, &launcher, "T-1")
    .expect_err("a scripted gate step must refuse");

    assert!(matches!(
        refusal,
        FinishRefusal::GateUndetermined {
            cause: GateUnknown::NotASingleCommand { .. },
            ..
        }
    ));
}
