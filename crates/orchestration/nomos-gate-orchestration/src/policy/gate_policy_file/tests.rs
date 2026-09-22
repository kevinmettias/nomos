//! What `nomos-gate.json` must mean once it is read: the policy families it resolves, the
//! refusals it owes the author who wrote a contradictory entry, and the precedence a
//! caller-constructed policy keeps over the file.
//!
//! Split out of `gate_policy_file.rs` when that file passed the workspace's own 500-line review
//! trigger. Nothing here changed meaning in the move; what did change is that each fixture now
//! takes the name of its scratch root in a [`ScratchName`] rather than a bare `&str`, because a
//! name and the policy text beside it are two arguments a caller could hand over in either
//! order and have the compiler accept both.

use super::{GatePolicyError, GatePolicyFile, Resolve_Gate_Policy, GATE_POLICY_FILE};
use crate::policy::Resolved_Gate_Policy;
use crate::{BaselineAllowance, CoveragePolicy, GateCommand, GatePhase, PhaseApproval, PhaseThreshold, SuppressionDisposition};
use nomos_contracts::RuleId;
use nomos_model::Subject_Of_Path;
use nomos_platform_std::StdFileSystem;
use std::path::PathBuf;

/// The distinguishing part of a scratch root's directory name.
///
/// Its own type rather than the `&str` it wraps, because every fixture takes it in a position
/// adjacent to the policy text it has nothing in common with: two adjacent `&str` parameters are
/// a pair a caller can hand over in the wrong order with the compiler's full agreement.
struct ScratchName(&'static str);

/// The occurrence count the declared entry below names, and the allowance it must resolve to.
///
/// One name for both, interpolated into the fixture text and asserted against, so the number the
/// file states and the number the assertion expects cannot drift apart.
const ACCEPTED_OCCURRENCES: u32 = 3;

/// The instant the declared waiver's expiry names, in whole Unix seconds -- the same number the
/// fixture file states and the assertion below reads back.
const DECLARED_EXPIRY_SECONDS: i64 = 1000;

/// The number of blocking findings the first declared phase below tolerates, read back by the
/// assertion that the number in the file reaches the type a run judges with.
const DECLARED_PHASE_TOLERANCE: usize = 2;

/// A policy file with every one of the six policy families non-default.
const EVERY_POLICY_FAMILY: &str = r#"{
    "suppressions": [
        {
            "rule": "naming-convention",
            "path": "src/generated.rs",
            "disposition": "false-positive",
            "rationale": "generated code",
            "owner": "author"
        }
    ],
    "baseline": [
        { "rule": "todo-format", "path": "src/legacy.rs", "rationale": "pre-existing" }
    ],
    "adoption": [
        { "rule": "deprecation", "rationale": "adopting incrementally" }
    ],
    "coverage": "require-completeness",
    "phases": [
        { "name": "naming", "rules": [ "naming-convention" ], "threshold": { "max-blocking-findings": 2 } },
        { "name": "dependencies", "rules": [ "dependency-direction" ] }
    ],
    "approvals": [
        { "phase": "naming", "rationale": "reviewed with the owning team" }
    ]
}"#;

/// The quantity `OD-GATE-030` v2 requires an author to be able to state, read off the file.
#[test]
fn Test_A_Declared_Entry_Should_Carry_The_Occurrence_Count_It_Accepted()
{
    let root = Root_With_Policy(
        ScratchName("accepted-count"),
        &format!(r#"{{ "baseline": [ {{ "rule": "todo-format", "path": "src/legacy.rs", "rationale": "adopted", "accepted_occurrence_count": {ACCEPTED_OCCURRENCES} }} ] }}"#),
    );

    let resolved = Resolve_Gate_Policy(&root, &StdFileSystem).expect("Root_With_Policy wrote this file, so it is present and readable").expect("the fixture declares the shape this module parses, so it resolves");

    assert_eq!(resolved.baseline.debt.first().expect("one entry").allowance, BaselineAllowance::AtMost(ACCEPTED_OCCURRENCES));
}

/// An entry naming no count keeps the meaning it was written under.
///
/// `OD-GATE-030` v2 decides this against the alternative of reading absence as one
/// occurrence, which would begin blocking builds over debt a repository did adopt. Every
/// entry authored before the key existed is this case, so it is the migration path and not
/// an edge.
#[test]
fn Test_An_Entry_Naming_No_Count_Should_Be_Unbounded_Rather_Than_Assumed()
{
    let root = Root_With_Policy(
        ScratchName("no-count"),
        r#"{ "baseline": [ { "rule": "todo-format", "path": "src/legacy.rs", "rationale": "adopted before the count existed" } ] }"#,
    );

    let resolved = Resolve_Gate_Policy(&root, &StdFileSystem).expect("Root_With_Policy wrote this file, so it is present and readable").expect("the fixture names no count, which this module reads as unbounded rather than refusing");

    assert_eq!(resolved.baseline.debt.first().expect("one entry").allowance, BaselineAllowance::Unbounded);
}

/// Zero is refused to the author rather than stored.
///
/// An entry accepting no occurrences tolerates nothing, so its only effect would be to block
/// exactly what writing it claims to permit. The refusal names the entry and says what to do
/// instead, the same shape `DeclaredSuppression::Problem` already refuses in.
#[test]
fn Test_An_Entry_Accepting_Zero_Occurrences_Should_Be_Refused()
{
    let root = Root_With_Policy(
        ScratchName("zero-count"),
        r#"{ "baseline": [ { "rule": "todo-format", "path": "src/legacy.rs", "rationale": "adopted", "accepted_occurrence_count": 0 } ] }"#,
    );

    let refusal = Resolve_Gate_Policy(&root, &StdFileSystem).expect_err("zero is refused");

    let GatePolicyError::Malformed(sentence) = refusal
    else
    {
        panic!("a declared entry this module read and rejected is malformed, not unreadable");
    };
    assert!(sentence.contains("accepts zero occurrences"), "{sentence}");
    assert!(sentence.contains("leaving the entry out"), "the refusal has to say what to do instead: {sentence}");
}

/// Every refusal this module writes reaches its author as single-spaced prose.
///
/// The defect this catches is invisible in the source and appears only in the output: a
/// sentence wrapped across source lines with the wrapping whitespace left inside the
/// literal renders as one line carrying a run of spaces where each break was, so a reader
/// is handed text that reads as a formatting accident at the exact moment they are being
/// told how to repair their policy. The assertion is therefore on the rendered string and
/// not on the literals -- a test reading the source would have to re-derive what the
/// wrapping renders to, which is the defect restated rather than caught.
///
/// Every one of them, because each is a separately written sentence and a fix applied to
/// one leaves the others rendering collapsed with nothing to say so. The phrase each case
/// must carry is asserted beside the spacing for the same reason: a fixture whose
/// disposition spelling `serde` does not know is refused by `serde` instead, and a
/// single-spaced parse error would satisfy a spacing assertion while proving nothing
/// about the refusal it was written for.
///
/// That second assertion is what makes this table the phase families' own linkage test as
/// well: each of the last four cases is refused by a rule
/// `crate::policy::gate_policy_file::declared_phases` states and `serde` cannot, so a phrase
/// arriving here at all proves [`Resolve_Gate_Policy`] consults it rather than resolving a
/// declaration the types cannot act on.
#[test]
fn Test_Every_Refusal_Sentence_Should_Render_As_Single_Spaced_Prose()
{
    let refusals = [
        (
            "single-spaced-waiver-without-expiry",
            r#"{ "suppressions": [ { "rule": "todo-format", "path": "src/legacy.rs", "disposition": "temporary-waiver", "rationale": "while the rename lands", "owner": "author" } ] }"#,
            "names no expiry",
        ),
        (
            "single-spaced-expiry-on-a-non-waiver",
            r#"{ "suppressions": [ { "rule": "todo-format", "path": "src/legacy.rs", "disposition": "false-positive", "rationale": "generated code", "owner": "author", "expiry": 1000 } ] }"#,
            "names an expiry",
        ),
        (
            "single-spaced-zero-count",
            r#"{ "baseline": [ { "rule": "todo-format", "path": "src/legacy.rs", "rationale": "adopted", "accepted_occurrence_count": 0 } ] }"#,
            "accepts zero occurrences",
        ),
        (
            "single-spaced-phase-without-rules",
            r#"{ "phases": [ { "name": "empty", "rules": [] } ] }"#,
            "names no rules",
        ),
        (
            "single-spaced-zero-threshold",
            r#"{ "phases": [ { "name": "naming", "rules": [ "naming-convention" ], "threshold": { "max-blocking-findings": 0 } } ] }"#,
            "zero blocking findings",
        ),
        (
            "single-spaced-repeated-phase-name",
            r#"{ "phases": [ { "name": "one", "rules": [ "naming-convention" ] }, { "name": "one", "rules": [ "dependency-direction" ] } ] }"#,
            "two phases are named",
        ),
        (
            "single-spaced-unmatched-approval",
            r#"{ "phases": [ { "name": "naming", "rules": [ "naming-convention" ] } ], "approvals": [ { "phase": "nameing", "rationale": "typo" } ] }"#,
            "matches no phase this file declares",
        ),
    ];

    for (name, policy, expected) in refusals
    {
        let sentence = Refused_Sentence(ScratchName(name), policy);

        assert!(
            sentence.contains(expected),
            "{name}: this is not the refusal it was written for, so it proves nothing about one: {sentence:?}"
        );
        assert_eq!(
            sentence.split_whitespace().collect::<Vec<_>>().join(" "),
            sentence,
            "{name}: this refusal does not reach its author as single-spaced prose"
        );
    }
}

/// The sentence [`Resolve_Gate_Policy`] hands an author who wrote `contents`.
fn Refused_Sentence(name: ScratchName, contents: &str) -> String
{
    let label = name.0;
    let root = Root_With_Policy(name, contents);
    let refusal = Resolve_Gate_Policy(&root, &StdFileSystem).expect_err("a refused entry is what this fixture writes");

    let GatePolicyError::Malformed(sentence) = refusal
    else
    {
        panic!("{label}: a declared entry this module read and rejected is malformed, not unreadable");
    };

    return sentence;
}

#[test]
fn Test_A_Root_With_No_Policy_File_Should_Resolve_To_Nothing_Rather_Than_Refusing()
{
    let root = Scratch_Root(ScratchName("absent"));

    assert_eq!(Resolve_Gate_Policy(&root, &StdFileSystem), Ok(None));
}

/// The end-to-end authoring case: every one of the six families non-default, from text.
#[test]
fn Test_A_Declared_File_Should_Resolve_Every_Policy_Family()
{
    let root = Root_With_Policy(ScratchName("every-family"), EVERY_POLICY_FAMILY);

    let resolved = Resolve_Gate_Policy(&root, &StdFileSystem).expect("Root_With_Policy wrote this file, so it is present and readable").expect("every entry in the fixture declares one of the six dispositions, so it resolves");

    assert_eq!(resolved.suppressions.suppressions.len(), 1);
    assert_eq!(resolved.baseline.debt.len(), 1);
    assert_eq!(resolved.adoption.calibrated.len(), 1);
    assert_eq!(resolved.coverage, CoveragePolicy::RequireCompleteness);
    assert_eq!(resolved.suppressions.suppressions.first().expect("one entry").disposition, SuppressionDisposition::FalsePositiveDisposition);
    assert_eq!(resolved.approvals.len(), 1);
    assert_eq!(
        resolved.phases.iter().map(|phase| return phase.threshold).collect::<Vec<PhaseThreshold>>(),
        vec![PhaseThreshold::MaxBlockingFindings { max: DECLARED_PHASE_TOLERANCE }, PhaseThreshold::AnyBlockingFinding],
        "the declared stages resolve in the order the file lists them, and the second names no threshold"
    );
}

/// A file naming neither phase family leaves a run in the state every caller was in before
/// the keys existed -- `OD-GATE-029`: an absent key still means unset.
#[test]
fn Test_A_File_Naming_No_Phases_Should_Resolve_To_No_Phase_Policy()
{
    let root = Root_With_Policy(ScratchName("no-phases"), r#"{ "coverage": "require-completeness" }"#);

    let resolved = Resolve_Gate_Policy(&root, &StdFileSystem).expect("Root_With_Policy wrote this file, so it is present and readable").expect("a file naming one family and not the others resolves");

    assert!(resolved.phases.is_empty(), "an absent `phases` is no phase policy, not an empty stage");
    assert!(resolved.approvals.is_empty());
}

/// The two phase families come from one source, never one each.
///
/// A caller that built stages in code keeps its own approvals -- including none -- because an
/// approval names the phase it covers, and a file's approval addressing a caller's stage would
/// be an approval for a phase its own source never declared. Asserted in both directions, so
/// neither half can be the one the precedence happens to agree with.
///
/// Through the layered resolution since `OD-POLICY-001`, which is where the coupling is now
/// declared as `crate::policy::effective_policy::PHASE_POLICY_UNIT`. The assertions are the
/// ones `GatePolicyFile::Resolved_Over` carried unchanged: what a run judges under did not move
/// when the rule was re-homed, and this is what says so.
#[test]
fn Test_A_Caller_That_Built_Phases_Should_Keep_Its_Own_Approvals()
{
    let from_file = GatePolicyFile {
        phases: vec![GatePhase { name: "declared".to_owned(), rules: vec![RuleId::New("naming-convention")], threshold: PhaseThreshold::AnyBlockingFinding }],
        approvals: vec![PhaseApproval { phase: "declared".to_owned(), rationale: "declared in the file".to_owned() }],
        ..GatePolicyFile::default()
    };
    let silent = GateCommand::default();

    let resolved = Resolved_Values(&from_file, &silent);

    assert_eq!(resolved.phases.len(), 1, "a command stating no phase takes the file's");
    assert_eq!(resolved.approvals.len(), 1, "and the approvals that came with them");

    let stated = GateCommand {
        phases: vec![GatePhase { name: "built".to_owned(), rules: vec![RuleId::New("dependency-direction")], threshold: PhaseThreshold::AnyBlockingFinding }],
        ..GateCommand::default()
    };

    let resolved = Resolved_Values(&from_file, &stated);

    assert_eq!(resolved.phases.first().expect("the command's own phase").name, "built");
    assert!(resolved.approvals.is_empty(), "a command stating its own stages states its own approvals too, including none");
}

/// The property that makes a path authorable at all: the identity this computes is the
/// one a real walk files a finding under, because both call the same kernel function.
#[test]
fn Test_A_Declared_Path_Should_Resolve_To_The_Subject_A_Walk_Would_File_It_Under()
{
    let root = Root_With_Policy(
        ScratchName("subject-identity"),
        r#"{ "baseline": [ { "rule": "todo-format", "path": "src/legacy.rs", "rationale": "pre-existing" } ] }"#,
    );

    let resolved = Resolve_Gate_Policy(&root, &StdFileSystem).expect("Root_With_Policy wrote this file, so it is present and readable").expect("the fixture's path field is one this module reads, so it resolves");

    assert_eq!(resolved.baseline.debt.first().expect("one entry").subject, Subject_Of_Path("src/legacy.rs"));
}

#[test]
fn Test_A_File_That_Is_Not_The_Declared_Shape_Should_Refuse_Rather_Than_Read_As_Empty()
{
    let root = Root_With_Policy(ScratchName("malformed"), "{ not json");

    assert!(matches!(Resolve_Gate_Policy(&root, &StdFileSystem), Err(GatePolicyError::Malformed(_))));
}

/// A temporary waiver naming an expiry is read, and the date reaches the domain type.
#[test]
fn Test_A_Temporary_Waiver_With_An_Expiry_Should_Resolve_To_That_Instant()
{
    let root = Root_With_Policy(
        ScratchName("waiver-with-expiry"),
        &format!(r#"{{ "suppressions": [ {{ "rule": "naming-convention", "path": "src/lib.rs", "disposition": "temporary-waiver", "rationale": "bounded", "owner": "someone", "expiry": {DECLARED_EXPIRY_SECONDS} }} ] }}"#),
    );

    let policy = Resolve_Gate_Policy(&root, &StdFileSystem).expect("Root_With_Policy wrote this file, so it is present and readable").expect("the fixture pairs its waiver with an expiry, so it resolves");

    assert_eq!(
        policy.suppressions.suppressions.first().expect("one entry").expiry,
        Some(nomos_platform::Timestamp::From_Unix_Seconds(DECLARED_EXPIRY_SECONDS))
    );
}

/// A temporary waiver with no expiry is refused rather than accepted as permanent.
///
/// The whole point of the field. Accepting this entry would produce a waiver that never
/// ends, which is a formal-risk-acceptance the author did not declare and would not know
/// they had.
#[test]
fn Test_A_Temporary_Waiver_Without_An_Expiry_Should_Be_Refused()
{
    let root = Root_With_Policy(
        ScratchName("waiver-without-expiry"),
        r#"{ "suppressions": [ { "rule": "naming-convention", "path": "src/lib.rs", "disposition": "temporary-waiver", "rationale": "bounded", "owner": "someone" } ] }"#,
    );

    assert!(
        matches!(Resolve_Gate_Policy(&root, &StdFileSystem), Err(GatePolicyError::Malformed(_))),
        "a temporary waiver with no end date was accepted, so it suppresses forever"
    );
}

/// A disposition that is not a waiver is refused an expiry rather than ignoring it.
///
/// The converse control. Without it the field could be accepted anywhere and read
/// nowhere, telling a later reader a date that governs nothing -- the same failure
/// `deny_unknown_fields` refuses for a misspelled key.
#[test]
fn Test_A_Non_Waiver_With_An_Expiry_Should_Be_Refused()
{
    let root = Root_With_Policy(
        ScratchName("acceptance-with-expiry"),
        r#"{ "suppressions": [ { "rule": "naming-convention", "path": "src/lib.rs", "disposition": "formal-risk-acceptance", "rationale": "owned", "owner": "someone", "expiry": 1000 } ] }"#,
    );

    assert!(
        matches!(Resolve_Gate_Policy(&root, &StdFileSystem), Err(GatePolicyError::Malformed(_))),
        "a disposition with no end-date semantics accepted one, so the file now carries a              date nothing reads"
    );
}

/// `deny_unknown_fields` is what makes a misspelled key a refusal instead of a silently
/// ignored line, which for this file is the difference between a suppression that
/// applies and one that does not.
#[test]
fn Test_A_Misspelled_Key_Should_Refuse_Rather_Than_Be_Ignored()
{
    let root = Root_With_Policy(ScratchName("unknown-key"), r#"{ "supressions": [] }"#);

    assert!(matches!(Resolve_Gate_Policy(&root, &StdFileSystem), Err(GatePolicyError::Malformed(_))));
}

/// A field the caller built wins over the file's, and a field it left at its default takes the
/// file's -- the precedence this reader has always had, now one case of the layered resolution.
#[test]
fn Test_A_Policy_A_Caller_Built_Should_Win_Over_The_File()
{
    let from_file = GatePolicyFile { coverage: CoveragePolicy::RequireCompleteness, ..GatePolicyFile::default() };
    let command = GateCommand { coverage: CoveragePolicy::Unset, ..GateCommand::default() };

    assert_eq!(Resolved_Values(&from_file, &command).coverage, CoveragePolicy::RequireCompleteness);

    let stated = GateCommand { coverage: CoveragePolicy::RequireCompleteness, ..GateCommand::default() };
    let silent_file = GatePolicyFile::default();

    assert_eq!(Resolved_Values(&silent_file, &stated).coverage, CoveragePolicy::RequireCompleteness);
}

/// The values `from_file` and `command` resolve to, as `crate::Run_Gate` resolves them.
///
/// The file contributes at the `Repository` layer and the command at `CommandLine` above it,
/// which is the two-level case `GatePolicyFile::Resolved_Over` was before `OD-POLICY-001`
/// re-homed the rule. Nothing in either fixture states a companion of a unit without its
/// deciding field, which is the only thing this pair of sources can be refused for.
fn Resolved_Values(from_file: &GatePolicyFile, command: &GateCommand) -> GatePolicyFile
{
    return Resolved_Gate_Policy(Some(from_file), command)
        .expect("no fixture here states an approval without the phases it would name")
        .values;
}

/// A tree of this test's own, named so two tests never share one.
fn Scratch_Root(name: ScratchName) -> PathBuf
{
    let root = std::env::temp_dir().join(format!("nomos-gate-policy-{}-{}", name.0, std::process::id()));
    std::fs::create_dir_all(&root).expect("the parent of this scratch root is the system temp directory, which already exists");

    return root;
}

/// Writes `contents` as the policy file under a fresh root and returns that root.
fn Root_With_Policy(name: ScratchName, contents: &str) -> PathBuf
{
    let root = Scratch_Root(name);
    std::fs::write(root.join(GATE_POLICY_FILE), contents).expect("Scratch_Root created the directory this file is written into");

    return root;
}
