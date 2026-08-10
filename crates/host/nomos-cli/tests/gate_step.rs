//! The gate runs the rule layer, and nothing here can excuse it from failing.
//!
//! `OD-GATE-004` appended a `Rules` step to `.github/workflows/gate.yml` running
//! `cargo run --quiet -p nomos-cli --bin nomos -- check --root .`. Everything that decides
//! whether that step *judges anything* lives in three places, and only one of them is code:
//! the step's presence, the tree it is pointed at, and whether it is allowed to fail.
//! `check_command.rs` beside this file asserts what the command does; this file asserts that
//! CI still runs it, over the whole workspace, with its exit code intact.
//!
//! # Why these are structural assertions and not a workflow run
//!
//! There is no runner here, so nothing in this repository can observe GitHub Actions failing
//! a step on a non-zero exit — and a test of that would be a test of a fixture rather than of
//! the platform's contract. What is checkable is everything on this side of it: that the step
//! exists and is derivable, that its argv still names `check` and still judges `.`, that no
//! step in the file carries an excuse, and that the workflow was read at all. The exit-code
//! policy itself is one assertion in `check.rs`,
//! `Test_Only_Ok_Should_Carry_The_Passing_Exit_Code`.
//!
//! The step is derived rather than grepped for, following
//! `crates/substrate/nomos-ledger/tests/gate_covers_finish.rs`. Deriving it means a step
//! rewritten as a shell script fails these assertions instead of satisfying them by
//! containing the right words.

use std::path::{Path, PathBuf};

/// The step `OD-GATE-004` added. Deliberately not `Lint`, and
/// `Test_The_Derived_Lint_Step_Should_Still_Be_Clippy` is why.
const RULES_STEP: &str = "Rules";

/// The three ways a step is told to run and not be believed.
///
/// `continue-on-error:` is the platform's own; `|| true` and a trailing `exit 0` are the
/// shell's. Any of them turns this into "a step that runs the command and ignores its exit
/// code", which `P10-CHECK-GATE`'s `done_when` names as not closing the item.
const EXCUSES: [&str; 3] = ["continue-on-error", "|| true", "exit 0"];

/// This repository's root, from this crate's manifest directory.
fn Repository_Root() -> PathBuf
{
    return Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
}

/// The real gate workflow, read from disk through the ledger's own path function.
///
/// `Workflow_Path` rather than a string joined here, so that if `work finish` ever starts
/// looking somewhere else these assertions follow it instead of quietly checking a file
/// nothing reads.
fn Workflow() -> String
{
    let path = nomos_ledger::Workflow_Path(&Repository_Root());

    return std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("this repository has a gate workflow at {}: {error}", path.display()));
}

/// Which excuses a workflow carries, comments not counted.
///
/// Extracted so that the control below exercises **this** predicate rather than a second
/// implementation of it written beside the first — the arrangement
/// `governing_records_are_present.rs` uses for `Disagreements`.
///
/// Comment lines are dropped because the predicate is about what the workflow *does*. The
/// `Rules` step's own comment explains why it carries no branch and no excuse, and a guard
/// that fired on prose would make explaining a decision impossible in the file the decision
/// is in.
fn Excuses_In(workflow: &str) -> Vec<String>
{
    let executable = workflow
        .lines()
        .filter(|line| return !line.trim_start().starts_with('#'))
        .collect::<Vec<&str>>()
        .join("\n");

    return EXCUSES
        .iter()
        .filter(|excuse| return executable.contains(**excuse))
        .map(|excuse| return (*excuse).to_owned())
        .collect();
}

/// The same workflow with the `Rules` step's `run:` turned into a block scalar.
///
/// Written by walking the step rather than by replacing a literal command, so it keeps
/// working when the argv changes and cannot silently rewrite nothing.
fn With_A_Scripted_Rules_Step(workflow: &str) -> String
{
    let mut rewritten = String::new();
    let mut inside = false;

    for line in workflow.lines()
    {
        let trimmed = line.trim();

        if let Some(name) = trimmed.strip_prefix("- name:")
        {
            inside = name.trim() == RULES_STEP;
        }
        else if inside && let Some(run) = trimmed.strip_prefix("run:")
        {
            rewritten.push_str("        run: |\n          ");
            rewritten.push_str(run.trim());
            rewritten.push('\n');
            inside = false;
            continue;
        }

        rewritten.push_str(line);
        rewritten.push('\n');
    }

    return rewritten;
}

/// The same workflow with the `Rules` step excused from failing.
fn With_An_Excused_Rules_Step(workflow: &str) -> String
{
    return workflow.replace(
        "      - name: Rules\n",
        "      - name: Rules\n        continue-on-error: true\n",
    );
}

// ---------------------------------------------------------------------------
// The step exists, and it is one command.
// ---------------------------------------------------------------------------

/// The gate still runs the rule layer.
///
/// The assertion that goes red if the step is deleted, renamed, rewritten as a script, or
/// pointed at some other command. Until `OD-GATE-004` there was nothing to assert: `nomos
/// check` existed, distinguished five outcomes by exit code, and nothing in CI ran it.
#[test]
fn Test_This_Repository_Gate_Should_Run_The_Rules()
{
    let argv = nomos_ledger::Derive_Step(&Workflow(), RULES_STEP)
        .unwrap_or_else(|refusal| panic!("{}", refusal.Describe()));

    assert_eq!(argv.first().map(String::as_str), Some("cargo"));
    assert!(
        argv.iter().any(|argument| return argument == "check"),
        "the Rules step must still run the check group, got {argv:?}"
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
    let argv = nomos_ledger::Derive_Step(&workflow, RULES_STEP)
        .unwrap_or_else(|refusal| panic!("{}", refusal.Describe()));

    let root = argv
        .iter()
        .position(|argument| return argument == "--root")
        .unwrap_or_else(|| panic!("the Rules step must name the tree it judges: {argv:?}"));

    assert_eq!(
        argv.get(root.saturating_add(1)).map(String::as_str),
        Some("."),
        "the Rules step must judge the whole workspace, got {argv:?}"
    );
    assert!(
        !workflow.contains("working-directory:"),
        "a working directory would move the tree the step judges without changing its argv"
    );
}

// ---------------------------------------------------------------------------
// Editing this workflow did not change what `work finish` runs.
// ---------------------------------------------------------------------------

/// The lint step `work finish` derives is still clippy, after a step was added beside it.
///
/// `OD-LEDGER-003` decided that finishing derives its lint argv from the workflow rather than
/// copying it, and `Derive_Step` finds that argv **by the step's name**. So a step in this
/// file named `Lint` would make every `work finish` run whatever that step runs — here, `cargo
/// run … check --root .` — and clippy would stop running before every item's predicate with
/// nothing saying so. That is why the step `OD-GATE-004` added is named `Rules`.
///
/// # This is a near-duplicate of `gate_covers_finish.rs`'s
/// `Test_This_Repository_Gate_Should_Still_Yield_A_Lint_Step`, and both are kept
///
/// Coverage at two altitudes rather than two answers to one question, and a later reader
/// should not delete either as a duplicate. They fail for different authors:
///
/// - the one in `crates/substrate/nomos-ledger/tests/gate_covers_finish.rs` (around `:439`)
///   is the older, sits in the crate that owns `Derive_Step`, and fires when *the workflow*
///   rots under the derivation — a renamed or scripted `Lint` step;
/// - this one fires when *an author editing this workflow to add a step* reaches for the wrong
///   name. It lives in the crate whose test suite that author is already running, and it is
///   the one their own item's predicate executes.
///
/// The failure they guard is silent in both directions, which is what justifies paying for it
/// twice. If one home is ever preferred, the ledger's is the older and this is the one to
/// drop.
#[test]
fn Test_The_Derived_Lint_Step_Should_Still_Be_Clippy()
{
    let argv = nomos_ledger::Derive_Step(&Workflow(), nomos_ledger::LINT_STEP)
        .unwrap_or_else(|refusal| panic!("{}", refusal.Describe()));

    assert!(
        argv.iter().any(|argument| return argument == "clippy"),
        "`work finish` derives its lint step by name, so a step named `Lint` that is not \
         clippy silently replaces linting: {argv:?}"
    );
    assert!(
        argv.iter().any(|argument| return argument == "-D"),
        "the lint step must still deny warnings, got {argv:?}"
    );
}

/// Exactly one step in this file is named `Lint`, and the count is the assertion.
///
/// The guard above is weaker than it looks, and this was measured rather than assumed.
/// `Derive_Step` scans lines and returns at the **first** `run:` inside the first step whose
/// name matches, so a second step named `Lint` behaves in two opposite ways depending only on
/// where it sits: above the real one it silently becomes what `work finish` runs, and below it
/// it is invisible to the derivation. Measured by renaming the `Rules` step to `Lint` in a copy
/// of this repository's workflow — the step sits after `Lint`, so
/// `Test_The_Derived_Lint_Step_Should_Still_Be_Clippy` stayed **green**, and the assertions
/// that fired were the ones about the `Rules` step having gone missing.
///
/// So the name collision is caught here, by counting, where position cannot matter. A gate
/// with two steps named `Lint` is wrong whichever order they are in: one of them is not being
/// derived and nothing says which.
#[test]
fn Test_Only_One_Step_Should_Be_Named_Lint()
{
    let lints = Workflow()
        .lines()
        .filter(|line| {
            return line
                .trim()
                .strip_prefix("- name:")
                .is_some_and(|name| return name.trim() == nomos_ledger::LINT_STEP);
        })
        .count();

    assert_eq!(
        lints, 1,
        "{lints} step(s) in the gate are named `{}`. `work finish` derives its lint argv from \
         the first one and says nothing about the rest, so a second one either replaces \
         clippy or is never derived depending on where it was pasted",
        nomos_ledger::LINT_STEP
    );
}

/// And the two steps are two steps.
///
/// Red on the pair collapsing into one, or on a copy-paste that leaves both running the same
/// command — which would give the gate the appearance of a rule step while running clippy
/// twice.
#[test]
fn Test_The_Rules_Step_Should_Not_Be_The_Lint_Step()
{
    let workflow = Workflow();

    assert_ne!(
        nomos_ledger::Derive_Step(&workflow, RULES_STEP),
        nomos_ledger::Derive_Step(&workflow, nomos_ledger::LINT_STEP),
        "the gate's rule step and its lint step run the same command"
    );
}

// ---------------------------------------------------------------------------
// No step is excused from failing.
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// The workflow was read at all.
// ---------------------------------------------------------------------------

/// The vacuity guard for this file.
///
/// Every assertion above reads the workflow as text, and an empty string contains no
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
