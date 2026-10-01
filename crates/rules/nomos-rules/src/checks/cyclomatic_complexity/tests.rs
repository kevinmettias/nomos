//! The rule's judgment over hand-built payloads, and its reading through a real registry, store
//! and reader -- with and without a declared limit, and with and without a fact to read.

use super::*;
use crate::checks::test_support::{self, FactToFile, OfferedProvider, Test_Context, TestOffering};
use nomos_analysis::{MemoryFactStore, Reader};
use nomos_cap_complexity::{Complexity_Descriptor, Encode_Payload};
use nomos_cap_limits_policy::{LimitsPolicyPayload, PolicyRow, Scope};
use nomos_capability::ProviderOffer;
use nomos_contracts::SubjectId;
use nomos_model::Content_Digest;

/// The limit every declared fixture here judges against.
const LIMIT: u32 = 10;
/// One past [`LIMIT`], the smallest complexity a declared limit of ten reports.
const PAST_THE_LIMIT: usize = 11;

fn Source(path: &str, text: &str) -> SourceFile
{
    let mut source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text.to_owned());
    source.language = crate::Recognized_Language_In_Tests(path);
    return source;
}

fn Function(name: &str, line: usize, complexity: usize) -> FunctionComplexity
{
    return FunctionComplexity { function: name.to_owned(), line, complexity };
}

fn Payload(functions: Vec<FunctionComplexity>) -> ComplexityPayload
{
    return ComplexityPayload { descriptor: Complexity_Descriptor(), functions };
}

#[test]
fn Test_Violations_In_Should_Report_Only_A_Function_Past_The_Limit()
{
    let payload = Payload(vec![Function("Simple", 1, 3), Function("Branchy", 9, PAST_THE_LIMIT)]);

    let findings = Violations_In(&payload, &Source("a.rs", ""), usize::try_from(LIMIT).expect("ten fits"));

    assert_eq!(findings.len(), 1, "{findings:?}");
    let found = findings.first().expect("asserted len 1 above");
    assert_eq!(found.subject_name, "a.rs:9");
    assert_eq!(found.applicability, Applicability::PartiallySupported);
    assert_eq!(found.gate, GateCategory::Advisory);
    assert!(found.summary.contains("`Branchy`") && found.summary.contains("complexity 11"), "{}", found.summary);
}

/// The limit is the most a function may reach, so reaching it is not a breach.
#[test]
fn Test_Violations_In_Should_Pass_A_Function_That_Reaches_The_Limit_Exactly()
{
    let payload = Payload(vec![Function("AtTheLimit", 1, 10)]);

    let findings = Violations_In(&payload, &Source("a.rs", ""), 10);

    assert!(findings.is_empty(), "{findings:?}");
}

/// A descriptor saying the values may be summed is not the reading this rule compares one
/// function at a time under, so the payload is refused rather than read under terms it was not
/// written for.
#[test]
fn Test_Violations_In_Should_Refuse_A_Payload_Whose_Descriptor_Allows_Aggregation()
{
    let mut payload = Payload(vec![Function("Branchy", 1, 40)]);
    payload.descriptor.aggregation = Aggregation::Sum;

    let findings = Violations_In(&payload, &Source("a.rs", ""), 10);

    assert_eq!(findings.len(), 1, "{findings:?}");
    let found = findings.first().expect("asserted len 1 above");
    assert_eq!(found.applicability, Applicability::Unparseable);
    assert!(found.summary.contains("sum aggregation"), "{}", found.summary);
}

#[test]
fn Test_Check_Should_Report_Nothing_Where_There_Is_No_Rust_Source()
{
    let Offerings { store, registry, .. } = Offerings();
    let mut reader = Reader::On(&store, &registry, Test_Context());

    let findings = Check_Cyclomatic_Complexity(&[Source("main.go", "package main\n")], &mut reader);

    assert!(findings.is_empty(), "{findings:?}");
}

/// No limit declared: nothing is judged, however complex, and the one finding says the axis is
/// undeclared and carries what was measured.
#[test]
fn Test_Check_Should_Report_An_Undeclared_Limit_Rather_Than_Judge_Against_A_Default()
{
    let source = Source("a.rs", "fn Branchy() {}\n");
    let Offerings { mut store, registry, complexity, .. } = Offerings();
    File_Complexity(&mut store, &complexity, &source, &Payload(vec![Function("Simple", 1, 1), Function("Branchy", 4, 40)]));
    let mut reader = Reader::On(&store, &registry, Test_Context());

    let findings = Check_Cyclomatic_Complexity(&[source], &mut reader);

    assert_eq!(findings.len(), 1, "{findings:?}");
    let found = findings.first().expect("asserted len 1 above");
    assert_eq!(found.applicability, Applicability::NotApplicable, "not coverage debt: nothing failed");
    assert_eq!(found.subject_name, "nomos-limits.json");
    assert_eq!(found.address.as_deref(), Some("nomos-limits.json#*.cyclomatic-complexity-max"));
    assert!(found.summary.contains("2 function(s) were measured in 1 of 1 Rust source(s)"), "{}", found.summary);
    assert!(found.summary.contains("`Branchy` at a.rs:4 with 40"), "{}", found.summary);
}

#[test]
fn Test_Check_Should_Judge_Against_A_Declared_Limit()
{
    let source = Source("a.rs", "fn Branchy() {}\n");
    let Offerings { mut store, registry, complexity, limits } = Offerings();
    File_Complexity(&mut store, &complexity, &source, &Payload(vec![Function("Simple", 1, 1), Function("Branchy", 4, PAST_THE_LIMIT)]));
    File_Limit(&mut store, &limits, LIMIT);
    let mut reader = Reader::On(&store, &registry, Test_Context());

    let findings = Check_Cyclomatic_Complexity(&[source], &mut reader);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "a.rs:4");
}

/// A declared limit and no fact to judge against it is a finding, never a clean source.
#[test]
fn Test_Check_Should_Report_A_Source_With_No_Fact_Rather_Than_Pass_It()
{
    let source = Source("a.rs", "fn Branchy() {}\n");
    let Offerings { mut store, registry, limits, .. } = Offerings();
    File_Limit(&mut store, &limits, LIMIT);
    let mut reader = Reader::On(&store, &registry, Test_Context());

    let findings = Check_Cyclomatic_Complexity(&[source], &mut reader);

    assert_eq!(findings.len(), 1, "{findings:?}");
    let found = findings.first().expect("asserted len 1 above");
    assert!(found.applicability.Is_Coverage_Debt(), "{found:?}");
}

/// A registry declaring both families this rule reads, each with one offer, and the store they
/// are filed into.
struct Offerings
{
    store: MemoryFactStore,
    registry: nomos_capability::Registry,
    complexity: ProviderOffer,
    limits: ProviderOffer,
}

fn Offerings() -> Offerings
{
    let TestOffering { store, mut registry, offer: complexity } = test_support::Offered_Registry(OfferedProvider {
        contract: nomos_cap_complexity::Capability_Contract(),
        capability: nomos_cap_complexity::Capability(),
        version: nomos_cap_complexity::CONTRACT_VERSION,
        provider: "nomos.test.complexity.counts",
        guarantee: Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Unsound, IncrementalGranularity::File),
    })
    .expect("a fresh Registry holds neither this contract nor this provider");
    let limits = ProviderOffer {
        provider: nomos_contracts::ProviderId::New("nomos.test.limits.provides"),
        capability: nomos_cap_limits_policy::Capability(),
        version: nomos_cap_limits_policy::CONTRACT_VERSION,
        guarantee: nomos_cap_limits_policy::Ceiling(),
    };
    registry
        .Declare_And_Offer(nomos_cap_limits_policy::Capability_Contract(), limits.clone())
        .expect("the registry holds only the complexity contract so far");

    return Offerings { store, registry, complexity, limits };
}

fn File_Complexity(store: &mut MemoryFactStore, offer: &ProviderOffer, source: &SourceFile, payload: &ComplexityPayload)
{
    test_support::Materialize_Fact(store, FactToFile {
        subject: source.subject,
        offer,
        semantic_inputs: InputDigest::Of(&[source.text.as_bytes()]),
        schema: nomos_cap_complexity::Payload_Schema(),
        bytes: Encode_Payload(payload),
    })
    .expect("the fixture's store holds no fact under this key at a newer generation");
}

fn File_Limit(store: &mut MemoryFactStore, offer: &ProviderOffer, limit: u32)
{
    let payload = LimitsPolicyPayload {
        rows: vec![PolicyRow { scope: Scope::Repository, key: CYCLOMATIC_COMPLEXITY_MAX.key.to_owned(), value: limit }],
    };
    test_support::Materialize_Fact(store, FactToFile {
        subject: nomos_model::Subject_Of_Path(""),
        offer,
        semantic_inputs: InputDigest::Of(&[]),
        schema: nomos_cap_limits_policy::Payload_Schema(),
        bytes: nomos_cap_limits_policy::Encode_Payload(&payload),
    })
    .expect("the fixture's store holds no fact under this key at a newer generation");
}
