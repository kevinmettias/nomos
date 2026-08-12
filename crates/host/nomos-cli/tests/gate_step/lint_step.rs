//! Editing this workflow did not change what `work finish` runs.

use crate::common::{RULES_STEP, Workflow};

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
/// - the one in `crates/substrate/nomos-ledger/tests/gate_covers_finish/repository_gate.rs`
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
