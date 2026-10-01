//! Turning one local execution of the gate's own step set into text and an [`ExitCode`].

use nomos_ledger::{LocalGateRun, StepExecution};
use std::io::Write;

use crate::gate::ExitCode;

/// Reports every step of one local execution, then what it means for the shell.
///
/// Every declared step is printed, including the ones this host did not run. That is the
/// whole point rather than a courtesy: `OD-GATE-033` requires a local run to report the set
/// it did not execute, because an execution that judged a subset and showed a clean result is
/// the defect `OD-GATE-001` and `OD-GATE-020` are about, and the `Corpus gates` step is this
/// workflow's own precedent for making such an absence visible.
///
/// The two ways a step goes unrun stay apart in the output, because they are different
/// claims. `unavailable` means no execution on this host could say anything about the step.
/// `refused` means this reader could not understand the step well enough to run it, and
/// carries the cause that says why.
pub(in crate::gate) fn Render_Steps(run: &LocalGateRun, host: &str, stdout: &mut impl Write) -> ExitCode
{
    let unexecuted = run.Unexecuted();
    let executed = run.Steps().len().saturating_sub(unexecuted.len());

    writeln!(stdout, "{} step(s) declared, executed as `{host}`", run.Steps().len()).ok();
    writeln!(stdout).ok();

    for (name, execution) in run.Steps()
    {
        writeln!(stdout, "  {}", Step_Line(name, execution)).ok();
    }

    writeln!(stdout).ok();
    writeln!(stdout, "  executed: {executed}").ok();

    if unexecuted.is_empty()
    {
        writeln!(stdout, "  not executed: none").ok();
    }
    else
    {
        writeln!(stdout, "  not executed: {}", unexecuted.join(", ")).ok();
    }

    return Disposition_Of(run);
}

/// One step's line: what it got, and for the two unrun values, why.
///
/// The value is padded to one width so the column of verdicts reads down the page. Thirteen
/// is what the widest of them needs: `executed(-101)`, an exit code a launcher can really
/// report, is exactly that wide.
fn Step_Line(name: &str, execution: &StepExecution) -> String
{
    const VERDICT_WIDTH: usize = 13;

    let (verdict, detail) = match execution
    {
        StepExecution::Executed { code } => (format!("executed({code})"), String::new()),
        StepExecution::Unavailable { host } => {
            ("unavailable".to_owned(), format!(" -- its guard does not admit `{host}`"))
        }
        StepExecution::Refused(cause) => ("refused".to_owned(), format!(" -- {}", cause.Describe())),
    };

    return format!("{verdict:<VERDICT_WIDTH$}{name}{detail}");
}

/// What the shell is told.
///
/// A step that ran and failed is a definite answer and takes precedence: the gate said no,
/// and that is actionable whether or not the rest of the set was reachable.
///
/// Otherwise an execution that could not cover the whole set is `Contradictory` rather than
/// `Ok`, and the reason is the one `OD-GATE-016` already established one layer out: a run
/// whose claim is incomplete reports `Indeterminate` and reaches this same code, because a
/// clean result over a subset would say something the run never established. `Ok` is
/// therefore reserved for the case it actually means -- every declared step ran here, and
/// every one of them passed.
fn Disposition_Of(run: &LocalGateRun) -> ExitCode
{
    let failed = run
        .Steps()
        .iter()
        .any(|(_, execution)| return matches!(execution, StepExecution::Executed { code } if *code != 0));

    if failed
    {
        return ExitCode::Violations;
    }

    if run.Unexecuted().is_empty()
    {
        return ExitCode::Ok;
    }

    return ExitCode::Contradictory;
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_ledger::{DerivedStep, GateUnknown, Run_Gate_Locally, StepGuard};
    use nomos_platform::{
        Command, DeterminismStrength, ExitOutcome, ProgramLauncher, ProgramOutput, ReproducibilityScope, Strategy,
        TraceEquivalence,
    };

    const LINUX: &str = "ubuntu-latest";
    const WINDOWS: &str = "windows-latest";

    /// A launcher that runs nothing and reports one exit code.
    struct Scripted(i32);

    impl Strategy for Scripted
    {
        const STRENGTH: DeterminismStrength = DeterminismStrength::State;
        const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
        const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
    }

    impl ProgramLauncher for Scripted
    {
        fn Run(&self, _command: &Command) -> Result<ProgramOutput, String>
        {
            return Ok(ProgramOutput {
                outcome: ExitOutcome::Exited { code: self.0 },
                stdout: String::new(),
                stderr: String::new(),
            });
        }
    }

    /// One step that every host runs, and one guarded to Windows.
    fn Two_Steps() -> Vec<DerivedStep>
    {
        return vec![
            DerivedStep {
                name: "Everywhere".to_owned(),
                guard: StepGuard::Unguarded,
                argv: Ok(vec!["cargo".to_owned(), "--version".to_owned()]),
            },
            DerivedStep {
                name: "Windows only".to_owned(),
                guard: StepGuard::Host(WINDOWS.to_owned()),
                argv: Ok(vec!["cargo".to_owned(), "--version".to_owned()]),
            },
        ];
    }

    fn Rendered(steps: &[DerivedStep], host: &str, code: i32) -> (String, ExitCode)
    {
        let run = Run_Gate_Locally(steps, host, &Scripted(code), None);
        let mut stdout = Vec::new();
        let exit = Render_Steps(&run, host, &mut stdout);

        return (String::from_utf8(stdout).expect("the renderer writes text"), exit);
    }

    /// The falsifier for the rendering. It reads the printed output rather than the values,
    /// so omitting the unexecuted steps from the report fails this while every value stays
    /// right.
    #[test]
    fn Test_The_Report_Should_Name_Every_Step_It_Did_Not_Execute()
    {
        let (printed, _) = Rendered(&Two_Steps(), LINUX, 0);

        assert!(printed.contains("not executed: Windows only"), "{printed}");
        assert!(printed.contains("unavailable"), "{printed}");
    }

    /// The falsifier for the exit code. Every executed step passed, and the run still must
    /// not tell the shell it was clean, because it never covered the whole set.
    #[test]
    fn Test_An_Unavailable_Step_Should_Keep_The_Run_From_Exiting_Clean()
    {
        let (_, exit) = Rendered(&Two_Steps(), LINUX, 0);

        assert_eq!(exit, ExitCode::Contradictory);
    }

    /// The same set on the host that runs all of it is the only case that exits clean.
    #[test]
    fn Test_A_Run_That_Covered_The_Whole_Set_Should_Exit_Clean()
    {
        let (printed, exit) = Rendered(&Two_Steps(), WINDOWS, 0);

        assert_eq!(exit, ExitCode::Ok);
        assert!(printed.contains("not executed: none"), "{printed}");
    }

    /// A step that ran and failed is a definite answer, and takes precedence over
    /// incompleteness.
    #[test]
    fn Test_A_Failing_Step_Should_Report_Violations_Even_When_The_Set_Was_Incomplete()
    {
        const FAILED: i32 = 101;

        let (printed, exit) = Rendered(&Two_Steps(), LINUX, FAILED);

        assert_eq!(exit, ExitCode::Violations);
        assert!(printed.contains("executed(101)"), "{printed}");
    }

    /// The two unrun values stay apart in the text a reader sees, not only in the values.
    #[test]
    fn Test_The_Report_Should_Keep_Unavailable_And_Refused_Apart()
    {
        let refused = vec![DerivedStep {
            name: "A script".to_owned(),
            guard: StepGuard::Unguarded,
            argv: Err(GateUnknown::NotASingleCommand {
                step: "A script".to_owned(),
                run: "echo one && echo two".to_owned(),
            }),
        }];

        let (printed, exit) = Rendered(&refused, LINUX, 0);

        assert!(printed.contains("refused"), "{printed}");
        assert!(!printed.contains("unavailable"), "{printed}");
        assert_eq!(exit, ExitCode::Contradictory);
    }
}
