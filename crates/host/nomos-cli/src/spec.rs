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
use nomos_spec_project::{Build, Catalogue, Profile, ProjectError, SIDECAR_SUFFIX};
use nomos_spec_store::{PathMatch, RowScope, StoreError};
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
            \x20 profiles\n\
            \x20 sources\n\
            \n\
            common: [--corpus <directory>] [--corpus-revision <label>]\n\
            \n\
            `record` and `table` write content to stdout and everything about it to \
            stderr, so a redirect captures exactly what the store holds.\n\
            \n\
            the store is assembled per invocation: this repository's governing records \
            are embedded, and the v14 corpus is read from --corpus or the environment. A \
            corpus that is not there is reported as an absence rather than as a shorter \
            answer — run `nomos spec sources` to see what a store holds.\n\
            \n\
            exit codes: 0 ok, 1 not found, 2 usage, 5 store error, 6 a source was absent, \
            7 the output could not be written"
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

    let assembly = match Assemble(request)
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
        SpecCommand::Profiles => Profiles(output, notes),
        SpecCommand::Sources => Sources(&assembly, output),
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

        return ExitCode::NotFound;
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

        for command in ["record", "table", "render", "profiles", "sources"]
        {
            assert!(usage.contains(command), "usage does not mention {command}");
        }
    }
}
