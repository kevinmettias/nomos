//! Two lifetime rules, ported from code-standards' `check-lifetime-discipline`
//! (`rules/language-specific/rust/lifetimes/lifetime-discipline`).
//!
//! [`Check_Lifetimes_Follow_The_Descriptive_Naming_Rule`] judges *names*: once a
//! declaration carries two or more distinct explicit lifetimes, a reader has to hold which
//! borrow each one stands for, and single letters give them nothing to hold it by. One
//! lifetime is exempt — where there is nothing to tell apart, `'a` names the only borrow
//! there is and a longer name adds nothing. `'static` and `'_` are excluded from the count
//! in both directions: neither is a name an author chose.
//!
//! [`Check_Static_Bounds_Are_Justified`] judges a *promise*. A `'static` bound says the
//! value outlives the whole program, which is a claim about the shape of the system rather
//! than about one function, and it propagates: every caller inherits it. It is often the
//! right answer — `std::thread::spawn` demands one — and the rule asks for the reason
//! rather than the removal.
//!
//! # A justification is an adjacent comment, not one marker spelling
//!
//! The original accepts exactly one marker. This accepts any adjacent explanatory comment,
//! which is the shape [`super::rust_text::Check_Inline_Always_Justification`] already
//! established in this crate and the direction `P45-SAFETY-JUSTIFICATION-ARTIFACT` names
//! for the unsafe rule, whose complaint is precisely that one demanded spelling rejects the
//! one Rust actually uses.
//!
//! That divergence is load-bearing rather than cosmetic, and it was measured. This
//! workspace has exactly one `'static` bound, in `nomos-platform-std`'s process launcher,
//! and it carries a five-line comment saying `std::thread::spawn` requires it and why no
//! borrow of the caller can reach the thread. The original passes that line only because it
//! is waived in the external tool's own suppression file — a waiver this repository cannot
//! express, since its own suppression policy is `P40-GATE-POLICY-AUTHORING-2` and unbuilt.
//! Ported literally, this rule would report a justified bound as unjustified and there
//! would be nowhere to say otherwise.

use super::code_prefix::{Code_Prefix, Code_With_String_Bodies_Masked};
use crate::{RUST_LANGUAGE, SourceFile};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};
use std::collections::BTreeSet;

/// The code-standards descriptive-lifetime-naming rule id.
pub const LIFETIMES_FOLLOW_THE_DESCRIPTIVE_NAMING_RULE: &str = "lifetimes-follow-the-descriptive-naming-rule";

/// The code-standards static-bound rule id.
pub const STATIC_BOUNDS_ARE_JUSTIFIED: &str = "static-bounds-are-justified";

/// The count at or above which a declaration's lifetimes have something to be told apart
/// from, and so owe descriptive names.
const LIFETIMES_NEEDING_NAMES: usize = 2;

/// The two lifetimes nobody chose: one names the whole program, the other declines to name
/// anything. Neither counts toward [`LIFETIMES_NEEDING_NAMES`] and neither is judged as terse.
const UNCHOSEN_LIFETIMES: [&str; 2] = ["static", "_"];

/// The declaration keywords a lifetime list can sit on. A lifetime appearing anywhere else
/// on a line is a use of one already declared.
const DECLARATION_KEYWORDS: [Keyword<'static>; 6] =
    [Keyword("struct"), Keyword("enum"), Keyword("trait"), Keyword("impl"), Keyword("fn"), Keyword("type")];

/// One of the keywords above, as a value rather than a bare `&str`: [`Names_The_Word`] takes
/// it beside the text it is looked for in, and two bare `&str`s in adjacent positions are
/// transposable at a call site with nothing to catch it.
#[derive(Clone, Copy)]
struct Keyword<'a>(&'a str);

/// An identifier read out of a line, for the same reason: [`Closes_A_Char_Literal`] takes it
/// beside the text it was read out of.
#[derive(Clone, Copy)]
struct Identifier<'a>(&'a str);

/// How far above a bound an explanation may sit and still be its explanation. Matches the
/// window this crate's other justification rules already read.
const JUSTIFICATION_WINDOW_LINES: usize = 3;

/// Reports declarations carrying two or more distinct explicit lifetimes where any of them
/// is a bare single letter.
#[must_use]
pub fn Check_Lifetimes_Follow_The_Descriptive_Naming_Rule(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources.iter().filter(|source| return source.Is_Written_In(RUST_LANGUAGE))
    {
        findings.extend(Terse_Lifetime_Findings_In(source));
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Terse_Lifetime_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for (index, line) in source.text.lines().enumerate()
    {
        let finding = Terse_Lifetime_Finding_For(source, line, index);
        findings.extend(finding);
    }

    return findings;
}

/// The finding `line` owes, if any: a declaration opening with two or more distinct chosen
/// lifetimes, one of them a bare single letter.
fn Terse_Lifetime_Finding_For(source: &SourceFile, line: &str, index: usize) -> Option<Finding>
{
    // `P69-SELF-MATCH-VIA-STRING-LITERALS-FIVE-MORE-RULES`: a string literal's own prose
    // can spell "struct"/"impl" and quote a lifetime by name without either being a real
    // declaration -- masking the literal's body before this scan is what keeps that
    // prose from being read as chosen syntax.
    let code = Code_With_String_Bodies_Masked(&Code_Prefix(line));

    if !Opens_A_Declaration(&code)
    {
        return None;
    }

    let chosen = Chosen_Lifetimes_In(&code);

    if chosen.len() < LIFETIMES_NEEDING_NAMES
    {
        return None;
    }

    let terse = chosen.iter().find(|name| return Is_A_Single_Letter(name))?;
    return Some(Terse_Lifetime_Finding(source, index.saturating_add(1), terse));
}

/// Whether `code` opens a declaration a lifetime list can sit on. The keyword may carry a
/// leading visibility, which is why this looks for the word rather than the line's start.
fn Opens_A_Declaration(code: &str) -> bool
{
    return DECLARATION_KEYWORDS.iter().any(|keyword| return Names_The_Word(code, *keyword));
}

/// Every distinct lifetime name an author chose on this line, without the leading quote and
/// without the two nobody chose.
///
/// A lifetime opens with a quote that is *not* a char literal's. The two are told apart by
/// what follows the name: a char literal closes with another quote, a lifetime does not.
fn Chosen_Lifetimes_In(code: &str) -> BTreeSet<&str>
{
    let mut chosen = BTreeSet::new();
    let mut searched_from = 0usize;

    while let Some(start) = Next_Quote_In(code, searched_from)
    {
        searched_from = start.saturating_add(1);

        if let Some(identifier) = Chosen_Lifetime_At(code, start)
        {
            chosen.insert(identifier.0);
        }
    }

    return chosen;
}

/// The index of the next `'` at or after `searched_from`, or `None` once `code` runs out.
fn Next_Quote_In(code: &str, searched_from: usize) -> Option<usize>
{
    let offset = code.get(searched_from..).and_then(|rest| return rest.find('\''))?;
    return Some(searched_from.saturating_add(offset));
}

/// The lifetime name the quote at `start` opens, or `None` when that quote opens a char
/// literal — or one of the two lifetimes nobody chose — rather than a name an author picked.
fn Chosen_Lifetime_At<'a>(code: &'a str, start: usize) -> Option<Identifier<'a>>
{
    let after_quote = code.get(start.saturating_add(1)..).unwrap_or("");
    let identifier = Leading_Identifier(after_quote)?;

    if Closes_A_Char_Literal(after_quote, identifier) || UNCHOSEN_LIFETIMES.contains(&identifier.0)
    {
        return None;
    }

    return Some(identifier);
}

/// The identifier `code` opens with, or `None` when it opens with anything else. A lone `_`
/// counts, because `'_` is a lifetime this rule has an opinion about excluding.
fn Leading_Identifier(code: &str) -> Option<Identifier<'_>>
{
    let first = code.chars().next()?;

    if !first.is_ascii_alphabetic() && first != '_'
    {
        return None;
    }

    let end = code
        .find(|character: char| return !character.is_ascii_alphanumeric() && character != '_')
        .unwrap_or(code.len());

    return code.get(..end).map(Identifier);
}

/// Whether the quote that opened `after_quote` was a char literal's rather than a
/// lifetime's, told by a closing quote immediately after the name.
fn Closes_A_Char_Literal(after_quote: &str, name: Identifier<'_>) -> bool
{
    return after_quote.get(name.0.len()..).is_some_and(|rest| return rest.starts_with('\''));
}

fn Is_A_Single_Letter(name: &str) -> bool
{
    return name.len() == 1 && name.starts_with(|character: char| return character.is_ascii_lowercase());
}

fn Terse_Lifetime_Finding(source: &SourceFile, line_number: usize, terse: &str) -> Finding
{
    let location = format!("{}:{line_number}", source.path);

    return Finding {
        rule: RuleId::New(LIFETIMES_FOLLOW_THE_DESCRIPTIVE_NAMING_RULE),
        subject: source.subject,
        subject_name: location.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!(
            "{location} declares more than one lifetime and names one of them {terse}, which says nothing about which borrow it stands for; name each for the thing it borrows from"
        ),
        locations: vec![location],
    };
}

/// Reports a `'static` bound with no adjacent comment explaining why the promise is the
/// right one.
#[must_use]
pub fn Check_Static_Bounds_Are_Justified(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources.iter().filter(|source| return source.Is_Written_In(RUST_LANGUAGE))
    {
        findings.extend(Static_Bound_Findings_In(source));
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Static_Bound_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let lines: Vec<&str> = source.text.lines().collect();
    let mut findings = Vec::new();

    for (index, line) in lines.iter().enumerate()
    {
        // Same reason as `Terse_Lifetime_Findings_In`: a string literal's own prose can
        // quote `'static` without that being a real trait bound.
        let code = Code_With_String_Bodies_Masked(&Code_Prefix(line));
        if Bounds_By_Static(&code) && !Has_Adjacent_Explanation(&lines, index)
        {
            let finding = Static_Bound_Finding(source, index.saturating_add(1));
            findings.push(finding);
        }
    }

    return findings;
}

/// Whether `code` bounds something by `'static` — a trait bound after a colon or a plus, or
/// a `where` clause naming it. A `&'static str` is a *reference* to something with that
/// lifetime rather than a bound placed on a caller's type, and is not this rule's subject.
fn Bounds_By_Static(code: &str) -> bool
{
    if Names_The_Word(code, Keyword("where")) && code.contains("'static")
    {
        return true;
    }

    let bound_openers = code
        .char_indices()
        .filter(|(_, character)| return *character == ':' || *character == '+')
        .map(|(offset, _)| return offset);

    for offset in bound_openers
    {
        let after = code.get(offset.saturating_add(1)..).unwrap_or("").trim_start();

        if after.starts_with("'static") && !Continues_An_Identifier(after, "'static".len())
        {
            return true;
        }
    }

    return false;
}

/// Whether any of the few lines above `index` carries a comment with something in it. A bare
/// divider is not an explanation, so the comment must have words after its slashes.
fn Has_Adjacent_Explanation(lines: &[&str], index: usize) -> bool
{
    let first = index.saturating_sub(JUSTIFICATION_WINDOW_LINES);

    return lines
        .get(first..index)
        .unwrap_or(&[])
        .iter()
        .any(|line| return Is_An_Explanatory_Comment(line));
}

fn Is_An_Explanatory_Comment(line: &str) -> bool
{
    let trimmed = line.trim_start();
    let Some(after_slashes) = trimmed.strip_prefix("//")
    else
    {
        return false;
    };

    return after_slashes
        .trim_start_matches(['/', '!', '/'])
        .trim()
        .chars()
        .any(|character| return character.is_ascii_alphanumeric());
}

fn Static_Bound_Finding(source: &SourceFile, line_number: usize) -> Finding
{
    let location = format!("{}:{line_number}", source.path);

    return Finding {
        rule: RuleId::New(STATIC_BOUNDS_ARE_JUSTIFIED),
        subject: source.subject,
        subject_name: location.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!(
            "{location} bounds by the whole program's lifetime with no adjacent comment saying why; the promise propagates to every caller, so say what forces it"
        ),
        locations: vec![location],
    };
}

/// Whether `haystack` contains `word` bounded on both sides by something that is not part of
/// an identifier, so `transform` does not contain `fn` and `structure` does not contain
/// `struct`.
fn Names_The_Word(haystack: &str, word: Keyword<'_>) -> bool
{
    let mut searched_from = 0usize;

    while let Some(offset) = haystack.get(searched_from..).and_then(|rest| return rest.find(word.0))
    {
        let start = searched_from.saturating_add(offset);
        let end = start.saturating_add(word.0.len());
        let before_is_open = start == 0 || !Continues_An_Identifier(haystack, start.saturating_sub(1));
        let after_is_open = !Continues_An_Identifier(haystack, end);

        if before_is_open && after_is_open
        {
            return true;
        }

        searched_from = start.saturating_add(1);
    }

    return false;
}

/// Whether the byte at `offset` is one an identifier can be made of.
fn Continues_An_Identifier(text: &str, offset: usize) -> bool
{
    return text
        .as_bytes()
        .get(offset)
        .is_some_and(|byte| return byte.is_ascii_alphanumeric() || *byte == b'_');
}

/// The code before any line comment. This crate's established per-file convention, which
/// `P45-CODE-PREFIX-KNOWS-STRINGS` will replace with one shared helper.

#[cfg(test)]
mod tests;
