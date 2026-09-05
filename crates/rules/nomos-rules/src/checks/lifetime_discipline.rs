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
const DECLARATION_KEYWORDS: [&str; 6] = ["struct", "enum", "trait", "impl", "fn", "type"];

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

fn Terse_Lifetime_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for (index, line) in source.text.lines().enumerate()
    {
        let code = Code_Prefix(line);

        if !Opens_A_Declaration(code)
        {
            continue;
        }

        let chosen = Chosen_Lifetimes_In(code);

        if chosen.len() < LIFETIMES_NEEDING_NAMES
        {
            continue;
        }

        if let Some(terse) = chosen.iter().find(|name| return Is_A_Single_Letter(name))
        {
            findings.push(Terse_Lifetime_Finding(source, index.saturating_add(1), terse));
        }
    }

    return findings;
}

fn Static_Bound_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let lines: Vec<&str> = source.text.lines().collect();
    let mut findings = Vec::new();

    for (index, line) in lines.iter().enumerate()
    {
        if Bounds_By_Static(Code_Prefix(line)) && !Has_Adjacent_Explanation(&lines, index)
        {
            findings.push(Static_Bound_Finding(source, index.saturating_add(1)));
        }
    }

    return findings;
}

/// Whether `code` opens a declaration a lifetime list can sit on. The keyword may carry a
/// leading visibility, which is why this looks for the word rather than the line's start.
fn Opens_A_Declaration(code: &str) -> bool
{
    return DECLARATION_KEYWORDS.iter().any(|keyword| return Names_The_Word(code, keyword));
}

/// Whether `haystack` contains `word` bounded on both sides by something that is not part of
/// an identifier, so `transform` does not contain `fn` and `structure` does not contain
/// `struct`.
fn Names_The_Word(haystack: &str, word: &str) -> bool
{
    let mut searched_from = 0usize;

    while let Some(offset) = haystack.get(searched_from..).and_then(|rest| return rest.find(word))
    {
        let start = searched_from.saturating_add(offset);
        let end = start.saturating_add(word.len());
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

/// Every distinct lifetime name an author chose on this line, without the leading quote and
/// without the two nobody chose.
///
/// A lifetime opens with a quote that is *not* a char literal's. The two are told apart by
/// what follows the name: a char literal closes with another quote, a lifetime does not.
fn Chosen_Lifetimes_In(code: &str) -> BTreeSet<&str>
{
    let mut chosen = BTreeSet::new();
    let mut searched_from = 0usize;

    while let Some(offset) = code.get(searched_from..).and_then(|rest| return rest.find('\''))
    {
        let start = searched_from.saturating_add(offset);
        let after_quote = code.get(start.saturating_add(1)..).unwrap_or("");
        let name = Leading_Identifier(after_quote);
        searched_from = start.saturating_add(1);

        let Some(name) = name
        else
        {
            continue;
        };

        if Closes_A_Char_Literal(after_quote, name) || UNCHOSEN_LIFETIMES.contains(&name)
        {
            continue;
        }

        chosen.insert(name);
    }

    return chosen;
}

/// Whether the quote that opened `after_quote` was a char literal's rather than a
/// lifetime's, told by a closing quote immediately after the name.
fn Closes_A_Char_Literal(after_quote: &str, name: &str) -> bool
{
    return after_quote.get(name.len()..).is_some_and(|rest| return rest.starts_with('\''));
}

/// The identifier `code` opens with, or `None` when it opens with anything else. A lone `_`
/// counts, because `'_` is a lifetime this rule has an opinion about excluding.
fn Leading_Identifier(code: &str) -> Option<&str>
{
    let first = code.chars().next()?;

    if !first.is_ascii_alphabetic() && first != '_'
    {
        return None;
    }

    let end = code
        .find(|character: char| return !character.is_ascii_alphanumeric() && character != '_')
        .unwrap_or(code.len());

    return code.get(..end);
}

fn Is_A_Single_Letter(name: &str) -> bool
{
    return name.len() == 1 && name.starts_with(|character: char| return character.is_ascii_lowercase());
}

/// Whether `code` bounds something by `'static` — a trait bound after a colon or a plus, or
/// a `where` clause naming it. A `&'static str` is a *reference* to something with that
/// lifetime rather than a bound placed on a caller's type, and is not this rule's subject.
fn Bounds_By_Static(code: &str) -> bool
{
    if Names_The_Word(code, "where") && code.contains("'static")
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

/// The code before any line comment. This crate's established per-file convention, which
/// `P45-CODE-PREFIX-KNOWS-STRINGS` will replace with one shared helper.
fn Code_Prefix(line: &str) -> &str
{
    return line.split("//").next().unwrap_or(line);
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::SubjectId;
    use nomos_model::Content_Digest;

    #[test]
    fn Test_Check_Lifetimes_Follow_The_Descriptive_Naming_Rule_Should_Report_Two_Single_Letter_Lifetimes()
    {
        let sources = vec![Source("demo/src/a.rs", &Declaration("struct", &[&Lifetime("a"), &Lifetime("b")]))];

        let findings = Check_Lifetimes_Follow_The_Descriptive_Naming_Rule(&sources);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.rule, RuleId::New(LIFETIMES_FOLLOW_THE_DESCRIPTIVE_NAMING_RULE));
        assert_eq!(found.gate, GateCategory::Blocking);
    }

    /// One lifetime has nothing to be told apart from, which is the whole reason the count
    /// gate exists.
    #[test]
    fn Test_Check_Lifetimes_Follow_The_Descriptive_Naming_Rule_Should_Accept_A_Single_Lifetime()
    {
        let only = Lifetime("a");
        let text = format!("fn Split<{only}>(rest: &{only} str) -> (&{only} str, &{only} str)");
        let sources = vec![Source("demo/src/a.rs", &text)];

        let findings = Check_Lifetimes_Follow_The_Descriptive_Naming_Rule(&sources);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Lifetimes_Follow_The_Descriptive_Naming_Rule_Should_Accept_Descriptive_Names()
    {
        let sources = vec![Source("demo/src/a.rs", &Declaration("struct", &[&Lifetime("arena"), &Lifetime("frame")]))];

        let findings = Check_Lifetimes_Follow_The_Descriptive_Naming_Rule(&sources);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// Neither of these is a name an author chose, so a pair of them is not a pair to tell
    /// apart. This is the shape most of this workspace's own signatures already carry.
    #[test]
    fn Test_Check_Lifetimes_Follow_The_Descriptive_Naming_Rule_Should_Not_Count_The_Unchosen_Lifetimes()
    {
        let anonymous = Lifetime("_");
        let text = format!("fn Read(reader: &mut Reader<{anonymous}, {anonymous}>) -> &{} str", Lifetime("static"));
        let sources = vec![Source("demo/src/a.rs", &text)];

        let findings = Check_Lifetimes_Follow_The_Descriptive_Naming_Rule(&sources);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// A char literal opens with the same character a lifetime does, and this crate has
    /// already paid twice for a scanner that could not tell them apart.
    #[test]
    fn Test_Check_Lifetimes_Follow_The_Descriptive_Naming_Rule_Should_Not_Read_A_Char_Literal_As_A_Lifetime()
    {
        let text = format!(
            "fn Split(line: &str) -> bool {{ return line.contains({}) && line.contains({}); }}",
            Char_Literal('a'),
            Char_Literal('b')
        );
        let sources = vec![Source("demo/src/a.rs", &text)];

        let findings = Check_Lifetimes_Follow_The_Descriptive_Naming_Rule(&sources);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// A lifetime used on a line that opens no declaration was declared somewhere else.
    #[test]
    fn Test_Check_Lifetimes_Follow_The_Descriptive_Naming_Rule_Should_Only_Judge_A_Declaration()
    {
        let text = format!("    left: &{} str,\n    right: &{} str,", Lifetime("a"), Lifetime("b"));
        let sources = vec![Source("demo/src/a.rs", &text)];

        let findings = Check_Lifetimes_Follow_The_Descriptive_Naming_Rule(&sources);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Static_Bounds_Are_Justified_Should_Report_An_Unexplained_Bound()
    {
        let sources = vec![Source("demo/src/a.rs", &Bounded_Function(""))];

        let findings = Check_Static_Bounds_Are_Justified(&sources);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.rule, RuleId::New(STATIC_BOUNDS_ARE_JUSTIFIED));
        assert_eq!(found.gate, GateCategory::Blocking);
    }

    #[test]
    fn Test_Check_Static_Bounds_Are_Justified_Should_Report_A_Bound_Added_With_A_Plus()
    {
        let sources = vec![Source("demo/src/a.rs", &Bounded_Function("Read + Send + "))];

        let findings = Check_Static_Bounds_Are_Justified(&sources);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_Static_Bounds_Are_Justified_Should_Report_A_Where_Clause_Bound()
    {
        let text = format!("    where Source: Read + {}", Lifetime("static"));
        let sources = vec![Source("demo/src/a.rs", &text)];

        let findings = Check_Static_Bounds_Are_Justified(&sources);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    /// The divergence this port is built on. The one real instance in this workspace carries
    /// exactly this shape: prose above the bound, in no particular spelling.
    #[test]
    fn Test_Check_Static_Bounds_Are_Justified_Should_Accept_An_Adjacent_Explanation()
    {
        let text = format!(
            "    // Spawning moves the reader onto a detached thread that outlives this call, so\n    // the standard library demands the bound rather than this signature choosing it.\n{}",
            Bounded_Function("Read + Send + ")
        );
        let sources = vec![Source("demo/src/a.rs", &text)];

        let findings = Check_Static_Bounds_Are_Justified(&sources);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// A divider is not an explanation.
    #[test]
    fn Test_Check_Static_Bounds_Are_Justified_Should_Not_Accept_A_Wordless_Comment()
    {
        let text = format!("    // ----\n{}", Bounded_Function(""));
        let sources = vec![Source("demo/src/a.rs", &text)];

        let findings = Check_Static_Bounds_Are_Justified(&sources);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    /// A reference *to* something living that long is not a bound placed on a caller's type,
    /// and this workspace is full of the former.
    #[test]
    fn Test_Check_Static_Bounds_Are_Justified_Should_Ignore_A_Static_Reference()
    {
        let text = format!("fn Name() -> &{} str", Lifetime("static"));
        let sources = vec![Source("demo/src/a.rs", &text)];

        let findings = Check_Static_Bounds_Are_Justified(&sources);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Static_Bounds_Are_Justified_Should_Ignore_A_Language_It_Does_Not_Judge()
    {
        let sources = vec![Source("demo/src/a.go", &Bounded_Function(""))];

        let findings = Check_Static_Bounds_Are_Justified(&sources);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// The character a lifetime opens with, held once so no fixture below has to spell one.
    ///
    /// Every fixture in this module is built rather than written out, because a rule about
    /// lifetimes whose own tests spell lifetimes is a rule that reports its own test module.
    /// Measured, not assumed: writing them literally produced five findings against this
    /// file on a real run over this workspace. Nine modules in this crate answer the same
    /// hazard with a path-suffix exemption that would exempt any file in any repository
    /// ending the same way, and `P45-CODE-PREFIX-KNOWS-STRINGS` is retiring it, so this
    /// file does not add a tenth.
    const LIFETIME_MARK: char = '\'';

    fn Lifetime(name: &str) -> String
    {
        return format!("{LIFETIME_MARK}{name}");
    }

    fn Char_Literal(held: char) -> String
    {
        return format!("{LIFETIME_MARK}{held}{LIFETIME_MARK}");
    }

    fn Declaration(keyword: &str, lifetimes: &[&String]) -> String
    {
        let named: Vec<&str> = lifetimes.iter().map(|lifetime| return lifetime.as_str()).collect();
        return format!("{keyword} Judged<{}>", named.join(", "));
    }

    fn Bounded_Function(leading_bounds: &str) -> String
    {
        return format!("fn Judged<T: {leading_bounds}{}>(value: T) -> T", Lifetime("static"));
    }

    fn Source(path: &str, text: &str) -> SourceFile
    {
        let mut source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
        source.language = crate::Recognized_Language_In_Tests(path);
        return source;
    }
}

