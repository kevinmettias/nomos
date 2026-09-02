//! Two architecture-shaped rules from code-standards, grouped only because they landed in
//! the same increment as genuine one-offs — [`Check_A_Package_Is_Named_After_Its_Directory`]
//! is Go-specific placement (a package's name must match its directory), [`Check_No_
//! Wildcard_Imports`] is a cross-language visibility rule (an import must name what it
//! brings in). Neither shares a shape with the other or with anything else this crate
//! ships, so neither earned a capability or a shared helper — both are decidable from
//! [`SourceFile`] text alone.

use crate::{GO_LANGUAGE, RUST_LANGUAGE, SourceFile};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// The code-standards Go package-placement rule id.
pub const A_PACKAGE_IS_NAMED_AFTER_ITS_DIRECTORY: &str = "a-package-is-named-after-its-directory";
/// The code-standards wildcard-import rule id.
pub const NO_WILDCARD_IMPORTS: &str = "no-wildcard-imports";

const MAIN_PACKAGE: &str = "main";
const TEST_PACKAGE_SUFFIX: &str = "_test";

/// Reports Go files whose declared `package` name does not match their directory's base
/// name (hyphens and underscores in the directory name are ignored for the comparison),
/// exempting `package main` (the directory names the command, not the package) and an
/// external test package `package foo_test` living in directory `foo`.
#[must_use]
pub fn Check_A_Package_Is_Named_After_Its_Directory(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if !source.Is_Written_In(GO_LANGUAGE)
        {
            continue;
        }

        let Some(package) = Declared_Go_Package_Name(&source.text) else { continue };
        if package == MAIN_PACKAGE
        {
            continue;
        }

        let Some(directory) = Normalized_Directory_Name(&source.path) else { continue };
        if package == directory
        {
            continue;
        }
        if package.strip_suffix(TEST_PACKAGE_SUFFIX) == Some(directory.as_str())
        {
            continue;
        }

        findings.push(Finding_For_Line(
            source,
            A_PACKAGE_IS_NAMED_AFTER_ITS_DIRECTORY,
            1,
            &format!("declares `package {package}`, which does not match its directory `{directory}`"),
        ));
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// Reports a wildcard import in Rust (`use path::*;`) or Go (`import . "path"`), exempting
/// `use super::*;` once it appears after the file's own first `#[cfg(test)]` attribute (the
/// idiom a test module reaching for the subject it exercises) and a Go external test
/// package (`package foo_test`) dot-importing its own subject (`foo`). Neither this
/// crate's syntax payload nor a hand-rolled parser can tell a curated prelude from any
/// other wildcard, so that exemption is not attempted here — this rule only reports what
/// it can already tell apart.
#[must_use]
pub fn Check_No_Wildcard_Imports(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if source.Is_Written_In(RUST_LANGUAGE)
        {
            findings.extend(Rust_Wildcard_Findings_In(source));
        }
        else if source.Is_Written_In(GO_LANGUAGE)
        {
            findings.extend(Go_Wildcard_Findings_In(source));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Rust_Wildcard_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let lines: Vec<&str> = source.text.lines().collect();
    let first_test_cfg_line = lines.iter().position(|line| return line.contains("#[cfg(test)]"));

    let mut findings = Vec::new();
    for (index, line) in lines.iter().enumerate()
    {
        if !Is_Rust_Wildcard_Use_Line(line)
        {
            continue;
        }

        let exempt_test_idiom = Is_Use_Super_Star(line) && first_test_cfg_line.is_some_and(|cfg_line| return index > cfg_line);
        if exempt_test_idiom
        {
            continue;
        }

        findings.push(Finding_For_Line(source, NO_WILDCARD_IMPORTS, index.saturating_add(1), "is a wildcard import; name what it brings in"));
    }

    return findings;
}

fn Go_Wildcard_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let declared_package = Declared_Go_Package_Name(&source.text);
    let test_subject = declared_package.as_deref().and_then(|name| return name.strip_suffix(TEST_PACKAGE_SUFFIX));

    let mut findings = Vec::new();
    for (index, line) in source.text.lines().enumerate()
    {
        let Some(path) = Go_Dot_Import_Path(line) else { continue };

        if let Some(subject) = test_subject
        {
            let last_segment = path.rsplit('/').next().unwrap_or(path);
            if last_segment == subject
            {
                continue;
            }
        }

        findings.push(Finding_For_Line(source, NO_WILDCARD_IMPORTS, index.saturating_add(1), "is a wildcard (dot) import; name what it brings in"));
    }

    return findings;
}

fn Is_Rust_Wildcard_Use_Line(line: &str) -> bool
{
    let trimmed = line.trim();
    let after_visibility = trimmed
        .strip_prefix("pub(crate) ")
        .or_else(|| return trimmed.strip_prefix("pub(super) "))
        .or_else(|| return trimmed.strip_prefix("pub "))
        .unwrap_or(trimmed);

    return after_visibility.starts_with("use ") && trimmed.ends_with("::*;");
}

fn Is_Use_Super_Star(line: &str) -> bool
{
    return line.trim() == "use super::*;";
}

/// The path a Go dot-import line names, or `None` if `line` is not one — a bare `. "path"`
/// (grouped import block) or `import . "path"` (single-line form).
fn Go_Dot_Import_Path(line: &str) -> Option<&str>
{
    let trimmed = line.trim();
    let after_keyword = trimmed.strip_prefix("import ").map_or(trimmed, str::trim_start);
    let after_dot = after_keyword.strip_prefix('.')?.trim_start();
    let quoted = after_dot.strip_prefix('"')?;

    return quoted.split('"').next();
}

/// The identifier after `package` on the file's own first uncommented `package` line, or
/// `None` if it declares none.
fn Declared_Go_Package_Name(text: &str) -> Option<String>
{
    for line in text.lines()
    {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix("package ") else { continue };
        let name = rest.split_whitespace().next()?;
        return Some(name.to_owned());
    }

    return None;
}

/// `path`'s parent directory's base name, with every hyphen and underscore removed, the
/// same normalization `a-package-is-named-after-its-directory` states for the comparison.
fn Normalized_Directory_Name(path: &str) -> Option<String>
{
    let normalized = path.replace('\\', "/");
    let (directory, _file) = normalized.rsplit_once('/')?;
    let base = directory.rsplit('/').next()?;

    return Some(base.chars().filter(|character| return *character != '-' && *character != '_').collect());
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
    fn Test_Check_A_Package_Is_Named_After_Its_Directory_Should_Report_A_Mismatch()
    {
        let source = Source("check/tool.go", "package badname\n");

        let findings = Check_A_Package_Is_Named_After_Its_Directory(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(A_PACKAGE_IS_NAMED_AFTER_ITS_DIRECTORY));
    }

    #[test]
    fn Test_Check_A_Package_Is_Named_After_Its_Directory_Should_Accept_A_Match()
    {
        let source = Source("checktooldocs/reader.go", "package checktooldocs\n");

        let findings = Check_A_Package_Is_Named_After_Its_Directory(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Package_Is_Named_After_Its_Directory_Should_Ignore_Hyphens_And_Underscores()
    {
        let source = Source("check-tool_docs/reader.go", "package checktooldocs\n");

        let findings = Check_A_Package_Is_Named_After_Its_Directory(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Package_Is_Named_After_Its_Directory_Should_Exempt_Package_Main()
    {
        let source = Source("cmd/tool/main.go", "package main\n");

        let findings = Check_A_Package_Is_Named_After_Its_Directory(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Package_Is_Named_After_Its_Directory_Should_Exempt_An_External_Test_Package()
    {
        let source = Source("widget/widget_ext_test.go", "package widget_test\n");

        let findings = Check_A_Package_Is_Named_After_Its_Directory(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_No_Wildcard_Imports_Should_Report_A_Rust_Wildcard()
    {
        let source = Source("src/lib.rs", "use acme_math::signals::filters::*;\n");

        let findings = Check_No_Wildcard_Imports(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(NO_WILDCARD_IMPORTS));
    }

    #[test]
    fn Test_Check_No_Wildcard_Imports_Should_Accept_A_Named_Import()
    {
        let source = Source("src/lib.rs", "use acme_math::signals::filters::Biquad;\n");

        let findings = Check_No_Wildcard_Imports(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_No_Wildcard_Imports_Should_Exempt_Use_Super_Star_Inside_A_Test_Module()
    {
        let source = Source(
            "src/lib.rs",
            "pub fn Compute() {}\n\n#[cfg(test)]\nmod tests\n{\n    use super::*;\n}\n",
        );

        let findings = Check_No_Wildcard_Imports(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_No_Wildcard_Imports_Should_Report_Use_Super_Star_In_Production_Code()
    {
        let source = Source("src/mod.rs", "use super::*;\n");

        let findings = Check_No_Wildcard_Imports(&[source]);

        assert_eq!(findings.len(), 1, "a wildcard reaching for the enclosing scope outside a test is still a finding: {findings:?}");
    }

    #[test]
    fn Test_Check_No_Wildcard_Imports_Should_Report_A_Go_Dot_Import()
    {
        let source = Source("main.go", "import . \"acme/widget\"\n");

        let findings = Check_No_Wildcard_Imports(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_No_Wildcard_Imports_Should_Report_A_Go_Dot_Import_In_A_Grouped_Block()
    {
        let source = Source("main.go", "import (\n\t\"fmt\"\n\t. \"acme/widget\"\n)\n");

        let findings = Check_No_Wildcard_Imports(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_No_Wildcard_Imports_Should_Exempt_A_Go_External_Test_Package_Dot_Importing_Its_Own_Subject()
    {
        let source = Source("widget/widget_test.go", "package widget_test\n\nimport . \"acme/widget\"\n");

        let findings = Check_No_Wildcard_Imports(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_No_Wildcard_Imports_Should_Report_A_Go_External_Test_Package_Dot_Importing_Something_Else()
    {
        let source = Source("widget/widget_test.go", "package widget_test\n\nimport . \"acme/other\"\n");

        let findings = Check_No_Wildcard_Imports(&[source]);

        assert_eq!(findings.len(), 1, "a test wildcard-importing something other than its own subject is still a finding: {findings:?}");
    }

    fn Source(path: &str, text: &str) -> SourceFile
    {
        let mut source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
        source.language = crate::Recognized_Language_In_Tests(path);
        return source;
    }
}
