//! `nomos workflow` — a thin renderer over `nomos-workflow-orchestration`'s
//! [`nomos_workflow_orchestration::Run`], the same "walk, choose a platform, render" shape
//! `gate.rs` and `correct.rs` already are over their own orchestration seams.
//!
//! # What this increment runs, and what it does not
//!
//! `nomos_workflow_orchestration::Run` already takes a whole `&[WorkflowStepPlan]` -- real
//! multi-step sequencing is that crate's own job, not this one's. What is missing is a way
//! to *reach* it from a shell at all (`P40-WORKFLOW-CLI-VERB`'s own why: "the ordinary
//! evidence that would tell us what workflows people actually compose is not being
//! collected"), and this command is exactly that reach, kept to the single-step shape a
//! `nomos_contracts::WorkflowStep` declared over argv can honestly build today: one body,
//! chosen by `--check`, `--executor` or `--model-backend`, wrapped in a fixed, always-
//! coherent declaration. A caller composing more than one step links
//! `nomos-workflow-orchestration` directly -- the same "no invented shape ahead of a real
//! body" restraint `gate.rs`'s own doc names for `compare`, applied here to a multi-step
//! plan format nothing has asked this binary to parse yet.
//!
//! `--check`'s own `CheckOutcome::Judged` renders `Ok` whatever its findings are: this verb
//! reports whether the step *ran*, not whether it passed the gate `nomos gate run` already
//! answers that question for. A caller wanting a pass/fail reading of a check step's
//! findings runs `nomos gate run` instead; folding that judgment in here would make this
//! verb a second place a check's disposition is decided; `nomos-check-orchestration::
//! CheckOutcome` and `nomos-gate-orchestration`'s own disposition already are that once.
//!
//! # This module is now four responsibilities behind one verb
//!
//! The file was 920 lines and one flat list; the seams were already there and only their
//! names were missing. [`parsing`] reads argv into a [`WorkflowCommand`] and nothing else,
//! [`sources`] is the one tree walk every body defers to, [`composition`] supplies the two
//! values only a binary can (`env!` resolves against the crate that calls it), and
//! [`report`] renders one step's own typed outcome. What stays here is the dispatch between
//! them: [`Run`] chooses a body, walks it through the one composition-root guard each body
//! owes, runs the step, and hands the outcome to the renderer. [`exit_code`] is the fifth,
//! because an exit code means one thing per binary and this group's five numbers are read by
//! every other group on it.

mod composition;
mod exit_code;
mod parsing;
mod report;
mod sources;
#[cfg(test)]
mod tests;

pub use exit_code::ExitCode;
pub use parsing::Command_From_String_Arguments;

use self::composition::{Coherent_Declaration, Workflow_Variant};
use self::report::{Rendered_Check, Rendered_Workflow_Outcome};
use self::sources::{Walked_Check, Walked_Correction, Walked_Gate};
use nomos_composer_std::{CLOCK, ENVIRONMENT, FILE_SYSTEM, LAUNCHER};
use nomos_platform::Clock;
use nomos_workflow_orchestration::{Body, CheckBody, Platform, WorkflowOutcome, WorkflowStepPlan};
use std::io::Write;
use std::path::Path;

/// What `nomos workflow run` was asked to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkflowCommand
{
    body: Body,
}

/// Runs `command`'s one step and renders what came back.
pub fn Run(command: &WorkflowCommand, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    let body = match Walked_Command(command, stdout, stderr)
    {
        Ok(body) => body,
        Err(code) => return code,
    };

    let outcome = Ran_Step(body);

    return Rendered_Workflow_Outcome(&outcome, stdout, stderr);
}

/// `command`'s own body, with its `root` already walked. `Err` names the code the process leaves
/// with when the walk itself decided the answer, so the caller runs no step over a tree that was
/// never read -- an unreadable root, or a check walk that found no source at all.
///
/// The body this cannot reach is cloned through unchanged: `Run` is generic over every body the
/// type has, not only the three that walk.
fn Walked_Command(command: &WorkflowCommand, stdout: &mut impl Write, stderr: &mut impl Write) -> Result<Body, ExitCode>
{
    return match &command.body
    {
        Body::Check(check) => Walked_Check_Body(check, stdout, stderr),
        Body::Correction(correction) => match Walked_Correction(correction)
        {
            Some(walked) => Ok(Body::Correction(walked)),
            None => Err(Not_A_Directory(&correction.root, stderr)),
        },
        Body::Gate(gate) => match Walked_Gate(gate)
        {
            Some(walked) => Ok(Body::Gate(walked)),
            None => Err(Not_A_Directory(&gate.command.root, stderr)),
        },
        other => Ok(other.clone()),
    };
}

/// `check`, with its own `sources` replaced by a real walk of its `root`, and the empty-walk
/// vacuity guard applied -- the composition root's own decision, made before `nomos_check_
/// orchestration::Run` is ever called, because that crate's own doc names an empty `sources` as
/// never its own case to classify. `check.rs`'s identical guard is what this mirrors.
fn Walked_Check_Body(check: &CheckBody, stdout: &mut impl Write, stderr: &mut impl Write) -> Result<Body, ExitCode>
{
    let Some(walked) = Walked_Check(check)
    else
    {
        return Err(Not_A_Directory(&check.root, stderr));
    };

    if walked.sources.is_empty()
    {
        return Err(Rendered_Check(&nomos_check_orchestration::CheckOutcome::NoSource, stdout, stderr));
    }

    return Ok(Body::Check(walked));
}

/// The one step this command composed, run through the seam this module is a thin renderer over.
///
/// The declaration, the platform and the run id are read here rather than in `Run` because they
/// are the environment this binary supplies, and the orchestration crate takes all three as
/// values for exactly that reason.
fn Ran_Step(body: Body) -> WorkflowOutcome
{
    use nomos_gate_orchestration::Fresh_Run_Id;

    let plan = [WorkflowStepPlan { declaration: Coherent_Declaration(), body }];
    let declared = crate::agent::Shipped_Targets();
    let platform = Platform { launcher: &LAUNCHER, filesystem: &FILE_SYSTEM, environment: &ENVIRONMENT, now: CLOCK.Now(), declared: &declared };
    let run = Fresh_Run_Id(CLOCK.Now());

    return nomos_workflow_orchestration::Run(&plan, &platform, &Workflow_Variant(), run);
}

/// Reports that a body's own root is not a directory, and the code that earns.
///
/// All three walking bodies make this identical report, so it sits in the trailing region rather
/// than beside any one of them -- the call-order rule exempts a helper more than one function
/// reaches, and three writers of the same sentence is exactly why.
fn Not_A_Directory(root: &Path, stderr: &mut impl Write) -> ExitCode
{
    let _ = writeln!(stderr, "`{}` is not a directory", root.display());

    return ExitCode::Unavailable;
}
