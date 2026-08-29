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
//! [`ExitCode::Absent`] rather than printing nothing and succeeding. See
//! [`nomos_spec_orchestration::corpus`].
//!
//! # What moved to `nomos-spec-orchestration`
//!
//! [`SpecCommand`], its six `*Request` types and the corpus-assembly plumbing
//! ([`Assemble_Corpus`], [`Assembly`], [`CorpusRequest`]) all moved verbatim to
//! `nomos-spec-orchestration` — `OD-HOST-002`'s family-9 seam, the same shape
//! `nomos-check-orchestration` already built for `nomos check`. All nine verbs are wired
//! through that crate's own [`nomos_spec_orchestration::Run`], built over four increments:
//! [`SpecCommand::Profiles`] and [`SpecCommand::Sources`] (increment 1);
//! [`SpecCommand::Record`], [`SpecCommand::Table`] and [`SpecCommand::Markdown`]
//! (increment 2); [`SpecCommand::Render`] and [`SpecCommand::Freshness`] (increment 3); and
//! [`SpecCommand::Preview`] and [`SpecCommand::Commit`] (increment 4) — the last four generic
//! over [`nomos_platform::FileSystem`], see that crate's own documentation for why those four
//! specifically earned that seam. `spec/verb/{listing,record,table,markdown,render,freshness,
//! editing}.rs` all delegate resolution (and, for `render` and `editing::Commit`, writing) to
//! it and keep only the `ExitCode` a rendering layer is responsible for, over
//! `nomos_platform_std::StdFileSystem` as the concrete platform this composition root
//! chooses — the same choice `nomos-cli::work` already makes for the ledger.
//! [`Note_Absences`] stays here too — it writes text, and writing text is this crate's job,
//! not the orchestration crate's.

mod parsing;
mod reporting;
mod verb;
#[cfg(test)]
mod tests;

pub use parsing::Spec_Command_From_String_Arguments;
use reporting::{Absent_Or, Report_Project_Error, Report_Store_Error};
use verb::{
    Commit_Edit, Empty_Section, EmptySection, Freshness_Of, List_Profiles, List_Sources,
    No_Such_Profile, Preview_Edit, Read_Record, Read_Table, Render_Markdown, Render_Profile,
    Report_Build_Error, Report_Edit_Error,
};

mod exit_code;

pub(crate) use exit_code::ExitCode;
pub(crate) use nomos_spec_orchestration::{
    CommitRequest, EditRequest, FreshnessRequest, RecordRequest, RenderRequest, SpecCommand,
    TableRequest,
};

use crate::arguments::{Name, Named_Value_From_String_Arguments, Named_Values_From_String_Arguments, Required_Value, Usage};
use nomos_spec_orchestration::corpus::{Assemble_Corpus, Assembly, CorpusRequest};
use nomos_spec_project::{Profile, ProjectError, SIDECAR_SUFFIX, Stamp};
use nomos_spec_store::{
    CommitReport, DocumentSource, EditError, EditPreview, NodeSummary, PathMatch, RecordProjection, RowCensus,
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
        return List_Profiles(&mut Channels { output, notes });
    }

    let mut channels = Channels { output, notes };

    let mut assembly = match Assemble_Corpus(request)
    {
        Ok(assembly) => assembly,
        Err(error) => return Report_Store_Error(&error, channels.notes),
    };

    Note_Absences(command, &assembly, channels.notes);

    return Dispatch_Command(command, &mut assembly, &mut channels);
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
fn Dispatch_Command(command: &SpecCommand, assembly: &mut Assembly, channels: &mut Channels<'_>)
    -> ExitCode
{
    return match command
    {
        SpecCommand::Record(request) => Read_Record(assembly, request, channels),
        SpecCommand::Table(request) => Read_Table(assembly, request, channels),
        SpecCommand::Render(request) => Render_Profile(assembly, request, channels),
        SpecCommand::Freshness(request) => Freshness_Of(assembly, request, channels),
        SpecCommand::Markdown(request) => Render_Markdown(assembly, request, channels),
        SpecCommand::Preview(request) => Preview_Edit(assembly, request, channels),
        SpecCommand::Commit(request) => Commit_Edit(assembly, request, channels),
        SpecCommand::Profiles => List_Profiles(channels),
        SpecCommand::Sources => List_Sources(assembly, channels.output),
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
