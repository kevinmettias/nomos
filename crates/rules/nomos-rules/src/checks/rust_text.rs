//! Rust source-text rules from code-standards.
//!
//! These checks deliberately stay conservative and text-local. They import standards whose
//! deciding evidence is visible in one Rust source file: panic primitive spelling, path
//! attributes, shared `Rc`/`Arc` plus `RefCell` ownership escapes, and four rules built on
//! the same "an attribute or construct carries no adjacent explanatory comment" shape —
//! `#[allow(...)]`, `unsafe` constructs, `#[inline(always)]`, and a bare `#[ignore]` with no
//! `= "reason"` value. Three of the four — `#[allow(...)]`, `#[inline(always)]` and the bare
//! `#[ignore]` — reuse [`Previous_Comment_Block_Has`], the same "walk the contiguous comment
//! block immediately above this line" primitive [`Panic_Findings_In`] and [`Shared_Interior_
//! Mutability_Findings_In`] already share — real repeat consumers, not a new abstraction
//! invented for them. `unsafe` is the fourth, and `OD-RULES-021` decided its own two
//! constructs — a bare `unsafe {}` block and an `unsafe fn`/`trait`/`impl` declaration —
//! need two different artifacts rather than one matcher applied uniformly: see
//! [`Has_Local_Safety_Justification`]'s own doc for why it does not reuse the shared
//! primitive either way.

use super::code_prefix::Code_Prefix;
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
/// The code-standards `#[inline(always)]` justification rule id.
pub const INLINE_ALWAYS_JUSTIFICATION: &str = "inline-always-requires-justification";
/// The code-standards disabled-test justification rule id.
pub const A_DISABLED_TEST_STATES_WHY: &str = "a-disabled-test-states-why";

/// Reports `unwrap()` and placeholder `expect(...)` outside test and example Rust sources.
#[must_use]
pub fn Check_Unwrap_Expect_Discipline(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if source.Is_Written_In(RUST_LANGUAGE) && !super::Is_Test_Or_Example_Source(source)
        {
            findings.extend(Unwrap_Expect_Findings_In(source));
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
        let code = Code_Prefix(line);
        Push_Unwrap_Finding(source, &code, index, &mut findings);
        Push_Placeholder_Expect_Finding(source, &code, index, &mut findings);
    }

    return findings;
}

fn Push_Unwrap_Finding(source: &SourceFile, code: &str, index: usize, findings: &mut Vec<Finding>)
{
    if code.contains(".unwrap()")
    {
        let finding = Finding_For_Line(source, UNWRAP_EXPECT_DISCIPLINE, Line_Number(index), "uses `unwrap()` outside tests/examples");
        findings.push(finding);
    }
}

fn Push_Placeholder_Expect_Finding(source: &SourceFile, code: &str, index: usize, findings: &mut Vec<Finding>)
{
    if Placeholder_Expect(code)
    {
        let finding = Finding_For_Line(
            source,
            UNWRAP_EXPECT_DISCIPLINE,
            Line_Number(index),
            "uses `expect(...)` without naming an invariant",
        );
        findings.push(finding);
    }
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

fn Panic_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    return Unjustified_Construct_Findings_In(
        source,
        Rule(PANICS_ARE_JUSTIFIED_DOCUMENTED_AND_VALIDATED),
        Message("uses a panic primitive without a local panic or invariant note"),
        Detector { has_construct: ConstructDetector(Has_Panic_Primitive), has_local_justification: JustificationDetector(Has_Local_Panic_Justification) },
    );
}

/// Reports Rust `#[path = "..."]` attributes whose value is absolute or escapes upward.
#[must_use]
pub fn Check_A_Rust_Path_Stays_Within_Its_Own_Subtree(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if source.Is_Written_In(RUST_LANGUAGE) && !Is_Own_Implementation_File(source)
        {
            findings.extend(Path_Attribute_Findings_In(source));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Path_Attribute_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for (index, line) in source.text.lines().enumerate()
    {
        let code = Code_Prefix(line);
        if let Some(value) = Path_Attribute_Value(&code)
        {
            if Path_Is_Absolute_Or_Escaping(value)
            {
                let finding = Finding_For_Line(
                    source,
                    A_RUST_PATH_STAYS_WITHIN_ITS_OWN_SUBTREE,
                    Line_Number(index),
                    "`#[path]` leaves the declaring file subtree",
                );
                findings.push(finding);
            }
        }
    }

    return findings;
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

    return Relative_Path_Escapes_Its_Own_Subtree(path);
}

/// Walks a relative path's components, tracking how many directories deep it has descended,
/// and reports whether a `..` ever climbs back above the starting point.
fn Relative_Path_Escapes_Its_Own_Subtree(path: &std::path::Path) -> bool
{
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

/// Reports shared `Rc`/`Arc` plus `RefCell` constructs that do not say why.
#[must_use]
pub fn Check_Shared_Interior_Mutability_Says_Why(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if source.Is_Written_In(RUST_LANGUAGE) && !Is_Own_Implementation_File(source)
        {
            findings.extend(Shared_Interior_Mutability_Findings_In(source));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Shared_Interior_Mutability_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    return Unjustified_Construct_Findings_In(
        source,
        Rule(SHARED_INTERIOR_MUTABILITY_SAYS_WHY),
        Message("uses shared interior mutability without `smart-pointer: allow: <reason>`"),
        Detector { has_construct: ConstructDetector(Has_Shared_RefCell_Construct), has_local_justification: JustificationDetector(Has_Local_Smart_Pointer_Reason) },
    );
}

/// Reports `#[allow(...)]`/`#![allow(...)]` attributes with no adjacent explanatory comment.
///
/// A test or example source is not judged, the same exemption
/// [`Check_Unwrap_Expect_Discipline`] above already reads. A test suppresses a lint to
/// construct the shape it is testing -- a deliberately wrong call, an unused binding held to
/// prove a drop -- and the justification the rule asks for is the test's own name. The
/// exemption is deliberately not extended to the sibling rules in this file: `unsafe` and
/// `#[inline(always)]` mean the same thing wherever they are written, and
/// `a-disabled-test-states-why` would be deleted outright by it, since a disabled test is in
/// a test source by construction.
#[must_use]
pub fn Check_Every_Allow_Carries_A_Justification(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        let is_judged_rust_source = source.Is_Written_In(RUST_LANGUAGE)
            && !super::Is_Test_Or_Example_Source(source)
            && !Is_Own_Implementation_File(source);
        if is_judged_rust_source
        {
            findings.extend(Allow_Findings_In(source));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Allow_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    return Unjustified_Construct_Findings_In(
        source,
        Rule(EVERY_ALLOW_CARRIES_A_JUSTIFICATION),
        Message("carries an #[allow(...)] with no adjacent comment explaining why"),
        Detector { has_construct: ConstructDetector(Has_Allow_Attribute), has_local_justification: JustificationDetector(Has_Local_Allow_Justification) },
    );
}

/// Reports an `unsafe {}` block with no adjacent `// SAFETY:` comment carrying a real
/// reason, or an `unsafe fn`/`unsafe trait`/`unsafe impl` declaration with no rustdoc
/// `# Safety` section carrying one — `OD-RULES-021`'s two artifact shapes, matched to
/// whether the construct has a declaration site of its own to document.
#[must_use]
pub fn Check_Unsafe_Justification(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if source.Is_Written_In(RUST_LANGUAGE) && !Is_Own_Implementation_File(source)
        {
            findings.extend(Unsafe_Findings_In(source));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Unsafe_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    return Unjustified_Construct_Findings_In(
        source,
        Rule(UNSAFE_JUSTIFICATION),
        Message("uses `unsafe` without an adjacent `// SAFETY:` comment"),
        Detector { has_construct: ConstructDetector(Has_Unsafe_Construct), has_local_justification: JustificationDetector(Has_Local_Safety_Justification) },
    );
}

/// Reports `#[inline(always)]` attributes with no adjacent explanatory comment.
#[must_use]
pub fn Check_Inline_Always_Justification(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if source.Is_Written_In(RUST_LANGUAGE) && !Is_Own_Implementation_File(source)
        {
            findings.extend(Inline_Always_Findings_In(source));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Inline_Always_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    return Unjustified_Construct_Findings_In(
        source,
        Rule(INLINE_ALWAYS_JUSTIFICATION),
        Message("carries #[inline(always)] with no adjacent comment explaining why"),
        Detector { has_construct: ConstructDetector(Has_Inline_Always_Attribute), has_local_justification: JustificationDetector(Has_Local_Inline_Always_Justification) },
    );
}

/// Reports a bare `#[ignore]` on a Rust test with neither an inline `= "reason"` value nor
/// an adjacent comment explaining why the test does not run.
#[must_use]
pub fn Check_A_Disabled_Test_States_Why(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if source.Is_Written_In(RUST_LANGUAGE) && !Is_Own_Implementation_File(source)
        {
            findings.extend(Disabled_Test_Findings_In(source));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Disabled_Test_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    return Unjustified_Construct_Findings_In(
        source,
        Rule(A_DISABLED_TEST_STATES_WHY),
        Message("disables a test with a bare #[ignore] and no reason"),
        Detector { has_construct: ConstructDetector(Has_Bare_Ignore_Attribute), has_local_justification: JustificationDetector(Has_Local_Ignore_Justification) },
    );
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

fn Has_Shared_RefCell_Construct(code: &str) -> bool
{
    let compact = code
        .chars()
        .filter(|character| return !character.is_whitespace())
        .collect::<String>();

    return Shared_Type_Contains_RefCell(CompactTypeText(&compact), WrapperName("Rc"))
        || Shared_Type_Contains_RefCell(CompactTypeText(&compact), WrapperName("Arc"))
        || compact.contains("Rc::new(RefCell::new(")
        || compact.contains("Arc::new(RefCell::new(")
        || compact.contains("Rc::<RefCell<")
        || compact.contains("Arc::<RefCell<");
}

/// `compact` and `wrapper` are both `&str`; without a distinct type per position, a call
/// site like `Shared_Type_Contains_RefCell(compact, wrapper)` reads as two interchangeable
/// strings and a swap compiles silently.
struct CompactTypeText<'a>(&'a str);
struct WrapperName<'a>(&'a str);

fn Shared_Type_Contains_RefCell(compact: CompactTypeText<'_>, wrapper: WrapperName<'_>) -> bool
{
    let compact = compact.0;
    let pattern = format!("{}<", wrapper.0);
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

fn Has_Inline_Always_Attribute(code: &str) -> bool
{
    return code.contains("#[inline(always)]");
}

fn Has_Local_Inline_Always_Justification(lines: &[&str], index: usize) -> bool
{
    if lines.get(index).is_some_and(|line| return Comment_Is_Non_Empty(line))
    {
        return true;
    }

    return Previous_Comment_Block_Has(lines, index, Comment_Is_Non_Empty);
}

/// A bare `#[ignore]` (or `#[ignore, ...]`) with no `= "reason"` value — the shape
/// `a-disabled-test-states-why` names as the one that needs a local comment instead.
/// `#[ignore = "..."]` already carries its own reason in the attribute itself and is never
/// flagged.
fn Has_Bare_Ignore_Attribute(code: &str) -> bool
{
    let Some(start) = code.find("#[ignore") else { return false };
    let after = code[start.saturating_add("#[ignore".len())..].trim_start();
    return after.starts_with(']') || after.starts_with(',');
}

fn Has_Local_Ignore_Justification(lines: &[&str], index: usize) -> bool
{
    if lines.get(index).is_some_and(|line| return Comment_Is_Non_Empty(line))
    {
        return true;
    }

    return Previous_Comment_Block_Has(lines, index, Comment_Is_Non_Empty);
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

/// `OD-RULES-021`'s dispatch: a declaration (`unsafe fn`/`trait`/`impl`) has its own
/// doc-comment site and is judged by [`Has_Rustdoc_Safety_Section`]; a bare `unsafe {}`
/// block has none and keeps the adjacent-`// SAFETY:`-comment shape every one of this
/// rule's own tests already exercises.
fn Has_Local_Safety_Justification(lines: &[&str], index: usize) -> bool
{
    let code = lines.get(index).map(|line| return Code_Prefix(line)).unwrap_or_default();
    if Is_Unsafe_Declaration(&code)
    {
        return Has_Rustdoc_Safety_Section(lines, index);
    }

    return Has_Safety_Comment_Block(lines, index);
}

/// Whether `code` (the comment-stripped current line) declares `unsafe fn`, `unsafe trait`
/// or `unsafe impl` rather than matching only on a bare `unsafe {}` block — the distinction
/// `OD-RULES-021` measured `Has_Unsafe_Construct` collapsing into one shape.
fn Is_Unsafe_Declaration(code: &str) -> bool
{
    return code.contains("unsafe fn ") || code.contains("unsafe fn(") || code.contains("unsafe impl") || code.contains("unsafe trait");
}

/// A rustdoc `# Safety` heading in the doc-comment block immediately above `index`, with
/// real text on a later line of the same block — `OD-RULES-021`'s second artifact shape,
/// measured directly against `aho-corasick`'s own `packed/ext.rs` and `automaton.rs`, both
/// of which carry exactly this convention on an `unsafe fn`/`unsafe trait` this rule
/// reported as unjustified before this fix.
fn Has_Rustdoc_Safety_Section(lines: &[&str], index: usize) -> bool
{
    let mut seen_heading = false;
    for doc_line in Preceding_Doc_Comment_Block(lines, index)
    {
        if seen_heading
        {
            if !doc_line.is_empty()
            {
                return true;
            }
        }
        else if doc_line.eq_ignore_ascii_case("# safety")
        {
            seen_heading = true;
        }
    }

    return false;
}

/// Every `///` line of the contiguous doc-comment block immediately above `index`, in file
/// order, `///` and surrounding whitespace stripped — tolerant of an intervening attribute
/// line (`#[must_use]`, ...) between the doc block and the declaration it documents, the
/// same way [`Is_Skippable_Block_Line`] already is for a `//` comment block.
fn Preceding_Doc_Comment_Block<'a>(lines: &[&'a str], index: usize) -> Vec<&'a str>
{
    let mut doc_lines = Vec::new();
    let mut cursor = index;

    while let Some(previous) = cursor.checked_sub(1)
    {
        let Some(line) = lines.get(previous)
        else
        {
            break;
        };

        if Is_Skippable_Block_Line(line)
        {
            cursor = previous;
            continue;
        }

        let Some(doc) = line.trim_start().strip_prefix("///")
        else
        {
            break;
        };

        doc_lines.push(doc.trim());
        cursor = previous;
    }

    doc_lines.reverse();
    return doc_lines;
}

/// A `// SAFETY:` block for a bare `unsafe {}` construct, `OD-RULES-021`'s first artifact
/// shape. Checked over the whole contiguous comment block rather than one line, because a
/// real safety comment routinely spells the marker on its own line and the actual reason on
/// a following bullet — `Test_Check_Unsafe_Justification_Should_Accept_A_Safety_Comment`'s
/// own fixture is exactly this shape. A marker with nothing else in its block —
/// `OD-RULES-021`'s vacuous-marker gap, unique to this rule among its siblings:
/// `Has_Local_Allow_Justification`, `Has_Local_Inline_Always_Justification` and
/// `Has_Local_Ignore_Justification` all check [`Comment_Is_Non_Empty`] already — does not
/// satisfy this either.
fn Has_Safety_Comment_Block(lines: &[&str], index: usize) -> bool
{
    let mut block = Preceding_Comment_Block(lines, index);
    if let Some(same_line) = lines.get(index).and_then(|line| return Comment_Text_Of(line))
    {
        block.push(same_line.trim());
    }

    let mut seen_marker = false;
    let mut has_reason = false;
    for comment in block
    {
        let lower = comment.to_ascii_lowercase();
        if let Some(after) = lower.strip_prefix("safety:")
        {
            seen_marker = true;
            if !after.trim().is_empty()
            {
                has_reason = true;
            }
        }
        else if seen_marker && !comment.trim().is_empty()
        {
            has_reason = true;
        }
    }

    return seen_marker && has_reason;
}

/// Every comment line ([`Comment_Text_Of`]) of the contiguous block immediately above
/// `index`, in file order, marker and surrounding whitespace stripped — tolerant of an
/// intervening blank or attribute line the same way [`Is_Skippable_Block_Line`] already is.
fn Preceding_Comment_Block<'a>(lines: &[&'a str], index: usize) -> Vec<&'a str>
{
    let mut comment_lines = Vec::new();
    let mut cursor = index;

    while let Some(previous) = cursor.checked_sub(1)
    {
        let Some(line) = lines.get(previous)
        else
        {
            break;
        };

        if Is_Skippable_Block_Line(line)
        {
            cursor = previous;
            continue;
        }

        let Some(comment) = Comment_Text_Of(line)
        else
        {
            break;
        };

        comment_lines.push(comment.trim());
        cursor = previous;
    }

    comment_lines.reverse();
    return comment_lines;
}

/// `rule` and `message` are both `&str`; without a distinct type per position, a call site
/// like `Unjustified_Construct_Findings_In(source, rule, message, ...)` reads as two
/// interchangeable strings and a swap compiles silently.
struct Rule<'a>(&'a str);
struct Message<'a>(&'a str);

/// The shape [`Panic_Findings_In`], [`Shared_Interior_Mutability_Findings_In`],
/// [`Allow_Findings_In`], [`Unsafe_Findings_In`], [`Inline_Always_Findings_In`] and
/// [`Disabled_Test_Findings_In`] all reduce to: a construct-matching predicate, an
/// unless-locally-justified predicate, and one finding message. This is the one place that
/// shape is written down.
/// Recognizes the construct this rule judges, named so it reads as a collaborator with one
/// documented operation rather than a bare stored callable.
struct ConstructDetector(fn(&str) -> bool);

impl ConstructDetector
{
    fn Detects(&self, code: &str) -> bool
    {
        return (self.0)(code);
    }
}

/// Recognizes a construct's local justification, named for the same reason as
/// [`ConstructDetector`].
struct JustificationDetector(fn(&[&str], usize) -> bool);

impl JustificationDetector
{
    fn Detects(&self, lines: &[&str], index: usize) -> bool
    {
        return (self.0)(lines, index);
    }
}

/// How to recognize the construct this rule judges and how to recognize its local
/// justification, grouped so the six call sites above and this function stay under the
/// parameter-count ceiling.
struct Detector
{
    has_construct: ConstructDetector,
    has_local_justification: JustificationDetector,
}

fn Unjustified_Construct_Findings_In(source: &SourceFile, rule: Rule<'_>, message: Message<'_>, detector: Detector) -> Vec<Finding>
{
    let lines = Lines_Of(source);
    let mut findings = Vec::new();

    for (index, line) in lines.iter().enumerate()
    {
        let code = Code_Prefix(line);
        if detector.has_construct.Detects(&code) && !detector.has_local_justification.Detects(&lines, index)
        {
            let finding = Finding_For_Line(source, rule.0, Line_Number(index), message.0);
            findings.push(finding);
        }
    }

    return findings;
}

/// This file's own path, checked with the same normalized-slash comparison
/// [`super::Is_Test_Or_Example_Source`] already uses. Every rule in this file that reads a
/// construct's own spelling (`unsafe {`, `#[allow(`, `Rc::new(RefCell::new(`, `#[path`)
/// exempts this exact file: its own test fixtures and each rule's own detection-pattern
/// string necessarily spell out the exact syntax the rule looks for, so this is the one
/// file in the workspace guaranteed to look like a violation of every rule it implements,
/// regardless of whether the match is a fixture or the pattern-matching code itself.
/// `#![forbid(unsafe_code)]` at the crate root makes the `unsafe-justification` instance of
/// this provably safe forever; the others are safe today (checked: no real `#[allow(...)]`/
/// `Rc<RefCell<...>>`/escaping `#[path]` usage exists in this file outside its own fixtures
/// and patterns), and trade a theoretical future in-file violation going unflagged for not
/// building the per-line, string-literal-aware self-reference tracking this crate has so
/// far declined to build — the same tradeoff every other path-based exemption here already
/// makes.
const OWN_IMPLEMENTATION_FILE: &str = "crates/rules/nomos-rules/src/checks/rust_text.rs";

fn Is_Own_Implementation_File(source: &SourceFile) -> bool
{
    return source.path.replace('\\', "/") == OWN_IMPLEMENTATION_FILE;
}

fn Lines_Of(source: &SourceFile) -> Vec<&str>
{
    return source.text.lines().collect();
}

fn Line_Number(index: usize) -> usize
{
    return index.saturating_add(1);
}

/// `every-allow-carries-a-justification`'s own example is plain prose with no special
/// marker, unlike the panic and smart-pointer rules' `panic:`/`smart-pointer: allow:`
/// keywords — so any non-empty comment satisfies it.
fn Comment_Is_Non_Empty(line: &str) -> bool
{
    return Comment_Text_Of(line).is_some_and(|comment| return !comment.trim().is_empty());
}

fn Previous_Comment_Block_Has(lines: &[&str], index: usize, predicate: fn(&str) -> bool) -> bool
{
    let mut cursor = index;
    while let Some(previous) = cursor.checked_sub(1)
    {
        if let Some(verdict) = Comment_Block_Step(lines, previous, predicate)
        {
            return verdict;
        }

        cursor = previous;
    }

    return false;
}

/// One backward step through the comment block above a flagged line: `None` means keep
/// walking upward, `Some(verdict)` means the walk has its answer (the block ended, or the
/// predicate matched).
fn Comment_Block_Step(lines: &[&str], previous: usize, predicate: fn(&str) -> bool) -> Option<bool>
{
    let Some(line) = lines.get(previous)
    else
    {
        return Some(false);
    };

    if Is_Skippable_Block_Line(line)
    {
        return None;
    }

    return Comment_Line_Verdict(line, predicate);
}

/// A blank line or a bare attribute (`#[...]`) is not itself a comment, but sits inside the
/// contiguous block the walk is scanning and does not end it.
fn Is_Skippable_Block_Line(line: &str) -> bool
{
    let is_blank = line.trim().is_empty();
    return is_blank || Is_Attribute_Line(line);
}

/// `None` means this comment line did not carry the reason and the walk should keep
/// scanning upward through the rest of the block.
fn Comment_Line_Verdict(line: &str, predicate: fn(&str) -> bool) -> Option<bool>
{
    if !Is_Comment_Line(line)
    {
        return Some(false);
    }

    if predicate(line)
    {
        return Some(true);
    }

    return None;
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

    /// The same unexplained allow the case above reports, moved into a test source. It is a
    /// path exemption, so the content is deliberately identical: only where the file sits
    /// differs.
    #[test]
    fn Test_Check_Every_Allow_Carries_A_Justification_Should_Not_Judge_A_Test_Source()
    {
        let source = Source(
            "crates/languages/nomos-lang-rust/tests/guarantee.rs",
            "#[allow(clippy::redundant_clone)]\nlet processed = input.clone();\n",
        );

        let findings = Check_Every_Allow_Carries_A_Justification(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// The sibling rules in this file deliberately do not carry the exemption, and this is
    /// the case that keeps that deliberate. A disabled test lives in a test source by
    /// construction, so exempting one here would delete the rule rather than narrow it.
    #[test]
    fn Test_Check_A_Disabled_Test_States_Why_Should_Still_Judge_A_Test_Source()
    {
        let source = Source(
            "crates/languages/nomos-lang-rust/tests/guarantee.rs",
            "#[ignore]\nfn Test_Something() {}\n",
        );

        let findings = Check_A_Disabled_Test_States_Why(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
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

    /// `OD-RULES-021`'s real regression: the shape `aho-corasick`'s own `packed/ext.rs` and
    /// `automaton.rs` both carry on a real `unsafe fn`/`unsafe trait`, measured directly and
    /// reported unjustified before this fix.
    #[test]
    fn Test_Check_Unsafe_Justification_Should_Accept_A_Rustdoc_Safety_Section_On_An_Unsafe_Fn()
    {
        let source = Source(
            "src/lib.rs",
            "/// Reads one byte from `ptr`.\n\
             ///\n\
             /// # Safety\n\
             ///\n\
             /// `ptr` must be valid for reads of one byte.\n\
             pub unsafe fn Read_Raw(ptr: *const u8) -> u8\n{\n    return *ptr;\n}\n",
        );

        let findings = Check_Unsafe_Justification(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Unsafe_Justification_Should_Accept_A_Rustdoc_Safety_Section_On_An_Unsafe_Trait()
    {
        let source = Source(
            "src/lib.rs",
            "/// # Safety\n\
             ///\n\
             /// Implementors must uphold the layout invariant.\n\
             pub unsafe trait Packed\n{\n}\n",
        );

        let findings = Check_Unsafe_Justification(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Unsafe_Justification_Should_Report_An_Unsafe_Fn_Whose_Safety_Section_Has_No_Text()
    {
        let source = Source(
            "src/lib.rs",
            "/// # Safety\n\
             pub unsafe fn Read_Raw(ptr: *const u8) -> u8\n{\n    return *ptr;\n}\n",
        );

        let findings = Check_Unsafe_Justification(&[source]);

        assert_eq!(findings.len(), 1, "a bare heading with nothing after it must not satisfy the rule: {findings:?}");
    }

    #[test]
    fn Test_Check_Unsafe_Justification_Should_Not_Accept_An_Unversioned_Comment_On_An_Unsafe_Fn()
    {
        let source = Source(
            "src/lib.rs",
            "// SAFETY: ptr is valid\n\
             pub unsafe fn Read_Raw(ptr: *const u8) -> u8\n{\n    return *ptr;\n}\n",
        );

        let findings = Check_Unsafe_Justification(&[source]);

        assert_eq!(
            findings.len(),
            1,
            "a declaration's own doc-comment site is the artifact this rule asks for, not an adjacent // comment: {findings:?}"
        );
    }

    #[test]
    fn Test_Check_Unsafe_Justification_Should_Report_A_Bare_Safety_Marker_With_No_Reason()
    {
        let source = Source(
            "src/lib.rs",
            "// SAFETY:\n\
             let slice = unsafe { core::slice::from_raw_parts(ptr, len) };\n",
        );

        let findings = Check_Unsafe_Justification(&[source]);

        assert_eq!(findings.len(), 1, "a marker with nothing after it must not satisfy the rule: {findings:?}");
    }

    #[test]
    fn Test_Check_Inline_Always_Justification_Should_Report_An_Unexplained_Inline_Always()
    {
        let source = Source("src/lib.rs", "#[inline(always)]\npub fn Sample_Texel() -> Color { todo!() }\n");

        let findings = Check_Inline_Always_Justification(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(INLINE_ALWAYS_JUSTIFICATION));
    }

    #[test]
    fn Test_Check_Inline_Always_Justification_Should_Accept_An_Explained_Inline_Always()
    {
        let source = Source(
            "src/lib.rs",
            "// hot path, measured 8% improvement in benches/hot_path.rs\n#[inline(always)]\npub fn Sample_Texel() -> Color { todo!() }\n",
        );

        let findings = Check_Inline_Always_Justification(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Disabled_Test_States_Why_Should_Report_A_Bare_Ignore()
    {
        let source = Source("tests/lib.rs", "#[test]\n#[ignore]\nfn Test_Something() {}\n");

        let findings = Check_A_Disabled_Test_States_Why(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(A_DISABLED_TEST_STATES_WHY));
    }

    #[test]
    fn Test_Check_A_Disabled_Test_States_Why_Should_Accept_An_Ignore_With_A_Reason_Value()
    {
        let source = Source(
            "tests/lib.rs",
            "#[test]\n#[ignore = \"needs a GPU adapter; no headless runner has one\"]\nfn Test_Something() {}\n",
        );

        let findings = Check_A_Disabled_Test_States_Why(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Disabled_Test_States_Why_Should_Accept_A_Preceding_Comment()
    {
        let source = Source(
            "tests/lib.rs",
            "#[test]\n// flaky under -race, see #88\n#[ignore]\nfn Test_Something() {}\n",
        );

        let findings = Check_A_Disabled_Test_States_Why(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Unsafe_Justification_Should_Not_Judge_Its_Own_Implementation_File()
    {
        let source = Source("crates/rules/nomos-rules/src/checks/rust_text.rs", "let slice = unsafe { core::slice::from_raw_parts(ptr, len) };\n");

        let findings = Check_Unsafe_Justification(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// `P45-CODE-PREFIX-KNOWS-STRINGS-2`'s real third defect: two files carrying the
    /// identical content, one at this crate's own real path and one at a path that merely
    /// ends the same way, must get identical verdicts — a path-suffix self-exemption a
    /// stranger's repository could reproduce is not a real self-exemption.
    #[test]
    fn Test_Check_Unsafe_Justification_Should_Judge_A_Path_That_Only_Ends_Like_The_Own_Implementation_File()
    {
        let text = "let slice = unsafe { core::slice::from_raw_parts(ptr, len) };\n";
        let real = Source("crates/rules/nomos-rules/src/checks/rust_text.rs", text);
        let spoofed = Source("vendored/crates/rules/nomos-rules/src/checks/rust_text.rs", text);

        let real_findings = Check_Unsafe_Justification(&[real]);
        let spoofed_findings = Check_Unsafe_Justification(&[spoofed]);

        assert!(real_findings.is_empty(), "{real_findings:?}");
        assert_eq!(spoofed_findings.len(), 1, "a suffix match is not this crate's own implementation file: {spoofed_findings:?}");
    }

    fn Source(path: &str, text: &str) -> SourceFile
    {
        let mut source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
        source.language = crate::Recognized_Language_In_Tests(path);
        return source;
    }
}
