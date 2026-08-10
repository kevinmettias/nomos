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

use crate::arguments::{Named_Value, Required};
use crate::corpus::{Assemble, Assembly, CorpusRequest};
use nomos_spec_project::{Build, Catalogue, Check, Profile, ProjectError, SIDECAR_SUFFIX};
use nomos_spec_store::{EditError, EditPreview, PathMatch, RowScope, StoreError};
use std::path::{Path, PathBuf};

/// What the process exits with.
///
/// The numbers are shared with every other group on this binary: an exit code means one
/// thing per binary rather than one thing per group, or an agent that runs both has to
/// know which command it ran before it can read the number. `3` and `4` are `work`'s
/// claim codes and are deliberately not reused here.
///
/// [`ExitCode::Absent`] is the distinction that earns its own code. An agent told the
/// corpus is absent should configure one; an agent told the identifier is unknown should
/// correct the identifier. Collapsing those into "non-zero" makes the fixable case
/// indistinguishable from the mistaken one — which is the confusion this whole group was
/// written to end.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitCode
{
    /// The question was answered.
    Ok = 0,
    /// The store does not hold it, and everything it would have come from was read.
    NotFound = 1,
    /// The command line was wrong.
    Usage = 2,
    /// The store could not be built or read at all.
    StoreError = 5,
    /// The answer is empty because something the store expected was not there.
    Absent = 6,
    /// The answer was produced and could not be written where it was asked to go.
    Unwritable = 7,
    /// A governed output on disk is no longer what the store and its stamp say it is.
    ///
    /// Apart from [`ExitCode::Absent`] on purpose. Absent is "nobody could find out";
    /// this is "somebody found out, and the answer is that the file drifted". A gate
    /// collapsing the two would report a machine without a corpus exactly as it reports
    /// an edited output, which is the confusion `OD-GATE-001` is already about.
    Stale = 8,
    /// An authoring step refused the edit it was given.
    ///
    /// Apart from [`ExitCode::Usage`] because the command line was right: the author asked
    /// for exactly what they meant and the *content* was refused — a text this surface would
    /// not have written, a front matter naming a different record, a rename onto an occupied
    /// path. An agent told its arguments were wrong will retype them; an agent told its edit
    /// was refused will read the reason.
    Refused = 9,
}

impl ExitCode
{
    /// The numeric code.
    #[must_use]
    pub const fn Value(self) -> i32
    {
        return self as i32;
    }
}

/// What to read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpecCommand
{
    /// Print a record's source, byte for byte.
    Record
    {
        /// The node identifier.
        id: String,
        /// Which revision of it, when more than one is held.
        revision: Option<String>,
    },
    /// Print a document's table rows, as authored.
    Table
    {
        /// A path, a file name, or a fragment of one.
        document: String,
        /// Only this block of the document.
        block: Option<u32>,
        /// Only this table within a block. Counted per block, so it narrows rather than
        /// addresses on its own.
        table: Option<u32>,
        /// Only this revision.
        revision: Option<String>,
    },
    /// Build a projection profile and write it out.
    Render
    {
        /// A shipped profile identifier.
        profile: String,
        /// The build root the profile's own relative output is placed under.
        into: PathBuf,
    },
    /// Compare the outputs already on disk against the store and their own stamps.
    Freshness
    {
        /// The build root the profiles' own relative outputs are read from.
        into: PathBuf,
        /// Only this profile. Without it, every shipped profile is looked for.
        profile: Option<String>,
    },
    /// Read a record out of the store as markdown, rendered from its rows.
    Markdown
    {
        /// The node identifier.
        id: String,
        /// Which revision of it, when more than one is held.
        revision: Option<String>,
    },
    /// Say what committing an edited record would change, and change nothing.
    Preview
    {
        id: String,
        /// The edited markdown.
        from: PathBuf,
        /// The path the record should move to. A rename is an ordinary edit.
        rename: Option<String>,
    },
    /// Preview an edited record and then commit it.
    Commit
    {
        id: String,
        from: PathBuf,
        rename: Option<String>,
        /// The tree the record's own path is written under. A record's path is repository
        /// relative, so a commit has to be told which tree it means; `.` is the default
        /// rather than the only option, so a test does not have to write into the tree it
        /// is testing.
        into: PathBuf,
    },
    /// List the shipped projection profiles.
    Profiles,
    /// Say what this store was assembled from, and what was missing.
    Sources,
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

    let value_of = |name: &str| return Named_Value(arguments, name);
    let required = |name: &str| return Required(value_of(name).as_ref(), name, &Usage_Text());

    return match verb.as_str()
    {
        "record" => Ok(SpecCommand::Record {
            id: required("--id")?,
            revision: value_of("--revision"),
        }),
        "table" => Ok(SpecCommand::Table {
            document: required("--document")?,
            block: Ordinal(value_of("--block").as_ref(), "--block")?,
            table: Ordinal(value_of("--table").as_ref(), "--table")?,
            revision: value_of("--revision"),
        }),
        "render" => Ok(SpecCommand::Render {
            profile: required("--profile")?,
            into: PathBuf::from(required("--into")?),
        }),
        "freshness" => Ok(SpecCommand::Freshness {
            into: PathBuf::from(required("--into")?),
            profile: value_of("--profile"),
        }),
        "markdown" => Ok(SpecCommand::Markdown {
            id: required("--id")?,
            revision: value_of("--revision"),
        }),
        "preview" => Ok(SpecCommand::Preview {
            id: required("--id")?,
            from: PathBuf::from(required("--from")?),
            rename: value_of("--rename"),
        }),
        "commit" => Ok(SpecCommand::Commit {
            id: required("--id")?,
            from: PathBuf::from(required("--from")?),
            rename: value_of("--rename"),
            into: value_of("--into").map_or_else(|| return PathBuf::from("."), PathBuf::from),
        }),
        "profiles" => Ok(SpecCommand::Profiles),
        "sources" => Ok(SpecCommand::Sources),
        other => Err(format!("unknown command `{other}`.\n\n{}", Usage_Text())),
    };
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
            \x20 render    --profile <id> --into <directory>\n\
            \x20 freshness --into <directory> [--profile <id>]\n\
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
        return Profiles(output, notes);
    }

    let mut assembly = match Assemble(request)
    {
        Ok(assembly) => assembly,
        Err(error) => return Report_Store_Error(&error, notes),
    };

    // Reported on the way through, on every command that is not about them already. A run
    // over a whole corpus prints nothing here, so the note's absence is itself the signal
    // — the same shape OD-GATE-001 settled on for the corpus gates.
    if !assembly.Is_Complete() && !matches!(command, SpecCommand::Sources)
    {
        let _ = writeln!(notes, "{}", assembly.Describe_Absences());
    }

    return match command
    {
        SpecCommand::Record { id, revision } =>
        {
            Record(&assembly, id, revision.as_deref(), output, notes)
        }
        SpecCommand::Table {
            document,
            block,
            table,
            revision,
        } => Table(&assembly, document, *block, *table, revision.as_deref(), output, notes),
        SpecCommand::Render { profile, into } => Render(&assembly, profile, into, output, notes),
        SpecCommand::Freshness { into, profile } =>
        {
            Freshness_Of(&assembly, into, profile.as_deref(), output, notes)
        }
        SpecCommand::Markdown { id, revision } =>
        {
            Markdown(&assembly, id, revision.as_deref(), output, notes)
        }
        SpecCommand::Preview { id, from, rename } =>
        {
            Preview(&assembly, id, from, rename.as_deref(), output, notes)
        }
        SpecCommand::Commit {
            id,
            from,
            rename,
            into,
        } => Commit(&mut assembly, id, from, rename.as_deref(), into, output, notes),
        SpecCommand::Profiles => Profiles(output, notes),
        SpecCommand::Sources => Sources(&assembly, output),
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
    id: &str,
    revision: Option<&str>,
    output: &mut impl std::io::Write,
    notes: &mut impl std::io::Write,
) -> ExitCode
{
    let projection = match assembly.store.Record_Markdown(id, revision)
    {
        Ok(projection) => projection,
        Err(error) => return Report_Edit_Error(assembly, &error, notes),
    };

    let _ = write!(output, "{}", projection.markdown);
    let _ = writeln!(
        notes,
        "{id}: {} at revision {}, rendered from the store's rows as {}",
        projection.path, projection.revision, projection.projected_hash
    );

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
fn Preview(
    assembly: &Assembly,
    id: &str,
    from: &Path,
    rename: Option<&str>,
    output: &mut impl std::io::Write,
    notes: &mut impl std::io::Write,
) -> ExitCode
{
    let staged = match Staged_Text(from, notes)
    {
        Ok(text) => text,
        Err(code) => return code,
    };

    let preview = match Previewed(assembly, id, &staged, rename, notes)
    {
        Ok(preview) => preview,
        Err(code) => return code,
    };

    let _ = writeln!(output, "{}", preview.Describe());
    let _ = writeln!(notes, "nothing was written. {EPHEMERAL}");

    return ExitCode::Ok;
}

/// The preview and then the commit, in that order, because the other order is not available.
fn Commit(
    assembly: &mut Assembly,
    id: &str,
    from: &Path,
    rename: Option<&str>,
    into: &Path,
    output: &mut impl std::io::Write,
    notes: &mut impl std::io::Write,
) -> ExitCode
{
    let staged = match Staged_Text(from, notes)
    {
        Ok(text) => text,
        Err(code) => return code,
    };

    let preview = match Previewed(assembly, id, &staged, rename, notes)
    {
        Ok(preview) => preview,
        Err(code) => return code,
    };
    let _ = writeln!(output, "{}", preview.Describe());
    let vacated = preview.Rename().map(|(before, _)| return before.to_owned());

    let report = match assembly.store.Commit_Edit(&preview)
    {
        Ok(report) => report,
        Err(error) => return Report_Edit_Error(assembly, &error, notes),
    };

    let vacated = vacated.map(|path| return into.join(path));
    if let Some(code) =
        Written(&into.join(&report.path), &staged, vacated.as_deref(), output, notes)
    {
        return code;
    }

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

    return Reproduced(assembly, id, &staged, output, notes);
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
    output: &mut impl std::io::Write,
    notes: &mut impl std::io::Write,
) -> Option<ExitCode>
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

    if let Some(old) = vacated
    {
        match std::fs::remove_file(old)
        {
            Ok(()) => drop(writeln!(output, "vacated {}", old.display())),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
            Err(error) => drop(writeln!(
                notes,
                "{} was written and {} could not be removed ({error}), so two files now \
                 declare this record",
                destination.display(),
                old.display()
            )),
        }
    }

    return None;
}

/// The round trip, closed on the way out: the store is asked to render what was just
/// committed, and the answer is compared with it.
fn Reproduced(
    assembly: &Assembly,
    id: &str,
    staged: &str,
    output: &mut impl std::io::Write,
    notes: &mut impl std::io::Write,
) -> ExitCode
{
    let _ = writeln!(notes, "{EPHEMERAL}");

    return match assembly.store.Record_Markdown(id, None)
    {
        Ok(projection) if projection.markdown == staged =>
        {
            let _ = writeln!(
                output,
                "the store renders it back as the same bytes ({})",
                projection.projected_hash
            );

            ExitCode::Ok
        }
        Ok(projection) =>
        {
            let _ = writeln!(
                notes,
                "the commit succeeded and the store renders {} rather than what was committed, \
                 so the round trip does not close here",
                projection.projected_hash
            );

            ExitCode::Stale
        }
        Err(error) => Report_Edit_Error(assembly, &error, notes),
    };
}

fn Staged_Text(from: &Path, notes: &mut impl std::io::Write) -> Result<String, ExitCode>
{
    return std::fs::read_to_string(from).map_err(|error| {
        let _ = writeln!(notes, "cannot read {}: {error}", from.display());

        return ExitCode::Usage;
    });
}

fn Previewed(
    assembly: &Assembly,
    id: &str,
    staged: &str,
    rename: Option<&str>,
    notes: &mut impl std::io::Write,
) -> Result<EditPreview, ExitCode>
{
    return assembly
        .store
        .Claim_For_Edit(id, None)
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
    notes: &mut impl std::io::Write,
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
    id: &str,
    revision: Option<&str>,
    output: &mut impl std::io::Write,
    notes: &mut impl std::io::Write,
) -> ExitCode
{
    let documents = match assembly.store.Documents_Behind(id, revision)
    {
        Ok(documents) => documents,
        Err(error) => return Report_Store_Error(&error, notes),
    };

    if let [only] = documents.as_slice()
    {
        let _ = writeln!(
            notes,
            "{id}: {} at revision {}, {}",
            only.path, only.revision, only.content_hash
        );
        let _ = write!(output, "{}", only.text);

        return ExitCode::Ok;
    }

    if documents.len() > 1
    {
        // Not a concatenation. Two revisions of one record are two answers to "what did
        // this say", and printing both under one heading is how a reader ends up quoting
        // the wrong one.
        let _ = writeln!(
            notes,
            "{id} is held at {} revisions: {}.\n\
             Narrow it with --revision; printing them one after another would make the \
             output a document that never existed.",
            documents.len(),
            documents
                .iter()
                .map(|document| return format!("{} ({})", document.revision, document.path))
                .collect::<Vec<String>>()
                .join(", ")
        );

        return ExitCode::NotFound;
    }

    return Nothing_Behind(assembly, id, revision, notes);
}

/// What to say when a record read produced no document.
///
/// Three different things, because they are three different situations and only one of
/// them is the reader's mistake.
fn Nothing_Behind(
    assembly: &Assembly,
    id: &str,
    revision: Option<&str>,
    notes: &mut impl std::io::Write,
) -> ExitCode
{
    let summary = match assembly.store.Node_Summary(id)
    {
        Ok(summary) => summary,
        Err(error) => return Report_Store_Error(&error, notes),
    };

    if let Some(node) = summary
    {
        let _ = writeln!(
            notes,
            "{id} is in the store as a {} node ({}, {}) titled {:?}, and no source document \
             is recorded against it{}.",
            node.kind,
            node.authority,
            node.representation,
            node.title,
            revision.map_or_else(String::new, |wanted| return format!(" at revision {wanted}"))
        );

        return Absent_Or(assembly, ExitCode::NotFound, notes);
    }

    let _ = writeln!(notes, "no node in this store is identified {id}.");

    return Absent_Or(assembly, ExitCode::NotFound, notes);
}

/// Phase 2's other half: the real rows.
fn Table(
    assembly: &Assembly,
    document: &str,
    block: Option<u32>,
    table: Option<u32>,
    revision: Option<&str>,
    output: &mut impl std::io::Write,
    notes: &mut impl std::io::Write,
) -> ExitCode
{
    let (matched, tier) = match assembly.store.Documents_Named(document, revision)
    {
        Ok(found) => found,
        Err(error) => return Report_Store_Error(&error, notes),
    };

    let Some(uid) = matched.first().copied()
    else
    {
        let _ = writeln!(notes, "no document in this store is named {document}.");

        return Absent_Or(assembly, ExitCode::NotFound, notes);
    };

    if matched.len() > 1
    {
        let _ = writeln!(
            notes,
            "{document} matches {} documents by {}; give a whole path.",
            matched.len(),
            tier.Label()
        );

        return ExitCode::NotFound;
    }

    let found = match assembly.store.Document(uid)
    {
        Ok(Some(found)) => found,
        Ok(None) => return Report_Store_Error(&Vanished(uid), notes),
        Err(error) => return Report_Store_Error(&error, notes),
    };

    let lines = match assembly.store.Table_Lines(uid, block, table)
    {
        Ok(lines) => lines,
        Err(error) => return Report_Store_Error(&error, notes),
    };

    let census = match assembly.store.Row_Census(RowScope::Document(uid))
    {
        Ok(census) => census,
        Err(error) => return Report_Store_Error(&error, notes),
    };

    if tier != PathMatch::Exact
    {
        let _ = writeln!(notes, "{document} matched {} by {}", found.path, tier.Label());
    }
    let _ = writeln!(
        notes,
        "{} at revision {}: {} pipe line(s) in the whole document — {} header, {} content, \
         {} separator",
        found.path, found.revision, census.lines, census.header, census.content, census.separator
    );

    if lines.is_empty()
    {
        let _ = writeln!(
            notes,
            "{} carries no table row{}.",
            found.path,
            Narrowed(block, table)
        );

        // The document was read, so this is an answer rather than a shortfall — unless
        // the narrowing selected a table that is not there, which the census above makes
        // visible either way.
        return if census.lines == 0 && !assembly.Is_Complete()
        {
            Absent_Or(assembly, ExitCode::NotFound, notes)
        }
        else
        {
            ExitCode::NotFound
        };
    }

    for line in &lines
    {
        let _ = writeln!(output, "{}", line.text);
    }

    let _ = writeln!(
        notes,
        "printed {} row(s){} — block(s) {}",
        lines.len(),
        Narrowed(block, table),
        Ordinals(&lines)
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
fn Render(
    assembly: &Assembly,
    profile: &str,
    into: &Path,
    output: &mut impl std::io::Write,
    notes: &mut impl std::io::Write,
) -> ExitCode
{
    let catalogue = match Catalogue::Shipped()
    {
        Ok(catalogue) => catalogue,
        Err(error) => return Report_Project_Error(&error, notes),
    };

    let declared = match Resolved(&catalogue, profile, notes)
    {
        Ok(declared) => declared,
        Err(code) => return code,
    };

    let built = match Build(&assembly.store, declared)
    {
        Ok(built) => built,
        Err(ProjectError::Empty {
            profile: named,
            section,
            content,
        }) =>
        {
            return Empty_Section(assembly, &named, &section, content, notes);
        }
        Err(error) => return Report_Project_Error(&error, notes),
    };

    let body = into.join(&built.path);
    let sidecar = into.join(&built.sidecar_path);
    let sheet = match built.Sidecar()
    {
        Ok(sheet) => sheet,
        Err(error) => return Report_Project_Error(&error, notes),
    };

    for (path, content) in [(&body, &built.body), (&sidecar, &sheet)]
    {
        if let Some(parent) = path.parent()
            && let Err(error) = std::fs::create_dir_all(parent)
        {
            let _ = writeln!(notes, "cannot create {}: {error}", parent.display());

            return ExitCode::Unwritable;
        }

        if let Err(error) = std::fs::write(path, content)
        {
            let _ = writeln!(notes, "cannot write {}: {error}", path.display());

            return ExitCode::Unwritable;
        }
    }

    let _ = writeln!(
        output,
        "{} -> {}\nsidecar ({SIDECAR_SUFFIX}) -> {}",
        declared.id,
        body.display(),
        sidecar.display()
    );
    for (title, count) in &built.stamp.sections
    {
        let _ = writeln!(output, "  {title}: {count}");
    }
    let _ = writeln!(
        output,
        "  content {} over inputs {}",
        built.stamp.content_digest, built.stamp.inputs_digest
    );

    return ExitCode::Ok;
}

/// The shipped profile that identifier names, or the message saying which ones exist.
fn Resolved<'a>(
    catalogue: &'a Catalogue,
    profile: &str,
    notes: &mut impl std::io::Write,
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
    into: &Path,
    only: Option<&str>,
    output: &mut impl std::io::Write,
    notes: &mut impl std::io::Write,
) -> ExitCode
{
    let catalogue = match Catalogue::Shipped()
    {
        Ok(catalogue) => catalogue,
        Err(error) => return Report_Project_Error(&error, notes),
    };

    let wanted: Vec<&Profile> = match only
    {
        Some(id) => match Resolved(&catalogue, id, notes)
        {
            Ok(declared) => vec![declared],
            Err(code) => return code,
        },
        None => catalogue.Profiles().iter().collect(),
    };

    let mut worst = ExitCode::Ok;
    let mut checked = 0_u32;
    let mut unbuilt: Vec<&str> = Vec::new();

    for profile in &wanted
    {
        match Verdict(assembly, profile, into, output, notes)
        {
            Some(code) =>
            {
                checked = checked.saturating_add(1);
                worst = Worse(worst, code);
            }
            None => unbuilt.push(profile.id.as_str()),
        }
    }

    return Census(&wanted, checked, &unbuilt, into, only, worst, output);
}

/// One profile's answer, or [`None`] when neither half of the pair is on disk.
fn Verdict(
    assembly: &Assembly,
    profile: &Profile,
    into: &Path,
    output: &mut impl std::io::Write,
    notes: &mut impl std::io::Write,
) -> Option<ExitCode>
{
    let body_path = into.join(&profile.output);
    let sidecar_path = into.join(format!("{}{SIDECAR_SUFFIX}", profile.output));
    let body = std::fs::read_to_string(&body_path).ok();
    let sidecar = std::fs::read_to_string(&sidecar_path).ok();

    return match (body, sidecar)
    {
        (None, None) => None,
        (Some(_), None) =>
        {
            let _ = writeln!(
                output,
                "{}: {} is there and {} is not, so nothing can say whether it is what the \
                 store produced",
                profile.id,
                profile.output,
                format_args!("{}{SIDECAR_SUFFIX}", profile.output)
            );

            Some(ExitCode::Stale)
        }
        (None, Some(_)) =>
        {
            let _ = writeln!(
                output,
                "{}: a sidecar is there and {} is not, so a governed output was deleted or \
                 never written",
                profile.id, profile.output
            );

            Some(ExitCode::Stale)
        }
        (Some(body), Some(sidecar)) =>
        {
            Some(Compared(assembly, profile, &body, &sidecar, output, notes))
        }
    };
}

/// The comparison itself, with a store that may not be whole.
fn Compared(
    assembly: &Assembly,
    profile: &Profile,
    body: &str,
    sidecar: &str,
    output: &mut impl std::io::Write,
    notes: &mut impl std::io::Write,
) -> ExitCode
{
    let freshness = match Check(&assembly.store, profile, Some(body), Some(sidecar))
    {
        Ok(freshness) => freshness,
        Err(ProjectError::Empty {
            profile: named,
            section,
            content,
        }) =>
        {
            return Empty_Section(assembly, &named, &section, content, notes);
        }
        Err(error) => return Report_Project_Error(&error, notes),
    };

    let _ = writeln!(output, "{}: {}", profile.id, freshness.Report(&profile.output));

    return if freshness.Is_Fresh()
    {
        ExitCode::Ok
    }
    else
    {
        ExitCode::Stale
    };
}

/// What the run looked at, printed whether or not it found anything.
///
/// A freshness command that prints nothing over a directory holding no outputs reads
/// exactly like one that checked everything and was happy, which is the defect the whole
/// group exists to avoid.
fn Census(
    wanted: &[&Profile],
    checked: u32,
    unbuilt: &[&str],
    into: &Path,
    only: Option<&str>,
    worst: ExitCode,
    output: &mut impl std::io::Write,
) -> ExitCode
{
    let _ = writeln!(
        output,
        "checked {checked} of {} governed output(s) under {}",
        wanted.len(),
        into.display()
    );

    if !unbuilt.is_empty()
    {
        let _ = writeln!(output, "not built here: {}", unbuilt.join(", "));
    }

    // Asking about one profile that is not there is a question about a named file, and
    // "no such file" is its answer. Asking about all of them over a build root that holds
    // three is the ordinary case and not a failure.
    if let Some(id) = only
        && checked == 0
    {
        let _ = writeln!(
            output,
            "{id} has not been built under {}, so there was nothing to compare",
            into.display()
        );

        return ExitCode::NotFound;
    }

    return worst;
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
    profile: &str,
    section: &str,
    content: &'static str,
    notes: &mut impl std::io::Write,
) -> ExitCode
{
    if assembly.Is_Complete()
    {
        let _ = writeln!(
            notes,
            "{}",
            ProjectError::Empty {
                profile: profile.to_owned(),
                section: section.to_owned(),
                content,
            }
        );

        return ExitCode::NotFound;
    }

    let _ = writeln!(
        notes,
        "{profile}: section {section:?} selected no {content}, and this store is not whole. \
         Reporting the absence above rather than the empty section: an empty projection over \
         a store nothing was read into is not a projection of an empty specification."
    );

    return ExitCode::Absent;
}

fn Profiles(output: &mut impl std::io::Write, notes: &mut impl std::io::Write) -> ExitCode
{
    let catalogue = match Catalogue::Shipped()
    {
        Ok(catalogue) => catalogue,
        Err(error) => return Report_Project_Error(&error, notes),
    };

    for profile in catalogue.Profiles()
    {
        let _ = writeln!(
            output,
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
fn Sources(assembly: &Assembly, output: &mut impl std::io::Write) -> ExitCode
{
    for line in &assembly.read
    {
        let _ = writeln!(output, "read: {line}");
    }

    if !assembly.Is_Complete()
    {
        let _ = writeln!(output, "{}", assembly.Describe_Absences());
    }

    if assembly.Is_Complete()
    {
        let _ = writeln!(output, "nothing this store expects is missing");

        return ExitCode::Ok;
    }

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
    notes: &mut impl std::io::Write,
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

fn Report_Store_Error(error: &StoreError, notes: &mut impl std::io::Write) -> ExitCode
{
    let _ = writeln!(notes, "{error}");

    return ExitCode::StoreError;
}

fn Report_Project_Error(error: &ProjectError, notes: &mut impl std::io::Write) -> ExitCode
{
    let _ = writeln!(notes, "{error}");

    return ExitCode::StoreError;
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
            SpecCommand::Record {
                id: "D-129".to_owned(),
                revision: None,
            }
        );
        assert_eq!(
            Parse(&Arguments("record --id D-129 --revision authored")).expect("parses"),
            SpecCommand::Record {
                id: "D-129".to_owned(),
                revision: Some("authored".to_owned()),
            }
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
            SpecCommand::Render {
                profile: "github-markdown".to_owned(),
                into: PathBuf::from("build"),
            }
        );
    }

    /// `--profile` is optional here and required by `render`, so the two must not share a
    /// parse. A freshness run over a whole build root is the useful one.
    #[test]
    fn Test_Freshness_Should_Take_A_Destination_And_An_Optional_Profile()
    {
        assert_eq!(
            Parse(&Arguments("freshness --into build")).expect("parses"),
            SpecCommand::Freshness {
                into: PathBuf::from("build"),
                profile: None,
            }
        );
        assert_eq!(
            Parse(&Arguments("freshness --into build --profile mcp-resource")).expect("parses"),
            SpecCommand::Freshness {
                into: PathBuf::from("build"),
                profile: Some("mcp-resource".to_owned()),
            }
        );
        assert!(Parse(&Arguments("freshness --profile mcp-resource")).is_err());
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
            SpecCommand::Commit {
                id: "D-129".to_owned(),
                from: PathBuf::from("staged.md"),
                rename: None,
                into: PathBuf::from("."),
            }
        );
        assert_eq!(
            Parse(&Arguments(
                "commit --id D-129 --from staged.md --rename docs/records/moved.md --into build"
            ))
            .expect("parses"),
            SpecCommand::Commit {
                id: "D-129".to_owned(),
                from: PathBuf::from("staged.md"),
                rename: Some("docs/records/moved.md".to_owned()),
                into: PathBuf::from("build"),
            }
        );
        assert!(Parse(&Arguments("commit --id D-129")).is_err());
    }

    #[test]
    fn Test_Markdown_And_Preview_Should_Parse()
    {
        assert_eq!(
            Parse(&Arguments("markdown --id D-129")).expect("parses"),
            SpecCommand::Markdown {
                id: "D-129".to_owned(),
                revision: None,
            }
        );
        assert_eq!(
            Parse(&Arguments("preview --id D-129 --from staged.md")).expect("parses"),
            SpecCommand::Preview {
                id: "D-129".to_owned(),
                from: PathBuf::from("staged.md"),
                rename: None,
            }
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
