//! The seam between `nomos_cli`'s `agent` group and `nomos_model_backend_ollama`.
//!
//! `agent/dispatch.rs::Dispatch_Task` calls
//! `nomos_model_backend_ollama::Execute_Task(task, &StdProcessLauncher)` unconditionally for
//! `--model-backend ollama`, spawning a real `ollama` subprocess with no injection point --
//! the same `check-test-coverage: allow-untested` reasoning
//! `tests/agent_executor_claude_code_seam.rs` documents applies here too, so this suite never
//! drives that call for real.
//!
//! Instead it calls the exact same public `Execute_Task<Launcher: ProcessLauncher>` nomos-cli
//! itself calls, with a scripted, in-process `ProcessLauncher` -- the identical pattern
//! `nomos-model-backend-ollama::address_tests` already uses for its own direct suite. This
//! proves the real contract nomos-cli's `Dispatch_Task` depends on for this backend: a
//! `TaskEnvelope` shaped the way `dispatch.rs::Execute_Task` builds it, in; an
//! `AgentExecutionOutcome` carrying only `response` -- the one field
//! `agent/dispatch.rs::Answered_Ollama` renders, honestly, since this backend has no
//! `denied_tool_uses` or dollar cost to report -- out, or an `AgentExecutionError` otherwise.

use nomos_agent_contracts::TaskEnvelope;
use nomos_model_backend_ollama::{AgentExecutionError, Execute_Task};
use nomos_platform::{Command, ExitOutcome, ProcessLauncher, ProcessOutput};

#[path = "support/mod.rs"]
mod support;

use support::Run;

/// A `ProcessLauncher` that never spawns a process, the same shape
/// `tests/agent_executor_claude_code_seam.rs` uses for the sibling backend.
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

/// The bare envelope `agent/dispatch.rs::Execute_Task` builds for `nomos agent execute
/// --goal <text> --model-backend ollama`.
fn Bare_Task(goal: &str) -> TaskEnvelope
{
    return TaskEnvelope {
        goal: goal.to_owned(),
        scope: nomos_ledger::Territory::Of_Files(Vec::<String>::new()),
        knowledge_context: Vec::new(),
        applicable_rules: Vec::new(),
        prohibited_changes: nomos_ledger::Territory::Of_Files(Vec::<String>::new()),
        available_tools: Vec::new(),
        expected_output_schema: nomos_contracts::SchemaId::New("nomos.agent.executor.cli.v1"),
        effort: nomos_model_package::EffortLevel::BackendDefault,
    };
}

/// The happy path: a clean scripted response reads back as the one field
/// `agent/dispatch.rs::Answered_Ollama` renders to a caller of `nomos agent execute
/// --model-backend ollama`.
#[test]
fn Test_Execute_Task_Should_Return_The_Trimmed_Response_The_Cli_Renders_For_A_Clean_Response()
{
    let launcher = Scripted { outcome: ExitOutcome::Exited { code: 0 }, stdout: "PONG\n".to_owned() };

    let outcome = Execute_Task(&Bare_Task("say PONG"), &launcher).expect("a well-formed scripted response");

    assert_eq!(outcome.response, "PONG", "the response is trimmed of trailing whitespace");
}

/// The error that crosses this boundary: a non-zero exit (including "the `ollama serve`
/// daemon this backend requires is unreachable", per `Execute_Task`'s own doc) is refused
/// as `AgentExecutionError::Unavailable` rather than read as a response -- the case
/// `agent/dispatch.rs::Backend_Unavailable` renders to `nomos agent execute`'s notes
/// stream.
#[test]
fn Test_Execute_Task_Should_Refuse_A_Non_Zero_Exit_Rather_Than_Read_It_As_A_Response()
{
    let launcher = Scripted { outcome: ExitOutcome::Exited { code: 1 }, stdout: String::new() };

    let error = Execute_Task(&Bare_Task("say PONG"), &launcher).expect_err("a non-zero exit must not be read as success");

    assert!(matches!(error, AgentExecutionError::Unavailable(_)), "{error}");
}

/// Through the real binary: `nomos agent execute --model-backend ollama` with no `--goal`
/// refuses before a `TaskEnvelope` is ever built and before `Dispatch_Task` is ever
/// called -- safe to run for real, since parsing never reaches either backend.
#[test]
fn Test_Agent_Execute_With_Ollama_Should_Refuse_Before_Ever_Reaching_The_Backend_When_Goal_Is_Missing()
{
    let ran = Run(&["agent", "execute", "--model-backend", "ollama"]);

    assert_eq!(ran.code, 2, "missing --goal is a usage refusal: {}", ran.stderr);
}
