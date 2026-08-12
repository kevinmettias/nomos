//! Band 10 — the `nomos` binary.
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

mod arguments;
mod check;
mod corpus;
mod spec;
mod work;

use std::path::PathBuf;

/// The environment variable naming the v14 authoring corpus.
///
/// The same variable the corpus-gated tests read, and named here for the same reason they
/// name it: the corpus is a tree this repository does not contain, so nothing can be
/// inferred about where it is. It is spelled once, in the composition root, and passed to
/// [`corpus::Assemble`] as data — which is what lets the whole read surface be exercised
/// on a machine that has no corpus at all.
const CORPUS_VARIABLE: &str = "NOMOS_V14_CORPUS";

fn main() -> std::process::ExitCode
{
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let code = match arguments.split_first()
    {
        Some((group, rest)) if group == "work" => Work(rest),
        Some((group, rest)) if group == "spec" => Spec(rest),
        Some((group, rest)) if group == "check" => Check(rest),
        _ => Usage(),
    };

    return std::process::ExitCode::from(u8::try_from(code).unwrap_or(1));
}

/// The work group: the ledger verbs, over this repository's board.
fn Work(rest: &[String]) -> i32
{
    let mut stdout = std::io::stdout();
    let Ok(command) = work::Parse(rest).inspect_err(|message| eprintln!("{message}"))
    else
    {
        return work::ExitCode::Usage.Value();
    };

    return work::Run(&command, &Work_Directory(), &mut stdout).Value();
}

/// The spec group: reading the specification store and rendering its projections.
fn Spec(rest: &[String]) -> i32
{
    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();
    let Ok(command) = spec::Parse(rest).inspect_err(|message| eprintln!("{message}"))
    else
    {
        return spec::ExitCode::Usage.Value();
    };

    return spec::Run(&command, &Corpus_Request(rest), &mut stdout, &mut stderr).Value();
}

/// The check group: running the rules over a tree and reporting what they find.
fn Check(rest: &[String]) -> i32
{
    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();
    let Ok(command) = check::Parse(rest).inspect_err(|message| eprintln!("{message}"))
    else
    {
        return check::ExitCode::Usage.Value();
    };

    return check::Run(&command, &mut stdout, &mut stderr).Value();
}

/// What the binary answers when it was not told which group it is being asked for.
fn Usage() -> i32
{
    eprintln!(
        "usage: nomos <group> <command>\n\n  \
         work   coordinate concurrent work over this repository\n  \
         spec   read the specification store and render its projections\n  \
         check  run the rules over a tree and report what they find"
    );

    return work::ExitCode::Usage.Value();
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

/// Which corpus to assemble the specification store from.
///
/// `--corpus` wins over the environment, so a caller can read one tree without changing
/// the variable every other tool on the machine is reading. Both are optional: neither
/// being given is a supported state and produces an absence rather than an error, because
/// the governing records are embedded and there are real questions this binary can still
/// answer.
fn Corpus_Request(rest: &[String]) -> corpus::CorpusRequest
{
    return corpus::CorpusRequest {
        variable: CORPUS_VARIABLE.to_owned(),
        root: arguments::Named_Value(rest, "--corpus")
            .map(PathBuf::from)
            .or_else(|| return std::env::var_os(CORPUS_VARIABLE).map(PathBuf::from)),
        revision: arguments::Named_Value(rest, "--corpus-revision")
            .unwrap_or_else(|| return corpus::DEFAULT_REVISION.to_owned()),
    };
}
