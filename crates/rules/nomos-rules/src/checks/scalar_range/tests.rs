use super::*;
use nomos_contracts::SubjectId;
use nomos_model::Content_Digest;

#[test]
fn Test_Check_Nonnegative_Storage_Is_Unsigned_Should_Report_A_Signed_Field_With_A_Nonnegative_Minimum()
{
    let source = Source(Path("src/port.rs"), Text("struct Config {\n    #[validate(range(min = 0, max = 65535))]\n    port: i64,\n}\n"));
    let findings = Check_Nonnegative_Storage_Is_Unsigned(&[source]);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(NONNEGATIVE_STORAGE_IS_UNSIGNED));
}

#[test]
fn Test_Check_Nonnegative_Storage_Is_Unsigned_Should_Accept_An_Unsigned_Field_With_The_Same_Bound()
{
    let source = Source(Path("src/port.rs"), Text("struct Config {\n    #[validate(range(min = 0, max = 65535))]\n    port: u64,\n}\n"));
    let findings = Check_Nonnegative_Storage_Is_Unsigned(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_A_Known_Range_Picks_Its_Type_Should_Report_A_Field_Wider_Than_Its_Bound_Needs()
{
    let source = Source(Path("src/port.rs"), Text("struct Config {\n    #[validate(range(min = 0, max = 65535))]\n    port: i64,\n}\n"));
    let findings = Check_A_Known_Range_Picks_Its_Type(&[source]);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings.first().expect("asserted len 1 above").summary.contains("u16"), "{:?}", findings.first());
}

#[test]
fn Test_Check_A_Known_Range_Picks_Its_Type_Should_Accept_A_Field_At_Its_Narrowest_Type()
{
    let source = Source(Path("src/port.rs"), Text("struct Config {\n    #[validate(range(min = 0, max = 65535))]\n    port: u16,\n}\n"));
    let findings = Check_A_Known_Range_Picks_Its_Type(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_A_Known_Range_Picks_Its_Type_Should_Never_Bound_A_Float_Field()
{
    let source = Source(Path("src/port.rs"), Text("struct Config {\n    #[validate(range(min = 0, max = 1))]\n    ratio: f64,\n}\n"));
    let findings = Check_A_Known_Range_Picks_Its_Type(&[source]);

    assert!(findings.is_empty(), "floats are projected and never reported: {findings:?}");
}

#[test]
fn Test_Check_A_Known_Range_Picks_Its_Type_Should_Ignore_An_Attribute_With_No_Range_Clause()
{
    let source = Source(Path("src/port.rs"), Text("struct Config {\n    #[serde(rename = \"port\")]\n    port: i64,\n}\n"));
    let findings = Check_A_Known_Range_Picks_Its_Type(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_A_Known_Range_Picks_Its_Type_Should_Ignore_An_Inverted_Range()
{
    let source = Source(Path("src/port.rs"), Text("struct Config {\n    #[validate(range(min = 100, max = 0))]\n    port: i64,\n}\n"));
    let findings = Check_A_Known_Range_Picks_Its_Type(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_A_Known_Range_Picks_Its_Type_Should_Pair_With_The_Second_Of_Two_Attributes()
{
    let source = Source(
        Path("src/port.rs"),
        Text("struct Config {\n    #[serde(rename = \"port\")]\n    #[validate(range(min = 0, max = 65535))]\n    port: i64,\n}\n"),
    );
    let findings = Check_A_Known_Range_Picks_Its_Type(&[source]);

    assert_eq!(findings.len(), 1, "the range clause is the second attribute, not the first: {findings:?}");
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
