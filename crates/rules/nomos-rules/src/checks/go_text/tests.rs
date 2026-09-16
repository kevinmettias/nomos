use super::*;
use nomos_contracts::SubjectId;
use nomos_model::Content_Digest;

#[test]
fn Test_Check_A_Discarded_Error_Is_Explained_Should_Report_An_Unexplained_Discard()
{
    let source = Source("main.go", "_ = os.Remove(path)\n".to_owned());
    let findings = Check_A_Discarded_Error_Is_Explained(&[source]);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(A_DISCARDED_ERROR_IS_EXPLAINED));
}

#[test]
fn Test_Check_A_Discarded_Error_Is_Explained_Should_Accept_A_Same_Line_Comment()
{
    let source = Source("main.go", "_ = os.Remove(path) // best-effort cleanup, nothing to do if it fails\n".to_owned());
    let findings = Check_A_Discarded_Error_Is_Explained(&[source]);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_A_Discarded_Error_Is_Explained_Should_Accept_A_Comment_On_The_Line_Above()
{
    let source = Source(
        "main.go",
        "// best-effort cleanup, nothing to do if it fails\n_ = os.Remove(path)\n".to_owned(),
    );
    let findings = Check_A_Discarded_Error_Is_Explained(&[source]);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_A_Discarded_Error_Is_Explained_Should_Ignore_A_Non_Call_Discard()
{
    let source = Source("main.go", "_ = value\n".to_owned());
    let findings = Check_A_Discarded_Error_Is_Explained(&[source]);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_A_Skipped_Test_States_Why_Should_Report_An_Empty_Skip()
{
    let source = Source("main_test.go", "t.Skip()\n".to_owned());
    let findings = Check_A_Skipped_Test_States_Why(&[source]);
    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn Test_Check_A_Skipped_Test_States_Why_Should_Accept_A_Skip_With_A_Message()
{
    let source = Source("main_test.go", "t.Skip(\"flaky on CI, see #123\")\n".to_owned());
    let findings = Check_A_Skipped_Test_States_Why(&[source]);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_A_Skipped_Test_States_Why_Should_Report_An_Unexplained_Skip_Now()
{
    let source = Source("main_test.go", "t.SkipNow()\n".to_owned());
    let findings = Check_A_Skipped_Test_States_Why(&[source]);
    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn Test_Check_A_Skipped_Test_States_Why_Should_Accept_A_Skip_Now_With_An_Adjacent_Comment()
{
    let source = Source("main_test.go", "// this platform has no filesystem to test against\nt.SkipNow()\n".to_owned());
    let findings = Check_A_Skipped_Test_States_Why(&[source]);
    assert!(findings.is_empty(), "{findings:?}");
}

fn Source(path: &str, text: String) -> SourceFile
{
    let mut source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
    source.language = crate::Recognized_Language_In_Tests(path);
    return source;
}
