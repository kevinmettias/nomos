//! A Rust enum variant whose payload is positional with two or more members, ported from
//! code-standards' `check-tuple-variant` (shared package `rules/general/style/shared/
//! variantshape`, Rust front end `language-kernels/programming/rust/rustlang/
//! rust_variant_shape.go`).
//!
//! `Point(f32, f32)` type-checks with its two members swapped; the compiler cannot police
//! the order a positional payload carries, the same defect an unnamed tuple return already
//! has. `Point { x: f32, y: f32 }` costs one line and the compiler checks it from then on.
//! A single-member payload (`Some(T)`, `Ok(T)`, any newtype) is exempt — there is no
//! position to get wrong.
//!
//! Three code-standards rule-doc ids all cite this one tool for this one judgment
//! (`named-fields-over-positional-variant-payloads`, `multi-field-variants-use-struct-
//! form`, and a third at a different doc path, `multi-field-enum-variants-use-struct-
//! form`, that the second doc explicitly points to as its own real authority) — real
//! upstream doc drift, not three separate judgments. This module reports under one
//! canonical id, [`NAMED_FIELDS_OVER_POSITIONAL_VARIANT_PAYLOADS`]: the fullest,
//! `MUST`-severity prose actually filed in `check-tuple-variant`'s own doc directory.
//!
//! # The verdict is broader than the doc's own prose
//!
//! `named-fields-over-positional-variant-payloads.md` reads "two or more fields of the
//! *same type*" — but the real `is_Variant_Unambiguous` fires on every payload of two or
//! more members regardless of whether a type repeats; `has_Repeated_Type` only changes the
//! finding's *message*, never the verdict. This module follows the implementation, the
//! same choice this crate already made once before for a stale `enforced_by` claim.
//!
//! # This crate's first whole-file brace-depth scan
//!
//! Every other `*_text.rs` rule here deliberately stays file-level or line-local. This one
//! cannot: telling a tuple variant from a positional function call needs to know which
//! enum body, if any, a line sits inside. `rust_variant_shape.go`'s own doc comment records
//! why the scan tracks depth across the *whole file* rather than assuming an enum's
//! opening brace sits on its header's own line — a first cut made exactly that assumption
//! and silently found zero across a 4,515-file real corpus, because the corpus writes
//! Allman-style braces (the header on one line, the opening brace on the next). The fix,
//! ported here verbatim: remember a pending enum name once its header is seen, and commit
//! the enum open only once an opening brace actually arrives, on that line or any later
//! one. The scan is otherwise exactly as naive as the real tool about a brace hiding
//! inside a string or a comment mid-line — a stated, accepted limit whose failure mode is
//! silence (a missed arm), never a false accusation.
//!
//! # The marker is portable here, unlike `naming-clarity`'s
//!
//! This rule reads raw source text and carries its own real line numbers from the scan
//! itself, so [`Has_Marker_Reason`] — the same same-line-or-immediately-above, non-empty-
//! reason shape `concurrency_text.rs` already established — applies directly. `naming_
//! clarity.rs`'s sibling marker could not be ported for exactly the opposite reason: that
//! rule reads `nomos_cap_syntax::PayloadItem`, which carries no line number at all.

use crate::{RUST_LANGUAGE, SourceFile};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// This crate's chosen canonical rule id among the three code-standards ids that name
/// this judgment.
pub const NAMED_FIELDS_OVER_POSITIONAL_VARIANT_PAYLOADS: &str = "named-fields-over-positional-variant-payloads";

/// The literal marker this family's reason comment must lead with.
const TUPLE_VARIANT_MARKER: &str = "tuple-variant: allow";

/// A member count of one or fewer has no ordering to get wrong.
const SMALLEST_AMBIGUOUS_PAYLOAD: usize = 2;

/// One tuple-form enum variant found by the scan, before the ambiguity filter.
struct Variant
{
    line_index: usize,
    enum_name: String,
    variant_name: String,
    members: Vec<String>,
}

/// Reports every Rust enum variant whose payload is positional with two or more members.
#[must_use]
pub fn Check_Named_Fields_Over_Positional_Variant_Payloads(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if source.Is_Written_In(RUST_LANGUAGE) && !Is_Own_Implementation_File(source)
        {
            findings.extend(Variant_Findings_In(source));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// This file's own path. Every fixture below spells a real `enum`/tuple-variant shape
/// inside a Rust string literal, which would otherwise self-match when this crate checks
/// its own workspace — the same self-exemption every other `*_text.rs`-shaped rule here
/// carries for the identical reason.
const OWN_IMPLEMENTATION_FILE: &str = "checks/enum_shape.rs";

fn Is_Own_Implementation_File(source: &SourceFile) -> bool
{
    return source.path.replace('\\', "/").ends_with(OWN_IMPLEMENTATION_FILE);
}

fn Variant_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let lines: Vec<&str> = source.text.lines().collect();
    let mut findings = Vec::new();

    for variant in Tuple_Variants_In(&lines)
    {
        if let Some(finding) = Variant_Finding(source, &lines, &variant)
        {
            findings.push(finding);
        }
    }

    return findings;
}

/// Threaded across [`Tuple_Variants_In`]'s one-pass scan. `enum_open` carries
/// `(depth_at_open, name)` once a `{` has actually arrived for a pending header; `pending`
/// carries a header seen with no `{` yet.
struct EnumScan
{
    depth: usize,
    enum_open: Option<(usize, String)>,
    pending: Option<String>,
}

/// The whole-file brace-depth scan, ported from `rustVariantScan.consume`.
fn Tuple_Variants_In(lines: &[&str]) -> Vec<Variant>
{
    let mut found = Vec::new();
    let mut scan = EnumScan { depth: 0, enum_open: None, pending: None };

    for (index, line) in lines.iter().enumerate()
    {
        if let Some(variant) = Tuple_Variant_At_Line(line, index, &mut scan)
        {
            found.push(variant);
        }
    }

    return found;
}

/// Judges one line against the accumulated `scan` state, then advances that state past it —
/// a comment or attribute line is judged and skipped without advancing the brace depth at
/// all.
fn Tuple_Variant_At_Line(line: &str, index: usize, scan: &mut EnumScan) -> Option<Variant>
{
    let trimmed = line.trim();
    if trimmed.starts_with("//") || trimmed.starts_with("#[")
    {
        return None;
    }

    let variant = Variant_At_Line(line, index, scan);

    Advance_Enum_Scan(line, scan);

    return variant;
}

/// Either a new pending enum header is looked for (none is open or pending yet), or -- once
/// one is open -- this line is tried as one of its direct-child tuple variants. Exactly one
/// of the two applies to a given line, matching the real tool's own `if`/`else if`.
fn Variant_At_Line(line: &str, index: usize, scan: &mut EnumScan) -> Option<Variant>
{
    if scan.enum_open.is_none() && scan.pending.is_none()
    {
        scan.pending = Enum_Header_Name(line).map(str::to_owned);
        return None;
    }

    let (open_depth, enum_name) = scan.enum_open.as_ref()?;
    if scan.depth != open_depth.saturating_add(1)
    {
        return None;
    }

    let (variant_name, payload) = Tuple_Variant_Match(line.trim())?;
    return Some(Variant {
        line_index: index,
        enum_name: enum_name.clone(),
        variant_name: variant_name.to_owned(),
        members: Variant_Members(payload),
    });
}

/// The name after a word-boundary `enum` followed by mandatory whitespace — `\benum\s+
/// (\w+)` ported as a hand search, this crate's established alternative to a `regex`
/// dependency it has never taken on.
fn Enum_Header_Name(line: &str) -> Option<&str>
{
    let bytes = line.as_bytes();
    let mut search_from = 0usize;

    while let Some(offset) = line.get(search_from..).and_then(|rest| return rest.find("enum"))
    {
        let start = search_from.saturating_add(offset);
        let end = start.saturating_add("enum".len());

        if let Some(name) = Enum_Name_After(line, bytes, start, end)
        {
            return Some(name);
        }

        search_from = start.saturating_add(1);
    }

    return None;
}

/// The identifier right after a candidate `enum` occurrence at `[start, end)`, if `start`
/// sits at a word boundary and at least one whitespace character separates the keyword from
/// a non-empty run of identifier characters.
fn Enum_Name_After<'a>(line: &'a str, bytes: &[u8], start: usize, end: usize) -> Option<&'a str>
{
    if !Has_Left_Boundary(bytes, start)
    {
        return None;
    }

    let after = line.get(end..)?;
    if !after.starts_with(char::is_whitespace)
    {
        return None;
    }

    let trimmed = after.trim_start();
    let name_len = trimmed.find(|character: char| return !(character.is_alphanumeric() || character == '_')).unwrap_or(trimmed.len());
    if name_len == 0
    {
        return None;
    }

    return trimmed.get(..name_len);
}

fn Has_Left_Boundary(bytes: &[u8], start: usize) -> bool
{
    return start.checked_sub(1).and_then(|previous| return bytes.get(previous)).is_none_or(|&byte| return !Is_Ident_Byte(byte));
}

fn Is_Ident_Byte(byte: u8) -> bool
{
    return byte.is_ascii_alphanumeric() || byte == b'_';
}

/// `^\s*(\w+)\s*\(([^)]*)\)\s*,?\s*$` ported as a hand match over an already-trimmed line:
/// a name, a single-line parenthesized payload with no nested unbalanced `)`, and an
/// optional trailing comma. A struct-form variant (`Name { ... }`) and a unit variant
/// (`Name,`) never reach the `(` this requires.
fn Tuple_Variant_Match(trimmed: &str) -> Option<(&str, &str)>
{
    let candidate = trimmed.strip_suffix(',').map_or(trimmed, str::trim_end);
    let without_close = candidate.strip_suffix(')')?;
    let open = without_close.find('(')?;
    let name = without_close.get(..open)?.trim_end();

    if name.is_empty() || !Is_Valid_Identifier(name)
    {
        return None;
    }

    let payload = without_close.get(open.saturating_add(1)..)?;
    if payload.contains(')')
    {
        return None;
    }

    return Some((name, payload));
}

fn Is_Valid_Identifier(name: &str) -> bool
{
    return name.chars().all(Is_Ident_Char);
}

/// Splits a payload at depth zero across `<([`/`>)]` pairs, so a generic member
/// (`HashMap<K, V>`) is one member and not a phantom extra one.
fn Variant_Members(payload: &str) -> Vec<String>
{
    if payload.trim().is_empty()
    {
        return Vec::new();
    }

    return Split_At_Top_Level_Commas(payload);
}

/// Splits `payload` at commas sitting at bracket depth zero, ignoring one nested inside
/// `<([`/`>)]` -- so a generic member (`HashMap<K, V>`) is one member and not a phantom
/// extra one.
fn Split_At_Top_Level_Commas(payload: &str) -> Vec<String>
{
    let mut members = Vec::new();
    let mut depth: i32 = 0;
    let mut start = 0usize;

    for (index, character) in payload.char_indices()
    {
        match character
        {
            '<' | '(' | '[' => depth = depth.saturating_add(1),
            '>' | ')' | ']' => depth = depth.saturating_sub(1),
            ',' if depth == 0 =>
            {
                members.push(payload.get(start..index).unwrap_or_default().trim().to_owned());
                start = index.saturating_add(1);
            }
            _ =>
            {}
        }
    }

    members.push(payload.get(start..).unwrap_or_default().trim().to_owned());
    return members;
}

fn Advance_Enum_Scan(line: &str, scan: &mut EnumScan)
{
    let opened = line.matches('{').count();
    let closed = line.matches('}').count();

    if let Some(name) = scan.pending.take_if(|_| return opened > 0)
    {
        scan.enum_open = Some((scan.depth, name));
    }

    scan.depth = scan.depth.saturating_add(opened);
    scan.depth = scan.depth.saturating_sub(closed);

    if scan.enum_open.as_ref().is_some_and(|(open_depth, _)| return scan.depth <= *open_depth)
    {
        scan.enum_open = None;
    }
}

fn Variant_Finding(source: &SourceFile, lines: &[&str], variant: &Variant) -> Option<Finding>
{
    if variant.members.len() < SMALLEST_AMBIGUOUS_PAYLOAD || Has_Marker_Reason(lines, variant.line_index)
    {
        return None;
    }

    let line_number = Line_Number(variant.line_index);
    let summary = Variant_Summary(source, variant, line_number);

    return Some(Finding {
        rule: RuleId::New(NAMED_FIELDS_OVER_POSITIONAL_VARIANT_PAYLOADS),
        subject: source.subject,
        subject_name: format!("{}:{line_number}", source.path),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary,
        locations: vec![format!("{}:{line_number}", source.path)],
    });
}

/// The current line's trailing comment, or a contiguous run of blank/comment/attribute
/// lines walking upward from it, carries the literal `tuple-variant: allow` marker with a
/// non-empty trailing reason — the same shape `concurrency_text.rs`'s own `Has_Marker_
/// Reason` already established for `atomic-ordering: allow`, duplicated per this crate's
/// per-file convention.
fn Has_Marker_Reason(lines: &[&str], index: usize) -> bool
{
    if lines.get(index).is_some_and(|line| return Marker_Reason_In(line).is_some())
    {
        return true;
    }

    return Marker_Reason_Above(lines, index).unwrap_or(false);
}

/// Walks upward from `index` across a contiguous run of blank/comment/attribute lines,
/// stopping at the first marker line found (its reason decides the verdict, `Some`) or the
/// first line that is none of those (nothing above applies, `None`).
fn Marker_Reason_Above(lines: &[&str], index: usize) -> Option<bool>
{
    let mut cursor = index;
    while cursor > 0
    {
        cursor = cursor.saturating_sub(1);
        let line = lines.get(cursor)?;

        if let Some(reason) = Marker_Reason_In(line)
        {
            return Some(!reason.is_empty());
        }

        if !Is_Skippable_Above(line)
        {
            return None;
        }
    }

    return None;
}

fn Is_Skippable_Above(line: &str) -> bool
{
    let trimmed = line.trim();
    return trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') || trimmed.starts_with("#[");
}

fn Line_Number(index: usize) -> usize
{
    return index.saturating_add(1);
}

fn Variant_Summary(source: &SourceFile, variant: &Variant, line_number: usize) -> String
{
    let site = format!("{}::{}", variant.enum_name, variant.variant_name);
    if Has_Repeated_Type(&variant.members)
    {
        return format!(
            "`{site}` at {}:{line_number} carries an unnamed payload with a repeated type; the compiler cannot tell a correct ordering from a swapped one",
            source.path
        );
    }

    return format!("`{site}` at {}:{line_number} carries an unnamed positional payload; a position is not a name", source.path);
}

/// Whether the same normalized type spelling appears twice — changes only the finding's
/// message, never whether it fires.
fn Has_Repeated_Type(members: &[String]) -> bool
{
    let mut seen: Vec<String> = Vec::new();

    for member in members
    {
        let normalized: String = member.split_whitespace().collect();
        if seen.contains(&normalized)
        {
            return true;
        }
        seen.push(normalized);
    }

    return false;
}

fn Is_Ident_Char(character: char) -> bool
{
    return character.is_alphanumeric() || character == '_';
}

fn Marker_Reason_In(line: &str) -> Option<&str>
{
    let comment = line.split_once("//").map(|(_, after)| return after)?;
    let after_marker = comment.trim_start().strip_prefix(TUPLE_VARIANT_MARKER)?;
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
    fn Test_Check_Named_Fields_Over_Positional_Variant_Payloads_Should_Report_A_Two_Member_Tuple_Variant()
    {
        let source = Source("src/shape.rs", "enum Shape {\n    Point(f32, f32),\n}\n");
        let findings = Check_Named_Fields_Over_Positional_Variant_Payloads(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(NAMED_FIELDS_OVER_POSITIONAL_VARIANT_PAYLOADS));
    }

    #[test]
    fn Test_Check_Named_Fields_Over_Positional_Variant_Payloads_Should_Ignore_A_Single_Member_Newtype_Variant()
    {
        let source = Source("src/shape.rs", "enum Maybe {\n    Some(u32),\n}\n");
        let findings = Check_Named_Fields_Over_Positional_Variant_Payloads(&[source]);

        assert!(findings.is_empty(), "a single member has no ordering to get wrong: {findings:?}");
    }

    #[test]
    fn Test_Check_Named_Fields_Over_Positional_Variant_Payloads_Should_Ignore_A_Struct_Form_Variant()
    {
        let source = Source("src/shape.rs", "enum Shape {\n    Point { x: f32, y: f32 },\n}\n");
        let findings = Check_Named_Fields_Over_Positional_Variant_Payloads(&[source]);

        assert!(findings.is_empty(), "the fields are already named: {findings:?}");
    }

    /// K&R style: the opening brace shares the header's own line, the body on separate
    /// lines below it — distinct from the Allman case right below, where the brace itself
    /// is on its own, later line.
    #[test]
    fn Test_Check_Named_Fields_Over_Positional_Variant_Payloads_Should_Report_Under_A_K_And_R_Brace_Enum()
    {
        let source = Source("src/shape.rs", "enum Shape {\n    Point(f32, f32),\n}\n");
        let findings = Check_Named_Fields_Over_Positional_Variant_Payloads(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    /// The corpus lesson itself: `rustVariantScan`'s own doc comment records a first cut
    /// that assumed the brace sits on the header's own line, and silently found zero
    /// across a real Allman-brace corpus.
    #[test]
    fn Test_Check_Named_Fields_Over_Positional_Variant_Payloads_Should_Report_Under_An_Allman_Brace_Enum()
    {
        let source = Source("src/shape.rs", "enum Shape\n{\n    Point(f32, f32),\n}\n");
        let findings = Check_Named_Fields_Over_Positional_Variant_Payloads(&[source]);

        assert_eq!(findings.len(), 1, "an Allman-style opening brace must still open the enum: {findings:?}");
    }

    #[test]
    fn Test_Check_Named_Fields_Over_Positional_Variant_Payloads_Should_Split_A_Generic_Member_As_One()
    {
        let source = Source("src/shape.rs", "enum Cache {\n    Load(HashMap<K, V>, u32),\n}\n");
        let findings = Check_Named_Fields_Over_Positional_Variant_Payloads(&[source]);

        assert_eq!(findings.len(), 1, "a two-parameter generic plus one more member is a pair, not a phantom triple: {findings:?}");
    }

    #[test]
    fn Test_Check_Named_Fields_Over_Positional_Variant_Payloads_Should_Name_A_Repeated_Type()
    {
        let source = Source("src/shape.rs", "enum Shape {\n    Point(f32, f32),\n}\n");
        let findings = Check_Named_Fields_Over_Positional_Variant_Payloads(&[source]);

        let summary = &findings.first().expect("asserted by an earlier test").summary;
        assert!(summary.contains("repeated type"), "{summary}");
    }

    #[test]
    fn Test_Check_Named_Fields_Over_Positional_Variant_Payloads_Should_Not_Name_A_Repeated_Type_For_Distinct_Members()
    {
        let source = Source("src/shape.rs", "enum Error {\n    Bad(u32, String),\n}\n");
        let findings = Check_Named_Fields_Over_Positional_Variant_Payloads(&[source]);

        let summary = &findings.first().expect("distinct-typed members still fire").summary;
        assert!(!summary.contains("repeated type"), "{summary}");
    }

    #[test]
    fn Test_Check_Named_Fields_Over_Positional_Variant_Payloads_Should_Accept_A_Marker_Reason_Above()
    {
        let source = Source(
            "src/shape.rs",
            "enum Shape {\n    // tuple-variant: allow: kept positional for a stable FFI layout\n    Point(f32, f32),\n}\n",
        );
        let findings = Check_Named_Fields_Over_Positional_Variant_Payloads(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Enum_Header_Name_Should_Require_A_Word_Boundary()
    {
        assert_eq!(Enum_Header_Name("enum Shape"), Some("Shape"));
        assert_eq!(Enum_Header_Name("frozenum Shape"), None, "enum must be a whole word");
    }

    #[test]
    fn Test_Tuple_Variant_Match_Should_Reject_A_Payload_With_A_Nested_Unbalanced_Paren()
    {
        assert_eq!(Tuple_Variant_Match("Bad((u32, String))"), None);
    }

    #[test]
    fn Test_Variant_Members_Should_Return_Nothing_For_An_Empty_Payload()
    {
        assert_eq!(Variant_Members(""), Vec::<String>::new());
    }

    fn Source(path: &str, text: &str) -> SourceFile
    {
        let mut source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
        source.language = crate::Recognized_Language_In_Tests(path);
        return source;
    }
}
