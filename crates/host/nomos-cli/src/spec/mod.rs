//! `nomos spec` — the specification store, from a terminal.
//!
//! Four phases built a store, a preservation ledger and fourteen projection profiles, and
//! until this group existed there was no way to ask any of them a question. Phase 2's
//! stated payoff was to ask what the canonical domain model said, verbatim, and get the
//! real rows; that is [`SpecCommand::Record`] and [`SpecCommand::Table`]. Phase 4 shipped
//! renderers nothing ran; that is [`SpecCommand::Render`].
//!
//! # Two streams, on purpose
//!
//! Content goes to standard output and everything *about* it goes to standard error. So
//! `nomos spec record --id D-129 > D-129.md` writes the record's bytes and nothing else,
//! and the file it produces hashes to what the store holds. A header line mixed into the
//! content would make "verbatim" a claim about the interesting part rather than about the
//! output, and the caller would have to know which lines to strip.
//!
//! # An absence is not an empty answer
//!
//! The store is assembled per invocation and most of it comes from a corpus that is not in
//! this repository. Every command therefore reports what the store was missing, and a
//! command whose answer is empty *because* something was missing says so and exits
//! [`ExitCode::Absent`] rather than printing nothing and succeeding. See [`crate::corpus`].

mod parsing;
mod reporting;
mod verb;
#[cfg(test)]
mod tests;

pub use parsing::Parse;
use reporting::{Absent_Or, Report_Project_Error, Report_Store_Error, Vanished};
use verb::{
    Commit, Empty_Section, EmptySection, Freshness_Of, Markdown, Placed, Preview, Profiles,
    Record, Render, Report_Build_Error, Report_Edit_Error, Resolved, Sources, Table,
};

mod exit_code;
mod request;
mod spec_command;

pub(crate) use exit_code::ExitCode;
pub(crate) use request::{
    CommitRequest, EditRequest, FreshnessRequest, RecordRequest, RenderRequest, TableRequest,
};
pub(crate) use spec_command::SpecCommand;

use crate::arguments::{Named_Value, Named_Values, Required};
use crate::corpus::{Assemble, Assembly, CorpusRequest};
use nomos_spec_project::{Build, Catalogue, Check, Output, Profile, ProjectError, SIDECAR_SUFFIX, Stamp};
use nomos_spec_store::{
    CommitReport, DocumentSource, EditError, EditPreview, NodeSummary, PathMatch, RecordProjection, RowCensus, RowScope,
    StoreError, TableLine,
};
use std::path::{Path, PathBuf};

/// Where a command writes: content to `output`, everything about it to `notes`.
///
/// The two travel together through every verb because keeping them apart is the surface's
/// whole point — a redirect has to capture exactly what the store holds and nothing about
/// it. Passing them as one value is also what keeps the verbs inside the parameter budget.
struct Channels<'a>
{
    /// What the store holds.
    output: &'a mut dyn std::io::Write,
    /// Everything about it.
    notes: &'a mut dyn std::io::Write,
}

/// Both halves of a rendered output, as they are on disk.
struct Rendered<'a>
{
    /// The projection itself.
    body: &'a str,
    /// The stamp beside it.
    sidecar: &'a str,
}

/// Runs a command, writing content to `output` and everything about it to `notes`.
///
/// Returns the exit code rather than exiting, so the whole surface is testable.
pub fn Run(
    command: &SpecCommand,
    request: &CorpusRequest,
    output: &mut impl std::io::Write,
    notes: &mut impl std::io::Write,
) -> ExitCode
{
    if let SpecCommand::Profiles = command
    {
        // The catalogue is embedded and answers without a store, so building one would
        // make listing the profiles fail on a machine that cannot open a database.
        return Profiles(&mut Channels { output, notes });
    }

    let mut channels = Channels { output, notes };

    let mut assembly = match Assemble(request)
    {
        Ok(assembly) => assembly,
        Err(error) => return Report_Store_Error(&error, channels.notes),
    };

    Note_Absences(command, &assembly, channels.notes);

    return Dispatch(command, &mut assembly, &mut channels);
}

/// What the store was missing, said on the way through.
///
/// On every command that is not about the absences already. A run over a whole corpus
/// prints nothing here, so the note's absence is itself the signal — the same shape
/// `OD-GATE-001` settled on for the corpus gates.
fn Note_Absences(command: &SpecCommand, assembly: &Assembly, notes: &mut dyn std::io::Write)
{
    if assembly.Is_Complete() || matches!(command, SpecCommand::Sources)
    {
        return;
    }

    let _ = writeln!(notes, "{}", assembly.Describe_Absences());
}

/// The verb itself, against a store that is already assembled and already reported on.
fn Dispatch(command: &SpecCommand, assembly: &mut Assembly, channels: &mut Channels<'_>)
    -> ExitCode
{
    return match command
    {
        SpecCommand::Record(request) => Record(assembly, request, channels),
        SpecCommand::Table(request) => Table(assembly, request, channels),
        SpecCommand::Render(request) => Render(assembly, request, channels),
        SpecCommand::Freshness(request) => Freshness_Of(assembly, request, channels),
        SpecCommand::Markdown(request) => Markdown(assembly, request, channels),
        SpecCommand::Preview(request) => Preview(assembly, request, channels),
        SpecCommand::Commit(request) => Commit(assembly, request, channels),
        SpecCommand::Profiles => Profiles(channels),
        SpecCommand::Sources => Sources(assembly, channels.output),
    };
}

/// What a `preview` or `commit` against this binary's store does and does not persist.
///
/// Said on every run rather than left to a record nobody has open. The store is assembled per
/// invocation and thrown away, so the transaction proves the edit is admissible and the file is
/// what survives it — and for a record this binary embeds, the seed keeps reading its own
/// compiled-in copy until the crate is rebuilt. `OD-SPEC-006` is why that is the arrangement
/// rather than a defect.
const EPHEMERAL: &str = "this store was assembled for this invocation and is now gone: the \
                         transaction is what checked the edit, and the file is what persists \
                         it. A record embedded in this binary is re-seeded from the copy \
                         compiled into it until nomos-spec-store is rebuilt. See OD-SPEC-006.";
