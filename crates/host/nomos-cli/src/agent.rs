//! `nomos agent` — dispatching a task to `nomos-agent-executor`, this workspace's first
//! real `AgentExecutor`.
//!
//! The composition root `nomos-agent-executor` itself cannot be: it depends on
//! `nomos-platform` and not on any concrete implementation of it, the same reason
//! `nomos-lang-rust-cargo` and every other `ProcessLauncher`-driven crate stays generic. This
//! module supplies `nomos_platform_std::StdProcessLauncher`, the same choice `check.rs` and
//! `work.rs` already make for their own subprocesses.
//!
//! `execute` is the only real caller `nomos-agent-executor` has anywhere in this workspace
//! today, other than its own tests. It renders [`nomos_agent_executor::AgentExecutionOutcome`]
//! directly rather than assembling a `nomos_agent_contracts::WorkResult` — `OD-CONTRACTS-003`
//! made `WorkResult.plan` representable as absent, but this command has no `RuleId` or
//! `SubjectId` to give a `Finding` either, since nothing dispatched it as a rule's judgment; it
//! is a person, asking a question directly. Reporting a `Finding` about nothing in particular
//! would be inventing a subject nobody named, the same category of dishonesty `OD-EXECUTOR-001`
//! already named for trusting a process's own account of itself.

use crate::arguments::{Name, Named_Value, Required, Usage};
use nomos_agent_contracts::TaskEnvelope;
use nomos_agent_executor::{AgentExecutionError, AgentExecutionOutcome};
use nomos_contracts::SchemaId;
use nomos_ledger::Territory;
use nomos_platform_std::StdProcessLauncher;

/// What the process exits with.
///
/// Shares its numbers with every other group on this binary — see `check::ExitCode`'s own
/// doc for the fuller statement of that discipline. `5` already carries "could not be read
/// / run / answered at all" for `check`'s `Unreadable` and `spec`'s `StoreError`; a process
/// that could not be started, exited non-zero, timed out, or answered with something other
/// than the JSON it promised is the identical shape one seam over, not a fresh meaning.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExitCode
{
    /// The executor ran and answered.
    Ok = 0,
    /// The command line was wrong.
    Usage = 2,
    /// The executor could not be started, exited non-zero, timed out or stalled, or
    /// answered with something other than the JSON `--output-format json` promises.
    Unavailable = 5,
}

impl ExitCode
{
    #[must_use]
    pub const fn Value(self) -> i32
    {
        return self as i32;
    }
}

/// What `nomos agent` was asked to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Command
{
    Execute
    {
        goal: String
    },
}

/// Parses `nomos agent` arguments.
///
/// # Errors
///
/// Returns a message naming what was wrong and what was expected.
pub(crate) fn Parse(arguments: &[String]) -> Result<Command, String>
{
    let Some(verb) = arguments.first()
    else
    {
        return Err(Usage_Text());
    };

    let rest = arguments.get(1..).unwrap_or_default();

    return match verb.as_str()
    {
        "execute" => Parse_Execute(rest),
        other => Err(format!("unknown command `{other}`.\n\n{}", Usage_Text())),
    };
}

fn Parse_Execute(arguments: &[String]) -> Result<Command, String>
{
    let value = Named_Value(arguments, "--goal");
    let goal = Required(value.as_ref(), Name("--goal"), Usage(&Usage_Text()))?;

    return Ok(Command::Execute { goal });
}

/// Runs a command, writing content to `output` and everything about it to `notes`.
///
/// Returns the exit code rather than exiting, so the whole surface is testable.
pub(crate) fn Run(command: &Command, output: &mut impl std::io::Write, notes: &mut impl std::io::Write) -> ExitCode
{
    return match command
    {
        Command::Execute { goal } => Execute(goal, output, notes),
    };
}

/// A bare `TaskEnvelope` naming only `goal`. `scope`, `prohibited_changes` and
/// `available_tools` are the empty value `OD-EXECUTOR-001` already reads as "nothing
/// enumerated, nothing granted" — this command has no configuration surface to fill them
/// from yet, and inventing one ahead of a real need would repeat the mistake this workspace
/// has already declined to make elsewhere. `expected_output_schema` names this call site
/// rather than a real schema, since nothing here validates a response against one.
fn Task(goal: &str) -> TaskEnvelope
{
    return TaskEnvelope {
        goal: goal.to_owned(),
        scope: Territory::Of_Files(Vec::<String>::new()),
        knowledge_context: Vec::new(),
        applicable_rules: Vec::new(),
        prohibited_changes: Territory::Of_Files(Vec::<String>::new()),
        available_tools: Vec::new(),
        expected_output_schema: SchemaId::New("nomos.agent.executor.cli.v1"),
    };
}

fn Execute(goal: &str, output: &mut impl std::io::Write, notes: &mut impl std::io::Write) -> ExitCode
{
    return match nomos_agent_executor::Execute(&Task(goal), &StdProcessLauncher)
    {
        Ok(outcome) => Answered(&outcome, output),
        Err(error) => Unavailable(&error, notes),
    };
}

/// Renders an outcome for what it structurally reported, never for what its own text
/// claims — `OD-EXECUTOR-001`'s rule, restated at the one place this workspace renders an
/// executor's answer for a person to read. `denied_tool_uses` is printed unconditionally,
/// empty or not, so its absence is a caller's own observation rather than a line that only
/// appears when there is bad news to report.
fn Answered(outcome: &AgentExecutionOutcome, output: &mut impl std::io::Write) -> ExitCode
{
    let _ = writeln!(output, "{}", outcome.response);
    let _ = writeln!(output, "denied tool uses: {:?}", outcome.denied_tool_uses);
    let _ = writeln!(output, "is_error: {}  cost_usd: {}  duration_ms: {}", outcome.is_error, outcome.cost_usd, outcome.duration_ms);

    return ExitCode::Ok;
}

fn Unavailable(error: &AgentExecutionError, notes: &mut impl std::io::Write) -> ExitCode
{
    let _ = writeln!(notes, "{error}");

    return ExitCode::Unavailable;
}

fn Usage_Text() -> String
{
    return "usage: nomos agent <command>\n\
            \n\
            \x20 execute --goal <text>\n\
            \n\
            `execute` dispatches --goal to Claude Code as a bounded, tool-free subprocess \
            through nomos-agent-executor, under the structural capability boundary \
            OD-EXECUTOR-001 decided: an isolated working directory, no MCP configuration, an \
            allow-list naming no real tool, a $1 budget cap, one --print turn. It renders the \
            response and, unconditionally, which tool uses (if any) were structurally denied \
            -- the response text is never evidence of what happened, only of what the process \
            said. It does not assemble a WorkResult or a Finding: this is a person's direct \
            question, not a rule's judgment, and there is no subject or rule identity to \
            report one against.\n\
            \n\
            exit codes: 0 ok, 2 usage, 5 the executor could not run or answer"
        .to_owned();
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn Arguments(text: &str) -> Vec<String>
    {
        return text.split_whitespace().map(str::to_owned).collect();
    }

    #[test]
    fn Test_An_Execute_Command_Should_Parse_Its_Goal()
    {
        let arguments = Arguments("execute --goal hello");

        let Command::Execute { goal } = Parse(&arguments).expect("parses");

        assert_eq!(goal, "hello");
    }

    #[test]
    fn Test_A_Missing_Goal_Should_Be_A_Usage_Error()
    {
        let arguments = Arguments("execute");

        let error = Parse(&arguments).expect_err("must refuse");

        assert!(error.contains("--goal"));
    }

    #[test]
    fn Test_An_Unknown_Verb_Should_Be_A_Usage_Error()
    {
        let arguments = Arguments("dance");

        let error = Parse(&arguments).expect_err("must refuse");

        assert!(error.contains("dance"));
    }

    #[test]
    fn Test_No_Verb_At_All_Should_Be_A_Usage_Error()
    {
        let error = Parse(&[]).expect_err("must refuse");

        assert!(error.contains("usage"));
    }

    /// The one place `--goal ""` reaches this crate's tests without spending a real
    /// invocation: parsing accepts an empty string exactly as it accepts any other, and
    /// whatever `nomos-agent-executor` or the real CLI does with it is that crate's own
    /// concern, verified there, not re-verified through this transport.
    #[test]
    fn Test_An_Empty_Goal_Still_Parses()
    {
        let arguments = Arguments("execute --goal");
        let error = Parse(&arguments).expect_err("a flag with nothing after it has no value");

        assert!(error.contains("--goal"));
    }
}
