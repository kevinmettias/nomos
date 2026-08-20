//! Composing an already-walked tree into a real `nomos gate run`, apart from choosing a
//! platform, walking a tree or rendering the answer.

use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::Finding;
use nomos_platform::ProcessLauncher;
use nomos_rules::SourceFile;
use nomos_workspace::BuildVariant;

use crate::{Disposition, GateCommand, GateRunOutcome, GateRunResult, RuleSelector, ScopeSelector};

/// Judges `walked` exactly as `nomos check` would, and reduces the result to a
/// [`GateRunResult`].
///
/// `walked` is the walk, already done and already decided by the composition root, the same
/// reason `nomos_check_orchestration::Run` takes `sources` rather than a root to read:
/// `None` for a root that was not a directory, `Some(sources)` otherwise -- including the
/// empty case, so the no-source-found decision stays visible to a caller rather than
/// collapsing into `Some` versus `None`. `command.scope` narrows `walked` before `Run` is
/// called; a walk that becomes empty after scoping is `CheckOutcome::NoSource`, the same
/// state an empty walk already was, because both mean "nothing was judged" to a caller.
/// `variant` and `launcher` cross to [`nomos_check_orchestration::Run`] unchanged; see its
/// own documentation for why each is a composition-root value this crate cannot compute for
/// itself.
#[must_use]
pub fn Run_Gate<P: ProcessLauncher>(walked: Option<Vec<SourceFile>>, variant: BuildVariant, command: &GateCommand, launcher: &P) -> GateRunResult
{
    let outcome = match walked.map(|sources| return Scoped(sources, &command.scope))
    {
        None => CheckOutcome::Unreadable,
        Some(sources) if sources.is_empty() => CheckOutcome::NoSource,
        Some(sources) => nomos_check_orchestration::Run(&sources, variant, &command.root, launcher),
    };

    let (blocking_findings, disposition) = Reduced(&outcome, &command.rules);

    return GateRunResult { root: command.root.clone(), check_outcome: outcome, blocking_findings, disposition };
}

/// `sources` narrowed to what `scope` admits.
fn Scoped(sources: Vec<SourceFile>, scope: &ScopeSelector) -> Vec<SourceFile>
{
    return sources.into_iter().filter(|source| return scope.Matches(&source.path)).collect();
}

/// The blocking findings and the disposition they imply, read off a [`CheckOutcome`]
/// this function does not own and must not consume -- `check_outcome` still has to end up
/// in [`GateRunResult`] afterward. `rules` narrows which findings count before either is
/// computed; a finding whose rule is not selected can neither block nor move the
/// disposition, but it still exists in `check_outcome` untouched.
fn Reduced(outcome: &CheckOutcome, rules: &RuleSelector) -> (Vec<Finding>, GateRunOutcome)
{
    let CheckOutcome::Judged { findings, .. } = outcome
    else
    {
        return (Vec::new(), GateRunOutcome::Indeterminate);
    };

    let selected: Vec<Finding> = findings.iter().filter(|finding| return rules.Matches(&finding.rule)).cloned().collect();
    let disposition = Disposition(&selected);
    let blocking_findings: Vec<Finding> =
        selected.into_iter().filter(|finding| return finding.Can_Fail_A_Build()).collect();

    return (blocking_findings, disposition);
}
