//! Go source-text rules from code-standards' suppression-justified family.
//!
//! The Go form of the same "a marker needs an adjacent or attached reason" shape
//! `rust_text.rs` already generalizes for Rust (`every-allow-carries-a-justification`,
//! `unsafe-justification`, `panics-are-justified-documented-and-validated`, `shared-
//! interior-mutability-says-why`): five constructs, three needing an adjacent comment
//! block, one needing a same-line trailing reason after a directive, and one needing
//! either a call argument or (when the call cannot carry one) an adjacent comment.
//! Deliberately not a capability: nothing about whether these markers need a reason is a
//! value a repository would configure -- it is always true, so the generalization is
//! shared code, not shared configuration, the same distinction `OD-RULES-011`'s own
//! naming/limits capabilities do not apply here.

use super::code_prefix::Code_Prefix;
use crate::{GO_LANGUAGE, SourceFile};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// The code-standards discarded-error rule id.
pub const A_DISCARDED_ERROR_IS_EXPLAINED: &str = "a-discarded-error-is-explained";
/// The code-standards skipped-test rule id.
pub const A_SKIPPED_TEST_STATES_WHY: &str = "a-skipped-test-states-why";
/// The code-standards excluded-file rule id.
pub const AN_EXCLUDED_FILE_SAYS_WHY: &str = "an-excluded-file-says-why";
/// The code-standards `//nolint` rule id.
pub const SUPPRESSION_DIRECTIVES_CARRY_A_REASON: &str = "suppression-directives-carry-a-reason";
/// The code-standards workspace-marker rule id.
pub const WORKSPACE_MARKERS_CARRY_A_REASON: &str = "workspace-markers-carry-a-reason";

/// Reports `_ = someCall()` with no adjacent comment explaining why the error cannot
/// matter -- the exact syntactic form the standard's own doc names, preferring a false
/// negative over a false positive since `go/ast` cannot resolve types here.
#[must_use]
pub fn Check_A_Discarded_Error_Is_Explained(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();
    for source in sources
    {
        if source.Is_Written_In(GO_LANGUAGE)
        {
            findings.extend(Discarded_Error_Findings_In(source));
        }
    }
    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Discarded_Error_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let lines = Lines_Of(source);
    let mut findings = Vec::new();

    for (index, line) in lines.iter().enumerate()
    {
        let code = Code_Prefix(line);
        if Is_Discarded_Call(&code) && !Has_Adjacent_Explanation(&lines, index)
        {
            let finding = Finding_For_Line(
                source,
                A_DISCARDED_ERROR_IS_EXPLAINED,
                Line_Number(index),
                "discards an error with `_ =` and no comment saying why it cannot matter",
            );
            findings.push(finding);
        }
    }

    return findings;
}

/// `_ = <call>(` — the exact syntactic form the standard names; not a bare `_ = value`
/// with no call, which this rule does not judge.
fn Is_Discarded_Call(code: &str) -> bool
{
    let trimmed = code.trim_start();
    let Some(rest) = trimmed.strip_prefix("_ = ")
    else
    {
        return false;
    };
    return rest.contains('(');
}

/// Reports `t.Skip()`/`t.Skipf()` with an empty argument list, and `t.SkipNow()` (which
/// cannot carry an argument at all) with no adjacent comment.
#[must_use]
pub fn Check_A_Skipped_Test_States_Why(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();
    for source in sources
    {
        if source.Is_Written_In(GO_LANGUAGE)
        {
            findings.extend(Skip_Findings_In(source));
        }
    }
    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Skip_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let lines = Lines_Of(source);
    let mut findings = Vec::new();

    for (index, line) in lines.iter().enumerate()
    {
        if let Some(finding) = Skip_Finding_For(source, &lines, index, line)
        {
            findings.push(finding);
        }
    }

    return findings;
}

fn Skip_Finding_For(source: &SourceFile, lines: &[&str], index: usize, line: &str) -> Option<Finding>
{
    let code = Code_Prefix(line);

    if Has_Empty_Skip_Call(CodeText(&code), CallName("t.Skip(")) || Has_Empty_Skip_Call(CodeText(&code), CallName("t.Skipf("))
    {
        let finding = Finding_For_Line(source, A_SKIPPED_TEST_STATES_WHY, Line_Number(index), "calls t.Skip/t.Skipf with no explanatory message");
        return Some(finding);
    }

    if code.contains("t.SkipNow()") && !Has_Adjacent_Explanation(lines, index)
    {
        let finding = Finding_For_Line(
            source,
            A_SKIPPED_TEST_STATES_WHY,
            Line_Number(index),
            "calls t.SkipNow(), which takes no message, with no adjacent comment saying why",
        );
        return Some(finding);
    }

    return None;
}

/// `code` and `call` are both `&str`; without a distinct type per position, a call site
/// like `Has_Empty_Skip_Call(code, call)` reads as two interchangeable strings and a swap
/// compiles silently.
struct CodeText<'a>(&'a str);
struct CallName<'a>(&'a str);

fn Has_Empty_Skip_Call(code: CodeText<'_>, call: CallName<'_>) -> bool
{
    let code = code.0;
    let call = call.0;
    let Some(start) = code.find(call)
    else
    {
        return false;
    };
    let after = &code[start.saturating_add(call.len())..];
    let close = after.find(')').unwrap_or(after.len());
    return after[..close].trim().is_empty();
}

/// Reports `//go:build ignore` with no adjacent comment explaining the exclusion.
#[must_use]
pub fn Check_An_Excluded_File_Says_Why(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();
    for source in sources
    {
        if source.Is_Written_In(GO_LANGUAGE)
        {
            findings.extend(Build_Ignore_Findings_In(source));
        }
    }
    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Build_Ignore_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let lines = Lines_Of(source);
    let mut findings = Vec::new();

    for (index, line) in lines.iter().enumerate()
    {
        if line.trim() == "//go:build ignore" && !Has_Build_Ignore_Explanation(&lines, index)
        {
            let finding = Finding_For_Line(
                source,
                AN_EXCLUDED_FILE_SAYS_WHY,
                Line_Number(index),
                "excludes the file with `//go:build ignore` and no adjacent comment explaining why",
            );
            findings.push(finding);
        }
    }

    return findings;
}

fn Has_Build_Ignore_Explanation(lines: &[&str], index: usize) -> bool
{
    let Is_Non_Empty_Comment = |maybe_index: Option<usize>| -> bool {
        let Some(candidate) = maybe_index
        else
        {
            return false;
        };
        return lines.get(candidate).is_some_and(|line| return Comment_Text_Of(line).is_some_and(|c| return !c.trim().is_empty()));
    };

    return Is_Non_Empty_Comment(index.checked_sub(1)) || Is_Non_Empty_Comment(index.checked_add(1));
}

/// Reports a bare `//nolint` or `//nolint:linter` with no trailing text after it.
#[must_use]
pub fn Check_Suppression_Directives_Carry_A_Reason(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();
    for source in sources
    {
        if source.Is_Written_In(GO_LANGUAGE)
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
            let finding = Finding_For_Line(
                source,
                SUPPRESSION_DIRECTIVES_CARRY_A_REASON,
                Line_Number(index),
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
        if source.Is_Written_In(GO_LANGUAGE)
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
    let comment = Comment_Text_Of(line)?;
    if !Is_Bare_Workspace_Marker(comment)
    {
        return None;
    }

    let finding = Finding_For_Line(
        source,
        WORKSPACE_MARKERS_CARRY_A_REASON,
        Line_Number(index),
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

fn Lines_Of(source: &SourceFile) -> Vec<&str>
{
    return source.text.lines().collect();
}

fn Line_Number(index: usize) -> usize
{
    return index.saturating_add(1);
}

fn Comment_Text_Of(line: &str) -> Option<&str>
{
    let trimmed = line.trim_start();
    let comment = trimmed.strip_prefix("//")?;
    return Some(comment.trim_start());
}

/// A trailing same-line comment with non-empty text, or a non-empty comment on the line
/// immediately above — the two shapes `a-discarded-error-is-explained` and `a-skipped-
/// test-states-why` (for `SkipNow`) both accept.
fn Has_Adjacent_Explanation(lines: &[&str], index: usize) -> bool
{
    if let Some(reason) = lines.get(index).and_then(|line| return line.split_once("//").map(|(_, rest)| return rest))
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

fn Finding_For_Line(source: &SourceFile, rule: &str, line_number: usize, because: &str) -> Finding
{
    return Finding {
        rule: RuleId::New(rule),
        subject: source.subject,
        subject_name: format!("{}:{line_number}", source.path),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!("{} line {line_number} {because}", source.path),
        locations: vec![format!("{}:{line_number}", source.path)],
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::SubjectId;
    use nomos_model::Content_Digest;

    #[test]
    fn Test_Check_A_Discarded_Error_Is_Explained_Should_Report_An_Unexplained_Discard()
    {
        let source = Source("main.go", "_ = os.Remove(path)\n");
        let findings = Check_A_Discarded_Error_Is_Explained(&[source]);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(A_DISCARDED_ERROR_IS_EXPLAINED));
    }

    #[test]
    fn Test_Check_A_Discarded_Error_Is_Explained_Should_Accept_A_Same_Line_Comment()
    {
        let source = Source("main.go", "_ = os.Remove(path) // best-effort cleanup, nothing to do if it fails\n");
        let findings = Check_A_Discarded_Error_Is_Explained(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Discarded_Error_Is_Explained_Should_Accept_A_Comment_On_The_Line_Above()
    {
        let source = Source(
            "main.go",
            "// best-effort cleanup, nothing to do if it fails\n_ = os.Remove(path)\n",
        );
        let findings = Check_A_Discarded_Error_Is_Explained(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Discarded_Error_Is_Explained_Should_Ignore_A_Non_Call_Discard()
    {
        let source = Source("main.go", "_ = value\n");
        let findings = Check_A_Discarded_Error_Is_Explained(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Skipped_Test_States_Why_Should_Report_An_Empty_Skip()
    {
        let source = Source("main_test.go", "t.Skip()\n");
        let findings = Check_A_Skipped_Test_States_Why(&[source]);
        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Skipped_Test_States_Why_Should_Accept_A_Skip_With_A_Message()
    {
        let source = Source("main_test.go", "t.Skip(\"flaky on CI, see #123\")\n");
        let findings = Check_A_Skipped_Test_States_Why(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Skipped_Test_States_Why_Should_Report_An_Unexplained_Skip_Now()
    {
        let source = Source("main_test.go", "t.SkipNow()\n");
        let findings = Check_A_Skipped_Test_States_Why(&[source]);
        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Skipped_Test_States_Why_Should_Accept_A_Skip_Now_With_An_Adjacent_Comment()
    {
        let source = Source("main_test.go", "// this platform has no filesystem to test against\nt.SkipNow()\n");
        let findings = Check_A_Skipped_Test_States_Why(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_An_Excluded_File_Says_Why_Should_Report_An_Unexplained_Build_Ignore()
    {
        let source = Source("scratch.go", "//go:build ignore\n\npackage main\n");
        let findings = Check_An_Excluded_File_Says_Why(&[source]);
        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_An_Excluded_File_Says_Why_Should_Accept_A_Following_Explanation()
    {
        let source = Source(
            "scratch.go",
            "//go:build ignore\n// this file is a manual repro script, not part of the build\n\npackage main\n",
        );
        let findings = Check_An_Excluded_File_Says_Why(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Suppression_Directives_Carry_A_Reason_Should_Report_A_Bare_Nolint()
    {
        let source = Source("main.go", "result, _ := risky() //nolint\n");
        let findings = Check_Suppression_Directives_Carry_A_Reason(&[source]);
        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_Suppression_Directives_Carry_A_Reason_Should_Report_A_Bare_Linter_Named_Nolint()
    {
        let source = Source("main.go", "result, _ := risky() //nolint:errcheck\n");
        let findings = Check_Suppression_Directives_Carry_A_Reason(&[source]);
        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_Suppression_Directives_Carry_A_Reason_Should_Accept_A_Nolint_With_A_Reason()
    {
        let source = Source("main.go", "result, _ := risky() //nolint:errcheck // the caller retries on failure\n");
        let findings = Check_Suppression_Directives_Carry_A_Reason(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Workspace_Markers_Carry_A_Reason_Should_Report_A_Bare_Marker()
    {
        let source = Source("main.go", "// literals: allow\n");
        let findings = Check_Workspace_Markers_Carry_A_Reason(&[source]);
        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_Workspace_Markers_Carry_A_Reason_Should_Report_A_Bare_Suffixed_Marker()
    {
        let source = Source("main.go", "// tool-tests: allow-untested\n");
        let findings = Check_Workspace_Markers_Carry_A_Reason(&[source]);
        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_Workspace_Markers_Carry_A_Reason_Should_Accept_A_Marker_With_A_Reason()
    {
        let source = Source("main.go", "// literals: allow -- this magic number is the protocol version, not a policy violation\n");
        let findings = Check_Workspace_Markers_Carry_A_Reason(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Workspace_Markers_Carry_A_Reason_Should_Ignore_A_String_Literal()
    {
        let source = Source("main.go", "message := \"// literals: allow\"\n");
        let findings = Check_Workspace_Markers_Carry_A_Reason(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    fn Source(path: &str, text: &str) -> SourceFile
    {
        let mut source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
        source.language = crate::Recognized_Language_In_Tests(path);
        return source;
    }
}
