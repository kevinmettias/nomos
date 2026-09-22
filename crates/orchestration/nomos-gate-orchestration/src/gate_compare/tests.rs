//! The provenance-and-identity half of `compare`'s tests.
//!
//! Split out of `gate_compare.rs` when that file passed the workspace's own 500-line review
//! trigger and its six public types each needed a file of their own. Nothing here changed
//! meaning in the move: these are the tests about *which* run judged what, `reason_tests` holds
//! the ones about a finding's stated reason, and `occurrence_tests` the ones about occurrence
//! identity.

use super::*;
use crate::{GateCommand, GateEnvironment, RuleSelector, Run_Gate};
use nomos_contracts::{Digest128, GateCategory, SubjectId};
use nomos_platform_std::{StdEnvironment, StdFileSystem, StdProgramLauncher};
use nomos_rules::SourceFile;
use nomos_workspace::BuildVariant;

/// The identity of the run a comparison is made against.
///
/// One pair of identities serves every fixture in these modules rather than a fresh fill byte
/// per test: a comparison test asks which *side* a difference is on, never which number names
/// the run, and a named side is what a reader can hold on to.
pub(super) const BASELINE_RUN: RunId = Run_Id_Of(1);

/// The identity of the run being compared.
pub(super) const CANDIDATE_RUN: RunId = Run_Id_Of(2);

/// One `a.rs` that mirrors nothing, so a run over it reports no finding.
const CLEAN_SOURCE: &str = "pub const THINGS: &[&str] = &[\"a\"];\n";

/// One `a.rs` whose constant is mirrored nowhere, which is one real blocking finding.
const MIRRORED_SOURCE: &str = "/// Mirrored by `Test_Compare_Ghost`.\npub const THINGS: &[&str] = &[\"a\"];\n";

/// One `a.rs` carrying a function rather than a constant, so a run over it is clean for a
/// different reason than [`CLEAN_SOURCE`]'s.
const QUIET_SOURCE: &str = "pub fn Something() -> u32\n{\n    return 1;\n}\n";

/// The digest of the one finding the disposition-change fixture carries.
///
/// Named rather than a bare fill at the construction site, and a constant rather than a
/// literal, because the value is a fixture's identity and not a quantity anything computes.
const FINDING_SUBJECT: Digest128 = Digest128::From_Bytes([9; Digest128::BYTE_LENGTH]);

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

/// A scratch root's own name, distinct from the policy the root declares.
#[derive(Clone, Copy)]
struct RootName<'a>(&'a str);

/// A `nomos-gate.json` body, distinct from the root's name.
#[derive(Clone, Copy)]
struct PolicyText<'a>(&'a str);

/// A run identity from a fill byte.
///
/// `const` so that the identities below are constants rather than a call at each use: a fill
/// byte spelled at a call site is a number a reader has to decode, and a comparison test is
/// about which side a difference is on.
pub(super) const fn Run_Id_Of(fill: u8) -> RunId
{
    return RunId::From_Digest(Digest128::From_Bytes([fill; Digest128::BYTE_LENGTH]));
}

fn Source_File(path: SourcePath<'_>, text: SourceText<'_>) -> SourceFile
{
    return SourceFile::New(path.0, nomos_model::Subject_Of_Path(path.0), text.0.to_owned());
}

/// The licensed case: two runs differing only in their source are compatible, so the
/// difference between them is a difference in the repository and may be read as one.
///
/// This is the assertion that keeps the other three honest. A comparability that reported
/// a caveat on every comparison would satisfy `OD-GATE-031`'s letter and destroy the verb.
#[test]
fn Test_Two_Runs_Differing_Only_In_Source_Should_Be_Compatible()
{
    let clean = vec![Source_File(SourcePath("a.rs"), SourceText(CLEAN_SOURCE))];
    let dirty = vec![Source_File(SourcePath("a.rs"), SourceText(MIRRORED_SOURCE))];

    let baseline = Run_Over(clean, BASELINE_RUN);
    let candidate = Run_Over(dirty, CANDIDATE_RUN);

    let compared = Compare_Gate_Runs(&baseline, &candidate).expect("both sides were judged");

    assert!(!compared.added.is_empty(), "the fixture is supposed to differ");
    assert_eq!(compared.comparability, Comparability::Compatible);
}

/// One source, two declared policies: the comparison says so rather than reporting the
/// movement as the repository's.
///
/// The reachable instance of the whole defect, and the reason it is reachable is not
/// exotic: `Run_Gate` resolves `nomos-gate.json` from `command.root`, and `compare`
/// judges two roots, so comparing two checkouts compares two policies.
#[test]
fn Test_Two_Policies_Over_One_Source_Should_State_The_Policy_Difference()
{
    let lenient = Root_Declaring(RootName("lenient"), PolicyText(r#"{ "coverage": "unset" }"#));
    let strict = Root_Declaring(RootName("strict"), PolicyText(r#"{ "coverage": "require-completeness" }"#));

    let baseline = Run_Over_Root(&lenient, RuleSelector::default(), BASELINE_RUN);
    let candidate = Run_Over_Root(&strict, RuleSelector::default(), CANDIDATE_RUN);

    let compared = Compare_Gate_Runs(&baseline, &candidate).expect("both sides were judged");

    assert_eq!(compared.comparability, Comparability::CompatibleWith(vec![JudgmentDifference::Policy]));
}

/// One source and one policy, two selections: a side told to look at less is named as
/// such, rather than having its absent findings read as the other side's additions.
#[test]
fn Test_Two_Selections_Over_One_Source_Should_State_The_Selection_Difference()
{
    let root = Root_Declaring(RootName("selection"), PolicyText("{}"));
    let narrowed = RuleSelector { include: vec![RuleId::New(nomos_rules::COMPLETENESS_MIRROR)] };

    let baseline = Run_Over_Root(&root, RuleSelector::default(), BASELINE_RUN);
    let candidate = Run_Over_Root(&root, narrowed, CANDIDATE_RUN);

    let compared = Compare_Gate_Runs(&baseline, &candidate).expect("both sides were judged");

    assert_eq!(compared.comparability, Comparability::CompatibleWith(vec![JudgmentDifference::Selection]));
}

/// A side that does not say what judged it makes the pair incomparable, and is named.
///
/// `OD-GATE-031` decided this rather than assuming a match. A comparability claim made
/// from missing evidence is the failure the whole mechanism exists to prevent, so the
/// mechanism must not make one itself.
#[test]
fn Test_A_Run_That_Cannot_Say_What_Judged_It_Should_Make_The_Pair_Incomparable()
{
    let known = Result_With(BASELINE_RUN, Empty_Findings());
    let unknown = GateRunResult { provenance: None, ..Result_With(CANDIDATE_RUN, Empty_Findings()) };

    let compared = Compare_Gate_Runs(&known, &unknown).expect("both sides were judged");

    assert_eq!(compared.comparability, Comparability::Incomparable(vec![CANDIDATE_RUN]));
}

/// Neither side saying is both sides named, not one.
#[test]
fn Test_Two_Runs_That_Cannot_Say_Should_Both_Be_Named()
{
    let one = GateRunResult { provenance: None, ..Result_With(BASELINE_RUN, Empty_Findings()) };
    let other = GateRunResult { provenance: None, ..Result_With(CANDIDATE_RUN, Empty_Findings()) };

    let compared = Compare_Gate_Runs(&one, &other).expect("both sides were judged");

    assert_eq!(compared.comparability, Comparability::Incomparable(vec![BASELINE_RUN, CANDIDATE_RUN]));
}

/// Two runs over a tree that gained one real blocking finding between them: the
/// second names one real addition and nothing else -- the population
/// `P40-GATE-COMPARE-VERB`'s own done_when asks for, over two runs that actually
/// differ rather than two identical ones.
#[test]
fn Test_Compare_Gate_Runs_Should_Name_A_Finding_Added_Between_Two_Real_Runs()
{
    let clean = vec![Source_File(SourcePath("a.rs"), SourceText(CLEAN_SOURCE))];
    let with_violation = vec![Source_File(SourcePath("a.rs"), SourceText(MIRRORED_SOURCE))];

    let baseline = Run_Over(clean, BASELINE_RUN);
    let candidate = Run_Over(with_violation, CANDIDATE_RUN);

    let compared = Comparison_Of(&baseline, &candidate);

    assert_eq!(compared.baseline, BASELINE_RUN);
    assert_eq!(compared.candidate, CANDIDATE_RUN);
    assert_eq!(compared.added.len(), 1, "{:?}", compared.added);
    assert_eq!(compared.added.first().expect("asserted len 1 above").rule, RuleId::New(nomos_rules::COMPLETENESS_MIRROR));
    assert!(compared.removed.is_empty(), "{:?}", compared.removed);
    assert!(compared.changed.is_empty(), "{:?}", compared.changed);
}

/// The inverse direction: a finding present in the baseline and fixed by the
/// candidate is reported removed, not silently dropped.
#[test]
fn Test_Compare_Gate_Runs_Should_Name_A_Finding_Removed_Between_Two_Real_Runs()
{
    let with_violation = vec![Source_File(SourcePath("a.rs"), SourceText(MIRRORED_SOURCE))];
    let fixed = vec![Source_File(SourcePath("a.rs"), SourceText(CLEAN_SOURCE))];

    let baseline = Run_Over(with_violation, BASELINE_RUN);
    let candidate = Run_Over(fixed, CANDIDATE_RUN);

    let compared = Comparison_Of(&baseline, &candidate);

    assert!(compared.added.is_empty(), "{:?}", compared.added);
    assert_eq!(compared.removed.len(), 1, "{:?}", compared.removed);
    assert!(compared.changed.is_empty(), "{:?}", compared.changed);
}

/// Two runs over the identical tree name nothing at all -- the vacuity guard every
/// diff needs: a comparison that always finds *something* has stopped comparing.
#[test]
fn Test_Compare_Gate_Runs_Should_Name_Nothing_Between_Two_Identical_Runs()
{
    let sources = vec![Source_File(SourcePath("a.rs"), SourceText(QUIET_SOURCE))];

    let baseline = Run_Over(sources.clone(), BASELINE_RUN);
    let candidate = Run_Over(sources, CANDIDATE_RUN);

    let compared = Comparison_Of(&baseline, &candidate);

    assert!(compared.added.is_empty(), "{:?}", compared.added);
    assert!(compared.removed.is_empty(), "{:?}", compared.removed);
    assert!(compared.changed.is_empty(), "{:?}", compared.changed);
}

/// The same finding, present in both runs but reported through a different bucket,
/// is a disposition change -- not an addition and a removal that happen to cancel
/// out. Built directly against [`GateFindings`] rather than through a real policy
/// change, since no real caller constructs a non-default `AdoptionPolicy`,
/// `SuppressionPolicy` or `BaselinePolicy` yet (every one of their own module docs
/// says so) and this test's job is [`Compare_Gate_Runs`]'s own bucket comparison, not
/// a second proof that a policy this crate already tests elsewhere matches a finding.
#[test]
fn Test_Compare_Gate_Runs_Should_Name_A_Disposition_Change_For_The_Same_Finding_In_A_Different_Bucket()
{
    let moved = Moved_Finding();

    let blocking = Bucketed_Findings(&moved, FindingDisposition::Blocking);
    let suppressed = Bucketed_Findings(&moved, FindingDisposition::Suppressed);

    let baseline = Result_With(BASELINE_RUN, blocking);
    let candidate = Result_With(CANDIDATE_RUN, suppressed);

    let compared = Comparison_Of(&baseline, &candidate);

    assert!(compared.added.is_empty(), "{:?}", compared.added);
    assert!(compared.removed.is_empty(), "{:?}", compared.removed);
    assert_eq!(compared.changed.len(), 1, "{:?}", compared.changed);

    let change = compared.changed.first().expect("asserted len 1 above");

    assert_eq!(change.before, FindingDisposition::Blocking);
    assert_eq!(change.after, FindingDisposition::Suppressed);
}

/// A finding a raised evidence floor moved is a disposition change, never a removal.
///
/// This is what `FindingDisposition::BelowEvidenceFloor` is for and the whole reason the sixth
/// bucket needed a variant of its own: `Population_Of` walks the variant list, so a bucket with
/// no variant would be missing from one side of a comparison, which reads as the finding having
/// been fixed. `OD-GATE-034` calls that the false causal story, and it would be at its worst
/// here -- a repository raising its floor would be told its tree got cleaner.
#[test]
fn Test_Compare_Gate_Runs_Should_Name_A_Raised_Evidence_Floor_As_A_Change_Rather_Than_A_Removal()
{
    let moved = Moved_Finding();

    let baseline = Result_With(BASELINE_RUN, Bucketed_Findings(&moved, FindingDisposition::Blocking));
    let candidate = Result_With(CANDIDATE_RUN, Bucketed_Findings(&moved, FindingDisposition::BelowEvidenceFloor));

    let compared = Comparison_Of(&baseline, &candidate);

    assert!(compared.removed.is_empty(), "a finding the floor took must not read as removed: {:?}", compared.removed);
    assert!(compared.added.is_empty(), "{:?}", compared.added);
    assert_eq!(compared.changed.len(), 1, "{:?}", compared.changed);

    let change = compared.changed.first().expect("asserted len 1 above");

    assert_eq!(change.before, FindingDisposition::Blocking);
    assert_eq!(change.after, FindingDisposition::BelowEvidenceFloor);
}

/// The one finding the disposition-change test moves between two buckets.
fn Moved_Finding() -> Finding
{
    return Finding {
        address: None,
        rule: RuleId::New(nomos_rules::COMPLETENESS_MIRROR),
        subject: SubjectId::From_Digest(FINDING_SUBJECT),
        subject_name: "a.rs".to_owned(),
        applicability: nomos_contracts::Applicability::Supported,
        evidence: nomos_contracts::EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: "a real finding, moved to a different bucket between two runs".to_owned(),
        locations: vec!["a.rs".to_owned()],
    };
}

/// `finding` alone in `bucket`, with no suppression reason recorded against it.
///
/// Shared with `reason_tests`, which needs the same one-finding population and adds a recorded
/// reason to it.
pub(super) fn Bucketed_Findings(finding: &Finding, bucket: FindingDisposition) -> GateFindings
{
    let mut findings = Empty_Findings();

    match bucket
    {
        FindingDisposition::Blocking => findings.blocking_findings.push(finding.clone()),
        FindingDisposition::Calibrated => findings.calibrated_findings.push(finding.clone()),
        FindingDisposition::Suppressed => findings.suppressed_findings.push(finding.clone()),
        FindingDisposition::Baselined => findings.baselined_findings.push(finding.clone()),
        FindingDisposition::BaselineExceeded => findings.baseline_exceeded_findings.push(finding.clone()),
        FindingDisposition::BelowEvidenceFloor => findings.below_evidence_floor_findings.push(finding.clone()),
    }

    return findings;
}

pub(super) fn Result_With(run: RunId, findings: GateFindings) -> GateRunResult
{
    return GateRunResult {
        unmatched_policy: Vec::new(),
        run,
        root: std::path::PathBuf::from("."),
        check_outcome: nomos_check_orchestration::CheckOutcome::NoSource,
        findings,
        disposition: crate::GateRunOutcome::Indeterminate,
        no_verdict: None,
        // These tests are about what two runs' findings differ by, and say nothing about
        // which layer stated the policy either was judged under.
        policy: None,
        // One fixed provenance for every result this helper builds, so two of them are
        // judged alike by construction and a test about findings stays a test about
        // findings. A test that wants its two sides judged differently says so itself.
        provenance: Some(Identical_Provenance()),
    };
}

/// The provenance every [`Result_With`] result carries, so that two of them differ in
/// their findings and in nothing else.
fn Identical_Provenance() -> crate::GateRunProvenance
{
    let digest = Digest128::From_Bytes([0; Digest128::BYTE_LENGTH]);

    return crate::GateRunProvenance {
        source: digest,
        policy: digest,
        selection: digest,
        instrument: digest,
        at: nomos_platform::Timestamp::From_Unix_Seconds(0),
    };
}

fn Test_Variant() -> BuildVariant
{
    return BuildVariant::New("test-target", "test-profile", "test-toolchain", std::iter::empty::<String>());
}

/// Runs the gate over `sources`, scoped to `Check_Completeness_Mirrors` alone -- the
/// one rule every fixture below is written against, so no other registered rule's own
/// opinion of a hand-written fixture snippet can add an unexpected finding neither
/// test asked about.
fn Run_Over(sources: Vec<SourceFile>, run: RunId) -> GateRunResult
{
    let command = GateCommand {
        root: std::path::PathBuf::from("."),
        rules: RuleSelector { include: vec![RuleId::New(nomos_rules::COMPLETENESS_MIRROR)] },
        ..GateCommand::default()
    };

    let environment = GateEnvironment { variant: Test_Variant(), launcher: &StdProgramLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment, now: nomos_platform::Timestamp::From_Unix_Seconds(0) };

    return Run_Gate(Some(sources), environment, &command, run);
}

/// A scratch root carrying `policy` as its own `nomos-gate.json`, so two runs can be
/// judged under two different declared policies over identical source -- which is what
/// makes the policy case reachable through this workspace's own hosts at all.
fn Root_Declaring(name: RootName<'_>, policy: PolicyText<'_>) -> std::path::PathBuf
{
    let root = std::env::temp_dir().join(format!("nomos-gate-comparability-{}-{}", name.0, std::process::id()));
    std::fs::create_dir_all(&root).expect("this call creates the directory the write below uses");
    std::fs::write(root.join("nomos-gate.json"), policy.0).expect("the directory above was created just before this write");

    return root;
}

/// [`Run_Over`] with a root and a rule selection of its own.
fn Run_Over_Root(root: &std::path::Path, rules: RuleSelector, run: RunId) -> GateRunResult
{
    let command = GateCommand { root: root.to_path_buf(), rules, ..GateCommand::default() };
    let sources = vec![Source_File(SourcePath("a.rs"), SourceText(CLEAN_SOURCE))];
    let environment = GateEnvironment { variant: Test_Variant(), launcher: &StdProgramLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment, now: nomos_platform::Timestamp::From_Unix_Seconds(0) };

    return Run_Gate(Some(sources), environment, &command, run);
}

/// A run that found nothing, for the two tests below that are about provenance alone.
fn Empty_Findings() -> GateFindings
{
    return GateFindings {
        blocking_findings: Vec::new(),
        calibrated_findings: Vec::new(),
        suppressed_findings: Vec::new(),
        baselined_findings: Vec::new(),
        baseline_exceeded_findings: Vec::new(),
        below_evidence_floor_findings: Vec::new(),
        baseline_populations: Vec::new(),
        suppression_reasons: Default::default(),
    };
}

/// [`Compare_Gate_Runs`] over two fixture-built results, refused rather than answered when a
/// fixture's own population collides.
///
/// A helper so the reason the comparison cannot fail is written once, at the one place the
/// refusal is raised, instead of being restated by every fixture that relies on it. These
/// populations are injective by construction, so a refusal here is the guard firing on a
/// mistake in the fixture and not the case under test.
pub(super) fn Comparison_Of(baseline: &GateRunResult, candidate: &GateRunResult) -> GateCompareResult
{
    return Compare_Gate_Runs(baseline, candidate)
        .expect("these fixtures hold distinct occurrences; a collision here is the guard firing, not the case under test");
}
