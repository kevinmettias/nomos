//! The atomic-ordering-justification family from code-standards.
//!
//! Three rule ids, one mechanism, and code-standards' own `check-atomic-ordering` states why
//! it is one tool rather than three: the five `std::sync::atomic::Ordering` variants partition
//! exactly across the three standards -- `Relaxed` goes to
//! [`Check_Relaxed_Not_Used_When_Ordering_Matters`], `SeqCst` goes to
//! [`Check_Seqcst_Justified_Explicitly`], and the remaining three (`AcqRel`, `Acquire`,
//! `Release`) go to [`Check_Atomic_Ordering_Choices_Are_Justified`] -- so one flagged call site
//! is judged by exactly one of the three, never zero and never two.
//!
//! Deliberately narrower than code-standards' own tool in one respect, matching this crate's
//! existing convention (`rust_text::Is_Test_Or_Example_Source`, `security_text::
//! Is_Test_Or_Fixture_Source`) rather than inventing a new one: the exemption is file-level
//! (a whole `tests/`/`examples/` source, or a `_test.rs`/`_tests.rs` file), not code-standards'
//! finer per-line `Test_Context_Lines` brace-depth tracking of an individual `#[cfg(test)] mod
//! tests { ... }` block inside an otherwise-production file. A conservative-ordering call
//! written only inside such a block in a production file is judged the same as one anywhere
//! else in that file, which is the same trade-off this crate already made for `Check_Unwrap_
//! Expect_Discipline`.
//!
//! The doc's own worked example for the first rule --
//! `// Acquire: pairs with the Release store in submit_work; ...` -- does not carry the literal
//! marker code-standards' own implementation requires (`// atomic-ordering: allow: <reason>`);
//! read `rules/language-specific/rust/concurrency/atomic-ordering/main.go`'s `Has_Reason` call
//! rather than the doc's prose, which only illustrates the kind of reasoning owed and predates
//! the marker convention. This module follows the implementation, the same choice this crate
//! already made once for a stale `enforced_by` claim on `inline-always-requires-justification`.

use crate::{RUST_LANGUAGE, SourceFile};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// The code-standards non-`Relaxed`-non-`SeqCst` atomic-ordering rule id.
pub const ATOMIC_ORDERING_CHOICES_ARE_JUSTIFIED: &str = "atomic-ordering-choices-are-justified";
/// The code-standards `SeqCst`-specific rule id.
pub const SEQCST_JUSTIFIED_EXPLICITLY: &str = "seqcst-justified-explicitly";
/// The code-standards `Relaxed`-specific rule id.
pub const RELAXED_NOT_USED_WHEN_ORDERING_MATTERS: &str = "relaxed-not-used-when-ordering-matters";

/// The five variant names `std::sync::atomic::Ordering` carries, in match-priority order --
/// order does not matter for correctness here since each name is distinct, but it mirrors the
/// Go implementation's own regex alternation.
const ORDERING_VARIANTS: &[&str] = &["SeqCst", "AcqRel", "Acquire", "Release", "Relaxed"];

/// The literal marker this family's reason comment must lead with.
const ATOMIC_ORDERING_MARKER: &str = "atomic-ordering: allow";

/// Reports an `AcqRel`, `Acquire` or `Release` ordering with no adjacent `atomic-ordering:
/// allow` reason -- the "name the happens-before pairing" obligation.
#[must_use]
pub fn Check_Atomic_Ordering_Choices_Are_Justified(sources: &[SourceFile]) -> Vec<Finding>
{
    return Findings_For(sources, ATOMIC_ORDERING_CHOICES_ARE_JUSTIFIED, |variant| {
        return variant != "SeqCst" && variant != "Relaxed";
    });
}

/// Reports a `SeqCst` ordering with no adjacent `atomic-ordering: allow` reason -- the "name the
/// total order and why weaker would not do" obligation.
#[must_use]
pub fn Check_Seqcst_Justified_Explicitly(sources: &[SourceFile]) -> Vec<Finding>
{
    return Findings_For(sources, SEQCST_JUSTIFIED_EXPLICITLY, |variant| return variant == "SeqCst");
}

/// Reports a `Relaxed` ordering with no adjacent `atomic-ordering: allow` reason -- the "name why
/// no ordering guarantee is needed" obligation.
#[must_use]
pub fn Check_Relaxed_Not_Used_When_Ordering_Matters(sources: &[SourceFile]) -> Vec<Finding>
{
    return Findings_For(sources, RELAXED_NOT_USED_WHEN_ORDERING_MATTERS, |variant| return variant == "Relaxed");
}

fn Findings_For(sources: &[SourceFile], rule: &str, matches_partition: fn(&str) -> bool) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if source.Is_Written_In(RUST_LANGUAGE) && !Is_Test_Or_Example_Source(source) && !Is_Own_Implementation_File(source)
        {
            findings.extend(Ordering_Findings_In(source, rule, matches_partition));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Ordering_Findings_In(source: &SourceFile, rule: &str, matches_partition: fn(&str) -> bool) -> Vec<Finding>
{
    let lines: Vec<&str> = source.text.lines().collect();
    let mut findings = Vec::new();

    for (index, line) in lines.iter().enumerate()
    {
        let code = Code_Prefix(line);
        if Is_Import_Line(code)
        {
            continue;
        }

        let Some(variant) = Ordering_Match_In(code)
        else
        {
            continue;
        };

        if !matches_partition(variant) || Has_Marker_Reason(&lines, index)
        {
            continue;
        }

        let line_number = Line_Number(index);
        findings.push(Finding {
            rule: RuleId::New(rule),
            subject: source.subject,
            subject_name: format!("{}:{line_number}", source.path),
            applicability: Applicability::Supported,
            evidence: EvidenceClass::Derived,
            gate: GateCategory::Blocking,
            summary: format!("`Ordering::{variant}` at {}:{line_number} carries no adjacent `atomic-ordering: allow` reason", source.path),
            locations: vec![format!("{}:{line_number}", source.path)],
        });
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

/// This file's own path. All three rules here exempt their own implementing file, the same
/// self-exemption `rust_text.rs`'s and `security_text.rs`'s own rules carry: every flagged
/// line here is inside this file's own `#[cfg(test)] mod tests { ... }` fixtures, which
/// necessarily spell out real `Ordering::Acquire`/`SeqCst`/`Relaxed` usages to prove the
/// rules catch them. `Is_Test_Or_Example_Source` above does not cover this case because it
/// is a file-path exemption and this file's own path (`checks/concurrency_text.rs`) is not
/// itself a test/example path, even though its content carries a test module.
const OWN_IMPLEMENTATION_FILE: &str = "checks/concurrency_text.rs";

fn Is_Own_Implementation_File(source: &SourceFile) -> bool
{
    return source.path.replace('\\', "/").ends_with(OWN_IMPLEMENTATION_FILE);
}

fn Line_Number(index: usize) -> usize
{
    return index.saturating_add(1);
}

/// The code before any `//` line comment -- this crate's established convention
/// (`rust_text::Code_Prefix`) for staying text-local rather than a full lexer, duplicated here
/// per this crate's per-file helper convention.
fn Code_Prefix(line: &str) -> &str
{
    return line.split("//").next().unwrap_or(line);
}

fn Is_Import_Line(code: &str) -> bool
{
    let trimmed = code.trim_start();
    return trimmed.starts_with("use ") || trimmed.starts_with("pub use ");
}

/// The leftmost `Ordering::<variant>` match in `code`, requiring a word boundary before
/// `Ordering` and after the variant name so a local identifier that merely ends in one of
/// these names (`MyOrdering::SeqCst`) is never mistaken for the real type, and matching
/// `SeqCst` before the others so a line naming it is never misclassified by a shorter
/// alternative appearing first in `ORDERING_VARIANTS`.
fn Ordering_Match_In(code: &str) -> Option<&'static str>
{
    let mut earliest: Option<(usize, &'static str)> = None;

    for &variant in ORDERING_VARIANTS
    {
        if let Some(start) = Find_Ordering_Variant(code, variant)
        {
            if earliest.is_none_or(|(earliest_start, _)| return start < earliest_start)
            {
                earliest = Some((start, variant));
            }
        }
    }

    return earliest.map(|(_, variant)| return variant);
}

fn Find_Ordering_Variant(code: &str, variant: &str) -> Option<usize>
{
    let bytes = code.as_bytes();
    let mut search_from = 0usize;

    while let Some(offset) = code.get(search_from..).and_then(|rest| return rest.find("Ordering"))
    {
        let start = search_from.saturating_add(offset);
        if Has_Left_Boundary(bytes, start) && Match_Qualified_Variant(&code[start.saturating_add("Ordering".len())..], variant)
        {
            return Some(start);
        }
        search_from = start.saturating_add("Ordering".len());
    }

    return None;
}

fn Has_Left_Boundary(bytes: &[u8], start: usize) -> bool
{
    return start.checked_sub(1).and_then(|previous| return bytes.get(previous)).is_none_or(|&byte| return !Is_Ident_Byte(byte));
}

fn Is_Ident_Byte(byte: u8) -> bool
{
    return byte.is_ascii_alphanumeric() || byte == b'_';
}

/// `rest` is everything after `"Ordering"`; this expects optional whitespace, `::`, optional
/// whitespace, the variant name, then a non-identifier character or end of input.
fn Match_Qualified_Variant(rest: &str, variant: &str) -> bool
{
    let after_ws = rest.trim_start();
    let Some(after_colons) = after_ws.strip_prefix("::")
    else
    {
        return false;
    };
    let after_ws_2 = after_colons.trim_start();
    let Some(after_variant) = after_ws_2.strip_prefix(variant)
    else
    {
        return false;
    };
    return after_variant.as_bytes().first().is_none_or(|&byte| return !Is_Ident_Byte(byte));
}

/// The current line's trailing comment, or a contiguous run of blank/comment/attribute lines
/// walking upward from it, carries the literal `atomic-ordering: allow` marker with a non-empty
/// trailing reason -- code-standards' own `ruststyle.Has_Reason` shape, ported rather than
/// relaxed to "any adjacent comment" the way `Check_Every_Allow_Carries_A_Justification` reads
/// it, because the implementation this crate follows checks for this literal marker text and
/// nothing weaker.
fn Has_Marker_Reason(lines: &[&str], index: usize) -> bool
{
    if lines.get(index).is_some_and(|line| return Marker_Reason_In(line).is_some())
    {
        return true;
    }

    let mut cursor = index;
    while cursor > 0
    {
        cursor = cursor.saturating_sub(1);
        let Some(line) = lines.get(cursor)
        else
        {
            break;
        };

        if let Some(reason) = Marker_Reason_In(line)
        {
            return !reason.is_empty();
        }

        if !Is_Skippable_Above(line)
        {
            break;
        }
    }

    return false;
}

fn Is_Skippable_Above(line: &str) -> bool
{
    let trimmed = line.trim();
    return trimmed.is_empty()
        || trimmed.starts_with("//")
        || trimmed.starts_with("/*")
        || trimmed.starts_with('*')
        || trimmed.starts_with("#[")
        || trimmed.starts_with("#![");
}

/// The marker must lead a `//` comment -- found immediately after `//` and optional whitespace,
/// never merely present anywhere on the line, so a string literal or unrelated trailing prose
/// spelling the marker's words cannot silence a real finding.
fn Marker_Reason_In(line: &str) -> Option<&str>
{
    let comment = line.split_once("//").map(|(_, after)| return after)?;
    let after_marker = comment.trim_start().strip_prefix(ATOMIC_ORDERING_MARKER)?;
    let reason = after_marker.trim_start().strip_prefix(':').unwrap_or(after_marker);
    return Some(reason.trim());
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::SubjectId;
    use nomos_model::Content_Digest;

    #[test]
    fn Test_Check_Atomic_Ordering_Choices_Are_Justified_Should_Report_An_Unexplained_Acquire()
    {
        let source = Source("src/counter.rs", "let count = work_count.load(Ordering::Acquire);\n");
        let findings = Check_Atomic_Ordering_Choices_Are_Justified(&[source]);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(ATOMIC_ORDERING_CHOICES_ARE_JUSTIFIED));
    }

    #[test]
    fn Test_Check_Atomic_Ordering_Choices_Are_Justified_Should_Accept_A_Marker_Reason_Above()
    {
        let source = Source(
            "src/counter.rs",
            "// atomic-ordering: allow: pairs with the Release store in submit_work\nlet count = work_count.load(Ordering::Acquire);\n",
        );
        let findings = Check_Atomic_Ordering_Choices_Are_Justified(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Atomic_Ordering_Choices_Are_Justified_Should_Reject_The_Docs_Own_Unmarked_Example()
    {
        let source = Source(
            "src/counter.rs",
            "// Acquire: pairs with the Release store in submit_work; ensures we\n// observe the work-item fields written before the release.\nlet count = work_count.load(Ordering::Acquire);\n",
        );
        let findings = Check_Atomic_Ordering_Choices_Are_Justified(&[source]);
        assert_eq!(findings.len(), 1, "the implementation requires the literal marker, not any adjacent prose: {findings:?}");
    }

    #[test]
    fn Test_Check_Atomic_Ordering_Choices_Are_Justified_Should_Accept_A_Trailing_Same_Line_Marker()
    {
        let source = Source("src/counter.rs", "let count = work_count.load(Ordering::Release); // atomic-ordering: allow: publishes count before the flag\n");
        let findings = Check_Atomic_Ordering_Choices_Are_Justified(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Atomic_Ordering_Choices_Are_Justified_Should_Ignore_An_Import_Line()
    {
        let source = Source("src/counter.rs", "use std::sync::atomic::Ordering::Acquire;\n");
        let findings = Check_Atomic_Ordering_Choices_Are_Justified(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Atomic_Ordering_Choices_Are_Justified_Should_Ignore_Test_Files()
    {
        let source = Source("tests/counter_test.rs", "let count = work_count.load(Ordering::Acquire);\n");
        let findings = Check_Atomic_Ordering_Choices_Are_Justified(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Atomic_Ordering_Choices_Are_Justified_Should_Ignore_The_Sibling_Cmp_Ordering_Enum()
    {
        let source = Source("src/sort.rs", "if a.cmp(&b) == std::cmp::Ordering::Less { return; }\n");
        let findings = Check_Atomic_Ordering_Choices_Are_Justified(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Seqcst_Justified_Explicitly_Should_Report_An_Unexplained_Seqcst()
    {
        let source = Source("src/counter.rs", "flag.store(true, Ordering::SeqCst);\n");
        let findings = Check_Seqcst_Justified_Explicitly(&[source]);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(SEQCST_JUSTIFIED_EXPLICITLY));
    }

    #[test]
    fn Test_Check_Seqcst_Justified_Explicitly_Should_Not_Also_Fire_The_Choices_Are_Justified_Rule()
    {
        let source = Source("src/counter.rs", "flag.store(true, Ordering::SeqCst);\n");
        let findings = Check_Atomic_Ordering_Choices_Are_Justified(&[source]);
        assert!(findings.is_empty(), "the three rules partition the argument -- SeqCst belongs only to its own rule: {findings:?}");
    }

    #[test]
    fn Test_Check_Relaxed_Not_Used_When_Ordering_Matters_Should_Report_An_Unexplained_Relaxed()
    {
        let source = Source("src/counter.rs", "counter.fetch_add(1, Ordering::Relaxed);\n");
        let findings = Check_Relaxed_Not_Used_When_Ordering_Matters(&[source]);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(RELAXED_NOT_USED_WHEN_ORDERING_MATTERS));
    }

    #[test]
    fn Test_Check_Relaxed_Not_Used_When_Ordering_Matters_Should_Accept_A_Marker_Reason()
    {
        let source = Source("src/counter.rs", "counter.fetch_add(1, Ordering::Relaxed); // atomic-ordering: allow: statistics-only, nothing else reads this\n");
        let findings = Check_Relaxed_Not_Used_When_Ordering_Matters(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Findings_Should_Report_Only_The_Leftmost_Ordering_On_A_Compare_Exchange_Line()
    {
        let source = Source("src/counter.rs", "state.compare_exchange(old, new, Ordering::AcqRel, Ordering::Acquire).ok();\n");
        let choices = Check_Atomic_Ordering_Choices_Are_Justified(&[source]);
        assert_eq!(choices.len(), 1, "one decision, one finding: {choices:?}");
    }

    #[test]
    fn Test_Check_Seqcst_Justified_Explicitly_Should_Not_Judge_Its_Own_Implementation_File()
    {
        let source = Source("crates/rules/nomos-rules/src/checks/concurrency_text.rs", "let value = counter.load(Ordering::SeqCst);\n");

        let findings = Check_Seqcst_Justified_Explicitly(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    fn Source(path: &str, text: &str) -> SourceFile
    {
        let mut source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
        source.language = crate::Recognized_Language_In_Tests(path);
        return source;
    }
}
