use super::*;
use nomos_contracts::SubjectId;
use nomos_model::Content_Digest;

#[test]
fn Test_Check_Named_Fields_Over_Positional_Variant_Payloads_Should_Report_A_Two_Member_Tuple_Variant()
{
    let source = Source(SourceText { path: "src/shape.rs", text: "enum Shape {\n Point(f32, f32),\n}\n" });
    let findings = Check_Named_Fields_Over_Positional_Variant_Payloads(&[source]);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(NAMED_FIELDS_OVER_POSITIONAL_VARIANT_PAYLOADS));
}

#[test]
fn Test_Check_Named_Fields_Over_Positional_Variant_Payloads_Should_Ignore_A_Single_Member_Newtype_Variant()
{
    let source = Source(SourceText { path: "src/shape.rs", text: "enum Maybe {\n Some(u32),\n}\n" });
    let findings = Check_Named_Fields_Over_Positional_Variant_Payloads(&[source]);

    assert!(findings.is_empty(), "a single member has no ordering to get wrong: {findings:?}");
}

#[test]
fn Test_Check_Named_Fields_Over_Positional_Variant_Payloads_Should_Ignore_A_Struct_Form_Variant()
{
    let source = Source(SourceText { path: "src/shape.rs", text: "enum Shape {\n Point { x: f32, y: f32 },\n}\n" });
    let findings = Check_Named_Fields_Over_Positional_Variant_Payloads(&[source]);

    assert!(findings.is_empty(), "the fields are already named: {findings:?}");
}

/// K&R style: the opening brace shares the header's own line, the body on separate
/// lines below it — distinct from the Allman case right below, where the brace itself
/// is on its own, later line.
#[test]
fn Test_Check_Named_Fields_Over_Positional_Variant_Payloads_Should_Report_Under_A_K_And_R_Brace_Enum()
{
    let source = Source(SourceText { path: "src/shape.rs", text: "enum Shape {\n Point(f32, f32),\n}\n" });
    let findings = Check_Named_Fields_Over_Positional_Variant_Payloads(&[source]);

    assert_eq!(findings.len(), 1, "{findings:?}");
}

/// The corpus lesson itself: `rustVariantScan`'s own doc comment records a first cut
/// that assumed the brace sits on the header's own line, and silently found zero
/// across a real Allman-brace corpus.
#[test]
fn Test_Check_Named_Fields_Over_Positional_Variant_Payloads_Should_Report_Under_An_Allman_Brace_Enum()
{
    let source = Source(SourceText { path: "src/shape.rs", text: "enum Shape\n{\n Point(f32, f32),\n}\n" });
    let findings = Check_Named_Fields_Over_Positional_Variant_Payloads(&[source]);

    assert_eq!(findings.len(), 1, "an Allman-style opening brace must still open the enum: {findings:?}");
}

#[test]
fn Test_Check_Named_Fields_Over_Positional_Variant_Payloads_Should_Split_A_Generic_Member_As_One()
{
    let source = Source(SourceText { path: "src/shape.rs", text: "enum Cache {\n Load(HashMap<K, V>, u32),\n}\n" });
    let findings = Check_Named_Fields_Over_Positional_Variant_Payloads(&[source]);

    assert_eq!(findings.len(), 1, "a two-parameter generic plus one more member is a pair, not a phantom triple: {findings:?}");
}

#[test]
fn Test_Check_Named_Fields_Over_Positional_Variant_Payloads_Should_Name_A_Repeated_Type()
{
    let source = Source(SourceText { path: "src/shape.rs", text: "enum Shape {\n Point(f32, f32),\n}\n" });
    let findings = Check_Named_Fields_Over_Positional_Variant_Payloads(&[source]);

    let summary = &findings.first().expect("asserted by an earlier test").summary;
    assert!(summary.contains("repeated type"), "{summary}");
}

#[test]
fn Test_Check_Named_Fields_Over_Positional_Variant_Payloads_Should_Not_Name_A_Repeated_Type_For_Distinct_Members()
{
    let source = Source(SourceText { path: "src/shape.rs", text: "enum Error {\n Bad(u32, String),\n}\n" });
    let findings = Check_Named_Fields_Over_Positional_Variant_Payloads(&[source]);

    let summary = &findings.first().expect("distinct-typed members still fire").summary;
    assert!(!summary.contains("repeated type"), "{summary}");
}

#[test]
fn Test_Check_Named_Fields_Over_Positional_Variant_Payloads_Should_Accept_A_Marker_Reason_Above()
{
    let source = Source(
        SourceText { path: "src/shape.rs", text: "enum Shape {\n // tuple-variant: allow: kept positional for a stable FFI layout\n Point(f32, f32),\n}\n" },
    );
    let findings = Check_Named_Fields_Over_Positional_Variant_Payloads(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Enum_Header_Name_Should_Require_A_Word_Boundary()
{
    assert_eq!(Enum_Header_Name("enum Shape"), Some("Shape"));
    assert_eq!(Enum_Header_Name("frozenum Shape"), None, "enum must be a whole word");
}

#[test]
fn Test_Tuple_Variant_Match_Should_Reject_A_Payload_With_A_Nested_Unbalanced_Paren()
{
    assert_eq!(Tuple_Variant_Match("Bad((u32, String))"), None);
}

#[test]
fn Test_Variant_Members_Should_Return_Nothing_For_An_Empty_Payload()
{
    assert_eq!(Variant_Members(""), Vec::<String>::new());
}

    /// A fixture source's two halves, grouped so a call site names which string is the path
/// and which is the text, rather than counting two adjacent `&str` positions a caller
/// could transpose without the compiler objecting.
struct SourceText<'text>
{
    path: &'text str,
    text: &'text str,
}

fn Source(source: SourceText<'_>) -> SourceFile
{
    let SourceText { path, text } = source;
    let mut source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
    source.language = crate::Recognized_Language_In_Tests(path);
    return source;
}
