//! Branching, joining, the bound a caller states on a dispatch group, the determinism that
//! survives it, and the cache a step's own declaration permits.

use super::support::{Arm, Bound, Branch_Node, Dispositions, Grouped_Names, Join_Node, Published, Recorded, Step_Node};
use super::*;

/// An agent body, labelled so a queued answer can be correlated with the node that
/// received it.
fn Body_For(node: &str) -> Body
{
    return Agent_Step(EXECUTOR_FAMILY, Task_Envelope(node));
}

/// The Check-then-decide-then-Fix-or-not-then-reconverge shape, which is the whole of what
/// a branch and a join are for.
fn Branching_Nodes() -> Vec<WorkflowNode>
{
    return vec![
        Step_Node("check", &[], Body_For("check")),
        Branch_Node("decide", "check", vec![Arm(ProducedState::Flagged, &["fix"]), Arm(ProducedState::Clean, &["publish"])]),
        Step_Node("fix", &[], Body_For("fix")),
        Step_Node("publish", &[], Body_For("publish")),
        Join_Node("done", &["fix", "publish"]),
    ];
}

/// A cacheable declaration: the same trivially coherent step, saying a prior result may
/// stand in for a fresh execution.
fn Cacheable_Step() -> WorkflowStep
{
    let mut step = Coherent_Step();
    step.cacheability = Cacheability::Cacheable { key_inputs: vec!["goal".to_owned()] };

    return step;
}

/// A step node declaring [`Cacheable_Step`].
fn Cacheable_Node(name: &str, requires: &[&str], body: Body) -> WorkflowNode
{
    let requires = requires.iter().map(|value| return (*value).to_owned()).collect();

    return WorkflowNode::Step { name: name.to_owned(), requires, step: WorkflowStepPlan { declaration: Cacheable_Step(), body } };
}

/// The arm the branched-on node's state chose runs; every node of every other arm is
/// skipped, and the queue proves it -- exactly one answer is queued for the arm that runs,
/// so a `publish` that dispatched would find nothing left and say so.
#[test]
fn Test_A_Branch_Should_Run_The_Arm_Its_Condition_Chose_And_Skip_The_Other()
{
    let definition = Published(Branching_Nodes());

    let record = Recorded(&definition, vec![Errored_Executor_Answer("check"), Clean_Executor_Answer("fix")], Parallelism::Of(Bound(2)));

    assert_eq!(
        Dispositions(record.Produced()),
        vec![
            ("check".to_owned(), NodeDisposition::Dispatched { produced: ProducedState::Flagged }),
            ("decide".to_owned(), NodeDisposition::Settled { produced: ProducedState::Flagged }),
            ("fix".to_owned(), NodeDisposition::Dispatched { produced: ProducedState::Clean }),
            ("publish".to_owned(), NodeDisposition::Skipped { by: "decide".to_owned() }),
            ("done".to_owned(), NodeDisposition::Settled { produced: ProducedState::Clean }),
        ]
    );
}

/// The other arm of the same definition, chosen by the other state -- which is what makes
/// the condition load-bearing rather than the first arm being a default.
#[test]
fn Test_The_Other_State_Should_Choose_The_Other_Arm()
{
    let definition = Published(Branching_Nodes());

    let record = Recorded(&definition, vec![Clean_Executor_Answer("check"), Clean_Executor_Answer("publish")], Parallelism::Of(Bound(2)));

    let dispositions = Dispositions(record.Produced());
    assert_eq!(dispositions.get(2), Some(&("fix".to_owned(), NodeDisposition::Skipped { by: "decide".to_owned() })));
    assert_eq!(dispositions.get(3), Some(&("publish".to_owned(), NodeDisposition::Dispatched { produced: ProducedState::Clean })));
}

/// Which arm ran, and why: the value read, the state it was read in, and the arm that
/// state selected.
#[test]
fn Test_The_Run_Should_Report_Which_Arm_Ran_And_Why()
{
    let definition = Published(Branching_Nodes());

    let record = Recorded(&definition, vec![Errored_Executor_Answer("check"), Clean_Executor_Answer("fix")], Parallelism::Of(Bound(2)));

    let expected = BranchChoice { branch: "decide".to_owned(), on: "check".to_owned(), observed: Some(ProducedState::Flagged), arm: Some(0) };
    assert_eq!(record.Produced().choices, vec![expected]);
}

/// A join settles over the arms it names once each has either run or been skipped, and
/// publishes a value the nodes after it can read.
#[test]
fn Test_A_Join_Should_Settle_Over_The_Arms_It_Names()
{
    let definition = Published(Branching_Nodes());

    let record = Recorded(&definition, vec![Errored_Executor_Answer("check"), Errored_Executor_Answer("fix")], Parallelism::Of(Bound(2)));

    let dispositions = Dispositions(record.Produced());
    assert_eq!(dispositions.get(4), Some(&("done".to_owned(), NodeDisposition::Settled { produced: ProducedState::Flagged })));
}

/// Four independent nodes and a bound of two: the wave arrives as two groups, not one
/// oversized one. The bound is honored, not advisory.
#[test]
fn Test_No_Dispatch_Group_Should_Exceed_The_Bound_The_Caller_Stated()
{
    let definition = Published(Independent_Nodes());

    let record = Recorded(&definition, Four_Clean_Answers(), Parallelism::Of(Bound(2)));

    assert_eq!(Grouped_Names(record.Produced()), vec![vec!["a".to_owned(), "b".to_owned()], vec!["c".to_owned(), "d".to_owned()]]);
    assert!(record.Produced().groups.iter().all(|group| return group.nodes.len() <= 2), "a group held more nodes than the stated bound");
}

/// The same four nodes under a bound of one: four groups, one node each.
#[test]
fn Test_A_Bound_Of_One_Should_Put_Every_Independent_Node_In_Its_Own_Group()
{
    let definition = Published(Independent_Nodes());

    let record = Recorded(&definition, Four_Clean_Answers(), Parallelism::Of(Bound(1)));

    assert_eq!(record.Produced().groups.len(), 4);
}

/// A node that depends on another is in a later wave, so no bound however wide puts the
/// two in one group -- which is what makes a group's members genuinely independent rather
/// than merely adjacent.
#[test]
fn Test_A_Dependent_Node_Should_Never_Share_A_Group_With_What_It_Depends_On()
{
    let nodes = vec![
        Step_Node("a", &[], Body_For("a")),
        Step_Node("independent", &[], Body_For("independent")),
        Step_Node("after-a", &["a"], Body_For("after-a")),
    ];
    let definition = Published(nodes);

    let record = Recorded(&definition, Four_Clean_Answers(), Parallelism::Of(Bound(8)));

    assert_eq!(Grouped_Names(record.Produced()), vec![vec!["a".to_owned(), "independent".to_owned()], vec!["after-a".to_owned()]]);
}

/// Determinism, stated as the comparison it is: one definition, two visit orders, and
/// every part of the report identical.
///
/// Both runs are queued the same four answers, so which node received which answer cannot
/// itself be what differs -- what is under test is whether the *report* depends on the
/// order a group's members were visited in. A run that reported its nodes or its attempts
/// in visit order rather than declared order fails this comparison.
#[test]
fn Test_Two_Runs_Of_One_Definition_Should_Report_The_Same_Sequence_Under_Either_Visit_Order()
{
    let definition = Published(Independent_Nodes());
    let reversed = Parallelism { bound: Bound(2), visit: GroupVisitOrder::Reversed };

    let declared_run = Recorded(&definition, Four_Clean_Answers(), Parallelism::Of(Bound(2)));
    let reversed_run = Recorded(&definition, Four_Clean_Answers(), reversed);

    assert_eq!(declared_run.Produced(), reversed_run.Produced());
}

/// A cacheable node repeating an earlier node's whole body is served that node's result
/// and never dispatches -- and the queue is the proof: one answer is queued for two nodes,
/// so a second dispatch would find nothing left.
#[test]
fn Test_A_Cacheable_Node_Repeating_An_Earlier_One_Should_Be_Served_Rather_Than_Dispatched()
{
    let nodes = vec![Cacheable_Node("first", &[], Body_For("work")), Cacheable_Node("second", &["first"], Body_For("work"))];
    let definition = Published(nodes);

    let record = Recorded(&definition, vec![Clean_Executor_Answer("work")], Parallelism::Of(Bound(2)));

    let served = NodeDisposition::Served { from: "first".to_owned(), produced: ProducedState::Clean };
    let dispositions = Dispositions(record.Produced());
    assert_eq!(dispositions.get(1), Some(&("second".to_owned(), served)));
    assert_eq!(record.Produced().attempts.len(), 1, "only the first node dispatched");
}

/// The same two nodes declaring `NotCacheable` dispatch twice. The declaration is what
/// decides, not the repetition.
#[test]
fn Test_A_Node_Declaring_Not_Cacheable_Should_Dispatch_Again()
{
    let nodes = vec![Step_Node("first", &[], Body_For("work")), Step_Node("second", &["first"], Body_For("work"))];
    let definition = Published(nodes);

    let record = Recorded(&definition, vec![Clean_Executor_Answer("work"), Clean_Executor_Answer("work")], Parallelism::Of(Bound(2)));

    assert_eq!(record.Produced().attempts.len(), 2, "both nodes dispatched");
}

/// Four nodes that read none of each other, so all four fall into wave zero.
fn Independent_Nodes() -> Vec<WorkflowNode>
{
    return vec![
        Step_Node("a", &[], Body_For("work")),
        Step_Node("b", &[], Body_For("work")),
        Step_Node("c", &[], Body_For("work")),
        Step_Node("d", &[], Body_For("work")),
    ];
}

/// Four interchangeable answers, so which node received which cannot be what a comparison
/// between two runs turns on.
fn Four_Clean_Answers() -> Vec<PortAnswer>
{
    return vec![
        Clean_Executor_Answer("done"),
        Clean_Executor_Answer("done"),
        Clean_Executor_Answer("done"),
        Clean_Executor_Answer("done"),
    ];
}
