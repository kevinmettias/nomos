//! Band 37 — the first real `AgentExecutor`: `nomos-agent-contracts::TaskEnvelope` in, a
//! bounded Claude Code subprocess dispatched through it, [`AgentExecutionOutcome`] out.
//!
//! `OD-EXECUTOR-001` decided what this dispatch is permitted to do before this crate existed
//! to do it: a freshly created, empty, isolated working directory; no MCP configuration; tool
//! access granted through an allow-list naming no real tool rather than denied through a list
//! that will always be one release behind the real tool set; one `--print` turn; and a result
//! read for what it structurally permitted, never for what its own free text claims happened.
//! [`Execute_Task`] is that record's rule, and nothing more. A second, distinct concern —
//! bounding what the dispatch may *cost*, not what it may *touch* — is not that record's
//! question; `Command_For` requests `--max-budget-usd`, verified empirically to abort before
//! the expensive model call runs rather than merely reporting overspend afterward, though a
//! small overshoot bounded by one cheap triage call is still possible and nothing caps turn
//! count directly.
//!
//! It does not assemble a `nomos_agent_contracts::WorkResult`. `OD-CONTRACTS-003` made
//! `WorkResult.plan` an `Option`, so the type itself can now represent a judgment-only
//! response — but this crate still does not build one, because a free-text `response` has no
//! honest, general mapping into `claims`/`assumptions`/`unresolved_questions` either.
//! `TaskEnvelope.expected_output_schema`, paired with Claude Code's own `--json-schema`
//! support, is the real shape a future increment would use to have the agent produce a
//! `WorkResult`-shaped response directly rather than have this crate guess one from prose —
//! not attempted here.
//!
//! `TaskEnvelope.goal` and, since `OD-CONTRACTS-004`, `TaskEnvelope.effort` are read. `scope`,
//! `prohibited_changes`, `available_tools`, `knowledge_context` and `applicable_rules` are
//! accepted and ignored, matching `OD-EXECUTOR-001`'s own finding that nothing in this
//! workspace enforces them yet — this crate does not pretend otherwise by silently honoring
//! some of them and not others.

#![forbid(unsafe_code)]

mod agent_execution_error;
mod agent_execution_outcome;
mod response;

pub use agent_execution_error::AgentExecutionError;
pub use agent_execution_outcome::AgentExecutionOutcome;

use nomos_agent_contracts::TaskEnvelope;
use nomos_model_package::EffortLevel;
use nomos_platform::{Command, ExitOutcome, ProcessLauncher};
use std::path::PathBuf;
use std::time::Duration;

/// A prompt this repository has already run against the real CLI takes well under a
/// minute; five minutes is headroom for a longer real task without leaving a hung
/// subprocess to wait out an unbounded timeout.
const TIMEOUT: Duration = Duration::from_secs(300);

/// Names no real tool, by construction. `--allowedTools` set to only this value grants
/// nothing: `OD-EXECUTOR-001`'s amendment measured a deny-list leaking twice — once for a
/// platform-specific tool name the list's author did not know to include, once for a
/// tool this crate never anticipated at all — before an allow-list naming this same kind
/// of placeholder held under an adversarial prompt. The leading underscores and the name
/// itself are chosen so a reader, or a future real tool, can never mistake this for
/// something meant to match.
const NO_TOOLS_GRANTED: &str = "__nomos_agent_executor_denies_all_tools__";

/// A starting bound, not a derived one — real invocations this crate has run cost between
/// under a cent and a few tens of cents. A dollar is generous headroom for a single
/// judgment-only turn while still being a real, structural ceiling on what a runaway
/// sequence of denied-tool retries could cost, verified empirically: capped at an
/// unreachably low budget, `claude` aborted with exit code 1 before its expensive model
/// call ran, incurring only a small triage-model cost first — the overshoot this default
/// cannot fully close, only bound.
const MAX_BUDGET_USD: &str = "1.00";

/// Dispatches `task.goal` to Claude Code as a subprocess, bounded by `OD-EXECUTOR-001`'s
/// structural capability boundary, and reads back what it reported.
///
/// # Errors
///
/// [`AgentExecutionError::Unavailable`] if the isolated working directory could not be
/// created, the process could not be started, exited non-zero, or was killed for timing
/// out or stalling. [`AgentExecutionError::Unparseable`] if its stdout was not the JSON
/// document `--output-format json` promises.
pub fn Execute_Task<Launcher: ProcessLauncher>(task: &TaskEnvelope, launcher: &Launcher) -> Result<AgentExecutionOutcome, AgentExecutionError>
{
    let working_directory = Isolated_Working_Directory()?;

    return Execute_In(task, launcher, &working_directory);
}

/// [`Execute_Task`], over a caller-chosen `working_directory` rather than a freshly generated
/// one — the seam this crate's own real, adversarial integration test uses to inspect
/// that directory afterward, since `Execute_Task`'s own isolated directory is otherwise
/// generated and discarded where no caller could ever name it.
pub(crate) fn Execute_In<Launcher: ProcessLauncher>(
    task: &TaskEnvelope,
    launcher: &Launcher,
    working_directory: &std::path::Path,
) -> Result<AgentExecutionOutcome, AgentExecutionError>
{
    let command = Command_For(task, working_directory);
    let output = launcher.Run(&command).map_err(AgentExecutionError::Unavailable)?;

    Require_Clean_Exit(&output.outcome, &output.stderr)?;

    return response::Parse_Response(&output.stdout);
}

/// The invocation `OD-EXECUTOR-001`'s rule describes, over `task.goal`, run from
/// `working_directory`, with `task.effort` appended per [`Effort_Flag`].
fn Command_For(task: &TaskEnvelope, working_directory: &std::path::Path) -> Command
{
    let mut argv = vec![
        CLAUDE_PROGRAM.to_owned(),
        "--print".to_owned(),
        Single_Line(&task.goal),
        "--output-format".to_owned(),
        "json".to_owned(),
        "--strict-mcp-config".to_owned(),
        "--allowedTools".to_owned(),
        NO_TOOLS_GRANTED.to_owned(),
        "--max-budget-usd".to_owned(),
        MAX_BUDGET_USD.to_owned(),
    ];
    if let Some(value) = Effort_Flag(task.effort)
    {
        argv.push("--effort".to_owned());
        argv.push(value.to_owned());
    }

    let mut command = Command::New(argv, TIMEOUT);
    command.working_directory = Some(working_directory.to_path_buf());

    return command;
}

/// `task.effort`, mapped to the real `--effort` value `claude --help` documents today —
/// verified directly against the installed CLI, not assumed: `low`, `medium`, `high`,
/// `xhigh`, `max`. `None` for [`EffortLevel::BackendDefault`]: the flag is omitted
/// entirely rather than passed a value naming "the default," which is exactly this
/// crate's own behavior for every caller before `OD-CONTRACTS-004` existed to name an
/// effort at all.
///
/// [`EffortLevel::Minimal`] has no distinct native control below `low` — mapped there as
/// this crate's own approximation, not a claim of an exact match, `MODEL-ROUTE-015`'s own
/// `MappingQuality::Approximate` shape for exactly this case. `claude`'s own `xhigh` tier
/// has no `EffortLevel` counterpart: `MODEL-ROUTE-004` closes the canonical enumeration at
/// six values, so this crate cannot request it, and does not fold it into `high` or `max`
/// to pretend otherwise.
#[must_use]
fn Effort_Flag(effort: EffortLevel) -> Option<&'static str>
{
    return match effort
    {
        EffortLevel::BackendDefault => None,
        EffortLevel::Minimal | EffortLevel::Low => Some("low"),
        EffortLevel::Medium => Some("medium"),
        EffortLevel::High => Some("high"),
        EffortLevel::Maximum => Some("max"),
    };
}

/// Refuses every outcome a launched process can report other than a clean, zero exit —
/// the same shape `nomos_lang_rust_cargo`'s own `Require_Clean_Exit` uses for the one
/// other subprocess this workspace ever runs through `ProcessLauncher`.
fn Require_Clean_Exit(outcome: &ExitOutcome, stderr: &str) -> Result<(), AgentExecutionError>
{
    return match outcome
    {
        ExitOutcome::Exited { code: 0 } => Ok(()),
        ExitOutcome::Exited { code } => {
            Err(AgentExecutionError::Unavailable(format!("claude exited {code}: {stderr}")))
        }
        ExitOutcome::TimedOut => {
            Err(AgentExecutionError::Unavailable(format!("claude was still running after {TIMEOUT:?} and was killed")))
        }
        ExitOutcome::Stalled { idle_elapsed } => Err(AgentExecutionError::Unavailable(format!(
            "claude produced no output for {idle_elapsed:?} and was judged stalled"
        ))),
        ExitOutcome::Terminated => {
            Err(AgentExecutionError::Unavailable("claude was terminated before it could finish".to_owned()))
        }
    };
}

/// A newline (`\n` or `\r`) collapsed to a space and a double quote turned into a single
/// one, so `task.goal` survives `Command_For` regardless of platform.
///
/// Both verified directly against the real CLI, as two distinct failures, not one.
/// `StdProcessLauncher` spawns `claude.cmd` through Rust's own `std::process::Command`
/// with no shell, and a goal carrying an embedded newline failed there with "batch file
/// arguments are invalid" — the Windows-only hardening `std` added for CVE-2024-24576,
/// which refuses certain argument content when the target is a `.bat`/`.cmd` file rather
/// than risk it being used to inject a second command when `cmd.exe` re-parses it. Once
/// that was fixed, a goal carrying an embedded `"` no longer triggered a refusal but
/// produced a *different* failure — `claude`'s own stdout was not the JSON it promised —
/// consistent with `cmd.exe`'s own batch-argument tokenizer, a second and separate layer
/// from `std`'s CVE fix, re-splitting the argument on the quote before `claude.cmd` ever
/// saw it, rather than passing it through as one value. Routing around either by
/// hand-escaping would be reproducing the exact class of bug the first fix exists to
/// close, so this crate does not try; a goal is free text for a model to read, not a
/// document whose exact punctuation this invocation depends on, and normalizing both is
/// honest rather than a workaround. Applied on every platform, not only Windows, so
/// `Command_For`'s output does not depend on which one built it.
///
/// The quote substitution has a real, observed cost, named rather than hidden: run
/// end-to-end against the real CLI with a goal quoting a Rust string literal
/// (`const X: &str = "1.00";`), the substitution turned it into `'1.00'` — a char literal,
/// not a string — and the model correctly reported the resulting snippet as broken code.
/// A goal embedding source code that itself uses double quotes will read differently to
/// the model than the caller wrote it. No fix for that is attempted here; it is a real
/// limitation of this invocation path, not a case this crate silently gets right.
fn Single_Line(goal: &str) -> String
{
    return goal.replace(['\n', '\r'], " ").replace('"', "'");
}

/// A freshly created, empty directory under the system temp root, never this repository's
/// own tree and never one carrying its own `.claude/settings*` or `CLAUDE.md` — the first
/// clause of `OD-EXECUTOR-001`'s rule.
///
/// Delegates to `nomos_agent_contracts::Isolated_Working_Directory`, shared with
/// `nomos-model-backend-ollama`'s own isolation step; this crate's only distinct part is
/// the prefix its directories are named from.
fn Isolated_Working_Directory() -> Result<PathBuf, AgentExecutionError>
{
    return nomos_agent_contracts::Isolated_Working_Directory("nomos-agent-executor").map_err(|error| {
        return AgentExecutionError::Unavailable(error.to_string());
    });
}

/// `claude` on every platform this workspace's own `StdProcessLauncher` runs on but
/// Windows, where the real entry point on `PATH` is an npm-generated `claude.cmd` shim
/// (confirmed directly against this machine: `where claude` names both an extensionless
/// POSIX shell script and `claude.cmd`, in that order). `StdProcessLauncher` spawns
/// `argv[0]` through `CreateProcess` directly, by its own deliberate "no shell, ever"
/// design — the same reason it does not attempt `PATHEXT` resolution a shell would do
/// silently, so the correct name is this crate's own responsibility to supply, once, here,
/// rather than a capability every caller of the shared launcher would otherwise need.
#[cfg(windows)]
const CLAUDE_PROGRAM: &str = "claude.cmd";

/// See [`CLAUDE_PROGRAM`]'s Windows doc.
#[cfg(not(windows))]
const CLAUDE_PROGRAM: &str = "claude";

#[cfg(test)]
mod tests;

/// The direct address for `Execute_Task`/`Execute_In`. `tests.rs`'s own suite exercises
/// both extensively -- Rust's own coverage attribution keys a test to the FILE that
/// physically contains it, and this crate deliberately keeps its behavioural suite in its
/// own file (`tests.rs`) rather than inline, so a test living there cannot address a
/// function declared here. These two are the address that file cannot supply.
#[cfg(test)]
mod address_tests
{
    use super::*;
    use nomos_ledger::Territory;
    use nomos_platform::ProcessOutput;

    struct Scripted
    {
        outcome: ExitOutcome,
        stdout: String,
    }

    impl ProcessLauncher for Scripted
    {
        fn Run(&self, _command: &Command) -> Result<ProcessOutput, String>
        {
            return Ok(ProcessOutput { outcome: self.outcome, stdout: self.stdout.clone(), stderr: String::new() });
        }
    }

    fn Bare_Task(goal: &str) -> TaskEnvelope
    {
        return TaskEnvelope {
            goal: goal.to_owned(),
            scope: Territory::Of_Files(Vec::<String>::new()),
            knowledge_context: Vec::new(),
            applicable_rules: Vec::new(),
            prohibited_changes: Territory::Of_Files(Vec::<String>::new()),
            available_tools: Vec::new(),
            expected_output_schema: nomos_contracts::SchemaId::New("nomos.agent.executor.v1"),
            effort: EffortLevel::BackendDefault,
        };
    }

    #[test]
    fn Test_Execute_Task_Should_Create_Its_Own_Isolated_Directory_And_Read_A_Clean_Response()
    {
        let launcher = Scripted {
            outcome: ExitOutcome::Exited { code: 0 },
            stdout: r#"{"result": "PONG", "is_error": false, "total_cost_usd": 0.01, "duration_ms": 500, "permission_denials": []}"#
                .to_owned(),
        };

        let outcome = Execute_Task(&Bare_Task("say PONG"), &launcher).expect("a well-formed scripted response");

        assert_eq!(outcome.response, "PONG");
    }

    #[test]
    fn Test_Execute_In_Should_Run_Over_A_Caller_Chosen_Directory()
    {
        let launcher = Scripted {
            outcome: ExitOutcome::Exited { code: 0 },
            stdout: r#"{"result": "PONG", "is_error": false, "total_cost_usd": 0.01, "duration_ms": 500, "permission_denials": []}"#
                .to_owned(),
        };
        let directory = std::env::temp_dir();

        let outcome = Execute_In(&Bare_Task("say PONG"), &launcher, &directory).expect("a well-formed scripted response");

        assert_eq!(outcome.response, "PONG");
    }
}
