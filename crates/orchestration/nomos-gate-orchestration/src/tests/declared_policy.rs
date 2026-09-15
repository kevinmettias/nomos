//! What a `nomos-gate.json` beside a root adds to a command that never mentioned it: the
//! declaration `Run_Gate` resolves itself, and the three answers it can give.

use super::{Command_At, Coverage_Debt_Fixture, Ran_Over, Source, SourcePath, SourceText};
use crate::GateRunOutcome;
use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::RuleId;
use nomos_rules::NO_SINGLE_LINE_FUNCTION_BODIES;
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

/// A path a fixture names its own tree by, so two fixtures in one test binary cannot share one.
#[derive(Clone, Copy)]
struct RootName<'a>(&'a str);

/// The text of a fixture's `nomos-gate.json`, distinct from the name of the tree it sits in.
#[derive(Clone, Copy)]
struct PolicyText<'a>(&'a str);

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
    let sources = || return vec![Source(SourcePath("b.rs"), SourceText("pub fn badName() {}\n")), Source(SourcePath("c.rs"), SourceText("// TODO fix this\npub fn Ok()\n{\n}\n"))];
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
    let source = Source(SourcePath("b.rs"), SourceText("pub fn badName() {}\n"));

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

/// A policy file that exists and cannot be parsed refuses the run. The check still happened
/// and `check_outcome` still carries it, but no verdict is reported, because the rules for
/// reaching one were unreadable -- a build that passed here would be passing under a policy
/// nobody authored.
#[test]
fn Test_A_Malformed_Policy_File_Should_Refuse_Rather_Than_Report_A_Verdict()
{
    let root = Root_Declaring(RootName("malformed"), PolicyText("{ not json"));
    let source = Source(SourcePath("b.rs"), SourceText("pub fn Named() {}\n"));

    let result = Ran_Over(vec![source], &Command_At(root));

    assert_eq!(result.disposition, GateRunOutcome::Indeterminate);
    assert!(matches!(result.check_outcome, CheckOutcome::Judged { .. }), "the check itself still ran: {:?}", result.check_outcome);
}
