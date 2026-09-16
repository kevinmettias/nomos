//! What `nomos agent` was asked to do.

use super::{Backend, Command};
use crate::arguments::{Name, Named_Value_From_String_Arguments, Required_Value, Usage};

/// Parses `nomos agent` arguments.
///
/// # Errors
///
/// Returns a message naming what was wrong and what was expected.
pub(crate) fn Command_From_String_Arguments(arguments: &[String]) -> Result<Command, String>
{
    let Some(verb) = arguments.first()
    else
    {
        return Err(Usage_Text());
    };

    let rest = arguments.get(1..).unwrap_or_default();

    return match verb.as_str()
    {
        "execute" => Execute_Command_From_String_Arguments(rest),
        "judge-role" => Judge_Role_Command_From_String_Arguments(rest),
        other => Err(format!("unknown command `{other}`.\n\n{}", Usage_Text())),
    };
}

fn Execute_Command_From_String_Arguments(arguments: &[String]) -> Result<Command, String>
{
    let value = Named_Value_From_String_Arguments(arguments, "--goal");
    let goal = Required_Value(value.as_ref(), Name("--goal"), Usage(&Usage_Text()))?;
    let effort = Effort_From_String_Arguments(arguments)?;
    let backend = Backend_From_String_Arguments(arguments)?;

    return Ok(Command::Execute { goal, effort, backend });
}

fn Judge_Role_Command_From_String_Arguments(arguments: &[String]) -> Result<Command, String>
{
    use std::path::PathBuf;

    let value = Named_Value_From_String_Arguments(arguments, "--crate");
    let crate_name = Required_Value(value.as_ref(), Name("--crate"), Usage(&Usage_Text()))?;
    let root = Named_Value_From_String_Arguments(arguments, "--root").map_or_else(|| return PathBuf::from("."), PathBuf::from);
    let effort = Effort_From_String_Arguments(arguments)?;
    let backend = Backend_From_String_Arguments(arguments)?;

    return Ok(Command::JudgeRole { crate_name, root, effort, backend });
}

/// `--executor`'s or `--model-backend`'s value, or [`Backend::ClaudeCode`] when neither flag
/// is given -- every caller before either flag existed reached
/// `nomos-agent-executor-claude-code`, so both absent must keep reaching it, byte-identical.
///
/// The two flags used to be one, `--backend`, spelling `claude-code` and `ollama` as if they
/// were peer choices of the same kind. `OD-PACKAGE-013` found they are not -- Ollama is a
/// `ModelBackend`, not a second `AgentExecutor` -- so a caller now names which family it is
/// choosing from, and `--executor ollama` or `--model-backend claude-code` is refused rather
/// than silently accepted the way one flag spanning both could not refuse it.
///
/// # Errors
///
/// Returns a message naming the accepted spelling for whichever flag was given, when its
/// value is not that spelling, or when both flags are given at once -- a call dispatches to
/// exactly one backend, and naming two is not a request either flag alone could satisfy.
fn Backend_From_String_Arguments(arguments: &[String]) -> Result<Backend, String>
{
    let executor = Named_Value_From_String_Arguments(arguments, "--executor");
    let model_backend = Named_Value_From_String_Arguments(arguments, "--model-backend");

    return match (executor, model_backend)
    {
        (Some(_), Some(_)) => Err(format!(
            "--executor and --model-backend both name a backend to dispatch to; a call reaches exactly one, so pass at most one of them.\n\n{}",
            Usage_Text()
        )),
        (Some(text), None) => match text.as_str()
        {
            "claude-code" => Ok(Backend::ClaudeCode),
            other => Err(format!("--executor {other:?} is not one of claude-code.\n\n{}", Usage_Text())),
        },
        (None, Some(text)) => match text.as_str()
        {
            "ollama" => Ok(Backend::Ollama),
            other => Err(format!("--model-backend {other:?} is not one of ollama.\n\n{}", Usage_Text())),
        },
        (None, None) => Ok(Backend::ClaudeCode),
    };
}

/// `--effort`'s value, or [`EffortLevel::BackendDefault`] when the flag is absent --
/// `BackendDefault` alone maps to "omit the flag entirely"
/// (`nomos_agent_executor_claude_code::Effort_Flag`), so an absent `--effort` reaches the
/// subprocess byte-identical to every invocation that predates this flag. The six spellings
/// are `MODEL-ROUTE-004`'s own closed enumeration, kebab-cased the same way `--kind` and
/// `--origin` already kebab-case theirs in `work/parse.rs`.
///
/// # Errors
///
/// Returns a message naming the six accepted spellings when `--effort`'s value is none of
/// them.
fn Effort_From_String_Arguments(arguments: &[String]) -> Result<nomos_model_package::EffortLevel, String>
{
    use nomos_model_package::EffortLevel;

    let Some(text) = Named_Value_From_String_Arguments(arguments, "--effort")
    else
    {
        return Ok(EffortLevel::BackendDefault);
    };

    return match text.as_str()
    {
        "backend-default" => Ok(EffortLevel::BackendDefault),
        "minimal" => Ok(EffortLevel::Minimal),
        "low" => Ok(EffortLevel::Low),
        "medium" => Ok(EffortLevel::Medium),
        "high" => Ok(EffortLevel::High),
        "maximum" => Ok(EffortLevel::Maximum),
        other => Err(format!(
            "--effort {other:?} is not one of backend-default, minimal, low, medium, high, \
             maximum.\n\n{}",
            Usage_Text()
        )),
    };
}

/// [`Usage_Text`]'s content -- a `const` rather than a literal inside that function's own
/// body, so the function measures as one line under this crate's clean-file line-count
/// limit instead of as this whole block: it is one cohesive piece of user-facing text, not
/// sectioned logic, so splitting it into helper functions would fragment a single string
/// for no reader's benefit.
const USAGE_TEXT: &str = "usage: nomos agent <command>\n\
        \n\
        \x20 execute --goal <text> [--effort <level>] [--executor <name> | --model-backend <name>]\n\
        \x20 judge-role --crate <name> [--root <path>] [--effort <level>] [--executor <name> | --model-backend <name>]\n\
        \n\
        `execute` dispatches --goal to the chosen backend as a bounded, tool-free \
        subprocess. It renders the response and, unconditionally, which tool uses (if \
        any) were structurally denied -- the response text is never evidence of what \
        happened, only of what the process said. It does not assemble a WorkResult or a \
        Finding: this is a person's direct question, not a rule's judgment, and there is \
        no subject or rule identity to report one against.\n\
        \n\
        `judge-role` reads --crate's row in --root's README.md band table and its \
        committed tests/contract/surface/<crate>.txt, runs the real \
        nomos_rules::Check_Declared_Role_Matches_Surface rule over them, and dispatches \
        the AgentRequired finding that rule produces -- never a paraphrase -- to execute. \
        --root defaults to the current directory.\n\
        \n\
        --executor and --model-backend name which family of backend to dispatch to; pass \
        at most one, since a call reaches exactly one. Omitting both is --executor \
        claude-code, byte-identical to every invocation before either flag existed. They \
        used to be one flag, --backend, spelling claude-code and ollama as if they were \
        peer choices of the same kind -- OD-PACKAGE-013 found Ollama's real mechanism is \
        a ModelBackendPackage's, not a second AgentExecutor's, so the flag that chooses it \
        says so.\n\
        \n\
        --executor takes claude-code (the only real AgentExecutor). Dispatches through \
        nomos-agent-executor-claude-code under OD-EXECUTOR-001's structural capability \
        boundary: an isolated working directory, no MCP configuration, an allow-list \
        naming no real tool, a $1 budget cap, one --print turn.\n\
        \n\
        --model-backend takes ollama (the only real ModelBackend). Dispatches through \
        nomos-model-backend-ollama, a local model, under OD-EXECUTOR-004's boundary: an \
        isolated working directory, never --experimental/--experimental-yolo/\
        --experimental-websearch (the only flags that open any tool-use capability), a \
        wall-clock timeout in place of a dollar budget. Its own outcome carries only the \
        response text -- no denied-tool-uses line, since there is no tool subsystem to \
        have denied anything from.\n\
        \n\
        --effort takes backend-default, minimal, low, medium, high or maximum -- \
        MODEL-ROUTE-004's closed vocabulary, carried on TaskEnvelope.effort. \
        nomos-agent-executor-claude-code maps it to a real `claude --effort` flag; \
        nomos-model-backend-ollama accepts and ignores it, a real, named gap rather than \
        an invented approximation. Omitting it is backend-default.\n\
        \n\
        exit codes: 0 ok, 2 usage, 5 the executor could not run or answer, 6 the named \
        crate has no README row or no committed surface snapshot";

fn Usage_Text() -> String
{
    return USAGE_TEXT.to_owned();
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Command_From_String_Arguments_Should_Parse_An_Execute_Command()
    {
        let arguments: Vec<String> = ["execute", "--goal", "say hello"].iter().map(|value| return (*value).to_owned()).collect();

        let command =
            Command_From_String_Arguments(&arguments).expect("--goal is the only value `execute` requires, and the list carries it");

        assert_eq!(
            command,
            Command::Execute {
                goal: "say hello".to_owned(),
                effort: nomos_model_package::EffortLevel::BackendDefault,
                backend: Backend::ClaudeCode,
            }
        );
    }

    #[test]
    fn Test_Command_From_String_Arguments_Should_Refuse_An_Empty_Argument_List()
    {
        let error = Command_From_String_Arguments(&[]).expect_err("no verb at all");

        assert!(error.starts_with("usage: nomos agent"), "{error}");
    }

    #[test]
    fn Test_Command_From_String_Arguments_Should_Refuse_An_Unknown_Verb()
    {
        let arguments: Vec<String> = ["not-a-real-verb"].iter().map(|value| return (*value).to_owned()).collect();

        let error = Command_From_String_Arguments(&arguments).expect_err("no such verb");

        assert!(error.contains("unknown command"), "{error}");
    }
}
