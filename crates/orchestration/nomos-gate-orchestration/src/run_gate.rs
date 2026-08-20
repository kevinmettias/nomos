//! Composing an already-walked tree into a real `nomos gate run`, apart from choosing a
//! platform, walking a tree or rendering the answer.

use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::Finding;
use nomos_platform::ProcessLauncher;
use nomos_rules::SourceFile;
use nomos_workspace::BuildVariant;
use std::path::Path;

use crate::{Disposition, GateCommand, GateRunOutcome, GateRunResult, RuleSelector, ScopeSelector, SuppressionPolicy};

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
#[must_use]
pub fn Run_Gate<P: ProcessLauncher>(walked: Option<Vec<SourceFile>>, variant: BuildVariant, command: &GateCommand, launcher: &P) -> GateRunResult
{
    let scoped = walked.map(|sources| return Scoped(sources, &command.scope));
    let outcome = Judged(scoped, variant, &command.root, launcher);

    let (blocking_findings, suppressed_findings, disposition) = Reduced(&outcome, &command.rules, &command.suppressions);

    return GateRunResult {
        root: command.root.clone(),
        check_outcome: outcome,
        blocking_findings,
        suppressed_findings,
        disposition,
    };
}

/// `sources` narrowed to what `scope` admits.
fn Scoped(sources: Vec<SourceFile>, scope: &ScopeSelector) -> Vec<SourceFile>
{
    return sources.into_iter().filter(|source| return scope.Matches(&source.path)).collect();
}

/// The blocking findings, the findings a `Suppression` kept from blocking, and the
/// disposition they imply, read off a [`CheckOutcome`] this function does not own and must
/// not consume -- `check_outcome` still has to end up in [`GateRunResult`] afterward.
/// `rules` narrows which findings count before either list is computed; a finding whose
/// rule is not selected can be neither blocking nor suppressed, but it still exists in
/// `check_outcome` untouched. `suppressions` then splits what remains: a matched finding
/// moves from blocking to suppressed rather than disappearing.
fn Reduced(outcome: &CheckOutcome, rules: &RuleSelector, suppressions: &SuppressionPolicy) -> (Vec<Finding>, Vec<Finding>, GateRunOutcome)
{
    let CheckOutcome::Judged { findings, .. } = outcome
    else
    {
        return (Vec::new(), Vec::new(), GateRunOutcome::Indeterminate);
    };

    let selected: Vec<Finding> = findings.iter().filter(|finding| return rules.Matches(&finding.rule)).cloned().collect();
    let blockable: Vec<Finding> = selected.into_iter().filter(|finding| return finding.Can_Fail_A_Build()).collect();
    let (suppressed_findings, blocking_findings): (Vec<Finding>, Vec<Finding>) =
        blockable.into_iter().partition(|finding| return suppressions.Suppressing(finding).is_some());
    let disposition = Disposition(&blocking_findings);

    return (blocking_findings, suppressed_findings, disposition);
}
