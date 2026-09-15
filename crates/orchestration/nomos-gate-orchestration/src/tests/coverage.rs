//! What a declared coverage floor does to a run whose own judgment came back incomplete.

use super::{Command_At, Coverage_Debt_Fixture, Mirrored_Source, Ran_Over, Repository_Root, SourcePath};
use crate::{CoveragePolicy, GateCommand, GateRunOutcome};
use nomos_check_orchestration::{CheckOutcome, Claim};

/// [`CoveragePolicy::Unset`] -- `Default`, the state every existing caller is in -- leaves a
/// run's disposition exactly as it always was: `Passed`, even though the run could not
/// materialize a fact for `broken.rs` and its own recomputed `Claim` is `Incomplete`. `Claim`
/// still rides through `check_outcome` for information only, unchanged from every increment
/// before this one -- `OD-GATE-016`'s own "unset behavior is provably unchanged" clause.
#[test]
fn Test_An_Unset_Coverage_Policy_Should_Leave_A_Passed_Disposition_Alone()
{
    let result = Ran_Over(Coverage_Debt_Fixture(), &Command_At(Repository_Root()));

    assert_eq!(result.disposition, GateRunOutcome::Passed);
    let CheckOutcome::Judged { claim, .. } = result.check_outcome
    else
    {
        panic!("a provider that read at least one file must still be judged");
    };
    assert_eq!(claim, Claim::Incomplete, "the fixture must actually be incomplete coverage, or this test proves nothing");
}

/// The same run under [`CoveragePolicy::RequireCompleteness`] reports [`GateRunOutcome::
/// Indeterminate`] instead of the `Passed` [`Test_An_Unset_Coverage_Policy_Should_Leave_A_
/// Passed_Disposition_Alone`] reports for the identical fixture -- [`crate::Run_Gate`] recomputed
/// `Claim` over the rule-and-scope-selected findings, found it incomplete, and refused to
/// let that read as a clean run. `OD-GATE-016`'s own decision.
#[test]
fn Test_Required_Completeness_Should_Downgrade_An_Incomplete_Passed_Run()
{
    let command = GateCommand { coverage: CoveragePolicy::RequireCompleteness, ..Command_At(Repository_Root()) };

    let result = Ran_Over(Coverage_Debt_Fixture(), &command);

    assert_eq!(result.disposition, GateRunOutcome::Indeterminate);
    assert!(
        result.findings.blocking_findings.is_empty(),
        "coverage must never manufacture a blocking finding: {:?}",
        result.findings.blocking_findings
    );
}

/// [`CoveragePolicy::RequireCompleteness`] does not touch a run that already reports
/// [`GateRunOutcome::Failed`]: a real blocking finding this run did reach a judgment about
/// is not made any less true by `broken.rs`, an unrelated subject the run could not judge --
/// [`CoveragePolicy::RequireCompleteness`]'s own doc says why this variant leaves `Failed`
/// alone rather than downgrading it the way it downgrades `Passed`.
#[test]
fn Test_Required_Completeness_Should_Not_Touch_A_Failed_Run()
{
    let phantom = Mirrored_Source(SourcePath("phantom.rs"));
    let mut sources = Coverage_Debt_Fixture();
    sources.push(phantom);
    let command = GateCommand { coverage: CoveragePolicy::RequireCompleteness, ..Command_At(Repository_Root()) };

    let result = Ran_Over(sources, &command);

    assert_eq!(result.disposition, GateRunOutcome::Failed);
    assert!(!result.findings.blocking_findings.is_empty());
}
