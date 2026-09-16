use super::*;
use crate::GO_LANGUAGE;
use crate::checks::test_support::{self, FactToFile, OfferedProvider, Test_Context, TestOffering};
use nomos_analysis::{InputDigest, MemoryFactStore, Reader};
use nomos_capability::ProviderOffer;
use nomos_contracts::{Assurance, FactVariant, GateCategory, Guarantee, IncrementalGranularity};

#[test]
fn Test_Violations_In_Should_Report_A_Free_Function_With_Five_Parameters()
{
    let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPublic\tBuild\t.\t+fn/5\n");

    let findings = Violations_In(Parameter_Count_Policy(), &payload, "src/lib.rs");

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(PARAMETER_COUNT));
}

#[test]
fn Test_Violations_In_Should_Accept_A_Free_Function_With_Four_Parameters()
{
    let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPublic\tBuild\t.\t+fn/4\n");

    let findings = Violations_In(Parameter_Count_Policy(), &payload, "src/lib.rs");

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Violations_In_Should_Not_Report_A_Qualified_Function_At_Receiver_Plus_Four()
{
    let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPublic\tBuilder::Build\t.\t+fn/5\n");

    let findings = Violations_In(Parameter_Count_Policy(), &payload, "src/lib.rs");

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Violations_In_Should_Report_A_Qualified_Function_With_Six_Parameters()
{
    let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPublic\tBuilder::Build\t.\t+fn/6\n");

    let findings = Violations_In(Parameter_Count_Policy(), &payload, "src/lib.rs");

    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn Test_Violations_In_Should_Use_The_Configured_Rule_Id_Threshold_And_Gate()
{
    const CAP: u32 = 2;
    let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPublic\tBuild\t.\t+fn/3\n");
    let policy = FunctionArityPolicy::New("custom-three-parameter-cap", CAP)
        .With_Gate(GateCategory::Advisory);

    let findings = Violations_In(policy, &payload, "src/lib.rs");

    assert_eq!(findings.len(), 1, "{findings:?}");
    let found = findings.first().expect("asserted len 1 above");
    assert_eq!(found.rule, RuleId::New("custom-three-parameter-cap"));
    assert_eq!(found.gate, GateCategory::Advisory);
    assert!(found.summary.contains("cap of 2"), "{}", found.summary);
}

#[test]
fn Test_Check_Function_Arity_Policy_Should_Filter_By_Configured_Source_Extension()
{
    const CAP: u32 = 2;
    let source = Source(Path("src/lib.rs"), Text("pub fn Build(a: A, b: B, c: C) {}"));
    let TestOffering { store, registry, .. } = Offering();
    let policy = FunctionArityPolicy::New("custom-go-only-cap", CAP).For_Language(GO_LANGUAGE);

    let mut reader = Reader::On(&store, &registry, Test_Context());
    let findings = Check_Function_Arity_Policy(&[source], &mut reader, policy);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Violations_In_Should_Apply_Receiver_Allowance_Only_When_Configured()
{
    const CAP: u32 = 4;
    let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPublic\tBuilder::Build\t.\t+fn/5\n");

    let strict_policy = FunctionArityPolicy::New("strict-cap", CAP);
    let strict = Violations_In(strict_policy, &payload, "src/lib.rs");
    let receiver_aware_policy =
        FunctionArityPolicy::New("receiver-aware-cap", CAP).Allow_One_Receiver_For_Qualified_Functions();
    let receiver_aware = Violations_In(receiver_aware_policy, &payload, "src/lib.rs");

    assert_eq!(strict.len(), 1, "{strict:?}");
    assert!(receiver_aware.is_empty(), "{receiver_aware:?}");
}

#[test]
fn Test_Check_Go_Helpers_Package_Five_Inputs_Should_Report_Under_The_Go_Rule_Id()
{
    let source = Source(Path("builder.go"), Text("func Build(a A, b B, c C, d D, e E) {}"));
    let TestOffering { store, registry, .. } = Offering_With_A_Five_Parameter_Build(&source);
    let mut reader = Reader::On(&store, &registry, Test_Context());
    let findings = Check_Go_Helpers_Package_Five_Inputs(&[source], &mut reader);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(
        findings.first().expect("asserted len 1 above").rule,
        RuleId::New(GO_HELPERS_PACKAGE_FIVE_INPUTS)
    );
}

#[test]
fn Test_Check_Go_Helpers_Package_Five_Inputs_Should_Ignore_Non_Go_Sources()
{
    let source = Source(Path("src/lib.rs"), Text("pub fn Build(a: A, b: B, c: C, d: D, e: E) {}"));
    let TestOffering { store, registry, .. } = Offering();

    let mut reader = Reader::On(&store, &registry, Test_Context());
    let findings = Check_Go_Helpers_Package_Five_Inputs(&[source], &mut reader);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Parameter_Count_Should_Read_And_Judge_A_Real_Fact()
{
    let source = Source(Path("src/build.rs"), Text("pub fn Build(a: A, b: B, c: C, d: D, e: E) {}"));
    let TestOffering { store, registry, .. } = Offering_With_A_Five_Parameter_Build(&source);
    let mut reader = Reader::On(&store, &registry, Test_Context());
    let findings = Check_Parameter_Count(&[source], &mut reader);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "Build");
}

/// The fact is materialized so the source really would report: without the exemption the
/// rule reads it, finds five parameters and returns a finding, so this fails if the filter
/// is deleted rather than passing on an unreadable source.
#[test]
fn Test_Check_Parameter_Count_Should_Not_Judge_A_Test_Source()
{
    let source = Source(Path("crates/rules/nomos-rules/tests/integration_seams.rs"), Text("pub fn Build(a: A, b: B, c: C, d: D, e: E) {}"));
    let TestOffering { store, registry, .. } = Offering_With_A_Five_Parameter_Build(&source);
    let mut reader = Reader::On(&store, &registry, Test_Context());
    let findings = Check_Parameter_Count(&[source], &mut reader);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Resolve_Limit_Should_Fall_Back_To_The_Default_When_No_Fact_Is_Materialized()
{
    let store = MemoryFactStore::New();
    let registry = nomos_capability::Registry::New();
    let mut facts = Reader::On(&store, &registry, Test_Context());

    let resolved = Resolve_Limit(&mut facts, None, PARAMETER_COUNT_MAX_KEY, MAX_VALUE_PARAMETERS);

    assert_eq!(resolved, MAX_VALUE_PARAMETERS);
}

#[test]
fn Test_Resolve_Limit_Should_Prefer_The_Repository_Wide_Row_Over_The_Default()
{
    const REPOSITORY_MAX: u32 = 6;
    let TestOffering { mut store, registry, offer } = Limits_Offering();
    Materialize_Limits_Fact(
        &mut store,
        &offer,
        vec![nomos_cap_limits_policy::PolicyRow {
            scope: Scope::Repository,
            key: PARAMETER_COUNT_MAX_KEY.to_owned(),
            value: REPOSITORY_MAX,
        }],
    );
    let mut facts = Reader::On(&store, &registry, Test_Context());

    let resolved = Resolve_Limit(&mut facts, None, PARAMETER_COUNT_MAX_KEY, MAX_VALUE_PARAMETERS);

    assert_eq!(resolved, REPOSITORY_MAX);
}

#[test]
fn Test_Resolve_Limit_Should_Prefer_The_Language_Row_Over_The_Repository_Wide_Row()
{
    const REPOSITORY_MAX: u32 = 4;
    const GO_MAX: u32 = 6;
    let TestOffering { mut store, registry, offer } = Limits_Offering();
    let rows = Repository_And_Go_Rows(REPOSITORY_MAX, GO_MAX);
    Materialize_Limits_Fact(&mut store, &offer, rows);
    let mut facts = Reader::On(&store, &registry, Test_Context());

    let resolved = Resolve_Limit(&mut facts, Some(GO), PARAMETER_COUNT_MAX_KEY, MAX_VALUE_PARAMETERS);

    assert_eq!(resolved, GO_MAX);
}

/// The two limit rows this suite's precedence test decides between: a repository-wide
/// ceiling, and a language-specific one set above it.
fn Repository_And_Go_Rows(repository_max: u32, go_max: u32) -> Vec<nomos_cap_limits_policy::PolicyRow>
{
    return vec![
        nomos_cap_limits_policy::PolicyRow {
            scope: Scope::Repository,
            key: PARAMETER_COUNT_MAX_KEY.to_owned(),
            value: repository_max,
        },
        nomos_cap_limits_policy::PolicyRow {
            scope: Scope::Language(GO.to_owned()),
            key: PARAMETER_COUNT_MAX_KEY.to_owned(),
            value: go_max,
        },
    ];
}

fn Limits_Offering() -> TestOffering
{
    return test_support::Offering(
        OfferedProvider {
            contract: nomos_cap_limits_policy::Capability_Contract(),
            capability: nomos_cap_limits_policy::Capability(),
            version: nomos_cap_limits_policy::CONTRACT_VERSION,
            provider: "nomos.test.function-shape.limits.provides",
            guarantee: nomos_cap_limits_policy::Ceiling(),
        },
    ).expect("a fresh Registry holds neither this contract nor this provider");
}

fn Materialize_Limits_Fact(store: &mut MemoryFactStore, offer: &ProviderOffer, rows: Vec<nomos_cap_limits_policy::PolicyRow>)
{
    let payload = nomos_cap_limits_policy::LimitsPolicyPayload { rows };
    test_support::Materialize(
        store,
        FactToFile {
            subject: nomos_model::Subject_Of_Path(""),
            offer,
            semantic_inputs: InputDigest::Of(&[]),
            schema: nomos_cap_limits_policy::Payload_Schema(),
            bytes: nomos_cap_limits_policy::Encode_Payload(&payload),
        },
    ).expect("the fixture's store holds no fact under this key at a newer generation");
}

fn Payload_From_Text(text: &str) -> SyntaxPayload
{
    return nomos_cap_syntax::Parse_Payload(text.as_bytes())
        .expect("this fixture payload is well formed");
}

/// The one-line syntax payload naming `Build` with five free parameters — the fact every
/// rule in this module that reads a real syntax payload materializes.
const FIVE_PARAMETER_BUILD_FACT: &str = "unexpanded\t0\nitem\t0\tFunction\tPublic\tBuild\t.\t+fn/5\n";

/// A declared-and-offered syntax capability with [`FIVE_PARAMETER_BUILD_FACT`] materialized
/// against `source`, ready for a rule that reads a real syntax payload — the setup the three
/// reading tests here share, so each states only its own check and what it expects from it.
fn Offering_With_A_Five_Parameter_Build(source: &SourceFile) -> TestOffering
{
    let TestOffering { mut store, registry, offer } = Offering();
    let inputs = InputDigest::Of(&[source.text.as_bytes()]);
    test_support::Materialize(
        &mut store,
        FactToFile {
            subject: source.subject,
            offer: &offer,
            semantic_inputs: inputs,
            schema: nomos_cap_syntax::Payload_Schema(),
            bytes: FIVE_PARAMETER_BUILD_FACT.as_bytes().to_vec(),
        },
    ).expect("the fixture's store holds no fact under this key at a newer generation");

    return TestOffering { store, registry, offer };
}

/// `path` and `text` are both `&str`; without a distinct type per position a call site like
/// `Source("src/lib.rs", "fn Clean() {}")` reads as two interchangeable strings and a swap
/// compiles silently. These wrappers give each position a type the other cannot satisfy.
#[derive(Clone, Copy)]
struct Path<'a>(&'a str);

#[derive(Clone, Copy)]
struct Text<'a>(&'a str);

fn Source(path: Path<'_>, text: Text<'_>) -> SourceFile
{
    let mut source = SourceFile::New(
        path.0,
        nomos_contracts::SubjectId::From_Digest(nomos_model::Content_Digest(path.0.as_bytes())),
        text.0,
    );
    source.language = crate::Recognized_Language_In_Tests(path.0);
    return source;
}

fn Offering() -> TestOffering
{
    let guarantee = Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Unknown, IncrementalGranularity::File);
    return test_support::Offering(
        OfferedProvider {
            contract: nomos_cap_syntax::Capability_Contract(),
            capability: nomos_cap_syntax::Capability(),
            version: nomos_cap_syntax::CONTRACT_VERSION,
            provider: "nomos.test.parameter-count.parses",
            guarantee,
        },
    ).expect("a fresh Registry holds neither this contract nor this provider");
}

fn Parameter_Count_Policy() -> FunctionArityPolicy
{
    return FunctionArityPolicy::New(PARAMETER_COUNT, MAX_VALUE_PARAMETERS).Allow_One_Receiver_For_Qualified_Functions();
}
