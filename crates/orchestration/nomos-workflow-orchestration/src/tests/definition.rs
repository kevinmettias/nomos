//! What publishing a workflow definition refuses, and why each refusal is taken before
//! anything could dispatch.

use super::support::{Arm, Branch_Node, Declaring_Node, FIRST_VERSION, Join_Node, Step_Node, Test_Definition_Id};
use super::*;

/// An agent body the definition cases dispatch, labelled so a queued answer can be
/// correlated with the node that received it.
fn Body_For(node: &str) -> Body
{
    return Agent_Step(EXECUTOR_FAMILY, Task_Envelope(node));
}

#[test]
fn Test_A_Coherent_Definition_Should_Publish_With_Its_Own_Identity_And_Version()
{
    let nodes = vec![Step_Node("check", &[], Body_For("check"))];

    let published = WorkflowDefinition::Publish(Test_Definition_Id(), FIRST_VERSION, nodes).expect("this definition is coherent");

    assert_eq!(published.Id(), &Test_Definition_Id());
    assert_eq!(published.Version(), FIRST_VERSION);
    assert_eq!(published.Nodes().len(), 1);
}

/// The refusal this whole publish-time check exists for: a branch reads a name nothing
/// writes, so nothing could ever decide which arm runs.
#[test]
fn Test_A_Branch_On_A_Value_No_Node_Produces_Should_Be_Refused()
{
    let nodes = vec![
        Step_Node("check", &[], Body_For("check")),
        Branch_Node("decide", "a-value-no-node-produces", vec![Arm(ProducedState::Flagged, &["fix"])]),
        Step_Node("fix", &[], Body_For("fix")),
    ];

    let refused = WorkflowDefinition::Publish(Test_Definition_Id(), FIRST_VERSION, nodes);

    let expected = DefinitionRefusal::BranchesOnUnproducedValue { branch: "decide".to_owned(), value: "a-value-no-node-produces".to_owned() };
    assert_eq!(refused.err(), Some(expected));
}

/// A branch may only read *backward*. Naming a node declared after it is the same defect
/// as naming one that does not exist: at the moment the branch decides, that node has not
/// published anything.
#[test]
fn Test_A_Branch_Reading_A_Later_Node_Should_Be_Refused()
{
    let nodes = vec![
        Branch_Node("decide", "check", vec![Arm(ProducedState::Flagged, &["fix"])]),
        Step_Node("check", &[], Body_For("check")),
        Step_Node("fix", &[], Body_For("fix")),
    ];

    let refused = WorkflowDefinition::Publish(Test_Definition_Id(), FIRST_VERSION, nodes);

    let expected = DefinitionRefusal::BranchesOnUnproducedValue { branch: "decide".to_owned(), value: "check".to_owned() };
    assert_eq!(refused.err(), Some(expected));
}

#[test]
fn Test_A_Step_Requiring_A_Value_No_Node_Produces_Should_Be_Refused()
{
    let nodes = vec![Step_Node("fix", &["check"], Body_For("fix"))];

    let refused = WorkflowDefinition::Publish(Test_Definition_Id(), FIRST_VERSION, nodes);

    let expected = DefinitionRefusal::RequiresUnproducedValue { node: "fix".to_owned(), value: "check".to_owned() };
    assert_eq!(refused.err(), Some(expected));
}

#[test]
fn Test_A_Branch_Arm_Naming_No_Later_Node_Should_Be_Refused()
{
    let nodes = vec![
        Step_Node("check", &[], Body_For("check")),
        Branch_Node("decide", "check", vec![Arm(ProducedState::Flagged, &["a-node-that-is-not-declared"])]),
    ];

    let refused = WorkflowDefinition::Publish(Test_Definition_Id(), FIRST_VERSION, nodes);

    let expected = DefinitionRefusal::NamesUnknownNode { node: "decide".to_owned(), named: "a-node-that-is-not-declared".to_owned() };
    assert_eq!(refused.err(), Some(expected));
}

#[test]
fn Test_A_Join_Naming_No_Earlier_Node_Should_Be_Refused()
{
    let nodes = vec![Step_Node("check", &[], Body_For("check")), Join_Node("done", &["fix"]), Step_Node("fix", &[], Body_For("fix"))];

    let refused = WorkflowDefinition::Publish(Test_Definition_Id(), FIRST_VERSION, nodes);

    let expected = DefinitionRefusal::NamesUnknownNode { node: "done".to_owned(), named: "fix".to_owned() };
    assert_eq!(refused.err(), Some(expected));
}

/// A duplicate name makes every reference to it resolve to one of two nodes arbitrarily,
/// which is why it is refused before any reference is checked rather than after.
#[test]
fn Test_Two_Nodes_Sharing_A_Name_Should_Be_Refused()
{
    let nodes = vec![Step_Node("check", &[], Body_For("first")), Step_Node("check", &[], Body_For("second"))];

    let refused = WorkflowDefinition::Publish(Test_Definition_Id(), FIRST_VERSION, nodes);

    assert_eq!(refused.err(), Some(DefinitionRefusal::DuplicateNodeName { name: "check".to_owned() }));
}

/// The difference between a definition and a plan, in one case.
///
/// The same two steps, the incoherent one second. A definition refuses at publish, so no
/// definition value exists and the first node's body never dispatches. The sequential
/// runner reaches the incoherent step only after the first one has already dispatched, and
/// reports one completed step alongside its refusal -- which is correct for `Run` and is
/// exactly what a definition improves on, because it holds the whole topology before
/// anything runs.
#[test]
fn Test_An_Incoherent_Step_Should_Be_Refused_At_Publish_While_A_Plan_Dispatches_The_Step_Before_It()
{
    let nodes = vec![Step_Node("first", &[], Body_For("first")), Declaring_Node("second", Incoherent_Step(), Body_For("second"))];

    let refused = WorkflowDefinition::Publish(Test_Definition_Id(), FIRST_VERSION, nodes);

    assert_eq!(refused.err(), Some(DefinitionRefusal::IncoherentStep { node: "second".to_owned() }));

    let plan = [
        WorkflowStepPlan { declaration: Coherent_Step(), body: Body_For("first") },
        WorkflowStepPlan { declaration: Incoherent_Step(), body: Body_For("second") },
    ];
    let outcome = Ran_Outcome(&plan, vec![Clean_Executor_Answer("first")]);

    assert!(matches!(outcome, WorkflowOutcome::Refused { ref completed, index: 1 } if completed.len() == 1));
}
