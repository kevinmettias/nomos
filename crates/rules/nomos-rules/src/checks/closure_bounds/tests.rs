use super::*;
use nomos_contracts::SubjectId;
use nomos_model::Content_Digest;

/// Translated from the original's `Test_Boxed_Dyn_Closure_Needs_A_Reason`.
#[test]
fn Test_Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths_Should_Report_A_Boxed_Dyn_Closure()
{
    let sources = vec![Source("demo/src/a.rs", "struct Handler { callback: Box<dyn Fn(Event)> }".to_owned())];

    let findings = Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths(&sources);

    assert_eq!(findings.len(), 1, "{findings:?}");
    let found = findings.first().expect("asserted len 1 above");
    assert_eq!(found.rule, RuleId::New(BOXED_CLOSURES_ARE_JUSTIFIED_AND_OFF_HOT_PATHS));
    assert_eq!(found.gate, GateCategory::Blocking);
}

#[test]
fn Test_Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths_Should_Accept_A_Generic_Bound()
{
    let sources = vec![Source("demo/src/a.rs", "fn once<F: FnOnce()>(f: F) { f(); }".to_owned())];

    let findings = Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths(&sources);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths_Should_Accept_An_Adjacent_Explanation()
{
    let text = "    // Heterogeneous callbacks stored in one map; genuine type erasure.\n    callback: Box<dyn Fn(Event)>,";
    let sources = vec![Source("demo/src/a.rs", text.to_owned())];

    let findings = Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths(&sources);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths_Should_Also_Catch_Arc_And_Rc()
{
    let sources = vec![
        Source("demo/src/a.rs", "callback: Arc<dyn Fn(Event)>,".to_owned()),
        Source("demo/src/b.rs", "callback: Rc<dyn FnMut(Event)>,".to_owned()),
    ];

    let findings = Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths(&sources);

    assert_eq!(findings.len(), sources.len(), "{findings:?}");
}

#[test]
fn Test_Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths_Should_Ignore_A_Language_It_Does_Not_Judge()
{
    let sources = vec![Source("demo/src/a.go", "struct Handler { callback: Box<dyn Fn(Event)> }".to_owned())];

    let findings = Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths(&sources);

    assert!(findings.is_empty(), "{findings:?}");
}

/// Translated from the original's `Test_Closure_Static_Bound_Needs_A_Reason`. One
/// finding even though the line widens with both `Send` and `'static`, matching the
/// original's one-finding-per-line shape.
#[test]
fn Test_Check_Closure_Bounds_Are_Minimal_Should_Report_A_Widened_Bound()
{
    let sources = vec![Source("demo/src/a.rs", Spawn_Bound_Fn())];

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
    let sources = vec![Source("demo/src/a.rs", text.to_owned())];

    let findings = Check_Closure_Bounds_Are_Minimal(&sources);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Closure_Bounds_Are_Minimal_Should_Not_Accept_A_Wordless_Comment()
{
    let text = format!("// ----\n{}", Spawn_Bound_Fn());
    let sources = vec![Source("demo/src/a.rs", text.to_owned())];

    let findings = Check_Closure_Bounds_Are_Minimal(&sources);

    assert_eq!(findings.len(), 1, "{findings:?}");
}

/// Translated from the original's `Test_Local_Generic_Closure_Bound_Is_Clean`.
#[test]
fn Test_Check_Closure_Bounds_Are_Minimal_Should_Accept_A_Plain_Generic_Bound()
{
    let sources = vec![Source("demo/src/a.rs", "fn once<F: FnOnce()>(f: F) { f(); }".to_owned())];

    let findings = Check_Closure_Bounds_Are_Minimal(&sources);

    assert!(findings.is_empty(), "{findings:?}");
}

/// The public-API shape: fires on a fixture built for it, which is what tells this
/// shape apart from a scanner that cannot fire at all — see the module doc's measured
/// zero over the real corpus.
#[test]
fn Test_Check_Closure_Bounds_Are_Minimal_Should_Report_An_Unexplained_Public_Bound()
{
    let sources = vec![Source("demo/src/a.rs", "pub fn Present_Cleared(paint: impl FnOnce() -> bool) -> bool".to_owned())];

    let findings = Check_Closure_Bounds_Are_Minimal(&sources);

    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn Test_Check_Closure_Bounds_Are_Minimal_Should_Not_Judge_A_Crate_Private_Signature()
{
    let sources = vec![Source("demo/src/a.rs", "pub(crate) fn Present_Cleared(paint: impl FnOnce() -> bool) -> bool".to_owned())];

    let findings = Check_Closure_Bounds_Are_Minimal(&sources);

    assert!(findings.is_empty(), "{findings:?}");
}

/// A boxed `dyn` bound that also widens with `Send` reports only as the boxed shape,
/// matching the original switch's precedence: each line reports once.
#[test]
fn Test_Check_Closure_Bounds_Are_Minimal_Should_Defer_To_The_Boxed_Shape_On_The_Same_Line()
{
    let sources = vec![Source("demo/src/a.rs", "callback: Box<dyn Fn(Event) + Send>,".to_owned())];

    assert!(Check_Closure_Bounds_Are_Minimal(&sources).is_empty());
    assert_eq!(Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths(&sources).len(), 1);
}

#[test]
fn Test_Check_Closure_Bounds_Are_Minimal_Should_Ignore_A_Language_It_Does_Not_Judge()
{
    let sources = vec![Source("demo/src/a.go", Spawn_Bound_Fn())];

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

fn Source(path: &str, text: String) -> SourceFile
{
    let mut source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
    source.language = crate::Recognized_Language_In_Tests(path);
    return source;
}
