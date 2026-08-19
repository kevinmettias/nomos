//! What a `nomos gate` run produced.
//!
//! One variant is real today. The others do not exist as variants at all, the same
//! "no invented shape ahead of a real body" choice [`crate::command`] documents for the verbs
//! themselves -- an outcome variant with nothing yet to carry would be exactly the empty seam
//! this workspace has learned not to build ahead of a second real case.

use nomos_contracts::Finding;
use nomos_rules::{RuleOffer, RuleRegistryError};

/// What a gate would evaluate, without evaluating it.
///
/// `rules` is every rule [`crate::composition::Registered`] holds, in [`nomos_rules::
/// RuleRegistry::Offers`]'s own order -- [`nomos_contracts::RuleId`] order, so two runs over
/// the same registration agree without depending on a hasher. It does not yet vary by
/// [`crate::GateCommand::root`] or filter by scope or rule: `ScopeSelector` and
/// `RuleSelector`, the two policy types `ARC-ROADMAP-001` names first, do not exist yet, and
/// this increment reports the one thing that is real without pretending to filter it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GatePlan
{
    /// Every rule this gate's registry holds.
    pub rules: Vec<RuleOffer>,
}

/// What a `nomos gate` run produced.
pub enum GateOutcome
{
    /// The plan: what this gate's rule registry holds.
    Planned(GatePlan),
    /// This crate's own rule composition is self-contradictory -- a defect in the
    /// composition, not in anything a caller supplied. Not reachable today; see
    /// [`crate::composition::Registered`]'s own doc.
    Contradictory(RuleRegistryError),
}

/// What a real `nomos gate run` judged, apart from the check facts that produced it.
///
/// Deliberately three variants and no more -- `ARC-ROADMAP-001`'s other policy types
/// (`ScopeSelector`, `RuleSelector`, `CoveragePolicy`, `BaselinePolicy`,
/// `SuppressionPolicy`) do not exist yet, so nothing here varies by them. `Indeterminate`
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
    /// a caller assigns it for a [`nomos_check_orchestration::CheckOutcome`] that never
    /// reached `Judged`, which this crate does not depend on and cannot see -- both
    /// `nomos-gate-orchestration` and `nomos-check-orchestration` are band 40, and a band
    /// may not depend on its own band.
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
