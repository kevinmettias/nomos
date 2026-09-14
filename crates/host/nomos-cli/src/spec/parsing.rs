//! What `nomos spec` was asked for, and the usage it prints when it cannot tell.

use super::{SpecCommand, Name, Named_Value_From_String_Arguments, Required_Value, PathBuf, RecordRequest, EditRequest, TableRequest, RenderRequest, FreshnessRequest, Named_Values_From_String_Arguments, CommitRequest, Usage};

/// Parses `nomos spec` arguments.
///
/// # Errors
///
/// Returns a message naming what was wrong and what was expected.
pub fn Spec_Command_From_String_Arguments(arguments: &[String]) -> Result<SpecCommand, String>
{
    let Some(verb) = arguments.first()
    else
    {
        return Err(Usage_Text());
    };

    return match verb.as_str()
    {
        "record" => Record_Command_From_String_Arguments(arguments),
        "table" => Table_Command_From_String_Arguments(arguments),
        "render" => Render_Command_From_String_Arguments(arguments),
        "freshness" => Freshness_Command_From_String_Arguments(arguments),
        "markdown" => Markdown_Command_From_String_Arguments(arguments),
        "preview" => Preview_Command_From_String_Arguments(arguments),
        "commit" => Commit_Command_From_String_Arguments(arguments),
        "profiles" => Ok(SpecCommand::Profiles),
        "sources" => Ok(SpecCommand::Sources),
        other => Err(format!("unknown command `{other}`.\n\n{}", Usage_Text())),
    };
}

/// A flag with no default, or a message naming it beside the usage.
pub(super) fn Required_Value_From_String_Arguments(arguments: &[String], name: &str) -> Result<String, String>
{
    let value = Named_Value_From_String_Arguments(arguments, name);

    return Required_Value(value.as_ref(), Name(name), Usage(&Usage_Text()));
}

/// A flag with no default, read as a path.
pub(super) fn Required_Path_From_String_Arguments(arguments: &[String], name: &str) -> Result<PathBuf, String>
{
    let value = Required_Value_From_String_Arguments(arguments, name)?;

    return Ok(PathBuf::from(value));
}

/// Which record a `record` or `markdown` run is about.
pub(super) fn Record_Request_From_String_Arguments(arguments: &[String]) -> Result<RecordRequest, String>
{
    return Ok(RecordRequest {
        id: Required_Value_From_String_Arguments(arguments, "--id")?,
        revision: Named_Value_From_String_Arguments(arguments, "--revision"),
    });
}

/// The edit a `preview` or `commit` run carries.
pub(super) fn Edit_Request_From_String_Arguments(arguments: &[String]) -> Result<EditRequest, String>
{
    return Ok(EditRequest {
        id: Required_Value_From_String_Arguments(arguments, "--id")?,
        from: Required_Path_From_String_Arguments(arguments, "--from")?,
        rename: Named_Value_From_String_Arguments(arguments, "--rename"),
    });
}

pub(super) fn Record_Command_From_String_Arguments(arguments: &[String]) -> Result<SpecCommand, String>
{
    let request = Record_Request_From_String_Arguments(arguments)?;

    return Ok(SpecCommand::Record(request));
}

pub(super) fn Table_Command_From_String_Arguments(arguments: &[String]) -> Result<SpecCommand, String>
{
    let block = Named_Value_From_String_Arguments(arguments, "--block");
    let table = Named_Value_From_String_Arguments(arguments, "--table");

    return Ok(SpecCommand::Table(TableRequest {
        document: Required_Value_From_String_Arguments(arguments, "--document")?,
        block: Parsed_Ordinal(block.as_ref(), "--block")?,
        table: Parsed_Ordinal(table.as_ref(), "--table")?,
        revision: Named_Value_From_String_Arguments(arguments, "--revision"),
    }));
}

pub(super) fn Render_Command_From_String_Arguments(arguments: &[String]) -> Result<SpecCommand, String>
{
    return Ok(SpecCommand::Render(RenderRequest {
        profile: Required_Value_From_String_Arguments(arguments, "--profile")?,
        into: Required_Path_From_String_Arguments(arguments, "--into")?,
        subject: Named_Value_From_String_Arguments(arguments, "--subject"),
    }));
}

pub(super) fn Freshness_Command_From_String_Arguments(arguments: &[String]) -> Result<SpecCommand, String>
{
    return Ok(SpecCommand::Freshness(FreshnessRequest {
        into: Required_Path_From_String_Arguments(arguments, "--into")?,
        profile: Named_Value_From_String_Arguments(arguments, "--profile"),
        require: Named_Values_From_String_Arguments(arguments, "--require"),
    }));
}

pub(super) fn Markdown_Command_From_String_Arguments(arguments: &[String]) -> Result<SpecCommand, String>
{
    let request = Record_Request_From_String_Arguments(arguments)?;

    return Ok(SpecCommand::Markdown(request));
}

pub(super) fn Preview_Command_From_String_Arguments(arguments: &[String]) -> Result<SpecCommand, String>
{
    let request = Edit_Request_From_String_Arguments(arguments)?;

    return Ok(SpecCommand::Preview(request));
}

pub(super) fn Commit_Command_From_String_Arguments(arguments: &[String]) -> Result<SpecCommand, String>
{
    let edit = Edit_Request_From_String_Arguments(arguments)?;
    let into = Named_Value_From_String_Arguments(arguments, "--into");

    return Ok(SpecCommand::Commit(CommitRequest {
        edit,
        into: into.map_or_else(|| return PathBuf::from("."), PathBuf::from),
    }));
}

/// A whole-number flag, or a message saying what was given instead.
pub(super) fn Parsed_Ordinal(value: Option<&String>, name: &str) -> Result<Option<u32>, String>
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

#[cfg(test)]
mod tests
{
    use super::*;

    /// The dispatcher itself: routing by verb, and its own fallback for one nothing names.
    ///
    /// `spec::tests` carries the fuller behavioral suite over every verb's own parse (one test per
    /// verb, each asserting what that verb's request looks like); this test stays with the
    /// declaration and covers only the dispatch this file's own top-level function performs.
    #[test]
    fn Test_Spec_Command_From_String_Arguments_Should_Dispatch_By_Verb()
    {
        assert_eq!(
            Spec_Command_From_String_Arguments(&Arguments("record --id D-129")).expect("parses"),
            SpecCommand::Record(RecordRequest { id: "D-129".to_owned(), revision: None })
        );
        assert_eq!(
            Spec_Command_From_String_Arguments(&Arguments("sources")).expect("parses"),
            SpecCommand::Sources
        );
        let error = Spec_Command_From_String_Arguments(&Arguments("frobnicate")).expect_err("must refuse");
        assert!(error.contains("frobnicate"), "{error}");
    }

    /// The usage text names some command at all.
    ///
    /// This used to be nine `contains` assertions, one per verb -- a third copy of a
    /// vocabulary `SpecCommand` declares, beside the enum and beside the list in
    /// `spec::tests`. It was weaker than either in two ways that compound: one direction, so
    /// a verb the parser accepts and the text omits passed, and substring rather than
    /// equality, so renaming the verb to `recordx` left `contains("record")` true. It was the
    /// one test that stayed green under the injection pass that caught seven others, which is
    /// how it was found.
    ///
    /// `spec::tests::Test_Usage_Text_Should_Name_Every_Command` is the comparison now: it
    /// drives every name the text prints through this parser and names the command each
    /// builds by an exhaustive match, so it asserts strictly more than the nine lines did.
    /// What is left here is the one claim that comparison cannot make, because it reads the
    /// text to find its subjects -- a usage text that lists no verb at all would leave it with
    /// nothing to compare and passing. `OD-AGENT-004` version 2 is why this is a floor rather
    /// than a second list.
    #[test]
    fn Test_Usage_Text_Should_Name_Some_Command_At_All()
    {
        let usage = Usage_Text();
        let (_, block) = usage.split_once("<command>\n").expect("the usage text has a verb block");
        let named = block
            .split_once("\ncommon:")
            .map_or(0, |(verbs, _)| return verbs.lines().filter(|line| return !line.trim().is_empty()).count());

        assert!(named > 0, "the usage text names no command: {usage}");
    }

    #[test]
    fn Test_Required_Value_From_String_Arguments_Should_Return_The_Flag_Or_Name_Itself_Missing()
    {
        assert_eq!(
            Required_Value_From_String_Arguments(&Arguments("--id D-129"), "--id").expect("present"),
            "D-129"
        );

        let error = Required_Value_From_String_Arguments(&Arguments(""), "--id").expect_err("must refuse");
        assert!(error.contains("--id"), "{error}");
        assert!(error.contains("usage"), "{error}");
    }

    #[test]
    fn Test_Required_Path_From_String_Arguments_Should_Read_The_Flags_Value_As_A_Path()
    {
        assert_eq!(
            Required_Path_From_String_Arguments(&Arguments("--into build"), "--into").expect("present"),
            PathBuf::from("build")
        );
        assert!(Required_Path_From_String_Arguments(&Arguments(""), "--into").is_err());
    }

    #[test]
    fn Test_Record_Request_From_String_Arguments_Should_Read_An_Optional_Revision()
    {
        assert_eq!(
            Record_Request_From_String_Arguments(&Arguments("--id D-129")).expect("parses"),
            RecordRequest { id: "D-129".to_owned(), revision: None }
        );
        assert_eq!(
            Record_Request_From_String_Arguments(&Arguments("--id D-129 --revision authored")).expect("parses"),
            RecordRequest { id: "D-129".to_owned(), revision: Some("authored".to_owned()) }
        );
    }

    #[test]
    fn Test_Edit_Request_From_String_Arguments_Should_Read_An_Optional_Rename()
    {
        assert_eq!(
            Edit_Request_From_String_Arguments(&Arguments("--id D-129 --from staged.md")).expect("parses"),
            EditRequest { id: "D-129".to_owned(), from: PathBuf::from("staged.md"), rename: None }
        );
        assert_eq!(
            Edit_Request_From_String_Arguments(&Arguments("--id D-129 --from staged.md --rename moved.md"))
                .expect("parses"),
            EditRequest {
                id: "D-129".to_owned(),
                from: PathBuf::from("staged.md"),
                rename: Some("moved.md".to_owned())
            }
        );
    }

    #[test]
    fn Test_Record_Command_From_String_Arguments_Should_Wrap_The_Request_In_SpecCommand_Record()
    {
        assert_eq!(
            Record_Command_From_String_Arguments(&Arguments("--id D-129")).expect("parses"),
            SpecCommand::Record(RecordRequest { id: "D-129".to_owned(), revision: None })
        );
    }

    #[test]
    fn Test_Table_Command_From_String_Arguments_Should_Parse_The_Optional_Block_And_Table_Ordinals()
    {
        assert_eq!(
            Table_Command_From_String_Arguments(&Arguments("--document x.md --block 2 --table 1")).expect("parses"),
            SpecCommand::Table(TableRequest {
                document: "x.md".to_owned(),
                block: Some(2),
                table: Some(1),
                revision: None
            })
        );
        assert!(Table_Command_From_String_Arguments(&Arguments("--document x.md --block seven")).is_err());
    }

    #[test]
    fn Test_Render_Command_From_String_Arguments_Should_Wrap_The_Request_In_SpecCommand_Render()
    {
        assert_eq!(
            Render_Command_From_String_Arguments(&Arguments("--profile github-markdown --into build"))
                .expect("parses"),
            SpecCommand::Render(RenderRequest {
                profile: "github-markdown".to_owned(),
                into: PathBuf::from("build"),
                subject: None
            })
        );
        assert!(Render_Command_From_String_Arguments(&Arguments("--into build")).is_err());
    }

    #[test]
    fn Test_Freshness_Command_From_String_Arguments_Should_Collect_Every_Requirement()
    {
        assert_eq!(
            Freshness_Command_From_String_Arguments(&Arguments(
                "--into build --require diagram-set --require html-site"
            ))
            .expect("parses"),
            SpecCommand::Freshness(FreshnessRequest {
                into: PathBuf::from("build"),
                profile: None,
                require: vec!["diagram-set".to_owned(), "html-site".to_owned()]
            })
        );
    }

    #[test]
    fn Test_Markdown_Command_From_String_Arguments_Should_Wrap_The_Request_In_SpecCommand_Markdown()
    {
        assert_eq!(
            Markdown_Command_From_String_Arguments(&Arguments("--id D-129")).expect("parses"),
            SpecCommand::Markdown(RecordRequest { id: "D-129".to_owned(), revision: None })
        );
    }

    #[test]
    fn Test_Preview_Command_From_String_Arguments_Should_Wrap_The_Request_In_SpecCommand_Preview()
    {
        assert_eq!(
            Preview_Command_From_String_Arguments(&Arguments("--id D-129 --from staged.md")).expect("parses"),
            SpecCommand::Preview(EditRequest {
                id: "D-129".to_owned(),
                from: PathBuf::from("staged.md"),
                rename: None
            })
        );
        assert!(Preview_Command_From_String_Arguments(&Arguments("--from staged.md")).is_err());
    }

    #[test]
    fn Test_Commit_Command_From_String_Arguments_Should_Default_Into_To_The_Current_Tree()
    {
        assert_eq!(
            Commit_Command_From_String_Arguments(&Arguments("--id D-129 --from staged.md")).expect("parses"),
            SpecCommand::Commit(CommitRequest {
                edit: EditRequest { id: "D-129".to_owned(), from: PathBuf::from("staged.md"), rename: None },
                into: PathBuf::from(".")
            })
        );
        assert_eq!(
            Commit_Command_From_String_Arguments(&Arguments("--id D-129 --from staged.md --into build"))
                .expect("parses"),
            SpecCommand::Commit(CommitRequest {
                edit: EditRequest { id: "D-129".to_owned(), from: PathBuf::from("staged.md"), rename: None },
                into: PathBuf::from("build")
            })
        );
    }

    #[test]
    fn Test_Parsed_Ordinal_Should_Accept_Absence_And_Whole_Numbers_And_Refuse_The_Rest()
    {
        assert_eq!(Parsed_Ordinal(None, "--block").expect("absent is fine"), None);
        assert_eq!(Parsed_Ordinal(Some(&"3".to_owned()), "--block").expect("parses"), Some(3));

        let error = Parsed_Ordinal(Some(&"seven".to_owned()), "--block").expect_err("must refuse");
        assert!(error.contains("--block"), "{error}");
        assert!(error.contains("seven"), "{error}");
    }

    fn Arguments(text: &str) -> Vec<String>
    {
        return text.split_whitespace().map(str::to_owned).collect();
    }
}
