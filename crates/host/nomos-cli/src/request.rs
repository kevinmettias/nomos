//! `nomos request` — submitting a feature request, design spec or feature result through the
//! one accept door `OD-SPEC-009` decided.
//!
//! This is a transport and nothing else, per that record: it parses arguments into a
//! [`SubmitRequest`] and dispatches to [`nomos_spec_orchestration::Submit_Corpus_Request`], which constructs
//! the `Submission` and calls `Accept_Submission` itself -- `OD-HOST-005`'s resolution that
//! this verb's real work is `nomos-spec-orchestration`'s `SpecCommand::Submit`, the same
//! store `nomos spec`'s other nine verbs already share. This module keeps only what
//! `nomos-cli::spec` keeps for those nine: argument parsing, exit-code mapping and text
//! rendering. It validates nothing and persists nothing on its own behalf — `OD-SPEC-009`
//! forbids a transport doing either. `--state` and `--contract-version` default when the
//! caller does not give them, and that is not the same thing: a default here is a fact about
//! *this run*, decided before any field is read, and it lands on `Submission`'s own struct
//! fields rather than on a value's origin. Every `--field` value is what the caller typed, so
//! every one of them carries origin `submitted`.
//!
//! # The store this verb writes to does not survive the process
//!
//! Nothing in this repository persists a specification database — `ARC-SPECDB-002` and
//! `OD-SPEC-006` are why. A submission accepted here exists for exactly as long as this
//! invocation runs, so `--into` is the one chance this run has to take the freshness proof
//! `ARC-SPECDB-002` charges a born-structured object with: a stamp taken now, over a store that
//! will not exist a moment later, is still a true stamp of what this store held.

use nomos_spec_orchestration::corpus::{Assemble_Corpus, Assembly, CorpusRequest};
use nomos_spec_orchestration::{SubmitRefusal, SubmitRequest};

mod exit_code;
mod parsing;
mod report;

#[cfg(test)]
mod tests;

pub(crate) use exit_code::ExitCode;
pub(crate) use parsing::Command_From_String_Arguments;
use report::{Report_Accepted, Report_Unwritten};

/// What `nomos request` was asked to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Command
{
    Submit(SubmitRequest),
}

/// Runs a command, writing content to `output` and everything about it to `notes`.
///
/// Returns the exit code rather than exiting, so the whole surface is testable.
pub(crate) fn Run(
    command: &Command,
    request: &CorpusRequest,
    output: &mut impl std::io::Write,
    notes: &mut impl std::io::Write,
) -> ExitCode
{
    return match command
    {
        Command::Submit(submit) => Assemble_And_Submit(submit, request, output, notes),
    };
}

/// Assembles the store `request` names, and dispatches `submit` against it through
/// `nomos-spec-orchestration::Submit_Corpus_Request` -- the same composition-root choice `nomos-cli::spec`
/// already makes for the other nine `SpecCommand` verbs, `nomos_platform_std::StdFileSystem`
/// as the concrete platform.
fn Assemble_And_Submit(
    submit: &SubmitRequest,
    request: &CorpusRequest,
    output: &mut impl std::io::Write,
    notes: &mut impl std::io::Write,
) -> ExitCode
{
    let mut assembly = match Assembled_Corpus(request, notes)
    {
        Ok(assembly) => assembly,
        Err(code) => return code,
    };

    return Submission_Exit_Code(&mut assembly, submit, output, notes);
}

/// The store `request` names, or `ExitCode::StoreError` reported to `notes` when it could not
/// be assembled at all.
fn Assembled_Corpus(request: &CorpusRequest, notes: &mut impl std::io::Write) -> Result<Assembly, ExitCode>
{
    return match Assemble_Corpus(request)
    {
        Ok(assembly) => Ok(assembly),
        Err(error) =>
        {
            let _ = writeln!(notes, "{error}");
            Err(ExitCode::StoreError)
        }
    };
}

/// Dispatches `submit` against `assembly` and renders whichever of the four outcomes it
/// produces.
fn Submission_Exit_Code(
    assembly: &mut Assembly,
    submit: &SubmitRequest,
    output: &mut impl std::io::Write,
    notes: &mut impl std::io::Write,
) -> ExitCode
{
    use nomos_platform_std::StdFileSystem;

    return match nomos_spec_orchestration::Submit_Corpus_Request(assembly, submit, &StdFileSystem)
    {
        Ok(answer) => Report_Accepted(&answer, output),
        Err(SubmitRefusal::Refused(refusal)) =>
        {
            let _ = writeln!(notes, "{refusal}");
            ExitCode::Refused
        }
        Err(SubmitRefusal::Store(error)) =>
        {
            let _ = writeln!(notes, "{error}");
            ExitCode::StoreError
        }
        Err(SubmitRefusal::Written(refusal)) => Report_Unwritten(&refusal, notes),
    };
}
