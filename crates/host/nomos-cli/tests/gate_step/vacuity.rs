//! The workflow was read at all.

use crate::common::Workflow;

/// The vacuity guard for this suite.
///
/// Every assertion beside it reads the workflow as text, and an empty string contains no
/// `continue-on-error`. A missing file panics in `Workflow()` and a renamed step fails
/// `Derive_Step`, but a *truncated* workflow would satisfy the excuse check while the job it
/// describes had lost most of its steps. This is the repository's own pattern —
/// `Test_The_Workspace_Should_Not_Appear_Empty` in `boundaries.rs`, and the non-empty
/// assertion at the head of `Test_Every_Canonical_Record_On_Disk_Should_Be_Governing`.
#[test]
fn Test_The_Workflow_Should_Not_Appear_Empty()
{
    let workflow = Workflow();

    let steps = workflow
        .lines()
        .filter(|line| return line.trim().starts_with("- name:"))
        .count();

    assert!(
        steps >= 5,
        "{steps} named step(s) in the gate workflow. Every assertion in this file is a \
         statement about a file that has been truncated or replaced"
    );
}
