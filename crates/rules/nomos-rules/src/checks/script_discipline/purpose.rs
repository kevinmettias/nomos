//! `script_discipline.rs`'s purpose-comment rule: the first nonblank line after a shebang must
//! say what the script is for before anything runs.

use super::{A_SCRIPT_DECLARES_ITS_PURPOSE, Because, Finding_For_Source, Is_Shebang_Script, Rule};
use crate::SourceFile;
use nomos_contracts::Finding;

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
            let finding = Finding_For_Source(
                source,
                Rule(A_SCRIPT_DECLARES_ITS_PURPOSE),
                Because("does not declare its purpose in the first nonblank line after the shebang"),
            );
            findings.push(finding);
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
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

#[cfg(test)]
mod tests
{
    use super::super::tests::{Source, SourceText};
    use super::*;
    use nomos_contracts::RuleId;

    /// The same inner attribute under the other rule: neither may read it as a script,
    /// because a Rust file has nowhere to put the purpose comment this one asks for.
    #[test]
    fn Test_Check_A_Script_Declares_Its_Purpose_Should_Ignore_A_Rust_Inner_Attribute()
    {
        let source = Source(SourceText { path: "src/lib.rs", text: "#![forbid(unsafe_code)]\n\npub fn Check() {}\n" });

        let findings = Check_A_Script_Declares_Its_Purpose(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Script_Declares_Its_Purpose_Should_Report_Code_As_The_First_Content()
    {
        let source = Source(SourceText { path: "scripts/check.sh", text: "#!/usr/bin/env bash\n\nset -u\n" });

        let findings = Check_A_Script_Declares_Its_Purpose(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(A_SCRIPT_DECLARES_ITS_PURPOSE));
    }

    #[test]
    fn Test_Check_A_Script_Declares_Its_Purpose_Should_Accept_A_Comment_After_Blanks()
    {
        let source = Source(SourceText { path: "scripts/check.sh", text: "#!/usr/bin/env bash\n\n# check -- run checks\nset -u\n" });

        let findings = Check_A_Script_Declares_Its_Purpose(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Script_Declares_Its_Purpose_Should_Ignore_Non_Shebang_Files()
    {
        let source = Source(SourceText { path: "src/lib.rs", text: "fn Check() {}\n" });

        let findings = Check_A_Script_Declares_Its_Purpose(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }
}
