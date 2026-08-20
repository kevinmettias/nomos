//! The step exists, and it is one command.

use crate::workflow::{RULES_STEP, With_A_Scripted_Rules_Step, Workflow};

/// The gate still runs the rule layer.
///
/// The assertion that goes red if the step is deleted, renamed, rewritten as a script, or
/// pointed at some other command. Until `OD-GATE-004` there was nothing to assert: `nomos
/// check` existed, distinguished five outcomes by exit code, and nothing in CI ran it.
#[test]
fn Test_This_Repository_Gate_Should_Run_The_Rules()
{
    let argv = nomos_ledger::Derive_Step(&Workflow(), RULES_STEP)
        // Deleting or renaming the step is the most likely way the rule layer stops running,
        // and it arrives here as a refusal rather than as a wrong argv — so this is the site
        // that has to be loud. Note that the very next test takes the opposite side: it calls
        // `expect_err` because a refusal is the correct answer there. The two are the same
        // outcome read against different workflows, which is why neither may be softened into
        // a tolerant `unwrap_or_default`.
        .unwrap_or_else(|refusal| panic!("{}", refusal.Describe()));

    assert_eq!(argv.first().map(String::as_str), Some("cargo"));
    assert!(
        argv.iter().any(|argument| return argument == "gate")
            && argv.iter().any(|argument| return argument == "run"),
        "the Rules step must still run `gate run`, got {argv:?}"
    );
    assert!(
        argv.iter().any(|argument| return argument == "nomos-cli"),
        "the Rules step must still run this repository's own binary, got {argv:?}"
    );
}

/// The control. Without it, `Test_This_Repository_Gate_Should_Run_The_Rules` is satisfied by
/// any derivation that shrugs, and a reader cannot tell that it checks something.
///
/// A block-scalar `run:` is refused rather than guessed
/// (`GateUnknown::NotASingleCommand`), which is what keeps the whole workflow uniformly
/// derivable — and is why the step above may not be "improved" into a shell script that
/// branches on `$?`.
#[test]
fn Test_A_Scripted_Rules_Step_Should_Not_Satisfy_That_Assertion()
{
    let scripted = With_A_Scripted_Rules_Step(&Workflow());

    assert!(
        scripted.contains("run: |"),
        "the fixture rewrote nothing, so this control is about a workflow that does not exist"
    );

    let refusal = nomos_ledger::Derive_Step(&scripted, RULES_STEP)
        .expect_err("a scripted step must not yield an argv");

    assert!(
        matches!(refusal, nomos_ledger::GateUnknown::NotASingleCommand { .. }),
        "{refusal:?}"
    );
}

/// The step judges the whole workspace.
///
/// `--root .` is written out in the workflow even though it is the default, precisely so that
/// narrowing it to `--root crates/rules` is visible in a diff and can be asserted against.
/// Looking at less is the most likely future way this step is made green, and it is
/// `OD-GATE-001`'s defect in the form it would take here.
#[test]
fn Test_The_Rules_Step_Should_Judge_The_Whole_Workspace()
{
    let workflow = Workflow();
    // This test's claim is about one argument inside the argv, so it has nothing to say
    // about a workflow that yields no argv at all — that claim belongs to the first test in
    // this file. Stopping here keeps the two separable: a reader of a red run sees the
    // step-exists failure fire there and the scope failure fire here, instead of one
    // vacuous pass hiding behind the other.
    let argv = nomos_ledger::Derive_Step(&workflow, RULES_STEP)
        .unwrap_or_else(|refusal| panic!("{}", refusal.Describe()));
    let root_value = Root_Flag_Value(&argv);

    assert_eq!(root_value, Some("."), "the Rules step must judge the whole workspace, got {argv:?}");
    assert!(
        !workflow.contains("working-directory:"),
        "a working directory would move the tree the step judges without changing its argv"
    );
}

/// The value following `--root` in `argv`.
///
/// No index can stand in for a missing flag. Defaulting to zero would make the caller's
/// assertion read `argv[1]` — the word `check` — and report the step as judging a tree it
/// never named, which is the `OD-GATE-001` defect wearing this test's badge.
fn Root_Flag_Value(argv: &[String]) -> Option<&str>
{
    let root = argv
        .iter()
        .position(|argument| return argument == "--root")
        .unwrap_or_else(|| panic!("the Rules step must name the tree it judges: {argv:?}"));

    return argv.get(root.saturating_add(1)).map(String::as_str);
}
