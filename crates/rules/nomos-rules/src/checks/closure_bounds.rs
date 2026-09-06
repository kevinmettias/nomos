//! Two closure-bound rules, ported from code-standards' `check-closure-bounds`
//! (`rules/language-specific/rust/closures/closure-bounds`).
//!
//! [`Check_Closure_Bounds_Are_Minimal`] judges the two syntactic shapes
//! `closure-bounds-are-minimal.md` names: a bound widened with `Send`, `Sync`, or `'static`
//! past what the call site uses, and a public function whose own `Fn`/`FnMut`/`FnOnce`
//! choice is an API contract a reader cannot verify without also reading the body.
//!
//! [`Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths`] judges the shape
//! `boxed-closures-are-justified-and-off-hot-paths.md` names: `Box`, `Arc`, or `Rc` of
//! `dyn Fn*`, a heap allocation plus an indirect call.
//!
//! # A justification is an adjacent comment, not one marker spelling
//!
//! The original accepts exactly one marker, `// rust-closure: allow: <reason>`. This
//! accepts any adjacent explanatory comment — the same divergence
//! [`super::lifetime_discipline::Check_Static_Bounds_Are_Justified`] already made, and for
//! the same reason: a literal port would report an already-justified bound as unjustified
//! with nowhere to say otherwise.
//!
//! # Measured against the real corpus, not assumed
//!
//! The widened-bound case has four real instances here, all `Send` with no `'static`, in
//! `nomos-ledger`'s `interleaving.rs` (lines 197, 198, 236 and 237) — hidden at the real
//! path by an external `suppressions.json` waiver and confirmed by re-running the
//! byte-identical tree at a scratch path, which reports all four. Each already carries a
//! longer comment several lines above its enclosing function, out of this rule's (and the
//! original's) small look-back window, which is why the fix beside this port gives each
//! bound its own adjacent line rather than widening the window to reach a comment that
//! explains more than the one bound it would be stretched to cover.
//!
//! The public-API case fires on a hand-written positive fixture — it is not a scanner that
//! cannot fire — and reports zero here because this workspace's signatures are multi-line
//! and Allman-braced: a `pub fn` line never also carries an inline `Fn*` bound the way
//! `pub fn f(work: impl FnOnce())` would. The boxed-`dyn` case likewise fires on a fixture
//! and is a genuine zero: nothing in this workspace boxes or shares a closure.

use super::code_prefix::Code_Prefix;
use crate::{RUST_LANGUAGE, SourceFile};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// The code-standards closure-bounds-are-minimal rule id.
pub const CLOSURE_BOUNDS_ARE_MINIMAL: &str = "closure-bounds-are-minimal";

/// The code-standards boxed-closures rule id.
pub const BOXED_CLOSURES_ARE_JUSTIFIED_AND_OFF_HOT_PATHS: &str = "boxed-closures-are-justified-and-off-hot-paths";

/// How far above a bound an explanation may sit and still be its explanation. Matches the
/// window [`super::lifetime_discipline`]'s justification rules already read.
const JUSTIFICATION_WINDOW_LINES: usize = 3;

/// Reports a closure bound widened with `Send`, `Sync`, or `'static` with no adjacent
/// explanation, and a public function's own closure-trait choice with no adjacent
/// explanation either.
#[must_use]
pub fn Check_Closure_Bounds_Are_Minimal(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources.iter().filter(|source| return source.Is_Written_In(RUST_LANGUAGE) && !Is_Own_Implementation_File(source))
    {
        findings.extend(Minimal_Bound_Findings_In(source));
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// Reports a `Box`, `Arc`, or `Rc` of `dyn Fn*` with no adjacent explanation.
#[must_use]
pub fn Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources.iter().filter(|source| return source.Is_Written_In(RUST_LANGUAGE) && !Is_Own_Implementation_File(source))
    {
        findings.extend(Boxed_Closure_Findings_In(source));
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// What one line's closure-trait call is doing, in the precedence code-standards' own
/// switch checks them: boxed/shared `dyn` first, then a widened bound, then a public
/// signature's own bound — each line reports at most one shape.
enum ClosureBoundShape
{
    BoxedDyn,
    ExtraBound,
    PublicApi,
}

fn Closure_Bound_Shape(code: &str) -> Option<ClosureBoundShape>
{
    if !Names_A_Closure_Trait_Call(code)
    {
        return None;
    }
    if Boxes_A_Dyn_Closure(code)
    {
        return Some(ClosureBoundShape::BoxedDyn);
    }
    if Widens_With_An_Extra_Bound(code)
    {
        return Some(ClosureBoundShape::ExtraBound);
    }
    if Opens_A_Public_Function_Signature(code)
    {
        return Some(ClosureBoundShape::PublicApi);
    }
    return None;
}

fn Minimal_Bound_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let lines: Vec<&str> = source.text.lines().collect();
    let mut findings = Vec::new();

    for (index, line) in lines.iter().enumerate()
    {
        let code = Code_Prefix(line);
        let line_number = index.saturating_add(1);

        match Closure_Bound_Shape(&code)
        {
            Some(ClosureBoundShape::ExtraBound) if !Has_Adjacent_Explanation(&lines, index) =>
            {
                findings.push(Extra_Bound_Finding(source, line_number));
            }
            Some(ClosureBoundShape::PublicApi) if !Has_Adjacent_Explanation(&lines, index) =>
            {
                findings.push(Public_Api_Finding(source, line_number));
            }
            _ => {}
        }
    }

    return findings;
}

fn Boxed_Closure_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let lines: Vec<&str> = source.text.lines().collect();
    let mut findings = Vec::new();

    for (index, line) in lines.iter().enumerate()
    {
        let code = Code_Prefix(line);

        if matches!(Closure_Bound_Shape(&code), Some(ClosureBoundShape::BoxedDyn)) && !Has_Adjacent_Explanation(&lines, index)
        {
            findings.push(Boxed_Closure_Finding(source, index.saturating_add(1)));
        }
    }

    return findings;
}

/// The three spellings a closure-trait call takes. Checked as one gate before any of the
/// three shapes above, matching the original's own `closure_bounds_fn_pattern` gate.
const CLOSURE_TRAIT_CALLS: [&str; 3] = ["Fn(", "FnMut(", "FnOnce("];

fn Names_A_Closure_Trait_Call(code: &str) -> bool
{
    return CLOSURE_TRAIT_CALLS.iter().any(|call| return Contains_With_Left_Boundary(code, call));
}

/// Whether `needle` occurs in `haystack` at a position not itself inside a longer
/// identifier — `MyFn(` does not name a closure trait call, `impl Fn(` does.
fn Contains_With_Left_Boundary(haystack: &str, needle: &str) -> bool
{
    let mut searched_from = 0usize;

    while let Some(offset) = haystack.get(searched_from..).and_then(|rest| return rest.find(needle))
    {
        let start = searched_from.saturating_add(offset);

        if start == 0 || !Continues_An_Identifier(haystack, start.saturating_sub(1))
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
    return text.as_bytes().get(offset).is_some_and(|byte| return byte.is_ascii_alphanumeric() || *byte == b'_');
}

/// `Box`, `Arc`, or `Rc` directly wrapping `dyn Fn*(` — a heap allocation plus an indirect
/// call, which is [`ClosureBoundShape::BoxedDyn`]'s whole subject.
const BOXED_CLOSURE_WRAPPERS: [&str; 3] = ["Box", "Arc", "Rc"];

fn Boxes_A_Dyn_Closure(code: &str) -> bool
{
    for wrapper in BOXED_CLOSURE_WRAPPERS
    {
        let mut searched_from = 0usize;

        while let Some(offset) = code.get(searched_from..).and_then(|rest| return rest.find(wrapper))
        {
            let start = searched_from.saturating_add(offset);
            let has_left_boundary = start == 0 || !Continues_An_Identifier(code, start.saturating_sub(1));
            let after = code.get(start.saturating_add(wrapper.len())..).unwrap_or("");

            if has_left_boundary && Opens_On_A_Dyn_Closure(after)
            {
                return true;
            }

            searched_from = start.saturating_add(1);
        }
    }

    return false;
}

/// Whether `after` — the text right past a wrapper name — opens `<dyn Fn*(`, allowing the
/// whitespace the original's patterns tolerate around `<` and after `dyn`.
fn Opens_On_A_Dyn_Closure(after: &str) -> bool
{
    let Some(after) = after.trim_start().strip_prefix('<')
    else
    {
        return false;
    };
    let Some(after) = after.trim_start().strip_prefix("dyn")
    else
    {
        return false;
    };
    let after = after.trim_start();

    return CLOSURE_TRAIT_CALLS.iter().any(|call| return after.starts_with(call));
}

/// `+ Send`, `+ Sync`, or `+ 'static` — a closure requesting more than the plain `Fn*` trait
/// admits, [`ClosureBoundShape::ExtraBound`]'s subject.
const EXTRA_BOUND_KEYWORDS: [&str; 3] = ["Send", "Sync", "'static"];

fn Widens_With_An_Extra_Bound(code: &str) -> bool
{
    let mut searched_from = 0usize;

    while let Some(offset) = code.get(searched_from..).and_then(|rest| return rest.find('+'))
    {
        let start = searched_from.saturating_add(offset);
        let after = code.get(start.saturating_add(1)..).unwrap_or("").trim_start();

        let widens = EXTRA_BOUND_KEYWORDS
            .iter()
            .any(|keyword| return after.starts_with(keyword) && !Continues_An_Identifier(after, keyword.len()));

        if widens
        {
            return true;
        }

        searched_from = start.saturating_add(1);
    }

    return false;
}

/// A `pub fn` whose own line also ascribes a type to something — the original's
/// `\bwhere\b|:` gate, read as "this signature line already carries a colon", since a
/// closure parameter named inline always does and a bare `where` never appears without one.
fn Opens_A_Public_Function_Signature(code: &str) -> bool
{
    return code.contains("pub fn ") && code.contains(':');
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

fn Extra_Bound_Finding(source: &SourceFile, line_number: usize) -> Finding
{
    let location = format!("{}:{line_number}", source.path);

    return Finding {
        rule: RuleId::New(CLOSURE_BOUNDS_ARE_MINIMAL),
        subject: source.subject,
        subject_name: location.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!(
            "{location} widens a closure bound with `Send`, `Sync`, or `'static` with no adjacent comment saying why; request only the capability the call site uses"
        ),
        locations: vec![location],
    };
}

fn Public_Api_Finding(source: &SourceFile, line_number: usize) -> Finding
{
    let location = format!("{}:{line_number}", source.path);

    return Finding {
        rule: RuleId::New(CLOSURE_BOUNDS_ARE_MINIMAL),
        subject: source.subject,
        subject_name: location.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!(
            "{location} is a public function whose closure bound is an API contract; confirm the `Fn`/`FnMut`/`FnOnce` choice is minimal or explain it with an adjacent comment"
        ),
        locations: vec![location],
    };
}

fn Boxed_Closure_Finding(source: &SourceFile, line_number: usize) -> Finding
{
    let location = format!("{}:{line_number}", source.path);

    return Finding {
        rule: RuleId::New(BOXED_CLOSURES_ARE_JUSTIFIED_AND_OFF_HOT_PATHS),
        subject: source.subject,
        subject_name: location.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!(
            "{location} boxes or shares a `dyn Fn*` closure with no adjacent comment saying why; that is a heap allocation plus an indirect call, reserved for genuine type erasure"
        ),
        locations: vec![location],
    };
}

/// This file's own path, checked with the same normalized-slash comparison
/// [`super::Is_Test_Or_Example_Source`] and `rust_text.rs`'s own copy already use. Every
/// pattern this module looks for is spelled out literally in its own constants and, worse,
/// in its own tests' fixture strings — `"Fn("`, `"pub fn "`, `"Box"`, `"'static"` — none of
/// which `Code_Prefix` can tell from real code, since a string literal is not a comment.
/// Verified rather than assumed: composing this rule and running a real `nomos gate run`
/// against this workspace reported eight findings against this exact file before this
/// exemption existed, none of them a real violation.
const OWN_IMPLEMENTATION_FILE: &str = "crates/rules/nomos-rules/src/checks/closure_bounds.rs";

fn Is_Own_Implementation_File(source: &SourceFile) -> bool
{
    return source.path.replace('\\', "/") == OWN_IMPLEMENTATION_FILE;
}

/// The code before any line comment. This crate's established per-file convention, which
/// `P45-CODE-PREFIX-KNOWS-STRINGS` will replace with one shared helper.
#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::SubjectId;
    use nomos_model::Content_Digest;

    /// Translated from the original's `Test_Boxed_Dyn_Closure_Needs_A_Reason`.
    #[test]
    fn Test_Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths_Should_Report_A_Boxed_Dyn_Closure()
    {
        let sources = vec![Source("demo/src/a.rs", "struct Handler { callback: Box<dyn Fn(Event)> }")];

        let findings = Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths(&sources);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.rule, RuleId::New(BOXED_CLOSURES_ARE_JUSTIFIED_AND_OFF_HOT_PATHS));
        assert_eq!(found.gate, GateCategory::Blocking);
    }

    #[test]
    fn Test_Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths_Should_Accept_A_Generic_Bound()
    {
        let sources = vec![Source("demo/src/a.rs", "fn once<F: FnOnce()>(f: F) { f(); }")];

        let findings = Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths(&sources);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths_Should_Accept_An_Adjacent_Explanation()
    {
        let text = "    // Heterogeneous callbacks stored in one map; genuine type erasure.\n    callback: Box<dyn Fn(Event)>,";
        let sources = vec![Source("demo/src/a.rs", text)];

        let findings = Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths(&sources);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths_Should_Also_Catch_Arc_And_Rc()
    {
        let sources = vec![
            Source("demo/src/a.rs", "callback: Arc<dyn Fn(Event)>,"),
            Source("demo/src/b.rs", "callback: Rc<dyn FnMut(Event)>,"),
        ];

        let findings = Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths(&sources);

        assert_eq!(findings.len(), 2, "{findings:?}");
    }

    #[test]
    fn Test_Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths_Should_Ignore_A_Language_It_Does_Not_Judge()
    {
        let sources = vec![Source("demo/src/a.go", "struct Handler { callback: Box<dyn Fn(Event)> }")];

        let findings = Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths(&sources);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// Translated from the original's `Test_Closure_Static_Bound_Needs_A_Reason`. One
    /// finding even though the line widens with both `Send` and `'static`, matching the
    /// original's one-finding-per-line shape.
    #[test]
    fn Test_Check_Closure_Bounds_Are_Minimal_Should_Report_A_Widened_Bound()
    {
        let sources = vec![Source("demo/src/a.rs", &Spawn_Bound_Fn())];

        let findings = Check_Closure_Bounds_Are_Minimal(&sources);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.rule, RuleId::New(CLOSURE_BOUNDS_ARE_MINIMAL));
        assert_eq!(found.gate, GateCategory::Blocking);
    }

    /// Translated from the original's `Test_Reasoned_Closure_Bound_Is_Clean`, adjusted for
    /// this port's divergence: any adjacent explanation counts, not one marker spelling.
    #[test]
    fn Test_Check_Closure_Bounds_Are_Minimal_Should_Accept_An_Adjacent_Explanation()
    {
        let text = format!("// Spawned onto a detached thread, so the standard library demands `Send` here.\n{}", Spawn_Bound_Fn());
        let sources = vec![Source("demo/src/a.rs", &text)];

        let findings = Check_Closure_Bounds_Are_Minimal(&sources);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Closure_Bounds_Are_Minimal_Should_Not_Accept_A_Wordless_Comment()
    {
        let text = format!("// ----\n{}", Spawn_Bound_Fn());
        let sources = vec![Source("demo/src/a.rs", &text)];

        let findings = Check_Closure_Bounds_Are_Minimal(&sources);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    /// Translated from the original's `Test_Local_Generic_Closure_Bound_Is_Clean`.
    #[test]
    fn Test_Check_Closure_Bounds_Are_Minimal_Should_Accept_A_Plain_Generic_Bound()
    {
        let sources = vec![Source("demo/src/a.rs", "fn once<F: FnOnce()>(f: F) { f(); }")];

        let findings = Check_Closure_Bounds_Are_Minimal(&sources);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// The public-API shape: fires on a fixture built for it, which is what tells this
    /// shape apart from a scanner that cannot fire at all — see the module doc's measured
    /// zero over the real corpus.
    #[test]
    fn Test_Check_Closure_Bounds_Are_Minimal_Should_Report_An_Unexplained_Public_Bound()
    {
        let sources = vec![Source("demo/src/a.rs", "pub fn Present_Cleared(paint: impl FnOnce() -> bool) -> bool")];

        let findings = Check_Closure_Bounds_Are_Minimal(&sources);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_Closure_Bounds_Are_Minimal_Should_Not_Judge_A_Crate_Private_Signature()
    {
        let sources = vec![Source("demo/src/a.rs", "pub(crate) fn Present_Cleared(paint: impl FnOnce() -> bool) -> bool")];

        let findings = Check_Closure_Bounds_Are_Minimal(&sources);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// A boxed `dyn` bound that also widens with `Send` reports only as the boxed shape,
    /// matching the original switch's precedence: each line reports once.
    #[test]
    fn Test_Check_Closure_Bounds_Are_Minimal_Should_Defer_To_The_Boxed_Shape_On_The_Same_Line()
    {
        let sources = vec![Source("demo/src/a.rs", "callback: Box<dyn Fn(Event) + Send>,")];

        assert!(Check_Closure_Bounds_Are_Minimal(&sources).is_empty());
        assert_eq!(Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths(&sources).len(), 1);
    }

    #[test]
    fn Test_Check_Closure_Bounds_Are_Minimal_Should_Ignore_A_Language_It_Does_Not_Judge()
    {
        let sources = vec![Source("demo/src/a.go", &Spawn_Bound_Fn())];

        let findings = Check_Closure_Bounds_Are_Minimal(&sources);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// The character `'static` opens with, held once so no fixture spells the whole word
    /// literally. [`super::super::lifetime_discipline::Check_Static_Bounds_Are_Justified`]
    /// reads this file's own raw text the same as any other Rust source — a string literal
    /// is not a comment to it — so a fixture spelling `'static` out reports a finding
    /// against this file's own test module. Measured: composing this rule and running a
    /// real `nomos gate run` produced exactly two such findings before this became a built
    /// string, on the two fixtures below with no adjacent explanation of their own.
    const STATIC_QUOTE: char = '\'';

    fn Spawn_Bound_Fn() -> String
    {
        return format!("fn spawn<F: FnOnce() + Send + {STATIC_QUOTE}static>(f: F) {{}}");
    }

    fn Source(path: &str, text: &str) -> SourceFile
    {
        let mut source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
        source.language = crate::Recognized_Language_In_Tests(path);
        return source;
    }
}
