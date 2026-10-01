//! Every axis of `Declared_Guarantee` exercised against what the provider emits -- the upward
//! direction `OD-CAPABILITY-016` requires, including the axis this provider fails.

use nomos_analysis::MaterializedFact;
use nomos_cap_complexity::{Complexity_Descriptor, FunctionComplexity, Parse_Payload};
use nomos_contracts::{BuildVariantId, ConfigurationId, Digest128, FactVariant, GenerationId, SnapshotId, SubjectId};
use nomos_lang_rust_complexity::{ComplexityReading, Declared_Guarantee, FactContext, Materialization, Materialize_Complexity_Fact, Read_Complexity};
use nomos_model::Content_Digest;

/// Seeds distinct enough that no two of the context's digests compare equal.
const SNAPSHOT_SEED: u8 = 1;
const VARIANT_SEED: u8 = 2;
const CONFIGURATION_SEED: u8 = 3;

fn Context() -> FactContext
{
    return FactContext {
        snapshot: SnapshotId::From_Digest(Digest128::From_Bytes([SNAPSHOT_SEED; Digest128::BYTE_LENGTH])),
        variant: BuildVariantId::From_Digest(Digest128::From_Bytes([VARIANT_SEED; Digest128::BYTE_LENGTH])),
        configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([CONFIGURATION_SEED; Digest128::BYTE_LENGTH])),
        generation: GenerationId::INITIAL,
    };
}

fn Subject(path: &str) -> SubjectId
{
    return SubjectId::From_Digest(Content_Digest(path.as_bytes()));
}

fn Fact(source: &str) -> MaterializedFact
{
    return match Materialize_Complexity_Fact(Subject("a.rs"), source, Context())
    {
        Materialization::Materialized(fact) => *fact,
        // Every fixture here is Rust written to parse; a refusal is a broken fixture, and the
        // comparisons below would otherwise have nothing on one side and read as agreement.
        Materialization::Unparseable(failure) => panic!("expected a fact: {failure}"),
    };
}

fn Functions(source: &str) -> Vec<FunctionComplexity>
{
    return Parse_Payload(&Fact(source).payload.bytes).expect("this provider writes nomos.metric.complexity.v1").functions;
}

fn Complexity_Of(source: &str, function: &str) -> usize
{
    return Functions(source)
        .into_iter()
        .find(|found| return found.function == function)
        .map_or_else(|| panic!("{function} is not reported for {source}"), |found| return found.complexity);
}

/// Syntactic: nothing resolves the call, so a caller of a function full of branches has none of
/// its own. A resolved reading could fold the callee in; this one cannot see where the call goes.
#[test]
fn Test_The_Variant_Should_Be_Syntactic_Because_A_Call_Is_Not_Followed()
{
    let source = "fn Branchy(a: u8) -> u8 { match a { 0 => 1, 1 => 2, _ => 3 } }\nfn Caller() -> u8 { Branchy(4) }\n";

    assert_eq!(Declared_Guarantee().variant, FactVariant::Syntactic);
    assert_eq!(Complexity_Of(source, "Branchy"), 3);
    assert_eq!(Complexity_Of(source, "Caller"), 1);
}

/// Sound: every function reported is one the source declares, and the fact decodes under the
/// schema's own reader carrying the descriptor this capability publishes.
#[test]
fn Test_Soundness_Should_Hold_Every_Reported_Function_Is_Declared_In_The_Source()
{
    let source = "struct Walker;\nimpl Walker { fn Step(&self, a: bool) { if a {} } }\nfn Free() {}\n";

    let payload = Parse_Payload(&Fact(source).payload.bytes).expect("this provider's own encoding");

    assert_eq!(payload.descriptor, Complexity_Descriptor());
    assert_eq!(payload.functions.len(), 2, "{payload:?}");
    for function in &payload.functions
    {
        let own_name = function.function.rsplit("::").next().expect("split yields at least one piece");
        assert!(source.contains(&format!("fn {own_name}")), "{function:?} is not declared in the source");
    }
}

/// Completeness Unsound, and the case that shows it: the same condition is two decision points
/// written as an `if` and none written inside `assert!`, because a macro's tokens are not parsed.
#[test]
fn Test_Completeness_Should_Be_Unsound_Because_A_Macro_Hides_Its_Branches()
{
    let written = "fn Written(a: bool, b: bool) { if !(a && b) { panic!(); } }\n";
    let hidden = "fn Hidden(a: bool, b: bool) { assert!(a && b); }\n";

    assert_eq!(Complexity_Of(written, "Written"), 3);
    assert_eq!(Complexity_Of(hidden, "Hidden"), 1, "a branch inside a macro must be the known omission, not a count");
}

/// The other half of the same omission: a function a macro defines is not reported at all.
#[test]
fn Test_Completeness_Should_Be_Unsound_Because_A_Macro_Defined_Function_Is_Not_Reported()
{
    let source = "macro_rules! define { () => { fn Generated() {} }; }\ndefine!();\nfn Written() {}\n";

    assert_eq!(Functions(source), vec![FunctionComplexity { function: "Written".to_owned(), line: 3, complexity: 1 }]);
}

/// File: one file's bytes are the whole input, so the same bytes are the same computation under
/// any subject, and a change to them is a different one.
#[test]
fn Test_The_Granularity_Should_Be_File_Because_A_Reading_Carries_Nothing_Across()
{
    let source = "fn One(a: bool) { if a {} }\n";
    let first = Materialize_Complexity_Fact(Subject("a.rs"), source, Context());
    let second = Materialize_Complexity_Fact(Subject("b.rs"), source, Context());
    let (Materialization::Materialized(first), Materialization::Materialized(second)) = (first, second)
    else
    {
        panic!("both fixtures parse");
    };

    assert_eq!(first.Key().semantic_inputs, second.Key().semantic_inputs);
    assert_eq!(first.payload.Digest(), second.payload.Digest());
    assert_ne!(first.Key().semantic_inputs, Fact("fn One(a: bool) { if a {} if a {} }\n").Key().semantic_inputs);
}

/// A file that did not parse and a file that defines nothing are different answers.
#[test]
fn Test_A_Failure_And_An_Empty_File_Should_Never_Be_The_Same_Answer()
{
    assert!(matches!(Read_Complexity("fn unclosed( {"), ComplexityReading::Unparseable(_)));
    assert_eq!(Read_Complexity(""), ComplexityReading::Parsed(Vec::new()));
    assert!(Functions("").is_empty());
}
