//! File structure rules from code-standards.
//!
//! The standards define two line-count triggers: files above roughly 500 lines are review
//! candidates for splitting, and files above roughly 1500 lines must carry an explicit
//! justification. `no-mod-rs-files` is text-independent: the path alone decides whether a
//! source file uses the old Rust module layout, so [`Check_No_Mod_Rs_Files`] alone still
//! takes only [`SourceFile`]s. Go carries its own, lower pair of the same two triggers — 500
//! lines for review and 1000 lines (not 1500) for the hard trigger — under their own rule
//! ids, scoped to `.go` sources.
//!
//! # The thresholds are now a repository's own, resolved rather than compiled in
//!
//! `OD-RULES-011` named this threshold family as its own future instance of the naming
//! decision `checks::naming::Resolve_Case` already generalizes: [`Resolve_Limit`] asks
//! `nomos.cap.limits.policy` for the line count a repository declares, falling back to each
//! rule's own prior hardcoded default when a repository declares none — the identical
//! `Require`-then-fall-back-on-any-`Err` shape, since `OD-CAPABILITY-004`/`OD-RULES-011`
//! settle that this capability is optional the same way naming's is. The thresholds named
//! above are those defaults, still real and still what an unconfigured repository gets.

use crate::{GO_LANGUAGE, SourceFile};
use nomos_analysis::{FactReader, InputDigest};
use nomos_cap_limits_policy::Scope;
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// The code-standards review-trigger rule id.
pub const FILE_SIZE_REVIEW_TRIGGER: &str = "500-lines";
/// The code-standards hard-trigger rule id.
pub const FILE_SIZE_JUSTIFICATION_TRIGGER: &str = "1500-lines";
/// The code-standards Rust module-layout rule id.
pub const NO_MOD_RS_FILES: &str = "no-mod-rs-files";
/// The code-standards Go review-trigger rule id.
pub const FIVE_HUNDRED_LINE_REVIEW_TRIGGER: &str = "five-hundred-line-review-trigger";
/// The code-standards Go hard-trigger rule id.
pub const ONE_THOUSAND_LINE_HARD_TRIGGER: &str = "one-thousand-line-hard-trigger";

const REVIEW_TRIGGER_LINES: usize = 500;
const JUSTIFICATION_TRIGGER_LINES: usize = 1500;
const GO_REVIEW_TRIGGER_LINES: usize = 500;
const GO_HARD_TRIGGER_LINES: usize = 1000;

/// `standards.json`'s row key for the review trigger, shared by the generic and the Go
/// rule since Go's own value equals the default and so needs no override row.
const FILE_SIZE_REVIEW_LINES_KEY: &str = "file-size-review-lines";
/// `standards.json`'s row key for the hard/justification trigger, where Go's own value
/// differs from the default and so is declared as a `languages.go.limits` override.
const FILE_SIZE_HARD_LINES_KEY: &str = "file-size-hard-lines";

const GO: &str = "go";

/// Reports files whose line count exceeds the review trigger.
#[must_use]
pub fn Check_File_Size_Review_Trigger(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let threshold = Resolve_Limit(facts, None, FILE_SIZE_REVIEW_LINES_KEY, REVIEW_TRIGGER_LINES);
    return Findings_For_Threshold(
        sources,
        FILE_SIZE_REVIEW_TRIGGER,
        threshold,
        &format!("exceeds the {threshold}-line review trigger for splitting"),
        |_| return true,
    );
}

/// Reports files whose line count exceeds the hard justification trigger.
#[must_use]
pub fn Check_File_Size_Justification_Trigger(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let threshold = Resolve_Limit(facts, None, FILE_SIZE_HARD_LINES_KEY, JUSTIFICATION_TRIGGER_LINES);
    return Findings_For_Threshold(
        sources,
        FILE_SIZE_JUSTIFICATION_TRIGGER,
        threshold,
        &format!("exceeds the {threshold}-line trigger and needs an explicit splitting justification"),
        |_| return true,
    );
}

/// Reports Go files whose line count exceeds Go's own, lower review trigger.
#[must_use]
pub fn Check_Go_File_Size_Review_Trigger(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let threshold = Resolve_Limit(facts, Some(GO), FILE_SIZE_REVIEW_LINES_KEY, GO_REVIEW_TRIGGER_LINES);
    return Findings_For_Threshold(
        sources,
        FIVE_HUNDRED_LINE_REVIEW_TRIGGER,
        threshold,
        &format!("exceeds Go's {threshold}-line review trigger for splitting"),
        |source| return source.Is_Written_In(GO_LANGUAGE),
    );
}

/// Reports Go files whose line count exceeds Go's own, lower hard trigger.
#[must_use]
pub fn Check_Go_File_Size_Hard_Trigger(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let threshold = Resolve_Limit(facts, Some(GO), FILE_SIZE_HARD_LINES_KEY, GO_HARD_TRIGGER_LINES);
    return Findings_For_Threshold(
        sources,
        ONE_THOUSAND_LINE_HARD_TRIGGER,
        threshold,
        &format!("exceeds Go's {threshold}-line trigger and needs decomposition or a documented locality justification"),
        |source| return source.Is_Written_In(GO_LANGUAGE),
    );
}

/// This crate's own floor for `nomos.cap.limits.policy` — stated at the capability's own
/// ceiling since there is only one real provider today and no weaker answer this crate
/// could honestly still act on. Mirrors `checks::naming::Naming_Policy_Requirement`
/// exactly, for the sibling capability.
fn Limits_Policy_Requirement() -> nomos_capability::Requirement
{
    return nomos_capability::Requirement::New(
        nomos_cap_limits_policy::Capability(),
        nomos_cap_limits_policy::CONTRACT_VERSION,
        nomos_cap_limits_policy::Ceiling(),
    );
}

/// Resolves the numeric threshold `key` must take: a repository's own declared
/// `nomos.cap.limits.policy`, most-specific key first (`language`'s own override, then the
/// repository-wide default), falling back to `default` when neither is declared.
///
/// `OD-CAPABILITY-004` and `OD-RULES-011` settle how an absent read is treated here,
/// mirroring `checks::naming::Resolve_Case` exactly: this capability is optional, every
/// caller already has a complete answer without it, so `facts.Require` failing for any
/// reason is exactly "no override" — never a `Finding`, never this capability's own
/// `Applicability` surfacing anywhere.
fn Resolve_Limit(facts: &mut dyn FactReader, language: Option<&str>, key: &str, default: usize) -> usize
{
    let subject = nomos_model::Subject_Of_Path("");
    let Ok(fact) =
        facts.Require(&nomos_cap_limits_policy::Capability(), &subject, InputDigest::Of(&[]), &Limits_Policy_Requirement())
    else
    {
        return default;
    };

    let Ok(payload) = nomos_cap_limits_policy::Parse_Payload(&fact.payload.bytes)
    else
    {
        return default;
    };

    if let Some(language) = language
    {
        let scope = Scope::Language(language.to_owned());
        if let Some(row) = payload.rows.iter().find(|row| return row.scope == scope && row.key == key)
        {
            return usize::try_from(row.value).unwrap_or(default);
        }
    }

    if let Some(row) = payload.rows.iter().find(|row| return row.scope == Scope::Repository && row.key == key)
    {
        return usize::try_from(row.value).unwrap_or(default);
    }

    return default;
}

/// Reports `src/**/mod.rs` files, exempting shared integration-test modules under `tests/`.
#[must_use]
pub fn Check_No_Mod_Rs_Files(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if Is_Disallowed_Mod_Rs(&source.path)
        {
            findings.push(Finding_For_Source(
                source,
                NO_MOD_RS_FILES,
                "uses the old Rust module layout; use a sibling module file instead",
            ));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Findings_For_Threshold(
    sources: &[SourceFile],
    rule: &str,
    threshold: usize,
    because: &str,
    accepts_source: impl Fn(&SourceFile) -> bool,
) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if !accepts_source(source)
        {
            continue;
        }

        let line_count = Line_Count(source);
        if line_count > threshold
        {
            findings.push(Finding_For_Source_With_Count(source, rule, line_count, because));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Line_Count(source: &SourceFile) -> usize
{
    return source.text.lines().count();
}

fn Is_Disallowed_Mod_Rs(path: &str) -> bool
{
    let normalized = path.replace('\\', "/");
    return normalized.ends_with("/mod.rs") && normalized.starts_with("src/");
}

fn Finding_For_Source_With_Count(source: &SourceFile, rule: &str, line_count: usize, because: &str) -> Finding
{
    return Finding {
        rule: RuleId::New(rule),
        subject: source.subject,
        subject_name: source.path.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!("{} has {line_count} lines and {because}", source.path),
        locations: vec![source.path.clone()],
    };
}

fn Finding_For_Source(source: &SourceFile, rule: &str, because: &str) -> Finding
{
    return Finding {
        rule: RuleId::New(rule),
        subject: source.subject,
        subject_name: source.path.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!("{} {because}", source.path),
        locations: vec![source.path.clone()],
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::checks::test_support::{self, TestOffering};
    use nomos_analysis::{MemoryFactStore, Reader};
    use nomos_cap_limits_policy::{Encode_Payload, LimitsPolicyPayload, PolicyRow};
    use nomos_capability::Registry;
    use nomos_contracts::SubjectId;
    use nomos_model::Content_Digest;

    #[test]
    fn Test_Check_File_Size_Review_Trigger_Should_Report_A_File_Over_500_Lines()
    {
        let source = Source("src/large.rs", Lines(501));
        let store = MemoryFactStore::New();
        let registry = Registry::New();
        let mut facts = Reader::On(&store, &registry, test_support::Test_Context());

        let findings = Check_File_Size_Review_Trigger(&[source], &mut facts);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.rule, RuleId::New(FILE_SIZE_REVIEW_TRIGGER));
        assert!(found.summary.contains("501 lines"), "{}", found.summary);
    }

    #[test]
    fn Test_Check_File_Size_Review_Trigger_Should_Accept_A_File_At_500_Lines()
    {
        let source = Source("src/medium.rs", Lines(500));
        let store = MemoryFactStore::New();
        let registry = Registry::New();
        let mut facts = Reader::On(&store, &registry, test_support::Test_Context());

        let findings = Check_File_Size_Review_Trigger(&[source], &mut facts);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_File_Size_Justification_Trigger_Should_Report_A_File_Over_1500_Lines()
    {
        let source = Source("src/huge.rs", Lines(1501));
        let store = MemoryFactStore::New();
        let registry = Registry::New();
        let mut facts = Reader::On(&store, &registry, test_support::Test_Context());

        let findings = Check_File_Size_Justification_Trigger(&[source], &mut facts);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.rule, RuleId::New(FILE_SIZE_JUSTIFICATION_TRIGGER));
        assert!(found.summary.contains("1501 lines"), "{}", found.summary);
    }

    #[test]
    fn Test_Check_File_Size_Justification_Trigger_Should_Accept_A_File_At_1500_Lines()
    {
        let source = Source("src/large.rs", Lines(1500));
        let store = MemoryFactStore::New();
        let registry = Registry::New();
        let mut facts = Reader::On(&store, &registry, test_support::Test_Context());

        let findings = Check_File_Size_Justification_Trigger(&[source], &mut facts);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Go_File_Size_Review_Trigger_Should_Report_A_Go_File_Over_500_Lines()
    {
        let source = Source("index.go", Lines(501));
        let store = MemoryFactStore::New();
        let registry = Registry::New();
        let mut facts = Reader::On(&store, &registry, test_support::Test_Context());

        let findings = Check_Go_File_Size_Review_Trigger(&[source], &mut facts);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(FIVE_HUNDRED_LINE_REVIEW_TRIGGER));
    }

    #[test]
    fn Test_Check_Go_File_Size_Review_Trigger_Should_Ignore_Non_Go_Files()
    {
        let source = Source("src/large.rs", Lines(501));
        let store = MemoryFactStore::New();
        let registry = Registry::New();
        let mut facts = Reader::On(&store, &registry, test_support::Test_Context());

        let findings = Check_Go_File_Size_Review_Trigger(&[source], &mut facts);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Go_File_Size_Hard_Trigger_Should_Report_A_Go_File_Over_1000_Lines()
    {
        let source = Source("index.go", Lines(1001));
        let store = MemoryFactStore::New();
        let registry = Registry::New();
        let mut facts = Reader::On(&store, &registry, test_support::Test_Context());

        let findings = Check_Go_File_Size_Hard_Trigger(&[source], &mut facts);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(ONE_THOUSAND_LINE_HARD_TRIGGER));
    }

    #[test]
    fn Test_Check_Go_File_Size_Hard_Trigger_Should_Accept_A_Go_File_At_1000_Lines()
    {
        let source = Source("index.go", Lines(1000));
        let store = MemoryFactStore::New();
        let registry = Registry::New();
        let mut facts = Reader::On(&store, &registry, test_support::Test_Context());

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
        let store = MemoryFactStore::New();
        let registry = Registry::New();
        let mut facts = Reader::On(&store, &registry, test_support::Test_Context());

        let resolved = Resolve_Limit(&mut facts, None, FILE_SIZE_HARD_LINES_KEY, JUSTIFICATION_TRIGGER_LINES);

        assert_eq!(resolved, JUSTIFICATION_TRIGGER_LINES);
    }

    #[test]
    fn Test_Resolve_Limit_Should_Prefer_The_Repository_Wide_Row_Over_The_Default()
    {
        let TestOffering { mut store, registry, offer } = Limits_Offering();
        Materialize_Limits_Fact(
            &mut store,
            &offer,
            vec![PolicyRow { scope: Scope::Repository, key: FILE_SIZE_HARD_LINES_KEY.to_owned(), value: 900 }],
        );
        let mut facts = Reader::On(&store, &registry, test_support::Test_Context());

        let resolved = Resolve_Limit(&mut facts, None, FILE_SIZE_HARD_LINES_KEY, JUSTIFICATION_TRIGGER_LINES);

        assert_eq!(resolved, 900);
    }

    #[test]
    fn Test_Resolve_Limit_Should_Prefer_The_Language_Row_Over_The_Repository_Wide_Row()
    {
        let TestOffering { mut store, registry, offer } = Limits_Offering();
        Materialize_Limits_Fact(
            &mut store,
            &offer,
            vec![
                PolicyRow { scope: Scope::Repository, key: FILE_SIZE_HARD_LINES_KEY.to_owned(), value: 1500 },
                PolicyRow { scope: Scope::Language(GO.to_owned()), key: FILE_SIZE_HARD_LINES_KEY.to_owned(), value: 1000 },
            ],
        );
        let mut facts = Reader::On(&store, &registry, test_support::Test_Context());

        let resolved = Resolve_Limit(&mut facts, Some(GO), FILE_SIZE_HARD_LINES_KEY, JUSTIFICATION_TRIGGER_LINES);

        assert_eq!(resolved, 1000);
    }

    #[test]
    fn Test_Check_File_Size_Review_Trigger_Should_Honor_A_Real_Materialized_Override()
    {
        let TestOffering { mut store, registry, offer } = Limits_Offering();
        Materialize_Limits_Fact(
            &mut store,
            &offer,
            vec![PolicyRow { scope: Scope::Repository, key: FILE_SIZE_REVIEW_LINES_KEY.to_owned(), value: 10 }],
        );
        let mut facts = Reader::On(&store, &registry, test_support::Test_Context());
        let source = Source("src/small.rs", Lines(11));

        let findings = Check_File_Size_Review_Trigger(&[source], &mut facts);

        assert_eq!(findings.len(), 1, "a repository declaring a 10-line ceiling must judge an 11-line file against it: {findings:?}");
    }

    fn Limits_Offering() -> TestOffering
    {
        return test_support::Offering(
            nomos_cap_limits_policy::Capability_Contract(),
            nomos_cap_limits_policy::Capability(),
            nomos_cap_limits_policy::CONTRACT_VERSION,
            "nomos.test.limits.provides",
            nomos_cap_limits_policy::Ceiling(),
        );
    }

    fn Materialize_Limits_Fact(store: &mut MemoryFactStore, offer: &nomos_capability::ProviderOffer, rows: Vec<PolicyRow>)
    {
        let payload = LimitsPolicyPayload { rows };
        test_support::Materialize(
            store,
            nomos_model::Subject_Of_Path(""),
            offer,
            nomos_analysis::InputDigest::Of(&[]),
            nomos_cap_limits_policy::Payload_Schema(),
            Encode_Payload(&payload),
        );
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
}
