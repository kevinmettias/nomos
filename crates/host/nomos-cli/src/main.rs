//! Zone: Host — the `nomos` binary.
//!
//! One binary with subcommands, not several binaries. A separate `nomos-work` would
//! grow its own flags, its own output conventions and eventually its own idea of what a
//! claim is — and the architecture's rule that every user-visible action has one
//! canonical service would have been violated by the first milestone.
//!
//! Everything here is a projection. The CLI parses arguments, calls a library, and
//! renders the result; it decides nothing. When the service layer arrives these
//! subcommands become renderers over service contracts rather than direct library
//! calls, and the shape will not have to change.
//!
//! This file is also the only place in the workspace that reads the environment. A module
//! that reaches for a variable itself is a module that behaves differently under test
//! than in a terminal, and the corpus variable in particular is one whose *absence* is a
//! reportable fact — so it is read here, once, and handed on as a value.

#![forbid(unsafe_code)]

mod agent;
mod arguments;
mod check;
mod correct;
mod gate;
mod request;
mod spec;
mod vacuity;
mod work;
mod workflow;

use std::path::PathBuf;

/// The environment variable naming the v14 authoring corpus.
///
/// The same variable the corpus-gated tests read, and named here for the same reason they
/// name it: the corpus is a tree this repository does not contain, so nothing can be
/// inferred about where it is. It is spelled once, in the composition root, and passed to
/// [`nomos_spec_orchestration::corpus::Assemble_Corpus`] as data — which is what lets the whole
/// read surface be exercised on a machine that has no corpus at all.
const CORPUS_VARIABLE: &str = "NOMOS_V14_CORPUS";

fn main() -> std::process::ExitCode
{
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    // Routed through `vacuity::Named` rather than compared here: `vacuity::Group` is the
    // closed set every group's no-vacuous-success stance is declared against
    // (`vacuity.rs`, `OD-GATE-003`), and matching it with no wildcard arm means a group
    // this binary does not yet know how to run cannot be added here without also being
    // named there.
    let code = match arguments.split_first().and_then(|(group, rest)| {
        return vacuity::Group_Named(group).map(|group| return (group, rest));
    })
    {
        Some((vacuity::Group::Work, rest)) => Run_Work_Group(rest),
        Some((vacuity::Group::Spec, rest)) => Run_Spec_Group(rest),
        Some((vacuity::Group::Check, rest)) => Run_Check_Group(rest),
        Some((vacuity::Group::Request, rest)) => Run_Request_Group(rest),
        Some((vacuity::Group::Gate, rest)) => Run_Gate_Group(rest),
        Some((vacuity::Group::Agent, rest)) => Run_Agent_Group(rest),
        Some((vacuity::Group::Correct, rest)) => Run_Correct_Group(rest),
        Some((vacuity::Group::Workflow, rest)) => Run_Workflow_Group(rest),
        None => Usage(),
    };

    return std::process::ExitCode::from(u8::try_from(code).unwrap_or(1));
}

/// The work group: the ledger verbs, over this repository's board.
fn Run_Work_Group(rest: &[String]) -> i32
{
    let mut stdout = std::io::stdout();
    let Ok(command) = work::Work_Command_From_String_Arguments(rest).inspect_err(|message| eprintln!("{message}"))
    else
    {
        return work::ExitCode::Usage.Value();
    };

    return work::Run(&command, &Work_Directory(), &mut stdout).Value();
}

/// Where the ledger lives.
///
/// Overridable so that tests and tools can point at a scratch ledger without changing
/// directory, which is what makes the whole surface testable from one process.
fn Work_Directory() -> PathBuf
{
    return std::env::var_os("NOMOS_WORK_DIR")
        .map_or_else(|| PathBuf::from("work"), PathBuf::from);
}

/// The spec group: reading the specification store and rendering its projections.
fn Run_Spec_Group(rest: &[String]) -> i32
{
    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();
    let Ok(command) = spec::Spec_Command_From_String_Arguments(rest).inspect_err(|message| eprintln!("{message}"))
    else
    {
        return spec::ExitCode::Usage.Value();
    };

    return spec::Run(&command, &Corpus_Request(rest), &mut stdout, &mut stderr).Value();
}

/// The check group: running the rules over a tree and reporting what they find.
fn Run_Check_Group(rest: &[String]) -> i32
{
    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();
    let Ok(command) = check::Check_Command_From_String_Arguments(rest).inspect_err(|message| eprintln!("{message}"))
    else
    {
        return check::ExitCode::Usage.Value();
    };

    return check::Run(&command, &mut stdout, &mut stderr).Value();
}

/// The request group: submitting a feature request, design spec or feature result through the
/// one accept function `OD-SPEC-009` decided.
fn Run_Request_Group(rest: &[String]) -> i32
{
    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();
    let Ok(command) = request::Command_From_String_Arguments(rest).inspect_err(|message| eprintln!("{message}"))
    else
    {
        return request::ExitCode::Usage.Value();
    };

    return request::Run(&command, &Corpus_Request(rest), &mut stdout, &mut stderr).Value();
}

/// The gate group: composing this repository's rule registry and reporting what it holds.
fn Run_Gate_Group(rest: &[String]) -> i32
{
    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();
    let Ok(command) = gate::Gate_Invocation_From_String_Arguments(rest).inspect_err(|message| eprintln!("{message}"))
    else
    {
        return gate::ExitCode::Usage.Value();
    };

    return gate::Run(&command, &mut stdout, &mut stderr).Value();
}

/// The agent group: dispatching a task to the first real `AgentExecutor`.
fn Run_Agent_Group(rest: &[String]) -> i32
{
    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();
    let Ok(command) = agent::Command_From_String_Arguments(rest).inspect_err(|message| eprintln!("{message}"))
    else
    {
        return agent::ExitCode::Usage.Value();
    };

    return agent::Run(&command, &mut stdout, &mut stderr).Value();
}

/// The correct group: the corrections product vertical's first real Finding-to-Commit
/// path, over a tree.
fn Run_Correct_Group(rest: &[String]) -> i32
{
    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();
    let Ok(command) = correct::Parse(rest).inspect_err(|message| eprintln!("{message}"))
    else
    {
        return correct::ExitCode::Usage.Value();
    };

    return correct::Run(&command, &mut stdout, &mut stderr).Value();
}

/// The workflow group: dispatching one step of an ordered `WorkflowStep` sequence.
fn Run_Workflow_Group(rest: &[String]) -> i32
{
    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();
    let Ok(command) = workflow::Command_From_String_Arguments(rest).inspect_err(|message| eprintln!("{message}"))
    else
    {
        return workflow::ExitCode::Usage.Value();
    };

    return workflow::Run(&command, &mut stdout, &mut stderr).Value();
}

/// What the binary answers when it was not told which group it is being asked for.
fn Usage() -> i32
{
    eprintln!(
        "usage: nomos <group> <command>\n\n  \
         work     coordinate concurrent work over this repository\n  \
         spec     read the specification store and render its projections\n  \
         check    run the rules over a tree and report what they find\n  \
         request  submit a feature request, design spec or feature result\n  \
         gate     compose this gate's rule registry and report what it holds\n  \
         agent    dispatch a task to the first real AgentExecutor\n  \
         correct  build and commit a real correction for one real finding\n  \
         workflow dispatch one step of an ordered WorkflowStep sequence"
    );

    return work::ExitCode::Usage.Value();
}

/// Which corpus to assemble the specification store from.
///
/// `--corpus` wins over the environment, so a caller can read one tree without changing
/// the variable every other tool on the machine is reading. Both are optional: neither
/// being given is a supported state and produces an absence rather than an error, because
/// the governing records are embedded and there are real questions this binary can still
/// answer.
fn Corpus_Request(rest: &[String]) -> nomos_spec_orchestration::corpus::CorpusRequest
{
    return nomos_spec_orchestration::corpus::CorpusRequest {
        variable: CORPUS_VARIABLE.to_owned(),
        root: arguments::Named_Value_From_String_Arguments(rest, "--corpus")
            .map(PathBuf::from)
            .or_else(|| return std::env::var_os(CORPUS_VARIABLE).map(PathBuf::from)),
        revision: arguments::Named_Value_From_String_Arguments(rest, "--corpus-revision")
            .unwrap_or_else(|| return nomos_spec_orchestration::corpus::DEFAULT_REVISION.to_owned()),
    };
}
