//! What `Run_Gate` composes and what it records about what judged it.

use super::reduction::{DispositionPolicies, Reduced_Findings};
use super::{JudgeContext, Judged_Sources, Run_Gate};
use crate::{
    CoveragePolicy, GateCommand, GatePhase, GateRunOutcome, GateRunProvenance, GateRunResult, NoVerdict, PhaseApproval, PhaseThreshold,
    RuleSelector,
};
use nomos_check_orchestration::{CheckOutcome, Claim};
use nomos_contracts::{Digest128, RuleId, RunId};
use nomos_model::Subject_Of_Path;
use nomos_platform_std::{StdEnvironment, StdFileSystem, StdProcessLauncher};
use nomos_rules::SourceFile;
use nomos_workspace::BuildVariant;
use std::path::{Path, PathBuf};

/// The run identity the phase-approval test judges its approved run under, distinct from the
/// ones beside it so a failure names which run it read.
const APPROVED_RUN_SEED: u8 = 9;

/// The run identity the unphased-finding test judges under, for the same reason.
const UNPHASED_RUN_SEED: u8 = 2;

/// A fixture's source path, as a type distinct from the text at it.
///
/// [`Source_File`] takes two adjacent string positions, which the file's own gate reported: a
/// caller can transpose them and the compiler will not object. Two named types make the
/// transposition a type error, and cost one line each.
#[derive(Clone, Copy)]
struct SourcePath<'a>(&'a str);

/// The text of a fixture's source file, distinct from the path it sits at.
#[derive(Clone, Copy)]
struct SourceText<'a>(&'a str);

/// A scratch root's own name, distinct from the policy text it declares.
#[derive(Clone, Copy)]
struct RootName<'a>(&'a str);

/// A `nomos-gate.json` body, distinct from the root's name.
#[derive(Clone, Copy)]
struct PolicyText<'a>(&'a str);

fn Source_File(path: SourcePath<'_>, text: SourceText<'_>) -> SourceFile
{
    return SourceFile::New(path.0, Subject_Of_Path(path.0), text.0);
}

/// One source guaranteed to produce a real `completeness-mirror` blocking finding --
/// the same fixture [`Test_Run_Gate_Should_Fail_On_A_Blocking_Finding`] already uses,
/// named so the phase tests below do not repeat its literal text.
fn Blocking_Sources() -> Vec<SourceFile>
{
    return vec![Source_File(
        SourcePath("a.rs"),
        SourceText("/// Mirrored by `Test_Nowhere`.\npub const T: &[&str] = &[];\n"),
    )];
}

/// A scratch root carrying `policy` as its own `nomos-gate.json`.
///
/// A third private copy of a helper `gate_policy_file.rs` and `tests.rs` each keep one of
/// already. Reaching across for either would make it public for a caller that wants three
/// lines, which costs this crate's surface more than the repetition costs a reader.
fn Root_With_Policy(name: RootName<'_>, policy: PolicyText<'_>) -> PathBuf
{
    let root = std::env::temp_dir().join(format!("nomos-gate-environment-{}-{}", name.0, std::process::id()));
    std::fs::create_dir_all(&root).expect("the scratch root sits under std::env::temp_dir(), which every platform this runs on provides");
    std::fs::write(root.join("nomos-gate.json"), policy.0).expect("the directory create_dir_all just returned Ok for is where this writes");

    return root;
}

/// `Run_Gate` over `sources` at `root`, under `command`, for the run seeded by `seed`.
///
/// `root` is folded into the command rather than kept beside it, because a run whose command
/// named one root and whose policy came from another would judge one tree under another's
/// declared policy -- exactly the confusion a provenance test exists to catch.
fn Ran_Over(root: &Path, sources: Vec<SourceFile>, command: GateCommand, seed: u8) -> GateRunResult
{
    let command = GateCommand { root: root.to_path_buf(), ..command };
    let run = RunId::From_Digest(Digest128::From_Bytes([seed; Digest128::BYTE_LENGTH]));
    let environment = super::GateEnvironment {
        variant: Test_Variant(),
        launcher: &StdProcessLauncher,
        filesystem: &StdFileSystem,
        environment: &StdEnvironment,
        now: nomos_platform::Timestamp::From_Unix_Seconds(0),
    };

    return Run_Gate(Some(sources), environment, &command, run);
}

/// `Run_Gate` over the shared blocking fixture at `root`, under `command`, for `seed`.
fn Ran_Blocking(root: &Path, command: GateCommand, seed: u8) -> GateRunResult
{
    return Ran_Over(root, Blocking_Sources(), command, seed);
}

/// Every rule `result`'s blocking findings name, so a phase can be declared over all of them.
fn Named_Rules(result: &GateRunResult) -> Vec<RuleId>
{
    let rules: Vec<RuleId> = result.findings.blocking_findings.iter().map(|finding| return finding.rule.clone()).collect();
    assert!(!rules.is_empty(), "the fixture must produce at least one blocking finding for this test to mean anything");

    return rules;
}

/// One unjudgeable finding judged with an incomplete claim, the shape both coverage tests read.
///
/// A rule with no provider is what makes the claim `Incomplete`, and the claim is what the
/// coverage floor reads, so every other field here exists only to make the finding real.
fn Incomplete_Judgment() -> CheckOutcome
{
    return CheckOutcome::Judged {
        findings: vec![Unjudgeable_Finding()],
        examined: nomos_check_orchestration::Examined { files: 1, facts: 1 },
        claim: Claim::Incomplete,
    };
}

/// One finding a rule could not judge, which is what makes a claim `Incomplete`.
fn Unjudgeable_Finding() -> nomos_contracts::Finding
{
    return nomos_contracts::Finding {
        rule: RuleId::New("dependency-policy"),
        subject: Subject_Of_Path("a.rs"),
        subject_name: "a.rs".to_owned(),
        applicability: nomos_contracts::Applicability::MissingCapability,
        evidence: nomos_contracts::EvidenceClass::Derived,
        gate: nomos_contracts::GateCategory::Advisory,
        summary: "no provider offered the capability this rule requires".to_owned(),
        locations: vec!["a.rs".to_owned()],
    };
}

/// A policy set naming nothing, as every test here that is not about a policy reads.
fn No_Dispositions<'a>(adoption: &'a crate::AdoptionPolicy, suppressions: &'a crate::SuppressionPolicy, baseline: &'a crate::BaselinePolicy)
    -> DispositionPolicies<'a>
{
    return DispositionPolicies { adoption, suppressions, baseline, now: nomos_platform::Timestamp::From_Unix_Seconds(0) };
}

#[test]
fn Test_Reduced_Findings_Should_Name_The_Coverage_Floor_That_Downgraded_A_Pass()
{
    // Driven through `Reduced_Findings` rather than `Run_Gate`: this is the exact mechanism
    // `OD-GATE-016` decided, and reaching it through a real walk would make the test depend
    // on which capabilities happen to be materializable on the machine running it.
    let (adoption, suppressions, baseline) = (Default::default(), Default::default(), Default::default());
    let policies = No_Dispositions(&adoption, &suppressions, &baseline);

    let reduced = Reduced_Findings(&Incomplete_Judgment(), &RuleSelector::default(), policies, CoveragePolicy::RequireCompleteness);

    assert_eq!(reduced.disposition, GateRunOutcome::Indeterminate);
    assert_eq!(reduced.no_verdict, Some(NoVerdict::IncompleteCoverage));
    assert!(reduced.findings.blocking_findings.is_empty(), "nothing here can fail a build; the floor is the whole reason");
}

/// The same findings without a declared floor reach a verdict and name no reason, which is
/// what makes the assertion above about the floor rather than about the findings.
#[test]
fn Test_Reduced_Findings_Should_Pass_The_Same_Findings_With_No_Declared_Floor()
{
    let (adoption, suppressions, baseline) = (Default::default(), Default::default(), Default::default());
    let policies = No_Dispositions(&adoption, &suppressions, &baseline);

    let reduced = Reduced_Findings(&Incomplete_Judgment(), &RuleSelector::default(), policies, CoveragePolicy::Unset);

    assert_eq!(reduced.disposition, GateRunOutcome::Passed);
    assert_eq!(reduced.no_verdict, None);
}

/// What `Run_Gate` recorded about a run over `root` judging `sources` under `rules`.
///
/// Every provenance test below differs from its partner in exactly one argument, which is
/// what makes each of them a statement about that one input rather than about a run.
///
/// They judge scratch roots rather than this repository's own, for two reasons. The
/// policy digest reads the `nomos-gate.json` under the root, and this tree's is shared
/// with live sessions, so a peer editing it mid-run would move a digest under a test
/// that is not about policy at all. And materializing this workspace's capabilities
/// costs a minute per call, for findings none of these assertions read.
fn Provenance_Over(root: &Path, sources: Vec<SourceFile>, rules: &RuleSelector) -> GateRunProvenance
{
    let command = GateCommand { rules: rules.clone(), ..Default::default() };
    let result = Ran_Over(root, sources, command, 0);

    return result.provenance.expect("Run_Gate sets Some(provenance) in its own GateRunResult for every run it judges");
}

/// One tree, two policies: the source agrees and the policy does not.
///
/// The likeliest instance of the whole defect. `Run_Gate` reads the policy from
/// `command.root` and a comparison judges two roots, so comparing two checkouts compares
/// two policies without anyone having asked for it, and every finding that moved bucket
/// for that reason reads as movement in the code.
#[test]
fn Test_Two_Policies_Over_One_Source_Should_Differ_In_The_Policy_Alone()
{
    let lenient = Root_With_Policy(RootName("provenance-lenient"), PolicyText(r#"{ "coverage": "unset" }"#));
    let strict = Root_With_Policy(RootName("provenance-strict"), PolicyText(r#"{ "coverage": "require-completeness" }"#));

    let one = Provenance_Over(&lenient, Blocking_Sources(), &RuleSelector::default());
    let other = Provenance_Over(&strict, Blocking_Sources(), &RuleSelector::default());

    assert_eq!(one.source, other.source, "the same files were judged on both sides");
    assert_ne!(one.policy, other.policy, "and they were judged under different policies");
    assert_eq!(one.selection, other.selection);
    assert_eq!(one.instrument, other.instrument);
}

/// One tree, two selections: a side told to look at less must be distinguishable from a
/// side that looked at everything and found less.
#[test]
fn Test_Two_Selections_Over_One_Source_Should_Differ_In_The_Selection_Alone()
{
    let root = Root_With_Policy(RootName("provenance-selection"), PolicyText("{}"));
    let everything = RuleSelector::default();
    let narrowed = RuleSelector { include: vec![RuleId::New("naming-convention")] };

    let whole = Provenance_Over(&root, Blocking_Sources(), &everything);
    let part = Provenance_Over(&root, Blocking_Sources(), &narrowed);

    assert_eq!(whole.source, part.source);
    assert_eq!(whole.policy, part.policy);
    assert_ne!(whole.selection, part.selection);
    assert_eq!(whole.instrument, part.instrument);
}

/// The licensed case: the source moved and nothing else did, so a difference in findings
/// is a difference in the repository and a comparison may say so.
#[test]
fn Test_A_Changed_Source_Should_Change_The_Source_And_Nothing_Else()
{
    let root = Root_With_Policy(RootName("provenance-source"), PolicyText("{}"));

    let before = Provenance_Over(&root, Blocking_Sources(), &RuleSelector::default());
    let after = Provenance_Over(&root, vec![Source_File(SourcePath("a.rs"), SourceText("pub fn Different() {}\n"))], &RuleSelector::default());

    assert_ne!(before.source, after.source);
    assert_eq!(before.policy, after.policy);
    assert_eq!(before.selection, after.selection);
    assert_eq!(before.instrument, after.instrument);
}

/// The same inputs record the same identity, which is what makes any of the assertions
/// above mean anything: a digest that varied on its own would make every one of them pass
/// for the wrong reason.
#[test]
fn Test_The_Same_Inputs_Should_Record_The_Same_Provenance()
{
    let root = Root_With_Policy(RootName("provenance-repeat"), PolicyText("{}"));

    let once = Provenance_Over(&root, Blocking_Sources(), &RuleSelector::default());
    let again = Provenance_Over(&root, Blocking_Sources(), &RuleSelector::default());

    assert_eq!(once, again);
}

/// The walker's ordering must not decide the source identity.
///
/// Without the sort in `Source_Digest`, two runs over identical content would disagree
/// whenever directory iteration did -- and a comparison would report the repository as
/// changed when nothing about it had, which is the worst available instance of the
/// failure `OD-GATE-031` exists to stop.
#[test]
fn Test_The_Same_Files_In_A_Different_Order_Should_Record_One_Source()
{
    let root = Root_With_Policy(RootName("provenance-order"), PolicyText("{}"));
    let first = Source_File(SourcePath("a.rs"), SourceText("pub fn One() {}\n"));
    let second = Source_File(SourcePath("b.rs"), SourceText("pub fn Two() {}\n"));

    let forwards = Provenance_Over(&root, vec![first.clone(), second.clone()], &RuleSelector::default());
    let backwards = Provenance_Over(&root, vec![second, first], &RuleSelector::default());

    assert_eq!(forwards.source, backwards.source);
}

/// A reordered selection selects identically, so it must not read as a different one.
///
/// `RuleSelector::Is_Included` answers with `any`. A stated difference nobody caused is
/// how a reader learns to stop reading them, which costs more than it saves.
#[test]
fn Test_A_Reordered_Selection_Should_Not_Read_As_A_Different_One()
{
    let root = Root_With_Policy(RootName("provenance-reorder"), PolicyText("{}"));
    let one_way = RuleSelector { include: vec![RuleId::New("naming-convention"), RuleId::New("nesting-depth")] };
    let other_way = RuleSelector { include: vec![RuleId::New("nesting-depth"), RuleId::New("naming-convention")] };

    let first = Provenance_Over(&root, Blocking_Sources(), &one_way);
    let second = Provenance_Over(&root, Blocking_Sources(), &other_way);

    assert_eq!(first.selection, second.selection);
}

#[test]
fn Test_Judged_Sources_Should_Report_Unreadable_For_An_Unwalked_Root()
{
    let root = Repository_Root();
    let outcome = Judged_Sources(None, JudgeContext { launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment, variant: Test_Variant(), root: &root, selected: &[] });

    assert!(matches!(outcome, CheckOutcome::Unreadable));
}

#[test]
fn Test_Judged_Sources_Should_Report_No_Source_For_An_Empty_Walk()
{
    let root = Repository_Root();
    let outcome = Judged_Sources(Some(Vec::new()), JudgeContext { launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment, variant: Test_Variant(), root: &root, selected: &[] });

    assert!(matches!(outcome, CheckOutcome::NoSource));
}

/// A policy file whose key is mis-spelled leaves the run without a verdict, and the run
/// says so *and* says which key was refused.
///
/// The likeliest operator error there is, in the one file a repository adopting this tool
/// writes by hand, and `DeclaredPolicy` refuses it under `deny_unknown_fields` on purpose.
/// The reader's message naming the offending key was computed and discarded until
/// `no_verdict` existed, so this asserts the key itself reaches a caller rather than only
/// that something went wrong.
#[test]
fn Test_Run_Gate_Should_Name_The_Key_A_Malformed_Policy_Was_Refused_For()
{
    let root = Root_With_Policy(RootName("malformed"), PolicyText(r#"{ "basline": [] }"#));

    let result = Ran_Blocking(&root, GateCommand::default(), 0);

    assert_eq!(result.disposition, GateRunOutcome::Indeterminate);
    let Some(NoVerdict::MalformedPolicy(detail)) = result.no_verdict
    else
    {
        panic!("expected a malformed policy, got {:?}", result.no_verdict);
    };
    assert!(detail.contains("basline"), "{detail}");
    // The judging still happened and is still reported in full. Refusing the verdict is
    // not refusing the answer, which is what Run_Gate's own comment promises.
    assert!(matches!(result.check_outcome, CheckOutcome::Judged { .. }));
}

/// A run that reaches a real verdict records no reason for one, so a caller reading
/// `no_verdict` on an ordinary run is told nothing rather than something empty.
#[test]
fn Test_Run_Gate_Should_Record_No_Reason_When_It_Reached_A_Verdict()
{
    let root = Repository_Root();

    let result = Ran_Blocking(&root, GateCommand::default(), 0);

    assert_eq!(result.disposition, GateRunOutcome::Failed);
    assert_eq!(result.no_verdict, None);
}

#[test]
fn Test_Run_Gate_Should_Fail_On_A_Blocking_Finding()
{
    let root = Repository_Root();

    let result = Ran_Blocking(&root, GateCommand::default(), 0);

    assert!(!result.findings.blocking_findings.is_empty());
}

/// Every rule the fixture's own blocking findings name becomes one phase, and approving it
/// passes the run without hiding the finding.
///
/// The rules are read off a real run rather than hard-coded: this fixture is shared with
/// [`Test_Run_Gate_Should_Fail_On_A_Blocking_Finding`] and may trip more than one rule
/// (`completeness-mirror` and `single-letter-names` both plausibly apply to `pub const T`),
/// and a phase that named only one of them would leave the other unphased, which is a
/// different test.
#[test]
fn Test_Run_Gate_Should_Pass_When_A_Phase_Approval_Covers_Every_Blocking_Finding()
{
    let root = Repository_Root();
    let unphased = Ran_Blocking(&root, GateCommand::default(), 0);
    let phase = GatePhase { name: "completeness".to_owned(), rules: Named_Rules(&unphased), threshold: PhaseThreshold::AnyBlockingFinding };
    let approval = PhaseApproval { phase: "completeness".to_owned(), rationale: "reviewed and accepted".to_owned() };
    let phased = GateCommand { phases: vec![phase], approvals: vec![approval], ..Default::default() };

    let result = Ran_Blocking(&root, phased, APPROVED_RUN_SEED);

    assert!(!result.findings.blocking_findings.is_empty(), "the finding must still be real and reported, not hidden");
    assert!(matches!(result.disposition, GateRunOutcome::Passed), "an approved phase covering every blocking finding must pass the run");
}

#[test]
fn Test_Run_Gate_Should_Stay_Failed_When_A_Blocking_Finding_Belongs_To_No_Declared_Phase()
{
    let root = Repository_Root();
    let phase = GatePhase { name: "unrelated".to_owned(), rules: vec![RuleId::New("naming-convention")], threshold: PhaseThreshold::AnyBlockingFinding };
    let command = GateCommand { phases: vec![phase], ..Default::default() };

    let result = Ran_Blocking(&root, command, UNPHASED_RUN_SEED);

    assert!(matches!(result.disposition, GateRunOutcome::Failed), "a phase policy must not let a finding outside its own scope silently stop blocking");
}

fn Test_Variant() -> BuildVariant
{
    return BuildVariant::New("test-target", "test-profile", "test-toolchain", std::iter::empty::<String>());
}

/// This repository's own real root -- [`Judged_Sources`]'s dependency step, through
/// `nomos_check_orchestration::Run`, runs `cargo metadata` against it regardless of what
/// sources a test hands in.
fn Repository_Root() -> PathBuf
{
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    return manifest.parent().and_then(std::path::Path::parent).and_then(std::path::Path::parent).map(PathBuf::from).expect("this crate sits three levels below the workspace root");
}
