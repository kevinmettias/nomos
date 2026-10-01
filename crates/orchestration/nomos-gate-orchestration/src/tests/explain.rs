//! What one named finding answers, and the two selections `explain` deliberately ignores.

use super::{
    Baseline_Of, Calibration_Of, Clean_Source, Command_At, Command_With_Baseline, Command_With_Calibration, Command_With_Suppression,
    Explained_Over, Mirrored_Source, Repository_Root, SourcePath, Suppression_Of,
};
use crate::{BaselineDebt, Explanation, FindingQuery, GateCommand, RuleCalibration, ScopeSelector, Suppression};
use nomos_contracts::{Finding, RuleId};
use nomos_rules::{SourceFile, COMPLETENESS_MIRROR};

/// The contract record `COMPLETENESS_MIRROR` cites, written out here rather than read off the
/// rule so that a citation which changed would fail the test below instead of moving with it.
const MIRROR_CONTRACT_RECORD: &str = "D-134";

/// The version [`MIRROR_CONTRACT_RECORD`] is cited at, named for the reason the record is.
const MIRROR_CONTRACT_VERSION: u32 = 2;

/// Answers `query` over `source` with every policy empty, and returns the finding it found --
/// the shared setup a suppression, a baseline and a calibration explain fixture all need
/// before any of them can address that finding with its own policy.
fn Real_Finding_For(query: &FindingQuery, source: SourceFile) -> Finding
{
    let unmatched = Explained_Over(vec![source], &Command_At(Repository_Root()), query);
    let Explanation::Found { finding, .. } = unmatched.explanation
    else
    {
        panic!("this fixture must produce the finding the query names");
    };

    return *finding;
}

/// The source, query and real finding a suppression, a baseline and a calibration `explain`
/// fixture all build identically, before each addresses that finding with its own policy.
///
/// A named result rather than a three-tuple: at three members the caller is counting positions,
/// and the third one is the finding while the second is the query that found it -- the pair a
/// positional reader swaps. The source is a built file rather than the callable that would build
/// one, because a stored callable is a collaborator nobody can name or substitute.
struct AppliesFixture
{
    source: SourceFile,
    query: FindingQuery,
    finding: Finding,
}

/// The source, query and real finding a suppression, a baseline and a calibration `explain`
/// fixture all build identically, before each addresses that finding with its own policy.
fn Explain_Applies_Fixture() -> AppliesFixture
{
    let source = Mirrored_Source(SourcePath("a.rs"));
    let query = FindingQuery { rule: RuleId::New(COMPLETENESS_MIRROR), location: "a.rs".to_owned() };
    let finding = Real_Finding_For(&query, source.clone());

    return AppliesFixture { source, query, finding };
}

/// The fields of `explanation`'s `Found` variant that the four explain fixtures below read,
/// each of them a different one.
///
/// A named result rather than a six-tuple: at that many members the caller is counting positions,
/// and two of the six are `Option`s of two different policies -- exactly the pair a positional
/// reader swaps while the types still compile.
struct ExplainedFindings
{
    finding: Finding,
    would_block: bool,
    contract: Option<(String, u32)>,
    calibrated_by: Option<RuleCalibration>,
    suppressed_by: Option<Suppression>,
    baselined_by: Option<BaselineDebt>,
}

/// Destructures `explanation`'s `Found` variant into [`ExplainedFindings`], or panics naming
/// what every fixture that reaches this helper has already asserted -- the explain-side
/// counterpart to `Judged_Findings`.
fn Explained_Found(explanation: Explanation) -> ExplainedFindings
{
    let Explanation::Found { finding, would_block, contract, calibrated_by, suppressed_by, baselined_by, .. } = explanation
    else
    {
        panic!("this fixture must still produce the finding the query names");
    };

    return ExplainedFindings { finding: *finding, would_block, contract, calibrated_by, suppressed_by, baselined_by };
}

/// A query naming a rule and location no finding carries is [`Explanation::NotFound`], not
/// a panic or a default -- the same "an absent answer is a typed state, not a shorter one"
/// discipline `CheckOutcome::NoSource` already keeps one layer down.
#[test]
fn Test_Explain_Should_Report_Not_Found_For_A_Query_Nothing_Answers()
{
    let sources = vec![Clean_Source(SourcePath("a.rs"))];
    let query = FindingQuery { rule: RuleId::New(COMPLETENESS_MIRROR), location: "nowhere.rs".to_owned() };

    let result = Explained_Over(sources, &Command_At(Repository_Root()), &query);

    assert!(matches!(result.check_outcome, nomos_check_orchestration::CheckOutcome::Judged { .. }));
    assert_eq!(result.explanation, Explanation::NotFound);
}

/// A real query naming the one blocking finding this fixture produces answers `Found`, with
/// `would_block` true and no suppression -- the everyday case, checked against a real judged
/// finding rather than a fixture built to look like one.
#[test]
fn Test_Explain_Gate_Should_Find_A_Real_Blocking_Finding()
{
    let sources = vec![Mirrored_Source(SourcePath("a.rs"))];
    let query = FindingQuery { rule: RuleId::New(COMPLETENESS_MIRROR), location: "a.rs".to_owned() };

    let result = Explained_Over(sources, &Command_At(Repository_Root()), &query);

    let explained = Explained_Found(result.explanation);
    assert_eq!(explained.finding.rule, RuleId::New(COMPLETENESS_MIRROR));
    assert!(explained.would_block);
    assert_eq!(explained.calibrated_by, None);
    assert_eq!(explained.suppressed_by, None);
    assert_eq!(explained.baselined_by, None);
    assert_eq!(
        explained.contract,
        Some((MIRROR_CONTRACT_RECORD.to_owned(), MIRROR_CONTRACT_VERSION)),
        "COMPLETENESS_MIRROR's own real contract citation"
    );
}

/// A [`crate::Suppression`] matching the queried finding flips `would_block` to `false` and names
/// itself in `suppressed_by` -- `explain` consults `command.suppressions` even though it
/// ignores `scope` and `rules`, because whether a suppression applies is part of this
/// finding's own explanation.
#[test]
fn Test_Explain_Should_Report_A_Suppression_That_Applies()
{
    let fixture = Explain_Applies_Fixture();
    let command = Command_With_Suppression(Repository_Root(), Suppression_Of(&fixture.finding));

    let result = Explained_Over(vec![fixture.source.clone()], &command, &fixture.query);

    let explained = Explained_Found(result.explanation);
    assert!(!explained.would_block);
    assert!(explained.suppressed_by.is_some());
}

/// A [`crate::BaselineDebt`] matching the queried finding flips `would_block` to `false` and
/// names itself in `baselined_by` -- mirrors
/// [`Test_Explain_Should_Report_A_Suppression_That_Applies`] for the second of `explain`'s two
/// consulted policies.
#[test]
fn Test_Explain_Should_Report_A_Baseline_That_Applies()
{
    let fixture = Explain_Applies_Fixture();
    let command = Command_With_Baseline(Repository_Root(), Baseline_Of(&fixture.finding));

    let result = Explained_Over(vec![fixture.source.clone()], &command, &fixture.query);

    let explained = Explained_Found(result.explanation);
    assert!(!explained.would_block);
    assert!(explained.baselined_by.is_some());
}

/// A [`crate::RuleCalibration`] matching the queried finding's rule flips `would_block` to
/// `false` and names itself in `calibrated_by` -- mirrors
/// [`Test_Explain_Should_Report_A_Suppression_That_Applies`] for the third of `explain`'s
/// three consulted policies.
#[test]
fn Test_Explain_Should_Report_A_Calibration_That_Applies()
{
    let fixture = Explain_Applies_Fixture();
    let command = Command_With_Calibration(Repository_Root(), Calibration_Of(&fixture.finding));

    let result = Explained_Over(vec![fixture.source.clone()], &command, &fixture.query);

    let explained = Explained_Found(result.explanation);
    assert!(!explained.would_block);
    assert!(explained.calibrated_by.is_some());
}

/// `explain` is independent of `command.scope`: a scope that would exclude `a.rs` from a
/// real `run` must not stop `explain` from finding and reporting the same query.
#[test]
fn Test_Explain_Should_Ignore_Scope()
{
    let sources = vec![Mirrored_Source(SourcePath("a.rs"))];
    let query = FindingQuery { rule: RuleId::New(COMPLETENESS_MIRROR), location: "a.rs".to_owned() };
    let command = GateCommand {
        scope: ScopeSelector { include: vec!["b.rs".to_owned()], exclude: Vec::new() },
        ..Command_At(Repository_Root())
    };

    let result = Explained_Over(sources, &command, &query);

    assert!(matches!(result.explanation, Explanation::Found { .. }));
}
