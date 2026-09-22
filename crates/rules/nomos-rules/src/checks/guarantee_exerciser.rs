//! A declared provider guarantee names what exercises it, and the name resolves.
//!
//! `OD-CAPABILITY-016` made the two-direction check a requirement. Downward,
//! `nomos_capability::Registry` refuses an offer claiming more than its capability's ceiling
//! permits. Upward, something must assert that what the provider *does* claim is true —
//! because `Assurance::Sound` is the single value `Satisfies_Requirement` returns true for,
//! so it is the one value that clears a rule's requirement floor, and the downward direction
//! establishes only that a claim is permitted.
//!
//! That record measured eleven guarantee-declaring crates and found two holding the
//! convention, one of the two *stating* it not among them. It is the second time the
//! convention was found stated and unheld, which is why it chose a mechanism rather than
//! more prose.
//!
//! # What is required, and what is not
//!
//! A `Declared_Guarantee` names its exerciser **or says it has none and why**. Both halves
//! are the requirement. The second is not an escape from it: forcing a test where no
//! property can be exhibited produces the tautology that record found in
//! `nomos-lang-go` — `Test_Declared_Guarantee_Should_State_A_Sound_Syntactic_File_Granular_Claim`,
//! which asserts the declared value equals itself and which a provider emitting nothing at
//! all would pass. A stated absence is information; silence is not. `OD-RULES-001` is the
//! same principle one level down.
//!
//! # The marker
//!
//! `Exercised by ` followed by either a backticked test name or the bare word `nothing`:
//!
//! ```text
//! /// Exercised by `Test_Soundness_Should_Hold_Every_Reported_Name_Occurs_In_The_Source`.
//! /// Exercised by nothing: a provider that reads a declared file and reports its
//! /// contents has no way to exhibit unsoundness.
//! ```
//!
//! One marker, two answers, which is why it is one marker. The grammar is
//! [`crate::universe_kind`]'s `Mirrored by ` deliberately rather than a second one invented
//! here, down to the refusal that matters most: **a marker followed by prose is not a
//! claim**. `Exercised by the tests below` names no test, and admitting it would invent an
//! exerciser out of whatever word came next — the same failure that rule's own
//! `Named_On` doc warns about.
//!
//! # Resolution is cross-file, and that is not incidental
//!
//! A guarantee is declared in `src/guarantee.rs` and exercised from `tests/guarantee.rs`, so
//! a name is resolved against every test function the whole run collected —
//! [`crate::facts::Check_Names_In`], already written for the mirror rule and reused rather
//! than re-derived. A run over a truncated source set would resolve a real test to nothing
//! and report a declaration that is in fact exercised, which is why a file whose fact could
//! not be read is reported as unread rather than passed over.
//!
//! # Advisory, and what would change that
//!
//! `OD-CAPABILITY-016` left the gate category to this increment. It is
//! [`GateCategory::Advisory`], and that is a statement about this workspace's population
//! rather than about the claim's weight. Fifteen of the eighteen declarations name nothing
//! today; the record that created the obligation is days old, and a rule that turned an
//! obligation into a red build the moment it was written would be punishing the workspace
//! for a gap its own author had just described. When the population reaches zero, nothing
//! about the claim argues for leaving it Advisory — an unexercised `Sound` clears a
//! requirement floor on a promise nobody checked, and that is a build-failing condition.
//! Raising it is a one-word change and a re-measurement, not a redesign.

use crate::SourceFile;
use crate::facts::Check_Names_In;
use nomos_analysis::FactReader;
use nomos_cap_syntax::{FUNCTION, PayloadItem, SyntaxPayload};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};
use std::collections::BTreeSet;

/// This rule's own identifier.
pub const GUARANTEE_DECLARES_ITS_EXERCISER: &str = "guarantee-declares-its-exerciser";

/// The record this rule implements.
pub const GUARANTEE_EXERCISER_CONTRACT_RECORD: &str = "OD-CAPABILITY-016";

/// The version of [`GUARANTEE_EXERCISER_CONTRACT_RECORD`] this implementation was written
/// against. `tests/contract/tests/rule_contract_citation.rs` reads the record's own front
/// matter and compares, so an amendment this has not caught up to fails rather than sitting
/// asserted in prose.
pub const GUARANTEE_EXERCISER_CONTRACT_RECORD_VERSION: u32 = 1;

/// The function whose doc carries the declaration this rule judges.
///
/// A name rather than a type, because a provider crate exports this by convention and
/// nothing enforces it — there is no `Provider` trait, as the provider recipe says in as
/// many words. The convention is the only handle there is.
const DECLARATION: &str = "Declared_Guarantee";

/// The marker a declaration states its exerciser behind.
const EXERCISER_MARKER: &str = "Exercised by ";

/// What a declaration writes instead of a name when no test can exhibit its axes.
const NOTHING: &str = "nothing";

/// Reports declared guarantees that name no exerciser, or name one that resolves to nothing.
#[must_use]
pub fn Check_Guarantee_Declares_Its_Exerciser(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let mut judged: Vec<JudgedFile> = Vec::new();
    let mut findings: Vec<Finding> = Vec::new();

    for source in sources
    {
        match super::naming::reading::Payload_Of(source, facts)
        {
            Ok(payload) => judged.push(JudgedFile { path: source.path.clone(), payload }),
            Err(finding) => findings.push(Unread_As_This_Rule(finding)),
        }
    }

    findings.extend(Findings_In(&judged));
    return findings;
}

fn Unread_As_This_Rule(mut finding: Finding) -> Finding
{
    finding.rule = RuleId::New(GUARANTEE_DECLARES_ITS_EXERCISER);
    finding.summary = finding
        .summary
        .replace("this file's naming could not be judged", "this file's guarantee declarations could not be judged");
    return finding;
}

/// One file's decoded payload beside the path it was read from.
///
/// A named pair rather than a `(String, SyntaxPayload)`: resolution is cross-file, so every
/// judgment below carries both halves of it, and a pair leaves the reader telling the path from
/// the payload by position alone.
struct JudgedFile
{
    path: String,
    payload: SyntaxPayload,
}

/// Every declaration's finding over an already-decoded set, which is the half a fixture can
/// reach. Kept apart from the reading above for the reason
/// [`super::naming::reading`]'s own module doc gives: a pure function of decoded payloads is
/// testable against hand-written text, and how a payload gets found is the half no fixture
/// touches.
///
/// Takes the whole set rather than one payload at a time, because resolution is cross-file
/// and a per-file judgment could only ever resolve a name declared in the file it sits in.
fn Findings_In(judged: &[JudgedFile]) -> Vec<Finding>
{
    let mut tests: BTreeSet<String> = BTreeSet::new();
    for judged_file in judged
    {
        tests.extend(Check_Names_In(&judged_file.payload));
    }

    let mut findings: Vec<Finding> = Vec::new();
    for judged_file in judged
    {
        for item in &judged_file.payload.items
        {
            let item_findings = Item_Findings(&judged_file.path, item, &tests);
            findings.extend(item_findings);
        }
    }

    findings.sort_by(|left, right| return left.locations.cmp(&right.locations));
    return findings;
}

/// One item's findings: empty unless the item is a guarantee declaration that fails to name
/// a resolvable exerciser.
fn Item_Findings(path: &str, item: &PayloadItem, tests: &BTreeSet<String>) -> Vec<Finding>
{
    if item.kind != FUNCTION || item.Own_Name() != DECLARATION
    {
        return Vec::new();
    }

    if !item.documentation.Was_Observed()
    {
        // The provider cannot see documentation, so nothing is known about what this
        // declaration claims. Reporting it as naming nothing would turn a provider's own
        // stated blindness into this crate's finding about somebody's code.
        return vec![Unobserved_Finding(path, item)];
    }

    return match Claimed_Exerciser(item.documentation.Value())
    {
        Claim::Absent => vec![Violation_Finding(path, item, &format!("names no exerciser: add `{EXERCISER_MARKER}` followed by a backticked test name, or by `{NOTHING}` and the reason no test can exhibit its axes"))],
        Claim::NoneStated => Vec::new(),
        Claim::Named(name) if tests.contains(&name) => Vec::new(),
        Claim::Named(name) => vec![Violation_Finding(path, item, &format!("names `{name}`, which resolves to no test function this run read"))],
    };
}

/// What a declaration's documentation claims about its exerciser.
enum Claim
{
    /// No marker anywhere in the doc.
    Absent,
    /// The marker, followed by the word this rule reserves for a stated absence.
    NoneStated,
    /// The marker, followed by a backticked name.
    Named(String),
}

/// The claim a documentation block makes, reading the first marker line that makes one.
///
/// A line carrying the marker but neither a backticked name nor [`NOTHING`] makes no claim
/// and is skipped rather than refused, so a doc that discusses exercisers in prose before
/// stating one is read by its statement. A doc that only discusses them states nothing,
/// which is [`Claim::Absent`] — and correctly so, since nothing there resolves.
fn Claimed_Exerciser(documentation: Option<&str>) -> Claim
{
    let Some(text) = documentation
    else
    {
        return Claim::Absent;
    };

    for line in text.lines()
    {
        if let Some(claim) = Claimed_On(line)
        {
            return claim;
        }
    }

    return Claim::Absent;
}

/// The claim one line makes, if it makes one.
fn Claimed_On(line: &str) -> Option<Claim>
{
    let (_, after) = line.split_once(EXERCISER_MARKER)?;

    if let Some(quoted) = after.strip_prefix('`')
    {
        let (name, _) = quoted.split_once('`')?;
        if name.is_empty()
        {
            return None;
        }
        return Some(Claim::Named(name.to_owned()));
    }

    if after.starts_with(NOTHING)
    {
        return Some(Claim::NoneStated);
    }

    return None;
}

fn Violation_Finding(path: &str, item: &PayloadItem, detail: &str) -> Finding
{
    let address = Address_Of(path, item);

    return Finding {
        address: Some(address.clone()),
        rule: RuleId::New(GUARANTEE_DECLARES_ITS_EXERCISER),
        subject: Subject_Of_Address(&address),
        subject_name: item.qualified_name.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!("this guarantee declaration {detail}"),
        locations: vec![path.to_owned()],
    };
}

/// A declaration whose documentation the answering provider cannot see.
///
/// `Applicability::Unparseable` rather than `Supported`: the rule looked and the field it
/// reads was not observable, which is neither a pass nor a violation.
fn Unobserved_Finding(path: &str, item: &PayloadItem) -> Finding
{
    let address = Address_Of(path, item);

    return Finding {
        address: Some(address.clone()),
        rule: RuleId::New(GUARANTEE_DECLARES_ITS_EXERCISER),
        subject: Subject_Of_Address(&address),
        subject_name: item.qualified_name.clone(),
        applicability: Applicability::Unparseable,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: "this guarantee declaration's exerciser could not be judged: the provider that answered for this file does not observe documentation".to_owned(),
        locations: vec![path.to_owned()],
    };
}

/// The string a person would write to address a finding about `item`.
///
/// Split from [`Subject_Of_Address`] so the two steps read in the order `OD-GATE-032`
/// decided them: a rule declares an address, and the subject is the digest of that address.
/// Computing the composite inside the subject function left it private, which is exactly the
/// state that made these findings unaddressable.
fn Address_Of(path: &str, item: &PayloadItem) -> String
{
    return format!("{path}::{}", item.qualified_name);
}

/// The identity of the thing `address` names.
fn Subject_Of_Address(address: &str) -> SubjectId
{
    use nomos_model::Content_Digest;

    return SubjectId::From_Digest(Content_Digest(address.as_bytes()));
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_A_Declaration_Naming_A_Test_That_Exists_Should_Not_Be_Reported()
    {
        let findings = Findings_In(&[Payload_From_Text(
            "src/guarantee.rs",
            "unexpanded\t0\n\
             item\t0\tFunction\tPublic\tDeclared_Guarantee\t+Exercised by `Test_It_Holds`.\t+fn/0\n\
             item\t1\tFunction\tPrivate\tTest_It_Holds\t.\t+fn/0\n".to_owned(),
        )]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// The whole point of the rule: a name that resolves to nothing is a claim nobody kept.
    #[test]
    fn Test_A_Declaration_Naming_A_Test_That_Does_Not_Exist_Should_Be_Reported()
    {
        let findings = Findings_In(&[Payload_From_Text(
            "src/guarantee.rs",
            "unexpanded\t0\nitem\t0\tFunction\tPublic\tDeclared_Guarantee\t+Exercised by `Test_Nowhere`.\t+fn/0\n".to_owned(),
        )]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert!(
            findings.first().expect("asserted len 1 above").summary.contains("Test_Nowhere"),
            "the finding must name what failed to resolve: {findings:?}"
        );
    }

    #[test]
    fn Test_A_Declaration_Naming_Nothing_Should_Be_Reported()
    {
        let findings = Findings_In(&[Payload_From_Text(
            "src/guarantee.rs",
            "unexpanded\t0\nitem\t0\tFunction\tPublic\tDeclared_Guarantee\t+What this provider promises.\t+fn/0\n".to_owned(),
        )]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert!(findings.first().expect("asserted len 1 above").summary.contains("names no exerciser"), "{findings:?}");
    }

    /// The second half of the requirement, and not an escape from it: a stated absence
    /// satisfies the rule where silence does not.
    #[test]
    fn Test_A_Declaration_Stating_It_Has_No_Exerciser_Should_Not_Be_Reported()
    {
        let findings = Findings_In(&[Payload_From_Text(
            "src/guarantee.rs",
            "unexpanded\t0\nitem\t0\tFunction\tPublic\tDeclared_Guarantee\t+Exercised by nothing: it reports a declared file verbatim.\t+fn/0\n".to_owned(),
        )]);

        assert!(findings.is_empty(), "a stated absence is information, not silence: {findings:?}");
    }

    /// `Mirrored by `'s own refusal, kept here: a marker followed by prose names nothing,
    /// and admitting it would invent an exerciser out of the next word.
    #[test]
    fn Test_A_Marker_Followed_By_Prose_Should_Not_Count_As_A_Claim()
    {
        let findings = Findings_In(&[Payload_From_Text(
            "src/guarantee.rs",
            "unexpanded\t0\nitem\t0\tFunction\tPublic\tDeclared_Guarantee\t+Exercised by the tests below.\t+fn/0\n".to_owned(),
        )]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert!(findings.first().expect("asserted len 1 above").summary.contains("names no exerciser"), "{findings:?}");
    }

    /// Resolution is cross-file, so a name is resolved against the whole run rather than
    /// against the file the declaration sits in. Without this the rule would report every
    /// real declaration in this workspace, all of which are exercised from another file.
    #[test]
    fn Test_A_Name_Should_Resolve_Against_Another_File()
    {
        let findings = Findings_In(&[
            Payload_From_Text(
                "src/guarantee.rs",
                "unexpanded\t0\nitem\t0\tFunction\tPublic\tDeclared_Guarantee\t+Exercised by `Test_Elsewhere`.\t+fn/0\n".to_owned(),
            ),
            Payload_From_Text("tests/guarantee.rs", "unexpanded\t0\nitem\t0\tFunction\tPrivate\tTest_Elsewhere\t.\t+fn/0\n".to_owned()),
        ]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// A function that is not a guarantee declaration is not this rule's subject, however
    /// its documentation reads.
    #[test]
    fn Test_Another_Function_Should_Not_Be_Judged()
    {
        let findings = Findings_In(&[Payload_From_Text(
            "src/guarantee.rs",
            "unexpanded\t0\nitem\t0\tFunction\tPublic\tProvider_Offer\t+What this provider offers.\t+fn/0\n".to_owned(),
        )]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// A provider that cannot see documentation has said nothing about this declaration,
    /// and the rule must not turn that into a finding of absence.
    #[test]
    fn Test_An_Unobserved_Documentation_Field_Should_Not_Read_As_Naming_Nothing()
    {
        let findings = Findings_In(&[Payload_From_Text(
            "src/guarantee.rs",
            "unexpanded\t0\nitem\t0\tFunction\tPublic\tDeclared_Guarantee\t-\t+fn/0\n".to_owned(),
        )]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let finding = findings.first().expect("asserted len 1 above");
        assert_eq!(finding.applicability, Applicability::Unparseable);
        assert!(finding.summary.contains("could not be judged"), "{findings:?}");
    }

    fn Payload_From_Text(path: &str, text: String) -> JudgedFile
    {
        let payload = nomos_cap_syntax::Parse_Payload(text.as_bytes()).expect("this fixture payload is well formed");
        return JudgedFile { path: path.to_owned(), payload };
    }
}
