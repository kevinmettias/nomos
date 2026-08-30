//! What this module promises, exercised.

use super::*;

fn Arguments(text: &str) -> Vec<String>
{
    return text.split_whitespace().map(str::to_owned).collect();
}

#[test]
fn Test_Record_Should_Parse_With_And_Without_A_Revision()
{
    assert_eq!(
        Spec_Command_From_String_Arguments(&Arguments("record --id D-129")).expect("parses"),
        SpecCommand::Record(RecordRequest {
            id: "D-129".to_owned(),
            revision: None,
        })
    );
    assert_eq!(
        Spec_Command_From_String_Arguments(&Arguments("record --id D-129 --revision authored")).expect("parses"),
        SpecCommand::Record(RecordRequest {
            id: "D-129".to_owned(),
            revision: Some("authored".to_owned()),
        })
    );
}

#[test]
fn Test_A_Missing_Argument_Should_Name_Itself()
{
    let error = Spec_Command_From_String_Arguments(&Arguments("record")).expect_err("must refuse");

    assert!(error.contains("--id"), "{error}");
    assert!(error.contains("usage"), "{error}");
}

/// A block ordinal that is not a number would otherwise become "no such block", which
/// sends the reader looking for a block instead of at what they typed.
#[test]
fn Test_A_Non_Numeric_Ordinal_Should_Be_Refused()
{
    let error = Spec_Command_From_String_Arguments(&Arguments("table --document x.md --block seven"))
        .expect_err("must refuse");

    assert!(error.contains("--block"), "{error}");
    assert!(error.contains("seven"), "{error}");
}

#[test]
fn Test_Render_Should_Require_Both_A_Profile_And_A_Destination()
{
    assert!(Spec_Command_From_String_Arguments(&Arguments("render --profile github-markdown")).is_err());
    assert!(Spec_Command_From_String_Arguments(&Arguments("render --into build")).is_err());
    assert_eq!(
        Spec_Command_From_String_Arguments(&Arguments("render --profile github-markdown --into build")).expect("parses"),
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
        Spec_Command_From_String_Arguments(&Arguments("freshness --into build")).expect("parses"),
        SpecCommand::Freshness(FreshnessRequest {
            into: PathBuf::from("build"),
            profile: None,
            require: Vec::new(),
        })
    );
    assert_eq!(
        Spec_Command_From_String_Arguments(&Arguments("freshness --into build --profile mcp-resource")).expect("parses"),
        SpecCommand::Freshness(FreshnessRequest {
            into: PathBuf::from("build"),
            profile: Some("mcp-resource".to_owned()),
            require: Vec::new(),
        })
    );
    assert!(Spec_Command_From_String_Arguments(&Arguments("freshness --profile mcp-resource")).is_err());
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
        Spec_Command_From_String_Arguments(&Arguments("render --profile subject-dossier --into . --subject D-129"))
            .expect("parses"),
        SpecCommand::Render(RenderRequest {
            profile: "subject-dossier".to_owned(),
            into: PathBuf::from("."),
            subject: Some("D-129".to_owned()),
        })
    );
    assert_eq!(
        Spec_Command_From_String_Arguments(&Arguments("render --profile diagram-set --into .")).expect("parses"),
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
        Spec_Command_From_String_Arguments(&Arguments(
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
fn Test_Spec_Command_From_String_Arguments_Should_Refuse_An_Unknown_Command()
{
    let error = Spec_Command_From_String_Arguments(&Arguments("frobnicate")).expect_err("must refuse");

    assert!(error.contains("frobnicate"), "{error}");
}

/// Every code `work` already spends on a claim outcome, so a second test can point at it
/// without repeating the pair inline.
fn Works_Claim_Codes() -> [i32; 2]
{
    return [
        crate::work::ExitCode::ClaimUnavailable.Value(),
        crate::work::ExitCode::Conflict.Value(),
    ];
}

/// The codes are a contract, and they are the binary's rather than the group's. `3`
/// and `4` belong to `work`'s claim refusals and must not acquire a second meaning.
#[test]
fn Test_Value_Should_Be_Stable_And_Not_Collide_With_Works_Claim_Codes()
{
    assert_eq!(ExitCode::Ok.Value(), crate::work::ExitCode::Ok.Value());
    assert_eq!(ExitCode::Usage.Value(), crate::work::ExitCode::Usage.Value());
    assert_eq!(ExitCode::NotFound.Value(), 1);
    assert_eq!(ExitCode::StoreError.Value(), 5);
    assert_eq!(ExitCode::Absent.Value(), 6);
    assert_eq!(ExitCode::Unwritable.Value(), 7);
    assert_eq!(ExitCode::Stale.Value(), 8);
    assert_eq!(ExitCode::Refused.Value(), 9);

    for taken in Works_Claim_Codes()
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

/// Every `nomos spec` command name the usage text must mention.
fn Every_Spec_Command_Name() -> [&'static str; 9]
{
    return [
        "record",
        "table",
        "render",
        "freshness",
        "markdown",
        "preview",
        "commit",
        "profiles",
        "sources",
    ];
}

#[test]
fn Test_Usage_Text_Should_Name_Every_Command()
{
    use super::parsing::Usage_Text;

    let usage = Usage_Text();

    for command in Every_Spec_Command_Name()
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
        Spec_Command_From_String_Arguments(&Arguments("commit --id D-129 --from staged.md")).expect("parses"),
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
        Spec_Command_From_String_Arguments(&Arguments(
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
    assert!(Spec_Command_From_String_Arguments(&Arguments("commit --id D-129")).is_err());
}

#[test]
fn Test_Markdown_And_Preview_Should_Spec_Command_From_String_Arguments()
{
    assert_eq!(
        Spec_Command_From_String_Arguments(&Arguments("markdown --id D-129")).expect("parses"),
        SpecCommand::Markdown(RecordRequest {
            id: "D-129".to_owned(),
            revision: None,
        })
    );
    assert_eq!(
        Spec_Command_From_String_Arguments(&Arguments("preview --id D-129 --from staged.md")).expect("parses"),
        SpecCommand::Preview(EditRequest {
            id: "D-129".to_owned(),
            from: PathBuf::from("staged.md"),
            rename: None,
        })
    );
    assert!(Spec_Command_From_String_Arguments(&Arguments("preview --from staged.md")).is_err());
}

/// `markdown` is not `record` with a flag, and the parse is where that stays true.
#[test]
fn Test_Reading_Bytes_And_Rendering_Markdown_Should_Be_Different_Commands()
{
    assert_ne!(
        Spec_Command_From_String_Arguments(&Arguments("record --id D-129")).expect("parses"),
        Spec_Command_From_String_Arguments(&Arguments("markdown --id D-129")).expect("parses")
    );
}
