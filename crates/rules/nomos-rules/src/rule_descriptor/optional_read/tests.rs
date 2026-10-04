//! Every rule's body held to the optional reads its row declares.

use super::recorder::Take_Resolved;
use super::OptionalRead;
use crate::checks::test_support::{self, FactToFile, OfferedProvider, TestOffering, Test_Context};
use crate::{SourceFile, UndeclaredOutcome, UndeclaredValue, DESCRIPTORS};
use nomos_analysis::{InputDigest, MemoryFactStore, Reader};
use nomos_capability::Registry;

/// One source written in `language`, so a rule that reads per language, or judges one language,
/// reaches the reads it makes for it.
fn Source_In(path: &str, language: &str) -> SourceFile
{
    let mut source = SourceFile::New(path, nomos_model::Subject_Of_Path(path), "fn main() {}\n");
    source.language = Some(nomos_cap_syntax::Language::New(language));

    return source;
}

/// What one rule's body resolved and its row did not declare, and what its row declared and its
/// body did not resolve, as one line for the failure message -- or nothing when the two agree.
fn Disagreement(rule: &str, declared: &[OptionalRead], resolved: &[OptionalRead]) -> Option<String>
{
    let undeclared: Vec<&OptionalRead> = resolved.iter().filter(|read| return !declared.contains(read)).collect();
    let unresolved: Vec<&OptionalRead> = declared.iter().filter(|read| return !resolved.contains(read)).collect();
    if undeclared.is_empty() && unresolved.is_empty()
    {
        return None;
    }

    return Some(format!("{rule}: resolved and not declared {undeclared:?}; declared and not resolved {unresolved:?}"));
}

/// `OD-RULES-011` version 3 decision 5: which optional values a rule reads is declared on its
/// descriptor as the one statement its body resolves by. Every composed rule's body is run over a
/// Rust and a Go source with no fact to read, and the reads it resolved must be exactly the ones
/// its row declares: a body resolving a read its row leaves out fails here, and so does a row
/// declaring a read its body never makes.
///
/// No fact is materialized, so every read falls back to what an undeclared value means, and every
/// rule that needs a syntax fact reports it unavailable -- after it has resolved what it reads,
/// which every body here does before it reads a source.
#[test]
fn Test_Every_Rule_Body_Should_Resolve_Exactly_The_Optional_Reads_Its_Row_Declares()
{
    let sources = vec![Source_In("src/lib.rs", "rust"), Source_In("helper/main.go", "go")];
    let store = MemoryFactStore::New();
    let registry = Registry::New();
    let mut disagreements = Vec::new();

    for descriptor in DESCRIPTORS
    {
        let _before = Take_Resolved();
        let mut facts = Reader::On(&store, &registry, Test_Context());

        let _findings = descriptor.check.Judges(&sources, &mut facts);

        disagreements.extend(Disagreement(descriptor.id, descriptor.optional_reads, &Take_Resolved()));
    }

    assert!(disagreements.is_empty(), "{disagreements:#?}");
}

/// The test above is not satisfied by rows that declare nothing: sixteen rows read an optional
/// family, the count `OD-RULES-011` version 3 measured as thirty less the fourteen that only add
/// exceptions or subjects.
#[test]
fn Test_Every_Row_That_Reads_An_Optional_Family_Should_Declare_What_It_Reads()
{
    let reading: Vec<&str> = DESCRIPTORS.iter().filter(|descriptor| return !descriptor.optional_reads.is_empty()).map(|descriptor| return descriptor.id).collect();

    assert_eq!(reading.len(), ROWS_READING_AN_OPTIONAL_FAMILY, "{reading:?}");
}

/// Rows reading a family `OD-RULES-011` version 3 decision 1 names: six limits rows, five naming
/// rows, `cyclomatic-complexity`, and the four that judge nothing for want of a norm.
const ROWS_READING_AN_OPTIONAL_FAMILY: usize = 16;

/// The row composing `rule`, which every caller here names by its exported identifier.
fn Row(rule: &str) -> &'static crate::RuleDescriptor
{
    return DESCRIPTORS.iter().find(|descriptor| return descriptor.id == rule).expect("every rule named here is composed");
}

/// A repository whose one materialized fact is `family`'s, answering `bytes`.
fn Repository_Declaring(family: OfferedProvider<'_>, schema: nomos_contracts::SchemaId, bytes: Vec<u8>) -> TestOffering
{
    let mut offering = test_support::Offered_Registry(family).expect("a fresh registry holds neither this contract nor this provider");
    test_support::Materialize_Fact(
        &mut offering.store,
        FactToFile { subject: nomos_model::Subject_Of_Path(""), offer: &offering.offer, semantic_inputs: InputDigest::Of(&[]), schema, bytes },
    )
    .expect("the fixture's store holds no fact under this key at a newer generation");

    return offering;
}

/// A repository whose `nomos-limits.json` declares exactly `rows`, repository-wide; an absent file
/// is read as declaring none.
fn Limits_Declaring(rows: &[(&str, u32)]) -> TestOffering
{
    let rows = rows
        .iter()
        .map(|(key, value)| return nomos_cap_limits_policy::PolicyRow { scope: nomos_cap_limits_policy::Scope::Repository, key: (*key).to_owned(), value: *value })
        .collect();
    let family = OfferedProvider {
        contract: nomos_cap_limits_policy::Capability_Contract(),
        capability: nomos_cap_limits_policy::Capability(),
        version: nomos_cap_limits_policy::CONTRACT_VERSION,
        provider: "nomos.test.limits.provides",
        guarantee: nomos_cap_limits_policy::Ceiling(),
    };

    return Repository_Declaring(
        family,
        nomos_cap_limits_policy::Payload_Schema(),
        nomos_cap_limits_policy::Encode_Payload(&nomos_cap_limits_policy::LimitsPolicyPayload { rows }),
    );
}

/// A repository whose `standards.json` declares exactly `rows` for naming.
fn Naming_Declaring(rows: Vec<nomos_cap_naming_policy::PolicyRow>) -> TestOffering
{
    let family = OfferedProvider {
        contract: nomos_cap_naming_policy::Capability_Contract(),
        capability: nomos_cap_naming_policy::Capability(),
        version: nomos_cap_naming_policy::CONTRACT_VERSION,
        provider: "nomos.test.naming.provides",
        guarantee: nomos_cap_naming_policy::Ceiling(),
    };

    return Repository_Declaring(
        family,
        nomos_cap_naming_policy::Payload_Schema(),
        nomos_cap_naming_policy::Encode_Payload(&nomos_cap_naming_policy::NamingPolicyPayload { rows }),
    );
}

/// A repository whose requirement-trace provider answered `payload`.
fn Trace_Answering(payload: &nomos_cap_requirement_trace::RequirementTracePayload) -> TestOffering
{
    let family = OfferedProvider {
        contract: nomos_cap_requirement_trace::Capability_Contract(),
        capability: nomos_cap_requirement_trace::Capability(),
        version: nomos_cap_requirement_trace::CONTRACT_VERSION,
        provider: "nomos.test.trace.provides",
        guarantee: nomos_cap_requirement_trace::Ceiling(),
    };

    return Repository_Declaring(family, nomos_cap_requirement_trace::Payload_Schema(), nomos_cap_requirement_trace::Encode_Payload(payload));
}

/// What `rule` names undeclared in `repository`, for a run that handed it `judged`.
fn Named(rule: &str, repository: &TestOffering, judged: &[SourceFile]) -> Vec<UndeclaredValue>
{
    let mut facts = Reader::On(&repository.store, &repository.registry, Test_Context());

    return Row(rule).Undeclared_Values(&mut facts, judged);
}

fn Limit(key: &'static str, language: Option<&str>, outcome: UndeclaredOutcome) -> UndeclaredValue
{
    return UndeclaredValue { family: "limits", declared_in: "nomos-limits.json", key, language: language.map(str::to_owned), outcome };
}

fn Judged_Against(value: &str) -> UndeclaredOutcome
{
    return UndeclaredOutcome::JudgedAgainst { value: value.to_owned() };
}

/// A limit nobody declared is named with the value the rule judged against -- the language's own
/// default for a rule reading it for one -- and an axis with no default is named reported
/// undeclared. `OD-RULES-011` version 3 decision 1.
#[test]
fn Test_Undeclared_Values_Should_Name_An_Undeclared_Limit_With_What_Its_Rule_Did()
{
    let repository = Limits_Declaring(&[]);

    assert_eq!(Named(crate::FILE_SIZE_JUSTIFICATION_TRIGGER, &repository, &[]), vec![Limit("file-size-hard-lines", None, Judged_Against("1500"))]);
    assert_eq!(Named(crate::ONE_THOUSAND_LINE_HARD_TRIGGER, &repository, &[]), vec![Limit("file-size-hard-lines", Some("go"), Judged_Against("1000"))]);
    assert_eq!(
        Named(crate::CYCLOMATIC_COMPLEXITY, &repository, &[]),
        vec![Limit("cyclomatic-complexity-max", None, UndeclaredOutcome::ReportedUndeclared)]
    );
}

/// A limit declared repository-wide is declared for every language a rule reads it for, so neither
/// trigger names it.
#[test]
fn Test_Undeclared_Values_Should_Not_Name_A_Limit_The_Repository_Declared()
{
    const DECLARED_LINES: u32 = 900;
    let repository = Limits_Declaring(&[("file-size-hard-lines", DECLARED_LINES)]);

    assert_eq!(Named(crate::FILE_SIZE_JUSTIFICATION_TRIGGER, &repository, &[]), Vec::new());
    assert_eq!(Named(crate::ONE_THOUSAND_LINE_HARD_TRIGGER, &repository, &[]), Vec::new());
}

/// A family that could not be read names nothing: that is coverage debt, reported by its own
/// finding, and not a value nobody declared.
#[test]
fn Test_Undeclared_Values_Should_Name_Nothing_From_A_Family_That_Could_Not_Be_Read()
{
    let unread = TestOffering { store: MemoryFactStore::New(), registry: Registry::New(), offer: Limits_Declaring(&[]).offer };
    let judged = [Source_In("src/lib.rs", "rust")];

    assert_eq!(Named(crate::FILE_SIZE_JUSTIFICATION_TRIGGER, &unread, &judged), Vec::new());
    assert_eq!(Named(crate::NAMING_CONVENTION, &unread, &judged), Vec::new());
    assert_eq!(Named(crate::REQUIREMENT_TRACE_STALENESS, &unread, &judged), Vec::new());
}

/// A case read in each source's own language is named once per language the rule judged -- never
/// for a language it judged no source in -- and not for a language the repository declared it for.
#[test]
fn Test_Undeclared_Values_Should_Name_A_Case_Once_For_Each_Language_It_Was_Read_In()
{
    use nomos_cap_naming_policy::{Case, PolicyRow, Scope};

    let judged = [Source_In("src/lib.rs", "rust"), Source_In("helper/main.go", "go"), Source_In("src/main.rs", "rust")];
    let function = |language: &str| {
        return UndeclaredValue {
            family: "naming",
            declared_in: "standards.json",
            key: "function",
            language: Some(language.to_owned()),
            outcome: Judged_Against("upper-snake"),
        };
    };
    let go_declared = Naming_Declaring(vec![PolicyRow { scope: Scope::Language("go".to_owned()), symbol: "function".to_owned(), case: Case::UpperCamel }]);
    let declared = Naming_Declaring(vec![PolicyRow { scope: Scope::Repository, symbol: "function".to_owned(), case: Case::UpperSnake }]);

    assert_eq!(Named(crate::NAMING_CONVENTION, &Naming_Declaring(Vec::new()), &judged), vec![function("rust"), function("go")]);
    assert_eq!(Named(crate::NAMING_CONVENTION, &Naming_Declaring(Vec::new()), &judged[..1]), vec![function("rust")]);
    assert_eq!(Named(crate::NAMING_CONVENTION, &go_declared, &judged), vec![function("rust")]);
    assert_eq!(Named(crate::NAMING_CONVENTION, &declared, &judged), Vec::new());
}

/// The requirement trace is named as having judged nothing only for a repository with no
/// requirements directory, never for a corpus whose every entry resolves. Decision 6.
#[test]
fn Test_Undeclared_Values_Should_Name_The_Requirement_Trace_Only_When_Its_Directory_Is_Absent()
{
    use nomos_cap_requirement_trace::RequirementTracePayload;

    let absent = Trace_Answering(&RequirementTracePayload { problems: Vec::new(), directory_absent: true });
    let resolved = Trace_Answering(&RequirementTracePayload::default());

    assert_eq!(
        Named(crate::REQUIREMENT_TRACE_STALENESS, &absent, &[]),
        vec![UndeclaredValue {
            family: "requirement-trace",
            declared_in: "tests/contract/requirements",
            key: "*.assessment",
            language: None,
            outcome: UndeclaredOutcome::JudgedNothing,
        }]
    );
    assert_eq!(Named(crate::REQUIREMENT_TRACE_STALENESS, &resolved, &[]), Vec::new());
}
