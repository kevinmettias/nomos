//! Walking a tree, judging it, and reducing the judgment to a disposition -- the part of
//! `gate run` that lives here rather than in `nomos-gate-orchestration`.
//!
//! `nomos-gate-orchestration` and `nomos-check-orchestration` are both band 40
//! (`README.md`), and `tests/contract/tests/boundaries/graph.rs`'s
//! `Test_Dependencies_Should_Run_Strictly_Downward` requires a dependency's band to be
//! strictly less than its dependent's -- so `nomos-gate-orchestration` may not depend on
//! `nomos-check-orchestration`, and cannot own a type that carries `CheckOutcome`.
//! `nomos-cli` is band 90 and already depends on both, so [`GateRunResult`] and the call
//! into `nomos_check_orchestration::Run` live here, the same way `check.rs` itself is
//! where a tree gets walked and a build variant gets read: `nomos-check-orchestration`
//! cannot do either from inside its own band either. What genuinely belongs to
//! `nomos-gate-orchestration` -- the [`nomos_gate_orchestration::GateRunOutcome`] type and
//! the `Failed`-if-any-blocking-finding reduction -- is called from here, not reimplemented
//! here.

use super::composition::Host_Variant;
use super::sources::Walked;
use super::{GateCommand, PathBuf};
use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::Finding;
use nomos_gate_orchestration::GateRunOutcome;

/// What a real `nomos gate run` produced.
///
/// `check_outcome` is carried in full -- including `Claim`, for information only, the same
/// choice `OD-COMPLETENESS-004` already made for `nomos check`'s own exit code -- so a
/// caller that wants the finer detail behind `disposition` does not have to re-walk or
/// re-judge anything to get it.
pub(super) struct GateRunResult
{
    /// The tree this run judged.
    pub(super) root: PathBuf,
    /// What `nomos_check_orchestration::Run` (or the walk decision made before it was ever
    /// called) produced.
    pub(super) check_outcome: CheckOutcome,
    /// Exactly the findings for which `Finding::Can_Fail_A_Build` is true. Empty whenever
    /// `disposition` is not [`GateRunOutcome::Failed`].
    pub(super) blocking_findings: Vec<Finding>,
    /// The reduced verdict.
    pub(super) disposition: GateRunOutcome,
}

/// Walks `command.root`, judges it exactly as `nomos check` would, and reduces the result
/// to a [`GateRunResult`].
///
/// `Judged` reduces through `nomos_gate_orchestration::Disposition`, this crate's one real
/// consumer of that function. Every other `CheckOutcome` variant -- the tree could not be
/// read, this build's own capability registry was self-contradictory, or no source or no
/// fact was found to judge -- becomes [`GateRunOutcome::Indeterminate`] directly: none of
/// those four conditions produced a list of findings for `Disposition` to reduce, and
/// `Disposition` itself cannot see `CheckOutcome` at all (see this module's own doc).
#[must_use]
pub(super) fn Run_Gate(command: &GateCommand) -> GateRunResult
{
    let outcome = match Walked(&command.root)
    {
        None => CheckOutcome::Unreadable,
        Some(sources) if sources.is_empty() => CheckOutcome::NoSource,
        Some(sources) => nomos_check_orchestration::Run(&sources, Host_Variant(), &command.root),
    };

    let (blocking_findings, disposition) = Reduced(&outcome);

    return GateRunResult { root: command.root.clone(), check_outcome: outcome, blocking_findings, disposition };
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

    let disposition = nomos_gate_orchestration::Disposition(findings);
    let blocking_findings: Vec<Finding> = findings
        .iter()
        .filter(|finding| return finding.Can_Fail_A_Build())
        .cloned()
        .collect();

    return (blocking_findings, disposition);
}
