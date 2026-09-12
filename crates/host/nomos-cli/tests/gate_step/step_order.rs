//! A tool the tests launch is on the runner before the step that launches them.
//!
//! # Why this is not the assertion it looks like it should be
//!
//! The obvious guard is "the install precedes every step whose command names the tool", and
//! it is vacuous here. Measured 2026-09-12: exactly one step of this workflow textually runs
//! `cargo deny` -- `Supply chain` -- and it already follows the install, so that assertion
//! passes today for a reason that has nothing to do with the defect. It would also have
//! passed on the broken workflow. The step that actually needed the binary was `Test`, whose
//! command is `cargo test --workspace` and never names the tool: `nomos-lang-rust-deny`'s
//! provider launches it from inside a test, and no reading of the workflow can see that.
//!
//! So the invariant here is about test execution rather than about naming. A step that runs
//! this workspace's tests may launch any tool this workspace knows how to launch, so every
//! install has to come first. `Rules` is downstream of the same install for the same reason
//! -- `nomos gate run` materializes the dependency-policy capability too -- and is already
//! ordered after it.
//!
//! # What this is worth
//!
//! `P82` moved the install and wrote a long comment at the step saying why it must stay
//! there. A comment is what failed the first time: the step sat after `Test` for the entire
//! life of this workflow, the provider reported a missing `cargo deny` as a *clean*
//! bans/licenses/sources verdict, and what finally surfaced it was `P81` making that provider
//! refuse -- unrelated work that happened to look. Nothing else in this suite asserts order.

use crate::workflow::{Installs_After_The_First_Test_Run, With_Tool_Installs_After_The_Tests, Workflow};

/// Every tool this gate installs is installed before the tests that may launch it.
#[test]
fn Test_Every_Tool_Should_Be_Installed_Before_The_Step_That_Runs_The_Tests()
{
    let late = Installs_After_The_First_Test_Run(&Workflow());

    assert!(
        late.is_empty(),
        "{late:?} install a tool after a step that runs this workspace's tests. A test that \
         launches a tool the runner does not have yet does not fail loudly -- it reports \
         whatever its provider reports for an absent command, which is how a clean \
         bans/licenses/sources verdict came to stand for a judgment nobody made"
    );
}

/// The control that shows the assertion above can fire.
///
/// It is the real historical shape rather than an invented one: the install ordered after
/// the step whose tests launch the tool is exactly where `gate.yml` had it until `P82`.
/// Without this, a workflow that stopped installing anything would satisfy the assertion
/// above, and a string-absence assertion never shown to fail is indistinguishable from a
/// comment -- the reasoning `Test_The_Excuse_Check_Should_Reject_An_Excused_Step` already
/// makes for its own subject.
///
/// It exercises the same predicate rather than a second implementation of it, which is the
/// arrangement `Excuses_In` documents for the same reason: two copies are how two guards
/// come to disagree.
#[test]
fn Test_The_Ordering_Check_Should_Reject_An_Install_Placed_After_The_Tests()
{
    let doctored = With_Tool_Installs_After_The_Tests(&Workflow());

    let late = Installs_After_The_First_Test_Run(&doctored);

    assert!(
        !late.is_empty(),
        "the doctored workflow puts every `cargo install` after the test step, so the check \
         must report it; reporting nothing means the check cannot see the shape it exists for"
    );
}

/// The doctoring moved a real step rather than rewriting nothing.
///
/// `With_Tool_Installs_After_The_Tests` walks the steps instead of replacing a literal, so
/// it survives the install's argv changing -- but a walk that matched no step would hand the
/// control above an unchanged workflow and make it fail for the wrong reason.
#[test]
fn Test_The_Doctored_Workflow_Should_Differ_From_The_Real_One()
{
    let real = Workflow();

    let doctored = With_Tool_Installs_After_The_Tests(&real);

    assert_ne!(doctored, real, "the doctoring matched no install step, so the control proves nothing");
}
