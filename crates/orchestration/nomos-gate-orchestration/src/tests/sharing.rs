//! What a judgment shared between callers must still answer each of them: the answer its own
//! inputs produce, never another caller's.
//!
//! `repository_judgments` hands one judgment to every caller whose inputs are equal. Each test
//! here asks for two judgments whose inputs differ in exactly one part, and asserts that each got
//! the answer its own inputs produce. A key that stopped telling that part apart would hand the
//! second caller the first one's answer and fail here -- where otherwise every test that asks for
//! only one of the two would go on passing over an answer that was never its own.
//!
//! One test per part of each consumer's key: the sources, the command and the run identity of a
//! run; the sources, the command and the query of an explanation. The run tests select the mirror
//! rule alone, which reaches no compiler-backed family, so they cost no database; an explanation
//! always judges every rule, and all but one of the explanations below are the same inputs other
//! tests in this crate already ask for, so the whole guard costs one database more than before.

use super::{
    Clean_Source, Command_At, Command_With_Suppression, Judged_Explanation, Judged_Run, Mirrored_Source, Repository_Root, SourcePath,
    Suppression_Of, Test_Run_Id,
};
use crate::{Explanation, FindingQuery, GateCommand, GateExplainResult, GateRunOutcome, RuleSelector};
use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::{Digest128, RuleId, RunId};
use nomos_rules::{COMPLETENESS_MIRROR, NAMING_CONVENTION};

/// The run identity the first of two otherwise-equal runs is recorded under.
const FIRST_RUN_SEED: u8 = 1;

/// The run identity the second is recorded under -- distinct from [`FIRST_RUN_SEED`], which is
/// all the test that uses both needs of it.
const SECOND_RUN_SEED: u8 = 2;

/// The repository's root with `rule` the only rule selected.
fn Selecting(rule: &str) -> GateCommand
{
    return GateCommand { rules: RuleSelector { include: vec![RuleId::New(rule)] }, ..Command_At(Repository_Root()) };
}

/// The mirror rule's query at `location` -- the query `explain`'s own fixtures ask at `a.rs`.
fn Mirror_Query_At(location: &str) -> FindingQuery
{
    return FindingQuery { rule: RuleId::New(COMPLETENESS_MIRROR), location: location.to_owned() };
}

/// A run identity of its own for `seed`.
fn Run_Seeded(seed: u8) -> RunId
{
    return RunId::From_Digest(Digest128::From_Bytes([seed; Digest128::BYTE_LENGTH]));
}

/// Whether `result`'s judgment carries a finding of the mirror rule.
fn Carries_A_Mirror_Finding(result: &GateExplainResult) -> bool
{
    let CheckOutcome::Judged { findings, .. } = &result.check_outcome
    else
    {
        panic!("every explanation here is over real sources, so it is judged: {:?}", result.check_outcome);
    };

    return findings.iter().any(|finding| return finding.rule == RuleId::New(COMPLETENESS_MIRROR));
}

#[test]
fn Test_Two_Runs_Differing_Only_In_Their_Sources_Should_Each_Get_Their_Own_Answer()
{
    let clean = Judged_Run(vec![Clean_Source(SourcePath("a.rs"))], &Selecting(COMPLETENESS_MIRROR), Test_Run_Id());
    let mirrored = Judged_Run(vec![Mirrored_Source(SourcePath("a.rs"))], &Selecting(COMPLETENESS_MIRROR), Test_Run_Id());

    assert_eq!(clean.disposition, GateRunOutcome::Passed, "{:?}", clean.findings);
    assert_eq!(mirrored.disposition, GateRunOutcome::Failed, "{:?}", mirrored.findings);
}

#[test]
fn Test_Two_Runs_Differing_Only_In_Their_Command_Should_Each_Get_Their_Own_Answer()
{
    let source = || return vec![Mirrored_Source(SourcePath("a.rs"))];

    let mirror_rule = Judged_Run(source(), &Selecting(COMPLETENESS_MIRROR), Test_Run_Id());
    let naming_rule = Judged_Run(source(), &Selecting(NAMING_CONVENTION), Test_Run_Id());

    assert_eq!(mirror_rule.disposition, GateRunOutcome::Failed, "{:?}", mirror_rule.findings);
    assert_eq!(naming_rule.disposition, GateRunOutcome::Passed, "{:?}", naming_rule.findings);
}

#[test]
fn Test_Two_Runs_Differing_Only_In_Their_Run_Identity_Should_Each_Get_Their_Own_Answer()
{
    let source = || return vec![Mirrored_Source(SourcePath("a.rs"))];

    let first = Judged_Run(source(), &Selecting(COMPLETENESS_MIRROR), Run_Seeded(FIRST_RUN_SEED));
    let second = Judged_Run(source(), &Selecting(COMPLETENESS_MIRROR), Run_Seeded(SECOND_RUN_SEED));

    assert_eq!(first.run, Run_Seeded(FIRST_RUN_SEED));
    assert_eq!(second.run, Run_Seeded(SECOND_RUN_SEED));
}

#[test]
fn Test_Two_Explanations_Differing_Only_In_Their_Sources_Should_Each_Get_Their_Own_Answer()
{
    let clean = Judged_Explanation(vec![Clean_Source(SourcePath("a.rs"))], &Command_At(Repository_Root()), &Mirror_Query_At("nowhere.rs"));
    let mirrored = Judged_Explanation(vec![Mirrored_Source(SourcePath("a.rs"))], &Command_At(Repository_Root()), &Mirror_Query_At("nowhere.rs"));

    assert!(!Carries_A_Mirror_Finding(&clean), "the clean source was judged as the mirrored one");
    assert!(Carries_A_Mirror_Finding(&mirrored), "the mirrored source was judged as the clean one");
}

#[test]
fn Test_Two_Explanations_Differing_Only_In_Their_Command_Should_Each_Get_Their_Own_Answer()
{
    let source = || return vec![Mirrored_Source(SourcePath("a.rs"))];
    let query = Mirror_Query_At("a.rs");

    let unmatched = Judged_Explanation(source(), &Command_At(Repository_Root()), &query);
    let Explanation::Found { finding, would_block: blocks_unmatched, .. } = unmatched.explanation
    else
    {
        panic!("the mirrored source is found at the location it sits at: {:?}", unmatched.explanation);
    };
    let suppressed = Judged_Explanation(source(), &Command_With_Suppression(Repository_Root(), Suppression_Of(&finding)), &query);

    assert!(blocks_unmatched, "the unsuppressed explanation was handed the suppressed one's answer");
    assert!(
        matches!(suppressed.explanation, Explanation::Found { would_block: false, .. }),
        "the suppressed explanation was handed the unsuppressed one's answer: {:?}",
        suppressed.explanation
    );
}

#[test]
fn Test_Two_Explanations_Differing_Only_In_Their_Query_Should_Each_Get_Their_Own_Answer()
{
    let source = || return vec![Mirrored_Source(SourcePath("a.rs"))];

    let named = Judged_Explanation(source(), &Command_At(Repository_Root()), &Mirror_Query_At("a.rs"));
    let elsewhere = Judged_Explanation(source(), &Command_At(Repository_Root()), &Mirror_Query_At("nowhere.rs"));

    assert!(matches!(named.explanation, Explanation::Found { .. }), "the query at a.rs was handed another query's answer: {:?}", named.explanation);
    assert_eq!(elsewhere.explanation, Explanation::NotFound, "the query at nowhere.rs was handed another query's answer");
}
