//! What a `nomos-gate.json` beside a root adds to a command that never mentioned it: the
//! declaration `Run_Gate` resolves itself, and the three answers it can give.

use super::{Command_At, Coverage_Debt_Fixture, Mirrored_Source, Ran_Over, Source_File, SourcePath, SourceText};
use crate::{GateRunOutcome, GateRunResult};
use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::RuleId;
use nomos_rules::NO_SINGLE_LINE_FUNCTION_BODIES;
use std::collections::BTreeSet;
use std::path::PathBuf;

/// The policy file `Test_A_Declared_Policy_File_Should_Tolerate_Findings_A_Command_Never_Mentioned`
/// writes into its own tree: a suppression and a baseline entry a person wrote, each addressing a
/// rule by a path inside the tree rather than by a subject a caller computed.
const TOLERATING_POLICY: &str = r#"{
            "suppressions": [
                {
                    "rule": "no-single-line-function-bodies",
                    "path": "b.rs",
                    "disposition": "false-positive",
                    "rationale": "test fixture",
                    "owner": "test"
                }
            ],
            "baseline": [
                { "rule": "todo-format-is-todo-name-description-ticket", "path": "c.rs", "rationale": "test fixture" }
            ]
        }"#;

/// The one-setting file `Test_A_Declared_Coverage_Floor_Should_Reach_The_Disposition` writes, kept
/// out of the test body so that test reads as the three assertions it is.
const COVERAGE_FLOOR_POLICY: &str = r#"{ "coverage": "require-completeness" }"#;

/// The one-setting file `Test_A_Declared_Evidence_Floor_Should_Reach_The_Reduction_Without_Losing_A_Finding`
/// writes.
///
/// `authoritative` rather than a class below `Derived`: `OD-GATE-034` measured that no rule in
/// this workspace yields a finding weaker than `Derived`, so a floor at or below that class is
/// the identity over every real run and would say nothing about whether the declaration reached
/// the reduction at all. The strongest class is the one every real finding is below.
const EVIDENCE_FLOOR_POLICY: &str = r#"{ "evidence_floor": "authoritative" }"#;

/// The name every phase declared below carries, written once so the approval that names it
/// and the phase it names cannot drift apart.
const DECLARED_PHASE: &str = "declared";

/// The approval entry a declaring fixture writes to cover [`DECLARED_PHASE`].
const APPROVAL_OF_THE_DECLARED_PHASE: &str = r#"{ "phase": "declared", "rationale": "reviewed and accepted" }"#;

/// The tolerance the exceeded half of the threshold test declares.
///
/// One rather than zero, because a threshold of zero is refused to its author as
/// `any-blocking-finding` under another name, and this test is about the number reaching the
/// run rather than about that refusal.
const EXCEEDED_TOLERANCE: usize = 1;

/// A path a fixture names its own tree by, so two fixtures in one test binary cannot share one.
#[derive(Clone, Copy)]
struct RootName<'a>(&'a str);

/// The text of a fixture's `nomos-gate.json`, distinct from the name of the tree it sits in.
#[derive(Clone, Copy)]
struct PolicyText<'a>(&'a str);

/// A threshold as a declared phase writes it, distinct from the approvals beside it.
#[derive(Clone, Copy)]
struct ThresholdText<'a>(&'a str);

/// The approvals a declared file lists, distinct from the threshold above them.
#[derive(Clone, Copy)]
struct ApprovalsText<'a>(&'a str);

/// A tree of this test's own, named after `name` and otherwise empty.
///
/// `name` and `policy` are two adjacent string positions in [`Root_Declaring`], which this crate's
/// own gate reported: a caller can transpose them and the compiler will not object. Two named types
/// make the transposition a type error, and cost one line each.
fn Fixture_Root(name: RootName<'_>) -> PathBuf
{
    let root = std::env::temp_dir().join(format!("nomos-gate-orchestration-policy-{}-{}", name.0, std::process::id()));
    std::fs::create_dir_all(&root).expect("this fixture's own temp directory is created here, before anything is read out of it or written into it");

    return root;
}

/// [`Fixture_Root`] carrying `policy` as its `nomos-gate.json` -- the declared source
/// `Run_Gate` resolves from, rather than a policy handed to it in a `GateCommand`.
fn Root_Declaring(name: RootName<'_>, policy: PolicyText<'_>) -> PathBuf
{
    let root = Fixture_Root(name);
    std::fs::write(root.join("nomos-gate.json"), policy.0).expect("the directory this write needs was created by Fixture_Root on the line above");

    return root;
}

/// [`Fixture_Root`] with no `nomos-gate.json` in it, whether or not a previous run of this binary
/// left one at the same reused temp path.
fn Root_Without_Policy(name: RootName<'_>) -> PathBuf
{
    let root = Fixture_Root(name);
    if let Err(error) = std::fs::remove_file(root.join("nomos-gate.json"))
    {
        assert_eq!(
            error.kind(),
            std::io::ErrorKind::NotFound,
            "the directory was just created, so a removal that failed for any other reason would mean this fixture's tree is not the empty one it reads as"
        );
    }

    return root;
}

/// The end-to-end case this whole item exists for: a suppression and a baseline entry a
/// person wrote in a file, resolved by a real run, tolerating two real findings that would
/// otherwise block.
///
/// Both rules here are addressed by the file's own subject, which is what makes a
/// path-authored entry match them -- see `crate::policy::gate_policy_file`'s own doc for the
/// rules this does not yet reach and why.
#[test]
fn Test_A_Declared_Policy_File_Should_Tolerate_Findings_A_Command_Never_Mentioned()
{
    let sources = || return vec![Source_File(SourcePath("b.rs"), SourceText("pub fn badName() {}\n")), Source_File(SourcePath("c.rs"), SourceText("// TODO fix this\npub fn Ok()\n{\n}\n"))];
    let root = Root_Declaring(RootName("tolerates"), PolicyText(TOLERATING_POLICY));

    let result = Ran_Over(sources(), &Command_At(root));

    assert!(!result.findings.suppressed_findings.is_empty(), "the file's suppression must have matched a real finding");
    assert!(!result.findings.baselined_findings.is_empty(), "the file's baseline entry must have matched a real finding");
    assert!(
        !result.findings.blocking_findings.iter().any(|finding| return finding.rule == RuleId::New(NO_SINGLE_LINE_FUNCTION_BODIES)),
        "the suppressed rule must not still block: {:?}",
        result.findings.blocking_findings
    );
}

/// The same tree with no policy file resolves to today's behavior exactly, which is what
/// makes the file safe to add: every existing caller, and CI's own `gate run --root .`, sits
/// in this case.
#[test]
fn Test_A_Root_With_No_Policy_File_Should_Judge_Exactly_As_Before()
{
    let root = Root_Without_Policy(RootName("absent"));
    let source = Source_File(SourcePath("b.rs"), SourceText("pub fn badName() {}\n"));

    let result = Ran_Over(vec![source], &Command_At(root));

    assert!(result.findings.suppressed_findings.is_empty());
    assert!(result.findings.baselined_findings.is_empty());
    assert_eq!(result.disposition, GateRunOutcome::Failed);
}

/// A coverage floor nobody could set before: declared in the file, it downgrades a run that
/// would otherwise report `Passed` rather than riding along for information only.
#[test]
fn Test_A_Declared_Coverage_Floor_Should_Reach_The_Disposition()
{
    let root = Root_Declaring(RootName("coverage"), PolicyText(COVERAGE_FLOOR_POLICY));

    let result = Ran_Over(Coverage_Debt_Fixture(), &Command_At(root));

    assert_ne!(result.disposition, GateRunOutcome::Failed, "this fixture must not block, so the floor is what is being observed");
    assert_eq!(result.disposition, GateRunOutcome::Indeterminate, "a declared coverage floor must reach the disposition");
}

/// A declared evidence floor reaches a real run's reduction, and takes nothing away from it.
///
/// Both trees are the same one fixture; only the declared file differs. The unfloored half is
/// asserted first and is not ceremony: a fixture that stopped producing a blocking finding
/// would make the floored half pass for the wrong reason, and this is the premise the second
/// half rests on.
///
/// `OD-GATE-034` is explicit that a below-floor finding is never dropped, so the assertion is
/// an equality between the two runs' populations rather than a check that one bucket is
/// non-empty: every finding that blocked without the floor is in the floor's own bucket with
/// it, and none went missing on the way.
#[test]
fn Test_A_Declared_Evidence_Floor_Should_Reach_The_Reduction_Without_Losing_A_Finding()
{
    let unfloored = Ran_Over(vec![Mirrored_Source(SourcePath("a.rs"))], &Command_At(Root_Without_Policy(RootName("no-evidence-floor"))));

    assert_eq!(unfloored.disposition, GateRunOutcome::Failed, "the fixture must block when no floor is declared, or the floor below proves nothing");
    assert!(unfloored.findings.below_evidence_floor_findings.is_empty(), "and nothing is floored when nothing declares a floor");

    let root = Root_Declaring(RootName("evidence-floor"), PolicyText(EVIDENCE_FLOOR_POLICY));
    let floored = Ran_Over(vec![Mirrored_Source(SourcePath("a.rs"))], &Command_At(root));

    assert!(floored.findings.blocking_findings.is_empty(), "a declared floor above every real class must empty the blocking bucket");
    assert_eq!(
        floored.findings.below_evidence_floor_findings, unfloored.findings.blocking_findings,
        "and the findings it took must be exactly the ones that blocked without it, in the bucket the floor moved them to"
    );
    assert_ne!(floored.disposition, GateRunOutcome::Failed, "a run whose only blocking findings are under the floor does not fail");
}

/// Every rule this run's own blocking findings name, so a declared phase can cover all of
/// them.
///
/// Read off a real run rather than written into the fixture, because [`Phased_Disposition`]
/// upgrades a failed run only when every blocking finding is named by some phase: a phase
/// naming only the rule whoever wrote the fixture had in mind would leave the rest unphased,
/// which is a different test and one that already exists.
///
/// [`Phased_Disposition`]: crate::Phased_Disposition
fn Blocking_Rules_Of(result: &GateRunResult) -> Vec<String>
{
    let named: BTreeSet<&str> = result.findings.blocking_findings.iter().map(|finding| return finding.rule.As_Str()).collect();
    assert!(!named.is_empty(), "the fixture must produce at least one blocking finding for a phase over it to mean anything");

    return named.into_iter().map(str::to_owned).collect();
}

/// A policy file declaring one phase named [`DECLARED_PHASE`] over `rules`, tolerating what
/// `threshold` states, with the approvals `approvals` states.
fn Declaring_One_Phase(rules: &[String], threshold: ThresholdText<'_>, approvals: ApprovalsText<'_>) -> String
{
    let named = rules.iter().map(|rule| return format!("\"{rule}\"")).collect::<Vec<String>>().join(", ");
    let threshold = threshold.0;
    let approvals = approvals.0;

    return format!(
        r#"{{ "phases": [ {{ "name": "{DECLARED_PHASE}", "rules": [ {named} ], "threshold": {threshold} }} ], "approvals": [ {approvals} ] }}"#
    );
}

/// The end-to-end case this item exists for: a stage and its approval, both written by a
/// person in a file, turning a run that fails without them into one that passes.
///
/// The discriminator is the file and nothing else. The same sources are judged twice under
/// the same command, and the only difference between the two runs is a `nomos-gate.json` that
/// the second tree has and the first does not.
#[test]
fn Test_A_Declared_Phase_With_An_Approval_Should_Pass_A_Run_That_Would_Otherwise_Fail()
{
    let sources = || return vec![Mirrored_Source(SourcePath("a.rs"))];
    let unphased = Ran_Over(sources(), &Command_At(Root_Without_Policy(RootName("phase-unphased"))));
    assert_eq!(unphased.disposition, GateRunOutcome::Failed, "the fixture must fail without a declared phase for this test to observe one");

    let declaration =
        Declaring_One_Phase(&Blocking_Rules_Of(&unphased), ThresholdText(r#""any-blocking-finding""#), ApprovalsText(APPROVAL_OF_THE_DECLARED_PHASE));
    let root = Root_Declaring(RootName("phase-approved"), PolicyText(declaration.as_str()));

    let result = Ran_Over(sources(), &Command_At(root));

    assert!(!result.findings.blocking_findings.is_empty(), "the findings must still be real and reported, not hidden by the approval");
    assert_eq!(result.disposition, GateRunOutcome::Passed, "a declared approval covering every blocking finding must pass the run");
}

/// The number a file writes is what decides, not the fact that a stage was declared.
///
/// Two declarations differing in that number alone: one names this run's own count of
/// blocking findings and tolerates them, the other names fewer and does not. A threshold that
/// never reached the run would give one answer to both.
#[test]
fn Test_A_Declared_Threshold_Should_Decide_Whether_Its_Phase_Tolerates_The_Findings()
{
    let sources = || return vec![Mirrored_Source(SourcePath("a.rs")), Mirrored_Source(SourcePath("b.rs"))];
    let unphased = Ran_Over(sources(), &Command_At(Root_Without_Policy(RootName("threshold-unphased"))));
    let rules = Blocking_Rules_Of(&unphased);
    let counted = unphased.findings.blocking_findings.len();
    assert!(counted > EXCEEDED_TOLERANCE, "this fixture must produce more than {EXCEEDED_TOLERANCE} blocking finding for two thresholds to differ over it");

    let tolerant = Declaring_One_Phase(&rules, ThresholdText(&format!(r#"{{ "max-blocking-findings": {counted} }}"#)), ApprovalsText(""));
    let exceeded =
        Declaring_One_Phase(&rules, ThresholdText(&format!(r#"{{ "max-blocking-findings": {EXCEEDED_TOLERANCE} }}"#)), ApprovalsText(""));

    let tolerated = Ran_Over(sources(), &Command_At(Root_Declaring(RootName("threshold-tolerant"), PolicyText(tolerant.as_str()))));
    let blocked = Ran_Over(sources(), &Command_At(Root_Declaring(RootName("threshold-exceeded"), PolicyText(exceeded.as_str()))));

    assert_eq!(tolerated.disposition, GateRunOutcome::Passed, "a declared threshold naming this run's own count tolerates it, unapproved");
    assert_eq!(blocked.disposition, GateRunOutcome::Failed, "and one naming fewer findings than the run produced does not");
}

/// A policy file that exists and cannot be parsed refuses the run. The check still happened
/// and `check_outcome` still carries it, but no verdict is reported, because the rules for
/// reaching one were unreadable -- a build that passed here would be passing under a policy
/// nobody authored.
#[test]
fn Test_A_Malformed_Policy_File_Should_Refuse_Rather_Than_Report_A_Verdict()
{
    let root = Root_Declaring(RootName("malformed"), PolicyText("{ not json"));
    let source = Source_File(SourcePath("b.rs"), SourceText("pub fn Named() {}\n"));

    let result = Ran_Over(vec![source], &Command_At(root));

    assert_eq!(result.disposition, GateRunOutcome::Indeterminate);
    assert!(matches!(result.check_outcome, CheckOutcome::Judged { .. }), "the check itself still ran: {:?}", result.check_outcome);
}
