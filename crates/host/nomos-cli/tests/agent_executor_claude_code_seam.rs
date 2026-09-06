//! The seam between `nomos_cli`'s `agent` group and `nomos_agent_executor_claude_code`.
//!
//! `P43-AGENT-CANONICAL-SEAM-2` moved the call this file once documented --
//! `agent/dispatch.rs::Dispatch_Task` calling `nomos_agent_executor_claude_code::
//! Execute_Task(task, &StdProcessLauncher)` directly -- into `nomos-agent-orchestration`'s
//! own `Run_Agent_Execute`/`Run_Agent_Judgment`, generic over `ProcessLauncher` rather than
//! fixed here. `nomos-cli` no longer names this crate in its own production dependencies;
//! it reaches it only transitively, through the shared seam. This suite is kept anyway,
//! as a test-only dependency (see `Cargo.toml`'s own comment), rather than deleted: it
//! still drives the exact backend both `nomos agent execute` and `nomos agent judge-role`
//! reach through that seam, and `nomos-agent-orchestration`'s own tests make the identical
//! assertion over the shared dispatch itself, so no coverage is lost either way.
//!
//! It never spawns a real `claude` (or `claude.cmd`) subprocess -- this machine may have a
//! real one on `PATH`, and a test that reached it could trigger a live, costly, recursive
//! agent invocation.
//!
//! Instead it drives the exact same public function the shared seam calls --
//! `Execute_Task<Launcher: ProcessLauncher>` -- with a scripted, in-process `ProcessLauncher`
//! that never spawns anything, the identical pattern
//! `nomos-agent-executor-claude-code::address_tests` and `nomos-agent-orchestration::run`'s
//! own tests already use. This proves the real contract the shared seam depends on: a
//! `TaskEnvelope` shaped the way `nomos_agent_orchestration::run`'s own `Bare_Task` builds
//! it, in; an `AgentExecutionOutcome` carrying `response`, `denied_tool_uses`, `is_error`,
//! `cost_usd` and `duration_ms` -- every field `nomos-cli`'s own `agent/dispatch.rs::
//! Rendered` prints -- out, or an `AgentExecutionError` when the process reports anything
//! other than a clean exit.

use nomos_agent_contracts::TaskEnvelope;
use nomos_agent_executor_claude_code::{AgentExecutionError, Execute_Task};
use nomos_platform::{Command, ExitOutcome, ProcessLauncher, ProcessOutput};

#[path = "support/mod.rs"]
mod support;

use support::Run;

/// A `ProcessLauncher` that never spawns a process -- it hands back exactly what it was
/// built with, so this suite can drive `Execute_Task` deterministically and with zero risk
/// of touching a real `claude` binary that may exist on this machine's `PATH`.
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
/// --goal <text>` -- only `goal` and `effort` carry real content.
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

/// The happy path: a clean scripted response reads back as an `AgentExecutionOutcome`
/// carrying every field `nomos-cli`'s own `agent/dispatch.rs::Rendered` prints for a
/// caller of `nomos agent execute`.
#[test]
fn Test_Execute_Task_Should_Return_Every_Field_The_Cli_Renders_For_A_Clean_Response()
{
    let launcher = Scripted {
        outcome: ExitOutcome::Exited { code: 0 },
        stdout: r#"{"result": "PONG", "is_error": false, "total_cost_usd": 0.01, "duration_ms": 500, "permission_denials": []}"#
            .to_owned(),
    };

    let outcome = Execute_Task(&Bare_Task("say PONG"), &launcher).expect("a well-formed scripted response");

    assert_eq!(outcome.response, "PONG");
    assert!(outcome.denied_tool_uses.is_empty());
    assert!(!outcome.is_error);
    assert!((outcome.cost_usd - 0.01).abs() < f64::EPSILON);
    assert_eq!(outcome.duration_ms, 500);
}

/// The error that crosses this boundary: a non-zero exit is refused as
/// `AgentExecutionError::Unavailable` rather than read as a response, the exact case
/// `nomos_agent_orchestration::AgentDispatchOutcome::Unavailable` folds it into and
/// `agent/dispatch.rs::Rendered` writes to `nomos agent execute`'s notes stream.
#[test]
fn Test_Execute_Task_Should_Refuse_A_Non_Zero_Exit_Rather_Than_Read_It_As_A_Response()
{
    let launcher = Scripted { outcome: ExitOutcome::Exited { code: 1 }, stdout: String::new() };

    let error = Execute_Task(&Bare_Task("say PONG"), &launcher).expect_err("a non-zero exit must not be read as success");

    assert!(matches!(error, AgentExecutionError::Unavailable(_)), "{error}");
}

/// Through the real binary: `nomos agent execute` with no `--goal` refuses before a
/// `TaskEnvelope` is ever built and before the shared seam is ever called -- the one
/// argument shape this suite may drive through the compiled `nomos` binary itself without
/// any risk of reaching a live `claude` subprocess.
#[test]
fn Test_Agent_Execute_Should_Refuse_Before_Ever_Reaching_The_Claude_Code_Executor_When_Goal_Is_Missing()
{
    let ran = Run(&["agent", "execute"]);

    assert_eq!(ran.code, 2, "missing --goal is a usage refusal: {}", ran.stderr);
}
