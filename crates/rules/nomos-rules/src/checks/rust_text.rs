//! Rust source-text rules from code-standards.
//!
//! These checks deliberately stay conservative and text-local. They import standards whose
//! deciding evidence is visible in one Rust source file: panic primitive spelling, path
//! attributes, shared `Rc`/`Arc` plus `RefCell` ownership escapes, and — the two rules this
//! file adds beyond its original four — `#[allow(...)]` and `unsafe` constructs that carry
//! no adjacent explanatory comment. Both new rules reuse [`Previous_Comment_Block_Has`],
//! the same "walk the contiguous comment block immediately above this line" primitive
//! [`Panic_Findings_In`] and [`Shared_Interior_Mutability_Findings_In`] already share —
//! a real second and third consumer, not a new abstraction invented for them.

use crate::{RUST_LANGUAGE, SourceFile};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};
use std::path::Component;

/// The code-standards `unwrap`/`expect` discipline rule id.
pub const UNWRAP_EXPECT_DISCIPLINE: &str = "unwrap-expect-discipline";
/// The code-standards explicit-panic justification rule id.
pub const PANICS_ARE_JUSTIFIED_DOCUMENTED_AND_VALIDATED: &str = "panics-are-justified-documented-and-validated";
/// The code-standards Rust path-attribute rule id.
pub const A_RUST_PATH_STAYS_WITHIN_ITS_OWN_SUBTREE: &str = "a-rust-path-stays-within-its-own-subtree";
/// The code-standards shared-interior-mutability rule id.
pub const SHARED_INTERIOR_MUTABILITY_SAYS_WHY: &str = "shared-interior-mutability-says-why";
/// The code-standards `#[allow(...)]` justification rule id.
pub const EVERY_ALLOW_CARRIES_A_JUSTIFICATION: &str = "every-allow-carries-a-justification";
/// The code-standards `unsafe` justification rule id.
pub const UNSAFE_JUSTIFICATION: &str = "unsafe-justification";

/// Reports `unwrap()` and placeholder `expect(...)` outside test and example Rust sources.
#[must_use]
pub fn Check_Unwrap_Expect_Discipline(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if source.Is_Written_In(RUST_LANGUAGE) && !Is_Test_Or_Example_Source(source)
        {
            findings.extend(Unwrap_Expect_Findings_In(source));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// Reports explicit panic primitives that do not carry a local panic/invariant note.
#[must_use]
pub fn Check_Panics_Are_Justified_Documented_And_Validated(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if source.Is_Written_In(RUST_LANGUAGE)
        {
            findings.extend(Panic_Findings_In(source));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// Reports Rust `#[path = "..."]` attributes whose value is absolute or escapes upward.
#[must_use]
pub fn Check_A_Rust_Path_Stays_Within_Its_Own_Subtree(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if source.Is_Written_In(RUST_LANGUAGE)
        {
            findings.extend(Path_Attribute_Findings_In(source));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// Reports shared `Rc`/`Arc` plus `RefCell` constructs that do not say why.
#[must_use]
pub fn Check_Shared_Interior_Mutability_Says_Why(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if source.Is_Written_In(RUST_LANGUAGE)
        {
            findings.extend(Shared_Interior_Mutability_Findings_In(source));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// Reports `#[allow(...)]`/`#![allow(...)]` attributes with no adjacent explanatory comment.
#[must_use]
pub fn Check_Every_Allow_Carries_A_Justification(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if source.Is_Written_In(RUST_LANGUAGE)
        {
            findings.extend(Allow_Findings_In(source));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// Reports `unsafe` blocks, functions, impls and traits with no adjacent `// SAFETY:` comment.
#[must_use]
pub fn Check_Unsafe_Justification(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if source.Is_Written_In(RUST_LANGUAGE)
        {
            findings.extend(Unsafe_Findings_In(source));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Unwrap_Expect_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for (index, line) in source.text.lines().enumerate()
    {
        if Code_Prefix(line).contains(".unwrap()")
        {
            findings.push(Finding_For_Line(
                source,
                UNWRAP_EXPECT_DISCIPLINE,
                Line_Number(index),
                "uses `unwrap()` outside tests/examples",
            ));
        }

        if Placeholder_Expect(Code_Prefix(line))
        {
            findings.push(Finding_For_Line(
                source,
                UNWRAP_EXPECT_DISCIPLINE,
                Line_Number(index),
                "uses `expect(...)` without naming an invariant",
            ));
        }
    }

    return findings;
}

fn Panic_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let lines = Lines_Of(source);
    let mut findings = Vec::new();

    for (index, line) in lines.iter().enumerate()
    {
        let code = Code_Prefix(line);
        if Has_Panic_Primitive(code) && !Has_Local_Panic_Justification(&lines, index)
        {
            findings.push(Finding_For_Line(
                source,
                PANICS_ARE_JUSTIFIED_DOCUMENTED_AND_VALIDATED,
                Line_Number(index),
                "uses a panic primitive without a local panic or invariant note",
            ));
        }
    }

    return findings;
}

fn Path_Attribute_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for (index, line) in source.text.lines().enumerate()
    {
        let code = Code_Prefix(line);
        if let Some(value) = Path_Attribute_Value(code)
        {
            if Path_Is_Absolute_Or_Escaping(value)
            {
                findings.push(Finding_For_Line(
                    source,
                    A_RUST_PATH_STAYS_WITHIN_ITS_OWN_SUBTREE,
                    Line_Number(index),
                    "`#[path]` leaves the declaring file subtree",
                ));
            }
        }
    }

    return findings;
}

fn Shared_Interior_Mutability_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let lines = Lines_Of(source);
    let mut findings = Vec::new();

    for (index, line) in lines.iter().enumerate()
    {
        let code = Code_Prefix(line);
        if Has_Shared_RefCell_Construct(code) && !Has_Local_Smart_Pointer_Reason(&lines, index)
        {
            findings.push(Finding_For_Line(
                source,
                SHARED_INTERIOR_MUTABILITY_SAYS_WHY,
                Line_Number(index),
                "uses shared interior mutability without `smart-pointer: allow: <reason>`",
            ));
        }
    }

    return findings;
}

fn Allow_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let lines = Lines_Of(source);
    let mut findings = Vec::new();

    for (index, line) in lines.iter().enumerate()
    {
        let code = Code_Prefix(line);
        if Has_Allow_Attribute(code) && !Has_Local_Allow_Justification(&lines, index)
        {
            findings.push(Finding_For_Line(
                source,
                EVERY_ALLOW_CARRIES_A_JUSTIFICATION,
                Line_Number(index),
                "carries an #[allow(...)] with no adjacent comment explaining why",
            ));
        }
    }

    return findings;
}

fn Unsafe_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let lines = Lines_Of(source);
    let mut findings = Vec::new();

    for (index, line) in lines.iter().enumerate()
    {
        let code = Code_Prefix(line);
        if Has_Unsafe_Construct(code) && !Has_Local_Safety_Justification(&lines, index)
        {
            findings.push(Finding_For_Line(
                source,
                UNSAFE_JUSTIFICATION,
                Line_Number(index),
                "uses `unsafe` without an adjacent `// SAFETY:` comment",
            ));
        }
    }

    return findings;
}

fn Is_Test_Or_Example_Source(source: &SourceFile) -> bool
{
    let normalized = source.path.replace('\\', "/");
    return normalized.starts_with("tests/")
        || normalized.starts_with("examples/")
        || normalized.contains("/tests/")
        || normalized.contains("/test/")
        || normalized.ends_with("_test.rs")
        || normalized.ends_with("_tests.rs");
}

fn Lines_Of(source: &SourceFile) -> Vec<&str>
{
    return source.text.lines().collect();
}

fn Line_Number(index: usize) -> usize
{
    return index.saturating_add(1);
}

fn Code_Prefix(line: &str) -> &str
{
    return line.split("//").next().unwrap_or(line);
}

fn Placeholder_Expect(code: &str) -> bool
{
    let Some(after_call) = code.split(".expect(").nth(1)
    else
    {
        return false;
    };

    let lower = after_call.to_ascii_lowercase();
    return lower.contains("\"should not happen\"")
        || lower.contains("\"impossible\"")
        || lower.contains("\"unreachable\"")
        || lower.contains("\"todo\"")
        || lower.contains("\"fixme\"");
}

fn Has_Panic_Primitive(code: &str) -> bool
{
    return code.contains("panic!(")
        || code.contains("todo!(")
        || code.contains("unimplemented!(")
        || code.contains("unreachable!(");
}

fn Has_Local_Panic_Justification(lines: &[&str], index: usize) -> bool
{
    if lines.get(index).is_some_and(|line| return Comment_Has_Panic_Reason(line))
    {
        return true;
    }

    return Previous_Comment_Block_Has(lines, index, Comment_Has_Panic_Reason);
}

fn Comment_Has_Panic_Reason(line: &str) -> bool
{
    let Some(comment) = Comment_Text_Of(line)
    else
    {
        return false;
    };

    let lower = comment.to_ascii_lowercase();
    return lower.contains("panic:")
        || lower.contains("panics:")
        || lower.contains("invariant:")
        || lower.contains("# panics");
}

fn Path_Attribute_Value(code: &str) -> Option<&str>
{
    let attribute_start = code.find("#[path")?;
    let after_attribute = code.get(attribute_start..)?;
    let first_quote = after_attribute.find('"')?;
    let after_first_quote = after_attribute.get(first_quote.saturating_add(1)..)?;
    let second_quote = after_first_quote.find('"')?;
    return after_first_quote.get(..second_quote);
}

fn Path_Is_Absolute_Or_Escaping(value: &str) -> bool
{
    let path = std::path::Path::new(value);
    if path.is_absolute()
    {
        return true;
    }

    let mut depth = 0usize;
    for component in path.components()
    {
        match component
        {
            Component::ParentDir =>
            {
                let Some(next_depth) = depth.checked_sub(1)
                else
                {
                    return true;
                };
                depth = next_depth;
            }
            Component::Normal(_) => depth = depth.saturating_add(1),
            Component::CurDir => {}
            Component::RootDir | Component::Prefix(_) => return true,
        }
    }

    return false;
}

fn Has_Shared_RefCell_Construct(code: &str) -> bool
{
    let compact = code
        .chars()
        .filter(|character| return !character.is_whitespace())
        .collect::<String>();

    return Shared_Type_Contains_RefCell(&compact, "Rc")
        || Shared_Type_Contains_RefCell(&compact, "Arc")
        || compact.contains("Rc::new(RefCell::new(")
        || compact.contains("Arc::new(RefCell::new(")
        || compact.contains("Rc::<RefCell<")
        || compact.contains("Arc::<RefCell<");
}

fn Shared_Type_Contains_RefCell(compact: &str, wrapper: &str) -> bool
{
    let pattern = format!("{wrapper}<");
    let Some(start) = compact.find(&pattern)
    else
    {
        return false;
    };

    let Some(rest) = compact.get(start.saturating_add(pattern.len())..)
    else
    {
        return false;
    };

    let Some(end) = rest.find('>')
    else
    {
        return false;
    };

    return rest.get(..end).is_some_and(|inner| return inner.contains("RefCell<"));
}

fn Has_Local_Smart_Pointer_Reason(lines: &[&str], index: usize) -> bool
{
    if lines
        .get(index)
        .is_some_and(|line| return Comment_Has_Smart_Pointer_Reason(line))
    {
        return true;
    }

    return Previous_Comment_Block_Has(lines, index, Comment_Has_Smart_Pointer_Reason);
}

fn Comment_Has_Smart_Pointer_Reason(line: &str) -> bool
{
    let Some(comment) = Comment_Text_Of(line)
    else
    {
        return false;
    };

    let Some(reason) = comment.split("smart-pointer: allow:").nth(1)
    else
    {
        return false;
    };

    return !reason.trim().is_empty();
}

fn Has_Allow_Attribute(code: &str) -> bool
{
    return code.contains("#[allow(") || code.contains("#![allow(");
}

fn Has_Local_Allow_Justification(lines: &[&str], index: usize) -> bool
{
    if lines.get(index).is_some_and(|line| return Comment_Is_Non_Empty(line))
    {
        return true;
    }

    return Previous_Comment_Block_Has(lines, index, Comment_Is_Non_Empty);
}

/// `every-allow-carries-a-justification`'s own example is plain prose with no special
/// marker, unlike the panic and smart-pointer rules' `panic:`/`smart-pointer: allow:`
/// keywords — so any non-empty comment satisfies it.
fn Comment_Is_Non_Empty(line: &str) -> bool
{
    return Comment_Text_Of(line).is_some_and(|comment| return !comment.trim().is_empty());
}

/// Matches `unsafe {`, `unsafe fn`, `unsafe impl` and `unsafe trait` specifically — not a
/// bare substring search for `"unsafe"`, which would false-positive on
/// `#![forbid(unsafe_code)]`.
fn Has_Unsafe_Construct(code: &str) -> bool
{
    return code.contains("unsafe {")
        || code.contains("unsafe fn ")
        || code.contains("unsafe fn(")
        || code.contains("unsafe impl")
        || code.contains("unsafe trait");
}

fn Has_Local_Safety_Justification(lines: &[&str], index: usize) -> bool
{
    if lines.get(index).is_some_and(|line| return Comment_Has_Safety_Reason(line))
    {
        return true;
    }

    return Previous_Comment_Block_Has(lines, index, Comment_Has_Safety_Reason);
}

fn Comment_Has_Safety_Reason(line: &str) -> bool
{
    let Some(comment) = Comment_Text_Of(line)
    else
    {
        return false;
    };

    return comment.to_ascii_lowercase().starts_with("safety:");
}

fn Previous_Comment_Block_Has(lines: &[&str], index: usize, predicate: fn(&str) -> bool) -> bool
{
    let mut cursor = index;
    while let Some(previous) = cursor.checked_sub(1)
    {
        let Some(line) = lines.get(previous)
        else
        {
            return false;
        };
        if line.trim().is_empty() || Is_Attribute_Line(line)
        {
            cursor = previous;
            continue;
        }

        if !Is_Comment_Line(line)
        {
            return false;
        }

        if predicate(line)
        {
            return true;
        }

        cursor = previous;
    }

    return false;
}

fn Is_Attribute_Line(line: &str) -> bool
{
    return line.trim_start().starts_with("#[");
}

fn Is_Comment_Line(line: &str) -> bool
{
    return Comment_Text_Of(line).is_some();
}

fn Comment_Text_Of(line: &str) -> Option<&str>
{
    let trimmed = line.trim_start();

    for marker in ["//", "///", "//!"]
    {
        if let Some(comment) = trimmed.strip_prefix(marker)
        {
            return Some(comment.trim_start());
        }
    }

    return None;
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
    fn Test_Check_Unwrap_Expect_Discipline_Should_Report_Unwrap_In_Production_Rust()
    {
        let source = Source("src/lib.rs", "let value = option.unwrap();\n");

        let findings = Check_Unwrap_Expect_Discipline(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(UNWRAP_EXPECT_DISCIPLINE));
    }

    #[test]
    fn Test_Check_Unwrap_Expect_Discipline_Should_Ignore_Test_Rust()
    {
        let source = Source("tests/parser.rs", "let value = option.unwrap();\n");

        let findings = Check_Unwrap_Expect_Discipline(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Unwrap_Expect_Discipline_Should_Report_Placeholder_Expect()
    {
        let source = Source("src/lib.rs", "let value = result.expect(\"should not happen\");\n");

        let findings = Check_Unwrap_Expect_Discipline(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_Panics_Are_Justified_Documented_And_Validated_Should_Report_Unexplained_Panic()
    {
        let source = Source("src/lib.rs", "fn Crash()\n{\n    panic!(\"bad\");\n}\n");

        let findings = Check_Panics_Are_Justified_Documented_And_Validated(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(
            findings.first().expect("asserted len 1 above").rule,
            RuleId::New(PANICS_ARE_JUSTIFIED_DOCUMENTED_AND_VALIDATED)
        );
    }

    #[test]
    fn Test_Check_Panics_Are_Justified_Documented_And_Validated_Should_Accept_A_Local_Invariant_Note()
    {
        let source = Source(
            "src/lib.rs",
            "fn Crash()\n{\n    // invariant: the parser produced a non-empty token stack\n    unreachable!();\n}\n",
        );

        let findings = Check_Panics_Are_Justified_Documented_And_Validated(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Rust_Path_Stays_Within_Its_Own_Subtree_Should_Report_Parent_Escape()
    {
        let source = Source("src/lib.rs", "#[path = \"../outside.rs\"]\nmod outside;\n");

        let findings = Check_A_Rust_Path_Stays_Within_Its_Own_Subtree(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(
            findings.first().expect("asserted len 1 above").rule,
            RuleId::New(A_RUST_PATH_STAYS_WITHIN_ITS_OWN_SUBTREE)
        );
    }

    #[test]
    fn Test_Check_A_Rust_Path_Stays_Within_Its_Own_Subtree_Should_Accept_Normalized_Internal_Parent()
    {
        let source = Source("src/lib.rs", "#[path = \"pool/../pool/tests.rs\"]\nmod tests;\n");

        let findings = Check_A_Rust_Path_Stays_Within_Its_Own_Subtree(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Shared_Interior_Mutability_Says_Why_Should_Report_Unexplained_Rc_RefCell()
    {
        let source = Source("src/lib.rs", "nodes: Vec<Rc<RefCell<Node>>>,\n");

        let findings = Check_Shared_Interior_Mutability_Says_Why(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(
            findings.first().expect("asserted len 1 above").rule,
            RuleId::New(SHARED_INTERIOR_MUTABILITY_SAYS_WHY)
        );
    }

    #[test]
    fn Test_Check_Shared_Interior_Mutability_Says_Why_Should_Accept_A_Reason()
    {
        let source = Source(
            "src/lib.rs",
            "// smart-pointer: allow: the graph is cyclic, so no single owner exists\nnodes: Rc<RefCell<Node>>,\n",
        );

        let findings = Check_Shared_Interior_Mutability_Says_Why(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Shared_Interior_Mutability_Says_Why_Should_Ignore_Arc_Mutex()
    {
        let source = Source("src/lib.rs", "state: Arc<Mutex<State>>,\n");

        let findings = Check_Shared_Interior_Mutability_Says_Why(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Every_Allow_Carries_A_Justification_Should_Report_An_Unexplained_Allow()
    {
        let source = Source("src/lib.rs", "#[allow(clippy::redundant_clone)]\nlet processed = input.clone();\n");

        let findings = Check_Every_Allow_Carries_A_Justification(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(
            findings.first().expect("asserted len 1 above").rule,
            RuleId::New(EVERY_ALLOW_CARRIES_A_JUSTIFICATION)
        );
    }

    #[test]
    fn Test_Check_Every_Allow_Carries_A_Justification_Should_Accept_An_Explained_Allow()
    {
        let source = Source(
            "src/lib.rs",
            "// the clone is required because the caller retains the original elsewhere\n\
             #[allow(clippy::redundant_clone)]\n\
             let processed = input.clone();\n",
        );

        let findings = Check_Every_Allow_Carries_A_Justification(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Every_Allow_Carries_A_Justification_Should_Accept_A_Crate_Level_Allow_With_A_Reason()
    {
        let source = Source(
            "src/lib.rs",
            "// this crate is a thin FFI shim and every public item is consumed externally\n#![allow(dead_code)]\n",
        );

        let findings = Check_Every_Allow_Carries_A_Justification(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Unsafe_Justification_Should_Report_An_Unexplained_Unsafe_Block()
    {
        let source = Source("src/lib.rs", "let slice = unsafe { core::slice::from_raw_parts(ptr, len) };\n");

        let findings = Check_Unsafe_Justification(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(UNSAFE_JUSTIFICATION));
    }

    #[test]
    fn Test_Check_Unsafe_Justification_Should_Accept_A_Safety_Comment()
    {
        let source = Source(
            "src/lib.rs",
            "// SAFETY:\n\
             // - ptr is valid for len elements, checked by the caller above\n\
             let slice = unsafe { core::slice::from_raw_parts(ptr, len) };\n",
        );

        let findings = Check_Unsafe_Justification(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Unsafe_Justification_Should_Ignore_The_Forbid_Unsafe_Code_Attribute()
    {
        let source = Source("src/lib.rs", "#![forbid(unsafe_code)]\n");

        let findings = Check_Unsafe_Justification(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Unsafe_Justification_Should_Report_An_Unsafe_Fn_With_No_Safety_Comment()
    {
        let source = Source("src/lib.rs", "pub unsafe fn Read_Raw(ptr: *const u8) -> u8\n{\n    return *ptr;\n}\n");

        let findings = Check_Unsafe_Justification(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    fn Source(path: &str, text: &str) -> SourceFile
    {
        let mut source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
        source.language = crate::Recognized_Language_In_Tests(path);
        return source;
    }
}
