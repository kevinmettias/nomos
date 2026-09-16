//! What a scope does to a rule answering a cross-file question: `no-orphan-modules` must give
//! the same answer to a narrowed run as to the whole walk the scope was taken from.

use super::{
    Command_At, Command_With_Suppression, Mirrored_Source, One_Real_Blocking_Finding, Ran_Over, Repository_Root, Source_File, SourcePath,
    SourceText, Suppression_Of,
};
use crate::{GateCommand, GateRunOutcome, ScopeSelector, Suppression, SuppressionDisposition};
use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::RuleId;
use nomos_model::Subject_Of_Path;
use nomos_rules::{SourceFile, NO_ORPHAN_MODULES, NO_SINGLE_LINE_FUNCTION_BODIES};

/// Every `no-orphan-modules` finding a run reported, as the text a reader would act on.
fn Orphan_Findings_Of(sources: Vec<SourceFile>, command: &GateCommand) -> Vec<String>
{
    let result = Ran_Over(sources, command);

    let CheckOutcome::Judged { findings, .. } = &result.check_outcome
    else
    {
        panic!("these fixtures are real source, so the run judges them: {:?}", result.check_outcome);
    };

    return findings
        .iter()
        .filter(|finding| return finding.rule == RuleId::New(NO_ORPHAN_MODULES))
        .map(|finding| return finding.subject_name.clone())
        .collect();
}

/// The property `OD-GATE-025` exists for, pinned in both directions: a narrowed run and a
/// whole-workspace run agree about the same file.
///
/// `src/thing.rs` is declared by `src/lib.rs` and is not an orphan. Before the scope moved
/// off the source set, `--include src/thing.rs` collected the file and dropped the `lib.rs`
/// that declares it, so `no-orphan-modules` answered a question about a world where nothing
/// declared it and reported a `Blocking` finding telling a reader to delete or re-declare
/// correct code. Both runs must now say the same thing about it, which is nothing.
#[test]
fn Test_A_Narrowed_Run_And_A_Whole_Run_Should_Agree_About_A_Declared_Module()
{
    let sources = || return vec![
        Source_File(SourcePath("crates/example/src/lib.rs"), SourceText("mod thing;\n")),
        Source_File(SourcePath("crates/example/src/thing.rs"), SourceText("pub fn Thing() {}\n")),
    ];
    let narrowed = GateCommand {
        scope: ScopeSelector { include: vec!["crates/example/src/thing.rs".to_owned()], exclude: Vec::new() },
        ..Command_At(Repository_Root())
    };

    let whole = Orphan_Findings_Of(sources(), &Command_At(Repository_Root()));
    let narrow = Orphan_Findings_Of(sources(), &narrowed);

    assert!(whole.is_empty(), "a declared module is not an orphan to a whole run: {whole:?}");
    assert!(narrow.is_empty(), "nor to a narrowed one -- the run that invented this is the defect: {narrow:?}");
}

/// The other half, so the fix is not bought by blinding the rule: a file nothing declares is
/// still reported, including when the run was narrowed to exactly that file.
///
/// Without this, every assertion above is satisfied by a rule that stopped answering.
#[test]
fn Test_A_Genuinely_Orphaned_Module_Should_Still_Be_Reported_By_A_Narrowed_Run()
{
    let sources = vec![
        Source_File(SourcePath("crates/example/src/lib.rs"), SourceText("mod thing;\n")),
        Source_File(SourcePath("crates/example/src/thing.rs"), SourceText("pub fn Thing() {}\n")),
        Source_File(SourcePath("crates/example/src/stray.rs"), SourceText("pub fn Stray() {}\n")),
    ];
    let narrowed = GateCommand {
        scope: ScopeSelector { include: vec!["crates/example/src/stray.rs".to_owned()], exclude: Vec::new() },
        ..Command_At(Repository_Root())
    };

    let reported = Orphan_Findings_Of(sources, &narrowed);

    assert_eq!(reported.len(), 1, "the one file nothing declares must still be named: {reported:?}");
    let named = reported.first().expect("asserted len 1 above");
    assert!(named.contains("stray.rs"), "{reported:?}");
}

/// `OD-GATE-024`'s surviving clause: an entry that matched no finding is named, so an author
/// can tell a mis-spelling from a finding that has since been fixed.
#[test]
fn Test_A_Declared_Entry_Matching_No_Finding_Should_Be_Reported()
{
    let unreachable = Suppression {
        rule: RuleId::New(NO_SINGLE_LINE_FUNCTION_BODIES),
        subject: Subject_Of_Path("a-file-this-run-never-saw.rs"),
        disposition: SuppressionDisposition::FalsePositiveDisposition,
        rationale: "test fixture".to_owned(),
        owner: "test".to_owned(),
        expiry: None,
    };
    let command = Command_With_Suppression(Repository_Root(), unreachable);
    let source = Source_File(SourcePath("b.rs"), SourceText("pub fn Named() {}\n"));

    let result = Ran_Over(vec![source], &command);

    assert_eq!(result.unmatched_policy.len(), 1, "{:?}", result.unmatched_policy);
    let named = result.unmatched_policy.first().expect("asserted len 1 above");
    assert!(named.contains("no-single-line-function-bodies"), "{named}");
    // Reported, not failed: a policy legitimately outlives the finding it was written for.
    assert_ne!(result.disposition, GateRunOutcome::Indeterminate, "an unmatched entry must not refuse the run");
}

/// The other half, without which the assertion above is satisfied by naming every entry: an
/// entry that did match its finding is not reported as unmatched.
#[test]
fn Test_A_Declared_Entry_That_Matched_Should_Not_Be_Reported_As_Unmatched()
{
    let source = || return Mirrored_Source(SourcePath("a.rs"));
    let real_finding = One_Real_Blocking_Finding(source);
    let command = Command_With_Suppression(Repository_Root(), Suppression_Of(&real_finding));

    let result = Ran_Over(vec![source()], &command);

    assert!(!result.findings.suppressed_findings.is_empty(), "the fixture's own suppression must have matched");
    assert!(result.unmatched_policy.is_empty(), "{:?}", result.unmatched_policy);
}
