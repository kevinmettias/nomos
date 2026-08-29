//! What `nomos gate` was asked for.

use super::{FindingQuery, GateCommand, GateInvocation, Named_Value_From_String_Arguments, PathBuf};
use crate::arguments::{Name, Named_Values_From_String_Arguments, Required_Value, Usage};
use nomos_contracts::RuleId;
use nomos_gate_orchestration::{RuleSelector, ScopeSelector};

/// The flags this group accepts besides `--root`, `--rule` and `--location`.
const KNOWN_ARGUMENTS: [&str; 5] = ["--root", "--include", "--exclude", "--rule", "--location"];

pub(super) const USAGE: &str = "usage: nomos gate plan    [--root <path>] [--include <path>]… \
[--exclude <path>]… [--rule <id>]…\n       \
     nomos gate run     [--root <path>] [--include <path>]… [--exclude <path>]… [--rule <id>]…\n       \
     nomos gate explain [--root <path>] --rule <id> --location <path>\n\n\
     plan composes this gate's rule registry and reports what it holds.\n\
     run walks the tree, judges it, and reports a real disposition.\n\
     explain walks the tree, judges it, and reports what one named finding looks like and \
whether it would block.\n\n\
     --include/--exclude narrow which files `run` judges, by path prefix; repeat for \
several. For plan/run, --rule (repeatable) narrows which rules' findings can fail the \
build. For explain, --rule names the one rule whose finding to explain -- required, not \
repeatable -- alongside --location, one of that finding's own locations, also \
required.\n\n\
     exit codes: 0 clean (the plan was composed, nothing judged can fail a build, or \
explain's finding was not found or would not block),\n\
     \x20           1 at least one finding can fail a build, or explain's finding would, \
2 usage,\n\
     \x20           5 this build's own composition is self-contradictory, or the tree could \
not be read,\n\
     \x20           6 nothing was judged: the walk found no source, or no fact was \
materialized for any of it";

/// Parses the group's arguments.
///
/// `compare` is `ARC-ROADMAP-001`'s one remaining named verb with no real implementation
/// behind it -- see `nomos_gate_orchestration`'s own `lib.rs` doc -- so only `plan`, `run`
/// and `explain` are recognized here. Accepting an unimplemented verb name and silently
/// running `plan` instead would answer a question nobody asked; refusing it as usage is
/// the same "no invented shape ahead of a real body" choice the crate itself already made.
///
/// # Errors
///
/// Returns the usage message when the verb is missing or unrecognized, `explain` is
/// missing `--rule` or `--location`, or an argument is not understood.
pub fn Gate_Invocation_From_String_Arguments(arguments: &[String]) -> Result<GateInvocation, String>
{
    let Some((verb, rest)) = arguments.split_first()
    else
    {
        return Err(USAGE.to_owned());
    };

    Known_Verb(verb)?;
    No_Unknown_Argument(rest)?;

    let root = Named_Value_From_String_Arguments(rest, "--root").map_or_else(|| return PathBuf::from("."), PathBuf::from);

    if verb == "explain"
    {
        return Explain_Invocation(rest, root);
    }

    let command = Plan_Or_Run_Command(root, rest);

    return Ok(if verb == "run" { GateInvocation::Run(command) } else { GateInvocation::Plan(command) });
}

/// Refuses anything but the three verbs this group implements today.
fn Known_Verb(verb: &str) -> Result<(), String>
{
    let is_unknown_verb = verb != "plan" && verb != "run" && verb != "explain";
    if is_unknown_verb
    {
        return Err(format!("unknown verb `{verb}`.\n\n{USAGE}"));
    }

    return Ok(());
}

/// Refuses any flag but `--root`, `--include`, `--exclude`, `--rule` and `--location`.
fn No_Unknown_Argument(rest: &[String]) -> Result<(), String>
{
    if let Some(unknown) = rest
        .iter()
        .find(|argument| return argument.starts_with('-') && !KNOWN_ARGUMENTS.contains(&argument.as_str()))
    {
        return Err(format!("unknown argument `{unknown}`.\n\n{USAGE}"));
    }

    return Ok(());
}

/// `explain`'s own required `--rule`/`--location`, read as single values rather than
/// `plan`/`run`'s repeatable `--rule`: naming one finding needs exactly one rule, not a
/// selector, so this does not populate `GateCommand::rules` at all -- `Explain_Gate`
/// ignores it regardless, and populating it from the same flag that also names the query
/// would read as a selector nobody asked for.
fn Explain_Invocation(rest: &[String], root: PathBuf) -> Result<GateInvocation, String>
{
    let rule = Required_Value(Named_Value_From_String_Arguments(rest, "--rule").as_ref(), Name("--rule"), Usage(USAGE))?;
    let location = Required_Value(Named_Value_From_String_Arguments(rest, "--location").as_ref(), Name("--location"), Usage(USAGE))?;

    let command = GateCommand { root, ..GateCommand::default() };
    let query = FindingQuery { rule: RuleId::New(rule), location };

    return Ok(GateInvocation::Explain { command, query });
}

/// Builds `plan`/`run`'s shared command from `rest`'s flags, now that the verb and its
/// argument shape are already known.
fn Plan_Or_Run_Command(root: PathBuf, rest: &[String]) -> GateCommand
{
    return GateCommand {
        root,
        scope: ScopeSelector {
            include: Named_Values_From_String_Arguments(rest, "--include"),
            exclude: Named_Values_From_String_Arguments(rest, "--exclude"),
        },
        rules: RuleSelector { include: Named_Values_From_String_Arguments(rest, "--rule").into_iter().map(RuleId::New).collect() },
        // No flag authors a Suppression, a BaselineDebt or a RuleCalibration yet -- see
        // `nomos_gate_orchestration::SuppressionPolicy`'s own doc for why inventing one now
        // would be premature.
        suppressions: nomos_gate_orchestration::SuppressionPolicy::default(),
        baseline: nomos_gate_orchestration::BaselinePolicy::default(),
        adoption: nomos_gate_orchestration::AdoptionPolicy::default(),
        // No flag authors a non-default CoveragePolicy yet -- see
        // `nomos_gate_orchestration::CoveragePolicy`'s own doc for why.
        coverage: nomos_gate_orchestration::CoveragePolicy::default(),
        // No flag authors a ModelExecutionProfile yet -- see
        // `nomos_gate_orchestration::GateCommand::model`'s own doc for why.
        model: None,
    };
}
