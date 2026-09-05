//! The error-message text hygiene family from code-standards.
//!
//! Three rule ids across two Go tools, both text-decidable against a single Rust source
//! file: `check-error-message` (`lowercase-first-letter`, `no-trailing-punctuation`) judges
//! the content of a derived `Display` message, and `check-context-laziness`
//! (`eager-vs-lazy-context`) judges whether a `.With_Context(...)` call's argument had to be
//! built.
//!
//! # A deliberate narrowing from code-standards' own tool
//!
//! `check-error-message` judges a message wherever it is written: a `#[error("...")]`
//! attribute the `thiserror`-style `derive(Error)` macro turns into `Display`, or a `write!`/
//! `writeln!` call inside a hand-written `impl ... Display for ...` block. Locating the
//! second site soundly needs to know whether a given macro call sits inside a `Display`
//! `impl` block specifically, and not any other block a file happens to contain -- exactly
//! the brace-depth tracking this crate declined to build for `concurrency_text`'s file-level
//! test exemption rather than code-standards' own finer per-block one. Judging every `write!`/
//! `writeln!` in a file regardless of context would convict ordinary logging and file output;
//! judging none of them is the choice this module makes instead, leaving the hand-written-impl
//! site unattempted rather than guessed at. Only the `#[error("...")]` attribute form --
//! syntactically unambiguous on the line that carries it -- is judged here.
//!
//! Both message rules and the context-laziness rule share this crate's established
//! same-line-only convention (`Check_Deprecation_Carries_A_Reason`'s own "arguments close on
//! the same line" limit is the precedent): an attribute or call spanning multiple lines is
//! left unjudged rather than guessed at.

use crate::{RUST_LANGUAGE, SourceFile};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// The code-standards message-starts-lowercase rule id.
pub const LOWERCASE_FIRST_LETTER: &str = "lowercase-first-letter";
/// The code-standards no-trailing-punctuation rule id.
pub const NO_TRAILING_PUNCTUATION: &str = "no-trailing-punctuation";
/// The code-standards eager-vs-lazy-context rule id.
pub const EAGER_VS_LAZY_CONTEXT: &str = "eager-vs-lazy-context";

const ERROR_MESSAGE_MARKER: &str = "error-message: allow";
const CONTEXT_LAZINESS_MARKER: &str = "context-laziness: allow";

/// A first word shorter than this carries no case information worth judging.
const CAPITALIZED_WORD_MINIMUM: usize = 2;

/// The punctuation `no-trailing-punctuation` forbids at the end of a message.
const ERROR_MESSAGE_TERMINALS: &[&str] = &[".", "!", "?"];

const EAGER_CONTEXT_CALL: &str = ".With_Context(";

/// The closed vocabulary of constructors `eager-vs-lazy-context` treats as allocating --
/// code-standards' own list, not a guess about this workspace's helpers.
const CONTEXT_LAZINESS_ALLOCATING: &[&str] = &["format!(", ".to_string(", ".to_owned(", ".to_vec(", ".join(", ".concat(", ".clone("];

/// Reports a `#[error("...")]` message whose first word is an ordinary capitalized word
/// (not an acronym, not a `{}` placeholder) -- a chain walker prefixes its own framing, so a
/// capitalized message reads as `Error: Failed to read` instead of `Error: failed to read`.
#[must_use]
pub fn Check_Error_Message_Starts_Lowercase(sources: &[SourceFile]) -> Vec<Finding>
{
    return Error_Message_Findings(sources, LOWERCASE_FIRST_LETTER, |text| {
        return Is_Starting_Capitalized(text).then_some("this error message starts with a capital letter; chain walkers prefix their own framing, so it should start lowercase".to_owned());
    });
}

/// A first word at least [`CAPITALIZED_WORD_MINIMUM`] characters long, starting with an
/// uppercase letter, with every other character in that word NOT uppercase -- an acronym like
/// `HTTP` or `TOML` has a later uppercase letter and is exempt, and a message opening with a
/// `{}` placeholder is not judged at all since the interpolated value decides its own case.
fn Is_Starting_Capitalized(text: &str) -> bool
{
    if text.starts_with('{')
    {
        return false;
    }

    let word = text.split_whitespace().next().unwrap_or("");
    return Is_Capitalized_Non_Acronym_Word(word);
}

/// A capitalized word that is not itself an acronym: an uppercase first letter with no
/// other uppercase letter behind it -- `HTTP` and `TOML` have one and are exempt.
fn Is_Capitalized_Non_Acronym_Word(word: &str) -> bool
{
    if word.chars().count() < CAPITALIZED_WORD_MINIMUM
    {
        return false;
    }

    let mut chars = word.chars();
    let Some(first) = chars.next()
    else
    {
        return false;
    };

    if !first.is_uppercase()
    {
        return false;
    }

    return !chars.any(char::is_uppercase);
}

/// Reports a `#[error("...")]` message ending in `.`, `!` or `?` -- a chain walker joins
/// messages with its own separator, so a terminal mark produces `failed to read.: No such
/// file`.
#[must_use]
pub fn Check_Error_Message_Has_No_Trailing_Punctuation(sources: &[SourceFile]) -> Vec<Finding>
{
    return Error_Message_Findings(sources, NO_TRAILING_PUNCTUATION, |text| {
        return Trailing_Punctuation(text).map(|mark| return format!("this error message ends with `{mark}`; a chain walker joins messages with its own separator, so drop the trailing punctuation"));
    });
}

fn Trailing_Punctuation(text: &str) -> Option<&'static str>
{
    let trimmed = text.trim_end_matches(' ');
    return ERROR_MESSAGE_TERMINALS.iter().find(|&&mark| return trimmed.ends_with(mark)).copied();
}

/// Reports a same-line `.With_Context(...)` call whose argument builds its value through one
/// of a closed set of allocating constructors -- the cost is paid on every call, including
/// the successful ones, to describe a failure that did not happen.
#[must_use]
pub fn Check_Eager_Vs_Lazy_Context(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if source.Is_Written_In(RUST_LANGUAGE) && !Is_Own_Implementation_File(source)
        {
            findings.extend(Context_Laziness_Findings_In(source));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Context_Laziness_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let lines: Vec<&str> = source.text.lines().collect();
    let mut findings = Vec::new();

    for (index, line) in lines.iter().enumerate()
    {
        if let Some(finding) = Context_Laziness_Finding_At(source, &lines, index, line)
        {
            findings.push(finding);
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Context_Laziness_Finding_At(source: &SourceFile, lines: &[&str], index: usize, line: &str) -> Option<Finding>
{
    let code = Code_Prefix(line);
    let call_start = code.find(EAGER_CONTEXT_CALL)?;

    let argument = code.get(call_start.saturating_add(EAGER_CONTEXT_CALL.len())..).unwrap_or("");
    let constructor = CONTEXT_LAZINESS_ALLOCATING.iter().find(|&&pattern| return argument.contains(pattern))?;

    if Has_Marker_Reason(lines, index, CONTEXT_LAZINESS_MARKER)
    {
        return None;
    }

    let line_number = Line_Number(index);
    let summary = format!(
        "this `With_Context` builds its argument with `{constructor}`, which runs on every call including the successful ones; use `With_Context_Lazy(|| ...)` instead"
    );
    return Some(Finding_At(source, EAGER_VS_LAZY_CONTEXT, line_number, &summary));
}

fn Code_Prefix(line: &str) -> &str
{
    return line.split("//").next().unwrap_or(line);
}

fn Error_Message_Findings(sources: &[SourceFile], rule: &str, judge: impl Fn(&str) -> Option<String>) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if source.Is_Written_In(RUST_LANGUAGE) && !Is_Own_Implementation_File(source)
        {
            let source_findings = Error_Attribute_Findings_In(source, rule, &judge);
            findings.extend(source_findings);
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Error_Attribute_Findings_In(source: &SourceFile, rule: &str, judge: &impl Fn(&str) -> Option<String>) -> Vec<Finding>
{
    let lines: Vec<&str> = source.text.lines().collect();
    let mut findings = Vec::new();

    for (index, line) in lines.iter().enumerate()
    {
        let Some((text, line_number)) = Judgeable_Error_Message_At(&lines, index, line)
        else
        {
            continue;
        };

        let Some(reason) = judge(text)
        else
        {
            continue;
        };

        let finding = Finding_At(source, rule, line_number, &reason);
        findings.push(finding);
    }

    return findings;
}

/// This line's `#[error("...")]` message and line number, if it has one worth judging: a
/// non-empty message with no `error-message: allow` marker in scope.
fn Judgeable_Error_Message_At<'a>(lines: &[&'a str], index: usize, line: &'a str) -> Option<(&'a str, usize)>
{
    let text = Error_Attribute_Message(line)?;
    if text.is_empty() || Has_Marker_Reason(lines, index, ERROR_MESSAGE_MARKER)
    {
        return None;
    }

    return Some((text, Line_Number(index)));
}

fn Finding_At(source: &SourceFile, rule: &str, line_number: usize, summary: &str) -> Finding
{
    return Finding {
        rule: RuleId::New(rule),
        subject: source.subject,
        subject_name: format!("{}:{line_number}", source.path),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!("{} line {line_number}: {summary}", source.path),
        locations: vec![format!("{}:{line_number}", source.path)],
    };
}

/// This file's own path. All three rules here exempt their own implementing file, the same
/// self-exemption `rust_text.rs`, `security_text.rs` and `concurrency_text.rs` each carry:
/// this file's own finding-message text and detection code (`#[error("...")]`,
/// `.With_Context(`) and its `#[cfg(test)] mod tests { ... }` fixtures necessarily spell out
/// the exact shapes each rule looks for.
const OWN_IMPLEMENTATION_FILE: &str = "checks/error_text.rs";

fn Is_Own_Implementation_File(source: &SourceFile) -> bool
{
    return source.path.replace('\\', "/").ends_with(OWN_IMPLEMENTATION_FILE);
}

fn Line_Number(index: usize) -> usize
{
    return index.saturating_add(1);
}

/// The content of a same-line `#[error("...")]` attribute, or `None` when the line carries
/// no such attribute. Mirrors `rust_text::Path_Attribute_Value`'s "first quote, second quote"
/// shape: escaped quotes inside the message and attributes spanning multiple lines are both
/// outside what this reads, the same simplification that helper already makes.
fn Error_Attribute_Message(line: &str) -> Option<&str>
{
    let attribute_start = line.find("#[error(")?;
    let after_attribute = line.get(attribute_start..)?;
    let first_quote = after_attribute.find('"')?;
    let after_first_quote = after_attribute.get(first_quote.saturating_add(1)..)?;
    let second_quote = after_first_quote.find('"')?;
    return after_first_quote.get(..second_quote);
}

/// The current line's trailing comment, or a contiguous run of blank/comment/attribute lines
/// walking upward from it, carries `marker` immediately after `//` with a non-empty trailing
/// reason. The same shape `concurrency_text::Has_Marker_Reason` already established,
/// duplicated per this crate's per-file helper convention and parameterized by marker text
/// since two distinct literal markers are judged in this file.
fn Has_Marker_Reason(lines: &[&str], index: usize, marker: &str) -> bool
{
    if lines.get(index).is_some_and(|line| return Marker_Reason_In(line, marker).is_some())
    {
        return true;
    }

    return Marker_Reason_Above(lines, index, marker).unwrap_or(false);
}

/// Walks upward from `index` across a contiguous run of blank/comment/attribute lines,
/// stopping at the first marker line found (its reason decides the verdict, `Some`) or the
/// first line that is none of those (nothing above applies, `None`).
fn Marker_Reason_Above(lines: &[&str], index: usize, marker: &str) -> Option<bool>
{
    let mut cursor = index;
    while cursor > 0
    {
        cursor = cursor.saturating_sub(1);
        let line = lines.get(cursor)?;

        if let Some(reason) = Marker_Reason_In(line, marker)
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
    return trimmed.is_empty()
        || trimmed.starts_with("//")
        || trimmed.starts_with("/*")
        || trimmed.starts_with('*')
        || trimmed.starts_with("#[")
        || trimmed.starts_with("#![");
}

fn Marker_Reason_In<'a>(line: &'a str, marker: &str) -> Option<&'a str>
{
    let comment = line.split_once("//").map(|(_, after)| return after)?;
    let after_marker = comment.trim_start().strip_prefix(marker)?;
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
    fn Test_Check_Error_Message_Starts_Lowercase_Should_Report_A_Capitalized_Message()
    {
        let source = Source("src/error.rs", "#[error(\"Failed to read scene file\")]\n");
        let findings = Check_Error_Message_Starts_Lowercase(&[source]);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(LOWERCASE_FIRST_LETTER));
    }

    #[test]
    fn Test_Check_Error_Message_Starts_Lowercase_Should_Accept_A_Lowercase_Message()
    {
        let source = Source("src/error.rs", "#[error(\"failed to read scene file\")]\n");
        let findings = Check_Error_Message_Starts_Lowercase(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Error_Message_Starts_Lowercase_Should_Exempt_An_Acronym_First_Word()
    {
        let source = Source("src/error.rs", "#[error(\"HTTP request failed\")]\n");
        let findings = Check_Error_Message_Starts_Lowercase(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Error_Message_Starts_Lowercase_Should_Exempt_A_Placeholder_Opener()
    {
        let source = Source("src/error.rs", "#[error(\"{0}: could not be opened\")]\n");
        let findings = Check_Error_Message_Starts_Lowercase(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Error_Message_Starts_Lowercase_Should_Accept_A_Marker_Reason()
    {
        let source = Source("src/error.rs", "#[error(\"Failed to read scene file\")] // error-message: allow: matches an external API's own error text verbatim\n");
        let findings = Check_Error_Message_Starts_Lowercase(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Error_Message_Has_No_Trailing_Punctuation_Should_Report_A_Trailing_Period()
    {
        let source = Source("src/error.rs", "#[error(\"failed to read scene file.\")]\n");
        let findings = Check_Error_Message_Has_No_Trailing_Punctuation(&[source]);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(NO_TRAILING_PUNCTUATION));
    }

    #[test]
    fn Test_Check_Error_Message_Has_No_Trailing_Punctuation_Should_Accept_A_Clean_Message()
    {
        let source = Source("src/error.rs", "#[error(\"failed to read scene file\")]\n");
        let findings = Check_Error_Message_Has_No_Trailing_Punctuation(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Error_Message_Has_No_Trailing_Punctuation_Should_Report_A_Trailing_Question_Mark()
    {
        let source = Source("src/error.rs", "#[error(\"is the file missing?\")]\n");
        let findings = Check_Error_Message_Has_No_Trailing_Punctuation(&[source]);
        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_Eager_Vs_Lazy_Context_Should_Report_A_Format_Macro_Argument()
    {
        let source = Source("src/read.rs", "Operation().With_Context(format!(\"processing entity {}\", entity_id))?;\n");
        let findings = Check_Eager_Vs_Lazy_Context(&[source]);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(EAGER_VS_LAZY_CONTEXT));
    }

    #[test]
    fn Test_Check_Eager_Vs_Lazy_Context_Should_Accept_The_Lazy_Form()
    {
        let source = Source("src/read.rs", "Operation().With_Context_Lazy(|| format!(\"processing entity {}\", entity_id))?;\n");
        let findings = Check_Eager_Vs_Lazy_Context(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Eager_Vs_Lazy_Context_Should_Accept_A_Cheap_Static_Argument()
    {
        let source = Source("src/read.rs", "Operation().With_Context(\"reading the scene file\")?;\n");
        let findings = Check_Eager_Vs_Lazy_Context(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Eager_Vs_Lazy_Context_Should_Report_A_Clone_Call()
    {
        let source = Source("src/read.rs", "Operation().With_Context(context.clone())?;\n");
        let findings = Check_Eager_Vs_Lazy_Context(&[source]);
        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_Eager_Vs_Lazy_Context_Should_Accept_A_Marker_Reason()
    {
        let source = Source(
            "src/read.rs",
            "Operation().With_Context(format!(\"processing entity {}\", entity_id))?; // context-laziness: allow: measured, allocation cost is negligible next to the I/O this wraps\n",
        );
        let findings = Check_Eager_Vs_Lazy_Context(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Error_Message_Starts_Lowercase_Should_Not_Judge_Its_Own_Implementation_File()
    {
        let source = Source(
            "crates/rules/nomos-rules/src/checks/error_text.rs",
            "#[error(\"Failed to read the file\")]\nstruct ReadError;\n",
        );

        let findings = Check_Error_Message_Starts_Lowercase(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    fn Source(path: &str, text: &str) -> SourceFile
    {
        let mut source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
        source.language = crate::Recognized_Language_In_Tests(path);
        return source;
    }
}
