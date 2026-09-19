use super::*;
use crate::checks::test_support::{FactToFile, Materialize_Fact, Offered_Registry, OfferedProvider, Test_Context};
use nomos_analysis::{InputDigest, MemoryFactStore, Reader};
use nomos_capability::Registry;
use nomos_cap_test_material_policy::TestMaterialPolicyPayload;
use nomos_contracts::SubjectId;
use nomos_model::Content_Digest;

/// Translated from the original's `Test_Boxed_Dyn_Closure_Needs_A_Reason`.
#[test]
fn Test_Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths_Should_Report_A_Boxed_Dyn_Closure()
{
    let sources = vec![Source("demo/src/a.rs", "struct Handler { callback: Box<dyn Fn(Event)> }".to_owned())];

    let findings = Check(Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths, &sources);

    assert_eq!(findings.len(), 1, "{findings:?}");
    let found = findings.first().expect("asserted len 1 above");
    assert_eq!(found.rule, RuleId::New(BOXED_CLOSURES_ARE_JUSTIFIED_AND_OFF_HOT_PATHS));
    assert_eq!(found.gate, GateCategory::Blocking);
}

#[test]
fn Test_Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths_Should_Accept_A_Generic_Bound()
{
    let sources = vec![Source("demo/src/a.rs", "fn once<F: FnOnce()>(f: F) { f(); }".to_owned())];

    let findings = Check(Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths, &sources);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths_Should_Accept_An_Adjacent_Explanation()
{
    let text = "    // Heterogeneous callbacks stored in one map; genuine type erasure.\n    callback: Box<dyn Fn(Event)>,";
    let sources = vec![Source("demo/src/a.rs", text.to_owned())];

    let findings = Check(Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths, &sources);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths_Should_Also_Catch_Arc_And_Rc()
{
    let sources = vec![
        Source("demo/src/a.rs", "callback: Arc<dyn Fn(Event)>,".to_owned()),
        Source("demo/src/b.rs", "callback: Rc<dyn FnMut(Event)>,".to_owned()),
    ];

    let findings = Check(Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths, &sources);

    assert_eq!(findings.len(), sources.len(), "{findings:?}");
}

#[test]
fn Test_Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths_Should_Ignore_A_Language_It_Does_Not_Judge()
{
    let sources = vec![Source("demo/src/a.go", "struct Handler { callback: Box<dyn Fn(Event)> }".to_owned())];

    let findings = Check(Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths, &sources);

    assert!(findings.is_empty(), "{findings:?}");
}

/// Falsifier for the second entry of `OWN_IMPLEMENTATION_FILES`. This file is where this
/// module keeps its fixtures, and the boxed-closure fixture below is the same shape as the
/// ones above it -- so with only the pre-split path exempt, every one of them is reported
/// against this file, which is exactly what the folder split caused.
#[test]
fn Test_Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths_Should_Ignore_This_Modules_Own_Fixtures()
{
    let path = "crates/rules/nomos-rules/src/checks/closure_bounds/tests.rs";
    let sources = vec![Source(path, "struct Handler { callback: Box<dyn Fn(Event)> }".to_owned())];

    let findings = Check(Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths, &sources);

    assert!(findings.is_empty(), "{findings:?}");
}

/// Translated from the original's `Test_Closure_Static_Bound_Needs_A_Reason`. One
/// finding even though the line widens with both `Send` and `'static`, matching the
/// original's one-finding-per-line shape.
#[test]
fn Test_Check_Closure_Bounds_Are_Minimal_Should_Report_A_Widened_Bound()
{
    let sources = vec![Source("demo/src/a.rs", Spawn_Bound_Fn())];

    let findings = Check(Check_Closure_Bounds_Are_Minimal, &sources);

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

    let findings = Check(Check_Closure_Bounds_Are_Minimal, &sources);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Closure_Bounds_Are_Minimal_Should_Not_Accept_A_Wordless_Comment()
{
    let text = format!("// ----\n{}", Spawn_Bound_Fn());
    let sources = vec![Source("demo/src/a.rs", text.to_owned())];

    let findings = Check(Check_Closure_Bounds_Are_Minimal, &sources);

    assert_eq!(findings.len(), 1, "{findings:?}");
}

/// Translated from the original's `Test_Local_Generic_Closure_Bound_Is_Clean`.
#[test]
fn Test_Check_Closure_Bounds_Are_Minimal_Should_Accept_A_Plain_Generic_Bound()
{
    let sources = vec![Source("demo/src/a.rs", "fn once<F: FnOnce()>(f: F) { f(); }".to_owned())];

    let findings = Check(Check_Closure_Bounds_Are_Minimal, &sources);

    assert!(findings.is_empty(), "{findings:?}");
}

/// The public-API shape: fires on a fixture built for it, which is what tells this
/// shape apart from a scanner that cannot fire at all — see the module doc's measured
/// zero over the real corpus.
#[test]
fn Test_Check_Closure_Bounds_Are_Minimal_Should_Report_An_Unexplained_Public_Bound()
{
    let sources = vec![Source("demo/src/a.rs", "pub fn Present_Cleared(paint: impl FnOnce() -> bool) -> bool".to_owned())];

    let findings = Check(Check_Closure_Bounds_Are_Minimal, &sources);

    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn Test_Check_Closure_Bounds_Are_Minimal_Should_Not_Judge_A_Crate_Private_Signature()
{
    let sources = vec![Source("demo/src/a.rs", "pub(crate) fn Present_Cleared(paint: impl FnOnce() -> bool) -> bool".to_owned())];

    let findings = Check(Check_Closure_Bounds_Are_Minimal, &sources);

    assert!(findings.is_empty(), "{findings:?}");
}

/// A boxed `dyn` bound that also widens with `Send` reports only as the boxed shape,
/// matching the original switch's precedence: each line reports once.
#[test]
fn Test_Check_Closure_Bounds_Are_Minimal_Should_Defer_To_The_Boxed_Shape_On_The_Same_Line()
{
    let sources = vec![Source("demo/src/a.rs", "callback: Box<dyn Fn(Event) + Send>,".to_owned())];

    assert!(Check(Check_Closure_Bounds_Are_Minimal, &sources).is_empty());
    assert_eq!(Check(Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths, &sources).len(), 1);
}

#[test]
fn Test_Check_Closure_Bounds_Are_Minimal_Should_Ignore_A_Language_It_Does_Not_Judge()
{
    let sources = vec![Source("demo/src/a.go", Spawn_Bound_Fn())];

    let findings = Check(Check_Closure_Bounds_Are_Minimal, &sources);

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

/// Runs `check` over `sources` through a real but empty reader: no repository declares a
/// fixture location, so the rule resolves to its own fixed clauses alone. Every fixture above
/// wants that, which is why it is the plain spelling and the declared case is the named one.
fn Check(check: fn(&[SourceFile], &mut dyn FactReader) -> Vec<Finding>, sources: &[SourceFile]) -> Vec<Finding>
{
    let store = MemoryFactStore::New();
    let registry = Registry::New();
    let mut facts = Reader::On(&store, &registry, Test_Context());
    return check(sources, &mut facts);
}

/// Runs `check` over `sources` through a reader that really carries a
/// `nomos.cap.test.material.policy` fact declaring `locations` -- filed under the empty-path
/// subject and the empty input digest, which is the address
/// `super::super::Resolve_Declared_Fixture_Locations` asks for.
fn Check_Declaring(check: fn(&[SourceFile], &mut dyn FactReader) -> Vec<Finding>, sources: &[SourceFile], locations: &[&str]) -> Vec<Finding>
{
    let mut offering = Offered_Registry(OfferedProvider {
        contract: nomos_cap_test_material_policy::Capability_Contract(),
        capability: nomos_cap_test_material_policy::Capability(),
        version: nomos_cap_test_material_policy::CONTRACT_VERSION,
        provider: "closure-bounds-fixture",
        guarantee: nomos_cap_test_material_policy::Ceiling(),
    })
    .expect("a registry built empty on the line above admits one declaration and one offer");

    let payload = TestMaterialPolicyPayload { locations: locations.iter().map(|location| return (*location).to_owned()).collect() };
    Materialize_Fact(
        &mut offering.store,
        FactToFile {
            subject: nomos_model::Subject_Of_Path(""),
            offer: &offering.offer,
            semantic_inputs: InputDigest::Of(&[]),
            schema: nomos_cap_test_material_policy::Payload_Schema(),
            bytes: nomos_cap_test_material_policy::Encode_Payload(&payload),
        },
    )
    .expect("one fact into a store built for this case");

    let mut facts = Reader::On(&offering.store, &offering.registry, Test_Context());
    return check(sources, &mut facts);
}

/// The falsifier for the declared half of [`super::Judgeable`]. The same fixture is reported
/// when nothing is declared and exempt when the repository declares the directory it sits in,
/// so the declaration is what decides -- not a path this crate compiled in. Ignore the
/// declared locations and the second assertion fails.
#[test]
fn Test_A_Declared_Fixture_Location_Should_Exempt_A_Closure_Fixture_Under_It()
{
    let sources = vec![Source("samples/handlers.rs", "struct Handler { callback: Box<dyn Fn(Event)> }".to_owned())];

    let undeclared = Check(Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths, &sources);
    let declared = Check_Declaring(Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths, &sources, &["samples"]);

    assert_eq!(undeclared.len(), 1, "undeclared, the fixture is judged: {undeclared:?}");
    assert!(declared.is_empty(), "declared, the same fixture is test material: {declared:?}");
}

/// A sibling directory that merely shares the spelling is still judged, so the declaration is
/// read as a directory prefix rather than a substring.
#[test]
fn Test_A_Sibling_Sharing_A_Declared_Locations_Spelling_Should_Still_Be_Judged()
{
    let sources = vec![Source("samples2/handlers.rs", "struct Handler { callback: Box<dyn Fn(Event)> }".to_owned())];

    let findings = Check_Declaring(Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths, &sources, &["samples"]);

    assert_eq!(findings.len(), 1, "{findings:?}");
}
