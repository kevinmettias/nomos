use super::*;
use nomos_contracts::SubjectId;
use nomos_model::Content_Digest;

#[test]
fn Test_Check_Constants_Are_The_Exception_To_Function_Scope_Use_Should_Report_A_Rust_Const_Inside_A_Function()
{
    let source = Source("src/timer.rs", "fn tick()\n{\n    const FRAME_INTERVAL_MS: u32 = 16;\n}\n".to_owned());
    let findings = Check_Constants_Are_The_Exception_To_Function_Scope_Use(&[source]);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(CONSTANTS_ARE_THE_EXCEPTION_TO_FUNCTION_SCOPE_USE));
}

#[test]
fn Test_Check_Constants_Are_The_Exception_To_Function_Scope_Use_Should_Ignore_A_Module_Scope_Const()
{
    let source = Source("src/timer.rs", "const FRAME_INTERVAL_MS: u32 = 16;\n\nfn tick()\n{\n}\n".to_owned());
    let findings = Check_Constants_Are_The_Exception_To_Function_Scope_Use(&[source]);

    assert!(findings.is_empty(), "module scope is where constants live: {findings:?}");
}

#[test]
fn Test_Check_Constants_Are_The_Exception_To_Function_Scope_Use_Should_Ignore_An_Associated_Const()
{
    let source = Source("src/timer.rs", "impl Timer\n{\n    const FRAME_INTERVAL_MS: u32 = 16;\n}\n".to_owned());
    let findings = Check_Constants_Are_The_Exception_To_Function_Scope_Use(&[source]);

    assert!(findings.is_empty(), "an associated const belongs to the type, not any function: {findings:?}");
}

#[test]
fn Test_Check_Constants_Are_The_Exception_To_Function_Scope_Use_Should_Not_Mistake_A_Const_Fn_For_A_Value()
{
    let source = Source("src/timer.rs", "fn outer()\n{\n    const fn helper() -> u32 { 16 }\n}\n".to_owned());
    let findings = Check_Constants_Are_The_Exception_To_Function_Scope_Use(&[source]);

    assert!(findings.is_empty(), "const fn is a function modifier, not a value declaration: {findings:?}");
}

#[test]
fn Test_Check_Constants_Are_The_Exception_To_Function_Scope_Use_Should_Report_A_Doubly_Nested_Const_Only_Once()
{
    let source = Source(
        "src/timer.rs",
        "fn outer()\n{\n    fn inner()\n    {\n        const FRAME_INTERVAL_MS: u32 = 16;\n    }\n}\n".to_owned(),
    );
    let findings = Check_Constants_Are_The_Exception_To_Function_Scope_Use(&[source]);

    assert_eq!(findings.len(), 1, "attributed to the innermost function only, not once per nesting level: {findings:?}");
}

#[test]
fn Test_Check_Constants_Are_The_Exception_To_Function_Scope_Use_Should_Report_A_Go_Const_Inside_A_Function()
{
    let source = Source("timer.go", "func Tick() {\n\tconst frameIntervalMs = 16\n}\n".to_owned());
    let findings = Check_Constants_Are_The_Exception_To_Function_Scope_Use(&[source]);

    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn Test_Check_Constants_Are_The_Exception_To_Function_Scope_Use_Should_Ignore_A_Go_Package_Scope_Const()
{
    let source = Source("timer.go", "const frameIntervalMs = 16\n\nfunc Tick() {\n}\n".to_owned());
    let findings = Check_Constants_Are_The_Exception_To_Function_Scope_Use(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Constants_Are_The_Exception_To_Function_Scope_Use_Should_Report_A_Go_Const_Block_Inside_A_Function()
{
    const EXPECTED_CONSTANTS: usize = 2;
    let source = Source("timer.go", "func Tick() {\n\tconst (\n\t\tframeIntervalMs = 16\n\t\tmaxFrames = 60\n\t)\n}\n".to_owned());
    let findings = Check_Constants_Are_The_Exception_To_Function_Scope_Use(&[source]);

    assert_eq!(findings.len(), EXPECTED_CONSTANTS, "{findings:?}");
}

fn Source(path: &str, text: String) -> SourceFile
{
    let mut source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
    source.language = crate::Recognized_Language_In_Tests(path);
    return source;
}
