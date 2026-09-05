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
const CONSTRUCT_KEYWORDS: [&str; 5] = ["if", "while", "for", "loop", "match"];

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
        findings.extend(Deep_Function_Findings_In(source, limit));
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

/// Reports the functions in one source whose nesting passes `limit`.
fn Deep_Function_Findings_In(source: &SourceFile, limit: usize) -> Vec<Finding>
{
    let mut findings = Vec::new();
    let mut literal = RustLiteralState::None;
    let mut depth = 0usize;
    let mut scan: Option<FunctionScan> = None;

    for (index, line) in source.text.lines().enumerate()
    {
        let code = String::from_utf8_lossy(&Advance_Literal_State(line, &mut literal)).into_owned();
        let opened = code.matches('{').count();
        let closed = code.matches('}').count();
        let after_closing = depth.saturating_sub(closed);

        if let Some(open) = scan.as_mut()
        {
            open.open_constructs.retain(|construct| return *construct <= after_closing);
        }

        let function_ended = scan.as_ref().is_some_and(|open| return after_closing < open.body_depth);

        if function_ended
            && let Some(finished) = scan.take()
            && let Some(breach) = finished.breach
        {
            findings.push(Nesting_Finding(source, &breach));
        }

        if scan.is_none() && Opens_A_Function(&code)
        {
            scan = Some(FunctionScan {
                limit,
                body_depth: depth.saturating_add(opened),
                open_constructs: Vec::new(),
                awaiting_block: None,
                breach: None,
            });
        }
        else if let Some(open) = scan.as_mut()
        {
            Consider_Line(open, &code, LinePosition { number: index.saturating_add(1), depth });
        }

        depth = depth.saturating_add(opened).saturating_sub(closed);
    }

    if let Some(finished) = scan
        && let Some(breach) = finished.breach
    {
        findings.push(Nesting_Finding(source, &breach));
    }

    return findings;
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

    if code.contains('{') && scan.awaiting_block.take().is_some()
    {
        scan.open_constructs.push(position.depth.saturating_add(1));
    }
}

/// Notes `level` as reached, and remembers the first line to pass the scan's own limit.
fn Record_Level(scan: &mut FunctionScan, level: usize, line_number: usize)
{
    if level > scan.limit && scan.breach.is_none()
    {
        scan.breach = Some(Breach { line_number, depth: level });
    }
}

/// The nesting level a control-flow construct on this line would occupy, or `None` when the
/// line opens none.
///
/// An `else if` takes its neighbour's level rather than one deeper; a plain `else` opens no
/// construct at all, because it is the other half of the `if` already counted.
fn Level_Opened_By(code: &str, open_constructs: usize) -> Option<usize>
{
    let mut rest = code.trim_start();

    while let Some(after_brace) = rest.strip_prefix('}')
    {
        rest = after_brace.trim_start();
    }

    rest = Without_Loop_Label(rest);
    let mut continues_a_decision = false;

    if let Some(after_else) = Word_Prefix(rest, "else")
    {
        continues_a_decision = true;
        rest = after_else.trim_start();

        Word_Prefix(rest, "if")?;
    }

    let keyword = CONSTRUCT_KEYWORDS.iter().find(|keyword| return Opens_Construct(rest, keyword))?;

    if continues_a_decision && *keyword == "if"
    {
        return Some(open_constructs.max(1));
    }

    return Some(open_constructs.saturating_add(1));
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
fn Opens_Construct(code: &str, keyword: &str) -> bool
{
    let Some(after_keyword) = code.strip_prefix(keyword)
    else
    {
        return false;
    };

    if *keyword == *"for" && after_keyword.starts_with('<')
    {
        return false;
    }

    return after_keyword.is_empty() || after_keyword.starts_with([' ', '\t', '{']);
}

/// Whether this line declares a function, past any modifiers it carries.
fn Opens_A_Function(code: &str) -> bool
{
    let mut rest = code.trim_start();

    loop
    {
        if Word_Prefix(rest, "fn").is_some()
        {
            return true;
        }

        let Some(modifier) = FUNCTION_MODIFIERS.iter().find_map(|modifier| return Word_Prefix(rest, modifier))
        else
        {
            return false;
        };
        let mut after = modifier.trim_start();

        if let Some(after_open) = after.strip_prefix('(')
            && let Some(close) = after_open.find(')')
        {
            after = after_open.get(close.saturating_add(1)..).unwrap_or("");
        }

        rest = after.trim_start();
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

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::checks::test_support;
    use nomos_analysis::{MemoryFactStore, Reader};
    use nomos_capability::Registry;
    use nomos_contracts::SubjectId;
    use nomos_model::Content_Digest;

    #[test]
    fn Test_Check_Nesting_Depth_Should_Report_A_Function_Past_The_Limit()
    {
        let findings = Judge(&[Source("demo/src/a.rs", &Nested(MAX_NESTING_DEPTH + 1))]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.rule, RuleId::New(NESTING_DEPTH));
        assert_eq!(found.gate, GateCategory::Blocking);
    }

    /// A function *reaching* the limit is fine; only one past it is reported.
    #[test]
    fn Test_Check_Nesting_Depth_Should_Accept_A_Function_At_The_Limit()
    {
        let findings = Judge(&[Source("demo/src/a.rs", &Nested(MAX_NESTING_DEPTH))]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// One finding per function, at the first construct to cross — not one per level.
    #[test]
    fn Test_Check_Nesting_Depth_Should_Report_A_Function_Once()
    {
        let findings = Judge(&[Source("demo/src/a.rs", &Nested(MAX_NESTING_DEPTH + 3))]);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    /// An else-if is the second question in one decision, not a decision inside one.
    #[test]
    fn Test_Check_Nesting_Depth_Should_Not_Deepen_For_An_Else_If()
    {
        let text = Function(&[
            "    if a", "    {", "        if b", "        {", "            if c",
            "            {", "                return 1;", "            }",
            "            else if d", "            {", "                return 2;",
            "            }", "        }", "    }",
        ]);

        let findings = Judge(&[Source("demo/src/a.rs", &text)]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// The other half of the same rule: an else-if does not deepen, but its body still does.
    #[test]
    fn Test_Check_Nesting_Depth_Should_Deepen_Inside_An_Else_If_Body()
    {
        let text = Function(&[
            "    if a", "    {", "        if b", "        {", "            return 1;",
            "        }", "        else if c", "        {", "            if d",
            "            {", "                if e", "                {",
            "                    return 2;", "                }", "            }",
            "        }", "    }",
        ]);

        let findings = Judge(&[Source("demo/src/a.rs", &text)]);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    /// A brace that opens no control flow adds no level, which is the whole reason this
    /// counts constructs rather than braces.
    #[test]
    fn Test_Check_Nesting_Depth_Should_Not_Count_A_Brace_That_Opens_No_Control_Flow()
    {
        let text = Function(&[
            "    if a", "    {", "        let held = Point { x: 1, y: 2 };",
            "        let run = |value: u8| { return value; };", "        match held.x",
            "        {", "            0 => { return 1; },", "            _ => { return 2; },",
            "        }", "    }",
        ]);

        let findings = Judge(&[Source("demo/src/a.rs", &text)]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// A labelled loop is a loop, and a scan that stopped at the quote would miss it.
    #[test]
    fn Test_Check_Nesting_Depth_Should_Count_A_Labelled_Loop()
    {
        let text = Function(&[
            "    'outer: for row in rows", "    {", "        for column in columns",
            "        {", "            while ready", "            {", "                loop",
            "                {", "                    break 'outer;", "                }",
            "            }", "        }", "    }",
        ]);

        let findings = Judge(&[Source("demo/src/a.rs", &text)]);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    /// A higher-ranked binder in a where clause is not a loop, and reading it as one is a
    /// mistake a sibling prototype actually made against this workspace.
    #[test]
    fn Test_Check_Nesting_Depth_Should_Not_Read_A_Higher_Ranked_Binder_As_A_Loop()
    {
        let text = Function(&[
            "    if a", "    {", "        if b", "        {", "            if c",
            "            {", "                return 1;", "            }", "        }", "    }",
        ]);
        let bound = format!("where\n    for<'error> Error: From<&'error Cause>,\n{text}");

        let findings = Judge(&[Source("demo/src/a.rs", &bound)]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// A brace inside a string literal is not a block. This crate has paid twice for a
    /// scanner that could not tell the difference.
    #[test]
    fn Test_Check_Nesting_Depth_Should_Not_Count_A_Brace_Inside_A_String()
    {
        let text = Function(&[
            "    if a", "    {", "        let shape = \"if b { if c { if d { deep\";",
            "        return shape.len();", "    }",
        ]);

        let findings = Judge(&[Source("demo/src/a.rs", &text)]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Nesting_Depth_Should_Judge_Each_Function_Separately()
    {
        let text = format!("{}\n{}", Nested(MAX_NESTING_DEPTH), Nested(MAX_NESTING_DEPTH + 1));

        let findings = Judge(&[Source("demo/src/a.rs", &text)]);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_Nesting_Depth_Should_Ignore_A_Language_It_Does_Not_Judge()
    {
        let findings = Judge(&[Source("demo/src/a.go", &Nested(MAX_NESTING_DEPTH + 1))]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// The limit is a repository's to declare, the same `OD-RULES-011` mechanism the
    /// file-size triggers already resolve through — a declared ceiling of one reports a
    /// function the compiled default accepts.
    #[test]
    fn Test_Check_Nesting_Depth_Should_Resolve_A_Declared_Limit()
    {
        let offering = test_support::Offering(
            nomos_cap_limits_policy::Capability_Contract(),
            nomos_cap_limits_policy::Capability(),
            nomos_cap_limits_policy::CONTRACT_VERSION,
            "nomos.test.limits.provides",
            nomos_cap_limits_policy::Ceiling(),
        );
        let test_support::TestOffering { mut store, registry, offer } = offering;
        let payload = nomos_cap_limits_policy::LimitsPolicyPayload {
            rows: vec![nomos_cap_limits_policy::PolicyRow {
                scope: nomos_cap_limits_policy::Scope::Repository,
                key: NESTING_DEPTH_MAX_KEY.to_owned(),
                value: 1,
            }],
        };
        test_support::Materialize(
            &mut store,
            nomos_model::Subject_Of_Path(""),
            &offer,
            nomos_analysis::InputDigest::Of(&[]),
            nomos_cap_limits_policy::Payload_Schema(),
            nomos_cap_limits_policy::Encode_Payload(&payload),
        );
        let mut facts = Reader::On(&store, &registry, test_support::Test_Context());

        let findings = Check_Nesting_Depth(&[Source("demo/src/a.rs", &Nested(2))], &mut facts);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    /// Every test above reads an empty store, so the compiled default is the limit.
    fn Judge(sources: &[SourceFile]) -> Vec<Finding>
    {
        let store = MemoryFactStore::New();
        let registry = Registry::New();
        let mut facts = Reader::On(&store, &registry, test_support::Test_Context());

        return Check_Nesting_Depth(sources, &mut facts);
    }

    /// A function whose control flow nests exactly `levels` deep, built rather than spelled
    /// so the shape stays readable at any depth.
    fn Nested(levels: usize) -> String
    {
        let mut body = Vec::new();

        for level in 0..levels
        {
            let indent = "    ".repeat(level.saturating_add(1));
            body.push(format!("{indent}if ready"));
            body.push(format!("{indent}{{"));
        }

        body.push("    return 1;".to_owned());

        for level in (0..levels).rev()
        {
            body.push(format!("{}}}", "    ".repeat(level.saturating_add(1))));
        }

        let borrowed: Vec<&str> = body.iter().map(|line| return line.as_str()).collect();
        return Function(&borrowed);
    }

    fn Function(body: &[&str]) -> String
    {
        let mut lines = vec!["fn Judged(ready: bool) -> u8".to_owned(), "{".to_owned()];
        lines.extend(body.iter().map(|line| return (*line).to_owned()));
        lines.push("    return 0;".to_owned());
        lines.push("}".to_owned());
        return lines.join("\n");
    }

    fn Source(path: &str, text: &str) -> SourceFile
    {
        let mut source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
        source.language = crate::Recognized_Language_In_Tests(path);
        return source;
    }
}
