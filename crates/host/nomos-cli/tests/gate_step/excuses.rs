//! No step is excused from failing.

use crate::workflow::{Excuses_In, With_An_Excused_Rules_Step, Workflow};

/// A red gate becomes a decorative one one `continue-on-error` at a time.
///
/// Asserted over every step rather than only the new one: the exit code this item made a CI
/// outcome is worth exactly as much as the job's willingness to fail on it, and an excuse
/// anywhere in the file is the shape `done_when` forbids.
#[test]
fn Test_No_Step_In_The_Gate_Should_Excuse_Itself()
{
    let excuses = Excuses_In(&Workflow());

    assert!(
        excuses.is_empty(),
        "the gate excuses a step from failing via {excuses:?}. A step that runs a command and \
         ignores its exit code is a step that checks nothing"
    );
}

/// The control that shows the assertion above can fire.
///
/// A string-absence assertion that has never been shown to fail is indistinguishable from a
/// comment.
#[test]
fn Test_The_Excuse_Check_Should_Reject_An_Excused_Step()
{
    let excused = With_An_Excused_Rules_Step(&Workflow());

    assert!(
        excused.contains("continue-on-error: true"),
        "the fixture excused nothing, so this control is about a workflow that does not exist"
    );
    assert_eq!(Excuses_In(&excused), vec!["continue-on-error".to_owned()]);
}
