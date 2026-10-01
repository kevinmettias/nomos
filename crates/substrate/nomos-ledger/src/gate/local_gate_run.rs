//! Executing the derived step set on this host, and reporting what it did not execute.

use std::path::Path;

use nomos_platform::{Command, ExitOutcome, ProgramLauncher};

use super::{DerivedStep, GateUnknown, StepExecution};

/// How long a single gate step may run before it is abandoned.
///
/// A real gate run on this workspace takes five to twenty-five minutes end to end, and the
/// longest single step is the workspace test. Forty-five minutes is past that with room,
/// and it is a bound rather than an expectation: a step that reaches it is reported through
/// the launcher's own outcome, not silently.
const STEP_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(45 * 60);

/// What one local execution of the whole step set produced.
///
/// Holds every step the workflow declares, in file order, whether or not this host ran it.
/// That is the point rather than an implementation detail: `OD-GATE-033` requires a local
/// run to report the set it did not execute, for the same reason the `Corpus gates` step
/// exists -- an absence a passing run swallows is invisible.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalGateRun
{
    steps: Vec<(String, StepExecution)>,
}

impl LocalGateRun
{
    /// Every declared step and what this execution gave it, in the workflow's own order.
    #[must_use]
    pub fn Steps(&self) -> &[(String, StepExecution)]
    {
        return &self.steps;
    }

    /// The names of the steps this execution did not run, in the workflow's own order.
    ///
    /// Both non-executed values are here together, because from the caller's side they
    /// share the one property that matters: this run is not evidence about them. Which of
    /// the two a step got, and why, is in [`Self::Steps`].
    #[must_use]
    pub fn Unexecuted(&self) -> Vec<&str>
    {
        return self
            .steps
            .iter()
            .filter(|(_, execution)| return !matches!(execution, StepExecution::Executed { .. }))
            .map(|(name, _)| return name.as_str())
            .collect();
    }
}

/// Runs every step the workflow declares that `host` admits, and reports all of them.
///
/// The steps that do not run are in the answer beside the ones that do. A caller cannot get
/// a clean result over a subset from this function, because it never returns only the
/// subset.
pub fn Run_Gate_Locally(
    steps: &[DerivedStep],
    host: &str,
    launcher: &impl ProgramLauncher,
    working_directory: Option<&Path>,
) -> LocalGateRun
{
    let mut executed = Vec::new();

    for step in steps
    {
        let execution = Execution_Of(step, host, launcher, working_directory);
        executed.push((step.name.clone(), execution));
    }

    return LocalGateRun { steps: executed };
}

/// One step's value, decided in the order `OD-GATE-033` decides it: a guard outside the
/// subset refuses before anything is run, an excluded host is unavailable without the argv
/// being consulted, and only an admitted step with a derivable command is executed.
fn Execution_Of(
    step: &DerivedStep,
    host: &str,
    launcher: &impl ProgramLauncher,
    working_directory: Option<&Path>,
) -> StepExecution
{
    if let super::StepGuard::Outside(guard) = &step.guard
    {
        return StepExecution::Refused(GateUnknown::GuardOutsideSubset {
            step: step.name.clone(),
            guard: guard.clone(),
        });
    }

    if !step.guard.Admits(host)
    {
        return StepExecution::Unavailable { host: host.to_owned() };
    }

    let argv = match &step.argv
    {
        Ok(argv) => argv.clone(),
        Err(cause) => return StepExecution::Refused(cause.clone()),
    };

    return Exit_Of(launcher, argv, working_directory);
}

/// Runs one admitted step and reports the code it exited with.
///
/// A launcher that could not start the process at all, and a process that did not exit on
/// its own, are both reported as a refusal rather than as an exit code, because neither is
/// an answer about the step's subject. `ExitOutcome` already separates the two from an
/// exit, and this preserves that separation instead of folding them into a number.
fn Exit_Of(launcher: &impl ProgramLauncher, argv: Vec<String>, working_directory: Option<&Path>) -> StepExecution
{
    let step = argv.join(" ");
    let mut command = Command::From_String_Arguments(argv, STEP_TIMEOUT);
    command.working_directory = working_directory.map(Path::to_path_buf);

    return match launcher.Run(&command)
    {
        Ok(output) => match output.outcome
        {
            ExitOutcome::Exited { code } => StepExecution::Executed { code },
            other => StepExecution::Refused(GateUnknown::DidNotRun { step, cause: format!("{other:?}") }),
        },
        Err(cause) => StepExecution::Refused(GateUnknown::DidNotRun { step, cause }),
    };
}
