//! What a real `nomos gate run` judged, apart from the check facts that produced it.

use nomos_contracts::Finding;

/// What a real `nomos gate run` judged, apart from the check facts that produced it.
///
/// Deliberately three variants and no more. `ScopeSelector` and `RuleSelector` narrow what
/// feeds [`Disposition`] rather than adding a variant of their own; `ARC-ROADMAP-001`'s
/// remaining policy types (`CoveragePolicy`, `BaselinePolicy`, `SuppressionPolicy`) still do
/// not exist, so nothing here varies by them. `Indeterminate`
/// must never collapse into `Passed`: a run this gate could not judge is not the same
/// claim as a run it judged clean, the same distinction `nomos_check_orchestration::
/// CheckOutcome::Vacuous`-shaped conditions already refuse to blur one layer down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateRunOutcome
{
    /// No finding this run saw can fail a build.
    Passed,
    /// At least one finding can fail a build.
    Failed,
    /// The check behind this run could not produce an authoritative judgment -- the tree
    /// could not be read, this build's own capability registry was self-contradictory, or
    /// no source or no fact was found to judge. [`Disposition`] never returns this variant;
    /// [`crate::Run_Gate`] assigns it directly for a [`nomos_check_orchestration::
    /// CheckOutcome`] that never reached `Judged`, the one condition `Disposition` itself
    /// cannot see because it reduces a list of findings, not the outcome that produced one.
    Indeterminate,
}

/// `Failed` if any finding can fail a build, `Passed` otherwise.
///
/// Never returns [`GateRunOutcome::Indeterminate`] -- that variant answers a question
/// about the check run itself (did it reach a judgment at all), which a list of findings
/// that already exist cannot represent by construction; a caller assigns `Indeterminate`
/// directly for the conditions under which no findings were ever produced. `Claim`
/// (coverage debt / agent-required subjects) is deliberately not consulted here, the same
/// choice `OD-COMPLETENESS-004` already made for `nomos check`'s own exit code: reported,
/// not gated on.
#[must_use]
pub fn Disposition(findings: &[Finding]) -> GateRunOutcome
{
    let blocking = findings.iter().any(|finding| return finding.Can_Fail_A_Build());

    return if blocking { GateRunOutcome::Failed } else { GateRunOutcome::Passed };
}
