//! Composing an already-walked tree into a real `nomos gate run`, apart from choosing a
//! platform, walking a tree or rendering the answer.

use nomos_check_orchestration::{CheckOutcome, Claim, Claim_Of};
use nomos_contracts::{Finding, RunId};
use nomos_platform::ProcessLauncher;
use nomos_rules::SourceFile;
use nomos_workspace::BuildVariant;
use std::path::Path;

use crate::{
    AdoptionPolicy, BaselinePolicy, CoveragePolicy, Disposition, GateCommand, GateRunOutcome, GateRunResult, RuleSelector, ScopeSelector,
    SuppressionPolicy,
};

/// Judges `walked` exactly as `nomos check` would.
///
/// `walked` is the walk, already done and already decided by the composition root, the same
/// reason `nomos_check_orchestration::Run` takes `sources` rather than a root to read:
/// `None` for a root that was not a directory, `Some(sources)` otherwise -- including the
/// empty case, so the no-source-found decision stays visible to a caller rather than
/// collapsing into `Some` versus `None`. `variant` and `launcher` cross to
/// [`nomos_check_orchestration::Run`] unchanged; see its own documentation for why each is a
/// composition-root value this crate cannot compute for itself.
///
/// Shared by [`Run_Gate`] (over a `command.scope`-narrowed walk) and
/// [`crate::Explain_Gate`] (over the whole one, since explain answers a question about one
/// named finding, not a scope-narrowed disposition) — factored out so the two do not
/// duplicate this match.
pub(crate) fn Judged<P: ProcessLauncher>(walked: Option<Vec<SourceFile>>, variant: BuildVariant, root: &Path, launcher: &P) -> CheckOutcome
{
    return match walked
    {
        None => CheckOutcome::Unreadable,
        Some(sources) if sources.is_empty() => CheckOutcome::NoSource,
        Some(sources) => nomos_check_orchestration::Run(&sources, variant, root, launcher),
    };
}

/// Judges `walked` exactly as `nomos check` would, and reduces the result to a
/// [`GateRunResult`].
///
/// `command.scope` narrows `walked` before [`Judged`] runs; a walk that becomes empty after
/// scoping is `CheckOutcome::NoSource`, the same state an empty walk already was, because
/// both mean "nothing was judged" to a caller.
///
/// `run` identifies this execution and is not computed here -- `OD-WORKFLOW-001`'s amendment
/// decided a `RunId` identifies one execution, not one configuration, so this function must
/// not derive it from `command` or `variant` the way everything else it composes is derived.
/// The composition root supplies one, typically [`crate::Fresh_Run_Id`] over a real clock
/// reading.
#[must_use]
pub fn Run_Gate<P: ProcessLauncher>(walked: Option<Vec<SourceFile>>, variant: BuildVariant, command: &GateCommand, launcher: &P, run: RunId) -> GateRunResult
{
    let scoped = walked.map(|sources| return Scoped(sources, &command.scope));
    let outcome = Judged(scoped, variant, &command.root, launcher);

    let (blocking_findings, calibrated_findings, suppressed_findings, baselined_findings, disposition) =
        Reduced(&outcome, &command.rules, &command.adoption, &command.suppressions, &command.baseline, command.coverage);

    return GateRunResult {
        root: command.root.clone(),
        run,
        check_outcome: outcome,
        blocking_findings,
        calibrated_findings,
        suppressed_findings,
        baselined_findings,
        disposition,
    };
}

/// `sources` narrowed to what `scope` admits.
fn Scoped(sources: Vec<SourceFile>, scope: &ScopeSelector) -> Vec<SourceFile>
{
    return sources.into_iter().filter(|source| return scope.Matches(&source.path)).collect();
}

/// The blocking findings, the findings an `AdoptionPolicy` calibration kept from blocking,
/// the findings a `Suppression` kept from blocking, the findings a `BaselineDebt` kept from
/// blocking, and the disposition they imply, read off a [`CheckOutcome`] this function does
/// not own and must not consume -- `check_outcome` still has to end up in [`GateRunResult`]
/// afterward. `rules` narrows which findings count before any list is computed; a finding
/// whose rule is not selected can be neither blocking, calibrated, suppressed nor baselined,
/// but it still exists in `check_outcome` untouched. `adoption` splits what remains first --
/// a coarser, rule-wide override rather than a per-finding one -- then `suppressions` splits
/// what calibration did not match, then `baseline` splits what neither matched: a finding
/// matched by more than one reports as calibrated, not counted twice. `coverage` is
/// consulted last, over `rules`' own selection rather than any of the four lists it splits
/// into -- calibration, suppression and baseline each answer "does this blocking finding
/// still block," a question about one finding at a time, while `coverage` answers "did this
/// run reach a judgment about everything it selected," a question about the run as a whole.
fn Reduced(
    outcome: &CheckOutcome,
    rules: &RuleSelector,
    adoption: &AdoptionPolicy,
    suppressions: &SuppressionPolicy,
    baseline: &BaselinePolicy,
    coverage: CoveragePolicy,
) -> (Vec<Finding>, Vec<Finding>, Vec<Finding>, Vec<Finding>, GateRunOutcome)
{
    let CheckOutcome::Judged { findings, .. } = outcome
    else
    {
        return (Vec::new(), Vec::new(), Vec::new(), Vec::new(), GateRunOutcome::Indeterminate);
    };

    let selected: Vec<Finding> = findings.iter().filter(|finding| return rules.Matches(&finding.rule)).cloned().collect();
    let blockable: Vec<Finding> = selected.iter().filter(|finding| return finding.Can_Fail_A_Build()).cloned().collect();
    let (calibrated_findings, uncalibrated): (Vec<Finding>, Vec<Finding>) =
        blockable.into_iter().partition(|finding| return adoption.Calibrating(finding).is_some());
    let (suppressed_findings, remaining): (Vec<Finding>, Vec<Finding>) =
        uncalibrated.into_iter().partition(|finding| return suppressions.Suppressing(finding).is_some());
    let (baselined_findings, blocking_findings): (Vec<Finding>, Vec<Finding>) =
        remaining.into_iter().partition(|finding| return baseline.Tolerating(finding).is_some());
    let disposition = Reduced_With_Coverage(Disposition(&blocking_findings), coverage, &selected);

    return (blocking_findings, calibrated_findings, suppressed_findings, baselined_findings, disposition);
}

/// `outcome`, downgraded from [`GateRunOutcome::Passed`] to [`GateRunOutcome::Indeterminate`]
/// when `coverage` requires completeness and `Claim_Of(selected)` is [`Claim::Incomplete`] --
/// `OD-GATE-016`'s own decision. Leaves every other `outcome` untouched: unset, this is the
/// identity function, and [`CoveragePolicy::RequireCompleteness`]'s own doc says why a
/// `Failed` outcome is left alone rather than downgraded the same way.
fn Reduced_With_Coverage(outcome: GateRunOutcome, coverage: CoveragePolicy, selected: &[Finding]) -> GateRunOutcome
{
    return match (coverage, outcome)
    {
        (CoveragePolicy::RequireCompleteness, GateRunOutcome::Passed) if Claim_Of(selected) == Claim::Incomplete => GateRunOutcome::Indeterminate,
        (_, outcome) => outcome,
    };
}
