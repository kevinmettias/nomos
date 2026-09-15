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
use nomos_cap_limits_policy::{PolicyRow, Scope};
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
    let because = format!("exceeds the {threshold}-line review trigger for splitting");
    return Findings_For_Threshold(
        sources,
        LineThreshold { rule: FILE_SIZE_REVIEW_TRIGGER, lines: threshold, because: &because },
        |_| return true,
    );
}

/// Reports files whose line count exceeds the hard justification trigger.
#[must_use]
pub fn Check_File_Size_Justification_Trigger(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let threshold = Resolve_Limit(facts, None, FILE_SIZE_HARD_LINES_KEY, JUSTIFICATION_TRIGGER_LINES);
    let because = format!("exceeds the {threshold}-line trigger and needs an explicit splitting justification");
    return Findings_For_Threshold(
        sources,
        LineThreshold { rule: FILE_SIZE_JUSTIFICATION_TRIGGER, lines: threshold, because: &because },
        |_| return true,
    );
}

/// Reports Go files whose line count exceeds Go's own, lower review trigger.
#[must_use]
pub fn Check_Go_File_Size_Review_Trigger(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let threshold = Resolve_Limit(facts, Some(GO), FILE_SIZE_REVIEW_LINES_KEY, GO_REVIEW_TRIGGER_LINES);
    let because = format!("exceeds Go's {threshold}-line review trigger for splitting");
    return Findings_For_Threshold(
        sources,
        LineThreshold { rule: FIVE_HUNDRED_LINE_REVIEW_TRIGGER, lines: threshold, because: &because },
        |source| return source.Is_Written_In(GO_LANGUAGE),
    );
}

/// Reports Go files whose line count exceeds Go's own, lower hard trigger.
#[must_use]
pub fn Check_Go_File_Size_Hard_Trigger(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let threshold = Resolve_Limit(facts, Some(GO), FILE_SIZE_HARD_LINES_KEY, GO_HARD_TRIGGER_LINES);
    let because = format!("exceeds Go's {threshold}-line trigger and needs decomposition or a documented locality justification");
    return Findings_For_Threshold(
        sources,
        LineThreshold { rule: ONE_THOUSAND_LINE_HARD_TRIGGER, lines: threshold, because: &because },
        |source| return source.Is_Written_In(GO_LANGUAGE),
    );
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
            let finding = Finding_For_Source(
                source,
                Rule(NO_MOD_RS_FILES),
                Because("uses the old Rust module layout; use a sibling module file instead"),
            );
            findings.push(finding);
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Is_Disallowed_Mod_Rs(path: &str) -> bool
{
    let normalized = path.replace('\\', "/");
    return normalized.ends_with("/mod.rs") && normalized.starts_with("src/");
}

/// `rule` and `because` are both `&str`; without a distinct type per position, a call site
/// like `Finding_For_Source(source, rule, because)` reads as two interchangeable strings and
/// a swap compiles silently.
struct Rule<'a>(&'a str);
struct Because<'a>(&'a str);

fn Finding_For_Source(source: &SourceFile, rule: Rule<'_>, because: Because<'_>) -> Finding
{
    return Finding {
        rule: RuleId::New(rule.0),
        subject: source.subject,
        subject_name: source.path.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!("{} {}", source.path, because.0),
        locations: vec![source.path.clone()],
    };
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
pub(super) fn Resolve_Limit(facts: &mut dyn FactReader, language: Option<&str>, key: &str, default: usize) -> usize
{
    let Some(payload) = Materialized_Limits_Payload(facts)
    else
    {
        return default;
    };

    if let Some(language) = language
    {
        if let Some(row) = Matching_Row(&payload, Scope::Language(language.to_owned()), key)
        {
            return usize::try_from(row.value).unwrap_or(default);
        }
    }

    if let Some(row) = Matching_Row(&payload, Scope::Repository, key)
    {
        return usize::try_from(row.value).unwrap_or(default);
    }

    return default;
}

/// The fact `Resolve_Limit` reads, decoded — `None` for both "no repository declares this
/// capability" and "the declared payload does not parse," the same "any `Err` is just no
/// override" reading the function's own doc comment settles.
fn Materialized_Limits_Payload(facts: &mut dyn FactReader) -> Option<nomos_cap_limits_policy::LimitsPolicyPayload>
{
    let subject = nomos_model::Subject_Of_Path("");
    let Ok(fact) =
        facts.Require(&nomos_cap_limits_policy::Capability(), &subject, InputDigest::Of(&[]), &Limits_Policy_Requirement())
    else
    {
        return None;
    };

    return nomos_cap_limits_policy::Parse_Payload(&fact.payload.bytes).ok();
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

/// The one row in `payload` declared for exactly `scope` and `key` — the shape both the
/// language-specific and repository-wide lookups in [`Resolve_Limit`] share.
fn Matching_Row<'a>(payload: &'a nomos_cap_limits_policy::LimitsPolicyPayload, scope: Scope, key: &str) -> Option<&'a PolicyRow>
{
    return payload.rows.iter().find(|row| return row.scope == scope && row.key == key);
}

/// A line-count threshold rule: the id reported, the count that trips it, and the sentence
/// saying what tripping it means.
///
/// The three are one parameter rather than three because they are one thing at every call
/// site -- `because` is written out of `lines`, and neither says anything without `rule` --
/// and because four separate ones put this helper over the value-parameter cap it exists to
/// help enforce. A rule reporting its own implementation is worth reading as evidence about
/// the code rather than about the rule, and here it was right.
struct LineThreshold<'a>
{
    rule: &'a str,
    lines: usize,
    because: &'a str,
}

fn Findings_For_Threshold(
    sources: &[SourceFile],
    threshold: LineThreshold<'_>,
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
        if line_count > threshold.lines
        {
            let finding = Finding_For_Source_With_Count(source, threshold.rule, line_count, threshold.because);
            findings.push(finding);
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Line_Count(source: &SourceFile) -> usize
{
    return source.text.lines().count();
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
