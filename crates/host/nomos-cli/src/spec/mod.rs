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

mod exit_code;
mod spec_command;
mod record_request;
mod table_request;
mod render_request;
mod freshness_request;
mod edit_request;
mod commit_request;

pub use exit_code::ExitCode;
pub use spec_command::SpecCommand;
pub use record_request::RecordRequest;
pub use table_request::TableRequest;
pub use render_request::RenderRequest;
pub use freshness_request::FreshnessRequest;
pub use edit_request::EditRequest;
pub use commit_request::CommitRequest;

use crate::arguments::Named_Value;
use crate::arguments::Named_Values;
use crate::arguments::Required;
use crate::corpus::Assemble;
use crate::corpus::Assembly;
use crate::corpus::CorpusRequest;
use nomos_spec_project::{
    Build, Catalogue, Check, Output, Profile, ProjectError, SIDECAR_SUFFIX, Stamp,
};
use nomos_spec_store::{
    CommitReport, DocumentSource, EditError, EditPreview, NodeSummary, PathMatch,
    RecordProjection, RowCensus, RowScope, StoreError, TableLine,
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

/// A section that selected nothing, as the projection machinery reported it.
struct EmptySection<'a>
{
    /// The profile being built.
    profile: &'a str,
    /// The section of it that came back empty.
    section: &'a str,
    /// What that section selects.
    content: &'static str,
}

/// Both halves of a rendered output, as they are on disk.
struct Rendered<'a>
{
    /// The projection itself.
    body: &'a str,
    /// The stamp beside it.
    sidecar: &'a str,
}

/// Parses `nomos spec` arguments.
///
/// # Errors
///
/// Returns a message naming what was wrong and what was expected.
pub fn Parse(arguments: &[String]) -> Result<SpecCommand, String>
{
    let Some(verb) = arguments.first()
    else
    {
        return Err(Usage_Text());
    };

    return match verb.as_str()
    {
        "record" => Parse_Record(arguments),
        "table" => Parse_Table(arguments),
        "render" => Parse_Render(arguments),
        "freshness" => Parse_Freshness(arguments),
        "markdown" => Parse_Markdown(arguments),
        "preview" => Parse_Preview(arguments),
        "commit" => Parse_Commit(arguments),
        "profiles" => Ok(SpecCommand::Profiles),
        "sources" => Ok(SpecCommand::Sources),
        other => Err(format!("unknown command `{other}`.\n\n{}", Usage_Text())),
    };
}

/// A flag with no default, or a message naming it beside the usage.
fn Required_Value(arguments: &[String], name: &str) -> Result<String, String>
{
    let value = Named_Value(arguments, name);

    return Required(value.as_ref(), name, &Usage_Text());
}

/// A flag with no default, read as a path.
fn Required_Path(arguments: &[String], name: &str) -> Result<PathBuf, String>
{
    let value = Required_Value(arguments, name)?;

    return Ok(PathBuf::from(value));
}

/// Which record a `record` or `markdown` run is about.
fn Parse_Record_Request(arguments: &[String]) -> Result<RecordRequest, String>
{
    return Ok(RecordRequest {
        id: Required_Value(arguments, "--id")?,
        revision: Named_Value(arguments, "--revision"),
    });
}

/// The edit a `preview` or `commit` run carries.
fn Parse_Edit_Request(arguments: &[String]) -> Result<EditRequest, String>
{
    return Ok(EditRequest {
        id: Required_Value(arguments, "--id")?,
        from: Required_Path(arguments, "--from")?,
        rename: Named_Value(arguments, "--rename"),
    });
}

fn Parse_Record(arguments: &[String]) -> Result<SpecCommand, String>
{
    let request = Parse_Record_Request(arguments)?;

    return Ok(SpecCommand::Record(request));
}

fn Parse_Table(arguments: &[String]) -> Result<SpecCommand, String>
{
    let block = Named_Value(arguments, "--block");
    let table = Named_Value(arguments, "--table");

    return Ok(SpecCommand::Table(TableRequest {
        document: Required_Value(arguments, "--document")?,
        block: Ordinal(block.as_ref(), "--block")?,
        table: Ordinal(table.as_ref(), "--table")?,
        revision: Named_Value(arguments, "--revision"),
    }));
}

fn Parse_Render(arguments: &[String]) -> Result<SpecCommand, String>
{
    return Ok(SpecCommand::Render(RenderRequest {
        profile: Required_Value(arguments, "--profile")?,
        into: Required_Path(arguments, "--into")?,
        subject: Named_Value(arguments, "--subject"),
    }));
}

fn Parse_Freshness(arguments: &[String]) -> Result<SpecCommand, String>
{
    return Ok(SpecCommand::Freshness(FreshnessRequest {
        into: Required_Path(arguments, "--into")?,
        profile: Named_Value(arguments, "--profile"),
        require: Named_Values(arguments, "--require"),
    }));
}

fn Parse_Markdown(arguments: &[String]) -> Result<SpecCommand, String>
{
    let request = Parse_Record_Request(arguments)?;

    return Ok(SpecCommand::Markdown(request));
}

fn Parse_Preview(arguments: &[String]) -> Result<SpecCommand, String>
{
    let request = Parse_Edit_Request(arguments)?;

    return Ok(SpecCommand::Preview(request));
}

fn Parse_Commit(arguments: &[String]) -> Result<SpecCommand, String>
{
    let edit = Parse_Edit_Request(arguments)?;
    let into = Named_Value(arguments, "--into");

    return Ok(SpecCommand::Commit(CommitRequest {
        edit,
        into: into.map_or_else(|| return PathBuf::from("."), PathBuf::from),
    }));
}

/// A whole-number flag, or a message saying what was given instead.
fn Ordinal(value: Option<&String>, name: &str) -> Result<Option<u32>, String>
{
    let Some(text) = value
    else
    {
        return Ok(None);
    };

    return text
        .parse::<u32>()
        .map(Some)
        .map_err(|_| return format!("{name} takes a whole number; `{text}` is not one"));
}

fn Usage_Text() -> String
{
    return "usage: nomos spec <command>\n\
            \n\
            \x20 record    --id <node-id> [--revision <label>]\n\
            \x20 table     --document <path|name> [--block <n>] [--table <n>] \
            [--revision <label>]\n\
            \x20 render    --profile <id> --into <directory> [--subject <node-id>]\n\
            \x20 freshness --into <directory> [--profile <id>] [--require <id> …]\n\
            \x20 markdown  --id <node-id> [--revision <label>]\n\
            \x20 preview   --id <node-id> --from <file> [--rename <path>]\n\
            \x20 commit    --id <node-id> --from <file> [--rename <path>]\n\
            \x20 profiles\n\
            \x20 sources\n\
            \n\
            common: [--corpus <directory>] [--corpus-revision <label>]\n\
            \n\
            `record` and `table` write content to stdout and everything about it to \
            stderr, so a redirect captures exactly what the store holds.\n\
            \n\
            `record` prints the bytes the store was given; `markdown` renders the record \
            back out of the store's own rows, which is the round trip D-129 decides. \
            `commit` refuses to write an edit it has not previewed, and prints the preview \
            it did.\n\
            \n\
            `freshness` reports on the outputs it finds; a build root holding a subset is \
            normal and not a finding. `--require` names an output this repository promises \
            to ship, and its absence becomes a failure rather than a line saying it was \
            not built here. Repeat it per profile.\n\
            \n\
            the store is assembled per invocation: this repository's governing records \
            are embedded, and the v14 corpus is read from --corpus or the environment. A \
            corpus that is not there is reported as an absence rather than as a shorter \
            answer — run `nomos spec sources` to see what a store holds.\n\
            \n\
            exit codes: 0 ok, 1 not found, 2 usage, 5 store error, 6 a source was absent, \
            7 the output could not be written, 8 an output on disk has drifted, 9 an edit \
            was refused"
        .to_owned();
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

/// `D-129`'s round trip, read half: the record as the store's own rows render it.
///
/// Deliberately a second command rather than a flag on [`SpecCommand::Record`]. `record`
/// answers *what bytes went in*, which is a preservation question; this answers *what the
/// store can write back out*, which is an authoring one. A flag would make the two look like
/// two formats of one answer, and the whole point of `P9-AUTHORING` is that they were not the
/// same answer until now.
fn Markdown(
    assembly: &Assembly,
    request: &RecordRequest,
    channels: &mut Channels<'_>,
) -> ExitCode
{
    let revision = request.revision.as_deref();
    let projection = match assembly.store.Record_Markdown(&request.id, revision)
    {
        Ok(projection) => projection,
        Err(error) => return Report_Edit_Error(assembly, &error, channels.notes),
    };

    let _ = write!(channels.output, "{}", projection.markdown);
    let _ = writeln!(
        channels.notes,
        "{}: {} at revision {}, rendered from the store's rows as {}",
        request.id, projection.path, projection.revision, projection.projected_hash
    );

    return Reproducible(&projection, channels.notes);
}

/// Whether the store can write back the bytes it was given, said out loud when it cannot.
fn Reproducible(projection: &RecordProjection, notes: &mut dyn std::io::Write) -> ExitCode
{
    if projection.Matches_Source()
    {
        return ExitCode::Ok;
    }

    let _ = writeln!(
        notes,
        "the bytes this document was ingested from hash to {}, so the store cannot reproduce \
         them. A v14 record carrying a byte order mark is the ordinary reason (D-131), and an \
         edit through this surface is refused until that is settled rather than silently \
         normalised.",
        projection.source_hash
    );

    return ExitCode::Stale;
}

/// The preview, printed, changing nothing.
fn Preview(assembly: &Assembly, request: &EditRequest, channels: &mut Channels<'_>) -> ExitCode
{
    let staged = match Staged_Text(&request.from, channels.notes)
    {
        Ok(text) => text,
        Err(code) => return code,
    };

    let preview = match Previewed(assembly, request, &staged, channels.notes)
    {
        Ok(preview) => preview,
        Err(code) => return code,
    };

    let _ = writeln!(channels.output, "{}", preview.Describe());
    let _ = writeln!(channels.notes, "nothing was written. {EPHEMERAL}");

    return ExitCode::Ok;
}

/// The preview and then the commit, in that order, because the other order is not available.
fn Commit(
    assembly: &mut Assembly,
    request: &CommitRequest,
    channels: &mut Channels<'_>,
) -> ExitCode
{
    let text = match Staged_Text(&request.edit.from, channels.notes)
    {
        Ok(text) => text,
        Err(code) => return code,
    };

    let preview = match Previewed(assembly, &request.edit, &text, channels.notes)
    {
        Ok(preview) => preview,
        Err(code) => return code,
    };
    let _ = writeln!(channels.output, "{}", preview.Describe());

    return Committed(assembly, &Staged { preview, text }, request, channels);
}

/// An edit the store has already checked, and the bytes it was checked against.
///
/// The two travel together because the preview is what proves the edit admissible and the
/// text is what lands on disk, and writing one without the other is the half-done commit
/// this surface exists to prevent.
struct Staged
{
    /// The edit as the store checked it.
    preview: EditPreview,
    /// The bytes that were staged.
    text: String,
}

/// The edit written where the author expects it, and the round trip closed behind it.
fn Committed(
    assembly: &mut Assembly,
    staged: &Staged,
    request: &CommitRequest,
    channels: &mut Channels<'_>,
) -> ExitCode
{
    let renamed = staged.preview.Rename().map(|(before, _)| return before.to_owned());
    let report = match assembly.store.Commit_Edit(&staged.preview)
    {
        Ok(report) => report,
        Err(error) => return Report_Edit_Error(assembly, &error, channels.notes),
    };

    let destination = request.into.join(&report.path);
    let vacated = renamed.map(|path| return request.into.join(path));
    if let Some(code) = Written(&destination, &staged.text, vacated.as_deref(), channels)
    {
        return code;
    }

    Report_Commit(&report, channels.output);

    return Reproduced(assembly, &request.edit.id, &staged.text, channels);
}

/// What the commit changed, counted.
fn Report_Commit(report: &CommitReport, output: &mut dyn std::io::Write)
{
    let _ = writeln!(
        output,
        "committed {} to {}: {} block(s), {} removed, {} relation(s) added, {} removed",
        report.node_id,
        report.path,
        report.blocks,
        report.blocks_removed,
        report.relations_added,
        report.relations_removed
    );
}

/// Writes the committed record where the author expects it, and vacates the path a rename
/// left.
///
/// Deleting the old file is part of the rename rather than left to the author: two files
/// declaring one identifier is what `Test_Every_Canonical_Record_On_Disk_Should_Be_Governing`
/// would report as a phantom record, and a rename that needs a follow-up step is a rename
/// somebody will half-do.
fn Written(
    destination: &Path,
    staged: &str,
    vacated: Option<&Path>,
    channels: &mut Channels<'_>,
) -> Option<ExitCode>
{
    if let Some(code) = Placed(destination, staged, channels.notes)
    {
        return Some(code);
    }

    if let Some(old) = vacated
    {
        Vacated(destination, old, channels);
    }

    return None;
}

/// The record's own bytes, under a directory that is made if it is not there.
fn Placed(destination: &Path, staged: &str, notes: &mut dyn std::io::Write) -> Option<ExitCode>
{
    if let Some(parent) = destination.parent()
        && let Err(error) = std::fs::create_dir_all(parent)
    {
        let _ = writeln!(notes, "cannot create {}: {error}", parent.display());

        return Some(ExitCode::Unwritable);
    }

    if let Err(error) = std::fs::write(destination, staged)
    {
        let _ = writeln!(notes, "cannot write {}: {error}", destination.display());

        return Some(ExitCode::Unwritable);
    }

    return None;
}

/// The path a rename left behind, removed.
///
/// A failure here is reported and not fatal: the new file is already written, so the run
/// succeeded at the edit and failed at the tidying, and saying so is more use than an exit
/// code that suggests nothing landed.
fn Vacated(destination: &Path, old: &Path, channels: &mut Channels<'_>)
{
    match std::fs::remove_file(old)
    {
        Ok(()) => drop(writeln!(channels.output, "vacated {}", old.display())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
        Err(error) => drop(writeln!(
            channels.notes,
            "{} was written and {} could not be removed ({error}), so two files now declare \
             this record",
            destination.display(),
            old.display()
        )),
    }
}

/// The round trip, closed on the way out: the store is asked to render what was just
/// committed, and the answer is compared with it.
fn Reproduced(
    assembly: &Assembly,
    id: &str,
    staged: &str,
    channels: &mut Channels<'_>,
) -> ExitCode
{
    let _ = writeln!(channels.notes, "{EPHEMERAL}");

    return match assembly.store.Record_Markdown(id, None)
    {
        Ok(projection) if projection.markdown == staged =>
        {
            let _ = writeln!(
                channels.output,
                "the store renders it back as the same bytes ({})",
                projection.projected_hash
            );

            ExitCode::Ok
        }
        Ok(projection) =>
        {
            let _ = writeln!(
                channels.notes,
                "the commit succeeded and the store renders {} rather than what was committed, \
                 so the round trip does not close here",
                projection.projected_hash
            );

            ExitCode::Stale
        }
        Err(error) => Report_Edit_Error(assembly, &error, channels.notes),
    };
}

fn Staged_Text(from: &Path, notes: &mut dyn std::io::Write) -> Result<String, ExitCode>
{
    return std::fs::read_to_string(from).map_err(|error| {
        let _ = writeln!(notes, "cannot read {}: {error}", from.display());

        return ExitCode::Usage;
    });
}

fn Previewed(
    assembly: &Assembly,
    request: &EditRequest,
    staged: &str,
    notes: &mut dyn std::io::Write,
) -> Result<EditPreview, ExitCode>
{
    let rename = request.rename.as_deref();

    return assembly
        .store
        .Claim_For_Edit(&request.id, None)
        .and_then(|claimed| return claimed.Stage(staged, rename))
        .and_then(|edit| return edit.Preview(&assembly.store))
        .map_err(|error| return Report_Edit_Error(assembly, &error, notes));
}

/// The code an authoring refusal reports.
///
/// Everything the author can fix by editing their text is [`ExitCode::Refused`]; an
/// identifier the store does not hold goes through [`Absent_Or`], because over a store the
/// corpus never reached the honest answer is that something was missing.
fn Report_Edit_Error(
    assembly: &Assembly,
    error: &EditError,
    notes: &mut dyn std::io::Write,
) -> ExitCode
{
    let _ = writeln!(notes, "{error}");

    return match error
    {
        EditError::NoSuchRecord { .. } | EditError::NoContent { .. } =>
        {
            Absent_Or(assembly, ExitCode::NotFound, notes)
        }
        EditError::Ambiguous { .. } => ExitCode::NotFound,
        EditError::Store(_) => ExitCode::StoreError,
        EditError::NotAuthored { .. }
        | EditError::Unreadable { .. }
        | EditError::IdentityChanged { .. }
        | EditError::NotCanonical { .. }
        | EditError::PathTaken { .. } => ExitCode::Refused,
    };
}

/// Phase 2's question: what did this record say?
fn Record(
    assembly: &Assembly,
    request: &RecordRequest,
    channels: &mut Channels<'_>,
) -> ExitCode
{
    let revision = request.revision.as_deref();
    let documents = match assembly.store.Documents_Behind(&request.id, revision)
    {
        Ok(documents) => documents,
        Err(error) => return Report_Store_Error(&error, channels.notes),
    };

    return match documents.as_slice()
    {
        [only] => Printed_Record(&request.id, only, channels),
        [] => Nothing_Behind(assembly, request, channels.notes),
        held => Ambiguous_Revision(&request.id, held, channels.notes),
    };
}

/// The one document behind an identifier, with a note saying which it was.
fn Printed_Record(id: &str, only: &DocumentSource, channels: &mut Channels<'_>) -> ExitCode
{
    let _ = writeln!(
        channels.notes,
        "{id}: {} at revision {}, {}",
        only.path, only.revision, only.content_hash
    );
    let _ = write!(channels.output, "{}", only.text);

    return ExitCode::Ok;
}

/// One identifier held at several revisions, refused rather than concatenated.
///
/// Two revisions of one record are two answers to "what did this say", and printing both
/// under one heading is how a reader ends up quoting the wrong one.
fn Ambiguous_Revision(
    id: &str,
    documents: &[DocumentSource],
    notes: &mut dyn std::io::Write,
) -> ExitCode
{
    let held = documents
        .iter()
        .map(|document| return format!("{} ({})", document.revision, document.path))
        .collect::<Vec<String>>()
        .join(", ");

    let _ = writeln!(
        notes,
        "{id} is held at {} revisions: {held}.\n\
         Narrow it with --revision; printing them one after another would make the output a \
         document that never existed.",
        documents.len()
    );

    return ExitCode::NotFound;
}

/// What to say when a record read produced no document.
///
/// Three different things, because they are three different situations and only one of
/// them is the reader's mistake.
fn Nothing_Behind(
    assembly: &Assembly,
    request: &RecordRequest,
    notes: &mut dyn std::io::Write,
) -> ExitCode
{
    let summary = match assembly.store.Node_Summary(&request.id)
    {
        Ok(summary) => summary,
        Err(error) => return Report_Store_Error(&error, notes),
    };

    match summary
    {
        Some(node) => Note_Unsourced_Node(request, &node, notes),
        None => drop(writeln!(
            notes,
            "no node in this store is identified {}.",
            request.id
        )),
    }

    return Absent_Or(assembly, ExitCode::NotFound, notes);
}

/// A node the store holds with no source document recorded against it.
fn Note_Unsourced_Node(
    request: &RecordRequest,
    node: &NodeSummary,
    notes: &mut dyn std::io::Write,
)
{
    let wanted = request
        .revision
        .as_deref()
        .map_or_else(String::new, |label| return format!(" at revision {label}"));

    let _ = writeln!(
        notes,
        "{} is in the store as a {} node ({}, {}) titled {:?}, and no source document is \
         recorded against it{wanted}.",
        request.id, node.kind, node.authority, node.representation, node.title
    );
}

/// Phase 2's other half: the real rows.
fn Table(assembly: &Assembly, request: &TableRequest, channels: &mut Channels<'_>) -> ExitCode
{
    let (uid, tier) = match Addressed(assembly, request, channels.notes)
    {
        Ok(addressed) => addressed,
        Err(code) => return code,
    };

    let read = match Read_Table(assembly, uid, request, channels.notes)
    {
        Ok(read) => read,
        Err(code) => return code,
    };

    Note_Document(&read, tier, &request.document, channels.notes);

    if read.lines.is_empty()
    {
        return Unselected(assembly, request, &read, channels.notes);
    }

    return Printed(&read.lines, request, channels);
}

/// A document, the rows the narrowing selected from it, and the census of the whole of it.
///
/// The census counts the document rather than the selection deliberately: it is what
/// distinguishes "this document has no tables" from "the block you named has none".
struct ReadTable
{
    /// The document itself.
    found: DocumentSource,
    /// The rows under the current narrowing.
    lines: Vec<TableLine>,
    /// Every pipe line in the document, by kind.
    census: RowCensus,
}

/// Everything a `table` run reads, or the code saying which read failed.
fn Read_Table(
    assembly: &Assembly,
    uid: i64,
    request: &TableRequest,
    notes: &mut dyn std::io::Write,
) -> Result<ReadTable, ExitCode>
{
    let found = match assembly.store.Document(uid)
    {
        Ok(Some(found)) => found,
        Ok(None) => return Err(Report_Store_Error(&Vanished(uid), notes)),
        Err(error) => return Err(Report_Store_Error(&error, notes)),
    };
    let lines = match assembly.store.Table_Lines(uid, request.block, request.table)
    {
        Ok(lines) => lines,
        Err(error) => return Err(Report_Store_Error(&error, notes)),
    };
    let census = match assembly.store.Row_Census(RowScope::Document(uid))
    {
        Ok(census) => census,
        Err(error) => return Err(Report_Store_Error(&error, notes)),
    };

    return Ok(ReadTable {
        found,
        lines,
        census,
    });
}

/// A document that was read and carried no row the narrowing selected.
fn Unselected(
    assembly: &Assembly,
    request: &TableRequest,
    read: &ReadTable,
    notes: &mut dyn std::io::Write,
) -> ExitCode
{
    let _ = writeln!(
        notes,
        "{} carries no table row{}.",
        read.found.path,
        Narrowed(request.block, request.table)
    );

    // The document was read, so this is an answer rather than a shortfall — unless the
    // narrowing selected a table that is not there, which the census makes visible either
    // way.
    return Nothing_Selected(assembly, read.census.lines, notes);
}

/// Which document was read, how it was matched, and how many pipe lines it carries.
fn Note_Document(read: &ReadTable, tier: PathMatch, asked: &str, notes: &mut dyn std::io::Write)
{
    let found = &read.found;
    let census = &read.census;

    if tier != PathMatch::Exact
    {
        let _ = writeln!(notes, "{asked} matched {} by {}", found.path, tier.Label());
    }

    let _ = writeln!(
        notes,
        "{} at revision {}: {} pipe line(s) in the whole document — {} header, {} content, \
         {} separator",
        found.path, found.revision, census.lines, census.header, census.content, census.separator
    );
}

/// The one document an address names, or the message saying why it names none or several.
fn Addressed(
    assembly: &Assembly,
    request: &TableRequest,
    notes: &mut dyn std::io::Write,
) -> Result<(i64, PathMatch), ExitCode>
{
    let revision = request.revision.as_deref();
    let (matched, tier) = match assembly.store.Documents_Named(&request.document, revision)
    {
        Ok(found) => found,
        Err(error) => return Err(Report_Store_Error(&error, notes)),
    };

    return match matched.as_slice()
    {
        [uid] => Ok((*uid, tier)),
        [] => Err(No_Such_Document(assembly, &request.document, notes)),
        many => Err(Several_Documents(&request.document, many.len(), tier, notes)),
    };
}

/// An address that names nothing in this store.
fn No_Such_Document(
    assembly: &Assembly,
    asked: &str,
    notes: &mut dyn std::io::Write,
) -> ExitCode
{
    let _ = writeln!(notes, "no document in this store is named {asked}.");

    return Absent_Or(assembly, ExitCode::NotFound, notes);
}

/// An address that names several, which is a question rather than an answer.
fn Several_Documents(
    asked: &str,
    matched: usize,
    tier: PathMatch,
    notes: &mut dyn std::io::Write,
) -> ExitCode
{
    let _ = writeln!(
        notes,
        "{asked} matches {matched} documents by {}; give a whole path.",
        tier.Label()
    );

    return ExitCode::NotFound;
}

/// A document that was read and carried no row the narrowing selected.
fn Nothing_Selected(
    assembly: &Assembly,
    pipe_lines: u32,
    notes: &mut dyn std::io::Write,
) -> ExitCode
{
    if pipe_lines == 0 && !assembly.Is_Complete()
    {
        return Absent_Or(assembly, ExitCode::NotFound, notes);
    }

    return ExitCode::NotFound;
}

/// The rows themselves, and a note saying which blocks they came from.
fn Printed(
    lines: &[TableLine],
    request: &TableRequest,
    channels: &mut Channels<'_>,
) -> ExitCode
{
    for line in lines
    {
        let _ = writeln!(channels.output, "{}", line.text);
    }

    let _ = writeln!(
        channels.notes,
        "printed {} row(s){} — block(s) {}",
        lines.len(),
        Narrowed(request.block, request.table),
        Ordinals(lines)
    );

    return ExitCode::Ok;
}

fn Narrowed(block: Option<u32>, table: Option<u32>) -> String
{
    return match (block, table)
    {
        (None, None) => String::new(),
        (Some(block), None) => format!(" in block {block}"),
        (None, Some(table)) => format!(" in table {table} of any block"),
        (Some(block), Some(table)) => format!(" in table {table} of block {block}"),
    };
}

fn Ordinals(lines: &[nomos_spec_store::TableLine]) -> String
{
    let mut seen: Vec<u32> = Vec::new();
    for line in lines
    {
        if !seen.contains(&line.block_ordinal)
        {
            seen.push(line.block_ordinal);
        }
    }

    return seen
        .iter()
        .map(u32::to_string)
        .collect::<Vec<String>>()
        .join(", ");
}

/// Phase 4's renderers, run.
fn Render(assembly: &Assembly, request: &RenderRequest, channels: &mut Channels<'_>) -> ExitCode
{
    let catalogue = match Catalogue::Shipped()
    {
        Ok(catalogue) => catalogue,
        Err(error) => return Report_Project_Error(&error, channels.notes),
    };

    let declared = match Declared(&catalogue, request, channels.notes)
    {
        Ok(declared) => declared,
        Err(code) => return code,
    };

    let built = match Build(&assembly.store, &declared)
    {
        Ok(built) => built,
        Err(error) => return Report_Build_Error(assembly, &error, channels.notes),
    };

    return Placed_Projection(&built, &declared.id, &request.into, channels);
}

/// The shipped profile a run names, resolved against the subject it was given.
///
/// Resolved before the store is touched. A profile that names a subject and a run that
/// does not supply one disagree about what is being built, and the disagreement is
/// answerable without reading a single row.
fn Declared(
    catalogue: &Catalogue,
    request: &RenderRequest,
    notes: &mut dyn std::io::Write,
) -> Result<Profile, ExitCode>
{
    let declared = Resolved(catalogue, &request.profile, notes)?;

    return match declared.For(request.subject.as_deref())
    {
        Ok(resolved) => Ok(resolved),
        Err(error) => Err(Report_Project_Error(&error, notes)),
    };
}

/// Both halves of a built projection, written where the run asked for them.
fn Placed_Projection(
    built: &Output,
    id: &str,
    into: &Path,
    channels: &mut Channels<'_>,
) -> ExitCode
{
    let body = into.join(&built.path);
    let sidecar = into.join(&built.sidecar_path);
    let sheet = match built.Sidecar()
    {
        Ok(sheet) => sheet,
        Err(error) => return Report_Project_Error(&error, channels.notes),
    };

    for (path, content) in [(&body, &built.body), (&sidecar, &sheet)]
    {
        if let Some(code) = Placed(path, content, channels.notes)
        {
            return code;
        }
    }

    Report_Render(id, &body, &sidecar, channels.output);
    Report_Stamp(&built.stamp, channels.output);

    return ExitCode::Ok;
}

/// A projection that could not be built.
///
/// The empty-section case is pulled out because it is the one a store without its corpus
/// reaches, and answering it with the projection machinery's own message would send the
/// reader to change a profile because of a variable that is not set.
fn Report_Build_Error(
    assembly: &Assembly,
    error: &ProjectError,
    notes: &mut dyn std::io::Write,
) -> ExitCode
{
    let ProjectError::Empty {
        profile,
        section,
        content,
    } = error
    else
    {
        return Report_Project_Error(error, notes);
    };

    let empty = EmptySection {
        profile,
        section,
        content,
    };

    return Empty_Section(assembly, &empty, notes);
}

/// Where the two halves of a projection landed.
fn Report_Render(id: &str, body: &Path, sidecar: &Path, output: &mut dyn std::io::Write)
{
    let _ = writeln!(
        output,
        "{id} -> {}\nsidecar ({SIDECAR_SUFFIX}) -> {}",
        body.display(),
        sidecar.display()
    );
}

/// What the projection selected, and what it hashes to.
fn Report_Stamp(stamp: &Stamp, output: &mut dyn std::io::Write)
{
    for (title, count) in &stamp.sections
    {
        let _ = writeln!(output, "  {title}: {count}");
    }

    let _ = writeln!(
        output,
        "  content {} over inputs {}",
        stamp.content_digest, stamp.inputs_digest
    );
}

/// The shipped profile that identifier names, or the message saying which ones exist.
fn Resolved<'a>(
    catalogue: &'a Catalogue,
    profile: &str,
    notes: &mut dyn std::io::Write,
) -> Result<&'a Profile, ExitCode>
{
    let Some(declared) = catalogue.Named(profile)
    else
    {
        let _ = writeln!(
            notes,
            "no shipped profile is named {profile}. There are {}: {}",
            catalogue.Profiles().len(),
            catalogue
                .Profiles()
                .iter()
                .map(|shipped| return shipped.id.clone())
                .collect::<Vec<String>>()
                .join(", ")
        );

        return Err(ExitCode::NotFound);
    };

    return Ok(declared);
}

/// `D-128`'s check, run over what is on disk.
///
/// [`nomos_spec_project::Check`] shipped with the renderers and was reachable from that
/// crate's own unit tests and from nothing else, so a hand edit to a generated output was
/// detectable in principle and detected by nobody — the shape `OD-GATE-001` is about. This
/// is the command that runs it.
///
/// The branch that earns its own case is the half-present pair. A body with no sidecar
/// beside it is a failure rather than something skipped, because otherwise deleting the
/// sidecar is how an edit stops being caught, and a check teaches that trick to the first
/// person who trips over it.
fn Freshness_Of(
    assembly: &Assembly,
    request: &FreshnessRequest,
    channels: &mut Channels<'_>,
) -> ExitCode
{
    let catalogue = match Catalogue::Shipped()
    {
        Ok(catalogue) => catalogue,
        Err(error) => return Report_Project_Error(&error, channels.notes),
    };

    let only = request.profile.as_deref();
    let (required, wanted) = match Examined(&catalogue, request, channels.notes)
    {
        Ok(examined) => examined,
        Err(code) => return code,
    };

    let mut census = Census::Over(wanted.len(), required);
    let worst = census.Survey(assembly, &wanted, &request.into, channels);

    return census.Report(&request.into, only, worst, channels.output);
}

/// Which profiles this run will look at, and which of them it was promised.
///
/// Both are resolved before any disk is read, so an unknown identifier stays a question
/// about a profile rather than becoming an answer about a file.
fn Examined<'a>(
    catalogue: &'a Catalogue,
    request: &FreshnessRequest,
    notes: &mut dyn std::io::Write,
) -> Result<(Vec<&'a str>, Vec<&'a Profile>), ExitCode>
{
    let only = request.profile.as_deref();
    let required = Required_Profiles(catalogue, &request.require, notes)?;

    let wanted: Vec<&Profile> = match only
    {
        Some(id) => vec![Resolved(catalogue, id, notes)?],
        None => catalogue.Profiles().iter().collect(),
    };

    Every_Requirement_Examined(&required, &wanted, only, notes)?;

    return Ok((required, wanted));
}

/// The profiles a run was told it must find, resolved before any disk is read.
///
/// Resolving first is what keeps an unknown `--require` a question about a profile rather
/// than an answer about a file. Reporting `diagram-sett` as a missing output would send a
/// reader looking for something that was never nameable, and the catalogue already knows
/// how to refuse an identifier by listing the ones that exist.
fn Required_Profiles<'a>(
    catalogue: &'a Catalogue,
    require: &[String],
    notes: &mut dyn std::io::Write,
) -> Result<Vec<&'a str>, ExitCode>
{
    let mut required: Vec<&str> = Vec::new();

    for id in require
    {
        required.push(Resolved(catalogue, id, notes)?.id.as_str());
    }

    return Ok(required);
}

/// Refuses a run that was promised an output it would never have looked at.
///
/// `--profile a --require b` asks for one profile to be examined and a different one to be
/// guaranteed. Answering it would mean reporting success over a requirement nothing
/// checked, which is the shape this flag exists against — so it is a usage error and not a
/// quiet pass.
fn Every_Requirement_Examined(
    required: &[&str],
    wanted: &[&Profile],
    only: Option<&str>,
    notes: &mut dyn std::io::Write,
) -> Result<(), ExitCode>
{
    let Some(unexamined) = required
        .iter()
        .find(|id| return !wanted.iter().any(|profile| return profile.id == **id))
    else
    {
        return Ok(());
    };

    let _ = writeln!(
        notes,
        "--require {unexamined} cannot hold while --profile {} narrows this run to one \
         other profile: the requirement would be reported as met by a run that never \
         looked for it",
        only.unwrap_or("<none>")
    );

    return Err(ExitCode::Usage);
}

/// One profile's answer, or [`None`] when neither half of the pair is on disk.
fn Verdict(
    assembly: &Assembly,
    profile: &Profile,
    into: &Path,
    channels: &mut Channels<'_>,
) -> Option<ExitCode>
{
    let body_path = into.join(&profile.output);
    let sidecar_path = into.join(format!("{}{SIDECAR_SUFFIX}", profile.output));
    let body = std::fs::read_to_string(&body_path).ok();
    let sidecar = std::fs::read_to_string(&sidecar_path).ok();

    return match (body, sidecar)
    {
        (None, None) => None,
        (Some(_), None) => Some(Unstamped(profile, channels.output)),
        (None, Some(_)) => Some(Unbodied(profile, channels.output)),
        (Some(body), Some(sidecar)) =>
        {
            let rendered = Rendered {
                body: &body,
                sidecar: &sidecar,
            };

            Some(Compared(assembly, profile, &rendered, channels))
        }
    };
}

/// A body with no stamp beside it.
///
/// A failure rather than something skipped, because otherwise deleting the sidecar is how
/// an edit stops being caught, and a check that skipped it would teach that trick to the
/// first person who tripped over it.
fn Unstamped(profile: &Profile, output: &mut dyn std::io::Write) -> ExitCode
{
    let _ = writeln!(
        output,
        "{}: {} is there and {}{SIDECAR_SUFFIX} is not, so nothing can say whether it is what \
         the store produced",
        profile.id, profile.output, profile.output
    );

    return ExitCode::Stale;
}

/// A stamp with no body beside it: a governed output was deleted or never written.
fn Unbodied(profile: &Profile, output: &mut dyn std::io::Write) -> ExitCode
{
    let _ = writeln!(
        output,
        "{}: a sidecar is there and {} is not, so a governed output was deleted or never \
         written",
        profile.id, profile.output
    );

    return ExitCode::Stale;
}

/// The comparison itself, with a store that may not be whole.
fn Compared(
    assembly: &Assembly,
    profile: &Profile,
    rendered: &Rendered<'_>,
    channels: &mut Channels<'_>,
) -> ExitCode
{
    let body = Some(rendered.body);
    let sidecar = Some(rendered.sidecar);
    let freshness = match Check(&assembly.store, profile, body, sidecar)
    {
        Ok(freshness) => freshness,
        Err(error) => return Report_Build_Error(assembly, &error, channels.notes),
    };

    let report = freshness.Report(&profile.output);
    let _ = writeln!(channels.output, "{}: {report}", profile.id);

    if freshness.Is_Fresh()
    {
        return ExitCode::Ok;
    }

    return ExitCode::Stale;
}

/// What the run looked at, printed whether or not it found anything.
///
/// A freshness command that prints nothing over a directory holding no outputs reads
/// exactly like one that checked everything and was happy, which is the defect the whole
/// group exists to avoid.
struct Census<'a>
{
    /// How many profiles this run was going to look for.
    wanted: usize,
    /// How many it found both halves of and compared.
    checked: u32,
    /// Those absent from the build root and not promised by anyone.
    unbuilt: Vec<&'a str>,
    /// Those this run was told must be present.
    required: Vec<&'a str>,
    /// Those it was promised and did not get a current output for, whether because
    /// nothing was on disk or because what was there did not hold up.
    unmet: Vec<&'a str>,
}

impl<'a> Census<'a>
{
    /// An empty census over a run that is about to look at `wanted` profiles.
    fn Over(wanted: usize, required: Vec<&'a str>) -> Self
    {
        return Self {
            wanted,
            checked: 0,
            unbuilt: Vec::new(),
            required,
            unmet: Vec::new(),
        };
    }

    /// Every wanted profile, examined, counted, and reduced to one code for the run.
    fn Survey(
        &mut self,
        assembly: &Assembly,
        wanted: &[&'a Profile],
        into: &Path,
        channels: &mut Channels<'_>,
    ) -> ExitCode
    {
        let mut worst = ExitCode::Ok;

        for profile in wanted
        {
            let found = Verdict(assembly, profile, into, channels);
            let code = self.Record(profile, found, channels.output);
            worst = Worse(worst, code);
        }

        return worst;
    }

    /// One profile's outcome, counted, and the code it contributes to the run.
    ///
    /// A promise is kept only by an output that is current. A half-present pair or an
    /// edited body has already printed its own line, and carrying that into the
    /// requirement summary is what stops the summary reporting a requirement as met by a
    /// file that just failed.
    fn Record(
        &mut self,
        profile: &'a Profile,
        found: Option<ExitCode>,
        output: &mut dyn std::io::Write,
    ) -> ExitCode
    {
        let promised = self.required.contains(&profile.id.as_str());

        let Some(code) = found
        else
        {
            return self.Absent(profile, promised, output);
        };

        self.checked = self.checked.saturating_add(1);
        if promised && !matches!(code, ExitCode::Ok)
        {
            self.unmet.push(profile.id.as_str());
        }

        return code;
    }

    /// A profile with neither half on disk: a broken promise, or simply not built here.
    fn Absent(
        &mut self,
        profile: &'a Profile,
        promised: bool,
        output: &mut dyn std::io::Write,
    ) -> ExitCode
    {
        if !promised
        {
            self.unbuilt.push(profile.id.as_str());

            return ExitCode::Ok;
        }

        let _ = writeln!(
            output,
            "{}: required here, and neither {} nor its stamp is on disk, so an output this \
             repository promises to ship was never written or has been deleted",
            profile.id, profile.output
        );
        self.unmet.push(profile.id.as_str());

        return ExitCode::Stale;
    }

    fn Report(
        &self,
        into: &Path,
        only: Option<&str>,
        worst: ExitCode,
        output: &mut dyn std::io::Write,
    ) -> ExitCode
    {
        let _ = writeln!(
            output,
            "checked {} of {} governed output(s) under {}",
            self.checked,
            self.wanted,
            into.display()
        );

        if !self.unbuilt.is_empty()
        {
            let _ = writeln!(output, "not built here: {}", self.unbuilt.join(", "));
        }

        self.Requirements(output);

        return self.Outcome(into, only, worst, output);
    }

    /// The code the run reports, once everything it looked at has been named.
    ///
    /// Asking about one profile that is not there is a question about a named file, and
    /// "no such file" is its answer. Asking about all of them over a build root that holds
    /// three is the ordinary case and not a failure. A profile that was *required* is
    /// neither: it has already been reported as a missing promise, and letting this answer
    /// for it would downgrade that finding to a lookup miss.
    fn Outcome(
        &self,
        into: &Path,
        only: Option<&str>,
        worst: ExitCode,
        output: &mut dyn std::io::Write,
    ) -> ExitCode
    {
        let Some(id) = only
        else
        {
            return worst;
        };

        if self.checked != 0 || !self.unmet.is_empty()
        {
            return worst;
        }

        let _ = writeln!(
            output,
            "{id} has not been built under {}, so there was nothing to compare",
            into.display()
        );

        return ExitCode::NotFound;
    }

    /// What this run was promised, named whether or not it was kept.
    ///
    /// The satisfied case prints too. A gate step whose green output does not say which
    /// outputs it enforced is indistinguishable from one that enforced nothing, and this
    /// whole flag exists because `checked 0 of 14` already exits zero.
    fn Requirements(&self, output: &mut dyn std::io::Write)
    {
        if self.required.is_empty()
        {
            return;
        }

        if self.unmet.is_empty()
        {
            let _ = writeln!(output, "required and current: {}", self.required.join(", "));

            return;
        }

        let _ = writeln!(
            output,
            "required and not current: {}",
            self.unmet.join(", ")
        );
    }
}

/// The code a run reports when its profiles disagreed about what happened.
///
/// [`ExitCode::Stale`] beats [`ExitCode::Absent`] deliberately: a definite finding about
/// one output is more actionable than a machine that could not check another, and the
/// text above has already said both.
const fn Worse(carried: ExitCode, found: ExitCode) -> ExitCode
{
    return match (carried, found)
    {
        (ExitCode::StoreError, _) | (_, ExitCode::StoreError) => ExitCode::StoreError,
        (ExitCode::Stale, _) | (_, ExitCode::Stale) => ExitCode::Stale,
        (ExitCode::Absent, _) | (_, ExitCode::Absent) => ExitCode::Absent,
        _ => ExitCode::Ok,
    };
}

/// A section that selected nothing, over a store that is missing its corpus.
///
/// The profile machinery already refuses to render an empty section, and its message is
/// the right one when the store is whole: declare `may_be_empty` if nothing is the honest
/// answer. Over a store the corpus never reached, that message sends the reader to change
/// a profile because of a variable that is not set.
fn Empty_Section(
    assembly: &Assembly,
    empty: &EmptySection<'_>,
    notes: &mut dyn std::io::Write,
) -> ExitCode
{
    if assembly.Is_Complete()
    {
        let reported = ProjectError::Empty {
            profile: empty.profile.to_owned(),
            section: empty.section.to_owned(),
            content: empty.content,
        };
        let _ = writeln!(notes, "{reported}");

        return ExitCode::NotFound;
    }

    let _ = writeln!(
        notes,
        "{}: section {:?} selected no {}, and this store is not whole. Reporting the absence \
         above rather than the empty section: an empty projection over a store nothing was \
         read into is not a projection of an empty specification.",
        empty.profile, empty.section, empty.content
    );

    return ExitCode::Absent;
}

fn Profiles(channels: &mut Channels<'_>) -> ExitCode
{
    let catalogue = match Catalogue::Shipped()
    {
        Ok(catalogue) => catalogue,
        Err(error) => return Report_Project_Error(&error, channels.notes),
    };

    for profile in catalogue.Profiles()
    {
        let _ = writeln!(
            channels.output,
            "{:<28} {:<12} {:<34} {}",
            profile.id,
            profile.format.Label(),
            profile.output,
            Sections(profile)
        );
    }

    return ExitCode::Ok;
}

fn Sections(profile: &Profile) -> String
{
    return profile
        .sections
        .iter()
        .map(|section| return section.content.Label())
        .collect::<Vec<&str>>()
        .join("+");
}

/// What went into this store, and what did not.
///
/// Exits [`ExitCode::Absent`] when anything is missing, so this is a check rather than a
/// description: a script can ask whether the store it is about to read is whole.
fn Sources(assembly: &Assembly, output: &mut dyn std::io::Write) -> ExitCode
{
    for line in &assembly.read
    {
        let _ = writeln!(output, "read: {line}");
    }

    if assembly.Is_Complete()
    {
        let _ = writeln!(output, "nothing this store expects is missing");

        return ExitCode::Ok;
    }

    let _ = writeln!(output, "{}", assembly.Describe_Absences());
    let _ = writeln!(
        output,
        "{} of this store's sources were not read",
        assembly.absent.len()
    );

    return ExitCode::Absent;
}

/// Turns an empty answer into an absence when something was missing.
///
/// The single place the rule lives. Every empty answer in this group goes through it, so
/// "nothing found" and "nothing was read in" cannot start printing the same way again.
fn Absent_Or(
    assembly: &Assembly,
    otherwise: ExitCode,
    notes: &mut dyn std::io::Write,
) -> ExitCode
{
    if assembly.Is_Complete()
    {
        return otherwise;
    }

    let _ = writeln!(
        notes,
        "This store is not whole — see the absence above. An identifier the corpus carries \
         is unknown here for that reason and not because nothing holds it, so this is \
         reported as an absence rather than as an empty result."
    );

    return ExitCode::Absent;
}

/// A document that resolved and then could not be read back.
///
/// Its own function because the situation is a store defect rather than a caller's
/// mistake: the surrogate came out of a query against the same connection.
fn Vanished(uid: i64) -> StoreError
{
    return StoreError::Sql(format!(
        "document {uid} resolved and then could not be read back from the same connection"
    ));
}

fn Report_Store_Error(error: &StoreError, notes: &mut dyn std::io::Write) -> ExitCode
{
    let _ = writeln!(notes, "{error}");

    return ExitCode::StoreError;
}

/// A projection failure, as a number a caller can branch on.
///
/// Not every one of these is a store problem, and `5` for all of them was defensible only
/// while every one of them was. A subject the caller did not give and a subject the profile
/// cannot use are both arguments that were wrong before a row was read — an agent told the
/// store failed will retry; an agent told its command line was wrong will fix it. The rest
/// stay `StoreError` because that is what they are: the projection could not be built out
/// of what the store holds.
fn Report_Project_Error(error: &ProjectError, notes: &mut dyn std::io::Write) -> ExitCode
{
    let _ = writeln!(notes, "{error}");

    return match *error
    {
        ProjectError::SubjectMissing { .. } | ProjectError::SubjectUnexpected { .. } =>
        {
            ExitCode::Usage
        }
        _ => ExitCode::StoreError,
    };
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn Arguments(text: &str) -> Vec<String>
    {
        return text.split_whitespace().map(str::to_owned).collect();
    }

    #[test]
    fn Test_Record_Should_Parse_With_And_Without_A_Revision()
    {
        assert_eq!(
            Parse(&Arguments("record --id D-129")).expect("parses"),
            SpecCommand::Record(RecordRequest {
                id: "D-129".to_owned(),
                revision: None,
            })
        );
        assert_eq!(
            Parse(&Arguments("record --id D-129 --revision authored")).expect("parses"),
            SpecCommand::Record(RecordRequest {
                id: "D-129".to_owned(),
                revision: Some("authored".to_owned()),
            })
        );
    }

    #[test]
    fn Test_A_Missing_Argument_Should_Name_Itself()
    {
        let error = Parse(&Arguments("record")).expect_err("must refuse");

        assert!(error.contains("--id"), "{error}");
        assert!(error.contains("usage"), "{error}");
    }

    /// A block ordinal that is not a number would otherwise become "no such block", which
    /// sends the reader looking for a block instead of at what they typed.
    #[test]
    fn Test_A_Non_Numeric_Ordinal_Should_Be_Refused()
    {
        let error = Parse(&Arguments("table --document x.md --block seven"))
            .expect_err("must refuse");

        assert!(error.contains("--block"), "{error}");
        assert!(error.contains("seven"), "{error}");
    }

    #[test]
    fn Test_Render_Should_Require_Both_A_Profile_And_A_Destination()
    {
        assert!(Parse(&Arguments("render --profile github-markdown")).is_err());
        assert!(Parse(&Arguments("render --into build")).is_err());
        assert_eq!(
            Parse(&Arguments("render --profile github-markdown --into build")).expect("parses"),
            SpecCommand::Render(RenderRequest {
                profile: "github-markdown".to_owned(),
                into: PathBuf::from("build"),
                subject: None,
            })
        );
    }

    /// `--profile` is optional here and required by `render`, so the two must not share a
    /// parse. A freshness run over a whole build root is the useful one.
    #[test]
    fn Test_Freshness_Should_Take_A_Destination_And_An_Optional_Profile()
    {
        assert_eq!(
            Parse(&Arguments("freshness --into build")).expect("parses"),
            SpecCommand::Freshness(FreshnessRequest {
                into: PathBuf::from("build"),
                profile: None,
                require: Vec::new(),
            })
        );
        assert_eq!(
            Parse(&Arguments("freshness --into build --profile mcp-resource")).expect("parses"),
            SpecCommand::Freshness(FreshnessRequest {
                into: PathBuf::from("build"),
                profile: Some("mcp-resource".to_owned()),
                require: Vec::new(),
            })
        );
        assert!(Parse(&Arguments("freshness --profile mcp-resource")).is_err());
    }

    /// `--subject` is optional at the parse, and required by the profile.
    ///
    /// Whether a run needs one is a fact about the profile named, which the parser has not
    /// resolved yet. Refusing here would mean teaching the command line which profiles are
    /// subject-addressed — a second copy of something the catalogue already says.
    #[test]
    fn Test_Render_Should_Carry_A_Subject_When_One_Is_Given()
    {
        assert_eq!(
            Parse(&Arguments("render --profile subject-dossier --into . --subject D-129"))
                .expect("parses"),
            SpecCommand::Render(RenderRequest {
                profile: "subject-dossier".to_owned(),
                into: PathBuf::from("."),
                subject: Some("D-129".to_owned()),
            })
        );
        assert_eq!(
            Parse(&Arguments("render --profile diagram-set --into .")).expect("parses"),
            SpecCommand::Render(RenderRequest {
                profile: "diagram-set".to_owned(),
                into: PathBuf::from("."),
                subject: None,
            })
        );
    }

    /// Repeated rather than comma-separated, so a run that promises two outputs says so
    /// twice and nothing has to decide what a comma inside an identifier would mean.
    #[test]
    fn Test_Freshness_Should_Collect_Every_Requirement()
    {
        assert_eq!(
            Parse(&Arguments(
                "freshness --into . --require diagram-set --require html-site"
            ))
            .expect("parses"),
            SpecCommand::Freshness(FreshnessRequest {
                into: PathBuf::from("."),
                profile: None,
                require: vec!["diagram-set".to_owned(), "html-site".to_owned()],
            })
        );
    }

    #[test]
    fn Test_An_Unknown_Command_Should_Be_A_Usage_Error()
    {
        let error = Parse(&Arguments("frobnicate")).expect_err("must refuse");

        assert!(error.contains("frobnicate"), "{error}");
    }

    /// The codes are a contract, and they are the binary's rather than the group's. `3`
    /// and `4` belong to `work`'s claim refusals and must not acquire a second meaning.
    #[test]
    fn Test_Exit_Codes_Should_Be_Stable_And_Not_Collide_With_Works()
    {
        assert_eq!(ExitCode::Ok.Value(), crate::work::ExitCode::Ok.Value());
        assert_eq!(ExitCode::Usage.Value(), crate::work::ExitCode::Usage.Value());
        assert_eq!(ExitCode::NotFound.Value(), 1);
        assert_eq!(ExitCode::StoreError.Value(), 5);
        assert_eq!(ExitCode::Absent.Value(), 6);
        assert_eq!(ExitCode::Unwritable.Value(), 7);
        assert_eq!(ExitCode::Stale.Value(), 8);
        assert_eq!(ExitCode::Refused.Value(), 9);

        for taken in [
            crate::work::ExitCode::ClaimUnavailable.Value(),
            crate::work::ExitCode::Conflict.Value(),
        ]
        {
            assert!(
                ![
                    ExitCode::NotFound.Value(),
                    ExitCode::StoreError.Value(),
                    ExitCode::Absent.Value(),
                    ExitCode::Unwritable.Value(),
                    ExitCode::Stale.Value(),
                    ExitCode::Refused.Value(),
                ]
                .contains(&taken),
                "spec reuses {taken}, which work already spends on a claim outcome"
            );
        }
    }

    #[test]
    fn Test_The_Usage_Text_Should_Name_Every_Command()
    {
        let usage = Usage_Text();

        for command in [
            "record",
            "table",
            "render",
            "freshness",
            "markdown",
            "preview",
            "commit",
            "profiles",
            "sources",
        ]
        {
            assert!(usage.contains(command), "usage does not mention {command}");
        }
    }

    /// `--rename` is optional and `--from` is not, so a rename cannot be a second parse of
    /// `commit` — and `--into` defaults, because a record's path is repository relative and
    /// most callers mean the tree they are standing in.
    #[test]
    fn Test_Commit_Should_Parse_With_A_Default_Tree_And_An_Optional_Rename()
    {
        assert_eq!(
            Parse(&Arguments("commit --id D-129 --from staged.md")).expect("parses"),
            SpecCommand::Commit(CommitRequest {
                edit: EditRequest {
                    id: "D-129".to_owned(),
                    from: PathBuf::from("staged.md"),
                    rename: None,
                },
                into: PathBuf::from("."),
            })
        );
        assert_eq!(
            Parse(&Arguments(
                "commit --id D-129 --from staged.md --rename docs/records/moved.md --into build"
            ))
            .expect("parses"),
            SpecCommand::Commit(CommitRequest {
                edit: EditRequest {
                    id: "D-129".to_owned(),
                    from: PathBuf::from("staged.md"),
                    rename: Some("docs/records/moved.md".to_owned()),
                },
                into: PathBuf::from("build"),
            })
        );
        assert!(Parse(&Arguments("commit --id D-129")).is_err());
    }

    #[test]
    fn Test_Markdown_And_Preview_Should_Parse()
    {
        assert_eq!(
            Parse(&Arguments("markdown --id D-129")).expect("parses"),
            SpecCommand::Markdown(RecordRequest {
                id: "D-129".to_owned(),
                revision: None,
            })
        );
        assert_eq!(
            Parse(&Arguments("preview --id D-129 --from staged.md")).expect("parses"),
            SpecCommand::Preview(EditRequest {
                id: "D-129".to_owned(),
                from: PathBuf::from("staged.md"),
                rename: None,
            })
        );
        assert!(Parse(&Arguments("preview --from staged.md")).is_err());
    }

    /// `markdown` is not `record` with a flag, and the parse is where that stays true.
    #[test]
    fn Test_Reading_Bytes_And_Rendering_Markdown_Should_Be_Different_Commands()
    {
        assert_ne!(
            Parse(&Arguments("record --id D-129")).expect("parses"),
            Parse(&Arguments("markdown --id D-129")).expect("parses")
        );
    }
}
