//! Assembling a bare `TaskEnvelope` and dispatching any `TaskEnvelope` to a chosen backend --
//! the primitive `execute` and `judge-role` both end at.

use super::{Backend, DispatchConfig, ExitCode};
use nomos_agent_contracts::TaskEnvelope;
use nomos_contracts::SchemaId;
use nomos_ledger::Territory;
use nomos_platform_std::StdProcessLauncher;

pub(super) fn Execute(goal: &str, config: DispatchConfig, output: &mut impl std::io::Write, notes: &mut impl std::io::Write) -> ExitCode
{
    let task = Task(goal, config.effort);

    return Dispatch(&task, config.backend, output, notes);
}

/// A bare `TaskEnvelope` naming only `goal` and `effort`. `scope`, `prohibited_changes` and
/// `available_tools` are the empty value `OD-EXECUTOR-001` already reads as "nothing
/// enumerated, nothing granted" — this command has no configuration surface to fill them
/// from yet, and inventing one ahead of a real need would repeat the mistake this workspace
/// has already declined to make elsewhere. `expected_output_schema` names this call site
/// rather than a real schema, since nothing here validates a response against one.
fn Task(goal: &str, effort: nomos_model_package::EffortLevel) -> TaskEnvelope
{
    return TaskEnvelope {
        goal: goal.to_owned(),
        scope: Territory::Of_Files(Vec::<String>::new()),
        knowledge_context: Vec::new(),
        applicable_rules: Vec::new(),
        prohibited_changes: Territory::Of_Files(Vec::<String>::new()),
        available_tools: Vec::new(),
        expected_output_schema: SchemaId::New("nomos.agent.executor.cli.v1"),
        effort,
    };
}

/// Runs `task` against `backend` and renders whichever of the two outcome shapes it
/// produces. The two crates share no trait -- `OD-EXECUTOR-001`/`OD-EXECUTOR-004` both
/// decline to invent one ahead of a real need, and `OD-EXECUTOR-005` found that trigger has
/// not fired even once `--executor`/`--model-backend` replaced `--backend`: there is still
/// only one real `AgentExecutor`, so this match is the entire dispatch, not a stand-in for a
/// trait either flag's own vocabulary would need.
pub(super) fn Dispatch(task: &TaskEnvelope, backend: Backend, output: &mut impl std::io::Write, notes: &mut impl std::io::Write) -> ExitCode
{
    return match backend
    {
        Backend::ClaudeCode => match nomos_agent_executor_claude_code::Execute(task, &StdProcessLauncher)
        {
            Ok(outcome) => Answered_Claude_Code(&outcome, output),
            Err(error) => Unavailable(&error, notes),
        },
        Backend::Ollama => match nomos_model_backend_ollama::Execute(task, &StdProcessLauncher)
        {
            Ok(outcome) => Answered_Ollama(&outcome, output),
            Err(error) => Unavailable(&error, notes),
        },
    };
}

/// Renders an outcome for what it structurally reported, never for what its own text
/// claims — `OD-EXECUTOR-001`'s rule, restated at the one place this workspace renders an
/// executor's answer for a person to read. `denied_tool_uses` is printed unconditionally,
/// empty or not, so its absence is a caller's own observation rather than a line that only
/// appears when there is bad news to report.
fn Answered_Claude_Code(outcome: &nomos_agent_executor_claude_code::AgentExecutionOutcome, output: &mut impl std::io::Write) -> ExitCode
{
    let _ = writeln!(output, "{}", outcome.response);
    let _ = writeln!(output, "denied tool uses: {:?}", outcome.denied_tool_uses);
    let _ = writeln!(output, "is_error: {}  cost_usd: {}  duration_ms: {}", outcome.is_error, outcome.cost_usd, outcome.duration_ms);

    return ExitCode::Ok;
}

fn Unavailable(error: &impl std::fmt::Display, notes: &mut impl std::io::Write) -> ExitCode
{
    let _ = writeln!(notes, "{error}");

    return ExitCode::Unavailable;
}

/// `nomos-model-backend-ollama`'s own outcome carries only `response`, honestly: there is
/// no `denied_tool_uses` to print because there is no tool subsystem to have denied
/// anything from, and no dollar cost because inference is local. Printing placeholder
/// values for fields this backend does not have would claim a signal it never produced.
fn Answered_Ollama(outcome: &nomos_model_backend_ollama::AgentExecutionOutcome, output: &mut impl std::io::Write) -> ExitCode
{
    let _ = writeln!(output, "{}", outcome.response);

    return ExitCode::Ok;
}
