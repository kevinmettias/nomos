//! Composing an already-walked tree into a real `nomos gate run`, apart from choosing a
//! platform, walking a tree or rendering the answer.

use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::Finding;
use nomos_platform::ProcessLauncher;
use nomos_rules::SourceFile;
use nomos_workspace::BuildVariant;
use std::path::Path;

use crate::{Disposition, GateRunOutcome, GateRunResult};

/// Judges `walked` exactly as `nomos check` would, and reduces the result to a
/// [`GateRunResult`].
///
/// `walked` is the walk, already done and already decided by the composition root, the same
/// reason `nomos_check_orchestration::Run` takes `sources` rather than a root to read:
/// `None` for a root that was not a directory, `Some(sources)` otherwise -- including the
/// empty case, so the no-source-found decision stays visible to a caller rather than
/// collapsing into `Some` versus `None`. `variant` and `launcher` cross to
/// [`nomos_check_orchestration::Run`] unchanged; see its own documentation for why each is a
/// composition-root value this crate cannot compute for itself.
#[must_use]
pub fn Run_Gate<P: ProcessLauncher>(walked: Option<Vec<SourceFile>>, variant: BuildVariant, root: &Path, launcher: &P) -> GateRunResult
{
    let outcome = match walked
    {
        None => CheckOutcome::Unreadable,
        Some(sources) if sources.is_empty() => CheckOutcome::NoSource,
        Some(sources) => nomos_check_orchestration::Run(&sources, variant, root, launcher),
    };

    let (blocking_findings, disposition) = Reduced(&outcome);

    return GateRunResult { root: root.to_path_buf(), check_outcome: outcome, blocking_findings, disposition };
}

/// The blocking findings and the disposition they imply, read off a [`CheckOutcome`]
/// this function does not own and must not consume -- `check_outcome` still has to end up
/// in [`GateRunResult`] afterward.
fn Reduced(outcome: &CheckOutcome) -> (Vec<Finding>, GateRunOutcome)
{
    let CheckOutcome::Judged { findings, .. } = outcome
    else
    {
        return (Vec::new(), GateRunOutcome::Indeterminate);
    };

    let disposition = Disposition(findings);
    let blocking_findings: Vec<Finding> = findings
        .iter()
        .filter(|finding| return finding.Can_Fail_A_Build())
        .cloned()
        .collect();

    return (blocking_findings, disposition);
}
