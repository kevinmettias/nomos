//! Replaying a recorded run against the definition it ran under, and refusing one whose
//! definition has moved.

use super::support::{FIRST_VERSION, Published, Recorded, Replayed, Step_Node, Test_Definition_Id};
use super::*;

/// An agent body, labelled so a queued answer can be correlated with the node that
/// received it.
fn Body_For(node: &str) -> Body
{
    return Agent_Step(EXECUTOR_FAMILY, Task_Envelope(node));
}

/// The definition every replay case records a run of.
fn Recorded_Nodes() -> Vec<WorkflowNode>
{
    return vec![Step_Node("check", &[], Body_For("check"))];
}

/// A bound every replay case runs under. The topology here is one node, so the bound is
/// not what any of these cases varies.
fn One_Node_At_A_Time() -> Parallelism
{
    return Parallelism::Of(super::support::Bound(1));
}

/// A run of [`Recorded_Nodes`] to replay.
fn A_Recorded_Run() -> WorkflowRunRecord
{
    return Recorded(&Published(Recorded_Nodes()), vec![Clean_Executor_Answer("check")], One_Node_At_A_Time());
}

#[test]
fn Test_A_Record_Should_Carry_The_Definition_It_Ran_Under()
{
    let record = A_Recorded_Run();

    assert_eq!(record.Definition().Id(), &Test_Definition_Id());
    assert_eq!(record.Definition().Version(), FIRST_VERSION);
    assert_eq!(record.Definition().Nodes(), Published(Recorded_Nodes()).Nodes());
}

#[test]
fn Test_A_Replay_Against_The_Definition_It_Ran_Under_Should_Run()
{
    let record = A_Recorded_Run();
    let unchanged = Published(Recorded_Nodes());

    let replayed = Replayed(&record, &unchanged, vec![Clean_Executor_Answer("check")]);

    assert_eq!(replayed.expect("an unchanged definition replays").outcome, record.Produced().outcome);
}

/// The refusal this comparison exists for: the same identity at the same version, saying
/// something different. Nothing about the identity or the version would have caught it,
/// and without the content comparison the replay would have run and reported a result
/// attributed to a definition that no longer says what it said.
#[test]
fn Test_A_Replay_Whose_Definition_Moved_Under_The_Same_Version_Should_Be_Refused()
{
    let record = A_Recorded_Run();
    let moved = Published(vec![Step_Node("check", &[], Body_For("something-else"))]);

    let replayed = Replayed(&record, &moved, vec![Clean_Executor_Answer("check")]);

    let expected = ReplayRefusal::Content { id: Test_Definition_Id(), version: FIRST_VERSION };
    assert_eq!(replayed.err(), Some(expected));
}

#[test]
fn Test_A_Replay_Against_A_Different_Identity_Should_Be_Refused()
{
    let record = A_Recorded_Run();
    let other = WorkflowDefinition::Publish(WorkflowDefinitionId::New("some.other.workflow"), FIRST_VERSION, Recorded_Nodes())
        .expect("this definition is coherent");

    let replayed = Replayed(&record, &other, vec![Clean_Executor_Answer("check")]);

    let expected = ReplayRefusal::Identity { recorded: Test_Definition_Id(), offered: WorkflowDefinitionId::New("some.other.workflow") };
    assert_eq!(replayed.err(), Some(expected));
}

#[test]
fn Test_A_Replay_Against_A_Later_Version_Should_Be_Refused()
{
    let record = A_Recorded_Run();
    let later = WorkflowDefinition::Publish(Test_Definition_Id(), NEXT_VERSION, Recorded_Nodes()).expect("this definition is coherent");

    let replayed = Replayed(&record, &later, vec![Clean_Executor_Answer("check")]);

    let expected = ReplayRefusal::Version { recorded: FIRST_VERSION, offered: NEXT_VERSION };
    assert_eq!(replayed.err(), Some(expected));
}

/// The version the same definition is republished at when a case needs a second one.
const NEXT_VERSION: u32 = 2;
