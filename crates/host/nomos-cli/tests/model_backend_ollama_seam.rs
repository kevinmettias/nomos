//! The seam between `nomos_cli`'s `agent` group and `nomos_model_backend_ollama`.
//!
//! `P43-AGENT-CANONICAL-SEAM-2` moved the call this file once documented --
//! `agent/dispatch.rs::Dispatch_Task` calling `nomos_model_backend_ollama::
//! Execute_Task(task, &LAUNCHER)` directly for `--model-backend ollama` -- into
//! `nomos-agent-orchestration`'s own `Run_Agent_Execute`/`Run_Agent_Judgment`, generic over
//! `ProcessLauncher` rather than fixed here. `nomos-cli` no longer names this crate in its
//! own production dependencies; it reaches it only transitively, through the shared seam.
//! This suite is kept anyway, as a test-only dependency (see `Cargo.toml`'s own comment),
//! rather than deleted -- see `tests/agent_executor_claude_code_seam.rs`'s own doc for the
//! fuller account, identical for this backend.
//!
//! It never spawns a real `ollama` subprocess.
//!
//! Instead it calls the exact same public `Execute_Task<Launcher: ProcessLauncher>` the
//! shared seam calls, with a scripted, in-process `ProcessLauncher` -- the identical
//! pattern `nomos-model-backend-ollama::address_tests` and `nomos-agent-orchestration::
//! run`'s own tests already use. This proves the real contract the shared seam depends on
//! for this backend: a `TaskEnvelope` shaped the way `nomos_agent_orchestration::run`'s own
//! `Bare_Task` builds it, in; an `AgentExecutionOutcome` carrying only `response` -- the
//! one field `nomos-cli`'s own `agent/dispatch.rs::Rendered` prints, honestly, since this
//! backend has no `denied_tool_uses` or dollar cost to report -- out, or an
//! `AgentExecutionError` otherwise.

use nomos_platform::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
use nomos_agent_contracts::TaskEnvelope;
use nomos_model_backend_ollama::{AgentExecutionError, Execute_Task};
use nomos_platform::{Command, ExitOutcome, ProcessLauncher, ProcessOutput};

#[path = "support/mod.rs"]
mod support;

use support::Run;

/// What a missing `--goal` leaves the process with -- `agent/exit_code.rs::ExitCode::Usage`'s
/// own value, the same code both this backend's sibling seam and `nomos agent`'s own suite pin.
const USAGE_EXIT_CODE: i32 = 2;

/// A `ProcessLauncher` that never spawns a process, the same shape
/// `tests/agent_executor_claude_code_seam.rs` uses for the sibling backend.
struct Scripted
{
    outcome: ExitOutcome,
    stdout: String,
}

/// Answers from fixed data, so its outputs reproduce byte for byte.
impl Strategy for Scripted
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
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

/// The happy path: a clean scripted response reads back as the one field `nomos-cli`'s
/// own `agent/dispatch.rs::Rendered` prints to a caller of `nomos agent execute
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
/// `nomos_agent_orchestration::AgentDispatchOutcome::Unavailable` folds it into and
/// `agent/dispatch.rs::Rendered` writes to `nomos agent execute`'s notes stream.
#[test]
fn Test_Execute_Task_Should_Refuse_A_Non_Zero_Exit_Rather_Than_Read_It_As_A_Response()
{
    let launcher = Scripted { outcome: ExitOutcome::Exited { code: 1 }, stdout: String::new() };

    let error = Execute_Task(&Bare_Task("say PONG"), &launcher).expect_err("a non-zero exit must not be read as success");

    assert!(matches!(error, AgentExecutionError::Unavailable(_)), "{error}");
}

/// Through the real binary: `nomos agent execute --model-backend ollama` with no `--goal`
/// refuses before a `TaskEnvelope` is ever built and before the shared seam is ever
/// called -- safe to run for real, since parsing never reaches either backend.
#[test]
fn Test_Agent_Execute_With_Ollama_Should_Refuse_Before_Ever_Reaching_The_Backend_When_Goal_Is_Missing()
{
    let ran = Run(&["agent", "execute", "--model-backend", "ollama"]);

    assert_eq!(ran.code, USAGE_EXIT_CODE, "missing --goal is a usage refusal: {}", ran.stderr);
}
