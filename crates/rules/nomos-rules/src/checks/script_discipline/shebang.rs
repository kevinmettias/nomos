//! `script_discipline.rs`'s shebang-portability rule: a first line that names its interpreter
//! by absolute path rather than reaching it through `/usr/bin/env`.

use super::{Because, Finding_For_Source, Rule, SCRIPTS_USE_A_PORTABLE_SHEBANG, Shebang_Interpreter};
use crate::SourceFile;
use nomos_contracts::Finding;

/// Reports shebang scripts whose first line hardcodes an interpreter path.
#[must_use]
pub fn Check_Scripts_Use_A_Portable_Shebang(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings: Vec<Finding> = sources.iter().filter_map(Portable_Shebang_Finding_For).collect();

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Portable_Shebang_Finding_For(source: &SourceFile) -> Option<Finding>
{
    let interpreter = Shebang_Interpreter(source)?;
    if interpreter.starts_with("/usr/bin/env ")
    {
        return None;
    }

    let finding = Finding_For_Source(
        source,
        Rule(SCRIPTS_USE_A_PORTABLE_SHEBANG),
        Because("has a hardcoded shebang; use `#!/usr/bin/env <interpreter>`"),
    );
    return Some(finding);
}

#[cfg(test)]
mod tests
{
    use super::super::tests::{Source, SourceText};
    use super::*;
    use nomos_contracts::RuleId;

    #[test]
    fn Test_Check_Scripts_Use_A_Portable_Shebang_Should_Report_A_Hardcoded_Shebang()
    {
        let source = Source(SourceText { path: "scripts/check.sh", text: "#!/bin/bash\n# check -- run checks\n" });

        let findings = Check_Scripts_Use_A_Portable_Shebang(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(SCRIPTS_USE_A_PORTABLE_SHEBANG));
    }

    #[test]
    fn Test_Check_Scripts_Use_A_Portable_Shebang_Should_Accept_A_Portable_Shebang()
    {
        let source = Source(SourceText { path: "scripts/check.sh", text: "#!/usr/bin/env bash\n# check -- run checks\n" });

        let findings = Check_Scripts_Use_A_Portable_Shebang(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// A Rust crate root opening on an inner attribute. `#![forbid(unsafe_code)]` begins
    /// with the same two bytes a shebang does, and this is the shape that made three of
    /// this workspace own crate roots report as scripts with a hardcoded interpreter.
    #[test]
    fn Test_Check_Scripts_Use_A_Portable_Shebang_Should_Ignore_A_Rust_Inner_Attribute()
    {
        let source = Source(SourceText { path: "src/lib.rs", text: "#![forbid(unsafe_code)]\n\npub fn Check() {}\n" });

        let findings = Check_Scripts_Use_A_Portable_Shebang(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// A space between the two bytes and the path is a shebang some conventions write, and
    /// it stays one: the discriminator is the absolute path, not the absence of a space.
    #[test]
    fn Test_Check_Scripts_Use_A_Portable_Shebang_Should_Still_Report_A_Spaced_Hardcoded_Shebang()
    {
        let source = Source(SourceText { path: "scripts/check.sh", text: "#! /bin/bash\n# check -- run checks\n" });

        let findings = Check_Scripts_Use_A_Portable_Shebang(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }
}
