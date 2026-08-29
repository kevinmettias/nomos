//! `nomos request` — submitting a feature request, design spec or feature result through the
//! one accept door `OD-SPEC-009` decided.
//!
//! This is a transport and nothing else, per that record: it parses arguments into a
//! [`SubmitRequest`] and dispatches to [`nomos_spec_orchestration::Submit`], which constructs
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

use crate::arguments::{Name, Named_Value, Named_Values, Required, Usage};
use nomos_platform_std::StdFileSystem;
use nomos_spec_orchestration::corpus::{Assemble, CorpusRequest};
use nomos_spec_orchestration::{RenderRefusal, SubmitAnswer, SubmitRefusal, SubmitRequest};

use nomos_spec_model::{DecisionGap, Severity, SubmissionKind, SubmissionState};
use nomos_spec_project::SIDECAR_SUFFIX;

use std::path::PathBuf;

/// What the process exits with.
///
/// Shares its numbers with every other group on this binary, so an agent that runs more than
/// one does not have to know which it ran before reading the number — see `spec::ExitCode` for
/// the fuller statement of that discipline. `9` is `spec`'s own "an edit was refused"; a
/// submission refused is the same shape one level up the same seam, and is deliberately not a
/// fresh number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExitCode
{
    /// The submission was accepted.
    Ok = 0,
    /// The command line was wrong.
    Usage = 2,
    /// The store could not be built or read at all.
    StoreError = 5,
    /// The submission was accepted and the projection asked for could not be written.
    Unwritable = 7,
    /// The submission failed the rule set. Nothing was stored.
    Refused = 9,
}

impl ExitCode
{
    #[must_use]
    pub const fn Value(self) -> i32
    {
        return self as i32;
    }
}

/// What `nomos request` was asked to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Command
{
    Submit(SubmitRequest),
}

/// Parses `nomos request` arguments.
///
/// # Errors
///
/// Returns a message naming what was wrong and what was expected.
pub(crate) fn Parse(arguments: &[String]) -> Result<Command, String>
{
    let Some(verb) = arguments.first()
    else
    {
        return Err(Usage_Text());
    };

    let rest = arguments.get(1..).unwrap_or_default();

    return match verb.as_str()
    {
        "submit" => Parse_Submit(rest),
        other => Err(format!("unknown command `{other}`.\n\n{}", Usage_Text())),
    };
}

fn Parse_Submit(arguments: &[String]) -> Result<Command, String>
{
    let kind_text = Required_Value(arguments, "--kind")?;
    let kind = Parse_Kind(&kind_text)?;
    let id = Required_Value(arguments, "--id")?;
    let by = Required_Value(arguments, "--by")?;
    let state = Parse_State(Named_Value(arguments, "--state").as_deref())?;
    let contract_version =
        Parse_Contract_Version(Named_Value(arguments, "--contract-version").as_deref())?;
    let fields = Parse_Fields(arguments)?;
    let gaps = Parse_Gaps(arguments)?;
    let into = Named_Value(arguments, "--into").map(PathBuf::from);

    return Ok(Command::Submit(SubmitRequest {
        kind,
        id,
        by,
        state,
        contract_version,
        fields,
        gaps,
        submitted_through: "cli".to_owned(),
        into,
    }));
}

fn Required_Value(arguments: &[String], name: &str) -> Result<String, String>
{
    let value = Named_Value(arguments, name);

    return Required(value.as_ref(), Name(name), Usage(&Usage_Text()));
}

fn Parse_Kind(text: &str) -> Result<SubmissionKind, String>
{
    return SubmissionKind::Parse(text).ok_or_else(|| {
        return format!(
            "--kind takes feature-request, design-spec or feature-result; `{text}` is none of \
             them.\n\n{}",
            Usage_Text()
        );
    });
}

fn Parse_State(text: Option<&str>) -> Result<SubmissionState, String>
{
    let Some(text) = text
    else
    {
        return Ok(SubmissionState::Draft);
    };

    return SubmissionState::Parse(text).ok_or_else(|| {
        return format!("--state takes draft or accepted; `{text}` is neither.\n\n{}", Usage_Text());
    });
}

fn Parse_Contract_Version(text: Option<&str>) -> Result<u32, String>
{
    let Some(text) = text
    else
    {
        return Ok(1);
    };

    return text.parse::<u32>().map_err(|cause| {
        return format!(
            "--contract-version takes a whole number; `{text}` is not one: {cause}\n\n{}",
            Usage_Text()
        );
    });
}

/// Every `--field name=value`, in the order they were given.
fn Parse_Fields(arguments: &[String]) -> Result<Vec<(String, String)>, String>
{
    let mut fields = Vec::new();

    for entry in Named_Values(arguments, "--field")
    {
        let Some((name, value)) = entry.split_once('=')
        else
        {
            return Err(format!(
                "--field takes `name=value`; `{entry}` has no `=`.\n\n{}",
                Usage_Text()
            ));
        };
        fields.push((name.to_owned(), value.to_owned()));
    }

    return Ok(fields);
}

/// Every `--gap question|blocked-fields|severity[|closed-by]`.
fn Parse_Gaps(arguments: &[String]) -> Result<Vec<DecisionGap>, String>
{
    let mut gaps = Vec::new();

    for entry in Named_Values(arguments, "--gap")
    {
        gaps.push(Parse_Gap(&entry)?);
    }

    return Ok(gaps);
}

fn Parse_Gap(entry: &str) -> Result<DecisionGap, String>
{
    let (question, blocks, severity, closed_by) = Gap_Fields(entry)?;
    let severity = Gap_Severity(severity)?;

    return Ok(DecisionGap {
        question: question.to_owned(),
        blocks: Gap_Blocks(blocks),
        severity,
        closed_by: closed_by.map(str::to_owned),
    });
}

/// How many pipe-separated fields `--gap` accepts: question, blocks, severity, closed-by.
const GAP_FIELD_COUNT: usize = 4;

/// Splits one `--gap` entry into its pipe-separated fields, refusing when either of the two
/// required ones is missing.
fn Gap_Fields(entry: &str) -> Result<(&str, &str, &str, Option<&str>), String>
{
    let mut parts = entry.splitn(GAP_FIELD_COUNT, '|');
    let question = parts.next().unwrap_or_default();
    let blocks = parts.next();
    let severity = parts.next();
    let closed_by = parts.next();

    let (Some(blocks), Some(severity)) = (blocks, severity)
    else
    {
        return Err(format!(
            "--gap takes `question|blocked-fields|severity[|closed-by]`; `{entry}` is short a \
             field.\n\n{}",
            Usage_Text()
        ));
    };

    return Ok((question, blocks, severity, closed_by));
}

/// Parses the severity field of a `--gap` entry.
fn Gap_Severity(text: &str) -> Result<Severity, String>
{
    return Severity::Parse(text).ok_or_else(|| {
        return format!(
            "--gap's severity takes blocking or non-blocking; `{text}` is neither.\n\n{}",
            Usage_Text()
        );
    });
}

/// The comma-separated blocked-field list, trimmed and with empty entries dropped.
fn Gap_Blocks(text: &str) -> Vec<String>
{
    return text
        .split(',')
        .map(str::trim)
        .filter(|entry| return !entry.is_empty())
        .map(str::to_owned)
        .collect();
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
        Command::Submit(submit) => Submit(submit, request, output, notes),
    };
}

/// Assembles the store `request` names, and dispatches `submit` against it through
/// `nomos-spec-orchestration::Submit` -- the same composition-root choice `nomos-cli::spec`
/// already makes for the other nine `SpecCommand` verbs, `nomos_platform_std::StdFileSystem`
/// as the concrete platform.
fn Submit(
    submit: &SubmitRequest,
    request: &CorpusRequest,
    output: &mut impl std::io::Write,
    notes: &mut impl std::io::Write,
) -> ExitCode
{
    let mut assembly = match Assemble(request)
    {
        Ok(assembly) => assembly,
        Err(error) =>
        {
            let _ = writeln!(notes, "{error}");
            return ExitCode::StoreError;
        }
    };

    return match nomos_spec_orchestration::Submit(&mut assembly, submit, &StdFileSystem)
    {
        Ok(answer) => Accepted(&answer, output),
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
        Err(SubmitRefusal::Written(refusal)) => Unwritten(&refusal, notes),
    };
}

/// A submission accepted, and its `subject-dossier` projection reported if one was written.
fn Accepted(answer: &SubmitAnswer, output: &mut impl std::io::Write) -> ExitCode
{
    let _ = writeln!(
        output,
        "accepted {} as {} ({}), uid {}",
        answer.submission.id,
        answer.submission.kind.Label(),
        answer.submission.state.Label(),
        answer.uid
    );

    if let Some(written) = &answer.written
    {
        let _ = writeln!(
            output,
            "{} -> {}\nsidecar ({SIDECAR_SUFFIX}) -> {}",
            written.id,
            written.body.display(),
            written.sidecar.display()
        );
    }

    return ExitCode::Ok;
}

/// The submission was accepted and its `subject-dossier` projection could not be built or
/// placed.
fn Unwritten(refusal: &RenderRefusal, notes: &mut impl std::io::Write) -> ExitCode
{
    return match refusal
    {
        RenderRefusal::Unwritable { path, error } =>
        {
            let _ = writeln!(notes, "cannot write {}: {error}", path.display());
            ExitCode::Unwritable
        }
        RenderRefusal::Store(error) =>
        {
            let _ = writeln!(notes, "{error}");
            ExitCode::StoreError
        }
        RenderRefusal::NoSuchProfile { requested, .. } =>
        {
            let _ = writeln!(notes, "the shipped catalogue no longer carries {requested}");
            ExitCode::StoreError
        }
        RenderRefusal::Project(error) =>
        {
            let _ = writeln!(notes, "{error}");
            ExitCode::StoreError
        }
    };
}

fn Usage_Text() -> String
{
    return "usage: nomos request <command>\n\
            \n\
            \x20 submit --kind <feature-request|design-spec|feature-result> --id <node-id>\n\
            \x20        --by <name> [--state draft|accepted] [--contract-version <n>]\n\
            \x20        --field <name>=<value> [--field <name>=<value> …]\n\
            \x20        [--gap <question>|<blocked-fields,comma-separated>|\
            <blocking|non-blocking>[|<closed-by>]] …\n\
            \x20        [--into <directory>]\n\
            \n\
            common: [--corpus <directory>] [--corpus-revision <label>]\n\
            \n\
            `submit` is a transport onto the one accept function OD-SPEC-009 decided: it \
            constructs a submission from exactly what was typed, with every --field value \
            carrying origin `submitted`, and calls the store's Accept_Submission. It validates \
            nothing and defaults nothing beyond --state (draft) and --contract-version (1), \
            which are facts about this run rather than about the submission.\n\
            \n\
            the store this verb writes to is assembled for this invocation and gone when it \
            exits — nothing in this repository persists a specification database. --into \
            renders the accepted submission through the subject-dossier profile before the \
            store is gone, which is the one chance this run has to take the freshness proof \
            ARC-SPECDB-002 charges a born-structured object with.\n\
            \n\
            exit codes: 0 ok, 2 usage, 5 store error, 7 the projection could not be written, \
            9 the submission was refused"
        .to_owned();
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
    fn Test_A_Submit_Command_Should_Parse_Its_Fields_And_Default_State_And_Version()
    {
        let arguments = Arguments(
            "submit --kind feature-request --id FR-100 --by kevin \
             --field title=t --field goal=g",
        );

        let Command::Submit(request) = Parse(&arguments).expect("parses");

        assert_eq!(request.kind, SubmissionKind::FeatureRequest);
        assert_eq!(request.id, "FR-100");
        assert_eq!(request.by, "kevin");
        assert_eq!(request.state, SubmissionState::Draft);
        assert_eq!(request.contract_version, 1);
        assert_eq!(
            request.fields,
            vec![("title".to_owned(), "t".to_owned()), ("goal".to_owned(), "g".to_owned())]
        );
    }

    #[test]
    fn Test_A_Field_With_No_Equals_Should_Be_A_Usage_Error()
    {
        let arguments =
            Arguments("submit --kind feature-request --id FR-101 --by kevin --field oops");

        let error = Parse(&arguments).expect_err("must refuse");

        assert!(error.contains("--field"), "{error}");
    }

    #[test]
    fn Test_An_Unrecognised_Kind_Should_Be_A_Usage_Error()
    {
        let arguments = Arguments("submit --kind nonsense --id FR-102 --by kevin");

        let error = Parse(&arguments).expect_err("must refuse");

        assert!(error.contains("--kind"), "{error}");
    }

    #[test]
    fn Test_A_Gap_Should_Parse_Its_Blocked_Fields_And_Severity()
    {
        let arguments = Arguments(
            "submit --kind feature-request --id FR-103 --by kevin \
             --gap which-substrate|behaviour,goal|blocking",
        );

        let Command::Submit(request) = Parse(&arguments).expect("parses");

        assert_eq!(request.gaps.len(), 1);
        let gap = request.gaps.first().expect("one gap");
        assert_eq!(gap.question, "which-substrate");
        assert_eq!(gap.blocks, vec!["behaviour".to_owned(), "goal".to_owned()]);
        assert_eq!(gap.severity, Severity::Blocking);
        assert!(gap.closed_by.is_none());
    }

    #[test]
    fn Test_A_Parsed_Submission_Should_Carry_This_Transport_Name()
    {
        let arguments = Arguments(
            "submit --kind feature-request --id FR-104 --by kevin --field title=t",
        );

        let Command::Submit(request) = Parse(&arguments).expect("parses");

        assert_eq!(request.submitted_through, "cli");
    }
}
