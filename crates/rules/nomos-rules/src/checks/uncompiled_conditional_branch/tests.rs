//! The rule's judgment over hand-built answers, and its reading through a real registry, store and
//! reader -- with every build's fact, with one missing, and with an offer below its floor.

use super::*;
use crate::checks::test_support::{self, FactToFile, OfferedProvider, Test_Context, TestOffering};
use nomos_analysis::{MemoryFactStore, Reader};
use nomos_cap_csharp_semantics::Encode_Payload;
use nomos_capability::ProviderOffer;
use nomos_contracts::SubjectId;
use nomos_model::Content_Digest;

const FILE: &str = "src/App/Widget.cs";

fn Selection(configuration: &str) -> BuildSelection
{
    return BuildSelection { project: "src/App/App.csproj".to_owned(), configuration: configuration.to_owned(), target_framework: "net8.0".to_owned() };
}

fn Region(branch: Branch, line: usize, end_line: usize, condition: &str, state: BranchState) -> ConditionalRegion
{
    return ConditionalRegion { branch, line, end_line, condition: condition.to_owned(), state };
}

/// One build's answer for a file holding three chains:
///
/// - `#if DEBUG` (line 3) / `#else` (line 5), which the two builds split;
/// - `#if NET8_0_OR_GREATER` (line 8) / `#else` (line 10), whose `#else` neither compiles, with a
///   `#if TRACE` nested inside that `#else` at line 11;
/// - `#if LEGACY` (line 16) / `#elif ANCIENT` (line 18), neither of which either build compiles --
///   two dead siblings, the second opening on the line that closes the first.
fn Answer(configuration: &str, debug: bool) -> ConditionalPayload
{
    let (taken, other) = if debug { (BranchState::Compiled, BranchState::Skipped) } else { (BranchState::Skipped, BranchState::Compiled) };
    return ConditionalPayload {
        selection: Selection(configuration),
        symbols: Vec::new(),
        definitions: Vec::new(),
        regions: vec![
            Region(Branch::If, 3, 5, "DEBUG", taken),
            Region(Branch::Else, 5, 7, "", other),
            Region(Branch::If, 8, 10, "NET8_0_OR_GREATER", BranchState::Compiled),
            Region(Branch::Else, 10, 14, "", BranchState::Skipped),
            Region(Branch::If, 11, 13, "TRACE", BranchState::Skipped),
            Region(Branch::If, 16, 18, "LEGACY", BranchState::Skipped),
            Region(Branch::Elif, 18, 20, "ANCIENT", BranchState::Skipped),
        ],
    };
}

fn Both_Builds() -> Vec<ConditionalPayload>
{
    return vec![Answer("Debug", true), Answer("Release", false)];
}

fn Subject_Names(findings: &[Finding]) -> Vec<&str>
{
    return findings.iter().map(|finding| return finding.subject_name.as_str()).collect();
}

/// The three branches no build compiles are reported, the one nested inside the first is not,
/// and the two branches the builds split are not: each is compiled by one of them.
#[test]
fn Test_A_Branch_Should_Be_Reported_Only_When_Every_Build_Skips_It()
{
    let findings = Uncompiled_Branches(FILE, &Both_Builds());

    assert_eq!(Subject_Names(&findings), [format!("{FILE}:10"), format!("{FILE}:16"), format!("{FILE}:18")], "{findings:?}");
    let first = findings.first().expect("asserted three above");
    assert_eq!(first.applicability, Applicability::Supported);
    assert_eq!(first.gate, GateCategory::Advisory);
    assert_eq!(first.subject, nomos_model::Subject_Of_Path(FILE));
    assert!(first.summary.contains("`#else` branch") && first.summary.contains("none of the 2 declared build(s)"), "{}", first.summary);
    assert!(first.summary.contains("src/App/App.csproj Debug net8.0, src/App/App.csproj Release net8.0"), "{}", first.summary);
    let second = findings.get(1).expect("asserted three above");
    assert!(second.summary.contains("`#if LEGACY` branch"), "{}", second.summary);
    let third = findings.get(2).expect("asserted three above");
    assert!(third.summary.contains("`#elif ANCIENT` branch"), "{}", third.summary);
}

/// Under one build alone, the `#else` of `#if DEBUG` is compiled by nothing that build compiles --
/// which is why the verdict is over every declared build and never over one.
#[test]
fn Test_A_Branch_Another_Declared_Build_Compiles_Should_Not_Be_Reported()
{
    let debug_only = Uncompiled_Branches(FILE, &[Answer("Debug", true)]);
    let both = Uncompiled_Branches(FILE, &Both_Builds());

    assert!(Subject_Names(&debug_only).contains(&format!("{FILE}:5").as_str()), "{debug_only:?}");
    assert!(!Subject_Names(&both).contains(&format!("{FILE}:5").as_str()), "{both:?}");
}

/// A branch one build cannot evaluate is not called dead: that build's compiler reports the
/// condition as an error, and "compiled by nothing" would be a guess about a build that fails.
#[test]
fn Test_A_Branch_Any_Build_Cannot_Evaluate_Should_Not_Be_Reported()
{
    let mut release = Answer("Release", false);
    if let Some(ancient) = release.regions.last_mut()
    {
        ancient.state = BranchState::Unevaluated;
    }

    let findings = Uncompiled_Branches(FILE, &[Answer("Debug", true), release]);

    assert_eq!(Subject_Names(&findings), [format!("{FILE}:10"), format!("{FILE}:16")], "{findings:?}");
}

/// Two answers for one text that disagree about the file's own branches are refused rather than
/// aligned by position, which could pair one chain's branch with another's.
#[test]
fn Test_Answers_That_Disagree_About_The_Files_Branches_Should_Be_Refused()
{
    let mut release = Answer("Release", false);
    release.regions.pop();

    let findings = Uncompiled_Branches(FILE, &[Answer("Debug", true), release]);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().map(|finding| return finding.applicability), Some(Applicability::Unparseable));
}

/// Through a real registry, store and reader, one source per build, each fact under that build's
/// own subject with the empty inputs the provider files -- the reading a run makes.
#[test]
fn Test_Check_Should_Judge_Every_Builds_Fact_Through_The_Reader()
{
    let Offering { mut store, registry, offer } = Offering(FactVariant::SemanticallyResolved);
    let sources: Vec<SourceFile> = Both_Builds().iter().map(|payload| return File_Answer(&mut store, &offer, payload)).collect();
    let mut reader = Reader::On(&store, &registry, Test_Context());

    let findings = Check_Uncompiled_Conditional_Branch(&sources, &mut reader);

    assert_eq!(Subject_Names(&findings), [format!("{FILE}:10"), format!("{FILE}:16"), format!("{FILE}:18")], "{findings:?}");
}

/// A build whose fact is missing leaves the whole file unjudged, reported as unread: the missing
/// build may be the one that compiles a branch the other skips.
#[test]
fn Test_Check_Should_Leave_A_File_Unjudged_When_One_Builds_Fact_Is_Missing()
{
    let Offering { mut store, registry, offer } = Offering(FactVariant::SemanticallyResolved);
    let debug = File_Answer(&mut store, &offer, &Answer("Debug", true));
    let release = SourceFile::New(FILE, Build_Subject_For("Release"), String::new());
    let mut reader = Reader::On(&store, &registry, Test_Context());

    let findings = Check_Uncompiled_Conditional_Branch(&[debug, release], &mut reader);

    assert_eq!(findings.len(), 1, "{findings:?}");
    let found = findings.first().expect("asserted one above");
    assert!(found.applicability.Is_Coverage_Debt(), "{found:?}");
    assert_eq!(found.subject_name, FILE);
}

/// An offer that reads the file on its face -- `Syntactic` -- is below this rule's floor, so its
/// answer is not believed however it is filed: a state read without the build's symbols is the
/// guess this rule exists to replace.
#[test]
fn Test_Check_Should_Not_Believe_An_Answer_Below_Its_Floor()
{
    let Offering { mut store, registry, offer } = Offering(FactVariant::Syntactic);
    let sources: Vec<SourceFile> = Both_Builds().iter().map(|payload| return File_Answer(&mut store, &offer, payload)).collect();
    let mut reader = Reader::On(&store, &registry, Test_Context());

    let findings = Check_Uncompiled_Conditional_Branch(&sources, &mut reader);

    assert!(!findings.is_empty(), "an unbelieved answer is reported, never passed");
    assert!(findings.iter().all(|finding| return finding.applicability.Is_Coverage_Debt()), "{findings:?}");
}

struct Offering
{
    store: MemoryFactStore,
    registry: nomos_capability::Registry,
    offer: ProviderOffer,
}

fn Offering(variant: FactVariant) -> Offering
{
    let TestOffering { store, registry, offer } = test_support::Offered_Registry(OfferedProvider {
        contract: nomos_cap_csharp_semantics::Capability_Contract(),
        capability: nomos_cap_csharp_semantics::Capability(),
        version: nomos_cap_csharp_semantics::CONTRACT_VERSION,
        provider: "nomos.test.csharp.conditional",
        guarantee: Guarantee::New(variant, Assurance::Sound, Assurance::Sound, IncrementalGranularity::File),
    })
    .expect("a fresh Registry holds neither this contract nor this provider");

    return Offering { store, registry, offer };
}

/// The subject a build's answer for [`FILE`] is filed under -- any digest distinct per build
/// serves, since the rule takes the subject from its source and never derives it.
fn Build_Subject_For(configuration: &str) -> SubjectId
{
    return SubjectId::From_Digest(Content_Digest(format!("{FILE}\n{configuration}").as_bytes()));
}

/// Files `payload` under its build's subject and returns the source a run would hand the rule.
fn File_Answer(store: &mut MemoryFactStore, offer: &ProviderOffer, payload: &ConditionalPayload) -> SourceFile
{
    let subject = Build_Subject_For(&payload.selection.configuration);
    test_support::Materialize_Fact(store, FactToFile {
        subject,
        offer,
        semantic_inputs: InputDigest::Of(&[]),
        schema: nomos_cap_csharp_semantics::Payload_Schema(),
        bytes: Encode_Payload(payload),
    })
    .expect("the fixture's store holds no fact under this key at a newer generation");

    return SourceFile::New(FILE, subject, String::new());
}
