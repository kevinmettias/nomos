use super::*;
use super::text::Re_Export;
use nomos_contracts::SubjectId;
use nomos_model::Content_Digest;

#[test]
fn Test_Check_A_Facade_Publishes_A_Child_One_Way_Should_Report_A_Child_Published_Twice()
{
    let source = Source(Path("src/pipeline.rs"), Text("pub mod pass_outcome;\npub use pass_outcome::PassOutcome;\n"));

    let findings = Check_A_Facade_Publishes_A_Child_One_Way(&[source]);

    assert_eq!(findings.len(), 1, "{findings:?}");
    let reported = findings.first().expect("asserted len 1 above");
    assert_eq!(reported.rule, RuleId::New(FACADE_CHOOSES_FLATTENING_OR_NAMESPACE));
    assert_eq!(reported.subject_name, "src/pipeline.rs:2");
}

#[test]
fn Test_Check_A_Facade_Publishes_A_Child_One_Way_Should_Accept_A_Private_Child_Lifted_Onto_The_Parent()
{
    let source = Source(Path("src/pipeline.rs"), Text("mod pass_outcome;\npub use pass_outcome::PassOutcome;\n"));

    let findings = Check_A_Facade_Publishes_A_Child_One_Way(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_A_Facade_Publishes_A_Child_One_Way_Should_Accept_A_Published_Namespace_Alone()
{
    let source = Source(Path("src/pipeline.rs"), Text("pub mod pass_outcome;\n"));

    let findings = Check_A_Facade_Publishes_A_Child_One_Way(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_A_Facade_Publishes_A_Child_One_Way_Should_Judge_Restricted_Visibility_Too()
{
    let source = Source(Path("src/pipeline.rs"), Text("pub(crate) mod pass_outcome;\npub(crate) use pass_outcome::PassOutcome;\n"));

    let findings = Check_A_Facade_Publishes_A_Child_One_Way(&[source]);

    assert_eq!(findings.len(), 1, "a facade is a boundary even inside the crate: {findings:?}");
}

#[test]
fn Test_Check_A_Facade_Publishes_A_Child_One_Way_Should_Ignore_A_Commented_Out_Declaration()
{
    let source = Source(Path("src/pipeline.rs"), Text("// pub mod pass_outcome;\npub use pass_outcome::PassOutcome;\n"));

    let findings = Check_A_Facade_Publishes_A_Child_One_Way(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_A_Facade_Publishes_A_Child_One_Way_Should_Not_Judge_A_File_In_Another_Language()
{
    let source = Source(Path("src/pipeline.go"), Text("pub mod pass_outcome;\npub use pass_outcome::PassOutcome;\n"));

    let findings = Check_A_Facade_Publishes_A_Child_One_Way(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_A_Renamed_Facade_Re_Export_Names_The_Contract_Should_Report_An_Unexplained_Alias()
{
    let source = Source(Path("src/pipeline.rs"), Text("pub use pass_outcome::Internal as PassOutcome;\n"));

    let findings = Check_A_Renamed_Facade_Re_Export_Names_The_Contract(&[source]);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(FACADE_ALIASES_NAME_THE_CONTRACT));
}

#[test]
fn Test_Check_A_Renamed_Facade_Re_Export_Names_The_Contract_Should_Accept_A_Reason_Above_It()
{
    let source = Source(
        Path("src/pipeline.rs"),
        Text("// facade-alias: allow: PassOutcome is the surface vocabulary this crate publishes\npub use pass_outcome::Internal as PassOutcome;\n"),
    );

    let findings = Check_A_Renamed_Facade_Re_Export_Names_The_Contract(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_A_Renamed_Facade_Re_Export_Names_The_Contract_Should_Accept_A_Reason_Beside_It()
{
    let source = Source(
        Path("src/pipeline.rs"),
        Text("pub use pass_outcome::Internal as PassOutcome; // facade-alias: allow: the migration name callers still bind to\n"),
    );

    let findings = Check_A_Renamed_Facade_Re_Export_Names_The_Contract(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_A_Renamed_Facade_Re_Export_Names_The_Contract_Should_Refuse_A_Marker_With_No_Reason()
{
    let source = Source(Path("src/pipeline.rs"), Text("// facade-alias: allow:\npub use pass_outcome::Internal as PassOutcome;\n"));

    let findings = Check_A_Renamed_Facade_Re_Export_Names_The_Contract(&[source]);

    assert_eq!(findings.len(), 1, "a marker states a reason or it states nothing: {findings:?}");
}

#[test]
fn Test_Check_A_Renamed_Facade_Re_Export_Names_The_Contract_Should_Accept_A_Re_Export_That_Renames_Nothing()
{
    let source = Source(Path("src/pipeline.rs"), Text("pub use pass_outcome::PassOutcome;\n"));

    let findings = Check_A_Renamed_Facade_Re_Export_Names_The_Contract(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_A_Consumer_Imports_Through_The_Facade_Should_Report_An_Import_Around_The_Facade()
{
    let facade = Source(Path("src/pipeline.rs"), Text("pub use pass_outcome::PassOutcome;\n"));
    let consumer = Source(Path("src/runner.rs"), Text("use crate::pipeline::pass_outcome::PassOutcome;\n"));

    let findings = Check_A_Consumer_Imports_Through_The_Facade(&[facade, consumer]);

    assert_eq!(findings.len(), 1, "{findings:?}");
    let reported = findings.first().expect("asserted len 1 above");
    assert_eq!(reported.rule, RuleId::New(FACADE_CONSUMERS_USE_THE_FACADE_PATH));
    assert!(reported.summary.contains("crate::pipeline::PassOutcome"), "names the canonical path: {reported:?}");
}

#[test]
fn Test_Check_A_Consumer_Imports_Through_The_Facade_Should_Accept_The_Facade_Path()
{
    let facade = Source(Path("src/pipeline.rs"), Text("pub use pass_outcome::PassOutcome;\n"));
    let consumer = Source(Path("src/runner.rs"), Text("use crate::pipeline::PassOutcome;\n"));

    let findings = Check_A_Consumer_Imports_Through_The_Facade(&[facade, consumer]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_A_Consumer_Imports_Through_The_Facade_Should_Report_A_Braced_Import_Around_The_Facade()
{
    let facade = Source(Path("src/pipeline.rs"), Text("pub use pass_outcome::PassOutcome;\n"));
    let consumer = Source(Path("src/runner.rs"), Text("use crate::pipeline::pass_outcome::{PassOutcome, Other};\n"));

    let findings = Check_A_Consumer_Imports_Through_The_Facade(&[facade, consumer]);

    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn Test_Check_A_Consumer_Imports_Through_The_Facade_Should_Root_A_Crate_Root_Facade_At_The_Crate()
{
    let facade = Source(Path("src/lib.rs"), Text("pub use pass_outcome::PassOutcome;\n"));
    let consumer = Source(Path("src/runner.rs"), Text("use crate::pass_outcome::PassOutcome;\n"));

    let findings = Check_A_Consumer_Imports_Through_The_Facade(&[facade, consumer]);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        findings.first().expect("asserted len 1 above").summary.contains("crate::PassOutcome"),
        "a crate root publishes at the crate itself: {findings:?}"
    );
}

#[test]
fn Test_Check_A_Consumer_Imports_Through_The_Facade_Should_Read_A_Mod_File_As_Its_Own_Directory()
{
    let facade = Source(Path("src/pipeline/mod.rs"), Text("pub use pass_outcome::PassOutcome;\n"));
    let consumer = Source(Path("src/runner.rs"), Text("use crate::pipeline::pass_outcome::PassOutcome;\n"));

    let findings = Check_A_Consumer_Imports_Through_The_Facade(&[facade, consumer]);

    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn Test_Check_A_Consumer_Imports_Through_The_Facade_Should_Publish_Nothing_From_Outside_A_Source_Directory()
{
    let facade = Source(Path("tests/harness.rs"), Text("pub use pass_outcome::PassOutcome;\n"));
    let consumer = Source(Path("src/runner.rs"), Text("use crate::pass_outcome::PassOutcome;\n"));

    let findings = Check_A_Consumer_Imports_Through_The_Facade(&[facade, consumer]);

    assert!(findings.is_empty(), "a file outside the module tree roots no facade: {findings:?}");
}

#[test]
fn Test_Check_A_Consumer_Imports_Through_The_Facade_Should_Not_Judge_The_Facades_Own_Re_Export()
{
    let facade = Source(Path("src/pipeline.rs"), Text("mod pass_outcome;\npub use pass_outcome::PassOutcome;\n"));

    let findings = Check_A_Consumer_Imports_Through_The_Facade(&[facade]);

    assert!(findings.is_empty(), "a facade publishing its own surface is not a consumer: {findings:?}");
}

#[test]
fn Test_Re_Export_Should_Read_A_Raw_Identifier_As_The_Module_It_Names()
{
    let parsed = Re_Export("pub use r#match::Matcher;").expect("a raw identifier is still a path segment");

    assert_eq!(parsed.path, vec!["match", "Matcher"]);
    assert_eq!(parsed.alias, None);
}

#[test]
fn Test_Re_Export_Should_Leave_A_Brace_Group_Undecided()
{
    assert!(Re_Export("pub use pass_outcome::{One, Two};").is_none());
}

/// The fixture's path position, named so a call site cannot transpose it with the text.
struct Path<'a>(&'a str);

/// The fixture's text position, named so a call site cannot transpose it with the path.
struct Text<'a>(&'a str);

fn Source(path: Path<'_>, text: Text<'_>) -> SourceFile
{
    let mut source = SourceFile::New(path.0, SubjectId::From_Digest(Content_Digest(path.0.as_bytes())), text.0);
    source.language = crate::Recognized_Language_In_Tests(path.0);
    return source;
}
