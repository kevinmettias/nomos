//! What `nomos workflow` was asked for.

use super::WorkflowCommand;
use crate::arguments::{Named_Value_From_String_Arguments, Named_Values_From_String_Arguments, Name, Required_Value, Usage};
use nomos_agent_contracts::TaskEnvelope;
use nomos_contracts::{RuleId, SchemaId};
use nomos_gate_orchestration::{GateCommand, RuleSelector};
use nomos_workflow_orchestration::{Body, CheckBody, CommitIntent, CorrectionBody, GateBody};
use std::path::PathBuf;

/// Parses `nomos workflow` arguments.
///
/// # Errors
///
/// Returns a message naming what was wrong and what was expected.
pub fn Command_From_String_Arguments(arguments: &[String]) -> Result<WorkflowCommand, String>
{
    let Some(verb) = arguments.first()
    else
    {
        return Err(Usage_Text());
    };

    let rest = arguments.get(1..).unwrap_or_default();

    return match verb.as_str()
    {
        "run" => Ok(WorkflowCommand { body: Body_From_String_Arguments(rest)? }),
        other => Err(format!("unknown command `{other}`.\n\n{}", Usage_Text())),
    };
}

/// `--check`'s, `--correct`'s, `--gate`'s, `--executor`'s or `--model-backend`'s own body --
/// exactly one of the five, the same "a call reaches exactly one" discipline `agent.rs`'s own
/// `Backend_From_String_Arguments` already holds between the latter two, extended to a third,
/// fourth and fifth family that share no trait with any of the rest.
///
/// # Errors
///
/// Returns a message when none or more than one of the five is given, or when a value naming a
/// required flag is missing.
fn Body_From_String_Arguments(arguments: &[String]) -> Result<Body, String>
{
    if Named_Body_Count_From_String_Arguments(arguments) > 1
    {
        return Err(format!(
            "--check, --correct, --gate, --executor and --model-backend each name a different body; a step dispatches through exactly one, so pass at most one of them.\n\n{}",
            Usage_Text()
        ));
    }

    return Dispatched_Body_From_String_Arguments(arguments);
}

/// A step body declaring the family the named flag value asks for.
///
/// The flag still names a backend, and the step still reaches it -- but it reaches it by
/// declaring the family and letting the declared set answer, rather than by being written as
/// the variant for that backend. That difference is what `OD-PACKAGE-016` decision 9's
/// wiring is: a step says what it wants, and a build whose declared set no longer offers it
/// says so instead of dispatching anyway.
fn Agent_Body(family: &str, task: nomos_agent_contracts::TaskEnvelope) -> Body
{
    return Body::Agent(nomos_workflow_orchestration::AgentBody {
        task,
        profile: nomos_model_package::ModelExecutionProfile::New(
            nomos_model_package::ModelSelector::BackendFamily(family.to_owned()),
            nomos_model_package::EffortLevel::BackendDefault,
        ),
    });
}

/// How many of the five body-naming flags `arguments` carries. The flags are counted as booleans
/// rather than collected, because a repeated flag still names one body and a `Vec` of them would
/// silently read a repeat as a conflict.
fn Named_Body_Count_From_String_Arguments(arguments: &[String]) -> usize
{
    let check = arguments.iter().any(|argument| return argument == "--check");
    let correct = arguments.iter().any(|argument| return argument == "--correct");
    let gate = arguments.iter().any(|argument| return argument == "--gate");
    let executor = Named_Value_From_String_Arguments(arguments, "--executor");
    let model_backend = Named_Value_From_String_Arguments(arguments, "--model-backend");

    return usize::from(check)
        .saturating_add(usize::from(correct))
        .saturating_add(usize::from(gate))
        .saturating_add(usize::from(executor.is_some()))
        .saturating_add(usize::from(model_backend.is_some()));
}

/// The one body `arguments` names, now that exactly one of the three flag-only families has been
/// ruled out and the two value-naming families are left to [`Backend_Body_From_String_Arguments`].
fn Dispatched_Body_From_String_Arguments(arguments: &[String]) -> Result<Body, String>
{
    if arguments.iter().any(|argument| return argument == "--check")
    {
        return Check_Body_From_String_Arguments(arguments);
    }
    if arguments.iter().any(|argument| return argument == "--correct")
    {
        return Correction_Body_From_String_Arguments(arguments);
    }
    if arguments.iter().any(|argument| return argument == "--gate")
    {
        return Gate_Body_From_String_Arguments(arguments);
    }

    return Backend_Body_From_String_Arguments(arguments);
}

/// `--root`'s (default `.`) and every `--rule`'s value, as a [`CheckBody`] -- the walk itself
/// deferred to [`super::Run`], the same division `gate.rs` and `correct.rs` already keep between
/// parsing a command and walking the tree it names.
fn Check_Body_From_String_Arguments(arguments: &[String]) -> Result<Body, String>
{
    let root = Named_Value_From_String_Arguments(arguments, "--root").map_or_else(|| return PathBuf::from("."), PathBuf::from);
    let selected: Vec<RuleId> = Named_Values_From_String_Arguments(arguments, "--rule").iter().map(RuleId::New).collect();
    let body = CheckBody::New(root, Vec::new(), selected);

    return Ok(Body::Check(body));
}

/// `--root`'s (default `.`) and `--commit`'s presence, as a [`CorrectionBody`] -- the walk itself
/// deferred to [`super::Run`], the identical division [`Check_Body_From_String_Arguments`]
/// already keeps.
fn Correction_Body_From_String_Arguments(arguments: &[String]) -> Result<Body, String>
{
    let root = Named_Value_From_String_Arguments(arguments, "--root").map_or_else(|| return PathBuf::from("."), PathBuf::from);
    let commit = arguments.iter().any(|argument| return argument == "--commit");
    let body = CorrectionBody::New(root, Vec::new(), CommitIntent::From_Flag(commit));

    return Ok(Body::Correction(body));
}

/// `--root`'s (default `.`) and every `--rule`'s value, as a [`GateBody`] -- the identical two
/// flags [`Check_Body_From_String_Arguments`] already reads, since `Run_Gate` narrows by the same
/// `root` and rule selection `nomos_check_orchestration::Run` does.
fn Gate_Body_From_String_Arguments(arguments: &[String]) -> Result<Body, String>
{
    let root = Named_Value_From_String_Arguments(arguments, "--root").map_or_else(|| return PathBuf::from("."), PathBuf::from);
    let include: Vec<RuleId> = Named_Values_From_String_Arguments(arguments, "--rule").iter().map(RuleId::New).collect();
    let command = GateCommand { root, rules: RuleSelector { include }, ..GateCommand::default() };
    let body = GateBody::New(Vec::new(), command);

    return Ok(Body::Gate(body));
}

/// `--executor`'s or `--model-backend`'s value, as the family of backend `--goal` dispatches to --
/// the two families that name a value rather than merely appearing, and so are read here rather
/// than by the flag-presence checks in [`Dispatched_Body_From_String_Arguments`].
fn Backend_Body_From_String_Arguments(arguments: &[String]) -> Result<Body, String>
{
    let executor = Named_Value_From_String_Arguments(arguments, "--executor");
    let model_backend = Named_Value_From_String_Arguments(arguments, "--model-backend");

    if let Some(text) = executor
    {
        return match text.as_str()
        {
            "claude-code" => Ok(Agent_Body(text.as_str(), Task_From_String_Arguments(arguments)?)),
            other => Err(format!("--executor {other:?} is not one of claude-code.\n\n{}", Usage_Text())),
        };
    }
    if let Some(text) = model_backend
    {
        return match text.as_str()
        {
            "ollama" => Ok(Agent_Body(text.as_str(), Task_From_String_Arguments(arguments)?)),
            other => Err(format!("--model-backend {other:?} is not one of ollama.\n\n{}", Usage_Text())),
        };
    }

    return Err(format!("one of --check, --correct, --gate, --executor or --model-backend is required.\n\n{}", Usage_Text()));
}

/// `--goal`'s value, as a bare [`TaskEnvelope`] -- every other field empty or its own default,
/// the identical shape `agent.rs`'s own `execute` builds for a person's direct question with no
/// scope, no prohibited changes and no tool allow-list of its own.
fn Task_From_String_Arguments(arguments: &[String]) -> Result<TaskEnvelope, String>
{
    use nomos_ledger::Territory;
    use nomos_model_package::EffortLevel;

    let value = Named_Value_From_String_Arguments(arguments, "--goal");
    let goal = Required_Value(value.as_ref(), Name("--goal"), Usage(&Usage_Text()))?;

    return Ok(TaskEnvelope {
        goal,
        scope: Territory::Of_Files(Vec::<String>::new()),
        knowledge_context: Vec::new(),
        applicable_rules: Vec::new(),
        prohibited_changes: Territory::Of_Files(Vec::<String>::new()),
        available_tools: Vec::new(),
        expected_output_schema: SchemaId::New("nomos.workflow.step.v1"),
        effort: EffortLevel::BackendDefault,
    });
}

/// [`Usage_Text`]'s content.
pub(super) const USAGE_TEXT: &str = "usage: nomos workflow <command>\n\
        \n\
        \x20 run --check --root <path> [--rule <id>]...\n\
        \x20 run --correct --root <path> [--commit]\n\
        \x20 run --gate --root <path> [--rule <id>]...\n\
        \x20 run --executor claude-code --goal <text>\n\
        \x20 run --model-backend ollama --goal <text>\n\
        \n\
        Dispatches the one step this command composes through `nomos-workflow-\
        orchestration::Run`, over a fixed, always-coherent WorkflowStep declaration -- a \
        thin renderer over that seam, not a second place workflow semantics live. Pass \
        exactly one of --check, --correct, --gate, --executor or --model-backend; a step \
        dispatches through exactly one.\n\
        \n\
        --check walks --root (default the current directory) for `.rs` and `.go` source \
        and runs nomos-check-orchestration::Run over it, narrowed to --rule if given \
        (repeatable; empty selects every registered rule). Reports Ok for a Judged \
        outcome regardless of its findings -- run `nomos gate run` for a pass/fail \
        reading.\n\
        \n\
        --correct walks --root (default the current directory) the identical way \
        --check does and runs nomos-correction-orchestration::Run_Correction over it, \
        staging and validating a real blocking claim from either correction family and, \
        only with --commit, writing the fix back. The identical seam and the identical \
        --commit flag `nomos correct` already has.\n\
        \n\
        --gate walks --root the identical way --check does and runs nomos-gate-\
        orchestration::Run_Gate over it, narrowed to --rule the identical way --check is. \
        Unlike every other body, a failing gate ends the workflow rather than merely \
        completing as this step's own answer -- exit code 1, the same code an incoherent \
        step's own refusal already carries.\n\
        \n\
        --executor and --model-backend name which family of backend --goal dispatches \
        to, the identical two flags and the identical restriction `nomos agent execute` \
        already has.\n\
        \n\
        exit codes: 0 ok, 1 the one step refused itself or a gate step failed, 2 usage, \
        5 the named root, registry, executor or model backend could not be read, run, or \
        answered at all, 6 the check, correction or gate step found nothing to judge";

fn Usage_Text() -> String
{
    return USAGE_TEXT.to_owned();
}
