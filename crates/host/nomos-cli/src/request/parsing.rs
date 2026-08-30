//! What `nomos request` was asked to do.

use super::Command;
use crate::arguments::{Name, Named_Value_From_String_Arguments, Named_Values_From_String_Arguments, Required_Value, Usage};
use nomos_spec_model::{DecisionGap, Severity, SubmissionKind, SubmissionState};

/// Parses `nomos request` arguments.
///
/// # Errors
///
/// Returns a message naming what was wrong and what was expected.
pub(crate) fn Command_From_String_Arguments(arguments: &[String]) -> Result<Command, String>
{
    let Some(verb) = arguments.first()
    else
    {
        return Err(Usage_Text());
    };

    let rest = arguments.get(1..).unwrap_or_default();

    return match verb.as_str()
    {
        "submit" => Submit_Command_From_String_Arguments(rest),
        other => Err(format!("unknown command `{other}`.\n\n{}", Usage_Text())),
    };
}

fn Submit_Command_From_String_Arguments(arguments: &[String]) -> Result<Command, String>
{
    use nomos_spec_orchestration::SubmitRequest;

    let (kind, id, by) = Required_Submission_Fields_From_String_Arguments(arguments)?;
    let (state, contract_version) = Defaulted_Submission_Fields_From_String_Arguments(arguments)?;
    let (fields, gaps, into) = Submission_Collections_From_String_Arguments(arguments)?;

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

/// The three values `--kind`, `--id` and `--by` name, with no default: every `SubmitRequest`
/// this transport builds must carry all three.
fn Required_Submission_Fields_From_String_Arguments(arguments: &[String]) -> Result<(SubmissionKind, String, String), String>
{
    let kind_text = Required_Value_From_String_Arguments(arguments, "--kind")?;
    let kind = Parse_Kind(&kind_text)?;
    let id = Required_Value_From_String_Arguments(arguments, "--id")?;
    let by = Required_Value_From_String_Arguments(arguments, "--by")?;

    return Ok((kind, id, by));
}

fn Required_Value_From_String_Arguments(arguments: &[String], name: &str) -> Result<String, String>
{
    let value = Named_Value_From_String_Arguments(arguments, name);

    return Required_Value(value.as_ref(), Name(name), Usage(&Usage_Text()));
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

/// `--state` and `--contract-version`, each falling back to its own default when the caller
/// does not give it -- a fact about this run, decided before either field is read, per
/// `request`'s own module doc.
fn Defaulted_Submission_Fields_From_String_Arguments(arguments: &[String]) -> Result<(SubmissionState, u32), String>
{
    let state = Parse_State(Named_Value_From_String_Arguments(arguments, "--state").as_deref())?;
    let contract_version =
        Parse_Contract_Version(Named_Value_From_String_Arguments(arguments, "--contract-version").as_deref())?;

    return Ok((state, contract_version));
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

/// The repeatable `--field` and `--gap` flags, and the optional `--into` directory.
fn Submission_Collections_From_String_Arguments(
    arguments: &[String],
) -> Result<(Vec<(String, String)>, Vec<DecisionGap>, Option<std::path::PathBuf>), String>
{
    let fields = Fields_From_String_Arguments(arguments)?;
    let gaps = Gaps_From_String_Arguments(arguments)?;
    let into = Named_Value_From_String_Arguments(arguments, "--into").map(std::path::PathBuf::from);

    return Ok((fields, gaps, into));
}

/// Every `--field name=value`, in the order they were given.
fn Fields_From_String_Arguments(arguments: &[String]) -> Result<Vec<(String, String)>, String>
{
    let mut fields = Vec::new();

    for entry in Named_Values_From_String_Arguments(arguments, "--field")
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
fn Gaps_From_String_Arguments(arguments: &[String]) -> Result<Vec<DecisionGap>, String>
{
    let mut gaps = Vec::new();

    for entry in Named_Values_From_String_Arguments(arguments, "--gap")
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

pub(super) fn Usage_Text() -> String
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

    #[test]
    fn Test_Command_From_String_Arguments_Should_Parse_A_Complete_Submit()
    {
        let arguments: Vec<String> = [
            "submit", "--kind", "feature-request", "--id", "FR-1", "--by", "kevin", "--field", "title=hello",
        ]
        .iter()
        .map(|value| return (*value).to_owned())
        .collect();

        let Command::Submit(submit) = Command_From_String_Arguments(&arguments).expect("parses");

        assert_eq!(submit.kind, SubmissionKind::FeatureRequest);
        assert_eq!(submit.id, "FR-1");
        assert_eq!(submit.by, "kevin");
        assert_eq!(submit.state, SubmissionState::Draft);
        assert_eq!(submit.contract_version, 1);
        assert_eq!(submit.fields, vec![("title".to_owned(), "hello".to_owned())]);
    }

    #[test]
    fn Test_Command_From_String_Arguments_Should_Refuse_An_Empty_Argument_List()
    {
        let error = Command_From_String_Arguments(&[]).expect_err("no verb at all");

        assert!(error.starts_with("usage: nomos request"), "{error}");
    }

    #[test]
    fn Test_Command_From_String_Arguments_Should_Refuse_An_Unknown_Verb()
    {
        let arguments = vec!["not-a-real-verb".to_owned()];

        let error = Command_From_String_Arguments(&arguments).expect_err("no such verb");

        assert!(error.contains("unknown command"), "{error}");
    }
}
