//! File structure rules from code-standards.
//!
//! The standards define two line-count triggers: files above roughly 500 lines are review
//! candidates for splitting, and files above roughly 1500 lines must carry an explicit
//! justification. `no-mod-rs-files` is text-independent: the path alone decides whether a
//! source file uses the old Rust module layout, so [`Check_No_Mod_Rs_Files`] alone still
//! takes only [`SourceFile`]s. Go carries its own, lower pair of the same two triggers — 500
//! lines for review and 1000 lines (not 1500) for the hard trigger — under their own rule
//! ids, scoped to `.go` sources. [`go_file_size`] holds that Go pair and [`mod_rs`] the
//! path-only layout rule, split out by responsibility the same way `dependency.rs` and
//! `naming.rs` split their own entry points; what stays here is the two Rust triggers and
//! the threshold machinery all four share.
//!
//! # The thresholds are now a repository's own, resolved rather than compiled in
//!
//! `OD-RULES-011` named this threshold family as its own future instance of the naming
//! decision `checks::naming::Resolve_Read` already generalizes: [`Resolve_Limit`] asks
//! `nomos.cap.limits.policy` for the line count a repository declares, falling back to the
//! axis's declared default when a repository declares none — the identical
//! `Require`-then-fall-back-on-any-`Err` shape, since `OD-CAPABILITY-004`/`OD-RULES-011`
//! settle that this capability is optional the same way naming's is. The thresholds named
//! above are those defaults, still real and still what an unconfigured repository gets, and
//! `OD-RULES-035` put them on the axes in `rule_descriptor::policy_axis` rather than here.
//!
//! [`Resolve_Limit`] is the limits family's one resolver: `function_shape` and
//! `nesting_depth` read through it too, where each once kept a copy or reached this one by a
//! widened visibility.

mod go_file_size;
mod mod_rs;

use super::optional_reads::{FILE_SIZE_HARD, FILE_SIZE_REVIEW};
use crate::rule_descriptor::policy_axis::PolicyAxis;
use crate::rule_descriptor::{LimitRead, Note_Read, OptionalRead, RequiredFact, UndeclaredOutcome, UndeclaredValue};
use crate::SourceFile;
use nomos_analysis::{FactReader, InputDigest};
use nomos_cap_limits_policy::{PolicyRow, Scope};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

pub use go_file_size::{Check_Go_File_Size_Hard_Trigger, Check_Go_File_Size_Review_Trigger};
pub use mod_rs::Check_No_Mod_Rs_Files;

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

/// Reports files whose line count exceeds the review trigger.
#[must_use]
pub fn Check_File_Size_Review_Trigger(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let Some(threshold) = Resolve_Count(facts, &FILE_SIZE_REVIEW)
    else
    {
        return vec![Undeclared_Limit(FILE_SIZE_REVIEW_TRIGGER, FILE_SIZE_REVIEW.axis)];
    };
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
    let Some(threshold) = Resolve_Count(facts, &FILE_SIZE_HARD)
    else
    {
        return vec![Undeclared_Limit(FILE_SIZE_JUSTIFICATION_TRIGGER, FILE_SIZE_HARD.axis)];
    };
    let because = format!("exceeds the {threshold}-line trigger and needs an explicit splitting justification");
    return Findings_For_Threshold(
        sources,
        LineThreshold { rule: FILE_SIZE_JUSTIFICATION_TRIGGER, lines: threshold, because: &because },
        |_| return true,
    );
}

/// Resolves the value `read` takes: a repository's own declared `nomos.cap.limits.policy`,
/// most-specific row first (the read's language's own override, then the repository-wide row),
/// falling back to what the axis says an undeclared value means for that language when neither
/// is declared -- its default, or `None` for an axis reported as undeclared.
///
/// `read` is a constant in `checks::optional_reads`, the one the calling rule's descriptor row
/// declares it reads (`OD-RULES-011` version 3 decision 5), and noting it here is what lets a test
/// hold every body to its row.
///
/// `None` is not a value a caller chooses a meaning for: every caller hands it to
/// [`Undeclared_Limit`], so what an undeclared axis means stays written on the axis
/// (`OD-RULES-035` section 3) and no rule decides it again in its own body.
///
/// `OD-CAPABILITY-004` and `OD-RULES-011` settle how an absent read is treated here,
/// mirroring `checks::naming::Resolve_Read` exactly: this capability is optional, so
/// `facts.Require` failing for any reason is exactly "nothing declared" — never this
/// capability's own `Applicability` surfacing anywhere.
pub(super) fn Resolve_Limit(facts: &mut dyn FactReader, read: &LimitRead) -> Option<u32>
{
    Note_Read(OptionalRead::Of_Limit(*read));
    let undeclared = read.axis.Undeclared_Value(read.language);
    let Some(payload) = Materialized_Limits_Payload(facts, read.axis.family)
    else
    {
        return undeclared;
    };

    return Declared_Limit(&payload, read).or(undeclared);
}

/// The value `payload` declares for `read`: its language's own row first, then the
/// repository-wide row, or `None` when it declares neither.
fn Declared_Limit(payload: &nomos_cap_limits_policy::LimitsPolicyPayload, read: &LimitRead) -> Option<u32>
{
    let for_language = read.language.and_then(|language| return Matching_Row(payload, Scope::Language(language.to_owned()), read.axis.key));

    return for_language.or_else(|| return Matching_Row(payload, Scope::Repository, read.axis.key)).map(|row| return row.value);
}

/// The value `read` reads, named when the repository's limits were read and declare it in
/// neither scope [`Resolve_Limit`] looks in: judged against the axis's default, or reported
/// undeclared for an axis with none. `OD-RULES-011` version 3 decision 1.
///
/// `None` when the repository declared it, and when its limits could not be read at all, which
/// is coverage debt and not a value nobody declared. An absent `nomos-limits.json` is read: its
/// provider answers no rows for it, so every limit it does not declare is named.
pub(crate) fn Undeclared_Limit_Value(facts: &mut dyn FactReader, read: &LimitRead) -> Option<UndeclaredValue>
{
    let payload = Materialized_Limits_Payload(facts, read.axis.family)?;
    if Declared_Limit(&payload, read).is_some()
    {
        return None;
    }

    let outcome = read.axis.Undeclared_Value(read.language).map_or(UndeclaredOutcome::ReportedUndeclared, |value| {
        return UndeclaredOutcome::JudgedAgainst { value: value.to_string() };
    });

    return Some(UndeclaredValue {
        family: "limits",
        declared_in: LIMITS_JSON,
        key: read.axis.key,
        language: read.language.map(str::to_owned),
        outcome,
    });
}

/// [`Resolve_Limit`] as a `usize`, which is how the four file-size triggers compare a line count,
/// `nesting_depth` compares a depth and `cyclomatic_complexity` a complexity.
///
/// A declared `u32` always fits a `usize` on every target this workspace builds for; the
/// saturating fallback exists only so the conversion states an answer rather than a panic.
pub(super) fn Resolve_Count(facts: &mut dyn FactReader, read: &LimitRead) -> Option<usize>
{
    return Resolve_Limit(facts, read).map(|limit| return usize::try_from(limit).unwrap_or(usize::MAX));
}

/// What a rule reports in place of a verdict when `axis` is undeclared and says so: one advisory
/// finding against `nomos-limits.json`, naming the key a person would declare.
///
/// `NotApplicable` rather than a coverage-debt state: nothing failed and nothing is missing from
/// the analysis. It is the positive statement that the rule does not bind here, because the
/// repository has not said what it would bind to.
pub(super) fn Undeclared_Limit(rule: &str, axis: &PolicyAxis<u32>) -> Finding
{
    return Finding {
        address: Some(format!("{LIMITS_JSON}#*.{}", axis.key)),
        rule: RuleId::New(rule),
        subject: nomos_model::Subject_Of_Path(LIMITS_JSON),
        subject_name: LIMITS_JSON.to_owned(),
        applicability: Applicability::NotApplicable,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!(
            "{LIMITS_JSON} declares no `{}`, and this limit is reported as undeclared rather than judged against a \
             default nobody chose, so nothing was judged; declaring it is the repository's choice",
            axis.key
        ),
        locations: vec![LIMITS_JSON.to_owned()],
    };
}

/// The file a repository declares its limits in, for every finding that names it.
///
/// `nomos_repo_policy::limits::LIMITS_JSON` owns this literal. It is repeated rather than named
/// because Rules zone may not depend on Provider zone, and a finding has to say which file a
/// person opens to fix it. Once in this crate: [`Undeclared_Limit`] and
/// `undeclared_policy_key` both read it here.
pub(super) const LIMITS_JSON: &str = "nomos-limits.json";

/// The fact `Resolve_Limit` reads, decoded — `None` for both "no repository declares this
/// capability" and "the declared payload does not parse," the same "any `Err` is just no
/// override" reading the function's own doc comment settles.
///
/// The capability required is the one `family` names, which for every limits axis is
/// `nomos.cap.limits.policy`: `rule_descriptor::policy_axis`'s own tests pin that.
pub(super) fn Materialized_Limits_Payload(facts: &mut dyn FactReader, family: RequiredFact) -> Option<nomos_cap_limits_policy::LimitsPolicyPayload>
{
    let subject = nomos_model::Subject_Of_Path("");
    let Ok(fact) = facts.Require(&family.Capability(), &subject, InputDigest::Of(&[]), &Limits_Policy_Requirement())
    else
    {
        return None;
    };

    return nomos_cap_limits_policy::Parse_Payload(&fact.payload.bytes).ok();
}

/// This crate's own floor for `nomos.cap.limits.policy` — stated at the capability's own
/// ceiling since there is only one real provider today and no weaker answer this crate
/// could honestly still act on. Mirrors `checks::naming::Naming_Policy_Requirement`
/// exactly, for the sibling capability, and it is the limits family's only one.
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
///
/// `pub(super)` because all four triggers construct it, the Go pair from
/// `structure/go_file_size.rs` and the Rust pair from here.
pub(super) struct LineThreshold<'a>
{
    pub(super) rule: &'a str,
    pub(super) lines: usize,
    pub(super) because: &'a str,
}

pub(super) fn Findings_For_Threshold(
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
        address: None,
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
#[path = "structure/tests.rs"]
mod tests;

/// Narrow, file-local proofs for this file's own public and crate-visible functions, addressed
/// by name.
///
/// [`tests`] above is `structure/tests.rs`, a separate physical file whose behavioural suite
/// this does not repeat or replace. `check-test-coverage`'s Rust front end keys a test's
/// companion unit off the literal file it is textually written in, so a test living in that
/// separate file can never address a function declared here, however it is named — this module
/// gives [`Check_File_Size_Review_Trigger`], [`Check_File_Size_Justification_Trigger`],
/// [`Resolve_Limit`], [`Resolve_Count`] and [`Findings_For_Threshold`] the one-file address
/// the check reads.
///
/// The fixture is [`super::tests::Fixture_Over_Threshold`], which lives beside the rest of this
/// module's fixtures rather than being restated here: a second, near-identical copy of it in
/// this file and in `structure/go_file_size.rs` was itself a duplication finding.
#[cfg(test)]
mod self_tests
{
    use super::tests::{Fixture_Over_Threshold, JUSTIFICATION_TRIGGER_LINES, REVIEW_TRIGGER_LINES};
    use super::*;
    use nomos_analysis::{MemoryFactStore, Reader};
    use nomos_capability::Registry;

    #[test]
    fn Test_Check_File_Size_Review_Trigger_Should_Report_A_File_One_Line_Over()
    {
        let findings = Fixture_Over_Threshold(Check_File_Size_Review_Trigger, "src/large.rs", REVIEW_TRIGGER_LINES);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(FILE_SIZE_REVIEW_TRIGGER));
    }

    #[test]
    fn Test_Check_File_Size_Justification_Trigger_Should_Report_A_File_One_Line_Over()
    {
        let findings = Fixture_Over_Threshold(Check_File_Size_Justification_Trigger, "src/huge.rs", JUSTIFICATION_TRIGGER_LINES);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(FILE_SIZE_JUSTIFICATION_TRIGGER));
    }

    /// The trigger machinery's own public address: [`Findings_For_Threshold`] is the one function
    /// all four line-count rules call, so a boundary right at the threshold is the case that
    /// tells whether "exceeds" is strict everywhere rather than in the four callers separately.
    #[test]
    fn Test_Findings_For_Threshold_Should_Report_A_File_One_Line_Over_And_Accept_It_At_The_Threshold()
    {
        let over = Fixture_Over_Threshold(Check_File_Size_Review_Trigger, "src/large.rs", REVIEW_TRIGGER_LINES);
        assert_eq!(over.len(), 1, "{over:?}");

        let at = Fixture_Over_Threshold(Check_File_Size_Review_Trigger, "src/large.rs", REVIEW_TRIGGER_LINES.saturating_sub(1));
        assert!(at.is_empty(), "{at:?}");
    }

    #[test]
    fn Test_Resolve_Limit_Should_Fall_Back_To_The_Default_With_Nothing_Materialized()
    {
        let store = MemoryFactStore::New();
        let registry = Registry::New();
        let mut facts = Reader::On(&store, &registry, crate::checks::test_support::Test_Context());

        let resolved = Resolve_Limit(&mut facts, &FILE_SIZE_HARD);

        assert_eq!(resolved.and_then(|lines| return usize::try_from(lines).ok()), Some(JUSTIFICATION_TRIGGER_LINES));
    }

    /// The axis states Go's own lower default, so a Go rule reading it with nothing declared is
    /// judged against 1000 rather than the repository-wide 1500.
    #[test]
    fn Test_Resolve_Count_Should_Take_Gos_Own_Default_With_Nothing_Materialized()
    {
        let store = MemoryFactStore::New();
        let registry = Registry::New();
        let mut facts = Reader::On(&store, &registry, crate::checks::test_support::Test_Context());

        let resolved = Resolve_Count(&mut facts, &super::super::optional_reads::GO_FILE_SIZE_HARD);

        assert_eq!(resolved, Some(super::tests::GO_HARD_TRIGGER_LINES));
    }
}
