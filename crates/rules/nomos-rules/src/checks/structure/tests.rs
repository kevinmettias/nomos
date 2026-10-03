use super::*;
use crate::checks::test_support::{self, FactToFile, OfferedProvider, TestOffering};
use nomos_analysis::{MemoryFactStore, Reader};
use nomos_cap_limits_policy::{Encode_Payload, LimitsPolicyPayload, PolicyRow};
use nomos_capability::Registry;
use nomos_contracts::SubjectId;
use nomos_model::Content_Digest;

/// code-standards' own line-count triggers, stated here as what an unconfigured repository must
/// be judged against rather than read back from `rule_descriptor::policy_axis`: a test that took
/// its expectation from the declaration would pass whatever the declaration said.
pub(super) const REVIEW_TRIGGER_LINES: usize = 500;
pub(super) const JUSTIFICATION_TRIGGER_LINES: usize = 1500;
pub(super) const GO_REVIEW_TRIGGER_LINES: usize = 500;
pub(super) const GO_HARD_TRIGGER_LINES: usize = 1000;

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

    let resolved = Resolve_Limit(&mut facts, None, &FILE_SIZE_HARD_LINES);

    assert_eq!(resolved.and_then(|lines| return usize::try_from(lines).ok()), Some(JUSTIFICATION_TRIGGER_LINES));
}

#[test]
fn Test_Resolve_Limit_Should_Prefer_The_Repository_Wide_Row_Over_The_Default()
{
    const REPOSITORY_OVERRIDE_LINES: u32 = 900;

    let TestOffering { mut store, registry, offer } = Limits_Offering();
    Materialize_Limits_Fact(
        &mut store,
        &offer,
        vec![PolicyRow { scope: Scope::Repository, key: FILE_SIZE_HARD_LINES.key.to_owned(), value: REPOSITORY_OVERRIDE_LINES }],
    );
    let mut facts = Facts_Reader(&store, &registry);

    let resolved = Resolve_Limit(&mut facts, None, &FILE_SIZE_HARD_LINES);

    assert_eq!(resolved, Some(REPOSITORY_OVERRIDE_LINES));
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
            PolicyRow { scope: Scope::Repository, key: FILE_SIZE_HARD_LINES.key.to_owned(), value: REPOSITORY_ROW_LINES },
            PolicyRow { scope: Scope::Language(GO.to_owned()), key: FILE_SIZE_HARD_LINES.key.to_owned(), value: LANGUAGE_ROW_LINES },
        ],
    );
    let mut facts = Facts_Reader(&store, &registry);

    let resolved = Resolve_Limit(&mut facts, Some(GO), &FILE_SIZE_HARD_LINES);

    assert_eq!(resolved, Some(LANGUAGE_ROW_LINES));
}

/// A repository-wide row outranks Go's own default: before the axis was declared, the Go rule
/// read the repository row first and fell back to its lower 1000 only when there was none, and
/// declaring the default on the axis must not reverse that order.
#[test]
fn Test_Resolve_Limit_Should_Prefer_The_Repository_Wide_Row_Over_A_Languages_Own_Default()
{
    const REPOSITORY_ROW_LINES: u32 = 2000;

    let TestOffering { mut store, registry, offer } = Limits_Offering();
    Materialize_Limits_Fact(
        &mut store,
        &offer,
        vec![PolicyRow { scope: Scope::Repository, key: FILE_SIZE_HARD_LINES.key.to_owned(), value: REPOSITORY_ROW_LINES }],
    );
    let mut facts = Facts_Reader(&store, &registry);

    let resolved = Resolve_Limit(&mut facts, Some(GO), &FILE_SIZE_HARD_LINES);

    assert_eq!(resolved, Some(REPOSITORY_ROW_LINES));
}

#[test]
fn Test_Check_File_Size_Review_Trigger_Should_Honor_A_Real_Materialized_Override()
{
    const OVERRIDE_CEILING_LINES: u32 = 10;

    let TestOffering { mut store, registry, offer } = Limits_Offering();
    Materialize_Limits_Fact(
        &mut store,
        &offer,
        vec![PolicyRow { scope: Scope::Repository, key: FILE_SIZE_REVIEW_LINES.key.to_owned(), value: OVERRIDE_CEILING_LINES }],
    );
    let mut facts = Facts_Reader(&store, &registry);
    let source = Source("src/small.rs", Lines(usize::try_from(OVERRIDE_CEILING_LINES).expect("test literal fits in usize") + 1));

    let findings = Check_File_Size_Review_Trigger(&[source], &mut facts);

    assert_eq!(findings.len(), 1, "a repository declaring a 10-line ceiling must judge an 11-line file against it: {findings:?}");
}

// The two tests below hold Go's two line-count triggers to `OD-RULES-011` version 3 decision 4: a
// finding states the threshold it was judged against, and nothing about where that threshold came
// from. Both called it Go's, which a repository may declare for Go or for every language. Each
// judges one Go file with no threshold declared, with exactly the threshold it would otherwise judge
// against declared for Go, and with another declared; the first two must report identical findings,
// so identical identities, and the exact text is asserted.

#[test]
fn Test_Check_Go_File_Size_Review_Trigger_Should_State_The_Threshold_It_Judged_Against_And_Not_Where_It_Came_From()
{
    const DECLARED_FOR_GO: u32 = 200;
    let key = FILE_SIZE_REVIEW_LINES.key;
    let lines = GO_REVIEW_TRIGGER_LINES.saturating_add(1);
    let restating = vec![Go_Row(key, u32::try_from(GO_REVIEW_TRIGGER_LINES).expect("500 fits in u32"))];

    let undeclared = Go_File_Judged(Check_Go_File_Size_Review_Trigger, lines, Vec::new());
    let restated = Go_File_Judged(Check_Go_File_Size_Review_Trigger, lines, restating);
    let declared = Go_File_Judged(Check_Go_File_Size_Review_Trigger, lines, vec![Go_Row(key, DECLARED_FOR_GO)]);

    assert_eq!(Summaries(&undeclared), vec!["index.go has 501 lines and exceeds the 500-line review trigger for splitting"]);
    assert_eq!(restated, undeclared, "declaring the threshold the rule already judged against must not change its finding");
    assert_eq!(Summaries(&declared), vec!["index.go has 501 lines and exceeds the 200-line review trigger for splitting"]);
}

/// The declared threshold here is repository-wide, so no part of it is Go's: the case in which
/// calling it Go's was false outright.
#[test]
fn Test_Check_Go_File_Size_Hard_Trigger_Should_State_The_Threshold_It_Judged_Against_And_Not_Where_It_Came_From()
{
    const DECLARED_FOR_EVERY_LANGUAGE: u32 = 800;
    let key = FILE_SIZE_HARD_LINES.key;
    let lines = GO_HARD_TRIGGER_LINES.saturating_add(1);
    let restating = vec![Go_Row(key, u32::try_from(GO_HARD_TRIGGER_LINES).expect("1000 fits in u32"))];
    let declaring = vec![PolicyRow { scope: Scope::Repository, key: key.to_owned(), value: DECLARED_FOR_EVERY_LANGUAGE }];

    let undeclared = Go_File_Judged(Check_Go_File_Size_Hard_Trigger, lines, Vec::new());
    let restated = Go_File_Judged(Check_Go_File_Size_Hard_Trigger, lines, restating);
    let declared = Go_File_Judged(Check_Go_File_Size_Hard_Trigger, lines, declaring);

    let because = "trigger and needs decomposition or a documented locality justification";
    assert_eq!(Summaries(&undeclared), vec![format!("index.go has 1001 lines and exceeds the 1000-line {because}")]);
    assert_eq!(restated, undeclared, "declaring the threshold the rule already judged against must not change its finding");
    assert_eq!(Summaries(&declared), vec![format!("index.go has 1001 lines and exceeds the 800-line {because}")]);
}

/// One limits row declared for Go alone.
fn Go_Row(key: &str, value: u32) -> PolicyRow
{
    return PolicyRow { scope: Scope::Language(GO.to_owned()), key: key.to_owned(), value };
}

fn Summaries(findings: &[Finding]) -> Vec<String>
{
    return findings.iter().map(|finding| return finding.summary.clone()).collect();
}

/// What `check` finds in a Go file of `lines` lines, in a repository whose limits file declares
/// `rows`.
fn Go_File_Judged(check: fn(&[SourceFile], &mut dyn FactReader) -> Vec<Finding>, lines: usize, rows: Vec<PolicyRow>) -> Vec<Finding>
{
    let TestOffering { mut store, registry, offer } = Limits_Offering();
    Materialize_Limits_Fact(&mut store, &offer, rows);
    let mut facts = Facts_Reader(&store, &registry);

    return check(&[Source("index.go", Lines(lines))], &mut facts);
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
