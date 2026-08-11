//! What `nomos spec` was asked for, and the usage it prints when it cannot tell.

use super::{SpecCommand, Named_Value, Required, PathBuf, RecordRequest, EditRequest, TableRequest, RenderRequest, FreshnessRequest, Named_Values, CommitRequest};

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
pub(super) fn Required_Value(arguments: &[String], name: &str) -> Result<String, String>
{
    let value = Named_Value(arguments, name);

    return Required(value.as_ref(), name, &Usage_Text());
}

/// A flag with no default, read as a path.
pub(super) fn Required_Path(arguments: &[String], name: &str) -> Result<PathBuf, String>
{
    let value = Required_Value(arguments, name)?;

    return Ok(PathBuf::from(value));
}

/// Which record a `record` or `markdown` run is about.
pub(super) fn Parse_Record_Request(arguments: &[String]) -> Result<RecordRequest, String>
{
    return Ok(RecordRequest {
        id: Required_Value(arguments, "--id")?,
        revision: Named_Value(arguments, "--revision"),
    });
}

/// The edit a `preview` or `commit` run carries.
pub(super) fn Parse_Edit_Request(arguments: &[String]) -> Result<EditRequest, String>
{
    return Ok(EditRequest {
        id: Required_Value(arguments, "--id")?,
        from: Required_Path(arguments, "--from")?,
        rename: Named_Value(arguments, "--rename"),
    });
}

pub(super) fn Parse_Record(arguments: &[String]) -> Result<SpecCommand, String>
{
    let request = Parse_Record_Request(arguments)?;

    return Ok(SpecCommand::Record(request));
}

pub(super) fn Parse_Table(arguments: &[String]) -> Result<SpecCommand, String>
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

pub(super) fn Parse_Render(arguments: &[String]) -> Result<SpecCommand, String>
{
    return Ok(SpecCommand::Render(RenderRequest {
        profile: Required_Value(arguments, "--profile")?,
        into: Required_Path(arguments, "--into")?,
        subject: Named_Value(arguments, "--subject"),
    }));
}

pub(super) fn Parse_Freshness(arguments: &[String]) -> Result<SpecCommand, String>
{
    return Ok(SpecCommand::Freshness(FreshnessRequest {
        into: Required_Path(arguments, "--into")?,
        profile: Named_Value(arguments, "--profile"),
        require: Named_Values(arguments, "--require"),
    }));
}

pub(super) fn Parse_Markdown(arguments: &[String]) -> Result<SpecCommand, String>
{
    let request = Parse_Record_Request(arguments)?;

    return Ok(SpecCommand::Markdown(request));
}

pub(super) fn Parse_Preview(arguments: &[String]) -> Result<SpecCommand, String>
{
    let request = Parse_Edit_Request(arguments)?;

    return Ok(SpecCommand::Preview(request));
}

pub(super) fn Parse_Commit(arguments: &[String]) -> Result<SpecCommand, String>
{
    let edit = Parse_Edit_Request(arguments)?;
    let into = Named_Value(arguments, "--into");

    return Ok(SpecCommand::Commit(CommitRequest {
        edit,
        into: into.map_or_else(|| return PathBuf::from("."), PathBuf::from),
    }));
}

/// A whole-number flag, or a message saying what was given instead.
pub(super) fn Ordinal(value: Option<&String>, name: &str) -> Result<Option<u32>, String>
{
    let Some(text) = value
    else
    {
        return Ok(None);
    };

    return text
        .parse::<u32>()
        .map(Some)
        .map_err(|cause| return format!("{name} takes a whole number; `{text}` is not one: {cause}"));
}

pub(super) fn Usage_Text() -> String
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
