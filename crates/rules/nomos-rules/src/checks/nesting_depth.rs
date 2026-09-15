//! A function whose control flow nests too deeply, ported from code-standards'
//! `check-nesting-depth` (the check at
//! `rules/general/style/checks/control-flow/readability/check-nesting-depth`, with the
//! judgment itself in the language-free shared package `rules/general/style/shared/nesting`).
//!
//! Deep nesting is the same signal an over-long function is: logic that wants extracting
//! into a helper, or flattening with a guard clause and an early return. By the fourth level
//! a reader is holding three conditions in their head to know whether the line in front of
//! them runs at all, and the conditions are not next to it.
//!
//! # Depth counts constructs, not braces
//!
//! The first `if`, loop or `match` in a body is level one, one directly inside it is level
//! two, and so on. A brace that opens no control flow — a bare block, a struct literal, a
//! closure body, a `match` arm's block — adds no level, which is why this counts constructs
//! and tracks braces only to know when one ends.
//!
//! An `else if` continues at its neighbour's level rather than deepening: it is the second
//! question in one decision, not a decision inside a decision. Its own body still deepens as
//! usual. Both halves of that live here rather than in the scan, exactly as the shared Go
//! package puts them in the judgment rather than in a front end.
//!
//! A function is reported **once**, at the first construct to cross the limit, so a deeply
//! nested body is one finding pointing at the outermost place a guard clause would help
//! rather than one per level.
//!
//! # Why this one is a text rule and `allman-brace-placement` is not
//!
//! The Go original reaches a tree-sitter parse, and that is not on its own a reason this
//! cannot be text: [`super::constant_scope`] and
//! [`super::formatting::Check_No_Single_Line_Function_Bodies`] are both ports of
//! tree-sitter-backed checks, re-decided over text and clean. Whether a given one survives
//! is a per-rule question, and it was answered by prototyping both.
//!
//! It failed for `allman-brace-placement`, which has to separate a *governed* block from
//! every other brace in the language — a prototype in exact agreement with the real tool on
//! fifteen probes still produced thirteen false positives over this workspace, in six
//! structural classes a text scanner cannot resolve. It succeeded here, because counting
//! constructs needs only that a construct be recognized where it opens, and a construct
//! opens a line. The prototype agreed with the real tool on a positive fixture and on
//! control probes covering if/else chains, items, nested blocks, match arms, closures and
//! every governed loop form, and then reported zero over this whole workspace, which is what
//! the real tool reports.
//!
//! # Brace counting reads code, not text
//!
//! A `{` inside a string literal is not a block, and this crate has already paid for a
//! scanner that could not tell the difference. Rather than a fifth private comment stripper,
//! this reads [`super::formatting::Advance_Literal_State`], the cross-line literal-state
//! scanner `formatting.rs` already owns — plain and raw strings, hash counting, escapes, and
//! a literal left open across lines. That is the substrate
//! `P45-CODE-PREFIX-KNOWS-STRINGS` is consolidating on.

use super::formatting::{Advance_Literal_State, RustLiteralState};
use super::structure::Resolve_Limit;
use crate::{RUST_LANGUAGE, SourceFile};
use nomos_analysis::FactReader;
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// The code-standards nesting rule id.
pub const NESTING_DEPTH: &str = "nesting-depth";

/// The deepest acceptable nesting. A function reaching it is fine; one past it is reported.
/// The code-standards value, and the default a repository that declares nothing gets.
const MAX_NESTING_DEPTH: usize = 3;

/// `standards.json`'s row key for the limit, the same `nomos.cap.limits.policy` mechanism
/// the file-size triggers already resolve through — `OD-RULES-011` names this whole family.
const NESTING_DEPTH_MAX_KEY: &str = "nesting-depth-max";

/// The control-flow keywords that open a level.
const CONSTRUCT_KEYWORDS: [ConstructKeyword<'static>; 5] =
    [ConstructKeyword("if"), ConstructKeyword("while"), ConstructKeyword("for"), ConstructKeyword("loop"), ConstructKeyword("match")];

/// One of the keywords above, as a value rather than a bare `&str`: [`Opens_Construct`] takes
/// it beside the line it is looked for in, and two bare `&str`s in adjacent positions are
/// transposable at a call site with nothing to catch it.
#[derive(Clone, Copy)]
struct ConstructKeyword<'a>(&'a str);

/// The modifiers a function declaration may carry before its keyword.
const FUNCTION_MODIFIERS: [&str; 5] = ["pub", "async", "unsafe", "const", "extern"];

/// Reports every Rust function whose control flow nests past the resolved limit.
#[must_use]
pub fn Check_Nesting_Depth(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let limit = Resolve_Limit(facts, None, NESTING_DEPTH_MAX_KEY, MAX_NESTING_DEPTH);
    let mut findings = Vec::new();

    for source in sources.iter().filter(|source| return source.Is_Written_In(RUST_LANGUAGE))
    {
        let found = Deep_Function_Findings_In(source, limit);
        findings.extend(found);
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// One function's breach: where nesting first went too deep, and how deep it got.
struct Breach
{
    line_number: usize,
    depth: usize,
}

/// Where one line sits: its 1-based number, for reporting, and the brace depth it opens at,
/// for knowing which construct a `{` belongs to. Two bare `usize`s in adjacent positions are
/// transposable at a call site; named together they are not.
#[derive(Clone, Copy)]
struct LinePosition
{
    number: usize,
    depth: usize,
}

/// The scan's position inside one function body.
struct FunctionScan
{
    /// The deepest nesting this scan accepts, resolved once for the whole run.
    limit: usize,
    /// The brace depth the body opened at, so the scan knows when the function ends.
    body_depth: usize,
    /// The brace depth each open construct's own block opened at.
    open_constructs: Vec<usize>,
    /// A construct whose keyword has been read and whose `{` has not arrived yet — Allman
    /// puts it on the following line, so a scan that required both together would never fire
    /// on this workspace at all.
    awaiting_block: Option<usize>,
    breach: Option<Breach>,
}

/// One pass over a source's lines, carrying what each line needs from the line before it: the
/// literal left open, the brace depth reached, the function body being scanned, and the
/// findings the walk has produced. The state the loop body would otherwise thread through five
/// locals, named so the walk reads as a walk rather than a tally.
struct NestingWalk<'a>
{
    /// The source whose lines are walked — the subject and path every finding carries.
    source: &'a SourceFile,
    /// The deepest nesting this walk accepts, resolved once for the whole run.
    limit: usize,
    /// The literal state the line before left open.
    literal: RustLiteralState,
    /// The brace depth the line before ended at.
    depth: usize,
    /// The function body being scanned, when the walk is inside one.
    open: Option<FunctionScan>,
    /// Every breach found so far, in line order.
    findings: Vec<Finding>,
}

/// Reports the functions in one source whose nesting passes `limit`.
fn Deep_Function_Findings_In(source: &SourceFile, limit: usize) -> Vec<Finding>
{
    let mut walk = NestingWalk::New(source, limit);

    for (index, line) in source.text.lines().enumerate()
    {
        walk.Step(index, line);
    }
    walk.Judge_Open_Function();

    return walk.findings;
}

impl<'a> NestingWalk<'a>
{
    fn New(source: &'a SourceFile, limit: usize) -> NestingWalk<'a>
    {
        return NestingWalk {
            source,
            limit,
            literal: RustLiteralState::None,
            depth: 0usize,
            open: None,
            findings: Vec::new(),
        };
    }

    /// One line of the walk: close the constructs its braces closed, judge a function it ended,
    /// then either open a scan or read the line's own construct — in that order, because a line
    /// that both ends one body and opens another is the shape a one-line function has.
    fn Step(&mut self, index: usize, line: &str)
    {
        let literal_bytes = Advance_Literal_State(line, &mut self.literal);
        let code = String::from_utf8_lossy(&literal_bytes).into_owned();
        let opened = code.matches('{').count();
        let closed = code.matches('}').count();
        let after_closing = self.depth.saturating_sub(closed);

        self.Close_Constructs_Above(after_closing);
        if self.Is_Past_The_Open_Function(after_closing)
        {
            self.Judge_Open_Function();
        }
        if self.open.is_none() && Opens_A_Function(&code)
        {
            self.open = Some(self.Opened_Scan(opened));
        }
        else if let Some(open) = self.open.as_mut()
        {
            Consider_Line(open, &code, LinePosition { number: index.saturating_add(1), depth: self.depth });
        }
        self.depth = self.depth.saturating_add(opened).saturating_sub(closed);
    }

    /// Drops every construct the closing braces on this line closed.
    fn Close_Constructs_Above(&mut self, after_closing: usize)
    {
        if let Some(open) = self.open.as_mut()
        {
            open.open_constructs.retain(|construct| return *construct <= after_closing);
        }
    }

    /// Whether this line's closing braces ended the function body being scanned.
    fn Is_Past_The_Open_Function(&self, after_closing: usize) -> bool
    {
        return self.open.as_ref().is_some_and(|open| return after_closing < open.body_depth);
    }

    /// Judges the function the walk is inside, if it is inside one at all, and reports its
    /// breach when it has one. A body with no breach is discarded without a finding.
    fn Judge_Open_Function(&mut self)
    {
        let Some(finished) = self.open.take()
        else
        {
            return;
        };
        if let Some(breach) = finished.breach
        {
            let finding = Nesting_Finding(self.source, &breach);
            self.findings.push(finding);
        }
    }

    /// The scan a function declaration opens: a body whose own depth is this line's brace depth
    /// plus the braces the line opened.
    fn Opened_Scan(&self, opened: usize) -> FunctionScan
    {
        return FunctionScan {
            limit: self.limit,
            body_depth: self.depth.saturating_add(opened),
            open_constructs: Vec::new(),
            awaiting_block: None,
            breach: None,
        };
    }
}

/// The one finding a too-deeply-nested function produces.
fn Nesting_Finding(source: &SourceFile, breach: &Breach) -> Finding
{
    let location = format!("{}:{}", source.path, breach.line_number);
    let depth = breach.depth;

    return Finding {
        rule: RuleId::New(NESTING_DEPTH),
        subject: source.subject,
        subject_name: location.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!(
            "{location} nests control flow {depth} levels deep; by this level a reader holds every enclosing condition in their head to know whether the line runs at all -- flatten it with a guard clause and an early return, or extract the inner levels into a named helper"
        ),
        locations: vec![location],
    };
}

/// Whether this line declares a function, past any modifiers it carries.
fn Opens_A_Function(code: &str) -> bool
{
    let mut rest = code.trim_start();

    while let Some(modifier) = FUNCTION_MODIFIERS.iter().find_map(|modifier| return Word_Prefix(rest, modifier))
    {
        let mut after = modifier.trim_start();

        if let Some(after_open) = after.strip_prefix('(')
            && let Some(close) = after_open.find(')')
        {
            after = after_open.get(close.saturating_add(1)..).unwrap_or("");
        }

        rest = after.trim_start();
    }

    return Word_Prefix(rest, "fn").is_some();
}

/// Advances one line of a function body: notes a construct whose keyword opens here, and
/// opens the pending construct's level when its `{` arrives.
fn Consider_Line(scan: &mut FunctionScan, code: &str, position: LinePosition)
{
    if let Some(level) = Level_Opened_By(code, scan.open_constructs.len())
    {
        scan.awaiting_block = Some(level);
        Record_Level(scan, level, position.number);
    }

    if code.contains('{')
    {
        let awaited = scan.awaiting_block.take().is_some();

        if awaited
        {
            scan.open_constructs.push(position.depth.saturating_add(1));
        }
    }
}

/// The nesting level a control-flow construct on this line would occupy, or `None` when the
/// line opens none.
///
/// An `else if` takes its neighbour's level rather than one deeper; a plain `else` opens no
/// construct at all, because it is the other half of the `if` already counted.
fn Level_Opened_By(code: &str, open_constructs: usize) -> Option<usize>
{
    let (rest, continues_a_decision) = Past_Else(Construct_Start(code))?;
    let keyword = CONSTRUCT_KEYWORDS.iter().find(|keyword| return Opens_Construct(rest, **keyword))?;

    if continues_a_decision && keyword.0 == "if"
    {
        return Some(open_constructs.max(1));
    }

    return Some(open_constructs.saturating_add(1));
}

/// The `rest` a construct keyword is read from, and whether this line continues the decision
/// above it (`else`/`else if`) rather than opening one. `None` when the line carries an `else`
/// with something other than an `if` after it, which opens no construct of its own — the `else`
/// belongs to the `if` already counted.
fn Past_Else(rest: &str) -> Option<(&str, bool)>
{
    let Some(after_else) = Word_Prefix(rest, "else")
    else
    {
        return Some((rest, false));
    };

    let following = after_else.trim_start();
    Word_Prefix(following, "if")?;

    return Some((following, true));
}

/// `code` past the closing braces and any loop label that lead it: where a construct's own
/// keyword would appear.
fn Construct_Start(code: &str) -> &str
{
    let mut rest = code.trim_start();

    while let Some(after_brace) = rest.strip_prefix('}')
    {
        rest = after_brace.trim_start();
    }

    return Without_Loop_Label(rest);
}

/// `code` past a loop label, when it carries one. `'outer: for row in rows` is a loop, and a
/// scan that stopped at the quote would miss every labelled one — and a labelled loop is
/// exactly the shape that appears where nesting is already deep.
fn Without_Loop_Label(code: &str) -> &str
{
    let Some(after_quote) = code.strip_prefix('\'')
    else
    {
        return code;
    };
    let name_length = after_quote
        .find(|character: char| return !character.is_ascii_alphanumeric() && character != '_')
        .unwrap_or(after_quote.len());
    let Some(after_name) = after_quote.get(name_length..)
    else
    {
        return code;
    };

    return match after_name.strip_prefix(':')
    {
        Some(after_colon) => after_colon.trim_start(),
        None => code,
    };
}

/// Whether `code` opens `keyword` as a construct rather than as part of a longer name — and,
/// for `for`, not as the higher-ranked `for<'lifetime>` of a `where` clause, which is a
/// binder and not a loop.
fn Opens_Construct(code: &str, keyword: ConstructKeyword<'_>) -> bool
{
    let Some(after_keyword) = code.strip_prefix(keyword.0)
    else
    {
        return false;
    };

    if keyword.0 == "for" && after_keyword.starts_with('<')
    {
        return false;
    }

    return after_keyword.is_empty() || after_keyword.starts_with([' ', '\t', '{']);
}

/// Notes `level` as reached, and remembers the first line to pass the scan's own limit.
fn Record_Level(scan: &mut FunctionScan, level: usize, line_number: usize)
{
    if level > scan.limit && scan.breach.is_none()
    {
        scan.breach = Some(Breach { line_number, depth: level });
    }
}

/// `code` past `word`, when `word` opens it as a whole word rather than as the start of a
/// longer identifier — `format` does not open with `for`.
fn Word_Prefix<'a>(code: &'a str, word: &str) -> Option<&'a str>
{
    let after_word = code.strip_prefix(word)?;

    if after_word.starts_with(|character: char| return character.is_ascii_alphanumeric() || character == '_')
    {
        return None;
    }

    return Some(after_word);
}

#[cfg(test)]
mod tests;
