//! Two architecture-shaped rules from code-standards, grouped only because they landed in
//! the same increment as genuine one-offs — [`Check_A_Package_Is_Named_After_Its_Directory`]
//! is Go-specific placement (a package's name must match its directory), [`Check_No_
//! Wildcard_Imports`] is a cross-language visibility rule (an import must name what it
//! brings in). Neither shares a shape with the other or with anything else this crate
//! ships, so neither earned a capability or a shared helper — both are decidable from
//! [`SourceFile`] text alone.

use crate::{GO_LANGUAGE, RUST_LANGUAGE, SourceFile};
use nomos_analysis::FactReader;
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
        if let Some(finding) = Package_Mismatch_Finding(source)
        {
            findings.push(finding);
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// The one finding a Go source contributes when its declared package does not match its
/// directory, or `None` when the source is not Go, declares no package, is `package main`,
/// already matches, or is an external test package matching its subject's directory.
fn Package_Mismatch_Finding(source: &SourceFile) -> Option<Finding>
{
    if !source.Is_Written_In(GO_LANGUAGE)
    {
        return None;
    }

    let package = Non_Main_Package_Name(&source.text)?;
    let directory = Normalized_Directory_Name(&source.path)?;
    if Is_Package_Matching_Directory(&package, Directory(&directory))
    {
        return None;
    }

    return Some(Finding_For_Line(
        source,
        A_PACKAGE_IS_NAMED_AFTER_ITS_DIRECTORY,
        1,
        &format!("declares `package {package}`, which does not match its directory `{directory}`"),
    ));
}

/// The declared Go package name, or `None` if the source declares none or declares
/// `package main` -- the directory names the command, not the package, so `main` is exempt.
fn Non_Main_Package_Name(text: &str) -> Option<String>
{
    let package = Declared_Go_Package_Name(text)?;
    if package == MAIN_PACKAGE
    {
        return None;
    }

    return Some(package);
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

/// A normalized directory name, wrapped so it cannot be transposed with a package name at a
/// call site: both are `&str`, but only one is what the package is checked against.
struct Directory<'a>(&'a str);

/// Whether `package` names `directory` -- exactly, or as the external test package
/// `<directory>_test`.
fn Is_Package_Matching_Directory(package: &str, directory: Directory<'_>) -> bool
{
    let directory = directory.0;

    return package == directory || package.strip_suffix(TEST_PACKAGE_SUFFIX) == Some(directory);
}

/// Reports a wildcard import in Rust (`use path::*;`) or Go (`import . "path"`). Two Rust
/// idioms are exempt, both a test file reaching for shared test infrastructure rather than
/// naming every item it needs one by one: `use super::*;` once it appears after the file's
/// own first `#[cfg(test)]` attribute (the inline `#[cfg(test)] mod tests { use super::*;
/// ... }` shape), and *any* wildcard import when the whole file is
/// [`super::Is_Test_Or_Example_Source`]'s shape. The second covers what the first cannot
/// see: a `mod tests;` split into its own `tests.rs` carries the `#[cfg(test)]` gate in its
/// *parent* file, invisible to a check reading this file's text alone, and a directory-per-
/// test-binary integration suite (`tests/<suite>/main.rs` declaring sibling modules) reaches
/// for a shared fixture module by name (`use crate::board::*;`) rather than `super`, since
/// there is no parent module to reach through. Every wildcard import this workspace's own
/// test/example sources carry today is one of these two shapes; a Go external test package
/// (`package foo_test`) dot-importing its own subject (`foo`) is exempt the same way for the
/// same reason. Neither this crate's syntax payload nor a hand-rolled parser can tell a
/// curated prelude from any other wildcard in *non*-test code, so that exemption is not
/// attempted there — this rule only reports what it can already tell apart.
#[must_use]
pub fn Check_No_Wildcard_Imports(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let declared = crate::checks::Resolve_Declared_Fixture_Locations(facts);
    let mut findings = Vec::new();

    for source in sources
    {
        if source.Is_Written_In(RUST_LANGUAGE)
        {
            findings.extend(Rust_Wildcard_Findings_In(source, &declared));
        }
        else if source.Is_Written_In(GO_LANGUAGE)
        {
            findings.extend(Go_Wildcard_Findings_In(source));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Rust_Wildcard_Findings_In(source: &SourceFile, declared: &[String]) -> Vec<Finding>
{
    let lines: Vec<&str> = source.text.lines().collect();
    let first_test_cfg_line = lines.iter().position(|line| return line.contains("#[cfg(test)]"));
    let whole_file_is_a_test_module = super::Is_Test_Or_Example_Source(source, declared);

    let mut findings = Vec::new();
    for (index, line) in lines.iter().enumerate()
    {
        let test_context = TestContext { whole_file_is_a_test_module, first_test_cfg_line };
        let finding = Rust_Wildcard_Finding_For_Line(source, line, index, test_context);
        if let Some(finding) = finding
        {
            findings.push(finding);
        }
    }

    return findings;
}

/// Whether `index`'s line sits in a file or a region this rule already exempts as test
/// idiom, packaged so its bare `bool` does not sit among [`Rust_Wildcard_Finding_For_Line`]'s
/// other positional parameters.
struct TestContext
{
    whole_file_is_a_test_module: bool,
    first_test_cfg_line: Option<usize>,
}

fn Rust_Wildcard_Finding_For_Line(source: &SourceFile, line: &str, index: usize, test_context: TestContext) -> Option<Finding>
{
    if !Is_Rust_Wildcard_Use_Line(line)
    {
        return None;
    }

    let exempt_test_idiom = test_context.whole_file_is_a_test_module
        || (Is_Use_Super_Star(line) && test_context.first_test_cfg_line.is_some_and(|cfg_line| return index > cfg_line));
    if exempt_test_idiom
    {
        return None;
    }

    return Some(Finding_For_Line(source, NO_WILDCARD_IMPORTS, index.saturating_add(1), "is a wildcard import; name what it brings in"));
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

        let finding = Finding_For_Line(source, NO_WILDCARD_IMPORTS, index.saturating_add(1), "is a wildcard (dot) import; name what it brings in");
        findings.push(finding);
    }

    return findings;
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

fn Finding_For_Line(source: &SourceFile, rule: &str, line_number: usize, because: &str) -> Finding
{
    return Finding {
        address: None,
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
    use nomos_analysis::{FactReader, MemoryFactStore, Reader};
    use nomos_capability::Registry;
    use nomos_contracts::SubjectId;
    use nomos_model::Content_Digest;

    #[test]
    fn Test_Check_A_Package_Is_Named_After_Its_Directory_Should_Report_A_Mismatch()
    {
        let source = Source_File(SourceText { path: "check/tool.go", text: "package badname\n" });

        let findings = Check_A_Package_Is_Named_After_Its_Directory(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(A_PACKAGE_IS_NAMED_AFTER_ITS_DIRECTORY));
    }

    #[test]
    fn Test_Check_A_Package_Is_Named_After_Its_Directory_Should_Accept_A_Match()
    {
        let source = Source_File(SourceText { path: "checktooldocs/reader.go", text: "package checktooldocs\n" });

        let findings = Check_A_Package_Is_Named_After_Its_Directory(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Package_Is_Named_After_Its_Directory_Should_Ignore_Hyphens_And_Underscores()
    {
        let source = Source_File(SourceText { path: "check-tool_docs/reader.go", text: "package checktooldocs\n" });

        let findings = Check_A_Package_Is_Named_After_Its_Directory(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Package_Is_Named_After_Its_Directory_Should_Exempt_Package_Main()
    {
        let source = Source_File(SourceText { path: "cmd/tool/main.go", text: "package main\n" });

        let findings = Check_A_Package_Is_Named_After_Its_Directory(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Package_Is_Named_After_Its_Directory_Should_Exempt_An_External_Test_Package()
    {
        let source = Source_File(SourceText { path: "widget/widget_ext_test.go", text: "package widget_test\n" });

        let findings = Check_A_Package_Is_Named_After_Its_Directory(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_No_Wildcard_Imports_Should_Report_A_Rust_Wildcard()
    {
        let source = Source_File(SourceText { path: "src/lib.rs", text: "use acme_math::signals::filters::*;\n" });

        let findings = Findings_From_Check(Check_No_Wildcard_Imports, source);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(NO_WILDCARD_IMPORTS));
    }

    #[test]
    fn Test_Check_No_Wildcard_Imports_Should_Accept_A_Named_Import()
    {
        let source = Source_File(SourceText { path: "src/lib.rs", text: "use acme_math::signals::filters::Biquad;\n" });

        let findings = Findings_From_Check(Check_No_Wildcard_Imports, source);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_No_Wildcard_Imports_Should_Exempt_Use_Super_Star_Inside_A_Test_Module()
    {
        let source = Source_File(
            SourceText { path: "src/lib.rs", text: "pub fn Compute() {}\n\n#[cfg(test)]\nmod tests\n{\n use super::*;\n}\n" },
        );

        let findings = Findings_From_Check(Check_No_Wildcard_Imports, source);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// `#[cfg(test)] mod tests;` split into its own `tests.rs` carries the gate in its
    /// *parent* file -- this file's own text never contains `#[cfg(test)]` at all, the shape
    /// most of this workspace's real findings shared before this exemption existed.
    #[test]
    fn Test_Check_No_Wildcard_Imports_Should_Exempt_Use_Super_Star_In_A_Standalone_Tests_File()
    {
        let source = Source_File(SourceText { path: "src/module/tests.rs", text: "use super::*;\n\n#[test]\nfn Test_Compute() {}\n" });

        let findings = Findings_From_Check(Check_No_Wildcard_Imports, source);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// A directory-per-test-binary integration suite (`tests/<suite>/main.rs` declaring
    /// sibling modules) has no parent module to reach through, so a test file wildcard-
    /// imports a shared fixture module by name instead -- `crates/substrate/nomos-ledger/
    /// tests/exclusion_holds/*.rs`'s own `use crate::board::*;` is exactly this shape, and
    /// the remaining real findings once `use super::*;` alone was exempted.
    #[test]
    fn Test_Check_No_Wildcard_Imports_Should_Exempt_A_Named_Wildcard_In_A_Test_File()
    {
        let source = Source_File(SourceText { path: "tests/suite/claiming.rs", text: "use crate::board::*;\n\n#[test]\nfn Test_Claim() {}\n" });

        let findings = Findings_From_Check(Check_No_Wildcard_Imports, source);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_No_Wildcard_Imports_Should_Report_Use_Super_Star_In_Production_Code()
    {
        let source = Source_File(SourceText { path: "src/mod.rs", text: "use super::*;\n" });

        let findings = Findings_From_Check(Check_No_Wildcard_Imports, source);

        assert_eq!(findings.len(), 1, "a wildcard reaching for the enclosing scope outside a test is still a finding: {findings:?}");
    }

    #[test]
    fn Test_Check_No_Wildcard_Imports_Should_Report_A_Go_Dot_Import()
    {
        let source = Source_File(SourceText { path: "main.go", text: "import . \"acme/widget\"\n" });

        let findings = Findings_From_Check(Check_No_Wildcard_Imports, source);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_No_Wildcard_Imports_Should_Report_A_Go_Dot_Import_In_A_Grouped_Block()
    {
        let source = Source_File(SourceText { path: "main.go", text: "import (\n\t\"fmt\"\n\t. \"acme/widget\"\n)\n" });

        let findings = Findings_From_Check(Check_No_Wildcard_Imports, source);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_No_Wildcard_Imports_Should_Exempt_A_Go_External_Test_Package_Dot_Importing_Its_Own_Subject()
    {
        let source = Source_File(SourceText { path: "widget/widget_test.go", text: "package widget_test\n\nimport . \"acme/widget\"\n" });

        let findings = Findings_From_Check(Check_No_Wildcard_Imports, source);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_No_Wildcard_Imports_Should_Report_A_Go_External_Test_Package_Dot_Importing_Something_Else()
    {
        let source = Source_File(SourceText { path: "widget/widget_test.go", text: "package widget_test\n\nimport . \"acme/other\"\n" });

        let findings = Findings_From_Check(Check_No_Wildcard_Imports, source);

        assert_eq!(findings.len(), 1, "a test wildcard-importing something other than its own subject is still a finding: {findings:?}");
    }

        /// A fixture source's two halves, grouped so a call site names which string is the path
    /// and which is the text, rather than counting two adjacent `&str` positions a caller
    /// could transpose without the compiler objecting.
    struct SourceText<'text>
    {
        path: &'text str,
        text: &'text str,
    }

    fn Source_File(source: SourceText<'_>) -> SourceFile
    {
        let SourceText { path, text } = source;
        let mut source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
        source.language = crate::Recognized_Language_In_Tests(path);
        return source;
    }

    /// Runs `check` over `source` through a real, empty reader — this check reads only
    /// `nomos.cap.test.material.policy`, which no fixture here declares, so `Require` fails
    /// and it resolves to its own fixed clauses alone.
    fn Findings_From_Check(check: fn(&[SourceFile], &mut dyn FactReader) -> Vec<Finding>, source: SourceFile) -> Vec<Finding>
    {
        let store = MemoryFactStore::New();
        let registry = Registry::New();
        let mut facts = Reader::On(&store, &registry, crate::checks::test_support::Test_Context());
        return check(&[source], &mut facts);
    }
}
