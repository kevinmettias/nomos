//! One of `security_text.rs`'s three rules -- a TLS/SSL client or server told, by a literal
//! value, to skip verifying the peer's certificate -- split out with its whole closure, the same
//! boundary `dependency.rs` and `naming.rs` draw for their own entry points, because three
//! `Check_`-prefixed names sharing one file hid a module the judgments had already named.

use super::{CERTIFICATE_VERIFICATION_IS_NOT_DISABLED, Finding_For_Line, Is_Own_Implementation_File, Is_Test_Or_Fixture_Source, Line_Number};
use crate::SourceFile;
use nomos_contracts::Finding;

/// Reports a TLS/SSL client or server told, by a literal value, to skip verifying the
/// peer's certificate: `InsecureSkipVerify` set to the literal `true`, `.danger_accept_
/// invalid_certs(true)`, `SslVerifyMode::NONE`, or `verify` set to the literal `False`.
/// Does not flag a value set from a variable or config flag — the standard is explicit
/// that whether that path disables verification is not a fact the source states — and
/// accepts either an adjacent explanatory comment (a reasoned, recorded exception) or a
/// test/fixture source, both of which the standard names as exempt.
#[must_use]
pub fn Check_Certificate_Verification_Is_Not_Disabled(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();
    for source in sources
    {
        if !Is_Test_Or_Fixture_Source(source) && !Is_Own_Implementation_File(source)
        {
            findings.extend(Tls_Verification_Findings_In(source));
        }
    }
    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Tls_Verification_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let lines: Vec<&str> = source.text.lines().collect();
    let mut findings = Vec::new();

    for (index, line) in lines.iter().enumerate()
    {
        if Has_Disabled_Certificate_Verification(line) && !Has_Adjacent_Explanation(&lines, index)
        {
            let finding = Finding_For_Line(
                source,
                CERTIFICATE_VERIFICATION_IS_NOT_DISABLED,
                Line_Number(index),
                "disables TLS peer-certificate verification with no adjacent comment recording why",
            );
            findings.push(finding);
        }
    }

    return findings;
}

fn Has_Disabled_Certificate_Verification(line: &str) -> bool
{
    let compact: String = line.chars().filter(|character| return !character.is_whitespace()).collect();

    return compact.contains("InsecureSkipVerify:true")
        || compact.contains("InsecureSkipVerify=true")
        || line.contains(".danger_accept_invalid_certs(true)")
        || line.contains("SslVerifyMode::NONE")
        || compact.contains("verify=False")
        || compact.contains("verify:False");
}

/// A trailing same-line comment with non-empty text, or a non-empty comment on the line
/// immediately above — "the reasoned exception is recorded rather than hidden," the same
/// adjacent-explanation shape `rust_text.rs`/`go_text.rs` already use, generalized to a
/// third consumer.
fn Has_Adjacent_Explanation(lines: &[&str], index: usize) -> bool
{
    if let Some(reason) = lines
        .get(index)
        .and_then(|line| return line.split_once("//").or_else(|| return line.split_once('#')).map(|(_, rest)| return rest))
    {
        if !reason.trim().is_empty()
        {
            return true;
        }
    }

    if let Some(previous) = index.checked_sub(1)
    {
        if lines.get(previous).is_some_and(|line| return Comment_Text_Of(line).is_some_and(|c| return !c.trim().is_empty()))
        {
            return true;
        }
    }

    return false;
}

fn Comment_Text_Of(line: &str) -> Option<&str>
{
    let trimmed = line.trim_start();
    for marker in ["//", "#"]
    {
        if let Some(comment) = trimmed.strip_prefix(marker)
        {
            return Some(comment.trim_start());
        }
    }
    return None;
}

/// Narrow, file-local proof for this file's own public function, addressed by name.
///
/// `security_text/tests.rs` is a separate physical file whose behavioural suite this does not
/// repeat or replace. `check-test-coverage`'s Rust front end keys a test's companion unit off
/// the literal file it is textually written in, so a test living in that separate file can
/// never address a function declared here, however it is named — this module gives
/// [`Check_Certificate_Verification_Is_Not_Disabled`] the one-file address the check reads.
#[cfg(test)]
mod self_tests
{
    use super::*;
    use nomos_contracts::{RuleId, SubjectId};
    use nomos_model::Content_Digest;

    #[test]
    fn Test_Check_Certificate_Verification_Is_Not_Disabled_Should_Report_A_Literal_Disabled_Verification()
    {
        let path = "src/client.rs";
        let text = "let client = Client::builder().danger_accept_invalid_certs(true).build();\n";
        let mut source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
        source.language = crate::Recognized_Language_In_Tests(path);

        let findings = Check_Certificate_Verification_Is_Not_Disabled(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(CERTIFICATE_VERIFICATION_IS_NOT_DISABLED));
    }
}
