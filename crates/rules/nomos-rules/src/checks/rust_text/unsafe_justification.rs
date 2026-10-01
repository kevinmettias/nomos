//! `unsafe-justification`: `OD-RULES-021`'s two artifact shapes.
//!
//! Split out of [`super`], which states the family's shared reasoning. Unlike the sibling
//! rules that accept any adjacent comment, this one dispatches on the construct: a bare
//! `unsafe {}` block keeps the adjacent `// SAFETY:` comment shape, and an
//! `unsafe fn`/`trait`/`impl` declaration -- which has a doc-comment site of its own -- is
//! judged by a rustdoc `# Safety` section instead.

use crate::checks::code_prefix::{Code_Prefix, Code_With_String_Bodies_Masked};
use crate::{RUST_LANGUAGE, SourceFile};
use nomos_contracts::Finding;

use super::comment_block::{Comment_Text_Of, Preceding_Comment_Block, Preceding_Documentation_Comment_Block};
use super::{
    ConstructDetector, Detector, Is_Own_Implementation_File, JustificationDetector, Message, Rule,
    Unjustified_Construct_Findings_In, UNSAFE_JUSTIFICATION,
};

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

/// Matches `unsafe {`, `unsafe fn`, `unsafe impl` and `unsafe trait` specifically — not a
/// bare substring search for `"unsafe"`, which would false-positive on
/// `#![forbid(unsafe_code)]`.
///
/// `P66-UNSAFE-JUSTIFICATION-STRING-BLINDNESS`: matched against
/// [`super::code_prefix::Code_With_String_Bodies_Masked`]'s own output, not `code` as
/// given, so a string literal spelling one of these phrases as data --
/// `("unsafe impl", Self::Implementation)`, a real committed table entry in
/// `nomos-lang-rust-scan`'s own source -- is read as the quoted text it is, not as a real
/// declaration. `code` has already had `Code_Prefix` remove any trailing `//` comment by
/// the time this runs; re-scanning it for quote boundaries is safe and correct, since
/// nothing about where a string opens or closes depends on whether a later comment was
/// already stripped.
fn Has_Unsafe_Construct(code: &str) -> bool
{
    let code = Code_With_String_Bodies_Masked(code);

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
/// `OD-RULES-021` measured `Has_Unsafe_Construct` collapsing into one shape. Matched against
/// the string-masked text for the identical reason [`Has_Unsafe_Construct`] now is.
fn Is_Unsafe_Declaration(code: &str) -> bool
{
    let code = Code_With_String_Bodies_Masked(code);

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
    for doc_line in Preceding_Documentation_Comment_Block(lines, index)
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

/// A `// SAFETY:` block for a bare `unsafe {}` construct, `OD-RULES-021`'s first artifact
/// shape. Checked over the whole contiguous comment block rather than one line, because a
/// real safety comment routinely spells the marker on its own line and the actual reason on
/// a following bullet — `Test_Check_Unsafe_Justification_Should_Accept_A_Safety_Comment`'s
/// own fixture is exactly this shape. A marker with nothing else in its block —
/// `OD-RULES-021`'s vacuous-marker gap, unique to this rule among its siblings:
/// `Has_Local_Allow_Justification`, `Has_Local_Inline_Always_Justification` and
/// `Has_Local_Ignore_Justification` all require a non-empty comment already — does not
/// satisfy this either.
fn Has_Safety_Comment_Block(lines: &[&str], index: usize) -> bool
{
    let block = Safety_Comment_Block(lines, index);
    return Has_A_Safety_Block_Reason(&block);
}

/// Every comment line of the block immediately above `index`, plus one written beside the
/// construct on the construct's own line — the second shape a real safety comment takes.
fn Safety_Comment_Block<'a>(lines: &[&'a str], index: usize) -> Vec<&'a str>
{
    let mut block = Preceding_Comment_Block(lines, index);
    if let Some(same_line) = lines.get(index).and_then(|line| return Comment_Text_Of(line))
    {
        block.push(same_line.trim());
    }

    return block;
}

/// Whether the block names the `SAFETY:` marker and carries a real reason under it: the
/// marker's own line may hold the reason, or any later line of the block may. A marker with
/// nothing after it anywhere in its block is `OD-RULES-021`'s vacuous-marker gap and does not
/// satisfy the rule.
fn Has_A_Safety_Block_Reason(block: &[&str]) -> bool
{
    let mut seen_marker = false;
    let mut has_reason = false;

    for comment in block
    {
        let lower = comment.to_ascii_lowercase();
        if let Some(after) = lower.strip_prefix("safety:")
        {
            seen_marker = true;
            has_reason = has_reason || Is_A_Comment_Line_With_Text(after);
        }
        else if seen_marker && Is_A_Comment_Line_With_Text(comment)
        {
            has_reason = true;
        }
    }

    return seen_marker && has_reason;
}

/// Whether a comment line carries text of its own, rather than only the whitespace a stripped
/// marker can leave behind.
fn Is_A_Comment_Line_With_Text(comment: &str) -> bool
{
    return !comment.trim().is_empty();
}
