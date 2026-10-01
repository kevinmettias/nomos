use super::*;
use nomos_contracts::SubjectId;
use nomos_model::Content_Digest;

#[test]
fn Test_Check_No_Trailing_Whitespace_Should_Report_A_Line_Ending_In_A_Space()
{
    let source = Source(Path("src/lib.rs"), Text("fn Clean()\n{\n    return; \n}\n"));

    let findings = Check_No_Trailing_Whitespace(&[source]);

    assert_eq!(findings.len(), 1, "{findings:?}");
    let found = findings.first().expect("asserted len 1 above");
    assert_eq!(found.rule, RuleId::New(NO_TRAILING_WHITESPACE));
    assert_eq!(found.subject_name, "src/lib.rs:3");
    assert_eq!(found.locations, vec!["src/lib.rs:3".to_owned()]);
    assert_eq!(found.gate, GateCategory::Blocking);
}

#[test]
fn Test_Check_No_Trailing_Whitespace_Should_Report_A_Line_Ending_In_A_Tab()
{
    let source = Source(Path("src/lib.rs"), Text("fn Clean()\n{\t\n}\n"));

    let findings = Check_No_Trailing_Whitespace(&[source]);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "src/lib.rs:2");
}

#[test]
fn Test_Check_No_Trailing_Whitespace_Should_Report_The_Final_Line_Without_A_Newline()
{
    let source = Source(Path("src/lib.rs"), Text("fn Clean() {} "));

    let findings = Check_No_Trailing_Whitespace(&[source]);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "src/lib.rs:1");
}

#[test]
fn Test_Check_No_Trailing_Whitespace_Should_Accept_Clean_Text()
{
    let source = Source(Path("src/lib.rs"), Text("fn Clean()\n{\n    return;\n}\n"));

    let findings = Check_No_Trailing_Whitespace(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Todo_Format_Should_Report_A_Todo_Without_Owner_Description_And_Ticket()
{
    let source = Source(Path("src/lib.rs"), Text("// TODO fix this later\n"));

    let findings = Check_Todo_Format(&[source]);

    assert_eq!(findings.len(), 1, "{findings:?}");
    let found = findings.first().expect("asserted len 1 above");
    assert_eq!(found.rule, RuleId::New(TODO_FORMAT));
    assert_eq!(found.subject_name, "src/lib.rs:1");
    assert_eq!(found.gate, GateCategory::Blocking);
}

#[test]
fn Test_Check_Todo_Format_Should_Accept_A_Tracked_Todo()
{
    let source = Source(Path("src/lib.rs"), Text("// TODO(kevin): replace fallback path once selection lands (#142)\n"));

    let findings = Check_Todo_Format(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Todo_Format_Should_Ignore_A_String_Containing_Todo()
{
    let source = Source(Path("src/lib.rs"), Text("let label = \"TODO fix this\";\n"));

    let findings = Check_Todo_Format(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Todo_Format_Should_Report_A_Todo_With_No_Ticket()
{
    let source = Source(Path("src/lib.rs"), Text("/// TODO(kevin): replace fallback path\n"));

    let findings = Check_Todo_Format(&[source]);

    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn Test_Check_Todo_Format_Should_Ignore_A_Todo_Mentioned_Mid_Comment()
{
    let source = Source(Path("src/lib.rs"), Text("// see TODO(kevin): fix later (#142) above\n"));

    let findings = Check_Todo_Format(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_No_Decorative_Section_Dividers_Should_Report_A_Bare_Divider()
{
    let source = Source(Path("src/lib.rs"), Text("// ====================\n"));

    let findings = Check_No_Decorative_Section_Dividers(&[source]);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(NO_DECORATIVE_SECTION_DIVIDERS));
}

#[test]
fn Test_Check_No_Decorative_Section_Dividers_Should_Report_A_Labelled_Divider()
{
    let source = Source(Path("src/lib.rs"), Text("// ===== Internal Helpers =====\n"));

    let findings = Check_No_Decorative_Section_Dividers(&[source]);

    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn Test_Check_No_Decorative_Section_Dividers_Should_Ignore_Prose_That_Quotes_A_Divider()
{
    let source = Source(Path("src/lib.rs"), Text("// The old code used // ===== Setup ===== as a divider.\n"));

    let findings = Check_No_Decorative_Section_Dividers(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Deprecation_Carries_A_Reason_Should_Report_A_Bare_Rust_Marker()
{
    let source = Source(Path("src/lib.rs"), Text("#[deprecated]\npub fn Old() {}\n"));

    let findings = Check_Deprecation_Carries_A_Reason(&[source]);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(DEPRECATION));
}

#[test]
fn Test_Check_Deprecation_Carries_A_Reason_Should_Report_Since_With_No_Note()
{
    let source = Source(Path("src/lib.rs"), Text("#[deprecated(since = \"2.1\")]\n"));

    let findings = Check_Deprecation_Carries_A_Reason(&[source]);

    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn Test_Check_Deprecation_Carries_A_Reason_Should_Accept_A_Note()
{
    let source = Source(Path("src/lib.rs"), Text("#[deprecated(since = \"2.1\", note = \"use New instead\")]\n"));

    let findings = Check_Deprecation_Carries_A_Reason(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Deprecation_Carries_A_Reason_Should_Ignore_A_Multiline_Attribute()
{
    let source = Source(Path("src/lib.rs"), Text("#[deprecated(\n    note = \"use New instead\"\n)]\n"));

    let findings = Check_Deprecation_Carries_A_Reason(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Deprecation_Carries_A_Reason_Should_Report_A_Bare_Go_Marker()
{
    let source = Source(Path("main.go"), Text("// Deprecated:\nfunc Old() {}\n"));

    let findings = Check_Deprecation_Carries_A_Reason(&[source]);

    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn Test_Check_Deprecation_Carries_A_Reason_Should_Accept_A_Go_Marker_With_A_Reason()
{
    let source = Source(Path("main.go"), Text("// Deprecated: use New instead.\n"));

    let findings = Check_Deprecation_Carries_A_Reason(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Deprecation_Carries_A_Reason_Should_Ignore_A_String_Containing_The_Marker()
{
    let source = Source(Path("src/lib.rs"), Text("let label = \"Deprecated: nothing\";\n"));

    let findings = Check_Deprecation_Carries_A_Reason(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_No_Single_Line_Function_Bodies_Should_Report_A_Collapsed_Body()
{
    let source = Source(Path("src/lib.rs"), Text("pub fn Add(a: i32, b: i32) -> i32 { return a + b; }\n"));

    let findings = Check_No_Single_Line_Function_Bodies(&[source]);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(NO_SINGLE_LINE_FUNCTION_BODIES));
}

#[test]
fn Test_Check_No_Single_Line_Function_Bodies_Should_Report_A_Collapsed_Empty_Body()
{
    let source = Source(Path("src/lib.rs"), Text("pub fn Noop() {}\n"));

    let findings = Check_No_Single_Line_Function_Bodies(&[source]);

    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn Test_Check_No_Single_Line_Function_Bodies_Should_Accept_An_Allman_Body()
{
    let source = Source(Path("src/lib.rs"), Text("pub fn Add(a: i32, b: i32) -> i32\n{\n    return a + b;\n}\n"));

    let findings = Check_No_Single_Line_Function_Bodies(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_No_Single_Line_Function_Bodies_Should_Accept_A_Trait_Method_Declaration()
{
    let source = Source(Path("src/lib.rs"), Text("trait Shape\n{\n    fn Area(&self) -> f64;\n}\n"));

    let findings = Check_No_Single_Line_Function_Bodies(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_No_Single_Line_Function_Bodies_Should_Ignore_A_Collapsed_Closure_Inside_A_Multiline_Body()
{
    let source = Source(
        Path("src/lib.rs"),
        Text("pub fn Sum(values: &[i32]) -> i32\n{\n    return values.iter().fold(0, |acc, x| { acc + x });\n}\n"),
    );

    let findings = Check_No_Single_Line_Function_Bodies(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

/// `crates/rules/nomos-rules/src/checks/script_discipline.rs`'s own real shape: a byte
/// char literal `b'"'` whose content is a bare `"`, found only once this rule was
/// composed into a real run and left every line after it misread as inside an unclosed
/// string. A lifetime (`'a`, `'_`) is not a char literal and must still fall through.
#[test]
fn Test_Check_No_Single_Line_Function_Bodies_Should_Not_Misread_A_Quote_Char_Literal()
{
    let source = Source(
        Path("src/lib.rs"),
        Text("pub fn Is_Quote<'a>(character: u8) -> bool\n{\n    return character == b'\"';\n}\n\npub fn Real() {}\n"),
    );

    let findings = Check_No_Single_Line_Function_Bodies(&[source]);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "src/lib.rs:6");
}

/// `crates/substrate/nomos-workspace/src/workspace.rs`'s own real shape: a fake file
/// content string handed to a test fixture, not a real declaration.
#[test]
fn Test_Check_No_Single_Line_Function_Bodies_Should_Ignore_A_Same_Line_String_Fixture()
{
    let source = Source(Path("src/lib.rs"), Text("let changes = Present(\"a.rs\", \"fn a() {}\");\n"));

    let findings = Check_No_Single_Line_Function_Bodies(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

/// `tests/contract/src/reading/source_files.rs`'s own real shape: a raw string spanning
/// several physical lines whose content looks, line by line, like a real declaration.
#[test]
fn Test_Check_No_Single_Line_Function_Bodies_Should_Ignore_A_Multiline_Raw_String()
{
    let source = Source(Path("src/lib.rs"), Text("let source = r\"\npub fn Helper() {}\n\";\n"));

    let findings = Check_No_Single_Line_Function_Bodies(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

/// `crates/languages/nomos-lang-rust/src/syntax/tests.rs`'s own real shape: a plain
/// string literal continued onto the next physical line by a trailing `\`.
#[test]
fn Test_Check_No_Single_Line_Function_Bodies_Should_Ignore_A_Backslash_Continued_String()
{
    let source = Source(Path("src/lib.rs"), Text("let source = \"pub fn exported() {}\\n\\\n     fn hidden() {}\\n\";\n"));

    let findings = Check_No_Single_Line_Function_Bodies(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

/// This rule's own module doc, before this fix, was itself one of the false positives.
#[test]
fn Test_Check_No_Single_Line_Function_Bodies_Should_Ignore_A_Comment_Quoting_The_Shape()
{
    let source = Source(Path("src/lib.rs"), Text("/// A collapsed body looks like `fn f() { ... }` in prose.\n"));

    let findings = Check_No_Single_Line_Function_Bodies(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

/// `path` and `text` are both `&str`; without a distinct type per position a call site like
/// `Source("src/lib.rs", "fn Clean() {}")` reads as two interchangeable strings and a swap
/// compiles silently. These wrappers give each position a type the other cannot satisfy.
#[derive(Clone, Copy)]
struct Path<'a>(&'a str);

#[derive(Clone, Copy)]
struct Text<'a>(&'a str);

fn Source(path: Path<'_>, text: Text<'_>) -> SourceFile
{
    let mut source = SourceFile::New(path.0, SubjectId::From_Digest(Content_Digest(path.0.as_bytes())), text.0);
    source.language = crate::Recognized_Language_In_Tests(path.0);
    return source;
}
