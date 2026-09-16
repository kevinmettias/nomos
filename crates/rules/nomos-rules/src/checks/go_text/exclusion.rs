//! `go_text`'s `//go:build ignore` rule, in its own file so the family's five rules no longer
//! share one `Check_` prefix.

use crate::SourceFile;
use nomos_contracts::Finding;

/// Reports `//go:build ignore` with no adjacent comment explaining the exclusion.
#[must_use]
pub fn Check_An_Excluded_File_Says_Why(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();
    for source in sources
    {
        if source.Is_Written_In(crate::GO_LANGUAGE)
        {
            findings.extend(Build_Ignore_Findings_In(source));
        }
    }
    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Build_Ignore_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let lines = super::Lines_Of(source);
    let mut findings = Vec::new();

    for (index, line) in lines.iter().enumerate()
    {
        if line.trim() == "//go:build ignore" && !Has_Build_Ignore_Explanation(&lines, index)
        {
            let finding = super::Finding_For_Line(
                source,
                super::AN_EXCLUDED_FILE_SAYS_WHY,
                super::Line_Number(index),
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
        return lines.get(candidate).is_some_and(|line| return super::Comment_Text_Of(line).is_some_and(|c| return !c.trim().is_empty()));
    };

    return Is_Non_Empty_Comment(index.checked_sub(1)) || Is_Non_Empty_Comment(index.checked_add(1));
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::SubjectId;
    use nomos_model::Content_Digest;

    #[test]
    fn Test_Check_An_Excluded_File_Says_Why_Should_Report_An_Unexplained_Build_Ignore()
    {
        let source = Source("scratch.go", "//go:build ignore\n\npackage main\n".to_owned());
        let findings = Check_An_Excluded_File_Says_Why(&[source]);
        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_An_Excluded_File_Says_Why_Should_Accept_A_Following_Explanation()
    {
        let source = Source(
            "scratch.go",
            "//go:build ignore\n// this file is a manual repro script, not part of the build\n\npackage main\n".to_owned(),
        );
        let findings = Check_An_Excluded_File_Says_Why(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    fn Source(path: &str, text: String) -> SourceFile
    {
        let mut source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
        source.language = crate::Recognized_Language_In_Tests(path);
        return source;
    }
}
