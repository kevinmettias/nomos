//! Text-local script discipline rules from code-standards.
//!
//! The repository-wide standard has more to say about scripts, including executable bits,
//! sourced shell libraries and declared tooling languages. This crate is handed source
//! text, not file modes or repository configuration, so these checks judge the exact
//! subset visible from a file's first lines: shebang portability and the purpose comment
//! immediately after the shebang/blank-line prefix.

use crate::SourceFile;
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// The code-standards portable-shebang rule id.
pub const SCRIPTS_USE_A_PORTABLE_SHEBANG: &str = "scripts-use-a-portable-shebang";
/// The code-standards script-purpose rule id.
pub const A_SCRIPT_DECLARES_ITS_PURPOSE: &str = "a-script-declares-its-purpose";

/// Reports shebang scripts whose first line hardcodes an interpreter path.
#[must_use]
pub fn Check_Scripts_Use_A_Portable_Shebang(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        let Some(first_line) = First_Line(source)
        else
        {
            continue;
        };

        if first_line.starts_with("#!") && !first_line.starts_with("#!/usr/bin/env ")
        {
            findings.push(Finding_For_Source(
                source,
                SCRIPTS_USE_A_PORTABLE_SHEBANG,
                "has a hardcoded shebang; use `#!/usr/bin/env <interpreter>`",
            ));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// Reports shebang scripts whose first nonblank line after the shebang is not a comment.
#[must_use]
pub fn Check_A_Script_Declares_Its_Purpose(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if !Is_Shebang_Script(source)
        {
            continue;
        }

        if !Has_Purpose_Comment(source)
        {
            findings.push(Finding_For_Source(
                source,
                A_SCRIPT_DECLARES_ITS_PURPOSE,
                "does not declare its purpose in the first nonblank line after the shebang",
            ));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn First_Line(source: &SourceFile) -> Option<&str>
{
    return source.text.lines().next();
}

fn Is_Shebang_Script(source: &SourceFile) -> bool
{
    return First_Line(source).is_some_and(|line| return line.starts_with("#!"));
}

fn Has_Purpose_Comment(source: &SourceFile) -> bool
{
    return source
        .text
        .lines()
        .skip(1)
        .find(|line| return !line.trim().is_empty())
        .is_some_and(|line| return line.trim_start().starts_with('#'));
}

fn Finding_For_Source(source: &SourceFile, rule: &str, because: &str) -> Finding
{
    return Finding {
        rule: RuleId::New(rule),
        subject: source.subject,
        subject_name: source.path.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!("{} {because}", source.path),
        locations: vec![source.path.clone()],
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::SubjectId;
    use nomos_model::Content_Digest;

    #[test]
    fn Test_Check_Scripts_Use_A_Portable_Shebang_Should_Report_A_Hardcoded_Shebang()
    {
        let source = Source("scripts/check.sh", "#!/bin/bash\n# check -- run checks\n");

        let findings = Check_Scripts_Use_A_Portable_Shebang(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(SCRIPTS_USE_A_PORTABLE_SHEBANG));
    }

    #[test]
    fn Test_Check_Scripts_Use_A_Portable_Shebang_Should_Accept_Env_Shebang()
    {
        let source = Source("scripts/check.sh", "#!/usr/bin/env bash\n# check -- run checks\n");

        let findings = Check_Scripts_Use_A_Portable_Shebang(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Script_Declares_Its_Purpose_Should_Report_Code_As_The_First_Content()
    {
        let source = Source("scripts/check.sh", "#!/usr/bin/env bash\n\nset -u\n");

        let findings = Check_A_Script_Declares_Its_Purpose(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(A_SCRIPT_DECLARES_ITS_PURPOSE));
    }

    #[test]
    fn Test_Check_A_Script_Declares_Its_Purpose_Should_Accept_A_Comment_After_Blanks()
    {
        let source = Source("scripts/check.sh", "#!/usr/bin/env bash\n\n# check -- run checks\nset -u\n");

        let findings = Check_A_Script_Declares_Its_Purpose(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Script_Declares_Its_Purpose_Should_Ignore_Non_Shebang_Files()
    {
        let source = Source("src/lib.rs", "fn Check() {}\n");

        let findings = Check_A_Script_Declares_Its_Purpose(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    fn Source(path: &str, text: &str) -> SourceFile
    {
        return SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
    }
}
