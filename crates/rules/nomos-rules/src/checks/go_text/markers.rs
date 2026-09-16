//! `go_text`'s two bare-comment-directive rules -- a `//nolint` and a workspace opt-out marker
//! -- in their own file so the family's five rules no longer share one `Check_` prefix.

use crate::SourceFile;
use nomos_contracts::Finding;

/// Reports a bare `//nolint` or `//nolint:linter` with no trailing text after it.
#[must_use]
pub fn Check_Suppression_Directives_Carry_A_Reason(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();
    for source in sources
    {
        if source.Is_Written_In(crate::GO_LANGUAGE)
        {
            findings.extend(Nolint_Findings_In(source));
        }
    }
    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Nolint_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for (index, line) in source.text.lines().enumerate()
    {
        if Has_Nolint_Directive(line) && !Nolint_Has_Reason(line)
        {
            let finding = super::Finding_For_Line(
                source,
                super::SUPPRESSION_DIRECTIVES_CARRY_A_REASON,
                super::Line_Number(index),
                "carries `//nolint` with no trailing text explaining why",
            );
            findings.push(finding);
        }
    }

    return findings;
}

fn Has_Nolint_Directive(line: &str) -> bool
{
    return line.contains("//nolint");
}

fn Nolint_Has_Reason(line: &str) -> bool
{
    let Some(start) = line.find("//nolint")
    else
    {
        return false;
    };

    let mut rest = &line[start.saturating_add("//nolint".len())..];
    if let Some(after_colon) = rest.strip_prefix(':')
    {
        let end = after_colon
            .find(|character: char| return !(character.is_ascii_alphanumeric() || character == ',' || character == '_' || character == '-'))
            .unwrap_or(after_colon.len());
        rest = &after_colon[end..];
    }

    return !rest.trim().is_empty();
}

/// Reports a `// <marker>: allow[-<word>]` comment with no trailing reason.
#[must_use]
pub fn Check_Workspace_Markers_Carry_A_Reason(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();
    for source in sources
    {
        if source.Is_Written_In(crate::GO_LANGUAGE)
        {
            findings.extend(Workspace_Marker_Findings_In(source));
        }
    }
    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Workspace_Marker_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for (index, line) in source.text.lines().enumerate()
    {
        if let Some(finding) = Workspace_Marker_Finding_For(source, index, line)
        {
            findings.push(finding);
        }
    }

    return findings;
}

fn Workspace_Marker_Finding_For(source: &SourceFile, index: usize, line: &str) -> Option<Finding>
{
    let comment = super::Comment_Text_Of(line)?;
    if !Is_Bare_Workspace_Marker(comment)
    {
        return None;
    }

    let finding = super::Finding_For_Line(
        source,
        super::WORKSPACE_MARKERS_CARRY_A_REASON,
        super::Line_Number(index),
        "carries a workspace opt-out marker with no trailing reason",
    );
    return Some(finding);
}

/// `// <marker>: allow` or `// <marker>: allow-<word>`, matched only as real comment
/// syntax on the annotated line — a marker inside a string literal or elsewhere in the
/// file is not this function's concern, since it is only ever handed real comment text.
fn Is_Bare_Workspace_Marker(comment: &str) -> bool
{
    let Some((_marker, after_colon)) = comment.split_once(": allow")
    else
    {
        return false;
    };

    let after_suffix = after_colon.strip_prefix('-').map_or(after_colon, |rest| {
        return rest.find(char::is_whitespace).map_or("", |end| return &rest[end..]);
    });

    return after_suffix.trim().is_empty();
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::SubjectId;
    use nomos_model::Content_Digest;

    #[test]
    fn Test_Check_Suppression_Directives_Carry_A_Reason_Should_Report_A_Bare_Nolint()
    {
        let source = Source("main.go", "result, _ := risky() //nolint\n".to_owned());
        let findings = Check_Suppression_Directives_Carry_A_Reason(&[source]);
        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_Suppression_Directives_Carry_A_Reason_Should_Report_A_Bare_Linter_Named_Nolint()
    {
        let source = Source("main.go", "result, _ := risky() //nolint:errcheck\n".to_owned());
        let findings = Check_Suppression_Directives_Carry_A_Reason(&[source]);
        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_Suppression_Directives_Carry_A_Reason_Should_Accept_A_Nolint_With_A_Reason()
    {
        let source = Source("main.go", "result, _ := risky() //nolint:errcheck // the caller retries on failure\n".to_owned());
        let findings = Check_Suppression_Directives_Carry_A_Reason(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Workspace_Markers_Carry_A_Reason_Should_Report_A_Bare_Marker()
    {
        let source = Source("main.go", "// literals: allow\n".to_owned());
        let findings = Check_Workspace_Markers_Carry_A_Reason(&[source]);
        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_Workspace_Markers_Carry_A_Reason_Should_Report_A_Bare_Suffixed_Marker()
    {
        let source = Source("main.go", "// tool-tests: allow-untested\n".to_owned());
        let findings = Check_Workspace_Markers_Carry_A_Reason(&[source]);
        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_Workspace_Markers_Carry_A_Reason_Should_Accept_A_Marker_With_A_Reason()
    {
        let source = Source("main.go", "// literals: allow -- this magic number is the protocol version, not a policy violation\n".to_owned());
        let findings = Check_Workspace_Markers_Carry_A_Reason(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Workspace_Markers_Carry_A_Reason_Should_Ignore_A_String_Literal()
    {
        let source = Source("main.go", "message := \"// literals: allow\"\n".to_owned());
        let findings = Check_Workspace_Markers_Carry_A_Reason(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    fn Source(path: &str, text: String) -> SourceFile
    {
        let mut source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
        source.language = crate::Recognized_Language_In_Tests(path);
        return source;
    }
}
