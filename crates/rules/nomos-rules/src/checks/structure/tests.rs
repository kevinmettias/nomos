use super::*;
use crate::checks::test_support::{self, FactToFile, OfferedProvider, TestOffering};
use nomos_analysis::{MemoryFactStore, Reader};
use nomos_cap_limits_policy::{Encode_Payload, LimitsPolicyPayload, PolicyRow};
use nomos_capability::Registry;
use nomos_contracts::SubjectId;
use nomos_model::Content_Digest;

#[test]
fn Test_Check_File_Size_Review_Trigger_Should_Report_A_File_Over_500_Lines()
{
    let source = Source("src/large.rs", Lines(REVIEW_TRIGGER_LINES + 1));
    let StoreAndRegistry { store, registry } = Empty_Store_And_Registry();
    let mut facts = Facts_Reader(&store, &registry);

    let findings = Check_File_Size_Review_Trigger(&[source], &mut facts);

    assert_eq!(findings.len(), 1, "{findings:?}");
    let found = findings.first().expect("asserted len 1 above");
    assert_eq!(found.rule, RuleId::New(FILE_SIZE_REVIEW_TRIGGER));
    assert!(found.summary.contains("501 lines"), "{}", found.summary);
}

#[test]
fn Test_Check_File_Size_Review_Trigger_Should_Accept_A_File_At_500_Lines()
{
    let source = Source("src/medium.rs", Lines(REVIEW_TRIGGER_LINES));
    let StoreAndRegistry { store, registry } = Empty_Store_And_Registry();
    let mut facts = Facts_Reader(&store, &registry);

    let findings = Check_File_Size_Review_Trigger(&[source], &mut facts);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_File_Size_Justification_Trigger_Should_Report_A_File_Over_1500_Lines()
{
    let source = Source("src/huge.rs", Lines(JUSTIFICATION_TRIGGER_LINES + 1));
    let StoreAndRegistry { store, registry } = Empty_Store_And_Registry();
    let mut facts = Facts_Reader(&store, &registry);

    let findings = Check_File_Size_Justification_Trigger(&[source], &mut facts);

    assert_eq!(findings.len(), 1, "{findings:?}");
    let found = findings.first().expect("asserted len 1 above");
    assert_eq!(found.rule, RuleId::New(FILE_SIZE_JUSTIFICATION_TRIGGER));
    assert!(found.summary.contains("1501 lines"), "{}", found.summary);
}

#[test]
fn Test_Check_File_Size_Justification_Trigger_Should_Accept_A_File_At_1500_Lines()
{
    let source = Source("src/large.rs", Lines(JUSTIFICATION_TRIGGER_LINES));
    let StoreAndRegistry { store, registry } = Empty_Store_And_Registry();
    let mut facts = Facts_Reader(&store, &registry);

    let findings = Check_File_Size_Justification_Trigger(&[source], &mut facts);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Go_File_Size_Review_Trigger_Should_Report_A_Go_File_Over_500_Lines()
{
    Assert_Reports_One_Go_File_Size_Finding(Check_Go_File_Size_Review_Trigger, GO_REVIEW_TRIGGER_LINES, FIVE_HUNDRED_LINE_REVIEW_TRIGGER);
}

#[test]
fn Test_Check_Go_File_Size_Review_Trigger_Should_Ignore_Non_Go_Files()
{
    let source = Source("src/large.rs", Lines(GO_REVIEW_TRIGGER_LINES + 1));
    let StoreAndRegistry { store, registry } = Empty_Store_And_Registry();
    let mut facts = Facts_Reader(&store, &registry);

    let findings = Check_Go_File_Size_Review_Trigger(&[source], &mut facts);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Go_File_Size_Hard_Trigger_Should_Report_A_Go_File_Over_1000_Lines()
{
    Assert_Reports_One_Go_File_Size_Finding(Check_Go_File_Size_Hard_Trigger, GO_HARD_TRIGGER_LINES, ONE_THOUSAND_LINE_HARD_TRIGGER);
}

#[test]
fn Test_Check_Go_File_Size_Hard_Trigger_Should_Accept_A_Go_File_At_1000_Lines()
{
    let source = Source("index.go", Lines(GO_HARD_TRIGGER_LINES));
    let StoreAndRegistry { store, registry } = Empty_Store_And_Registry();
    let mut facts = Facts_Reader(&store, &registry);

    let findings = Check_Go_File_Size_Hard_Trigger(&[source], &mut facts);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_No_Mod_Rs_Files_Should_Report_A_Mod_File_Under_Source()
{
    let source = Source("src/pipeline/mod.rs", String::new());

    let findings = Check_No_Mod_Rs_Files(&[source]);

    assert_eq!(findings.len(), 1, "{findings:?}");
    let found = findings.first().expect("asserted len 1 above");
    assert_eq!(found.rule, RuleId::New(NO_MOD_RS_FILES));
    assert_eq!(found.gate, GateCategory::Blocking);
}

#[test]
fn Test_Check_No_Mod_Rs_Files_Should_Allow_Shared_Test_Modules()
{
    let source = Source("tests/common/mod.rs", String::new());

    let findings = Check_No_Mod_Rs_Files(&[source]);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Resolve_Limit_Should_Fall_Back_To_The_Default_When_No_Fact_Is_Materialized()
{
    let StoreAndRegistry { store, registry } = Empty_Store_And_Registry();
    let mut facts = Facts_Reader(&store, &registry);

    let resolved = Resolve_Limit(&mut facts, None, FILE_SIZE_HARD_LINES_KEY, JUSTIFICATION_TRIGGER_LINES);

    assert_eq!(resolved, JUSTIFICATION_TRIGGER_LINES);
}

#[test]
fn Test_Resolve_Limit_Should_Prefer_The_Repository_Wide_Row_Over_The_Default()
{
    const REPOSITORY_OVERRIDE_LINES: u32 = 900;

    let TestOffering { mut store, registry, offer } = Limits_Offering();
    Materialize_Limits_Fact(
        &mut store,
        &offer,
        vec![PolicyRow { scope: Scope::Repository, key: FILE_SIZE_HARD_LINES_KEY.to_owned(), value: REPOSITORY_OVERRIDE_LINES }],
    );
    let mut facts = Facts_Reader(&store, &registry);

    let resolved = Resolve_Limit(&mut facts, None, FILE_SIZE_HARD_LINES_KEY, JUSTIFICATION_TRIGGER_LINES);

    assert_eq!(resolved, usize::try_from(REPOSITORY_OVERRIDE_LINES).expect("test literal fits in usize"));
}

#[test]
fn Test_Resolve_Limit_Should_Prefer_The_Language_Row_Over_The_Repository_Wide_Row()
{
    const REPOSITORY_ROW_LINES: u32 = 1500;
    const LANGUAGE_ROW_LINES: u32 = 1000;

    let TestOffering { mut store, registry, offer } = Limits_Offering();
    Materialize_Limits_Fact(
        &mut store,
        &offer,
        vec![
            PolicyRow { scope: Scope::Repository, key: FILE_SIZE_HARD_LINES_KEY.to_owned(), value: REPOSITORY_ROW_LINES },
            PolicyRow { scope: Scope::Language(GO.to_owned()), key: FILE_SIZE_HARD_LINES_KEY.to_owned(), value: LANGUAGE_ROW_LINES },
        ],
    );
    let mut facts = Facts_Reader(&store, &registry);

    let resolved = Resolve_Limit(&mut facts, Some(GO), FILE_SIZE_HARD_LINES_KEY, JUSTIFICATION_TRIGGER_LINES);

    assert_eq!(resolved, usize::try_from(LANGUAGE_ROW_LINES).expect("test literal fits in usize"));
}

#[test]
fn Test_Check_File_Size_Review_Trigger_Should_Honor_A_Real_Materialized_Override()
{
    const OVERRIDE_CEILING_LINES: u32 = 10;

    let TestOffering { mut store, registry, offer } = Limits_Offering();
    Materialize_Limits_Fact(
        &mut store,
        &offer,
        vec![PolicyRow { scope: Scope::Repository, key: FILE_SIZE_REVIEW_LINES_KEY.to_owned(), value: OVERRIDE_CEILING_LINES }],
    );
    let mut facts = Facts_Reader(&store, &registry);
    let source = Source("src/small.rs", Lines(usize::try_from(OVERRIDE_CEILING_LINES).expect("test literal fits in usize") + 1));

    let findings = Check_File_Size_Review_Trigger(&[source], &mut facts);

    assert_eq!(findings.len(), 1, "a repository declaring a 10-line ceiling must judge an 11-line file against it: {findings:?}");
}

/// The shape [`Test_Check_Go_File_Size_Review_Trigger_Should_Report_A_Go_File_Over_500_Lines`]
/// and [`Test_Check_Go_File_Size_Hard_Trigger_Should_Report_A_Go_File_Over_1000_Lines`] both
/// need: a `.go` file one line over `threshold`, judged by `check`, reporting exactly one
/// finding under `rule`.
fn Assert_Reports_One_Go_File_Size_Finding(check: fn(&[SourceFile], &mut dyn FactReader) -> Vec<Finding>, threshold: usize, rule: &'static str)
{
    let source = Source("index.go", Lines(threshold.saturating_add(1)));
    let StoreAndRegistry { store, registry } = Empty_Store_And_Registry();
    let mut facts = Facts_Reader(&store, &registry);

    let findings = check(&[source], &mut facts);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(rule));
}

/// What one of the four line-count triggers does with a file at `path` carrying exactly
/// `threshold + 1` lines, read through an empty fact store and registry.
///
/// `pub(super)` because the two `self_tests` modules need it as well: their tests must live in
/// the same physical file as the function they address, and this fixture is the one thing every
/// one of those tests needs and none of them cares about — stated once here rather than
/// near-copied into `structure.rs` and `structure/go_file_size.rs`, which is what the
/// duplication check reported before it was.
pub(super) fn Fixture_Over_Threshold(check: fn(&[SourceFile], &mut dyn FactReader) -> Vec<Finding>, path: &str, threshold: usize) -> Vec<Finding>
{
    let source = Source(path, Lines(threshold.saturating_add(1)));
    let StoreAndRegistry { store, registry } = Empty_Store_And_Registry();
    let mut facts = Facts_Reader(&store, &registry);

    return check(&[source], &mut facts);
}

/// An empty fact store paired with an empty capability registry, named so the pair
/// cannot be swapped the way an unnamed tuple invites.
struct StoreAndRegistry
{
    store: MemoryFactStore,
    registry: Registry,
}

/// The setup every test in this file needs and no test cares about: an empty fact
/// store paired with an empty capability registry, read through a fresh [`Reader`].
fn Empty_Store_And_Registry() -> StoreAndRegistry
{
    return StoreAndRegistry { store: MemoryFactStore::New(), registry: Registry::New() };
}

/// Every test in this file builds its [`Reader`] the same one way, whether `store` and
/// `registry` came from [`Empty_Store_And_Registry`] or from a materialized
/// [`TestOffering`] — one place to say what "reading" means in this file's tests.
fn Facts_Reader<'store, 'registry>(store: &'store MemoryFactStore, registry: &'registry Registry) -> Reader<'store, 'registry>
{
    return Reader::On(store, registry, test_support::Test_Context());
}

fn Limits_Offering() -> TestOffering
{
    return test_support::Offered_Registry(
        OfferedProvider {
            contract: nomos_cap_limits_policy::Capability_Contract(),
            capability: nomos_cap_limits_policy::Capability(),
            version: nomos_cap_limits_policy::CONTRACT_VERSION,
            provider: "nomos.test.limits.provides",
            guarantee: nomos_cap_limits_policy::Ceiling(),
        },
    ).expect("a fresh Registry holds neither this contract nor this provider");
}

fn Materialize_Limits_Fact(store: &mut MemoryFactStore, offer: &nomos_capability::ProviderOffer, rows: Vec<PolicyRow>)
{
    let payload = LimitsPolicyPayload { rows };
    test_support::Materialize_Fact(
        store,
        FactToFile {
            subject: nomos_model::Subject_Of_Path(""),
            offer,
            semantic_inputs: nomos_analysis::InputDigest::Of(&[]),
            schema: nomos_cap_limits_policy::Payload_Schema(),
            bytes: Encode_Payload(&payload),
        },
    ).expect("the fixture's store holds no fact under this key at a newer generation");
}

fn Lines(count: usize) -> String
{
    return (0..count).map(|_| return "line").collect::<Vec<_>>().join("\n");
}

fn Source(path: &str, text: String) -> SourceFile
{
    let mut source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
    source.language = crate::Recognized_Language_In_Tests(path);
    return source;
}
