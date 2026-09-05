//! `nomos check` — run the rules over a tree and report what they find.
//!
//! The composition root and renderer for `nomos-check-orchestration`. That crate composes
//! the capability registry, ingests already-walked source into facts and runs the
//! completeness rule over them; it cannot read a disk and cannot pick a build variant, so
//! this module is where the tree is walked, what this binary was compiled as is read, and
//! the typed [`nomos_check_orchestration::CheckOutcome`] it hands back is turned into text
//! and an [`ExitCode`] — the seam `OD-HOST-002` asked for, the same shape `OD-HOST-001`
//! already built for `nomos work`.
//!
//! # What a composition root still has to assemble
//!
//! Three things now, since composing the registry, ingesting the walk and judging it live
//! in `nomos-check-orchestration`:
//!
//! 1. walk the tree — [`sources::Walked_Sources`] — because no [`nomos_platform::FileSystem`]
//!    directory-listing port exists, the same exception `nomos-cli::work::
//!    Published_Records` already has;
//! 2. read what this binary was compiled as — [`composition::Host_Variant`] — because
//!    `env!` resolves against the crate that calls it and cannot be read correctly from
//!    inside the orchestration crate;
//! 3. choose a [`nomos_platform::ProcessLauncher`] — [`nomos_platform_std::StdProcessLauncher`]
//!    — for the one provider in that composition that runs a process, the same choice
//!    `work.rs` already makes for `nomos work`.
//!
//! All three are handed to [`nomos_check_orchestration::Run`] as values, the same way
//! `nomos_work_orchestration::Run` takes `published` as a value rather than deriving it.
//!
//! # The vacuity guard lives here, and it now has two shapes
//!
//! A rule that finds no subjects returns no findings, which renders identically to a
//! clean run. That is the defect the sibling workspace recorded four times over —
//! `check-standards-tree /nonexistent` walked nothing, found nothing and reported CLEAN —
//! and `boundaries.rs` already guards its own assertions against it. The guard belongs
//! here rather than in the rule, because "did I see a plausible amount of the world" is a
//! question only the caller that chose the tree can answer.
//!
//! The second shape arrived with the fact layer: a walk that found source and materialized
//! no fact for any of it. That is the same lie one layer in — the rule would resolve every
//! mirror claim against an empty index and, because the index is empty, would refuse to
//! block on any of them. Both cases are now typed
//! ([`nomos_check_orchestration::CheckOutcome::NoSource`] and
//! [`nomos_check_orchestration::CheckOutcome::NoFacts`]) and both render
//! [`ExitCode::Vacuous`], which already means "the answer is empty because something
//! expected was not there". No seventh code: an exit code means one thing per binary, and
//! this is that thing.
//!
//! Where the vacuity guard *belongs* was a live question; `OD-GATE-003` answers it. The
//! guard stays here, decided by this caller, for the reason given above — but `vacuity.rs`
//! is now the one place a reader finds that this module and `spec.rs` both made that
//! decision, rather than discovering it twice by reading two doc comments.
//!
//! # This module's exit-code design underlies a gate step now, and that changes what a code costs
//!
//! `.github/workflows/gate.yml`'s `Rules` step no longer invokes this command directly.
//! Since the increment that gave `gate` a real `run` verb, that step invokes
//! `cargo run --quiet -p nomos-cli --bin nomos -- gate run --root .`, and it is
//! [`crate::gate::ExitCode`] — not this module's own [`ExitCode`] — that CI's build now
//! observes. `gate/run.rs` walks and judges the tree through the very
//! [`nomos_check_orchestration::Run`] call this module makes, and reduces the same
//! [`CheckOutcome`] this module renders; [`crate::gate::ExitCode`] deliberately carries
//! forward the zero-is-the-only-success meanings this module gave those numbers first. So
//! the per-variant narration below documents the design those meanings originated from, and
//! that `gate::ExitCode` now enforces directly against the `Rules` step — not a first-hand
//! claim about this module's own role in CI, which it no longer has.
//!
//! **Zero is the only success, and the workflow says so by containing no branch.** Actions
//! fails a step on any non-zero exit, and that default *is* the policy — so the numbers
//! below stay spelled here, once, rather than being restated in YAML where they would go
//! stale against this enum. What each code originally means, and what a build now reads off
//! `gate::ExitCode` in its place:
//!
//! - [`ExitCode::Ok`] — the rules ran over the workspace, materialized facts, and nothing
//!   they found can fail a build. Not "found nothing": twelve admitted gaps and the two
//!   `tests/corpus/analysis/gamma/broken.rs` advisories print on every run, and the counts
//!   line carries both denominators. **Green.**
//! - [`ExitCode::Violations`] — the arm the step exists for. Reachable only since
//!   `OD-RULES-002` made incompleteness a property of the claim rather than of the run;
//!   before that a phantom anywhere in this workspace was downgraded by `broken.rs` and the
//!   step could not have failed for its own reason. **Red.**
//! - [`ExitCode::Usage`] — from a step, this means *the workflow's own argv is wrong*. A
//!   gate that mistypes its invocation and passes is a gate checking nothing. **Red.**
//! - [`ExitCode::Unreadable`] — no tree, so nothing was checked. **Red.**
//! - [`ExitCode::Vacuous`] — the one this wiring is really about. A gate treating "I judged
//!   nothing" as success would be `OD-GATE-001`'s defect installed one level up from where
//!   that record found it, this time with a green tick beside it. **Red.**
//! - anything else — `101` from a build failure or from [`nomos_check_orchestration::
//!   Registered`]'s own composition failing without ever reaching this module, a signal, a
//!   truncation. Absence, unknown and error must not become success, and the default gives
//!   that for free. **Red.**
//!
//! Two consequences for anybody editing this module, and both still true regardless of
//! which binary invocation CI runs, because `gate/run.rs` reduces off the same
//! [`CheckOutcome`] this module produces rather than owning a second judgment of its own. A
//! sixth code must survive `main`'s `u8::try_from(code).unwrap_or(1)`, or a distinct outcome
//! arrives at CI as an ordinary violation. And [`ExitCode::Vacuous`] must never be
//! renumbered to `0` "because there is nothing to report" —
//! `Test_Only_Ok_Should_Carry_The_Passing_Exit_Code` below is the whole exit-code policy as
//! one assertion, and it is where that would go red.
//!
//! No corpus is involved. This command reads no environment variable at run time —
//! `main.rs` passes it none — so a CI runner with no `NOMOS_*` set produces the full
//! answer. Measured. What it does need is what `build.rs` baked in for
//! [`composition::Host_Variant`], and a runner that cannot supply those cannot link the
//! binary and fails at `Lint` long before this step.

mod parsing;
mod sources;
mod composition;
mod report;
#[cfg(test)]
mod tests;

pub use parsing::Check_Command_From_String_Arguments;

mod exit_code;

pub(crate) use exit_code::ExitCode;
pub(crate) use nomos_check_orchestration::CheckCommand;

use crate::arguments::Named_Value_From_String_Arguments;
use nomos_contracts::Finding;
use nomos_rules::SourceFile;
use nomos_model::Subject_Of_Path;
use nomos_workspace::BuildVariant;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Runs the rules and renders what they say.
pub fn Run(command: &CheckCommand, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    use report::Render_Outcome;

    let outcome = Outcome_For(&command.root);

    return Render_Outcome(&command.root, &outcome, stdout, stderr);
}

/// Walks `root` and judges what it finds, or reports why nothing was judged -- the walk and
/// the vacuity guard, factored out of [`Run`] so that function's own body reads as one line
/// per section rather than the two decisions run together.
fn Outcome_For(root: &Path) -> nomos_check_orchestration::CheckOutcome
{
    use sources::Walked_Sources;
    use nomos_check_orchestration::CheckOutcome;

    return match Walked_Sources(root)
    {
        None => CheckOutcome::Unreadable,
        Some(sources) if sources.is_empty() => CheckOutcome::NoSource,
        Some(sources) => Judged_Sources(root, &sources),
    };
}

/// Composes the registry this run needs and runs the rules over `sources`, already walked
/// from `root`.
fn Judged_Sources(root: &Path, sources: &[SourceFile]) -> nomos_check_orchestration::CheckOutcome
{
    use composition::Host_Variant;
    use nomos_platform_std::{StdFileSystem, StdProcessLauncher};

    return nomos_check_orchestration::Run(
        sources,
        nomos_check_orchestration::RunContext {
            variant: Host_Variant(),
            root,
            launcher: &StdProcessLauncher,
            filesystem: &StdFileSystem,
            workspace: &mut None,
            store: &mut nomos_analysis::MemoryFactStore::New(),
        },
        &[],
    );
}

#[cfg(test)]
mod run_coverage
{
    use super::*;
    use nomos_check_orchestration::CheckCommand;

    /// `Run` driven end to end over a root that does not exist, so `Walked_Sources` fails
    /// before any rule ever runs — the same safe, fast path `check/tests.rs`'s own broader
    /// `Run` coverage uses.
    #[test]
    fn Test_Run_Should_Report_Unreadable_For_A_Root_That_Does_Not_Exist()
    {
        let command = CheckCommand { root: PathBuf::from("no-such-tree-anywhere-for-run-coverage-test") };
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let code = Run(&command, &mut stdout, &mut stderr);

        assert_eq!(code, ExitCode::Unreadable);
    }
}
