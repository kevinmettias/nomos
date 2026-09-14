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

/// Every code this group can leave the process with.
///
/// `spec::ExitCode` carries no census of its own the way `check::ExitCode::All()` does,
/// and its declaration is not this item's territory, so the census is here. [`Labelled`]'s
/// match has no wildcard arm, so a variant added to [`ExitCode`] fails this file to
/// *compile* rather than to pass — that is the forcing function that brings an author to
/// this array in the same edit. It is honestly weaker than a census living beside the
/// declaration, and it is what a test file can do without editing what it judges.
fn Every_Exit_Code() -> &'static [ExitCode]
{
    return &[
        ExitCode::Ok,
        ExitCode::NotFound,
        ExitCode::Usage,
        ExitCode::StoreError,
        ExitCode::Absent,
        ExitCode::Unwritable,
        ExitCode::Stale,
        ExitCode::Refused,
    ];
}

/// A code's name, as an exhaustive match, so that adding one stops the build here.
fn Labelled(code: ExitCode) -> &'static str
{
    return match code
    {
        ExitCode::Ok => "Ok",
        ExitCode::NotFound => "NotFound",
        ExitCode::Usage => "Usage",
        ExitCode::StoreError => "StoreError",
        ExitCode::Absent => "Absent",
        ExitCode::Unwritable => "Unwritable",
        ExitCode::Stale => "Stale",
        ExitCode::Refused => "Refused",
    };
}

/// A list of codes in ascending order, so that two of them can be compared as sets.
fn Sorted(codes: impl Iterator<Item = i32>) -> Vec<i32>
{
    let mut sorted: Vec<i32> = codes.collect();
    sorted.sort_unstable();

    return sorted;
}

/// The codes this group's help text documents are the codes this group can exit with.
///
/// The same comparison `check` and `gate` have each carried for a while, against this
/// group's own enum. Eight groups print an exit-code list and only those two mirrored it;
/// the other six were correct rather than guarded, which is a different thing, and
/// `OD-AGENT-004`'s amendment says a printed vocabulary is admissible only where a test
/// compares it against its authority. The usage text is prose a person reads and
/// [`ExitCode`] is what the process returns, the two were written separately, and a code
/// added or renumbered in one of them and not the other is the failure that actually
/// happens.
///
/// Apart from `Test_Value_Should_Be_Stable_And_Not_Collide_With_Works_Claim_Codes` above,
/// which pins the numbers against `work`'s claim codes and says nothing about what this
/// group prints. This one compares the printed list against the enum and says nothing
/// about the numbers being right.
#[test]
fn Test_The_Documented_Exit_Codes_Should_Be_The_Ones_This_Group_Can_Exit_With()
{
    let usage = super::parsing::Usage_Text();

    assert!(
        usage.starts_with("usage: nomos spec"),
        "this compared some other group's help text: {usage}"
    );

    let (_, spelled) = usage
        .split_once("exit codes:")
        .expect("the usage text documents the exit codes");
    let documented = Sorted(spelled.split_whitespace().filter_map(|word| return word.parse().ok()));
    let implemented = Sorted(Every_Exit_Code().iter().map(|code| return code.Value()));

    assert!(
        !documented.is_empty(),
        "no exit code was parsed out of the usage text, so this compared nothing: {spelled}"
    );
    assert_eq!(
        documented,
        implemented,
        "the usage text and ExitCode disagree about what this command can exit with; the \
         enum declares {:?}",
        Every_Exit_Code().iter().map(|code| return Labelled(*code)).collect::<Vec<_>>()
    );
}

/// The verb each command name in the usage text sits on, first token of its own line.
///
/// The verb block is everything between the `<command>` header and the `common:` line, and
/// every verb occupies one rendered line beginning with its own name. Read out of the text
/// rather than listed here: a hand list beside a compiled vocabulary is the second authority
/// `OD-AGENT-004` is about, and this replaced one.
fn Named_Commands(usage: &str) -> Vec<String>
{
    let Some((_, after_header)) = usage.split_once("<command>\n")
    else
    {
        return Vec::new();
    };
    let Some((block, _)) = after_header.split_once("\ncommon:")
    else
    {
        return Vec::new();
    };

    return block
        .lines()
        .filter_map(|line| return line.split_whitespace().next())
        .map(str::to_owned)
        .collect();
}

/// The command a name builds, as an exhaustive match, so that adding one stops the build here.
///
/// This is what makes the comparison below closed on the authority's side. A variant added to
/// [`SpecCommand`] fails this file to *compile*, which is the forcing function that brings an
/// author to [`Minimal_Line`] and to the usage text in the same edit.
fn Named(command: &SpecCommand) -> &'static str
{
    return match *command
    {
        SpecCommand::Record(_) => "record",
        SpecCommand::Table(_) => "table",
        SpecCommand::Render(_) => "render",
        SpecCommand::Freshness(_) => "freshness",
        SpecCommand::Markdown(_) => "markdown",
        SpecCommand::Preview(_) => "preview",
        SpecCommand::Commit(_) => "commit",
        SpecCommand::Profiles => "profiles",
        SpecCommand::Sources => "sources",
    };
}

/// The shortest argument list each verb accepts, so the parser can be asked what it builds.
///
/// [`None`] for a name with no line rather than a panic here, so the test reports *which* verb
/// the usage text grew without a fixture — the failure a new command actually causes.
fn Minimal_Line(verb: &str) -> Option<&'static str>
{
    return match verb
    {
        "record" => Some("record --id D-129"),
        "table" => Some("table --document d"),
        "render" => Some("render --profile p --into ."),
        "freshness" => Some("freshness --into ."),
        "markdown" => Some("markdown --id D-129"),
        "preview" => Some("preview --id D-129 --from f"),
        "commit" => Some("commit --id D-129 --from f"),
        "profiles" => Some("profiles"),
        "sources" => Some("sources"),
        _ => None,
    };
}

/// The commands the usage text names are the commands this group can build, both directions.
///
/// This replaces a check that compared the text against a nine-name array written beside it, in
/// one direction. That array was a second authority for a vocabulary
/// `nomos_spec_orchestration::SpecCommand` already declares, and one direction meant a verb the
/// parser accepts and the text omits passed — which is exactly the shape of the defect
/// `OD-AGENT-004` version 2 was amended for.
///
/// Each name is now driven through the real parser and the command it builds is named by an
/// exhaustive match, so a name in the text that the parser refuses fails, a name that builds
/// the wrong command fails, and a variant added to [`SpecCommand`] stops the build.
#[test]
fn Test_Usage_Text_Should_Name_Every_Command()
{
    let named = Named_Commands(&super::parsing::Usage_Text());

    assert!(
        !named.is_empty(),
        "no command was parsed out of the usage text, so this compared nothing: {}",
        super::parsing::Usage_Text()
    );

    let mut built = Vec::new();
    for verb in &named
    {
        let line = Minimal_Line(verb)
            .unwrap_or_else(|| panic!("the usage text names `{verb}` and no minimal line is written for it here"));
        let command = Spec_Command_From_String_Arguments(&Arguments(line))
            .unwrap_or_else(|error| panic!("the usage text names `{verb}` and the parser refuses it: {error}"));

        assert_eq!(Named(&command), verb, "`{verb}` builds a different command");
        built.push(Named(&command));
    }

    built.sort_unstable();
    built.dedup();

    assert_eq!(built.len(), named.len(), "two command names built the same command: {named:?}");
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
